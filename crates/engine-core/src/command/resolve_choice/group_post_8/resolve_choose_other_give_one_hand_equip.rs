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
