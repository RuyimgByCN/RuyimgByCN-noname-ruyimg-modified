//! U05-5a (`杀海拾遗·gwent`) 快照恢复路径 `ChoiceSpec` 重建（`asks.rs` / `asks_mtg.rs` 调用）。

use super::super::super::*;
use super::adventure_a::luoqi_sha_options;
use super::adventure_b::{
    airuiting_sha_options, airuiting_sha_targets_for_card, fulisi_card_options,
};
use super::adventure_huoge::{huoge_card_can_use, huoge_card_targets, jiedao_victim_options};
use super::silver_spells::{
    eligible_bronze_tags, eligible_bronze_targets, eligible_delay_options,
};
use super::{
    clean_gwent_name, BRONZE_SPELL_NAMES, GOLD_SPELL_PICK_NAMES, SILVER_SPELL_PICK_NAMES,
};
use crate::catalog::Catalog;
use crate::snap::ExpectedAsk;
use crate::state::{GameMode, GameStateCore};

fn ask(
    observer: PlayerId,
    prompt: &'static str,
    kind: ChoiceKind,
    options: Vec<ChoiceOption>,
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> ExpectedAsk {
    ExpectedAsk {
        observer,
        prompt,
        kind,
        options,
        min,
        max,
        allow_cancel,
        source_skill: None,
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::content::host) fn ask_u05_gwent_card(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let cname = card.and_then(|c| catalog.cards.get(c as usize).map(|d| clean_gwent_name(d.name)))?;
    match (cname, stage) {
        ("gw_xinsheng", HostStage::SelectCard) => {
            let n = core
                .vm_aux
                .flags
                .get(&(owner, "u05_gw_xinsheng_n".to_string()))
                .copied()
                .unwrap_or(0);
            Some(ask(
                owner,
                "gw_xinsheng_general",
                ChoiceKind::PickTag,
                (0..n as u8).map(ChoiceOption::Tag).collect(),
                1,
                1,
                false,
            ))
        }
        ("gw_xinsheng", HostStage::TargetChoice) => {
            let opts: Vec<ChoiceOption> = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| {
                    core.player(p).alive
                        && (!matches!(core.mode, GameMode::Kingdoms)
                            || core.player(p).revealed_generals[0]
                            || core.player(p).revealed_generals[1])
                })
                .map(ChoiceOption::Player)
                .collect();
            Some(ask(
                owner,
                "gw_xinsheng_target",
                ChoiceKind::SelectPlayers,
                opts,
                1,
                1,
                false,
            ))
        }
        ("gw_huangjiashenpan", HostStage::SelectCard) => {
            let opts: Vec<ChoiceOption> = GOLD_SPELL_PICK_NAMES
                .iter()
                .enumerate()
                .filter(|(_, name)| {
                    core.deck
                        .iter()
                        .chain(core.discard.iter())
                        .any(|c| clean_gwent_name(catalog.cards[*c as usize].name) == **name)
                })
                .map(|(i, _)| ChoiceOption::Tag(i as u8))
                .collect();
            Some(ask(
                owner,
                "gw_huangjiashenpan_pick",
                ChoiceKind::PickTag,
                opts,
                1,
                1,
                false,
            ))
        }
        ("gw_tongdi", HostStage::SelectCard) => {
            let t = target?;
            Some(ask(
                owner,
                "gw_tongdi_pick",
                ChoiceKind::SelectCards,
                core.player(t).hand.iter().copied().map(ChoiceOption::Card).collect(),
                1,
                1,
                false,
            ))
        }
        ("gw_butianshu", HostStage::SelectNature) => {
            let t = target?;
            Some(ask(
                owner,
                "gw_butianshu_delay",
                ChoiceKind::PickTag,
                eligible_delay_options(core, catalog, t),
                1,
                1,
                false,
            ))
        }
        ("gw_zuihouyuanwang", HostStage::SelectCard) => {
            let n = core
                .vm_aux
                .flags
                .get(&(owner, "u05_gw_zuihou_n".to_string()))
                .copied()
                .unwrap_or(0) as usize;
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .chain(core.player(owner).equips.iter())
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            let req = n.min(opts.len());
            Some(ask(
                owner,
                "gw_zuihouyuanwang_discard",
                ChoiceKind::SelectCards,
                opts,
                req,
                req,
                false,
            ))
        }
        ("gw_zirankuizeng", HostStage::SelectCard) => Some(ask(
            owner,
            "gw_zirankuizeng_spell",
            ChoiceKind::PickTag,
            eligible_bronze_tags(core, catalog, owner),
            1,
            1,
            false,
        )),
        ("gw_zirankuizeng", HostStage::TargetChoice) => {
            let tag = core
                .vm_aux
                .flags
                .get(&(owner, "u05_gw_ziran_tag".to_string()))
                .copied()
                .unwrap_or(0);
            let &(_, kind) = BRONZE_SPELL_NAMES.get(tag as usize)?;
            Some(ask(
                owner,
                "gw_zirankuizeng_target",
                ChoiceKind::SelectPlayers,
                eligible_bronze_targets(core, catalog, owner, kind),
                1,
                1,
                false,
            ))
        }
        ("gwmaoxian_luoqi", HostStage::SelectCard) => {
            let responder = target?;
            Some(ask(
                responder,
                "gwmaoxian_luoqi_sha",
                ChoiceKind::SelectCards,
                luoqi_sha_options(core, catalog, responder),
                1,
                1,
                true,
            ))
        }
        ("gwmaoxian_fulisi", HostStage::SelectCard) => {
            let opts = fulisi_card_options(core, targets);
            let max = opts.len().min(2);
            Some(ask(
                owner,
                "gwmaoxian_fulisi_discard",
                ChoiceKind::SelectCards,
                opts,
                1,
                max,
                true,
            ))
        }
        ("gwmaoxian_kaerweite", HostStage::SelectCard) => {
            let t = target?;
            Some(ask(
                owner,
                "gwmaoxian_kaerweite_pick",
                ChoiceKind::SelectCards,
                core.player(t).hand.iter().copied().map(ChoiceOption::Card).collect(),
                1,
                1,
                false,
            ))
        }
        ("gwmaoxian_bulanwang", HostStage::SelectCard) => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .chain(core.player(owner).equips.iter())
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            let max = opts.len().min(2);
            Some(ask(
                owner,
                "gwmaoxian_bulanwang_discard",
                ChoiceKind::SelectCards,
                opts,
                1,
                max,
                true,
            ))
        }
        ("gwmaoxian_gaier", HostStage::SelectNature) => Some(ask(
            owner,
            "gwmaoxian_gaier_branch",
            ChoiceKind::PickTag,
            vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
            1,
            1,
            false,
        )),
        ("gwmaoxian_airuiting", HostStage::SelectCard) => {
            let p = target?;
            Some(ask(
                p,
                "gwmaoxian_airuiting_sha",
                ChoiceKind::SelectCards,
                airuiting_sha_options(core, catalog, p),
                1,
                1,
                true,
            ))
        }
        ("gwmaoxian_airuiting", HostStage::TargetChoice) => {
            let p = target?;
            let sha = core
                .vm_aux
                .flags
                .get(&(owner, "u05_gw_air_sha".to_string()))
                .and_then(|v| v.checked_sub(1));
            Some(ask(
                p,
                "gwmaoxian_airuiting_target",
                ChoiceKind::SelectPlayers,
                airuiting_sha_targets_for_card(core, catalog, p, sha),
                1,
                1,
                true,
            ))
        }
        ("gwmaoxian_aisinie", HostStage::SelectCard) => {
            let opts: Vec<ChoiceOption> = SILVER_SPELL_PICK_NAMES
                .iter()
                .enumerate()
                .filter(|(_, n)| {
                    core.deck
                        .iter()
                        .chain(core.discard.iter())
                        .any(|c| clean_gwent_name(catalog.cards[*c as usize].name) == **n)
                })
                .map(|(i, _)| ChoiceOption::Tag(i as u8))
                .collect();
            Some(ask(
                owner,
                "gwmaoxian_aisinie_pick",
                ChoiceKind::PickTag,
                opts,
                1,
                1,
                false,
            ))
        }
        ("gwmaoxian_falanxisika", HostStage::SelectCard) => {
            let n = core
                .vm_aux
                .flags
                .get(&(owner, "u05_gw_falan_n".to_string()))
                .copied()
                .unwrap_or(0);
            let mut opts = Vec::new();
            for i in 0..n {
                if let Some(tag) = core
                    .vm_aux
                    .flags
                    .get(&(owner, format!("u05_gw_falan_g:{i}")))
                    .copied()
                {
                    opts.push(ChoiceOption::Tag(tag as u8));
                }
            }
            Some(ask(
                owner,
                "gwmaoxian_falanxisika_pick",
                ChoiceKind::PickTag,
                opts,
                1,
                1,
                false,
            ))
        }
        ("gwmaoxian_huoge", HostStage::SelectCard) => {
            let usable: Vec<ChoiceOption> = cards
                .iter()
                .copied()
                .filter(|&c| huoge_card_can_use(core, catalog, owner, c))
                .map(ChoiceOption::Card)
                .collect();
            let max = usable.len().min(2);
            Some(ask(
                owner,
                "gwmaoxian_huoge_pick",
                ChoiceKind::SelectCards,
                usable,
                1,
                max,
                true,
            ))
        }
        ("gwmaoxian_huoge", HostStage::SelectNature) => {
            let &c = cards.first()?;
            let (opts, min_t, max_t) = huoge_card_targets(core, catalog, owner, c)?;
            Some(ask(
                owner,
                "gwmaoxian_huoge_target",
                ChoiceKind::SelectPlayers,
                opts,
                min_t,
                max_t,
                false,
            ))
        }
        ("gwmaoxian_huoge", HostStage::CardsAsk) => {
            let holder = target?;
            Some(ask(
                owner,
                "gwmaoxian_huoge_target",
                ChoiceKind::SelectPlayers,
                jiedao_victim_options(core, catalog, owner, holder),
                1,
                1,
                false,
            ))
        }
        _ => None,
    }
}

pub(in crate::content::host) fn ask_u05_gwent_skill(
    stage: HostStage,
    owner: PlayerId,
    core: &GameStateCore,
) -> Option<ExpectedAsk> {
    if stage != HostStage::SelectCard {
        return None;
    }
    let opts: Vec<ChoiceOption> = core
        .player(owner)
        .hand
        .iter()
        .chain(core.player(owner).equips.iter())
        .copied()
        .map(ChoiceOption::Card)
        .collect();
    Some(ask(
        owner,
        "gw_baobaoshu_discard",
        ChoiceKind::SelectCards,
        opts,
        1,
        1,
        false,
    ))
}
