//! Golden for OLD「勘律荀勖」character closure + 勘律 damage-draw path.
//!
//! Drives shipped npack + `.skill` source (not a re-implementation).

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, schedule_damage, EffectError};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const KANLV: u32 = 2_212_101_609;
const YJSHENWEI: u32 = 3_627_315_775;
const YJ_Y_XUNXU: u32 = 3_077_663_284;

const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/old/old.npack.json");
const KANLV_SOURCE: &str = include_str!("../../../../content/original/ported/old/kanlv.skill");

fn isolated(skill_def: u32, seed: u64) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let players = engine.state().players.alive_by_seat();
    let (owner, other) = (players[0], players[1]);
    let mut tx = Transaction::begin(engine.state().clone());
    let mut setup: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    setup.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, setup).unwrap();
    (tx, owner, other)
}

fn accept(tx: Transaction, owner: PlayerId, key: u64) -> engine_core::state::GameState {
    let (state, pipeline) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_publishes_yj_y_xunxu_closure_with_both_skills() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|s| s.skill_def_id == KANLV && s.name == "kanlv"));
    assert!(package
        .skill_index
        .iter()
        .any(|s| s.skill_def_id == YJSHENWEI && s.name == "yjshenwei"));
    assert!(package
        .view_as_index
        .iter()
        .any(|v| v.skill_def_id == YJSHENWEI));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == KANLV
            && t.event == "damage"
            && t.filter == "owner_is_target"
            && t.optional
    }));

    let char = package
        .character_index
        .iter()
        .find(|c| c.name_key == "yj_y_xunxu" && c.id == YJ_Y_XUNXU)
        .expect("yj_y_xunxu");
    assert_eq!(char.skills, vec![KANLV, YJSHENWEI]);
    assert_eq!(
        char.portrait_asset.as_deref(),
        Some("character_yj_y_xunxu.jpg")
    );
    assert!(package.asset_paths.contains_key("character_yj_y_xunxu.jpg"));
}

#[test]
fn kanlv_source_and_npack_agree_on_optional_damage_draw() {
    let compiled = content_compiler::compile_source_named("kanlv-test", KANLV_SOURCE).unwrap();
    assert!(compiled.trigger_index.iter().any(|t| {
        t.skill_def_id == KANLV
            && t.event == "damage"
            && t.filter == "owner_is_target"
            && t.optional
    }));

    let (mut tx, owner, other) = isolated(KANLV, 20_260_808_41);
    let before = tx.state().hand_count(owner);

    assert_eq!(
        schedule_damage(&mut tx, other, owner, 1).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept(tx, owner, 41);
    assert_eq!(state.hand_count(owner), before + 1);
    let p = state.players.get(owner).unwrap();
    assert_eq!(p.hp, p.max_hp as i32 - 1);
}

#[test]
fn yjshenwei_loads_as_view_as_from_npack() {
    let (tx, owner, _) = isolated(YJSHENWEI, 20_260_808_42);
    assert!(tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|sid| {
            tx.state()
                .skills
                .get(*sid)
                .map(|s| s.def.raw() == YJSHENWEI)
                .unwrap_or(false)
        }));
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let view = package
        .view_as_index
        .iter()
        .find(|v| v.skill_def_id == YJSHENWEI)
        .expect("view_as");
    assert_eq!(view.as_kind.as_str(), "tao");
    assert!(view.same_suit);
    assert_eq!(view.max_cards, 2);
}
