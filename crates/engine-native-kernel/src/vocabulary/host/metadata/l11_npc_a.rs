//! L11 太虚幻境跨包 NPC Part A（tag 1391）元数据钩子：按技能名派发见内容层 `skill_defs.rs`。
use super::super::*;

impl HostBehavior {
    pub(super) fn trigger_meta_l11_npc_a(&self) -> Option<TriggerMeta> {
        match self {
            HostBehavior::L11CrossNpcA => None,
            _ => None,
        }
    }
}
