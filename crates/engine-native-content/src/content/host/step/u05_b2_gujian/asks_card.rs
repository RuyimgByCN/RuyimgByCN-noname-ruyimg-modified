use super::cards_and_foods::{yanjiadan_targets_for, yanjiadan_triplet};
use super::helpers::*;
use super::super::*;
use crate::catalog::{CardKind, Catalog};
use crate::choice::{ChoiceKind, ChoiceOption};
use crate::ids::{CardId, PlayerId};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

pub(in crate::content::host) fn ask_yuheng_cost_only(
    owner: PlayerId,
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let allow_black = find_yuheng_equip_card(core, catalog, owner)
        .is_some_and(|c| yuheng_tier(core, catalog, c) >= 1);
    let opts: Vec<ChoiceOption> = core
        .player(owner)
        .hand
        .iter()
        .copied()
        .filter(|&c| {
            if allow_black {
                is_black(catalog, c)
            } else {
                catalog.cards[c as usize].suit == crate::catalog::Suit::Spade
            }
        })
        .map(ChoiceOption::Card)
        .collect();
    build_ask(
        owner,
        "gj_yuheng_cost",
        ChoiceKind::SelectCards,
        opts,
        1,
        1,
        true,
    )
}

pub(in crate::content::host) fn ask_card_batch(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    _targets: &[PlayerId],
    _card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let ask_id = current_ask_id(core, owner, HostBehavior::U05GujianCardBatch);
    let obs = current_ask_obs(core, owner, HostBehavior::U05GujianCardBatch);
    match (stage, ask_id) {
        (HostStage::SelectCard, ASK_YANJIADAN_VCARD) => {
            let def_key = target_def_key_from_card_or_owner(core, catalog, _card);
            let trio = yanjiadan_triplet(def_key);
            let opts: Vec<ChoiceOption> = trio
                .iter()
                .enumerate()
                .filter_map(|(i, &k)| {
                    let (cands, _, _) = yanjiadan_targets_for(core, catalog, owner, k);
                    if cands.is_empty() {
                        None
                    } else {
                        Some(ChoiceOption::Tag(i as u8))
                    }
                })
                .collect();
            build_ask(obs, "gj_yanjiadan_vcard", ChoiceKind::PickTag, opts, 1, 1, false)
        }
        (HostStage::TargetChoice, ASK_YANJIADAN_TARGET) => {
            let def_key = target_def_key_from_card_or_owner(core, catalog, _card);
            let trio = yanjiadan_triplet(def_key);
            let idx = cards.first().copied().unwrap_or(0) as usize;
            let (cands, min, max) = yanjiadan_targets_for(core, catalog, owner, trio[idx.min(2)]);
            build_ask(
                obs,
                "gj_yanjiadan_target",
                ChoiceKind::SelectPlayers,
                cands.into_iter().map(ChoiceOption::Player).collect(),
                min,
                max,
                false,
            )
        }
        (HostStage::SelectCard, ASK_G_SHIHUIFEN_USE) => {
            let mut opts: Vec<ChoiceOption> = vec![ChoiceOption::Pass];
            opts.extend(
                core.player(owner)
                    .hand
                    .iter()
                    .copied()
                    .filter(|&c| catalog.card_kind(c) == CardKind::Shihuifen)
                    .map(ChoiceOption::Card),
            );
            let _ = target;
            build_ask(
                obs,
                "gj_shihuifen_use",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                false,
            )
        }
        (HostStage::SelectCard, ASK_YUHENG_COST) => ask_yuheng_cost_only(obs, core, catalog),
        _ => None,
    }
}

fn target_def_key_from_card_or_owner<'a>(
    _core: &GameStateCore,
    catalog: &'a Catalog,
    card: Option<u32>,
) -> &'a str {
    card.and_then(|c| catalog.cards.get(c as usize))
        .map(|cd| cd.name)
        .unwrap_or("yanjiadan_heart")
}
