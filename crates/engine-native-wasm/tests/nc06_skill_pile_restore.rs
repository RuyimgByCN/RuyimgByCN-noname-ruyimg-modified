//! The production archive wrapper restores a skill-owned physical card pile.
use engine_native::{ChoiceOption, CommandOutcome, GameAction, GameCommand};
use engine_native_wasm::test_support::{
    assign_general_for, engine_from_archive, engine_from_packs, hex_encode,
};
use serde_json::{json, Value};

#[test]
fn installed_skill_pile_restores_and_corrupt_duplicate_is_rejected() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/wudian-upstream-noname");
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(root.join("package.npack.json")).unwrap()).unwrap();
    let cards: Value =
        serde_json::from_slice(&std::fs::read(root.join("assets/cards.json")).unwrap()).unwrap();
    let packs = json!([{ "manifest": manifest["manifest"], "cards": cards }]).to_string();
    let mut engine = engine_from_packs(3, 3501, &packs).unwrap();
    assign_general_for(&mut engine, 0, "刘馥").unwrap();
    // R05-A2 H73：筑城确认门在整回合结束窗 phaseEnd 段（经过弃牌阶段 discard_to_limit 后到达 trigger_confirm）
    for _ in 0..64 {
        let (actor, action) = match engine.open_choice().cloned() {
            Some(choice) if choice.spec.prompt == "trigger_confirm" => {
                assert_eq!(choice.observer, 0);
                (
                    0,
                    GameAction::Answer {
                        choice_id: choice.id,
                        selection: vec![ChoiceOption::Confirm],
                    },
                )
            }
            Some(choice) => {
                let sel: Vec<ChoiceOption> = choice
                    .spec
                    .options
                    .iter()
                    .take(choice.spec.min.max(1))
                    .copied()
                    .collect();
                (
                    choice.observer,
                    GameAction::Answer {
                        choice_id: choice.id,
                        selection: sel,
                    },
                )
            }
            None => (engine.turn().current, GameAction::NextPhase),
        };
        let result = engine.submit(GameCommand {
            actor,
            expected_revision: engine.revision(),
            dedup_id: engine.revision() + 1,
            action,
        });
        assert!(!matches!(result, CommandOutcome::Rejected(_)), "{result:?}");
        if !engine.state_core().player(0).zhucheng.is_empty() {
            break;
        }
    }
    let stored = engine.state_core().player(0).zhucheng.clone();
    assert_eq!(stored.len(), 1);
    let snapshot = engine.save();
    let hex = hex_encode(&engine_native::archive::encode_archive(&snapshot));
    let restored = engine_from_archive(&hex, &packs).unwrap();
    assert_eq!(restored.state_core().player(0).zhucheng, stored);
    assert_eq!(restored.state_hash(), engine.state_hash());
    assert!(engine_from_archive(&hex, "[]").is_err());
    let mut duplicate = snapshot;
    duplicate.core.deck.push(stored[0]);
    let bad = hex_encode(&engine_native::archive::encode_archive(&duplicate));
    let error = engine_from_archive(&bad, &packs).err().unwrap();
    assert!(error.contains("card_in_multiple_zones"), "{error}");
    assert_eq!(engine.state_core().player(0).zhucheng, stored);
}
