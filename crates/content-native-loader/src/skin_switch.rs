//! L24 皮肤切换（noname-liuli）的可移植决策。
//!
//! 上游是动态皮肤/动画框架（624 KB 主文件，其中 `_pfqh_check_phaseEnd` 520 KB
//! 为**数据**）。本模块只迁移**可移植策略**：
//! - `_pfqh_reapply_saved_params` / `_pfqh_reapply_on_init`：开局与角色初始化时
//!   **重新应用已保存的动皮参数**——依赖（`skinSwitch`/`decadeUI`）未就绪时按
//!   0/500/1000/2000 ms 退避重试；角色初始化路径要求 `player.dynamic.primary`。
//! - `_pfqh_check_hp_change` 的**低血变身**：回复且未开启 `recover` → 跳过；
//!   每个头像位（主/副将）记录**历史最低体力**（初值 = 当前体力 + 1000），
//!   仅当出现**新低**时才触发并更新记录。
//! - 配置面：`dynamicSkin`/`useDynamic`/`attackEffect`/`isAttackFlipX`/
//!   `hideHuanFu`/`czgEnable`/`cugDynamicBg`/`showTopArc`/`savedPositions`。
//!
//! Spine/Worker、出框/攻击/背景动画渲染与十周年/千幻互调**不迁移**（登记差异）。
//! 本模块不读写文件、不请求网络、不 eval。

use serde_json::Value;
use std::collections::BTreeMap;

/// 重试退避表（上游 `setTimeout` 0 → 500 → 1000 → 2000 ms）。
pub const REAPPLY_BACKOFF_MS: [u64; 4] = [0, 500, 1000, 2000];

/// 重新应用参数的决策。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReapplyDecision {
    /// 立即应用。
    Apply,
    /// 依赖未就绪：按该毫秒数退避后重试。
    RetryAfter(u64),
    /// 重试次数用尽：放弃（上游最后一次尝试后不再重试）。
    GiveUp,
}

/// 重新应用已保存动皮参数的决策。
///
/// `deps_ready` = 上游 `window.skinSwitch.updateDecadeDynamicSkin` 可用且
/// `window.decadeUI` 已加载；`attempt` 从 0 起。
pub fn reapply_decision(deps_ready: bool, attempt: usize) -> ReapplyDecision {
    if deps_ready {
        return ReapplyDecision::Apply;
    }
    match REAPPLY_BACKOFF_MS.get(attempt + 1) {
        Some(ms) => ReapplyDecision::RetryAfter(*ms),
        None => ReapplyDecision::GiveUp,
    }
}

/// 角色初始化路径的过滤（上游 `player.dynamic && player.dynamic.primary`）。
pub fn reapply_on_init_allowed(has_dynamic: bool, is_primary: bool) -> bool {
    has_dynamic && is_primary
}

/// 低血变身记录（每个头像位一份历史最低体力）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LowHpState {
    pub primary_lowest: Option<i32>,
    pub deputy_lowest: Option<i32>,
}

impl LowHpState {
    fn slot(&mut self, is_primary: bool) -> &mut Option<i32> {
        if is_primary {
            &mut self.primary_lowest
        } else {
            &mut self.deputy_lowest
        }
    }
}

/// 低血变身决策（上游 `_pfqh_check_hp_change`）。
///
/// - `delta > 0`（回复）且未开启 `recover` → 不触发；
/// - 记录初值 = 当前体力 + 1000（上游 `hp + 1000`）；
/// - 当前体力 ≥ 记录值（即非新低）→ 不触发；
/// - 否则更新记录并触发。
pub fn lowhp_decision(
    state: &mut LowHpState,
    is_primary: bool,
    hp: i32,
    delta: i32,
    recover_allowed: bool,
) -> bool {
    if delta > 0 && !recover_allowed {
        return false;
    }
    let slot = state.slot(is_primary);
    let recorded = slot.unwrap_or(hp + 1000);
    if hp >= recorded {
        *slot = Some(recorded);
        return false;
    }
    *slot = Some(hp);
    true
}

#[derive(Debug, Clone, PartialEq)]
pub struct SkinSwitchConfig {
    pub source: String,
    pub upstream_path: String,
    pub upstream_commit: String,
    /// 动态皮肤总开关（上游 `dynamicSkin`）。
    pub dynamic_skin: bool,
    /// 使用动态资源（上游 `useDynamic`）。
    pub use_dynamic: bool,
    /// 攻击特效（上游 `attackEffect`）。
    pub attack_effect: bool,
    /// 攻击翻转 X（上游 `isAttackFlipX`）。
    pub attack_flip_x: bool,
    /// 隐藏幻符（上游 `hideHuanFu`）。
    pub hide_huanfu: bool,
    /// 出框（上游 `czgEnable`）。
    pub czg_enable: bool,
    /// 自定义动态背景（上游 `cugDynamicBg`）。
    pub cug_dynamic_bg: bool,
    /// 顶部弧线（上游 `showTopArc`）。
    pub show_top_arc: bool,
    /// 保存的位置参数（上游 `savedPositions`：键 → 数值）。
    pub saved_positions: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkinSwitchError {
    BadField(String),
    MissingField(String),
}

impl SkinSwitchError {
    pub fn message(&self) -> String {
        match self {
            SkinSwitchError::BadField(f) => format!("配置字段非法：{f}"),
            SkinSwitchError::MissingField(f) => format!("配置字段缺失：{f}"),
        }
    }
}

fn opt_str(v: &Value, key: &str) -> Result<String, SkinSwitchError> {
    v.get(key)
        .and_then(|x| x.as_str())
        .map(str::to_string)
        .ok_or_else(|| SkinSwitchError::MissingField(key.to_string()))
}

fn opt_bool(v: &Value, key: &str, default: bool) -> bool {
    v.get(key).and_then(|x| x.as_bool()).unwrap_or(default)
}

/// 解析转换包内的 `assets/config.json`。
pub fn parse_skin_switch_config(json: &str) -> Result<SkinSwitchConfig, SkinSwitchError> {
    let v: Value = serde_json::from_str(json)
        .map_err(|e| SkinSwitchError::BadField(e.to_string()))?;
    let mut saved = BTreeMap::new();
    if let Some(map) = v.get("savedPositions") {
        let Some(map) = map.as_object() else {
            return Err(SkinSwitchError::BadField("savedPositions".into()));
        };
        for (k, val) in map {
            let Some(n) = val.as_f64() else {
                return Err(SkinSwitchError::BadField(format!("savedPositions.{k}")));
            };
            saved.insert(k.clone(), n);
        }
    }
    Ok(SkinSwitchConfig {
        source: opt_str(&v, "source")?,
        upstream_path: opt_str(&v, "upstream_path")?,
        upstream_commit: opt_str(&v, "upstream_commit")?,
        dynamic_skin: opt_bool(&v, "dynamicSkin", true),
        use_dynamic: opt_bool(&v, "useDynamic", true),
        attack_effect: opt_bool(&v, "attackEffect", true),
        attack_flip_x: opt_bool(&v, "isAttackFlipX", false),
        hide_huanfu: opt_bool(&v, "hideHuanFu", false),
        czg_enable: opt_bool(&v, "czgEnable", true),
        cug_dynamic_bg: opt_bool(&v, "cugDynamicBg", false),
        show_top_arc: opt_bool(&v, "showTopArc", false),
        saved_positions: saved,
    })
}
