//! L15 手杀全扩。改名、将包筛选和禁将走调用方给出的武将表。
//! 不改稳定 id。不播放奶龙音频，不画界面。

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

const INFO_ENDINGS: [&str; 6] = ["。", "；", "）", ")", "}", ">"];
const POOL_RULES: &[(&str, &str)] = &[
    ("standard", "standard"),
    ("refresh", "refresh"),
    ("jiangshan", "jiangshan"),
    ("refresh", "shiji"),
    ("sp", "sp"),
    ("sp", "tw"),
    ("sp2", "sp2"),
    ("extra", "extra"),
    ("mobile", "mobile"),
    ("shenhua", "shenhua"),
    ("yijiang", "yijiang"),
    ("yingbian", "yingbian"),
    ("xinghuoliaoyuan", "xinghuoliaoyuan"),
    ("sb", "sb"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShoushaConfig {
    pub source: String,
    pub upstream_path: String,
    pub upstream_commit: String,
    pub jinjiang: bool,
    pub jinjiang2: bool,
    pub huanyuan: bool,
    pub yinjian: bool,
    pub unseen: String,
    pub packs: bool,
    pub txhj: bool,
    pub sex_forbid: String,
    pub shousha_name: String,
    pub nailong: bool,
    pub infofix: bool,
    pub change_group: bool,
    tables: Tables,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Tables {
    base: BTreeMap<String, String>,
    doudizhu: BTreeMap<String, String>,
    huanyuan: BTreeMap<String, String>,
    yinjian: BTreeMap<String, String>,
    jinjiang_packs: Vec<String>,
    ban_packs: Vec<String>,
    exclude: BTreeMap<String, Vec<String>>,
    add: Vec<String>,
    bt_packs: Vec<String>,
    bt_exclude_extra: Vec<String>,
    bt_add: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShoushaRequest {
    pub mode: String,
    pub ban_bt: bool,
    pub characters: Vec<CharacterIn>,
    pub packs: Vec<String>,
    pub pack_members: BTreeMap<String, Vec<String>>,
    pub skills: Vec<SkillIn>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterIn {
    pub id: String,
    pub gender: String,
    pub pack: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillIn {
    pub id: String,
    pub info: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterOut {
    pub id: String,
    pub name: String,
    pub tags: Vec<String>,
    pub group: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillOut {
    pub id: String,
    pub info: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShoushaReport {
    pub characters: Vec<CharacterOut>,
    pub packs: Vec<String>,
    pub forbidai: Vec<String>,
    pub re_sunben: Option<String>,
    pub skills: Vec<SkillOut>,
    pub audio_played: bool,
    pub audio_looped: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShoushaError {
    BadConfig(String),
    MissingPack(String),
    BadName(String),
}

impl ShoushaConfig {
    pub fn base_len(&self) -> usize {
        self.tables.base.len()
    }

    pub fn doudizhu_len(&self) -> usize {
        self.tables.doudizhu.len()
    }

    pub fn huanyuan_len(&self) -> usize {
        self.tables.huanyuan.len()
    }

    pub fn yinjian_len(&self) -> usize {
        self.tables.yinjian.len()
    }

    pub fn jinjiang_packs(&self) -> &[String] {
        &self.tables.jinjiang_packs
    }

    pub fn mapped_name(&self, which: &str, id: &str) -> Option<&str> {
        let map = match which {
            "base" => &self.tables.base,
            "doudizhu" => &self.tables.doudizhu,
            "huanyuan" => &self.tables.huanyuan,
            "yinjian" => &self.tables.yinjian,
            _ => return None,
        };
        map.get(id).map(String::as_str)
    }
}

pub fn parse_shousha_config(text: &str) -> Result<ShoushaConfig, ShoushaError> {
    let value: Value =
        serde_json::from_str(text).map_err(|err| ShoushaError::BadConfig(err.to_string()))?;
    let obj = value
        .as_object()
        .ok_or_else(|| ShoushaError::BadConfig("config must be an object".into()))?;
    let source = token(obj.get("source"), "source")?;
    let upstream_path = required_text(obj, "upstream_path")?;
    let upstream_commit = required_text(obj, "upstream_commit")?;
    if upstream_path.is_empty() || upstream_commit.is_empty() {
        return Err(ShoushaError::BadConfig(
            "upstream identity must be non-empty".into(),
        ));
    }
    let tables = parse_tables(obj.get("tables"))?;
    Ok(ShoushaConfig {
        source,
        upstream_path,
        upstream_commit,
        jinjiang: required_bool(obj, "jinjiang")?,
        jinjiang2: required_bool(obj, "jinjiang2")?,
        huanyuan: required_bool(obj, "huanyuan")?,
        yinjian: required_bool(obj, "yinjian")?,
        unseen: required_text(obj, "unseen")?,
        packs: required_bool(obj, "packs")?,
        txhj: required_bool(obj, "txhj")?,
        sex_forbid: required_text(obj, "sexforbid")?,
        shousha_name: required_text(obj, "shoushanameX")?,
        nailong: required_bool(obj, "nailong")?,
        infofix: required_bool(obj, "infofix")?,
        change_group: match obj.get("changeGroup") {
            None => false,
            Some(Value::Bool(value)) => *value,
            Some(_) => return Err(ShoushaError::BadConfig("changeGroup must be a bool".into())),
        },
        tables,
    })
}

pub fn apply_shousha(
    config: &ShoushaConfig,
    request: &ShoushaRequest,
) -> Result<ShoushaReport, ShoushaError> {
    let mut characters: Vec<CharacterOut> = request
        .characters
        .iter()
        .map(|character| CharacterOut {
            id: character.id.clone(),
            name: character.name.clone(),
            tags: Vec::new(),
            group: None,
        })
        .collect();
    if config.jinjiang {
        let tag = if config.unseen.is_empty() {
            "unseen"
        } else {
            config.unseen.as_str()
        };
        for (out, input) in characters.iter_mut().zip(request.characters.iter()) {
            let listed = config.tables.base.contains_key(&input.id);
            let doudizhu =
                request.mode == "doudizhu" && config.tables.doudizhu.contains_key(&input.id);
            if !listed && !doudizhu {
                add_tag(&mut out.tags, tag);
            }
        }
    }
    let forbidai = if config.jinjiang2 {
        let allowed = allowed_ids(&config.tables, request.ban_bt, &request.pack_members)?;
        request
            .characters
            .iter()
            .filter(|character| {
                !allowed.iter().any(|id| id == &character.id) && !character.id.contains("boss_")
            })
            .map(|character| character.id.clone())
            .collect()
    } else {
        Vec::new()
    };
    if config.huanyuan {
        apply_pool_tags(
            &mut characters,
            &config.tables.huanyuan,
            config.change_group,
        );
    }
    if config.yinjian {
        apply_pool_tags(&mut characters, &config.tables.yinjian, config.change_group);
    }
    if config.txhj && request.mode != "taixuhuanjing" {
        for character in &mut characters {
            if character.id.contains("txhj_") {
                add_tag(&mut character.tags, "unseen");
            }
        }
    }
    if config.sex_forbid == "male" || config.sex_forbid == "female" {
        for (out, input) in characters.iter_mut().zip(request.characters.iter()) {
            if input.gender != config.sex_forbid {
                add_tag(&mut out.tags, "forbidai");
            }
        }
    }
    let mut packs = request.packs.clone();
    if config.jinjiang && config.packs {
        packs = keep_listed(&packs, &config.tables.jinjiang_packs);
    }
    if config.huanyuan {
        packs = pool_filter(&packs);
    }
    if config.yinjian {
        packs = pool_filter(&packs);
    }
    let mut re_sunben = None;
    if config.jinjiang || config.shousha_name != "none" {
        re_sunben = Some("界孙策".to_string());
    }
    if config.shousha_name != "none" {
        let map = rename_map(config);
        let mut tralist = Vec::new();
        for character in &request.characters {
            if map.contains_key(&character.id)
                && !character.name.is_empty()
                && !tralist.contains(&character.name)
            {
                tralist.push(character.name.clone());
            }
        }
        let prefixes: &[&str] = if config.shousha_name == "shousha" {
            &["手杀", "新杀"]
        } else {
            &["手杀", "旧", "OL", "新杀", "TW", "DC"]
        };
        for character in &mut characters {
            if character.name.is_empty() {
                return Err(ShoushaError::BadName(character.id.clone()));
            }
            let original = character.name.clone();
            for prefix in prefixes {
                let Some(real) = original.strip_prefix(prefix) else {
                    continue;
                };
                let pre = if original.starts_with("手杀") && tralist.iter().any(|name| name == real)
                {
                    "界"
                } else {
                    ""
                };
                character.name = format!("{pre}{real}");
            }
            if character.id == "re_sunben" {
                re_sunben = Some(character.name.clone());
            }
        }
    }
    let skills = request
        .skills
        .iter()
        .map(|skill| SkillOut {
            id: skill.id.clone(),
            info: if config.infofix {
                fix_info(&skill.id, &skill.info)
            } else {
                skill.info.clone()
            },
        })
        .collect();
    Ok(ShoushaReport {
        characters,
        packs,
        forbidai,
        re_sunben,
        skills,
        audio_played: false,
        audio_looped: false,
    })
}

pub fn apply_shousha_line(config_json: &str, request_json: &str) -> Result<String, ShoushaError> {
    let config = parse_shousha_config(config_json)?;
    let request = parse_request(request_json)?;
    Ok(format_report(&apply_shousha(&config, &request)?))
}

pub fn format_report(report: &ShoushaReport) -> String {
    let characters = report
        .characters
        .iter()
        .map(|character| {
            let tags = if character.tags.is_empty() {
                "-".to_string()
            } else {
                character.tags.join("+")
            };
            format!("{}:{}:{tags}", character.id, character.name)
        })
        .collect::<Vec<_>>()
        .join(",");
    let skills = report
        .skills
        .iter()
        .map(|skill| format!("{}={}", skill.id, skill.info))
        .collect::<Vec<_>>()
        .join(";");
    let forbid = if report.forbidai.is_empty() {
        "-".to_string()
    } else {
        report.forbidai.join(",")
    };
    format!(
        "packs:{} | {characters} | forbid:{forbid} | sunben:{} | skills:{skills} | audio:{} | loop:{}",
        report.packs.join(","),
        report.re_sunben.as_deref().unwrap_or("-"),
        u8::from(report.audio_played),
        u8::from(report.audio_looped),
    )
}

fn parse_request(text: &str) -> Result<ShoushaRequest, ShoushaError> {
    let value: Value =
        serde_json::from_str(text).map_err(|err| ShoushaError::BadConfig(err.to_string()))?;
    let obj = value
        .as_object()
        .ok_or_else(|| ShoushaError::BadConfig("request must be an object".into()))?;
    let mode = required_text(obj, "mode")?;
    let ban_bt = required_bool(obj, "banBt")?;
    let characters = obj
        .get("characters")
        .and_then(Value::as_array)
        .ok_or_else(|| ShoushaError::BadConfig("characters must be an array".into()))?
        .iter()
        .map(|value| {
            let item = value
                .as_object()
                .ok_or_else(|| ShoushaError::BadConfig("character must be an object".into()))?;
            Ok(CharacterIn {
                id: required_text(item, "id")?,
                gender: required_text(item, "gender")?,
                pack: required_text(item, "pack")?,
                name: match item.get("name") {
                    Some(Value::String(text)) => text.clone(),
                    Some(_) => return Err(ShoushaError::BadConfig("name must be a string".into())),
                    None => String::new(),
                },
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let packs = string_array(obj.get("packs"), "packs")?;
    let mut pack_members = BTreeMap::new();
    if let Some(value) = obj.get("packMembers") {
        let members = value
            .as_object()
            .ok_or_else(|| ShoushaError::BadConfig("packMembers must be an object".into()))?;
        for (pack, ids) in members {
            pack_members.insert(pack.clone(), string_array(Some(ids), "packMembers")?);
        }
    }
    let skills = match obj.get("skills") {
        None => Vec::new(),
        Some(value) => value
            .as_array()
            .ok_or_else(|| ShoushaError::BadConfig("skills must be an array".into()))?
            .iter()
            .map(|value| {
                let item = value
                    .as_object()
                    .ok_or_else(|| ShoushaError::BadConfig("skill must be an object".into()))?;
                Ok(SkillIn {
                    id: required_text(item, "id")?,
                    info: required_text(item, "info")?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?,
    };
    Ok(ShoushaRequest {
        mode,
        ban_bt,
        characters,
        packs,
        pack_members,
        skills,
    })
}

fn parse_tables(value: Option<&Value>) -> Result<Tables, ShoushaError> {
    let obj = value
        .and_then(Value::as_object)
        .ok_or_else(|| ShoushaError::BadConfig("tables must be an object".into()))?;
    let base = string_map(obj.get("base"), "base")?;
    let doudizhu = string_map(obj.get("doudizhu"), "doudizhu")?;
    let huanyuan = string_map(obj.get("huanyuan"), "huanyuan")?;
    let yinjian = string_map(obj.get("yinjian"), "yinjian")?;
    let jinjiang_packs = string_array(obj.get("jinjiangPacks"), "jinjiangPacks")?;
    let ban = obj
        .get("ban")
        .and_then(Value::as_object)
        .ok_or_else(|| ShoushaError::BadConfig("ban must be an object".into()))?;
    let bt = ban
        .get("bt")
        .and_then(Value::as_object)
        .ok_or_else(|| ShoushaError::BadConfig("ban.bt must be an object".into()))?;
    let tables = Tables {
        base,
        doudizhu,
        huanyuan,
        yinjian,
        jinjiang_packs,
        ban_packs: string_array(ban.get("packs"), "ban.packs")?,
        exclude: string_map_of_arrays(ban.get("exclude"), "ban.exclude")?,
        add: string_array(ban.get("add"), "ban.add")?,
        bt_packs: string_array(bt.get("packs"), "ban.bt.packs")?,
        bt_exclude_extra: string_array(bt.get("excludeExtra"), "ban.bt.excludeExtra")?,
        bt_add: string_array(bt.get("add"), "ban.bt.add")?,
    };
    if tables.base.len() != 511 || tables.base.get("liubei").map(String::as_str) != Some("刘备") {
        return Err(ShoushaError::BadConfig("base rename table drifted".into()));
    }
    if tables.doudizhu.len() != 11
        || tables.doudizhu.get("sunwukong").map(String::as_str) != Some("孙悟空")
    {
        return Err(ShoushaError::BadConfig("doudizhu table drifted".into()));
    }
    if tables.huanyuan.len() != 167 || tables.yinjian.len() != 373 {
        return Err(ShoushaError::BadConfig("pool table drifted".into()));
    }
    if tables.jinjiang_packs.first().map(String::as_str) != Some("standard")
        || !tables.jinjiang_packs.iter().any(|pack| pack == "huicui")
        || tables.jinjiang_packs.iter().any(|pack| pack == "tw")
    {
        return Err(ShoushaError::BadConfig("jinjiang packs drifted".into()));
    }
    if tables.ban_packs.first().map(String::as_str) != Some("mobile")
        || tables.ban_packs.len() != 11
    {
        return Err(ShoushaError::BadConfig("ban packs drifted".into()));
    }
    let extra = tables
        .exclude
        .get("extra")
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let refresh = tables
        .exclude
        .get("refresh")
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let sp = tables.exclude.get("sp").map(Vec::as_slice).unwrap_or(&[]);
    if extra.len() != 12
        || !extra.iter().any(|id| id == "shen_diaochan")
        || refresh.len() != 66
        || sp.len() != 46
        || tables.add.len() != 10
    {
        return Err(ShoushaError::BadConfig("ban exclude drifted".into()));
    }
    if tables.bt_packs != ["extra"]
        || tables.bt_exclude_extra.len() != 20
        || tables.bt_add.len() != 78
    {
        return Err(ShoushaError::BadConfig("ban bt drifted".into()));
    }
    Ok(tables)
}

fn allowed_ids(
    tables: &Tables,
    ban_bt: bool,
    members: &BTreeMap<String, Vec<String>>,
) -> Result<Vec<String>, ShoushaError> {
    let packs = if ban_bt {
        &tables.bt_packs
    } else {
        &tables.ban_packs
    };
    let add = if ban_bt { &tables.bt_add } else { &tables.add };
    let mut allowed = Vec::new();
    for pack in packs {
        let ids = members
            .get(pack)
            .ok_or_else(|| ShoushaError::MissingPack(pack.clone()))?;
        let excluded: &[String] = if ban_bt && pack == "extra" {
            &tables.bt_exclude_extra
        } else {
            tables.exclude.get(pack).map(Vec::as_slice).unwrap_or(&[])
        };
        for id in ids {
            if !excluded.iter().any(|item| item == id) {
                allowed.push(id.clone());
            }
        }
    }
    allowed.extend(add.iter().cloned());
    Ok(allowed)
}

fn apply_pool_tags(
    characters: &mut [CharacterOut],
    map: &BTreeMap<String, String>,
    change_group: bool,
) {
    for character in characters {
        if let Some(name) = map.get(&character.id) {
            add_tag(&mut character.tags, "forbidai");
            if change_group {
                character.group = Some(name.clone());
            }
        }
    }
}

fn rename_map(config: &ShoushaConfig) -> &BTreeMap<String, String> {
    if config.yinjian {
        &config.tables.yinjian
    } else if config.huanyuan {
        &config.tables.huanyuan
    } else {
        &config.tables.base
    }
}

fn pool_filter(packs: &[String]) -> Vec<String> {
    let present: BTreeSet<&str> = packs.iter().map(String::as_str).collect();
    let mut allow = Vec::new();
    for (requires, name) in POOL_RULES {
        if present.contains(requires) && !allow.contains(&(*name).to_string()) {
            allow.push((*name).to_string());
        }
    }
    keep_listed(packs, &allow)
}

fn keep_listed(packs: &[String], allow: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for pack in packs {
        if allow.iter().any(|item| item == pack) && !out.iter().any(|item| item == pack) {
            out.push(pack.clone());
        }
    }
    out
}

fn add_tag(tags: &mut Vec<String>, tag: &str) {
    if !tags.iter().any(|item| item == tag) {
        tags.push(tag.to_string());
    }
}

fn fix_info(id: &str, info: &str) -> String {
    if info.is_empty() || id.contains("_map") {
        return info.to_string();
    }
    let trimmed = info.trim();
    let Some(last) = trimmed.chars().next_back() else {
        return info.to_string();
    };
    let last = last.to_string();
    if INFO_ENDINGS.contains(&last.as_str()) {
        info.to_string()
    } else {
        format!("{info}。")
    }
}

fn token(value: Option<&Value>, field: &str) -> Result<String, ShoushaError> {
    match value {
        Some(Value::String(text))
            if !text.is_empty()
                && text
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_') =>
        {
            Ok(text.clone())
        }
        _ => Err(ShoushaError::BadConfig(format!("{field} must be a token"))),
    }
}

fn required_text(
    obj: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<String, ShoushaError> {
    match obj.get(field) {
        Some(Value::String(text)) => Ok(text.clone()),
        _ => Err(ShoushaError::BadConfig(format!("{field} must be a string"))),
    }
}

fn required_bool(obj: &serde_json::Map<String, Value>, field: &str) -> Result<bool, ShoushaError> {
    match obj.get(field) {
        Some(Value::Bool(value)) => Ok(*value),
        _ => Err(ShoushaError::BadConfig(format!("{field} must be a bool"))),
    }
}

fn string_array(value: Option<&Value>, field: &str) -> Result<Vec<String>, ShoushaError> {
    value
        .and_then(Value::as_array)
        .ok_or_else(|| ShoushaError::BadConfig(format!("{field} must be an array")))?
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_string)
                .ok_or_else(|| ShoushaError::BadConfig(format!("{field} must contain strings")))
        })
        .collect()
}

fn string_map(
    value: Option<&Value>,
    field: &str,
) -> Result<BTreeMap<String, String>, ShoushaError> {
    let obj = value
        .and_then(Value::as_object)
        .ok_or_else(|| ShoushaError::BadConfig(format!("{field} must be an object")))?;
    let mut out = BTreeMap::new();
    for (key, item) in obj {
        let text = item
            .as_str()
            .ok_or_else(|| ShoushaError::BadConfig(format!("{field} values must be strings")))?;
        out.insert(key.clone(), text.to_string());
    }
    Ok(out)
}

fn string_map_of_arrays(
    value: Option<&Value>,
    field: &str,
) -> Result<BTreeMap<String, Vec<String>>, ShoushaError> {
    let obj = value
        .and_then(Value::as_object)
        .ok_or_else(|| ShoushaError::BadConfig(format!("{field} must be an object")))?;
    let mut out = BTreeMap::new();
    for (key, item) in obj {
        out.insert(key.clone(), string_array(Some(item), field)?);
    }
    Ok(out)
}
