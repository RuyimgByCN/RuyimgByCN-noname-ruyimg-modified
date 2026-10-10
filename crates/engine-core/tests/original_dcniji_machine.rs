//! 第二百六十批：dcniji 逆击状态机行为测试（ABI 710 / tag 733）。
//! 锁定技双臂：成为非装备牌目标后摸 1 张打「逆击」标记；结束阶段弃置手牌中
//! 全部标记牌。装备牌目标不触发。

use engine_api::{CardId, DomainEvent, PlayerId, SkillDefId, Suit, TurnPhaseTag};
use engine_core::state::CardKind;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCNIJI: u32 = 3_061_815_608;
const DCNIJI_TAG: u32 = 3_628_735_017;
const DCNIJI_SRC: &str =
    include_str!("../../../content/original/ported/sp2/dcniji.skill");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("dcniji-b260", DCNIJI_SRC).unwrap();
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
        skill_def: SkillDefId::new(DCNIJI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

/// 从牌堆取一张指定花色的非装备牌（引用不移动）。
fn take_non_equip(state: &GameState, suit: Suit) -> CardId {
    let draw = state.zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
    *state
        .zones
        .get(draw)
        .unwrap()
        .cards
        .iter()
        .rev()
        .find(|card| {
            state
                .cards
                .get(**card)
                .is_some_and(|c| c.suit == suit && c.kind.equip_slot().is_none())
        })
        .unwrap_or_else(|| panic!("no {suit:?} non-equip card"))
}

/// 从牌堆取一张装备牌。
fn take_equip(state: &GameState, suit: Suit) -> CardId {
    let draw = state.zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
    *state
        .zones
        .get(draw)
        .unwrap()
        .cards
        .iter()
        .rev()
        .find(|card| {
            state
                .cards
                .get(**card)
                .is_some_and(|c| c.suit == suit && c.kind.equip_slot().is_some())
        })
        .unwrap_or_else(|| panic!("no {suit:?} equipment"))
}

fn tagged_count(state: &GameState, player: PlayerId) -> usize {
    state
        .hand_cards(player)
        .iter()
        .filter(|card| state.card_has_content_tag(**card, DCNIJI_TAG))
        .count()
}

fn fire_targeted(state: GameState, source: PlayerId, target: PlayerId, card: CardId) -> GameState {
    let mut tx = Transaction::begin(state);
    match dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: source,
            card,
            targets: vec![target],
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(engine_core::effect::EffectError::ChoiceRequested) => {}
        Ok(effects) => {
            engine_core::effect::run_effects(&mut tx, effects).expect("effects apply");
        }
        Err(other) => panic!("unexpected: {other:?}"),
    }
    tx.commit().state
}

fn fire_end(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .expect("forced dispatch");
    engine_core::effect::run_effects(&mut tx, effects).expect("machine runs");
    tx.commit().state
}

#[test]
fn dcniji_draw_tag_and_end_phase_discard() {
    // 非装备牌指定目标 → 摸 1 打标；结束后标记牌全部弃置。
    let (state, owner, other) = setup(20_260_913_11);
    let sha = take_non_equip(&state, Suit::Spade);

    let state = fire_targeted(state, other, owner, sha);
    assert_eq!(tagged_count(&state, owner), 1, "drawn card tagged");
    let hand_before = state.hand_count(owner);

    let state = fire_end(state, owner);
    assert_eq!(tagged_count(&state, owner), 0, "end phase discards tagged");
    assert_eq!(state.hand_count(owner), hand_before - 1);
    let _ = sha;
}

#[test]
fn dcniji_equip_card_target_noop() {
    // 装备牌指定目标：不摸牌不打标。
    let (mut state, owner, other) = setup(20_260_913_12);
    let equip = take_equip(&mut state, Suit::Spade);
    let hand_before = state.hand_count(owner);

    let state = fire_targeted(state, other, owner, equip);
    assert_eq!(state.hand_count(owner), hand_before);
    assert_eq!(tagged_count(&state, owner), 0);
    let _ = equip;
}
