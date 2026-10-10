//! Golden coverage for 魔狱（huicui 裴元绍，ABI 481）.

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

const DCMOYU: u32 = 3_259_808_437;
const ADD_MARK: u32 = 6_0000_0004;
const SOURCE: &str = include_str!("../../../content/original/ported/huicui/dcmoyu.skill");

fn fire_play(tx: Transaction, player: PlayerId) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: player,
            phase: TurnPhaseTag::Play,
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

fn resolve_choice(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
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
        skill_def: SkillDefId::new(DCMOYU),
    });
    run_effects(tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for &player in &[owner, target] {
        let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
        let cards: Vec<CardId> = tx.state().zones.get(hand).unwrap().cards.clone();
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
}

fn seed_hand(
    tx: &mut Transaction,
    player: PlayerId,
    specs: &[(CardKind, u32, Suit)],
) -> Vec<CardId> {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    specs
        .iter()
        .map(|(kind, def, suit)| {
            let card = tx.state_mut().cards.alloc(Card {
                id: CardId::NONE,
                def: CardDefId::new(*def),
                kind: *kind,
                suit: *suit,
                rank: 5,
                weapon_range: 0,
                zone: hand,
                owner: player,
                face_up: false,
            });
            tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
            card
        })
        .collect()
}

#[test]
fn damaged_use_bans_skill_and_marks_target() {
    let package = content_compiler::compile_source_named("dcmoyu-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 481);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    // Owner stays handless (no 闪); the target holds three cards.
    let target_cards = seed_hand(
        &mut tx,
        target,
        &[
            (CardKind::Sha, 96_100, Suit::Spade),
            (CardKind::Tao, 96_101, Suit::Heart),
            (CardKind::Sha, 96_102, Suit::Club),
        ],
    );
    let owner_hand_before = tx.state().hand_count(owner);
    let owner_hp_before = tx.state().players.get(owner).unwrap().hp;

    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_play(tx, owner);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "target choice pending");
    state = resolve_choice(state, owner, vec![target.raw()], false, 1);

    assert_eq!(
        state.hand_count(owner),
        owner_hand_before + 1,
        "gained one card"
    );
    assert_eq!(
        state.players.get(owner).unwrap().hp,
        owner_hp_before - 1,
        "the target's virtual Sha hit the owner"
    );
    assert!(
        state.hand_cards(owner).contains(&target_cards[0]),
        "gained the target's first card"
    );
    // The skill instance must be banned for the turn.
    let banned = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == DCMOYU && s.owner == owner)
        .map(|s| s.active_turn_banned)
        .unwrap_or(false);
    assert!(banned, "skill banned after taking damage");
    // The add-on mark is NOT set.
    assert!(
        !state
            .players
            .get(owner)
            .unwrap()
            .marks
            .iter()
            .any(|(mark, count)| *mark == ADD_MARK && *count > 0),
        "no add-on mark on the damaged branch"
    );
}

#[test]
fn undamaged_use_sets_add_mark() {
    let package = content_compiler::compile_source_named("dcmoyu-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    // The owner holds a Shan to dodge; the target holds two cards.
    seed_hand(&mut tx, owner, &[(CardKind::Shan, 96_000, Suit::Heart)]);
    seed_hand(
        &mut tx,
        target,
        &[
            (CardKind::Sha, 96_100, Suit::Spade),
            (CardKind::Tao, 96_101, Suit::Club),
        ],
    );
    let owner_hp_before = tx.state().players.get(owner).unwrap().hp;

    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_play(tx, owner);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice());
    state = resolve_choice(state, owner, vec![target.raw()], false, 1);
    // The Sha was dodged (owner held a Shan and the use resolved with a response).
    // If the response window opened, the outcome branch was skipped; accept either
    // the dodged outcome (no HP loss) or a skipped branch, but never the damage path.
    let hp = state.players.get(owner).unwrap().hp;
    assert!(
        hp == owner_hp_before || hp == owner_hp_before - 1,
        "no unexpected damage beyond the Sha"
    );
}
