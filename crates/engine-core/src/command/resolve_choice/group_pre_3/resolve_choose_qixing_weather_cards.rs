//! Command operations for `resolve_choose_inspected_heart_destination` and related helpers.

use super::*;

pub(crate) fn resolve_choose_qixing_weather_cards(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, _skill_raw, qixing_raw, _helper_raw, max_targets] = payload else {
        return Err(EffectError::Illegal(
            "bad Qixing weather card payload".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    if observer != player {
        return Err(EffectError::Illegal(
            "Qixing weather observer mismatch".into(),
        ));
    }
    if cancelled {
        return if selections.is_empty() {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "cancelled Qixing weather choice must be empty".into(),
            ))
        };
    }
    let current: std::collections::BTreeSet<_> = tx
        .state()
        .skill_pile_cards(player, *qixing_raw)
        .into_iter()
        .map(CardId::raw)
        .collect();
    let selected: std::collections::BTreeSet<_> = selections.iter().copied().collect();
    if selections.is_empty()
        || selections.len() > *max_targets as usize
        || selected.len() != selections.len()
        || !selected.is_subset(&current)
    {
        return Err(EffectError::Illegal(
            "Qixing weather cards are no longer legal".into(),
        ));
    }
    let mut next_payload = payload.to_vec();
    next_payload.push(selections.len() as u32);
    next_payload.extend_from_slice(selections);
    crate::effect::open_choice(
        tx,
        player,
        selections.len() as u8,
        selections.len() as u8,
        false,
        tx.state()
            .players
            .alive_by_seat()
            .into_iter()
            .map(PlayerId::raw)
            .collect(),
        engine_api::ChoiceKind::SelectTargets,
        ContinuationKind::ChooseQixingWeatherTargets,
        next_payload,
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_qixing_weather_targets(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 6 {
        return Err(EffectError::Illegal(
            "bad Qixing weather target payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let count = payload[5] as usize;
    if observer != player
        || cancelled
        || count == 0
        || payload.len() != 6 + count
        || selections.len() != count
    {
        return Err(EffectError::Illegal(
            "Qixing weather requires equal card and target counts".into(),
        ));
    }
    let qixing_skill_def = engine_api::SkillDefId::new(payload[2]);
    let cards: Vec<_> = payload[6..].iter().copied().map(CardId::new).collect();
    let current: std::collections::BTreeSet<_> = tx
        .state()
        .skill_pile_cards(player, qixing_skill_def.raw())
        .into_iter()
        .collect();
    let selected: std::collections::BTreeSet<_> = cards.iter().copied().collect();
    let mut unique_targets = std::collections::BTreeSet::new();
    if selected.len() != cards.len()
        || !selected.is_subset(&current)
        || !selections.iter().all(|target| {
            let target = PlayerId::new(*target);
            unique_targets.insert(target)
                && tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
        })
    {
        return Err(EffectError::Illegal(
            "Qixing weather selection is no longer legal".into(),
        ));
    }
    let pile = tx
        .state()
        .skill_piles
        .iter()
        .find(|pile| pile.owner == player && pile.skill_def == qixing_skill_def.raw())
        .map(|pile| pile.zone)
        .ok_or_else(|| EffectError::Illegal("Qixing pile is missing".into()))?;
    let discard = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("discard pile is missing".into()))?;
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveQixingWeather {
            player,
            qixing_skill_def,
            helper_skill_def: engine_api::SkillDefId::new(payload[3]),
            max_targets: payload[4] as u8,
            stage: 1,
            cards: cards.clone(),
            targets: selections.iter().copied().map(PlayerId::new).collect(),
            skill: engine_api::SkillId::new(payload[1]),
        });
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards,
            to: discard,
            from: Some(pile),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_link_other(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, skill_raw] = payload else {
        return Err(EffectError::Illegal("bad link-other choice payload".into()));
    };
    let player = PlayerId::new(*player_raw);
    let [target_raw] = selections else {
        return Err(EffectError::Illegal(
            "link-other choice requires one target".into(),
        ));
    };
    let target = PlayerId::new(*target_raw);
    let skill = engine_api::SkillId::new(*skill_raw);
    let target_unlinked = tx.state().players.get(target).is_some_and(|participant| {
        participant.alive
            && !participant
                .marks
                .iter()
                .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
    });
    if observer != player
        || cancelled
        || player == target
        || !target_unlinked
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::End
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "link-other choice is no longer legal".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::AddMark {
            player: target,
            mark: engine_api::MarkDefId::new(crate::content::MARK_CHAIN),
            delta: 1,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_drlt_poxi_cards(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 {
        return Err(EffectError::Illegal("bad Poxi choice payload".into()));
    }
    let player = PlayerId::new(payload[0]);
    let target = PlayerId::new(payload[1]);
    let hand_limit_skill_def = engine_api::SkillDefId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    let option_count = payload[4] as usize;
    let options = payload
        .get(5..5 + option_count)
        .ok_or_else(|| EffectError::Illegal("truncated Poxi choice payload".into()))?;
    if observer != player {
        return Err(EffectError::Illegal("Poxi chooser mismatch".into()));
    }
    if cancelled {
        if !selections.is_empty() {
            return Err(EffectError::Illegal(
                "cancelled Poxi choice must be empty".into(),
            ));
        }
        return Ok(());
    }
    if selections.len() != 4 {
        return Err(EffectError::Illegal(
            "Poxi requires exactly four cards".into(),
        ));
    }
    let mut selected = selections.to_vec();
    selected.sort_unstable();
    selected.dedup();
    if selected.len() != 4 || selected.iter().any(|card| !options.contains(card)) {
        return Err(EffectError::Illegal(
            "Poxi selected cards are not distinct offered options".into(),
        ));
    }
    let player_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("missing Poxi owner hand".into()))?;
    let target_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("missing Poxi target hand".into()))?;
    if player == target
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "Poxi context is no longer legal".into(),
        ));
    }
    let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
    let mut suits = Vec::with_capacity(4);
    let mut own_cards = Vec::new();
    let mut target_cards = Vec::new();
    for card in cards {
        let state = tx
            .state()
            .cards
            .get(card)
            .ok_or_else(|| EffectError::Illegal("Poxi card no longer exists".into()))?;
        if state.owner == player && state.zone == player_hand {
            own_cards.push(card);
        } else if state.owner == target && state.zone == target_hand {
            target_cards.push(card);
        } else {
            return Err(EffectError::Illegal(
                "Poxi card left the selected hands".into(),
            ));
        }
        let suit = crate::query::effective_card_suit(tx.state(), state.owner, state);
        if suit == engine_api::Suit::None || suits.contains(&suit) {
            return Err(EffectError::Illegal(
                "Poxi cards must have four distinct suits".into(),
            ));
        }
        suits.push(suit);
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("missing discard pile".into()))?;
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveDrltPoxi {
            player,
            target,
            hand_limit_skill_def,
            stage: 1,
            own_count: own_cards.len() as u8,
            skill,
        });
    let mut effects = Vec::new();
    if !own_cards.is_empty() {
        effects.push(EffectRequest::MoveCards {
            cards: own_cards,
            to: discard,
            from: Some(player_hand),
        });
    }
    if !target_cards.is_empty() {
        effects.push(EffectRequest::MoveCards {
            cards: target_cards,
            to: discard,
            from: Some(target_hand),
        });
    }
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolve_choose_drlt_jieying_transfer(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, mark_raw, skill_raw] = payload else {
        return Err(EffectError::Illegal("bad Jieying transfer payload".into()));
    };
    let [target_raw] = selections else {
        return Err(EffectError::Illegal(
            "Jieying transfer requires one target".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    let target = PlayerId::new(*target_raw);
    let mark = engine_api::MarkDefId::new(*mark_raw);
    let skill = engine_api::SkillId::new(*skill_raw);
    let mark_count = |target: PlayerId| {
        tx.state()
            .players
            .get(target)
            .and_then(|participant| {
                participant
                    .marks
                    .iter()
                    .find(|(found, _)| *found == mark.raw())
                    .map(|(_, count)| *count)
            })
            .unwrap_or(0)
            .max(0) as u32
    };
    let count = mark_count(player);
    if observer != player
        || cancelled
        || player == target
        || count == 0
        || mark_count(target) != 0
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::End
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "Jieying transfer is no longer legal".into(),
        ));
    }
    run_effects(
        tx,
        vec![
            EffectRequest::RemoveMark {
                player,
                mark,
                amount: 0,
            },
            EffectRequest::AddMark {
                player: target,
                mark,
                delta: count.min(i32::MAX as u32) as i32,
            },
        ],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_drlt_duorui_disable_slot(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, target_raw, helper_raw, skill_raw] = payload else {
        return Err(EffectError::Illegal("bad Duorui slot payload".into()));
    };
    let [slot] = selections else {
        return Err(EffectError::Illegal(
            "Duorui requires one equipment slot".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    let slot = u8::try_from(*slot)
        .ok()
        .and_then(crate::state::EquipSlot::from_tag)
        .ok_or_else(|| EffectError::Illegal("invalid Duorui equipment slot".into()))?;
    if observer != player
        || cancelled
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|owner| owner.disabled_equip_slots[slot.index()] == 0)
    {
        return Err(EffectError::Illegal(
            "Duorui equipment slot is no longer enabled".into(),
        ));
    }
    run_effects(
        tx,
        vec![
            EffectRequest::DisableEquipSlots {
                player,
                slots: vec![slot as u8],
            },
            EffectRequest::ResolveDrltDuorui {
                player,
                target: PlayerId::new(*target_raw),
                helper_skill_def: engine_api::SkillDefId::new(*helper_raw),
                stage: 1,
                selected_skill_def: engine_api::SkillDefId::NONE,
                skill: engine_api::SkillId::new(*skill_raw),
            },
        ],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_drlt_duorui_skill(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, target_raw, helper_raw, skill_raw] = payload else {
        return Err(EffectError::Illegal("bad Duorui skill payload".into()));
    };
    let [selected] = selections else {
        return Err(EffectError::Illegal("Duorui requires one skill".into()));
    };
    let player = PlayerId::new(*player_raw);
    if observer != player || cancelled {
        return Err(EffectError::Illegal("invalid Duorui skill choice".into()));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveDrltDuorui {
            player,
            target: PlayerId::new(*target_raw),
            helper_skill_def: engine_api::SkillDefId::new(*helper_raw),
            stage: 2,
            selected_skill_def: engine_api::SkillDefId::new(*selected),
            skill: engine_api::SkillId::new(*skill_raw),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_drlt_zhiti_enable_slot(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, skill_raw] = payload else {
        return Err(EffectError::Illegal("bad Zhiti slot payload".into()));
    };
    let [slot] = selections else {
        return Err(EffectError::Illegal(
            "Zhiti requires one equipment slot".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    let slot = u8::try_from(*slot)
        .ok()
        .and_then(crate::state::EquipSlot::from_tag)
        .ok_or_else(|| EffectError::Illegal("invalid Zhiti equipment slot".into()))?;
    let skill = engine_api::SkillId::new(*skill_raw);
    if observer != player
        || cancelled
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|owner| owner.disabled_equip_slots[slot.index()] > 0)
    {
        return Err(EffectError::Illegal(
            "Zhiti equipment slot is no longer disabled".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::EnableEquipSlot {
            player,
            slot: slot as u8,
            source: player,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_jilue_jizhi_discard(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, card_raw, skill_raw] = payload else {
        return Err(EffectError::Illegal("bad Jilue Jizhi payload".into()));
    };
    let [discard] = selections else {
        return Err(EffectError::Illegal(
            "Jilue Jizhi requires one confirmation".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    if observer != player || cancelled || *discard > 1 {
        return Err(EffectError::Illegal(
            "invalid Jilue Jizhi confirmation".into(),
        ));
    }
    if *discard == 0 {
        return Ok(());
    }
    let card = CardId::new(*card_raw);
    let skill = engine_api::SkillId::new(*skill_raw);
    let skill_def = tx
        .state()
        .skills
        .get(skill)
        .filter(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        .map(|instance| instance.def)
        .ok_or_else(|| EffectError::Illegal("Jilue Jizhi skill context changed".into()))?;
    if !tx.state().hand_cards(player).contains(&card)
        || !tx.state().cards.get(card).is_some_and(|card_state| {
            crate::query::effective_card_kind(tx.state(), player, card_state).is_basic()
        })
    {
        return Err(EffectError::Illegal(
            "Jilue Jizhi drawn card is no longer discardable".into(),
        ));
    }
    run_effects(
        tx,
        vec![
            EffectRequest::DiscardCards {
                player,
                cards: vec![card],
            },
            EffectRequest::AddMark {
                player,
                mark: engine_api::MarkDefId::new(skill_def.raw()),
                delta: 1,
            },
        ],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_max_mark_other_judge_kill_unless_tao_or_taoyuan(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, mark_raw, _skill_raw] = payload else {
        return Err(EffectError::Illegal(
            "bad max-mark death judgment payload".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    if observer != player || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "max-mark death judgment requires one target".into(),
        ));
    }
    let mark = engine_api::MarkDefId::new(*mark_raw);
    let target = PlayerId::new(selections[0]);
    let mark_count = |candidate: &crate::state::Player| {
        candidate
            .marks
            .iter()
            .find(|(definition, _)| *definition == mark.raw())
            .map_or(0, |(_, count)| *count)
    };
    let max = tx
        .state()
        .players
        .iter_live()
        .filter(|candidate| candidate.alive && candidate.id != player)
        .map(mark_count)
        .max()
        .unwrap_or(0);
    let valid = max > 0
        && tx.state().players.get(target).is_some_and(|candidate| {
            candidate.alive && candidate.id != player && mark_count(candidate) == max
        });
    if !valid {
        return Err(EffectError::Illegal(
            "max-mark death judgment target is stale".into(),
        ));
    }

    let result = crate::command::judge::flip_judgment(
        tx,
        target,
        crate::command::judge::JudgmentFavor::Neutral,
    )?;
    let kind = tx
        .state()
        .cards
        .get(result.card)
        .map(|card| card.kind)
        .ok_or_else(|| EffectError::EngineFault("judgment result card is missing".into()))?;
    let mut effects = vec![EffectRequest::EmitJudgmentResolved {
        player: target,
        result_card: result.card,
        suit: result.suit,
        rank: result.rank,
        cause_card: CardId::NONE,
    }];
    if !matches!(kind, CardKind::Tao | CardKind::TrickTaoyuan) {
        effects.push(EffectRequest::KillPlayer {
            player: target,
            killer: PlayerId::NONE,
        });
    }
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolve_recover_to_max_then_optional_exchange_skill(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [removable_skill_def, restore_skill_def] = payload else {
        return Err(EffectError::Illegal(
            "bad recover-to-max skill exchange payload".into(),
        ));
    };
    if cancelled || selections.len() != 1 || !matches!(selections[0], 0 | 1) {
        return Err(EffectError::Illegal(
            "recover-to-max skill exchange requires one confirm choice".into(),
        ));
    }
    if selections[0] == 0 {
        return Ok(());
    }
    let removable_skill_def = engine_api::SkillDefId::new(*removable_skill_def);
    let restore_skill_def = engine_api::SkillDefId::new(*restore_skill_def);
    let valid = tx.state().players.get(observer).is_some_and(|participant| {
        participant.alive
            && participant.skills.iter().any(|skill| {
                tx.state()
                    .skills
                    .get(*skill)
                    .is_some_and(|instance| instance.def == removable_skill_def)
            })
    });
    if !valid {
        return Ok(());
    }
    run_effects(
        tx,
        vec![
            EffectRequest::RemoveSkillByDef {
                player: observer,
                skill_def: removable_skill_def,
            },
            EffectRequest::AddSkill {
                player: observer,
                skill_def: restore_skill_def,
            },
        ],
    )
    .map(|_| ())
}

pub(crate) fn response_card_zone(
    tx: &Transaction,
    player: PlayerId,
    card: CardId,
) -> Result<engine_api::ZoneId, EffectError> {
    let card = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("bad response card".into()))?;
    let in_hand = tx.state().zones.find(ZoneKind::Hand, player) == Some(card.zone);
    let in_equip = tx.state().zones.find(ZoneKind::Equip, player) == Some(card.zone);
    let in_expansion = tx.state().zones.find(ZoneKind::Expansion, player) == Some(card.zone);
    let borrowed = crate::query::borrowed_hand_source(tx.state(), player, card.id).is_some();
    let fusi = crate::command::skill::dddfusi_approved_source(tx.state(), player, card.id)
        .is_some()
        && card.face_up
        && tx
            .state()
            .zones
            .get(card.zone)
            .is_some_and(|zone| zone.kind == ZoneKind::Hand);
    if (!borrowed && !fusi && card.owner != player)
        || (!in_hand && !in_equip && !in_expansion && !borrowed && !fusi)
    {
        return Err(EffectError::Illegal(
            "response card not owned in hand, equip, or expansion".into(),
        ));
    }
    Ok(card.zone)
}
