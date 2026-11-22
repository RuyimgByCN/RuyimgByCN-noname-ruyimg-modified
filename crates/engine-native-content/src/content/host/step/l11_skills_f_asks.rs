//! L11（太虚幻境）武将技能 Group F（`L11GeneralSkillsF` tag 732）的询问构建（`l11_skills_f_asks.rs`）。

use super::l11_common::*;
use super::l11_skills_e_asks::find_red_or_black_color_val;
use super::*;
use crate::catalog::{CardKind, Catalog};
use crate::choice::ChoiceSpec;
use crate::ids::{CardId, PlayerId};
use crate::rvm_bridge::modifiers::eval_card_target_canceled;
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

pub(super) const ASK_SUOMING_TARGETS: u32 = 1;
pub(super) const ASK_HUANGMEN_MODE: u32 = 2;
pub(super) const ASK_MOUQIANG_TARGET: u32 = 3;
pub(super) const ASK_MOUQIANG_HE: u32 = 4;
pub(super) const ASK_SUZHI_HE: u32 = 5;
pub(super) const ASK_MANZHEN_CARDS: u32 = 6;
pub(super) const ASK_MANZHEN_TARGETS: u32 = 7;
pub(super) const ASK_MANZHEN_CONFIRM_DMG: u32 = 8;
pub(super) const ASK_ZHENNU_TARGET: u32 = 9;
pub(super) const ASK_YONGGUAN_DISCARD: u32 = 10;
pub(super) const ASK_CHENGYUAN_CARD: u32 = 11;
pub(super) const ASK_CHENGYUAN_TARGET: u32 = 12;
pub(super) const ASK_BENJI_TARGET: u32 = 13;
pub(super) const ASK_TXJIANG_CONFIRM: u32 = 14;

pub(super) fn manzhen_valid_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<PlayerId> {
    alive_others(core, owner)
        .into_iter()
        .filter(|&t| {
            !eval_card_target_canceled(
                core,
                catalog,
                owner,
                t,
                0,
                CardKind::SavageAttack,
                crate::event::DamageNature::Normal,
            )
        })
        .collect()
}

pub(super) fn build_l11_skills_f_ask(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    _targets: &[PlayerId],
    _card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    let tgt = target.unwrap_or(owner);
    match ask_id {
        ASK_SUOMING_TARGETS => {
            let unlinked: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !core.player(p).linked)
                .collect();
            let max = unlinked.len();
            player_ask(owner, "txhj_suoming:choose_targets", unlinked, 1, max, true)
        }
        ASK_HUANGMEN_MODE => tag_ask(owner, "txhj_huangmen:choose_mode", 0..=1, false),
        ASK_MOUQIANG_TARGET => player_ask(
            owner,
            "txhj_mouqiang:choose_target",
            alive_all(core, owner),
            1,
            1,
            true,
        ),
        ASK_MOUQIANG_HE => {
            let pool = he_cards(core, tgt);
            let want = (cards.first().copied().unwrap_or(2) as usize / 2)
                .max(1)
                .min(pool.len());
            card_ask(owner, "txhj_mouqiang:choose_he", pool, want, want, false)
        }
        ASK_SUZHI_HE => card_ask(owner, "txhj_suzhi:choose_he", he_cards(core, tgt), 1, 1, false),
        ASK_MANZHEN_CARDS => {
            let hand = core.player(owner).hand.clone();
            let max = hand.len().min(manzhen_valid_targets(core, catalog, owner).len());
            card_ask(owner, "txmanzhen:choose_cards", hand, 1, max, true)
        }
        ASK_MANZHEN_TARGETS => {
            let need = cards.len();
            let valid = manzhen_valid_targets(core, catalog, owner);
            player_ask(owner, "txmanzhen:choose_targets", valid, need, need, true)
        }
        ASK_MANZHEN_CONFIRM_DMG => confirm_ask(owner, "txmanzhen:confirm_damage"),
        ASK_ZHENNU_TARGET => player_ask(
            owner,
            "txzhennu:choose_target",
            alive_others(core, owner),
            1,
            1,
            true,
        ),
        ASK_YONGGUAN_DISCARD => {
            let marks = flag(core, owner, "l11_yongguan_marks") as usize;
            let hand = core.player(owner).hand.clone();
            let max = marks.min(hand.len());
            card_ask(owner, "txyongguan_fate:discard", hand, 1, max, true)
        }
        ASK_CHENGYUAN_CARD => {
            let used_mask = flag(core, owner, "skill_turn:l11_chengyuan_used");
            let pool: Vec<CardId> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| {
                    let bit = find_red_or_black_color_val(catalog, core, owner, c);
                    (used_mask & bit) == 0
                })
                .collect();
            card_ask(owner, "txchengyuan:choose_card", pool, 1, 1, true)
        }
        ASK_CHENGYUAN_TARGET => player_ask(
            owner,
            "txchengyuan:choose_target",
            alive_others(core, owner),
            1,
            1,
            true,
        ),
        ASK_BENJI_TARGET => player_ask(
            owner,
            "txbenji:choose_target",
            alive_all(core, owner),
            1,
            1,
            true,
        ),
        ASK_TXJIANG_CONFIRM => confirm_ask(owner, "txjiang:confirm"),
        _ => None,
    }
}

pub(in crate::content::host) fn ask_l11_skills_f(
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
        build_l11_skills_f_ask(ask_id, owner, target, targets, card, cards, core, catalog)?;
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
