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
                let mut w = CardDef::weapon(interned, range.max(1));
                w.kind = kind;
                w
            } else {
                CardDef::basic(interned, kind)
            }
            .with_suit_rank(suit, rank);
            if matches!(
                cname,
                "gjyuheng_plus"
                    | "gjyuheng_pro"
                    | "yanjiadan_heart"
                    | "yanjiadan_diamond"
                    | "yanjiadan_club"
                    | "yanjiadan_spade"
            ) {
                def = def.with_off_deck();
            }
            def.equip_skills = if def.name.starts_with("gw_dieyi_") {
                card_kind_equip_skills(kind).to_vec()
            } else if cname == "gjyuheng_plus" {
                vec!["gjyuheng_plus_skill"]
            } else if cname == "gjyuheng_pro" {
                vec!["gjyuheng_pro_skill"]
            } else if cname == "fuxiqin" {
                vec!["kongxin"]
            } else if kind == CardKind::WuxingPan {
                vec!["wuxingpan_skill"]
            } else {
                card_kind_equip_skills(kind).to_vec()
            };
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

fn lookup_swd_u05_7a_behavior(name: &str) -> Option<engine_native::content::host::HostBehavior> {
    use engine_native::content::host::HostBehavior;
    if matches!(
        name,
        "cangchizhibi_equip1"
            | "cangchizhibi_equip2"
            | "cangchizhibi_equip3"
            | "cangchizhibi_equip4"
            | "cangchizhibi_equip5"
            | "huanglinzhicong_equip1"
            | "huanglinzhicong_equip2"
            | "huanglinzhicong_equip3"
            | "huanglinzhicong_equip4"
            | "huanglinzhicong_equip5"
            | "xuanwuzhihuang_equip1"
            | "xuanwuzhihuang_equip2"
            | "xuanwuzhihuang_equip3"
            | "xuanwuzhihuang_equip4"
            | "xuanwuzhihuang_equip5"
            | "zhuquezhizhang_equip1"
            | "zhuquezhizhang_equip2"
            | "zhuquezhizhang_equip3"
            | "zhuquezhizhang_equip4"
            | "zhuquezhizhang_equip5"
            | "baishouzhihu_equip1"
            | "baishouzhihu_equip2"
            | "baishouzhihu_equip3"
            | "baishouzhihu_equip4"
            | "baishouzhihu_equip5"
            | "qinglongzhigui_equip1"
            | "qinglongzhigui_equip2"
            | "qinglongzhigui_equip3"
            | "qinglongzhigui_equip4"
            | "qinglongzhigui_equip5"
    ) {
        return Some(HostBehavior::U05SwdForgingEnd);
    }
    match name {
        "g_hufu_sha" | "g_hufu_shan" | "g_hufu_jiu" => Some(HostBehavior::U05SwdHufuViewAs),
        "g_yuchan_equip" => Some(HostBehavior::U05SwdYuchanEquip),
        "zhiluxiaohu" => Some(HostBehavior::U05SwdZhiluxiaohuDraw),
        "huanpodan_skill" => Some(HostBehavior::U05SwdHuanpodanGuard),
        "pangufu" => Some(HostBehavior::U05SwdPangufu),
        "qiankundai" | "zhufangshenshi" | "g_zhufangshenshi" | "lianyaohu_skill" | "yuruyi"
        | "shuchui" | "xuejibingbao" | "gouhunluo" | "haotianta" | "kongdongyin"
        | "xuanyuanjian" => Some(HostBehavior::U05SwdPassiveMarker),
        "g_yuchan_swap" => Some(HostBehavior::SwdYuchanSwap),
        "g_shencaojie" => Some(HostBehavior::SwdShencaojieGuard),
        "g_shenmiguo" => Some(HostBehavior::SwdShenmiguoWindow),
        "shuchui2" => Some(HostBehavior::SwdShuchuiUse),
        "gouhunluo2" => Some(HostBehavior::SwdGouhunluoSub),
        "jiguanyuan" => Some(HostBehavior::SwdJiguanyuanGain),
        "g_qinglongzhigui" => Some(HostBehavior::SwdQinglongzhiguiHeld),
        "g_baishouzhihu" => Some(HostBehavior::SwdBaishouzhihuHeld),
        "g_zhuquezhizhang" => Some(HostBehavior::SwdZhuquezhizhangHeld),
        "g_xuanwuzhihuang" => Some(HostBehavior::SwdXuanwuzhihuangHeld),
        "g_huanglinzhicong" => Some(HostBehavior::SwdHuanglinzhicongHeld),
        "g_cangchizhibi" => Some(HostBehavior::SwdCangchizhibiHeld),
        "kunlunjingc" => Some(HostBehavior::SwdKunlunjingcCard),
        "lianhua" => Some(HostBehavior::SwdLianyaohuLianhua),
        "shouna" => Some(HostBehavior::SwdLianyaohuShouna),
        "donghuangzhong" => Some(HostBehavior::SwdDonghuangzhongCard),
        "xuanyuanjian2" => Some(HostBehavior::XuanyuanjianDamage),
        "xuanyuanjian3" => Some(HostBehavior::XuanyuanjianBacklash),
        "shennongding" => Some(HostBehavior::SwdShennongdingCard),
        _ => None,
    }
}

fn lookup_swd_u05_7b_behavior(name: &str) -> Option<engine_native::content::host::HostBehavior> {
    use engine_native::content::host::HostBehavior;
    match name {
        "nvwashi" => Some(HostBehavior::SwdNvwashiGuard),
        "kongxin" => Some(HostBehavior::U05SwdKongxin),
        "kongxin2" => Some(HostBehavior::U05SwdKongxin2),
        "qinglianxindeng" => Some(HostBehavior::U05SwdQinglianxindeng),
        "yiluan" => Some(HostBehavior::SwdSifeizhenmianCard),
        "hslingjian_xuanfengzhiren_equip1"
        | "hslingjian_zhongxinghujia_equip1"
        | "hslingjian_jinjilengdong_equip1"
        | "hslingjian_yinmilichang_equip1"
        | "hslingjian_xingtigaizao_equip1" => Some(HostBehavior::U05SwdLingjianDmgSrc),
        "hslingjian_xuanfengzhiren_equip2"
        | "hslingjian_zhongxinghujia_equip2"
        | "hslingjian_jinjilengdong_equip2"
        | "hslingjian_yinmilichang_equip2"
        | "hslingjian_xingtigaizao_equip2" => Some(HostBehavior::U05SwdLingjianDmgTgt),
        "hslingjian_xuanfengzhiren_equip3" | "hslingjian_xuanfengzhiren_equip4" => {
            Some(HostBehavior::U05SwdLingjianLoseTemp)
        }
        "hslingjian_shijianhuisu_equip1" | "hslingjian_shijianhuisu_equip2" => {
            Some(HostBehavior::U05SwdLingjianEquipped)
        }
        "hslingjian_xingtigaizao_equip5" => Some(HostBehavior::U05SwdLingjianDrawPhase),
        "hslingjian_jinjilengdong_equip5" | "hslingjian_xingtigaizao" => {
            Some(HostBehavior::U05SwdLingjianPhaseEnd)
        }
        "hslingjian_xuanfengzhiren_equip5"
        | "hslingjian_zhongxinghujia_equip5"
        | "hslingjian_shengxiuhaojiao_equip5"
        | "hslingjian_shijianhuisu_equip5" => Some(HostBehavior::U05SwdLingjianPhaseUse),
        "hslingjian_shengxiuhaojiao_equip1" => Some(HostBehavior::U05SwdLingjianShaBegin),
        "hslingjian_xuanfengzhiren_equip3_dist"
        | "hslingjian_xuanfengzhiren_equip4_dist"
        | "hslingjian_zhongxinghujia_equip3"
        | "hslingjian_zhongxinghujia_equip4"
        | "hslingjian_jinjilengdong_equip3"
        | "hslingjian_jinjilengdong_equip4"
        | "hslingjian_yinmilichang_equip3"
        | "hslingjian_yinmilichang_equip4"
        | "hslingjian_yinmilichang_equip5"
        | "hslingjian_xingtigaizao_equip3"
        | "hslingjian_xingtigaizao_equip4"
        | "hslingjian_shengxiuhaojiao_equip2"
        | "hslingjian_shengxiuhaojiao_equip3"
        | "hslingjian_shengxiuhaojiao_equip4"
        | "hslingjian_shijianhuisu_equip3"
        | "hslingjian_shijianhuisu_equip4"
        | "hslingjian_chaofeng"
        | "hslingjian_jinjilengdong" => Some(HostBehavior::U05SwdLingjianPassive),
        "jiguanyaoshu_skill" => Some(HostBehavior::U05SwdJiguanyaoshuSkill),
        "lingjianduanzao" => Some(HostBehavior::U05SwdLingjianduanzao),
        "_lingjianduanzao" => Some(HostBehavior::U05SwdForgeAction),
        "qinglonglingzhu" => Some(HostBehavior::U05SwdQinglonglingzhu),
        "xingjunyan" => Some(HostBehavior::U05SwdXingjunyan),
        "baihupifeng" => Some(HostBehavior::U05SwdBaihupifeng),
        "fengxueren" => Some(HostBehavior::U05SwdFengxueren),
        "chilongya" => Some(HostBehavior::U05SwdChilongya),
        "chilongya2" => Some(HostBehavior::U05SwdChilongya2),
        "chilongfengxue" => Some(HostBehavior::U05SwdChilongfengxue),
        "shentou" => Some(HostBehavior::SwdShentouEquipSkill),
        "old_longfan" => Some(HostBehavior::U05SwdOldLongfan),
        "longfan" => Some(HostBehavior::SwdLangeguaiyiEquipSkill),
        "touzhi" => Some(HostBehavior::SwdYufuluEquipSkill),
        "xixue" => Some(HostBehavior::U05SwdXixue),
        "guangshatianyi" => Some(HostBehavior::U05SwdGuangshatianyi),
        "nigong" => Some(HostBehavior::SwdNigongAccumulate),
        "nigong2" | "nigong3" => Some(HostBehavior::SwdNigongStrike),
        "sadengjinhuan" => Some(HostBehavior::U05SwdSadengjinhuan),
        "guiyanfadao" => Some(HostBehavior::U05SwdGuiyanfadao),
        "guiyanfadao2" => Some(HostBehavior::U05SwdPassiveMarker),
        "tianxianjiu" => Some(HostBehavior::U05SwdTianxianjiu),
        _ => None,
    }
}

fn lookup_xianjian_u05_4_behavior(name: &str) -> Option<engine_native::content::host::HostBehavior> {
    use engine_native::content::host::HostBehavior;
    match name {
        "qiongguang" | "huanlei" | "tianwu" | "yujia" | "yanshi" | "tanhua" | "ywuhun"
        | "qixia" | "jianzhen" => Some(HostBehavior::U05XianjianBatchA),
        "feizhua" | "diesha" | "guijiang2" | "guijiang3" | "fenxing" | "xfenxin"
        | "xfenxin2" | "qijian" | "qianfang2" | "poyun2" | "zhuyue2" | "longxi"
        | "xjzhimeng3" => Some(HostBehavior::U05XianjianBatchB),
        "xiaoyue" | "xiaoyue2" | "yanzhan2" | "yanzhan3" | "yanzhan4" | "danqing"
        | "danqing_count" | "zhangmu_ai" | "leiyu" | "leiyu2" | "leiyu3" | "leiyu4"
        | "guijin2" | "chengxin2" | "yuexing2" => Some(HostBehavior::U05XianjianBatchC),
        "lingdi" | "xjzongyu" | "dujiang" | "yanzhan" | "feixia" | "huxi" | "guijiang"
        | "duxinshu" | "tuoqiao" => Some(HostBehavior::U05XianjianBatchD),
        _ => None,
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
    let ow_beh = (package_id == "shaai-ow")
        .then(|| crate::shahai_u05_ow::lookup_shahai_ow_host_behavior(clean))
        .flatten();
    let hearth_beh = (package_id == "shahai-hearth")
        .then(|| crate::shahai_u05_hearth::lookup_shahai_hearth_host_behavior(clean))
        .flatten();
    let xianjian_beh = (package_id == "shaai-xianjian")
        .then(|| lookup_xianjian_u05_4_behavior(clean))
        .flatten();
    let swd_beh = (package_id == "shahai-swd")
        .then(|| lookup_swd_u05_7a_behavior(clean).or_else(|| lookup_swd_u05_7b_behavior(clean)))
        .flatten();
    if let Some(behavior) = ow_beh
        .or(hearth_beh)
        .or(xianjian_beh)
        .or(swd_beh)
        .or_else(|| crate::shahai_u05_b1::lookup_shahai_b1_host_behavior(clean))
        .or_else(|| lookup_upstream_host_behavior(&sk_owned))
        .or_else(|| lookup_upstream_host_behavior(clean))
    {
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
    let attached = match package_id {
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
            crate::shahai_u05_hearth::apply_shahai_hearth_pack_fixups(
                package_id,
                pack,
                seen_cards,
                seen_skills,
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
            crate::shahai_u05_ow::apply_shahai_ow_pack_fixups(
                package_id,
                pack,
                seen_cards,
                seen_skills,
            );
            true
        }
        "shaai-xianjian" => {
            if seen_cards.insert("du".to_string()) {
                pack.cards.push(
                    CardDef::basic(names::intern("du".to_string()), CardKind::Caoyao)
                        .with_suit_rank(Suit::Spade, 2)
                        .with_off_deck(),
                );
            }
            for idx in 2u8..=32 {
                let cname = format!("xj_du_{idx}");
                if seen_cards.insert(cname.clone()) {
                    let rank = ((idx - 1) % 13) + 1;
                    pack.cards.push(
                        CardDef::basic(names::intern(cname), CardKind::Caoyao)
                            .with_suit_rank(Suit::Spade, rank)
                            .with_off_deck(),
                    );
                }
            }
            for (cname, suit, rank) in [
                ("xj_mujiaren", Suit::Diamond, 12u8),
                ("xj_zhiluxiaohu", Suit::Spade, 13),
                ("xj_jiguantong", Suit::Heart, 9),
                ("xj_jiguanyuan", Suit::Club, 9),
            ] {
                if seen_cards.insert(cname.to_string()) {
                    pack.cards.push(
                        CardDef::basic(names::intern(cname.to_string()), CardKind::Jiguan)
                            .with_suit_rank(suit, rank)
                            .with_off_deck(),
                    );
                }
            }
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
    };
    if attached {
        crate::shahai_u05_b1::apply_shahai_b1_pack_fixups(
            package_id,
            pack,
            seen_cards,
            seen_skills,
        );
    }
    attached
}
