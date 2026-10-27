//! Diagnostic: a continuation can still be redirected to another real choice site.
use rule_vm::{op, BytecodeModule, CollectingHost, ExecResult, RuleVm, VerifiedModule};

#[test]
fn redirecting_to_another_choice_site_is_not_authenticated() {
    let code = vec![
        op::HOST_SELF,
        op::CONST_I32,
        7,
        0,
        0,
        0,
        op::HOST_CHOICE,
        1,
        op::POP,
        op::HALT,
        op::CONST_I32,
        7,
        0,
        0,
        0,
        op::HOST_CHOICE,
        1,
        op::HOST_EFFECT,
        1,
        op::HALT,
    ];
    let name = "resume-authentication-probe".to_string();
    let raw = BytecodeModule {
        logic_hash: BytecodeModule::compute_logic_hash(&name, &code, 0, 0, 0),
        name,
        code,
        local_count: 0,
        max_stack: 8,
        entry_pc: 0,
        caps_mask: 0,
    };
    let module = VerifiedModule::verify(raw).unwrap();
    let mut vm = RuleVm::new();
    let mut host = CollectingHost::default();
    let ExecResult::Paused { continuation, .. } = vm.execute(&module, &mut host, 100).unwrap()
    else {
        panic!("expected choice")
    };
    let mut redirected = continuation.clone();
    assert_eq!(continuation.pc, 8);
    redirected.pc = 17;
    vm.resume(&module, &mut host, continuation, 7).unwrap();
    assert!(host.effects.is_empty());
    vm.resume(&module, &mut host, redirected, 7).unwrap();
    assert_eq!(host.effects.len(), 1);
    println!(
        "same module: issued pc=8 emits 0 effects; forged choice-site pc=17 emits {:?}",
        host.effects
    );
}
