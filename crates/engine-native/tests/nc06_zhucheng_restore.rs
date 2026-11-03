//! NC06/NC07: a real skill-owned card pile must survive save and restore.
use engine_native::content::extension::ExtensionPack;
use engine_native::snap::{SnapshotError, StateSnapshot};
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};

fn submit(e: &mut Engine, actor: u8, action: GameAction) {
    let result = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: e.revision() + 1,
        action,
    });
    assert!(!matches!(result, CommandOutcome::Rejected(_)), "{result:?}");
}

fn fixture() -> (Engine, ExtensionPack) {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let json = std::fs::read_to_string(
        root.join("../../packages/extensions/wudian-upstream-noname/assets/cards.json"),
    )
    .unwrap();
    let pack = content_native_loader::bridge_pack(&json).unwrap().pack;
    let mut e = Engine::new_with_extensions(3501, 3, &[&pack]).unwrap();
    e.assign_general_named(0, "刘馥").unwrap();
    // A2 H73：筑城确认门在整回合结束窗 phaseEnd 段（K14 直推）——推进
    // p0 各阶段（途中的弃牌等窗口按最小集合作答）到窗内确认门。
    for _ in 0..64 {
        if let Some(choice) = e.open_choice().cloned() {
            if choice.spec.prompt == "trigger_confirm" {
                assert_eq!(choice.observer, 0);
                assert!(choice.spec.options.contains(&ChoiceOption::Confirm));
                submit(
                    &mut e,
                    0,
                    GameAction::Answer {
                        choice_id: choice.id,
                        selection: vec![ChoiceOption::Confirm],
                    },
                );
                break;
            }
            let observer = choice.observer;
            let sel: Vec<ChoiceOption> = choice
                .spec
                .options
                .iter()
                .take(choice.spec.min.max(1))
                .copied()
                .collect();
            submit(
                &mut e,
                observer,
                GameAction::Answer {
                    choice_id: choice.id,
                    selection: sel,
                },
            );
            continue;
        }
        let actor = e.turn().current;
        submit(&mut e, actor, GameAction::NextPhase);
    }
    assert_eq!(e.state_core().player(0).zhucheng.len(), 1);
    (e, pack)
}

fn restore_error(snapshot: &StateSnapshot, pack: &ExtensionPack) -> SnapshotError {
    Engine::restore_with_extensions(snapshot, &[pack])
        .err()
        .expect("corrupted card ownership must fail")
}

fn pile<'a>(snapshot: &'a mut StateSnapshot, name: &str) -> &'a mut Vec<u32> {
    let player = snapshot.core.player_mut(1);
    match name {
        "zhucheng" => &mut player.zhucheng,
        "lianyaohu" => &mut player.lianyaohu_store,
        "sxianjing" => &mut player.sxianjing_traps,
        "hutian" => &mut player.hutian_hold,
        _ => panic!("unknown pile: {name}"),
    }
}

#[test]
fn all_owned_piles_are_counted_once() {
    let engine = Engine::new(3502, 3);
    let mut failures = Vec::new();
    for name in ["zhucheng", "lianyaohu", "sxianjing", "hutian"] {
        let mut snapshot = engine.save();
        let card = snapshot.core.deck.pop().unwrap();
        pile(&mut snapshot, name).push(card);
        let bytes = engine_native::archive::encode_archive(&snapshot);
        let decoded = engine_native::archive::decode_archive(&bytes).unwrap();
        if Engine::restore(&decoded).is_err() {
            failures.push(format!("{name}/valid"));
        }
        let mut duplicate = snapshot.clone();
        duplicate.core.deck.push(card);
        if !matches!(Engine::restore(&duplicate), Err(SnapshotError::IdOutOfBounds { what: "card_in_multiple_zones", got }) if got == card as u64)
        {
            failures.push(format!("{name}/duplicate"));
        }
        let mut missing = snapshot.clone();
        pile(&mut missing, name).clear();
        if !matches!(Engine::restore(&missing), Err(SnapshotError::IdOutOfBounds { what: "card_missing_from_all_zones", got }) if got == card as u64)
        {
            failures.push(format!("{name}/missing"));
        }
        let mut unknown = snapshot;
        pile(&mut unknown, name)[0] = engine.catalog().cards.len() as u32;
        if !matches!(
            Engine::restore(&unknown),
            Err(SnapshotError::IdOutOfBounds { what: "card", .. })
        ) {
            failures.push(format!("{name}/unknown"));
        }
    }
    assert!(failures.is_empty(), "ownership checks: {failures:?}");
}

#[test]
fn non_owning_card_references_are_not_counted_twice() {
    let engine = Engine::new(3503, 3);
    let mut snapshot = engine.save();
    let card = snapshot.core.deck.pop().unwrap();
    let player = snapshot.core.player_mut(1);
    player.off_court.push(card);
    player.ctianfu_card = Some(card);
    player.jiguanyuan_cards.push(card);
    player.yishan_lost.push(card);
    assert!(Engine::restore(&snapshot).is_ok());
}

#[test]
fn skill_pile_archive_restores_and_continues() {
    let (mut original, pack) = fixture();
    let bytes = engine_native::archive::encode_archive(&original.save());
    let snapshot = engine_native::archive::decode_archive(&bytes).unwrap();
    assert_eq!(snapshot, original.save());
    let mut restored = Engine::restore_with_extensions(&snapshot, &[&pack]).unwrap();
    assert_eq!(restored.state_hash(), original.state_hash());
    assert_eq!(
        restored.state_core().player(0).zhucheng,
        original.state_core().player(0).zhucheng
    );
    for _ in 0..8 {
        if original.open_choice().is_some() || original.outcome().is_some() {
            break;
        }
        let actor = original.turn().current;
        submit(&mut original, actor, GameAction::NextPhase);
        submit(&mut restored, actor, GameAction::NextPhase);
        assert_eq!(restored.state_hash(), original.state_hash());
        assert_eq!(restored.open_choice(), original.open_choice());
    }
}

#[test]
fn skill_pile_rejects_duplicate_missing_and_unknown_cards() {
    let (e, pack) = fixture();
    let snapshot = e.save();
    let card = snapshot.core.player(0).zhucheng[0];
    let mut duplicate = snapshot.clone();
    duplicate.core.deck.push(card);
    assert!(matches!(
        restore_error(&duplicate, &pack),
        SnapshotError::IdOutOfBounds { what: "card_in_multiple_zones", got } if got == card as u64
    ));
    let mut missing = snapshot.clone();
    missing.core.player_mut(0).zhucheng.clear();
    assert!(matches!(
        restore_error(&missing, &pack),
        SnapshotError::IdOutOfBounds { what: "card_missing_from_all_zones", got } if got == card as u64
    ));
    let mut unknown = snapshot;
    unknown.core.player_mut(0).zhucheng[0] = e.catalog().cards.len() as u32;
    assert!(matches!(
        restore_error(&unknown, &pack),
        SnapshotError::IdOutOfBounds { what: "card", .. }
    ));
}

#[test]
fn ai_redaction_preserves_actual_zhucheng_pile() {
    let (engine, pack) = fixture();
    let res = engine.resolution();
    assert!(res.stack.is_empty() && res.parks.is_empty() && res.open_choice.is_none());
    let stored = engine.state_core().player(0).zhucheng.clone();
    let mut failures = Vec::new();
    for observer in 0..3 {
        let mut sim = engine.clone();
        engine_native::ai::redact_opponent_hands(&mut sim, observer);
        assert_eq!(sim.state_core().player(0).zhucheng, stored);
        if let Err(error) = Engine::restore_with_extensions(&sim.save(), &[&pack]) {
            failures.push(format!("observer {observer}: {error:?}"));
        }
    }
    assert!(
        failures.is_empty(),
        "real zhucheng simulation: {failures:?}"
    );
}
