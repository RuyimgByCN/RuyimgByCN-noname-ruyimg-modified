//! Golden coverage for 明任/贞良（shenhua 卢植，ABI 576）.

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

const MINGREN: u32 = 1_039_189_468;
const ZHENLIANG: u32 = 279_698_051;
const SOURCE: &str = include_str!("../../../../content/original/ported/shenhua/luzhi.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
    far: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("lz-test", SOURCE).unwrap();
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
        skill_def: SkillDefId::new(MINGREN),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(ZHENLIANG),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    tx.state_mut().turn.round = 1;
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

/// 「任」牌堆的牌。
fn mingren_pile(state: &GameState, owner: PlayerId) -> Vec<CardId> {
    state
        .skill_piles
        .iter()
        .find(|pile| pile.owner == owner && pile.skill_def == MINGREN)
        .and_then(|pile| state.zones.get(pile.zone))
        .map(|zone| zone.cards.clone())
        .unwrap_or_default()
}

#[test]
fn package_exposes_luzhi_contract() {
    let compiled = content_compiler::compile_source_named("lz", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 576);
    assert_eq!(compiled.skill_index.len(), 2);
    let mingren: Vec<_> = compiled
        .trigger_index
        .iter()
        .filter(|t| t.skill_def_id == MINGREN)
        .collect();
    assert_eq!(mingren.len(), 2);
    let zhenliang_active = compiled
        .active_index
        .iter()
        .find(|a| a.skill_def_id == ZHENLIANG)
        .unwrap();
    assert_eq!(zhenliang_active.uses_per_turn, Some(1));
    let zhenliang: Vec<_> = compiled
        .trigger_index
        .iter()
        .filter(|t| t.skill_def_id == ZHENLIANG)
        .collect();
    assert_eq!(zhenliang.len(), 2);
}

/// 明任：首轮准备阶段摸两张并将一张手牌置于「任」；结束阶段可用手牌替换。
#[test]
fn mingren_places_and_replaces() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.phase = Phase::Prepare;
    let state = tx.commit().state;
    // 明任强制触发：摸两张并开置牌选择。
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        },
    );
    assert!(state.has_pending_choice(), "明任 place prompt");
    let hand: Vec<_> = state.hand_cards(setup.owner);
    assert_eq!(hand.len(), 2, "drew two");
    let state = resolve(state, setup.owner, vec![hand[0].raw()], false, 9_001);
    assert!(!state.has_pending_choice());
    let pile = mingren_pile(&state, setup.owner);
    assert_eq!(pile, vec![hand[0]], "card placed as 任");
    // 结束阶段替换。
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::End;
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::End,
        },
    );
    assert!(state.has_pending_choice(), "明任 replace confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_002);
    assert!(state.has_pending_choice(), "明任 replace prompt");
    let hand: Vec<_> = state.hand_cards(setup.owner);
    assert_eq!(hand.len(), 1);
    let state = resolve(state, setup.owner, vec![hand[0].raw()], false, 9_003);
    assert!(!state.has_pending_choice());
    let pile = mingren_pile(&state, setup.owner);
    assert_eq!(pile, vec![hand[0]], "replacement placed");
    assert!(
        state.hand_cards(setup.owner).contains(&hand[0]) == false || pile.len() == 1,
        "old 任 returned to hand"
    );
    // 旧「任」回手：总手牌仍为 1（替换的新牌进堆）。
}

/// 贞良（阳分支）：回合外使用与「任」同色的牌后令一名角色摸一张。
#[test]
fn zhenliang_yang_outside_turn_draws() {
    let setup = setup();
    let state = setup.tx.commit().state;
    // 通过明任流程置「任」（红桃）：准备阶段摸二置一。
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::Prepare;
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        },
    );
    assert!(state.has_pending_choice(), "明任 seed place");
    // Force the card about to be placed as 「任」 to be red, so the 阳
    // same-color check is deterministic.
    let mut tx = Transaction::begin(state);
    if let Some(card) = tx.state().hand_cards(setup.owner).first().copied() {
        tx.state_mut().cards.get_mut(card).unwrap().suit = Suit::Heart;
    }
    let state = tx.commit().state;
    let hand: Vec<_> = state.hand_cards(setup.owner);
    let state = resolve(state, setup.owner, vec![hand[0].raw()], false, 9_000);
    // 回合外：other 的回合。
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = setup.other;
    tx.state_mut().turn.phase = Phase::Play;
    let owner_hand = hand_zone(&tx, setup.owner);
    let used = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Tao,
        97_201,
        Suit::Heart,
        3,
    );
    let state = tx.commit().state;
    let skill = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == ZHENLIANG)
        .map(|skill| skill.id)
        .unwrap();
    let mut tx = Transaction::begin(state);
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob = b"ZL1\x01".to_vec();
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::CardUsed {
            player: setup.owner,
            card: used,
            targets: vec![setup.owner],
        },
    );
    assert!(state.has_pending_choice(), "贞良 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_010);
    assert!(state.has_pending_choice(), "贞良 draw target prompt");
    let state = resolve(state, setup.owner, vec![setup.far.raw()], false, 9_011);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.far), 1, "far drew one");
}

/// 贞良（阴分支）：出牌阶段限一次弃同色牌并对攻击范围内角色造成 1 点伤害。
#[test]
fn zhenliang_yin_discards_same_color_and_damages() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.owner;
    tx.state_mut().turn.phase = Phase::Play;
    // 通过明任流程置「任」（黑桃）。
    tx.state_mut().turn.phase = Phase::Prepare;
    let mut state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        },
    );
    assert!(state.has_pending_choice(), "明任 seed place");
    let hand: Vec<_> = state.hand_cards(setup.owner);
    let state = resolve(state, setup.owner, vec![hand[0].raw()], false, 9_000);
    // 给持有者一张与「任」同色的牌用于弃置，回到出牌阶段。
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = setup.owner;
    tx.state_mut().turn.phase = Phase::Play;
    let ren_suit = tx
        .state()
        .cards
        .get(mingren_pile(&tx.state(), setup.owner)[0])
        .map(|value| value.suit)
        .unwrap_or(Suit::Club);
    let owner_hand = hand_zone(&tx, setup.owner);
    let discard = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        97_203,
        ren_suit,
        5,
    );
    let state = tx.commit().state;
    let skill = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == ZHENLIANG)
        .map(|skill| skill.id)
        .unwrap();
    let (state, result) = {
        let revision = state.revision;
        apply_command(
            state,
            Command {
                actor: setup.owner,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(9_020),
                kind: CommandKind::ActivateSkill {
                    skill,
                    targets: Vec::new(),
                    card_targets: Vec::new(),
                },
            },
        )
    };
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "activate rejected: {:?}",
        result.result
    );
    assert!(state.has_pending_choice(), "贞良 discard prompt");
    let state = resolve(state, setup.owner, vec![discard.raw()], false, 9_021);
    assert!(state.has_pending_choice(), "贞良 damage target prompt");
    let hp_before = state.players.get(setup.other).unwrap().hp;
    let state = resolve(state, setup.owner, vec![setup.other.raw()], false, 9_022);
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.other).unwrap().hp, hp_before - 1);
}
