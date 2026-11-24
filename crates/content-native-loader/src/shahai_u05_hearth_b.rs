//! Batch U05-6b `杀海拾遗` `hearth` Part 2 加载器映射与供应扩展（`shahai-hearth`）。

use std::collections::BTreeSet;

use engine_native::catalog::{CardDef, CardKind, Suit};
use engine_native::content::extension::ExtensionPack;
use engine_native::content::host::{HostBehavior, HostSkillDecl};
use engine_native::content::names;

/// Batch U05-6b 全部 107 项清单技能（含 `stone_chenshui`；`hsmengjing_mengye` 已在 U05-6a）
/// + 4 项 companion 子技（`qianghua2` / `shengyan2` / `fengxing2` / `bianxing2`），共 111 项，tag `1250..=1255`。
pub(crate) const HEARTH_B6B_SKILL_MAP: &[(&str, HostBehavior)] = &[
    // Part 2 Batch A (tag 1250): `xieneng` .. `hhudun` (17)
    ("xieneng", HostBehavior::U05Hearth2BatchA),
    ("fbeifa", HostBehavior::U05Hearth2BatchA),
    ("oldfbeifa", HostBehavior::U05Hearth2BatchA),
    ("yufa", HostBehavior::U05Hearth2BatchA),
    ("yufa2", HostBehavior::U05Hearth2BatchA),
    ("yufa3", HostBehavior::U05Hearth2BatchA),
    ("bingyan", HostBehavior::U05Hearth2BatchA),
    ("shifa", HostBehavior::U05Hearth2BatchA),
    ("oldshifa", HostBehavior::U05Hearth2BatchA),
    ("yuanzheng", HostBehavior::U05Hearth2BatchA),
    ("yulu", HostBehavior::U05Hearth2BatchA),
    ("oldyulu", HostBehavior::U05Hearth2BatchA),
    ("duzhang", HostBehavior::U05Hearth2BatchA),
    ("duzhang2", HostBehavior::U05Hearth2BatchA),
    ("hannu", HostBehavior::U05Hearth2BatchA),
    ("chuidiao", HostBehavior::U05Hearth2BatchA),
    ("hhudun", HostBehavior::U05Hearth2BatchA),
    // Part 2 Batch B (tag 1251): `fenlie` .. `oldmalymowang` (25)
    ("fenlie", HostBehavior::U05Hearth2BatchB),
    ("oldfenlie", HostBehavior::U05Hearth2BatchB),
    ("nianfu", HostBehavior::U05Hearth2BatchB),
    ("xiaorong", HostBehavior::U05Hearth2BatchB),
    ("shixu", HostBehavior::U05Hearth2BatchB),
    ("jixuan", HostBehavior::U05Hearth2BatchB),
    ("qianghua", HostBehavior::U05Hearth2BatchB),
    ("qianghua2", HostBehavior::U05Hearth2BatchB),
    ("biri", HostBehavior::U05Hearth2BatchB),
    ("stuxi", HostBehavior::U05Hearth2BatchB),
    ("stuxi2", HostBehavior::U05Hearth2BatchB),
    ("bingdong", HostBehavior::U05Hearth2BatchB),
    ("luoshi", HostBehavior::U05Hearth2BatchB),
    ("ronghuo", HostBehavior::U05Hearth2BatchB),
    ("ronghuo2", HostBehavior::U05Hearth2BatchB),
    ("fushi", HostBehavior::U05Hearth2BatchB),
    ("oldfushi", HostBehavior::U05Hearth2BatchB),
    ("moyao", HostBehavior::U05Hearth2BatchB),
    ("jiaohui", HostBehavior::U05Hearth2BatchB),
    ("bimeng", HostBehavior::U05Hearth2BatchB),
    ("zhoujiang", HostBehavior::U05Hearth2BatchB),
    ("liehun", HostBehavior::U05Hearth2BatchB),
    ("xjumo", HostBehavior::U05Hearth2BatchB),
    ("malymowang", HostBehavior::U05Hearth2BatchB),
    ("oldmalymowang", HostBehavior::U05Hearth2BatchB),
    // Part 2 Batch C (tag 1252): `lingzhou` .. `shengdun` (18)
    ("lingzhou", HostBehavior::U05Hearth2BatchC),
    ("mieshi", HostBehavior::U05Hearth2BatchC),
    ("xmojian", HostBehavior::U05Hearth2BatchC),
    ("xshixin", HostBehavior::U05Hearth2BatchC),
    ("enze", HostBehavior::U05Hearth2BatchC),
    ("oldenze", HostBehavior::U05Hearth2BatchC),
    ("chongsheng", HostBehavior::U05Hearth2BatchC),
    ("guozai", HostBehavior::U05Hearth2BatchC),
    ("guozai2", HostBehavior::U05Hearth2BatchC),
    ("guozaix", HostBehavior::U05Hearth2BatchC),
    ("guozaix2", HostBehavior::U05Hearth2BatchC),
    ("hanshuang", HostBehavior::U05Hearth2BatchC),
    ("oldhanshuang", HostBehavior::U05Hearth2BatchC),
    ("bingshi", HostBehavior::U05Hearth2BatchC),
    ("bingshi2", HostBehavior::U05Hearth2BatchC),
    ("huanwu", HostBehavior::U05Hearth2BatchC),
    ("fengnu", HostBehavior::U05Hearth2BatchC),
    ("shengdun", HostBehavior::U05Hearth2BatchC),
    // Part 2 Batch D (tag 1253): `jingmeng` .. `anying` (20)
    ("jingmeng", HostBehavior::U05Hearth2BatchD),
    ("hswuji", HostBehavior::U05Hearth2BatchD),
    ("bingshuang", HostBehavior::U05Hearth2BatchD),
    ("yanshu", HostBehavior::U05Hearth2BatchD),
    ("oldyanshu", HostBehavior::U05Hearth2BatchD),
    ("shengyan", HostBehavior::U05Hearth2BatchD),
    ("shengyan2", HostBehavior::U05Hearth2BatchD),
    ("liechao", HostBehavior::U05Hearth2BatchD),
    ("qingliu", HostBehavior::U05Hearth2BatchD),
    ("zhongjia", HostBehavior::U05Hearth2BatchD),
    ("dunji", HostBehavior::U05Hearth2BatchD),
    ("fengxing", HostBehavior::U05Hearth2BatchD),
    ("fengxing2", HostBehavior::U05Hearth2BatchD),
    ("fengxian", HostBehavior::U05Hearth2BatchD),
    ("zhanhou", HostBehavior::U05Hearth2BatchD),
    ("oldzhanhou", HostBehavior::U05Hearth2BatchD),
    ("shijie", HostBehavior::U05Hearth2BatchD),
    ("shengguang", HostBehavior::U05Hearth2BatchD),
    ("xinci", HostBehavior::U05Hearth2BatchD),
    ("anying", HostBehavior::U05Hearth2BatchD),
    // Part 2 Batch E (tag 1254): `huanjue` .. `jihuo` + `stone_chenshui` (15)
    ("huanjue", HostBehavior::U05Hearth2BatchE),
    ("bingjia", HostBehavior::U05Hearth2BatchE),
    ("bingjia2", HostBehavior::U05Hearth2BatchE),
    ("mianyi", HostBehavior::U05Hearth2BatchE),
    ("bianxing2", HostBehavior::U05Hearth2BatchE),
    ("zuzhou", HostBehavior::U05Hearth2BatchE),
    ("mdzhoufu", HostBehavior::U05Hearth2BatchE),
    ("mdzhoufu2", HostBehavior::U05Hearth2BatchE),
    ("xianzhi", HostBehavior::U05Hearth2BatchE),
    ("jingxiang", HostBehavior::U05Hearth2BatchE),
    ("wlianji", HostBehavior::U05Hearth2BatchE),
    ("mengun", HostBehavior::U05Hearth2BatchE),
    ("mengun2", HostBehavior::U05Hearth2BatchE),
    ("jianren", HostBehavior::U05Hearth2BatchE),
    ("jihuo", HostBehavior::U05Hearth2BatchE),
    ("stone_chenshui", HostBehavior::U05Hearth2BatchE),
    // Part 2 Batch F (tag 1255): `tuteng_s` .. `hongxi` (16)
    ("tuteng_s", HostBehavior::U05Hearth2BatchF),
    ("s_tuteng", HostBehavior::U05Hearth2BatchF),
    ("tuteng", HostBehavior::U05Hearth2BatchF),
    ("zuling", HostBehavior::U05Hearth2BatchF),
    ("huanfeng", HostBehavior::U05Hearth2BatchF),
    ("tuteng_h", HostBehavior::U05Hearth2BatchF),
    ("tuteng_lose", HostBehavior::U05Hearth2BatchF),
    ("tuteng1", HostBehavior::U05Hearth2BatchF),
    ("tuteng2", HostBehavior::U05Hearth2BatchF),
    ("tuteng3", HostBehavior::U05Hearth2BatchF),
    ("tuteng4", HostBehavior::U05Hearth2BatchF),
    ("tuteng5", HostBehavior::U05Hearth2BatchF),
    ("tuteng6", HostBehavior::U05Hearth2BatchF),
    ("tuteng7", HostBehavior::U05Hearth2BatchF),
    ("tuteng8", HostBehavior::U05Hearth2BatchF),
    ("hongxi", HostBehavior::U05Hearth2BatchF),
    ("rejieming", HostBehavior::U05Hearth2BatchF),
    ("xinliegong", HostBehavior::U05Hearth2BatchF),
    ("xinwuyan", HostBehavior::U05Hearth2BatchF),
];

pub(crate) fn lookup_shahai_hearth_b6b_host_behavior(clean: &str) -> Option<HostBehavior> {
    HEARTH_B6B_SKILL_MAP
        .iter()
        .find_map(|&(name, behavior)| (name == clean).then_some(behavior))
}

pub(crate) fn apply_shahai_hearth_b6b_fixups(
    pack: &mut ExtensionPack,
    seen_cards: &mut BTreeSet<String>,
    seen_skills: &mut BTreeSet<String>,
) {
    pack.vm_skills
        .retain(|v| lookup_shahai_hearth_b6b_host_behavior(v.name.as_str()).is_none());
    for &(sk, beh) in HEARTH_B6B_SKILL_MAP {
        if let Some(hs) = pack.host_skills.iter_mut().find(|h| h.name == sk) {
            hs.behavior = beh;
        } else if seen_skills.insert(sk.to_string()) {
            pack.host_skills.push(HostSkillDecl {
                name: names::intern(sk.to_string()),
                behavior: beh,
                presentation_binding: None,
            });
        }
    }

    const EXTRA_TOKENS: &[(&str, CardKind, Suit, u8, u8)] = &[
        ("fulei", CardKind::ShanDian, Suit::Spade, 1, 1),
        ("chuansongmen", CardKind::Chuansongmen, Suit::Club, 5, 4),
        ("liuxinghuoyu", CardKind::Liuxinghuoyu, Suit::Heart, 11, 12),
        ("hslingjian_jinjilengdong", CardKind::Hslingjian, Suit::Spade, 6, 4),
        ("hs_huogong", CardKind::Linghunzhihuo, Suit::Diamond, 12, 2),
        ("hs_horse_plus", CardKind::HorsePlus, Suit::Heart, 5, 2),
        ("hs_horse_minus", CardKind::HorseMinus, Suit::Spade, 5, 2),
        ("hs_armor", CardKind::Armor, Suit::Club, 2, 4),
        ("hs_weapon", CardKind::Weapon, Suit::Spade, 5, 4),
        ("hs_sha", CardKind::Sha, Suit::Spade, 7, 18),
        ("hs_firesha", CardKind::FireSha, Suit::Heart, 7, 6),
        ("hs_leisha", CardKind::LeiSha, Suit::Club, 7, 6),
        ("hs_shan", CardKind::Shan, Suit::Diamond, 2, 18),
        ("hs_tao", CardKind::Tao, Suit::Heart, 3, 6),
        ("hs_jiu", CardKind::Jiu, Suit::Spade, 9, 6),
    ];
    for &(prefix, kind, suit, rank, count) in EXTRA_TOKENS {
        for idx in 1..=count {
            let name = format!("{prefix}_{idx}");
            if !seen_cards.insert(name.clone()) {
                continue;
            }
            let suit_v = if prefix == "liuxinghuoyu" {
                if idx % 2 == 1 { Suit::Heart } else { Suit::Diamond }
            } else if prefix == "fulei" {
                Suit::Spade
            } else {
                match (idx - 1) % 4 {
                    0 => suit,
                    1 => Suit::Heart,
                    2 => Suit::Club,
                    _ => Suit::Diamond,
                }
            };
            let rank_v = ((rank - 1 + idx - 1) % 13) + 1;
            let def = if kind == CardKind::Weapon {
                CardDef::weapon(names::intern(name), 2)
            } else {
                CardDef::basic(names::intern(name), kind)
            }
            .with_suit_rank(suit_v, rank_v)
            .with_off_deck();
            pack.cards.push(def);
        }
    }
}
