use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use fxhash::FxHasher;

/// Represents a single field type in a template layout.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TemplateFieldType {
    IntegerSigned,
    IntegerUnsigned,
    Float64,
    Utf8String,
    Binary,
    Boolean,
}

/// Template untuk struktur data berlapis.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructTemplate {
    pub fields: Vec<TemplateFieldType>,
}

impl StructTemplate {
    pub fn new(fields: Vec<TemplateFieldType>) -> Self {
        Self { fields }
    }

    /// Hitung fingerprint 64-bit yang deterministik (FxHash) untuk template.
    pub fn fingerprint(&self) -> u64 {
        let mut hasher = FxHasher::default();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

/// Registry yang memetakan fingerprint ke ID template 14-bit.
#[derive(Debug, Default)]
pub struct TemplateRegistry {
    map: HashMap<u64, u16>,
    next_id: u16,
}

impl TemplateRegistry {
    pub fn new() -> Self {
        Self { map: HashMap::default(), next_id: 0 }
    }

    /// Daftarkan template baru. Kembalikan `u16` ID atau ID yang sudah ada.
    pub fn register(&mut self, template: &StructTemplate) -> Result<u16, String> {
        let fp = template.fingerprint();
        if let Some(&id) = self.map.get(&fp) {
            return Ok(id);
        }

        if self.next_id >= 0x3FFF {
            return Err("Template registry full".to_string());
        }

        let id = self.next_id;
        self.map.insert(fp, id);
        self.next_id += 1;
        Ok(id)
    }

    pub fn lookup(&self, template: &StructTemplate) -> Option<u16> {
        self.map.get(&template.fingerprint()).copied()
    }
}

pub const TEMPLATE_FLAG: u16 = 0x4000;
pub const SYMBOL_FLAG: u16 = 0x8000;

/// Encode template token: ID + template flag.
pub fn template_token(id: u16) -> u16 {
    TEMPLATE_FLAG | (id & !TEMPLATE_FLAG)
}

/// Decode template token.
pub fn decode_template_token(token: u16) -> Option<u16> {
    if token & TEMPLATE_FLAG != 0 {
        Some(token & !TEMPLATE_FLAG)
    } else {
        None
    }
}
