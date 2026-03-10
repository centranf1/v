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
pub use string::{is_empty as string_is_empty, length, to_upper, to_lower, trim, substring, pad_left, pad_right, split, parse_int, format_template, reverse, find as string_find, replace, remove_whitespace, capitalize, join};
pub use buffer::{is_empty as buffer_is_empty, size, zeros, hex_encode, hex_decode, concat, slice, xor, adler32, find_pattern};
pub use collection::{find as collection_find, count, filter, map, unique, chunks, zip, sum, mean};
pub use math::*;
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
