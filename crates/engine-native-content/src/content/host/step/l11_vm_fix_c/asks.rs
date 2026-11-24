//! Prompt builders (`ChoiceSpec`) for `HostBehavior::L11VmFixC`.

use super::helpers::*;
use super::*;
use crate::catalog::{CardKind, Catalog, Suit};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::ids::{CardId, PlayerId};
use crate::state::GameStateCore;

fn spec(
    chooser: PlayerId,
    kind: ChoiceKind,
    prompt: &'static str,
    options: Vec<ChoiceOption>,
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> Option<(PlayerId, ChoiceSpec)> {
    (!options.is_empty() && options.len() >= min).then_some((
        chooser,
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

fn confirm_ask(chooser: PlayerId, prompt: &'static str) -> Option<(PlayerId, ChoiceSpec)> {
    spec(
        chooser,
        ChoiceKind::Confirm,
        prompt,
        vec![ChoiceOption::Confirm],
        0,
        1,
        true,
    )
}

pub(super) fn has_custom_cancel(ask_id: u32) -> bool {
    matches!(
        ask_id,
        ASK_REQIANGXI_WEAPON | ASK_QIAOSHUI3_ADD_TARGET | ASK_RETIEJI_DISCARD
    )
}

pub(super) fn ask_l11_vm_c(
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<crate::snap::ExpectedAsk> {
    let ask_id = flag(core, owner, "l11c_ask_id");
    if ask_id == 0 {
        return None;
    }
    let (observer, spec) =
        build_ask_l11_vm_c(owner, target, targets, card, cards, core, catalog)?;
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

pub(super) fn build_ask_l11_vm_c(
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    let ask_id = flag(core, owner, "l11c_ask_id");
    match ask_id {
        ASK_CHONGZHEN_CONFIRM => confirm_ask(owner, "chongzhen:confirm"),
        ASK_CHONGZHEN_CARD => {
            let t = target.unwrap_or(owner);
            let opts = core.player(t).hand.iter().copied().map(ChoiceOption::Card).collect();
            spec(owner, ChoiceKind::SelectCards, "chongzhen:choose_card", opts, 1, 1, false)
        }
        ASK_DUJIN_CONFIRM => confirm_ask(owner, "dujin:confirm"),
        ASK_GNJINFAN_CARDS => {
            let used_suits: u32 = pile_cards(core, owner, "gnjinfan")
                .into_iter()
                .fold(0, |m, c| m | suit_bit(card_suit(core, catalog, owner, c)));
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| (used_suits & suit_bit(card_suit(core, catalog, owner, c))) == 0)
                .map(ChoiceOption::Card)
                .collect();
            let max = opts.len();
            spec(owner, ChoiceKind::SelectCards, "gnjinfan:choose_cards", opts, 1, max, true)
        }
        ASK_HUOJI_CARD => {
            let opts = hes_cards(core, owner)
                .into_iter()
                .filter(|&c| is_red_suit(card_suit(core, catalog, owner, c)))
                .map(ChoiceOption::Card)
                .collect();
            spec(owner, ChoiceKind::SelectCards, "huoji:choose_card", opts, 1, 1, true)
        }
        ASK_HUOJI_TARGET => {
            let cid = card.unwrap_or(0);
            let opts = single_targets_for_virtual_trick(core, catalog, owner, CardKind::Linghunzhihuo, false)
                .into_iter()
                .filter(|&t| {
                    !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                        core, catalog, owner, t, cid, CardKind::Linghunzhihuo, crate::event::DamageNature::Fire,
                    )
                })
                .map(ChoiceOption::Player)
                .collect();
            spec(owner, ChoiceKind::SelectPlayers, "huoji:choose_target", opts, 1, 1, true)
        }
        ASK_JUECE_CONFIRM => confirm_ask(owner, "juece:confirm"),
        ASK_LONGHUN_KIND => {
            let need = longhun_need(core, owner);
            let hes = hes_cards(core, owner);
            let mut tags = Vec::new();
            let hearts = hes.iter().filter(|&&c| card_matches_suit(core, catalog, owner, c, Suit::Heart)).count();
            let diamonds = hes.iter().filter(|&&c| card_matches_suit(core, catalog, owner, c, Suit::Diamond)).count();
            if core.player(owner).hp < core.player(owner).max_hp as i32 && hearts >= need {
                tags.push(ChoiceOption::Tag(0));
            }
            if diamonds >= need
                && can_use_sha_limit_for_card(core, catalog, owner, None, CardKind::FireSha, false)
                && !sha_targets_for_card(core, catalog, owner, None, CardKind::FireSha, false).is_empty()
            {
                tags.push(ChoiceOption::Tag(1));
            }
            spec(owner, ChoiceKind::PickTag, "longhun:choose_use_kind", tags, 1, 1, true)
        }
        ASK_LONGHUN_CARDS => {
            let need = longhun_need(core, owner);
            let want_suit = match cards.first().copied().unwrap_or(0) {
                0 => Suit::Heart,
                1 => Suit::Diamond,
                2 => Suit::Spade,
                _ => Suit::Club,
            };
            let opts = hes_cards(core, owner)
                .into_iter()
                .filter(|&c| card_matches_suit(core, catalog, owner, c, want_suit))
                .map(ChoiceOption::Card)
                .collect();
            spec(owner, ChoiceKind::SelectCards, "longhun:choose_cards", opts, need, need, true)
        }
        ASK_LONGHUN_TARGET => {
            let cid = cards.first().copied();
            let opts = sha_targets_for_card(core, catalog, owner, cid, CardKind::FireSha, false)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            spec(owner, ChoiceKind::SelectPlayers, "longhun:choose_target", opts, 1, 1, true)
        }
        ASK_MIEJI_TARGET | ASK_QIAOSHUI3_ADD_TARGET => {
            let opts: Vec<ChoiceOption> = cards.iter().map(|&p| ChoiceOption::Player(p as PlayerId)).collect();
            let prompt = if ask_id == ASK_MIEJI_TARGET {
                "mieji:choose_target"
            } else {
                "qiaoshui3:add_target"
            };
            spec(owner, ChoiceKind::SelectPlayers, prompt, opts, 0, 1, true)
        }
        ASK_QIAOSHUI3_REMOVE_TARGET => {
            let opts = cards.iter().map(|&p| ChoiceOption::Player(p as PlayerId)).collect();
            spec(owner, ChoiceKind::SelectPlayers, "qiaoshui3:remove_target", opts, 0, 1, true)
        }
        ASK_NSFEIXIONG_TARGET | ASK_QIAOSHUI_TARGET => {
            let opts = alive_others(core, owner)
                .into_iter()
                .filter(|&p| {
                    !core.player(p).hand.is_empty()
                        && crate::rvm_bridge::modifiers::d34_l11_mods_vm_c::can_compare_target(core, catalog, p)
                })
                .map(ChoiceOption::Player)
                .collect();
            let prompt = if ask_id == ASK_NSFEIXIONG_TARGET {
                "nsfeixiong:choose_target"
            } else {
                "qiaoshui:target"
            };
            spec(owner, ChoiceKind::SelectPlayers, prompt, opts, 0, 1, true)
        }
        ASK_NSFEIXIONG_PINDIAN_OWNER | ASK_QIAOSHUI_PINDIAN_OWNER | ASK_ZHUIKONG_PINDIAN_OWNER => {
            let prompt = match ask_id {
                ASK_NSFEIXIONG_PINDIAN_OWNER => "nsfeixiong:pindian_owner",
                ASK_QIAOSHUI_PINDIAN_OWNER => "qiaoshui:pindian_owner",
                _ => "zhuikong:pindian_owner",
            };
            let opts = core.player(owner).hand.iter().copied().map(ChoiceOption::Card).collect();
            spec(owner, ChoiceKind::SelectCards, prompt, opts, 1, 1, false)
        }
        ASK_NSFEIXIONG_PINDIAN_TARGET | ASK_QIAOSHUI_PINDIAN_TARGET | ASK_ZHUIKONG_PINDIAN_TARGET => {
            let t = target.unwrap_or(owner);
            let prompt = match ask_id {
                ASK_NSFEIXIONG_PINDIAN_TARGET => "nsfeixiong:pindian_target",
                ASK_QIAOSHUI_PINDIAN_TARGET => "qiaoshui:pindian_target",
                _ => "zhuikong:pindian_target",
            };
            let opts = core.player(t).hand.iter().copied().map(ChoiceOption::Card).collect();
            spec(t, ChoiceKind::SelectCards, prompt, opts, 1, 1, false)
        }
        ASK_NSYICE_ORDER => {
            let opts = cards.iter().copied().map(ChoiceOption::Card).collect();
            let n = cards.len();
            spec(owner, ChoiceKind::SelectCards, "nsyice:order_cards", opts, n, n, false)
        }
        ASK_NSYICE_TOP => {
            let opts = cards.iter().copied().map(ChoiceOption::Card).collect();
            spec(owner, ChoiceKind::SelectCards, "nsyice:top_card", opts, 1, 1, false)
        }
        ASK_NSYICE_DAMAGE_TARGET => {
            let opts = alive_all(core, owner).into_iter().map(ChoiceOption::Player).collect();
            spec(owner, ChoiceKind::SelectPlayers, "nsyice:damage_target", opts, 1, 1, false)
        }
        ASK_NZRY_JIEYING_TARGET => {
            let opts = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !core.player(p).linked)
                .map(ChoiceOption::Player)
                .collect();
            spec(owner, ChoiceKind::SelectPlayers, "nzry_jieying:choose_target", opts, 1, 1, false)
        }
        ASK_QICE_TRICK => {
            let opts = qice_legal_tricks(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Tag)
                .collect();
            spec(owner, ChoiceKind::PickTag, "qice:choose_trick", opts, 1, 1, true)
        }
        ASK_QICE_TARGETS => {
            let kind_idx = card.unwrap_or(0) as usize;
            let kind = QICE_KINDS.get(kind_idx).copied().unwrap_or(CardKind::Wuzhong);
            if kind == CardKind::Jiedao {
                let mut holders = Vec::new();
                for (h, _) in qice_jiedao_pairs(core, catalog, owner) {
                    if !holders.contains(&ChoiceOption::Player(h)) {
                        holders.push(ChoiceOption::Player(h));
                    }
                }
                let opts = if let Some(&h) = targets.first() {
                    alive_all(core, owner)
                        .into_iter()
                        .filter(|&v| v != h && core.in_attack_range(catalog, h, v))
                        .map(ChoiceOption::Player)
                        .collect()
                } else {
                    holders
                };
                spec(owner, ChoiceKind::SelectPlayers, "qice:choose_targets", opts, 1, 1, true)
            } else {
                let cands = single_targets_for_virtual_trick(core, catalog, owner, kind, false);
                let max = match kind {
                    CardKind::Chain => cands.len().min(2),
                    _ => cands.len().min(1),
                };
                let opts = cands.into_iter().map(ChoiceOption::Player).collect();
                spec(owner, ChoiceKind::SelectPlayers, "qice:choose_targets", opts, 1, max, true)
            }
        }
        ASK_REJIZHI_CONFIRM => confirm_ask(owner, "rejizhi:confirm"),
        ASK_REJIZHI_DISCARD => {
            let c = card?;
            spec(owner, ChoiceKind::SelectCards, "rejizhi:discard", vec![ChoiceOption::Card(c)], 0, 1, true)
        }
        ASK_RELIANJI_TARGETS => {
            let opts = alive_others(core, owner).into_iter().map(ChoiceOption::Player).collect();
            spec(owner, ChoiceKind::SelectPlayers, "relianji:choose_targets", opts, 2, 2, true)
        }
        ASK_REQIANGXI_TARGET => {
            let tstamp = turn_stamp(core);
            let opts = alive_others(core, owner)
                .into_iter()
                .filter(|&p| {
                    core.in_attack_range(catalog, owner, p)
                        && flag(core, p, &format!("l11c_reqiangxi_off:{owner}")) != tstamp
                })
                .map(ChoiceOption::Player)
                .collect();
            spec(owner, ChoiceKind::SelectPlayers, "reqiangxi:choose_targets", opts, 1, 1, true)
        }
        ASK_REQIANGXI_WEAPON => {
            let opts = he_cards(core, owner)
                .into_iter()
                .filter(|&c| is_weapon_card(catalog, core, c))
                .map(ChoiceOption::Card)
                .collect();
            spec(owner, ChoiceKind::SelectCards, "reqiangxi:choose_weapon", opts, 0, 1, true)
        }
        ASK_RESHANXI_COST => {
            let opts = reshanxi_cost_cards(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            spec(owner, ChoiceKind::SelectCards, "reshanxi:choose_cost", opts, 0, 1, true)
        }
        ASK_RESHANXI_TARGET => {
            let opts = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !he_cards(core, p).is_empty())
                .map(ChoiceOption::Player)
                .collect();
            spec(owner, ChoiceKind::SelectPlayers, "reshanxi:choose_target", opts, 1, 1, true)
        }
        ASK_RESHANXI_EXILE => {
            let t = target.unwrap_or(owner);
            let he = he_cards(core, t);
            let max = (core.player(owner).hp.max(1) as usize).min(he.len());
            let opts = he.into_iter().map(ChoiceOption::Card).collect();
            spec(owner, ChoiceKind::SelectCards, "reshanxi:exile_cards", opts, 1, max, false)
        }
        ASK_RETIEJI_CONFIRM => confirm_ask(owner, "retieji:confirm"),
        ASK_RETIEJI_DISCARD => {
            let t = target.unwrap_or(owner);
            let req_suit = card.unwrap_or(0);
            let opts: Vec<ChoiceOption> = he_cards(core, t)
                .into_iter()
                .filter(|&c| suit_bit(card_suit(core, catalog, t, c)) == req_suit)
                .map(ChoiceOption::Card)
                .collect();
            if opts.is_empty() {
                None
            } else {
                spec(t, ChoiceKind::SelectCards, "retieji:discard", opts, 0, 1, true)
            }
        }
        ASK_REZHENXING_CONFIRM => confirm_ask(owner, "rezhenxing:confirm"),
        ASK_REZHENXING_PICK => {
            let opts: Vec<ChoiceOption> = cards
                .iter()
                .copied()
                .filter(|&c| {
                    let s = card_suit(core, catalog, owner, c);
                    cards.iter().filter(|&&o| card_suit(core, catalog, owner, o) == s).count() == 1
                })
                .map(ChoiceOption::Card)
                .collect();
            if opts.is_empty() {
                spec(
                    owner,
                    ChoiceKind::SelectCards,
                    "rezhenxing:pick_card",
                    cards.iter().copied().map(ChoiceOption::Card).collect(),
                    0,
                    0,
                    true,
                )
            } else {
                spec(owner, ChoiceKind::SelectCards, "rezhenxing:pick_card", opts, 0, 1, true)
            }
        }
        ASK_REZHIHENG_CARDS => {
            let opts = he_cards(core, owner).into_iter().map(ChoiceOption::Card).collect();
            let max = core.player(owner).hand.len() + core.player(owner).equips.len();
            spec(owner, ChoiceKind::SelectCards, "rezhiheng:choose_cards", opts, 1, max, true)
        }
        ASK_SHELIE_PICK => {
            let mut suits_mask = 0u32;
            for &c in cards {
                suits_mask |= suit_bit(card_suit(core, catalog, owner, c));
            }
            let need = (suits_mask.count_ones() as usize).max(1);
            let opts = cards.iter().copied().map(ChoiceOption::Card).collect();
            spec(owner, ChoiceKind::SelectCards, "shelie:pick_card", opts, need, need, false)
        }
        ASK_TWJIANGE_CARD => {
            let opts = hes_cards(core, owner)
                .into_iter()
                .filter(|&c| !catalog.resolved_card_kind(core, c).is_basic_kind())
                .map(ChoiceOption::Card)
                .collect();
            spec(owner, ChoiceKind::SelectCards, "twjiange:choose_card", opts, 1, 1, true)
        }
        ASK_TWJIANGE_TARGET => {
            let cid = card;
            let opts = sha_targets_for_card(core, catalog, owner, cid, CardKind::Sha, true)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            spec(owner, ChoiceKind::SelectPlayers, "twjiange:choose_target", opts, 1, 1, true)
        }
        ASK_TWKUJIAN_CARDS => {
            let opts = core.player(owner).hand.iter().copied().map(ChoiceOption::Card).collect();
            let max = core.player(owner).hand.len().min(3);
            spec(owner, ChoiceKind::SelectCards, "twkujian:choose_cards", opts, 1, max, true)
        }
        ASK_TWKUJIAN_TARGET => {
            let opts = alive_others(core, owner).into_iter().map(ChoiceOption::Player).collect();
            spec(owner, ChoiceKind::SelectPlayers, "twkujian:choose_target", opts, 1, 1, true)
        }
        ASK_TWKUJIAN_DISCARD => {
            let who = target.unwrap_or(owner);
            let opts = he_cards(core, who).into_iter().map(ChoiceOption::Card).collect();
            spec(who, ChoiceKind::SelectCards, "twkujian:discard", opts, 1, 1, false)
        }
        ASK_XINANJIAN_MODE => spec(
            owner,
            ChoiceKind::PickTag,
            "xinanjian:choose_mode",
            vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
            1,
            1,
            false,
        ),
        ASK_XINKUANGGU_MODE => {
            let tags = if core.player(owner).hp >= core.player(owner).max_hp as i32 {
                vec![ChoiceOption::Tag(0)]
            } else {
                vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)]
            };
            spec(owner, ChoiceKind::PickTag, "xinkuanggu:choose_mode", tags, 0, 1, true)
        }
        ASK_XINNILUAN_CARD => {
            let opts = xinniluan_usable_cards(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            spec(owner, ChoiceKind::SelectCards, "xinniluan:choose_card", opts, 0, 1, true)
        }
        ASK_XINNILUAN_DISCARD => {
            let t = target.unwrap_or(owner);
            let opts = he_cards(core, t).into_iter().map(ChoiceOption::Card).collect();
            spec(owner, ChoiceKind::SelectCards, "xinniluan:discard_target", opts, 1, 1, false)
        }
        ASK_XIONGYI_TARGETS => {
            let opts: Vec<ChoiceOption> = alive_all(core, owner).into_iter().map(ChoiceOption::Player).collect();
            let max = opts.len();
            spec(owner, ChoiceKind::SelectPlayers, "xiongyi:choose_targets", opts, 1, max, true)
        }
        ASK_ZHUIKONG_CONFIRM => confirm_ask(owner, "zhuikong:confirm"),
        ASK_PINDIAN_YINGYANG_OWNER => {
            let ask = super::super::l11_common_b2::ask_txyingyang_mode(owner);
            Some((
                ask.observer,
                ChoiceSpec {
                    kind: ask.kind,
                    prompt: ask.prompt,
                    options: ask.options,
                    min: ask.min,
                    max: ask.max,
                    allow_cancel: ask.allow_cancel,
                },
            ))
        }
        ASK_PINDIAN_YINGYANG_TARGET => {
            let t = target.unwrap_or(owner);
            let ask = super::super::l11_common_b2::ask_txyingyang_mode(t);
            Some((
                ask.observer,
                ChoiceSpec {
                    kind: ask.kind,
                    prompt: ask.prompt,
                    options: ask.options,
                    min: ask.min,
                    max: ask.max,
                    allow_cancel: ask.allow_cancel,
                },
            ))
        }
        _ => None,
    }
}
