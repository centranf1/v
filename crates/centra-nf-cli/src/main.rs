//! centra-nf — Command-line interface for CENTRA-NF compiler
//!
//! Usage:
//!   centra-nf compile <input.cnf> [--output <output>]
//!   centra-nf check <input.cnf>
//!   centra-nf help

use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

use cnf_compiler::compile;

mod tools;
use tools::{format_source, lint_source, IssueLevelity};

#[derive(Parser)]
#[command(name = "centra-nf")]
#[command(about = "CENTRA-NF Compiler — Deterministic, fail-fast compilation", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a .cnf source file to intermediate representation
    Compile {
        /// Input .cnf file
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output file for IR (default: stdout)
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        /// Verbose output (show IR instructions)
        #[arg(short, long)]
        verbose: bool,
    },

    /// Check syntax of a .cnf file without compiling
    Check {
        /// Input .cnf file
        #[arg(value_name = "FILE")]
        input: PathBuf,
    },

    /// Run a .cnf program using the runtime
    Run {
        /// Input .cnf file
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Buffer data as hex string (for INPUT variables)
        #[arg(short, long, value_name = "HEX")]
        buffer: Option<String>,

        /// Verbose output (show IR instructions and buffer states)
        #[arg(short, long)]
        verbose: bool,
    },

    /// Interactive REPL (Read-Eval-Print-Loop) for testing snippets
    Repl,

    /// Format a .cnf source file to canonical style
    Format {
        /// Input .cnf file
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output file (default: stdout)
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        /// Check formatting without writing (dry-run)
        #[arg(short, long)]
        check: bool,
    },

    /// Lint a .cnf source file for style and semantic issues
    Lint {
        /// Input .cnf file
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output format: table, json, or text (default: table)
        #[arg(short, long, value_name = "FORMAT", default_value = "table")]
        format: String,

        /// Fail if any warnings are found (strict mode)
        #[arg(short, long)]
        strict: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compile {
            input,
            output,
            verbose,
        } => {
            compile_file(&input, output.as_ref(), verbose);
        }
        Commands::Check { input } => {
            check_file(&input);
        }
        Commands::Run {
            input,
            buffer,
            verbose,
        } => {
            run_file(&input, buffer.as_deref(), verbose);
        }
        Commands::Repl => {
            println!("🎯 CENTRA-NF REPL (Interactive Shell)");
            println!("Type 'help' for commands, 'quit' to exit\n");
            // TODO: Implement interactive REPL for v0.3.0
            println!("REPL coming in v0.3.0");
        }
        Commands::Format {
            input,
            output,
            check,
        } => {
            format_file(&input, output.as_ref(), check);
        }
        Commands::Lint {
            input,
            format: fmt,
            strict,
        } => {
            lint_file(&input, &fmt, strict);
        }
    }
}

/// Compile a .cnf file and output IR
fn compile_file(input_path: &PathBuf, output_path: Option<&PathBuf>, verbose: bool) {
    // Read source file
    let source = match fs::read_to_string(input_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading file '{}': {}", input_path.display(), e);
            std::process::exit(1);
        }
    };

    // Compile
    match compile(&source) {
        Ok(instructions) => {
            if verbose {
                eprintln!(
                    "ℹ️  Compiled successfully. Generated {} instruction(s)",
                    instructions.len()
                );
            }

            // Format output
            let output_text = if instructions.is_empty() {
                "(empty program)".to_string()
            } else {
                instructions
                    .iter()
                    .map(|instr| instr.to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            };

            // Write output
            if let Some(out_path) = output_path {
                match fs::write(out_path, &output_text) {
                    Ok(_) => {
                        if verbose {
                            eprintln!("✓ Output written to '{}'", out_path.display());
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Error writing file '{}': {}", out_path.display(), e);
                        std::process::exit(1);
                    }
                }
            } else {
                println!("{}", output_text);
            }
        }
        Err(e) => {
            eprintln!("❌ Compilation error:\n{}", e);
            std::process::exit(1);
        }
    }
}

/// Check syntax of a .cnf file
fn check_file(input_path: &PathBuf) {
    // Read source file
    let source = match fs::read_to_string(input_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading file '{}': {}", input_path.display(), e);
            std::process::exit(1);
        }
    };

    // Compile (check only)
    match compile(&source) {
        Ok(_) => {
            eprintln!("✓ Syntax OK: '{}'", input_path.display());
        }
        Err(e) => {
            eprintln!("❌ Syntax error in '{}':\n{}", input_path.display(), e);
            std::process::exit(1);
        }
    }
}

/// Run a .cnf program using the runtime
fn run_file(input_path: &PathBuf, buffer_hex: Option<&str>, verbose: bool) {
    // Read source file
    let source = match fs::read_to_string(input_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading file '{}': {}", input_path.display(), e);
            std::process::exit(1);
        }
    };

    // Compile
    let instructions = match compile(&source) {
        Ok(instr) => instr,
        Err(e) => {
            eprintln!("❌ Compilation error:\n{}", e);
            std::process::exit(1);
        }
    };

    if verbose {
        eprintln!(
            "ℹ️  Compiled successfully. Generated {} instruction(s)",
            instructions.len()
        );
        for instr in &instructions {
            eprintln!("  → {}", instr);
        }
    }

    // Initialize runtime
    let mut runtime = cnf_runtime::Runtime::new();

    // Add buffer if provided. We attempt to infer the intended variable name
    // from the compiled IR so that the caller need not hardcode it.
    if let Some(hex) = buffer_hex {
        let data = match hex::decode(hex) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("❌ Invalid hex buffer: {}", e);
                std::process::exit(1);
            }
        };

        // heuristically pick the first target name mentioned in the program
        fn infer_name(instrs: &[cnf_compiler::ir::Instruction]) -> Option<String> {
            use cnf_compiler::ir::Instruction;
            for instr in instrs {
                match instr {
                    Instruction::Compress { target }
                    | Instruction::VerifyIntegrity { target }
                    | Instruction::Encrypt { target }
                    | Instruction::Decrypt { target }
                    | Instruction::Transcode { target, .. }
                    | Instruction::Filter { target, .. }
                    | Instruction::Convert { target, .. }
                    | Instruction::Split { target, .. }
                    | Instruction::Validate { target, .. }
                    | Instruction::Extract { target, .. }
                    | Instruction::Print { target, .. }
                    | Instruction::Read { target }
                    | Instruction::Set { target, .. }
                    | Instruction::Add { target, .. }
                    | Instruction::Subtract { target, .. }
                    | Instruction::Multiply { target, .. }
                    | Instruction::Divide { target, .. } => {
                        return Some(target.clone());
                    }
                    Instruction::Aggregate { targets, .. } | Instruction::Merge { targets, .. } => {
                        if let Some(first) = targets.first() {
                            return Some(first.clone());
                        }
                    }
                    Instruction::IfStatement {
                        then_instrs,
                        else_instrs,
                        ..
                    } => {
                        if let Some(name) = infer_name(then_instrs) {
                            return Some(name);
                        }
                        if let Some(e) = else_instrs {
                            if let Some(name) = infer_name(e) {
                                return Some(name);
                            }
                        }
                    }
                    Instruction::ForLoop { instrs, .. } | Instruction::WhileLoop { instrs, .. } => {
                        if let Some(name) = infer_name(instrs) {
                            return Some(name);
                        }
                    }
                    Instruction::FunctionDef { instrs, .. } => {
                        // dive into function body
                        if let Some(name) = infer_name(instrs) {
                            return Some(name);
                        }
                    }
                    _ => {}
                }
            }
            None
        }

        let buf_name = infer_name(&instructions).unwrap_or_else(|| "INPUT".to_string());
        runtime.add_buffer(buf_name, data);
    }

    // Execute IR instructions through the runtime
    if let Err(e) = runtime.execute_instructions(&instructions) {
        eprintln!("❌ Runtime error:\n{}", e);
        std::process::exit(1);
    }

    if verbose {
        eprintln!("✓ Execution completed successfully");
        // dump buffer states
        for (name, buf) in runtime.list_buffers() {
            println!("BUFFER {}: {}", name, hex::encode(buf));
        }
    } else {
        eprintln!("✓ Execution completed successfully");
    }
}

/// Format a .cnf file to canonical style
fn format_file(input_path: &PathBuf, output_path: Option<&PathBuf>, dry_run: bool) {
    // Read source file
    let source = match fs::read_to_string(input_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading file '{}': {}", input_path.display(), e);
            std::process::exit(1);
        }
    };

    // Format
    let result = format_source(&source);

    if !result.success {
        eprintln!("❌ Formatting failed: {}", result.message);
        for issue in &result.issues {
            eprintln!("  {} - {}", issue.level, issue.message);
        }
        std::process::exit(1);
    }

    if let Some(formatted) = result.output {
        if dry_run {
            eprintln!("✓ Format check passed (no changes needed)");
            // Show diff preview if formatting would change file
            if formatted.trim() != source.trim() {
                eprintln!("ℹ️  File would be reformatted");
            }
        } else if let Some(out_path) = output_path {
            // Write to output file
            match fs::write(out_path, &formatted) {
                Ok(_) => {
                    eprintln!("✓ Formatted and written to '{}'", out_path.display());
                }
                Err(e) => {
                    eprintln!("❌ Error writing file '{}': {}", out_path.display(), e);
                    std::process::exit(1);
                }
            }
        } else {
            // Write to stdout
            println!("{}", formatted);
        }
    }
}

/// Lint a .cnf file for style and semantic issues
fn lint_file(input_path: &PathBuf, output_format: &str, strict: bool) {
    // Read source file
    let source = match fs::read_to_string(input_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading file '{}': {}", input_path.display(), e);
            std::process::exit(1);
        }
    };

    // Lint
    let result = lint_source(&source);

    // Print results based on format
    match output_format {
        "json" => {
            // JSON output format
            println!("{{");
            println!(
                "  \"success\": {},",
                if result.success { "true" } else { "false" }
            );
            println!("  \"message\": \"{}\",", result.message);
            println!("  \"issues\": [");
            for (i, issue) in result.issues.iter().enumerate() {
                println!("    {{");
                println!("      \"level\": \"{}\",", issue.level);
                println!("      \"message\": \"{}\",", issue.message);
                if let Some(line) = issue.line {
                    println!("      \"line\": {},", line);
                } else {
                    println!("      \"line\": null,");
                }
                println!("    }}{}", if i < result.issues.len() - 1 { "," } else { "" });
            }
            println!("  ]");
            println!("}}");
        }
        "text" => {
            // Text output format
            eprintln!("{}", result.message);
            for issue in &result.issues {
                let line_info = issue
                    .line
                    .map(|l| format!(" (line {})", l))
                    .unwrap_or_default();
                eprintln!(
                    "  [{}] {}{}",
                    issue.level.to_string(),
                    issue.message,
                    line_info
                );
            }
        }
        "table" | _ => {
            // Table output format (default)
            eprintln!("{}", result.message);
            if !result.issues.is_empty() {
                eprintln!(
                    "\n{:<8} {:<50} {:<10}",
                    "LEVEL", "MESSAGE", "LINE"
                );
                eprintln!("{}", "─".repeat(68));
                for issue in &result.issues {
                    let line_str = issue
                        .line
                        .map(|l| l.to_string())
                        .unwrap_or_else(|| "—".to_string());
                    eprintln!(
                        "{:<8} {:<50} {:<10}",
                        issue.level.to_string(),
                        &issue.message[..issue.message.len().min(48)],
                        line_str
                    );
                }
            }
        }
    }

    // Exit with error if strict mode and issues found
    if strict && !result.issues.is_empty() {
        std::process::exit(1);
    }

    // Success exit code
    if !result.success {
        std::process::exit(1);
    }
}
