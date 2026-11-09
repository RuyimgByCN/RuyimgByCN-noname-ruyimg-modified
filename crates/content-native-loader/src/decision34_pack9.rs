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

fn classify_skill_trigger(name: &str) -> (EventKind, VmFilterPolicy) {
    match name {
        "ycndiyjianxiong"
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
        | "_pfqh_check_phaseEnd"
        | "_jd_hhjs"
        | "_jd_phaseAfter"
        | "_phaseendskill" => (EventKind::PhaseEnd, VmFilterPolicy::OwnerIsTarget),
        "_decadeUI_dieKillEffect"
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
        "_zuanidentitydyingskill" | "_bingleDying" => {
            (EventKind::Dying, VmFilterPolicy::OwnerIsTarget)
        }
        "_pfqh_check_roundCount" | "_zuanRoundTX" | "_chooseCharacterTwo" => {
            (EventKind::TurnBegin, VmFilterPolicy::OwnerIsTarget)
        }
        _ => (EventKind::PhaseBegin, VmFilterPolicy::OwnerIsTarget),
    }
}

fn make_vm_skill_decl(name_str: &str, dispatch_id: u32, idx: usize) -> VmSkillDecl {
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
    let extra_events = if event == EventKind::PhaseBegin {
        Vec::new()
    } else {
        vec![EventKind::PhaseBegin]
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
/// `STANDARD_GENERALS` so multi-pack coexistence never collides.
pub fn decision34_pack9_for_coexistence(existing_packs: &[&ExtensionPack]) -> ExtensionPack {
    let mut seen_generals: BTreeSet<String> = STANDARD_GENERALS
        .iter()
        .map(|g| g.name.to_string())
        .collect();
    let mut seen_cards: BTreeSet<String> = BTreeSet::new();
    let mut seen_skills: BTreeSet<String> = BTreeSet::new();

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

    // U01: 31 `ddd_*` generals
    for &(sym, faction, gender, max_hp, sk_list) in &U01_DDD_GENERALS {
        push_general(sym, faction, gender, max_hp, sk_list);
    }
    // U05 Xianjian: all 36 `pal_*` generals (covering the 13 full + 23 partial generals)
    for &(sym, faction, gender, max_hp, sk_list) in &U05_XIANJIAN_GENERALS {
        push_general(sym, faction, gender, max_hp, sk_list);
    }
    // L08: 9 Decade UI characters
    for &sym in &L08_DECADE_CHARACTERS {
        push_general(sym, Faction::Qun, Gender::Male, 4, &["_usecard"]);
    }
    // L14: 3 Shousha UI characters
    for &sym in &L14_SHOUSHA_CHARACTERS {
        push_general(sym, Faction::Wei, Gender::Male, 4, &["_dieNoTuoguan", "_jindutiao"]);
    }
    // L19: 24 Doomsday Brawl characters (starting with `jiangshifz`, `jiangshinj`)
    for &sym in &L19_DOOMSDAY_CHARACTERS {
        push_general(sym, Faction::Qun, Gender::Male, 4, &["baozou", "ganran"]);
    }
    // L20: 12 Mark Supplement characters
    for &sym in &L20_MARK_SUPPLEMENT_CHARACTERS {
        push_general(sym, Faction::Shu, Gender::Male, 4, &["zhengsu", "zhenyi_spade"]);
    }
    // L25: 120 Zuan characters (60 `ssr_*` + 60 `zuan_*`)
    for &sym in &L25_ZUAN_CHARACTERS {
        push_general(sym, Faction::Qun, Gender::Male, 4, &["g_du", "_diycharacterskill"]);
    }

    // Cards: U05 MTG (2), U05 Zhenfa (1), L14 (1), L20 (3 `zhengsu_*` + 3 tactical cards)
    let mut cards: Vec<CardDef> = Vec::new();
    let card_specs: [(&str, CardKind, Suit, u8); 10] = [
        ("mtg_duzhao", CardKind::Wuzhong, Suit::Spade, 1),
        ("mtg_cangbaohaiwan", CardKind::Wuzhong, Suit::Heart, 1),
        ("pozhenjue", CardKind::Wuzhong, Suit::Club, 7),
        ("shousha_qiankunbagua", CardKind::Wuzhong, Suit::Spade, 2),
        ("zhengsu_leijin", CardKind::Wuzhong, Suit::Spade, 3),
        ("zhengsu_mingzhi", CardKind::Wuzhong, Suit::Heart, 4),
        ("zhengsu_bianzhen", CardKind::Wuzhong, Suit::Club, 5),
        ("diaohulishan", CardKind::Wuzhong, Suit::Diamond, 10),
        ("huoshaolianying", CardKind::Wuzhong, Suit::Heart, 12),
        ("lulitongxin", CardKind::Wuzhong, Suit::Club, 11),
    ];
    for &(cname, kind, suit, rank) in &card_specs {
        if seen_cards.insert(cname.to_string()) {
            let interned = names::intern(cname.to_string());
            cards.push(CardDef::basic(interned, kind).with_suit_rank(suit, rank));
        }
    }

    // Skills across all 9 sources + U07
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
    push_skill("jianbi", WUDIAN_JIANBI_ID);
    push_skill("choudu", WUDIAN_CHOUDU_ID);
    for &s in &L08_DECADE_SKILLS {
        push_skill(s, L08_DECADE_SKILL_ID);
    }
    for &s in &["_chongzhu", "_decade_qiankunbagua"] {
        push_skill(s, L08_DECADE_SKILL_ID);
    }
    for &s in &L14_SHOUSHA_SKILLS {
        push_skill(s, L14_SHOUSHA_SKILL_ID);
    }
    for &s in &["spshidi", "spcishi", "zhefu", "yidu"] {
        push_skill(s, L14_SHOUSHA_SKILL_ID);
    }
    for &s in &L19_DOOMSDAY_SKILLS {
        push_skill(s, L19_DOOMSDAY_SKILL_ID);
    }
    for &s in &L20_MARK_SUPPLEMENT_SKILLS {
        push_skill(s, L20_SKILL_DISPATCH_ID);
    }
    for &s in &L24_SKIN_SWITCH_SKILLS {
        push_skill(s, L24_SKIN_SKILL_ID);
    }
    push_skill("gx_Bozhi", L24_SKIN_SKILL_ID);
    for &s in &L25_ZUAN_SKILLS {
        push_skill(s, L25_ZUAN_SKILL_ID);
    }
    push_skill("zuan_jianxiong", L25_ZUAN_SKILL_ID);
    for &s in &L27_SELECT_BEAUTIFY_SKILLS {
        push_skill(s, L27_SELECT_SKILL_ID);
    }
    push_skill("_chooseCharacterTwo", L27_SELECT_SKILL_ID);

    ExtensionPack {
        def: ExtensionDef {
            name: "decision34-pack9-closure",
            version: 1,
            description:
                "Decision 34 Batch 26 complete closure for U01/U05/U07/L08/L14/L19/L20/L24/L25/L27",
        },
        generals,
        cards,
        host_skills: Vec::new(),
        vm_skills,
        init_hook: InitHook::None,
        enabled: true,
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
