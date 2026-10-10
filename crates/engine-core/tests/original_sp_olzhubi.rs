//! Golden coverage for SP OL 刘巴「铸币」(olzhubi).
//! Play-phase active: selected target draws 1.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
    SkillId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLZHUBI: u32 = 4_168_204_873;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/olzhubi.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId, SkillId) {
    let compiled = content_compiler::compile_source_named("olzhubi-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_53, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLZHUBI),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == OLZHUBI)
        .unwrap();
    (tx.commit().state, owner, other, skill)
}

#[test]
fn compiled_source_and_npack_publish_the_active_draw() {
    let compiled = content_compiler::compile_source_named("olzhubi", SOURCE).unwrap();
    let active = compiled
        .active_index
        .iter()
        .find(|active| active.skill_def_id == OLZHUBI)
        .expect("olzhubi active");
    assert_eq!(active.window, "play_phase");
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .active_index
        .iter()
        .any(|active| active.skill_def_id == OLZHUBI && active.window == "play_phase"));
}

#[test]
fn selected_target_draws_one() {
    let (state, owner, other, skill) = setup();
    let before = state.hand_count(other);
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![other],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }), "{:?}", result.result);
    assert_eq!(state.hand_count(other), before + 1);
}
