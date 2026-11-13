//! L20 标记补充配置与整肃展示卡加载器（`archive/upstream/noname-liuli/noname/extension/标记补充/extension.js:6319-6321,6398-6491`）。
//!
//! 覆盖项：
//! - 15 项 `L20:configs:*`（`extension.js:6398-6491`）：
//!   1. `jinengxiugai`（`bool`，默认 `true`）
//!   2. `biaojixiugai`（`bool`，默认 `true`）
//!   3. `zhijiechifa`（`bool`，默认 `true`）
//!   4. `doudizhu_guanqiuxiaoguo`（`bool`，默认 `true`）
//!   5. `huajiatexiao`（`bool`，默认 `true`）
//!   6. `guozhanmeihua`（`bool`，默认 `true`）
//!   7. `shoushajishatexiao`（`bool`，默认 `true`）
//!   8. `shiyongkapaitixiao`（`bool`，默认 `true`）
//!   9. `youxishengfu`（枚举：`identity` | `mobileMode` | `off`，默认 `off`）
//!   10. `daojianfu`（枚举：`shousha` | `4.5` | `off`，默认 `4.5`）
//!   11. `daojianfu_type`（枚举：`random` | `no` | `dao` | `jian` | `fuzi`，默认 `random`）
//!   12. `xinshadoudizhu`（`bool`，默认 `false`）
//!   13. `xlfbmoshi`（`bool`，默认 `false`；兼容包配置别名 `xueliangfanbei`）
//!   14. `mpfbmoshi`（`bool`，默认 `false`）
//!   15. `zhujizhongwang`（`bool`，默认 `false`）
//! - 3 张 `L20:cards:*` 整肃展示卡（`extension.js:6319-6321`）：
//!   1. `zhengsu_leijin`（擂进，`fullskin: true`）
//!   2. `zhengsu_mingzhi`（鸣止，`fullskin: true`）
//!   3. `zhengsu_bianzhen`（变阵，`fullskin: true`）

use serde_json::Value;

/// L20 标记补充配置加载错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkSupplementError {
    Json(String),
    Schema(String),
}

/// 上游 `标记补充/extension.js:6398-6491` 的 15 个配置键名。
pub const L20_MARK_SUPPLEMENT_CONFIG_KEYS: [&str; 15] = [
    "jinengxiugai",
    "biaojixiugai",
    "zhijiechifa",
    "doudizhu_guanqiuxiaoguo",
    "huajiatexiao",
    "guozhanmeihua",
    "shoushajishatexiao",
    "shiyongkapaitixiao",
    "youxishengfu",
    "daojianfu",
    "daojianfu_type",
    "xinshadoudizhu",
    "xlfbmoshi",
    "mpfbmoshi",
    "zhujizhongwang",
];

/// 上游 `标记补充/extension.js:6319-6321` 的 3 张整肃展示卡定义。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkSupplementCardSpec {
    pub id: &'static str,
    pub zh_name: &'static str,
    pub fullskin: bool,
    pub rule_summary: &'static str,
}

/// 3 张 `L20:cards:*` 整肃展示卡规范表。
pub const L20_MARK_SUPPLEMENT_CARD_SPECS: [MarkSupplementCardSpec; 3] = [
    MarkSupplementCardSpec {
        id: "zhengsu_leijin",
        zh_name: "擂进",
        fullskin: true,
        rule_summary: "出牌阶段内使用过至少 3 张牌且点数严格递增",
    },
    MarkSupplementCardSpec {
        id: "zhengsu_mingzhi",
        zh_name: "鸣止",
        fullskin: true,
        rule_summary: "弃牌阶段内弃置过至少 2 张牌且花色互不相同",
    },
    MarkSupplementCardSpec {
        id: "zhengsu_bianzhen",
        zh_name: "变阵",
        fullskin: true,
        rule_summary: "出牌阶段内使用过至少 2 张牌且花色完全相同",
    },
];

/// `youxishengfu`（游戏胜负特效，`extension.js:6437`）枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YouxiShengfuMode {
    Identity,
    MobileMode,
    Off,
}

impl YouxiShengfuMode {
    pub fn parse(raw: &str) -> Result<Self, MarkSupplementError> {
        match raw {
            "identity" => Ok(Self::Identity),
            "mobileMode" => Ok(Self::MobileMode),
            "off" => Ok(Self::Off),
            other => Err(MarkSupplementError::Schema(format!(
                "unknown mark-supplement youxishengfu: {other}"
            ))),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::MobileMode => "mobileMode",
            Self::Off => "off",
        }
    }
}

/// `daojianfu`（刀剑斧样式，`extension.js:6449`）枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaoJianFuStyle {
    Shousha,
    V45,
    Off,
}

impl DaoJianFuStyle {
    pub fn parse(raw: &str) -> Result<Self, MarkSupplementError> {
        match raw {
            "shousha" => Ok(Self::Shousha),
            "4.5" => Ok(Self::V45),
            "off" => Ok(Self::Off),
            other => Err(MarkSupplementError::Schema(format!(
                "unknown mark-supplement daojianfu: {other}"
            ))),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Shousha => "shousha",
            Self::V45 => "4.5",
            Self::Off => "off",
        }
    }
}

/// `daojianfu_type`（刀剑斧类型，`extension.js:6460`）枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaoJianFuType {
    Random,
    No,
    Dao,
    Jian,
    Fuzi,
}

impl DaoJianFuType {
    pub fn parse(raw: &str) -> Result<Self, MarkSupplementError> {
        match raw {
            "random" => Ok(Self::Random),
            "no" => Ok(Self::No),
            "dao" => Ok(Self::Dao),
            "jian" => Ok(Self::Jian),
            "fuzi" => Ok(Self::Fuzi),
            other => Err(MarkSupplementError::Schema(format!(
                "unknown mark-supplement daojianfu_type: {other}"
            ))),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Random => "random",
            Self::No => "no",
            Self::Dao => "dao",
            Self::Jian => "jian",
            Self::Fuzi => "fuzi",
        }
    }
}

/// L20 标记补充扩展的完整 15 项类型化配置模型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkSupplementConfig {
    pub jinengxiugai: bool,
    pub biaojixiugai: bool,
    pub zhijiechifa: bool,
    pub doudizhu_guanqiuxiaoguo: bool,
    pub huajiatexiao: bool,
    pub guozhanmeihua: bool,
    pub shoushajishatexiao: bool,
    pub shiyongkapaitixiao: bool,
    pub youxishengfu: YouxiShengfuMode,
    pub daojianfu: DaoJianFuStyle,
    pub daojianfu_type: DaoJianFuType,
    pub xinshadoudizhu: bool,
    pub xlfbmoshi: bool,
    pub mpfbmoshi: bool,
    pub zhujizhongwang: bool,
}

impl Default for MarkSupplementConfig {
    fn default() -> Self {
        Self {
            jinengxiugai: true,
            biaojixiugai: true,
            zhijiechifa: true,
            doudizhu_guanqiuxiaoguo: true,
            huajiatexiao: true,
            guozhanmeihua: true,
            shoushajishatexiao: true,
            shiyongkapaitixiao: true,
            youxishengfu: YouxiShengfuMode::Off,
            daojianfu: DaoJianFuStyle::V45,
            daojianfu_type: DaoJianFuType::Random,
            xinshadoudizhu: false,
            xlfbmoshi: false,
            mpfbmoshi: false,
            zhujizhongwang: false,
        }
    }
}

fn read_bool(
    obj: &serde_json::Map<String, Value>,
    key: &str,
    default: bool,
) -> Result<bool, MarkSupplementError> {
    match obj.get(key) {
        None => Ok(default),
        Some(Value::Bool(b)) => Ok(*b),
        Some(other) => Err(MarkSupplementError::Schema(format!(
            "mark-supplement config '{key}' must be bool, got {other}"
        ))),
    }
}

/// 解析并校验 `mark-supplement-upstream-liuli` 的 `config.json`。
pub fn load_mark_supplement_config(json: &str) -> Result<MarkSupplementConfig, MarkSupplementError> {
    let root: Value =
        serde_json::from_str(json).map_err(|e| MarkSupplementError::Json(e.to_string()))?;
    let obj = root.as_object().ok_or_else(|| {
        MarkSupplementError::Schema("mark-supplement config must be a JSON object".into())
    })?;
    let defaults = MarkSupplementConfig::default();

    let youxishengfu = match obj.get("youxishengfu") {
        None => defaults.youxishengfu,
        Some(Value::String(s)) => YouxiShengfuMode::parse(s.as_str())?,
        Some(other) => {
            return Err(MarkSupplementError::Schema(format!(
                "mark-supplement config 'youxishengfu' must be string, got {other}"
            )));
        }
    };

    let daojianfu = match obj.get("daojianfu") {
        None => defaults.daojianfu,
        Some(Value::String(s)) => DaoJianFuStyle::parse(s.as_str())?,
        Some(Value::Bool(false)) => DaoJianFuStyle::Off,
        Some(Value::Bool(true)) => DaoJianFuStyle::V45,
        Some(other) => {
            return Err(MarkSupplementError::Schema(format!(
                "mark-supplement config 'daojianfu' must be string or bool, got {other}"
            )));
        }
    };

    let daojianfu_type = match obj.get("daojianfu_type") {
        None => defaults.daojianfu_type,
        Some(Value::String(s)) => DaoJianFuType::parse(s.as_str())?,
        Some(other) => {
            return Err(MarkSupplementError::Schema(format!(
                "mark-supplement config 'daojianfu_type' must be string, got {other}"
            )));
        }
    };

    let xlfbmoshi = if obj.contains_key("xlfbmoshi") {
        read_bool(obj, "xlfbmoshi", defaults.xlfbmoshi)?
    } else {
        read_bool(obj, "xueliangfanbei", defaults.xlfbmoshi)?
    };

    Ok(MarkSupplementConfig {
        jinengxiugai: read_bool(obj, "jinengxiugai", defaults.jinengxiugai)?,
        biaojixiugai: read_bool(obj, "biaojixiugai", defaults.biaojixiugai)?,
        zhijiechifa: read_bool(obj, "zhijiechifa", defaults.zhijiechifa)?,
        doudizhu_guanqiuxiaoguo: read_bool(
            obj,
            "doudizhu_guanqiuxiaoguo",
            defaults.doudizhu_guanqiuxiaoguo,
        )?,
        huajiatexiao: read_bool(obj, "huajiatexiao", defaults.huajiatexiao)?,
        guozhanmeihua: read_bool(obj, "guozhanmeihua", defaults.guozhanmeihua)?,
        shoushajishatexiao: read_bool(obj, "shoushajishatexiao", defaults.shoushajishatexiao)?,
        shiyongkapaitixiao: read_bool(obj, "shiyongkapaitixiao", defaults.shiyongkapaitixiao)?,
        youxishengfu,
        daojianfu,
        daojianfu_type,
        xinshadoudizhu: read_bool(obj, "xinshadoudizhu", defaults.xinshadoudizhu)?,
        xlfbmoshi,
        mpfbmoshi: read_bool(obj, "mpfbmoshi", defaults.mpfbmoshi)?,
        zhujizhongwang: read_bool(obj, "zhujizhongwang", defaults.zhujizhongwang)?,
    })
}

/// 按卡名查找 L20 整肃展示卡定义（`zhengsu_leijin`、`zhengsu_mingzhi`、`zhengsu_bianzhen`）。
pub fn lookup_mark_supplement_card(card_id: &str) -> Option<&'static MarkSupplementCardSpec> {
    L20_MARK_SUPPLEMENT_CARD_SPECS
        .iter()
        .find(|spec| spec.id == card_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_and_repo_config_parse_all_15_keys_and_3_zhengsu_cards() {
        assert_eq!(L20_MARK_SUPPLEMENT_CONFIG_KEYS.len(), 15);
        assert_eq!(L20_MARK_SUPPLEMENT_CARD_SPECS.len(), 3);
        for spec in &L20_MARK_SUPPLEMENT_CARD_SPECS {
            assert!(spec.fullskin);
            assert_eq!(lookup_mark_supplement_card(spec.id), Some(spec));
        }
        assert!(lookup_mark_supplement_card("unknown_card").is_none());

        let cfg = load_mark_supplement_config("{}").expect("empty object uses upstream defaults");
        assert!(cfg.jinengxiugai);
        assert!(cfg.biaojixiugai);
        assert!(cfg.zhijiechifa);
        assert!(cfg.doudizhu_guanqiuxiaoguo);
        assert!(cfg.huajiatexiao);
        assert!(cfg.guozhanmeihua);
        assert!(cfg.shoushajishatexiao);
        assert!(cfg.shiyongkapaitixiao);
        assert_eq!(cfg.youxishengfu, YouxiShengfuMode::Off);
        assert_eq!(cfg.daojianfu, DaoJianFuStyle::V45);
        assert_eq!(cfg.daojianfu_type, DaoJianFuType::Random);
        assert!(!cfg.xinshadoudizhu);
        assert!(!cfg.xlfbmoshi);
        assert!(!cfg.mpfbmoshi);
        assert!(!cfg.zhujizhongwang);

        let repo_cfg = include_str!(
            "../../../packages/extensions/mark-supplement-upstream-liuli/assets/config.json"
        );
        let parsed = load_mark_supplement_config(repo_cfg).expect("repo config.json");
        assert!(!parsed.mpfbmoshi);
        assert!(!parsed.xlfbmoshi);
        assert_eq!(parsed.daojianfu, DaoJianFuStyle::Off);
    }

    #[test]
    fn rejects_invalid_enum_or_type_in_mark_supplement_config() {
        assert!(load_mark_supplement_config(r#"{"youxishengfu":"invalid"}"#).is_err());
        assert!(load_mark_supplement_config(r#"{"daojianfu":"invalid"}"#).is_err());
        assert!(load_mark_supplement_config(r#"{"daojianfu_type":"invalid"}"#).is_err());
        assert!(load_mark_supplement_config(r#"{"jinengxiugai":"yes"}"#).is_err());
    }
}
