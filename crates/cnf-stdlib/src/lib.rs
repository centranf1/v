//! Standard Library for CENTRA-NF
//!
//! Built-in functions and utilities for common operations.

/// String utilities
pub mod string {
    /// Check if string is empty
    pub fn is_empty(s: &str) -> bool {
        s.is_empty()
    }

    /// Get string length 
    pub fn length(s: &str) -> usize {
        s.len()
    }

    /// Convert to uppercase
    pub fn to_upper(s: &str) -> String {
        s.to_uppercase()
    }

    /// Convert to lowercase
    pub fn to_lower(s: &str) -> String {
        s.to_lowercase()
    }

    /// Trim whitespace
    pub fn trim(s: &str) -> &str {
        s.trim()
    }
}

/// Buffer utilities
pub mod buffer {
    /// Get buffer size in bytes
    pub fn size(buf: &[u8]) -> usize {
        buf.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(buf: &[u8]) -> bool {
        buf.is_empty()
    }

    /// Create zero-filled buffer
    #[allow(dead_code)]
    pub fn zeros(size: usize) -> Vec<u8> {
        vec![0; size]
    }
}

/// Collection operations
pub mod collection {
    /// Count elements in collection
    pub fn count(items: &[&str]) -> usize {
        items.len()
    }

    /// Find element in collection
    pub fn find(items: &[&str], target: &str) -> Option<usize> {
        items.iter().position(|&x| x == target)
    }

    /// Filter collection by predicate (stub)
    #[allow(dead_code)]
    pub fn filter(items: Vec<&str>, _predicate: bool) -> Vec<&str> {
        // Stub: real implementation would apply predicate
        items
    }
}

/// Math utilities
pub mod math {
    /// Maximum of two values
    pub fn max(a: i64, b: i64) -> i64 {
        if a > b { a } else { b }
    }

    /// Minimum of two values
    pub fn min(a: i64, b: i64) -> i64 {
        if a < b { a } else { b }
    }

    /// Absolute value
    pub fn abs(a: i64) -> i64 {
        if a < 0 { -a } else { a }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_utilities() {
        assert!(string::is_empty(""));
        assert!(!string::is_empty("hello"));
        assert_eq!(string::length("test"), 4);
        assert_eq!(string::to_upper("hello"), "HELLO");
        assert_eq!(string::to_lower("HELLO"), "hello");
    }

    #[test]
    fn test_buffer_utilities() {
        let buf = b"hello";
        assert_eq!(buffer::size(buf), 5);
        assert!(!buffer::is_empty(buf));

        let empty: &[u8] = &[];
        assert!(buffer::is_empty(empty));
    }

    #[test]
    fn test_collection_utilities() {
        let items = vec!["a", "b", "c"];
        assert_eq!(collection::count(&items), 3);
        assert_eq!(collection::find(&items, "b"), Some(1));
        assert_eq!(collection::find(&items, "z"), None);
    }

    #[test]
    fn test_math_utilities() {
        assert_eq!(math::max(5, 3), 5);
        assert_eq!(math::min(5, 3), 3);
        assert_eq!(math::abs(-42), 42);
        assert_eq!(math::abs(42), 42);
    }
}
