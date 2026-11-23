//! U05 Batch 7c 询问派生（pending 快照恢复 / `open_choice` 自省重建）。

use super::super::*;
use crate::catalog::Catalog;
use crate::snap::*;
use crate::state::GameStateCore;

#[allow(clippy::too_many_arguments)]
pub(in crate::content::host) fn ask_u05_7c(
    behavior: HostBehavior,
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    _targets: &[PlayerId],
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let hand_opts = |p: PlayerId| -> Vec<ChoiceOption> {
        core.player(p)
            .hand
            .iter()
            .copied()
            .map(ChoiceOption::Card)
            .collect()
    };
    let he_opts = |p: PlayerId| -> Vec<ChoiceOption> {
        core.player(p)
            .hand
            .iter()
            .chain(core.player(p).equips.iter())
            .copied()
            .map(ChoiceOption::Card)
            .collect()
    };
    match behavior {
        HostBehavior::SwdKbolan if stage == HostStage::CompareTargetPick => Some(ask(
            owner,
            "kbolan_confirm",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm],
            1,
            1,
            true,
        )),
        HostBehavior::SwdKbolan2 if stage == HostStage::Act => Some(ask(
            owner,
            "kbolan2_top",
            ChoiceKind::SelectCards,
            hand_opts(owner),
            1,
            1,
            false,
        )),
        HostBehavior::SwdTouxi if stage == HostStage::SelectCard => Some(ask(
            target?,
            "touxi_discard",
            ChoiceKind::SelectCards,
            he_opts(owner),
            1,
            1,
            true,
        )),
        HostBehavior::SwdTiebi if stage == HostStage::SelectCard => Some(ask(
            owner,
            "tiebi_discard",
            ChoiceKind::SelectCards,
            super::u05_7c_skills::tiebi_black_hand_options(core, catalog, owner),
            1,
            1,
            true,
        )),
        HostBehavior::SwdSenluo if stage == HostStage::SelectCard => Some(ask(
            target?,
            "senluo_discard",
            ChoiceKind::SelectCards,
            hand_opts(target?),
            2,
            2,
            false,
        )),
        HostBehavior::SwdLingxian if stage == HostStage::TargetChoice => {
            let options = core
                .alive_players()
                .filter(|p| *p != owner && core.computed_distance(catalog, owner, *p) > 1)
                .map(ChoiceOption::Player)
                .collect();
            Some(ask(
                owner,
                "lingxian_target",
                ChoiceKind::SelectPlayers,
                options,
                1,
                1,
                true,
            ))
        }
        HostBehavior::SwdGuaili if stage == HostStage::SelectCard => Some(ask(
            owner,
            "guaili2_discard",
            ChoiceKind::SelectCards,
            hand_opts(owner),
            2,
            2,
            false,
        )),
        HostBehavior::SwdLingyue => match stage {
            HostStage::CompareTargetPick => Some(ask(
                owner,
                "lingyue_confirm",
                ChoiceKind::Confirm,
                vec![ChoiceOption::Confirm],
                1,
                1,
                true,
            )),
            HostStage::SelectCard => Some(ask(
                target?,
                "lingyue_discard",
                ChoiceKind::SelectCards,
                he_opts(target?),
                1,
                1,
                false,
            )),
            _ => None,
        },
        HostBehavior::SwdLinyun => match stage {
            HostStage::SelectNature => {
                let t = target?;
                let opts = core
                    .player(t)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| catalog.card_kind(*c) == CardKind::Shan)
                    .map(ChoiceOption::Card)
                    .collect();
                Some(ask(t, "linyun2_shan", ChoiceKind::SelectCards, opts, 1, 1, true))
            }
            HostStage::CardsAsk => Some(ask(
                owner,
                "linyun_pick_two",
                ChoiceKind::SelectCards,
                he_opts(owner),
                2,
                2,
                true,
            )),
            HostStage::TargetChoice => {
                let opts = core
                    .alive_players()
                    .filter(|p| {
                        *p != owner
                            && core.computed_distance(catalog, owner, *p) <= core.attack_range(catalog, owner)
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                Some(ask(owner, "linyun_sha_target", ChoiceKind::SelectPlayers, opts, 1, 1, true))
            }
            _ => None,
        },
        HostBehavior::SwdJiefen => match stage {
            HostStage::JiefenTargetDiscard => {
                let my = core.player(owner).hand.len();
                let opts = core
                    .alive_players()
                    .filter(|p| *p != owner && core.player(*p).hand.len() > my)
                    .map(ChoiceOption::Player)
                    .collect();
                Some(ask(owner, "swd_jiefen_target", ChoiceKind::SelectPlayers, opts, 1, 1, true))
            }
            HostStage::JiefenOwnerDiscard => Some(ask(
                target?,
                "swd_jiefen_give",
                ChoiceKind::SelectCards,
                hand_opts(target?),
                1,
                1,
                false,
            )),
            HostStage::SelectCard => Some(ask(
                owner,
                "swd_jiefen_give_card",
                ChoiceKind::SelectCards,
                hand_opts(owner),
                1,
                1,
                false,
            )),
            HostStage::CompareTargetPick => {
                let my = core.player(owner).hand.len();
                let opts = core
                    .alive_players()
                    .filter(|p| *p != owner && core.player(*p).hand.len() < my)
                    .map(ChoiceOption::Player)
                    .collect();
                Some(ask(owner, "swd_jiefen_give_target", ChoiceKind::SelectPlayers, opts, 1, 1, false))
            }
            _ => None,
        },
        HostBehavior::SwdNlianji => match stage {
            HostStage::SelectCard => Some(ask(
                owner,
                "nlianji_throw",
                ChoiceKind::SelectCards,
                hand_opts(owner),
                1,
                1,
                true,
            )),
            HostStage::CardsAsk => {
                let opts = core
                    .alive_players()
                    .filter(|p| *p != owner && !core.player(*p).hand.is_empty())
                    .map(ChoiceOption::Player)
                    .collect();
                Some(ask(owner, "nlianji_target0", ChoiceKind::SelectPlayers, opts, 1, 1, true))
            }
            HostStage::CompareTargetPick => {
                let t0 = target?;
                let opts = core
                    .alive_players()
                    .filter(|p| *p != owner && *p != t0 && !core.player(*p).hand.is_empty())
                    .map(ChoiceOption::Player)
                    .collect();
                Some(ask(owner, "nlianji_target1", ChoiceKind::SelectPlayers, opts, 1, 1, true))
            }
            HostStage::ComparePlayerPick => Some(ask(
                target?,
                "nlianji_t0_pick",
                ChoiceKind::SelectCards,
                hand_opts(target?),
                1,
                1,
                false,
            )),
            HostStage::JiefenTarget => Some(ask(
                target?,
                "nlianji_t1_pick",
                ChoiceKind::SelectCards,
                hand_opts(target?),
                1,
                1,
                false,
            )),
            _ => None,
        },
        HostBehavior::SwdDuishi => match stage {
            HostStage::SelectCard => Some(ask(
                owner,
                "duishi_discard",
                ChoiceKind::SelectCards,
                hand_opts(owner),
                1,
                1,
                true,
            )),
            HostStage::TargetChoice => {
                let opts = core
                    .alive_players()
                    .filter(|p| {
                        *p != owner
                            && !core.player(*p).hand.is_empty()
                            && !core.vm_aux.flags.contains_key(&(*p, "swd_duishi3".to_string()))
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                Some(ask(owner, "duishi_target", ChoiceKind::SelectPlayers, opts, 1, 1, true))
            }
            HostStage::CardsAsk => {
                let t = target?;
                let suit = catalog.effective_card_suit(core, owner, card?);
                let opts = core
                    .player(t)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| catalog.effective_card_suit(core, t, *c) == suit)
                    .map(ChoiceOption::Card)
                    .collect();
                Some(ask(
                    t,
                    "duishi_discard_same_suit",
                    ChoiceKind::SelectCards,
                    opts,
                    1,
                    1,
                    true,
                ))
            }
            HostStage::CompareTargetPick => Some(ask(
                owner,
                "duishi_gain",
                ChoiceKind::SelectCards,
                he_opts(target?),
                1,
                1,
                false,
            )),
            _ => None,
        },
        HostBehavior::SwdYuchen if stage == HostStage::TargetChoice => {
            let opts = core
                .alive_players()
                .filter(|p| *p != owner && (!core.player(*p).hand.is_empty() || !core.player(*p).equips.is_empty()))
                .map(ChoiceOption::Player)
                .collect();
            Some(ask(owner, "yuchen_target", ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        HostBehavior::SwdYuchen if stage == HostStage::SelectCard => {
            let t = target?;
            Some(ask(owner, "yuchen_discard", ChoiceKind::SelectCards, he_opts(t), 1, 1, false))
        }
        HostBehavior::SwdPozhou2 if stage == HostStage::TargetChoice => {
            let opts: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|p| {
                    *p != owner
                        && !catalog
                            .skill_by_name(SWD_POZHOU_SEALED_KEY)
                            .is_some_and(|d| core.has_skill(*p, d))
                })
                .map(ChoiceOption::Player)
                .collect();
            let cap = (core.player(owner).pozhou_count as usize).min(opts.len()).max(1);
            Some(ask(owner, "pozhou_target", ChoiceKind::SelectPlayers, opts, 1, cap, true))
        }
        _ => ask_u05_7c_extra(
            behavior, stage, owner, target, _targets, card, cards, core, catalog,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn ask_u05_7c_extra(
    behavior: HostBehavior,
    stage: HostStage,
    owner: PlayerId,
    _target: Option<PlayerId>,
    _targets: &[PlayerId],
    _card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let hand_opts = |p: PlayerId| -> Vec<ChoiceOption> {
        core.player(p)
            .hand
            .iter()
            .copied()
            .map(ChoiceOption::Card)
            .collect()
    };
    let he_opts = |p: PlayerId| -> Vec<ChoiceOption> {
        core.player(p)
            .hand
            .iter()
            .chain(core.player(p).equips.iter())
            .copied()
            .map(ChoiceOption::Card)
            .collect()
    };
    match behavior {
        HostBehavior::SwdHuanxing => match stage {
            HostStage::SelectCard => Some(ask(
                owner,
                "huanxing_discard",
                ChoiceKind::SelectCards,
                he_opts(owner),
                1,
                1,
                true,
            )),
            HostStage::TargetChoice => {
                let opts = super::swd_card_targets::huanxing_candidate_targets(core, catalog, owner)
                    .into_iter()
                    .map(ChoiceOption::Player)
                    .collect();
                Some(ask(owner, "huanxing_target", ChoiceKind::SelectPlayers, opts, 1, 1, true))
            }
            _ => None,
        },
        HostBehavior::SwdAojian => match stage {
            HostStage::SelectCard => {
                let x = (core.player(owner).max_hp as i32 - core.player(owner).hp).max(0) as usize;
                let in_range = core
                    .alive_players()
                    .filter(|p| *p != owner && core.computed_distance(catalog, owner, *p) <= 1)
                    .count();
                let n = x.min(core.player(owner).hand.len()).min(in_range).max(1);
                Some(ask(
                    owner,
                    "aojian_discard",
                    ChoiceKind::SelectCards,
                    hand_opts(owner),
                    1,
                    n,
                    true,
                ))
            }
            HostStage::TargetChoice => {
                let opts = core
                    .alive_players()
                    .filter(|p| *p != owner && core.computed_distance(catalog, owner, *p) <= 1)
                    .map(ChoiceOption::Player)
                    .collect();
                let n = cards.len().max(1);
                Some(ask(owner, "aojian_targets", ChoiceKind::SelectPlayers, opts, n, n, true))
            }
            _ => None,
        },
        HostBehavior::SwdNingxian => match stage {
            HostStage::SelectCard => {
                let blacks = core
                    .player(owner)
                    .hand
                    .iter()
                    .chain(core.player(owner).equips.iter())
                    .copied()
                    .filter(|c| {
                        matches!(
                            catalog.effective_card_suit(core, owner, *c),
                            crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                        )
                    })
                    .map(ChoiceOption::Card)
                    .collect::<Vec<_>>();
                let max = blacks
                    .len()
                    .min(core.alive_players().filter(|p| *p != owner).count())
                    .max(1);
                Some(ask(
                    owner,
                    "ningxian_discard",
                    ChoiceKind::SelectCards,
                    blacks,
                    1,
                    max,
                    true,
                ))
            }
            HostStage::TargetChoice => {
                let opts = core
                    .alive_players()
                    .filter(|p| *p != owner)
                    .map(ChoiceOption::Player)
                    .collect();
                let n = cards.len().max(1);
                Some(ask(owner, "ningxian_targets", ChoiceKind::SelectPlayers, opts, n, n, true))
            }
            _ => None,
        },
        HostBehavior::SwdHuodan => match stage {
            HostStage::SelectCard => {
                let reds = core
                    .player(owner)
                    .hand
                    .iter()
                    .chain(core.player(owner).equips.iter())
                    .copied()
                    .filter(|c| {
                        matches!(
                            catalog.effective_card_suit(core, owner, *c),
                            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                        )
                    })
                    .map(ChoiceOption::Card)
                    .collect();
                Some(ask(owner, "huodan_pick", ChoiceKind::SelectCards, reds, 1, 1, true))
            }
            HostStage::TargetChoice => {
                let opts = core.alive_players().map(ChoiceOption::Player).collect();
                Some(ask(owner, "huodan_target", ChoiceKind::SelectPlayers, opts, 1, 2, true))
            }
            _ => None,
        },
        HostBehavior::SwdZhanxing => match stage {
            HostStage::SelectCard => {
                let opts = he_opts(owner);
                let max = opts.len().max(1);
                Some(ask(
                    owner,
                    "zhanxing_discard",
                    ChoiceKind::SelectCards,
                    opts,
                    1,
                    max,
                    true,
                ))
            }
            HostStage::TargetChoice => {
                let opts = core.alive_players().map(ChoiceOption::Player).collect();
                Some(ask(owner, "zhanxing_target", ChoiceKind::SelectPlayers, opts, 1, 1, true))
            }
            _ => None,
        },
        HostBehavior::SwdJianji => match stage {
            HostStage::SelectCard => {
                let opts = core
                    .player(owner)
                    .hand
                    .iter()
                    .chain(core.player(owner).equips.iter())
                    .copied()
                    .filter(|c| catalog.card_kind(*c).is_equip())
                    .map(ChoiceOption::Card)
                    .collect();
                Some(ask(owner, "jianji_pick", ChoiceKind::SelectCards, opts, 1, 1, true))
            }
            HostStage::TargetChoice => {
                let opts = core
                    .alive_players()
                    .filter(|p| {
                        *p != owner
                            && core.computed_distance(catalog, owner, *p)
                                <= core.attack_range(catalog, owner)
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                Some(ask(owner, "jianji_target", ChoiceKind::SelectPlayers, opts, 1, 1, true))
            }
            _ => None,
        },
        HostBehavior::SwdBingjian => match stage {
            HostStage::SelectCard => {
                let opts = core
                    .player(owner)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| {
                        catalog.card_kind(*c).is_sha_like()
                            && matches!(
                                catalog.effective_card_suit(core, owner, *c),
                                crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                            )
                    })
                    .map(ChoiceOption::Card)
                    .collect();
                Some(ask(owner, "bingjian_discard", ChoiceKind::SelectCards, opts, 1, 1, true))
            }
            HostStage::TargetChoice => {
                let opts = core
                    .alive_players()
                    .filter(|p| *p != owner && !core.player(*p).hand.is_empty())
                    .map(ChoiceOption::Player)
                    .collect();
                Some(ask(owner, "bingjian_target", ChoiceKind::SelectPlayers, opts, 1, 1, true))
            }
            _ => None,
        },
        HostBehavior::SwdShangxi => match stage {
            HostStage::SelectCard => {
                Some(ask(owner, "shangxi_discard", ChoiceKind::SelectCards, he_opts(owner), 1, 1, true))
            }
            HostStage::TargetChoice => {
                let opts = core
                    .alive_players()
                    .filter(|p| {
                        *p != owner
                            && core.computed_distance(catalog, owner, *p) <= 1
                            && core.player(owner).hp <= core.player(*p).hp
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                Some(ask(owner, "shangxi_target", ChoiceKind::SelectPlayers, opts, 1, 1, true))
            }
            _ => None,
        },
        HostBehavior::SwdLmazui => match stage {
            HostStage::SelectCard => {
                let opts = core
                    .player(owner)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| {
                        matches!(
                            catalog.effective_card_suit(core, owner, *c),
                            crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                        )
                    })
                    .map(ChoiceOption::Card)
                    .collect();
                Some(ask(owner, "lmazui_pick", ChoiceKind::SelectCards, opts, 1, 1, true))
            }
            HostStage::TargetChoice => {
                let lm2_def = catalog.skill_by_name(crate::vocabulary::host::SWD_LMAZUI2_KEY);
                let opts = core
                    .alive_players()
                    .filter(|p| !lm2_def.is_some_and(|d| core.has_skill(*p, d)))
                    .map(ChoiceOption::Player)
                    .collect();
                Some(ask(owner, "lmazui_target", ChoiceKind::SelectPlayers, opts, 1, 1, true))
            }
            _ => None,
        },
        HostBehavior::SwdMingfu => match stage {
            HostStage::SelectCard => {
                let opts = core
                    .player(owner)
                    .hand
                    .iter()
                    .chain(core.player(owner).equips.iter())
                    .copied()
                    .filter(|c| {
                        matches!(
                            catalog.effective_card_suit(core, owner, *c),
                            crate::catalog::Suit::Club
                        )
                    })
                    .map(ChoiceOption::Card)
                    .collect();
                Some(ask(owner, "mingfu_pick", ChoiceKind::SelectCards, opts, 1, 1, true))
            }
            HostStage::TargetChoice => {
                let opts = core
                    .alive_players()
                    .filter(|p| {
                        *p != owner
                            && !catalog.blocks_delayed_trick(core, *p)
                            && !core.player(*p).judge.iter().any(|jc| {
                                core.declared_kinds.get(jc).copied().unwrap_or_else(|| catalog.card_kind(*jc))
                                    == CardKind::Guiyoujie
                            })
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                Some(ask(owner, "mingfu_target", ChoiceKind::SelectPlayers, opts, 1, 1, true))
            }
            _ => None,
        },
        HostBehavior::SwdHuangyu if stage == HostStage::SelectCard => {
            let reds = core
                .player(owner)
                .hand
                .iter()
                .chain(core.player(owner).equips.iter())
                .copied()
                .filter(|c| {
                    matches!(
                        catalog.effective_card_suit(core, owner, *c),
                        crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                    )
                })
                .map(ChoiceOption::Card)
                .collect();
            Some(ask(owner, "huangyu_discard", ChoiceKind::SelectCards, reds, 2, 2, true))
        }
        HostBehavior::SwdYeying => match stage {
            HostStage::SelectCard => Some(ask(
                owner,
                "yeying_pick",
                ChoiceKind::SelectCards,
                super::u05_7c_viewas::he_black_options(core, catalog, owner),
                1,
                1,
                true,
            )),
            HostStage::TargetChoice => Some(ask(
                owner,
                "yeying_target",
                ChoiceKind::SelectPlayers,
                super::u05_7c_viewas::yeying_target_options(core, owner),
                1,
                1,
                true,
            )),
            _ => None,
        },
        HostBehavior::SwdGuiying => match stage {
            HostStage::SelectCard => Some(ask(
                owner,
                "guiying_pick",
                ChoiceKind::SelectCards,
                super::u05_7c_viewas::he_black_options(core, catalog, owner),
                1,
                1,
                true,
            )),
            HostStage::TargetChoice => Some(ask(
                owner,
                "guiying_target",
                ChoiceKind::SelectPlayers,
                super::u05_7c_viewas::guiying_target_options(core, owner),
                1,
                1,
                true,
            )),
            _ => None,
        },
        HostBehavior::SwdFengze if stage == HostStage::SelectCard => Some(ask(
            owner,
            "fengze_pick",
            ChoiceKind::SelectCards,
            super::u05_7c_viewas::he_black_options(core, catalog, owner),
            1,
            1,
            true,
        )),
        HostBehavior::SwdHuanxia => match stage {
            HostStage::SelectCard => Some(ask(
                owner,
                "huanxia_pick",
                ChoiceKind::SelectCards,
                super::u05_7c_viewas::he_red_options(core, catalog, owner),
                1,
                1,
                true,
            )),
            HostStage::TargetChoice => Some(ask(
                owner,
                "huanxia_target",
                ChoiceKind::SelectPlayers,
                super::u05_7c_viewas::sha_target_options(core, catalog, owner),
                1,
                1,
                true,
            )),
            _ => None,
        },
        HostBehavior::SwdYaotong => match stage {
            HostStage::SelectCard => Some(ask(
                owner,
                "yaotong_pick",
                ChoiceKind::SelectCards,
                super::u05_7c_viewas::hand_all_options(core, catalog, owner),
                1,
                1,
                true,
            )),
            HostStage::TargetChoice => Some(ask(
                owner,
                "yaotong_target",
                ChoiceKind::SelectPlayers,
                super::u05_7c_viewas::sha_target_options(core, catalog, owner),
                1,
                1,
                true,
            )),
            _ => None,
        },
        HostBehavior::Jilve => match stage {
            HostStage::SelectCard => Some(ask(
                owner,
                "jilve_card",
                ChoiceKind::SelectCards,
                super::skills_virtual_use::jilve_card_options(core, catalog, owner),
                1,
                1,
                true,
            )),
            HostStage::TargetChoice => {
                let kind = catalog.card_kind(_card?);
                let (min, max, opts) = super::skills_virtual_use::jilve_legal_targets(core, catalog, kind, owner);
                Some(ask(
                    owner,
                    "jilve_targets",
                    ChoiceKind::SelectPlayers,
                    opts.into_iter().map(ChoiceOption::Player).collect(),
                    min as usize,
                    max as usize,
                    false,
                ))
            }
            HostStage::JiefenTarget => Some(ask(
                owner,
                "jilve_jiedao_victim",
                ChoiceKind::SelectPlayers,
                core.legal_jiedao_victims(catalog, _target?)
                    .into_iter()
                    .map(ChoiceOption::Player)
                    .collect(),
                1,
                1,
                false,
            )),
            _ => None,
        },
        _ => None,
    }
}
