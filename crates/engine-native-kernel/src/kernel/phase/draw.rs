use super::super::*;
use crate::event::{DrawOutcome as O, DrawPhaseProgress as DP};

pub(super) fn begin_draw(ctx: &mut Ctx<'_>, player: PlayerId) {
    // R05（规则 333）：摸牌阶段改由真实 D 帧驱动——跳过路径 D 以预置终态
    // Skipped 入机，D 机只发恒发射终窗 After（R05-F2：跳过也有触发面）；
    // 正常路径 D 从 Before 走完整窗口序，主 draw 经 R/G 异步完成（替代旧
    // 同步 draw_cards＋num 修正内联堆叠；各修正已按同口径移入 D 机的
    // pending 前窗）。
    let skipped = ctx.core.turn.skip_draw_phase
        || ctx.core.player(player).skip_next_draw
        || ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_JIUSHOU)
            .map(|def| ctx.core.has_skill(player, def))
            .unwrap_or(false);
    if skipped {
        if ctx.core.turn.skip_draw_phase {
            ctx.core.turn.skip_draw_phase = false;
        }
        if ctx.core.player(player).skip_next_draw {
            ctx.core.player_mut(player).skip_next_draw = false;
        }
        ctx.record_phase_skipped(player);
        // R05-F2（缺口2）：跳过路径以预置终态 outcome=Skipped 从 Before 入
        // 机——D 机见预置终态只发恒发射终窗 After（DrawPhaseAfter 触发面，
        // 上游 phaseDrawSkipped 类技能由此承载；Ended/Skipped 都落 After、
        // D 载荷 outcome 区分），不再直接 Intrinsic→Done 无窗终止。
        ctx.emit(EventData::DrawPhase {
            player,
            progress: DP::Before,
            pending: 0,
            num_fixed: false,
            actual_gain: 0,
            outcome: O::Skipped,
        });
        return;
    }
    ctx.core.turn.mengtai_draw_pending = false;
    ctx.emit(EventData::DrawPhase {
        player,
        progress: DP::Before,
        pending: 2,
        num_fixed: false,
        actual_gain: 0,
        outcome: O::Running,
    });
}
