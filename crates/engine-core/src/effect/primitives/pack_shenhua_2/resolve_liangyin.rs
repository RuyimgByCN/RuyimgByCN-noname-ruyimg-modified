//! Effect primitives for `resolve_liangyin` and related helpers.

use super::*;

pub(crate) fn resolve_liangyin(
    tx: &mut Transaction,
    player: PlayerId,
    kongsheng_def: engine_api::SkillDefId,
    stage: u8,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
                || liangyin_used(tx, skill)
            {
                return Ok(Vec::new());
            }
            liangyin_mark(tx, skill);
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&candidate| candidate != player)
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![RESOLVE_LIANGYIN_DRAW, 1, player.raw(), skill.raw()],
            )
        }
        3 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
                || liangyin_used(tx, skill)
            {
                return Ok(Vec::new());
            }
            liangyin_mark(tx, skill);
            let options: Vec<_> = tx
                .state()
                .hand_cards(player)
                .into_iter()
                .chain(
                    tx.state()
                        .zones
                        .find(ZoneKind::Equip, player)
                        .map(|zone| {
                            tx.state()
                                .zones
                                .get(zone)
                                .map(|zone| zone.cards.clone())
                                .unwrap_or_default()
                        })
                        .unwrap_or_default(),
                )
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![RESOLVE_LIANGYIN_DISCARD, 4, player.raw(), skill.raw()],
            )
        }
        2 => {
            // 摸牌后的回复选择（由 deferred 步骤继续，目标在摸牌中途的触发选择后恢复）。
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
            {
                return Ok(Vec::new());
            }
            crate::command::resolve_choice::resume_liangyin_recover_check(
                tx, player, target, skill,
            )?;
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault(
            "良姻 stage must resolve through its choice continuation".into(),
        )),
    }
}

/// 箜声：准备阶段将任意张牌置为「箜」；结束阶段非装备箜回手、装备箜令一名角色使用。
pub(crate) fn resolve_kongsheng(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    target: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
                || tx.state().turn.current != player
            {
                return Ok(Vec::new());
            }
            let options: Vec<_> = tx
                .state()
                .hand_cards(player)
                .into_iter()
                .chain(
                    tx.state()
                        .zones
                        .find(ZoneKind::Equip, player)
                        .map(|zone| {
                            tx.state()
                                .zones
                                .get(zone)
                                .map(|zone| zone.cards.clone())
                                .unwrap_or_default()
                        })
                        .unwrap_or_default(),
                )
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                0,
                options.len().min(u8::MAX as usize) as u8,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![RESOLVE_KONGSHENG_PLACE, 1, player.raw(), skill.raw()],
            )
        }
        2 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
                || tx.state().turn.current != player
            {
                return Ok(Vec::new());
            }
            let pile = kongsheng_pile_cards(tx, player);
            if pile.is_empty() {
                return Ok(Vec::new());
            }
            let (equips, non_equips): (Vec<_>, Vec<_>) = pile.into_iter().partition(|&card| {
                tx.state()
                    .cards
                    .get(card)
                    .is_some_and(|value| value.kind.is_equip())
            });
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
            let pile_zone = tx
                .state()
                .skill_piles
                .iter()
                .find(|pile| pile.owner == player)
                .map(|pile| pile.zone)
                .ok_or_else(|| EffectError::Illegal("no 箜 pile".into()))?;
            let mut requests = Vec::new();
            for card in non_equips {
                requests.push(EffectRequest::MoveCards {
                    cards: vec![card],
                    to: hand,
                    from: Some(pile_zone),
                });
            }
            if !requests.is_empty() {
                // 先压入下一步（装备箜目标选择），使回手触发的中途选择恢复后能继续。
                tx.state_mut().resolution.deferred_effects.insert(
                    0,
                    EffectRequest::ResolveKongsheng {
                        player,
                        stage: 3,
                        target: PlayerId::NONE,
                        card: CardId::NONE,
                        skill,
                    },
                );
                match run_effects(tx, requests) {
                    Ok(_) => {
                        // 回手未触发选择：预压步骤作废，本步直接继续。
                        tx.state_mut().resolution.deferred_effects.retain(|effect| {
                            !matches!(effect, EffectRequest::ResolveKongsheng { stage: 3, .. })
                        });
                    }
                    Err(e) => return Err(e),
                }
            }
            if equips.is_empty() {
                return Ok(Vec::new());
            }
            let options: Vec<_> = tx.state().players.alive_by_seat();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                RESOLVE_KONGSHENG_TARGET,
                3,
                player.raw(),
                skill.raw(),
                equips.len() as u32,
            ];
            payload.extend(equips.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )
        }
        3 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
                || tx.state().turn.current != player
            {
                return Ok(Vec::new());
            }
            let pile = kongsheng_pile_cards(tx, player);
            let equips: Vec<_> = pile
                .into_iter()
                .filter(|&card| {
                    tx.state()
                        .cards
                        .get(card)
                        .is_some_and(|value| value.kind.is_equip())
                })
                .collect();
            if equips.is_empty() {
                return Ok(Vec::new());
            }
            let options: Vec<_> = tx.state().players.alive_by_seat();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                RESOLVE_KONGSHENG_TARGET,
                3,
                player.raw(),
                skill.raw(),
                equips.len() as u32,
            ];
            payload.extend(equips.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )
        }
        4 => {
            // 装备移入目标手牌：先压入使用步骤，再移牌，中途触发选择恢复后继续。
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
                || tx.state().turn.current != player
            {
                return Ok(Vec::new());
            }
            if card.is_none() || target.is_none() {
                return Ok(Vec::new());
            }
            let pile_zone = tx
                .state()
                .skill_piles
                .iter()
                .find(|pile| pile.owner == player)
                .map(|pile| pile.zone)
                .ok_or_else(|| EffectError::Illegal("no 箜 pile".into()))?;
            let target_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("no target hand zone".into()))?;
            tx.state_mut().resolution.deferred_effects.insert(
                0,
                EffectRequest::ResolveKongsheng {
                    player,
                    stage: 5,
                    target,
                    card,
                    skill,
                },
            );
            run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![card],
                    to: target_hand,
                    from: Some(pile_zone),
                }],
            )?;
            Ok(Vec::new())
        }
        5 => {
            // 装备使用：先压入下一件移牌步骤，再使用当前件。
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
                || tx.state().turn.current != player
            {
                return Ok(Vec::new());
            }
            if card.is_none() || target.is_none() {
                return Ok(Vec::new());
            }
            let next = kongsheng_pile_cards(tx, player)
                .into_iter()
                .find(|&value| {
                    tx.state()
                        .cards
                        .get(value)
                        .is_some_and(|v| v.kind.is_equip())
                })
                .unwrap_or(CardId::NONE);
            tx.state_mut().resolution.deferred_effects.insert(
                0,
                EffectRequest::ResolveKongsheng {
                    player,
                    stage: 4,
                    target,
                    card: next,
                    skill,
                },
            );
            crate::command::play::use_equipment_from_hand(tx, target, card)?;
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault(
            "箜声 stage must resolve through its choice continuation".into(),
        )),
    }
}

pub(crate) fn pottuntian_max_charge(tx: &Transaction, skill: engine_api::SkillId) -> u32 {
    tx.state()
        .skills
        .get(skill)
        .and_then(|skill| skill.state_blob.get(..4))
        .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
        .unwrap_or(0)
}

pub(crate) fn pottuntian_charge(tx: &Transaction, player: PlayerId, mark: u32) -> u32 {
    tx.state()
        .players
        .get(player)
        .into_iter()
        .flat_map(|player| player.marks.iter())
        .find(|(existing, _)| *existing == mark)
        .map(|(_, count)| (*count).max(0) as u32)
        .unwrap_or(0)
}

pub(crate) fn zhugemengxue_card_in_hand_equip(
    tx: &Transaction,
    player: PlayerId,
    card: CardId,
) -> bool {
    tx.state().cards.get(card).is_some_and(|state| {
        state.owner == player
            && [
                tx.state().zones.find(ZoneKind::Hand, player),
                tx.state().zones.find(ZoneKind::Equip, player),
            ]
            .contains(&Some(state.zone))
    })
}

pub(crate) fn zhugemengxue_branch_mask(tx: &Transaction, skill: engine_api::SkillId) -> u8 {
    tx.state()
        .skills
        .get(skill)
        .and_then(|instance| {
            (instance.state_blob.len() == 5 && &instance.state_blob[..4] == b"JCH1")
                .then(|| instance.state_blob[4])
        })
        .unwrap_or(0)
}

pub(crate) fn set_zhugemengxue_branch_mask(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    mask: u8,
) {
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob = [b"JCH1".as_slice(), &[mask]].concat();
    }
}

pub(crate) fn zhugemengxue_jichun_target_is_legal(
    tx: &Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    mask: u8,
) -> bool {
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
    {
        return false;
    }
    let owner_hand = tx.state().hand_count(player);
    let target_hand = tx.state().hand_count(target);
    (target_hand < owner_hand && mask & 1 == 0)
        || (target_hand > owner_hand
            && mask & 2 == 0
            && crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&card)
            && !crate::command::play::discardable_target_zone_card_tokens(tx, player, target)
                .is_empty())
}

pub(crate) fn zhugemengxue_hanying_targets(
    tx: &Transaction,
    player: PlayerId,
    card: CardId,
) -> Vec<PlayerId> {
    let Some(card_state) = tx.state().cards.get(card) else {
        return Vec::new();
    };
    let Some(draw) = tx.state().zones.find(ZoneKind::DrawPile, PlayerId::NONE) else {
        return Vec::new();
    };
    if card_state.zone != draw {
        return Vec::new();
    }
    let kind = crate::query::effective_card_kind(tx.state(), player, card_state);
    let Some(slot) = kind.equip_slot() else {
        return Vec::new();
    };
    let hand = tx.state().hand_count(player);
    tx.state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| {
            tx.state().hand_count(*target) == hand
                && tx.state().players.get(*target).is_some_and(|participant| {
                    !participant.dying && participant.equip_slot_enabled(slot)
                })
                && !crate::query::card_use_cancelled_for_card(tx.state(), *target, card)
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
/// XIANDING 诸葛梦雪「寄春 / 寒英」（ABI 588）。
pub(crate) fn resolve_zhugemengxue(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    action: u8,
    cards: Vec<CardId>,
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
            let [card] = cards.as_slice() else {
                return Err(EffectError::Illegal("寄春须选择一张牌".into()));
            };
            let card = *card;
            if !zhugemengxue_card_in_hand_equip(tx, player, card)
                || crate::query::card_name_length(tx.state(), player, card).is_none()
            {
                return Err(EffectError::Illegal("寄春所选牌已失效".into()));
            }
            let mask = zhugemengxue_branch_mask(tx, skill);
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| {
                    zhugemengxue_jichun_target_is_legal(tx, player, *target, card, mask)
                })
                .collect();
            if options.is_empty() {
                return Err(EffectError::Illegal("寄春没有合法目标".into()));
            }
            let mut payload = vec![
                RESOLVE_ZHUGEMENGXUE_CHOICE,
                0,
                player.raw(),
                skill.raw(),
                card.raw(),
                options.len() as u32,
            ];
            payload.extend(options.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )
        }
        1 => {
            let mask = zhugemengxue_branch_mask(tx, skill);
            if !originating_skill_is_enabled(tx, player, skill)
                || !zhugemengxue_card_in_hand_equip(tx, player, card)
                || crate::query::card_name_length(tx.state(), player, card).is_none()
                || !zhugemengxue_jichun_target_is_legal(tx, player, target, card, mask)
            {
                return Err(EffectError::Illegal("寄春目标或牌已失效".into()));
            }
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveZhugemengxue {
                    player,
                    target,
                    card,
                    action: 3,
                    cards: Vec::new(),
                    skill,
                },
            );
            Ok(vec![DomainEvent::CardsRevealed {
                cards: vec![card],
                to: Vec::new(),
            }])
        }
        2 => {
            if tx.state().skills.get(skill).is_some_and(|instance| {
                instance.owner == player && instance.state_blob.get(..4) == Some(b"JCH1")
            }) {
                tx.state_mut()
                    .skills
                    .get_mut(skill)
                    .unwrap()
                    .state_blob
                    .clear();
            }
            Ok(Vec::new())
        }
        3 => {
            let length = crate::query::card_name_length(tx.state(), player, card)
                .ok_or_else(|| EffectError::Illegal("寄春牌名字数不可用".into()))?;
            let mask = zhugemengxue_branch_mask(tx, skill);
            if !originating_skill_is_enabled(tx, player, skill)
                || !zhugemengxue_card_in_hand_equip(tx, player, card)
                || !zhugemengxue_jichun_target_is_legal(tx, player, target, card, mask)
            {
                return Err(EffectError::Illegal("寄春结算条件已变化".into()));
            }
            if tx.state().hand_count(target) < tx.state().hand_count(player) {
                set_zhugemengxue_branch_mask(tx, skill, mask | 1);
                let from = tx.state().cards.get(card).unwrap().zone;
                let to = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, target)
                    .ok_or_else(|| EffectError::EngineFault("寄春目标手牌区不存在".into()))?;
                tx.state_mut()
                    .resolution
                    .deferred_tail_effects
                    .push(EffectRequest::DrawCards {
                        player,
                        count: u32::from(length),
                    });
                give_cards_as(tx, player, vec![card], to, from)
            } else {
                set_zhugemengxue_branch_mask(tx, skill, mask | 2);
                tx.state_mut().resolution.deferred_effects.push(
                    EffectRequest::ResolveZhugemengxue {
                        player,
                        target,
                        card,
                        action: 4,
                        cards: Vec::new(),
                        skill,
                    },
                );
                discard_cards(tx, player, vec![card])
            }
        }
        4 => {
            let Some(length) = crate::query::card_name_length(tx.state(), player, card) else {
                return Ok(Vec::new());
            };
            let options =
                crate::command::play::discardable_target_zone_card_tokens(tx, player, target);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let max = usize::from(length)
                .min(options.len())
                .min(usize::from(u8::MAX)) as u8;
            let mut payload = vec![
                RESOLVE_ZHUGEMENGXUE_CHOICE,
                1,
                player.raw(),
                skill.raw(),
                target.raw(),
                u32::from(max),
                options.len() as u32,
            ];
            payload.extend(options.iter().copied());
            open_choice(
                tx,
                player,
                1,
                max,
                false,
                options,
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                payload,
            )
        }
        10 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            let Some(draw) = tx.state().zones.find(ZoneKind::DrawPile, PlayerId::NONE) else {
                return Ok(Vec::new());
            };
            let card = tx
                .state()
                .zones
                .get(draw)
                .into_iter()
                .flat_map(|zone| zone.cards.iter().rev().copied())
                .find(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|state| state.kind.is_equip())
                        && !crate::query::card_is_gift(tx.state(), *card)
                });
            let Some(card) = card else {
                return Ok(Vec::new());
            };
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveZhugemengxue {
                    player,
                    target: PlayerId::NONE,
                    card,
                    action: 12,
                    cards: Vec::new(),
                    skill,
                },
            );
            Ok(vec![DomainEvent::CardsRevealed {
                cards: vec![card],
                to: Vec::new(),
            }])
        }
        11 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !zhugemengxue_hanying_targets(tx, player, card).contains(&target)
            {
                return Err(EffectError::Illegal("寒英目标或装备已失效".into()));
            }
            crate::command::play::use_equipment_from_shared_zone(tx, target, card)?;
            Ok(Vec::new())
        }
        12 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            let options = zhugemengxue_hanying_targets(tx, player, card);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                RESOLVE_ZHUGEMENGXUE_CHOICE,
                2,
                player.raw(),
                skill.raw(),
                card.raw(),
                options.len() as u32,
            ];
            payload.extend(options.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault("诸葛梦雪 action 非法".into())),
    }
}

pub(crate) fn bailingyun_card_in_processing_or_discard(tx: &Transaction, card: CardId) -> bool {
    tx.state().cards.get(card).is_some_and(|state| {
        tx.state()
            .zones
            .get(state.zone)
            .is_some_and(|zone| matches!(zone.kind, ZoneKind::Processing | ZoneKind::DiscardPile))
    })
}

pub(crate) fn return_bailingyun_cards_to_top(
    tx: &mut Transaction,
    cards: Vec<CardId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("灵慧牌堆不存在".into()))?;
    let mut events = Vec::new();
    let filtered: Vec<_> = cards
        .into_iter()
        .filter(|card| bailingyun_card_in_processing_or_discard(tx, *card))
        .rev()
        .collect();
    for card in filtered {
        let from = tx.state().cards.get(card).unwrap().zone;
        events.extend(move_cards(tx, vec![card], draw, Some(from))?);
    }
    Ok(events)
}

pub(crate) fn bailingyun_xiace_mask(tx: &Transaction, player: PlayerId) -> u8 {
    tx.state()
        .turn
        .flag(turn_flag::xiace_used_by_player(player))
        .clamp(0, u8::MAX as i32) as u8
}

#[allow(clippy::too_many_arguments)]
/// XIANDING 柏灵筠「灵慧 / 黠策」（ABI 593）。
pub(crate) fn resolve_bailingyun(
    tx: &mut Transaction,
    player: PlayerId,
    _target: PlayerId,
    card: CardId,
    action: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || tx.state().turn.phase != Phase::End
                || (tx.state().turn.current != player
                    && tx.state().turn.entered_dying_players.is_empty())
            {
                return Ok(Vec::new());
            }
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("灵慧牌堆不存在".into()))?;
            while tx
                .state()
                .zones
                .get(draw)
                .map_or(0, |zone| zone.cards.len())
                < 3
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
                .map(|zone| zone.cards.iter().rev().take(3).copied().collect())
                .unwrap_or_default();
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let processing = tx
                .state()
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("灵慧处理区不存在".into()))?;
            let mut events = move_cards(tx, cards.clone(), processing, Some(draw))?;
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveBailingyun {
                    player,
                    target: PlayerId::NONE,
                    card: CardId::NONE,
                    action: 1,
                    cards: cards.clone(),
                    skill,
                },
            );
            events.push(DomainEvent::CardsRevealed {
                cards,
                to: vec![player],
            });
            Ok(events)
        }
        1 => {
            let available: Vec<_> = cards
                .iter()
                .copied()
                .filter(|card| {
                    bailingyun_card_in_processing_or_discard(tx, *card)
                        && crate::command::play::skill_card_target_spec(tx, player, *card).is_ok()
                })
                .collect();
            if !originating_skill_is_enabled(tx, player, skill) || available.is_empty() {
                return return_bailingyun_cards_to_top(tx, cards);
            }
            let mut payload = vec![
                RESOLVE_BAILINGYUN_CHOICE,
                0,
                player.raw(),
                skill.raw(),
                cards.len() as u32,
            ];
            payload.extend(cards.iter().map(|card| card.raw()));
            payload.push(available.len() as u32);
            payload.extend(available.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                available.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                payload,
            )
        }
        2 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !bailingyun_card_in_processing_or_discard(tx, card)
            {
                return return_bailingyun_cards_to_top(tx, cards);
            }
            match crate::command::play::skill_card_target_spec(tx, player, card)? {
                crate::command::play::SkillCardTargetSpec::Immediate(targets) => {
                    tx.state_mut().resolution.deferred_tail_effects.push(
                        EffectRequest::ResolveBailingyun {
                            player,
                            target: PlayerId::NONE,
                            card: CardId::NONE,
                            action: 3,
                            cards,
                            skill,
                        },
                    );
                    crate::command::play::handle_skill_card(tx, player, card, &targets)?;
                    Ok(Vec::new())
                }
                crate::command::play::SkillCardTargetSpec::Select { min, max, options } => {
                    let mut payload = vec![
                        RESOLVE_BAILINGYUN_CHOICE,
                        1,
                        player.raw(),
                        skill.raw(),
                        card.raw(),
                        u32::from(min),
                        u32::from(max),
                        cards.len() as u32,
                    ];
                    payload.extend(cards.iter().map(|card| card.raw()));
                    payload.push(options.len() as u32);
                    payload.extend(options.iter().map(|target| target.raw()));
                    open_choice(
                        tx,
                        player,
                        min,
                        max,
                        false,
                        options.into_iter().map(PlayerId::raw).collect(),
                        ChoiceKind::SelectTargets,
                        ContinuationKind::Custom,
                        payload,
                    )
                }
                crate::command::play::SkillCardTargetSpec::Pairs(pairs) => {
                    let mut payload = vec![
                        RESOLVE_BAILINGYUN_CHOICE,
                        2,
                        player.raw(),
                        skill.raw(),
                        card.raw(),
                        cards.len() as u32,
                    ];
                    payload.extend(cards.iter().map(|card| card.raw()));
                    payload.push(pairs.len() as u32);
                    for (left, right) in &pairs {
                        payload.push(left.raw());
                        payload.push(right.raw());
                    }
                    open_choice(
                        tx,
                        player,
                        1,
                        1,
                        false,
                        (1..=pairs.len() as u32).collect(),
                        ChoiceKind::Custom,
                        ContinuationKind::Custom,
                        payload,
                    )
                }
            }
        }
        3 => {
            let mut available: Vec<_> = cards
                .iter()
                .copied()
                .filter(|card| bailingyun_card_in_processing_or_discard(tx, *card))
                .collect();
            if available.is_empty()
                || !tx
                    .state()
                    .players
                    .get(player)
                    .is_some_and(|participant| participant.alive)
            {
                return return_bailingyun_cards_to_top(tx, cards);
            }
            let mut rng = RuleRng::from_state(tx.state().rng);
            tx.journal(JournalEntry::RngCheckpoint);
            let index = rng.gen_range_u32(available.len() as u32) as usize;
            tx.state_mut().rng = rng.into_state();
            let gained = available.remove(index);
            let remaining: Vec<_> = cards.into_iter().filter(|card| *card != gained).collect();
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveBailingyun {
                    player,
                    target: PlayerId::NONE,
                    card: CardId::NONE,
                    action: 4,
                    cards: remaining,
                    skill,
                },
            );
            gain_cards_by_zone(tx, player, &[gained])
        }
        4 => return_bailingyun_cards_to_top(tx, cards),
        10 => {
            let mask = bailingyun_xiace_mask(tx, player);
            if !originating_skill_is_enabled(tx, player, skill) || mask & 1 != 0 {
                return Ok(Vec::new());
            }
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| *target != player)
                .filter(|target| {
                    !tx.state()
                        .turn
                        .non_locked_skill_suppressed_players
                        .contains(target)
                })
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                RESOLVE_BAILINGYUN_CHOICE,
                3,
                player.raw(),
                skill.raw(),
                options.len() as u32,
            ];
            payload.extend(options.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )
        }
        11 => {
            let mask = bailingyun_xiace_mask(tx, player);
            let wounded = tx.state().players.get(player).is_some_and(|participant| {
                participant.alive && participant.hp < participant.max_hp as i32
            });
            if !originating_skill_is_enabled(tx, player, skill) || mask & 2 != 0 || !wounded {
                return Ok(Vec::new());
            }
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                RESOLVE_BAILINGYUN_CHOICE,
                4,
                player.raw(),
                skill.raw(),
                options.len() as u32,
            ];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault("柏灵筠 action 非法".into())),
    }
}

pub(crate) fn resolve_dcshaowei(
    tx: &mut Transaction,
    player: PlayerId,
    action: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    if action == 1 {
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
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            options,
            ChoiceKind::Custom,
            ContinuationKind::ChooseDrltZhitiEnableSlot,
            vec![player.raw(), skill.raw()],
        );
    }
    if action != 0 {
        return Err(EffectError::ContentFault("绍威 action 非法".into()));
    }

    let key = turn_flag::dcshaowei_used_by_player(player);
    let used = tx.state().turn.flag(key).clamp(0, u8::MAX as i32) as u8;
    let mut suits = 0u8;
    for card in cards {
        let Some(card) = tx.state().cards.get(card) else {
            continue;
        };
        match crate::query::effective_card_suit(tx.state(), player, card) {
            Suit::Diamond => suits |= 1,
            Suit::Heart => suits |= 2,
            _ => {}
        }
    }
    let pending = suits & !used;
    if pending == 0 {
        return Ok(Vec::new());
    }

    let mut events = Vec::new();
    let mut next_used = used;
    if pending & 1 != 0 {
        let mut red: Vec<_> =
            tx.state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .and_then(|draw| tx.state().zones.get(draw))
                .into_iter()
                .flat_map(|zone| zone.cards.iter().copied())
                .filter(|card| {
                    tx.state().cards.get(*card).is_some_and(|card| {
                        crate::query::card_color_mask_for_suit(card.suit) == 0b01
                    })
                })
                .collect();
        if !red.is_empty() {
            let mut rng = RuleRng::from_state(tx.state().rng);
            tx.journal(JournalEntry::RngCheckpoint);
            rng.shuffle(&mut red);
            tx.state_mut().rng = rng.into_state();
            red.truncate(3);
            events.extend(gain_cards_by_zone(tx, player, &red)?);
        }
        next_used |= 1;
    }
    if pending & 2 != 0 {
        next_used |= 2;
        tx.state_mut().resolution.deferred_tail_effects.extend([
            EffectRequest::RecoverHp {
                source: player,
                player,
                amount: 1,
            },
            EffectRequest::ResolveDcshaowei {
                player,
                action: 1,
                cards: Vec::new(),
                skill,
            },
        ]);
    }
    tx.state_mut().turn.set_flag(key, i32::from(next_used));
    Ok(events)
}

pub(crate) fn dcdichou_mark(
    tx: &Transaction,
    skill: engine_api::SkillId,
) -> Option<engine_api::MarkDefId> {
    tx.state()
        .skills
        .get(skill)
        .map(|instance| engine_api::MarkDefId::new(instance.def.raw()))
}

pub(crate) fn dcdichou_mark_count(
    tx: &Transaction,
    target: PlayerId,
    mark: engine_api::MarkDefId,
) -> i32 {
    tx.state()
        .players
        .get(target)
        .and_then(|player| player.marks.iter().find(|(id, _)| *id == mark.raw()))
        .map(|(_, count)| *count)
        .unwrap_or(0)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_dcdichou(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    action: u8,
    helper_skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let mark = dcdichou_mark(tx, skill)
        .ok_or_else(|| EffectError::EngineFault("涤仇技能不存在".into()))?;
    match action {
        0 => {
            let options: Vec<_> = tx
                .state()
                .players
                .get(player)
                .into_iter()
                .flat_map(|owner| owner.disabled_equip_slots.iter().enumerate())
                .filter_map(|(index, count)| (*count == 0).then_some(index as u32 + 1))
                .collect();
            if options.is_empty() {
                return resolve_dcdichou(
                    tx,
                    player,
                    PlayerId::NONE,
                    CardId::NONE,
                    1,
                    helper_skill_def,
                    skill,
                );
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
                    RESOLVE_DCDICHOU_CHOICE,
                    0,
                    player.raw(),
                    skill.raw(),
                    helper_skill_def.raw(),
                ],
            )
        }
        1 => {
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| {
                    dcdichou_mark_count(tx, *target, mark) > 0
                        && !crate::query::discardable_hand_equip_cards(tx.state(), *target)
                            .is_empty()
                })
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                RESOLVE_DCDICHOU_CHOICE,
                1,
                player.raw(),
                skill.raw(),
                helper_skill_def.raw(),
                options.len() as u32,
            ];
            payload.extend(options.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )
        }
        2 => {
            if dcdichou_mark_count(tx, target, mark) <= 0 {
                return Err(EffectError::Illegal("涤仇目标已失效".into()));
            }
            let options = crate::query::discardable_hand_equip_cards(tx.state(), target);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                RESOLVE_DCDICHOU_CHOICE,
                2,
                player.raw(),
                skill.raw(),
                helper_skill_def.raw(),
                target.raw(),
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
                ContinuationKind::Custom,
                payload,
            )
        }
        3 => {
            let Some(length) = crate::query::card_name_length(tx.state(), player, card) else {
                return Ok(Vec::new());
            };
            tx.state_mut().resolution.deferred_tail_effects.extend([
                EffectRequest::RemoveMark {
                    player,
                    mark: engine_api::MarkDefId::new(helper_skill_def.raw()),
                    amount: 0,
                },
                EffectRequest::AddSkillUntilTurnEnd {
                    player,
                    skill_def: helper_skill_def,
                },
                EffectRequest::AddMark {
                    player,
                    mark: engine_api::MarkDefId::new(helper_skill_def.raw()),
                    delta: i32::from(length),
                },
            ]);
            Ok(Vec::new())
        }
        10 => {
            // R01：候选 = 当前仍有仇的角色 **加上本回合已移除仇的角色**。
            let all: Vec<PlayerId> = tx
                .state()
                .players
                .iter_live()
                .map(|participant| participant.id)
                .collect();
            let removed = tx.state().turn.removed_marks.clone();
            let options: Vec<_> = all
                .iter()
                .copied()
                .filter(|target| {
                    dcdichou_mark_count(tx, *target, mark) > 0
                        || removed.contains(&(mark.raw(), *target))
                })
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            // 选择前先移除**所有**仇标记；候选快照在 payload 中，选择后不再依赖
            // 残留标记。这样伤害结算时标记已清空，且未选者也被清空。
            let mut cleared = Vec::new();
            for target in &all {
                if dcdichou_mark_count(tx, *target, mark) == 0 {
                    continue;
                }
                let prev = tx
                    .state()
                    .players
                    .get(*target)
                    .map(|participant| participant.marks.clone())
                    .unwrap_or_default();
                tx.journal(crate::transaction::JournalEntry::PlayerMarks {
                    id: *target,
                    prev,
                });
                if let Some(participant) = tx.state_mut().players.get_mut(*target) {
                    participant.marks.retain(|(existing, _)| *existing != mark.raw());
                }
                cleared.push(DomainEvent::MarkChanged {
                    player: *target,
                    mark_def: mark.raw(),
                    count: 0,
                });
            }
            tx.state_mut()
                .turn
                .removed_marks
                .retain(|(existing, _)| *existing != mark.raw());
            let mut payload = vec![
                RESOLVE_DCDICHOU_CHOICE,
                10,
                player.raw(),
                skill.raw(),
                helper_skill_def.raw(),
                options.len() as u32,
            ];
            payload.extend(options.iter().map(|target| target.raw()));
            let mut events = open_choice(
                tx,
                player,
                1,
                options.len().min(u8::MAX as usize) as u8,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )?;
            events.splice(0..0, cleared);
            Ok(events)
        }
        _ => Err(EffectError::ContentFault("涤仇 action 非法".into())),
    }
}

pub(crate) fn resolve_dcdufeng(
    tx: &mut Transaction,
    player: PlayerId,
    action: u8,
    helper_skill_def: engine_api::SkillDefId,
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
    match action {
        0 => {
            let slots: Vec<_> = tx
                .state()
                .players
                .get(player)
                .into_iter()
                .flat_map(|owner| owner.disabled_equip_slots.iter().enumerate())
                .filter_map(|(index, count)| (*count == 0).then_some(index as u32 + 1))
                .collect();
            let mut options = vec![0];
            options.extend(slots);
            let mut payload = vec![
                RESOLVE_DCDUFENG_CHOICE,
                player.raw(),
                skill.raw(),
                helper_skill_def.raw(),
                options.len() as u32,
            ];
            payload.extend(options.iter().copied());
            open_choice(
                tx,
                player,
                1,
                if options.len() > 1 { 2 } else { 1 },
                false,
                options,
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                payload,
            )
        }
        1 => {
            let Some(owner) = tx.state().players.get(player) else {
                return Ok(Vec::new());
            };
            let disabled = owner
                .disabled_equip_slots
                .iter()
                .filter(|count| **count > 0)
                .count() as u32;
            let lost_hp = (i64::from(owner.max_hp) - i64::from(owner.hp)).max(0) as u32;
            let amount = disabled.saturating_add(lost_hp).min(owner.max_hp);
            if amount == 0 {
                return Ok(Vec::new());
            }
            let mut effects = vec![EffectRequest::DrawCards {
                player,
                count: amount,
            }];
            if amount > 1 {
                effects.extend([
                    EffectRequest::RemoveMark {
                        player,
                        mark: engine_api::MarkDefId::new(helper_skill_def.raw()),
                        amount: 0,
                    },
                    EffectRequest::AddSkillUntilTurnEnd {
                        player,
                        skill_def: helper_skill_def,
                    },
                    EffectRequest::AddMark {
                        player,
                        mark: engine_api::MarkDefId::new(helper_skill_def.raw()),
                        delta: amount.saturating_sub(1) as i32,
                    },
                ]);
            }
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .extend(effects);
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault("独锋 action 非法".into())),
    }
}
