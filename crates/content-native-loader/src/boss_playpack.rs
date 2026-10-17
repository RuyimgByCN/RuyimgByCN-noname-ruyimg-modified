//! U02 / L03 boss「play 包」的可移植决策（playpackconfig）。
//!
//! 上游 `boss` 是一个 `game.import("play", …)` 扩展：`init` 从
//! `localStorage['boss_storage_playpackconfig']` 读入挑战/剑阁武将包与翻译，
//! 归一化体力（Infinity）、按 `boss_enableai_playpackconfig` 追加禁 AI 名单、
//! 写翻译默认与 intro 别名；`arenaReady` 过滤 `isBossAllowed` 并把模式技能表
//! 合并（排除 `versus_ladder`）。两仓实现语义一致，仅包键/标签/别名/允许旗标
//! 不同（noname：`boss`/`jiange`/`isBossAllowed`；liuli：`mode_extension_boss`
//! /`mode_extension_jiange`/标签 `mode:boss`/`bossallowed`）。
//!
//! 本模块只做纯数据变换：不读写 localStorage、不加载模式、不画界面、不播音。

use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayError {
    BadConfig(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HpKind {
    Number(u32),
    /// 上游允许 `"3/4"` 形式的分数体力（仅 boss 包保留）。
    Fraction(String),
    Infinity,
}

impl HpKind {
    pub fn label(&self) -> String {
        match self {
            HpKind::Number(n) => n.to_string(),
            HpKind::Fraction(s) => s.clone(),
            HpKind::Infinity => "Infinity".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub hp: HpKind,
    pub tags: Vec<String>,
    /// 允许进 boss/jiange 池（noname `isBossAllowed`，liuli tags 含
    /// `bossallowed`）——两来源统一解析到本字段。
    pub allowed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PlayStorage {
    pub boss: Vec<Entry>,
    pub versus: Vec<Entry>,
    pub translate: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BossPlayConfig {
    pub source: String,
    pub upstream_path: String,
    pub upstream_commit: String,
    pub boss_pack_key: String,
    pub jiange_pack_key: String,
    pub boss_translate_key: String,
    pub jiange_translate_key: String,
    pub boss_translate_default: String,
    pub jiange_translate_default: String,
    /// liuli 追加的标签（noname 为 `None`）。
    pub tag_boss: Option<String>,
    pub tag_versus: Option<String>,
    pub intro_aliases: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InitOutcome {
    /// 模式为 `tafang` 时早退，其余字段为空。
    pub skipped: bool,
    pub boss_pack: Vec<Entry>,
    pub jiange_pack: Vec<Entry>,
    /// `lib.character` 覆盖（boss 包后接剑阁包或 else 分支）。
    pub characters: Vec<Entry>,
    pub forbidai: Vec<String>,
    /// 归一化后的翻译表（默认键已覆盖 storage 值）。
    pub translate: Vec<(String, String)>,
    pub intro: Vec<(String, String)>,
}

/// 上游 `init`：模式门控 + 体力归一 + 禁 AI + 翻译默认与 intro 别名。
pub fn init_outcome(
    cfg: &BossPlayConfig,
    mode: &str,
    versus_mode: &str,
    status_mode: &str,
    storage: &PlayStorage,
    enableai: bool,
) -> InitOutcome {
    if mode == "tafang" {
        return InitOutcome {
            skipped: true,
            ..Default::default()
        };
    }
    let mut out = InitOutcome::default();
    if mode != "boss" {
        for entry in &storage.boss {
            let mut e = entry.clone();
            e.hp = normalize_boss_hp(&entry.hp);
            if let Some(tag) = &cfg.tag_boss {
                e.tags.push(tag.clone());
            }
            out.boss_pack.push(e.clone());
            out.characters.push(e.clone());
            if !enableai {
                out.forbidai.push(e.name.clone());
            }
        }
    }
    if mode != "versus" || versus_mode != "jiange" {
        for entry in &storage.versus {
            let mut e = entry.clone();
            e.hp = normalize_jiange_hp(&entry.hp);
            if let Some(tag) = &cfg.tag_versus {
                e.tags.push(tag.clone());
            }
            out.jiange_pack.push(e.clone());
            out.characters.push(e.clone());
            if !enableai {
                out.forbidai.push(e.name.clone());
            }
        }
        out.intro = cfg.intro_aliases.clone();
    } else if status_mode != "jiange" {
        for entry in &storage.versus {
            out.characters.push(entry.clone());
            if !enableai {
                out.forbidai.push(entry.name.clone());
            }
        }
    }
    // 翻译：storage 副本 + 默认键覆盖，再按 `||=` 合并进现有表。
    out.translate = storage.translate.clone();
    set_translate(&mut out.translate, &cfg.boss_translate_key, &cfg.boss_translate_default);
    set_translate(
        &mut out.translate,
        &cfg.jiange_translate_key,
        &cfg.jiange_translate_default,
    );
    out
}

fn set_translate(list: &mut Vec<(String, String)>, key: &str, value: &str) {
    match list.iter_mut().find(|(k, _)| k == key) {
        Some(slot) => slot.1 = value.to_string(),
        None => list.push((key.to_string(), value.to_string())),
    }
}

/// boss 包体力归一：数字保留；含 `/` 的字符串保留；其余 → Infinity。
pub fn normalize_boss_hp(hp: &HpKind) -> HpKind {
    match hp {
        HpKind::Number(_) => hp.clone(),
        HpKind::Fraction(_) => hp.clone(),
        HpKind::Infinity => HpKind::Infinity,
    }
}

/// 剑阁包体力归一：数字保留；其余（含分数）→ Infinity。
pub fn normalize_jiange_hp(hp: &HpKind) -> HpKind {
    match hp {
        HpKind::Number(_) => hp.clone(),
        _ => HpKind::Infinity,
    }
}

/// `arenaReady` 的池过滤：仅保留 `isBossAllowed` / `bossallowed` 者。
pub fn filter_allowed(pack: &[Entry]) -> Vec<Entry> {
    pack.iter().filter(|e| e.allowed).cloned().collect()
}

/// 翻译合并（`lib.translate[i] ||= list[i]`）：现有值优先，缺键补入。
pub fn merge_translate(
    existing: &[(String, String)],
    incoming: &[(String, String)],
) -> Vec<(String, String)> {
    let mut out = existing.to_vec();
    for (k, v) in incoming {
        if !out.iter().any(|(ek, _)| ek == k) {
            out.push((k.clone(), v.clone()));
        }
    }
    out
}

/// 模式技能表合并：排除 `versus_ladder`。
pub fn exclude_versus_ladder(names: &[String]) -> Vec<String> {
    names
        .iter()
        .filter(|n| n.as_str() != "versus_ladder")
        .cloned()
        .collect()
}

pub fn parse_boss_play_config(text: &str) -> Result<BossPlayConfig, PlayError> {
    let value: Value =
        serde_json::from_str(text).map_err(|_| PlayError::BadConfig("json".into()))?;
    let obj = value
        .as_object()
        .ok_or_else(|| PlayError::BadConfig("json".into()))?;
    let source = req_str(obj, "source")?;
    if source.is_empty() {
        return Err(PlayError::BadConfig("source".into()));
    }
    let aliases = req_array(obj, "intro_aliases")?;
    let mut intro_aliases = Vec::new();
    for item in aliases {
        let pair = item
            .as_array()
            .ok_or_else(|| PlayError::BadConfig("intro_aliases".into()))?;
        if pair.len() != 2 {
            return Err(PlayError::BadConfig("intro_aliases".into()));
        }
        let from = pair[0]
            .as_str()
            .ok_or_else(|| PlayError::BadConfig("intro_aliases".into()))?;
        let to = pair[1]
            .as_str()
            .ok_or_else(|| PlayError::BadConfig("intro_aliases".into()))?;
        intro_aliases.push((from.to_string(), to.to_string()));
    }
    Ok(BossPlayConfig {
        source: source.to_string(),
        upstream_path: req_str(obj, "upstream_path")?.to_string(),
        upstream_commit: req_str(obj, "upstream_commit")?.to_string(),
        boss_pack_key: req_str(obj, "boss_pack_key")?.to_string(),
        jiange_pack_key: req_str(obj, "jiange_pack_key")?.to_string(),
        boss_translate_key: req_str(obj, "boss_translate_key")?.to_string(),
        jiange_translate_key: req_str(obj, "jiange_translate_key")?.to_string(),
        boss_translate_default: req_str(obj, "boss_translate_default")?.to_string(),
        jiange_translate_default: req_str(obj, "jiange_translate_default")?.to_string(),
        tag_boss: opt_str(obj, "tag_boss"),
        tag_versus: opt_str(obj, "tag_versus"),
        intro_aliases,
    })
}

fn req_str<'a>(
    obj: &'a serde_json::Map<String, Value>,
    key: &'static str,
) -> Result<&'a str, PlayError> {
    match obj.get(key) {
        Some(Value::String(text)) => Ok(text),
        _ => Err(PlayError::BadConfig(key.into())),
    }
}

fn opt_str(obj: &serde_json::Map<String, Value>, key: &'static str) -> Option<String> {
    match obj.get(key) {
        Some(Value::String(text)) => Some(text.clone()),
        _ => None,
    }
}

fn req_array<'a>(
    obj: &'a serde_json::Map<String, Value>,
    key: &'static str,
) -> Result<&'a Vec<Value>, PlayError> {
    match obj.get(key) {
        Some(Value::Array(items)) => Ok(items),
        _ => Err(PlayError::BadConfig(key.into())),
    }
}

/// 解析 `boss_storage_playpackconfig`（上游 `try JSON.parse || {}`：非法输入
/// 视为空存储，不报错）。
pub fn parse_storage(text: &str) -> PlayStorage {
    let Ok(value) = serde_json::from_str::<Value>(text) else {
        return PlayStorage::default();
    };
    let Some(obj) = value.as_object() else {
        return PlayStorage::default();
    };
    PlayStorage {
        boss: parse_pack(obj.get("boss")),
        versus: parse_pack(obj.get("versus")),
        translate: match obj.get("translate") {
            Some(Value::Object(m)) => m
                .iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect(),
            _ => Vec::new(),
        },
    }
}

fn parse_pack(value: Option<&Value>) -> Vec<Entry> {
    let Some(Value::Object(map)) = value else {
        return Vec::new();
    };
    map.iter()
        .map(|(name, entry)| {
            let hp = match entry.get("hp") {
                Some(Value::Number(n)) => HpKind::Number(n.as_u64().unwrap_or(0) as u32),
                Some(Value::String(s)) if s.contains('/') => HpKind::Fraction(s.clone()),
                _ => HpKind::Infinity,
            };
            let tags: Vec<String> = match entry.get("tags") {
                Some(Value::Array(items)) => items
                    .iter()
                    .filter_map(|t| t.as_str().map(str::to_string))
                    .collect(),
                _ => Vec::new(),
            };
            let allowed = entry
                .get("isBossAllowed")
                .and_then(Value::as_bool)
                .unwrap_or(false)
                || tags.iter().any(|t| t == "bossallowed");
            Entry {
                name: name.clone(),
                hp,
                tags,
                allowed,
            }
        })
        .collect()
}

/// 目标平台字符串辅助（engine-native-wasm）：把一次 play 包决策压成一行。
pub fn apply_boss_play_line(config_json: &str, request_json: &str) -> Result<String, PlayError> {
    let cfg = parse_boss_play_config(config_json)?;
    let value: Value =
        serde_json::from_str(request_json).map_err(|_| PlayError::BadConfig("request".into()))?;
    let obj = value
        .as_object()
        .ok_or_else(|| PlayError::BadConfig("request".into()))?;
    let mode = obj.get("mode").and_then(Value::as_str).unwrap_or("prototype");
    let versus_mode = obj.get("versus_mode").and_then(Value::as_str).unwrap_or("");
    let status_mode = obj.get("status_mode").and_then(Value::as_str).unwrap_or("");
    let enableai = obj
        .get("enableai")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let storage = match obj.get("storage") {
        Some(s) => parse_storage(&s.to_string()),
        None => PlayStorage::default(),
    };
    let out = init_outcome(&cfg, mode, versus_mode, status_mode, &storage, enableai);
    let skills: Vec<String> = match obj.get("skills") {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|i| i.as_str().map(str::to_string))
            .collect(),
        _ => Vec::new(),
    };
    let kept_skills = exclude_versus_ladder(&skills);
    let allowed = filter_allowed(&out.jiange_pack).len();
    Ok(format!(
        "skipped:{} | boss:{} | jiange:{} | chars:{} | forbidai:{} | allowed_jiange:{} | skills:{} | translate:{} | intro:{}",
        u8::from(out.skipped),
        out.boss_pack.len(),
        out.jiange_pack.len(),
        out.characters.len(),
        out.forbidai.len(),
        allowed,
        kept_skills.len(),
        out.translate.len(),
        out.intro.len(),
    ))
}
