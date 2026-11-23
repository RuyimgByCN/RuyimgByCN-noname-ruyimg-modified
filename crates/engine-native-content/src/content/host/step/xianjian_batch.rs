//! U05 仙剑（`xianjian`）`Batch A..E` 统一外观模块（`xianjian_batch.rs`）。

use super::*;
use crate::catalog::{Catalog, QueryCtx, SkillLimits, TriggerMeta};
use crate::event::EventData;
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

#[path = "xianjian_batch_asks.rs"]
mod xianjian_batch_asks;
#[path = "xianjian_batch_a.rs"]
mod xianjian_batch_a;
#[path = "xianjian_batch_b.rs"]
mod xianjian_batch_b;
#[path = "xianjian_batch_c.rs"]
mod xianjian_batch_c;
#[path = "xianjian_batch_d.rs"]
mod xianjian_batch_d;

pub(crate) use self::xianjian_batch_a::host_sha_end_xianjian;
pub(crate) use self::xianjian_batch_asks::{
    host_draw_begin_xianjian, host_on_choose_respond_begin_xianjian, host_on_damage_end_xianjian,
    host_on_hand_card_removed_xianjian, host_on_hp_changed_xianjian, host_on_phase_end_xianjian,
    host_on_phase_use_end_xianjian, host_on_recover_before_xianjian,
};
pub(crate) use self::xianjian_batch_c::{
    host_card_use_after_xianjian, host_on_round_start_xianjian, host_play_begin_xianjian,
};

pub(in crate::content::host) fn trigger_meta(behavior: HostBehavior, name: &str) -> Option<TriggerMeta> {
    match behavior {
        HostBehavior::U05XianjianBatchA => xianjian_batch_a::trigger_meta(name),
        HostBehavior::U05XianjianBatchB => xianjian_batch_b::trigger_meta(name),
        HostBehavior::U05XianjianBatchC => xianjian_batch_c::trigger_meta(name),
        HostBehavior::U05XianjianBatchD => xianjian_batch_d::trigger_meta(name),
        _ => None,
    }
}

pub(in crate::content::host) fn limits(behavior: HostBehavior, name: &str) -> SkillLimits {
    match behavior {
        HostBehavior::U05XianjianBatchA => xianjian_batch_a::limits(name),
        HostBehavior::U05XianjianBatchB => xianjian_batch_b::limits(name),
        HostBehavior::U05XianjianBatchC => xianjian_batch_c::limits(name),
        HostBehavior::U05XianjianBatchD => xianjian_batch_d::limits(name),
        _ => SkillLimits::default(),
    }
}

pub(in crate::content::host) fn has_phase_use_activation(behavior: HostBehavior, name: &str) -> bool {
    behavior == HostBehavior::U05XianjianBatchD && xianjian_batch_d::has_active(name)
}

pub(in crate::content::host) fn activate(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<HostMachine> {
    if skill.behavior == HostBehavior::U05XianjianBatchD {
        xianjian_batch_d::activate(skill, ctx, owner)
    } else {
        None
    }
}

pub(in crate::content::host) fn filter_with_frames(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    frames: &[crate::event::Frame],
    frame: &crate::event::Frame,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    match skill.behavior {
        HostBehavior::U05XianjianBatchA => xianjian_batch_a::filter_with_frame(skill, ctx, frame, owner, event),
        HostBehavior::U05XianjianBatchB => xianjian_batch_b::filter_with_frames(skill, ctx, frames, frame, owner, event),
        HostBehavior::U05XianjianBatchC => xianjian_batch_c::filter(skill, ctx, owner, event),
        HostBehavior::U05XianjianBatchD => xianjian_batch_d::filter(skill, ctx, owner, event),
        _ => false,
    }
}

pub(in crate::content::host) fn machine(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    match skill.behavior {
        HostBehavior::U05XianjianBatchA => xianjian_batch_a::machine(skill, owner, event),
        HostBehavior::U05XianjianBatchB => xianjian_batch_b::machine(skill, owner, event),
        HostBehavior::U05XianjianBatchC => xianjian_batch_c::machine(skill, owner, event),
        HostBehavior::U05XianjianBatchD => xianjian_batch_d::machine(skill, owner, event),
        _ => HostMachine {
            def_key: skill.name.clone(),
            behavior: skill.behavior,
            owner,
            target: None,
            card: None,
            cards: Vec::new(),
            targets: Vec::new(),
            stage: HostStage::Act,
        },
    }
}

pub(in crate::content::host) fn step(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.behavior {
        HostBehavior::U05XianjianBatchA => xianjian_batch_a::step(m, ctx),
        HostBehavior::U05XianjianBatchB => xianjian_batch_b::step(m, ctx),
        HostBehavior::U05XianjianBatchC => xianjian_batch_c::step(m, ctx),
        HostBehavior::U05XianjianBatchD => xianjian_batch_d::step(m, ctx),
        HostBehavior::U05XianjianBatchE => Flow::Done,
        _ => Flow::Done,
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::content::host) fn ask(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    xianjian_batch_asks::ask_xianjian_batch_impl(stage, owner, target, targets, card, cards, core, catalog)
}
