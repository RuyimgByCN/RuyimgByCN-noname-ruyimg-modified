//! Effect primitives for `resolve_pottuntian` and related helpers.

use super::*;

pub(crate) fn resolve_pottuntian(
    tx: &mut Transaction,
    player: PlayerId,
    action: u8,
    amount: u8,
    targets: Vec<PlayerId>,
    zaoxian_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let mark = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.def.raw())
        .ok_or_else(|| EffectError::EngineFault("势屯田技能不存在".into()))?;
    let charge = pottuntian_charge(tx, player, mark);
    let max_charge = pottuntian_max_charge(tx, skill);
    match action {
        0 => {
            if charge >= max_charge {
                return Ok(Vec::new());
            }
            add_mark(tx, player, mark, 1)
        }
        1 => {
            if charge < max_charge {
                return Ok(Vec::new());
            }
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.state_blob = max_charge.saturating_add(1).to_le_bytes().to_vec();
            }
            draw_cards(tx, player, 1)
        }
        2 => {
            let distinct: std::collections::BTreeSet<_> = targets.iter().copied().collect();
            if tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || charge == 0
                || targets.is_empty()
                || targets.len() > charge as usize
                || distinct.len() != targets.len()
                || targets.iter().any(|target| {
                    !tx.state()
                        .players
                        .get(*target)
                        .is_some_and(|participant| participant.alive)
                })
            {
                return Err(EffectError::Illegal("势屯田发动条件已变化".into()));
            }
            let mut payload = vec![
                POTTUNTIAN_CUSTOM_CHOICE,
                player.raw(),
                skill.raw(),
                zaoxian_def.raw(),
                targets.len() as u32,
            ];
            payload.extend(targets.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                (targets.len() as u32..=charge).collect(),
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                payload,
            )
        }
        3 => {
            if amount == 0
                || u32::from(amount) > charge
                || targets.is_empty()
                || targets.len() > usize::from(amount)
            {
                return Err(EffectError::Illegal("势屯田蓄力消耗已变化".into()));
            }
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("弃牌堆不存在".into()))?;
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("牌堆不存在".into()))?;
            let owner_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("势屯田手牌区不存在".into()))?;
            let has_zaoxian = player_has_skill_def(tx, player, zaoxian_def, true);
            let thresholds = [
                (2, CardKind::TrickWuzhong),
                (5, CardKind::TrickWuxie),
                (7, CardKind::TrickWugu),
            ];
            let mut reserved = std::collections::BTreeSet::new();
            let mut effects = vec![EffectRequest::RemoveMark {
                player,
                mark: engine_api::MarkDefId::new(mark),
                amount: u32::from(amount),
            }];
            if has_zaoxian {
                for (threshold, kind) in thresholds {
                    if amount < threshold {
                        continue;
                    }
                    let card = tx
                        .state()
                        .zones
                        .get(discard)
                        .into_iter()
                        .flat_map(|zone| zone.cards.iter().rev().copied())
                        .find(|card| {
                            !reserved.contains(card)
                                && tx
                                    .state()
                                    .cards
                                    .get(*card)
                                    .is_some_and(|card| card.kind == kind)
                        });
                    if let Some(card) = card {
                        reserved.insert(card);
                        effects.push(EffectRequest::MoveCards {
                            cards: vec![card],
                            to: owner_hand,
                            from: Some(discard),
                        });
                    }
                }
            }
            let heart_cards: Vec<_> = [draw, discard]
                .into_iter()
                .flat_map(|zone| {
                    tx.state()
                        .zones
                        .get(zone)
                        .into_iter()
                        .flat_map(|zone| zone.cards.iter().rev().copied())
                        .map(move |card| (zone, card))
                })
                .filter(|(_, card)| {
                    !reserved.contains(card)
                        && tx
                            .state()
                            .cards
                            .get(*card)
                            .is_some_and(|card| card.suit == engine_api::Suit::Heart)
                })
                .take(targets.len())
                .collect();
            for (target, (from, card)) in targets.into_iter().zip(heart_cards) {
                let hand = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, target)
                    .ok_or_else(|| EffectError::EngineFault("势屯田目标手牌区不存在".into()))?;
                effects.push(EffectRequest::MoveCards {
                    cards: vec![card],
                    to: hand,
                    from: Some(from),
                });
            }
            run_effects(tx, effects)
        }
        _ => Err(EffectError::ContentFault("势屯田 action 非法".into())),
    }
}
