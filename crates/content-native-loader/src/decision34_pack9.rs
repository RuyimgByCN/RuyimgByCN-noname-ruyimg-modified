//! Batch 26 (Decision 34 Complete 37-Source In-Repo Closure): native `ExtensionPack`
//! builder, coexistence filter, and state machines for the remaining 9 sources
//! (`U01`, `U05`, `L08`, `L14`, `L19`, `L20`, `L24`, `L25`, `L27`) plus `U07` (`jianbi`, `choudu`).

use std::collections::BTreeSet;

use engine_native::catalog::{CardDef, CardKind, Faction, Suit};
use engine_native::content::extension::{
    ExtensionDef, ExtensionGeneral, ExtensionPack, InitHook, VmSkillDecl,
};
use engine_native::content::names;
use engine_native::content::standard::{Gender, STANDARD_GENERALS};
use engine_native::event::EventKind;
use engine_native::rvm_bridge::VmFilterPolicy;

#[path = "decision34_pack9_data.rs"]
pub mod decision34_pack9_data;
pub use decision34_pack9_data::*;

use super::DECISION34_STORAGE_DISPATCH_KEY;

pub const DDD_SKILL_DISPATCH_ID: u32 = 1_926_001_001;
pub const SWD_DUIJUE_ID: u32 = 1_926_002_001;
pub const SWD_TANLIN_ID: u32 = 1_926_002_002;
pub const YUNCHOU_SKILL_ID: u32 = 1_926_002_003;
pub const MTG_LAND_CARD_ID: u32 = 1_926_002_004;
pub const ZHENFA_POZHENJUE_ID: u32 = 1_926_002_005;
pub const OW_SKILL_DISPATCH_ID: u32 = 1_926_002_006;
pub const XIANJIAN_SKILL_DISPATCH_ID: u32 = 1_926_002_007;
pub const WUDIAN_JIANBI_ID: u32 = 1_926_003_001;
pub const WUDIAN_CHOUDU_ID: u32 = 1_926_003_002;
pub const L08_DECADE_SKILL_ID: u32 = 1_926_004_001;
pub const L14_SHOUSHA_SKILL_ID: u32 = 1_926_005_001;
pub const L19_DOOMSDAY_SKILL_ID: u32 = 1_926_006_001;
pub const L20_SKILL_DISPATCH_ID: u32 = 1_926_007_001;
pub const L24_SKIN_SKILL_ID: u32 = 1_926_008_001;
pub const L25_ZUAN_SKILL_ID: u32 = 1_926_009_001;
pub const L27_SELECT_SKILL_ID: u32 = 1_926_010_001;
pub const L11_TAIXU_SKILL_ID: u32 = 1_926_011_001;

fn classify_skill_trigger(name: &str) -> (EventKind, VmFilterPolicy) {
    if name.ends_with("__ref_busi") {
        return (EventKind::Dying, VmFilterPolicy::OwnerIsTarget);
    }
    match name {
        "ycndiyjianxiong"
        | "cyxianjiang"
        | "ssr_newganglie"
        | "ssr_newtianxiang"
        | "ssr_newjiyuan"
        | "ssr_guimou"
        | "ssr_newduanxian"
        | "ssr_wangxi"
        | "txhj_dundunSkill2"
        | "txhj_yayaSkill2"
        | "txhj_piliche"
        | "_pfqh_check_damage"
        | "_pfqh_check_damage_times"
        | "_pfqh_check_hp_change"
        | "_zuan_damageSendEgg"
        | "_zuan_damageSendFlower"
        | "_extradamageskill"
        | "_wujinmoshiDamage"
        | "_wujinmoshiDamageE"
        | "_losehp_"
        | "_mianshang_"
        | "baonvezhi" => (EventKind::Damage, VmFilterPolicy::OwnerIsTarget),
        "tanlin"
        | "feizhua"
        | "tianjian"
        | "shanguang"
        | "ssr_wushuang"
        | "ssr_baonue"
        | "ssr_newfabing"
        | "ssr_newjilue"
        | "ssr_tieji"
        | "ssr_liegong"
        | "txhj_taijifuchen"
        | "gx_taijifuchen"
        | "_usecard"
        | "_decadeUI_usecardBegin"
        | "_changeEquipExt"
        | "_tsx"
        | "_gj"
        | "_ts"
        | "_chupaiA"
        | "_chupaiB"
        | "_chupaiC"
        | "_chupaiD"
        | "_chupaiE"
        | "_chupaiF"
        | "_chupaiG"
        | "_chupaiH"
        | "_useskill"
        | "_extrauseskill"
        | "_shiyongkapaitexiao_" => (EventKind::CardUsed, VmFilterPolicy::OwnerIsSource),
        "jianbi" | "_jd_shiyongshanD" | "_checkDcdShan" | "_shanskill" => {
            (EventKind::CardUsed, VmFilterPolicy::OwnerIsTarget)
        }
        "leiyu"
        | "zhengsu"
        | "ssr_biyue"
        | "ssr_jushou"
        | "ssr_zhuhai"
        | "ssr_shendao"
        | "_pfqh_check_phaseEnd"
        | "_jd_hhjs"
        | "_jd_phaseAfter"
        | "_phaseendskill" => (EventKind::PhaseEnd, VmFilterPolicy::OwnerIsTarget),
        "dddqianlong"
        | "_decadeUI_dieKillEffect"
        | "_kill_framebg"
        | "_dieNoTuoguan"
        | "_jd_wjsw"
        | "_check_die_yh"
        | "_pfqh_check_jisha"
        | "_killskill"
        | "_extradieskill"
        | "_dieskill"
        | "_guozhan_diebiaoji_"
        | "_shoushajisha_" => (EventKind::Death, VmFilterPolicy::OwnerIsTarget),
        "busi" | "ssr_newbuqu" | "_zuanidentitydyingskill" | "_bingleDying" => {
            (EventKind::Dying, VmFilterPolicy::OwnerIsTarget)
        }
        "_pfqh_check_roundCount" | "_zuanRoundTX" | "_chooseCharacterTwo" => {
            (EventKind::TurnBegin, VmFilterPolicy::OwnerIsTarget)
        }
        _ => (EventKind::PhaseBegin, VmFilterPolicy::OwnerIsTarget),
    }
}

pub(super) fn make_vm_skill_decl(name_str: &str, dispatch_id: u32, idx: usize) -> VmSkillDecl {
    let mod_name = format!("{name_str}#d34p9");
    let mut code = Vec::with_capacity(14);
    code.push(0x57); // HOST_SKILL
    code.push(0x10); // CONST_I32
    code.extend_from_slice(&(DECISION34_STORAGE_DISPATCH_KEY as i32).to_le_bytes());
    code.push(0x10); // CONST_I32
    code.extend_from_slice(&(dispatch_id as i32).to_le_bytes());
    code.push(0x5A); // HOST_EFFECT_EXT
    code.extend_from_slice(&649u16.to_le_bytes()); // STORAGE_SET = 649
    code.push(0x01); // HALT
    let module = engine_native::rvm_bridge::BytecodeModule {
        logic_hash: engine_native::rvm_bridge::BytecodeModule::compute_logic_hash(
            &mod_name, &code, 0, 1, 0,
        ),
        name: mod_name,
        code,
        local_count: 0,
        max_stack: 4,
        entry_pc: 0,
        caps_mask: 1,
    };
    let skill_def_id = dispatch_id.wrapping_add(idx as u32);
    let (event, inner_filter) = classify_skill_trigger(name_str);
    let extra_events = if name_str == "dddxiaxing" {
        vec![EventKind::Dying]
    } else {
        Vec::new()
    };
    VmSkillDecl {
        declaration_hash: None,
        presentation_binding: None,
        name: name_str.to_string(),
        module,
        trigger: engine_native::catalog::TriggerMeta {
            event,
            timing: engine_native::event::Timing::Normal,
            priority: 15,
            order_selectable: false,
            optional: false,
            extra_events,
        },
        filter: VmFilterPolicy::Declared {
            skill_def_id,
            hand_limit_base_max_hp: false,
            choice_actor: engine_native::rvm_bridge::VmChoiceActor::Owner,
            modifiers: Vec::new(),
            inner: Box::new(inner_filter),
        },
        r05_identity: None,
    }
}

/// Build the complete `ExtensionPack` covering the remaining 9 sources (`U01`, `U05`,
/// `L08`, `L14`, `L19`, `L20`, `L24`, `L25`, `L27`) plus `U07` (`jianbi`, `choudu`).
pub fn decision34_pack9_extension_pack() -> ExtensionPack {
    decision34_pack9_for_coexistence(&[])
}

/// Build the `ExtensionPack` for the remaining 9 sources + `U07`, automatically
/// filtering out any general, card, or skill already present in `existing_packs` or
/// `STANDARD_GENERALS` / `standard_catalog()` so multi-pack coexistence never collides.
pub fn decision34_pack9_for_coexistence(existing_packs: &[&ExtensionPack]) -> ExtensionPack {
    let std_cat = engine_native::content::standard::standard_catalog();
    let mut seen_generals: BTreeSet<String> = STANDARD_GENERALS
        .iter()
        .map(|g| g.name.to_string())
        .collect();
    let mut seen_cards: BTreeSet<String> =
        std_cat.cards.iter().map(|c| c.name.to_string()).collect();
    let mut seen_skills: BTreeSet<String> =
        std_cat.skills.iter().map(|s| s.name.to_string()).collect();

    for p in existing_packs {
        for g in &p.generals {
            seen_generals.insert(g.name.to_string());
        }
        for c in &p.cards {
            seen_cards.insert(c.name.to_string());
        }
        for hs in &p.host_skills {
            seen_skills.insert(hs.name.to_string());
        }
        for vs in &p.vm_skills {
            seen_skills.insert(vs.name.clone());
        }
    }

    let mut generals: Vec<ExtensionGeneral> = Vec::new();
    let mut push_general =
        |sym: &str, faction: Faction, gender: Gender, max_hp: u32, sk_list: &[&str]| {
            if seen_generals.insert(sym.to_string()) {
                let interned: Vec<&'static str> = sk_list
                    .iter()
                    .map(|&s| names::intern(s.to_string()))
                    .collect();
                generals.push(ExtensionGeneral {
                    name: names::intern(sym.to_string()),
                    faction,
                    skills: names::intern_list(interned),
                    max_hp,
                    gender,
                    presentation_binding: None,
                });
            }
        };

    // U01: 31 `ddd_*` generals (`archive/upstream/noname/apps/core/extension/3D精选/character/character.js:5-193`)
    for &(sym, faction, gender, max_hp, sk_list) in &U01_DDD_GENERALS {
        push_general(sym, faction, gender, max_hp, sk_list);
    }
    // U05 Xianjian: all 36 `pal_*` generals (`archive/upstream/noname/apps/core/character/xianjian.js`)
    for &(sym, faction, gender, max_hp, sk_list) in &U05_XIANJIAN_GENERALS {
        push_general(sym, faction, gender, max_hp, sk_list);
    }
    // U07 Wudian: all 9 `wdb_*` generals (`archive/upstream/noname/apps/core/extension/玩点论杀/character/character.js:5-62`)
    for &(sym, faction, gender, max_hp, sk_list) in &U07_WUDIAN_GENERALS {
        push_general(sym, faction, gender, max_hp, sk_list);
    }
    // U08 Hero Kill: all 43 `yxs_*` generals (`archive/upstream/noname/apps/core/extension/英雄杀/character/character.js:3-287`)
    for &(sym, faction, gender, max_hp, sk_list) in &U08_YXS_GENERALS {
        push_general(sym, faction, gender, max_hp, sk_list);
    }
    // L19: 2 authentic Doomsday Brawl zombie generals (`jiangshifz`, `jiangshinj`, `末日乱斗/extension.js:3383-3384`)
    push_general(
        "jiangshifz",
        Faction::Qun,
        Gender::Male,
        5,
        &["xunmeng", "zaibian", "ganran", "wansha", "paoxiao"],
    );
    push_general(
        "jiangshinj",
        Faction::Qun,
        Gender::Female,
        3,
        &["baozou", "wansha", "xueji", "shishi", "ganran"],
    );
    // L25: 60 authentic Zuan `ssr_*` characters (`祖安武将/extension.js:338-397`)
    for &(sym, faction, gender, max_hp, sk_list) in &super::L25_SSR_GENERALS {
        push_general(sym, faction, gender, max_hp, sk_list);
    }
    // L29: `mb_caomao` and `mb_caomao2` (`龙血玄黄/extension.js:24-25`)
    push_general(
        "mb_caomao",
        Faction::Wei,
        Gender::Male,
        3,
        &[
            "mbqianlong",
            "mbcmqingzheng",
            "mbcmjiushi",
            "mbcmfangzhu",
            "mbjuejin",
            "mb_fankui",
        ],
    );
    push_general(
        "mb_caomao2",
        Faction::Wei,
        Gender::Male,
        3,
        &[
            "mbqianlong",
            "mbcmqingzheng",
            "mbcmjiushi",
            "mbcmfangzhu",
            "mbjuejin",
        ],
    );

    // Cards: U05 MTG (2), U05 Zhenfa (1), L20 (3 `zhengsu_*` cards from `标记补充/extension.js:6319-6321`),
    // L01 (`yiyi` from `AI优化/extension.js:9205`), L06 (`wuxingpan` from `五行生克/extension.js:116`),
    // L25 (`ssr_newsance_*` & `ssr_yongsi_yuxi_card` from `祖安武将/extension.js:6535-28385`)
    let mut cards: Vec<CardDef> = Vec::new();
    let card_specs: [(&str, CardKind, Suit, u8); 8] = [
        ("mtg_duzhao", CardKind::Wuzhong, Suit::Spade, 1),
        ("mtg_cangbaohaiwan", CardKind::Wuzhong, Suit::Heart, 1),
        ("pozhenjue", CardKind::Wuzhong, Suit::Club, 7),
        ("zhengsu_leijin", CardKind::Wuzhong, Suit::Spade, 3),
        ("zhengsu_mingzhi", CardKind::Wuzhong, Suit::Heart, 4),
        ("zhengsu_bianzhen", CardKind::Wuzhong, Suit::Club, 5),
        ("yiyi", CardKind::Wuzhong, Suit::Heart, 4),
        ("wuxingpan", CardKind::Wuzhong, Suit::Diamond, 5),
    ];
    for &(cname, kind, suit, rank) in card_specs.iter().chain(super::L25_ZUAN_FORMAL_CARDS.iter()) {
        if seen_cards.insert(cname.to_string()) {
            let interned = names::intern(cname.to_string());
            cards.push(CardDef::basic(interned, kind).with_suit_rank(suit, rank));
        }
    }

    // Skills across all sources
    let mut vm_skills: Vec<VmSkillDecl> = Vec::new();
    let mut idx = 0usize;
    let mut push_skill = |name: &str, dispatch_id: u32| {
        if seen_skills.insert(name.to_string()) {
            vm_skills.push(make_vm_skill_decl(name, dispatch_id, idx));
            idx += 1;
        }
    };

    for &s in &U01_DDD_SKILLS {
        push_skill(s, DDD_SKILL_DISPATCH_ID);
    }
    push_skill("duijue", SWD_DUIJUE_ID);
    push_skill("tanlin", SWD_TANLIN_ID);
    for &s in &U05_YUNCHOU_REMAINING_SKILLS {
        push_skill(s, YUNCHOU_SKILL_ID);
    }
    for &(cname, _, _, g_sk) in &U05_MTG_LAND_CARDS {
        push_skill(cname, MTG_LAND_CARD_ID);
        push_skill(g_sk, MTG_LAND_CARD_ID);
    }
    for &(cname, _, _, g_sk) in &U05_ZHENFA_CARDS {
        push_skill(cname, ZHENFA_POZHENJUE_ID);
        push_skill(g_sk, ZHENFA_POZHENJUE_ID);
    }
    for &s in &U05_OW_SKILLS {
        push_skill(s, OW_SKILL_DISPATCH_ID);
    }
    for &s in &U05_XIANJIAN_SKILLS {
        push_skill(s, XIANJIAN_SKILL_DISPATCH_ID);
    }
    for &(_, _, _, _, sk_list) in &U05_XIANJIAN_GENERALS {
        for &s in sk_list {
            push_skill(s, XIANJIAN_SKILL_DISPATCH_ID);
        }
    }
    push_skill("kuwu", super::HUANLE_KUWU_INTERACTIVE_ID);
    for &s in &["monkey", "mianju", "xuelunyang", "jiuwei"] {
        push_skill(s, super::U06_HUANLE_SKILL_ID);
    }
    push_skill("jianbi", WUDIAN_JIANBI_ID);
    push_skill("choudu", WUDIAN_CHOUDU_ID);
    for &s in &U07_WUDIAN_ALL_SKILLS {
        push_skill(s, super::U07_WUDIAN_SKILL_ID);
    }
    for &s in &U08_YXS_SKILLS {
        push_skill(s, super::U08_YXS_SKILL_ID);
    }
    for &s in &super::L10_SCS_COMPANION_SKILLS {
        push_skill(s, super::L10_SCS_SKILL_ID);
    }
    push_skill("yiyi", super::L01_YIYI_ID);
    for &s in &["_shengke", "wuxingpan_skill", "wuxing"] {
        push_skill(s, super::L06_WUXING_ID);
    }
    for &s in &L08_DECADE_SKILLS {
        push_skill(s, L08_DECADE_SKILL_ID);
    }
    push_skill("_chongzhu", L08_DECADE_SKILL_ID);
    for &s in &L14_SHOUSHA_SKILLS {
        push_skill(s, L14_SHOUSHA_SKILL_ID);
    }
    for &s in &L19_DOOMSDAY_SKILLS {
        push_skill(s, L19_DOOMSDAY_SKILL_ID);
    }
    for &s in &["wansha", "paoxiao", "xueji"] {
        push_skill(s, L19_DOOMSDAY_SKILL_ID);
    }
    for &s in &L20_MARK_SUPPLEMENT_SKILLS {
        push_skill(s, L20_SKILL_DISPATCH_ID);
    }
    for &s in &L24_SKIN_SWITCH_SKILLS {
        push_skill(s, L24_SKIN_SKILL_ID);
    }
    for &s in super::L25_SSR_GENERAL_SKILLS
        .iter()
        .chain(super::L25_ALL_RULE_SKILLS.iter())
        .chain(super::L25_ZUAN_CARD_ENTRIES.iter())
        .chain(["ssr_yongsi_yuxi_card"].iter())
        .chain(L25_ZUAN_SKILLS.iter())
    {
        push_skill(s, L25_ZUAN_SKILL_ID);
    }
    for &s in &L27_SELECT_BEAUTIFY_SKILLS {
        push_skill(s, L27_SELECT_SKILL_ID);
    }
    push_skill("_chooseCharacterTwo", L27_SELECT_SKILL_ID);
    for &s in &["mbcmqingzheng", "mbqianlong", "mb_fankui", "_CMeffects"] {
        push_skill(s, super::L29_LONGXUE_SKILL_ID);
    }

    ExtensionPack {
        def: ExtensionDef {
            name: "decision34-pack9-closure",
            version: 1,
            description:
                "Decision 34 authentic upstream pack for U01/U05/U06/U07/U08/L01/L06/L08/L14/L19/L20/L24/L25/L27/L29",
        },
        generals,
        cards,
        host_skills: Vec::new(),
        vm_skills,
        init_hook: InitHook::None,
        enabled: true,
    }
}

/// Wire authentic upstream generals, cards, and VM skills into a formal `packages/extensions/*`
/// `ExtensionPack` loaded on the production `locked_content` path (`Web/WASM`, `Worker`, `Server`, `Desktop`).
pub fn attach_formal_package_upstream_rules(
    namespace: &str,
    package_id: &str,
    pack: &mut ExtensionPack,
    prior_packs: &[ExtensionPack],
) {
    let std_cat = engine_native::content::standard::standard_catalog();
    let mut seen_generals: BTreeSet<String> = STANDARD_GENERALS
        .iter()
        .map(|g| g.name.to_string())
        .collect();
    let mut seen_cards: BTreeSet<String> = std_cat
        .cards
        .iter()
        .map(|c| c.name.to_string())
        .collect();
    let mut seen_skills: BTreeSet<String> = std_cat
        .skills
        .iter()
        .map(|s| s.name.to_string())
        .collect();
    for p in prior_packs {
        for g in &p.generals {
            seen_generals.insert(g.name.to_string());
        }
        for c in &p.cards {
            seen_cards.insert(c.name.to_string());
        }
        for hs in &p.host_skills {
            seen_skills.insert(hs.name.to_string());
        }
        for vs in &p.vm_skills {
            seen_skills.insert(vs.name.clone());
        }
    }

    // Deduplicate pre-parsed formal package entries against standard catalog and prior_packs
    // so multi-package loading is symmetric regardless of package order (e.g. `[shahai-zhenfa, wuxing]`).
    pack.generals.retain(|g| seen_generals.insert(g.name.to_string()));
    pack.cards.retain(|c| seen_cards.insert(c.name.to_string()));
    pack.host_skills.retain(|hs| seen_skills.insert(hs.name.to_string()));
    pack.vm_skills.retain(|vs| seen_skills.insert(vs.name.clone()));

    if super::try_attach_shahai_formal_package(
        namespace,
        package_id,
        pack,
        prior_packs,
        &mut seen_generals,
        &mut seen_cards,
        &mut seen_skills,
    ) {
        return;
    }

    let had_generals = !pack.generals.is_empty();
    let had_cards = !pack.cards.is_empty();

    let mut push_general =
        |sym: &str, faction: Faction, gender: Gender, max_hp: u32, sk_list: &[&str]| {
            if seen_generals.insert(sym.to_string()) {
                let interned: Vec<&'static str> = sk_list
                    .iter()
                    .map(|&s| names::intern(s.to_string()))
                    .collect();
                pack.generals.push(ExtensionGeneral {
                    name: names::intern(sym.to_string()),
                    faction,
                    skills: names::intern_list(interned),
                    max_hp,
                    gender,
                    presentation_binding: None,
                });
            }
        };

    let mut push_card = |cname: &str, kind: CardKind, suit: Suit, rank: u8| {
        if seen_cards.insert(cname.to_string()) {
            let interned = names::intern(cname.to_string());
            pack.cards
                .push(CardDef::basic(interned, kind).with_suit_rank(suit, rank));
        }
    };

    let mut idx = pack.vm_skills.len();
    let mut push_skill = |name: &str, dispatch_id: u32| {
        if seen_skills.insert(name.to_string()) {
            pack.vm_skills.push(make_vm_skill_decl(name, dispatch_id, idx));
            idx += 1;
        }
    };

    match (namespace, package_id) {
        ("upstream-noname", "d3-select") => {
            for &(sym, faction, gender, max_hp, sk_list) in &U01_DDD_GENERALS {
                push_general(sym, faction, gender, max_hp, sk_list);
            }
            for &s in &U01_DDD_SKILLS {
                push_skill(s, DDD_SKILL_DISPATCH_ID);
            }
        }
        ("upstream-noname", "huanle-cards") => {
            push_skill("kuwu", super::HUANLE_KUWU_INTERACTIVE_ID);
            push_skill("huanle_kuwu_interactive", super::HUANLE_KUWU_INTERACTIVE_ID);
            for &s in &["monkey", "mianju", "xuelunyang", "jiuwei"] {
                push_skill(s, super::U06_HUANLE_SKILL_ID);
            }
        }
        ("upstream-noname", "wudian") => {
            if !had_generals {
                for &(sym, faction, gender, max_hp, sk_list) in &U07_WUDIAN_GENERALS {
                    push_general(sym, faction, gender, max_hp, sk_list);
                }
            }
            push_skill("diaobingqianjiang", super::DIAOBINGQIANJIANG_ID);
            push_skill("duoqi", super::DUOQI_ID);
            push_skill("youzhan", super::YOUZHAN_ID);
            push_skill("jianbi", WUDIAN_JIANBI_ID);
            push_skill("choudu", WUDIAN_CHOUDU_ID);
            for &s in &U07_WUDIAN_ALL_SKILLS {
                push_skill(s, super::U07_WUDIAN_SKILL_ID);
            }
            for g in &mut pack.generals {
                let extra = match g.name {
                    "刘馥" | "wdb_liufu" => Some("duoqi"),
                    "田豫" | "wdb_tianyu" => Some("youzhan"),
                    "造之仁君" | "wdb_zaozhirenjun" => Some("jianbi"),
                    "杨仪" | "wdb_yangyi" => Some("choudu"),
                    _ => None,
                };
                if let Some(sk) = extra {
                    if !g.skills.contains(&sk) {
                        let mut updated: Vec<&'static str> = g.skills.to_vec();
                        updated.push(names::intern(sk.to_string()));
                        g.skills = names::intern_list(updated);
                    }
                }
            }
        }
        ("upstream-noname", "hero-kill") => {
            for &(sym, faction, gender, max_hp, sk_list) in &U08_YXS_GENERALS {
                push_general(sym, faction, gender, max_hp, sk_list);
            }
            push_skill("yxs_menshen2_guard", super::YXS_MENSHEN2_GUARD_ID);
            for &s in &U08_YXS_SKILLS {
                push_skill(s, super::U08_YXS_SKILL_ID);
            }
        }
        ("upstream-liuli", "ai-tuning") => {
            push_card("yiyi", CardKind::Wuzhong, Suit::Heart, 4);
            push_skill("yiyi", super::L01_YIYI_ID);
        }
        ("upstream-liuli", "wuxing") => {
            if !had_cards {
                push_card("wuxingpan", CardKind::Wuzhong, Suit::Diamond, 5);
            }
            for &s in &["_shengke", "wuxingpan_skill", "wuxing"] {
                push_skill(s, super::L06_WUXING_ID);
            }
        }
        ("upstream-liuli", "epic-cards") => {
            push_general(
                "shichangshi",
                Faction::Qun,
                Gender::Male,
                1,
                &["k_danggu", "k_mowang"],
            );
            push_skill("k_danggu", super::K_DANGGU_ID);
            push_skill("k_mowang", super::K_MOWANG_ID);
            for &s in &super::L10_SCS_COMPANION_SKILLS {
                push_skill(s, super::L10_SCS_SKILL_ID);
            }
        }
        ("upstream-liuli", "doomsday-brawl") => {
            push_general(
                "jiangshifz",
                Faction::Qun,
                Gender::Male,
                5,
                &["xunmeng", "zaibian", "ganran", "wansha", "paoxiao"],
            );
            push_general(
                "jiangshinj",
                Faction::Qun,
                Gender::Female,
                3,
                &["baozou", "wansha", "xueji", "shishi", "ganran"],
            );
            for &s in &L19_DOOMSDAY_SKILLS {
                push_skill(s, L19_DOOMSDAY_SKILL_ID);
            }
            for &s in &["wansha", "paoxiao", "xueji"] {
                push_skill(s, L19_DOOMSDAY_SKILL_ID);
            }
        }
        ("upstream-liuli", "mark-supplement") => {
            for &cname in &L20_MARK_SUPPLEMENT_CARDS {
                push_card(cname, CardKind::Wuzhong, Suit::Spade, 3);
            }
            for &s in &L20_MARK_SUPPLEMENT_SKILLS {
                push_skill(s, L20_SKILL_DISPATCH_ID);
            }
            push_skill("l20_mark_dispatch", super::L20_MARK_DISPATCH_ID);
        }
        ("upstream-liuli", "taixu") => {
            for &(sym, faction, gender, max_hp, sk_list) in &super::L11_TAIXU_GENERALS {
                push_general(sym, faction, gender, max_hp, sk_list);
            }
            for &(cname, kind, range, suit, rank) in &super::L11_TAIXU_CARDS {
                if seen_cards.insert(cname.to_string()) {
                    let interned = names::intern(cname.to_string());
                    let card_def = if kind == CardKind::Weapon {
                        CardDef::weapon(interned, range).with_suit_rank(suit, rank)
                    } else {
                        CardDef::basic(interned, kind).with_suit_rank(suit, rank)
                    };
                    pack.cards.push(card_def);
                }
            }
            for &s in super::L11_TAIXU_SKILLS
                .iter()
                .chain(super::L11_TAIXU_CROSS_SKILLS.iter())
                .chain(super::L11_TAIXU_CARD_IDS.iter())
            {
                push_skill(s, L11_TAIXU_SKILL_ID);
            }
        }
        ("upstream-liuli", "zuan") => {
            for &(sym, faction, gender, max_hp, sk_list) in &super::L25_SSR_GENERALS {
                push_general(sym, faction, gender, max_hp, sk_list);
            }
            for &(cname, kind, suit, rank) in &super::L25_ZUAN_FORMAL_CARDS {
                push_card(cname, kind, suit, rank);
            }
            for &s in super::L25_SSR_GENERAL_SKILLS
                .iter()
                .chain(super::L25_ALL_RULE_SKILLS.iter())
                .chain(super::L25_ZUAN_CARD_ENTRIES.iter())
                .chain(["ssr_yongsi_yuxi_card"].iter())
                .chain(L25_ZUAN_SKILLS.iter())
            {
                push_skill(s, L25_ZUAN_SKILL_ID);
            }
        }
        ("upstream-liuli", "select-beautify") => {
            for &s in &L27_SELECT_BEAUTIFY_SKILLS {
                push_skill(s, L27_SELECT_SKILL_ID);
            }
            push_skill("_chooseCharacterTwo", L27_SELECT_SKILL_ID);
        }
        ("upstream-liuli", "thunder") => {
            push_card("th_taipingyaoshu", CardKind::Wuzhong, Suit::Heart, 3);
            for &(sym, faction, gender, max_hp, sk_list) in &super::L28_THUNDER_GENERALS {
                push_general(sym, faction, gender, max_hp, sk_list);
            }
            push_skill("th_dunshi", super::TH_DUNSHI_ID);
            push_skill("th_jingxie", super::TH_JINGXIE_ID);
            push_skill("th_qiaosi", super::TH_QIAOSI_ID);
            push_skill("th_yeyan", super::TH_YEYAN_ID);
            push_skill("th_taipingyaoshu", super::TH_TAIPINGYAOSHU_ID);
            for &s in &super::L28_THUNDER_EXTRA_SKILLS {
                push_skill(s, super::TH_SKILL_DISPATCH_ID);
            }
        }
        ("upstream-liuli", "longxue") => {
            push_general(
                "mb_caomao",
                Faction::Wei,
                Gender::Male,
                3,
                &[
                    "mbqianlong",
                    "mbcmqingzheng",
                    "mbcmjiushi",
                    "mbcmfangzhu",
                    "mbjuejin",
                    "mb_fankui",
                ],
            );
            push_general(
                "mb_caomao2",
                Faction::Wei,
                Gender::Male,
                3,
                &[
                    "mbqianlong",
                    "mbcmqingzheng",
                    "mbcmjiushi",
                    "mbcmfangzhu",
                    "mbjuejin",
                ],
            );
            push_skill("mbcmjiushi", super::MBCMJIUSHI_ID);
            push_skill("mbcmfangzhu", super::MBCMFANGZHU_ID);
            push_skill("mbjuejin", super::MBJUEJIN_ID);
            for &s in &["mbcmqingzheng", "mbqianlong", "mb_fankui", "_CMeffects"] {
                push_skill(s, super::L29_LONGXUE_SKILL_ID);
            }
        }
        _ => {}
    }
}

/// `U01` (`3D精选`) 3D perspective standee gallery state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct U01GalleryState {
    pub selected_general: String,
    pub yaw_deg: i16,
    pub pitch_deg: i16,
    pub zoom_pct: u16,
}

/// Resolve and validate `U01` 3D select gallery camera/standee parameters.
pub fn resolve_u01_3d_gallery(
    general_id: &str,
    yaw_deg: i16,
    pitch_deg: i16,
    zoom_pct: u16,
) -> Result<U01GalleryState, String> {
    if !U01_DDD_GENERALS.iter().any(|&(g, _, _, _, _)| g == general_id) {
        return Err(format!("UNKNOWN_U01_DDD_GENERAL:{general_id}"));
    }
    Ok(U01GalleryState {
        selected_general: general_id.to_string(),
        yaw_deg: yaw_deg.clamp(-45, 45),
        pitch_deg: pitch_deg.clamp(-30, 30),
        zoom_pct: zoom_pct.clamp(80, 150),
    })
}

/// `L24` (`皮肤切换`) dynamic skin transform & Spine animation clip state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct L24SkinTransform {
    pub general_id: String,
    pub skin_slot: u8,
    pub stand_x: i32,
    pub stand_y: i32,
    pub scale_pct: u16,
    pub active_clip: String,
}

/// Resolve and validate `L24` dynamic skin coordinate offsets and clip triggers
/// (`daiji`, `gongji`, `teshu`, `chuchang`, `shan`).
pub fn resolve_l24_skin_transform(
    general_id: &str,
    skin_slot: u8,
    clip: &str,
) -> Result<L24SkinTransform, String> {
    if general_id.trim().is_empty() {
        return Err("EMPTY_GENERAL_ID".to_string());
    }
    if !(1..=4).contains(&skin_slot) {
        return Err(format!("INVALID_SKIN_SLOT:{skin_slot}"));
    }
    if !matches!(clip, "daiji" | "gongji" | "teshu" | "chuchang" | "shan") {
        return Err(format!("INVALID_SPINE_CLIP:{clip}"));
    }
    let slot_i = i32::from(skin_slot);
    let (dx, dy, scale_pct) = match clip {
        "gongji" => (slot_i * 12, -8, 108),
        "teshu" => (slot_i * 8, -14, 112),
        "chuchang" => (0, -20, 115),
        "shan" => (-slot_i * 10, 4, 96),
        _ => (slot_i * 4, 0, 100),
    };
    Ok(L24SkinTransform {
        general_id: general_id.to_string(),
        skin_slot,
        stand_x: dx,
        stand_y: dy,
        scale_pct,
        active_clip: clip.to_string(),
    })
}

/// `L25` (`祖安扩展`) offline-gated sandboxed DIY general/skill editor draft.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct L25DiyEditorDraft {
    pub general_id: String,
    pub faction: String,
    pub max_hp: u8,
    pub skills: Vec<String>,
    pub online_room_active: bool,
}

/// Validate and apply an `L25` DIY character draft; strictly rejects online multiplayer rooms.
pub fn validate_and_apply_l25_diy(draft: &L25DiyEditorDraft) -> Result<String, String> {
    if draft.online_room_active {
        return Err("ONLINE_MULTIPLAYER_DIY_FORBIDDEN".to_string());
    }
    if draft.general_id.trim().is_empty() {
        return Err("EMPTY_DIY_GENERAL_ID".to_string());
    }
    if !(1..=15).contains(&draft.max_hp) {
        return Err(format!("INVALID_DIY_MAX_HP:{}", draft.max_hp));
    }
    if draft.skills.is_empty() {
        return Err("EMPTY_DIY_SKILLS".to_string());
    }
    Ok(format!(
        "diy_ok:{}|faction:{}|hp:{}|skills:{}",
        draft.general_id,
        draft.faction,
        draft.max_hp,
        draft.skills.join(",")
    ))
}

/// `L27` (`选将美化`) dual-general selection (`chooseCharacterTwo`) state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct L27DualGeneralPick {
    pub candidates: Vec<String>,
    pub selected: Vec<String>,
}

/// Resolve and validate `L27` `chooseCharacterTwo` dual-general pick.
pub fn resolve_l27_choose_character_two(
    pick: &L27DualGeneralPick,
) -> Result<(String, String, u8), String> {
    if pick.selected.len() != 2 {
        return Err(format!("DUAL_PICK_REQUIRES_2_GOT_{}", pick.selected.len()));
    }
    let primary = &pick.selected[0];
    let secondary = &pick.selected[1];
    if primary == secondary {
        return Err("DUAL_PICK_DUPLICATE_GENERAL".to_string());
    }
    if !pick.candidates.contains(primary) || !pick.candidates.contains(secondary) {
        return Err("DUAL_PICK_NOT_IN_CANDIDATES".to_string());
    }
    Ok((primary.clone(), secondary.clone(), 4))
}
