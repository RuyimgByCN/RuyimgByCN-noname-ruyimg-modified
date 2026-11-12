//! L10 史诗卡牌（noname-liuli）的可移植决策。
//!
//! 上游是「卡面/技能特效 + 两个规则技能」的混合扩展：
//! - 规则面：`reguhuo`（蛊惑：声明任意基本/锦囊牌使用）与 `rangjie`（让节：
//!   受伤后获得指定类型牌/移动场上牌）——两者已在 `engine-native` 以宿主行为
//!   落地（tag 515/516）。
//! - 十常侍面：把外部武将 `shichangshi` 的技能替换为 `k_danggu`/`k_mowang`
//!   并换图。`k_danggu`/`k_mowang` 已在 `engine-native` 以宿主行为落地
//!   （`HostBehavior::KDanggu`/`HostBehavior::KMowang`，tag 549/550），本模块
//!   同时保留十常侍面开关与四态依赖判定逻辑。
//! - 展示面：卡面/技能特效、emoji、折角、幻影、皮肤特效——原生不迁移渲染。
//!
//! 本模块不读写文件、不请求网络、不 eval。

use serde_json::Value;

/// 十常侍面依赖的外部技能（本仓未定义）。
pub const EXTERNAL_SKILLS: [&str; 2] = ["k_danggu", "k_mowang"];
/// 十常侍面覆盖的目标武将。
pub const OVERRIDE_CHARACTER: &str = "shichangshi";
/// 上游要求同时启用的扩展（`lib.config['extension_十周年UI_enable']`）。
pub const REQUIRED_EXTENSION: &str = "十周年UI";

#[derive(Debug, Clone, PartialEq)]
pub struct EpicConfig {
    pub source: String,
    pub upstream_path: String,
    pub upstream_commit: String,
    /// 卡面特效（上游 `cardEffects`）。
    pub card_effects: bool,
    /// 技能特效（上游 `skillEffects`）。
    pub skill_effects: bool,
    /// 技能特效缩放（上游 `skillEffectsScale`）。
    pub skill_effects_scale: f64,
    /// 卡面表情（上游 `cardEmoji`）。
    pub card_emoji: bool,
    /// 卡面折角（上游 `cardFold`）。
    pub card_fold: bool,
    /// 卡面幻影（上游 `cardPhantom`）。
    pub card_phantom: bool,
    /// 十常侍技能特效（上游 `meihuaskills`）。
    pub meihua_skills: bool,
    /// 十常侍面开关（上游 `shichangshi`）。
    pub shichangshi: bool,
    /// 皮肤特效（上游 `skinEffects`）。
    pub skin_effects: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EpicError {
    BadField(String),
    MissingField(String),
}

impl EpicError {
    pub fn message(&self) -> String {
        match self {
            EpicError::BadField(f) => format!("配置字段非法：{f}"),
            EpicError::MissingField(f) => format!("配置字段缺失：{f}"),
        }
    }
}

fn opt_str(v: &Value, key: &str) -> Result<String, EpicError> {
    v.get(key)
        .and_then(|x| x.as_str())
        .map(str::to_string)
        .ok_or_else(|| EpicError::MissingField(key.to_string()))
}

fn opt_bool(v: &Value, key: &str, default: bool) -> bool {
    v.get(key).and_then(|x| x.as_bool()).unwrap_or(default)
}

/// 解析转换包内的 `assets/config.json`（展示面配置）。
pub fn parse_epic_config(json: &str) -> Result<EpicConfig, EpicError> {
    let v: Value = serde_json::from_str(json).map_err(|e| EpicError::BadField(e.to_string()))?;
    let scale = v
        .get("skillEffectsScale")
        .and_then(|x| x.as_f64())
        .unwrap_or(1.0);
    if !(0.0..=10.0).contains(&scale) {
        return Err(EpicError::BadField("skillEffectsScale".into()));
    }
    Ok(EpicConfig {
        source: opt_str(&v, "source")?,
        upstream_path: opt_str(&v, "upstream_path")?,
        upstream_commit: opt_str(&v, "upstream_commit")?,
        card_effects: opt_bool(&v, "cardEffects", true),
        skill_effects: opt_bool(&v, "skillEffects", true),
        skill_effects_scale: scale,
        card_emoji: opt_bool(&v, "cardEmoji", false),
        card_fold: opt_bool(&v, "cardFold", false),
        card_phantom: opt_bool(&v, "cardPhantom", false),
        meihua_skills: opt_bool(&v, "meihuaskills", false),
        shichangshi: opt_bool(&v, "shichangshi", false),
        skin_effects: opt_bool(&v, "skinEffects", false),
    })
}

/// 展示面开关汇总（原生不迁移渲染——仅登记「上游会开启哪些展示面」）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresentationFlags {
    pub card_effects: bool,
    pub skill_effects: bool,
    pub card_emoji: bool,
    pub card_fold: bool,
    pub card_phantom: bool,
    pub skin_effects: bool,
}

impl EpicConfig {
    /// 本包启用的展示面（供宿主登记/调试面查询）。
    pub fn presentation(&self) -> PresentationFlags {
        PresentationFlags {
            card_effects: self.card_effects,
            skill_effects: self.skill_effects,
            card_emoji: self.card_emoji,
            card_fold: self.card_fold,
            card_phantom: self.card_phantom,
            skin_effects: self.skin_effects,
        }
    }

    /// 十常侍面是否可执行：开关开启 + 目标武将与两个外部技能齐备。
    ///
    /// `has_character`/`has_skill` 由宿主注入（原生目录查询）。
    pub fn shichangshi_plan(
        &self,
        has_character: bool,
        has_skill: impl Fn(&str) -> bool,
    ) -> ShichangshiPlan {
        if !self.shichangshi {
            return ShichangshiPlan::Disabled;
        }
        if !has_character {
            return ShichangshiPlan::MissingCharacter(OVERRIDE_CHARACTER.to_string());
        }
        for skill in EXTERNAL_SKILLS {
            if !has_skill(skill) {
                return ShichangshiPlan::MissingSkill(skill.to_string());
            }
        }
        ShichangshiPlan::Apply {
            character: OVERRIDE_CHARACTER.to_string(),
            skills: EXTERNAL_SKILLS.iter().map(|s| (*s).to_string()).collect(),
            required_extension: REQUIRED_EXTENSION.to_string(),
        }
    }
}

/// 十常侍面处置结论。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShichangshiPlan {
    /// 配置未开启。
    Disabled,
    /// 目标武将不在目录（外部包未装载）。
    MissingCharacter(String),
    /// 外部技能缺失（本仓未定义——上游依赖十周年UI）。
    MissingSkill(String),
    /// 可执行覆盖（宿主按此替换技能并换图）。
    Apply {
        character: String,
        skills: Vec<String>,
        required_extension: String,
    },
}
