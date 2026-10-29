//! L26 笨蛋插件里仍会执行的决策。
//!
//! `Fool_chooseCard` 的配置已注释，运行入口是 `if(false)`，不执行。
//! 选将语音里按 `_choose.mp3` 找文件的那段也在注释中。
//! 本模块不画界面，不播放音频，不启动定时器，不采样 `Math.random`，
//! 不生成随机锚点，不请求网络，不写盘，不 `eval`，不睡眠。

use serde_json::Value;
mod audio;
pub use audio::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FoolError {
    BadConfig(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoolConfig {
    pub source: String,
    pub upstream_path: String,
    pub upstream_commit: String,
    pub upstream_version: String,
    pub throw_emotion: bool,
    pub skill_calc: bool,
    pub choose_audio: bool,
    pub peiyin_fix: bool,
    pub liegong: String,
    pub derivation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FangCard {
    pub zone: String,
    pub number: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HintQuery {
    pub viewer_is_me: bool,
    pub event_name: String,
    pub skills: Vec<String>,
    pub selected_count: i64,
    pub player_hand: i64,
    pub player_hp: i64,
    pub attack_range: i64,
    pub target_is_self: bool,
    pub target_hand: i64,
    pub target_hp: i64,
    pub in_range: bool,
    pub original_filter: bool,
    pub distance_target_to_player: i64,
    pub distance_player_to_target: i64,
    pub color: String,
    pub suit: String,
    pub spshidi_mark: i64,
    pub phase_has_sha: bool,
    pub phase_sha_count: i64,
    pub use_count: i64,
    pub damaged_hp: i64,
    pub empty_slots_2_to_6: bool,
    pub mark_basic: bool,
    pub mark_damage: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AudioSpec {
    Missing,
    Number(i64),
    Text(String),
    Bool(bool),
    Pair(String, i64),
    List(Vec<AudioSpec>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SkillAudio {
    pub id: String,
    pub translate: Option<String>,
    pub info: Option<String>,
    pub audio: AudioSpec,
    pub group: Option<Vec<String>>,
    pub no_choosing_audio: bool,
    pub audioname: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivSkill {
    pub id: String,
    pub translate: Option<String>,
    pub info: Option<String>,
    pub content: Option<String>,
    pub inherit: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivCharacter {
    pub skills: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LiegongSave {
    pub reload_called: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThrowOut {
    pub thrown: bool,
    pub move_time: Option<&'static str>,
    pub ease: &'static str,
    pub rotate: bool,
    pub drop: bool,
    pub spin: bool,
    pub snow: bool,
    pub shoe_shadow: bool,
    pub random_landing: bool,
    pub audio_name: Option<String>,
    pub video: bool,
    pub delay_ms: u32,
    pub sleep_called: bool,
    pub dom: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoiceClip {
    File(String),
    Skill(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceOut {
    pub skill: Option<String>,
    pub clip: Option<VoiceClip>,
    pub play_called: bool,
    pub interval: bool,
    pub fade_called: bool,
    pub custom_choose_file: bool,
}

pub fn parse_fool_config(text: &str) -> Result<FoolConfig, FoolError> {
    let value: Value =
        serde_json::from_str(text).map_err(|_| FoolError::BadConfig("json".into()))?;
    let obj = value
        .as_object()
        .ok_or_else(|| FoolError::BadConfig("json".into()))?;
    let source = required_str(obj, "source")?;
    if source.is_empty() {
        return Err(FoolError::BadConfig("source".into()));
    }
    Ok(FoolConfig {
        source,
        upstream_path: required_str(obj, "upstream_path")?,
        upstream_commit: required_str(obj, "upstream_commit")?,
        upstream_version: required_str(obj, "upstream_version")?,
        throw_emotion: required_bool(obj, "Fool_throwEmotion")?,
        skill_calc: required_bool(obj, "Fool_skill")?,
        choose_audio: required_bool(obj, "Fool_chooseAudio")?,
        peiyin_fix: required_bool(obj, "Fool_peiyinFix")?,
        liegong: required_str(obj, "Fool_liegong")?,
        derivation: required_bool(obj, "Fool_derivationSkill")?,
    })
}

pub fn ziyuan_allows(selected: &[i64], number: i64) -> bool {
    sum_allows(selected, number, 13)
}

pub fn minsi_allows(selected: &[i64], number: i64) -> bool {
    sum_allows(selected, number, 13)
}

pub fn fangtong_allows(selected: &[FangCard], card: &FangCard) -> bool {
    if !selected.is_empty() {
        if card.zone != "x" {
            return false;
        }
    } else if card.zone == "x" {
        return false;
    }
    let sum: i64 = selected.iter().map(|item| item.number).sum();
    card.number + sum <= 36
}

pub fn fangtong_select_len(selected: &[FangCard]) -> usize {
    let sum: i64 = selected.iter().map(|item| item.number).sum();
    if sum == 36 {
        selected.len()
    } else {
        selected.len() + 2
    }
}

pub fn sha_filter_wrapped(style: &str) -> bool {
    style != "close"
}

pub fn sha_target_allowed(query: &HintQuery) -> bool {
    query.original_filter
}

pub fn sha_hint(style: &str, query: &HintQuery) -> Option<String> {
    if !sha_filter_wrapped(style) || !query.viewer_is_me || query.event_name != "chooseToUse" {
        return None;
    }
    if query.target_is_self {
        return None;
    }
    let has = |name: &str| query.skills.iter().any(|skill| skill == name);
    let mut liegong = false;
    let mut qiangming = false;
    let mut shanghai = false;
    if has("DIY_liegong") || has("liegong") || has("xinliegong") {
        let num = query.selected_count;
        let cond = (has("DIY_liegong") && query.target_hand <= query.player_hand - num)
            || (has("liegong")
                && (query.target_hand >= query.player_hp
                    || query.target_hand <= query.attack_range))
            || (has("xinliegong")
                && (query.target_hand <= query.player_hand - num
                    || query.target_hp >= query.player_hp));
        if cond && query.in_range {
            liegong = true;
            if has("xinliegong") {
                if query.target_hand <= query.player_hand - num {
                    qiangming = true;
                }
                if query.target_hp >= query.player_hp {
                    shanghai = true;
                }
            }
        }
    }
    if has("spshidi")
        && query.color == "black"
        && query.spshidi_mark % 2 == 0
        && query.in_range
        && query.original_filter
    {
        qiangming = true;
    }
    if has("fuqi")
        && query.in_range
        && query.original_filter
        && query.distance_target_to_player <= 1
    {
        qiangming = true;
    }
    if has("refuqi")
        && query.in_range
        && query.original_filter
        && query.distance_player_to_target <= 1
    {
        qiangming = true;
    }
    if has("wushen") && query.suit == "heart" && query.in_range && query.original_filter {
        qiangming = true;
    }
    if has("sbpaoxiao") && query.phase_has_sha && query.in_range && query.original_filter {
        qiangming = true;
    }
    if has("liyong2") && query.in_range && query.original_filter {
        qiangming = true;
    }
    if has("zhaxiang2") && query.color == "red" && query.in_range && query.original_filter {
        qiangming = true;
    }
    if has("sbzhaxiang")
        && query.use_count < query.damaged_hp
        && query.in_range
        && query.original_filter
    {
        qiangming = true;
    }
    if has("relinglong") && query.empty_slots_2_to_6 && query.in_range && query.original_filter {
        qiangming = true;
    }
    if has("jintao") && (query.phase_sha_count == 0 || query.phase_sha_count == 1) {
        if query.phase_sha_count == 0 {
            shanghai = true;
        } else {
            qiangming = true;
        }
    }
    if has("dcgonghu")
        && query.mark_basic
        && query.mark_damage
        && query.color == "red"
        && query.in_range
        && query.original_filter
    {
        qiangming = true;
    }
    let mut text = String::new();
    if liegong {
        text.push_str("可烈弓");
        if qiangming {
            if style == "shousha" {
                text = "烈弓强中".into();
            } else {
                text.push_str(" 不可响应");
            }
            if shanghai {
                if style == "shousha" {
                    text = "强中加伤".into();
                } else {
                    text.push_str(" 伤害+1");
                }
            }
        } else if shanghai {
            if style == "shousha" {
                text = "烈弓加伤".into();
            } else {
                text.push_str(" 伤害+1");
            }
        }
    } else if qiangming {
        text = "不可响应".into();
        if shanghai {
            if style == "shousha" {
                text = "强中加伤".into();
            } else {
                text.push_str(" 伤害+1");
            }
        }
    } else if shanghai {
        if style == "shousha" {
            text = "加伤".into();
        } else {
            text.push_str(" 伤害+1");
        }
    }
    if style != "shousha" && style != "olten" && !text.is_empty() && liegong {
        text = "可烈弓".into();
    }
    Some(text)
}

pub fn is_phone(user_agent: &str) -> bool {
    let lower = user_agent.to_lowercase();
    [
        "mobile",
        "android",
        "webos",
        "iphone",
        "ipad",
        "ipod",
        "blackberry",
        "iemobile",
        "operamini",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

pub fn save_liegong(value: &str) -> (String, LiegongSave) {
    (
        value.to_string(),
        LiegongSave {
            reload_called: false,
        },
    )
}

pub fn rewrite_derivation(characters: &[DerivCharacter], skills: &mut [DerivSkill]) -> Vec<String> {
    let mut seen = Vec::new();
    for character in characters {
        for skill in &character.skills {
            if !seen.contains(skill) {
                seen.push(skill.clone());
            }
        }
    }
    let mut changed = Vec::new();
    for skill_id in seen {
        let Some(index) = skills.iter().position(|skill| skill.id == skill_id) else {
            continue;
        };
        let parent_name = skills[index].translate.clone();
        let mut content = skills[index]
            .content
            .clone()
            .filter(|text| !text.is_empty());
        if content.is_none() {
            if let Some(inherit) = skills[index].inherit.clone() {
                content = skills
                    .iter()
                    .find(|skill| skill.id == inherit)
                    .and_then(|skill| skill.content.clone())
                    .filter(|text| !text.is_empty());
            }
        }
        let Some(parent_name) = parent_name else {
            continue;
        };
        let Some(content) = content else {
            continue;
        };
        let Some(mut info) = skills[index].info.clone() else {
            continue;
        };
        let mut ids = Vec::new();
        let mut names: Vec<Option<String>> = Vec::new();
        for found in capture_calls(&content, "Skill")
            .into_iter()
            .chain(capture_calls(&content, "SkillLog"))
        {
            let name = skills
                .iter()
                .find(|skill| skill.id == found)
                .and_then(|skill| skill.translate.clone());
            if names.contains(&name) {
                continue;
            }
            names.push(name);
            ids.push(found);
        }
        let before = info.clone();
        for found in ids {
            let Some(found_skill) = skills.iter().find(|skill| skill.id == found) else {
                continue;
            };
            let Some(name) = found_skill.translate.clone() else {
                continue;
            };
            if found_skill
                .info
                .as_ref()
                .map(|text| text.is_empty())
                .unwrap_or(true)
            {
                continue;
            }
            if name.is_empty() || !info.contains(&name) || parent_name == name {
                continue;
            }
            let replacement = format!("※{name}");
            info = split_protect(&info)
                .into_iter()
                .map(|part| {
                    if protected_span(&part) {
                        part
                    } else {
                        part.replace(&name, &replacement)
                    }
                })
                .collect();
        }
        if info != before {
            skills[index].info = Some(info);
            changed.push(skill_id);
        }
    }
    changed
}

pub fn derivation_sample() -> String {
    let characters = vec![DerivCharacter {
        skills: vec!["parent".into(), "parent".into()],
    }];
    let mut skills = vec![
        DerivSkill {
            id: "parent".into(),
            translate: Some("父".into()),
            info: Some("获得资援<b>资援</b>结束".into()),
            content: Some("Skill('ziyuan');SkillLog(\"minsi\")".into()),
            inherit: None,
        },
        DerivSkill {
            id: "ziyuan".into(),
            translate: Some("资援".into()),
            info: Some("资援描述".into()),
            content: None,
            inherit: None,
        },
        DerivSkill {
            id: "minsi".into(),
            translate: Some("敏思".into()),
            info: Some("敏思描述".into()),
            content: None,
            inherit: None,
        },
    ];
    rewrite_derivation(&characters, &mut skills);
    skills[0].info.clone().unwrap_or_default()
}

pub fn choose_card_runs() -> bool {
    false
}

pub fn custom_choose_file_runs() -> bool {
    false
}

pub fn landing_offset(name: &str, random_x: f64, random_y: f64) -> Option<(f64, f64)> {
    if !matches!(name, "egg" | "flower" | "shoe" | "jiaozi" | "snow") {
        return None;
    }
    Some(((random_x - 0.5) * 100.0, (random_y - 0.3) * 120.0))
}

pub fn throw_emotion(
    enabled: bool,
    self_target: bool,
    throwing: bool,
    throwing_forbid: bool,
    name: &str,
    background_audio: bool,
    audio_index: i64,
) -> ThrowOut {
    let mut out = ThrowOut {
        thrown: false,
        move_time: None,
        ease: "ease",
        rotate: false,
        drop: false,
        spin: false,
        snow: false,
        shoe_shadow: false,
        random_landing: false,
        audio_name: None,
        video: false,
        delay_ms: 0,
        sleep_called: false,
        dom: 0,
    };
    if !enabled || self_target || (!throwing_forbid && throwing) {
        return out;
    }
    out.thrown = true;
    out.video = true;
    out.delay_ms = 20;
    out.move_time = Some(if throwing_forbid && name == "wine" {
        "0.9"
    } else {
        "0.7"
    });
    if throwing_forbid && name == "flower" {
        out.ease = "linear";
    }
    out.rotate = name == "flower";
    out.drop = matches!(name, "egg" | "shoe");
    out.spin = matches!(name, "egg" | "shoe" | "jiaozi" | "snow");
    out.snow = name == "snow";
    out.shoe_shadow = name == "shoe";
    out.random_landing = matches!(name, "egg" | "flower" | "shoe" | "jiaozi" | "snow");
    if background_audio {
        out.audio_name = Some(format!("throw_{name}{audio_index}"));
    }
    out
}

pub fn choose_voice(
    enabled: bool,
    selected: bool,
    background_speak: bool,
    character_known: bool,
    skills: &[SkillAudio],
    skill_ids: &[String],
    player_name: &str,
    pick: usize,
    file_pick: usize,
) -> VoiceOut {
    let mut out = VoiceOut {
        skill: None,
        clip: None,
        play_called: false,
        interval: false,
        fade_called: false,
        custom_choose_file: false,
    };
    if !enabled || !selected || !character_known {
        return out;
    }
    let ready: Vec<&SkillAudio> = skill_ids
        .iter()
        .filter_map(|id| skills.iter().find(|skill| skill.id == *id))
        .filter(|skill| {
            if !background_speak {
                return false;
            }
            !audio_list(skills, &skill.id, player_name, &mut Vec::new(), None).is_empty()
        })
        .collect();
    let Some(skill) = ready.get(pick) else {
        return out;
    };
    out.skill = Some(skill.id.clone());
    let list = audio_list(skills, &skill.id, player_name, &mut Vec::new(), None);
    out.clip = list.get(file_pick).cloned();
    out
}

pub fn apply_fool_line(config_text: &str, request_text: &str) -> Result<String, FoolError> {
    let config = parse_fool_config(config_text)?;
    let request: Value =
        serde_json::from_str(request_text).map_err(|_| FoolError::BadConfig("request".into()))?;
    let obj = request
        .as_object()
        .ok_or_else(|| FoolError::BadConfig("request".into()))?;
    let user_agent = optional_str(obj, "ua")?.unwrap_or("Mozilla/5.0 (iPhone; CPU iPhone OS)");
    let self_target = optional_bool(obj, "selfTarget")?.unwrap_or(false);
    let throwing = optional_bool(obj, "throwing")?.unwrap_or(false);
    let point = if config.skill_calc {
        let selected = [10];
        let hand = FangCard {
            zone: "h".into(),
            number: 20,
        };
        let fang = FangCard {
            zone: "x".into(),
            number: 16,
        };
        let over = FangCard {
            zone: "x".into(),
            number: 17,
        };
        format!(
            "zy:{} | zy_over:{} | ms:{} | ft_hand:{} | ft_fang:{} | ft_next:{} | ft_over:{} | ft_sel:{} | ft_need:{}",
            bit(ziyuan_allows(&selected, 3)),
            bit(ziyuan_allows(&selected, 4)),
            bit(minsi_allows(&selected, 3)),
            bit(fangtong_allows(&[], &hand)),
            bit(fangtong_allows(&[], &fang)),
            bit(fangtong_allows(&[hand.clone()], &fang)),
            bit(fangtong_allows(&[hand.clone()], &over)),
            fangtong_select_len(&[hand.clone(), fang.clone()]),
            fangtong_select_len(&[hand]),
        )
    } else {
        "zy:- | zy_over:- | ms:- | ft_hand:- | ft_fang:- | ft_next:- | ft_over:- | ft_sel:- | ft_need:-".into()
    };
    let query = default_hint();
    let hint = match sha_hint(&config.liegong, &query) {
        None => "-".into(),
        Some(text) if text.is_empty() => "empty".into(),
        Some(text) => text,
    };
    let thrown = throw_emotion(
        config.throw_emotion,
        self_target,
        throwing,
        false,
        "flower",
        false,
        1,
    );
    let voice = choose_voice(
        config.choose_audio,
        true,
        true,
        true,
        &[skill_audio(
            "rende",
            Some("仁德"),
            Some("描述"),
            AudioSpec::Number(1),
            None,
        )],
        &["rende".into()],
        "liubei",
        0,
        0,
    );
    let voice_skill = voice.skill.unwrap_or_else(|| "-".into());
    let voice_file = match voice.clip {
        Some(VoiceClip::File(path)) => path,
        Some(VoiceClip::Skill(skill)) => format!("skill:{skill}"),
        None => "-".into(),
    };
    let peiyin = if config.peiyin_fix {
        let ids = peiyin_sample();
        if ids.is_empty() {
            "-".into()
        } else {
            ids.join(",")
        }
    } else {
        "-".into()
    };
    let deriv = if config.derivation {
        let info = derivation_sample();
        if info.is_empty() {
            "-".into()
        } else {
            info
        }
    } else {
        "-".into()
    };
    let move_time = thrown.move_time.unwrap_or("-");
    Ok(format!(
        "{point} | hint:{hint} | wrap:{} | allowed:{} | phone:{} | peiyin:{peiyin} | deriv:{deriv} | throw:{} | move:{move_time} | rot:{} | land:{} | sampled:0 | video:{} | voice:{voice_skill} | file:{voice_file} | play:0 | cardopt:{} | custom:0 | reload:0 | eval:0 | fetch:0 | write:0 | sleep:0 | dom:0 | audio:{}",
        bit(sha_filter_wrapped(&config.liegong)),
        bit(sha_target_allowed(&query)),
        bit(is_phone(user_agent)),
        bit(thrown.thrown),
        bit(thrown.rotate),
        bit(thrown.random_landing),
        bit(thrown.video),
        bit(choose_card_runs()),
        bit(thrown.audio_name.is_some()),
    ))
}

fn default_hint() -> HintQuery {
    HintQuery {
        viewer_is_me: true,
        event_name: "chooseToUse".into(),
        skills: vec!["xinliegong".into()],
        selected_count: 1,
        player_hand: 3,
        player_hp: 3,
        attack_range: 1,
        target_is_self: false,
        target_hand: 1,
        target_hp: 4,
        in_range: true,
        original_filter: true,
        distance_target_to_player: 2,
        distance_player_to_target: 2,
        color: "red".into(),
        suit: "spade".into(),
        spshidi_mark: 1,
        phase_has_sha: false,
        phase_sha_count: 3,
        use_count: 0,
        damaged_hp: 0,
        empty_slots_2_to_6: false,
        mark_basic: false,
        mark_damage: false,
    }
}

fn sum_allows(selected: &[i64], number: i64, limit: i64) -> bool {
    selected.iter().sum::<i64>() + number <= limit
}

fn bit(value: bool) -> u8 {
    if value {
        1
    } else {
        0
    }
}

fn capture_calls(content: &str, fname: &str) -> Vec<String> {
    let mut out = Vec::new();
    let token_dq = format!("{fname}(\"");
    let token_sq = format!("{fname}('");
    let mut rest = content;
    while !rest.is_empty() {
        let double = rest.find(&token_dq);
        let single = rest.find(&token_sq);
        let (index, token_len) = match (double, single) {
            (Some(left), Some(right)) if left <= right => (left, token_dq.len()),
            (Some(_), Some(right)) => (right, token_sq.len()),
            (Some(left), None) => (left, token_dq.len()),
            (None, Some(right)) => (right, token_sq.len()),
            (None, None) => break,
        };
        let after = &rest[index + token_len..];
        let end = after.find(['\'', '"', '\n']);
        if let Some(end) = end {
            if after.as_bytes().get(end).copied() != Some(b'\n')
                && after[end + 1..].starts_with(')')
                && end > 0
            {
                out.push(after[..end].to_string());
                rest = &after[end + 2..];
                continue;
            }
        }
        rest = &rest[index + 1..];
    }
    out
}

fn split_protect(info: &str) -> Vec<String> {
    let chars: Vec<char> = info.chars().collect();
    let mut parts = Vec::new();
    let mut index = 0;
    let mut start = 0;
    while index < chars.len() {
        if chars[index] == '>' {
            let mut end = index + 1;
            while end < chars.len() && chars[end] != '<' && chars[end] != '>' {
                end += 1;
            }
            if end < chars.len() && chars[end] == '<' {
                if start < index {
                    parts.push(chars[start..index].iter().collect());
                }
                parts.push(chars[index..=end].iter().collect());
                index = end + 1;
                start = index;
                continue;
            }
        }
        index += 1;
    }
    if start < chars.len() {
        parts.push(chars[start..].iter().collect());
    } else if parts.is_empty() {
        parts.push(String::new());
    }
    parts
}

fn protected_span(part: &str) -> bool {
    let mut chars = part.chars();
    let first = chars.next();
    let last = part.chars().last();
    (first == Some('>') && last == Some('<')) || (first == Some('\'') && last == Some('：'))
}

fn truncate(mut clips: Vec<VoiceClip>, fixed: Option<i64>) -> Vec<VoiceClip> {
    if let Some(fixed) = fixed {
        if fixed > 0 && (fixed as usize) < clips.len() {
            clips.truncate(fixed as usize);
        }
    }
    clips
}

fn required_str(obj: &serde_json::Map<String, Value>, key: &str) -> Result<String, FoolError> {
    match obj.get(key) {
        Some(Value::String(text)) => Ok(text.clone()),
        _ => Err(FoolError::BadConfig(key.into())),
    }
}

fn required_bool(obj: &serde_json::Map<String, Value>, key: &str) -> Result<bool, FoolError> {
    match obj.get(key) {
        Some(Value::Bool(value)) => Ok(*value),
        _ => Err(FoolError::BadConfig(key.into())),
    }
}

fn optional_str<'a>(
    obj: &'a serde_json::Map<String, Value>,
    key: &str,
) -> Result<Option<&'a str>, FoolError> {
    match obj.get(key) {
        None => Ok(None),
        Some(Value::String(text)) => Ok(Some(text)),
        Some(_) => Err(FoolError::BadConfig(key.into())),
    }
}

fn optional_bool(
    obj: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<Option<bool>, FoolError> {
    match obj.get(key) {
        None => Ok(None),
        Some(Value::Bool(value)) => Ok(Some(*value)),
        Some(_) => Err(FoolError::BadConfig(key.into())),
    }
}
