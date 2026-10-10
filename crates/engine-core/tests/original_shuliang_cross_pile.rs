//! 第二百一十八批：shuliang 输粮零铸造晋升行为测试（跨技堆单张移除 + event_player 摸牌）。
//! 形状：任一角色结束阶段 → 粮堆（tunchu 技能堆 def 3943350142）非空 →
//! body 级 if 手牌<体力 → ABI 686 选堆单卡弃置（取消=不发动）→ 该角色摸 2。
//! 另定界：twqingce（active 目标引用缺位）、biaozhao3（全场手牌上限查询）、
//! twxingwu/sbyingmen/pelishi（多张移去面复合）——维持登记债，移去面关账。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Zone, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const SHULIANG: u32 = 3_563_189_871;
const TUNCHU_PILE: u32 = 3_943_350_142;
const SHULIANG_SRC: &str = include_str!("../../../content/original/ported/mobile/shuliang.skill");
const MOBILE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("shuliang-b218", SHULIANG_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SHULIANG),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn inject_hand_card(state: GameState, owner: PlayerId, def: u32) -> (GameState, CardId) {
    let mut tx = Transaction::begin(state);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    (tx.commit().state, card)
}

/// 直构 (owner, TUNCHU_PILE) 粮堆并放入 n 张（模拟 tunchu 置入产物）。
fn fill_grain_pile(state: GameState, owner: PlayerId, n: usize) -> (GameState, Vec<CardId>) {
    let mut state = state;
    let mut cards = Vec::new();
    for i in 0..n {
        let (s2, card) = inject_hand_card(state, owner, 9_930 + i as u32);
        state = s2;
        let mut tx = Transaction::begin(state);
        let existing = tx
            .state()
            .skill_piles
            .iter()
            .find(|p| p.owner == owner && p.skill_def == TUNCHU_PILE)
            .map(|p| p.zone);
        let zone = match existing {
            Some(zone) => zone,
            None => {
                let zone = tx.state_mut().zones.alloc(Zone {
                    id: engine_api::ZoneId::NONE,
                    kind: ZoneKind::Expansion,
                    owner,
                    cards: Vec::new(),
                });
                tx.state_mut()
                    .skill_piles
                    .push(engine_core::state::SkillPileRef {
                        owner,
                        skill_def: TUNCHU_PILE,
                        zone,
                        usable_as_hand: false,
                        wants_lost_count: false,
                    });
                zone
            }
        };
        let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
        tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
        tx.state_mut()
            .zones
            .get_mut(hand)
            .unwrap()
            .cards
            .retain(|c| *c != card);
        if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
            card_state.zone = zone;
        }
        cards.push(card);
        state = tx.commit().state;
    }
    (state, cards)
}

fn fire_end_phase(state: GameState, current: PlayerId) -> (GameState, Result<(), EffectError>) {
    let mut tx = Transaction::begin(state);
    let outcome = match dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
        Ok(effects) => engine_core::effect::run_effects(&mut tx, effects).map(|_| ()),
        Err(other) => panic!("unexpected dispatch error: {other:?}"),
    };
    (tx.commit().state, outcome)
}


/// 压低手牌至 target_count 张（逐张移入弃牌堆；同步卡区状态）。
fn shrink_hand_to(mut state: GameState, owner: PlayerId, target_count: u32) -> GameState {
    while state.hand_count(owner) > target_count {
        let mut tx = Transaction::begin(state);
        let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
        let discard = tx
            .state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .expect("discard pile");
        let card = tx.state().zones.get(hand).unwrap().cards[0];
        tx.state_mut()
            .zones
            .get_mut(hand)
            .unwrap()
            .cards
            .retain(|c| *c != card);
        tx.state_mut().zones.get_mut(discard).unwrap().cards.push(card);
        if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
            card_state.zone = discard;
            card_state.owner = PlayerId::NONE;
        }
        state = tx.commit().state;
    }
    state
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
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
        "resolve rejected {key}: {:?}",
        result.result
    );
    next
}

#[test]
fn shuliang_contract_and_cross_pile_removal() {
    // 契约：包内触发存在（phase_end、optional、跨技粮堆过滤器）。
    let package = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == SHULIANG)
        .expect("shuliang trigger");
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(trigger.filter, "owner_alive", "global end-phase scope");

    // 行为：other 的结束阶段（other 手牌 bootstrap 5 > hp? bootstrap hp=4 时不满足；
    // 用 owner 自己的结束阶段验证全链：owner 手牌 bootstrap 4 < hp 4? 用直接构造
    // 堆 + 确认 + 选卡 → 堆-1 + event_player 摸 2。此处 current=owner（自回合）。
    let (state, owner, _other) = setup(20_260_924_01);
    let owner_hp = state.players.get(owner).unwrap().hp as u32;
    let state = shrink_hand_to(state, owner, owner_hp - 1);
    let (state, cards) = fill_grain_pile(state, owner, 2);
    let owner_hand_before = state.hand_count(owner);

    let (state, outcome) = fire_end_phase(state, owner);
    assert!(
        matches!(outcome, Err(EffectError::ChoiceRequested)),
        "optional confirm must open: {outcome:?}"
    );
    let state = resolve(state, owner, vec![1], 20_260_924_11);
    assert!(
        state.has_pending_choice(),
        "pile card selection must follow confirm"
    );
    // 选一张粮 → 弃置 → 摸 2。
    let state = resolve(state, owner, vec![cards[0].raw()], 20_260_924_12);
    assert!(!state.has_pending_choice());
    assert_eq!(
        state.skill_pile_cards(owner, TUNCHU_PILE),
        vec![cards[1]],
        "one grain removed"
    );
    assert_eq!(
        state.hand_count(owner),
        owner_hand_before + 2,
        "event player (owner) draws 2"
    );
}

#[test]
fn shuliang_fires_on_other_end_phase_and_other_draws() {
    // other 的结束阶段（other 手牌<体力）→ 移 owner 粮堆一张 → other 摸 2。
    let (state, owner, other) = setup(20_260_924_02);
    let other_hp = state.players.get(other).unwrap().hp as u32;
    let state = shrink_hand_to(state, other, other_hp - 1);
    let other_hand_before = state.hand_count(other);
    let (state, cards) = fill_grain_pile(state, owner, 1);

    let (state, outcome) = fire_end_phase(state, other);
    assert!(
        matches!(outcome, Err(EffectError::ChoiceRequested)),
        "global end-phase trigger must fire for other's phase: {outcome:?}"
    );
    let state = resolve(state, owner, vec![1], 20_260_924_21);
    assert!(state.has_pending_choice());
    let state = resolve(state, owner, vec![cards[0].raw()], 20_260_924_22);
    assert!(!state.has_pending_choice());
    assert_eq!(state.skill_pile_cards(owner, TUNCHU_PILE).len(), 0);
    assert_eq!(
        state.hand_count(other),
        other_hand_before + 2,
        "event player (other) draws 2"
    );
}

#[test]
fn shuliang_hand_not_below_hp_gate_blocks() {
    // 事件角色手牌数 ≥ 体力值 → body 级门控拦截：确认后无选卡。
    let (state, owner, other) = setup(20_260_924_03);
    let (state, _) = fill_grain_pile(state, owner, 1);

    let (state, outcome) = fire_end_phase(state, other);
    assert!(
        matches!(outcome, Err(EffectError::ChoiceRequested)),
        "confirm still opens (filter has no hand gate)"
    );
    let state = resolve(state, owner, vec![1], 20_260_924_31);
    assert!(
        !state.has_pending_choice(),
        "hand>=hp gate blocks selection"
    );
    assert_eq!(
        state.skill_pile_cards(owner, TUNCHU_PILE).len(),
        1,
        "pile intact"
    );
}
