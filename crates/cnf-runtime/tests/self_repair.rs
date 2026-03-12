// Unit test & mutation test untuk SelfRepairEngine
use cnf_runtime::{IRBlock, IRInstruction, SelfRepairEngine};
use cnf_runtime::adaptive::RuntimeError;

#[test]
fn test_divide_by_zero_repair() {
    let ir_block = IRBlock { instructions: vec![IRInstruction::Div] };
    let error = RuntimeError::DivideByZero { instr_idx: 0 };

    let patched = SelfRepairEngine::handle_error(&error, &ir_block)
        .expect("Patch should succeed");

    assert!(matches!(patched.instructions[0], IRInstruction::SafeDiv));
}
