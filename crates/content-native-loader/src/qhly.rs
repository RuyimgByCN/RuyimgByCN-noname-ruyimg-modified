//! L09 千幻聆音（noname-liuli）的可移植决策。
//!
//! 上游是皮肤/语音/详情页框架（718 KB 主文件，其中 `_qhly_bgm` 482 KB、
//! `_qhlyCheckSkin` 96 KB 为**数据/索引**）。本模块只迁移**可移植策略**：
//! - `_qhlyChangeSkin`：受伤/回合结束换肤——按角色皮肤表 `qhly_skinChange`
//!   的规则（`source` 以 `hp_` 开头）决定是否切换，并以「本回合已因掉血换过」
//!   标志门控阶段结束的切换；`phase` 标志表示该规则仅在阶段结束生效。
//! - `_qhly_randskin`：开局随机皮肤——候选含「默认(false)」，剔除被封禁皮肤与
//!   名称含「娘化」「云涯」者；`qhly_randskin_me` 开启时不作用于人类玩家。
//! - `_qhly_autoc`：开局自动换肤（配置开关）。
//!
//! 皮肤/语音**数据**（立绘索引、BGM 库）不迁移（登记差异）。本模块不读写
//! 文件、不请求网络、不 eval。

use serde_json::Value;
use std::collections::BTreeMap;

/// 换肤触发（上游 `trigger.name`：`recover` / `phase`，以及掉血路径）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkinTrigger {
    /// 体力变化（`changeHp`）——上游按 `trigger.num` 正负区分回复/掉血。
    HpChange { delta: i32 },
    /// 回合结束（`phaseEnd`）。
    PhaseEnd,
}

/// 皮肤表规则（上游 `qhly_skinChange[角色][皮肤]`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkinChangeRule {
    /// 规则来源（上游 `source`；仅以 `hp_` 开头的规则参与换肤）。
    pub source: String,
    /// 仅阶段结束生效（上游 `phase`）。
    pub phase_only: bool,
    /// 目标皮肤（上游由音频/皮肤索引检查得出；此处显式建模）。
    pub target: Option<String>,
}

impl SkinChangeRule {
    /// 是否参与换肤（上游过滤：`source.indexOf('hp_') == 0`）。
    pub fn is_active(&self) -> bool {
        self.source.starts_with("hp_")
    }
}

/// 换肤决策状态（上游 `player._qhly_gonnaChange[i]`）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SkinChangeState {
    /// 本回合是否已因掉血换过肤。
    pub switched_on_hp_loss: bool,
}

/// 换肤决策：返回应切换到的目标皮肤（`None` = 不切换）。
///
/// 上游口径：仅 `hp_` 规则参与；`phase_only` 规则在阶段结束且**本回合未因
/// 掉血换过**时生效；掉血（`delta < 0`）时置位「已换」标志；回复（`delta > 0`）
/// 不触发（上游仅在 `trigger.num > 0` 时置位、且 `phase` 规则此时被跳过）。
pub fn decide_skin(
    rule: &SkinChangeRule,
    trigger: SkinTrigger,
    state: &mut SkinChangeState,
) -> Option<String> {
    if !rule.is_active() {
        return None;
    }
    match trigger {
        SkinTrigger::HpChange { delta } => {
            if delta > 0 {
                // 回复：上游置位「本回合将换肤」但跳过 phase 规则；此处不切换
                return None;
            }
            if delta < 0 {
                state.switched_on_hp_loss = true;
            }
            rule.target.clone()
        }
        SkinTrigger::PhaseEnd => {
            if state.switched_on_hp_loss {
                // 本回合已因掉血换过 → 阶段结束不再切换（上游 continue）
                state.switched_on_hp_loss = false;
                return None;
            }
            state.switched_on_hp_loss = false;
            rule.target.clone()
        }
    }
}

/// 随机皮肤候选（上游 `qhly_getSkinList` + 过滤 + `list.push(false)`）。
///
/// 返回被选中的皮肤；`None` 表示选中「默认」（上游 `false`）。
pub fn pick_random_skin(
    candidates: &[String],
    banned: &[String],
    exclude_terms: &[&str],
    roll: usize,
) -> Option<String> {
    let mut pool: Vec<Option<String>> = candidates
        .iter()
        .filter(|s| !banned.iter().any(|b| b == *s))
        .filter(|s| !exclude_terms.iter().any(|t| s.contains(t)))
        .cloned()
        .map(Some)
        .collect();
    // 上游把 `false`（默认皮肤）也放入候选池
    pool.push(None);
    if pool.is_empty() {
        return None;
    }
    pool[roll % pool.len()].clone()
}

#[derive(Debug, Clone, PartialEq)]
pub struct QhlyConfig {
    pub source: String,
    pub upstream_path: String,
    pub upstream_commit: String,
    /// 开局随机皮肤（上游 `qhly_randskin`）。
    pub rand_skin: bool,
    /// 随机皮肤不作用于人类玩家（上游 `qhly_randskin_me`）。
    pub rand_skin_me: bool,
    /// 开局自动换肤（上游 `qhly_autoChangeSkin`；`close`/空 = 关闭）。
    pub auto_change_skin: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QhlyError {
    BadField(String),
    MissingField(String),
}

impl QhlyError {
    pub fn message(&self) -> String {
        match self {
            QhlyError::BadField(f) => format!("配置字段非法：{f}"),
            QhlyError::MissingField(f) => format!("配置字段缺失：{f}"),
        }
    }
}

fn opt_str(v: &Value, key: &str) -> Result<String, QhlyError> {
    v.get(key)
        .and_then(|x| x.as_str())
        .map(str::to_string)
        .ok_or_else(|| QhlyError::MissingField(key.to_string()))
}

/// 解析转换包内的 `assets/config.json`。
pub fn parse_qhly_config(json: &str) -> Result<QhlyConfig, QhlyError> {
    let v: Value = serde_json::from_str(json).map_err(|e| QhlyError::BadField(e.to_string()))?;
    let auto = v
        .get("qhly_autoChangeSkin")
        .and_then(|x| x.as_str())
        .map(str::to_string)
        .filter(|s| s != "close" && !s.is_empty());
    Ok(QhlyConfig {
        source: opt_str(&v, "source")?,
        upstream_path: opt_str(&v, "upstream_path")?,
        upstream_commit: opt_str(&v, "upstream_commit")?,
        rand_skin: v.get("qhly_randskin").and_then(|x| x.as_bool()).unwrap_or(true),
        rand_skin_me: v
            .get("qhly_randskin_me")
            .and_then(|x| x.as_bool())
            .unwrap_or(false),
        auto_change_skin: auto,
    })
}

/// 皮肤表（角色 → 皮肤 → 规则）。
pub type SkinChangeTable = BTreeMap<String, BTreeMap<String, SkinChangeRule>>;
