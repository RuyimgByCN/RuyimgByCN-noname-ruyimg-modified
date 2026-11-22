//! L11（太虚幻境）Batch L11-5 领主与通用武将技（`L11GeneralBossC` tag 741 & `L11BossLingfeng` tag 742：`l11_boss_c.rs`）。
//!
//! 承载前半 11 技并转派后半 11 技至 `l11_boss_c2`：
//! - `txhj_mouduan`、`olshanjia`、`boss_xuanlei`、`txhj_xuanlve`、`boss_baolian`、`boss_lingfeng`
//! - `dz_gs_yinfu`、`boss_leili`、`boss_fengxing`、`boss_yushou`、`boss_xixing`

use super::l11_boss_c2::*;
use super::l11_boss_c_asks::*;
use super::l11_common::*;
use super::l11_common_b2::{damage_effective_kind, emit_virtual_kind_use};
use super::l11_skills_b_asks::{can_receive_moved_card, movable_field_sources};
use super::*;
use crate::catalog::{CardKind, QueryCtx, SkillLimits, TriggerMeta};
use crate::event::{
    DamageNature, DrawOutcome, EventData, EventKind, Frame, LostOrigin, Phase, Timing,
};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn wait_bc_ask(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    stage: HostStage,
    ask_id: u32,
) -> Flow {
    m.card = Some(ask_id);
    if let Some((observer, spec)) = build_l11_boss_c_ask(
        ask_id,
        m.owner,
        m.target,
        &m.targets,
        m.card,
        &m.cards,
        ctx.core,
        ctx.catalog,
    ) {
        m.stage = stage;
        Flow::WaitChoice { observer, spec }
    } else {
        m.card = None;
        Flow::Done
    }
}

pub(in crate::content::host) fn trigger_meta_l11_boss_c(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, timing: Timing, priority: i32| TriggerMeta {
        event,
        timing,
        priority,
        order_selectable: false,
        optional: false,
        extra_events: Vec::new(),
    };
    match name {
        "txhj_mouduan" | "boss_baolian" | "boss_lianyu" => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0))
        }
        "olshanjia" | "boss_xixing" | "boss_modao" | "boss_zhennu" | "boss_yushou"
        | "boss_mojian" | "boss_skonghun" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 0)),
        "boss_xuanlei" | "dz_gs_yinfu" | "boss_fengxing" => {
            Some(meta(EventKind::TurnBegin, Timing::Normal, 1))
        }
        "txhj_xuanlve" => Some(meta(EventKind::CardsLost, Timing::Normal, 0)),
        "boss_moyany" | "boss_danshu" => Some(TriggerMeta {
            event: EventKind::CardsLost,
            timing: Timing::Normal,
            priority: 0,
            order_selectable: false,
            optional: false,
            extra_events: vec![
                EventKind::CardsDiscarded,
                EventKind::CardsMoved,
                EventKind::CardUsed,
            ],
        }),
        "boss_lingfeng" => Some(meta(EventKind::DrawPhaseBefore, Timing::Normal, 1)),
        "boss_jueji" => Some(meta(EventKind::DrawPhaseBegin1, Timing::Normal, 1)),
        "boss_leili" => Some(meta(EventKind::DamageEnd, Timing::Normal, 0)),
        "txhj_zhuwei" => Some(meta(EventKind::OrdinaryJudge, Timing::Normal, 0)),
        "txhj_chuhai2" => Some(meta(EventKind::DamageSource, Timing::Normal, 0)),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l11_boss_c(name: &str) -> SkillLimits {
    if name == "txhj_chuhai" {
        SkillLimits {
            once_per_turn: true,
            active_outside_play: false,
        }
    } else {
        SkillLimits::default()
    }
}

pub(in crate::content::host) fn filter_l11_boss_c_with_frames(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    frames: &[Frame],
    frame: Option<&Frame>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    match skill.name.as_str() {
        "txhj_mouduan" => {
            if !is_unblocked_l11_owner(state, ctx.catalog, owner) {
                return false;
            }
            let suits_cnt = flag(state, owner, "l11_turn_used_suits_mask").count_ones();
            let types_cnt = flag(state, owner, "l11_turn_used_types_mask").count_ones();
            matches!(
                event,
                EventData::Phase {
                    player,
                    phase: Phase::End,
                    begin: true
                } if *player == owner
            ) && (suits_cnt >= 4 || types_cnt >= 3)
                && !movable_field_sources(state, ctx.catalog, owner).is_empty()
        }
        "olshanjia" => {
            is_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(
                    event,
                    EventData::Phase {
                        player,
                        phase: Phase::Play,
                        begin: true
                    } if *player == owner
                )
        }
        "boss_xuanlei" => {
            is_locked_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(event, EventData::TurnBegin { player } if *player == owner)
                && alive_others(state, owner)
                    .into_iter()
                    .any(|p| !state.is_friend_of(ctx.catalog, owner, p) && !state.player(p).judge.is_empty())
        }
        "txhj_xuanlve" => {
            is_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(
                    event,
                    EventData::CardsLost { player, origins, .. }
                        if *player == owner && origins.contains(&LostOrigin::Equip)
                )
                && alive_others(state, owner)
                    .into_iter()
                    .any(|p| !he_cards(state, p).is_empty())
        }
        "boss_baolian" => {
            is_locked_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(
                    event,
                    EventData::Phase {
                        player,
                        phase: Phase::End,
                        begin: true
                    } if *player == owner
                )
        }
        "boss_lingfeng" => {
            is_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(
                    event,
                    EventData::DrawPhase {
                        player,
                        outcome: DrawOutcome::Running,
                        ..
                    } if *player == owner
                )
        }
        "dz_gs_yinfu" => {
            is_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(event, EventData::TurnBegin { player } if *player == owner)
                && {
                    let dmg_hp = (state.player(owner).max_hp as i32 - state.player(owner).hp).max(0) as u32;
                    dmg_hp >= state.turn.round.max(1)
                }
        }
        "boss_leili" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && is_unblocked_l11_owner(state, ctx.catalog, owner)
                    && damage_effective_kind(state, frames, frame, ctx.catalog, d)
                        .is_some_and(CardKind::is_sha_like)
                    && !enemy_targets(state, ctx.catalog, owner, Some(d.target)).is_empty()
        ),
        "boss_fengxing" => {
            is_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(event, EventData::TurnBegin { player } if *player == owner)
                && enemy_targets(state, ctx.catalog, owner, None)
                    .into_iter()
                    .any(|p| sha_target_valid(state, ctx.catalog, owner, p))
        }
        "boss_yushou" => {
            is_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(
                    event,
                    EventData::Phase {
                        player,
                        phase: Phase::Play,
                        begin: true
                    } if *player == owner
                )
                && !aoe_enemy_targets(state, ctx.catalog, owner, CardKind::SavageAttack).is_empty()
        }
        "boss_xixing" => {
            is_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(
                    event,
                    EventData::Phase {
                        player,
                        phase: Phase::Prepare,
                        begin: true
                    } if *player == owner
                )
                && alive_others(state, owner)
                    .into_iter()
                    .any(|p| state.player(p).linked)
        }
        _ => filter_l11_boss_c2_with_frames(skill, ctx, frames, frame, owner, event),
    }
}

pub(in crate::content::host) fn machine_l11_boss_c(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let behavior = if skill.name == "boss_lingfeng" {
        HostBehavior::L11BossLingfeng
    } else {
        HostBehavior::L11GeneralBossC
    };
    let (target, card) = match event {
        EventData::Damage(d) => (Some(d.target), d.card),
        EventData::DrawPhase { player, .. } => (Some(*player), None),
        EventData::DrawWindow { root, .. } => (None, Some(*root)),
        EventData::OrdinaryJudge { invocation, .. } => (None, Some(*invocation)),
        _ => (None, None),
    };
    HostMachine {
        def_key: skill.name.clone(),
        behavior,
        owner,
        target,
        card,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn activate_l11_boss_c(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    if skill.name != "txhj_chuhai"
        || !is_unblocked_l11_owner(ctx.state, ctx.catalog, owner)
        || chuhai_targets(ctx.state, owner).is_empty()
    {
        return None;
    }
    active_skill_machine("txhj_chuhai", HostBehavior::L11GeneralBossC, owner)
}

pub(in crate::content::host) fn handle_l11_boss_c(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if !ctx.core.player(owner).alive {
        set_flag(ctx.core, owner, "l11_moyany_ing", 0);
        return Flow::Done;
    }
    match m.def_key.as_str() {
        "txhj_mouduan" => handle_mouduan(m, ctx),
        "olshanjia" => handle_olshanjia(m, ctx),
        "boss_xuanlei" => handle_xuanlei(m, ctx),
        "txhj_xuanlve" => handle_xuanlve(m, ctx),
        "boss_baolian" => {
            ctx.start_draw(owner, 2);
            Flow::Done
        }
        "boss_lingfeng" => handle_lingfeng(m, ctx),
        "dz_gs_yinfu" => {
            ctx.recover(owner, 1, None);
            let n = flag(ctx.core, owner, "l11_yinfu_uses") + 1;
            set_flag(ctx.core, owner, "l11_yinfu_uses", n);
            if n >= 3 {
                remove_named_skill(ctx, owner, "dz_gs_yinfu");
            }
            Flow::Done
        }
        "boss_leili" => handle_leili(m, ctx),
        "boss_fengxing" => handle_fengxing(m, ctx),
        "boss_yushou" => match m.stage {
            HostStage::Act => wait_bc_ask(m, ctx, HostStage::TargetChoice, ASK_YUSHOU_CONFIRM),
            HostStage::TargetChoice => {
                m.card = None;
                if !is_confirmed(ctx) {
                    return Flow::Done;
                }
                let tgts = aoe_enemy_targets(ctx.core, ctx.catalog, owner, CardKind::SavageAttack);
                if tgts.is_empty() {
                    return Flow::Done;
                }
                emit_virtual_kind_use(
                    ctx,
                    owner,
                    CardKind::SavageAttack,
                    tgts,
                    DamageNature::Normal,
                );
                m.stage = HostStage::DrawSettle;
                Flow::Yield
            }
            _ => Flow::Done,
        },
        "boss_xixing" => match m.stage {
            HostStage::Act => wait_bc_ask(m, ctx, HostStage::TargetChoice, ASK_XIXING_TARGET),
            HostStage::TargetChoice => {
                m.card = None;
                let Some(t) = answered_players(ctx)
                    .into_iter()
                    .next()
                    .filter(|&p| p != owner && ctx.core.player(p).alive && ctx.core.player(p).linked)
                else {
                    return Flow::Done;
                };
                m.stage = HostStage::SelectNature;
                emit_damage(ctx, Some(owner), t, 1, DamageNature::Thunder, None);
                Flow::Yield
            }
            HostStage::SelectNature => {
                if ctx.core.player(owner).alive {
                    ctx.recover(owner, 1, None);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        _ => handle_l11_boss_c2(m, ctx),
    }
}

fn handle_mouduan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let _owner = m.owner;
    match m.stage {
        HostStage::Act => wait_bc_ask(m, ctx, HostStage::TargetChoice, ASK_MOUDUAN_CONFIRM),
        HostStage::TargetChoice => {
            m.card = None;
            if !is_confirmed(ctx) {
                return Flow::Done;
            }
            wait_bc_ask(m, ctx, HostStage::SelectCard, ASK_MOUDUAN_FROM)
        }
        HostStage::SelectCard => {
            m.card = None;
            let Some(from) = answered_players(ctx).into_iter().next() else {
                return Flow::Done;
            };
            m.target = Some(from);
            wait_bc_ask(m, ctx, HostStage::SelectNature, ASK_MOUDUAN_CARD)
        }
        HostStage::SelectNature => {
            m.card = None;
            let Some(cid) = answered_cards(ctx).into_iter().next() else {
                return Flow::Done;
            };
            m.cards = vec![cid];
            wait_bc_ask(m, ctx, HostStage::CompareTargetPick, ASK_MOUDUAN_TO)
        }
        HostStage::CompareTargetPick => {
            m.card = None;
            let (Some(from), Some(cid), Some(to)) = (
                m.target.filter(|&p| ctx.core.player(p).alive),
                m.cards.pop(),
                answered_players(ctx).into_iter().next(),
            ) else {
                return Flow::Done;
            };
            if can_receive_moved_card(ctx.core, ctx.catalog, from, to, cid) {
                if ctx.core.player(from).equips.contains(&cid) {
                    ctx.core.player_mut(from).equips.retain(|&c| c != cid);
                    crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, from, cid);
                    ctx.note_cards_lost(from, &[cid]);
                    ctx.core.player_mut(to).equips.push(cid);
                    crate::kernel::grant_equip_skills(ctx, to, cid);
                    set_flag(ctx.core, to, &format!("l11_move_equip_evt:{cid}"), 1);
                    ctx.emit(EventData::CardsLost {
                        player: from,
                        cards: vec![cid],
                        origins: vec![LostOrigin::Equip],
                    });
                    ctx.emit(EventData::Equipped {
                        player: to,
                        card: cid,
                        replaced: None,
                    });
                    m.stage = HostStage::DrawSettle;
                    return Flow::Yield;
                } else if ctx.core.player(from).judge.contains(&cid) {
                    ctx.core.player_mut(from).judge.retain(|&c| c != cid);
                    ctx.note_cards_lost(from, &[cid]);
                    ctx.core.player_mut(to).judge.push(cid);
                    ctx.emit(EventData::CardsLost {
                        player: from,
                        cards: vec![cid],
                        origins: vec![LostOrigin::Judge],
                    });
                    m.stage = HostStage::DrawSettle;
                    return Flow::Yield;
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn handle_olshanjia(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_bc_ask(m, ctx, HostStage::TargetChoice, ASK_OLSHANJIA_CONFIRM),
        HostStage::TargetChoice => {
            m.card = None;
            if !is_confirmed(ctx) {
                return Flow::Done;
            }
            m.stage = HostStage::DrawSettle;
            ctx.start_draw(owner, 3);
            Flow::Yield
        }
        HostStage::DrawSettle => {
            if !ctx.core.player(owner).alive {
                return Flow::Done;
            }
            let x = flag(ctx.core, owner, "l11_olshanjia_lost_equips").min(3);
            let need = 3 - x;
            if need == 0 {
                return wait_bc_ask(m, ctx, HostStage::SelectNature, ASK_OLSHANJIA_SHA_TARGET);
            }
            let pool = he_cards(ctx.core, owner);
            if pool.is_empty() {
                return wait_bc_ask(m, ctx, HostStage::SelectNature, ASK_OLSHANJIA_SHA_TARGET);
            }
            if pool.len() <= need as usize {
                let any_bt = pool
                    .iter()
                    .any(|&c| ctx.catalog.card_kind(c).equip_slot().is_none());
                discard_player_cards(ctx, owner, &pool);
                if any_bt {
                    return Flow::Done;
                }
                m.stage = HostStage::CompareTargetPick;
                return Flow::Yield;
            }
            set_flag(ctx.core, owner, "l11_olshanjia_need", need);
            wait_bc_ask(m, ctx, HostStage::SelectCard, ASK_OLSHANJIA_DISCARD)
        }
        HostStage::CompareTargetPick => {
            wait_bc_ask(m, ctx, HostStage::SelectNature, ASK_OLSHANJIA_SHA_TARGET)
        }
        HostStage::SelectCard => {
            m.card = None;
            let need = (flag(ctx.core, owner, "l11_olshanjia_need") as usize).clamp(1, 3);
            set_flag(ctx.core, owner, "l11_olshanjia_need", 0);
            let pool = he_cards(ctx.core, owner);
            let picks: Vec<CardId> = answered_cards(ctx)
                .into_iter()
                .filter(|c| pool.contains(c))
                .collect();
            if picks.len() != need {
                return Flow::Done;
            }
            let any_bt = picks
                .iter()
                .any(|&c| ctx.catalog.card_kind(c).equip_slot().is_none());
            discard_player_cards(ctx, owner, &picks);
            if any_bt {
                return Flow::Done;
            }
            wait_bc_ask(m, ctx, HostStage::SelectNature, ASK_OLSHANJIA_SHA_TARGET)
        }
        HostStage::SelectNature => {
            m.card = None;
            if let Some(t) = answered_players(ctx)
                .into_iter()
                .next()
                .filter(|&p| p != owner && sha_target_valid(ctx.core, ctx.catalog, owner, p))
            {
                set_flag(ctx.core, owner, "vm_va_sha_no_count", 1);
                emit_virtual_kind_use(ctx, owner, CardKind::Sha, vec![t], DamageNature::Normal);
                m.stage = HostStage::ComparePlayerPick;
                return Flow::Yield;
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn handle_xuanlei(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act {
        m.targets = alive_others(ctx.core, owner)
            .into_iter()
            .filter(|&p| !ctx.core.is_friend_of(ctx.catalog, owner, p) && !ctx.core.player(p).judge.is_empty())
            .collect();
        m.stage = HostStage::DrawSettle;
    }
    while let Some(t) = m.targets.first().copied() {
        m.targets.remove(0);
        if ctx.core.player(t).alive {
            emit_damage(ctx, Some(owner), t, 1, DamageNature::Thunder, None);
            return Flow::Yield;
        }
    }
    Flow::Done
}

fn handle_xuanlve(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_bc_ask(m, ctx, HostStage::TargetChoice, ASK_XUANLVE_TARGET),
        HostStage::TargetChoice => {
            m.card = None;
            let Some(t) = answered_players(ctx)
                .into_iter()
                .next()
                .filter(|&p| p != owner && ctx.core.player(p).alive && !he_cards(ctx.core, p).is_empty())
            else {
                return Flow::Done;
            };
            m.target = Some(t);
            wait_bc_ask(m, ctx, HostStage::SelectCard, ASK_XUANLVE_CARD)
        }
        HostStage::SelectCard => {
            m.card = None;
            if let (Some(t), Some(cid)) = (
                m.target.filter(|&p| ctx.core.player(p).alive),
                answered_cards(ctx).into_iter().next(),
            ) {
                if he_cards(ctx.core, t).contains(&cid) {
                    discard_player_cards(ctx, t, &[cid]);
                    m.stage = HostStage::DrawSettle;
                    return Flow::Yield;
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn handle_lingfeng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if let Some(root) = m.card.take() {
                set_flag(ctx.core, owner, "l11_lingfeng_draw_root", root + 1);
            }
            wait_bc_ask(m, ctx, HostStage::TargetChoice, ASK_LINGFENG_CONFIRM)
        }
        HostStage::TargetChoice => {
            m.card = None;
            let root = flag(ctx.core, owner, "l11_lingfeng_draw_root").checked_sub(1);
            set_flag(ctx.core, owner, "l11_lingfeng_draw_root", 0);
            if !is_confirmed(ctx) {
                return Flow::Done;
            }
            let frame_idx = root
                .and_then(|id| {
                    ctx.res.stack.iter().position(|f| {
                        f.id == id
                            && matches!(
                                f.data,
                                EventData::DrawPhase {
                                    player,
                                    outcome: DrawOutcome::Running,
                                    ..
                                } if player == owner
                            )
                    })
                })
                .or_else(|| {
                    ctx.res.stack.iter().rposition(|f| {
                        matches!(
                            f.data,
                            EventData::DrawPhase {
                                player,
                                outcome: DrawOutcome::Running,
                                ..
                            } if player == owner
                        )
                    })
                });
            if let Some(fr) = frame_idx.map(|i| &mut ctx.res.stack[i]) {
                if let EventData::DrawPhase { outcome, .. } = &mut fr.data {
                    *outcome = DrawOutcome::Cancelled;
                }
            }
            m.cards.clear();
            for _ in 0..2 {
                if let Some(c) = pop_deck_with_reshuffle(ctx.core) {
                    m.cards.push(c);
                }
            }
            if m.cards.len() == 2
                && effective_color_is_red(ctx.core, ctx.catalog, owner, m.cards[0])
                    != effective_color_is_red(ctx.core, ctx.catalog, owner, m.cards[1])
                && !enemy_targets(ctx.core, ctx.catalog, owner, None).is_empty()
            {
                return wait_bc_ask(m, ctx, HostStage::SelectCard, ASK_LINGFENG_TARGET);
            }
            deliver_lingfeng_cards(m, ctx);
            Flow::Done
        }
        HostStage::SelectCard => {
            m.card = None;
            if let Some(t) = answered_players(ctx)
                .into_iter()
                .next()
                .filter(|&p| ctx.core.player(p).alive && !ctx.core.is_friend_of(ctx.catalog, owner, p))
            {
                m.stage = HostStage::SelectNature;
                emit_lose_hp(ctx, t, 1);
                return Flow::Yield;
            }
            deliver_lingfeng_cards(m, ctx);
            Flow::Done
        }
        HostStage::SelectNature => {
            deliver_lingfeng_cards(m, ctx);
            Flow::Done
        }
        _ => {
            deliver_lingfeng_cards(m, ctx);
            Flow::Done
        }
    }
}

fn deliver_lingfeng_cards(m: &mut HostMachine, ctx: &mut Ctx<'_>) {
    let owner = m.owner;
    if m.cards.is_empty() {
        return;
    }
    let taken: Vec<CardId> = m.cards.drain(..).collect();
    if ctx.core.player(owner).alive {
        let delivered = (ctx.catalog.hooks.on_before_gain_hand)(ctx, None, owner, &taken);
        for &c in &delivered {
            ctx.core.player_mut(owner).hand.push(c);
        }
        if !delivered.is_empty() {
            ctx.out.push(LogEvent::CardsDrawn {
                player: owner,
                cards: delivered.clone(),
            });
            ctx.emit(EventData::CardsDrawn {
                player: owner,
                cards: delivered,
            });
        }
    } else {
        for c in taken {
            ctx.to_discard(c);
        }
    }
}

fn handle_leili(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_bc_ask(m, ctx, HostStage::TargetChoice, ASK_LEILI_TARGET),
        HostStage::TargetChoice => {
            m.card = None;
            if let Some(t) = answered_players(ctx)
                .into_iter()
                .next()
                .filter(|&p| {
                    Some(p) != m.target
                        && ctx.core.player(p).alive
                        && !ctx.core.is_friend_of(ctx.catalog, owner, p)
                })
            {
                m.stage = HostStage::DrawSettle;
                emit_damage(ctx, Some(owner), t, 1, DamageNature::Thunder, None);
                return Flow::Yield;
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn handle_fengxing(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_bc_ask(m, ctx, HostStage::TargetChoice, ASK_FENGXING_TARGET),
        HostStage::TargetChoice => {
            m.card = None;
            if let Some(t) = answered_players(ctx)
                .into_iter()
                .next()
                .filter(|&p| {
                    !ctx.core.is_friend_of(ctx.catalog, owner, p)
                        && sha_target_valid(ctx.core, ctx.catalog, owner, p)
                })
            {
                set_flag(ctx.core, owner, "vm_va_sha_no_count", 1);
                emit_virtual_kind_use(ctx, owner, CardKind::Sha, vec![t], DamageNature::Normal);
                m.stage = HostStage::DrawSettle;
                return Flow::Yield;
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
