//! L19 末日乱斗（noname-liuli）的可移植决策。
//!
//! 上游是乱斗模式框架 + 66 个技能（145 KB）。本模块迁移**语义明确的策略**：
//! - `chanceIdentity` 链：乱斗模式按人数依次给「下一个玩家」分配身份名
//!   （`er`/`san`/`si`/`wu1`/`liu`/`qi`/`ba`/`jiu1`/`shi`/`shiyi`…），每步以
//!   `game.players.length > N` 为续链门（见 `IDENTITY_GATES`）。
//! - `zaibian`（灾变）：出牌阶段开始锁定——摸
//!   `忠 + 2 - 反 - 内`（下限 0）张牌。
//! - `_wujinmoshi*`（无尽模式配置面）：摸牌/伤害/回复/手牌上限的**配置加成**
//!   （上游以 `player == game.me` 限定「人类位」——原生以参数注入座位集合，
//!   登记差异）。
//!
//! 乱斗模式框架（模式注册、24 处武将引用、动画）**不迁移**。本模块不读写
//! 文件、不请求网络、不 eval。

use std::collections::BTreeMap;

/// `chanceIdentity` 链的身份名（按链序；首项 `er` 无门，其余见 `IDENTITY_GATES`）。
pub const IDENTITY_NAMES: [&str; 15] = [
    "er", "san", "si", "wu1", "liu", "qi", "ba", "jiu1", "shi", "shiyi", "shier", "shisan", "shisi",
    "shiwu", "shiliu",
];

/// 每步续链门（`game.players.length > gate`）；下标与 [`IDENTITY_NAMES`] 对齐。
/// 首项 `None` = 无门（`chanceIdentity` 本身不检查人数）。
pub const IDENTITY_GATES: [Option<usize>; 15] = [
    None,
    Some(3),
    Some(4),
    Some(5),
    Some(6),
    Some(7),
    Some(8),
    Some(9),
    Some(10),
    Some(11),
    Some(12),
    Some(13),
    Some(14),
    Some(15),
    Some(16),
];

/// 按人数求身份分配序列（`seats` 名玩家，依次分配到「下一个玩家」）。
///
/// 上游为**链式**：`chanceIdentity` 无条件分配首项，之后仅当
/// `players.length > gate` 才继续下一项（例如 4 人 → `er/san/si` 三项）。
pub fn identity_chain(seats: usize) -> Vec<&'static str> {
    let mut out = Vec::new();
    for (i, name) in IDENTITY_NAMES.iter().enumerate() {
        match IDENTITY_GATES[i] {
            None => out.push(*name),
            Some(gate) => {
                if seats > gate {
                    out.push(*name);
                } else {
                    break;
                }
            }
        }
    }
    out
}

/// 灾变（`zaibian`）摸牌数：`忠 + 2 - 反 - 内`（下限 0）。
pub fn zaibian_draw(loyalists: usize, rebels: usize, renegades: usize) -> usize {
    (loyalists as i64 + 2 - rebels as i64 - renegades as i64).max(0) as usize
}

/// 无尽模式配置（`lib.config.wujinmoshi*`）。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WujinMoshiConfig {
    /// 摸牌阶段额外摸牌（`wujinmoshiDraw`）。
    pub draw: u32,
    /// 造成的伤害 +N（`wujinmoshiDamage`）。
    pub damage: u32,
    /// 受到的伤害修正（`wujinmoshiDamageI` 减项）。
    pub damage_taken: u32,
    /// 回复量 +N（`wujinmoshiRecover`）。
    pub recover: u32,
    /// 手牌上限 +N（`wujinmoshiMaxHandCard`）。
    pub max_hand_card: u32,
    /// 额外伤害除数（`wujinmoshiDXB / 15` 向下取整；`_wujinmoshiDamageE`）。
    pub damage_divisor: u32,
    /// 「人类位」座位集合（上游 `player == game.me`；登记差异）。
    pub human_seats: Vec<u8>,
}

impl WujinMoshiConfig {
    /// 该座位是否享受无尽模式加成（上游 `player == game.me`）。
    pub fn applies_to(&self, seat: u8) -> bool {
        self.human_seats.contains(&seat)
    }

    /// 摸牌加成（非人类位为 0）。
    pub fn draw_bonus(&self, seat: u8) -> u32 {
        if self.applies_to(seat) {
            self.draw
        } else {
            0
        }
    }

    /// 伤害加成（非人类位为 0）。
    pub fn damage_bonus(&self, seat: u8) -> u32 {
        if self.applies_to(seat) {
            self.damage
        } else {
            0
        }
    }

    /// 回复加成（非人类位为 0）。
    pub fn recover_bonus(&self, seat: u8) -> u32 {
        if self.applies_to(seat) {
            self.recover
        } else {
            0
        }
    }

    /// 手牌上限加成（非人类位为 0）。
    pub fn hand_card_bonus(&self, seat: u8) -> u32 {
        if self.applies_to(seat) {
            self.max_hand_card
        } else {
            0
        }
    }

    /// 非人类位造成的伤害修正：`floor(DXB/15) - damage_taken`（可为负）。
    pub fn non_human_damage_delta(&self) -> i64 {
        (self.damage_divisor / 15) as i64 - self.damage_taken as i64
    }
}

/// 无尽模式配置解析（键名 → 字段）。
pub fn parse_wujinmoshi(values: &BTreeMap<String, i64>) -> WujinMoshiConfig {
    let get = |k: &str| -> u32 { values.get(k).copied().unwrap_or(0).max(0) as u32 };
    WujinMoshiConfig {
        draw: get("wujinmoshiDraw"),
        damage: get("wujinmoshiDamage"),
        damage_taken: get("wujinmoshiDamageI"),
        recover: get("wujinmoshiRecover"),
        max_hand_card: get("wujinmoshiMaxHandCard"),
        damage_divisor: get("wujinmoshiDXB"),
        human_seats: values
            .get("humanSeat")
            .copied()
            .filter(|v| (0..64).contains(v))
            .map(|v| vec![v as u8])
            .unwrap_or_default(),
    }
}

/// 末日乱斗 2 位僵尸武将完整元数据（`末日乱斗/extension.js:3383-3395`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DoomsdayZombieGeneralSpec {
    pub id: &'static str,
    pub name_zh: &'static str,
    pub gender: &'static str,
    pub group: &'static str,
    pub max_hp: u32,
    pub skills: &'static [&'static str],
}

pub const JIANGSHIFZ_SKILLS: [&str; 5] = ["xunmeng", "zaibian", "ganran", "wansha", "paoxiao"];
pub const JIANGSHINJ_SKILLS: [&str; 5] = ["baozou", "wansha", "xueji", "shishi", "ganran"];

pub const DOOMSDAY_ZOMBIE_GENERALS: [DoomsdayZombieGeneralSpec; 2] = [
    DoomsdayZombieGeneralSpec {
        id: "jiangshifz",
        name_zh: "邪灵僵尸",
        gender: "male",
        group: "ye",
        max_hp: 5,
        skills: &JIANGSHIFZ_SKILLS,
    },
    DoomsdayZombieGeneralSpec {
        id: "jiangshinj",
        name_zh: "鬼影僵尸",
        gender: "female",
        group: "ye",
        max_hp: 3,
        skills: &JIANGSHINJ_SKILLS,
    },
];

/// `zuan_brawl` 配置项（`末日乱斗/extension.js:3360-3381`）：乱斗模式武将禁用策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZuanBrawlMode {
    /// 关闭（`"off"`）
    Off,
    /// 标准（`"standard"`）
    Standard,
    /// 随机（`"random"`）
    Random,
    /// 全部（`"all"`，默认值）
    All,
    /// 日常（`"daily"`）
    Daily,
}

pub const DEFAULT_ZUAN_BRAWL: &str = "all";
pub const ZUAN_BRAWL_MODES: [&str; 5] = ["off", "standard", "random", "all", "daily"];

impl ZuanBrawlMode {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Standard => "standard",
            Self::Random => "random",
            Self::All => "all",
            Self::Daily => "daily",
        }
    }
}

/// 校验 `zuan_brawl` 配置字符串是否合法（`"off" | "standard" | "random" | "all" | "daily"`）。
pub fn validate_zuan_brawl(raw: &str) -> bool {
    matches!(raw, "off" | "standard" | "random" | "all" | "daily")
}

/// 解析 `zuan_brawl` 配置（缺省取 `"all"`）。
pub fn parse_zuan_brawl(raw: Option<&str>) -> Result<ZuanBrawlMode, &'static str> {
    match raw.unwrap_or(DEFAULT_ZUAN_BRAWL) {
        "off" => Ok(ZuanBrawlMode::Off),
        "standard" => Ok(ZuanBrawlMode::Standard),
        "random" => Ok(ZuanBrawlMode::Random),
        "all" => Ok(ZuanBrawlMode::All),
        "daily" => Ok(ZuanBrawlMode::Daily),
        _ => Err("invalid zuan_brawl config: expected off|standard|random|all|daily"),
    }
}
