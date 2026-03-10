//! Phase 3 Integration Tests
//! Tests the complete compile + runtime pipeline for SET and arithmetic operations

#[cfg(test)]
mod phase3_integration {
    use cnf_compiler::compile;
    use cnf_runtime::Runtime;

    #[test]
    fn test_phase3_set_operation() {
        let source = r#"
IDENTIFICATION DIVISION.   
    PROGRAM-ID. testset.

ENVIRONMENT DIVISION.

DATA DIVISION.
    INPUT
    INPUT X NUMBER-INTEGER.

PROCEDURE DIVISION.
    SET X "42".
"#;

        // Compile
        let instructions = compile(source).expect("Compilation should succeed");
        assert!(!instructions.is_empty(), "Should generate instructions");

        // Execute
        let mut runtime = Runtime::new();
        runtime
            .execute_instructions(&instructions)
            .expect("Execution should succeed");

        // Verify
        let x_val = runtime.get_variable("X").expect("X should exist");
        assert_eq!(x_val.to_string(), "42", "X should be 42");
    }

    #[test]
    fn test_phase3_integer_addition() {
        let source = r#"
IDENTIFICATION DIVISION.
    PROGRAM-ID. testadd.

ENVIRONMENT DIVISION.

DATA DIVISION.
    INPUT
    INPUT A NUMBER-INTEGER.
    INPUT B NUMBER-INTEGER.
    RESULT NUMBER-INTEGER.

PROCEDURE DIVISION.
    SET A "10".
    SET B "5".
    ADD RESULT A B.
"#;

        // Compile
        let instructions = compile(source).expect("Compilation should succeed");

        // Execute
        let mut runtime = Runtime::new();
        runtime
            .execute_instructions(&instructions)
            .expect("Execution should succeed");

        // Verify
        let result_val = runtime.get_variable("RESULT").expect("RESULT should exist");
        assert_eq!(result_val.to_string(), "15", "10 + 5 should be 15");
    }

    #[test]
    fn test_phase3_integer_subtraction() {
        let source = r#"
IDENTIFICATION DIVISION.
    PROGRAM-ID. testsub.

ENVIRONMENT DIVISION.

DATA DIVISION.
    INPUT
    INPUT A NUMBER-INTEGER.
    INPUT B NUMBER-INTEGER.
    RESULT NUMBER-INTEGER.

PROCEDURE DIVISION.
    SET A "20".
    SET B "8".
    SUBTRACT RESULT A B.
"#;

        // Compile
        let instructions = compile(source).expect("Compilation should succeed");

        // Execute
        let mut runtime = Runtime::new();
        runtime
            .execute_instructions(&instructions)
            .expect("Execution should succeed");

        // Verify
        let result_val = runtime.get_variable("RESULT").expect("RESULT should exist");
        assert_eq!(result_val.to_string(), "12", "20 - 8 should be 12");
    }

    #[test]
    fn test_phase3_integer_multiplication() {
        let source = r#"
IDENTIFICATION DIVISION.
    PROGRAM-ID. testmult.

ENVIRONMENT DIVISION.

DATA DIVISION.
    INPUT
    INPUT A NUMBER-INTEGER.
    INPUT B NUMBER-INTEGER.
    RESULT NUMBER-INTEGER.

PROCEDURE DIVISION.
    SET A "6".
    SET B "7".
    MULTIPLY RESULT A B.
"#;

        // Compile
        let instructions = compile(source).expect("Compilation should succeed");

        // Execute
        let mut runtime = Runtime::new();
        runtime
            .execute_instructions(&instructions)
            .expect("Execution should succeed");

        // Verify
        let result_val = runtime.get_variable("RESULT").expect("RESULT should exist");
        assert_eq!(result_val.to_string(), "42", "6 * 7 should be 42");
    }

    #[test]
    fn test_phase3_integer_division() {
        let source = r#"
IDENTIFICATION DIVISION.
    PROGRAM-ID. testdiv.

ENVIRONMENT DIVISION.

DATA DIVISION.
    INPUT
    INPUT A NUMBER-INTEGER.
    INPUT B NUMBER-INTEGER.
    RESULT NUMBER-INTEGER.

PROCEDURE DIVISION.
    SET A "20".
    SET B "4".
    DIVIDE RESULT A B.
"#;

        // Compile
        let instructions = compile(source).expect("Compilation should succeed");

        // Execute
        let mut runtime = Runtime::new();
        runtime
            .execute_instructions(&instructions)
            .expect("Execution should succeed");

        // Verify
        let result_val = runtime.get_variable("RESULT").expect("RESULT should exist");
        assert_eq!(result_val.to_string(), "5", "20 / 4 should be 5");
    }

    #[test]
    fn test_phase3_decimal_arithmetic() {
        let source = r#"
IDENTIFICATION DIVISION.
    PROGRAM-ID. testdecimal.

ENVIRONMENT DIVISION.

DATA DIVISION.
    INPUT
    X NUMBER-DECIMAL.
    Y NUMBER-DECIMAL.
    RESULT NUMBER-DECIMAL.

PROCEDURE DIVISION.
    SET X "3.5".
    SET Y "2.5".
    ADD RESULT X Y.
"#;

        // Compile
        let instructions = compile(source).expect("Compilation should succeed");

        // Execute
        let mut runtime = Runtime::new();
        runtime
            .execute_instructions(&instructions)
            .expect("Execution should succeed");

        // Verify
        let result_val = runtime.get_variable("RESULT").expect("RESULT should exist");
        let result_str = result_val.to_string();
        assert!(
            result_str.starts_with("6"),
            "3.5 + 2.5 should be ~6, got: {}",
            result_str
        );
    }

    #[test]
    fn test_phase3_variable_reference() {
        let source = r#"
IDENTIFICATION DIVISION.
    PROGRAM-ID. testvarref.

ENVIRONMENT DIVISION.

DATA DIVISION.
    INPUT
    INPUT X NUMBER-INTEGER.
    INPUT Y NUMBER-INTEGER.

PROCEDURE DIVISION.
    SET X "100".
    SET Y X.
"#;

        // Compile
        let instructions = compile(source).expect("Compilation should succeed");

        // Execute
        let mut runtime = Runtime::new();
        runtime
            .execute_instructions(&instructions)
            .expect("Execution should succeed");

        // Verify
        let y_val = runtime.get_variable("Y").expect("Y should exist");
        assert_eq!(y_val.to_string(), "100", "Y should reference X's value (100)");
    }

    #[test]
    fn test_phase4_string_operations_end_to_end() {
        // Use same source as test_string_ops.cnf for full pipeline
        let source = r#"
IDENTIFICATION DIVISION.
    PROGRAM-ID. StringOpsTest.

ENVIRONMENT DIVISION.
    OS "Linux".
    ARCH "x86_64".

DATA DIVISION.
    INPUT
    INPUT TEXT-STRING AS str1.
    INPUT TEXT-STRING AS str2.
    INPUT TEXT-STRING AS str3.
    OUTPUT TEXT-STRING AS result.
    OUTPUT NUMBER-INTEGER AS length_val.

PROCEDURE DIVISION.
    SET str1 "Hello".
    SET str2 "World".
    SET str3 "Test".
    CONCATENATE result str1 str2 str3.
    LENGTH length_val result.
"#;

        // Compile
        let instructions = compile(source).expect("Compilation should succeed");
        assert!(
            instructions
                .iter()
                .any(|instr| instr.to_string().contains("CONCATENATE")),
            "IR should contain CONCATENATE"
        );

        // Execute
        let mut runtime = Runtime::new();
        runtime
            .execute_instructions(&instructions)
            .expect("Execution should succeed");

        // Verify result and length
        let res = runtime.get_variable("result").expect("result exists");
        assert_eq!(res.to_string(), "HelloWorldTest");
        let len = runtime.get_variable("length_val").expect("length_val exists");
        assert_eq!(len.to_string(), "14");
    }

    #[test]
    fn test_compile_control_flow_statements() {
        let source = r#"
IDENTIFICATION DIVISION.
    PROGRAM-ID. FlowTest.

ENVIRONMENT DIVISION.

DATA DIVISION.
    INPUT
    INPUT NUMBER-INTEGER AS X.

PROCEDURE DIVISION.
    IF X = 0 THEN
        SET X "1".
    END-IF.
    FOR I IN "a,b" DO
        SET X "2".
    END-FOR.
    WHILE X < 5 DO
        SET X "3".
    END-WHILE.
"#;
        let instructions = compile(source).expect("Compilation should succeed");
        assert!(
            instructions.iter().any(|instr| matches!(instr, cnf_compiler::ir::Instruction::IfStatement { .. })),
            "Should emit IF instruction"
        );
        assert!(
            instructions.iter().any(|instr| matches!(instr, cnf_compiler::ir::Instruction::ForLoop { .. })),
            "Should emit FOR instruction"
        );
        assert!(
            instructions.iter().any(|instr| matches!(instr, cnf_compiler::ir::Instruction::WhileLoop { .. })),
            "Should emit WHILE instruction"
        );
    }
}
