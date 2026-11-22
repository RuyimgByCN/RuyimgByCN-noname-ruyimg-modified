//! L11（太虚幻境）Batch L11-4 衍生/被动辅助技能（`L11BuffHelpersB` tag 738）：
//!
//! 承载 10 项 `extension_skill.js:3425..3607` 衍生技能：
//! - `txhj_taoyuanyishi`、`taoyuanyishiX`（桃园己方回血 +1 标记）
//! - `txhj_jufuxionghao`（上游注释空操作）
//! - `txhj_luoshenjuezhang`（每轮首次使用或打出【闪】后摸 1，并挂 `txhj_luoshen`）
//! - `txhj_luoshen`（新轮次 `TurnBegin` 时自动移除，重置 `txhj_luoshenjuezhang` 轮次计数）
//! - `txhj_mowangdejianiang`（非醉杀/非连环【杀】伤害按本回合用酒次数加成）
//! - `txhj_jianiang_clear`（`Phase::End` 结束 / `TurnEnded` 清除佳酿层数）
//! - `txhj_shierqicejin`、`txhj_shierqicetui`（AOE/群体锦囊豁免标记）
//! - `txhj_pofuchenzhou`（准备阶段与结束阶段开始时各摸 1）

use super::l11_common::*;
use super::l11_common_b2::damage_effective_kind;
use super::l11_common_b4::*;
use super::*;
use crate::catalog::{CardKind, QueryCtx, SkillLimits, SkillMachine, TriggerMeta};
use crate::event::{EventData, EventKind, Phase, Timing};
use crate::ids::PlayerId;
use crate::machine::{Ctx, Flow};

pub(in crate::content::host) fn trigger_meta_l11_buff_helpers_b(
    name: &str,
) -> Option<TriggerMeta> {
    let meta = |event: EventKind, timing: Timing, priority: i32, extra_events| TriggerMeta {
        event,
        timing,
        priority,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "txhj_luoshenjuezhang" => Some(meta(
            EventKind::CardResponded,
            Timing::Normal,
            0,
            vec![EventKind::CardUsed],
        )),
        "txhj_luoshen" => Some(meta(EventKind::TurnBegin, Timing::Normal, 0, Vec::new())),
        "txhj_mowangdejianiang" => Some(meta(
            EventKind::DamageBegin1,
            Timing::Normal,
            6,
            Vec::new(),
        )),
        "txhj_jianiang_clear" => Some(meta(
            EventKind::PhaseEnd,
            Timing::Normal,
            0,
            vec![EventKind::TurnEnded],
        )),
        "txhj_pofuchenzhou" => Some(meta(
            EventKind::PhaseBegin,
            Timing::Normal,
            12,
            Vec::new(),
        )),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l11_buff_helpers_b(_name: &str) -> SkillLimits {
    SkillLimits::default()
}

pub(in crate::content::host) fn activate_l11_buff_helpers_b(
    _skill: &HostSkillBehavior,
    _ctx: &QueryCtx<'_>,
    _owner: PlayerId,
) -> Option<Box<dyn SkillMachine>> {
    None
}

pub(in crate::content::host) fn filter_l11_buff_helpers_b_with_frames(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    frames: &[crate::event::Frame],
    frame: Option<&crate::event::Frame>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    if !is_locked_unblocked_l11_owner(state, ctx.catalog, owner) {
        return false;
    }
    match skill.name.as_str() {
        "txhj_luoshenjuezhang" => {
            let round_stamp = state.turn.round + 1;
            if has_named_skill(state, ctx.catalog, owner, "txhj_luoshen")
                || flag(state, owner, "l11_luoshen_round") == round_stamp
            {
                return false;
            }
            match event {
                EventData::CardUsed(u) => {
                    u.player == owner && !u.cancelled && u.effective_kind(ctx.catalog) == CardKind::Shan
                }
                EventData::CardResponded { card, player } => {
                    *player == owner && effective_card_kind(state, ctx.catalog, *card) == CardKind::Shan
                }
                _ => false,
            }
        }
        "txhj_luoshen" => matches!(
            event,
            EventData::TurnBegin { .. }
                if flag(state, owner, "l11_luoshen_round") != state.turn.round + 1
        ),
        "txhj_mowangdejianiang" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.from_chain
                    && !d.prevented
                    && flag(state, owner, "l11_jianiang_stacks") > 0
                    && damage_effective_kind(state, frames, frame, ctx.catalog, d)
                        .is_some_and(CardKind::is_sha_like)
                    && !sha_damage_used_jiu(state, frames, frame, ctx.catalog, owner)
        ),
        "txhj_jianiang_clear" => {
            flag(state, owner, "l11_jianiang_stacks") > 0
                && (matches!(
                    event,
                    EventData::Phase {
                        player,
                        phase: Phase::End,
                        begin: false
                    } if *player == owner
                ) || matches!(event, EventData::TurnEnded { player } if *player == owner))
        }
        "txhj_pofuchenzhou" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Prepare | Phase::End,
                begin: true
            } if *player == owner
        ),
        _ => false,
    }
}

pub(in crate::content::host) fn machine_l11_buff_helpers_b(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    _event: &EventData,
) -> HostMachine {
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L11BuffHelpersB,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn handle_l11_buff_helpers_b(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "txhj_luoshenjuezhang" => {
            let round_stamp = ctx.core.turn.round + 1;
            set_flag(ctx.core, owner, "l11_luoshen_round", round_stamp);
            ensure_perm_skill(ctx, owner, "txhj_luoshen");
            ctx.start_draw(owner, 1);
            Flow::Done
        }
        "txhj_luoshen" => {
            set_flag(ctx.core, owner, "l11_luoshen_round", 0);
            remove_named_skill(ctx, owner, "txhj_luoshen");
            Flow::Done
        }
        "txhj_mowangdejianiang" => {
            let stacks = flag(ctx.core, owner, "l11_jianiang_stacks");
            if stacks > 0 {
                with_current_damage_mut(ctx, |dm| {
                    dm.amount += stacks as i32;
                });
            }
            Flow::Done
        }
        "txhj_jianiang_clear" => {
            set_flag(ctx.core, owner, "l11_jianiang_stacks", 0);
            Flow::Done
        }
        "txhj_pofuchenzhou" => {
            ctx.start_draw(owner, 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}
