//! Effect primitives for `oldxuanfeng_distance_one` and related helpers.

use super::*;

pub(crate) fn oldxuanfeng_distance_one(
    tx: &Transaction,
    player: PlayerId,
    target: PlayerId,
) -> bool {
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

pub(crate) fn remove_oldxuanfeng_virtual_sha(
    tx: &mut Transaction,
    card: CardId,
) -> Result<(), EffectError> {
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
