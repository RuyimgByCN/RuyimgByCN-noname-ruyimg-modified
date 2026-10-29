//! Command operations for `resolve_chegu` and related helpers.

use super::*;

pub(crate) fn resolve_chegu(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload
        .first()
        .ok_or_else(|| EffectError::EngineFault("missing Chegu choice stage".into()))?;
    let player = PlayerId::new(*payload.get(1).unwrap_or(&0));
    if cancelled || observer != player {
        return Err(EffectError::Illegal("invalid Chegu choice actor".into()));
    }
    match stage {
        0 => {
            let [branch] = selections else {
                return Err(EffectError::Illegal(
                    "Chegu requires exactly one branch".into(),
                ));
            };
            if !matches!(branch, 1 | 2) {
                return Err(EffectError::Illegal("invalid Chegu branch".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveChegu {
                    player,
                    card: CardId::new(*payload.get(2).unwrap_or(&0)),
                    targets: Vec::new(),
                    cards: Vec::new(),
                    branch: *branch as u8,
                    index: 0,
                    stage: 1,
                    skill: engine_api::SkillId::new(*payload.get(3).unwrap_or(&0)),
                }],
            )
            .map(|_| ())
        }
        1 => {
            if selections.is_empty() {
                return Err(EffectError::Illegal(
                    "Chegu requires at least one discard target".into(),
                ));
            }
            let mut distinct = std::collections::BTreeSet::new();
            let targets = selections
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect::<Vec<_>>();
            if targets.iter().any(|target| {
                !distinct.insert(*target)
                    || crate::query::discardable_target_cards(
                        tx.state(),
                        player,
                        *target,
                        rule_api::TargetCardScope::HandEquip,
                    )
                    .is_empty()
            }) {
                return Err(EffectError::Illegal(
                    "Chegu discard target became illegal".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveChegu {
                    player,
                    card: CardId::new(*payload.get(2).unwrap_or(&0)),
                    targets,
                    cards: Vec::new(),
                    branch: *payload.get(3).unwrap_or(&0) as u8,
                    index: 0,
                    stage: 2,
                    skill: engine_api::SkillId::new(*payload.get(4).unwrap_or(&0)),
                }],
            )
            .map(|_| ())
        }
        2 => {
            let branch = *payload.get(3).unwrap_or(&0) as u8;
            let index = *payload.get(4).unwrap_or(&0) as u8;
            let skill = engine_api::SkillId::new(*payload.get(5).unwrap_or(&0));
            let target_len = *payload.get(6).unwrap_or(&u32::MAX) as usize;
            let targets_end = 7usize.saturating_add(target_len);
            let targets = payload
                .get(7..targets_end)
                .ok_or_else(|| EffectError::EngineFault("truncated Chegu targets".into()))?
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect::<Vec<_>>();
            let discarded_len = *payload
                .get(targets_end)
                .ok_or_else(|| EffectError::EngineFault("missing Chegu discard count".into()))?
                as usize;
            let discarded = payload
                .get(targets_end + 1..targets_end + 1 + discarded_len)
                .ok_or_else(|| EffectError::EngineFault("truncated Chegu discards".into()))?
                .iter()
                .copied()
                .map(CardId::new)
                .collect::<Vec<_>>();
            let target = *targets
                .get(usize::from(index))
                .ok_or_else(|| EffectError::EngineFault("bad Chegu target index".into()))?;
            let legal = crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::HandEquip,
            );
            let mut distinct = std::collections::BTreeSet::new();
            let selected = selections
                .iter()
                .copied()
                .map(CardId::new)
                .collect::<Vec<_>>();
            if selected.is_empty()
                || selected
                    .iter()
                    .any(|card| !distinct.insert(*card) || !legal.contains(card))
            {
                return Err(EffectError::Illegal(
                    "Chegu discard cards became illegal".into(),
                ));
            }
            let mut cards = discarded;
            cards.extend(selected.iter().copied());
            run_effects(
                tx,
                vec![
                    EffectRequest::DiscardTargetCards {
                        actor: player,
                        target,
                        cards: selected,
                        scope: rule_api::TargetCardScope::HandEquip,
                    },
                    EffectRequest::ResolveChegu {
                        player,
                        card: CardId::new(*payload.get(2).unwrap_or(&0)),
                        targets,
                        cards,
                        branch,
                        index: index.saturating_add(1),
                        stage: 2,
                        skill,
                    },
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown Chegu choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_junkguixin(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload
        .first()
        .ok_or_else(|| EffectError::EngineFault("missing Junkguixin choice stage".into()))?;
    let player = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let [selected] = selections else {
        return Err(EffectError::Illegal(
            "Junkguixin requires exactly one choice".into(),
        ));
    };
    if cancelled || observer != player {
        return Err(EffectError::Illegal(
            "invalid Junkguixin choice actor".into(),
        ));
    }
    let skill = engine_api::SkillId::new(*payload.last().unwrap_or(&0));
    let effects = match stage {
        0 if matches!(selected, 1 | 2) => vec![EffectRequest::ResolveJunkguixin {
            player,
            target: PlayerId::NONE,
            stage: *selected as u8,
            group: 0,
            skill,
        }],
        1 => {
            let selected = engine_api::SkillDefId::new(*selected);
            if !crate::query::gainable_lord_skill_defs(tx.state(), player).contains(&selected) {
                return Err(EffectError::Illegal(
                    "selected Junkguixin lord skill became illegal".into(),
                ));
            }
            vec![EffectRequest::AddSkill {
                player,
                skill_def: selected,
            }]
        }
        2 => {
            let target = PlayerId::new(*selected);
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|target_state| target_state.alive && target != player)
            {
                return Err(EffectError::Illegal(
                    "selected Junkguixin target became illegal".into(),
                ));
            }
            vec![EffectRequest::ResolveJunkguixin {
                player,
                target,
                stage: 3,
                group: 0,
                skill,
            }]
        }
        3 if (1..=8).contains(selected) => vec![EffectRequest::ResolveJunkguixin {
            player,
            target: PlayerId::new(*payload.get(2).unwrap_or(&0)),
            stage: 4,
            group: *selected as u8,
            skill,
        }],
        _ => return Err(EffectError::Illegal("invalid Junkguixin choice".into())),
    };
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolve_guhuo(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if cancelled {
        return Err(EffectError::Illegal("Guhuo choice cannot cancel".into()));
    }
    match payload {
        [0, player, skill] => {
            let player = PlayerId::new(*player);
            let skill = engine_api::SkillId::new(*skill);
            let [source] = selections else {
                return Err(EffectError::Illegal(
                    "Guhuo requires exactly one hidden source".into(),
                ));
            };
            let source = CardId::new(*source);
            if observer != player || !tx.state().hand_cards(player).contains(&source) {
                return Err(EffectError::Illegal(
                    "Guhuo hidden source became illegal".into(),
                ));
            }
            let mut challengers = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| {
                    *candidate != player
                        && tx
                            .state()
                            .players
                            .get(*candidate)
                            .is_some_and(|candidate| candidate.hp != 0)
                })
                .collect::<Vec<_>>();
            if let Some(current) = challengers
                .iter()
                .position(|candidate| *candidate == tx.state().turn.current)
            {
                challengers.rotate_left(current);
            }
            if challengers.is_empty() {
                tx.state_mut()
                    .skills
                    .get_mut(skill)
                    .ok_or_else(|| EffectError::EngineFault("Guhuo skill disappeared".into()))?
                    .set_source_free_physical_card(source);
                return Ok(());
            }
            open_guhuo_challenge(tx, player, skill, source, 0, &challengers, &[])
        }
        [1, player, skill, source, index, challenger_len, rest @ ..] => {
            let player = PlayerId::new(*player);
            let skill = engine_api::SkillId::new(*skill);
            let source = CardId::new(*source);
            let challenger_len = *challenger_len as usize;
            let challengers = rest
                .get(..challenger_len)
                .ok_or_else(|| EffectError::EngineFault("truncated Guhuo challengers".into()))?
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect::<Vec<_>>();
            let betrayed_len = *rest
                .get(challenger_len)
                .ok_or_else(|| EffectError::EngineFault("missing Guhuo challenge count".into()))?
                as usize;
            let mut betrayed = rest
                .get(challenger_len + 1..challenger_len + 1 + betrayed_len)
                .ok_or_else(|| EffectError::EngineFault("truncated Guhuo challenges".into()))?
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect::<Vec<_>>();
            let index = *index as usize;
            let challenger = *challengers
                .get(index)
                .ok_or_else(|| EffectError::EngineFault("bad Guhuo challenger index".into()))?;
            let [decision @ (0 | 1)] = selections else {
                return Err(EffectError::Illegal(
                    "Guhuo challenge requires ally or challenge".into(),
                ));
            };
            if observer != challenger {
                return Err(EffectError::Illegal(
                    "Guhuo challenge answered by the wrong player".into(),
                ));
            }
            if *decision == 1 {
                betrayed.push(challenger);
            }
            if index + 1 < challengers.len() {
                return open_guhuo_challenge(
                    tx,
                    player,
                    skill,
                    source,
                    index + 1,
                    &challengers,
                    &betrayed,
                );
            }
            settle_guhuo(tx, player, skill, source, &betrayed)
        }
        _ => Err(EffectError::EngineFault(
            "bad Guhuo continuation payload".into(),
        )),
    }
}

pub(crate) fn resolve_face_down_view_as(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, skill, mode @ ..] = payload else {
        return Err(EffectError::EngineFault(
            "bad face-down view-as payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let skill = engine_api::SkillId::new(*skill);
    if mode.first() == Some(&5) {
        let kind = mode
            .get(1)
            .and_then(|tag| CardKind::from_tag(*tag as u8))
            .ok_or_else(|| EffectError::EngineFault("missing 腹笥 card declaration".into()))?;
        let [source] = selections else {
            return Err(EffectError::Illegal("腹笥须选择一张明置手牌".into()));
        };
        let source = CardId::new(*source);
        let legal = crate::command::skill::dddfusi_sources(tx.state(), player, skill, kind);
        if cancelled || observer != player || !legal.contains(&source) {
            return Err(EffectError::Illegal("腹笥明置牌已不可用".into()));
        }
        let provider = tx
            .state()
            .cards
            .get(source)
            .map(|card| card.owner)
            .ok_or_else(|| EffectError::Illegal("腹笥明置牌已消失".into()))?;
        if crate::command::skill::dddfusi_policy(tx.state(), skill, provider) == 1 {
            crate::command::skill::set_dddfusi_approval(
                tx.state_mut(),
                skill,
                source,
                provider,
                None,
            )?;
            tx.state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::EngineFault("腹笥借牌技能已消失".into()))?
                .set_source_free_physical_card(source);
            return Ok(());
        }
        crate::effect::open_choice(
            tx,
            provider,
            1,
            1,
            false,
            vec![0, 1, 2, 3],
            engine_api::ChoiceKind::Custom,
            ContinuationKind::ResolveFaceDownViewAs,
            vec![
                player.raw(),
                skill.raw(),
                6,
                kind.tag() as u32,
                source.raw(),
            ],
        )?;
        return Ok(());
    }
    if mode.first() == Some(&6) {
        let source = CardId::new(*mode.get(2).unwrap_or(&0));
        let provider = tx
            .state()
            .cards
            .get(source)
            .map(|card| card.owner)
            .ok_or_else(|| EffectError::Illegal("腹笥明置牌已消失".into()))?;
        let [decision] = selections else {
            return Err(EffectError::Illegal("腹笥牌主须确认".into()));
        };
        if cancelled || observer != provider || !matches!(*decision, 0..=3) {
            return Err(EffectError::Illegal("腹笥确认已失效".into()));
        }
        let policy = match *decision {
            2 => Some(1),
            3 => Some(2),
            _ => None,
        };
        if matches!(*decision, 0 | 2) {
            crate::command::skill::set_dddfusi_approval(
                tx.state_mut(),
                skill,
                source,
                provider,
                policy,
            )?;
            tx.state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::EngineFault("腹笥借牌技能已消失".into()))?
                .set_source_free_physical_card(source);
        } else {
            if policy.is_some() {
                crate::command::skill::set_dddfusi_approval(
                    tx.state_mut(),
                    skill,
                    CardId::NONE,
                    provider,
                    policy,
                )?;
            }
            tx.state_mut().resolution.resume_kind = ContinuationKind::None;
            tx.state_mut().resolution.resume_payload.clear();
        }
        return Ok(());
    }
    if mode == [4] {
        let legal = crate::command::skill::dddshilie_sources(tx.state(), player);
        let selected: Vec<_> = selections.iter().copied().map(CardId::new).collect();
        let unique: std::collections::BTreeSet<_> = selected.iter().copied().collect();
        let threshold =
            crate::command::skill::dddshilie_threshold(tx.state(), player).ok_or_else(|| {
                EffectError::Illegal("示烈 is unavailable outside a live turn".into())
            })?;
        let total = selected.iter().try_fold(0u16, |total, card| {
            tx.state().cards.get(*card).map(|card| {
                total.saturating_add(u16::from(crate::query::effective_card_number(
                    tx.state(),
                    player,
                    card,
                )))
            })
        });
        if cancelled
            || observer != player
            || selected.is_empty()
            || unique.len() != selected.len()
            || selected.iter().any(|card| !legal.contains(card))
            || total.is_none_or(|total| total < threshold)
            || !tx.state().skills.get(skill).is_some_and(|instance| {
                instance.owner == player && tx.state().skill_effectively_enabled(instance)
            })
        {
            return Err(EffectError::Illegal("invalid 示烈 shown cards".into()));
        }
        run_effects(
            tx,
            vec![EffectRequest::RevealCards {
                cards: selected.clone(),
                to: Vec::new(),
            }],
        )?;
        for card in &selected {
            tx.state_mut()
                .cards
                .get_mut(*card)
                .expect("validated 示烈 card")
                .face_up = true;
        }
        if total == Some(threshold) {
            run_effects(
                tx,
                vec![EffectRequest::DrawCards {
                    player,
                    count: selected.len().min(u32::MAX as usize) as u32,
                }],
            )?;
        }
        return Ok(());
    }
    let [source] = selections else {
        return Err(EffectError::Illegal(
            "face-down view-as requires one source card".into(),
        ));
    };
    let source = CardId::new(*source);
    let hand_equip_mark = mode == [1];
    let potdaozhuan = mode.first() == Some(&2);
    let dczhaowen = mode == [3];
    let legal = if potdaozhuan {
        crate::command::skill::potdaozhuan_cost_cards(tx.state(), player)
    } else if hand_equip_mark {
        crate::command::skill::hand_equip_view_as_sources(tx.state(), player)
    } else if dczhaowen {
        tx.state().skills.get(skill).map_or_else(Vec::new, |skill| {
            crate::command::skill::dczhaowen_view_as_sources(tx.state(), skill)
        })
    } else {
        crate::command::skill::face_down_view_as_sources(tx.state(), player)
    };
    if cancelled || observer != player || !legal.contains(&source) {
        return Err(EffectError::Illegal(
            "face-down view-as source became illegal".into(),
        ));
    }
    if potdaozhuan {
        let kind = mode
            .get(1)
            .and_then(|tag| CardKind::from_tag(*tag as u8))
            .ok_or_else(|| EffectError::EngineFault("missing 道转 declaration".into()))?;
        let current = tx.state().turn.current;
        let source_owner = tx
            .state()
            .cards
            .get(source)
            .map(|card| card.owner)
            .ok_or_else(|| EffectError::Illegal("道转 cost card disappeared".into()))?;
        let name_tag = if kind.is_sha() {
            CardKind::Sha.tag()
        } else {
            kind.tag()
        };
        let name_bit = (name_tag < 32)
            .then_some(1u32 << name_tag)
            .ok_or_else(|| EffectError::Illegal("unsupported 道转 basic name".into()))?;
        let round = tx.state().turn.round;
        tx.state_mut()
            .skills
            .get_mut(skill)
            .ok_or_else(|| EffectError::EngineFault("道转 skill disappeared".into()))?
            .record_potdaozhuan(
                round,
                current,
                name_bit,
                source_owner == current && current != player,
            );
        let discard = tx
            .state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
        let from = tx
            .state()
            .cards
            .get(source)
            .map(|card| card.zone)
            .ok_or_else(|| EffectError::Illegal("道转 cost card disappeared".into()))?;
        run_effects(
            tx,
            vec![EffectRequest::MoveCardsWithReason {
                cards: vec![source],
                to: discard,
                from: Some(from),
                reason: CardMoveReason::Discard,
            }],
        )?;
        return Ok(());
    }
    if !hand_equip_mark {
        let kind = tx
            .state()
            .resolution
            .resume_payload
            .get(2)
            .and_then(|kind| CardKind::from_tag(*kind as u8))
            .ok_or_else(|| EffectError::EngineFault("missing face-down declaration".into()))?;
        crate::command::skill::record_face_down_view_as_kind(tx.state_mut(), skill, kind)?;
    }
    tx.state_mut()
        .skills
        .get_mut(skill)
        .ok_or_else(|| EffectError::EngineFault("face-down view-as skill disappeared".into()))?
        .set_source_free_physical_card(source);
    Ok(())
}

pub(crate) fn resolve_swap_lower_hand_view_as(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad swap-lower-hand view-as payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let skill = SkillId::new(*skill);
    let [target] = selections else {
        return Err(EffectError::Illegal(
            "swap-lower-hand view-as requires one target".into(),
        ));
    };
    let target = PlayerId::new(*target);
    if cancelled
        || observer != player
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || !crate::command::skill::swap_lower_hand_targets(tx.state(), player).contains(&target)
    {
        return Err(EffectError::Illegal(
            "swap-lower-hand view-as target became illegal".into(),
        ));
    }
    let player_cards = tx.state().hand_cards(player);
    let target_cards = tx.state().hand_cards(target);
    let player_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("source hand missing".into()))?;
    let target_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::EngineFault("target hand missing".into()))?;
    crate::effect::move_cards(tx, player_cards, target_hand, Some(player_hand))?;
    crate::effect::move_cards(tx, target_cards, player_hand, Some(target_hand))?;
    Ok(())
}

pub(crate) fn resolve_yao_chenwei(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [stage, player, third, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad Yao Chenwei continuation payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let skill = engine_api::SkillId::new(*skill);
    let [selected] = selections else {
        return Err(EffectError::Illegal(
            "Yao Chenwei requires exactly one selection".into(),
        ));
    };
    if cancelled {
        return Err(EffectError::Illegal(
            "Yao Chenwei choice cannot cancel".into(),
        ));
    }
    let effect = match stage {
        0 if observer == player && matches!(third, 0 | 1) => EffectRequest::ResolveYaoChenwei {
            player,
            target: PlayerId::new(*selected),
            card: CardId::NONE,
            mode: *third as u8,
            stage: 1,
            skill,
        },
        1 => {
            let target = PlayerId::new(*third);
            if observer != target {
                return Err(EffectError::Illegal(
                    "Yao Chenwei card must be chosen by its target".into(),
                ));
            }
            EffectRequest::ResolveYaoChenwei {
                player,
                target,
                card: CardId::new(*selected),
                mode: 0,
                stage: 2,
                skill,
            }
        }
        2 if observer == player => EffectRequest::ResolveYaoChenwei {
            player,
            target: PlayerId::new(*third),
            card: CardId::new(*selected),
            mode: 1,
            stage: 2,
            skill,
        },
        _ => {
            return Err(EffectError::Illegal(
                "invalid Yao Chenwei choice actor or stage".into(),
            ));
        }
    };
    run_effects(tx, vec![effect]).map(|_| ())
}
