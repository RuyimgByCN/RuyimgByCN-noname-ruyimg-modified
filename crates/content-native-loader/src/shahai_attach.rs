//! Formal `packages/extensions/sha*-upstream-noname` wiring for `U05 杀海拾遗`
//! (`archive/upstream/noname/apps/core/extension/杀海拾遗`).
//!
//! Attaches upstream generals, non-duplicate cards (with `card_kind_equip_skills` &
//! `card_kind_host_skills`, never pushing fake `VmSkillDecl`s for card names), native
//! `HostSkillDecl`s (`lookup_upstream_host_behavior`), `standard_catalog` skill bindings, and
//! verified `U05` VM skills (`is_implemented_u05_vm_skill`). Unported skills are omitted from
//! `Catalog` and `ExtensionGeneral.skills` so zero dead-flag stubs exist.

use std::collections::BTreeSet;

use engine_native::catalog::{CardDef, CardKind, EquipSlot, Faction, Suit};
use engine_native::content::extension::{ExtensionGeneral, ExtensionPack};
use engine_native::content::host::card_kind_equip_skills;
use engine_native::content::names;
use engine_native::content::standard::Gender;

use super::{
    lookup_standard_catalog_skill_zh, lookup_upstream_host_behavior, make_vm_skill_decl,
    push_host_skill_if_unseen, MTG_LAND_CARD_ID, OW_SKILL_DISPATCH_ID, SWD_DUIJUE_ID,
    SWD_TANLIN_ID, U05_GUJIAN_ALL_CARDS, U05_GUJIAN_ALL_SKILLS, U05_GUJIAN_GENERALS,
    U05_GWENT_ALL_CARDS, U05_GWENT_ALL_SKILLS, U05_GWENT_GENERALS, U05_HEARTH_ALL_CARDS,
    U05_HEARTH_ALL_SKILLS, U05_HEARTH_GENERALS, U05_MTG_ALL_CARDS, U05_MTG_ALL_SKILLS,
    U05_MTG_GENERALS, U05_OW_ALL_SKILLS, U05_OW_GENERALS, U05_STONE_GENERALS, U05_SWD_ALL_CARDS,
    U05_SWD_CARD_SKILLS, U05_SWD_CHAR_SKILLS, U05_SWD_GENERALS, U05_WUXING_ALL_CARDS,
    U05_WUXING_PLAY_SKILLS, U05_XIANJIAN_ALL_SKILLS, U05_XIANJIAN_GENERALS_FULL,
    U05_YUNCHOU_ALL_CARDS, U05_YUNCHOU_ALL_SKILLS, U05_YUNCHOU_GENERALS,
    U05_YUNCHOU_REMAINING_SKILLS, U05_ZHENFA_ALL_CARDS, XIANJIAN_SKILL_DISPATCH_ID,
    YUNCHOU_SKILL_ID, ZHENFA_POZHENJUE_ID,
};

#[path = "shahai_implemented_skills.rs"]
pub(crate) mod shahai_implemented_skills;

pub const U05_SHAHAI_SKILL_ID: u32 = 1_926_002_010;

type GeneralRow = (&'static str, Faction, Gender, u32, &'static [&'static str]);
type CardRow = (&'static str, CardKind, u32, Suit, u8);

fn is_already_bridged_in_cards_json(
    package_id: &str,
    cname: &str,
    seen_cards: &BTreeSet<String>,
) -> bool {
    if seen_cards.contains(cname) {
        return true;
    }
    let prefix = match package_id {
        "shahai-gujian" => "gj_",
        "shahai-gwent" => "gw_",
        "shahai-hearth" => "ht_",
        "shahai-mtg" => "mtg_",
        "shahai-swd" => "swd_",
        "shahai-yunchou" => "yc_",
        "shahai-zhenfa" => "zf_",
        _ => "",
    };
    let stripped = cname.strip_prefix("gw_").unwrap_or(cname);
    seen_cards.iter().any(|existing| {
        let base = match existing.rsplit_once('_') {
            Some((stem, idx)) if !idx.is_empty() && idx.bytes().all(|b| b.is_ascii_digit()) => stem,
            _ => existing.as_str(),
        };
        base == cname
            || (!prefix.is_empty() && base == format!("{prefix}{cname}"))
            || (!prefix.is_empty() && base == format!("{prefix}{stripped}"))
            || base.starts_with(&format!("swd_{cname}"))
            || base == format!("zf_{cname}")
    })
}

fn attach_card_rows(
    package_id: &str,
    rows: &[CardRow],
    pack: &mut ExtensionPack,
    seen_cards: &mut BTreeSet<String>,
) {
    for &(cname, kind, range, suit, rank) in rows {
        if is_already_bridged_in_cards_json(package_id, cname, seen_cards) {
            continue;
        }
        if seen_cards.insert(cname.to_string()) {
            let interned = names::intern(cname.to_string());
            let mut def = if kind.equip_slot() == Some(EquipSlot::Weapon) {
                CardDef::weapon(interned, range.max(1))
            } else {
                CardDef::basic(interned, kind)
            }
            .with_suit_rank(suit, rank);
            def.equip_skills = card_kind_equip_skills(kind).to_vec();
            pack.cards.push(def);
        }
    }
}

fn map_cross_subpack_skill(package_id: &str, sk: &str) -> String {
    if sk.contains("__ref_") {
        return sk.to_string();
    }
    if (package_id != "shahai-swd" && matches!(sk, "busi" | "huopu" | "luanji" | "dangping"))
        || (package_id != "shaai-xianjian" && sk == "xuanning")
    {
        format!("{package_id}__ref_{sk}")
    } else {
        sk.to_string()
    }
}

fn resolve_and_register_u05_skill(
    package_id: &str,
    raw_sk: &str,
    pack: &mut ExtensionPack,
    seen_skills: &mut BTreeSet<String>,
    vm_idx: &mut usize,
    default_dispatch: u32,
) -> Option<&'static str> {
    let clean = raw_sk
        .rsplit_once("__ref_")
        .map(|(_, s)| s)
        .unwrap_or(raw_sk);
    if let Some(zh) = lookup_standard_catalog_skill_zh(clean) {
        return Some(zh);
    }
    let sk_owned = map_cross_subpack_skill(package_id, raw_sk);
    if let Some(behavior) = lookup_upstream_host_behavior(clean) {
        push_host_skill_if_unseen(&sk_owned, behavior, &mut pack.host_skills, seen_skills);
        return Some(names::intern(sk_owned));
    }
    if !shahai_implemented_skills::is_implemented_u05_vm_skill(&sk_owned) {
        return None;
    }
    let dispatch_id = match clean {
        "duijue" => SWD_DUIJUE_ID,
        "tanlin" => SWD_TANLIN_ID,
        "mtg_duzhao" | "g_mtg_duzhao" | "mtg_cangbaohaiwan" | "g_mtg_cangbaohaiwan" => {
            MTG_LAND_CARD_ID
        }
        "pozhenjue" | "g_pozhenjue" => ZHENFA_POZHENJUE_ID,
        _ => default_dispatch,
    };
    if seen_skills.insert(sk_owned.clone()) {
        if let Some(decl) = make_vm_skill_decl(&sk_owned, dispatch_id, *vm_idx) {
            pack.vm_skills.push(decl);
            *vm_idx += 1;
        }
    }
    Some(names::intern(sk_owned))
}

fn attach_skill_rows(
    package_id: &str,
    skills: &[&str],
    pack: &mut ExtensionPack,
    seen_skills: &mut BTreeSet<String>,
    vm_idx: &mut usize,
    default_dispatch: u32,
) {
    for &raw_sk in skills {
        let _ = resolve_and_register_u05_skill(
            package_id,
            raw_sk,
            pack,
            seen_skills,
            vm_idx,
            default_dispatch,
        );
    }
}

fn attach_general_rows(
    package_id: &str,
    rows: &[GeneralRow],
    pack: &mut ExtensionPack,
    seen_generals: &mut BTreeSet<String>,
    seen_skills: &mut BTreeSet<String>,
    vm_idx: &mut usize,
    default_dispatch: u32,
) {
    for &(sym, faction, gender, max_hp, sk_list) in rows {
        let mut resolved: Vec<&'static str> = Vec::with_capacity(sk_list.len());
        for &raw_sk in sk_list {
            if let Some(interned) = resolve_and_register_u05_skill(
                package_id,
                raw_sk,
                pack,
                seen_skills,
                vm_idx,
                default_dispatch,
            ) {
                if !resolved.contains(&interned) {
                    resolved.push(interned);
                }
            }
        }
        if seen_generals.insert(sym.to_string()) {
            pack.generals.push(ExtensionGeneral {
                name: names::intern(sym.to_string()),
                faction,
                skills: names::intern_list(resolved),
                max_hp,
                gender,
                presentation_binding: None,
            });
        }
    }
}

/// Attach authentic `U05 杀海拾遗` generals, non-duplicate cards, native `HostSkillDecl`s,
/// and verified VM skills when loading any of the 9 formal `sha*-upstream-noname` packages.
pub fn try_attach_shahai_formal_package(
    namespace: &str,
    package_id: &str,
    pack: &mut ExtensionPack,
    _prior_packs: &[ExtensionPack],
    seen_generals: &mut BTreeSet<String>,
    seen_cards: &mut BTreeSet<String>,
    seen_skills: &mut BTreeSet<String>,
) -> bool {
    if namespace != "upstream-noname"
        || !matches!(
            package_id,
            "shahai-gujian"
                | "shahai-gwent"
                | "shahai-hearth"
                | "shahai-mtg"
                | "shahai-swd"
                | "shahai-yunchou"
                | "shahai-zhenfa"
                | "shaai-ow"
                | "shaai-xianjian"
        )
    {
        return false;
    }
    let mut vm_idx = pack.vm_skills.len();
    match package_id {
        "shahai-gujian" => {
            attach_card_rows(package_id, &U05_GUJIAN_ALL_CARDS, pack, seen_cards);
            attach_skill_rows(
                package_id,
                &U05_GUJIAN_ALL_SKILLS,
                pack,
                seen_skills,
                &mut vm_idx,
                U05_SHAHAI_SKILL_ID,
            );
            attach_general_rows(
                package_id,
                &U05_GUJIAN_GENERALS,
                pack,
                seen_generals,
                seen_skills,
                &mut vm_idx,
                U05_SHAHAI_SKILL_ID,
            );
            true
        }
        "shahai-gwent" => {
            attach_card_rows(package_id, &U05_GWENT_ALL_CARDS, pack, seen_cards);
            attach_skill_rows(
                package_id,
                &U05_GWENT_ALL_SKILLS,
                pack,
                seen_skills,
                &mut vm_idx,
                U05_SHAHAI_SKILL_ID,
            );
            attach_general_rows(
                package_id,
                &U05_GWENT_GENERALS,
                pack,
                seen_generals,
                seen_skills,
                &mut vm_idx,
                U05_SHAHAI_SKILL_ID,
            );
            true
        }
        "shahai-hearth" => {
            attach_card_rows(package_id, &U05_HEARTH_ALL_CARDS, pack, seen_cards);
            attach_skill_rows(
                package_id,
                &U05_HEARTH_ALL_SKILLS,
                pack,
                seen_skills,
                &mut vm_idx,
                U05_SHAHAI_SKILL_ID,
            );
            attach_general_rows(
                package_id,
                &U05_HEARTH_GENERALS,
                pack,
                seen_generals,
                seen_skills,
                &mut vm_idx,
                U05_SHAHAI_SKILL_ID,
            );
            attach_general_rows(
                package_id,
                &U05_STONE_GENERALS,
                pack,
                seen_generals,
                seen_skills,
                &mut vm_idx,
                U05_SHAHAI_SKILL_ID,
            );
            true
        }
        "shahai-mtg" => {
            attach_card_rows(package_id, &U05_MTG_ALL_CARDS, pack, seen_cards);
            attach_skill_rows(
                package_id,
                &U05_MTG_ALL_SKILLS,
                pack,
                seen_skills,
                &mut vm_idx,
                U05_SHAHAI_SKILL_ID,
            );
            attach_general_rows(
                package_id,
                &U05_MTG_GENERALS,
                pack,
                seen_generals,
                seen_skills,
                &mut vm_idx,
                U05_SHAHAI_SKILL_ID,
            );
            true
        }
        "shahai-swd" => {
            attach_card_rows(package_id, &U05_SWD_ALL_CARDS, pack, seen_cards);
            attach_skill_rows(
                package_id,
                &U05_SWD_CARD_SKILLS,
                pack,
                seen_skills,
                &mut vm_idx,
                U05_SHAHAI_SKILL_ID,
            );
            attach_skill_rows(
                package_id,
                &U05_SWD_CHAR_SKILLS,
                pack,
                seen_skills,
                &mut vm_idx,
                U05_SHAHAI_SKILL_ID,
            );
            attach_general_rows(
                package_id,
                &U05_SWD_GENERALS,
                pack,
                seen_generals,
                seen_skills,
                &mut vm_idx,
                U05_SHAHAI_SKILL_ID,
            );
            true
        }
        "shahai-yunchou" => {
            attach_card_rows(package_id, &U05_YUNCHOU_ALL_CARDS, pack, seen_cards);
            attach_skill_rows(
                package_id,
                &U05_YUNCHOU_REMAINING_SKILLS,
                pack,
                seen_skills,
                &mut vm_idx,
                YUNCHOU_SKILL_ID,
            );
            attach_skill_rows(
                package_id,
                &U05_YUNCHOU_ALL_SKILLS,
                pack,
                seen_skills,
                &mut vm_idx,
                YUNCHOU_SKILL_ID,
            );
            attach_general_rows(
                package_id,
                &U05_YUNCHOU_GENERALS,
                pack,
                seen_generals,
                seen_skills,
                &mut vm_idx,
                YUNCHOU_SKILL_ID,
            );
            true
        }
        "shahai-zhenfa" => {
            attach_card_rows(package_id, &U05_ZHENFA_ALL_CARDS, pack, seen_cards);
            attach_card_rows(package_id, &U05_WUXING_ALL_CARDS, pack, seen_cards);
            attach_skill_rows(
                package_id,
                &U05_WUXING_PLAY_SKILLS,
                pack,
                seen_skills,
                &mut vm_idx,
                U05_SHAHAI_SKILL_ID,
            );
            attach_skill_rows(
                package_id,
                &["g_pozhenjue"],
                pack,
                seen_skills,
                &mut vm_idx,
                ZHENFA_POZHENJUE_ID,
            );
            true
        }
        "shaai-ow" => {
            attach_skill_rows(
                package_id,
                &U05_OW_ALL_SKILLS,
                pack,
                seen_skills,
                &mut vm_idx,
                OW_SKILL_DISPATCH_ID,
            );
            attach_general_rows(
                package_id,
                &U05_OW_GENERALS,
                pack,
                seen_generals,
                seen_skills,
                &mut vm_idx,
                OW_SKILL_DISPATCH_ID,
            );
            true
        }
        "shaai-xianjian" => {
            attach_skill_rows(
                package_id,
                &U05_XIANJIAN_ALL_SKILLS,
                pack,
                seen_skills,
                &mut vm_idx,
                XIANJIAN_SKILL_DISPATCH_ID,
            );
            attach_general_rows(
                package_id,
                &U05_XIANJIAN_GENERALS_FULL,
                pack,
                seen_generals,
                seen_skills,
                &mut vm_idx,
                XIANJIAN_SKILL_DISPATCH_ID,
            );
            true
        }
        _ => false,
    }
}
