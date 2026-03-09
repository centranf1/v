//! Environment utilities for CENTRA-NF

/// Deteksi OS
pub fn os() -> &'static str {
    std::env::consts::OS
}

/// Deteksi arsitektur
pub fn arch() -> &'static str {
    std::env::consts::ARCH
}

/// Ambil env var (deterministik: fallback kosong jika tidak ada)
pub fn get_env(key: &str) -> String {
    std::env::var(key).unwrap_or_default()
}

/// Hitung jumlah CPU
pub fn cpu_count() -> usize {
    num_cpus::get()
}

/// Info runtime
pub struct RuntimeInfo {
    pub os: &'static str,
    pub arch: &'static str,
    pub cpu_count: usize,
}

impl RuntimeInfo {
    pub fn gather() -> Self {
        Self {
            os: os(),
            arch: arch(),
            cpu_count: cpu_count(),
        }
    }
}
