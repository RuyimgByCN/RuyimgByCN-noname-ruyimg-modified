use engine_native_wasm::NativeSession;

#[test]
fn auto_step_uses_the_same_public_card_advice_and_submits_once() {
    use engine_native::{
        ai::{projected_choice_action, BoundedPlanner},
        ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase,
    };
    let mut engine = Engine::new(2303, 3);
    engine.debug_set_turn(0, Phase::Play, 1);
    for seat in 0..3 {
        engine.debug_clear_hand(seat);
    }
    engine.debug_place(1, "shan");
    let sha = engine.debug_place(0, "sha");
    assert!(matches!(
        engine.submit(GameCommand {
            actor: 0,
            expected_revision: engine.revision(),
            dedup_id: 1,
            action: GameAction::PlayCard {
                card: sha,
                targets: vec![1]
            }
        }),
        CommandOutcome::Pending { .. }
    ));
    let choice = engine.open_choice().unwrap();
    let (observer, choice_id) = (choice.observer, choice.id);
    let action = projected_choice_action(engine.ai_session(observer).choice().unwrap()).unwrap();
    assert!(
        matches!(&action, GameAction::Answer { selection, .. } if matches!(selection.as_slice(), [ChoiceOption::Card(_)]))
    );
    let plan = engine.plan_action(observer, &BoundedPlanner::new(64));
    assert_eq!(format!("{:?}", plan.action), format!("{action:?}"));
    let GameAction::Answer { selection, .. } = action else {
        unreachable!()
    };
    let indices = selection
        .iter()
        .map(|option| {
            choice
                .spec
                .options
                .iter()
                .position(|item| item == option)
                .unwrap() as u32
        })
        .collect();
    let archive =
        engine_native_wasm::hex_encode(&engine_native::archive::encode_archive(&engine.save()));
    let mut expected = NativeSession::new(3, 0).unwrap();
    expected.restore_hex_checked(&archive).unwrap();
    expected
        .answer(observer as u32, choice_id, indices)
        .unwrap();
    let mut actual = NativeSession::new(3, 0).unwrap();
    actual.restore_hex_checked(&archive).unwrap();
    let revision = actual.revision();
    let reply: serde_json::Value = serde_json::from_str(&actual.auto_step().unwrap()).unwrap();
    assert_eq!(reply["kind"], "answer");
    assert_eq!(actual.revision(), revision + 1);
    assert_eq!(actual.save_hex(), expected.save_hex());
}
