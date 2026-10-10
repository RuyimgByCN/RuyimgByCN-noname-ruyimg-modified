//! Golden tests for SP2「凶算」rewritten from the legacy JavaScript semantics.

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XIONGSUAN: u32 = 3_088_500_344;
const LIQUEGUOSI: u32 = 2_859_668_029;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn started_state(seed: u64) -> (GameState, PlayerId, PlayerId, SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
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
    for player in [owner, other] {
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.hp = 10;
        state.max_hp = 10;
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(XIONGSUAN),
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
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == XIONGSUAN)
        .unwrap();
    (tx.state().clone(), owner, other, skill)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    target: PlayerId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let card = state.hand_cards(owner)[0];
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
fn package_contains_complete_character_and_active_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == LIQUEGUOSI)
        .expect("李傕郭汜 character entry");
    assert_eq!(character.skills, vec![XIONGSUAN]);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == XIONGSUAN)
        .expect("凶算 active entry");
    assert_eq!(active.uses_per_turn, Some(1));
    assert_eq!(
        active
            .targets
            .as_ref()
            .map(|targets| (targets.min, targets.max)),
        Some((1, 1))
    );
    assert_eq!(
        active.selection.as_ref().map(|selection| (
            selection.zone.as_str(),
            selection.min,
            selection.max
        )),
        Some(("hand", 1, 1))
    );
}

#[test]
fn discards_damages_draws_and_loses_hp_only_for_another_target() {
    let (state, owner, other, skill) = started_state(2_026_071_505);
    let owner_hand = state.hand_count(owner);
    let owner_hp = state.players.get(owner).unwrap().hp;
    let target_hp = state.players.get(other).unwrap().hp;
    let discarded = state.hand_cards(owner)[0];

    let (state, pipe) = activate(state, owner, skill, other, 1);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), owner_hand + 2);
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);
    assert_eq!(state.players.get(other).unwrap().hp, target_hp - 1);
    assert_eq!(
        state.cards.get(discarded).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
    assert!(pipe.events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        } if *source == owner && *target == other
    )));
    assert!(pipe.events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            target,
            amount: 1,
            is_lose_hp: true,
            ..
        } if *target == owner
    )));

    let before_retry = canonical_state_hash(&state);
    let (rejected, pipe) = activate(state, owner, skill, other, 2);
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&rejected), before_retry);
}

#[test]
fn self_target_takes_damage_without_extra_hp_loss() {
    let (state, owner, _other, skill) = started_state(2_026_071_506);
    let hp = state.players.get(owner).unwrap().hp;
    let (state, pipe) = activate(state, owner, skill, owner, 1);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);
    assert!(!pipe.events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            target,
            is_lose_hp: true,
            ..
        } if *target == owner
    )));
}

#[test]
fn missing_or_dead_target_is_rejected_without_mutation() {
    let (state, owner, other, skill) = started_state(2_026_071_507);
    let card = state.hand_cards(owner)[0];
    let before = canonical_state_hash(&state);
    let (same, pipe) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![card],
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let mut dead = same;
    dead.players.get_mut(other).unwrap().alive = false;
    let before = canonical_state_hash(&dead);
    let (same, pipe) = activate(dead, owner, skill, other, 2);
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
