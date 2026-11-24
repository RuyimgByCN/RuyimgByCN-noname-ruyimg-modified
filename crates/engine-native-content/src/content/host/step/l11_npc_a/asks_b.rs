//! Phase 3 P3-C1a `L11` 跨包 NPC Part A 选择询问构建（后段 `71..=145`，`asks_b.rs`）。

use super::helpers::*;
use super::l11_common::*;
use super::l11_skills_b_asks::can_receive_moved_card;
use crate::catalog::{CardKind, Catalog, EquipSlot, Suit};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::ids::{CardId, PlayerId};
use crate::state::GameStateCore;

// ---- Damage / CardUsed (71..=105) ----
pub(super) const ASK_BENYU_MODE: u32 = 71;
pub(super) const ASK_BENYU_DISCARD: u32 = 72;
pub(super) const ASK_BMCANSHI_CANCEL_CONFIRM: u32 = 73;
pub(super) const ASK_BMCANSHI_EXTRA_TARGETS: u32 = 74;
pub(super) const ASK_CHOUCE_CONFIRM: u32 = 75;
pub(super) const ASK_CHOUCE_DISCARD_TARGET: u32 = 76;
pub(super) const ASK_CHOUCE_DISCARD_CARD: u32 = 77;
pub(super) const ASK_CHOUCE_DRAW_TARGET: u32 = 78;
pub(super) const ASK_DCMOUKUI_MODE: u32 = 79;
pub(super) const ASK_DCMOUKUI_DODGED_DISCARD: u32 = 80;
pub(super) const ASK_GAOYUAN_TARGET: u32 = 81;
pub(super) const ASK_GAOYUAN_CARD: u32 = 82;
pub(super) const ASK_JIEYUAN_MORE_CARD: u32 = 83;
pub(super) const ASK_JIEYUAN_LESS_CARD: u32 = 84;
pub(super) const ASK_JSRGSHICHONG_CONFIRM: u32 = 85;
pub(super) const ASK_JSRGSHICHONG_STEAL: u32 = 86;
pub(super) const ASK_JSRGSHICHONG_GIVE: u32 = 87;
pub(super) const ASK_ZHIDIAN_AIM_EFFECT: u32 = 88;
pub(super) const ASK_LEIJI_TARGET: u32 = 89;
pub(super) const ASK_LIYU_CONFIRM_TARGET: u32 = 90;
pub(super) const ASK_LIYU_GAIN_CARD: u32 = 91;
pub(super) const ASK_NSXIANHAI_CONFIRM: u32 = 92;
pub(super) const ASK_NSXIANHAI_SLOT: u32 = 93;
pub(super) const ASK_SHICAI_CONFIRM: u32 = 94;
pub(super) const ASK_OLZHUIJI_MODE: u32 = 95;
pub(super) const ASK_OLZHUIJI_DISCARD: u32 = 96;
pub(super) const ASK_QINGXIAN_TARGET: u32 = 97;
pub(super) const ASK_QINGXIAN_MODE: u32 = 98;
pub(super) const ASK_QINGXIAN_DISCARD_EQUIP: u32 = 99;
pub(super) const ASK_REDUODAO_DISCARD: u32 = 100;
pub(super) const ASK_REPOJUN_CARDS: u32 = 101;
pub(super) const ASK_RETIANXIANG_CARD: u32 = 102;
pub(super) const ASK_RETIANXIANG_TARGET: u32 = 103;
pub(super) const ASK_RETIANXIANG_MODE: u32 = 104;
pub(super) const ASK_REXINGLUAN_TARGET: u32 = 105;
pub(super) const ASK_REXINGLUAN_CONFIRM: u32 = 106;
pub(super) const ASK_REXINGLUAN_EJ_CARD: u32 = 107;
pub(super) const ASK_REYAJIAO_CONFIRM: u32 = 108;
pub(super) const ASK_REYAJIAO_GIVE_TARGET: u32 = 109;
pub(super) const ASK_REYAJIAO_DIFF_MODE: u32 = 110;

// ---- Phase / Turn / Dying / Death / Lose (111..=145) ----
pub(super) const ASK_DIANCAI_CONFIRM: u32 = 111;
pub(super) const ASK_GNSHEQUE_SHA: u32 = 112;
pub(super) const ASK_JIGONG_CONFIRM: u32 = 113;
pub(super) const ASK_LIXIA_MODE: u32 = 114;
pub(super) const ASK_MOZHI_CARD: u32 = 115;
pub(super) const ASK_MOZHI_TARGET: u32 = 116;
pub(super) const ASK_KUIZHU_MODE: u32 = 117;
pub(super) const ASK_KUIZHU_DRAW_TARGETS: u32 = 118;
pub(super) const ASK_KUIZHU_DMG_TARGETS: u32 = 119;
pub(super) const ASK_PIAOLING_CONFIRM: u32 = 120;
pub(super) const ASK_PIAOLING_GIVE_TARGET: u32 = 121;
pub(super) const ASK_PIAOLING_SELF_DISCARD: u32 = 122;
pub(super) const ASK_QINGZHONGX_CONFIRM: u32 = 123;
pub(super) const ASK_QINGZHONGX_SWAP_TARGET: u32 = 124;
pub(super) const ASK_REGUANXING_CONFIRM: u32 = 125;
pub(super) const ASK_REGUANXING_TOP: u32 = 126;
pub(super) const ASK_REJUECE_TARGET: u32 = 127;
pub(super) const ASK_RELUOYI_CONFIRM: u32 = 128;
pub(super) const ASK_RESHUANGXIONG_CONFIRM: u32 = 129;
pub(super) const ASK_RESHUANGXIONG_PICK: u32 = 130;
pub(super) const ASK_RESHUANGXIONG_DUEL_CONFIRM: u32 = 131;
pub(super) const ASK_RETUXI_TARGETS: u32 = 132;
pub(super) const ASK_REZAIQI_TARGETS: u32 = 133;
pub(super) const ASK_REZAIQI_TARGET_MODE: u32 = 134;
pub(super) const ASK_CHENQING_TARGET: u32 = 135;
pub(super) const ASK_CHENQING_DISCARD: u32 = 136;
pub(super) const ASK_OLNIEPAN_CONFIRM: u32 = 137;
pub(super) const ASK_OLNIEPAN_SKILL: u32 = 138;
pub(super) const ASK_REXINGSHANG_MODE: u32 = 139;
pub(super) const ASK_FENJI_CONFIRM: u32 = 140;
pub(super) const ASK_LIANGZHU_MODE: u32 = 141;
pub(super) const ASK_LIRANG_TARGET: u32 = 142;
pub(super) const ASK_LIRANG_CARDS: u32 = 143;
pub(super) const ASK_RELIANYING_TARGETS: u32 = 144;
pub(super) const ASK_RENSHE_MODE: u32 = 145;
pub(super) const ASK_RENSHE_DRAW_TARGET: u32 = 146;
pub(super) const ASK_RENSHE_FACTION_TARGET: u32 = 147;
pub(super) const ASK_REQUANJI_CONFIRM: u32 = 148;
pub(super) const ASK_REQUANJI_CARD: u32 = 149;
pub(super) const ASK_RETUNTIAN_CONFIRM: u32 = 150;
pub(super) const ASK_REXUANFENG_MODE: u32 = 151;
pub(super) const ASK_REXUANFENG_DISC_TARGET: u32 = 152;
pub(super) const ASK_REXUANFENG_DISC_CARD: u32 = 153;
pub(super) const ASK_REXUANFENG_MOVE_FROM: u32 = 154;
pub(super) const ASK_REXUANFENG_MOVE_CARD: u32 = 155;
pub(super) const ASK_REXUANFENG_MOVE_TO: u32 = 156;
pub(super) const ASK_AOCAI_CHOOSE_CARD: u32 = 157;

#[allow(clippy::too_many_arguments)]
pub(super) fn build_l11_npc_a_ask_b(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    let all = alive_all(core, owner);
    let others = alive_others(core, owner);
    let he = he_cards(core, owner);
    match ask_id {
        ASK_BENYU_MODE => {
            let src = target?;
            let mut tags = Vec::new();
            if core.player(owner).hand.len() < core.player(src).hand.len().min(5) {
                tags.push(0u8);
            }
            if he.len() > core.player(src).hand.len() {
                tags.push(1u8);
            }
            tag_ask(owner, "benyu:choose_mode", tags, true)
        }
        ASK_BENYU_DISCARD => {
            let src = target?;
            let need = core.player(src).hand.len() + 1;
            card_ask(owner, "benyu:discard_cards", he, need, need, false)
        }
        ASK_BMCANSHI_CANCEL_CONFIRM => confirm_ask(owner, "bmcanshi:cancel_confirm"),
        ASK_BMCANSHI_EXTRA_TARGETS => {
            let pool: Vec<PlayerId> = all
                .into_iter()
                .filter(|&p| !targets.contains(&p) && flag(core, p, "l11_kui_mark") > 0)
                .collect();
            let max = pool.len().max(1);
            player_ask(owner, "bmcanshi:extra_targets", pool, 1, max, true)
        }
        ASK_CHOUCE_CONFIRM => confirm_ask(owner, "chouce:confirm"),
        ASK_CHOUCE_DISCARD_TARGET => {
            let pool: Vec<PlayerId> = all
                .into_iter()
                .filter(|&p| !hej_cards(core, p).is_empty())
                .collect();
            player_ask(owner, "chouce:discard_target", pool, 1, 1, false)
        }
        ASK_CHOUCE_DISCARD_CARD => {
            let t = target?;
            card_ask(owner, "chouce:discard_card", hej_cards(core, t), 1, 1, false)
        }
        ASK_CHOUCE_DRAW_TARGET => player_ask(owner, "chouce:draw_target", all, 1, 1, false),
        ASK_DCMOUKUI_MODE => {
            let has_he = target.is_some_and(|t| !he_cards(core, t).is_empty());
            let tags: Vec<u8> = if has_he { vec![0, 1, 2] } else { vec![0] };
            tag_ask(owner, "dcmoukui:choose_mode", tags, true)
        }
        ASK_DCMOUKUI_DODGED_DISCARD => {
            let t = target?;
            card_ask(t, "dcmoukui:target_discard_owner", he, 1, 1, false)
        }
        ASK_GAOYUAN_TARGET => {
            let actor = *targets.first().unwrap_or(&owner);
            let pool: Vec<PlayerId> = others
                .into_iter()
                .filter(|&p| p != actor && flag(core, p, "l11_zhengjian_mark") > 0)
                .collect();
            player_ask(owner, "gaoyuan:choose_target", pool, 1, 1, true)
        }
        ASK_GAOYUAN_CARD => card_ask(owner, "gaoyuan:discard_card", he, 1, 1, false),
        ASK_JIEYUAN_MORE_CARD => {
            let pool: Vec<CardId> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| !is_red_card(catalog, core, owner, c))
                .collect();
            card_ask(owner, "jieyuan:more_card", pool, 1, 1, true)
        }
        ASK_JIEYUAN_LESS_CARD => {
            let pool: Vec<CardId> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| is_red_card(catalog, core, owner, c))
                .collect();
            card_ask(owner, "jieyuan:less_card", pool, 1, 1, true)
        }
        ASK_JSRGSHICHONG_CONFIRM => confirm_ask(owner, "jsrgshichong:confirm"),
        ASK_JSRGSHICHONG_STEAL => {
            let t = target?;
            card_ask(
                owner,
                "jsrgshichong:steal_card",
                core.player(t).hand.clone(),
                1,
                1,
                false,
            )
        }
        ASK_JSRGSHICHONG_GIVE => {
            let t = target?;
            card_ask(
                t,
                "jsrgshichong:give_card",
                core.player(t).hand.clone(),
                1,
                1,
                true,
            )
        }
        ASK_ZHIDIAN_AIM_EFFECT => {
            let last = flag(core, owner, "l11_kyou_zhidian_last");
            let tags: Vec<u8> = (1u8..=4).filter(|&i| u32::from(i) != last).collect();
            tag_ask(owner, "kyou_zhidian:choose_effect", tags, false)
        }
        ASK_LEIJI_TARGET => player_ask(owner, "leiji:choose_target", all, 1, 1, true),
        ASK_LIYU_CONFIRM_TARGET => {
            let t = target?;
            let pool: Vec<PlayerId> = core
                .alive_players()
                .filter(|&p| {
                    p != t
                        && p != owner
                        && catalog.ow_target_enabled(
                            core,
                            owner,
                            p,
                            None,
                            CardKind::Duel,
                            crate::event::DamageNature::Normal,
                        )
                })
                .collect();
            player_ask(t, "liyu:choose_duel_target", pool, 1, 1, true)
        }
        ASK_LIYU_GAIN_CARD => {
            let t = target?;
            card_ask(owner, "liyu:gain_card", he_cards(core, t), 1, 1, false)
        }
        ASK_NSXIANHAI_CONFIRM => confirm_ask(owner, "nsxianhai:confirm"),
        ASK_NSXIANHAI_SLOT => {
            let src = target?;
            let mut tags = Vec::new();
            for s in 1u8..=5 {
                let slot = match s {
                    1 => EquipSlot::Weapon,
                    2 => EquipSlot::Armor,
                    3 => EquipSlot::HorsePlus,
                    4 => EquipSlot::HorseMinus,
                    _ => EquipSlot::Treasure,
                };
                if catalog.is_equip_slot_enabled(core, src, slot) {
                    let tag = if s == 4 { 3 } else { s };
                    if !tags.contains(&tag) {
                        tags.push(tag);
                    }
                }
            }
            tag_ask(owner, "nsxianhai:choose_slot", tags, false)
        }
        ASK_SHICAI_CONFIRM => confirm_ask(owner, "nzry_shicai:confirm"),
        ASK_OLZHUIJI_MODE => {
            let t = target?;
            tag_ask(t, "olzhuiji:choose_mode", [0, 1], false)
        }
        ASK_OLZHUIJI_DISCARD => {
            let t = target?;
            card_ask(t, "olzhuiji:discard_card", he_cards(core, t), 1, 1, false)
        }
        ASK_QINGXIAN_TARGET => player_ask(owner, "qingxian:choose_target", others, 1, 1, true),
        ASK_QINGXIAN_MODE => {
            let t = target?;
            let mut tags = vec![0u8];
            if !core.player(t).equips.is_empty() {
                tags.push(1u8);
            }
            tag_ask(owner, "qingxian:choose_mode", tags, true)
        }
        ASK_QINGXIAN_DISCARD_EQUIP => {
            let t = target?;
            card_ask(
                t,
                "qingxian:discard_equip",
                core.player(t).equips.clone(),
                1,
                1,
                false,
            )
        }
        ASK_REDUODAO_DISCARD => card_ask(owner, "reduodao:discard_card", he, 1, 1, true),
        ASK_REPOJUN_CARDS => {
            let t = target?;
            let the = he_cards(core, t);
            let max_n = (core.player(t).hp.max(1) as usize).min(the.len());
            card_ask(owner, "repojun:choose_cards", the, 1, max_n, true)
        }
        ASK_RETIANXIANG_CARD => {
            let pool: Vec<CardId> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| card_suit(catalog, core, owner, c) == Suit::Heart)
                .collect();
            card_ask(owner, "retianxiang:discard_card", pool, 1, 1, true)
        }
        ASK_RETIANXIANG_TARGET => player_ask(owner, "retianxiang:choose_target", others, 1, 1, false),
        ASK_RETIANXIANG_MODE => tag_ask(owner, "retianxiang:choose_mode", [0, 1], false),
        ASK_REXINGLUAN_TARGET => {
            let cid = *cards.first()?;
            let rank = catalog.cards.get(cid as usize).map_or(0, |d| d.rank);
            let pool: Vec<PlayerId> = all
                .into_iter()
                .filter(|&p| {
                    core.player(p)
                        .equips
                        .iter()
                        .chain(core.player(p).judge.iter())
                        .any(|&c| catalog.cards.get(c as usize).is_some_and(|d| d.rank == rank))
                })
                .collect();
            player_ask(owner, "rexingluan:choose_target", pool, 1, 1, true)
        }
        ASK_REXINGLUAN_CONFIRM => confirm_ask(owner, "rexingluan:confirm"),
        ASK_REXINGLUAN_EJ_CARD => {
            let t = target?;
            let cid = *cards.first()?;
            let rank = catalog.cards.get(cid as usize).map_or(0, |d| d.rank);
            let pool: Vec<CardId> = core
                .player(t)
                .equips
                .iter()
                .chain(core.player(t).judge.iter())
                .copied()
                .filter(|&c| catalog.cards.get(c as usize).is_some_and(|d| d.rank == rank))
                .collect();
            card_ask(owner, "rexingluan:choose_card", pool, 1, 1, false)
        }
        ASK_REYAJIAO_CONFIRM => confirm_ask(owner, "reyajiao:confirm"),
        ASK_REYAJIAO_GIVE_TARGET => player_ask(owner, "reyajiao:give_target", all, 1, 1, true),
        ASK_REYAJIAO_DIFF_MODE => tag_ask(owner, "reyajiao:diff_mode", [0, 1], false),
        ASK_DIANCAI_CONFIRM => confirm_ask(owner, "diancai:confirm"),
        ASK_GNSHEQUE_SHA => {
            let pool: Vec<CardId> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| catalog.resolved_card_kind(core, c).is_sha_like())
                .collect();
            card_ask(owner, "gnsheque:choose_sha", pool, 1, 1, true)
        }
        ASK_JIGONG_CONFIRM => confirm_ask(owner, "jigong:confirm"),
        ASK_LIXIA_MODE => tag_ask(owner, "lixia:choose_mode", [0, 1], false),
        ASK_MOZHI_CARD => card_ask(
            owner,
            "mozhi:choose_card",
            core.player(owner).hand.clone(),
            1,
            1,
            true,
        ),
        ASK_MOZHI_TARGET => {
            let k_tag = *cards.get(1)?;
            let kind = CardKind::ALL.get(k_tag as usize).copied().unwrap_or(CardKind::Sha);
            let (_, _, pool) = core.legal_targets(catalog, kind, owner);
            let pool: Vec<PlayerId> = pool
                .into_iter()
                .filter(|&p| {
                    catalog.ow_target_enabled(
                        core,
                        owner,
                        p,
                        None,
                        kind,
                        crate::event::DamageNature::Normal,
                    )
                })
                .collect();
            player_ask(owner, "mozhi:choose_target", pool, 1, 1, true)
        }
        ASK_KUIZHU_MODE => tag_ask(owner, "nzry_kuizhu:choose_mode", [0, 1], true),
        ASK_KUIZHU_DRAW_TARGETS => {
            let x = flag(core, owner, "l11_kuizhu_discard_cnt").max(1) as usize;
            player_ask(
                owner,
                "nzry_kuizhu:draw_targets",
                all.clone(),
                1,
                x.min(all.len().max(1)),
                true,
            )
        }
        ASK_KUIZHU_DMG_TARGETS => {
            let x = flag(core, owner, "l11_kuizhu_discard_cnt").max(1) as i32;
            let pool: Vec<PlayerId> = all
                .into_iter()
                .filter(|&p| core.player(p).hp <= x)
                .collect();
            player_ask(owner, "nzry_kuizhu:damage_targets", pool.clone(), 1, pool.len().max(1), true)
        }
        ASK_PIAOLING_CONFIRM => confirm_ask(owner, "piaoling:confirm"),
        ASK_PIAOLING_GIVE_TARGET => player_ask(owner, "piaoling:choose_target", all, 1, 1, true),
        ASK_PIAOLING_SELF_DISCARD => card_ask(owner, "piaoling:discard_card", he, 1, 1, false),
        ASK_QINGZHONGX_CONFIRM => confirm_ask(owner, "qingzhongx:confirm"),
        ASK_QINGZHONGX_SWAP_TARGET => {
            let min_h = core
                .alive_players()
                .map(|p| core.player(p).hand.len())
                .min()
                .unwrap_or(0);
            let pool: Vec<PlayerId> = others
                .into_iter()
                .filter(|&p| core.player(p).hand.len() == min_h)
                .collect();
            player_ask(owner, "qingzhongx:choose_target", pool, 1, 1, false)
        }
        ASK_REGUANXING_CONFIRM => confirm_ask(owner, "reguanxing:confirm"),
        ASK_REGUANXING_TOP => {
            let peek: Vec<CardId> = core
                .vm_aux
                .piles
                .get(&(owner, "l11_reguanxing_peek".to_string()))
                .cloned()
                .unwrap_or_default();
            card_ask(owner, "reguanxing:to_top", peek.clone(), 0, peek.len(), false)
        }
        ASK_REJUECE_TARGET => {
            let pool: Vec<PlayerId> = others
                .into_iter()
                .filter(|&p| flag(core, p, "l11_lost_card_turn") > 0)
                .collect();
            player_ask(owner, "rejuece:choose_target", pool, 1, 1, true)
        }
        ASK_RELUOYI_CONFIRM => confirm_ask(owner, "reluoyi:confirm"),
        ASK_RESHUANGXIONG_CONFIRM => confirm_ask(owner, "reshuangxiong:confirm"),
        ASK_RESHUANGXIONG_PICK => {
            let peek = core
                .vm_aux
                .piles
                .get(&(owner, "l11_reshuangxiong_peek".to_string()))
                .cloned()
                .unwrap_or_default();
            card_ask(owner, "reshuangxiong:pick_card", peek, 1, 1, false)
        }
        ASK_RESHUANGXIONG_DUEL_CONFIRM => confirm_ask(owner, "reshuangxiong:gain_shas_confirm"),
        ASK_RETUXI_TARGETS => {
            let max_n = cards.first().copied().unwrap_or(1) as usize;
            let my_h = core.player(owner).hand.len();
            let pool: Vec<PlayerId> = others
                .into_iter()
                .filter(|&p| core.player(p).hand.len() >= my_h)
                .collect();
            player_ask(
                owner,
                "retuxi:choose_targets",
                pool.clone(),
                1,
                max_n.min(pool.len().max(1)),
                true,
            )
        }
        ASK_REZAIQI_TARGETS => {
            let max_n = flag(core, owner, "l11_rezaiqi_red_discard_turn").max(1) as usize;
            player_ask(
                owner,
                "rezaiqi:choose_targets",
                all.clone(),
                1,
                max_n.min(all.len().max(1)),
                true,
            )
        }
        ASK_REZAIQI_TARGET_MODE => {
            let t = target?;
            let mut tags = vec![0u8];
            if core.player(owner).hp < core.player(owner).max_hp as i32 {
                tags.push(1u8);
            }
            tag_ask(t, "rezaiqi:target_mode", tags, false)
        }
        ASK_CHENQING_TARGET => {
            let dying = target?;
            let pool: Vec<PlayerId> = others
                .into_iter()
                .filter(|&p| p != dying)
                .collect();
            player_ask(owner, "chenqing:choose_target", pool, 1, 1, true)
        }
        ASK_CHENQING_DISCARD => {
            let t = *targets.first()?;
            let the = he_cards(core, t);
            let n = 4usize.min(the.len());
            card_ask(t, "chenqing:discard_cards", the, n, n, false)
        }
        ASK_OLNIEPAN_CONFIRM => confirm_ask(owner, "olniepan:confirm"),
        ASK_OLNIEPAN_SKILL => {
            let mut tags = Vec::new();
            for (idx, sk) in ["bazhen", "olhuoji", "olkanpo"].iter().enumerate() {
                if !has_npc_a_skill(core, catalog, owner, sk) {
                    tags.push(idx as u8);
                }
            }
            tag_ask(owner, "olniepan:choose_skill", tags, false)
        }
        ASK_REXINGSHANG_MODE => {
            let dead = target?;
            let mut tags = Vec::new();
            if !he_cards(core, dead).is_empty() {
                tags.push(0u8);
            }
            if core.player(owner).hp < core.player(owner).max_hp as i32 {
                tags.push(1u8);
            }
            tag_ask(owner, "rexingshang:choose_mode", tags, true)
        }
        ASK_FENJI_CONFIRM => confirm_ask(owner, "fenji:confirm"),
        ASK_LIANGZHU_MODE => tag_ask(owner, "liangzhu:choose_mode", [0, 1], true),
        ASK_LIRANG_TARGET => player_ask(owner, "lirang:choose_target", others, 1, 1, true),
        ASK_LIRANG_CARDS => {
            let pool: Vec<CardId> = cards
                .iter()
                .copied()
                .filter(|c| core.discard.contains(c))
                .collect();
            card_ask(owner, "lirang:choose_cards", pool.clone(), 1, pool.len().max(1), true)
        }
        ASK_RELIANYING_TARGETS => {
            let n = (*cards.first().unwrap_or(&1) as usize).max(1);
            player_ask(
                owner,
                "relianying:choose_targets",
                all.clone(),
                1,
                n.min(all.len().max(1)),
                true,
            )
        }
        ASK_RENSHE_MODE => {
            let mut tags = vec![1u8];
            if !others.is_empty() {
                tags.insert(0, 0);
            }
            let my_f = core.effective_faction(catalog, owner);
            if others
                .iter()
                .any(|&p| core.effective_faction(catalog, p).is_some() && core.effective_faction(catalog, p) != my_f)
            {
                tags.push(2u8);
            }
            tag_ask(owner, "renshe:choose_mode", tags, true)
        }
        ASK_RENSHE_DRAW_TARGET => player_ask(owner, "renshe:draw_target", others, 1, 1, false),
        ASK_RENSHE_FACTION_TARGET => {
            let my_f = core.effective_faction(catalog, owner);
            let pool: Vec<PlayerId> = others
                .into_iter()
                .filter(|&p| core.effective_faction(catalog, p).is_some() && core.effective_faction(catalog, p) != my_f)
                .collect();
            player_ask(owner, "renshe:faction_target", pool, 1, 1, false)
        }
        ASK_REQUANJI_CONFIRM => confirm_ask(owner, "requanji:confirm"),
        ASK_REQUANJI_CARD => card_ask(
            owner,
            "requanji:choose_card",
            core.player(owner).hand.clone(),
            1,
            1,
            false,
        ),
        ASK_RETUNTIAN_CONFIRM => confirm_ask(owner, "retuntian:confirm"),
        ASK_REXUANFENG_MODE => {
            let can_move = others.iter().any(|&from| {
                core.player(from).equips.iter().any(|&eq| {
                    others
                        .iter()
                        .any(|&to| to != from && can_receive_moved_card(core, catalog, from, to, eq))
                })
            });
            let mut tags = Vec::new();
            if others.iter().any(|&p| !he_cards(core, p).is_empty()) {
                tags.push(0u8);
            }
            if can_move {
                tags.push(1u8);
            }
            tag_ask(owner, "rexuanfeng:mode", tags, true)
        }
        ASK_REXUANFENG_DISC_TARGET => {
            let pool: Vec<PlayerId> = others
                .into_iter()
                .filter(|&p| !he_cards(core, p).is_empty())
                .collect();
            player_ask(owner, "rexuanfeng:discard_target", pool, 1, 1, true)
        }
        ASK_REXUANFENG_DISC_CARD => {
            let t = target?;
            card_ask(owner, "rexuanfeng:card", he_cards(core, t), 1, 1, false)
        }
        ASK_REXUANFENG_MOVE_FROM => {
            let pool: Vec<PlayerId> = others
                .iter()
                .copied()
                .filter(|&f| {
                    core.player(f).equips.iter().any(|&eq| {
                        others
                            .iter()
                            .any(|&to| to != f && can_receive_moved_card(core, catalog, f, to, eq))
                    })
                })
                .collect();
            player_ask(owner, "rexuanfeng:move_from", pool, 1, 1, true)
        }
        ASK_REXUANFENG_MOVE_CARD => {
            let from = target?;
            let pool: Vec<CardId> = core
                .player(from)
                .equips
                .iter()
                .copied()
                .filter(|&eq| {
                    others
                        .iter()
                        .any(|&to| to != from && can_receive_moved_card(core, catalog, from, to, eq))
                })
                .collect();
            card_ask(owner, "rexuanfeng:move_card", pool, 1, 1, false)
        }
        ASK_REXUANFENG_MOVE_TO => {
            let from = target?;
            let eq = *cards.first()?;
            let pool: Vec<PlayerId> = others
                .into_iter()
                .filter(|&to| to != from && can_receive_moved_card(core, catalog, from, to, eq))
                .collect();
            player_ask(owner, "rexuanfeng:move_to", pool, 1, 1, false)
        }
        ASK_AOCAI_CHOOSE_CARD => {
            let need_tag = *cards.first().unwrap_or(&0);
            let pool: Vec<CardId> = core
                .vm_aux
                .piles
                .get(&(owner, "l11_aocai_peek".to_string()))
                .map(Vec::as_slice)
                .unwrap_or(&[])
                .iter()
                .copied()
                .filter(|&c| {
                    let k = catalog.card_kind(c);
                    match need_tag {
                        1 => k == CardKind::Shan,
                        2 => k.is_sha_like(),
                        4 => k == CardKind::Tao,
                        5 => matches!(k, CardKind::Tao | CardKind::Jiu),
                        _ => false,
                    }
                })
                .collect();
            spec_ask(
                owner,
                ChoiceKind::SelectCards,
                "aocai:choose_card",
                pool.into_iter().map(ChoiceOption::Card).collect(),
                1,
                1,
                true,
            )
        }
        _ => None,
    }
}
