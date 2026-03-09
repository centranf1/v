use std::collections::HashMap;
use std::sync::Arc;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsmDictionary {
    map: HashMap<u16, Arc<[u8]>>,
    checksum: [u8; 32],
}

impl CsmDictionary {
    pub fn new() -> Self {
        Self { map: HashMap::new(), checksum: [0; 32] }
    }

    pub fn insert(&mut self, symbol: u16, value: &[u8]) {
        self.map.insert(symbol, Arc::from(value));
        self.update_checksum();
    }

    pub fn lookup(&self, symbol: u16) -> Option<&[u8]> {
        self.map.get(&symbol).map(|arc| &**arc)
    }

    pub fn verify_checksum(&self) -> bool {
        self.checksum == Self::compute_checksum(&self.map)
    }

    pub fn checksum(&self) -> [u8; 32] {
        self.checksum
    }

    fn update_checksum(&mut self) {
        self.checksum = Self::compute_checksum(&self.map);
    }

    fn compute_checksum(map: &HashMap<u16, Arc<[u8]>>) -> [u8; 32] {
        let mut entries: Vec<_> = map.iter().collect();
        entries.sort_by_key(|(k, _)| *k);
        let mut hasher = Sha256::new();
        for (k, v) in entries {
            hasher.update(&k.to_be_bytes());
            hasher.update(&v);
        }
        let result = hasher.finalize();
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&result);
        arr
    }
}
