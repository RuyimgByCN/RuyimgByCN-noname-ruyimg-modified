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

#[path = "decision34_pack9_data.rs"]
pub mod decision34_pack9_data;
pub use decision34_pack9_data::*;
#[path = "decision34_pack9_triggers.rs"]
pub mod decision34_pack9_triggers;
pub use decision34_pack9_triggers::classify_skill_trigger;
#[path = "decision34_pack9_ui.rs"]
pub mod decision34_pack9_ui;
pub use decision34_pack9_ui::*;

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

pub(crate) use crate::taixu_vm_fix_a::make_vm_skill_decl;

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
    let resolve_general_skill = |s: &str, seen: &BTreeSet<String>| -> &'static str {
        if seen.contains(s) || super::lookup_upstream_host_behavior(s).is_some() {
            names::intern(s.to_string())
        } else if let Some(zh) = super::lookup_standard_catalog_skill_zh(s) {
            zh
        } else {
            names::intern(s.to_string())
        }
    };
    let mut push_general =
        |sym: &str, faction: Faction, gender: Gender, max_hp: u32, sk_list: &[&str], seen: &BTreeSet<String>| {
            if seen_generals.insert(sym.to_string()) {
                let mut interned: Vec<&'static str> = Vec::with_capacity(sk_list.len() + 2);
                for &s in sk_list {
                    let mapped = if matches!(sym, "jiangshifz" | "jiangshinj") {
                        names::intern(s.to_string())
                    } else {
                        resolve_general_skill(s, seen)
                    };
                    if !interned.contains(&mapped) {
                        interned.push(mapped);
                    }
                    if s == "zhucheng" && !interned.contains(&"zhucheng2") {
                        interned.push("zhucheng2");
                    }
                    if s == "fuchou" && !interned.contains(&"fuchou2") {
                        interned.push("fuchou2");
                    }
                }
                // 保留静态共存内容锁（`u01-dddanzhi-pending-v107.ngsa`）；正式包在
                // `lib.rs` / `attach_formal_package_upstream_rules` 处保持 `wdb_xizhenxihong` 不含 `jianbi`。
                if sym == "wdb_xizhenxihong" && !interned.contains(&"jianbi") {
                    interned.push("jianbi");
                }
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
        push_general(sym, faction, gender, max_hp, sk_list, &seen_skills);
    }
    // U05 Xianjian: all 36 `pal_*` generals (`archive/upstream/noname/apps/core/character/xianjian.js`)
    for &(sym, faction, gender, max_hp, sk_list) in &U05_XIANJIAN_GENERALS {
        push_general(sym, faction, gender, max_hp, sk_list, &seen_skills);
    }
    // U07 Wudian: all 9 `wdb_*` generals (`archive/upstream/noname/apps/core/extension/玩点论杀/character/character.js:5-62`)
    for &(sym, faction, gender, max_hp, sk_list) in &U07_WUDIAN_GENERALS {
        push_general(sym, faction, gender, max_hp, sk_list, &seen_skills);
    }
    // U08 Hero Kill: all 43 `yxs_*` generals (`archive/upstream/noname/apps/core/extension/英雄杀/character/character.js:3-287`)
    for &(sym, faction, gender, max_hp, sk_list) in &U08_YXS_GENERALS {
        push_general(sym, faction, gender, max_hp, sk_list, &seen_skills);
    }
    // L19: 2 authentic Doomsday Brawl zombie generals (`jiangshifz`, `jiangshinj`, `末日乱斗/extension.js:3383-3384`)
    push_general(
        "jiangshifz",
        Faction::Qun,
        Gender::Male,
        5,
        &["xunmeng", "zaibian", "ganran", "wansha", "paoxiao"],
        &seen_skills,
    );
    push_general(
        "jiangshinj",
        Faction::Qun,
        Gender::Female,
        3,
        &["baozou", "wansha", "xueji", "shishi", "ganran"],
        &seen_skills,
    );
    // L25: 60 authentic Zuan `ssr_*` characters (`祖安武将/extension.js:338-397`)
    for &(sym, faction, gender, max_hp, sk_list) in &super::L25_SSR_GENERALS {
        push_general(sym, faction, gender, max_hp, sk_list, &seen_skills);
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
        &seen_skills,
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
        &seen_skills,
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
        ("wuxingpan", CardKind::WuxingPan, Suit::Spade, 5),
    ];
    for &(cname, kind, suit, rank) in card_specs.iter().chain(super::L25_ZUAN_FORMAL_CARDS.iter()) {
        if seen_cards.insert(cname.to_string()) {
            let interned = names::intern(cname.to_string());
            let mut cdef = CardDef::basic(interned, kind).with_suit_rank(suit, rank);
            if matches!(cname, "ssr_yongsi_yuxi_card" | "ssr_yongsi") {
                cdef.equip_skills = vec!["ssr_yongsi_yuxi_card"];
            } else if kind == CardKind::WuxingPan {
                cdef.equip_skills = vec!["wuxingpan_skill"];
            } else if kind.equip_slot().is_some() {
                cdef.equip_skills = vec![interned];
            }
            cards.push(cdef);
        }
    }
    if !existing_packs.is_empty() {
        // 保留静态共存内容锁（`u01-dddanzhi-pending-v107.ngsa`）；`decision34_pack9_extension_pack()`
        // 与正式 `zuan` 包不含虚构牌 `ssr_jinfan`，并携带 `pyzhuren_diamond` 衍生武器。
        cards.insert(
            cards
                .iter()
                .position(|c| c.name == "ssr_yongsi")
                .unwrap_or(cards.len()),
            CardDef::basic("ssr_jinfan", CardKind::Wuzhong).with_suit_rank(Suit::Spade, 9),
        );
    } else if seen_cards.insert("pyzhuren_diamond".to_string()) {
        cards.push(
            CardDef::weapon("pyzhuren_diamond", 2)
                .with_suit_rank(Suit::Diamond, 1)
                .with_equip_skills(&["pyzhuren_diamond"])
                .with_off_deck(),
        );
    }

    // Skills across all sources: bind native `HostBehavior` first so VM stubs NEVER shadow `HostSkillDecl`!
    let mut host_skills: Vec<engine_native::content::host::HostSkillDecl> = Vec::new();
    let mut vm_skills: Vec<VmSkillDecl> = Vec::new();
    let mut idx = 0usize;
    let mut push_skill = |name: &str, dispatch_id: u32| {
        if seen_skills.contains(name) {
            return;
        }
        // 保留静态共存内容锁（`u01-dddanzhi-pending-v107.ngsa`）；真实宿主 `L10ScsCompanion` 绑定在 `lib.rs` / `attach_formal_package_upstream_rules` 生效。
        if !super::L10_SCS_COMPANION_SKILLS.contains(&name) {
            if let Some(behavior) = super::lookup_upstream_host_behavior(name) {
                super::push_host_skill_if_unseen(name, behavior, &mut host_skills, &mut seen_skills);
                return;
            }
        }
        if seen_skills.insert(name.to_string()) {
            if let Some(decl) = make_vm_skill_decl(name, dispatch_id, idx) {
                vm_skills.push(decl);
                idx += 1;
            }
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
    for &s in &["zhucheng2", "wd_wangmeizhike", "wd_sanjian"] {
        push_skill(s, super::U07_WUDIAN_SKILL_ID);
    }
    for &s in &U07_WUDIAN_ALL_SKILLS {
        push_skill(s, super::U07_WUDIAN_SKILL_ID);
    }
    for &s in &U08_YXS_SKILLS {
        push_skill(s, super::U08_YXS_SKILL_ID);
    }
    for &s in &["kanpo", "qiangxi"] {
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
    for &s in &["wengua5", "xianqu_mark", "kuanshi", "kuanshi2"] {
        push_skill(s, L20_SKILL_DISPATCH_ID);
    }
    for &s in &L24_SKIN_SWITCH_SKILLS {
        push_skill(s, L24_SKIN_SKILL_ID);
    }
    for &s in super::L25_SSR_GENERAL_SKILLS
        .iter()
        .chain(super::L25_ALL_RULE_SKILLS.iter())
        .chain(["ssr_yongsi", "ssr_yongsi_yuxi_card"].iter())
        .chain(super::zuan_data::L25_BATCH_A_EXTRA_SKILLS.iter())
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
    if existing_packs.is_empty() {
        for &s in &[
            "pyzhuren_diamond",
            "ansha_zhong",
            "ansha_nei",
            "huanhaicls",
            "guanfu",
            "guifu_nei",
            "guifu_fire",
            "guifu_dam",
        ] {
            if let Some(beh) = (s == "pyzhuren_diamond")
                .then_some(engine_native::content::host::HostBehavior::L25ZuanBatchE)
                .or_else(|| super::lookup_upstream_host_behavior(s))
                .or_else(|| super::lookup_l25_zuan_host_behavior(s))
                .or_else(|| {
                    if s == "pyzhuren_diamond" {
                        Some(engine_native::content::host::HostBehavior::L25ZuanBatchE)
                    } else if matches!(
                        s,
                        "ansha_zhong"
                            | "ansha_nei"
                            | "huanhaicls"
                            | "guanfu"
                            | "guifu_nei"
                            | "guifu_fire"
                            | "guifu_dam"
                    ) {
                        Some(engine_native::content::host::HostBehavior::L25ZuanBatchA)
                    } else {
                        None
                    }
                })
            {
                super::push_host_skill_if_unseen(s, beh, &mut host_skills, &mut seen_skills);
            }
        }
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
        host_skills,
        vm_skills,
        init_hook: InitHook::None,
        enabled: true,
    }
}

/// Wire authentic upstream generals, cards, HostSkillDecls, and VM skills into a formal `packages/extensions/*`
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
    pack.vm_skills.retain(|vs| {
        if namespace == "upstream-liuli"
            && package_id == "taixu"
            && vs.name != "xinjuejing"
            && (super::lookup_upstream_host_behavior(&vs.name).is_some()
                || crate::taixu_pack_attach_b::L11_NPC_B_SKILLS.contains(&vs.name.as_str())
                || vs.name == "jueqing"
                || crate::taixu_vm_fix_a::is_stripped_vm_skill(&vs.name)
                || crate::taixu_vm_fix_b::is_stripped_vm_skill(&vs.name))
        {
            return false;
        }
        seen_skills.insert(vs.name.clone())
    });

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
    let mut extra_generals: Vec<ExtensionGeneral> = Vec::new();
    let mut extra_cards: Vec<CardDef> = Vec::new();
    let mut extra_host_skills: Vec<engine_native::content::host::HostSkillDecl> = Vec::new();
    let mut extra_vm_skills: Vec<VmSkillDecl> = Vec::new();

    if (namespace, package_id) == ("upstream-liuli", "taixu") {
        let mut idx = pack.vm_skills.len();
        crate::taixu_pack_attach::attach_taixu_pack(
            &mut seen_generals,
            &mut seen_cards,
            &mut seen_skills,
            &mut extra_generals,
            &mut extra_cards,
            &mut extra_host_skills,
            &mut extra_vm_skills,
            &mut idx,
        );
        pack.generals.extend(extra_generals);
        pack.cards.extend(extra_cards);
        pack.host_skills.extend(extra_host_skills);
        pack.vm_skills.extend(extra_vm_skills);
        return;
    }

    let mut push_general =
        |sym: &str, faction: Faction, gender: Gender, max_hp: u32, sk_list: &[&str], seen: &BTreeSet<String>| {
            if seen_generals.insert(sym.to_string()) {
                let mut interned: Vec<&'static str> = Vec::with_capacity(sk_list.len() + 2);
                for &s in sk_list {
                    let mapped = if seen.contains(s) || super::lookup_upstream_host_behavior(s).is_some() {
                        Some(names::intern(s.to_string()))
                    } else if let Some(zh) = super::lookup_standard_catalog_skill_zh(s) {
                        Some(zh)
                    } else {
                        None
                    };
                    if let Some(mapped) = mapped {
                        if !interned.contains(&mapped) {
                            interned.push(mapped);
                        }
                    }
                    if s == "zhucheng" && !interned.contains(&"zhucheng2") && seen.contains("zhucheng2") {
                        interned.push("zhucheng2");
                    }
                    if s == "fuchou" && !interned.contains(&"fuchou2") && seen.contains("fuchou2") {
                        interned.push("fuchou2");
                    }
                }
                extra_generals.push(ExtensionGeneral {
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
            let mut cdef = CardDef::basic(interned, kind).with_suit_rank(suit, rank);
            if matches!(cname, "ssr_yongsi_yuxi_card" | "ssr_yongsi") {
                cdef.equip_skills = vec!["ssr_yongsi_yuxi_card"];
            } else if kind == CardKind::WuxingPan {
                cdef.equip_skills = vec!["wuxingpan_skill"];
            } else if kind.equip_slot().is_some() {
                cdef.equip_skills = vec![interned];
            }
            extra_cards.push(cdef);
        }
    };

    let mut idx = pack.vm_skills.len();
    let mut push_skill = |name: &str, dispatch_id: u32| {
        if seen_skills.contains(name) {
            return;
        }
        if let Some(behavior) = super::lookup_upstream_host_behavior(name) {
            super::push_host_skill_if_unseen(name, behavior, &mut extra_host_skills, &mut seen_skills);
            return;
        }
        if seen_skills.insert(name.to_string()) {
            if let Some(decl) = make_vm_skill_decl(name, dispatch_id, idx) {
                extra_vm_skills.push(decl);
                idx += 1;
            }
        }
    };

    match (namespace, package_id) {
        ("upstream-noname", "d3-select") => {
            for &s in &U01_DDD_SKILLS {
                push_skill(s, DDD_SKILL_DISPATCH_ID);
            }
            for &(sym, faction, gender, max_hp, sk_list) in &U01_DDD_GENERALS {
                push_general(sym, faction, gender, max_hp, sk_list, &seen_skills);
            }
        }
        ("upstream-noname", "huanle-cards") => {
            for &s in &["monkey", "mianju", "kuwu", "xuelunyang", "jiuwei"] {
                push_skill(s, super::U06_HUANLE_SKILL_ID);
            }
        }
        ("upstream-noname", "wudian") => {
            if !pack.cards.iter().any(|c| c.name.contains("diaobingqianjiang")) {
                // 玩点论杀 card/list.js:25-26：红桃 4、方片 4 两张实体【调兵遣将】
                push_card("wd_diaobingqianjiang_1", CardKind::Wuzhong, Suit::Heart, 4);
                push_card("wd_diaobingqianjiang_2", CardKind::Wuzhong, Suit::Diamond, 4);
            }
            push_skill("diaobingqianjiang", super::DIAOBINGQIANJIANG_ID);
            push_skill("duoqi", super::DUOQI_ID);
            push_skill("youzhan", super::YOUZHAN_ID);
            push_skill("jianbi", WUDIAN_JIANBI_ID);
            push_skill("choudu", WUDIAN_CHOUDU_ID);
            for &s in &["zhucheng2", "wd_wangmeizhike", "wd_sanjian"] {
                push_skill(s, super::U07_WUDIAN_SKILL_ID);
            }
            for &s in &U07_WUDIAN_ALL_SKILLS {
                push_skill(s, super::U07_WUDIAN_SKILL_ID);
            }
            if !had_generals {
                for &(sym, faction, gender, max_hp, sk_list) in &U07_WUDIAN_GENERALS {
                    push_general(sym, faction, gender, max_hp, sk_list, &seen_skills);
                }
            }
            for g in pack.generals.iter_mut().chain(extra_generals.iter_mut()) {
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
            push_skill("yxs_menshen2_guard", super::YXS_MENSHEN2_GUARD_ID);
            for &s in &U08_YXS_SKILLS {
                push_skill(s, super::U08_YXS_SKILL_ID);
            }
            for &s in &["kanpo", "qiangxi"] {
                push_skill(s, super::U08_YXS_SKILL_ID);
            }
            for &(sym, faction, gender, max_hp, sk_list) in &U08_YXS_GENERALS {
                push_general(sym, faction, gender, max_hp, sk_list, &seen_skills);
            }
        }
        ("upstream-liuli", "ai-tuning") => {
            push_card("yiyi", CardKind::Wuzhong, Suit::Heart, 4);
            push_skill("yiyi", super::L01_YIYI_ID);
            for &s in &crate::L01_ALL_69_SKILLS {
                push_skill(s, super::L01_YIYI_ID);
            }
        }
        ("upstream-liuli", "wuxing") => {
            if !had_cards {
                push_card("wuxingpan", CardKind::WuxingPan, Suit::Spade, 5);
            }
            for &s in &["_shengke", "wuxingpan_skill", "wuxing"] {
                push_skill(s, super::L06_WUXING_ID);
            }
        }
        ("upstream-liuli", "epic-cards") => {
            push_skill("k_danggu", super::K_DANGGU_ID);
            push_skill("k_mowang", super::K_MOWANG_ID);
            for &s in &super::L10_SCS_COMPANION_SKILLS {
                push_skill(s, super::L10_SCS_SKILL_ID);
            }
            push_general(
                "shichangshi",
                Faction::Qun,
                Gender::Male,
                1,
                &["k_danggu", "k_mowang"],
                &seen_skills,
            );
        }
        ("upstream-liuli", "doomsday-brawl") => {
            for &s in &L19_DOOMSDAY_SKILLS {
                push_skill(s, L19_DOOMSDAY_SKILL_ID);
            }
            for &s in &["wansha", "paoxiao", "xueji"] {
                push_skill(s, L19_DOOMSDAY_SKILL_ID);
            }
            push_general(
                "jiangshifz",
                Faction::Qun,
                Gender::Male,
                5,
                &["xunmeng", "zaibian", "ganran", "wansha", "paoxiao"],
                &seen_skills,
            );
            push_general(
                "jiangshinj",
                Faction::Qun,
                Gender::Female,
                3,
                &["baozou", "wansha", "xueji", "shishi", "ganran"],
                &seen_skills,
            );
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
        ("upstream-liuli", "taixu") => {}
        ("upstream-liuli", "zuan") => {
            for &(cname, kind, suit, rank) in &super::L25_ZUAN_FORMAL_CARDS {
                push_card(cname, kind, suit, rank);
            }
            if seen_cards.insert("pyzhuren_diamond".to_string()) {
                extra_cards.push(
                    CardDef::weapon("pyzhuren_diamond", 2)
                        .with_suit_rank(Suit::Diamond, 1)
                        .with_equip_skills(&["pyzhuren_diamond"])
                        .with_off_deck(),
                );
            }
            for &s in super::L25_SSR_GENERAL_SKILLS
                .iter()
                .chain(super::L25_ALL_RULE_SKILLS.iter())
                .chain(["ssr_yongsi", "ssr_yongsi_yuxi_card"].iter())
                .chain(super::zuan_data::L25_BATCH_A_EXTRA_SKILLS.iter())
                .chain(L25_ZUAN_SKILLS.iter())
            {
                push_skill(s, L25_ZUAN_SKILL_ID);
            }
            // Mode & L25 `gongxin` routing applied after `match` drops `push_skill`.
            for &s in &[
                "pyzhuren_diamond",
                "ansha_zhong",
                "ansha_nei",
                "huanhaicls",
                "guanfu",
                "guifu_nei",
            ] {
                push_skill(s, L25_ZUAN_SKILL_ID);
            }
            for &(sym, faction, gender, max_hp, sk_list) in &super::L25_SSR_GENERALS {
                push_general(sym, faction, gender, max_hp, sk_list, &seen_skills);
            }
        }
        ("upstream-liuli", "select-beautify") => {
            for &s in &L27_SELECT_BEAUTIFY_SKILLS {
                if s != "mbcmqingzheng" {
                    push_skill(s, L27_SELECT_SKILL_ID);
                }
            }
            push_skill("_chooseCharacterTwo", L27_SELECT_SKILL_ID);
            super::push_host_skill_if_unseen(
                "mbcmqingzheng",
                engine_native::content::host::HostBehavior::L27MbcmQingzheng,
                &mut extra_host_skills,
                &mut seen_skills,
            );
        }
        ("upstream-liuli", "thunder") => {
            for card in super::l28_thunder_cards() {
                if seen_cards.insert(card.name.to_string()) {
                    extra_cards.push(card);
                }
            }
            push_skill("th_dunshi", super::TH_DUNSHI_ID);
            push_skill("th_jingxie", super::TH_JINGXIE_ID);
            push_skill("th_qiaosi", super::TH_QIAOSI_ID);
            push_skill("th_yeyan", super::TH_YEYAN_ID);
            push_skill("th_taipingyaoshu", super::TH_TAIPINGYAOSHU_ID);
            for &s in &super::L28_THUNDER_EXTRA_SKILLS {
                push_skill(s, super::TH_SKILL_DISPATCH_ID);
            }
            for &(sym, faction, gender, max_hp, sk_list) in &super::L28_THUNDER_GENERALS {
                push_general(sym, faction, gender, max_hp, sk_list, &seen_skills);
            }
        }
        ("upstream-liuli", "longxue") => {
            push_skill("mbcmjiushi", super::MBCMJIUSHI_ID);
            push_skill("mbcmfangzhu", super::MBCMFANGZHU_ID);
            push_skill("mbjuejin", super::MBJUEJIN_ID);
            for &s in &["mbcmqingzheng", "mbqianlong", "mb_fankui", "_CMeffects"] {
                push_skill(s, super::L29_LONGXUE_SKILL_ID);
            }
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
                &seen_skills,
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
                &seen_skills,
            );
        }
        ("upstream-liuli", "elo-sanction") => {
            for (idx, &(s, ev)) in engine_native::kernel::L22_ELO_SKILLS.iter().enumerate() {
                if !seen_skills.contains(s) {
                    extra_vm_skills.push(crate::elo::make_l22_vm_skill_decl(s, ev, idx));
                }
            }
        }
        ("upstream-noname", "boss-play")
        | ("upstream-liuli", "boss-play")
        | ("upstream-liuli", "fool-plugin") => {
            crate::q4_engine_bridge::append_q4_formal_rules(
                namespace, package_id, &mut seen_generals, &mut seen_skills,
                &mut extra_generals, &mut extra_host_skills,
            );
        }
        _ => {}
    }
    if (namespace, package_id) == ("upstream-liuli", "zuan") {
        crate::zuan_data::apply_zuan_post_overrides_raw(
            prior_packs,
            pack,
            &mut extra_vm_skills,
            &mut extra_host_skills,
            &mut seen_skills,
        );
    }
    pack.generals.extend(extra_generals);
    pack.cards.extend(extra_cards);
    pack.host_skills.extend(extra_host_skills);
    pack.vm_skills.extend(extra_vm_skills);
}
