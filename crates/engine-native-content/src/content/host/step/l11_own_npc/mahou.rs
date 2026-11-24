//! L11-6a 太虚自有 NPC 魔法倒计时类技能：`txhj_zhouzu`/`txhj_yaoshu`/`txhj_zhoufa`/`txhj_renfang`。

use super::*;
use crate::choice::ChoiceOption;
use crate::event::DamageNature;

pub(super) fn handle_mahou(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Option<Flow> {
    match m.def_key.as_str() {
        "txhj_zhouzu" => Some(handle_zhouzu(m, ctx)),
        "txhj_yaoshu" => Some(handle_yaoshu(m, ctx)),
        "txhj_zhoufa" => Some(handle_zhoufa(m, ctx)),
        "txhj_renfang" => Some(handle_renfang(m, ctx)),
        _ => None,
    }
}

fn read_duration(ctx: &Ctx<'_>) -> Option<u32> {
    match ctx.answer().first().copied() {
        Some(ChoiceOption::Tag(i)) if i < 3 => Some(u32::from(i) + 1),
        _ => None,
    }
}

fn handle_zhouzu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(MODE_TICK) {
        let rem = flag(ctx.core, owner, "l11_mahou_zhouzu_rem");
        if rem == 0 {
            return Flow::Done;
        }
        if rem > 1 {
            set_flag(ctx.core, owner, "l11_mahou_zhouzu_rem", rem - 1);
            return Flow::Done;
        }
        let orig = flag(ctx.core, owner, "l11_mahou_zhouzu_orig");
        let tgt_raw = flag(ctx.core, owner, "l11_mahou_zhouzu_tgt");
        set_flag(ctx.core, owner, "l11_mahou_zhouzu_rem", 0);
        set_flag(ctx.core, owner, "l11_mahou_zhouzu_orig", 0);
        set_flag(ctx.core, owner, "l11_mahou_zhouzu_tgt", 0);
        let Some(t) = tgt_raw.checked_sub(1).and_then(|v| u8::try_from(v).ok()) else {
            return Flow::Done;
        };
        if !ctx.core.player(t).alive {
            return Flow::Done;
        }
        m.target = Some(t);
        set_flag(ctx.core, owner, "l11_mahou_zhouzu_disc_n", orig);
        if he_cards(ctx.core, t).is_empty() {
            set_flag(ctx.core, owner, "l11_mahou_zhouzu_disc_n", 0);
            emit_damage(ctx, Some(owner), t, 1, DamageNature::Thunder, None);
            return Flow::Yield;
        }
        return wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_ZHOUZU_DISCARD);
    }
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_ZHOUZU_TARGET),
        HostStage::TargetChoice => {
            m.card = None;
            let Some(t) = answered_players(ctx).into_iter().next().filter(|&p| p != owner && ctx.core.player(p).alive) else {
                return Flow::Done;
            };
            m.target = Some(t);
            wait_npc_ask(m, ctx, HostStage::SelectNature, ASK_ZHOUZU_DUR)
        }
        HostStage::SelectNature => {
            m.card = None;
            let Some(dur) = read_duration(ctx) else {
                return Flow::Done;
            };
            let Some(t) = m.target else {
                return Flow::Done;
            };
            ctx.core.vm_aux.inc_once(owner, 1, "txhj_zhouzu");
            set_flag(ctx.core, owner, "l11_mahou_zhouzu_orig", dur);
            set_flag(ctx.core, owner, "l11_mahou_zhouzu_rem", dur);
            set_flag(ctx.core, owner, "l11_mahou_zhouzu_tgt", u32::from(t) + 1);
            ctx.log(LogEvent::SkillTriggered {
                skill: "咒诅",
                owner,
                presentation: None,
            });
            Flow::Done
        }
        HostStage::SelectCard => {
            m.card = None;
            set_flag(ctx.core, owner, "l11_mahou_zhouzu_disc_n", 0);
            let Some(t) = m.target else {
                return Flow::Done;
            };
            let picked: Vec<CardId> = answered_cards(ctx)
                .into_iter()
                .filter(|c| ctx.core.player(t).hand.contains(c) || ctx.core.player(t).equips.contains(c))
                .collect();
            if !picked.is_empty() {
                discard_player_cards(ctx, t, &picked);
            }
            if ctx.core.player(t).alive {
                m.stage = HostStage::DrawSettle;
                emit_damage(ctx, Some(owner), t, 1, DamageNature::Thunder, None);
                return Flow::Yield;
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn handle_yaoshu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_YAOSHU_TARGET),
        HostStage::TargetChoice => {
            m.card = None;
            let Some(t) = answered_players(ctx).into_iter().next().filter(|&p| {
                p != owner && ctx.core.player(p).alive && flag(ctx.core, p, "l11_yaoshu_mahou_rem") == 0
            }) else {
                return Flow::Done;
            };
            m.target = Some(t);
            wait_npc_ask(m, ctx, HostStage::SelectNature, ASK_YAOSHU_DUR)
        }
        HostStage::SelectNature => {
            m.card = None;
            let Some(dur) = read_duration(ctx) else {
                return Flow::Done;
            };
            let Some(t) = m.target else {
                return Flow::Done;
            };
            ctx.core.vm_aux.inc_once(owner, 1, "txhj_yaoshu");
            set_flag(ctx.core, t, "l11_yaoshu_mahou_orig", dur);
            set_flag(ctx.core, t, "l11_yaoshu_mahou_rem", dur);
            ctx.log(LogEvent::SkillTriggered {
                skill: "妖术",
                owner,
                presentation: None,
            });
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn handle_zhoufa(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(MODE_TICK) {
        let rem = flag(ctx.core, owner, "l11_mahou_zhoufa_rem");
        if rem == 0 {
            return Flow::Done;
        }
        if rem > 1 {
            set_flag(ctx.core, owner, "l11_mahou_zhoufa_rem", rem - 1);
            return Flow::Done;
        }
        let orig = flag(ctx.core, owner, "l11_mahou_zhoufa_orig");
        let tgt_raw = flag(ctx.core, owner, "l11_mahou_zhoufa_tgt");
        set_flag(ctx.core, owner, "l11_mahou_zhoufa_rem", 0);
        set_flag(ctx.core, owner, "l11_mahou_zhoufa_orig", 0);
        set_flag(ctx.core, owner, "l11_mahou_zhoufa_tgt", 0);
        if let Some(t) = tgt_raw.checked_sub(1).and_then(|v| u8::try_from(v).ok()) {
            if ctx.core.player(t).alive && orig > 0 {
                ctx.log(LogEvent::SkillTriggered {
                    skill: "咒法",
                    owner,
                    presentation: None,
                });
                emit_damage(ctx, Some(owner), t, orig, DamageNature::Thunder, None);
            }
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_ZHOUFA_TARGET),
        HostStage::TargetChoice => {
            m.card = None;
            let Some(t) = answered_players(ctx).into_iter().next().filter(|&p| p != owner && ctx.core.player(p).alive) else {
                return Flow::Done;
            };
            m.target = Some(t);
            wait_npc_ask(m, ctx, HostStage::SelectNature, ASK_ZHOUFA_DUR)
        }
        HostStage::SelectNature => {
            m.card = None;
            let Some(dur) = read_duration(ctx) else {
                return Flow::Done;
            };
            let Some(t) = m.target else {
                return Flow::Done;
            };
            ctx.core.vm_aux.inc_once(owner, 1, "txhj_zhoufa");
            set_flag(ctx.core, owner, "l11_mahou_zhoufa_orig", dur);
            set_flag(ctx.core, owner, "l11_mahou_zhoufa_rem", dur);
            set_flag(ctx.core, owner, "l11_mahou_zhoufa_tgt", u32::from(t) + 1);
            ctx.log(LogEvent::SkillTriggered {
                skill: "咒法",
                owner,
                presentation: None,
            });
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn handle_renfang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(MODE_TICK) {
        let rem = flag(ctx.core, owner, "l11_mahou_renfang_rem");
        if rem == 0 {
            return Flow::Done;
        }
        if rem > 1 {
            set_flag(ctx.core, owner, "l11_mahou_renfang_rem", rem - 1);
            return Flow::Done;
        }
        let orig = flag(ctx.core, owner, "l11_mahou_renfang_orig");
        set_flag(ctx.core, owner, "l11_mahou_renfang_rem", 0);
        set_flag(ctx.core, owner, "l11_mahou_renfang_orig", 0);
        if orig == 0 {
            return Flow::Done;
        }
        set_flag(ctx.core, owner, "l11_mahou_renfang_active_cnt", orig);
        m.cards.clear();
        m.targets.clear();
        ctx.log(LogEvent::SkillTriggered {
            skill: "人方",
            owner,
            presentation: None,
        });
        return wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_RENFANG_TARGET);
    }
    match m.stage {
        HostStage::Act => {
            let pile = ctx
                .core
                .vm_aux
                .piles
                .get_mut(&(owner, "txhj_renwang".to_string()));
            let Some(pile) = pile.filter(|p| p.len() >= 36) else {
                return Flow::Done;
            };
            let spent: Vec<CardId> = pile.drain(..36).collect();
            for &c in &spent {
                ctx.to_discard(c);
            }
            (ctx.catalog.hooks.on_cards_to_discard)(ctx, owner, &spent, false);
            ctx.core.vm_aux.inc_once(owner, 1, "txhj_renfang");
            wait_npc_ask(m, ctx, HostStage::SelectNature, ASK_RENFANG_DUR)
        }
        HostStage::SelectNature => {
            m.card = None;
            let Some(dur) = read_duration(ctx) else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "l11_mahou_renfang_orig", dur);
            set_flag(ctx.core, owner, "l11_mahou_renfang_rem", dur);
            Flow::Done
        }
        HostStage::TargetChoice => {
            m.card = None;
            let Some(t) = answered_players(ctx)
                .into_iter()
                .next()
                .filter(|&p| p != owner && ctx.core.player(p).alive && !m.targets.contains(&p))
            else {
                set_flag(ctx.core, owner, "l11_mahou_renfang_active_cnt", 0);
                return Flow::Done;
            };
            m.targets.push(t);
            let left = flag(ctx.core, owner, "l11_mahou_renfang_active_cnt").saturating_sub(1);
            set_flag(ctx.core, owner, "l11_mahou_renfang_active_cnt", left);
            emit_damage(ctx, Some(owner), t, 36, DamageNature::Thunder, None);
            if left > 0 {
                m.stage = HostStage::DrawSettle;
                return Flow::Yield;
            }
            Flow::Done
        }
        HostStage::DrawSettle => {
            let left = flag(ctx.core, owner, "l11_mahou_renfang_active_cnt");
            if left == 0 {
                return Flow::Done;
            }
            let res = wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_RENFANG_TARGET);
            if matches!(res, Flow::Done) {
                set_flag(ctx.core, owner, "l11_mahou_renfang_active_cnt", 0);
            }
            res
        }
        _ => Flow::Done,
    }
}
