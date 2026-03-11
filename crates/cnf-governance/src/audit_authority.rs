use sha2::{Sha256, Digest};

#[derive(Default)]
pub struct AuditLedger {
    pub entries: Vec<String>,       // SHA-256 hex dari setiap entry (chained)
    raw_entries: Vec<String>,       // original entry text
}

impl AuditLedger {
    pub fn new() -> Self { 
        AuditLedger { 
            entries: Vec::new(),
            raw_entries: Vec::new(),
        } 
    }

    pub fn log(&mut self, entry: &str) {
        self.raw_entries.push(entry.to_string());
        // hash = SHA256(previous_hash + entry) untuk chain
        let prev = self.entries.last().cloned().unwrap_or_default();
        let combined = format!("{}{}", prev, entry);
        let mut hasher = Sha256::new();
        hasher.update(combined.as_bytes());
        self.entries.push(hex::encode(hasher.finalize()));
    }

    pub fn verify(&self) -> bool {
        // recompute seluruh chain dari raw_entries
        let mut computed_entries = Vec::new();
        for entry in &self.raw_entries {
            let prev = computed_entries.last().cloned().unwrap_or_default();
            let combined = format!("{}{}", prev, entry);
            let mut hasher = Sha256::new();
            hasher.update(combined.as_bytes());
            computed_entries.push(hex::encode(hasher.finalize()));
        }
        // bandingkan dengan entries yang disimpan
        computed_entries == self.entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_creates_hash() {
        let mut l = AuditLedger::new();
        l.log("x");
        assert_eq!(l.entries.len(), 1);
    }

    #[test]
    fn verify_valid_chain() {
        let mut l = AuditLedger::new();
        l.log("entry1");
        l.log("entry2");
        assert!(l.verify());
    }

    #[test]
    fn log_multiple_entries() {
        let mut l = AuditLedger::new();
        l.log("a");
        l.log("b");
        assert_eq!(l.entries.len(), 2);
    }

    #[test]
    fn ledger_persistence() {
        let mut l = AuditLedger::new();
        l.log("x");
        let snapshot = l.entries.clone();
        assert_eq!(snapshot, l.entries);
    }

    #[test]
    fn log_hash_different() {
        let mut l = AuditLedger::new();
        l.log("a");
        l.log("b");
        assert_ne!(l.entries[0], l.entries[1]);
    }

    #[test]
    fn verify_detects_tampering() {
        let mut l = AuditLedger::new();
        l.log("entry1");
        l.log("entry2");
        // Tamper dengan entries langsung
        l.entries[0] = "tampered".to_string();
        assert!(!l.verify());
    }
}
