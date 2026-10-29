//! Command operations for `resolve_choose_gain_sha_for_opening_extra_play` and related helpers.

use super::*;

pub(crate) fn resolve_choose_gain_sha_for_opening_extra_play(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, card, from, skill, damage_baseline] = payload else {
        return Err(EffectError::Illegal(
            "bad opening extra Play Sha-gain payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let card = CardId::new(*card);
    let from = engine_api::ZoneId::new(*from);
    let skill = engine_api::SkillId::new(*skill);
    if observer != player {
        return Err(EffectError::Illegal(
            "opening extra Play Sha gain belongs to another player".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return if selections.is_empty() {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "cancelled opening extra Play Sha gain has selections".into(),
            ))
        };
    }
    if selections != [1] {
        return Err(EffectError::Illegal(
            "opening extra Play Sha gain requires the confirm option".into(),
        ));
    }
    let state = tx.state();
    if state.turn.current != player
        || state.turn.phase != Phase::Play
        || state.turn.opening_extra_play_active_skill != skill
        || !state.skills.get(skill).is_some_and(|instance| {
            instance.owner == player && state.skill_effectively_enabled(instance)
        })
        || !state.cards.get(card).is_some_and(|candidate| {
            candidate.zone == from
                && candidate.kind == CardKind::Sha
                && state.zones.get(from).is_some_and(|zone| {
                    matches!(zone.kind, ZoneKind::DrawPile | ZoneKind::DiscardPile)
                })
        })
    {
        return Err(EffectError::Illegal(
            "opening extra Play Sha-gain context changed".into(),
        ));
    }
    let hand = state
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let policy = crate::state::SkillInstance::phase_card_use_policy_state(
        card,
        crate::state::SkillInstance::CARD_USE_POLICY_DISTANCE_UNLIMITED,
        *damage_baseline,
    );
    tx.state_mut()
        .skills
        .get_mut(skill)
        .expect("validated originating skill")
        .state_blob = policy;
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: hand,
            from: Some(from),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_opening_extra_play_hp_loss_gain(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, kind, mandatory_skill_def, skill] = payload else {
        return Err(EffectError::Illegal(
            "bad opening extra Play HP-loss gain payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let skill = engine_api::SkillId::new(*skill);
    let mandatory_skill_def = engine_api::SkillDefId::new(*mandatory_skill_def);
    if observer != player {
        return Err(EffectError::Illegal(
            "opening extra Play HP-loss gain belongs to another player".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return if selections.is_empty() {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "cancelled opening extra Play HP-loss gain has selections".into(),
            ))
        };
    }
    if selections != [1] {
        return Err(EffectError::Illegal(
            "opening extra Play HP-loss gain requires the confirm option".into(),
        ));
    }
    let state = tx.state();
    if state.turn.current != player
        || state.turn.phase != Phase::Play
        || state.turn.opening_extra_play_active_skill != skill
        || !state.skills.get(skill).is_some_and(|instance| {
            instance.owner == player && state.skill_effectively_enabled(instance)
        })
        || state
            .skills
            .sorted_live()
            .into_iter()
            .any(|instance| instance.owner == player && instance.def == mandatory_skill_def)
    {
        return Err(EffectError::Illegal(
            "opening extra Play HP-loss gain context changed".into(),
        ));
    }
    crate::effect::commit_opening_extra_play_hp_loss_gain(
        tx,
        player,
        u8::try_from(*kind)
            .map_err(|_| EffectError::Illegal("invalid opening gain card kind".into()))?,
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_target_draw_or_discard_by_skill_use_count_link_if_wounded(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.first().unwrap_or(&u32::MAX);
    let player = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let target = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    let count = *payload.get(4).unwrap_or(&0);
    let toggle_link = payload.get(5).copied().unwrap_or(0) != 0;
    if cancelled || count == 0 {
        return Err(EffectError::Illegal(
            "draw-or-discard branch cannot be cancelled or empty".into(),
        ));
    }
    if !tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    }) || !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "draw-or-discard branch is no longer available".into(),
        ));
    }
    let finish = EffectRequest::FinishLinkIfTargetWounded {
        player,
        target,
        toggle_link,
        skill,
    };
    match (stage, selections) {
        (0, [1]) if observer == player => run_effects(
            tx,
            vec![
                EffectRequest::DrawCardsFromSkill {
                    player: target,
                    count,
                    skill,
                },
                finish,
            ],
        )
        .map(|_| ()),
        (0, [2]) if observer == player => {
            let options = crate::query::discardable_hand_equip_cards(tx.state(), target);
            if options.is_empty() {
                return Err(EffectError::Illegal(
                    "target discard branch is no longer available".into(),
                ));
            }
            let required = count.min(options.len() as u32) as u8;
            crate::effect::open_choice(
                tx,
                target,
                required,
                required,
                false,
                options.into_iter().map(CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                ContinuationKind::ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded,
                vec![
                    1,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                    count,
                    u32::from(toggle_link),
                ],
            )
            .map(|_| ())
        }
        (1, _) if observer == target => {
            let options: std::collections::BTreeSet<u32> =
                crate::query::discardable_hand_equip_cards(tx.state(), target)
                    .into_iter()
                    .map(CardId::raw)
                    .collect();
            let selected: std::collections::BTreeSet<u32> = selections.iter().copied().collect();
            let required = count.min(options.len() as u32) as usize;
            if required == 0
                || selections.len() != required
                || selected.len() != required
                || !selected.is_subset(&options)
            {
                return Err(EffectError::Illegal(
                    "target must discard the current required card count".into(),
                ));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::DiscardCards {
                        player: target,
                        cards: selections.iter().copied().map(CardId::new).collect(),
                    },
                    finish,
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal(
            "choose one valid draw-or-discard branch".into(),
        )),
    }
}

pub(crate) fn resolve_choose_reveal_other_hand_card_remember_category(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.first().unwrap_or(&u32::MAX);
    let player = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(2).unwrap_or(&0));
    let bound_target = PlayerId::new(*payload.get(3).unwrap_or(&0));
    let phase_owner = if stage == 2 { bound_target } else { player };
    if cancelled
        || observer != player
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || tx.state().turn.current != phase_owner
        || tx.state().turn.phase != Phase::Play
    {
        return Err(EffectError::Illegal(
            "remembered-category choice context changed".into(),
        ));
    }
    match (stage, selections) {
        (0, [target]) => {
            let target = PlayerId::new(*target);
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal(
                    "choose another living player with hand cards".into(),
                ));
            }
            let options: Vec<_> = tx.state().hand_cards(target);
            if options.is_empty() {
                return Err(EffectError::Illegal(
                    "chosen player's hand is now empty".into(),
                ));
            }
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                ContinuationKind::ChooseRevealOtherHandCardRememberCategory,
                vec![1, player.raw(), skill.raw(), target.raw()],
            )
            .map(|_| ())
        }
        (1, [card]) if payload.len() == 4 => {
            let target = PlayerId::new(payload[3]);
            let card = CardId::new(*card);
            if !tx.state().hand_cards(target).contains(&card) {
                return Err(EffectError::Illegal(
                    "revealed card is no longer in the chosen hand".into(),
                ));
            }
            let category = tx
                .state()
                .cards
                .get(card)
                .map(|card| crate::query::effective_card_kind(tx.state(), target, card))
                .map(|kind| {
                    if kind.is_basic() {
                        1
                    } else if kind.is_trick() {
                        2
                    } else if kind.is_equip() {
                        4
                    } else {
                        0
                    }
                })
                .unwrap_or(0);
            if !tx
                .state_mut()
                .skills
                .get_mut(skill)
                .is_some_and(|instance| instance.set_remembered_category(category))
            {
                return Err(EffectError::Illegal(
                    "revealed card has no supported logical category".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::RevealCards {
                    cards: vec![card],
                    to: Vec::new(),
                }],
            )
            .map(|_| ())
        }
        (2, [card]) if payload.len() == 4 => {
            let target = bound_target;
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal(
                    "bound reveal target is no longer alive".into(),
                ));
            }
            let card = CardId::new(*card);
            if !tx.state().hand_cards(target).contains(&card) {
                return Err(EffectError::Illegal(
                    "revealed card is no longer in the bound hand".into(),
                ));
            }
            let category = tx
                .state()
                .cards
                .get(card)
                .map(|card| crate::query::effective_card_kind(tx.state(), target, card))
                .map(|kind| {
                    if kind.is_basic() {
                        1
                    } else if kind.is_trick() {
                        2
                    } else if kind.is_equip() {
                        4
                    } else {
                        0
                    }
                })
                .unwrap_or(0);
            if !tx
                .state_mut()
                .skills
                .get_mut(skill)
                .is_some_and(|instance| instance.set_bound_category_target(target, category))
            {
                return Err(EffectError::Illegal(
                    "revealed card has no supported bound category".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::RevealCards {
                    cards: vec![card],
                    to: Vec::new(),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal(
            "choose exactly one reveal target or hand card".into(),
        )),
    }
}

pub(crate) fn resolve_give_exact_record_phase_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault(
            "bad exact phase-gift continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let target = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    let count = payload[3] as usize;
    if cancelled || observer != player || count == 0 || selections.len() != count {
        return Err(EffectError::Illegal(
            "phase gift requires the exact card count".into(),
        ));
    }
    if !tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    }) || tx.state().turn.current != target
        || tx.state().turn.phase != Phase::Play
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "phase gift context changed before payment".into(),
        ));
    }
    let selected: std::collections::BTreeSet<_> = selections.iter().copied().collect();
    if selected.len() != count {
        return Err(EffectError::Illegal(
            "phase gift cards must be distinct".into(),
        ));
    }
    let hand = tx.state().zones.find(ZoneKind::Hand, player);
    let equip = tx.state().zones.find(ZoneKind::Equip, player);
    let current: std::collections::BTreeMap<_, _> = selections
        .iter()
        .copied()
        .map(CardId::new)
        .filter_map(|card| {
            tx.state()
                .cards
                .get(card)
                .filter(|state| {
                    state.owner == player && (Some(state.zone) == hand || Some(state.zone) == equip)
                })
                .map(|state| (card, state.zone))
        })
        .collect();
    if current.len() != count {
        return Err(EffectError::Illegal(
            "one or more phase gift cards left hand/equipment".into(),
        ));
    }
    if !tx
        .state_mut()
        .skills
        .get_mut(skill)
        .is_some_and(|instance| instance.set_phase_gift_target(target))
    {
        return Err(EffectError::ContentFault(
            "could not record phase gift target".into(),
        ));
    }
    let target_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::EngineFault("phase gift target hand missing".into()))?;
    let mut by_zone = std::collections::BTreeMap::<u32, Vec<CardId>>::new();
    for card in selections.iter().copied().map(CardId::new) {
        let zone = current[&card];
        by_zone.entry(zone.raw()).or_default().push(card);
    }
    let effects = by_zone
        .into_iter()
        .map(|(zone, cards)| EffectRequest::GiveCards {
            cards,
            to: target_hand,
            from: engine_api::ZoneId::new(zone),
        })
        .collect();
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolved_sha_gift_groups(
    payload: &[u32],
    offset: usize,
) -> Result<(Vec<CardId>, Vec<CardId>, Vec<PlayerId>), EffectError> {
    let sha_len = *payload
        .get(offset)
        .ok_or_else(|| EffectError::EngineFault("missing resolved Sha group length".into()))?
        as usize;
    let sha_end = offset
        .checked_add(1)
        .and_then(|start| start.checked_add(sha_len))
        .ok_or_else(|| EffectError::EngineFault("resolved Sha group overflow".into()))?;
    let sha: Vec<_> = payload
        .get(offset + 1..sha_end)
        .ok_or_else(|| EffectError::EngineFault("truncated resolved Sha group".into()))?
        .iter()
        .copied()
        .map(CardId::new)
        .collect();
    let shan_len = *payload
        .get(sha_end)
        .ok_or_else(|| EffectError::EngineFault("missing resolved Shan group length".into()))?
        as usize;
    let shan_end = sha_end
        .checked_add(1)
        .and_then(|start| start.checked_add(shan_len))
        .ok_or_else(|| EffectError::EngineFault("resolved Shan group overflow".into()))?;
    let shan: Vec<_> = payload
        .get(sha_end + 1..shan_end)
        .ok_or_else(|| EffectError::EngineFault("truncated resolved Shan group".into()))?
        .iter()
        .copied()
        .map(CardId::new)
        .collect();
    let excluded_len = *payload
        .get(shan_end)
        .ok_or_else(|| EffectError::EngineFault("missing resolved Sha exclusions".into()))?
        as usize;
    let excluded_end = shan_end
        .checked_add(1)
        .and_then(|start| start.checked_add(excluded_len))
        .ok_or_else(|| EffectError::EngineFault("resolved Sha exclusions overflow".into()))?;
    if payload.len() != excluded_end {
        return Err(EffectError::EngineFault(
            "truncated resolved Sha exclusions".into(),
        ));
    }
    let excluded = payload[shan_end + 1..excluded_end]
        .iter()
        .copied()
        .map(PlayerId::new)
        .collect();
    Ok((sha, shan, excluded))
}

pub(crate) fn current_discard_group(tx: &Transaction, cards: &[CardId]) -> Vec<CardId> {
    let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
    cards
        .iter()
        .copied()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|state| Some(state.zone) == discard)
        })
        .collect()
}

pub(crate) fn give_resolved_sha_group(
    tx: &mut Transaction,
    owner: PlayerId,
    recipient: PlayerId,
    skill: engine_api::SkillId,
    cards: &[CardId],
    draw_if_black: bool,
) -> Result<(), EffectError> {
    let cards = current_discard_group(tx, cards);
    if cards.is_empty() {
        return Err(EffectError::Illegal(
            "chosen resolved card group is no longer in discard".into(),
        ));
    }
    let red = cards.iter().any(|card| {
        tx.state().cards.get(*card).is_some_and(|card| {
            matches!(
                card.suit,
                engine_api::Suit::Heart | engine_api::Suit::Diamond
            )
        })
    });
    let black = draw_if_black
        && cards.iter().any(|card| {
            tx.state().cards.get(*card).is_some_and(|card| {
                matches!(card.suit, engine_api::Suit::Spade | engine_api::Suit::Club)
            })
        });
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, recipient)
        .ok_or_else(|| EffectError::Illegal("no recipient hand".into()))?;
    if red {
        tx.state_mut().resolution.deferred_tail_effects.push(
            EffectRequest::ChooseUseShaAfterResolvedGift {
                owner,
                player: recipient,
                skill,
            },
        );
    }
    if black {
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::DrawCards {
                player: recipient,
                count: 1,
            });
    }
    run_effects(
        tx,
        vec![EffectRequest::GiveCards {
            cards,
            to: hand,
            from: discard,
        }],
    )
    .map(|_| ())
}
