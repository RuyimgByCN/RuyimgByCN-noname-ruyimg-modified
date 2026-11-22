//! Choice builders for Batch C skills (`2035..=2050`) in `u01_fixes_2`.

use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn build_ask_c_2(
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
        2036 => {
            let tgt = target?;
            let need = core.player(tgt).hp.max(1) as usize;
            let unshown: Vec<ChoiceOption> = dddfusi_unshown_hand(core, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            (unshown.len() >= need).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddqiahua_show",
                    options: unshown,
                    min: need,
                    max: need,
                    allow_cancel: true,
                },
            ))
        }
        2037 => Some((owner, ChoiceSpec::confirm("dddxunxun_confirm"))),
        2038 => {
            let opts: Vec<ChoiceOption> = cards.iter().copied().map(ChoiceOption::Card).collect();
            let pick = opts.len().min(2);
            (pick > 0).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddxunxun_top",
                    options: opts,
                    min: pick,
                    max: pick,
                    allow_cancel: false,
                },
            ))
        }
        2039 => {
            let opts: Vec<ChoiceOption> = cards.iter().copied().map(ChoiceOption::Card).collect();
            (!opts.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddxunxun_bottom",
                    max: opts.len(),
                    min: opts.len(),
                    options: opts,
                    allow_cancel: false,
                },
            ))
        }
        2040 => {
            let raw_need = flag(core, owner, "u01_fusi_need");
            let opts: Vec<CardId> = if raw_need == 0 {
                dddfusi_usable_play_cards(core, catalog, owner)
            } else {
                let need = decode_respond_need(raw_need);
                dddfusi_respond_cards(core, catalog, owner, need)
            };
            (!opts.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddfusi_pick",
                    options: opts.into_iter().map(ChoiceOption::Card).collect(),
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        2041 => {
            let cid = card?;
            let kind = catalog.card_kind(cid);
            let opts: Vec<ChoiceOption> = dddfusi_play_targets(core, catalog, owner, cid)?
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            (!opts.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddfusi_target",
                    options: opts,
                    min: 1,
                    max: if kind == CardKind::Chain { 2 } else { 1 },
                    allow_cancel: true,
                },
            ))
        }
        2042 => {
            let src = target?;
            Some((
                src,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "dddfusi_agree",
                    options: vec![
                        ChoiceOption::Tag(0),
                        ChoiceOption::Tag(1),
                        ChoiceOption::Tag(2),
                        ChoiceOption::Tag(3),
                    ],
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        2043 => Some((owner, ChoiceSpec::confirm("dddtuoji_confirm"))),
        2044 => {
            let opts = he_options(core, owner);
            (!opts.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddchashi_discard",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        2045 => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::PickTag,
                prompt: "dddmiaoxing_out_group",
                options: vec![
                    ChoiceOption::Tag(0),
                    ChoiceOption::Tag(1),
                    ChoiceOption::Tag(2),
                ],
                min: 1,
                max: 1,
                allow_cancel: false,
            },
        )),
        2046 => {
            let tags = dddmiaoxing_mismatch_tags(core, owner);
            (!tags.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "dddmiaoxing_pile",
                    min: 1,
                    max: tags.len(),
                    options: tags.into_iter().map(ChoiceOption::Tag).collect(),
                    allow_cancel: false,
                },
            ))
        }
        2047 => {
            let key = if card.unwrap_or(0) == 1 {
                "dddmiaoxing_2"
            } else {
                "dddmiaoxing_1"
            };
            let pile = core
                .vm_aux
                .piles
                .get(&(owner, key.to_string()))
                .cloned()
                .unwrap_or_default();
            let hand_len = core.player(owner).hand.len();
            let excess = pile.len().saturating_sub(hand_len);
            (excess > 0).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddmiaoxing_discard",
                    options: pile.into_iter().map(ChoiceOption::Card).collect(),
                    min: excess,
                    max: excess,
                    allow_cancel: false,
                },
            ))
        }
        2048 => {
            let _ = targets;
            let opts: Vec<ChoiceOption> = dddfenji_usable_tricks(core, catalog, owner)
                .into_iter()
                .map(|(tag, _)| ChoiceOption::Tag(tag))
                .collect();
            (!opts.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "dddfenji_trick",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        2049 => {
            let tag = card? as u8;
            let kind = dddfenji_kind_for_tag(catalog, tag)?;
            let opts: Vec<ChoiceOption> = dddfenji_legal_targets(core, catalog, owner, kind)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            (!opts.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddfenji_target",
                    options: opts,
                    min: 1,
                    max: if kind == CardKind::Chain { 2 } else { 1 },
                    allow_cancel: false,
                },
            ))
        }
        2050 => {
            let need = card.unwrap_or(1) as usize;
            let hand_len = core.player(owner).hand.len();
            let n = need.min(hand_len);
            (n > 0).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddfenji_discard",
                    options: hand_options(core, owner),
                    min: n,
                    max: n,
                    allow_cancel: false,
                },
            ))
        }
        2051 => {
            let holder = target?;
            let opts: Vec<ChoiceOption> = jiedao_victims_for(core, catalog, holder)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            (!opts.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddlingyong_jiedao_victim",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        2052 => {
            let holder = target?;
            let opts: Vec<ChoiceOption> = jiedao_victims_for(core, catalog, holder)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            (!opts.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddfusi_jiedao_victim",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        2053 => {
            let holder = target?;
            let opts: Vec<ChoiceOption> = jiedao_victims_for(core, catalog, holder)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            (!opts.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddfenji_jiedao_victim",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        _ => None,
    }
}
