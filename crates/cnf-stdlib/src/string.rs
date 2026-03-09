//! String utilities for CENTRA-NF

/// Mengecek apakah string kosong
pub fn is_empty(s: &str) -> bool {
    s.is_empty()
}

/// Panjang string
pub fn length(s: &str) -> usize {
    s.len()
}

/// Konversi ke huruf besar
pub fn to_upper(s: &str) -> String {
    s.to_uppercase()
}

/// Konversi ke huruf kecil
pub fn to_lower(s: &str) -> String {
    s.to_lowercase()
}

/// Trim whitespace
pub fn trim(s: &str) -> &str {
    s.trim()
}

/// Substring (mulai, panjang)
pub fn substring(s: &str, start: usize, len: usize) -> &str {
    let end = usize::min(start + len, s.len());
    &s[start..end]
}

/// Pad kiri
pub fn pad_left(s: &str, width: usize, pad: char) -> String {
    if s.len() >= width {
        s.to_string()
    } else {
        let mut out = String::new();
        for _ in 0..(width - s.len()) {
            out.push(pad);
        }
        out.push_str(s);
        out
    }
}

/// Pad kanan
pub fn pad_right(s: &str, width: usize, pad: char) -> String {
    if s.len() >= width {
        s.to_string()
    } else {
        let mut out = String::from(s);
        for _ in 0..(width - s.len()) {
            out.push(pad);
        }
        out
    }
}

/// Split string dengan delimiter
pub fn split(s: &str, delim: char) -> Vec<&str> {
    s.split(delim).collect()
}

/// Parse integer dari string
pub fn parse_int(s: &str) -> Result<i64, std::num::ParseIntError> {
    s.trim().parse::<i64>()
}

/// Format template sederhana: ganti {0}, {1}, ...
pub fn format_template(template: &str, args: &[&str]) -> String {
    let mut out = template.to_string();
    for (i, val) in args.iter().enumerate() {
        let pat = format!("{{{}}}", i);
        out = out.replace(&pat, val);
    }
    out
}

/// Reverse string
pub fn reverse(s: &str) -> String {
    s.chars().rev().collect()
}

/// Find substring, return index
pub fn find(haystack: &str, needle: &str) -> Option<usize> {
    haystack.find(needle)
}

/// Replace substring
pub fn replace(s: &str, from: &str, to: &str) -> String {
    s.replace(from, to)
}

/// Remove whitespace
pub fn remove_whitespace(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Capitalize kata pertama
pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

/// Join string array dengan delimiter
pub fn join(arr: &[&str], delim: &str) -> String {
    arr.join(delim)
}
