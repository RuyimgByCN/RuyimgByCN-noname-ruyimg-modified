//! 原生技能内容模块：具体技能的条件、流程与局部状态归此处，
//! 与通用内核（移动牌、伤害、选择、事件调度）分离。

pub mod standard;
pub mod host;
pub mod names;

pub mod extension;
mod hooks;
pub mod pile;
