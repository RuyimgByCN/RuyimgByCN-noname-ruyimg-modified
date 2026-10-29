//! Effect primitives for `guanxing_top_by_falu_marks` and related helpers.

use super::*;

/// ABI 683 (batch 183): count non-zero stacks among the four falu marks and run
/// [`guanxing_top`] for that many top cards.
pub(crate) fn guanxing_top_by_falu_marks(
    tx: &mut Transaction,
    player: PlayerId,
    marks: [engine_api::SkillDefId; 4],
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let participant = tx
        .state()
        .players
        .get(player)
        .ok_or_else(|| EffectError::Illegal("bad guanxing viewer".into()))?;
    let n = marks
        .iter()
        .filter(|def| {
            participant
                .marks
                .iter()
                .any(|(found, count)| *found == def.raw() && *count > 0)
        })
        .count() as u32;
    guanxing_top(tx, player, n, skill)
}

pub(crate) fn resolve_zaiqi(
    tx: &mut Transaction,
    player: PlayerId,
    extra: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Draw
        || tx.state().turn.draw_phase_is_fixed()
    {
        return Ok(Vec::new());
    }
    let Some(owner) = tx.state().players.get(player) else {
        return Ok(Vec::new());
    };
    if !owner.alive || owner.hp as u32 >= owner.max_hp {
        return Ok(Vec::new());
    }
    let lost = (owner.max_hp as i32 - owner.hp).max(1) as u32;
    let count = lost.min(12) as usize + extra as usize;
    if count == 0 {
        return Ok(Vec::new());
    }
    tx.state_mut().turn.set_draw_phase_fixed_count(0);
    // 翻亮牌堆顶 count 张（旧 JS showCards）。
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    let mut flipped = Vec::new();
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
            .and_then(|zone| zone.cards.last().copied())
        else {
            break;
        };
        move_one_card(tx, card, draw, processing, PlayerId::NONE, true)?;
        flipped.push(card);
    }
    if flipped.is_empty() {
        return Ok(Vec::new());
    }
    let mut events = vec![DomainEvent::CardsMoved {
        cards: flipped.clone(),
        from: draw,
        to: processing,
        reason: CardMoveReason::Unspecified,
    }];
    events.push(DomainEvent::CardsRevealed {
        cards: flipped.clone(),
        to: Vec::new(),
    });
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
    let mut hearts = 0u32;
    let mut gained = Vec::new();
    let mut discard_cards = Vec::new();
    for card in flipped {
        let suit = tx
            .state()
            .cards
            .get(card)
            .map(|value| crate::query::effective_card_suit(tx.state(), player, value))
            .unwrap_or(engine_api::Suit::None);
        if suit == engine_api::Suit::Heart {
            hearts += 1;
            discard_cards.push(card);
        } else {
            gained.push(card);
        }
    }
    events.extend(move_cards(tx, gained, hand, Some(processing))?);
    events.extend(move_cards(tx, discard_cards, discard, Some(processing))?);
    if hearts > 0 {
        events.extend(run_effects(
            tx,
            vec![EffectRequest::RecoverHp {
                source: PlayerId::NONE,
                player,
                amount: hearts,
            }],
        )?);
    }
    Ok(events)
}

pub(crate) fn resolve_twqiushou(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx.state().players.get(player).is_some_and(|p| p.alive)
    {
        return Ok(Vec::new());
    }
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| {
            matches!(
                tx.state().player_group(candidate),
                Some("shu") | Some("qun")
            )
        })
        .collect();
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    run_effects(
        tx,
        targets
            .into_iter()
            .map(|target| EffectRequest::DrawCards {
                player: target,
                count: 1,
            })
            .collect(),
    )
}

pub(crate) fn skill_instance_by_def(
    tx: &Transaction,
    player: PlayerId,
    def: engine_api::SkillDefId,
) -> Option<engine_api::SkillId> {
    tx.state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|instance| instance.owner == player && instance.def == def)
        .map(|instance| instance.id)
}

pub(crate) fn state_u32(tx: &Transaction, skill: engine_api::SkillId, offset: usize) -> u32 {
    tx.state()
        .skills
        .get(skill)
        .and_then(|instance| instance.state_blob.get(offset..offset + 4))
        .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
        .unwrap_or(0)
}

pub(crate) fn set_state_u32(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    offset: usize,
    value: u32,
) {
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        if instance.state_blob.len() < offset + 4 {
            instance.state_blob.resize(offset + 4, 0);
        }
        instance.state_blob[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_mbzhonghui(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    action: u8,
    amount: u32,
    mbsizi_def: engine_api::SkillDefId,
    mbkechang_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let charge = pottuntian_charge(tx, player, mbsizi_def.raw());
    match action {
        0 => {
            if tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || charge == 0
            {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                (1..=charge.min(4)).collect(),
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    MBZHONGHUI_CUSTOM_CHOICE,
                    player.raw(),
                    skill.raw(),
                    mbsizi_def.raw(),
                    mbkechang_def.raw(),
                ],
            )
        }
        1 => {
            if charge >= 4 {
                Ok(Vec::new())
            } else {
                add_mark(tx, player, mbsizi_def.raw(), (4 - charge) as i32)
            }
        }
        2 => {
            let countdown = state_u32(tx, skill, 0);
            if countdown == 0 {
                return Ok(Vec::new());
            }
            set_state_u32(tx, skill, 0, countdown - 1);
            let users: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| {
                    tx.state()
                        .turn
                        .flag(turn_flag::sha_used_by_player(*candidate))
                        > 0
                })
                .collect();
            let mut effects = vec![EffectRequest::DrawCards { player, count: 2 }];
            effects.extend(users.iter().map(|candidate| EffectRequest::DealDamage {
                source: PlayerId::NONE,
                target: *candidate,
                amount: 1,
                is_lose_hp: true,
                nature: rule_api::damage_nature::NORMAL,
            }));
            if users.is_empty()
                && tx
                    .state()
                    .skills
                    .get(skill)
                    .is_some_and(|instance| instance.state_blob.get(4) == Some(&1))
                && tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                effects.push(EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target,
                    amount: 1,
                    is_lose_hp: true,
                    nature: rule_api::damage_nature::NORMAL,
                });
            }
            run_effects(tx, effects)
        }
        3 => {
            let damage_id = tx.state().resolution.current_damage_id;
            if state_u32(tx, skill, 0) > 0
                && bound_sha_damage_matches(tx, damage_id, player, target)
            {
                increase_bound_damage_by_one(tx, damage_id)?;
            }
            Ok(Vec::new())
        }
        4 => {
            let gained = amount.min(4_u32.saturating_sub(charge));
            let mut events = add_mark(tx, player, mbsizi_def.raw(), gained as i32)?;
            if amount > gained {
                set_state_u32(tx, skill, 0, state_u32(tx, skill, 0).saturating_add(1));
                events.push(DomainEvent::RuleNote {
                    code: 553,
                    payload: vec![player.raw(), skill.raw()],
                });
            }
            Ok(events)
        }
        5 => {
            set_state_u32(tx, skill, 0, tx.state().turn.round.saturating_add(1));
            Ok(Vec::new())
        }
        6 => {
            if state_u32(tx, skill, 4) != 0
                || state_u32(tx, skill, 0) != tx.state().turn.round.saturating_add(1)
            {
                return Ok(Vec::new());
            }
            set_state_u32(tx, skill, 4, 1);
            tx.state_mut().group_overrides.insert(player, "qun".into());
            let mut events = if skill_instance_by_def(tx, player, mbkechang_def).is_none() {
                add_skill(tx, player, mbkechang_def)?
            } else {
                Vec::new()
            };
            if let Some(kechang) = skill_instance_by_def(tx, player, mbkechang_def) {
                if let Some(instance) = tx.state_mut().skills.get_mut(kechang) {
                    instance.state_blob = vec![1];
                }
            }
            events.push(DomainEvent::RuleNote {
                code: 553,
                payload: vec![player.raw(), target.raw(), mbkechang_def.raw()],
            });
            Ok(events)
        }
        7 => {
            if tx
                .state()
                .skills
                .get(skill)
                .is_some_and(|instance| instance.state_blob.first() == Some(&1))
            {
                make_current_sha_target_unrespondable(tx, player, CardId::new(amount), target)
            } else {
                Ok(Vec::new())
            }
        }
        8 => {
            if amount == 0 || amount > charge.min(4) {
                return Err(EffectError::Illegal("俟资蓄力消耗已变化".into()));
            }
            add_mark(tx, player, mbsizi_def.raw(), -(amount as i32))?;
            set_state_u32(tx, skill, 0, amount);
            let hp = tx
                .state()
                .players
                .get(player)
                .map_or(0, |p| p.hp.max(0) as u32);
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                if instance.state_blob.len() < 5 {
                    instance.state_blob.resize(5, 0);
                }
                instance.state_blob[4] = u8::from(amount > hp);
            }
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault("unknown 势钟会 action".into())),
    }
}

pub(crate) fn resolve_potjiejie(
    tx: &mut Transaction,
    player: PlayerId,
    actor: PlayerId,
    used_def: engine_api::SkillDefId,
    suit_defs: [engine_api::SkillDefId; 4],
    qingshi_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != actor
        || tx.state().turn.phase != Phase::Play
        || !tx
            .state()
            .players
            .get(actor)
            .is_some_and(|state| state.alive)
        || tx.state().hand_cards(actor).is_empty()
        || skill_instance_by_def(tx, actor, used_def).is_some()
    {
        return Ok(Vec::new());
    }
    if [used_def, qingshi_def]
        .into_iter()
        .chain(suit_defs)
        .any(|def| def.is_none() || tx.state().content.skill_metadata(def).is_none())
    {
        return Err(EffectError::ContentFault(
            "诫节 helper definition missing".into(),
        ));
    }
    let hand = tx.state().hand_cards(actor);
    let suits = hand.iter().fold(0u8, |mask, card| {
        mask | match tx
            .state()
            .cards
            .get(*card)
            .map(|card| crate::query::effective_card_suit(tx.state(), actor, card))
        {
            Some(Suit::Spade) => 1,
            Some(Suit::Heart) => 2,
            Some(Suit::Club) => 4,
            Some(Suit::Diamond) => 8,
            _ => 0,
        }
    });
    let round = tx.state().turn.round;
    let alive_players = tx.state().players.alive_by_seat();
    let bonus = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .is_some_and(|instance| {
            instance.record_potjiejie_suits(round, actor, suits, &alive_players)
        });
    let mut events = add_skill_until_turn_end(tx, actor, used_def)?;
    events.push(DomainEvent::CardsRevealed {
        cards: hand,
        to: vec![player],
    });
    events.extend(open_choice(
        tx,
        player,
        1,
        1,
        false,
        vec![0, 1, 2, 3],
        ChoiceKind::Custom,
        ContinuationKind::Custom,
        vec![
            POTJIEJIE_CUSTOM_CHOICE,
            player.raw(),
            actor.raw(),
            skill.raw(),
            qingshi_def.raw(),
            suit_defs[0].raw(),
            suit_defs[1].raw(),
            suit_defs[2].raw(),
            suit_defs[3].raw(),
            u32::from(bonus),
        ],
    )?);
    Ok(events)
}

pub(crate) fn resolve_potqingshi(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let targets = tx.state().players.alive_by_seat();
    GuardedChoice::new(
        skill,
        Participants::owner_any_state(player),
        ContinuationKind::Custom,
        vec![POTQINGSHI_CUSTOM_CHOICE, player.raw(), skill.raw()],
    )
    .shape(ChoiceShape::one_optional())
    .open_targets(tx, targets)
}

pub(crate) fn resolve_zhengrong(
    tx: &mut Transaction,
    player: PlayerId,
    candidates: &[PlayerId],
    target: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            let owner_hand = tx.state().hand_count(player);
            let richer_with_cards: Vec<_> = candidates
                .iter()
                .copied()
                .filter(|&candidate| {
                    candidate != player
                        && tx.state().players.get(candidate).is_some_and(|p| p.alive)
                        && tx.state().hand_count(candidate) >= owner_hand
                        && !crate::query::discardable_hand_equip_cards(tx.state(), candidate)
                            .is_empty()
                })
                .collect();
            GuardedChoice::new(
                skill,
                Participants::owner(player),
                ContinuationKind::Custom,
                vec![
                    RESOLVE_ZHENGRONG_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    PlayerId::NONE.raw(),
                    skill.raw(),
                ],
            )
            .open_targets(tx, richer_with_cards)
        }
        _ => Err(stage_must_resume_via_continuation("征荣")),
    }
}

pub(crate) fn resolve_hongju(
    tx: &mut Transaction,
    player: PlayerId,
    qingjian_def: engine_api::SkillDefId,
    rong_def: engine_api::SkillDefId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            // 业务前置：荣标记不足或已觉醒（持有清俭）则不再结算。
            if tx.state().skill_pile_cards(player, rong_def.raw()).len() < 3 {
                return Ok(Vec::new());
            }
            if tx
                .state()
                .skills
                .sorted_live()
                .iter()
                .any(|instance| instance.owner == player && instance.def == qingjian_def)
            {
                return Ok(Vec::new());
            }
            // 技能启用 + 存活 + 自己回合由 GuardedChoice 守卫（ADR 0595）。
            GuardedChoice::new(
                skill,
                Participants::owner(player),
                ContinuationKind::Custom,
                vec![
                    RESOLVE_HONGJU_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    qingjian_def.raw(),
                    rong_def.raw(),
                    skill.raw(),
                ],
            )
            .during(TurnContext::OwnersTurn)
            .shape(ChoiceShape::one_optional())
            .open_tokens(tx, vec![1, 0], ChoiceKind::Custom)
        }
        _ => Err(EffectError::ContentFault(
            "鸿举 stage must resolve through its choice continuation".into(),
        )),
    }
}

pub(crate) fn resolve_qingjian(
    tx: &mut Transaction,
    player: PlayerId,
    rong_def: engine_api::SkillDefId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            // 荣牌堆有牌才能赠，且赠牌需要有手牌可换。
            if tx.state().hand_cards(player).is_empty() {
                return Ok(Vec::new());
            }
            let rong = tx.state().skill_pile_cards(player, rong_def.raw());
            GuardedChoice::new(
                skill,
                Participants::owner(player),
                ContinuationKind::Custom,
                vec![
                    RESOLVE_QINGJIAN_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    rong_def.raw(),
                    skill.raw(),
                ],
            )
            .during(TurnContext::OwnersPhase(Phase::Play))
            .open_cards(tx, rong)
        }
        _ => Err(stage_must_resume_via_continuation("清俭")),
    }
}

pub(crate) fn resolve_releiji(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            let others: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&candidate| candidate != player)
                .collect();
            GuardedChoice::new(
                skill,
                Participants::owner(player),
                ContinuationKind::Custom,
                vec![
                    RESOLVE_RELEIJI_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    PlayerId::NONE.raw(),
                    skill.raw(),
                ],
            )
            .open_targets(tx, others)
        }
        _ => Err(stage_must_resume_via_continuation("雷击")),
    }
}

pub(crate) fn resolve_huangtian(
    tx: &mut Transaction,
    player: PlayerId,
    actor: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            // 仅群雄角色可响应黄天。
            if tx.state().player_group(actor) != Some("qun") {
                return Ok(Vec::new());
            }
            let shan_like: Vec<_> = tx
                .state()
                .hand_cards(actor)
                .into_iter()
                .filter(|card| {
                    tx.state().cards.get(*card).is_some_and(|value| {
                        matches!(
                            crate::query::effective_card_kind(tx.state(), actor, value),
                            CardKind::Shan | CardKind::TrickShandian
                        )
                    })
                })
                .collect();
            GuardedChoice::new(
                skill,
                Participants::pair(player, actor),
                ContinuationKind::Custom,
                vec![
                    RESOLVE_HUANGTIAN_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    actor.raw(),
                    skill.raw(),
                ],
            )
            .chosen_by(actor)
            .shape(ChoiceShape::one_optional())
            .open_cards(tx, shan_like)
        }
        _ => Err(stage_must_resume_via_continuation("黄天")),
    }
}
