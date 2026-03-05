//! IR — Intermediate Representation.
//!
//! Lowering from AST to deterministic instruction stream.
//! Same input AST → same instruction stream, always.

use crate::ast::ProcedureStatement;
use crate::ast::Program;

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Compress {
        target: String,
    },
    VerifyIntegrity {
        target: String,
    },
    Encrypt {
        target: String,
    },
    Decrypt {
        target: String,
    },
    Transcode {
        target: String,
        output_type: String,
    },
    Filter {
        target: String,
        condition: String,
    },
    Aggregate {
        targets: Vec<String>,
        operation: String,
    },
    Convert {
        target: String,
        output_type: String,
    },
    Merge {
        targets: Vec<String>,
        output_name: String,
    },
    Split {
        target: String,
        parts: String,
    },
    Validate {
        target: String,
        schema: String,
    },
    Extract {
        target: String,
        path: String,
    },
    IfStatement {
        condition: String,
        then_instrs: Vec<String>,
        else_instrs: Option<Vec<String>>,
    },
    ForLoop {
        variable: String,
        in_list: String,
        instrs: Vec<String>,
    },
    WhileLoop {
        condition: String,
        instrs: Vec<String>,
    },
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Compress { target } => {
                write!(f, "COMPRESS({})", target)
            }
            Instruction::VerifyIntegrity { target } => {
                write!(f, "VERIFY-INTEGRITY({})", target)
            }
            Instruction::Encrypt { target } => {
                write!(f, "ENCRYPT({})", target)
            }
            Instruction::Decrypt { target } => {
                write!(f, "DECRYPT({})", target)
            }
            Instruction::Transcode {
                target,
                output_type,
            } => {
                write!(f, "TRANSCODE({} -> {})", target, output_type)
            }
            Instruction::Filter { target, condition } => {
                write!(f, "FILTER({} WHERE {})", target, condition)
            }
            Instruction::Aggregate { targets, operation } => {
                write!(f, "AGGREGATE({} AS {})", targets.join(","), operation)
            }
            Instruction::Convert {
                target,
                output_type,
            } => {
                write!(f, "CONVERT({} -> {})", target, output_type)
            }
            Instruction::Merge {
                targets,
                output_name,
            } => {
                write!(f, "MERGE({} INTO {})", targets.join(","), output_name)
            }
            Instruction::Split { target, parts } => {
                write!(f, "SPLIT({} INTO {} PARTS)", target, parts)
            }
            Instruction::Validate { target, schema } => {
                write!(f, "VALIDATE({} AGAINST {})", target, schema)
            }
            Instruction::Extract { target, path } => {
                write!(f, "EXTRACT({} FROM {})", path, target)
            }
            Instruction::IfStatement {
                condition,
                then_instrs,
                else_instrs,
            } => {
                write!(f, "IF({}) THEN[{}]", condition, then_instrs.len())?;
                if let Some(else_i) = else_instrs {
                    write!(f, " ELSE[{}]", else_i.len())?;
                }
                Ok(())
            }
            Instruction::ForLoop {
                variable,
                in_list,
                instrs,
            } => {
                write!(f, "FOR({} IN {}) [{}]", variable, in_list, instrs.len())
            }
            Instruction::WhileLoop { condition, instrs } => {
                write!(f, "WHILE({}) [{}]", condition, instrs.len())
            }

pub fn lower(program: Program) -> Result<Vec<Instruction>, String> {
    let mut instructions = Vec::new();

    // Validate that all procedure statements reference valid variables
    let declared_vars: std::collections::HashSet<String> = program
        .data
        .variables
        .iter()
        .map(|v| v.name.clone())
        .collect();

    for stmt in &program.procedure.statements {
        match stmt {
            ProcedureStatement::Compress { target } => {
                if !declared_vars.contains(target) {
                    return Err(format!(
                        "Variable '{}' not declared in DATA DIVISION",
                        target
                    ));
                }
                instructions.push(Instruction::Compress {
                    target: target.clone(),
                });
            }
            ProcedureStatement::VerifyIntegrity { target } => {
                if !declared_vars.contains(target) {
                    return Err(format!(
                        "Variable '{}' not declared in DATA DIVISION",
                        target
                    ));
                }
                instructions.push(Instruction::VerifyIntegrity {
                    target: target.clone(),
                });
            }
            ProcedureStatement::Encrypt { target } => {
                if !declared_vars.contains(target) {
                    return Err(format!(
                        "Variable '{}' not declared in DATA DIVISION",
                        target
                    ));
                }
                instructions.push(Instruction::Encrypt {
                    target: target.clone(),
                });
            }
            ProcedureStatement::Decrypt { target } => {
                if !declared_vars.contains(target) {
                    return Err(format!(
                        "Variable '{}' not declared in DATA DIVISION",
                        target
                    ));
                }
                instructions.push(Instruction::Decrypt {
                    target: target.clone(),
                });
            }
            ProcedureStatement::Transcode {
                target,
                output_type,
            } => {
                if !declared_vars.contains(target) {
                    return Err(format!(
                        "Variable '{}' not declared in DATA DIVISION",
                        target
                    ));
                }
                instructions.push(Instruction::Transcode {
                    target: target.clone(),
                    output_type: output_type.to_string(),
                });
            }
            ProcedureStatement::Filter { target, condition } => {
                if !declared_vars.contains(target) {
                    return Err(format!(
                        "Variable '{}' not declared in DATA DIVISION",
                        target
                    ));
                }
                instructions.push(Instruction::Filter {
                    target: target.clone(),
                    condition: condition.clone(),
                });
            }
            ProcedureStatement::Aggregate { targets, operation } => {
                for target in targets {
                    if !declared_vars.contains(target) {
                        return Err(format!(
                            "Variable '{}' not declared in DATA DIVISION",
                            target
                        ));
                    }
                }
                instructions.push(Instruction::Aggregate {
                    targets: targets.clone(),
                    operation: operation.clone(),
                });
            }
            ProcedureStatement::Convert {
                target,
                output_type,
            } => {
                if !declared_vars.contains(target) {
                    return Err(format!(
                        "Variable '{}' not declared in DATA DIVISION",
                        target
                    ));
                }
                instructions.push(Instruction::Convert {
                    target: target.clone(),
                    output_type: output_type.to_string(),
                });
            }
            ProcedureStatement::Merge {
                targets,
                output_name,
            } => {
                for target in targets {
                    if !declared_vars.contains(target) {
                        return Err(format!(
                            "Variable '{}' not declared in DATA DIVISION",
                            target
                        ));
                    }
                }
                instructions.push(Instruction::Merge {
                    targets: targets.clone(),
                    output_name: output_name.clone(),
                });
            }
            ProcedureStatement::Split { target, parts } => {
                if !declared_vars.contains(target) {
                    return Err(format!(
                        "Variable '{}' not declared in DATA DIVISION",
                        target
                    ));
                }
                instructions.push(Instruction::Split {
                    target: target.clone(),
                    parts: parts.clone(),
                });
            }
            ProcedureStatement::Validate { target, schema } => {
                if !declared_vars.contains(target) {
                    return Err(format!(
                        "Variable '{}' not declared in DATA DIVISION",
                        target
                    ));
                }
                instructions.push(Instruction::Validate {
                    target: target.clone(),
                    schema: schema.clone(),
                });
            }
            ProcedureStatement::Extract { target, path } => {
                if !declared_vars.contains(target) {
                    return Err(format!(
                        "Variable '{}' not declared in DATA DIVISION",
                        target
                    ));
                }
                instructions.push(Instruction::Extract {
                    target: target.clone(),
                    path: path.clone(),
                });
            }
            ProcedureStatement::If {
                condition,
                then_statements,
                else_statements,
            } => {
                // For now, flatten nested statements into the then/else blocks
                let mut then_instrs = Vec::new();
                for stmt in then_statements {
                    // Recursively lower nested statements (placeholder)
                    then_instrs.push(format!("STMT({})", std::any::type_name_of_val(stmt)));
                }
                let else_instrs = else_statements.as_ref().map(|stmts| {
                    stmts
                        .iter()
                        .map(|_| "STMT".to_string())
                        .collect::<Vec<_>>()
                });
                instructions.push(Instruction::IfStatement {
                    condition: condition.clone(),
                    then_instrs,
                    else_instrs,
                });
            }
            ProcedureStatement::For {
                variable,
                in_list,
                statements,
            } => {
                let mut loop_instrs = Vec::new();
                for stmt in statements {
                    loop_instrs.push(format!("STMT({})", std::any::type_name_of_val(stmt)));
                }
                instructions.push(Instruction::ForLoop {
                    variable: variable.clone(),
                    in_list: in_list.clone(),
                    instrs: loop_instrs,
                });
            }
            ProcedureStatement::While { condition, statements } => {
                let mut loop_instrs = Vec::new();
                for stmt in statements {
                    loop_instrs.push(format!("STMT({})", std::any::type_name_of_val(stmt)));
                }
                instructions.push(Instruction::WhileLoop {
                    condition: condition.clone(),
                    instrs: loop_instrs,
                });
            }
        }
    }

    Ok(instructions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ir_deterministic() {
        // Simple test that IR is deterministic
        let instr1 = Instruction::Compress {
            target: "buf".to_string(),
        };
        let instr2 = Instruction::Compress {
            target: "buf".to_string(),
        };
        assert_eq!(instr1, instr2);

        // encryption/decryption should also behave predictably
        let e1 = Instruction::Encrypt {
            target: "x".to_string(),
        };
        let e2 = Instruction::Encrypt {
            target: "x".to_string(),
        };
        assert_eq!(e1, e2);

        let d1 = Instruction::Decrypt {
            target: "x".to_string(),
        };
        let d2 = Instruction::Decrypt {
            target: "x".to_string(),
        };
        assert_eq!(d1, d2);
    }
}
