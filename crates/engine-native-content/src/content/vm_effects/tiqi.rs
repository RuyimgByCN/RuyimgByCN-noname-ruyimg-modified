//! R05-E 涕泣（合同 v3 §5）：D 终窗 live 统计本 D 的 n——排除自己的 D；
//! n≠2 可选确认 → 真实摸 d=|n−2|（start_draw＋Yield，取消不撤已摸牌）→
//! 目标（D 角色）仍在场才选 +d/−d/取消 → 对本逻辑身份的 Limit 槽
//! 累加有符号值（Some0 保留，不落通用 AddMark 的零删除语义）。
use super::*;

/// 状态：`[stage, n, target+1]`；stage 1=确认询问、2=奖励 R 子树结算中、
/// 3=符号询问。stage 1 前无状态（窗口资格未通过即静默结束）。
pub(super) fn step(ctx: &mut Ctx<'_>, skill: &str, owner: u8, state: &mut Vec<u32>) -> Flow {
    // 入口存活门（§5：进入单个 async content 后不再逐 await 重加拦截）。
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    if state.is_empty() {
        // 窗口资格：当前必须处在真实 D 的终窗内（D 帧仍在栈上，n 为
        // live 折叠值）；NO_FRAME/Intrinsic 不能冒充该授权（§7）。
        let Some((draw_player, n)) = current_draw_phase(ctx) else {
            r05_fault(ctx, skill, "R05WindowMismatch", "tiqi requires a live draw phase frame");
            return Flow::Done;
        };
        if draw_player == owner {
            return Flow::Done; // 排除自己的 D
        }
        if n == 2 {
            return Flow::Done; // n==2 不询问
        }
        *state = vec![1, n, draw_player as u32 + 1];
        return wait(ask(
            owner,
            "rvm_tiqi_confirm",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm],
            1,
            1,
            true,
        ));
    }
    let target = (state[2] - 1) as u8;
    let d = (state[1] as i32 - 2).unsigned_abs();
    match state[0] {
        1 => {
            if ctx.answer().is_empty() {
                return Flow::Done; // 确认取消：不摸牌不写槽
            }
            state[0] = 2;
            // 真实 draw(d)：R/G 子树由调度器在 Yield 后推进（§3）。
            ctx.start_draw(owner, d);
            Flow::Yield
        }
        2 => {
            // 目标仍在场才选符号；离场/死亡都不选（§9.5，奖励牌不撤）。
            if !ctx.core.player(target).alive {
                return Flow::Done;
            }
            state[0] = 3;
            wait(ask(
                owner,
                "rvm_tiqi_sign",
                ChoiceKind::PickTag,
                vec![ChoiceOption::Tag(1), ChoiceOption::Tag(2)],
                1,
                1,
                true,
            ))
        }
        3 => {
            let delta = match ctx.answer().first() {
                Some(ChoiceOption::Tag(1)) => d as i32,
                Some(ChoiceOption::Tag(2)) => -(d as i32),
                _ => return Flow::Done, // 取消符号：已摸牌不撤
            };
            let Some(def) = r05_helper_def(ctx.catalog, skill, R05_ROLES[0]) else {
                r05_fault(ctx, skill, "R05HelperMissing", "limit helper not derived");
                return Flow::Done;
            };
            if r05_mark_add(ctx.core, target, def, delta).is_none() {
                r05_fault(ctx, skill, "R05MarkOverflow", "limit slot i32 overflow");
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn active_ask(core: &GameStateCore, owner: u8, state: &[u32]) -> Option<ExpectedAsk> {
    if !core.player(owner).alive {
        return None;
    }
    Some(match state.first() {
        Some(1) => ask(
            owner,
            "rvm_tiqi_confirm",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm],
            1,
            1,
            true,
        ),
        Some(3) => ask(
            owner,
            "rvm_tiqi_sign",
            ChoiceKind::PickTag,
            vec![ChoiceOption::Tag(1), ChoiceOption::Tag(2)],
            1,
            1,
            true,
        ),
        _ => return None,
    })
}
