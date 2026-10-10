//! 第二百五十八批：chuanyun 穿云衍生技状态机行为测试（ABI 708 / tag 731）。
//! 锁定衍生技：杀指定目标后若其装备区有牌 → 随机弃置其一。
//! 测试：给目标注入装备（黑桃/红桃各 1）+ 驱动 card_targeted（杀）→ 断言
//! 恰弃其一；无装备目标 → 无弃置；非杀事件牌 → 无弃置。

use engine_api::{CardId, DomainEvent, PlayerId, SkillDefId, Suit};
use engine_core::state::CardKind;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const CHUANYUN: u32 = 2_334_949_774;
const CHUANYUN_SRC: &str =
    include_str!("../../../content/original/ported/sp2/chuanyun.skill");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("chuanyun-b258", CHUANYUN_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(CHUANYUN),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, target)
}

fn equip_cards(state: &GameState, player: PlayerId) -> Vec<CardId> {
    let equip = state.zones.find(ZoneKind::Equip, player).unwrap();
    state.zones.get(equip).unwrap().cards.clone()
}

/// 从牌堆取一张指定花色+牌种+点数的杀。
fn take_sha(state: &GameState, suit: Suit) -> CardId {
    let draw = state.zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
    *state
        .zones
        .get(draw)
        .unwrap()
        .cards
        .iter()
        .rev()
        .find(|card| {
            state.cards.get(**card).is_some_and(|c| {
                c.suit == suit
                    && matches!(
                        c.kind,
                        CardKind::Sha | CardKind::ShaFire | CardKind::ShaThunder
                    )
            })
        })
        .unwrap_or_else(|| panic!("no {suit:?} sha in draw pile"))
}

/// 从牌堆取一张指定花色的装备置入 player 装备区。
fn add_equip_of_suit(state: &mut GameState, player: PlayerId, suit: Suit) -> CardId {
    let draw = state.zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
    let equip = state.zones.find(ZoneKind::Equip, player).unwrap();
    let draw_cards = state.zones.get(draw).unwrap().cards.clone();
    let card = *draw_cards
        .iter()
        .rev()
        .find(|card| {
            state
                .cards
                .get(**card)
                .is_some_and(|c| c.suit == suit && c.kind.equip_slot().is_some())
        })
        .unwrap_or_else(|| panic!("no {suit:?} equipment in draw pile"));
    state
        .zones
        .get_mut(draw)
        .unwrap()
        .cards
        .retain(|c| *c != card);
    state.zones.get_mut(equip).unwrap().cards.push(card);
    if let Some(card_state) = state.cards.get_mut(card) {
        card_state.zone = equip;
        card_state.owner = player;
    }
    card
}

fn fire_sha_targeted(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    sha: CardId,
) -> GameState {
    let mut tx = Transaction::begin(state);
    match dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: sha,
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

#[test]
fn chuanyun_discards_random_equip_on_sha_target() {
    let (mut state, owner, target) = setup(20_260_912_141);
    let e1 = add_equip_of_suit(&mut state, target, Suit::Spade);
    let e2 = add_equip_of_suit(&mut state, target, Suit::Heart);
    let sha = take_sha(&state, Suit::Club);
    let discard_before = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .map(|z| state.zones.get(z).unwrap().cards.len())
        .unwrap_or(0);

    let state = fire_sha_targeted(state, owner, target, sha);
    // 恰弃其一：装备区剩 1 张，弃牌堆 +1，且弃的必是两者之一。
    assert_eq!(equip_cards(&state, target).len(), 1);
    let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE).unwrap();
    let d = state.zones.get(discard).unwrap().cards.clone();
    assert_eq!(d.len(), discard_before + 1);
    let discarded = d[d.len() - 1];
    assert!(discarded == e1 || discarded == e2);
}

#[test]
fn chuanyun_no_equip_noop() {
    let (state, owner, target) = setup(20_260_912_142);
    let sha = take_sha(&state, Suit::Spade);
    let equip_before = equip_cards(&state, target).len();
    let state = fire_sha_targeted(state, owner, target, sha);
    assert_eq!(equip_cards(&state, target).len(), equip_before);
}
