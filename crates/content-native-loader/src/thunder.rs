//! L28 雷霆万钧（noname-liuli）的可移植决策。
//!
//! 上游是「小游戏武将包 + 本体同名武将技能替换 + UI/标记补丁」的混合扩展：
//! - **同名替换**（`replaceCha`）：把本体武将的技能表替换为 `th_*` 变体
//!   （`sameHero`/`chgHero` 两张平行表）——纯数据表，可移植。
//! - **隐藏专属包**（`hidePack`）：隐藏 `thunder` 武将包——策略开关。
//! - **禁同名**（`forbidSame`）：重名武将标记 `unseen` 并在选将时避免同名。
//! - **标记文本补丁**：曹金玉标记「转人话」、孙寒华妙剑/莲华分级标记、神姜维
//!   九伐标记改文字——纯标签重写，可移植为数据。
//! - 其余（`UIpatch`/`gameSpeed`/`UIItroduc`/`yuqiTheme`/`timeTheme`/`standPath`）为渲染面：
//!   本模块只校验取值并解析出档位文案、主题跟随与立绘路径，不做渲染。
//!
//! 本模块不读写文件、不请求网络、不 eval。

use serde_json::Value;

/// 上游同名替换表（本体武将）。
pub const SAME_HERO: [&str; 12] = [
    "majun",
    "ol_puyuan",
    "sunhanhua",
    "pangdegong",
    "zhengxuan",
    "xin_mamidi",
    "nanhualaoxian",
    "zhouqun",
    "wanglang",
    "liuhui",
    "guanning",
    "shen_zhouyu",
];

/// 上游同名替换表（`th_*` 变体，与 [`SAME_HERO`] 逐位对应）。
pub const CHG_HERO: [&str; 12] = [
    "th_majun",
    "th_puyuan",
    "th_sunhanhua",
    "th_pangdegong",
    "th_zhengxuan",
    "th_mamidi",
    "th_nanhualaoxian",
    "th_zhouqun",
    "th_wanglang",
    "th_liuhui",
    "th_guanning",
    "th_shen_zhouyu",
];

/// 专属武将包名（`hidePack` 隐藏对象）。
pub const EXCLUSIVE_PACK: &str = "thunder";

#[derive(Debug, Clone, PartialEq)]
pub struct ThunderConfig {
    pub source: String,
    pub upstream_path: String,
    pub upstream_commit: String,
    /// 小游戏速度档位（0 极速 … 8 龟速；渲染面）。
    pub game_speed: u8,
    /// 本体同名武将技能替换。
    pub replace_cha: bool,
    /// 隐藏小游戏武将专属包。
    pub hide_pack: bool,
    /// 禁止同名武将。
    pub forbid_same: bool,
    /// UI 补丁（渲染面）。
    pub ui_patch: bool,
    /// 曹金玉标记补丁（标记文本）。
    pub jinyu: bool,
    /// 孙寒华标记优化。
    pub shh_biaoji: bool,
    /// 孙寒华标记改版（分级上标数字）。
    pub shh_new: bool,
    /// 神姜维九伐标记改文字。
    pub jiufa_text: bool,
    /// 管宁技能全扩。
    pub guanning: bool,
    /// UI 补丁使用指南入口（`UIItroduc`，上游唯一项 `intro`）。
    pub ui_introduc: String,
    /// 对话框主题（`yuqiTheme`：`default`/`decade`/`shousha`/`follow`）。
    pub yuqi_theme: String,
    /// 进度条风格（`timeTheme`：`decade`/`shousha`/`follow`）。
    pub time_theme: String,
    /// 立绘存放目录（`standPath`，两项上游目录之一）。
    pub stand_path: String,
}

/// `gameSpeed` 档位文案（`extension.js:9068-9079`）。
pub const GAME_SPEED_LABELS: [&str; 9] = [
    "极速", "很快", "快速", "稍快", "常速", "稍慢", "慢速", "很慢", "龟速",
];
/// `UIItroduc` 合法项（`extension.js:9091-9094`）。
pub const UI_INTRODUC_ITEMS: [&str; 1] = ["intro"];
/// `yuqiTheme` 合法项（`extension.js:9109-9118`）。
pub const YUQI_THEME_ITEMS: [&str; 4] = ["default", "decade", "shousha", "follow"];
/// `timeTheme` 合法项（`extension.js:9120-9128`）。
pub const TIME_THEME_ITEMS: [&str; 3] = ["decade", "shousha", "follow"];
/// `standPath` 合法项（`extension.js:9134-9140`）。
pub const STAND_PATH_ITEMS: [&str; 2] = [
    "extension/雷霆万钧/image/stand/",
    "extension/千幻聆音/lihui/",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThunderError {
    BadField(String),
    MissingField(String),
}

impl ThunderError {
    pub fn message(&self) -> String {
        match self {
            ThunderError::BadField(f) => format!("配置字段非法：{f}"),
            ThunderError::MissingField(f) => format!("配置字段缺失：{f}"),
        }
    }
}

fn opt_str(v: &Value, key: &str) -> Result<String, ThunderError> {
    v.get(key)
        .and_then(|x| x.as_str())
        .map(str::to_string)
        .ok_or_else(|| ThunderError::MissingField(key.to_string()))
}

fn opt_bool(v: &Value, key: &str, default: bool) -> bool {
    v.get(key).and_then(|x| x.as_bool()).unwrap_or(default)
}

/// 选项型配置：缺省取上游 `init`，存在时必须是上游 `item` 中的一项。
fn opt_item(v: &Value, key: &str, items: &[&str], default: &str) -> Result<String, ThunderError> {
    match v.get(key) {
        None => Ok(default.to_string()),
        Some(x) => x
            .as_str()
            .filter(|s| items.contains(s))
            .map(str::to_string)
            .ok_or_else(|| ThunderError::BadField(key.to_string())),
    }
}

/// 解析转换包内的 `assets/config.json`。
pub fn parse_thunder_config(json: &str) -> Result<ThunderConfig, ThunderError> {
    let v: Value = serde_json::from_str(json).map_err(|e| ThunderError::BadField(e.to_string()))?;
    let speed = v.get("gameSpeed").and_then(|x| x.as_u64()).unwrap_or(4);
    if speed > 8 {
        return Err(ThunderError::BadField("gameSpeed".into()));
    }
    Ok(ThunderConfig {
        source: opt_str(&v, "source")?,
        upstream_path: opt_str(&v, "upstream_path")?,
        upstream_commit: opt_str(&v, "upstream_commit")?,
        game_speed: speed as u8,
        replace_cha: opt_bool(&v, "replaceCha", true),
        hide_pack: opt_bool(&v, "hidePack", true),
        forbid_same: opt_bool(&v, "forbidSame", false),
        ui_patch: opt_bool(&v, "UIpatch", true),
        jinyu: opt_bool(&v, "jinyu", true),
        shh_biaoji: opt_bool(&v, "shhbiaoji", true),
        shh_new: opt_bool(&v, "shh_new", false),
        jiufa_text: opt_bool(&v, "jiufatext", true),
        guanning: opt_bool(&v, "guanning", false),
        ui_introduc: opt_item(&v, "UIItroduc", &UI_INTRODUC_ITEMS, "intro")?,
        yuqi_theme: opt_item(&v, "yuqiTheme", &YUQI_THEME_ITEMS, "follow")?,
        time_theme: opt_item(&v, "timeTheme", &TIME_THEME_ITEMS, "follow")?,
        stand_path: opt_item(&v, "standPath", &STAND_PATH_ITEMS, STAND_PATH_ITEMS[0])?,
    })
}

/// 同名替换计划：`(本体武将, 变体武将)` 逐位配对（配置开启时非空）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplacePlan {
    pub pairs: Vec<(String, String)>,
}

impl ThunderConfig {
    /// 同名替换计划（`replaceCha` 关闭时为空）。
    pub fn replace_plan(&self) -> ReplacePlan {
        if !self.replace_cha {
            return ReplacePlan { pairs: Vec::new() };
        }
        ReplacePlan {
            pairs: SAME_HERO
                .iter()
                .zip(CHG_HERO.iter())
                .map(|(a, b)| ((*a).to_string(), (*b).to_string()))
                .collect(),
        }
    }

    /// 是否隐藏专属包（`hidePack`）。
    pub fn hides_exclusive_pack(&self) -> bool {
        self.hide_pack
    }

    /// 小游戏速度档位文案（`gameSpeed`）。
    pub fn game_speed_label(&self) -> &'static str {
        GAME_SPEED_LABELS[usize::from(self.game_speed.min(8))]
    }

    /// 实际对话框主题：`follow` 跟随十周年 UI 设定（传入其当前风格）。
    pub fn effective_dialog_theme<'a>(&'a self, decade_style: &'a str) -> &'a str {
        if self.yuqi_theme == "follow" {
            decade_style
        } else {
            &self.yuqi_theme
        }
    }

    /// 实际进度条风格：`follow` 跟随十周年 UI 设定。
    pub fn effective_time_theme<'a>(&'a self, decade_style: &'a str) -> &'a str {
        if self.time_theme == "follow" {
            decade_style
        } else {
            &self.time_theme
        }
    }

    /// 立绘文件路径（`standPath` + 皮肤同名文件）。
    pub fn stand_image_path(&self, skin: &str) -> String {
        format!("{}{}.png", self.stand_path, skin)
    }
}

/// 标记文本补丁（上游 `lib.translate` 重写；`th_chongxu` 的 `init`）。
///
/// 返回 `(标记键, 文本)` 列表：孙寒华妙剑/莲华按等级后缀（`shh_new` 用上标
/// 数字 `⁰¹²³⁴⁵⁶⁷⁸⁹`，否则「N级」）。
pub fn mark_labels(cfg: &ThunderConfig, miaojian: u8, lianhua: u8) -> Vec<(String, String)> {
    if !cfg.shh_biaoji {
        return Vec::new();
    }
    const SUPER: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];
    let render = |n: u8| -> String {
        if cfg.shh_new {
            SUPER
                .get(n as usize)
                .map(|c| c.to_string())
                .unwrap_or_else(|| n.to_string())
        } else {
            format!("{}级", n)
        }
    };
    vec![
        ("miaojian".into(), format!("妙剑{}", render(miaojian + 1))),
        ("shhlianhua".into(), format!("莲华{}", render(lianhua + 1))),
    ]
}
