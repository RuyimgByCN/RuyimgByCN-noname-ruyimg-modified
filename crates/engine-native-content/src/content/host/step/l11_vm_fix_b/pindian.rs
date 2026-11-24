//! L11-E2 `taixu` VM Fix B 共享拼点机（`quhu` / `rexianzhen` / `tianyi` / `zhuandui`）：
//! 完整支持 `txyonglve`（牌堆顶拼点）与 `txyingyang`（拼点点数 ±3），并保持与现有拼点回归测试选牌/选目标预置兼容。

use super::super::super::*;
use super::super::l11_common::*;
use super::super::l11_common_b2::{
    apply_yingyang_mode, has_txyingyang_ready, has_txyonglve_ready, pop_yonglve_deck_top,
};
use super::asks::{
    take_preset_card, wait_vm_b_ask, ASK_PINDIAN_SELF_CARD, ASK_PINDIAN_TARGET_CARD,
    ASK_TXYINGYANG_OWNER, ASK_TXYINGYANG_TARGET, ASK_TXYONGLVE_OWNER, ASK_TXYONGLVE_TARGET,
};
use crate::choice::ChoiceOption;
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::state::GameStateCore;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PindianStep {
    /// 拼点进行中（已暂停或待选）。
    Pending,
    /// 拼点已比点结算完毕（供外层按胜负分支）。
    Resolved {
        owner_card: CardId,
        target_card: CardId,
        win: bool,
    },
    /// 无法继续拼点（缺手牌或已离线）。
    Aborted,
}

fn flag_opt(core: &GameStateCore, p: PlayerId, key: &str) -> Option<u32> {
    core.vm_aux.flags.get(&(p, key.to_string())).copied()
}

pub(super) fn can_compare_pindian(
    core: &GameStateCore,
    catalog: &crate::catalog::Catalog,
    player: PlayerId,
) -> bool {
    !core.player(player).hand.is_empty() || has_txyonglve_ready(core, catalog, player)
}

pub(super) fn clear_pindian_ctx_flags(core: &mut GameStateCore, owner: PlayerId) {
    for suffix in ["pindian_oc", "pindian_oh", "pindian_tc", "pindian_th", "pindian_ody", "pindian_tdy"] {
        core.vm_aux.flags.remove(&(owner, format!("l11_vm_b:{suffix}")));
    }
}

/// 推进拼点状态机：`HostStage::Act` 进入时开始选己方牌（或 `txyonglve` / 测试 `vm_aux.selected_cards`），
/// 然后选目标牌、`txyingyang` 点数修改，最终同时离手弃置并得出结果。
pub(super) fn step_pindian(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    target: PlayerId,
) -> (PindianStep, Flow) {
    if !ctx.core.player(owner).alive || !ctx.core.player(target).alive {
        clear_pindian_ctx_flags(ctx.core, owner);
        return (PindianStep::Aborted, Flow::Done);
    }

    match (m.stage, m.card) {
        (HostStage::ComparePlayerPick, Some(ASK_TXYONGLVE_OWNER)) => {
            m.card = None;
            if is_confirmed(ctx) {
                if let Some(c) = pop_yonglve_deck_top(ctx) {
                    set_flag(ctx.core, owner, "l11_vm_b:pindian_oc", c + 1);
                    set_flag(ctx.core, owner, "l11_vm_b:pindian_oh", 0);
                    return proceed_target_pick(m, ctx, owner, target);
                }
            }
            if ctx.core.player(owner).hand.is_empty() {
                clear_pindian_ctx_flags(ctx.core, owner);
                return (PindianStep::Aborted, Flow::Done);
            }
            let flow = wait_vm_b_ask(m, ctx, HostStage::SelectCard, ASK_PINDIAN_SELF_CARD);
            (PindianStep::Pending, flow)
        }
        (HostStage::SelectCard, Some(ASK_PINDIAN_SELF_CARD)) => {
            m.card = None;
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                clear_pindian_ctx_flags(ctx.core, owner);
                return (PindianStep::Aborted, Flow::Done);
            };
            if !ctx.core.player(owner).hand.contains(&c) {
                clear_pindian_ctx_flags(ctx.core, owner);
                return (PindianStep::Aborted, Flow::Done);
            }
            set_flag(ctx.core, owner, "l11_vm_b:pindian_oc", c + 1);
            set_flag(ctx.core, owner, "l11_vm_b:pindian_oh", 1);
            proceed_target_pick(m, ctx, owner, target)
        }
        (HostStage::CompareTargetPick, Some(ASK_TXYONGLVE_TARGET)) => {
            m.card = None;
            if is_confirmed(ctx) {
                if let Some(c) = pop_yonglve_deck_top(ctx) {
                    set_flag(ctx.core, owner, "l11_vm_b:pindian_tc", c + 1);
                    set_flag(ctx.core, owner, "l11_vm_b:pindian_th", 0);
                    return proceed_owner_yingyang(m, ctx, owner, target);
                }
            }
            let t_hand_len = ctx.core.player(target).hand.len();
            if t_hand_len == 0 {
                clear_pindian_ctx_flags(ctx.core, owner);
                return (PindianStep::Aborted, Flow::Done);
            }
            if t_hand_len == 1 {
                let tc = ctx.core.player(target).hand[0];
                set_flag(ctx.core, owner, "l11_vm_b:pindian_tc", tc + 1);
                set_flag(ctx.core, owner, "l11_vm_b:pindian_th", 1);
                return proceed_owner_yingyang(m, ctx, owner, target);
            }
            let flow = wait_vm_b_ask(m, ctx, HostStage::CompareTargetPick, ASK_PINDIAN_TARGET_CARD);
            (PindianStep::Pending, flow)
        }
        (HostStage::CompareTargetPick, Some(ASK_PINDIAN_TARGET_CARD)) => {
            m.card = None;
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                clear_pindian_ctx_flags(ctx.core, owner);
                return (PindianStep::Aborted, Flow::Done);
            };
            if !ctx.core.player(target).hand.contains(&c) {
                clear_pindian_ctx_flags(ctx.core, owner);
                return (PindianStep::Aborted, Flow::Done);
            }
            set_flag(ctx.core, owner, "l11_vm_b:pindian_tc", c + 1);
            set_flag(ctx.core, owner, "l11_vm_b:pindian_th", 1);
            proceed_owner_yingyang(m, ctx, owner, target)
        }
        (HostStage::SelectNature, Some(ASK_TXYINGYANG_OWNER)) => {
            m.card = None;
            if let Some(ChoiceOption::Tag(mode)) = ctx.answer().first().copied() {
                set_flag(ctx.core, owner, "l11_vm_b:pindian_ody", u32::from(mode) + 1);
            }
            proceed_target_yingyang(m, ctx, owner, target)
        }
        (HostStage::SelectNature, Some(ASK_TXYINGYANG_TARGET)) => {
            m.card = None;
            if let Some(ChoiceOption::Tag(mode)) = ctx.answer().first().copied() {
                set_flag(ctx.core, owner, "l11_vm_b:pindian_tdy", u32::from(mode) + 1);
            }
            finalize_pindian(m, ctx, owner, target)
        }
        _ => {
            // 新启动拼点：消费可能存在的手工测试预置拼点牌
            clear_pindian_ctx_flags(ctx.core, owner);
            if let Some(preset) = take_preset_card(ctx.core, owner) {
                if ctx.core.player(owner).hand.contains(&preset) {
                    set_flag(ctx.core, owner, "l11_vm_b:pindian_oc", preset + 1);
                    set_flag(ctx.core, owner, "l11_vm_b:pindian_oh", 1);
                    return proceed_target_pick(m, ctx, owner, target);
                }
            }
            if has_txyonglve_ready(ctx.core, ctx.catalog, owner) {
                let flow = wait_vm_b_ask(m, ctx, HostStage::ComparePlayerPick, ASK_TXYONGLVE_OWNER);
                return (PindianStep::Pending, flow);
            }
            if ctx.core.player(owner).hand.is_empty() {
                return (PindianStep::Aborted, Flow::Done);
            }
            let flow = wait_vm_b_ask(m, ctx, HostStage::SelectCard, ASK_PINDIAN_SELF_CARD);
            (PindianStep::Pending, flow)
        }
    }
}

fn proceed_target_pick(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    target: PlayerId,
) -> (PindianStep, Flow) {
    if has_txyonglve_ready(ctx.core, ctx.catalog, target) {
        let flow = wait_vm_b_ask(m, ctx, HostStage::CompareTargetPick, ASK_TXYONGLVE_TARGET);
        return (PindianStep::Pending, flow);
    }
    let t_hand = &ctx.core.player(target).hand;
    if t_hand.is_empty() {
        clear_pindian_ctx_flags(ctx.core, owner);
        return (PindianStep::Aborted, Flow::Done);
    }
    if t_hand.len() == 1 {
        let tc = t_hand[0];
        set_flag(ctx.core, owner, "l11_vm_b:pindian_tc", tc + 1);
        set_flag(ctx.core, owner, "l11_vm_b:pindian_th", 1);
        return proceed_owner_yingyang(m, ctx, owner, target);
    }
    let flow = wait_vm_b_ask(m, ctx, HostStage::CompareTargetPick, ASK_PINDIAN_TARGET_CARD);
    (PindianStep::Pending, flow)
}

fn proceed_owner_yingyang(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    target: PlayerId,
) -> (PindianStep, Flow) {
    if has_txyingyang_ready(ctx.core, ctx.catalog, owner) {
        let flow = wait_vm_b_ask(m, ctx, HostStage::SelectNature, ASK_TXYINGYANG_OWNER);
        return (PindianStep::Pending, flow);
    }
    proceed_target_yingyang(m, ctx, owner, target)
}

fn proceed_target_yingyang(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    target: PlayerId,
) -> (PindianStep, Flow) {
    if has_txyingyang_ready(ctx.core, ctx.catalog, target) {
        let flow = wait_vm_b_ask(m, ctx, HostStage::SelectNature, ASK_TXYINGYANG_TARGET);
        return (PindianStep::Pending, flow);
    }
    finalize_pindian(m, ctx, owner, target)
}

fn finalize_pindian(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    target: PlayerId,
) -> (PindianStep, Flow) {
    m.stage = HostStage::Act;
    m.card = None;
    let Some(oc_p1) = flag_opt(ctx.core, owner, "l11_vm_b:pindian_oc") else {
        clear_pindian_ctx_flags(ctx.core, owner);
        return (PindianStep::Aborted, Flow::Done);
    };
    let Some(tc_p1) = flag_opt(ctx.core, owner, "l11_vm_b:pindian_tc") else {
        clear_pindian_ctx_flags(ctx.core, owner);
        return (PindianStep::Aborted, Flow::Done);
    };
    let oc = oc_p1 - 1;
    let tc = tc_p1 - 1;
    let oh = flag(ctx.core, owner, "l11_vm_b:pindian_oh") > 0;
    let th = flag(ctx.core, owner, "l11_vm_b:pindian_th") > 0;
    let ody = flag_opt(ctx.core, owner, "l11_vm_b:pindian_ody").map(|v| v - 1);
    let tdy = flag_opt(ctx.core, owner, "l11_vm_b:pindian_tdy").map(|v| v - 1);
    clear_pindian_ctx_flags(ctx.core, owner);

    if oh {
        ctx.core.player_mut(owner).hand.retain(|&x| x != oc);
    }
    if th {
        ctx.core.player_mut(target).hand.retain(|&x| x != tc);
    }
    ctx.to_discard(oc);
    ctx.to_discard(tc);
    if oh {
        ctx.note_cards_lost(owner, &[oc]);
        (ctx.catalog.hooks.on_hand_card_removed)(ctx, owner, oc);
    }
    if th {
        ctx.note_cards_lost(target, &[tc]);
        (ctx.catalog.hooks.on_hand_card_removed)(ctx, target, tc);
    }

    let r_o_raw = effective_card_rank(ctx.catalog, oc);
    let r_t_raw = effective_card_rank(ctx.catalog, tc);
    let r_o = ody.map_or(r_o_raw, |mode| apply_yingyang_mode(r_o_raw, mode));
    let r_t = tdy.map_or(r_t_raw, |mode| apply_yingyang_mode(r_t_raw, mode));
    let win = r_o > r_t;
    (
        PindianStep::Resolved {
            owner_card: oc,
            target_card: tc,
            win,
        },
        Flow::Done,
    )
}
