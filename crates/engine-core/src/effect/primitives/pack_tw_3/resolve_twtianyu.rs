//! Effect primitives for `resolve_twtianyu` and related helpers.

use super::*;

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_twtianyu(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    helper_skill_def: u32,
    action: u8,
    stage: u8,
    card: CardId,
    branch: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    if action == 1 {
        return if crate::query::all_other_players_in_attack_range(tx.state(), player) {
            gain_card_by_kind(tx, player, CardKind::Sha.tag())
        } else {
            run_effects(
                tx,
                vec![
                    EffectRequest::AddSkill {
                        player,
                        skill_def: engine_api::SkillDefId::new(helper_skill_def),
                    },
                    EffectRequest::AddMark {
                        player,
                        mark: engine_api::MarkDefId::new(helper_skill_def),
                        delta: 1,
                    },
                ],
            )
        };
    }
    match stage {
        0 => {
            let can_discard = !crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::Hand,
            )
            .is_empty();
            let can_move = !crate::query::movable_field_cards_from(tx.state(), target).is_empty();
            let target_hp = tx.state().players.get(target).map_or(i32::MIN, |p| p.hp);
            let max_hp = tx
                .state()
                .players
                .iter_live()
                .filter(|p| p.alive)
                .map(|p| p.hp)
                .max()
                .unwrap_or(i32::MIN);
            let owner_hp = tx.state().players.get(player).map_or(i32::MAX, |p| p.hp);
            let mut choices = Vec::new();
            if can_discard {
                choices.push(1);
            }
            if can_move {
                choices.push(2);
            }
            if can_discard && can_move && (target_hp > owner_hp || target_hp == max_hp) {
                choices.push(3);
            }
            if choices.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                choices,
                ChoiceKind::Custom,
                ContinuationKind::ResolveTwtianyu,
                vec![0, player.raw(), target.raw(), helper_skill_def, skill.raw()],
            )
        }
        1 => {
            if branch & 1 == 0 {
                return resolve_twtianyu(
                    tx,
                    player,
                    target,
                    helper_skill_def,
                    0,
                    2,
                    CardId::NONE,
                    branch,
                    skill,
                );
            }
            let cards = crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::Hand,
            );
            let distance = match crate::query::query(
                tx.state(),
                &rule_api::QueryRequest::Distance {
                    from: player,
                    to: target,
                },
                &mut crate::query::QueryContext::default(),
            ) {
                rule_api::QueryResponse::Int(value) => value.max(0) as usize,
                _ => 0,
            };
            let count = distance.min(cards.len());
            if count == 0 {
                return resolve_twtianyu(
                    tx,
                    player,
                    target,
                    helper_skill_def,
                    0,
                    2,
                    CardId::NONE,
                    branch,
                    skill,
                );
            }
            if count == cards.len() {
                if branch & 2 != 0 {
                    tx.state_mut().resolution.deferred_tail_effects.push(
                        EffectRequest::ResolveTwtianyu {
                            player,
                            target,
                            helper_skill_def,
                            action: 0,
                            stage: 2,
                            card: CardId::NONE,
                            branch,
                            skill,
                        },
                    );
                }
                return run_effects(
                    tx,
                    vec![EffectRequest::DiscardCards {
                        player: target,
                        cards,
                    }],
                );
            }
            let mut payload = vec![
                1,
                player.raw(),
                target.raw(),
                helper_skill_def,
                skill.raw(),
                u32::from(branch),
            ];
            payload.extend(cards.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                count as u8,
                count as u8,
                false,
                cards.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwtianyu,
                payload,
            )
        }
        2 => {
            if branch & 2 == 0 {
                return Ok(Vec::new());
            }
            let cards = crate::query::movable_field_cards_from(tx.state(), target);
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
                ContinuationKind::ResolveTwtianyu,
                vec![2, player.raw(), target.raw(), skill.raw()],
            )
        }
        3 => {
            let destinations = crate::query::field_card_move_destinations(tx.state(), card);
            if destinations.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                destinations.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveTwtianyu,
                vec![3, player.raw(), target.raw(), card.raw(), skill.raw()],
            )
        }
        _ => Err(EffectError::ContentFault(
            "unknown TW Tianyu resolver stage".into(),
        )),
    }
}

pub(crate) fn resolve_twwujing(
    tx: &mut Transaction,
    player: PlayerId,
    target_count: u8,
    action: u8,
    event_cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    if action == 0 {
        return choose_optional_targets_draw(
            tx,
            player,
            1,
            1,
            target_count,
            DrawTargetScope::Any,
            skill,
        );
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("shared discard pile missing".into()))?;
    let mut cards: Vec<_> = event_cards
        .into_iter()
        .filter(|card| {
            tx.state().cards.get(*card).is_some_and(|card_state| {
                card_state.zone == discard
                    && crate::query::effective_card_color_mask(tx.state(), player, *card) == 0b01
            })
        })
        .collect();
    cards.sort_unstable_by_key(|card| card.raw());
    cards.dedup();
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        cards.len().min(u8::MAX as usize) as u8,
        true,
        cards.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveTwwujing,
        vec![0, player.raw(), skill.raw()],
    )
}

pub(crate) fn resolve_twshanghe(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    mut remaining: Vec<PlayerId>,
    got_jiu: bool,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|owner| owner.alive)
    {
        return Ok(Vec::new());
    }
    if stage == 0 {
        remaining = tx.state().players.alive_by_seat();
        let Some(owner_index) = remaining.iter().position(|target| *target == player) else {
            return Ok(Vec::new());
        };
        remaining.rotate_left(owner_index + 1);
        remaining.retain(|target| *target != player);
    }
    while let Some(giver) = remaining.first().copied() {
        remaining.remove(0);
        let cards: Vec<_> = [ZoneKind::Hand, ZoneKind::Equip]
            .into_iter()
            .filter_map(|kind| tx.state().zones.find(kind, giver))
            .filter_map(|zone| tx.state().zones.get(zone))
            .flat_map(|zone| zone.cards.iter().copied())
            .collect();
        if cards.is_empty() {
            continue;
        }
        let mut payload = vec![
            player.raw(),
            giver.raw(),
            skill.raw(),
            u32::from(got_jiu),
            remaining.len() as u32,
        ];
        payload.extend(remaining.iter().map(|target| target.raw()));
        return open_choice(
            tx,
            giver,
            1,
            1,
            false,
            cards.into_iter().map(CardId::raw).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::ResolveTwshanghe,
            payload,
        );
    }
    if got_jiu {
        return Ok(Vec::new());
    }
    let amount = tx
        .state()
        .players
        .get(player)
        .map(|owner| (1_i32 - owner.hp).max(0) as u32)
        .unwrap_or(0);
    if amount == 0 {
        Ok(Vec::new())
    } else {
        recover_hp(tx, player, player, amount)
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_twwangchang(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    selected_card: CardId,
    action: u8,
    stage: u8,
    targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|owner| owner.alive)
    {
        return Ok(Vec::new());
    }
    if action == 0 {
        if stage == 0 {
            let options = tx.state().players.alive_by_seat();
            let max = (1 + tx.state().lifetime_dying_players.len())
                .min(options.len())
                .min(u8::MAX as usize) as u8;
            return open_choice(
                tx,
                player,
                1,
                max,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveTwwangchang,
                vec![3, player.raw(), skill.raw()],
            );
        }
        let mut events = Vec::new();
        let mut nonbasic = false;
        for target in targets {
            let drawn = draw_cards(tx, target, 1)?;
            nonbasic |= drawn.iter().any(|event| match event {
                DomainEvent::CardsDrawn { player: who, cards } if *who == target => {
                    cards.iter().any(|card| {
                        tx.state().cards.get(*card).is_some_and(|card_state| {
                            !crate::query::effective_card_kind(tx.state(), target, card_state)
                                .is_basic()
                        })
                    })
                }
                _ => false,
            });
            events.extend(drawn);
        }
        if nonbasic {
            events.extend(draw_cards(tx, player, 1)?);
        }
        return Ok(events);
    }
    match stage {
        0 => {
            let has_region_card = [ZoneKind::Hand, ZoneKind::Equip, ZoneKind::Judge]
                .into_iter()
                .filter_map(|kind| tx.state().zones.find(kind, target))
                .filter_map(|zone| tx.state().zones.get(zone))
                .any(|zone| !zone.cards.is_empty());
            let mut options = vec![0];
            if has_region_card {
                options.push(1);
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::Custom,
                ContinuationKind::ResolveTwwangchang,
                vec![0, player.raw(), target.raw(), card.raw(), skill.raw()],
            )
        }
        2 => {
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveTwwangchang {
                    player,
                    target,
                    card,
                    selected_card: CardId::NONE,
                    action,
                    stage: 3,
                    targets: Vec::new(),
                    skill,
                },
            );
            draw_cards(tx, player, 1)
        }
        3 => {
            if tx.state().hand_count(player) != tx.state().hand_count(target) {
                return Ok(Vec::new());
            }
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.turn_fire_count = instance.turn_fire_count.saturating_sub(1);
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                vec![1],
                ChoiceKind::Custom,
                ContinuationKind::ResolveTwwangchang,
                vec![2, player.raw(), target.raw(), card.raw(), skill.raw()],
            )
        }
        4 => {
            let from = tx
                .state()
                .cards
                .get(selected_card)
                .map(|card| card.zone)
                .ok_or_else(|| EffectError::Illegal("TW Shepan selected card missing".into()))?;
            let valid = [ZoneKind::Hand, ZoneKind::Equip, ZoneKind::Judge]
                .into_iter()
                .any(|kind| tx.state().zones.find(kind, target) == Some(from));
            if !valid {
                return Err(EffectError::Illegal(
                    "TW Shepan selected card left target region".into(),
                ));
            }
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("draw pile missing".into()))?;
            let events = move_cards(tx, vec![selected_card], draw, Some(from))?;
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveTwwangchang {
                    player,
                    target,
                    card,
                    selected_card: CardId::NONE,
                    action,
                    stage: 3,
                    targets: Vec::new(),
                    skill,
                },
            );
            Ok(events)
        }
        _ => Err(EffectError::ContentFault(
            "unknown TW Wangchang resolver stage".into(),
        )),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_twcaozhao(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    source: PlayerId,
    card: CardId,
    selected_skill_def: u32,
    twfuzuan_skill_def: u32,
    twfeifu_skill_def: u32,
    action: u8,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|owner| owner.alive)
    {
        return Ok(Vec::new());
    }
    match (action, stage) {
        (0, 0) => {
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| {
                    !crate::query::conversion_skill_defs(tx.state(), *target).is_empty()
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
                ContinuationKind::ResolveTwcaozhao,
                vec![
                    0,
                    player.raw(),
                    twfuzuan_skill_def,
                    twfeifu_skill_def,
                    skill.raw(),
                ],
            )
        }
        (0, 1) => {
            let Some(instance) = tx
                .state()
                .players
                .get(target)
                .into_iter()
                .flat_map(|owner| owner.skills.iter())
                .copied()
                .find(|instance| {
                    tx.state().skills.get(*instance).is_some_and(|owned| {
                        owned.def.raw() == selected_skill_def
                            && tx
                                .state()
                                .content
                                .skill_metadata(owned.def)
                                .is_some_and(|metadata| metadata.conversion)
                    })
                })
            else {
                return Err(EffectError::Illegal(
                    "TW Fuzuan conversion skill changed".into(),
                ));
            };
            let branch = tx
                .state()
                .skills
                .get(instance)
                .unwrap()
                .view_as_branch
                .min(1);
            tx.state_mut()
                .skills
                .get_mut(instance)
                .unwrap()
                .view_as_branch = 1 - branch;
            Ok(Vec::new())
        }
        (1, 0) => {
            let mut events = run_effects(
                tx,
                tx.state()
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .map(|target| EffectRequest::AddSkill {
                        player: target,
                        skill_def: engine_api::SkillDefId::new(twfeifu_skill_def),
                    })
                    .collect(),
            )?;
            let others: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| *target != player)
                .collect();
            if !others.is_empty() {
                events.extend(open_choice(
                    tx,
                    player,
                    1,
                    1,
                    true,
                    others.into_iter().map(PlayerId::raw).collect(),
                    ChoiceKind::SelectTargets,
                    ContinuationKind::ResolveTwcaozhao,
                    vec![
                        2,
                        player.raw(),
                        twfuzuan_skill_def,
                        twfeifu_skill_def,
                        skill.raw(),
                    ],
                )?);
            }
            Ok(events)
        }
        (2, 0) => {
            let branch = tx.state().skills.get(skill).unwrap().view_as_branch.min(1);
            tx.state_mut().skills.get_mut(skill).unwrap().view_as_branch = 1 - branch;
            let options: Vec<_> = [ZoneKind::Hand, ZoneKind::Equip]
                .into_iter()
                .filter_map(|kind| tx.state().zones.find(kind, target))
                .filter_map(|zone| tx.state().zones.get(zone))
                .flat_map(|zone| zone.cards.iter().copied())
                .collect();
            if options.is_empty() || source.is_none() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwcaozhao,
                vec![
                    3,
                    player.raw(),
                    target.raw(),
                    source.raw(),
                    card.raw(),
                    skill.raw(),
                ],
            )
        }
        (2, 1) => {
            let Some(card_state) = tx.state().cards.get(card) else {
                return Ok(Vec::new());
            };
            let kind = crate::query::effective_card_kind(tx.state(), source, card_state);
            let in_hand = tx.state().zones.find(ZoneKind::Hand, source) == Some(card_state.zone);
            let can_equip = kind
                .equip_slot()
                .and_then(|slot| {
                    tx.state()
                        .players
                        .get(source)
                        .map(|owner| owner.equip_slot_enabled(slot))
                })
                .unwrap_or(false);
            if !in_hand || !kind.is_equip() || !can_equip {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                source,
                1,
                1,
                true,
                vec![1],
                ChoiceKind::Custom,
                ContinuationKind::UseGivenEquipment,
                vec![source.raw(), card.raw()],
            )
        }
        _ => Err(EffectError::ContentFault(
            "unknown TW Caozhao resolver stage".into(),
        )),
    }
}
