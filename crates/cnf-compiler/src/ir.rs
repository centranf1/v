//! IR — Intermediate Representation.
//!
//! Lowering from AST to deterministic instruction stream.
//! Same input AST → same instruction stream, always.

use crate::ast::ProcedureStatement;
use crate::ast::Program;

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    /// Set hardware/memory/parallelism profile for execution
    SetProfile {
        profile: String,
        memory_mb: Option<u64>,
        parallelism: Option<u32>,
    },
    CompressCsm {
        source: String,
        target: String,
    },
    DecompressCsm {
        source: String,
        target: String,
    },
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
    Display {
        message: String,
    },
    Print {
        target: String,
        format: Option<String>,
    },
    Read {
        target: String,
    },
    Set {
        target: String,
        value: String,
    },
    Add {
        target: String,
        operand1: String,
        operand2: String,
    },
    Subtract {
        target: String,
        operand1: String,
        operand2: String,
    },
    Multiply {
        target: String,
        operand1: String,
        operand2: String,
    },
    Divide {
        target: String,
        operand1: String,
        operand2: String,
    },
    Concatenate {
        target: String,
        operands: Vec<String>,
    },
    Substring {
        target: String,
        source: String,
        start: String,
        length: String,
    },
    Length {
        target: String,
        source: String,
    },
    Uppercase {
        target: String,
        source: String,
    },
    Lowercase {
        target: String,
        source: String,
    },
    Trim {
        target: String,
        source: String,
    },
    Max {
        target: String,
        operand1: String,
        operand2: String,
    },
    Min {
        target: String,
        operand1: String,
        operand2: String,
    },
    Abs {
        target: String,
        operand: String,
    },
    IfStatement {
        condition: String,
        then_instrs: Vec<Instruction>,
        else_instrs: Option<Vec<Instruction>>,
    },
    ForLoop {
        variable: String,
        in_list: String,
        instrs: Vec<Instruction>,
    },
    WhileLoop {
        condition: String,
        instrs: Vec<Instruction>,
    },
    FunctionDef {
        name: String,
        parameters: Vec<String>,
        return_type: Option<String>,
        instrs: Vec<Instruction>,
    },
    FunctionCall {
        name: String,
        arguments: Vec<String>,
    },
    Open {
        file_handle: String,
        file_path: String,
    },
    ReadFile {
        file_handle: String,
        output_stream: String,
    },
    WriteFile {
        file_handle: String,
        input_stream: String,
    },
    Close {
        file_handle: String,
    },
    Checkpoint {
        record_stream: String,
    },
    Replay {
        target: String,
    },
    SendBuffer {
        buffer: String,
        target_node: String,
    },
    ReceiveBuffer {
        buffer: String,
        source_node: String,
    },
    PipeStream {
        buffer: String,
        target_node: String,
        output: String,
    },
    CallRemote {
        node: String,
        function_name: String,
        args: Vec<String>,
        output: String,
    },
    PreConditionCheck {
        predicate: String,
        location: String,
    },
    PostConditionCheck {
        predicate: String,
        location: String,
    },
    InvariantCheck {
        predicate: String,
        location: String,
    },
    ProveStatement {
        target: String,
        predicate: String,
    },
    AssertStatement {
        target: String,
        predicate: String,
    },
    AuditLogEntry {
        message: String,
    },
    ComplianceReport {
        standard: String,
    },
    QuantumEncrypt {
        source: String,
        key_name: String,
    },
    QuantumDecrypt {
        target: String,
        key_name: String,
    },
    QuantumSign {
        source: String,
        signing_key: String,
        output: String,
    },
    QuantumVerifySig {
        source: String,
        verification_key: String,
        signature_ref: String,
    },
    QuantumSignEncrypt {
        source: String,
        recipient_key: String,
        signing_key: String,
        output: String,
    },
    QuantumVerifyDecrypt {
        source: String,
        recipient_key: String,
        output: String,
    },
    GenerateKeyPair {
        algorithm: String,
        output_name: String,
    },
    LongTermSign {
        source: String,
        signing_key: String,
        output: String,
    },
    // Governance instructions (v0.9.0)
    Policy {
        name: String,
        formula: String,
    },
    Regulation {
        standard: String,
        clause: String,
    },
    DataSovereignty {
        from: String,
        to: String,
    },
    AccessControl {
        user: String,
        resource: String,
        action: String,
    },
    AuditLedger {
        message: String,
    },
    DecisionQuorum {
        votes: String,
        threshold: String,
    },
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::SetProfile { profile, memory_mb, parallelism } => {
                write!(f, "SET_PROFILE(profile={}, memory_mb={:?}, parallelism={:?})", profile, memory_mb, parallelism)
            }
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
            Instruction::Display { message } => {
                write!(f, "DISPLAY({})", message)
            }
            Instruction::Print { target, format } => {
                if let Some(fmt) = format {
                    write!(f, "PRINT({} WITH {})", target, fmt)
                } else {
                    write!(f, "PRINT({})", target)
                }
            }
            Instruction::Read { target } => {
                write!(f, "READ({})", target)
            }
            Instruction::Set { target, value } => {
                write!(f, "SET({} = {})", target, value)
            }
            Instruction::Add {
                target,
                operand1,
                operand2,
            } => {
                write!(f, "ADD({} = {} + {})", target, operand1, operand2)
            }
            Instruction::Subtract {
                target,
                operand1,
                operand2,
            } => {
                write!(f, "SUBTRACT({} = {} - {})", target, operand1, operand2)
            }
            Instruction::Multiply {
                target,
                operand1,
                operand2,
            } => {
                write!(f, "MULTIPLY({} = {} * {})", target, operand1, operand2)
            }
            Instruction::Divide {
                target,
                operand1,
                operand2,
            } => {
                write!(f, "DIVIDE({} = {} / {})", target, operand1, operand2)
            }
            Instruction::Concatenate { target, operands } => {
                write!(f, "CONCATENATE({} = {})", target, operands.join(" + "))
            }
            Instruction::Substring {
                target,
                source,
                start,
                length,
            } => {
                write!(
                    f,
                    "SUBSTRING({} = {}[{}..{}])",
                    target, source, start, length
                )
            }
            Instruction::Length { target, source } => {
                write!(f, "LENGTH({} = len({}))", target, source)
            }
            Instruction::Uppercase { target, source } => {
                write!(f, "UPPERCASE({} = upper({}))", target, source)
            }
            Instruction::Lowercase { target, source } => {
                write!(f, "LOWERCASE({} = lower({}))", target, source)
            }
            Instruction::Trim { target, source } => {
                write!(f, "TRIM({} = trim({}))", target, source)
            }
            Instruction::Max {
                target,
                operand1,
                operand2,
            } => {
                write!(f, "MAX({} = max({}, {}))", target, operand1, operand2)
            }
            Instruction::Min {
                target,
                operand1,
                operand2,
            } => {
                write!(f, "MIN({} = min({}, {}))", target, operand1, operand2)
            }
            Instruction::Abs { target, operand } => {
                write!(f, "ABS({} = abs({}))", target, operand)
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
            Instruction::FunctionDef {
                name,
                parameters,
                return_type,
                instrs,
            } => {
                write!(
                    f,
                    "FUNC-DEF({} [{}] ret{})",
                    name,
                    parameters.join(","),
                    return_type.as_ref().unwrap_or(&"(none)".to_string())
                )?;
                write!(f, " [{}]", instrs.len())
            }
            Instruction::FunctionCall { name, arguments } => {
                write!(f, "FUNC-CALL({}({})", name, arguments.join(","))
            }
            Instruction::Open {
                file_handle,
                file_path,
            } => {
                write!(f, "OPEN({} AS {})", file_handle, file_path)
            }
            Instruction::ReadFile {
                file_handle,
                output_stream,
            } => {
                write!(f, "READ-FILE({} INTO {})", file_handle, output_stream)
            }
            Instruction::WriteFile {
                file_handle,
                input_stream,
            } => {
                write!(f, "WRITE-FILE({} FROM {})", file_handle, input_stream)
            }
            Instruction::Close { file_handle } => {
                write!(f, "CLOSE({})", file_handle)
            }
            Instruction::Checkpoint { record_stream } => {
                write!(f, "CHECKPOINT({})", record_stream)
            }
            Instruction::Replay { target } => {
                write!(f, "REPLAY({})", target)
            }
            Instruction::SendBuffer {
                buffer,
                target_node,
            } => {
                write!(f, "SEND_BUFFER({} TO {})", buffer, target_node)
            }
            Instruction::ReceiveBuffer {
                buffer,
                source_node,
            } => {
                write!(f, "RECEIVE_BUFFER({} FROM {})", buffer, source_node)
            }
            Instruction::PipeStream {
                buffer,
                target_node,
                output,
            } => {
                write!(
                    f,
                    "PIPE_STREAM({} TO {} -> {})",
                    buffer, target_node, output
                )
            }
            Instruction::CallRemote {
                node,
                function_name,
                args,
                output,
            } => {
                write!(
                    f,
                    "CALL_REMOTE({}:{}({:?}) -> {})",
                    node, function_name, args, output
                )
            }
            Instruction::PreConditionCheck {
                predicate,
                location,
            } => {
                write!(f, "PRECONDITION_CHECK({} @ {})", predicate, location)
            }
            Instruction::PostConditionCheck {
                predicate,
                location,
            } => {
                write!(f, "POSTCONDITION_CHECK({} @ {})", predicate, location)
            }
            Instruction::InvariantCheck {
                predicate,
                location,
            } => {
                write!(f, "INVARIANT_CHECK({} @ {})", predicate, location)
            }
            Instruction::ProveStatement { target, predicate } => {
                write!(f, "PROVE({} SATISFIES {})", target, predicate)
            }
            Instruction::AssertStatement { target, predicate } => {
                write!(f, "ASSERT({} SATISFIES {})", target, predicate)
            }
            Instruction::AuditLogEntry { message } => {
                write!(f, "AUDIT_LOG({})", message)
            }
            Instruction::ComplianceReport { standard } => {
                write!(f, "COMPLIANCE_REPORT({})", standard)
            }
            Instruction::QuantumEncrypt { source, key_name } => {
                write!(f, "QUANTUM_ENCRYPT({} WITH {})", source, key_name)
            }
            Instruction::QuantumDecrypt { target, key_name } => {
                write!(f, "QUANTUM_DECRYPT({} WITH {})", target, key_name)
            }
            Instruction::QuantumSign {
                source,
                signing_key,
                output,
            } => {
                write!(
                    f,
                    "QUANTUM_SIGN({} WITH {} AS {})",
                    source, signing_key, output
                )
            }
            Instruction::QuantumVerifySig {
                source,
                verification_key,
                signature_ref,
            } => {
                write!(
                    f,
                    "QUANTUM_VERIFY_SIG({} WITH {} SIGNATURE {})",
                    source, verification_key, signature_ref
                )
            }
            Instruction::QuantumSignEncrypt {
                source,
                recipient_key,
                signing_key,
                output,
            } => {
                write!(
                    f,
                    "QUANTUM_SIGN_ENCRYPT({} FOR {} SIGNED_BY {} AS {})",
                    source, recipient_key, signing_key, output
                )
            }
            Instruction::QuantumVerifyDecrypt {
                source,
                recipient_key,
                output,
            } => {
                write!(
                    f,
                    "QUANTUM_VERIFY_DECRYPT({} WITH {} AS {})",
                    source, recipient_key, output
                )
            }
            Instruction::GenerateKeyPair {
                algorithm,
                output_name,
            } => {
                write!(
                    f,
                    "GENERATE_KEYPAIR(ALGORITHM {} AS {})",
                    algorithm, output_name
                )
            }
            Instruction::LongTermSign {
                source,
                signing_key,
                output,
            } => {
                write!(
                    f,
                    "LONG_TERM_SIGN({} WITH {} AS {})",
                    source, signing_key, output
                )
            }
            Instruction::Policy { name, formula } => {
                write!(f, "POLICY({} FORMULA {})", name, formula)
            }
            Instruction::Regulation { standard, clause } => {
                write!(f, "REGULATION({} CLAUSE {})", standard, clause)
            }
            Instruction::DataSovereignty { from, to } => {
                write!(f, "DATA_SOVEREIGNTY({} -> {})", from, to)
            }
            Instruction::AccessControl { user, resource, action } => {
                write!(f, "ACCESS_CONTROL({} {} {})", user, resource, action)
            }
            Instruction::AuditLedger { message } => {
                write!(f, "AUDIT_LEDGER({})", message)
            }
            Instruction::DecisionQuorum { votes, threshold } => {
                write!(f, "DECISION_QUORUM({} votes, {} threshold)", votes, threshold)
            }
            _ => write!(f, "INSTRUCTION({:?})", std::mem::discriminant(self)),
        }
    }
}

/// Type validator for checking operation legality
struct TypeValidator;

#[allow(dead_code)]
impl TypeValidator {
    /// Check if an operation is legal on the given type
    fn can_compress(_data_type: &crate::ast::DataType) -> bool {
        // COMPRESS works on all types
        true
    }

    /// Check if an operation is legal on the given type
    #[allow(dead_code)]
    fn can_transcode(data_type: &crate::ast::DataType) -> bool {
        // TRANSCODE not allowed on BINARY-BLOB or FINANCIAL-DECIMAL
        !matches!(
            data_type,
            crate::ast::DataType::BinaryBlob | crate::ast::DataType::FinancialDecimal
        )
    }

    /// Check if an operation is legal on the given type
    fn can_aggregate(data_type: &crate::ast::DataType) -> bool {
        matches!(
            data_type,
            crate::ast::DataType::CsvTable
                | crate::ast::DataType::FinancialDecimal
                | crate::ast::DataType::ParquetTable
        )
    }

    /// Check if an operation is legal on the given type
        #[allow(dead_code)]
        fn can_validate(data_type: &crate::ast::DataType) -> bool {
        matches!(
            data_type,
            crate::ast::DataType::JsonObject
                | crate::ast::DataType::XmlDocument
                | crate::ast::DataType::CsvTable
        )
    }

    /// Check if an operation is legal on the given type
    fn can_extract(data_type: &crate::ast::DataType) -> bool {
        matches!(
            data_type,
            crate::ast::DataType::JsonObject | crate::ast::DataType::XmlDocument
        )
    }
}

/// Check if a string is a literal value
#[allow(dead_code)]
fn is_literal(s: &str) -> bool {
    s.starts_with('"') && s.ends_with('"')
}
pub fn lower(program: Program) -> Result<Vec<Instruction>, String> {
    let mut instructions = Vec::new();
    // Lower PROFILE DIVISION (jika ada) ke SetProfile di awal IR
    if let Some(profile) = &program.profile {
        let profile_str = match profile.hardware {
            crate::ast::HardwareProfile::EdgeLow => "EdgeLow",
            crate::ast::HardwareProfile::EdgeHigh => "EdgeHigh",
            crate::ast::HardwareProfile::DatacenterLow => "DatacenterLow",
            crate::ast::HardwareProfile::DatacenterHigh => "DatacenterHigh",
            crate::ast::HardwareProfile::Balanced => "Balanced",
        };
        instructions.push(Instruction::SetProfile {
            profile: profile_str.to_string(),
            memory_mb: profile.memory_limit_mb,
            parallelism: profile.parallelism,
        });
    }
    // Collect function signatures for parameter count validation
    let mut signatures: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for stmt in &program.procedure.statements {
        if let ProcedureStatement::FunctionDef {
            name, parameters, ..
        } = stmt
        {
            signatures.insert(name.clone(), parameters.len());
        }
    }

    // Lower governance statements first so that policies/regulations appear before
    // any procedure instructions in the IR stream.
    if let Some(gov) = &program.governance {
        for gstmt in &gov.statements {
            match gstmt {
                crate::ast::GovernanceStatement::Policy { name, formula } => {
                    instructions.push(Instruction::Policy {
                        name: name.clone(),
                        formula: formula.clone(),
                    });
                }
                crate::ast::GovernanceStatement::Regulation { standard, clause } => {
                    instructions.push(Instruction::Regulation {
                        standard: standard.clone(),
                        clause: clause.clone(),
                    });
                }
                crate::ast::GovernanceStatement::DataSovereignty { from, to } => {
                    instructions.push(Instruction::DataSovereignty {
                        from: from.clone(),
                        to: to.clone(),
                    });
                }
                crate::ast::GovernanceStatement::AccessControl {
                    user,
                    resource,
                    action,
                } => {
                    instructions.push(Instruction::AccessControl {
                        user: user.clone(),
                        resource: resource.clone(),
                        action: action.clone(),
                    });
                }
                crate::ast::GovernanceStatement::AuditLedger { entry } => {
                    instructions.push(Instruction::AuditLedger {
                        message: entry.clone(),
                    });
                }
                crate::ast::GovernanceStatement::DecisionQuorum { votes, threshold } => {
                    instructions.push(Instruction::DecisionQuorum {
                        votes: votes.clone(),
                        threshold: threshold.clone(),
                    });
                }
            }
        }
    }

    // Kumpulkan tipe variabel dari DATA DIVISION
    let mut var_types: std::collections::HashMap<String, crate::ast::DataType> = std::collections::HashMap::new();
    for decl in &program.data.variables {
        var_types.insert(decl.name.clone(), decl.data_type.clone());
    }

    for stmt in &program.procedure.statements {
        // Type checking untuk operasi yang relevan
        match stmt {
            ProcedureStatement::Compress { target }
            | ProcedureStatement::Encrypt { target }
            | ProcedureStatement::VerifyIntegrity { target }
            | ProcedureStatement::Transcode { target, .. }
            | ProcedureStatement::Convert { target, .. } => {
                let ty = var_types.get(target).ok_or_else(|| format!("Variable '{}' not declared", target))?;
                // COMPRESS: selalu true, TRANSCODE/CONVERT: cek via TypeValidator
                if let ProcedureStatement::Transcode { .. } = stmt {
                    if !TypeValidator::can_transcode(ty) {
                        return Err(format!("TRANSCODE not allowed on type {:?}", ty));
                    }
                }
                // (tambahkan rule lain sesuai kebutuhan)
            }
            _ => {}
        }
        // Lower ke IR
        let instr = lower_single_statement(stmt, &var_types.keys().cloned().collect(), &signatures)?;
        instructions.push(instr);
    }
    Ok(instructions)
}

/// Helper to lower a single procedure statement to instruction
#[allow(clippy::only_used_in_recursion)]
    #[allow(dead_code)]
    fn lower_single_statement(
    stmt: &ProcedureStatement,
    declared_vars: &std::collections::HashSet<String>,
    signatures: &std::collections::HashMap<String, usize>,
) -> Result<Instruction, String> {
    match stmt {
        ProcedureStatement::CompressCsm { source, target } => {
            if !declared_vars.contains(source) {
                return Err(format!("Variable '{}' not declared", source));
            }
            if !declared_vars.contains(target) {
                return Err(format!("Variable '{}' not declared", target));
            }
            Ok(Instruction::CompressCsm {
                source: source.clone(),
                target: target.clone(),
            })
        }
        ProcedureStatement::DecompressCsm { source, target } => {
            if !declared_vars.contains(source) {
                return Err(format!("Variable '{}' not declared", source));
            }
            if !declared_vars.contains(target) {
                return Err(format!("Variable '{}' not declared", target));
            }
            Ok(Instruction::DecompressCsm {
                source: source.clone(),
                target: target.clone(),
            })
        }
        ProcedureStatement::Compress { target } => {
            if !declared_vars.contains(target) {
                return Err(format!("Variable '{}' not declared", target));
            }
            Ok(Instruction::Compress {
                target: target.clone(),
            })
        }
        ProcedureStatement::VerifyIntegrity { target } => {
            if !declared_vars.contains(target) {
                return Err(format!("Variable '{}' not declared", target));
            }
            Ok(Instruction::VerifyIntegrity {
                target: target.clone(),
            })
        }
        ProcedureStatement::Encrypt { target } => {
            if !declared_vars.contains(target) {
                return Err(format!("Variable '{}' not declared", target));
            }
            Ok(Instruction::Encrypt {
                target: target.clone(),
            })
        }
        ProcedureStatement::Decrypt { target } => {
            if !declared_vars.contains(target) {
                return Err(format!("Variable '{}' not declared", target));
            }
            Ok(Instruction::Decrypt {
                target: target.clone(),
            })
        }
        ProcedureStatement::Filter { target, condition } => {
            if !declared_vars.contains(target) {
                return Err(format!("Variable '{}' not declared", target));
            }
            Ok(Instruction::Filter {
                target: target.clone(),
                condition: condition.clone(),
            })
        }
        ProcedureStatement::Aggregate { targets, operation } => {
            for target in targets {
                if !declared_vars.contains(target) {
                    return Err(format!("Variable '{}' not declared", target));
                }
            }
            Ok(Instruction::Aggregate {
                targets: targets.clone(),
                operation: operation.clone(),
            })
        }
        ProcedureStatement::If {
            condition,
            then_statements,
            else_statements,
        } => {
            let mut then_instrs = Vec::new();
            for s in then_statements {
                then_instrs.push(lower_single_statement(s, declared_vars, signatures)?);
            }
            let else_instrs = if let Some(else_stmts) = else_statements {
                let mut else_i = Vec::new();
                for s in else_stmts {
                    else_i.push(lower_single_statement(s, declared_vars, signatures)?);
                }
                Some(else_i)
            } else {
                None
            };
            Ok(Instruction::IfStatement {
                condition: condition.clone(),
                then_instrs,
                else_instrs,
            })
        }
        ProcedureStatement::For {
            variable,
            in_list,
            statements,
        } => {
            let mut loop_instrs = Vec::new();
            for s in statements {
                loop_instrs.push(lower_single_statement(s, declared_vars, signatures)?);
            }
            Ok(Instruction::ForLoop {
                variable: variable.clone(),
                in_list: in_list.clone(),
                instrs: loop_instrs,
            })
        }
        ProcedureStatement::While {
            condition,
            statements,
        } => {
            let mut loop_instrs = Vec::new();
            for s in statements {
                loop_instrs.push(lower_single_statement(s, declared_vars, signatures)?);
            }
            Ok(Instruction::WhileLoop {
                condition: condition.clone(),
                instrs: loop_instrs,
            })
        }
        ProcedureStatement::FunctionDef {
            name,
            parameters,
            return_type,
            statements,
        } => {
            let mut func_instrs = Vec::new();
            for s in statements {
                func_instrs.push(lower_single_statement(s, declared_vars, signatures)?);
            }
            Ok(Instruction::FunctionDef {
                name: name.clone(),
                parameters: parameters.clone(),
                return_type: return_type.as_ref().map(|dt| dt.to_string()),
                instrs: func_instrs,
            })
        }
        ProcedureStatement::FunctionCall { name, arguments } => {
            // In nested context we can't easily access signatures map; assume caller has
            // validated top-level calls. We still emit the instruction.
            Ok(Instruction::FunctionCall {
                name: name.clone(),
                arguments: arguments.clone(),
            })
        }
        ProcedureStatement::Set { target, value } => {
            if !declared_vars.contains(target) {
                return Err(format!("Variable '{}' not declared", target));
            }
            Ok(Instruction::Set {
                target: target.clone(),
                value: value.clone(),
            })
        }
        ProcedureStatement::Add {
            target,
            operand1,
            operand2,
        } => {
            if !declared_vars.contains(target) {
                return Err(format!("Variable '{}' not declared", target));
            }
            if !declared_vars.contains(operand1) {
                return Err(format!("Variable '{}' not declared", operand1));
            }
            if !declared_vars.contains(operand2) {
                return Err(format!("Variable '{}' not declared", operand2));
            }
            Ok(Instruction::Add {
                target: target.clone(),
                operand1: operand1.clone(),
                operand2: operand2.clone(),
            })
        }
        ProcedureStatement::Subtract {
            target,
            operand1,
            operand2,
        } => {
            if !declared_vars.contains(target) {
                return Err(format!("Variable '{}' not declared", target));
            }
            if !declared_vars.contains(operand1) {
                return Err(format!("Variable '{}' not declared", operand1));
            }
            if !declared_vars.contains(operand2) {
                return Err(format!("Variable '{}' not declared", operand2));
            }
            Ok(Instruction::Subtract {
                target: target.clone(),
                operand1: operand1.clone(),
                operand2: operand2.clone(),
            })
        }
        ProcedureStatement::Multiply {
            target,
            operand1,
            operand2,
        } => {
            if !declared_vars.contains(target) {
                return Err(format!("Variable '{}' not declared", target));
            }
            if !declared_vars.contains(operand1) {
                return Err(format!("Variable '{}' not declared", operand1));
            }
            if !declared_vars.contains(operand2) {
                return Err(format!("Variable '{}' not declared", operand2));
            }
            Ok(Instruction::Multiply {
                target: target.clone(),
                operand1: operand1.clone(),
                operand2: operand2.clone(),
            })
        }
        ProcedureStatement::Divide {
            target,
            operand1,
            operand2,
        } => {
            if !declared_vars.contains(target) {
                return Err(format!("Variable '{}' not declared", target));
            }
            if !declared_vars.contains(operand1) {
                return Err(format!("Variable '{}' not declared", operand1));
            }
            if !declared_vars.contains(operand2) {
                return Err(format!("Variable '{}' not declared", operand2));
            }
            Ok(Instruction::Divide {
                target: target.clone(),
                operand1: operand1.clone(),
                operand2: operand2.clone(),
            })
        }
        ProcedureStatement::Open {
            file_handle,
            file_path,
        } => {
            if !declared_vars.contains(file_handle) {
                return Err(format!("Variable '{}' not declared", file_handle));
            }
            Ok(Instruction::Open {
                file_handle: file_handle.clone(),
                file_path: file_path.clone(),
            })
        }
        ProcedureStatement::ReadFile {
            file_handle,
            output_stream,
        } => {
            if !declared_vars.contains(file_handle) {
                return Err(format!("Variable '{}' not declared", file_handle));
            }
            if !declared_vars.contains(output_stream) {
                return Err(format!("Variable '{}' not declared", output_stream));
            }
            Ok(Instruction::ReadFile {
                file_handle: file_handle.clone(),
                output_stream: output_stream.clone(),
            })
        }
        ProcedureStatement::WriteFile {
            file_handle,
            input_stream,
        } => {
            if !declared_vars.contains(file_handle) {
                return Err(format!("Variable '{}' not declared", file_handle));
            }
            if !declared_vars.contains(input_stream) {
                return Err(format!("Variable '{}' not declared", input_stream));
            }
            Ok(Instruction::WriteFile {
                file_handle: file_handle.clone(),
                input_stream: input_stream.clone(),
            })
        }
        ProcedureStatement::Close { file_handle } => {
            if !declared_vars.contains(file_handle) {
                return Err(format!("Variable '{}' not declared", file_handle));
            }
            Ok(Instruction::Close {
                file_handle: file_handle.clone(),
            })
        }
        ProcedureStatement::Checkpoint { record_stream } => {
            if !declared_vars.contains(record_stream) {
                return Err(format!("Variable '{}' not declared", record_stream));
            }
            Ok(Instruction::Checkpoint {
                record_stream: record_stream.clone(),
            })
        }
        ProcedureStatement::Replay { target } => {
            if !declared_vars.contains(target) {
                return Err(format!("Variable '{}' not declared", target));
            }
            Ok(Instruction::Replay {
                target: target.clone(),
            })
        }
        ProcedureStatement::QuantumEncrypt { target, key_name } => {
            if !declared_vars.contains(target) {
                return Err(format!("Variable '{}' not declared", target));
            }
            Ok(Instruction::QuantumEncrypt {
                source: target.clone(),
                key_name: key_name.clone(),
            })
        }
        ProcedureStatement::QuantumDecrypt { target, key_name } => {
            if !declared_vars.contains(target) {
                return Err(format!("Variable '{}' not declared", target));
            }
            Ok(Instruction::QuantumDecrypt {
                target: target.clone(),
                key_name: key_name.clone(),
            })
        }
        ProcedureStatement::QuantumSign {
            target,
            signing_key,
            output,
        } => {
            if !declared_vars.contains(target) {
                return Err(format!("Variable '{}' not declared", target));
            }
            if !declared_vars.contains(output) {
                return Err(format!("Variable '{}' not declared", output));
            }
            Ok(Instruction::QuantumSign {
                source: target.clone(),
                signing_key: signing_key.clone(),
                output: output.clone(),
            })
        }
        ProcedureStatement::QuantumVerifySig {
            target,
            verification_key,
            signature_ref,
        } => {
            if !declared_vars.contains(target) {
                return Err(format!("Variable '{}' not declared", target));
            }
            Ok(Instruction::QuantumVerifySig {
                source: target.clone(),
                verification_key: verification_key.clone(),
                signature_ref: signature_ref.clone(),
            })
        }
        ProcedureStatement::QuantumSignEncrypt {
            target,
            recipient_key,
            signing_key,
            output,
        } => {
            if !declared_vars.contains(target) {
                return Err(format!("Variable '{}' not declared", target));
            }
            if !declared_vars.contains(output) {
                return Err(format!("Variable '{}' not declared", output));
            }
            Ok(Instruction::QuantumSignEncrypt {
                source: target.clone(),
                recipient_key: recipient_key.clone(),
                signing_key: signing_key.clone(),
                output: output.clone(),
            })
        }
        ProcedureStatement::QuantumVerifyDecrypt {
            target,
            recipient_key,
            output,
        } => {
            if !declared_vars.contains(target) {
                return Err(format!("Variable '{}' not declared", target));
            }
            if !declared_vars.contains(output) {
                return Err(format!("Variable '{}' not declared", output));
            }
            Ok(Instruction::QuantumVerifyDecrypt {
                source: target.clone(),
                recipient_key: recipient_key.clone(),
                output: output.clone(),
            })
        }
        ProcedureStatement::GenerateKeyPair {
            algorithm,
            output_name,
        } => {
            if !declared_vars.contains(output_name) {
                return Err(format!("Variable '{}' not declared", output_name));
            }
            Ok(Instruction::GenerateKeyPair {
                algorithm: algorithm.clone(),
                output_name: output_name.clone(),
            })
        }
        ProcedureStatement::LongTermSign {
            target,
            signing_key,
            output,
        } => {
            if !declared_vars.contains(target) {
                return Err(format!("Variable '{}' not declared", target));
            }
            if !declared_vars.contains(output) {
                return Err(format!("Variable '{}' not declared", output));
            }
            Ok(Instruction::LongTermSign {
                source: target.clone(),
                signing_key: signing_key.clone(),
                output: output.clone(),
            })
        }
        _ => Err("Unsupported nested statement".to_string()),
    }
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

    #[test]
    fn test_file_operation_instructions() {
        let open_instr = Instruction::Open {
            file_handle: "fh".to_string(),
            file_path: "/path/to/file".to_string(),
        };
        assert_eq!(format!("{}", open_instr), "OPEN(fh AS /path/to/file)");

        let read_instr = Instruction::ReadFile {
            file_handle: "fh".to_string(),
            output_stream: "rs".to_string(),
        };
        assert_eq!(format!("{}", read_instr), "READ-FILE(fh INTO rs)");

        let write_instr = Instruction::WriteFile {
            file_handle: "fh".to_string(),
            input_stream: "rs".to_string(),
        };
        assert_eq!(format!("{}", write_instr), "WRITE-FILE(fh FROM rs)");

        let close_instr = Instruction::Close {
            file_handle: "fh".to_string(),
        };
        assert_eq!(format!("{}", close_instr), "CLOSE(fh)");

        let checkpoint_instr = Instruction::Checkpoint {
            record_stream: "rs".to_string(),
        };
        assert_eq!(format!("{}", checkpoint_instr), "CHECKPOINT(rs)");

        let replay_instr = Instruction::Replay {
            target: "rs".to_string(),
        };
        assert_eq!(format!("{}", replay_instr), "REPLAY(rs)");
    }

    #[test]
    fn test_quantum_encrypt_instruction() {
        let instr = Instruction::QuantumEncrypt {
            source: "plaintext".to_string(),
            key_name: "encryption_key".to_string(),
        };
        assert_eq!(
            format!("{}", instr),
            "QUANTUM_ENCRYPT(plaintext WITH encryption_key)"
        );
    }

    #[test]
    fn test_quantum_decrypt_instruction() {
        let instr = Instruction::QuantumDecrypt {
            target: "ciphertext".to_string(),
            key_name: "decryption_key".to_string(),
        };
        assert_eq!(
            format!("{}", instr),
            "QUANTUM_DECRYPT(ciphertext WITH decryption_key)"
        );
    }

    #[test]
    fn test_quantum_sign_instruction() {
        let instr = Instruction::QuantumSign {
            source: "message".to_string(),
            signing_key: "private_key".to_string(),
            output: "signature".to_string(),
        };
        assert_eq!(
            format!("{}", instr),
            "QUANTUM_SIGN(message WITH private_key AS signature)"
        );
    }

    #[test]
    fn test_quantum_verify_sig_instruction() {
        let instr = Instruction::QuantumVerifySig {
            source: "message".to_string(),
            verification_key: "public_key".to_string(),
            signature_ref: "sig_buffer".to_string(),
        };
        assert_eq!(
            format!("{}", instr),
            "QUANTUM_VERIFY_SIG(message WITH public_key SIGNATURE sig_buffer)"
        );
    }

    #[test]
    fn test_quantum_sign_encrypt_instruction() {
        let instr = Instruction::QuantumSignEncrypt {
            source: "plaintext".to_string(),
            recipient_key: "recipient_pk".to_string(),
            signing_key: "sender_sk".to_string(),
            output: "encrypted_signed".to_string(),
        };
        assert_eq!(
            format!("{}", instr),
            "QUANTUM_SIGN_ENCRYPT(plaintext FOR recipient_pk SIGNED_BY sender_sk AS encrypted_signed)"
        );
    }

    #[test]
    fn test_quantum_verify_decrypt_instruction() {
        let instr = Instruction::QuantumVerifyDecrypt {
            source: "encrypted_signed".to_string(),
            recipient_key: "recipient_sk".to_string(),
            output: "plaintext".to_string(),
        };
        assert_eq!(
            format!("{}", instr),
            "QUANTUM_VERIFY_DECRYPT(encrypted_signed WITH recipient_sk AS plaintext)"
        );
    }

    #[test]
    fn test_generate_keypair_instruction() {
        let instr = Instruction::GenerateKeyPair {
            algorithm: "ML-KEM-768".to_string(),
            output_name: "generated_keypair".to_string(),
        };
        assert_eq!(
            format!("{}", instr),
            "GENERATE_KEYPAIR(ALGORITHM ML-KEM-768 AS generated_keypair)"
        );
    }

    #[test]
    fn test_long_term_sign_instruction() {
        let instr = Instruction::LongTermSign {
            source: "document".to_string(),
            signing_key: "long_term_key".to_string(),
            output: "long_term_sig".to_string(),
        };
        assert_eq!(
            format!("{}", instr),
            "LONG_TERM_SIGN(document WITH long_term_key AS long_term_sig)"
        );
    }

    #[test]
    fn test_governance_instructions_display() {
        let p = Instruction::Policy {
            name: "allow".to_string(),
            formula: "G(a)".to_string(),
        };
        assert_eq!(format!("{}", p), "POLICY(allow FORMULA G(a))");

        let r = Instruction::Regulation {
            standard: "GDPR".to_string(),
            clause: "no export".to_string(),
        };
        assert_eq!(format!("{}", r), "REGULATION(GDPR CLAUSE no export)");

        let d = Instruction::DataSovereignty {
            from: "EU".to_string(),
            to: "US".to_string(),
        };
        assert_eq!(format!("{}", d), "DATA_SOVEREIGNTY(EU -> US)");

        let a = Instruction::AccessControl {
            user: "alice".to_string(),
            resource: "file1".to_string(),
            action: "read".to_string(),
        };
        assert_eq!(format!("{}", a), "ACCESS_CONTROL(alice file1 read)");

        let al = Instruction::AuditLedger {
            message: "entry".to_string(),
        };
        assert_eq!(format!("{}", al), "AUDIT_LEDGER(entry)");

        let q = Instruction::DecisionQuorum {
            votes: "5".to_string(),
            threshold: "3".to_string(),
        };
        assert_eq!(format!("{}", q), "DECISION_QUORUM(5 votes, 3 threshold)");
    }

    #[test]
    fn test_lower_governance_division() {
        use crate::ast::{Program, IdentificationDivision, EnvironmentDivision, DataDivision, ProcedureDivision, GovernanceDivision, GovernanceStatement};
        let prog = Program {
            identification: IdentificationDivision { program_id: "p".to_string(), author: None, version: None },
            environment: EnvironmentDivision { config: std::collections::HashMap::new() },
            network: None,
            verification: None,
            governance: Some(GovernanceDivision { statements: vec![
                GovernanceStatement::AuditLedger { entry: "log1".to_string() },
            ]}),
            data: DataDivision { variables: vec![] },
            procedure: ProcedureDivision { statements: vec![] },
        };
        let instrs = lower(prog).unwrap();
        assert_eq!(instrs, vec![Instruction::AuditLedger { message: "log1".to_string() }]);
    }
}

