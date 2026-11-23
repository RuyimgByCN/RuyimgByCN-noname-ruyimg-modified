//! Batch U05-1 杀海拾遗加载器扩展修复（`main`/`zhenfa`/`mtg`/`yunchou`）。
//!
//! 严格独立于 `decision34_pack9_for_coexistence` 与 `u01-dddanzhi-pending-v107.ngsa`，
//! 仅在形式包装载路径 (`shahai_attach`) 生效。

use std::collections::BTreeSet;

use engine_native::catalog::{CardDef, CardKind, Suit};
use engine_native::content::extension::ExtensionPack;
use engine_native::content::host::{HostBehavior, HostSkillDecl};
use engine_native::content::names;

/// Batch U05-1 宿主行为分派（覆盖标签 `920..=924`）。
pub(crate) fn lookup_shahai_b1_host_behavior(clean: &str) -> Option<HostBehavior> {
    match clean {
        "stone_chenshui" => Some(HostBehavior::U05MainStoneBatch),
        "u05_zhenfa_recast" => Some(HostBehavior::U05ZhenfaRecastSkill),
        "mduohun"
        | "lingyong"
        | "subplayer"
        | "mhuanyi"
        | "msuoling"
        | "mtongnian"
        | "mbaizhan"
        | "msilian" => Some(HostBehavior::U05MtgCharBatch),
        "mtg_bingheyaosai_skill"
        | "mtg_duzhao_skill"
        | "mtg_cangbaohaiwan_skill"
        | "mtg_lindixiliu_skill" => Some(HostBehavior::U05MtgLandExtraBatch),
        "ycndiyjianxiong"
        | "diyjianxiong"
        | "xinfu_jingxie"
        | "ycnxinfu_qiaosi"
        | "xinfu_qiaosi" => Some(HostBehavior::U05YunchouCharBatch),
        "diy_jiaoxia" => Some(HostBehavior::YunchouJiaoxia),
        _ => None,
    }
}

/// 在形式包附加完成后对 `shahai-mtg` / `shahai-zhenfa` / `shahai-yunchou` 做 U05-1 原生补齐：
/// - `shahai-mtg`: 注入 4 张 `mtg_du_0..=3` 供毒沼使用、2 张 `mtg_mirror_0..=1` 供统念镜像；
/// - `shahai-zhenfa`: 确保全域 `u05_zhenfa_recast` 可发现；
/// - `shahai-yunchou`: 修正 `yc_huoshan_0` 为 `CardKind::Le`、`yc_suolianjia_0` 设 `link_on_equip = true`，
///   并把 `ycndiyjianxiong` / `xinfu_jingxie` / `ycnxinfu_qiaosi` 绑定为 `HostBehavior`（清除旧 VM 代理）。
pub(crate) fn apply_shahai_b1_pack_fixups(
    package_id: &str,
    pack: &mut ExtensionPack,
    seen_cards: &mut BTreeSet<String>,
    seen_skills: &mut BTreeSet<String>,
) {
    match package_id {
        "shahai-mtg" => {
            for i in 0..4u8 {
                let cname = format!("mtg_du_{i}");
                if seen_cards.insert(cname.clone()) {
                    let suit = if i % 2 == 0 { Suit::Spade } else { Suit::Club };
                    let def = CardDef::basic(names::intern(cname), CardKind::Wuzhong)
                        .with_suit_rank(suit, 4 + i)
                        .with_off_deck();
                    pack.cards.push(def);
                }
            }
            for i in 0..2u8 {
                let cname = format!("mtg_mirror_{i}");
                if seen_cards.insert(cname.clone()) {
                    let def = CardDef::basic(names::intern(cname), CardKind::Sha)
                        .with_suit_rank(Suit::Spade, 7 + i)
                        .with_off_deck();
                    pack.cards.push(def);
                }
            }
            let sub_name = "subplayer".to_string();
            if seen_skills.insert(sub_name.clone()) {
                pack.host_skills.push(HostSkillDecl {
                    name: names::intern(sub_name),
                    behavior: HostBehavior::U05MtgCharBatch,
                    presentation_binding: None,
                });
            }
        }
        "shahai-zhenfa" => {
            let recast = "u05_zhenfa_recast".to_string();
            if seen_skills.insert(recast.clone()) {
                pack.host_skills.push(HostSkillDecl {
                    name: names::intern(recast),
                    behavior: HostBehavior::U05ZhenfaRecastSkill,
                    presentation_binding: None,
                });
            }
        }
        "shahai-yunchou" => {
            for c in &mut pack.cards {
                if c.name == "yc_huoshan_0" || c.name == "huoshan" {
                    c.kind = CardKind::Le;
                }
                if c.name == "yc_suolianjia_0" || c.name == "suolianjia" {
                    c.link_on_equip = true;
                }
            }
            let targets = ["ycndiyjianxiong", "xinfu_jingxie", "ycnxinfu_qiaosi"];
            pack.vm_skills.retain(|v| !targets.contains(&v.name.as_str()));
            for &sk in &targets {
                if let Some(hs) = pack.host_skills.iter_mut().find(|h| h.name == sk) {
                    hs.behavior = HostBehavior::U05YunchouCharBatch;
                } else {
                    seen_skills.insert(sk.to_string());
                    pack.host_skills.push(HostSkillDecl {
                        name: names::intern(sk.to_string()),
                        behavior: HostBehavior::U05YunchouCharBatch,
                        presentation_binding: None,
                    });
                }
            }
        }
        _ => {}
    }
}
