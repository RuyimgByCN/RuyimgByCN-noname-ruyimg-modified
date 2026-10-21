//! Command operations for `resolve_twniujin_custom` and related helpers.

use super::*;

pub(crate) fn resolve_twniujin_custom(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    match payload.get(1).copied() {
        Some(0) => {
            if payload.len() != 4 {
                return Err(EffectError::EngineFault(
                    "bad TW Cuorui damage continuation".into(),
                ));
            }
            let player = PlayerId::new(payload[2]);
            let skill = SkillId::new(payload[3]);
            if observer != player
                || !tx.state().skills.get(skill).is_some_and(|instance| {
                    instance.owner == player && tx.state().skill_effectively_enabled(instance)
                })
            {
                return Err(EffectError::Illegal(
                    "TW Cuorui damage context changed".into(),
                ));
            }
            if cancelled || selections.is_empty() {
                return Ok(());
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "TW Cuorui chooses at most one target".into(),
                ));
            };
            let target = PlayerId::new(*selected);
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("TW Cuorui target is invalid".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::DealDamage {
                    source: player,
                    target,
                    amount: 1,
                    is_lose_hp: false,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )
            .map(|_| ())
        }
        Some(1) => {
            if payload.len() != 5 || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("bad TW Liewei reward choice".into()));
            }
            let player = PlayerId::new(payload[2]);
            let limited_skill_def = engine_api::SkillDefId::new(payload[3]);
            let skill = SkillId::new(payload[4]);
            if observer != player
                || !tx.state().skills.get(skill).is_some_and(|instance| {
                    instance.owner == player && tx.state().skill_effectively_enabled(instance)
                })
            {
                return Err(EffectError::Illegal("TW Liewei context changed".into()));
            }
            match selections[0] {
                0 => {
                    run_effects(tx, vec![EffectRequest::DrawCards { player, count: 2 }]).map(|_| ())
                }
                1 => {
                    let limited = tx
                        .state()
                        .skills
                        .sorted_live()
                        .into_iter()
                        .find(|instance| {
                            instance.owner == player && instance.def == limited_skill_def
                        })
                        .map(|instance| instance.id)
                        .ok_or_else(|| {
                            EffectError::Illegal("TW Cuorui skill disappeared".into())
                        })?;
                    let instance = tx.state_mut().skills.get_mut(limited).unwrap();
                    if instance.lifetime_fire_count == 0 {
                        return Err(EffectError::Illegal(
                            "TW Cuorui is already available".into(),
                        ));
                    }
                    instance.lifetime_fire_count = 0;
                    Ok(())
                }
                _ => Err(EffectError::Illegal("unknown TW Liewei reward".into())),
            }
        }
        _ => Err(EffectError::EngineFault(
            "unknown TW Niujin continuation".into(),
        )),
    }
}

pub(crate) fn resolve_twmoukui_custom(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 5 || payload.first() != Some(&crate::effect::TWMOUKUI_CUSTOM_CHOICE) {
        return Err(EffectError::EngineFault(
            "bad TW Moukui continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    let card = CardId::new(payload[3]);
    let skill = SkillId::new(payload[4]);
    if observer != player {
        return Err(EffectError::Illegal(
            "TW Moukui choice must be resolved by its owner".into(),
        ));
    }
    if cancelled {
        return if selections.is_empty() {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "cancelled TW Moukui selected a branch".into(),
            ))
        };
    }
    if !tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    }) || !tx
        .state()
        .resolution
        .sha_target_resolving(player, card, target)
    {
        return Err(EffectError::Illegal("TW Moukui context changed".into()));
    }
    match selections {
        [1] => run_effects(
            tx,
            vec![EffectRequest::DrawCardsFromSkill {
                player,
                count: 1,
                skill,
            }],
        )
        .map(|_| ()),
        [2] if !crate::query::discardable_target_cards(
            tx.state(),
            player,
            target,
            rule_api::TargetCardScope::HandEquip,
        )
        .is_empty() =>
        {
            run_effects(
                tx,
                vec![EffectRequest::ChooseDiscardTargetCard {
                    player,
                    target,
                    scope: rule_api::TargetCardScope::HandEquip,
                }],
            )
            .map(|_| ())
        }
        [3] => {
            if !tx
                .state_mut()
                .resolution
                .record_sha_backwater_target(player, card, skill, target)
            {
                return Err(EffectError::Illegal(
                    "TW Moukui Sha target disappeared".into(),
                ));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::DrawCardsFromSkill {
                        player,
                        count: 1,
                        skill,
                    },
                    EffectRequest::ChooseDiscardTargetCard {
                        player,
                        target,
                        scope: rule_api::TargetCardScope::HandEquip,
                    },
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal("unknown TW Moukui branch".into())),
    }
}

pub(crate) fn resolve_luanqun(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    match payload.first().copied() {
        Some(0) => {
            if payload.len() < 7 {
                return Err(EffectError::EngineFault(
                    "truncated Luanqun selection payload".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let effect_skill_def = SkillDefId::new(payload[2]);
            let direct_hit_skill_def = SkillDefId::new(payload[3]);
            let skill = SkillId::new(payload[4]);
            let index = payload[5] as usize;
            let count = payload[6] as usize;
            if count == 0 || index >= count || payload.len() != 7 + count + index {
                return Err(EffectError::EngineFault(
                    "bad Luanqun selection payload".into(),
                ));
            }
            let participants: Vec<_> = payload[7..7 + count]
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect();
            let mut cards: Vec<_> = payload[7 + count..]
                .iter()
                .copied()
                .map(CardId::new)
                .collect();
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "Luanqun requires exactly one hand card".into(),
                ));
            };
            let selected = CardId::new(*selected);
            let chooser = participants[index];
            let hand = tx.state().zones.find(ZoneKind::Hand, chooser);
            if cancelled
                || observer != chooser
                || !tx
                    .state()
                    .cards
                    .get(selected)
                    .is_some_and(|card| Some(card.zone) == hand && card.owner == chooser)
            {
                return Err(EffectError::Illegal(
                    "Luanqun private selection context changed".into(),
                ));
            }
            cards.push(selected);
            if index + 1 < count {
                let next = participants[index + 1];
                let options = tx.state().hand_cards(next);
                if options.is_empty() {
                    return Err(EffectError::Illegal(
                        "Luanqun participant lost every hand card".into(),
                    ));
                }
                let mut next_payload = vec![
                    0,
                    player.raw(),
                    effect_skill_def.raw(),
                    direct_hit_skill_def.raw(),
                    skill.raw(),
                    (index + 1) as u32,
                    count as u32,
                ];
                next_payload.extend(participants.iter().map(|player| player.raw()));
                next_payload.extend(cards.iter().map(|card| card.raw()));
                return crate::effect::open_choice(
                    tx,
                    next,
                    1,
                    1,
                    false,
                    options.into_iter().map(CardId::raw).collect(),
                    engine_api::ChoiceKind::SelectCards,
                    ContinuationKind::ResolveLuanqun,
                    next_payload,
                )
                .map(|_| ());
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveLuanqun {
                    player,
                    effect_skill_def,
                    direct_hit_skill_def,
                    stage: 1,
                    participants,
                    cards,
                    skill,
                }],
            )
            .map(|_| ())
        }
        Some(1) => {
            if payload.len() < 4 {
                return Err(EffectError::EngineFault(
                    "truncated Luanqun gain payload".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let skill = SkillId::new(payload[2]);
            let count = payload[3] as usize;
            let options = &payload[4..];
            if options.len() != count || observer != player {
                return Err(EffectError::EngineFault("bad Luanqun gain payload".into()));
            }
            if cancelled {
                if selections.is_empty() {
                    return Ok(());
                }
                return Err(EffectError::Illegal(
                    "cancelled Luanqun gain carried selections".into(),
                ));
            }
            let cap = if tx.state().ruleset.name.starts_with("identity") {
                4
            } else {
                2
            };
            let mut seen = std::collections::BTreeSet::new();
            if selections.is_empty()
                || selections.len() > cap
                || selections
                    .iter()
                    .any(|selected| !options.contains(selected) || !seen.insert(*selected))
                || !tx.state().skills.get(skill).is_some_and(|instance| {
                    instance.owner == player && tx.state().skill_effectively_enabled(instance)
                })
            {
                return Err(EffectError::Illegal(
                    "Luanqun gain selection is no longer legal".into(),
                ));
            }
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("Luanqun owner hand missing".into()))?;
            let mut effects = Vec::new();
            for selected in selections {
                let card = CardId::new(*selected);
                let state = tx
                    .state()
                    .cards
                    .get(card)
                    .ok_or_else(|| EffectError::Illegal("Luanqun card disappeared".into()))?;
                let from = tx.state().zones.get(state.zone).ok_or_else(|| {
                    EffectError::EngineFault("Luanqun source hand missing".into())
                })?;
                if from.kind != ZoneKind::Hand || state.owner == player {
                    return Err(EffectError::Illegal(
                        "Luanqun matching card left its hand".into(),
                    ));
                }
                effects.push(EffectRequest::MoveCards {
                    cards: vec![card],
                    to,
                    from: Some(state.zone),
                });
            }
            run_effects(tx, effects).map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown Luanqun choice stage".into(),
        )),
    }
}

/// Resolve the 择一加技能 (choose one of two skills to grant) custom choice.
///
/// ABI 644 (ADR 0612): the chooser picked option 1 (first_def) or 2 (second_def);
/// the chosen skill definition is granted to `target`.
pub(crate) fn resolve_choose_other_gain_one_of_two_skills(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&crate::effect::CHOOSE_OTHER_GAIN_ONE_OF_TWO_SKILLS_CUSTOM_CHOICE)
        || payload.len() != 7
    {
        return Err(EffectError::EngineFault(
            "malformed choose-one-of-two-skills continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[2]);
    let target = PlayerId::new(payload[3]);
    let first_def = SkillDefId::new(payload[4]);
    let second_def = SkillDefId::new(payload[5]);
    let skill = SkillId::new(payload[6]);
    if observer != player
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "choose-one-of-two-skills context changed".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal(
            "choose exactly one of the two skills".into(),
        ));
    }
    let chosen = match selections[0] {
        1 => first_def,
        2 => second_def,
        _ => {
            return Err(EffectError::Illegal(
                "choose-one-of-two-skills option is out of range".into(),
            ))
        }
    };
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: target,
            skill_def: chosen,
        }],
    )
    .map(|_| ())
}

/// Resolve the 水淹七军 (guozhan) target choice: 1=弃置全部装备牌, 2=受 1 点雷电伤害.
///
/// The queued-trick tail rides in payload[3..]; the chosen effect runs and then the
/// independent trick queue advances (or finishes when the tail is absent).
pub(crate) fn resolve_shuiyanqijunx_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&crate::effect::SHUIYANQIJUNX_CUSTOM_CHOICE) || payload.len() < 3 {
        return Err(EffectError::EngineFault(
            "malformed 水淹七军 continuation".into(),
        ));
    }
    let actor = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    let queue = payload.get(3..).unwrap_or_default();
    if observer != target {
        return Err(EffectError::Illegal(
            "水淹七军 choice observer mismatch".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal(
            "choose exactly one 水淹七军 option".into(),
        ));
    }
    let effects = match selections[0] {
        1 => vec![EffectRequest::DiscardAllTargetEquipment { actor, target }],
        2 => vec![EffectRequest::DealDamage {
            source: actor,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::THUNDER,
        }],
        _ => {
            return Err(EffectError::Illegal(
                "水淹七军 option is out of range".into(),
            ))
        }
    };
    if queue.is_empty() {
        return run_effects(tx, effects).map(|_| ());
    }
    crate::command::play::run_effects_then_finish_independent_trick_target(tx, effects, queue)
}

/// Resolve the 克复中原 (guozhan) target choice: 1=视为使用杀（蜀势力伤害基数+1），
/// 2=摸牌（蜀势力摸两张，否则一张）。
///
/// The queued-trick tail rides in payload[3..]; the chosen effect runs and then the
/// independent trick queue advances (or finishes when the tail is absent).
pub(crate) fn resolve_kefuzhongyuan_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&crate::effect::KEFUZHONGYUAN_CUSTOM_CHOICE) || payload.len() < 3 {
        return Err(EffectError::EngineFault(
            "malformed 克复中原 continuation".into(),
        ));
    }
    let actor = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    let queue = payload.get(3..).unwrap_or_default();
    if observer != target {
        return Err(EffectError::Illegal(
            "克复中原 choice observer mismatch".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal(
            "choose exactly one 克复中原 option".into(),
        ));
    }
    let is_shu = tx
        .state()
        .player_group(target)
        .map(str::to_string)
        .as_deref()
        == Some("shu");
    let effects = match selections[0] {
        1 => vec![EffectRequest::ChooseUseVirtualShaWithBonus {
            player: target,
            ignore_distance: false,
            damage_bonus: if is_shu { 1 } else { 0 },
        }],
        2 => vec![EffectRequest::DrawCards {
            player: target,
            count: if is_shu { 2 } else { 1 },
        }],
        _ => {
            return Err(EffectError::Illegal(
                "克复中原 option is out of range".into(),
            ))
        }
    };
    if queue.is_empty() {
        return run_effects(tx, effects).map(|_| ());
    }
    crate::command::play::run_effects_then_finish_independent_trick_target(tx, effects, queue)
}

/// Resolve the 固国安邦 (guozhan) staged settlement.
///
/// Stage 1: discard ≥ 6 hand cards. Stage 2: if 吴, pick 1–2 of the discarded cards to
/// give. Stage 3: pick one other 吴 target to receive them. Single give round (上游循环
/// 给多目标为近似省略).
pub(crate) fn resolve_guguoanbang_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&crate::effect::GUGUOANBANG_CUSTOM_CHOICE) || payload.len() < 4 {
        return Err(EffectError::EngineFault(
            "malformed 固国安邦 continuation".into(),
        ));
    }
    let stage = payload[1];
    let actor = PlayerId::new(payload[2]);
    let target = PlayerId::new(payload[3]);
    let tail = payload.get(4..).unwrap_or_default();
    if observer != target {
        return Err(EffectError::Illegal(
            "固国安邦 choice observer mismatch".into(),
        ));
    }
    let finish = |tx: &mut Transaction| -> Result<(), EffectError> {
        crate::command::play::finish_independent_trick_target(tx, tail)
    };
    match stage {
        1 => {
            if cancelled {
                return finish(tx);
            }
            if selections.len() < 6 {
                return Err(EffectError::Illegal(
                    "固国安邦 requires discarding at least six hand cards".into(),
                ));
            }
            let hand = tx.state().hand_cards(target);
            if selections
                .iter()
                .any(|raw| !hand.contains(&CardId::new(*raw)))
            {
                return Err(EffectError::Illegal(
                    "固国安邦 discard selection is not in hand".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: target,
                    cards: selections.iter().copied().map(CardId::new).collect(),
                }],
            )?;
            let is_wu = tx
                .state()
                .player_group(target)
                .map(str::to_string)
                .as_deref()
                == Some("wu");
            let other_wu: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| {
                    *candidate != target
                        && tx
                            .state()
                            .player_group(*candidate)
                            .map(str::to_string)
                            .as_deref()
                            == Some("wu")
                })
                .collect();
            if !is_wu || selections.is_empty() || other_wu.is_empty() {
                return finish(tx);
            }
            let mut next = vec![
                crate::effect::GUGUOANBANG_CUSTOM_CHOICE,
                2,
                actor.raw(),
                target.raw(),
            ];
            next.push(selections.len().min(2) as u32);
            next.extend(selections.iter().copied());
            next.extend_from_slice(tail);
            crate::effect::open_choice(
                tx,
                target,
                1,
                selections.len().min(2) as u8,
                false,
                selections.to_vec(),
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::Custom,
                next,
            )?;
            Err(EffectError::ChoiceRequested)
        }
        2 => {
            if cancelled {
                return finish(tx);
            }
            if payload.len() < 5 {
                return Err(EffectError::EngineFault(
                    "truncated 固国安邦 give-cards payload".into(),
                ));
            }
            let give_count = payload[4] as usize;
            let give_cards: Vec<_> = payload
                .get(5..5 + give_count)
                .ok_or_else(|| EffectError::EngineFault("truncated give cards".into()))?
                .iter()
                .copied()
                .map(CardId::new)
                .collect();
            let tail2 = payload.get(5 + give_count..).unwrap_or_default();
            if selections.is_empty()
                || selections.len() > give_count
                || selections
                    .iter()
                    .any(|raw| !give_cards.contains(&CardId::new(*raw)))
            {
                return Err(EffectError::Illegal(
                    "固国安邦 give-card selection is invalid".into(),
                ));
            }
            let other_wu: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| {
                    *candidate != target
                        && tx
                            .state()
                            .player_group(*candidate)
                            .map(str::to_string)
                            .as_deref()
                            == Some("wu")
                })
                .collect();
            if other_wu.is_empty() {
                return finish(tx);
            }
            let mut next = vec![
                crate::effect::GUGUOANBANG_CUSTOM_CHOICE,
                3,
                actor.raw(),
                target.raw(),
            ];
            next.push(selections.len() as u32);
            next.extend(selections.iter().copied());
            next.extend_from_slice(tail2);
            crate::effect::open_choice(
                tx,
                target,
                1,
                1,
                false,
                other_wu.into_iter().map(PlayerId::raw).collect(),
                engine_api::ChoiceKind::SelectTargets,
                crate::state::ContinuationKind::Custom,
                next,
            )?;
            Err(EffectError::ChoiceRequested)
        }
        3 => {
            if cancelled || selections.len() != 1 {
                return finish(tx);
            }
            if payload.len() < 5 {
                return Err(EffectError::EngineFault(
                    "truncated 固国安邦 give-target payload".into(),
                ));
            }
            let give_count = payload[4] as usize;
            let give_cards: Vec<_> = payload
                .get(5..5 + give_count)
                .ok_or_else(|| EffectError::EngineFault("truncated give cards".into()))?
                .iter()
                .copied()
                .map(CardId::new)
                .collect();
            let tail3 = payload.get(5 + give_count..).unwrap_or_default();
            let recipient = PlayerId::new(selections[0]);
            if recipient == target
                || !tx.state().players.get(recipient).is_some_and(|p| p.alive)
                || tx
                    .state()
                    .player_group(recipient)
                    .map(str::to_string)
                    .as_deref()
                    != Some("wu")
            {
                return Err(EffectError::Illegal(
                    "固国安邦 recipient must be another living 吴 player".into(),
                ));
            }
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, recipient)
                .ok_or_else(|| EffectError::Illegal("recipient hand missing".into()))?;
            let from = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("discard pile missing".into()))?;
            run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: give_cards,
                    to,
                    from: Some(from),
                }],
            )?;
            crate::command::play::finish_independent_trick_target(tx, tail3)
        }
        _ => Err(EffectError::Illegal(
            "固国安邦 stage is out of range".into(),
        )),
    }
}

/// Resolve the 联军盛宴 (guozhan) draw/recover split for the user: the picked token is
/// the recover amount; draw = X − recover. The queued-trick tail rides after [magic,
/// actor, X].
pub(crate) fn resolve_lianjunshengyan_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&crate::effect::LIANJUNSHENGYAN_CUSTOM_CHOICE) || payload.len() < 3 {
        return Err(EffectError::EngineFault(
            "malformed 联军盛宴 continuation".into(),
        ));
    }
    let actor = PlayerId::new(payload[1]);
    let x = payload[2];
    let tail = payload.get(3..).unwrap_or_default();
    if observer != actor {
        return Err(EffectError::Illegal(
            "联军盛宴 choice observer mismatch".into(),
        ));
    }
    if cancelled || selections.len() != 1 {
        return crate::command::play::finish_independent_trick_target(tx, tail);
    }
    let recover = selections[0];
    let participant = tx
        .state()
        .players
        .get(actor)
        .ok_or_else(|| EffectError::Illegal("user missing".into()))?;
    let damaged = participant
        .max_hp
        .saturating_sub(participant.hp.max(0) as u32);
    let cap = x.min(damaged);
    if recover > cap {
        return Err(EffectError::Illegal(
            "联军盛宴 recover amount exceeds the damage cap".into(),
        ));
    }
    let draw = x.saturating_sub(recover);
    let mut effects = Vec::with_capacity(2);
    if draw > 0 {
        effects.push(EffectRequest::DrawCards {
            player: actor,
            count: draw,
        });
    }
    if recover > 0 {
        effects.push(EffectRequest::RecoverHp {
            source: actor,
            player: actor,
            amount: recover,
        });
    }
    if tail.is_empty() {
        return run_effects(tx, effects).map(|_| ());
    }
    crate::command::play::run_effects_then_finish_independent_trick_target(tx, effects, tail)
}

/// Resolve the 文和乱武 (guozhan) staged settlement.
///
/// Stage 1: the card user (actor) picks ① 令目标弃置两张类型不同的手牌 or ② 弃置其一张
/// 手牌. Stage 2: the discard choice — for ① the target picks 2 (or 1 when only a single
/// card type is available) of their hand cards; for ② the actor picks 1 of the target's
/// hand cards. After the discard, a 群 faction target with an empty hand draws up to its
/// current HP (capped at 5), mirroring upstream `target.draw(Math.min(5, target.hp))`.
/// Payload: [magic, stage, actor, target, discarder, count, ...queued-tail].
pub(crate) fn resolve_wenheluanwu_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&crate::effect::WENHELUANWU_CUSTOM_CHOICE) || payload.len() < 4 {
        return Err(EffectError::EngineFault(
            "malformed 文和乱武 continuation".into(),
        ));
    }
    let stage = payload[1];
    let actor = PlayerId::new(payload[2]);
    let target = PlayerId::new(payload[3]);
    match stage {
        1 => {
            if observer != actor {
                return Err(EffectError::Illegal(
                    "文和乱武 stage-1 choice belongs to the user".into(),
                ));
            }
            let tail = payload.get(4..).unwrap_or_default();
            let finish = |tx: &mut Transaction| -> Result<(), EffectError> {
                crate::command::play::finish_independent_trick_target(tx, tail)
            };
            if cancelled {
                return finish(tx);
            }
            if selections.len() != 1 || !matches!(selections[0], 1 | 2) {
                return Err(EffectError::Illegal(
                    "choose exactly one 文和乱武 option".into(),
                ));
            }
            let hand = tx.state().hand_cards(target);
            if hand.is_empty() {
                return finish(tx);
            }
            let (discarder, count) = if selections[0] == 1 {
                // ① 令目标弃置两张类型不同的手牌；仅一种类型可弃时退化为弃一张。
                let mut groups = std::collections::BTreeSet::new();
                for cid in &hand {
                    groups.insert(
                        tx.state()
                            .cards
                            .get(*cid)
                            .map(|c| c.kind.card_type_group())
                            .unwrap_or(0),
                    );
                }
                (1, if groups.len() >= 2 { 2 } else { 1 })
            } else {
                (2, 1)
            };
            let mut next = vec![
                crate::effect::WENHELUANWU_CUSTOM_CHOICE,
                2,
                actor.raw(),
                target.raw(),
                discarder,
                count,
            ];
            next.extend_from_slice(tail);
            let chooser = if discarder == 1 { target } else { actor };
            crate::effect::open_choice(
                tx,
                chooser,
                count as u8,
                count as u8,
                false,
                hand.into_iter().map(|cid| cid.raw()).collect(),
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::Custom,
                next,
            )?;
            Err(EffectError::ChoiceRequested)
        }
        2 => {
            if payload.len() < 6 {
                return Err(EffectError::EngineFault(
                    "truncated 文和乱武 discard payload".into(),
                ));
            }
            let discarder = payload[4];
            let count = payload[5] as usize;
            let tail = payload.get(6..).unwrap_or_default();
            let finish = |tx: &mut Transaction| -> Result<(), EffectError> {
                crate::command::play::finish_independent_trick_target(tx, tail)
            };
            if cancelled {
                return finish(tx);
            }
            let expected_observer = if discarder == 1 { target } else { actor };
            if observer != expected_observer || !matches!(discarder, 1 | 2) {
                return Err(EffectError::Illegal(
                    "文和乱武 discard choice observer mismatch".into(),
                ));
            }
            if count == 0 || count > 2 || selections.len() != count {
                return Err(EffectError::Illegal(
                    "文和乱武 discard selection count is invalid".into(),
                ));
            }
            let hand = tx.state().hand_cards(target);
            if selections
                .iter()
                .any(|raw| !hand.contains(&CardId::new(*raw)))
            {
                return Err(EffectError::Illegal(
                    "文和乱武 discard selection is not in target hand".into(),
                ));
            }
            if count == 2 {
                let groups: Vec<u8> = selections
                    .iter()
                    .map(|raw| {
                        tx.state()
                            .cards
                            .get(CardId::new(*raw))
                            .map(|c| c.kind.card_type_group())
                            .unwrap_or(0)
                    })
                    .collect();
                if groups[0] == groups[1] {
                    return Err(EffectError::Illegal(
                        "文和乱武 requires two hand cards of different types".into(),
                    ));
                }
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: target,
                    cards: selections.iter().copied().map(CardId::new).collect(),
                }],
            )?;
            // 群势力且弃后空手：摸至当前体力（至多 5）。弃牌已完成，按弃后手牌判断。
            let is_qun = tx
                .state()
                .player_group(target)
                .map(str::to_string)
                .as_deref()
                == Some("qun");
            if is_qun && tx.state().hand_cards(target).is_empty() {
                let hp = tx
                    .state()
                    .players
                    .get(target)
                    .map(|p| p.hp.max(0) as u32)
                    .unwrap_or(0);
                if hp > 0 {
                    run_effects(
                        tx,
                        vec![EffectRequest::DrawCards {
                            player: target,
                            count: 5.min(hp),
                        }],
                    )?;
                }
            }
            finish(tx)
        }
        _ => Err(EffectError::Illegal(
            "文和乱武 stage is out of range".into(),
        )),
    }
}

/// Resolve the 号令天下 (guozhan) per-player chain.
///
/// Payload: `[magic, stage, mode?, ...chain_queue]` where chain_queue is the
/// independent-trick-queue superset documented at `advance_haolingtianxia_player`.
/// Stage 1 = option choice (observer = current chain player): ① 弃牌视为对目标使用杀
/// （魏势力免弃牌）, ② 弃置（魏势力改为获得）目标一张牌. Stage 2 = card pick:
/// mode 1 = 出杀费用弃本人「he」牌, mode 2 = 弃目标「he」牌, mode 3 = 魏获得目标
/// 「he」牌. Any card-moving effect may suspend on a trigger choice; the whole step is
/// run through the IndependentTrickQueue resume so the chain continues afterwards.
pub(crate) fn resolve_haolingtianxia_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&crate::effect::HAOLINGTIANXIA_CUSTOM_CHOICE) || payload.len() < 11 {
        return Err(EffectError::EngineFault(
            "malformed 号令天下 continuation".into(),
        ));
    }
    let stage = payload[1];
    let queue = if stage == 1 {
        &payload[2..]
    } else {
        &payload[3..]
    };
    if queue.len() < 8 {
        return Err(EffectError::EngineFault(
            "truncated 号令天下 chain queue".into(),
        ));
    }
    let index = queue[5] as usize;
    let len = queue[6] as usize;
    let current = PlayerId::new(queue[7 + index]);
    let trick_target = PlayerId::new(queue[7 + len]);
    if observer != current {
        return Err(EffectError::Illegal(
            "号令天下 choice observer mismatch".into(),
        ));
    }
    let is_wei = tx
        .state()
        .player_group(current)
        .map(str::to_string)
        .as_deref()
        == Some("wei");
    let advance = |tx: &mut Transaction| -> Result<(), EffectError> {
        let mut next = queue.to_vec();
        next[5] = next[5].saturating_add(1);
        crate::command::play::advance_haolingtianxia_player(tx, &next)
    };
    let step_effects =
        |tx: &mut Transaction, effects: Vec<EffectRequest>| -> Result<(), EffectError> {
            let mut next = queue.to_vec();
            next[5] = next[5].saturating_add(1);
            crate::command::play::run_effects_then_resume_trick_queue(tx, effects, next)
        };
    match stage {
        1 => {
            if cancelled {
                return advance(tx);
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "choose exactly one 号令天下 option".into(),
                ));
            }
            match selections[0] {
                1 => {
                    // ① 弃一张牌（魏免），视为对目标使用杀。
                    if is_wei {
                        return step_effects(
                            tx,
                            vec![EffectRequest::DealDamage {
                                source: current,
                                target: trick_target,
                                amount: 1,
                                is_lose_hp: false,
                                nature: rule_api::damage_nature::NORMAL,
                            }],
                        );
                    }
                    let tokens = crate::command::play::target_zone_card_tokens(
                        tx, current, true, true, false,
                    );
                    if tokens.is_empty() {
                        return advance(tx);
                    }
                    let mut next = vec![
                        crate::effect::HAOLINGTIANXIA_CUSTOM_CHOICE,
                        2,
                        1, // mode: sha cost discard
                    ];
                    next.extend_from_slice(queue);
                    crate::effect::open_choice(
                        tx,
                        current,
                        1,
                        1,
                        true,
                        tokens,
                        engine_api::ChoiceKind::SelectCards,
                        crate::state::ContinuationKind::Custom,
                        next,
                    )?;
                    Err(EffectError::ChoiceRequested)
                }
                2 => {
                    // ② 弃置（魏势力改为获得）目标一张「he」牌。
                    let mode = if is_wei { 3 } else { 2 };
                    let tokens: Vec<u32> = if is_wei {
                        crate::query::gainable_target_cards(
                            tx.state(),
                            current,
                            trick_target,
                            rule_api::TargetCardScope::HandEquip,
                        )
                        .into_iter()
                        .map(|cid| cid.raw())
                        .collect()
                    } else {
                        crate::command::play::target_zone_card_tokens(
                            tx,
                            trick_target,
                            true,
                            true,
                            false,
                        )
                    };
                    if tokens.is_empty() {
                        return advance(tx);
                    }
                    let mut next = vec![crate::effect::HAOLINGTIANXIA_CUSTOM_CHOICE, 2, mode];
                    next.extend_from_slice(queue);
                    crate::effect::open_choice(
                        tx,
                        current,
                        1,
                        1,
                        true,
                        tokens,
                        engine_api::ChoiceKind::SelectCards,
                        crate::state::ContinuationKind::Custom,
                        next,
                    )?;
                    Err(EffectError::ChoiceRequested)
                }
                _ => Err(EffectError::Illegal(
                    "号令天下 option is out of range".into(),
                )),
            }
        }
        2 => {
            if cancelled {
                return advance(tx);
            }
            let mode = payload[2];
            if selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "pick exactly one 号令天下 card".into(),
                ));
            }
            let sel = CardId::new(selections[0]);
            match mode {
                1 => {
                    // 出杀费用：本人弃一张「he」牌，然后对目标造成 1 点伤害。
                    // 近似说明：引擎杀管线（ResumePlayedSha）独占 resume 单槽，杀完成后
                    // 无法续链号令天下链；以直接伤害近似（无闪响应），ADR 0623 备注。
                    let tokens = crate::command::play::target_zone_card_tokens(
                        tx, current, true, true, false,
                    );
                    if !tokens.contains(&selections[0]) {
                        return Err(EffectError::Illegal(
                            "号令天下 sha cost must be a current he card".into(),
                        ));
                    }
                    step_effects(
                        tx,
                        vec![
                            EffectRequest::DiscardCards {
                                player: current,
                                cards: vec![sel],
                            },
                            EffectRequest::DealDamage {
                                source: current,
                                target: trick_target,
                                amount: 1,
                                is_lose_hp: false,
                                nature: rule_api::damage_nature::NORMAL,
                            },
                        ],
                    )
                }
                2 => {
                    // 弃置目标一张「he」牌。
                    let tokens = crate::command::play::target_zone_card_tokens(
                        tx,
                        trick_target,
                        true,
                        true,
                        false,
                    );
                    if !tokens.contains(&selections[0]) {
                        return Err(EffectError::Illegal(
                            "号令天下 discard must be a target he card".into(),
                        ));
                    }
                    step_effects(
                        tx,
                        vec![EffectRequest::DiscardCards {
                            player: trick_target,
                            cards: vec![sel],
                        }],
                    )
                }
                3 => {
                    // 魏势力获得目标一张「he」牌。
                    let gainable = crate::query::gainable_target_cards(
                        tx.state(),
                        current,
                        trick_target,
                        rule_api::TargetCardScope::HandEquip,
                    );
                    if !gainable.contains(&sel) {
                        return Err(EffectError::Illegal(
                            "号令天下 gain must be a gainable target he card".into(),
                        ));
                    }
                    let zone = tx
                        .state()
                        .cards
                        .get(sel)
                        .map(|c| c.zone)
                        .ok_or_else(|| EffectError::Illegal("bad card".into()))?;
                    let to_hand = tx
                        .state()
                        .zones
                        .find(ZoneKind::Hand, current)
                        .ok_or_else(|| EffectError::Illegal("no current hand".into()))?;
                    step_effects(
                        tx,
                        vec![EffectRequest::MoveCards {
                            cards: vec![sel],
                            to: to_hand,
                            from: Some(zone),
                        }],
                    )
                }
                _ => Err(EffectError::Illegal("号令天下 mode is out of range".into())),
            }
        }
        _ => Err(EffectError::Illegal(
            "号令天下 stage is out of range".into(),
        )),
    }
}

/// Resolve the 敕令 (guozhan) per-target choice.
///
/// Payload: `[magic, stage, mode?, target, ...queued-tail]`.
/// Stage 1 = the target picks ① 明置一张武将牌并摸一张 ② 失去1点体力 ③ 弃置一张装备牌
/// （③ 仅在有装备牌时可选，① 的槽位选择在 Stage 2）。Stage 2 mode 1 = 选一个暗置槽
/// 位明置；mode 3 = 选一张装备牌弃置。明置经 `mode::reveal_general`（确定性状态变更），
/// 摸牌经 `DrawCards`。
pub(crate) fn resolve_chiling_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&crate::effect::CHILING_CUSTOM_CHOICE) || payload.len() < 4 {
        return Err(EffectError::EngineFault(
            "malformed 敕令 continuation".into(),
        ));
    }
    let stage = payload[1];
    // Stage 1: [magic, 1, target, ...tail]; Stage 2: [magic, 2, mode, target, ...tail].
    let target = PlayerId::new(payload[if stage == 1 { 2 } else { 3 }]);
    let tail_offset = if stage == 1 { 3 } else { 4 };
    let tail = &payload[tail_offset.min(payload.len())..];
    let finish = |tx: &mut Transaction| -> Result<(), EffectError> {
        crate::command::play::finish_independent_trick_target(tx, tail)
    };
    if observer != target {
        return Err(EffectError::Illegal("敕令 choice observer mismatch".into()));
    }
    match stage {
        1 => {
            if cancelled {
                return finish(tx);
            }
            if selections.len() != 1 || !matches!(selections[0], 1 | 2 | 3) {
                return Err(EffectError::Illegal(
                    "choose exactly one 敕令 option".into(),
                ));
            }
            match selections[0] {
                1 => {
                    // ① 选要明置的暗置槽位（仅列出仍暗置的槽）。
                    let unseen: Vec<u32> = tx
                        .state()
                        .players
                        .get(target)
                        .map(|p| {
                            p.unseen
                                .iter()
                                .enumerate()
                                .filter_map(|(i, &hidden)| hidden.then_some(i as u32))
                                .collect()
                        })
                        .unwrap_or_default();
                    if unseen.is_empty() {
                        return finish(tx);
                    }
                    let mut next = vec![
                        crate::effect::CHILING_CUSTOM_CHOICE,
                        2,
                        1, // mode: reveal slot
                        target.raw(),
                    ];
                    next.extend_from_slice(tail);
                    crate::effect::open_choice(
                        tx,
                        target,
                        1,
                        1,
                        false,
                        unseen,
                        engine_api::ChoiceKind::Custom,
                        crate::state::ContinuationKind::Custom,
                        next,
                    )?;
                    Err(EffectError::ChoiceRequested)
                }
                2 => {
                    // ② 失去1点体力（无来源）。
                    let effects = vec![EffectRequest::DealDamage {
                        source: PlayerId::NONE,
                        target,
                        amount: 1,
                        is_lose_hp: true,
                        nature: rule_api::damage_nature::NORMAL,
                    }];
                    if tail.is_empty() {
                        return run_effects(tx, effects).map(|_| ());
                    }
                    crate::command::play::run_effects_then_finish_independent_trick_target(
                        tx, effects, tail,
                    )
                }
                _ => {
                    // ③ 弃置一张装备牌（he 中的装备类牌）。
                    let tokens = crate::command::play::equip_typed_zone_cards(tx, target);
                    if tokens.is_empty() {
                        return finish(tx);
                    }
                    let mut next = vec![
                        crate::effect::CHILING_CUSTOM_CHOICE,
                        2,
                        3, // mode: discard equipment
                        target.raw(),
                    ];
                    next.extend_from_slice(tail);
                    crate::effect::open_choice(
                        tx,
                        target,
                        1,
                        1,
                        false,
                        tokens,
                        engine_api::ChoiceKind::SelectCards,
                        crate::state::ContinuationKind::Custom,
                        next,
                    )?;
                    Err(EffectError::ChoiceRequested)
                }
            }
        }
        2 => {
            if cancelled {
                return finish(tx);
            }
            let mode = payload[2];
            if selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "pick exactly one 敕令 card/slot".into(),
                ));
            }
            match mode {
                1 => {
                    // 明置选中的暗置槽位，然后摸一张牌。
                    let slot = selections[0];
                    let unseen = tx
                        .state()
                        .players
                        .get(target)
                        .map(|p| p.unseen.get(slot as usize).copied().unwrap_or(false))
                        .unwrap_or(false);
                    if !unseen {
                        return Err(EffectError::Illegal(
                            "敕令 reveal slot must be unseen".into(),
                        ));
                    }
                    crate::mode::reveal_general(
                        tx.state_mut(),
                        target,
                        u8::try_from(slot).unwrap_or(u8::MAX),
                    )
                    .map_err(EffectError::Illegal)?;
                    let effects = vec![EffectRequest::DrawCards {
                        player: target,
                        count: 1,
                    }];
                    if tail.is_empty() {
                        return run_effects(tx, effects).map(|_| ());
                    }
                    crate::command::play::run_effects_then_finish_independent_trick_target(
                        tx, effects, tail,
                    )
                }
                3 => {
                    // 弃置一张装备牌。
                    let tokens = crate::command::play::equip_typed_zone_cards(tx, target);
                    if !tokens.contains(&selections[0]) {
                        return Err(EffectError::Illegal(
                            "敕令 discard must be an equipment-typed zone card".into(),
                        ));
                    }
                    let effects = vec![EffectRequest::DiscardCards {
                        player: target,
                        cards: vec![CardId::new(selections[0])],
                    }];
                    if tail.is_empty() {
                        return run_effects(tx, effects).map(|_| ());
                    }
                    crate::command::play::run_effects_then_finish_independent_trick_target(
                        tx, effects, tail,
                    )
                }
                _ => Err(EffectError::Illegal("敕令 mode is out of range".into())),
            }
        }
        _ => Err(EffectError::Illegal("敕令 stage is out of range".into())),
    }
}
