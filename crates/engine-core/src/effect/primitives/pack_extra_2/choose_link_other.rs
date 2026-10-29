//! Effect primitives for `choose_link_other` and related helpers.

use super::*;

pub(crate) fn choose_link_other(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::End
    {
        return Ok(Vec::new());
    }
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != player)
        .filter(|&target| {
            !tx.state().players.get(target).is_some_and(|participant| {
                participant
                    .marks
                    .iter()
                    .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
            })
        })
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseLinkOther,
        vec![player.raw(), skill.raw()],
    )
}

pub(crate) fn resolve_drlt_poxi(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    hand_limit_skill_def: engine_api::SkillDefId,
    stage: u8,
    own_count: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if hand_limit_skill_def.is_none() || stage > 1 || own_count > 4 {
        return Err(EffectError::ContentFault(
            "Poxi requires a helper skill and valid state".into(),
        ));
    }
    if stage == 1 {
        if !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        {
            return Ok(Vec::new());
        }
        let effects = match own_count {
            0 => vec![EffectRequest::ChangeMaxHp { player, delta: -1 }],
            1 => vec![
                EffectRequest::AddSkillUntilTurnEnd {
                    player,
                    skill_def: hand_limit_skill_def,
                },
                EffectRequest::SkipCurrentPhase { player },
            ],
            2 => Vec::new(),
            3 => vec![EffectRequest::RecoverHp {
                source: player,
                player,
                amount: 1,
            }],
            4 => vec![EffectRequest::DrawCards { player, count: 4 }],
            _ => unreachable!(),
        };
        return super::super::run_effects(tx, effects).map(|_| Vec::new());
    }
    if !originating_skill_is_enabled(tx, player, skill)
        || player == target
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || tx.state().hand_cards(target).is_empty()
    {
        return Ok(Vec::new());
    }
    let mut options = tx.state().hand_cards(player);
    options.extend(crate::query::discardable_target_cards(
        tx.state(),
        player,
        target,
        rule_api::TargetCardScope::Hand,
    ));
    options.retain(|card| {
        tx.state().cards.get(*card).is_some_and(|card| {
            crate::query::effective_card_suit(tx.state(), card.owner, card) != Suit::None
        })
    });
    options.sort_by_key(|card| card.raw());
    options.dedup();
    let mut payload = vec![
        player.raw(),
        target.raw(),
        hand_limit_skill_def.raw(),
        skill.raw(),
        options.len() as u32,
    ];
    payload.extend(options.iter().map(|card| card.raw()));
    open_choice(
        tx,
        player,
        4,
        4,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseDrltPoxiCards,
        payload,
    )
}

pub(crate) fn resolve_drlt_jieying(
    tx: &mut Transaction,
    player: PlayerId,
    event_player: PlayerId,
    mark: engine_api::MarkDefId,
    mode: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if mark.is_none() || mode > 2 {
        return Err(EffectError::ContentFault(
            "Jieying requires a mark and valid mode".into(),
        ));
    }
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let mark_count = |tx: &Transaction, target: PlayerId| {
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
    match mode {
        0 => {
            if event_player != player
                || tx.state().turn.current != player
                || tx
                    .state()
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .any(|target| mark_count(tx, target) > 0)
            {
                return Ok(Vec::new());
            }
            add_mark(tx, player, mark.raw(), 1)
        }
        1 => {
            if event_player != player
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::End
                || mark_count(tx, player) == 0
            {
                return Ok(Vec::new());
            }
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&target| target != player && mark_count(tx, target) == 0)
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ChooseDrltJieyingTransfer,
                vec![player.raw(), mark.raw(), skill.raw()],
            )
        }
        2 => {
            if event_player == player
                || !tx
                    .state()
                    .players
                    .get(event_player)
                    .is_some_and(|participant| participant.alive)
                || mark_count(tx, event_player) == 0
            {
                return Ok(Vec::new());
            }
            let mut events = remove_mark(tx, event_player, mark.raw(), 0)?;
            let from = tx
                .state()
                .zones
                .find(ZoneKind::Hand, event_player)
                .ok_or_else(|| EffectError::Illegal("missing Jieying source hand".into()))?;
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("missing Jieying owner hand".into()))?;
            let cards = tx
                .state()
                .zones
                .get(from)
                .map(|zone| zone.cards.clone())
                .unwrap_or_default();
            if !cards.is_empty() {
                events.extend(give_cards_as(tx, event_player, cards, to, from)?);
            }
            Ok(events)
        }
        _ => unreachable!(),
    }
}

pub(crate) fn eligible_drlt_duorui_skills(
    tx: &Transaction,
    target: PlayerId,
) -> Vec<(engine_api::SkillDefId, engine_api::SkillId)> {
    const BANNED: [u32; 18] = [
        3_743_953_905,
        3_569_170_747,
        2_525_731_933,
        2_662_892_097,
        4_036_172_090,
        4_046_904_597,
        2_934_243_125,
        2_223_381_613,
        2_345_258_373,
        3_484_005_747,
        3_081_703_772,
        2_706_590_540,
        2_975_766_196,
        3_929_269_413,
        3_212_702_585,
        3_111_899_420,
        3_435_019_014,
        2_799_539_959,
    ];
    let Some(player) = tx.state().players.get(target) else {
        return Vec::new();
    };
    let Some(character) = tx.state().content.character(player.character_def) else {
        return Vec::new();
    };
    let base: std::collections::BTreeSet<_> = character.skills.iter().copied().collect();
    let mut eligible: Vec<_> = player
        .skills
        .iter()
        .filter_map(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .map(|instance| (*skill, instance.def))
        })
        .filter(|(_, def)| base.contains(&def.raw()) && !BANNED.contains(&def.raw()))
        .filter(|(_, def)| {
            tx.state()
                .content
                .skill_metadata(*def)
                .is_some_and(|metadata| !metadata.lord_only && !metadata.non_gainable)
        })
        .filter(|(_, def)| {
            !tx.state()
                .content
                .active_defs_for_skill(*def)
                .iter()
                .any(|active| active.once_per_game)
        })
        .map(|(skill, def)| (def, skill))
        .collect();
    eligible.sort_by_key(|(def, skill)| (def.raw(), skill.raw()));
    eligible.dedup_by_key(|(def, _)| def.raw());
    eligible
}

pub(crate) fn resolve_drlt_duorui(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    helper_skill_def: engine_api::SkillDefId,
    stage: u8,
    selected_skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if helper_skill_def.is_none() || stage > 3 {
        return Err(EffectError::ContentFault(
            "Duorui requires a helper skill and valid stage".into(),
        ));
    }
    if stage == 3 {
        let Some(helper) = tx.state().skills.get(skill).cloned() else {
            return Ok(Vec::new());
        };
        let Some((borrower, duorui, _target_skill, borrowed_skill)) = helper.drlt_duorui_record()
        else {
            return Err(EffectError::ContentFault(
                "Duorui cleanup helper has invalid state".into(),
            ));
        };
        if helper.owner != player || helper.def != helper_skill_def || target != player {
            return Ok(Vec::new());
        }
        let mut events = Vec::new();
        if borrowed_skill.is_some() && tx.state().skills.get(borrowed_skill).is_some() {
            events.extend(remove_skill(tx, borrowed_skill)?);
        }
        if let Some(main) = tx.state_mut().skills.get_mut(duorui) {
            if main.owner == borrower && main.drlt_duorui_helper() == Some(skill) {
                main.state_blob.clear();
            }
        }
        if tx.state().skills.get(skill).is_some() {
            events.extend(remove_skill(tx, skill)?);
        }
        return Ok(events);
    }
    if !originating_skill_is_enabled(tx, player, skill)
        || player == target
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || tx
            .state()
            .skills
            .get(skill)
            .is_some_and(|instance| instance.drlt_duorui_helper().is_some())
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let eligible = eligible_drlt_duorui_skills(tx, target);
    if eligible.is_empty() {
        return Ok(Vec::new());
    }
    if stage == 0 {
        let enabled_slots: Vec<_> = tx
            .state()
            .players
            .get(player)
            .expect("validated Duorui owner")
            .disabled_equip_slots
            .iter()
            .enumerate()
            .filter_map(|(index, count)| (*count == 0).then_some(index as u32 + 1))
            .collect();
        if enabled_slots.is_empty() {
            return resolve_drlt_duorui(
                tx,
                player,
                target,
                helper_skill_def,
                1,
                engine_api::SkillDefId::NONE,
                skill,
            );
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            enabled_slots,
            ChoiceKind::Custom,
            ContinuationKind::ChooseDrltDuoruiDisableSlot,
            vec![
                player.raw(),
                target.raw(),
                helper_skill_def.raw(),
                skill.raw(),
            ],
        );
    }
    if stage == 1 {
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            eligible.iter().map(|(def, _)| def.raw()).collect(),
            ChoiceKind::Custom,
            ContinuationKind::ChooseDrltDuoruiSkill,
            vec![
                player.raw(),
                target.raw(),
                helper_skill_def.raw(),
                skill.raw(),
            ],
        );
    }
    let Some((_, target_skill)) = eligible
        .into_iter()
        .find(|(def, _)| *def == selected_skill_def)
    else {
        return Err(EffectError::Illegal(
            "selected Duorui skill is no longer eligible".into(),
        ));
    };
    let (helper, mut events) = grant_skill_instance(tx, target, helper_skill_def, Vec::new())?;
    let already_owned = tx.state().players.get(player).is_some_and(|owner| {
        owner.skills.iter().any(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def == selected_skill_def)
        })
    });
    let borrowed = if already_owned {
        engine_api::SkillId::NONE
    } else {
        let (borrowed, added) = grant_skill_instance(tx, player, selected_skill_def, Vec::new())?;
        events.extend(added);
        borrowed
    };
    tx.state_mut()
        .skills
        .get_mut(helper)
        .expect("new Duorui helper")
        .set_drlt_duorui_record(player, skill, target_skill, borrowed);
    let disabled = tx
        .state_mut()
        .skills
        .get_mut(target_skill)
        .expect("eligible Duorui target skill");
    disabled.disabled_by_skills.push(helper);
    disabled
        .disabled_by_skills
        .sort_by_key(|reason| reason.raw());
    disabled.disabled_by_skills.dedup();
    tx.state_mut()
        .skills
        .get_mut(skill)
        .expect("validated Duorui skill")
        .set_drlt_duorui_helper(helper);
    tx.state_mut().rebuild_derived_indexes();
    Ok(events)
}

pub(crate) fn resolve_drlt_zhiti(
    tx: &mut Transaction,
    player: PlayerId,
    event_player: PlayerId,
    event_source: PlayerId,
    event_amount: u32,
    mode: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if mode > 2 {
        return Err(EffectError::ContentFault("invalid Zhiti mode".into()));
    }
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let other = match mode {
        0 if event_amount.saturating_sub(1) == player.raw() => {
            if event_player == player {
                event_source
            } else if event_source == player {
                event_player
            } else {
                return Ok(Vec::new());
            }
        }
        1 if event_player == player => event_source,
        2 if event_player == player => event_source,
        _ => return Ok(Vec::new()),
    };
    if other.is_none()
        || !tx.state().players.get(other).is_some_and(|participant| {
            participant.alive && participant.hp < participant.max_hp as i32
        })
        || !crate::query::player_in_attack_range(tx.state(), player, other)
    {
        return Ok(Vec::new());
    }
    let options: Vec<_> = tx
        .state()
        .players
        .get(player)
        .into_iter()
        .flat_map(|owner| owner.disabled_equip_slots.iter().enumerate())
        .filter_map(|(index, count)| (*count > 0).then_some(index as u32 + 1))
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options,
        ChoiceKind::Custom,
        ContinuationKind::ChooseDrltZhitiEnableSlot,
        vec![player.raw(), skill.raw()],
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_shen_simayi(
    tx: &mut Transaction,
    player: PlayerId,
    _event_player: PlayerId,
    _event_amount: u32,
    mark: engine_api::MarkDefId,
    mode: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let enabled = tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    });
    if !enabled {
        return Ok(Vec::new());
    }
    let mark_count = tx
        .state()
        .players
        .get(player)
        .into_iter()
        .flat_map(|player| player.marks.iter())
        .find(|(definition, _)| *definition == mark.raw())
        .map(|(_, count)| (*count).max(0) as u32)
        .unwrap_or(0);

    match mode {
        0 => {
            let discarded = tx
                .state()
                .turn
                .flag(turn_flag::DISCARD_PHASE_DISCARDED)
                .max(0);
            add_mark(tx, player, mark.raw(), discarded)
        }
        1 => {
            if tx
                .state()
                .turn
                .flag(turn_flag::kill_count_by_player(player))
                <= 0
                || tx.state().turn.extra_turn_queue.len() >= 8
            {
                return Ok(Vec::new());
            }
            tx.state_mut().turn.extra_turn_queue.push(player);
            Ok(Vec::new())
        }
        2 => {
            if mark_count == 0
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
            {
                return Ok(Vec::new());
            }
            let skill_def = tx
                .state()
                .skills
                .get(skill)
                .map(|instance| instance.def)
                .ok_or_else(|| EffectError::Illegal("missing Jilue Wansha skill".into()))?;
            tx.state_mut().turn.non_locked_skill_suppression = skill;
            tx.state_mut().rebuild_derived_indexes();
            run_effects(
                tx,
                vec![
                    EffectRequest::RemoveMark {
                        player,
                        mark,
                        amount: 1,
                    },
                    EffectRequest::AddMark {
                        player,
                        mark: engine_api::MarkDefId::new(skill_def.raw()),
                        delta: 1,
                    },
                ],
            )?;
            Ok(Vec::new())
        }
        3 => {
            if mark_count == 0 || cards.is_empty() {
                return Ok(Vec::new());
            }
            let count = selected_discard_draw_count(tx, player, &cards)?;
            run_effects(
                tx,
                vec![
                    EffectRequest::RemoveMark {
                        player,
                        mark,
                        amount: 1,
                    },
                    EffectRequest::DiscardCards { player, cards },
                    EffectRequest::DrawCardsFromSkill {
                        player,
                        count,
                        skill,
                    },
                ],
            )?;
            Ok(Vec::new())
        }
        4 => {
            if mark_count == 0 {
                return Ok(Vec::new());
            }
            let mut events = remove_mark(tx, player, mark.raw(), 1)?;
            let draw_events = draw_cards(tx, player, 1)?;
            let drawn = draw_events.iter().find_map(|event| match event {
                DomainEvent::CardsDrawn {
                    player: owner,
                    cards,
                } if *owner == player => cards.first().copied(),
                _ => None,
            });
            events.extend(draw_events);
            if let Some(card) = drawn {
                tx.state_mut().resolution.deferred_tail_effects.push(
                    EffectRequest::ResolveShenSimayi {
                        player,
                        event_player: PlayerId::NONE,
                        event_amount: 0,
                        mark,
                        mode: 5,
                        cards: vec![card],
                        targets: Vec::new(),
                        skill,
                    },
                );
            }
            Ok(events)
        }
        5 => {
            let Some(card) = cards.first().copied() else {
                return Ok(Vec::new());
            };
            if !tx.state().hand_cards(player).contains(&card)
                || !tx
                    .state()
                    .cards
                    .get(card)
                    .is_some_and(|card| card.kind.is_basic())
            {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                vec![0, 1],
                ChoiceKind::Confirm,
                ContinuationKind::ChooseJilueJizhiDiscard,
                vec![player.raw(), card.raw(), skill.raw()],
            )
        }
        _ => Err(EffectError::ContentFault(
            "unknown Shen Simayi resolution mode".into(),
        )),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_beige(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    source: PlayerId,
    stage: u8,
    result_card: CardId,
    suit: Suit,
    rank: u8,
    rewriters: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    resolve_beige_with_policy(
        tx,
        player,
        target,
        source,
        stage,
        result_card,
        suit,
        rank,
        rewriters,
        skill,
        BeigeOutcomePolicy::Classic,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_beige_configured(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    source: PlayerId,
    stage: u8,
    result_card: CardId,
    suit: Suit,
    rank: u8,
    rewriters: Vec<PlayerId>,
    heart_recover_amount: u32,
    diamond_draw_count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    resolve_beige_with_policy(
        tx,
        player,
        target,
        source,
        stage,
        result_card,
        suit,
        rank,
        rewriters,
        skill,
        BeigeOutcomePolicy::Configured {
            heart_recover_amount,
            diamond_draw_count,
        },
    )
}
