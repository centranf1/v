//! PROFILE DIVISION integration tests

#[cfg(test)]
mod profile_tests {
    use cnf_compiler::compile;
    use cnf_runtime::Runtime;

    #[test]
    fn test_profile_division_lowering_and_runtime() {
        let source = r#"
            IDENTIFICATION DIVISION.
                PROGRAM-ID. ProfileTest.
            ENVIRONMENT DIVISION.
                OS "Linux".
            PROFILE DIVISION.
                HARDWARE EdgeHigh.
                MEMORY-LIMIT 4096.
                PARALLELISM 8.
            DATA DIVISION.
                INPUT VIDEO-MP4 AS VID.
            PROCEDURE DIVISION.
                COMPRESS VID.
        "#;

        let ir = compile(source).expect("PROFILE DIVISION should compile");
        let ir_strings: Vec<String> = ir.iter().map(|instr| instr.to_string()).collect();
        // SetProfile harus muncul di awal IR
        assert!(ir_strings[0].starts_with("SET_PROFILE"), "SetProfile must be first IR");
        assert!(ir_strings.iter().any(|s| s.contains("profile=EdgeHigh")));
        assert!(ir_strings.iter().any(|s| s.contains("memory_mb=Some(4096)")));
        assert!(ir_strings.iter().any(|s| s.contains("parallelism=Some(8)")));

        // Jalankan di runtime dan cek active_profile serta trace
        let mut runtime = Runtime::new();
        runtime.execute_instructions(&ir).expect("Execution should succeed");
        let profile = runtime.active_profile.as_ref().expect("Profile should be set");
        assert_eq!(profile.0, "EdgeHigh");
        assert_eq!(profile.1, Some(4096));
        assert_eq!(profile.2, Some(8));
        // Trace harus mencatat profile
        let trace = &runtime.execution_trace;
        assert!(trace.iter().any(|s| s.contains("PROFILE SET: profile=EdgeHigh")));
    }
}
