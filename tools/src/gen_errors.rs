use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

// ============================================================================
// Data Structures
// ============================================================================

/// Single error registry database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRegistry {
    pub metadata: RegistryMetadata,
    pub errors: HashMap<String, ErrorEntry>,
}

/// Metadata for the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryMetadata {
    pub format_version: String,
    pub last_updated: String,
    pub total_count: u32,
    pub layers: HashMap<String, String>,
}

/// Individual error entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEntry {
    pub code: String,
    pub layer: u32,
    pub layer_name: String,
    pub category: String,
    pub title: String,
    pub description: String,
    pub trigger_code: String,
    pub expected_error: String,
    pub fix: String,
}


// ============================================================================
// Permutation Engine for Unique Error Variations
// ============================================================================

struct PermutationEngine {
    keywords: Vec<&'static str>,
    data_types: Vec<&'static str>,
    contexts: Vec<&'static str>,
    layers: HashMap<u32, LayerConfig>,
}

#[derive(Clone)]
struct LayerConfig {
    name: String,
    code_range: (u32, u32),
    base_error_type: String,
}

impl PermutationEngine {
    fn new() -> Self {
        let mut layers = HashMap::new();

        layers.insert(
            1,
            LayerConfig {
                name: "Lexer".to_string(),
                code_range: (1001, 1999),
                base_error_type: "TokenError".to_string(),
            },
        );
        layers.insert(
            2,
            LayerConfig {
                name: "Parser".to_string(),
                code_range: (2001, 2999),
                base_error_type: "SyntaxError".to_string(),
            },
        );
        layers.insert(
            3,
            LayerConfig {
                name: "IR".to_string(),
                code_range: (3001, 3999),
                base_error_type: "TypeError".to_string(),
            },
        );
        layers.insert(
            4,
            LayerConfig {
                name: "Runtime".to_string(),
                code_range: (4001, 4999),
                base_error_type: "RuntimeError".to_string(),
            },
        );
        layers.insert(
            5,
            LayerConfig {
                name: "Security".to_string(),
                code_range: (5001, 5999),
                base_error_type: "SecurityError".to_string(),
            },
        );

        Self {
            keywords: vec![
                "IDENTIFICATION", "ENVIRONMENT", "DATA", "PROCEDURE", "DIVISION",
                "COMPRESS", "VERIFY", "ENCRYPT", "DECRYPT", "TRANSCODE",
                "FILTER", "AGGREGATE", "MERGE", "SPLIT", "VALIDATE",
                "EXTRACT", "CONVERT", "OS", "ARCH", "INVALID_KEYWORD",
            ],
            data_types: vec![
                "VIDEO-MP4", "IMAGE-JPG", "AUDIO-WAV", "CSV-TABLE",
                "JSON-OBJECT", "XML-DOCUMENT", "PARQUET-TABLE", "BINARY-BLOB",
            ],
            contexts: vec![
                "in IDENTIFICATION DIVISION",
                "in ENVIRONMENT DIVISION",
                "in DATA DIVISION",
                "in PROCEDURE DIVISION",
                "in declaration",
                "in assignment",
                "in operation",
                "in expression",
            ],
            layers,
        }
    }

    /// Generate errors for a specific layer using permutation logic
    fn generate_for_layer(&self, layer: u32, count: u32) -> Vec<ErrorEntry> {
        let config = match self.layers.get(&layer) {
            Some(c) => c.clone(),
            None => return vec![],
        };

        let mut errors = Vec::new();
        let (start_code, _end_code) = config.code_range;

        for idx in 0..count {
            let code_num = start_code + idx;
            let code = format!("L{}", code_num);

            let keyword_idx = (idx as usize) % self.keywords.len();
            let type_idx = ((idx / self.keywords.len() as u32) as usize) % self.data_types.len();
            let context_idx =
                ((idx / (self.keywords.len() as u32 * self.data_types.len() as u32)) as usize)
                    % self.contexts.len();

            let keyword = self.keywords[keyword_idx];
            let data_type = self.data_types[type_idx];
            let context = self.contexts[context_idx];

            let (title, description, trigger_code, expected_error, fix) =
                self.generate_error_content(layer, keyword, data_type, context);

            errors.push(ErrorEntry {
                code: code.clone(),
                layer,
                layer_name: config.name.clone(),
                category: config.base_error_type.clone(),
                title,
                description,
                trigger_code,
                expected_error,
                fix,
            });
        }

        errors
    }

    /// Generate error content based on layer and parameters
    fn generate_error_content(
        &self,
        layer: u32,
        keyword: &str,
        data_type: &str,
        context: &str,
    ) -> (String, String, String, String, String) {
        match layer {
            1 => {
                // Lexer errors
                let title = format!("Invalid token '{}' {}", keyword, context);
                let description =
                    format!("Lexer encountered invalid token when parsing {}", context);
                let trigger_code = format!(
                    "IDENTIFICATION DIVISION.\n    {} {}.",
                    keyword, data_type
                );
                let expected_error = format!("Invalid token '{}'", keyword);
                let fix = format!(
                    "Use valid CENTRA-NF keywords only. '{}' is not recognized.",
                    keyword
                );
                (title, description, trigger_code, expected_error, fix)
            }
            2 => {
                // Parser errors
                let title = format!("Invalid {} declaration {}", data_type, context);
                let description = format!(
                    "Parser found invalid {} declaration: expected valid syntax",
                    data_type
                );
                let trigger_code = format!(
                    "DATA DIVISION.\n    {} = {}.",
                    keyword, data_type
                );
                let expected_error = format!("Expected PICTURE or PIC clause, found '{}'", keyword);
                let fix = format!(
                    "Use proper COBOL-style declaration: PICTURE or PIC with valid format"
                );
                (title, description, trigger_code, expected_error, fix)
            }
            3 => {
                // IR errors
                let title = format!("Type mismatch: {} vs {}", data_type, keyword);
                let description =
                    format!("IR lowering failed: incompatible types {} and {}", data_type, keyword);
                let trigger_code = format!(
                    "PROCEDURE DIVISION.\n    COMPRESS {} AS {}.",
                    keyword, data_type
                );
                let expected_error = format!("Type error: cannot apply COMPRESS to {}", keyword);
                let fix = format!("Ensure {} is compatible with COMPRESS operation", data_type);
                (title, description, trigger_code, expected_error, fix)
            }
            4 => {
                // Runtime errors
                let title = format!("Runtime failure on {} with {}", keyword, data_type);
                let description = format!("Runtime error occurred during {} operation", keyword);
                let trigger_code = format!(
                    "DATA DIVISION.\n        {} AS {}.\n    PROCEDURE DIVISION.\n        VERIFY-INTEGRITY {}.",
                    keyword, data_type, keyword
                );
                let expected_error = format!("Runtime error: {} not initialized", keyword);
                let fix = format!("Ensure {} is properly initialized before use", keyword);
                (title, description, trigger_code, expected_error, fix)
            }
            5 => {
                // Security errors
                let title = format!("Encryption failure with {}", data_type);
                let description = format!("Security operation failed on {} type", data_type);
                let trigger_code = format!("PROCEDURE DIVISION.\n    ENCRYPT {} .", keyword);
                let expected_error = format!("Encryption error: invalid {} for encryption", data_type);
                let fix = format!("Use supported data types for encryption: VIDEO-MP4, IMAGE-JPG, AUDIO-WAV");
                (title, description, trigger_code, expected_error, fix)
            }
            _ => (
                "Unknown error".to_string(),
                "Unknown error".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            ),
        }
    }
}


// ============================================================================
// Error Registry Manager
// ============================================================================

pub struct ErrorManager {
    registry_path: String,
    docs_path: String,
    registry: ErrorRegistry,
}

impl ErrorManager {
    /// Load or create error registry
    pub fn new(registry_path: &str, docs_path: &str) -> Result<Self, String> {
        let registry = if Path::new(registry_path).exists() {
            let content = fs::read_to_string(registry_path)
                .map_err(|e| format!("Failed to read registry: {}", e))?;
            serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse registry JSON: {}", e))?
        } else {
            ErrorRegistry {
                metadata: RegistryMetadata {
                    format_version: "1.0".to_string(),
                    last_updated: "2026-03-05".to_string(),
                    total_count: 0,
                    layers: {
                        let mut m = HashMap::new();
                        m.insert("L1".to_string(), "Lexer (1001-1999)".to_string());
                        m.insert("L2".to_string(), "Parser (2001-2999)".to_string());
                        m.insert("L3".to_string(), "IR (3001-3999)".to_string());
                        m.insert("L4".to_string(), "Runtime (4001-4999)".to_string());
                        m.insert("L5".to_string(), "Security (5001-5999)".to_string());
                        m
                    },
                },
                errors: HashMap::new(),
            }
        };

        Ok(Self {
            registry_path: registry_path.to_string(),
            docs_path: docs_path.to_string(),
            registry,
        })
    }

    /// Generate and add errors for a layer (idempotent)
    pub fn generate_layer(&mut self, layer: u32, count: u32) -> Result<u32, String> {
        let engine = PermutationEngine::new();
        let new_errors = engine.generate_for_layer(layer, count);

        let mut added = 0;
        for error in new_errors {
            if !self.registry.errors.contains_key(&error.code) {
                self.registry.errors.insert(error.code.clone(), error);
                added += 1;
            }
        }

        self.registry.metadata.total_count = self.registry.errors.len() as u32;

        Ok(added)
    }

    /// Save registry to JSON file
    pub fn save_registry(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.registry)
            .map_err(|e| format!("Failed to serialize registry: {}", e))?;

        fs::write(&self.registry_path, json)
            .map_err(|e| format!("Failed to write registry: {}", e))?;

        println!("✅ Registry saved to: {}", self.registry_path);
        Ok(())
    }

    /// Auto-sync documentation from registry
    pub fn sync_docs(&self) -> Result<(), String> {
        let mut doc_content = String::from(
            "# CENTRA-NF Error Codes Reference\n\n\
             **Auto-generated from errors_registry.json - Do not edit manually**\n\n",
        );

        // Group errors by layer
        let mut layers: HashMap<u32, Vec<_>> = HashMap::new();
        for error in self.registry.errors.values() {
            layers.entry(error.layer).or_insert_with(Vec::new).push(error);
        }

        // Generate doc sections per layer
        for layer in 1..=5 {
            if let Some(mut errors) = layers.remove(&layer) {
                errors.sort_by(|a, b| a.code.cmp(&b.code));

                let layer_name = match layer {
                    1 => "Lexer",
                    2 => "Parser",
                    3 => "IR",
                    4 => "Runtime",
                    5 => "Security",
                    _ => "Unknown",
                };

                doc_content.push_str(&format!(
                    "## Layer {}: {} ({} errors)\n\n",
                    layer,
                    layer_name,
                    errors.len()
                ));
                doc_content.push_str("| Code | Title | Category | Description | Trigger | Fix |\n");
                doc_content.push_str("|------|-------|----------|-------------|---------|-----|\n");

                for error in errors {
                    doc_content.push_str(&format!(
                        "| {} | {} | {} | {} | `{}` | {} |\n",
                        error.code,
                        error.title,
                        error.category,
                        error.description,
                        error.trigger_code.lines().next().unwrap_or(""),
                        error.fix
                    ));
                }

                doc_content.push_str("\n");
            }
        }

        fs::write(&self.docs_path, doc_content)
            .map_err(|e| format!("Failed to write docs: {}", e))?;

        println!("✅ Documentation synced to: {}", self.docs_path);
        Ok(())
    }

    /// Virtual test: run error in-memory without persistent files
    pub fn test_error_virtual(&self, code: &str) -> Result<bool, String> {
        let error = self
            .registry
            .errors
            .get(code)
            .ok_or_else(|| format!("Error code {} not found", code))?;

        // Write to temp file
        let temp_file = format!("/tmp/{}_test.cnf", code);
        fs::write(&temp_file, &error.trigger_code)
            .map_err(|e| format!("Failed to write temp file: {}", e))?;

        // Run compiler on temp file
        let output = Command::new("cargo")
            .args(&["run", "--bin", "centra-nf", "--", "check", &temp_file])
            .current_dir("/workspaces/v1")
            .output()
            .map_err(|e| format!("Failed to run compiler: {}", e))?;

        // Clean up temp file immediately
        let _ = fs::remove_file(&temp_file);

        // Check if expected error appears in output
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let full_output = format!("{}\n{}", stdout, stderr);

        let test_passed = full_output.contains(&error.expected_error);

        Ok(test_passed)
    }

    /// Get registry statistics
    pub fn get_stats(&self) -> (u32, HashMap<u32, u32>) {
        let mut layer_counts: HashMap<u32, u32> = HashMap::new();
        for error in self.registry.errors.values() {
            *layer_counts.entry(error.layer).or_insert(0) += 1;
        }
        (self.registry.errors.len() as u32, layer_counts)
    }
}

// ============================================================================
// Main Entry Point
// ============================================================================

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();

    let registry_path = "/workspaces/v1/errors_registry.json";
    let docs_path = "/workspaces/v1/docs/error-codes.md";

    // Parse arguments: layer (default 1), count (default 100)
    let layer = if args.len() > 1 {
        args[1].parse::<u32>().unwrap_or(1)
    } else {
        1
    };

    let count = if args.len() > 2 {
        args[2].parse::<u32>().unwrap_or(100)
    } else {
        100
    };

    println!("🔧 CENTRA-NF Error Code Generator");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📋 Registry: {}", registry_path);
    println!("📄 Docs: {}", docs_path);
    println!("🎯 Generating {} errors for Layer {}\n", count, layer);

    let mut manager = ErrorManager::new(registry_path, docs_path)?;

    // Generate new errors
    match manager.generate_layer(layer, count) {
        Ok(added) => {
            let (total, layer_counts) = manager.get_stats();

            println!("✅ Added {} new error codes", added);
            println!("📊 Total errors in registry: {}\n", total);

            println!("Layer breakdown:");
            for l in 1..=5 {
                if let Some(c) = layer_counts.get(&l) {
                    let layer_name = match l {
                        1 => "Lexer",
                        2 => "Parser",
                        3 => "IR",
                        4 => "Runtime",
                        5 => "Security",
                        _ => "Unknown",
                    };
                    println!("  Layer {}: {} errors", layer_name, c);
                }
            }

            // Save and sync
            println!("\n📦 Saving...");
            manager.save_registry()?;

            println!("🔄 Syncing documentation...");
            manager.sync_docs()?;

            println!("\n✨ Generation complete!");
            println!("🎉 Ready to test errors with virtual test engine\n");

            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Error: {}", e);
            Err(e)
        }
    }
}
