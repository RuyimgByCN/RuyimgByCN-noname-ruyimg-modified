use super::*;
use crate::choice::{ChoiceKind, ChoiceOption};
use crate::snap::{ask, ExpectedAsk};

pub(in crate::content::host) fn anruo_usable_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<CardId> {
    anruo_usable_cards_for_ctx(core, catalog, owner, 0)
}

pub(in crate::content::host) fn anruo_usable_cards_for_ctx(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    resp_ctx: u32,
) -> Vec<CardId> {
    if catalog.blocks_all_cards(core, owner) {
        return Vec::new();
    }
    let tao_futai2 = crate::kernel::tao_futai2_banned(core, catalog, owner);
    let can_tao = !tao_futai2 && core.player(owner).hp < core.player(owner).max_hp as i32;
    let (_, _, sha_tgts) = core.legal_targets(catalog, CardKind::FireSha, owner);
    let sha_ok = !core.turn.sha_used_play_phase
        || super::super::l10_reguhuo::has_unlimited_sha(core, catalog, owner);
    let can_sha = sha_ok && !sha_tgts.is_empty();
    let chihe_block_mask = core
        .vm_aux
        .flags
        .get(&(owner, "l10_scschihe_block_suits".to_string()))
        .copied()
        .unwrap_or(0);
    core.player(owner)
        .hand
        .iter()
        .chain(core.player(owner).equips.iter())
        .copied()
        .filter(|&c| {
            let s = catalog.effective_card_suit(core, owner, c);
            match resp_ctx {
                1 => s == Suit::Club && (chihe_block_mask & suit_bit(Suit::Club)) == 0,
                2 => s == Suit::Diamond,
                3 => s == Suit::Spade,
                4 | 5 => s == Suit::Heart && !tao_futai2,
                _ => match s {
                    Suit::Heart => can_tao,
                    Suit::Diamond => can_sha,
                    _ => false,
                },
            }
        })
        .collect()
}

pub(in crate::content::host) fn scsanruo_steal_target_candidates(
    core: &GameStateCore,
    owner: PlayerId,
) -> Vec<PlayerId> {
    core.alive_players()
        .filter(|&p| {
            p != owner && (!core.player(p).hand.is_empty() || !core.player(p).equips.is_empty())
        })
        .collect()
}

pub(in crate::content::host) fn scsanruo_steal_pool(
    core: &GameStateCore,
    target: PlayerId,
    equip_only: bool,
) -> Vec<CardId> {
    if !core.player(target).alive {
        return Vec::new();
    }
    let pt = core.player(target);
    if equip_only {
        pt.equips.clone()
    } else {
        pt.hand.iter().chain(pt.equips.iter()).copied().collect()
    }
}

pub(in crate::content::host) fn kuiji_card_options(
    core: &GameStateCore,
    _catalog: &Catalog,
    owner: PlayerId,
    target: PlayerId,
    _chosen: &[CardId],
) -> Vec<ChoiceOption> {
    let mut opts = Vec::new();
    for &c in &core.player(owner).hand {
        opts.push(ChoiceOption::Card(c));
    }
    for &c in &core.player(target).hand {
        opts.push(ChoiceOption::Card(c));
    }
    opts
}

#[allow(clippy::too_many_arguments)]
pub(in crate::content::host) fn ask_l10_scs(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    _targets: &[PlayerId],
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let ask_id = card?;
    match (ask_id, stage) {
        (ASK_SCSNIQU_TARGET, HostStage::TargetChoice) => {
            let opts: Vec<ChoiceOption> = core.alive_players().map(ChoiceOption::Player).collect();
            (!opts.is_empty()).then(|| {
                ask(
                    owner,
                    "scsniqu_target",
                    ChoiceKind::SelectPlayers,
                    opts,
                    1,
                    1,
                    true,
                )
            })
        }
        (ASK_SCSTAOLUAN_CARD, HostStage::SelectCard) => {
            let p = core.player(owner);
            let opts: Vec<ChoiceOption> = p
                .hand
                .iter()
                .chain(p.equips.iter())
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            (!opts.is_empty()).then(|| {
                ask(
                    owner,
                    "scstaoluan_card",
                    ChoiceKind::SelectCards,
                    opts,
                    1,
                    1,
                    true,
                )
            })
        }
        (ASK_SCSTAOLUAN_NAME, HostStage::SelectNature) => {
            let opts = super::super::l10_reguhuo::reguhuo_name_options(core, catalog, owner, 0);
            (!opts.is_empty()).then(|| {
                ask(
                    owner,
                    "scstaoluan_name",
                    ChoiceKind::PickTag,
                    opts,
                    1,
                    1,
                    true,
                )
            })
        }
        (ASK_SCSTAOLUAN_TARGETS, HostStage::TargetChoice) => {
            let idx = *cards.get(1)?;
            let kind = super::super::l10_reguhuo::reguhuo_kind_at_catalog(catalog, idx as u8)?;
            if kind == CardKind::Jiedao {
                if let Some(holder) = target {
                    let victims = core.legal_jiedao_victims(catalog, holder);
                    return (!victims.is_empty()).then(|| {
                        ask(
                            owner,
                            "scstaoluan_jiedao_victim",
                            ChoiceKind::SelectPlayers,
                            victims.into_iter().map(ChoiceOption::Player).collect(),
                            1,
                            1,
                            true,
                        )
                    });
                }
            }
            let (min, max, tgts) = core.legal_targets(catalog, kind, owner);
            (tgts.len() >= min as usize).then(|| {
                ask(
                    owner,
                    "scstaoluan_targets",
                    ChoiceKind::SelectPlayers,
                    tgts.into_iter().map(ChoiceOption::Player).collect(),
                    min as usize,
                    max as usize,
                    true,
                )
            })
        }
        (ASK_SCSCHIYAN_PICK, HostStage::SelectCard) => {
            let t = target?;
            let pt = core.player(t);
            let opts: Vec<ChoiceOption> = pt
                .hand
                .iter()
                .chain(pt.equips.iter())
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            (!opts.is_empty()).then(|| {
                ask(
                    owner,
                    "scschiyan_pick",
                    ChoiceKind::SelectCards,
                    opts,
                    0,
                    1,
                    true,
                )
            })
        }
        (ASK_SCSCHIHE_CONFIRM, HostStage::SelectCard) => Some(ask(
            owner,
            "scschihe_confirm",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm],
            0,
            1,
            true,
        )),
        (ASK_SCSPICAI_CONTINUE, HostStage::SelectNature) => Some(ask(
            owner,
            "scspicai_continue",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm],
            0,
            1,
            true,
        )),
        (ASK_SCSPICAI_TARGET, HostStage::TargetChoice) => {
            let opts: Vec<ChoiceOption> = core.alive_players().map(ChoiceOption::Player).collect();
            (!opts.is_empty()).then(|| {
                ask(
                    owner,
                    "scspicai_target",
                    ChoiceKind::SelectPlayers,
                    opts,
                    1,
                    1,
                    false,
                )
            })
        }
        (ASK_SCSYAOZHUO_TARGET, HostStage::TargetChoice) => {
            let opts: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| p != owner && !core.player(p).hand.is_empty())
                .map(ChoiceOption::Player)
                .collect();
            (!opts.is_empty()).then(|| {
                ask(
                    owner,
                    "scsyaozhuo_target",
                    ChoiceKind::SelectPlayers,
                    opts,
                    1,
                    1,
                    true,
                )
            })
        }
        (ASK_SCSYAOZHUO_OWN_CARD, HostStage::ComparePlayerPick) => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            (!opts.is_empty()).then(|| {
                ask(
                    owner,
                    "scsyaozhuo_pindian_own",
                    ChoiceKind::SelectCards,
                    opts,
                    1,
                    1,
                    false,
                )
            })
        }
        (ASK_SCSYAOZHUO_TGT_CARD, HostStage::CompareTargetPick) => {
            let t = target?;
            let opts: Vec<ChoiceOption> = core
                .player(t)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            (!opts.is_empty()).then(|| {
                ask(
                    t,
                    "scsyaozhuo_pindian_target",
                    ChoiceKind::SelectCards,
                    opts,
                    1,
                    1,
                    false,
                )
            })
        }
        (ASK_SCSYAOZHUO_DISCARD, HostStage::SelectCard) => {
            let p = core.player(owner);
            let opts: Vec<ChoiceOption> = p
                .hand
                .iter()
                .chain(p.equips.iter())
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            (opts.len() >= 2).then(|| {
                ask(
                    owner,
                    "scsyaozhuo_discard",
                    ChoiceKind::SelectCards,
                    opts,
                    2,
                    2,
                    false,
                )
            })
        }
        (ASK_SCSXIAOLU_MODE, HostStage::SelectNature) => {
            let has_other = core.alive_players().any(|p| p != owner);
            let opts = if has_other {
                vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)]
            } else {
                vec![ChoiceOption::Tag(1)]
            };
            Some(ask(
                owner,
                "scsxiaolu_mode",
                ChoiceKind::PickTag,
                opts,
                1,
                1,
                false,
            ))
        }
        (ASK_SCSXIAOLU_CARDS, HostStage::CardsAsk) => {
            let p = core.player(owner);
            let opts: Vec<ChoiceOption> = p
                .hand
                .iter()
                .chain(p.equips.iter())
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            (opts.len() >= 2).then(|| {
                ask(
                    owner,
                    "scsxiaolu_cards",
                    ChoiceKind::SelectCards,
                    opts,
                    2,
                    2,
                    false,
                )
            })
        }
        (ASK_SCSXIAOLU_TARGET, HostStage::TargetChoice) => {
            let opts: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| p != owner)
                .map(ChoiceOption::Player)
                .collect();
            (!opts.is_empty()).then(|| {
                ask(
                    owner,
                    "scsxiaolu_target",
                    ChoiceKind::SelectPlayers,
                    opts,
                    1,
                    1,
                    false,
                )
            })
        }
        (ASK_SCSKUIJI_TARGET, HostStage::TargetChoice) => {
            let opts: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| p != owner && !core.player(p).hand.is_empty())
                .map(ChoiceOption::Player)
                .collect();
            (!opts.is_empty()).then(|| {
                ask(
                    owner,
                    "scskuiji_target",
                    ChoiceKind::SelectPlayers,
                    opts,
                    1,
                    1,
                    true,
                )
            })
        }
        (ASK_SCSKUIJI_CARDS, HostStage::SelectCard) => {
            let t = target?;
            let opts = kuiji_card_options(core, catalog, owner, t, cards);
            (!opts.is_empty()).then(|| {
                ask(
                    owner,
                    "scskuiji_cards",
                    ChoiceKind::SelectCards,
                    opts,
                    4,
                    4,
                    true,
                )
            })
        }
        (ASK_SCSANRUO_CARD, HostStage::SelectCard) => {
            let resp_ctx = cards.first().copied().unwrap_or(0);
            let opts: Vec<ChoiceOption> =
                anruo_usable_cards_for_ctx(core, catalog, owner, resp_ctx)
                    .into_iter()
                    .map(ChoiceOption::Card)
                    .collect();
            (!opts.is_empty()).then(|| {
                ask(
                    owner,
                    "scsanruo_card",
                    ChoiceKind::SelectCards,
                    opts,
                    1,
                    1,
                    true,
                )
            })
        }
        (ASK_SCSANRUO_TARGET, HostStage::TargetChoice) => {
            let (_, _, tgts) = core.legal_targets(catalog, CardKind::FireSha, owner);
            (!tgts.is_empty()).then(|| {
                ask(
                    owner,
                    "scsanruo_target",
                    ChoiceKind::SelectPlayers,
                    tgts.into_iter().map(ChoiceOption::Player).collect(),
                    1,
                    1,
                    true,
                )
            })
        }
        (ASK_SCS_ORDER, HostStage::SelectNature) => Some(ask(
            owner,
            "l10_scschiyan_scschihe_order",
            ChoiceKind::PickTag,
            vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
            1,
            1,
            false,
        )),
        (ASK_SCSANRUO_STEAL_CONFIRM, HostStage::CardsAsk) => Some(ask(
            owner,
            "scsanruo_steal_confirm",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm],
            0,
            1,
            true,
        )),
        (ASK_SCSANRUO_STEAL_TARGET, HostStage::TargetChoice) => {
            let opts: Vec<ChoiceOption> = scsanruo_steal_target_candidates(core, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            (!opts.is_empty()).then(|| {
                ask(
                    owner,
                    "scsanruo_steal_target",
                    ChoiceKind::SelectPlayers,
                    opts,
                    0,
                    1,
                    true,
                )
            })
        }
        (ASK_SCSANRUO_STEAL_CARD, HostStage::SelectCard) => {
            let t = target?;
            let equip_only = cards.get(1).copied().unwrap_or(0) == 1;
            let opts: Vec<ChoiceOption> = scsanruo_steal_pool(core, t, equip_only)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            (!opts.is_empty()).then(|| {
                ask(
                    owner,
                    "scsanruo_steal_card",
                    ChoiceKind::SelectCards,
                    opts,
                    1,
                    1,
                    false,
                )
            })
        }
        _ => None,
    }
}
