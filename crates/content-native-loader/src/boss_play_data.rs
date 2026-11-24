//! Upstream `boss-play` (`U02` noname 15 generals, `L03` liuli 8 Jiange generals)
//! authoritative general definitions and host skill registration helpers.

use std::collections::BTreeSet;

use engine_native::catalog::Faction;
use engine_native::content::extension::{ExtensionDef, ExtensionGeneral, ExtensionPack, InitHook};
use engine_native::content::host::{HostBehavior, HostSkillDecl};
use engine_native::content::names;
use engine_native::content::standard::Gender;

use crate::{lookup_standard_catalog_skill_zh, lookup_upstream_host_behavior, push_host_skill_if_unseen};

pub type BossGeneralEntry = (&'static str, Faction, Gender, u32, &'static [&'static str]);

/// Exact 15 upstream boss/jiange generals covered by `U02` (`boss-play-upstream-noname`).
/// First 8 entries equal `L03` (`boss-play-upstream-liuli`).
pub const U02_BOSS_GENERALS: [BossGeneralEntry; 15] = [
    ("boss_liedixuande", Faction::Shu, Gender::Male, 5, &["boss_lingfeng", "boss_jizhen"]),
    ("boss_gongshenyueying", Faction::Shu, Gender::Female, 4, &["boss_gongshenjg", "boss_jingmiao", "boss_zhinang"]),
    ("boss_tianhoukongming", Faction::Shu, Gender::Male, 4, &["boss_biantian", "bazhen"]),
    ("boss_yuhuoshiyuan", Faction::Shu, Gender::Male, 4, &["boss_yuhuojg", "boss_qiwu", "boss_tianyujg"]),
    ("boss_qiaokuijunyi", Faction::Wei, Gender::Male, 4, &["boss_huodi", "boss_jueji"]),
    ("boss_jiarenzidan", Faction::Wei, Gender::Male, 5, &["boss_chiying", "boss_jingfan"]),
    ("boss_duanyuzhongda", Faction::Wei, Gender::Male, 5, &["boss_fanshi", "boss_xuanlei", "boss_skonghun"]),
    ("boss_juechenmiaocai", Faction::Wei, Gender::Male, 5, &["boss_chuanyun", "boss_leili", "boss_fengxing"]),
    ("boss_kumuyuanrang", Faction::Wei, Gender::Male, 5, &["boss_bashi", "boss_danjing"]),
    ("boss_baijiwenyuan", Faction::Wei, Gender::Male, 5, &["boss_jiaoxie"]),
    ("boss_yihanyunchang", Faction::Shu, Gender::Male, 5, &["boss_xiaorui", "boss_huchen"]),
    ("boss_fuweizilong", Faction::Shu, Gender::Male, 5, &["boss_fengjian", "boss_keding"]),
    ("boss_lvbu1", Faction::Qun, Gender::Male, 8, &["mashu", "wushuang", "boss_baonu", "boss_jingjia", "boss_aozhan"]),
    ("boss_lvbu2", Faction::Qun, Gender::Male, 6, &["mashu", "wushuang", "xiuluo", "shenwei", "shenji"]),
    ("boss_lvbu3", Faction::Qun, Gender::Male, 6, &["wushuang", "shenqu", "jiwu"]),
];

/// Exact 8 upstream Jiange boss generals covered by `L03` (`boss-play-upstream-liuli`).
pub const L03_BOSS_GENERALS: &[BossGeneralEntry] = &[
    U02_BOSS_GENERALS[0],
    U02_BOSS_GENERALS[1],
    U02_BOSS_GENERALS[2],
    U02_BOSS_GENERALS[3],
    U02_BOSS_GENERALS[4],
    U02_BOSS_GENERALS[5],
    U02_BOSS_GENERALS[6],
    U02_BOSS_GENERALS[7],
];

const EXTRA_JIWU_SKILLS_U02: [&str; 4] = ["qiangxix", "retieji", "olxuanfeng", "rewansha"];
// L03 registers `"retieji"` before `"rexuanfeng"`; runtime dispatch distinguishes L03
// (`chooseToDiscard`, `directHit` only) from U02 (`chooseToGive`, `directHit` + 1 damage).
const EXTRA_JIWU_SKILLS_L03: [&str; 4] = ["qiangxix", "retieji", "rexuanfeng", "wansha"];

/// Resolve an upstream boss-play skill name to its native `HostBehavior`.
pub fn boss_play_skill_behavior(name: &str) -> Option<HostBehavior> {
    if matches!(
        name,
        "boss_jizhen"
            | "boss_gongshenjg"
            | "boss_jingmiao"
            | "boss_zhinang"
            | "boss_biantian"
            | "boss_yuhuojg"
            | "boss_qiwu"
            | "boss_tianyujg"
            | "boss_huodi"
            | "boss_chiying"
            | "boss_jingfan"
            | "boss_fanshi"
            | "boss_bashi"
            | "boss_danjing"
            | "boss_jiaoxie"
            | "boss_xiaorui"
            | "boss_huchen"
            | "boss_fengjian"
            | "boss_keding"
            | "boss_baonu"
            | "boss_jingjia"
            | "boss_aozhan"
            | "xiuluo"
            | "shenji"
            | "shenqu"
            | "jiwu"
            | "xuanfeng"
            | "olxuanfeng"
            | "rexuanfeng"
            | "qiangxix"
            | "retieji"
            | "rewansha"
            | "wansha"
            | "boss_lingfeng"
            | "boss_jueji"
            | "boss_leili"
            | "boss_fengxing"
            | "boss_xuanlei"
            | "boss_skonghun"
            | "bazhen"
    ) {
        return Some(HostBehavior::Q4BossPlaySkill);
    }
    match name {
        "shenwei" => Some(HostBehavior::Shenwei),
        _ => lookup_upstream_host_behavior(name),
    }
}

/// Populate all missing generals and host skills for `boss-play` onto existing output buffers.
pub fn append_boss_play_content(
    source: &str,
    seen_generals: &mut BTreeSet<String>,
    seen_skills: &mut BTreeSet<String>,
    out_generals: &mut Vec<ExtensionGeneral>,
    out_host_skills: &mut Vec<HostSkillDecl>,
) {
    let is_liuli = source == "noname-liuli" || source == "liuli";
    let entries: &[BossGeneralEntry] = if is_liuli {
        L03_BOSS_GENERALS
    } else {
        &U02_BOSS_GENERALS
    };
    let extra_jiwu: &[&str] = if is_liuli {
        &EXTRA_JIWU_SKILLS_L03
    } else {
        &EXTRA_JIWU_SKILLS_U02
    };
    for &extra_skill in extra_jiwu {
        if let Some(behavior) = boss_play_skill_behavior(extra_skill) {
            push_host_skill_if_unseen(extra_skill, behavior, out_host_skills, seen_skills);
        }
    }
    for &(_, _, _, _, sk_list) in entries {
        for &skill in sk_list {
            if let Some(behavior) = boss_play_skill_behavior(skill) {
                push_host_skill_if_unseen(skill, behavior, out_host_skills, seen_skills);
            }
        }
    }
    for &(sym, faction, gender, max_hp, sk_list) in entries {
        if !seen_generals.insert(sym.to_string()) {
            continue;
        }
        let mut resolved: Vec<&'static str> = Vec::with_capacity(sk_list.len());
        for &sk in sk_list {
            let mapped = if seen_skills.contains(sk) || boss_play_skill_behavior(sk).is_some() {
                Some(names::intern(sk.to_string()))
            } else {
                lookup_standard_catalog_skill_zh(sk)
            };
            if let Some(m) = mapped {
                if !resolved.contains(&m) {
                    resolved.push(m);
                }
            }
        }
        out_generals.push(ExtensionGeneral {
            name: names::intern(sym.to_string()),
            faction,
            skills: names::intern_list(resolved),
            max_hp,
            gender,
            presentation_binding: None,
        });
    }
}

/// Build an authoritative `ExtensionPack` for `boss-play` (`source`: `"noname"` | `"noname-liuli"`).
pub fn boss_play_extension_pack(source: &str, prior_packs: &[&ExtensionPack]) -> ExtensionPack {
    let mut seen_generals = BTreeSet::new();
    let mut seen_skills = BTreeSet::new();
    for p in prior_packs {
        for g in &p.generals {
            seen_generals.insert(g.name.to_string());
        }
        for s in &p.host_skills {
            seen_skills.insert(s.name.to_string());
        }
        for s in &p.vm_skills {
            seen_skills.insert(s.name.clone());
        }
    }
    let mut generals = Vec::new();
    let mut host_skills = Vec::new();
    append_boss_play_content(source, &mut seen_generals, &mut seen_skills, &mut generals, &mut host_skills);
    let is_liuli = source == "noname-liuli" || source == "liuli";
    ExtensionPack {
        def: ExtensionDef {
            name: if is_liuli {
                "boss-play-upstream-liuli"
            } else {
                "boss-play-upstream-noname"
            },
            version: 1,
            description: "Upstream boss/jiange boss-play generals and native skills",
        },
        generals,
        cards: Vec::new(),
        host_skills,
        vm_skills: Vec::new(),
        init_hook: InitHook::None,
        enabled: true,
    }
}
