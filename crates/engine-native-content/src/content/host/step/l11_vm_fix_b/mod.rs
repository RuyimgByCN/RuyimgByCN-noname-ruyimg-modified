//! L11-E2 `taixu` VM Fix B（`HostBehavior::L11VmFixB`，tag 1390，34 主技 + 13 伴生子技）。

mod active;
mod asks;
mod combat_triggers;
mod phase_triggers;
mod pindian;

pub(in crate::content::host) use phase_triggers::try_tiaojiyanmei_intrinsic;

use super::super::*;
use super::l11_common::*;
use super::l11_common_b2::{damage_effective_kind, is_ordinary_trick_kind};
use crate::catalog::{Catalog, EquipSlot, QueryCtx, SkillLimits, TriggerMeta};
use crate::event::{CardUseData, DrawOutcome, EventData, EventKind, Frame, LostOrigin, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

fn tmeta(
    event: EventKind,
    timing: Timing,
    priority: i32,
    optional: bool,
    extra_events: Vec<EventKind>,
) -> TriggerMeta {
    TriggerMeta {
        event,
        timing,
        priority,
        order_selectable: true,
        optional,
        extra_events,
    }
}

pub(in crate::content::host) fn trigger_meta_l11_vm_fix_b(
    name: &str,
    kind: Option<EventKind>,
) -> Option<TriggerMeta> {
    let m = match name {
        "qiangzhi" | "rejiangchi" | "twshengxi" | "yinghun" | "zhengjian" => {
            let mut extra = vec![];
            if name == "zhengjian" {
                extra.push(EventKind::TurnBegin);
            }
            tmeta(EventKind::PhaseBegin, Timing::Normal, 0, false, extra)
        }
        "sbyangwei_counter" | "rezhiyi" => tmeta(EventKind::PhaseBegin, Timing::Normal, 0, false, vec![]),
        "zaoxian" => tmeta(EventKind::PhaseBegin, Timing::Normal, 5, false, vec![]),
        "zaiqi" => tmeta(EventKind::DrawPhaseBegin1, Timing::Normal, 0, true, vec![]),
        "zhaohuo" => tmeta(EventKind::Dying, Timing::Normal, 0, false, vec![]),
        "repolu" => tmeta(EventKind::Death, Timing::Normal, 0, false, vec![]),
        "qiangzhi_draw" | "spchijie" | "spdengli" | "xinliegong" => {
            let extra = if name == "qiangzhi_draw" {
                vec![EventKind::Equipped]
            } else {
                vec![]
            };
            tmeta(EventKind::CardUsed, Timing::Normal, 0, true, extra)
        }
        "reshejian" | "sphuangen" | "zhuandui" => {
            tmeta(EventKind::CardUsed, Timing::Normal, 0, false, vec![])
        }
        "zhengjian_mark" => tmeta(
            EventKind::CardUsed,
            Timing::Normal,
            10,
            false,
            vec![EventKind::CardResponded, EventKind::Equipped],
        ),
        "releiji" => tmeta(
            EventKind::CardUsed,
            Timing::Normal,
            0,
            false,
            vec![EventKind::CardResponded],
        ),
        "qingxi" => tmeta(EventKind::DamageBegin1, Timing::Normal, 0, true, vec![]),
        "reweimu" => tmeta(EventKind::DamageBefore, Timing::Normal, 0, false, vec![]),
        "rejianxiong" | "yiyong" => tmeta(EventKind::DamageEnd, Timing::Normal, 0, true, vec![]),
        "rebeige" | "rejieming" => tmeta(EventKind::DamageEnd, Timing::Normal, 0, false, vec![]),
        "xinfu_qinguo" => tmeta(
            EventKind::Equipped,
            Timing::Normal,
            0,
            false,
            vec![EventKind::CardsLost],
        ),
        "xinfu_qinguo_recover" => tmeta(EventKind::CardsLost, Timing::Normal, 0, false, vec![]),
        _ => return None,
    };
    if let Some(k) = kind {
        if m.event != k && !m.extra_events.contains(&k) {
            return None;
        }
    }
    Some(m)
}

pub(in crate::content::host) fn limits_l11_vm_fix_b(name: &str) -> SkillLimits {
    let once = matches!(
        name,
        "quhu" | "refanjian" | "rexianzhen" | "shanxie" | "tianyi" | "tiaoxin" | "spchijie"
    );
    SkillLimits {
        once_per_turn: once,
        active_outside_play: false,
    }
}

pub(in crate::content::host) fn has_phase_use_l11_vm_fix_b(name: &str) -> bool {
    matches!(
        name,
        "quhu"
            | "refanjian"
            | "reluanwu"
            | "rexianzhen"
            | "sbyangwei"
            | "shanxie"
            | "shenfen"
            | "tianyi"
            | "tiaoxin"
    )
}

pub(in crate::content::host) fn activate_l11_vm_fix_b(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    let state = ctx.state;
    if state.turn.phase != Phase::Play || state.turn.current != owner || !state.player(owner).alive {
        return None;
    }
    if !is_unblocked_l11_owner(state, ctx.catalog, owner) {
        return None;
    }
    let def = ctx.catalog.skill_by_name(&skill.name)?;
    let inst = state.skills.get(&(owner, def))?;
    if !inst.enabled {
        return None;
    }
    let eligible = match skill.name.as_str() {
        "quhu" => {
            (!state.player(owner).hand.is_empty()
                || super::l11_common_b2::has_txyonglve_ready(state, ctx.catalog, owner))
                && alive_others(state, owner).into_iter().any(|p| {
                    state.player(p).hp > state.player(owner).hp
                        && (!state.player(p).hand.is_empty()
                            || super::l11_common_b2::has_txyonglve_ready(state, ctx.catalog, p))
                })
        }
        "rexianzhen" | "tianyi" => {
            (!state.player(owner).hand.is_empty()
                || super::l11_common_b2::has_txyonglve_ready(state, ctx.catalog, owner))
                && alive_others(state, owner).into_iter().any(|p| {
                    !state.player(p).hand.is_empty()
                        || super::l11_common_b2::has_txyonglve_ready(state, ctx.catalog, p)
                })
        }
        "refanjian" => !state.player(owner).hand.is_empty() && !alive_others(state, owner).is_empty(),
        "reluanwu" => flag(state, owner, "l11_vm_b:reluanwu_used") == 0 && !alive_others(state, owner).is_empty(),
        "sbyangwei" => !has_named_skill(state, ctx.catalog, owner, "sbyangwei_counter"),
        "shanxie" => true,
        "shenfen" => {
            flag(state, owner, "l11_vm_b:shenfen_used") == 0
                && state
                    .vm_aux
                    .marks
                    .get(&(owner, active::BAONU_MARK_ID))
                    .copied()
                    .unwrap_or(0)
                    >= 6
        }
        "tiaoxin" => alive_others(state, owner).into_iter().any(|p| {
            super::l11_common_b2::in_attack_range(state, ctx.catalog, p, owner)
                && !he_cards(state, p).is_empty()
        }),
        _ => false,
    };
    if !eligible {
        return None;
    }
    active_skill_machine(&skill.name, HostBehavior::L11VmFixB, owner)
}

pub(in crate::content::host) fn filter_l11_vm_fix_b(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    frames: &[Frame],
    frame: Option<&Frame>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    let name = skill.name.as_str();
    if name == "repolu" {
        if state.outcome.is_some() {
            return false;
        }
        return matches!(event, EventData::Death { player, .. } if *player == owner);
    }
    if !state.player(owner).alive {
        return false;
    }
    let is_locked = matches!(
        name,
        "sbyangwei_counter" | "rezhiyi" | "reweimu" | "zaoxian" | "zhaohuo" | "zhengjian" | "zhengjian_mark"
    );
    let unblocked = if is_locked {
        is_locked_unblocked_l11_owner(state, ctx.catalog, owner)
    } else {
        is_unblocked_l11_owner(state, ctx.catalog, owner)
    };
    if !unblocked {
        return false;
    }
    match name {
        "qiangzhi" => {
            matches!(event, EventData::Phase { player, phase: Phase::Play, begin: true } if *player == owner)
                && alive_others(state, owner)
                    .into_iter()
                    .any(|p| !state.player(p).hand.is_empty())
        }
        "qiangzhi_draw" => {
            let cat = flag(state, owner, "l11_vm_b:qiangzhi_cat");
            cat > 0
                && state.turn.phase == Phase::Play
                && state.turn.current == owner
                && match event {
                    EventData::CardUsed(u) => {
                        u.player == owner
                            && !u.cancelled
                            && phase_triggers::card_category_code(u.effective_kind(ctx.catalog)) == cat
                    }
                    EventData::Equipped { player, card, .. } => {
                        let fid = frame.map_or(0, |f| f.id);
                        *player == owner
                            && fid > 0
                            && flag(state, owner, &format!("l11_vm_b_equip_use:{fid}")) > 0
                            && phase_triggers::card_category_code(
                                ctx.catalog.resolved_card_kind(state, *card),
                            ) == cat
                    }
                    _ => false,
                }
        }
        "rejiangchi" => {
            matches!(event, EventData::Phase { player, phase: Phase::Play, begin: true } if *player == owner)
        }
        "sbyangwei_counter" => {
            matches!(event, EventData::Phase { player, phase: Phase::End, begin: true } if *player == owner)
        }
        "rezhiyi" => {
            let ts = turn_stamp(state);
            matches!(event, EventData::Phase { phase: Phase::End, begin: true, .. })
                && flag(state, owner, "l11_vm_b:rezhiyi_ts") == ts
                && flag(state, owner, "l11_vm_b:rezhiyi_mask") > 0
        }
        "twshengxi" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } if *player == owner => state
                .detached_cards
                .iter()
                .map(|&(c, _)| c)
                .chain(state.deck.iter().copied())
                .any(|c| {
                    ctx.catalog
                        .cards
                        .get(c as usize)
                        .is_some_and(|d| phase_triggers::is_tiaojiyanmei_name(d.name))
                }),
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } if *player == owner => {
                let ts = turn_stamp(state);
                flag(state, owner, "l11_vm_b:tw_used_ts") == ts
                    && flag(state, owner, "l11_vm_b:tw_dmg_ts") != ts
            }
            _ => false,
        },
        "yinghun" => {
            matches!(event, EventData::Phase { player, phase: Phase::Prepare, begin: true } if *player == owner)
                && state.player(owner).hp < state.player(owner).max_hp as i32
                && !alive_others(state, owner).is_empty()
        }
        "zaiqi" => {
            matches!(
                event,
                EventData::DrawPhase {
                    player,
                    num_fixed: false,
                    outcome: DrawOutcome::Running,
                    ..
                } if *player == owner
            ) && state.player(owner).hp < state.player(owner).max_hp as i32
        }
        "zaoxian" => {
            if ctx.catalog.skill_by_name("zaoxian").is_none()
                || flag(state, owner, "l11_vm_b:zaoxian_used") > 0
            {
                return false;
            }
            if !matches!(event, EventData::Phase { player, phase: Phase::Prepare, begin: true } if *player == owner)
            {
                return false;
            }
            let total: usize = ["retuntian", "tuntian", "屯田"]
                .iter()
                .filter_map(|p| state.vm_aux.piles.get(&(owner, (*p).to_string())))
                .map(|v| v.len())
                .sum();
            total >= 3
        }
        "zhaohuo" => {
            matches!(event, EventData::Dying { player, .. } if *player != owner)
                && state.player(owner).max_hp > 1
        }
        "zhengjian" => match event {
            EventData::TurnBegin { player } if *player == owner => (0..state.players.len() as PlayerId)
                .any(|p| has_named_skill(state, ctx.catalog, p, "zhengjian_mark")),
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } if *player == owner => true,
            _ => false,
        },
        "zhengjian_mark" => match event {
            EventData::CardUsed(u) => u.player == owner && !u.cancelled,
            EventData::CardResponded { player, .. } => *player == owner,
            EventData::Equipped { player, .. } => {
                let fid = frame.map_or(0, |f| f.id);
                *player == owner
                    && fid > 0
                    && flag(state, owner, &format!("l11_vm_b_equip_use:{fid}")) > 0
            }
            _ => false,
        },
        "qingxi" => match event {
            EventData::Damage(d) => {
                !d.prevented
                    && d.amount > 0
                    && d.source == Some(owner)
                    && damage_effective_kind(state, frames, frame, ctx.catalog, d)
                        .is_some_and(|k| k.is_sha_like())
                    && asks::weapon_range_sum(state, ctx.catalog, owner) > 0
                    && state.player(d.target).alive
            }
            _ => false,
        },
        "rebeige" => match event {
            EventData::Damage(d) => {
                !d.prevented
                    && d.amount > 0
                    && d.source.is_some()
                    && state.player(d.target).alive
                    && !he_cards(state, owner).is_empty()
                    && damage_effective_kind(state, frames, frame, ctx.catalog, d)
                        .is_some_and(|k| k.is_sha_like())
            }
            _ => false,
        },
        "rejianxiong" => matches!(
            event,
            EventData::Damage(d)
                if !d.prevented
                    && d.amount > 0
                    && d.target == owner
                    && d.card
                        .is_some_and(|cid| state.table.contains(&cid) || state.discard.contains(&cid))
        ),
        "rejieming" => matches!(
            event,
            EventData::Damage(d) if !d.prevented && d.amount > 0 && d.target == owner
        ),
        "releiji" => match event {
            EventData::CardUsed(u) => {
                u.player == owner
                    && !u.cancelled
                    && u.effective_kind(ctx.catalog) == CardKind::Shan
                    && !alive_others(state, owner).is_empty()
            }
            EventData::CardResponded { player, card } => {
                *player == owner
                    && ctx.catalog.resolved_card_kind(state, *card) == CardKind::Shan
                    && !alive_others(state, owner).is_empty()
            }
            _ => false,
        },
        "reshejian" => {
            let ts = turn_stamp(state);
            let cnt = if flag(state, owner, "l11_vm_b:reshejian_ts") == ts {
                flag(state, owner, "l11_vm_b:reshejian_cnt")
            } else {
                0
            };
            cnt < 2
                && state.player(owner).hand.len() >= 2
                && matches!(
                    event,
                    EventData::CardUsed(u)
                        if !u.cancelled
                            && u.player != owner
                            && u.targets.len() == 1
                            && u.targets[0] == owner
                )
        }
        "reweimu" => matches!(
            event,
            EventData::Damage(d)
                if !d.prevented && d.amount > 0 && d.target == owner && state.turn.current == owner
        ),
        "spchijie" => {
            flag(state, owner, "l11_vm_b:spchijie_used_ts") != turn_stamp(state)
                && matches!(
                    event,
                    EventData::CardUsed(u)
                        if !u.cancelled
                            && u.player != owner
                            && u.targets.len() == 1
                            && u.targets[0] == owner
                )
        }
        "spdengli" => match event {
            EventData::CardUsed(u) if !u.cancelled && u.effective_kind(ctx.catalog).is_sha_like() => {
                let ohp = state.player(owner).hp;
                if u.player == owner {
                    u.targets
                        .iter()
                        .any(|&t| state.player(t).alive && state.player(t).hp == ohp)
                } else {
                    u.targets.contains(&owner)
                        && state.player(u.player).alive
                        && state.player(u.player).hp == ohp
                }
            }
            _ => false,
        },
        "sphuangen" => match event {
            EventData::CardUsed(u) => {
                let fid = frame.map_or(0, |f| f.id);
                fid > 0
                    && flag(state, owner, "l11_vm_b:sphuangen_fid") != fid
                    && !u.cancelled
                    && u.targets.len() >= 2
                    && state.player(owner).hp > 0
                    && is_ordinary_trick_kind(u.effective_kind(ctx.catalog))
            }
            _ => false,
        },
        "xinliegong" => match event {
            EventData::CardUsed(u)
                if !u.cancelled
                    && u.player == owner
                    && u.effective_kind(ctx.catalog).is_sha_like() =>
            {
                let oh = state.player(owner).hand.len();
                let ohp = state.player(owner).hp;
                u.targets.iter().any(|&t| {
                    state.player(t).alive
                        && (state.player(t).hand.len() <= oh || state.player(t).hp >= ohp)
                })
            }
            _ => false,
        },
        "yiyong" => match event {
            EventData::Damage(d) => {
                !d.prevented
                    && d.amount > 0
                    && d.target == owner
                    && d.source.is_some_and(|s| s != owner && state.player(s).alive)
                    && d.card.is_some_and(|c| state.table.contains(&c) || state.discard.contains(&c))
                    && state
                        .player(owner)
                        .equips
                        .iter()
                        .any(|&c| ctx.catalog.card_slot_of(state, c) == Some(EquipSlot::Weapon))
                    && damage_effective_kind(state, frames, frame, ctx.catalog, d)
                        .is_some_and(|k| k.is_sha_like())
            }
            _ => false,
        },
        "zhuandui" => {
            if !pindian::can_compare_pindian(state, ctx.catalog, owner) {
                return false;
            }
            match event {
                EventData::CardUsed(u) if !u.cancelled && u.effective_kind(ctx.catalog).is_sha_like() => {
                    if u.player == owner {
                        u.targets.iter().any(|&t| {
                            state.player(t).alive
                                && pindian::can_compare_pindian(state, ctx.catalog, t)
                        })
                    } else {
                        u.targets.contains(&owner)
                            && state.player(u.player).alive
                            && pindian::can_compare_pindian(state, ctx.catalog, u.player)
                    }
                }
                _ => false,
            }
        }
        "xinfu_qinguo" => match event {
            EventData::Equipped { player, card, .. } => {
                let fid = frame.map_or(0, |f| f.id);
                *player == owner
                    && fid > 0
                    && flag(state, owner, &format!("l11_vm_b_equip_use:{fid}")) > 0
                    && ctx.catalog.card_kind(*card).equip_slot().is_some()
                    && !asks::valid_virtual_sha_targets(
                        state,
                        ctx.catalog,
                        owner,
                        CardKind::Sha,
                        crate::event::DamageNature::Normal,
                        true,
                        false,
                    )
                    .is_empty()
            }
            EventData::CardsLost { player, origins, .. } => {
                let is_swap1 = frame
                    .and_then(|f| f.parent)
                    .is_some_and(|pid| flag(state, owner, &format!("eq_swap1:{pid}")) > 0);
                !is_swap1
                    && !has_named_skill(state, ctx.catalog, owner, "xinfu_qinguo_recover")
                    && {
                        let pl = state.player(owner);
                        *player == owner
                            && origins.contains(&LostOrigin::Equip)
                            && !pl.equips.is_empty()
                            && pl.equips.len() as i32 == pl.hp
                            && pl.hp < pl.max_hp as i32
                    }
            }
            _ => false,
        },
        "xinfu_qinguo_recover" => {
            let is_swap1 = frame
                .and_then(|f| f.parent)
                .is_some_and(|pid| flag(state, owner, &format!("eq_swap1:{pid}")) > 0);
            let pl = state.player(owner);
            !is_swap1
                && !pl.equips.is_empty()
                && pl.equips.len() as i32 == pl.hp
                && pl.hp < pl.max_hp as i32
                && matches!(
                    event,
                    EventData::CardsLost { player, origins, .. }
                        if *player == owner && origins.contains(&LostOrigin::Equip)
                )
        }
        _ => false,
    }
}

pub(in crate::content::host) fn machine_l11_vm_fix_b(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut target = None;
    let mut cards = Vec::new();
    let mut targets = Vec::new();
    match (skill.name.as_str(), event) {
        ("qingxi", EventData::Damage(d)) => target = Some(d.target),
        ("rebeige", EventData::Damage(d)) => {
            target = Some(d.target);
            if let Some(s) = d.source {
                targets.push(s);
            }
            cards.push(d.amount.max(1) as u32);
        }
        ("rejianxiong", EventData::Damage(d)) => {
            if let Some(c) = d.card {
                cards.push(c);
            }
        }
        ("rejieming", EventData::Damage(d)) => cards.push(d.amount.max(1) as u32),
        ("reweimu", EventData::Damage(d)) => cards.push(d.amount.max(0) as u32),
        ("yiyong", EventData::Damage(d)) => {
            target = d.source;
            if let Some(c) = d.card {
                cards.push(c);
            }
        }
        ("reshejian", EventData::CardUsed(u)) => target = Some(u.player),
        ("sphuangen", EventData::CardUsed(u)) => targets = u.targets.clone(),
        ("spdengli", EventData::CardUsed(u)) => {
            // `cards[0]` 在 handle 时重算或按事件写入；由于无 state，此处保存目标列表（由 handle 按需摸牌）
            if u.player == owner {
                targets = u.targets.clone();
                cards.push(1);
            } else {
                cards.push(0);
            }
        }
        ("twshengxi", EventData::Phase { phase: Phase::End, .. }) => cards.push(1),
        ("repolu", EventData::Death { .. }) => cards.push(1),
        ("zhengjian", EventData::TurnBegin { .. }) => cards.push(1),
        ("zhuandui", EventData::CardUsed(u)) => {
            if u.player == owner {
                cards.push(1);
                targets = u.targets.clone();
            } else {
                cards.push(0);
                targets.push(u.player);
            }
        }
        _ => {}
    }
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L11VmFixB,
        owner,
        target,
        card: None,
        cards,
        targets,
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn handle_l11_vm_fix_b(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act
        && m.card.is_none()
        && (m.def_key == "reluanwu" || m.def_key == "shenfen")
    {
        let flag_key = if m.def_key == "reluanwu" {
            "l11_vm_b:reluanwu_used"
        } else {
            "l11_vm_b:shenfen_used"
        };
        set_flag(ctx.core, owner, flag_key, 1);
    }
    if m.def_key == "sphuangen" && m.stage == HostStage::Act && m.card.is_none() {
        let fid = ctx.current_event_frame_id();
        set_flag(ctx.core, owner, "l11_vm_b:sphuangen_fid", fid);
    }
    if m.def_key == "spdengli" && m.stage == HostStage::Act && m.card.is_none() {
        let ohp = ctx.core.player(owner).hp;
        let cnt = if m.cards.first().copied() == Some(1) {
            m.targets
                .iter()
                .filter(|&&t| ctx.core.player(t).alive && ctx.core.player(t).hp == ohp)
                .count()
                .max(1) as u32
        } else {
            1
        };
        m.cards = vec![cnt];
    }
    match m.def_key.as_str() {
        "quhu" => active::handle_quhu(m, ctx),
        "refanjian" => active::handle_refanjian(m, ctx),
        "reluanwu" => active::handle_reluanwu(m, ctx),
        "rexianzhen" => active::handle_rexianzhen(m, ctx),
        "sbyangwei" => active::handle_sbyangwei(m, ctx),
        "shanxie" => active::handle_shanxie(m, ctx),
        "shenfen" => active::handle_shenfen(m, ctx),
        "tianyi" => active::handle_tianyi(m, ctx),
        "tiaoxin" => active::handle_tiaoxin(m, ctx),
        "qingxi" => combat_triggers::handle_qingxi(m, ctx),
        "rebeige" => combat_triggers::handle_rebeige(m, ctx),
        "rejianxiong" => combat_triggers::handle_rejianxiong(m, ctx),
        "rejieming" => combat_triggers::handle_rejieming(m, ctx),
        "releiji" => combat_triggers::handle_releiji(m, ctx),
        "reshejian" => combat_triggers::handle_reshejian(m, ctx),
        "reweimu" => combat_triggers::handle_reweimu(m, ctx),
        "spchijie" => combat_triggers::handle_spchijie(m, ctx),
        "spdengli" => combat_triggers::handle_spdengli(m, ctx),
        "sphuangen" => combat_triggers::handle_sphuangen(m, ctx),
        "xinliegong" => combat_triggers::handle_xinliegong(m, ctx),
        "yiyong" => combat_triggers::handle_yiyong(m, ctx),
        "zhuandui" => combat_triggers::handle_zhuandui(m, ctx),
        "qiangzhi" => phase_triggers::handle_qiangzhi(m, ctx),
        "qiangzhi_draw" => phase_triggers::handle_qiangzhi_draw(m, ctx),
        "rejiangchi" => phase_triggers::handle_rejiangchi(m, ctx),
        "repolu" => phase_triggers::handle_repolu(m, ctx),
        "rezhiyi" => phase_triggers::handle_rezhiyi(m, ctx),
        "sbyangwei_counter" => phase_triggers::handle_sbyangwei_counter(m, ctx),
        "twshengxi" => phase_triggers::handle_twshengxi(m, ctx),
        "xinfu_qinguo" => {
            if m.stage == HostStage::Act
                && m.card.is_none()
                && ctx.core.player(owner).equips.len() as i32 == ctx.core.player(owner).hp
                && ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32
                && matches!(ctx.res.stack.last().map(|f| &f.data), Some(EventData::CardsLost { .. }))
            {
                phase_triggers::handle_xinfu_qinguo_recover(m, ctx)
            } else if m.card == Some(asks::ASK_XINFU_QINGUO_RECOVER) {
                phase_triggers::handle_xinfu_qinguo_recover(m, ctx)
            } else {
                phase_triggers::handle_xinfu_qinguo(m, ctx)
            }
        }
        "xinfu_qinguo_recover" => phase_triggers::handle_xinfu_qinguo_recover(m, ctx),
        "yinghun" => phase_triggers::handle_yinghun(m, ctx),
        "zaiqi" => phase_triggers::handle_zaiqi(m, ctx),
        "zaoxian" => phase_triggers::handle_zaoxian(m, ctx),
        "zhaohuo" => phase_triggers::handle_zhaohuo(m, ctx),
        "zhengjian" => phase_triggers::handle_zhengjian(m, ctx),
        "zhengjian_mark" => phase_triggers::handle_zhengjian_mark(m, ctx),
        "zhiyi" => phase_triggers::handle_zhiyi(m, ctx),
        "tiaojiyanmei__loop" => phase_triggers::handle_tiaojiyanmei_loop(m, ctx),
        "tiaojiyanmei__one" => phase_triggers::handle_tiaojiyanmei_one(m, ctx),
        _ => Flow::Done,
    }
}

pub(in crate::content::host) fn ask_l11_vm_fix_b(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    asks::ask_l11_vm_fix_b_inner(stage, owner, target, targets, card, cards, core, catalog)
}

pub(in crate::content::host) fn on_card_use_after_l11_vm_fix_b(
    ctx: &mut Ctx<'_>,
    _fid: crate::ids::EventId,
    u: &CardUseData,
) {
    let owner = u.player;
    if !ctx.core.player(owner).alive {
        return;
    }
    let ts = turn_stamp(ctx.core);
    if !u.cancelled
        && ctx.core.turn.current == owner
        && ctx.catalog.skill_by_name("twshengxi").is_some()
    {
        set_flag(ctx.core, owner, "l11_vm_b:tw_used_ts", ts);
    }
    let kind = u.effective_kind(ctx.catalog);
    if kind.is_basic_kind() {
        if let Some(tag) = phase_triggers::record_basic_turn_use_or_resp(ctx.core, ctx.catalog, owner, kind) {
            phase_triggers::push_vm_b_park(
                ctx,
                "zhiyi",
                owner,
                None,
                vec![u32::from(tag)],
                Vec::new(),
            );
        }
    }
    if !u.cancelled
        && kind.equip_slot().is_some()
        && is_unblocked_l11_owner(ctx.core, ctx.catalog, owner)
        && has_named_skill(ctx.core, ctx.catalog, owner, "xinfu_qinguo")
        && !asks::valid_virtual_sha_targets(
            ctx.core,
            ctx.catalog,
            owner,
            crate::catalog::CardKind::Sha,
            crate::event::DamageNature::Normal,
            true,
            false,
        )
        .is_empty()
    {
        phase_triggers::push_vm_b_park(
            ctx,
            "xinfu_qinguo",
            owner,
            None,
            Vec::new(),
            Vec::new(),
        );
    }
}

pub(in crate::content::host) fn on_equip_gained_l11_vm_fix_b(ctx: &mut Ctx<'_>, player: PlayerId) {
    if !ctx.core.player(player).alive {
        return;
    }
    let fid = (!ctx.res.stack.is_empty()).then(|| ctx.current_event_frame_id());
    if let Some(f) = fid {
        if ctx.core.turn.current == player
            && ctx.catalog.skill_by_name("twshengxi").is_some()
            && flag(ctx.core, player, &format!("l11_vm_b_equip_use:{f}")) > 0
        {
            let ts = turn_stamp(ctx.core);
            set_flag(ctx.core, player, "l11_vm_b:tw_used_ts", ts);
        }
    }
    if !(has_named_skill(ctx.core, ctx.catalog, player, "xinfu_qinguo")
        || has_named_skill(ctx.core, ctx.catalog, player, "xinfu_qinguo_recover"))
        || !is_unblocked_l11_owner(ctx.core, ctx.catalog, player)
    {
        return;
    }
    if let Some(f) = fid {
        if flag(ctx.core, player, &format!("eq_swap1:{f}")) > 0 {
            return;
        }
    }
    let pl = ctx.core.player(player);
    if !pl.equips.is_empty() && pl.equips.len() as i32 == pl.hp && pl.hp < pl.max_hp as i32 {
        phase_triggers::push_vm_b_park(
            ctx,
            "xinfu_qinguo_recover",
            player,
            None,
            Vec::new(),
            Vec::new(),
        );
    }
}

pub(in crate::content::host) fn on_host_event_l11_vm_fix_b(ctx: &mut Ctx<'_>, event: &EventData) {
    match event {
        EventData::Damage(d) if !d.prevented && d.amount > 0 => {
            if let Some(src) = d.source {
                if ctx.core.turn.current == src && ctx.catalog.skill_by_name("twshengxi").is_some()
                {
                    let ts = turn_stamp(ctx.core);
                    set_flag(ctx.core, src, "l11_vm_b:tw_dmg_ts", ts);
                }
            }
        }
        EventData::CardResponded { player, card } => {
            let p = *player;
            if ctx.core.player(p).alive {
                let kind = ctx.catalog.resolved_card_kind(ctx.core, *card);
                if kind.is_basic_kind() {
                    if let Some(tag) =
                        phase_triggers::record_basic_turn_use_or_resp(ctx.core, ctx.catalog, p, kind)
                    {
                        phase_triggers::push_vm_b_park(
                            ctx,
                            "zhiyi",
                            p,
                            None,
                            vec![u32::from(tag)],
                            Vec::new(),
                        );
                    }
                }
            }
        }
        _ => {}
    }
}

pub(in crate::content::host) fn on_player_died_l11_vm_fix_b(
    ctx: &mut Ctx<'_>,
    dead: PlayerId,
    killer: Option<PlayerId>,
) {
    if ctx.core.outcome.is_some() {
        return;
    }
    let Some(k) = killer else {
        return;
    };
    if k == dead || !ctx.core.player(k).alive {
        return;
    }
    if has_named_skill(ctx.core, ctx.catalog, k, "repolu")
        && is_unblocked_l11_owner(ctx.core, ctx.catalog, k)
    {
        phase_triggers::push_vm_b_park(ctx, "repolu", k, None, Vec::new(), Vec::new());
    }
}

pub(in crate::content::host) fn on_phase_use_end_l11_vm_fix_b(ctx: &mut Ctx<'_>, player: PlayerId) {
    set_flag(ctx.core, player, "l11_vm_b:qiangzhi_cat", 0);
    for sk in ["jiangchi2", "rejiangchi3", "qiangzhi_draw"] {
        remove_named_skill(ctx, player, sk);
    }
}

pub(in crate::content::host) fn on_turn_phase_after_l11_vm_fix_b(ctx: &mut Ctx<'_>, player: PlayerId) {
    set_flag(ctx.core, player, "bound_target", 0);
    set_flag(ctx.core, player, "l11_vm_b:zhiyi_ts", 0);
    set_flag(ctx.core, player, "l11_vm_b:rezhiyi_ts", 0);
    set_flag(ctx.core, player, "l11_vm_b:rezhiyi_mask", 0);
    set_flag(ctx.core, player, "l11_vm_b:tw_used_ts", 0);
    set_flag(ctx.core, player, "l11_vm_b:tw_dmg_ts", 0);
    set_flag(ctx.core, player, "l11_vm_b:spchijie_used_ts", 0);
    set_flag(ctx.core, player, "l11_vm_b:reshejian_ts", 0);
    set_flag(ctx.core, player, "l11_vm_b:reshejian_cnt", 0);
    set_flag(ctx.core, player, "l11_vm_b:sphuangen_fid", 0);
    for sk in ["rexianzhen2", "rexianzhen3", "rexianzhen4", "sbyangwei_effect", "tianyi2", "tianyi3"] {
        remove_named_skill(ctx, player, sk);
    }
}

pub(in crate::content::host) fn on_turn_begin_l11_vm_fix_b(ctx: &mut Ctx<'_>, player: PlayerId) {
    let _ = (ctx, player);
}

pub(in crate::content::host) fn on_skill_removed_l11_vm_fix_b(
    core: &mut GameStateCore,
    owner: PlayerId,
    skill_name: &str,
) {
    match skill_name {
        "zhengjian_mark" => set_flag(core, owner, "l11_vm_b:zhengjian_cnt", 0),
        "qiangzhi_draw" => set_flag(core, owner, "l11_vm_b:qiangzhi_cat", 0),
        "sbyangwei_counter" => set_flag(core, owner, "l11_vm_b:sbyangwei_cd_armed", 0),
        "rexianzhen2" => set_flag(core, owner, "bound_target", 0),
        "repolu" => set_flag(core, owner, "l11_vm_b:repolu_cnt", 0),
        _ => {}
    }
}
