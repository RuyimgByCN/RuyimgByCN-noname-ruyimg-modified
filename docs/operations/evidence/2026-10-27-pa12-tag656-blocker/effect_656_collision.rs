use rule_api::EffectRequest;
use rule_vm::{BytecodeModule, CollectingHost, HostContext, RuleVm, VerifiedModule};

#[test]
fn sp_yingshi_abi745_dispatches_its_own_effect() {
    let pkg: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../content/original/ported/sp/sp.npack.json"
    )).unwrap();
    assert_eq!(pkg["manifest"]["rule_vm_abi_version"], 745);
    let module = pkg["modules"].as_array().unwrap().iter()
        .find(|m| m["name"] == "yingshi_die_death").unwrap();
    let module: BytecodeModule = serde_json::from_value(module.clone()).unwrap();
    let mut host = CollectingHost { ctx: HostContext { self_id: 1, event_player: 2, ..Default::default() }, ..Default::default() };
    RuleVm::new().execute(&VerifiedModule::verify(module).unwrap(), &mut host, 100).unwrap();
    assert!(matches!(host.effects.as_slice(), [EffectRequest::GainOtherSkillPileCards { .. }]), "wrong effect: {:?}", host.effects);
}
