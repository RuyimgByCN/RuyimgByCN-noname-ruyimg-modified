//! L11-6a 太虚自有 NPC 武将技能家族（`HostBehavior::L11OwnNpc`，tag 1389，29 技）。

mod asks;
mod combat_damage;
mod death_revive;
mod filter;
mod mahou;
mod mark_expand;

pub(in crate::content::host) use asks::ask_l11_own_npc;
pub(crate) use death_revive::on_player_died_l11_own_npc;
pub(in crate::content::host) use filter::filter_l11_own_npc;
pub(in crate::content::host) use mark_expand::is_youji_eligible_use;

use asks::*;
use super::super::*;
use super::l11_common::*;
use crate::catalog::{QueryCtx, SkillLimits, TriggerMeta};
use crate::event::{EventData, EventKind, Timing};
use crate::ids::{CardId, EventId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::state::GameStateCore;

pub(in crate::content::host) const MODE_TICK: u32 = 91;
pub(in crate::content::host) const MODE_SECONDARY: u32 = 92;
pub(in crate::content::host) const MODE_SHIELD: u32 = 93;

pub(in crate::content::host) fn wait_npc_ask(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    stage: HostStage,
    ask_id: u32,
) -> Flow {
    m.card = Some(ask_id);
    if let Some(exp) = ask_l11_own_npc(
        stage,
        m.owner,
        m.target,
        &m.targets,
        m.card,
        &m.cards,
        ctx.core,
        ctx.catalog,
    ) {
        m.stage = stage;
        Flow::WaitChoice {
            observer: exp.observer,
            spec: crate::choice::ChoiceSpec {
                kind: exp.kind,
                prompt: crate::content::names::intern(exp.prompt.to_string()),
                options: exp.options,
                min: exp.min,
                max: exp.max,
                allow_cancel: exp.allow_cancel,
            },
        }
    } else {
        m.card = None;
        Flow::Done
    }
}

pub(in crate::content::host) fn current_use_frame_id(ctx: &Ctx<'_>) -> Option<EventId> {
    ctx.res.stack.iter().rev().find_map(|f| {
        matches!(f.data, EventData::CardUsed(_)).then_some(f.id)
    })
}

pub(in crate::content::host) fn trigger_meta_l11_own_npc(
    name: &str,
    kind: Option<EventKind>,
) -> Option<TriggerMeta> {
    let meta = |event: EventKind, timing: Timing, priority: i32, optional: bool, extra_events: Vec<EventKind>| {
        TriggerMeta {
            event,
            timing,
            priority,
            order_selectable: false,
            optional,
            extra_events,
        }
    };
    match name {
        "txhj_zuijiu" | "txhj_baoji" | "txhj_shiyuan" => {
            Some(meta(EventKind::DamageBefore, Timing::Normal, 0, false, Vec::new()))
        }
        "txhj_kuangbao" => Some(meta(
            EventKind::DamageBefore,
            Timing::Normal,
            0,
            false,
            vec![EventKind::Dying],
        )),
        "txhj_wangyuan" => Some(meta(EventKind::DamageBefore, Timing::Normal, 4, false, Vec::new())),
        "txhj_guiyan" => {
            let prio = if kind == Some(EventKind::DamageBefore) { 30 } else { 2 };
            Some(meta(
                EventKind::PhaseBegin,
                Timing::Normal,
                prio,
                false,
                vec![EventKind::DamageBefore],
            ))
        }
        "txhj_xiefeng" => {
            let prio = if kind == Some(EventKind::DamageBefore) { -10 } else { 0 };
            Some(meta(
                EventKind::TurnPhaseAfter,
                Timing::Normal,
                prio,
                false,
                vec![EventKind::TurnPhaseEnd, EventKind::DamageBefore],
            ))
        }
        "txhj_juhun" => {
            let prio = if kind == Some(EventKind::TurnPhaseAfter) { -15 } else { 1 };
            Some(meta(
                EventKind::TurnBegin,
                Timing::Normal,
                prio,
                false,
                vec![EventKind::TurnPhaseAfter],
            ))
        }
        "txhj_baolian" | "tx_modao" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new())),
        "txhj_beiming" | "txhj_wangzu" => Some(meta(EventKind::Death, Timing::Normal, 0, false, Vec::new())),
        "txhj_tunshi" => Some(meta(EventKind::TurnBegin, Timing::Normal, 0, false, Vec::new())),
        "txhj_chuanyun" => Some(meta(EventKind::TurnPhaseEnd, Timing::Normal, 0, false, Vec::new())),
        "txhj_guihuo" | "txhj_luolei" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new())),
        "txhj_huangjin" => Some(meta(EventKind::CardUsed, Timing::Normal, 0, false, Vec::new())),
        "txhj_guimen" => Some(meta(EventKind::CardsDiscarded, Timing::Normal, 0, false, Vec::new())),
        "txhj_zhouzu" | "txhj_zhoufa" => {
            Some(meta(EventKind::TurnPhaseEnd, Timing::Normal, 0, false, Vec::new()))
        }
        "txhj_renfang" => Some(meta(EventKind::TurnPhaseEnd, Timing::Normal, 2, false, Vec::new())),
        "txhj_dizhou" => {
            let prio = if kind == Some(EventKind::CardUsed) { 10 } else { 0 };
            Some(meta(
                EventKind::TurnBegin,
                Timing::Normal,
                prio,
                false,
                vec![EventKind::TurnPhaseEnd, EventKind::CardUsed, EventKind::DamageEnd],
            ))
        }
        "txhj_zhoulei" => Some(meta(EventKind::CardsDrawn, Timing::Normal, 0, false, Vec::new())),
        "txqyyouji" => Some(meta(EventKind::CardUsed, Timing::Normal, -2, false, Vec::new())),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l11_own_npc(name: &str) -> SkillLimits {
    // Active Play limits for `txhj_zhouzu`/`txhj_zhoufa`/`txhj_renfang` are gated by `vm_aux.once_count`
    // in `activate_l11_own_npc` so their `TurnPhaseEnd` mahou ticks still run on the activation turn.
    if matches!(name, "txhj_yaoshu" | "txhj_zhuhun") {
        SkillLimits {
            once_per_turn: true,
            active_outside_play: false,
        }
    } else {
        SkillLimits::default()
    }
}

pub(in crate::content::host) fn machine_l11_own_npc(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut target = None;
    let mut cards = Vec::new();
    let mut card = None;
    match (skill.name.as_str(), event) {
        ("txhj_kuangbao", EventData::Dying { .. }) => card = Some(MODE_SECONDARY),
        ("txhj_guiyan", EventData::Damage(_)) => card = Some(MODE_SECONDARY),
        ("txhj_xiefeng", EventData::Damage(_)) => card = Some(MODE_SHIELD),
        ("txhj_xiefeng", EventData::TurnEndWindow { after: false, .. }) => card = Some(MODE_TICK),
        ("txhj_juhun", EventData::TurnEndWindow { after: true, .. }) => card = Some(MODE_SECONDARY),
        ("txhj_huangjin", EventData::CardUsed(u)) => {
            if let Some(c) = u.card {
                cards.push(c);
            }
        }
        ("txhj_guimen", EventData::CardsDiscarded { cards: cs, .. }) => cards = cs.clone(),
        ("txhj_dizhou", EventData::CardUsed(u)) => {
            card = Some(MODE_SECONDARY);
            target = Some(u.player);
        }
        ("txhj_dizhou", EventData::Damage(_)) => card = Some(MODE_SHIELD),
        ("txhj_zhouzu" | "txhj_zhoufa" | "txhj_renfang", EventData::TurnEndWindow { after: false, .. }) => {
            card = Some(MODE_TICK);
        }
        ("txhj_zhoulei", EventData::CardsDrawn { player, .. }) => target = Some(*player),
        ("txhj_beiming", EventData::Death { killer, .. }) => target = *killer,
        _ => {}
    }
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L11OwnNpc,
        owner,
        target,
        card,
        cards,
        targets: Vec::new(),
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn handle_l11_own_npc(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if let Some(f) = combat_damage::handle_combat_damage(m, ctx) {
        return f;
    }
    if let Some(f) = mark_expand::handle_mark_expand(m, ctx) {
        return f;
    }
    if let Some(f) = mahou::handle_mahou(m, ctx) {
        return f;
    }
    if let Some(f) = death_revive::handle_death_revive(m, ctx) {
        return f;
    }
    Flow::Done
}

pub(crate) fn on_round_start_l11_own_npc(ctx: &mut Ctx<'_>) {
    if ctx.core.turn.round <= 1 {
        return;
    }
    let seats = ctx.core.players.len() as PlayerId;
    for owner in 0..seats {
        if !has_named_skill(ctx.core, ctx.catalog, owner, "txhj_kuangbao")
            || !is_locked_unblocked_l11_owner(ctx.core, ctx.catalog, owner)
        {
            continue;
        }
        let debuff = flag(ctx.core, owner, "l11_kuangbao_debuff");
        if debuff == 0 {
            let add = flag(ctx.core, owner, "l11_kuangbao_add") + 1;
            if add >= 7 {
                set_flag(ctx.core, owner, "l11_kuangbao_add", 0);
                let bonus = flag(ctx.core, owner, "l11_kuangbao_bonus") + 1;
                set_flag(ctx.core, owner, "l11_kuangbao_bonus", bonus);
            } else {
                set_flag(ctx.core, owner, "l11_kuangbao_add", add);
            }
        }
        set_flag(ctx.core, owner, "l11_kuangbao_debuff", 0);
    }
}

pub(crate) fn on_turn_begin_l11_own_npc(ctx: &mut Ctx<'_>, player: PlayerId) {
    set_flag(ctx.core, player, "l11_juhun_armed", 0);
}

pub(crate) fn on_turn_phase_after_l11_own_npc(ctx: &mut Ctx<'_>, player: PlayerId) {
    set_flag(ctx.core, player, "l11_guiyan_target", 0);
    set_flag(ctx.core, player, "l11_guiyan_turn", 0);
    let seats = ctx.core.players.len() as PlayerId;
    for seat in 0..seats {
        if !ctx.core.player(seat).alive {
            continue;
        }
        let rem = flag(ctx.core, seat, "l11_yaoshu_mahou_rem");
        if rem > 1 {
            set_flag(ctx.core, seat, "l11_yaoshu_mahou_rem", rem - 1);
        } else if rem == 1 {
            let orig = flag(ctx.core, seat, "l11_yaoshu_mahou_orig");
            set_flag(ctx.core, seat, "l11_yaoshu_mahou_rem", 0);
            set_flag(ctx.core, seat, "l11_yaoshu_mahou_orig", 0);
            let stacks = flag(ctx.core, seat, "l11_yaoshu_effect_stacks");
            set_flag(ctx.core, seat, "l11_yaoshu_effect_stacks", stacks + orig);
        }
    }
}

pub(crate) fn on_skill_removed_l11_own_npc(
    core: &mut GameStateCore,
    owner: PlayerId,
    skill_name: &str,
) -> Vec<CardId> {
    match skill_name {
        "txhj_dizhou" | "txhj_renwang" => core
            .vm_aux
            .piles
            .remove(&(owner, skill_name.to_string()))
            .unwrap_or_default(),
        "txhj_juhun" => {
            set_flag(core, owner, "l11_juhun_dead_mask", 0);
            set_flag(core, owner, "l11_juhun_armed", 0);
            Vec::new()
        }
        "txhj_zhuhun" => {
            set_flag(core, owner, "l11_zhuhun_dead_mask", 0);
            Vec::new()
        }
        "txhj_xiefeng" => {
            set_flag(core, owner, "l11_xiefeng_stacks", 0);
            Vec::new()
        }
        "txhj_guiyan" => {
            set_flag(core, owner, "l11_guiyan_target", 0);
            set_flag(core, owner, "l11_guiyan_turn", 0);
            Vec::new()
        }
        _ => Vec::new(),
    }
}

pub(in crate::content::host) fn has_phase_use_l11_own_npc(name: &str) -> bool {
    matches!(
        name,
        "txhj_zhouzu" | "txhj_yaoshu" | "txhj_zhoufa" | "txhj_renfang" | "txhj_zhuhun"
    )
}

pub(in crate::content::host) fn activate_l11_own_npc(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    let state = ctx.state;
    if state.turn.phase != crate::event::Phase::Play
        || !is_unblocked_l11_owner(state, ctx.catalog, owner)
        || state.vm_aux.once_count(owner, 1, &skill.name) >= 1
    {
        return None;
    }
    let ok = match skill.name.as_str() {
        "txhj_zhouzu" => {
            flag(state, owner, "l11_mahou_zhouzu_rem") == 0 && !alive_others(state, owner).is_empty()
        }
        "txhj_yaoshu" => alive_others(state, owner)
            .into_iter()
            .any(|p| flag(state, p, "l11_yaoshu_mahou_rem") == 0),
        "txhj_zhoufa" => {
            flag(state, owner, "l11_mahou_zhoufa_rem") == 0 && !alive_others(state, owner).is_empty()
        }
        "txhj_renfang" => {
            flag(state, owner, "l11_mahou_renfang_rem") == 0
                && state
                    .vm_aux
                    .piles
                    .get(&(owner, "txhj_renwang".to_string()))
                    .is_some_and(|p| p.len() >= 36)
        }
        "txhj_zhuhun" => {
            let mask = flag(state, owner, "l11_zhuhun_dead_mask");
            (0..state.players.len() as PlayerId)
                .any(|p| p != owner && !state.player(p).alive && (mask & (1u32 << p)) != 0)
        }
        _ => false,
    };
    if !ok {
        return None;
    }
    Some(Box::new(HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L11OwnNpc,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }))
}
