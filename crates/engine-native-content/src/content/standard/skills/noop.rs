use super::*;

// ---------------------------------------------------------------------------
// 目录
// ---------------------------------------------------------------------------

/// 权限型能力（无触发时机）的空行为：判定流程经 skill_by_name 查询持有者，
/// 该行为永不进入触发队列与挂起槽。
pub(in super::super) struct NoopBehavior;

impl SkillBehavior for NoopBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, _owner: PlayerId, _event: &EventData) -> bool {
        false
    }
    fn start(&self, _owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(crate::kernel::NoopMachine::new())
    }
}
