//! Phase 3 P3-C1a `L11` 跨包 NPC Part A 主动技执行（前段 12 技：`active_a.rs`）。

use super::asks::*;
use super::helpers::*;
use super::l11_common::*;
use super::*;
use crate::catalog::CardKind;
use crate::event::{CardUseData, DamageNature, EventData};
use crate::ids::PlayerId;
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn handle_active_a(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "baiyi" => step_baiyi(m, ctx),
        "duwu" => step_duwu(m, ctx),
        "fenxun" => step_fenxun(m, ctx),
        "guanzong" => step_guanzong(m, ctx),
        "gushe" => step_gushe(m, ctx),
        "jinglve" => step_jinglve(m, ctx),
        "jujian" => step_jujian(m, ctx),
        "lianzhu" => step_lianzhu(m, ctx),
        "lihun" => step_lihun(m, ctx),
        "mingjian" => step_mingjian(m, ctx),
        "mizhao" => step_mizhao(m, ctx),
        "mouzhu" => step_mouzhu(m, ctx),
        _ => clear_and_done(m),
    }
}

fn step_baiyi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_BAIYI_TARGETS),
        HostStage::TargetChoice => {
            let ts = answered_players(ctx);
            if ts.len() == 2 && ts[0] != ts[1] {
                set_flag(ctx.core, owner, "l11_baiyi_used", 1);
                ctx.core.swap_seat(ts[0], ts[1]);
            }
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}

fn step_duwu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if let Some(dying_target) = ctx.res.stack.iter().rev().find_map(|f| match f.data {
                EventData::DyingSaved { player, .. } => Some(player),
                _ => None,
            }) {
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(owner, format!("l11_duwu_dying_target:{dying_target}")));
                set_flag(ctx.core, owner, "l11_duwu_banned_turn", 1);
                emit_lose_hp(ctx, owner, 1);
                return clear_and_yield(m, HostStage::DrawSettle);
            }
            wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_DUWU_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            m.target = Some(t);
            wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_DUWU_CARDS)
        }
        HostStage::SelectCard => {
            let cs = answered_cards(ctx);
            let Some(t) = m.target else {
                return clear_and_done(m);
            };
            let need = ctx.core.player(t).hp.max(0) as usize;
            if need == 0 || cs.len() != need {
                return clear_and_done(m);
            }
            discard_player_cards(ctx, owner, &cs);
            set_flag(ctx.core, owner, &format!("l11_duwu_dying_target:{t}"), 1);
            emit_damage(ctx, Some(owner), t, 1, DamageNature::Normal, None);
            clear_and_yield(m, HostStage::DrawSettle)
        }
        HostStage::DrawSettle => {
            if let Some(t) = m.target {
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(owner, format!("l11_duwu_dying_target:{t}")));
            }
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}

fn step_fenxun(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_FENXUN_TARGET),
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            m.target = Some(t);
            wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_FENXUN_CARD)
        }
        HostStage::SelectCard => {
            let Some(c) = answered_cards(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            let Some(t) = m.target else {
                return clear_and_done(m);
            };
            mark_skill_used(ctx, owner, "fenxun");
            discard_player_cards(ctx, owner, &[c]);
            set_flag(ctx.core, owner, &format!("l11_mod:fenxun_to:{t}"), 1);
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}

fn step_guanzong(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_GUANZONG_SRC),
        HostStage::TargetChoice => {
            let Some(src) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            m.target = Some(src);
            wait_npc_ask(m, ctx, HostStage::ShuchuiTarget, ASK_GUANZONG_VICTIM)
        }
        HostStage::ShuchuiTarget => {
            let Some(victim) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            let Some(src) = m.target else {
                return clear_and_done(m);
            };
            mark_skill_used(ctx, owner, "guanzong");
            emit_damage(ctx, Some(src), victim, 1, DamageNature::Normal, None);
            clear_and_yield(m, HostStage::DrawSettle)
        }
        HostStage::DrawSettle => clear_and_done(m),
        _ => clear_and_done(m),
    }
}

fn gushe_cleanup(ctx: &mut Ctx<'_>, owner: PlayerId) {
    for k in ["l11_gushe_idx", "l11_gushe_num1", "l11_gushe_owner_done"] {
        ctx.core.vm_aux.flags.remove(&(owner, k.to_string()));
    }
}

fn gushe_step_loop(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let idx = flag(ctx.core, owner, "l11_gushe_idx") as usize;
    if idx >= m.targets.len() || !ctx.core.player(owner).alive {
        gushe_cleanup(ctx, owner);
        return clear_and_done(m);
    }
    let t = m.targets[idx];
    let Some(&tcard) = m.cards.get(1 + idx) else {
        gushe_cleanup(ctx, owner);
        return clear_and_done(m);
    };
    let num1 = flag(ctx.core, owner, "l11_gushe_num1");
    let num2 = u32::from(effective_card_rank(ctx.catalog, tcard));
    if num1 <= num2 && flag(ctx.core, owner, "l11_gushe_owner_done") == 0 {
        set_flag(ctx.core, owner, "l11_gushe_owner_done", 1);
        let marks = inc_flag(ctx.core, owner, "l11_gushe_marks", 1);
        if marks >= 7 {
            gushe_cleanup(ctx, owner);
            emit_lose_hp(ctx, owner, 1);
            return clear_and_yield(m, HostStage::DrawSettle);
        }
        return wait_npc_ask(m, ctx, HostStage::JiefenOwnerDiscard, ASK_GUSHE_OWNER_CHOICE);
    }
    set_flag(ctx.core, owner, "l11_gushe_idx", (idx + 1) as u32);
    set_flag(ctx.core, owner, "l11_gushe_owner_done", 0);
    if num1 >= num2 && ctx.core.player(t).alive {
        m.target = Some(t);
        if !he_cards(ctx.core, t).is_empty() {
            return wait_npc_ask(m, ctx, HostStage::JiefenTargetDiscard, ASK_GUSHE_TGT_CHOICE);
        }
        ctx.start_draw(owner, 1);
        return clear_and_yield(m, HostStage::ShuchuiTargetLoop);
    }
    gushe_step_loop(m, ctx)
}

fn step_gushe(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_GUSHE_TARGETS),
        HostStage::TargetChoice => {
            let ts = answered_players(ctx);
            if ts.is_empty() || ts.len() > 3 {
                return clear_and_done(m);
            }
            mark_skill_used(ctx, owner, "gushe");
            m.targets = ts;
            m.cards.clear();
            wait_npc_ask(m, ctx, HostStage::ComparePlayerPick, ASK_GUSHE_OWNER_PINDIAN)
        }
        HostStage::ComparePlayerPick => {
            let Some(oc) = answered_cards(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            m.cards = vec![oc];
            m.target = m.targets.first().copied();
            wait_npc_ask(m, ctx, HostStage::CompareTargetPick, ASK_GUSHE_TGT_PINDIAN)
        }
        HostStage::CompareTargetPick => {
            let Some(tc) = answered_cards(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            m.cards.push(tc);
            let done_targets = m.cards.len().saturating_sub(1);
            if done_targets < m.targets.len() {
                m.target = Some(m.targets[done_targets]);
                return wait_npc_ask(m, ctx, HostStage::CompareTargetPick, ASK_GUSHE_TGT_PINDIAN);
            }
            discard_player_cards(ctx, owner, &[m.cards[0]]);
            for i in 0..m.targets.len() {
                if let Some(&c) = m.cards.get(1 + i) {
                    discard_player_cards(ctx, m.targets[i], &[c]);
                }
            }
            ctx.out.push(LogEvent::CardsRevealed {
                cards: m.cards.clone(),
                to: ctx.core.alive_players().collect(),
            });
            let num1 = u32::from(effective_card_rank(ctx.catalog, m.cards[0]));
            set_flag(ctx.core, owner, "l11_jici_compare_num1", num1);
            set_flag(ctx.core, owner, "l11_gushe_num1", num1);
            set_flag(ctx.core, owner, "l11_gushe_idx", 0);
            set_flag(ctx.core, owner, "l11_gushe_owner_done", 0);
            let marks = flag(ctx.core, owner, "l11_gushe_marks");
            if marks > 0
                && num1 <= marks
                && has_npc_a_skill(ctx.core, ctx.catalog, owner, "jici")
                && is_unblocked_npc_a(ctx.core, ctx.catalog, owner)
            {
                return wait_npc_ask(m, ctx, HostStage::SelectNature, ASK_JICI_CONFIRM);
            }
            gushe_step_loop(m, ctx)
        }
        HostStage::SelectNature => {
            if is_confirmed(ctx) {
                let marks = flag(ctx.core, owner, "l11_gushe_marks");
                let num1 = flag(ctx.core, owner, "l11_gushe_num1");
                if num1 < marks {
                    set_flag(ctx.core, owner, "l11_gushe_num1", num1 + marks);
                } else {
                    refund_once_used(ctx, owner, "gushe");
                }
            }
            gushe_step_loop(m, ctx)
        }
        HostStage::JiefenOwnerDiscard => {
            if let Some(c) = answered_cards(ctx).into_iter().next() {
                discard_player_cards(ctx, owner, &[c]);
                gushe_step_loop(m, ctx)
            } else {
                ctx.start_draw(owner, 1);
                clear_and_yield(m, HostStage::ShuchuiTargetLoop)
            }
        }
        HostStage::JiefenTargetDiscard => {
            let Some(t) = m.target else {
                return gushe_step_loop(m, ctx);
            };
            if let Some(c) = answered_cards(ctx).into_iter().next() {
                discard_player_cards(ctx, t, &[c]);
                gushe_step_loop(m, ctx)
            } else {
                ctx.start_draw(owner, 1);
                clear_and_yield(m, HostStage::ShuchuiTargetLoop)
            }
        }
        HostStage::ShuchuiTargetLoop => gushe_step_loop(m, ctx),
        HostStage::DrawSettle => clear_and_done(m),
        _ => clear_and_done(m),
    }
}

fn step_jinglve(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_JINGLVE_TARGET),
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            m.target = Some(t);
            wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_JINGLVE_CARD)
        }
        HostStage::SelectCard => {
            let Some(c) = answered_cards(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            let Some(t) = m.target else {
                return clear_and_done(m);
            };
            mark_skill_used(ctx, owner, "jinglve");
            set_flag(ctx.core, t, &format!("l11_jinglve_sishi_card:{c}"), 1);
            set_flag(ctx.core, t, &format!("l11_jinglve_sishi_from:{owner}"), c + 1);
            set_flag(ctx.core, owner, "l11_jinglve_active_target", u32::from(t) + 1);
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}

fn step_jujian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_JUJIAN_CARDS),
        HostStage::SelectCard => {
            let cs = answered_cards(ctx);
            if cs.is_empty() || cs.len() > 3 {
                return clear_and_done(m);
            }
            m.cards = cs;
            wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_JUJIAN_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            mark_skill_used(ctx, owner, "jujian");
            let n = m.cards.len();
            let same_cat = n >= 3 && {
                let first = card_category(ctx.catalog.card_kind(m.cards[0]));
                m.cards
                    .iter()
                    .all(|&c| card_category(ctx.catalog.card_kind(c)) == first)
            };
            discard_player_cards(ctx, owner, &m.cards);
            m.target = Some(u32::from(same_cat) as PlayerId);
            if ctx.core.player(t).alive {
                ctx.start_draw(t, n as u32);
                clear_and_yield(m, HostStage::DrawSettle)
            } else {
                if same_cat && ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                    emit_recover(ctx, owner, 1);
                    return clear_and_yield(m, HostStage::ShuchuiTarget);
                }
                clear_and_done(m)
            }
        }
        HostStage::DrawSettle => {
            if m.target == Some(1)
                && ctx.core.player(owner).alive
                && ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32
            {
                emit_recover(ctx, owner, 1);
                return clear_and_yield(m, HostStage::ShuchuiTarget);
            }
            clear_and_done(m)
        }
        HostStage::ShuchuiTarget => clear_and_done(m),
        _ => clear_and_done(m),
    }
}

fn step_lianzhu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_LIANZHU_CARD),
        HostStage::SelectCard => {
            let Some(c) = answered_cards(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            m.cards = vec![c];
            wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_LIANZHU_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            let Some(&c) = m.cards.first() else {
                return clear_and_done(m);
            };
            mark_skill_used(ctx, owner, "lianzhu");
            let is_black = !is_red_card(ctx.catalog, ctx.core, owner, c);
            ctx.out.push(LogEvent::CardsRevealed {
                cards: vec![c],
                to: ctx.core.alive_players().collect(),
            });
            move_cards_to_hand(ctx, owner, t, &[c]);
            if is_black && ctx.core.player(t).alive {
                m.target = Some(t);
                if he_cards(ctx.core, t).len() >= 2 {
                    return wait_npc_ask(m, ctx, HostStage::JiefenTargetDiscard, ASK_LIANZHU_DISCARD);
                }
                ctx.start_draw(owner, 2);
                return clear_and_yield(m, HostStage::DrawSettle);
            }
            clear_and_done(m)
        }
        HostStage::JiefenTargetDiscard => {
            let cs = answered_cards(ctx);
            let Some(t) = m.target else {
                return clear_and_done(m);
            };
            if cs.len() == 2 {
                discard_player_cards(ctx, t, &cs);
                clear_and_done(m)
            } else {
                ctx.start_draw(owner, 2);
                clear_and_yield(m, HostStage::DrawSettle)
            }
        }
        HostStage::DrawSettle => clear_and_done(m),
        _ => clear_and_done(m),
    }
}

fn step_lihun(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if let Some(&phase_code) = m.cards.first() {
                if (phase_code & 0xff) == 4 && (phase_code >> 8) == 0 {
                    return super::phase_turn::step_lihun_phase_end(m, ctx);
                }
            }
            wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_LIHUN_CARD)
        }
        HostStage::SelectCard => {
            let Some(c) = answered_cards(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            m.cards = vec![c];
            wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_LIHUN_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            let Some(&c) = m.cards.first() else {
                return clear_and_done(m);
            };
            mark_skill_used(ctx, owner, "lihun");
            set_flag(ctx.core, owner, "l11_lihun_used_play", 1);
            discard_player_cards(ctx, owner, &[c]);
            set_flag(ctx.core, owner, "l11_lihun_target", u32::from(t) + 1);
            emit_turn_over(ctx, owner);
            let th = ctx.core.player(t).hand.clone();
            if !th.is_empty() && ctx.core.player(owner).alive {
                move_cards_to_hand(ctx, t, owner, &th);
            }
            clear_and_yield(m, HostStage::DrawSettle)
        }
        HostStage::DrawSettle => clear_and_done(m),
        HostStage::JiefenOwnerDiscard => super::phase_turn::step_lihun_phase_end(m, ctx),
        _ => clear_and_done(m),
    }
}

fn step_mingjian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_MINGJIAN_TARGET),
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            mark_skill_used(ctx, owner, "mingjian");
            let hs = ctx.core.player(owner).hand.clone();
            move_cards_to_hand(ctx, owner, t, &hs);
            inc_flag(ctx.core, t, "l11_mingjian_pending", 1);
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}

fn step_mizhao(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_MIZHAO_T1),
        HostStage::TargetChoice => {
            let Some(t1) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            m.target = Some(t1);
            m.targets = vec![t1];
            wait_npc_ask(m, ctx, HostStage::ShuchuiTarget, ASK_MIZHAO_T2)
        }
        HostStage::ShuchuiTarget => {
            let Some(t2) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            let t1 = m.targets[0];
            m.targets.push(t2);
            mark_skill_used(ctx, owner, "mizhao");
            let hs = ctx.core.player(owner).hand.clone();
            if !hs.is_empty() {
                move_cards_to_hand(ctx, owner, t1, &hs);
            }
            if ctx.core.player(t1).hand.is_empty() || ctx.core.player(t2).hand.is_empty() {
                return clear_and_done(m);
            }
            wait_npc_ask(m, ctx, HostStage::ComparePlayerPick, ASK_MIZHAO_P1_CARD)
        }
        HostStage::ComparePlayerPick => {
            let Some(c1) = answered_cards(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            m.cards = vec![c1];
            wait_npc_ask(m, ctx, HostStage::CompareTargetPick, ASK_MIZHAO_P2_CARD)
        }
        HostStage::CompareTargetPick => {
            let Some(c2) = answered_cards(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            let c1 = m.cards[0];
            let (p1, p2) = (m.targets[0], m.targets[1]);
            discard_player_cards(ctx, p1, &[c1]);
            discard_player_cards(ctx, p2, &[c2]);
            ctx.out.push(LogEvent::CardsRevealed {
                cards: vec![c1, c2],
                to: ctx.core.alive_players().collect(),
            });
            let r1 = effective_card_rank(ctx.catalog, c1);
            let r2 = effective_card_rank(ctx.catalog, c2);
            if r1 != r2 {
                let (win, lose) = if r1 > r2 { (p1, p2) } else { (p2, p1) };
                if ctx.core.player(win).alive && ctx.core.player(lose).alive {
                    ctx.emit(EventData::CardUsed(CardUseData {
                        card: None,
                        player: win,
                        targets: vec![lose],
                        nature: DamageNature::Normal,
                        use_as: Some(CardKind::Sha),
                        cancelled: false,
                    }));
                    return clear_and_yield(m, HostStage::DrawSettle);
                }
            }
            clear_and_done(m)
        }
        HostStage::DrawSettle => clear_and_done(m),
        _ => clear_and_done(m),
    }
}

fn step_mouzhu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_MOUZHU_TARGET),
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            mark_skill_used(ctx, owner, "mouzhu");
            m.target = Some(t);
            wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_MOUZHU_GIVE)
        }
        HostStage::SelectCard => {
            let Some(c) = answered_cards(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            let Some(t) = m.target else {
                return clear_and_done(m);
            };
            move_cards_to_hand(ctx, t, owner, &[c]);
            if ctx.core.player(owner).alive
                && ctx.core.player(t).alive
                && ctx.core.player(owner).hand.len() > ctx.core.player(t).hand.len()
            {
                wait_npc_ask(m, ctx, HostStage::SelectNature, ASK_MOUZHU_MODE)
            } else {
                clear_and_done(m)
            }
        }
        HostStage::SelectNature => {
            let mode = answered_tag(ctx).unwrap_or(0);
            let Some(t) = m.target else {
                return clear_and_done(m);
            };
            let kind = if mode == 1 { CardKind::Duel } else { CardKind::Sha };
            ctx.emit(EventData::CardUsed(CardUseData {
                card: None,
                player: t,
                targets: vec![owner],
                nature: DamageNature::Normal,
                use_as: Some(kind),
                cancelled: false,
            }));
            clear_and_yield(m, HostStage::DrawSettle)
        }
        HostStage::DrawSettle => clear_and_done(m),
        _ => clear_and_done(m),
    }
}
