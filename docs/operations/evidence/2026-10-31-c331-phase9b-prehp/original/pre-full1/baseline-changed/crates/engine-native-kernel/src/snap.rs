//! 快照与恢复：状态规范形态 + 信任边界校验 + 开放选择承接派生。
//!
//! 三分职责（N04/N05，ADR 0774）：
//! - **规范快照类型**：`StateSnapshot` = 核心状态 + 执行位置 + 段数据；
//!   挂起机器以 `MachineSnapshot` 全字段导出（字节存档与规范摘要共用）；
//! - **恢复期校验**：`validate_core` / `validate_card_ownership` /
//!   `validate_resolution` 在任何恢复路径上拒绝越界 ID、破损帧链、
//!   牌归属不闭合与承接关系不一致的快照（恶意快照不得产出引擎）；
//! - **承接派生**：`active_ask` 从挂起机器的 stage/asked 状态派生
//!   「当前应有的开放选择询问」，与快照内的 `open_choice` 逐字段核对
//!   （N04-e）——机器字段篡改即失去承接。

use crate::catalog::{CardKind, Catalog, SkillMachine};
use crate::choice::{ChoiceKind, ChoiceOption, OpenChoice, Selection};
use crate::vocabulary::host::GJ_FOOD_MIZHLIANOU_KEY;
use crate::vocabulary::host::HostBehavior;
use crate::vocabulary::generals::{general_gender, Gender};
use crate::vocabulary::names::{GUI_CAI, JI_JIU, LONG_DAN, QING_GUO};
use crate::event::{DamageNature, EventData, Frame};
use crate::ids::{CardId, PlayerId, SkillDefId};
use crate::resolution::{Slot, NO_FRAME};
use crate::state::{GameMode, GameStateCore, Outcome, Winner};

mod types;
pub use types::*;
mod ownership;
pub use ownership::*;
mod validation;
pub use validation::*;
mod machine_validation;
use machine_validation::*;
mod asks;
pub use asks::*;
mod restore;
