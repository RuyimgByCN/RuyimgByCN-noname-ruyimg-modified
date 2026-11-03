//! R05-E 避祸（合同 v3 §5）：按触发窗口分三面——
//! · 合格 Source 窗 −1／End 窗 +1（每事件一次；可取消）：owner 选一名
//!   在场目标（含自己）→ 对 Next 槽有符号累加（抵消 Some0 不删）。
//!   资格：source 存在且存活、非自伤、owner 在场（§7：victim 死亡不
//!   取消存活 source 资格；每事件一次由触发面结构保证）。
//! · 目标真实回合开始窗（TurnBegin）：Next 槽存在（含 0）且目标存活
//!   → 原子 take：移除 Next → 写 Current（累加防御，含 0 保留）→ 装
//!   默认 Current helper。
//! 其余窗口（含 NO_FRAME/Intrinsic 主动面）不能冒充授权（§7）。
use super::*;

/// 状态：`[stage, mode]`；stage 1=窗口目标询问（mode 0=Source −1、
/// 1=End +1）。take 面无状态（原子一步）。
pub(super) fn step(ctx: &mut Ctx<'_>, skill: &str, owner: u8, state: &mut Vec<u32>) -> Flow {
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    if state.is_empty() {
        if let Some((window, root)) = current_damage_window(ctx) {
            let mode = match window {
                crate::event::DamageWindowKind::Source => 0u32,
                crate::event::DamageWindowKind::End => 1u32,
                _ => {
                    // 其他伤害窗不是避祸面（同一 compound 只声明 Source/
                    // End/回合开始三面；别的窗口拒）。
                    r05_fault(
                        ctx,
                        skill,
                        "R05WindowMismatch",
                        "dcbihuo window must be damage source or end",
                    );
                    return Flow::Done;
                }
            };
            // 凭据：source 存在、存活、非自伤（§7——与伤害是否已致 victim
            // 死亡无关）。
            let qualified = damage_root_data(ctx, root).is_some_and(|d| {
                d.source.is_some_and(|s| s != d.target && ctx.core.player(s).alive)
            });
            if !qualified {
                return Flow::Done;
            }
            *state = vec![1, mode];
            return wait(ask(
                owner,
                "rvm_dcbihuo_target",
                ChoiceKind::SelectPlayers,
                alive_options(ctx.core),
                1,
                1,
                true,
            ));
        }
        if let Some(turn_player) = current_turn_begin_player(ctx) {
            return take_next(ctx, skill, turn_player);
        }
        r05_fault(
            ctx,
            skill,
            "R05WindowMismatch",
            "dcbihuo requires a real damage source/end or turn-begin window",
        );
        Flow::Done
    } else {
        // 窗口目标询问已答：取消或写 Next 槽。
        let mode = state[1];
        let Some(ChoiceOption::Player(target)) = ctx.answer().first() else {
            return Flow::Done; // 取消
        };
        let target = *target;
        if !ctx.core.player(target).alive {
            return Flow::Done;
        }
        let delta = if mode == 0 { -1 } else { 1 };
        let Some(def) = r05_helper_def(ctx.catalog, skill, R05_ROLES[2]) else {
            r05_fault(ctx, skill, "R05HelperMissing", "next helper not derived");
            return Flow::Done;
        };
        if r05_mark_add(ctx.core, target, def, delta).is_none() {
            r05_fault(ctx, skill, "R05MarkOverflow", "next slot i32 overflow");
            return Flow::Done;
        }
        // F 批（§6）：首次写 Next 即装目标实例（enabled）——take 门要求
        // enabled 实例存在；实例随 Next 条目原样保留。
        r05_install_helper(ctx, target, def);
        Flow::Done
    }
}

/// 回合开始面的原子 take：含 0 即取（Some0 也是启用值）；移除 Next、
/// 写 Current、装默认 Current helper。Next 缺失或目标已死则原样保留。
fn take_next(ctx: &mut Ctx<'_>, skill: &str, turn_player: u8) -> Flow {
    let (Some(next_def), Some(current_def)) = (
        r05_helper_def(ctx.catalog, skill, R05_ROLES[2]),
        r05_helper_def(ctx.catalog, skill, R05_ROLES[3]),
    ) else {
        r05_fault(ctx, skill, "R05HelperMissing", "next/current helper not derived");
        return Flow::Done;
    };
    if !ctx.core.player(turn_player).alive {
        return Flow::Done;
    }
    // 未尽项 7（F 批）：Next 的 take 以 helper 实例 enabled 为精确门（§6
    // 「Next 读取还要求原 enabled 与 alive」）——条目存在但实例 disabled
    //（显式 disable）或不存在实例（绕过装载器）都不 take；条目保留。
    let enabled = u16::try_from(next_def)
        .map(|id| {
            ctx.core
                .skills
                .get(&(turn_player, id))
                .is_some_and(|inst| inst.enabled)
        })
        .unwrap_or(false);
    if !enabled {
        return Flow::Done;
    }
    if let Some(value) = ctx.core.vm_aux.marks.remove(&(turn_player, next_def)) {
        // 同回合新伤害继续写 Next、不覆盖 Current（§5）：take 写入按
        // 累加处理（含 0 保留条目）。
        if r05_mark_add(ctx.core, turn_player, current_def, value).is_none() {
            ctx.core.vm_aux.marks.insert((turn_player, next_def), value);
            r05_fault(ctx, skill, "R05MarkOverflow", "current slot i32 overflow");
            return Flow::Done;
        }
        r05_install_helper(ctx, turn_player, current_def);
    }
    Flow::Done
}

pub(super) fn active_ask(core: &GameStateCore, owner: u8, state: &[u32]) -> Option<ExpectedAsk> {
    if !core.player(owner).alive || state.first() != Some(&1) {
        return None;
    }
    Some(ask(
        owner,
        "rvm_dcbihuo_target",
        ChoiceKind::SelectPlayers,
        alive_options(core),
        1,
        1,
        true,
    ))
}
