impl Default for CsmDictionary {
    fn default() -> Self {
        Self::new()
    }
}

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


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsmDictionary {
    pub slots: Vec<PhfSlot>,
    pub by_first_byte: Vec<Vec<(u16, Arc<[u8]>)>>, // [256] bucket, each sorted by entry.len desc
    pub count: usize,
    pub checksum: [u8; 32],
}


impl CsmDictionary {
    pub fn new() -> Self {
        Self::with_capacity(PHF_SIZE)
    }

    pub fn with_capacity(n: usize) -> Self {
        let mut by_first_byte = Vec::with_capacity(256);
        by_first_byte.resize_with(256, Vec::new);
        Self {
            slots: vec![PhfSlot::Empty; PHF_SIZE.max(n)],
            by_first_byte,
            count: 0,
            checksum: [0; 32],
        }
    }


    pub fn insert(&mut self, symbol: u16, value: &[u8]) -> Result<(), CsmError> {
        eprintln!("[dict] insert symbol={} len={} value[..8]={:?}", symbol, value.len(), &value[..value.len().min(8)]);
        if value.len() > MAX_ENTRY_LEN {
            return Err(CsmError::EntryTooLong(MAX_ENTRY_LEN));
        }
        if self.count >= MAX_DICT_SYMBOLS {
            return Err(CsmError::MaxSymbols);
        }
        let arc: Arc<[u8]> = Arc::from(value);
        let mut slot = (symbol as usize) % PHF_SIZE;
        let mut first_deleted: Option<usize> = None;
        for _ in 0..PHF_SIZE {
            match &self.slots[slot] {
                PhfSlot::Empty => {
                    let insert_at = first_deleted.unwrap_or(slot);
                    self.slots[insert_at] = PhfSlot::Occupied { symbol, value: arc.clone() };
                    self.count += 1;
                    // Insert into by_first_byte, sorted by len desc
                    if let Some(b) = value.first() {
                        let bucket = &mut self.by_first_byte[*b as usize];
                        let pos = bucket.partition_point(|e| e.1.len() > value.len());
                        bucket.insert(pos, (symbol, arc.clone()));
                    }
                    self.update_checksum();
                    return Ok(());
                }
                PhfSlot::Deleted => {
                    if first_deleted.is_none() {
                        first_deleted = Some(slot);
                    }
                }
                PhfSlot::Occupied { symbol: s, .. } if *s == symbol => {
                    // Replace value
                    if let Some(b) = value.first() {
                        let bucket = &mut self.by_first_byte[*b as usize];
                        if let Some(idx) = bucket.iter().position(|(sym, _)| *sym == symbol) {
                            bucket.remove(idx);
                        }
                        let pos = bucket.partition_point(|e| e.1.len() > value.len());
                        bucket.insert(pos, (symbol, arc.clone()));
                    }
                    self.slots[slot] = PhfSlot::Occupied { symbol, value: arc.clone() };
                    self.update_checksum();
                    return Ok(());
                }
                _ => {}
            }
            slot = (slot + 1) % PHF_SIZE;
        }
        Err(CsmError::MaxSymbols)
    }


    pub fn lookup(&self, symbol: u16) -> Option<&[u8]> {
        let mut slot = (symbol as usize) % PHF_SIZE;
        for _ in 0..PHF_SIZE {
            match &self.slots[slot] {
                PhfSlot::Empty => {
                    eprintln!("[dict] lookup symbol={} -> not found (empty)", symbol);
                    return None;
                },
                PhfSlot::Deleted => {},
                PhfSlot::Occupied { symbol: s, value } if *s == symbol => {
                    eprintln!("[dict] lookup symbol={} -> found len={} value[..8]={:?}", symbol, value.len(), &value[..value.len().min(8)]);
                    return Some(&**value)
                },
                _ => {}
            }
            slot = (slot + 1) % PHF_SIZE;
        }
        eprintln!("[dict] lookup symbol={} -> not found (full scan)", symbol);
        None
    }

    pub fn remove(&mut self, symbol: u16) -> bool {
        let mut slot = (symbol as usize) % PHF_SIZE;
        for _ in 0..PHF_SIZE {
            match &self.slots[slot] {
                PhfSlot::Empty => return false,
                PhfSlot::Deleted => {},
                PhfSlot::Occupied { symbol: s, value } if *s == symbol => {
                    // Remove from by_first_byte
                    if let Some(b) = value.first() {
                        let bucket = &mut self.by_first_byte[*b as usize];
                        if let Some(idx) = bucket.iter().position(|(sym, _)| *sym == symbol) {
                            bucket.remove(idx);
                        }
                    }
                    self.slots[slot] = PhfSlot::Deleted;
                    self.count -= 1;
                    self.update_checksum();
                    return true;
                }
                _ => {}
            }
            slot = (slot + 1) % PHF_SIZE;
        }
        false
    }

    pub fn candidates_for_byte(&self, b: u8) -> impl Iterator<Item = (u16, &[u8])> + '_ {
        self.by_first_byte[b as usize]
            .iter()
            .map(|(sym, arc)| (*sym, &**arc))
    }

    pub fn iter(&self) -> impl Iterator<Item = (u16, &[u8])> + '_ {
        self.slots.iter().filter_map(|slot| {
            if let PhfSlot::Occupied { symbol, value } = slot {
                Some((*symbol, &**value))
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .into_iter()
        .sorted_by_key(|(sym, _)| *sym)
    }

    pub fn verify_checksum(&self) -> bool {
        self.checksum == Self::compute_checksum(self)
    }

    pub fn checksum(&self) -> [u8; 32] {
        self.checksum
    }

    fn update_checksum(&mut self) {
        self.checksum = Self::compute_checksum(self);
    }

    fn compute_checksum(dict: &CsmDictionary) -> [u8; 32] {
        let mut entries: Vec<_> = dict.iter().collect();
        entries.sort_by_key(|(k, _)| *k);
        let mut hasher = Sha256::new();
        for (k, v) in entries {
            hasher.update(k.to_be_bytes());
            hasher.update(v);
        }
        let result = hasher.finalize();
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&result);
        arr
    }
}
