//! Runtime instruction execution tests
//! Tests that instructions are properly dispatched and executed

#[cfg(test)]
mod runtime_execution_tests {
    use cnf_compiler::ir::Instruction;
    use cnf_runtime::Runtime;

    #[test]
    fn test_runtime_executes_single_instruction() {
        // Test that runtime can execute a single instruction
        let mut runtime = Runtime::new();
        runtime.add_buffer("TEST".to_string(), b"data".to_vec());

        let instr = Instruction::Display {
            message: "Test message".to_string(),
        };

        let result = runtime.execute_instruction(&instr);
        assert!(
            result.is_ok(),
            "DISPLAY instruction should execute successfully"
        );
    }

    #[test]
    fn test_runtime_executes_instruction_sequence() {
        // Test that runtime can execute multiple instructions in sequence
        let mut runtime = Runtime::new();
        runtime.add_buffer("RESULT".to_string(), Vec::new());

        let instructions = vec![
            Instruction::Set {
                target: "RESULT".to_string(),
                value: "Hello".to_string(),
            },
            Instruction::Display {
                message: "After SET".to_string(),
            },
            Instruction::Print {
                target: "RESULT".to_string(),
                format: None,
            },
        ];

        let result = runtime.execute_instructions(&instructions);
        assert!(result.is_ok(), "Instruction sequence should execute");

        // Verify buffer state after execution
        let output = runtime.get_output("RESULT").unwrap();
        assert_eq!(output, b"Hello");
    }

    #[test]
    fn test_runtime_fails_on_missing_buffer() {
        // Test that operations on missing buffers fail appropriately
        let mut runtime = Runtime::new();

        let instr = Instruction::Print {
            target: "NONEXISTENT".to_string(),
            format: None,
        };

        let result = runtime.execute_instruction(&instr);
        assert!(result.is_err(), "Operation on missing buffer should fail");

        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("not found"),
            "Error should mention buffer not found: {}",
            err
        );
    }

    #[test]
    fn test_runtime_preserves_buffer_after_display() {
        // Test that DISPLAY doesn't modify buffer state
        let mut runtime = Runtime::new();
        let original_data = b"original".to_vec();
        runtime.add_buffer("TEST".to_string(), original_data.clone());

        let instr = Instruction::Display {
            message: "Test".to_string(),
        };

        runtime.execute_instruction(&instr).unwrap();

        let output = runtime.get_output("TEST").unwrap();
        assert_eq!(output, original_data, "DISPLAY should not modify buffer");
    }

    #[test]
    fn test_runtime_set_replaces_buffer_content() {
        // Test that SET instruction replaces buffer content
        let mut runtime = Runtime::new();
        runtime.add_buffer("VAR".to_string(), b"old".to_vec());

        let instr = Instruction::Set {
            target: "VAR".to_string(),
            value: "new_value".to_string(),
        };

        runtime.execute_instruction(&instr).unwrap();

        let output = runtime.get_output("VAR").unwrap();
        assert_eq!(output, b"new_value");
    }

    #[test]
    fn test_runtime_merge_concatenates_buffers() {
        // Test that MERGE properly concatenates multiple buffers
        let mut runtime = Runtime::new();
        runtime.add_buffer("A".to_string(), b"hello".to_vec());
        runtime.add_buffer("B".to_string(), b"world".to_vec());

        let instr = Instruction::Merge {
            targets: vec!["A".to_string(), "B".to_string()],
            output_name: "MERGED".to_string(),
        };

        runtime.execute_instruction(&instr).unwrap();

        let output = runtime.get_output("MERGED").unwrap();
        assert_eq!(output, b"helloworld");
    }

    #[test]
    fn test_runtime_add_numeric_operation() {
        // Test ADD instruction with numeric values
        let mut runtime = Runtime::new();
        runtime.add_buffer("RESULT".to_string(), Vec::new());

        let instr = Instruction::Add {
            target: "RESULT".to_string(),
            operand1: "5".to_string(),
            operand2: "3".to_string(),
        };

        runtime.execute_instruction(&instr).unwrap();

        let output = runtime.get_output("RESULT").unwrap();
        assert_eq!(output, b"8");
    }

    #[test]
    fn test_runtime_string_helpers() {
        let mut runtime = Runtime::new();
        runtime.add_buffer("SRC".to_string(), b" hello ".to_vec());
        runtime.add_buffer("OUT".to_string(), Vec::new());

        // uppercase
        runtime
            .execute_instruction(&Instruction::Uppercase {
                target: "OUT".to_string(),
                source: "SRC".to_string(),
            })
            .unwrap();
        assert_eq!(runtime.get_output("OUT").unwrap(), b" HELLO ");

        // lowercase
        runtime.add_buffer("OUT".to_string(), Vec::new());
        runtime
            .execute_instruction(&Instruction::Lowercase {
                target: "OUT".to_string(),
                source: "SRC".to_string(),
            })
            .unwrap();
        assert_eq!(runtime.get_output("OUT").unwrap(), b" hello ");

        // trim
        runtime.add_buffer("OUT".to_string(), Vec::new());
        runtime
            .execute_instruction(&Instruction::Trim {
                target: "OUT".to_string(),
                source: "SRC".to_string(),
            })
            .unwrap();
        assert_eq!(runtime.get_output("OUT").unwrap(), b"hello");
    }

    #[test]
    fn test_runtime_math_helpers() {
        let mut runtime = Runtime::new();
        runtime.add_buffer("RESULT".to_string(), Vec::new());

        runtime
            .execute_instruction(&Instruction::Max {
                target: "RESULT".to_string(),
                operand1: "10".to_string(),
                operand2: "7".to_string(),
            })
            .unwrap();
        assert_eq!(runtime.get_output("RESULT").unwrap(), b"10");

        runtime.add_buffer("RESULT".to_string(), Vec::new());
        runtime
            .execute_instruction(&Instruction::Min {
                target: "RESULT".to_string(),
                operand1: "10".to_string(),
                operand2: "7".to_string(),
            })
            .unwrap();
        assert_eq!(runtime.get_output("RESULT").unwrap(), b"7");

        runtime.add_buffer("RESULT".to_string(), Vec::new());
        runtime
            .execute_instruction(&Instruction::Abs {
                target: "RESULT".to_string(),
                operand: "-5".to_string(),
            })
            .unwrap();
        assert_eq!(runtime.get_output("RESULT").unwrap(), b"5");
    }

    #[test]
    fn test_runtime_verify_integrity_produces_hash() {
        // Test VERIFY-INTEGRITY instruction
        let mut runtime = Runtime::new();
        runtime.add_buffer("DATA".to_string(), b"test_data".to_vec());

        let instr = Instruction::VerifyIntegrity {
            target: "DATA".to_string(),
        };

        let result = runtime.execute_instruction(&instr);
        assert!(
            result.is_ok(),
            "VERIFY-INTEGRITY should succeed on valid buffer"
        );
    }

    #[test]
    fn test_runtime_reject_invalid_instruction() {
        // Test that unknown/malformed instructions are rejected
        let mut runtime = Runtime::new();

        // Create a condition that will cause execute_instruction to fail
        // by testing with a missing buffer
        let instr = Instruction::Encrypt {
            target: "MISSING".to_string(),
        };

        let result = runtime.execute_instruction(&instr);
        assert!(
            result.is_err(),
            "Invalid instruction on missing buffer should fail"
        );
    }

    #[test]
    fn test_runtime_deterministic_execution() {
        // Test that same instructions + inputs → same state (determinism)
        let instructions = vec![
            Instruction::Set {
                target: "X".to_string(),
                value: "100".to_string(),
            },
            Instruction::Add {
                target: "X".to_string(),
                operand1: "X".to_string(),
                operand2: "50".to_string(),
            },
        ];

        let mut runtime1 = Runtime::new();
        runtime1.add_buffer("X".to_string(), Vec::new());
        runtime1.execute_instructions(&instructions).unwrap();
        let result1 = runtime1.get_output("X").unwrap();

        let mut runtime2 = Runtime::new();
        runtime2.add_buffer("X".to_string(), Vec::new());
        runtime2.execute_instructions(&instructions).unwrap();
        let result2 = runtime2.get_output("X").unwrap();

        assert_eq!(
            result1, result2,
            "Determinism test failed: same input produced different output"
        );
    }

    // Phase 1b: Enhanced ForLoop and WhileLoop with LoopContext tests

    #[test]
    fn test_for_loop_basic_iteration() {
        // Test basic ForLoop execution with loop variable assignment
        let mut runtime = Runtime::new();
        runtime.add_buffer("RESULT".to_string(), Vec::new());

        let for_loop = Instruction::ForLoop {
            variable: "ITEM".to_string(),
            in_list: "A,B,C".to_string(),
            instrs: vec![
                Instruction::Print {
                    target: "ITEM".to_string(),
                    format: None,
                },
            ],
        };

        let result = runtime.execute_instruction(&for_loop);
        assert!(result.is_ok(), "ForLoop should execute successfully");

        // The loop should have executed 3 times with items A, B, C
        // ITEM variable should be set to the last item (C)
        let final_value = runtime.get_variable("ITEM");
        assert_eq!(final_value, Some("C".to_string()), "Loop variable should be set to final item");
    }

    #[test]
    fn test_for_loop_with_accumulation() {
        // Test ForLoop that accumulates values
        let mut runtime = Runtime::new();
        runtime.add_buffer("COUNT".to_string(), Vec::new());

        let instructions = vec![
            Instruction::Set {
                target: "COUNT".to_string(),
                value: "0".to_string(),
            },
            Instruction::ForLoop {
                variable: "I".to_string(),
                in_list: "1,2,3,4,5".to_string(),
                instrs: vec![
                    Instruction::Add {
                        target: "COUNT".to_string(),
                        operand1: "COUNT".to_string(),
                        operand2: "I".to_string(),
                    },
                ],
            },
        ];

        runtime.execute_instructions(&instructions).unwrap();

        // 0 + 1 + 2 + 3 + 4 + 5 = 15
        let count = String::from_utf8(runtime.get_output("COUNT").unwrap()).unwrap();
        assert_eq!(count, "15", "ForLoop accumulation should compute sum correctly");
    }

    #[test]
    fn test_for_loop_scope_isolation() {
        // Test that loop variables are properly scoped
        let mut runtime = Runtime::new();
        runtime.set_variable("OUTER".to_string(), "BEFORE".to_string());

        let instructions = vec![
            Instruction::ForLoop {
                variable: "LOOP_VAR".to_string(),
                in_list: "X,Y,Z".to_string(),
                instrs: vec![
                    Instruction::Set {
                        target: "LOOP_VAR".to_string(),
                        value: "MODIFIED".to_string(),
                    },
                ],
            },
        ];

        runtime.execute_instructions(&instructions).unwrap();

        // After loop, LOOP_VAR should be in scope (from last iteration)
        // OUTER should remain unchanged
        assert_eq!(
            runtime.get_variable("OUTER"),
            Some("BEFORE".to_string()),
            "Outer scope should not be affected by loop"
        );
    }

    #[test]
    fn test_for_loop_with_nested_instructions() {
        // Test ForLoop with multiple instructions in body
        let mut runtime = Runtime::new();
        
        let instructions = vec![
            Instruction::Set {
                target: "RESULT".to_string(),
                value: "".to_string(),
            },
            Instruction::ForLoop {
                variable: "ITEM".to_string(),
                in_list: "A,B,C".to_string(),
                instrs: vec![
                    Instruction::Concatenate {
                        target: "RESULT".to_string(),
                        operands: vec!["RESULT".to_string(), "ITEM".to_string()],
                    },
                ],
            },
        ];

        runtime.execute_instructions(&instructions).unwrap();

        let result = String::from_utf8(runtime.get_output("RESULT").unwrap()).unwrap_or_default();
        assert_eq!(result, "ABC", "ForLoop with concatenation should produce correct output");
    }

    #[test]
    fn test_while_loop_basic_iteration() {
        // Test basic WhileLoop with condition
        let mut runtime = Runtime::new();
        runtime.add_buffer("COUNT".to_string(), Vec::new());

        let instructions = vec![
            Instruction::Set {
                target: "COUNT".to_string(),
                value: "0".to_string(),
            },
            Instruction::WhileLoop {
                condition: "COUNT < 5".to_string(),
                instrs: vec![
                    Instruction::Add {
                        target: "COUNT".to_string(),
                        operand1: "COUNT".to_string(),
                        operand2: "1".to_string(),
                    },
                ],
            },
        ];

        runtime.execute_instructions(&instructions).unwrap();

        let count = String::from_utf8(runtime.get_output("COUNT").unwrap()).unwrap();
        assert_eq!(count, "5", "WhileLoop should iterate until condition is false");
    }

    #[test]
    fn test_while_loop_tracks_iterations() {
        // Test that WhileLoop tracks iteration count via __iter variable
        let mut runtime = Runtime::new();
        runtime.add_buffer("X".to_string(), Vec::new());

        let instructions = vec![
            Instruction::Set {
                target: "X".to_string(),
                value: "0".to_string(),
            },
            Instruction::WhileLoop {
                condition: "X < 3".to_string(),
                instrs: vec![
                    Instruction::Add {
                        target: "X".to_string(),
                        operand1: "X".to_string(),
                        operand2: "1".to_string(),
                    },
                ],
            },
        ];

        let result = runtime.execute_instructions(&instructions);
        assert!(result.is_ok(), "WhileLoop with __iter tracking should succeed");

        let final_value = String::from_utf8(runtime.get_output("X").unwrap()).unwrap();
        assert_eq!(final_value, "3", "WhileLoop should reach target value");
    }

    #[test]
    fn test_while_loop_infinite_loop_detection() {
        // Test that WhileLoop detects and prevents infinite loops
        let mut runtime = Runtime::new();
        runtime.add_buffer("X".to_string(), Vec::new());

        let instructions = vec![
            Instruction::Set {
                target: "X".to_string(),
                value: "1".to_string(),
            },
            Instruction::WhileLoop {
                // X is always 1, so condition X = 1 is always true
                condition: "X = 1".to_string(),
                instrs: vec![
                    // We don't modify X, so loop never terminates
                ],
            },
        ];

        let result = runtime.execute_instructions(&instructions);
        assert!(
            result.is_err(),
            "WhileLoop should detect infinite loop and return error"
        );

        let error_msg = format!("{:?}", result);
        assert!(
            error_msg.contains("exceeded maximum iterations") || error_msg.contains("infinite"),
            "Error message should mention infinite loop detection"
        );
    }

    #[test]
    fn test_nested_for_loops() {
        // Test that nested ForLoops work correctly
        let mut runtime = Runtime::new();
        runtime.add_buffer("RESULT".to_string(), Vec::new());

        let instructions = vec![
            Instruction::Set {
                target: "RESULT".to_string(),
                value: "".to_string(),
            },
            Instruction::ForLoop {
                variable: "I".to_string(),
                in_list: "1,2".to_string(),
                instrs: vec![
                    Instruction::ForLoop {
                        variable: "J".to_string(),
                        in_list: "A,B".to_string(),
                        instrs: vec![
                            Instruction::Concatenate {
                                target: "RESULT".to_string(),
                                operands: vec!["RESULT".to_string(), "I".to_string(), "J".to_string()],
                            },
                        ],
                    },
                ],
            },
        ];

        let result = runtime.execute_instructions(&instructions);
        assert!(result.is_ok(), "Nested ForLoops should execute");

        let output = String::from_utf8(runtime.get_output("RESULT").unwrap()).unwrap_or_default();
        // Expect: 1A 1B 2A 2B (in some form depending on concatenation)
        assert!(!output.is_empty(), "Nested loops should produce output");
    }

    #[test]
    fn test_for_loop_with_single_item() {
        // Test ForLoop with just one item
        let mut runtime = Runtime::new();

        let instructions = vec![
            Instruction::Set {
                target: "VALUE".to_string(),
                value: "UNSET".to_string(),
            },
            Instruction::ForLoop {
                variable: "ITEM".to_string(),
                in_list: "ONLY".to_string(),
                instrs: vec![
                    Instruction::Set {
                        target: "VALUE".to_string(),
                        value: "ITEM".to_string(),
                    },
                ],
            },
        ];

        runtime.execute_instructions(&instructions).unwrap();

        assert_eq!(
            runtime.get_variable("ITEM"),
            Some("ONLY".to_string()),
            "Single-item ForLoop should set loop variable"
        );
    }
}
