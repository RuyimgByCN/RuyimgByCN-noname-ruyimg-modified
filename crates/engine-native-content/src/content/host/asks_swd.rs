//! 宿主机器询问派生：轩辕剑（swd）与 U05 机关/神秘果/白兽之琥补充询问。

use super::*;
use crate::catalog::Catalog;
use crate::snap::*;
use crate::state::GameStateCore;

#[allow(clippy::too_many_arguments)]
pub(super) fn active_ask_swd(
    behavior: HostBehavior,
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<CardId>,
    _cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    match behavior {
        HostBehavior::SwdShenmiguoWindow if stage == HostStage::TargetChoice => {
            let spec = crate::choice::ChoiceSpec::confirm("trigger_confirm");
            Some(ask(
                owner,
                spec.prompt,
                spec.kind,
                spec.options,
                spec.min,
                spec.max,
                spec.allow_cancel,
            ))
        }
        HostBehavior::SwdBaishouzhihuHeld => {
            if stage != HostStage::TargetChoice {
                return None;
            }
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| {
                    p != owner
                        && (!core.player(p).hand.is_empty() || !core.player(p).equips.is_empty())
                })
                .map(ChoiceOption::Player)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some(ask(
                owner,
                "baishouzhihu_target",
                ChoiceKind::SelectPlayers,
                options,
                1,
                1,
                true,
            ))
        }
        HostBehavior::SwdJiguantongCard if stage == HostStage::SelectCard => {
            let t = *targets.first()?;
            Some(ask(
                t,
                crate::content::host::SWD_JIGUANTONG_DISCARD,
                ChoiceKind::SelectCards,
                hand_card_options(&core.player(t).hand),
                1,
                1,
                true,
            ))
        }
        HostBehavior::SwdJiguanfengCard if stage == HostStage::SelectCard => {
            let t = target?;
            let options =
                crate::kernel::response_kind_cards_core(core, catalog, t, CardKind::Shan)
                    .into_iter()
                    .map(ChoiceOption::Card)
                    .collect();
            Some(ask(
                t,
                crate::content::host::SWD_JIGUANFENG_SHAN,
                ChoiceKind::SelectCards,
                options,
                0,
                1,
                true,
            ))
        }
        HostBehavior::SwdJiguanyuanCard if stage == HostStage::SelectCard => {
            let options: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .chain(core.player(owner).equips.iter())
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            Some(ask(
                owner,
                crate::content::host::SWD_JIGUANYUAN_CHOOSE,
                ChoiceKind::SelectCards,
                options,
                1,
                1,
                false,
            ))
        }
        HostBehavior::SwdYufuluEquipSkill => match stage {
            HostStage::SelectCard => {
                let mut options: Vec<ChoiceOption> = core
                    .player(owner)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| {
                        catalog.card_slot_of(core, *c) == Some(crate::catalog::EquipSlot::Weapon)
                    })
                    .map(ChoiceOption::Card)
                    .collect();
                options.extend(
                    core.player(owner)
                        .equips
                        .iter()
                        .copied()
                        .filter(|c| {
                            catalog.card_slot_of(core, *c)
                                == Some(crate::catalog::EquipSlot::Weapon)
                        })
                        .map(ChoiceOption::Card),
                );
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    crate::content::host::SWD_YUFULU_WEAPON,
                    ChoiceKind::SelectCards,
                    options,
                    1,
                    1,
                    false,
                ))
            }
            HostStage::TargetChoice => Some(ask(
                owner,
                crate::content::host::SWD_YUFULU_TARGET,
                ChoiceKind::SelectPlayers,
                other_player_options(core, owner),
                1,
                1,
                false,
            )),
            _ => None,
        },
        HostBehavior::SwdYijue => match stage {
            HostStage::TargetChoice => {
                let opts: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|p| *p != owner && !core.player(*p).hand.is_empty())
                    .map(ChoiceOption::Player)
                    .collect();
                (!opts.is_empty())
                    .then(|| ask(owner, "yijue_target", ChoiceKind::SelectPlayers, opts, 1, 1, true))
            }
            HostStage::CompareTargetPick => {
                let t = target?;
                let opts = hand_card_options(&core.player(t).hand);
                (!opts.is_empty())
                    .then(|| ask(t, "yijue_target_pick", ChoiceKind::SelectCards, opts, 1, 1, false))
            }
            HostStage::ComparePlayerPick => {
                let opts = hand_card_options(&core.player(owner).hand);
                (!opts.is_empty()).then(|| {
                    ask(
                        owner,
                        "yijue_player_pick",
                        ChoiceKind::SelectCards,
                        opts,
                        1,
                        1,
                        false,
                    )
                })
            }
            HostStage::SelectNature => target.map(|t| {
                ask(
                    owner,
                    "swd_yijue_recover_confirm",
                    ChoiceKind::SelectPlayers,
                    vec![ChoiceOption::Player(t)],
                    1,
                    1,
                    true,
                )
            }),
            _ => None,
        },
        HostBehavior::SwdJqimou => super::step::ask_swd_jqimou(stage, owner, card, core, catalog)
            .map(|spec| {
                ask(
                    owner,
                    spec.prompt,
                    spec.kind,
                    spec.options,
                    spec.min,
                    spec.max,
                    spec.allow_cancel,
                )
            }),
        _ => None,
    }
}
