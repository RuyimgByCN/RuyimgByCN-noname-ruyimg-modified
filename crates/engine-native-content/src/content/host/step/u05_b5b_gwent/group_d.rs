//! U05-5b Batch D（13 技能 + 4 子技）：
//! `nuhou`, `shewu`, `shewu_dist`, `gwzhanjiang`, `gwzhanjiang2`, `gwzhanjiang3`, `gwchuanxin`,
//! `fengjian`, `huandie`, `huandie_discard`, `xuezhou`, `xuezhou_hp`, `xuezhou_hp2`, `fayin`,
//! `gwbaquan`, `huihun`, `lanquan`。

use super::group_d_asks::*;
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

pub(super) fn trigger_meta_group_d(name: &str) -> Option<TriggerMeta> {
    match name {
        "nuhou" => Some(meta(EventKind::Damage, Vec::new())),
        "gwzhanjiang" | "gwzhanjiang2" | "huandie" => Some(meta(EventKind::PhaseBegin, Vec::new())),
        "fengjian" => Some(meta(EventKind::CardUsed, Vec::new())),
        "huandie_discard" => Some(meta(EventKind::TurnPhaseEnd, Vec::new())),
        "xuezhou" => Some(meta(EventKind::PhaseBegin, vec![EventKind::Damage])),
        "xuezhou_hp" => Some(meta(EventKind::Damage, Vec::new())),
        "huihun" => Some(meta(
            EventKind::CardUsed,
            vec![EventKind::TurnPhaseEnd, EventKind::TurnPhaseAfter],
        )),
        _ => None,
    }
}

pub(super) fn filter_group_d(
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
        "nuhou" => matches!(
            event,
            EventData::Damage(d)
                if d.settlement.committed_damage.is_some()
                    && !d.prevented
                    && d.amount > 0
                    && d.target == owner
                    && !discardable_hand_and_equips(state, ctx.catalog, owner).is_empty()
        ),
        "gwzhanjiang" => matches!(
            *event,
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true
            } if player != owner
                && !has_named_skill(state, ctx.catalog, owner, "gwzhanjiang2")
                && !discardable_hand_and_equips(state, ctx.catalog, owner).is_empty()
        ),
        "gwzhanjiang2" => matches!(
            *event,
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true
            } if player == owner
        ),
        "fengjian" => {
            let EventData::CardUsed(u) = event else {
                return false;
            };
            if u.player != owner || u.cancelled {
                return false;
            }
            is_upstream_trick_use(ctx.catalog, u)
                && state.alive_players().any(|p| {
                    p != owner
                        && !u.targets.contains(&p)
                        && !crate::kernel::sha_user_or_target_blocked(state, ctx.catalog, owner, p)
                })
        }
        "huandie" => matches!(
            *event,
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true
            } if player == owner
        ),
        "huandie_discard" => matches!(
            *event,
            EventData::TurnEndWindow {
                player,
                after: false
            } if player == owner
                && state
                    .alive_players()
                    .any(|p| state.player(p).hand.len() > state.player(p).hp.max(0) as usize)
        ),
        "xuezhou" | "xuezhou_hp" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } if skill.name == "xuezhou" => *player == owner,
            EventData::Damage(d) => {
                if d.settlement.committed_damage.is_none() || d.prevented || d.amount <= 0 {
                    return false;
                }
                let xowner = flag(state, 0, "gw_xuezhou_owner");
                if xowner != u32::from(owner) + 1 {
                    return false;
                }
                if has_named_skill(state, ctx.catalog, state.turn.current, "xuezhou_hp2") {
                    return false;
                }
                let mode = flag(state, owner, "gw_xuezhou_mode");
                match mode {
                    1 => d.target != owner && state.player(d.target).alive,
                    2 => d
                        .source
                        .is_some_and(|s| s != owner && state.player(s).alive),
                    _ => false,
                }
            }
            _ => false,
        },
        "huihun" => match event {
            EventData::CardUsed(u) => {
                u.player == owner && state.turn.current == owner && !u.cancelled && u.card.is_some()
            }
            EventData::TurnEndWindow {
                player,
                after: false,
            } => {
                *player == owner && {
                    let n = flag(state, owner, "gw_huihun_n");
                    (0..n).any(|i| {
                        let v = flag(state, owner, &format!("gw_huihun_c:{i}"));
                        v > 0 && state.discard.contains(&(v - 1))
                    })
                }
            }
            EventData::TurnEndWindow {
                player,
                after: true,
            } => *player == owner,
            _ => false,
        },
        _ => false,
    }
}

pub(super) fn machine_group_d(
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
        ("gwzhanjiang", EventData::Phase { player, .. }) => m.target = Some(*player),
        ("fengjian", EventData::CardUsed(u)) => m.targets = u.targets.clone(),
        ("xuezhou" | "xuezhou_hp", EventData::Damage(d)) => {
            m.card = Some(1);
            m.target = Some(d.target);
            if let Some(src) = d.source {
                m.targets = vec![src];
            }
        }
        ("huihun", EventData::CardUsed(u)) => {
            m.card = u.card;
            m.stage = HostStage::ComparePlayerPick;
        }
        ("huihun", EventData::TurnEndWindow { after: false, .. }) => m.stage = HostStage::DrawSettle,
        ("huihun", EventData::TurnEndWindow { after: true, .. }) => m.stage = HostStage::HuanrenPick,
        _ => {}
    }
    m
}

pub(super) fn activate_group_d(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<HostMachine> {
    let state = ctx.state;
    let ok = match skill.name.as_str() {
        "shewu" => !discardable_hand_and_equips(state, ctx.catalog, owner).is_empty(),
        "gwbaquan" => state
            .alive_players()
            .any(|p| p != owner && !state.player(p).hand.is_empty()),
        "lanquan" => {
            (!state.deck.is_empty()
                && !lanquan_playable_options(state, ctx.catalog, owner).is_empty())
                || (state.deck.len() < 6 && !state.discard.is_empty())
        }
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

pub(super) fn try_spawn_fayin_sha_begin(
    ctx: &mut Ctx<'_>,
    sha_frame_id: EventId,
    source: PlayerId,
    target: PlayerId,
) -> bool {
    if !has_named_skill(ctx.core, ctx.catalog, source, "fayin")
        || !ctx.core.player(target).alive
        || discardable_hand_and_equips(ctx.core, ctx.catalog, source).is_empty()
        || flag(ctx.core, source, &format!("sha_fayin_done:{sha_frame_id}")) > 0
    {
        return false;
    }
    set_flag(ctx.core, source, &format!("sha_fayin_done:{sha_frame_id}"), 1);
    ctx.res.parks.push(crate::resolution::Park {
        resume_depth: ctx.res.stack.len(),
        slot: crate::resolution::Slot::Trigger { frame: sha_frame_id },
        awaiting_answer: false,
        machine: Box::new(HostMachine {
            def_key: "fayin".to_string(),
            behavior: HostBehavior::U05GwentCharBatchD,
            owner: source,
            target: Some(target),
            card: None,
            cards: Vec::new(),
            targets: Vec::new(),
            stage: HostStage::Act,
        }),
    });
    true
}

pub(super) fn try_spawn_chuanxin_sha_end(
    ctx: &mut Ctx<'_>,
    sha_frame_id: EventId,
    source: PlayerId,
    target: PlayerId,
) -> bool {
    if !has_named_skill(ctx.core, ctx.catalog, source, "gwchuanxin")
        || !ctx.core.player(target).alive
        || (ctx.core.deck.is_empty() && ctx.core.discard.is_empty())
        || flag(
            ctx.core,
            source,
            &format!("sha_chuanxin_done:{sha_frame_id}:{target}"),
        ) > 0
    {
        return false;
    }
    set_flag(
        ctx.core,
        source,
        &format!("sha_chuanxin_done:{sha_frame_id}:{target}"),
        1,
    );
    ctx.res.parks.push(crate::resolution::Park {
        resume_depth: ctx.res.stack.len(),
        slot: crate::resolution::Slot::Trigger { frame: sha_frame_id },
        awaiting_answer: false,
        machine: Box::new(HostMachine {
            def_key: "gwchuanxin".to_string(),
            behavior: HostBehavior::U05GwentCharBatchD,
            owner: source,
            target: Some(target),
            card: None,
            cards: Vec::new(),
            targets: Vec::new(),
            stage: HostStage::Act,
        }),
    });
    true
}
