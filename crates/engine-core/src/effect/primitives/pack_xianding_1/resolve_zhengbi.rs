//! Effect primitives for `resolve_zhengbi` and related helpers.

use super::*;

/// 崔琰毛玠「征辟」（ABI 585）。action 0 选择目标与分支；action 1 在本阶段结束时
/// 取得待结算目标的手牌/装备；action 2 交出基本牌；action 3 在赠牌触发完成后收取回礼。
pub(crate) fn resolve_zhengbi(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
            {
                return Ok(Vec::new());
            }
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| *candidate != player)
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
                vec![RESOLVE_ZHENGBI_CHOICE, 0, player.raw(), skill.raw()],
            )
        }
        1 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            let target = take_zhengbi_target(tx, skill);
            if !target.is_some()
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                || !tx.state().turn.gained_card_in_play_phase(target)
            {
                return Ok(Vec::new());
            }
            let options = crate::query::gainable_target_cards(
                tx.state(),
                player,
                target,
                TargetCardScope::HandEquip,
            );
            let hand = tx.state().zones.find(ZoneKind::Hand, target);
            let equip = tx.state().zones.find(ZoneKind::Equip, target);
            let count = u32::from(options.iter().any(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|card| Some(card.zone) == hand)
            })) + u32::from(options.iter().any(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|card| Some(card.zone) == equip)
            }));
            if count == 0 {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                count.min(u8::MAX as u32) as u8,
                count.min(u8::MAX as u32) as u8,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_ZHENGBI_CHOICE,
                    5,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )
        }
        2 => {
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            if player == target
                || !card.is_some()
                || !tx.state().cards.get(card).is_some_and(|value| {
                    value.owner == player
                        && Some(value.zone) == hand
                        && crate::query::effective_card_kind(tx.state(), player, value).is_basic()
                })
            {
                return Err(EffectError::Illegal("征辟赠牌已失效".into()));
            }
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("征辟目标手牌区不存在".into()))?;
            let from = hand.ok_or_else(|| EffectError::Illegal("征辟手牌区不存在".into()))?;
            let events = give_cards_as(tx, player, vec![card], to, from)?;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveZhengbi {
                    player,
                    target,
                    card: CardId::NONE,
                    action: 3,
                    skill,
                });
            Ok(events)
        }
        3 => {
            if ![player, target].into_iter().all(|participant| {
                tx.state()
                    .players
                    .get(participant)
                    .is_some_and(|state| state.alive)
            }) {
                return Ok(Vec::new());
            }
            let non_basic = zhengbi_payment_cards(tx, target, true);
            let basics = zhengbi_payment_cards(tx, target, false);
            let has_non_basic = !non_basic.is_empty();
            let has_two_basics = basics.len() >= 2;
            if has_non_basic && has_two_basics {
                return open_choice(
                    tx,
                    target,
                    1,
                    1,
                    false,
                    vec![0, 1],
                    ChoiceKind::Confirm,
                    ContinuationKind::Custom,
                    vec![
                        RESOLVE_ZHENGBI_CHOICE,
                        3,
                        player.raw(),
                        target.raw(),
                        skill.raw(),
                    ],
                );
            }
            if has_non_basic || has_two_basics {
                let (mode, options, count) = if has_non_basic {
                    (0, non_basic, 1)
                } else {
                    (1, basics, 2)
                };
                return open_choice(
                    tx,
                    target,
                    count,
                    count,
                    false,
                    options.into_iter().map(CardId::raw).collect(),
                    ChoiceKind::SelectCards,
                    ContinuationKind::Custom,
                    vec![
                        RESOLVE_ZHENGBI_CHOICE,
                        4,
                        player.raw(),
                        target.raw(),
                        skill.raw(),
                        mode,
                    ],
                );
            }
            let cards = tx.state().hand_cards(target);
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("征辟拥有者手牌区不存在".into()))?;
            let from = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("征辟目标手牌区不存在".into()))?;
            give_cards_as(tx, target, cards, to, from)
        }
        4 => {
            if player != target
                && tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                write_zhengbi_target(tx, skill, target);
            }
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault("bad Zhengbi action".into())),
    }
}

/// 崔琰毛玠「奉迎」（ABI 585）。辅助技能用 0/1 队列标记跳过此前已排队的同角色额外回合，
/// 只在本次奉迎插入的回合开始时补牌。
pub(crate) fn resolve_fengying(
    tx: &mut Transaction,
    player: PlayerId,
    helper_def: engine_api::SkillDefId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
            {
                return Ok(Vec::new());
            }
            let hand = tx.state().hand_cards(player);
            let discardable: Vec<_> =
                crate::query::discardable_hand_equip_cards(tx.state(), player)
                    .into_iter()
                    .filter(|card| hand.contains(card))
                    .collect();
            if discardable.len() != hand.len() {
                return Ok(Vec::new());
            }
            if hand.is_empty() {
                return resolve_fengying(tx, player, helper_def, 1, skill);
            }
            let events = discard_cards(tx, player, hand)?;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveFengying {
                    player,
                    helper_def,
                    action: 1,
                    skill,
                });
            Ok(events)
        }
        1 => {
            if !tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive)
                || tx.state().turn.extra_turn_queue.len() >= 8
            {
                return Ok(Vec::new());
            }
            let queued = tx
                .state()
                .turn
                .extra_turn_queue
                .iter()
                .filter(|candidate| **candidate == player)
                .count();
            let existing = tx
                .state()
                .players
                .get(player)
                .into_iter()
                .flat_map(|participant| participant.skills.iter())
                .find(|candidate| {
                    tx.state()
                        .skills
                        .get(**candidate)
                        .is_some_and(|instance| instance.def == helper_def)
                })
                .copied();
            let mut markers = existing
                .and_then(|helper| tx.state().skills.get(helper))
                .filter(|instance| instance.state_blob.get(..4) == Some(b"FYH1"))
                .map(|instance| instance.state_blob[4..].to_vec())
                .unwrap_or_default();
            markers.resize(markers.len().max(queued), 0);
            markers.push(1);
            let blob = [b"FYH1".as_slice(), markers.as_slice()].concat();
            let events = add_skill_with_state(tx, player, helper_def, Some(blob))?;
            tx.state_mut().turn.extra_turn_queue.push(player);
            Ok(events)
        }
        2 => {
            let mut markers = tx
                .state()
                .skills
                .get(skill)
                .filter(|instance| {
                    instance.def == helper_def && instance.state_blob.get(..4) == Some(b"FYH1")
                })
                .map(|instance| instance.state_blob[4..].to_vec())
                .unwrap_or_default();
            if markers.is_empty() {
                return run_effects(tx, vec![EffectRequest::RemoveSkill { skill }]);
            }
            let applies = markers.remove(0) != 0;
            let mut effects = Vec::new();
            if markers.is_empty() {
                effects.push(EffectRequest::RemoveSkill { skill });
            } else if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.state_blob = [b"FYH1".as_slice(), markers.as_slice()].concat();
            }
            if applies {
                let hp = tx
                    .state()
                    .players
                    .get(player)
                    .map(|state| state.hp)
                    .unwrap_or(0);
                let min_hp = tx
                    .state()
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .filter_map(|candidate| tx.state().players.get(candidate).map(|state| state.hp))
                    .min();
                let max_hp = tx
                    .state()
                    .players
                    .get(player)
                    .map(|state| state.max_hp)
                    .unwrap_or(0);
                let hand = tx.state().hand_cards(player).len().min(u32::MAX as usize) as u32;
                if min_hp == Some(hp) && max_hp > hand {
                    effects.push(EffectRequest::DrawCards {
                        player,
                        count: max_hp - hand,
                    });
                }
            }
            run_effects(tx, effects)
        }
        _ => Err(EffectError::ContentFault("bad Fengying action".into())),
    }
}

pub(crate) fn qiaojian_rank_sum(tx: &Transaction, cards: &[CardId]) -> u32 {
    cards
        .iter()
        .filter_map(|card| tx.state().cards.get(*card))
        .map(|card| u32::from(card.rank))
        .sum()
}

pub(crate) fn qiaojian_selection_valid(
    tx: &Transaction,
    player: PlayerId,
    cards: &[CardId],
) -> bool {
    let allowed: std::collections::BTreeSet<_> =
        crate::query::discardable_hand_equip_cards(tx.state(), player)
            .into_iter()
            .collect();
    cards.iter().all(|card| allowed.contains(card))
        && cards
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            == cards.len()
}

pub(crate) fn xicha_entries(blob: &[u8]) -> Option<Vec<(PlayerId, Vec<CardId>)>> {
    if blob.is_empty() {
        return Some(Vec::new());
    }
    if blob.get(..4) != Some(b"XCH1") {
        return None;
    }
    let mut cursor = 4;
    let mut entries = Vec::new();
    while cursor < blob.len() {
        if cursor + 5 > blob.len() {
            return None;
        }
        let source = PlayerId::new(u32::from_le_bytes(
            blob[cursor..cursor + 4].try_into().ok()?,
        ));
        let count = blob[cursor + 4] as usize;
        cursor += 5;
        if cursor + count.saturating_mul(4) > blob.len() {
            return None;
        }
        let cards = blob[cursor..cursor + count * 4]
            .chunks_exact(4)
            .map(|raw| CardId::new(u32::from_le_bytes(raw.try_into().unwrap())))
            .collect();
        cursor += count * 4;
        entries.push((source, cards));
    }
    Some(entries)
}

pub(crate) fn write_xicha_entries(
    instance: &mut crate::state::SkillInstance,
    entries: &[(PlayerId, Vec<CardId>)],
) {
    instance.state_blob = b"XCH1".to_vec();
    for (source, cards) in entries {
        instance
            .state_blob
            .extend_from_slice(&source.raw().to_le_bytes());
        instance
            .state_blob
            .push(cards.len().min(u8::MAX as usize) as u8);
        for card in cards.iter().take(u8::MAX as usize) {
            instance
                .state_blob
                .extend_from_slice(&card.raw().to_le_bytes());
        }
    }
}
