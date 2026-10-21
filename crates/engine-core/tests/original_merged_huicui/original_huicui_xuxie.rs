//! Golden coverage for 虚泄（huicui 邢道荣，ABI 483）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag, ZoneId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const XUXIE: u32 = 3_207_964_441;
const SOURCE: &str = include_str!("../../../../content/original/ported/huicui/xuxie.skill");

fn fire_phase(tx: Transaction, player: PlayerId, tag: TurnPhaseTag) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: player,
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

fn setup(tx: &mut Transaction, owner: PlayerId) {
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XUXIE),
    });
    run_effects(tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    kind: CardKind,
    def: u32,
    suit: Suit,
    zone: ZoneId,
) -> CardId {
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 5,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn discard_branch_and_max_hp_loss_and_restore() {
    let package = content_compiler::compile_source_named("xuxie-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 483);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, adjacent) = (seats[0], seats[1]);
    setup(&mut tx, owner);
    // Deterministic: empty both hands, give the adjacent player two cards.
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for &player in &[owner, adjacent] {
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
    let adjacent_hand = tx.state().zones.find(ZoneKind::Hand, adjacent).unwrap();
    let c1 = alloc_card(
        &mut tx,
        adjacent,
        CardKind::Sha,
        96_100,
        Suit::Spade,
        adjacent_hand,
    );
    let c2 = alloc_card(
        &mut tx,
        adjacent,
        CardKind::Tao,
        96_101,
        Suit::Heart,
        adjacent_hand,
    );
    let max_hp_before = tx.state().players.get(owner).unwrap().max_hp;

    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.phase = Phase::Play;
    let tx = fire_phase(tx, owner, TurnPhaseTag::Play);
    let mut state = tx.commit().state;
    // The skill lost 1 maxHp first, then the branch choice opens.
    assert_eq!(
        state.players.get(owner).unwrap().max_hp,
        max_hp_before - 1,
        "lost 1 maxHp"
    );
    assert!(state.has_pending_choice(), "branch choice pending");
    // Discard branch: marker -> card choice -> marker.
    state = resolve_choice(state, owner, vec![0], 1);
    assert!(state.has_pending_choice(), "discard card choice pending");
    state = resolve_choice(state, owner, vec![c1.raw()], 2);
    assert!(state.has_pending_choice(), "loop marker choice pending");
    state = resolve_choice(state, owner, vec![0], 3);

    assert_eq!(state.hand_count(adjacent), 1, "one card discarded");
    assert!(
        state.hand_cards(adjacent).contains(&c2),
        "the other card remains"
    );
    // The play-phase end: the adjacent player's maxHp (4) equals the owner's after the
    // loss (5-1=4), so no restore happens.
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: owner,
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
            Ok(_) => {}
            Err(engine_core::effect::EffectError::ChoiceRequested) => {}
            Err(e) => panic!("effect error: {e:?}"),
        },
        Err(engine_core::effect::EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    let state = tx.commit().state;
    assert_eq!(
        state.players.get(owner).unwrap().max_hp,
        max_hp_before - 1,
        "maxHp unchanged (no one bigger)"
    );
}

#[test]
fn draw_branch_draws_for_adjacent() {
    let package = content_compiler::compile_source_named("xuxie-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, adjacent) = (seats[0], seats[1]);
    setup(&mut tx, owner);
    let adjacent_before = tx.state().hand_count(adjacent);

    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.phase = Phase::Play;
    let tx = fire_phase(tx, owner, TurnPhaseTag::Play);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice());
    state = resolve_choice(state, owner, vec![1], 1); // draw branch
    assert_eq!(
        state.hand_count(adjacent),
        adjacent_before + 1,
        "adjacent player drew one"
    );
}
