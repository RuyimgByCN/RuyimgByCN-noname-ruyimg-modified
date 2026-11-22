//! L11（太虚幻境）装备与专属宝物牌技能的询问构建（`l11_equips_asks.rs`）。

use super::l11_common::*;
use super::*;
use crate::catalog::{CardKind, Catalog, EquipSlot};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::DamageNature;
use crate::ids::{CardId, PlayerId};
use crate::rvm_bridge::modifiers::eval_card_target_canceled;
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

pub(super) const ASK_NODA_AXE_DISCARD: u32 = 1;
pub(super) const ASK_NODA_AXE_RAW_DISCARD: u32 = 2;
pub(super) const ASK_QILIN_CONFIRM: u32 = 3;
pub(super) const ASK_QILIN_SKILL_CONFIRM: u32 = 4;
pub(super) const ASK_FANGTIAN_TARGETS: u32 = 5;
pub(super) const ASK_PILICHE_CONFIRM: u32 = 6;
pub(super) const ASK_LEIJIMU_CONFIRM: u32 = 7;
pub(super) const ASK_FEILONG_CONFIRM: u32 = 8;
pub(super) const ASK_FEILONG_SKILL_CONFIRM: u32 = 9;
pub(super) const ASK_MINGZHU_DISCARD: u32 = 10;
pub(super) const ASK_TAIJI_DISCARD: u32 = 11;
pub(super) const ASK_GX_TAIJI_DISCARD: u32 = 12;

pub(super) fn axe_discardable_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<CardId> {
    let wpn = equip_slot_card(core, catalog, owner, EquipSlot::Weapon).filter(|&c| {
        catalog
            .cards
            .get(c as usize)
            .is_some_and(|cd| cd.name == "txhj_xingtianpojunfu" || cd.name == "txhj_noda_axe")
    });
    he_cards(core, owner)
        .into_iter()
        .filter(|&c| Some(c) != wpn)
        .collect()
}

pub(super) fn mingzhu_discardable_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<CardId> {
    let treasure = equip_slot_card(core, catalog, owner, EquipSlot::Treasure).filter(|&c| {
        catalog
            .cards
            .get(c as usize)
            .is_some_and(|cd| cd.name == "txhj_dinglanyemingzhu")
    });
    he_cards(core, owner)
        .into_iter()
        .filter(|&c| Some(c) != treasure)
        .collect()
}

pub(super) fn set_fangtian_meta(
    core: &mut GameStateCore,
    owner: PlayerId,
    kind: CardKind,
    nature: DamageNature,
) {
    let kind_code = match kind {
        CardKind::FireSha => 2,
        CardKind::LeiSha => 3,
        _ => 1,
    };
    let nature_code = match nature {
        DamageNature::Fire => 2,
        DamageNature::Thunder => 3,
        _ => 1,
    };
    set_flag(core, owner, "l11_fangtian_kind", kind_code);
    set_flag(core, owner, "l11_fangtian_nature", nature_code);
}

pub(super) fn clear_fangtian_meta(core: &mut GameStateCore, owner: PlayerId) {
    set_flag(core, owner, "l11_fangtian_kind", 0);
    set_flag(core, owner, "l11_fangtian_nature", 0);
}

pub(super) fn decode_fangtian_meta(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    sha_cid: Option<CardId>,
) -> (CardKind, DamageNature) {
    let kind = match flag(core, owner, "l11_fangtian_kind") {
        2 => CardKind::FireSha,
        3 => CardKind::LeiSha,
        1 => CardKind::Sha,
        _ => sha_cid.map(|c| catalog.card_kind(c)).unwrap_or(CardKind::Sha),
    };
    let nature = match flag(core, owner, "l11_fangtian_nature") {
        2 => DamageNature::Fire,
        3 => DamageNature::Thunder,
        1 => DamageNature::Normal,
        _ => match kind {
            CardKind::FireSha => DamageNature::Fire,
            CardKind::LeiSha => DamageNature::Thunder,
            _ => DamageNature::Normal,
        },
    };
    (kind, nature)
}

pub(super) fn fangtian_extra_candidates(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    already: &[PlayerId],
    sha_card: Option<CardId>,
    sha_kind: CardKind,
    sha_nature: DamageNature,
) -> Vec<PlayerId> {
    let card_for_mod = sha_card.unwrap_or(u32::MAX);
    let range_unlimited = crate::rvm_bridge::modifiers::eval_card_distance_unlimited(
        core,
        catalog,
        owner,
        card_for_mod,
        sha_kind,
        sha_nature,
    );
    let atk_range = core.attack_range(catalog, owner);
    let kongcheng_def = catalog.skill_by_name(crate::content::standard::KONG_CHENG);
    alive_others(core, owner)
        .into_iter()
        .filter(|&p| {
            !already.contains(&p)
                && (range_unlimited || core.computed_distance(catalog, owner, p) <= atk_range)
                && !kongcheng_def.is_some_and(|d| core.has_skill(p, d) && core.player(p).hand.is_empty())
                && !eval_card_target_canceled(
                    core,
                    catalog,
                    owner,
                    p,
                    card_for_mod,
                    sha_kind,
                    sha_nature,
                )
        })
        .collect()
}

pub(super) fn build_l11_equip_ask(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    _card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    let tgt = target.unwrap_or(owner);
    match ask_id {
        ASK_NODA_AXE_DISCARD => card_ask(
            owner,
            "txhj_xingtianpojunfu:discard_cards",
            axe_discardable_cards(core, catalog, owner),
            2,
            2,
            true,
        ),
        ASK_NODA_AXE_RAW_DISCARD => card_ask(
            owner,
            "txhj_noda_axe:discard_cards",
            axe_discardable_cards(core, catalog, owner),
            2,
            2,
            true,
        ),
        ASK_QILIN_CONFIRM => confirm_ask(owner, "txhj_qilingong:discard_horse"),
        ASK_QILIN_SKILL_CONFIRM => {
            let treasures: Vec<CardId> = core
                .player(tgt)
                .equips
                .iter()
                .copied()
                .filter(|&c| {
                    catalog.card_slot_of(core, c) == Some(EquipSlot::Treasure)
                        && !crate::rvm_bridge::modifiers::eval_card_discard_protected(
                            core, catalog, owner, tgt, c,
                        )
                })
                .collect();
            if treasures.is_empty() {
                return None;
            }
            card_ask(owner, "qilin_skill:discard_horse", treasures, 0, 1, true)
        }
        ASK_FANGTIAN_TARGETS => {
            let sha_cid = cards.first().copied();
            let (sha_kind, sha_nature) = decode_fangtian_meta(core, catalog, owner, sha_cid);
            let cands = fangtian_extra_candidates(
                core,
                catalog,
                owner,
                targets,
                sha_cid,
                sha_kind,
                sha_nature,
            );
            let max_add = 2usize.min(cands.len()).max(1);
            player_ask(
                owner,
                "txhj_fangtianhuaji:extra_targets",
                cands,
                1,
                max_add,
                false,
            )
        }
        ASK_PILICHE_CONFIRM => confirm_ask(owner, "txhj_piliche:confirm"),
        ASK_LEIJIMU_CONFIRM => confirm_ask(owner, "txhj_leijimu:confirm"),
        ASK_FEILONG_CONFIRM => confirm_ask(owner, "txhj_feilong:confirm"),
        ASK_FEILONG_SKILL_CONFIRM => confirm_ask(owner, "txhj_feilong_skill:confirm"),
        ASK_MINGZHU_DISCARD => {
            let pool = mingzhu_discardable_cards(core, catalog, owner);
            let max = (core.player(owner).max_hp as usize).min(pool.len());
            card_ask(owner, "txhj_dinglanyemingzhu:discard_cards", pool, 1, max, true)
        }
        ASK_TAIJI_DISCARD => {
            let mut opts = vec![ChoiceOption::Pass];
            opts.extend(he_cards(core, tgt).into_iter().map(ChoiceOption::Card));
            spec_ask(
                tgt,
                ChoiceKind::Confirm,
                "txhj_taijifuchen:target_discard",
                opts,
                1,
                1,
                true,
            )
        }
        ASK_GX_TAIJI_DISCARD => {
            let mut opts = vec![ChoiceOption::Pass];
            opts.extend(he_cards(core, tgt).into_iter().map(ChoiceOption::Card));
            spec_ask(
                tgt,
                ChoiceKind::Confirm,
                "gx_taijifuchen:target_discard",
                opts,
                1,
                1,
                true,
            )
        }
        _ => None,
    }
}

pub(in crate::content::host) fn ask_l11_equip_cards(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    if stage == HostStage::Act || stage == HostStage::DrawSettle {
        return None;
    }
    let ask_id = card?;
    let (observer, spec) =
        build_l11_equip_ask(ask_id, owner, target, targets, card, cards, core, catalog)?;
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
