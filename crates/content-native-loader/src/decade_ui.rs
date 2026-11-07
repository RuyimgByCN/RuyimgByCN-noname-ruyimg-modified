//! L08 十周年UI（`archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js`）
//! 配置解析、子模块清单与状态快照模型。
//!
//! 上游 `decadeModule.init` 通过 `<script>` 动态注入 14 个 JS 子模块、CSS 样式表与
//! `window.decadeUI` 全局单例，并在客户端覆写 DOM 动画/Spine 骨骼/手牌折叠。
//! 原生层将展示层样式选择收口为确定性配置结构 [`DecadeUiConfig`]，不执行动态脚本注入，
//! 不修改内核规则与权威状态哈希。

use serde_json::Value;

/// 上游 `decadeModule.init` 的 14 个 JS 子模块清单（按上游声明顺序）。
pub const DECADE_UI_SUBMODULES: [&str; 14] = [
    "spine.js",
    "component.js",
    "skill.js",
    "content.js",
    "effect.js",
    "dynamicSkin.js",
    "menu.js",
    "animationlist.js",
    "meihua.js",
    "splash.js",
    "markskill.js",
    "pause.js",
    "animation.js",
    "animation_new.js",
];

/// G0 盘点基数：`(skill_count, mark_skill_count, card_count, character_count, translate_count)`。
pub const DECADE_UI_COUNTS: (usize, usize, usize, usize, usize) = (332, 209, 116, 178, 122);

/// 十周年UI 整体样式切换（对应上游 `config.newDecadeStyle`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecadeStyle {
    /// 十周年样式（`"on"`）。
    On,
    /// 手杀样式（`"off"`）。
    Off,
    /// 新杀样式（`"othersOff"`）。
    OthersOff,
    /// OL 样式（`"onlineUI"`）。
    OnlineUI,
    /// 欢乐三国杀样式（`"babysha"`）。
    BabySha,
    /// 代号杀样式（`"codename"`）。
    Codename,
}

impl DecadeStyle {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "on" => Some(Self::On),
            "off" => Some(Self::Off),
            "othersOff" => Some(Self::OthersOff),
            "onlineUI" => Some(Self::OnlineUI),
            "babysha" => Some(Self::BabySha),
            "codename" => Some(Self::Codename),
            _ => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::On => "on",
            Self::Off => "off",
            Self::OthersOff => "othersOff",
            Self::OnlineUI => "onlineUI",
            Self::BabySha => "babysha",
            Self::Codename => "codename",
        }
    }

    /// 对应上游 `decadeUI.config.newDecadeStyle` 在 `skinSwitch` 等联动中的样式通道名。
    pub const fn skin_channel(self) -> &'static str {
        match self {
            Self::On => "shizhounian",
            Self::Off => "shousha",
            Self::OthersOff => "xinsha",
            Self::OnlineUI => "ol",
            Self::BabySha => "huanle",
            Self::Codename => "codename",
        }
    }
}

/// 卡牌美化风格（对应上游 `config.cardPrettify`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardPrettify {
    Off,
    Original,
    Decade,
    OL,
}

impl CardPrettify {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "off" => Some(Self::Off),
            "original" => Some(Self::Original),
            "decade" => Some(Self::Decade),
            "ol" => Some(Self::OL),
            _ => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Original => "original",
            Self::Decade => "decade",
            Self::OL => "ol",
        }
    }
}

/// 标记样式（对应上游 `config.markStyle`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkStyle {
    Red,
    Yellow,
    Decade,
}

impl MarkStyle {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "red" => Some(Self::Red),
            "yellow" => Some(Self::Yellow),
            "decade" => Some(Self::Decade),
            _ => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Red => "red",
            Self::Yellow => "yellow",
            Self::Decade => "decade",
        }
    }
}

/// L08 十周年UI 配置。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecadeUiConfig {
    pub new_decade_style: DecadeStyle,
    pub dynamic_skin: bool,
    pub card_prettify: CardPrettify,
    pub hand_fold: bool,
    pub phase_tip: bool,
    pub damage_pop: bool,
    pub equip_separate: bool,
    pub mark_style: MarkStyle,
}

impl Default for DecadeUiConfig {
    fn default() -> Self {
        Self {
            new_decade_style: DecadeStyle::On,
            dynamic_skin: false,
            card_prettify: CardPrettify::Decade,
            hand_fold: true,
            phase_tip: true,
            damage_pop: true,
            equip_separate: true,
            mark_style: MarkStyle::Decade,
        }
    }
}

/// L08 十周年UI 配置与指令错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecadeUiError {
    BadJson(String),
    MissingField(&'static str),
    InvalidStyle(String),
    InvalidCardPrettify(String),
    InvalidMarkStyle(String),
    InvalidBool(&'static str),
    InvalidCount { field: &'static str, got: usize, expected: usize },
    UnknownCommand(String),
    SnapshotMalformed(String),
}

impl std::fmt::Display for DecadeUiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadJson(e) => write!(f, "十周年UI JSON 解析失败: {e}"),
            Self::MissingField(fld) => write!(f, "十周年UI 缺少字段: {fld}"),
            Self::InvalidStyle(v) => write!(f, "十周年UI 非法 newDecadeStyle: {v}"),
            Self::InvalidCardPrettify(v) => write!(f, "十周年UI 非法 cardPrettify: {v}"),
            Self::InvalidMarkStyle(v) => write!(f, "十周年UI 非法 markStyle: {v}"),
            Self::InvalidBool(fld) => write!(f, "十周年UI 字段 {fld} 必须为布尔值"),
            Self::InvalidCount { field, got, expected } => {
                write!(f, "十周年UI 计数 {field} 不匹配: 实际 {got}, 期望 {expected}")
            }
            Self::UnknownCommand(cmd) => write!(f, "十周年UI 未知指令: {cmd}"),
            Self::SnapshotMalformed(reason) => write!(f, "十周年UI 快照损坏: {reason}"),
        }
    }
}

impl std::error::Error for DecadeUiError {}

fn read_bool(obj: &serde_json::Map<String, Value>, camel: &'static str, snake: &'static str) -> Result<bool, DecadeUiError> {
    let val = obj
        .get(camel)
        .or_else(|| obj.get(snake))
        .ok_or(DecadeUiError::MissingField(camel))?;
    val.as_bool().ok_or(DecadeUiError::InvalidBool(camel))
}

fn read_str<'a>(obj: &'a serde_json::Map<String, Value>, camel: &'static str, snake: &'static str) -> Result<&'a str, DecadeUiError> {
    let val = obj
        .get(camel)
        .or_else(|| obj.get(snake))
        .ok_or(DecadeUiError::MissingField(camel))?;
    val.as_str().ok_or(DecadeUiError::MissingField(camel))
}

/// 解析 `decade-ui-upstream-liuli/assets/config.json` 或直接的 `default_config` 对象。
pub fn parse_decade_ui_config(raw: &str) -> Result<DecadeUiConfig, DecadeUiError> {
    let root: Value = serde_json::from_str(raw).map_err(|e| DecadeUiError::BadJson(e.to_string()))?;
    let root_obj = root
        .as_object()
        .ok_or_else(|| DecadeUiError::BadJson("根值必须为 JSON 对象".into()))?;

    let cfg_obj = if let Some(inner) = root_obj.get("default_config") {
        // 同时核验顶层 G0 计数（若提供）
        for (field, expected) in [
            ("skill_count", DECADE_UI_COUNTS.0),
            ("mark_skill_count", DECADE_UI_COUNTS.1),
            ("card_count", DECADE_UI_COUNTS.2),
            ("character_count", DECADE_UI_COUNTS.3),
            ("translate_count", DECADE_UI_COUNTS.4),
        ] {
            if let Some(v) = root_obj.get(field) {
                let got = v.as_u64().ok_or(DecadeUiError::MissingField(field))? as usize;
                if got != expected {
                    return Err(DecadeUiError::InvalidCount { field, got, expected });
                }
            }
        }
        inner
            .as_object()
            .ok_or(DecadeUiError::MissingField("default_config"))?
    } else {
        root_obj
    };

    let style_raw = read_str(cfg_obj, "newDecadeStyle", "new_decade_style")?;
    let new_decade_style = DecadeStyle::parse(style_raw)
        .ok_or_else(|| DecadeUiError::InvalidStyle(style_raw.to_string()))?;

    let prettify_raw = read_str(cfg_obj, "cardPrettify", "card_prettify")?;
    let card_prettify = CardPrettify::parse(prettify_raw)
        .ok_or_else(|| DecadeUiError::InvalidCardPrettify(prettify_raw.to_string()))?;

    let mark_raw = read_str(cfg_obj, "markStyle", "mark_style")?;
    let mark_style = MarkStyle::parse(mark_raw)
        .ok_or_else(|| DecadeUiError::InvalidMarkStyle(mark_raw.to_string()))?;

    Ok(DecadeUiConfig {
        new_decade_style,
        dynamic_skin: read_bool(cfg_obj, "dynamicSkin", "dynamic_skin")?,
        card_prettify,
        hand_fold: read_bool(cfg_obj, "handFold", "hand_fold")?,
        phase_tip: read_bool(cfg_obj, "phaseTip", "phase_tip")?,
        damage_pop: read_bool(cfg_obj, "damagePop", "damage_pop")?,
        equip_separate: read_bool(cfg_obj, "equipSeparate", "equip_separate")?,
        mark_style,
    })
}

/// 序列化为确定性单行快照。
pub fn snapshot_decade_ui(cfg: &DecadeUiConfig) -> String {
    format!(
        "style={} | dynamic_skin={} | card_prettify={} | hand_fold={} | phase_tip={} | damage_pop={} | equip_separate={} | mark_style={}",
        cfg.new_decade_style.as_str(),
        u8::from(cfg.dynamic_skin),
        cfg.card_prettify.as_str(),
        u8::from(cfg.hand_fold),
        u8::from(cfg.phase_tip),
        u8::from(cfg.damage_pop),
        u8::from(cfg.equip_separate),
        cfg.mark_style.as_str(),
    )
}

/// 从单行快照恢复配置。
pub fn restore_decade_ui(snap: &str) -> Result<DecadeUiConfig, DecadeUiError> {
    let mut cfg = DecadeUiConfig::default();
    let mut seen = 0usize;
    for part in snap.split('|') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (k, v) = part
            .split_once('=')
            .ok_or_else(|| DecadeUiError::SnapshotMalformed(part.to_string()))?;
        let k = k.trim();
        let v = v.trim();
        let parse_bit = |fld: &'static str, s: &str| match s {
            "1" | "true" => Ok(true),
            "0" | "false" => Ok(false),
            _ => Err(DecadeUiError::InvalidBool(fld)),
        };
        match k {
            "style" => {
                cfg.new_decade_style = DecadeStyle::parse(v)
                    .ok_or_else(|| DecadeUiError::InvalidStyle(v.to_string()))?;
            }
            "dynamic_skin" => cfg.dynamic_skin = parse_bit("dynamicSkin", v)?,
            "card_prettify" => {
                cfg.card_prettify = CardPrettify::parse(v)
                    .ok_or_else(|| DecadeUiError::InvalidCardPrettify(v.to_string()))?;
            }
            "hand_fold" => cfg.hand_fold = parse_bit("handFold", v)?,
            "phase_tip" => cfg.phase_tip = parse_bit("phaseTip", v)?,
            "damage_pop" => cfg.damage_pop = parse_bit("damagePop", v)?,
            "equip_separate" => cfg.equip_separate = parse_bit("equipSeparate", v)?,
            "mark_style" => {
                cfg.mark_style = MarkStyle::parse(v)
                    .ok_or_else(|| DecadeUiError::InvalidMarkStyle(v.to_string()))?;
            }
            other => return Err(DecadeUiError::SnapshotMalformed(other.to_string())),
        }
        seen += 1;
    }
    if seen < 8 {
        return Err(DecadeUiError::SnapshotMalformed("字段不足 8 项".into()));
    }
    Ok(cfg)
}

/// 执行单行诊断命令（`config <json>` / `style <raw>` / `snapshot` / `restore <snap>`）。
pub fn apply_decade_ui_line(cfg: &mut DecadeUiConfig, line: &str) -> Result<String, DecadeUiError> {
    let trimmed = line.trim();
    if trimmed == "snapshot" {
        return Ok(snapshot_decade_ui(cfg));
    }
    if trimmed == "submodules" {
        return Ok(format!(
            "count={} | first={} | last={} | script_injected=0",
            DECADE_UI_SUBMODULES.len(),
            DECADE_UI_SUBMODULES[0],
            DECADE_UI_SUBMODULES[DECADE_UI_SUBMODULES.len() - 1]
        ));
    }
    let (cmd, rest) = trimmed
        .split_once(' ')
        .ok_or_else(|| DecadeUiError::UnknownCommand(trimmed.to_string()))?;
    let rest = rest.trim();
    match cmd {
        "config" => {
            *cfg = parse_decade_ui_config(rest)?;
            Ok(snapshot_decade_ui(cfg))
        }
        "style" => {
            let style = DecadeStyle::parse(rest)
                .ok_or_else(|| DecadeUiError::InvalidStyle(rest.to_string()))?;
            cfg.new_decade_style = style;
            Ok(format!(
                "style={} | channel={} | dom_mutated=0",
                style.as_str(),
                style.skin_channel()
            ))
        }
        "restore" => {
            *cfg = restore_decade_ui(rest)?;
            Ok(snapshot_decade_ui(cfg))
        }
        other => Err(DecadeUiError::UnknownCommand(other.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_roundtrip_decade_ui_config() {
        let json = r#"{
            "extension_name": "十周年UI",
            "source": "noname-liuli",
            "upstream_path": "archive/upstream/noname-liuli/extension/十周年UI/extension.js",
            "default_config": {
                "newDecadeStyle": "on",
                "dynamicSkin": false,
                "cardPrettify": "decade",
                "handFold": true,
                "phaseTip": true,
                "damagePop": true,
                "equipSeparate": true,
                "markStyle": "decade"
            },
            "skill_count": 332,
            "mark_skill_count": 209,
            "card_count": 116,
            "character_count": 178,
            "translate_count": 122
        }"#;
        let mut cfg = parse_decade_ui_config(json).expect("valid config");
        assert_eq!(cfg, DecadeUiConfig::default());
        assert_eq!(DECADE_UI_SUBMODULES.len(), 14);

        let line_out = apply_decade_ui_line(&mut cfg, "style onlineUI").unwrap();
        assert_eq!(line_out, "style=onlineUI | channel=ol | dom_mutated=0");
        let snap = apply_decade_ui_line(&mut cfg, "snapshot").unwrap();
        let mut restored = DecadeUiConfig::default();
        apply_decade_ui_line(&mut restored, &format!("restore {snap}")).unwrap();
        assert_eq!(restored.new_decade_style, DecadeStyle::OnlineUI);
    }
}
