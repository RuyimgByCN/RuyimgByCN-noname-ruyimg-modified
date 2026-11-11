//! Decision29 integration: actual consumer provenance and strict private106 boundary.
use engine_native::{
    ChoiceOption, ChoiceView, CommandOutcome, Engine, GameAction, GameCommand, Phase,
};
fn answer(e: &mut Engine, selection: Vec<ChoiceOption>) {
    let c = e.open_choice().unwrap().clone();
    let out = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: e.save().last_dedup + 1,
        action: GameAction::Answer {
            choice_id: c.id,
            selection,
        },
    });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
}
fn source(e: &Engine) -> Option<&'static str> {
    let Some(ChoiceView::Own { source_skill, .. }) = e.choice_view_for(Some(1)) else {
        panic!("own")
    };
    source_skill
}
#[test]
fn phase7_actual_order_park_does_not_hide_optional_gate_consumer_source() {
    let mut e = Engine::new(290733, 3);
    e.debug_set_turn(0, Phase::Play, 1);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    for name in ["遗计", "刚烈"] {
        e.debug_grant(1, name);
    }
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "trigger_order");
    assert_eq!(
        source(&e),
        None,
        "order asks are generic, never root certificates"
    );
    let def = e.catalog().skill_by_name("遗计").unwrap();
    answer(&mut e, vec![ChoiceOption::Skill(def)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "trigger_confirm");
    let saved = e.save();
    assert!(matches!(
        saved.resolution.parks[0].machine,
        engine_native::MachineSnapshot::TriggerOrder { .. }
    ));
    assert!(saved.resolution.parks.iter().any(|p| matches!(
        p.machine,
        engine_native::MachineSnapshot::OptionalGate { .. }
    )));
    assert_eq!(
        source(&e),
        Some("遗计"),
        "source follows the formal non-order consumer, not the first same-depth order park"
    );
    let decoded =
        engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&saved))
            .unwrap();
    let mut r = Engine::restore(&decoded).unwrap();
    assert_eq!(source(&r), Some("遗计"));
    assert!(matches!(
        r.choice_view_for(Some(0)),
        Some(ChoiceView::Other { .. })
    ));
    let cursor = e.log().len();
    for engine in [&mut e, &mut r] {
        answer(engine, vec![ChoiceOption::Confirm]);
    }
    let expected = e.save();
    let mut actual = r.save();
    actual.auth_generation = expected.auth_generation;
    assert_eq!(actual, expected);
    assert_eq!(&e.log()[cursor..], r.log());
    assert_eq!(e.state_hash(), r.state_hash());
}
#[test]
fn phase7_genuine105_is_rejected_without_migration() {
    let raw = include_bytes!("fixtures/c331_phase7_genuine_105.ngsa");
    assert_eq!(&raw[..6], b"NGSAi\0");
    assert!(matches!(
        engine_native::archive::decode_archive(raw),
        Err(engine_native::archive::ArchiveError::UnsupportedFormatVersion(105))
    ));
}
#[test]
fn phase7_private106_header_and_full_state_roundtrip() {
    let e = Engine::new(2907106, 3);
    let raw = engine_native::archive::encode_archive(&e.save());
    // R05-G 批（2026-11-04）：106→107——detached 实体区入档。
    assert_eq!(u16::from_le_bytes([raw[4], raw[5]]), 107);
    let decoded = engine_native::archive::decode_archive(&raw).unwrap();
    assert_eq!(decoded, e.save());
    let r = Engine::restore(&decoded).unwrap();
    assert_eq!(r.state_hash(), e.state_hash());
}
#[test]
fn phase7_all_trigger_bearing_host_behaviors_have_bound_source_order() {
    use engine_native::content::host::HostBehavior as H;
    let trigger_bearing: Vec<_> = H::ALL
        .iter()
        .copied()
        .filter(|h| h.trigger_meta().is_some())
        .collect();
    assert_eq!(trigger_bearing.len(), 296);
    assert!(
        trigger_bearing.iter().all(|h| h.source_order().is_some()),
        "every trigger-bearing HostBehavior must have an explicit source_order tuple"
    );
}

