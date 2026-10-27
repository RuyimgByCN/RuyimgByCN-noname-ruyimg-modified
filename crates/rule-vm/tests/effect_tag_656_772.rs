use rule_api::EffectRequest;
use rule_vm::{BytecodeModule, CollectingHost, HostContext, RuleVm, VerifiedModule};

fn effect(package: &[u8], module_name: &str) -> EffectRequest {
    let package: serde_json::Value = serde_json::from_slice(package).unwrap();
    let module = package["modules"].as_array().unwrap().iter()
        .find(|module| module["name"] == module_name).unwrap();
    let module: BytecodeModule = serde_json::from_value(module.clone()).unwrap();
    let mut host = CollectingHost {
        ctx: HostContext { self_id: 1, event_player: 2, ..Default::default() },
        ..Default::default()
    };
    RuleVm::new().execute(&VerifiedModule::verify(module).unwrap(), &mut host, 100).unwrap();
    assert_eq!(host.effects.len(), 1);
    host.effects.remove(0)
}

#[test]
fn real_sp_and_corrected_onlyol_modules_dispatch_different_effects() {
    let sp = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
    let onlyol = include_bytes!("../../../content/original/ported/onlyOL/onlyOL.npack.json");
    assert!(matches!(effect(sp, "yingshi_die_death"),
        EffectRequest::GainOtherSkillPileCards { player, target, .. }
        if player.raw() == 1 && target.raw() == 2));
    assert!(matches!(effect(onlyol, "ol_pingkou_phase_end"),
        EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamageDraw { player, amount: 1, .. }
        if player.raw() == 1));
}
