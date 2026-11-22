//! Choice builders for Batch B skills (`2001..=2034`) in `u01_fixes_2`.

use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn build_ask_b_2(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    match ask_id {
        2001 => {
            let victim = target?;
            let mut opts = Vec::new();
            if core.player(victim).hp < core.player(victim).max_hp as i32 {
                opts.push(ChoiceOption::Tag(0));
            }
            opts.push(ChoiceOption::Tag(1));
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "dddzhengjun_hp_choice",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        2002 => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::PickTag,
                prompt: "dddzhengjun_hs_choice",
                options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                min: 1,
                max: 1,
                allow_cancel: true,
            },
        )),
        2003 => {
            let victim = target?;
            let opts = he_options(core, victim);
            (!opts.is_empty()).then_some((
                victim,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddzhengjun_hs_discard",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        2004 => Some((owner, ChoiceSpec::confirm("dddzhengjun_es_confirm"))),
        2005 => {
            let src = target?;
            let opts: Vec<ChoiceOption> = dddzhengjun_movable_field_cards(core, catalog, src)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            (!opts.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddzhengjun_es_card",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        2006 => {
            let c = card?;
            let opts: Vec<ChoiceOption> = zhishu_valid_dests(core, catalog, c)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            (!opts.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddzhengjun_es_dest",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        2007 => {
            let opts: Vec<ChoiceOption> =
                dddzhengjun_match_targets(core, owner, target.unwrap_or(owner), targets, 1)
                    .into_iter()
                    .map(ChoiceOption::Player)
                    .collect();
            (!opts.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddzhengjun_hp_target",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        2008 => {
            let opts: Vec<ChoiceOption> =
                dddzhengjun_match_targets(core, owner, target.unwrap_or(owner), targets, 2)
                    .into_iter()
                    .map(ChoiceOption::Player)
                    .collect();
            (!opts.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddzhengjun_hs_target",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        2009 => {
            let nature = match card.unwrap_or(0) {
                1 => DamageNature::Fire,
                2 => DamageNature::Thunder,
                _ => DamageNature::Normal,
            };
            let opts: Vec<ChoiceOption> =
                dddxianxi_eligible_targets(core, catalog, owner, targets, nature)
                    .into_iter()
                    .map(ChoiceOption::Player)
                    .collect();
            (!opts.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddxianxi_extra_target",
                    max: opts.len(),
                    options: opts,
                    min: 1,
                    allow_cancel: true,
                },
            ))
        }
        2010 => {
            let num = targets.len();
            let opts = he_options(core, owner);
            (num > 0 && opts.len() >= num).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddxianxi_discard",
                    options: opts,
                    min: num,
                    max: num,
                    allow_cancel: true,
                },
            ))
        }
        2011 => Some((owner, ChoiceSpec::confirm("dddlingyong_confirm"))),
        2012 => {
            let cid = card?;
            let tgts = dddlingyong_targets(core, catalog, owner, cid)?;
            if dddlingyong_needs_target_prompt(catalog, cid) {
                Some((
                    owner,
                    ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "dddlingyong_use_target",
                        options: tgts.into_iter().map(ChoiceOption::Player).collect(),
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                ))
            } else {
                Some((owner, ChoiceSpec::confirm("dddlingyong_use_target")))
            }
        }
        2013 => Some((owner, ChoiceSpec::confirm("dddlingyong_repeat"))),
        2014 => {
            let opts: Vec<ChoiceOption> = dddxuxiao_valid_cards(core, catalog, cards)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            (!opts.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddxuxiao_card",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        2015 => {
            let opts: Vec<ChoiceOption> = dddxuxiao_valid_targets(core, catalog)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            (!opts.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddxuxiao_target",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        2016 => Some((owner, ChoiceSpec::confirm("dddxuyu_confirm"))),
        2017 => {
            let opts = he_options(core, owner);
            (!opts.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddxuyu_discard",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        2018 => {
            let opts = he_options(core, owner);
            (!opts.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddshijian_give",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        2019 => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::PickTag,
                prompt: "dddshijian_mode",
                options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                min: 1,
                max: 1,
                allow_cancel: false,
            },
        )),
        2020 => Some((owner, ChoiceSpec::confirm("dddxuyu_old_confirm"))),
        2021 => {
            let opts = he_options(core, owner);
            (!opts.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddxuyu_old_discard",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        2022 => Some((owner, ChoiceSpec::confirm("dddshijian_old_confirm"))),
        2023 => {
            let opts: Vec<ChoiceOption> = core.alive_players().map(ChoiceOption::Player).collect();
            (!opts.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddtaisi_target",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        2024 => {
            let opts: Vec<ChoiceOption> = dddtaisi_discard_cards(core)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            (!opts.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddtaisi_card",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        2025 => {
            let opts: Vec<ChoiceOption> = dddquche_red_cards(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            (!opts.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddquche_red_card",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        2026 => {
            let opts: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| p != owner)
                .map(ChoiceOption::Player)
                .collect();
            (!opts.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddquche_give_target",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        2027 => {
            let opts: Vec<ChoiceOption> = dddquche_black_cards(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            (!opts.is_empty()
                && !virtual_legal_targets(core, catalog, owner, CardKind::Sha).is_empty())
            .then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddquche_black_sha",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        2028 => {
            let opts: Vec<ChoiceOption> = virtual_legal_targets(core, catalog, owner, CardKind::Sha)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            (!opts.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddquche_sha_target",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        2029 => Some((owner, ChoiceSpec::confirm("ddddongcha_confirm"))),
        2030 => {
            let opts = hand_options(core, owner);
            (!opts.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "ddddongcha_owner_put",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        2031 => {
            let tgt = target?;
            let opts = hand_options(core, tgt);
            (!opts.is_empty()).then_some((
                tgt,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "ddddongcha_target_put",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        2032 => {
            let opts: Vec<ChoiceOption> = ddd_jian_cards(core, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            (!opts.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "ddddongcha_end_owner_pick",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        2033 => {
            let tgt = target?;
            let opts: Vec<ChoiceOption> = ddd_jian_cards(core, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            (!opts.is_empty()).then_some((
                tgt,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "ddddongcha_end_target_pick",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        2034 => {
            let raw = ddd_jian_cards(core, owner);
            let is_red = |c| effective_is_red(core, catalog, owner, c);
            let reds: Vec<CardId> = raw.iter().copied().filter(|&c| is_red(c)).collect();
            let blacks: Vec<CardId> = raw.iter().copied().filter(|&c| !is_red(c)).collect();
            let diff_color = flag(core, owner, "dddzhijie_need_diff_color") != 0;
            let mut ordered = Vec::with_capacity(raw.len());
            if diff_color {
                if let (Some(&r), Some(&b)) = (reds.first(), blacks.first()) {
                    ordered.push(r);
                    ordered.push(b);
                }
            } else if reds.len() >= 2 {
                ordered.push(reds[0]);
                ordered.push(reds[1]);
            } else if blacks.len() >= 2 {
                ordered.push(blacks[0]);
                ordered.push(blacks[1]);
            }
            for c in raw {
                if !ordered.contains(&c) {
                    ordered.push(c);
                }
            }
            let opts: Vec<ChoiceOption> = ordered.into_iter().map(ChoiceOption::Card).collect();
            (opts.len() >= 2).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddzhijie_cards",
                    options: opts,
                    min: 2,
                    max: 2,
                    allow_cancel: true,
                },
            ))
        }
        _ => None,
    }
}
