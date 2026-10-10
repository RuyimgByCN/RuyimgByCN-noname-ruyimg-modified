//! Golden coverage for 单骑 + 怒嗔（sp2 新杀SP关羽，ABI 466）.

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

const DCDANJI: u32 = 3_851_102_612;
const DCNUCHEN: u32 = 2_960_451_206;
const MASHU: u32 = 2;
const SOURCE: &str = include_str!("../../../content/original/ported/sp2/dcdanji.skill");

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
    eprintln!(
        "fire_phase({tag:?}): dispatch = {:?}",
        fired.as_ref().map(|v| (
            v.len(),
            v.iter()
                .take(8)
                .map(|e| format!("{e:?}"))
                .collect::<Vec<_>>()
        ))
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

fn setup(tx: &mut Transaction, owner: PlayerId, target: PlayerId) {
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DCNUCHEN),
    });
    run_effects(tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    // Deterministic hands: move everything to the discard pile via the engine (keeps zone
    // pointers consistent), then seed exactly. Target: Spade 5. Owner: Spade 7 + Heart 3.
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for (zone_kind, player) in [(ZoneKind::Hand, owner), (ZoneKind::Hand, target)] {
        let zone_id = tx.state().zones.find(zone_kind, player).unwrap();
        let cards: Vec<CardId> = tx.state().zones.get(zone_id).unwrap().cards.clone();
        let moves = cards
            .into_iter()
            .map(|card| EffectRequest::MoveCards {
                cards: vec![card],
                to: discard,
                from: Some(zone_id),
            })
            .collect();
        run_effects(tx, moves).unwrap();
    }
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let target_hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    let mut alloc = |def: u32, kind: CardKind, suit: Suit, rank: u8, hand: engine_api::ZoneId| {
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(def),
            kind,
            suit,
            rank,
            weapon_range: 0,
            zone: hand,
            owner,
            face_up: false,
        });
        tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
        card
    };
    alloc(96_000, CardKind::Sha, Suit::Spade, 5, target_hand);
    alloc(96_001, CardKind::Sha, Suit::Spade, 7, owner_hand);
    alloc(96_002, CardKind::Tao, Suit::Heart, 3, owner_hand);
}

#[test]
fn awakening_at_prepare_when_hej_above_hp() {
    let package = content_compiler::compile_source_named("dcdanji-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 466);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCDANJI),
        }],
    )
    .unwrap();
    // hej > hp: set hp below maxHp to also exercise recover+draw.
    let max_hp_before = tx.state().players.get(owner).unwrap().max_hp;
    tx.state_mut().players.get_mut(owner).unwrap().hp = max_hp_before as i32 - 2;
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let extra = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_003),
        kind: CardKind::Sha,
        suit: Suit::Club,
        rank: 9,
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
        .push(extra);
    let hand_before = tx.state().hand_count(owner);

    let expected_num = 1; // (maxHp-1) - hp after the loss, with hp = maxHp-2
    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.current = owner;
    let tx = fire_phase(tx, owner, TurnPhaseTag::Prepare);
    let state = tx.commit().state;
    let owner_state = state.players.get(owner).unwrap();
    assert_eq!(owner_state.max_hp, max_hp_before - 1, "maxHp reduced by 1");
    assert_eq!(
        owner_state.hp,
        max_hp_before as i32 - 1,
        "recovered to the new maxHp"
    );
    assert_eq!(
        state.hand_count(owner),
        hand_before + expected_num,
        "drew maxHp-hp after recovery"
    );
    let owned: Vec<u32> = state
        .skills
        .sorted_live()
        .into_iter()
        .filter(|skill| skill.owner == owner)
        .map(|skill| skill.def.raw())
        .collect();
    assert!(owned.contains(&MASHU), "gained mashu: {owned:?}");
    assert!(owned.contains(&DCNUCHEN), "gained dcnuchen: {owned:?}");
    assert!(
        !owned.contains(&DCDANJI),
        "awakened skill removed: {owned:?}"
    );
}

#[test]
fn dcnuchen_discard_same_suit_damages() {
    let package = content_compiler::compile_source_named("dcnuchen-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    let target_hp_before = tx.state().players.get(target).unwrap().hp;
    let owner_hand_before = tx.state().hand_count(owner);

    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_phase(tx, owner, TurnPhaseTag::Play);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "target choice pending");
    state = drive_choice(state, owner, vec![target.raw()], false);
    assert!(state.has_pending_choice(), "hand-card choice pending");
    let target_hand = state.hand_cards(target);
    let revealed = *target_hand
        .iter()
        .find(|c| state.cards.get(**c).unwrap().suit == Suit::Spade)
        .unwrap();
    state = drive_choice(state, owner, vec![revealed.raw()], false);
    assert!(
        state.has_pending_choice(),
        "same-suit discard choice pending"
    );
    let owner_hand: Vec<u32> = state
        .hand_cards(owner)
        .into_iter()
        .filter(|c| state.cards.get(*c).unwrap().suit == Suit::Spade)
        .map(|c| c.raw())
        .collect();
    assert_eq!(owner_hand.len(), 1, "only the Spade is offered");
    state = drive_choice(state, owner, owner_hand.clone(), false);

    let target_hp_after = state.players.get(target).unwrap().hp;
    assert_eq!(
        target_hp_after,
        target_hp_before - 1,
        "1 damage for 1 discarded"
    );
    assert_eq!(
        state.hand_count(owner),
        owner_hand_before - 1,
        "discarded one card"
    );
    assert!(
        !state
            .hand_cards(owner)
            .iter()
            .any(|c| c.raw() == owner_hand[0]),
        "discarded Spade left the hand"
    );
}

#[test]
fn dcnuchen_cancel_gains_target_same_suit_hand() {
    let package = content_compiler::compile_source_named("dcnuchen-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    let target_spade: Vec<u32> = tx
        .state()
        .hand_cards(target)
        .into_iter()
        .filter(|c| tx.state().cards.get(*c).unwrap().suit == Suit::Spade)
        .map(|c| c.raw())
        .collect();
    assert!(
        target_spade.len() >= 1,
        "seeded Spade present (bootstrap may add more Spades)"
    );
    let owner_before = tx.state().hand_count(owner);

    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_phase(tx, owner, TurnPhaseTag::Play);
    let mut state = tx.commit().state;
    state = drive_choice(state, owner, vec![target.raw()], false);
    let revealed = *state
        .hand_cards(target)
        .iter()
        .find(|c| state.cards.get(**c).unwrap().suit == Suit::Spade)
        .unwrap();
    state = drive_choice(state, owner, vec![revealed.raw()], false);
    assert!(state.has_pending_choice());
    state = drive_choice(state, owner, vec![], true);

    assert_eq!(
        state.hand_count(owner),
        owner_before + 1,
        "gained the Spade"
    );
    assert!(state
        .hand_cards(owner)
        .iter()
        .any(|c| c.raw() == target_spade[0]));
    let target_kept = state.hand_count(target);
    assert_eq!(
        target_kept, 0,
        "target lost its only hand card: {target_kept}"
    );
}
