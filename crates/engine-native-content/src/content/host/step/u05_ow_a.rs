//! U05-3 (`shaai-ow`) Batch A（标签 1040）：奥丽莎 / 温斯顿 / 堡垒 / 查莉娅 / 路霸 / 黑影（20 项）。

use super::u05_ow::{
    alive_others, answered_cards, answered_confirm, answered_players, answered_tag,
    discard_he_card, draw_direct, flag, grant_skill_u05, grant_temp_skill, has_skill_u05,
    random_discard_he, refund_once_used, remove_skill_u05, set_flag, suit_of, turn_over_to_back,
    wait_ow, ASK_OW_DIANJI_CARD, ASK_OW_FEITIAO_CARD, ASK_OW_FEITIAO_TARGET, ASK_OW_LIANGOU_CARD,
    ASK_OW_LIANGOU_TARGET, ASK_OW_PINGZHANG_CARD, ASK_OW_QIANGGU2_DISCARD, ASK_OW_QIANGGU_CARDS,
    ASK_OW_QINRU_CONFIRM, ASK_OW_WOLIU_TARGETS, ASK_OW_XIYANG_CONFIRM, ASK_OW_ZHENCHA_DR,
    ASK_OW_ZHENCHA_MODE,
};
use super::*;
use crate::catalog::{CardKind, QueryCtx, Suit, TriggerMeta};
use crate::event::{DamageNature, EventData, EventKind, Frame, LostOrigin, Phase, Timing};
use crate::ids::PlayerId;
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn trigger_meta_a(name: &str) -> Option<TriggerMeta> {
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
        "woliu" => Some(forced(
            EventKind::TurnPhaseEnd,
            Timing::Normal,
            1,
            vec![EventKind::TurnBegin, EventKind::Death],
        )),
        "woliu2" => Some(forced(EventKind::CardUsed, Timing::First, 2, Vec::new())),
        "qianggu2" => Some(forced(EventKind::TurnBegin, Timing::Normal, 2, Vec::new())),
        "feitiao" => Some(forced(EventKind::PhaseBegin, Timing::Normal, 1, Vec::new())),
        "zhencha" | "bshaowei" => {
            Some(forced(EventKind::TurnPhaseEnd, Timing::Normal, 1, Vec::new()))
        }
        "pingzhang" => Some(forced(
            EventKind::DamageBefore,
            Timing::Normal,
            1,
            vec![EventKind::TurnBegin],
        )),
        "liangou2" => Some(forced(EventKind::DamageBegin, Timing::Normal, 1, Vec::new())),
        "xiyang" => Some(forced(EventKind::TurnPhaseEnd, Timing::Normal, 1, Vec::new())),
        "qinru" => Some(forced(
            EventKind::TurnPhaseAfter,
            Timing::Normal,
            2,
            Vec::new(),
        )),
        "yinshen" => Some(forced(
            EventKind::CardsLost,
            Timing::Normal,
            1,
            vec![
                EventKind::CardUsed,
                EventKind::CardResponded,
                EventKind::CardsMoved,
                EventKind::CardsDiscarded,
            ],
        )),
        "maichong" => Some(forced(EventKind::CardUsed, Timing::Normal, 1, Vec::new())),
        _ => None,
    }
}

pub(super) fn can_activate_a(name: &str, ctx: &QueryCtx<'_>, owner: PlayerId) -> bool {
    let p = ctx.state.player(owner);
    match name {
        "qianggu" => p.hand.len() + p.equips.len() >= 2,
        "dianji" => p.hand.iter().any(|&c| {
            alive_others(ctx.state, owner).into_iter().any(|t| {
                ctx.state.computed_distance(ctx.catalog, owner, t) <= 2
                    && ctx.catalog.ow_target_enabled(
                        ctx.state,
                        owner,
                        t,
                        Some(c),
                        CardKind::Jingleishan,
                        DamageNature::Thunder,
                    )
            })
        }),
        "liangou" => {
            (!p.hand.is_empty() || !p.equips.is_empty())
                && !alive_others(ctx.state, owner).is_empty()
        }
        _ => false,
    }
}

pub(super) fn filter_a(
    name: &str,
    ctx: &QueryCtx<'_>,
    _frames: &[Frame],
    _frame: &Frame,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let st = ctx.state;
    let p = st.player(owner);
    match name {
        "woliu" => match event {
            EventData::TurnEndWindow { player, after: false } => {
                *player == owner && !alive_others(st, owner).is_empty()
            }
            EventData::TurnBegin { player } | EventData::Death { player, .. } => *player == owner,
            _ => false,
        },
        "woliu2" => matches!(
            event,
            EventData::CardUsed(u) if !u.cancelled
                && u.effective_kind(ctx.catalog).is_sha_like()
                && u.targets.contains(&owner)
                && st.alive_players().any(|other| {
                    other != owner
                        && other != u.player
                        && !u.targets.contains(&other)
                        && has_skill_u05(st, ctx.catalog, other, "woliu2")
                })
        ),
        "qianggu2" => matches!(event, EventData::TurnBegin { player } if *player == owner),
        "feitiao" => matches!(
            event,
            EventData::Phase { player, phase: Phase::Play, begin: true }
                if *player == owner && (!p.hand.is_empty() || !p.equips.is_empty()) && !alive_others(st, owner).is_empty()
        ),
        "zhencha" => matches!(
            event,
            EventData::TurnEndWindow { player, after: false }
                if *player == owner && !has_skill_u05(st, ctx.catalog, owner, "zhencha2") && p.baolei_mode == 1
        ),
        "bshaowei" => matches!(
            event,
            EventData::TurnEndWindow { player, after: false }
                if *player == owner && !has_skill_u05(st, ctx.catalog, owner, "zhencha2") && p.baolei_mode != 1
        ),
        "pingzhang" => match event {
            EventData::Damage(d) if d.amount > 0 => {
                let want = if d.target == owner {
                    if has_skill_u05(st, ctx.catalog, owner, "pingzhang2") {
                        return false;
                    }
                    Suit::Heart
                } else {
                    if has_skill_u05(st, ctx.catalog, owner, "pingzhang3") {
                        return false;
                    }
                    Suit::Spade
                };
                p.hand
                    .iter()
                    .chain(p.equips.iter())
                    .any(|c| suit_of(st, ctx.catalog, owner, *c) == want)
            }
            EventData::TurnBegin { player } => *player == owner,
            _ => false,
        },
        "liangou2" => matches!(
            event,
            EventData::Damage(d) if d.target == owner && flag(st, owner, "ow_liangou2_dmg") > 0
        ),
        "xiyang" => matches!(
            event,
            EventData::TurnEndWindow { player, after: false }
                if *player == owner && !p.face_down && p.hp < p.max_hp as i32
        ),
        "qinru" => matches!(
            event,
            EventData::TurnEndWindow { player, after: true } if *player == owner
        ),
        "yinshen" => match event {
            _ if p.yinshen_active => false,
            EventData::CardsLost { player, cards, origins } => {
                *player == owner
                    && cards.iter().zip(origins.iter()).any(|(c, o)| {
                        *o == LostOrigin::Hand && ctx.catalog.card_kind(*c).is_basic_kind()
                    })
                    && p.hand.iter().all(|c| !ctx.catalog.card_kind(*c).is_basic_kind())
            }
            EventData::CardUsed(u) if !u.cancelled && u.player == owner => {
                u.card
                    .is_some_and(|c| ctx.catalog.card_kind(c).is_basic_kind() && !p.hand.contains(&c))
                    && p.hand.iter().all(|c| !ctx.catalog.card_kind(*c).is_basic_kind())
            }
            EventData::CardResponded { player, card } if *player == owner => {
                ctx.catalog.card_kind(*card).is_basic_kind()
                    && p.hand.iter().all(|c| !ctx.catalog.card_kind(*c).is_basic_kind())
            }
            EventData::CardsMoved { from, cards, .. } if *from == owner => {
                cards.iter().any(|c| ctx.catalog.card_kind(*c).is_basic_kind())
                    && p.hand.iter().all(|c| !ctx.catalog.card_kind(*c).is_basic_kind())
            }
            EventData::CardsDiscarded { player, cards } if *player == owner => {
                cards.iter().any(|c| ctx.catalog.card_kind(*c).is_basic_kind())
                    && p.hand.iter().all(|c| !ctx.catalog.card_kind(*c).is_basic_kind())
            }
            _ => false,
        },
        "maichong" => match event {
            EventData::CardUsed(u) if !u.cancelled && u.player == owner => {
                let Some(cid) = u.card else { return false };
                let k = ctx.catalog.card_kind(cid);
                let real_trick = u.use_as.is_none_or(|uk| uk == k)
                    && !k.is_basic_kind()
                    && k.equip_slot().is_none()
                    && k.is_basic_trick_or_equip()
                    && !matches!(k, CardKind::Le | CardKind::Bing | CardKind::ShanDian);
                real_trick
                    && alive_others(st, owner).into_iter().any(|t| {
                        flag(st, owner, &format!("ow_qinru_turn:{t}")) > 0
                            && (!st.player(t).hand.is_empty() || !st.player(t).equips.is_empty())
                    })
            }
            _ => false,
        },
        _ => false,
    }
}

pub(super) fn step_a(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "woliu" => match m.stage {
            HostStage::Act => {
                let is_turn_end =
                    m.cards.len() == 1 && m.cards[0] == 0 && m.target == Some(owner);
                if is_turn_end {
                    let has_woliu2_end = ctx.core.alive_players().any(|p| p != owner && !has_skill_u05(ctx.core, ctx.catalog, p, "woliu2"));
                    let _ = has_woliu2_end;
                    if alive_others(ctx.core, owner).is_empty() {
                        return Flow::Done;
                    }
                    wait_ow(m, ctx, HostStage::TargetChoice, ASK_OW_WOLIU_TARGETS)
                } else {
                    // TurnBegin / Death clear woliu2
                    let seats = ctx.core.players.len() as PlayerId;
                    for p in 0..seats {
                        remove_skill_u05(ctx, p, "woliu2");
                    }
                    set_flag(ctx.core, owner, "ow_woliu_used", 0);
                    Flow::Done
                }
            }
            HostStage::TargetChoice => {
                let picked = answered_players(ctx);
                if picked.is_empty() {
                    return Flow::Done;
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "woliu",
                    owner,
                    presentation: None,
                });
                set_flag(ctx.core, owner, "ow_woliu_used", 1);
                grant_skill_u05(ctx, owner, "woliu2", false);
                for t in picked {
                    if t != owner && ctx.core.player(t).alive {
                        grant_skill_u05(ctx, t, "woliu2", false);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "woliu2" => {
            let shooter = m.target.unwrap_or(owner);
            let extra: Vec<PlayerId> = ctx
                .core
                .alive_players()
                .filter(|&p| p != owner && p != shooter && has_skill_u05(ctx.core, ctx.catalog, p, "woliu2"))
                .collect();
            if !extra.is_empty() {
                ctx.log(LogEvent::SkillTriggered {
                    skill: "woliu2",
                    owner,
                    presentation: None,
                });
                super::u05_ow::with_current_card_use_mut(ctx, |u| {
                    for p in extra {
                        if !u.targets.contains(&p) {
                            u.targets.push(p);
                        }
                    }
                });
            }
            Flow::Done
        }
        "qianggu" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::SelectCard, ASK_OW_QIANGGU_CARDS),
            HostStage::SelectCard => {
                let picks = answered_cards(ctx);
                if picks.len() != 2 {
                    refund_once_used(ctx, owner, "qianggu");
                    return Flow::Done;
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "qianggu",
                    owner,
                    presentation: None,
                });
                for c in picks {
                    discard_he_card(ctx, owner, c);
                }
                let cur = ctx.core.player(owner).hujia;
                ctx.core.player_mut(owner).hujia = cur.saturating_add(2);
                grant_skill_u05(ctx, owner, "qianggu2", false);
                set_flag(ctx.core, owner, "ow_qianggu2_caster", u32::from(owner) + 1);
                Flow::Done
            }
            _ => Flow::Done,
        },
        "qianggu2" => match m.stage {
            HostStage::Act => {
                if m.card.is_none() {
                    remove_skill_u05(ctx, owner, "qianggu2");
                    return Flow::Done;
                }
                m.targets = vec![m.targets.first().copied().unwrap_or(owner)];
                // m.card already carries sha_frame_id from park; save sha_frame_id in m.cards[0] before wait_ow overwrites m.card!
                let fid = m.card.unwrap_or(0);
                m.cards = vec![fid];
                wait_ow(m, ctx, HostStage::SelectCard, ASK_OW_QIANGGU2_DISCARD)
            }
            HostStage::SelectCard => {
                let attacker = m.targets.first().copied().unwrap_or(owner);
                let fid = m.cards.first().copied().unwrap_or(0);
                let picked = answered_cards(ctx);
                if let Some(&cid) = picked.first() {
                    if ctx.core.player(attacker).hand.contains(&cid)
                        && ctx.catalog.card_kind(cid).is_basic_kind()
                    {
                        ctx.discard_from_hand(attacker, cid);
                        return Flow::Done;
                    }
                }
                ctx.core
                    .vm_aux
                    .flags
                    .insert((owner, format!("sha_cancelled_target:{fid}")), 1);
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dianji" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::SelectCard, ASK_OW_DIANJI_CARD),
            HostStage::SelectCard => {
                let Some(&cid) = answered_cards(ctx).first() else {
                    refund_once_used(ctx, owner, "dianji");
                    return Flow::Done;
                };
                if !ctx.core.player(owner).hand.contains(&cid) {
                    refund_once_used(ctx, owner, "dianji");
                    return Flow::Done;
                }
                let mut targets: Vec<PlayerId> = alive_others(ctx.core, owner)
                    .into_iter()
                    .filter(|t| {
                        ctx.core.computed_distance(ctx.catalog, owner, *t) <= 2
                            && ctx.catalog.ow_target_enabled(
                                ctx.core,
                                owner,
                                *t,
                                Some(cid),
                                CardKind::Jingleishan,
                                DamageNature::Thunder,
                            )
                    })
                    .collect();
                targets.reverse();
                if targets.is_empty() {
                    refund_once_used(ctx, owner, "dianji");
                    return Flow::Done;
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dianji",
                    owner,
                    presentation: None,
                });
                ctx.emit(EventData::CardUsed(crate::machine::card_use_with_nature(
                    cid,
                    owner,
                    targets,
                    DamageNature::Thunder,
                    Some(CardKind::Jingleishan),
                )));
                Flow::Done
            }
            _ => Flow::Done,
        },
        "feitiao" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::SelectCard, ASK_OW_FEITIAO_CARD),
            HostStage::SelectCard => {
                let Some(&cid) = answered_cards(ctx).first() else {
                    return Flow::Done;
                };
                m.cards = vec![cid];
                wait_ow(m, ctx, HostStage::TargetChoice, ASK_OW_FEITIAO_TARGET)
            }
            HostStage::TargetChoice => {
                let Some(&t) = answered_players(ctx).first() else {
                    return Flow::Done;
                };
                let Some(cid) = m.cards.first().copied() else {
                    return Flow::Done;
                };
                ctx.log(LogEvent::SkillTriggered {
                    skill: "feitiao",
                    owner,
                    presentation: None,
                });
                discard_he_card(ctx, owner, cid);
                grant_temp_skill(ctx, owner, "feitiao2");
                ctx.core.turn.feitiao_target = Some(t);
                if ctx.core.player(t).alive {
                    random_discard_he(ctx, t);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "zhencha" | "bshaowei" => match m.stage {
            HostStage::Act => {
                if m.card == Some(1) {
                    wait_ow(m, ctx, HostStage::SelectNature, ASK_OW_ZHENCHA_DR)
                } else {
                    wait_ow(m, ctx, HostStage::SelectNature, ASK_OW_ZHENCHA_MODE)
                }
            }
            HostStage::SelectNature => {
                if m.cards.is_empty() && m.targets.first().copied().is_some() && !m.targets.is_empty() {
                    // ASK_OW_ZHENCHA_DR (shaBegin)
                    if let Some(tag) = answered_tag(ctx) {
                        ctx.log(LogEvent::SkillTriggered {
                            skill: "zhencha",
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
                } else {
                    if answered_confirm(ctx) {
                        let next_mode = if ctx.core.player(owner).baolei_mode == 1 { 0 } else { 1 };
                        ctx.core.player_mut(owner).baolei_mode = next_mode;
                        grant_temp_skill(ctx, owner, "zhencha2");
                        ctx.log(LogEvent::SkillTriggered {
                            skill: if next_mode == 1 { "bshaowei" } else { "zhencha" },
                            owner,
                            presentation: None,
                        });
                    }
                    Flow::Done
                }
            }
            _ => Flow::Done,
        },
        "pingzhang" => match m.stage {
            HostStage::Act => {
                if !m.cards.is_empty() {
                    // DamageBefore (cards = [card_plus1, amount])
                    wait_ow(m, ctx, HostStage::SelectCard, ASK_OW_PINGZHANG_CARD)
                } else {
                    // TurnBegin count
                    let mut n = 0u32;
                    if has_skill_u05(ctx.core, ctx.catalog, owner, "pingzhang2") {
                        n += 1;
                        remove_skill_u05(ctx, owner, "pingzhang2");
                    }
                    if has_skill_u05(ctx.core, ctx.catalog, owner, "pingzhang3") {
                        n += 1;
                        remove_skill_u05(ctx, owner, "pingzhang3");
                    }
                    set_flag(ctx.core, owner, "ow_pingzhang_draw", n);
                    Flow::Done
                }
            }
            HostStage::SelectCard => {
                let Some(&cid) = answered_cards(ctx).first() else {
                    return Flow::Done;
                };
                ctx.log(LogEvent::SkillTriggered {
                    skill: "pingzhang",
                    owner,
                    presentation: None,
                });
                discard_he_card(ctx, owner, cid);
                super::u05_ow::with_current_damage_mut(ctx, |dm| {
                    dm.amount = (dm.amount - 1).max(0);
                });
                if m.target == Some(owner) {
                    grant_skill_u05(ctx, owner, "pingzhang2", false);
                } else {
                    grant_skill_u05(ctx, owner, "pingzhang3", false);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "liangou" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::SelectCard, ASK_OW_LIANGOU_CARD),
            HostStage::SelectCard => {
                let Some(&cid) = answered_cards(ctx).first() else {
                    refund_once_used(ctx, owner, "liangou");
                    return Flow::Done;
                };
                m.cards = vec![cid];
                wait_ow(m, ctx, HostStage::TargetChoice, ASK_OW_LIANGOU_TARGET)
            }
            HostStage::TargetChoice => {
                let Some(&t) = answered_players(ctx).first() else {
                    refund_once_used(ctx, owner, "liangou");
                    return Flow::Done;
                };
                let Some(cid) = m.cards.first().copied() else {
                    return Flow::Done;
                };
                ctx.log(LogEvent::SkillTriggered {
                    skill: "liangou",
                    owner,
                    presentation: None,
                });
                discard_he_card(ctx, owner, cid);
                m.target = Some(t);
                m.stage = HostStage::DrawSettle;
                ctx.emit(EventData::OrdinaryJudge {
                    owner,
                    invocation: u32::MAX,
                });
                Flow::Yield
            }
            HostStage::DrawSettle => {
                let Some(t) = m.target else { return Flow::Done };
                let ok = flag(ctx.core, owner, "last_judge_suit") != 2;
                if ok && ctx.core.player(t).alive {
                    grant_temp_skill(ctx, t, "liangou2");
                    set_flag(ctx.core, t, "ow_liangou2_from", u32::from(owner) + 1);
                    set_flag(ctx.core, t, "ow_liangou2_dmg", 1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "liangou2" => {
            if flag(ctx.core, owner, "ow_liangou2_dmg") > 0 {
                set_flag(ctx.core, owner, "ow_liangou2_dmg", 0);
                super::u05_ow::with_current_damage_mut(ctx, |dm| dm.amount += 1);
            }
            Flow::Done
        }
        "xiyang" => match m.stage {
            HostStage::Act => wait_ow(m, ctx, HostStage::SelectNature, ASK_OW_XIYANG_CONFIRM),
            HostStage::SelectNature => {
                if !answered_confirm(ctx) {
                    return Flow::Done;
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "xiyang",
                    owner,
                    presentation: None,
                });
                m.stage = HostStage::DrawSettle;
                turn_over_to_back(ctx, owner);
                Flow::Yield
            }
            HostStage::DrawSettle => {
                if ctx.core.player(owner).alive {
                    ctx.recover(owner, 2, None);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "qinru" => match m.stage {
            HostStage::Act => {
                if !m.targets.is_empty() {
                    wait_ow(m, ctx, HostStage::SelectNature, ASK_OW_QINRU_CONFIRM)
                } else {
                    // TurnPhaseAfter: decrement turns
                    let seats = ctx.core.players.len() as PlayerId;
                    for t in 0..seats {
                        let k = format!("ow_qinru_turn:{t}");
                        let v = flag(ctx.core, owner, &k);
                        if v > 1 {
                            set_flag(ctx.core, owner, &k, v - 1);
                        } else if v == 1 {
                            set_flag(ctx.core, owner, &k, 0);
                        }
                    }
                    Flow::Done
                }
            }
            HostStage::SelectNature => {
                if !answered_confirm(ctx) {
                    return Flow::Done;
                }
                let Some(t) = m.target else { return Flow::Done };
                ctx.log(LogEvent::SkillTriggered {
                    skill: "qinru",
                    owner,
                    presentation: None,
                });
                m.stage = HostStage::DrawSettle;
                ctx.emit(EventData::OrdinaryJudge {
                    owner: t,
                    invocation: u32::MAX,
                });
                Flow::Yield
            }
            HostStage::DrawSettle => {
                let Some(t) = m.target else { return Flow::Done };
                let ok = flag(ctx.core, t, "last_judge_suit") != 2;
                if ok && ctx.core.player(t).alive {
                    let turns = if ctx.core.turn.current == owner { 4 } else { 3 };
                    set_flag(ctx.core, owner, &format!("ow_qinru_turn:{t}"), turns);
                    set_flag(ctx.core, t, "ow_qinru_fengyin", 1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "yinshen" => {
            ctx.log(LogEvent::SkillTriggered {
                skill: "yinshen",
                owner,
                presentation: None,
            });
            ctx.core.player_mut(owner).yinshen_active = true;
            Flow::Done
        }
        "maichong" => {
            ctx.log(LogEvent::SkillTriggered {
                skill: "maichong",
                owner,
                presentation: None,
            });
            for t in alive_others(ctx.core, owner) {
                if flag(ctx.core, owner, &format!("ow_qinru_turn:{t}")) > 0 {
                    random_discard_he(ctx, t);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
