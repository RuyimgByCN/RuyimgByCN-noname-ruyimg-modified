//! Effect primitive implementations — extra 2 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

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
        return super::run_effects(tx, effects).map(|_| Vec::new());
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BeigeOutcomePolicy {
    Classic,
    Configured {
        heart_recover_amount: u32,
        diamond_draw_count: u32,
    },
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn beige_stage_effect(
    policy: BeigeOutcomePolicy,
    player: PlayerId,
    target: PlayerId,
    source: PlayerId,
    stage: u8,
    result_card: CardId,
    suit: Suit,
    rank: u8,
    rewriters: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> EffectRequest {
    match policy {
        BeigeOutcomePolicy::Classic => EffectRequest::ResolveBeige {
            player,
            target,
            source,
            stage,
            result_card,
            suit,
            rank,
            rewriters,
            skill,
        },
        BeigeOutcomePolicy::Configured {
            heart_recover_amount,
            diamond_draw_count,
        } => EffectRequest::ResolveBeigeConfigured {
            player,
            target,
            source,
            stage,
            result_card,
            suit,
            rank,
            rewriters,
            heart_recover_amount,
            diamond_draw_count,
            skill,
        },
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_beige_with_policy(
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
    policy: BeigeOutcomePolicy,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx
                    .state()
                    .players
                    .get(player)
                    .is_some_and(|participant| participant.alive)
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                || source.is_none()
                || tx.state().players.get(source).is_none()
            {
                return Ok(Vec::new());
            }
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            let Some(only) = (options.len() == 1).then(|| options[0]) else {
                if options.is_empty() {
                    return Ok(Vec::new());
                }
                tx.state_mut()
                    .resolution
                    .deferred_tail_effects
                    .push(beige_stage_effect(
                        policy,
                        player,
                        target,
                        source,
                        1,
                        CardId::NONE,
                        Suit::None,
                        0,
                        Vec::new(),
                        skill,
                    ));
                let mut payload = vec![player.raw(), skill.raw(), 1, options.len() as u32];
                payload.extend(options.iter().map(|card| card.raw()));
                return open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    options.into_iter().map(CardId::raw).collect(),
                    ChoiceKind::SelectCards,
                    ContinuationKind::ChooseDiscardHandEquipThenRemoveSelfSkill,
                    payload,
                );
            };
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(beige_stage_effect(
                    policy,
                    player,
                    target,
                    source,
                    1,
                    CardId::NONE,
                    Suit::None,
                    0,
                    Vec::new(),
                    skill,
                ));
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player,
                    cards: vec![only],
                }],
            )
        }
        1 => {
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
            if tx
                .state()
                .zones
                .get(draw)
                .is_none_or(|zone| zone.cards.is_empty())
            {
                reshuffle_discard_into_draw(tx)?;
            }
            let result_card = tx
                .state()
                .zones
                .get(draw)
                .and_then(|zone| zone.cards.last())
                .copied()
                .ok_or_else(|| EffectError::Illegal("draw pile empty after reshuffle".into()))?;
            let card = tx
                .state()
                .cards
                .get(result_card)
                .ok_or_else(|| EffectError::Illegal("bad Beige judgment card".into()))?
                .clone();
            let suit = crate::query::effective_card_suit(tx.state(), target, &card);
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(beige_stage_effect(
                    policy,
                    player,
                    target,
                    source,
                    2,
                    result_card,
                    suit,
                    card.rank,
                    Vec::new(),
                    skill,
                ));
            run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![result_card],
                    to: discard,
                    from: Some(draw),
                }],
            )
        }
        2 => {
            let rewriters = crate::command::judge::judgment_rewriters(tx, target);
            continue_beige_judgment_rewriters_with_policy(
                tx,
                player,
                target,
                source,
                result_card,
                suit,
                rank,
                &rewriters,
                skill,
                policy,
            )
        }
        3 => match suit {
            Suit::Heart
                if tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive) =>
            {
                run_effects(
                    tx,
                    vec![EffectRequest::RecoverHp {
                        source: PlayerId::NONE,
                        player: target,
                        amount: match policy {
                            BeigeOutcomePolicy::Classic => 1,
                            BeigeOutcomePolicy::Configured {
                                heart_recover_amount,
                                ..
                            } => heart_recover_amount,
                        },
                    }],
                )
            }
            Suit::Diamond
                if tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive) =>
            {
                run_effects(
                    tx,
                    vec![EffectRequest::DrawCardsFromSkill {
                        player: target,
                        count: match policy {
                            BeigeOutcomePolicy::Classic => 2,
                            BeigeOutcomePolicy::Configured {
                                diamond_draw_count, ..
                            } => diamond_draw_count,
                        },
                        skill,
                    }],
                )
            }
            Suit::Club
                if tx
                    .state()
                    .players
                    .get(source)
                    .is_some_and(|participant| participant.alive) =>
            {
                run_effects(
                    tx,
                    vec![
                        EffectRequest::ChooseDiscardTargetCard {
                            player: source,
                            target: source,
                            scope: TargetCardScope::HandEquip,
                        },
                        EffectRequest::ChooseDiscardTargetCard {
                            player: source,
                            target: source,
                            scope: TargetCardScope::HandEquip,
                        },
                    ],
                )
            }
            Suit::Spade
                if tx
                    .state()
                    .players
                    .get(source)
                    .is_some_and(|participant| participant.alive) =>
            {
                run_effects(tx, vec![EffectRequest::TurnOver { player: source }])
            }
            _ => Ok(Vec::new()),
        },
        4 => continue_beige_judgment_rewriters_with_policy(
            tx,
            player,
            target,
            source,
            result_card,
            suit,
            rank,
            &rewriters,
            skill,
            policy,
        ),
        _ => Err(EffectError::ContentFault(
            "unknown Beige resolution stage".into(),
        )),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn continue_beige_judgment_rewriters(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    source: PlayerId,
    result_card: CardId,
    suit: Suit,
    rank: u8,
    rewriters: &[PlayerId],
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    continue_beige_judgment_rewriters_with_policy(
        tx,
        player,
        target,
        source,
        result_card,
        suit,
        rank,
        rewriters,
        skill,
        BeigeOutcomePolicy::Classic,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn continue_beige_judgment_rewriters_configured(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    source: PlayerId,
    result_card: CardId,
    suit: Suit,
    rank: u8,
    rewriters: &[PlayerId],
    heart_recover_amount: u32,
    diamond_draw_count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    continue_beige_judgment_rewriters_with_policy(
        tx,
        player,
        target,
        source,
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

#[allow(clippy::too_many_arguments)]
pub(crate) fn continue_beige_judgment_rewriters_with_policy(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    source: PlayerId,
    result_card: CardId,
    suit: Suit,
    rank: u8,
    rewriters: &[PlayerId],
    skill: engine_api::SkillId,
    policy: BeigeOutcomePolicy,
) -> Result<Vec<DomainEvent>, EffectError> {
    for (index, rewriter) in rewriters.iter().copied().enumerate() {
        let options = crate::command::judge::judgment_rewrite_cards(tx, rewriter);
        if options.is_empty() {
            continue;
        }
        let remaining = &rewriters[index..];
        let mut payload = vec![
            player.raw(),
            target.raw(),
            source.raw(),
            result_card.raw(),
            crate::command::judge::suit_tag(suit),
            u32::from(rank),
            skill.raw(),
            remaining.len() as u32,
        ];
        payload.extend(remaining.iter().map(|player| player.raw()));
        if let BeigeOutcomePolicy::Configured {
            heart_recover_amount,
            diamond_draw_count,
        } = policy
        {
            payload.extend([1, heart_recover_amount, diamond_draw_count]);
        }
        return open_choice(
            tx,
            rewriter,
            0,
            1,
            true,
            options.into_iter().map(CardId::raw).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::RewriteBeigeJudgment,
            payload,
        );
    }
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(beige_stage_effect(
            policy,
            player,
            target,
            source,
            3,
            result_card,
            suit,
            rank,
            Vec::new(),
            skill,
        ));
    run_effects(
        tx,
        vec![EffectRequest::EmitJudgmentResolved {
            player: target,
            result_card,
            suit,
            rank,
            cause_card: CardId::NONE,
        }],
    )
}

pub(crate) fn clear_phase_gift_target(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(instance) = tx.state_mut().skills.get_mut(skill) else {
        return Ok(Vec::new());
    };
    if instance.owner != player {
        return Err(EffectError::ContentFault(
            "phase gift state belongs to another player".into(),
        ));
    }
    instance.clear_phase_gift_target();
    Ok(Vec::new())
}

pub(crate) fn skip_current_phase(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if tx.state().turn.current != player
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "skip-current-phase owner is no longer current and alive".into(),
        ));
    }
    let phase = tx.state().turn.phase;
    if phase == Phase::Idle {
        return Err(EffectError::ContentFault(
            "skip_current_phase is not valid while idle".into(),
        ));
    }

    let mut events = vec![crate::command::turn::record_phase_skipped(
        tx, player, phase,
    )];
    if crate::query::restore_skipped_phase(tx.state(), player) {
        return Ok(events);
    }
    let transition = match phase {
        Phase::Draw => {
            if tx.state().turn.skip_play() {
                tx.state_mut().turn.set_flag(turn_flag::SKIP_PLAY, 0);
                events.push(crate::command::turn::record_phase_skipped(
                    tx,
                    player,
                    Phase::Play,
                ));
                set_turn(tx, player, Phase::Discard.to_u8())?
            } else {
                set_turn(tx, player, Phase::Play.to_u8())?
            }
        }
        Phase::Play if tx.state().turn.flag(turn_flag::OPENING_EXTRA_PLAY_ACTIVE) != 0 => {
            advance_opening_extra_play(tx, player)?
        }
        Phase::Play => set_turn(tx, player, Phase::Discard.to_u8())?,
        Phase::Discard => set_turn(tx, player, Phase::End.to_u8())?,
        Phase::Prepare | Phase::Judge | Phase::End => {
            crate::command::turn::finish_current_phase_abruptly(tx)?;
            Vec::new()
        }
        Phase::Idle => unreachable!(),
    };
    events.extend(transition);
    Ok(events)
}

pub(crate) fn finish_current_phase_abort_resolution(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
) -> Result<Vec<DomainEvent>, EffectError> {
    if tx.state().players.get(player).is_none() {
        return Err(EffectError::Illegal(
            "phase-abort skill owner is missing".into(),
        ));
    }
    match stage {
        0 => {
            let processing = tx
                .state()
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("processing zone is missing".into()))?;
            let cards = tx
                .state()
                .zones
                .get(processing)
                .map(|zone| zone.cards.clone())
                .unwrap_or_default();
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("discard pile is missing".into()))?;
            move_cards_with_reason(
                tx,
                cards,
                discard,
                Some(processing),
                CardMoveReason::Discard,
            )
        }
        1 => {
            crate::command::turn::finish_current_phase_abruptly(tx)?;
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault(
            "invalid phase-abort stage".into(),
        )),
    }
}

pub(crate) fn choose_other_targets_by_skipped_phase_count_damage(
    tx: &mut Transaction,
    player: PlayerId,
    amount: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if amount == 0
        || !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::End
    {
        return Ok(Vec::new());
    }
    let skipped = tx.state().turn.skipped_phase_count();
    if skipped == 0 {
        return Ok(Vec::new());
    }
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != player)
        .collect();
    let max = skipped.min(options.len() as u32).min(u8::MAX as u32) as u8;
    if max == 0 {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        max,
        false,
        options.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseOtherTargetsBySkippedPhaseCountDamage,
        vec![player.raw(), amount, skill.raw()],
    )
}

pub(crate) fn choose_other_damage(
    tx: &mut Transaction,
    player: PlayerId,
    amount: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if amount == 0
        || !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != player)
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
        ContinuationKind::ChooseOtherDamage,
        vec![player.raw(), amount, skill.raw()],
    )
}

pub(crate) fn choose_bound_source_attack_range_damage(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    amount: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let skill_matches = tx
        .state()
        .skills
        .get(skill)
        .filter(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        .and_then(|instance| instance.bound_target())
        == Some(source);
    if amount == 0
        || !skill_matches
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(source)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != source)
        .filter(|&target| crate::query::player_in_attack_range(tx.state(), source, target))
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
        ContinuationKind::ChooseBoundSourceAttackRangeDamage,
        vec![player.raw(), source.raw(), amount, skill.raw()],
    )
}

pub(crate) fn choose_other_discard_target_card(
    tx: &mut Transaction,
    player: PlayerId,
    scope: rule_api::TargetCardScope,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
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
            !crate::query::discardable_target_cards(tx.state(), player, target, scope).is_empty()
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
        ContinuationKind::ChooseOtherDiscardTargetCard,
        vec![player.raw(), u32::from(scope.tag()), skill.raw()],
    )
}

pub(crate) fn reveal_uniform_hand_draw_self_choose_others(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.phase == Phase::Idle
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let cards = tx.state().hand_cards(player);
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let mut events = vec![DomainEvent::CardsRevealed {
        cards: cards.clone(),
        to: Vec::new(),
    }];
    let mut color = None;
    for card in &cards {
        let current = crate::query::effective_card_color_mask(tx.state(), player, *card);
        if !matches!(current, 0b01 | 0b10) || color.is_some_and(|color| color != current) {
            return Ok(events);
        }
        color = Some(current);
    }
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != player)
        .collect();
    let max = cards.len().min(options.len()).min(usize::from(u8::MAX)) as u8;
    if max == 0 {
        events.extend(draw_cards(tx, player, 1)?);
        return Ok(events);
    }
    events.extend(open_choice(
        tx,
        player,
        1,
        max,
        true,
        options.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseOthersAfterUniformHandReveal,
        vec![
            player.raw(),
            u32::from(max),
            skill.raw(),
            tx.state().turn.current.raw(),
            u32::from(tx.state().turn.phase.to_u8()),
        ],
    )?);
    Ok(events)
}

pub(crate) fn reveal_uniform_hand_choose_damage(
    tx: &mut Transaction,
    player: PlayerId,
    amount: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if amount == 0
        || !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.phase == Phase::Idle
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let cards = tx.state().hand_cards(player);
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let mut events = vec![DomainEvent::CardsRevealed {
        cards: cards.clone(),
        to: Vec::new(),
    }];
    let mut color = None;
    for card in cards {
        let current = crate::query::effective_card_color_mask(tx.state(), player, card);
        if !matches!(current, 0b01 | 0b10) || color.is_some_and(|color| color != current) {
            return Ok(events);
        }
        color = Some(current);
    }
    let options = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .map(PlayerId::raw)
        .collect();
    events.extend(open_choice(
        tx,
        player,
        1,
        1,
        false,
        options,
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseOtherDamage,
        vec![player.raw(), amount, skill.raw()],
    )?);
    Ok(events)
}

pub(crate) fn choose_card_user_or_add_target(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || tx
            .state()
            .skills
            .get(skill)
            .is_some_and(|instance| instance.play_phase_fire_count > 0)
    {
        return Ok(Vec::new());
    }
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| {
            target != player
                && crate::command::play::saved_card_extra_target_is_legal(tx, player, card, target)
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
        ContinuationKind::ChooseCardUserOrAddTarget,
        vec![player.raw(), card.raw(), skill.raw()],
    )
}

pub(crate) fn discardable_equipment_cards(tx: &Transaction, player: PlayerId) -> Vec<CardId> {
    crate::query::discardable_hand_equip_cards(tx.state(), player)
        .into_iter()
        .filter(|card| {
            tx.state().cards.get(*card).is_some_and(|card_state| {
                crate::query::effective_card_kind(tx.state(), player, card_state).is_equip()
            })
        })
        .collect()
}

pub(crate) fn choose_discard_equipment_current_damage(
    tx: &mut Transaction,
    player: PlayerId,
    damage_id: u32,
    skill: engine_api::SkillId,
    prevent: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if damage_id == 0 || tx.state().resolution.current_damage_id != damage_id {
        return Err(EffectError::ContentFault(
            "equipment damage reduction used outside its damage_before event".into(),
        ));
    }
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let cards = discardable_equipment_cards(tx, player);
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        cards.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseDiscardEquipmentReduceCurrentDamage,
        vec![player.raw(), damage_id, skill.raw(), u32::from(prevent)],
    )
}

pub(crate) fn damage_endpoint_discard_cards(
    tx: &Transaction,
    player: PlayerId,
    target: PlayerId,
) -> Vec<CardId> {
    let mut cards: Vec<_> = [player, target]
        .into_iter()
        .flat_map(|endpoint| {
            crate::query::discardable_target_cards(
                tx.state(),
                player,
                endpoint,
                TargetCardScope::HandEquip,
            )
        })
        .collect();
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    cards
}

pub(crate) fn choose_discard_damage_endpoint_card_increase_if_suit(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    damage_id: u32,
    suit: u8,
    amount: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !(1..=4).contains(&suit) || amount == 0 {
        return Err(EffectError::ContentFault(
            "damage-endpoint discard requires a real suit and positive amount".into(),
        ));
    }
    if !originating_skill_is_enabled(tx, player, skill)
        || !bound_sha_damage_matches_any_segment(tx, damage_id, player, target)
    {
        return Err(EffectError::Illegal(
            "damage-endpoint discard context changed".into(),
        ));
    }
    let options = damage_endpoint_discard_cards(tx, player, target);
    if options.is_empty() {
        return Ok(Vec::new());
    }
    let mut payload = vec![
        player.raw(),
        target.raw(),
        damage_id,
        u32::from(suit),
        amount,
        skill.raw(),
        options.len() as u32,
    ];
    payload.extend(options.iter().map(|card| card.raw()));
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseDiscardDamageEndpointCardIncreaseIfSuit,
        payload,
    )
}

pub(crate) fn choose_move_target_equipment_or_draw(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    choose_move_target_equipment_draw_options(tx, player, target, skill, 0)
}

pub(crate) fn choose_move_target_equipment_and_or_draw(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    choose_move_target_equipment_draw_options(tx, player, target, skill, 3)
}

pub(crate) fn choose_draw_view_two_gain_or_move_equipment(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    choose_move_target_equipment_draw_options(tx, player, target, skill, 4)
}

pub(crate) fn choose_move_target_equipment_draw_options(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || player == target
        || !tx
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
        return Ok(Vec::new());
    }
    let mut options = vec![1]; // draw one
    if stage == 4 && !tx.state().hand_cards(target).is_empty() {
        options.push(2); // privately view up to two hand cards, then gain one
    }
    if !crate::query::movable_target_equipment_cards(tx.state(), player, target).is_empty() {
        options.insert(0, 0); // move one equipment card
    }
    open_choice(
        tx,
        player,
        1,
        if stage == 3 { options.len() as u8 } else { 1 },
        true,
        options,
        ChoiceKind::Custom,
        ContinuationKind::ChooseMoveTargetEquipmentOrDraw,
        vec![stage, player.raw(), target.raw(), skill.raw()],
    )
}

pub(crate) fn give_all_equipment_choose_recover_or_damage(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if player == target
        || !tx
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
            "equipment gift requires distinct living participants".into(),
        ));
    }
    let from = tx
        .state()
        .zones
        .find(ZoneKind::Equip, player)
        .ok_or_else(|| EffectError::Illegal("missing source equipment zone".into()))?;
    let cards = tx
        .state()
        .zones
        .get(from)
        .map(|zone| zone.cards.clone())
        .unwrap_or_default();
    if cards.is_empty() {
        return Err(EffectError::Illegal(
            "equipment gift requires at least one equipped card".into(),
        ));
    }
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("missing recipient hand zone".into()))?;
    let count = cards.len().min(u32::MAX as usize) as u32;
    super::run_effects(
        tx,
        vec![
            EffectRequest::GiveCards { cards, to, from },
            EffectRequest::ChooseRecoverOwnerOrDamageByGivenEquipment {
                owner: player,
                recipient: target,
                count,
                skill,
            },
        ],
    )
    .map(|_| Vec::new())
}

pub(crate) fn choose_recover_owner_or_damage_by_given_equipment(
    tx: &mut Transaction,
    owner: PlayerId,
    recipient: PlayerId,
    count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if count == 0
        || !tx
            .state()
            .skills
            .get(skill)
            .is_some_and(|instance| instance.owner == owner)
        || !tx
            .state()
            .players
            .get(recipient)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| crate::query::player_in_attack_range(tx.state(), recipient, target))
        .collect();
    if options.is_empty() {
        if tx
            .state()
            .players
            .get(owner)
            .is_some_and(|participant| participant.alive)
        {
            return recover_hp(tx, recipient, owner, count);
        }
        return Ok(Vec::new());
    }
    let max = count.min(options.len() as u32).min(u32::from(u8::MAX)) as u8;
    open_choice(
        tx,
        recipient,
        1,
        max,
        true,
        options.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseRecoverOwnerOrDamageByGivenEquipment,
        vec![owner.raw(), recipient.raw(), count, skill.raw()],
    )
}

pub(crate) fn physical_equipped_weapons(tx: &Transaction, player: PlayerId) -> Vec<CardId> {
    tx.state()
        .zones
        .find(ZoneKind::Equip, player)
        .and_then(|zone| tx.state().zones.get(zone))
        .map(|zone| {
            zone.cards
                .iter()
                .copied()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|card| card.kind == CardKind::Weapon)
                })
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn physical_weapon_range(tx: &Transaction, player: PlayerId) -> Option<u8> {
    physical_equipped_weapons(tx, player)
        .into_iter()
        .find_map(|card| {
            tx.state()
                .cards
                .get(card)
                .map(|card| card.weapon_range.max(1))
        })
}

pub(crate) fn bound_sha_damage_matches(
    tx: &Transaction,
    damage_id: u32,
    player: PlayerId,
    target: PlayerId,
) -> bool {
    bound_sha_damage_matches_inner(tx, damage_id, player, target, true)
}

pub(crate) fn bound_sha_damage_matches_any_segment(
    tx: &Transaction,
    damage_id: u32,
    player: PlayerId,
    target: PlayerId,
) -> bool {
    bound_sha_damage_matches_inner(tx, damage_id, player, target, false)
}

pub(crate) fn bound_sha_damage_matches_inner(
    tx: &Transaction,
    damage_id: u32,
    player: PlayerId,
    target: PlayerId,
    direct_only: bool,
) -> bool {
    damage_id != 0
        && tx.state().resolution.current_damage_id == damage_id
        && tx
            .state()
            .resolution
            .deferred_tail_effects
            .iter()
            .any(|request| {
                matches!(
                    request,
                    EffectRequest::CommitDamage {
                        damage_id: saved,
                        source,
                        target: saved_target,
                        is_lose_hp: false,
                        card_kind,
                        card_flags,
                        ..
                    } if *saved == damage_id
                        && *source == player
                        && *saved_target == target
                        && CardKind::from_tag(*card_kind).is_some_and(CardKind::is_sha)
                        && (!direct_only
                            || *card_flags & rule_api::damage_card_flag::CHAIN_PROPAGATED == 0)
                )
            })
}

pub(crate) fn bound_damage_matches(
    tx: &Transaction,
    damage_id: u32,
    player: PlayerId,
    target: PlayerId,
) -> bool {
    damage_id != 0
        && tx.state().resolution.current_damage_id == damage_id
        && tx
            .state()
            .resolution
            .deferred_tail_effects
            .iter()
            .any(|request| {
                matches!(
                    request,
                    EffectRequest::CommitDamage {
                        damage_id: saved,
                        source,
                        target: saved_target,
                        is_lose_hp: false,
                        ..
                    } if *saved == damage_id && *source == player && *saved_target == target
                ) || matches!(
                    request,
                    EffectRequest::CommitDamageBanActiveIfDying {
                        damage_id: saved,
                        source,
                        target: saved_target,
                        ..
                    } if *saved == damage_id && *source == player && *saved_target == target
                )
            })
}

pub(crate) const DAMAGE_DISCARD_WEAPON_RANGE: u32 = 0;
pub(crate) const DAMAGE_DISCARD_DISTANCE: u32 = 1;

pub(crate) fn distance_hand_discard_count(
    tx: &Transaction,
    player: PlayerId,
    target: PlayerId,
) -> Option<usize> {
    match crate::query::query(
        tx.state(),
        &rule_api::QueryRequest::Distance {
            from: player,
            to: target,
        },
        &mut crate::query::QueryContext::new(),
    ) {
        rule_api::QueryResponse::Int(distance) => {
            Some(4_i32.saturating_sub(distance).max(1) as usize)
        }
        _ => None,
    }
}

pub(crate) fn open_target_discard_hand_or_increase_damage(
    tx: &mut Transaction,
    mode: u32,
    player: PlayerId,
    target: PlayerId,
    damage_id: u32,
    required: usize,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let required = u8::try_from(required)
        .map_err(|_| EffectError::Illegal("damage discard count exceeds choice limit".into()))?;
    let options = tx.state().hand_cards(target);
    if options.len() < usize::from(required) {
        increase_bound_damage_by_one(tx, damage_id)?;
        return Ok(Vec::new());
    }
    let mut payload = vec![
        mode,
        player.raw(),
        target.raw(),
        damage_id,
        skill.raw(),
        u32::from(required),
        options.len().min(u32::MAX as usize) as u32,
    ];
    payload.extend(options.iter().map(|card| card.raw()));
    open_choice(
        tx,
        target,
        required,
        required,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseTargetDiscardWeaponRangeHandOrIncreaseDamage,
        payload,
    )
}

pub(crate) fn choose_target_discard_weapon_range_hand_or_increase_damage(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    damage_id: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || player == target
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || !bound_sha_damage_matches(tx, damage_id, player, target)
    {
        return Err(EffectError::Illegal(
            "weapon-range damage choice context changed".into(),
        ));
    }
    let required = physical_weapon_range(tx, player).ok_or_else(|| {
        EffectError::Illegal("weapon-range damage choice requires a physical weapon".into())
    })?;
    open_target_discard_hand_or_increase_damage(
        tx,
        DAMAGE_DISCARD_WEAPON_RANGE,
        player,
        target,
        damage_id,
        usize::from(required),
        skill,
    )
}

pub(crate) fn choose_target_discard_distance_hand_or_increase_damage(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    damage_id: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || player == target
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || !bound_damage_matches(tx, damage_id, player, target)
    {
        return Err(EffectError::Illegal(
            "distance damage choice context changed".into(),
        ));
    }
    let required = distance_hand_discard_count(tx, player, target)
        .ok_or_else(|| EffectError::Illegal("damage distance is unavailable".into()))?;
    open_target_discard_hand_or_increase_damage(
        tx,
        DAMAGE_DISCARD_DISTANCE,
        player,
        target,
        damage_id,
        required,
        skill,
    )
}

pub(crate) fn current_attack_range(tx: &Transaction, player: PlayerId) -> usize {
    match crate::query::query(
        tx.state(),
        &rule_api::QueryRequest::AttackRange { player },
        &mut crate::query::QueryContext::new(),
    ) {
        rule_api::QueryResponse::Int(range) => range.max(0) as usize,
        _ => 0,
    }
}

pub(crate) fn open_attack_range_discard_choice(
    tx: &mut Transaction,
    stage: u32,
    player: PlayerId,
    target: PlayerId,
    damage_id: u32,
    skill: engine_api::SkillId,
    attack_range: usize,
    options: Vec<CardId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    // ponytail: ChoiceRequest counts are u8; split into batches if decks can exceed 255 cards.
    let required = attack_range.min(options.len()).min(usize::from(u8::MAX)) as u8;
    if required == 0 {
        return Ok(Vec::new());
    }
    let mut payload = vec![
        stage,
        player.raw(),
        target.raw(),
        damage_id,
        skill.raw(),
        attack_range.min(u32::MAX as usize) as u32,
        options.len().min(u32::MAX as usize) as u32,
    ];
    payload.extend(options.iter().map(|card| card.raw()));
    open_choice(
        tx,
        target,
        required,
        required,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth,
        payload,
    )
}

pub(crate) fn choose_target_discard_attack_range_cards_or_increase_damage_weapon_forces_both(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    damage_id: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || player == target
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || !bound_sha_damage_matches(tx, damage_id, player, target)
    {
        return Err(EffectError::Illegal(
            "attack-range damage choice context changed".into(),
        ));
    }
    let attack_range = current_attack_range(tx, player);
    let options = crate::query::discardable_hand_equip_cards(tx.state(), target);
    let weapons = physical_equipped_weapons(tx, player);
    if !weapons.is_empty() {
        if options.is_empty() || attack_range == 0 {
            return super::run_effects(
                tx,
                vec![
                    EffectRequest::ContinueIncreaseBoundDamage {
                        player,
                        damage_id,
                        amount: 1,
                        skill,
                    },
                    EffectRequest::DiscardCards {
                        player,
                        cards: weapons,
                    },
                ],
            );
        }
        return open_attack_range_discard_choice(
            tx,
            2,
            player,
            target,
            damage_id,
            skill,
            attack_range,
            options,
        );
    }
    if options.is_empty() || attack_range == 0 {
        increase_bound_damage_by_one(tx, damage_id)?;
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        target,
        1,
        1,
        false,
        vec![1, 2],
        ChoiceKind::Custom,
        ContinuationKind::ChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth,
        vec![
            0,
            player.raw(),
            target.raw(),
            damage_id,
            skill.raw(),
            attack_range.min(u32::MAX as usize) as u32,
        ],
    )
}

pub(crate) fn choose_hand_count_different_target_draw_or_random_discard(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "hand-count target choice context changed".into(),
        ));
    }
    let hand_count = tx.state().hand_count(player);
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| tx.state().hand_count(target) != hand_count)
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    let mut payload = vec![player.raw(), skill.raw(), options.len() as u32];
    payload.extend(options.iter().map(|target| target.raw()));
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseHandCountDifferentTargetDrawOrRandomDiscard,
        payload,
    )
}

pub(crate) fn rezhenjun_context_is_current(
    tx: &Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> bool {
    originating_skill_is_enabled(tx, player, skill)
        && tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        && tx.state().turn.current == player
        && tx.state().turn.phase == Phase::Prepare
}

pub(crate) fn choose_target_discard_excess_then_self_discard_or_target_draw(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !rezhenjun_context_is_current(tx, player, skill) {
        return Err(EffectError::Illegal(
            "prepare-phase target discard context changed".into(),
        ));
    }
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| {
            !crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::HandEquip,
            )
            .is_empty()
        })
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    let mut payload = vec![0, player.raw(), skill.raw(), options.len() as u32];
    payload.extend(options.iter().map(|target| target.raw()));
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw,
        payload,
    )
}

pub(crate) fn choose_self_discard_or_target_draw_after_target_discard(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if count == 0 {
        return Ok(Vec::new());
    }
    if !rezhenjun_context_is_current(tx, player, skill)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "prepare-phase self-discard branch context changed".into(),
        ));
    }
    let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
    let required = usize::try_from(count).unwrap_or(usize::MAX);
    if options.len() < required || count > u32::from(u8::MAX) {
        return super::run_effects(
            tx,
            vec![EffectRequest::DrawCardsFromSkill {
                player: target,
                count,
                skill,
            }],
        );
    }
    let mut payload = vec![
        2,
        player.raw(),
        target.raw(),
        skill.raw(),
        count,
        options.len() as u32,
    ];
    payload.extend(options.iter().map(|card| card.raw()));
    open_choice(
        tx,
        player,
        count as u8,
        count as u8,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw,
        payload,
    )
}

pub(crate) fn decadezhenjun_context_is_current(
    tx: &Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> bool {
    originating_skill_is_enabled(tx, player, skill)
        && tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        && tx.state().turn.current == player
        && matches!(tx.state().turn.phase, Phase::Prepare | Phase::End)
}

pub(crate) fn resolve_decadezhenjun(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !decadezhenjun_context_is_current(tx, player, skill) {
                return Err(EffectError::Illegal(
                    "Decade Zhenjun phase context changed".into(),
                ));
            }
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&target| {
                    !crate::query::discardable_target_cards(
                        tx.state(),
                        player,
                        target,
                        rule_api::TargetCardScope::HandEquip,
                    )
                    .is_empty()
                })
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![3, player.raw(), skill.raw(), options.len() as u32];
            payload.extend(options.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw,
                payload,
            )
        }
        1 => {
            if count == 0
                || !decadezhenjun_context_is_current(tx, player, skill)
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if options.is_empty() {
                return super::run_effects(
                    tx,
                    vec![EffectRequest::DrawCardsFromSkill {
                        player: target,
                        count,
                        skill,
                    }],
                );
            }
            let mut payload = vec![
                5,
                player.raw(),
                target.raw(),
                skill.raw(),
                count,
                options.len() as u32,
            ];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw,
                payload,
            )
        }
        _ => Err(EffectError::Illegal(
            "invalid Decade Zhenjun internal stage".into(),
        )),
    }
}

pub(crate) fn sp_xiaoguo_initial_context_is_current(
    tx: &Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> bool {
    originating_skill_is_enabled(tx, player, skill)
        && player != target
        && tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        && tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        && tx.state().turn.current == target
        && tx.state().turn.phase == Phase::End
}

pub(crate) fn sp_xiaoguo_participants_alive(tx: &Transaction, player: PlayerId, target: PlayerId) -> bool {
    tx.state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        && tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
}

pub(crate) fn resolve_sp_xiaoguo(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !sp_xiaoguo_initial_context_is_current(tx, player, target, skill) {
                return Err(EffectError::Illegal(
                    "SP Xiaoguo phase context changed".into(),
                ));
            }
            let options = crate::query::sp_xiaoguo_hand_cards(tx.state(), player);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveSpXiaoguo,
                vec![0, player.raw(), target.raw(), skill.raw()],
            )
        }
        1 => {
            if !sp_xiaoguo_participants_alive(tx, player, target) {
                return Ok(Vec::new());
            }
            let options = crate::query::sp_xiaoguo_equipment_cards(tx.state(), target);
            if options.is_empty() {
                return super::run_effects(
                    tx,
                    vec![EffectRequest::ResolveSpXiaoguo {
                        player,
                        target,
                        stage: 2,
                        skill,
                    }],
                );
            }
            open_choice(
                tx,
                target,
                0,
                1,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveSpXiaoguo,
                vec![2, player.raw(), target.raw(), skill.raw()],
            )
        }
        2 => {
            if !sp_xiaoguo_participants_alive(tx, player, target) {
                return Ok(Vec::new());
            }
            super::run_effects(
                tx,
                vec![EffectRequest::DealDamage {
                    source: player,
                    target,
                    amount: 1,
                    is_lose_hp: false,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )
        }
        3 => {
            if !tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            super::run_effects(
                tx,
                vec![EffectRequest::DrawCardsFromSkill {
                    player,
                    count: 1,
                    skill,
                }],
            )
        }
        _ => Err(EffectError::Illegal(
            "invalid SP Xiaoguo internal stage".into(),
        )),
    }
}

pub(crate) fn eu_tuxi_context_is_current(
    tx: &Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> bool {
    originating_skill_is_enabled(tx, player, skill)
        && tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
}

/// Reproduce legacy `trigger.player`, `getPrevious()`, `getNext()` after the draw barrier.
/// Previous and next intentionally remain duplicated in a two-player game.
pub(crate) fn damage_target_and_legacy_neighbors(tx: &Transaction, target: PlayerId) -> Option<Vec<PlayerId>> {
    let target_seat = tx.state().players.get(target)?.seat.raw();
    let alive = tx.state().players.alive_by_seat();
    let mut targets = vec![target];
    if alive.len() <= 1 {
        return Some(targets);
    }

    if let Some(position) = alive.iter().position(|&player| player == target) {
        targets.push(alive[(position + alive.len() - 1) % alive.len()]);
        targets.push(alive[(position + 1) % alive.len()]);
        return Some(targets);
    }

    let previous = alive
        .iter()
        .rev()
        .copied()
        .find(|player| {
            tx.state()
                .players
                .get(*player)
                .is_some_and(|participant| participant.seat.raw() < target_seat)
        })
        .or_else(|| alive.last().copied());
    let next = alive
        .iter()
        .copied()
        .find(|player| {
            tx.state()
                .players
                .get(*player)
                .is_some_and(|participant| participant.seat.raw() > target_seat)
        })
        .or_else(|| alive.first().copied());
    targets.extend(previous);
    targets.extend(next);
    Some(targets)
}

pub(crate) fn eu_tuxi_gain_options(
    tx: &Transaction,
    player: PlayerId,
    target: PlayerId,
) -> Vec<CardId> {
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
    {
        return Vec::new();
    }
    let hand = (target != player)
        .then(|| tx.state().zones.find(ZoneKind::Hand, target))
        .flatten();
    let equip = tx.state().zones.find(ZoneKind::Equip, target);
    let mut options: Vec<_> = [hand, equip]
        .into_iter()
        .flatten()
        .filter_map(|zone| tx.state().zones.get(zone))
        .flat_map(|zone| zone.cards.iter().copied())
        .collect();
    options.sort_by_key(|card| card.raw());
    options.dedup();
    options
}

pub(crate) fn draw_then_choose_gain_one_from_damage_target_and_neighbors(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
    ready_to_choose: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !eu_tuxi_context_is_current(tx, player, skill) {
        return if ready_to_choose {
            Ok(Vec::new())
        } else {
            Err(EffectError::Illegal(
                "damage-source sequential gain context changed".into(),
            ))
        };
    }
    if tx.state().players.get(target).is_none() {
        return if ready_to_choose {
            Ok(Vec::new())
        } else {
            Err(EffectError::Illegal("missing damaged player".into()))
        };
    }
    if !ready_to_choose {
        let events = draw_cards(tx, player, 1)?;
        tx.state_mut().resolution.deferred_tail_effects.push(
            EffectRequest::DrawThenChooseGainOneFromDamageTargetAndNeighbors {
                player,
                target,
                skill,
                ready_to_choose: true,
            },
        );
        return Ok(events);
    }

    let Some(targets) = damage_target_and_legacy_neighbors(tx, target) else {
        return Ok(Vec::new());
    };
    continue_choose_gain_one_from_damage_target_and_neighbors(
        tx,
        player,
        targets,
        0,
        CardId::NONE,
        skill,
    )
}

pub(crate) fn continue_choose_gain_one_from_damage_target_and_neighbors(
    tx: &mut Transaction,
    player: PlayerId,
    targets: Vec<PlayerId>,
    mut index: u32,
    selected: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !eu_tuxi_context_is_current(tx, player, skill) {
        return Ok(Vec::new());
    }

    if selected.is_some() {
        let current = targets
            .get(index as usize)
            .copied()
            .ok_or_else(|| EffectError::Illegal("sequential gain index changed".into()))?;
        if !eu_tuxi_gain_options(tx, player, current).contains(&selected) {
            return Err(EffectError::Illegal(
                "sequential gain selection is invalid or stale".into(),
            ));
        }
        let from = tx
            .state()
            .cards
            .get(selected)
            .map(|card| card.zone)
            .ok_or_else(|| EffectError::Illegal("missing selected gain card".into()))?;
        let to = tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::Illegal("missing skill owner hand".into()))?;
        tx.state_mut().resolution.deferred_tail_effects.push(
            EffectRequest::ContinueChooseGainOneFromDamageTargetAndNeighbors {
                player,
                targets,
                index: index.saturating_add(1),
                selected: CardId::NONE,
                skill,
            },
        );
        return move_cards(tx, vec![selected], to, Some(from));
    }

    while let Some(current) = targets.get(index as usize).copied() {
        let options = eu_tuxi_gain_options(tx, player, current);
        if options.is_empty() {
            index = index.saturating_add(1);
            continue;
        }
        let mut payload = vec![player.raw(), skill.raw(), index, targets.len() as u32];
        payload.extend(targets.iter().map(|target| target.raw()));
        payload.push(options.len() as u32);
        payload.extend(options.iter().map(|card| card.raw()));
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            options.into_iter().map(CardId::raw).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::ChooseGainOneFromDamageTargetAndNeighbors,
            payload,
        );
    }
    Ok(Vec::new())
}

pub(crate) fn modify_current_damage(
    tx: &mut Transaction,
    player: PlayerId,
    delta: i32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if delta == 0 {
        return Ok(Vec::new());
    }
    let damage_id = tx.state().resolution.current_damage_id;
    if damage_id == 0 {
        return Err(EffectError::ContentFault(
            "current damage modification used outside its damage_before event".into(),
        ));
    }
    if delta > 0 {
        increase_bound_damage(tx, damage_id, delta as u32)?;
    } else {
        for _ in 0..(-delta) {
            reduce_bound_damage_by_one(tx, damage_id)?;
        }
    }
    let _ = player;
    Ok(Vec::new())
}

pub(crate) fn reduce_bound_damage_by_one(
    tx: &mut Transaction,
    damage_id: u32,
) -> Result<(u32, u32), EffectError> {
    if damage_id == 0 || tx.state().resolution.current_damage_id != damage_id {
        return Err(EffectError::Illegal(
            "damage reduction context changed".into(),
        ));
    }
    match tx
        .state_mut()
        .resolution
        .deferred_tail_effects
        .iter_mut()
        .find(|request| {
            matches!(
                request,
                EffectRequest::CommitDamage { damage_id: saved, .. }
                    | EffectRequest::CommitDamageBanActiveIfDying {
                        damage_id: saved,
                        ..
                    } if *saved == damage_id
            )
        }) {
        Some(request) => {
            let amount = match request {
                EffectRequest::CommitDamage { amount, .. }
                | EffectRequest::CommitDamageBanActiveIfDying { amount, .. } => amount,
                _ => unreachable!(),
            };
            let before = *amount;
            let after = before.saturating_sub(1);
            *amount = after;
            Ok((before, after))
        }
        // 批 282：触发执行期 CommitDamage 尚未入列（defer_damage_tail 之前），
        // 把减量暂存到 pending 队列，deferral 时统一套用。
        None => {
            tx.state_mut()
                .resolution
                .pending_bound_damage_deltas
                .push((damage_id, -1));
            Ok((0, 0))
        }
    }
}
