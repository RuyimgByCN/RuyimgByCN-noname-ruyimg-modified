//! Golden for Yijiang `sanyao`, rewritten from legacy JavaScript.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
    SkillId,
};
use engine_core::canonical_state_hash;
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SANYAO: u32 = 3_737_839_931;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

fn started() -> (GameState, PlayerId, PlayerId, PlayerId, SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 2_026_071_717, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let players = tx.state().players.alive_by_seat();
    let owner = tx.state().turn.current;
    let mut others = players.into_iter().filter(|player| *player != owner);
    let highest = others.next().unwrap();
    let lower = others.next().unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 3;
    tx.state_mut().players.get_mut(highest).unwrap().hp = 4;
    tx.state_mut().players.get_mut(lower).unwrap().hp = 2;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SANYAO),
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
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == SANYAO)
        .unwrap();
    (tx.state().clone(), owner, highest, lower, skill)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    card: engine_api::CardId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: vec![card],
            },
        },
    )
}

#[test]
fn package_exposes_highest_hp_target_and_hand_equip_cost() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == SANYAO)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    let targets = active.targets.as_ref().unwrap();
    assert_eq!((targets.min, targets.max), (1, 1));
    assert!(targets.allow_self);
    assert!(targets.highest_hp);
    assert!(!targets.lowest_hp);
    let selection = active.selection.as_ref().unwrap();
    assert_eq!(
        (selection.zone.as_str(), selection.min, selection.max),
        ("hand_equip", 1, 1)
    );
}

#[test]
fn only_a_highest_hp_target_can_be_damaged_and_the_cost_is_discarded() {
    let (state, owner, highest, lower, skill) = started();
    let cost = state.hand_cards(owner)[0];
    let before = canonical_state_hash(&state);
    let (same, rejected) = activate(state.clone(), owner, lower, skill, cost, 1);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let hp = state.players.get(highest).unwrap().hp;
    let (state, accepted) = activate(state, owner, highest, skill, cost, 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(highest).unwrap().hp, hp - 1);
    assert_eq!(
        state.cards.get(cost).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
    assert_eq!(state.skills.get(skill).unwrap().use_count, 1);
}
