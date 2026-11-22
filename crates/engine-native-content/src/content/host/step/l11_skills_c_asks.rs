//! L11（太虚幻境）武将技能 Batch C（S58–S71）的询问构建（`l11_skills_c_asks.rs`）。

use super::l11_common::*;
use super::*;
use crate::catalog::Catalog;
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::ids::{CardId, PlayerId};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

pub(super) const ASK_BAOYING_CONFIRM: u32 = 1;
pub(super) const ASK_FANGONG_SHA: u32 = 2;
pub(super) const ASK_JIELVE_CONFIRM: u32 = 3;
pub(super) const ASK_JIELVE_H: u32 = 4;
pub(super) const ASK_JIELVE_E: u32 = 5;
pub(super) const ASK_JIELVE_J: u32 = 6;
pub(super) const ASK_MOQU_DISCARD: u32 = 7;
pub(super) const ASK_YAOWU_MODE: u32 = 8;
pub(super) const ASK_YINGHUN_TARGET: u32 = 9;
pub(super) const ASK_YINGHUN_MODE: u32 = 10;
pub(super) const ASK_YINGHUN_DISCARD: u32 = 11;

pub(super) fn build_l11_skills_c_ask(
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
        ASK_BAOYING_CONFIRM => confirm_ask(owner, "cxy_BaoYing:confirm"),
        ASK_FANGONG_SHA => {
            let mut opts = vec![ChoiceOption::Pass];
            for &c in &core.player(owner).hand {
                if effective_card_kind(core, catalog, c).is_sha_like() {
                    opts.push(ChoiceOption::Card(c));
                }
            }
            spec_ask(
                owner,
                ChoiceKind::Confirm,
                "cxyFanGong:choose_sha",
                opts,
                1,
                1,
                true,
            )
        }
        ASK_JIELVE_CONFIRM => confirm_ask(owner, "cxyJieLve:confirm"),
        ASK_JIELVE_H => card_ask(
            owner,
            "cxyJieLve:choose_h",
            core.player(tgt).hand.clone(),
            1,
            1,
            false,
        ),
        ASK_JIELVE_E => card_ask(
            owner,
            "cxyJieLve:choose_e",
            core.player(tgt).equips.clone(),
            1,
            1,
            false,
        ),
        ASK_JIELVE_J => card_ask(
            owner,
            "cxyJieLve:choose_j",
            core.player(tgt).judge.clone(),
            1,
            1,
            false,
        ),
        ASK_MOQU_DISCARD => card_ask(
            owner,
            "cxyMoQu:discard_card",
            he_cards(core, owner),
            1,
            1,
            false,
        ),
        ASK_YAOWU_MODE => tag_ask(tgt, "cxyYaoWu:choose_mode", 0..=1, false),
        ASK_YINGHUN_TARGET => player_ask(
            owner,
            "cxyYingHun:choose_target",
            alive_others(core, owner),
            1,
            1,
            true,
        ),
        ASK_YINGHUN_MODE => tag_ask(owner, "cxyYingHun:choose_mode", 0..=1, false),
        ASK_YINGHUN_DISCARD => {
            let pool = he_cards(core, tgt);
            let need = (cards.first().copied().unwrap_or(1) as usize).min(pool.len());
            card_ask(tgt, "cxyYingHun:discard_cards", pool, need, need, false)
        }
        _ => None,
    }
}

pub(in crate::content::host) fn ask_l11_skills_c(
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
        build_l11_skills_c_ask(ask_id, owner, target, targets, card, cards, core, catalog)?;
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
