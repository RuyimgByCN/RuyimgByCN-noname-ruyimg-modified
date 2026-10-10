//! Choice-continuation handlers (pre 3).
//!
//! Split out of the former single-file `resolve_choice.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the dispatch in the parent module can reach it.

use super::*;

pub(crate) fn resolve_new_yajiao(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [selected] = selections else {
        return Err(EffectError::Illegal(
            "New Yajiao requires exactly one selection".into(),
        ));
    };
    if cancelled {
        return Err(EffectError::Illegal(
            "New Yajiao choice cannot cancel".into(),
        ));
    }
    match payload {
        [0, player, source_card, revealed, source_category, skill]
            if observer == PlayerId::new(*player) =>
        {
            run_effects(
                tx,
                vec![EffectRequest::ResolveNewYajiao {
                    player: PlayerId::new(*player),
                    source_card: CardId::new(*source_card),
                    revealed: CardId::new(*revealed),
                    target: PlayerId::new(*selected),
                    source_category: u8::try_from(*source_category).unwrap_or(0),
                    stage: 2,
                    skill: engine_api::SkillId::new(*skill),
                }],
            )
            .map(|_| ())
        }
        [1, player] if observer == PlayerId::new(*player) => {
            let player = PlayerId::new(*player);
            let card = CardId::new(*selected);
            if !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&card) {
                return Err(EffectError::Illegal(
                    "New Yajiao discard card is no longer legal".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardTargetCards {
                    actor: player,
                    target: player,
                    cards: vec![card],
                    scope: rule_api::TargetCardScope::HandEquip,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal(
            "invalid New Yajiao choice actor or stage".into(),
        )),
    }
}

pub(crate) fn open_guhuo_challenge(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    source: CardId,
    index: usize,
    challengers: &[PlayerId],
    betrayed: &[PlayerId],
) -> Result<(), EffectError> {
    let mut payload = vec![
        1,
        player.raw(),
        skill.raw(),
        source.raw(),
        index as u32,
        challengers.len() as u32,
    ];
    payload.extend(challengers.iter().map(|challenger| challenger.raw()));
    payload.push(betrayed.len() as u32);
    payload.extend(betrayed.iter().map(|challenger| challenger.raw()));
    crate::effect::open_choice(
        tx,
        challengers[index],
        1,
        1,
        false,
        vec![0, 1],
        engine_api::ChoiceKind::Custom,
        ContinuationKind::ResolveGuhuo,
        payload,
    )
    .map(|_| ())
}

pub(crate) fn settle_guhuo(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    source: CardId,
    challengers: &[PlayerId],
) -> Result<(), EffectError> {
    if challengers.is_empty() {
        tx.state_mut()
            .skills
            .get_mut(skill)
            .ok_or_else(|| EffectError::EngineFault("Guhuo skill disappeared".into()))?
            .set_source_free_physical_card(source);
        return Ok(());
    }
    let declared = tx
        .state()
        .resolution
        .resume_payload
        .get(2)
        .and_then(|kind| CardKind::from_tag(*kind as u8))
        .ok_or_else(|| EffectError::EngineFault("missing Guhuo declaration".into()))?;
    let source_state = tx
        .state()
        .cards
        .get(source)
        .ok_or_else(|| EffectError::Illegal("Guhuo source disappeared".into()))?;
    let actual = crate::query::effective_card_kind(tx.state(), player, source_state);
    let truthful = actual == declared;
    let heart = crate::query::effective_card_suit(tx.state(), player, source_state)
        == engine_api::Suit::Heart;
    let succeeds = truthful && heart;
    let mut effects = Vec::new();
    if truthful {
        effects.extend(
            challengers
                .iter()
                .map(|challenger| EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target: *challenger,
                    amount: 1,
                    is_lose_hp: true,
                    nature: 0,
                }),
        );
    } else {
        effects.extend(
            challengers
                .iter()
                .map(|challenger| EffectRequest::DrawCards {
                    player: *challenger,
                    count: 1,
                }),
        );
    }
    if succeeds {
        tx.state_mut()
            .skills
            .get_mut(skill)
            .ok_or_else(|| EffectError::EngineFault("Guhuo skill disappeared".into()))?
            .set_source_free_physical_card(source);
    } else {
        let resolution = &mut tx.state_mut().resolution;
        resolution.resume_kind = ContinuationKind::None;
        resolution.resume_payload.clear();
        effects.push(EffectRequest::DiscardCards {
            player,
            cards: vec![source],
        });
    }
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolve_drlt_jueyan(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, sha_raw, weapon_raw, mount_raw, armor_raw, treasure_raw, skill_raw] = payload
    else {
        return Err(EffectError::EngineFault(
            "bad Drlt Jueyan slot payload".into(),
        ));
    };
    let [slot] = selections else {
        return Err(EffectError::Illegal("Drlt Jueyan requires one slot".into()));
    };
    let player = PlayerId::new(*player_raw);
    if cancelled || observer != player || !matches!(slot, 1 | 2 | 3 | 5) {
        return Err(EffectError::Illegal(
            "invalid Drlt Jueyan slot choice".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveDrltJueyan {
            player,
            stage: 1,
            slot: *slot as u8,
            first: false,
            sha_skill_def: engine_api::SkillDefId::new(*sha_raw),
            weapon_skill_def: engine_api::SkillDefId::new(*weapon_raw),
            mount_skill_def: engine_api::SkillDefId::new(*mount_raw),
            armor_skill_def: engine_api::SkillDefId::new(*armor_raw),
            treasure_skill_def: engine_api::SkillDefId::new(*treasure_raw),
            skill: engine_api::SkillId::new(*skill_raw),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_rejizhi(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, card_raw, skill_raw] = payload else {
        return Err(EffectError::EngineFault("bad Rejizhi payload".into()));
    };
    let player = PlayerId::new(*player_raw);
    if cancelled || observer != player || !matches!(selections, [0] | [1]) {
        return Err(EffectError::Illegal(
            "invalid Rejizhi discard choice".into(),
        ));
    }
    if selections[0] == 0 {
        return Ok(());
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveRejizhi {
            player,
            stage: 2,
            card: CardId::new(*card_raw),
            skill: engine_api::SkillId::new(*skill_raw),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_jiexun(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    match payload.first().copied() {
        Some(0) => {
            let [_, player_raw, draw_count, discard_count, helper_raw, skill_raw] = payload else {
                return Err(EffectError::EngineFault("bad Jiexun target payload".into()));
            };
            let [target] = selections else {
                return Err(EffectError::Illegal("Jiexun requires one target".into()));
            };
            let player = PlayerId::new(*player_raw);
            if cancelled || observer != player {
                return Err(EffectError::Illegal("invalid Jiexun target".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveJiexun {
                    player,
                    stage: 1,
                    target: PlayerId::new(*target),
                    draw_count: *draw_count,
                    discard_count: *discard_count,
                    funan_helper_skill_def: engine_api::SkillDefId::new(*helper_raw),
                    skill: engine_api::SkillId::new(*skill_raw),
                }],
            )
            .map(|_| ())
        }
        Some(1) => {
            if payload.len() < 7 {
                return Err(EffectError::EngineFault(
                    "bad Jiexun discard payload".into(),
                ));
            }
            let target = PlayerId::new(payload[2]);
            let option_count = payload[5] as usize;
            let required = payload[6] as usize;
            if cancelled
                || observer != target
                || payload.len() != 7 + option_count
                || selections.len() != required
            {
                return Err(EffectError::Illegal("invalid Jiexun discard count".into()));
            }
            let original: std::collections::BTreeSet<_> = payload[7..].iter().copied().collect();
            let current: std::collections::BTreeSet<_> =
                crate::query::discardable_hand_equip_cards(tx.state(), target)
                    .into_iter()
                    .map(CardId::raw)
                    .collect();
            let mut unique = std::collections::BTreeSet::new();
            if !selections.iter().all(|card| {
                unique.insert(*card) && original.contains(card) && current.contains(card)
            }) {
                return Err(EffectError::Illegal(
                    "Jiexun discard cards became stale".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: target,
                    cards: selections.iter().copied().map(CardId::new).collect(),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown Jiexun choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_wengua(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    match payload.first().copied() {
        Some(0) => {
            let [_, player_raw, card_raw, main_raw, marker_raw, skill_raw] = payload else {
                return Err(EffectError::EngineFault("bad Wengua target payload".into()));
            };
            let [target] = selections else {
                return Err(EffectError::Illegal("Wengua requires one target".into()));
            };
            let player = PlayerId::new(*player_raw);
            if cancelled || observer != player {
                return Err(EffectError::Illegal("invalid Wengua target".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveWengua {
                    player,
                    stage: 1,
                    target: PlayerId::new(*target),
                    card: CardId::new(*card_raw),
                    branch: 0,
                    main_skill_def: engine_api::SkillDefId::new(*main_raw),
                    marker_skill_def: engine_api::SkillDefId::new(*marker_raw),
                    skill: engine_api::SkillId::new(*skill_raw),
                }],
            )
            .map(|_| ())
        }
        Some(1) => {
            let [_, player_raw, target_raw, card_raw, main_raw, marker_raw, skill_raw] = payload
            else {
                return Err(EffectError::EngineFault("bad Wengua branch payload".into()));
            };
            let target = PlayerId::new(*target_raw);
            if observer != target {
                return Err(EffectError::Illegal("wrong Wengua branch chooser".into()));
            }
            if cancelled {
                return if selections.is_empty() {
                    Ok(())
                } else {
                    Err(EffectError::Illegal(
                        "cancelled Wengua branch must be empty".into(),
                    ))
                };
            }
            if !matches!(selections, [0] | [1]) {
                return Err(EffectError::Illegal(
                    "invalid Wengua top/bottom branch".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveWengua {
                    player: PlayerId::new(*player_raw),
                    stage: 3,
                    target,
                    card: CardId::new(*card_raw),
                    branch: selections[0] as u8,
                    main_skill_def: engine_api::SkillDefId::new(*main_raw),
                    marker_skill_def: engine_api::SkillDefId::new(*marker_raw),
                    skill: engine_api::SkillId::new(*skill_raw),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown Wengua choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_discard_tiaojiyanmei_card(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [actor_raw, left_raw, right_raw, higher_raw] = payload else {
        return Err(EffectError::Illegal("bad 调剂盐梅 discard payload".into()));
    };
    let actor = PlayerId::new(*actor_raw);
    let left = PlayerId::new(*left_raw);
    let right = PlayerId::new(*right_raw);
    let higher = PlayerId::new(*higher_raw);
    let [card_raw] = selections else {
        return Err(EffectError::Illegal(
            "调剂盐梅 requires one discarded card".into(),
        ));
    };
    let card = CardId::new(*card_raw);
    if cancelled
        || observer != higher
        || !crate::query::discardable_hand_equip_cards(tx.state(), higher).contains(&card)
    {
        return Err(EffectError::Illegal(
            "调剂盐梅 discard context changed".into(),
        ));
    }
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::FinalizeTiaojiyanmei {
            actor,
            left,
            right,
            card,
        });
    run_effects(
        tx,
        vec![EffectRequest::DiscardCards {
            player: higher,
            cards: vec![card],
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_tiaojiyanmei_gain_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [actor_raw, card_raw] = payload else {
        return Err(EffectError::Illegal("bad 调剂盐梅 gain payload".into()));
    };
    let actor = PlayerId::new(*actor_raw);
    if observer != actor {
        return Err(EffectError::Illegal("调剂盐梅 gain chooser changed".into()));
    }
    if cancelled {
        return Ok(());
    }
    let [target_raw] = selections else {
        return Err(EffectError::Illegal(
            "调剂盐梅 requires one gain target".into(),
        ));
    };
    let target = PlayerId::new(*target_raw);
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|player| player.alive)
    {
        return Err(EffectError::Illegal("调剂盐梅 gain target changed".into()));
    }
    let card = CardId::new(*card_raw);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
    if !tx
        .state()
        .cards
        .get(card)
        .is_some_and(|card| card.zone == discard)
    {
        return Err(EffectError::Illegal("调剂盐梅 discarded card moved".into()));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::EngineFault("gain target hand missing".into()))?;
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: hand,
            from: Some(discard),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_changandajian_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, remaining, skill_raw] = payload else {
        return Err(EffectError::Illegal(
            "bad Changandajian target payload".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    let skill = engine_api::SkillId::new(*skill_raw);
    let [target_raw] = selections else {
        return Err(EffectError::Illegal(
            "Changandajian requires one target".into(),
        ));
    };
    let target = PlayerId::new(*target_raw);
    if observer != player
        || cancelled
        || *remaining == 0
        || *remaining > 32
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|target| target.alive)
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "Changandajian target context changed".into(),
        ));
    }
    let options: Vec<_> = [ZoneKind::Equip, ZoneKind::Judge]
        .into_iter()
        .filter_map(|kind| tx.state().zones.find(kind, target))
        .flat_map(|zone| {
            tx.state()
                .zones
                .get(zone)
                .map(|zone| zone.cards.clone())
                .unwrap_or_default()
        })
        .collect();
    if options.is_empty() {
        return Err(EffectError::Illegal(
            "Changandajian target no longer has an equipment or judgment card".into(),
        ));
    }
    let mut next = vec![
        player.raw(),
        target.raw(),
        *remaining,
        skill.raw(),
        options.len() as u32,
    ];
    next.extend(options.iter().map(|card| card.raw()));
    crate::effect::open_choice(
        tx,
        player,
        1,
        1,
        false,
        options.into_iter().map(CardId::raw).collect(),
        engine_api::ChoiceKind::SelectCards,
        ContinuationKind::ChooseChangandajianCard,
        next,
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_changandajian_card(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 {
        return Err(EffectError::Illegal(
            "bad Changandajian card payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let target = PlayerId::new(payload[1]);
    let remaining = payload[2];
    let skill = engine_api::SkillId::new(payload[3]);
    let option_count = payload[4] as usize;
    let [selected] = selections else {
        return Err(EffectError::Illegal(
            "Changandajian requires one card".into(),
        ));
    };
    if observer != player
        || cancelled
        || remaining == 0
        || remaining > 32
        || payload.len() != 5 + option_count
        || !payload[5..].contains(selected)
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "Changandajian card context changed".into(),
        ));
    }
    let card = CardId::new(*selected);
    let card_state = tx
        .state()
        .cards
        .get(card)
        .filter(|card| card.owner == target)
        .cloned()
        .ok_or_else(|| EffectError::Illegal("Changandajian card moved".into()))?;
    if !tx.state().zones.get(card_state.zone).is_some_and(|zone| {
        zone.owner == target && matches!(zone.kind, ZoneKind::Equip | ZoneKind::Judge)
    }) {
        return Err(EffectError::Illegal(
            "Changandajian card is no longer in equipment or judgment".into(),
        ));
    }
    if remaining > 1 {
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::ResolveDiliYuheng {
                player,
                action: 129,
                card: CardId::NONE,
                event_cards: vec![CardId::NONE; (remaining - 1) as usize],
                skill,
            });
    }
    let destination = if matches!(card_state.rank, 1 | 11 | 12 | 13) {
        tx.state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::EngineFault("Changandajian owner hand missing".into()))?
    } else {
        tx.state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?
    };
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: destination,
            from: Some(card_state.zone),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_end_extra_draw_or_play(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, skill_raw] = payload else {
        return Err(EffectError::Illegal(
            "bad end extra-phase choice payload".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    let skill = engine_api::SkillId::new(*skill_raw);
    let [phases] = selections else {
        return Err(EffectError::Illegal(
            "end extra-phase choice requires one option".into(),
        ));
    };
    if observer != player
        || cancelled
        || !matches!(*phases, 1 | 2)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::End
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "end extra-phase choice context changed".into(),
        ));
    }
    crate::effect::queue_end_extra_phases(tx, *phases as u8);
    Ok(())
}

pub(crate) fn resolve_choose_zone_priority_gain_random_card_from_each_other_then_turn_over(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, skill_raw] = payload else {
        return Err(EffectError::Illegal(
            "bad zone-priority gain continuation payload".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    let skill = engine_api::SkillId::new(*skill_raw);
    let [priority] = selections else {
        return Err(EffectError::Illegal(
            "zone-priority gain requires exactly one control".into(),
        ));
    };
    if observer != player
        || cancelled
        || *priority > 2
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "zone-priority gain context changed".into(),
        ));
    }
    let remaining_targets = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| *target != player)
        .collect();
    run_effects(
        tx,
        vec![
            EffectRequest::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver {
                player,
                priority: *priority as u8,
                remaining_targets,
                skill,
            },
        ],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_gain_card_from_each_other_then_turn_over(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 {
        return Err(EffectError::Illegal(
            "bad gain-from-each continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let target = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    let option_count = payload[3] as usize;
    let [card_raw] = selections else {
        return Err(EffectError::Illegal(
            "gain-from-each requires exactly one card".into(),
        ));
    };
    let card = CardId::new(*card_raw);
    let card_state = tx.state().cards.get(card);
    let gainable = crate::query::gainable_target_cards(
        tx.state(),
        player,
        target,
        rule_api::TargetCardScope::HandEquipJudge,
    );
    if cancelled
        || observer != player
        || payload.len() != 4 + option_count
        || !payload[4..].contains(card_raw)
        || !gainable.contains(&card)
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "gain-from-each card is no longer legal".into(),
        ));
    }
    let from = card_state.unwrap().zone;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("Guixin owner hand zone missing".into()))?;
    run_effects(
        tx,
        vec![
            EffectRequest::MoveCards {
                cards: vec![card],
                to: hand,
                from: Some(from),
            },
            EffectRequest::RevealCards {
                cards: vec![card],
                to: Vec::new(),
            },
        ],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_hidden_skill_pile_hand_cards(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, pile_def, _action, _card, _skill] = payload else {
        return Err(EffectError::Illegal(
            "bad hidden skill-pile hand payload".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    if observer != player {
        return Err(EffectError::Illegal(
            "hidden skill-pile hand observer mismatch".into(),
        ));
    }
    if cancelled {
        return if selections.is_empty() {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "cancelled hidden skill-pile hand choice must be empty".into(),
            ))
        };
    }
    let current_hand: std::collections::BTreeSet<_> = tx
        .state()
        .hand_cards(player)
        .into_iter()
        .map(CardId::raw)
        .collect();
    let selected: std::collections::BTreeSet<_> = selections.iter().copied().collect();
    let hidden = tx.state().hidden_skill_pile_cards(player, *pile_def);
    if selections.is_empty()
        || selections.len() > hidden.len()
        || selected.len() != selections.len()
        || !selected.is_subset(&current_hand)
    {
        return Err(EffectError::Illegal(
            "hidden skill-pile hand cards are no longer legal".into(),
        ));
    }
    let mut next = payload.to_vec();
    next.push(selections.len() as u32);
    next.extend_from_slice(selections);
    crate::effect::open_choice(
        tx,
        player,
        selections.len() as u8,
        selections.len() as u8,
        false,
        hidden.into_iter().map(CardId::raw).collect(),
        engine_api::ChoiceKind::SelectCards,
        ContinuationKind::ChooseHiddenSkillPileCards,
        next,
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_hidden_skill_pile_cards(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 {
        return Err(EffectError::Illegal(
            "bad hidden skill-pile card payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let action = payload[2] as u8;
    let (count, hand_start) = if action == 2 {
        (payload.get(5).copied().unwrap_or(0) as usize, 6)
    } else {
        (1, payload.len())
    };
    if observer != player
        || cancelled
        || selections.len() != count
        || (action == 2 && payload.len() != hand_start + count)
    {
        return Err(EffectError::Illegal(
            "hidden skill-pile card count mismatch".into(),
        ));
    }
    let current: std::collections::BTreeSet<_> = tx
        .state()
        .hidden_skill_pile_cards(player, payload[1])
        .into_iter()
        .map(CardId::raw)
        .collect();
    let selected: std::collections::BTreeSet<_> = selections.iter().copied().collect();
    if selected.len() != selections.len() || !selected.is_subset(&current) {
        return Err(EffectError::Illegal(
            "hidden skill-pile cards are no longer legal".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveHiddenSkillPile {
            player,
            pile_skill_def: engine_api::SkillDefId::new(payload[1]),
            action,
            card: CardId::new(payload[3]),
            stage: 1,
            hand_cards: if action == 2 {
                payload[hand_start..]
                    .iter()
                    .copied()
                    .map(CardId::new)
                    .collect()
            } else {
                Vec::new()
            },
            pile_cards: selections.iter().copied().map(CardId::new).collect(),
            event_cards: Vec::new(),
            skill: engine_api::SkillId::new(payload[4]),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_mark_parity_control_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, mark_raw, mass_threshold, damage_amount, skill_raw] = payload else {
        return Err(EffectError::Illegal(
            "bad mark parity target continuation payload".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    if observer != player {
        return Err(EffectError::Illegal(
            "mark parity target observer mismatch".into(),
        ));
    }
    if cancelled {
        return if selections.is_empty() {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "cancelled mark parity target must be empty".into(),
            ))
        };
    }
    let [target_raw] = selections else {
        return Err(EffectError::Illegal(
            "mark parity control requires one target".into(),
        ));
    };
    let target = PlayerId::new(*target_raw);
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "mark parity control target is no longer alive".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveMarkParityControl {
            player,
            mark: engine_api::MarkDefId::new(*mark_raw),
            mass_threshold: *mass_threshold,
            damage_amount: *damage_amount,
            stage: 1,
            index: 0,
            target,
            targets: Vec::new(),
            skill: engine_api::SkillId::new(*skill_raw),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_discard_mark_parity_control_target_card(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, target_raw, mark_raw, mass_threshold, damage_amount, skill_raw] = payload
    else {
        return Err(EffectError::Illegal(
            "bad mark parity discard continuation payload".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    let target = PlayerId::new(*target_raw);
    let [card_raw] = selections else {
        return Err(EffectError::Illegal(
            "mark parity discard requires one card".into(),
        ));
    };
    let card = CardId::new(*card_raw);
    if cancelled
        || observer != player
        || !crate::query::discardable_target_cards(
            tx.state(),
            player,
            target,
            rule_api::TargetCardScope::HandEquipJudge,
        )
        .contains(&card)
    {
        return Err(EffectError::Illegal(
            "mark parity discard card is no longer legal".into(),
        ));
    }
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveMarkParityControl {
            player,
            mark: engine_api::MarkDefId::new(*mark_raw),
            mass_threshold: *mass_threshold,
            damage_amount: *damage_amount,
            stage: 3,
            index: 0,
            target,
            targets: Vec::new(),
            skill: engine_api::SkillId::new(*skill_raw),
        });
    run_effects(
        tx,
        vec![EffectRequest::DiscardTargetCards {
            actor: player,
            target,
            cards: vec![card],
            scope: rule_api::TargetCardScope::HandEquipJudge,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_confirm_mark_parity_mass_damage(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, mark_raw, mass_threshold, damage_amount, skill_raw] = payload else {
        return Err(EffectError::Illegal(
            "bad mark parity mass continuation payload".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    let [choice] = selections else {
        return Err(EffectError::Illegal(
            "mark parity mass confirmation requires one option".into(),
        ));
    };
    if cancelled || observer != player || *choice > 1 {
        return Err(EffectError::Illegal(
            "invalid mark parity mass confirmation".into(),
        ));
    }
    if *choice == 0 {
        return Ok(());
    }
    let mark = engine_api::MarkDefId::new(*mark_raw);
    let marks = tx
        .state()
        .players
        .get(player)
        .and_then(|owner| {
            owner
                .marks
                .iter()
                .find(|(definition, _)| *definition == mark.raw())
                .map(|(_, count)| (*count).max(0) as u32)
        })
        .unwrap_or(0);
    if marks <= *mass_threshold {
        return Err(EffectError::Illegal(
            "mark parity mass threshold is no longer met".into(),
        ));
    }
    let targets = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| *target != player)
        .collect();
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveMarkParityControl {
            player,
            mark,
            mass_threshold: *mass_threshold,
            damage_amount: *damage_amount,
            stage: 4,
            index: 0,
            target: PlayerId::NONE,
            targets,
            skill: engine_api::SkillId::new(*skill_raw),
        });
    run_effects(
        tx,
        vec![EffectRequest::RemoveMark {
            player,
            mark,
            amount: 0,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_selected_target_damage(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 {
        return Err(EffectError::Illegal(
            "bad selected-target damage continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let damage_amount = payload[1];
    let nature = payload[2] as u8;
    let target_count = payload[4] as usize;
    let [target_raw] = selections else {
        return Err(EffectError::Illegal(
            "selected-target damage requires one target".into(),
        ));
    };
    let target = PlayerId::new(*target_raw);
    if cancelled
        || observer != player
        || payload.len() != 5 + target_count
        || !payload[5..].contains(target_raw)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "selected-target damage choice is no longer legal".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source: player,
            target,
            amount: damage_amount,
            is_lose_hp: false,
            nature,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_distinct_suit_cards(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 3 {
        return Err(EffectError::Illegal(
            "bad distinct-suit continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let count = payload[2] as usize;
    if cancelled || observer != player || payload.len() != 3 + count {
        return Err(EffectError::Illegal(
            "invalid distinct-suit selection context".into(),
        ));
    }
    let original: std::collections::BTreeSet<_> = payload[3..].iter().copied().collect();
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
    let all_suits: std::collections::BTreeSet<_> = payload[3..]
        .iter()
        .filter_map(|card| tx.state().cards.get(CardId::new(*card)))
        .filter(|card| card.zone == processing)
        .map(|card| {
            crate::command::judge::suit_tag(crate::query::effective_card_suit(tx.state(), player, card))
        })
        .collect();
    let mut selected_cards = std::collections::BTreeSet::new();
    let selected_suits: std::collections::BTreeSet<_> = selections
        .iter()
        .filter_map(|card| {
            let card = tx.state().cards.get(CardId::new(*card))?;
            (original.contains(&card.id.raw())
                && card.zone == processing
                && selected_cards.insert(card.id.raw()))
            .then(|| {
                crate::command::judge::suit_tag(crate::query::effective_card_suit(tx.state(), player, card))
            })
        })
        .collect();
    if selected_cards.len() != selections.len()
        || selections.len() != all_suits.len()
        || selected_suits != all_suits
    {
        return Err(EffectError::Illegal(
            "selection must contain exactly one revealed card per distinct suit".into(),
        ));
    }
    let selected: Vec<_> = selections.iter().copied().map(CardId::new).collect();
    let rest: Vec<_> = payload[3..]
        .iter()
        .copied()
        .filter(|card| !selected_cards.contains(card))
        .map(CardId::new)
        .collect();
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("player hand missing".into()))?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
    let mut effects = vec![EffectRequest::MoveCards {
        cards: selected,
        to: hand,
        from: Some(processing),
    }];
    if !rest.is_empty() {
        effects.push(EffectRequest::MoveCards {
            cards: rest,
            to: discard,
            from: Some(processing),
        });
    }
    run_effects(tx, effects).map(|_| ())
}

/// Resolve the 「试探」 (scskuiji) cross-hand distinct-suit discard. The chooser may pick up to
/// `count` cards from its own and `target`'s hands combined; the selection must have distinct
/// effective suits, and all selected cards are then discarded (split by owner).
pub(crate) fn resolve_choose_scskuiji_discard(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 {
        return Err(EffectError::Illegal(
            "bad scskuiji continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let target = PlayerId::new(payload[1]);
    let count = payload[2] as usize;
    let offered_len = payload[4] as usize;
    if observer != player || payload.len() != 5 + offered_len || selections.len() > count {
        return Err(EffectError::Illegal(
            "invalid scskuiji selection context".into(),
        ));
    }
    let offered: std::collections::BTreeSet<_> = payload[5..].iter().copied().collect();
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    // Each selected card must be one of the offered cards, still in either participant's hand, and
    // no two may share an effective suit.
    let mut seen_cards = std::collections::BTreeSet::new();
    let mut seen_suits = std::collections::BTreeSet::new();
    let hand_zones = [
        tx.state().zones.find(ZoneKind::Hand, player),
        tx.state().zones.find(ZoneKind::Hand, target),
    ];
    for raw in selections {
        let card = tx
            .state()
            .cards
            .get(CardId::new(*raw))
            .ok_or_else(|| EffectError::Illegal("scskuiji selected card missing".into()))?;
        if !offered.contains(&card.id.raw())
            || !hand_zones.iter().flatten().any(|zone| *zone == card.zone)
            || !seen_cards.insert(card.id.raw())
        {
            return Err(EffectError::Illegal(
                "scskuiji selection must reference distinct offered hand cards".into(),
            ));
        }
        let suit =
            crate::command::judge::suit_tag(crate::query::effective_card_suit(tx.state(), player, card));
        if !seen_suits.insert(suit) {
            return Err(EffectError::Illegal(
                "scskuiji selection must use distinct effective suits".into(),
            ));
        }
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
    let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
    let effects = cards
        .iter()
        .map(|card| {
            let owner = tx
                .state()
                .cards
                .get(*card)
                .map(|card| card.owner)
                .unwrap_or(PlayerId::NONE);
            EffectRequest::MoveCards {
                cards: vec![*card],
                to: discard,
                from: tx.state().zones.find(ZoneKind::Hand, owner),
            }
        })
        .collect::<Vec<_>>();
    run_effects(tx, effects).map(|_| ())
}

/// Resolve the 「破试探」 (pskuiji) cross-hand distinct-suit discard with an asymmetric penalty:
/// identical validation/discard to scskuiji, and if the owner and target discarded different
/// numbers of cards, the side that discarded fewer loses 1 HP. (The legacy `rechouai` grant is a
/// dangling no-op in upstream — the skill is never defined — and is intentionally not modeled.)
pub(crate) fn resolve_choose_pskuiji_discard(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 {
        return Err(EffectError::Illegal(
            "bad pskuiji continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let target = PlayerId::new(payload[1]);
    let count = payload[2] as usize;
    let offered_len = payload[4] as usize;
    if observer != player || payload.len() != 5 + offered_len || selections.len() > count {
        return Err(EffectError::Illegal(
            "invalid pskuiji selection context".into(),
        ));
    }
    let offered: std::collections::BTreeSet<_> = payload[5..].iter().copied().collect();
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let mut seen_cards = std::collections::BTreeSet::new();
    let mut seen_suits = std::collections::BTreeSet::new();
    let hand_zones = [
        tx.state().zones.find(ZoneKind::Hand, player),
        tx.state().zones.find(ZoneKind::Hand, target),
    ];
    let mut owner_counts: [(PlayerId, u32); 2] = [(player, 0), (target, 0)];
    for raw in selections {
        let card = tx
            .state()
            .cards
            .get(CardId::new(*raw))
            .ok_or_else(|| EffectError::Illegal("pskuiji selected card missing".into()))?;
        if !offered.contains(&card.id.raw())
            || !hand_zones.iter().flatten().any(|zone| *zone == card.zone)
            || !seen_cards.insert(card.id.raw())
        {
            return Err(EffectError::Illegal(
                "pskuiji selection must reference distinct offered hand cards".into(),
            ));
        }
        let suit =
            crate::command::judge::suit_tag(crate::query::effective_card_suit(tx.state(), player, card));
        if !seen_suits.insert(suit) {
            return Err(EffectError::Illegal(
                "pskuiji selection must use distinct effective suits".into(),
            ));
        }
        for (owner, cnt) in owner_counts.iter_mut() {
            if *owner == card.owner {
                *cnt += 1;
            }
        }
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
    let mut effects: Vec<EffectRequest> = selections
        .iter()
        .copied()
        .map(CardId::new)
        .map(|card| {
            let owner = tx
                .state()
                .cards
                .get(card)
                .map(|card| card.owner)
                .unwrap_or(PlayerId::NONE);
            EffectRequest::MoveCards {
                cards: vec![card],
                to: discard,
                from: tx.state().zones.find(ZoneKind::Hand, owner),
            }
        })
        .collect();
    // Asymmetric penalty: the side that discarded fewer loses 1 HP.
    let [(_owner_a, cnt_a), (_owner_b, cnt_b)] = owner_counts;
    if cnt_a != cnt_b {
        let under_discarder = if cnt_a < cnt_b { player } else { target };
        effects.push(EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: under_discarder,
            amount: 1,
            is_lose_hp: true,
            nature: 0,
        });
    }
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolve_choose_inspected_heart_card(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, target_raw, skill_raw] = payload else {
        return Err(EffectError::Illegal(
            "bad inspected-heart continuation payload".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    if observer != player {
        return Err(EffectError::Illegal(
            "inspected-heart observer mismatch".into(),
        ));
    }
    if cancelled {
        return if selections.is_empty() {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "cancelled inspected-heart selection must be empty".into(),
            ))
        };
    }
    let [card_raw] = selections else {
        return Err(EffectError::Illegal(
            "inspected-heart selection requires one card".into(),
        ));
    };
    let target = PlayerId::new(*target_raw);
    let card = CardId::new(*card_raw);
    let hand = tx.state().zones.find(ZoneKind::Hand, target);
    if !tx.state().cards.get(card).is_some_and(|state| {
        hand == Some(state.zone)
            && crate::query::effective_card_suit(tx.state(), target, state)
                == engine_api::Suit::Heart
    }) {
        return Err(EffectError::Illegal(
            "selected inspected card is no longer a heart hand card".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::InspectTargetHandChooseHeartDiscardOrTop {
            player,
            target,
            stage: 1,
            card,
            skill: engine_api::SkillId::new(*skill_raw),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_inspected_heart_destination(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, target_raw, card_raw, _skill_raw] = payload else {
        return Err(EffectError::Illegal(
            "bad inspected-heart destination payload".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    let target = PlayerId::new(*target_raw);
    let card = CardId::new(*card_raw);
    let [destination] = selections else {
        return Err(EffectError::Illegal(
            "inspected-heart destination requires one option".into(),
        ));
    };
    let hand = tx.state().zones.find(ZoneKind::Hand, target);
    if cancelled
        || observer != player
        || *destination > 1
        || !tx.state().cards.get(card).is_some_and(|state| {
            hand == Some(state.zone)
                && crate::query::effective_card_suit(tx.state(), target, state)
                    == engine_api::Suit::Heart
        })
    {
        return Err(EffectError::Illegal(
            "inspected-heart destination is no longer legal".into(),
        ));
    }
    if *destination == 0 {
        return run_effects(
            tx,
            vec![EffectRequest::DiscardTargetCards {
                actor: player,
                target,
                cards: vec![card],
                scope: rule_api::TargetCardScope::Hand,
            }],
        )
        .map(|_| ());
    }
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("draw pile missing".into()))?;
    run_effects(
        tx,
        vec![
            EffectRequest::RevealCards {
                cards: vec![card],
                to: Vec::new(),
            },
            EffectRequest::MoveCards {
                cards: vec![card],
                to: draw,
                from: hand,
            },
        ],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_yeyan_mode(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, skill_raw] = payload else {
        return Err(EffectError::Illegal("bad Yeyan mode payload".into()));
    };
    let [mode] = selections else {
        return Err(EffectError::Illegal(
            "Yeyan mode requires exactly one option".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    if cancelled || observer != player || *mode > 1 {
        return Err(EffectError::Illegal("invalid Yeyan mode choice".into()));
    }
    if *mode == 1 {
        let suits: std::collections::BTreeSet<_> =
            crate::query::yeyan_hand_cards(tx.state(), player)
                .into_iter()
                .filter_map(|card| tx.state().cards.get(card))
                .map(|card| {
                    crate::command::judge::suit_tag(crate::query::effective_card_suit(
                        tx.state(),
                        player,
                        card,
                    ))
                })
                .collect();
        if suits.len() < 4 {
            return Err(EffectError::Illegal(
                "big Yeyan no longer has four distinct suits".into(),
            ));
        }
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveYeyan {
            player,
            mode: *mode as u8,
            stage: 1,
            index: 0,
            targets: Vec::new(),
            data: 0,
            skill: engine_api::SkillId::new(*skill_raw),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_yeyan_big_cards(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 3 {
        return Err(EffectError::Illegal("bad big-Yeyan card payload".into()));
    }
    let player = PlayerId::new(payload[0]);
    let skill = engine_api::SkillId::new(payload[1]);
    let count = payload[2] as usize;
    if cancelled || observer != player || payload.len() != 3 + count || selections.len() != 4 {
        return Err(EffectError::Illegal(
            "big Yeyan requires exactly four cards".into(),
        ));
    }
    let original: std::collections::BTreeSet<_> = payload[3..].iter().copied().collect();
    let current: std::collections::BTreeSet<_> = crate::query::yeyan_hand_cards(tx.state(), player)
        .into_iter()
        .map(CardId::raw)
        .collect();
    let mut cards = std::collections::BTreeSet::new();
    let suits: std::collections::BTreeSet<_> = selections
        .iter()
        .filter_map(|raw| {
            let card = tx.state().cards.get(CardId::new(*raw))?;
            (original.contains(raw) && current.contains(raw) && cards.insert(*raw)).then(|| {
                crate::command::judge::suit_tag(crate::query::effective_card_suit(tx.state(), player, card))
            })
        })
        .collect();
    if cards.len() != 4 || suits.len() != 4 {
        return Err(EffectError::Illegal(
            "big Yeyan cards must have four distinct effective suits".into(),
        ));
    }
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveYeyan {
            player,
            mode: 1,
            stage: 2,
            index: 0,
            targets: Vec::new(),
            data: 0,
            skill,
        });
    run_effects(
        tx,
        vec![EffectRequest::DiscardCards {
            player,
            cards: selections.iter().copied().map(CardId::new).collect(),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_yeyan_targets(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, mode, skill_raw] = payload else {
        return Err(EffectError::Illegal("bad Yeyan target payload".into()));
    };
    let player = PlayerId::new(*player_raw);
    let max = if *mode == 0 { 3 } else { 2 };
    let mut unique = std::collections::BTreeSet::new();
    let targets: Vec<_> = selections
        .iter()
        .copied()
        .map(PlayerId::new)
        .filter(|target| unique.insert(target.raw()))
        .collect();
    if cancelled
        || observer != player
        || !matches!(*mode, 0 | 1)
        || targets.len() != selections.len()
        || !(1..=max).contains(&targets.len())
        || targets.iter().any(|target| {
            !tx.state()
                .players
                .get(*target)
                .is_some_and(|participant| participant.alive)
        })
    {
        return Err(EffectError::Illegal(
            "Yeyan targets are no longer legal".into(),
        ));
    }
    let skill = engine_api::SkillId::new(*skill_raw);
    if *mode == 0 {
        return run_effects(
            tx,
            vec![EffectRequest::ResolveYeyan {
                player,
                mode: 0,
                stage: 4,
                index: 0,
                targets,
                data: 0,
                skill,
            }],
        )
        .map(|_| ());
    }
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveYeyan {
            player,
            mode: 1,
            stage: 3,
            index: 0,
            targets,
            data: 0,
            skill,
        });
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: player,
            amount: 3,
            is_lose_hp: true,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_yeyan_single_damage(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, skill_raw, target_raw] = payload else {
        return Err(EffectError::Illegal(
            "bad single-target Yeyan damage payload".into(),
        ));
    };
    let [amount] = selections else {
        return Err(EffectError::Illegal(
            "single-target Yeyan requires one amount".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    if cancelled || observer != player || !matches!(*amount, 2 | 3) {
        return Err(EffectError::Illegal(
            "invalid single-target Yeyan damage".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveYeyan {
            player,
            mode: 1,
            stage: 4,
            index: 0,
            targets: vec![PlayerId::new(*target_raw)],
            data: *amount,
            skill: engine_api::SkillId::new(*skill_raw),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_yeyan_double_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, skill_raw, first, second] = payload else {
        return Err(EffectError::Illegal(
            "bad two-target Yeyan allocation payload".into(),
        ));
    };
    let [boosted] = selections else {
        return Err(EffectError::Illegal(
            "two-target Yeyan requires one boosted target".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    if cancelled || observer != player || ![*first, *second].contains(boosted) {
        return Err(EffectError::Illegal(
            "invalid two-target Yeyan allocation".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveYeyan {
            player,
            mode: 1,
            stage: 4,
            index: 0,
            targets: vec![PlayerId::new(*first), PlayerId::new(*second)],
            data: *boosted,
            skill: engine_api::SkillId::new(*skill_raw),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_all_lose_hp_or_recover(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, skill_raw] = payload else {
        return Err(EffectError::Illegal("bad Qinyin choice payload".into()));
    };
    let player = PlayerId::new(*player_raw);
    if observer != player {
        return Err(EffectError::Illegal("Qinyin observer mismatch".into()));
    }
    if cancelled {
        return if selections.is_empty() {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "cancelled Qinyin choice must be empty".into(),
            ))
        };
    }
    let [choice] = selections else {
        return Err(EffectError::Illegal(
            "Qinyin requires one lose-or-recover option".into(),
        ));
    };
    let skill = engine_api::SkillId::new(*skill_raw);
    if *choice > 1
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::End
        || tx
            .state()
            .turn
            .flag(crate::state::turn_flag::DISCARD_PHASE_DISCARDED)
            < 2
    {
        return Err(EffectError::Illegal("Qinyin context changed".into()));
    }
    let effects = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .map(|target| {
            if *choice == 0 {
                EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target,
                    amount: 1,
                    is_lose_hp: true,
                    nature: rule_api::damage_nature::NORMAL,
                }
            } else {
                EffectRequest::RecoverHp {
                    source: player,
                    player: target,
                    amount: 1,
                }
            }
        })
        .collect();
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolve_choose_discard_weapon_or_lose_hp(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, skill_raw, stage] = payload else {
        return Err(EffectError::Illegal(
            "bad discard-weapon-or-lose-HP payload".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    let skill = engine_api::SkillId::new(*skill_raw);
    if observer != player
        || cancelled
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
    {
        return Err(EffectError::Illegal(
            "discard-weapon-or-lose-HP context changed".into(),
        ));
    }
    match (*stage, selections) {
        (0, [0])
            if tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive && participant.hp >= 1) =>
        {
            run_effects(
                tx,
                vec![EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target: player,
                    amount: 1,
                    is_lose_hp: true,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )
            .map(|_| ())
        }
        (0, [1]) => {
            let weapons = crate::effect::discardable_weapon_cards(tx, player);
            if weapons.is_empty() {
                return Err(EffectError::Illegal("no discardable weapon remains".into()));
            }
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                weapons.into_iter().map(CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                ContinuationKind::ChooseDiscardWeaponOrLoseHp,
                vec![player.raw(), skill.raw(), 1],
            )?;
            Err(EffectError::ChoiceRequested)
        }
        (1, [card]) => {
            let card = CardId::new(*card);
            if !crate::effect::discardable_weapon_cards(tx, player).contains(&card) {
                return Err(EffectError::Illegal(
                    "selected card is not a discardable weapon".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player,
                    cards: vec![card],
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal(
            "invalid discard-weapon-or-lose-HP selection".into(),
        )),
    }
}

/// 弃一张手牌/装备牌或失去体力（庸肆 xinyongsi2）的选择恢复。
/// payload: `[player, skill, lose_amount, stage]`；
/// stage 0 = 二选一（0=失去体力, 1=选牌），stage 1 = 确认弃置的牌。
pub(crate) fn resolve_choose_discard_one_he_or_lose_hp(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, skill_raw, lose_amount, stage] = payload else {
        return Err(EffectError::Illegal(
            "bad discard-one-or-lose-HP payload".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    let skill = engine_api::SkillId::new(*skill_raw);
    if observer != player
        || cancelled
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
    {
        return Err(EffectError::Illegal(
            "discard-one-or-lose-HP context changed".into(),
        ));
    }
    match (*stage, selections) {
        (0, [0])
            if tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive && participant.hp >= *lose_amount as i32) =>
        {
            run_effects(
                tx,
                vec![EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target: player,
                    amount: *lose_amount,
                    is_lose_hp: true,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )
            .map(|_| ())
        }
        (0, [1]) => {
            let discardable = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if discardable.is_empty() {
                return Err(EffectError::Illegal("no discardable hand/equip card remains".into()));
            }
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                discardable.into_iter().map(CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                ContinuationKind::ChooseDiscardOneHeOrLoseHp,
                vec![player.raw(), skill.raw(), *lose_amount, 1],
            )?;
            Err(EffectError::ChoiceRequested)
        }
        (1, [card]) => {
            let card = CardId::new(*card);
            if !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&card) {
                return Err(EffectError::Illegal(
                    "selected card is not discardable hand/equip".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player,
                    cards: vec![card],
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal(
            "invalid discard-one-or-lose-HP selection".into(),
        )),
    }
}

/// 随势（suishi2）来源确认恢复：payload `[owner, source, skill, amount]`。
/// 来源确认（selections=[1]）后 owner 失去 amount 点体力；取消则无事发生。
pub(crate) fn resolve_choose_source_confirm_owner_lose_hp(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [owner_raw, source_raw, skill_raw, amount] = payload else {
        return Err(EffectError::Illegal(
            "bad source-confirm-owner-lose-HP payload".into(),
        ));
    };
    let owner = PlayerId::new(*owner_raw);
    let source = PlayerId::new(*source_raw);
    let skill = engine_api::SkillId::new(*skill_raw);
    if observer != source
        || cancelled
        || selections != [1]
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == owner && tx.state().skill_effectively_enabled(instance)
        })
        || !tx
            .state()
            .players
            .get(owner)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "source-confirm-owner-lose-HP context changed".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: owner,
            amount: *amount,
            is_lose_hp: true,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_gain_target_card_alternate_zone_then_target_draw(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 {
        return Err(EffectError::Illegal(
            "bad alternate-zone target gain payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let target = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    let option_count = payload[3] as usize;
    if payload.len() != 4 + option_count
        || observer != player
        || cancelled
        || selections.len() != 1
        || !payload[4..].contains(&selections[0])
    {
        return Err(EffectError::Illegal(
            "invalid alternate-zone target gain choice".into(),
        ));
    }
    run_effects(
        tx,
        vec![
            EffectRequest::ChooseGainTargetCardAlternateZoneThenTargetDraw {
                player,
                target,
                selected: CardId::new(selections[0]),
                skill,
            },
        ],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_qixing_hand_cards(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, skill_raw] = payload else {
        return Err(EffectError::Illegal(
            "bad Qixing hand choice payload".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    let skill = engine_api::SkillId::new(*skill_raw);
    if observer != player {
        return Err(EffectError::Illegal("Qixing observer mismatch".into()));
    }
    if cancelled {
        return if selections.is_empty() {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "cancelled Qixing choice must be empty".into(),
            ))
        };
    }
    let skill_def = tx
        .state()
        .skills
        .get(skill)
        .filter(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        .map(|instance| instance.def.raw())
        .ok_or_else(|| EffectError::Illegal("Qixing skill is no longer enabled".into()))?;
    let current_hand: std::collections::BTreeSet<_> = tx
        .state()
        .hand_cards(player)
        .into_iter()
        .map(CardId::raw)
        .collect();
    let selected: std::collections::BTreeSet<_> = selections.iter().copied().collect();
    let pile = tx.state().skill_pile_cards(player, skill_def);
    if selections.is_empty()
        || selections.len() > u8::MAX as usize
        || selected.len() != selections.len()
        || !selected.is_subset(&current_hand)
        || selections.len() > pile.len()
    {
        return Err(EffectError::Illegal(
            "Qixing hand cards are no longer legal".into(),
        ));
    }
    let mut next_payload = vec![player.raw(), skill.raw(), selections.len() as u32];
    next_payload.extend_from_slice(selections);
    crate::effect::open_choice(
        tx,
        player,
        selections.len() as u8,
        selections.len() as u8,
        false,
        pile.into_iter().map(CardId::raw).collect(),
        engine_api::ChoiceKind::SelectCards,
        ContinuationKind::ChooseQixingPileCards,
        next_payload,
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_qixing_pile_cards(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 3 {
        return Err(EffectError::Illegal(
            "bad Qixing pile choice payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let skill = engine_api::SkillId::new(payload[1]);
    let count = payload[2] as usize;
    if observer != player
        || cancelled
        || count == 0
        || payload.len() != 3 + count
        || selections.len() != count
    {
        return Err(EffectError::Illegal(
            "Qixing requires equal hand and pile card counts".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveQixing {
            player,
            initialize: false,
            stage: 2,
            hand_cards: payload[3..].iter().copied().map(CardId::new).collect(),
            pile_cards: selections.iter().copied().map(CardId::new).collect(),
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_qixing_weather_cards(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, _skill_raw, qixing_raw, _helper_raw, max_targets] = payload else {
        return Err(EffectError::Illegal(
            "bad Qixing weather card payload".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    if observer != player {
        return Err(EffectError::Illegal(
            "Qixing weather observer mismatch".into(),
        ));
    }
    if cancelled {
        return if selections.is_empty() {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "cancelled Qixing weather choice must be empty".into(),
            ))
        };
    }
    let current: std::collections::BTreeSet<_> = tx
        .state()
        .skill_pile_cards(player, *qixing_raw)
        .into_iter()
        .map(CardId::raw)
        .collect();
    let selected: std::collections::BTreeSet<_> = selections.iter().copied().collect();
    if selections.is_empty()
        || selections.len() > *max_targets as usize
        || selected.len() != selections.len()
        || !selected.is_subset(&current)
    {
        return Err(EffectError::Illegal(
            "Qixing weather cards are no longer legal".into(),
        ));
    }
    let mut next_payload = payload.to_vec();
    next_payload.push(selections.len() as u32);
    next_payload.extend_from_slice(selections);
    crate::effect::open_choice(
        tx,
        player,
        selections.len() as u8,
        selections.len() as u8,
        false,
        tx.state()
            .players
            .alive_by_seat()
            .into_iter()
            .map(PlayerId::raw)
            .collect(),
        engine_api::ChoiceKind::SelectTargets,
        ContinuationKind::ChooseQixingWeatherTargets,
        next_payload,
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_qixing_weather_targets(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 6 {
        return Err(EffectError::Illegal(
            "bad Qixing weather target payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let count = payload[5] as usize;
    if observer != player
        || cancelled
        || count == 0
        || payload.len() != 6 + count
        || selections.len() != count
    {
        return Err(EffectError::Illegal(
            "Qixing weather requires equal card and target counts".into(),
        ));
    }
    let qixing_skill_def = engine_api::SkillDefId::new(payload[2]);
    let cards: Vec<_> = payload[6..].iter().copied().map(CardId::new).collect();
    let current: std::collections::BTreeSet<_> = tx
        .state()
        .skill_pile_cards(player, qixing_skill_def.raw())
        .into_iter()
        .collect();
    let selected: std::collections::BTreeSet<_> = cards.iter().copied().collect();
    let mut unique_targets = std::collections::BTreeSet::new();
    if selected.len() != cards.len()
        || !selected.is_subset(&current)
        || !selections.iter().all(|target| {
            let target = PlayerId::new(*target);
            unique_targets.insert(target)
                && tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
        })
    {
        return Err(EffectError::Illegal(
            "Qixing weather selection is no longer legal".into(),
        ));
    }
    let pile = tx
        .state()
        .skill_piles
        .iter()
        .find(|pile| pile.owner == player && pile.skill_def == qixing_skill_def.raw())
        .map(|pile| pile.zone)
        .ok_or_else(|| EffectError::Illegal("Qixing pile is missing".into()))?;
    let discard = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("discard pile is missing".into()))?;
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveQixingWeather {
            player,
            qixing_skill_def,
            helper_skill_def: engine_api::SkillDefId::new(payload[3]),
            max_targets: payload[4] as u8,
            stage: 1,
            cards: cards.clone(),
            targets: selections.iter().copied().map(PlayerId::new).collect(),
            skill: engine_api::SkillId::new(payload[1]),
        });
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards,
            to: discard,
            from: Some(pile),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_link_other(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, skill_raw] = payload else {
        return Err(EffectError::Illegal("bad link-other choice payload".into()));
    };
    let player = PlayerId::new(*player_raw);
    let [target_raw] = selections else {
        return Err(EffectError::Illegal(
            "link-other choice requires one target".into(),
        ));
    };
    let target = PlayerId::new(*target_raw);
    let skill = engine_api::SkillId::new(*skill_raw);
    let target_unlinked = tx.state().players.get(target).is_some_and(|participant| {
        participant.alive
            && !participant
                .marks
                .iter()
                .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
    });
    if observer != player
        || cancelled
        || player == target
        || !target_unlinked
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::End
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "link-other choice is no longer legal".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::AddMark {
            player: target,
            mark: engine_api::MarkDefId::new(crate::content::MARK_CHAIN),
            delta: 1,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_drlt_poxi_cards(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 {
        return Err(EffectError::Illegal("bad Poxi choice payload".into()));
    }
    let player = PlayerId::new(payload[0]);
    let target = PlayerId::new(payload[1]);
    let hand_limit_skill_def = engine_api::SkillDefId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    let option_count = payload[4] as usize;
    let options = payload
        .get(5..5 + option_count)
        .ok_or_else(|| EffectError::Illegal("truncated Poxi choice payload".into()))?;
    if observer != player {
        return Err(EffectError::Illegal("Poxi chooser mismatch".into()));
    }
    if cancelled {
        if !selections.is_empty() {
            return Err(EffectError::Illegal(
                "cancelled Poxi choice must be empty".into(),
            ));
        }
        return Ok(());
    }
    if selections.len() != 4 {
        return Err(EffectError::Illegal(
            "Poxi requires exactly four cards".into(),
        ));
    }
    let mut selected = selections.to_vec();
    selected.sort_unstable();
    selected.dedup();
    if selected.len() != 4 || selected.iter().any(|card| !options.contains(card)) {
        return Err(EffectError::Illegal(
            "Poxi selected cards are not distinct offered options".into(),
        ));
    }
    let player_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("missing Poxi owner hand".into()))?;
    let target_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("missing Poxi target hand".into()))?;
    if player == target
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "Poxi context is no longer legal".into(),
        ));
    }
    let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
    let mut suits = Vec::with_capacity(4);
    let mut own_cards = Vec::new();
    let mut target_cards = Vec::new();
    for card in cards {
        let state = tx
            .state()
            .cards
            .get(card)
            .ok_or_else(|| EffectError::Illegal("Poxi card no longer exists".into()))?;
        if state.owner == player && state.zone == player_hand {
            own_cards.push(card);
        } else if state.owner == target && state.zone == target_hand {
            target_cards.push(card);
        } else {
            return Err(EffectError::Illegal(
                "Poxi card left the selected hands".into(),
            ));
        }
        let suit = crate::query::effective_card_suit(tx.state(), state.owner, state);
        if suit == engine_api::Suit::None || suits.contains(&suit) {
            return Err(EffectError::Illegal(
                "Poxi cards must have four distinct suits".into(),
            ));
        }
        suits.push(suit);
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("missing discard pile".into()))?;
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveDrltPoxi {
            player,
            target,
            hand_limit_skill_def,
            stage: 1,
            own_count: own_cards.len() as u8,
            skill,
        });
    let mut effects = Vec::new();
    if !own_cards.is_empty() {
        effects.push(EffectRequest::MoveCards {
            cards: own_cards,
            to: discard,
            from: Some(player_hand),
        });
    }
    if !target_cards.is_empty() {
        effects.push(EffectRequest::MoveCards {
            cards: target_cards,
            to: discard,
            from: Some(target_hand),
        });
    }
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolve_choose_drlt_jieying_transfer(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, mark_raw, skill_raw] = payload else {
        return Err(EffectError::Illegal("bad Jieying transfer payload".into()));
    };
    let [target_raw] = selections else {
        return Err(EffectError::Illegal(
            "Jieying transfer requires one target".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    let target = PlayerId::new(*target_raw);
    let mark = engine_api::MarkDefId::new(*mark_raw);
    let skill = engine_api::SkillId::new(*skill_raw);
    let mark_count = |target: PlayerId| {
        tx.state()
            .players
            .get(target)
            .and_then(|participant| {
                participant
                    .marks
                    .iter()
                    .find(|(found, _)| *found == mark.raw())
                    .map(|(_, count)| *count)
            })
            .unwrap_or(0)
            .max(0) as u32
    };
    let count = mark_count(player);
    if observer != player
        || cancelled
        || player == target
        || count == 0
        || mark_count(target) != 0
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::End
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "Jieying transfer is no longer legal".into(),
        ));
    }
    run_effects(
        tx,
        vec![
            EffectRequest::RemoveMark {
                player,
                mark,
                amount: 0,
            },
            EffectRequest::AddMark {
                player: target,
                mark,
                delta: count.min(i32::MAX as u32) as i32,
            },
        ],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_drlt_duorui_disable_slot(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, target_raw, helper_raw, skill_raw] = payload else {
        return Err(EffectError::Illegal("bad Duorui slot payload".into()));
    };
    let [slot] = selections else {
        return Err(EffectError::Illegal(
            "Duorui requires one equipment slot".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    let slot = u8::try_from(*slot)
        .ok()
        .and_then(crate::state::EquipSlot::from_tag)
        .ok_or_else(|| EffectError::Illegal("invalid Duorui equipment slot".into()))?;
    if observer != player
        || cancelled
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|owner| owner.disabled_equip_slots[slot.index()] == 0)
    {
        return Err(EffectError::Illegal(
            "Duorui equipment slot is no longer enabled".into(),
        ));
    }
    run_effects(
        tx,
        vec![
            EffectRequest::DisableEquipSlots {
                player,
                slots: vec![slot as u8],
            },
            EffectRequest::ResolveDrltDuorui {
                player,
                target: PlayerId::new(*target_raw),
                helper_skill_def: engine_api::SkillDefId::new(*helper_raw),
                stage: 1,
                selected_skill_def: engine_api::SkillDefId::NONE,
                skill: engine_api::SkillId::new(*skill_raw),
            },
        ],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_drlt_duorui_skill(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, target_raw, helper_raw, skill_raw] = payload else {
        return Err(EffectError::Illegal("bad Duorui skill payload".into()));
    };
    let [selected] = selections else {
        return Err(EffectError::Illegal("Duorui requires one skill".into()));
    };
    let player = PlayerId::new(*player_raw);
    if observer != player || cancelled {
        return Err(EffectError::Illegal("invalid Duorui skill choice".into()));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveDrltDuorui {
            player,
            target: PlayerId::new(*target_raw),
            helper_skill_def: engine_api::SkillDefId::new(*helper_raw),
            stage: 2,
            selected_skill_def: engine_api::SkillDefId::new(*selected),
            skill: engine_api::SkillId::new(*skill_raw),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_drlt_zhiti_enable_slot(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, skill_raw] = payload else {
        return Err(EffectError::Illegal("bad Zhiti slot payload".into()));
    };
    let [slot] = selections else {
        return Err(EffectError::Illegal(
            "Zhiti requires one equipment slot".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    let slot = u8::try_from(*slot)
        .ok()
        .and_then(crate::state::EquipSlot::from_tag)
        .ok_or_else(|| EffectError::Illegal("invalid Zhiti equipment slot".into()))?;
    let skill = engine_api::SkillId::new(*skill_raw);
    if observer != player
        || cancelled
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|owner| owner.disabled_equip_slots[slot.index()] > 0)
    {
        return Err(EffectError::Illegal(
            "Zhiti equipment slot is no longer disabled".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::EnableEquipSlot {
            player,
            slot: slot as u8,
            source: player,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_jilue_jizhi_discard(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, card_raw, skill_raw] = payload else {
        return Err(EffectError::Illegal("bad Jilue Jizhi payload".into()));
    };
    let [discard] = selections else {
        return Err(EffectError::Illegal(
            "Jilue Jizhi requires one confirmation".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    if observer != player || cancelled || *discard > 1 {
        return Err(EffectError::Illegal(
            "invalid Jilue Jizhi confirmation".into(),
        ));
    }
    if *discard == 0 {
        return Ok(());
    }
    let card = CardId::new(*card_raw);
    let skill = engine_api::SkillId::new(*skill_raw);
    let skill_def = tx
        .state()
        .skills
        .get(skill)
        .filter(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        .map(|instance| instance.def)
        .ok_or_else(|| EffectError::Illegal("Jilue Jizhi skill context changed".into()))?;
    if !tx.state().hand_cards(player).contains(&card)
        || !tx.state().cards.get(card).is_some_and(|card_state| {
            crate::query::effective_card_kind(tx.state(), player, card_state).is_basic()
        })
    {
        return Err(EffectError::Illegal(
            "Jilue Jizhi drawn card is no longer discardable".into(),
        ));
    }
    run_effects(
        tx,
        vec![
            EffectRequest::DiscardCards {
                player,
                cards: vec![card],
            },
            EffectRequest::AddMark {
                player,
                mark: engine_api::MarkDefId::new(skill_def.raw()),
                delta: 1,
            },
        ],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_max_mark_other_judge_kill_unless_tao_or_taoyuan(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, mark_raw, _skill_raw] = payload else {
        return Err(EffectError::Illegal(
            "bad max-mark death judgment payload".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    if observer != player || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "max-mark death judgment requires one target".into(),
        ));
    }
    let mark = engine_api::MarkDefId::new(*mark_raw);
    let target = PlayerId::new(selections[0]);
    let mark_count = |candidate: &crate::state::Player| {
        candidate
            .marks
            .iter()
            .find(|(definition, _)| *definition == mark.raw())
            .map_or(0, |(_, count)| *count)
    };
    let max = tx
        .state()
        .players
        .iter_live()
        .filter(|candidate| candidate.alive && candidate.id != player)
        .map(mark_count)
        .max()
        .unwrap_or(0);
    let valid = max > 0
        && tx.state().players.get(target).is_some_and(|candidate| {
            candidate.alive && candidate.id != player && mark_count(candidate) == max
        });
    if !valid {
        return Err(EffectError::Illegal(
            "max-mark death judgment target is stale".into(),
        ));
    }

    let result = crate::command::judge::flip_judgment(
        tx,
        target,
        crate::command::judge::JudgmentFavor::Neutral,
    )?;
    let kind = tx
        .state()
        .cards
        .get(result.card)
        .map(|card| card.kind)
        .ok_or_else(|| EffectError::EngineFault("judgment result card is missing".into()))?;
    let mut effects = vec![EffectRequest::EmitJudgmentResolved {
        player: target,
        result_card: result.card,
        suit: result.suit,
        rank: result.rank,
        cause_card: CardId::NONE,
    }];
    if !matches!(kind, CardKind::Tao | CardKind::TrickTaoyuan) {
        effects.push(EffectRequest::KillPlayer {
            player: target,
            killer: PlayerId::NONE,
        });
    }
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolve_recover_to_max_then_optional_exchange_skill(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [removable_skill_def, restore_skill_def] = payload else {
        return Err(EffectError::Illegal(
            "bad recover-to-max skill exchange payload".into(),
        ));
    };
    if cancelled || selections.len() != 1 || !matches!(selections[0], 0 | 1) {
        return Err(EffectError::Illegal(
            "recover-to-max skill exchange requires one confirm choice".into(),
        ));
    }
    if selections[0] == 0 {
        return Ok(());
    }
    let removable_skill_def = engine_api::SkillDefId::new(*removable_skill_def);
    let restore_skill_def = engine_api::SkillDefId::new(*restore_skill_def);
    let valid = tx.state().players.get(observer).is_some_and(|participant| {
        participant.alive
            && participant.skills.iter().any(|skill| {
                tx.state()
                    .skills
                    .get(*skill)
                    .is_some_and(|instance| instance.def == removable_skill_def)
            })
    });
    if !valid {
        return Ok(());
    }
    run_effects(
        tx,
        vec![
            EffectRequest::RemoveSkillByDef {
                player: observer,
                skill_def: removable_skill_def,
            },
            EffectRequest::AddSkill {
                player: observer,
                skill_def: restore_skill_def,
            },
        ],
    )
    .map(|_| ())
}

pub(crate) fn response_card_zone(
    tx: &Transaction,
    player: PlayerId,
    card: CardId,
) -> Result<engine_api::ZoneId, EffectError> {
    let card = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("bad response card".into()))?;
    let in_hand = tx.state().zones.find(ZoneKind::Hand, player) == Some(card.zone);
    let in_equip = tx.state().zones.find(ZoneKind::Equip, player) == Some(card.zone);
    let in_expansion = tx.state().zones.find(ZoneKind::Expansion, player) == Some(card.zone);
    let borrowed = crate::query::borrowed_hand_source(tx.state(), player, card.id).is_some();
    let fusi = crate::command::skill::dddfusi_approved_source(tx.state(), player, card.id).is_some()
        && card.face_up
        && tx
            .state()
            .zones
            .get(card.zone)
            .is_some_and(|zone| zone.kind == ZoneKind::Hand);
    if (!borrowed && !fusi && card.owner != player)
        || (!in_hand && !in_equip && !in_expansion && !borrowed && !fusi)
    {
        return Err(EffectError::Illegal(
            "response card not owned in hand, equip, or expansion".into(),
        ));
    }
    Ok(card.zone)
}

pub(crate) fn append_pothaoshi_refills(
    state: &crate::state::GameState,
    player: PlayerId,
    cards: &[CardId],
    effects: &mut Vec<EffectRequest>,
) {
    effects.extend(crate::effect::pothaoshi_refill_effects(
        state, player, cards,
    ));
}

pub(crate) fn resolve_choose_take_larger_hand_until_max(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    match (payload.first().copied(), payload) {
        (Some(0), [_, player, skill]) => {
            let player = PlayerId::new(*player);
            let skill = engine_api::SkillId::new(*skill);
            if observer != player {
                return Err(EffectError::Illegal(
                    "larger-hand target choice belongs to another player".into(),
                ));
            }
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal(
                        "cancelled larger-hand target choice has selections".into(),
                    ));
                }
                return Ok(());
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "select exactly one larger-hand target".into(),
                ));
            }
            let target = PlayerId::new(selections[0]);
            crate::effect::open_take_larger_hand_card_choice(tx, player, target, skill).map(|_| ())
        }
        (Some(1), [_, player, target, skill]) => {
            let player = PlayerId::new(*player);
            let target = PlayerId::new(*target);
            let skill = engine_api::SkillId::new(*skill);
            if observer != player || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "larger-hand card choice requires exactly one card".into(),
                ));
            }
            if tx.state().hand_count(target) <= tx.state().hand_count(player) {
                return Err(EffectError::Illegal(
                    "chosen target no longer has more hand cards".into(),
                ));
            }
            let card = CardId::new(selections[0]);
            let from = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| {
                    EffectError::Illegal("larger-hand target has no hand zone".into())
                })?;
            if !tx.state().hand_cards(target).contains(&card) {
                return Err(EffectError::Illegal(
                    "selected card is no longer in the larger hand".into(),
                ));
            }
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("skill owner has no hand zone".into()))?;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ContinueTakeLargerHandUntilMax { player, skill });
            run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![card],
                    to,
                    from: Some(from),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "bad larger-hand take continuation payload".into(),
        )),
    }
}
