//! Effect primitives for `resolve_potlvyi` and related helpers.

use super::*;

pub(crate) fn potcaoshuang_timing(action: u8, offset: u8) -> Result<u8, EffectError> {
    match action.checked_sub(offset) {
        Some(0) => Ok(0b001),
        Some(1) => Ok(0b010),
        Some(2) => Ok(0b100),
        _ => Err(EffectError::ContentFault(
            "invalid 势曹爽 timing action".into(),
        )),
    }
}

pub(crate) fn resolve_potcaoshuang(
    tx: &mut Transaction,
    player: PlayerId,
    _target: PlayerId,
    card: CardId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let skill_def = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.def.raw())
        .ok_or_else(|| EffectError::Illegal("势曹爽技能已消失".into()))?;
    match action {
        // 渐专：首次三种时机分配给典易/奢权；全部分配后的下一次时机觉醒。
        0..=2 => {
            if skill_def != crate::state::POTJIANZHUAN_DEF {
                return Err(EffectError::ContentFault("渐专技能来源不匹配".into()));
            }
            let timing = potcaoshuang_timing(action, 0)?;
            let assigned = potcaoshuang_assignment_union(tx, player);
            let options = potcaoshuang_skill_options(tx, player);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            if assigned == 0b111 {
                return open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    options,
                    ChoiceKind::Custom,
                    ContinuationKind::Custom,
                    vec![POTCAOSHUANG_CUSTOM_CHOICE, 0, player.raw(), skill.raw()],
                );
            }
            if assigned & timing != 0 {
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
                    POTCAOSHUANG_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    skill.raw(),
                    u32::from(timing),
                ],
            )
        }
        // 典易：本轮第 X 次发动，随机获得至多 X 张场上/牌堆/弃牌堆的“奢权”牌。
        3..=5 => {
            if skill_def != crate::state::POTDIANYI_DEF {
                return Err(EffectError::ContentFault("典易技能来源不匹配".into()));
            }
            let timing = potcaoshuang_timing(action, 3)?;
            let round = tx.state().turn.round;
            let (_, triggered, _) = tx
                .state()
                .skills
                .get(skill)
                .unwrap()
                .potcaoshuang_state(round);
            let count = triggered.count_ones().saturating_add(1) as usize;
            let mut candidates: Vec<_> = tx
                .state()
                .cards
                .sorted_live()
                .into_iter()
                .filter(|card| {
                    tx.state()
                        .card_has_content_tag(card.id, crate::state::SHEQUAN_CARD_TAG)
                })
                .filter(|card| {
                    tx.state()
                        .zones
                        .get(card.zone)
                        .is_some_and(|zone| match zone.kind {
                            ZoneKind::DrawPile | ZoneKind::DiscardPile => true,
                            ZoneKind::Hand | ZoneKind::Equip | ZoneKind::Judge => {
                                zone.owner.is_some() && zone.owner != player
                            }
                            _ => false,
                        })
                })
                .map(|card| card.id)
                .collect();
            let mut rng = RuleRng::from_state(tx.state().rng);
            rng.shuffle(&mut candidates);
            tx.journal(JournalEntry::RngCheckpoint);
            tx.state_mut().rng = rng.into_state();
            candidates.truncate(count);
            let total_gained = tx
                .state_mut()
                .skills
                .get_mut(skill)
                .and_then(|instance| {
                    instance.fire_potcaoshuang_timing(round, timing, candidates.len() as u32)
                })
                .map(|(_, gained)| gained)
                .ok_or_else(|| EffectError::Illegal("典易本轮时机已结算".into()))?;
            if candidates.is_empty() {
                return Ok(Vec::new());
            }
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("owner hand missing".into()))?;
            let mut effects = Vec::with_capacity(candidates.len() + 1);
            for candidate in candidates {
                let from = tx.state().cards.get(candidate).unwrap().zone;
                effects.push(EffectRequest::MoveCardsWithReason {
                    cards: vec![candidate],
                    to: hand,
                    from: Some(from),
                    reason: CardMoveReason::Unspecified,
                });
            }
            if total_gained
                > tx.state()
                    .players
                    .get(player)
                    .map_or(0, |owner| owner.max_hp)
            {
                effects.push(EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target: player,
                    amount: 1,
                    is_lose_hp: true,
                    nature: rule_api::damage_nature::NORMAL,
                });
            }
            run_effects(tx, effects)
        }
        // 奢权：所有其他角色先各自私选一张未标记手牌，最后统一写入持久标签。
        6..=8 => {
            if skill_def != crate::state::POTSHEQUAN_DEF {
                return Err(EffectError::ContentFault("奢权技能来源不匹配".into()));
            }
            let timing = potcaoshuang_timing(action, 6)?;
            let round = tx.state().turn.round;
            tx.state_mut()
                .skills
                .get_mut(skill)
                .and_then(|instance| instance.fire_potcaoshuang_timing(round, timing, 0))
                .ok_or_else(|| EffectError::Illegal("奢权本轮时机已结算".into()))?;
            let mut participants: Vec<PlayerId> = tx.state().players.alive_by_seat();
            if let Some(index) = participants
                .iter()
                .position(|participant| *participant == player)
            {
                let shift = (index + 1) % participants.len().max(1);
                participants.rotate_left(shift);
            }
            participants.retain(|participant| {
                *participant != player
                    && tx.state().hand_cards(*participant).iter().any(|card| {
                        !tx.state()
                            .card_has_content_tag(*card, crate::state::SHEQUAN_CARD_TAG)
                    })
            });
            let Some(observer) = participants.first().copied() else {
                return Ok(Vec::new());
            };
            let options: Vec<_> = tx
                .state()
                .hand_cards(observer)
                .into_iter()
                .filter(|card| {
                    !tx.state()
                        .card_has_content_tag(*card, crate::state::SHEQUAN_CARD_TAG)
                })
                .collect();
            let mut payload = vec![
                POTCAOSHUANG_CUSTOM_CHOICE,
                2,
                player.raw(),
                skill.raw(),
                0,
                participants.len() as u32,
            ];
            payload.extend(participants.iter().map(|participant| participant.raw()));
            open_choice(
                tx,
                observer,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                payload,
            )
        }
        // 溺尊：回合结束收集弃牌堆中的全部“奢权”牌。
        9 => {
            if skill_def != crate::state::POTNIZUN_DEF
                || tx.state().turn.flag(turn_flag::card_used_by_player(player)) <= 0
            {
                return Ok(Vec::new());
            }
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
            let cards: Vec<_> = tx
                .state()
                .zones
                .get(discard)
                .into_iter()
                .flat_map(|zone| zone.cards.iter())
                .copied()
                .filter(|card| {
                    tx.state()
                        .card_has_content_tag(*card, crate::state::SHEQUAN_CARD_TAG)
                })
                .collect();
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let pile = ensure_skill_pile_zone(tx, player, skill_def)?;
            if let Some(reference) = tx
                .state_mut()
                .skill_piles
                .iter_mut()
                .find(|reference| reference.zone == pile)
            {
                reference.usable_as_hand = true;
            }
            move_cards_with_reason(tx, cards, pile, Some(discard), CardMoveReason::Unspecified)
        }
        // 溺尊：本轮每使用一张牌堆牌，受到的伤害增加一。
        10 => {
            if skill_def != crate::state::POTNIZUN_DEF {
                return Err(EffectError::ContentFault("溺尊技能来源不匹配".into()));
            }
            let count = tx.state().skills.get(skill).map_or(0, |instance| {
                instance.potnizun_use_count(tx.state().turn.round)
            });
            modify_current_damage(tx, player, i32::try_from(count).unwrap_or(i32::MAX))
        }
        // 溺尊：记录本轮从本技能牌堆使用/打出的精确实体牌。
        11 => {
            if skill_def != crate::state::POTNIZUN_DEF
                || !tx.state().turn.used_playable_skill_pile_card(player, card)
                || !tx.state().turn.used_playable_skill_pile(player, skill_def)
                || !tx
                    .state()
                    .card_has_content_tag(card, crate::state::SHEQUAN_CARD_TAG)
            {
                return Ok(Vec::new());
            }
            let round = tx.state().turn.round;
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.record_potnizun_card(round, card);
            }
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault("unknown 势曹爽 action".into())),
    }
}

pub(crate) fn resolve_potcaoshuang_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 || payload[0] != POTCAOSHUANG_CUSTOM_CHOICE {
        return Err(EffectError::EngineFault("bad 势曹爽 payload".into()));
    }
    let player = PlayerId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    if observer.is_none() || !originating_skill_is_enabled(tx, player, skill) || cancelled {
        return Err(EffectError::Illegal("势曹爽技能或强制选择已变化".into()));
    }
    match payload[1] {
        // 渐专觉醒：失去所选技能、获得溺尊，并按该技能时机数成长。
        0 => {
            if observer != player
                || selections.len() != 1
                || potcaoshuang_assignment_union(tx, player) != 0b111
            {
                return Err(EffectError::Illegal("渐专觉醒选择无效".into()));
            }
            let def = selections[0];
            if !matches!(
                def,
                crate::state::POTDIANYI_DEF | crate::state::POTSHEQUAN_DEF
            ) {
                return Err(EffectError::Illegal("渐专失去技能无效".into()));
            }
            let removed = skill_instance_by_def(tx, player, engine_api::SkillDefId::new(def))
                .ok_or_else(|| EffectError::Illegal("渐专所选技能已消失".into()))?;
            let assigned = tx
                .state()
                .skills
                .get(removed)
                .unwrap()
                .potcaoshuang_state(tx.state().turn.round)
                .0
                .count_ones()
                .saturating_sub(1);
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.lifetime_fire_count = 1;
            }
            let mut effects = vec![
                EffectRequest::RemoveSkill { skill: removed },
                EffectRequest::AddSkill {
                    player,
                    skill_def: engine_api::SkillDefId::new(crate::state::POTNIZUN_DEF),
                },
            ];
            if assigned > 0 {
                effects.push(EffectRequest::ChangeMaxHp {
                    player,
                    delta: i32::try_from(assigned).unwrap_or(i32::MAX),
                });
                effects.push(EffectRequest::RecoverHp {
                    source: player,
                    player,
                    amount: assigned,
                });
            }
            run_effects(tx, effects).map(|_| ())
        }
        // 渐专首次时机分配。
        1 => {
            if observer != player || payload.len() != 5 || selections.len() != 1 {
                return Err(EffectError::Illegal("渐专分配选择无效".into()));
            }
            let timing = payload[4] as u8;
            if !matches!(timing, 0b001 | 0b010 | 0b100)
                || potcaoshuang_assignment_union(tx, player) & timing != 0
            {
                return Err(EffectError::Illegal("渐专时机已变化".into()));
            }
            let def = selections[0];
            if !matches!(
                def,
                crate::state::POTDIANYI_DEF | crate::state::POTSHEQUAN_DEF
            ) {
                return Err(EffectError::Illegal("渐专目标技能无效".into()));
            }
            let target_skill = skill_instance_by_def(tx, player, engine_api::SkillDefId::new(def))
                .ok_or_else(|| EffectError::Illegal("渐专目标技能已消失".into()))?;
            let round = tx.state().turn.round;
            if !tx
                .state_mut()
                .skills
                .get_mut(target_skill)
                .is_some_and(|instance| instance.assign_potcaoshuang_timing(round, timing))
            {
                return Err(EffectError::Illegal("渐专时机写入失败".into()));
            }
            Ok(())
        }
        // 奢权的逐角色私选；最后一人完成后统一标记。
        2 => {
            if payload.len() < 6 {
                return Err(EffectError::EngineFault("奢权选择载荷截断".into()));
            }
            let index = payload[4] as usize;
            let count = payload[5] as usize;
            if count == 0 || index >= count || payload.len() != 6 + count + index {
                return Err(EffectError::EngineFault("奢权选择载荷无效".into()));
            }
            let participants: Vec<_> = payload[6..6 + count]
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect();
            let chooser = participants[index];
            let [selected] = selections else {
                return Err(EffectError::Illegal("奢权必须选择一张牌".into()));
            };
            let selected = CardId::new(*selected);
            let hand = tx.state().zones.find(ZoneKind::Hand, chooser);
            if observer != chooser
                || tx
                    .state()
                    .cards
                    .get(selected)
                    .is_none_or(|card| card.owner != chooser || Some(card.zone) != hand)
                || tx
                    .state()
                    .card_has_content_tag(selected, crate::state::SHEQUAN_CARD_TAG)
            {
                return Err(EffectError::Illegal("奢权所选手牌已变化".into()));
            }
            let mut cards: Vec<_> = payload[6 + count..]
                .iter()
                .copied()
                .map(CardId::new)
                .collect();
            cards.push(selected);
            if index + 1 < count {
                let next = participants[index + 1];
                let options: Vec<_> = tx
                    .state()
                    .hand_cards(next)
                    .into_iter()
                    .filter(|card| {
                        !tx.state()
                            .card_has_content_tag(*card, crate::state::SHEQUAN_CARD_TAG)
                    })
                    .collect();
                if options.is_empty() {
                    return Err(EffectError::Illegal("奢权下一角色已无未标记手牌".into()));
                }
                let mut next_payload = vec![
                    POTCAOSHUANG_CUSTOM_CHOICE,
                    2,
                    player.raw(),
                    skill.raw(),
                    (index + 1) as u32,
                    count as u32,
                ];
                next_payload.extend(participants.iter().map(|participant| participant.raw()));
                next_payload.extend(cards.iter().map(|card| card.raw()));
                return open_choice(
                    tx,
                    next,
                    1,
                    1,
                    false,
                    options.into_iter().map(CardId::raw).collect(),
                    ChoiceKind::SelectCards,
                    ContinuationKind::Custom,
                    next_payload,
                )
                .map(|_| ());
            }
            for card in cards {
                tx.state_mut()
                    .add_content_card_tag(card, crate::state::SHEQUAN_CARD_TAG);
            }
            Ok(())
        }
        _ => Err(EffectError::EngineFault(
            "unknown 势曹爽 choice stage".into(),
        )),
    }
}
