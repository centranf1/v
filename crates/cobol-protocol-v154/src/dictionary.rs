use std::sync::Arc;
use itertools::Itertools;
use sha2::{Digest, Sha256};
use crate::error::{CsmError, MAX_ENTRY_LEN, MAX_DICT_SYMBOLS};

const PHF_SIZE: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
enum PhfSlot {
    Empty,
    Deleted,
    Occupied { symbol: u16, value: Arc<[u8]> },
}

/// Single layer hash dictionary (linear probing PHF, zero-copy for loops).
#[derive(Debug, Clone)]
struct DictLayer {
    slots: Vec<PhfSlot>,
    by_first_byte: Vec<Vec<(u16, Arc<[u8]>)>>,
    count: usize,
}

impl DictLayer {
    fn new(cap: usize) -> Self {
        let mut by_first_byte = Vec::with_capacity(256);
        by_first_byte.resize_with(256, Vec::new);
        Self { slots: vec![PhfSlot::Empty; cap], by_first_byte, count: 0 }
    }

    fn insert(&mut self, symbol:u16, value:&[u8]) -> Result<(), CsmError> {
        if value.len() > MAX_ENTRY_LEN { return Err(CsmError::EntryTooLong(MAX_ENTRY_LEN)); }
        if self.count >= MAX_DICT_SYMBOLS { return Err(CsmError::MaxSymbols); }

        let arc: Arc<[u8]> = Arc::from(value);
        let mut slot = symbol as usize % self.slots.len();
        let mut first_deleted = None;
        for _ in 0..self.slots.len() {
            match &self.slots[slot] {
                PhfSlot::Empty => {
                    let idx = first_deleted.unwrap_or(slot);
                    self.slots[idx] = PhfSlot::Occupied { symbol, value: arc.clone() };
                    self.count += 1;
                    if let Some(b) = value.first() {
                        let bucket = &mut self.by_first_byte[*b as usize];
                        let pos = bucket.partition_point(|e| e.1.len() > value.len());
                        bucket.insert(pos, (symbol, arc.clone()));
                    }
                    return Ok(());
                }
                PhfSlot::Deleted => { first_deleted = first_deleted.or(Some(slot)); }
                PhfSlot::Occupied { symbol: s, .. } if *s == symbol => {
                    if let Some(b) = value.first() {
                        let bucket = &mut self.by_first_byte[*b as usize];
                        if let Some(idx) = bucket.iter().position(|(sym, _)| *sym == symbol) {
                            bucket.remove(idx);
                        }
                        let pos = bucket.partition_point(|e| e.1.len() > value.len());
                        bucket.insert(pos, (symbol, arc.clone()));
                    }
                    self.slots[slot] = PhfSlot::Occupied { symbol, value: arc.clone() };
                    return Ok(());
                }
                _ => {}
            }
            slot = (slot + 1) % self.slots.len();
        }
        Err(CsmError::MaxSymbols)
    }

    #[inline(always)]
    fn lookup(&self, symbol:u16) -> Option<&[u8]> {
        let mut slot = symbol as usize % self.slots.len();
        for _ in 0..self.slots.len() {
            match &self.slots[slot] {
                PhfSlot::Empty => return None,
                PhfSlot::Deleted => {},
                PhfSlot::Occupied { symbol: s, value } if *s == symbol => return Some(&**value),
                _ => {}
            }
            slot = (slot + 1) % self.slots.len();
        }
        None
    }

    fn remove(&mut self, symbol:u16) -> bool {
        let mut slot = symbol as usize % self.slots.len();
        for _ in 0..self.slots.len() {
            match &self.slots[slot] {
                PhfSlot::Empty => return false,
                PhfSlot::Deleted => {},
                PhfSlot::Occupied { symbol: s, value } if *s == symbol => {
                    if let Some(b) = value.first() {
                        let bucket = &mut self.by_first_byte[*b as usize];
                        if let Some(idx) = bucket.iter().position(|(sym, _)| *sym == symbol) {
                            bucket.remove(idx);
                        }
                    }
                    self.slots[slot] = PhfSlot::Deleted;
                    self.count -= 1;
                    return true;
                }
                _ => {}
            }
            slot = (slot + 1) % self.slots.len();
        }
        false
    }

    fn iter(&self) -> impl Iterator<Item = (u16, &[u8])> + '_ {
        self.slots.iter().filter_map(|slot| {
            if let PhfSlot::Occupied { symbol, value } = slot {
                Some((*symbol, &**value))
            } else {
                None
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LookupOrigin {
    Local,
    Global,
}

#[derive(Debug, Clone)]
pub struct CsmDictionary {
    global: DictLayer,
    local: DictLayer,
    pub checksum: [u8; 32],
}

impl Default for CsmDictionary {
    fn default() -> Self {
        Self::new()
    }
}

pub const LOOKUP_ORIGIN_FLAG: u16 = 0x4000; // 0 = global, 1 = local (example atomik flag)

impl CsmDictionary {
    pub fn new() -> Self {
        Self {
            global: DictLayer::new(PHF_SIZE),
            local: DictLayer::new(PHF_SIZE),
            checksum: [0u8; 32],
        }
    }

    pub fn insert_global(&mut self, symbol:u16, value:&[u8]) -> Result<(), CsmError> {
        let r = self.global.insert(symbol, value);
        self.update_checksum();
        r
    }

    pub fn insert_local(&mut self, symbol:u16, value:&[u8]) -> Result<(), CsmError> {
        let r = self.local.insert(symbol, value);
        self.update_checksum();
        r
    }

    pub fn insert(&mut self, symbol:u16, value:&[u8]) -> Result<(), CsmError> {
        // Default action per-frame: local
        self.insert_local(symbol, value)
    }

    #[inline(always)]
    pub fn lookup(&self, symbol:u16) -> Option<&[u8]> {
        self.local.lookup(symbol).or_else(|| self.global.lookup(symbol))
    }

    /// Lookup atomically set flag bit in returned symbol id.
    pub fn lookup_atomic(&self, symbol:u16) -> Option<(u16, LookupOrigin, &[u8])> {
        if let Some(value) = self.local.lookup(symbol) {
            return Some((symbol | LOOKUP_ORIGIN_FLAG, LookupOrigin::Local, value));
        }
        if let Some(value) = self.global.lookup(symbol) {
            return Some((symbol & !LOOKUP_ORIGIN_FLAG, LookupOrigin::Global, value));
        }
        None
    }

    pub fn remove(&mut self, symbol:u16) -> bool {
        let removed = self.local.remove(symbol) || self.global.remove(symbol);
        if removed { self.update_checksum(); }
        removed
    }

    pub fn reset_local(&mut self) {
        self.local = DictLayer::new(PHF_SIZE);
        self.update_checksum();
    }

    pub fn iter(&self) -> impl Iterator<Item = (u16, &[u8])> + '_ {
        let global_iter = self.global.iter().map(|(sym, data)| (sym, data));
        let local_iter = self.local.iter();

        // Local takes precedence if symbols duplicate.
        let mut combined: Vec<(u16, &[u8])> = global_iter.collect();
        for (sym, data) in local_iter {
            combined.retain(|(s, _)| *s != sym);
            combined.push((sym, data));
        }

        combined.into_iter().sorted_by_key(|(sym, _)| *sym)
    }

    /// Return dictionary entries starting with given first byte, using bucket index.
    pub fn candidates_for_byte(&self, first_byte: u8) -> Vec<(u16, &[u8])> {
        let mut cands = Vec::new();
        let bucket = first_byte as usize;
        // Local entries first (higher precedence)
        for (sym, entry) in &self.local.by_first_byte[bucket] {
            cands.push((*sym, &**entry));
        }
        // Global entries only if not shadowed by local
        for (sym, entry) in &self.global.by_first_byte[bucket] {
            if self.local.lookup(*sym).is_none() {
                cands.push((*sym, &**entry));
            }
        }
        cands
    }

    pub fn verify_checksum(&self) -> bool {
        self.checksum == Self::compute_checksum(self)
    }

    fn update_checksum(&mut self) {
        self.checksum = Self::compute_checksum(self);
    }

    fn compute_checksum(dict:&CsmDictionary) -> [u8; 32] {
        let mut entries: Vec<_> = dict.iter().collect();
        entries.sort_by_key(|(k, _)| *k);
        let mut hasher = Sha256::new();
        for (k, v) in entries {
            hasher.update(k.to_be_bytes());
            hasher.update(v);
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&hasher.finalize());
        arr
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_hierarchical_lookup() {
        let mut dict = CsmDictionary::new();
        dict.insert_global(1, b"global").unwrap();
        dict.insert_local(1, b"local").unwrap();
        assert_eq!(dict.lookup(1), Some(b"local" as &[u8]));
        dict.reset_local();
        assert_eq!(dict.lookup(1), Some(b"global" as &[u8]));
    }
}



