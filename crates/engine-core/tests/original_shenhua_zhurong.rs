//! Golden coverage for 巨象/烈刃（shenhua 祝融，ABI 530）.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const JUXIANG: u32 = 4_234_952_102;
const LIEREN: u32 = 3_941_156_292;
const LIEREN_WIN: u32 = 1_258_122_096;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/zhurong.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    source: PlayerId,
    target: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("zhurong-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let source = seats[1];
    let target = seats[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JUXIANG),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(LIEREN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup {
        tx,
        owner,
        source,
        target,
    }
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    kind: CardKind,
    def: u32,
    suit: Suit,
    rank: u8,
) -> CardId {
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn fire_used(tx: Transaction, user: PlayerId, card: CardId, targets: Vec<PlayerId>) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: user,
            card,
            targets,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => tx,
            Err(EffectError::ChoiceRequested) => tx,
            Err(e) => panic!("effect error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => tx,
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

#[test]
fn package_exposes_juxiang_lieren_contract() {
    let compiled = content_compiler::compile_source_named("zhurong", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 531);
    let juxiang = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == JUXIANG)
        .unwrap();
    assert_eq!(juxiang.event, "card_resolved");
    assert_eq!(juxiang.filter, "owner_alive_other_nanman_cards_discard");
    let lieren = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == LIEREN)
        .unwrap();
    assert_eq!(lieren.event, "damage");
    assert_eq!(
        lieren.filter,
        "owner_source_sha_damage_target_alive_both_hand"
    );
    assert!(lieren.optional);
    let win = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == LIEREN_WIN)
        .unwrap();
    assert_eq!(win.filter, "owner_alive_skill_bound_target_is_event_source");
}

#[test]
fn juxiang_gains_resolved_nanman_from_discard() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let hand = tx.state().zones.find(ZoneKind::Hand, setup.source).unwrap();
    let nanman = alloc_card(
        &mut tx,
        setup.source,
        hand,
        CardKind::TrickNanman,
        96_300,
        Suit::Club,
        7,
    );
    // The resolved Nanman sits in the discard pile.
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![nanman],
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardResolved {
            source: setup.source,
            card: nanman,
            kind: 24,
            targets: vec![setup.owner, setup.target],
            damaged_targets: vec![],
            source_cards: vec![],
            all_sources_were_hand: false,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => {}
            Err(EffectError::ChoiceRequested) => {}
            Err(e) => panic!("effect error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    let state = tx.commit().state;
    assert!(
        state.hand_cards(setup.owner).contains(&nanman),
        "the owner gained the resolved Nanman"
    );
}

#[test]
fn lieren_win_takes_one_card_from_damaged_target() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    // Compare cards: owner 8 beats target 5.
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let owner_card = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_310,
        Suit::Spade,
        8,
    );
    let target_hand = tx.state().zones.find(ZoneKind::Hand, setup.target).unwrap();
    let target_card = alloc_card(
        &mut tx,
        setup.target,
        target_hand,
        CardKind::Shan,
        96_311,
        Suit::Heart,
        5,
    );
    let target_give = alloc_card(
        &mut tx,
        setup.target,
        target_hand,
        CardKind::Tao,
        96_312,
        Suit::Heart,
        6,
    );
    let sha = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_313,
        Suit::Club,
        9,
    );
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealShaDamage {
                source: setup.owner,
                target: setup.target,
                card: sha,
                amount: 1,
                nature: 0,
                card_flags: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut state = tx.commit().state;
    state = resolve_choice(state, setup.owner, vec![1], 1);
    // Compare card choices.
    state = resolve_choice(state, setup.owner, vec![owner_card.raw()], 2);
    state = resolve_choice(state, setup.target, vec![target_card.raw()], 3);
    // The owner won: take one card from the damaged target.
    assert!(state.has_pending_choice(), "take-one-card choice pending");
    state = resolve_choice(state, setup.owner, vec![target_give.raw()], 4);
    assert!(
        state.hand_cards(setup.owner).contains(&target_give),
        "the owner gained a card from the damaged target"
    );
    assert!(!state.hand_cards(setup.target).contains(&target_give));
    assert!(!state.has_pending_choice());
}

#[test]
fn lieren_loss_takes_nothing() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let owner_card = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_320,
        Suit::Spade,
        5,
    );
    let target_hand = tx.state().zones.find(ZoneKind::Hand, setup.target).unwrap();
    let target_card = alloc_card(
        &mut tx,
        setup.target,
        target_hand,
        CardKind::Shan,
        96_321,
        Suit::Heart,
        8,
    );
    let target_give = alloc_card(
        &mut tx,
        setup.target,
        target_hand,
        CardKind::Tao,
        96_322,
        Suit::Heart,
        6,
    );
    let sha = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_323,
        Suit::Club,
        9,
    );
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealShaDamage {
                source: setup.owner,
                target: setup.target,
                card: sha,
                amount: 1,
                nature: 0,
                card_flags: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut state = tx.commit().state;
    state = resolve_choice(state, setup.owner, vec![1], 1);
    state = resolve_choice(state, setup.owner, vec![owner_card.raw()], 2);
    state = resolve_choice(state, setup.target, vec![target_card.raw()], 3);
    // The owner lost: no take choice.
    assert!(!state.has_pending_choice());
    assert!(!state.hand_cards(setup.owner).contains(&target_give));
}
