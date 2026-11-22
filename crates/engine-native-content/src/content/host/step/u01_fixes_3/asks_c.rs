use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn active_ask_u01_fix3_c(
    ask_id: u32,
    _stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    _card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    match ask_id {
        3001 => {
            let opts: Vec<ChoiceOption> = core.alive_players().map(ChoiceOption::Player).collect();
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddxiaheng_discard_target",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        3002 => {
            let t = target?;
            let opts: Vec<ChoiceOption> = he_cards(core, t)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            let need = opts.len().min(2);
            Some((
                t,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddxiaheng_discard_cards",
                    options: opts,
                    min: need,
                    max: need,
                    allow_cancel: false,
                },
            ))
        }
        3003 => {
            let opts: Vec<ChoiceOption> = core.alive_players().map(ChoiceOption::Player).collect();
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddxiaheng_damage_target",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        3004 => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            let max = opts.len().max(1);
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddfengzheng_cards",
                    options: opts,
                    min: 1,
                    max,
                    allow_cancel: true,
                },
            ))
        }
        3005 => {
            let opts: Vec<ChoiceOption> = core.alive_players().map(ChoiceOption::Player).collect();
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddfengzheng_give_target",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        3006 => {
            let opts: Vec<ChoiceOption> = cards
                .iter()
                .copied()
                .filter(|c| core.player(owner).off_court.contains(c))
                .map(ChoiceOption::Card)
                .collect();
            let max = opts.len().max(1);
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddfengzheng_give_cards",
                    options: opts,
                    min: 1,
                    max,
                    allow_cancel: false,
                },
            ))
        }
        3007 => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddyulv_put",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        3008 => {
            let cur = target?;
            let mut opts = vec![ChoiceOption::Tag(0)];
            if !he_cards(core, cur).is_empty() {
                opts.push(ChoiceOption::Tag(1));
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "dddyulv_mode",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        3009 => {
            let cur = target?;
            let opts: Vec<ChoiceOption> = he_cards(core, cur)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            Some((
                cur,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddyulv_discard",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        3010 => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddyulv_swap",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        3011 => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::PickTag,
                prompt: "dddyulv_old_declare",
                options: (1u8..=13).map(ChoiceOption::Tag).collect(),
                min: 1,
                max: 1,
                allow_cancel: false,
            },
        )),
        3012 => {
            let cur = target?;
            let mut opts = vec![ChoiceOption::Tag(0)];
            if !he_cards(core, cur).is_empty() {
                opts.push(ChoiceOption::Tag(1));
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "dddyulv_old_mode",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        3013 => {
            let cur = target?;
            let opts: Vec<ChoiceOption> = he_cards(core, cur)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            Some((
                cur,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddyulv_old_discard",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        3014 => {
            let opts: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| p != owner && !core.player(p).hand.is_empty())
                .map(ChoiceOption::Player)
                .collect();
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddfenye_target",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        3015 => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddfenye_owner_card",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        3016 => {
            let t = target?;
            let opts: Vec<ChoiceOption> = core
                .player(t)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            Some((
                t,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddfenye_target_card",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        3017 => {
            let bystander = *targets.first()?;
            let opts: Vec<ChoiceOption> = core
                .player(bystander)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            Some((
                bystander,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddfenye_join_card",
                    options: opts,
                    min: 0,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        3018 => {
            let bystander = *targets.first()?;
            let t = target?;
            Some((
                bystander,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddfenye_join_side",
                    options: vec![ChoiceOption::Player(owner), ChoiceOption::Player(t)],
                    min: 0,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        3019 => {
            let winner = *targets.first()?;
            let losers_mask = flag(core, owner, "dddfenye_losers_mask");
            let opts: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| (losers_mask & (1u32 << p)) != 0)
                .filter(|&p| !is_sha_target_blocked(core, catalog, winner, p, CardKind::Sha, DamageNature::Normal))
                .map(ChoiceOption::Player)
                .collect();
            Some((
                winner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddfenye_sha_target",
                    options: opts,
                    min: 0,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        3020 => {
            let mark = flag(core, owner, "dddshichao_mark") as usize;
            let opts: Vec<ChoiceOption> = dddshichao_nth_group_targets(core, mark)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddshichao_target",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        3021 => {
            let need = flag(core, owner, "dddshichao_discard_need") as usize;
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            let n = need.min(opts.len());
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddshichao_discard",
                    options: opts,
                    min: n,
                    max: n,
                    allow_cancel: false,
                },
            ))
        }
        3022 => {
            let attacker = target?;
            Some((attacker, u01_confirm_spec("dddshichao_avoid_confirm", true)))
        }
        3023 => Some((owner, u01_confirm_spec("dddfengzheng_confirm", true))),
        _ => None,
    }
}
