//! U05-3 (`shaai-ow`) Batch C（标签 1042）：黑百合 / D.Va / 死神 / 士兵76 / 源氏（含连破，27 项）。

use super::u05_ow::{
    alive_others, answered_cards, answered_confirm, answered_players, answered_tag,
    claim_off_deck_supply, damage_card_suit, damage_is_sha_like, discard_he_card, draw_direct,
    flag, grant_skill_u05, grant_temp_skill, hand_and_equips, has_skill_u05, is_black_card,
    is_red_card, mujing_sha_usable, mujing_valid_targets, random_discard_he, refund_once_used,
    remove_skill_u05, set_flag, suit_of, valid_single_sha_target, wait_ow, with_current_damage_mut,
    ASK_OW_DULEI_CARD, ASK_OW_FEIREN3_CONFIRM, ASK_OW_JUJI_CARDS, ASK_OW_JUJI_TARGET,
    ASK_OW_LIANPO_CONFIRM, ASK_OW_LICHANG2_DR, ASK_OW_LICHANG2_TARGET, ASK_OW_LICHANG_CARD,
    ASK_OW_MUJING_CARD, ASK_OW_MUJING_TARGET, ASK_OW_TUIJIN_TARGET, ASK_OW_YIHUN_CARD,
    ASK_OW_YIHUN_TARGET, ASK_OW_ZHANLONG_CONFIRM, ASK_OW_ZIHUI_DISCARD,
};
use super::*;
use crate::catalog::{CardKind, QueryCtx, Suit, TriggerMeta};
use crate::event::{DamageNature, EventData, EventKind, Frame, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{card_use_as, damage_data_with, Ctx, Flow, LogEvent};

pub(super) fn trigger_meta_c(name: &str) -> Option<TriggerMeta> {
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
        "duwen" => Some(forced(EventKind::DamageBegin, Timing::Normal, 1, Vec::new())),
        "duwen2" => Some(forced(EventKind::DamageEnd, Timing::Normal, 1, Vec::new())),
        "juji2" => Some(forced(EventKind::TurnPhaseAfter, Timing::Normal, 1, Vec::new())),
        "dulei2" => Some(forced(EventKind::CardUsed, Timing::Normal, 1, Vec::new())),
        "chongzhuang" => Some(forced(EventKind::DamageEnd, Timing::Normal, 1, Vec::new())),
        "shouge" => Some(forced(EventKind::Death, Timing::Normal, 1, Vec::new())),
        "yihun" => Some(forced(EventKind::TurnPhaseEnd, Timing::Normal, 1, Vec::new())),
        "yihun2" => Some(forced(
            EventKind::TurnBegin,
            Timing::First,
            3,
            vec![EventKind::Death],
        )),
        "lichang" => Some(forced(EventKind::TurnPhaseEnd, Timing::Normal, 1, Vec::new())),
        "lichang2" => Some(forced(EventKind::TurnBegin, Timing::Normal, 1, Vec::new())),
        "zhanlong" => Some(forced(EventKind::TurnBegin, Timing::Normal, 1, Vec::new())),
        "feiren" => Some(forced(EventKind::DamageBegin, Timing::Normal, 1, Vec::new())),
        "lianpo" => Some(forced(EventKind::TurnPhaseAfter, Timing::Normal, 1, Vec::new())),
        _ => None,
    }
}

pub(super) fn can_activate_c(name: &str, ctx: &QueryCtx<'_>, owner: PlayerId) -> bool {
    let st = ctx.state;
    let p = st.player(owner);
    match name {
        "juji" => {
            (!p.hand.is_empty() || !p.equips.is_empty())
                && alive_others(st, owner)
                    .into_iter()
                    .any(|t| !st.player(t).hand.is_empty())
        }
        "dulei" => !has_skill_u05(st, ctx.catalog, owner, "dulei2") && !p.hand.is_empty(),
        "tuijin" => {
            p.jijia_hp > 0
                && alive_others(st, owner)
                    .into_iter()
                    .any(|t| st.computed_distance(ctx.catalog, owner, t) > 1)
        }
        "zihui" => {
            p.jijia_hp > 0
                && alive_others(st, owner)
                    .into_iter()
                    .any(|t| st.computed_distance(ctx.catalog, owner, t) <= 2)
        }
        "mujing" => {
            mujing_sha_usable(st, ctx.catalog, owner)
                && !mujing_valid_targets(st, ctx.catalog, owner).is_empty()
                && p.hand
                    .iter()
                    .chain(p.equips.iter())
                    .any(|c| is_black_card(st, ctx.catalog, owner, *c))
        }
        _ => false,
    }
}

pub(super) fn filter_c(
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
        "duwen" => matches!(
            event,
            EventData::Damage(d) if d.source == Some(owner)
                && !d.from_chain
                && p.hand.len() == st.player(d.target).hand.len()
        ),
        "duwen2" => matches!(
            event,
            EventData::Damage(d) if d.source == Some(owner)
                && !d.from_chain
                && damage_is_sha_like(frames, frame, ctx.catalog, d)
                && p.hp == st.player(d.target).hp
        ),
        "juji2" => matches!(
            event,
            EventData::TurnEndWindow { player, after: true } if *player == owner
        ),
        "dulei2" => match event {
            EventData::CardUsed(u) if u.player != owner && !u.cancelled => {
                let Some(trap) = st
                    .vm_aux
                    .piles
                    .get(&(owner, "ow_dulei".to_string()))
                    .and_then(|v| v.first().copied())
                else {
                    return false;
                };
                u.targets.contains(&owner)
                    && u.suit(ctx.catalog) == Some(suit_of(st, ctx.catalog, owner, trap))
            }
            _ => false,
        },
        "chongzhuang" => matches!(
            event,
            EventData::Damage(d) if d.source == Some(owner)
                && d.amount > 0
                && p.jijia_hp == 0
        ),
        "shouge" => matches!(
            event,
            EventData::Death { killer, .. } if *killer == Some(owner)
        ),
        "yihun" => matches!(
            event,
            EventData::TurnEndWindow { player, after: false }
                if *player == owner
                    && !has_skill_u05(st, ctx.catalog, owner, "yihun2")
                    && !alive_others(st, owner).is_empty()
                    && p.hand.iter().chain(p.equips.iter()).any(|c| is_black_card(st, ctx.catalog, owner, *c))
        ),
        "yihun2" => match event {
            EventData::TurnBegin { player } | EventData::Death { player, .. } => {
                flag(st, owner, "ow_yihun2_target") == u32::from(*player) + 1
            }
            _ => false,
        },
        "lichang" => matches!(
            event,
            EventData::TurnEndWindow { player, after: false }
                if *player == owner
                    && p.hand.iter().chain(p.equips.iter()).any(|c| is_red_card(st, ctx.catalog, owner, *c))
        ),
        "lichang2" => matches!(
            event,
            EventData::TurnBegin { player }
                if *player == owner
                    && st.alive_players().any(|t| st.computed_distance(ctx.catalog, owner, t) <= 1)
        ),
        "zhanlong" => matches!(
            event,
            EventData::TurnBegin { player }
                if *player == owner
                    && flag(st, owner, "ow_zhanlong_used") == 0
                    && p.hp == 1
                    && (!p.hand.is_empty() || !p.equips.is_empty())
        ),
        "feiren" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.from_chain
                    && damage_is_sha_like(frames, frame, ctx.catalog, d)
                    && damage_card_suit(frames, frame, st, ctx.catalog, d) == Some(Suit::Spade)
        ),
        "lianpo" => matches!(
            event,
            EventData::TurnEndWindow { after: true, .. } if flag(st, owner, "ow_turn_kill_count") > 0
        ),
        _ => false,
    }
}

fn zihui_step_next(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let num = m.cards.get(1).copied().unwrap_or(1) as usize;
    while (m.cards[0] as usize) < m.targets.len() {
        let idx = m.cards[0] as usize;
        let t = m.targets[idx];
        m.cards[0] = (idx + 1) as u32;
        if !ctx.core.player(t).alive {
            continue;
        }
        if hand_and_equips(ctx.core, t).len() < num {
            m.stage = HostStage::DrawSettle;
            ctx.emit(EventData::Damage(damage_data_with(
                Some(owner),
                t,
                2,
                DamageNature::Fire,
                None,
            )));
            return Flow::Yield;
        }
        m.target = Some(t);
        return wait_ow(m, ctx, HostStage::TargetChoice, ASK_OW_ZIHUI_DISCARD);
    }
    ctx.core.player_mut(owner).jijia_hp = 0;
    Flow::Done
}

pub(super) fn step_c(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "duwen" => {
            ctx.log(LogEvent::SkillTriggered {
                skill: "duwen",
                owner,
                presentation: None,
            });
            with_current_damage_mut(ctx, |dm| dm.amount += 1);
            Flow::Done
        }
        "duwen2" => {
            ctx.log(LogEvent::SkillTriggered {
                skill: "duwen2",
                owner,
                presentation: None,
            });
            draw_direct(ctx, owner, 2);
            Flow::Done
        }
        "juji" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::SelectCard, ASK_OW_JUJI_CARDS),
            HostStage::SelectCard => {
                let picks = answered_cards(ctx);
                if picks.is_empty() || picks.len() > 4 {
                    refund_once_used(ctx, owner, "juji");
                    return Flow::Done;
                }
                let mut seen_suits = Vec::new();
                for &c in &picks {
                    let s = suit_of(ctx.core, ctx.catalog, owner, c);
                    if seen_suits.contains(&s) {
                        return wait_ow(m, ctx, HostStage::SelectCard, ASK_OW_JUJI_CARDS);
                    }
                    seen_suits.push(s);
                }
                m.cards = picks;
                wait_ow(m, ctx, HostStage::TargetChoice, ASK_OW_JUJI_TARGET)
            }
            HostStage::TargetChoice => {
                let Some(&t) = answered_players(ctx).first() else {
                    refund_once_used(ctx, owner, "juji");
                    return Flow::Done;
                };
                ctx.log(LogEvent::SkillTriggered {
                    skill: "juji",
                    owner,
                    presentation: None,
                });
                let mut disc_mask = 0u8;
                let picked_cards = m.cards.clone();
                for c in picked_cards {
                    let s = suit_of(ctx.core, ctx.catalog, owner, c);
                    disc_mask |= match s {
                        Suit::Spade => 1,
                        Suit::Heart => 2,
                        Suit::Club => 4,
                        Suit::Diamond => 8,
                    };
                    discard_he_card(ctx, owner, c);
                }
                let hit = ctx.core.player(t).hand.iter().any(|&hc| {
                    let bit = match suit_of(ctx.core, ctx.catalog, t, hc) {
                        Suit::Spade => 1,
                        Suit::Heart => 2,
                        Suit::Club => 4,
                        Suit::Diamond => 8,
                    };
                    (disc_mask & bit) != 0
                });
                if hit && ctx.core.player(t).alive {
                    grant_temp_skill(ctx, owner, "juji2");
                    grant_temp_skill(ctx, owner, "juji3");
                    set_flag(ctx.core, owner, "ow_juji2_target", u32::from(t) + 1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "juji2" => {
            set_flag(ctx.core, owner, "ow_juji2_target", 0);
            remove_skill_u05(ctx, owner, "juji2");
            remove_skill_u05(ctx, owner, "juji3");
            Flow::Done
        }
        "dulei" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::SelectCard, ASK_OW_DULEI_CARD),
            HostStage::SelectCard => {
                let Some(&cid) = answered_cards(ctx).first() else {
                    return Flow::Done;
                };
                if !ctx.core.player(owner).hand.contains(&cid) {
                    return Flow::Done;
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dulei",
                    owner,
                    presentation: None,
                });
                ctx.take_hand_card(owner, cid);
                ctx.core
                    .vm_aux
                    .piles
                    .insert((owner, "ow_dulei".to_string()), vec![cid]);
                grant_skill_u05(ctx, owner, "dulei2", false);
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dulei2" => {
            let attacker = m.target.unwrap_or(owner);
            ctx.log(LogEvent::SkillTriggered {
                skill: "dulei2",
                owner,
                presentation: None,
            });
            if let Some(cards) = ctx.core.vm_aux.piles.remove(&(owner, "ow_dulei".to_string())) {
                for c in cards {
                    ctx.to_discard(c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: owner,
                        cards: vec![c],
                    });
                }
            }
            remove_skill_u05(ctx, owner, "dulei2");
            if ctx.core.player(attacker).alive {
                random_discard_he(ctx, attacker);
                ctx.emit(EventData::LoseHp {
                    player: attacker,
                    amount: 1,
                });
            }
            Flow::Done
        }
        "chongzhuang" => {
            let amt = m.cards.get(1).copied().unwrap_or(0);
            let charge = flag(ctx.core, owner, "ow_jijia_charge") + amt;
            if charge >= 4 {
                ctx.core.player_mut(owner).jijia_hp = 4;
                set_flag(ctx.core, owner, "ow_jijia_charge", 0);
                ctx.log(LogEvent::SkillTriggered {
                    skill: "chongzhuang",
                    owner,
                    presentation: None,
                });
            } else {
                set_flag(ctx.core, owner, "ow_jijia_charge", charge);
            }
            Flow::Done
        }
        "tuijin" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::TargetChoice, ASK_OW_TUIJIN_TARGET),
            HostStage::TargetChoice => {
                let Some(&t) = answered_players(ctx).first() else {
                    refund_once_used(ctx, owner, "tuijin");
                    return Flow::Done;
                };
                ctx.log(LogEvent::SkillTriggered {
                    skill: "tuijin",
                    owner,
                    presentation: None,
                });
                grant_temp_skill(ctx, owner, "tuijin2");
                set_flag(ctx.core, owner, "ow_tuijin_target", u32::from(t) + 1);
                Flow::Done
            }
            _ => Flow::Done,
        },
        "zihui" => match m.stage {
            HostStage::Act => {
                let targets: Vec<PlayerId> = alive_others(ctx.core, owner)
                    .into_iter()
                    .filter(|t| ctx.core.computed_distance(ctx.catalog, owner, *t) <= 2)
                    .collect();
                if targets.is_empty() || ctx.core.player(owner).jijia_hp == 0 {
                    return Flow::Done;
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "zihui",
                    owner,
                    presentation: None,
                });
                let n = u32::from(ctx.core.player(owner).jijia_hp);
                m.targets = targets;
                m.cards = vec![0, n];
                zihui_step_next(m, ctx)
            }
            HostStage::TargetChoice => {
                let Some(t) = m.target else { return Flow::Done };
                let n = m.cards.get(1).copied().unwrap_or(1) as usize;
                let picks = answered_cards(ctx);
                let he = hand_and_equips(ctx.core, t);
                let valid = picks.len() == n && picks.iter().all(|c| he.contains(c));
                if valid {
                    for c in picks {
                        discard_he_card(ctx, t, c);
                    }
                    zihui_step_next(m, ctx)
                } else {
                    m.stage = HostStage::DrawSettle;
                    ctx.emit(EventData::Damage(damage_data_with(
                        Some(owner),
                        t,
                        2,
                        DamageNature::Fire,
                        None,
                    )));
                    Flow::Yield
                }
            }
            HostStage::DrawSettle => zihui_step_next(m, ctx),
            _ => Flow::Done,
        },
        "shouge" => {
            ctx.log(LogEvent::SkillTriggered {
                skill: "shouge",
                owner,
                presentation: None,
            });
            claim_off_deck_supply(ctx, owner, CardKind::Zhiliaobo, "ow_zhiliaobo_", false);
            Flow::Done
        }
        "yihun" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::SelectCard, ASK_OW_YIHUN_CARD),
            HostStage::SelectCard => {
                let Some(&cid) = answered_cards(ctx).first() else {
                    return Flow::Done;
                };
                m.cards = vec![cid];
                wait_ow(m, ctx, HostStage::TargetChoice, ASK_OW_YIHUN_TARGET)
            }
            HostStage::TargetChoice => {
                let Some(&t) = answered_players(ctx).first() else {
                    return Flow::Done;
                };
                let Some(cid) = m.cards.first().copied() else {
                    return Flow::Done;
                };
                ctx.log(LogEvent::SkillTriggered {
                    skill: "yihun",
                    owner,
                    presentation: None,
                });
                discard_he_card(ctx, owner, cid);
                if ctx.core.player(t).alive {
                    grant_skill_u05(ctx, owner, "yihun2", false);
                    set_flag(ctx.core, owner, "ow_yihun2_target", u32::from(t) + 1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "yihun2" => {
            let t_opt = flag(ctx.core, owner, "ow_yihun2_target")
                .checked_sub(1)
                .map(|v| v as PlayerId);
            set_flag(ctx.core, owner, "ow_yihun2_target", 0);
            remove_skill_u05(ctx, owner, "yihun2");
            if let Some(t) = t_opt {
                if ctx.core.player(t).alive && ctx.core.player(owner).alive {
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "yihun2",
                        owner,
                        presentation: None,
                    });
                    ctx.use_virtual_card(owner, CardKind::Sha, vec![t]);
                }
            }
            Flow::Done
        }
        "mujing" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::SelectCard, ASK_OW_MUJING_CARD),
            HostStage::SelectCard => {
                let Some(&cid) = answered_cards(ctx).first() else {
                    return Flow::Done;
                };
                m.cards = vec![cid];
                wait_ow(m, ctx, HostStage::TargetChoice, ASK_OW_MUJING_TARGET)
            }
            HostStage::TargetChoice => {
                let picked = answered_players(ctx);
                if picked.is_empty() {
                    return Flow::Done;
                }
                let Some(cid) = m.cards.first().copied() else {
                    return Flow::Done;
                };
                if !hand_and_equips(ctx.core, owner).contains(&cid) {
                    return Flow::Done;
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "mujing",
                    owner,
                    presentation: None,
                });
                ctx.emit(EventData::CardUsed(card_use_as(
                    cid,
                    owner,
                    picked,
                    Some(CardKind::Sha),
                )));
                Flow::Done
            }
            _ => Flow::Done,
        },
        "lichang" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::SelectCard, ASK_OW_LICHANG_CARD),
            HostStage::SelectCard => {
                let Some(&cid) = answered_cards(ctx).first() else {
                    return Flow::Done;
                };
                ctx.log(LogEvent::SkillTriggered {
                    skill: "lichang",
                    owner,
                    presentation: None,
                });
                discard_he_card(ctx, owner, cid);
                grant_skill_u05(ctx, owner, "lichang2", false);
                Flow::Done
            }
            _ => Flow::Done,
        },
        "lichang2" => match m.stage {
            HostStage::Act => {
                remove_skill_u05(ctx, owner, "lichang2");
                wait_ow(m, ctx, HostStage::TargetChoice, ASK_OW_LICHANG2_TARGET)
            }
            HostStage::TargetChoice => {
                let Some(&t) = answered_players(ctx).first() else {
                    return Flow::Done;
                };
                if !ctx.core.player(t).alive {
                    return Flow::Done;
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "lichang2",
                    owner,
                    presentation: None,
                });
                m.target = Some(t);
                wait_ow(m, ctx, HostStage::SelectNature, ASK_OW_LICHANG2_DR)
            }
            HostStage::SelectNature => {
                let Some(t) = m.target else { return Flow::Done };
                let tag = answered_tag(ctx).unwrap_or(0);
                if tag == 1 && ctx.core.player(t).hp < ctx.core.player(t).max_hp as i32 {
                    ctx.recover(t, 1, None);
                } else {
                    draw_direct(ctx, t, 2);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "zhanlong" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::SelectNature, ASK_OW_ZHANLONG_CONFIRM),
            HostStage::SelectNature => {
                if !answered_confirm(ctx) {
                    return Flow::Done;
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "zhanlong",
                    owner,
                    presentation: None,
                });
                set_flag(ctx.core, owner, "ow_zhanlong_used", 1);
                for c in hand_and_equips(ctx.core, owner) {
                    discard_he_card(ctx, owner, c);
                }
                grant_temp_skill(ctx, owner, "zhanlong2");
                for _ in 0..3 {
                    claim_off_deck_supply(ctx, owner, CardKind::Sha, "ow_sha_", false);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "feiren" => {
            ctx.log(LogEvent::SkillTriggered {
                skill: "feiren",
                owner,
                presentation: None,
            });
            with_current_damage_mut(ctx, |dm| dm.amount += 1);
            Flow::Done
        }
        "feiren3" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::SelectNature, ASK_OW_FEIREN3_CONFIRM),
            HostStage::SelectNature => {
                if !answered_confirm(ctx) {
                    return Flow::Done;
                }
                let targets: Vec<PlayerId> = m
                    .targets
                    .iter()
                    .copied()
                    .filter(|t| valid_single_sha_target(ctx.core, ctx.catalog, owner, *t))
                    .collect();
                if targets.is_empty() {
                    return Flow::Done;
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "feiren3",
                    owner,
                    presentation: None,
                });
                let nature_code = m.cards.first().copied().unwrap_or(0);
                let (as_kind, nature) = match nature_code {
                    1 => (CardKind::FireSha, DamageNature::Fire),
                    2 => (CardKind::LeiSha, DamageNature::Thunder),
                    _ => (CardKind::Sha, DamageNature::Normal),
                };
                set_flag(ctx.core, owner, "ow_feiren3_skip_next", 1);
                // 优先取离线黑桃杀置于桌面重发；若全被占用则回落虚拟杀+单帧黑桃覆写
                let spade_cid: Option<CardId> = ctx
                    .core
                    .detached_cards
                    .iter()
                    .position(|(c, _)| {
                        ctx.catalog.card_kind(*c) == CardKind::Sha
                            && ctx.catalog.cards[*c as usize].suit == Suit::Spade
                            && ctx.catalog.cards[*c as usize].name.starts_with("ow_sha_")
                    })
                    .map(|idx| ctx.core.detached_cards.remove(idx).0)
                    .or_else(|| {
                        ctx.core
                            .discard
                            .iter()
                            .position(|c| {
                                ctx.catalog.card_kind(*c) == CardKind::Sha
                                    && ctx.catalog.cards[*c as usize].suit == Suit::Spade
                                    && ctx.catalog.cards[*c as usize].name.starts_with("ow_sha_")
                            })
                            .map(|idx| ctx.core.discard.remove(idx))
                    });
                if let Some(cid) = spade_cid {
                    ctx.core.table.push(cid);
                    ctx.emit(EventData::CardUsed(crate::machine::card_use_with_nature(
                        cid,
                        owner,
                        targets,
                        nature,
                        Some(as_kind),
                    )));
                } else {
                    ctx.use_virtual_card_with_suit_and_nature(
                        owner,
                        as_kind,
                        nature,
                        true,
                        targets,
                    );
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "lianpo" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::SelectNature, ASK_OW_LIANPO_CONFIRM),
            HostStage::SelectNature => {
                set_flag(ctx.core, owner, "ow_turn_kill_count", 0);
                if !answered_confirm(ctx) {
                    return Flow::Done;
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "lianpo",
                    owner,
                    presentation: None,
                });
                set_flag(ctx.core, owner, "extra_turn_insert_queued", 1);
                Flow::Done
            }
            _ => Flow::Done,
        },
        _ => Flow::Done,
    }
}
