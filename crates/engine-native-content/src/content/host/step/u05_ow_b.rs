//! U05-3 (`shaai-ow`) Batch B（标签 1041）：莱因哈特 / 托比昂 / 半藏 / 安娜 / 狂鼠 / 麦克雷 / 美（35 项）。

use super::u05_ow::{
    alive_others, answered_cards, answered_confirm, answered_players, bingqiang_valid_targets,
    claim_off_deck_supply, clear_bingqiang_by_owner, damage_is_sha_like, discard_he_card,
    draw_direct, flag, grant_skill_u05, grant_temp_skill, hand_and_equips, has_skill_u05,
    is_black_card, prev_next_targets_clear_of, random_discard_he, refund_once_used,
    remove_skill_u05, set_flag, suit_of, toggle_turn_over, turn_over_to_back,
    valid_single_sha_target, wait_ow, with_current_card_use_mut, with_current_damage_mut,
    ASK_OW_AQIANGHUA_TARGET, ASK_OW_BAOXUE_TARGETS, ASK_OW_BINGQIANG_CARDS,
    ASK_OW_BINGQIANG_TARGET, ASK_OW_JIDONG_CONFIRM, ASK_OW_LIUDAN_CONFIRM,
    ASK_OW_MAODING2_CARDS, ASK_OW_MAODING2_TARGET, ASK_OW_MAODING_CONFIRM, ASK_OW_MIANZHEN_CARD,
    ASK_OW_MIANZHEN_SHAN, ASK_OW_MIANZHEN_TARGET, ASK_OW_PAOTAI_CARD, ASK_OW_SHANGUANG_CARD,
    ASK_OW_SHANGUANG_TARGET, ASK_OW_SHOUJIA_CARD, ASK_OW_SHOUJIA_TARGET, ASK_OW_TIANDAN_CONFIRM,
    ASK_OW_YINBO_CARD, ASK_OW_ZHIYUAN_CONFIRM, ASK_OW_ZHONGDUN_CARD, ASK_OW_ZHONGDUN_TARGET,
};
use super::*;
use crate::catalog::{CardKind, QueryCtx, Suit, TriggerMeta};
use crate::event::{DamageNature, DamageOutcome, DrawOutcome, EventData, EventKind, Frame, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{damage_data_with, Ctx, Flow, LogEvent};

pub(super) fn trigger_meta_b(name: &str) -> Option<TriggerMeta> {
    let forced = |event: EventKind, timing: Timing, priority: i32, extra: Vec<EventKind>| {
        TriggerMeta {
            event,
            timing,
            priority,
            order_selectable: false,
            optional: false,
            extra_events: extra,
        }
    };
    match name {
        "mengji" => Some(forced(EventKind::DamageBegin, Timing::Normal, 1, Vec::new())),
        "maoding" => Some(forced(EventKind::DamageEnd, Timing::Normal, 1, Vec::new())),
        "paotai" => Some(forced(
            EventKind::TurnPhaseEnd,
            Timing::Normal,
            1,
            vec![EventKind::DamageEnd],
        )),
        "paotai2" => Some(forced(EventKind::TurnPhaseEnd, Timing::Normal, 1, Vec::new())),
        "paotai3" => Some(forced(EventKind::DamageEnd, Timing::Normal, 1, Vec::new())),
        "bfengshi" => Some(forced(
            EventKind::DamageBegin,
            Timing::Normal,
            1,
            vec![EventKind::CardUsed],
        )),
        "bfengshi2" => {
            Some(forced(EventKind::DamageBegin, Timing::Normal, 1, Vec::new()))
        }
        "bfengshi3" => {
            Some(forced(EventKind::CardUsed, Timing::Normal, 1, Vec::new()))
        }
        "aqianghua2" => Some(forced(EventKind::DamageBegin, Timing::Normal, 1, Vec::new())),
        "shihuo" => Some(forced(EventKind::DamageEnd, Timing::Normal, 1, Vec::new())),
        "shoujia2" => Some(forced(
            EventKind::CardUsed,
            Timing::Normal,
            1,
            vec![EventKind::DamageEnd],
        )),
        "shoujia3" => Some(forced(EventKind::DamageEnd, Timing::Normal, 1, Vec::new())),
        "liudan" => Some(forced(EventKind::CardUsed, Timing::First, 3, Vec::new())),
        "shenqiang" => Some(forced(EventKind::DamageEnd, Timing::Normal, 1, Vec::new())),
        "tiandan" => Some(forced(EventKind::DrawPhaseBefore, Timing::Normal, 1, Vec::new())),
        "mianzhen2" => Some(forced(
            EventKind::DamageEnd,
            Timing::Normal,
            1,
            vec![EventKind::TurnPhaseEnd],
        )),
        "zhiyuan" => Some(forced(EventKind::DamageBefore, Timing::Normal, 1, Vec::new())),
        "bingqiang" | "bingqiang_remove" => Some(forced(
            EventKind::TurnBegin,
            Timing::Normal,
            1,
            vec![EventKind::Death],
        )),
        "jidong" => Some(forced(EventKind::TurnPhaseEnd, Timing::Normal, 1, Vec::new())),
        _ => None,
    }
}

pub(super) fn can_activate_b(name: &str, ctx: &QueryCtx<'_>, owner: PlayerId) -> bool {
    let st = ctx.state;
    let p = st.player(owner);
    let has_he = !p.hand.is_empty() || !p.equips.is_empty();
    match name {
        "zhongdun" => {
            p.hujia > 0
                && has_he
                && alive_others(st, owner)
                    .into_iter()
                    .any(|t| st.player(t).hujia == 0)
        }
        "maoding" | "maoding2" => {
            p.hand
                .iter()
                .filter(|c| ctx.catalog.card_kind(**c) == CardKind::Hslingjian)
                .count()
                >= 2
                && st.alive_players().any(|t| st.player(t).hujia == 0)
        }
        "paotai" => {
            flag(st, owner, "ow_paotai_lv") < 3
                && p.hand.iter().any(|c| ctx.catalog.card_kind(*c).is_sha_like())
        }
        "yinbo" => p
            .hand
            .iter()
            .chain(p.equips.iter())
            .any(|c| suit_of(st, ctx.catalog, owner, *c) == Suit::Spade),
        "aqianghua" => !p.hand.is_empty() && !alive_others(st, owner).is_empty(),
        "shoujia" => {
            !p.hand.is_empty()
                && alive_others(st, owner)
                    .into_iter()
                    .any(|t| !has_skill_u05(st, ctx.catalog, t, "shoujia2"))
        }
        "shanguang" => {
            let rng = st.attack_range(ctx.catalog, owner);
            p.hand
                .iter()
                .chain(p.equips.iter())
                .any(|c| suit_of(st, ctx.catalog, owner, *c) == Suit::Diamond)
                && alive_others(st, owner)
                    .into_iter()
                    .any(|t| st.computed_distance(ctx.catalog, owner, t) <= rng)
        }
        "baoxue" => {
            flag(st, owner, "ow_baoxue_used") == 0
                && !alive_others(st, owner).is_empty()
                && p.hand
                    .iter()
                    .chain(p.equips.iter())
                    .any(|c| is_black_card(st, ctx.catalog, owner, *c))
        }
        "mianzhen" => {
            has_he
                && alive_others(st, owner)
                    .into_iter()
                    .any(|t| !has_skill_u05(st, ctx.catalog, t, "mianzhen2"))
        }
        "bingqiang" => has_he && !bingqiang_valid_targets(st, ctx.catalog).is_empty(),
        _ => false,
    }
}

pub(super) fn filter_b(
    name: &str,
    ctx: &QueryCtx<'_>,
    frames: &[Frame],
    frame: &Frame,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let st = ctx.state;
    let p = st.player(owner);
    match name {
        "mengji" => matches!(
            event,
            EventData::Damage(d) if d.source == Some(owner)
                && p.hujia == 0
                && !d.from_chain
                && damage_is_sha_like(frames, frame, ctx.catalog, d)
        ),
        "maoding" => matches!(
            event,
            EventData::Damage(d) if d.target == owner || d.source == Some(owner)
        ),
        "paotai" => match event {
            EventData::TurnEndWindow { player, after: false } => {
                *player == owner && flag(st, owner, "ow_paotai_lv") > 0
            }
            EventData::Damage(d) => {
                d.target == owner && d.amount > 0 && flag(st, owner, "ow_paotai_lv") > 0
            }
            _ => false,
        },
        "paotai2" => matches!(
            event,
            EventData::TurnEndWindow { player, after: false }
                if *player == owner
                    && !has_skill_u05(st, ctx.catalog, owner, "paotai")
                    && flag(st, owner, "ow_paotai_lv") > 0
        ),
        "paotai3" => matches!(
            event,
            EventData::Damage(d)
                if d.target == owner
                    && d.amount > 0
                    && !has_skill_u05(st, ctx.catalog, owner, "paotai")
                    && flag(st, owner, "ow_paotai_lv") > 0
        ),
        "bfengshi" => match event {
            EventData::CardUsed(u) => {
                u.player == owner
                    && !u.cancelled
                    && !has_skill_u05(st, ctx.catalog, owner, "bfengshi4")
                    && u.effective_kind(ctx.catalog).is_sha_like()
            }
            EventData::Damage(d) => {
                d.source == Some(owner)
                    && !has_skill_u05(st, ctx.catalog, owner, "bfengshi4")
                    && damage_is_sha_like(frames, frame, ctx.catalog, d)
            }
            _ => false,
        },
        "bfengshi3" => matches!(
            event,
            EventData::CardUsed(u) if u.player == owner
                && !u.cancelled
                && !has_skill_u05(st, ctx.catalog, owner, "bfengshi")
                && !has_skill_u05(st, ctx.catalog, owner, "bfengshi4")
                && u.effective_kind(ctx.catalog).is_sha_like()
        ),
        "bfengshi2" => matches!(
            event,
            EventData::Damage(d) if d.source == Some(owner)
                && !has_skill_u05(st, ctx.catalog, owner, "bfengshi")
                && !has_skill_u05(st, ctx.catalog, owner, "bfengshi4")
                && damage_is_sha_like(frames, frame, ctx.catalog, d)
        ),
        "aqianghua2" => matches!(event, EventData::Damage(d) if d.source == Some(owner)),
        "shihuo" => matches!(event, EventData::Damage(d) if d.nature == DamageNature::Fire),
        "shoujia2" => match event {
            EventData::CardUsed(u) if u.player == owner && !u.cancelled => {
                let Some(trap) = st
                    .vm_aux
                    .piles
                    .get(&(owner, "ow_shoujia".to_string()))
                    .and_then(|v| v.first().copied())
                else {
                    return false;
                };
                u.targets.iter().any(|&t| t != owner)
                    && u.suit(ctx.catalog) == Some(suit_of(st, ctx.catalog, owner, trap))
            }
            EventData::Damage(d) => flag(st, owner, "ow_shoujia2_caster") == u32::from(d.target) + 1,
            _ => false,
        },
        "shoujia3" => matches!(
            event,
            EventData::Damage(d)
                if !has_skill_u05(st, ctx.catalog, owner, "shoujia2")
                    && flag(st, owner, "ow_shoujia2_caster") == u32::from(d.target) + 1
        ),
        "liudan" => matches!(
            event,
            EventData::CardUsed(u)
                if u.player == owner
                    && !u.cancelled
                    && u.effective_kind(ctx.catalog).is_sha_like()
                    && alive_others(st, owner).into_iter().any(|t| {
                        !u.targets.contains(&t) && valid_single_sha_target(st, ctx.catalog, owner, t)
                    })
        ),
        "shenqiang" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && st.turn.current == owner
                    && damage_is_sha_like(frames, frame, ctx.catalog, d)
        ),
        "tiandan" => matches!(
            event,
            EventData::DrawPhase {
                player,
                outcome: DrawOutcome::Running,
                ..
            } if *player == owner
                && !st.turn.skip_play_phase
                && !p.skip_discard_phase
                && (p.hp.clamp(0, 5) as usize) > p.hand.len()
        ),
        "mianzhen2" => match event {
            EventData::Damage(d) => d.target == owner,
            EventData::TurnEndWindow { player, after: false } => *player == owner,
            _ => false,
        },
        "zhiyuan" => matches!(
            event,
            EventData::Damage(d) if d.source == Some(owner) && d.amount > 0
        ),
        "bingqiang" | "bingqiang_remove" => matches!(
            event,
            EventData::TurnBegin { player } | EventData::Death { player, .. } if *player == owner
        ),
        "jidong" => matches!(
            event,
            EventData::TurnEndWindow { after: false, .. } if p.hp == 1 && !p.face_down
        ),
        _ => false,
    }
}

fn discard_shoujia_trap(ctx: &mut Ctx<'_>, owner: PlayerId) {
    if let Some(cards) = ctx.core.vm_aux.piles.remove(&(owner, "ow_shoujia".to_string())) {
        for c in cards {
            ctx.to_discard(c);
            ctx.log(LogEvent::CardDiscarded {
                player: owner,
                cards: vec![c],
            });
        }
    }
    set_flag(ctx.core, owner, "ow_shoujia2_caster", 0);
    remove_skill_u05(ctx, owner, "shoujia2");
    remove_skill_u05(ctx, owner, "shoujia3");
}

fn step_baoxue_loop(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let cur = m.cards.first().copied().unwrap_or(0) as usize;
    if cur < m.targets.len() {
        let t = m.targets[cur];
        m.cards[0] = (cur + 1) as u32;
        if ctx.core.player(t).alive {
            random_discard_he(ctx, t);
            m.stage = HostStage::ShuchuiTargetLoop;
            if !ctx.core.player(t).face_down {
                turn_over_to_back(ctx, t);
                return Flow::Yield;
            }
        }
        return step_baoxue_loop(m, ctx);
    }
    if cur == m.targets.len() {
        m.cards[0] = (cur + 1) as u32;
        if ctx.core.player(m.owner).alive && !ctx.core.player(m.owner).face_down {
            m.stage = HostStage::ShuchuiTargetLoop;
            turn_over_to_back(ctx, m.owner);
            return Flow::Yield;
        }
    }
    Flow::Done
}

pub(super) fn step_b(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "mengji" => {
            ctx.log(LogEvent::SkillTriggered {
                skill: "mengji",
                owner,
                presentation: None,
            });
            with_current_damage_mut(ctx, |dm| dm.amount += 1);
            Flow::Done
        }
        "zhongdun" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::SelectCard, ASK_OW_ZHONGDUN_CARD),
            HostStage::SelectCard => {
                let Some(&cid) = answered_cards(ctx).first() else {
                    refund_once_used(ctx, owner, "zhongdun");
                    return Flow::Done;
                };
                m.cards = vec![cid];
                wait_ow(m, ctx, HostStage::TargetChoice, ASK_OW_ZHONGDUN_TARGET)
            }
            HostStage::TargetChoice => {
                let Some(&t) = answered_players(ctx).first() else {
                    refund_once_used(ctx, owner, "zhongdun");
                    return Flow::Done;
                };
                let Some(cid) = m.cards.first().copied() else {
                    return Flow::Done;
                };
                ctx.log(LogEvent::SkillTriggered {
                    skill: "zhongdun",
                    owner,
                    presentation: None,
                });
                if ctx.core.player(owner).hujia == 0 {
                    return Flow::Done;
                }
                discard_he_card(ctx, owner, cid);
                ctx.core.player_mut(owner).hujia =
                    ctx.core.player(owner).hujia.saturating_sub(1);
                if ctx.core.player(t).alive {
                    let h = ctx.core.player(t).hujia;
                    ctx.core.player_mut(t).hujia = h.saturating_add(1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "maoding" | "maoding2" => match m.stage {
            HostStage::Act => {
                if m.target.is_none() && ctx.core.turn.phase == Phase::Play {
                    wait_ow(m, ctx, HostStage::SelectCard, ASK_OW_MAODING2_CARDS)
                } else {
                    wait_ow(m, ctx, HostStage::SelectNature, ASK_OW_MAODING_CONFIRM)
                }
            }
            HostStage::SelectNature => {
                if answered_confirm(ctx) {
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "maoding",
                        owner,
                        presentation: None,
                    });
                    claim_off_deck_supply(
                        ctx,
                        owner,
                        CardKind::Hslingjian,
                        "ow_hslingjian_",
                        true,
                    );
                }
                Flow::Done
            }
            HostStage::SelectCard => {
                let picks = answered_cards(ctx);
                if picks.len() != 2 {
                    return Flow::Done;
                }
                m.cards = picks;
                wait_ow(m, ctx, HostStage::TargetChoice, ASK_OW_MAODING2_TARGET)
            }
            HostStage::TargetChoice => {
                let Some(&t) = answered_players(ctx).first() else {
                    return Flow::Done;
                };
                ctx.log(LogEvent::SkillTriggered {
                    skill: "maoding2",
                    owner,
                    presentation: None,
                });
                for &c in &m.cards {
                    ctx.discard_from_hand(owner, c);
                }
                if ctx.core.player(t).alive {
                    let h = ctx.core.player(t).hujia;
                    ctx.core.player_mut(t).hujia = h.saturating_add(1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "paotai" | "paotai2" | "paotai3" => match m.stage {
            HostStage::Act => {
                if m.target.is_none() && ctx.core.turn.phase == Phase::Play {
                    wait_ow(m, ctx, HostStage::SelectCard, ASK_OW_PAOTAI_CARD)
                } else if m.cards.len() == 1 && m.cards[0] == 0 && m.target == Some(owner) {
                    // TurnPhaseEnd -> paotai2
                    let lv = flag(ctx.core, owner, "ow_paotai_lv");
                    let pct = match lv {
                        1 => 30,
                        2 => 60,
                        3.. => 100,
                        _ => 0,
                    };
                    if pct > 0 && ctx.core.rng.gen_range(100u32) < pct {
                        let enemies: Vec<PlayerId> = alive_others(ctx.core, owner)
                            .into_iter()
                            .filter(|&p| {
                                ctx.core.source_is_enemy_of(ctx.catalog, owner, p) == Some(true)
                            })
                            .collect();
                        if !enemies.is_empty() {
                            let idx = ctx.core.rng.gen_range(enemies.len() as u32) as usize;
                            let t = enemies[idx];
                            ctx.log(LogEvent::SkillTriggered {
                                skill: "paotai2",
                                owner,
                                presentation: None,
                            });
                            ctx.emit(EventData::Damage(damage_data_with(
                                Some(owner),
                                t,
                                1,
                                DamageNature::Fire,
                                None,
                            )));
                        }
                    }
                    Flow::Done
                } else {
                    // DamageEnd -> paotai3
                    let amt = m.cards.get(1).copied().unwrap_or(0);
                    let lv = flag(ctx.core, owner, "ow_paotai_lv");
                    set_flag(ctx.core, owner, "ow_paotai_lv", lv.saturating_sub(amt));
                    Flow::Done
                }
            }
            HostStage::SelectCard => {
                let Some(&cid) = answered_cards(ctx).first() else {
                    return Flow::Done;
                };
                ctx.log(LogEvent::SkillTriggered {
                    skill: "paotai",
                    owner,
                    presentation: None,
                });
                ctx.discard_from_hand(owner, cid);
                let lv = flag(ctx.core, owner, "ow_paotai_lv");
                set_flag(ctx.core, owner, "ow_paotai_lv", (lv + 1).min(3));
                Flow::Done
            }
            _ => Flow::Done,
        },
        "bfengshi" => {
            if matches!(
                ctx.res.stack.last().map(|f| &f.data),
                Some(EventData::CardUsed(_))
            ) {
                grant_temp_skill(ctx, owner, "bfengshi4");
                return Flow::Done;
            }
            let pct = (ctx.core.turn.cards_used_turn * 20).min(100);
            if pct > 0 && ctx.core.rng.gen_range(100u32) < pct {
                ctx.log(LogEvent::SkillTriggered {
                    skill: "bfengshi2",
                    owner,
                    presentation: None,
                });
                with_current_damage_mut(ctx, |dm| dm.amount += 1);
            }
            Flow::Done
        }
        "bfengshi2" => {
            let pct = (ctx.core.turn.cards_used_turn * 20).min(100);
            if pct > 0 && ctx.core.rng.gen_range(100u32) < pct {
                ctx.log(LogEvent::SkillTriggered {
                    skill: "bfengshi2",
                    owner,
                    presentation: None,
                });
                with_current_damage_mut(ctx, |dm| dm.amount += 1);
            }
            Flow::Done
        }
        "bfengshi3" => {
            grant_temp_skill(ctx, owner, "bfengshi4");
            Flow::Done
        }
        "yinbo" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::SelectCard, ASK_OW_YINBO_CARD),
            HostStage::SelectCard => {
                let Some(&cid) = answered_cards(ctx).first() else {
                    refund_once_used(ctx, owner, "yinbo");
                    return Flow::Done;
                };
                ctx.log(LogEvent::SkillTriggered {
                    skill: "yinbo",
                    owner,
                    presentation: None,
                });
                discard_he_card(ctx, owner, cid);
                let mut pool: Vec<PlayerId> = alive_others(ctx.core, owner)
                    .into_iter()
                    .filter(|&p| {
                        ctx.core.source_is_enemy_of(ctx.catalog, owner, p) == Some(true)
                            && (!ctx.core.player(p).hand.is_empty()
                                || !ctx.core.player(p).equips.is_empty())
                    })
                    .collect();
                let mut picked = Vec::new();
                while !pool.is_empty() && picked.len() < 3 {
                    let i = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                    picked.push(pool.swap_remove(i));
                }
                let seat_order = ctx.core.seat_order_from(ctx.core.next_alive(owner));
                picked.sort_by_key(|p| {
                    seat_order.iter().position(|x| x == p).unwrap_or(usize::MAX)
                });
                for t in picked {
                    if ctx.core.player(t).alive {
                        random_discard_he(ctx, t);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "aqianghua" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::TargetChoice, ASK_OW_AQIANGHUA_TARGET),
            HostStage::TargetChoice => {
                let Some(&t) = answered_players(ctx).first() else {
                    refund_once_used(ctx, owner, "aqianghua");
                    return Flow::Done;
                };
                ctx.log(LogEvent::SkillTriggered {
                    skill: "aqianghua",
                    owner,
                    presentation: None,
                });
                let all_hand: Vec<CardId> = ctx.core.player(owner).hand.clone();
                if !all_hand.is_empty() {
                    ctx.give_hand_cards(owner, t, &all_hand);
                }
                if ctx.core.player(t).alive {
                    let h = ctx.core.player(t).hujia;
                    ctx.core.player_mut(t).hujia = h.saturating_add(1);
                    grant_skill_u05(ctx, t, "aqianghua2", false);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "aqianghua2" => {
            ctx.log(LogEvent::SkillTriggered {
                skill: "aqianghua2",
                owner,
                presentation: None,
            });
            with_current_damage_mut(ctx, |dm| dm.amount += 1);
            remove_skill_u05(ctx, owner, "aqianghua2");
            Flow::Done
        }
        "shihuo" => {
            ctx.log(LogEvent::SkillTriggered {
                skill: "shihuo",
                owner,
                presentation: None,
            });
            draw_direct(ctx, owner, 1);
            Flow::Done
        }
        "shoujia" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::SelectCard, ASK_OW_SHOUJIA_CARD),
            HostStage::SelectCard => {
                let Some(&cid) = answered_cards(ctx).first() else {
                    refund_once_used(ctx, owner, "shoujia");
                    return Flow::Done;
                };
                m.cards = vec![cid];
                wait_ow(m, ctx, HostStage::TargetChoice, ASK_OW_SHOUJIA_TARGET)
            }
            HostStage::TargetChoice => {
                let Some(&t) = answered_players(ctx).first() else {
                    refund_once_used(ctx, owner, "shoujia");
                    return Flow::Done;
                };
                let Some(cid) = m.cards.first().copied() else {
                    return Flow::Done;
                };
                if !ctx.core.player(owner).hand.contains(&cid) {
                    return Flow::Done;
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "shoujia",
                    owner,
                    presentation: None,
                });
                ctx.take_hand_card(owner, cid);
                ctx.core
                    .vm_aux
                    .piles
                    .insert((t, "ow_shoujia".to_string()), vec![cid]);
                grant_skill_u05(ctx, t, "shoujia2", false);
                set_flag(ctx.core, t, "ow_shoujia2_caster", u32::from(owner) + 1);
                Flow::Done
            }
            _ => Flow::Done,
        },
        "shoujia2" | "shoujia3" => {
            // DamageEnd if cards.len() == 2 (from machine_u05_ow for Damage)
            if m.cards.len() == 2 {
                ctx.log(LogEvent::SkillTriggered {
                    skill: "shoujia3",
                    owner,
                    presentation: None,
                });
                discard_shoujia_trap(ctx, owner);
                Flow::Done
            } else {
                // CardUsed trap triggered!
                ctx.log(LogEvent::SkillTriggered {
                    skill: "shoujia2",
                    owner,
                    presentation: None,
                });
                discard_shoujia_trap(ctx, owner);
                turn_over_to_back(ctx, owner);
                Flow::Done
            }
        }
        "liudan" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::SelectNature, ASK_OW_LIUDAN_CONFIRM),
            HostStage::SelectNature => {
                if !answered_confirm(ctx) {
                    return Flow::Done;
                }
                let existing = m.targets.clone();
                let mut added = Vec::new();
                for t in alive_others(ctx.core, owner) {
                    if !existing.contains(&t)
                        && valid_single_sha_target(ctx.core, ctx.catalog, owner, t)
                        && ctx.core.rng.gen_range(2u32) == 0
                    {
                        added.push(t);
                    }
                }
                if !added.is_empty() {
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "liudan",
                        owner,
                        presentation: None,
                    });
                    with_current_card_use_mut(ctx, |u| {
                        for t in added {
                            if !u.targets.contains(&t) {
                                u.targets.push(t);
                            }
                        }
                    });
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "shenqiang" => {
            ctx.log(LogEvent::SkillTriggered {
                skill: "shenqiang",
                owner,
                presentation: None,
            });
            ctx.core.turn.sha_used_play_phase = false;
            Flow::Done
        }
        "tiandan" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::SelectNature, ASK_OW_TIANDAN_CONFIRM),
            HostStage::SelectNature => {
                if !answered_confirm(ctx) {
                    return Flow::Done;
                }
                let cap = ctx.core.player(owner).hp.clamp(0, 5) as usize;
                let cur = ctx.core.player(owner).hand.len();
                let need = cap.saturating_sub(cur);
                if need == 0 {
                    return Flow::Done;
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "tiandan",
                    owner,
                    presentation: None,
                });
                for _ in 0..need {
                    claim_off_deck_supply(ctx, owner, CardKind::Sha, "ow_sha_", false);
                }
                ctx.core.turn.skip_play_phase = true;
                ctx.core.player_mut(owner).skip_discard_phase = true;
                Flow::Done
            }
            _ => Flow::Done,
        },
        "shanguang" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::SelectCard, ASK_OW_SHANGUANG_CARD),
            HostStage::SelectCard => {
                let Some(&cid) = answered_cards(ctx).first() else {
                    refund_once_used(ctx, owner, "shanguang");
                    return Flow::Done;
                };
                m.cards = vec![cid];
                wait_ow(m, ctx, HostStage::TargetChoice, ASK_OW_SHANGUANG_TARGET)
            }
            HostStage::TargetChoice => {
                let Some(&t) = answered_players(ctx).first() else {
                    refund_once_used(ctx, owner, "shanguang");
                    return Flow::Done;
                };
                let Some(cid) = m.cards.first().copied() else {
                    return Flow::Done;
                };
                ctx.log(LogEvent::SkillTriggered {
                    skill: "shanguang",
                    owner,
                    presentation: None,
                });
                discard_he_card(ctx, owner, cid);
                if ctx.core.player(t).alive {
                    grant_temp_skill(ctx, t, "shanguang2");
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "baoxue" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::TargetChoice, ASK_OW_BAOXUE_TARGETS),
            HostStage::TargetChoice => {
                let mut targets = answered_players(ctx);
                if targets.is_empty() {
                    return Flow::Done;
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "baoxue",
                    owner,
                    presentation: None,
                });
                set_flag(ctx.core, owner, "ow_baoxue_used", 1);
                let blacks: Vec<CardId> = hand_and_equips(ctx.core, owner)
                    .into_iter()
                    .filter(|c| is_black_card(ctx.core, ctx.catalog, owner, *c))
                    .collect();
                for c in blacks {
                    discard_he_card(ctx, owner, c);
                }
                let order = ctx.core.seat_order_from(ctx.core.next_alive(owner));
                targets.sort_by_key(|p| order.iter().position(|x| x == p).unwrap_or(usize::MAX));
                m.targets = targets;
                m.cards = vec![0];
                step_baoxue_loop(m, ctx)
            }
            HostStage::ShuchuiTargetLoop => step_baoxue_loop(m, ctx),
            _ => Flow::Done,
        },
        "mianzhen" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::SelectCard, ASK_OW_MIANZHEN_CARD),
            HostStage::SelectCard => {
                let Some(&cid) = answered_cards(ctx).first() else {
                    refund_once_used(ctx, owner, "mianzhen");
                    return Flow::Done;
                };
                m.cards = vec![cid];
                wait_ow(m, ctx, HostStage::TargetChoice, ASK_OW_MIANZHEN_TARGET)
            }
            HostStage::TargetChoice => {
                let Some(&t) = answered_players(ctx).first() else {
                    refund_once_used(ctx, owner, "mianzhen");
                    return Flow::Done;
                };
                let Some(cid) = m.cards.first().copied() else {
                    return Flow::Done;
                };
                ctx.log(LogEvent::SkillTriggered {
                    skill: "mianzhen",
                    owner,
                    presentation: None,
                });
                discard_he_card(ctx, owner, cid);
                if !ctx.core.player(t).alive {
                    return Flow::Done;
                }
                m.target = Some(t);
                let shan_opts = crate::kernel::response_kind_cards_core(
                    ctx.core,
                    ctx.catalog,
                    t,
                    CardKind::Shan,
                );
                if shan_opts.is_empty() {
                    grant_skill_u05(ctx, t, "mianzhen2", false);
                    return Flow::Done;
                }
                wait_ow(m, ctx, HostStage::SelectNature, ASK_OW_MIANZHEN_SHAN)
            }
            HostStage::SelectNature => {
                let Some(t) = m.target else { return Flow::Done };
                if let Some(&shan) = answered_cards(ctx).first() {
                    ctx.take_owned_card(t, shan);
                    ctx.to_discard(shan);
                    ctx.emit(EventData::CardResponded { card: shan, player: t });
                } else if ctx.core.player(t).alive {
                    grant_skill_u05(ctx, t, "mianzhen2", false);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "mianzhen2" => {
            remove_skill_u05(ctx, owner, "mianzhen2");
            Flow::Done
        }
        "zhiyuan" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::SelectNature, ASK_OW_ZHIYUAN_CONFIRM),
            HostStage::SelectNature => {
                if !answered_confirm(ctx) {
                    return Flow::Done;
                }
                let Some(t) = m.target else { return Flow::Done };
                let mut amt = 0u32;
                with_current_damage_mut(ctx, |dm| {
                    amt = dm.amount.max(0) as u32;
                    dm.prevented = true;
                    dm.settlement.outcome = DamageOutcome::Cancelled;
                });
                ctx.log(LogEvent::SkillTriggered {
                    skill: "zhiyuan",
                    owner,
                    presentation: None,
                });
                if amt > 0 && ctx.core.player(t).alive {
                    ctx.recover(t, amt, None);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "bingqiang" | "bingqiang_remove" => match m.stage {
            HostStage::Act => {
                if m.target.is_none() && ctx.core.turn.phase == Phase::Play {
                    wait_ow(m, ctx, HostStage::SelectCard, ASK_OW_BINGQIANG_CARDS)
                } else {
                    clear_bingqiang_by_owner(ctx, owner);
                    Flow::Done
                }
            }
            HostStage::SelectCard => {
                let picks = answered_cards(ctx);
                if picks.is_empty() {
                    return Flow::Done;
                }
                let first_red = super::u05_ow::is_red_card(ctx.core, ctx.catalog, owner, picks[0]);
                if picks
                    .iter()
                    .any(|&c| super::u05_ow::is_red_card(ctx.core, ctx.catalog, owner, c) != first_red)
                {
                    return wait_ow(m, ctx, HostStage::SelectCard, ASK_OW_BINGQIANG_CARDS);
                }
                m.cards = picks;
                wait_ow(m, ctx, HostStage::TargetChoice, ASK_OW_BINGQIANG_TARGET)
            }
            HostStage::TargetChoice => {
                let Some(&t) = answered_players(ctx).first() else {
                    return Flow::Done;
                };
                if !prev_next_targets_clear_of(ctx.core, ctx.catalog, t) || m.cards.is_empty() {
                    return Flow::Done;
                }
                let first_red = super::u05_ow::is_red_card(ctx.core, ctx.catalog, owner, m.cards[0]);
                let n = m.cards.len() as u32;
                ctx.log(LogEvent::SkillTriggered {
                    skill: "bingqiang",
                    owner,
                    presentation: None,
                });
                for &c in &m.cards {
                    discard_he_card(ctx, owner, c);
                }
                let prev = ctx
                    .core
                    .seat_order_from(ctx.core.next_alive(t))
                    .into_iter()
                    .last()
                    .unwrap_or(t);
                let next = ctx.core.next_alive(t);
                let mut affected = vec![t];
                for p in [prev, next] {
                    if !affected.contains(&p) {
                        affected.push(p);
                    }
                }
                for ap in affected {
                    if !ctx.core.player(ap).alive {
                        continue;
                    }
                    if first_red {
                        set_flag(ctx.core, ap, "ow_bingqiang2", n);
                        set_flag(ctx.core, ap, "ow_bingqiang2_owner", u32::from(owner) + 1);
                        grant_skill_u05(ctx, ap, "bingqiang2", false);
                    } else {
                        set_flag(ctx.core, ap, "ow_bingqiang5", n);
                        set_flag(ctx.core, ap, "ow_bingqiang5_owner", u32::from(owner) + 1);
                        grant_skill_u05(ctx, ap, "bingqiang5", false);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "jidong" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::SelectNature, ASK_OW_JIDONG_CONFIRM),
            HostStage::SelectNature => {
                if !answered_confirm(ctx) {
                    return Flow::Done;
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "jidong",
                    owner,
                    presentation: None,
                });
                m.stage = HostStage::DrawSettle;
                toggle_turn_over(ctx, owner);
                Flow::Yield
            }
            HostStage::DrawSettle => {
                if ctx.core.player(owner).alive {
                    ctx.recover(owner, 2, None);
                    if ctx.core.player(owner).face_down {
                        ctx.core.player_mut(owner).jidong_active = true;
                        grant_skill_u05(ctx, owner, "jidong2", false);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        _ => Flow::Done,
    }
}
