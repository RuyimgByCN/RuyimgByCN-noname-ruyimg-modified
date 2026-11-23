//! Batch U05-6b (`shahai-hearth` Part 2) Group A: 17 技能 (`xieneng` .. `hhudun`)。

use super::asks::{
    ASK_B6B_BINGYAN_CARD, ASK_B6B_DUZHANG_TARGET, ASK_B6B_FBEIFA_TARGET,
    ASK_B6B_OLDFBEIFA_TARGET, ASK_B6B_OLDSHIFA_DRAW_CONFIRM, ASK_B6B_SHIFA_DRAW_CONFIRM,
    ASK_B6B_XIENENG_PICK, ASK_B6B_YUANZHENG_CARD, ASK_B6B_YUFA_TARGET, ASK_B6B_YULU_DISCARD,
    ASK_B6B_YULU_TARGETS, bingyan_aoe_targets, bingyan_eligible_cards, clear_b6b_ask, wait_b6b,
};
use super::{
    b6b_flag, clear_b6b_flag, emit_free_sha_no_count, has_skill_b6b, hs_b6b_cnt, hs_b6b_inc,
    set_b6b_flag, sha_target_options,
};
use super::super::u05_hearth::{
    alive_others, discard_card_from_zone, discard_cards_from_zone, exact_base_name,
    gain_card_to_hand, grant_skill, is_delayed_trick_hearth, is_hearth_enemy, meta, remove_skill,
    rng_pick, spawn_card_by_base, take_he_card_as,
};
use super::super::{HostMachine, HostSkillBehavior, HostStage};
use crate::catalog::{CardKind, QueryCtx, SkillLimits, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn trigger_meta_a(name: &str) -> Option<TriggerMeta> {
    match name {
        "xieneng" | "duzhang" | "chuidiao" => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()))
        }
        "fbeifa" | "oldfbeifa" => {
            Some(meta(EventKind::DamageAfter, Timing::Normal, 0, false, Vec::new()))
        }
        "yufa" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new())),
        "yufa2" => Some(meta(EventKind::DamageEnd, Timing::Normal, 0, false, Vec::new())),
        "yufa3" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 10, false, Vec::new())),
        "shifa" | "oldshifa" => Some(meta(
            EventKind::PhaseBegin,
            Timing::Normal,
            0,
            false,
            vec![EventKind::CardUsed],
        )),
        "yuanzheng" => Some(meta(EventKind::CardUsed, Timing::Normal, 0, false, Vec::new())),
        "duzhang2" => Some(meta(EventKind::TurnPhaseAfter, Timing::Normal, 0, false, Vec::new())),
        "hannu" => Some(meta(EventKind::DamageEnd, Timing::Normal, 0, false, Vec::new())),
        "hhudun" => Some(meta(
            EventKind::PhaseBegin,
            Timing::Normal,
            0,
            false,
            vec![EventKind::DamageEnd],
        )),
        _ => None,
    }
}

pub(super) fn limits_a(_name: &str) -> SkillLimits {
    SkillLimits::default()
}

pub(super) fn has_active_a(name: &str) -> bool {
    matches!(name, "bingyan" | "yulu" | "oldyulu")
}

fn has_live_fbeifa(state: &crate::state::GameStateCore, owner: PlayerId, sk: &str) -> bool {
    b6b_flag(state, owner, &format!("hs_{sk}_live")) > 0
}

fn find_fbeifa_damage_frame(ctx: &Ctx<'_>, owner: PlayerId, sk: &str) -> bool {
    let pref = format!("hs_{sk}_frame:");
    ctx.res
        .stack
        .iter()
        .any(|f| b6b_flag(ctx.core, owner, &format!("{pref}{}", f.id)) > 0)
}

pub(super) fn filter_a(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let st = ctx.state;
    match skill.name.as_str() {
        "xieneng" | "chuidiao" => matches!(
            event,
            EventData::Phase { player, phase: Phase::End, begin: true } if *player == owner
        ),
        "fbeifa" => matches!(
            event,
            EventData::Damage(d) if d.source == Some(owner) && has_live_fbeifa(st, owner, "fbeifa")
        ),
        "oldfbeifa" => matches!(
            event,
            EventData::Damage(d) if d.source == Some(owner) && has_live_fbeifa(st, owner, "oldfbeifa")
        ),
        "yufa" => match event {
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } => {
                let rec = b6b_flag(st, owner, "hs_yufa_attacker");
                rec == u32::from(*player) + 1 && !alive_others(st, *player).is_empty()
            }
            _ => false,
        },
        "yufa2" => matches!(
            event,
            EventData::Damage(d)
                if d.target == owner
                    && d.source.is_some_and(|s| s != owner && s == st.turn.current)
        ),
        "yufa3" => matches!(
            event,
            EventData::Phase { phase: Phase::Prepare, begin: true, .. }
                if b6b_flag(st, owner, "hs_yufa_attacker") > 0
        ),
        "shifa" => match event {
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } => *player == owner,
            EventData::CardUsed(u) => {
                let eff = u.effective_kind(ctx.catalog);
                u.player == owner
                    && st.turn.current == owner
                    && !u.cancelled
                    && u.card.is_some()
                    && !eff.is_basic_kind()
                    && eff.equip_slot().is_none()
                    && !is_delayed_trick_hearth(eff)
                    && hs_b6b_cnt(st, owner, "shifa_draw") < 3
            }
            _ => false,
        },
        "oldshifa" => match event {
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } => *player == owner,
            EventData::CardUsed(u) => {
                let eff = u.effective_kind(ctx.catalog);
                u.player == owner
                    && st.turn.current == owner
                    && !u.cancelled
                    && u.card.is_some()
                    && !eff.is_basic_kind()
                    && eff.equip_slot().is_none()
                    && !is_delayed_trick_hearth(eff)
            }
            _ => false,
        },
        "yuanzheng" => match event {
            EventData::CardUsed(u) if u.player == owner && !u.cancelled => u.targets.iter().any(|&t| {
                t != owner
                    && st.player(t).alive
                    && st.computed_distance(ctx.catalog, owner, t) > 1
                    && (!st.player(t).hand.is_empty() || !st.player(t).equips.is_empty())
            }),
            _ => false,
        },
        "duzhang" => matches!(
            event,
            EventData::Phase { player, phase: Phase::End, begin: true }
                if *player == owner
                    && b6b_flag(st, owner, "hs_duzhang_used_trick") == 0
                    && !alive_others(st, owner).is_empty()
        ),
        "duzhang2" => matches!(
            event,
            EventData::TurnEndWindow { player, after: true } if *player == owner
        ),
        "hannu" => matches!(
            event,
            EventData::Damage(d) if d.target == owner && !st.player(owner).hand.is_empty()
        ),
        "hhudun" => match event {
            EventData::Phase {
                phase: Phase::Prepare,
                begin: true,
                ..
            } => st.player(owner).hujia == 0,
            EventData::Damage(d) => {
                d.target == owner && b6b_flag(st, owner, "hs_hhudun_hujia_full") > 0
            }
            _ => false,
        },
        _ => false,
    }
}

pub(super) fn activate_a(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<bool> {
    let st = ctx.state;
    match skill.name.as_str() {
        "bingyan" => Some(
            hs_b6b_cnt(st, owner, "bingyan") == 0
                && !bingyan_eligible_cards(st, ctx.catalog, owner).is_empty(),
        ),
        "yulu" => Some(hs_b6b_cnt(st, owner, "yulu") == 0),
        "oldyulu" => Some(hs_b6b_cnt(st, owner, "oldyulu") == 0),
        _ => None,
    }
}

fn step_fbeifa(m: &mut HostMachine, ctx: &mut Ctx<'_>, old: bool) -> Flow {
    let sk = if old { "oldfbeifa" } else { "fbeifa" };
    let ask_id = if old {
        ASK_B6B_OLDFBEIFA_TARGET
    } else {
        ASK_B6B_FBEIFA_TARGET
    };
    if m.stage == HostStage::Act && m.target.is_some() {
        // Triggered on DamageAfter (`fbeifa_draw`): `machine_u05_b6b` always sets `m.target = Some(d.target)`.
        if find_fbeifa_damage_frame(ctx, m.owner, sk) && ctx.core.player(m.owner).alive {
            ctx.start_draw(m.owner, 1);
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => {
            clear_b6b_flag(ctx.core, m.owner, "hs_fbeifa_parked");
            if !has_skill_b6b(ctx.core, ctx.catalog, m.owner, sk)
                || !ctx.core.player(m.owner).hand.is_empty()
                || hs_b6b_cnt(ctx.core, m.owner, sk) >= 3
                || sha_target_options(ctx.core, ctx.catalog, m.owner).is_empty()
            {
                return Flow::Done;
            }
            wait_b6b(m, ctx, HostStage::TargetChoice, ask_id)
        }
        HostStage::TargetChoice => {
            clear_b6b_ask(m, ctx, HostStage::TargetChoice);
            if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                if ctx.core.player(t).alive && t != m.owner {
                    hs_b6b_inc(ctx.core, m.owner, sk);
                    let fid = ctx.res.next_event_id;
                    set_b6b_flag(ctx.core, m.owner, &format!("hs_{sk}_frame:{fid}"), 1);
                    set_b6b_flag(ctx.core, m.owner, &format!("hs_{sk}_live"), 1);
                    emit_free_sha_no_count(ctx, m.owner, t);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn spawn_random_ordinary_trick(ctx: &mut Ctx<'_>, recipient: PlayerId) {
    let cat = ctx.catalog;
    let list: Vec<CardId> = ctx
        .core
        .detached_cards
        .iter()
        .filter_map(|&(cid, _)| {
            let k = cat.card_kind(cid);
            (!k.is_basic_kind()
                && k.equip_slot().is_none()
                && !is_delayed_trick_hearth(k)
                && exact_base_name(cat.cards[cid as usize].name) != "du")
            .then_some(cid)
        })
        .collect();
    if let Some(pick) = rng_pick(ctx, &list) {
        ctx.core.detached_cards.retain(|&(x, _)| x != pick);
        gain_card_to_hand(ctx, recipient, pick);
    }
}

fn step_shifa(m: &mut HostMachine, ctx: &mut Ctx<'_>, old: bool) -> Flow {
    let cur = ctx.res.stack.last().map(|f| f.data.clone());
    if matches!(
        cur,
        Some(EventData::Phase {
            phase: Phase::Play,
            begin: true,
            ..
        })
    ) {
        spawn_random_ordinary_trick(ctx, m.owner);
        let enemies: Vec<PlayerId> = alive_others(ctx.core, m.owner)
            .into_iter()
            .filter(|&p| is_hearth_enemy(ctx.core, ctx.catalog, m.owner, p))
            .collect();
        if let Some(e) = rng_pick(ctx, &enemies) {
            spawn_random_ordinary_trick(ctx, e);
        }
        return Flow::Done;
    }
    let ask_id = if old {
        ASK_B6B_OLDSHIFA_DRAW_CONFIRM
    } else {
        ASK_B6B_SHIFA_DRAW_CONFIRM
    };
    let cnt_key = if old { "oldshifa_draw" } else { "shifa_draw" };
    match m.stage {
        HostStage::Act => {
            if !old && hs_b6b_cnt(ctx.core, m.owner, cnt_key) >= 3 {
                return Flow::Done;
            }
            wait_b6b(m, ctx, HostStage::CardsAsk, ask_id)
        }
        HostStage::CardsAsk => {
            clear_b6b_ask(m, ctx, HostStage::CardsAsk);
            if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                hs_b6b_inc(ctx.core, m.owner, cnt_key);
                ctx.start_draw(m.owner, 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_yulu(m: &mut HostMachine, ctx: &mut Ctx<'_>, old: bool) -> Flow {
    let sk = if old { "oldyulu" } else { "yulu" };
    let n: u32 = if old { 2 } else { 1 };
    match m.stage {
        HostStage::Act => wait_b6b(m, ctx, HostStage::TargetChoice, ASK_B6B_YULU_TARGETS),
        HostStage::TargetChoice => {
            clear_b6b_ask(m, ctx, HostStage::TargetChoice);
            let ts: Vec<PlayerId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Player(p) if ctx.core.player(*p).alive => Some(*p),
                    _ => None,
                })
                .collect();
            if ts.is_empty() {
                return Flow::Done;
            }
            hs_b6b_inc(ctx.core, m.owner, sk);
            m.targets = ts.clone();
            m.card = Some(n);
            for t in ts.iter().copied().rev() {
                ctx.start_draw(t, n);
            }
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            while !m.targets.is_empty() {
                let next_t = m.targets[0];
                let pl = ctx.core.player(next_t);
                let has_hej =
                    pl.alive && (!pl.hand.is_empty() || !pl.equips.is_empty() || !pl.judge.is_empty());
                if !has_hej {
                    m.targets.remove(0);
                    continue;
                }
                m.target = Some(next_t);
                return wait_b6b(m, ctx, HostStage::SelectCard, ASK_B6B_YULU_DISCARD);
            }
            Flow::Done
        }
        HostStage::SelectCard => {
            clear_b6b_ask(m, ctx, HostStage::SelectCard);
            if let Some(t) = m.target {
                let chosen: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                if !chosen.is_empty() {
                    discard_cards_from_zone(ctx, t, &chosen);
                    ctx.log(LogEvent::CardDiscarded {
                        player: t,
                        cards: chosen,
                    });
                }
            }
            if !m.targets.is_empty() {
                m.targets.remove(0);
            }
            m.stage = HostStage::DrawSettle;
            step_yulu(m, ctx, old)
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_a(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "xieneng" => match m.stage {
            HostStage::Act => wait_b6b(m, ctx, HostStage::SelectNature, ASK_B6B_XIENENG_PICK),
            HostStage::SelectNature => {
                clear_b6b_ask(m, ctx, HostStage::SelectNature);
                let name = match ctx.answer().first().copied() {
                    Some(ChoiceOption::Tag(0)) => Some("hsshenqi_morijingxiang"),
                    Some(ChoiceOption::Tag(1)) => Some("hsshenqi_kongbusangzhong"),
                    Some(ChoiceOption::Tag(2)) => Some("hsshenqi_nengliangzhiguang"),
                    _ => None,
                };
                if let Some(base) = name {
                    if let Some(c) = spawn_card_by_base(ctx, base) {
                        gain_card_to_hand(ctx, m.owner, c);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "fbeifa" => step_fbeifa(m, ctx, false),
        "oldfbeifa" => step_fbeifa(m, ctx, true),
        "yufa" => match m.stage {
            HostStage::Act => {
                let rec = b6b_flag(ctx.core, m.owner, "hs_yufa_attacker");
                if rec == 0 {
                    return Flow::Done;
                }
                m.target = Some((rec - 1) as PlayerId);
                wait_b6b(m, ctx, HostStage::TargetChoice, ASK_B6B_YUFA_TARGET)
            }
            HostStage::TargetChoice => {
                clear_b6b_ask(m, ctx, HostStage::TargetChoice);
                if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                    if ctx.core.player(t).alive {
                        if let Some(c) = spawn_card_by_base(ctx, "chuansongmen") {
                            gain_card_to_hand(ctx, t, c);
                        }
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "yufa2" => {
            if let Ok(d) = ctx.current_damage() {
                if let Some(src) = d.source {
                    if src != m.owner && ctx.core.turn.current == src {
                        set_b6b_flag(ctx.core, m.owner, "hs_yufa_attacker", u32::from(src) + 1);
                    }
                }
            }
            Flow::Done
        }
        "yufa3" => {
            clear_b6b_flag(ctx.core, m.owner, "hs_yufa_attacker");
            Flow::Done
        }
        "bingyan" => match m.stage {
            HostStage::Act => wait_b6b(m, ctx, HostStage::SelectCard, ASK_B6B_BINGYAN_CARD),
            HostStage::SelectCard => {
                clear_b6b_ask(m, ctx, HostStage::SelectCard);
                if let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() {
                    let kind = if super::is_b6b_red_suit(
                        ctx.catalog.effective_card_suit(ctx.core, m.owner, cid),
                    ) {
                        CardKind::Chiyuxi
                    } else {
                        CardKind::Jingleishan
                    };
                    let ts = bingyan_aoe_targets(ctx.core, ctx.catalog, m.owner, cid, kind);
                    if !ts.is_empty() {
                        hs_b6b_inc(ctx.core, m.owner, "bingyan");
                        take_he_card_as(ctx, m.owner, cid, ts, kind);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "shifa" => step_shifa(m, ctx, false),
        "oldshifa" => step_shifa(m, ctx, true),
        "yuanzheng" => match m.stage {
            HostStage::Act => {
                let qual: Vec<PlayerId> = m
                    .targets
                    .iter()
                    .copied()
                    .filter(|&t| {
                        t != m.owner
                            && ctx.core.player(t).alive
                            && ctx.core.computed_distance(ctx.catalog, m.owner, t) > 1
                            && (!ctx.core.player(t).hand.is_empty()
                                || !ctx.core.player(t).equips.is_empty())
                    })
                    .collect();
                if qual.is_empty() {
                    return Flow::Done;
                }
                m.targets = qual;
                let first = m.targets.remove(0);
                m.target = Some(first);
                wait_b6b(m, ctx, HostStage::SelectCard, ASK_B6B_YUANZHENG_CARD)
            }
            HostStage::SelectCard => {
                clear_b6b_ask(m, ctx, HostStage::SelectCard);
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    if let Some(t) = m.target {
                        let pl = ctx.core.player(t);
                        let valid = pl.alive
                            && (pl.hand.contains(&c)
                                || pl.equips.contains(&c)
                                || pl.judge.contains(&c));
                        if valid {
                            discard_card_from_zone(ctx, t, c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: t,
                                cards: vec![c],
                            });
                        }
                    }
                }
                while let Some(next_t) = (!m.targets.is_empty()).then(|| m.targets.remove(0)) {
                    let pl = ctx.core.player(next_t);
                    if pl.alive && (!pl.hand.is_empty() || !pl.equips.is_empty()) {
                        m.target = Some(next_t);
                        return wait_b6b(m, ctx, HostStage::SelectCard, ASK_B6B_YUANZHENG_CARD);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "yulu" => step_yulu(m, ctx, false),
        "oldyulu" => step_yulu(m, ctx, true),
        "duzhang" => match m.stage {
            HostStage::Act => wait_b6b(m, ctx, HostStage::TargetChoice, ASK_B6B_DUZHANG_TARGET),
            HostStage::TargetChoice => {
                clear_b6b_ask(m, ctx, HostStage::TargetChoice);
                if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                    if ctx.core.player(t).alive && t != m.owner {
                        grant_skill(ctx, t, "duzhang2", false);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "duzhang2" => {
            remove_skill(ctx, m.owner, "duzhang2");
            Flow::Done
        }
        "hannu" => match m.stage {
            HostStage::Act => {
                let nh = ctx.core.player(m.owner).hand.len() as u32;
                if nh == 0 {
                    return Flow::Done;
                }
                ctx.start_draw(m.owner, nh);
                m.stage = HostStage::DrawSettle;
                Flow::Yield
            }
            HostStage::DrawSettle => {
                let pl = ctx.core.player(m.owner);
                let nh = pl.hand.len();
                let hp = pl.hp.max(0) as usize;
                if nh > 10 && nh > hp {
                    let drop = nh - hp;
                    let mut pool = ctx.core.player(m.owner).hand.clone();
                    let mut discarded = Vec::with_capacity(drop);
                    for _ in 0..drop {
                        if let Some(c) = rng_pick(ctx, &pool) {
                            pool.retain(|&x| x != c);
                            discarded.push(c);
                        }
                    }
                    if !discarded.is_empty() {
                        discard_cards_from_zone(ctx, m.owner, &discarded);
                        ctx.log(LogEvent::CardDiscarded {
                            player: m.owner,
                            cards: discarded,
                        });
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "chuidiao" => {
            let num = ctx.core.rng.gen_range(3);
            if num > 0 {
                ctx.start_draw(m.owner, num);
            }
            Flow::Done
        }
        "hhudun" => {
            if ctx.current_damage().is_ok() {
                if b6b_flag(ctx.core, m.owner, "hs_hhudun_hujia_full") > 0 {
                    clear_b6b_flag(ctx.core, m.owner, "hs_hhudun_hujia_full");
                    ctx.start_draw(m.owner, 1);
                }
            } else if ctx.core.player(m.owner).hujia == 0 {
                ctx.change_hujia(m.owner, 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
