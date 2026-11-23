//! U05-5b Batch B（19 技能）：
//! `shuangxi`, `yangfan`, `gwfengshi`, `gwchenshui`, `gwliedi`, `julian`, `gwfusheng`, `gwqinwu`,
//! `gwqinwu2`, `huanshu`, `huanshu2`, `huanshu3`, `gwjieyin`, `zhengjun`, `gwxuezhan`, `bolang`,
//! `gwjushi`, `gwjushi2`, `gwfengchi`。

use super::group_b_asks::*;
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

pub(super) fn trigger_meta_group_b(name: &str) -> Option<TriggerMeta> {
    match name {
        "shuangxi" => Some(meta(EventKind::Damage, Vec::new())),
        "yangfan" | "gwqinwu" => Some(meta(EventKind::CardUsed, Vec::new())),
        "gwfengshi" | "huanshu" | "gwxuezhan" => Some(meta(
            if name == "gwxuezhan" {
                EventKind::PhaseBegin
            } else {
                EventKind::TurnPhaseEnd
            },
            Vec::new(),
        )),
        "gwchenshui" => Some(meta(EventKind::DamageBefore, vec![EventKind::TurnPhaseEnd])),
        "gwliedi" => Some(meta(
            EventKind::DamageBefore,
            vec![EventKind::Damage, EventKind::TurnPhaseEnd],
        )),
        "julian" | "gwfengchi" => Some(meta(EventKind::PhaseBegin, Vec::new())),
        "huanshu2" => Some(meta(EventKind::CardUsed, vec![EventKind::PhaseBegin])),
        "huanshu3" | "gwjieyin" => Some(meta(EventKind::PhaseBegin, Vec::new())),
        "zhengjun" => Some(meta(
            EventKind::CardUsed,
            vec![EventKind::CardResponded, EventKind::TurnPhaseEnd],
        )),
        "bolang" => Some(meta(EventKind::PhaseBegin, vec![EventKind::Damage])),
        "bolang_gain" => Some(meta(EventKind::Damage, Vec::new())),
        "gwjushi2" => Some(meta(EventKind::PhaseBegin, vec![EventKind::DamageEnd])),
        _ => None,
    }
}

pub(super) fn filter_group_b(
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
        "shuangxi" => matches!(
            event,
            EventData::Damage(d)
                if d.settlement.committed_damage.is_some()
                    && !d.prevented
                    && d.amount > 0
                    && d.source == Some(owner)
        ),
        "yangfan" => {
            let EventData::CardUsed(u) = event else {
                return false;
            };
            if u.player != owner || u.cancelled {
                return false;
            }
            let Some(uc) = u.card else {
                return false;
            };
            let eff = u.use_as.unwrap_or_else(|| ctx.catalog.card_kind(uc));
            if eff.equip_slot().is_some() {
                return false;
            }
            let red = is_red_card(ctx.catalog, uc);
            state.player(owner).hand.iter().any(|&h| {
                h != uc
                    && is_red_card(ctx.catalog, h) == red
                    && !crate::kernel::is_gw_dudayuanshuai2_locked(state, ctx.catalog, owner, h)
            })
        }
        "gwfengshi" => matches!(
            *event,
            EventData::TurnEndWindow {
                player,
                after: false
            } if player == owner
        ),
        "gwchenshui" => match event {
            EventData::Damage(d) => {
                d.settlement.committed_damage.is_none()
                    && !d.prevented
                    && d.amount > 0
                    && (d.target == owner || d.source == Some(owner))
            }
            EventData::TurnEndWindow {
                player,
                after: false,
            } => *player == owner && flag(state, owner, "gwchenshui_count") >= 3,
            _ => false,
        },
        "gwliedi" => match event {
            EventData::Damage(d) => {
                if d.source != Some(owner) || d.target == owner || d.prevented || d.amount <= 0 {
                    return false;
                }
                if d.settlement.committed_damage.is_none() {
                    state.computed_distance(ctx.catalog, owner, d.target) >= 2
                } else {
                    true
                }
            }
            EventData::TurnEndWindow {
                player,
                after: false,
            } => *player == owner,
            _ => false,
        },
        "julian" => {
            let EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } = *event
            else {
                return false;
            };
            if player != owner {
                return false;
            }
            let my_h = state.player(owner).hand.len();
            state
                .alive_players()
                .filter(|&p| p != owner)
                .map(|p| state.player(p).hand.len())
                .max()
                .is_some_and(|mx| mx > my_h)
        }
        "gwqinwu" => {
            let EventData::CardUsed(u) = event else {
                return false;
            };
            if u.player != owner || u.cancelled {
                return false;
            }
            let eff = u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog));
            if !eff.is_basic_kind() {
                return false;
            }
            let stamp = state.turn.round.saturating_mul(16) + u32::from(state.turn.current) + 1;
            flag(state, owner, "gw_qinwu_used_turn") != stamp
        }
        "huanshu" => matches!(
            *event,
            EventData::TurnEndWindow {
                player,
                after: false
            } if player == owner
                && !state.player(owner).hand.is_empty()
                && !has_named_skill(state, ctx.catalog, owner, "huanshu2")
        ),
        "huanshu2" => match event {
            EventData::CardUsed(u) => {
                if u.player == owner || u.cancelled || state.is_friend_of(ctx.catalog, owner, u.player)
                {
                    return false;
                }
                let stored = flag(state, owner, "gw_huanshu2_card");
                if stored == 0 {
                    return false;
                }
                let stored_c = stored - 1;
                if !is_upstream_trick_use(ctx.catalog, u) {
                    return false;
                }
                let stored_red = is_red_card(ctx.catalog, stored_c);
                u.card.is_some_and(|uc| is_red_card(ctx.catalog, uc) == stored_red)
            }
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner && flag(state, owner, "gw_huanshu2_card") > 0,
            _ => false,
        },
        "huanshu3" => matches!(
            *event,
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true
            } if player == owner && flag(state, owner, "gw_huanshu2_card") > 0
        ),
        "gwjieyin" => matches!(
            *event,
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true
            } if player == owner
        ),
        "zhengjun" => match event {
            EventData::CardUsed(u) => u.player == owner && u.card.is_some(),
            EventData::CardResponded { player, .. } => *player == owner,
            EventData::TurnEndWindow {
                player,
                after: false,
            } => *player == owner && flag(state, owner, "gw_zj_cnt") >= 1,
            _ => false,
        },
        "gwxuezhan" => {
            let EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } = *event
            else {
                return false;
            };
            if player != owner {
                return false;
            }
            let my_h = state.player(owner).hand.len();
            state
                .alive_players()
                .all(|p| state.player(p).hand.len() >= my_h)
        }
        "bolang" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner && (!state.deck.is_empty() || !state.discard.is_empty()),
            EventData::Damage(d) => {
                d.settlement.committed_damage.is_some()
                    && !d.prevented
                    && d.amount > 0
                    && d.source == Some(owner)
                    && {
                        let stamp =
                            state.turn.round.saturating_mul(16) + u32::from(state.turn.current) + 1;
                        flag(state, owner, "gw_bolang_gain_turn") != stamp
                    }
                    && !bolang_reclaimable(state, owner).is_empty()
            }
            _ => false,
        },
        "bolang_gain" => {
            matches!(
                event,
                EventData::Damage(d)
                    if d.settlement.committed_damage.is_some()
                        && !d.prevented
                        && d.amount > 0
                        && d.source == Some(owner)
            ) && !bolang_reclaimable(state, owner).is_empty()
        }
        "gwjushi2" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner && flag(state, owner, "gw_jushi2_card") > 0,
            EventData::Damage(d) => {
                d.settlement.committed_damage.is_some()
                    && !d.prevented
                    && d.amount > 0
                    && d.target == owner
                    && flag(state, owner, "gw_jushi2_card") > 0
            }
            _ => false,
        },
        "gwfengchi" => {
            matches!(
                *event,
                EventData::Phase {
                    player,
                    phase: Phase::Play,
                    begin: true
                } if player == owner
            ) && !collect_fengchi_pool(ctx.catalog, state, owner).is_empty()
        }
        _ => false,
    }
}

pub(super) fn machine_group_b(
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
        ("yangfan", EventData::CardUsed(u)) => m.card = u.card,
        ("gwchenshui", EventData::Damage(_)) => m.card = Some(0),
        ("gwchenshui", EventData::TurnEndWindow { .. }) => m.card = Some(1),
        ("gwliedi", EventData::Damage(d)) => {
            m.card = Some(if d.settlement.committed_damage.is_none() { 0 } else { 1 });
        }
        ("gwliedi", EventData::TurnEndWindow { .. }) => m.card = Some(2),
        ("huanshu2", EventData::CardUsed(u)) => {
            m.card = Some(1);
            if let Some(c) = u.card {
                m.cards.push(c);
            }
        }
        ("huanshu2", EventData::Phase { .. }) => m.card = Some(2),
        ("gwjieyin", EventData::Phase { .. }) => m.card = Some(99),
        ("zhengjun", EventData::CardUsed(u)) => {
            m.card = u.card;
            m.stage = HostStage::ComparePlayerPick;
        }
        ("zhengjun", EventData::CardResponded { card, .. }) => {
            m.card = Some(*card);
            m.stage = HostStage::ComparePlayerPick;
        }
        ("zhengjun", EventData::TurnEndWindow { .. }) => m.stage = HostStage::DrawSettle,
        ("bolang", EventData::Damage(_)) => m.card = Some(1),
        ("gwjushi2", EventData::Damage(_)) => m.card = Some(1),
        _ => {}
    }
    m
}

pub(super) fn activate_group_b(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<HostMachine> {
    let state = ctx.state;
    let ok = match skill.name.as_str() {
        "shuangxi" => {
            state.turn.round >= flag(state, owner, "gw_shuangxi_ready_round")
                && !eligible_bronze_targets(state, ctx.catalog, owner, CardKind::Ciguhanshuang)
                    .is_empty()
        }
        "gwjieyin" => !jieyin_mask_options(state, owner).is_empty(),
        "gwjushi" => {
            !has_named_skill(state, ctx.catalog, owner, "gwjushi2")
                && !jushi_target_options(state, ctx.catalog, owner).is_empty()
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

pub(super) fn collect_fengchi_pool(catalog: &Catalog, state: &GameStateCore, owner: PlayerId) -> Vec<SkillDefId> {
    let mut pool = Vec::new();
    for (idx, sdef) in catalog.skills.iter().enumerate() {
        let def = idx as SkillDefId;
        if state.has_skill(owner, def) {
            continue;
        }
        if sdef.limits.active_outside_play {
            continue;
        }
        if sdef.name.starts_with('_') || sdef.name.ends_with("_card") || sdef.name.ends_with("_global") {
            continue;
        }
        let eligible = if let Some(beh) = sdef.behavior.as_host_behavior() {
            crate::content::host::activation::is_host_skill_bolan_eligible(beh, sdef.name)
        } else {
            false
        };
        if eligible {
            pool.push(def);
        }
    }
    pool
}
