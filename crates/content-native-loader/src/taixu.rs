//! L11 太虚幻境（`archive/upstream/noname-liuli/noname/extension/太虚幻境/extension.js`）
//! 战役模式配置与单机肉鸽路线状态机。
//!
//! 上游扩展是一个 11,488 行的独立单机 PvE 战役模式，在开局拦截主循环并挂载专属战役地图、
//! 奇遇事件、祝福购买与侍灵绑定。原生层在 `content-native-loader::taixu` 中提供配置校验与
//! 确定性战役节点状态机 [`TaixuRunState`]，不污染标准身份/国战内核状态。

use serde_json::Value;

/// G0 盘点基数：`(skill_count, card_count, character_count, translate_count)`。
pub const TAIXU_COUNTS: (usize, usize, usize, usize) = (146, 26, 54, 152);

/// 太虚幻境扩展配置。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaixuConfig {
    pub enable_campaign: bool,
    pub season: String,
    pub spirit_enabled: bool,
    pub buff_limit: usize,
    pub starting_gold: u32,
}

impl Default for TaixuConfig {
    fn default() -> Self {
        Self {
            enable_campaign: true,
            season: "taixu_s1".to_string(),
            spirit_enabled: true,
            buff_limit: 6,
            starting_gold: 100,
        }
    }
}

/// 太虚幻境单次战役运行状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaixuRunState {
    pub season: String,
    pub node_index: usize,
    pub gold: u32,
    pub buffs: Vec<String>,
    pub spirits: Vec<String>,
    pub completed: bool,
}

impl TaixuRunState {
    pub fn from_config(cfg: &TaixuConfig) -> Self {
        Self {
            season: cfg.season.clone(),
            node_index: 0,
            gold: cfg.starting_gold,
            buffs: Vec::new(),
            spirits: Vec::new(),
            completed: false,
        }
    }
}

/// L11 太虚幻境错误类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaixuError {
    BadJson(String),
    MissingField(&'static str),
    InvalidField { field: &'static str, reason: String },
    InvalidCount { field: &'static str, got: usize, expected: usize },
    CampaignDisabled,
    SpiritDisabled,
    InsufficientGold { have: u32, need: u32 },
    BuffLimitReached { limit: usize },
    DuplicateBuff(String),
    DuplicateSpirit(String),
    AlreadyCompleted,
    UnknownCommand(String),
    SnapshotMalformed(String),
}

impl std::fmt::Display for TaixuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadJson(e) => write!(f, "太虚幻境 JSON 解析失败: {e}"),
            Self::MissingField(fld) => write!(f, "太虚幻境 缺少字段: {fld}"),
            Self::InvalidField { field, reason } => {
                write!(f, "太虚幻境 字段 {field} 非法: {reason}")
            }
            Self::InvalidCount { field, got, expected } => {
                write!(f, "太虚幻境 计数 {field} 不匹配: 实际 {got}, 期望 {expected}")
            }
            Self::CampaignDisabled => write!(f, "太虚幻境 战役模式未启用"),
            Self::SpiritDisabled => write!(f, "太虚幻境 侍灵系统未启用"),
            Self::InsufficientGold { have, need } => {
                write!(f, "太虚幻境 金币不足: 拥有 {have}, 需要 {need}")
            }
            Self::BuffLimitReached { limit } => {
                write!(f, "太虚幻境 祝福数量已达上限 {limit}")
            }
            Self::DuplicateBuff(b) => write!(f, "太虚幻境 重复购买祝福: {b}"),
            Self::DuplicateSpirit(s) => write!(f, "太虚幻境 重复绑定侍灵: {s}"),
            Self::AlreadyCompleted => write!(f, "太虚幻境 战役已通关"),
            Self::UnknownCommand(cmd) => write!(f, "太虚幻境 未知指令: {cmd}"),
            Self::SnapshotMalformed(reason) => write!(f, "太虚幻境 快照损坏: {reason}"),
        }
    }
}

impl std::error::Error for TaixuError {}

/// 解析 `taixu-upstream-liuli/assets/config.json` 或直接的 `default_config` 对象。
pub fn parse_taixu_config(raw: &str) -> Result<TaixuConfig, TaixuError> {
    let root: Value = serde_json::from_str(raw).map_err(|e| TaixuError::BadJson(e.to_string()))?;
    let root_obj = root
        .as_object()
        .ok_or_else(|| TaixuError::BadJson("根值必须为 JSON 对象".into()))?;

    let cfg_obj = if let Some(inner) = root_obj.get("default_config") {
        for (field, expected) in [
            ("skill_count", TAIXU_COUNTS.0),
            ("card_count", TAIXU_COUNTS.1),
            ("character_count", TAIXU_COUNTS.2),
            ("translate_count", TAIXU_COUNTS.3),
        ] {
            if let Some(v) = root_obj.get(field) {
                let got = v.as_u64().ok_or(TaixuError::MissingField(field))? as usize;
                if got != expected {
                    return Err(TaixuError::InvalidCount { field, got, expected });
                }
            }
        }
        inner
            .as_object()
            .ok_or(TaixuError::MissingField("default_config"))?
    } else {
        root_obj
    };

    let enable_campaign = cfg_obj
        .get("enable_campaign")
        .ok_or(TaixuError::MissingField("enable_campaign"))?
        .as_bool()
        .ok_or(TaixuError::InvalidField {
            field: "enable_campaign",
            reason: "必须为布尔值".into(),
        })?;

    let season = cfg_obj
        .get("season")
        .ok_or(TaixuError::MissingField("season"))?
        .as_str()
        .ok_or(TaixuError::InvalidField {
            field: "season",
            reason: "必须为字符串".into(),
        })?
        .trim()
        .to_string();
    if season.is_empty() {
        return Err(TaixuError::InvalidField {
            field: "season",
            reason: "赛季标识不能为空".into(),
        });
    }

    let spirit_enabled = cfg_obj
        .get("spirit_enabled")
        .ok_or(TaixuError::MissingField("spirit_enabled"))?
        .as_bool()
        .ok_or(TaixuError::InvalidField {
            field: "spirit_enabled",
            reason: "必须为布尔值".into(),
        })?;

    let buff_limit = cfg_obj
        .get("buff_limit")
        .ok_or(TaixuError::MissingField("buff_limit"))?
        .as_u64()
        .ok_or(TaixuError::InvalidField {
            field: "buff_limit",
            reason: "必须为非负整数".into(),
        })? as usize;
    if buff_limit == 0 || buff_limit > 32 {
        return Err(TaixuError::InvalidField {
            field: "buff_limit",
            reason: "必须在 1..=32 范围内".into(),
        });
    }

    let starting_gold = cfg_obj
        .get("starting_gold")
        .ok_or(TaixuError::MissingField("starting_gold"))?
        .as_u64()
        .ok_or(TaixuError::InvalidField {
            field: "starting_gold",
            reason: "必须为非负整数".into(),
        })?;
    if starting_gold > 100_000 {
        return Err(TaixuError::InvalidField {
            field: "starting_gold",
            reason: "超出上限 100000".into(),
        });
    }

    Ok(TaixuConfig {
        enable_campaign,
        season,
        spirit_enabled,
        buff_limit,
        starting_gold: starting_gold as u32,
    })
}

/// 序列化战役状态为确定性单行快照。
pub fn snapshot_taixu(state: &TaixuRunState) -> String {
    let buffs = if state.buffs.is_empty() {
        "-".to_string()
    } else {
        state.buffs.join(",")
    };
    let spirits = if state.spirits.is_empty() {
        "-".to_string()
    } else {
        state.spirits.join(",")
    };
    format!(
        "season={} | node={} | gold={} | buffs={} | spirits={} | completed={}",
        state.season,
        state.node_index,
        state.gold,
        buffs,
        spirits,
        u8::from(state.completed)
    )
}

/// 从单行快照恢复战役状态。
pub fn restore_taixu(snap: &str) -> Result<TaixuRunState, TaixuError> {
    let mut state = TaixuRunState::from_config(&TaixuConfig::default());
    let mut seen = 0usize;
    for part in snap.split('|') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (k, v) = part
            .split_once('=')
            .ok_or_else(|| TaixuError::SnapshotMalformed(part.to_string()))?;
        let k = k.trim();
        let v = v.trim();
        match k {
            "season" => {
                if v.is_empty() {
                    return Err(TaixuError::SnapshotMalformed("season 为空".into()));
                }
                state.season = v.to_string();
            }
            "node" => {
                state.node_index = v
                    .parse::<usize>()
                    .map_err(|_| TaixuError::SnapshotMalformed(format!("非法 node: {v}")))?;
            }
            "gold" => {
                state.gold = v
                    .parse::<u32>()
                    .map_err(|_| TaixuError::SnapshotMalformed(format!("非法 gold: {v}")))?;
            }
            "buffs" => {
                state.buffs = if v == "-" || v.is_empty() {
                    Vec::new()
                } else {
                    v.split(',').map(|s| s.trim().to_string()).collect()
                };
            }
            "spirits" => {
                state.spirits = if v == "-" || v.is_empty() {
                    Vec::new()
                } else {
                    v.split(',').map(|s| s.trim().to_string()).collect()
                };
            }
            "completed" => {
                state.completed = match v {
                    "1" | "true" => true,
                    "0" | "false" => false,
                    _ => return Err(TaixuError::SnapshotMalformed(format!("非法 completed: {v}"))),
                };
            }
            other => return Err(TaixuError::SnapshotMalformed(other.to_string())),
        }
        seen += 1;
    }
    if seen < 6 {
        return Err(TaixuError::SnapshotMalformed("字段不足 6 项".into()));
    }
    Ok(state)
}

/// 执行太虚幻境状态机单行指令：
/// - `start`：按配置初始化战役状态
/// - `advance <reward_gold>`：推进一关并获得金币（第 5 关通关）
/// - `buy_buff <buff_id> <cost>`：消耗金币购买祝福
/// - `bind_spirit <spirit_id>`：绑定侍灵
/// - `snapshot`：导出单行快照
/// - `restore <snap>`：从单行快照恢复
pub fn apply_taixu_line(
    cfg: &TaixuConfig,
    state: &mut TaixuRunState,
    line: &str,
) -> Result<String, TaixuError> {
    let trimmed = line.trim();
    if trimmed == "snapshot" {
        return Ok(snapshot_taixu(state));
    }
    if trimmed == "start" {
        if !cfg.enable_campaign {
            return Err(TaixuError::CampaignDisabled);
        }
        *state = TaixuRunState::from_config(cfg);
        return Ok(snapshot_taixu(state));
    }
    let (cmd, rest) = trimmed
        .split_once(' ')
        .ok_or_else(|| TaixuError::UnknownCommand(trimmed.to_string()))?;
    let rest = rest.trim();
    match cmd {
        "advance" => {
            if !cfg.enable_campaign {
                return Err(TaixuError::CampaignDisabled);
            }
            if state.completed {
                return Err(TaixuError::AlreadyCompleted);
            }
            let reward = rest.parse::<u32>().map_err(|_| TaixuError::InvalidField {
                field: "reward_gold",
                reason: "必须为非负整数".into(),
            })?;
            state.node_index += 1;
            state.gold = state.gold.saturating_add(reward);
            if state.node_index >= 5 {
                state.completed = true;
            }
            Ok(snapshot_taixu(state))
        }
        "buy_buff" => {
            if !cfg.enable_campaign {
                return Err(TaixuError::CampaignDisabled);
            }
            let (buff_id, cost_str) = rest.split_once(' ').ok_or(TaixuError::InvalidField {
                field: "buy_buff",
                reason: "格式为 buy_buff <id> <cost>".into(),
            })?;
            let buff_id = buff_id.trim();
            let cost = cost_str
                .trim()
                .parse::<u32>()
                .map_err(|_| TaixuError::InvalidField {
                    field: "cost",
                    reason: "必须为非负整数".into(),
                })?;
            if state.buffs.len() >= cfg.buff_limit {
                return Err(TaixuError::BuffLimitReached {
                    limit: cfg.buff_limit,
                });
            }
            if state.buffs.iter().any(|b| b == buff_id) {
                return Err(TaixuError::DuplicateBuff(buff_id.to_string()));
            }
            if state.gold < cost {
                return Err(TaixuError::InsufficientGold {
                    have: state.gold,
                    need: cost,
                });
            }
            state.gold -= cost;
            state.buffs.push(buff_id.to_string());
            Ok(snapshot_taixu(state))
        }
        "bind_spirit" => {
            if !cfg.enable_campaign {
                return Err(TaixuError::CampaignDisabled);
            }
            if !cfg.spirit_enabled {
                return Err(TaixuError::SpiritDisabled);
            }
            if rest.is_empty() {
                return Err(TaixuError::InvalidField {
                    field: "spirit_id",
                    reason: "不能为空".into(),
                });
            }
            if state.spirits.iter().any(|s| s == rest) {
                return Err(TaixuError::DuplicateSpirit(rest.to_string()));
            }
            state.spirits.push(rest.to_string());
            Ok(snapshot_taixu(state))
        }
        "restore" => {
            *state = restore_taixu(rest)?;
            Ok(snapshot_taixu(state))
        }
        other => Err(TaixuError::UnknownCommand(other.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taixu_config_and_state_machine() {
        let json = r#"{
            "extension_name": "太虚幻境",
            "source": "noname-liuli",
            "upstream_path": "archive/upstream/noname-liuli/extension/太虚幻境/extension.js",
            "default_config": {
                "enable_campaign": true,
                "season": "taixu_s1",
                "spirit_enabled": true,
                "buff_limit": 6,
                "starting_gold": 100
            },
            "skill_count": 146,
            "card_count": 26,
            "character_count": 54,
            "translate_count": 152
        }"#;
        let cfg = parse_taixu_config(json).expect("valid config");
        let mut st = TaixuRunState::from_config(&cfg);
        apply_taixu_line(&cfg, &mut st, "start").unwrap();
        apply_taixu_line(&cfg, &mut st, "buy_buff tx_sharp_blade 40").unwrap();
        apply_taixu_line(&cfg, &mut st, "bind_spirit spirit_qilin").unwrap();
        apply_taixu_line(&cfg, &mut st, "advance 25").unwrap();
        assert_eq!(st.gold, 85);
        assert_eq!(st.node_index, 1);

        let snap = apply_taixu_line(&cfg, &mut st, "snapshot").unwrap();
        let mut restored = TaixuRunState::from_config(&cfg);
        apply_taixu_line(&cfg, &mut restored, &format!("restore {snap}")).unwrap();
        assert_eq!(restored, st);
    }
}
