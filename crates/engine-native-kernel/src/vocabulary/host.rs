//! 宿主行为的不可变描述、键、标签与触发元数据；执行由内容层 wrapper 承担。

use crate::catalog::TriggerMeta;
use crate::event::{EventKind, Timing};

mod behavior;
pub use behavior::HostBehavior;
mod behavior_list;
mod keys;
mod tags;
mod tag_decode;
mod metadata;
mod constants;
pub use constants::*;

#[derive(Debug, Clone, Copy)]
pub struct HostSkillDecl {
    /// 技能名（目录内唯一；桥接期固定为 'static）。
    pub name: &'static str,
    pub behavior: HostBehavior,
}
