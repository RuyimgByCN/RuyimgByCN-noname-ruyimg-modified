//! 龙血玄黄（L29）配置桥接与道心值报告。
//!
//! 上游 `extension/龙血玄黄/extension.js` 的规则面由内核 `mbqianlong`（道心值
//! 系统）承载；本模块只负责**包配置**（`config.shengjian` 圣剑显示开关）与
//! **阈值解锁报告**（宿主/规则面查询面），不做显示层。

use serde_json::Value;

/// 龙血玄黄包配置（`assets/config.json`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LongxueConfig {
    pub source: String,
    pub upstream_path: String,
    pub package_name: String,
    pub package_version: String,
    pub author: String,
    /// `config.shengjian.init`：圣剑显示特效开关（**显示面**——原生不迁移
    /// Spine/DOM，仅登记配置值）。
    pub shengjian: bool,
}

/// 配置解析错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LongxueError {
    Json(String),
    Missing(&'static str),
    BadField(&'static str),
}

/// 道心值阈值表（上游 `mbqianlong` subSkill 5：25 清正 / 50 酒诗 / 75 放逐 /
/// 99 绝进）。返回 (技能名, 阈值, 是否已转换)。
pub const LONGXUE_THRESHOLDS: [(&str, u8, bool); 4] = [
    ("mbcmqingzheng", 25, true),
    ("mbcmjiushi", 50, true),
    ("mbcmfangzhu", 75, true),
    ("mbjuejin", 99, true),
];

/// 解析包配置：字段缺失/类型错误一律拒绝（与其余来源同口径）。
pub fn parse_longxue_config(text: &str) -> Result<LongxueConfig, LongxueError> {
    let v: Value = serde_json::from_str(text).map_err(|e| LongxueError::Json(e.to_string()))?;
    let obj = v.as_object().ok_or(LongxueError::Json("root".into()))?;
    let source = obj
        .get("source")
        .and_then(|x| x.as_str())
        .ok_or(LongxueError::Missing("source"))?;
    if source.is_empty() {
        return Err(LongxueError::BadField("source"));
    }
    let upstream_path = obj
        .get("upstream_path")
        .and_then(|x| x.as_str())
        .ok_or(LongxueError::Missing("upstream_path"))?;
    if upstream_path.is_empty() {
        return Err(LongxueError::BadField("upstream_path"));
    }
    let package_name = obj
        .get("package_name")
        .and_then(|x| x.as_str())
        .ok_or(LongxueError::Missing("package_name"))?;
    let package_version = obj
        .get("package_version")
        .and_then(|x| x.as_str())
        .ok_or(LongxueError::Missing("package_version"))?;
    let author = obj
        .get("author")
        .and_then(|x| x.as_str())
        .ok_or(LongxueError::Missing("author"))?;
    let settings = obj
        .get("settings")
        .and_then(|x| x.as_object())
        .ok_or(LongxueError::Missing("settings"))?;
    let shengjian = settings
        .get("shengjian")
        .and_then(|x| x.as_bool())
        .ok_or(LongxueError::Missing("settings.shengjian"))?;
    Ok(LongxueConfig {
        source: source.to_string(),
        upstream_path: upstream_path.to_string(),
        package_name: package_name.to_string(),
        package_version: package_version.to_string(),
        author: author.to_string(),
        shengjian,
    })
}

/// 道心值解锁报告（宿主查询面）：已达标技能与外部依赖标注。
pub fn longxue_unlock_report(marks: u8) -> String {
    let mut parts: Vec<String> = Vec::new();
    for (name, threshold, converted) in LONGXUE_THRESHOLDS {
        if marks < threshold {
            continue;
        }
        parts.push(if converted {
            name.to_string()
        } else {
            format!("{name}(external)")
        });
    }
    format!("marks:{marks}|unlocked:{}", parts.join(","))
}
