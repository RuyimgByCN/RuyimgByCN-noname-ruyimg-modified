//! Command operations for `resolve_choose_other_give_one_hand_equip` and related helpers.

use super::*;

pub(crate) fn resolve_choose_other_give_one_hand_equip(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad choose-other gift continuation".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let skill = engine_api::SkillId::new(*skill);
    if cancelled || observer != player || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "choose exactly one other gift target".into(),
        ));
    }
    let target = PlayerId::new(selections[0]);
    let skill_enabled = tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    });
    let player_alive = tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive);
    let target_alive = tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive);
    let has_card = [ZoneKind::Hand, ZoneKind::Equip].into_iter().any(|kind| {
        tx.state()
            .zones
            .find(kind, player)
            .and_then(|zone| tx.state().zones.get(zone))
            .is_some_and(|zone| !zone.cards.is_empty())
    });
    if !skill_enabled || !player_alive || !target_alive || player == target || !has_card {
        return Err(EffectError::Illegal(
            "choose-other gift context changed".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::DrawThenGiveOne {
            player,
            target,
            count: 0,
            drawn_only: false,
            use_given_equipment: false,
            skill,
            drawn: Vec::new(),
            ready_to_give: true,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_discard_target_card_gain_mount(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let scope = payload
        .get(2)
        .and_then(|tag| u8::try_from(*tag).ok())
        .and_then(rule_api::TargetCardScope::from_tag)
        .ok_or_else(|| EffectError::Illegal("invalid target-card scope".into()))?;
    resolve_scoped_discard_target_card_gain_mount(
        tx, observer, payload, selections, cancelled, scope, true,
    )
}

pub(crate) fn resolve_choose_discard_target_card(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let scope = payload
        .get(2)
        .and_then(|tag| u8::try_from(*tag).ok())
        .and_then(rule_api::TargetCardScope::from_tag)
        .ok_or_else(|| EffectError::Illegal("invalid target-card scope".into()))?;
    resolve_scoped_discard_target_card_gain_mount(
        tx, observer, payload, selections, cancelled, scope, false,
    )
}

pub(crate) fn resolve_choose_target_reclaim_equipment_draw_if_range_reduced(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.first().unwrap_or(&0));
    let target = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(2).unwrap_or(&0));
    let option_count = *payload
        .get(3)
        .ok_or_else(|| EffectError::EngineFault("truncated reclamation options".into()))?
        as usize;
    if payload.len() != 4 + option_count {
        return Err(EffectError::EngineFault(
            "invalid reclamation option count".into(),
        ));
    }
    if cancelled || observer != player || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "choose exactly one target equipment card".into(),
        ));
    }
    let card = CardId::new(selections[0]);
    let equip = tx
        .state()
        .zones
        .find(ZoneKind::Equip, target)
        .ok_or_else(|| EffectError::Illegal("missing target equipment zone".into()))?;
    if !payload[4..].contains(&card.raw())
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .zones
            .get(equip)
            .is_some_and(|zone| zone.cards.contains(&card))
    {
        return Err(EffectError::Illegal(
            "selected target equipment is stale".into(),
        ));
    }
    let before = crate::effect::attack_range_population(tx, target);
    run_effects(
        tx,
        vec![
            EffectRequest::CommitTargetReclaimEquipmentDrawIfRangeReduced {
                player,
                target,
                card,
                before,
                skill,
            },
        ],
    )
    .map(|_| ())
}

pub(crate) fn resolve_twchenxun_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(2).unwrap_or(&0));
    let option_count = *payload
        .get(3)
        .ok_or_else(|| EffectError::EngineFault("truncated TW Chenxun options".into()))?
        as usize;
    if payload.len() != 4 + option_count || observer != player {
        return Err(EffectError::Illegal(
            "TW Chenxun target context changed".into(),
        ));
    }
    if cancelled {
        return if selections.is_empty() {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "cancelled TW Chenxun has selections".into(),
            ))
        };
    }
    if selections.len() != 1 || !payload[4..].contains(&selections[0]) {
        return Err(EffectError::Illegal(
            "TW Chenxun requires one offered target".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveTwchenxun {
            player,
            target: PlayerId::new(selections[0]),
            card: CardId::NONE,
            action: 3,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_twfenxian_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "TW Fenxian requires one offered choice".into(),
        ));
    }
    let stage = payload[1];
    let owner = PlayerId::new(payload[2]);
    let actor = PlayerId::new(payload[3]);
    let skill = engine_api::SkillId::new(payload[4]);
    if observer != actor || !matches!((stage, payload.len()), (0 | 3, 6) | (2, 5)) {
        return Err(EffectError::Illegal(
            "TW Fenxian choice context changed".into(),
        ));
    }
    let request = match stage {
        0 if selections[0] <= 1 && payload[5] & (1 << selections[0]) != 0 => {
            EffectRequest::ResolveTwfenxian {
                player: owner,
                target: actor,
                card: CardId::NONE,
                action: if selections[0] == 0 { 2 } else { 1 },
                skill,
            }
        }
        2 => EffectRequest::ResolveTwfenxian {
            player: owner,
            target: actor,
            card: CardId::new(selections[0]),
            action: 3,
            skill,
        },
        3 => EffectRequest::ResolveTwfenxian {
            player: actor,
            target: PlayerId::new(selections[0]),
            card: CardId::new(payload[5]),
            action: 4,
            skill,
        },
        _ => {
            return Err(EffectError::Illegal(
                "TW Fenxian choice is not offered".into(),
            ));
        }
    };
    run_effects(tx, vec![request]).map(|_| ())
}

pub(crate) fn resolve_choose_use_virtual_card(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.first().unwrap_or(&0));
    let card = CardId::new(*payload.get(1).unwrap_or(&0));
    let kind = payload
        .get(2)
        .and_then(|value| u8::try_from(*value).ok())
        .and_then(CardKind::from_tag)
        .filter(|kind| matches!(kind, CardKind::TrickWanjian | CardKind::TrickJuedou))
        .ok_or_else(|| EffectError::EngineFault("invalid virtual-card choice kind".into()))?;
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    let option_count = *payload
        .get(4)
        .ok_or_else(|| EffectError::EngineFault("truncated virtual-card options".into()))?
        as usize;
    let option_end = 5 + option_count;
    let dcctjiuxian = payload.get(option_end) == Some(&1);
    if payload.len() != option_end + usize::from(dcctjiuxian) {
        return Err(EffectError::EngineFault(
            "invalid virtual-card option count".into(),
        ));
    }
    if cancelled || observer != player {
        return Err(EffectError::Illegal(
            "mandatory virtual-card target choice cannot be cancelled".into(),
        ));
    }
    if !tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    }) || tx
        .state()
        .cards
        .get(card)
        .is_none_or(|state| state.kind != kind)
    {
        return Err(EffectError::Illegal(
            "virtual-card use context is stale".into(),
        ));
    }
    let crate::command::play::SkillCardTargetSpec::Select { min, max, options } =
        crate::command::play::skill_card_target_spec(tx, player, card)?
    else {
        return Err(EffectError::Illegal(
            "virtual-card target contract changed".into(),
        ));
    };
    let mut distinct = std::collections::BTreeSet::new();
    let selected: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
    if selected.len() < usize::from(min)
        || selected.len() > usize::from(max)
        || selected
            .iter()
            .any(|target| !distinct.insert(*target) || !options.contains(target))
        || selections
            .iter()
            .any(|target| !payload[5..option_end].contains(target))
    {
        return Err(EffectError::Illegal(
            "virtual-card target selection is invalid or stale".into(),
        ));
    }
    if dcctjiuxian {
        if kind != CardKind::TrickJuedou
            || selected.len() != 1
            || !tx
                .state_mut()
                .skills
                .get_mut(skill)
                .is_some_and(|instance| instance.record_active_play_phase_target(selected[0]))
        {
            return Err(EffectError::Illegal(
                "Dcctjiuxian Duel target context changed".into(),
            ));
        }
    }
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_CARD, card.raw() as i32);
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_SKILL, skill.raw() as i32);
    tx.state_mut().turn.set_flag(
        crate::state::turn_flag::VIEW_AS_LOGICAL_KIND,
        i32::from(kind.tag()),
    );
    let result = crate::command::play::handle_skill_card(tx, player, card, &selected);
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_CARD, 0);
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_SKILL, 0);
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_LOGICAL_KIND, 0);
    result
}

pub(crate) fn resolve_choose_discard_target_card_current_sha(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if !matches!(payload, [_, _, _] | [_, _, _, _]) {
        return Err(EffectError::EngineFault(
            "bad current Sha target-card payload".into(),
        ));
    }
    if cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "choose exactly one target card for current Sha".into(),
        ));
    }
    let source = PlayerId::new(*payload.first().unwrap_or(&0));
    let target = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let card = CardId::new(*payload.get(2).unwrap_or(&0));
    let nonbasic_bonus_skill = payload.get(3).copied().map(engine_api::SkillDefId::new);
    if observer != source {
        return Err(EffectError::Illegal(
            "only the current Sha user may resolve the target-card choice".into(),
        ));
    }
    crate::command::play::validate_saved_sha_target(tx, source, card, target)?;
    let selected = CardId::new(selections[0]);
    if !crate::query::discardable_target_cards(
        tx.state(),
        source,
        target,
        rule_api::TargetCardScope::HandEquip,
    )
    .contains(&selected)
    {
        return Err(EffectError::Illegal(
            "current Sha target-card selection is no longer legal".into(),
        ));
    }
    let selected_kind = tx
        .state()
        .cards
        .get(selected)
        .map(|selected| crate::query::effective_card_kind(tx.state(), source, selected))
        .ok_or_else(|| EffectError::Illegal("selected target card no longer exists".into()))?;
    let blocks_response = nonbasic_bonus_skill
        .map_or_else(|| selected_kind.is_equip(), |_| !selected_kind.is_basic());
    let mut effects = vec![EffectRequest::DiscardTargetCards {
        actor: source,
        target,
        cards: vec![selected],
        scope: rule_api::TargetCardScope::HandEquip,
    }];
    if blocks_response {
        effects.push(EffectRequest::MakeCurrentShaTargetUnrespondable {
            source,
            card,
            target,
        });
        if let Some(skill_def) = nonbasic_bonus_skill {
            effects.push(EffectRequest::AddSkill {
                player: source,
                skill_def,
            });
            effects.push(EffectRequest::AddMark {
                player: source,
                mark: engine_api::MarkDefId::new(skill_def.raw()),
                delta: 1,
            });
        }
    } else {
        effects.push(EffectRequest::GainEventCards {
            player: target,
            cards: vec![card],
        });
    }
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolve_dcqiaomeng(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 7 {
        return Err(EffectError::EngineFault(
            "bad Dcqiaomeng choice payload".into(),
        ));
    }
    let target_count = payload[6] as usize;
    let Some(targets_raw) = payload.get(7..7 + target_count) else {
        return Err(EffectError::EngineFault(
            "bad Dcqiaomeng target snapshot".into(),
        ));
    };
    if payload.len() != 7 + target_count {
        return Err(EffectError::EngineFault(
            "bad Dcqiaomeng choice payload length".into(),
        ));
    }
    let [selection] = selections else {
        return Err(EffectError::Illegal(
            "Dcqiaomeng requires exactly one choice".into(),
        ));
    };
    let player = PlayerId::new(payload[1]);
    if cancelled || observer != player {
        return Err(EffectError::Illegal(
            "only the Dcqiaomeng owner may resolve its choice".into(),
        ));
    }
    let card = CardId::new(payload[2]);
    let target = PlayerId::new(payload[3]);
    let helper_skill_def = engine_api::SkillDefId::new(payload[4]);
    let skill = engine_api::SkillId::new(payload[5]);
    let targets = targets_raw.iter().copied().map(PlayerId::new).collect();
    let effect = match payload[0] {
        0 => EffectRequest::ResolveDcqiaomeng {
            player,
            card,
            targets,
            target: PlayerId::new(*selection),
            selected: CardId::NONE,
            helper_skill_def,
            stage: 1,
            skill,
        },
        1 => EffectRequest::ResolveDcqiaomeng {
            player,
            card,
            targets,
            target,
            selected: CardId::new(*selection),
            helper_skill_def,
            stage: 2,
            skill,
        },
        _ => {
            return Err(EffectError::Illegal(
                "invalid Dcqiaomeng choice stage".into(),
            ));
        }
    };
    run_effects(tx, vec![effect]).map(|_| ())
}

pub(crate) fn resolve_retieji(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [source, card, target, suit] = payload else {
        return Err(EffectError::EngineFault(
            "bad Retieji choice payload".into(),
        ));
    };
    let source = PlayerId::new(*source);
    let card = CardId::new(*card);
    let target = PlayerId::new(*target);
    let suit = crate::command::judge::suit_from_tag(*suit);
    if observer != target
        || ![source, target].into_iter().all(|player| {
            tx.state()
                .players
                .get(player)
                .is_some_and(|state| state.alive)
        })
    {
        return Err(EffectError::Illegal(
            "Retieji choice context changed".into(),
        ));
    }
    crate::command::play::validate_saved_sha_target(tx, source, card, target)?;
    if cancelled || selections.is_empty() {
        if !selections.is_empty() {
            return Err(EffectError::Illegal(
                "cancelled Retieji choice contains a card".into(),
            ));
        }
        return run_effects(
            tx,
            vec![
                EffectRequest::MakeCurrentShaTargetUnrespondable {
                    source,
                    card,
                    target,
                },
                EffectRequest::IncreaseCurrentShaTargetDamage {
                    source,
                    card,
                    target,
                    amount: 1,
                },
            ],
        )
        .map(|_| ());
    }
    let [selected] = selections else {
        return Err(EffectError::Illegal(
            "Retieji accepts at most one card".into(),
        ));
    };
    let selected = CardId::new(*selected);
    if !crate::query::discardable_hand_equip_cards(tx.state(), target).contains(&selected)
        || !tx
            .state()
            .cards
            .get(selected)
            .is_some_and(|card| crate::query::effective_card_suit(tx.state(), target, card) == suit)
    {
        return Err(EffectError::Illegal(
            "Retieji card is no longer eligible".into(),
        ));
    }
    let from = response_card_zone(tx, target, selected)?;
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, source)
        .ok_or_else(|| EffectError::EngineFault("Retieji recipient hand is missing".into()))?;
    run_effects(
        tx,
        vec![EffectRequest::GiveCards {
            cards: vec![selected],
            to,
            from,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_new_reluoyi(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, buff_skill_def, skill, card_count, cards @ ..] = payload else {
        return Err(EffectError::EngineFault(
            "bad New Reluoyi choice payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let card_count = usize::try_from(*card_count)
        .map_err(|_| EffectError::EngineFault("bad New Reluoyi card count".into()))?;
    if observer != player
        || cancelled
        || selections.len() != 1
        || cards.len() != card_count
        || !matches!(selections[0], 0 | 1)
    {
        return Err(EffectError::Illegal(
            "New Reluoyi choice context changed".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveNewReluoyi {
            player,
            buff_skill_def: SkillDefId::new(*buff_skill_def),
            stage: 2,
            cards: cards.iter().copied().map(CardId::new).collect(),
            accepted: selections[0] == 1,
            skill: SkillId::new(*skill),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_xinyaoming(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload
        .first()
        .ok_or_else(|| EffectError::EngineFault("missing Xinyaoming stage".into()))?;
    match stage {
        0 => {
            let option_count = *payload.get(3).ok_or_else(|| {
                EffectError::EngineFault("truncated Xinyaoming target payload".into())
            })? as usize;
            if payload.len() != 4 + option_count {
                return Err(EffectError::EngineFault(
                    "bad Xinyaoming target payload".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let skill = SkillId::new(payload[2]);
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "Xinyaoming requires exactly one target".into(),
                ));
            };
            let target = PlayerId::new(*selected);
            let used_mask = tx
                .state()
                .skills
                .get(skill)
                .filter(|instance| {
                    instance.owner == player && tx.state().skill_effectively_enabled(instance)
                })
                .map_or(u8::MAX, |instance| instance.xinyaoming_used_mask());
            if cancelled
                || observer != player
                || !payload[4..].contains(selected)
                || crate::query::xinyaoming_available_actions(tx.state(), player, target, used_mask)
                    .is_empty()
            {
                return Err(EffectError::Illegal(
                    "Xinyaoming target choice context changed".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveXinyaoming {
                    player,
                    target,
                    action: 0,
                    skill,
                }],
            )
            .map(|_| ())
        }
        1 => {
            let option_count = *payload.get(4).ok_or_else(|| {
                EffectError::EngineFault("truncated Xinyaoming action payload".into())
            })? as usize;
            if payload.len() != 5 + option_count {
                return Err(EffectError::EngineFault(
                    "bad Xinyaoming action payload".into(),
                ));
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "Xinyaoming requires exactly one action".into(),
                ));
            };
            let player = PlayerId::new(payload[1]);
            let target = PlayerId::new(payload[2]);
            let skill = SkillId::new(payload[3]);
            let used_mask = tx
                .state()
                .skills
                .get(skill)
                .filter(|instance| {
                    instance.owner == player && tx.state().skill_effectively_enabled(instance)
                })
                .map_or(u8::MAX, |instance| instance.xinyaoming_used_mask());
            let action = u8::try_from(*selected)
                .map_err(|_| EffectError::Illegal("bad Xinyaoming action token".into()))?;
            if cancelled
                || observer != player
                || !payload[5..].contains(selected)
                || !crate::query::xinyaoming_available_actions(
                    tx.state(),
                    player,
                    target,
                    used_mask,
                )
                .contains(&action)
            {
                return Err(EffectError::Illegal(
                    "Xinyaoming action choice context changed".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveXinyaoming {
                    player,
                    target,
                    action,
                    skill,
                }],
            )
            .map(|_| ())
        }
        2 => {
            let option_count = *payload.get(4).ok_or_else(|| {
                EffectError::EngineFault("truncated Xinyaoming discard payload".into())
            })? as usize;
            if payload.len() != 5 + option_count {
                return Err(EffectError::EngineFault(
                    "bad Xinyaoming discard payload".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let target = PlayerId::new(payload[2]);
            let skill = SkillId::new(payload[3]);
            let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
            let current = crate::query::discardable_target_cards(
                tx.state(),
                target,
                target,
                rule_api::TargetCardScope::HandEquip,
            );
            let skill_enabled = tx.state().skills.get(skill).is_some_and(|instance| {
                instance.owner == player && tx.state().skill_effectively_enabled(instance)
            });
            if cancelled
                || observer != target
                || !skill_enabled
                || cards.is_empty()
                || cards.len() > 2
                || cards.iter().enumerate().any(|(index, card)| {
                    cards[..index].contains(card)
                        || !payload[5..].contains(&card.raw())
                        || !current.contains(card)
                })
            {
                return Err(EffectError::Illegal(
                    "Xinyaoming discard choice context changed".into(),
                ));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::DiscardCards {
                        player: target,
                        cards: cards.clone(),
                    },
                    EffectRequest::DrawCardsFromSkill {
                        player: target,
                        count: cards.len() as u32,
                        skill,
                    },
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown Xinyaoming choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_rehuaiyi(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload
        .first()
        .ok_or_else(|| EffectError::EngineFault("missing Rehuaiyi stage".into()))?;
    match stage {
        0 => {
            let option_count = *payload.get(3).ok_or_else(|| {
                EffectError::EngineFault("truncated Rehuaiyi color payload".into())
            })? as usize;
            if payload.len() != 4 + option_count {
                return Err(EffectError::EngineFault(
                    "bad Rehuaiyi color payload".into(),
                ));
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "Rehuaiyi requires exactly one color".into(),
                ));
            };
            let player = PlayerId::new(payload[1]);
            let skill = SkillId::new(payload[2]);
            let color = u8::try_from(*selected)
                .map_err(|_| EffectError::Illegal("bad Rehuaiyi color token".into()))?;
            if cancelled
                || observer != player
                || !matches!(color, 0b01 | 0b10)
                || !payload[4..].contains(selected)
            {
                return Err(EffectError::Illegal(
                    "Rehuaiyi color choice context changed".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveRehuaiyi {
                    player,
                    stage: 0,
                    color,
                    discarded_count: 0,
                    targets: Vec::new(),
                    index: 0,
                    gained_count: 0,
                    skill,
                }],
            )
            .map(|_| ())
        }
        1 => {
            let discarded_count = *payload.get(3).ok_or_else(|| {
                EffectError::EngineFault("truncated Rehuaiyi target payload".into())
            })?;
            let option_count = *payload.get(4).ok_or_else(|| {
                EffectError::EngineFault("truncated Rehuaiyi target options".into())
            })? as usize;
            if payload.len() != 5 + option_count {
                return Err(EffectError::EngineFault(
                    "bad Rehuaiyi target payload".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let skill = SkillId::new(payload[2]);
            if observer != player || (cancelled && !selections.is_empty()) {
                return Err(EffectError::Illegal(
                    "invalid Rehuaiyi target cancellation".into(),
                ));
            }
            if cancelled {
                return Ok(());
            }
            let max = (discarded_count as usize).min(option_count);
            let mut unique = std::collections::BTreeSet::new();
            if selections.is_empty()
                || selections.len() > max
                || !selections.iter().all(|selected| {
                    unique.insert(*selected)
                        && payload[5..].contains(selected)
                        && PlayerId::new(*selected) != player
                        && !crate::query::gainable_target_cards(
                            tx.state(),
                            player,
                            PlayerId::new(*selected),
                            rule_api::TargetCardScope::HandEquip,
                        )
                        .is_empty()
                })
            {
                return Err(EffectError::Illegal(
                    "Rehuaiyi target choice context changed".into(),
                ));
            }
            let selected: std::collections::BTreeSet<_> = selections.iter().copied().collect();
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| selected.contains(&target.raw()))
                .collect();
            run_effects(
                tx,
                vec![EffectRequest::ResolveRehuaiyi {
                    player,
                    stage: 2,
                    color: 0,
                    discarded_count: 0,
                    targets,
                    index: 0,
                    gained_count: 0,
                    skill,
                }],
            )
            .map(|_| ())
        }
        2 => {
            let target_count = *payload
                .get(5)
                .ok_or_else(|| EffectError::EngineFault("truncated Rehuaiyi gain payload".into()))?
                as usize;
            let options_index = 6usize
                .checked_add(target_count)
                .ok_or_else(|| EffectError::EngineFault("Rehuaiyi payload overflow".into()))?;
            let option_count = *payload
                .get(options_index)
                .ok_or_else(|| EffectError::EngineFault("truncated Rehuaiyi card options".into()))?
                as usize;
            if payload.len() != options_index + 1 + option_count {
                return Err(EffectError::EngineFault("bad Rehuaiyi gain payload".into()));
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "Rehuaiyi requires exactly one target card".into(),
                ));
            };
            let player = PlayerId::new(payload[1]);
            let skill = SkillId::new(payload[2]);
            let index = payload[3] as usize;
            let gained_count = payload[4];
            let targets: Vec<_> = payload[6..options_index]
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect();
            let target = targets.get(index).copied().ok_or_else(|| {
                EffectError::EngineFault("Rehuaiyi target index out of bounds".into())
            })?;
            let card = CardId::new(*selected);
            let current = crate::query::gainable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::HandEquip,
            );
            let skill_valid = tx.state().skills.get(skill).is_some_and(|instance| {
                instance.owner == player && tx.state().skill_effectively_enabled(instance)
            });
            if cancelled
                || observer != player
                || !skill_valid
                || !payload[options_index + 1..].contains(selected)
                || !current.contains(&card)
            {
                return Err(EffectError::Illegal(
                    "Rehuaiyi target-card choice context changed".into(),
                ));
            }
            let from = tx
                .state()
                .cards
                .get(card)
                .map(|card| card.zone)
                .ok_or_else(|| EffectError::Illegal("Rehuaiyi card disappeared".into()))?;
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("Rehuaiyi owner hand missing".into()))?;
            run_effects(
                tx,
                vec![
                    EffectRequest::MoveCards {
                        cards: vec![card],
                        to,
                        from: Some(from),
                    },
                    EffectRequest::ResolveRehuaiyi {
                        player,
                        stage: 2,
                        color: 0,
                        discarded_count: 0,
                        targets,
                        index: (index as u32).saturating_add(1),
                        gained_count: gained_count.saturating_add(1),
                        skill,
                    },
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown Rehuaiyi choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_reanguo(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, skill_raw, option_count, options @ ..] = payload else {
        return Err(EffectError::EngineFault(
            "truncated Reanguo recast payload".into(),
        ));
    };
    if options.len() != *option_count as usize {
        return Err(EffectError::EngineFault(
            "bad Reanguo recast payload".into(),
        ));
    }
    let player = PlayerId::new(*player_raw);
    let skill = SkillId::new(*skill_raw);
    if observer != player || (cancelled && !selections.is_empty()) {
        return Err(EffectError::Illegal(
            "invalid Reanguo recast cancellation".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    let hand = tx.state().zones.find(ZoneKind::Hand, player);
    let equip = tx.state().zones.find(ZoneKind::Equip, player);
    let max = options.len().min(u8::MAX as usize);
    let mut unique = std::collections::BTreeSet::new();
    let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
    let skill_valid = tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    });
    if !skill_valid
        || cards.is_empty()
        || cards.len() > max
        || cards.iter().any(|card| {
            !unique.insert(card.raw())
                || !options.contains(&card.raw())
                || tx.state().cards.get(*card).is_none_or(|state| {
                    state.owner != player || (Some(state.zone) != hand && Some(state.zone) != equip)
                })
        })
    {
        return Err(EffectError::Illegal(
            "Reanguo recast choice context changed".into(),
        ));
    }
    run_effects(tx, vec![EffectRequest::RecastCards { player, cards }]).map(|_| ())
}

pub(crate) fn resolve_scoped_discard_target_card_gain_mount(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
    scope: rule_api::TargetCardScope,
    gain_mount: bool,
) -> Result<(), EffectError> {
    if cancelled {
        if !gain_mount && payload.get(3).copied().unwrap_or(0) == 1 {
            return Ok(());
        }
        return Err(EffectError::Illegal(
            "choose exactly one target card".into(),
        ));
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal(
            "choose exactly one target card".into(),
        ));
    }
    let chooser = PlayerId::new(*payload.first().unwrap_or(&0));
    let target = PlayerId::new(*payload.get(1).unwrap_or(&0));
    if observer != chooser {
        return Err(EffectError::Illegal(
            "only the target-card discard chooser may resolve this choice".into(),
        ));
    }
    if ![chooser, target].into_iter().all(|player| {
        tx.state()
            .players
            .get(player)
            .is_some_and(|state| state.alive)
    }) {
        return Err(EffectError::Illegal(
            "target-card discard participants are no longer alive".into(),
        ));
    }
    let selected = CardId::new(selections[0]);
    if !crate::query::discardable_target_cards(tx.state(), chooser, target, scope)
        .contains(&selected)
    {
        return Err(EffectError::Illegal(
            "target card selection is no longer legal".into(),
        ));
    }
    let is_mount = tx
        .state()
        .cards
        .get(selected)
        .is_some_and(|card| matches!(card.kind, CardKind::HorseOff | CardKind::HorseDef));
    let mut effects = vec![EffectRequest::DiscardTargetCards {
        actor: chooser,
        target,
        cards: vec![selected],
        scope,
    }];
    if gain_mount && is_mount {
        let discard = tx
            .state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::EngineFault("missing discard pile".into()))?;
        let hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, chooser)
            .ok_or_else(|| EffectError::EngineFault("missing chooser hand".into()))?;
        effects.push(EffectRequest::MoveCards {
            cards: vec![selected],
            to: hand,
            from: Some(discard),
        });
    }
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolve_beizhu(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    match payload.first().copied() {
        Some(0) => {
            if payload.len() < 5 {
                return Err(EffectError::EngineFault(
                    "bad Beizhu discard payload".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let target = PlayerId::new(payload[2]);
            let skill = engine_api::SkillId::new(payload[3]);
            let option_count = payload[4] as usize;
            if cancelled
                || observer != player
                || selections.len() != 1
                || payload.len() != 5 + option_count
            {
                return Err(EffectError::Illegal(
                    "Beizhu requires one target card".into(),
                ));
            }
            let card = CardId::new(selections[0]);
            if !payload[5..].contains(&card.raw())
                || !tx
                    .state()
                    .skills
                    .get(skill)
                    .is_some_and(|instance| instance.owner == player)
                || !crate::query::discardable_target_cards(
                    tx.state(),
                    player,
                    target,
                    rule_api::TargetCardScope::HandEquip,
                )
                .contains(&card)
            {
                return Err(EffectError::Illegal(
                    "Beizhu target-card choice context changed".into(),
                ));
            }
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveBeizhu {
                    player,
                    target,
                    stage: 1,
                    cards: Vec::new(),
                    skill,
                });
            run_effects(
                tx,
                vec![EffectRequest::DiscardTargetCards {
                    actor: player,
                    target,
                    cards: vec![card],
                    scope: rule_api::TargetCardScope::HandEquip,
                }],
            )
            .map(|_| ())
        }
        Some(1) => {
            let [_, player_raw, target_raw, card_raw, draw_raw, skill_raw] = payload else {
                return Err(EffectError::EngineFault(
                    "bad Beizhu draw-pile Sha payload".into(),
                ));
            };
            let player = PlayerId::new(*player_raw);
            if observer != player {
                return Err(EffectError::Illegal(
                    "only the Beizhu owner may resolve this choice".into(),
                ));
            }
            if cancelled {
                return Ok(());
            }
            if selections != [1] {
                return Err(EffectError::Illegal(
                    "invalid Beizhu draw-pile Sha choice".into(),
                ));
            }
            let target = PlayerId::new(*target_raw);
            let card = CardId::new(*card_raw);
            let draw = engine_api::ZoneId::new(*draw_raw);
            let skill = engine_api::SkillId::new(*skill_raw);
            if ![player, target].into_iter().all(|participant| {
                tx.state()
                    .players
                    .get(participant)
                    .is_some_and(|state| state.alive)
            }) || !tx
                .state()
                .skills
                .get(skill)
                .is_some_and(|instance| instance.owner == player)
                || !tx.state().zones.get(draw).is_some_and(|zone| {
                    zone.kind == ZoneKind::DrawPile
                        && zone.owner.is_none()
                        && zone.cards.contains(&card)
                })
                || !tx
                    .state()
                    .cards
                    .get(card)
                    .is_some_and(|state| state.zone == draw && state.kind.is_sha())
            {
                return Err(EffectError::Illegal(
                    "Beizhu draw-pile Sha choice context changed".into(),
                ));
            }
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::EngineFault("Beizhu target hand is missing".into()))?;
            run_effects(
                tx,
                vec![EffectRequest::GiveCards {
                    cards: vec![card],
                    to: hand,
                    from: draw,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown Beizhu choice stage".into(),
        )),
    }
}

pub(crate) fn vm_trigger_marker(marker: [u32; 3]) -> EffectRequest {
    EffectRequest::EmitRuleEvent {
        code: 100,
        payload: marker.to_vec(),
    }
}

/// Store a VM pause into resolution state and open the corresponding choice.
pub(crate) fn open_vm_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    module_name: &str,
    skill_id: u32,
    ctx: [u32; 4],
    trigger_marker: [u32; 3],
    pause: crate::vm_host::VmPause,
) -> Result<(), EffectError> {
    {
        let r = &mut tx.state_mut().resolution;
        r.vm_cont_frames = pause.frames;
        r.vm_cont_module = module_name.to_string();
        r.vm_cont_pc = pause.pc;
        r.vm_cont_fuel = pause.fuel_remaining;
        r.vm_cont_ctx = ctx;
        r.vm_cont_skill = skill_id;
        r.vm_cont_trigger_marker = trigger_marker;
    }
    crate::effect::open_choice(
        tx,
        observer,
        1,
        1,
        false,
        pause.options,
        engine_api::ChoiceKind::Custom,
        ContinuationKind::VmResume,
        Vec::new(),
    )?;
    Ok(())
}

pub(crate) fn jiedao_give_weapon(
    tx: &mut Transaction,
    actor: PlayerId,
    holder: PlayerId,
) -> Result<(), EffectError> {
    use crate::state::CardKind;
    let equip = tx
        .state()
        .zones
        .find(ZoneKind::Equip, holder)
        .ok_or_else(|| EffectError::Illegal("no equip".into()))?;
    let weapon = tx
        .state()
        .zones
        .get(equip)
        .and_then(|z| {
            z.cards.iter().copied().find(|cid| {
                tx.state()
                    .cards
                    .get(*cid)
                    .map(|c| c.kind == CardKind::Weapon)
                    .unwrap_or(false)
            })
        })
        .ok_or_else(|| EffectError::Illegal("no weapon".into()))?;
    let to_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, actor)
        .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![weapon],
            to: to_hand,
            from: Some(equip),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_multi_card_view_as_response(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.first().unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(1).unwrap_or(&0));
    let kind = payload
        .get(2)
        .and_then(|tag| CardKind::from_tag(*tag as u8))
        .ok_or_else(|| EffectError::EngineFault("bad multi-card response kind".into()))?;
    if cancelled || observer != player || selections.is_empty() {
        return Err(EffectError::Illegal(
            "multi-card response source choice cannot cancel".into(),
        ));
    }
    let rule = crate::command::skill::multi_card_response_rule(tx.state(), player, skill, kind)
        .ok_or_else(|| EffectError::Illegal("multi-card response conversion unavailable".into()))?;
    let cards: Vec<CardId> = selections.iter().copied().map(CardId::new).collect();
    if !crate::command::skill::view_as_sources_match(tx.state(), player, &rule, &cards)
        || cards
            .iter()
            .any(|card| crate::query::card_color_locked_for_card(tx.state(), player, *card))
    {
        return Err(EffectError::Illegal(
            "invalid multi-card response sources".into(),
        ));
    }
    let instance = tx
        .state()
        .skills
        .get(skill)
        .cloned()
        .ok_or_else(|| EffectError::Illegal("unknown response conversion skill".into()))?;
    if rule
        .uses_per_turn
        .is_some_and(|limit| instance.use_count >= u32::from(limit))
    {
        return Err(EffectError::Illegal(
            "view_as skill use limit reached".into(),
        ));
    }
    if rule.cost.as_deref() == Some("potqingyan") {
        run_effects(
            tx,
            vec![EffectRequest::RevealCards {
                cards: cards.clone(),
                to: Vec::new(),
            }],
        )?;
        let round = tx.state().turn.round;
        if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
            if instance.view_as_use_round != round {
                instance.view_as_use_round = round;
                instance.view_as_used_branches = 0;
            }
            instance.view_as_used_branches = instance.view_as_used_branches.saturating_add(1);
            instance.state_blob = cards
                .iter()
                .flat_map(|card| card.raw().to_le_bytes())
                .collect();
        }
    }
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.use_count = instance.use_count.saturating_add(1);
    }
    let bonus = (cards.len() > 1)
        .then_some(rule.multi_card_bonus.as_deref())
        .flatten();
    if rule.cost.as_deref() == Some("potqingyan") {
        complete_view_as_response(tx, payload, &[], bonus)
    } else {
        complete_view_as_response(tx, payload, &cards, bonus)
    }
}

/// Complete a source-free conversion after its cost has resolved.
pub(crate) fn resume_source_free_response(
    tx: &mut Transaction,
    payload: &[u32],
) -> Result<(), EffectError> {
    let skill = engine_api::SkillId::new(*payload.get(1).unwrap_or(&0));
    let source = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .and_then(|skill| skill.take_source_free_physical_card());
    match source {
        Some(source) => complete_view_as_response(tx, payload, &[source], None),
        None => complete_view_as_response(tx, payload, &[], None),
    }
}
