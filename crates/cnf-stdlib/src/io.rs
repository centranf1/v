//! IO utilities for CENTRA-NF

/// Simulasi DISPLAY (output ke buffer, bukan stdout langsung)
pub fn display(msg: &str) -> String {
    format!("[DISPLAY] {}", msg)
}

/// Simulasi PRINT (output ke buffer, bukan stdout langsung)
pub fn print(msg: &str) -> String {
    format!("[PRINT] {}", msg)
}

/// Simulasi READ (input dari buffer, bukan stdin langsung)
pub fn read(input: &str) -> String {
    input.to_string()
}
