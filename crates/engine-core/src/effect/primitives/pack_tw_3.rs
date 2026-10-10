//! Effect primitive implementations — tw 3 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

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

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_twguohuai(
    tx: &mut Transaction,
    player: PlayerId,
    _target: PlayerId,
    source: PlayerId,
    card: CardId,
    mark_def: u32,
    helper_skill_def: u32,
    action: u8,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match (action, stage) {
        (0, 0) => {
            let earned = tx.state().turn.drew_in_play_phase(player)
                || tx.state().turn.damage_dealt_count(player) > 0;
            if !earned && card.is_some() {
                let baseline = tx.state().turn.damage_dealt_count(player);
                tx.state_mut().skills.get_mut(skill).unwrap().state_blob =
                    crate::state::SkillInstance::phase_card_use_policy_state(
                        card,
                        crate::state::SkillInstance::CARD_USE_POLICY_DRAW_ON_DAMAGE,
                        baseline,
                    );
            }
            let mut effects = vec![EffectRequest::DrawCards { player, count: 2 }];
            if earned {
                effects.push(EffectRequest::AddMark {
                    player,
                    mark: engine_api::MarkDefId::new(mark_def),
                    delta: 1,
                });
            }
            run_effects(tx, effects)
        }
        (1, 0) => {
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .clear_phase_card_use_policy_state();
            add_mark(tx, player, mark_def, 1)
        }
        (2, 0) => {
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .clear_phase_card_use_policy_state();
            Ok(Vec::new())
        }
        (3, 0) => {
            let mut options = vec![1];
            if !crate::query::discardable_hand_equip_cards(tx.state(), source).is_empty() {
                options.push(2);
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::Custom,
                ContinuationKind::ResolveTwguohuai,
                vec![
                    0,
                    player.raw(),
                    source.raw(),
                    mark_def,
                    helper_skill_def,
                    skill.raw(),
                ],
            )
        }
        (4, 0) => {
            let mut events = remove_mark(tx, player, mark_def, 1)?;
            events.extend(skip_current_phase(tx, player)?);
            Ok(events)
        }
        _ => Err(EffectError::ContentFault(
            "unknown TW Guohuai resolver stage".into(),
        )),
    }
}

pub(crate) const TWCHENGPU_CUSTOM_CHOICE: u32 = 0x5457_8901;

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_twchengpu(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    source: PlayerId,
    card: CardId,
    chunlao_skill_def: u32,
    action: u8,
    stage: u8,
    _selected_card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match (action, stage) {
        (0, 0) => {
            crate::command::play::convert_saved_sha_to_fire(tx, player, card)?;
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .set_twlihuo_dying_cost(card);
            Ok(Vec::new())
        }
        (1, 0) => {
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| !twchengpu_field_cards(tx.state(), *target).is_empty())
                .collect();
            if targets.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                targets.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![
                    TWCHENGPU_CUSTOM_CHOICE,
                    0,
                    player.raw(),
                    chunlao_skill_def,
                    skill.raw(),
                ],
            )
        }
        (2, 0) if source == player => {
            crate::command::play::increase_saved_sha_base_damage(tx, source, card, 1)?;
            Ok(Vec::new())
        }
        (2, 0) => {
            let options = twchengpu_hand_equip_cards(tx.state(), source);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                source,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    TWCHENGPU_CUSTOM_CHOICE,
                    2,
                    player.raw(),
                    source.raw(),
                    card.raw(),
                    chunlao_skill_def,
                    skill.raw(),
                ],
            )
        }
        (2, 1) => {
            crate::command::play::increase_saved_sha_base_damage(tx, source, card, 1)?;
            Ok(Vec::new())
        }
        (3, 0) => {
            let cards = tx.state().skill_pile_cards(target, chunlao_skill_def);
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let from = tx
                .state()
                .skill_piles
                .iter()
                .find(|pile| pile.owner == target && pile.skill_def == chunlao_skill_def)
                .map(|pile| pile.zone)
                .ok_or_else(|| EffectError::EngineFault("TW Chunlao pile missing".into()))?;
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
            let mut events =
                move_cards_with_reason(tx, cards, discard, Some(from), CardMoveReason::Discard)?;
            events.extend(run_effects(
                tx,
                vec![
                    EffectRequest::DrawCards { player, count: 1 },
                    EffectRequest::RecoverHp {
                        source: player,
                        player: target,
                        amount: 1,
                    },
                ],
            )?);
            Ok(events)
        }
        _ => Err(EffectError::ContentFault(
            "unknown TW Chengpu resolver stage".into(),
        )),
    }
}

pub(crate) fn twchengpu_field_cards(
    state: &crate::state::GameState,
    player: PlayerId,
) -> Vec<CardId> {
    [ZoneKind::Hand, ZoneKind::Equip, ZoneKind::Judge]
        .into_iter()
        .filter_map(|kind| state.zones.find(kind, player))
        .filter_map(|zone| state.zones.get(zone))
        .flat_map(|zone| zone.cards.iter().copied())
        .collect()
}

pub(crate) fn twchengpu_hand_equip_cards(
    state: &crate::state::GameState,
    player: PlayerId,
) -> Vec<CardId> {
    [ZoneKind::Hand, ZoneKind::Equip]
        .into_iter()
        .filter_map(|kind| state.zones.find(kind, player))
        .filter_map(|zone| state.zones.get(zone))
        .flat_map(|zone| zone.cards.iter().copied())
        .collect()
}

pub(crate) const TWCAOANG_CUSTOM_CHOICE: u32 = 0x5457_9001;
pub(crate) const TWNIUJIN_CUSTOM_CHOICE: u32 = 0x5457_9101;
pub(crate) const TWMOUKUI_CUSTOM_CHOICE: u32 = 0x5457_9301;
pub(crate) const TWCHENXUN_CUSTOM_CHOICE: u32 = 0x5457_9701;
pub(crate) const TWFENXIAN_CUSTOM_CHOICE: u32 = 0x5457_9901;

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_twcaoang(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    source: PlayerId,
    card: CardId,
    helper_skill_def: u32,
    action: u8,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match (action, stage) {
        (0, 0) => {
            crate::command::play::redirect_saved_sha_target(tx, source, card, target, player)?;
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .set_twxiaolian_redirect(card, target);
            Ok(Vec::new())
        }
        (1, 0) => {
            let Some((remembered, original_target)) = tx
                .state()
                .skills
                .get(skill)
                .and_then(|skill| skill.twxiaolian_redirect())
            else {
                return Ok(Vec::new());
            };
            if remembered != card {
                return Ok(Vec::new());
            }
            let options = twchengpu_hand_equip_cards(tx.state(), player);
            if options.is_empty() {
                tx.state_mut()
                    .skills
                    .get_mut(skill)
                    .unwrap()
                    .clear_twxiaolian_redirect();
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
                ContinuationKind::Custom,
                vec![
                    TWCAOANG_CUSTOM_CHOICE,
                    0,
                    player.raw(),
                    original_target.raw(),
                    helper_skill_def,
                    skill.raw(),
                ],
            )
        }
        (2, 0) => {
            if tx
                .state()
                .skills
                .get(skill)
                .and_then(|skill| skill.twxiaolian_redirect())
                .is_some_and(|(remembered, _)| remembered == card)
            {
                tx.state_mut()
                    .skills
                    .get_mut(skill)
                    .unwrap()
                    .clear_twxiaolian_redirect();
            }
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault(
            "unknown TW Caoang resolver stage".into(),
        )),
    }
}

pub(crate) fn resolve_twniujin(
    tx: &mut Transaction,
    player: PlayerId,
    limited_skill_def: engine_api::SkillDefId,
    action: u8,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match (action, stage) {
        (0, 0) => {
            let hand = tx.state().hand_count(player);
            let highest = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(|target| tx.state().hand_count(target))
                .max()
                .unwrap_or(hand);
            let count = highest.saturating_sub(hand).min(5);
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveTwniujin {
                    player,
                    limited_skill_def,
                    action,
                    stage: 1,
                    skill,
                });
            run_effects(tx, vec![EffectRequest::DrawCards { player, count }])
        }
        (0, 1) if !tx.state().disabled_judge_zones.contains(&player) => {
            tx.state_mut().disabled_judge_zones.insert(player);
            let judge = tx
                .state()
                .zones
                .find(ZoneKind::Judge, player)
                .ok_or_else(|| EffectError::EngineFault("TW Niujin judge zone missing".into()))?;
            let cards = tx
                .state()
                .zones
                .get(judge)
                .map(|zone| zone.cards.clone())
                .unwrap_or_default();
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
            move_cards_with_reason(
                tx,
                cards,
                discard,
                Some(judge),
                engine_api::CardMoveReason::Discard,
            )
        }
        (0, 1) => {
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| *target != player)
                .map(PlayerId::raw)
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![TWNIUJIN_CUSTOM_CHOICE, 0, player.raw(), skill.raw()],
            )
        }
        (1, 0) => {
            let can_reset = tx
                .state()
                .skills
                .sorted_live()
                .into_iter()
                .find(|instance| instance.owner == player && instance.def == limited_skill_def)
                .is_some_and(|instance| instance.lifetime_fire_count > 0);
            if !can_reset {
                return draw_cards(tx, player, 2);
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                vec![0, 1],
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    TWNIUJIN_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    limited_skill_def.raw(),
                    skill.raw(),
                ],
            )
        }
        _ => Err(EffectError::ContentFault(
            "unknown TW Niujin resolver stage".into(),
        )),
    }
}

pub(crate) fn resolve_twmoukui(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if stage != 0 {
        return Err(EffectError::ContentFault(
            "unknown TW Moukui resolver stage".into(),
        ));
    }
    // 谋溃只在自己的【杀】指定该目标的结算窗口内发动。
    if !tx
        .state()
        .resolution
        .sha_target_resolving(player, card, target)
    {
        return Ok(Vec::new());
    }
    let mut options = vec![1];
    if !crate::query::discardable_target_cards(
        tx.state(),
        player,
        target,
        rule_api::TargetCardScope::HandEquip,
    )
    .is_empty()
    {
        options.push(2);
    }
    options.push(3);
    GuardedChoice::new(
        skill,
        Participants::pair(player, target),
        ContinuationKind::Custom,
        vec![
            TWMOUKUI_CUSTOM_CHOICE,
            player.raw(),
            target.raw(),
            card.raw(),
            skill.raw(),
        ],
    )
    .shape(ChoiceShape::range(0, 1, true))
    .open_tokens(tx, options, ChoiceKind::Custom)
}

pub(crate) fn resolve_twfuman(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    cards: Vec<CardId>,
    action: u8,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if stage != 0 || action > 2 {
        return Err(EffectError::ContentFault(
            "unknown TW Fuman resolver stage".into(),
        ));
    }
    if action != 0 {
        if !tx
            .state()
            .skills
            .get(skill)
            .is_some_and(|instance| instance.owner == player)
        {
            return Ok(Vec::new());
        }
        tx.state_mut()
            .skills
            .get_mut(skill)
            .unwrap()
            .take_tracked_cards(&cards);
        return Ok(Vec::new());
    }
    if !originating_skill_is_enabled(tx, player, skill)
        || player == target
        || ![player, target].into_iter().all(|participant| {
            tx.state()
                .players
                .get(participant)
                .is_some_and(|participant| participant.alive)
        })
        || cards.len() != 1
    {
        return Err(EffectError::Illegal("invalid TW Fuman gift".into()));
    }
    let from = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("TW Fuman owner has no hand".into()))?;
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("TW Fuman target has no hand".into()))?;
    let card = cards[0];
    if !tx
        .state()
        .cards
        .get(card)
        .is_some_and(|state| state.owner == player && state.zone == from)
    {
        return Err(EffectError::Illegal(
            "TW Fuman card left the owner's hand".into(),
        ));
    }
    record_active_target(tx, player, target, skill)?;
    if !tx
        .state_mut()
        .skills
        .get_mut(skill)
        .is_some_and(|instance| instance.add_tracked_cards(&cards))
    {
        return Err(EffectError::ContentFault(
            "TW Fuman tracked-card state is occupied".into(),
        ));
    }
    give_cards_as(tx, player, cards, to, from)
}

pub(crate) fn resolve_cuijin(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(instance) = tx.state().skills.get(skill) else {
        return Ok(Vec::new());
    };
    if instance.owner != player || action > 1 {
        return Err(EffectError::ContentFault(
            "invalid Cuijin resolver context".into(),
        ));
    }
    if action == 0 {
        if !originating_skill_is_enabled(tx, player, skill)
            || !tx
                .state()
                .resolution
                .sha_resolution_records
                .iter()
                .any(|record| record.source == source && record.card == card)
        {
            return Ok(Vec::new());
        }
        tx.state_mut()
            .skills
            .get_mut(skill)
            .unwrap()
            .set_source_free_physical_card(card);
    } else if instance.source_free_physical_card() == Some(card) {
        tx.state_mut()
            .skills
            .get_mut(skill)
            .unwrap()
            .take_source_free_physical_card();
    }
    Ok(Vec::new())
}

pub(crate) fn resolve_twqingkou(
    tx: &mut Transaction,
    player: PlayerId,
    winner: PlayerId,
    card: CardId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(instance) = tx.state().skills.get(skill) else {
        return Ok(Vec::new());
    };
    if instance.owner != player || action > 1 {
        return Err(EffectError::ContentFault(
            "invalid TW Qingkou resolver context".into(),
        ));
    }
    if action == 0 {
        return choose_use_virtual_card_impl(
            tx,
            player,
            CardKind::TrickJuedou.tag(),
            skill,
            VirtualCardTracking::SourceFree,
            true,
        );
    }
    if instance.source_free_physical_card() != Some(card)
        || !tx
            .state()
            .players
            .get(winner)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    tx.state_mut()
        .skills
        .get_mut(skill)
        .unwrap()
        .take_source_free_physical_card();
    if winner == player {
        if !tx.state().pending_judge_skips.contains(&player) {
            tx.state_mut().pending_judge_skips.push(player);
            tx.state_mut()
                .pending_judge_skips
                .sort_by_key(|participant| participant.raw());
        }
        tx.state_mut().turn.set_flag(turn_flag::SKIP_DISCARD, 1);
    }
    draw_cards(tx, winner, 1)
}

pub(crate) fn twchenxun_targets(
    tx: &Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Vec<PlayerId> {
    let used = tx
        .state()
        .skills
        .get(skill)
        .map_or_else(Vec::new, |instance| instance.twchenxun_targets());
    tx.state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| {
            !used.contains(target)
                && crate::command::play::validate_source_free_card_on_target(
                    tx,
                    player,
                    CardKind::TrickJuedou,
                    *target,
                )
                .is_ok()
        })
        .collect()
}

pub(crate) fn open_twchenxun_target(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let targets = twchenxun_targets(tx, player, skill);
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    let mut payload = vec![
        TWCHENXUN_CUSTOM_CHOICE,
        player.raw(),
        skill.raw(),
        targets.len().min(u32::MAX as usize) as u32,
    ];
    payload.extend(targets.iter().map(|target| target.raw()));
    open_choice(
        tx,
        player,
        0,
        1,
        true,
        targets.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::Custom,
        payload,
    )
}

pub(crate) fn resolve_twchenxun(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) || action > 4 {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            let round = tx.state().turn.round;
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .reset_twchenxun_round(round);
            open_twchenxun_target(tx, player, skill)
        }
        1 => {
            if tx
                .state()
                .skills
                .get(skill)
                .and_then(|instance| instance.twchenxun_card())
                != Some(card)
            {
                return Ok(Vec::new());
            }
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .clear_twchenxun_card();
            run_effects(
                tx,
                vec![
                    EffectRequest::DrawCardsFromSkill {
                        player,
                        count: 1,
                        skill,
                    },
                    EffectRequest::ResolveTwchenxun {
                        player,
                        target: PlayerId::NONE,
                        card: CardId::NONE,
                        action: 4,
                        skill,
                    },
                ],
            )
        }
        2 => {
            if tx
                .state()
                .skills
                .get(skill)
                .and_then(|instance| instance.twchenxun_card())
                != Some(card)
            {
                return Ok(Vec::new());
            }
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .clear_twchenxun_card();
            run_effects(
                tx,
                vec![EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target: player,
                    amount: 1,
                    is_lose_hp: true,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )
        }
        3 => {
            if !twchenxun_targets(tx, player, skill).contains(&target) {
                return Ok(Vec::new());
            }
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .add_twchenxun_target(target);
            let card = materialize_virtual_card(tx, player, CardKind::TrickJuedou)?;
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .set_twchenxun_card(card);
            crate::command::play::play_as_kind(
                tx,
                player,
                card,
                CardKind::TrickJuedou,
                &[target],
                false,
                false,
                false,
                false,
            )?;
            Ok(Vec::new())
        }
        4 => open_twchenxun_target(tx, player, skill),
        _ => unreachable!(),
    }
}

pub(crate) fn twfenxian_duel_targets(tx: &Transaction, owner: PlayerId, actor: PlayerId) -> Vec<PlayerId> {
    tx.state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| {
            *target != owner
                && crate::command::play::validate_source_free_card_on_target(
                    tx,
                    actor,
                    CardKind::TrickJuedou,
                    *target,
                )
                .is_ok()
        })
        .collect()
}

pub(crate) fn twfenxian_duel_cards(tx: &Transaction, owner: PlayerId, actor: PlayerId) -> Vec<CardId> {
    let mut players = vec![owner];
    if actor != owner {
        players.push(actor);
    }
    let mut cards: Vec<_> = players
        .into_iter()
        .flat_map(|player| {
            [ZoneKind::Equip, ZoneKind::Judge]
                .into_iter()
                .filter_map(move |kind| tx.state().zones.find(kind, player))
        })
        .filter_map(|zone| tx.state().zones.get(zone))
        .flat_map(|zone| zone.cards.iter().copied())
        .collect();
    if twfenxian_duel_targets(tx, owner, actor).is_empty() {
        cards.clear();
    }
    cards
}

pub(crate) fn open_twfenxian_choice(
    tx: &mut Transaction,
    actor: PlayerId,
    kind: ChoiceKind,
    options: Vec<u32>,
    payload: Vec<u32>,
) -> Result<Vec<DomainEvent>, EffectError> {
    open_choice(
        tx,
        actor,
        1,
        1,
        false,
        options,
        kind,
        ContinuationKind::Custom,
        payload,
    )
}

pub(crate) fn resolve_twfenxian(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(instance) = tx.state().skills.get(skill) else {
        return Ok(Vec::new());
    };
    let owner = instance.owner;
    if !tx.state().skill_effectively_enabled(instance) || action > 4 {
        return Ok(Vec::new());
    }
    if action < 4 && player != owner {
        return Err(EffectError::ContentFault(
            "invalid TW Fenxian owner context".into(),
        ));
    }
    match action {
        0 => {
            let mut options = Vec::new();
            if !twfenxian_duel_cards(tx, owner, target).is_empty() {
                options.push(0);
            }
            if crate::command::play::validate_source_free_card_on_target(
                tx,
                owner,
                CardKind::TrickHuogong,
                target,
            )
            .is_ok()
            {
                options.push(1);
            }
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let branch_mask = options.iter().fold(0, |mask, branch| mask | (1 << branch));
            open_twfenxian_choice(
                tx,
                target,
                ChoiceKind::Custom,
                options,
                vec![
                    TWFENXIAN_CUSTOM_CHOICE,
                    0,
                    owner.raw(),
                    target.raw(),
                    skill.raw(),
                    branch_mask,
                ],
            )
        }
        1 => {
            crate::command::play::validate_source_free_card_on_target(
                tx,
                owner,
                CardKind::TrickHuogong,
                target,
            )?;
            let card = materialize_virtual_card(tx, owner, CardKind::TrickHuogong)?;
            crate::command::play::play_as_kind(
                tx,
                owner,
                card,
                CardKind::TrickHuogong,
                &[target],
                true,
                false,
                true,
                false,
            )?;
            Ok(Vec::new())
        }
        2 => {
            let cards = twfenxian_duel_cards(tx, owner, target);
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            open_twfenxian_choice(
                tx,
                target,
                ChoiceKind::SelectCards,
                cards.iter().map(|card| card.raw()).collect(),
                vec![
                    TWFENXIAN_CUSTOM_CHOICE,
                    2,
                    owner.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )
        }
        3 => {
            if !twfenxian_duel_cards(tx, owner, target).contains(&card) {
                return Err(EffectError::Illegal(
                    "TW Fenxian field card is stale".into(),
                ));
            }
            let targets = twfenxian_duel_targets(tx, owner, target);
            if targets.is_empty() {
                return Ok(Vec::new());
            }
            open_twfenxian_choice(
                tx,
                target,
                ChoiceKind::SelectTargets,
                targets.iter().map(|target| target.raw()).collect(),
                vec![
                    TWFENXIAN_CUSTOM_CHOICE,
                    3,
                    owner.raw(),
                    target.raw(),
                    skill.raw(),
                    card.raw(),
                ],
            )
        }
        4 => {
            if !twfenxian_duel_cards(tx, owner, player).contains(&card)
                || !twfenxian_duel_targets(tx, owner, player).contains(&target)
            {
                return Err(EffectError::Illegal(
                    "TW Fenxian Duel context changed".into(),
                ));
            }
            crate::command::play::play_field_card_as_kind(
                tx,
                player,
                card,
                CardKind::TrickJuedou,
                &[target],
            )?;
            Ok(Vec::new())
        }
        _ => unreachable!(),
    }
}

pub(crate) fn set_skill_content_flag(
    tx: &mut Transaction,
    player: PlayerId,
    bit: u8,
    value: bool,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let instance = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .filter(|instance| instance.owner == player)
        .ok_or_else(|| EffectError::ContentFault("invalid originating skill flag owner".into()))?;
    if !instance.set_content_flag(bit, value) {
        return Err(EffectError::ContentFault(
            "skill content flag bit must be below 64".into(),
        ));
    }
    Ok(Vec::new())
}

pub(crate) fn open_twmuyue_target(
    tx: &mut Transaction,
    player: PlayerId,
    card_kind: u8,
    matching_discard: bool,
    helper_skill_def: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let options = tx.state().players.alive_by_seat();
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options.iter().map(|target| target.raw()).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ResolveTwchenzhen,
        vec![
            3,
            player.raw(),
            helper_skill_def,
            skill.raw(),
            card_kind.into(),
            u32::from(matching_discard),
        ],
    )
}

pub(crate) fn apply_twchayi_initial(
    tx: &mut Transaction,
    target: PlayerId,
    branch: u8,
    effect_skill_def: u32,
    record_skill_def: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let hand_count = tx.state().hand_count(target);
    let mut events = run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: target,
            skill_def: engine_api::SkillDefId::new(record_skill_def),
        }],
    )?;
    let record = tx
        .state()
        .players
        .get(target)
        .into_iter()
        .flat_map(|player| player.skills.iter())
        .copied()
        .find(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == record_skill_def)
        })
        .ok_or_else(|| EffectError::EngineFault("TW Chayi record skill missing".into()))?;
    tx.state_mut()
        .skills
        .get_mut(record)
        .expect("validated record skill")
        .set_twchayi_state(branch, hand_count);
    if branch == 0 {
        let cards = tx.state().hand_cards(target);
        if !cards.is_empty() {
            events.push(DomainEvent::CardsRevealed {
                cards,
                to: Vec::new(),
            });
        }
    } else {
        events.extend(run_effects(
            tx,
            vec![
                EffectRequest::AddSkill {
                    player: target,
                    skill_def: engine_api::SkillDefId::new(effect_skill_def),
                },
                EffectRequest::AddMark {
                    player: target,
                    mark: engine_api::MarkDefId::new(effect_skill_def),
                    delta: 1,
                },
            ],
        )?);
    }
    Ok(events)
}

/// 嬉战：弃置一张牌或失去 1 点体力；弃牌时暂时禁用芳踪，并按花色视为用牌。
pub(crate) fn resolve_spxizhan(
    tx: &mut Transaction,
    player: PlayerId,
    phase_player: PlayerId,
    fangzong_skill_def: engine_api::SkillDefId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if player == phase_player
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(phase_player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let cards = crate::query::discardable_target_cards(
        tx.state(),
        player,
        player,
        rule_api::TargetCardScope::HandEquip,
    );
    if cards.is_empty() {
        // 无可弃牌时只能失去体力。
        return apply_effect(
            tx,
            EffectRequest::DealDamage {
                source: PlayerId::NONE,
                target: player,
                amount: 1,
                is_lose_hp: true,
                nature: rule_api::damage_nature::NORMAL,
            },
        );
    }
    open_choice(
        tx,
        player,
        0,
        1,
        false,
        cards.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseSpxizhanDiscardOrLose,
        vec![player.raw(), phase_player.raw(), fangzong_skill_def.raw()],
    )
}

/// 送枢（SP 张温）：其他角色摸牌阶段开始时，将摸牌数置零，
/// 令其从仁库（拥有者「割卜」技能牌堆）选至多 min(5, 拥有者体力, 仁库牌数) 张获得，
/// 并临时获得“不能指定其他角色”技能。
pub(crate) fn resolve_spsongshu(
    tx: &mut Transaction,
    player: PlayerId,
    phase_player: PlayerId,
    renku_skill_def: engine_api::SkillDefId,
    block_skill_def: engine_api::SkillDefId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if player == phase_player
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive && participant.hp > 0)
        || !tx
            .state()
            .players
            .get(phase_player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let cards = tx.state().skill_pile_cards(player, renku_skill_def.raw());
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let owner_hp = tx
        .state()
        .players
        .get(player)
        .map(|p| p.hp.max(0) as usize)
        .unwrap_or(0);
    let max_count = cards.len().min(5).min(owner_hp.max(1)) as u8;
    tx.state_mut().turn.set_draw_phase_fixed_count(0);
    open_choice(
        tx,
        phase_player,
        1,
        max_count,
        false,
        cards.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseSpsongshuCards,
        vec![
            player.raw(),
            phase_player.raw(),
            renku_skill_def.raw(),
            block_skill_def.raw(),
        ],
    )
}

/// 存嗣：拥有者翻面，从牌堆/弃牌堆取一张【杀】移动到目标手牌。
pub(crate) fn resolve_spcunsi(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
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
        return Ok(Vec::new());
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let mut events = Vec::new();
    events.extend(apply_effect(tx, EffectRequest::TurnOver { player })?);
    if let Some((from, card)) = first_shared_pile_card_by_kind(tx, CardKind::Sha)? {
        events.extend(move_cards(tx, vec![card], hand, Some(from))?);
    }
    Ok(events)
}

/// 驭雳（神马超，ABI 409）：
/// mode 0 —— 自己造成伤害前：非雷伤害改为雷属性（prevent + 同源同量同牌雷重放），
/// 已是雷则伤害 +1；mode 1 —— 自己受伤前：取消伤害并摸伤害点数张。
/// 重放复用 ABI 374 伤害重放管线（prevented_damage_ids + damage_redirect_in_progress）
/// 防止重放伤害再次触发本技能形成递归。
pub(crate) fn resolve_yuli(
    tx: &mut Transaction,
    player: PlayerId,
    mode: u8,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
    nature: u8,
    card: CardId,
    card_kind: u8,
    card_flags: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let damage_id = tx.state().resolution.current_damage_id;
    // mode 0（改雷重放）在重放期间跳过以防递归；mode 1（防御）必须正常响应
    // 重放出的雷伤害（它是重放链的合法观察者）。
    if damage_id == 0
        || (mode == 0 && tx.state().resolution.damage_redirect_in_progress)
        || amount == 0
        || tx
            .state()
            .resolution
            .prevented_damage_ids
            .contains(&damage_id)
    {
        return Ok(Vec::new());
    }
    match mode {
        0 => {
            let already_thunder = nature == rule_api::damage_nature::THUNDER;
            tx.state_mut()
                .resolution
                .prevented_damage_ids
                .push(damage_id);
            tx.state_mut().resolution.damage_redirect_in_progress = true;
            let result = run_effects(
                tx,
                vec![EffectRequest::DealCardDamage {
                    source,
                    target,
                    amount: amount.saturating_add(u32::from(already_thunder)),
                    is_lose_hp: false,
                    nature: rule_api::damage_nature::THUNDER,
                    card,
                    card_kind,
                    card_flags,
                }],
            );
            tx.state_mut().resolution.damage_redirect_in_progress = false;
            result
        }
        1 => {
            // 旧 JS 中「自己受伤」分支仅在伤害为雷属性时触发（damageBegin4 filter）。
            if nature != rule_api::damage_nature::THUNDER {
                return Ok(Vec::new());
            }
            tx.state_mut()
                .resolution
                .prevented_damage_ids
                .push(damage_id);
            run_effects(
                tx,
                vec![EffectRequest::DrawCards {
                    player,
                    count: amount,
                }],
            )
        }
        _ => Err(EffectError::ContentFault("unknown Yuli mode".into())),
    }
}

/// 霆威（神马超，ABI 410）：
/// mode 0 —— 使用【杀】指定首个目标后：加 4 枚霆标记，打开「选择该杀一个目标」choice
/// （观察者=拥有者）；mode 1 —— 所选目标四选一：非锁定技失效至其回合结束 / 交给拥有者
/// 一张装备牌 / 此【杀】对其伤害 +1 / 随机弃一张牌；取消则其横置。
/// 两段 choice 由 resolve_choice.rs 的 ContinuationKind::TingweiSelectTarget /
/// TingweiTargetOption 驱动。
pub(crate) fn resolve_tingwei(
    tx: &mut Transaction,
    player: PlayerId,
    mode: u8,
    card: CardId,
    targets: Vec<PlayerId>,
    target: PlayerId,
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
    // “霆”标记定义 ID（与 shiji 寂灭 jimie 共用）。
    const TING_MARK_DEF: u32 = 4_145_075_227;
    let mut events = Vec::new();
    match mode {
        0 => {
            if targets.is_empty() {
                return Ok(Vec::new());
            }
            events.extend(apply_effect(
                tx,
                EffectRequest::AddMark {
                    player,
                    mark: engine_api::MarkDefId::new(TING_MARK_DEF),
                    delta: 4,
                },
            )?);
            let mut payload: Vec<u32> = vec![RESOLVE_TINGWEI_SELECT, player.raw(), card.raw()];
            payload.extend(targets.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                targets.iter().map(|target| target.raw()).collect(),
                ChoiceKind::Custom,
                crate::state::ContinuationKind::Custom,
                payload,
            )?;
            Ok(events)
        }
        1 => {
            if target.is_none() || !tx.state().players.get(target).is_some_and(|p| p.alive) {
                return Ok(Vec::new());
            }
            let payload: Vec<u32> = vec![
                RESOLVE_TINGWEI_OPTION,
                player.raw(),
                target.raw(),
                card.raw(),
            ];
            open_choice(
                tx,
                target,
                1,
                1,
                true,
                vec![0, 1, 2, 3],
                ChoiceKind::Custom,
                crate::state::ContinuationKind::Custom,
                payload,
            )?;
            Ok(events)
        }
        _ => Err(EffectError::ContentFault("unknown Tingwei mode".into())),
    }
}

/// 新名士（孔融，ABI 411）：拥有者受伤后（有谦标记）伤害来源强制弃一张
/// 手牌/装备/判定区牌；弃牌由 resolve_choice 的
/// RESOLVE_XINMINGSHI_CUSTOM_CHOICE 分支结算（红色→拥有者回复 1；
/// 进入弃牌堆→拥有者获得）。
pub(crate) fn resolve_xinmingshi(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    const QIAN_MARK: u32 = 2_286_538_070;
    // 须持有谦标记（新礼让授予）。
    if !tx.state().players.get(player).is_some_and(|participant| {
        participant
            .marks
            .iter()
            .any(|(m, n)| *m == QIAN_MARK && *n > 0)
    }) {
        return Ok(Vec::new());
    }
    let gainable = crate::query::gainable_target_cards(
        tx.state(),
        source,
        source,
        rule_api::TargetCardScope::HandEquipJudge,
    );
    GuardedChoice::new(
        skill,
        Participants::distinct_pair(player, source),
        ContinuationKind::Custom,
        vec![RESOLVE_XINMINGSHI_CUSTOM_CHOICE, player.raw(), skill.raw()],
    )
    .chosen_by(source)
    .open_cards(tx, gainable)
}

/// 挽危（主动阶段）：目标回复 player.hp 点体力，拥有者失去 player.hp 点体力。
pub(crate) fn spwanwei_active(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let p = tx
        .state()
        .players
        .get(player)
        .ok_or_else(|| EffectError::Illegal("bad player".into()))?;
    let t = tx
        .state()
        .players
        .get(target)
        .ok_or_else(|| EffectError::Illegal("bad target".into()))?;
    if !p.alive || !t.alive || p.hp < 1 {
        return Ok(Vec::new());
    }
    let num = p.hp as u32;
    let mut events = Vec::new();
    events.extend(apply_effect(
        tx,
        EffectRequest::RecoverHp {
            source: player,
            player: target,
            amount: num,
        },
    )?);
    events.extend(apply_effect(
        tx,
        EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: player,
            amount: num,
            is_lose_hp: true,
            nature: rule_api::damage_nature::NORMAL,
        },
    )?);
    Ok(events)
}

/// 挽危（濒死救援）：目标回复 max(player.hp + 1, 1 - target.hp) 体力，拥有者失去 player.hp 点体力。
pub(crate) fn spwanwei_dying(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let p = tx
        .state()
        .players
        .get(player)
        .ok_or_else(|| EffectError::Illegal("bad player".into()))?;
    let t = tx
        .state()
        .players
        .get(target)
        .ok_or_else(|| EffectError::Illegal("bad target".into()))?;
    if !p.alive || !t.alive || p.hp < 1 {
        return Ok(Vec::new());
    }
    let num = p.hp as u32;
    let needed = if t.hp < 1 { (1 - t.hp) as u32 } else { 1 };
    let recover_amount = (num + 1).max(needed);
    let mut events = Vec::new();
    events.extend(apply_effect(
        tx,
        EffectRequest::RecoverHp {
            source: player,
            player: target,
            amount: recover_amount,
        },
    )?);
    events.extend(apply_effect(
        tx,
        EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: player,
            amount: num,
            is_lose_hp: true,
            nature: rule_api::damage_nature::NORMAL,
        },
    )?);
    Ok(events)
}

/// 殉义（王甫赵累，ABI 412）：mode 0 选择“义”目标（获义标记）；mode 1 伤害事件分流
/// ——拥有者/义目标任一造成伤害另一人摸 1、任一受伤另一人强制弃 1 张手牌/装备。
/// 选择义目标的 choice 由 resolve_choice 的 RESOLVE_XUNYI_SELECT 分支结算。
pub(crate) fn resolve_xunyi(
    tx: &mut Transaction,
    player: PlayerId,
    mode: u8,
    source: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    // “义”标记定义 ID（= xunyi 技能定义 ID）。
    const YI_MARK: u32 = 2_517_009_871;
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    match mode {
        0 => {
            // 已存在义标记者（含已死目标）则不重复选择。
            if tx
                .state()
                .players
                .iter_live()
                .any(|p| p.id != player && p.marks.iter().any(|(m, n)| *m == YI_MARK && *n > 0))
            {
                return Ok(Vec::new());
            }
            let candidates: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| *candidate != player)
                .map(|candidate| candidate.raw())
                .collect();
            if candidates.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                candidates,
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![RESOLVE_XUNYI_SELECT, player.raw(), skill.raw()],
            )
        }
        1 => {
            let yi = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .find(|candidate| {
                    *candidate != player
                        && tx.state().players.get(*candidate).is_some_and(|p| {
                            p.marks.iter().any(|(m, n)| *m == YI_MARK && *n > 0)
                        })
                });
            let mut effects = Vec::new();
            if source == player {
                // 王甫造成伤害 → 义目标摸 1。
                if let Some(yi) = yi {
                    effects.push(EffectRequest::DrawCards {
                        player: yi,
                        count: 1,
                    });
                }
            } else if yi == Some(source) {
                // 义目标造成伤害 → 王甫摸 1。
                effects.push(EffectRequest::DrawCards { player, count: 1 });
            } else if target == player {
                // 王甫受伤 → 义目标强制弃 1 张手牌/装备。
                if let Some(yi) = yi {
                    effects.push(EffectRequest::ChooseDiscardOneHandOrEquip { player: yi });
                }
            } else if yi == Some(target) {
                // 义目标受伤 → 王甫强制弃 1 张手牌/装备。
                effects.push(EffectRequest::ChooseDiscardOneHandOrEquip { player });
            }
            run_effects(tx, effects)
        }
        _ => Err(EffectError::ContentFault("unknown Xunyi mode".into())),
    }
}

/// 博明（许靖，ABI 413）：把主动技选中的牌赠给目标（触发 CardsGiven 事件）。
pub(crate) fn resolve_boming(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || card.is_none()
        || target == player
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let from = tx
        .state()
        .cards
        .get(card)
        .map(|card| card.zone)
        .ok_or_else(|| EffectError::Illegal("boming card missing".into()))?;
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("target hand missing".into()))?;
    give_cards_as(tx, player, vec![card], to, from)
}

/// 恶荐（许靖，ABI 414）：获牌者除赠牌外仍有同类（基本/锦囊/装备）手牌或装备牌时，
/// 二选一——受 1 点伤害 / 展示并弃所有同类牌。选择由 resolve_choice 的
/// RESOLVE_EJIAN_CHOICE 分支结算。
pub(crate) fn resolve_ejian(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) || card.is_none() {
        return Ok(Vec::new());
    }
    let category = card_category_of(tx.state(), card);
    let has_same = [ZoneKind::Hand, ZoneKind::Equip]
        .into_iter()
        .filter_map(|kind| tx.state().zones.find(kind, target))
        .filter_map(|zone| tx.state().zones.get(zone))
        .flat_map(|zone| zone.cards.iter().copied())
        .any(|candidate| {
            candidate != card && card_category_of(tx.state(), candidate) == category
        });
    if !has_same {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        target,
        1,
        1,
        false,
        vec![0, 1],
        ChoiceKind::Custom,
        ContinuationKind::Custom,
        vec![
            RESOLVE_EJIAN_CHOICE,
            player.raw(),
            target.raw(),
            card.raw(),
        ],
    )
}

/// 异勇（王双，ABI 415）：受到【杀】伤害后获得该杀实体牌，若来源存活且实体牌在
/// 手牌中，立即用其反击来源（来源无武器则伤害基数 2；实体牌随反击消耗）。
pub(crate) fn resolve_yiyong(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    source: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || card.is_none()
        || source == player
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let mut effects = Vec::new();
    // 1. 获得弃牌堆中的杀实体。
    let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
    let gained = discard.is_some_and(|discard| {
        tx.state()
            .cards
            .get(card)
            .is_some_and(|c| c.zone == discard && c.kind == CardKind::Sha)
    });
    if gained {
        let hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?;
        effects.push(EffectRequest::MoveCards {
            cards: vec![card],
            to: hand,
            from: discard,
        });
    }
    // 2. 反击条件：来源存活、实体牌已到手。
    let source_alive = tx
        .state()
        .players
        .get(source)
        .is_some_and(|participant| participant.alive);
    if gained && source_alive {
        // 来源有武器则基数 1，否则基数 2。
        let source_armed = tx
            .state()
            .zones
            .find(ZoneKind::Equip, source)
            .is_some_and(|zone| {
                tx.state().zones.get(zone).is_some_and(|z| {
                    z.cards.iter().any(|c| {
                        tx.state()
                            .cards
                            .get(*c)
                            .is_some_and(|v| v.kind == CardKind::Weapon)
                    })
                })
            });
        let base = if source_armed { 1 } else { 2 };
        effects.push(EffectRequest::DealDamage {
            source: player,
            target: source,
            amount: base,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        });
        // 实体牌随反击消耗（移入弃牌堆）。
        if let Some(discard) = discard {
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player);
            if let Some(hand) = hand {
                effects.push(EffectRequest::MoveCards {
                    cards: vec![card],
                    to: discard,
                    from: Some(hand),
                });
            }
        }
    }
    run_effects(tx, effects)
}

/// 善械（王双，ABI 416）：牌堆（仅 DrawPile）有武器则获得其中一张；否则随机夺一名
/// 有武器角色的武器（随机数只使用可回放 RuleRng）。
pub(crate) fn resolve_shanxie(
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
    let to_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?;
    // 牌堆（仅 DrawPile）武器检索。
    let draw = tx.state().zones.find(ZoneKind::DrawPile, PlayerId::NONE);
    let weapon_in_draw = draw.and_then(|zone| {
        tx.state().zones.get(zone).and_then(|z| {
            z.cards.iter().rev().copied().find(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|v| v.kind == CardKind::Weapon)
            })
        })
    });
    if let Some(card) = weapon_in_draw {
        let from = tx
            .state()
            .cards
            .get(card)
            .map(|card| card.zone)
            .ok_or_else(|| EffectError::Illegal("weapon card missing".into()))?;
        return run_effects(
            tx,
            vec![EffectRequest::MoveCards {
                cards: vec![card],
                to: to_hand,
                from: Some(from),
            }],
        );
    }
    // 随机夺一名有武器角色的武器。
    let armed: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|candidate| *candidate != player)
        .filter(|candidate| {
            tx.state()
                .zones
                .find(ZoneKind::Equip, *candidate)
                .is_some_and(|zone| {
                    tx.state().zones.get(zone).is_some_and(|z| {
                        z.cards.iter().any(|c| {
                            tx.state()
                                .cards
                                .get(*c)
                                .is_some_and(|v| v.kind == CardKind::Weapon)
                        })
                    })
                })
        })
        .collect();
    if armed.is_empty() {
        return Ok(Vec::new());
    }
    let mut rng = crate::rng::RuleRng::from_state(tx.state().rng);
    let victim = armed[rng.next_u64() as usize % armed.len()];
    let weapons: Vec<_> = tx
        .state()
        .zones
        .find(ZoneKind::Equip, victim)
        .map(|zone| {
            tx.state()
                .zones
                .get(zone)
                .map(|z| {
                    z.cards
                        .iter()
                        .copied()
                        .filter(|c| {
                            tx.state()
                                .cards
                                .get(*c)
                                .is_some_and(|v| v.kind == CardKind::Weapon)
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        })
        .unwrap_or_default();
    if weapons.is_empty() {
        tx.state_mut().rng = rng.into_state();
        return Ok(Vec::new());
    }
    let card = weapons[rng.next_u64() as usize % weapons.len()];
    tx.state_mut().rng = rng.into_state();
    let from = tx
        .state()
        .cards
        .get(card)
        .map(|card| card.zone)
        .ok_or_else(|| EffectError::Illegal("weapon card missing".into()))?;
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: to_hand,
            from: Some(from),
        }],
    )
}

/// 卡牌粗类别标签（0=基本 1=锦囊 2=装备），供 owner_card_category_mask 位运算。
pub(crate) fn card_category_tag(kind: CardKind) -> Option<u32> {
    if kind.is_basic() {
        Some(0)
    } else if kind.is_trick() {
        Some(1)
    } else if kind.is_equip() {
        Some(2)
    } else {
        None
    }
}

/// 舍医（蔡贞姬，ABI 417）：其他角色受伤前（每轮一次，DSL filter 限）交至少
/// max(1, hp) 张手牌/装备牌防伤。选牌 choice 由 resolve_choice 的
/// RESOLVE_SHEYI_CHOICE 分支结算（赠牌 + prevent）。
pub(crate) fn resolve_sheyi(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || target == player
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
    // 每轮一次（引擎内检查；DSL filter 移除 once_per_round 以避免与
    // optional 确认流程的交互问题）。
    let round = tx.state().turn.round;
    if tx
        .state()
        .skills
        .get(skill)
        .is_some_and(|instance| instance.round_fire_round == round)
    {
        return Ok(Vec::new());
    }
    tx.state_mut()
        .skills
        .get_mut(skill)
        .map(|instance| instance.round_fire_round = round);
    let hp = tx.state().players.get(player).map(|p| p.hp).unwrap_or(0);
    let need = hp.max(1) as u32;
    let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
    if (options.len() as u32) < need {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        need as u8,
        options.len() as u8,
        true,
        options.iter().map(|card| card.raw()).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        vec![
            RESOLVE_SHEYI_CHOICE,
            player.raw(),
            target.raw(),
            skill.raw(),
        ],
    )
}

/// 天荫（蔡贞姬，ABI 418）：结束阶段获得牌堆顶每个本阶段未用类别牌的第一张。
pub(crate) fn resolve_tianyin(
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
    let mut seen = tx.state().turn.owner_card_category_mask(player) as u32 & 0b111;
    let draw = tx.state().zones.find(ZoneKind::DrawPile, PlayerId::NONE);
    let Some(draw) = draw else {
        return Ok(Vec::new());
    };
    let mut collected = Vec::new();
    for card in tx
        .state()
        .zones
        .get(draw)
        .map(|z| z.cards.iter().rev().copied().collect::<Vec<_>>())
        .unwrap_or_default()
    {
        let Some(category) = tx
            .state()
            .cards
            .get(card)
            .and_then(|value| card_category_tag(value.kind))
        else {
            continue;
        };
        let bit = 1u32 << category;
        if seen & bit == 0 {
            collected.push(card);
            seen |= bit;
            if seen & 0b111 == 0b111 {
                break;
            }
        }
    }
    if collected.is_empty() {
        return Ok(Vec::new());
    }
    let to_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?;
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: collected,
            to: to_hand,
            from: Some(draw),
        }],
    )
}

/// 追锋（标文鸯，ABI 419）：失 1 点体力，然后选择一名其他角色并对其视为使用
/// 一张【决斗】（无距离限制近似）。
pub(crate) fn resolve_dbzhuifeng(
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
        || tx.state().players.get(player).map(|p| p.hp).unwrap_or(0) <= 0
    {
        return Ok(Vec::new());
    }
    let mut events = Vec::new();
    events.extend(run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: player,
            amount: 1,
            is_lose_hp: true,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )?);
    let candidates: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|candidate| *candidate != player)
        .map(|candidate| candidate.raw())
        .collect();
    if candidates.is_empty() {
        return Ok(events);
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        candidates,
        ChoiceKind::Custom,
        ContinuationKind::Custom,
        vec![RESOLVE_DBZHUIFENG_TARGET, player.raw(), skill.raw()],
    )?;
    Ok(events)
}

/// Custom-continuation marker for ABI 419 追锋选目标（payload[0]）。
pub(crate) const RESOLVE_DBZHUIFENG_TARGET: u32 = 0xDCDA_4190;

/// 曲迪（标文鸯，ABI 420）：单目标杀/决斗指定后三选一。
pub(crate) fn resolve_dbquedi(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if card.is_none() {
        return Ok(Vec::new());
    }
    // 三个选项至少要有一个可执行（获牌 / 弃基本牌），否则不开选择。
    let can_gain = !crate::query::gainable_target_cards(
        tx.state(),
        player,
        target,
        rule_api::TargetCardScope::Hand,
    )
    .is_empty();
    let has_basic = tx.state().hand_cards(player).into_iter().any(|candidate| {
        tx.state()
            .cards
            .get(candidate)
            .is_some_and(|value| value.kind.is_basic())
    });
    if !can_gain && !has_basic {
        return Ok(Vec::new());
    }
    GuardedChoice::new(
        skill,
        Participants::distinct_other_only(player, target),
        ContinuationKind::Custom,
        vec![
            RESOLVE_DBQUEDI_CHOICE,
            player.raw(),
            target.raw(),
            card.raw(),
            skill.raw(),
        ],
    )
    .shape(ChoiceShape::one_optional())
    .open_tokens(tx, vec![0, 1, 2], ChoiceKind::Custom)
}

/// 隐据（辛毗，ABI 421）：目标二选一——对拥有者使用【杀】/ 获得跳过标记。
pub(crate) fn resolve_spyinju(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    GuardedChoice::new(
        skill,
        Participants::distinct_other_only(player, target),
        ContinuationKind::Custom,
        vec![
            RESOLVE_SPYINJU_CHOICE,
            player.raw(),
            target.raw(),
            skill.raw(),
        ],
    )
    .chosen_by(target)
    .open_tokens(tx, vec![0, 1], ChoiceKind::Custom)
}
