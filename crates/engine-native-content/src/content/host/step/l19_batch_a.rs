//! L19 末日乱斗 Batch A（模式 1–3 核心技能 + 20 注释/VFX 桩，`末日乱斗/extension.js`）。
//!
//! 覆盖项：
//! - 模式 1（僵尸模式）：`baozou`、`ganran`、`ganran2`、`_jisuangailv`、`_tuizhi`、`_tuizhi2`、
//!   `_jiangshi`、`_jiangshi2`、`_jiangshi3`、`_jiangshi4`、`_huzhu`
//! - 模式 2（兼并模式）：`_gainSkill`
//! - 模式 3（无尽模式）：`_wujinmoshiRE`、`wujinmoshiRE1`、`wujinmoshiRE2`、`_wujinmoshiDamage`、
//!   `_wujinmoshiDamageE`、`_wujinmoshiMaxHandCard`、`_wujinmoshiMaxHandCardE`、`_wujinmoshiDraw`、
//!   `_wujinmoshiDrawE`、`_wujinmoshiRecover`、`_wujinmoshiRecoverE`
//! - 20 个注释/VFX 桩：`chanceIdentity`、`chanceIdentity0`、`chanceIdentity2`..`chanceIdentity15`、
//!   `_jiangshiTx`、`_jiangshiTx2`、`_jiangshiTx3`、`_keepideskill`

use super::super::*;
use crate::catalog::{CardKind, Catalog, QueryCtx, SkillLimits, TriggerMeta};
use crate::event::{EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::state::{GameMode, GameStateCore, Role, SkillInstance};

#[path = "l19_batch_a_asks.rs"]
mod l19_batch_a_asks;
pub(in crate::content::host) use l19_batch_a_asks::ask_l19_batch_a;

pub(super) const ASK_GANRAN2_EQUIP: u32 = 10;
pub(super) const ASK_HUZHU_DISCARD_TAO: u32 = 20;
pub(super) const ASK_HUZHU_TARGET: u32 = 21;

pub(super) fn flag(core: &GameStateCore, p: PlayerId, key: &str) -> u32 {
    core.vm_aux
        .flags
        .get(&(p, key.to_string()))
        .copied()
        .unwrap_or(0)
}

pub(super) fn set_flag(core: &mut GameStateCore, p: PlayerId, key: &str, val: u32) {
    if val == 0 {
        core.vm_aux.flags.remove(&(p, key.to_string()));
    } else {
        core.vm_aux.flags.insert((p, key.to_string()), val);
    }
}

pub(super) fn turn_stamp(core: &GameStateCore) -> u32 {
    (core.turn.round.max(1) << 8) | u32::from(core.turn.current)
}

pub(super) fn player_role(core: &GameStateCore, p: PlayerId) -> Option<Role> {
    core.identity_roles().and_then(|r| r.get(p as usize).copied())
}

pub(super) fn set_player_role(core: &mut GameStateCore, p: PlayerId, role: Role) {
    if let GameMode::Identity { roles } = &mut core.mode {
        if let Some(slot) = roles.get_mut(p as usize) {
            *slot = role;
        }
    }
}

pub(super) fn has_skill_named(core: &GameStateCore, catalog: &Catalog, p: PlayerId, name: &str) -> bool {
    catalog
        .skill_by_name(name)
        .is_some_and(|def| core.has_skill(p, def))
}

pub(super) fn grant_skill_named(ctx: &mut Ctx<'_>, p: PlayerId, name: &str) -> Option<u16> {
    if let Some(def) = ctx.catalog.skill_by_name(name) {
        ctx.core.skills.insert(
            (p, def),
            SkillInstance {
                owner: p,
                enabled: true,
                once_used: false,
                expires_end_of_turn: false,
                rounds: 0,
            },
        );
        return Some(def);
    }
    None
}

pub(super) fn remove_skill_named(ctx: &mut Ctx<'_>, p: PlayerId, name: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(name) {
        ctx.core.skills.remove(&(p, def));
    }
}

pub(super) fn grant_zombie_skills(ctx: &mut Ctx<'_>, p: PlayerId, is_fz: bool) {
    let list: &[&str] = if is_fz {
        &["xunmeng", "zaibian", "ganran", "wansha", "paoxiao"]
    } else {
        &["baozou", "wansha", "xueji", "shishi", "ganran"]
    };
    for &sk in list {
        let _ = grant_skill_named(ctx, p, sk);
    }
}

pub(super) fn refund_once_used(ctx: &mut Ctx<'_>, p: PlayerId, name: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(name) {
        if let Some(inst) = ctx.core.skills.get_mut(&(p, def)) {
            inst.once_used = false;
        }
    }
}

pub(super) fn discard_cards(ctx: &mut Ctx<'_>, p: PlayerId, cards: &[CardId]) {
    let mut discarded = Vec::new();
    for &c in cards {
        if ctx.core.player(p).hand.contains(&c) {
            ctx.discard_from_hand(p, c);
            discarded.push(c);
        } else if ctx.core.player(p).equips.contains(&c) {
            ctx.take_owned_card(p, c);
            ctx.to_discard(c);
            discarded.push(c);
        }
    }
    if !discarded.is_empty() {
        ctx.emit(EventData::CardsDiscarded {
            player: p,
            cards: discarded,
        });
    }
}

pub(super) fn equip_hand_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<CardId> {
    core.player(owner)
        .hand
        .iter()
        .copied()
        .filter(|&cid| {
            catalog
                .cards
                .get(cid as usize)
                .is_some_and(|cd| cd.kind.equip_slot().is_some())
        })
        .collect()
}

pub(super) fn tao_hand_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<CardId> {
    core.player(owner)
        .hand
        .iter()
        .copied()
        .filter(|&cid| {
            catalog
                .cards
                .get(cid as usize)
                .is_some_and(|cd| cd.kind == CardKind::Tao)
        })
        .collect()
}

pub(super) fn huzhu_allies(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<PlayerId> {
    core.seat_order_from(owner)
        .into_iter()
        .filter(|&p| {
            p != owner
                && core.player(p).alive
                && core.player(p).hp < core.player(p).max_hp as i32
                && flag(core, p, "zombie_role") == 0
                && !matches!(player_role(core, p), Some(Role::Rebel | Role::Traitor))
                && core.computed_distance(catalog, owner, p) <= 1
        })
        .collect()
}

fn current_tuizhi(core: &GameStateCore, owner: PlayerId) -> u32 {
    let direct = flag(core, owner, "_tuizhi").max(flag(core, owner, "l19_tuizhi_turns"));
    if direct > 0 {
        return direct;
    }
    (0..core.players.len() as PlayerId)
        .find(|&p| player_role(core, p) == Some(Role::Lord))
        .map(|z| flag(core, z, "_tuizhi").max(flag(core, z, "l19_tuizhi_turns")))
        .unwrap_or(0)
}

pub(in crate::content::host) fn trigger_meta_l19_batch_a(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, priority: i32, extra_events: Vec<EventKind>| TriggerMeta {
        event,
        timing: Timing::Normal,
        priority,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "baozou" | "ganran" => Some(meta(EventKind::TurnBegin, 15, vec![EventKind::PhaseBegin])),
        "ganran2" | "_huzhu" => Some(meta(EventKind::PhaseBegin, 10, Vec::new())),
        "_jisuangailv" => Some(meta(EventKind::TurnBegin, 25, Vec::new())),
        "_tuizhi" => Some(meta(EventKind::PhaseBegin, 20, Vec::new())),
        "_tuizhi2" | "_tuizhi3" => Some(meta(EventKind::PhaseBegin, 10, Vec::new())),
        "_jiangshi2" => Some(meta(EventKind::PhaseBegin, 15, Vec::new())),
        "_jiangshi4" => Some(meta(EventKind::Dying, 25, vec![EventKind::Death])),
        "_jiangshi" => Some(meta(EventKind::Dying, 15, vec![EventKind::Death])),
        "_jiangshi3" | "_gainSkill" => Some(meta(EventKind::Death, 15, Vec::new())),
        "_wujinmoshiRE" => Some(meta(EventKind::Dying, 22, Vec::new())),
        "wujinmoshiRE1" => Some(meta(EventKind::Dying, 21, Vec::new())),
        "wujinmoshiRE2" => Some(meta(EventKind::Dying, 20, Vec::new())),
        "_wujinmoshiDamage" | "_wujinmoshiDamageE" => Some(meta(EventKind::Damage, 15, Vec::new())),
        "_wujinmoshiDraw" | "_wujinmoshiDrawE" => Some(meta(EventKind::PhaseBegin, 15, Vec::new())),
        "_wujinmoshiRecover" | "_wujinmoshiRecoverE" => Some(meta(
            EventKind::PhaseBegin,
            15,
            vec![EventKind::PhaseEnd, EventKind::TurnEnded],
        )),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l19_batch_a(name: &str) -> SkillLimits {
    match name {
        "_huzhu" => SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        _ => SkillLimits::default(),
    }
}

pub(in crate::content::host) fn filter_l19_batch_a(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if !ctx.state.player(owner).alive
        && !matches!(
            skill.name.as_str(),
            "_jiangshi" | "_jiangshi4" | "_wujinmoshiRE" | "wujinmoshiRE1" | "wujinmoshiRE2"
        )
    {
        return false;
    }
    match skill.name.as_str() {
        "baozou" | "ganran" => match event {
            EventData::TurnBegin { player } => *player == owner,
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } => *player == owner,
            _ => false,
        },
        "ganran2" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } if *player == owner && !equip_hand_cards(ctx.state, ctx.catalog, owner).is_empty()
        ),
        "_huzhu" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } if *player == owner
                && flag(ctx.state, owner, "_huzhu_used_stamp") != turn_stamp(ctx.state)
                && !tao_hand_cards(ctx.state, ctx.catalog, owner).is_empty()
                && !huzhu_allies(ctx.state, ctx.catalog, owner).is_empty()
        ),
        "_jisuangailv" => matches!(
            event,
            EventData::TurnBegin { .. } if flag(ctx.state, owner, "_jisuangailv_init") == 0
        ),
        "_tuizhi" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } if *player == owner && flag(ctx.state, owner, "zombie_role") == 0
        ),
        "_tuizhi2" | "_tuizhi3" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } if *player == owner && current_tuizhi(ctx.state, owner) >= 7
        ),
        "_jiangshi2" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } if *player == owner
                && flag(ctx.state, owner, "_jiangshi2_transformed") == 0
                && flag(ctx.state, owner, "zombie_role") == 0
                && current_tuizhi(ctx.state, owner) == 2
        ),
        "_jiangshi4" => match event {
            EventData::Dying { player, .. } | EventData::Death { player, .. } => {
                *player == owner
                    && ctx.state.player(owner).hp <= 0
                    && flag(ctx.state, owner, "_jiangshi4_transferred") == 0
                    && (player_role(ctx.state, owner) == Some(Role::Lord)
                        || flag(ctx.state, owner, "is_zhu") == 1
                        || owner == 0)
            }
            _ => false,
        },
        "_jiangshi" => match event {
            EventData::Dying { player, .. } | EventData::Death { player, .. } => {
                *player == owner
                    && ctx.state.player(owner).hp <= 0
                    && flag(ctx.state, owner, "_jiangshi_infected") == 0
                    && flag(ctx.state, owner, "_jiangshi_transformed") == 0
                    && flag(ctx.state, owner, "zombie_role") == 0
            }
            _ => false,
        },
        "_jiangshi3" => matches!(
            event,
            EventData::Death {
                player,
                killer: Some(k),
            } if *k == owner
                && *player != owner
                && (flag(ctx.state, owner, "zombie_role") == 2
                    || player_role(ctx.state, owner) == Some(Role::Traitor))
        ),
        "_gainSkill" => matches!(
            event,
            EventData::Death {
                player,
                killer: Some(k),
            } if *k == owner && *player != owner
        ),
        "_wujinmoshiRE" | "wujinmoshiRE1" | "wujinmoshiRE2" => matches!(
            event,
            EventData::Dying { player, .. }
                if *player == owner
                    && ctx.state.player(owner).hp <= 0
                    && (has_skill_named(ctx.state, ctx.catalog, owner, "wujinmoshiRE1")
                        || flag(ctx.state, owner, "wujinmoshiRE1") > 0
                        || has_skill_named(ctx.state, ctx.catalog, owner, "wujinmoshiRE2")
                        || flag(ctx.state, owner, "wujinmoshiRE2") > 0)
        ),
        "_wujinmoshiDamage" | "_wujinmoshiDamageE" => matches!(
            event,
            EventData::Damage(d) if !d.prevented && d.amount > 0 && d.source == Some(owner)
        ),
        "_wujinmoshiDraw" | "_wujinmoshiDrawE" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Draw,
                begin: true,
            } if *player == owner
        ),
        "_wujinmoshiRecover" | "_wujinmoshiRecoverE" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } if *player == owner
                && ctx.state.player(owner).hp < ctx.state.player(owner).max_hp as i32
        ),
        _ => false,
    }
}

pub(in crate::content::host) fn activate_l19_batch_a(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    if !ctx.state.player(owner).alive {
        return None;
    }
    let can_activate = match skill.name.as_str() {
        "ganran2" => !equip_hand_cards(ctx.state, ctx.catalog, owner).is_empty(),
        "_huzhu" => {
            flag(ctx.state, owner, "_huzhu_used_stamp") != turn_stamp(ctx.state)
                && !tao_hand_cards(ctx.state, ctx.catalog, owner).is_empty()
                && !huzhu_allies(ctx.state, ctx.catalog, owner).is_empty()
        }
        _ => false,
    };
    if !can_activate {
        return None;
    }
    Some(Box::new(HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L19DoomsdayBatchA,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }))
}

pub(in crate::content::host) fn machine_l19_batch_a(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut target = None;
    let mut targets = Vec::new();
    match event {
        EventData::Dying { player, source } => {
            target = *source;
            targets.push(*player);
        }
        EventData::Death { player, killer } => {
            target = *killer;
            targets.push(*player);
        }
        EventData::Damage(d) => {
            target = Some(d.target);
        }
        _ => {}
    }
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L19DoomsdayBatchA,
        owner,
        target,
        card: None,
        cards: Vec::new(),
        targets,
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn handle_l19_batch_a(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    let owner = m.owner;
    let ask_id = flag(ctx.core, owner, "l19_ask_id");
    if m.stage != HostStage::Act && ctx.answer().is_empty() {
        if l19_batch_a_asks::is_first_cancellable(ask_id) {
            refund_once_used(ctx, owner, &m.def_key);
        }
        set_flag(ctx.core, owner, "l19_ask_id", 0);
        return Flow::Done;
    }
    match m.def_key.as_str() {
        "baozou" => {
            set_flag(ctx.core, owner, "baozou_unlimited", 1);
            set_flag(ctx.core, owner, "l19_baozou_active", 1);
            set_flag(ctx.core, owner, "l19_baozou_sha_unlimited", 1);
            Flow::Done
        }
        "ganran" => {
            set_flag(ctx.core, owner, "ganran_active", 1);
            Flow::Done
        }
        "ganran2" => l19_batch_a_asks::step_ganran2(m, ctx),
        "_huzhu" => l19_batch_a_asks::step_huzhu(m, ctx),
        "_jisuangailv" => {
            set_flag(ctx.core, owner, "_jisuangailv_init", 1);
            set_flag(ctx.core, 0, "_jisuangailv_init", 1);
            Flow::Done
        }
        "_tuizhi" => {
            let next = flag(ctx.core, owner, "_tuizhi")
                .max(flag(ctx.core, owner, "l19_tuizhi_turns"))
                + 1;
            set_flag(ctx.core, owner, "_tuizhi", next);
            set_flag(ctx.core, owner, "l19_tuizhi_turns", next);
            if next >= 8 {
                set_flag(ctx.core, owner, "_tuizhi2_human_win", 1);
                set_flag(ctx.core, owner, "l19_tuizhi_victory", 1);
            }
            Flow::Done
        }
        "_tuizhi2" | "_tuizhi3" => {
            if current_tuizhi(ctx.core, owner) >= 8 {
                set_flag(ctx.core, owner, "_tuizhi2_human_win", 1);
                set_flag(ctx.core, owner, "l19_tuizhi_victory", 1);
            }
            Flow::Done
        }
        "_jiangshi" => {
            if ctx.core.player(owner).hp <= 0 && flag(ctx.core, owner, "_jiangshi_infected") == 0 {
                l19_batch_a_asks::transform_player_into_zombie(ctx, owner, m.target);
            }
            Flow::Done
        }
        "_jiangshi2" => {
            if flag(ctx.core, owner, "_jiangshi2_transformed") == 0
                && current_tuizhi(ctx.core, owner) == 2
            {
                ctx.core.player_mut(owner).max_hp += 5;
                ctx.recover(owner, 3, None);
                set_player_role(ctx.core, owner, Role::Rebel);
                grant_zombie_skills(ctx, owner, true);
                set_flag(ctx.core, owner, "_jiangshi2_transformed", 1);
                set_flag(ctx.core, owner, "_jiangshi_infected", 1);
                set_flag(ctx.core, owner, "zombie_role", 1);
            }
            Flow::Done
        }
        "_jiangshi3" => {
            ctx.core.player_mut(owner).max_hp = 5;
            ctx.recover(owner, 2, None);
            set_player_role(ctx.core, owner, Role::Rebel);
            grant_zombie_skills(ctx, owner, true);
            set_flag(ctx.core, owner, "_jiangshi3_transformed", 1);
            set_flag(ctx.core, owner, "zombie_role", 1);
            Flow::Done
        }
        "_jiangshi4" => l19_batch_a_asks::step_jiangshi4(m, ctx),
        "_gainSkill" => {
            if let Some(&victim) = m.targets.first() {
                let victim_defs: Vec<u16> = ctx
                    .core
                    .skills
                    .iter()
                    .filter_map(|(&(p, def), inst)| (p == victim && inst.enabled).then_some(def))
                    .collect();
                for def in victim_defs {
                    if ctx.catalog.skills.get(def as usize).is_some_and(|s| s.name != "_gainSkill")
                    {
                        ctx.core.skills.insert(
                            (owner, def),
                            SkillInstance {
                                owner,
                                enabled: true,
                                once_used: false,
                                expires_end_of_turn: false,
                                rounds: 0,
                            },
                        );
                    }
                }
                for g_opt in ctx.core.player(victim).general_slots {
                    if let Some(g_idx) = g_opt {
                        if let Some((_, _, sks, _, _)) = ctx.catalog.general_meta(g_idx) {
                            for sk in sks {
                                let _ = grant_skill_named(ctx, owner, sk);
                            }
                        }
                    }
                }
                let cnt = flag(ctx.core, owner, "_gainSkill_count") + 1;
                set_flag(ctx.core, owner, "_gainSkill_count", cnt);
                set_flag(ctx.core, owner, "l19_annexed_skill_count", cnt);
                set_flag(ctx.core, owner, "drlt_duorui_sealed_slots_mask", 0);
                set_flag(ctx.core, owner, "pro_drlt_duorui_sealed_slots_mask", 0);
            }
            Flow::Done
        }
        "_wujinmoshiRE" | "wujinmoshiRE1" | "wujinmoshiRE2" => {
            l19_batch_a_asks::step_wujin_re(m, ctx)
        }
        "_wujinmoshiDamage" | "_wujinmoshiDamageE" => {
            let bonus = flag(ctx.core, owner, "wujin_damage_bonus")
                .max(flag(ctx.core, owner, "l19_wujin_damage_bonus"))
                .max(1);
            set_flag(ctx.core, owner, "_wujinmoshiDamage_triggered", bonus);
            Flow::Done
        }
        "_wujinmoshiDraw" | "_wujinmoshiDrawE" => {
            let bonus = flag(ctx.core, owner, "wujin_draw_bonus")
                .max(flag(ctx.core, owner, "l19_wujin_draw_bonus"))
                .max(1);
            ctx.start_draw(owner, bonus);
            set_flag(ctx.core, owner, "_wujinmoshiDraw_drawn", bonus);
            Flow::Done
        }
        "_wujinmoshiRecover" | "_wujinmoshiRecoverE" => {
            if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                let bonus = flag(ctx.core, owner, "wujin_recover_bonus")
                    .max(flag(ctx.core, owner, "l19_wujin_recover_bonus"))
                    .max(1);
                ctx.recover(owner, bonus, None);
                set_flag(ctx.core, owner, "_wujinmoshiRecover_healed", bonus);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
