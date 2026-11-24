//! Part B ask builders (IDs `100..=202`) sharing one authoritative spec with snapshots.

use super::super::l11_common::{alive_all, alive_others, he_cards, hej_cards};
use super::helpers::{
    can_target_sha_no_range_npc, flag, is_weapon_card, pile_cards, suit_bit,
};
use super::*;
use crate::catalog::Catalog;
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
pub(super) fn build_ask_b(
    ask_id: u32,
    _stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    match ask_id {
        ASK_TWFENMING_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !he_cards(core, p).is_empty() || !core.player(p).linked)
                .map(ChoiceOption::Player)
                .collect();
            mk(owner, ChoiceKind::SelectPlayers, "twfenming:choose_target", opts, 0, 1, true)
        }
        ASK_TWFENMING_MODE => {
            let t = target?;
            let mut opts = Vec::new();
            if !he_cards(core, t).is_empty() {
                opts.push(ChoiceOption::Tag(0));
            }
            if !core.player(t).linked {
                opts.push(ChoiceOption::Tag(1));
            }
            if !core.player(owner).linked && !core.player(t).linked && !he_cards(core, t).is_empty()
            {
                opts.push(ChoiceOption::Tag(2));
            }
            mk(owner, ChoiceKind::PickTag, "twfenming:choose_mode", opts, 1, 1, true)
        }
        ASK_TWFENMING_DISCARD => {
            let t = target?;
            let opts: Vec<ChoiceOption> = he_cards(core, t).into_iter().map(ChoiceOption::Card).collect();
            mk(t, ChoiceKind::SelectCards, "twfenming:target_discard", opts, 1, 1, false)
        }
        ASK_TWHENGJIANG_CONFIRM => mk(
            owner,
            ChoiceKind::Confirm,
            "twhengjiang:confirm",
            vec![ChoiceOption::Confirm, ChoiceOption::Pass],
            1,
            1,
            true,
        ),
        ASK_TWHONGJU_SWAP_HAND => {
            let pile = pile_cards(core, owner, "l11b_twzhengrong");
            let max_pick = core.player(owner).hand.len().min(pile.len());
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            mk(owner, ChoiceKind::SelectCards, "twhongju:swap_hand", opts, 0, max_pick, true)
        }
        ASK_TWHONGJU_SWAP_PILE => {
            let need = card.unwrap_or(1) as usize;
            let opts: Vec<ChoiceOption> = pile_cards(core, owner, "l11b_twzhengrong")
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            mk(owner, ChoiceKind::SelectCards, "twhongju:swap_pile", opts, need, need, false)
        }
        ASK_TWHONGJU_SAOTAO => mk(
            owner,
            ChoiceKind::Confirm,
            "twhongju:gain_saotao",
            vec![ChoiceOption::Confirm, ChoiceOption::Pass],
            1,
            1,
            true,
        ),
        ASK_TWQINGCE_PILE => {
            let opts: Vec<ChoiceOption> = pile_cards(core, owner, "l11b_twzhengrong")
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            mk(owner, ChoiceKind::SelectCards, "twqingce:choose_pile", opts, 1, 1, true)
        }
        ASK_TWQINGCE_TARGET => {
            let opts: Vec<ChoiceOption> = alive_all(core, owner)
                .into_iter()
                .filter(|&p| !hej_cards(core, p).is_empty())
                .map(ChoiceOption::Player)
                .collect();
            mk(owner, ChoiceKind::SelectPlayers, "twqingce:choose_target", opts, 1, 1, true)
        }
        ASK_TWQINGCE_DISCARD => {
            let t = target?;
            let opts: Vec<ChoiceOption> = hej_cards(core, t).into_iter().map(ChoiceOption::Card).collect();
            mk(owner, ChoiceKind::SelectCards, "twqingce:discard_card", opts, 1, 1, false)
        }
        ASK_TWJIEFAN_TARGET => {
            let opts: Vec<ChoiceOption> = alive_all(core, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            mk(owner, ChoiceKind::SelectPlayers, "twjiefan:choose_target", opts, 1, 1, true)
        }
        ASK_TWJIEFAN_WEAPON => {
            let cur_idx = flag(core, owner, "l11b_jiefan_idx") as usize;
            let responder = *targets.get(cur_idx)?;
            let opts: Vec<ChoiceOption> = he_cards(core, responder)
                .into_iter()
                .filter(|&c| is_weapon_card(catalog, core, c))
                .map(ChoiceOption::Card)
                .collect();
            mk(responder, ChoiceKind::SelectCards, "twjiefan:discard_weapon", opts, 0, 1, true)
        }
        ASK_TWJUNTUN_TARGET => {
            let opts: Vec<ChoiceOption> = alive_all(core, owner)
                .into_iter()
                .filter(|&p| !super::super::l11_common::has_named_skill(core, catalog, p, "twxiongjun"))
                .map(ChoiceOption::Player)
                .collect();
            mk(owner, ChoiceKind::SelectPlayers, "twjuntun:choose_target", opts, 0, 1, true)
        }
        ASK_TWLIANCAI_TARGET => {
            let opts: Vec<ChoiceOption> = alive_all(core, owner)
                .into_iter()
                .filter(|&p| !core.player(p).equips.is_empty())
                .map(ChoiceOption::Player)
                .collect();
            mk(owner, ChoiceKind::SelectPlayers, "twliancai:choose_target", opts, 0, 1, true)
        }
        ASK_TWLIANCAI_EQUIP => {
            let t = target?;
            let opts: Vec<ChoiceOption> = core
                .player(t)
                .equips
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            mk(owner, ChoiceKind::SelectCards, "twliancai:gain_equip", opts, 1, 1, false)
        }
        ASK_TWLIANCAI_DRAW => mk(
            owner,
            ChoiceKind::Confirm,
            "twliancai:draw_confirm",
            vec![ChoiceOption::Confirm, ChoiceOption::Pass],
            1,
            1,
            true,
        ),
        ASK_TWLINGFA_MODE => mk(
            owner,
            ChoiceKind::Confirm,
            "twlingfa:choose_mode",
            vec![ChoiceOption::Confirm, ChoiceOption::Pass],
            1,
            1,
            true,
        ),
        ASK_TWLINGFA_SHA_DISCARD => {
            let t = target?;
            let opts: Vec<ChoiceOption> = he_cards(core, t).into_iter().map(ChoiceOption::Card).collect();
            mk(t, ChoiceKind::SelectCards, "twlingfa:sha_discard", opts, 0, 1, true)
        }
        ASK_TWLINGFA_TAO_GIVE => {
            let t = target?;
            let opts: Vec<ChoiceOption> = he_cards(core, t).into_iter().map(ChoiceOption::Card).collect();
            mk(t, ChoiceKind::SelectCards, "twlingfa:tao_give", opts, 0, 1, true)
        }
        ASK_TWZHIAN_MODE => {
            let mut opts = Vec::new();
            let on_field = card
                .and_then(|cid| super::phase_hooks::find_field_card_owner(core, cid))
                .is_some();
            if on_field {
                opts.push(ChoiceOption::Tag(0));
                if !core.player(owner).hand.is_empty() {
                    opts.push(ChoiceOption::Tag(1));
                }
            }
            if target.is_some_and(|t| core.player(t).alive) {
                opts.push(ChoiceOption::Tag(2));
            }
            mk(owner, ChoiceKind::PickTag, "twzhian:choose_mode", opts, 0, 1, true)
        }
        ASK_TWZHIAN_DISCARD_HAND => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            mk(owner, ChoiceKind::SelectCards, "twzhian:discard_hand", opts, 1, 1, false)
        }
        ASK_TWLINGLU_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            mk(owner, ChoiceKind::SelectPlayers, "twlinglu:choose_target", opts, 0, 1, true)
        }
        ASK_TWMOU_CARD => {
            let opts: Vec<ChoiceOption> = core
                .deck
                .iter()
                .copied()
                .filter(|&c| super::helpers::can_use_basic_in_play(core, catalog, owner, c))
                .map(ChoiceOption::Card)
                .collect();
            mk(owner, ChoiceKind::SelectCards, "twmouli:choose_card", opts, 1, 1, true)
        }
        ASK_TWMOU_TARGET => {
            let cid = card?;
            let k = catalog.card_kind(cid);
            let opts: Vec<ChoiceOption> = if k.is_sha_like() {
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
            mk(owner, ChoiceKind::SelectPlayers, "twmouli:choose_target", opts, 1, 1, true)
        }
        ASK_TWQIANXI_CONFIRM => mk(
            owner,
            ChoiceKind::Confirm,
            "twqianxi:confirm",
            vec![ChoiceOption::Confirm, ChoiceOption::Pass],
            1,
            1,
            true,
        ),
        ASK_TWQIANXI_DISCARD => {
            let opts: Vec<ChoiceOption> = he_cards(core, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            mk(owner, ChoiceKind::SelectCards, "twqianxi:discard", opts, 1, 1, false)
        }
        ASK_TWQIANXI_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| core.computed_distance(catalog, owner, p) <= 1)
                .map(ChoiceOption::Player)
                .collect();
            mk(owner, ChoiceKind::SelectPlayers, "twqianxi:choose_target", opts, 1, 1, false)
        }
        ASK_TWQIRANG_TARGET => {
            let opts: Vec<ChoiceOption> = cards.iter().map(|&c| ChoiceOption::Player(c as u8)).collect();
            mk(owner, ChoiceKind::SelectPlayers, "twqirang:modify_target", opts, 0, 1, true)
        }
        ASK_TWWUHUN_TARGETS => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| flag(core, p, "l11b_twwuhun_mark") > 0)
                .map(ChoiceOption::Player)
                .collect();
            let n = opts.len();
            mk(owner, ChoiceKind::SelectPlayers, "twwuhun:choose_targets", opts, 1, n.max(1), false)
        }
        ASK_TWXIONGXI_DISCARD => {
            let need = 5u32.saturating_sub(flag(core, owner, "baonvezhi")) as usize;
            let opts: Vec<ChoiceOption> = he_cards(core, owner).into_iter().map(ChoiceOption::Card).collect();
            mk(owner, ChoiceKind::SelectCards, "twxiongxi:discard", opts, need, need, true)
        }
        ASK_TWXIONGXI_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner).into_iter().map(ChoiceOption::Player).collect();
            mk(owner, ChoiceKind::SelectPlayers, "twxiongxi:choose_target", opts, 1, 1, true)
        }
        ASK_TWXIONGZHENG_ACTION => {
            let mut opts = Vec::new();
            if flag(core, owner, "l11b_xz_non_atk_mask") != 0 {
                opts.push(ChoiceOption::Tag(0));
            }
            if flag(core, owner, "l11b_xz_atk_mask") != 0 {
                opts.push(ChoiceOption::Tag(1));
            }
            mk(owner, ChoiceKind::PickTag, "twxiongzheng:choose_action", opts, 0, 1, true)
        }
        ASK_TWXIONGZHENG_SHA_TARGETS => {
            let mask = flag(core, owner, "l11b_xz_non_atk_mask");
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| (mask & (1u32 << p)) != 0)
                .map(ChoiceOption::Player)
                .collect();
            let n = opts.len().max(1);
            mk(owner, ChoiceKind::SelectPlayers, "twxiongzheng:choose_sha_target", opts, 0, n, true)
        }
        ASK_TWXIONGZHENG_DRAW_TARGETS => {
            let mask = flag(core, owner, "l11b_xz_atk_mask");
            let opts: Vec<ChoiceOption> = alive_all(core, owner)
                .into_iter()
                .filter(|&p| (mask & (1u32 << p)) != 0)
                .map(ChoiceOption::Player)
                .collect();
            let n = opts.len().max(1);
            mk(owner, ChoiceKind::SelectPlayers, "twxiongzheng:choose_draw_targets", opts, 0, n, true)
        }
        ASK_TWXIONGZHENG_TARGET => {
            let used = flag(core, owner, "l11b_xz_used_targets");
            let opts: Vec<ChoiceOption> = alive_all(core, owner)
                .into_iter()
                .filter(|&p| (used & (1u32 << p)) == 0)
                .map(ChoiceOption::Player)
                .collect();
            mk(owner, ChoiceKind::SelectPlayers, "twxiongzheng:choose_target", opts, 0, 1, true)
        }
        ASK_TWXUANHUO_CARDS => {
            let opts: Vec<ChoiceOption> = he_cards(core, owner).into_iter().map(ChoiceOption::Card).collect();
            mk(owner, ChoiceKind::SelectCards, "twxuanhuo:choose_cards", opts, 0, 2, true)
        }
        ASK_TWXUANHUO_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner).into_iter().map(ChoiceOption::Player).collect();
            mk(owner, ChoiceKind::SelectPlayers, "twxuanhuo:choose_target", opts, 1, 1, true)
        }
        ASK_TWXUANHUO_DUEL_TARGET => {
            let a = target?;
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| p != a)
                .map(ChoiceOption::Player)
                .collect();
            mk(owner, ChoiceKind::SelectPlayers, "twxuanhuo:choose_duel_target", opts, 1, 1, false)
        }
        ASK_TWXUANHUO_TARGET_CHOICE => {
            let a = target?;
            mk(
                a,
                ChoiceKind::PickTag,
                "twxuanhuo:target_choose",
                vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1), ChoiceOption::Tag(2)],
                1,
                1,
                false,
            )
        }
        ASK_TWXUANHUO_TAKE_CARDS => {
            let a = target?;
            let all = he_cards(core, a);
            let n = all.len().min(2);
            let opts: Vec<ChoiceOption> = all.into_iter().map(ChoiceOption::Card).collect();
            mk(owner, ChoiceKind::SelectCards, "twxuanhuo:take_cards", opts, n, n, false)
        }
        ASK_TWXUECHANG_TARGET | ASK_XINFENYUE_TARGET | ASK_XINFU_KANNAN_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| {
                    !core.player(p).hand.is_empty()
                        && (ask_id != ASK_XINFU_KANNAN_TARGET
                            || flag(core, p, "l11b_kannan_phase_used") != super::helpers::phase_stamp(core))
                })
                .map(ChoiceOption::Player)
                .collect();
            let prompt = match ask_id {
                ASK_TWXUECHANG_TARGET => "twxuechang:choose_target",
                ASK_XINFENYUE_TARGET => "xinfenyue:choose_target",
                _ => "xinfu_kannan:choose_target",
            };
            mk(owner, ChoiceKind::SelectPlayers, prompt, opts, 1, 1, true)
        }
        ASK_TWXUECHANG_PINDIAN_OWNER
        | ASK_XINFENYUE_PINDIAN_OWNER
        | ASK_XINFU_KANNAN_PINDIAN_OWNER => {
            let opts: Vec<ChoiceOption> = core.player(owner).hand.iter().copied().map(ChoiceOption::Card).collect();
            let prompt = match ask_id {
                ASK_TWXUECHANG_PINDIAN_OWNER => "twxuechang:pindian_owner",
                ASK_XINFENYUE_PINDIAN_OWNER => "xinfenyue:pindian_owner",
                _ => "xinfu_kannan:pindian_owner",
            };
            mk(owner, ChoiceKind::SelectCards, prompt, opts, 1, 1, false)
        }
        ASK_TWXUECHANG_PINDIAN_TARGET
        | ASK_XINFENYUE_PINDIAN_TARGET
        | ASK_XINFU_KANNAN_PINDIAN_TARGET => {
            let t = target?;
            let opts: Vec<ChoiceOption> = core.player(t).hand.iter().copied().map(ChoiceOption::Card).collect();
            let prompt = match ask_id {
                ASK_TWXUECHANG_PINDIAN_TARGET => "twxuechang:pindian_target",
                ASK_XINFENYUE_PINDIAN_TARGET => "xinfenyue:pindian_target",
                _ => "xinfu_kannan:pindian_target",
            };
            mk(t, ChoiceKind::SelectCards, prompt, opts, 1, 1, false)
        }
        ASK_TWXUECHANG_GAIN | ASK_XINFENYUE_GAIN => {
            let t = target?;
            let opts: Vec<ChoiceOption> = he_cards(core, t).into_iter().map(ChoiceOption::Card).collect();
            let prompt = if ask_id == ASK_TWXUECHANG_GAIN {
                "twxuechang:gain_card"
            } else {
                "xinfenyue:gain_card"
            };
            mk(owner, ChoiceKind::SelectCards, prompt, opts, 1, 1, false)
        }
        ASK_TWXUHE_TARGET_CHOOSE => {
            let t = target?;
            mk(
                t,
                ChoiceKind::PickTag,
                "twxuhe:target_choose",
                vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                1,
                1,
                false,
            )
        }
        ASK_TWYILIE_MODE => mk(
            owner,
            ChoiceKind::PickTag,
            "twyilie:choose_mode",
            vec![
                ChoiceOption::Tag(0),
                ChoiceOption::Tag(1),
                ChoiceOption::Tag(2),
            ],
            0,
            1,
            true,
        ),
        ASK_TWYINGJIA_CARD => {
            let opts: Vec<ChoiceOption> = he_cards(core, owner).into_iter().map(ChoiceOption::Card).collect();
            mk(owner, ChoiceKind::SelectCards, "twyingjia:choose_card", opts, 0, 1, true)
        }
        ASK_TWYINGJIA_TARGET => {
            let opts: Vec<ChoiceOption> = alive_all(core, owner).into_iter().map(ChoiceOption::Player).collect();
            mk(owner, ChoiceKind::SelectPlayers, "twyingjia:choose_target", opts, 1, 1, false)
        }
        ASK_TWYUANHU_EQUIP => {
            let opts: Vec<ChoiceOption> = he_cards(core, owner)
                .into_iter()
                .filter(|&c| catalog.card_slot_of(core, c).is_some())
                .map(ChoiceOption::Card)
                .collect();
            mk(owner, ChoiceKind::SelectCards, "twyuanhu:choose_equip", opts, 1, 1, true)
        }
        ASK_TWYUANHU_TARGET => {
            let cid = card?;
            let slot = catalog.card_slot_of(core, cid)?;
            let opts: Vec<ChoiceOption> = alive_all(core, owner)
                .into_iter()
                .filter(|&p| crate::rvm_ops::has_enabled_equip_slot(core, p, slot))
                .map(ChoiceOption::Player)
                .collect();
            mk(owner, ChoiceKind::SelectPlayers, "twyuanhu:choose_target", opts, 1, 1, true)
        }
        ASK_TWYUANHU_WEAPON_TGT => {
            let t = target?;
            let opts: Vec<ChoiceOption> = alive_all(core, owner)
                .into_iter()
                .filter(|&p| p != t && core.computed_distance(catalog, t, p) == 1 && !hej_cards(core, p).is_empty())
                .map(ChoiceOption::Player)
                .collect();
            mk(owner, ChoiceKind::SelectPlayers, "twyuanhu:weapon_target", opts, 1, 1, false)
        }
        ASK_TWYUANHU_WEAPON_DISCARD => {
            let vt = target?;
            let opts: Vec<ChoiceOption> = hej_cards(core, vt).into_iter().map(ChoiceOption::Card).collect();
            mk(owner, ChoiceKind::SelectCards, "twyuanhu:weapon_discard", opts, 1, 1, false)
        }
        ASK_TWYUHUA_GUANXING => {
            let opts: Vec<ChoiceOption> = cards.iter().copied().map(ChoiceOption::Card).collect();
            let n = opts.len();
            mk(owner, ChoiceKind::SelectCards, "twyuhua:guanxing_top", opts, 0, n, true)
        }
        ASK_TWYUHUA_DRAW => mk(
            owner,
            ChoiceKind::Confirm,
            "twyuhua:draw_confirm",
            vec![ChoiceOption::Confirm, ChoiceOption::Pass],
            1,
            1,
            true,
        ),
        ASK_TWZHENGRONG_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !he_cards(core, p).is_empty())
                .map(ChoiceOption::Player)
                .collect();
            mk(owner, ChoiceKind::SelectPlayers, "twzhengrong:choose_target", opts, 0, 1, true)
        }
        ASK_TWZHENGRONG_CARD => {
            let t = target?;
            let opts: Vec<ChoiceOption> = he_cards(core, t).into_iter().map(ChoiceOption::Card).collect();
            mk(owner, ChoiceKind::SelectCards, "twzhengrong:choose_card", opts, 1, 1, false)
        }
        ASK_WAISHI_CARDS => {
            let max_other_hand = alive_others(core, owner)
                .into_iter()
                .map(|p| core.player(p).hand.len())
                .max()
                .unwrap_or(0);
            let max_x = (super::helpers::distinct_alive_factions(core, catalog) as usize)
                .min(max_other_hand);
            let opts: Vec<ChoiceOption> = he_cards(core, owner).into_iter().map(ChoiceOption::Card).collect();
            let n = max_x.min(opts.len());
            mk(owner, ChoiceKind::SelectCards, "waishi:choose_cards", opts, 1, n, true)
        }
        ASK_WAISHI_TARGET => {
            let n = cards.len();
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| core.player(p).hand.len() >= n)
                .map(ChoiceOption::Player)
                .collect();
            mk(owner, ChoiceKind::SelectPlayers, "waishi:choose_target", opts, 1, 1, true)
        }
        ASK_WAISHI_TARGET_CARDS => {
            let t = target?;
            let n = cards.len();
            let opts: Vec<ChoiceOption> = core.player(t).hand.iter().copied().map(ChoiceOption::Card).collect();
            mk(owner, ChoiceKind::SelectCards, "waishi:choose_target_cards", opts, n, n, false)
        }
        ASK_WEIFENG_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| flag(core, p, "l11b_weifeng_kind") == 0)
                .map(ChoiceOption::Player)
                .collect();
            mk(owner, ChoiceKind::SelectPlayers, "weifeng:choose_target", opts, 1, 1, false)
        }
        ASK_WEIFENG3_GAIN => {
            let t = target?;
            let opts: Vec<ChoiceOption> = he_cards(core, t).into_iter().map(ChoiceOption::Card).collect();
            mk(owner, ChoiceKind::SelectCards, "weifeng3:gain_card", opts, 1, 1, false)
        }
        ASK_WEIJING_MODE => mk(
            owner,
            ChoiceKind::PickTag,
            "weijing:choose_mode",
            vec![ChoiceOption::Tag(0)],
            1,
            1,
            true,
        ),
        ASK_WEIJING_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| core.in_attack_range(catalog, owner, p) && can_target_sha_no_range_npc(core, catalog, owner, p))
                .map(ChoiceOption::Player)
                .collect();
            mk(owner, ChoiceKind::SelectPlayers, "weijing:choose_target", opts, 1, 1, true)
        }
        ASK_WULIE_TARGETS => {
            let max_n = core.player(owner).hp.max(0) as usize;
            let opts: Vec<ChoiceOption> = alive_others(core, owner).into_iter().map(ChoiceOption::Player).collect();
            let n = max_n.min(opts.len());
            mk(owner, ChoiceKind::SelectPlayers, "wulie:choose_targets", opts, 0, n, true)
        }
        ASK_XIANFU_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner).into_iter().map(ChoiceOption::Player).collect();
            mk(owner, ChoiceKind::SelectPlayers, "xianfu:choose_target", opts, 1, 1, false)
        }
        ASK_XINFU_DAIGONG_GIVE => {
            let t = target?;
            let mask = card.unwrap_or(0);
            let opts: Vec<ChoiceOption> = he_cards(core, t)
                .into_iter()
                .filter(|&c| (mask & suit_bit(catalog.effective_card_suit(core, t, c))) == 0)
                .map(ChoiceOption::Card)
                .collect();
            mk(t, ChoiceKind::SelectCards, "xinfu_daigong:give_card", opts, 0, 1, true)
        }
        ASK_XINFU_ZENGDAO_CARDS => {
            let opts: Vec<ChoiceOption> = core.player(owner).equips.iter().copied().map(ChoiceOption::Card).collect();
            let n = opts.len().max(1);
            mk(owner, ChoiceKind::SelectCards, "xinfu_zengdao:choose_cards", opts, 1, n, true)
        }
        ASK_XINFU_ZENGDAO_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner).into_iter().map(ChoiceOption::Player).collect();
            mk(owner, ChoiceKind::SelectPlayers, "xinfu_zengdao:choose_target", opts, 1, 1, true)
        }
        ASK_XINFU_ZHAOXIN_CARDS => {
            let have = pile_cards(core, owner, "l11b_zhaoxin").len();
            let max_pick = 3usize.saturating_sub(have).min(core.player(owner).hand.len());
            let opts: Vec<ChoiceOption> = core.player(owner).hand.iter().copied().map(ChoiceOption::Card).collect();
            mk(owner, ChoiceKind::SelectCards, "xinfu_zhaoxin:choose_cards", opts, 1, max_pick, true)
        }
        ASK_XINFU_ZHAOXIN_PILE => {
            let opts: Vec<ChoiceOption> = pile_cards(core, owner, "l11b_zhaoxin")
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            mk(owner, ChoiceKind::SelectCards, "xinfu_zhaoxin:choose_pile", opts, 0, 1, true)
        }
        ASK_XINFU_ZHAOXIN_TARGET_GAIN => {
            let t = target?;
            mk(
                t,
                ChoiceKind::Confirm,
                "xinfu_zhaoxin:target_gain",
                vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                1,
                1,
                true,
            )
        }
        ASK_XINFU_ZHAOXIN_DAMAGE => mk(
            owner,
            ChoiceKind::Confirm,
            "xinfu_zhaoxin:damage_confirm",
            vec![ChoiceOption::Confirm, ChoiceOption::Pass],
            1,
            1,
            true,
        ),
        ASK_XINJIEWEI_DISCARD => {
            let opts: Vec<ChoiceOption> = he_cards(core, owner).into_iter().map(ChoiceOption::Card).collect();
            mk(owner, ChoiceKind::SelectCards, "xinjiewei_move:discard", opts, 0, 1, true)
        }
        ASK_XINJIEWEI_SRC | ASK_YONGJIN_SRC => {
            let only_equip = ask_id == ASK_YONGJIN_SRC;
            let opts: Vec<ChoiceOption> = alive_all(core, owner)
                .into_iter()
                .filter(|&src| {
                    let cs: Vec<CardId> = if only_equip {
                        core.player(src).equips.clone()
                    } else {
                        core.player(src)
                            .equips
                            .iter()
                            .chain(core.player(src).judge.iter())
                            .copied()
                            .collect()
                    };
                    cs.into_iter().any(|c| {
                        (!only_equip || !cards.contains(&c))
                            && alive_all(core, owner).into_iter().any(|dst| {
                                dst != src && super::helpers::can_move_card_to(core, catalog, c, dst)
                            })
                    })
                })
                .map(ChoiceOption::Player)
                .collect();
            let prompt = if only_equip {
                "yongjin:choose_src"
            } else {
                "xinjiewei_move:choose_src"
            };
            mk(owner, ChoiceKind::SelectPlayers, prompt, opts, 0, 1, true)
        }
        ASK_XINJIEWEI_CARD | ASK_YONGJIN_CARD => {
            let src = target?;
            let only_equip = ask_id == ASK_YONGJIN_CARD;
            let mut cs: Vec<CardId> = core.player(src).equips.clone();
            if !only_equip {
                cs.extend(core.player(src).judge.iter().copied());
            }
            let opts: Vec<ChoiceOption> = cs
                .into_iter()
                .filter(|&c| {
                    (!only_equip || !cards.contains(&c))
                        && alive_all(core, owner)
                            .into_iter()
                            .any(|dst| dst != src && super::helpers::can_move_card_to(core, catalog, c, dst))
                })
                .map(ChoiceOption::Card)
                .collect();
            let prompt = if only_equip {
                "yongjin:choose_card"
            } else {
                "xinjiewei_move:choose_card"
            };
            mk(owner, ChoiceKind::SelectCards, prompt, opts, 1, 1, false)
        }
        ASK_XINJIEWEI_DST | ASK_YONGJIN_DST => {
            let src = target?;
            let cid = card?;
            let opts: Vec<ChoiceOption> = alive_all(core, owner)
                .into_iter()
                .filter(|&dst| dst != src && super::helpers::can_move_card_to(core, catalog, cid, dst))
                .map(ChoiceOption::Player)
                .collect();
            let prompt = if ask_id == ASK_YONGJIN_DST {
                "yongjin:choose_dst"
            } else {
                "xinjiewei_move:choose_dst"
            };
            mk(owner, ChoiceKind::SelectPlayers, prompt, opts, 1, 1, false)
        }
        ASK_XINJUSHOU_CARD => {
            let opts: Vec<ChoiceOption> = core.player(owner).hand.iter().copied().map(ChoiceOption::Card).collect();
            mk(owner, ChoiceKind::SelectCards, "xinjushou:choose_card", opts, 1, 1, false)
        }
        ASK_XINKUANGFU_TARGET => {
            let opts: Vec<ChoiceOption> = alive_all(core, owner)
                .into_iter()
                .filter(|&p| !core.player(p).equips.is_empty())
                .map(ChoiceOption::Player)
                .collect();
            mk(owner, ChoiceKind::SelectPlayers, "xinkuangfu:choose_target", opts, 1, 1, true)
        }
        ASK_XINKUANGFU_DISCARD_EQUIP => {
            let t = target?;
            let opts: Vec<ChoiceOption> = core.player(t).equips.iter().copied().map(ChoiceOption::Card).collect();
            mk(owner, ChoiceKind::SelectCards, "xinkuangfu:discard_equip", opts, 1, 1, false)
        }
        ASK_XINKUANGFU_SHA_TARGET => {
            let opts: Vec<ChoiceOption> = super::helpers::usable_sha_targets_no_range(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            mk(owner, ChoiceKind::SelectPlayers, "xinkuangfu:use_sha_target", opts, 1, 1, true)
        }
        ASK_XINKUANGFU_DISCARD_TWO => {
            let n = core.player(owner).hand.len().min(2);
            let opts: Vec<ChoiceOption> = core.player(owner).hand.iter().copied().map(ChoiceOption::Card).collect();
            mk(owner, ChoiceKind::SelectCards, "xinkuangfu:discard_two", opts, n, n, false)
        }
        ASK_XINSHENG_CONFIRM => mk(
            owner,
            ChoiceKind::Confirm,
            "xinsheng:confirm",
            vec![ChoiceOption::Confirm, ChoiceOption::Pass],
            1,
            1,
            true,
        ),
        ASK_YECHOU_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| (core.player(p).max_hp as i32 - core.player(p).hp) > 1)
                .map(ChoiceOption::Player)
                .collect();
            mk(owner, ChoiceKind::SelectPlayers, "yechou:choose_target", opts, 0, 1, true)
        }
        ASK_YINBING_CARDS => {
            let opts: Vec<ChoiceOption> = he_cards(core, owner)
                .into_iter()
                .filter(|&c| !catalog.resolved_card_kind(core, c).is_basic_kind())
                .map(ChoiceOption::Card)
                .collect();
            let n = opts.len().max(1);
            mk(owner, ChoiceKind::SelectCards, "yinbing:choose_cards", opts, 0, n, true)
        }
        ASK_JUEDI_TARGET => {
            let ohp = core.player(owner).hp;
            let opts: Vec<ChoiceOption> = alive_all(core, owner)
                .into_iter()
                .filter(|&p| core.player(p).hp <= ohp)
                .map(ChoiceOption::Player)
                .collect();
            mk(owner, ChoiceKind::SelectPlayers, "juedi:choose_target", opts, 1, 1, false)
        }
        ASK_YISE_RECOVER => {
            let _t = target?;
            mk(
                owner,
                ChoiceKind::Confirm,
                "yise:recover_confirm",
                vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                1,
                1,
                true,
            )
        }
        ASK_YJJIANSHU_CARD => {
            let opts: Vec<ChoiceOption> = core.player(owner).hand.iter().copied().map(ChoiceOption::Card).collect();
            mk(owner, ChoiceKind::SelectCards, "yjjianshu:choose_card", opts, 1, 1, true)
        }
        ASK_YJJIANSHU_TARGETS => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner).into_iter().map(ChoiceOption::Player).collect();
            mk(owner, ChoiceKind::SelectPlayers, "yjjianshu:choose_targets", opts, 2, 2, true)
        }
        ASK_YJJIANSHU_PINDIAN_1 | ASK_YJJIANSHU_PINDIAN_2 => {
            let t = if ask_id == ASK_YJJIANSHU_PINDIAN_1 {
                *targets.first()?
            } else {
                *targets.get(1)?
            };
            let opts: Vec<ChoiceOption> = core.player(t).hand.iter().copied().map(ChoiceOption::Card).collect();
            let prompt = if ask_id == ASK_YJJIANSHU_PINDIAN_1 {
                "yjjianshu:pindian_first"
            } else {
                "yjjianshu:pindian_second"
            };
            mk(t, ChoiceKind::SelectCards, prompt, opts, 1, 1, false)
        }
        ASK_YONGSI2_DISCARD => {
            let need = (super::helpers::distinct_alive_factions(core, catalog) as usize)
                .min(he_cards(core, owner).len());
            let opts: Vec<ChoiceOption> = he_cards(core, owner).into_iter().map(ChoiceOption::Card).collect();
            mk(owner, ChoiceKind::SelectCards, "yongsi2:discard", opts, need, need, false)
        }
        ASK_YUCE_SHOW => {
            let opts: Vec<ChoiceOption> = core.player(owner).hand.iter().copied().map(ChoiceOption::Card).collect();
            mk(owner, ChoiceKind::SelectCards, "yuce:show_card", opts, 0, 1, true)
        }
        ASK_YUCE_SOURCE_DISCARD => {
            let t = target?;
            let shown_group = card.unwrap_or(0) as u8;
            let opts: Vec<ChoiceOption> = core
                .player(t)
                .hand
                .iter()
                .copied()
                .filter(|&c| super::helpers::card_type_group(catalog.resolved_card_kind(core, c)) != shown_group)
                .map(ChoiceOption::Card)
                .collect();
            mk(t, ChoiceKind::SelectCards, "yuce:source_discard", opts, 0, 1, true)
        }
        _ => None,
    }
}
