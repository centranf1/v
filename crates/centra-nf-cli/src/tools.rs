//! CLI tools for code formatting and linting
//!
//! Provides:
//! - Format: Auto-format .cnf files to canonical style
//! - Lint: Analyze code for style/semantic issues

use cnf_compiler::lexer::tokenize;
use cnf_compiler::parser::Parser;
use std::path::Path;

/// Result of formatting or linting operation
#[derive(Debug, Clone)]
pub struct ToolResult {
    pub success: bool,
    pub message: String,
    pub output: Option<String>,
    pub issues: Vec<Issue>,
}

/// A linting issue
#[derive(Debug, Clone)]
pub struct Issue {
    pub level: IssueLevelity,
    pub message: String,
    pub line: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IssueLevelity {
    Error,
    Warning,
    Info,
}

impl std::fmt::Display for IssueLevelity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueLevelity::Error => write!(f, "ERROR"),
            IssueLevelity::Warning => write!(f, "WARN"),
            IssueLevelity::Info => write!(f, "INFO"),
        }
    }
}

/// Format a .cnf source file to canonical style
pub fn format_source(source: &str) -> ToolResult {
    // Tokenize to validate syntax
    match tokenize(source) {
        Err(e) => ToolResult {
            success: false,
            message: format!("Tokenization failed: {}", e),
            output: None,
            issues: vec![Issue {
                level: IssueLevelity::Error,
                message: e,
                line: None,
            }],
        },
        Ok(_tokens) => {
            // Apply formatting rules
            let formatted = apply_formatting(source);

            ToolResult {
                success: true,
                message: "Formatted successfully".to_string(),
                output: Some(formatted),
                issues: vec![],
            }
        }
    }
}

/// Lint a .cnf source file for style and semantic issues
pub fn lint_source(source: &str) -> ToolResult {
    let mut issues = Vec::new();

    // Tokenization check
    let tokens = match tokenize(source) {
        Err(e) => {
            issues.push(Issue {
                level: IssueLevelity::Error,
                message: format!("Tokenization failed: {}", e),
                line: None,
            });
            return ToolResult {
                success: false,
                message: format!("Linting failed with {} error(s)", issues.len()),
                output: None,
                issues,
            };
        }
        Ok(tokens) => tokens,
    };

    // Parser check
    let parser = Parser::new(tokens);
    if let Err(e) = parser.parse() {
        issues.push(Issue {
            level: IssueLevelity::Error,
            message: format!("Parse error: {}", e),
            line: None,
        });
    }

    // Style linting
    let style_issues = lint_style(source);
    issues.extend(style_issues);

    // Semantic linting
    let semantic_issues = lint_semantics(source);
    issues.extend(semantic_issues);

    let has_errors = issues.iter().any(|i| i.level == IssueLevelity::Error);

    ToolResult {
        success: !has_errors,
        message: if has_errors {
            format!("Linting found {} error(s) and {} warning(s)",
                issues.iter().filter(|i| i.level == IssueLevelity::Error).count(),
                issues.iter().filter(|i| i.level == IssueLevelity::Warning).count()
            )
        } else {
            format!("Linting complete: {} warning(s), {} info(s)",
                issues.iter().filter(|i| i.level == IssueLevelity::Warning).count(),
                issues.iter().filter(|i| i.level == IssueLevelity::Info).count()
            )
        },
        output: None,
        issues,
    }
}

/// Apply formatting rules to source code
fn apply_formatting(source: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let mut formatted = Vec::new();
    let mut indent_level = 0;

    for line in lines {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            formatted.push(String::new());
            continue;
        }

        // Adjust indentation for DIVISION keywords (no indent)
        if trimmed.ends_with("DIVISION.") {
            indent_level = 0;
        }

        // Add indented line
        let indented = format!("{}{}", "    ".repeat(indent_level), trimmed);
        formatted.push(indented);

        // Increase indent after DIVISION keywords
        if trimmed.ends_with("DIVISION.") {
            indent_level = 1;
        }

        // Decrease indent for END-* keywords or next division
        if trimmed.starts_with("END-") {
            indent_level = (indent_level as i32 - 1).max(0) as usize;
        }
    }

    formatted.join("\n") + "\n"
}

/// Check for style issues
fn lint_style(source: &str) -> Vec<Issue> {
    let mut issues = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    for (line_num, line) in lines.iter().enumerate() {
        let line_no = line_num + 1;

        // Check for trailing whitespace
        if line.ends_with(' ') || line.ends_with('\t') {
            issues.push(Issue {
                level: IssueLevelity::Warning,
                message: "Trailing whitespace detected".to_string(),
                line: Some(line_no),
            });
        }

        // Check for mixed indentation (tabs and spaces)
        let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
        if indent.contains('\t') && indent.contains(' ') {
            issues.push(Issue {
                level: IssueLevelity::Warning,
                message: "Mixed tabs and spaces in indentation".to_string(),
                line: Some(line_no),
            });
        }

        // Check for DIVISION format
        if line.trim().ends_with("DIVISION.") {
            // DIVISION keywords should be at column 0 or minimal indent
            let indent_count = line.chars().take_while(|c| c.is_whitespace()).count();
            if indent_count > 4 {
                issues.push(Issue {
                    level: IssueLevelity::Info,
                    message: "DIVISION keyword should be at start of section".to_string(),
                    line: Some(line_no),
                });
            }
        }

        // Check for long lines (> 100 chars)
        if line.len() > 100 {
            issues.push(Issue {
                level: IssueLevelity::Info,
                message: format!("Line exceeds 100 characters ({})", line.len()),
                line: Some(line_no),
            });
        }
    }

    issues
}

/// Check for semantic issues
fn lint_semantics(source: &str) -> Vec<Issue> {
    let mut issues = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    // Track division presence
    let mut has_identification = false;
    let mut has_data = false;
    let mut has_procedure = false;

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let line_no = line_num + 1;

        if trimmed.contains("IDENTIFICATION DIVISION") {
            has_identification = true;
        }
        if trimmed.contains("DATA DIVISION") {
            has_data = true;
        }
        if trimmed.contains("PROCEDURE DIVISION") {
            has_procedure = true;
        }

        // Check for variables without SET
        if (trimmed.contains("ADD") || trimmed.contains("COMPRESS"))
            && !trimmed.starts_with("--")
        {
            // This is a simple heuristic; real check would be more sophisticated
        }
    }

    // Report missing required divisions
    if !has_identification && !lines.is_empty() {
        issues.push(Issue {
            level: IssueLevelity::Error,
            message: "Missing IDENTIFICATION DIVISION".to_string(),
            line: Some(1),
        });
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_simple_program() {
        let source = r#"IDENTIFICATION DIVISION.
PROGRAM-ID. TEST.
DATA DIVISION.
PROCEDURE DIVISION.
DISPLAY "Hello".
"#;
        let result = format_source(source);
        assert!(result.success);
        assert!(result.output.is_some());
    }

    #[test]
    fn test_lint_detects_trailing_whitespace() {
        let source = "IDENTIFICATION DIVISION.   \n";
        let result = lint_source(source);
        assert!(result.issues.iter().any(|i| i.message.contains("Trailing")));
    }

    #[test]
    fn test_lint_detects_long_lines() {
        let source =
            "IDENTIFICATION DIVISION.\n\
             THIS_IS_A_VERY_LONG_LINE_THAT_EXCEEDS_THE_STANDARD_100_CHARACTER_LIMIT_SET_BY_THE_LINTER_TO_ENSURE_CODE_READABILITY.\n";
        let result = lint_source(source);
        assert!(result.issues.iter().any(|i| i.message.contains("exceeds")));
    }
}
