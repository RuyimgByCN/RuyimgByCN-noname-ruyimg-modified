//! Effect primitives for `resolve_dc_sb_zhangxiu` and related helpers.

use super::*;

pub(crate) fn resolve_dc_sb_zhangxiu(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    action: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || tx.state().turn.current != target
                || tx.state().turn.phase != Phase::Play
                || target == player
            {
                return Ok(Vec::new());
            }
            let max_hand = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(|candidate| tx.state().hand_count(candidate))
                .max()
                .unwrap_or(0);
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|target_state| target_state.alive)
                || tx.state().hand_count(target) != max_hand
            {
                return Ok(Vec::new());
            }
            let can_give = !dc_sb_zhangxiu_hand_equip_cards(tx, player).is_empty();
            let can_discard = !crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::HandEquip,
            )
            .is_empty();
            let can_sha = crate::command::play::source_free_sha_can_target_without_distance(
                tx.state(),
                player,
                target,
            );
            let mut options = Vec::new();
            if can_give {
                options.push(0);
            }
            if can_discard || can_sha {
                options.push(1);
            }
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
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DC_SB_ZHANGXIU_CHOICE,
                    0,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )
        }
        1 => try_use_virtual_sha_on_without_distance(tx, player, target, false, 0),
        10 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::End
            {
                return Ok(Vec::new());
            }
            let cards = dcsbhaoyi_cards(tx);
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let events = gain_event_cards(tx, player, &cards)?;
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveDcSbZhangxiu {
                    player,
                    target: PlayerId::NONE,
                    action: 11,
                    cards,
                    skill,
                },
            );
            Ok(events)
        }
        11 => {
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("Haoyi owner hand missing".into()))?;
            let mut cards: Vec<_> = cards
                .into_iter()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|card| card.owner == player && card.zone == hand)
                })
                .collect();
            cards.sort_by_key(|card| card.raw());
            cards.dedup();
            let has_other = tx
                .state()
                .players
                .iter_live()
                .any(|candidate| candidate.alive && candidate.id != player);
            if cards.is_empty() || !has_other {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                RESOLVE_DC_SB_ZHANGXIU_CHOICE,
                3,
                player.raw(),
                skill.raw(),
                cards.len() as u32,
            ];
            payload.extend(cards.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                cards.len().min(u8::MAX as usize) as u8,
                true,
                cards.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault(
            "unknown DC SB Zhangxiu action".into(),
        )),
    }
}

pub(crate) fn resolve_dc_sb_zhangxiu_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&RESOLVE_DC_SB_ZHANGXIU_CHOICE) || payload.len() < 4 {
        return Err(EffectError::EngineFault(
            "truncated DC SB Zhangxiu choice".into(),
        ));
    }
    let mode = payload[1];
    let player = PlayerId::new(payload[2]);
    if observer != player {
        return Err(EffectError::Illegal("Zhangxiu chooser mismatch".into()));
    }
    match mode {
        0 => {
            if cancelled || selections.len() != 1 || payload.len() != 5 {
                return Err(EffectError::Illegal("invalid Fuxi branch".into()));
            }
            let target = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            match selections[0] {
                0 => {
                    let cards = dc_sb_zhangxiu_hand_equip_cards(tx, player);
                    if cards.is_empty() {
                        return Err(EffectError::Illegal("Fuxi give card is stale".into()));
                    }
                    let mut next = vec![
                        RESOLVE_DC_SB_ZHANGXIU_CHOICE,
                        1,
                        player.raw(),
                        target.raw(),
                        skill.raw(),
                        cards.len() as u32,
                    ];
                    next.extend(cards.iter().map(|card| card.raw()));
                    open_choice(
                        tx,
                        player,
                        1,
                        1,
                        false,
                        cards.into_iter().map(CardId::raw).collect(),
                        ChoiceKind::SelectCards,
                        ContinuationKind::Custom,
                        next,
                    )
                    .map(|_| ())
                }
                1 => {
                    let cards = crate::query::discardable_target_cards(
                        tx.state(),
                        player,
                        target,
                        rule_api::TargetCardScope::HandEquip,
                    );
                    if cards.is_empty() {
                        return run_effects(
                            tx,
                            vec![EffectRequest::ResolveDcSbZhangxiu {
                                player,
                                target,
                                action: 1,
                                cards: Vec::new(),
                                skill,
                            }],
                        )
                        .map(|_| ());
                    }
                    let mut next = vec![
                        RESOLVE_DC_SB_ZHANGXIU_CHOICE,
                        2,
                        player.raw(),
                        target.raw(),
                        skill.raw(),
                        cards.len() as u32,
                    ];
                    next.extend(cards.iter().map(|card| card.raw()));
                    open_choice(
                        tx,
                        player,
                        1,
                        1,
                        false,
                        cards.into_iter().map(CardId::raw).collect(),
                        ChoiceKind::SelectCards,
                        ContinuationKind::Custom,
                        next,
                    )
                    .map(|_| ())
                }
                _ => Err(EffectError::Illegal("unknown Fuxi branch".into())),
            }
        }
        1 | 2 => {
            if cancelled || selections.len() != 1 || payload.len() < 6 {
                return Err(EffectError::Illegal("invalid Fuxi card choice".into()));
            }
            let count = payload[5] as usize;
            if payload.len() != 6 + count || !payload[6..].contains(&selections[0]) {
                return Err(EffectError::Illegal("stale Fuxi card choice".into()));
            }
            let target = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            let card = CardId::new(selections[0]);
            if mode == 1 {
                let from = tx
                    .state()
                    .cards
                    .get(card)
                    .filter(|card| card.owner == player)
                    .map(|card| card.zone)
                    .ok_or_else(|| EffectError::Illegal("Fuxi give card moved".into()))?;
                let to = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, target)
                    .ok_or_else(|| EffectError::Illegal("Fuxi target hand missing".into()))?;
                run_effects(
                    tx,
                    vec![
                        EffectRequest::GiveCards {
                            cards: vec![card],
                            to,
                            from,
                        },
                        EffectRequest::DrawCards { player, count: 2 },
                    ],
                )
                .map(|_| ())
            } else {
                if !crate::query::discardable_target_cards(
                    tx.state(),
                    player,
                    target,
                    rule_api::TargetCardScope::HandEquip,
                )
                .contains(&card)
                {
                    return Err(EffectError::Illegal("Fuxi discard card moved".into()));
                }
                run_effects(
                    tx,
                    vec![
                        EffectRequest::DiscardCards {
                            player: target,
                            cards: vec![card],
                        },
                        EffectRequest::ResolveDcSbZhangxiu {
                            player,
                            target,
                            action: 1,
                            cards: Vec::new(),
                            skill,
                        },
                    ],
                )
                .map(|_| ())
            }
        }
        3 => {
            if cancelled {
                return Ok(());
            }
            if selections.is_empty() || payload.len() < 5 {
                return Err(EffectError::Illegal("invalid Haoyi card choice".into()));
            }
            let count = payload[4] as usize;
            if payload.len() != 5 + count
                || selections.iter().any(|card| !payload[5..].contains(card))
            {
                return Err(EffectError::Illegal("stale Haoyi card choice".into()));
            }
            let mut selected = selections.to_vec();
            selected.sort_unstable();
            selected.dedup();
            if selected.len() != selections.len() {
                return Err(EffectError::Illegal("duplicate Haoyi card".into()));
            }
            let mut remaining: Vec<_> = payload[5..]
                .iter()
                .copied()
                .filter(|card| !selected.contains(card))
                .collect();
            remaining.sort_unstable();
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| *target != player)
                .collect();
            if targets.is_empty() {
                return Ok(());
            }
            let mut next = vec![
                RESOLVE_DC_SB_ZHANGXIU_CHOICE,
                4,
                player.raw(),
                payload[3],
                selected.len() as u32,
            ];
            next.extend(selected);
            next.push(remaining.len() as u32);
            next.extend(remaining);
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                targets.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                next,
            )
            .map(|_| ())
        }
        4 => {
            if cancelled || selections.len() != 1 || payload.len() < 6 {
                return Err(EffectError::Illegal("invalid Haoyi target choice".into()));
            }
            let selected_count = payload[4] as usize;
            let remaining_index = 5 + selected_count;
            if payload.len() <= remaining_index {
                return Err(EffectError::EngineFault("truncated Haoyi plan".into()));
            }
            let remaining_count = payload[remaining_index] as usize;
            if payload.len() != remaining_index + 1 + remaining_count {
                return Err(EffectError::EngineFault("malformed Haoyi plan".into()));
            }
            let target = PlayerId::new(selections[0]);
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|target| target.alive)
            {
                return Err(EffectError::Illegal("Haoyi target is stale".into()));
            }
            let cards: Vec<_> = payload[5..remaining_index]
                .iter()
                .copied()
                .map(CardId::new)
                .collect();
            let remaining: Vec<_> = payload[remaining_index + 1..]
                .iter()
                .copied()
                .map(CardId::new)
                .collect();
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("Haoyi owner hand missing".into()))?;
            if cards.iter().any(|card| {
                !tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|card| card.owner == player && card.zone == hand)
            }) {
                return Err(EffectError::Illegal("Haoyi cards moved".into()));
            }
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("Haoyi target hand missing".into()))?;
            run_effects(
                tx,
                vec![
                    EffectRequest::GiveCards {
                        cards,
                        to,
                        from: hand,
                    },
                    EffectRequest::ResolveDcSbZhangxiu {
                        player,
                        target: PlayerId::NONE,
                        action: 11,
                        cards: remaining,
                        skill: engine_api::SkillId::new(payload[3]),
                    },
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown DC SB Zhangxiu choice".into(),
        )),
    }
}

pub(crate) fn dcsbyingmou_sequence(
    instance: &crate::state::SkillInstance,
) -> Option<DcsbyingmouSequence> {
    (instance.state_blob.len() == 17
        && instance.state_blob.get(..4) == Some(DCSBYINGMOU_STATE_TAG.as_slice()))
    .then(|| {
        let flags = instance.state_blob[16];
        DcsbyingmouSequence {
            target: PlayerId::new(u32::from_le_bytes(
                instance.state_blob[4..8].try_into().unwrap(),
            )),
            source: PlayerId::new(u32::from_le_bytes(
                instance.state_blob[8..12].try_into().unwrap(),
            )),
            card: CardId::new(u32::from_le_bytes(
                instance.state_blob[12..16].try_into().unwrap(),
            )),
            any_damage: flags & 1 != 0,
            had_damage_cards: flags & 2 != 0,
            draw_branch: flags & 4 != 0,
        }
    })
}

pub(crate) fn set_dcsbyingmou_sequence(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    sequence: DcsbyingmouSequence,
) -> Result<(), EffectError> {
    let instance = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .ok_or_else(|| EffectError::Illegal("Yingmou skill disappeared".into()))?;
    instance.state_blob = DCSBYINGMOU_STATE_TAG.to_vec();
    instance
        .state_blob
        .extend_from_slice(&sequence.target.raw().to_le_bytes());
    instance
        .state_blob
        .extend_from_slice(&sequence.source.raw().to_le_bytes());
    instance
        .state_blob
        .extend_from_slice(&sequence.card.raw().to_le_bytes());
    instance.state_blob.push(
        u8::from(sequence.any_damage)
            | u8::from(sequence.had_damage_cards) << 1
            | u8::from(sequence.draw_branch) << 2,
    );
    Ok(())
}

pub(crate) fn dcsbyingmou_tracks_card(
    instance: &crate::state::SkillInstance,
    card: CardId,
) -> bool {
    card.is_some() && dcsbyingmou_sequence(instance).is_some_and(|state| state.card == card)
}

pub(crate) fn dcsbyingmou_damage_cards(tx: &Transaction, source: PlayerId) -> Vec<CardId> {
    let mut cards: Vec<_> = tx
        .state()
        .hand_cards(source)
        .into_iter()
        .filter(|card| {
            tx.state().cards.get(*card).is_some_and(|card_state| {
                let kind = crate::query::effective_card_kind(tx.state(), source, card_state);
                kind.is_sha() || kind.is_damage_trick()
            })
        })
        .collect();
    cards.sort_by_key(|card| card.raw());
    cards
}

pub(crate) fn finish_dcsbyingmou(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(sequence) = tx.state().skills.get(skill).and_then(dcsbyingmou_sequence) else {
        return Ok(Vec::new());
    };
    tx.state_mut()
        .skills
        .get_mut(skill)
        .unwrap()
        .state_blob
        .clear();
    if sequence.any_damage
        || !tx
            .state()
            .players
            .get(sequence.target)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source: player,
            target: sequence.target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::FIRE,
        }],
    )
}

pub(crate) fn continue_dcsbyingmou_spree(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(mut sequence) = tx.state().skills.get(skill).and_then(dcsbyingmou_sequence) else {
        return Ok(Vec::new());
    };
    if !tx
        .state()
        .players
        .get(sequence.target)
        .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(sequence.source)
            .is_some_and(|participant| participant.alive && !participant.dying)
    {
        return finish_dcsbyingmou(tx, player, skill);
    }
    let legal: Vec<_> = dcsbyingmou_damage_cards(tx, sequence.source)
        .into_iter()
        .filter(|card| {
            crate::command::play::physical_damage_card_can_target_without_distance(
                tx,
                sequence.source,
                *card,
                sequence.target,
            )
        })
        .collect();
    if let Some(card) = (!legal.is_empty()).then(|| {
        let mut rng = RuleRng::from_state(tx.state().rng);
        let card = legal[rng.gen_range_u32(legal.len() as u32) as usize];
        tx.journal(JournalEntry::RngCheckpoint);
        tx.state_mut().rng = rng.into_state();
        card
    }) {
        let kind = crate::query::effective_card_kind(
            tx.state(),
            sequence.source,
            tx.state().cards.get(card).expect("validated Yingmou card"),
        );
        sequence.card = card;
        set_dcsbyingmou_sequence(tx, skill, sequence)?;
        crate::command::play::play_as_kind(
            tx,
            sequence.source,
            card,
            kind,
            &[sequence.target],
            true,
            false,
            false,
            true,
        )?;
        return Ok(Vec::new());
    }
    if !sequence.had_damage_cards {
        let hand = tx.state().hand_cards(sequence.source);
        let owner_hand = tx.state().hand_count(player) as usize;
        let count = hand.len().saturating_sub(owner_hand);
        if count > 0 {
            let exact = u8::try_from(count).map_err(|_| {
                EffectError::ContentFault("Yingmou discard exceeds choice protocol".into())
            })?;
            let mut payload = vec![
                RESOLVE_DC_SB_ZHOUYU_CHOICE,
                12,
                player.raw(),
                sequence.target.raw(),
                sequence.source.raw(),
                skill.raw(),
                count as u32,
                hand.len() as u32,
            ];
            payload.extend(hand.iter().map(|card| card.raw()));
            return open_choice(
                tx,
                sequence.source,
                exact,
                exact,
                false,
                hand.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                payload,
            );
        }
    }
    finish_dcsbyingmou(tx, player, skill)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_dc_sb_zhouyu(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    action: u8,
    event_targets: Vec<PlayerId>,
    damaged_targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if matches!(action, 0 | 1 | 10 | 11) && !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            tx.state_mut()
                .skills
                .get_mut(skill)
                .expect("validated Ronghuo skill")
                .set_content_flag(0, true);
            Ok(Vec::new())
        }
        1 => {
            if source != player || card.is_none() {
                return Ok(Vec::new());
            }
            let groups = crate::query::alive_group_count(tx.state()).max(0) as u32;
            crate::command::play::increase_saved_card_base_damage(tx, player, card, groups)?;
            let armed = tx
                .state()
                .skills
                .get(skill)
                .is_some_and(|instance| instance.content_flag(0) == Some(true));
            if !armed {
                return Ok(Vec::new());
            }
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .set_content_flag(0, false);
            let targets =
                crate::command::play::current_card_target_adjustments(tx, player, card)?.0;
            if targets.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                targets.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DC_SB_ZHOUYU_CHOICE,
                    0,
                    player.raw(),
                    card.raw(),
                    skill.raw(),
                ],
            )
        }
        10 => {
            let instance = tx.state_mut().skills.get_mut(skill).unwrap();
            instance.view_as_branch = 1 - instance.view_as_branch.min(1);
            Ok(Vec::new())
        }
        11 => {
            if source != player || card.is_none() {
                return Ok(Vec::new());
            }
            let yin = tx
                .state()
                .skills
                .get(skill)
                .is_some_and(|instance| instance.view_as_branch.min(1) == 1);
            let max_hand = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(|candidate| tx.state().hand_count(candidate))
                .max()
                .unwrap_or(0);
            let mut targets: Vec<_> = event_targets
                .into_iter()
                .filter(|target| {
                    *target != player
                        && tx
                            .state()
                            .players
                            .get(*target)
                            .is_some_and(|participant| participant.alive)
                })
                .filter(|target| {
                    !yin || tx
                        .state()
                        .players
                        .alive_by_seat()
                        .into_iter()
                        .any(|candidate| {
                            candidate != *target && tx.state().hand_count(candidate) == max_hand
                        })
                })
                .collect();
            targets.sort_by_key(|target| target.raw());
            targets.dedup();
            if targets.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                RESOLVE_DC_SB_ZHOUYU_CHOICE,
                10,
                player.raw(),
                source.raw(),
                card.raw(),
                skill.raw(),
                u32::from(yin),
                targets.len() as u32,
            ];
            payload.extend(targets.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                targets.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )
        }
        12 => {
            let Some(target) = event_targets.first().copied() else {
                return Err(EffectError::ContentFault(
                    "Yingmou draw target missing".into(),
                ));
            };
            let instance = tx
                .state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::Illegal("Yingmou skill disappeared".into()))?;
            if instance.view_as_branch.min(1) == 1 {
                return Err(EffectError::Illegal("Yingmou branch changed".into()));
            }
            instance.view_as_branch = 1;
            let count = tx
                .state()
                .hand_count(target)
                .saturating_sub(tx.state().hand_count(player))
                .min(5);
            let mut effects = Vec::new();
            if count > 0 {
                effects.push(EffectRequest::DrawCards { player, count });
            }
            effects.push(EffectRequest::ResolveDcSbZhouyu {
                player,
                source: player,
                card: CardId::NONE,
                action: 14,
                event_targets: vec![target],
                damaged_targets: Vec::new(),
                skill,
            });
            run_effects(tx, effects)
        }
        13 => {
            let Some(target) = event_targets.first().copied() else {
                return Err(EffectError::ContentFault(
                    "Yingmou spree target missing".into(),
                ));
            };
            let instance = tx
                .state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::Illegal("Yingmou skill disappeared".into()))?;
            if instance.view_as_branch.min(1) != 1 {
                return Err(EffectError::Illegal("Yingmou branch changed".into()));
            }
            instance.view_as_branch = 0;
            let had_damage_cards = !dcsbyingmou_damage_cards(tx, source).is_empty();
            set_dcsbyingmou_sequence(
                tx,
                skill,
                DcsbyingmouSequence {
                    target,
                    source,
                    card: CardId::NONE,
                    any_damage: false,
                    had_damage_cards,
                    draw_branch: false,
                },
            )?;
            continue_dcsbyingmou_spree(tx, player, skill)
        }
        14 => {
            let Some(target) = event_targets.first().copied() else {
                return Err(EffectError::ContentFault(
                    "Yingmou Huogong target missing".into(),
                ));
            };
            if crate::command::play::validate_source_free_card_on_target(
                tx,
                player,
                CardKind::TrickHuogong,
                target,
            )
            .is_err()
            {
                set_dcsbyingmou_sequence(
                    tx,
                    skill,
                    DcsbyingmouSequence {
                        target,
                        source: player,
                        card: CardId::NONE,
                        any_damage: false,
                        had_damage_cards: false,
                        draw_branch: true,
                    },
                )?;
                return finish_dcsbyingmou(tx, player, skill);
            }
            let card = materialize_virtual_card(tx, player, CardKind::TrickHuogong)?;
            set_dcsbyingmou_sequence(
                tx,
                skill,
                DcsbyingmouSequence {
                    target,
                    source: player,
                    card,
                    any_damage: false,
                    had_damage_cards: false,
                    draw_branch: true,
                },
            )?;
            crate::command::play::play_as_kind(
                tx,
                player,
                card,
                CardKind::TrickHuogong,
                &[target],
                true,
                false,
                false,
                false,
            )?;
            Ok(Vec::new())
        }
        20 => {
            let Some(mut sequence) = tx.state().skills.get(skill).and_then(dcsbyingmou_sequence)
            else {
                return Ok(Vec::new());
            };
            if sequence.card != card || sequence.source != source {
                return Ok(Vec::new());
            }
            sequence.any_damage |= damaged_targets.contains(&sequence.target);
            sequence.card = CardId::NONE;
            set_dcsbyingmou_sequence(tx, skill, sequence)?;
            if sequence.draw_branch {
                finish_dcsbyingmou(tx, player, skill)
            } else {
                continue_dcsbyingmou_spree(tx, player, skill)
            }
        }
        22 => finish_dcsbyingmou(tx, player, skill),
        _ => Err(EffectError::ContentFault(
            "unknown DC SB Zhouyu action".into(),
        )),
    }
}

pub(crate) fn resolve_dc_sb_zhouyu_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&RESOLVE_DC_SB_ZHOUYU_CHOICE) || payload.len() < 5 {
        return Err(EffectError::EngineFault(
            "truncated DC SB Zhouyu choice".into(),
        ));
    }
    match payload[1] {
        0 => {
            let player = PlayerId::new(payload[2]);
            let card = CardId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            if observer != player {
                return Err(EffectError::Illegal("Ronghuo chooser mismatch".into()));
            }
            if cancelled || selections.is_empty() {
                return Ok(());
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "Ronghuo needs at most one target".into(),
                ));
            }
            let target = PlayerId::new(selections[0]);
            if !crate::command::play::current_card_target_adjustments(tx, player, card)?
                .0
                .contains(&target)
            {
                return Err(EffectError::Illegal("Ronghuo target changed".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::CommitAddSkillBoundCardTarget {
                    player,
                    card,
                    target,
                    skill,
                }],
            )
            .map(|_| ())
        }
        10 => {
            if payload.len() < 8 {
                return Err(EffectError::EngineFault(
                    "truncated Yingmou target choice".into(),
                ));
            }
            let player = PlayerId::new(payload[2]);
            if observer != player || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("invalid Yingmou target choice".into()));
            }
            let count = payload[7] as usize;
            if payload.len() != 8 + count || !payload[8..].contains(&selections[0]) {
                return Err(EffectError::Illegal("stale Yingmou target choice".into()));
            }
            let target = PlayerId::new(selections[0]);
            let skill = engine_api::SkillId::new(payload[5]);
            let yin = payload[6] != 0;
            let current_yin = tx
                .state()
                .skills
                .get(skill)
                .is_some_and(|instance| instance.view_as_branch.min(1) == 1);
            if yin != current_yin {
                return Err(EffectError::Illegal("Yingmou branch changed".into()));
            }
            if !yin {
                return run_effects(
                    tx,
                    vec![EffectRequest::ResolveDcSbZhouyu {
                        player,
                        source: player,
                        card: CardId::NONE,
                        action: 12,
                        event_targets: vec![target],
                        damaged_targets: Vec::new(),
                        skill,
                    }],
                )
                .map(|_| ());
            }
            let max_hand = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(|candidate| tx.state().hand_count(candidate))
                .max()
                .unwrap_or(0);
            let sources: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|source| *source != target && tx.state().hand_count(*source) == max_hand)
                .collect();
            if sources.is_empty() {
                return Err(EffectError::Illegal(
                    "Yingmou max-hand source changed".into(),
                ));
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                sources.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DC_SB_ZHOUYU_CHOICE,
                    11,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )
            .map(|_| ())
        }
        11 => {
            let player = PlayerId::new(payload[2]);
            let target = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            if observer != player || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("invalid Yingmou source choice".into()));
            }
            let source = PlayerId::new(selections[0]);
            let max_hand = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(|candidate| tx.state().hand_count(candidate))
                .max()
                .unwrap_or(0);
            if source == target
                || !tx
                    .state()
                    .players
                    .get(source)
                    .is_some_and(|participant| participant.alive)
                || tx.state().hand_count(source) != max_hand
            {
                return Err(EffectError::Illegal("Yingmou source changed".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDcSbZhouyu {
                    player,
                    source,
                    card: CardId::NONE,
                    action: 13,
                    event_targets: vec![target],
                    damaged_targets: Vec::new(),
                    skill,
                }],
            )
            .map(|_| ())
        }
        12 => {
            if payload.len() < 8 {
                return Err(EffectError::EngineFault("truncated Yingmou discard".into()));
            }
            let player = PlayerId::new(payload[2]);
            let source = PlayerId::new(payload[4]);
            let skill = engine_api::SkillId::new(payload[5]);
            let expected = payload[6] as usize;
            let count = payload[7] as usize;
            if observer != source
                || cancelled
                || selections.len() != expected
                || payload.len() != 8 + count
                || selections.iter().any(|card| !payload[8..].contains(card))
            {
                return Err(EffectError::Illegal("invalid Yingmou discard".into()));
            }
            let mut selected = selections.to_vec();
            selected.sort_unstable();
            selected.dedup();
            if selected.len() != selections.len() {
                return Err(EffectError::Illegal("duplicate Yingmou discard".into()));
            }
            let cards: Vec<_> = selected.into_iter().map(CardId::new).collect();
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, source)
                .ok_or_else(|| EffectError::Illegal("Yingmou source hand missing".into()))?;
            if cards.iter().any(|card| {
                !tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|card_state| card_state.zone == hand && card_state.owner == source)
            }) {
                return Err(EffectError::Illegal("Yingmou discard cards moved".into()));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::DiscardCards {
                        player: source,
                        cards,
                    },
                    EffectRequest::ResolveDcSbZhouyu {
                        player,
                        source,
                        card: CardId::NONE,
                        action: 22,
                        event_targets: Vec::new(),
                        damaged_targets: Vec::new(),
                        skill,
                    },
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown DC SB Zhouyu choice".into(),
        )),
    }
}

pub(crate) fn dcsbmengmou_sequence(
    instance: &crate::state::SkillInstance,
) -> Option<DcsbmengmouSequence> {
    (instance.state_blob.len() == 17
        && instance.state_blob.get(..4) == Some(DCSBMENGMOU_STATE_TAG.as_slice()))
    .then(|| DcsbmengmouSequence {
        target: PlayerId::new(u32::from_le_bytes(
            instance.state_blob[4..8].try_into().unwrap(),
        )),
        card: CardId::new(u32::from_le_bytes(
            instance.state_blob[8..12].try_into().unwrap(),
        )),
        remaining: u32::from_le_bytes(instance.state_blob[12..16].try_into().unwrap()),
        yin: instance.state_blob[16] != 0,
    })
}

pub(crate) fn set_dcsbmengmou_sequence(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    sequence: Option<DcsbmengmouSequence>,
) -> Result<(), EffectError> {
    let instance = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .ok_or_else(|| EffectError::Illegal("Mengmou skill disappeared".into()))?;
    instance.state_blob.clear();
    if let Some(sequence) = sequence {
        instance
            .state_blob
            .extend_from_slice(&DCSBMENGMOU_STATE_TAG);
        instance
            .state_blob
            .extend_from_slice(&sequence.target.raw().to_le_bytes());
        instance
            .state_blob
            .extend_from_slice(&sequence.card.raw().to_le_bytes());
        instance
            .state_blob
            .extend_from_slice(&sequence.remaining.to_le_bytes());
        instance.state_blob.push(u8::from(sequence.yin));
    }
    Ok(())
}

pub(crate) fn dcsbmengmou_tracks_sha(
    instance: &crate::state::SkillInstance,
    source: PlayerId,
    card: CardId,
) -> bool {
    card.is_some()
        && dcsbmengmou_sequence(instance).is_some_and(|sequence| {
            !sequence.yin && sequence.target == source && sequence.card == card
        })
}

pub(crate) fn dcsbmengmou_transfer_counterpart(
    tx: &Transaction,
    player: PlayerId,
    source: PlayerId,
    cards: &[CardId],
) -> Option<PlayerId> {
    let destination_owner = cards
        .iter()
        .filter_map(|card| tx.state().cards.get(*card).map(|card| card.owner))
        .find(|owner| owner.is_some())?;
    let counterpart = if source == player {
        destination_owner
    } else if destination_owner == player {
        source
    } else {
        PlayerId::NONE
    };
    (counterpart.is_some() && counterpart != player).then_some(counterpart)
}

pub(crate) fn finish_dcsbmengmou_yin(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(sequence) = tx.state().skills.get(skill).and_then(dcsbmengmou_sequence) else {
        return Ok(Vec::new());
    };
    set_dcsbmengmou_sequence(tx, skill, None)?;
    if sequence.remaining == 0
        || !tx
            .state()
            .players
            .get(sequence.target)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: sequence.target,
            amount: sequence.remaining,
            is_lose_hp: true,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
}

pub(crate) fn continue_dcsbmengmou(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(sequence) = tx.state().skills.get(skill).and_then(dcsbmengmou_sequence) else {
        return Ok(Vec::new());
    };
    if sequence.remaining == 0 {
        set_dcsbmengmou_sequence(tx, skill, None)?;
        return Ok(Vec::new());
    }
    if !tx
        .state()
        .players
        .get(sequence.target)
        .is_some_and(|participant| participant.alive && !participant.dying)
    {
        set_dcsbmengmou_sequence(tx, skill, None)?;
        return Ok(Vec::new());
    }
    if sequence.yin {
        let options = crate::command::play::response_sha_tokens(tx.state(), sequence.target);
        if options.is_empty() {
            return finish_dcsbmengmou_yin(tx, player, skill);
        }
        return open_choice(
            tx,
            sequence.target,
            0,
            1,
            true,
            options,
            ChoiceKind::SelectCards,
            ContinuationKind::Custom,
            vec![
                RESOLVE_DC_SB_LUSU_CHOICE,
                12,
                player.raw(),
                sequence.target.raw(),
                skill.raw(),
            ],
        );
    }
    let cards: Vec<_> =
        crate::command::play::prompted_sha_source_cards(tx.state(), sequence.target)
            .into_iter()
            .filter(|card| {
                !crate::command::play::prompted_sha_targets_normal(tx, sequence.target, *card)
                    .is_empty()
            })
            .collect();
    if cards.is_empty() {
        set_dcsbmengmou_sequence(tx, skill, None)?;
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        sequence.target,
        0,
        1,
        true,
        cards.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        vec![
            RESOLVE_DC_SB_LUSU_CHOICE,
            10,
            player.raw(),
            sequence.target.raw(),
            skill.raw(),
        ],
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_dc_sb_lusu(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    action: u8,
    cards: Vec<CardId>,
    amount: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if matches!(action, 0 | 10 | 11) && !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => run_effects(
            tx,
            vec![
                EffectRequest::DrawCards { player, count: 2 },
                EffectRequest::ResolveDcSbLusu {
                    player,
                    source: player,
                    card: CardId::NONE,
                    action: 1,
                    cards: Vec::new(),
                    amount: 0,
                    skill,
                },
            ],
        ),
        1 => {
            let hand = tx.state().hand_cards(player);
            if hand.len() < 3
                || !tx
                    .state()
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .any(|target| target != player)
            {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                3,
                3,
                false,
                hand.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![RESOLVE_DC_SB_LUSU_CHOICE, 0, player.raw(), skill.raw()],
            )
        }
        2 => {
            let target = source;
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            let valid: Vec<_> = cards
                .into_iter()
                .filter(|card| {
                    tx.state().cards.get(*card).is_some_and(|card_state| {
                        Some(card_state.zone) == hand && card_state.owner == player
                    })
                })
                .collect();
            if valid.is_empty()
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive && target != player)
            {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                RESOLVE_DC_SB_LUSU_CHOICE,
                2,
                player.raw(),
                target.raw(),
                skill.raw(),
                valid.len() as u32,
            ];
            payload.extend(valid.iter().map(|card| card.raw()));
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                valid.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                payload,
            )
        }
        10 => {
            let instance = tx.state_mut().skills.get_mut(skill).unwrap();
            instance.view_as_branch = 1 - instance.view_as_branch.min(1);
            Ok(Vec::new())
        }
        11 => {
            let Some(target) = dcsbmengmou_transfer_counterpart(tx, player, source, &cards) else {
                return Ok(Vec::new());
            };
            let branch = tx
                .state()
                .skills
                .get(skill)
                .map_or(0, |instance| instance.view_as_branch.min(1));
            let key = turn_flag::dcsbmengmou_used_by_player(player);
            let used = tx.state().turn.flag(key).max(0) as u32;
            let bit = 1u32 << branch;
            if used & bit != 0 {
                return Ok(Vec::new());
            }
            tx.state_mut().turn.set_flag(key, (used | bit) as i32);
            tx.state_mut().skills.get_mut(skill).unwrap().view_as_branch = 1 - branch;
            let remaining = tx
                .state()
                .players
                .get(player)
                .map_or(0, |participant| participant.max_hp);
            set_dcsbmengmou_sequence(
                tx,
                skill,
                Some(DcsbmengmouSequence {
                    target,
                    card: CardId::NONE,
                    remaining,
                    yin: branch == 1,
                }),
            )?;
            continue_dcsbmengmou(tx, player, skill)
        }
        20 => {
            let tracked = tx
                .state()
                .skills
                .get(skill)
                .is_some_and(|instance| dcsbmengmou_tracks_sha(instance, source, card));
            if !tracked || amount == 0 {
                return Ok(Vec::new());
            }
            run_effects(
                tx,
                vec![EffectRequest::RecoverHp {
                    source,
                    player: source,
                    amount,
                }],
            )
        }
        21 => {
            let Some(mut sequence) = tx.state().skills.get(skill).and_then(dcsbmengmou_sequence)
            else {
                return Ok(Vec::new());
            };
            if sequence.yin || sequence.target != source || sequence.card != card {
                return Ok(Vec::new());
            }
            sequence.card = CardId::NONE;
            set_dcsbmengmou_sequence(tx, skill, Some(sequence))?;
            continue_dcsbmengmou(tx, player, skill)
        }
        22 => {
            let Some(mut sequence) = tx.state().skills.get(skill).and_then(dcsbmengmou_sequence)
            else {
                return Ok(Vec::new());
            };
            if !sequence.yin || sequence.target != source {
                return Ok(Vec::new());
            }
            sequence.remaining = sequence.remaining.saturating_sub(1);
            set_dcsbmengmou_sequence(tx, skill, Some(sequence))?;
            continue_dcsbmengmou(tx, player, skill)
        }
        23 => finish_dcsbmengmou_yin(tx, player, skill),
        _ => Err(EffectError::ContentFault(
            "unknown DC SB Lusu action".into(),
        )),
    }
}
