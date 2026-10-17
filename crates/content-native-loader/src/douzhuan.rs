//! L17 斗转星移的禁将方案、导入导出、描述改写和斗地主排序。
//!
//! 不打开菜单，不播放音频，不调用 `game.reload`，不 `eval` 方案或动态描述。
//! 菜单在 boss 开局前、太虚幻境和乱斗里提前返回；禁将标记仍按模式方案写入。
//! 稀有度和抽样由调用方给出。空方案导出是 `['']`，再读回来是一条空字符串。

use std::collections::BTreeMap;

use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DouzhuanError {
    BadConfig(String),
    BadPlan(String),
    BadPick(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DouzhuanConfig {
    pub source: String,
    pub upstream_path: String,
    pub upstream_commit: String,
    pub upstream_version: String,
    pub jin_jiang: bool,
    pub jin_jiang_type: String,
    pub jin_jiang_ru_zhen: String,
    pub jj_group_color: bool,
    pub qnss_background: bool,
    pub ai_qianghua: bool,
    pub qidongye: bool,
    pub miaoshu: bool,
    pub forbid_reload: bool,
    pub plan_names: Vec<String>,
    pub plans: BTreeMap<u8, Vec<String>>,
    pub mode_plans: BTreeMap<String, u8>,
    pub stars: BTreeMap<String, Vec<i64>>,
    pub plan_packs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterIn {
    pub id: String,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterOut {
    pub id: String,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillIn {
    pub id: String,
    pub info: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillOut {
    pub id: String,
    pub info: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateIn {
    pub id: String,
    pub rarity: Option<String>,
    pub stars: Option<Vec<i64>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PickIn {
    pub candidates: Vec<CandidateIn>,
    pub zhu: bool,
    pub double: bool,
    pub roll: f64,
    pub second: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PackView {
    pub id: String,
    pub count: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DouzhuanRequest {
    pub mode: String,
    pub brawl: bool,
    pub story: Option<String>,
    pub network: Option<String>,
    pub previous_tempmode: Option<String>,
    pub characters: Vec<CharacterIn>,
    pub skills: Vec<SkillIn>,
    pub dynamic: Vec<String>,
    pub plans: Option<BTreeMap<u8, Vec<String>>>,
    pub mode_plans: Option<BTreeMap<String, u8>>,
    pub import_text: Option<String>,
    pub import_index: Option<u8>,
    pub rename: Option<String>,
    pub rename_index: Option<u8>,
    pub edit_index: Option<u8>,
    pub ban_ids: Vec<String>,
    pub open_ids: Vec<String>,
    pub keep_ids: Vec<String>,
    pub pack_base: Option<Vec<String>>,
    pub pack_views: Vec<PackView>,
    pub pick: Option<PickIn>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DouzhuanReport {
    pub mode: String,
    pub forbid_type: String,
    pub gate: bool,
    pub plan_index: Option<u8>,
    pub banned: Vec<String>,
    pub lock: Vec<String>,
    pub now: Vec<String>,
    pub characters: Vec<CharacterOut>,
    pub plan_names: Vec<String>,
    pub plans: BTreeMap<u8, Vec<String>>,
    pub plan_packs: Vec<String>,
    pub sorted: Vec<String>,
    pub pick: Option<String>,
    pub pick_second: Option<String>,
    pub skills: Vec<SkillOut>,
    pub dynamic: Vec<String>,
    pub would_reload: bool,
    pub reload_called: bool,
    pub audio_played: bool,
    pub eval_called: bool,
    pub group_colored: bool,
    pub menu_opened: bool,
}

pub fn parse_douzhuan_config(text: &str) -> Result<DouzhuanConfig, DouzhuanError> {
    let value: Value =
        serde_json::from_str(text).map_err(|_| DouzhuanError::BadConfig("json".into()))?;
    let obj = value
        .as_object()
        .ok_or_else(|| DouzhuanError::BadConfig("json".into()))?;
    let source = required_string(obj, "source")?;
    if source.is_empty() {
        return Err(DouzhuanError::BadConfig("source".into()));
    }
    Ok(DouzhuanConfig {
        source,
        upstream_path: optional_string(obj, "upstream_path"),
        upstream_commit: optional_string(obj, "upstream_commit"),
        upstream_version: optional_string(obj, "upstream_version"),
        jin_jiang: optional_bool(obj, "jinJiang", true)?,
        jin_jiang_type: {
            let text = optional_string_or(obj, "jinJiangType", "forbidai");
            if text.is_empty() {
                "forbidai".into()
            } else {
                text
            }
        },
        jin_jiang_ru_zhen: optional_string_or(obj, "jinJiangRuZhen", "online"),
        jj_group_color: optional_bool(obj, "jjGroupColor", true)?,
        qnss_background: optional_bool(obj, "qnssBackground", true)?,
        ai_qianghua: optional_bool(obj, "aiQianghua", true)?,
        qidongye: optional_bool(obj, "qidongyeMeiHua", false)?,
        miaoshu: optional_bool(obj, "miaoshuXiuGai", false)?,
        forbid_reload: optional_bool(obj, "forbidReload", true)?,
        plan_names: match obj.get("planNames") {
            None => vec!["方案一".into(), "方案二".into(), "方案三".into()],
            Some(value) => string_array(value, "planNames")?,
        },
        plans: match obj.get("plans") {
            None => default_plans(),
            Some(value) => plan_map(value)?,
        },
        mode_plans: match obj.get("modePlans") {
            None => BTreeMap::new(),
            Some(value) => mode_map(value)?,
        },
        stars: match obj.get("stars") {
            None => BTreeMap::new(),
            Some(value) => star_map(value)?,
        },
        plan_packs: match obj.get("planPacks") {
            None => Vec::new(),
            Some(value) => string_array(value, "planPacks")?,
        },
    })
}

pub fn rank_score(saved: Option<&[i64]>, identity: &str, rarity: Option<&str>) -> Option<i64> {
    if let Some(saved) = saved {
        let index = match identity {
            "全能" => Some(0),
            "主公" => Some(1),
            "忠臣" => Some(2),
            "反贼" => Some(3),
            "内奸" => Some(4),
            "地主" => Some(5),
            "农民" => Some(6),
            _ => None,
        }?;
        return saved.get(index).copied();
    }
    match rarity {
        Some("junk") => Some(2),
        Some("common") => Some(4),
        Some("rare") => Some(6),
        Some("epic") => Some(8),
        Some("legend") => Some(10),
        _ => None,
    }
}

pub fn rewrite_skill_text(text: &str) -> String {
    let swapped = text
        .replace("不大于", "temp")
        .replace("不小于", "≥")
        .replace("temp", "≤");
    swapped
        .replace('阴', "temp")
        .replace('阳', "阴")
        .replace("temp", "阳")
}

/// 上游导出把数组用单引号拼起来。空数组拼出来是 `['']`，不是 `[]`。
pub fn export_plan(ids: &[String]) -> String {
    let body = ids.join("','");
    format!("['{body}']")
}

/// 只接受引号字符串数组。含调用、语句或非字符串元素时拒绝，不执行文本。
pub fn parse_plan_text(text: &str) -> Result<Vec<String>, DouzhuanError> {
    let text = text.trim();
    if text.contains(';') || text.contains('(') || text.contains('{') || text.contains('`') {
        return Err(DouzhuanError::BadPlan("eval".into()));
    }
    if !text.starts_with('[') || !text.ends_with(']') {
        return Err(DouzhuanError::BadPlan("brackets".into()));
    }
    let inner = text[1..text.len() - 1].trim();
    if inner.is_empty() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    let mut rest = inner;
    while !rest.is_empty() {
        rest = rest.trim_start();
        let quote = rest
            .chars()
            .next()
            .ok_or_else(|| DouzhuanError::BadPlan("item".into()))?;
        if quote != '\'' && quote != '"' {
            return Err(DouzhuanError::BadPlan("item".into()));
        }
        let body = &rest[quote.len_utf8()..];
        let end = body
            .find(quote)
            .ok_or_else(|| DouzhuanError::BadPlan("item".into()))?;
        let id = &body[..end];
        if id.contains('\\') {
            return Err(DouzhuanError::BadPlan("item".into()));
        }
        out.push(id.to_string());
        rest = body[end + quote.len_utf8()..].trim_start();
        if rest.is_empty() {
            break;
        }
        if !rest.starts_with(',') {
            return Err(DouzhuanError::BadPlan("item".into()));
        }
        rest = &rest[1..];
        if rest.trim().is_empty() {
            return Err(DouzhuanError::BadPlan("item".into()));
        }
    }
    Ok(out)
}

pub fn apply_douzhuan(
    config: &DouzhuanConfig,
    request: &DouzhuanRequest,
) -> Result<DouzhuanReport, DouzhuanError> {
    let mut plans = request
        .plans
        .clone()
        .unwrap_or_else(|| config.plans.clone());
    let mode_plans = request
        .mode_plans
        .clone()
        .unwrap_or_else(|| config.mode_plans.clone());
    let mut names = config.plan_names.clone();
    if let Some(name) = request.rename.as_deref() {
        if !name.is_empty() {
            let index = request
                .rename_index
                .ok_or_else(|| DouzhuanError::BadPlan("rename".into()))?;
            if index == 0 || index as usize > names.len() {
                return Err(DouzhuanError::BadPlan("rename".into()));
            }
            names[index as usize - 1] = name.chars().take(4).collect();
        }
    }
    if let Some(text) = request.import_text.as_deref() {
        let index = request
            .import_index
            .ok_or_else(|| DouzhuanError::BadPlan("import".into()))?;
        plans.insert(index, parse_plan_text(text)?);
    }
    if !request.ban_ids.is_empty() || !request.open_ids.is_empty() {
        let index = request
            .edit_index
            .ok_or_else(|| DouzhuanError::BadPlan("edit".into()))?;
        let plan = plans.entry(index).or_default();
        for id in &request.ban_ids {
            add_unique(plan, id);
        }
        for id in &request.open_ids {
            if request.keep_ids.iter().any(|kept| kept == id) {
                continue;
            }
            plan.retain(|item| item != id);
        }
    }
    let mut packs = request
        .pack_base
        .clone()
        .unwrap_or_else(|| config.plan_packs.clone());
    for view in &request.pack_views {
        if view.count == 0 {
            packs.retain(|id| id != &view.id);
        } else {
            add_unique(&mut packs, &view.id);
        }
    }

    let mut mode = request.mode.clone();
    if request.brawl {
        mode = "brawl".into();
    }
    if let Some(story) = request.story.as_deref() {
        if !story.is_empty() {
            mode = story.to_string();
        }
    }
    let forbid_type = if config.jin_jiang_type == "forbidai" || config.jin_jiang_type == "unseen" {
        config.jin_jiang_type.clone()
    } else {
        "unseen".into()
    };
    let network = ban_network(request.network.as_deref());
    let plan_index = mode_plans.get(&mode).copied();
    let stored = plan_index.and_then(|index| plans.get(&index));
    let gate =
        config.jin_jiang && config.jin_jiang_ru_zhen.find(network).is_some() && stored.is_some();
    let banned = if gate {
        stored.cloned().unwrap_or_default()
    } else {
        Vec::new()
    };

    let mut characters = Vec::new();
    let mut lock = Vec::new();
    let mut now = Vec::new();
    for character in &request.characters {
        let Some(tags) = character.tags.as_ref() else {
            characters.push(CharacterOut {
                id: character.id.clone(),
                tags: None,
            });
            continue;
        };
        if tags
            .iter()
            .any(|tag| tag == "unseen" || tag == &forbid_type)
        {
            add_unique(&mut now, &character.id);
            add_unique(&mut lock, &character.id);
        }
        characters.push(CharacterOut {
            id: character.id.clone(),
            tags: Some(tags.clone()),
        });
    }
    if gate {
        for id in &banned {
            if let Some(character) = characters.iter_mut().find(|item| item.id == *id) {
                if let Some(tags) = character.tags.as_mut() {
                    add_unique(tags, &forbid_type);
                    if forbid_type == "unseen" {
                        add_unique(tags, "imforbid");
                    }
                }
            }
        }
    }

    let rewrite = |text: &str| {
        if config.miaoshu {
            rewrite_skill_text(text)
        } else {
            text.to_string()
        }
    };
    let skills = request
        .skills
        .iter()
        .map(|skill| SkillOut {
            id: skill.id.clone(),
            info: rewrite(&skill.info),
        })
        .collect();
    let dynamic = request.dynamic.iter().map(|text| rewrite(text)).collect();

    let previous = request
        .previous_tempmode
        .as_deref()
        .filter(|text| !text.is_empty());
    let would_reload = !config.forbid_reload && previous.is_some_and(|text| text != request.mode);

    let (sorted, pick, pick_second) = select_pick(config, request)?;

    Ok(DouzhuanReport {
        mode,
        forbid_type,
        gate,
        plan_index: if gate { plan_index } else { None },
        banned,
        lock,
        now,
        characters,
        plan_names: names,
        plans,
        plan_packs: packs,
        sorted,
        pick,
        pick_second,
        skills,
        dynamic,
        would_reload,
        reload_called: false,
        audio_played: false,
        eval_called: false,
        group_colored: false,
        menu_opened: false,
    })
}

pub fn apply_douzhuan_line(config_json: &str, request_json: &str) -> Result<String, DouzhuanError> {
    let config = parse_douzhuan_config(config_json)?;
    let request = request_from_json(request_json)?;
    let report = apply_douzhuan(&config, &request)?;
    Ok(format_report(&report))
}

pub fn request_from_json(text: &str) -> Result<DouzhuanRequest, DouzhuanError> {
    let value: Value =
        serde_json::from_str(text).map_err(|_| DouzhuanError::BadConfig("json".into()))?;
    let obj = value
        .as_object()
        .ok_or_else(|| DouzhuanError::BadConfig("json".into()))?;
    let mode = required_string(obj, "mode")?;
    if mode.is_empty() {
        return Err(DouzhuanError::BadConfig("mode".into()));
    }
    let characters = match obj.get("characters") {
        None => return Err(DouzhuanError::BadConfig("characters".into())),
        Some(Value::Array(items)) => items
            .iter()
            .map(character_from_json)
            .collect::<Result<Vec<_>, _>>()?,
        Some(_) => return Err(DouzhuanError::BadConfig("characters".into())),
    };
    let skills = match obj.get("skills") {
        None => Vec::new(),
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| {
                let row = item
                    .as_object()
                    .ok_or_else(|| DouzhuanError::BadConfig("skills".into()))?;
                Ok(SkillIn {
                    id: required_string(row, "id")?,
                    info: optional_string(row, "info"),
                })
            })
            .collect::<Result<Vec<_>, _>>()?,
        Some(_) => return Err(DouzhuanError::BadConfig("skills".into())),
    };
    let dynamic = match obj.get("dynamic") {
        None => Vec::new(),
        Some(value) => string_array(value, "dynamic")?,
    };
    let pick = match obj.get("pick") {
        None | Some(Value::Null) => None,
        Some(value) => Some(pick_from_json(value)?),
    };
    Ok(DouzhuanRequest {
        mode,
        brawl: optional_bool(obj, "brawl", false)?,
        story: optional_present(obj, "story"),
        network: optional_present(obj, "network"),
        previous_tempmode: optional_present(obj, "previousTempmode"),
        characters,
        skills,
        dynamic,
        plans: match obj.get("plans") {
            None => None,
            Some(value) => Some(plan_map(value)?),
        },
        mode_plans: match obj.get("modePlans") {
            None => None,
            Some(value) => Some(mode_map(value)?),
        },
        import_text: optional_present(obj, "importText"),
        import_index: optional_index(obj, "importIndex")?,
        rename: optional_present(obj, "rename"),
        rename_index: optional_index(obj, "renameIndex")?,
        edit_index: optional_index(obj, "editIndex")?,
        ban_ids: match obj.get("banIds") {
            None => Vec::new(),
            Some(value) => string_array(value, "banIds")?,
        },
        open_ids: match obj.get("openIds") {
            None => Vec::new(),
            Some(value) => string_array(value, "openIds")?,
        },
        keep_ids: match obj.get("keepIds") {
            None => Vec::new(),
            Some(value) => string_array(value, "keepIds")?,
        },
        pack_base: match obj.get("packBase") {
            None => None,
            Some(value) => Some(string_array(value, "packBase")?),
        },
        pack_views: match obj.get("packViews") {
            None => Vec::new(),
            Some(Value::Array(items)) => items
                .iter()
                .map(pack_view_from_json)
                .collect::<Result<Vec<_>, _>>()?,
            Some(_) => return Err(DouzhuanError::BadConfig("packViews".into())),
        },
        pick,
    })
}

pub fn format_report(report: &DouzhuanReport) -> String {
    let plan = match report.plan_index {
        Some(index) => index.to_string(),
        None => "-".into(),
    };
    let tags = if report
        .characters
        .iter()
        .all(|character| character.tags.is_none())
    {
        "-".into()
    } else {
        report
            .characters
            .iter()
            .filter_map(|character| {
                let tags = character.tags.as_ref()?;
                let body = if tags.is_empty() {
                    "-".into()
                } else {
                    tags.join("+")
                };
                Some(format!("{}={body}", character.id))
            })
            .collect::<Vec<_>>()
            .join(",")
    };
    let names = if report.plan_names.is_empty() {
        "-".into()
    } else {
        report
            .plan_names
            .iter()
            .enumerate()
            .map(|(index, name)| format!("{}={name}", index + 1))
            .collect::<Vec<_>>()
            .join(",")
    };
    let plans = {
        let parts: Vec<_> = report
            .plans
            .iter()
            .filter(|(_, ids)| !ids.is_empty())
            .map(|(index, ids)| format!("{index}={}", ids.join("+")))
            .collect();
        if parts.is_empty() {
            "-".into()
        } else {
            parts.join(",")
        }
    };
    let packs = csv(&report.plan_packs);
    let sort = csv(&report.sorted);
    let pick = match (&report.pick, &report.pick_second) {
        (None, _) => "-".into(),
        (Some(first), None) => first.clone(),
        (Some(first), Some(second)) => format!("{first}/{second}"),
    };
    let info = if report.skills.is_empty() {
        "-".into()
    } else {
        report
            .skills
            .iter()
            .map(|skill| format!("{}={}", skill.id, skill.info))
            .collect::<Vec<_>>()
            .join(";")
    };
    let dynamic = if report.dynamic.is_empty() {
        "-".into()
    } else {
        report.dynamic.join(";")
    };
    format!(
        "mode:{} | type:{} | gate:{} | plan:{plan} | banned:{} | lock:{} | now:{} | tags:{tags} | names:{names} | plans:{plans} | packs:{packs} | sort:{sort} | pick:{pick} | info:{info} | dyn:{dynamic} | would:{} | reload:0 | audio:0 | eval:0",
        report.mode,
        report.forbid_type,
        u8::from(report.gate),
        csv(&report.banned),
        csv(&report.lock),
        csv(&report.now),
        u8::from(report.would_reload),
    )
}

fn select_pick(
    config: &DouzhuanConfig,
    request: &DouzhuanRequest,
) -> Result<(Vec<String>, Option<String>, Option<String>), DouzhuanError> {
    let Some(pick) = &request.pick else {
        return Ok((Vec::new(), None, None));
    };
    let network = ai_network(request.network.as_deref());
    if !config.ai_qianghua || network != "online" || request.mode != "doudizhu" {
        return Ok((Vec::new(), None, None));
    }
    if pick.candidates.is_empty() {
        return Ok((Vec::new(), None, None));
    }
    let identity = if pick.zhu { "地主" } else { "农民" };
    let scores: Vec<Option<i64>> = pick
        .candidates
        .iter()
        .map(|candidate| {
            let saved = candidate
                .stars
                .as_deref()
                .or_else(|| config.stars.get(&candidate.id).map(Vec::as_slice));
            rank_score(saved, identity, candidate.rarity.as_deref())
        })
        .collect();
    let mut order: Vec<usize> = (0..pick.candidates.len()).collect();
    order.sort_by(|&left, &right| match (scores[left], scores[right]) {
        (Some(left_score), Some(right_score)) => {
            right_score.cmp(&left_score).then(left.cmp(&right))
        }
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => left.cmp(&right),
    });
    let sorted: Vec<String> = order
        .iter()
        .map(|&index| pick.candidates[index].id.clone())
        .collect();
    let first = sorted[0].clone();
    let first_score = scores[order[0]];
    let second_score = order.get(1).map(|&index| scores[index]).unwrap_or(None);
    if pick.double {
        if sorted.len() == 1 {
            return Ok((sorted, Some(first), None));
        }
        let second = pick
            .second
            .ok_or_else(|| DouzhuanError::BadPick("second".into()))?;
        let rest = sorted.len() - 1;
        if second >= rest {
            return Err(DouzhuanError::BadPick("second".into()));
        }
        let chosen = sorted[second + 1].clone();
        return Ok((sorted, Some(first), Some(chosen)));
    }
    let gap = if pick.zhu { 1 } else { 2 };
    let threshold = if pick.zhu { 0.8 } else { 0.7 };
    let take_second = pick.roll >= threshold
        && sorted.len() > 1
        && match (first_score, second_score) {
            (Some(high), Some(low))
                if low != 0 && i128::from(high) - i128::from(low) <= i128::from(gap) =>
            {
                true
            }
            _ => false,
        };
    if take_second {
        let chosen = sorted[1].clone();
        Ok((sorted, Some(chosen), None))
    } else {
        Ok((sorted, Some(first), None))
    }
}

fn default_plans() -> BTreeMap<u8, Vec<String>> {
    let mut plans = BTreeMap::new();
    for index in 0..4 {
        plans.insert(index, Vec::new());
    }
    plans
}

fn ban_network(network: Option<&str>) -> &str {
    match network {
        Some(text) if !text.is_empty() => text,
        _ => "offline",
    }
}

fn ai_network(network: Option<&str>) -> &str {
    match network {
        Some(text) if !text.is_empty() => text,
        _ => "online",
    }
}

fn add_unique(list: &mut Vec<String>, id: &str) {
    if !list.iter().any(|item| item == id) {
        list.push(id.to_string());
    }
}

fn csv(ids: &[String]) -> String {
    if ids.is_empty() {
        "-".into()
    } else {
        ids.join(",")
    }
}

fn required_string(
    obj: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<String, DouzhuanError> {
    match obj.get(key) {
        Some(Value::String(text)) => Ok(text.clone()),
        _ => Err(DouzhuanError::BadConfig(key.into())),
    }
}

fn optional_string(obj: &serde_json::Map<String, Value>, key: &str) -> String {
    obj.get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

fn optional_string_or(obj: &serde_json::Map<String, Value>, key: &str, default: &str) -> String {
    match obj.get(key) {
        Some(Value::String(text)) => text.clone(),
        _ => default.to_string(),
    }
}

fn optional_present(obj: &serde_json::Map<String, Value>, key: &str) -> Option<String> {
    match obj.get(key) {
        Some(Value::String(text)) if !text.is_empty() => Some(text.clone()),
        _ => None,
    }
}

fn optional_bool(
    obj: &serde_json::Map<String, Value>,
    key: &str,
    default: bool,
) -> Result<bool, DouzhuanError> {
    match obj.get(key) {
        None => Ok(default),
        Some(Value::Bool(value)) => Ok(*value),
        Some(_) => Err(DouzhuanError::BadConfig(key.into())),
    }
}

fn optional_index(
    obj: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<Option<u8>, DouzhuanError> {
    match obj.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(number)) => {
            let value = number
                .as_u64()
                .ok_or_else(|| DouzhuanError::BadPlan(key.into()))?;
            u8::try_from(value)
                .map(Some)
                .map_err(|_| DouzhuanError::BadPlan(key.into()))
        }
        Some(_) => Err(DouzhuanError::BadPlan(key.into())),
    }
}

fn string_array(value: &Value, key: &str) -> Result<Vec<String>, DouzhuanError> {
    let Some(items) = value.as_array() else {
        return Err(DouzhuanError::BadConfig(key.into()));
    };
    items
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_string)
                .ok_or_else(|| DouzhuanError::BadConfig(key.into()))
        })
        .collect()
}

fn plan_map(value: &Value) -> Result<BTreeMap<u8, Vec<String>>, DouzhuanError> {
    let Some(obj) = value.as_object() else {
        return Err(DouzhuanError::BadConfig("plans".into()));
    };
    let mut plans = BTreeMap::new();
    for (key, item) in obj {
        let index: u8 = key
            .parse()
            .map_err(|_| DouzhuanError::BadConfig("plans".into()))?;
        plans.insert(index, string_array(item, "plans")?);
    }
    Ok(plans)
}

fn mode_map(value: &Value) -> Result<BTreeMap<String, u8>, DouzhuanError> {
    let Some(obj) = value.as_object() else {
        return Err(DouzhuanError::BadConfig("modePlans".into()));
    };
    let mut modes = BTreeMap::new();
    for (key, item) in obj {
        let number = item
            .as_u64()
            .ok_or_else(|| DouzhuanError::BadConfig("modePlans".into()))?;
        let index =
            u8::try_from(number).map_err(|_| DouzhuanError::BadConfig("modePlans".into()))?;
        modes.insert(key.clone(), index);
    }
    Ok(modes)
}

fn star_map(value: &Value) -> Result<BTreeMap<String, Vec<i64>>, DouzhuanError> {
    let Some(obj) = value.as_object() else {
        return Err(DouzhuanError::BadConfig("stars".into()));
    };
    let mut stars = BTreeMap::new();
    for (key, item) in obj {
        let Some(items) = item.as_array() else {
            return Err(DouzhuanError::BadConfig("stars".into()));
        };
        let mut row = Vec::new();
        for number in items {
            let value = number
                .as_i64()
                .ok_or_else(|| DouzhuanError::BadConfig("stars".into()))?;
            row.push(value);
        }
        stars.insert(key.clone(), row);
    }
    Ok(stars)
}

fn character_from_json(value: &Value) -> Result<CharacterIn, DouzhuanError> {
    let obj = value
        .as_object()
        .ok_or_else(|| DouzhuanError::BadConfig("characters".into()))?;
    let id = required_string(obj, "id")?;
    let tags = match obj.get("tags") {
        None | Some(Value::Null) => None,
        Some(value) => Some(string_array(value, "tags")?),
    };
    Ok(CharacterIn { id, tags })
}

fn pack_view_from_json(value: &Value) -> Result<PackView, DouzhuanError> {
    let obj = value
        .as_object()
        .ok_or_else(|| DouzhuanError::BadConfig("packViews".into()))?;
    let id = required_string(obj, "id")?;
    let count = obj
        .get("count")
        .and_then(Value::as_i64)
        .ok_or_else(|| DouzhuanError::BadConfig("packViews".into()))?;
    Ok(PackView { id, count })
}

fn pick_from_json(value: &Value) -> Result<PickIn, DouzhuanError> {
    let obj = value
        .as_object()
        .ok_or_else(|| DouzhuanError::BadConfig("pick".into()))?;
    let Some(items) = obj.get("candidates").and_then(Value::as_array) else {
        return Err(DouzhuanError::BadConfig("pick".into()));
    };
    let mut candidates = Vec::new();
    for item in items {
        let row = item
            .as_object()
            .ok_or_else(|| DouzhuanError::BadConfig("pick".into()))?;
        let stars = match row.get("stars") {
            None => None,
            Some(value) => {
                let Some(numbers) = value.as_array() else {
                    return Err(DouzhuanError::BadConfig("stars".into()));
                };
                let mut saved = Vec::new();
                for number in numbers {
                    saved.push(
                        number
                            .as_i64()
                            .ok_or_else(|| DouzhuanError::BadConfig("stars".into()))?,
                    );
                }
                Some(saved)
            }
        };
        candidates.push(CandidateIn {
            id: required_string(row, "id")?,
            rarity: row
                .get("rarity")
                .and_then(Value::as_str)
                .map(str::to_string),
            stars,
        });
    }
    let roll = obj
        .get("roll")
        .and_then(Value::as_f64)
        .ok_or_else(|| DouzhuanError::BadConfig("pick".into()))?;
    let second = match obj.get("second") {
        None | Some(Value::Null) => None,
        Some(Value::Number(number)) => Some(
            usize::try_from(
                number
                    .as_u64()
                    .ok_or_else(|| DouzhuanError::BadPick("second".into()))?,
            )
            .map_err(|_| DouzhuanError::BadPick("second".into()))?,
        ),
        Some(_) => return Err(DouzhuanError::BadPick("second".into())),
    };
    Ok(PickIn {
        candidates,
        zhu: optional_bool(obj, "zhu", false)?,
        double: optional_bool(obj, "double", false)?,
        roll,
        second,
    })
}
