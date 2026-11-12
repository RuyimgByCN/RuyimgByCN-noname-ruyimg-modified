//! L27 选将美化（`noname/extension/选将美化/extension.js:9001-9053`）10 项配置解析与校验。
//!
//! 覆盖全部 10 项 `L27:configs:*`：
//! 1. `twotwo`（自由选将时可以选择两个武将，默认 `true`）
//! 2. `gaibiaoji`（部分武将技能覆盖+改标记，默认 `true`）
//! 3. `dianjibiaoji`（标记样式美化+点击查看标记内容，默认 `true`）
//! 4. `shoupaihuase`（显示其他角色的手牌花色，默认 `true`）
//! 5. `feichuxuanxiang`（废除装备栏美化，默认 `true`）
//! 6. `shuzianniu`（数字选择按钮美化，默认 `true`）
//! 7. `xuanzejineng`（选择获得技能界面美化，默认 `true`）
//! 8. `zediewugu`（折叠五谷丰登卡牌，默认 `true`）
//! 9. `jinxuanbiaoyu`（禁选标语样式：`"1"` 黑底红叉 / `"2"` 禁选锁链 / `"3"` 禁选锁链（动态），默认 `"1"`）
//! 10. `jinengtexiaoxiugai`（出杀技能特效修改，默认 `true`）

use serde_json::Value;

/// `jinxuanbiaoyu` 合法枚举项（`extension.js:9038-9047`）。
pub const JINXUAN_BIAOYU_ITEMS: [&str; 3] = ["1", "2", "3"];

/// `jinxuanbiaoyu` 枚举项对应的中文标签。
pub const JINXUAN_BIAOYU_LABELS: [&str; 3] = ["黑底红叉", "禁选锁链", "禁选锁链（动态）"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectBeautifyConfig {
    pub source: String,
    pub upstream_path: String,
    pub upstream_commit: String,
    /// `L27:configs:twotwo`：自由选将时可以选择两个武将。
    pub twotwo: bool,
    /// `L27:configs:gaibiaoji`：部分武将技能覆盖+改标记。
    pub gaibiaoji: bool,
    /// `L27:configs:dianjibiaoji`：标记样式美化+点击查看标记内容。
    pub dianjibiaoji: bool,
    /// `L27:configs:shoupaihuase`：显示其他角色的手牌花色。
    pub shoupaihuase: bool,
    /// `L27:configs:feichuxuanxiang`：废除装备栏美化。
    pub feichuxuanxiang: bool,
    /// `L27:configs:shuzianniu`：数字选择按钮美化。
    pub shuzianniu: bool,
    /// `L27:configs:xuanzejineng`：选择获得技能界面美化。
    pub xuanzejineng: bool,
    /// `L27:configs:zediewugu`：折叠五谷丰登卡牌。
    pub zediewugu: bool,
    /// `L27:configs:jinxuanbiaoyu`：禁选标语样式（`"1"` / `"2"` / `"3"`）。
    pub jinxuanbiaoyu: String,
    /// `L27:configs:jinengtexiaoxiugai`：出杀技能特效修改。
    pub jinengtexiaoxiugai: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectBeautifyError {
    BadField(String),
    MissingField(String),
}

impl SelectBeautifyError {
    pub fn message(&self) -> String {
        match self {
            SelectBeautifyError::BadField(f) => format!("选将美化配置字段非法：{f}"),
            SelectBeautifyError::MissingField(f) => format!("选将美化配置字段缺失：{f}"),
        }
    }
}

fn req_str(v: &Value, key: &str) -> Result<String, SelectBeautifyError> {
    v.get(key)
        .and_then(|x| x.as_str())
        .map(str::to_string)
        .ok_or_else(|| SelectBeautifyError::MissingField(key.to_string()))
}

fn opt_bool_strict(v: &Value, key: &str, default: bool) -> Result<bool, SelectBeautifyError> {
    match v.get(key) {
        None => Ok(default),
        Some(x) => x
            .as_bool()
            .ok_or_else(|| SelectBeautifyError::BadField(key.to_string())),
    }
}

fn opt_item(
    v: &Value,
    key: &str,
    items: &[&str],
    default: &str,
) -> Result<String, SelectBeautifyError> {
    match v.get(key) {
        None => Ok(default.to_string()),
        Some(x) => x
            .as_str()
            .filter(|s| items.contains(s))
            .map(str::to_string)
            .ok_or_else(|| SelectBeautifyError::BadField(key.to_string())),
    }
}

/// 解析并校验 `select-beautify-upstream-liuli/assets/config.json`。
pub fn parse_select_beautify_config(
    json: &str,
) -> Result<SelectBeautifyConfig, SelectBeautifyError> {
    let v: Value =
        serde_json::from_str(json).map_err(|e| SelectBeautifyError::BadField(e.to_string()))?;
    Ok(SelectBeautifyConfig {
        source: req_str(&v, "source")?,
        upstream_path: req_str(&v, "upstream_path")?,
        upstream_commit: req_str(&v, "upstream_commit")?,
        twotwo: opt_bool_strict(&v, "twotwo", true)?,
        gaibiaoji: opt_bool_strict(&v, "gaibiaoji", true)?,
        dianjibiaoji: opt_bool_strict(&v, "dianjibiaoji", true)?,
        shoupaihuase: opt_bool_strict(&v, "shoupaihuase", true)?,
        feichuxuanxiang: opt_bool_strict(&v, "feichuxuanxiang", true)?,
        shuzianniu: opt_bool_strict(&v, "shuzianniu", true)?,
        xuanzejineng: opt_bool_strict(&v, "xuanzejineng", true)?,
        zediewugu: opt_bool_strict(&v, "zediewugu", true)?,
        jinxuanbiaoyu: opt_item(&v, "jinxuanbiaoyu", &JINXUAN_BIAOYU_ITEMS, "1")?,
        jinengtexiaoxiugai: opt_bool_strict(&v, "jinengtexiaoxiugai", true)?,
    })
}

impl SelectBeautifyConfig {
    /// 返回 `jinxuanbiaoyu` 枚举项对应的中文文案。
    pub fn jinxuan_biaoyu_label(&self) -> &'static str {
        match self.jinxuanbiaoyu.as_str() {
            "2" => JINXUAN_BIAOYU_LABELS[1],
            "3" => JINXUAN_BIAOYU_LABELS[2],
            _ => JINXUAN_BIAOYU_LABELS[0],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_all_10_l27_configs_and_rejects_bad_values() {
        let json = r#"{
            "source": "noname-liuli",
            "upstream_path": "noname/extension/选将美化/extension.js",
            "upstream_commit": "8485dbb29ff16f9b258a766b1a17ef75f883aade",
            "twotwo": true,
            "gaibiaoji": true,
            "dianjibiaoji": true,
            "shoupaihuase": false,
            "feichuxuanxiang": true,
            "shuzianniu": true,
            "xuanzejineng": true,
            "zediewugu": false,
            "jinxuanbiaoyu": "2",
            "jinengtexiaoxiugai": true
        }"#;
        let cfg = parse_select_beautify_config(json).expect("valid L27 config");
        assert!(cfg.twotwo);
        assert!(cfg.gaibiaoji);
        assert!(cfg.dianjibiaoji);
        assert!(!cfg.shoupaihuase);
        assert!(cfg.feichuxuanxiang);
        assert!(cfg.shuzianniu);
        assert!(cfg.xuanzejineng);
        assert!(!cfg.zediewugu);
        assert_eq!(cfg.jinxuanbiaoyu, "2");
        assert_eq!(cfg.jinxuan_biaoyu_label(), "禁选锁链");
        assert!(cfg.jinengtexiaoxiugai);

        let bad_enum = json.replace("\"jinxuanbiaoyu\": \"2\"", "\"jinxuanbiaoyu\": \"99\"");
        assert_eq!(
            parse_select_beautify_config(&bad_enum),
            Err(SelectBeautifyError::BadField("jinxuanbiaoyu".into()))
        );
    }
}
