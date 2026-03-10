/// Error type for CENTRA-NF runtime
#[derive(Debug, Clone)]
pub enum CnfError {
    BufferNotFound(String),
    InvalidInstruction(String),
    EncryptionFailed(String),
    DecryptionFailed(String),
    IoError(String),
    RuntimeError(String),
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

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        match self {
            RuntimeValue::Integer(n) => n.to_string(),
            RuntimeValue::Decimal(d) => d.to_string(),
            RuntimeValue::Binary(b) => hex::encode(b),
            RuntimeValue::Text(s) => s.clone(),
            RuntimeValue::List(items) => {
                format!(
                    "[{}]",
                    items
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
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
}

impl Runtime {
    /// Create new runtime with empty variable store
    pub fn new() -> Self {
        Runtime {
            variables: VariableStore::new(),
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

    /// Execute a list of IR instructions
    pub fn execute_instructions(
        &mut self,
        instructions: &[cnf_compiler::ir::Instruction],
    ) -> Result<(), CnfError> {
        use cnf_compiler::ir::Instruction;

        for instr in instructions {
            match instr {
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

                // === PHASE 4+: OTHER INSTRUCTIONS (stubs) ===
                Instruction::Compress { target: _ } => {
                    return Err(CnfError::InvalidInstruction(
                        "Compress not yet implemented".to_string(),
                    ));
                }
                Instruction::VerifyIntegrity { target: _ } => {
                    return Err(CnfError::InvalidInstruction(
                        "VerifyIntegrity not yet implemented".to_string(),
                    ));
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
                        println!("{}", val.to_string());
                    }
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

    /// SET target value
    /// Sets a variable to a literal value or reference to another variable
    fn dispatch_set(&mut self, target: &str, value_expr: &str) -> Result<(), CnfError> {
        // Try to parse as integer
        if let Ok(n) = value_expr.parse::<i64>() {
            self.variables
                .set(target.to_string(), RuntimeValue::Integer(n));
            return Ok(());
        }

        // Try to parse as float
        if let Ok(d) = value_expr.parse::<f64>() {
            self.variables
                .set(target.to_string(), RuntimeValue::Decimal(d));
            return Ok(());
        }

        // Otherwise treat as variable reference
        if let Some(val) = self.variables.get(value_expr) {
            self.variables.set(target.to_string(), val);
            return Ok(());
        }

        // Default: treat as text
        self.variables
            .set(target.to_string(), RuntimeValue::Text(value_expr.to_string()));
        Ok(())
    }

    /// ADD operand1 + operand2 → target
    /// Performs numeric addition with type coercion
    fn dispatch_add(
        &mut self,
        target: &str,
        op1_name: &str,
        op2_name: &str,
    ) -> Result<(), CnfError> {
        let op1 = self.resolve_operand(op1_name)?;
        let op2 = self.resolve_operand(op2_name)?;

        // Try integer addition first
        if let (Ok(a), Ok(b)) = (op1.as_integer(), op2.as_integer()) {
            self.variables
                .set(target.to_string(), RuntimeValue::Integer(a + b));
            return Ok(());
        }

        // Fall back to decimal
        let a = op1.as_decimal()?;
        let b = op2.as_decimal()?;
        self.variables
            .set(target.to_string(), RuntimeValue::Decimal(a + b));
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

        // Try integer subtraction first
        if let (Ok(a), Ok(b)) = (op1.as_integer(), op2.as_integer()) {
            self.variables
                .set(target.to_string(), RuntimeValue::Integer(a - b));
            return Ok(());
        }

        // Fall back to decimal
        let a = op1.as_decimal()?;
        let b = op2.as_decimal()?;
        self.variables
            .set(target.to_string(), RuntimeValue::Decimal(a - b));
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

        // Try integer multiplication first
        if let (Ok(a), Ok(b)) = (op1.as_integer(), op2.as_integer()) {
            self.variables
                .set(target.to_string(), RuntimeValue::Integer(a * b));
            return Ok(());
        }

        // Fall back to decimal
        let a = op1.as_decimal()?;
        let b = op2.as_decimal()?;
        self.variables
            .set(target.to_string(), RuntimeValue::Decimal(a * b));
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

        // Try integer division first
        if let (Ok(a), Ok(b)) = (op1.as_integer(), op2.as_integer()) {
            if b == 0 {
                return Err(CnfError::RuntimeError("Division by zero".to_string()));
            }
            self.variables
                .set(target.to_string(), RuntimeValue::Integer(a / b));
            return Ok(());
        }

        // Fall back to decimal
        let a = op1.as_decimal()?;
        let b = op2.as_decimal()?;
        if b == 0.0 {
            return Err(CnfError::RuntimeError("Division by zero".to_string()));
        }
        self.variables
            .set(target.to_string(), RuntimeValue::Decimal(a / b));
        Ok(())
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
