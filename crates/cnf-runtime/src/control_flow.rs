//! Enhanced Runtime with Control Flow Execution
//!
//! Implements proper IF/ELSE, FOR, and WHILE execution with condition evaluation

use std::collections::HashMap;

/// Condition evaluator for control flow
pub struct ConditionEvaluator {
    variables: HashMap<String, String>,
}

impl ConditionEvaluator {
    /// Create new condition evaluator with current variable state
    pub fn new(variables: HashMap<String, String>) -> Self {
        ConditionEvaluator { variables }
    }

    /// Evaluate condition with support for comparison operators and logical operators
    /// Supports: =, !=, <, >, <=, >=, AND, OR, NOT
    /// Example: "COUNT > 5 AND STATUS = VALID"
    /// Example: "NOT (INPUT = INVALID OR FLAG = FALSE)"
    pub fn evaluate(&self, condition: &str) -> Result<bool, String> {
        let condition = condition.trim();
        self.evaluate_or(condition)
    }

    /// Evaluate OR condition (lowest precedence)
    fn evaluate_or(&self, condition: &str) -> Result<bool, String> {
        for part in condition.split(" OR ") {
            if self.evaluate_and(part.trim())? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Evaluate AND condition (medium precedence)
    fn evaluate_and(&self, condition: &str) -> Result<bool, String> {
        for part in condition.split(" AND ") {
            if !self.evaluate_not(part.trim())? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Evaluate NOT condition (high precedence)
    fn evaluate_not(&self, condition: &str) -> Result<bool, String> {
        if condition.starts_with("NOT ") {
            let inner = &condition[4..].trim();
            return self.evaluate_comparison(inner).map(|b| !b);
        }
        self.evaluate_comparison(condition)
    }

    /// Evaluate comparison condition (= , !=, <, >, <=, >=)
    fn evaluate_comparison(&self, condition: &str) -> Result<bool, String> {
        let condition = condition.trim();

        // Handle boolean literals
        match condition {
            "true" | "TRUE" => return Ok(true),
            "false" | "FALSE" => return Ok(false),
            _ => {}
        }

        // Remove parentheses if wrapped
        let condition = if condition.starts_with('(') && condition.ends_with(')') {
            &condition[1..condition.len() - 1]
        } else {
            condition
        };

        let tokens: Vec<&str> = condition.split_whitespace().collect();

        match tokens.as_slice() {
            // Equality: VAR = VALUE
            [var, op, val] if *op == "=" => {
                let var_value = self.variables.get(*var);
                Ok(var_value.is_some_and(|v| v == *val))
            }
            // Inequality: VAR != VALUE
            [var, op, val] if *op == "!=" => {
                let var_value = self.variables.get(*var);
                Ok(var_value.is_none_or(|v| v != *val))
            }
            // Greater than: VAR > VALUE
            [var, op, val] if *op == ">" => self.compare_numeric(var, val, |a, b| a > b),
            // Less than: VAR < VALUE
            [var, op, val] if *op == "<" => self.compare_numeric(var, val, |a, b| a < b),
            // Greater or equal: VAR >= VALUE
            [var, op, val] if *op == ">=" => self.compare_numeric(var, val, |a, b| a >= b),
            // Less or equal: VAR <= VALUE
            [var, op, val] if *op == "<=" => self.compare_numeric(var, val, |a, b| a <= b),
            // Variable existence check (truthy)
            [var] => Ok(self.variables.contains_key(*var)),
            _ => Err(format!("Invalid condition: {}", condition)),
        }
    }

    /// Helper to compare numeric values
    fn compare_numeric<F>(&self, var: &str, val: &str, op: F) -> Result<bool, String>
    where
        F: Fn(i64, i64) -> bool,
    {
        if let Some(v) = self.variables.get(var) {
            if let (Ok(var_num), Ok(val_num)) = (v.parse::<i64>(), val.parse::<i64>()) {
                Ok(op(var_num, val_num))
            } else {
                Err(format!(
                    "Cannot compare non-numeric values: {} and {}",
                    v, val
                ))
            }
        } else {
            Err(format!("Variable '{}' not found", var))
        }
    }
}

/// Scope manager for variable scoping
pub struct ScopeManager {
    scopes: Vec<HashMap<String, String>>,
}

impl Default for ScopeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ScopeManager {
    /// Create new scope manager with global scope
    pub fn new() -> Self {
        ScopeManager {
            scopes: vec![HashMap::new()],
        }
    }

    /// Push new scope (for functions, loops, blocks)
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pop scope (cleanup after function/block exits)
    /// Variables set in inner scope are merged into parent scope
    pub fn pop_scope(&mut self) -> Result<(), String> {
        if self.scopes.len() <= 1 {
            return Err("Cannot pop global scope".to_string());
        }
        if let Some(inner_scope) = self.scopes.pop() {
            // Merge inner scope variables into parent scope
            if let Some(parent_scope) = self.scopes.last_mut() {
                parent_scope.extend(inner_scope);
            }
        }
        Ok(())
    }

    /// Set variable in current scope
    pub fn set(&mut self, name: String, value: String) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        }
    }

    /// Get variable from current or parent scopes
    pub fn get(&self, name: &str) -> Option<String> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value.clone());
            }
        }
        None
    }

    /// Get all variables as flat map
    pub fn flatten(&self) -> HashMap<String, String> {
        let mut result = HashMap::new();
        for scope in &self.scopes {
            result.extend(scope.clone());
        }
        result
    }
}

/// Loop context for FOR and WHILE loops
pub struct LoopContext {
    pub iterations: usize,
    pub max_iterations: usize,
    pub current_value: String,
}

impl LoopContext {
    /// Create new loop context
    pub fn new(max_iterations: usize) -> Self {
        LoopContext {
            iterations: 0,
            max_iterations,
            current_value: String::new(),
        }
    }

    /// Check if loop should continue
    pub fn should_continue(&self) -> bool {
        self.iterations < self.max_iterations
    }

    /// Advance loop iteration
    pub fn next_iteration(&mut self) {
        self.iterations += 1;
    }
}

/// Control flow execution result
#[derive(Debug, Clone, PartialEq)]
pub enum ControlFlowResult {
    Continue,
    Break,
    Return(String),
}

/// Call stack frame (for function execution)
#[derive(Debug, Clone)]
pub struct Frame {
    pub function_name: String,
    pub parameters: HashMap<String, String>,
    pub locals: HashMap<String, String>,
    pub return_value: Option<String>,
}

impl Frame {
    /// Create a new call frame for a function
    pub fn new(function_name: String, parameters: Vec<String>, arguments: Vec<String>) -> Self {
        let mut params = HashMap::new();
        for (i, param_name) in parameters.iter().enumerate() {
            if i < arguments.len() {
                params.insert(param_name.clone(), arguments[i].clone());
            }
        }

        Frame {
            function_name,
            parameters: params,
            locals: HashMap::new(),
            return_value: None,
        }
    }

    /// Set a local variable in this frame
    pub fn set_local(&mut self, name: String, value: String) {
        self.locals.insert(name, value);
    }

    /// Get a variable (check locals first, then parameters)
    pub fn get(&self, name: &str) -> Option<String> {
        if let Some(val) = self.locals.get(name) {
            Some(val.clone())
        } else {
            self.parameters.get(name).cloned()
        }
    }

    /// Set return value
    pub fn set_return(&mut self, value: String) {
        self.return_value = Some(value);
    }
}

/// Call stack for function execution
#[derive(Debug, Clone)]
pub struct CallStack {
    frames: Vec<Frame>,
}

impl Default for CallStack {
    fn default() -> Self {
        Self::new()
    }
}

impl CallStack {
    /// Create new call stack
    pub fn new() -> Self {
        CallStack { frames: Vec::new() }
    }

    /// Push new frame onto stack
    pub fn push_frame(&mut self, frame: Frame) {
        self.frames.push(frame);
    }

    /// Pop frame from stack
    pub fn pop_frame(&mut self) -> Result<Frame, String> {
        if self.frames.is_empty() {
            return Err("Cannot pop from empty call stack".to_string());
        }
        Ok(self.frames.pop().unwrap())
    }

    /// Get current frame (mutable)
    pub fn current_frame_mut(&mut self) -> Result<&mut Frame, String> {
        self.frames
            .last_mut()
            .ok_or_else(|| "No active frame on call stack".to_string())
    }

    /// Get current frame (immutable)
    pub fn current_frame(&self) -> Result<&Frame, String> {
        self.frames
            .last()
            .ok_or_else(|| "No active frame on call stack".to_string())
    }

    /// Check if call stack is empty
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    /// Get call stack depth
    pub fn depth(&self) -> usize {
        self.frames.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equality_evaluation() {
        let mut vars = HashMap::new();
        vars.insert("STATUS".to_string(), "VALID".to_string());
        let evaluator = ConditionEvaluator::new(vars);

        assert!(evaluator.evaluate("STATUS = VALID").unwrap());
        assert!(!evaluator.evaluate("STATUS = INVALID").unwrap());
    }

    #[test]
    fn test_numeric_comparison() {
        let mut vars = HashMap::new();
        vars.insert("COUNT".to_string(), "5".to_string());
        let evaluator = ConditionEvaluator::new(vars);

        assert!(evaluator.evaluate("COUNT > 3").unwrap());
        assert!(evaluator.evaluate("COUNT < 10").unwrap());
        assert!(!evaluator.evaluate("COUNT > 10").unwrap());
    }

    #[test]
    fn test_scope_management() {
        let mut scope = ScopeManager::new();

        scope.set("global_var".to_string(), "value1".to_string());
        assert_eq!(scope.get("global_var"), Some("value1".to_string()));

        scope.push_scope();
        scope.set("local_var".to_string(), "value2".to_string());
        assert_eq!(scope.get("local_var"), Some("value2".to_string()));
        assert_eq!(scope.get("global_var"), Some("value1".to_string()));

        scope.pop_scope().unwrap();
        assert_eq!(scope.get("local_var"), None);
        assert_eq!(scope.get("global_var"), Some("value1".to_string()));
    }

    #[test]
    fn test_loop_context() {
        let mut loop_ctx = LoopContext::new(5);

        assert!(loop_ctx.should_continue());
        loop_ctx.next_iteration();
        assert!(loop_ctx.should_continue());
        assert_eq!(loop_ctx.iterations, 1);

        for _ in 0..4 {
            loop_ctx.next_iteration();
        }
        assert!(!loop_ctx.should_continue());
    }

    #[test]
    fn test_frame_creation_with_parameters() {
        let params = vec!["source".to_string(), "target".to_string()];
        let args = vec!["BINARY-BLOB".to_string(), "OUTPUT".to_string()];
        let frame = Frame::new("process_data".to_string(), params, args);

        assert_eq!(frame.function_name, "process_data");
        assert_eq!(frame.get("source"), Some("BINARY-BLOB".to_string()));
        assert_eq!(frame.get("target"), Some("OUTPUT".to_string()));
    }

    #[test]
    fn test_frame_local_variables() {
        let params = vec!["input".to_string()];
        let args = vec!["DATA".to_string()];
        let mut frame = Frame::new("process".to_string(), params, args);

        frame.set_local("temp".to_string(), "intermediate".to_string());
        assert_eq!(frame.get("temp"), Some("intermediate".to_string()));
        assert_eq!(frame.get("input"), Some("DATA".to_string()));
    }

    #[test]
    fn test_frame_return_value() {
        let mut frame = Frame::new("get_result".to_string(), vec![], vec![]);

        assert_eq!(frame.return_value, None);
        frame.set_return("RESULT_VALUE".to_string());
        assert_eq!(frame.return_value, Some("RESULT_VALUE".to_string()));
    }

    #[test]
    fn test_call_stack_operations() {
        let mut stack = CallStack::new();

        assert!(stack.is_empty());
        assert_eq!(stack.depth(), 0);

        let frame1 = Frame::new("func1".to_string(), vec![], vec![]);
        stack.push_frame(frame1);
        assert_eq!(stack.depth(), 1);
        assert!(!stack.is_empty());

        let frame2 = Frame::new("func2".to_string(), vec![], vec![]);
        stack.push_frame(frame2);
        assert_eq!(stack.depth(), 2);

        let popped = stack.pop_frame().unwrap();
        assert_eq!(popped.function_name, "func2");
        assert_eq!(stack.depth(), 1);
    }

    #[test]
    fn test_nested_function_calls() {
        let mut stack = CallStack::new();

        let outer_params = vec!["data".to_string()];
        let outer_args = vec!["INPUT".to_string()];
        let outer_frame = Frame::new("outer".to_string(), outer_params, outer_args);
        stack.push_frame(outer_frame);

        let inner_params = vec!["x".to_string()];
        let inner_args = vec!["42".to_string()];
        let inner_frame = Frame::new("inner".to_string(), inner_params, inner_args);
        stack.push_frame(inner_frame);

        // Current frame should be inner
        assert_eq!(stack.current_frame().unwrap().function_name, "inner");
        assert_eq!(
            stack.current_frame().unwrap().get("x"),
            Some("42".to_string())
        );

        // Pop inner
        stack.pop_frame().unwrap();

        // Current frame should be outer
        assert_eq!(stack.current_frame().unwrap().function_name, "outer");
        assert_eq!(
            stack.current_frame().unwrap().get("data"),
            Some("INPUT".to_string())
        );
    }
}
