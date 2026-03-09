//! Standard Library for CENTRA-NF
//!
//! Built-in functions and utilities for common operations.

pub mod string;
pub mod buffer;
pub mod math;
pub mod collection;
pub mod io;
pub mod convert;
pub mod compress;
pub mod integrity;
pub mod crypto;
pub mod format;
pub mod time;
pub mod env;

// Re-export utama (opsional, bisa diatur sesuai kebutuhan)
pub use string::*;
pub use buffer::*;
pub use math::*;
pub use collection::*;
pub use io::*;
pub use convert::*;
pub use compress::*;
pub use integrity::*;
pub use crypto::*;
pub use format::*;
pub use time::*;
pub use env::*;

#[cfg(test)]
mod tests {
    // Pengujian akan dipindahkan ke modul per domain
}
