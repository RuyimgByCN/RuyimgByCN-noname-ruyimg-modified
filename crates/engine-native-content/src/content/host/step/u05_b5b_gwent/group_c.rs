//! U05-5b Batch C（16 技能 + 2 子技）：
//! `lingji`, `gwjinyan`, `gwshenyu`, `junchi`, `hupeng`, `hupeng2`, `hupeng3`, `hupeng4`,
//! `hunmo`, `hunmo2`, `shuijian`, `yunhuo`, `yinzhang`, `gwtianbian`, `gwxiaoshou`, `kuanglie`,
//! `kuanglie2`, `gwjiquan`。

use super::group_c_asks::*;
use super::*;
use crate::catalog::TriggerMeta;
use crate::event::{EventKind, Timing};

fn meta(event: EventKind, extra_events: Vec<EventKind>) -> TriggerMeta {
    TriggerMeta {
        event,
        timing: Timing::Normal,
        priority: 0,
        order_selectable: false,
        optional: false,
        extra_events,
    }
}

pub(super) fn trigger_meta_group_c(name: &str) -> Option<TriggerMeta> {
    match name {
        "gwjinyan" => Some(meta(EventKind::DamageBefore, vec![EventKind::PhaseBegin])),
        "gwshenyu" | "shuijian" | "yunhuo" | "gwtianbian" => {
            Some(meta(EventKind::PhaseBegin, Vec::new()))
        }
        "hupeng3" => Some(meta(EventKind::TurnPhaseEnd, Vec::new())),
        "hupeng4" | "kuanglie" => Some(meta(EventKind::CardUsed, Vec::new())),
        _ => None,
    }
}

pub(super) fn filter_group_c(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    if !state.player(owner).alive {
        return false;
    }
    match skill.name.as_str() {
        "gwjinyan" => match event {
            EventData::Damage(d) => {
                d.target == owner
                    && d.settlement.committed_damage.is_none()
                    && !d.prevented
                    && d.amount > 0
                    && state.turn.round % 3 != 0
            }
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner && state.turn.round % 3 == 0,
            _ => false,
        },
        "gwshenyu" => {
            let EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } = *event
            else {
                return false;
            };
            player == owner
                && (!shenyu_discard_spells(state, ctx.catalog).is_empty()
                    || state
                        .alive_players()
                        .any(|p| state.player(p).hp < state.player(p).max_hp as i32))
        }
        "hupeng3" => matches!(
            *event,
            EventData::TurnEndWindow {
                player,
                after: false
            } if player == owner
                && state
                    .player(owner)
                    .hand
                    .iter()
                    .any(|&c| ctx.catalog.cards[c as usize].name == "gw_dudayuanshuai2")
        ),
        "hupeng4" => {
            let EventData::CardUsed(u) = event else {
                return false;
            };
            u.player != owner
                && !u.cancelled
                && u.targets.contains(&owner)
                && !ctx.catalog.blocks_all_cards(state, owner)
                && state
                    .player(owner)
                    .hand
                    .iter()
                    .any(|&c| ctx.catalog.cards[c as usize].name == "gw_dudayuanshuai1")
        }
        "shuijian" => {
            matches!(
                *event,
                EventData::Phase {
                    player,
                    phase: Phase::Prepare,
                    begin: true
                } if player == owner && !discardable_hand(state, ctx.catalog, owner).is_empty()
            )
        }
        "yunhuo" => matches!(
            *event,
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true
            } if player == owner
                && state.turn.round % 4 == 0
                && flag(state, owner, "gw_yunhuo_fired_round") != state.turn.round
        ),
        "gwtianbian" => matches!(
            *event,
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true
            } if player == owner && state.alive_players().any(|p| p != owner)
        ),
        "kuanglie" => {
            let EventData::CardUsed(u) = event else {
                return false;
            };
            if u.player != owner || u.cancelled {
                return false;
            }
            let black = u.card.is_some_and(|cid| !is_red_card(ctx.catalog, cid));
            black
                && u.targets
                    .iter()
                    .any(|&t| t != owner && state.player(t).alive && !hand_and_equips(state, t).is_empty())
        }
        _ => false,
    }
}

pub(super) fn machine_group_c(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut m = HostMachine {
        def_key: skill.name.clone(),
        behavior: skill.behavior,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    };
    match (skill.name.as_str(), event) {
        ("gwjinyan", EventData::Damage(_)) => m.card = Some(1),
        ("gwjinyan", EventData::Phase { .. }) => m.card = Some(2),
        ("kuanglie", EventData::CardUsed(u)) => {
            m.targets = u
                .targets
                .iter()
                .copied()
                .filter(|&t| t != owner)
                .collect();
        }
        _ => {}
    }
    m
}

pub(super) fn activate_group_c(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<HostMachine> {
    let state = ctx.state;
    let ok = match skill.name.as_str() {
        "lingji" => !state.deck.is_empty() || !state.discard.is_empty(),
        "hupeng" | "yinzhang" => !discardable_hand_and_equips(state, ctx.catalog, owner).is_empty(),
        "gwxiaoshou" => {
            flag(state, owner, "gw_use2_cnt:gwxiaoshou") < 2
                && !discardable_hand_and_equips(state, ctx.catalog, owner).is_empty()
        }
        "hunmo" => {
            let small = state.player(owner).hand.len() < 2;
            (small
                || state.player(owner).hand.iter().any(|&c| {
                    !crate::kernel::is_gw_dudayuanshuai2_locked(state, ctx.catalog, owner, c)
                }))
                && !hunmo_targets(state, ctx.catalog, owner).is_empty()
        }
        "gwjiquan" => !jiquan_targets(state, owner).is_empty(),
        _ => false,
    };
    ok.then(|| HostMachine {
        def_key: skill.name.clone(),
        behavior: skill.behavior,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    })
}

pub(super) fn try_spawn_junchi_sha_end(
    ctx: &mut Ctx<'_>,
    sha_frame_id: EventId,
    source: PlayerId,
    target: PlayerId,
) -> bool {
    if !ctx.core.player(target).alive {
        return false;
    }
    let Some(def) = ctx.catalog.skill_by_name("junchi") else {
        return false;
    };
    let turn_stamp = ctx
        .core
        .turn
        .round
        .saturating_mul(16)
        .saturating_add(u32::from(ctx.core.turn.current))
        .saturating_add(1);
    let n = ctx.core.players.len() as PlayerId;
    let cur = ctx.core.turn.current;
    for i in 0..n {
        let owner = (cur + i) % n;
        if owner == source || owner == target || !ctx.core.player(owner).alive {
            continue;
        }
        if !ctx.core.has_skill(owner, def) {
            continue;
        }
        if flag(ctx.core, owner, &format!("gw_junchi_sha_done:{sha_frame_id}")) > 0
            || flag(ctx.core, owner, "gw_junchi_used_turn") == turn_stamp
        {
            continue;
        }
        if junchi_playable_cards(ctx.core, ctx.catalog, owner, target).is_empty() {
            continue;
        }
        set_flag(
            ctx.core,
            owner,
            &format!("gw_junchi_sha_done:{sha_frame_id}"),
            1,
        );
        ctx.res.parks.push(crate::resolution::Park {
            resume_depth: ctx.res.stack.len(),
            slot: crate::resolution::Slot::Trigger { frame: sha_frame_id },
            awaiting_answer: false,
            machine: Box::new(HostMachine {
                def_key: "junchi_sha".to_string(),
                behavior: HostBehavior::U05GwentCharBatchC,
                owner,
                target: Some(target),
                card: None,
                cards: Vec::new(),
                targets: Vec::new(),
                stage: HostStage::Act,
            }),
        });
        return true;
    }
    false
}

