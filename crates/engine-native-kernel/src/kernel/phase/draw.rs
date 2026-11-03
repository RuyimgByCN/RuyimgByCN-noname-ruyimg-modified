use super::super::*;
use crate::event::{DrawOutcome as O, DrawPhaseProgress as DP};

pub(super) fn begin_draw(ctx: &mut Ctx<'_>, player: PlayerId) {
    // R05（规则 333）：摸牌阶段改由真实 D 帧驱动——跳过路径 D 直接以
    // Skipped 终止（Before 前终止，不发窗口）；正常路径 D 从 Before 走
    // 完整窗口序，主 draw 经 R/G 异步完成（替代旧同步 draw_cards＋
    // num 修正内联堆叠；各修正已按同口径移入 D 机的 pending 前窗）。
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
        ctx.emit(EventData::DrawPhase {
            player,
            progress: DP::Done,
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
