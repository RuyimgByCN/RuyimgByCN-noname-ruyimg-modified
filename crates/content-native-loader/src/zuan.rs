//! L25 祖安武将（`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js`）
//! 配置门禁、联机房间准入校验与快照模型。
//!
//! 上游扩展混合了本地扩展编辑器/代码生成器（直接调用 `game.writeFile` 覆写本地文件）与
//! 恶搞变体武将包。原生层通过安全门禁禁止在非离线调试环境下开启编辑器，并将联机对局与
//! 标准规则对局门禁显式化。

use serde_json::Value;

/// G0 盘点基数：`(skill_count, card_count, character_count, translate_count)`。
pub const ZUAN_COUNTS: (usize, usize, usize, usize) = (244, 190, 400, 239);

/// L25 祖安武将配置。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZuanConfig {
    pub enable_editor: bool,
    pub rule_variant: bool,
    pub offline_debug_only: bool,
    pub taunt_pack: bool,
}

impl Default for ZuanConfig {
    fn default() -> Self {
        Self {
            enable_editor: false,
            rule_variant: false,
            offline_debug_only: true,
            taunt_pack: true,
        }
    }
}

/// L25 祖安武将错误类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZuanError {
    BadJson(String),
    MissingField(&'static str),
    InvalidField { field: &'static str, reason: String },
    InvalidCount { field: &'static str, got: usize, expected: usize },
    UnsafeEditorOutsideOfflineDebug,
    OnlineRoomRejected(&'static str),
    UnknownCommand(String),
    SnapshotMalformed(String),
}

impl std::fmt::Display for ZuanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadJson(e) => write!(f, "祖安武将 JSON 解析失败: {e}"),
            Self::MissingField(fld) => write!(f, "祖安武将 缺少字段: {fld}"),
            Self::InvalidField { field, reason } => {
                write!(f, "祖安武将 字段 {field} 非法: {reason}")
            }
            Self::InvalidCount { field, got, expected } => {
                write!(f, "祖安武将 计数 {field} 不匹配: 实际 {got}, 期望 {expected}")
            }
            Self::UnsafeEditorOutsideOfflineDebug => {
                write!(f, "祖安武将 拒绝在非离线调试模式下启用 enable_editor")
            }
            Self::OnlineRoomRejected(reason) => {
                write!(f, "祖安武将 联机房间准入拒绝: {reason}")
            }
            Self::UnknownCommand(cmd) => write!(f, "祖安武将 未知指令: {cmd}"),
            Self::SnapshotMalformed(reason) => write!(f, "祖安武将 快照损坏: {reason}"),
        }
    }
}

impl std::error::Error for ZuanError {}

/// 解析 `zuan-upstream-liuli/assets/config.json` 或直接的 `default_config` 对象。
/// 当 `enable_editor: true` 且 `offline_debug_only: false` 时拒绝装载。
pub fn parse_zuan_config(raw: &str) -> Result<ZuanConfig, ZuanError> {
    let root: Value = serde_json::from_str(raw).map_err(|e| ZuanError::BadJson(e.to_string()))?;
    let root_obj = root
        .as_object()
        .ok_or_else(|| ZuanError::BadJson("根值必须为 JSON 对象".into()))?;

    let cfg_obj = if let Some(inner) = root_obj.get("default_config") {
        for (field, expected) in [
            ("skill_count", ZUAN_COUNTS.0),
            ("card_count", ZUAN_COUNTS.1),
            ("character_count", ZUAN_COUNTS.2),
            ("translate_count", ZUAN_COUNTS.3),
        ] {
            if let Some(v) = root_obj.get(field) {
                let got = v.as_u64().ok_or(ZuanError::MissingField(field))? as usize;
                if got != expected {
                    return Err(ZuanError::InvalidCount {
                        field,
                        got,
                        expected,
                    });
                }
            }
        }
        inner
            .as_object()
            .ok_or(ZuanError::MissingField("default_config"))?
    } else {
        root_obj
    };

    let read_bool = |field: &'static str| -> Result<bool, ZuanError> {
        cfg_obj
            .get(field)
            .ok_or(ZuanError::MissingField(field))?
            .as_bool()
            .ok_or(ZuanError::InvalidField {
                field,
                reason: "必须为布尔值".into(),
            })
    };

    let enable_editor = read_bool("enable_editor")?;
    let rule_variant = read_bool("rule_variant")?;
    let offline_debug_only = read_bool("offline_debug_only")?;
    let taunt_pack = read_bool("taunt_pack")?;

    if enable_editor && !offline_debug_only {
        return Err(ZuanError::UnsafeEditorOutsideOfflineDebug);
    }

    Ok(ZuanConfig {
        enable_editor,
        rule_variant,
        offline_debug_only,
        taunt_pack,
    })
}

/// 序列化为确定性单行快照。
pub fn snapshot_zuan(cfg: &ZuanConfig) -> String {
    format!(
        "enable_editor={} | rule_variant={} | offline_debug_only={} | taunt_pack={}",
        u8::from(cfg.enable_editor),
        u8::from(cfg.rule_variant),
        u8::from(cfg.offline_debug_only),
        u8::from(cfg.taunt_pack),
    )
}

/// 从单行快照恢复配置。
pub fn restore_zuan(snap: &str) -> Result<ZuanConfig, ZuanError> {
    let mut cfg = ZuanConfig::default();
    let mut seen = 0usize;
    for part in snap.split('|') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (k, v) = part
            .split_once('=')
            .ok_or_else(|| ZuanError::SnapshotMalformed(part.to_string()))?;
        let k = k.trim();
        let v = v.trim();
        let parse_bit = |field: &'static str, s: &str| match s {
            "1" | "true" => Ok(true),
            "0" | "false" => Ok(false),
            _ => Err(ZuanError::InvalidField {
                field,
                reason: format!("非法布尔值 {s}"),
            }),
        };
        match k {
            "enable_editor" => cfg.enable_editor = parse_bit("enable_editor", v)?,
            "rule_variant" => cfg.rule_variant = parse_bit("rule_variant", v)?,
            "offline_debug_only" => cfg.offline_debug_only = parse_bit("offline_debug_only", v)?,
            "taunt_pack" => cfg.taunt_pack = parse_bit("taunt_pack", v)?,
            other => return Err(ZuanError::SnapshotMalformed(other.to_string())),
        }
        seen += 1;
    }
    if seen < 4 {
        return Err(ZuanError::SnapshotMalformed("字段不足 4 项".into()));
    }
    if cfg.enable_editor && !cfg.offline_debug_only {
        return Err(ZuanError::UnsafeEditorOutsideOfflineDebug);
    }
    Ok(cfg)
}

/// 执行单行诊断命令（`config <json>` / `check_room_gate <online|offline>` / `snapshot` / `restore <snap>`）。
pub fn apply_zuan_line(cfg: &mut ZuanConfig, line: &str) -> Result<String, ZuanError> {
    let trimmed = line.trim();
    if trimmed == "snapshot" {
        return Ok(snapshot_zuan(cfg));
    }
    let (cmd, rest) = trimmed
        .split_once(' ')
        .ok_or_else(|| ZuanError::UnknownCommand(trimmed.to_string()))?;
    let rest = rest.trim();
    match cmd {
        "config" => {
            *cfg = parse_zuan_config(rest)?;
            Ok(snapshot_zuan(cfg))
        }
        "check_room_gate" => match rest {
            "online" => {
                if cfg.enable_editor {
                    return Err(ZuanError::OnlineRoomRejected(
                        "联机房间禁止启用本地文件编辑器",
                    ));
                }
                if cfg.rule_variant {
                    return Err(ZuanError::OnlineRoomRejected(
                        "联机标准房间禁止启用非标恶搞规则变体",
                    ));
                }
                Ok("room=online | allowed=1 | fs_write=0 | rule_variant=0".to_string())
            }
            "offline" => Ok(format!(
                "room=offline | allowed=1 | fs_write=0 | rule_variant={} | taunt_pack={}",
                u8::from(cfg.rule_variant),
                u8::from(cfg.taunt_pack)
            )),
            other => Err(ZuanError::InvalidField {
                field: "room_mode",
                reason: format!("未知房间模式 {other}"),
            }),
        },
        "restore" => {
            *cfg = restore_zuan(rest)?;
            Ok(snapshot_zuan(cfg))
        }
        other => Err(ZuanError::UnknownCommand(other.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zuan_config_rejects_unsafe_editor_and_gates_online() {
        let json = r#"{
            "extension_name": "祖安武将",
            "source": "noname-liuli",
            "upstream_path": "archive/upstream/noname-liuli/extension/祖安武将/extension.js",
            "default_config": {
                "enable_editor": false,
                "rule_variant": false,
                "offline_debug_only": true,
                "taunt_pack": true
            },
            "skill_count": 244,
            "card_count": 190,
            "character_count": 400,
            "translate_count": 239
        }"#;
        let mut cfg = parse_zuan_config(json).expect("valid config");
        assert_eq!(cfg, ZuanConfig::default());
        assert_eq!(
            apply_zuan_line(&mut cfg, "check_room_gate online").unwrap(),
            "room=online | allowed=1 | fs_write=0 | rule_variant=0"
        );

        let unsafe_json = r#"{
            "enable_editor": true,
            "rule_variant": false,
            "offline_debug_only": false,
            "taunt_pack": true
        }"#;
        assert_eq!(
            parse_zuan_config(unsafe_json),
            Err(ZuanError::UnsafeEditorOutsideOfflineDebug)
        );
    }
}
