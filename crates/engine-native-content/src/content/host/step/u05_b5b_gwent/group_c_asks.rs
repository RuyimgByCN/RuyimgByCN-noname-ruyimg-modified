//! U05-5b Batch C 询问 `ChoiceSpec` 构建与快照恢复共享模块。

use super::*;

pub(super) const ASK_LINGJI_DISCARD: u32 = 36;
pub(super) const ASK_GWSHENYU_TARGET: u32 = 37;
pub(super) const ASK_GWSHENYU_MODE: u32 = 38;
pub(super) const ASK_GWSHENYU_PICK: u32 = 39;
pub(super) const ASK_JUNCHI_USE: u32 = 40;
pub(super) const ASK_HUPENG_CARD: u32 = 41;
pub(super) const ASK_HUPENG_TARGET: u32 = 42;
pub(super) const ASK_HUPENG_PICK: u32 = 43;
pub(super) const ASK_HUPENG4_CONFIRM: u32 = 44;
pub(super) const ASK_HUNMO_TARGET: u32 = 45;
pub(super) const ASK_HUNMO_PICK: u32 = 46;
pub(super) const ASK_HUNMO_DISCARD_SELF: u32 = 47;
pub(super) const ASK_HUNMO_DISCARD_OTHER: u32 = 48;
pub(super) const ASK_SHUIJIAN_COST: u32 = 49;
pub(super) const ASK_YINZHANG_COST: u32 = 50;
pub(super) const ASK_YINZHANG_PICK: u32 = 51;
pub(super) const ASK_GWTIANBIAN_MODE: u32 = 52;
pub(super) const ASK_GWXIAOSHOU_COST: u32 = 53;
pub(super) const ASK_GWXIAOSHOU_TARGET: u32 = 54;
pub(super) const ASK_GWJIQUAN_TARGETS: u32 = 55;
pub(super) const ASK_GWJIQUAN_STEAL: u32 = 56;

pub(super) fn shenyu_discard_spells(core: &GameStateCore, catalog: &Catalog) -> Vec<CardId> {
    core.discard
        .iter()
        .copied()
        .filter(|&c| {
            let n = clean_gwent_name(catalog.cards[c as usize].name);
            let is_non_gold_spell = SILVER_SPELL_PICK_NAMES.contains(&n)
                || BRONZE_SPELL_NAMES.iter().any(|&(bn, _)| bn == n);
            is_non_gold_spell && flag(core, 0, &format!("gw_shenyu_vanishtag:{c}")) == 0
        })
        .collect()
}

pub(super) fn junchi_playable_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    target: PlayerId,
) -> Vec<ChoiceOption> {
    if !core.player(target).alive || owner == target || is_target_hidden(core, target) {
        return Vec::new();
    }
    core.player(owner)
        .hand
        .iter()
        .copied()
        .filter(|&c| {
            let name = clean_gwent_name(catalog.cards[c as usize].name);
            let kind = catalog.card_kind(c);
            if matches!(
                name,
                "gw_dieyi"
                    | "gwjinli_jiu"
                    | "gw_fuyuan"
                    | "gw_dudayuanshuai1"
                    | "gw_dudayuanshuai2"
                    | "gw_baishuang"
                    | "gw_baobaoshu"
                    | "gwmaoxian_yioufeisi"
                    | "gwmaoxian_fulisi"
                    | "gwmaoxian_kaerweite"
                    | "gwmaoxian_fuertaisite"
            ) {
                return false;
            }
            if is_sha_or_wuyao(catalog, c) {
                return is_single_target_legal_no_dist(core, catalog, owner, c, target);
            }
            if kind.is_equip()
                || kind.aoe_need().is_some()
                || matches!(
                    kind,
                    CardKind::Shan
                        | CardKind::Nullify
                        | CardKind::Tao
                        | CardKind::Jiu
                        | CardKind::Wuzhong
                        | CardKind::Taoyuan
                        | CardKind::Wugu
                        | CardKind::ShanDian
                        | CardKind::HongShui
                        | CardKind::Jiedao
                        | CardKind::Poxiao
                        | CardKind::Shanbengshu
                        | CardKind::Wenyi
                )
            {
                return false;
            }
            if matches!(
                kind,
                CardKind::Qinpendayu
                    | CardKind::Birinongwu
                    | CardKind::Ciguhanshuang
                    | CardKind::Baoxueyaoshui
                    | CardKind::Zhihuanjun
                    | CardKind::Zumoshoukao
                    | CardKind::Aozuzhilei
                    | CardKind::Kunenfayin
                    | CardKind::Yanziyaoshui
            ) {
                return b5b_card_targets(core, catalog, owner, c)
                    .is_some_and(|(opts, _, _)| opts.contains(&ChoiceOption::Player(target)));
            }
            is_single_target_legal_no_dist(core, catalog, owner, c, target)
        })
        .map(ChoiceOption::Card)
        .collect()
}

pub(super) fn hunmo_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<ChoiceOption> {
    let small_hand = core.player(owner).hand.len() < 2;
    core.seat_order_from(owner)
        .into_iter()
        .filter(|&p| {
            p != owner
                && core.player(p).alive
                && !has_named_skill(core, catalog, p, "hunmo2")
                && if small_hand {
                    core.player(p).hand.len() < 2
                } else {
                    core.player(p).hand.len() >= 2
                        && core.player(p).hand.iter().any(|&c| {
                            !crate::kernel::is_gw_dudayuanshuai2_locked(core, catalog, p, c)
                        })
                }
        })
        .map(ChoiceOption::Player)
        .collect()
}

pub(super) fn jiquan_targets(core: &GameStateCore, owner: PlayerId) -> Vec<ChoiceOption> {
    core.seat_order_from(owner)
        .into_iter()
        .filter(|&p| p != owner && core.player(p).alive && !hand_and_equips(core, p).is_empty())
        .map(ChoiceOption::Player)
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_ask_group_c(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    _targets: &[PlayerId],
    _card: Option<u32>,
    _cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    match ask_id {
        ASK_LINGJI_DISCARD => {
            let opts: Vec<ChoiceOption> = hand_and_equips(core, owner)
                .into_iter()
                .filter(|&c| !crate::kernel::is_gw_dudayuanshuai2_locked(core, catalog, owner, c))
                .map(ChoiceOption::Card)
                .collect();
            let need = opts.len().min(2);
            Some(ask_spec(
                owner,
                "lingji_discard",
                ChoiceKind::SelectCards,
                opts,
                need,
                need,
                false,
            ))
        }
        ASK_GWSHENYU_TARGET => {
            let any_spell = !shenyu_discard_spells(core, catalog).is_empty();
            let opts: Vec<ChoiceOption> = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| {
                    core.player(p).alive && (any_spell || core.player(p).hp < core.player(p).max_hp as i32)
                })
                .map(ChoiceOption::Player)
                .collect();
            Some(ask_spec(
                owner,
                "gwshenyu_target",
                ChoiceKind::SelectPlayers,
                opts,
                1,
                1,
                true,
            ))
        }
        ASK_GWSHENYU_MODE => {
            let t = target?;
            Some(ask_spec(
                t,
                "gwshenyu_mode",
                ChoiceKind::PickTag,
                vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                1,
                1,
                false,
            ))
        }
        ASK_GWSHENYU_PICK => {
            let t = target?;
            let opts: Vec<ChoiceOption> = shenyu_discard_spells(core, catalog)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            Some(ask_spec(
                t,
                "gwshenyu_pick",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                false,
            ))
        }
        ASK_JUNCHI_USE => {
            let t = target?;
            Some(ask_spec(
                owner,
                "junchi_use",
                ChoiceKind::SelectCards,
                junchi_playable_cards(core, catalog, owner, t),
                1,
                1,
                true,
            ))
        }
        ASK_HUPENG_CARD => {
            let opts: Vec<ChoiceOption> = discardable_hand_and_equips(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            Some(ask_spec(
                owner,
                "hupeng_card",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                true,
            ))
        }
        ASK_HUPENG_TARGET => {
            let opts: Vec<ChoiceOption> = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| core.player(p).alive)
                .map(ChoiceOption::Player)
                .collect();
            Some(ask_spec(
                owner,
                "hupeng_target",
                ChoiceKind::SelectPlayers,
                opts,
                1,
                1,
                false,
            ))
        }
        ASK_HUPENG_PICK => Some(ask_spec(
            owner,
            "hupeng_pick",
            ChoiceKind::PickTag,
            vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
            1,
            1,
            false,
        )),
        ASK_HUPENG4_CONFIRM => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| catalog.cards[c as usize].name == "gw_dudayuanshuai1")
                .map(ChoiceOption::Card)
                .collect();
            Some(ask_spec(
                owner,
                "hupeng4_confirm",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                true,
            ))
        }
        ASK_HUNMO_TARGET => Some(ask_spec(
            owner,
            "hunmo_target",
            ChoiceKind::SelectPlayers,
            hunmo_targets(core, catalog, owner),
            1,
            1,
            true,
        )),
        ASK_HUNMO_PICK => {
            let opts: Vec<ChoiceOption> = _cards.iter().copied().map(ChoiceOption::Card).collect();
            Some(ask_spec(
                owner,
                "hunmo_pick",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                false,
            ))
        }
        ASK_HUNMO_DISCARD_SELF => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| {
                    !crate::kernel::is_gw_dudayuanshuai2_locked(core, catalog, owner, c)
                })
                .map(ChoiceOption::Card)
                .collect();
            Some(ask_spec(
                owner,
                "hunmo_discard_self",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                false,
            ))
        }
        ASK_HUNMO_DISCARD_OTHER => {
            let t = target?;
            let opts: Vec<ChoiceOption> = core
                .player(t)
                .hand
                .iter()
                .copied()
                .filter(|&c| !crate::kernel::is_gw_dudayuanshuai2_locked(core, catalog, t, c))
                .map(ChoiceOption::Card)
                .collect();
            Some(ask_spec(
                owner,
                "hunmo_discard_other",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                false,
            ))
        }
        ASK_SHUIJIAN_COST => {
            let opts: Vec<ChoiceOption> = discardable_hand(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            Some(ask_spec(
                owner,
                "shuijian_cost",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                true,
            ))
        }
        ASK_YINZHANG_COST => {
            let opts: Vec<ChoiceOption> = discardable_hand_and_equips(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            Some(ask_spec(
                owner,
                "yinzhang_cost",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                true,
            ))
        }
        ASK_YINZHANG_PICK => {
            let n = flag(core, owner, "gw_yinzhang_n");
            let mut opts = Vec::new();
            for i in 0..n {
                let t = flag(core, owner, &format!("gw_yinzhang_opt:{i}"));
                if t > 0 {
                    opts.push(ChoiceOption::Tag((t - 1) as u8));
                }
            }
            Some(ask_spec(
                owner,
                "yinzhang_pick",
                ChoiceKind::PickTag,
                opts,
                1,
                1,
                false,
            ))
        }
        ASK_GWTIANBIAN_MODE => Some(ask_spec(
            owner,
            "gwtianbian_mode",
            ChoiceKind::PickTag,
            vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
            1,
            1,
            true,
        )),
        ASK_GWXIAOSHOU_COST => {
            let opts: Vec<ChoiceOption> = discardable_hand_and_equips(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            Some(ask_spec(
                owner,
                "gwxiaoshou_cost",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                true,
            ))
        }
        ASK_GWXIAOSHOU_TARGET => {
            let max_hp = core
                .alive_players()
                .map(|p| core.player(p).hp)
                .max()
                .unwrap_or(0);
            let opts: Vec<ChoiceOption> = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| core.player(p).alive && core.player(p).hp == max_hp)
                .map(ChoiceOption::Player)
                .collect();
            Some(ask_spec(
                owner,
                "gwxiaoshou_target",
                ChoiceKind::SelectPlayers,
                opts,
                1,
                1,
                false,
            ))
        }
        ASK_GWJIQUAN_TARGETS => {
            let opts = jiquan_targets(core, owner);
            let max = opts.len();
            Some(ask_spec(
                owner,
                "gwjiquan_targets",
                ChoiceKind::SelectPlayers,
                opts,
                1,
                max,
                true,
            ))
        }
        ASK_GWJIQUAN_STEAL => {
            let t = target?;
            let opts: Vec<ChoiceOption> = hand_and_equips(core, t)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            Some(ask_spec(
                owner,
                "gwjiquan_steal",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                false,
            ))
        }
        _ => None,
    }
}
