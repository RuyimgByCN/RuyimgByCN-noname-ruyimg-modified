use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};

#[test]
fn different_jar_pile_orders_share_hash_but_produce_different_options() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let json = std::fs::read_to_string(
        root.join("../../packages/extensions/shahai-swd-upstream-noname/assets/cards.json"),
    ).unwrap();
    let pack = content_native_loader::bridge_pack(&json).unwrap().pack;
    let mut e = Engine::new_with_extensions(3504, 4, &[&pack]).unwrap();
    for _ in 0..8 {
        if e.turn().phase == Phase::Play { break; }
        let out = e.submit(GameCommand {
            actor: 0, expected_revision: e.revision(), dedup_id: e.revision() + 1,
            action: GameAction::NextPhase,
        });
        assert!(!matches!(out, CommandOutcome::Rejected(_)));
    }
    assert_eq!(e.turn().phase, Phase::Play);
    e.debug_grant(0, "lianyaohu_lianhua");
    let first = e.state_core_mut().deck.pop().unwrap();
    let second = e.state_core_mut().deck.pop().unwrap();
    e.state_core_mut().players[0].lianyaohu_store = vec![first, second];
    let left = e.save();
    let mut right = left.clone();
    right.core.players[0].lianyaohu_store.reverse();
    let mut a = Engine::restore_with_extensions(&left, &[&pack]).unwrap();
    let mut b = Engine::restore_with_extensions(&right, &[&pack]).unwrap();
    assert_ne!(a.state_core().players[0].lianyaohu_store, b.state_core().players[0].lianyaohu_store);
    assert_eq!(a.state_hash(), b.state_hash());
    let def = a.catalog().skill_by_name("lianyaohu_lianhua").unwrap();
    for game in [&mut a, &mut b] {
        let out = game.submit(GameCommand {
            actor: 0, expected_revision: game.revision(), dedup_id: game.revision() + 1,
            action: GameAction::ActivateSkill { skill: def },
        });
        assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    }
    let options_a = &a.open_choice().unwrap().spec.options;
    let options_b = &b.open_choice().unwrap().spec.options;
    assert_ne!(options_a, options_b);
    println!("jar orders {:?} and {:?}: equal state hash, different choice options {:?} / {:?}",
        left.core.players[0].lianyaohu_store, right.core.players[0].lianyaohu_store, options_a, options_b);
}
