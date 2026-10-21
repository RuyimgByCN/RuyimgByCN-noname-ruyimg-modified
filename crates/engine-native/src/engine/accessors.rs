use super::*;

impl Engine {
    pub fn catalog(&self) -> &SharedCatalog {
        &self.catalog
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
    /// 执行位置只读视图（含 NC04 瞬态故障槽，供宿主诊断）。
    pub fn resolution(&self) -> &crate::resolution::Resolution {
        &self.state.resolution
    }
}
