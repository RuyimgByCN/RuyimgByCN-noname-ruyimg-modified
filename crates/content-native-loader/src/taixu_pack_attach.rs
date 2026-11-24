//! L11 (`upstream-liuli/taixu`) 正式扩展包挂载逻辑（从 `decision34_pack9.rs` 职责拆出）。

use std::collections::BTreeSet;

use engine_native::catalog::{CardDef, CardKind, Faction, Suit};
use engine_native::content::extension::{ExtensionGeneral, VmSkillDecl};
use engine_native::content::host::{HostBehavior, HostSkillDecl};
use engine_native::content::names;
use engine_native::content::standard::Gender;

use crate::decision34_pack9::{
    make_vm_skill_decl, DDD_SKILL_DISPATCH_ID, L11_TAIXU_SKILL_ID, L20_SKILL_DISPATCH_ID,
    L27_SELECT_SKILL_ID,
};

fn push_taixu_skill(
    name: &str,
    dispatch_id: u32,
    seen_skills: &mut BTreeSet<String>,
    extra_host_skills: &mut Vec<HostSkillDecl>,
    extra_vm_skills: &mut Vec<VmSkillDecl>,
    idx: &mut usize,
) {
    if seen_skills.contains(name) {
        return;
    }
    if let Some(behavior) = crate::lookup_upstream_host_behavior(name) {
        crate::push_host_skill_if_unseen(name, behavior, extra_host_skills, seen_skills);
        return;
    }
    if seen_skills.insert(name.to_string()) {
        if let Some(decl) = make_vm_skill_decl(name, dispatch_id, *idx) {
            extra_vm_skills.push(decl);
            *idx += 1;
        }
    }
}

fn push_taixu_general(
    sym: &str,
    faction: Faction,
    gender: Gender,
    max_hp: u32,
    sk_list: &[&str],
    seen_generals: &mut BTreeSet<String>,
    seen_skills: &BTreeSet<String>,
    extra_generals: &mut Vec<ExtensionGeneral>,
) {
    if seen_generals.insert(sym.to_string()) {
        let mut interned: Vec<&'static str> = Vec::with_capacity(sk_list.len() + 2);
        for &s in sk_list {
            let mapped =
                if seen_skills.contains(s) || crate::lookup_upstream_host_behavior(s).is_some() {
                    Some(names::intern(s.to_string()))
                } else {
                    crate::lookup_standard_catalog_skill_zh(s)
                };
            if let Some(mapped) = mapped {
                if !interned.contains(&mapped) {
                    interned.push(mapped);
                }
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
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn attach_taixu_pack(
    seen_generals: &mut BTreeSet<String>,
    seen_cards: &mut BTreeSet<String>,
    seen_skills: &mut BTreeSet<String>,
    extra_generals: &mut Vec<ExtensionGeneral>,
    extra_cards: &mut Vec<CardDef>,
    extra_host_skills: &mut Vec<HostSkillDecl>,
    extra_vm_skills: &mut Vec<VmSkillDecl>,
    idx: &mut usize,
) {
    for &(cname, kind, range, suit, rank) in &crate::L11_TAIXU_CARDS {
        if seen_cards.insert(cname.to_string()) {
            let interned = names::intern(cname.to_string());
            let mut card_def = if kind == CardKind::Weapon {
                CardDef::weapon(interned, range).with_suit_rank(suit, rank)
            } else {
                CardDef::basic(interned, kind).with_suit_rank(suit, rank)
            };
            if kind.equip_slot().is_some() {
                card_def.equip_skills = vec![interned];
            }
            extra_cards.push(card_def);
        }
    }

    crate::taixu_vm_fix_b::attach_l11_vm_fix_b_cards(extra_cards, seen_cards);

    // P3-D: L11 DLC / extension_card off-deck equips and shared L28 cards.
    for (cname, kind, range, suit, rank, sk_opt) in [
        ("txhj_zhenhunqin", CardKind::Weapon, 4u32, Suit::Diamond, 1u8, Some("txhj_zhenhunqin")),
        ("txhj_xieshenmianju", CardKind::Armor, 0u32, Suit::Spade, 2u8, Some("txhj_xieshenmianju")),
        ("txhj_shengguangbaiyi", CardKind::Armor, 0u32, Suit::Heart, 2u8, Some("txhj_shengguangbaiyi")),
        ("yajiaoqiang", CardKind::Weapon, 3u32, Suit::Diamond, 5u8, Some("yajiaoqiang_skill")),
        ("nvzhuang", CardKind::Armor, 0u32, Suit::Heart, 9u8, Some("nvzhuang")),
        ("yitianjian", CardKind::Weapon, 2u32, Suit::Club, 5u8, Some("yitianjian")),
        ("baiyin", CardKind::Armor, 0u32, Suit::Club, 1u8, None),
        ("tengjia", CardKind::Armor, 0u32, Suit::Spade, 2u8, None),
        ("lanyinjia", CardKind::Armor, 0u32, Suit::Spade, 2u8, None),
    ] {
        if seen_cards.insert(cname.to_string()) {
            let interned = names::intern(cname.to_string());
            let mut card_def = if kind == CardKind::Weapon {
                CardDef::weapon(interned, range).with_suit_rank(suit, rank)
            } else {
                CardDef::basic(interned, kind).with_suit_rank(suit, rank)
            };
            if let Some(sk) = sk_opt {
                card_def.equip_skills = vec![names::intern(sk.to_string())];
            }
            extra_cards.push(card_def.with_off_deck());
        }
    }
    for card in crate::l28_thunder_cards() {
        if seen_cards.insert(card.name.to_string()) {
            extra_cards.push(card);
        }
    }

    // P3-E1: strip-safe native VM Fix A bindings + `xiangle` (avoid U01 stub).
    crate::push_host_skill_if_unseen(
        "xiangle",
        HostBehavior::U01DddBatchD,
        extra_host_skills,
        seen_skills,
    );
    for &s in &crate::taixu_vm_fix_a::L11_TAIXU_VM_FIX_A_SKILLS {
        crate::push_host_skill_if_unseen(
            s,
            HostBehavior::L11VmFixA,
            extra_host_skills,
            seen_skills,
        );
    }
    crate::taixu_vm_fix_b::attach_l11_vm_fix_b_skills(extra_host_skills, seen_skills);
    crate::taixu_vm_fix_c::attach_taixu_vm_fix_c(extra_host_skills, seen_skills);

    // P3-D: DLC & extension_card equip skill bindings (`L11ModesEquips` + `L11Weapons`).
    for &s in &[
        "txhj_zhenhunqin",
        "txhj_xieshenmianju",
        "txhj_shengguangbaiyi",
        "yajiaoqiang_skill",
        "nvzhuang",
        "guilongzhanyuedao",
        "wuxinghelingshan_skill",
        "wushuangfangtianji_skill",
        "chixueqingfeng",
        "bintieshuangji_skill",
        "wutiesuolian_skill",
        "linglongshimandai_skill",
        "hongmianbaihuapao_skill",
        "qimenbagua",
        "huxinjing",
        "heiguangkai_skill",
        "shufazijinguan_skill",
        "xuwangzhimian",
        "tianjitu_skill",
        "taigongyinfu_skill",
        "taigongyinfu_link",
        "sanlve_skill",
        "zhaogujing_skill",
    ] {
        push_taixu_skill(
            s,
            L11_TAIXU_SKILL_ID,
            seen_skills,
            extra_host_skills,
            extra_vm_skills,
            idx,
        );
    }

    // P3-C1a: Part A 80 跨包 NPC 技 + 3 派生技 (`olhuoji`, `olkanpo`, `rexuanfeng`)
    // 统一绑定到原生行为组 `HostBehavior::L11CrossNpcA`；同时保证 `bazhen` / `reyingzi` 派生基础技可查。
    for &sk in crate::L11_TAIXU_IMPLEMENTED_CROSS_SKILLS[..80]
        .iter()
        .chain(["olhuoji", "olkanpo", "rexuanfeng"].iter())
    {
        crate::push_host_skill_if_unseen(
            sk,
            HostBehavior::L11CrossNpcA,
            extra_host_skills,
            seen_skills,
        );
    }
    crate::push_host_skill_if_unseen(
        "bazhen",
        HostBehavior::L11CrossNpcA,
        extra_host_skills,
        seen_skills,
    );
    crate::push_host_skill_if_unseen(
        "reyingzi",
        HostBehavior::SwdReyingzi,
        extra_host_skills,
        seen_skills,
    );

    for &s in crate::L11_TAIXU_SKILLS
        .iter()
        .chain(crate::L11_TAIXU_OWN_NPC_SKILLS.iter())
        .chain(crate::L11_TAIXU_EQUIP_CARD_SKILLS.iter())
        .chain(crate::L11_TAIXU_B1_AUX_SUBSKILLS.iter())
        .chain(crate::L11_TAIXU_IMPLEMENTED_CROSS_SKILLS[80..].iter())
    {
        if s != "xinfu_sidao"
            && !crate::taixu_pack_attach_b::L11_NPC_B_SKILLS.contains(&s)
            && !crate::taixu_vm_fix_c::is_stripped_vm_skill(s)
        {
            push_taixu_skill(
                s,
                L11_TAIXU_SKILL_ID,
                seen_skills,
                extra_host_skills,
                extra_vm_skills,
                idx,
            );
        }
    }

    for &s in &crate::L11_TAIXU_CROSS_SKILLS {
        if crate::taixu_pack_attach_b::L11_NPC_B_SKILLS.contains(&s)
            || crate::taixu_vm_fix_a::is_stripped_vm_skill(s)
            || crate::taixu_vm_fix_b::is_stripped_vm_skill(s)
            || crate::taixu_vm_fix_c::is_stripped_vm_skill(s)
        {
            continue;
        }
        if crate::lookup_upstream_host_behavior(s).is_some() {
            push_taixu_skill(
                s,
                L11_TAIXU_SKILL_ID,
                seen_skills,
                extra_host_skills,
                extra_vm_skills,
                idx,
            );
        } else if matches!(s, "dddfenji" | "dddjiexing" | "dddqiaoduan") {
            push_taixu_skill(
                s,
                DDD_SKILL_DISPATCH_ID,
                seen_skills,
                extra_host_skills,
                extra_vm_skills,
                idx,
            );
        } else if matches!(s, "chajue" | "yxs_fanji" | "qianglue" | "xiadao") {
            push_taixu_skill(
                s,
                crate::U08_YXS_SKILL_ID,
                seen_skills,
                extra_host_skills,
                extra_vm_skills,
                idx,
            );
        } else if s == "xueji" {
            // Handled immediately below via `HostBehavior::L11GeneralBossC`.
        } else if matches!(s, "kuanshi" | "xinfu_langxi" | "xinfu_lingren") {
            push_taixu_skill(
                s,
                L20_SKILL_DISPATCH_ID,
                seen_skills,
                extra_host_skills,
                extra_vm_skills,
                idx,
            );
        } else if matches!(s, "dcxiaoxi" | "rehuashen" | "twxiafeng") {
            push_taixu_skill(
                s,
                L27_SELECT_SKILL_ID,
                seen_skills,
                extra_host_skills,
                extra_vm_skills,
                idx,
            );
        } else if s == "feiying" {
            // Handled below as pure passive DistanceFromOthers modifier.
        } else if crate::shahai_attach::shahai_implemented_skills::is_implemented_u05_vm_skill(s)
            && crate::lookup_standard_catalog_skill_zh(s).is_none()
        {
            push_taixu_skill(
                s,
                crate::U05_SHAHAI_SKILL_ID,
                seen_skills,
                extra_host_skills,
                extra_vm_skills,
                idx,
            );
        }
    }

    crate::taixu_pack_attach_b::attach_taixu_native_extras(
        L11_TAIXU_SKILL_ID,
        extra_host_skills,
        extra_vm_skills,
        extra_cards,
        seen_skills,
        seen_cards,
    );

    for &(sym, faction, gender, max_hp, sk_list) in &crate::L11_TAIXU_GENERALS {
        push_taixu_general(
            sym,
            faction,
            gender,
            max_hp,
            sk_list,
            seen_generals,
            seen_skills,
            extra_generals,
        );
    }
    for &(sym, faction, gender, max_hp, sk_list) in &[
        (
            "lijue",
            Faction::Qun,
            Gender::Male,
            6u32,
            &["xinfu_langxi", "xinfu_yisuan"][..],
        ),
        (
            "guosi",
            Faction::Qun,
            Gender::Male,
            4u32,
            &["xinfu_tanbei", "xinfu_sidao"][..],
        ),
        (
            "zhangji",
            Faction::Qun,
            Gender::Male,
            4u32,
            &["xinfu_lveming", "xinfu_tunjun"][..],
        ),
    ] {
        push_taixu_general(
            sym,
            faction,
            gender,
            max_hp,
            sk_list,
            seen_generals,
            seen_skills,
            extra_generals,
        );
    }
}
