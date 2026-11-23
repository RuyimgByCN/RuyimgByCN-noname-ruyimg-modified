//! Batch U05-3 杀海拾遗 `ow` 守望先锋加载器扩展修复（`shaai-ow`）。
//!
//! 严格独立于 `decision34_pack9_for_coexistence`、`bridge_pack`（保留 `g1_wudian` 测试契约）
//! 与 `u01-dddanzhi-pending-v107.ngsa`，仅在形式包装载路径 (`shahai_attach`) 为 `shaai-ow` 生效。

use std::collections::BTreeSet;

use engine_native::catalog::{CardDef, CardKind, Suit};
use engine_native::content::extension::ExtensionPack;
use engine_native::content::host::{HostBehavior, HostSkillDecl};
use engine_native::content::names;

/// 全部 `U05_OW_ALL_SKILLS`（103 项）+ 4 个空标辅助技 + 2 个跨包通用技（`feiying`、`lianpo`）。
pub(crate) const OW_HOST_SKILL_MAP: &[(&str, HostBehavior)] = &[
    // Batch A (tag 1040): 奥丽莎 / 温斯顿 / 堡垒 / 查莉娅 / 路霸 / 黑影
    ("woliu", HostBehavior::U05OwBatchA),
    ("woliu2", HostBehavior::U05OwBatchA),
    ("qianggu", HostBehavior::U05OwBatchA),
    ("qianggu2", HostBehavior::U05OwBatchA),
    ("dianji", HostBehavior::U05OwBatchA),
    ("feitiao", HostBehavior::U05OwBatchA),
    ("feitiao2", HostBehavior::U05OwBatchA),
    ("zhencha", HostBehavior::U05OwBatchA),
    ("bshaowei", HostBehavior::U05OwBatchA),
    ("zhencha2", HostBehavior::U05OwBatchA),
    ("pingzhang", HostBehavior::U05OwBatchA),
    ("pingzhang2", HostBehavior::U05OwBatchA),
    ("pingzhang3", HostBehavior::U05OwBatchA),
    ("owliyong", HostBehavior::U05OwBatchA),
    ("liangou", HostBehavior::U05OwBatchA),
    ("liangou2", HostBehavior::U05OwBatchA),
    ("xiyang", HostBehavior::U05OwBatchA),
    ("qinru", HostBehavior::U05OwBatchA),
    ("yinshen", HostBehavior::U05OwBatchA),
    ("maichong", HostBehavior::U05OwBatchA),
    // Batch B (tag 1041): 莱因哈特 / 托比昂 / 半藏 / 安娜 / 狂鼠 / 麦克雷 / 美
    ("mengji", HostBehavior::U05OwBatchB),
    ("zhongdun", HostBehavior::U05OwBatchB),
    ("maoding", HostBehavior::U05OwBatchB),
    ("maoding2", HostBehavior::U05OwBatchB),
    ("paotai", HostBehavior::U05OwBatchB),
    ("paotai2", HostBehavior::U05OwBatchB),
    ("paotai3", HostBehavior::U05OwBatchB),
    ("bfengshi", HostBehavior::U05OwBatchB),
    ("bfengshi2", HostBehavior::U05OwBatchB),
    ("bfengshi3", HostBehavior::U05OwBatchB),
    ("bfengshi4", HostBehavior::U05OwBatchB),
    ("yinbo", HostBehavior::U05OwBatchB),
    ("aqianghua", HostBehavior::U05OwBatchB),
    ("aqianghua2", HostBehavior::U05OwBatchB),
    ("shihuo", HostBehavior::U05OwBatchB),
    ("zhiyuan", HostBehavior::U05OwBatchB),
    ("mianzhen", HostBehavior::U05OwBatchB),
    ("mianzhen2", HostBehavior::U05OwBatchB),
    ("shoujia", HostBehavior::U05OwBatchB),
    ("shoujia2", HostBehavior::U05OwBatchB),
    ("shoujia3", HostBehavior::U05OwBatchB),
    ("liudan", HostBehavior::U05OwBatchB),
    ("shenqiang", HostBehavior::U05OwBatchB),
    ("tiandan", HostBehavior::U05OwBatchB),
    ("shanguang", HostBehavior::U05OwBatchB),
    ("shanguang2", HostBehavior::U05OwBatchB),
    ("baoxue", HostBehavior::U05OwBatchB),
    ("bingqiang", HostBehavior::U05OwBatchB),
    ("bingqiang_remove", HostBehavior::U05OwBatchB),
    ("bingqiang2", HostBehavior::U05OwBatchB),
    ("bingqiang3", HostBehavior::U05OwBatchB),
    ("bingqiang4", HostBehavior::U05OwBatchB),
    ("bingqiang5", HostBehavior::U05OwBatchB),
    ("jidong", HostBehavior::U05OwBatchB),
    ("jidong2", HostBehavior::U05OwBatchB),
    // Batch C (tag 1042): 黑百合 / D.Va / 死神 / 士兵76 / 源氏（含连破）
    ("duwen", HostBehavior::U05OwBatchC),
    ("duwen2", HostBehavior::U05OwBatchC),
    ("juji", HostBehavior::U05OwBatchC),
    ("juji2", HostBehavior::U05OwBatchC),
    ("juji3", HostBehavior::U05OwBatchC),
    ("dulei", HostBehavior::U05OwBatchC),
    ("dulei2", HostBehavior::U05OwBatchC),
    ("chongzhuang", HostBehavior::U05OwBatchC),
    ("tuijin", HostBehavior::U05OwBatchC),
    ("tuijin2", HostBehavior::U05OwBatchC),
    ("jijia", HostBehavior::U05OwBatchC),
    ("zihui", HostBehavior::U05OwBatchC),
    ("xiandan", HostBehavior::U05OwBatchC),
    ("xiandan2", HostBehavior::U05OwBatchC),
    ("shouge", HostBehavior::U05OwBatchC),
    ("yihun", HostBehavior::U05OwBatchC),
    ("yihun2", HostBehavior::U05OwBatchC),
    ("tuji", HostBehavior::U05OwBatchC),
    ("mujing", HostBehavior::U05OwBatchC),
    ("mujing2", HostBehavior::U05OwBatchC),
    ("lichang", HostBehavior::U05OwBatchC),
    ("lichang2", HostBehavior::U05OwBatchC),
    ("zhanlong", HostBehavior::U05OwBatchC),
    ("zhanlong2", HostBehavior::U05OwBatchC),
    ("feiren", HostBehavior::U05OwBatchC),
    ("feiren3", HostBehavior::U05OwBatchC),
    ("lianpo", HostBehavior::U05OwBatchC),
    // Batch D (tag 1043): 禅雅塔 / 法老之鹰 / 卢西奥 / 秩序之光 / 天使（含飞影）/ 猎空
    ("xie", HostBehavior::U05OwBatchD),
    ("xie2", HostBehavior::U05OwBatchD),
    ("xie3", HostBehavior::U05OwBatchD),
    ("xie4", HostBehavior::U05OwBatchD),
    ("luan", HostBehavior::U05OwBatchD),
    ("luan2", HostBehavior::U05OwBatchD),
    ("luan3", HostBehavior::U05OwBatchD),
    ("luan4", HostBehavior::U05OwBatchD),
    ("sheng", HostBehavior::U05OwBatchD),
    ("sheng2", HostBehavior::U05OwBatchD),
    ("huoyu", HostBehavior::U05OwBatchD),
    ("feidan", HostBehavior::U05OwBatchD),
    ("yuedong", HostBehavior::U05OwBatchD),
    ("huhuan", HostBehavior::U05OwBatchD),
    ("kuoyin", HostBehavior::U05OwBatchD),
    ("kuoyin2", HostBehavior::U05OwBatchD),
    ("guangshu", HostBehavior::U05OwBatchD),
    ("guangshu_diamond", HostBehavior::U05OwBatchD),
    ("guangshu_heart", HostBehavior::U05OwBatchD),
    ("guangshu_club", HostBehavior::U05OwBatchD),
    ("guangshu_spade", HostBehavior::U05OwBatchD),
    ("ziyu", HostBehavior::U05OwBatchD),
    ("shouhu", HostBehavior::U05OwBatchD),
    ("feiying", HostBehavior::U05OwBatchD),
    ("shanxian", HostBehavior::U05OwBatchD),
    ("shanxian2", HostBehavior::U05OwBatchD),
    ("shanhui", HostBehavior::U05OwBatchD),
];

/// Batch U05-3 `ow` 宿主行为分派（覆盖标签 `1040..=1043`）。
pub(crate) fn lookup_shahai_ow_host_behavior(clean: &str) -> Option<HostBehavior> {
    OW_HOST_SKILL_MAP
        .iter()
        .find_map(|&(name, behavior)| (name == clean).then_some(behavior))
}

/// 在形式包附加完成后对 `shaai-ow` 做 U05-3 原生补齐：
/// - 将 `cards.json` 预置的旧 `Wudian*` 声明与旧 VM 代理全部替换为 `U05OwBatchA..=D`；
/// - 注入离线 (`with_off_deck`) 派生卡供应实体：
///   7 张 `ow_hslingjian_*`（瞄定）、4 张 `ow_zhiliaobo_*`（收割）、12 张 `ow_sha_*`（填弹/斩龙）。
pub(crate) fn apply_shahai_ow_pack_fixups(
    package_id: &str,
    pack: &mut ExtensionPack,
    seen_cards: &mut BTreeSet<String>,
    seen_skills: &mut BTreeSet<String>,
) {
    if package_id != "shaai-ow" {
        return;
    }
    pack.vm_skills
        .retain(|v| lookup_shahai_ow_host_behavior(v.name.as_str()).is_none());
    for &(sk, beh) in OW_HOST_SKILL_MAP {
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
    const LINGJIAN_SPECS: [(&str, Suit, u8); 7] = [
        ("ow_hslingjian_xuanfengzhiren", Suit::Spade, 2),
        ("ow_hslingjian_zhongxinghujia", Suit::Heart, 3),
        ("ow_hslingjian_xingtigaizao", Suit::Club, 4),
        ("ow_hslingjian_shijianhuisu", Suit::Diamond, 5),
        ("ow_hslingjian_jinjilengdong", Suit::Spade, 6),
        ("ow_hslingjian_shengxiuhaojiao", Suit::Heart, 7),
        ("ow_hslingjian_yinmilichang", Suit::Club, 8),
    ];
    for copy_idx in 0..8u8 {
        for &(base_name, suit, rank) in &LINGJIAN_SPECS {
            let cname = if copy_idx == 0 {
                base_name.to_string()
            } else {
                format!("{base_name}_{copy_idx}")
            };
            if seen_cards.insert(cname.clone()) {
                pack.cards.push(
                    CardDef::basic(names::intern(cname), CardKind::Hslingjian)
                        .with_suit_rank(suit, rank)
                        .with_off_deck(),
                );
            }
        }
    }
    for i in 0..24u8 {
        let cname = format!("ow_zhiliaobo_{i}");
        if seen_cards.insert(cname.clone()) {
            pack.cards.push(
                CardDef::basic(names::intern(cname), CardKind::Zhiliaobo)
                    .with_suit_rank(Suit::Club, 9)
                    .with_off_deck(),
            );
        }
    }
    for i in 0..52u8 {
        let cname = format!("ow_sha_{i}");
        let suit = match i % 4 {
            0 => Suit::Spade,
            1 => Suit::Heart,
            2 => Suit::Club,
            _ => Suit::Diamond,
        };
        if seen_cards.insert(cname.clone()) {
            pack.cards.push(
                CardDef::basic(names::intern(cname), CardKind::Sha)
                    .with_suit_rank(suit, (i % 13) + 1)
                    .with_off_deck(),
            );
        }
    }
}
