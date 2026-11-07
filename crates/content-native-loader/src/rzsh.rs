//! L12 如真似幻（noname-liuli）的可移植决策。
//!
//! 上游是**纯渲染/大厅**扩展（596 KB 主文件 + 5.9 MB 资源脚本）：`lib.skill`
//! 0 个、`lib.card` 2 处（仅牌表引用）、`lib.character` 8 处（展示用），
//! 无 `extension_如真似幻_*` 配置键；59 个函数中绝大多数直接操作 DOM/WebGL。
//! 本模块只迁移**语义明确的纯函数**：
//! - `numtoroma`：1–6 → 罗马数字（大厅段位/关卡编号展示）；
//! - `getRandomSanguoText` / `getRandomTip`：从固定文案表随机取一条
//!   （数据表随模块迁移；随机性由调用方给定 `roll`，可复现）。
//!
//! 渲染/动画/大厅流程（WebGL、spine、抽屉、按钮）**不迁移**（登记差异）。
//! 本模块不读写文件、不请求网络、不 eval。

pub const SANGUO_TEXTS: [&str; 11] = [
    "三国杀是一款流行的桌面卡牌游戏，基于三国历史背景。",
    "在三国杀中，玩家需要策略地使用各种角色卡牌来击败对手。",
    "诸葛亮、曹操和刘备是三国杀中的著名角色。",
    "游戏中的卡牌包括杀、闪、桃等各种不同的功能。",
    "每位角色都有独特的技能和特点，增加了游戏的变化性。",
    "三国杀的策略性和战术性使其成为一款受欢迎的卡牌游戏。",
    "在三国杀中，胜利需要巧妙地使用卡牌和角色技能。",
    "游戏中的合作和背叛元素增加了战局的紧张感。",
    "使用你喜欢的武将，积累武将经验，可以获得炫酷的武将表现效果",
    "付费购买的武将及招募的武将均可以分解成一定数量的将魂",
    "开通会员后，可以加速等级的提升哦",
];

pub const TIPS: [&str; 4] = [
    "寻找对手中,不要着急哦~",
    "回合结束时,能保留的手牌数不能超过当前体力值",
    "[万箭齐发] : ( 锦囊 ) 所有人,展现你灵巧的身姿吧",
    "[无懈可击] : ( 锦囊 ) 休想得逞",
];

/// `numtoroma`：1–6 → 罗马数字；其余返回 `None`。
pub fn roman(num: u32) -> Option<&'static str> {
    match num {
        1 => Some("Ⅰ"),
        2 => Some("Ⅱ"),
        3 => Some("Ⅲ"),
        4 => Some("Ⅳ"),
        5 => Some("Ⅴ"),
        6 => Some("Ⅵ"),
        _ => None,
    }
}

/// `getRandomSanguoText`：按 `roll` 取文案（表内取模；空表返回 `None`）。
pub fn sanguo_text(roll: usize) -> Option<&'static str> {
    if SANGUO_TEXTS.is_empty() {
        return None;
    }
    SANGUO_TEXTS.get(roll % SANGUO_TEXTS.len()).copied()
}

/// `getRandomTip`：按 `roll` 取提示（表内取模；空表返回 `None`）。
pub fn tip(roll: usize) -> Option<&'static str> {
    if TIPS.is_empty() {
        return None;
    }
    TIPS.get(roll % TIPS.len()).copied()
}

/// G0 盘点基数：`(skill_count, card_count, character_count, translate_count)`。
pub const RZSH_COUNTS: (usize, usize, usize, usize) = (0, 92, 8, 8);

/// L12 如真似幻配置。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RzshConfig {
    pub theme: String,
    pub bgm: String,
    pub stage_effect: bool,
    pub voice_pack: String,
    pub ui_scale: u32,
}

impl Default for RzshConfig {
    fn default() -> Self {
        Self {
            theme: "szn".to_string(),
            bgm: "default".to_string(),
            stage_effect: true,
            voice_pack: "standard".to_string(),
            ui_scale: 100,
        }
    }
}

/// L12 如真似幻配置与指令错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RzshError {
    BadJson(String),
    MissingField(&'static str),
    InvalidField { field: &'static str, reason: String },
    InvalidCount { field: &'static str, got: usize, expected: usize },
    UnknownCommand(String),
    SnapshotMalformed(String),
}

impl std::fmt::Display for RzshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadJson(e) => write!(f, "如真似幻 JSON 解析失败: {e}"),
            Self::MissingField(fld) => write!(f, "如真似幻 缺少字段: {fld}"),
            Self::InvalidField { field, reason } => {
                write!(f, "如真似幻 字段 {field} 非法: {reason}")
            }
            Self::InvalidCount { field, got, expected } => {
                write!(f, "如真似幻 计数 {field} 不匹配: 实际 {got}, 期望 {expected}")
            }
            Self::UnknownCommand(cmd) => write!(f, "如真似幻 未知指令: {cmd}"),
            Self::SnapshotMalformed(reason) => write!(f, "如真似幻 快照损坏: {reason}"),
        }
    }
}

impl std::error::Error for RzshError {}

/// 解析 `rzsh-upstream-liuli/assets/config.json` 或直接的 `default_config` 对象。
pub fn parse_rzsh_config(raw: &str) -> Result<RzshConfig, RzshError> {
    let root: serde_json::Value =
        serde_json::from_str(raw).map_err(|e| RzshError::BadJson(e.to_string()))?;
    let root_obj = root
        .as_object()
        .ok_or_else(|| RzshError::BadJson("根值必须为 JSON 对象".into()))?;

    let cfg_obj = if let Some(inner) = root_obj.get("default_config") {
        for (field, expected) in [
            ("skill_count", RZSH_COUNTS.0),
            ("card_count", RZSH_COUNTS.1),
            ("character_count", RZSH_COUNTS.2),
            ("translate_count", RZSH_COUNTS.3),
        ] {
            if let Some(v) = root_obj.get(field) {
                let got = v.as_u64().ok_or(RzshError::MissingField(field))? as usize;
                if got != expected {
                    return Err(RzshError::InvalidCount {
                        field,
                        got,
                        expected,
                    });
                }
            }
        }
        inner
            .as_object()
            .ok_or(RzshError::MissingField("default_config"))?
    } else {
        root_obj
    };

    let theme = cfg_obj
        .get("theme")
        .ok_or(RzshError::MissingField("theme"))?
        .as_str()
        .ok_or(RzshError::InvalidField {
            field: "theme",
            reason: "必须为字符串".into(),
        })?;
    if !matches!(theme, "szn" | "ol" | "mobile" | "classic") {
        return Err(RzshError::InvalidField {
            field: "theme",
            reason: format!("未知主题 {theme}"),
        });
    }

    let bgm = cfg_obj
        .get("bgm")
        .ok_or(RzshError::MissingField("bgm"))?
        .as_str()
        .ok_or(RzshError::InvalidField {
            field: "bgm",
            reason: "必须为字符串".into(),
        })?;
    if bgm.trim().is_empty() {
        return Err(RzshError::InvalidField {
            field: "bgm",
            reason: "不能为空".into(),
        });
    }

    let stage_effect = cfg_obj
        .get("stage_effect")
        .ok_or(RzshError::MissingField("stage_effect"))?
        .as_bool()
        .ok_or(RzshError::InvalidField {
            field: "stage_effect",
            reason: "必须为布尔值".into(),
        })?;

    let voice_pack = cfg_obj
        .get("voice_pack")
        .ok_or(RzshError::MissingField("voice_pack"))?
        .as_str()
        .ok_or(RzshError::InvalidField {
            field: "voice_pack",
            reason: "必须为字符串".into(),
        })?;
    if voice_pack.trim().is_empty() {
        return Err(RzshError::InvalidField {
            field: "voice_pack",
            reason: "不能为空".into(),
        });
    }

    let ui_scale = cfg_obj
        .get("ui_scale")
        .ok_or(RzshError::MissingField("ui_scale"))?
        .as_u64()
        .ok_or(RzshError::InvalidField {
            field: "ui_scale",
            reason: "必须为整数".into(),
        })? as u32;
    if !(50..=200).contains(&ui_scale) {
        return Err(RzshError::InvalidField {
            field: "ui_scale",
            reason: "必须在 50..=200 范围内".into(),
        });
    }

    Ok(RzshConfig {
        theme: theme.to_string(),
        bgm: bgm.to_string(),
        stage_effect,
        voice_pack: voice_pack.to_string(),
        ui_scale,
    })
}

/// 序列化为确定性单行快照。
pub fn snapshot_rzsh(cfg: &RzshConfig) -> String {
    format!(
        "theme={} | bgm={} | stage_effect={} | voice_pack={} | ui_scale={}",
        cfg.theme,
        cfg.bgm,
        u8::from(cfg.stage_effect),
        cfg.voice_pack,
        cfg.ui_scale,
    )
}

/// 从单行快照恢复配置。
pub fn restore_rzsh(snap: &str) -> Result<RzshConfig, RzshError> {
    let mut cfg = RzshConfig::default();
    let mut seen = 0usize;
    for part in snap.split('|') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (k, v) = part
            .split_once('=')
            .ok_or_else(|| RzshError::SnapshotMalformed(part.to_string()))?;
        let k = k.trim();
        let v = v.trim();
        match k {
            "theme" => {
                if !matches!(v, "szn" | "ol" | "mobile" | "classic") {
                    return Err(RzshError::InvalidField {
                        field: "theme",
                        reason: v.to_string(),
                    });
                }
                cfg.theme = v.to_string();
            }
            "bgm" => cfg.bgm = v.to_string(),
            "stage_effect" => {
                cfg.stage_effect = match v {
                    "1" | "true" => true,
                    "0" | "false" => false,
                    _ => {
                        return Err(RzshError::InvalidField {
                            field: "stage_effect",
                            reason: v.to_string(),
                        })
                    }
                };
            }
            "voice_pack" => cfg.voice_pack = v.to_string(),
            "ui_scale" => {
                let s = v.parse::<u32>().map_err(|_| RzshError::InvalidField {
                    field: "ui_scale",
                    reason: v.to_string(),
                })?;
                if !(50..=200).contains(&s) {
                    return Err(RzshError::InvalidField {
                        field: "ui_scale",
                        reason: v.to_string(),
                    });
                }
                cfg.ui_scale = s;
            }
            other => return Err(RzshError::SnapshotMalformed(other.to_string())),
        }
        seen += 1;
    }
    if seen < 5 {
        return Err(RzshError::SnapshotMalformed("字段不足 5 项".into()));
    }
    Ok(cfg)
}

/// 执行单行诊断命令（`config <json>` / `roman <n>` / `sanguo <roll>` / `tip <roll>` / `snapshot` / `restore <snap>`）。
pub fn apply_rzsh_line(cfg: &mut RzshConfig, line: &str) -> Result<String, RzshError> {
    let trimmed = line.trim();
    if trimmed == "snapshot" {
        return Ok(snapshot_rzsh(cfg));
    }
    let (cmd, rest) = trimmed
        .split_once(' ')
        .ok_or_else(|| RzshError::UnknownCommand(trimmed.to_string()))?;
    let rest = rest.trim();
    match cmd {
        "config" => {
            *cfg = parse_rzsh_config(rest)?;
            Ok(snapshot_rzsh(cfg))
        }
        "roman" => {
            let n = rest.parse::<u32>().map_err(|_| RzshError::InvalidField {
                field: "roman",
                reason: rest.to_string(),
            })?;
            Ok(roman(n).unwrap_or("-").to_string())
        }
        "sanguo" => {
            let roll = rest.parse::<usize>().map_err(|_| RzshError::InvalidField {
                field: "roll",
                reason: rest.to_string(),
            })?;
            Ok(sanguo_text(roll).unwrap_or("-").to_string())
        }
        "tip" => {
            let roll = rest.parse::<usize>().map_err(|_| RzshError::InvalidField {
                field: "roll",
                reason: rest.to_string(),
            })?;
            Ok(tip(roll).unwrap_or("-").to_string())
        }
        "restore" => {
            *cfg = restore_rzsh(rest)?;
            Ok(snapshot_rzsh(cfg))
        }
        other => Err(RzshError::UnknownCommand(other.to_string())),
    }
}

