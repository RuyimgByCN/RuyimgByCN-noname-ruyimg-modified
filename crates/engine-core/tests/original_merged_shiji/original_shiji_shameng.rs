//! Golden tests for SHIJI「歃盟」rewritten from the legacy JavaScript semantics.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
    SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SHAMENG: u32 = 3_467_425_391;
const CHENZHEN: u32 = 4_187_828_074;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/shiji/shiji.npack.json");

fn started_state(seed: u64) -> (GameState, PlayerId, PlayerId, SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    tx.state_mut().revision = 1;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    tx.state_mut().turn.phase = Phase::Play;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SHAMENG),
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
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == SHAMENG)
        .unwrap();
    (tx.state().clone(), owner, other, skill)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    target: PlayerId,
    cards: Vec<engine_api::CardId>,
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
                card_targets: cards,
            },
        },
    )
}

#[test]
fn package_contains_complete_character_and_constraints() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == CHENZHEN)
        .expect("陈震 character entry");
    assert_eq!(character.skills, vec![SHAMENG]);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == SHAMENG)
        .expect("歃盟 active entry");
    assert_eq!(active.uses_per_turn, Some(1));
    assert_eq!(
        active
            .targets
            .as_ref()
            .map(|targets| (targets.min, targets.max, targets.allow_self)),
        Some((1, 1, false))
    );
    assert_eq!(
        active.selection.as_ref().map(|selection| (
            selection.min,
            selection.max,
            selection.same_color
        )),
        Some((2, 2, true))
    );
}

#[test]
fn same_color_cost_then_target_draws_two_and_owner_draws_three() {
    let (mut state, owner, other, skill) = started_state(2_026_071_508);
    let cards = state.hand_cards(owner)[..2].to_vec();
    state.cards.get_mut(cards[0]).unwrap().suit = Suit::Heart;
    state.cards.get_mut(cards[1]).unwrap().suit = Suit::Diamond;
    let owner_hand = state.hand_count(owner);
    let target_hand = state.hand_count(other);

    let (state, pipe) = activate(state, owner, skill, other, cards.clone(), 1);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), owner_hand + 1);
    assert_eq!(state.hand_count(other), target_hand + 2);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(cards
        .iter()
        .all(|card| state.cards.get(*card).unwrap().zone == discard));

    let retry_cards = state.hand_cards(owner)[..2].to_vec();
    let before = canonical_state_hash(&state);
    let (same, pipe) = activate(state, owner, skill, other, retry_cards, 2);
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}

#[test]
fn mixed_colors_and_self_target_are_rejected_without_mutation() {
    let (mut state, owner, other, skill) = started_state(2_026_071_509);
    let cards = state.hand_cards(owner)[..2].to_vec();
    state.cards.get_mut(cards[0]).unwrap().suit = Suit::Heart;
    state.cards.get_mut(cards[1]).unwrap().suit = Suit::Spade;
    let before = canonical_state_hash(&state);
    let (same, pipe) = activate(state, owner, skill, other, cards.clone(), 1);
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let mut state = same;
    state.cards.get_mut(cards[1]).unwrap().suit = Suit::Diamond;
    let before = canonical_state_hash(&state);
    let (same, pipe) = activate(state, owner, skill, owner, cards, 2);
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
