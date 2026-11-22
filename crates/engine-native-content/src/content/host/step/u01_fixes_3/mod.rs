//! U01 Sub-batch 3 (`3D精选` 21 技) 上游等价修复模块。

use super::super::*;
use super::u01_batch_a::{flag, he_cards, set_flag, ASK_KEY};
use super::u01_fixes_1::{
    effective_is_red, is_delayed, is_sha_target_blocked, u01_discard_batch, u01_give_card_ex,
    virtual_legal_targets,
};
use crate::catalog::{CardKind, Catalog, QueryCtx, SkillLimits, TriggerMeta};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::{CardUseData, DamageNature, EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, EventId, PlayerId, SkillDefId};
use crate::machine::{damage_data_with, Ctx, Flow, LogEvent};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

mod asks_c;
mod asks_d;
mod helpers;
mod routing;
mod step_c;
mod step_d1;
mod step_d2;

use helpers::*;
pub(crate) use routing::*;

pub(crate) fn is_u01_fix_3_batch_c(name: &str) -> bool {
    matches!(
        name,
        "dddxiaheng"
            | "dddfengzheng"
            | "dddfengzheng_global"
            | "dddyulv"
            | "dddyulv_old"
            | "dddfenye"
            | "dddshichao"
    )
}

pub(crate) fn is_u01_fix_3_batch_d(name: &str) -> bool {
    matches!(
        name,
        "dddyouxue"
            | "dddchengjing"
            | "dddduanbing"
            | "dddxiaoxing"
            | "dddfuyi"
            | "dddzhuanshe"
            | "dddweiqiu"
            | "dddlianer"
            | "dddanzhi"
            | "dddshixing"
            | "xiangle"
            | "jiushi"
            | "rezhiheng"
            | "ddddanggu"
            | "dddlanghuai"
            | "dddxuanlun"
            | "dddjiexing"
            | "dddbailei"
    )
}

pub(crate) fn is_u01_fix_3_name(name: &str) -> bool {
    is_u01_fix_3_batch_c(name) || is_u01_fix_3_batch_d(name)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn active_ask_u01_fix3(
    ask_id: u32,
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let pair = if (3001..=3039).contains(&ask_id) {
        asks_c::active_ask_u01_fix3_c(
            ask_id, stage, owner, target, targets, card, cards, core, catalog,
        )
    } else if (3040..=3099).contains(&ask_id) {
        asks_d::active_ask_u01_fix3_d(
            ask_id, stage, owner, target, targets, card, cards, core, catalog,
        )
    } else {
        None
    }?;
    let (observer, spec) = pair;
    Some(crate::snap::ask(
        observer,
        spec.prompt,
        spec.kind,
        spec.options,
        spec.min,
        spec.max,
        spec.allow_cancel,
    ))
}

pub(crate) fn step_u01_fix3(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Option<Flow> {
    if is_u01_fix_3_batch_c(&machine.def_key) {
        step_c::step_u01_fix3_c(machine, ctx)
    } else if is_u01_fix_3_batch_d(&machine.def_key) {
        step_d1::step_u01_fix3_d1(machine, ctx)
            .or_else(|| step_d2::step_u01_fix3_d2(machine, ctx))
    } else {
        None
    }
}

pub(crate) fn u01_card_use_after_hook_3(ctx: &mut Ctx<'_>, u: &CardUseData) {
    let fid = flag(ctx.core, u.player, "last_popped_card_use_fid");

    // 1. `dddlianer`: AFTER red single-card use finishes, if physical card is STILL in discard -> optional regain + rank cap.
    if !u.cancelled {
        if let Some(cid) = u.card {
            if ctx.core.discard.contains(&cid)
                && ctx.core.player(u.player).alive
                && !ctx.catalog.is_tieji_sealed(ctx.core, u.player)
                && ctx
                    .catalog
                    .skill_by_name("dddlianer")
                    .is_some_and(|d| ctx.core.has_skill(u.player, d))
                && effective_is_red(ctx.core, ctx.catalog, u.player, cid)
            {
                push_intrinsic_park_3(
                    ctx,
                    "dddlianer",
                    HostBehavior::U01DddBatchD,
                    u.player,
                    None,
                    Some(cid),
                    Vec::new(),
                );
            }
        }
    }

    // 2. `dddchengjing_use` (`useCardAfter`): if card used via dddchengjing is still in discard/table -> optional give to mark_holder.next_alive!
    if fid != 0 && flag(ctx.core, u.player, &format!("dddchengjing_use_fid:{fid}")) != 0 {
        ctx.core
            .vm_aux
            .flags
            .remove(&(u.player, format!("dddchengjing_use_fid:{fid}")));
        if !u.cancelled {
            if let Some(cid) = u.card {
                let mut pushed_give = false;
                if (ctx.core.discard.contains(&cid) || ctx.core.table.contains(&cid))
                    && ctx.core.player(u.player).alive
                {
                    if let Some(holder) = global_youxue_holder(ctx.core) {
                        let dest = ctx.core.next_alive(holder);
                        if ctx.core.player(dest).alive {
                            push_intrinsic_park_3(
                                ctx,
                                "dddchengjing",
                                HostBehavior::U01DddBatchD,
                                u.player,
                                Some(dest),
                                Some(cid),
                                Vec::new(),
                            );
                            pushed_give = true;
                        }
                    }
                }
                if !pushed_give {
                    if ctx.core.table.contains(&cid) {
                        ctx.core.table.retain(|&x| x != cid);
                        ctx.to_discard(cid);
                    }
                    ctx.core.declared_kinds.remove(&cid);
                }
            }
        } else if let Some(cid) = u.card {
            if ctx.core.table.contains(&cid) {
                ctx.core.table.retain(|&x| x != cid);
                ctx.to_discard(cid);
            }
            ctx.core.declared_kinds.remove(&cid);
        }
    }

    // 3. `dddyulv` / `dddyulv_old`: flush any cards that entered discard during this use!
    flush_yulv_discard_queue(ctx);
}

pub(crate) fn on_cards_to_discard_u01_3(ctx: &mut Ctx<'_>) {
    flush_yulv_discard_queue(ctx);
}

pub(crate) fn on_outer_settled_u01_3(ctx: &mut Ctx<'_>) {
    flush_yulv_discard_queue(ctx);
    if ctx.res.stack.is_empty() {
        let cur = ctx.core.turn.current;
        if ctx
            .core
            .vm_aux
            .flags
            .remove(&(cur, "u01_abort_turn".to_string()))
            .unwrap_or(0)
            != 0
        {
            ctx.emit(EventData::TurnEnded { player: cur });
        }
    }
}

pub(crate) fn on_skill_removed_u01_3(
    core: &mut GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    skill_name: &str,
) {
    match skill_name {
        "dddxiaheng" => {
            core.vm_aux
                .flags
                .remove(&(owner, "dddxiaheng_damaged_mask".to_string()));
            core.vm_aux
                .flags
                .remove(&(owner, "dddxiaheng_del".to_string()));
        }
        "dddfengzheng" => {
            crate::state::sync_dddfusi_global(core, catalog);
        }
        "dddxiaoxing" => {
            core.vm_aux
                .flags
                .remove(&(owner, "dddxiaoxing_init_done".to_string()));
        }
        "dddshixing" => {
            let mask = flag(core, owner, "dddshixing_granted_mask");
            core.vm_aux
                .flags
                .remove(&(owner, "dddshixing_granted_mask".to_string()));
            for (bit, sname) in [(1u32, "xiangle"), (2u32, "jiushi"), (4u32, "rezhiheng")] {
                if (mask & bit) != 0 {
                    if let Some(d) = catalog.skill_by_name(sname) {
                        core.skills.remove(&(owner, d));
                    }
                }
            }
        }
        _ => {}
    }
}

pub(crate) fn u01_respond_skill_options_3(
    core: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    need: CardKind,
    is_use: bool,
    out: &mut Vec<ChoiceOption>,
) {
    if is_use
        && (need == CardKind::Jiu || (need == CardKind::Tao && core.player(player).dying))
        && !core.player(player).face_down
        && !catalog.is_tieji_sealed(core, player)
    {
        if let Some(def) = catalog.skill_by_name("jiushi") {
            if core.has_skill(player, def) {
                out.push(ChoiceOption::Skill(def));
            }
        }
    }
}

pub(crate) fn u01_try_activate_respond_skill_3(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
    def: SkillDefId,
    need: CardKind,
    _fid: EventId,
    is_use: bool,
) -> Option<bool> {
    if ctx.catalog.skill_name(def) != "jiushi" {
        return None;
    }
    if !is_use
        || ctx.core.player(player).face_down
        || !(need == CardKind::Jiu || (need == CardKind::Tao && ctx.core.player(player).dying))
    {
        return Some(false);
    }
    ctx.emit(EventData::TurnOverBefore {
        player,
        cancelled: false,
    });
    ctx.log(LogEvent::SkillTriggered {
        skill: "jiushi",
        owner: player,
        presentation: None,
    });
    Some(true)
}
