use super::*;

impl Engine {
    pub fn catalog(&self) -> &SharedCatalog {
        &self.catalog
    }

    /// 在受控工厂内对目录挂载已验证的 `presentation_binding`（若当前共享标准目录被多处持有，则先构造独立副本再原子替换；若扩展目录被多处持有则拒绝修改）。
    pub fn try_with_catalog_mutation(
        &mut self,
        f: impl FnOnce(&mut crate::catalog::Catalog) -> Result<(), String>,
    ) -> Result<(), String> {
        if let Some(cat) = std::sync::Arc::get_mut(&mut self.catalog) {
            f(cat)
        } else if self.catalog.extension_skill_range.is_empty()
            && self.catalog.extension_generals.is_empty()
        {
            let mut cat = crate::content::standard::standard_catalog();
            f(&mut cat)?;
            self.catalog = std::sync::Arc::new(cat);
            Ok(())
        } else {
            Err("cannot mutate shared extension catalog in-place".into())
        }
    }

    pub fn log(&self) -> &[LogEvent] {
        &self.log
    }

    pub fn outcome(&self) -> Option<&Outcome> {
        self.state.core.outcome.as_ref()
    }

    pub fn turn(&self) -> &TurnState {
        &self.state.core.turn
    }

    /// 身份局的身份表（座位序；非身份局为 None）。
    pub fn roles(&self) -> Option<&[crate::state::Role]> {
        self.state.core.identity_roles()
    }

    /// 当前状态修订号：客户端提交下一条命令时回填 `expected_revision`。
    pub fn revision(&self) -> u64 {
        self.state.core.revision
    }

    /// 本目录的规则版本（与状态内 `ruleset_version` 一致）。
    pub fn ruleset_version(&self) -> u64 {
        self.catalog.ruleset_version
    }

    pub fn open_choice(&self) -> Option<&OpenChoice> {
        self.state.resolution.open_choice.as_ref()
    }

    /// 规范状态哈希。PA03：`canonical_hash` 已缓冲化（先拼缓冲再一次
    /// blake3 update），字节序与逐字段流式写入完全一致。
    pub fn state_hash(&self) -> [u8; 32] {
        canonical_hash(&self.state)
    }

    /// 目录指针稳定性探针：命令前后应一致（目录不随状态克隆）。
    pub fn catalog_ptr(&self) -> *const crate::catalog::Catalog {
        std::ptr::from_ref(self.catalog.as_ref())
    }

    /// 保存对局：含挂起机器局部状态的完整快照（规则版本绑定）。
    ///
    /// 快照可在任意静止点（Accepted/Pending 之后）获取；宿主日志不进快照，
    /// 恢复后的引擎从空日志开始。字节级落盘属平台/持久化阶段（见进度文档）。
    /// 测试/工具：可变核心状态访问（供 ai 模块信息集重采样用）。
    pub(crate) fn core_mut(&mut self) -> &mut crate::state::GameStateCore {
        &mut self.state.core
    }

    /// 测试/工具：只读访问核心状态。
    /// 测试钩子：可变核心状态（计数器等测试面直写）。
    pub fn state_core_mut(&mut self) -> &mut GameStateCore {
        &mut self.state.core
    }

    pub fn state_core(&self) -> &GameStateCore {
        &self.state.core
    }
}

impl Engine {
    pub(crate) fn resolution_mut(&mut self) -> &mut crate::resolution::Resolution {
        &mut self.state.resolution
    }

    /// 执行位置只读视图（含 NC04 瞬态故障槽，供宿主诊断）。
    pub fn resolution(&self) -> &crate::resolution::Resolution {
        &self.state.resolution
    }

    /// 测试/工具：注入真实事件帧并经引擎同一提交管线驱动。
    pub fn debug_drive_event(&mut self, event: EventData) -> CommandOutcome {
        let log_mark = self.log.len();
        let mut work = self.state.clone();
        work.resolution.budget = self.debug_budget.unwrap_or(DEFAULT_BUDGET);
        crate::resolution::push_frame(&mut work.resolution, event);
        self.finish_segment(work, log_mark)
    }
}
