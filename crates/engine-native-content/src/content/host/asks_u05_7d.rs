//! U05-7d 杀海拾遗 swd 人物技能与修复挂起询问派生（`asks_u05_7d.rs`）。

use super::step::swd_u05_7d::same_suit_hand_candidates;
use super::step::swd_u05_7d_fixes_cards::xuanzhou_delay_tags;
use super::*;
use crate::catalog::Catalog;
use crate::snap::*;
use crate::state::GameStateCore;

fn he_card_options(core: &GameStateCore, p: PlayerId) -> Vec<ChoiceOption> {
    let mut out = hand_card_options(&core.player(p).hand);
    out.extend(core.player(p).equips.iter().copied().map(ChoiceOption::Card));
    out
}

#[allow(clippy::too_many_arguments)]
pub(super) fn active_ask_u05_7d(
    behavior: HostBehavior,
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    _targets: &[PlayerId],
    _card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    match behavior {
        HostBehavior::SwdDuijue if stage == HostStage::TargetChoice => {
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|p| *p != owner && core.player(*p).hp > 1 && !catalog.has_undist(core, *p))
                .map(ChoiceOption::Player)
                .collect();
            Some(ask(owner, "duijue_target", ChoiceKind::SelectPlayers, options, 1, 1, true))
        }
        HostBehavior::SwdTanlin => match stage {
            HostStage::TargetChoice => {
                let options: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|p| {
                        *p != owner
                            && !core.player(*p).hand.is_empty()
                            && !catalog.has_undist(core, *p)
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                Some(ask(owner, "tanlin_target", ChoiceKind::SelectPlayers, options, 1, 1, true))
            }
            HostStage::SelectCard => {
                let options = hand_card_options(&core.player(owner).hand);
                Some(ask(owner, "tanlin_pindian", ChoiceKind::SelectCards, options, 1, 1, true))
            }
            HostStage::CardsAsk => {
                let t = target?;
                let options = hand_card_options(&core.player(t).hand);
                Some(ask(t, "tanlin_target_pindian", ChoiceKind::SelectCards, options, 1, 1, false))
            }
            _ => None,
        },
        HostBehavior::SwdTanlinDefence if stage == HostStage::SelectCard => {
            let src = target?;
            let max = core.player(owner).hand.len().min(core.player(src).hand.len());
            if max == 0 {
                return None;
            }
            Some(ask(
                owner,
                "tanlin_defence_discard",
                ChoiceKind::SelectCards,
                hand_card_options(&core.player(owner).hand),
                1,
                max,
                true,
            ))
        }
        HostBehavior::SwdOldLiaoyuan => match stage {
            HostStage::SelectCard => {
                let cands: Vec<ChoiceOption> =
                    same_suit_hand_candidates(catalog, core, owner).into_iter().map(ChoiceOption::Card).collect();
                Some(ask(owner, "oldliaoyuan_cards", ChoiceKind::SelectCards, cands, 2, 2, true))
            }
            HostStage::TargetChoice => {
                let options: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|p| *p != owner && !catalog.has_undist(core, *p))
                    .map(ChoiceOption::Player)
                    .collect();
                Some(ask(owner, "oldliaoyuan_target", ChoiceKind::SelectPlayers, options, 1, 1, true))
            }
            HostStage::CardsAsk => {
                let t = target?;
                let he = he_card_options(core, t);
                Some(ask(owner, "oldliaoyuan_target_discard", ChoiceKind::SelectCards, he, 1, 1, false))
            }
            _ => None,
        },
        HostBehavior::SwdOldLiaoyuan2 => match stage {
            HostStage::SelectCard => {
                let cands: Vec<ChoiceOption> =
                    same_suit_hand_candidates(catalog, core, owner).into_iter().map(ChoiceOption::Card).collect();
                Some(ask(owner, "oldliaoyuan2_cards", ChoiceKind::SelectCards, cands, 2, 2, true))
            }
            HostStage::TargetChoice => {
                let options: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|p| *p != owner && !catalog.has_undist(core, *p))
                    .map(ChoiceOption::Player)
                    .collect();
                Some(ask(owner, "oldliaoyuan2_target", ChoiceKind::SelectPlayers, options, 1, 1, true))
            }
            _ => None,
        },
        HostBehavior::SwdXuanzhou => match stage {
            HostStage::SelectCard => {
                let options: Vec<ChoiceOption> = he_card_options(core, owner)
                    .into_iter()
                    .filter(|o| match o {
                        ChoiceOption::Card(c) => catalog.card_kind(*c).is_ordinary_trick_kind(),
                        _ => false,
                    })
                    .collect();
                Some(ask(owner, "xuanzhou_trick", ChoiceKind::SelectCards, options, 1, 1, true))
            }
            HostStage::TargetChoice => {
                let options: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|p| *p != owner && !xuanzhou_delay_tags(core, catalog, *p).is_empty())
                    .map(ChoiceOption::Player)
                    .collect();
                Some(ask(owner, "xuanzhou_target", ChoiceKind::SelectPlayers, options, 1, 1, true))
            }
            HostStage::SelectNature => {
                let t = target?;
                let options: Vec<ChoiceOption> = xuanzhou_delay_tags(core, catalog, t)
                    .into_iter()
                    .map(ChoiceOption::Tag)
                    .collect();
                Some(ask(owner, "xuanzhou_delay_kind", ChoiceKind::PickTag, options, 1, 1, true))
            }
            _ => None,
        },
        HostBehavior::SwdLqingcheng if stage == HostStage::ComparePlayerPick => Some(ask(
            owner,
            "lqingcheng_continue",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm],
            0,
            1,
            true,
        )),
        HostBehavior::SwdPozhen => match stage {
            HostStage::SelectCard => {
                let n = cards.len();
                Some(ask(
                    owner,
                    "pozhen_discard",
                    ChoiceKind::SelectCards,
                    hand_card_options(&core.player(owner).hand),
                    n,
                    n,
                    true,
                ))
            }
            HostStage::TargetChoice => Some(ask(
                owner,
                "pozhen_discard_source",
                ChoiceKind::Confirm,
                vec![ChoiceOption::Confirm],
                0,
                1,
                true,
            )),
            _ => None,
        },
        HostBehavior::SwdTianshu => match stage {
            HostStage::SelectCard => {
                let options: Vec<ChoiceOption> = core
                    .player(owner)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| catalog.card_kind(*c).is_trick_kind())
                    .map(ChoiceOption::Card)
                    .collect();
                Some(ask(owner, "swdtianshu_card", ChoiceKind::SelectCards, options, 1, 1, false))
            }
            HostStage::TargetChoice => {
                let options: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|p| {
                        *p != owner && !super::support::swdtianshu_skills_for_state(core, catalog, *p, owner).is_empty()
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                Some(ask(owner, "swdtianshu_target", ChoiceKind::SelectPlayers, options, 1, 1, false))
            }
            HostStage::ShuchuiTargetLoop => {
                let t = target?;
                let names = super::support::swdtianshu_skills_for_state(core, catalog, t, owner);
                let options: Vec<ChoiceOption> = (0..names.len()).map(|i| ChoiceOption::Tag(i as u8)).collect();
                Some(ask(owner, "swdtianshu_skill", ChoiceKind::PickTag, options, 1, 1, false))
            }
            _ => None,
        },
        HostBehavior::SwdToudan => match stage {
            HostStage::SelectCard => {
                let options: Vec<ChoiceOption> = he_card_options(core, owner)
                    .into_iter()
                    .filter(|o| match o {
                        ChoiceOption::Card(c) => {
                            catalog.effective_card_suit(core, owner, *c) == crate::catalog::Suit::Spade
                        }
                        _ => false,
                    })
                    .collect();
                Some(ask(owner, "toudan_pick", ChoiceKind::SelectCards, options, 1, 1, true))
            }
            HostStage::TargetChoice => {
                let options: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|p| *p != owner)
                    .map(ChoiceOption::Player)
                    .collect();
                Some(ask(owner, "toudan_target", ChoiceKind::SelectPlayers, options, 1, 1, true))
            }
            HostStage::CardsAsk => {
                let cur = target?;
                let options = he_card_options(core, cur);
                Some(ask(cur, "toudan_neighbor_discard", ChoiceKind::SelectCards, options, 1, 1, false))
            }
            _ => None,
        },
        HostBehavior::SwdDuanyi => match stage {
            HostStage::SelectCard => {
                let options: Vec<ChoiceOption> = core
                    .player(owner)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| catalog.card_kind(*c).is_sha_like())
                    .map(ChoiceOption::Card)
                    .collect();
                Some(ask(owner, "duanyi_discard", ChoiceKind::SelectCards, options, 2, 2, true))
            }
            HostStage::TargetChoice => {
                let options: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|p| *p != owner)
                    .map(ChoiceOption::Player)
                    .collect();
                Some(ask(owner, "duanyi_target", ChoiceKind::SelectPlayers, options, 1, 1, true))
            }
            _ => None,
        },
        HostBehavior::SwdGuanhu if stage == HostStage::SelectCard => {
            let t = target?;
            let max = if !core.player(t).hand.is_empty() && !core.player(t).equips.is_empty() {
                2
            } else {
                1
            };
            Some(ask(owner, "guanhu_discard", ChoiceKind::SelectCards, he_card_options(core, t), 0, max, true))
        }
        HostBehavior::SwdShehun => match stage {
            HostStage::SelectCard => Some(ask(
                owner,
                "shehun_discard",
                ChoiceKind::SelectCards,
                hand_card_options(&core.player(owner).hand),
                1,
                4,
                true,
            )),
            HostStage::TargetChoice => {
                let options: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|p| *p != owner && !he_card_options(core, *p).is_empty())
                    .map(ChoiceOption::Player)
                    .collect();
                Some(ask(owner, "shehun_target", ChoiceKind::SelectPlayers, options, 1, 1, true))
            }
            HostStage::CardsAsk => {
                let t = target?;
                let he = he_card_options(core, t);
                let take = cards.len().min(he.len());
                Some(ask(t, "shehun_target_discard", ChoiceKind::SelectCards, he, take, take, false))
            }
            _ => None,
        },
        HostBehavior::Moyan => match stage {
            HostStage::SelectCard => {
                let reds: Vec<ChoiceOption> = core
                    .player(owner)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| matches!(catalog.effective_card_suit(core, owner, *c), crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond))
                    .map(ChoiceOption::Card)
                    .collect();
                let others = core.alive_players().filter(|p| *p != owner).count();
                let hp_gap = (core.player(owner).max_hp as i32 - core.player(owner).hp).max(0) as usize;
                let max = reds.len().min(others).min(hp_gap);
                Some(ask(owner, "moyan_cards", ChoiceKind::SelectCards, reds, 1, max, true))
            }
            HostStage::TargetChoice => {
                let need = cards.len();
                let options: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|p| *p != owner)
                    .map(ChoiceOption::Player)
                    .collect();
                Some(ask(owner, "moyan_targets", ChoiceKind::SelectPlayers, options, need, need, true))
            }
            _ => None,
        },
        HostBehavior::SwdHuajian => match stage {
            HostStage::SelectCard => Some(ask(
                owner,
                "huajian_discard",
                ChoiceKind::SelectCards,
                he_card_options(core, owner),
                1,
                1,
                true,
            )),
            HostStage::TargetChoice => {
                let options: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|p| {
                        engine_native_kernel::kernel::can_target_sha_no_range(
                            core,
                            catalog,
                            owner,
                            *p,
                            CardKind::Sha,
                        )
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                Some(ask(owner, "huajian_target", ChoiceKind::SelectPlayers, options, 1, 1, true))
            }
            _ => None,
        },
        HostBehavior::Lxianglong if stage == HostStage::TargetChoice => {
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|p| *p != owner && core.in_attack_range(catalog, owner, *p))
                .map(ChoiceOption::Player)
                .collect();
            Some(ask(owner, "lxianglong_target", ChoiceKind::SelectPlayers, options, 1, 1, true))
        }
        HostBehavior::Kunlunjing if stage == HostStage::SelectNature => Some(ask(
            owner,
            "trigger_confirm",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm],
            1,
            1,
            true,
        )),
        HostBehavior::SwdHuanhun if stage == HostStage::SelectCard => Some(ask(
            owner,
            "huanhun_discard",
            ChoiceKind::SelectCards,
            he_card_options(core, owner),
            1,
            1,
            true,
        )),
        HostBehavior::SwdDunxing if stage == HostStage::SelectCard => Some(ask(
            owner,
            "dunxing_discard",
            ChoiceKind::SelectCards,
            he_card_options(core, owner),
            1,
            1,
            true,
        )),
        HostBehavior::SwdYinguo if stage == HostStage::SelectCard => Some(ask(
            owner,
            "yinguo_discard",
            ChoiceKind::SelectCards,
            he_card_options(core, owner),
            1,
            1,
            true,
        )),
        HostBehavior::SwdSusheng if stage == HostStage::SelectCard => Some(ask(
            owner,
            "susheng_discard",
            ChoiceKind::SelectCards,
            hand_card_options(&core.player(owner).hand),
            0,
            1,
            true,
        )),
        HostBehavior::SwdGuiyan => match stage {
            HostStage::TargetChoice => {
                let options: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|p| *p != owner && !core.player(*p).hand.is_empty())
                    .map(ChoiceOption::Player)
                    .collect();
                Some(ask(owner, "guiyan_target", ChoiceKind::SelectPlayers, options, 1, 1, true))
            }
            HostStage::SelectCard => {
                let t = target?;
                let options: Vec<ChoiceOption> = core
                    .player(t)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| catalog.effective_card_suit(core, t, *c) == crate::catalog::Suit::Club)
                    .map(ChoiceOption::Card)
                    .collect();
                Some(ask(owner, "guiyan_take", ChoiceKind::SelectCards, options, 1, 1, true))
            }
            _ => None,
        },
        HostBehavior::SwdRexue if stage == HostStage::SelectCard => {
            let t = target?;
            let options =
                engine_native_kernel::kernel::usable_sha_options(core, catalog, owner, t);
            Some(ask(owner, "rexue_sha", ChoiceKind::SelectCards, options, 1, 1, true))
        }
        _ => None,
    }
}
