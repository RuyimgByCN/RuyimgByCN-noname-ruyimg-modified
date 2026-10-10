//! Golden coverage for 相术（huicui 张闿，ABI 486）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCXIANG_SHU: u32 = 3_579_538_404;
const SOURCE: &str = include_str!("../../../content/original/ported/huicui/dcxiangshu.skill");

fn fire_phase(tx: Transaction, current: PlayerId, tag: TurnPhaseTag) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current,
            phase: tag,
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

fn fire_play_end(tx: Transaction, current: PlayerId) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current,
            phase: TurnPhaseTag::Play,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_hp_changed_players: 0,
            phase_targeted_players: 0,
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

fn resolve_choice(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
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

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    kind: CardKind,
    def: u32,
    suit: Suit,
) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn exact_match_gains_card_and_deals_damage() {
    let package = content_compiler::compile_source_named("dcxiangshu-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 486);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, observed) = (seats[0], seats[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DCXIANG_SHU),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = observed;
    // Deterministic hands: empty both, give the observed player exactly 2 cards.
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for &player in &[owner, observed] {
        let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
        let cards: Vec<CardId> = tx.state().zones.get(hand).unwrap().cards.clone();
        run_effects(
            &mut tx,
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
    alloc_card(&mut tx, observed, CardKind::Sha, 96_100, Suit::Spade);
    let c2 = alloc_card(&mut tx, observed, CardKind::Tao, 96_101, Suit::Heart);
    // The observed player's hp must be <= 2 (hand 2 >= hp).
    tx.state_mut().players.get_mut(observed).unwrap().hp = 2;
    let observed_hp_before = tx.state().players.get(observed).unwrap().hp;

    // The observed player's play phase begins.
    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.phase = Phase::Play;
    let tx = fire_phase(tx, observed, TurnPhaseTag::Play);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "number choice pending");
    // Guess exactly 2 (matches the observed player's hand count).
    state = resolve_choice(state, owner, vec![2], 1);

    // Play phase ends with the hand still at 2: delta 0 -> gain one card + damage 1.
    let mut tx = Transaction::begin(state);
    let tx = fire_play_end(tx, observed);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "gain choice pending");
    state = resolve_choice(state, owner, vec![c2.raw()], 2);

    assert_eq!(
        state.players.get(observed).unwrap().hp,
        observed_hp_before - 2 + 1,
        "the observed player took 1 damage from the exact match"
    );
    assert!(
        state.hand_cards(owner).contains(&c2),
        "the owner gained a card from the observed player"
    );
}
