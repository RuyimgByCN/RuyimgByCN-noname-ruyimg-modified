//! R05 抽牌事件族（裁决 28／29；合同 v3 §3–§4）。
//!
//! - `DrawPhase`（D）：一次真实「摸牌阶段／额外摸牌阶段」的根帧，
//!   窗口序 Before→Begin→Begin1→Begin2→主 draw→End→After。
//! - `Draw`（R）：一次真实抽牌；`held_cards` 是已抽未获牌实体的**唯一
//!   owner**（在手/牌堆/弃堆之外的中立持有区，对席位/观战隐藏牌面）。
//! - `DrawGain`（G）：一次获得；`eligible` 是有序历史引用（Transfer 后
//!   保留），`progress` 分阶段约束见合同表。
//! - `detached_cards`：抽后 gain 取消/死亡且源未指定返还时的实体去处
//!   （`Unclaimed`/`Destroyed`，不可洗回）。

/// D 的窗口序（`EventData::DrawWindow`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawWindowKind {
    Before,
    Begin,
    Begin1,
    Begin2,
    End,
    After,
}

impl DrawWindowKind {
    pub fn event_kind(self) -> crate::event::EventKind {
        use crate::event::EventKind;
        match self {
            Self::Before => EventKind::DrawPhaseBefore,
            Self::Begin => EventKind::DrawPhaseBegin,
            Self::Begin1 => EventKind::DrawPhaseBegin1,
            Self::Begin2 => EventKind::DrawPhaseBegin2,
            Self::End => EventKind::DrawPhaseEnd,
            Self::After => EventKind::DrawPhaseAfter,
        }
    }
    /// 存档编码（表尾追加，顺序即值）。
    pub fn tag(self) -> u8 {
        match self {
            Self::Before => 0,
            Self::Begin => 1,
            Self::Begin1 => 2,
            Self::Begin2 => 3,
            Self::End => 4,
            Self::After => 5,
        }
    }
    pub fn from_tag(t: u8) -> Result<Self, crate::archive::ArchiveError> {
        Ok(match t {
            0 => Self::Before,
            1 => Self::Begin,
            2 => Self::Begin1,
            3 => Self::Begin2,
            4 => Self::End,
            5 => Self::After,
            _ => return Err(crate::archive::ArchiveError::BadTag("draw_window", t)),
        })
    }
}

/// D 的终态（§3 outcome）。`Running` 只在 D 载荷存在；弹栈前必为其余四者。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawOutcome {
    Running,
    Ended,
    Skipped,
    Cancelled,
    Omitted,
}

impl DrawOutcome {
    pub fn tag(self) -> u8 {
        match self {
            Self::Running => 0,
            Self::Ended => 1,
            Self::Skipped => 2,
            Self::Cancelled => 3,
            Self::Omitted => 4,
        }
    }
    pub fn from_tag(t: u8) -> Result<Self, crate::archive::ArchiveError> {
        Ok(match t {
            0 => Self::Running,
            1 => Self::Ended,
            2 => Self::Skipped,
            3 => Self::Cancelled,
            4 => Self::Omitted,
            _ => return Err(crate::archive::ArchiveError::BadTag("draw_outcome", t)),
        })
    }
}

/// R 的终态（§4 terminal）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawTerminal {
    Running,
    Completed,
    Cancelled,
    Prevented,
    /// 牌堆与弃堆皆枯竭：已抽部分转 detached，本次无 gain。
    Exhausted,
}

impl DrawTerminal {
    pub fn tag(self) -> u8 {
        match self {
            Self::Running => 0,
            Self::Completed => 1,
            Self::Cancelled => 2,
            Self::Prevented => 3,
            Self::Exhausted => 4,
        }
    }
    pub fn from_tag(t: u8) -> Result<Self, crate::archive::ArchiveError> {
        Ok(match t {
            0 => Self::Running,
            1 => Self::Completed,
            2 => Self::Cancelled,
            3 => Self::Prevented,
            4 => Self::Exhausted,
            _ => return Err(crate::archive::ArchiveError::BadTag("draw_terminal", t)),
        })
    }
}

/// G 的分阶段（§4 表）。`Done` 为弹栈态；`After` 允许牌已弃/转移/销毁。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawGainProgress {
    Filter,
    RecordHistory,
    Transfer,
    After,
    Done,
}

impl DrawGainProgress {
    pub fn tag(self) -> u8 {
        match self {
            Self::Filter => 0,
            Self::RecordHistory => 1,
            Self::Transfer => 2,
            Self::After => 3,
            Self::Done => 4,
        }
    }
    pub fn from_tag(t: u8) -> Result<Self, crate::archive::ArchiveError> {
        Ok(match t {
            0 => Self::Filter,
            1 => Self::RecordHistory,
            2 => Self::Transfer,
            3 => Self::After,
            4 => Self::Done,
            _ => return Err(crate::archive::ArchiveError::BadTag("draw_gain_progress", t)),
        })
    }
}

/// D 机推进位（机器态，随帧入档）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawPhaseProgress {
    Before,
    Begin,
    Begin1,
    Begin2,
    /// 等待主 draw（R）子树完成。
    MainWaiting,
    End,
    After,
    Done,
}

impl DrawPhaseProgress {
    pub fn tag(self) -> u8 {
        match self {
            Self::Before => 0,
            Self::Begin => 1,
            Self::Begin1 => 2,
            Self::Begin2 => 3,
            Self::MainWaiting => 4,
            Self::End => 5,
            Self::After => 6,
            Self::Done => 7,
        }
    }
    pub fn from_tag(t: u8) -> Result<Self, crate::archive::ArchiveError> {
        Ok(match t {
            0 => Self::Before,
            1 => Self::Begin,
            2 => Self::Begin1,
            3 => Self::Begin2,
            4 => Self::MainWaiting,
            5 => Self::End,
            6 => Self::After,
            7 => Self::Done,
            _ => return Err(crate::archive::ArchiveError::BadTag("draw_phase_progress", t)),
        })
    }
}

/// detached 实体的种类（§4）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetachedKind {
    Unclaimed,
    Destroyed,
}

impl DetachedKind {
    pub fn tag(self) -> u8 {
        match self {
            Self::Unclaimed => 0,
            Self::Destroyed => 1,
        }
    }
    pub fn from_tag(t: u8) -> Result<Self, crate::archive::ArchiveError> {
        Ok(match t {
            0 => Self::Unclaimed,
            1 => Self::Destroyed,
            _ => return Err(crate::archive::ArchiveError::BadTag("detached_kind", t)),
        })
    }
}
