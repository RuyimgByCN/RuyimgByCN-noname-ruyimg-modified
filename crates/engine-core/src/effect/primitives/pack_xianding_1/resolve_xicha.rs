//! Effect primitives for `resolve_zhengbi` and related helpers.

use super::*;

/// 傅干「析察」（ABI 586）。每个来源只有一份“下一次用牌”监听；同来源重复发动会合并实体牌。
pub(crate) fn resolve_xicha(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    action: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill) || source.is_none() {
                return Ok(Vec::new());
            }
            let hand = tx.state().hand_cards(source);
            if hand.is_empty() {
                return Ok(Vec::new());
            }
            let count = hand.len().min(2) as u8;
            open_choice(
                tx,
                player,
                count,
                count,
                false,
                hand.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_XICHA_CHOICE,
                    player.raw(),
                    source.raw(),
                    skill.raw(),
                ],
            )
        }
        1 => {
            let mut entries = tx
                .state()
                .skills
                .get(skill)
                .and_then(|instance| xicha_entries(&instance.state_blob))
                .unwrap_or_default();
            let Some(index) = entries.iter().position(|(saved, _)| *saved == source) else {
                return Ok(Vec::new());
            };
            let watched = entries.remove(index).1;
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                write_xicha_entries(instance, &entries);
            }
            if !cards.iter().any(|used| watched.contains(used)) {
                return Ok(Vec::new());
            }
            crate::command::play::cancel_saved_card_use(tx, source, card)?;
            let movable: Vec<_> = cards
                .into_iter()
                .filter(|used| {
                    tx.state()
                        .cards
                        .get(*used)
                        .is_some_and(|state| state.owner != player)
                })
                .collect();
            gain_cards_by_zone(tx, player, &movable)
        }
        2 => {
            let source_hand = tx.state().zones.find(ZoneKind::Hand, source);
            let expected = tx.state().hand_cards(source).len().min(2);
            if expected == 0
                || cards.len() != expected
                || cards.iter().any(|card| {
                    !tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|state| Some(state.zone) == source_hand)
                })
            {
                return Err(EffectError::Illegal("析察选择已失效".into()));
            }
            let instance = tx
                .state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::Illegal("析察技能已消失".into()))?;
            let mut entries = xicha_entries(&instance.state_blob).unwrap_or_default();
            if let Some((_, tracked)) = entries.iter_mut().find(|(saved, _)| *saved == source) {
                tracked.extend(cards);
                tracked.sort_unstable_by_key(|card| card.raw());
                tracked.dedup();
            } else {
                entries.push((source, cards));
                entries.sort_by_key(|(source, _)| source.raw());
            }
            write_xicha_entries(instance, &entries);
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault("bad Xicha action".into())),
    }
}

/// 胡遵「斩涛 / 安境」（ABI 587）。
#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_huzun(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    action: u8,
    cards: Vec<CardId>,
    targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx
                    .state()
                    .players
                    .get(player)
                    .is_some_and(|owner| owner.alive)
            {
                return Ok(Vec::new());
            }
            if crate::query::physical_card_number_sum(tx.state(), card, &cards).is_none() {
                return Ok(Vec::new());
            }
            let result = crate::command::judge::flip_judgment(
                tx,
                player,
                crate::command::judge::JudgmentFavor::Neutral,
            )?;
            let mut continuation_cards = Vec::with_capacity(cards.len() + 1);
            continuation_cards.push(result.card);
            continuation_cards.extend(cards);
            run_effects(
                tx,
                vec![
                    EffectRequest::EmitJudgmentResolved {
                        player,
                        result_card: result.card,
                        suit: result.suit,
                        rank: result.rank,
                        cause_card: CardId::NONE,
                    },
                    EffectRequest::ResolveHuzun {
                        player,
                        source,
                        card,
                        action: 3,
                        cards: continuation_cards,
                        targets: Vec::new(),
                        skill,
                    },
                ],
            )?;
            Ok(Vec::new())
        }
        1 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            let mut options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| {
                    tx.state()
                        .players
                        .get(*target)
                        .is_some_and(|target| target.hp < target.max_hp as i32)
                })
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let count = tx
                .state()
                .skills
                .get(skill)
                .and_then(|instance| {
                    (instance.state_blob.len() == 8 && &instance.state_blob[..4] == b"ANJ1")
                        .then(|| u32::from_le_bytes(instance.state_blob[4..8].try_into().unwrap()))
                })
                .unwrap_or(0)
                .saturating_add(1);
            let mut blob = b"ANJ1".to_vec();
            blob.extend_from_slice(&count.to_le_bytes());
            tx.state_mut().skills.get_mut(skill).unwrap().state_blob = blob;
            let max = options.len().min(count as usize).min(usize::from(u8::MAX)) as u8;
            let mut payload = vec![
                RESOLVE_HUZUN_CHOICE,
                player.raw(),
                skill.raw(),
                u32::from(max),
                options.len() as u32,
            ];
            payload.extend(options.drain(..).map(PlayerId::raw));
            open_choice(
                tx,
                player,
                1,
                max,
                false,
                payload[5..].to_vec(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )
        }
        2 => {
            let alive = tx.state().players.alive_by_seat();
            let current = tx.state().turn.current;
            let mut ordered = targets;
            ordered.sort_by_key(|target| {
                crate::trigger::seat_rel_to_current(&alive, current, *target)
            });
            let mut effects: Vec<_> = ordered
                .iter()
                .map(|target| EffectRequest::DrawCards {
                    player: *target,
                    count: 1,
                })
                .collect();
            effects.push(EffectRequest::ResolveHuzun {
                player,
                source: PlayerId::NONE,
                card: CardId::NONE,
                action: 4,
                cards: Vec::new(),
                targets: ordered,
                skill,
            });
            run_effects(tx, effects)
        }
        3 => {
            let Some((&judgment, materials)) = cards.split_first() else {
                return Err(EffectError::EngineFault("斩涛缺少判定牌".into()));
            };
            let judgment_rank = tx
                .state()
                .cards
                .get(judgment)
                .map(|card| card.rank)
                .unwrap_or(0);
            let threshold = crate::query::physical_card_number_sum(tx.state(), card, materials);
            if threshold.is_none_or(|threshold| u32::from(judgment_rank) <= threshold)
                || !tx
                    .state()
                    .players
                    .get(source)
                    .is_some_and(|target| target.alive)
            {
                return Ok(Vec::new());
            }
            let virtual_sha = allocate_virtual_sha(tx)?;
            if crate::command::play::validate_sha_target_with_options(
                tx,
                player,
                virtual_sha,
                source,
                false,
            )
            .is_err()
            {
                remove_unused_virtual_card(tx, virtual_sha)?;
                return Ok(Vec::new());
            }
            crate::command::play::handle_virtual_sha_with_policies_and_bonus(
                tx,
                player,
                virtual_sha,
                &[source],
                false,
                true,
                0,
            )?;
            Ok(Vec::new())
        }
        4 => {
            let min_hp = targets
                .iter()
                .filter_map(|target| {
                    tx.state()
                        .players
                        .get(*target)
                        .filter(|target| target.alive)
                        .map(|target| target.hp)
                })
                .min();
            let Some(min_hp) = min_hp else {
                return Ok(Vec::new());
            };
            let candidates: Vec<_> = targets
                .into_iter()
                .filter(|target| {
                    tx.state()
                        .players
                        .get(*target)
                        .is_some_and(|target| target.alive && target.hp == min_hp)
                })
                .collect();
            let mut rng = RuleRng::from_state(tx.state().rng);
            let target = candidates[rng.gen_range_u32(candidates.len() as u32) as usize];
            tx.journal(JournalEntry::RngCheckpoint);
            tx.state_mut().rng = rng.into_state();
            run_effects(
                tx,
                vec![EffectRequest::RecoverHp {
                    source: player,
                    player: target,
                    amount: 1,
                }],
            )
        }
        _ => Err(EffectError::ContentFault("bad Huzun action".into())),
    }
}

pub(crate) fn give_selected_then_target_may_use_draw_owner(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || player == target
                || ![player, target].into_iter().all(|participant| {
                    tx.state()
                        .players
                        .get(participant)
                        .is_some_and(|state| state.alive)
                })
            {
                return Err(EffectError::Illegal("invalid Yuanlve context".into()));
            }
            let from = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("Yuanlve owner hand missing".into()))?;
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("Yuanlve target hand missing".into()))?;
            if !tx.state().cards.get(card).is_some_and(|state| {
                state.owner == player
                    && state.zone == from
                    && !crate::query::effective_card_kind(tx.state(), player, state).is_equip()
            }) {
                return Err(EffectError::Illegal(
                    "Yuanlve requires one non-equipment hand card".into(),
                ));
            }
            let events = give_cards_as(tx, player, vec![card], to, from)?;
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::GiveSelectedThenTargetMayUseDrawOwner {
                    player,
                    target,
                    card,
                    targets: Vec::new(),
                    stage: 1,
                    skill,
                },
            );
            Ok(events)
        }
        1 => {
            let hand = tx.state().zones.find(ZoneKind::Hand, target);
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|state| state.alive)
                || !tx
                    .state()
                    .cards
                    .get(card)
                    .is_some_and(|state| state.owner == target && Some(state.zone) == hand)
                || crate::command::play::skill_card_target_spec(tx, target, card).is_err()
            {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                target,
                1,
                1,
                true,
                vec![card.raw()],
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    YUANLVE_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    target.raw(),
                    card.raw(),
                    skill.raw(),
                ],
            )
        }
        2 => {
            if tx
                .state()
                .players
                .get(player)
                .is_some_and(|state| state.alive)
            {
                tx.state_mut()
                    .resolution
                    .deferred_tail_effects
                    .push(EffectRequest::DrawCards { player, count: 1 });
            }
            crate::command::play::handle_skill_card(tx, target, card, &targets)?;
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault("bad Yuanlve stage".into())),
    }
}

pub(crate) fn choose_gain_target_hand_ceil_half_difference(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
    ready_to_choose: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || player == target
        || ![player, target].into_iter().all(|participant| {
            tx.state()
                .players
                .get(participant)
                .is_some_and(|state| state.alive)
        })
    {
        return Err(EffectError::Illegal("invalid Yangwu context".into()));
    }
    if !ready_to_choose {
        let hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::Illegal("Yangwu owner hand missing".into()))?;
        if !tx.state().cards.get(card).is_some_and(|state| {
            state.owner == player
                && state.zone == hand
                && crate::query::effective_card_suit(tx.state(), player, state)
                    == engine_api::Suit::Heart
        }) {
            return Err(EffectError::Illegal(
                "Yangwu cost must be one heart hand card".into(),
            ));
        }
        return super::super::run_effects(
            tx,
            vec![
                EffectRequest::DiscardCards {
                    player,
                    cards: vec![card],
                },
                EffectRequest::ChooseGainTargetHandCeilHalfDifference {
                    player,
                    target,
                    card,
                    skill,
                    ready_to_choose: true,
                },
            ],
        );
    }
    let difference = tx
        .state()
        .hand_count(target)
        .saturating_sub(tx.state().hand_count(player));
    let count = difference.div_ceil(2);
    if count == 0 {
        return Ok(Vec::new());
    }
    let options = tx.state().hand_cards(target);
    if options.len() < count as usize {
        return Err(EffectError::Illegal("Yangwu target hand changed".into()));
    }
    open_choice(
        tx,
        player,
        u8::try_from(count).unwrap_or(u8::MAX),
        u8::try_from(count).unwrap_or(u8::MAX),
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        vec![
            YANGWU_GAIN_HAND_CUSTOM_CHOICE,
            player.raw(),
            target.raw(),
            skill.raw(),
            count,
        ],
    )
}

pub(crate) fn resolve_refuhai(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
    {
        return Ok(Vec::new());
    }
    let mut participants = tx.state().players.alive_by_seat();
    let Some(owner_index) = participants.iter().position(|target| *target == player) else {
        return Ok(Vec::new());
    };
    participants.rotate_left(owner_index + 1);
    participants.retain(|target| *target != player);
    let Some(observer) = participants.first().copied() else {
        return Ok(Vec::new());
    };
    let mut payload = vec![
        REFUHAI_CUSTOM_CHOICE,
        player.raw(),
        skill.raw(),
        0,
        participants.len() as u32,
    ];
    payload.extend(participants.iter().map(|target| target.raw()));
    open_choice(
        tx,
        observer,
        1,
        1,
        false,
        vec![1, 2],
        ChoiceKind::Custom,
        ContinuationKind::Custom,
        payload,
    )
}

pub(crate) fn choose_other_random_gain_region_then_draw_compare_color_lose_hp(
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
        return Ok(Vec::new());
    }
    let options: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| {
            *target != player
                && [ZoneKind::Hand, ZoneKind::Equip, ZoneKind::Judge]
                    .into_iter()
                    .any(|kind| {
                        tx.state()
                            .zones
                            .find(kind, *target)
                            .and_then(|zone| tx.state().zones.get(zone))
                            .is_some_and(|zone| !zone.cards.is_empty())
                    })
        })
        .map(PlayerId::raw)
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
        ChoiceKind::SelectTargets,
        ContinuationKind::Custom,
        vec![
            RANDOM_GAIN_REGION_COMPARE_COLOR_CUSTOM_CHOICE,
            player.raw(),
            skill.raw(),
        ],
    )
}

pub(crate) fn reveal_choose_category_distribute_self_threshold_hp_loss(
    tx: &mut Transaction,
    player: PlayerId,
    reveal_count: u8,
    recipient_cap: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if reveal_count == 0 || recipient_cap == 0 || !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("category distribution draw pile missing".into()))?;
    while tx
        .state()
        .zones
        .get(draw)
        .map_or(0, |zone| zone.cards.len())
        < usize::from(reveal_count)
    {
        let before = tx
            .state()
            .zones
            .get(draw)
            .map_or(0, |zone| zone.cards.len());
        reshuffle_discard_into_draw(tx)?;
        if tx
            .state()
            .zones
            .get(draw)
            .map_or(0, |zone| zone.cards.len())
            == before
        {
            break;
        }
    }
    let cards: Vec<_> = tx
        .state()
        .zones
        .get(draw)
        .map(|zone| {
            zone.cards
                .iter()
                .rev()
                .take(usize::from(reveal_count))
                .copied()
                .collect()
        })
        .unwrap_or_default();
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| {
            EffectError::Illegal("category distribution processing zone missing".into())
        })?;
    let mut categories: Vec<_> = cards
        .iter()
        .map(|card| effective_card_category(tx.state(), player, *card))
        .filter(|category| *category != 0)
        .collect();
    categories.sort_unstable();
    categories.dedup();
    run_effects(
        tx,
        vec![
            EffectRequest::MoveCards {
                cards: cards.clone(),
                to: processing,
                from: Some(draw),
            },
            EffectRequest::RevealCards {
                cards: cards.clone(),
                to: Vec::new(),
            },
        ],
    )?;
    if categories.is_empty() {
        let discard = tx
            .state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::Illegal("category distribution discard missing".into()))?;
        return run_effects(
            tx,
            vec![EffectRequest::MoveCards {
                cards,
                to: discard,
                from: Some(processing),
            }],
        );
    }
    let mut payload = vec![
        REVEAL_CATEGORY_DISTRIBUTION_CUSTOM_CHOICE,
        player.raw(),
        skill.raw(),
        u32::from(recipient_cap),
        0,
        cards.len() as u32,
    ];
    payload.extend(cards.iter().map(|card| card.raw()));
    payload.extend([0, 0]); // no recipient counts, no pending cards
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        categories.into_iter().map(u32::from).collect(),
        ChoiceKind::Custom,
        ContinuationKind::Custom,
        payload,
    )
}
