//! Golden coverage for 封印（collab SP伏完，ABI 471）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag, ZoneId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const SPFENGYIN: u32 = 3_552_373_198;
const SOURCE: &str = include_str!("../../../content/original/ported/collab/spfengyin.skill");

fn drive_choice(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
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
                cancelled,
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

fn fire_prepare(tx: Transaction, current: PlayerId) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => tx,
            Err(engine_core::effect::EffectError::ChoiceRequested) => tx,
            Err(e) => panic!("effect error: {e:?}"),
        },
        Err(engine_core::effect::EffectError::ChoiceRequested) => tx,
        Err(e) => panic!("dispatch error: {e:?}"),
    }
}

fn empty_hand(tx: &mut Transaction, player: PlayerId) {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let cards: Vec<CardId> = tx.state().zones.get(hand).unwrap().cards.clone();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        tx,
        cards
            .into_iter()
            .map(|card| EffectRequest::MoveCards {
                cards: vec![card],
                to: discard,
                from: Some(hand),
            })
            .collect(),
    )
    .unwrap();
}

/// Seed a Sha into the player's hand; returns its id.
fn seed_sha(tx: &mut Transaction, player: PlayerId) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_000),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn gives_sha_and_skips_play_and_discard_phases() {
    let package = content_compiler::compile_source_named("spfengyin-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 471);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SPFENGYIN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 4;
    tx.state_mut().players.get_mut(target).unwrap().hp = 4;
    empty_hand(&mut tx, owner);
    empty_hand(&mut tx, target);
    // Owner holds one Sha; the target's hp (4) >= owner's (4).
    let sha = seed_sha(&mut tx, owner);

    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.current = target;
    let tx = fire_prepare(tx, target);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "Sha give choice pending");
    state = drive_choice(state, owner, vec![sha.raw()], false);

    assert_eq!(state.hand_count(owner), 0, "Sha given away");
    assert_eq!(state.hand_count(target), 1, "target received the Sha");
    assert_eq!(
        state.turn.flag(engine_core::state::turn_flag::SKIP_PLAY),
        1,
        "play phase skipped"
    );
    assert_eq!(
        state.turn.flag(engine_core::state::turn_flag::SKIP_DISCARD),
        1,
        "discard phase skipped"
    );
}

#[test]
fn cancel_keeps_phases() {
    let package = content_compiler::compile_source_named("spfengyin-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SPFENGYIN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 4;
    tx.state_mut().players.get_mut(target).unwrap().hp = 4;
    empty_hand(&mut tx, owner);
    empty_hand(&mut tx, target);
    let sha = seed_sha(&mut tx, owner);

    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.current = target;
    let tx = fire_prepare(tx, target);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice());
    state = drive_choice(state, owner, vec![], true);
    assert_eq!(state.hand_count(owner), 1, "Sha kept");
    assert_eq!(state.hand_count(target), 0);
    assert_eq!(
        state.turn.flag(engine_core::state::turn_flag::SKIP_PLAY),
        0,
        "play phase not skipped"
    );
    assert_eq!(
        state.turn.flag(engine_core::state::turn_flag::SKIP_DISCARD),
        0,
        "discard phase not skipped"
    );
}

#[test]
fn no_choice_when_target_hp_below_owner() {
    let package = content_compiler::compile_source_named("spfengyin-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SPFENGYIN),
    });
    run_effects(&mut tx, effects).unwrap();
    empty_hand(&mut tx, owner);
    empty_hand(&mut tx, target);
    seed_sha(&mut tx, owner);
    // Make the owner tankier: target hp < owner hp blocks the seal.
    let owner_max = tx.state().players.get(owner).unwrap().max_hp;
    tx.state_mut().players.get_mut(owner).unwrap().hp = owner_max as i32;
    tx.state_mut().players.get_mut(target).unwrap().hp = 1;

    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.current = target;
    let tx = fire_prepare(tx, target);
    let state = tx.commit().state;
    assert!(
        !state.has_pending_choice(),
        "no choice when the target's hp is below the owner's"
    );
}
