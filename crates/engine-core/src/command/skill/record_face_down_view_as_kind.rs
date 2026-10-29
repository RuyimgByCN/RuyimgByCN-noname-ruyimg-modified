//! Command operations for `validate_source_free_cost` and related helpers.

use super::*;

pub(crate) fn record_face_down_view_as_kind(
    state: &mut crate::state::GameState,
    skill: SkillId,
    kind: CardKind,
) -> Result<(), EffectError> {
    let bit = face_down_view_as_kind_bit(kind)
        .ok_or_else(|| EffectError::Illegal("unsupported face-down declaration".into()))?;
    let token = state
        .turn
        .round
        .saturating_mul(16)
        .saturating_add(state.turn.current.raw());
    let skill = state
        .skills
        .get_mut(skill)
        .ok_or_else(|| EffectError::Illegal("face-down view-as skill disappeared".into()))?;
    if skill.view_as_use_round != token {
        skill.view_as_use_round = token;
        skill.view_as_used_branches = 0;
    }
    if skill.view_as_used_branches & bit != 0 {
        return Err(EffectError::Illegal(
            "declared card name already used this turn".into(),
        ));
    }
    skill.view_as_used_branches |= bit;
    Ok(())
}

pub(crate) fn use_materialized_virtual_card(
    tx: &mut Transaction,
    actor: PlayerId,
    skill: SkillId,
    inst: &crate::state::SkillInstance,
    kind: CardKind,
    targets: &[PlayerId],
) -> Result<(), EffectError> {
    let post_draw = source_free_post_draw(tx.state(), skill, kind);
    // The use count is recorded before materialization; inspect the already-authorized rule
    // without re-running availability checks that would reject that same use.
    let rule = tx
        .state()
        .content
        .view_as_defs_for_skill(inst.def)
        .into_iter()
        .find(|rule| {
            rule.source_zone.as_deref() == Some("none")
                && matches!(rule.window.as_str(), "any" | "play_phase")
                && source_free_family_contains(&rule.as_kind, kind)
        });
    let ignore_use_count = rule
        .as_ref()
        .is_some_and(|rule| rule.cost.as_deref() == Some("lose_max_hp_1"));
    let qice_conversion = rule
        .as_ref()
        .is_some_and(|rule| rule.cost.as_deref() == Some("all_hand"));
    let qice_materials = qice_conversion
        .then(|| tx.state().hand_cards(actor))
        .unwrap_or_default();
    let source = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .and_then(|skill| skill.take_source_free_physical_card());
    if let Some(card) = source {
        let fusi_provider = dddfusi_approved_source(tx.state(), actor, card);
        if !tx.state().cards.get(card).is_some_and(|source| {
            source.owner == actor
                && tx.state().zones.get(source.zone).is_some_and(|zone| {
                    zone.owner == actor && matches!(zone.kind, ZoneKind::Hand | ZoneKind::Equip)
                })
        }) && !fusi_provider.is_some_and(|provider| {
            tx.state().cards.get(card).is_some_and(|source| {
                source.owner == provider
                    && source.face_up
                    && tx
                        .state()
                        .zones
                        .get(source.zone)
                        .is_some_and(|zone| zone.owner == provider && zone.kind == ZoneKind::Hand)
            })
        }) {
            return Err(EffectError::Illegal(
                "view-as source left the owner's hand/equipment".into(),
            ));
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
        let result = (|| {
            run_effects(
                tx,
                vec![EffectRequest::EmitRuleEvent {
                    code: 120,
                    payload: vec![
                        skill.raw(),
                        inst.def.raw(),
                        actor.raw(),
                        card.raw(),
                        kind.tag() as u32,
                    ],
                }],
            )?;
            if qice_conversion {
                let extras: Vec<_> = qice_materials
                    .iter()
                    .copied()
                    .filter(|material| *material != card)
                    .collect();
                play_as_kind_with_extra_damage_flags(
                    tx,
                    actor,
                    card,
                    kind,
                    targets,
                    false,
                    false,
                    &[],
                    ignore_use_count,
                    true,
                    0,
                    &extras,
                    0,
                )
            } else {
                play_as_kind(
                    tx,
                    actor,
                    card,
                    kind,
                    targets,
                    false,
                    false,
                    ignore_use_count,
                    true,
                )
            }
        })();
        tx.state_mut()
            .turn
            .set_flag(crate::state::turn_flag::VIEW_AS_CARD, 0);
        tx.state_mut()
            .turn
            .set_flag(crate::state::turn_flag::VIEW_AS_SKILL, 0);
        tx.state_mut()
            .turn
            .set_flag(crate::state::turn_flag::VIEW_AS_LOGICAL_KIND, 0);
        let fusi_tail = fusi_provider.map(|provider| {
            let skill = tx
                .state()
                .players
                .get(provider)
                .and_then(|participant| {
                    participant.skills.iter().copied().find(|owned| {
                        tx.state()
                            .skills
                            .get(*owned)
                            .is_some_and(|instance| instance.def.raw() == DDD_FUSI_DEF)
                    })
                })
                .unwrap_or(SkillId::NONE);
            EffectRequest::ResolveDddXianglang {
                player: provider,
                actor,
                action: 2,
                skill,
            }
        });
        return match result {
            Ok(()) => {
                let mut tail: Vec<_> = fusi_tail.into_iter().collect();
                if post_draw > 0 {
                    tail.push(EffectRequest::DrawCards {
                        player: actor,
                        count: u32::from(post_draw),
                    });
                }
                run_effects(tx, tail).map(|_| ())
            }
            Err(EffectError::ChoiceRequested) if fusi_tail.is_some() || post_draw > 0 => {
                if let Some(effect) = fusi_tail {
                    tx.state_mut().resolution.deferred_tail_effects.push(effect);
                }
                if post_draw > 0 {
                    tx.state_mut().resolution.deferred_tail_effects.push(
                        EffectRequest::DrawCards {
                            player: actor,
                            count: u32::from(post_draw),
                        },
                    );
                }
                Err(EffectError::ChoiceRequested)
            }
            other => other,
        };
    }
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, actor)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::NONE,
        kind,
        suit: engine_api::Suit::None,
        rank: 0,
        weapon_range: 0,
        zone: processing,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(processing)
        .unwrap()
        .cards
        .push(card);
    crate::effect::move_one_card(tx, card, processing, hand, actor, false)?;
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
    let result = (|| {
        run_effects(
            tx,
            vec![EffectRequest::EmitRuleEvent {
                code: 120,
                payload: vec![
                    skill.raw(),
                    inst.def.raw(),
                    actor.raw(),
                    card.raw(),
                    kind.tag() as u32,
                ],
            }],
        )?;
        play_as_kind(
            tx,
            actor,
            card,
            kind,
            targets,
            false,
            false,
            ignore_use_count,
            false,
        )
    })();
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

pub(crate) fn validate_active_targets(
    tx: &Transaction,
    actor: PlayerId,
    targets: &[PlayerId],
    min: u8,
    max: u8,
    allow_self: bool,
    lowest_hp: bool,
    highest_hp: bool,
    unlinked: bool,
    linked: bool,
    wounded: bool,
    male: bool,
    hp_at_least_owner: bool,
    hp_gt_owner: bool,
    hp_lte_owner: bool,
    hp_gt: Option<u8>,
    max_hp_gt: Option<u8>,
    hand_gt_owner: bool,
    hand_lte_owner: bool,
    pair_hand_difference_lte_three: bool,
    hand_lt: Option<u8>,
    hand_nonempty: bool,
    can_compare: bool,
    can_use_on_owner: Option<&str>,
    second_can_use_juedou_on_first: bool,
    has_equip: bool,
    has_discardable_equip: bool,
    has_gainable_equip: bool,
    has_gainable_hand_equip: bool,
    in_attack_range: bool,
    owner_in_target_attack_range: bool,
    hand_equip_nonempty: bool,
    has_discardable_hand_equip: bool,
    equipment_pair_with_armor: bool,
    excluded_targets: &[PlayerId],
) -> Result<(), EffectError> {
    if targets.len() < usize::from(min) || targets.len() > usize::from(max) {
        return Err(EffectError::Illegal(
            "active target count out of range".into(),
        ));
    }
    if pair_hand_difference_lte_three {
        let [left, right] = targets else {
            return Err(EffectError::Illegal(
                "hand-difference constraint requires exactly two targets".into(),
            ));
        };
        if tx
            .state()
            .hand_count(*left)
            .abs_diff(tx.state().hand_count(*right))
            > 3
        {
            return Err(EffectError::Illegal(
                "active targets' hand counts differ by more than three".into(),
            ));
        }
    }
    let mut seen = std::collections::BTreeSet::new();
    let minimum_hp = lowest_hp
        .then(|| tx.state().players.iter_live().map(|player| player.hp).min())
        .flatten();
    let maximum_hp = highest_hp
        .then(|| tx.state().players.iter_live().map(|player| player.hp).max())
        .flatten();
    let owner_hp = tx
        .state()
        .players
        .get(actor)
        .map(|player| player.hp)
        .unwrap_or(i32::MAX);
    if can_compare && tx.state().hand_cards(actor).is_empty() {
        return Err(EffectError::Illegal(
            "active skill owner has no hand card to compare".into(),
        ));
    }
    for target in targets {
        if !allow_self && *target == actor {
            return Err(EffectError::Illegal(
                "active skill cannot target its owner".into(),
            ));
        }
        if target.is_none() || !seen.insert(target.raw()) {
            return Err(EffectError::Illegal(
                "active targets must be distinct players".into(),
            ));
        }
        if excluded_targets.contains(target) {
            return Err(EffectError::Illegal(
                "active target was already used by this skill this turn".into(),
            ));
        }
        if !tx
            .state()
            .players
            .get(*target)
            .is_some_and(|player| player.alive)
        {
            return Err(EffectError::Illegal("active target is not alive".into()));
        }
        if minimum_hp.is_some_and(|minimum| tx.state().players.get(*target).unwrap().hp != minimum)
        {
            return Err(EffectError::Illegal(
                "active target does not have the lowest hp".into(),
            ));
        }
        if maximum_hp.is_some_and(|maximum| tx.state().players.get(*target).unwrap().hp != maximum)
        {
            return Err(EffectError::Illegal(
                "active target does not have the highest hp".into(),
            ));
        }
        if unlinked
            && tx
                .state()
                .players
                .get(*target)
                .unwrap()
                .marks
                .iter()
                .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
        {
            return Err(EffectError::Illegal(
                "active target must be unlinked".into(),
            ));
        }
        if linked
            && !tx
                .state()
                .players
                .get(*target)
                .unwrap()
                .marks
                .iter()
                .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
        {
            return Err(EffectError::Illegal("active target must be linked".into()));
        }
        if wounded && {
            let player = tx.state().players.get(*target).unwrap();
            player.hp >= player.max_hp as i32
        } {
            return Err(EffectError::Illegal("active target must be wounded".into()));
        }
        if male
            && !tx
                .state()
                .players
                .get(*target)
                .and_then(|player| tx.state().content.character(player.character_def))
                .is_some_and(|character| matches!(character.sex.as_str(), "male" | "double"))
        {
            return Err(EffectError::Illegal(
                "active target must be male or double-sex".into(),
            ));
        }
        if hp_at_least_owner && tx.state().players.get(*target).unwrap().hp < owner_hp {
            return Err(EffectError::Illegal(
                "active target hp is below skill owner hp".into(),
            ));
        }
        if hp_gt_owner && tx.state().players.get(*target).unwrap().hp <= owner_hp {
            return Err(EffectError::Illegal(
                "active target hp must exceed skill owner hp".into(),
            ));
        }
        if hp_lte_owner && tx.state().players.get(*target).unwrap().hp > owner_hp {
            return Err(EffectError::Illegal(
                "active target hp must not exceed skill owner hp".into(),
            ));
        }
        if hp_gt
            .is_some_and(|limit| tx.state().players.get(*target).unwrap().hp <= i32::from(limit))
        {
            return Err(EffectError::Illegal(
                "active target hp must exceed the configured threshold".into(),
            ));
        }
        if max_hp_gt
            .is_some_and(|limit| tx.state().players.get(*target).unwrap().max_hp <= u32::from(limit))
        {
            return Err(EffectError::Illegal(
                "active target max hp must exceed the configured threshold".into(),
            ));
        }
        if hand_gt_owner && tx.state().hand_count(*target) <= tx.state().hand_count(actor) {
            return Err(EffectError::Illegal(
                "active target hand count must exceed skill owner hand count".into(),
            ));
        }
        if hand_lte_owner && tx.state().hand_count(*target) > tx.state().hand_count(actor) {
            return Err(EffectError::Illegal(
                "active target hand count must not exceed skill owner hand count".into(),
            ));
        }
        if hand_lt.is_some_and(|limit| tx.state().hand_count(*target) >= u32::from(limit)) {
            return Err(EffectError::Illegal(
                "active target hand count must be below the configured limit".into(),
            ));
        }
        if hand_nonempty && tx.state().hand_count(*target) == 0 {
            return Err(EffectError::Illegal(
                "active target must have at least one hand card".into(),
            ));
        }
        if can_compare && tx.state().hand_cards(*target).is_empty() {
            return Err(EffectError::Illegal(
                "active target has no hand card to compare".into(),
            ));
        }
        if has_equip
            && tx
                .state()
                .zones
                .find(ZoneKind::Equip, *target)
                .and_then(|zone| tx.state().zones.get(zone))
                .is_none_or(|zone| zone.cards.is_empty())
        {
            return Err(EffectError::Illegal(
                "active target has no equipment".into(),
            ));
        }
        if has_discardable_equip
            && crate::query::discardable_equipment_cards(tx.state(), actor, *target).is_empty()
        {
            return Err(EffectError::Illegal(
                "active target has no discardable equipment".into(),
            ));
        }
        if has_gainable_equip
            && crate::query::gainable_target_cards(
                tx.state(),
                actor,
                *target,
                rule_api::TargetCardScope::Equip,
            )
            .is_empty()
        {
            return Err(EffectError::Illegal(
                "active target has no gainable equipment".into(),
            ));
        }
        if has_gainable_hand_equip
            && crate::query::gainable_target_cards(
                tx.state(),
                actor,
                *target,
                rule_api::TargetCardScope::HandEquip,
            )
            .is_empty()
        {
            return Err(EffectError::Illegal(
                "active target has no gainable hand/equipment card".into(),
            ));
        }
        if in_attack_range
            && *target != actor
            && !crate::query::player_in_attack_range(tx.state(), actor, *target)
        {
            return Err(EffectError::Illegal(
                "active target is outside the skill owner's attack range".into(),
            ));
        }
        if owner_in_target_attack_range
            && *target != actor
            && !crate::query::player_in_attack_range(tx.state(), *target, actor)
        {
            return Err(EffectError::Illegal(
                "skill owner is outside the target's attack range".into(),
            ));
        }
        if hand_equip_nonempty
            && crate::query::discardable_hand_equip_cards(tx.state(), *target).is_empty()
        {
            return Err(EffectError::Illegal(
                "active target has no hand/equipment card".into(),
            ));
        }
        if has_discardable_hand_equip
            && crate::query::discardable_target_cards(
                tx.state(),
                actor,
                *target,
                rule_api::TargetCardScope::HandEquip,
            )
            .is_empty()
        {
            return Err(EffectError::Illegal(
                "active target has no discardable hand/equipment card".into(),
            ));
        }
        if let Some(kind) = can_use_on_owner {
            let kind = map_as_kind(kind)
                .ok_or_else(|| EffectError::ContentFault("bad active target card kind".into()))?;
            super::super::play::validate_source_free_card_on_target(tx, *target, kind, actor)?;
        }
    }
    if equipment_pair_with_armor
        && !crate::query::equipment_pair_with_armor_legal(tx.state(), actor, targets)
    {
        return Err(EffectError::Illegal(
            "active targets require discardable equipment and armor".into(),
        ));
    }
    if second_can_use_juedou_on_first {
        let [target, user] = targets else {
            return Err(EffectError::ContentFault(
                "second_can_use_juedou_on_first requires exactly two targets".into(),
            ));
        };
        super::super::play::validate_source_free_card_on_target(
            tx,
            *user,
            CardKind::TrickJuedou,
            *target,
        )?;
    }
    Ok(())
}

pub(crate) fn validate_active_window(
    tx: &Transaction,
    actor: PlayerId,
    window: &str,
) -> Result<(), EffectError> {
    match window {
        "play_phase" => {
            if tx.state().turn.current != actor {
                return Err(EffectError::Illegal("not current player".into()));
            }
            if tx.state().turn.phase != Phase::Play {
                return Err(EffectError::Illegal("not play phase".into()));
            }
        }
        "any" => {}
        other => {
            return Err(EffectError::Illegal(format!("bad window {other}")));
        }
    }
    Ok(())
}

pub(crate) fn active_selection_cards(
    tx: &Transaction,
    actor: PlayerId,
    zone: &str,
    filter: Option<&str>,
    excluded_category_mask: u8,
) -> Result<Vec<CardId>, EffectError> {
    let cards = match zone {
        "hand" => Ok(tx.state().hand_cards(actor)),
        "hand_equip" => Ok(crate::query::discardable_hand_equip_cards(
            tx.state(),
            actor,
        )),
        other => Err(EffectError::ContentFault(format!(
            "unsupported active selection zone {other}"
        ))),
    }?;
    let cards = match filter {
        None => Ok(cards),
        Some("card_equip") => Ok(cards
            .into_iter()
            .filter(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|card| card.kind.is_equip())
            })
            .collect()),
        Some("card_armor") => Ok(cards
            .into_iter()
            .filter(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|card| card.kind == CardKind::Armor)
            })
            .collect()),
        Some("card_sha") => Ok(cards
            .into_iter()
            .filter(|card| {
                tx.state().cards.get(*card).is_some_and(|card| {
                    crate::query::effective_card_kind(tx.state(), actor, card).is_sha()
                })
            })
            .collect()),
        Some("card_basic") => Ok(cards
            .into_iter()
            .filter(|card| {
                tx.state().cards.get(*card).is_some_and(|card| {
                    crate::query::effective_card_kind(tx.state(), actor, card).is_basic()
                })
            })
            .collect()),
        Some("card_nonbasic") => Ok(cards
            .into_iter()
            .filter(|card| {
                tx.state().cards.get(*card).is_some_and(|card| {
                    !crate::query::effective_card_kind(tx.state(), actor, card).is_basic()
                })
            })
            .collect()),
        Some("card_non_equip") => Ok(cards
            .into_iter()
            .filter(|card| {
                tx.state().cards.get(*card).is_some_and(|card| {
                    !crate::query::effective_card_kind(tx.state(), actor, card).is_equip()
                })
            })
            .collect()),
        Some("card_red") => Ok(cards
            .into_iter()
            .filter(|card| active_card_is_red(tx, actor, *card))
            .collect()),
        Some("card_black") => Ok(cards
            .into_iter()
            .filter(|card| !active_card_is_red(tx, actor, *card))
            .collect()),
        Some("card_heart") => Ok(cards
            .into_iter()
            .filter(|card| {
                tx.state().cards.get(*card).is_some_and(|card| {
                    crate::query::effective_card_suit(tx.state(), actor, card)
                        == engine_api::Suit::Heart
                })
            })
            .collect()),
        Some(other) => Err(EffectError::ContentFault(format!(
            "unsupported active card filter {other}"
        ))),
    }?;
    Ok(cards
        .into_iter()
        .filter(|card| active_card_category_bit(tx, actor, *card) & excluded_category_mask == 0)
        .collect())
}

pub(crate) fn validate_selected_cards(
    tx: &Transaction,
    actor: PlayerId,
    cards: &[CardId],
    selection: &content_schema::PackageSelectionSpec,
    excluded_category_mask: u8,
) -> Result<(), EffectError> {
    if cards.len() < usize::from(selection.min) || cards.len() > usize::from(selection.max) {
        return Err(EffectError::Illegal(
            "active card count out of range".into(),
        ));
    }
    let allowed: std::collections::BTreeSet<_> = active_selection_cards(
        tx,
        actor,
        &selection.zone,
        selection.filter.as_deref(),
        excluded_category_mask,
    )?
    .into_iter()
    .map(CardId::raw)
    .collect();
    let mut seen = std::collections::BTreeSet::new();
    for card in cards {
        if !seen.insert(card.raw()) {
            return Err(EffectError::Illegal("duplicate active card".into()));
        }
        let card_state = tx
            .state()
            .cards
            .get(*card)
            .ok_or_else(|| EffectError::Illegal("bad active card".into()))?;
        if card_state.owner != actor || !allowed.contains(&card.raw()) {
            return Err(EffectError::Illegal(
                "active card is outside the declared selection zone".into(),
            ));
        }
    }
    if selection.same_color {
        let first = active_card_is_red(tx, actor, cards[0]);
        if cards
            .iter()
            .skip(1)
            .any(|card| active_card_is_red(tx, actor, *card) != first)
        {
            return Err(EffectError::Illegal(
                "active cards must have the same color".into(),
            ));
        }
    }
    if selection.same_suit {
        let first = cards
            .first()
            .and_then(|card| tx.state().cards.get(*card))
            .map(|card| crate::query::effective_card_suit(tx.state(), actor, card));
        if first.is_none_or(|suit| suit == engine_api::Suit::None)
            || cards.iter().skip(1).any(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .map(|card| crate::query::effective_card_suit(tx.state(), actor, card))
                    != first
            })
        {
            return Err(EffectError::Illegal(
                "active cards must have the same effective suit".into(),
            ));
        }
    }
    if selection.distinct_categories {
        let mut categories = 0u8;
        for card in cards {
            let category = active_card_category_bit(tx, actor, *card);
            if category == 0 || categories & category != 0 {
                return Err(EffectError::Illegal(
                    "active cards must have distinct categories".into(),
                ));
            }
            categories |= category;
        }
    }
    if selection.distinct_suits {
        let mut suits = 0u8;
        for card in cards {
            let suit = tx
                .state()
                .cards
                .get(*card)
                .map(|card| crate::query::effective_card_suit(tx.state(), actor, card));
            let bit = match suit {
                Some(engine_api::Suit::Spade) => 1,
                Some(engine_api::Suit::Heart) => 2,
                Some(engine_api::Suit::Club) => 4,
                Some(engine_api::Suit::Diamond) => 8,
                _ => 0,
            };
            if bit == 0 || suits & bit != 0 {
                return Err(EffectError::Illegal(
                    "active cards must have distinct effective suits".into(),
                ));
            }
            suits |= bit;
        }
    }
    if let Some(required) = selection.sum_effective_number_exact {
        let total = cards.iter().try_fold(0u16, |total, card| {
            tx.state().cards.get(*card).map(|state_card| {
                total.saturating_add(u16::from(crate::query::effective_card_number(
                    tx.state(),
                    actor,
                    state_card,
                )))
            })
        });
        if total != Some(required) {
            return Err(EffectError::Illegal(
                "active cards must match the required effective-number sum".into(),
            ));
        }
    }
    Ok(())
}

pub(crate) fn active_card_category_bit(tx: &Transaction, actor: PlayerId, card: CardId) -> u8 {
    tx.state()
        .cards
        .get(card)
        .map(|card| crate::query::effective_card_kind(tx.state(), actor, card))
        .map(|kind| {
            if kind.is_basic() {
                1
            } else if kind.is_trick() {
                2
            } else if kind.is_equip() {
                4
            } else {
                0
            }
        })
        .unwrap_or(0)
}

pub(crate) fn active_card_is_red(tx: &Transaction, actor: PlayerId, card: CardId) -> bool {
    let card = tx
        .state()
        .cards
        .get(card)
        .expect("active card existence validated before color check");
    matches!(
        crate::query::effective_card_suit(tx.state(), actor, card),
        engine_api::Suit::Heart | engine_api::Suit::Diamond
    )
}
