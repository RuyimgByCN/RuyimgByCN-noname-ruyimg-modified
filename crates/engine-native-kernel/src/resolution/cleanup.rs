//! `boss_baonu` 等跳回 `phaseLoop` 时的全局栈/牌区清理（弹栈单帧清理复用 `frame_hooks`）。

use crate::ids::PlayerId;
use crate::machine::Ctx;

/// Unwind all in-flight frames/parks and processing-zone cards when jumping straight back to
/// `phaseLoop` (e.g., `boss_baonu`).
pub fn unwind_to_phase_loop(ctx: &mut Ctx<'_>) {
    while let Some(popped) = ctx.res.stack.pop() {
        crate::resolution::frame_hooks::cleanup_unwound_frame(&popped, ctx);
    }
    ctx.res.parks.clear();
    ctx.res.open_choice = None;
    ctx.res.answer = None;
    for p in ctx.core.players.iter_mut() {
        p.dying = false;
    }
    for p in 0..ctx.core.players.len() as PlayerId {
        ctx.core
            .vm_aux
            .flags
            .remove(&(p, "q4_rewansha_sealed".to_string()));
    }
    let table_cards: Vec<crate::ids::CardId> = ctx.core.table.drain(..).collect();
    for c in table_cards {
        ctx.to_discard(c);
    }
}
