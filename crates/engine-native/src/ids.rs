//! 基础标识类型。全部为小型整数，可哈希、可比较、可编码进状态摘要。

/// 玩家（座位固定，按索引即座位顺序）。
pub type PlayerId = u8;
/// 牌定义/牌实例共用编号：目录中每张牌一个编号，对局中牌以编号指代。
pub type CardId = u32;
/// 技能定义编号：只读目录中的下标。
pub type SkillDefId = u16;
/// 事件编号：对局内单调递增。
pub type EventId = u32;

pub const NO_PLAYERS_MAX: usize = 8;
