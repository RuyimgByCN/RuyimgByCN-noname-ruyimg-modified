//! L28 蒲元 `th_shengong` 18 张专属装备技能询问构建（`l28_shengong_equips_asks.rs`）。

use super::super::*;
use super::{
    is_l11_taixu_context, zhaogujing_eligible_cards, zhaogujing_huogong_targets,
    zhaogujing_is_huogong, zhaogujing_jiedao_victims, zhaogujing_targets_for,
};
use crate::catalog::{CardKind, Catalog};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::ids::{CardId, PlayerId};
use crate::state::GameStateCore;

pub(super) const ASK_SG_WUSHUANG_MODE: u32 = 100;
pub(super) const ASK_SG_WUSHUANG_DISCARD: u32 = 101;
pub(super) const ASK_SG_BINTIE_CONFIRM: u32 = 102;
pub(super) const ASK_SG_WUXING_NATURE: u32 = 103;
pub(super) const ASK_SG_LINGLONG_CONFIRM: u32 = 104;
pub(super) const ASK_SG_HUXINJING_CONFIRM: u32 = 105;
pub(super) const ASK_SG_SHUFA_TARGET: u32 = 106;
pub(super) const ASK_SG_TIANJITU_DISCARD: u32 = 107;
pub(super) const ASK_SG_TAIGONG_TARGET: u32 = 108;
pub(super) const ASK_SG_TAIGONG_RECAST: u32 = 109;
pub(super) const ASK_SG_ZHAOGUJING_CARD: u32 = 110;
pub(super) const ASK_SG_ZHAOGUJING_TARGET: u32 = 111;
pub(super) const ASK_SG_ZHAOGUJING_JIEDAO_VICTIM: u32 = 112;
pub(super) const ASK_SG_ZHAOGUJING_HUOGONG_DISCARD: u32 = 113;

pub(super) fn spec_ask(
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

/// Constructs `ChoiceSpec` for `ASK_SG_*` equipment prompts (`ask_id >= 100`).
pub(in crate::content::host::step) fn build_equip_ask(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    card: Option<CardId>,
    _cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    match ask_id {
        ASK_SG_WUSHUANG_MODE => {
            let can_disc = target
                .is_some_and(|t| core.player(t).alive && !hand_and_equips(core, t).is_empty());
            let mut opts = vec![ChoiceOption::Tag(0)];
            if can_disc {
                opts.push(ChoiceOption::Tag(1));
            }
            spec_ask(
                owner,
                ChoiceKind::PickTag,
                "wushuangfangtianji:choose_mode",
                opts,
                1,
                1,
                !is_l11_taixu_context(catalog),
            )
        }
        ASK_SG_WUSHUANG_DISCARD => {
            let t = target?;
            let opts = hand_and_equips(core, t)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            spec_ask(
                owner,
                ChoiceKind::SelectCards,
                "wushuangfangtianji:discard_card",
                opts,
                1,
                1,
                false,
            )
        }
        ASK_SG_BINTIE_CONFIRM => spec_ask(
            owner,
            ChoiceKind::Confirm,
            "bintieshuangji:confirm",
            vec![ChoiceOption::Confirm],
            0,
            1,
            true,
        ),
        ASK_SG_WUXING_NATURE => spec_ask(
            owner,
            ChoiceKind::Confirm,
            "wuxinghelingshan:choose_nature",
            vec![ChoiceOption::Confirm],
            0,
            1,
            true,
        ),
        ASK_SG_LINGLONG_CONFIRM => spec_ask(
            owner,
            ChoiceKind::Confirm,
            "linglongshimandai:confirm",
            vec![ChoiceOption::Confirm],
            0,
            1,
            true,
        ),
        ASK_SG_HUXINJING_CONFIRM => spec_ask(
            owner,
            ChoiceKind::Confirm,
            "huxinjing:confirm",
            vec![ChoiceOption::Confirm],
            0,
            1,
            true,
        ),
        ASK_SG_SHUFA_TARGET => {
            let opts = alive_others(core, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            spec_ask(
                owner,
                ChoiceKind::SelectPlayers,
                "shufazijinguan:choose_target",
                opts,
                1,
                1,
                true,
            )
        }
        ASK_SG_TIANJITU_DISCARD => {
            let opts = hand_and_equips(core, owner)
                .into_iter()
                .filter(|&c| Some(c) != card)
                .map(ChoiceOption::Card)
                .collect();
            spec_ask(
                owner,
                ChoiceKind::SelectCards,
                "tianjitu:discard_card",
                opts,
                1,
                1,
                false,
            )
        }
        ASK_SG_TAIGONG_TARGET => {
            let opts = alive_all(core, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            spec_ask(
                owner,
                ChoiceKind::SelectPlayers,
                "taigongyinfu:choose_target",
                opts,
                1,
                1,
                true,
            )
        }
        ASK_SG_TAIGONG_RECAST => {
            let opts = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            spec_ask(
                owner,
                ChoiceKind::SelectCards,
                "taigongyinfu:recast_card",
                opts,
                1,
                1,
                true,
            )
        }
        ASK_SG_ZHAOGUJING_CARD => {
            let opts = zhaogujing_eligible_cards(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            spec_ask(
                owner,
                ChoiceKind::SelectCards,
                "zhaogujing:choose_card",
                opts,
                1,
                1,
                true,
            )
        }
        ASK_SG_ZHAOGUJING_TARGET => {
            let c = card?;
            let (targets, max) = if zhaogujing_is_huogong(catalog, c) {
                (zhaogujing_huogong_targets(core, catalog, owner), 1)
            } else {
                let kind = catalog.card_kind(c);
                let ts = zhaogujing_targets_for(core, catalog, owner, kind);
                let m = if kind == CardKind::Chain {
                    ts.len().min(2)
                } else {
                    1
                };
                (ts, m)
            };
            let opts = targets.into_iter().map(ChoiceOption::Player).collect();
            spec_ask(
                owner,
                ChoiceKind::SelectPlayers,
                "zhaogujing:choose_target",
                opts,
                1,
                max,
                true,
            )
        }
        ASK_SG_ZHAOGUJING_JIEDAO_VICTIM => {
            let holder = target?;
            let opts = zhaogujing_jiedao_victims(core, catalog, holder)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            spec_ask(
                owner,
                ChoiceKind::SelectPlayers,
                "zhaogujing:choose_target",
                opts,
                1,
                1,
                true,
            )
        }
        ASK_SG_ZHAOGUJING_HUOGONG_DISCARD => {
            let t = target?;
            let &shown = core.player(t).hand.first()?;
            let suit = catalog.effective_card_suit(core, t, shown);
            let opts = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&hc| catalog.effective_card_suit(core, owner, hc) == suit)
                .map(ChoiceOption::Card)
                .collect();
            spec_ask(
                owner,
                ChoiceKind::SelectCards,
                "zhaogujing:choose_card",
                opts,
                1,
                1,
                true,
            )
        }
        _ => None,
    }
}
