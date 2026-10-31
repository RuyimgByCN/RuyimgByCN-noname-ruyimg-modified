//! L01 AI优化（noname-liuli）的可移植决策。
//!
//! 上游是纯 **AI 调参**扩展（不改规则）：技能威胁度表（`cf`）、武将权重表
//! （`qz`）与一组策略开关。本模块把「参数 + 结算口径」抽成可装载数据：
//! - 威胁度补充模式（`applyCf`：不补充/自动补充/手动补充）；
//! - 权重补充来源（`ckQz`：不设置/评级（线性映射到 `[0.8,1.95]`）/威胁度）；
//! - 权重参与效益结算（`qzCf`：开启后「权重 × 结果」）；
//! - 查询面：`threaten(skill)` / `weight(general, rating)` / `benefit(...)`。
//!
//! 原生 AI（`engine-ai`）为独立策略引擎——本模块只提供**参数与口径**，
//! 消费面由宿主注入（登记差异）。本模块不读写文件、不请求网络、不 eval。

use serde_json::Value;
use std::collections::BTreeMap;

/// 威胁度补充模式（上游 `applyCf`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CfMode {
    /// `off`：不补充。
    Off,
    /// `pj`：进入游戏时按武将评级自动补充。
    Auto,
    /// `sd`：游戏开始/隐匿武将展示时手动补充。
    Manual,
}

/// 武将权重补充来源（上游 `ckQz`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QzMode {
    /// `off`：不设置。
    Off,
    /// `pj`：按武将评级线性映射到 `[0.8, 1.95]`。
    Rating,
    /// `cf`：以武将威胁度作为权重。
    Threat,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AiTuningConfig {
    pub source: String,
    pub upstream_path: String,
    pub upstream_commit: String,
    /// 技能威胁度表（`cf`：技能名 → 威胁度）。
    pub cf: BTreeMap<String, f64>,
    /// 武将权重表（`qz`：武将名 → 权重）。
    pub qz: BTreeMap<String, f64>,
    /// 威胁度补充模式。
    pub apply_cf: CfMode,
    /// 武将登场补充权重（`applyQz`）。
    pub apply_qz: bool,
    /// 第二权重参考来源（`ckQz`）。
    pub ck_qz: QzMode,
    /// 权重参与效益结算（`qzCf`）。
    pub qz_cf: bool,
    /// 显示好感度（`viewAtt`）。
    pub view_att: bool,
    /// 农民友方判定（`nhFriends`）。
    pub nh_friends: bool,
    /// 南蛮/万箭策略（`ntAoe`）。
    pub nt_aoe: bool,
    /// 禁用千幻（`banQh`）。
    pub ban_qh: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiTuningError {
    BadField(String),
    MissingField(String),
}

impl AiTuningError {
    pub fn message(&self) -> String {
        match self {
            AiTuningError::BadField(f) => format!("配置字段非法：{f}"),
            AiTuningError::MissingField(f) => format!("配置字段缺失：{f}"),
        }
    }
}

fn opt_str(v: &Value, key: &str) -> Result<String, AiTuningError> {
    v.get(key)
        .and_then(|x| x.as_str())
        .map(str::to_string)
        .ok_or_else(|| AiTuningError::MissingField(key.to_string()))
}

fn opt_bool(v: &Value, key: &str, default: bool) -> bool {
    v.get(key).and_then(|x| x.as_bool()).unwrap_or(default)
}

fn number_table(v: &Value, key: &str) -> Result<BTreeMap<String, f64>, AiTuningError> {
    let mut out = BTreeMap::new();
    let Some(map) = v.get(key) else {
        return Ok(out);
    };
    let Some(map) = map.as_object() else {
        return Err(AiTuningError::BadField(key.to_string()));
    };
    for (k, val) in map {
        let Some(n) = val.as_f64() else {
            return Err(AiTuningError::BadField(format!("{key}.{k}")));
        };
        if !(0.0..=100.0).contains(&n) {
            return Err(AiTuningError::BadField(format!("{key}.{k}")));
        }
        out.insert(k.clone(), n);
    }
    Ok(out)
}

/// 解析转换包内的 `assets/config.json`。
pub fn parse_ai_tuning_config(json: &str) -> Result<AiTuningConfig, AiTuningError> {
    let v: Value =
        serde_json::from_str(json).map_err(|e| AiTuningError::BadField(e.to_string()))?;
    let apply_cf = match v.get("applyCf").and_then(|x| x.as_str()).unwrap_or("off") {
        "off" => CfMode::Off,
        "pj" => CfMode::Auto,
        "sd" => CfMode::Manual,
        other => return Err(AiTuningError::BadField(format!("applyCf={other}"))),
    };
    let ck_qz = match v.get("ckQz").and_then(|x| x.as_str()).unwrap_or("off") {
        "off" => QzMode::Off,
        "pj" => QzMode::Rating,
        "cf" => QzMode::Threat,
        other => return Err(AiTuningError::BadField(format!("ckQz={other}"))),
    };
    Ok(AiTuningConfig {
        source: opt_str(&v, "source")?,
        upstream_path: opt_str(&v, "upstream_path")?,
        upstream_commit: opt_str(&v, "upstream_commit")?,
        cf: number_table(&v, "cf")?,
        qz: number_table(&v, "qz")?,
        apply_cf,
        apply_qz: opt_bool(&v, "applyQz", false),
        ck_qz,
        qz_cf: opt_bool(&v, "qzCf", false),
        view_att: opt_bool(&v, "viewAtt", false),
        nh_friends: opt_bool(&v, "nhFriends", false),
        nt_aoe: opt_bool(&v, "ntAoe", false),
        ban_qh: opt_bool(&v, "banQh", false),
    })
}

/// 技能威胁度查询：表内取值；未配置返回 `None`（上游未覆盖即不改动）。
pub fn threaten(cfg: &AiTuningConfig, skill: &str) -> Option<f64> {
    cfg.cf.get(skill).copied()
}

/// 武将权重查询。
///
/// - 表内有值 → 该值；
/// - 否则按 `ckQz`：`Rating` → 评级线性映射到 `[0.8, 1.95]`（评级 0–5 钳制）；
///   `Threat` → 该武将技能威胁度的**最大值**（无记录则 1.0）；`Off` → 1.0。
pub fn weight(cfg: &AiTuningConfig, general: &str, rating: Option<f64>, skills: &[&str]) -> f64 {
    if let Some(w) = cfg.qz.get(general) {
        return *w;
    }
    match cfg.ck_qz {
        QzMode::Off => 1.0,
        QzMode::Rating => rating.map(rating_to_weight).unwrap_or(1.0),
        QzMode::Threat => skills
            .iter()
            .filter_map(|s| cfg.cf.get(*s).copied())
            .fold(None, |acc: Option<f64>, v| {
                Some(acc.map_or(v, |a| a.max(v)))
            })
            .unwrap_or(1.0),
    }
}

/// 评级 → 权重（上游：正相关映射到 `[0.8, 1.95]`；评级 0–5 钳制）。
pub fn rating_to_weight(rating: f64) -> f64 {
    let r = rating.clamp(0.0, 5.0);
    0.8 + (1.95 - 0.8) * (r / 5.0)
}

/// 效益结算：`qzCf` 开启时「权重 × 结果」，否则原结果。
pub fn benefit(cfg: &AiTuningConfig, w: f64, value: f64) -> f64 {
    if cfg.qz_cf {
        w * value
    } else {
        value
    }
}
