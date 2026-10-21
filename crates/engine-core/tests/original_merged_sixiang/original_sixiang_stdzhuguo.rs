//! Golden coverage for SIXIANG「助国」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const STDZHUGUO: u32 = 3_147_233_152;
const SOURCE: &str = include_str!("../../../../content/original/ported/sixiang/stdzhuguo.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/sixiang/sixiang.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId, SkillId) {
    let package = content_compiler::compile_source_named("stdzhuguo-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_724, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(
        players
            .into_iter()
            .map(|player| EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }),
    );
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(STDZHUGUO),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == STDZHUGUO)
        .unwrap();
    tx.state_mut().revision = 1;
    (tx.commit().state, owner, target, skill)
}

fn inject_hand(state: &mut GameState, owner: PlayerId, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    )
}

#[test]
fn package_publishes_abi128_literal_hand_target_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == STDZHUGUO)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    let targets = active.targets.as_ref().unwrap();
    assert_eq!(
        (
            targets.min,
            targets.max,
            targets.allow_self,
            targets.hand_lt
        ),
        (1, 1, true, Some(2))
    );
}

#[test]
fn target_with_zero_or_one_card_draws_exactly_to_two() {
    let (state, owner, target, skill) = setup();
    let (resolved, result) = activate(state, owner, target, skill, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(resolved.hand_count(target), 2);
    assert!(!resolved.has_pending_choice());

    let (mut state, owner, target, skill) = setup();
    inject_hand(&mut state, target, 128_001);
    let (resolved, result) = activate(state, owner, target, skill, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(resolved.hand_count(target), 2);
}

#[test]
fn self_is_legal_but_two_card_target_and_second_use_are_rejected_atomically() {
    let (state, owner, _, skill) = setup();
    let (state, result) = activate(state, owner, owner, skill, 10);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), 2);
    let before_retry = canonical_state_hash(&state);
    let (same, result) = activate(state, owner, owner, skill, 11);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before_retry);

    let (mut state, owner, target, skill) = setup();
    inject_hand(&mut state, target, 128_010);
    inject_hand(&mut state, target, 128_011);
    let before = canonical_state_hash(&state);
    let (same, result) = activate(state, owner, target, skill, 12);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
