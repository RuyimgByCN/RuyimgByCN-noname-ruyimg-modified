//! L27 选将美化 Batch B 技能 7–16 与 2 个辅助子技能步进（`l27_batch_b_tw.rs`）。
//!
//! 覆盖项：
//! 7. `twxiafeng`（TW 牛辅董翓·黠凤，`6350`）
//! 8. `twxingzhui`（TW 张宁·星坠，`6479`）
//! 9. `xingchong`（滕公主·幸宠，`6633`）
//! 10. `dcxiaoxi`（DC 牛辅·骁袭，`6774`）
//! 11. `twfengji`（TW 张曼成·蜂集，`6889`）
//! 12. `twharvestinori`（丰祈，`7044`）
//! 13. `twzhouhu`（咒护，`7168`）
//! 14. `twzuhuo`（阻祸，`7293`）
//! 15. `twzhouzu`（咒诅，`7441`）
//! 16. 辅助子技能：`jishe2`（`1959`）、`new_zhixi_recover`（`1802`）

use super::*;
use crate::catalog::Suit;
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};

pub(super) fn step_twxiafeng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.card == Some(16) {
                set_flag(ctx.core, owner, "twxiafeng_x", 0);
                set_flag(ctx.core, owner, "twxiafeng_rem", 0);
                set_flag(ctx.core, owner, "l27_twxiafeng_spent", 0);
                return Flow::Done;
            }
            if baonvezhi_count(ctx.core, owner) == 0 {
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectNature, ASK_TWXIAFENG_COUNT)
        }
        HostStage::SelectNature => {
            let max_m = baonvezhi_count(ctx.core, owner).min(3);
            let x = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(n)) => u32::from(n).clamp(1, max_m.max(1)),
                _ => return Flow::Done,
            };
            let rem = baonvezhi_count(ctx.core, owner).saturating_sub(x);
            set_flag(ctx.core, owner, "baonvezhi", rem);
            set_flag(ctx.core, owner, "baonvezhi_marks", rem);
            set_flag(ctx.core, owner, "twxiafeng_x", x);
            set_flag(ctx.core, owner, "twxiafeng_rem", x);
            set_flag(ctx.core, owner, "l27_twxiafeng_spent", x);
            set_flag(ctx.core, owner, "l27_b_ask_id", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_twxingzhui(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.card == Some(16) {
                let rem = flag(ctx.core, owner, "twxingzhui_rem");
                if rem > 1 {
                    set_flag(ctx.core, owner, "twxingzhui_rem", rem - 1);
                    return Flow::Done;
                }
                set_flag(ctx.core, owner, "twxingzhui_rem", 0);
                let x = flag(ctx.core, owner, "twxingzhui_x").clamp(1, 3);
                let mut revealed = Vec::new();
                for _ in 0..(2 * x) {
                    if let Some(c) = ctx.core.deck.pop() {
                        ctx.core.table.push(c);
                        revealed.push(c);
                    }
                }
                m.cards = revealed;
                let has_black = m.cards.iter().any(|&c| {
                    ctx.catalog
                        .cards
                        .get(c as usize)
                        .is_some_and(|cd| matches!(cd.suit, Suit::Spade | Suit::Club))
                });
                if has_black && !alive_others(ctx.core, owner).is_empty() {
                    return wait_ask(m, ctx, HostStage::TargetChoice, ASK_TWXINGZHUI_TARGET);
                }
                for &c in &m.cards {
                    if let Some(pos) = ctx.core.table.iter().position(|&x| x == c) {
                        ctx.core.table.remove(pos);
                        ctx.core.discard.push(c);
                    }
                }
                set_flag(ctx.core, owner, "twxingzhui_x", 0);
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectNature, ASK_TWXINGZHUI_TURNS)
        }
        HostStage::SelectNature => {
            let x = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(n)) => u32::from(n).clamp(1, 3),
                _ => return Flow::Done,
            };
            set_flag(ctx.core, owner, "twxingzhui_rem", x);
            set_flag(ctx.core, owner, "twxingzhui_x", x);
            ctx.emit(EventData::LoseHp {
                player: owner,
                amount: 1,
            });
            set_flag(ctx.core, owner, "l27_b_ask_id", 0);
            Flow::Done
        }
        HostStage::TargetChoice => {
            let t = match ctx.answer().first().copied() {
                Some(ChoiceOption::Player(p)) => p,
                _ => alive_others(ctx.core, owner).first().copied().unwrap_or(owner),
            };
            let x = flag(ctx.core, owner, "twxingzhui_x").clamp(1, 3);
            set_flag(ctx.core, owner, "twxingzhui_x", 0);
            let mut black_count = 0u32;
            for &c in &m.cards {
                if let Some(pos) = ctx.core.table.iter().position(|&id| id == c) {
                    ctx.core.table.remove(pos);
                }
                let is_black = ctx
                    .catalog
                    .cards
                    .get(c as usize)
                    .is_some_and(|cd| matches!(cd.suit, Suit::Spade | Suit::Club));
                if is_black {
                    black_count += 1;
                    ctx.core.player_mut(t).hand.push(c);
                } else {
                    ctx.core.discard.push(c);
                }
            }
            if black_count >= x && ctx.core.player(t).alive {
                emit_damage(ctx, Some(owner), t, x as i32, DamageNature::Thunder);
            }
            set_flag(ctx.core, owner, "l27_b_ask_id", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_xingchong(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.card == Some(25) {
                let mut k = 0u32;
                for &c in &m.cards {
                    let key = format!("xingchong_card:{c}");
                    if flag(ctx.core, owner, &key) > 0 {
                        set_flag(ctx.core, owner, &key, 0);
                        k += 1;
                    }
                }
                if k > 0 {
                    draw_direct(ctx, owner, 2 * k);
                }
                return Flow::Done;
            }
            let old_keys: Vec<String> = ctx
                .core
                .vm_aux
                .flags
                .keys()
                .filter_map(|(p, k)| {
                    (*p == owner && k.starts_with("xingchong_card:")).then(|| k.clone())
                })
                .collect();
            for k in old_keys {
                ctx.core.vm_aux.flags.remove(&(owner, k));
            }
            set_flag(ctx.core, owner, "xingchong_tagged_count", 0);
            set_flag(
                ctx.core,
                owner,
                "xingchong_round_used",
                ctx.core.turn.round.max(1),
            );
            wait_ask(m, ctx, HostStage::SelectNature, ASK_XINGCHONG_DRAW)
        }
        HostStage::SelectNature => {
            let limit = (ctx.core.player(owner).max_hp as u32).min(5);
            let x = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(n)) => u32::from(n).min(limit),
                _ => return Flow::Done,
            };
            if x > 0 {
                draw_direct(ctx, owner, x);
            }
            let rem = limit.saturating_sub(x);
            if rem > 0 && !ctx.core.player(owner).hand.is_empty() {
                set_flag(ctx.core, owner, "xingchong_show_max", rem);
                return wait_ask(m, ctx, HostStage::SelectCard, ASK_XINGCHONG_SHOW);
            }
            set_flag(ctx.core, owner, "l27_b_ask_id", 0);
            Flow::Done
        }
        HostStage::SelectCard => {
            let mut tagged = 0u32;
            for opt in ctx.answer().clone() {
                if let ChoiceOption::Card(c) = opt {
                    set_flag(ctx.core, owner, &format!("xingchong_card:{c}"), 1);
                    tagged += 1;
                }
            }
            set_flag(ctx.core, owner, "xingchong_tagged_count", tagged);
            set_flag(ctx.core, owner, "xingchong_show_max", 0);
            set_flag(ctx.core, owner, "l27_b_ask_id", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_dcxiaoxi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if ctx.core.player(owner).max_hp <= 1 {
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectNature, ASK_DCXIAOXI_COUNT)
        }
        HostStage::SelectNature => {
            let max_lose = ctx.core.player(owner).max_hp.saturating_sub(1).min(2) as u32;
            let x = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(n)) => u32::from(n).clamp(1, max_lose.max(1)),
                _ => 1,
            };
            let p = ctx.core.player_mut(owner);
            p.max_hp = p.max_hp.saturating_sub(x).max(1);
            if p.hp > p.max_hp as i32 {
                p.hp = p.max_hp as i32;
            }
            set_flag(ctx.core, owner, "dcxiaoxi_x", x);
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_DCXIAOXI_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, owner, "l27_b_ask_id", 0);
                return Flow::Done;
            };
            m.target = Some(t);
            wait_ask(m, ctx, HostStage::SelectCard, ASK_DCXIAOXI_MODE)
        }
        HostStage::SelectCard => {
            let mode = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(n)) => n & 1,
                _ => 0,
            };
            let t = m.target.unwrap_or(owner);
            let x = flag(ctx.core, owner, "dcxiaoxi_x").clamp(1, 2) as usize;
            set_flag(ctx.core, owner, "dcxiaoxi_x", 0);
            if mode == 0 {
                let he: Vec<CardId> = hand_and_equips(ctx.core, t).into_iter().take(x).collect();
                move_cards_to_hand(ctx, t, owner, &he);
            } else {
                for _ in 0..x {
                    if ctx.core.player(t).alive {
                        emit_damage(ctx, Some(owner), t, 1, DamageNature::Normal);
                    }
                }
            }
            set_flag(ctx.core, owner, "l27_b_ask_id", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_twfengji(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.card == Some(1) {
                let rem = flag(ctx.core, owner, "twfengji_rem");
                if rem > 1 {
                    set_flag(ctx.core, owner, "twfengji_rem", rem - 1);
                    return Flow::Done;
                }
                set_flag(ctx.core, owner, "twfengji_rem", 0);
                let x = flag(ctx.core, owner, "twfengji_x").clamp(1, 3) as usize;
                set_flag(ctx.core, owner, "twfengji_x", 0);
                let exp = ctx
                    .core
                    .vm_aux
                    .piles
                    .remove(&(owner, "twfengji".to_string()))
                    .unwrap_or_default();
                let target_kind = exp.first().map(|&c| ctx.catalog.card_kind(c));
                for c in exp {
                    ctx.core.discard.push(c);
                }
                let mut gained = Vec::new();
                if let Some(tk) = target_kind {
                    let mut i = ctx.core.deck.len();
                    while i > 0 && gained.len() < x {
                        i -= 1;
                        let c = ctx.core.deck[i];
                        if ctx.catalog.card_kind(c) == tk {
                            ctx.core.deck.remove(i);
                            gained.push(c);
                        }
                    }
                    let mut j = ctx.core.discard.len();
                    while j > 0 && gained.len() < x {
                        j -= 1;
                        let c = ctx.core.discard[j];
                        if ctx.catalog.card_kind(c) == tk {
                            ctx.core.discard.remove(j);
                            gained.push(c);
                        }
                    }
                }
                while gained.len() < x {
                    if let Some(c) = ctx.core.deck.pop() {
                        gained.push(c);
                    } else {
                        break;
                    }
                }
                ctx.core.player_mut(owner).hand.extend(gained);
                set_flag(ctx.core, owner, "twfengji_kind", 0);
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectCard, ASK_TWFENGJI_CARD)
        }
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.card = Some(c);
            wait_ask(m, ctx, HostStage::SelectNature, ASK_TWFENGJI_TURNS)
        }
        HostStage::SelectNature => {
            let x = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(n)) => u32::from(n).clamp(1, 3),
                _ => 1,
            };
            if let Some(c) = m.card {
                ctx.take_owned_card(owner, c);
                ctx.core
                    .vm_aux
                    .piles
                    .entry((owner, "twfengji".to_string()))
                    .or_default()
                    .push(c);
                set_flag(
                    ctx.core,
                    owner,
                    "twfengji_kind",
                    (ctx.catalog.card_kind(c) as u32) + 1,
                );
            }
            set_flag(ctx.core, owner, "twfengji_rem", x);
            set_flag(ctx.core, owner, "twfengji_x", x);
            set_flag(ctx.core, owner, "l27_b_ask_id", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_twharvestinori(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.card == Some(16) {
                let rem = flag(ctx.core, owner, "twharvestinori_rem");
                if rem > 1 {
                    set_flag(ctx.core, owner, "twharvestinori_rem", rem - 1);
                    return Flow::Done;
                }
                set_flag(ctx.core, owner, "twharvestinori_rem", 0);
                let x = flag(ctx.core, owner, "twharvestinori_x").clamp(1, 3);
                set_flag(ctx.core, owner, "twharvestinori_x", 0);
                draw_direct(ctx, owner, 2 * x);
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectCard, ASK_TWHARVESTINORI_CARD)
        }
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.card = Some(c);
            wait_ask(m, ctx, HostStage::SelectNature, ASK_TWHARVESTINORI_TURNS)
        }
        HostStage::SelectNature => {
            let x = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(n)) => u32::from(n).clamp(1, 3),
                _ => 1,
            };
            if let Some(c) = m.card {
                discard_cards(ctx, owner, &[c]);
            }
            set_flag(ctx.core, owner, "twharvestinori_rem", x);
            set_flag(ctx.core, owner, "twharvestinori_x", x);
            set_flag(ctx.core, owner, "l27_b_ask_id", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_twzhouhu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.card == Some(16) {
                let rem = flag(ctx.core, owner, "twzhouhu_rem");
                if rem > 1 {
                    set_flag(ctx.core, owner, "twzhouhu_rem", rem - 1);
                    return Flow::Done;
                }
                set_flag(ctx.core, owner, "twzhouhu_rem", 0);
                let x = flag(ctx.core, owner, "twzhouhu_x").clamp(1, 3);
                set_flag(ctx.core, owner, "twzhouhu_x", 0);
                ctx.recover(owner, x, None);
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectCard, ASK_TWZHOUHU_CARD)
        }
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.card = Some(c);
            wait_ask(m, ctx, HostStage::SelectNature, ASK_TWZHOUHU_TURNS)
        }
        HostStage::SelectNature => {
            let x = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(n)) => u32::from(n).clamp(1, 3),
                _ => 1,
            };
            if let Some(c) = m.card {
                discard_cards(ctx, owner, &[c]);
            }
            set_flag(ctx.core, owner, "twzhouhu_rem", x);
            set_flag(ctx.core, owner, "twzhouhu_x", x);
            set_flag(ctx.core, owner, "l27_b_ask_id", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_twzuhuo(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.card == Some(16) {
                let rem = flag(ctx.core, owner, "twzuhuo_rem");
                if rem > 1 {
                    set_flag(ctx.core, owner, "twzuhuo_rem", rem - 1);
                    return Flow::Done;
                }
                set_flag(ctx.core, owner, "twzuhuo_rem", 0);
                let x = flag(ctx.core, owner, "twzuhuo_x").clamp(1, 3);
                set_flag(ctx.core, owner, "twzuhuo_x", 0);
                let shields = flag(ctx.core, owner, "twzuhuo_shields")
                    .max(flag(ctx.core, owner, "l27_twzuhuo_shield"))
                    + x;
                set_flag(ctx.core, owner, "twzuhuo_shields", shields);
                set_flag(ctx.core, owner, "l27_twzuhuo_shield", shields);
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectCard, ASK_TWZUHUO_CARD)
        }
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.card = Some(c);
            wait_ask(m, ctx, HostStage::SelectNature, ASK_TWZUHUO_TURNS)
        }
        HostStage::SelectNature => {
            let x = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(n)) => u32::from(n).clamp(1, 3),
                _ => 1,
            };
            if let Some(c) = m.card {
                discard_cards(ctx, owner, &[c]);
            }
            set_flag(ctx.core, owner, "twzuhuo_rem", x);
            set_flag(ctx.core, owner, "twzuhuo_x", x);
            set_flag(ctx.core, owner, "l27_b_ask_id", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_twzhouzu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.card == Some(16) {
                let rem = flag(ctx.core, owner, "twzhouzu_rem");
                if rem > 1 {
                    set_flag(ctx.core, owner, "twzhouzu_rem", rem - 1);
                    return Flow::Done;
                }
                set_flag(ctx.core, owner, "twzhouzu_rem", 0);
                let x = flag(ctx.core, owner, "twzhouzu_x").clamp(1, 3) as usize;
                set_flag(ctx.core, owner, "twzhouzu_x", 0);
                let t_enc = flag(ctx.core, owner, "twzhouzu_target");
                set_flag(ctx.core, owner, "twzhouzu_target", 0);
                if t_enc > 0 {
                    let t = (t_enc - 1) as PlayerId;
                    if ctx.core.player(t).alive {
                        let to_disc: Vec<CardId> =
                            hand_and_equips(ctx.core, t).into_iter().take(x).collect();
                        discard_cards(ctx, t, &to_disc);
                        emit_damage(ctx, Some(owner), t, 1, DamageNature::Thunder);
                    }
                }
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_TWZHOUZU_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.target = Some(t);
            wait_ask(m, ctx, HostStage::SelectNature, ASK_TWZHOUZU_TURNS)
        }
        HostStage::SelectNature => {
            let x = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(n)) => u32::from(n).clamp(1, 3),
                _ => 1,
            };
            if let Some(t) = m.target {
                set_flag(ctx.core, owner, "twzhouzu_target", u32::from(t) + 1);
            }
            set_flag(ctx.core, owner, "twzhouzu_rem", x);
            set_flag(ctx.core, owner, "twzhouzu_x", x);
            set_flag(ctx.core, owner, "l27_b_ask_id", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_jishe2(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(16) {
        set_flag(ctx.core, owner, "jishe2", 0);
        set_flag(ctx.core, owner, "l27_jishe_limit_minus", 0);
    } else {
        let cur = flag(ctx.core, owner, "jishe2")
            .max(flag(ctx.core, owner, "l27_jishe_limit_minus"))
            + 1;
        set_flag(ctx.core, owner, "jishe2", cur);
        set_flag(ctx.core, owner, "l27_jishe_limit_minus", cur);
    }
    Flow::Done
}

pub(super) fn step_new_zhixi_recover(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    set_flag(ctx.core, owner, "new_zhixi", 0);
    set_flag(ctx.core, owner, "new_zhixi_mark2", 0);
    for key in ["new_zhixi", "new_zhixi_mark2"] {
        if let Some(def) = ctx.catalog.skill_by_name(key) {
            ctx.core.vm_aux.marks.remove(&(owner, u32::from(def)));
        }
    }
    Flow::Done
}

pub(super) fn step_xinfu_xionghuo(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if flag(ctx.core, owner, "xinfu_xionghuo_init") == 0 {
        set_flag(ctx.core, owner, "xinfu_xionghuo_init", 1);
        if flag(ctx.core, owner, "xinfu_xionghuo_marks") == 0 {
            set_flag(ctx.core, owner, "xinfu_xionghuo_marks", 3);
            set_flag(ctx.core, owner, "xionghuo", 3);
        }
    }
    match m.stage {
        HostStage::Act => {
            if m.card == Some(20) {
                return Flow::Done;
            }
            if m.card == Some(16) {
                let p = m.target.unwrap_or(owner);
                set_flag(ctx.core, p, "xinfu_xionghuo_no_sha_turn", 0);
                set_flag(ctx.core, p, "xinfu_xionghuo_hand_minus", 0);
                return Flow::Done;
            }
            if m.card == Some(4) {
                let t = m.target.unwrap_or(owner);
                set_flag(ctx.core, t, "xinfu_xionghuo_marked", 0);
                set_flag(ctx.core, t, "xionghuo", 0);
                let forced = flag(ctx.core, owner, "xinfu_xionghuo_force_mode");
                let mode = if forced > 0 {
                    (forced - 1) % 3
                } else {
                    ctx.core.rng.gen_range(3)
                };
                set_flag(ctx.core, t, "xinfu_xionghuo_last_effect", mode + 1);
                match mode {
                    0 => {
                        set_flag(ctx.core, t, "xinfu_xionghuo_no_sha_turn", 1);
                        emit_damage(ctx, Some(owner), t, 1, DamageNature::Fire);
                    }
                    1 => {
                        set_flag(ctx.core, t, "xinfu_xionghuo_hand_minus", 2);
                        ctx.emit(EventData::LoseHp {
                            player: t,
                            amount: 1,
                        });
                    }
                    _ => {
                        let mut steal = Vec::new();
                        if let Some(&hc) = ctx.core.player(t).hand.first() {
                            steal.push(hc);
                        }
                        if let Some(&ec) = ctx.core.player(t).equips.first() {
                            steal.push(ec);
                        }
                        move_cards_to_hand(ctx, t, owner, &steal);
                    }
                }
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_XIONGHUO_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let rem = flag(ctx.core, owner, "xinfu_xionghuo_marks").saturating_sub(1);
            set_flag(ctx.core, owner, "xinfu_xionghuo_marks", rem);
            set_flag(ctx.core, owner, "xionghuo", rem);
            set_flag(ctx.core, t, "xinfu_xionghuo_marked", 1);
            set_flag(ctx.core, t, "xionghuo", 1);
            set_flag(ctx.core, owner, "l27_b_ask_id", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_mbwuling(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.card == Some(1) {
                let t = m.target.unwrap_or(owner);
                let cur = flag(ctx.core, t, "mbwuling_spirit");
                if cur == 0 {
                    return Flow::Done;
                }
                let next = (cur % 5) + 1;
                set_flag(ctx.core, t, "mbwuling_spirit", next);
                apply_mbwuling_spirit_entry(ctx, t, next);
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_MBWULING_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.target = Some(t);
            wait_ask(m, ctx, HostStage::SelectNature, ASK_MBWULING_ORDER)
        }
        HostStage::SelectNature => {
            let tag = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(n)) => u32::from(n).min(4),
                _ => 0,
            };
            let t = m.target.unwrap_or(owner);
            let stamp = turn_stamp(ctx.core);
            let used = mbwuling_used_turn(ctx.core, owner) + 1;
            set_flag(ctx.core, owner, "mbwuling_turn_stamp", stamp);
            set_flag(ctx.core, owner, "mbwuling_used_turn", used);
            let spirit = tag + 1;
            set_flag(ctx.core, t, "mbwuling_spirit", spirit);
            apply_mbwuling_spirit_entry(ctx, t, spirit);
            set_flag(ctx.core, owner, "l27_b_ask_id", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn apply_mbwuling_spirit_entry(ctx: &mut Ctx<'_>, t: PlayerId, spirit: u32) {
    match spirit {
        2 => {
            let judges = std::mem::take(&mut ctx.core.player_mut(t).judge);
            for c in judges {
                ctx.core.discard.push(c);
            }
            if ctx.core.player(t).hp < ctx.core.player(t).max_hp as i32 {
                ctx.recover(t, 1, None);
            }
        }
        4 => draw_direct(ctx, t, 1),
        5 => draw_direct(ctx, t, 2),
        _ => {}
    }
}

pub(super) fn step_sbqingzheng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_SBQINGZHENG_SUITS),
        HostStage::SelectNature => {
            let mut suit_mask = 0u8;
            for opt in ctx.answer() {
                if let ChoiceOption::Tag(s) = *opt {
                    suit_mask |= 1u8 << (s & 3);
                }
            }
            if suit_mask == 0 {
                return Flow::Done;
            }
            let to_disc: Vec<CardId> = ctx
                .core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| {
                    ctx.catalog
                        .cards
                        .get(c as usize)
                        .is_some_and(|cd| (suit_mask & (1u8 << suit_tag_of(cd.suit))) != 0)
                })
                .collect();
            let self_cnt = to_disc.len() as u32;
            discard_cards(ctx, owner, &to_disc);
            set_flag(ctx.core, owner, "sbqingzheng_self_discarded", self_cnt);
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_SBQINGZHENG_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, owner, "l27_b_ask_id", 0);
                return Flow::Done;
            };
            m.target = Some(t);
            wait_ask(m, ctx, HostStage::SelectCard, ASK_SBQINGZHENG_TARGET_SUIT)
        }
        HostStage::SelectCard => {
            let s_tag = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(n)) => n & 3,
                _ => 0,
            };
            let t = m.target.unwrap_or(owner);
            let mut t_disc: Vec<CardId> = ctx
                .core
                .player(t)
                .hand
                .iter()
                .copied()
                .filter(|&c| {
                    ctx.catalog
                        .cards
                        .get(c as usize)
                        .is_some_and(|cd| suit_tag_of(cd.suit) == s_tag)
                })
                .collect();
            if t_disc.is_empty() {
                if let Some(&first) = ctx.core.player(t).hand.first() {
                    let fs = ctx.catalog.cards.get(first as usize).map(|cd| cd.suit);
                    t_disc = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .filter(|&c| ctx.catalog.cards.get(c as usize).map(|cd| cd.suit) == fs)
                        .collect();
                }
            }
            let target_cnt = t_disc.len() as u32;
            discard_cards(ctx, t, &t_disc);
            set_flag(ctx.core, owner, "sbqingzheng_target_discarded", target_cnt);
            let self_cnt = flag(ctx.core, owner, "sbqingzheng_self_discarded");
            if self_cnt > target_cnt {
                emit_damage(ctx, Some(owner), t, 1, DamageNature::Normal);
                let marks = flag(ctx.core, owner, "sbjianxiong_marks")
                    .max(flag(ctx.core, owner, "sbjianxiong"));
                if marks < 2 {
                    return wait_ask(m, ctx, HostStage::CardsAsk, ASK_SBQINGZHENG_GAIN_MARK);
                }
            }
            set_flag(ctx.core, owner, "l27_b_ask_id", 0);
            Flow::Done
        }
        HostStage::CardsAsk => {
            if !ctx.answer().is_empty()
                && !matches!(ctx.answer().first(), Some(ChoiceOption::Tag(0)))
            {
                let marks = (flag(ctx.core, owner, "sbjianxiong_marks")
                    .max(flag(ctx.core, owner, "sbjianxiong"))
                    + 1)
                    .min(2);
                set_flag(ctx.core, owner, "sbjianxiong_marks", marks);
                set_flag(ctx.core, owner, "sbjianxiong", marks);
            }
            set_flag(ctx.core, owner, "l27_b_ask_id", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_spyanjiao(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.card == Some(1) {
                let n = flag(ctx.core, owner, "spyanjiao_draw_pending");
                set_flag(ctx.core, owner, "spyanjiao_draw_pending", 0);
                if n > 0 {
                    draw_direct(ctx, owner, n);
                }
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectNature, ASK_SPYANJIAO_SUIT)
        }
        HostStage::SelectNature => {
            let Some(ChoiceOption::Tag(s)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.card = Some(u32::from(s & 3));
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_SPYANJIAO_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                set_flag(ctx.core, owner, "l27_b_ask_id", 0);
                return Flow::Done;
            };
            let s_tag = (m.card.unwrap_or(0) as u8) & 3;
            let mut give: Vec<CardId> = ctx
                .core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| {
                    ctx.catalog
                        .cards
                        .get(c as usize)
                        .is_some_and(|cd| suit_tag_of(cd.suit) == s_tag)
                })
                .collect();
            if give.is_empty() {
                if let Some(&first) = ctx.core.player(owner).hand.first() {
                    give.push(first);
                }
            }
            let n = give.len() as u32;
            if n > 0 {
                move_cards_to_hand(ctx, owner, t, &give);
                emit_damage(ctx, Some(owner), t, 1, DamageNature::Normal);
                let pending = flag(ctx.core, owner, "spyanjiao_draw_pending") + n;
                set_flag(ctx.core, owner, "spyanjiao_draw_pending", pending);
            }
            set_flag(ctx.core, owner, "l27_b_ask_id", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}
