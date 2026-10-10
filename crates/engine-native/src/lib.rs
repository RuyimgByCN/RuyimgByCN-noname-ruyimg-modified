//! 干净原生内核原型：只读规则目录 + 紧凑对局状态 + 显式事件栈 +
//! 类型化技能暂停恢复。
//!
//! 本 crate 是重写第一阶段（原生内核）的落地验证场，不以兼容旧
//! `engine-core` 为目标；语义参照见 `docs/adr/0703-*`。

// 穷尽匹配中的重复臂（牌种路由重复、快照解码表重复 tag）会遮蔽
// 真实分支，按错误处理。
#![deny(unreachable_patterns)]

pub mod catalog;
pub mod choice;
pub mod ai;
pub mod archive;
pub mod content;
pub mod view;
pub mod engine;
pub mod event;
pub mod ids;
pub mod kernel;
pub mod machine;
pub mod presentation;
pub mod resolution;
pub mod rng;
pub mod snap;
pub mod state;

pub use catalog::{CardKind, Catalog, EquipSlot, SharedCatalog, SkillBehavior, SkillDef, SkillMachine};
pub use choice::{ChoiceKind, ChoiceOption, ChoiceSpec, OpenChoice};
pub use engine::{AuthToken, CommandOutcome, Engine, GameAction, GameCommand, RejectReason};
pub use event::{DamageData, EventData, EventKind, Phase, Stage, Timing};
pub use state::{GameMode, Outcome, Role, Winner};
pub use view::{ChoiceView, EventView, HandView, PlayerCellView, PlayerView};
pub use ids::{CardId, EventId, PlayerId, SkillDefId};
pub use machine::{Ctx, Flow, LogEvent};
pub use resolution::Resolution;
pub use snap::{MachineSnapshot, ParkSnapshot, ResolutionSnapshot, SnapshotError, StateSnapshot};
