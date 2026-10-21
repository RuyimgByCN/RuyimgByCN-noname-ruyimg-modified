//! Golden coverage for 奇制/进趋（shenhua 王基，ABI 573）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const QIZHI: u32 = 2_224_182_078;
const JINQU: u32 = 126_618_454;
const SOURCE: &str = include_str!("../../../../content/original/ported/shenhua/wangji.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
    far: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("wangji-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let other = seats[1];
    let far = seats[2];
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
        skill_def: SkillDefId::new(QIZHI),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JINQU),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    Setup {
        tx,
        owner,
        other,
        far,
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

fn hand_zone(tx: &Transaction, player: PlayerId) -> engine_api::ZoneId {
    tx.state().zones.find(ZoneKind::Hand, player).unwrap()
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
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
        "unexpected result: {:?}",
        result.result
    );
    state
}

fn dispatch(state: GameState, event: DomainEvent) -> GameState {
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new());
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Err(EffectError::ChoiceRequested) => {}
            Ok(_) => {}
            Err(e) => panic!("dispatch error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    tx.commit().state
}

#[test]
fn package_exposes_wangji_contract() {
    let compiled = content_compiler::compile_source_named("wangji", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 573);
    assert_eq!(compiled.skill_index.len(), 2);
    let qizhi = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == QIZHI)
        .unwrap();
    assert_eq!(qizhi.event, "card_used");
    assert!(qizhi.optional);
    let jinqu = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == JINQU)
        .unwrap();
    assert_eq!(jinqu.event, "phase_end");
    assert!(jinqu.optional);
}

/// 奇制：用牌后弃非目标角色一张牌并令其摸一张，计数 +1。
#[test]
fn qizhi_discards_non_target_and_draws() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.owner;
    tx.state_mut().turn.phase = Phase::Play;
    let other_hand = hand_zone(&tx, setup.other);
    let victim_card = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Sha,
        97_000,
        Suit::Club,
        2,
    );
    let far_hand = hand_zone(&tx, setup.far);
    let _far_card = alloc_card(
        &mut tx,
        setup.far,
        far_hand,
        CardKind::Tao,
        97_001,
        Suit::Heart,
        3,
    );
    let owner_hand = hand_zone(&tx, setup.owner);
    let used = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        97_002,
        Suit::Spade,
        4,
    );
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::CardUsed {
            player: setup.owner,
            card: used,
            targets: vec![setup.far],
        },
    );
    assert!(state.has_pending_choice(), "奇制 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_001);
    assert!(state.has_pending_choice(), "奇制 target prompt");
    // The card's own target (far) is excluded from the discard choice.
    let state = resolve(state, setup.owner, vec![setup.other.raw()], false, 9_002);
    assert!(state.has_pending_choice(), "奇制 card prompt");
    let state = resolve(state, setup.owner, vec![victim_card.raw()], false, 9_003);
    assert!(!state.has_pending_choice());
    // The victim's card is discarded; the victim drew one.
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(victim_card).unwrap().zone, discard);
    assert_eq!(state.hand_count(setup.other), 1, "victim drew one");
    // The count incremented.
    let skill = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == QIZHI)
        .unwrap();
    assert_eq!(skill.state_blob, b"QZ1\x01\x00\x00\x00\x01");
    let _ = used;
}

/// 进趋：出牌阶段结束时摸两张并弃至奇制次数张（计数 0 → 弃两张至 0）。
#[test]
fn jinqu_draws_two_and_discards_to_qizhi_count() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.owner;
    tx.state_mut().turn.phase = Phase::End;
    let owner_hand = hand_zone(&tx, setup.owner);
    let h1 = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        97_003,
        Suit::Club,
        5,
    );
    let h2 = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Shan,
        97_004,
        Suit::Heart,
        6,
    );
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::End,
        },
    );
    assert!(state.has_pending_choice(), "进趋 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_010);
    assert!(state.has_pending_choice(), "进趋 discard prompt");
    // count 0: discard all four (2 held + 2 drawn) down to zero.
    let hand: Vec<_> = state.hand_cards(setup.owner);
    assert_eq!(hand.len(), 4);
    let picks: Vec<_> = hand.iter().map(|card| card.raw()).collect();
    let state = resolve(state, setup.owner, picks, false, 9_011);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.owner), 0);
}

/// 进趋计数 1：弃一张保留一张。
#[test]
fn jinqu_keeps_qizhi_count_cards() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.owner;
    tx.state_mut().turn.phase = Phase::End;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == QIZHI)
        .map(|skill| skill.id)
        .unwrap();
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob = b"QZ1\x01\x00\x00\x00\x01".to_vec();
    let owner_hand = hand_zone(&tx, setup.owner);
    let h1 = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        97_005,
        Suit::Club,
        7,
    );
    let h2 = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Shan,
        97_006,
        Suit::Heart,
        8,
    );
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::End,
        },
    );
    assert!(state.has_pending_choice(), "进趋 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_020);
    // 2 held + 2 drawn = 4; keep 1 -> discard 3.
    assert!(state.has_pending_choice(), "进趋 discard prompt");
    let hand: Vec<_> = state.hand_cards(setup.owner);
    assert_eq!(hand.len(), 4);
    let picks: Vec<_> = hand
        .iter()
        .filter(|card| **card != h1)
        .take(3)
        .map(|card| card.raw())
        .collect();
    let state = resolve(state, setup.owner, picks, false, 9_021);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.owner), 1, "kept one");
    assert!(state.hand_cards(setup.owner).contains(&h1));
}
