//! 刘徽 `th_geyuan`（割圆，`extension.js:8384-8590`）/ `th_jieshu`（解术，
//! `extension.js:8591-8613`）/ `th_gusuan`（孤算，`extension.js:8614-8633`）。
//!
//! - `th_geyuan_qyubee`：首回合开始时将「圆环之理」（`th_geyuan_ring_mask`）初始化为 1..=13。
//! - `th_jieshu`：使用或打出的牌若满足 `filter_number`（在「圆环之理」上、不在「圆环之弧」上、
//!   且「圆环之弧」为空或与已有弧相邻），摸一张牌。
//! - `th_geyuan`：牌进入弃牌堆或被使用/打出后，将满足 `filter_number` 的点数记入「圆环之弧」
//!   （`th_geyuan_homura_mask`）；当弧长达到环长时：
//!   - 孤算觉醒前：获得全场 `ej` 区与牌堆中点数等于首尾（`num1`/`num2`）的所有牌；
//!   - 孤算觉醒后（`th_geyuan_magica`）：选择 1~3 名角色，依次令第 1 人摸 3 张牌、第 2 人弃置
//!     4 张牌、第 3 人将所有手牌与牌堆底 5 张牌交换；
//!   随后清空「圆环之弧」，若「圆环之理」长度 > 3 则从中移除 `num1` 与 `num2`。
//! - `th_gusuan`（觉醒技）：任意角色回合结束时，若「圆环之理」长度为 3，减 1 点体力上限并升级割圆。

use super::*;

const FULL_RING_MASK: u32 = 0x3FFE; // bits 1..=13

fn ring_mask(core: &GameStateCore, owner: PlayerId) -> u32 {
    let m = flag(core, owner, "th_geyuan_ring_mask");
    if m != 0 {
        return m;
    }
    let len = flag(core, owner, "th_geyuan_ring_len");
    if (3..=13).contains(&len) {
        (1..=len).fold(0u32, |acc, r| acc | (1 << r))
    } else {
        FULL_RING_MASK
    }
}

fn ring_ranks(core: &GameStateCore, owner: PlayerId) -> Vec<u32> {
    let m = ring_mask(core, owner);
    (1..=13).filter(|&r| (m & (1 << r)) != 0).collect()
}

/// `lib.skill.th_geyuan.filterNumber(player, num)`（`extension.js:8493-8506`）。
pub(super) fn filter_number(core: &GameStateCore, owner: PlayerId, num: u32) -> bool {
    if !(1..=13).contains(&num) {
        return false;
    }
    let list1 = ring_ranks(core, owner);
    let Some(madoka) = list1.iter().position(|&x| x == num) else {
        return false;
    };
    let homura_mask = flag(core, owner, "th_geyuan_homura_mask");
    if homura_mask == 0 {
        return true;
    }
    if (homura_mask & (1 << num)) != 0 {
        return false;
    }
    let len = list1.len();
    list1.iter().enumerate().any(|(homura, &r)| {
        (homura_mask & (1 << r)) != 0 && {
            let dist = madoka.abs_diff(homura);
            dist == 1 || (len > 1 && dist == len - 1)
        }
    })
}

pub(super) fn jieshu_filter(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let card = match event {
        EventData::CardUsed(u) if u.player == owner => u.card,
        EventData::CardResponded { player, card } if *player == owner => Some(*card),
        _ => None,
    };
    card.is_some_and(|c| filter_number(core, owner, card_rank(catalog, c)))
}

pub(super) fn step_th_jieshu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if let Some(c) = m.card {
        let r = card_rank(ctx.catalog, c);
        if (1..=13).contains(&r) {
            let mask = flag(ctx.core, owner, "th_jieshu_rank_mask") | (1 << r);
            set_flag(ctx.core, owner, "th_jieshu_rank_mask", mask);
        }
    }
    set_flag(ctx.core, owner, "th_jieshu_drawn", 1);
    ctx.start_draw(owner, 1);
    Flow::Done
}

pub(super) fn geyuan_candidates(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    event: &EventData,
) -> Vec<CardId> {
    let cards: Vec<CardId> = match event {
        EventData::CardsDiscarded { cards, .. } | EventData::CardsLost { cards, .. } => {
            cards.clone()
        }
        EventData::CardUsed(u) => u.card.into_iter().collect(),
        EventData::CardResponded { card, .. } => vec![*card],
        _ => Vec::new(),
    };
    if cards
        .iter()
        .any(|&c| filter_number(core, owner, card_rank(catalog, c)))
    {
        cards
    } else {
        Vec::new()
    }
}

pub(super) fn step_th_geyuan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act if m.cards.is_empty() => {
            // `th_geyuan_qyubee`（`extension.js:8508`）：首回合开始初始化 1..=13 圆环之理。
            set_flag(ctx.core, owner, "th_geyuan_ring_mask", FULL_RING_MASK);
            set_flag(ctx.core, owner, "th_geyuan_ring_len", 13);
            Flow::Done
        }
        HostStage::Act => {
            let init_mask = ring_mask(ctx.core, owner);
            set_flag(ctx.core, owner, "th_geyuan_ring_mask", init_mask);
            let ring_len = init_mask.count_ones();
            set_flag(ctx.core, owner, "th_geyuan_ring_len", ring_len);

            let mut remaining = m.cards.clone();
            while let Some(pos) = remaining
                .iter()
                .position(|&c| filter_number(ctx.core, owner, card_rank(ctx.catalog, c)))
            {
                let c = remaining.remove(pos);
                let r = card_rank(ctx.catalog, c);
                let prev = flag(ctx.core, owner, "th_geyuan_homura_mask");
                if prev == 0 {
                    set_flag(ctx.core, owner, "th_geyuan_homura_first", r);
                }
                let next = prev | (1 << r);
                set_flag(ctx.core, owner, "th_geyuan_homura_mask", next);
                set_flag(ctx.core, owner, "th_geyuan_homura_last", r);
                let jmask = flag(ctx.core, owner, "th_jieshu_rank_mask") | (1 << r);
                set_flag(ctx.core, owner, "th_jieshu_rank_mask", jmask);
                if next.count_ones() >= ring_len {
                    break;
                }
            }
            if flag(ctx.core, owner, "th_geyuan_homura_mask").count_ones() < ring_len {
                return Flow::Done;
            }
            set_flag(ctx.core, owner, "th_geyuan_pi", 1);
            if flag(ctx.core, owner, "th_gusuan_awakened") > 0 {
                return wait_ask(m, ctx, HostStage::TargetChoice, ASK_GEYUAN_MAGICA_TARGETS);
            }
            resolve_pre_awakened_geyuan(ctx, owner);
            finish_geyuan_arc(ctx.core, owner);
            Flow::Done
        }
        HostStage::TargetChoice => {
            m.targets = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(p) => Some(p),
                    _ => None,
                })
                .collect();
            let Some(&t0) = m.targets.first() else {
                finish_geyuan_arc(ctx.core, owner);
                return Flow::Done;
            };
            ctx.start_draw(t0, 3);
            if let Some(&t1) = m.targets.get(1) {
                let he = hand_and_equips(ctx.core, t1);
                if he.len() <= 4 {
                    discard_cards(ctx, t1, &he);
                } else {
                    m.target = Some(t1);
                    return wait_ask(m, ctx, HostStage::CardsAsk, ASK_GEYUAN_MAGICA_DISCARD);
                }
            }
            resolve_magica_third(m, ctx);
            finish_geyuan_arc(ctx.core, owner);
            Flow::Done
        }
        HostStage::CardsAsk => {
            if let Some(t1) = m.target {
                let cards: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match *o {
                        ChoiceOption::Card(c) => Some(c),
                        _ => None,
                    })
                    .collect();
                discard_cards(ctx, t1, &cards);
            }
            resolve_magica_third(m, ctx);
            finish_geyuan_arc(ctx.core, owner);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn resolve_pre_awakened_geyuan(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let num1 = flag(ctx.core, owner, "th_geyuan_homura_first");
    let num2 = flag(ctx.core, owner, "th_geyuan_homura_last");
    let matches_end = |catalog: &Catalog, c: CardId| {
        let r = card_rank(catalog, c);
        r == num1 || r == num2
    };
    for p in alive_all(ctx.core, owner) {
        let eq_take: Vec<CardId> = ctx
            .core
            .player(p)
            .equips
            .iter()
            .copied()
            .filter(|&c| matches_end(ctx.catalog, c))
            .collect();
        for c in eq_take {
            ctx.core.player_mut(p).equips.retain(|&x| x != c);
            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, p, c);
            ctx.core.player_mut(owner).hand.push(c);
        }
        let jd_take: Vec<CardId> = ctx
            .core
            .player(p)
            .judge
            .iter()
            .copied()
            .filter(|&c| matches_end(ctx.catalog, c))
            .collect();
        for c in jd_take {
            ctx.core.player_mut(p).judge.retain(|&x| x != c);
            ctx.core.player_mut(owner).hand.push(c);
        }
    }
    let deck_take: Vec<CardId> = ctx
        .core
        .deck
        .iter()
        .copied()
        .filter(|&c| matches_end(ctx.catalog, c))
        .collect();
    for c in deck_take {
        ctx.core.deck.retain(|&x| x != c);
        ctx.core.player_mut(owner).hand.push(c);
    }
}

fn resolve_magica_third(m: &HostMachine, ctx: &mut Ctx<'_>) {
    let Some(&t2) = m.targets.get(2) else {
        return;
    };
    let bottom_n = ctx.core.deck.len().min(5);
    let bottom_cards: Vec<CardId> = ctx.core.deck.drain(0..bottom_n).collect();
    let old_hand = std::mem::take(&mut ctx.core.player_mut(t2).hand);
    for c in old_hand {
        ctx.core.deck.insert(0, c);
    }
    ctx.core.player_mut(t2).hand.extend(bottom_cards);
}

fn finish_geyuan_arc(core: &mut GameStateCore, owner: PlayerId) {
    let num1 = flag(core, owner, "th_geyuan_homura_first");
    let num2 = flag(core, owner, "th_geyuan_homura_last");
    set_flag(core, owner, "th_geyuan_homura_mask", 0);
    set_flag(core, owner, "th_geyuan_homura_first", 0);
    set_flag(core, owner, "th_geyuan_homura_last", 0);
    let cur_mask = ring_mask(core, owner);
    let next_mask = if cur_mask.count_ones() > 3 {
        cur_mask & !((1 << num1) | (1 << num2))
    } else {
        cur_mask
    };
    set_flag(core, owner, "th_geyuan_ring_mask", next_mask);
    set_flag(core, owner, "th_geyuan_ring_len", next_mask.count_ones());
}

pub(super) fn step_th_gusuan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    set_flag(ctx.core, owner, "th_gusuan_awakened", 1);
    ctx.lose_max_hp(owner, 1);
    Flow::Done
}
