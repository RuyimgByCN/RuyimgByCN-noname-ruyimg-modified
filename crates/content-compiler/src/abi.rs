use super::*;

mod since_abi_154;
mod since_abi_353;
mod since_abi_241;
mod since_abi_677;
mod since_abi_513;

pub(super) fn required_rule_vm_abi(ir: &RuleModuleIr, module_required_abi: u32) -> u32 {
    since_abi_677::required_abi(ir, module_required_abi).max(module_required_abi)
}

fn statements_require_abi4(statements: &[StmtIr]) -> bool {
    statements.iter().any(|statement| match statement {
        StmtIr::EffectChooseUseVirtualSha { .. } => true,
        StmtIr::If {
            then_body,
            else_body,
            ..
        } => statements_require_abi4(then_body) || statements_require_abi4(else_body),
        StmtIr::Loop { body, .. } => statements_require_abi4(body),
        _ => false,
    })
}
