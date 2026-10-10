//! Effect primitive implementations — old cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

pub(crate) fn oldxuanfeng_distance_one(tx: &Transaction, player: PlayerId, target: PlayerId) -> bool {
    let mut ctx = crate::query::QueryContext::new();
    matches!(
        crate::query::query(
            tx.state(),
            &QueryRequest::Distance {
                from: player,
                to: target,
            },
            &mut ctx,
        ),
        QueryResponse::Int(distance) if distance <= 1
    )
}

pub(crate) fn remove_oldxuanfeng_virtual_sha(tx: &mut Transaction, card: CardId) -> Result<(), EffectError> {
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    let removed = tx
        .state()
        .zones
        .find(ZoneKind::Removed, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no removed zone".into()))?;
    move_one_card(tx, card, processing, removed, PlayerId::NONE, true).map(|_| ())
}

pub(crate) fn resolve_jianrou(
    tx: &mut Transaction,
    player: PlayerId,
    damage_id: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if damage_id == 0 || tx.state().resolution.current_damage_id != damage_id {
        return Err(EffectError::ContentFault(
            "Jianrou used outside its damage_before event".into(),
        ));
    }
    let cards = crate::query::discardable_hand_equip_cards(tx.state(), player);
    GuardedChoice::new(
        skill,
        Participants::owner_any_state(player),
        ContinuationKind::ResolveJianrou,
        vec![player.raw(), damage_id, skill.raw()],
    )
    .shape(ChoiceShape::exactly(2))
    .open_cards(tx, cards)
}

pub(crate) fn resolve_oldhuisheng(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    damage_id: u32,
    stage: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if damage_id == 0 || tx.state().resolution.current_damage_id != damage_id {
        return Err(EffectError::ContentFault(
            "old Huisheng used outside its damage_before event".into(),
        ));
    }
    if !originating_skill_is_enabled(tx, player, skill)
        || source == player
        || !tx
            .state()
            .players
            .get(source)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if options.is_empty()
                || tx
                    .state()
                    .skills
                    .get(skill)
                    .is_some_and(|skill| skill.oldhuisheng_sources().contains(&source))
            {
                return Ok(Vec::new());
            }
            tx.state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::EngineFault("old Huisheng skill disappeared".into()))?
                .add_oldhuisheng_source(source);
            open_choice(
                tx,
                player,
                1,
                u8::try_from(options.len()).unwrap_or(u8::MAX),
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveOldHuisheng,
                vec![0, player.raw(), source.raw(), damage_id, skill.raw()],
            )
        }
        1 => run_effects(
            tx,
            vec![
                EffectRequest::RevealCards {
                    cards: cards.clone(),
                    to: vec![source],
                },
                EffectRequest::ResolveOldHuisheng {
                    player,
                    source,
                    damage_id,
                    stage: 2,
                    cards,
                    skill,
                },
            ],
        ),
        2 => {
            let can_discard =
                crate::query::discardable_hand_equip_cards(tx.state(), source).len() >= cards.len();
            let mut payload = vec![
                1,
                player.raw(),
                source.raw(),
                damage_id,
                skill.raw(),
                cards.len() as u32,
            ];
            payload.extend(cards.iter().map(|card| card.raw()));
            open_choice(
                tx,
                source,
                1,
                1,
                can_discard,
                cards.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveOldHuisheng,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault(
            "invalid old Huisheng stage".into(),
        )),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_oldjiefan(
    tx: &mut Transaction,
    player: PlayerId,
    dying: PlayerId,
    current: PlayerId,
    card: CardId,
    damage_id: u32,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || dying == player
                || current == player
                || current != tx.state().turn.current
                || !tx
                    .state()
                    .players
                    .get(dying)
                    .is_some_and(|target| target.alive && target.dying && target.hp <= 0)
            {
                return Ok(Vec::new());
            }
            let options =
                crate::command::play::oldjiefan_sha_source_cards(tx.state(), player, current);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveOldJiefan,
                vec![player.raw(), dying.raw(), current.raw(), skill.raw()],
            )
        }
        1 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !crate::command::play::oldjiefan_sha_source_cards(tx.state(), player, current)
                    .contains(&card)
            {
                return Ok(Vec::new());
            }
            tx.state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::EngineFault("old Jiefan skill disappeared".into()))?
                .set_oldjiefan_dying(dying);
            crate::command::play::play_oldjiefan_sha_source(tx, player, current, card)?;
            Ok(Vec::new())
        }
        2 => {
            if damage_id == 0 || tx.state().resolution.current_damage_id != damage_id {
                return Err(EffectError::ContentFault(
                    "old Jiefan recovery used outside its damage_before event".into(),
                ));
            }
            let Some(dying) = tx
                .state()
                .skills
                .get(skill)
                .and_then(|skill| skill.oldjiefan_dying())
            else {
                return Ok(Vec::new());
            };
            let mut effects = vec![EffectRequest::PreventDamage { damage_id }];
            let can_use_tao = tx
                .state()
                .players
                .get(player)
                .is_some_and(|owner| owner.alive && !owner.dying)
                && tx
                    .state()
                    .players
                    .get(dying)
                    .is_some_and(|target| target.alive && target.hp < target.max_hp as i32)
                && !crate::query::card_save_cancelled_without_hand_source(
                    tx.state(),
                    player,
                    dying,
                    CardKind::Tao,
                );
            if can_use_tao {
                let tao = materialize_virtual_card(tx, player, CardKind::Tao)?;
                effects.push(EffectRequest::UseCard {
                    player,
                    card: tao,
                    targets: vec![dying],
                });
                effects.push(EffectRequest::RecoverHp {
                    source: player,
                    player: dying,
                    amount: 1,
                });
            }
            run_effects(tx, effects)
        }
        _ => Err(EffectError::ContentFault("invalid old Jiefan stage".into())),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_chegu(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    cards: Vec<CardId>,
    branch: u8,
    index: u8,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) || tx.state().turn.current != player {
        return Ok(Vec::new());
    }
    let discard_targets = |state: &crate::state::GameState| {
        state
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|&target| {
                !crate::query::discardable_target_cards(
                    state,
                    player,
                    target,
                    rule_api::TargetCardScope::HandEquip,
                )
                .is_empty()
            })
            .collect::<Vec<_>>()
    };
    match stage {
        0 => {
            if card.is_none() || targets.is_empty() {
                return Err(EffectError::ContentFault(
                    "Chegu requires the triggering card and its targets".into(),
                ));
            }
            crate::command::play::exclude_all_saved_card_targets(tx, player, card, &targets)?;
            if discard_targets(tx.state()).is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                vec![1, 2],
                ChoiceKind::Custom,
                ContinuationKind::ResolveChegu,
                vec![0, player.raw(), card.raw(), skill.raw()],
            )
        }
        1 => {
            if !matches!(branch, 1 | 2) {
                return Err(EffectError::ContentFault("invalid Chegu branch".into()));
            }
            let options = discard_targets(tx.state());
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let bonus = tx
                .state()
                .skills
                .get(skill)
                .map_or(0, |skill| skill.chegu_bonus(tx.state().turn.round, player));
            let max = if branch == 1 {
                1
            } else {
                2u32.saturating_add(bonus)
                    .min(options.len() as u32)
                    .min(u8::MAX as u32) as u8
            };
            open_choice(
                tx,
                player,
                1,
                max,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveChegu,
                vec![1, player.raw(), card.raw(), u32::from(branch), skill.raw()],
            )
        }
        2 => {
            if usize::from(index) >= targets.len() {
                if cards.is_empty() {
                    return Ok(Vec::new());
                }
                let attributes = cards
                    .iter()
                    .filter_map(|card| tx.state().cards.get(*card))
                    .map(|card| {
                        let color = match card.suit {
                            engine_api::Suit::Heart | engine_api::Suit::Diamond => 1,
                            engine_api::Suit::Spade | engine_api::Suit::Club => 2,
                            engine_api::Suit::None => 0,
                        };
                        let category = if card.kind.is_basic() {
                            1
                        } else if card.kind.is_trick() {
                            2
                        } else if card.kind.is_equip() {
                            3
                        } else {
                            4
                        };
                        (color, category)
                    })
                    .collect::<Vec<_>>();
                let uniform = attributes.len() == cards.len()
                    && attributes.first().is_some_and(|first| {
                        attributes.iter().all(|value| value.0 == first.0)
                            || attributes.iter().all(|value| value.1 == first.1)
                    });
                if uniform {
                    let round = tx.state().turn.round;
                    tx.state_mut()
                        .skills
                        .get_mut(skill)
                        .ok_or_else(|| EffectError::EngineFault("Chegu skill disappeared".into()))?
                        .add_chegu_bonus(round, player);
                }
                return Ok(Vec::new());
            }
            let target = targets[usize::from(index)];
            let options = crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::HandEquip,
            );
            if options.is_empty() {
                return resolve_chegu(
                    tx,
                    player,
                    card,
                    targets,
                    cards,
                    branch,
                    index.saturating_add(1),
                    2,
                    skill,
                );
            }
            let bonus = tx
                .state()
                .skills
                .get(skill)
                .map_or(0, |skill| skill.chegu_bonus(tx.state().turn.round, player));
            let max = if branch == 1 {
                2u32.saturating_add(bonus)
                    .min(options.len() as u32)
                    .min(u8::MAX as u32) as u8
            } else {
                1
            };
            let mut payload = vec![
                2,
                player.raw(),
                card.raw(),
                u32::from(branch),
                u32::from(index),
                skill.raw(),
                targets.len() as u32,
            ];
            payload.extend(targets.iter().map(|target| target.raw()));
            payload.push(cards.len() as u32);
            payload.extend(cards.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                max,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveChegu,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault("invalid Chegu stage".into())),
    }
}

pub(crate) fn resolve_junkguixin(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    group: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let lord_skills = crate::query::gainable_lord_skill_defs(tx.state(), player);
    let other_players = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != player)
        .collect::<Vec<_>>();
    match stage {
        0 => match (!lord_skills.is_empty(), !other_players.is_empty()) {
            (true, true) => open_choice(
                tx,
                player,
                1,
                1,
                false,
                vec![1, 2],
                ChoiceKind::Custom,
                ContinuationKind::ResolveJunkguixin,
                vec![0, player.raw(), skill.raw()],
            ),
            (true, false) => resolve_junkguixin(tx, player, target, 1, 0, skill),
            (false, true) => resolve_junkguixin(tx, player, target, 2, 0, skill),
            (false, false) => Ok(Vec::new()),
        },
        1 => open_choice(
            tx,
            player,
            1,
            1,
            false,
            lord_skills
                .into_iter()
                .map(engine_api::SkillDefId::raw)
                .collect(),
            ChoiceKind::Custom,
            ContinuationKind::ResolveJunkguixin,
            vec![1, player.raw(), skill.raw()],
        ),
        2 => open_choice(
            tx,
            player,
            1,
            1,
            false,
            other_players.into_iter().map(PlayerId::raw).collect(),
            ChoiceKind::SelectTargets,
            ContinuationKind::ResolveJunkguixin,
            vec![2, player.raw(), skill.raw()],
        ),
        3 => {
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|target_state| target_state.alive && target != player)
            {
                return Ok(Vec::new());
            }
            let current = tx.state().player_group(target);
            let options = (1..=8)
                .filter(|group| junkguixin_group(*group) != current)
                .collect();
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::Custom,
                ContinuationKind::ResolveJunkguixin,
                vec![3, player.raw(), target.raw(), skill.raw()],
            )
        }
        4 => {
            let Some(group) = junkguixin_group(u32::from(group)) else {
                return Err(EffectError::ContentFault(
                    "invalid Junkguixin faction".into(),
                ));
            };
            if tx
                .state()
                .players
                .get(target)
                .is_some_and(|target_state| target_state.alive && target != player)
                && tx.state().player_group(target) != Some(group)
            {
                tx.state_mut().group_overrides.insert(target, group.into());
            }
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault("invalid Junkguixin stage".into())),
    }
}

pub(crate) fn resolve_yao_chenwei(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    mode: u8,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let mode = tx
                .state()
                .skills
                .get(skill)
                .map_or(0, |skill| skill.view_as_branch.min(1));
            let targets = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| {
                    mode == 0
                        || *target != player
                            && !crate::query::discardable_target_cards(
                                tx.state(),
                                player,
                                *target,
                                rule_api::TargetCardScope::HandEquip,
                            )
                            .is_empty()
                })
                .collect::<Vec<_>>();
            if targets.is_empty() || mode == 0 && tx.state().hand_cards(player).is_empty() {
                return Ok(Vec::new());
            }
            tx.state_mut()
                .skills
                .get_mut(skill)
                .expect("validated skill")
                .view_as_branch = 1 - mode;
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                targets.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveYaoChenwei,
                vec![0, player.raw(), u32::from(mode), skill.raw()],
            )
        }
        1 => {
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|target| target.alive)
            {
                return Ok(Vec::new());
            }
            let (observer, options, choice_stage) = if mode == 0 {
                (target, tx.state().hand_cards(player), 1)
            } else {
                (
                    player,
                    crate::query::discardable_target_cards(
                        tx.state(),
                        player,
                        target,
                        rule_api::TargetCardScope::HandEquip,
                    ),
                    2,
                )
            };
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                observer,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveYaoChenwei,
                vec![choice_stage, player.raw(), target.raw(), skill.raw()],
            )
        }
        2 if mode == 0 => {
            if !tx.state().hand_cards(player).contains(&card) {
                return Err(EffectError::Illegal(
                    "Yao Chenwei hand card became illegal".into(),
                ));
            }
            if !tx.state_mut().face_down_hand_cards.remove(&card) {
                tx.state_mut().face_down_hand_cards.insert(card);
            }
            Ok(Vec::new())
        }
        2 if mode == 1 => {
            if !crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::HandEquip,
            )
            .contains(&card)
            {
                return Err(EffectError::Illegal(
                    "Yao Chenwei gain card became illegal".into(),
                ));
            }
            let from = tx
                .state()
                .cards
                .get(card)
                .map(|card| card.zone)
                .ok_or_else(|| EffectError::Illegal("Yao Chenwei card disappeared".into()))?;
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("owner hand missing".into()))?;
            let events = move_cards(tx, vec![card], hand, Some(from))?;
            tx.state_mut().face_down_hand_cards.insert(card);
            Ok(events)
        }
        _ => Err(EffectError::ContentFault(
            "invalid Yao Chenwei stage".into(),
        )),
    }
}

pub(crate) fn junkguixin_group(group: u32) -> Option<&'static str> {
    ["wei", "shu", "wu", "qun", "jin", "key", "western", "devil"]
        .get(group.checked_sub(1)? as usize)
        .copied()
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_drlt_jueyan(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    slot: u8,
    first: bool,
    sha_skill_def: engine_api::SkillDefId,
    weapon_skill_def: engine_api::SkillDefId,
    mount_skill_def: engine_api::SkillDefId,
    armor_skill_def: engine_api::SkillDefId,
    treasure_skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let Some(participant) = tx.state().players.get(player) else {
                return Ok(Vec::new());
            };
            let mut options = Vec::new();
            if participant.equip_slot_enabled(EquipSlot::Weapon) {
                options.push(1);
            }
            if participant.equip_slot_enabled(EquipSlot::Armor) {
                options.push(2);
            }
            if participant.equip_slot_enabled(EquipSlot::DefensiveMount)
                || participant.equip_slot_enabled(EquipSlot::OffensiveMount)
            {
                options.push(3);
            }
            if participant.equip_slot_enabled(EquipSlot::Treasure) {
                options.push(5);
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
                ContinuationKind::ResolveDrltJueyan,
                vec![
                    player.raw(),
                    sha_skill_def.raw(),
                    weapon_skill_def.raw(),
                    mount_skill_def.raw(),
                    armor_skill_def.raw(),
                    treasure_skill_def.raw(),
                    skill.raw(),
                ],
            )
        }
        1 => {
            let slots = if slot == 3 { vec![3, 4] } else { vec![slot] };
            let first = tx
                .state()
                .skills
                .get(skill)
                .is_some_and(|instance| instance.state_blob.is_empty());
            if first {
                tx.state_mut()
                    .skills
                    .get_mut(skill)
                    .expect("validated Jueyan skill")
                    .state_blob = b"JYN1".to_vec();
            }
            tx.state_mut().resolution.deferred_effects.insert(
                0,
                EffectRequest::ResolveDrltJueyan {
                    player,
                    stage: 2,
                    slot,
                    first,
                    sha_skill_def,
                    weapon_skill_def,
                    mount_skill_def,
                    armor_skill_def,
                    treasure_skill_def,
                    skill,
                },
            );
            disable_equip_slots(tx, player, slots)
        }
        2 => match slot {
            1 => {
                let mut events = add_skill_until_turn_end(tx, player, weapon_skill_def)?;
                if first {
                    events.extend(add_skill(tx, player, sha_skill_def)?);
                }
                Ok(events)
            }
            2 => {
                tx.state_mut().resolution.deferred_effects.insert(
                    0,
                    EffectRequest::ResolveDrltJueyan {
                        player,
                        stage: 3,
                        slot,
                        first,
                        sha_skill_def,
                        weapon_skill_def,
                        mount_skill_def,
                        armor_skill_def,
                        treasure_skill_def,
                        skill,
                    },
                );
                draw_cards(tx, player, 3)
            }
            3 => {
                if first {
                    add_skill(tx, player, mount_skill_def)
                } else {
                    add_skill_until_turn_end(tx, player, mount_skill_def)
                }
            }
            5 => {
                if first {
                    add_skill(tx, player, treasure_skill_def)
                } else {
                    add_skill_until_turn_end(tx, player, treasure_skill_def)
                }
            }
            _ => Err(EffectError::ContentFault("invalid Drlt Jueyan slot".into())),
        },
        3 if slot == 2 => {
            if first {
                add_skill(tx, player, armor_skill_def)
            } else {
                add_skill_until_turn_end(tx, player, armor_skill_def)
            }
        }
        _ => Err(EffectError::ContentFault(
            "invalid Drlt Jueyan stage".into(),
        )),
    }
}

pub(crate) fn resolve_rejizhi(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let events = draw_cards(tx, player, 1)?;
            let card = events.iter().find_map(|event| match event {
                DomainEvent::CardsDrawn {
                    player: owner,
                    cards,
                } if *owner == player => cards.first().copied(),
                _ => None,
            });
            if card.is_some_and(|card| {
                tx.state()
                    .cards
                    .get(card)
                    .is_some_and(|state| state.kind.is_basic())
            }) {
                tx.state_mut().resolution.deferred_tail_effects.push(
                    EffectRequest::ResolveRejizhi {
                        player,
                        stage: 1,
                        card: card.unwrap(),
                        skill,
                    },
                );
            }
            Ok(events)
        }
        1 => {
            let in_hand = tx.state().cards.get(card).is_some_and(|card_state| {
                card_state.owner == player
                    && tx
                        .state()
                        .zones
                        .get(card_state.zone)
                        .is_some_and(|zone| zone.kind == ZoneKind::Hand)
            });
            if !in_hand {
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
                ContinuationKind::ResolveRejizhi,
                vec![player.raw(), card.raw(), skill.raw()],
            )
        }
        2 => {
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveRejizhi {
                    player,
                    stage: 3,
                    card: CardId::NONE,
                    skill,
                });
            discard_cards(tx, player, vec![card])
        }
        3 => {
            let mark = tx
                .state()
                .skills
                .get(skill)
                .map(|instance| instance.def.raw())
                .ok_or_else(|| EffectError::EngineFault("Rejizhi skill disappeared".into()))?;
            add_mark(tx, player, mark, 1)
        }
        _ => Err(EffectError::ContentFault("invalid Rejizhi stage".into())),
    }
}

pub(crate) fn count_field_diamonds(tx: &Transaction) -> u32 {
    tx.state()
        .players
        .alive_by_seat()
        .into_iter()
        .flat_map(|player| {
            [ZoneKind::Equip, ZoneKind::Judge]
                .into_iter()
                .filter_map(move |kind| {
                    tx.state()
                        .zones
                        .find(kind, player)
                        .map(|zone| (player, zone))
                })
        })
        .flat_map(|(player, zone)| {
            tx.state()
                .zones
                .get(zone)
                .into_iter()
                .flat_map(|zone| zone.cards.iter())
                .copied()
                .map(move |card| (player, card))
        })
        .filter(|&(player, card)| {
            tx.state().cards.get(card).is_some_and(|card| {
                crate::query::effective_card_suit(tx.state(), player, card) == Suit::Diamond
            })
        })
        .count()
        .min(u32::MAX as usize) as u32
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_jiexun(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    target: PlayerId,
    draw_count: u32,
    discard_count: u32,
    funan_helper_skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
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
            let mark = tx
                .state()
                .skills
                .get(skill)
                .map(|instance| instance.def.raw())
                .ok_or_else(|| EffectError::EngineFault("Jiexun skill disappeared".into()))?;
            let discard_count = tx
                .state()
                .players
                .get(player)
                .into_iter()
                .flat_map(|player| player.marks.iter())
                .find(|(existing, _)| *existing == mark)
                .map(|(_, count)| (*count).max(0) as u32)
                .unwrap_or(0);
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveJiexun,
                vec![
                    0,
                    player.raw(),
                    count_field_diamonds(tx),
                    discard_count,
                    funan_helper_skill_def.raw(),
                    skill.raw(),
                ],
            )
        }
        1 => {
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveJiexun {
                    player,
                    stage: 2,
                    target,
                    draw_count,
                    discard_count,
                    funan_helper_skill_def,
                    skill,
                });
            if draw_count == 0 {
                return Ok(Vec::new());
            }
            draw_cards(tx, target, draw_count)
        }
        2 => {
            let mark = tx
                .state()
                .skills
                .get(skill)
                .map(|instance| instance.def.raw())
                .ok_or_else(|| EffectError::EngineFault("Jiexun skill disappeared".into()))?;
            let mut events = add_mark(tx, player, mark, 1)?;
            if discard_count == 0 {
                return Ok(events);
            }
            let options = crate::query::discardable_hand_equip_cards(tx.state(), target);
            let required = discard_count.min(options.len() as u32) as usize;
            if required == 0 {
                return Ok(events);
            }
            if required == options.len() {
                tx.state_mut().resolution.deferred_tail_effects.push(
                    EffectRequest::ResolveJiexun {
                        player,
                        stage: 3,
                        target,
                        draw_count,
                        discard_count,
                        funan_helper_skill_def,
                        skill,
                    },
                );
                events.extend(discard_cards(tx, target, options)?);
                return Ok(events);
            }
            let mut payload = vec![
                1,
                player.raw(),
                target.raw(),
                funan_helper_skill_def.raw(),
                skill.raw(),
                options.len().min(u32::MAX as usize) as u32,
                required as u32,
            ];
            payload.extend(options.iter().map(|card| card.raw()));
            events.extend(open_choice(
                tx,
                target,
                required as u8,
                required as u8,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveJiexun,
                payload,
            )?);
            Ok(events)
        }
        3 => {
            let mark = tx
                .state()
                .skills
                .get(skill)
                .map(|instance| instance.def.raw())
                .ok_or_else(|| EffectError::EngineFault("Jiexun skill disappeared".into()))?;
            let mut events = remove_mark(tx, player, mark, 0)?;
            events.extend(remove_skill(tx, skill)?);
            events.extend(add_skill(tx, player, funan_helper_skill_def)?);
            Ok(events)
        }
        _ => Err(EffectError::ContentFault("invalid Jiexun stage".into())),
    }
}

pub(crate) fn player_has_skill_def(
    tx: &Transaction,
    player: PlayerId,
    skill_def: engine_api::SkillDefId,
    enabled: bool,
) -> bool {
    tx.state()
        .players
        .get(player)
        .into_iter()
        .flat_map(|player| player.skills.iter())
        .filter_map(|skill| tx.state().skills.get(*skill))
        .any(|skill| {
            skill.def == skill_def && (!enabled || tx.state().skill_effectively_enabled(skill))
        })
}

pub(crate) fn resolve_wengua_setup(
    tx: &mut Transaction,
    player: PlayerId,
    helper_skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let mut events = Vec::new();
    for target in tx.state().players.alive_by_seat() {
        events.extend(add_skill(tx, target, helper_skill_def)?);
    }
    Ok(events)
}

pub(crate) fn draw_cards_from_bottom(
    tx: &mut Transaction,
    player: PlayerId,
    count: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("draw pile missing".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("hand missing".into()))?;
    let mut cards = Vec::new();
    for _ in 0..count {
        if tx
            .state()
            .zones
            .get(draw)
            .is_none_or(|zone| zone.cards.is_empty())
        {
            reshuffle_discard_into_draw(tx)?;
        }
        let Some(card) = tx
            .state()
            .zones
            .get(draw)
            .and_then(|zone| zone.cards.first().copied())
        else {
            break;
        };
        move_one_card(tx, card, draw, hand, player, false)?;
        cards.push(card);
    }
    Ok((!cards.is_empty())
        .then_some(DomainEvent::CardsDrawn { player, cards })
        .into_iter()
        .collect())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_wengua(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    target: PlayerId,
    card: CardId,
    branch: u8,
    main_skill_def: engine_api::SkillDefId,
    marker_skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            if !reanxu_cards(tx, player).contains(&card) {
                return Err(EffectError::Illegal(
                    "Wengua selected card became illegal".into(),
                ));
            }
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&target| {
                    player_has_skill_def(tx, target, main_skill_def, true)
                        && !player_has_skill_def(tx, target, marker_skill_def, false)
                })
                .collect();
            match options.as_slice() {
                [] => Ok(Vec::new()),
                [target] => resolve_wengua(
                    tx,
                    player,
                    1,
                    *target,
                    card,
                    0,
                    main_skill_def,
                    marker_skill_def,
                    skill,
                ),
                _ => open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    options.into_iter().map(PlayerId::raw).collect(),
                    ChoiceKind::SelectTargets,
                    ContinuationKind::ResolveWengua,
                    vec![
                        0,
                        player.raw(),
                        card.raw(),
                        main_skill_def.raw(),
                        marker_skill_def.raw(),
                        skill.raw(),
                    ],
                ),
            }
        }
        1 => {
            if !player_has_skill_def(tx, target, main_skill_def, true)
                || player_has_skill_def(tx, target, marker_skill_def, false)
                || !reanxu_cards(tx, player).contains(&card)
            {
                return Err(EffectError::Illegal(
                    "Wengua recipient became illegal".into(),
                ));
            }
            let mut events = add_skill(tx, target, marker_skill_def)?;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveWengua {
                    player,
                    stage: 2,
                    target,
                    card,
                    branch: 0,
                    main_skill_def,
                    marker_skill_def,
                    skill,
                });
            if target != player {
                let hand = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, target)
                    .ok_or_else(|| EffectError::EngineFault("Wengua target hand missing".into()))?;
                let from = tx.state().cards.get(card).unwrap().zone;
                events.extend(move_cards(tx, vec![card], hand, Some(from))?);
            }
            Ok(events)
        }
        2 => {
            if !reanxu_cards(tx, target).contains(&card) {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                target,
                if target == player { 1 } else { 0 },
                1,
                target != player,
                vec![0, 1],
                ChoiceKind::Custom,
                ContinuationKind::ResolveWengua,
                vec![
                    1,
                    player.raw(),
                    target.raw(),
                    card.raw(),
                    main_skill_def.raw(),
                    marker_skill_def.raw(),
                    skill.raw(),
                ],
            )
        }
        3 => {
            if !reanxu_cards(tx, target).contains(&card) {
                return Err(EffectError::Illegal(
                    "Wengua placement card became illegal".into(),
                ));
            }
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("draw pile missing".into()))?;
            let from = tx.state().cards.get(card).unwrap().zone;
            let events = move_cards(tx, vec![card], draw, Some(from))?;
            if branch == 1 {
                let draw_zone = tx.state_mut().zones.get_mut(draw).unwrap();
                let card = draw_zone.cards.pop().unwrap();
                draw_zone.cards.insert(0, card);
            }
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveWengua {
                    player,
                    stage: 4,
                    target,
                    card,
                    branch,
                    main_skill_def,
                    marker_skill_def,
                    skill,
                });
            Ok(events)
        }
        4 => {
            let one_draw = player == target
                || tx
                    .state()
                    .zones
                    .find(ZoneKind::DrawPile, PlayerId::NONE)
                    .and_then(|draw| tx.state().zones.get(draw))
                    .is_none_or(|zone| zone.cards.len() <= 1);
            let mut events = if branch == 0 {
                draw_cards_from_bottom(tx, player, 1)?
            } else {
                draw_cards(tx, player, 1)?
            };
            if !one_draw {
                events.extend(if branch == 0 {
                    draw_cards_from_bottom(tx, target, 1)?
                } else {
                    draw_cards(tx, target, 1)?
                });
            }
            Ok(events)
        }
        _ => Err(EffectError::ContentFault("invalid Wengua stage".into())),
    }
}

pub(crate) fn xinanguo_metric_is_min(tx: &Transaction, player: PlayerId, index: u8) -> bool {
    let value = |target| match index {
        0 => tx.state().hand_count(target) as i64,
        1 => tx
            .state()
            .players
            .get(target)
            .map(|player| i64::from(player.hp))
            .unwrap_or(i64::MAX),
        2 => tx
            .state()
            .zones
            .find(ZoneKind::Equip, target)
            .and_then(|zone| tx.state().zones.get(zone))
            .map(|zone| zone.cards.len() as i64)
            .unwrap_or(i64::MAX),
        _ => i64::MAX,
    };
    let current = value(player);
    tx.state()
        .players
        .alive_by_seat()
        .into_iter()
        .all(|target| current <= value(target))
}

pub(crate) fn equip_random_from_draw_pile(
    tx: &mut Transaction,
    player: PlayerId,
    required_slot: Option<EquipSlot>,
) -> Result<(Vec<DomainEvent>, bool), EffectError> {
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("draw pile missing".into()))?;
    let equip = tx
        .state()
        .zones
        .find(ZoneKind::Equip, player)
        .ok_or_else(|| EffectError::EngineFault("equip zone missing".into()))?;
    let mut candidates: Vec<_> = tx
        .state()
        .zones
        .get(draw)
        .into_iter()
        .flat_map(|zone| zone.cards.iter())
        .copied()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .and_then(|card| card.kind.equip_slot())
                .is_some_and(|slot| {
                    required_slot.is_none_or(|required| required == slot)
                        && tx
                            .state()
                            .players
                            .get(player)
                            .is_some_and(|player| player.equip_slot_enabled(slot))
                })
        })
        .collect();
    if candidates.is_empty() {
        return Ok((Vec::new(), false));
    }
    candidates.sort_by_key(|card| card.raw());
    let mut rng = RuleRng::from_state(tx.state().rng);
    let card = candidates[rng.gen_range_u32(candidates.len() as u32) as usize];
    tx.state_mut().rng = rng.into_state();
    let slot = tx
        .state()
        .cards
        .get(card)
        .unwrap()
        .kind
        .equip_slot()
        .unwrap();
    let existing: Vec<_> = tx
        .state()
        .zones
        .get(equip)
        .into_iter()
        .flat_map(|zone| zone.cards.iter())
        .copied()
        .filter(|existing| {
            tx.state()
                .cards
                .get(*existing)
                .and_then(|card| card.kind.equip_slot())
                == Some(slot)
        })
        .collect();
    let mut events = Vec::new();
    if !existing.is_empty() {
        let discard = tx
            .state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
        events.extend(move_cards(tx, existing, discard, Some(equip))?);
    }
    events.extend(move_cards(tx, vec![card], equip, Some(draw))?);
    Ok((events, true))
}

pub(crate) fn resolve_xinanguo(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    index: u8,
    remaining: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    resolve_anguo_flow(tx, player, target, stage, index, remaining, skill, false)
}

pub(crate) fn resolve_reanguo(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    index: u8,
    remaining: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    resolve_anguo_flow(tx, player, target, stage, index, remaining, skill, true)
}

pub(crate) fn resolve_beizhu(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    mut cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if stage == 0 {
        if !originating_skill_is_enabled(tx, player, skill) {
            return Ok(Vec::new());
        }
        if player == target
            || ![player, target].into_iter().all(|participant| {
                tx.state()
                    .players
                    .get(participant)
                    .is_some_and(|state| state.alive)
            })
            || tx.state().hand_cards(target).is_empty()
        {
            return Err(EffectError::Illegal(
                "Beizhu requires one living other target with hand cards".into(),
            ));
        }
        cards = tx
            .state()
            .hand_cards(target)
            .into_iter()
            .filter(|card| {
                tx.state().cards.get(*card).is_some_and(|card| {
                    crate::query::effective_card_kind(tx.state(), target, card).is_sha()
                })
            })
            .collect();
        if cards.is_empty() {
            let options = crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::HandEquip,
            );
            if options.is_empty() {
                return resolve_beizhu(tx, player, target, 1, Vec::new(), skill);
            }
            let mut payload = vec![
                0,
                player.raw(),
                target.raw(),
                skill.raw(),
                options.len() as u32,
            ];
            payload.extend(options.iter().map(|card| card.raw()));
            return open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveBeizhu,
                payload,
            );
        }
        let mut rng = RuleRng::from_state(tx.state().rng);
        rng.shuffle(&mut cards);
        tx.state_mut().rng = rng.into_state();
        return resolve_beizhu(tx, player, target, 2, cards, skill);
    }

    if stage == 1 {
        if ![player, target].into_iter().all(|participant| {
            tx.state()
                .players
                .get(participant)
                .is_some_and(|state| state.alive)
        }) {
            return Ok(Vec::new());
        }
        let draw = tx
            .state()
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::EngineFault("Beizhu draw pile is missing".into()))?;
        let sha = tx.state().zones.get(draw).and_then(|zone| {
            zone.cards.iter().rev().copied().find(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|card| card.kind.is_sha())
            })
        });
        let Some(sha) = sha else {
            return Ok(Vec::new());
        };
        return open_choice(
            tx,
            player,
            1,
            1,
            true,
            vec![1],
            ChoiceKind::Custom,
            ContinuationKind::ResolveBeizhu,
            vec![
                1,
                player.raw(),
                target.raw(),
                sha.raw(),
                draw.raw(),
                skill.raw(),
            ],
        );
    }

    if stage != 2 {
        return Err(EffectError::ContentFault(
            "unknown Beizhu resolution stage".into(),
        ));
    }
    while let Some(card) = cards.first().copied() {
        cards.remove(0);
        if !crate::command::play::beizhu_sha_usable(tx, target, player, card) {
            continue;
        }
        if !cards.is_empty() {
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveBeizhu {
                    player,
                    target,
                    stage: 2,
                    cards,
                    skill,
                });
        }
        crate::command::play::play_beizhu_sha_source(tx, target, player, card)?;
        return Ok(Vec::new());
    }
    Ok(Vec::new())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_anguo_flow(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    index: u8,
    remaining: u8,
    skill: engine_api::SkillId,
    allow_recast: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    if stage == 0 {
        if target == player
            || !tx
                .state()
                .players
                .get(target)
                .is_some_and(|target| target.alive)
        {
            return Err(EffectError::Illegal(
                "Anguo requires one living other target".into(),
            ));
        }
        return resolve_anguo_flow(tx, player, target, 1, 0, 0b111, skill, allow_recast);
    }
    if index >= 3 {
        if stage == 1 {
            return resolve_anguo_flow(tx, player, target, 2, 0, remaining, skill, allow_recast);
        }
        if allow_recast && remaining == 0 {
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            let equip = tx.state().zones.find(ZoneKind::Equip, player);
            let mut options: Vec<_> = [hand, equip]
                .into_iter()
                .flatten()
                .flat_map(|zone| {
                    tx.state()
                        .zones
                        .get(zone)
                        .into_iter()
                        .flat_map(|zone| zone.cards.iter().copied())
                })
                .collect();
            options.sort_by_key(|card| card.raw());
            options.dedup();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let max = options.len().min(u8::MAX as usize) as u8;
            let mut payload = vec![player.raw(), skill.raw(), options.len() as u32];
            payload.extend(options.iter().map(|card| card.raw()));
            return open_choice(
                tx,
                player,
                1,
                max,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveReanguo,
                payload,
            );
        }
        return Ok(Vec::new());
    }
    let actor = if stage == 1 { target } else { player };
    let bit = 1u8 << index;
    if remaining & bit == 0 {
        return resolve_anguo_flow(
            tx,
            player,
            target,
            stage,
            index + 1,
            remaining,
            skill,
            allow_recast,
        );
    }
    let condition = xinanguo_metric_is_min(tx, actor, index)
        && (index != 1
            || tx
                .state()
                .players
                .get(actor)
                .is_some_and(|actor| actor.hp < actor.max_hp as i32));
    if !condition {
        return resolve_anguo_flow(
            tx,
            player,
            target,
            stage,
            index + 1,
            remaining,
            skill,
            allow_recast,
        );
    }
    let (events, performed) = match index {
        0 => (draw_cards(tx, actor, 1)?, true),
        1 => (recover_hp(tx, player, actor, 1)?, true),
        2 => equip_random_from_draw_pile(tx, actor, None)?,
        _ => unreachable!(),
    };
    if !performed {
        return resolve_anguo_flow(
            tx,
            player,
            target,
            stage,
            index + 1,
            remaining,
            skill,
            allow_recast,
        );
    }
    let remaining = remaining & !bit;
    let next = if allow_recast {
        EffectRequest::ResolveReanguo {
            player,
            target,
            stage,
            index: index + 1,
            remaining,
            skill,
        }
    } else {
        EffectRequest::ResolveXinanguo {
            player,
            target,
            stage,
            index: index + 1,
            remaining,
            skill,
        }
    };
    tx.state_mut().resolution.deferred_tail_effects.push(next);
    Ok(events)
}

pub(crate) fn resolve_ganglie_three(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let enemies: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|candidate| {
            matches!(
                crate::query::query(
                    tx.state(),
                    &QueryRequest::Relation {
                        a: player,
                        b: *candidate,
                    },
                    &mut crate::query::QueryContext::default(),
                ),
                QueryResponse::Relation(RelationTag::Enemy)
            )
        })
        .collect();
    if target.is_none() {
        return GuardedChoice::new(
            skill,
            Participants::owner_any_state(player),
            ContinuationKind::ResolveGanglieThree,
            vec![player.raw(), skill.raw()],
        )
        .open_targets(tx, enemies);
    }
    if !enemies.contains(&target) {
        return Err(EffectError::Illegal(
            "Ganglie Three target must be a living enemy".into(),
        ));
    }
    judge_target_discard_two_or_damage(tx, player, target, false)
}

pub(crate) fn resolve_zhongyi(
    tx: &mut Transaction,
    player: PlayerId,
    action: u8,
    helper_skill_def: engine_api::SkillDefId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            let card_state =
                tx.state().cards.get(card).ok_or_else(|| {
                    EffectError::Illegal("Zhongyi selected card is missing".into())
                })?;
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            let equip = tx.state().zones.find(ZoneKind::Equip, player);
            if card_state.owner != player
                || (Some(card_state.zone) != hand && Some(card_state.zone) != equip)
                || !tx
                    .state()
                    .skill_pile_cards(player, helper_skill_def.raw())
                    .is_empty()
            {
                return Err(EffectError::Illegal(
                    "Zhongyi requires one current hand/equipment card and an empty pile".into(),
                ));
            }
            let from = card_state.zone;
            let round = tx.state().turn.round;
            let pile = ensure_skill_pile_zone(tx, player, helper_skill_def.raw())?;
            let mut events = add_skill_with_state(
                tx,
                player,
                helper_skill_def,
                Some(round.to_le_bytes().into()),
            )?;
            events.extend(move_cards(tx, vec![card], pile, Some(from))?);
            Ok(events)
        }
        1 => {
            let valid = tx.state().skills.get(skill).is_some_and(|instance| {
                instance.owner == player
                    && instance.def == helper_skill_def
                    && tx.state().skill_effectively_enabled(instance)
            }) && !tx
                .state()
                .skill_pile_cards(player, helper_skill_def.raw())
                .is_empty();
            if !valid {
                return Ok(Vec::new());
            }
            let damage_id = tx.state().resolution.current_damage_id;
            increase_bound_damage(tx, damage_id, 1)?;
            Ok(Vec::new())
        }
        2 => {
            let Some(instance) = tx.state().skills.get(skill) else {
                return Ok(Vec::new());
            };
            if instance.owner != player
                || instance.def != helper_skill_def
                || instance.state_blob.len() != 4
            {
                return Ok(Vec::new());
            }
            let activated_round =
                u32::from_le_bytes(instance.state_blob.as_slice().try_into().unwrap());
            if tx.state().turn.round <= activated_round {
                return Ok(Vec::new());
            }
            let cards = tx.state().skill_pile_cards(player, helper_skill_def.raw());
            let mut events = if cards.is_empty() {
                Vec::new()
            } else {
                let pile = ensure_skill_pile_zone(tx, player, helper_skill_def.raw())?;
                let discard = tx
                    .state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
                move_cards(tx, cards, discard, Some(pile))?
            };
            events.extend(remove_skill(tx, skill)?);
            Ok(events)
        }
        _ => Err(EffectError::ContentFault("unknown Zhongyi action".into())),
    }
}

pub(crate) fn discard_equipment_slot(
    tx: &mut Transaction,
    player: PlayerId,
    slot: u8,
) -> Result<Vec<DomainEvent>, EffectError> {
    let slot = EquipSlot::from_tag(slot)
        .ok_or_else(|| EffectError::ContentFault("invalid equipment slot".into()))?;
    let Some(equip) = tx.state().zones.find(ZoneKind::Equip, player) else {
        return Ok(Vec::new());
    };
    let cards: Vec<_> = tx
        .state()
        .zones
        .get(equip)
        .into_iter()
        .flat_map(|zone| zone.cards.iter())
        .copied()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .and_then(|card| card.kind.equip_slot())
                == Some(slot)
        })
        .collect();
    if cards.is_empty() {
        Ok(Vec::new())
    } else {
        discard_cards(tx, player, cards)
    }
}

pub(crate) fn record_active_target(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let state = tx.state();
    let instance = state
        .skills
        .get(skill)
        .ok_or_else(|| EffectError::ContentFault("active target recorder missing skill".into()))?;
    let contract = state
        .content
        .first_active_for_skill(instance.def)
        .ok_or_else(|| {
            EffectError::ContentFault("active target recorder missing contract".into())
        })?;
    let target_contract = contract.targets.as_ref().ok_or_else(|| {
        EffectError::ContentFault("active target recorder missing targets".into())
    })?;
    let turn_scoped = target_contract.unused_by_skill_this_turn;
    let play_phase_scoped = target_contract.unused_by_skill_this_play_phase;
    if instance.owner != player
        || !state.skill_effectively_enabled(instance)
        || state.turn.current != player
        || state.turn.phase != Phase::Play
        || turn_scoped == play_phase_scoped
    {
        return Err(EffectError::ContentFault(
            "active target recorder requires an unused-target contract".into(),
        ));
    }
    if (target == player && !target_contract.allow_self)
        || !state
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "active target recorder requires another living target".into(),
        ));
    }
    let recorded = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .is_some_and(|instance| {
            if play_phase_scoped {
                instance.record_active_play_phase_target(target)
            } else {
                instance.record_active_target(target)
            }
        });
    if !recorded {
        return Err(EffectError::Illegal(
            "active target was already selected in this uniqueness window".into(),
        ));
    }
    Ok(Vec::new())
}

pub(crate) fn resolve_dingpin(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::ContentFault(
            "Dingpin requires its enabled owner and a living Play-phase target".into(),
        ));
    }
    let result = crate::command::judge::flip_judgment(
        tx,
        target,
        crate::command::judge::JudgmentFavor::Neutral,
    )?;
    let judgment_event = completed_judgment_event(target, result, CardId::NONE);
    tx.push_event(DomainEvent::RuleNote {
        code: 232,
        payload: vec![
            target.raw(),
            crate::command::judge::suit_tag(result.suit),
            u32::from(result.rank),
        ],
    });
    match result.suit {
        Suit::Spade | Suit::Club => {
            record_active_target(tx, player, target, skill)?;
            let lost_hp = tx
                .state()
                .players
                .get(target)
                .map(|participant| {
                    (i64::from(participant.max_hp) - i64::from(participant.hp)).max(0) as u32
                })
                .unwrap_or(0);
            draw_cards(tx, target, lost_hp).map(|events| prepend_event(judgment_event, events))
        }
        Suit::Heart | Suit::Diamond => {
            turn_over(tx, player).map(|events| prepend_event(judgment_event, events))
        }
        Suit::None => Ok(vec![judgment_event]),
    }
}

pub(crate) fn discardable_weapon_cards(tx: &Transaction, player: PlayerId) -> Vec<CardId> {
    crate::query::discardable_hand_equip_cards(tx.state(), player)
        .into_iter()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|card| card.kind == CardKind::Weapon)
        })
        .collect()
}

pub(crate) fn choose_discard_weapon_or_lose_hp(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
    {
        return Err(EffectError::Illegal(
            "invalid discard-weapon-or-lose-HP context".into(),
        ));
    }
    let can_lose_hp = tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive && participant.hp >= 1);
    let weapons = discardable_weapon_cards(tx, player);
    match (can_lose_hp, weapons.is_empty()) {
        (false, true) => Err(EffectError::Illegal(
            "discard-weapon-or-lose-HP cost is unavailable".into(),
        )),
        (true, true) => deal_damage(
            tx,
            PlayerId::NONE,
            player,
            1,
            true,
            rule_api::damage_nature::NORMAL,
            false,
            CardId::NONE,
            0,
            0,
        ),
        (false, false) => open_choice(
            tx,
            player,
            1,
            1,
            false,
            weapons.into_iter().map(CardId::raw).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::ChooseDiscardWeaponOrLoseHp,
            vec![player.raw(), skill.raw(), 1],
        ),
        (true, false) => open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![0, 1],
            ChoiceKind::Custom,
            ContinuationKind::ChooseDiscardWeaponOrLoseHp,
            vec![player.raw(), skill.raw(), 0],
        ),
    }
}

/// 弃置一张手牌/装备牌，或失去 `lose_amount` 点体力（庸肆 xinyongsi2 弃一张或失去 1 点）。
/// 与 `choose_discard_weapon_or_lose_hp` 同构：可选弃牌时开二选一（0=失去体力,1=选牌），
/// 无可弃牌时直接失去体力。Choice 恢复时重验上下文。
pub(crate) fn choose_discard_one_he_or_lose_hp(
    tx: &mut Transaction,
    player: PlayerId,
    lose_amount: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
    {
        return Err(EffectError::Illegal(
            "invalid discard-one-or-lose-HP context".into(),
        ));
    }
    let can_lose_hp = tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive && participant.hp >= lose_amount as i32);
    let discardable = crate::query::discardable_hand_equip_cards(tx.state(), player);
    match (can_lose_hp, discardable.is_empty()) {
        (false, true) => Err(EffectError::Illegal(
            "discard-one-or-lose-HP cost is unavailable".into(),
        )),
        (true, true) => deal_damage(
            tx,
            PlayerId::NONE,
            player,
            lose_amount,
            true,
            rule_api::damage_nature::NORMAL,
            false,
            CardId::NONE,
            0,
            0,
        ),
        (false, false) => open_choice(
            tx,
            player,
            1,
            1,
            false,
            discardable.into_iter().map(CardId::raw).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::ChooseDiscardOneHeOrLoseHp,
            vec![player.raw(), skill.raw(), lose_amount, 1],
        ),
        (true, false) => open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![0, 1],
            ChoiceKind::Custom,
            ContinuationKind::ChooseDiscardOneHeOrLoseHp,
            vec![player.raw(), skill.raw(), lose_amount, 0],
        ),
    }
}

/// 随势（suishi2）：其他角色死亡后，其伤害来源选择是否令拥有者失去 1 点体力。
/// 上游：trigger.source.chooseBool() 确认后 player.loseHp()。Choice 恢复时重验来源存活。
pub(crate) fn choose_source_confirm_owner_lose_hp(
    tx: &mut Transaction,
    owner: PlayerId,
    source: PlayerId,
    amount: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, owner, skill)
        || source == PlayerId::NONE
        || source == owner
        || !tx
            .state()
            .players
            .get(owner)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "invalid source-confirm-owner-lose-HP context".into(),
        ));
    }
    open_choice(
        tx,
        source,
        0,
        1,
        false,
        vec![1],
        ChoiceKind::Confirm,
        ContinuationKind::ChooseSourceConfirmOwnerLoseHp,
        vec![owner.raw(), source.raw(), skill.raw(), amount],
    )
}

/// 峻攻：弃置至多 `use_count + 1` 张手牌/装备牌或失去等量体力，然后对目标使用虚拟【杀】。
///
/// 旧 JS 的可用过滤 `num < hp || num <= countCards("he")` 与选牌 `[0, num]`（选 0 张即
/// 失去体力）由本原语校验并在 Choice 恢复时重验；虚拟【杀】沿普通结算队列运行，命中目标
/// 时由配套触发 `owner_source_sha_damage_recorded_by_skill` 禁用本主动技至回合结束。
pub(crate) fn spjungong_discard_or_lose_then_sha(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "invalid 峻攻 discard-or-lose context".into(),
        ));
    }
    let instance = tx
        .state()
        .skills
        .get(skill)
        .ok_or_else(|| EffectError::ContentFault("峻攻 active skill missing".into()))?;
    let n = instance.use_count.saturating_add(1);
    let hp = tx
        .state()
        .players
        .get(player)
        .map(|participant| participant.hp)
        .unwrap_or(0);
    let cards = crate::query::discardable_target_cards(
        tx.state(),
        player,
        player,
        rule_api::TargetCardScope::HandEquip,
    );
    if !(hp > n as i32) && cards.len() < n as usize {
        // 旧 JS filter：num < hp || num <= countCards("he")。
        return Ok(Vec::new());
    }
    let max_select = (n as usize).min(cards.len()).min(u8::MAX as usize) as u8;
    open_choice(
        tx,
        player,
        0,
        max_select,
        false,
        cards.into_iter().map(CardId::raw).collect(),
        engine_api::ChoiceKind::SelectCards,
        ContinuationKind::ChooseSpjungongDiscardOrLose,
        vec![player.raw(), skill.raw(), target.raw(), n],
    )
}

/// 把主动技实例的 `active_turn_banned` 置位，禁用该主动技直到当前回合结束。
pub(crate) fn ban_active_until_turn_end(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let instance = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .ok_or_else(|| EffectError::ContentFault("turn-end active ban skill missing".into()))?;
    if instance.owner != player {
        return Err(EffectError::ContentFault(
            "turn-end active ban owner mismatch".into(),
        ));
    }
    instance.active_turn_banned = true;
    Ok(vec![DomainEvent::RuleNote {
        code: 241,
        payload: vec![player.raw(), skill.raw(), 1],
    }])
}

pub(crate) fn force_others_use_sha_on_nearest_or_lose_hp(
    tx: &mut Transaction,
    owner: PlayerId,
    remaining: Vec<PlayerId>,
    started: bool,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let mut remaining = if started {
        remaining
    } else {
        if !originating_skill_is_enabled(tx, owner, skill)
            || !tx
                .state()
                .players
                .get(owner)
                .is_some_and(|player| player.alive)
        {
            return Ok(Vec::new());
        }
        let order = tx.state().players.alive_by_seat();
        let Some(start) = order.iter().position(|&player| player == owner) else {
            return Ok(Vec::new());
        };
        (1..order.len())
            .map(|offset| order[(start + offset) % order.len()])
            .collect()
    };

    while remaining.first().is_some_and(|player| {
        !tx.state()
            .players
            .get(*player)
            .is_some_and(|participant| participant.alive)
    }) {
        remaining.remove(0);
    }
    if remaining.is_empty() {
        return Ok(Vec::new());
    }

    let current = remaining.remove(0);
    let target_limit = crate::query::sha_target_limit_for(tx.state(), current);
    let cards = if target_limit == 0 {
        Vec::new()
    } else {
        crate::command::play::prompted_sha_source_cards(tx.state(), current)
            .into_iter()
            .filter(|card| !crate::command::play::luanwu_sha_targets(tx, current, *card).is_empty())
            .collect()
    };
    if cards.is_empty() {
        crate::effect::continue_luanwu_after_hp_loss(tx, owner, current, skill, remaining)?;
        return Ok(Vec::new());
    }

    let mut payload = vec![
        0,
        owner.raw(),
        current.raw(),
        skill.raw(),
        remaining.len() as u32,
    ];
    payload.extend(remaining.into_iter().map(PlayerId::raw));
    open_choice(
        tx,
        current,
        0,
        1,
        true,
        cards.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ForceOthersUseShaOnNearestOrLoseHp,
        payload,
    )
}

pub(crate) fn discard_selected_record_unique_category_target(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let state = tx.state();
    let instance = state
        .skills
        .get(skill)
        .ok_or_else(|| EffectError::ContentFault("missing originating active skill".into()))?;
    let contract = state
        .content
        .first_active_for_skill(instance.def)
        .ok_or_else(|| EffectError::ContentFault("missing originating active contract".into()))?;
    let targets = contract.targets.as_ref();
    let selection = contract.selection.as_ref();
    let turn_scoped = targets.is_some_and(|targets| {
        targets.min == 1
            && targets.max == 1
            && !targets.allow_self
            && targets.unused_by_skill_this_turn
    }) && selection.is_some_and(|selection| {
        selection.min == 1 && selection.max == 1 && selection.unused_category_by_skill_this_turn
    });
    let play_phase_scoped = targets.is_some_and(|targets| {
        targets.min == 1
            && targets.max == 1
            && targets.allow_self
            && targets.unused_by_skill_this_play_phase
    }) && selection.is_some_and(|selection| {
        selection.min == 1
            && selection.max == 1
            && selection.unused_category_by_skill_this_play_phase
    });
    if instance.owner != player
        || !state.skill_effectively_enabled(instance)
        || state.turn.current != player
        || state.turn.phase != Phase::Play
        || turn_scoped == play_phase_scoped
    {
        return Err(EffectError::ContentFault(
            "active uniqueness effect requires its declared contract".into(),
        ));
    }
    if (target == player && !targets.is_some_and(|targets| targets.allow_self))
        || !state
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || !crate::query::discardable_hand_equip_cards(state, player).contains(&card)
    {
        return Err(EffectError::Illegal(
            "active uniqueness target or selected card is no longer legal".into(),
        ));
    }
    let category_bit = state
        .cards
        .get(card)
        .map(|card| crate::query::effective_card_kind(state, player, card))
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
    let recorded = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .is_some_and(|instance| {
            if turn_scoped {
                instance.record_active_unique_use(category_bit, target)
            } else if category_bit != 0
                && instance.play_phase_category_mask & category_bit == 0
                && !instance
                    .active_play_phase_targets()
                    .unwrap_or_default()
                    .contains(&target)
                && instance.record_active_play_phase_target(target)
            {
                instance.play_phase_category_mask |= category_bit;
                true
            } else {
                false
            }
        });
    if !recorded {
        return Err(EffectError::Illegal(
            "active target or logical card category was already used in this uniqueness window"
                .into(),
        ));
    }
    discard_cards(tx, player, vec![card])
}

pub(crate) fn record_selected_play_phase_category(
    tx: &mut Transaction,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let state = tx.state();
    let instance = state
        .skills
        .get(skill)
        .ok_or_else(|| EffectError::ContentFault("missing originating active skill".into()))?;
    let contract = state
        .content
        .first_active_for_skill(instance.def)
        .ok_or_else(|| EffectError::ContentFault("missing originating active contract".into()))?;
    if !state.skill_effectively_enabled(instance)
        || state.turn.current != instance.owner
        || state.turn.phase != Phase::Play
        || !contract.selection.as_ref().is_some_and(|selection| {
            selection.min == 1
                && selection.max == 1
                && selection.unused_category_by_skill_this_play_phase
        })
    {
        return Err(EffectError::ContentFault(
            "play-phase category effect requires its declared active contract".into(),
        ));
    }
    let owner = instance.owner;
    if !crate::query::discardable_hand_equip_cards(state, owner).contains(&card) {
        return Err(EffectError::Illegal(
            "selected card is no longer legal for play-phase category recording".into(),
        ));
    }
    let category_bit = state
        .cards
        .get(card)
        .map(|card| crate::query::effective_card_kind(state, owner, card))
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
    let instance = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .expect("originating active skill validated");
    if category_bit == 0 || instance.play_phase_category_mask & category_bit != 0 {
        return Err(EffectError::Illegal(
            "logical card category was already used in this play phase".into(),
        ));
    }
    instance.play_phase_category_mask |= category_bit;
    Ok(Vec::new())
}

pub(crate) fn choose_target_draw_or_discard_by_skill_use_count_link_if_wounded(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    toggle_link: bool,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
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
    {
        return Ok(Vec::new());
    }
    let count = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.use_count)
        .unwrap_or(0);
    if count == 0 {
        return Err(EffectError::ContentFault(
            "active draw-or-discard effect requires a recorded use".into(),
        ));
    }
    if crate::query::discardable_hand_equip_cards(tx.state(), target).is_empty() {
        tx.state_mut().resolution.deferred_tail_effects.push(
            EffectRequest::FinishLinkIfTargetWounded {
                player,
                target,
                toggle_link,
                skill,
            },
        );
        return draw_cards(tx, target, count);
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        vec![1, 2],
        ChoiceKind::Custom,
        ContinuationKind::ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded,
        vec![
            0,
            player.raw(),
            target.raw(),
            skill.raw(),
            count,
            u32::from(toggle_link),
        ],
    )
}

pub(crate) fn finish_link_if_target_wounded(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    toggle_link: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    let target_wounded = tx.state().players.get(target).is_some_and(|participant| {
        participant.alive && i64::from(participant.hp) < i64::from(participant.max_hp)
    });
    let player_unlinked = tx.state().players.get(player).is_some_and(|participant| {
        participant.alive
            && !participant
                .marks
                .iter()
                .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
    });
    if !target_wounded {
        Ok(Vec::new())
    } else if toggle_link && !player_unlinked {
        remove_mark(tx, player, crate::content::MARK_CHAIN, 0)
    } else if player_unlinked {
        add_mark(tx, player, crate::content::MARK_CHAIN, 1)
    } else {
        Ok(Vec::new())
    }
}

pub(crate) fn choose_reveal_other_hand_card_remember_category(
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
    let options: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != player && tx.state().hand_count(target) > 0)
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
        ContinuationKind::ChooseRevealOtherHandCardRememberCategory,
        vec![0, player.raw(), skill.raw()],
    )
}

pub(crate) fn choose_reveal_hand_card_bind_category_target(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != target
        || tx.state().turn.phase != Phase::Play
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let options: Vec<u32> = tx
        .state()
        .hand_cards(target)
        .into_iter()
        .map(CardId::raw)
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
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseRevealOtherHandCardRememberCategory,
        vec![2, player.raw(), skill.raw(), target.raw()],
    )
}

pub(crate) fn recover_to_max_then_optional_exchange_skill(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    removable_skill_def: engine_api::SkillDefId,
    restore_skill_def: engine_api::SkillDefId,
    ready_to_choose: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if ready_to_choose {
        if restore_skill_def.raw() == 0
            || !tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive)
        {
            return Ok(Vec::new());
        }
        let has_removable = tx.state().players.get(player).is_some_and(|participant| {
            participant.skills.iter().any(|skill| {
                tx.state()
                    .skills
                    .get(*skill)
                    .is_some_and(|instance| instance.def == removable_skill_def)
            })
        });
        if !has_removable {
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![0, 1],
            ChoiceKind::Confirm,
            ContinuationKind::RecoverToMaxThenOptionalExchangeSkill,
            vec![removable_skill_def.raw(), restore_skill_def.raw()],
        );
    }

    let Some(instance) = tx.state().skills.get(skill).cloned() else {
        return Ok(Vec::new());
    };
    if instance.owner != player || !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let Some(participant) = tx
        .state()
        .players
        .get(player)
        .filter(|participant| participant.alive)
    else {
        return Ok(Vec::new());
    };
    let amount = (i64::from(participant.max_hp) - i64::from(participant.hp))
        .max(0)
        .min(i64::from(u32::MAX)) as u32;
    let restore_skill_def = instance.def;
    let mut events = remove_skill(tx, skill)?;
    tx.state_mut().resolution.deferred_tail_effects.push(
        EffectRequest::RecoverToMaxThenOptionalExchangeSkill {
            player,
            skill: engine_api::SkillId::NONE,
            removable_skill_def,
            restore_skill_def,
            ready_to_choose: true,
        },
    );
    events.extend(recover_hp(tx, player, player, amount)?);
    Ok(events)
}

pub(crate) fn draw_then_give_exact_record_phase_target(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    count: u32,
    skill: engine_api::SkillId,
    ready_to_give: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    let valid = originating_skill_is_enabled(tx, player, skill)
        && count > 0
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
        && tx.state().turn.phase == Phase::Play;
    if !valid {
        return Ok(Vec::new());
    }
    if !ready_to_give {
        let events = draw_cards(tx, player, count)?;
        tx.state_mut().resolution.deferred_tail_effects.push(
            EffectRequest::DrawThenGiveExactRecordPhaseTarget {
                player,
                target,
                count,
                skill,
                ready_to_give: true,
            },
        );
        return Ok(events);
    }
    let mut options: Vec<CardId> = [ZoneKind::Hand, ZoneKind::Equip]
        .into_iter()
        .filter_map(|kind| tx.state().zones.find(kind, player))
        .flat_map(|zone| {
            tx.state()
                .zones
                .get(zone)
                .map(|state| state.cards.clone())
                .unwrap_or_default()
        })
        .collect();
    options.sort_by_key(|card| card.raw());
    options.dedup();
    if options.len() < count as usize || count > u8::MAX as u32 {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        count as u8,
        count as u8,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::GiveExactRecordPhaseTarget,
        vec![player.raw(), target.raw(), skill.raw(), count],
    )
}

pub(crate) fn hand_cards_sorted(tx: &Transaction, player: PlayerId) -> Vec<CardId> {
    let mut cards = tx.state().hand_cards(player);
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    cards
}

pub(crate) fn discardable_hand_cards_sorted(tx: &Transaction, player: PlayerId) -> Vec<CardId> {
    let hand = tx.state().zones.find(ZoneKind::Hand, player);
    let mut cards: Vec<_> = crate::query::discardable_hand_equip_cards(tx.state(), player)
        .into_iter()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|state| Some(state.zone) == hand)
        })
        .collect();
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    cards
}

pub(crate) fn standard_ganglie_discardable_hand_cards(
    tx: &Transaction,
    player: PlayerId,
) -> Vec<CardId> {
    discardable_hand_cards_sorted(tx, player)
}

pub(crate) fn ganglie_discardable_cards(
    tx: &Transaction,
    player: PlayerId,
    hand_only: bool,
) -> Vec<CardId> {
    if hand_only {
        return standard_ganglie_discardable_hand_cards(tx, player);
    }
    let mut cards = crate::query::discardable_hand_equip_cards(tx.state(), player);
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    cards
}

pub(crate) fn open_draw_give_or_discard_hand_discard(
    tx: &mut Transaction,
    player: PlayerId,
    payment_count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let options = discardable_hand_cards_sorted(tx, player);
    let required = options.len().min(payment_count as usize);
    if required == 0 {
        return Ok(Vec::new());
    }
    if required == options.len() {
        return discard_cards(tx, player, options);
    }
    open_choice(
        tx,
        player,
        required as u8,
        required as u8,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::DrawThenChooseGiveOrDiscardHand,
        vec![3, player.raw(), required as u32, skill.raw()],
    )
}

pub(crate) fn draw_then_choose_give_or_discard_hand(
    tx: &mut Transaction,
    player: PlayerId,
    draw_count: u32,
    payment_count: u32,
    skill: engine_api::SkillId,
    ready_to_choose: bool,
    recipient_scope: DrawTargetScope,
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
    if payment_count == 0 || payment_count > u8::MAX as u32 {
        return Err(EffectError::ContentFault(
            "give-or-discard hand payment count must be in 1..=255".into(),
        ));
    }
    if !ready_to_choose {
        let events = if draw_count == 0 {
            Vec::new()
        } else {
            draw_cards(tx, player, draw_count)?
        };
        tx.state_mut().resolution.deferred_tail_effects.push(
            EffectRequest::DrawThenChooseGiveOrDiscardHand {
                player,
                draw_count,
                payment_count,
                skill,
                ready_to_choose: true,
            },
        );
        return Ok(events);
    }

    let hand_cards = hand_cards_sorted(tx, player);
    if hand_cards.len() < payment_count as usize {
        return open_draw_give_or_discard_hand_discard(tx, player, payment_count, skill);
    }
    let has_recipient = tx
        .state()
        .players
        .iter_live()
        .any(|target| player_target_matches(tx.state(), player, target.id, recipient_scope));
    let has_discard = !discardable_hand_cards_sorted(tx, player).is_empty();
    let mut branches = Vec::with_capacity(2);
    if has_recipient {
        branches.push(1);
    }
    if has_discard {
        branches.push(2);
    }
    match branches.as_slice() {
        [] => Ok(Vec::new()),
        [2] => open_draw_give_or_discard_hand_discard(tx, player, payment_count, skill),
        _ => open_choice(
            tx,
            player,
            1,
            1,
            false,
            branches,
            ChoiceKind::Custom,
            ContinuationKind::DrawThenChooseGiveOrDiscardHand,
            vec![
                0,
                player.raw(),
                payment_count,
                skill.raw(),
                recipient_scope.tag() as u32,
            ],
        ),
    }
}

pub(crate) fn finish_optional_hand_distribution(
    tx: &mut Transaction,
    player: PlayerId,
    give_cap: u32,
    cards: Vec<CardId>,
    targets: Vec<PlayerId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    if cards.len() != targets.len() || cards.len() > give_cap as usize {
        return Err(EffectError::EngineFault(
            "invalid optional hand-distribution plan".into(),
        ));
    }
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let from = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("giver hand zone missing".into()))?;
    let current: std::collections::BTreeSet<_> =
        hand_cards_sorted(tx, player).into_iter().collect();
    let mut distinct = std::collections::BTreeSet::new();
    if cards
        .iter()
        .any(|card| !distinct.insert(*card) || !current.contains(card))
    {
        return Err(EffectError::Illegal(
            "planned hand-distribution card is duplicated or stale".into(),
        ));
    }

    let mut groups: Vec<(PlayerId, ZoneId, Vec<CardId>)> = Vec::new();
    for (card, target) in cards.into_iter().zip(targets) {
        if target == player
            || !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
        {
            return Err(EffectError::Illegal(
                "planned hand-distribution recipient is stale".into(),
            ));
        }
        if let Some((_, _, grouped_cards)) = groups
            .iter_mut()
            .find(|(group_target, _, _)| *group_target == target)
        {
            grouped_cards.push(card);
            continue;
        }
        let to = tx
            .state()
            .zones
            .find(ZoneKind::Hand, target)
            .ok_or_else(|| EffectError::EngineFault("recipient hand zone missing".into()))?;
        groups.push((target, to, vec![card]));
    }

    let mut events = Vec::new();
    for (_, to, cards) in groups {
        events.extend(give_cards_as(tx, player, cards, to, from)?);
    }
    Ok(events)
}
