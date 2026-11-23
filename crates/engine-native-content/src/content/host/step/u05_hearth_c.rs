//! Batch U05-6a (`shahai-hearth` Part 1): Batch C 20 技能 (`szbianshen` .. `jinzhou`)。

use super::u05_hearth::{
    alive_others, change_max_hp, discard_card_from_zone, emit_hearth_virtual, flag,
    gain_card_to_hand, grant_skill, has_any_skill, has_skill_hearth, is_delayed_trick_hearth,
    meta, remove_skill, rng_pick, set_flag, spawn_card_by_base, spawn_card_by_kind,
};
use super::u05_hearth_asks::{
    set_active_ask, wait_hs, ASK_HS_FUHUA2_GIVE, ASK_HS_FUHUA_CARD, ASK_HS_FUHUA_CHOICE,
    ASK_HS_FUHUA_TARGET, ASK_HS_JINHUA_SKILL, ASK_HS_JINZHOU_TARGET, ASK_HS_KEKAO_PICK,
    ASK_HS_KQIZHOU_PICK, ASK_HS_KUIXIN_TARGET, ASK_HS_MENGYE_TARGET, ASK_HS_QIANHOU_DISCARD,
    ASK_HS_SZBIANSHEN_CONFIRM, ASK_HS_SZBIANSHEN_GENERAL, ASK_HS_ZHONGJI_DISCARD,
};
use super::*;
use crate::catalog::{CardKind, QueryCtx, SkillLimits, Suit, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{EventData, EventKind, Timing};
use crate::ids::{CardId, PlayerId, SkillDefId};
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn trigger_meta_c(name: &str) -> Option<TriggerMeta> {
    Some(match name {
        "szbianshen" => meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()),
        "kekao" => meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()),
        "jinhua" => meta(EventKind::CardUsed, Timing::Normal, 0, false, Vec::new()),
        "kqizhou" => meta(
            EventKind::PhaseBegin,
            Timing::Normal,
            0,
            false,
            vec![EventKind::CardUsed],
        ),
        "shengzhang" | "fuwen" => meta(
            EventKind::PhaseEnd,
            Timing::Normal,
            0,
            false,
            vec![EventKind::CardsDiscarded],
        ),
        "pyuhuo" => meta(EventKind::Dying, Timing::Normal, 10, false, Vec::new()),
        "mengye" => meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()),
        "mengye2" => meta(
            EventKind::TurnPhaseAfter,
            Timing::Normal,
            0,
            false,
            vec![EventKind::Death],
        ),
        "lianzhan" => meta(EventKind::Damage, Timing::Normal, 0, false, Vec::new()),
        "kuixin" => meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()),
        "fuhua2" => meta(EventKind::PhaseBegin, Timing::Normal, 1, false, Vec::new()),
        "fuhua3" => meta(EventKind::DamageBefore, Timing::Normal, 0, false, Vec::new()),
        "moxie" => meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()),
        "qianhou" => meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()),
        "zhongji" => meta(EventKind::DamageBefore, Timing::Normal, 0, false, Vec::new()),
        "jinzhou" => meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()),
        _ => return None,
    })
}

pub(super) fn limits_c(name: &str) -> SkillLimits {
    match name {
        "fuhua" => SkillLimits::default(),
        _ => SkillLimits::default(),
    }
}

pub(super) fn has_active_c(name: &str) -> bool {
    matches!(name, "jingcu" | "fuhua")
}

pub(super) fn activate_c(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<bool> {
    let pl = ctx.state.player(owner);
    Some(match skill.name.as_str() {
        "jingcu" => pl.max_hp > 1,
        "fuhua" => {
            let has_du = pl
                .hand
                .iter()
                .any(|&c| super::u05_hearth::is_du_card_hearth(ctx.state, ctx.catalog, owner, c));
            let has_target = alive_others(ctx.state, owner).into_iter().any(|p| {
                !has_skill_hearth(ctx.state, ctx.catalog, p, "moxie")
                    && !has_skill_hearth(ctx.state, ctx.catalog, p, "fuhua2")
                    && flag(ctx.state, p, "hs_fuhua_immune") == 0
            });
            has_du && has_target
        }
        _ => return None,
    })
}

pub(super) fn filter_c(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    match skill.name.as_str() {
        "szbianshen" => matches!(
            event,
            EventData::Phase {
                player,
                phase: crate::event::Phase::Prepare,
                begin: true,
            } if *player == owner
                && ctx.state.turn.round >= 3
                && flag(ctx.state, owner, "hs_szbianshen_used") == 0
        ),
        "kekao" => matches!(
            event,
            EventData::Phase {
                player,
                phase: crate::event::Phase::End,
                begin: true,
            } if *player == owner
        ),
        "jinhua" => matches!(
            event,
            EventData::CardUsed(u) if u.player == owner
                && u.card.is_some()
                && u.use_as.is_none()
                && {
                    let k = u.effective_kind(ctx.catalog);
                    !k.is_basic_kind()
                        && k.equip_slot().is_none()
                        && !is_delayed_trick_hearth(k)
                        && (u.targets.contains(&owner)
                            || (u.targets.is_empty()
                                && matches!(k, CardKind::Wuzhong | CardKind::Taoyuan | CardKind::Wugu)))
                }
        ),
        "kqizhou" => match event {
            EventData::CardUsed(u) => {
                let k = u.effective_kind(ctx.catalog);
                u.player == owner
                    && ctx.state.turn.current == owner
                    && !k.is_basic_kind()
                    && k.equip_slot().is_none()
                    && !is_delayed_trick_hearth(k)
            }
            EventData::Phase {
                player,
                phase: crate::event::Phase::Prepare,
                begin: true,
            } => *player == owner && flag(ctx.state, owner, "hs_kqizhou_pending") > 0,
            _ => false,
        },
        "shengzhang" => match event {
            EventData::CardsDiscarded { player, cards } => {
                *player == owner
                    && ctx.state.turn.phase == crate::event::Phase::Discard
                    && !cards.is_empty()
            }
            EventData::Phase {
                player,
                phase: crate::event::Phase::Discard,
                begin: false,
            } => *player == owner && flag(ctx.state, owner, "hs_dp_disc_cnt") > 0,
            _ => false,
        },
        "fuwen" => match event {
            EventData::CardsDiscarded { player, cards } => {
                *player == owner
                    && ctx.state.turn.phase == crate::event::Phase::Discard
                    && cards.iter().any(|&c| {
                        let k = ctx.catalog.card_kind(c);
                        !k.is_basic_kind() && k.equip_slot().is_none()
                    })
            }
            EventData::Phase {
                player,
                phase: crate::event::Phase::Discard,
                begin: false,
            } => *player == owner && flag(ctx.state, owner, "hs_dp_trick") > 0,
            _ => false,
        },
        "pyuhuo" => matches!(
            event,
            EventData::Dying { player, .. }
                if *player == owner && flag(ctx.state, owner, "hs_pyuhuo_stage") < 2
        ),
        "mengye" => matches!(
            event,
            EventData::Phase {
                player,
                phase: crate::event::Phase::End,
                begin: true,
            } if *player == owner && !alive_with_hand(ctx.state, owner).is_empty()
        ),
        "mengye2" => matches!(
            event,
            EventData::TurnEndWindow { player, after: true } | EventData::Death { player, .. }
                if *player == owner
        ),
        "lianzhan" => matches!(
            event,
            EventData::Damage(d) if d.source == Some(owner) && !d.prevented && d.amount > 0
        ),
        "kuixin" => matches!(
            event,
            EventData::Phase {
                player,
                phase: crate::event::Phase::End,
                begin: true,
            } if *player == owner
                && alive_others(ctx.state, owner)
                    .into_iter()
                    .any(|p| ctx.state.player(p).hand.len() >= ctx.state.player(owner).hand.len().max(1))
        ),
        "fuhua2" => matches!(
            event,
            EventData::Phase {
                player,
                phase: crate::event::Phase::End,
                begin: true,
            } if *player == owner
                && !ctx.state.player(owner).hand.is_empty()
                && {
                    let s = flag(ctx.state, owner, "hs_fuhua2_source");
                    s > 0 && ctx.state.player((s - 1) as PlayerId).alive
                }
        ),
        "fuhua3" => matches!(
            event,
            EventData::Damage(d) if d.target == owner
                && d.source.is_some_and(|src| flag(ctx.state, owner, "hs_fuhua_duel_target") == u32::from(src) + 1)
        ),
        "moxie" => matches!(
            event,
            EventData::Phase {
                player,
                phase: crate::event::Phase::End,
                begin: true,
            } if *player == owner
                && ctx.state.player(owner).hand.iter().any(|&c| {
                    !super::u05_hearth::is_du_card_hearth(ctx.state, ctx.catalog, owner, c)
                })
        ),
        "qianhou" => matches!(
            event,
            EventData::Phase {
                player,
                phase: crate::event::Phase::Prepare,
                begin: true,
            } if *player == owner
        ),
        "zhongji" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && ctx.state.player(owner).hand.iter().any(|&c| {
                        matches!(ctx.catalog.effective_card_suit(ctx.state, owner, c), Suit::Spade | Suit::Club)
                    })
        ),
        "jinzhou" => matches!(
            event,
            EventData::Phase {
                player,
                phase: crate::event::Phase::End,
                begin: true,
            } if *player == owner
                && ctx.state.player(owner).hand.iter().any(|&c| {
                    ctx.catalog.effective_card_suit(ctx.state, owner, c) == Suit::Spade
                })
                && alive_others(ctx.state, owner)
                    .into_iter()
                    .any(|p| {
                        flag(ctx.state, p, "hs_jinzhou_sealed") == 0
                            && flag(ctx.state, p, "hs_fengyin_xuetu") == 0
                    })
        ),
        _ => false,
    }
}

fn alive_with_hand(core: &crate::state::GameStateCore, _owner: PlayerId) -> Vec<PlayerId> {
    core.alive_players()
        .filter(|&p| !core.player(p).hand.is_empty())
        .collect()
}

pub(super) fn step_c(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "szbianshen" => step_szbianshen(m, ctx),
        "kekao" => step_kekao(m, ctx),
        "jinhua" => step_jinhua(m, ctx),
        "kqizhou" => step_kqizhou(m, ctx),
        "jingcu" => {
            change_max_hp(ctx, m.owner, -1);
            ctx.start_draw(m.owner, 2);
            Flow::Done
        }
        "shengzhang" => {
            if matches!(
                ctx.res.stack.last().map(|f| &f.data),
                Some(EventData::CardsDiscarded { .. })
            ) {
                set_flag(ctx.core, m.owner, "hs_dp_disc_cnt", 1);
            } else {
                set_flag(ctx.core, m.owner, "hs_dp_disc_cnt", 0);
                change_max_hp(ctx, m.owner, 1);
            }
            Flow::Done
        }
        "fuwen" => {
            if matches!(
                ctx.res.stack.last().map(|f| &f.data),
                Some(EventData::CardsDiscarded { .. })
            ) {
                set_flag(ctx.core, m.owner, "hs_dp_trick", 1);
            } else {
                set_flag(ctx.core, m.owner, "hs_dp_trick", 0);
                ctx.change_hujia(m.owner, 1);
            }
            Flow::Done
        }
        "pyuhuo" => step_pyuhuo(m, ctx),
        "mengye" => step_mengye(m, ctx),
        "mengye2" => {
            remove_skill(ctx, m.owner, "mengye2");
            Flow::Done
        }
        "lianzhan" => step_lianzhan(m, ctx),
        "kuixin" => step_kuixin(m, ctx),
        "fuhua" => step_fuhua(m, ctx),
        "fuhua2" => step_fuhua2(m, ctx),
        "fuhua3" => {
            let src = ctx.res.stack.iter().rev().find_map(|f| match &f.data {
                EventData::Damage(d) if d.target == m.owner => d.source,
                _ => None,
            });
            if let Some(t) = src {
                set_flag(ctx.core, t, "hs_fuhua_immune", 1);
                set_flag(ctx.core, m.owner, "hs_fuhua_duel_target", 0);
                remove_skill(ctx, m.owner, "fuhua3");
            }
            Flow::Done
        }
        "moxie" => step_moxie(m, ctx),
        "qianhou" => step_qianhou(m, ctx),
        "zhongji" => step_zhongji(m, ctx),
        "jinzhou" => step_jinzhou(m, ctx),
        _ => Flow::Done,
    }
}

fn step_szbianshen(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::CompareTargetPick, ASK_HS_SZBIANSHEN_CONFIRM),
        HostStage::CompareTargetPick => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::CompareTargetPick, 0);
            if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                return Flow::Done;
            }
            let in_play: Vec<u16> = ctx
                .core
                .players
                .iter()
                .flat_map(|p| p.general_slots.iter().flatten().copied())
                .collect();
            let mut strong: Vec<u16> = (0..ctx.catalog.general_count() as u16)
                .filter(|&g| {
                    !in_play.contains(&g)
                        && ctx.catalog.general_meta(g).is_some_and(|m_| m_.3 >= 5)
                })
                .collect();
            if strong.is_empty() {
                strong = (0..ctx.catalog.general_count() as u16)
                    .filter(|g| !in_play.contains(g))
                    .collect();
            }
            if strong.is_empty() {
                return Flow::Done;
            }
            let mut offer = Vec::new();
            while !strong.is_empty() && offer.len() < 5 {
                let idx = ctx.core.rng.gen_range(strong.len() as u32) as usize;
                offer.push(strong.swap_remove(idx));
            }
            set_flag(ctx.core, m.owner, "hs_choice_count", offer.len() as u32);
            for (i, g) in offer.iter().enumerate() {
                set_flag(ctx.core, m.owner, &format!("hs_choice_g:{i}"), u32::from(*g) + 1);
            }
            set_flag(ctx.core, m.owner, "hs_szbianshen_used", 1);
            wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_SZBIANSHEN_GENERAL)
        }
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            let idx = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(t)) => t,
                _ => 0,
            };
            let raw = flag(ctx.core, m.owner, &format!("hs_choice_g:{idx}"));
            if raw > 0 {
                let g = (raw - 1) as u16;
                let removed: Vec<SkillDefId> = ctx
                    .core
                    .skills
                    .keys()
                    .filter(|(p, _)| *p == m.owner)
                    .map(|(_, d)| *d)
                    .collect();
                for d in removed {
                    ctx.core.skills.remove(&(m.owner, d));
                }
                ctx.core.player_mut(m.owner).general_slots[0] = Some(g);
                crate::state::apply_general_assembly(ctx.core, ctx.catalog, m.owner, 0);
                let max_hp = ctx.core.player(m.owner).max_hp as i32;
                ctx.core.player_mut(m.owner).hp = max_hp;
                ctx.log(LogEvent::SkillToggled {
                    player: m.owner,
                    skill: "szbianshen_reinit",
                    enabled: true,
                });
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_kekao(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_KEKAO_PICK),
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let kind = match tag {
                0 => CardKind::Le,
                1 => CardKind::Bing,
                _ => CardKind::ShanDian,
            };
            if let Some(c) = spawn_card_by_kind(ctx, kind) {
                gain_card_to_hand(ctx, m.owner, c);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_jinhua(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            let mut pool: Vec<SkillDefId> = (0..ctx.catalog.skills.len() as u16)
                .filter(|&d| {
                    let name = ctx.catalog.skill_name(d);
                    !name.is_empty()
                        && !name.starts_with('_')
                        && !name.ends_with("2")
                        && !name.ends_with("3")
                        && !has_any_skill(ctx.core, ctx.catalog, m.owner, name)
                })
                .collect();
            if pool.is_empty() {
                return Flow::Done;
            }
            let mut picked = Vec::new();
            while !pool.is_empty() && picked.len() < 3 {
                let idx = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                picked.push(pool.swap_remove(idx));
            }
            set_flag(ctx.core, m.owner, "hs_choice_count", picked.len() as u32);
            for (i, d) in picked.iter().enumerate() {
                set_flag(ctx.core, m.owner, &format!("hs_choice_sk:{i}"), u32::from(*d) + 1);
            }
            wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_JINHUA_SKILL)
        }
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            let idx = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(i)) => i,
                _ => 0,
            };
            let raw = flag(ctx.core, m.owner, &format!("hs_choice_sk:{idx}"));
            if raw > 0 {
                let name = ctx.catalog.skill_name((raw - 1) as u16).to_string();
                grant_skill(ctx, m.owner, &name, false);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_kqizhou(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if matches!(
        ctx.res.stack.last().map(|f| &f.data),
        Some(EventData::CardUsed(_))
    ) {
        set_flag(ctx.core, m.owner, "hs_kqizhou_pending", 1);
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => {
            set_flag(ctx.core, m.owner, "hs_kqizhou_pending", 0);
            wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_KQIZHOU_PICK)
        }
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let name = match tag {
                0 => "hsqizhou_feng",
                1 => "hsqizhou_shui",
                2 => "hsqizhou_huo",
                _ => "hsqizhou_tu",
            };
            if let Some(c) = spawn_card_by_base(ctx, name) {
                gain_card_to_hand(ctx, m.owner, c);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_pyuhuo(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let stage = flag(ctx.core, m.owner, "hs_pyuhuo_stage");
    if stage >= 2 {
        return Flow::Done;
    }
    let hand = ctx.core.player(m.owner).hand.clone();
    let eq = ctx.core.player(m.owner).equips.clone();
    let jg = ctx.core.player(m.owner).judge.clone();
    let mut dropped = Vec::new();
    for c in hand.into_iter().chain(eq).chain(jg) {
        discard_card_from_zone(ctx, m.owner, c);
        dropped.push(c);
    }
    if !dropped.is_empty() {
        ctx.log(LogEvent::CardDiscarded {
            player: m.owner,
            cards: dropped,
        });
    }
    {
        let p = ctx.core.player_mut(m.owner);
        p.linked = false;
        p.face_down = false;
        p.dying = false;
    }
    if stage == 0 {
        set_flag(ctx.core, m.owner, "hs_pyuhuo_stage", 1);
        let p = ctx.core.player_mut(m.owner);
        p.max_hp = 4;
        p.hp = 4;
        ctx.start_draw(m.owner, 4);
    } else {
        set_flag(ctx.core, m.owner, "hs_pyuhuo_stage", 2);
        let p = ctx.core.player_mut(m.owner);
        p.max_hp = 6;
        p.hp = 6;
        ctx.start_draw(m.owner, 6);
    }
    Flow::Done
}

fn step_mengye(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::TargetChoice, ASK_HS_MENGYE_TARGET),
        HostStage::TargetChoice => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::TargetChoice, 0);
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let non_du: Vec<CardId> = ctx
                .core
                .player(t)
                .hand
                .iter()
                .copied()
                .filter(|&c| !super::u05_hearth::is_du_card_hearth(ctx.core, ctx.catalog, t, c))
                .collect();
            if let Some(c) = rng_pick(ctx, &non_du) {
                set_flag(ctx.core, t, &format!("is_du_card:{c}"), 1);
            }
            ctx.change_hujia(t, 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_lianzhan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let cur_dmg = ctx.res.stack.iter().rev().find_map(|f| match &f.data {
        EventData::Damage(d) if d.source == Some(m.owner) => Some(d.amount),
        _ => None,
    });
    let num = cur_dmg.unwrap_or(1);
    let dealt = ctx.core.player(m.owner).turn_damage as i32;
    if dealt > num {
        change_max_hp(ctx, m.owner, 1);
        ctx.recover(m.owner, 1, None);
    } else {
        ctx.start_draw(m.owner, 2);
    }
    Flow::Done
}

fn step_kuixin(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::TargetChoice, ASK_HS_KUIXIN_TARGET),
        HostStage::TargetChoice => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::TargetChoice, 0);
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let hand = ctx.core.player(t).hand.clone();
            if let Some(c) = rng_pick(ctx, &hand) {
                ctx.give_hand_card(t, m.owner, c);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_fuhua(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_FUHUA_CARD),
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.cards = vec![picked];
            wait_hs(m, ctx, HostStage::TargetChoice, ASK_HS_FUHUA_TARGET)
        }
        HostStage::TargetChoice => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::TargetChoice, 0);
            let (Some(&card), Some(ChoiceOption::Player(t))) =
                (m.cards.first(), ctx.answer().first().copied())
            else {
                return Flow::Done;
            };
            m.target = Some(t);
            set_flag(ctx.core, m.owner, "g_du_gifting", 1);
            ctx.give_hand_card(m.owner, t, card);
            set_flag(ctx.core, m.owner, "g_du_gifting", 0);
            wait_hs(m, ctx, HostStage::CompareTargetPick, ASK_HS_FUHUA_CHOICE)
        }
        HostStage::CompareTargetPick => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::CompareTargetPick, 0);
            let Some(t) = m.target else {
                return Flow::Done;
            };
            let choice = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(tag)) => tag,
                _ => 0,
            };
            if choice == 0 {
                set_flag(ctx.core, t, "hs_fuhua2_source", u32::from(m.owner) + 1);
                grant_skill(ctx, t, "fuhua2", false);
                grant_skill(ctx, t, "moxie", false);
                return Flow::Done;
            }
            set_flag(ctx.core, m.owner, "hs_fuhua_duel_target", u32::from(t) + 1);
            grant_skill(ctx, m.owner, "fuhua3", false);
            m.stage = HostStage::DrawSettle;
            emit_hearth_virtual(ctx, m.owner, CardKind::Duel, vec![t]);
            Flow::Yield
        }
        HostStage::DrawSettle => {
            set_flag(ctx.core, m.owner, "hs_fuhua_duel_target", 0);
            remove_skill(ctx, m.owner, "fuhua3");
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_fuhua2(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_FUHUA2_GIVE),
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let src = flag(ctx.core, m.owner, "hs_fuhua2_source");
            if src > 0 {
                let to = (src - 1) as PlayerId;
                if ctx.core.player(to).alive {
                    set_flag(ctx.core, m.owner, "g_du_gifting", 1);
                    ctx.give_hand_card(m.owner, to, picked);
                    set_flag(ctx.core, m.owner, "g_du_gifting", 0);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_moxie(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let non_du: Vec<CardId> = ctx
        .core
        .player(m.owner)
        .hand
        .iter()
        .copied()
        .filter(|&c| !super::u05_hearth::is_du_card_hearth(ctx.core, ctx.catalog, m.owner, c))
        .collect();
    if let Some(c) = rng_pick(ctx, &non_du) {
        set_flag(ctx.core, m.owner, &format!("is_du_card:{c}"), 1);
    }
    Flow::Done
}

fn step_qianhou(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            let single_tricks = [
                CardKind::Duel,
                CardKind::Dismantle,
                CardKind::Snatch,
                CardKind::Wuzhong,
                CardKind::Linghunzhihuo,
                CardKind::Zhaomingdan,
                CardKind::Zhiliaobo,
            ];
            let mut pool: Vec<CardKind> = single_tricks.to_vec();
            while !pool.is_empty() {
                let idx = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                let kind = pool.swap_remove(idx);
                let target_opt = match kind {
                    CardKind::Wuzhong => Some(m.owner),
                    CardKind::Snatch => {
                        let cands: Vec<PlayerId> = alive_others(ctx.core, m.owner)
                            .into_iter()
                            .filter(|&p| {
                                ctx.core.computed_distance(ctx.catalog, m.owner, p) <= 1
                                    && (!ctx.core.player(p).hand.is_empty()
                                        || !ctx.core.player(p).equips.is_empty())
                            })
                            .collect();
                        rng_pick(ctx, &cands)
                    }
                    CardKind::Dismantle => {
                        let cands: Vec<PlayerId> = alive_others(ctx.core, m.owner)
                            .into_iter()
                            .filter(|&p| {
                                !ctx.core.player(p).hand.is_empty()
                                    || !ctx.core.player(p).equips.is_empty()
                                    || !ctx.core.player(p).judge.is_empty()
                            })
                            .collect();
                        rng_pick(ctx, &cands)
                    }
                    CardKind::Zhiliaobo => {
                        let cands: Vec<PlayerId> = ctx
                            .core
                            .alive_players()
                            .filter(|&p| ctx.core.player(p).hp < ctx.core.player(p).max_hp as i32)
                            .collect();
                        rng_pick(ctx, &cands)
                    }
                    _ => {
                        let cands = alive_others(ctx.core, m.owner);
                        rng_pick(ctx, &cands)
                    }
                };
                let Some(t) = target_opt else {
                    continue;
                };
                if t != m.owner {
                    let code = crate::archive::kind_code(kind) as u32;
                    set_flag(ctx.core, m.owner, "hs_qianhou_kind", code);
                    m.stage = HostStage::DrawSettle;
                } else {
                    set_flag(ctx.core, m.owner, "hs_qianhou_kind", 0);
                }
                emit_hearth_virtual(ctx, m.owner, kind, vec![t]);
                return if t != m.owner { Flow::Yield } else { Flow::Done };
            }
            Flow::Done
        }
        HostStage::DrawSettle => {
            if flag(ctx.core, m.owner, "hs_qianhou_kind") > 0
                && !ctx.core.player(m.owner).hand.is_empty()
            {
                return wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_QIANHOU_DISCARD);
            }
            set_flag(ctx.core, m.owner, "hs_qianhou_kind", 0);
            Flow::Done
        }
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            let code = flag(ctx.core, m.owner, "hs_qianhou_kind");
            set_flag(ctx.core, m.owner, "hs_qianhou_kind", 0);
            let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            discard_card_from_zone(ctx, m.owner, picked);
            ctx.log(LogEvent::CardDiscarded {
                player: m.owner,
                cards: vec![picked],
            });
            if let Ok(kind) = crate::archive::kind_from_code(code as u16) {
                if let Some(copy) = spawn_card_by_kind(ctx, kind) {
                    gain_card_to_hand(ctx, m.owner, copy);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_zhongji(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_ZHONGJI_DISCARD),
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            discard_card_from_zone(ctx, m.owner, picked);
            ctx.log(LogEvent::CardDiscarded {
                player: m.owner,
                cards: vec![picked],
            });
            for f in ctx.res.stack.iter_mut().rev() {
                if let EventData::Damage(d) = &mut f.data {
                    if d.source == Some(m.owner) && !d.prevented {
                        d.amount += 1;
                        break;
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_jinzhou(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::TargetChoice, ASK_HS_JINZHOU_TARGET),
        HostStage::TargetChoice => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::TargetChoice, 0);
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, t, "hs_jinzhou_sealed", 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}
