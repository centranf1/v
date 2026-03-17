// Unit test & mutation test untuk SelfRepairEngine
use cnf_runtime::{IRBlock, IRInstruction, SelfRepairEngine};
use cnf_runtime::adaptive::{IRSnapshot, RuntimeError};

#[test]
fn test_divide_by_zero_repair() {
    let mut ir_block = IRBlock { instructions: vec![IRInstruction::Div] };
    let error = RuntimeError::DivideByZero { instr_idx: 0 };
    let snapshot = IRSnapshot { state: "initial".to_string() };

    SelfRepairEngine::handle_error(&mut ir_block, &error, &snapshot)
        .expect("Patch should succeed");

    assert!(matches!(ir_block.instructions[0], IRInstruction::SafeDiv));
}
