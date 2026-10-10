//! Golden coverage for 劫刀（sp2 忙牙长，ABI 469）.

use engine_api::{
    CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SPJIEDAO: u32 = 4_242_283_768;
const SOURCE: &str = include_str!("../../../content/original/ported/sp2/spjiedao.skill");

fn drive_choice(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selections: Vec<u32>,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted: {:?}",
        result.result
    );
    next
}

fn commit_damage(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
) -> Result<(), engine_core::effect::EffectError> {
    match run_effects(
        tx,
        vec![EffectRequest::CommitDamage {
            damage_id: 0,
            source,
            target,
            amount,
            base_amount: amount,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }],
    ) {
        Ok(_) => Ok(()),
        Err(engine_core::effect::EffectError::ChoiceRequested) => Ok(()),
        Err(e) => Err(e),
    }
}

#[test]
fn boosts_damage_and_discards_chosen_cards() {
    let package = content_compiler::compile_source_named("spjiedao-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 469);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, victim) = (seats[0], seats[1]);
    // Strip skills; give the owner 劫刀 and wound them (maxHp - 2 → 2 lost HP).
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SPJIEDAO),
    });
    run_effects(&mut tx, effects).unwrap();
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;
    tx.state_mut().players.get_mut(owner).unwrap().hp = max_hp as i32 - 2;
    // Deterministic discardable hand: empty the owner's hand, seed exactly two cards.
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let cards: Vec<CardId> = tx.state().zones.get(owner_hand).unwrap().cards.clone();
    run_effects(
        &mut tx,
        cards
            .into_iter()
            .map(|card| EffectRequest::MoveCards {
                cards: vec![card],
                to: discard,
                from: Some(owner_hand),
            })
            .collect(),
    )
    .unwrap();
    let mut seeded = Vec::new();
    for def in [96_000u32, 96_001] {
        let card = tx.state_mut().cards.alloc(engine_core::state::Card {
            id: CardId::NONE,
            def: engine_api::CardDefId::new(def),
            kind: engine_core::state::CardKind::Sha,
            suit: engine_api::Suit::Spade,
            rank: 5,
            weapon_range: 0,
            zone: owner_hand,
            owner,
            face_up: false,
        });
        tx.state_mut()
            .zones
            .get_mut(owner_hand)
            .unwrap()
            .cards
            .push(card);
        seeded.push(card);
    }
    let victim_hp_before = tx.state().players.get(victim).unwrap().hp;

    let mut tx = Transaction::begin(tx.commit().state);
    commit_damage(&mut tx, owner, victim, 1).unwrap();
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "optional confirm pending");
    state = drive_choice(state, owner, vec![1]); // confirm 劫刀
    assert!(state.has_pending_choice(), "boost amount choice pending");
    // Choose X = 2 (both lost HP), then discard exactly 2 he cards.
    state = drive_choice(state, owner, vec![2]);
    assert!(state.has_pending_choice(), "discard choice pending");
    let discardable: Vec<u32> = state
        .hand_cards(owner)
        .into_iter()
        .map(|id| id.raw())
        .collect();
    assert_eq!(discardable.len(), 2, "exactly the seeded cards");
    state = drive_choice(state, owner, discardable);

    assert_eq!(
        state.players.get(victim).unwrap().hp,
        victim_hp_before - 3,
        "1 base + 2 boost = 3 damage"
    );
    assert_eq!(state.hand_count(owner), 0, "both cards discarded");
}

#[test]
fn cancel_leaves_damage_unboosted() {
    let package = content_compiler::compile_source_named("spjiedao-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, victim) = (seats[0], seats[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SPJIEDAO),
    });
    run_effects(&mut tx, effects).unwrap();
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;
    tx.state_mut().players.get_mut(owner).unwrap().hp = max_hp as i32 - 2;
    let victim_hp_before = tx.state().players.get(victim).unwrap().hp;

    let mut tx = Transaction::begin(tx.commit().state);
    commit_damage(&mut tx, owner, victim, 1).unwrap();
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice());
    state = drive_choice(state, owner, vec![1]); // confirm 劫刀
    assert!(state.has_pending_choice(), "boost amount choice pending");
    let hand_before = state.hand_count(owner);
    // Cancel the boost: damage stays 1.
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![],
                cancelled: true,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state = next;
    assert_eq!(
        state.players.get(victim).unwrap().hp,
        victim_hp_before - 1,
        "unboosted damage"
    );
    assert_eq!(state.hand_count(owner), hand_before, "nothing discarded");
}

#[test]
fn second_damage_of_turn_not_boostable() {
    let package = content_compiler::compile_source_named("spjiedao-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, victim) = (seats[0], seats[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SPJIEDAO),
    });
    run_effects(&mut tx, effects).unwrap();
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;
    tx.state_mut().players.get_mut(owner).unwrap().hp = max_hp as i32 - 2;
    let victim_hp_before = tx.state().players.get(victim).unwrap().hp;

    let mut tx = Transaction::begin(tx.commit().state);
    // First damage: confirm then cancel the boost.
    commit_damage(&mut tx, owner, victim, 1).unwrap();
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice());
    state = drive_choice(state, owner, vec![1]); // confirm 劫刀
    assert!(state.has_pending_choice(), "boost amount choice pending");
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![],
                cancelled: true,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state = next;
    assert_eq!(state.players.get(victim).unwrap().hp, victim_hp_before - 1);

    // Second damage of the turn: the confirm still opens (optional), but the composite
    // sees the owner already dealt damage this turn and does not offer a boost.
    let mut tx = Transaction::begin(state);
    commit_damage(&mut tx, owner, victim, 1).unwrap();
    let state = tx.commit().state;
    assert!(state.has_pending_choice(), "optional confirm opens again");
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(4),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let state = next;
    assert!(
        !state.has_pending_choice(),
        "second damage of the turn is not boostable"
    );
    assert_eq!(state.players.get(victim).unwrap().hp, victim_hp_before - 2);
}
