//! Part B ask builders (IDs `1..=99`) sharing one authoritative spec with snapshots.

use super::super::l11_common::{alive_others, he_cards};
use super::helpers::{
    can_target_sha_no_range_npc, is_red_suit, pile_cards, SHEFU_NAMES,
};
use super::*;
use crate::catalog::{CardKind, Catalog};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::ids::{CardId, PlayerId};
use crate::state::GameStateCore;

fn mk(
    observer: PlayerId,
    kind: ChoiceKind,
    prompt: &'static str,
    options: Vec<ChoiceOption>,
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> Option<(PlayerId, ChoiceSpec)> {
    if options.is_empty() || (!allow_cancel && options.len() < min) {
        return None;
    }
    Some((
        observer,
        ChoiceSpec {
            kind,
            prompt,
            options,
            min,
            max,
            allow_cancel,
        },
    ))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_ask_a(
    ask_id: u32,
    _stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    _targets: &[PlayerId],
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    match ask_id {
        ASK_SATOMI_DISCARD_SHAN => {
            let t = target?;
            let opts: Vec<ChoiceOption> = core
                .player(t)
                .hand
                .iter()
                .copied()
                .filter(|&c| catalog.resolved_card_kind(core, c) == CardKind::Shan)
                .map(ChoiceOption::Card)
                .collect();
            mk(owner, ChoiceKind::SelectCards, "satomi_luodao:discard_shan", opts, 1, 1, false)
        }
        ASK_SATOMI_DISCARD_SELF => {
            let opts: Vec<ChoiceOption> = he_cards(core, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            mk(owner, ChoiceKind::SelectCards, "satomi_luodao:discard_self", opts, 1, 1, false)
        }
        ASK_SBLIEGONG_CONFIRM => mk(
            owner,
            ChoiceKind::Confirm,
            "sbliegong:confirm",
            vec![ChoiceOption::Confirm, ChoiceOption::Pass],
            1,
            1,
            true,
        ),
        ASK_SHANGYI_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !core.player(p).hand.is_empty())
                .map(ChoiceOption::Player)
                .collect();
            mk(owner, ChoiceKind::SelectPlayers, "shangyi:choose_target", opts, 1, 1, true)
        }
        ASK_SHANGYI_DISCARD_BLACK => {
            let t = target?;
            let opts: Vec<ChoiceOption> = core
                .player(t)
                .hand
                .iter()
                .copied()
                .filter(|&c| !is_red_suit(catalog.effective_card_suit(core, t, c)))
                .map(ChoiceOption::Card)
                .collect();
            mk(owner, ChoiceKind::SelectCards, "shangyi:discard_black", opts, 1, 1, false)
        }
        ASK_SHEFU_NAME => {
            let used_mask = super::helpers::flag(core, owner, "l11b_shefu_name_mask");
            let opts: Vec<ChoiceOption> = (0..SHEFU_NAMES.len() as u8)
                .filter(|&idx| (used_mask & (1u32 << idx)) == 0)
                .map(ChoiceOption::Tag)
                .collect();
            mk(owner, ChoiceKind::PickTag, "shefu:choose_name", opts, 0, 1, true)
        }
        ASK_SHEFU_CARD => {
            let opts: Vec<ChoiceOption> = he_cards(core, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            mk(owner, ChoiceKind::SelectCards, "shefu:choose_card", opts, 0, 1, true)
        }
        ASK_SHEFU2_CONFIRM => mk(
            owner,
            ChoiceKind::Confirm,
            "shefu2:confirm",
            vec![ChoiceOption::Confirm, ChoiceOption::Pass],
            1,
            1,
            true,
        ),
        ASK_SHENSU1_TARGET => {
            let opts: Vec<ChoiceOption> = super::helpers::usable_sha_targets_no_range(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            mk(owner, ChoiceKind::SelectPlayers, "shensu1:choose_target", opts, 0, 1, true)
        }
        ASK_SHENSU2_CARD => {
            let opts: Vec<ChoiceOption> = he_cards(core, owner)
                .into_iter()
                .filter(|&c| catalog.card_kind(c).equip_slot().is_some())
                .map(ChoiceOption::Card)
                .collect();
            mk(owner, ChoiceKind::SelectCards, "shensu2:choose_card", opts, 0, 1, true)
        }
        ASK_SHENSU2_TARGET => {
            let opts: Vec<ChoiceOption> = super::helpers::usable_sha_targets_no_range(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            mk(owner, ChoiceKind::SelectPlayers, "shensu2:choose_target", opts, 1, 1, true)
        }
        ASK_SHUNSHI_CARD => {
            let opts: Vec<ChoiceOption> = he_cards(core, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            mk(owner, ChoiceKind::SelectCards, "shunshi:choose_card", opts, 0, 1, true)
        }
        ASK_SHUNSHI_TARGET => {
            let excl = card.filter(|&v| v != u32::MAX).map(|v| v as PlayerId);
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| excl != Some(p))
                .map(ChoiceOption::Player)
                .collect();
            mk(owner, ChoiceKind::SelectPlayers, "shunshi:choose_target", opts, 1, 1, true)
        }
        ASK_SPSHIJI_CONFIRM => mk(
            owner,
            ChoiceKind::Confirm,
            "spshiji:confirm",
            vec![ChoiceOption::Confirm, ChoiceOption::Pass],
            1,
            1,
            true,
        ),
        ASK_SPWENJI_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !he_cards(core, p).is_empty())
                .map(ChoiceOption::Player)
                .collect();
            mk(owner, ChoiceKind::SelectPlayers, "spwenji:choose_target", opts, 0, 1, true)
        }
        ASK_SPWENJI_GIVE => {
            let t = target?;
            let opts: Vec<ChoiceOption> = he_cards(core, t)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            mk(t, ChoiceKind::SelectCards, "spwenji:target_give", opts, 1, 1, false)
        }
        ASK_SPYISHI_EQUIP => {
            let t = target?;
            let opts: Vec<ChoiceOption> = core
                .player(t)
                .equips
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            mk(owner, ChoiceKind::SelectCards, "spyishi:choose_equip", opts, 1, 1, false)
        }
        ASK_TIANMING_DISCARD => {
            let all = he_cards(core, owner);
            let n = all.len().min(2);
            let opts: Vec<ChoiceOption> = all.into_iter().map(ChoiceOption::Card).collect();
            mk(owner, ChoiceKind::SelectCards, "tianming:discard", opts, n, n, false)
        }
        ASK_TIANMING_OTHER_CONFIRM => {
            let t = target?;
            mk(
                t,
                ChoiceKind::Confirm,
                "tianming:other_confirm",
                vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                1,
                1,
                true,
            )
        }
        ASK_TIANMING_OTHER_DISCARD => {
            let t = target?;
            let all = he_cards(core, t);
            let n = all.len().min(2);
            let opts: Vec<ChoiceOption> = all.into_iter().map(ChoiceOption::Card).collect();
            mk(t, ChoiceKind::SelectCards, "tianming:other_discard", opts, n, n, false)
        }
        ASK_TIANSHU_CARD => {
            let opts: Vec<ChoiceOption> = he_cards(core, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            mk(owner, ChoiceKind::SelectCards, "tianshu:choose_card", opts, 0, 1, true)
        }
        ASK_TIANSHU_TARGET => {
            let opts: Vec<ChoiceOption> = super::super::l11_common::alive_all(core, owner)
                .into_iter()
                .filter(|&p| {
                    crate::rvm_ops::has_enabled_equip_slot(
                        core,
                        p,
                        crate::catalog::EquipSlot::Armor,
                    )
                })
                .map(ChoiceOption::Player)
                .collect();
            mk(owner, ChoiceKind::SelectPlayers, "tianshu:choose_target", opts, 1, 1, false)
        }
        ASK_TUIFENG_CARDS => {
            let max_n = card.unwrap_or(1) as usize;
            let all = he_cards(core, owner);
            let max_pick = max_n.min(all.len());
            let opts: Vec<ChoiceOption> = all.into_iter().map(ChoiceOption::Card).collect();
            mk(owner, ChoiceKind::SelectCards, "tuifeng:choose_cards", opts, 0, max_pick, true)
        }
        ASK_ZHANYI_DISCARD => {
            let opts: Vec<ChoiceOption> = he_cards(core, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            mk(owner, ChoiceKind::SelectCards, "zhanyi:discard", opts, 1, 1, true)
        }
        ASK_ZHANYI_BASIC_KIND => mk(
            owner,
            ChoiceKind::PickTag,
            "zhanyi_basic:choose_kind",
            vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1), ChoiceOption::Tag(2)],
            1,
            1,
            true,
        ),
        ASK_ZHANYI_BASIC_CARD => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| catalog.card_kind(c).is_basic_kind())
                .map(ChoiceOption::Card)
                .collect();
            mk(owner, ChoiceKind::SelectCards, "zhanyi_basic:choose_card", opts, 1, 1, true)
        }
        ASK_ZHANYI_BASIC_TARGET => {
            let k = match card.unwrap_or(0) {
                0 => CardKind::Sha,
                1 => CardKind::Jiu,
                _ => CardKind::Tao,
            };
            let opts: Vec<ChoiceOption> = if k == CardKind::Sha {
                alive_others(core, owner)
                    .into_iter()
                    .filter(|&p| {
                        core.in_attack_range(catalog, owner, p)
                            && can_target_sha_no_range_npc(core, catalog, owner, p)
                    })
                    .map(ChoiceOption::Player)
                    .collect()
            } else {
                vec![ChoiceOption::Player(owner)]
            };
            mk(owner, ChoiceKind::SelectPlayers, "zhanyi_basic:choose_target", opts, 1, 1, true)
        }
        ASK_ZHANYI_EQUIP_DISCARD => {
            let t = target?;
            let all = he_cards(core, t);
            let n = all.len().min(2);
            let opts: Vec<ChoiceOption> = all.into_iter().map(ChoiceOption::Card).collect();
            mk(t, ChoiceKind::SelectCards, "zhanyi_equip:target_discard", opts, n, n, false)
        }
        ASK_ZHENDU_DISCARD => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            mk(owner, ChoiceKind::SelectCards, "zhendu:discard", opts, 0, 1, true)
        }
        ASK_ZHIDAO_HAND => {
            let t = target?;
            let opts: Vec<ChoiceOption> = core
                .player(t)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            mk(owner, ChoiceKind::SelectCards, "zhidao:choose_hand", opts, 1, 1, false)
        }
        ASK_ZHIDAO_EQUIP => {
            let t = target?;
            let opts: Vec<ChoiceOption> = core
                .player(t)
                .equips
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            mk(owner, ChoiceKind::SelectCards, "zhidao:choose_equip", opts, 1, 1, false)
        }
        ASK_ZHIDAO_JUDGE => {
            let t = target?;
            let opts: Vec<ChoiceOption> = core
                .player(t)
                .judge
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            mk(owner, ChoiceKind::SelectCards, "zhidao:choose_judge", opts, 1, 1, false)
        }
        ASK_ZHIMAN_CARD => {
            let t = target?;
            let opts: Vec<ChoiceOption> = core
                .player(t)
                .equips
                .iter()
                .chain(core.player(t).judge.iter())
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            mk(owner, ChoiceKind::SelectCards, "zhiman:choose_card", opts, 1, 1, false)
        }
        ASK_ZHIYAN_TARGET => {
            let opts: Vec<ChoiceOption> = super::super::l11_common::alive_all(core, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            mk(owner, ChoiceKind::SelectPlayers, "zhiyan:choose_target", opts, 0, 1, true)
        }
        ASK_ZHONGYONG_TARGET => {
            let sha_t = target;
            let opts: Vec<ChoiceOption> = super::super::l11_common::alive_all(core, owner)
                .into_iter()
                .filter(|&p| Some(p) != sha_t)
                .map(ChoiceOption::Player)
                .collect();
            mk(owner, ChoiceKind::SelectPlayers, "zhongyong:choose_target", opts, 0, 1, true)
        }
        ASK_ZHONGYONG_USE_SHA => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| {
                    catalog.resolved_card_kind(core, c).is_sha_like()
                        || catalog
                            .find_player_vm_view_as(core, owner, c, true, Some(CardKind::Sha))
                            .is_some()
                })
                .map(ChoiceOption::Card)
                .collect();
            mk(owner, ChoiceKind::SelectCards, "zhongyong:use_sha", opts, 0, 1, true)
        }
        ASK_ZHUIDE_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            mk(owner, ChoiceKind::SelectPlayers, "zhuide:choose_target", opts, 0, 1, true)
        }
        ASK_ZHUIDE_BASICS => {
            let opts: Vec<ChoiceOption> = cards.iter().copied().map(ChoiceOption::Card).collect();
            let max_pick = opts.len().min(4);
            mk(owner, ChoiceKind::SelectCards, "zhuide:choose_basics", opts, 1.min(max_pick), max_pick, true)
        }
        ASK_ZILI_MODE => {
            let mut opts = vec![ChoiceOption::Tag(0)];
            if core.player(owner).hp < core.player(owner).max_hp as i32 {
                opts.push(ChoiceOption::Tag(1));
            }
            mk(owner, ChoiceKind::PickTag, "zili:choose_mode", opts, 1, 1, false)
        }
        ASK_PAIYI_TARGET => {
            let opts: Vec<ChoiceOption> = super::super::l11_common::alive_all(core, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            mk(owner, ChoiceKind::SelectPlayers, "paiyi:choose_target", opts, 1, 1, true)
        }
        ASK_PAIYI_PILE => {
            let opts: Vec<ChoiceOption> = pile_cards(core, owner, "l11b_quanji")
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            mk(owner, ChoiceKind::SelectCards, "paiyi:choose_pile", opts, 1, 1, true)
        }
        ASK_ZONGKUI_TARGET => {
            let is_round = card == Some(1);
            let cands: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| super::helpers::flag(core, p, "l11_kui_mark") == 0)
                .collect();
            let filtered = if is_round {
                let min_hp = cands.iter().map(|&p| core.player(p).hp).min().unwrap_or(i32::MAX);
                cands
                    .into_iter()
                    .filter(|&p| core.player(p).hp == min_hp)
                    .map(ChoiceOption::Player)
                    .collect()
            } else {
                cands.into_iter().map(ChoiceOption::Player).collect()
            };
            mk(
                owner,
                ChoiceKind::SelectPlayers,
                "zongkui:choose_target",
                filtered,
                usize::from(is_round),
                1,
                !is_round,
            )
        }
        ASK_SHIZHI_CARD => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| catalog.resolved_card_kind(core, c) == CardKind::Shan)
                .map(ChoiceOption::Card)
                .collect();
            mk(owner, ChoiceKind::SelectCards, "shizhi:choose_card", opts, 1, 1, true)
        }
        ASK_SHIZHI_TARGET => {
            let cid = card?;
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| {
                    super::helpers::card_usable_on_target_with_card(
                        core,
                        catalog,
                        owner,
                        p,
                        cid,
                        CardKind::Sha,
                        false,
                    )
                })
                .map(ChoiceOption::Player)
                .collect();
            mk(owner, ChoiceKind::SelectPlayers, "shizhi:choose_target", opts, 1, 1, true)
        }
        ASK_TWWUSHEN_CARD => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| catalog.effective_card_suit(core, owner, c) == crate::catalog::Suit::Heart)
                .map(ChoiceOption::Card)
                .collect();
            mk(owner, ChoiceKind::SelectCards, "twwushen:choose_card", opts, 1, 1, true)
        }
        ASK_TWWUSHEN_TARGET => {
            let opts: Vec<ChoiceOption> = super::helpers::usable_sha_targets_no_range(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            mk(owner, ChoiceKind::SelectPlayers, "twwushen:choose_target", opts, 1, 1, true)
        }
        ASK_MEIYINGQIANG_CARD => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| {
                    catalog.resolved_card_kind(core, c).is_sha_like()
                        || catalog
                            .find_player_vm_view_as(core, owner, c, true, Some(CardKind::Sha))
                            .is_some()
                })
                .map(ChoiceOption::Card)
                .collect();
            mk(owner, ChoiceKind::SelectCards, "meiyingqiang:choose_sha", opts, 0, 1, true)
        }
        ASK_MEIYINGQIANG_TARGET => {
            let cid = card?;
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| {
                    super::helpers::card_usable_on_target_with_card(
                        core,
                        catalog,
                        owner,
                        p,
                        cid,
                        CardKind::Sha,
                        false,
                    )
                })
                .map(ChoiceOption::Player)
                .collect();
            mk(owner, ChoiceKind::SelectPlayers, "meiyingqiang:choose_target", opts, 1, 1, true)
        }
        _ => None,
    }
}
