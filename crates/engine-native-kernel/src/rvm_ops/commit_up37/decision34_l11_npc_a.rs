//! Phase 3 P3-C1a 迁移完成桩：`L11` Cross NPC Part A（80 + 3 技）已全部迁入
//! `engine-native-content` 原生行为组 `HostBehavior::L11CrossNpcA`，不再经
//! `L11_TAIXU_SKILL_ID` (`PhaseBegin`) 分派；保留本入口供 `decision34_l11_skills`
//! 仅对 Part B（`[80..]`）调用时恒返回 `false`。

use super::*;

pub(super) fn try_resolve_l11_npc_skill_a(
    _ctx: &mut Ctx<'_>,
    _skill_name: &str,
    _owner: PlayerId,
) -> bool {
    false
}
