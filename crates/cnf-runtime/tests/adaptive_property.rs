use cnf_runtime::adaptive::*;

use proptest::prelude::*;

proptest! {
    #[test]
    fn mutation_is_deterministic(instr in prop_oneof![Just(IRInstruction::Div), Just(IRInstruction::SafeDiv)]) {
        let ir = IRBlock { instructions: vec![instr.clone()] };
        let policy = EvolutionEngine {
            max_mutation_rate: 1.0,
            max_generation_depth: 2,
            safe_instruction_set: vec![IRInstruction::SafeDiv],
        };
        let variants1 = EvolutionEngine::mutate(&ir, &policy);
        let variants2 = EvolutionEngine::mutate(&ir, &policy);
        prop_assert_eq!(variants1, variants2);
    }

    #[test]
    fn self_repair_is_deterministic(instr in prop_oneof![Just(IRInstruction::Div), Just(IRInstruction::SafeDiv)]) {
        let mut ir = IRBlock { instructions: vec![instr.clone()] };
        let error = RuntimeError::DivideByZero { instr_idx: 0 };
        let snapshot = IRSnapshot { state: "test".to_string() };
        let result1 = SelfRepairEngine::handle_error(&mut ir, &error, &snapshot);
        let mut ir2 = IRBlock { instructions: vec![instr.clone()] };
        let result2 = SelfRepairEngine::handle_error(&mut ir2, &error, &snapshot);
        prop_assert_eq!(result1.is_ok(), result2.is_ok());
        prop_assert_eq!(ir, ir2);
    }
}
