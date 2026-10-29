//! Effect primitives for `resolve_zhengbi` and related helpers.

use super::*;

pub(crate) fn gain_cards_by_zone(
    tx: &mut Transaction,
    player: PlayerId,
    cards: &[CardId],
) -> Result<Vec<DomainEvent>, EffectError> {
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("gain target hand missing".into()))?;
    let mut by_zone = std::collections::BTreeMap::<u32, Vec<CardId>>::new();
    for card in cards {
        if let Some(state) = tx.state().cards.get(*card) {
            if state.zone != hand {
                by_zone.entry(state.zone.raw()).or_default().push(*card);
            }
        }
    }
    let mut events = Vec::new();
    for (zone, cards) in by_zone {
        events.extend(move_cards_with_reason(
            tx,
            cards,
            hand,
            Some(ZoneId::new(zone)),
            CardMoveReason::Unspecified,
        )?);
    }
    Ok(events)
}

/// 傅干「巧谏」（ABI 586）。双方选择均完成后才公开并同时提交弃牌；后续状态由单个辅助
/// 技能承载，等待目标下回合开始再激活，回合结束自动移除。
#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_qiaojian(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    helper_def: engine_api::SkillDefId,
    action: u8,
    cards: Vec<CardId>,
    split: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|state| state.alive)
            {
                return Ok(Vec::new());
            }
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            open_choice(
                tx,
                player,
                0,
                options.len().min(u8::MAX as usize) as u8,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_QIAOJIAN_CHOICE,
                    0,
                    player.raw(),
                    target.raw(),
                    helper_def.raw(),
                    skill.raw(),
                ],
            )
        }
        1 => {
            if ![player, target].into_iter().all(|candidate| {
                tx.state()
                    .players
                    .get(candidate)
                    .is_some_and(|state| state.alive)
            }) || !qiaojian_selection_valid(tx, player, &cards)
            {
                return Err(EffectError::Illegal("巧谏选择已失效".into()));
            }
            let options = crate::query::discardable_hand_equip_cards(tx.state(), target);
            let mut payload = vec![
                RESOLVE_QIAOJIAN_CHOICE,
                1,
                player.raw(),
                target.raw(),
                helper_def.raw(),
                skill.raw(),
                cards.len() as u32,
            ];
            payload.extend(cards.into_iter().map(CardId::raw));
            open_choice(
                tx,
                target,
                0,
                options.len().min(u8::MAX as usize) as u8,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                payload,
            )
        }
        2 => {
            let split = split.min(cards.len() as u32) as usize;
            let (owner_cards, target_cards) = cards.split_at(split);
            if !qiaojian_selection_valid(tx, player, owner_cards)
                || !qiaojian_selection_valid(tx, target, target_cards)
            {
                return Err(EffectError::Illegal("巧谏弃牌已失效".into()));
            }
            let mut events = Vec::new();
            if !owner_cards.is_empty() {
                events.extend(discard_cards(tx, player, owner_cards.to_vec())?);
            }
            if !target_cards.is_empty() {
                events.extend(discard_cards(tx, target, target_cards.to_vec())?);
            }
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveQiaojian {
                    player,
                    target,
                    helper_def,
                    action: 3,
                    cards,
                    split: split as u32,
                    skill,
                });
            Ok(events)
        }
        3 => {
            let split = split.min(cards.len() as u32) as usize;
            if qiaojian_rank_sum(tx, &cards[..split]) > qiaojian_rank_sum(tx, &cards[split..]) {
                tx.state_mut().resolution.deferred_tail_effects.push(
                    EffectRequest::ResolveQiaojian {
                        player,
                        target,
                        helper_def,
                        action: 4,
                        cards: Vec::new(),
                        split: 0,
                        skill,
                    },
                );
                return recover_hp(tx, player, player, 1);
            }
            let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            let available: Vec<_> = cards
                .into_iter()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|state| Some(state.zone) == discard)
                })
                .collect();
            if available.is_empty() {
                return Ok(Vec::new());
            }
            let events = gain_cards_by_zone(tx, player, &available)?;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveQiaojian {
                    player,
                    target,
                    helper_def,
                    action: 7,
                    cards: available,
                    split: 0,
                    skill,
                });
            Ok(events)
        }
        4 => open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![1, 2],
            ChoiceKind::Confirm,
            ContinuationKind::Custom,
            vec![
                RESOLVE_QIAOJIAN_CHOICE,
                2,
                player.raw(),
                target.raw(),
                helper_def.raw(),
                skill.raw(),
            ],
        ),
        5 => {
            let Some(helper) = tx.state_mut().skills.get_mut(skill) else {
                return Ok(Vec::new());
            };
            if helper.owner != player || helper.def != helper_def || helper.state_blob.len() != 6 {
                return Ok(Vec::new());
            }
            let mode = helper.state_blob[4];
            if !(1..=3).contains(&mode) || helper.state_blob[..4] != [0, 0, 0, 0] {
                return Ok(Vec::new());
            }
            helper.state_blob[0] = u8::from(mode & 2 != 0);
            helper.state_blob[5] = 0;
            helper.temporary_until_turn_end_of = player;
            Ok(Vec::new())
        }
        6 => {
            let kind = tx
                .state()
                .cards
                .get(cards.first().copied().unwrap_or(CardId::NONE))
                .map(|card| crate::query::effective_card_kind(tx.state(), player, card));
            let bit = kind.and_then(|kind| {
                if kind.is_sha() {
                    Some(1)
                } else if kind.is_trick() && !kind.is_delayed_trick() {
                    Some(2)
                } else {
                    None
                }
            });
            let Some(bit) = bit else {
                return Ok(Vec::new());
            };
            let Some(helper) = tx.state_mut().skills.get_mut(skill) else {
                return Ok(Vec::new());
            };
            if helper.state_blob.len() != 6
                || helper.temporary_until_turn_end_of != player
                || helper.state_blob[4] & 1 == 0
                || helper.state_blob[5] & bit != 0
            {
                return Ok(Vec::new());
            }
            helper.state_blob[5] |= bit;
            let card = cards.first().copied().unwrap_or(CardId::NONE);
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                vec![1],
                ChoiceKind::Confirm,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_QIAOJIAN_CHOICE,
                    3,
                    player.raw(),
                    card.raw(),
                    skill.raw(),
                ],
            )
        }
        7 => {
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            let retained: Vec<_> = cards
                .into_iter()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|state| Some(state.zone) == hand)
                })
                .collect();
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                if !instance.add_tracked_cards(&retained) {
                    return Err(EffectError::ContentFault("巧谏无法记录获得牌".into()));
                }
            }
            Ok(Vec::new())
        }
        8 => {
            let card = cards.first().copied().unwrap_or(CardId::NONE);
            let Some(card_state) = tx.state().cards.get(card) else {
                return Ok(Vec::new());
            };
            let kind = crate::query::effective_card_kind(tx.state(), player, card_state);
            let targets = crate::command::play::saved_targeted_card_targets(tx, player, card)?;
            tx.state_mut()
                .resolution
                .register_card_replay(crate::state::PendingCardReplay {
                    source_card: card,
                    player,
                    kind: kind.tag(),
                    targets,
                    skill,
                });
            Ok(Vec::new())
        }
        9 => {
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.take_tracked_cards(&cards);
            }
            Ok(Vec::new())
        }
        10 => {
            let branch = split.min(2) as u8;
            let existing = tx
                .state()
                .players
                .get(target)
                .into_iter()
                .flat_map(|player| player.skills.iter())
                .find(|id| {
                    tx.state()
                        .skills
                        .get(**id)
                        .is_some_and(|instance| instance.def == helper_def)
                })
                .copied();
            if let Some(existing) = existing {
                let helper = tx.state_mut().skills.get_mut(existing).unwrap();
                if helper.state_blob.len() == 6 && helper.state_blob[..4] == [0, 0, 0, 0] {
                    helper.state_blob[4] |= branch;
                    return Ok(Vec::new());
                }
            }
            add_skill_with_state(tx, target, helper_def, Some(vec![0, 0, 0, 0, branch, 0]))
        }
        _ => Err(EffectError::ContentFault("bad Qiaojian action".into())),
    }
}
