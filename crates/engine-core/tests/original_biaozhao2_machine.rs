//! 第二百五十九批：biaozhao2 表召·应状态机行为测试（ABI 709 / tag 732）。
//! 任意角色弃置与「表」同花色同点数的牌 → 「表」交予弃牌者 + owner 失去
//! 1 体力。花色点数不匹配 → 静默。

use engine_api::{
    CardId, DomainEvent, PlayerId, SkillDefId, Suit,
};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_api::CardMoveReason;
use engine_core::state::{GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const BIAOZHAO2: u32 = 3_118_576_324;
const BIAOZHAO_PILE_DEF: u32 = 3_640_810_944;
const BIAOZHAO2_SRC: &str =
    include_str!("../../../content/original/ported/sp2/biaozhao2.skill");

fn setup_with_pile(
    seed: u64,
    pile_suit: Suit,
    pile_rank: u8,
) -> (GameState, PlayerId, PlayerId, CardId) {
    let compiled = content_compiler::compile_source_named("biaozhao2-b259", BIAOZHAO2_SRC).unwrap();
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
        skill_def: SkillDefId::new(BIAOZHAO2),
    });
    run_effects(&mut tx, effects).unwrap();

    // 从牌堆取一张指定花色点数的牌作为「表」，建堆注入。
    let draw = tx.state().zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
    let draw_cards = tx.state().zones.get(draw).unwrap().cards.clone();
    let biao = *draw_cards
        .iter()
        .rev()
        .find(|card| {
            tx.state().cards.get(**card).is_some_and(|c| {
                c.suit == pile_suit && c.rank == pile_rank
            })
        })
        .expect("pile card not found");
    let pile_zone = tx
        .state_mut()
        .zones
        .alloc(engine_core::state::Zone {
            id: engine_api::ZoneId::NONE,
            kind: ZoneKind::Expansion,
            owner,
            cards: vec![biao],
        });
    tx.state_mut().skill_piles.push(engine_core::state::SkillPileRef {
        owner,
        skill_def: BIAOZHAO_PILE_DEF,
        zone: pile_zone,
        usable_as_hand: false,
        wants_lost_count: false,
    });
    if let Some(card_state) = tx.state_mut().cards.get_mut(biao) {
        card_state.zone = pile_zone;
    }
    let state = tx.commit().state;
    (state, owner, other, biao)
}

/// 直派 CardsMoved（→弃牌堆，Discard 原因）。
fn fire_discard(state: GameState, discarder: PlayerId, cards: Vec<CardId>) -> GameState {
    let from = state.zones.find(ZoneKind::Hand, discarder).unwrap();
    let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE).unwrap();
    let mut tx = Transaction::begin(state);
    match dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardsMoved {
            cards,
            from,
            to: discard,
            reason: CardMoveReason::Discard,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => {}
        Ok(effects) => {
            engine_core::effect::run_effects(&mut tx, effects).expect("effects apply");
        }
        Err(other) => panic!("unexpected: {other:?}"),
    }
    tx.commit().state
}

fn match_card(state: &GameState, suit: Suit, rank: u8) -> Option<CardId> {
    let draw = state.zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
    state
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
                .is_some_and(|c| c.suit == suit && c.rank == rank)
        })
        .copied()
}

#[test]
fn biaozhao2_matching_discard_gives_biao_and_loses_hp() {
    // 「表」= 黑桃 5。other 弃一张黑桃 5 → 「表」给 other + owner 失 1 体力。
    let (state, owner, other, biao) = setup_with_pile(20_260_913_01, Suit::Spade, 5);
    // 为 other 准备一张黑桃 5（从牌堆找到后搬入其手牌）。
    let draw = state.zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
    let t_hand = state.zones.find(ZoneKind::Hand, other).unwrap();
    let matched = match_card(&state, Suit::Spade, 5).expect("spade-5 in draw pile");
    let mut tx = Transaction::begin(state);
    tx.state_mut()
        .zones
        .get_mut(draw)
        .unwrap()
        .cards
        .retain(|c| *c != matched);
    tx.state_mut().zones.get_mut(t_hand).unwrap().cards.push(matched);
    if let Some(card_state) = tx.state_mut().cards.get_mut(matched) {
        card_state.zone = t_hand;
        card_state.owner = other;
    }
    let state = tx.commit().state;
    let hp_owner = state.players.get(owner).unwrap().hp;

    let state = fire_discard(state, other, vec![matched]);

    // 「表」交给弃牌者 other；owner 失 1 体力；堆清空。
    assert!(hand_of(&state, other).contains(&biao), "biao given to discarder");
    assert_eq!(
        state.players.get(owner).unwrap().hp,
        hp_owner - 1,
        "owner loses 1 hp"
    );
    assert_eq!(state.skill_pile_cards(owner, BIAOZHAO_PILE_DEF).len(), 0);
    let _ = t_hand;
}

fn hand_of(state: &GameState, player: PlayerId) -> Vec<CardId> {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    state.zones.get(hand).unwrap().cards.clone()
}

#[test]
fn biaozhao2_non_matching_discard_stays_silent() {
    // 弃牌与「表」花色点数均不同 → 静默。
    let (mut state, owner, other, _biao) = setup_with_pile(20_260_913_02, Suit::Spade, 5);
    // other 注入一张非黑桃5的牌。
    let draw = state.zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
    let t_hand = state.zones.find(ZoneKind::Hand, other).unwrap();
    let other_card = *state
        .zones
        .get(draw)
        .unwrap()
        .cards
        .iter()
        .rev()
        .find(|card| {
            state.cards.get(**card).is_some_and(|c| {
                !(c.suit == Suit::Spade && c.rank == 5)
            })
        })
        .unwrap();
    state
        .zones
        .get_mut(draw)
        .unwrap()
        .cards
        .retain(|c| *c != other_card);
    state.zones.get_mut(t_hand).unwrap().cards.push(other_card);
    if let Some(card_state) = state.cards.get_mut(other_card) {
        card_state.zone = t_hand;
        card_state.owner = other;
    }
    let hp_owner = state.players.get(owner).unwrap().hp;
    let state = fire_discard(state, other, vec![other_card]);
    assert_eq!(state.players.get(owner).unwrap().hp, hp_owner);
    assert_eq!(state.skill_pile_cards(owner, BIAOZHAO_PILE_DEF).len(), 1);
}
