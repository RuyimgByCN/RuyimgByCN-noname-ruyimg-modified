//! 愚人节包配音：音频规格解析、配音修正与语音选择。

use super::*;

pub(super) fn audio_truthy(audio: &AudioSpec) -> bool {
    match audio {
        AudioSpec::Missing => false,
        AudioSpec::Number(0) => false,
        AudioSpec::Number(_) => true,
        AudioSpec::Text(text) => !text.is_empty(),
        AudioSpec::Bool(value) => *value,
        AudioSpec::Pair(_, _) | AudioSpec::List(_) => true,
    }
}

pub(super) fn audio_is_number(audio: &AudioSpec) -> bool {
    matches!(audio, AudioSpec::Number(_))
}

pub fn fix_peiyin(skills: &mut [SkillAudio]) -> Vec<String> {
    let mut changed = Vec::new();
    let len = skills.len();
    for index in 0..len {
        if !audio_truthy(&skills[index].audio) {
            if let Some(translate) = skills[index].translate.clone() {
                for donor in 0..len {
                    let donor_id = {
                        let donor_skill = &skills[donor];
                        if donor_skill.translate.is_none() || donor_skill.info.is_none() {
                            None
                        } else if donor_skill.id.starts_with('_') {
                            None
                        } else if donor_skill.translate.as_deref() != Some(translate.as_str()) {
                            None
                        } else if !audio_truthy(&donor_skill.audio)
                            || !audio_is_number(&donor_skill.audio)
                        {
                            None
                        } else {
                            Some(donor_skill.id.clone())
                        }
                    };
                    let Some(donor_id) = donor_id else {
                        continue;
                    };
                    skills[index].audio = AudioSpec::Text(donor_id);
                    if !changed.contains(&skills[index].id) {
                        changed.push(skills[index].id.clone());
                    }
                }
            }
        }
        if !audio_truthy(&skills[index].audio) {
            continue;
        }
        let Some(group) = skills[index].group.clone() else {
            continue;
        };
        let parent = skills[index].id.clone();
        for child_id in group {
            if !child_id.contains(&parent) {
                continue;
            }
            let Some(child_index) = skills.iter().position(|skill| skill.id == child_id) else {
                continue;
            };
            if audio_truthy(&skills[child_index].audio) {
                continue;
            }
            skills[child_index].audio = AudioSpec::Text(parent.clone());
            if !changed.contains(&child_id) {
                changed.push(child_id);
            }
        }
    }
    changed
}

pub fn peiyin_sample() -> Vec<String> {
    let mut skills = vec![
        skill_audio("a", Some("杀"), None, AudioSpec::Missing, None),
        skill_audio("b", Some("杀"), Some("描述"), AudioSpec::Number(2), None),
        skill_audio("c", Some("杀"), Some("描述"), AudioSpec::Number(1), None),
        skill_audio("_pub", Some("杀"), Some("描述"), AudioSpec::Number(3), None),
        skill_audio(
            "d",
            Some("其他"),
            Some("描述"),
            AudioSpec::Number(1),
            Some(vec!["d2".into(), "nope".into()]),
        ),
        skill_audio("d2", Some("二"), None, AudioSpec::Missing, None),
        skill_audio("nope", Some("三"), None, AudioSpec::Missing, None),
        skill_audio(
            "zero",
            Some("零"),
            Some("描述"),
            AudioSpec::Number(0),
            Some(vec!["zero2".into()]),
        ),
        skill_audio("zero2", Some("零二"), None, AudioSpec::Missing, None),
    ];
    fix_peiyin(&mut skills)
}

pub(super) fn skill_audio(
    id: &str,
    translate: Option<&str>,
    info: Option<&str>,
    audio: AudioSpec,
    group: Option<Vec<String>>,
) -> SkillAudio {
    SkillAudio {
        id: id.into(),
        translate: translate.map(str::to_string),
        info: info.map(str::to_string),
        audio,
        group,
        no_choosing_audio: false,
        audioname: Vec::new(),
    }
}

pub(super) fn audio_list(
    skills: &[SkillAudio],
    skill_id: &str,
    player_name: &str,
    history: &mut Vec<String>,
    fixed: Option<i64>,
) -> Vec<VoiceClip> {
    let Some(skill) = skills.iter().find(|item| item.id == skill_id) else {
        return Vec::new();
    };
    if skill.no_choosing_audio {
        return Vec::new();
    }
    if history.iter().any(|item| item == skill_id) {
        return if !matches!(skill.audio, AudioSpec::Bool(false)) {
            vec![VoiceClip::Skill(skill_id.to_string())]
        } else {
            Vec::new()
        };
    }
    history.push(skill_id.to_string());
    parse_audio(
        skills,
        skill_id,
        &skill.audio,
        player_name,
        &skill.audioname,
        history,
        fixed,
    )
}

pub(super) fn parse_audio(
    skills: &[SkillAudio],
    skill_id: &str,
    audio: &AudioSpec,
    player_name: &str,
    audioname: &[String],
    history: &mut Vec<String>,
    fixed: Option<i64>,
) -> Vec<VoiceClip> {
    match audio {
        AudioSpec::List(items) => {
            let mut total = Vec::new();
            for item in items {
                total.extend(parse_audio(
                    skills,
                    skill_id,
                    item,
                    player_name,
                    audioname,
                    history,
                    fixed,
                ));
            }
            truncate(total, fixed)
        }
        AudioSpec::Pair(name, count) => {
            let next_fixed = match fixed {
                Some(value) if value != 0 => Some(value),
                _ => Some(*count),
            };
            if skills.iter().any(|skill| skill.id == *name) {
                audio_list(skills, name, player_name, history, next_fixed)
            } else {
                truncate(
                    parse_audio(
                        skills,
                        name,
                        &AudioSpec::Number(*count),
                        player_name,
                        audioname,
                        history,
                        next_fixed,
                    ),
                    next_fixed,
                )
            }
        }
        AudioSpec::Bool(false) => Vec::new(),
        AudioSpec::Missing => Vec::new(),
        other => {
            if let AudioSpec::Text(text) = other {
                if skills.iter().any(|skill| skill.id == *text) {
                    return audio_list(skills, text, player_name, history, fixed);
                }
            }
            let rendered = match other {
                AudioSpec::Number(number) => number.to_string(),
                AudioSpec::Text(text) => text.clone(),
                AudioSpec::Bool(true) => "true".into(),
                _ => return Vec::new(),
            };
            truncate(
                expand_audio_text(skill_id, &rendered, player_name, audioname),
                fixed,
            )
        }
    }
}

pub(super) fn expand_audio_text(
    skill_id: &str,
    audio_info: &str,
    player_name: &str,
    audioname: &[String],
) -> Vec<VoiceClip> {
    let suffix = if audioname.iter().any(|name| name == player_name) {
        format!("_{player_name}")
    } else {
        String::new()
    };
    if let Some((path, count, ext)) = match_audio_pattern(audio_info) {
        if count == "true" {
            let folder = if path.is_empty() { "skill" } else { path };
            let extension = if ext.is_empty() { "mp3" } else { ext };
            return vec![VoiceClip::File(format!(
                "{folder}/{skill_id}{suffix}.{extension}"
            ))];
        }
        if !count.is_empty() {
            let total: i64 = count.parse().unwrap_or(0);
            let folder = if path.is_empty() { "skill" } else { path };
            let extension = if ext.is_empty() { "mp3" } else { ext };
            return (1..=total)
                .map(|index| {
                    VoiceClip::File(format!("{folder}/{skill_id}{suffix}{index}.{extension}"))
                })
                .collect();
        }
    }
    if audio_info.starts_with("db:")
        || audio_info.starts_with("ext:")
        || audio_info.starts_with('/')
    {
        vec![VoiceClip::File(audio_info.to_string())]
    } else {
        vec![VoiceClip::File(format!("skill/{audio_info}"))]
    }
}

pub(super) fn match_audio_pattern(audio_info: &str) -> Option<(&str, &str, &str)> {
    let (path, rest) = if let Some((path, rest)) = audio_info.split_once(':') {
        (path, rest)
    } else {
        ("", audio_info)
    };
    if rest == "true" {
        return Some((path, "true", ""));
    }
    if let Some((count, ext)) = rest.split_once(':') {
        if count == "true" || count.chars().all(|ch| ch.is_ascii_digit()) {
            return Some((path, count, ext));
        }
        return None;
    }
    if rest == "true" || (!rest.is_empty() && rest.chars().all(|ch| ch.is_ascii_digit())) {
        return Some((path, rest, ""));
    }
    if rest.is_empty() && audio_info.ends_with(':') {
        return Some((path, "", ""));
    }
    None
}
