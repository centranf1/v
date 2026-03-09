//! Math utilities for CENTRA-NF

/// Nilai maksimum dua i64
pub fn max(a: i64, b: i64) -> i64 {
    if a > b { a } else { b }
}

/// Nilai minimum dua i64
pub fn min(a: i64, b: i64) -> i64 {
    if a < b { a } else { b }
}

/// Nilai mutlak
pub fn abs(a: i64) -> i64 {
    if a < 0 { -a } else { a }
}

/// Penjumlahan checked
pub fn checked_add(a: i64, b: i64) -> Option<i64> {
    a.checked_add(b)
}

/// Pengurangan checked
pub fn checked_sub(a: i64, b: i64) -> Option<i64> {
    a.checked_sub(b)
}

/// Perkalian checked
pub fn checked_mul(a: i64, b: i64) -> Option<i64> {
    a.checked_mul(b)
}

/// Pembagian checked
pub fn checked_div(a: i64, b: i64) -> Option<i64> {
    a.checked_div(b)
}

/// Akar kuadrat (f64)
pub fn sqrt(x: f64) -> f64 {
    x.sqrt()
}

/// Logaritma natural (f64)
pub fn ln(x: f64) -> f64 {
    x.ln()
}

/// Logaritma basis 10 (f64)
pub fn log10(x: f64) -> f64 {
    x.log10()
}

/// Pembulatan ke n digit
pub fn round_to(x: f64, digits: u32) -> f64 {
    let factor = 10f64.powi(digits as i32);
    (x * factor).round() / factor
}
