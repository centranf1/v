use cnf_runtime::adaptive::*;

#[test]
fn test_self_repair_divide_by_zero() {
    let mut ir = IRBlock { instructions: vec![IRInstruction::Div] };
    let error = RuntimeError::DivideByZero { instr_idx: 0 };
    let snapshot = IRSnapshot { state: "dummy".to_string() };
    let result = SelfRepairEngine::handle_error(&mut ir, &error, &snapshot);
    assert!(result.is_ok());
    assert_eq!(ir.instructions[0], IRInstruction::SafeDiv);
}

#[test]
fn test_pipeline_optimization() {
    let pipeline = Pipeline { steps: vec!["Compress".into(), "Encrypt".into()] };
    let metrics = ExecutionMetrics { performance_score: 10.0, reliability_score: 5.0, resource_cost: 2.0 };
    let optimized = PipelineOptimizer::optimize(&pipeline, &metrics);
    assert!(!optimized.steps.is_empty());
}

#[test]
fn test_evolution_logger() {
    EvolutionLogger::log_event("Test event");
}
