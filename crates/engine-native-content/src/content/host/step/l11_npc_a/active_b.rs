//! Phase 3 P3-C1a `L11` 跨包 NPC Part A 主动技执行（后段 17 技：`active_b.rs`）。

use super::asks::*;
use super::helpers::*;
use super::l11_common::*;
use super::*;
use crate::catalog::{CardKind, EquipSlot};
use crate::event::{CardUseData, DamageNature, EventData};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn handle_active_b(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "new_reqingnang" => step_new_reqingnang(m, ctx),
        "new_yijue" => step_new_yijue(m, ctx),
        "nzry_chenglve" => step_nzry_chenglve(m, ctx),
        "nzry_feijun" => step_nzry_feijun(m, ctx),
        "quji" => step_quji(m, ctx),
        "refenxun" => step_refenxun(m, ctx),
        "reganlu" => step_reganlu(m, ctx),
        "reguose" => step_reguose(m, ctx),
        "rejunxing" => step_rejunxing(m, ctx),
        "reluanji" => step_reluanji(m, ctx),
        "rewurong" => step_rewurong(m, ctx),
        "rezhijian" => step_rezhijian(m, ctx),
        "rongbei" => step_rongbei(m, ctx),
        "kyou_zhidian" => step_kyou_zhidian(m, ctx),
        "dclbjiuxian" => step_dclbjiuxian(m, ctx),
        "olhuoji" => step_olhuoji(m, ctx),
        "reshuangxiong" => step_reshuangxiong(m, ctx),
        _ => clear_and_done(m),
    }
}

fn step_new_reqingnang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_NEW_REQINGNANG_CARD),
        HostStage::SelectCard => {
            let Some(c) = answered_cards(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            m.cards = vec![c];
            wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_NEW_REQINGNANG_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            let Some(&c) = m.cards.first() else {
                return clear_and_done(m);
            };
            let black = !is_red_card(ctx.catalog, ctx.core, owner, c);
            discard_player_cards(ctx, owner, &[c]);
            set_flag(ctx.core, t, "l11_new_reqingnang_t", 1);
            if black {
                set_flag(ctx.core, owner, "l11_new_reqingnang_ban", 1);
            }
            emit_recover(ctx, t, 1);
            clear_and_yield(m, HostStage::DrawSettle)
        }
        HostStage::DrawSettle => clear_and_done(m),
        _ => clear_and_done(m),
    }
}

fn step_new_yijue(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_NEW_YIJUE_CARD),
        HostStage::SelectCard => {
            let Some(c) = answered_cards(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            m.cards = vec![c];
            wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_NEW_YIJUE_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            let Some(&c) = m.cards.first() else {
                return clear_and_done(m);
            };
            mark_skill_used(ctx, owner, "new_yijue");
            discard_player_cards(ctx, owner, &[c]);
            m.target = Some(t);
            wait_npc_ask(m, ctx, HostStage::SelectNature, ASK_NEW_YIJUE_SHOW)
        }
        HostStage::SelectNature => {
            let Some(shown) = answered_cards(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            let Some(t) = m.target else {
                return clear_and_done(m);
            };
            ctx.out.push(LogEvent::CardsRevealed {
                cards: vec![shown],
                to: ctx.core.alive_players().collect(),
            });
            if !is_red_card(ctx.catalog, ctx.core, t, shown) {
                set_flag(ctx.core, t, "l11_new_yijue_hand_ban", 1);
                if flag(ctx.core, t, "xj_fengyin_sealed") == 0 {
                    set_flag(ctx.core, t, "xj_fengyin_sealed", 1);
                    set_flag(ctx.core, t, "l11_yijue_set_xj", 1);
                }
                set_flag(ctx.core, owner, &format!("l11_yijue_heart_target:{t}"), 1);
                return clear_and_done(m);
            }
            move_cards_to_hand(ctx, t, owner, &[shown]);
            if ctx.core.player(t).alive && ctx.core.player(t).hp < ctx.core.player(t).max_hp as i32
            {
                wait_npc_ask(m, ctx, HostStage::JiefenTarget, ASK_NEW_YIJUE_HEAL)
            } else {
                clear_and_done(m)
            }
        }
        HostStage::JiefenTarget => {
            if is_confirmed(ctx) {
                if let Some(t) = m.target {
                    emit_recover(ctx, t, 1);
                    return clear_and_yield(m, HostStage::DrawSettle);
                }
            }
            clear_and_done(m)
        }
        HostStage::DrawSettle => clear_and_done(m),
        _ => clear_and_done(m),
    }
}

fn step_nzry_chenglve(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            mark_skill_used(ctx, owner, "nzry_chenglve");
            let is_yin = flag(ctx.core, owner, "l11_chenglve_yin") > 0;
            set_flag(ctx.core, owner, "l11_chenglve_yin", u32::from(!is_yin));
            let (draw_n, disc_n) = if is_yin { (2u32, 1u32) } else { (1u32, 2u32) };
            m.cards = vec![disc_n];
            ctx.start_draw(owner, draw_n);
            clear_and_yield(m, HostStage::DrawSettle)
        }
        HostStage::DrawSettle => {
            if ctx.core.player(owner).hand.is_empty() {
                return clear_and_done(m);
            }
            wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_CHENGLVE_DISCARD)
        }
        HostStage::SelectCard => {
            let cs = answered_cards(ctx);
            let mut mask = flag(ctx.core, owner, "l11_nzry_chenglve_suits");
            for &c in &cs {
                mask |= suit_bit(card_suit(ctx.catalog, ctx.core, owner, c));
            }
            if !cs.is_empty() {
                discard_player_cards(ctx, owner, &cs);
            }
            set_flag(ctx.core, owner, "l11_nzry_chenglve_suits", mask);
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}

fn step_nzry_feijun(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_FEIJUN_CARD),
        HostStage::SelectCard => {
            let Some(c) = answered_cards(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            mark_skill_used(ctx, owner, "nzry_feijun");
            discard_player_cards(ctx, owner, &[c]);
            let my_h = ctx.core.player(owner).hand.len();
            let my_e = ctx.core.player(owner).equips.len();
            let has_target = alive_others(ctx.core, owner).into_iter().any(|t| {
                ctx.core.player(t).hand.len() > my_h || ctx.core.player(t).equips.len() > my_e
            });
            if !has_target {
                return clear_and_done(m);
            }
            wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_FEIJUN_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            m.target = Some(t);
            if !ctx.core.player(t).alive {
                return clear_and_done(m);
            }
            let h_ok = ctx.core.player(t).hand.len() > ctx.core.player(owner).hand.len();
            let e_ok = ctx.core.player(t).equips.len() > ctx.core.player(owner).equips.len();
            if h_ok && e_ok {
                wait_npc_ask(m, ctx, HostStage::SelectNature, ASK_FEIJUN_MODE)
            } else if h_ok || e_ok {
                m.cards = vec![u32::from(!h_ok)];
                wait_npc_ask(
                    m,
                    ctx,
                    HostStage::JiefenTargetDiscard,
                    ASK_FEIJUN_TARGET_CARD,
                )
            } else {
                clear_and_done(m)
            }
        }
        HostStage::SelectNature => {
            let mode = answered_tag(ctx).unwrap_or(0);
            m.cards = vec![u32::from(mode)];
            wait_npc_ask(
                m,
                ctx,
                HostStage::JiefenTargetDiscard,
                ASK_FEIJUN_TARGET_CARD,
            )
        }
        HostStage::JiefenTargetDiscard => {
            let Some(tc) = answered_cards(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            let Some(t) = m.target else {
                return clear_and_done(m);
            };
            if m.cards.first().copied() == Some(1) {
                discard_player_cards(ctx, t, &[tc]);
            } else {
                move_cards_to_hand(ctx, t, owner, &[tc]);
            }
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}

fn step_quji(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_QUJI_CARDS),
        HostStage::SelectCard => {
            let cs = answered_cards(ctx);
            let x = (ctx.core.player(owner).max_hp as i32 - ctx.core.player(owner).hp).max(0)
                as usize;
            if x == 0 || cs.len() != x {
                return clear_and_done(m);
            }
            m.cards = cs;
            wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_QUJI_TARGETS)
        }
        HostStage::TargetChoice => {
            let ts = answered_players(ctx);
            if ts.is_empty() {
                return clear_and_done(m);
            }
            mark_skill_used(ctx, owner, "quji");
            let has_black = m
                .cards
                .iter()
                .any(|&c| !is_red_card(ctx.catalog, ctx.core, owner, c));
            discard_player_cards(ctx, owner, &m.cards);
            for &t in ts.iter().rev() {
                emit_recover(ctx, t, 1);
            }
            m.target = Some(u32::from(has_black) as PlayerId);
            clear_and_yield(m, HostStage::DrawSettle)
        }
        HostStage::DrawSettle => {
            if m.target == Some(1) && ctx.core.player(owner).alive {
                m.target = None;
                emit_lose_hp(ctx, owner, 1);
                return clear_and_yield(m, HostStage::ShuchuiTarget);
            }
            clear_and_done(m)
        }
        HostStage::ShuchuiTarget => clear_and_done(m),
        _ => clear_and_done(m),
    }
}

fn step_refenxun(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if let Some(&phase_code) = m.cards.first() {
                if (phase_code & 0xff) == 6 && (phase_code >> 8) > 0 {
                    return super::phase_turn::step_refenxun_end(m, ctx);
                }
            }
            wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_REFENXUN_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            mark_skill_used(ctx, owner, "refenxun");
            set_flag(ctx.core, owner, "l11_refenxun_used_play", 1);
            set_flag(ctx.core, owner, &format!("l11_mod:fenxun_to:{t}"), 1);
            clear_and_done(m)
        }
        HostStage::JiefenOwnerDiscard => super::phase_turn::step_refenxun_end(m, ctx),
        _ => clear_and_done(m),
    }
}

fn step_reganlu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_REGANLU_T1),
        HostStage::TargetChoice => {
            let Some(t1) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            m.target = Some(t1);
            wait_npc_ask(m, ctx, HostStage::ShuchuiTarget, ASK_REGANLU_T2)
        }
        HostStage::ShuchuiTarget => {
            let Some(t2) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            let Some(t1) = m.target else {
                return clear_and_done(m);
            };
            mark_skill_used(ctx, owner, "reganlu");
            swap_equip_zones(ctx, t1, t2);
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}

fn step_reguose(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_REGUOSE_CARD),
        HostStage::SelectCard => {
            let Some(c) = answered_cards(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            m.cards = vec![c];
            wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_REGUOSE_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            let Some(&c) = m.cards.first() else {
                return clear_and_done(m);
            };
            mark_skill_used(ctx, owner, "reguose");
            let existing_le = ctx
                .core
                .player(t)
                .judge
                .iter()
                .copied()
                .find(|&j| ctx.catalog.resolved_card_kind(ctx.core, j) == CardKind::Le);
            if let Some(le_c) = existing_le {
                discard_player_cards(ctx, owner, &[c]);
                discard_player_cards(ctx, t, &[le_c]);
                ctx.start_draw(owner, 1);
                return clear_and_yield(m, HostStage::DrawSettle);
            }
            ctx.emit(EventData::CardUsed(CardUseData {
                card: Some(c),
                player: owner,
                targets: vec![t],
                nature: DamageNature::Normal,
                use_as: Some(CardKind::Le),
                cancelled: false,
            }));
            clear_and_yield(m, HostStage::SelectNature)
        }
        HostStage::SelectNature => {
            if ctx.core.player(owner).alive {
                ctx.start_draw(owner, 1);
                return clear_and_yield(m, HostStage::DrawSettle);
            }
            clear_and_done(m)
        }
        HostStage::DrawSettle => clear_and_done(m),
        _ => clear_and_done(m),
    }
}

fn step_rejunxing(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_REJUNXING_CARDS),
        HostStage::SelectCard => {
            let cs = answered_cards(ctx);
            if cs.is_empty() {
                return clear_and_done(m);
            }
            m.cards = cs;
            wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_REJUNXING_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            mark_skill_used(ctx, owner, "rejunxing");
            discard_player_cards(ctx, owner, &m.cards);
            m.target = Some(t);
            if !ctx.core.player(t).alive {
                return clear_and_done(m);
            }
            wait_npc_ask(m, ctx, HostStage::SelectNature, ASK_REJUNXING_MODE)
        }
        HostStage::SelectNature => {
            let mode = answered_tag(ctx).unwrap_or(1);
            let Some(t) = m.target else {
                return clear_and_done(m);
            };
            let x = m.cards.len();
            if mode == 0 && he_cards(ctx.core, t).len() >= x {
                return wait_npc_ask(m, ctx, HostStage::JiefenTargetDiscard, ASK_REJUNXING_DISCARD);
            }
            emit_turn_over(ctx, t);
            ctx.start_draw(t, x as u32);
            clear_and_yield(m, HostStage::DrawSettle)
        }
        HostStage::JiefenTargetDiscard => {
            let cs = answered_cards(ctx);
            let Some(t) = m.target else {
                return clear_and_done(m);
            };
            if cs.len() == m.cards.len() {
                discard_player_cards(ctx, t, &cs);
                emit_lose_hp(ctx, t, 1);
                clear_and_yield(m, HostStage::DrawSettle)
            } else {
                emit_turn_over(ctx, t);
                ctx.start_draw(t, m.cards.len() as u32);
                clear_and_yield(m, HostStage::DrawSettle)
            }
        }
        HostStage::DrawSettle => clear_and_done(m),
        _ => clear_and_done(m),
    }
}

fn step_reluanji(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_RELUANJI_CARDS),
        HostStage::SelectCard => {
            let cs = answered_cards(ctx);
            if cs.len() != 2 || cs[0] == cs[1] {
                return clear_and_done(m);
            }
            let used = flag(ctx.core, owner, "l11_reluanji_suits_turn");
            let b0 = suit_bit(card_suit(ctx.catalog, ctx.core, owner, cs[0]));
            let b1 = suit_bit(card_suit(ctx.catalog, ctx.core, owner, cs[1]));
            if (used & b0) != 0 || (used & b1) != 0 {
                return clear_and_done(m);
            }
            set_flag(ctx.core, owner, "l11_reluanji_suits_turn", used | b0 | b1);
            for &c in &cs {
                consume_he_card_for_virtual_use(ctx, owner, c);
            }
            let targets: Vec<PlayerId> = ctx
                .core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| {
                    p != owner
                        && ctx.core.player(p).alive
                        && ctx.catalog.ow_target_enabled(
                            ctx.core,
                            owner,
                            p,
                            None,
                            CardKind::ArcheryAttack,
                            DamageNature::Normal,
                        )
                })
                .collect();
            let next_fid = ctx.res.next_event_id;
            set_flag(ctx.core, owner, &format!("l11_reluanji_fid:{next_fid}"), 1);
            ctx.emit(EventData::CardUsed(CardUseData {
                card: None,
                player: owner,
                targets,
                nature: DamageNature::Normal,
                use_as: Some(CardKind::ArcheryAttack),
                cancelled: false,
            }));
            clear_and_yield(m, HostStage::DrawSettle)
        }
        HostStage::DrawSettle => clear_and_done(m),
        _ => clear_and_done(m),
    }
}

fn step_rewurong(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_REWURONG_TARGET),
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            mark_skill_used(ctx, owner, "rewurong");
            m.target = Some(t);
            wait_npc_ask(m, ctx, HostStage::ComparePlayerPick, ASK_REWURONG_OWNER_SHOW)
        }
        HostStage::ComparePlayerPick => {
            let Some(oc) = answered_cards(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            m.cards = vec![oc];
            wait_npc_ask(m, ctx, HostStage::CompareTargetPick, ASK_REWURONG_TGT_SHOW)
        }
        HostStage::CompareTargetPick => {
            let Some(tc) = answered_cards(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            let Some(t) = m.target else {
                return clear_and_done(m);
            };
            let oc = m.cards[0];
            ctx.out.push(LogEvent::CardsRevealed {
                cards: vec![oc, tc],
                to: ctx.core.alive_players().collect(),
            });
            let o_sha = ctx.catalog.resolved_card_kind(ctx.core, oc).is_sha_like();
            let t_shan = ctx.catalog.resolved_card_kind(ctx.core, tc) == CardKind::Shan;
            if o_sha && !t_shan {
                emit_damage(ctx, Some(owner), t, 1, DamageNature::Normal, None);
                return clear_and_yield(m, HostStage::DrawSettle);
            }
            if !o_sha && t_shan && !he_cards(ctx.core, t).is_empty() {
                return wait_npc_ask(m, ctx, HostStage::JiefenTargetDiscard, ASK_REWURONG_GAIN);
            }
            clear_and_done(m)
        }
        HostStage::JiefenTargetDiscard => {
            if let Some(c) = answered_cards(ctx).into_iter().next() {
                if let Some(t) = m.target {
                    move_cards_to_hand(ctx, t, owner, &[c]);
                }
            }
            clear_and_done(m)
        }
        HostStage::DrawSettle => clear_and_done(m),
        _ => clear_and_done(m),
    }
}

fn step_rezhijian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act
        && ctx
            .res
            .stack
            .last()
            .is_some_and(|f| matches!(f.data, EventData::CardUsed(_)))
    {
        return super::card_use_b::step_rezhijian_draw(m, ctx);
    }
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_REZHIJIAN_CARD),
        HostStage::SelectCard => {
            let Some(c) = answered_cards(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            m.cards = vec![c];
            wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_REZHIJIAN_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            let Some(&cid) = m.cards.first() else {
                return clear_and_done(m);
            };
            if !ctx.core.player(owner).hand.contains(&cid) {
                return clear_and_done(m);
            }
            ctx.take_hand_card(owner, cid);
            ctx.note_cards_lost(owner, &[cid]);
            equip_card_direct(ctx, t, cid);
            m.target = Some(t);
            clear_and_yield(m, HostStage::DrawSettle)
        }
        HostStage::DrawSettle => {
            if ctx.core.player(owner).alive {
                ctx.start_draw(owner, 1);
                return clear_and_yield(m, HostStage::ShuchuiTarget);
            }
            clear_and_done(m)
        }
        HostStage::ShuchuiTarget => clear_and_done(m),
        HostStage::JiefenTarget => super::card_use_b::step_rezhijian_draw(m, ctx),
        _ => clear_and_done(m),
    }
}

fn step_rongbei(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_RONGBEI_TARGET),
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            set_flag(ctx.core, owner, "l11_rongbei_used", 1);
            let slots = [
                EquipSlot::Weapon,
                EquipSlot::Armor,
                EquipSlot::HorsePlus,
                EquipSlot::HorseMinus,
                EquipSlot::Treasure,
            ];
            let mut equipped: Vec<CardId> = Vec::new();
            for sl in slots {
                if !ctx.catalog.is_equip_slot_enabled(ctx.core, t, sl)
                    || ctx
                        .core
                        .player(t)
                        .equips
                        .iter()
                        .any(|&e| ctx.catalog.card_slot_of(ctx.core, e) == Some(sl))
                {
                    continue;
                }
                let found = if let Some(pos) = ctx
                    .core
                    .deck
                    .iter()
                    .rposition(|&c| ctx.catalog.card_slot_of(ctx.core, c) == Some(sl))
                {
                    Some(ctx.core.deck.remove(pos))
                } else if let Some(pos) = ctx
                    .core
                    .discard
                    .iter()
                    .position(|&c| ctx.catalog.card_slot_of(ctx.core, c) == Some(sl))
                {
                    Some(ctx.core.discard.remove(pos))
                } else {
                    None
                };
                if let Some(cid) = found {
                    equipped.push(cid);
                }
            }
            for cid in equipped {
                equip_card_direct(ctx, t, cid);
            }
            if !ctx.res.stack.is_empty() {
                clear_and_yield(m, HostStage::DrawSettle)
            } else {
                clear_and_done(m)
            }
        }
        HostStage::DrawSettle => clear_and_done(m),
        _ => clear_and_done(m),
    }
}

fn step_kyou_zhidian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::SelectNature
        || (m.stage == HostStage::Act
            && ctx
                .res
                .stack
                .last()
                .is_some_and(|f| matches!(f.data, EventData::CardUsed(_))))
    {
        return super::card_use_b::step_kyou_zhidian_aim(m, ctx);
    }
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_ZHIDIAN_ACT_CARD),
        HostStage::SelectCard => {
            let Some(c) = answered_cards(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            m.cards = vec![c];
            wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_ZHIDIAN_ACT_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            let Some(&c) = m.cards.first() else {
                return clear_and_done(m);
            };
            if ctx.core.turn.sha_used_play_phase {
                let (_, unlim) = crate::rvm_bridge::modifiers::eval_sha_use_limit(
                    ctx.core,
                    ctx.catalog,
                    owner,
                    Some(c),
                    CardKind::Sha,
                    DamageNature::Normal,
                    1,
                );
                if !unlim {
                    inc_flag(ctx.core, owner, "vm_sha_extra_used", 1);
                }
            }
            ctx.emit(EventData::CardUsed(CardUseData {
                card: Some(c),
                player: owner,
                targets: vec![t],
                nature: DamageNature::Normal,
                use_as: Some(CardKind::Sha),
                cancelled: false,
            }));
            clear_and_yield(m, HostStage::DrawSettle)
        }
        HostStage::DrawSettle => clear_and_done(m),
        _ => clear_and_done(m),
    }
}

fn step_dclbjiuxian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_JIUXIAN_ACT_CARD),
        HostStage::SelectCard => {
            let Some(c) = answered_cards(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            ctx.emit(EventData::CardUsed(CardUseData {
                card: Some(c),
                player: owner,
                targets: Vec::new(),
                nature: DamageNature::Normal,
                use_as: Some(CardKind::Jiu),
                cancelled: false,
            }));
            clear_and_yield(m, HostStage::DrawSettle)
        }
        HostStage::DrawSettle => clear_and_done(m),
        _ => clear_and_done(m),
    }
}

fn step_olhuoji(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_OLHUOJI_ACT_CARD),
        HostStage::SelectCard => {
            let Some(c) = answered_cards(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            m.cards = vec![c];
            wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_OLHUOJI_ACT_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            let Some(&c) = m.cards.first() else {
                return clear_and_done(m);
            };
            consume_he_card_for_virtual_use(ctx, owner, c);
            set_flag(ctx.core, owner, VIRTUAL_HUOGONG_FLAG, 1);
            set_flag(ctx.core, owner, "l11_olhuoji_active", 1);
            ctx.emit(EventData::CardUsed(CardUseData {
                card: None,
                player: owner,
                targets: vec![t],
                nature: DamageNature::Normal,
                use_as: Some(CardKind::Wuzhong),
                cancelled: false,
            }));
            clear_and_yield(m, HostStage::DrawSettle)
        }
        HostStage::DrawSettle => clear_and_done(m),
        _ => clear_and_done(m),
    }
}

fn step_reshuangxiong(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if (m.stage == HostStage::Act
        && ctx.res.stack.last().is_some_and(|f| {
            matches!(
                f.data,
                EventData::DrawPhase { .. }
                    | EventData::Damage(_)
                    | EventData::DamageWindow { .. }
                    | EventData::CardResponded { .. }
            )
        }))
        || matches!(
            m.stage,
            HostStage::JiefenTarget | HostStage::HuanrenPick | HostStage::CardsAsk
        )
    {
        return super::phase_turn::step_reshuangxiong_trigger(m, ctx);
    }
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_SHUANGXIONG_ACT_CARD),
        HostStage::SelectCard => {
            let Some(c) = answered_cards(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            m.cards = vec![c];
            wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_SHUANGXIONG_ACT_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            let Some(&c) = m.cards.first() else {
                return clear_and_done(m);
            };
            ctx.emit(EventData::CardUsed(CardUseData {
                card: Some(c),
                player: owner,
                targets: vec![t],
                nature: DamageNature::Normal,
                use_as: Some(CardKind::Duel),
                cancelled: false,
            }));
            clear_and_yield(m, HostStage::DrawSettle)
        }
        HostStage::DrawSettle => clear_and_done(m),
        _ => clear_and_done(m),
    }
}
