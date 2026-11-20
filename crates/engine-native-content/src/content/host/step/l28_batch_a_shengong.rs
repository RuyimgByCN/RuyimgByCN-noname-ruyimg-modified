//! L28 雷霆万钧 Batch A：`th_bushi`（陆凯 `extension.js:4203-4387`）与 `th_shengong`（蒲元 `extension.js:5163-5502`）。
//! 自 `l28_batch_a.rs` / `l28_batch_a_asks.rs` 拆出以保持非测试源文件有效行数在限制内。

use super::super::*;
use super::equip_card_onto;
use super::l28_batch_a_asks::wait_ask;
use super::{
    alive_all, alive_others, bushi_suits, discard_cards, flag, hand_and_equips, set_flag,
    suit_index, turn_stamp, ASK_BUSHI_POSTURE, ASK_SHENGONG_ASSIST, ASK_SHENGONG_CARD,
    ASK_SHENGONG_DISCARD, ASK_SHENGONG_TARGET,
};
use crate::catalog::{Catalog, EquipSlot};
use crate::choice::ChoiceOption;
use crate::event::{DetachedKind, EventData};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::state::GameStateCore;

/// `th_bushi` 全 24 种 `[s0, s1, s2, s3]` 花色排列（`extension.js:4208-4320`）。
/// 索引 0 为上游初始顺序 `['spade', 'diamond', 'club', 'heart']` = `[0, 3, 2, 1]`；
/// 索引 1..=3 保留原循环偏移以兼容既有测试，其余 20 项覆盖剩余非循环排列。
const BUSHI_PERMS: [[u32; 4]; 24] = [
    [0, 3, 2, 1],
    [1, 2, 3, 0],
    [2, 3, 0, 1],
    [3, 0, 1, 2],
    [0, 1, 2, 3],
    [0, 1, 3, 2],
    [0, 2, 1, 3],
    [0, 2, 3, 1],
    [0, 3, 1, 2],
    [1, 0, 2, 3],
    [1, 0, 3, 2],
    [1, 2, 0, 3],
    [1, 3, 0, 2],
    [1, 3, 2, 0],
    [2, 0, 1, 3],
    [2, 0, 3, 1],
    [2, 1, 0, 3],
    [2, 1, 3, 0],
    [2, 3, 1, 0],
    [3, 0, 2, 1],
    [3, 1, 0, 2],
    [3, 1, 2, 0],
    [3, 2, 0, 1],
    [3, 2, 1, 0],
];

/// `th_bushi` 摸花色牌（`extension.js:4377-4384` `get.cardPile`）：自牌堆顶向下，否则弃牌堆自顶向下；均无则不摸牌。
pub(super) fn gain_suit_card(ctx: &mut Ctx<'_>, owner: PlayerId, suit_idx: u32) {
    if let Some(pos) = ctx
        .core
        .deck
        .iter()
        .rposition(|&c| suit_index(ctx.catalog.cards[c as usize].suit) == suit_idx)
    {
        let c = ctx.core.deck.remove(pos);
        ctx.core.player_mut(owner).hand.push(c);
    } else if let Some(pos) = ctx
        .core
        .discard
        .iter()
        .rposition(|&c| suit_index(ctx.catalog.cards[c as usize].suit) == suit_idx)
    {
        let c = ctx.core.discard.remove(pos);
        ctx.core.player_mut(owner).hand.push(c);
    }
}

pub(super) fn step_th_bushi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => match m.target {
            Some(0) if m.card.is_none() => {
                wait_ask(m, ctx, HostStage::SelectNature, ASK_BUSHI_POSTURE)
            }
            Some(5) if m.card.is_none() => {
                let (_, _, _, s3) = bushi_suits(ctx.core, owner);
                gain_suit_card(ctx, owner, s3);
                Flow::Done
            }
            _ => {
                let (_, s1, s2, s3) = bushi_suits(ctx.core, owner);
                let actor = m.targets.first().copied().unwrap_or(owner);
                let c_suit = m
                    .card
                    .map(|c| suit_index(ctx.catalog.effective_card_suit(ctx.core, actor, c)));
                if m.targets.first().copied() == Some(owner) && c_suit == Some(s1) {
                    ctx.start_draw(owner, 2);
                }
                if m.target == Some(owner) && c_suit == Some(s2) {
                    gain_suit_card(ctx, owner, s3);
                }
                Flow::Done
            }
        },
        HostStage::SelectNature => {
            let ans = ctx.answer().clone();
            let idx = match ans.first().copied() {
                Some(ChoiceOption::Tag(t)) => (t as usize) % BUSHI_PERMS.len(),
                _ => 0,
            };
            let [s0, s1, s2, s3] = BUSHI_PERMS[idx];
            set_flag(ctx.core, owner, "th_bushi_s0", s0);
            set_flag(ctx.core, owner, "th_bushi_s1", s1);
            set_flag(ctx.core, owner, "th_bushi_s2", s2);
            set_flag(ctx.core, owner, "th_bushi_s3", s3);
            set_flag(ctx.core, owner, "th_bushi_tao", s0);
            set_flag(ctx.core, owner, "th_bushi_sha", s1);
            set_flag(ctx.core, owner, "th_bushi_ying", s2);
            set_flag(ctx.core, owner, "th_bushi_zhi", s3);
            set_flag(ctx.core, owner, "th_bushi_unlimited_suit", s0 + 1);
            set_flag(ctx.core, owner, "th_bushi_suits_assigned", 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn shengong_category(slot: EquipSlot) -> u8 {
    match slot {
        EquipSlot::Weapon => 0,
        EquipSlot::Armor => 1,
        EquipSlot::HorsePlus | EquipSlot::HorseMinus | EquipSlot::Treasure => 2,
    }
}

pub(super) fn shengong_used_mask(core: &GameStateCore, owner: PlayerId) -> u32 {
    if flag(core, owner, "th_shengong_used_stamp") == turn_stamp(core) {
        flag(core, owner, "th_shengong_used_mask")
    } else {
        0
    }
}

pub(super) fn shengong_discardable_equips(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<CardId> {
    let used = shengong_used_mask(core, owner);
    hand_and_equips(core, owner)
        .into_iter()
        .filter(|&c| {
            catalog
                .card_kind(c)
                .equip_slot()
                .is_some_and(|slot| (used & (1u32 << shengong_category(slot))) == 0)
        })
        .collect()
}

pub(super) fn shengong_card_names(cat: u8) -> &'static [&'static str] {
    match cat {
        0 => &[
            "wushuangfangtianji",
            "guilongzhanyuedao",
            "chixueqingfeng",
            "bintieshuangji",
            "wutiesuolian",
            "wuxinghelingshan",
        ],
        1 => &[
            "linglongshimandai",
            "hongmianbaihuapao",
            "qimenbagua",
            "guofengyupao",
            "huxinjing",
            "heiguangkai",
        ],
        _ => &[
            "shufazijinguan",
            "xuwangzhimian",
            "tianjitu",
            "taigongyinfu",
            "sanlve",
            "zhaogujing",
        ],
    }
}

pub(super) fn shengong_targets_for(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    forged: Option<CardId>,
) -> Vec<PlayerId> {
    alive_all(core, owner)
        .into_iter()
        .filter(|&p| forged.is_none_or(|c| super::l28_batch_a_ext::equip_slot_usable(core, catalog, p, c)))
        .collect()
}

fn advance_shengong_assist(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    while !m.targets.is_empty() {
        let next = m.targets.remove(0);
        if ctx.core.player(next).alive {
            m.target = Some(next);
            return wait_ask(m, ctx, HostStage::SelectNature, ASK_SHENGONG_ASSIST);
        }
    }
    m.target = None;
    let cat = flag(ctx.core, owner, "th_shengong_cat") as u8;
    let help = flag(ctx.core, owner, "th_shengong_help");
    let hamper = flag(ctx.core, owner, "th_shengong_hamper");
    let duanzao_result: u32 = if hamper == 0 {
        2
    } else if help >= hamper {
        1
    } else {
        0
    };
    set_flag(ctx.core, owner, "th_shengong_quality", duanzao_result + 1);
    let max_opts = (duanzao_result + 1) as usize;
    let names = shengong_card_names(cat);
    let mut avail: Vec<CardId> = ctx
        .core
        .detached_cards
        .iter()
        .filter_map(|&(cid, _)| {
            let name = ctx.catalog.cards.get(cid as usize)?.name;
            names.contains(&name).then_some(cid)
        })
        .collect();
    if avail.is_empty() {
        return Flow::Done;
    }
    let mut picked = Vec::new();
    while !avail.is_empty() && picked.len() < max_opts {
        let idx = ctx.core.rng.gen_range(avail.len() as u32) as usize;
        picked.push(avail.remove(idx));
    }
    m.cards = picked;
    wait_ask(m, ctx, HostStage::HuanrenPick, ASK_SHENGONG_CARD)
}

/// 蒲元 `th_shengong` & 触发器 `th_shengong_draw`（`extension.js:5163-5502`）。
pub(super) fn step_th_shengong(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act && m.target == Some(5) {
        let n = flag(ctx.core, owner, "th_shengong_draw_marks");
        set_flag(ctx.core, owner, "th_shengong_draw_marks", 0);
        set_flag(ctx.core, owner, "th_shengong_draw_stamp", 0);
        if n > 0 {
            ctx.start_draw(owner, n);
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_SHENGONG_DISCARD),
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let Some(slot) = ctx.catalog.card_kind(c).equip_slot() else {
                return Flow::Done;
            };
            let cat = shengong_category(slot);
            discard_cards(ctx, owner, &[c]);
            let stamp = turn_stamp(ctx.core);
            let used = shengong_used_mask(ctx.core, owner) | (1u32 << cat);
            set_flag(ctx.core, owner, "th_shengong_used_stamp", stamp);
            set_flag(ctx.core, owner, "th_shengong_used_mask", used);
            set_flag(ctx.core, owner, "th_shengong_category", u32::from(cat) + 1);
            set_flag(ctx.core, owner, "th_shengong_cat", u32::from(cat));
            // `extension.js:5309`: 蒲元先置 1 张牌作基础助力点数，其余存活角色可选择助力/妨害/不参与。
            let base_card = ctx.draw_judge_card();
            let base_rank = u32::from(ctx.catalog.cards[base_card as usize].rank);
            ctx.to_discard(base_card);
            set_flag(ctx.core, owner, "th_shengong_help", base_rank);
            set_flag(ctx.core, owner, "th_shengong_hamper", 0);
            m.targets = alive_others(ctx.core, owner);
            advance_shengong_assist(m, ctx)
        }
        HostStage::SelectNature => {
            match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(0)) => {
                    let c = ctx.draw_judge_card();
                    let r = u32::from(ctx.catalog.cards[c as usize].rank);
                    ctx.to_discard(c);
                    let cur = flag(ctx.core, owner, "th_shengong_help");
                    set_flag(ctx.core, owner, "th_shengong_help", cur + r);
                }
                Some(ChoiceOption::Tag(1)) => {
                    let c = ctx.draw_judge_card();
                    let r = u32::from(ctx.catalog.cards[c as usize].rank);
                    ctx.to_discard(c);
                    let cur = flag(ctx.core, owner, "th_shengong_hamper");
                    set_flag(ctx.core, owner, "th_shengong_hamper", cur + r);
                }
                _ => {}
            }
            advance_shengong_assist(m, ctx)
        }
        HostStage::HuanrenPick => {
            let Some(ChoiceOption::Card(forged)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.card = Some(forged);
            m.cards.clear();
            if shengong_targets_for(ctx.core, ctx.catalog, owner, Some(forged)).is_empty() {
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_SHENGONG_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let Some(forged) = m.card else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, &format!("th_shengong_destroy:{forged}"), 1);
            set_flag(ctx.core, owner, "th_shengong_forged", 1);
            if super::l28_batch_a_ext::equip_slot_usable(ctx.core, ctx.catalog, t, forged)
                && ctx.core.player(t).alive
            {
                ctx.core.detached_cards.retain(|&(cid, _)| cid != forged);
                for p in 0..ctx.core.players.len() as PlayerId {
                    ctx.core.player_mut(p).hand.retain(|&c| c != forged);
                    if ctx.core.player(p).equips.contains(&forged) {
                        ctx.core.player_mut(p).equips.retain(|&c| c != forged);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, p, forged);
                    }
                }
                ctx.core.player_mut(t).hand.push(forged);
                ctx.emit(EventData::Equipped {
                    player: t,
                    card: forged,
                    replaced: None,
                });
            } else {
                equip_card_onto(ctx, t, forged);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

/// `th_shengong_destroy`（`extension.js:5456-5486`）：神工专属装备进入弃牌堆时移出游戏（归 `detached_cards`）并记本回合结束摸牌标记。
pub(super) fn intercept_shengong_destroy(ctx: &mut Ctx<'_>, cards: &[CardId]) {
    if ctx.core.vm_aux.flags.is_empty() || cards.is_empty() {
        return;
    }
    let seats = ctx.core.players.len() as PlayerId;
    let stamp = turn_stamp(ctx.core);
    for &c in cards {
        if !ctx.core.discard.contains(&c) {
            continue;
        }
        let key = format!("th_shengong_destroy:{c}");
        for p in 0..seats {
            if flag(ctx.core, p, &key) == 0 {
                continue;
            }
            ctx.core.vm_aux.flags.remove(&(p, key.clone()));
            ctx.core.discard.retain(|&x| x != c);
            if !ctx.core.detached_cards.iter().any(|&(cid, _)| cid == c) {
                ctx.core.detached_cards.push((c, DetachedKind::Unclaimed));
            }
            let prev = if flag(ctx.core, p, "th_shengong_draw_stamp") == stamp {
                flag(ctx.core, p, "th_shengong_draw_marks")
            } else {
                0
            };
            set_flag(ctx.core, p, "th_shengong_draw_stamp", stamp);
            set_flag(ctx.core, p, "th_shengong_draw_marks", prev + 1);
            break;
        }
    }
}
