use super::super::super::*;
use super::super::l11_common::{alive_all, alive_others, flag, he_cards};
use super::helpers::*;
use crate::catalog::{Catalog, Suit};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::ids::{CardId, PlayerId};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

fn mk(
    observer: PlayerId,
    prompt: &'static str,
    kind: ChoiceKind,
    options: Vec<ChoiceOption>,
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> Option<(PlayerId, ChoiceSpec)> {
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

fn confirm(observer: PlayerId, prompt: &'static str, allow_cancel: bool) -> Option<(PlayerId, ChoiceSpec)> {
    mk(
        observer,
        prompt,
        ChoiceKind::Confirm,
        vec![ChoiceOption::Confirm, ChoiceOption::Pass],
        1,
        1,
        allow_cancel,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_ask(
    ask_id: u32,
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    let _ = stage;
    match ask_id {
        ASK_MIJI_CARDS => {
            let need = card.unwrap_or(1) as usize;
            let opts: Vec<ChoiceOption> = core.player(owner).hand.iter().copied().map(ChoiceOption::Card).collect();
            (opts.len() >= need).then(|| mk(owner, "l11_vm_a_miji_cards", ChoiceKind::SelectCards, opts, need, need, true)).flatten()
        }
        ASK_MIJI_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner).into_iter().map(ChoiceOption::Player).collect();
            (!opts.is_empty()).then(|| mk(owner, "l11_vm_a_miji_target", ChoiceKind::SelectPlayers, opts, 1, 1, true)).flatten()
        }
        ASK_FUJI_CONFIRM => confirm(owner, "l11_vm_a_fuji_confirm", true),
        ASK_WURONG_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !core.player(p).hand.is_empty())
                .map(ChoiceOption::Player)
                .collect();
            (!opts.is_empty()).then(|| mk(owner, "l11_vm_a_wurong_target", ChoiceKind::SelectPlayers, opts, 1, 1, true)).flatten()
        }
        ASK_WURONG_CARD => {
            let chooser = if stage == HostStage::CompareTargetPick { target? } else { owner };
            let opts: Vec<ChoiceOption> = core.player(chooser).hand.iter().copied().map(ChoiceOption::Card).collect();
            (!opts.is_empty()).then(|| mk(chooser, "l11_vm_a_wurong_card", ChoiceKind::SelectCards, opts, 1, 1, false)).flatten()
        }
        ASK_WURONG_GAIN => {
            let t = target?;
            let opts: Vec<ChoiceOption> = he_cards(core, t).into_iter().map(ChoiceOption::Card).collect();
            (!opts.is_empty()).then(|| mk(owner, "l11_vm_a_wurong_gain", ChoiceKind::SelectCards, opts, 1, 1, false)).flatten()
        }
        ASK_REFANKUI_GAIN => {
            let src = target?;
            let pool = if src != owner { he_cards(core, src) } else { core.player(src).equips.clone() };
            let opts: Vec<ChoiceOption> = pool.into_iter().map(ChoiceOption::Card).collect();
            (!opts.is_empty()).then(|| mk(owner, "l11_vm_a_refankui_gain", ChoiceKind::SelectCards, opts, 1, 1, true)).flatten()
        }
        ASK_YEYAN_MODE => mk(
            owner,
            "l11_vm_a_yeyan_mode",
            ChoiceKind::PickTag,
            vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
            1,
            1,
            true,
        ),
        ASK_YEYAN_CARD => {
            let want = match cards.len() {
                0 => Suit::Spade,
                1 => Suit::Heart,
                2 => Suit::Club,
                _ => Suit::Diamond,
            };
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| catalog.effective_card_suit(core, owner, c) == want)
                .map(ChoiceOption::Card)
                .collect();
            (!opts.is_empty()).then(|| mk(owner, "l11_vm_a_yeyan_card", ChoiceKind::SelectCards, opts, 1, 1, true)).flatten()
        }
        ASK_YEYAN_SMALL_TARGETS => {
            let opts: Vec<ChoiceOption> = alive_all(core, owner).into_iter().map(ChoiceOption::Player).collect();
            let max = opts.len().min(3);
            (max > 0).then(|| mk(owner, "l11_vm_a_yeyan_small_targets", ChoiceKind::SelectPlayers, opts, 1, max, true)).flatten()
        }
        ASK_YEYAN_GREAT_TARGETS => {
            let opts: Vec<ChoiceOption> = alive_all(core, owner).into_iter().map(ChoiceOption::Player).collect();
            let max = opts.len().min(2);
            (max > 0).then(|| mk(owner, "l11_vm_a_yeyan_great_targets", ChoiceKind::SelectPlayers, opts, 1, max, true)).flatten()
        }
        ASK_YEYAN_2DMG_TARGET => {
            let opts: Vec<ChoiceOption> = targets.iter().copied().map(ChoiceOption::Player).collect();
            (opts.len() >= 2).then(|| mk(owner, "l11_vm_a_yeyan_2dmg_target", ChoiceKind::SelectPlayers, opts, 1, 1, false)).flatten()
        }
        ASK_YEYAN_DMG_AMOUNT => mk(
            owner,
            "l11_vm_a_yeyan_dmg_amount",
            ChoiceKind::PickTag,
            vec![ChoiceOption::Tag(2), ChoiceOption::Tag(3)],
            1,
            1,
            false,
        ),
        ASK_FENWEI_TARGETS => {
            let opts: Vec<ChoiceOption> = targets.iter().copied().map(ChoiceOption::Player).collect();
            let max = opts.len();
            (max > 0).then(|| mk(owner, "l11_vm_a_fenwei_targets", ChoiceKind::SelectPlayers, opts, 1, max, true)).flatten()
        }
        ASK_KUANGFU_EQUIP => {
            let t = target?;
            let opts: Vec<ChoiceOption> = core.player(t).equips.iter().copied().map(ChoiceOption::Card).collect();
            (!opts.is_empty()).then(|| mk(owner, "l11_vm_a_kuangfu_equip", ChoiceKind::SelectCards, opts, 1, 1, true)).flatten()
        }
        ASK_KUANGFU_EQUIP_SELF => confirm(owner, "l11_vm_a_kuangfu_equip_self", true),
        ASK_LANGMIE_PLAY_DISCARD => {
            let opts: Vec<ChoiceOption> = he_cards(core, owner).into_iter().map(ChoiceOption::Card).collect();
            (!opts.is_empty()).then(|| mk(owner, "l11_vm_a_langmie_play_discard", ChoiceKind::SelectCards, opts, 1, 1, true)).flatten()
        }
        ASK_LANGMIE_END_DISCARD => {
            let opts: Vec<ChoiceOption> = he_cards(core, owner).into_iter().map(ChoiceOption::Card).collect();
            (!opts.is_empty()).then(|| mk(owner, "l11_vm_a_langmie_end_discard", ChoiceKind::SelectCards, opts, 1, 1, true)).flatten()
        }
        ASK_YINGJIAN_TARGET => {
            let opts: Vec<ChoiceOption> = alive_all(core, owner)
                .into_iter()
                .filter(|&p| can_virtual_sha_target(core, catalog, owner, p, true))
                .map(ChoiceOption::Player)
                .collect();
            (!opts.is_empty()).then(|| mk(owner, "l11_vm_a_yingjian_target", ChoiceKind::SelectPlayers, opts, 1, 1, true)).flatten()
        }
        ASK_QIAOMENG_DISCARD => {
            let t = target?;
            let opts: Vec<ChoiceOption> = core.player(t).equips.iter().copied().map(ChoiceOption::Card).collect();
            (!opts.is_empty()).then(|| mk(owner, "l11_vm_a_qiaomeng_discard", ChoiceKind::SelectCards, opts, 1, 1, true)).flatten()
        }
        ASK_SHENXIAN_CONFIRM => confirm(owner, "l11_vm_a_shenxian_confirm", true),
        ASK_LIANPO_CONFIRM => confirm(owner, "l11_vm_a_lianpo_confirm", true),
        ASK_BIHUO_CONFIRM => confirm(owner, "l11_vm_a_bihuo_confirm", true),
        ASK_CHUNLAO_STORE => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| catalog.resolved_card_kind(core, c).is_sha_like())
                .map(ChoiceOption::Card)
                .collect();
            let max = opts.len();
            (max > 0).then(|| mk(owner, "l11_vm_a_chunlao_store", ChoiceKind::SelectCards, opts, 1, max, true)).flatten()
        }
        ASK_CHUNLAO_SPEND => {
            let opts: Vec<ChoiceOption> = core
                .vm_aux
                .piles
                .get(&(owner, "chunlao".to_string()))
                .map(|p| p.iter().copied().map(ChoiceOption::Card).collect())
                .unwrap_or_default();
            (!opts.is_empty()).then(|| mk(owner, "l11_vm_a_chunlao_spend", ChoiceKind::SelectCards, opts, 1, 1, false)).flatten()
        }
        ASK_XIONGLUAN_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner).into_iter().map(ChoiceOption::Player).collect();
            (!opts.is_empty()).then(|| mk(owner, "l11_vm_a_xiongluan_target", ChoiceKind::SelectPlayers, opts, 1, 1, true)).flatten()
        }
        ASK_FENMING_CONFIRM => confirm(owner, "l11_vm_a_fenming_confirm", true),
        ASK_FENMING_CARD => {
            let t = target?;
            let opts: Vec<ChoiceOption> = he_cards(core, t).into_iter().map(ChoiceOption::Card).collect();
            (!opts.is_empty()).then(|| mk(owner, "l11_vm_a_fenming_card", ChoiceKind::SelectCards, opts, 1, 1, false)).flatten()
        }
        ASK_FENYIN_CONFIRM => confirm(owner, "l11_vm_a_fenyin_confirm", true),
        ASK_GUZHENG_CONFIRM => confirm(owner, "l11_vm_a_guzheng_confirm", true),
        ASK_GUZHENG_KEEP => {
            let t = target?;
            let (_, own) = guzheng_discarded_lists(core, t);
            let opts: Vec<ChoiceOption> = own.into_iter().map(ChoiceOption::Card).collect();
            (!opts.is_empty()).then(|| mk(owner, "l11_vm_a_guzheng_keep", ChoiceKind::SelectCards, opts, 1, 1, true)).flatten()
        }
        ASK_GUZHENG_GAIN => confirm(owner, "l11_vm_a_guzheng_gain", true),
        ASK_GZYINGHUN_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner).into_iter().map(ChoiceOption::Player).collect();
            (!opts.is_empty()).then(|| mk(owner, "l11_vm_a_gzyinghun_target", ChoiceKind::SelectPlayers, opts, 1, 1, true)).flatten()
        }
        ASK_GZYINGHUN_MODE => mk(
            owner,
            "l11_vm_a_gzyinghun_mode",
            ChoiceKind::PickTag,
            vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
            1,
            1,
            false,
        ),
        ASK_GZYINGHUN_DISCARD => {
            let t = target?;
            let opts: Vec<ChoiceOption> = he_cards(core, t).into_iter().map(ChoiceOption::Card).collect();
            let need = (card.unwrap_or(1) as usize).min(opts.len());
            (need > 0).then(|| mk(t, "l11_vm_a_gzyinghun_discard", ChoiceKind::SelectCards, opts, need, need, false)).flatten()
        }
        ASK_JIANYING_CONFIRM => confirm(owner, "l11_vm_a_jianying_confirm", true),
        ASK_KAIKANG_CONFIRM => confirm(owner, "l11_vm_a_kaikang_confirm", true),
        ASK_KAIKANG_GIVE => {
            let opts: Vec<ChoiceOption> = he_cards(core, owner).into_iter().map(ChoiceOption::Card).collect();
            (!opts.is_empty()).then(|| mk(owner, "l11_vm_a_kaikang_give", ChoiceKind::SelectCards, opts, 1, 1, false)).flatten()
        }
        ASK_KAIKANG_USE_EQUIP => {
            let t = target?;
            confirm(t, "l11_vm_a_kaikang_use_equip", true)
        }
        ASK_KUNFEN_CONFIRM => confirm(owner, "l11_vm_a_kunfen_confirm", true),
        ASK_LIHUO_CONVERT => confirm(owner, "l11_vm_a_lihuo_convert", true),
        ASK_LIHUO_EXTRA => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|p| !targets.contains(p) && can_virtual_sha_target(core, catalog, owner, *p, false))
                .map(ChoiceOption::Player)
                .collect();
            (!opts.is_empty()).then(|| mk(owner, "l11_vm_a_lihuo_extra", ChoiceKind::SelectPlayers, opts, 1, 1, true)).flatten()
        }
        ASK_MINGCE_CARD => {
            let opts: Vec<ChoiceOption> = mingce_eligible_cards(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            (!opts.is_empty()).then(|| mk(owner, "l11_vm_a_mingce_card", ChoiceKind::SelectCards, opts, 1, 1, true)).flatten()
        }
        ASK_MINGCE_GIVE_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| is_mingce_give_candidate(core, catalog, owner, p))
                .map(ChoiceOption::Player)
                .collect();
            (!opts.is_empty()).then(|| mk(owner, "l11_vm_a_mingce_give_target", ChoiceKind::SelectPlayers, opts, 1, 1, true)).flatten()
        }
        ASK_MINGCE_SHA_TARGET => {
            let t1 = target?;
            let opts: Vec<ChoiceOption> = alive_all(core, t1)
                .into_iter()
                .filter(|&p| p != t1 && core.in_attack_range(catalog, t1, p))
                .map(ChoiceOption::Player)
                .collect();
            (!opts.is_empty()).then(|| mk(owner, "l11_vm_a_mingce_sha_target", ChoiceKind::SelectPlayers, opts, 1, 1, false)).flatten()
        }
        ASK_MINGCE_CHOICE => {
            let t1 = target?;
            mk(
                t1,
                "l11_vm_a_mingce_choice",
                ChoiceKind::PickTag,
                vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                1,
                1,
                false,
            )
        }
        ASK_NSYANGWU_TARGET => {
            let my_hand = core.player(owner).hand.len();
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| core.player(p).hand.len() > my_hand)
                .map(ChoiceOption::Player)
                .collect();
            (!opts.is_empty()).then(|| mk(owner, "l11_vm_a_nsyangwu_target", ChoiceKind::SelectPlayers, opts, 1, 1, true)).flatten()
        }
        ASK_NSYANGWU_CARD => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| catalog.effective_card_suit(core, owner, c) == Suit::Heart)
                .map(ChoiceOption::Card)
                .collect();
            (!opts.is_empty()).then(|| mk(owner, "l11_vm_a_nsyangwu_card", ChoiceKind::SelectCards, opts, 1, 1, true)).flatten()
        }
        ASK_NSYANGWU_GAIN => {
            let t = target?;
            let opts: Vec<ChoiceOption> = core.player(t).hand.iter().copied().map(ChoiceOption::Card).collect();
            let need = (card.unwrap_or(1) as usize).min(opts.len());
            (need > 0).then(|| mk(owner, "l11_vm_a_nsyangwu_gain", ChoiceKind::SelectCards, opts, need, need, false)).flatten()
        }
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::content::host) fn ask(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let ask_id = flag(core, owner, ASK_KEY);
    if ask_id == 0 {
        return None;
    }
    let (observer, spec) =
        build_ask(ask_id, stage, owner, target, targets, card, cards, core, catalog)?;
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
