//! Command operations for `resolve_choose_category_then_target_give_matching_or_sha_unrespondable` and related helpers.

use super::*;

pub(crate) fn resolve_choose_category_then_target_give_matching_or_sha_unrespondable(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [stage, player, sha, target, skill, rest @ ..] = payload else {
        return Err(EffectError::EngineFault(
            "bad category-gift-or-direct-hit payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let sha = CardId::new(*sha);
    let target = PlayerId::new(*target);
    let skill = SkillId::new(*skill);
    let damage_mode = match *stage {
        0 => rest.first() == Some(&crate::effect::CATEGORY_GIFT_DAMAGE_PLUS_ONE_MODE),
        1 => rest.get(1) == Some(&crate::effect::CATEGORY_GIFT_DAMAGE_PLUS_ONE_MODE),
        _ => false,
    };
    if ![player, target].into_iter().all(|participant| {
        tx.state()
            .players
            .get(participant)
            .is_some_and(|state| state.alive)
    }) {
        return Err(EffectError::Illegal(
            "category-gift-or-direct-hit participants changed".into(),
        ));
    }
    if damage_mode {
        if tx.state().resolution.current_damage_id != sha.raw() {
            return Err(EffectError::Illegal(
                "category-gift damage context changed".into(),
            ));
        }
    } else {
        crate::command::play::validate_saved_sha_target(tx, player, sha, target)?;
    }
    let matching = |tx: &Transaction, category: u8| {
        [ZoneKind::Hand, ZoneKind::Equip]
            .into_iter()
            .filter_map(|kind| tx.state().zones.find(kind, target))
            .filter_map(|zone| tx.state().zones.get(zone))
            .flat_map(|zone| zone.cards.iter().copied())
            .filter(|card| {
                crate::effect::effective_card_category(tx.state(), target, *card) == category
            })
            .collect::<Vec<_>>()
    };
    let fallback = |tx: &mut Transaction| {
        if damage_mode {
            crate::effect::increase_bound_damage_by_one(tx, sha.raw()).map(|_| ())
        } else {
            run_effects(
                tx,
                vec![EffectRequest::MakeCurrentShaTargetUnrespondable {
                    source: player,
                    card: sha,
                    target,
                }],
            )
            .map(|_| ())
        }
    };

    match *stage {
        0 => {
            if cancelled || observer != player {
                return Err(EffectError::Illegal(
                    "category choice cannot cancel or change actor".into(),
                ));
            }
            let [category] = selections else {
                return Err(EffectError::Illegal("choose one card category".into()));
            };
            let category = u8::try_from(*category)
                .ok()
                .filter(|category| (1..=3).contains(category))
                .ok_or_else(|| EffectError::Illegal("unknown card category".into()))?;
            let options = matching(tx, category);
            if options.is_empty() {
                return fallback(tx);
            }
            let mut next = vec![
                1,
                player.raw(),
                sha.raw(),
                target.raw(),
                skill.raw(),
                u32::from(category),
            ];
            if damage_mode {
                next.push(crate::effect::CATEGORY_GIFT_DAMAGE_PLUS_ONE_MODE);
            }
            open_choice(
                tx,
                target,
                1,
                1,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable,
                next,
            )?;
            Err(EffectError::ChoiceRequested)
        }
        1 => {
            if observer != target {
                return Err(EffectError::Illegal("Sha target must answer".into()));
            }
            if cancelled {
                if selections.is_empty() {
                    return fallback(tx);
                }
                return Err(EffectError::Illegal(
                    "cancelled category gift contains a card".into(),
                ));
            }
            let category = rest
                .first()
                .and_then(|category| u8::try_from(*category).ok())
                .ok_or_else(|| EffectError::EngineFault("missing card category".into()))?;
            let [selected] = selections else {
                return Err(EffectError::Illegal("give one matching card".into()));
            };
            let selected = CardId::new(*selected);
            if !matching(tx, category).contains(&selected) {
                return Err(EffectError::Illegal("matching category card moved".into()));
            }
            let from = response_card_zone(tx, target, selected)?;
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("recipient hand missing".into()))?;
            run_effects(
                tx,
                vec![EffectRequest::GiveCards {
                    cards: vec![selected],
                    to,
                    from,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown category-gift-or-direct-hit stage".into(),
        )),
    }
}

pub(crate) fn resolve_jiaohua(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload
        .first()
        .ok_or_else(|| EffectError::EngineFault("missing Jiaohua stage".into()))?;
    let [selected] = selections else {
        return Err(EffectError::Illegal(
            "Jiaohua requires exactly one selection".into(),
        ));
    };
    match stage {
        0 => {
            let [_, player, skill, option_count, options @ ..] = payload else {
                return Err(EffectError::EngineFault(
                    "truncated Jiaohua category payload".into(),
                ));
            };
            if options.len() != *option_count as usize {
                return Err(EffectError::EngineFault(
                    "bad Jiaohua category payload".into(),
                ));
            }
            let player = PlayerId::new(*player);
            let category = u8::try_from(*selected)
                .map_err(|_| EffectError::Illegal("bad Jiaohua category token".into()))?;
            if cancelled || observer != player || !options.contains(selected) {
                return Err(EffectError::Illegal(
                    "Jiaohua category choice context changed".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveJiaohua {
                    player,
                    category,
                    target: PlayerId::NONE,
                    skill: SkillId::new(*skill),
                }],
            )
            .map(|_| ())
        }
        1 => {
            let [_, player, category, skill, option_count, options @ ..] = payload else {
                return Err(EffectError::EngineFault(
                    "truncated Jiaohua target payload".into(),
                ));
            };
            if options.len() != *option_count as usize {
                return Err(EffectError::EngineFault(
                    "bad Jiaohua target payload".into(),
                ));
            }
            let player = PlayerId::new(*player);
            let target = PlayerId::new(*selected);
            let category = u8::try_from(*category)
                .map_err(|_| EffectError::Illegal("bad Jiaohua category state".into()))?;
            if cancelled
                || observer != player
                || !options.contains(selected)
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal(
                    "Jiaohua target choice context changed".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveJiaohua {
                    player,
                    category,
                    target,
                    skill: SkillId::new(*skill),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown Jiaohua choice stage".into(),
        )),
    }
}
