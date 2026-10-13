//! O2 全量合并第二批（2026-10-14）：家族 twmouzhi 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_twmouzhi_machine {
//! 第二百四十五批：twmouzhi 谋制专属状态机行为测试（ABI 700 / tag 723）。
//! 锁定技：受到伤害时若此牌颜色与上次受伤牌颜色相同则防止（防止不更新记录）；
//! 每次受伤后记录颜色（无牌清除）。驱动：直接推 CommitDamage（带牌/无牌）。

use engine_api::{CardId, PlayerId, SkillDefId, Suit};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::{damage_nature, EffectRequest};

const TWMOUZHI: u32 = 2_608_393_664;
const TWMOUZHI_SRC: &str =
    include_str!("../../../content/original/ported/tw/twmouzhi.skill");
const COLOR_KEY: u32 = 1;

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("twmouzhi-b245", TWMOUZHI_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let source = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWMOUZHI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, source)
}

fn skill_instance(state: &GameState) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .iter()
        .find(|skill| skill.def.raw() == TWMOUZHI)
        .map(|skill| skill.id)
        .unwrap()
}

fn recorded_color(state: &GameState) -> u32 {
    state
        .skills
        .storage_get(skill_instance(state), COLOR_KEY)
}

/// 找一张指定花色的牌（留在牌堆，不移动——伤害管线只引用其颜色）。
fn take_card_of_suit(state: &GameState, suit: Suit) -> CardId {
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
                .is_some_and(|card_state| card_state.suit == suit)
        })
        .unwrap_or_else(|| panic!("no {suit:?} card in draw pile"))
}

fn damage_with_card(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
    card: Option<CardId>,
) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::CommitDamage {
            damage_id: 0,
            source,
            target,
            amount,
            base_amount: amount,
            is_lose_hp: false,
            nature: damage_nature::NORMAL,
            card: card.unwrap_or(CardId::NONE),
            card_kind: if card.is_some() {
                CardKind::Sha.tag()
            } else {
                0
            },
            card_flags: 0,
        }],
    )
    .expect("damage pipeline");
    tx.commit().state
}

#[test]
fn twmouzhi_contract_triggers_registered() {
    let compiled = content_compiler::compile_source_named("twmouzhi-c", TWMOUZHI_SRC).unwrap();
    assert!(compiled.trigger_index.iter().any(|t| {
        t.skill_def_id == TWMOUZHI
            && t.event == "damage_before"
            && t.filter == "owner_is_target"
            && !t.optional
    }));
    assert!(compiled.trigger_index.iter().any(|t| {
        t.skill_def_id == TWMOUZHI && t.event == "damage" && t.filter == "owner_is_target"
    }));
}

#[test]
fn twmouzhi_same_color_damage_prevented_and_sticky() {
    let (state, owner, source) = setup(20_260_912_31);
    let red = take_card_of_suit(&state, Suit::Heart);
    let black = take_card_of_suit(&state, Suit::Spade);
    let hp = state.players.get(owner).unwrap().hp;

    // 首次红杀伤害：生效并记录红色。
    let state = damage_with_card(state, source, owner, 1, Some(red));
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);
    assert_eq!(recorded_color(&state), 1);

    // 连续红杀伤害：防止（hp 不变），且防止不更新记录。
    let state = damage_with_card(state, source, owner, 1, Some(red));
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);
    assert_eq!(recorded_color(&state), 1);
    let state = damage_with_card(state, source, owner, 1, Some(red));
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);

    // 黑色伤害：颜色不同，生效并把记录切换为黑。
    let state = damage_with_card(state, source, owner, 1, Some(black));
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 2);
    assert_eq!(recorded_color(&state), 2);
}

#[test]
fn twmouzhi_cardless_damage_clears_record() {
    let (state, owner, source) = setup(20_260_912_32);
    let red = take_card_of_suit(&state, Suit::Heart);
    let hp = state.players.get(owner).unwrap().hp;

    // 建立红色记录。
    let state = damage_with_card(state, source, owner, 1, Some(red));
    assert_eq!(recorded_color(&state), 1);

    // 无牌伤害：不触发防止（无色），且清除记录。
    let state = damage_with_card(state, source, owner, 1, None);
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 2);
    assert_eq!(recorded_color(&state), 0);

    // 记录清空后，红色伤害重新生效并重建记录。
    let state = damage_with_card(state, source, owner, 1, Some(red));
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 3);
    assert_eq!(recorded_color(&state), 1);
}
}
