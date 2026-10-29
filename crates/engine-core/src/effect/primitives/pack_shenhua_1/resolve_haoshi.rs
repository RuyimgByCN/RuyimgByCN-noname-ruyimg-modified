//! Effect primitives for `guanxing_top_by_falu_marks` and related helpers.

use super::*;

pub(crate) fn resolve_haoshi(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Draw
                || tx.state().turn.draw_phase_is_fixed()
            {
                return Ok(Vec::new());
            }
            let count = tx
                .state()
                .turn
                .draw_phase_fixed_count()
                .unwrap_or(2)
                .saturating_add(2);
            tx.state_mut().turn.set_draw_phase_fixed_count(count);
            Ok(Vec::new())
        }
        1 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || tx.state().turn.current != player
                || tx.state().hand_count(player) <= 5
            {
                return Ok(Vec::new());
            }
            let min_hand = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&candidate| candidate != player)
                .map(|candidate| (candidate, tx.state().hand_count(candidate)))
                .min_by_key(|(_, count)| *count)
                .map(|(candidate, _)| candidate);
            let Some(target) = min_hand else {
                return Ok(Vec::new());
            };
            let give = tx.state().hand_count(player) / 2;
            if give == 0 {
                return Ok(Vec::new());
            }
            let give_u8 = give.min(u8::MAX as u32) as u8;
            // 目标已确定（手牌最少者），直接开选牌。
            open_choice(
                tx,
                player,
                give_u8,
                give_u8,
                false,
                tx.state()
                    .hand_cards(player)
                    .into_iter()
                    .map(CardId::raw)
                    .collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_HAOSHI_CUSTOM_CHOICE,
                    2,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )
        }
        _ => Err(EffectError::ContentFault(
            "好施 stage must resolve through its choice continuation".into(),
        )),
    }
}

pub(crate) fn resolve_dimeng(
    tx: &mut Transaction,
    player: PlayerId,
    targets: &[PlayerId],
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
                vec![RESOLVE_DIMENG_CUSTOM_CHOICE, 1, player.raw(), skill.raw()],
            )
            .during(TurnContext::OwnersPhase(Phase::Play))
            .shape(ChoiceShape::exactly(2))
            .open_targets(tx, others)
        }
        _ => Err(stage_must_resume_via_continuation("缔盟")),
    }
}

pub(crate) fn exclude_saved_sha_target(
    tx: &mut Transaction,
    card: CardId,
    target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|player| player.alive)
        || !tx.state().cards.get(card).is_some_and(|value| {
            crate::query::effective_card_kind(tx.state(), target, value).is_sha()
        })
    {
        return Err(EffectError::Illegal(
            "exclude-saved-sha-target participant no longer legal".into(),
        ));
    }
    if !tx
        .state_mut()
        .resolution
        .exclude_saved_sha_target(card, target)
    {
        return Err(EffectError::Illegal(
            "no matching currently resolving Sha target".into(),
        ));
    }
    Ok(vec![DomainEvent::RuleNote {
        code: 560,
        payload: vec![card.raw(), target.raw()],
    }])
}

pub(crate) fn resolve_qiaobian(
    tx: &mut Transaction,
    player: PlayerId,
    phase: u8,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            // phase 参数选中要跳过的阶段，须与当前阶段一致。
            let phase_ok = match phase {
                0 => tx.state().turn.phase == Phase::Judge,
                1 => tx.state().turn.phase == Phase::Draw,
                2 => tx.state().turn.phase == Phase::Play,
                3 => tx.state().turn.phase == Phase::Discard,
                _ => false,
            };
            if !phase_ok {
                return Ok(Vec::new());
            }
            let hand = tx.state().hand_cards(player);
            GuardedChoice::new(
                skill,
                Participants::owner(player),
                ContinuationKind::Custom,
                vec![
                    RESOLVE_QIAOBIAN_DISCARD,
                    1,
                    u32::from(phase),
                    player.raw(),
                    skill.raw(),
                ],
            )
            .during(TurnContext::OwnersTurn)
            .shape(ChoiceShape::one_optional())
            .open_cards(tx, hand)
        }
        _ => Err(stage_must_resume_via_continuation("巧变")),
    }
}

pub(crate) fn resolve_xiangle(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            let choice = GuardedChoice::new(
                skill,
                Participants::distinct_pair(player, source),
                ContinuationKind::Custom,
                vec![
                    RESOLVE_XIANGLE_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    source.raw(),
                    card.raw(),
                    skill.raw(),
                ],
            )
            .chosen_by(source)
            .shape(ChoiceShape::one_optional());
            if !choice.guard_passes(tx)
                || !tx.state().cards.get(card).is_some_and(|value| {
                    crate::query::effective_card_kind(tx.state(), source, value).is_sha()
                })
            {
                return Ok(Vec::new());
            }
            let basics: Vec<_> = tx
                .state()
                .hand_cards(source)
                .into_iter()
                .filter(|&candidate| {
                    tx.state()
                        .cards
                        .get(candidate)
                        .is_some_and(|value| value.kind.is_basic())
                })
                .collect();
            if basics.is_empty() {
                // 无可弃基本牌：直接令此杀无效。
                return exclude_saved_sha_target(tx, card, player);
            }
            choice.open_cards(tx, basics)
        }
        _ => Err(stage_must_resume_via_continuation("享乐")),
    }
}

/// 放权：跳过本回合出牌阶段并标记结束阶段可授额外回合。
pub(crate) fn resolve_fangquan_skip(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx.state().players.get(player).is_some_and(|p| p.alive)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || tx
            .state()
            .skills
            .get(skill)
            .is_some_and(|instance| instance.state_blob == b"FQ1")
    {
        return Ok(Vec::new());
    }
    tx.state_mut()
        .skills
        .get_mut(skill)
        .ok_or_else(|| EffectError::EngineFault("放权 skill disappeared".into()))?
        .state_blob = b"FQ1".to_vec();
    run_effects(tx, vec![EffectRequest::SkipCurrentPhase { player }])
}

/// 放权：结束阶段弃一张手牌并令一名其他角色进行一个额外回合。
pub(crate) fn resolve_fangquan_grant(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            // 仅在本回合已跳过出牌阶段（FQ1）后于结束阶段兑现。
            if !tx
                .state()
                .skills
                .get(skill)
                .is_some_and(|instance| instance.state_blob == b"FQ1")
            {
                return Ok(Vec::new());
            }
            let hand = tx.state().hand_cards(player);
            GuardedChoice::new(
                skill,
                Participants::owner(player),
                ContinuationKind::Custom,
                vec![RESOLVE_FANGQUAN_DISCARD, 1, player.raw(), skill.raw()],
            )
            .during(TurnContext::OwnersPhase(Phase::End))
            .shape(ChoiceShape::one_optional())
            .open_cards(tx, hand)
        }
        _ => Err(stage_must_resume_via_continuation("放权")),
    }
}

/// 若愚（主公技）：准备阶段体力值为全场最少时觉醒，加上限、回复并授予激将。
pub(crate) fn resolve_ruoyu(
    tx: &mut Transaction,
    player: PlayerId,
    jijiang_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx.state().players.get(player).is_some_and(|p| p.alive)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Prepare
        || tx
            .state()
            .skills
            .get(skill)
            .is_some_and(|instance| instance.state_blob == b"RY1")
    {
        return Ok(Vec::new());
    }
    let hp = tx
        .state()
        .players
        .get(player)
        .map(|p| p.hp)
        .unwrap_or(i32::MAX);
    let min_hp = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .map(|candidate| {
            tx.state()
                .players
                .get(candidate)
                .map(|p| p.hp)
                .unwrap_or(i32::MAX)
        })
        .min()
        .unwrap_or(hp);
    if hp != min_hp {
        return Ok(Vec::new());
    }
    tx.state_mut()
        .skills
        .get_mut(skill)
        .ok_or_else(|| EffectError::EngineFault("若愚 skill disappeared".into()))?
        .state_blob = b"RY1".to_vec();
    run_effects(
        tx,
        vec![
            EffectRequest::ChangeMaxHp { player, delta: 1 },
            EffectRequest::RecoverHp {
                player,
                amount: 1,
                source: PlayerId::NONE,
            },
            EffectRequest::AddSkill {
                player,
                skill_def: jijiang_def,
            },
        ],
    )
}

/// 罪论：揭示牌堆顶三张并按条件计数分派；N==3 时选择一名其他角色一起失体力。
pub(crate) fn resolve_zuilun(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::End
            {
                return Ok(Vec::new());
            }
            let lost = tx.state().turn.flag(turn_flag::DISCARD_PHASE_DISCARDED) >= 1
                || tx
                    .state()
                    .turn
                    .lost_cards
                    .iter()
                    .any(|(owner, _)| *owner == player);
            let hand = tx.state().hand_count(player);
            let min_hand = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .all(|candidate| tx.state().hand_count(candidate) >= hand);
            let undamaged = !tx.state().turn.player_received_damage(player);
            let count = u32::from(lost) + u32::from(min_hand) + u32::from(undamaged);
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
            if tx
                .state()
                .zones
                .get(draw)
                .is_none_or(|zone| zone.cards.is_empty())
            {
                reshuffle_discard_into_draw(tx)?;
            }
            let revealed = tx
                .state()
                .zones
                .get(draw)
                .map(|zone| zone.cards.iter().rev().take(3).copied().collect::<Vec<_>>())
                .unwrap_or_default();
            if revealed.is_empty() {
                return Ok(Vec::new());
            }
            let processing = tx
                .state()
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
            for &card in &revealed {
                move_one_card(tx, card, draw, processing, PlayerId::NONE, true)?;
            }
            let hand_zone = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
            if count == 0 {
                return crate::effect::run_effects(
                    tx,
                    vec![EffectRequest::MoveCards {
                        cards: revealed,
                        to: hand_zone,
                        from: Some(processing),
                    }],
                );
            }
            if count == 3 {
                // 全部置顶，并选择一名其他角色一起失去 1 点体力。
                crate::effect::run_effects(
                    tx,
                    vec![EffectRequest::MoveCards {
                        cards: revealed.clone(),
                        to: draw,
                        from: Some(processing),
                    }],
                )?;
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
                return open_choice(
                    tx,
                    player,
                    1,
                    1,
                    true,
                    options.into_iter().map(PlayerId::raw).collect(),
                    ChoiceKind::SelectTargets,
                    ContinuationKind::Custom,
                    vec![RESOLVE_ZUILUN_TARGET, 2, player.raw(), skill.raw()],
                );
            }
            let mut payload = vec![RESOLVE_ZUILUN_PICK, 1, player.raw(), skill.raw(), count];
            payload.extend(revealed.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                count.min(u8::MAX as u32) as u8,
                count.min(u8::MAX as u32) as u8,
                false,
                revealed.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault(
            "罪论 stage must resolve through its choice continuation".into(),
        )),
    }
}

/// 父荫：本回合首次被手牌数不小于自己的其他角色以【杀】指定为目标时，此杀无效。
pub(crate) fn resolve_fuyin(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx.state().players.get(player).is_some_and(|p| p.alive)
        || !tx.state().players.get(source).is_some_and(|p| p.alive)
        || source == player
        || !tx.state().cards.get(card).is_some_and(|value| {
            crate::query::effective_card_kind(tx.state(), source, value).is_sha()
        })
        || tx.state().hand_count(source) < tx.state().hand_count(player)
    {
        return Ok(Vec::new());
    }
    let round = tx.state().turn.round;
    let used_this_round = tx
        .state()
        .skills
        .get(skill)
        .and_then(|instance| {
            (instance.state_blob.len() == 7 && &instance.state_blob[..3] == b"FY1").then(|| {
                (u32::from_le_bytes(
                    instance.state_blob[3..7].try_into().unwrap(),
                ),)
            })
        })
        .is_some_and(|(marked_round,)| marked_round == round);
    if used_this_round {
        return Ok(Vec::new());
    }
    let mut blob = b"FY1".to_vec();
    blob.extend_from_slice(&round.to_le_bytes());
    tx.state_mut()
        .skills
        .get_mut(skill)
        .ok_or_else(|| EffectError::EngineFault("父荫 skill disappeared".into()))?
        .state_blob = blob;
    exclude_saved_sha_target(tx, card, player)
}

/// 回合内禁用指定角色对另一角色使用牌。
pub(crate) fn ban_use_against(
    tx: &mut Transaction,
    user: PlayerId,
    target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if user == target
        || !tx.state().players.get(user).is_some_and(|p| p.alive)
        || !tx.state().players.get(target).is_some_and(|p| p.alive)
    {
        return Err(EffectError::Illegal(
            "ban-use-against participant no longer legal".into(),
        ));
    }
    let pairs = &mut tx.state_mut().turn.use_ban_pairs;
    if !pairs.contains(&(user, target)) {
        pairs.push((user, target));
    }
    Ok(vec![DomainEvent::RuleNote {
        code: 571,
        payload: vec![user.raw(), target.raw()],
    }])
}

/// 拒战（转换技）：阴——受杀各摸一张并禁来源用牌；阳——用杀获目标一张牌并禁己方用牌。
pub(crate) fn resolve_juzhan(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
                || !tx.state().cards.get(card).is_some_and(|value| {
                    crate::query::effective_card_kind(tx.state(), source, value).is_sha()
                })
            {
                return Ok(Vec::new());
            }
            // 转换技分支：技能状态首字节为 0（阴，受杀）或 1（阳，用杀）。
            let branch = tx
                .state()
                .skills
                .get(skill)
                .and_then(|instance| {
                    (instance.state_blob.len() == 4 && &instance.state_blob[..3] == b"JZ1")
                        .then(|| instance.state_blob[3])
                })
                .unwrap_or(0);
            let mut blob = b"JZ1".to_vec();
            blob.push(if branch == 0 { 1 } else { 0 });
            tx.state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::EngineFault("拒战 skill disappeared".into()))?
                .state_blob = blob;
            let target = if branch == 0 {
                // 阴：source 为杀使用者。
                source
            } else {
                // 阳：source 为杀目标（事件来源之外取首个声明目标）。
                targets
                    .iter()
                    .copied()
                    .find(|&candidate| candidate != player && candidate == source)
                    .or_else(|| {
                        targets
                            .iter()
                            .copied()
                            .find(|&candidate| candidate != player)
                    })
                    .unwrap_or(PlayerId::NONE)
            };
            if branch == 0 {
                // 双方各摸一张；来源本回合不能对持有者使用牌。
                let mut events = Vec::new();
                events.extend(run_effects(
                    tx,
                    vec![
                        EffectRequest::DrawCards { player, count: 1 },
                        EffectRequest::DrawCards {
                            player: source,
                            count: 1,
                        },
                    ],
                )?);
                events.extend(ban_use_against(tx, source, player)?);
                return Ok(events);
            }
            if target == PlayerId::NONE || !tx.state().players.get(target).is_some_and(|p| p.alive)
            {
                return Ok(Vec::new());
            }
            // 阳：获得目标一张手牌/装备牌，随后持有者本回合不能对目标使用牌。
            let options: Vec<_> = tx
                .state()
                .hand_cards(target)
                .into_iter()
                .chain(
                    tx.state()
                        .zones
                        .find(ZoneKind::Equip, target)
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
                return ban_use_against(tx, player, target);
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
                vec![
                    RESOLVE_JUZHAN_GAIN,
                    1,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )
        }
        _ => Err(EffectError::ContentFault(
            "拒战 stage must resolve through its choice continuation".into(),
        )),
    }
}
