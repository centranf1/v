/// Integration tests: Full pipeline validation
///
/// Tests the complete flow:
/// Source (.cnf) → Lexer → Parser → AST → IR → Runtime
///
/// Each test validates determinism and explicit error handling.

#[cfg(test)]
mod integration_tests {
    use cnf_compiler::compile;
    use cnf_runtime::Runtime;

    #[test]
    fn test_pipeline_rejects_invalid_division_order() {
        let source = r#"
            ENVIRONMENT DIVISION.
            IDENTIFICATION DIVISION.
        "#;

        let result = compile(source);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("Division order error"));
    }

    #[test]
    fn test_pipeline_rejects_unquoted_env_value() {
        let source = r#"
            IDENTIFICATION DIVISION.
                PROGRAM-ID. Test.
            ENVIRONMENT DIVISION.
                OS Linux.
            DATA DIVISION.
            PROCEDURE DIVISION.
        "#;

        let result = compile(source);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("Expected quoted string"));
    }

    #[test]
    fn test_pipeline_determinism_compile_twice_same_result() {
        let source = r#"
            IDENTIFICATION DIVISION.
                PROGRAM-ID. Determinism.
            ENVIRONMENT DIVISION.
                OS "Linux".
            DATA DIVISION.
            PROCEDURE DIVISION.
        "#;

        let ir1 = compile(source).expect("First compile should succeed");
        let ir2 = compile(source).expect("Second compile should succeed");

        // Verify byte-for-byte identical IR
        // Same source → same AST → same IR (deterministically, even if empty)
        assert_eq!(
            ir1, ir2,
            "IR must be identical on repeated compilation of identical source"
        );
    }

    #[test]
    fn test_runtime_buffer_ownership() {
        let mut runtime = Runtime::new();

        // Add buffer
        let data = vec![1, 2, 3, 4, 5];
        runtime.add_buffer("test_buf".to_string(), data);

        // Retrieve buffer
        let retrieved = runtime.get_output("test_buf");
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap(), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_runtime_rejects_missing_buffer() {
        let runtime = Runtime::new();
        let result = runtime.get_output("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_error_messages_are_explicit() {
        // Test that error messages cite what was expected vs received
        let source = r#"
            DATA DIVISION.
            IDENTIFICATION DIVISION.
        "#;

        let result = compile(source);
        assert!(result.is_err());
        let error = result.unwrap_err();

        // Should explain the requirement
        assert!(error.contains("expected") || error.contains("Expected"));
        assert!(error.contains("received") || error.contains("got"));
    }
}
