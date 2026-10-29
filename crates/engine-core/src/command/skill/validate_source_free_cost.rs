//! Command operations for `validate_source_free_cost` and related helpers.

use super::*;

pub(crate) fn validate_source_free_cost(
    tx: &Transaction,
    actor: PlayerId,
    rule: &ViewAsDef,
    equip_slot: u8,
) -> Result<Option<EquipSlot>, EffectError> {
    match rule.cost.as_deref() {
        Some("turn_over")
            if equip_slot == 0
                && tx.state().players.get(actor).is_some_and(|player| {
                    player.alive
                        && !player.turned_over
                        && !crate::query::turn_over_cancelled(tx.state(), actor)
                        && !crate::query::turn_face_down_cancelled(tx.state(), actor)
                }) =>
        {
            Ok(None)
        }
        Some("face_down_hand")
            if equip_slot == 0
                && face_down_view_as_kind_available_for_skill(
                    tx.state(),
                    actor,
                    rule,
                    CardKind::Other,
                ) =>
        {
            Ok(None)
        }
        Some("dczhaowen")
            if equip_slot == 0
                && dczhaowen_view_as_kind_available_for_skill(
                    tx.state(),
                    actor,
                    rule,
                    CardKind::Other,
                ) =>
        {
            Ok(None)
        }
        Some("guhuo") if equip_slot == 0 && !tx.state().hand_cards(actor).is_empty() => Ok(None),
        Some("gain_max_hp_1") if equip_slot == 0 => Ok(None),
        Some("swap_lower_hand")
            if equip_slot == 0 && !swap_lower_hand_targets(tx.state(), actor).is_empty() =>
        {
            Ok(None)
        }
        Some("draw_one_outside_turn_empty_hand")
            if equip_slot == 0
                && tx.state().turn.current != actor
                && tx.state().hand_cards(actor).is_empty() =>
        {
            Ok(None)
        }
        Some("hand_equip_mark")
            if equip_slot == 0
                && !hand_equip_view_as_sources(tx.state(), actor).is_empty()
                && rule.mark_cost.is_some_and(|mark| {
                    tx.state().players.get(actor).is_some_and(|player| {
                        player
                            .marks
                            .iter()
                            .any(|(existing, count)| *existing == mark && *count > 0)
                    })
                }) =>
        {
            Ok(None)
        }
        Some("lose_max_hp_1")
            if equip_slot == 0
                && tx
                    .state()
                    .players
                    .get(actor)
                    .is_some_and(|player| player.alive && player.max_hp > 0) =>
        {
            Ok(None)
        }
        Some("all_hand") if equip_slot == 0 && !tx.state().hand_cards(actor).is_empty() => Ok(None),
        Some("dddshilie")
            if equip_slot == 0
                && !dddshilie_sources(tx.state(), actor).is_empty()
                && dddshilie_threshold(tx.state(), actor).is_some_and(|threshold| {
                    dddshilie_sources(tx.state(), actor)
                        .iter()
                        .filter_map(|card| tx.state().cards.get(*card))
                        .map(|card| {
                            u16::from(crate::query::effective_card_number(tx.state(), actor, card))
                        })
                        .sum::<u16>()
                        >= threshold
                }) =>
        {
            Ok(None)
        }
        Some("dddfusi") if equip_slot == 0 => Ok(None),
        Some("potdaozhuan")
            if equip_slot == 0 && !potdaozhuan_cost_cards(tx.state(), actor).is_empty() =>
        {
            Ok(None)
        }
        Some("disable_equip_slot") => {
            let slot = EquipSlot::from_tag(equip_slot)
                .ok_or_else(|| EffectError::Illegal("invalid equipment slot cost".into()))?;
            if !tx
                .state()
                .players
                .get(actor)
                .is_some_and(|player| player.equip_slot_enabled(slot))
            {
                return Err(EffectError::Illegal(
                    "equipment slot already disabled".into(),
                ));
            }
            Ok(Some(slot))
        }
        Some("gain_max_hp_1") => Err(EffectError::Illegal(
            "max-hp conversion does not select an equipment slot".into(),
        )),
        _ => Err(EffectError::ContentFault(
            "unsupported source-free conversion cost".into(),
        )),
    }
}

pub(crate) fn record_source_free_use(
    tx: &mut Transaction,
    skill: SkillId,
    rule: &ViewAsDef,
) -> Result<(), EffectError> {
    if matches!(
        rule.cost.as_deref(),
        Some("swap_lower_hand" | "hand_equip_mark" | "lose_max_hp_1" | "all_hand" | "dddshilie")
    ) {
        let inst = tx
            .state_mut()
            .skills
            .get_mut(skill)
            .ok_or_else(|| EffectError::Illegal("unknown skill".into()))?;
        inst.use_count = inst.use_count.saturating_add(1);
        return Ok(());
    }
    if matches!(
        rule.cost.as_deref(),
        Some(
            "guhuo"
                | "face_down_hand"
                | "dczhaowen"
                | "draw_one_outside_turn_empty_hand"
                | "dddfusi"
                | "turn_over"
        )
    ) {
        return Ok(());
    }
    if rule.cost.as_deref() == Some("potdaozhuan") {
        return Ok(());
    }
    let branch = rule
        .branch
        .ok_or_else(|| EffectError::ContentFault("missing conversion branch".into()))?;
    let next = rule
        .next_branch
        .ok_or_else(|| EffectError::ContentFault("missing next conversion branch".into()))?;
    let round = tx.state().turn.round;
    let inst = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .ok_or_else(|| EffectError::Illegal("unknown skill".into()))?;
    if inst.view_as_use_round != round {
        inst.view_as_use_round = round;
        inst.view_as_used_branches = 0;
    }
    inst.view_as_used_branches |= 1u32 << branch;
    inst.view_as_branch = next;
    Ok(())
}

pub(crate) fn pay_source_free_cost(
    tx: &mut Transaction,
    actor: PlayerId,
    skill: SkillId,
    kind: CardKind,
    rule: &ViewAsDef,
    slot: Option<EquipSlot>,
) -> Result<(), EffectError> {
    match rule.cost.as_deref() {
        Some("turn_over") => {
            crate::effect::run_effects(tx, vec![EffectRequest::TurnOver { player: actor }])?;
            if !tx
                .state()
                .players
                .get(actor)
                .is_some_and(|player| player.turned_over)
            {
                return Err(EffectError::Illegal("酒诗翻面代价未完成".into()));
            }
            Ok(())
        }
        Some("face_down_hand") => {
            let options = face_down_view_as_sources(tx.state(), actor);
            crate::effect::open_choice(
                tx,
                actor,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::ResolveFaceDownViewAs,
                vec![actor.raw(), skill.raw()],
            )?;
            Err(EffectError::ChoiceRequested)
        }
        Some("dczhaowen") => {
            let options = dczhaowen_view_as_sources_for_skill(tx.state(), actor, rule);
            crate::effect::open_choice(
                tx,
                actor,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::ResolveFaceDownViewAs,
                vec![actor.raw(), skill.raw(), 3],
            )?;
            Err(EffectError::ChoiceRequested)
        }
        Some("guhuo") => {
            let options = tx
                .state()
                .hand_cards(actor)
                .into_iter()
                .map(CardId::raw)
                .collect();
            crate::effect::open_choice(
                tx,
                actor,
                1,
                1,
                false,
                options,
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::ResolveGuhuo,
                vec![0, actor.raw(), skill.raw()],
            )?;
            Err(EffectError::ChoiceRequested)
        }
        Some("hand_equip_mark") => {
            let mark = rule
                .mark_cost
                .ok_or_else(|| EffectError::ContentFault("missing source-free mark cost".into()))?;
            run_effects(
                tx,
                vec![EffectRequest::RemoveMark {
                    player: actor,
                    mark: MarkDefId::new(mark),
                    amount: 1,
                }],
            )?;
            crate::effect::open_choice(
                tx,
                actor,
                1,
                1,
                false,
                hand_equip_view_as_sources(tx.state(), actor)
                    .into_iter()
                    .map(CardId::raw)
                    .collect(),
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::ResolveFaceDownViewAs,
                vec![actor.raw(), skill.raw(), 1],
            )?;
            Err(EffectError::ChoiceRequested)
        }
        Some("swap_lower_hand") => {
            let options = swap_lower_hand_targets(tx.state(), actor);
            crate::effect::open_choice(
                tx,
                actor,
                1,
                1,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                engine_api::ChoiceKind::SelectTargets,
                crate::state::ContinuationKind::ResolveSwapLowerHandViewAs,
                vec![actor.raw(), skill.raw()],
            )?;
            Err(EffectError::ChoiceRequested)
        }
        Some("potdaozhuan") => {
            crate::effect::open_choice(
                tx,
                actor,
                1,
                1,
                false,
                potdaozhuan_cost_cards(tx.state(), actor)
                    .into_iter()
                    .map(CardId::raw)
                    .collect(),
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::ResolveFaceDownViewAs,
                vec![actor.raw(), skill.raw(), 2, kind.tag() as u32],
            )?;
            Err(EffectError::ChoiceRequested)
        }
        Some("dddshilie") => {
            let options = dddshilie_sources(tx.state(), actor);
            crate::effect::open_choice(
                tx,
                actor,
                1,
                options.len().min(u8::MAX as usize) as u8,
                false,
                options.into_iter().map(CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::ResolveFaceDownViewAs,
                vec![actor.raw(), skill.raw(), 4],
            )?;
            Err(EffectError::ChoiceRequested)
        }
        Some("dddfusi") => {
            let options = dddfusi_sources(tx.state(), actor, skill, kind);
            crate::effect::open_choice(
                tx,
                actor,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::ResolveFaceDownViewAs,
                vec![actor.raw(), skill.raw(), 5, kind.tag() as u32],
            )?;
            Err(EffectError::ChoiceRequested)
        }
        Some("gain_max_hp_1") => run_effects(
            tx,
            vec![EffectRequest::ChangeMaxHp {
                player: actor,
                delta: 1,
            }],
        )
        .map(|_| ()),
        Some("draw_one_outside_turn_empty_hand") => run_effects(
            tx,
            vec![EffectRequest::DrawCards {
                player: actor,
                count: 1,
            }],
        )
        .map(|_| ()),
        Some("lose_max_hp_1") => run_effects(
            tx,
            vec![EffectRequest::ChangeMaxHp {
                player: actor,
                delta: -1,
            }],
        )
        .map(|_| ()),
        Some("all_hand") => {
            let cards = tx.state().hand_cards(actor);
            let (&source, _) = cards.split_first().ok_or_else(|| {
                EffectError::Illegal("all-hand conversion requires hand cards".into())
            })?;
            tx.state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::Illegal("unknown skill".into()))?
                .set_source_free_physical_card(source);
            Ok(())
        }
        Some("disable_equip_slot") => {
            let slot = slot.ok_or_else(|| {
                EffectError::ContentFault("missing equipment slot conversion cost".into())
            })?;
            run_effects(
                tx,
                vec![EffectRequest::DisableEquipSlots {
                    player: actor,
                    slots: vec![slot as u8],
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::ContentFault(
            "unsupported source-free conversion cost".into(),
        )),
    }
}

pub(crate) fn dddshilie_sources(state: &crate::state::GameState, player: PlayerId) -> Vec<CardId> {
    state
        .hand_cards(player)
        .into_iter()
        .filter(|card| state.cards.get(*card).is_some_and(|card| !card.face_up))
        .collect()
}

pub(crate) fn dddshilie_threshold(
    state: &crate::state::GameState,
    player: PlayerId,
) -> Option<u16> {
    if state.turn.phase == Phase::Idle {
        return None;
    }
    let owner = state.players.get(player)?;
    let current = state.players.get(state.turn.current)?;
    (owner.alive && current.alive).then(|| {
        u16::try_from(owner.hp.max(0))
            .unwrap_or(u16::MAX)
            .saturating_add(u16::try_from(current.hp.max(0)).unwrap_or(u16::MAX))
    })
}

pub(crate) fn swap_lower_hand_targets(
    state: &crate::state::GameState,
    player: PlayerId,
) -> Vec<PlayerId> {
    let hand = state.hand_count(player);
    state
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| *target != player && state.hand_count(*target) < hand)
        .collect()
}

pub(crate) fn face_down_view_as_sources(
    state: &crate::state::GameState,
    player: PlayerId,
) -> Vec<CardId> {
    state
        .hand_cards(player)
        .into_iter()
        .filter(|card| state.face_down_hand_cards.contains(card))
        .collect()
}

pub(crate) fn dczhaowen_view_as_sources(
    state: &crate::state::GameState,
    skill: &crate::state::SkillInstance,
) -> Vec<CardId> {
    let token = state
        .turn
        .round
        .saturating_mul(16)
        .saturating_add(state.turn.current.raw());
    if skill.owner != state.turn.current || skill.view_as_use_round != token {
        return Vec::new();
    }
    let tracked = skill.tracked_cards();
    state
        .hand_cards(skill.owner)
        .into_iter()
        .filter(|card| {
            tracked.contains(card)
                && state.cards.get(*card).is_some_and(|card| {
                    matches!(
                        crate::query::effective_card_suit(state, skill.owner, card),
                        engine_api::Suit::Spade | engine_api::Suit::Club
                    )
                })
        })
        .collect()
}

pub(crate) fn dczhaowen_view_as_sources_for_skill(
    state: &crate::state::GameState,
    player: PlayerId,
    rule: &ViewAsDef,
) -> Vec<CardId> {
    state
        .players
        .get(player)
        .into_iter()
        .flat_map(|player| player.skills.iter())
        .filter_map(|skill| state.skills.get(*skill))
        .find(|skill| {
            state
                .content
                .view_as_defs_for_skill(skill.def)
                .iter()
                .any(|candidate| candidate == rule)
        })
        .map_or_else(Vec::new, |skill| dczhaowen_view_as_sources(state, skill))
}

pub(crate) fn dczhaowen_view_as_kind_available_for_skill(
    state: &crate::state::GameState,
    player: PlayerId,
    rule: &ViewAsDef,
    kind: CardKind,
) -> bool {
    state
        .players
        .get(player)
        .into_iter()
        .flat_map(|player| player.skills.iter())
        .filter_map(|skill| state.skills.get(*skill))
        .filter(|skill| {
            state
                .content
                .view_as_defs_for_skill(skill.def)
                .iter()
                .any(|candidate| candidate == rule)
        })
        .any(|skill| {
            kind == CardKind::Other || dczhaowen_view_as_kind_available(state, skill, kind)
        })
}

pub(crate) fn dczhaowen_view_as_kind_available(
    state: &crate::state::GameState,
    skill: &crate::state::SkillInstance,
    kind: CardKind,
) -> bool {
    let Some(bit) = face_down_view_as_kind_bit(kind) else {
        return false;
    };
    !dczhaowen_view_as_sources(state, skill).is_empty() && skill.view_as_used_branches & bit == 0
}

pub(crate) fn hand_equip_view_as_sources(
    state: &crate::state::GameState,
    player: PlayerId,
) -> Vec<CardId> {
    crate::query::discardable_hand_equip_cards(state, player)
}

pub(crate) fn face_down_view_as_kind_available_for_skill(
    state: &crate::state::GameState,
    player: PlayerId,
    rule: &ViewAsDef,
    kind: CardKind,
) -> bool {
    state
        .players
        .get(player)
        .into_iter()
        .flat_map(|player| player.skills.iter())
        .filter_map(|skill| state.skills.get(*skill))
        .filter(|skill| {
            state
                .content
                .view_as_defs_for_skill(skill.def)
                .iter()
                .any(|candidate| candidate == rule)
        })
        .any(|skill| {
            kind == CardKind::Other || face_down_view_as_kind_available(state, skill, kind)
        })
}

pub(crate) fn face_down_view_as_kind_available(
    state: &crate::state::GameState,
    skill: &crate::state::SkillInstance,
    kind: CardKind,
) -> bool {
    let Some(bit) = face_down_view_as_kind_bit(kind) else {
        return false;
    };
    let token = state
        .turn
        .round
        .saturating_mul(16)
        .saturating_add(state.turn.current.raw());
    !face_down_view_as_sources(state, skill.owner).is_empty()
        && (skill.view_as_use_round != token || skill.view_as_used_branches & bit == 0)
}

pub(crate) fn face_down_view_as_kind_bit(kind: CardKind) -> Option<u32> {
    let index = match kind {
        CardKind::Sha | CardKind::ShaFire | CardKind::ShaThunder => 0,
        CardKind::Tao => 1,
        CardKind::Shan => 2,
        CardKind::Jiu => 3,
        kind if kind.is_trick() && !kind.is_delayed_trick() => 4 + kind.tag().checked_sub(20)?,
        _ => return None,
    };
    (index < 32).then(|| 1u32 << index)
}
