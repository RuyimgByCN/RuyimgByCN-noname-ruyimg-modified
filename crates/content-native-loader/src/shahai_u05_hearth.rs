//! Batch U05-6a `杀海拾遗` `hearth` Part 1 加载器扩展修复（`shahai-hearth`）。
//!
//! 严格独立于 `decision34_pack9_for_coexistence`、`bridge_pack`（保留 `g1_hearth` 与
//! `nc08_upstream_hearth_*` 测试契约）与 `u01-dddanzhi-pending-v107.ngsa`，仅在形式包
//! 装载路径（`shahai_attach`）为 `shahai-hearth` 生效。

use std::collections::BTreeSet;

use engine_native::catalog::{CardDef, CardKind, EquipSlot, Suit};
use engine_native::content::extension::ExtensionPack;
use engine_native::content::host::{HostBehavior, HostSkillDecl};
use engine_native::content::names;

use super::U05_HEARTH_ALL_CARDS;

/// U05-6a 全部 109 个技能 + `chuansongmen2` / `chuansongmen3`（`card/hearth/skill.js:5-6`）。
pub(crate) const HEARTH_HOST_SKILL_MAP: &[(&str, HostBehavior)] = &[
    // Card skills & companions (tag 1220)
    ("shihuawuqi", HostBehavior::U05HearthCards),
    ("jihuocard2", HostBehavior::U05HearthCards),
    ("chuansongmen2", HostBehavior::U05HearthCards),
    ("chuansongmen3", HostBehavior::U05HearthCards),
    ("hsmengjing_mengye", HostBehavior::U05HearthCards),
    // Batch A (tag 1221): `hshuanyu` .. `hstianqi_shali` (20)
    ("hshuanyu", HostBehavior::U05HearthBatchA),
    ("wxuying", HostBehavior::U05HearthBatchA),
    ("buwendingyibian_lose", HostBehavior::U05HearthBatchA),
    ("yibian", HostBehavior::U05HearthBatchA),
    ("hualing", HostBehavior::U05HearthBatchA),
    ("muyin", HostBehavior::U05HearthBatchA),
    ("tqchuanyue", HostBehavior::U05HearthBatchA),
    ("oldhuanjue", HostBehavior::U05HearthBatchA),
    ("hshuanling", HostBehavior::U05HearthBatchA),
    ("hsxingyi", HostBehavior::U05HearthBatchA),
    ("asyouzhang", HostBehavior::U05HearthBatchA),
    ("ylyuchu", HostBehavior::U05HearthBatchA),
    ("nsaiqi", HostBehavior::U05HearthBatchA),
    ("nsbeiming", HostBehavior::U05HearthBatchA),
    ("hsnitai", HostBehavior::U05HearthBatchA),
    ("hspuzhao", HostBehavior::U05HearthBatchA),
    ("hsyanxin", HostBehavior::U05HearthBatchA),
    ("hstianqi", HostBehavior::U05HearthBatchA),
    ("hstianqi_dalian", HostBehavior::U05HearthBatchA),
    ("hstianqi_shali", HostBehavior::U05HearthBatchA),
    // Batch B (tag 1222): `ysjqisha` .. `zhuilie` (20)
    ("ysjqisha", HostBehavior::U05HearthBatchB),
    ("ysjqisha_ju", HostBehavior::U05HearthBatchB),
    ("ysjqisha_kuang", HostBehavior::U05HearthBatchB),
    ("ysjqisha_nu", HostBehavior::U05HearthBatchB),
    ("ysjqisha_yi", HostBehavior::U05HearthBatchB),
    ("ysjqisha_wang", HostBehavior::U05HearthBatchB),
    ("ysjqisha_hen", HostBehavior::U05HearthBatchB),
    ("ysjqisha_ao", HostBehavior::U05HearthBatchB),
    ("yindan", HostBehavior::U05HearthBatchB),
    ("hllingxi", HostBehavior::U05HearthBatchB),
    ("zhaochao", HostBehavior::U05HearthBatchB),
    ("zhaochao2", HostBehavior::U05HearthBatchB),
    ("xiyong", HostBehavior::U05HearthBatchB),
    ("srjici", HostBehavior::U05HearthBatchB),
    ("yinzong", HostBehavior::U05HearthBatchB),
    ("tansuo", HostBehavior::U05HearthBatchB),
    ("lieqi", HostBehavior::U05HearthBatchB),
    ("azaowu", HostBehavior::U05HearthBatchB),
    ("shouwang", HostBehavior::U05HearthBatchB),
    ("shouwang2", HostBehavior::U05HearthBatchB),
    ("qingtian", HostBehavior::U05HearthBatchB),
    ("qianfu", HostBehavior::U05HearthBatchB),
    ("qianfu2", HostBehavior::U05HearthBatchB),
    ("shimo", HostBehavior::U05HearthBatchB),
    ("lieyang", HostBehavior::U05HearthBatchB),
    ("lieyang2", HostBehavior::U05HearthBatchB),
    ("zhuilie", HostBehavior::U05HearthBatchB),
    // Batch C (tag 1223): `szbianshen` .. `jinzhou` (20)
    ("szbianshen", HostBehavior::U05HearthBatchC),
    ("kekao", HostBehavior::U05HearthBatchC),
    ("jinhua", HostBehavior::U05HearthBatchC),
    ("kqizhou", HostBehavior::U05HearthBatchC),
    ("jingcu", HostBehavior::U05HearthBatchC),
    ("shengzhang", HostBehavior::U05HearthBatchC),
    ("pyuhuo", HostBehavior::U05HearthBatchC),
    ("mengye", HostBehavior::U05HearthBatchC),
    ("mengye2", HostBehavior::U05HearthBatchC),
    ("lianzhan", HostBehavior::U05HearthBatchC),
    ("kuixin", HostBehavior::U05HearthBatchC),
    ("fuhua", HostBehavior::U05HearthBatchC),
    ("fuhua2", HostBehavior::U05HearthBatchC),
    ("fuhua3", HostBehavior::U05HearthBatchC),
    ("moxie", HostBehavior::U05HearthBatchC),
    ("qianhou", HostBehavior::U05HearthBatchC),
    ("hlongyi", HostBehavior::U05HearthBatchC),
    ("zhongji", HostBehavior::U05HearthBatchC),
    ("fuwen", HostBehavior::U05HearthBatchC),
    ("jinzhou", HostBehavior::U05HearthBatchC),
    // Batch D (tag 1224): `midian` .. `wzhanyi` (20)
    ("midian", HostBehavior::U05HearthBatchD),
    ("xingluo", HostBehavior::U05HearthBatchD),
    ("yuelu", HostBehavior::U05HearthBatchD),
    ("yushou", HostBehavior::U05HearthBatchD),
    ("yushou_lose", HostBehavior::U05HearthBatchD),
    ("yushou_misha", HostBehavior::U05HearthBatchD),
    ("yushou_huofu", HostBehavior::U05HearthBatchD),
    ("yushou_leiouke", HostBehavior::U05HearthBatchD),
    ("qingzun", HostBehavior::U05HearthBatchD),
    ("ayuling", HostBehavior::U05HearthBatchD),
    ("aoshu", HostBehavior::U05HearthBatchD),
    ("bzhuiji", HostBehavior::U05HearthBatchD),
    ("lianjin", HostBehavior::U05HearthBatchD),
    ("shouji", HostBehavior::U05HearthBatchD),
    ("yingxi", HostBehavior::U05HearthBatchD),
    ("yingxi2", HostBehavior::U05HearthBatchD),
    ("hsguimou", HostBehavior::U05HearthBatchD),
    ("peiyu", HostBehavior::U05HearthBatchD),
    ("peiyu2", HostBehavior::U05HearthBatchD),
    ("wzhanyi", HostBehavior::U05HearthBatchD),
    // Batch E (tag 1225): `shengteng` .. `hsbaowu_huangjinyuanhou2` (27)
    ("shengteng", HostBehavior::U05HearthBatchE),
    ("yuansu", HostBehavior::U05HearthBatchE),
    ("chouhuo", HostBehavior::U05HearthBatchE),
    ("nuyan2", HostBehavior::U05HearthBatchE),
    ("nuyan", HostBehavior::U05HearthBatchE),
    ("duxin", HostBehavior::U05HearthBatchE),
    ("oldduxin", HostBehavior::U05HearthBatchE),
    ("hsdusu_shinancao", HostBehavior::U05HearthBatchE),
    ("xiubu", HostBehavior::U05HearthBatchE),
    ("zengli", HostBehavior::U05HearthBatchE),
    ("mobao", HostBehavior::U05HearthBatchE),
    ("mobao2", HostBehavior::U05HearthBatchE),
    ("mobao3", HostBehavior::U05HearthBatchE),
    ("xianji", HostBehavior::U05HearthBatchE),
    ("xianji2", HostBehavior::U05HearthBatchE),
    ("xianji3", HostBehavior::U05HearthBatchE),
    ("tanmi", HostBehavior::U05HearthBatchE),
    ("xueren", HostBehavior::U05HearthBatchE),
    ("maoxian", HostBehavior::U05HearthBatchE),
    ("yiwen", HostBehavior::U05HearthBatchE),
    ("qianghuax", HostBehavior::U05HearthBatchE),
    ("zhuizong", HostBehavior::U05HearthBatchE),
    ("xunbao", HostBehavior::U05HearthBatchE),
    ("xunbao2", HostBehavior::U05HearthBatchE),
    ("hsbaowu_cangbaotu", HostBehavior::U05HearthBatchE),
    ("hsbaowu_huangjinyuanhou", HostBehavior::U05HearthBatchE),
    ("hsbaowu_huangjinyuanhou2", HostBehavior::U05HearthBatchE),
];

/// Batch U05-6a `hearth` Part 1 宿主行为分派（覆盖标签 `1220..=1225`）。
pub(crate) fn lookup_shahai_hearth_host_behavior(clean: &str) -> Option<HostBehavior> {
    HEARTH_HOST_SKILL_MAP
        .iter()
        .find_map(|&(name, behavior)| (name == clean).then_some(behavior))
}

fn is_hearth_derived_card_name(name: &str) -> bool {
    name == "shihuawuqi"
        || name.starts_with("hsfashu_")
        || name.starts_with("hstianqi_")
        || name.starts_with("hsjixie_")
        || name.starts_with("hsqizhou_")
        || name.starts_with("hsqingyu_")
        || name.starts_with("hsdusu_")
        || name.starts_with("hsbaowu_")
        || name.starts_with("hsshenqi_")
        || name.starts_with("hsmengjing_")
        || name.starts_with("hszuzhou_")
        || name.starts_with("tuteng")
}

fn push_off_deck_card(
    pack: &mut ExtensionPack,
    seen_cards: &mut BTreeSet<String>,
    name: String,
    kind: CardKind,
    range: u32,
    suit: Suit,
    rank: u8,
) {
    if !seen_cards.insert(name.clone()) {
        return;
    }
    let interned = names::intern(name.clone());
    let mut def = if kind.equip_slot() == Some(EquipSlot::Weapon) {
        CardDef::weapon(interned, range.max(1))
    } else {
        CardDef::basic(interned, kind)
    }
    .with_suit_rank(suit, rank)
    .with_off_deck();
    if name == "hstianqi_dalian" || name.starts_with("hstianqi_dalian_") {
        def.equip_skills = vec!["hstianqi_dalian"];
    } else if name == "hstianqi_shali" || name.starts_with("hstianqi_shali_") {
        def.equip_skills = vec!["hstianqi_shali"];
    }
    pack.cards.push(def);
}

/// 在形式包附加完成后对 `shahai-hearth` 做 U05-6a 原生补齐：
/// 1. 清除 U05-6a 技能的旧 VM 代理并确保全量绑定 `U05HearthCards` / `U05HearthBatchA..=E`；
/// 2. 修正 `ht_shandianjian_*` 形式包类型为锦囊 (`CardKind::Wuzhong`)，保留 `bridge_pack` 测试；
/// 3. 将 `shihuawuqi` 与 44 张 `character/hearth/card.js` 衍生卡标记为 `off_deck`，
///    并补充充足的 `detached_cards` 衍生供应牌实体以保物理牌守恒。
pub(crate) fn apply_shahai_hearth_pack_fixups(
    package_id: &str,
    pack: &mut ExtensionPack,
    seen_cards: &mut BTreeSet<String>,
    seen_skills: &mut BTreeSet<String>,
) {
    if package_id != "shahai-hearth" {
        return;
    }
    pack.vm_skills
        .retain(|v| lookup_shahai_hearth_host_behavior(v.name.as_str()).is_none());
    for &(sk, beh) in HEARTH_HOST_SKILL_MAP {
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

    for c in &mut pack.cards {
        if c.name == "shandianjian"
            || c.name.starts_with("shandianjian_")
            || c.name.starts_with("ht_shandianjian_")
        {
            c.kind = CardKind::Wuzhong;
        }
        if is_hearth_derived_card_name(c.name) {
            c.off_deck = true;
        }
        if c.name == "hstianqi_dalian" {
            c.equip_skills = vec!["hstianqi_dalian"];
        } else if c.name == "hstianqi_shali" {
            c.equip_skills = vec!["hstianqi_shali"];
        }
    }

    // Extra off-deck copies for every `character/hearth/card.js` card + `shihuawuqi`
    for &(cname, kind, range, suit, rank) in &U05_HEARTH_ALL_CARDS {
        if !is_hearth_derived_card_name(cname) {
            continue;
        }
        let copies = if cname == "hsfashu_buwendingyibian" || cname == "hsjixie_zhadan" {
            4u8
        } else {
            2u8
        };
        for idx in 1..=copies {
            let mut s = suit;
            let mut r = rank;
            if cname == "hstianqi_dalian" {
                s = Suit::Spade;
                r = 1;
            } else if cname == "hstianqi_shali" {
                s = Suit::Heart;
                r = 1;
            } else if cname == "hstianqi_suolasi" {
                s = Suit::Diamond;
                r = 1;
            } else if cname == "hstianqi_nazigelin" {
                s = Suit::Club;
                r = 1;
            } else {
                r = ((r - 1 + idx) % 13) + 1;
            }
            push_off_deck_card(pack, seen_cards, format!("{cname}_{idx}"), kind, range, s, r);
        }
    }

    // Extra off-deck standard / special token supplies for `game.createCard` effects
    const TOKEN_SUPPLIES: &[(&str, CardKind, u32, Suit, u8, u8)] = &[
        ("hs_sha", CardKind::Sha, 1, Suit::Spade, 7, 6),
        ("hs_firesha", CardKind::FireSha, 1, Suit::Heart, 7, 2),
        ("hs_leisha", CardKind::LeiSha, 1, Suit::Club, 7, 2),
        ("hs_shan", CardKind::Shan, 0, Suit::Diamond, 2, 6),
        ("hs_tao", CardKind::Tao, 0, Suit::Heart, 3, 2),
        ("hs_jiu", CardKind::Jiu, 0, Suit::Spade, 9, 2),
        ("hs_wuzhong", CardKind::Wuzhong, 0, Suit::Heart, 7, 2),
        ("hs_juedou", CardKind::Duel, 0, Suit::Spade, 1, 2),
        ("hs_guohe", CardKind::Dismantle, 0, Suit::Spade, 3, 2),
        ("hs_shunshou", CardKind::Snatch, 0, Suit::Diamond, 3, 2),
        ("hs_tiesuo", CardKind::Chain, 0, Suit::Club, 12, 2),
        ("hs_nanman", CardKind::SavageAttack, 0, Suit::Club, 7, 2),
        ("hs_wanjian", CardKind::ArcheryAttack, 0, Suit::Heart, 1, 2),
        ("hs_taoyuan", CardKind::Taoyuan, 0, Suit::Heart, 1, 2),
        ("hs_wugu", CardKind::Wugu, 0, Suit::Heart, 3, 2),
        ("hs_wuxie", CardKind::Nullify, 0, Suit::Club, 13, 2),
        ("hs_lebu", CardKind::Le, 0, Suit::Heart, 6, 2),
        ("hs_bingliang", CardKind::Bing, 0, Suit::Club, 10, 2),
        ("hs_shandian", CardKind::ShanDian, 0, Suit::Spade, 1, 2),
        ("hs_qinggang", CardKind::Qinggang, 2, Suit::Spade, 6, 4),
        ("hs_hanbing", CardKind::Hanbing, 2, Suit::Spade, 2, 4),
        ("hs_zhuge", CardKind::Zhuge, 1, Suit::Diamond, 1, 4),
        ("hs_cixiong", CardKind::Cixiong, 2, Suit::Spade, 2, 4),
        ("hs_guanshi", CardKind::Guanshi, 3, Suit::Diamond, 5, 4),
        ("hs_qinglong", CardKind::Qinglong, 3, Suit::Spade, 5, 4),
        ("hs_zhangba", CardKind::Zhangba, 3, Suit::Spade, 12, 4),
        ("hs_fangtian", CardKind::Fangtian, 4, Suit::Diamond, 12, 4),
        ("hs_qilin", CardKind::Qilin, 5, Suit::Heart, 5, 4),
        ("hs_renwang", CardKind::Renwang, 0, Suit::Club, 2, 2),
        ("du", CardKind::Wuzhong, 0, Suit::Spade, 2, 4),
        ("hsyaoshui", CardKind::Wuzhong, 0, Suit::Club, 6, 4),
        ("hsnitai_card", CardKind::Wuzhong, 0, Suit::Heart, 8, 4),
        ("hs_lingjian", CardKind::Hslingjian, 0, Suit::Club, 5, 4),
    ];
    for &(prefix, kind, range, suit, rank, count) in TOKEN_SUPPLIES {
        for idx in 1..=count {
            let suit_v = match (idx - 1) % 4 {
                0 => suit,
                1 => Suit::Heart,
                2 => Suit::Club,
                _ => Suit::Diamond,
            };
            let rank_v = ((rank - 1 + idx - 1) % 13) + 1;
            push_off_deck_card(
                pack,
                seen_cards,
                format!("{prefix}_{idx}"),
                kind,
                range,
                suit_v,
                rank_v,
            );
        }
    }
}
