//! `U05 杀海拾遗` (`archive/upstream/noname/apps/core/extension/杀海拾遗/main/{config,content}.js`)
//! configuration schema, mode-specific forbid lists, and deterministic snapshot/restore.

use serde_json::Value;

/// Upstream `main/config.js:19-28` elemental-card probability (`rand`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShahaiWuxingCardRate {
    /// `"0.1"` (`10%`, `main/config.js:23`)
    Rate10,
    /// `"0.2"` (`20%`, `main/config.js:24`)
    Rate20,
    /// `"0.3"` (`30%`, default `init: "0.3"`, `main/config.js:21,25`)
    Rate30,
    /// `"0.5"` (`50%`, `main/config.js:26`)
    Rate50,
}

impl ShahaiWuxingCardRate {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Rate10 => "0.1",
            Self::Rate20 => "0.2",
            Self::Rate30 => "0.3",
            Self::Rate50 => "0.5",
        }
    }

    pub const fn percent(self) -> u8 {
        match self {
            Self::Rate10 => 10,
            Self::Rate20 => 20,
            Self::Rate30 => 30,
            Self::Rate50 => 50,
        }
    }

    pub fn from_code(s: &str) -> Result<Self, String> {
        match s {
            "0.1" => Ok(Self::Rate10),
            "0.2" => Ok(Self::Rate20),
            "0.3" => Ok(Self::Rate30),
            "0.5" => Ok(Self::Rate50),
            other => Err(format!("INVALID_SHAHAI_RAND:{other}")),
        }
    }
}

/// Complete `U05 杀海拾遗` configuration (`main/config.js:4-48`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShahaiMainConfig {
    /// `tip` (`clear: true` prompt banner, `main/config.js:4-13`).
    pub tip_banner_visible: bool,
    /// `wuxing` (`init: false`, `main/config.js:14-18`).
    pub wuxing: bool,
    /// `rand` (`init: "0.3"`, `main/config.js:19-28`).
    pub rand: ShahaiWuxingCardRate,
    /// `mtg` (`init: true`, `main/config.js:29-33`).
    pub mtg: bool,
    /// `zhenfa` (`init: true`, `main/config.js:34-38`).
    pub zhenfa: bool,
    /// `gwent` (`init: true`, `main/config.js:39-43`).
    pub gwent: bool,
    /// `yunchou` (`init: true`, `main/config.js:44-48`).
    pub yunchou: bool,
}

impl Default for ShahaiMainConfig {
    fn default() -> Self {
        Self {
            tip_banner_visible: true,
            wuxing: false,
            rand: ShahaiWuxingCardRate::Rate30,
            mtg: true,
            zhenfa: true,
            gwent: true,
            yunchou: true,
        }
    }
}

/// Upstream `main/content.js:6` `forbidboss` general list.
pub const SHAHAI_FORBID_BOSS: [&str; 1] = ["gjqt_aruan"];

/// Upstream `main/content.js:7` `forbidchess` general list.
pub const SHAHAI_FORBID_CHESS: [&str; 1] = ["swd_kangnalishi"];

/// Upstream `main/content.js:8` `forbiddouble` general list.
pub const SHAHAI_FORBID_DOUBLE: [&str; 4] = [
    "hs_siwangzhiyi",
    "hs_ronghejuren",
    "hs_shanlingjuren",
    "swd_kangnalishi",
];

/// Upstream `main/content.js:9` `forbidstone` general list.
pub const SHAHAI_FORBID_STONE: [&str; 23] = [
    "gjqt_bailitusu",
    "gjqt_aruan",
    "hs_malfurion",
    "hs_siwangzhiyi",
    "hs_xuanzhuanjijia",
    "hs_guldan",
    "hs_yngvar",
    "hs_anduin",
    "hs_antonidas",
    "hs_lreno",
    "ow_tianshi",
    "pal_xuanxiao",
    "pal_yuejinzhao",
    "swd_anka",
    "swd_nicole",
    "swd_tuwei",
    "swd_guyue",
    "swd_rongshuang",
    "swd_jiangziya",
    "swd_qiner",
    "swd_hanluo",
    "swd_huanglei",
    "swd_muyun",
];

/// Upstream `main/content.js:10` `forbidthreecard` weather card list.
pub const SHAHAI_FORBID_THREECARD: [&str; 4] = [
    "gw_ciguhanshuang",
    "gw_birinongwu",
    "gw_qinpendayu",
    "gw_poxiao",
];

/// Upstream `main/content.js:383` `stone` mode banned cards.
pub const SHAHAI_STONE_BANNED_CARDS: [&str; 6] = [
    "chuansongmen",
    "fengxueren",
    "guiyoujie",
    "jiguanshu",
    "lingjiandai",
    "sifeizhenmian",
];

/// Parse and validate `ShahaiMainConfig` from JSON (`main/config.js:4-48`).
pub fn parse_shahai_main_config(raw: &str) -> Result<ShahaiMainConfig, String> {
    let val: Value = serde_json::from_str(raw).map_err(|e| format!("BAD_SHAHAI_CONFIG_JSON:{e}"))?;
    let obj = val
        .as_object()
        .ok_or_else(|| "SHAHAI_CONFIG_NOT_OBJECT".to_string())?;
    let mut cfg = ShahaiMainConfig::default();

    let read_bool = |key: &str| -> Result<Option<bool>, String> {
        match obj.get(key) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::Bool(b)) => Ok(Some(*b)),
            Some(other) => Err(format!("INVALID_SHAHAI_BOOL:{key}:{other}")),
        }
    };
    if let Some(v) = read_bool("tip")? {
        cfg.tip_banner_visible = v;
    }
    if let Some(v) = read_bool("wuxing")? {
        cfg.wuxing = v;
    }
    if let Some(v) = read_bool("mtg")? {
        cfg.mtg = v;
    }
    if let Some(v) = read_bool("zhenfa")? {
        cfg.zhenfa = v;
    }
    if let Some(v) = read_bool("gwent")? {
        cfg.gwent = v;
    }
    if let Some(v) = read_bool("yunchou")? {
        cfg.yunchou = v;
    }
    if let Some(r) = obj.get("rand") {
        let code = r
            .as_str()
            .ok_or_else(|| format!("INVALID_SHAHAI_RAND_TYPE:{r}"))?;
        cfg.rand = ShahaiWuxingCardRate::from_code(code)?;
    }
    Ok(cfg)
}

/// Resolve enabled formal `sha*-upstream-noname` package IDs under `cfg`.
pub fn enabled_shahai_subpackages(cfg: &ShahaiMainConfig) -> Vec<&'static str> {
    let mut pkgs = vec![
        "shahai-gujian",
        "shahai-hearth",
        "shahai-swd",
        "shaai-ow",
        "shaai-xianjian",
    ];
    if cfg.gwent {
        pkgs.push("shahai-gwent");
    }
    if cfg.mtg {
        pkgs.push("shahai-mtg");
    }
    if cfg.yunchou {
        pkgs.push("shahai-yunchou");
    }
    if cfg.zhenfa {
        pkgs.push("shahai-zhenfa");
    }
    pkgs
}

/// Check whether an entity (`general_or_card_id`) is forbidden in `mode` (`main/content.js:5-11,383`).
pub fn is_shahai_forbid_in_mode(mode: &str, id: &str) -> bool {
    match mode {
        "boss" => SHAHAI_FORBID_BOSS.contains(&id),
        "chess" => SHAHAI_FORBID_CHESS.contains(&id),
        "double" => SHAHAI_FORBID_DOUBLE.contains(&id),
        "stone" => SHAHAI_FORBID_STONE.contains(&id) || SHAHAI_STONE_BANNED_CARDS.contains(&id),
        "threecard" => SHAHAI_FORBID_THREECARD.contains(&id),
        _ => false,
    }
}

/// Deterministic snapshot string for `ShahaiMainConfig`.
pub fn snapshot_shahai_config(cfg: &ShahaiMainConfig) -> String {
    serde_json::json!({
        "tip": cfg.tip_banner_visible,
        "wuxing": cfg.wuxing,
        "rand": cfg.rand.as_str(),
        "mtg": cfg.mtg,
        "zhenfa": cfg.zhenfa,
        "gwent": cfg.gwent,
        "yunchou": cfg.yunchou,
    })
    .to_string()
}

/// Restore `ShahaiMainConfig` from a deterministic snapshot string.
pub fn restore_shahai_config(snap: &str) -> Result<ShahaiMainConfig, String> {
    parse_shahai_main_config(snap)
}
