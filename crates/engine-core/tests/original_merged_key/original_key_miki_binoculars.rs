//! Golden for key 望远镜 (miki_binoculars): locked — other players' hand cards
//! are face-up to the owner.

use engine_api::{Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId};
use engine_core::effect::run_effects;
use engine_core::query::observer_foreign_hand_faces_visible;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const MIKI_BINOCULARS: u32 = 3_578_894_093;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/key/key.npack.json");

fn start() -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.load_package(&pkg).unwrap();
    let result = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_261_901,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(matches!(result, CommandResult::Accepted { .. }));
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    (engine, owner, other)
}

#[test]
fn package_declares_observer_visibility_modifier() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.modifier_index.iter().any(|m| {
        m.skill_def_id == MIKI_BINOCULARS
            && m.target == "observer_foreign_hand_faces_visible"
            && m.mode == "override"
            && m.value == 1
    }));
}

#[test]
fn foreign_hand_faces_visible_to_owner() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: engine_api::SkillDefId::new(MIKI_BINOCULARS),
    });
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;

    assert!(
        observer_foreign_hand_faces_visible(&state, owner),
        "owner sees other players' hand faces"
    );
    // The owner's own hand is still hidden from the other player.
    assert!(
        !observer_foreign_hand_faces_visible(&state, other),
        "the other player does not see the owner's hand"
    );
}
