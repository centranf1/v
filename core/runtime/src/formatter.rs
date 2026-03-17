//! Format string interpolation and variable substitution
//!
//! Supports:
//! - Variable substitution: {VAR_NAME}
//! - Numeric formatting: {VAR:hex}, {VAR:upper}, {VAR:lower}
//! - Padding: {VAR:5} = pad to 5 chars
//! - Alignment: {VAR:left}, {VAR:right}, {VAR:center}
//! - Escape sequences: \n, \t, \\, \\{, \\}

use std::collections::HashMap;

/// Format string with variable substitution
pub fn format_display(
    template: &str,
    variables: &HashMap<String, String>,
) -> Result<String, String> {
    let mut result = String::new();
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                // Handle escape sequences
                if let Some(next) = chars.next() {
                    match next {
                        'n' => result.push('\n'),
                        't' => result.push('\t'),
                        'r' => result.push('\r'),
                        '\\' => result.push('\\'),
                        '{' => result.push('{'),
                        '}' => result.push('}'),
                        _ => {
                            result.push('\\');
                            result.push(next);
                        }
                    }
                } else {
                    result.push('\\');
                }
            }
            '{' => {
                // Parse variable substitution or literal brace
                if chars.peek() == Some(&'{') {
                    // Escaped opening brace
                    chars.next();
                    result.push('{');
                } else {
                    // Variable substitution: {VAR_NAME} or {VAR_NAME:format}
                    let substitution = parse_variable_expr(&mut chars, variables)?;
                    result.push_str(&substitution);
                }
            }
            _ => result.push(ch),
        }
    }

    Ok(result)
}

/// Parse a variable expression like {VAR}, {VAR:hex}, {VAR:left:5}
fn parse_variable_expr(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    variables: &HashMap<String, String>,
) -> Result<String, String> {
    let mut expr = String::new();

    // Parse until closing brace
    while let Some(&ch) = chars.peek() {
        if ch == '}' {
            chars.next();
            break;
        }
        expr.push(chars.next().unwrap());
    }

    if expr.is_empty() {
        return Err("Empty variable name in format string".to_string());
    }

    // Parse variable name and format specifiers
    let parts: Vec<&str> = expr.split(':').collect();
    if parts.is_empty() {
        return Err("Invalid variable expression".to_string());
    }

    let var_name = parts[0];

    // Parse format specifiers, handling those with parameters
    let mut format_specs = Vec::new();
    let mut i = 1;
    while i < parts.len() {
        let spec = parts[i];
        // Check if this specifier needs a parameter
        if matches!(spec, "left" | "right" | "center" | "pad") {
            if i + 1 < parts.len() {
                // Combine specifier with its parameter
                format_specs.push(format!("{}:{}", spec, parts[i + 1]));
                i += 2;
            } else {
                return Err(format!("Format specifier '{}' requires a parameter", spec));
            }
        } else {
            format_specs.push(spec.to_string());
            i += 1;
        }
    }

    // Get variable value
    let mut value = variables
        .get(var_name)
        .cloned()
        .ok_or_else(|| format!("Undefined variable in format string: {}", var_name))?;

    // Apply format specifiers in order
    for spec in format_specs {
        value = apply_format_spec(&value, &spec)?;
    }

    Ok(value)
}

/// Apply a single format specifier to a value
fn apply_format_spec(value: &str, spec: &str) -> Result<String, String> {
    match spec {
        "upper" => Ok(value.to_uppercase()),
        "lower" => Ok(value.to_lowercase()),
        "uppercase" => Ok(value.to_uppercase()),
        "lowercase" => Ok(value.to_lowercase()),
        "hex" => {
            // Convert to hex representation
            Ok(format!(
                "0x{}",
                value
                    .as_bytes()
                    .iter()
                    .map(|&b| format!("{:02x}", b))
                    .collect::<String>()
            ))
        }
        "length" | "len" => {
            // Return length of value
            Ok(value.len().to_string())
        }
        "trim" | "trimmed" => Ok(value.trim().to_string()),
        "reverse" => Ok(value.chars().rev().collect()),
        s if s.starts_with("pad:") => {
            // Pad to specified width: pad:10
            let width_str = &s[4..];
            let width = width_str
                .parse::<usize>()
                .map_err(|_| format!("Invalid padding width: {}", width_str))?;
            Ok(format!("{:width$}", value, width = width))
        }
        s if s.starts_with("left:") => {
            // Left-align to specified width: left:10
            let width_str = &s[5..];
            let width = width_str
                .parse::<usize>()
                .map_err(|_| format!("Invalid width: {}", width_str))?;
            Ok(format!("{:<width$}", value, width = width))
        }
        s if s.starts_with("right:") => {
            // Right-align to specified width: right:10
            let width_str = &s[6..];
            let width = width_str
                .parse::<usize>()
                .map_err(|_| format!("Invalid width: {}", width_str))?;
            Ok(format!("{:>width$}", value, width = width))
        }
        s if s.starts_with("center:") => {
            // Center-align to specified width: center:10
            let width_str = &s[7..];
            let width = width_str
                .parse::<usize>()
                .map_err(|_| format!("Invalid width: {}", width_str))?;
            let padding = (width as i32 - value.len() as i32).max(0) as usize;
            let left_pad = padding / 2;
            let right_pad = padding - left_pad;
            Ok(format!(
                "{}{}{}",
                " ".repeat(left_pad),
                value,
                " ".repeat(right_pad)
            ))
        }
        s if s.chars().all(|c| c.is_ascii_digit()) => {
            // Plain number: pad with spaces to width (default left-align)
            let width = s
                .parse::<usize>()
                .map_err(|_| format!("Invalid width specifier: {}", s))?;
            Ok(format!("{:<width$}", value, width = width))
        }
        _ => Err(format!("Unknown format specifier: {}", spec)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::HashMap;

    #[test]
    fn test_simple_variable_substitution() {
        let mut vars = HashMap::new();
        vars.insert("NAME".to_string(), "Alice".to_string());

        let result = format_display("Hello, {NAME}!", &vars).unwrap();
        assert_eq!(result, "Hello, Alice!");
    }

    #[test]
    fn test_multiple_variables() {
        let mut vars = HashMap::new();
        vars.insert("FIRST".to_string(), "John".to_string());
        vars.insert("LAST".to_string(), "Doe".to_string());

        let result = format_display("{FIRST} {LAST}", &vars).unwrap();
        assert_eq!(result, "John Doe");
    }

    #[test]
    fn test_uppercase_format() {
        let mut vars = HashMap::new();
        vars.insert("STATUS".to_string(), "ready".to_string());

        let result = format_display("Status: {STATUS:upper}", &vars).unwrap();
        assert_eq!(result, "Status: READY");
    }

    #[test]
    fn test_lowercase_format() {
        let mut vars = HashMap::new();
        vars.insert("CODE".to_string(), "ERROR".to_string());

        let result = format_display("{CODE:lower}", &vars).unwrap();
        assert_eq!(result, "error");
    }

    #[test]
    fn test_padding_format() {
        let mut vars = HashMap::new();
        vars.insert("NUM".to_string(), "42".to_string());

        let result = format_display("Value: |{NUM:pad:5}|", &vars).unwrap();
        assert_eq!(result, "Value: |42   |");
    }

    #[test]
    fn test_escape_sequences() {
        let mut vars = HashMap::new();
        vars.insert("VAR".to_string(), "test".to_string());

        let result = format_display("Line1\\nLine2\\tTabbed", &vars).unwrap();
        assert_eq!(result, "Line1\nLine2\tTabbed");
    }

    #[test]
    fn test_undefined_variable_error() {
        let vars = HashMap::new();
        let result = format_display("Hello {UNDEFINED}", &vars);
        assert!(result.is_err());
    }

    #[test]
    fn test_hex_format() {
        let mut vars = HashMap::new();
        vars.insert("DATA".to_string(), "ABC".to_string());

        let result = format_display("{DATA:hex}", &vars).unwrap();
        assert_eq!(result, "0x414243");
    }

    #[test]
    fn test_length_format() {
        let mut vars = HashMap::new();
        vars.insert("STR".to_string(), "hello".to_string());

        let result = format_display("Length: {STR:len}", &vars).unwrap();
        assert_eq!(result, "Length: 5");
    }

    #[test]
    fn test_left_align() {
        let mut vars = HashMap::new();
        vars.insert("VAL".to_string(), "x".to_string());

        let result = format_display("|{VAL:left:5}|", &vars).unwrap();
        assert_eq!(result, "|x    |");
    }

    #[test]
    fn test_right_align() {
        let mut vars = HashMap::new();
        vars.insert("VAL".to_string(), "x".to_string());

        let result = format_display("|{VAL:right:5}|", &vars).unwrap();
        assert_eq!(result, "|    x|");
    }

    #[test]
    fn test_complex_format_chain() {
        let mut vars = HashMap::new();
        vars.insert("TEXT".to_string(), "hello".to_string());

        let result = format_display("{TEXT:upper:left:8}", &vars).unwrap();
        assert_eq!(result, "HELLO   ");
    }

    // Property-based tests for comprehensive coverage
    proptest! {
        #[test]
        fn test_format_display_arbitrary_text(text in "\\PC*", width in 1..100usize) {
            let mut vars = HashMap::new();
            vars.insert("VAR".to_string(), text.clone());

            // Test basic substitution
            let result = format_display("{VAR}", &vars).unwrap();
            prop_assert_eq!(result, text.clone());

            // Test padding
            let pad_result = format_display(&format!("{{VAR:pad:{}}}", width), &vars).unwrap();
            let text_chars = text.chars().count();
            prop_assert!(pad_result.chars().count() >= text_chars);
            prop_assert!(pad_result.starts_with(&text));

            // Test left alignment
            let left_result = format_display(&format!("{{VAR:left:{}}}", width), &vars).unwrap();
            prop_assert_eq!(left_result.chars().count(), width.max(text_chars));
            prop_assert!(left_result.starts_with(&text));

            // Test right alignment
            let right_result = format_display(&format!("{{VAR:right:{}}}", width), &vars).unwrap();
            prop_assert_eq!(right_result.chars().count(), width.max(text_chars));
            prop_assert!(right_result.ends_with(&text));
        }

        #[test]
        fn test_format_specifier_chains(text in "\\PC*", count in 1..5usize) {
            let mut vars = HashMap::new();
            vars.insert("VAR".to_string(), text.clone());

            // Generate a chain of format specifiers
            let mut spec_chain = String::new();
            for i in 0..count {
                match i % 4 {
                    0 => spec_chain.push_str("upper:"),
                    1 => spec_chain.push_str("lower:"),
                    2 => spec_chain.push_str("left:10:"),
                    3 => spec_chain.push_str("right:15:"),
                    _ => unreachable!("i % 4 should always be 0-3"),
                }
            }
            // Remove trailing colon
            if spec_chain.ends_with(':') {
                spec_chain.pop();
            }

            let template = format!("{{VAR:{}}}", spec_chain);
            let result = format_display(&template, &vars);

            // Should not panic and should produce some result
            prop_assert!(result.is_ok());
            let formatted = result.unwrap();

            // Result should be non-empty for non-empty input
            if !text.is_empty() {
                prop_assert!(!formatted.is_empty());
            }
        }

        #[test]
        fn test_hex_format_arbitrary_bytes(bytes in prop::collection::vec(any::<u8>(), 0..100)) {
            let text = String::from_utf8_lossy(&bytes).to_string();
            let mut vars = HashMap::new();
            vars.insert("DATA".to_string(), text.clone());

            let result = format_display("{DATA:hex}", &vars).unwrap();

            // Should start with 0x
            prop_assert!(result.starts_with("0x"));

            // Should have correct length (2 chars for "0x" + 2 chars per byte in the UTF-8 representation)
            let expected_hex_len = text.len() * 2;
            prop_assert_eq!(result.len(), 2 + expected_hex_len);

            // Should only contain valid hex characters
            for ch in result[2..].chars() {
                prop_assert!(ch.is_ascii_hexdigit());
            }
        }

        #[test]
        fn test_length_format_consistency(text in "\\PC*") {
            let mut vars = HashMap::new();
            vars.insert("STR".to_string(), text.clone());

            let len_result = format_display("{STR:len}", &vars).unwrap();
            let expected_len = text.len().to_string();

            prop_assert_eq!(len_result, expected_len);
        }

        #[test]
        fn test_escape_sequences_preserve_other_text(prefix in "[^{}\\\\]*", suffix in "[^{}\\\\]*") {
            let mut vars = HashMap::new();
            vars.insert("VAR".to_string(), "test".to_string());

            let template = format!("{}{{VAR}}\n\t{}", prefix, suffix);
            let result = format_display(&template, &vars).unwrap();

            // Should contain the prefix, variable, escaped chars, and suffix
            prop_assert!(result.contains(&prefix));
            prop_assert!(result.contains("test"));
            prop_assert!(result.contains("\n"));
            prop_assert!(result.contains("\t"));
            prop_assert!(result.contains(&suffix));
        }
    }
}
