//! Phase 3 P3-C1a `L11` 跨包 NPC Part A 选择询问构建（前段 `1..=70`，`asks.rs`）。

use super::helpers::*;
use super::l11_common::*;
use crate::catalog::{CardKind, Catalog, EquipSlot, Suit};
use crate::choice::ChoiceSpec;
use crate::ids::{CardId, PlayerId};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

// ---- Active A (1..=31) ----
pub(super) const ASK_BAIYI_TARGETS: u32 = 1;
pub(super) const ASK_DUWU_TARGET: u32 = 2;
pub(super) const ASK_DUWU_CARDS: u32 = 3;
pub(super) const ASK_FENXUN_CARD: u32 = 4;
pub(super) const ASK_FENXUN_TARGET: u32 = 5;
pub(super) const ASK_GUANZONG_SRC: u32 = 6;
pub(super) const ASK_GUANZONG_VICTIM: u32 = 7;
pub(super) const ASK_GUSHE_TARGETS: u32 = 8;
pub(super) const ASK_GUSHE_OWNER_PINDIAN: u32 = 9;
pub(super) const ASK_GUSHE_TGT_PINDIAN: u32 = 10;
pub(super) const ASK_JICI_CONFIRM: u32 = 11;
pub(super) const ASK_GUSHE_OWNER_CHOICE: u32 = 12;
pub(super) const ASK_GUSHE_TGT_CHOICE: u32 = 13;
pub(super) const ASK_JINGLVE_TARGET: u32 = 14;
pub(super) const ASK_JINGLVE_CARD: u32 = 15;
pub(super) const ASK_JUJIAN_CARDS: u32 = 16;
pub(super) const ASK_JUJIAN_TARGET: u32 = 17;
pub(super) const ASK_LIANZHU_CARD: u32 = 18;
pub(super) const ASK_LIANZHU_TARGET: u32 = 19;
pub(super) const ASK_LIANZHU_DISCARD: u32 = 20;
pub(super) const ASK_LIHUN_CARD: u32 = 21;
pub(super) const ASK_LIHUN_TARGET: u32 = 22;
pub(super) const ASK_LIHUN_RETURN: u32 = 23;
pub(super) const ASK_MINGJIAN_TARGET: u32 = 24;
pub(super) const ASK_MIZHAO_T1: u32 = 25;
pub(super) const ASK_MIZHAO_T2: u32 = 26;
pub(super) const ASK_MIZHAO_P1_CARD: u32 = 27;
pub(super) const ASK_MIZHAO_P2_CARD: u32 = 28;
pub(super) const ASK_MOUZHU_TARGET: u32 = 29;
pub(super) const ASK_MOUZHU_GIVE: u32 = 30;
pub(super) const ASK_MOUZHU_MODE: u32 = 31;

// ---- Active B (32..=70) ----
pub(super) const ASK_NEW_REQINGNANG_CARD: u32 = 32;
pub(super) const ASK_NEW_REQINGNANG_TARGET: u32 = 33;
pub(super) const ASK_NEW_YIJUE_CARD: u32 = 34;
pub(super) const ASK_NEW_YIJUE_TARGET: u32 = 35;
pub(super) const ASK_NEW_YIJUE_SHOW: u32 = 36;
pub(super) const ASK_NEW_YIJUE_HEAL: u32 = 37;
pub(super) const ASK_CHENGLVE_DISCARD: u32 = 38;
pub(super) const ASK_FEIJUN_CARD: u32 = 39;
pub(super) const ASK_FEIJUN_TARGET: u32 = 40;
pub(super) const ASK_FEIJUN_MODE: u32 = 41;
pub(super) const ASK_FEIJUN_TARGET_CARD: u32 = 42;
pub(super) const ASK_QUJI_CARDS: u32 = 43;
pub(super) const ASK_QUJI_TARGETS: u32 = 44;
pub(super) const ASK_REFENXUN_TARGET: u32 = 45;
pub(super) const ASK_REFENXUN2_DISCARD: u32 = 46;
pub(super) const ASK_REGANLU_T1: u32 = 47;
pub(super) const ASK_REGANLU_T2: u32 = 48;
pub(super) const ASK_REGUOSE_CARD: u32 = 49;
pub(super) const ASK_REGUOSE_TARGET: u32 = 50;
pub(super) const ASK_REJUNXING_CARDS: u32 = 51;
pub(super) const ASK_REJUNXING_TARGET: u32 = 52;
pub(super) const ASK_REJUNXING_MODE: u32 = 53;
pub(super) const ASK_REJUNXING_DISCARD: u32 = 54;
pub(super) const ASK_RELUANJI_CARDS: u32 = 55;
pub(super) const ASK_REWURONG_TARGET: u32 = 56;
pub(super) const ASK_REWURONG_OWNER_SHOW: u32 = 57;
pub(super) const ASK_REWURONG_TGT_SHOW: u32 = 58;
pub(super) const ASK_REWURONG_GAIN: u32 = 59;
pub(super) const ASK_REZHIJIAN_CARD: u32 = 60;
pub(super) const ASK_REZHIJIAN_TARGET: u32 = 61;
pub(super) const ASK_REZHIJIAN_DRAW_CONFIRM: u32 = 62;
pub(super) const ASK_RONGBEI_TARGET: u32 = 63;
pub(super) const ASK_ZHIDIAN_ACT_CARD: u32 = 64;
pub(super) const ASK_ZHIDIAN_ACT_TARGET: u32 = 65;
pub(super) const ASK_JIUXIAN_ACT_CARD: u32 = 66;
pub(super) const ASK_OLHUOJI_ACT_CARD: u32 = 67;
pub(super) const ASK_OLHUOJI_ACT_TARGET: u32 = 68;
pub(super) const ASK_SHUANGXIONG_ACT_CARD: u32 = 69;
pub(super) const ASK_SHUANGXIONG_ACT_TARGET: u32 = 70;

pub(super) fn ganlu_pair_ok(
    core: &GameStateCore,
    owner: PlayerId,
    a: PlayerId,
    b: PlayerId,
) -> bool {
    if a == b || !core.player(a).alive || !core.player(b).alive {
        return false;
    }
    let ea = core.player(a).equips.len();
    let eb = core.player(b).equips.len();
    let lost = (core.player(owner).max_hp as i32 - core.player(owner).hp).max(0) as usize;
    (ea > 0 || eb > 0) && (ea.abs_diff(eb) <= lost || a == owner || b == owner)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_l11_npc_a_ask(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    let others = alive_others(core, owner);
    let he = he_cards(core, owner);
    match ask_id {
        ASK_BAIYI_TARGETS => player_ask(owner, "baiyi:choose_targets", others, 2, 2, true),
        ASK_DUWU_TARGET => {
            let pool: Vec<PlayerId> = others
                .into_iter()
                .filter(|&t| {
                    core.in_attack_range(catalog, owner, t)
                        && core.player(t).hp > 0
                        && he.len() >= core.player(t).hp as usize
                })
                .collect();
            player_ask(owner, "duwu:choose_target", pool, 1, 1, true)
        }
        ASK_DUWU_CARDS => {
            let t = target?;
            let x = core.player(t).hp.max(0) as usize;
            card_ask(owner, "duwu:discard_cards", he, x, x, true)
        }
        ASK_FENXUN_CARD => card_ask(owner, "fenxun:discard_card", he, 1, 1, true),
        ASK_FENXUN_TARGET => player_ask(owner, "fenxun:choose_target", others, 1, 1, true),
        ASK_GUANZONG_SRC => player_ask(owner, "guanzong:choose_source", others, 1, 1, true),
        ASK_GUANZONG_VICTIM => {
            let src = target?;
            let pool: Vec<PlayerId> = others.into_iter().filter(|&p| p != src).collect();
            player_ask(owner, "guanzong:choose_target", pool, 1, 1, true)
        }
        ASK_GUSHE_TARGETS => {
            let pool: Vec<PlayerId> = others
                .into_iter()
                .filter(|&t| !core.player(t).hand.is_empty())
                .collect();
            player_ask(owner, "gushe:choose_targets", pool, 1, 3, true)
        }
        ASK_GUSHE_OWNER_PINDIAN => card_ask(
            owner,
            "gushe:owner_pindian",
            core.player(owner).hand.clone(),
            1,
            1,
            false,
        ),
        ASK_GUSHE_TGT_PINDIAN => {
            let t = target?;
            card_ask(
                t,
                "gushe:target_pindian",
                core.player(t).hand.clone(),
                1,
                1,
                false,
            )
        }
        ASK_JICI_CONFIRM => confirm_ask(owner, "jici:confirm"),
        ASK_GUSHE_OWNER_CHOICE => {
            card_ask(owner, "gushe:owner_discard_or_draw", he, 1, 1, true)
        }
        ASK_GUSHE_TGT_CHOICE => {
            let t = target?;
            card_ask(
                t,
                "gushe:target_discard_or_draw",
                he_cards(core, t),
                1,
                1,
                true,
            )
        }
        ASK_JINGLVE_TARGET => {
            let pool: Vec<PlayerId> = others
                .into_iter()
                .filter(|&t| !core.player(t).hand.is_empty())
                .collect();
            player_ask(owner, "jinglve:choose_target", pool, 1, 1, true)
        }
        ASK_JINGLVE_CARD => {
            let t = target?;
            card_ask(owner, "jinglve:choose_card", core.player(t).hand.clone(), 1, 1, false)
        }
        ASK_JUJIAN_CARDS => card_ask(owner, "jujian:discard_cards", he, 1, 3, true),
        ASK_JUJIAN_TARGET => player_ask(owner, "jujian:choose_target", others, 1, 1, true),
        ASK_LIANZHU_CARD => card_ask(owner, "lianzhu:give_card", he, 1, 1, true),
        ASK_LIANZHU_TARGET => player_ask(owner, "lianzhu:choose_target", others, 1, 1, true),
        ASK_LIANZHU_DISCARD => {
            let t = target?;
            card_ask(
                t,
                "lianzhu:target_discard",
                he_cards(core, t),
                2,
                2,
                true,
            )
        }
        ASK_LIHUN_CARD => card_ask(owner, "lihun:discard_card", he, 1, 1, true),
        ASK_LIHUN_TARGET => {
            let pool: Vec<PlayerId> = others
                .into_iter()
                .filter(|&t| core.matches_gender(catalog, t, crate::content::standard::Gender::Male))
                .collect();
            player_ask(owner, "lihun:choose_target", pool, 1, 1, true)
        }
        ASK_LIHUN_RETURN => {
            let t = target?;
            let k = (core.player(t).hp.max(0) as usize).min(he.len());
            card_ask(owner, "lihun:return_cards", he, k, k, false)
        }
        ASK_MINGJIAN_TARGET => player_ask(owner, "mingjian:choose_target", others, 1, 1, true),
        ASK_MIZHAO_T1 => {
            let pool: Vec<PlayerId> = others
                .iter()
                .copied()
                .filter(|&t1| {
                    others
                        .iter()
                        .any(|&t2| t2 != t1 && !core.player(t2).hand.is_empty())
                })
                .collect();
            player_ask(owner, "mizhao:choose_target1", pool, 1, 1, true)
        }
        ASK_MIZHAO_T2 => {
            let t1 = target?;
            let pool: Vec<PlayerId> = others
                .into_iter()
                .filter(|&p| p != t1 && !core.player(p).hand.is_empty())
                .collect();
            player_ask(owner, "mizhao:choose_target2", pool, 1, 1, false)
        }
        ASK_MIZHAO_P1_CARD => {
            let p1 = *targets.first()?;
            card_ask(
                p1,
                "mizhao:pindian_card1",
                core.player(p1).hand.clone(),
                1,
                1,
                false,
            )
        }
        ASK_MIZHAO_P2_CARD => {
            let p2 = *targets.get(1)?;
            card_ask(
                p2,
                "mizhao:pindian_card2",
                core.player(p2).hand.clone(),
                1,
                1,
                false,
            )
        }
        ASK_MOUZHU_TARGET => {
            let pool: Vec<PlayerId> = others
                .into_iter()
                .filter(|&t| !core.player(t).hand.is_empty())
                .collect();
            player_ask(owner, "mouzhu:choose_target", pool, 1, 1, true)
        }
        ASK_MOUZHU_GIVE => {
            let t = target?;
            card_ask(
                t,
                "mouzhu:target_give",
                core.player(t).hand.clone(),
                1,
                1,
                false,
            )
        }
        ASK_MOUZHU_MODE => {
            let t = target?;
            tag_ask(t, "mouzhu:choose_mode", [0, 1], false)
        }
        ASK_NEW_REQINGNANG_CARD => card_ask(
            owner,
            "new_reqingnang:discard_card",
            core.player(owner).hand.clone(),
            1,
            1,
            true,
        ),
        ASK_NEW_REQINGNANG_TARGET => {
            let pool: Vec<PlayerId> = alive_all(core, owner)
                .into_iter()
                .filter(|&t| {
                    core.player(t).hp < core.player(t).max_hp as i32
                        && flag(core, t, "l11_new_reqingnang_t") == 0
                })
                .collect();
            player_ask(owner, "new_reqingnang:choose_target", pool, 1, 1, true)
        }
        ASK_NEW_YIJUE_CARD => card_ask(owner, "new_yijue:discard_card", he, 1, 1, true),
        ASK_NEW_YIJUE_TARGET => {
            let pool: Vec<PlayerId> = others
                .into_iter()
                .filter(|&t| !core.player(t).hand.is_empty())
                .collect();
            player_ask(owner, "new_yijue:choose_target", pool, 1, 1, true)
        }
        ASK_NEW_YIJUE_SHOW => {
            let t = target?;
            card_ask(
                t,
                "new_yijue:target_show",
                core.player(t).hand.clone(),
                1,
                1,
                false,
            )
        }
        ASK_NEW_YIJUE_HEAL => confirm_ask(owner, "new_yijue:confirm_recover"),
        ASK_CHENGLVE_DISCARD => {
            let n = (*cards.first().unwrap_or(&1) as usize).min(core.player(owner).hand.len());
            card_ask(
                owner,
                "nzry_chenglve:discard_cards",
                core.player(owner).hand.clone(),
                n,
                n,
                false,
            )
        }
        ASK_FEIJUN_CARD => card_ask(owner, "nzry_feijun:discard_card", he, 1, 1, true),
        ASK_FEIJUN_TARGET => {
            let my_h = core.player(owner).hand.len();
            let my_e = core.player(owner).equips.len();
            let pool: Vec<PlayerId> = others
                .into_iter()
                .filter(|&t| {
                    core.player(t).hand.len() > my_h || core.player(t).equips.len() > my_e
                })
                .collect();
            player_ask(owner, "nzry_feijun:choose_target", pool, 1, 1, true)
        }
        ASK_FEIJUN_MODE => {
            let t = target?;
            let mut tags = Vec::new();
            if core.player(t).hand.len() > core.player(owner).hand.len() {
                tags.push(0u8);
            }
            if core.player(t).equips.len() > core.player(owner).equips.len() {
                tags.push(1u8);
            }
            tag_ask(owner, "nzry_feijun:choose_mode", tags, false)
        }
        ASK_FEIJUN_TARGET_CARD => {
            let t = target?;
            let is_equip_mode = cards.first().copied().unwrap_or(0) == 1;
            let pool = if is_equip_mode {
                core.player(t).equips.clone()
            } else {
                he_cards(core, t)
            };
            card_ask(t, "nzry_feijun:target_card", pool, 1, 1, false)
        }
        ASK_QUJI_CARDS => {
            let x = (core.player(owner).max_hp as i32 - core.player(owner).hp).max(1) as usize;
            card_ask(owner, "quji:discard_cards", he, x, x, true)
        }
        ASK_QUJI_TARGETS => {
            let x = (core.player(owner).max_hp as i32 - core.player(owner).hp).max(1) as usize;
            let pool: Vec<PlayerId> = alive_all(core, owner)
                .into_iter()
                .filter(|&t| core.player(t).hp < core.player(t).max_hp as i32)
                .collect();
            let max_pick = x.min(pool.len().max(1));
            player_ask(owner, "quji:choose_targets", pool, 1, max_pick, false)
        }
        ASK_REFENXUN_TARGET => player_ask(owner, "refenxun:choose_target", others, 1, 1, true),
        ASK_REFENXUN2_DISCARD => card_ask(owner, "refenxun2:discard_card", he, 1, 1, false),
        ASK_REGANLU_T1 => {
            let all = alive_all(core, owner);
            let pool: Vec<PlayerId> = all
                .iter()
                .copied()
                .filter(|&a| all.iter().any(|&b| ganlu_pair_ok(core, owner, a, b)))
                .collect();
            player_ask(owner, "reganlu:choose_target1", pool, 1, 1, true)
        }
        ASK_REGANLU_T2 => {
            let t1 = target?;
            let pool: Vec<PlayerId> = alive_all(core, owner)
                .into_iter()
                .filter(|&b| ganlu_pair_ok(core, owner, t1, b))
                .collect();
            player_ask(owner, "reganlu:choose_target2", pool, 1, 1, true)
        }
        ASK_REGUOSE_CARD => {
            let pool: Vec<CardId> = he
                .into_iter()
                .filter(|&c| card_suit(catalog, core, owner, c) == Suit::Diamond)
                .collect();
            card_ask(owner, "reguose:choose_card", pool, 1, 1, true)
        }
        ASK_REGUOSE_TARGET => {
            let pool: Vec<PlayerId> = alive_all(core, owner)
                .into_iter()
                .filter(|&t| {
                    let has_le = core
                        .player(t)
                        .judge
                        .iter()
                        .any(|&j| catalog.resolved_card_kind(core, j) == CardKind::Le);
                    has_le
                        || (t != owner
                            && !catalog.blocks_delayed_trick(core, t)
                            && !catalog
                                .skill_by_name(crate::vocabulary::names::QIAN_XUN)
                                .is_some_and(|d| core.has_skill(t, d))
                            && !catalog
                                .skill_by_name("qianxun")
                                .is_some_and(|d| core.has_skill(t, d))
                            && catalog.ow_target_enabled(
                                core,
                                owner,
                                t,
                                cards.first().copied(),
                                CardKind::Le,
                                crate::event::DamageNature::Normal,
                            ))
                })
                .collect();
            player_ask(owner, "reguose:choose_target", pool, 1, 1, true)
        }
        ASK_REJUNXING_CARDS => card_ask(
            owner,
            "rejunxing:discard_cards",
            core.player(owner).hand.clone(),
            1,
            core.player(owner).hand.len().max(1),
            true,
        ),
        ASK_REJUNXING_TARGET => player_ask(owner, "rejunxing:choose_target", others, 1, 1, true),
        ASK_REJUNXING_MODE => {
            let t = target?;
            let x = cards.len();
            let mut tags = vec![1u8];
            if he_cards(core, t).len() >= x {
                tags.insert(0, 0);
            }
            tag_ask(t, "rejunxing:target_mode", tags, false)
        }
        ASK_REJUNXING_DISCARD => {
            let t = target?;
            let x = cards.len().min(he_cards(core, t).len());
            card_ask(t, "rejunxing:target_discard", he_cards(core, t), x, x, false)
        }
        ASK_RELUANJI_CARDS => {
            let used = flag(core, owner, "l11_reluanji_suits_turn");
            let pool: Vec<CardId> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| (used & suit_bit(card_suit(catalog, core, owner, c))) == 0)
                .collect();
            card_ask(owner, "reluanji:choose_cards", pool, 2, 2, true)
        }
        ASK_REWURONG_TARGET => {
            let pool: Vec<PlayerId> = others
                .into_iter()
                .filter(|&t| !core.player(t).hand.is_empty())
                .collect();
            player_ask(owner, "rewurong:choose_target", pool, 1, 1, true)
        }
        ASK_REWURONG_OWNER_SHOW => card_ask(
            owner,
            "rewurong:owner_show",
            core.player(owner).hand.clone(),
            1,
            1,
            false,
        ),
        ASK_REWURONG_TGT_SHOW => {
            let t = target?;
            card_ask(
                t,
                "rewurong:target_show",
                core.player(t).hand.clone(),
                1,
                1,
                false,
            )
        }
        ASK_REWURONG_GAIN => {
            let t = target?;
            card_ask(owner, "rewurong:gain_card", he_cards(core, t), 1, 1, false)
        }
        ASK_REZHIJIAN_CARD => {
            let pool: Vec<CardId> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| {
                    catalog.card_slot_of(core, c).is_some_and(|sl| {
                        others.iter().any(|&t| {
                            catalog.is_equip_slot_enabled(core, t, sl)
                                && !core
                                    .player(t)
                                    .equips
                                    .iter()
                                    .any(|&e| catalog.card_slot_of(core, e) == Some(sl))
                        })
                    })
                })
                .collect();
            card_ask(owner, "rezhijian:choose_card", pool, 1, 1, true)
        }
        ASK_REZHIJIAN_TARGET => {
            let cid = *cards.first()?;
            let sl = catalog.card_slot_of(core, cid)?;
            let pool: Vec<PlayerId> = others
                .into_iter()
                .filter(|&t| {
                    catalog.is_equip_slot_enabled(core, t, sl)
                        && !core
                            .player(t)
                            .equips
                            .iter()
                            .any(|&e| catalog.card_slot_of(core, e) == Some(sl))
                })
                .collect();
            player_ask(owner, "rezhijian:choose_target", pool, 1, 1, true)
        }
        ASK_REZHIJIAN_DRAW_CONFIRM => confirm_ask(owner, "rezhijian:confirm_draw"),
        ASK_RONGBEI_TARGET => {
            let pool: Vec<PlayerId> = alive_all(core, owner)
                .into_iter()
                .filter(|&t| {
                    [
                        EquipSlot::Weapon,
                        EquipSlot::Armor,
                        EquipSlot::HorsePlus,
                        EquipSlot::HorseMinus,
                        EquipSlot::Treasure,
                    ]
                    .into_iter()
                    .any(|sl| {
                        catalog.is_equip_slot_enabled(core, t, sl)
                            && !core
                                .player(t)
                                .equips
                                .iter()
                                .any(|&e| catalog.card_slot_of(core, e) == Some(sl))
                    })
                })
                .collect();
            player_ask(owner, "rongbei:choose_target", pool, 1, 1, true)
        }
        ASK_ZHIDIAN_ACT_CARD => {
            let pool: Vec<CardId> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| {
                    let k = catalog.card_kind(c);
                    !k.is_basic_kind() && !k.is_equip()
                })
                .collect();
            card_ask(owner, "kyou_zhidian:choose_card", pool, 1, 1, true)
        }
        ASK_ZHIDIAN_ACT_TARGET => {
            let pool: Vec<PlayerId> = others
                .into_iter()
                .filter(|&t| {
                    catalog.ow_target_enabled(
                        core,
                        owner,
                        t,
                        cards.first().copied(),
                        CardKind::Sha,
                        crate::event::DamageNature::Normal,
                    )
                })
                .collect();
            player_ask(owner, "kyou_zhidian:choose_target", pool, 1, 1, true)
        }
        ASK_JIUXIAN_ACT_CARD => {
            let pool: Vec<CardId> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| {
                    crate::rvm_bridge::modifiers::d34_l11_mods_npc_a::is_multi_target_trick_for_jiuxian(
                        catalog.card_kind(c),
                        catalog.cards[c as usize].name,
                    )
                })
                .collect();
            card_ask(owner, "dclbjiuxian:choose_card", pool, 1, 1, true)
        }
        ASK_OLHUOJI_ACT_CARD => {
            let pool: Vec<CardId> = he
                .into_iter()
                .filter(|&c| is_red_card(catalog, core, owner, c))
                .collect();
            card_ask(owner, "olhuoji:choose_card", pool, 1, 1, true)
        }
        ASK_OLHUOJI_ACT_TARGET => {
            let pool: Vec<PlayerId> = alive_all(core, owner)
                .into_iter()
                .filter(|&t| !core.player(t).hand.is_empty())
                .collect();
            player_ask(owner, "olhuoji:choose_target", pool, 1, 1, true)
        }
        ASK_SHUANGXIONG_ACT_CARD => {
            let col = flag(core, owner, "shuangxiong_color");
            let pool: Vec<CardId> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| {
                    let red = is_red_card(catalog, core, owner, c);
                    (col == 1 && !red) || (col == 2 && red)
                })
                .collect();
            card_ask(owner, "reshuangxiong:choose_card", pool, 1, 1, true)
        }
        ASK_SHUANGXIONG_ACT_TARGET => player_ask(
            owner,
            "reshuangxiong:choose_target",
            others
                .into_iter()
                .filter(|&t| {
                    catalog.ow_target_enabled(
                        core,
                        owner,
                        t,
                        cards.first().copied(),
                        CardKind::Duel,
                        crate::event::DamageNature::Normal,
                    )
                })
                .collect(),
            1,
            1,
            true,
        ),
        _ => super::asks_b::build_l11_npc_a_ask_b(
            ask_id, owner, target, targets, cards, core, catalog,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::content::host) fn ask_l11_npc_a(
    stage: super::super::HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    if stage == super::super::HostStage::Act {
        return None;
    }
    let ask_id = match card {
        Some(0) => flag(core, owner, "l11_npc_a_ask"),
        Some(id) => id,
        None => return None,
    };
    if ask_id == 0 {
        return None;
    }
    let (observer, spec) =
        build_l11_npc_a_ask(ask_id, owner, target, targets, cards, core, catalog)?;
    Some(crate::snap::ask(
        observer,
        spec.prompt,
        spec.kind,
        spec.options,
        spec.min,
        spec.max,
        spec.allow_cancel,
    ))
}
