//! L14 手杀ui（`archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js`）
//! 布局配置解析、87+5 技能分类与快照模型（独立于 `L15 手杀全扩` 的 `shousha.rs`）。
//!
//! 上游扩展含 92 个 `lib.skill` 条目：其中 87 个在 `lbtn/main1.js` 与 `skill/main1.js` 中
//! 仅用于手杀底部操作栏、手牌折叠与技能按钮停靠渲染；另外 5 个在顶层 `extension.js` 中为
//! 特定武将击杀/死亡弹图与气泡台词轮播特效。全部 92 个技能均无规则语义。

use serde_json::Value;

/// 上游 `extension.js` 顶层定义的 5 个纯视觉/台词特效技能。
pub const SHOUSHA_UI_VISUAL_FX_SKILLS: [&str; 5] = [
    "_jd_ddxyA",
    "_jd_ddxyB",
    "_jd_ddxyC",
    "_jd_dfsk",
    "_wmkzSayChange",
];

/// G0 盘点基数：`(skill_count, ui_hook_skill_count, visual_fx_skill_count, card_count, character_count, translate_count)`。
pub const SHOUSHA_UI_COUNTS: (usize, usize, usize, usize, usize, usize) =
    (92, 87, 5, 122, 103, 108);

/// L14 手杀ui 配置。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShoushaUiConfig {
    pub player_frame: String,
    pub card_layout: String,
    pub lbtn_enabled: bool,
    pub skill_dock: String,
    pub popup_image: bool,
    pub say_bubble: bool,
}

impl Default for ShoushaUiConfig {
    fn default() -> Self {
        Self {
            player_frame: "mobile".to_string(),
            card_layout: "right".to_string(),
            lbtn_enabled: true,
            skill_dock: "bottom_right".to_string(),
            popup_image: true,
            say_bubble: true,
        }
    }
}

/// L14 手杀ui 错误类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShoushaUiError {
    BadJson(String),
    MissingField(&'static str),
    InvalidField { field: &'static str, reason: String },
    InvalidCount { field: &'static str, got: usize, expected: usize },
    UnknownCommand(String),
    SnapshotMalformed(String),
}

impl std::fmt::Display for ShoushaUiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadJson(e) => write!(f, "手杀ui JSON 解析失败: {e}"),
            Self::MissingField(fld) => write!(f, "手杀ui 缺少字段: {fld}"),
            Self::InvalidField { field, reason } => {
                write!(f, "手杀ui 字段 {field} 非法: {reason}")
            }
            Self::InvalidCount { field, got, expected } => {
                write!(f, "手杀ui 计数 {field} 不匹配: 实际 {got}, 期望 {expected}")
            }
            Self::UnknownCommand(cmd) => write!(f, "手杀ui 未知指令: {cmd}"),
            Self::SnapshotMalformed(reason) => write!(f, "手杀ui 快照损坏: {reason}"),
        }
    }
}

impl std::error::Error for ShoushaUiError {}

/// 解析 `shousha-ui-upstream-liuli/assets/config.json` 或直接的 `default_config` 对象。
pub fn parse_shousha_ui_config(raw: &str) -> Result<ShoushaUiConfig, ShoushaUiError> {
    let root: Value =
        serde_json::from_str(raw).map_err(|e| ShoushaUiError::BadJson(e.to_string()))?;
    let root_obj = root
        .as_object()
        .ok_or_else(|| ShoushaUiError::BadJson("根值必须为 JSON 对象".into()))?;

    let cfg_obj = if let Some(inner) = root_obj.get("default_config") {
        for (field, expected) in [
            ("skill_count", SHOUSHA_UI_COUNTS.0),
            ("ui_hook_skill_count", SHOUSHA_UI_COUNTS.1),
            ("visual_fx_skill_count", SHOUSHA_UI_COUNTS.2),
            ("card_count", SHOUSHA_UI_COUNTS.3),
            ("character_count", SHOUSHA_UI_COUNTS.4),
            ("translate_count", SHOUSHA_UI_COUNTS.5),
        ] {
            if let Some(v) = root_obj.get(field) {
                let got = v.as_u64().ok_or(ShoushaUiError::MissingField(field))? as usize;
                if got != expected {
                    return Err(ShoushaUiError::InvalidCount {
                        field,
                        got,
                        expected,
                    });
                }
            }
        }
        inner
            .as_object()
            .ok_or(ShoushaUiError::MissingField("default_config"))?
    } else {
        root_obj
    };

    let player_frame = cfg_obj
        .get("player_frame")
        .ok_or(ShoushaUiError::MissingField("player_frame"))?
        .as_str()
        .ok_or(ShoushaUiError::InvalidField {
            field: "player_frame",
            reason: "必须为字符串".into(),
        })?;
    if !matches!(player_frame, "mobile" | "wide" | "classic") {
        return Err(ShoushaUiError::InvalidField {
            field: "player_frame",
            reason: format!("未知座位框样式 {player_frame}"),
        });
    }

    let card_layout = cfg_obj
        .get("card_layout")
        .ok_or(ShoushaUiError::MissingField("card_layout"))?
        .as_str()
        .ok_or(ShoushaUiError::InvalidField {
            field: "card_layout",
            reason: "必须为字符串".into(),
        })?;
    if !matches!(card_layout, "right" | "center" | "left") {
        return Err(ShoushaUiError::InvalidField {
            field: "card_layout",
            reason: format!("未知手牌布局 {card_layout}"),
        });
    }

    let skill_dock = cfg_obj
        .get("skill_dock")
        .ok_or(ShoushaUiError::MissingField("skill_dock"))?
        .as_str()
        .ok_or(ShoushaUiError::InvalidField {
            field: "skill_dock",
            reason: "必须为字符串".into(),
        })?;
    if !matches!(skill_dock, "bottom_right" | "bottom_left" | "floating") {
        return Err(ShoushaUiError::InvalidField {
            field: "skill_dock",
            reason: format!("未知技能停靠位 {skill_dock}"),
        });
    }

    let read_bool = |field: &'static str| -> Result<bool, ShoushaUiError> {
        cfg_obj
            .get(field)
            .ok_or(ShoushaUiError::MissingField(field))?
            .as_bool()
            .ok_or(ShoushaUiError::InvalidField {
                field,
                reason: "必须为布尔值".into(),
            })
    };

    Ok(ShoushaUiConfig {
        player_frame: player_frame.to_string(),
        card_layout: card_layout.to_string(),
        lbtn_enabled: read_bool("lbtn_enabled")?,
        skill_dock: skill_dock.to_string(),
        popup_image: read_bool("popup_image")?,
        say_bubble: read_bool("say_bubble")?,
    })
}

/// 序列化为确定性单行快照。
pub fn snapshot_shousha_ui(cfg: &ShoushaUiConfig) -> String {
    format!(
        "player_frame={} | card_layout={} | lbtn_enabled={} | skill_dock={} | popup_image={} | say_bubble={}",
        cfg.player_frame,
        cfg.card_layout,
        u8::from(cfg.lbtn_enabled),
        cfg.skill_dock,
        u8::from(cfg.popup_image),
        u8::from(cfg.say_bubble),
    )
}

/// 从单行快照恢复配置。
pub fn restore_shousha_ui(snap: &str) -> Result<ShoushaUiConfig, ShoushaUiError> {
    let mut cfg = ShoushaUiConfig::default();
    let mut seen = 0usize;
    for part in snap.split('|') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (k, v) = part
            .split_once('=')
            .ok_or_else(|| ShoushaUiError::SnapshotMalformed(part.to_string()))?;
        let k = k.trim();
        let v = v.trim();
        let parse_bit = |field: &'static str, s: &str| match s {
            "1" | "true" => Ok(true),
            "0" | "false" => Ok(false),
            _ => Err(ShoushaUiError::InvalidField {
                field,
                reason: format!("非法布尔值 {s}"),
            }),
        };
        match k {
            "player_frame" => {
                if !matches!(v, "mobile" | "wide" | "classic") {
                    return Err(ShoushaUiError::InvalidField {
                        field: "player_frame",
                        reason: v.to_string(),
                    });
                }
                cfg.player_frame = v.to_string();
            }
            "card_layout" => {
                if !matches!(v, "right" | "center" | "left") {
                    return Err(ShoushaUiError::InvalidField {
                        field: "card_layout",
                        reason: v.to_string(),
                    });
                }
                cfg.card_layout = v.to_string();
            }
            "lbtn_enabled" => cfg.lbtn_enabled = parse_bit("lbtn_enabled", v)?,
            "skill_dock" => {
                if !matches!(v, "bottom_right" | "bottom_left" | "floating") {
                    return Err(ShoushaUiError::InvalidField {
                        field: "skill_dock",
                        reason: v.to_string(),
                    });
                }
                cfg.skill_dock = v.to_string();
            }
            "popup_image" => cfg.popup_image = parse_bit("popup_image", v)?,
            "say_bubble" => cfg.say_bubble = parse_bit("say_bubble", v)?,
            other => return Err(ShoushaUiError::SnapshotMalformed(other.to_string())),
        }
        seen += 1;
    }
    if seen < 6 {
        return Err(ShoushaUiError::SnapshotMalformed("字段不足 6 项".into()));
    }
    Ok(cfg)
}

/// 执行单行诊断命令（`config <json>` / `classify_skill <id>` / `snapshot` / `restore <snap>`）。
pub fn apply_shousha_ui_line(
    cfg: &mut ShoushaUiConfig,
    line: &str,
) -> Result<String, ShoushaUiError> {
    let trimmed = line.trim();
    if trimmed == "snapshot" {
        return Ok(snapshot_shousha_ui(cfg));
    }
    let (cmd, rest) = trimmed
        .split_once(' ')
        .ok_or_else(|| ShoushaUiError::UnknownCommand(trimmed.to_string()))?;
    let rest = rest.trim();
    match cmd {
        "config" => {
            *cfg = parse_shousha_ui_config(rest)?;
            Ok(snapshot_shousha_ui(cfg))
        }
        "classify_skill" => {
            if SHOUSHA_UI_VISUAL_FX_SKILLS.contains(&rest) {
                let enabled = if rest == "_wmkzSayChange" {
                    cfg.say_bubble
                } else {
                    cfg.popup_image
                };
                Ok(format!(
                    "skill={rest} | class=visual_fx | enabled={} | rule_effect=0",
                    u8::from(enabled)
                ))
            } else {
                Ok(format!(
                    "skill={rest} | class=ui_hook | enabled={} | rule_effect=0",
                    u8::from(cfg.lbtn_enabled)
                ))
            }
        }
        "restore" => {
            *cfg = restore_shousha_ui(rest)?;
            Ok(snapshot_shousha_ui(cfg))
        }
        other => Err(ShoushaUiError::UnknownCommand(other.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_classify_shousha_ui() {
        let json = r#"{
            "extension_name": "手杀ui",
            "source": "noname-liuli",
            "upstream_path": "archive/upstream/noname-liuli/extension/手杀ui/extension.js",
            "default_config": {
                "player_frame": "mobile",
                "card_layout": "right",
                "lbtn_enabled": true,
                "skill_dock": "bottom_right",
                "popup_image": true,
                "say_bubble": true
            },
            "skill_count": 92,
            "ui_hook_skill_count": 87,
            "visual_fx_skill_count": 5,
            "card_count": 122,
            "character_count": 103,
            "translate_count": 108
        }"#;
        let mut cfg = parse_shousha_ui_config(json).expect("valid config");
        assert_eq!(cfg, ShoushaUiConfig::default());
        assert_eq!(
            apply_shousha_ui_line(&mut cfg, "classify_skill _jd_dfsk").unwrap(),
            "skill=_jd_dfsk | class=visual_fx | enabled=1 | rule_effect=0"
        );
        assert_eq!(
            apply_shousha_ui_line(&mut cfg, "classify_skill _lbtn_sort_hand").unwrap(),
            "skill=_lbtn_sort_hand | class=ui_hook | enabled=1 | rule_effect=0"
        );
    }
}
