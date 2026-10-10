//! Probe: execute the compiled gwent hupeng active module end-to-end in the VM
//! (ADR 0626 regression guard for the skill-from-host-context fix).

use rule_api::{EffectRequest, PlayerId, SkillDefId, SkillId};
use rule_vm::{CollectingHost, RuleVm, DEFAULT_FUEL};

#[test]
fn hupeng_runtime_executes_without_underflow() {
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../content/original/ported/shahai/shahai_impl.skill")).unwrap();
    let pkg = content_compiler::compile_source_named("shahai", &src).unwrap();
    let modules: Vec<_> = pkg
        .modules
        .iter()
        .filter(|m| m.name.contains("hupeng"))
        .cloned()
        .collect();
    assert!(!modules.is_empty(), "no hupeng module compiled");
    let mut emitted = Vec::new();
    for module in &modules {
        let mut host = CollectingHost::default();
        host.ctx.self_id = 1; // the skill owner
        host.ctx.skill_id = 42; // the skill instance id
        host.ctx.event_source = 2; // the chosen target (other)
        host.selected_cards = vec![7]; // the he cost card
        host.selected_targets = vec![2];
        let mut vm = RuleVm::new();
        let result = vm.execute(module, &mut host, DEFAULT_FUEL);
        if let Err(e) = &result {
            panic!("hupeng module {:?} failed: {:?}", module.name, e);
        }
        emitted.extend(host.effects.iter().cloned());
    }
    // gw_dudayuanshuai1_skill: card_targeted trigger → cancel_incoming_card_use.
    let dud1_modules: Vec<_> = pkg
        .modules
        .iter()
        .filter(|m| m.name.contains("gw_dudayuanshuai1"))
        .cloned()
        .collect();
    let mut cancel_ok = false;
    for module in &dud1_modules {
        let mut host = CollectingHost::default();
        host.ctx.self_id = 1; // the skill owner (card holder)
        host.ctx.skill_id = 43;
        host.ctx.event_source = 2; // the other player who used the card
        host.ctx.event_card = 9; // the incoming card
        host.ctx.event_player = 2;
        let mut vm = RuleVm::new();
        vm.execute(module, &mut host, DEFAULT_FUEL)
            .unwrap_or_else(|e| panic!("gw_dudayuanshuai1 module {:?} failed: {:?}", module.name, e));
        if host.effects.iter().any(|e| matches!(e, EffectRequest::CancelIncomingCardUse { player, source, card, skill }
            if *player == PlayerId::new(1) && *source == PlayerId::new(2)
            && card == &rule_api::CardId::new(9) && *skill == SkillId::new(43))) {
            cancel_ok = true;
        }
    }
    assert!(cancel_ok, "gw_dudayuanshuai1 did not emit CancelIncomingCardUse");
    // gwyewu: active module → gwyewu_random_discard_chain host effect.
    let gwyewu_modules: Vec<_> = pkg
        .modules
        .iter()
        .filter(|m| m.name.contains("gwyewu"))
        .cloned()
        .collect();
    assert!(!gwyewu_modules.is_empty(), "no gwyewu module compiled");
    let mut gwyewu_ok = false;
    for module in &gwyewu_modules {
        let mut host = CollectingHost::default();
        host.ctx.self_id = 1; // the skill owner
        host.ctx.skill_id = 44;
        host.selected_cards = vec![7]; // the hand cost card
        let mut vm = RuleVm::new();
        vm.execute(module, &mut host, DEFAULT_FUEL)
            .unwrap_or_else(|e| panic!("gwyewu module {:?} failed: {:?}", module.name, e));
        if host.effects.iter().any(|e| matches!(e, EffectRequest::ResolveGwyewuChain { player, skill, cost_cards, wuyao_def, lang_def }
            if *player == PlayerId::new(1) && *skill == SkillId::new(44)
            && cost_cards == &[rule_api::CardId::new(7)]
            && wuyao_def == &SkillDefId::new(784509644)
            && lang_def == &SkillDefId::new(2836710318))) {
            gwyewu_ok = true;
        }
    }
    assert!(gwyewu_ok, "gwyewu did not emit ResolveGwyewuChain");
    // gw_dudayuanshuai2_skill: phase_end trigger → discard_hand(self, 2).
    let dud2_modules: Vec<_> = pkg
        .modules
        .iter()
        .filter(|m| m.name.contains("gw_dudayuanshuai2"))
        .cloned()
        .collect();
    assert!(!dud2_modules.is_empty(), "no gw_dudayuanshuai2 module compiled");
    let mut dud2_ok = false;
    for module in &dud2_modules {
        let mut host = CollectingHost::default();
        host.ctx.self_id = 1;
        host.ctx.skill_id = 45;
        host.ctx.event_player = 1;
        let mut vm = RuleVm::new();
        vm.execute(module, &mut host, DEFAULT_FUEL)
            .unwrap_or_else(|e| panic!("gw_dudayuanshuai2 module {:?} failed: {:?}", module.name, e));
        if host.effects.iter().any(|e| matches!(e, EffectRequest::DiscardFromHand { player, count }
            if *player == PlayerId::new(1) && *count == 2)) {
            dud2_ok = true;
        }
    }
    assert!(dud2_ok, "gw_dudayuanshuai2 did not emit DiscardHand(2)");
    let ok = emitted.iter().any(|e| {
        matches!(e, EffectRequest::ChooseOtherGainOneOfTwoSkills { player, target, first_def, second_def, skill }
            if *player == PlayerId::new(1) && *target == PlayerId::new(2)
            && first_def == &SkillDefId::new(3583584906)
            && second_def == &SkillDefId::new(3832623127)
            && *skill == SkillId::new(42))
    });
    assert!(ok, "hupeng did not emit the expected two-skills effect: {:?}", emitted);
}
