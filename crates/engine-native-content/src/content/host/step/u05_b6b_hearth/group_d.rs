//! Batch U05-6b (`shahai-hearth` Part 2) Group D: 20 技能 (`jingmeng` .. `anying`)。

use super::asks::{
    ASK_B6B_ANYING_CARD, ASK_B6B_BINGSHUANG_CONFIRM, ASK_B6B_DUNJI_TARGETS, ASK_B6B_FENGXIAN_DISCARD,
    ASK_B6B_FENGXING_DISCARD, ASK_B6B_HSWUJI_CONFIRM, ASK_B6B_JINGMENG_CONFIRM,
    ASK_B6B_OLDYANSHU_CONFIRM, ASK_B6B_SHENGGUANG_CARD, ASK_B6B_SHENGGUANG_TARGET,
    ASK_B6B_SHENGYAN_CONFIRM, ASK_B6B_SHIJIE_TARGET, ASK_B6B_XINCI_CARD, ASK_B6B_XINCI_TARGET,
    ASK_B6B_YANSHU_CONFIRM, ASK_B6B_ZHANHOU_CARD, clear_b6b_ask, wait_b6b,
};
use super::{
    b6b_flag, clear_b6b_flag, hs_b6b_cnt, hs_b6b_inc, is_b6b_red_suit, set_b6b_flag,
};
use super::super::u05_hearth::{
    alive_others, discard_card_from_zone, flag, gain_card_to_hand, grant_skill, has_any_skill,
    in_attack_range_hearth, is_delayed_trick_hearth, make_damage, meta, remove_skill, rng_pick,
    set_flag, spawn_card_by_base,
};
use super::super::{HostMachine, HostSkillBehavior, HostStage};
use crate::catalog::{CardKind, EquipSlot, QueryCtx, SkillLimits, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData, EventKind, LostOrigin, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn trigger_meta_d(name: &str) -> Option<TriggerMeta> {
    match name {
        "jingmeng" => Some(meta(EventKind::CardUsed, Timing::Normal, 0, false, Vec::new())),
        "hswuji" => Some(meta(EventKind::PhaseEnd, Timing::First, 0, false, Vec::new())),
        "bingshuang" | "zhongjia" => {
            Some(meta(EventKind::DamageEnd, Timing::Normal, 0, false, Vec::new()))
        }
        "yanshu" | "oldyanshu" => {
            Some(meta(EventKind::CardsDiscarded, Timing::Normal, 0, false, Vec::new()))
        }
        "shengyan" => Some(meta(EventKind::Recover, Timing::Normal, 0, false, Vec::new())),
        "qingliu" => Some(meta(EventKind::DamageBefore, Timing::Normal, 0, false, Vec::new())),
        "fengxing" => Some(meta(EventKind::CardsLost, Timing::Normal, 0, false, Vec::new())),
        "shijie" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new())),
        _ => None,
    }
}

pub(super) fn limits_d(name: &str) -> SkillLimits {
    SkillLimits {
        once_per_turn: false,
        active_outside_play: false,
    }
    .tap_none(name)
}

trait LimitsExt {
    fn tap_none(self, _name: &str) -> Self;
}
impl LimitsExt for SkillLimits {
    fn tap_none(self, _name: &str) -> Self {
        self
    }
}

pub(super) fn has_active_d(name: &str) -> bool {
    matches!(
        name,
        "liechao" | "dunji" | "fengxian" | "zhanhou" | "oldzhanhou" | "shengguang" | "xinci" | "anying"
    )
}

fn kind_category(catalog: &crate::catalog::Catalog, kind: CardKind) -> u8 {
    let _ = catalog;
    if kind.is_basic_kind() {
        0
    } else if kind.equip_slot().is_some() {
        1
    } else if is_delayed_trick_hearth(kind) {
        2
    } else {
        3
    }
}

pub(super) fn filter_d(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let st = ctx.state;
    match skill.name.as_str() {
        "jingmeng" => matches!(
            event,
            EventData::CardUsed(u)
                if u.player == owner
                    && !u.cancelled
                    && st.turn.current == owner
                    && b6b_flag(st, owner, "hs_b6b_turn_card_used") == 0
                    && b6b_flag(st, owner, "hs_jingmeng_trig") == 0
        ),
        "hswuji" => matches!(
            event,
            EventData::Phase { player, phase: Phase::Play, begin: false }
                if *player == owner && b6b_flag(st, owner, "hs_b6b_turn_card_used") > 0
        ),
        "bingshuang" => match event {
            EventData::Damage(d) => {
                d.source == Some(owner)
                    && st.player(d.target).alive
                    && !st.player(d.target).face_down
                    && d.card.is_some_and(|c| {
                        let k = ctx.catalog.card_kind(c);
                        !k.is_basic_kind() && k.equip_slot().is_none() && !is_delayed_trick_hearth(k)
                    })
            }
            _ => false,
        },
        "yanshu" => match event {
            EventData::CardsDiscarded { player, cards } => {
                *player == owner
                    && hs_b6b_cnt(st, owner, "yanshu") == 0
                    && cards.iter().any(|&c| !ctx.catalog.card_kind(c).is_basic_kind())
            }
            _ => false,
        },
        "oldyanshu" => match event {
            EventData::CardsDiscarded { player, cards } => {
                *player == owner
                    && cards.iter().any(|&c| !ctx.catalog.card_kind(c).is_basic_kind())
            }
            _ => false,
        },
        "shengyan" => matches!(
            event,
            EventData::Recover { player, amount, .. }
                if *amount > 0
                    && !has_any_skill(st, ctx.catalog, owner, "shengyan2")
                    && st.player(*player).alive
                    && st.player(*player).hp < st.player(*player).max_hp as i32
        ),
        "qingliu" => matches!(
            event,
            EventData::Damage(d) if d.target == owner && d.nature == DamageNature::Fire
        ),
        "zhongjia" => matches!(
            event,
            EventData::Damage(d) if d.target == owner && d.amount > 0
        ),
        "fengxing" => matches!(
            event,
            EventData::CardsLost { player, .. }
                if *player == owner
                    && st.turn.current != owner
                    && !has_any_skill(st, ctx.catalog, owner, "fengxing2")
                    && b6b_flag(st, owner, "hs_fengxing_parked") == 0
                    && (!st.player(owner).hand.is_empty() || !st.player(owner).equips.is_empty())
        ),
        "shijie" => matches!(
            event,
            EventData::Phase { player, phase: Phase::End, begin: true }
                if *player == owner
                    && alive_others(st, owner)
                        .into_iter()
                        .any(|p| !st.player(p).hand.is_empty())
        ),
        _ => false,
    }
}

pub(super) fn activate_d(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<bool> {
    let st = ctx.state;
    let pl = st.player(owner);
    let has_armor = || {
        pl.hand
            .iter()
            .chain(pl.equips.iter())
            .any(|&c| ctx.catalog.card_kind(c).equip_slot() == Some(EquipSlot::Armor))
    };
    let has_red = || {
        pl.hand
            .iter()
            .chain(pl.equips.iter())
            .any(|&c| is_b6b_red_suit(ctx.catalog.effective_card_suit(st, owner, c)))
    };
    let has_black = || {
        pl.hand
            .iter()
            .chain(pl.equips.iter())
            .any(|&c| !is_b6b_red_suit(ctx.catalog.effective_card_suit(st, owner, c)))
    };
    match skill.name.as_str() {
        "liechao" => Some(
            hs_b6b_cnt(st, owner, "liechao") == 0
                && !pl.face_down
                && (pl.hand.len() as i32) <= pl.hp,
        ),
        "dunji" => Some(
            hs_b6b_cnt(st, owner, "dunji") == 0
                && pl.hujia > 0
                && alive_others(st, owner)
                    .into_iter()
                    .any(|p| in_attack_range_hearth(st, ctx.catalog, owner, p)),
        ),
        "fengxian" => Some(
            hs_b6b_cnt(st, owner, "fengxian") == 0
                && st.alive_players().any(|p| !st.player(p).hand.is_empty()),
        ),
        "zhanhou" => Some(hs_b6b_cnt(st, owner, "zhanhou") == 0 && has_armor()),
        "oldzhanhou" => Some(has_armor()),
        "shengguang" => {
            let aw = flag(st, owner, "hs_anying_awakened") > 0;
            let tgt_ok = st
                .alive_players()
                .any(|p| aw || st.player(p).hp < st.player(p).max_hp as i32);
            Some(hs_b6b_cnt(st, owner, "shengguang") == 0 && has_red() && tgt_ok)
        }
        "xinci" => Some(hs_b6b_cnt(st, owner, "xinci") == 0 && has_black()),
        "anying" => Some(flag(st, owner, "hs_anying_awakened") == 0 && has_black()),
        _ => None,
    }
}

pub(super) fn step_d(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "jingmeng" => match m.stage {
            HostStage::Act => {
                set_b6b_flag(ctx.core, m.owner, "hs_jingmeng_trig", 1);
                wait_b6b(m, ctx, HostStage::CardsAsk, ASK_B6B_JINGMENG_CONFIRM)
            }
            HostStage::CardsAsk => {
                clear_b6b_ask(m, ctx, HostStage::CardsAsk);
                if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                    if let Some(src_c) = m.card {
                        let want_cat = kind_category(ctx.catalog, ctx.catalog.card_kind(src_c));
                        let cat = ctx.catalog;
                        let matches: Vec<CardId> = ctx
                            .core
                            .deck
                            .iter()
                            .copied()
                            .filter(|&c| kind_category(cat, cat.card_kind(c)) == want_cat)
                            .collect();
                        if let Some(picked) = rng_pick(ctx, &matches) {
                            ctx.core.deck.retain(|&x| x != picked);
                            gain_card_to_hand(ctx, m.owner, picked);
                        }
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "hswuji" => match m.stage {
            HostStage::Act => wait_b6b(m, ctx, HostStage::CardsAsk, ASK_B6B_HSWUJI_CONFIRM),
            HostStage::CardsAsk => {
                clear_b6b_ask(m, ctx, HostStage::CardsAsk);
                if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                    let n = b6b_flag(ctx.core, m.owner, "hs_b6b_turn_card_used");
                    if n > 0 {
                        ctx.start_draw(m.owner, n);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "bingshuang" => match m.stage {
            HostStage::Act => wait_b6b(m, ctx, HostStage::CardsAsk, ASK_B6B_BINGSHUANG_CONFIRM),
            HostStage::CardsAsk => {
                clear_b6b_ask(m, ctx, HostStage::CardsAsk);
                if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                    if let Some(t) = m.target {
                        if ctx.core.player(t).alive {
                            ctx.start_draw(t, 2);
                            m.stage = HostStage::DrawSettle;
                            return Flow::Yield;
                        }
                    }
                }
                Flow::Done
            }
            HostStage::DrawSettle => {
                if let Some(t) = m.target {
                    if ctx.core.player(t).alive && !ctx.core.player(t).face_down {
                        ctx.turn_over(t);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "yanshu" | "oldyanshu" => {
            let limited = m.def_key == "yanshu";
            let ask_id = if limited {
                ASK_B6B_YANSHU_CONFIRM
            } else {
                ASK_B6B_OLDYANSHU_CONFIRM
            };
            match m.stage {
                HostStage::Act => wait_b6b(m, ctx, HostStage::CardsAsk, ask_id),
                HostStage::CardsAsk => {
                    clear_b6b_ask(m, ctx, HostStage::CardsAsk);
                    if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                        if limited {
                            hs_b6b_inc(ctx.core, m.owner, "yanshu");
                        }
                        if let Some(c) = spawn_card_by_base(ctx, "liuxinghuoyu") {
                            gain_card_to_hand(ctx, m.owner, c);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            }
        }
        "shengyan" => match m.stage {
            HostStage::Act => wait_b6b(m, ctx, HostStage::CardsAsk, ASK_B6B_SHENGYAN_CONFIRM),
            HostStage::CardsAsk => {
                clear_b6b_ask(m, ctx, HostStage::CardsAsk);
                if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                    if !has_any_skill(ctx.core, ctx.catalog, m.owner, "shengyan2") {
                        grant_skill(ctx, m.owner, "shengyan2", true);
                        if let Some(t) = m.target {
                            if ctx.core.player(t).alive
                                && ctx.core.player(t).hp < ctx.core.player(t).max_hp as i32
                            {
                                ctx.recover(t, 1, None);
                            }
                        }
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "liechao" => {
            hs_b6b_inc(ctx.core, m.owner, "liechao");
            ctx.start_draw(m.owner, 4);
            ctx.turn_over(m.owner);
            ctx.core.player_mut(m.owner).skip_discard_phase = true;
            Flow::Done
        }
        "qingliu" => {
            if ctx.current_damage().is_ok() {
                ctx.current_damage_mut().prevented = true;
            }
            Flow::Done
        }
        "zhongjia" => {
            ctx.change_hujia(m.owner, 1);
            Flow::Done
        }
        "dunji" => match m.stage {
            HostStage::Act => wait_b6b(m, ctx, HostStage::TargetChoice, ASK_B6B_DUNJI_TARGETS),
            HostStage::TargetChoice => {
                clear_b6b_ask(m, ctx, HostStage::TargetChoice);
                let ts: Vec<PlayerId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Player(p)
                            if *p != m.owner && ctx.core.player(*p).alive =>
                        {
                            Some(*p)
                        }
                        _ => None,
                    })
                    .collect();
                if ts.is_empty() {
                    return Flow::Done;
                }
                let max_t = ctx.core.player(m.owner).hujia as usize;
                let clamped: Vec<PlayerId> = ts.into_iter().take(max_t).collect();
                if clamped.is_empty() {
                    return Flow::Done;
                }
                hs_b6b_inc(ctx.core, m.owner, "dunji");
                ctx.change_hujia(m.owner, -(clamped.len() as i32));
                for t in clamped.into_iter().rev() {
                    ctx.emit(EventData::Damage(make_damage(
                        Some(m.owner),
                        t,
                        1,
                        DamageNature::Normal,
                        None,
                    )));
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "fengxing" => match m.stage {
            HostStage::Act => {
                clear_b6b_flag(ctx.core, m.owner, "hs_fengxing_parked");
                if ctx.core.turn.current == m.owner
                    || has_any_skill(ctx.core, ctx.catalog, m.owner, "fengxing2")
                {
                    return Flow::Done;
                }
                let pl = ctx.core.player(m.owner);
                if pl.hand.is_empty() && pl.equips.is_empty() {
                    return Flow::Done;
                }
                // `player.addTempSkill("fengxing2")` 须在弃牌选择前注入，以阻断弃牌引起的二次离手递归。
                grant_skill(ctx, m.owner, "fengxing2", true);
                wait_b6b(m, ctx, HostStage::SelectCard, ASK_B6B_FENGXING_DISCARD)
            }
            HostStage::SelectCard => {
                clear_b6b_ask(m, ctx, HostStage::SelectCard);
                if let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() {
                    discard_card_from_zone(ctx, m.owner, cid);
                    ctx.log(LogEvent::CardDiscarded {
                        player: m.owner,
                        cards: vec![cid],
                    });
                    ctx.start_draw(m.owner, 2);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "fengxian" => match m.stage {
            HostStage::Act => {
                let ts: Vec<PlayerId> = ctx
                    .core
                    .seat_order_from(m.owner)
                    .into_iter()
                    .filter(|&p| ctx.core.player(p).alive && !ctx.core.player(p).hand.is_empty())
                    .collect();
                if ts.is_empty() {
                    return Flow::Done;
                }
                hs_b6b_inc(ctx.core, m.owner, "fengxian");
                m.targets = ts;
                m.stage = HostStage::TargetChoice;
                step_d(m, ctx)
            }
            HostStage::TargetChoice => {
                while !m.targets.is_empty() {
                    let t = m.targets[0];
                    if !ctx.core.player(t).alive || ctx.core.player(t).hand.is_empty() {
                        m.targets.remove(0);
                        continue;
                    }
                    m.target = Some(t);
                    return wait_b6b(m, ctx, HostStage::SelectCard, ASK_B6B_FENGXIAN_DISCARD);
                }
                Flow::Done
            }
            HostStage::SelectCard => {
                clear_b6b_ask(m, ctx, HostStage::SelectCard);
                if let Some(t) = m.target {
                    if let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() {
                        if ctx.core.player(t).hand.contains(&cid) {
                            ctx.discard_from_hand(t, cid);
                        }
                    }
                }
                if !m.targets.is_empty() {
                    m.targets.remove(0);
                }
                m.stage = HostStage::TargetChoice;
                step_d(m, ctx)
            }
            _ => Flow::Done,
        },
        "zhanhou" | "oldzhanhou" => {
            let limited = m.def_key == "zhanhou";
            match m.stage {
                HostStage::Act => wait_b6b(m, ctx, HostStage::SelectCard, ASK_B6B_ZHANHOU_CARD),
                HostStage::SelectCard => {
                    clear_b6b_ask(m, ctx, HostStage::SelectCard);
                    if let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() {
                        if limited {
                            hs_b6b_inc(ctx.core, m.owner, "zhanhou");
                        }
                        discard_card_from_zone(ctx, m.owner, cid);
                        ctx.log(LogEvent::CardDiscarded {
                            player: m.owner,
                            cards: vec![cid],
                        });
                        ctx.change_hujia(m.owner, 1);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            }
        }
        "shijie" => match m.stage {
            HostStage::Act => wait_b6b(m, ctx, HostStage::TargetChoice, ASK_B6B_SHIJIE_TARGET),
            HostStage::TargetChoice => {
                clear_b6b_ask(m, ctx, HostStage::TargetChoice);
                if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                    if ctx.core.player(t).alive && t != m.owner {
                        let pool = ctx.core.player(t).hand.clone();
                        if let Some(c) = rng_pick(ctx, &pool) {
                            ctx.take_hand_card(t, c);
                            ctx.note_cards_lost(t, &[c]);
                            ctx.emit(EventData::CardsLost {
                                player: t,
                                cards: vec![c],
                                origins: vec![LostOrigin::Hand],
                            });
                            gain_card_to_hand(ctx, m.owner, c);
                            ctx.emit(EventData::CardsMoved {
                                cards: vec![c],
                                from: t,
                                to: m.owner,
                            });
                            ctx.start_draw(t, 1);
                        }
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "shengguang" => match m.stage {
            HostStage::Act => wait_b6b(m, ctx, HostStage::SelectCard, ASK_B6B_SHENGGUANG_CARD),
            HostStage::SelectCard => {
                clear_b6b_ask(m, ctx, HostStage::SelectCard);
                let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.card = Some(cid);
                wait_b6b(m, ctx, HostStage::TargetChoice, ASK_B6B_SHENGGUANG_TARGET)
            }
            HostStage::TargetChoice => {
                clear_b6b_ask(m, ctx, HostStage::TargetChoice);
                let Some(cid) = m.card else {
                    return Flow::Done;
                };
                if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                    if ctx.core.player(t).alive {
                        hs_b6b_inc(ctx.core, m.owner, "shengguang");
                        discard_card_from_zone(ctx, m.owner, cid);
                        ctx.log(LogEvent::CardDiscarded {
                            player: m.owner,
                            cards: vec![cid],
                        });
                        if flag(ctx.core, m.owner, "hs_anying_awakened") > 0 {
                            ctx.emit(EventData::LoseHp {
                                player: t,
                                amount: 1,
                            });
                        } else if ctx.core.player(t).hp < ctx.core.player(t).max_hp as i32 {
                            ctx.recover(t, 1, None);
                        }
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "xinci" => match m.stage {
            HostStage::Act => wait_b6b(m, ctx, HostStage::SelectCard, ASK_B6B_XINCI_CARD),
            HostStage::SelectCard => {
                clear_b6b_ask(m, ctx, HostStage::SelectCard);
                let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.card = Some(cid);
                wait_b6b(m, ctx, HostStage::TargetChoice, ASK_B6B_XINCI_TARGET)
            }
            HostStage::TargetChoice => {
                clear_b6b_ask(m, ctx, HostStage::TargetChoice);
                let Some(cid) = m.card else {
                    return Flow::Done;
                };
                if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                    if ctx.core.player(t).alive {
                        hs_b6b_inc(ctx.core, m.owner, "xinci");
                        discard_card_from_zone(ctx, m.owner, cid);
                        ctx.log(LogEvent::CardDiscarded {
                            player: m.owner,
                            cards: vec![cid],
                        });
                        ctx.emit(EventData::LoseHp {
                            player: t,
                            amount: 1,
                        });
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "anying" => match m.stage {
            HostStage::Act => wait_b6b(m, ctx, HostStage::SelectCard, ASK_B6B_ANYING_CARD),
            HostStage::SelectCard => {
                clear_b6b_ask(m, ctx, HostStage::SelectCard);
                if let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() {
                    set_flag(ctx.core, m.owner, "hs_anying_awakened", 1);
                    discard_card_from_zone(ctx, m.owner, cid);
                    ctx.log(LogEvent::CardDiscarded {
                        player: m.owner,
                        cards: vec![cid],
                    });
                    remove_skill(ctx, m.owner, "shengguang");
                    grant_skill(ctx, m.owner, "xinci", false);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        _ => Flow::Done,
    }
}
