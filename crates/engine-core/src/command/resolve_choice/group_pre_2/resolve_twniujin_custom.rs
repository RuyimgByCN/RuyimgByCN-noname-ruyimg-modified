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
