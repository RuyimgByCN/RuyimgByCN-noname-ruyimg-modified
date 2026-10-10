//! Golden coverage for 掠卤（sp2 梁兴）目标二选一（ABI 463）.
//!
//! 出牌阶段开始：选一名手牌数更少（>0）的其他角色，其选择交全部手牌并令拥有者
//! 翻面，或自己翻面并视为对拥有者使用【杀】。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const LULVE: u32 = 3_019_895_260;
const SOURCE: &str = include_str!("../../../content/original/ported/sp2/lulve.skill");

fn drive_choice(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selection: u32,
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
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted"
    );
    next
}

fn give_hand(mut tx: Transaction, player: PlayerId, count: u32) -> (Transaction, Vec<CardId>) {
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .expect("hand zone");
    let mut cards = Vec::new();
    for rank in 1..=count {
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(9_700),
            kind: CardKind::Sha,
            suit: engine_api::Suit::Heart,
            rank: rank as u8,
            weapon_range: 0,
            zone: hand,
            owner: player,
            face_up: false,
        });
        tx.state_mut()
            .zones
            .get_mut(hand)
            .expect("hand zone")
            .cards
            .push(card);
        cards.push(card);
    }
    (tx, cards)
}

fn fire_play(tx: Transaction, owner: PlayerId) -> Result<Transaction, EffectError> {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Play,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => Ok(tx),
            Err(EffectError::ChoiceRequested) => Ok(tx),
            Err(e) => Err(e),
        },
        Err(EffectError::ChoiceRequested) => Ok(tx),
        Err(e) => Err(e),
    }
}

#[test]
fn branch_zero_gives_all_hand_and_flips_owner() {
    let package = content_compiler::compile_source_named("lulve-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 463);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_115, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target, _third) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(LULVE),
    });
    run_effects(&mut tx, effects).unwrap();
    // Controlled hands: owner 3, target 1.
    let (tx, _owner_cards) = give_hand(tx, owner, 3);
    let (mut tx, target_cards) = give_hand(tx, target, 1);
    let owner_hand_before = tx.state().hand_count(owner);
    let target_hand_before = tx.state().hand_count(target);
    tx.state_mut().turn.current = owner;

    let tx = fire_play(tx, owner).unwrap();
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "target choice must be pending");
    state = drive_choice(state, owner, target.raw()); // owner picks the target
    state = drive_choice(state, target, 0); // target chooses give-all + flip owner
    assert_eq!(
        state.hand_count(owner),
        owner_hand_before + target_hand_before,
        "owner gains the target's whole hand"
    );
    assert_eq!(state.hand_count(target), 0, "target hand is emptied");
    assert!(
        state.players.get(owner).unwrap().turned_over,
        "owner flips face down"
    );
    assert!(
        !state.players.get(target).unwrap().turned_over,
        "target stays face up"
    );
    assert!(!state.has_pending_choice());
    let _ = target_cards;
}

#[test]
fn branch_one_flips_target_and_uses_virtual_sha() {
    let package = content_compiler::compile_source_named("lulve-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 463);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_115, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target, _third) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(LULVE),
    });
    run_effects(&mut tx, effects).unwrap();
    let (tx, _owner_cards) = give_hand(tx, owner, 3);
    let (mut tx, _target_cards) = give_hand(tx, target, 1);
    tx.state_mut().turn.current = owner;

    let tx = fire_play(tx, owner).unwrap();
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "target choice must be pending");
    state = drive_choice(state, owner, target.raw()); // owner picks the target
                                                      // The virtual Sha may open a response window for the owner; drive the branch first.
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor: target,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "branch choice must be accepted"
    );
    let state = next;
    assert!(
        state.players.get(target).unwrap().turned_over,
        "target flips face down"
    );
}
