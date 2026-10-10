//! Golden for Yijiang `shenxing`, rewritten from legacy JavaScript.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
    SkillId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SHENXING: u32 = 3_283_042_324;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

fn started() -> (GameState, PlayerId, SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 2_026_071_522, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SHENXING),
        }],
    )
    .unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == SHENXING)
        .unwrap();
    (tx.state().clone(), owner, skill)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let cards = state.hand_cards(owner)[..2].to_vec();
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: cards,
            },
        },
    )
}

#[test]
fn shenxing_discards_two_draws_one_and_has_no_implicit_use_limit() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == SHENXING)
        .unwrap();
    assert_eq!(active.uses_per_turn, None);
    let selection = active.selection.as_ref().unwrap();
    assert_eq!(
        (selection.zone.as_str(), selection.min, selection.max),
        ("hand_equip", 2, 2)
    );

    let (state, owner, skill) = started();
    let hand = state.hand_count(owner);
    let (state, first) = activate(state, owner, skill, 1);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand - 1);

    let (state, second) = activate(state, owner, skill, 2);
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand - 2);
    assert_eq!(state.skills.get(skill).unwrap().use_count, 2);
}
