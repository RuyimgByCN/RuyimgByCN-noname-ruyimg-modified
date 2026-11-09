//! Formal `packages/extensions/sha*-upstream-noname` wiring for `U05 杀海拾遗`
//! (`archive/upstream/noname/apps/core/extension/杀海拾遗`).
//! Attaches all 332 upstream generals (+ `stone_yisela`), 275 upstream cards, and 1,007 upstream
//! skills (+ cross-referenced skills) on the production `locked_content` path.

use std::collections::BTreeSet;

use engine_native::catalog::{CardDef, CardKind, EquipSlot, Faction, Suit};
use engine_native::content::extension::{ExtensionGeneral, ExtensionPack};
use engine_native::content::names;
use engine_native::content::standard::Gender;

use super::{
    make_vm_skill_decl, MTG_LAND_CARD_ID, OW_SKILL_DISPATCH_ID, SWD_DUIJUE_ID, SWD_TANLIN_ID,
    U05_GUJIAN_ALL_CARDS, U05_GUJIAN_ALL_SKILLS, U05_GUJIAN_GENERALS, U05_GWENT_ALL_CARDS,
    U05_GWENT_ALL_SKILLS, U05_GWENT_GENERALS, U05_HEARTH_ALL_CARDS, U05_HEARTH_ALL_SKILLS,
    U05_HEARTH_GENERALS, U05_MTG_ALL_CARDS, U05_MTG_ALL_SKILLS, U05_MTG_GENERALS,
    U05_OW_ALL_SKILLS, U05_OW_GENERALS, U05_STONE_GENERALS, U05_SWD_ALL_CARDS,
    U05_SWD_CARD_SKILLS, U05_SWD_CHAR_SKILLS, U05_SWD_GENERALS, U05_WUXING_ALL_CARDS,
    U05_WUXING_PLAY_SKILLS, U05_XIANJIAN_ALL_SKILLS, U05_XIANJIAN_GENERALS_FULL,
    U05_YUNCHOU_ALL_CARDS, U05_YUNCHOU_ALL_SKILLS, U05_YUNCHOU_GENERALS,
    U05_YUNCHOU_REMAINING_SKILLS, U05_ZHENFA_ALL_CARDS, XIANJIAN_SKILL_DISPATCH_ID,
    YUNCHOU_SKILL_ID, ZHENFA_POZHENJUE_ID,
};

pub const U05_SHAHAI_SKILL_ID: u32 = 1_926_002_010;

type GeneralRow = (&'static str, Faction, Gender, u32, &'static [&'static str]);
type CardRow = (&'static str, CardKind, u32, Suit, u8);

fn attach_card_rows(
    rows: &[CardRow],
    pack: &mut ExtensionPack,
    seen_cards: &mut BTreeSet<String>,
    seen_skills: &mut BTreeSet<String>,
    vm_idx: &mut usize,
    default_dispatch: u32,
) {
    for &(cname, kind, range, suit, rank) in rows {
        if seen_cards.insert(cname.to_string()) {
            let interned = names::intern(cname.to_string());
            let def = if kind.equip_slot() == Some(EquipSlot::Weapon) {
                CardDef::weapon(interned, range.max(1))
            } else {
                CardDef::basic(interned, kind)
            }
            .with_suit_rank(suit, rank);
            pack.cards.push(def);
        }
        let dispatch_id = match cname {
            "mtg_duzhao" | "mtg_cangbaohaiwan" => MTG_LAND_CARD_ID,
            "pozhenjue" => ZHENFA_POZHENJUE_ID,
            _ => default_dispatch,
        };
        if seen_skills.insert(cname.to_string()) {
            pack.vm_skills
                .push(make_vm_skill_decl(cname, dispatch_id, *vm_idx));
            *vm_idx += 1;
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

fn attach_skill_rows(
    package_id: &str,
    skills: &[&str],
    pack: &mut ExtensionPack,
    seen_skills: &mut BTreeSet<String>,
    vm_idx: &mut usize,
    default_dispatch: u32,
) {
    for &raw_sk in skills {
        let sk_owned = map_cross_subpack_skill(package_id, raw_sk);
        let dispatch_id = match raw_sk {
            "duijue" => SWD_DUIJUE_ID,
            "tanlin" => SWD_TANLIN_ID,
            "mtg_duzhao" | "g_mtg_duzhao" | "mtg_cangbaohaiwan" | "g_mtg_cangbaohaiwan" => {
                MTG_LAND_CARD_ID
            }
            "pozhenjue" | "g_pozhenjue" => ZHENFA_POZHENJUE_ID,
            _ => default_dispatch,
        };
        if seen_skills.insert(sk_owned.clone()) {
            pack.vm_skills
                .push(make_vm_skill_decl(&sk_owned, dispatch_id, *vm_idx));
            *vm_idx += 1;
        }
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
        let mapped_skills: Vec<String> = sk_list
            .iter()
            .map(|&s| map_cross_subpack_skill(package_id, s))
            .collect();
        let mapped_refs: Vec<&str> = mapped_skills.iter().map(String::as_str).collect();
        attach_skill_rows(package_id, &mapped_refs, pack, seen_skills, vm_idx, default_dispatch);
        if seen_generals.insert(sym.to_string()) {
            let interned: Vec<&'static str> = mapped_skills
                .into_iter()
                .map(names::intern)
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
    }
}

/// Attach authentic `U05 杀海拾遗` generals, cards, and VM skills when loading any of the 9
/// formal `sha*-upstream-noname` packages on the production `locked_content` path.
pub fn try_attach_shahai_formal_package(
    namespace: &str,
    package_id: &str,
    pack: &mut ExtensionPack,
    prior_packs: &[ExtensionPack],
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
    if !prior_packs.is_empty() {
        let mut prior_skills = BTreeSet::new();
        let mut prior_generals = BTreeSet::new();
        let mut prior_cards = BTreeSet::new();
        for p in prior_packs {
            for hs in &p.host_skills {
                prior_skills.insert(hs.name.to_string());
            }
            for vs in &p.vm_skills {
                prior_skills.insert(vs.name.clone());
            }
            for g in &p.generals {
                prior_generals.insert(g.name.to_string());
            }
            for c in &p.cards {
                prior_cards.insert(c.name.to_string());
            }
        }
        pack.host_skills
            .retain(|hs| !prior_skills.contains(hs.name));
        pack.vm_skills
            .retain(|vs| !prior_skills.contains(&vs.name));
        pack.generals
            .retain(|g| !prior_generals.contains(g.name));
        pack.cards.retain(|c| !prior_cards.contains(c.name));
    }
    let mut vm_idx = pack.vm_skills.len();
    match package_id {
        "shahai-gujian" => {
            attach_card_rows(
                &U05_GUJIAN_ALL_CARDS,
                pack,
                seen_cards,
                seen_skills,
                &mut vm_idx,
                U05_SHAHAI_SKILL_ID,
            );
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
            attach_card_rows(
                &U05_GWENT_ALL_CARDS,
                pack,
                seen_cards,
                seen_skills,
                &mut vm_idx,
                U05_SHAHAI_SKILL_ID,
            );
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
            attach_card_rows(
                &U05_HEARTH_ALL_CARDS,
                pack,
                seen_cards,
                seen_skills,
                &mut vm_idx,
                U05_SHAHAI_SKILL_ID,
            );
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
            attach_card_rows(
                &U05_MTG_ALL_CARDS,
                pack,
                seen_cards,
                seen_skills,
                &mut vm_idx,
                U05_SHAHAI_SKILL_ID,
            );
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
            attach_card_rows(
                &U05_SWD_ALL_CARDS,
                pack,
                seen_cards,
                seen_skills,
                &mut vm_idx,
                U05_SHAHAI_SKILL_ID,
            );
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
            attach_card_rows(
                &U05_YUNCHOU_ALL_CARDS,
                pack,
                seen_cards,
                seen_skills,
                &mut vm_idx,
                YUNCHOU_SKILL_ID,
            );
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
            attach_card_rows(
                &U05_ZHENFA_ALL_CARDS,
                pack,
                seen_cards,
                seen_skills,
                &mut vm_idx,
                U05_SHAHAI_SKILL_ID,
            );
            attach_card_rows(
                &U05_WUXING_ALL_CARDS,
                pack,
                seen_cards,
                seen_skills,
                &mut vm_idx,
                U05_SHAHAI_SKILL_ID,
            );
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
                &["pozhenjue", "g_pozhenjue"],
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
