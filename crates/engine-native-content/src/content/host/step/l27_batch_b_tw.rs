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
use crate::catalog::{Catalog, Suit};
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::state::GameStateCore;

fn has_active_l11_buff(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    name: &str,
) -> bool {
    catalog
        .skill_by_name(name)
        .is_some_and(|d| core.has_skill(owner, d))
        && flag(core, owner, "mbcmfangzhu_ban") == 0
}

pub(in crate::content::host::step) fn mahou_max_x(core: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> u32 {
    if has_active_l11_buff(core, catalog, owner, "buff_txhj_digongdeyouhu") {
        4
    } else {
        3
    }
}

fn mahou_effective_rem(core: &GameStateCore, catalog: &Catalog, owner: PlayerId, x: u32) -> u32 {
    if x > 1 && has_active_l11_buff(core, catalog, owner, "buff_txhj_tiangongdeyouhu") {
        x - 1
    } else {
        x
    }
}

pub(in crate::content::host::step) fn finish_mahou_cast_stage(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    prefix: &str,
    x: u32,
    next_stage: HostStage,
) -> Flow {
    let owner = m.owner;
    let rem = mahou_effective_rem(ctx.core, ctx.catalog, owner, x);
    set_flag(ctx.core, owner, &format!("{prefix}_rem"), rem);
    set_flag(ctx.core, owner, &format!("{prefix}_x"), x);
    if has_active_l11_buff(ctx.core, ctx.catalog, owner, "buff_txhj_renigongdeyouhu") {
        return wait_ask(m, ctx, next_stage, ASK_TWMAHOU_RENIGONG_CONFIRM);
    }
    set_flag(ctx.core, owner, "l27_b_ask_id", 0);
    Flow::Done
}

pub(in crate::content::host::step) fn finish_mahou_cast(m: &mut HostMachine, ctx: &mut Ctx<'_>, prefix: &str, x: u32) -> Flow {
    finish_mahou_cast_stage(m, ctx, prefix, x, HostStage::DrawSettle)
}

pub(in crate::content::host::step) fn finish_renigong_draw(ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    if matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm)) {
        ctx.start_draw(owner, 1);
    }
    set_flag(ctx.core, owner, "l27_b_ask_id", 0);
    Flow::Done
}

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
                let max_x = mahou_max_x(ctx.core, ctx.catalog, owner);
                let x = flag(ctx.core, owner, "twxingzhui_x").clamp(1, max_x);
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
            let max_x = mahou_max_x(ctx.core, ctx.catalog, owner);
            let x = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(n)) => u32::from(n).clamp(1, max_x),
                _ => return Flow::Done,
            };
            ctx.emit(EventData::LoseHp {
                player: owner,
                amount: 1,
            });
            finish_mahou_cast(m, ctx, "twxingzhui", x)
        }
        HostStage::DrawSettle => finish_renigong_draw(ctx, owner),
        HostStage::TargetChoice => {
            let t = match ctx.answer().first().copied() {
                Some(ChoiceOption::Player(p)) => p,
                _ => alive_others(ctx.core, owner).first().copied().unwrap_or(owner),
            };
            let max_x = mahou_max_x(ctx.core, ctx.catalog, owner);
            let x = flag(ctx.core, owner, "twxingzhui_x").clamp(1, max_x);
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
                let max_x = mahou_max_x(ctx.core, ctx.catalog, owner);
                let x = flag(ctx.core, owner, "twharvestinori_x").clamp(1, max_x);
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
            let max_x = mahou_max_x(ctx.core, ctx.catalog, owner);
            let x = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(n)) => u32::from(n).clamp(1, max_x),
                _ => 1,
            };
            if let Some(c) = m.card {
                discard_cards(ctx, owner, &[c]);
            }
            finish_mahou_cast(m, ctx, "twharvestinori", x)
        }
        HostStage::DrawSettle => finish_renigong_draw(ctx, owner),
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
                let max_x = mahou_max_x(ctx.core, ctx.catalog, owner);
                let x = flag(ctx.core, owner, "twzhouhu_x").clamp(1, max_x);
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
            let max_x = mahou_max_x(ctx.core, ctx.catalog, owner);
            let x = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(n)) => u32::from(n).clamp(1, max_x),
                _ => 1,
            };
            if let Some(c) = m.card {
                discard_cards(ctx, owner, &[c]);
            }
            finish_mahou_cast(m, ctx, "twzhouhu", x)
        }
        HostStage::DrawSettle => finish_renigong_draw(ctx, owner),
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
                let max_x = mahou_max_x(ctx.core, ctx.catalog, owner);
                let x = flag(ctx.core, owner, "twzuhuo_x").clamp(1, max_x);
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
            let max_x = mahou_max_x(ctx.core, ctx.catalog, owner);
            let x = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(n)) => u32::from(n).clamp(1, max_x),
                _ => 1,
            };
            if let Some(c) = m.card {
                discard_cards(ctx, owner, &[c]);
            }
            finish_mahou_cast(m, ctx, "twzuhuo", x)
        }
        HostStage::DrawSettle => finish_renigong_draw(ctx, owner),
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
