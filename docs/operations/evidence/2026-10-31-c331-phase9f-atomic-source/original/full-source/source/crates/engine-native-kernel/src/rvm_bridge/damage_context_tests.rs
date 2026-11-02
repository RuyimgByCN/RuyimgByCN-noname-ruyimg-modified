use super::*;
#[test]
fn negative_canonical_damage_reaches_filter_and_signed_vm_host_unchanged() {
    use rule_vm::{assemble, BytecodeModule, CollectingHost, Op, VerifiedModule};
    let event = EventData::Damage(crate::machine::damage_data(None, 0, -1));
    let trigger = trigger_ctx_of(&event);
    assert_eq!(trigger.event_amount, u32::MAX);
    assert_eq!(filter_view(&trigger).event_amount, u32::MAX);
    let code = assemble(&[
        Op::HostEventAmount,
        Op::ConstI32(-1),
        Op::Eq,
        Op::HostChoice(1),
        Op::Halt,
    ]);
    let name = "signed-damage-probe".to_string();
    let logic_hash = BytecodeModule::compute_logic_hash(&name, &code, 0, 0, 0);
    let module = BytecodeModule {
        name,
        code,
        local_count: 0,
        max_stack: 8,
        entry_pc: 0,
        logic_hash,
        caps_mask: rule_vm::bytecode::caps::REQUEST_CHOICE,
    };
    let mut host = CollectingHost {
        ctx: HostContext {
            event_amount: trigger.event_amount,
            ..Default::default()
        },
        ..Default::default()
    };
    let result = RuleVm::new()
        .execute(&VerifiedModule::verify(module).unwrap(), &mut host, 100)
        .unwrap();
    assert!(
        matches!(result,ExecResult::Paused{options,..}if options==vec![1]),
        "HOST_EVENT_AMOUNT must evaluate to signed -1"
    );
}
