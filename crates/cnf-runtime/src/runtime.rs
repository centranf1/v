//! Runtime — CENTRA-NF execution engine.
//!
//! Execute IR instructions against owned buffers.
//! Fail fast on invalid operations.
//! No global mutable state.

use std::fmt;

/// Error type for CENTRA-NF runtime
#[derive(Debug, Clone)]
pub enum CnfError {
    BufferNotFound(String),
    InvalidInstruction(String),
    EncryptionFailed(String),
    DecryptionFailed(String),
    IoError(String),
    RuntimeError(String),
    CsmError(String),
    DivisionByZero,
    LoopLimitExceeded(usize),
    AccessDenied { user: String, resource: String, action: String },
    GovernancePolicyViolation(String),
    BufferCorrupted(String),
    IntegrityViolation(String),
    #[cfg(feature = "verifier")]
    PreconditionFailed(String),
    #[cfg(feature = "verifier")]
    PostconditionFailed(String),
    #[cfg(feature = "verifier")]
    InvariantViolated(String),
    #[cfg(feature = "verifier")]
    ProofNotFound(String),
    #[cfg(feature = "verifier")]
    AssertionFailed(String),
    #[cfg(feature = "verifier")]
    AuditChainError(String),
}

impl std::fmt::Display for CnfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CnfError::BufferNotFound(name) => write!(f, "Buffer not found: {}", name),
            CnfError::InvalidInstruction(msg) => write!(f, "Invalid instruction: {}", msg),
            CnfError::EncryptionFailed(msg) => write!(f, "Encryption failed: {}", msg),
            CnfError::DecryptionFailed(msg) => write!(f, "Decryption failed: {}", msg),
            CnfError::IoError(msg) => write!(f, "I/O error: {}", msg),
            CnfError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
            CnfError::CsmError(msg) => write!(f, "CSM error: {}", msg),
            CnfError::DivisionByZero => write!(f, "Division by zero"),
            CnfError::LoopLimitExceeded(limit) => write!(f, "Loop limit exceeded: {}", limit),
            CnfError::AccessDenied { user, resource, action } => write!(f, "Access denied: {} cannot {} on {}", user, action, resource),
            CnfError::GovernancePolicyViolation(msg) => write!(f, "Governance policy violation: {}", msg),
            CnfError::BufferCorrupted(msg) => write!(f, "Buffer corrupted: {}", msg),
            CnfError::IntegrityViolation(msg) => write!(f, "Integrity violation: {}", msg),
            #[cfg(feature = "verifier")]
            CnfError::PreconditionFailed(loc) => write!(f, "Precondition failed at {}", loc),
            #[cfg(feature = "verifier")]
            CnfError::PostconditionFailed(loc) => write!(f, "Postcondition failed at {}", loc),
            #[cfg(feature = "verifier")]
            CnfError::InvariantViolated(loc) => write!(f, "Invariant violated at {}", loc),
            #[cfg(feature = "verifier")]
            CnfError::ProofNotFound(target) => write!(f, "Proof not found for {}", target),
            #[cfg(feature = "verifier")]
            CnfError::AssertionFailed(target) => write!(f, "Assertion failed: {}", target),
            #[cfg(feature = "verifier")]
            CnfError::AuditChainError(msg) => write!(f, "Audit chain error: {}", msg),
        }
    }
}

impl std::error::Error for CnfError {}

/// Runtime value representation for Phase 3+
/// Supports multiple data types for deterministic operations
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    /// Integer number (64-bit signed)
    Integer(i64),
    /// Floating-point number (64-bit)
    Decimal(f64),
    /// Binary buffer (bytes)
    Binary(Vec<u8>),
    /// Text string
    Text(String),
    /// List of values
    List(Vec<RuntimeValue>),
}

impl RuntimeValue {
    /// Convert to integer if possible (fail-fast)
    pub fn as_integer(&self) -> Result<i64, CnfError> {
        match self {
            RuntimeValue::Integer(n) => Ok(*n),
            RuntimeValue::Decimal(d) => Ok(*d as i64),
            _ => Err(CnfError::RuntimeError(
                format!("Expected integer, got {:?}", self),
            )),
        }
    }

    /// Convert to decimal if possible (fail-fast)
    pub fn as_decimal(&self) -> Result<f64, CnfError> {
        match self {
            RuntimeValue::Integer(n) => Ok(*n as f64),
            RuntimeValue::Decimal(d) => Ok(*d),
            _ => Err(CnfError::RuntimeError(
                format!("Expected decimal, got {:?}", self),
            )),
        }
    }

    /// Convert to binary if possible (fail-fast)
    pub fn as_binary(&self) -> Result<Vec<u8>, CnfError> {
        match self {
            RuntimeValue::Binary(b) => Ok(b.clone()),
            RuntimeValue::Text(s) => Ok(s.as_bytes().to_vec()),
            _ => Err(CnfError::RuntimeError(
                format!("Expected binary/text, got {:?}", self),
            )),
        }
    }
}

impl fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeValue::Integer(n) => write!(f, "{}", n),
            RuntimeValue::Decimal(d) => write!(f, "{}", d),
            RuntimeValue::Binary(b) => write!(f, "{}", hex::encode(b)),
            RuntimeValue::Text(s) => write!(f, "{}", s),
            RuntimeValue::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
        }
    }
}

/// Variable store for runtime state
pub struct VariableStore {
    variables: std::collections::HashMap<String, RuntimeValue>,
}

impl VariableStore {
    pub fn new() -> Self {
        VariableStore {
            variables: std::collections::HashMap::new(),
        }
    }

    pub fn set(&mut self, name: String, value: RuntimeValue) {
        self.variables.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<RuntimeValue> {
        self.variables.get(name).cloned()
    }

    pub fn get_or_default(&self, name: &str) -> RuntimeValue {
        self.variables
            .get(name)
            .cloned()
            .unwrap_or(RuntimeValue::Integer(0))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &RuntimeValue)> {
        self.variables.iter()
    }
}

impl Default for VariableStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Runtime execution engine
/// Executes IR instructions deterministically with phase-by-phase dispatch
pub struct Runtime {
    variables: VariableStore,
    /// Aktif profile dari PROFILE DIVISION (jika ada)
    pub active_profile: Option<(String, Option<u64>, Option<u32>)>,
    /// CSM dictionary for COMPRESS-CSM / DECOMPRESS-CSM instructions
    pub csm_dict: Option<cobol_protocol_v154::dictionary::CsmDictionary>,
    /// Audit log entries (append-only, never cleared during execution)
    pub audit_log: Vec<String>,
    /// Access control rules: (user, resource) -> allowed actions
    access_rules: std::collections::HashMap<(String, String), Vec<String>>,
    /// Active governance policies: name -> LTL formula string
    governance_policies: std::collections::HashMap<String, String>,
    #[cfg(feature = "governance")]
    governance: cnf_governance::policy_engine::PolicyEngine,
    #[cfg(feature = "governance")]
    governance_trace: cnf_governance::policy_engine::ExecutionTrace,
    /// Execution trace for LTL evaluation
    execution_trace: Vec<String>,
    #[cfg(feature = "quantum")]
    quantum_keys: std::collections::HashMap<String, (Vec<u8>, Vec<u8>)>,
}

impl Runtime {
    /// Create new runtime with empty variable store and fresh scope manager
    pub fn new() -> Self {
        Runtime {
            variables: VariableStore::new(),
            csm_dict: None,
            audit_log: Vec::new(),
            access_rules: std::collections::HashMap::new(),
            governance_policies: std::collections::HashMap::new(),
            #[cfg(feature = "governance")]
            governance: cnf_governance::policy_engine::PolicyEngine::new(),
            #[cfg(feature = "governance")]
            governance_trace: cnf_governance::policy_engine::ExecutionTrace::default(),
            execution_trace: Vec::new(),
            #[cfg(feature = "quantum")]
            quantum_keys: std::collections::HashMap::new(),
            active_profile: None,
        }
    }

    /// Add a buffer (binary data) as a variable
    pub fn add_buffer(&mut self, name: String, data: Vec<u8>) {
        self.variables.set(name, RuntimeValue::Binary(data));
    }

    /// List all buffers with their current state
    pub fn list_buffers(&self) -> Vec<(String, Vec<u8>)> {
        self.variables
            .iter()
            .filter_map(|(name, value)| {
                if let RuntimeValue::Binary(data) = value {
                    Some((name.clone(), data.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get a variable's value as bytes (helper for tests)
    pub fn get_output(&self, name: &str) -> Option<Vec<u8>> {
        match self.variables.get(name)? {
            RuntimeValue::Binary(b) => Some(b),
            RuntimeValue::Text(s) => Some(s.as_bytes().to_vec()),
            RuntimeValue::Integer(n) => Some(n.to_string().as_bytes().to_vec()),
            RuntimeValue::Decimal(d) => Some(d.to_string().as_bytes().to_vec()),
            RuntimeValue::List(_) => None,
        }
    }

    /// Get a variable's value (helper for tests)
    pub fn get_variable(&self, name: &str) -> Option<RuntimeValue> {
        self.variables.get(name)
    }

    /// Set a variable's value (helper for tests)
    pub fn set_variable(&mut self, name: String, value: RuntimeValue) {
        self.variables.set(name, value);
    }

    /// Execute a single instruction (helper method)
    pub fn execute_instruction(&mut self, instr: &cnf_compiler::ir::Instruction) -> Result<(), CnfError> {
        self.execute_instructions(std::slice::from_ref(instr))
    }

    /// Execute a list of IR instructions
    pub fn execute_instructions(
        &mut self,
        instructions: &[cnf_compiler::ir::Instruction],
    ) -> Result<(), CnfError> {
        use cnf_compiler::ir::Instruction;

        for instr in instructions {
            match instr {
                Instruction::SetProfile { profile, memory_mb, parallelism } => {
                    self.active_profile = Some((profile.clone(), *memory_mb, *parallelism));
                    self.execution_trace.push(format!(
                        "PROFILE SET: profile={}, memory_mb={:?}, parallelism={:?}",
                        profile, memory_mb, parallelism
                    ));
                }
                // === PHASE 3: ASSIGNMENT & ARITHMETIC ===
                Instruction::Set { target, value } => {
                    self.dispatch_set(target, value)?;
                }
                Instruction::Add {
                    target,
                    operand1,
                    operand2,
                } => {
                    self.dispatch_add(target, operand1, operand2)?;
                }
                Instruction::Subtract {
                    target,
                    operand1,
                    operand2,
                } => {
                    self.dispatch_subtract(target, operand1, operand2)?;
                }
                Instruction::Multiply {
                    target,
                    operand1,
                    operand2,
                } => {
                    self.dispatch_multiply(target, operand1, operand2)?;
                }
                Instruction::Divide {
                    target,
                    operand1,
                    operand2,
                } => {
                    self.dispatch_divide(target, operand1, operand2)?;
                }

                // === PHASE 4+: OTHER INSTRUCTIONS ===
                Instruction::CompressCsm { source, target } => {
                    self.dispatch_compress_csm(source, target)?;
                }
                Instruction::DecompressCsm { source, target } => {
                    self.dispatch_decompress_csm(source, target)?;
                }
                Instruction::Compress { target } => {
                    self.dispatch_compress(target)?;
                }
                Instruction::Filter { target, condition } => {
                    self.dispatch_filter(target, condition)?;
                }
                Instruction::Aggregate { targets, operation } => {
                    self.dispatch_aggregate(targets, operation)?;
                }
                Instruction::Convert { target, output_type } => {
                    self.dispatch_convert(target, output_type)?;
                }
                Instruction::VerifyIntegrity { target } => {
                    match self.variables.get(target) {
                        Some(RuntimeValue::Binary(data)) => {
                            let hash = cnf_security::sha256_hex(&data);
                            let key = format!("__integrity__{}", target);
                            // Jika hash sebelumnya sudah tersimpan, bandingkan
                            if let Some(RuntimeValue::Text(prev_hash)) = self.variables.get(&key) {
                                if prev_hash != hash {
                                    return Err(CnfError::IntegrityViolation(
                                        format!("Integrity check failed for '{}': hash mismatch", target)
                                    ));
                                }
                            }
                            // Simpan hash untuk referensi berikutnya
                            self.variables.set(key, RuntimeValue::Text(hash.clone()));
                            self.execution_trace.push(format!("VERIFY-INTEGRITY {} sha256={}", target, &hash[..16]));
                            self.audit_log.push(format!("INTEGRITY_VERIFIED: {} hash={}", target, hash));
                        }
                        Some(_) => return Err(CnfError::RuntimeError(
                            format!("VERIFY-INTEGRITY: '{}' is not binary data", target)
                        )),
                        None => return Err(CnfError::BufferNotFound(target.to_string())),
                    }
                }
                Instruction::Merge { targets, output_name } => {
                    self.dispatch_merge(targets, output_name)?;
                }
                Instruction::Split { target, parts } => {
                    self.dispatch_split(target, parts)?;
                }
                Instruction::Validate { target, schema } => {
                    self.dispatch_validate(target, schema)?;
                }
                Instruction::Extract { target, path } => {
                    self.dispatch_extract(target, path)?;
                }
                Instruction::Display { message } => {
                    println!("{}", message);
                }
                Instruction::Read { target } => {
                    // Placeholder: read from stdin
                    use std::io::{self, BufRead};
                    let stdin = io::stdin();
                    let mut line = String::new();
                    stdin.lock().read_line(&mut line).ok();
                    self.variables
                        .set(target.clone(), RuntimeValue::Text(line.trim().to_string()));
                }
                Instruction::Print { target, format: _ } => {
                    if let Some(val) = self.variables.get(target) {
                        println!("{}", val);
                    }
                }

                // === PHASE 4: STRING OPERATIONS ===
                Instruction::Concatenate { target, operands } => {
                    self.dispatch_concatenate(target, operands)?;
                }
                Instruction::Substring {
                    target,
                    source,
                    start,
                    length,
                } => {
                    self.dispatch_substring(target, source, start, length)?;
                }
                Instruction::Length { target, source } => {
                    self.dispatch_length(target, source)?;
                }
                Instruction::Uppercase { target, source } => {
                    self.dispatch_uppercase(target, source)?;
                }
                Instruction::Lowercase { target, source } => {
                    self.dispatch_lowercase(target, source)?;
                }
                Instruction::Trim { target, source } => {
                    self.dispatch_trim(target, source)?;
                }

                // === CONTROL FLOW ===
                Instruction::IfStatement {
                    condition,
                    then_instrs,
                    else_instrs,
                } => {
                    self.dispatch_if(condition, then_instrs, else_instrs)?;
                }
                Instruction::ForLoop { variable, in_list, instrs } => {
                    self.dispatch_for(variable, in_list, instrs)?;
                }
                Instruction::WhileLoop { condition, instrs } => {
                    self.dispatch_while(condition, instrs)?;
                }

                // === QUANTUM CRYPTOGRAPHY ===
                #[cfg(feature = "quantum")]
                Instruction::QuantumEncrypt { source, key_name } => {
                    self.dispatch_quantum_encrypt(source, key_name)?;
                }
                #[cfg(feature = "quantum")]
                Instruction::QuantumDecrypt { target, key_name } => {
                    self.dispatch_quantum_decrypt(target, key_name)?;
                }
                #[cfg(feature = "quantum")]
                Instruction::QuantumSign { source, signing_key, output } => {
                    self.dispatch_quantum_sign(source, signing_key, output)?;
                }
                #[cfg(feature = "quantum")]
                Instruction::QuantumVerifySig { source, verification_key, signature_ref } => {
                    self.dispatch_quantum_verify_sig(source, verification_key, signature_ref)?;
                }
                #[cfg(feature = "quantum")]
                Instruction::QuantumSignEncrypt { source, recipient_key, signing_key, output } => {
                    self.dispatch_quantum_sign_encrypt(source, recipient_key, signing_key, output)?;
                }
                #[cfg(feature = "quantum")]
                Instruction::QuantumVerifyDecrypt { source, recipient_key, output } => {
                    self.dispatch_quantum_verify_decrypt(source, recipient_key, output)?;
                }
                #[cfg(feature = "quantum")]
                Instruction::GenerateKeyPair { algorithm, output_name } => {
                    self.dispatch_generate_keypair(algorithm, output_name)?;
                }
                #[cfg(feature = "quantum")]
                Instruction::LongTermSign { source, signing_key, output } => {
                    self.dispatch_long_term_sign(source, signing_key, output)?;
                }

                // === GOVERNANCE ===
                Instruction::Policy { name, formula } => {
                    self.dispatch_policy(name, formula)?;
                }
                Instruction::Regulation { standard, clause } => {
                    self.dispatch_regulation(standard, clause)?;
                }
                #[cfg(feature = "governance")]
                Instruction::DataSovereignty { from, to } => {
                    self.dispatch_data_sovereignty(from, to)?;
                }
                Instruction::AccessControl { user, resource, action } => {
                    self.dispatch_access_control(user, resource, action)?;
                }
                Instruction::AuditLedger { message } => {
                    self.dispatch_audit_ledger(message)?;
                }
                #[cfg(feature = "governance")]
                Instruction::DecisionQuorum { votes, threshold } => {
                    self.dispatch_decision_quorum(votes, threshold)?;
                }

                // === VERIFIER HOOKS ===
                #[cfg(feature = "verifier")]
                Instruction::PreConditionCheck { condition } => {
                    // Log precondition check — full Z3 verification in cnf-verifier
                    self.execution_trace.push(format!("PRE_CHECK: {}", condition));
                    self.audit_log.push(format!("[VERIFY] Precondition checked: {}", condition));
                }
                #[cfg(feature = "verifier")]
                Instruction::PostConditionCheck { condition } => {
                    self.execution_trace.push(format!("POST_CHECK: {}", condition));
                    self.audit_log.push(format!("[VERIFY] Postcondition checked: {}", condition));
                }
                #[cfg(feature = "verifier")]
                Instruction::InvariantCheck { condition } => {
                    self.execution_trace.push(format!("INVARIANT: {}", condition));
                    self.audit_log.push(format!("[VERIFY] Invariant checked: {}", condition));
                }

                // Stub implementations for unimplemented instructions
                _ => {
                    return Err(CnfError::InvalidInstruction(format!(
                        "Instruction not yet implemented: {:?}",
                        instr
                    )));
                }
            }
        }

        Ok(())
    }

    // ============ PHASE 3: DISPATCH METHODS ============

    /// Helper sets a value in the variable store
    fn set_value(&mut self, name: &str, value: RuntimeValue) {
        self.variables.set(name.to_string(), value);
    }

    /// SET target value
    /// Sets a variable to a literal value or reference to another variable
    fn dispatch_set(&mut self, target: &str, value_expr: &str) -> Result<(), CnfError> {
        // Try to parse as integer
        if let Ok(n) = value_expr.parse::<i64>() {
            self.set_value(target, RuntimeValue::Integer(n));
            return Ok(());
        }

        // Try to parse as float
        if let Ok(d) = value_expr.parse::<f64>() {
            self.set_value(target, RuntimeValue::Decimal(d));
            return Ok(());
        }

        // Otherwise treat as variable reference
        if let Some(val) = self.variables.get(value_expr) {
            self.set_value(target, val);
            return Ok(());
        }

        // Default: treat as text
        self.set_value(
            target,
            RuntimeValue::Text(value_expr.to_string()),
        );
        Ok(())
    }

    /// ADD operand1 + operand2 → target
    /// Performs numeric addition with type coercion
    /// Prefers the "wider" type (decimal > integer)
    fn dispatch_add(
        &mut self,
        target: &str,
        op1_name: &str,
        op2_name: &str,
    ) -> Result<(), CnfError> {
        let op1 = self.resolve_operand(op1_name)?;
        let op2 = self.resolve_operand(op2_name)?;

        // If either operand is Decimal, perform decimal arithmetic
        if matches!(op1, RuntimeValue::Decimal(_)) || matches!(op2, RuntimeValue::Decimal(_)) {
            let a = op1.as_decimal()?;
            let b = op2.as_decimal()?;
            self.set_value(target, RuntimeValue::Decimal(a + b));
            return Ok(());
        }

        // Otherwise both are integers
        let a = op1.as_integer()?;
        let b = op2.as_integer()?;
        self.set_value(target, RuntimeValue::Integer(a + b));
        Ok(())
    }

    /// SUBTRACT operand1 - operand2 → target
    fn dispatch_subtract(
        &mut self,
        target: &str,
        op1_name: &str,
        op2_name: &str,
    ) -> Result<(), CnfError> {
        let op1 = self.resolve_operand(op1_name)?;
        let op2 = self.resolve_operand(op2_name)?;

        // If either operand is Decimal, perform decimal arithmetic
        if matches!(op1, RuntimeValue::Decimal(_)) || matches!(op2, RuntimeValue::Decimal(_)) {
            let a = op1.as_decimal()?;
            let b = op2.as_decimal()?;
            self.set_value(target, RuntimeValue::Decimal(a - b));
            return Ok(());
        }

        // Otherwise both are integers
        let a = op1.as_integer()?;
        let b = op2.as_integer()?;
        self.set_value(target, RuntimeValue::Integer(a - b));
        Ok(())
    }

    /// MULTIPLY operand1 * operand2 → target
    fn dispatch_multiply(
        &mut self,
        target: &str,
        op1_name: &str,
        op2_name: &str,
    ) -> Result<(), CnfError> {
        let op1 = self.resolve_operand(op1_name)?;
        let op2 = self.resolve_operand(op2_name)?;

        // If either operand is Decimal, perform decimal arithmetic
        if matches!(op1, RuntimeValue::Decimal(_)) || matches!(op2, RuntimeValue::Decimal(_)) {
            let a = op1.as_decimal()?;
            let b = op2.as_decimal()?;
            self.set_value(target, RuntimeValue::Decimal(a * b));
            return Ok(());
        }

        // Otherwise both are integers
        let a = op1.as_integer()?;
        let b = op2.as_integer()?;
        self.set_value(target, RuntimeValue::Integer(a * b));
        Ok(())
    }

    /// DIVIDE operand1 / operand2 → target (fail-fast on division by zero)
    fn dispatch_divide(
        &mut self,
        target: &str,
        op1_name: &str,
        op2_name: &str,
    ) -> Result<(), CnfError> {
        let op1 = self.resolve_operand(op1_name)?;
        let op2 = self.resolve_operand(op2_name)?;

        // If either operand is Decimal, perform decimal arithmetic
        if matches!(op1, RuntimeValue::Decimal(_)) || matches!(op2, RuntimeValue::Decimal(_)) {
            let a = op1.as_decimal()?;
            let b = op2.as_decimal()?;
            if b == 0.0 {
                return Err(CnfError::RuntimeError("Division by zero".to_string()));
            }
            self.set_value(target, RuntimeValue::Decimal(a / b));
            return Ok(());
        }

        // Otherwise both are integers
        let a = op1.as_integer()?;
        let b = op2.as_integer()?;
        if b == 0 {
            return Err(CnfError::RuntimeError("Division by zero".to_string()));
        }
        self.set_value(target, RuntimeValue::Integer(a / b));
        Ok(())
    }

    // ============ PHASE 4: STRING DISPATCH METHODS ============

    /// CONCATENATE operands (converted to text) → target
    fn dispatch_concatenate(&mut self, target: &str, operands: &[String]) -> Result<(), CnfError> {
        let mut result = String::new();
        for op in operands {
            let val = self.resolve_operand(op)?;
            result.push_str(&format!("{}", val));
        }
        self.variables
            .set(target.to_string(), RuntimeValue::Text(result));
        Ok(())
    }

    /// SUBSTRING source[start : start+length] (byte-based)
    fn dispatch_substring(
        &mut self,
        target: &str,
        source: &str,
        start_expr: &str,
        length_expr: &str,
    ) -> Result<(), CnfError> {
        let src_val = self.resolve_operand(source)?;
        let src_str = src_val.to_string();
        let start_idx: usize = start_expr
            .parse()
            .map_err(|_| CnfError::InvalidInstruction(start_expr.to_string()))?;
        let len: usize = length_expr
            .parse()
            .map_err(|_| CnfError::InvalidInstruction(length_expr.to_string()))?;
        let substring = if start_idx < src_str.len() {
            let end = (start_idx + len).min(src_str.len());
            src_str[start_idx..end].to_string()
        } else {
            String::new()
        };
        self.set_value(target, RuntimeValue::Text(substring));
        Ok(())
    }

    /// LENGTH of text representation → Integer
    fn dispatch_length(&mut self, target: &str, source: &str) -> Result<(), CnfError> {
        let src_val = self.resolve_operand(source)?;
        let len = src_val.to_string().len() as i64;
        self.set_value(target, RuntimeValue::Integer(len));
        Ok(())
    }

    /// Convert source to uppercase text
    fn dispatch_uppercase(&mut self, target: &str, source: &str) -> Result<(), CnfError> {
        let src_val = self.resolve_operand(source)?;
        let src_str = src_val.to_string();
        let result = cnf_stdlib::string::to_upper(&src_str);
        self.set_value(target, RuntimeValue::Text(result));
        Ok(())
    }

    /// Convert source to lowercase text
    fn dispatch_lowercase(&mut self, target: &str, source: &str) -> Result<(), CnfError> {
        let src_val = self.resolve_operand(source)?;
        let src_str = src_val.to_string();
        let result = cnf_stdlib::string::to_lower(&src_str);
        self.set_value(target, RuntimeValue::Text(result));
        Ok(())
    }

    /// Trim whitespace from text
    fn dispatch_trim(&mut self, target: &str, source: &str) -> Result<(), CnfError> {
        let src_val = self.resolve_operand(source)?;
        let src_str = src_val.to_string();
        let result = cnf_stdlib::string::trim(&src_str).to_string();
        self.set_value(target, RuntimeValue::Text(result));
        Ok(())
    }

    fn dispatch_filter(&mut self, target: &str, condition: &str) -> Result<(), CnfError> {
        // Basic filter: if the variable exists and is Text, split into lines and
        // retain only those containing the condition substring.
        if let Some(RuntimeValue::Text(text)) = self.variables.get(target) {
            let filtered: String = text
                .lines()
                .filter(|line| line.contains(condition))
                .collect::<Vec<_>>()
                .join("\n");
            self.set_value(target, RuntimeValue::Text(filtered));
            self.execution_trace.push(format!(
                "FILTER {} where contains '{}'",
                target, condition
            ));
        }
        Ok(())
    }

    fn dispatch_aggregate(&mut self, targets: &[String], operation: &str) -> Result<(), CnfError> {
        let result = match operation {
            "SUM" => {
                let mut sum: i64 = 0;
                for t in targets {
                    if let Some(val) = self.variables.get(t) {
                        if let Ok(i) = val.as_integer() {
                            sum += i;
                        }
                    }
                }
                RuntimeValue::Integer(sum)
            }
            "COUNT" => RuntimeValue::Integer(targets.len() as i64),
            _ => return Err(CnfError::RuntimeError(format!("Unknown aggregate: {}", operation))),
        };
        self.set_value("__aggregate_result", result);
        self.execution_trace
            .push(format!("AGGREGATE {} on {:?}", operation, targets));
        Ok(())
    }

    fn dispatch_convert(&mut self, target: &str, _output_type: &str) -> Result<(), CnfError> {
        // Placeholder: just log the operation
        self.execution_trace.push(format!("CONVERT {} to {}", target, _output_type));
        Ok(())
    }

    fn dispatch_merge(&mut self, _targets: &[String], _output_name: &str) -> Result<(), CnfError> {
        // Placeholder: just log the operation
        self.execution_trace.push("MERGE".to_string());
        Ok(())
    }

    fn dispatch_split(&mut self, _target: &str, _parts: &str) -> Result<(), CnfError> {
        // Placeholder: just log the operation
        self.execution_trace.push("SPLIT".to_string());
        Ok(())
    }

    fn dispatch_validate(&mut self, _target: &str, _schema: &str) -> Result<(), CnfError> {
        // Placeholder: just log the operation
        self.execution_trace.push("VALIDATE".to_string());
        Ok(())
    }

    fn dispatch_extract(&mut self, _target: &str, _path: &str) -> Result<(), CnfError> {
        // Placeholder: just log the operation
        self.execution_trace.push("EXTRACT".to_string());
        Ok(())
    }

    fn dispatch_compress_csm(&mut self, source: &str, target: &str) -> Result<(), CnfError> {
        let dict = self.csm_dict.as_ref().ok_or_else(|| {
            CnfError::CsmError("CSM dictionary not loaded. Call runtime.csm_dict = Some(dict) before COMPRESS-CSM".to_string())
        })?;
        let data = match self.variables.get(source) {
            Some(RuntimeValue::Binary(b)) => b.clone(),
            Some(RuntimeValue::Text(t)) => t.as_bytes().to_vec(),
            Some(_) => return Err(CnfError::CsmError(format!("Variable '{}' is not binary or text", source))),
            None => return Err(CnfError::BufferNotFound(source.to_string())),
        };
        let compressed = cobol_protocol_v154::compress_csm(&data, dict)
            .map_err(|e| CnfError::CsmError(format!("CSM compression failed: {}", e)))?;
        self.variables.set(target.to_string(), RuntimeValue::Binary(compressed));
        self.audit_log.push(format!("COMPRESS-CSM: {} -> {} ({} bytes -> compressed)", source, target, data.len()));
        Ok(())
    }

    fn dispatch_decompress_csm(&mut self, source: &str, target: &str) -> Result<(), CnfError> {
        let dict = self.csm_dict.as_ref().ok_or_else(|| {
            CnfError::CsmError("CSM dictionary not loaded".to_string())
        })?;
        let data = match self.variables.get(source) {
            Some(RuntimeValue::Binary(b)) => b.clone(),
            None => return Err(CnfError::BufferNotFound(source.to_string())),
            _ => return Err(CnfError::CsmError(format!("Variable '{}' is not binary", source))),
        };
        let decompressed = cobol_protocol_v154::decompress_csm(&data, dict)
            .map_err(|e| CnfError::CsmError(format!("CSM decompression failed: {}", e)))?;
        self.variables.set(target.to_string(), RuntimeValue::Binary(decompressed));
        Ok(())
    }

    // ============ CONTROL FLOW ============

    fn dispatch_if(
        &mut self,
        condition: &str,
        then_instrs: &[cnf_compiler::ir::Instruction],
        else_instrs: &Option<Vec<cnf_compiler::ir::Instruction>>,
    ) -> Result<(), CnfError> {
        let vars = self.variables.iter()
            .map(|(k, v)| (k.clone(), v.to_string()))
            .collect::<std::collections::HashMap<_, _>>();
        let evaluator = crate::control_flow::ConditionEvaluator::new(vars);
        let result = evaluator.evaluate(condition)
            .map_err(|e| CnfError::RuntimeError(format!("Condition evaluation failed: {}", e)))?;
        self.execution_trace.push(format!("IF({}={})", condition, result));
        if result {
            self.execute_instructions(then_instrs)?;
        } else if let Some(else_i) = else_instrs {
            self.execute_instructions(else_i)?;
        }
        Ok(())
    }

    fn dispatch_while(
        &mut self,
        condition: &str,
        instrs: &[cnf_compiler::ir::Instruction],
    ) -> Result<(), CnfError> {
        const MAX_ITERATIONS: usize = 10_000;
        let mut count = 0usize;
        loop {
            if count >= MAX_ITERATIONS {
                return Err(CnfError::LoopLimitExceeded(MAX_ITERATIONS));
            }
            let vars = self.variables.iter()
                .map(|(k, v)| (k.clone(), v.to_string()))
                .collect::<std::collections::HashMap<_, _>>();
            let evaluator = crate::control_flow::ConditionEvaluator::new(vars);
            let cond = evaluator.evaluate(condition)
                .map_err(|e| CnfError::RuntimeError(format!("While condition error: {}", e)))?;
            if !cond { break; }
            self.execute_instructions(instrs)?;
            count += 1;
        }
        Ok(())
    }

    fn dispatch_for(
        &mut self,
        variable: &str,
        in_list: &str,
        instrs: &[cnf_compiler::ir::Instruction],
    ) -> Result<(), CnfError> {
        let items: Vec<String> = in_list.split(',').map(|s| s.trim().to_string()).collect();
        for item in items {
            self.variables.set(variable.to_string(), RuntimeValue::Text(item.clone()));
            self.execution_trace.push(format!("FOR {}={}", variable, item));
            self.execute_instructions(instrs)?;
        }
        Ok(())
    }

    // ============ GOVERNANCE DISPATCH METHODS ============

    pub fn dispatch_governance_begin(&mut self) -> Result<(), CnfError> {
        self.execution_trace.push("GOVERNANCE_BEGIN".to_string());
        Ok(())
    }

    pub fn dispatch_governance_policy(&mut self, name: &str) -> Result<(), CnfError> {
        self.dispatch_policy(name, "true") // dummy formula
    }

    pub fn dispatch_governance_regulation(&mut self, standard: &str) -> Result<(), CnfError> {
        self.dispatch_regulation(standard, "compliant")
    }

    #[cfg(feature = "governance")]
    pub fn dispatch_governance_data_sovereignty(&mut self, from: &str, to: &str) -> Result<(), CnfError> {
        self.dispatch_data_sovereignty(from, to)
    }

    pub fn dispatch_governance_access_control(&mut self, user: &str, resource: &str) -> Result<(), CnfError> {
        self.dispatch_access_control(user, resource, "access")
    }

    pub fn dispatch_governance_audit_ledger(&mut self, message: &str) -> Result<(), CnfError> {
        self.dispatch_audit_ledger(message)
    }

    pub fn dispatch_governance_end(&mut self) -> Result<(), CnfError> {
        self.execution_trace.push("GOVERNANCE_END".to_string());
        Ok(())
    }

    pub fn dispatch_compress(&mut self, target: &str) -> Result<(), CnfError> {
        // Simple compression: prepend "COMPRESSED:" to the buffer
        if let Some(RuntimeValue::Binary(buf)) = self.variables.get(target) {
            let mut compressed = b"COMPRESSED:".to_vec();
            compressed.extend_from_slice(&buf);
            self.variables.set(target.to_string(), RuntimeValue::Binary(compressed));
        }
        self.execution_trace.push(format!("COMPRESS {}", target));
        Ok(())
    }

    fn dispatch_policy(&mut self, name: &str, formula: &str) -> Result<(), CnfError> {
        self.governance_policies.insert(name.to_string(), formula.to_string());
        self.execution_trace.push(format!("POLICY {}: {}", name, formula));
        Ok(())
    }

    fn dispatch_regulation(&mut self, standard: &str, clause: &str) -> Result<(), CnfError> {
        // For now, just log it. In full implementation, would validate against regulatory standards
        self.execution_trace.push(format!("REGULATION {}: {}", standard, clause));
        Ok(())
    }

    #[cfg(feature = "governance")]
    fn dispatch_data_sovereignty(&mut self, from: &str, to: &str) -> Result<(), CnfError> {
        // Check data sovereignty rules
        use cnf_governance::data_sovereignty::{SovereigntyChecker, Region};
        let from_region = match from {
            "EU" => Region::EU,
            "US" => Region::US,
            "APAC" => Region::APAC,
            _ => Region::OTHER(from.to_string()),
        };
        let to_region = match to {
            "EU" => Region::EU,
            "US" => Region::US,
            "APAC" => Region::APAC,
            _ => Region::OTHER(to.to_string()),
        };
        let checker = SovereigntyChecker::new();
        checker.validate_transfer(&from_region, &to_region)
            .map_err(|e| CnfError::GovernancePolicyViolation(format!("Data sovereignty violation: {:?}", e)))?;
        self.execution_trace.push(format!("DATA_SOVEREIGNTY {} -> {}", from, to));
        Ok(())
    }

    fn dispatch_access_control(&mut self, user: &str, resource: &str, action: &str) -> Result<(), CnfError> {
        // Store access rule
        self.access_rules.insert((user.to_string(), resource.to_string()), vec![action.to_string()]);
        self.execution_trace.push(format!("ACCESS_CONTROL {} {} {}", user, resource, action));
        Ok(())
    }

    fn dispatch_audit_ledger(&mut self, message: &str) -> Result<(), CnfError> {
        self.audit_log.push(message.to_string());
        self.execution_trace.push(format!("AUDIT: {}", message));
        Ok(())
    }

    #[cfg(feature = "governance")]
    fn dispatch_decision_quorum(&mut self, votes: &str, threshold: &str) -> Result<(), CnfError> {
        use cnf_governance::distributed_rules::ConsensusQuorum;
        let vote_count: usize = votes.parse()
            .map_err(|_| CnfError::RuntimeError("Invalid vote count".to_string()))?;
        let node_count: usize = threshold.parse()
            .map_err(|_| CnfError::RuntimeError("Invalid node count".to_string()))?;
        let quorum = ConsensusQuorum::new(node_count);
        quorum.check(vote_count)
            .map_err(|e| CnfError::GovernancePolicyViolation(e.to_string()))?;
        self.execution_trace.push(
            format!("DECISION_QUORUM: {}/{} votes (required: {})", vote_count, node_count, quorum.agreement)
        );
        Ok(())
    }

    // ============ QUANTUM CRYPTOGRAPHY DISPATCH METHODS ============

    #[cfg(feature = "quantum")]
    fn dispatch_quantum_encrypt(&mut self, source: &str, key_name: &str) -> Result<(), CnfError> {
        let data = self.resolve_operand(source)?.as_binary()?;
        // Assume key is stored as binary in variables
        let key_data = self.resolve_operand(key_name)?.as_binary()?;
        // Deserialize key (simplified - in real impl, use proper deserialization)
        let key: cnf_quantum::KyberKeyPair = serde_json::from_slice(&key_data)
            .map_err(|e| CnfError::RuntimeError(format!("Invalid quantum key: {}", e)))?;
        let encrypted = cnf_quantum::quantum_encrypt(&data, &key.public_key)
            .map_err(|e| CnfError::EncryptionFailed(format!("Quantum encryption failed: {:?}", e)))?;
        let encrypted_bytes = serde_json::to_vec(&encrypted)
            .map_err(|e| CnfError::RuntimeError(format!("Serialization failed: {}", e)))?;
        self.set_value(source, RuntimeValue::Binary(encrypted_bytes));
        self.execution_trace.push(format!("QUANTUM_ENCRYPT {} with {}", source, key_name));
        Ok(())
    }

    #[cfg(feature = "quantum")]
    fn dispatch_quantum_decrypt(&mut self, target: &str, key_name: &str) -> Result<(), CnfError> {
        let encrypted_data = self.resolve_operand(target)?.as_binary()?;
        let key_data = self.resolve_operand(key_name)?.as_binary()?;
        let key: cnf_quantum::KyberKeyPair = serde_json::from_slice(&key_data)
            .map_err(|e| CnfError::RuntimeError(format!("Invalid quantum key: {}", e)))?;
        let encrypted: cnf_quantum::QuantumEncryptedBlob = serde_json::from_slice(&encrypted_data)
            .map_err(|e| CnfError::RuntimeError(format!("Invalid encrypted data: {}", e)))?;
        let decrypted = cnf_quantum::quantum_decrypt(&encrypted, &key.secret_key)
            .map_err(|e| CnfError::DecryptionFailed(format!("Quantum decryption failed: {:?}", e)))?;
        self.set_value(target, RuntimeValue::Binary(decrypted));
        self.execution_trace.push(format!("QUANTUM_DECRYPT {} with {}", target, key_name));
        Ok(())
    }

    #[cfg(feature = "quantum")]
    fn dispatch_quantum_sign(&mut self, source: &str, signing_key: &str, output: &str) -> Result<(), CnfError> {
        let data = self.resolve_operand(source)?.as_binary()?;
        let key_data = self.resolve_operand(signing_key)?.as_binary()?;
        let key: cnf_quantum::DilithiumKeyPair = serde_json::from_slice(&key_data)
            .map_err(|e| CnfError::RuntimeError(format!("Invalid signing key: {}", e)))?;
        let signature = cnf_quantum::dilithium_sign(&data, &key.secret_key)
            .map_err(|e| CnfError::RuntimeError(format!("Quantum signing failed: {:?}", e)))?;
        let sig_bytes = serde_json::to_vec(&signature)
            .map_err(|e| CnfError::RuntimeError(format!("Serialization failed: {}", e)))?;
        self.set_value(output, RuntimeValue::Binary(sig_bytes));
        self.execution_trace.push(format!("QUANTUM_SIGN {} with {} -> {}", source, signing_key, output));
        Ok(())
    }

    #[cfg(feature = "quantum")]
    fn dispatch_quantum_verify_sig(&mut self, source: &str, verification_key: &str, signature_ref: &str) -> Result<(), CnfError> {
        let data = self.resolve_operand(source)?.as_binary()?;
        let key_data = self.resolve_operand(verification_key)?.as_binary()?;
        let sig_data = self.resolve_operand(signature_ref)?.as_binary()?;
        let key: cnf_quantum::DilithiumKeyPair = serde_json::from_slice(&key_data)
            .map_err(|e| CnfError::RuntimeError(format!("Invalid verification key: {}", e)))?;
        let signature: cnf_quantum::DilithiumSignature = serde_json::from_slice(&sig_data)
            .map_err(|e| CnfError::RuntimeError(format!("Invalid signature: {}", e)))?;
        cnf_quantum::dilithium_verify(&data, &signature, &key.public_key)
            .map_err(|e| CnfError::RuntimeError(format!("Quantum verification failed: {:?}", e)))?;
        self.execution_trace.push(format!("QUANTUM_VERIFY_SIG {} with {} sig {}", source, verification_key, signature_ref));
        Ok(())
    }

    #[cfg(feature = "quantum")]
    fn dispatch_quantum_sign_encrypt(&mut self, source: &str, recipient_key: &str, signing_key: &str, output: &str) -> Result<(), CnfError> {
        let data = self.resolve_operand(source)?.as_binary()?;
        let rec_key_data = self.resolve_operand(recipient_key)?.as_binary()?;
        let sig_key_data = self.resolve_operand(signing_key)?.as_binary()?;
        let rec_key: cnf_quantum::KyberKeyPair = serde_json::from_slice(&rec_key_data)
            .map_err(|e| CnfError::RuntimeError(format!("Invalid recipient key: {}", e)))?;
        let sig_key: cnf_quantum::DilithiumKeyPair = serde_json::from_slice(&sig_key_data)
            .map_err(|e| CnfError::RuntimeError(format!("Invalid signing key: {}", e)))?;
        let signed_encrypted = cnf_quantum::quantum_sign_and_encrypt(&data, &rec_key.public_key, &sig_key.secret_key)
            .map_err(|e| CnfError::RuntimeError(format!("Quantum sign-encrypt failed: {:?}", e)))?;
        let se_bytes = serde_json::to_vec(&signed_encrypted)
            .map_err(|e| CnfError::RuntimeError(format!("Serialization failed: {}", e)))?;
        self.set_value(output, RuntimeValue::Binary(se_bytes));
        self.execution_trace.push(format!("QUANTUM_SIGN_ENCRYPT {} to {} with {} -> {}", source, recipient_key, signing_key, output));
        Ok(())
    }

    #[cfg(feature = "quantum")]
    fn dispatch_quantum_verify_decrypt(&mut self, source: &str, recipient_key: &str, output: &str) -> Result<(), CnfError> {
        let se_data = self.resolve_operand(source)?.as_binary()?;
        let rec_key_data = self.resolve_operand(recipient_key)?.as_binary()?;
        let rec_key: cnf_quantum::KyberKeyPair = serde_json::from_slice(&rec_key_data)
            .map_err(|e| CnfError::RuntimeError(format!("Invalid recipient key: {}", e)))?;
        let signed_encrypted: cnf_quantum::SignedEncryptedBlob = serde_json::from_slice(&se_data)
            .map_err(|e| CnfError::RuntimeError(format!("Invalid signed-encrypted data: {}", e)))?;
        let (decrypted, verified) = cnf_quantum::quantum_verify_and_decrypt(&signed_encrypted, &rec_key.secret_key)
            .map_err(|e| CnfError::RuntimeError(format!("Quantum verify-decrypt failed: {:?}", e)))?;
        if !verified {
            return Err(CnfError::RuntimeError("Signature verification failed".to_string()));
        }
        self.set_value(output, RuntimeValue::Binary(decrypted));
        self.execution_trace.push(format!("QUANTUM_VERIFY_DECRYPT {} with {} -> {}", source, recipient_key, output));
        Ok(())
    }

    #[cfg(feature = "quantum")]
    #[cfg(feature = "quantum")]
    fn dispatch_generate_keypair(&mut self, algorithm: &str, output_name: &str) -> Result<(), CnfError> {
        let keypair = match algorithm {
            "ML-KEM-768" | "KYBER" => {
                let kp = cnf_quantum::generate_kyber_keypair()
                    .map_err(|e| CnfError::RuntimeError(e.to_string()))?;
                (kp.public_key.clone(), kp.secret_key.clone())
            }
            "ML-DSA-65" | "DILITHIUM" => {
                let kp = cnf_quantum::generate_dilithium_keypair()
                    .map_err(|e| CnfError::RuntimeError(e.to_string()))?;
                (kp.verification_key.clone(), kp.signing_key.clone())
            }
            other => return Err(CnfError::RuntimeError(format!("Unknown algorithm: {}", other))),
        };
        self.quantum_keys.insert(output_name.to_string(), keypair);
        self.audit_log.push(format!("GENERATE_KEYPAIR: {} algo={}", output_name, algorithm));
        Ok(())
    }

    #[cfg(feature = "quantum")]
    fn dispatch_long_term_sign(&mut self, source: &str, signing_key: &str, output: &str) -> Result<(), CnfError> {
        let data = self.resolve_operand(source)?.as_binary()?;
        let key_data = self.resolve_operand(signing_key)?.as_binary()?;
        let key: cnf_quantum::SphincsKeyPair = serde_json::from_slice(&key_data)
            .map_err(|e| CnfError::RuntimeError(format!("Invalid long-term signing key: {}", e)))?;
        let signature = cnf_quantum::sphincs_sign(&data, &key.secret_key)
            .map_err(|e| CnfError::RuntimeError(format!("Long-term signing failed: {:?}", e)))?;
        let sig_bytes = serde_json::to_vec(&signature)
            .map_err(|e| CnfError::RuntimeError(format!("Serialization failed: {}", e)))?;
        self.set_value(output, RuntimeValue::Binary(sig_bytes));
        self.execution_trace.push(format!("LONG_TERM_SIGN {} with {} -> {}", source, signing_key, output));
        Ok(())
    }

    // ============ GOVERNANCE LTL VERIFICATION ============

    #[cfg(feature = "governance")]
    pub fn verify_policy(&self, policy_name: &str) -> Result<bool, CnfError> {
        use cnf_governance::policy_engine::LtlFormula;
        let formula_str = self.governance_policies.get(policy_name)
            .ok_or_else(|| CnfError::RuntimeError(format!("Policy '{}' not found", policy_name)))?;
        // Parse simple atoms only — full LTL parser is future work
        let formula = LtlFormula::Atom(formula_str.clone());
        let result = self.governance.verify(&formula, &self.governance_trace)
            .map_err(|e| CnfError::GovernancePolicyViolation(e.to_string()))?;
        Ok(result)
    }

    pub fn execution_trace(&self) -> &[String] {
        &self.execution_trace
    }

    // ============ HELPER METHODS ============

    /// Resolve operand: variable ref or literal value
    fn resolve_operand(&self, operand: &str) -> Result<RuntimeValue, CnfError> {
        // Try to parse as integer literal
        if let Ok(n) = operand.parse::<i64>() {
            return Ok(RuntimeValue::Integer(n));
        }

        // Try to parse as float literal
        if let Ok(d) = operand.parse::<f64>() {
            return Ok(RuntimeValue::Decimal(d));
        }

        // Treat as variable reference
        self.variables
            .get(operand)
            .ok_or_else(|| {
                CnfError::BufferNotFound(format!("Variable not found: {}", operand))
            })
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_value_as_integer() {
        let val_int = RuntimeValue::Integer(42);
        assert_eq!(val_int.as_integer().unwrap(), 42);

        let val_dec = RuntimeValue::Decimal(42.5);
        assert_eq!(val_dec.as_integer().unwrap(), 42);
    }

    #[test]
    fn test_runtime_value_as_decimal() {
        let val_int = RuntimeValue::Integer(42);
        assert_eq!(val_int.as_decimal().unwrap(), 42.0);

        let val_dec = RuntimeValue::Decimal(42.5);
        assert_eq!(val_dec.as_decimal().unwrap(), 42.5);
    }

    #[test]
    fn test_variable_store_set_and_get() {
        let mut store = VariableStore::new();
        store.set("x".to_string(), RuntimeValue::Integer(10));

        assert_eq!(
            store.get("x"),
            Some(RuntimeValue::Integer(10))
        );
        assert_eq!(store.get("nonexistent"), None);
    }

    #[test]
    fn test_variable_store_get_or_default() {
        let store = VariableStore::new();
        // Nonexistent variable defaults to 0
        assert_eq!(
            store.get_or_default("missing"),
            RuntimeValue::Integer(0)
        );
    }

    #[test]
    fn test_dispatch_set_integer_literal() {
        let mut runtime = Runtime::new();
        runtime.dispatch_set("x", "42").unwrap();

        assert_eq!(
            runtime.variables.get("x"),
            Some(RuntimeValue::Integer(42))
        );
    }

    #[test]
    fn test_dispatch_set_decimal_literal() {
        let mut runtime = Runtime::new();
        runtime.dispatch_set("x", "3.14").unwrap();

        let val = runtime.variables.get("x").unwrap();
        match val {
            RuntimeValue::Decimal(d) => {
                assert!((d - 3.14).abs() < 0.001);
            }
            _ => panic!("Expected decimal"),
        }
    }

    #[test]
    fn test_dispatch_set_text_literal() {
        let mut runtime = Runtime::new();
        runtime.dispatch_set("msg", "hello").unwrap();

        assert_eq!(
            runtime.variables.get("msg"),
            Some(RuntimeValue::Text("hello".to_string()))
        );
    }

    #[test]
    fn test_dispatch_set_variable_reference() {
        let mut runtime = Runtime::new();
        runtime
            .variables
            .set("x".to_string(), RuntimeValue::Integer(42));

        runtime.dispatch_set("y", "x").unwrap();

        assert_eq!(
            runtime.variables.get("y"),
            Some(RuntimeValue::Integer(42))
        );
    }

    #[test]
    fn test_dispatch_add_integers() {
        let mut runtime = Runtime::new();
        runtime
            .variables
            .set("a".to_string(), RuntimeValue::Integer(10));
        runtime
            .variables
            .set("b".to_string(), RuntimeValue::Integer(5));

        runtime.dispatch_add("result", "a", "b").unwrap();

        assert_eq!(
            runtime.variables.get("result"),
            Some(RuntimeValue::Integer(15))
        );
    }

    #[test]
    fn test_dispatch_add_decimals() {
        let mut runtime = Runtime::new();
        // When loading decimals, treat first as trying integer,
        // which will fail, then fall back to decimal.
        // We directly use literals here to avoid variable resolution issues
        runtime
            .variables
            .set("dec_a".to_string(), RuntimeValue::Decimal(10.5));
        runtime
            .variables
            .set("dec_b".to_string(), RuntimeValue::Decimal(5.2));

        runtime.dispatch_add("result", "dec_a", "dec_b").unwrap();

        let val = runtime.variables.get("result").unwrap();
        match val {
            RuntimeValue::Decimal(d) => {
                assert!((d - 15.7).abs() < 0.001);
            }
            _ => panic!("Expected decimal, got {:?}", val),
        }
    }

    #[test]
    fn test_dispatch_add_integer_literals() {
        let mut runtime = Runtime::new();
        runtime.dispatch_add("result", "10", "5").unwrap();

        assert_eq!(
            runtime.variables.get("result"),
            Some(RuntimeValue::Integer(15))
        );
    }

    #[test]
    fn test_dispatch_subtract_integers() {
        let mut runtime = Runtime::new();
        runtime
            .variables
            .set("a".to_string(), RuntimeValue::Integer(10));
        runtime
            .variables
            .set("b".to_string(), RuntimeValue::Integer(3));

        runtime.dispatch_subtract("result", "a", "b").unwrap();

        assert_eq!(
            runtime.variables.get("result"),
            Some(RuntimeValue::Integer(7))
        );
    }

    #[test]
    fn test_dispatch_multiply_integers() {
        let mut runtime = Runtime::new();
        runtime
            .variables
            .set("a".to_string(), RuntimeValue::Integer(6));
        runtime
            .variables
            .set("b".to_string(), RuntimeValue::Integer(7));

        runtime.dispatch_multiply("result", "a", "b").unwrap();

        assert_eq!(
            runtime.variables.get("result"),
            Some(RuntimeValue::Integer(42))
        );
    }

    #[test]
    fn test_dispatch_divide_integers() {
        let mut runtime = Runtime::new();
        runtime
            .variables
            .set("a".to_string(), RuntimeValue::Integer(20));
        runtime
            .variables
            .set("b".to_string(), RuntimeValue::Integer(4));

        runtime.dispatch_divide("result", "a", "b").unwrap();

        assert_eq!(
            runtime.variables.get("result"),
            Some(RuntimeValue::Integer(5))
        );
    }

    #[test]
    fn test_dispatch_divide_by_zero_fails() {
        let mut runtime = Runtime::new();
        runtime
            .variables
            .set("a".to_string(), RuntimeValue::Integer(10));
        runtime
            .variables
            .set("b".to_string(), RuntimeValue::Integer(0));

        let result = runtime.dispatch_divide("result", "a", "b");
        assert!(result.is_err());

        match result {
            Err(CnfError::RuntimeError(msg)) => {
                assert!(msg.contains("Division by zero"));
            }
            _ => panic!("Expected RuntimeError for division by zero"),
        }
    }

    #[test]
    fn test_dispatch_filter_basic() {
        let mut runtime = Runtime::new();
        runtime.variables.set(
            "buf".to_string(),
            RuntimeValue::Text("foo\nbar\nbaz".to_string()),
        );
        runtime.dispatch_filter("buf", "ba").unwrap();
        assert_eq!(
            runtime.variables.get("buf"),
            Some(RuntimeValue::Text("bar\nbaz".to_string()))
        );
    }

    #[test]
    fn test_dispatch_aggregate_sum_and_count() {
        let mut runtime = Runtime::new();
        runtime
            .variables
            .set("a".to_string(), RuntimeValue::Integer(2));
        runtime
            .variables
            .set("b".to_string(), RuntimeValue::Integer(3));
        runtime
            .dispatch_aggregate(&vec!["a".to_string(), "b".to_string()], "SUM")
            .unwrap();
        assert_eq!(
            runtime.variables.get("__aggregate_result"),
            Some(RuntimeValue::Integer(5))
        );

        runtime
            .dispatch_aggregate(&vec!["a".to_string()], "COUNT")
            .unwrap();
        assert_eq!(
            runtime.variables.get("__aggregate_result"),
            Some(RuntimeValue::Integer(1))
        );
    }

    #[test]
    fn test_resolve_operand_integer_literal() {
        let runtime = Runtime::new();
        let val = runtime.resolve_operand("42").unwrap();
        assert_eq!(val, RuntimeValue::Integer(42));
    }

    #[test]
    fn test_resolve_operand_variable_reference() {
        let mut runtime = Runtime::new();
        runtime
            .variables
            .set("x".to_string(), RuntimeValue::Integer(100));

        let val = runtime.resolve_operand("x").unwrap();
        assert_eq!(val, RuntimeValue::Integer(100));
    }

    #[test]
    fn test_resolve_operand_missing_variable_fails() {
        let runtime = Runtime::new();
        let result = runtime.resolve_operand("missing");
        assert!(result.is_err());

        match result {
            Err(CnfError::BufferNotFound(msg)) => {
                assert!(msg.contains("Variable not found"));
            }
            _ => panic!("Expected BufferNotFound error"),
        }
    }

    #[test]
    fn test_add_buffer() {
        let mut runtime = Runtime::new();
        let data = vec![1, 2, 3, 4];
        runtime.add_buffer("TEST".to_string(), data.clone());

        assert_eq!(
            runtime.variables.get("TEST"),
            Some(RuntimeValue::Binary(data))
        );
    }

    #[test]
    fn test_list_buffers() {
        let mut runtime = Runtime::new();
        runtime.add_buffer("BUF1".to_string(), vec![1, 2, 3]);
        runtime.add_buffer("BUF2".to_string(), vec![4, 5, 6]);
        runtime
            .variables
            .set("VAR".to_string(), RuntimeValue::Integer(42));

        let buffers = runtime.list_buffers();
        assert_eq!(buffers.len(), 2);
        assert!(buffers.iter().any(|(name, _)| name == "BUF1"));
        assert!(buffers.iter().any(|(name, _)| name == "BUF2"));
    }

    // === string operation tests ===

    #[test]
    fn test_dispatch_concatenate_text() {
        let mut runtime = Runtime::new();
        runtime
            .variables
            .set("a".to_string(), RuntimeValue::Text("Hello".to_string()));
        runtime
            .variables
            .set("b".to_string(), RuntimeValue::Text("World".to_string()));

        runtime
            .dispatch_concatenate("out", &["a".to_string(), "b".to_string()])
            .unwrap();
        assert_eq!(
            runtime.variables.get("out"),
            Some(RuntimeValue::Text("HelloWorld".to_string()))
        );
    }

    #[test]
    fn test_dispatch_substring_basic() {
        let mut runtime = Runtime::new();
        runtime
            .variables
            .set("s".to_string(), RuntimeValue::Text("abcdef".to_string()));

        runtime
            .dispatch_substring("out", "s", "2", "3")
            .unwrap();
        assert_eq!(
            runtime.variables.get("out"),
            Some(RuntimeValue::Text("cde".to_string()))
        );

        // start past end returns empty
        runtime
            .dispatch_substring("out2", "s", "10", "4")
            .unwrap();
        assert_eq!(runtime.variables.get("out2"), Some(RuntimeValue::Text("".to_string())));
    }

    #[test]
    fn test_dispatch_length_integer() {
        let mut runtime = Runtime::new();
        runtime
            .variables
            .set("s".to_string(), RuntimeValue::Text("xyz".to_string()));
        runtime.dispatch_length("len", "s").unwrap();
        assert_eq!(runtime.variables.get("len"), Some(RuntimeValue::Integer(3)));

        // length of numeric value coerces to text
        runtime
            .variables
            .set("n".to_string(), RuntimeValue::Integer(12345));
        runtime.dispatch_length("len2", "n").unwrap();
        assert_eq!(runtime.variables.get("len2"), Some(RuntimeValue::Integer(5)));
    }

    #[test]
    fn test_dispatch_case_trim() {
        let mut runtime = Runtime::new();
        runtime
            .variables
            .set("x".to_string(), RuntimeValue::Text(" Foo Bar ".to_string()));
        runtime.dispatch_trim("trimmed", "x").unwrap();
        assert_eq!(
            runtime.variables.get("trimmed"),
            Some(RuntimeValue::Text("Foo Bar".to_string()))
        );

        runtime.dispatch_uppercase("up", "x").unwrap();
        assert_eq!(
            runtime.variables.get("up"),
            Some(RuntimeValue::Text(" FOO BAR ".to_string()))
        );

        runtime.dispatch_lowercase("low", "x").unwrap();
        assert_eq!(
            runtime.variables.get("low"),
            Some(RuntimeValue::Text(" foo bar ".to_string()))
        );
    }

    #[test]
    fn test_dispatch_if_for_while() {
        let mut runtime = Runtime::new();
        runtime
            .variables
            .set("NUM".to_string(), RuntimeValue::Text("0".to_string()));

        // IF statement
        let if_instr = cnf_compiler::ir::Instruction::IfStatement {
            condition: "true".to_string(),
            then_instrs: vec![cnf_compiler::ir::Instruction::Set {
                target: "NUM".to_string(),
                value: "1".to_string(),
            }],
            else_instrs: None,
        };
        runtime.execute_instruction(&if_instr).unwrap();
        assert_eq!(
            runtime.variables.get("NUM"),
            Some(RuntimeValue::Integer(1))
        );

        // FOR loop over list
        let for_instr = cnf_compiler::ir::Instruction::ForLoop {
            variable: "ITEM".to_string(),
            in_list: "a,b".to_string(),
            instrs: vec![cnf_compiler::ir::Instruction::Set {
                target: "NUM".to_string(),
                value: "2".to_string(),
            }],
        };
        runtime.execute_instruction(&for_instr).unwrap();
        assert_eq!(
            runtime.variables.get("ITEM"),
            Some(RuntimeValue::Text("b".to_string()))
        );

        // WHILE loop increments counter
        runtime
            .variables
            .set("CNT".to_string(), RuntimeValue::Text("0".to_string()));
        let while_instr = cnf_compiler::ir::Instruction::WhileLoop {
            condition: "CNT < 3".to_string(),
            instrs: vec![cnf_compiler::ir::Instruction::Set {
                target: "CNT".to_string(),
                value: "3".to_string(), // Set to 3 to exit loop
            }],
        };
        runtime.execute_instruction(&while_instr).unwrap();
        // After loop, CNT should be 3 (integer)
        assert_eq!(
            runtime.variables.get("CNT"),
            Some(RuntimeValue::Integer(3))
        );
    }
}

