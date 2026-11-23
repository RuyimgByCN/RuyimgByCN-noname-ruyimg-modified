//! U05-3 (`shaai-ow`) Batch D（标签 1043）：禅雅塔 / 法老之鹰 / 卢西奥 / 秩序之光 / 天使 / 猎空（27 项）。

use super::u05_ow::{
    alive_others, answered_cards, answered_confirm, answered_players, answered_tag,
    damage_is_sha_like, discard_he_card, draw_direct, flag, grant_skill_u05, hand_and_equips,
    has_skill_u05, is_red_card, remove_skill_u05, set_flag, suit_of, toggle_turn_over, wait_ow,
    ASK_OW_FEIDAN_CARD, ASK_OW_GUANGSHU_CARD, ASK_OW_GUANGSHU_TARGET, ASK_OW_HUHUAN_CARDS, ASK_OW_HUOYU_CARDS, ASK_OW_KUOYIN_CARDS, ASK_OW_LUAN_CARD,
    ASK_OW_LUAN_TARGET, ASK_OW_SHANHUI_CONFIRM, ASK_OW_SHANXIAN_CONFIRM, ASK_OW_SHENG_TARGETS,
    ASK_OW_SHOUHU_CARD, ASK_OW_SHOUHU_TARGET, ASK_OW_XIE_CARD, ASK_OW_XIE_TARGET,
    ASK_OW_YUEDONG_TARGETS, ASK_OW_ZIYU_DR,
};
use super::*;
use crate::catalog::{CardKind, EquipSlot, QueryCtx, Suit, TriggerMeta};
use crate::event::{DamageNature, EventData, EventKind, Frame, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{damage_data, Ctx, Flow, LogEvent};

pub(super) fn trigger_meta_d(name: &str) -> Option<TriggerMeta> {
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
        "xie2" => Some(forced(
            EventKind::TurnPhaseEnd,
            Timing::Normal,
            1,
            vec![EventKind::Death],
        )),
        "xie4" => Some(forced(EventKind::Death, Timing::Normal, 1, Vec::new())),
        "luan2" => Some(forced(
            EventKind::DamageEnd,
            Timing::Normal,
            1,
            vec![EventKind::DyingSaved, EventKind::Death],
        )),
        "luan3" => Some(forced(EventKind::DyingSaved, Timing::Normal, 1, Vec::new())),
        "luan4" => Some(forced(EventKind::Death, Timing::Normal, 1, Vec::new())),
        "sheng2" => Some(forced(EventKind::TurnBegin, Timing::Normal, 1, Vec::new())),
        "feidan" => Some(forced(EventKind::DamageAfter, Timing::Normal, 1, Vec::new())),
        "yuedong" => Some(forced(EventKind::PhaseEnd, Timing::Normal, 1, Vec::new())),
        "kuoyin" | "kuoyin2" => Some(forced(EventKind::TurnBegin, Timing::Normal, 1, Vec::new())),
        "guangshu_diamond" => Some(forced(EventKind::DamageEnd, Timing::Normal, 1, Vec::new())),
        "guangshu_heart" => Some(forced(EventKind::DamageEnd, Timing::Normal, 1, Vec::new())),
        "guangshu_club" => Some(forced(EventKind::TurnPhaseEnd, Timing::Normal, 1, Vec::new())),
        // guangshu_spade 已在 kernel/events/turn.rs:110 承担，不重复派发
        "ziyu" => Some(forced(EventKind::TurnPhaseEnd, Timing::Normal, 1, Vec::new())),
        "shanxian" => Some(forced(EventKind::TurnBegin, Timing::First, 5, Vec::new())),
        "shanxian2" => Some(forced(
            EventKind::CardsDrawn,
            Timing::Normal,
            1,
            vec![
                EventKind::CardsMoved,
                EventKind::CardsLost,
                EventKind::CardsDiscarded,
            ],
        )),
        "shanhui" => Some(forced(EventKind::DamageEnd, Timing::Normal, 1, Vec::new())),
        _ => None,
    }
}

pub(super) fn can_activate_d(name: &str, ctx: &QueryCtx<'_>, owner: PlayerId) -> bool {
    let st = ctx.state;
    let p = st.player(owner);
    let he_len = p.hand.len() + p.equips.len();
    match name {
        "xie" => {
            p.hand
                .iter()
                .any(|c| suit_of(st, ctx.catalog, owner, *c) == Suit::Heart)
                && alive_others(st, owner)
                    .into_iter()
                    .any(|t| !has_skill_u05(st, ctx.catalog, t, "xie2"))
        }
        "luan" => {
            p.hand
                .iter()
                .any(|c| suit_of(st, ctx.catalog, owner, *c) == Suit::Spade)
                && alive_others(st, owner)
                    .into_iter()
                    .any(|t| !has_skill_u05(st, ctx.catalog, t, "luan2"))
        }
        "sheng" => {
            flag(st, owner, "ow_sheng_used") == 0
                && st
                    .alive_players()
                    .any(|t| st.player(t).hp < st.player(t).max_hp as i32)
        }
        "huoyu" => {
            flag(st, owner, "ow_huoyu_used") == 0
                && alive_others(st, owner).into_iter().any(|t| {
                    ctx.catalog.ow_target_enabled(
                        st,
                        owner,
                        t,
                        None,
                        CardKind::Chiyuxi,
                        DamageNature::Fire,
                    )
                })
                && p.hand
                    .iter()
                    .chain(p.equips.iter())
                    .filter(|c| is_red_card(st, ctx.catalog, owner, **c))
                    .count()
                    >= 2
        }
        "huhuan" => he_len >= 2 && flag(st, owner, "ow_yuedong_recover") == 0,
        "kuoyin" => {
            let has_num = flag(st, owner, "ow_yuedong_num") > 0;
            let has_eff = flag(st, owner, "ow_yuedong_eff") > 0;
            if has_num && has_eff {
                false
            } else if has_num {
                he_len >= 2
            } else {
                he_len >= 1
            }
        }
        "guangshu" => {
            he_len > 0
                && st.alive_players().any(|t| {
                    !has_skill_u05(st, ctx.catalog, t, "guangshu_heart")
                        && !has_skill_u05(st, ctx.catalog, t, "guangshu_spade")
                        && !has_skill_u05(st, ctx.catalog, t, "guangshu_club")
                        && !has_skill_u05(st, ctx.catalog, t, "guangshu_diamond")
                })
        }
        "shouhu" => {
            p.hand
                .iter()
                .any(|c| ctx.catalog.card_kind(*c).is_sha_like())
                && alive_others(st, owner)
                    .into_iter()
                    .any(|t| st.player(t).hp < st.player(t).max_hp as i32)
        }
        _ => false,
    }
}

pub(super) fn filter_d(
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
        "xie2" => match event {
            EventData::TurnEndWindow { player, after: false } => {
                let xie_p = flag(st, owner, "ow_xie_phase");
                if xie_p == 0 {
                    *player == owner
                } else {
                    let cur = flag(st, 0, "ow_phase_number");
                    cur > xie_p && (cur - xie_p) % 6 == 0
                }
            }
            EventData::Death { player, .. } => {
                flag(st, owner, "ow_xie_owner") == u32::from(*player) + 1
            }
            _ => false,
        },
        "xie4" => matches!(
            event,
            EventData::Death { player, .. }
                if !has_skill_u05(st, ctx.catalog, owner, "xie2")
                    && flag(st, owner, "ow_xie_owner") == u32::from(*player) + 1
        ),
        "luan2" => match event {
            EventData::Damage(d) => d.target == owner,
            EventData::DyingSaved { player, .. } => *player == owner,
            EventData::Death { player, .. } => {
                flag(st, owner, "ow_luan_owner") == u32::from(*player) + 1
            }
            _ => false,
        },
        "luan3" => matches!(
            event,
            EventData::DyingSaved { player, .. }
                if *player == owner && !has_skill_u05(st, ctx.catalog, owner, "luan2")
        ),
        "luan4" => matches!(
            event,
            EventData::Death { player, .. }
                if !has_skill_u05(st, ctx.catalog, owner, "luan2")
                    && flag(st, owner, "ow_luan_owner") == u32::from(*player) + 1
        ),
        "sheng2" => matches!(event, EventData::TurnBegin { player } if *player == owner),
        "feidan" => matches!(
            event,
            EventData::Damage(d) if d.source == Some(owner)
                && damage_is_sha_like(frames, frame, ctx.catalog, d)
                && (!p.hand.is_empty() || !p.equips.is_empty())
                && alive_others(st, d.target)
                    .into_iter()
                    .any(|t| st.computed_distance(ctx.catalog, d.target, t) <= 1)
        ),
        "yuedong" => match event {
            crate::event::EventData::Phase {
                player,
                phase: crate::event::Phase::Play,
                begin: false,
            } if *player == owner => {
                let recover = flag(st, owner, "ow_yuedong_recover") > 0;
                !recover
                    || st
                        .alive_players()
                        .any(|t| st.player(t).hp < st.player(t).max_hp as i32)
            }
            _ => false,
        },
        "kuoyin" | "kuoyin2" => matches!(event, EventData::TurnBegin { player } if *player == owner),
        "guangshu_diamond" => matches!(event, EventData::Damage(d) if d.source == Some(owner)),
        "guangshu_heart" => matches!(event, EventData::Damage(d) if d.target == owner),
        "guangshu_club" => matches!(
            event,
            EventData::TurnEndWindow { player, after: false } if *player == owner
        ),
        "ziyu" => matches!(
            event,
            EventData::TurnEndWindow { after: false, .. }
                if flag(st, 0, "ow_phase_number") > 0 && flag(st, 0, "ow_phase_number") % 4 == 0
        ),
        "shanxian" => match event {
            EventData::TurnBegin { player } => {
                *player != owner
                    && !p.face_down
                    && flag(st, owner, "ow_shanxian_target") == 0
            }
            _ => false,
        },
        "shanxian2" => match event {
            EventData::CardsDrawn { player, .. }
            | EventData::CardsLost { player, .. }
            | EventData::CardsDiscarded { player, .. } => *player == owner,
            EventData::CardsMoved { from, to, .. } => *from == owner || *to == owner,
            _ => false,
        },
        "shanhui" => matches!(
            event,
            EventData::Damage(d)
                if (d.target == owner || d.source == Some(owner))
                    && flag(st, owner, "ow_shanxian_n") > 0
                    && !has_skill_u05(st, ctx.catalog, owner, "shanxian2")
        ),
        _ => false,
    }
}

fn reclaim_ow_card(ctx: &mut Ctx<'_>, to: PlayerId, cid: CardId, to_equip: bool) {
    // 1) 从所有可能位置移除（防重复 ID；若在它处装备栏先注销装备技）
    let mut removed = false;
    if ctx.core.player(to).hand.contains(&cid) {
        if !to_equip {
            return;
        }
        ctx.core.player_mut(to).hand.retain(|&c| c != cid);
        removed = true;
    } else if ctx.core.player(to).equips.contains(&cid) {
        if to_equip {
            return;
        }
        ctx.core.player_mut(to).equips.retain(|&c| c != cid);
        ctx.core.declared_slots.remove(&cid);
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, to, cid);
        removed = true;
    }
    if !removed {
        if let Some(pos) = ctx.core.discard.iter().position(|&c| c == cid) {
            ctx.core.discard.remove(pos);
            removed = true;
        } else if let Some(pos) = ctx.core.deck.iter().position(|&c| c == cid) {
            ctx.core.deck.remove(pos);
            removed = true;
        } else if let Some(pos) = ctx.core.table.iter().position(|&c| c == cid) {
            ctx.core.table.remove(pos);
            removed = true;
        } else if let Some(pos) = ctx.core.detached_cards.iter().position(|(c, _)| *c == cid) {
            ctx.core.detached_cards.remove(pos);
            removed = true;
        }
    }
    if !removed {
        let seats = ctx.core.players.len() as PlayerId;
        for p in 0..seats {
            if ctx.core.player(p).hand.contains(&cid) {
                ctx.core.player_mut(p).hand.retain(|&c| c != cid);
                removed = true;
                break;
            }
            if ctx.core.player(p).equips.contains(&cid) {
                ctx.core.player_mut(p).equips.retain(|&c| c != cid);
                ctx.core.declared_slots.remove(&cid);
                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, p, cid);
                removed = true;
                break;
            }
            if ctx.core.player(p).judge.contains(&cid) {
                ctx.core.player_mut(p).judge.retain(|&c| c != cid);
                removed = true;
                break;
            }
            if ctx.core.player(p).off_court.contains(&cid) {
                ctx.core.player_mut(p).off_court.retain(|&c| c != cid);
                removed = true;
                break;
            }
        }
    }
    if !removed {
        for pile in ctx.core.vm_aux.piles.values_mut() {
            if let Some(pos) = pile.iter().position(|&c| c == cid) {
                pile.remove(pos);
                removed = true;
                break;
            }
        }
    }
    if !removed {
        return;
    }
    if to_equip {
        let want_slot: Option<EquipSlot> = ctx.catalog.card_slot_of(ctx.core, cid);
        let occupied = want_slot.is_some_and(|ws| {
            ctx.core
                .player(to)
                .equips
                .iter()
                .any(|ec| ctx.catalog.card_slot_of(ctx.core, *ec) == Some(ws))
        });
        if want_slot.is_some() && !occupied {
            ctx.core.player_mut(to).equips.push(cid);
            crate::kernel::grant_equip_skills(ctx, to, cid);
        } else {
            ctx.core.player_mut(to).hand.push(cid);
        }
    } else {
        ctx.core.player_mut(to).hand.push(cid);
    }
}

fn step_sheng_loop(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.cards.first().copied() == Some(0) {
        m.cards[0] = 1;
        set_flag(ctx.core, owner, "ow_sheng_used", 1);
        grant_skill_u05(ctx, owner, "sheng2", false);
        m.stage = HostStage::ShuchuiTargetLoop;
        toggle_turn_over(ctx, owner);
        return Flow::Yield;
    }
    for &t in &m.targets {
        if ctx.core.player(t).alive && ctx.core.player(t).hp < ctx.core.player(t).max_hp as i32 {
            ctx.recover(t, 1, None);
        }
    }
    Flow::Done
}

pub(super) fn step_d(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "xie" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::SelectCard, ASK_OW_XIE_CARD),
            HostStage::SelectCard => {
                let Some(&cid) = answered_cards(ctx).first() else {
                    return Flow::Done;
                };
                m.cards = vec![cid];
                wait_ow(m, ctx, HostStage::TargetChoice, ASK_OW_XIE_TARGET)
            }
            HostStage::TargetChoice => {
                let Some(&t) = answered_players(ctx).first() else {
                    return Flow::Done;
                };
                let Some(cid) = m.cards.first().copied() else {
                    return Flow::Done;
                };
                ctx.log(LogEvent::SkillTriggered {
                    skill: "xie",
                    owner,
                    presentation: None,
                });
                ctx.discard_from_hand(owner, cid);
                let seats = ctx.core.players.len() as PlayerId;
                for p in 0..seats {
                    if has_skill_u05(ctx.core, ctx.catalog, p, "xie2") {
                        remove_skill_u05(ctx, p, "xie2");
                    }
                }
                if ctx.core.player(t).alive {
                    grant_skill_u05(ctx, t, "xie2", false);
                    set_flag(ctx.core, t, "ow_xie_phase", 0);
                    set_flag(ctx.core, t, "ow_xie_owner", u32::from(owner) + 1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "xie2" | "xie4" => {
            if m.cards.len() == 1 && m.cards[0] == 0 {
                // TurnPhaseEnd
                if flag(ctx.core, owner, "ow_xie_phase") == 0 {
                    let cur = flag(ctx.core, 0, "ow_phase_number").max(1);
                    set_flag(ctx.core, owner, "ow_xie_phase", cur);
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "xie2",
                    owner,
                    presentation: None,
                });
                ctx.recover(owner, 1, None);
            } else {
                // Death of owner -> remove xie2
                remove_skill_u05(ctx, owner, "xie2");
            }
            Flow::Done
        }
        "luan" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::SelectCard, ASK_OW_LUAN_CARD),
            HostStage::SelectCard => {
                let Some(&cid) = answered_cards(ctx).first() else {
                    return Flow::Done;
                };
                m.cards = vec![cid];
                wait_ow(m, ctx, HostStage::TargetChoice, ASK_OW_LUAN_TARGET)
            }
            HostStage::TargetChoice => {
                let Some(&t) = answered_players(ctx).first() else {
                    return Flow::Done;
                };
                let Some(cid) = m.cards.first().copied() else {
                    return Flow::Done;
                };
                ctx.log(LogEvent::SkillTriggered {
                    skill: "luan",
                    owner,
                    presentation: None,
                });
                ctx.discard_from_hand(owner, cid);
                let seats = ctx.core.players.len() as PlayerId;
                for p in 0..seats {
                    if has_skill_u05(ctx.core, ctx.catalog, p, "luan2") {
                        remove_skill_u05(ctx, p, "luan2");
                    }
                }
                if ctx.core.player(t).alive {
                    grant_skill_u05(ctx, t, "luan2", false);
                    set_flag(ctx.core, t, "ow_luan_owner", u32::from(owner) + 1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "luan2" | "luan3" | "luan4" => {
            if m.cards.len() == 2 {
                // DamageEnd
                ctx.log(LogEvent::SkillTriggered {
                    skill: "luan2",
                    owner,
                    presentation: None,
                });
                ctx.emit(EventData::LoseHp {
                    player: owner,
                    amount: 1,
                });
            } else {
                // DyingSaved or caster Death
                remove_skill_u05(ctx, owner, "luan2");
            }
            Flow::Done
        }
        "sheng" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::TargetChoice, ASK_OW_SHENG_TARGETS),
            HostStage::TargetChoice => {
                let mut targets = answered_players(ctx);
                if targets.is_empty() {
                    return Flow::Done;
                }
                let order = ctx.core.seat_order_from(owner);
                targets.sort_by_key(|p| order.iter().position(|x| x == p).unwrap_or(usize::MAX));
                ctx.log(LogEvent::SkillTriggered {
                    skill: "sheng",
                    owner,
                    presentation: None,
                });
                m.targets = targets;
                m.cards = vec![0];
                step_sheng_loop(m, ctx)
            }
            HostStage::ShuchuiTargetLoop => step_sheng_loop(m, ctx),
            _ => Flow::Done,
        },
        "sheng2" => {
            remove_skill_u05(ctx, owner, "sheng2");
            Flow::Done
        }
        "huoyu" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::SelectCard, ASK_OW_HUOYU_CARDS),
            HostStage::SelectCard => {
                let picks = answered_cards(ctx);
                if picks.len() != 2
                    || picks
                        .iter()
                        .any(|c| !is_red_card(ctx.core, ctx.catalog, owner, *c))
                {
                    return Flow::Done;
                }
                let mut targets: Vec<PlayerId> = alive_others(ctx.core, owner)
                    .into_iter()
                    .filter(|t| {
                        ctx.catalog.ow_target_enabled(
                            ctx.core,
                            owner,
                            *t,
                            None,
                            CardKind::Chiyuxi,
                            DamageNature::Fire,
                        )
                    })
                    .collect();
                targets.reverse();
                if targets.is_empty() {
                    return Flow::Done;
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "huoyu",
                    owner,
                    presentation: None,
                });
                set_flag(ctx.core, owner, "ow_huoyu_used", 1);
                for c in picks {
                    discard_he_card(ctx, owner, c);
                }
                m.stage = HostStage::DrawSettle;
                ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                    card: None,
                    player: owner,
                    targets,
                    nature: DamageNature::Fire,
                    use_as: Some(CardKind::Chiyuxi),
                    cancelled: false,
                }));
                Flow::Yield
            }
            HostStage::DrawSettle => {
                if ctx.core.player(owner).alive {
                    let mut targets: Vec<PlayerId> = alive_others(ctx.core, owner)
                        .into_iter()
                        .filter(|t| {
                            ctx.catalog.ow_target_enabled(
                                ctx.core,
                                owner,
                                *t,
                                None,
                                CardKind::Chiyuxi,
                                DamageNature::Fire,
                            )
                        })
                        .collect();
                    targets.reverse();
                    if !targets.is_empty() {
                        ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                            card: None,
                            player: owner,
                            targets,
                            nature: DamageNature::Fire,
                            use_as: Some(CardKind::Chiyuxi),
                            cancelled: false,
                        }));
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "feidan" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::SelectCard, ASK_OW_FEIDAN_CARD),
            HostStage::SelectCard => {
                let Some(&cid) = answered_cards(ctx).first() else {
                    return Flow::Done;
                };
                let Some(victim) = m.target else {
                    return Flow::Done;
                };
                let adj: Vec<PlayerId> = alive_others(ctx.core, victim)
                    .into_iter()
                    .filter(|t| ctx.core.computed_distance(ctx.catalog, victim, *t) <= 1)
                    .collect();
                if adj.is_empty() {
                    return Flow::Done;
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "feidan",
                    owner,
                    presentation: None,
                });
                discard_he_card(ctx, owner, cid);
                // LIFO 栈序：先压尾部目标使座位序从靠前开始结算，或逐一推进
                for t in adj.into_iter().rev() {
                    ctx.emit(EventData::Damage(damage_data(Some(owner), t, 1)));
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "yuedong" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::TargetChoice, ASK_OW_YUEDONG_TARGETS),
            HostStage::TargetChoice => {
                let mut picked = answered_players(ctx);
                if picked.is_empty() {
                    return Flow::Done;
                }
                let order = ctx.core.seat_order_from(owner);
                picked.sort_by_key(|p| order.iter().position(|x| x == p).unwrap_or(usize::MAX));
                ctx.log(LogEvent::SkillTriggered {
                    skill: "yuedong",
                    owner,
                    presentation: None,
                });
                let eff = 1 + flag(ctx.core, owner, "ow_yuedong_eff");
                let recover = flag(ctx.core, owner, "ow_yuedong_recover") > 0;
                for t in picked {
                    if !ctx.core.player(t).alive {
                        continue;
                    }
                    if recover {
                        ctx.recover(t, eff, None);
                    } else {
                        draw_direct(ctx, t, eff);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "huhuan" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::SelectCard, ASK_OW_HUHUAN_CARDS),
            HostStage::SelectCard => {
                let picks = answered_cards(ctx);
                if picks.len() != 2 {
                    return Flow::Done;
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "huhuan",
                    owner,
                    presentation: None,
                });
                for c in picks {
                    discard_he_card(ctx, owner, c);
                }
                set_flag(ctx.core, owner, "ow_yuedong_recover", 1);
                Flow::Done
            }
            _ => Flow::Done,
        },
        "kuoyin" | "kuoyin2" => match m.stage {
            HostStage::Act => {
                if m.target.is_none() && ctx.core.turn.phase == crate::event::Phase::Play {
                    wait_ow(m, ctx, HostStage::SelectCard, ASK_OW_KUOYIN_CARDS)
                } else {
                    set_flag(ctx.core, owner, "ow_yuedong_recover", 0);
                    set_flag(ctx.core, owner, "ow_yuedong_num", 0);
                    set_flag(ctx.core, owner, "ow_yuedong_eff", 0);
                    Flow::Done
                }
            }
            HostStage::SelectCard => {
                let picks = answered_cards(ctx);
                let has_num = flag(ctx.core, owner, "ow_yuedong_num") > 0;
                let has_eff = flag(ctx.core, owner, "ow_yuedong_eff") > 0;
                let ok = match picks.len() {
                    1 => !has_num,
                    2 => !has_eff,
                    _ => false,
                };
                if !ok {
                    return Flow::Done;
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "kuoyin",
                    owner,
                    presentation: None,
                });
                for &c in &picks {
                    discard_he_card(ctx, owner, c);
                }
                if picks.len() == 1 {
                    let cur = flag(ctx.core, owner, "ow_yuedong_num");
                    set_flag(ctx.core, owner, "ow_yuedong_num", cur + 2);
                } else {
                    let cur = flag(ctx.core, owner, "ow_yuedong_eff");
                    set_flag(ctx.core, owner, "ow_yuedong_eff", cur + 1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "guangshu" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::SelectCard, ASK_OW_GUANGSHU_CARD),
            HostStage::SelectCard => {
                let Some(&cid) = answered_cards(ctx).first() else {
                    return Flow::Done;
                };
                m.cards = vec![cid];
                wait_ow(m, ctx, HostStage::TargetChoice, ASK_OW_GUANGSHU_TARGET)
            }
            HostStage::TargetChoice => {
                let Some(&t) = answered_players(ctx).first() else {
                    return Flow::Done;
                };
                let Some(cid) = m.cards.first().copied() else {
                    return Flow::Done;
                };
                let sub = match suit_of(ctx.core, ctx.catalog, owner, cid) {
                    Suit::Heart => "guangshu_heart",
                    Suit::Diamond => "guangshu_diamond",
                    Suit::Club => "guangshu_club",
                    Suit::Spade => "guangshu_spade",
                };
                ctx.log(LogEvent::SkillTriggered {
                    skill: "guangshu",
                    owner,
                    presentation: None,
                });
                discard_he_card(ctx, owner, cid);
                if ctx.core.player(t).alive {
                    grant_skill_u05(ctx, t, sub, false);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "guangshu_diamond" => {
            ctx.log(LogEvent::SkillTriggered {
                skill: "guangshu_diamond",
                owner,
                presentation: None,
            });
            draw_direct(ctx, owner, 2);
            remove_skill_u05(ctx, owner, "guangshu_diamond");
            Flow::Done
        }
        "guangshu_heart" => {
            ctx.log(LogEvent::SkillTriggered {
                skill: "guangshu_heart",
                owner,
                presentation: None,
            });
            ctx.recover(owner, 1, None);
            remove_skill_u05(ctx, owner, "guangshu_heart");
            Flow::Done
        }
        "guangshu_club" => {
            remove_skill_u05(ctx, owner, "guangshu_club");
            Flow::Done
        }
        "ziyu" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::SelectNature, ASK_OW_ZIYU_DR),
            HostStage::SelectNature => {
                if let Some(tag) = answered_tag(ctx) {
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "ziyu",
                        owner,
                        presentation: None,
                    });
                    if tag == 1 && ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                        ctx.recover(owner, 1, None);
                    } else {
                        draw_direct(ctx, owner, 1);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "shouhu" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::SelectCard, ASK_OW_SHOUHU_CARD),
            HostStage::SelectCard => {
                let Some(&cid) = answered_cards(ctx).first() else {
                    return Flow::Done;
                };
                m.cards = vec![cid];
                wait_ow(m, ctx, HostStage::TargetChoice, ASK_OW_SHOUHU_TARGET)
            }
            HostStage::TargetChoice => {
                let Some(&t) = answered_players(ctx).first() else {
                    return Flow::Done;
                };
                let Some(cid) = m.cards.first().copied() else {
                    return Flow::Done;
                };
                ctx.log(LogEvent::SkillTriggered {
                    skill: "shouhu",
                    owner,
                    presentation: None,
                });
                ctx.discard_from_hand(owner, cid);
                if ctx.core.player(t).alive {
                    ctx.recover(t, 1, None);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "shanxian" => match m.stage {
            HostStage::Act => {
                if m.cards == [2] {
                    set_flag(ctx.core, owner, "ow_shanxian_target", 0);
                    ctx.handoff_after_shanxian(owner);
                    return Flow::Done;
                }
                wait_ow(m, ctx, HostStage::SelectNature, ASK_OW_SHANXIAN_CONFIRM)
            }
            HostStage::SelectNature => {
                if !answered_confirm(ctx) {
                    if m.cards == [1] {
                        if let Some(victim_b) = m.target {
                            ctx.resume_facedown_turn_skip(victim_b);
                        }
                    }
                    return Flow::Done;
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "shanxian",
                    owner,
                    presentation: None,
                });
                m.stage = HostStage::DrawSettle;
                ctx.start_draw(owner, 1);
                Flow::Yield
            }
            HostStage::DrawSettle => {
                // 清理历史快照键，再记录现行手牌与装备快照
                let stale_keys: Vec<String> = ctx
                    .core
                    .vm_aux
                    .flags
                    .keys()
                    .filter_map(|(p, k)| {
                        (*p == owner
                            && (k.starts_with("ow_shanxian_h:") || k.starts_with("ow_shanxian_e:")))
                        .then(|| k.clone())
                    })
                    .collect();
                for k in stale_keys {
                    ctx.core.vm_aux.flags.remove(&(owner, k));
                }
                let hs = ctx.core.player(owner).hand.clone();
                let es = ctx.core.player(owner).equips.clone();
                set_flag(ctx.core, owner, "ow_shanxian_h_len", hs.len() as u32);
                for (i, c) in hs.iter().enumerate() {
                    set_flag(ctx.core, owner, &format!("ow_shanxian_h:{i}"), *c);
                }
                set_flag(ctx.core, owner, "ow_shanxian_e_len", es.len() as u32);
                for (i, c) in es.iter().enumerate() {
                    set_flag(ctx.core, owner, &format!("ow_shanxian_e:{i}"), *c);
                }
                set_flag(ctx.core, owner, "ow_shanxian_n", 1);
                remove_skill_u05(ctx, owner, "shanxian2");
                let victim_b = m.target.unwrap_or(owner);
                set_flag(ctx.core, owner, "ow_shanxian_target", u32::from(victim_b) + 1);
                set_flag(ctx.core, owner, "ow_shanxian_pending_turnover", 1);
                set_flag(ctx.core, 0, "ow_shanxian_resume_target", u32::from(victim_b) + 1);
                if m.cards == [1] {
                    ctx.start_hijacked_turn(owner);
                } else {
                    let fid = ctx
                        .res
                        .stack
                        .iter()
                        .rev()
                        .find(|f| f.kind == EventKind::TurnBegin)
                        .map(|f| f.id)
                        .unwrap_or_else(|| ctx.current_event_frame_id());
                    set_flag(
                        ctx.core,
                        0,
                        &format!("ow_shanxian_hijack_frame:{fid}"),
                        u32::from(owner) + 1,
                    );
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "shanxian2" => {
            remove_skill_u05(ctx, owner, "shanxian2");
            Flow::Done
        }
        "shanhui" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::SelectNature, ASK_OW_SHANHUI_CONFIRM),
            HostStage::SelectNature => {
                if !answered_confirm(ctx) {
                    return Flow::Done;
                }
                let h_len = flag(ctx.core, owner, "ow_shanxian_h_len") as usize;
                let e_len = flag(ctx.core, owner, "ow_shanxian_e_len") as usize;
                let mut h_ids = Vec::with_capacity(h_len);
                for i in 0..h_len {
                    if let Some(c) = ctx
                        .core
                        .vm_aux
                        .flags
                        .get(&(owner, format!("ow_shanxian_h:{i}")))
                        .copied()
                    {
                        h_ids.push(c);
                    }
                }
                let mut e_ids = Vec::with_capacity(e_len);
                for i in 0..e_len {
                    if let Some(c) = ctx
                        .core
                        .vm_aux
                        .flags
                        .get(&(owner, format!("ow_shanxian_e:{i}")))
                        .copied()
                    {
                        e_ids.push(c);
                    }
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "shanhui",
                    owner,
                    presentation: None,
                });
                let old_he = hand_and_equips(ctx.core, owner);
                let old_count = old_he.len();
                for c in old_he {
                    discard_he_card(ctx, owner, c);
                }
                for c in &h_ids {
                    reclaim_ow_card(ctx, owner, *c, false);
                }
                for c in &e_ids {
                    reclaim_ow_card(ctx, owner, *c, true);
                }
                if old_count > h_ids.len() + e_ids.len() && ctx.core.player(owner).alive {
                    ctx.recover(owner, 1, None);
                }
                let n = flag(ctx.core, owner, "ow_shanxian_n").saturating_sub(1);
                set_flag(ctx.core, owner, "ow_shanxian_n", n);
                if n == 0 {
                    // 仅清闪回快照字段；保留 `ow_shanxian_pending_turnover` 与 `ow_shanxian_target` 至闪现插队回合结束（`ow/skill.js:3305-3308`）
                    let keys: Vec<String> = ctx
                        .core
                        .vm_aux
                        .flags
                        .keys()
                        .filter_map(|(p, k)| {
                            (*p == owner
                                && (k == "ow_shanxian_n"
                                    || k == "ow_shanxian_h_len"
                                    || k.starts_with("ow_shanxian_h:")
                                    || k == "ow_shanxian_e_len"
                                    || k.starts_with("ow_shanxian_e:")))
                                .then(|| k.clone())
                        })
                        .collect();
                    for k in keys {
                        ctx.core.vm_aux.flags.remove(&(owner, k));
                    }
                } else {
                    grant_skill_u05(ctx, owner, "shanxian2", false);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        _ => Flow::Done,
    }
}
