//! Trigger predicates for `Always` and related requests.

use super::*;

pub(super) fn filter_match(
    state: &crate::state::GameState,
    active: &ActiveTrigger,
    ctx: &TriggerContext,
    _view_as_skill: u32,
    _view_as_card: u32,
    history: Option<&TriggerHistorySnapshot>,
    def: &crate::trigger::TriggerDefinition,
) -> bool {
    use crate::state::ZoneKind;
    let owner_is_player = active.owner == ctx.player;
    let owner_is_source = active.owner == ctx.source;
    let owner_source_distance_le_one = owner_is_source
        && state
            .players
            .get(active.owner)
            .is_some_and(|player| player.alive)
        && matches!(
            query(
                state,
                &QueryRequest::Distance {
                    from: active.owner,
                    to: ctx.player,
                },
                &mut QueryContext::new(),
            ),
            QueryResponse::Int(distance) if distance <= 1
        );
    let source_alive = state
        .players
        .get(ctx.source)
        .map(|player| player.alive)
        .unwrap_or(false);
    let owner_alive = state
        .players
        .get(active.owner)
        .map(|player| player.alive)
        .unwrap_or(false);
    let event_player_alive = state
        .players
        .get(ctx.player)
        .map(|player| player.alive)
        .unwrap_or(false);
    let equip_count = |player_id| {
        state
            .zones
            .find(ZoneKind::Equip, player_id)
            .and_then(|zone| state.zones.get(zone))
            .map_or(0, |zone| zone.cards.len())
    };
    let turn_fires = state
        .skills
        .get(active.skill_id)
        .map(|s| s.turn_fire_count)
        .unwrap_or(0);
    let play_phase_fires = state
        .skills
        .get(active.skill_id)
        .map(|s| s.play_phase_fire_count)
        .unwrap_or(0);
    let card_kind = ctx.card_kind.and_then(CardKind::from_tag);
    let card_is_trick = card_kind.is_some_and(CardKind::is_trick);
    let card_is_ordinary_trick =
        card_kind.is_some_and(|kind| kind.is_trick() && !kind.is_delayed_trick());
    let card_is_sha = matches!(ctx.card_kind, Some(1 | 5 | 6));
    let owner_was_targeted = ctx.targets.contains(&active.owner);
    let suit_owner = if ctx.source.is_some() {
        ctx.source
    } else {
        ctx.player
    };
    let card_suit = ctx
        .card
        .and_then(|card| state.cards.get(card))
        .map(|card| crate::query::effective_card_suit(state, suit_owner, card));
    let card_is_black = matches!(
        card_suit,
        Some(engine_api::Suit::Spade | engine_api::Suit::Club)
    );
    match &def.filter {
        TriggerFilter::Always => true,

        TriggerFilter::OwnerIsEventPlayerOtherHandNonempty => {
            owner_alive
                && owner_is_player
                && matches!(ctx.kind, EventKind::PhasePlay | EventKind::PhaseDraw)
                && (ctx.kind != EventKind::PhaseDraw || !state.turn.draw_phase_is_fixed())
                && state.players.iter_live().any(|player| {
                    player.alive && player.id != active.owner && state.hand_count(player.id) > 0
                })
        }

        TriggerFilter::OwnerIsEventPlayerPositiveDrawOtherHandNonempty => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseDraw
                && !state.turn.draw_phase_is_fixed()
                && crate::query::draw_count_for(state, active.owner).saturating_add(
                    state.turn.flag(turn_flag::CURRENT_DRAW_COUNT_BONUS).max(0) as u32,
                ) > 0
                && state.players.iter_live().any(|player| {
                    player.alive && player.id != active.owner && state.hand_count(player.id) > 0
                })
        }

        TriggerFilter::OwnerSourceFirstTargetBlackCardAnyTargetOtherDiscardableHandEquip => {
            let black = ctx
                .card
                .and_then(|card| state.cards.get(card))
                .map(|card| crate::query::effective_card_suit(state, active.owner, card))
                .is_some_and(|suit| {
                    matches!(suit, engine_api::Suit::Spade | engine_api::Suit::Club)
                });
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardTargeted
                && ctx.targets.first() == Some(&ctx.player)
                && black
                && ctx.targets.iter().any(|target| {
                    *target != active.owner
                        && state
                            .players
                            .get(*target)
                            .is_some_and(|participant| participant.alive)
                        && !crate::query::discardable_target_cards(
                            state,
                            active.owner,
                            *target,
                            rule_api::TargetCardScope::HandEquip,
                        )
                        .is_empty()
                })
        }

        TriggerFilter::OwnerSourceDirectShaDamageTargetArmorOrMountCardGainable => {
            let card_gainable = ctx.card.is_some_and(|card| {
                state
                    .resolution
                    .sha_source_cards(ctx.source, card)
                    .into_iter()
                    .any(|card| {
                        state
                            .cards
                            .get(card)
                            .filter(|card| card.def != engine_api::CardDefId::NONE)
                            .and_then(|card| state.zones.get(card.zone))
                            .is_some_and(|zone| {
                                matches!(zone.kind, ZoneKind::Processing | ZoneKind::DiscardPile)
                            })
                    })
            });
            let target_has_armor_or_mount = state
                .zones
                .find(ZoneKind::Equip, ctx.player)
                .and_then(|zone| state.zones.get(zone))
                .is_some_and(|zone| {
                    zone.cards.iter().any(|card| {
                        state.cards.get(*card).is_some_and(|card| {
                            matches!(
                                card.kind,
                                CardKind::Armor | CardKind::HorseOff | CardKind::HorseDef
                            )
                        })
                    })
                });
            owner_alive
                && owner_is_source
                && event_player_alive
                && ctx.kind == EventKind::Damage
                && card_is_sha
                && ctx.card_flags & rule_api::damage_card_flag::CHAIN_PROPAGATED == 0
                && card_gainable
                && target_has_armor_or_mount
        }

        TriggerFilter::OwnerAliveEventPlayerLostArmorOrMountHasOriginatingSkillPile => {
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.from_zone == Some(ZoneKind::Equip)
                && ctx.cards.iter().any(|card| {
                    state.cards.get(*card).is_some_and(|card| {
                        matches!(
                            card.kind,
                            CardKind::Armor | CardKind::HorseOff | CardKind::HorseDef
                        )
                    })
                })
                && !state
                    .skill_pile_cards(ctx.player, active.skill_def.raw())
                    .is_empty()
        }

        TriggerFilter::OwnerSourcePlayCardMatchesRememberedCategory => {
            let category = card_kind.map(|kind| {
                if kind.is_basic() {
                    1
                } else if kind.is_trick() {
                    2
                } else if kind.is_equip() {
                    4
                } else {
                    0
                }
            });
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardUsed
                && state.turn.current == active.owner
                && state.turn.phase == Phase::Play
                && state
                    .skills
                    .get(active.skill_id)
                    .and_then(|skill| skill.remembered_category())
                    .zip(category)
                    .is_some_and(|(remembered, used)| remembered == used)
        }

        TriggerFilter::OwnerAliveEventPlayerHandNonemptyOncePerRound => {
            owner_alive
                && ctx.kind == EventKind::PhasePlay
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|player| player.alive && state.hand_count(player.id) > 0)
        }

        TriggerFilter::OwnerSourceBoundCategoryFirstThreePlayPhase => {
            let category = card_kind.map(|kind| {
                if kind.is_basic() {
                    1
                } else if kind.is_trick() {
                    2
                } else if kind.is_equip() {
                    4
                } else {
                    0
                }
            });
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && state.turn.phase == Phase::Play
                && state.turn.current == ctx.source
                && state
                    .skills
                    .get(active.skill_id)
                    .and_then(|skill| skill.bound_category_target())
                    .zip(category)
                    .is_some_and(|((target, remembered), used)| {
                        target == ctx.source && remembered == used
                    })
        }

        TriggerFilter::OwnerPhaseGiftTargetIsEventSource => {
            owner_alive
                && ctx.kind == EventKind::Death
                && state
                    .skills
                    .get(active.skill_id)
                    .and_then(|skill| skill.phase_gift_target())
                    == Some(ctx.source)
        }

        TriggerFilter::OwnerPhaseGiftTargetIsEventPlayer => {
            owner_alive
                && ctx.kind == EventKind::PhasePlayEnd
                && state
                    .skills
                    .get(active.skill_id)
                    .and_then(|skill| skill.phase_gift_target())
                    == Some(ctx.player)
        }

        TriggerFilter::OwnerSourceShaResolvedHasGainableCardsAndRecipient => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            let sha_gainable = ctx.card.is_some_and(|card| {
                state.cards.get(card).is_some_and(|card_state| {
                    card_state.def != engine_api::CardDefId::NONE
                        && Some(card_state.zone) == discard
                })
            });
            let shan_gainable = ctx.cards.iter().any(|card| {
                state
                    .cards
                    .get(*card)
                    .is_some_and(|card_state| Some(card_state.zone) == discard)
            });
            let recipient_exists =
                state.players.alive_by_seat().into_iter().any(|recipient| {
                    recipient != active.owner && !ctx.targets.contains(&recipient)
                });
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::ShaResolved
                && (sha_gainable || shan_gainable)
                && recipient_exists
        }

        TriggerFilter::OwnerAliveOtherSourceShaResolvedNotTargetGainableOncePerTurn => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            owner_alive
                && ctx.kind == EventKind::ShaResolved
                && ctx.source.is_some()
                && ctx.source != active.owner
                && !ctx.targets.contains(&active.owner)
                && ctx.card.is_some_and(|card| {
                    state.cards.get(card).is_some_and(|card_state| {
                        card_state.def != engine_api::CardDefId::NONE
                            && Some(card_state.zone) == discard
                    })
                })
                && turn_fires < 1
        }

        TriggerFilter::OwnerIsEventPlayerTiedMaxHand => {
            let owner_hand = state.hand_count(active.owner);
            owner_alive
                && owner_is_player
                && state
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .all(|player| state.hand_count(player) <= owner_hand)
        }

        TriggerFilter::OwnerIsEventPlayerTiedMaxHp => {
            let owner_hp = state
                .players
                .get(active.owner)
                .map(|player| player.hp)
                .unwrap_or(i32::MIN);
            owner_alive
                && owner_is_player
                && state.players.alive_by_seat().into_iter().all(|player| {
                    state
                        .players
                        .get(player)
                        .is_some_and(|participant| participant.hp <= owner_hp)
                })
        }

        TriggerFilter::OwnerIsEventSourceOrPlayerXinyaomingAvailable => {
            let used_mask = state
                .skills
                .get(active.skill_id)
                .map_or(u8::MAX, |skill| skill.xinyaoming_used_mask());
            ctx.kind == EventKind::Damage
                && owner_alive
                && (owner_is_source || owner_is_player)
                && state.players.alive_by_seat().into_iter().any(|target| {
                    !crate::query::xinyaoming_available_actions(
                        state,
                        active.owner,
                        target,
                        used_mask,
                    )
                    .is_empty()
                })
        }

        TriggerFilter::OwnerIsEventPlayerTiedMaxEquipNonempty => {
            let owner_equip = equip_count(active.owner);
            owner_alive
                && owner_is_player
                && owner_equip > 0
                && state
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .all(|player| equip_count(player) <= owner_equip)
        }

        TriggerFilter::OwnerIsEventPlayerSkippedPhaseCountPositive => {
            owner_alive && owner_is_player && state.turn.skipped_phase_count() > 0
        }

        TriggerFilter::OwnerDiscardedHasHandOncePerPhase => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardsMoved
                && matches!(ctx.from_zone, Some(ZoneKind::Hand | ZoneKind::Equip))
                && ctx.move_reason == CardMoveReason::Discard
                && ctx.amount > 0
                && state.hand_count(active.owner) > 0
                && state.turn.phase != Phase::Idle
                && play_phase_fires == 0
        }

        TriggerFilter::OwnerPhaseEndLostCards => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseEnd
                && state.turn.card_loss_count(active.owner) > 0
        }

        TriggerFilter::OwnerAliveSecondGlobalDying => {
            // 全局第二次濒死结算后（DyingAfter 时刻计数已含本次进入）。
            owner_alive && ctx.kind == EventKind::DyingAfter && state.lifetime_dying_events == 2
        }

        TriggerFilter::OwnerIsTargetSourceHpGtOwnerOrSourceDamagedOwner => {
            // 粗遐：CardTargeted 逐目标 ctx（player=目标 owner，source=使用者）。
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardTargeted
                && ctx.source != active.owner
                && (state.players.get(ctx.source).is_some_and(|source| {
                    source.hp > state.players.get(active.owner).map(|o| o.hp).unwrap_or(0)
                }) || state.damage_history.contains(&(ctx.source, active.owner)))
        }

        TriggerFilter::OwnerIsTargetOwnerHpGtSourceOrSourceDamagedOwner => {
            // 未命：owner 体力更高，或使用者伤害过 owner。
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardTargeted
                && ctx.source != active.owner
                && (state.players.get(active.owner).is_some_and(|owner| {
                    owner.hp > state.players.get(ctx.source).map(|s| s.hp).unwrap_or(0)
                }) || state.damage_history.contains(&(ctx.source, active.owner)))
        }

        TriggerFilter::OwnerIsTargetSanshouTypeTracked => owner_alive && owner_is_player,

        TriggerFilter::OwnerSourceInTurnTargetUnharmedCardActionsGeTargetHp => {
            // 威临：自己回合内、目标本局未受过伤、owner 本局用牌数 ≥ 目标体力。
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::DamageBefore
                && state.turn.current == active.owner
                && !state
                    .damage_history
                    .iter()
                    .any(|(_, damaged)| *damaged == ctx.player)
                && state
                    .card_action_totals
                    .get(&active.owner)
                    .copied()
                    .unwrap_or(0)
                    >= state
                        .players
                        .get(ctx.player)
                        .map(|p| p.hp.max(0) as u32)
                        .unwrap_or(0)
        }

        TriggerFilter::OwnerUsedAndWasTargetedThisTurnInRange => {
            // 直谏：任意角色结束阶段，owner 本回合用过/打出过牌、被指定为
            // 目标过且射程内有角色（上游 useCard||respond 双历史）。
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && state
                    .turn
                    .flag(turn_flag::card_action_count_by_player(active.owner))
                    > 0
                && state.turn.card_targeted_count(active.owner) > 0
                && crate::query::all_other_players_in_attack_range(state, active.owner)
        }

        TriggerFilter::OwnerAliveLostHandThisTurn => {
            owner_alive
                && !owner_is_player
                && ctx.kind == EventKind::PhaseEnd
                && ctx.player != active.owner
                && state.turn.card_loss_count(active.owner) > 0
        }

        TriggerFilter::OwnerAlivePhaseEndLostCardsAtLeastTwo => {
            // 精练：任意角色结束阶段，若你本回合至少失去过两张牌（回合级失牌计数）。
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && state.turn.card_loss_count(active.owner) >= 2
        }

        TriggerFilter::OwnerSourceFirstShaThisPlayPhase => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && owner_is_source
                && active.owner == state.turn.current
                && state.turn.phase == Phase::Play
                && ctx.card_kind.is_some_and(|kind| {
                    crate::state::CardKind::from_tag(kind).is_some_and(|kind| kind.is_sha())
                })
                && state.turn.flag(turn_flag::sha_used_by_player(active.owner)) == 0
        }

        TriggerFilter::OwnerSourceDamageTargetDistanceAtMostOneEmptyEquipSlotUnused => {
            let empty_slot = state.players.get(active.owner).is_some_and(|owner| {
                let equip = state
                    .zones
                    .find(ZoneKind::Equip, active.owner)
                    .and_then(|zone| state.zones.get(zone));
                (1..=5).any(|tag| {
                    let slot = crate::state::EquipSlot::from_tag(tag).expect("valid equip slot");
                    owner.equip_slot_enabled(slot)
                        && equip.is_none_or(|zone| {
                            zone.cards.iter().all(|card| {
                                state
                                    .cards
                                    .get(*card)
                                    .and_then(|card| card.kind.equip_slot())
                                    != Some(slot)
                            })
                        })
                })
            });
            owner_alive
                && ctx.kind == EventKind::Damage
                && owner_source_distance_le_one
                && event_player_alive
                && empty_slot
                && turn_fires == 0
        }

        TriggerFilter::OwnerSourceFirstShaTargetThisTurn => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_source
                && event_player_alive
                && card_is_sha
                && history.map_or_else(
                    || state.turn.flag(turn_flag::sha_used_by_player(active.owner)) == 1,
                    |history| history.owner_sha_used == 1,
                )
        }

        TriggerFilter::OwnerAliveOtherPhasePlayEndTwzhengjianCondition => {
            let (branch, awakened, _) = state
                .skills
                .get(active.skill_id)
                .map_or((None, false, Vec::new()), |skill| skill.twzhengjian_state());
            let failed = match branch {
                Some(0) => {
                    state
                        .turn
                        .flag(turn_flag::play_card_category_history_by_player(ctx.player))
                        & 0b110
                        == 0
                }
                Some(1) => !state.turn.gained_card_in_play_phase(ctx.player),
                _ => false,
            };
            owner_alive
                && ctx.kind == EventKind::PhasePlayEnd
                && ctx.player != active.owner
                && event_player_alive
                && failed
                && (awakened
                    || !crate::query::discardable_hand_equip_cards(state, ctx.player).is_empty())
        }

        TriggerFilter::OwnerSourceShaTargetTwzhenxiAvailableOncePerTurn => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_source
                && event_player_alive
                && ctx.player != active.owner
                && card_is_sha
                && turn_fires == 0
                && (!crate::query::discardable_target_cards(
                    state,
                    active.owner,
                    ctx.player,
                    rule_api::TargetCardScope::Hand,
                )
                .is_empty()
                    || !crate::query::movable_field_cards_from(state, ctx.player).is_empty())
        }

        TriggerFilter::OwnerSourceFirstTargetDamageCardOncePerTurn => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_source
                && ctx.targets.first() == Some(&ctx.player)
                && ctx.target_count > 0
                && card_kind.is_some_and(|kind| kind.is_sha() || kind.is_damage_trick())
        }

        TriggerFilter::OwnerOutsideTurnDiscardedRedCardsInDiscard => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            owner_alive
                && owner_is_player
                && state.turn.current != active.owner
                && ctx.kind == EventKind::CardsMoved
                && ctx.move_reason == CardMoveReason::Discard
                && ctx.to_zone == discard
                && state
                    .players
                    .iter_live()
                    .any(|player| player.alive && player.id != active.owner)
                && ctx.cards.iter().any(|card| {
                    state.cards.get(*card).is_some_and(|card_state| {
                        Some(card_state.zone) == discard
                            && crate::query::effective_card_color_mask(state, active.owner, *card)
                                == 0b01
                    })
                })
        }

        TriggerFilter::OwnerAliveOtherTwkujianUsedOrResponded => {
            let tracked = state
                .skills
                .get(active.skill_id)
                .map_or_else(Vec::new, |skill| skill.twkujian_cards());
            owner_alive
                && event_player_alive
                && ctx.player != active.owner
                && match ctx.kind {
                    EventKind::CardUsed | EventKind::CardResponded => {
                        ctx.card.is_some_and(|card| tracked.contains(&card))
                    }
                    EventKind::CardsMoved => {
                        ctx.from_zone == Some(ZoneKind::Hand)
                            && ctx.move_reason.is_use_or_respond()
                            && ctx.cards.iter().any(|card| tracked.contains(card))
                    }
                    _ => false,
                }
        }

        TriggerFilter::OwnerAliveOtherTwkujianLost => {
            let tracked = state
                .skills
                .get(active.skill_id)
                .map_or_else(Vec::new, |skill| skill.twkujian_cards());
            owner_alive
                && event_player_alive
                && ctx.player != active.owner
                && ctx.kind == EventKind::CardsMoved
                && ctx.from_zone == Some(ZoneKind::Hand)
                && !ctx.move_reason.is_use_or_respond()
                && ctx.cards.iter().any(|card| tracked.contains(card))
        }

        TriggerFilter::OwnerAliveTwruilianRoundUnprompted => {
            owner_alive
                && ctx.kind == EventKind::PhaseBegin
                && state.turn.round > 0
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.twruilian_state().0 != state.turn.round)
        }

        TriggerFilter::OwnerAliveTwruilianSelectedTargetDiscarded => {
            let selected = state
                .skills
                .get(active.skill_id)
                .map_or(PlayerId::NONE, |skill| skill.twruilian_state().1);
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && selected.is_some()
                && ctx.player == selected
                && state.turn.current == selected
                && ctx.move_reason == CardMoveReason::Discard
                && !ctx.cards.is_empty()
        }

        TriggerFilter::OwnerAliveTwruilianSelectedTargetTurnEnded => {
            owner_alive
                && ctx.kind == EventKind::TurnEnded
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.twruilian_state().1 == ctx.player)
        }

        TriggerFilter::OwnerAliveLostWeaponOutsideTurn => {
            let to_own_hand = ctx
                .to_zone
                .and_then(|zone| state.zones.get(zone))
                .is_some_and(|zone| zone.kind == ZoneKind::Hand && zone.owner == active.owner);
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.player == active.owner
                && state.turn.current != active.owner
                && ctx.from_zone == Some(ZoneKind::Equip)
                && !to_own_hand
                && ctx.cards.iter().any(|card| {
                    state
                        .cards
                        .get(*card)
                        .is_some_and(|card| card.kind.equip_slot() == Some(EquipSlot::Weapon))
                })
        }

        TriggerFilter::OwnerAliveOtherPhasePrepareCanCompareOwner => {
            owner_alive
                && ctx.kind == EventKind::PhasePrepare
                && ctx.player != active.owner
                && event_player_alive
                && !state.hand_cards(ctx.player).is_empty()
                && !state.hand_cards(active.owner).is_empty()
        }

        TriggerFilter::OwnerTargetPhysicalShaSourceHpGtOwner => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_was_targeted
                && ctx.source != active.owner
                && card_is_sha
                && ctx
                    .card
                    .and_then(|card| state.cards.get(card))
                    .is_some_and(|card| !card.def.is_none())
                && state
                    .players
                    .get(ctx.source)
                    .zip(state.players.get(active.owner))
                    .is_some_and(|(source, owner)| source.alive && source.hp > owner.hp)
        }

        TriggerFilter::OwnerSourceOtherTargetShaOrOrdinaryTrickTwguoyiCondition => {
            let target_is_max_hp = state.players.alive_by_seat().iter().all(|player| {
                state.players.get(*player).map_or(true, |candidate| {
                    state
                        .players
                        .get(ctx.player)
                        .is_some_and(|target| target.hp >= candidate.hp)
                })
            });
            let target_is_max_hand = state
                .players
                .alive_by_seat()
                .iter()
                .all(|player| state.hand_count(ctx.player) >= state.hand_count(*player));
            let owner_low_hand = state.players.get(active.owner).is_some_and(|owner| {
                state.hand_count(active.owner) as i32 <= (owner.max_hp as i32 - owner.hp).max(0) + 1
            });
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_source
                && ctx.player != active.owner
                && event_player_alive
                && (card_is_sha || card_is_ordinary_trick)
                && state.resolution.effect_source_skill != active.skill_id.raw()
                && (target_is_max_hp || target_is_max_hand || owner_low_hand)
        }

        TriggerFilter::OwnerAlivePhaseEndCausedTwoDistinctDying => {
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && state.turn.dying_caused_count(active.owner) >= 2
        }

        TriggerFilter::OwnerGainedEventCardsFromOtherPersonalZone => {
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.source.is_some()
                && ctx.source != active.owner
                && matches!(
                    ctx.from_zone,
                    Some(ZoneKind::Hand | ZoneKind::Equip | ZoneKind::Judge)
                )
                && ctx
                    .to_zone
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| zone.kind == ZoneKind::Hand && zone.owner == active.owner)
                && ctx.cards.iter().any(|card| {
                    state.cards.get(*card).is_some_and(|card| {
                        card.owner == active.owner
                            && state
                                .zones
                                .get(card.zone)
                                .is_some_and(|zone| zone.kind == ZoneKind::Hand)
                    })
                })
        }

        TriggerFilter::OwnerAliveOtherSingleTargetUsedTrackedTwyizhu => {
            let tracked = state
                .skills
                .get(active.skill_id)
                .map(|skill| skill.twkujian_cards())
                .unwrap_or_default();
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && ctx.source.is_some()
                && ctx.source != active.owner
                && ctx.target_count == 1
                && ctx.cards.iter().any(|card| tracked.contains(card))
        }

        TriggerFilter::OwnerTrackedTwyizhuEnteredDiscard => {
            let tracked = state
                .skills
                .get(active.skill_id)
                .map(|skill| skill.twkujian_cards())
                .unwrap_or_default();
            ctx.kind == EventKind::CardsMoved
                && ctx
                    .to_zone
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| zone.kind == ZoneKind::DiscardPile)
                && ctx.cards.iter().any(|card| tracked.contains(card))
        }

        TriggerFilter::OwnerAliveOtherShaResolvedTrackedSkillCardDealtDamage
        | TriggerFilter::OwnerAliveOtherShaResolvedTrackedSkillCardNoDamage => {
            let tracked = state
                .skills
                .get(active.skill_id)
                .map_or_else(Vec::new, |skill| skill.tracked_cards());
            let dealt_damage =
                ctx.card_flags & rule_api::damage_card_flag::CARD_USE_DEALT_DAMAGE != 0;
            owner_alive
                && source_alive
                && ctx.kind == EventKind::ShaResolved
                && ctx.source != active.owner
                && ctx.card.is_some_and(|card| tracked.contains(&card))
                && (dealt_damage
                    == matches!(
                        def.filter,
                        TriggerFilter::OwnerAliveOtherShaResolvedTrackedSkillCardDealtDamage
                    ))
        }

        TriggerFilter::OwnerTrackedSkillCardLeftHandNotUseOrRespond => {
            let tracked = state
                .skills
                .get(active.skill_id)
                .map_or_else(Vec::new, |skill| skill.tracked_cards());
            ctx.kind == EventKind::CardsMoved
                && ctx.player != active.owner
                && ctx.from_zone == Some(ZoneKind::Hand)
                && !ctx.move_reason.is_use_or_respond()
                && ctx.cards.iter().any(|card| tracked.contains(card))
        }

        TriggerFilter::OwnerAliveShaUsedBySelfOrInRangeDiscardableHandEquip => {
            owner_alive
                && source_alive
                && ctx.kind == EventKind::CardUsed
                && card_kind.is_some_and(CardKind::is_sha)
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
                && (ctx.source == active.owner
                    || crate::query::player_in_attack_range(state, active.owner, ctx.source))
        }

        TriggerFilter::OwnerAliveTrackedCuijinShaResolvedNoDamage
        | TriggerFilter::OwnerAliveTrackedCuijinShaResolvedDealtDamage => {
            let tracked = state
                .skills
                .get(active.skill_id)
                .and_then(|skill| skill.source_free_physical_card());
            let dealt_damage =
                ctx.card_flags & rule_api::damage_card_flag::CARD_USE_DEALT_DAMAGE != 0;
            owner_alive
                && ctx.kind == EventKind::ShaResolved
                && ctx.card == tracked
                && (dealt_damage
                    == matches!(
                        def.filter,
                        TriggerFilter::OwnerAliveTrackedCuijinShaResolvedDealtDamage
                    ))
                && (dealt_damage || source_alive)
        }

        TriggerFilter::OwnerAliveTrackedTwqingkouDuelResolved => {
            let tracked = state
                .skills
                .get(active.skill_id)
                .and_then(|skill| skill.source_free_physical_card());
            owner_alive
                && ctx.kind == EventKind::DuelResolved
                && ctx.respond_to == active.owner
                && ctx.card == tracked
                && ctx.card_flags & rule_api::damage_card_flag::CARD_USE_DEALT_DAMAGE != 0
        }

        TriggerFilter::OwnerAliveTwchenxunRoundUnprompted => {
            owner_alive
                && ctx.kind == EventKind::PhaseBegin
                && state.turn.round > 0
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.twchenxun_round() != state.turn.round)
                && crate::command::play::source_free_card_has_legal_use(
                    state,
                    active.owner,
                    CardKind::TrickJuedou,
                )
        }

        TriggerFilter::OwnerAliveTrackedTwchenxunDuelTargetDamaged
        | TriggerFilter::OwnerAliveTrackedTwchenxunDuelTargetNotDamaged => {
            let tracked = state
                .skills
                .get(active.skill_id)
                .and_then(|skill| skill.twchenxun_card());
            let target_damaged = ctx.card_flags & rule_api::damage_card_flag::CARD_USE_DEALT_DAMAGE
                != 0
                && ctx.targets.first().copied() == Some(ctx.source);
            owner_alive
                && ctx.kind == EventKind::DuelResolved
                && ctx.respond_to == active.owner
                && ctx.card == tracked
                && (target_damaged
                    == matches!(
                        def.filter,
                        TriggerFilter::OwnerAliveTrackedTwchenxunDuelTargetDamaged
                    ))
        }

        TriggerFilter::OwnerAliveTrackedTwchenxunCardLeftProcessing => {
            let tracked = state
                .skills
                .get(active.skill_id)
                .and_then(|skill| skill.twchenxun_card());
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.from_zone == Some(ZoneKind::Processing)
                && tracked.is_some_and(|card| ctx.cards.contains(&card))
        }

        TriggerFilter::OwnerAliveOtherTwgonghuanDamageBeforeHpLteOwnerUnused => {
            let target_has_helper = state.skills.sorted_live().into_iter().any(|instance| {
                instance.owner == ctx.player
                    && instance.def == active.skill_def
                    && state.skill_effectively_enabled(instance)
            });
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && ctx.player != active.owner
                && ctx.amount > 0
                && !state.resolution.damage_redirect_in_progress
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.turn_fire_count == 0)
                && target_has_helper
                && state
                    .players
                    .get(ctx.player)
                    .zip(state.players.get(active.owner))
                    .is_some_and(|(target, owner)| target.alive && target.hp <= owner.hp)
        }

        TriggerFilter::OwnerPhaseBeginPlayableSkillPileNonempty => {
            owner_alive
                && ctx.kind == EventKind::PhaseBegin
                && ctx.player == active.owner
                && state.skill_piles.iter().any(|pile| {
                    pile.owner == active.owner
                        && pile.usable_as_hand
                        && state
                            .zones
                            .get(pile.zone)
                            .is_some_and(|zone| !zone.cards.is_empty())
                })
        }

        TriggerFilter::OwnerUsedPlayableSkillPileCardUnused => {
            owner_alive
                && matches!(ctx.kind, EventKind::CardUsed | EventKind::CardResponded)
                && (ctx.source == active.owner || ctx.player == active.owner)
                && ctx.card.is_some_and(|card| {
                    state.turn.used_playable_skill_pile_card(active.owner, card)
                })
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.turn_fire_count == 0)
        }

        TriggerFilter::OwnerTargetDamageBeforePlayableSkillPilesEmpty => {
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && ctx.player == active.owner
                && !state.skill_piles.iter().any(|pile| {
                    pile.owner == active.owner
                        && pile.usable_as_hand
                        && state
                            .zones
                            .get(pile.zone)
                            .is_some_and(|zone| !zone.cards.is_empty())
                })
        }

        TriggerFilter::OwnerAliveDamageBeforeTargetMinHpUnused => {
            let min_hp = state
                .players
                .alive_by_seat()
                .into_iter()
                .filter_map(|player| state.players.get(player).map(|p| p.hp))
                .min();
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|target| target.alive && Some(target.hp) == min_hp)
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.turn_fire_count == 0)
        }

        TriggerFilter::OwnerAliveGlobalPhaseEnd => owner_alive && ctx.kind == EventKind::PhaseEnd,

        TriggerFilter::OwnerAliveOtherPhaseEndMobiledanshou => {
            let count = state.turn.card_target_count(ctx.player, active.owner);
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && ctx.player != active.owner
                && (count == 0
                    || (event_player_alive
                        && count as usize
                            <= crate::query::discardable_hand_equip_cards(state, active.owner)
                                .len()))
        }

        TriggerFilter::OwnerAliveOtherPhaseEndNotDamagedOwnerThisTurn => {
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && ctx.player != active.owner
                && state.turn.damage_pair_count(ctx.player, active.owner) == 0
        }

        TriggerFilter::OwnerAliveOtherPhaseEndUsedCardOnOwnerUndamagedThisTurn => {
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && ctx.player != active.owner
                && state.turn.card_target_count(ctx.player, active.owner) > 0
                && state.turn.damage_taken_count(active.owner) == 0
        }

        TriggerFilter::OwnerAliveGlobalPhaseEndLostCardsTargetOtherAlive => {
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && ctx.player != active.owner
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|player| player.alive)
                && state.turn.card_loss_count(active.owner) > 0
        }

        TriggerFilter::OwnerAliveGlobalPhaseEndDiscardedCardsAvailable => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            let has_discarded_card = state.turn.discarded_cards.iter().any(|card| {
                state
                    .cards
                    .get(*card)
                    .is_some_and(|card| Some(card.zone) == discard)
            });
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && event_player_alive
                && (ctx.player != active.owner
                    || state
                        .players
                        .iter_live()
                        .any(|player| player.alive && player.id != active.owner))
                && has_discarded_card
        }

        TriggerFilter::OwnerSourceActiveDczhaowenTrackedRed => {
            let token = state
                .turn
                .round
                .saturating_mul(16)
                .saturating_add(state.turn.current.raw());
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardUsed
                && state.turn.current == active.owner
                && state.turn.phase == Phase::Play
                && state.skills.get(active.skill_id).is_some_and(|skill| {
                    skill.view_as_use_round == token
                        && ctx.card.is_some_and(|card| {
                            skill.tracked_cards().contains(&card)
                                && state.cards.get(card).is_some_and(|card| {
                                    matches!(
                                        crate::query::effective_card_suit(
                                            state,
                                            active.owner,
                                            card
                                        ),
                                        engine_api::Suit::Heart | engine_api::Suit::Diamond
                                    )
                                })
                        })
                })
        }

        TriggerFilter::OwnerIsTargetEventCardJiu => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardTargeted
                && card_kind == Some(CardKind::Jiu)
        }

        TriggerFilter::OwnerIsTargetOtherSourceBlackDcjiudunAvailable => {
            let has_jiu = state.players.get(active.owner).is_some_and(|player| {
                player
                    .marks
                    .iter()
                    .any(|(mark, count)| *mark == crate::content::MARK_JIU && *count > 0)
            });
            let has_discardable_hand =
                crate::query::discardable_hand_equip_cards(state, active.owner)
                    .into_iter()
                    .any(|card| {
                        state.cards.get(card).is_some_and(|card| {
                            state
                                .zones
                                .get(card.zone)
                                .is_some_and(|zone| zone.kind == ZoneKind::Hand)
                        })
                    });
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardTargeted
                && ctx.source.is_some()
                && ctx.source != active.owner
                && card_is_black
                && (!has_jiu || has_discardable_hand)
        }

        TriggerFilter::OwnerAlivePhaseEndUsedCardNoDamage => {
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && ctx.player == active.owner
                && state
                    .turn
                    .flag(turn_flag::card_used_by_player(active.owner))
                    > 0
                && state.turn.damage_dealt_count(active.owner) == 0
        }

        TriggerFilter::OwnerCardsEnteredDiscardNotUseUnused => {
            let to_discard = ctx
                .to_zone
                .and_then(|zone| state.zones.get(zone))
                .is_some_and(|zone| zone.kind == ZoneKind::DiscardPile);
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.player == active.owner
                && matches!(
                    ctx.from_zone,
                    Some(ZoneKind::Hand | ZoneKind::Equip | ZoneKind::Judge)
                )
                && to_discard
                && !ctx.move_reason.is_use_or_respond()
                && !ctx.cards.is_empty()
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.turn_fire_count == 0)
        }

        TriggerFilter::OwnerIsEventPlayerDiscardedAtLeastTwoInDiscardPhase => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseDiscardEnd
                && state.turn.flag(turn_flag::DISCARD_PHASE_DISCARDED) >= 2
        }

        TriggerFilter::OwnerAliveOtherDiscardedInDiscardPhase => {
            owner_alive
                && ctx.kind == EventKind::PhaseDiscardEnd
                && ctx.player != active.owner
                && crate::effect::discard_phase_discards_for(state, ctx.player)
                    .next()
                    .is_some()
        }

        TriggerFilter::OwnerAliveOtherPlayPhaseZhengsuMissingInRange => {
            // 整肃三技能定义（朱儁侯封 / 吕范燕集共用，ABI 432）。
            const ZHENGSU_DEFS: [u32; 3] = [2_584_022_884, 3_044_720_297, 3_824_072_820];
            owner_alive
                && ctx.kind == EventKind::PhasePlay
                && ctx.player != active.owner
                && ctx.player == state.turn.current
                && zhengsu_missing_count(state, ctx.player, &ZHENGSU_DEFS) > 0
                && crate::effect::dcchiying_in_range(state, active.owner, ctx.player)
        }

        TriggerFilter::OwnerIsTargetMissingZhengsu => {
            const ZHENGSU_DEFS: [u32; 3] = [2_584_022_884, 3_044_720_297, 3_824_072_820];
            owner_is_player
                && ctx.kind == EventKind::PhasePlay
                && zhengsu_missing_count(state, active.owner, &ZHENGSU_DEFS) > 0
        }

        TriggerFilter::OwnerAliveOtherWuPlayPhaseCanCompareOwner => {
            owner_alive
                && ctx.kind == EventKind::PhasePlay
                && ctx.player != active.owner
                && ctx.player == state.turn.current
                && state.player_group(ctx.player) == Some("wu")
                && state.player_group(active.owner).is_some()
                && !state.hand_cards(ctx.player).is_empty()
                && !state.hand_cards(active.owner).is_empty()
        }

        TriggerFilter::OwnerIsTargetMultiTargetTrickHasHandEquip => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_player
                && ctx.target_count > 1
                && card_is_trick
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
        }

        TriggerFilter::OwnerIsTargetFeijunNewTarget => {
            owner_alive
                && ctx.kind == EventKind::PhasePlayEnd
                && owner_is_player
                && state
                    .turn
                    .flag(turn_flag::feijun_new_target_by_player(active.owner))
                    != 0
        }

        TriggerFilter::OwnerSourceShaDamageTargetAliveBothHand => {
            owner_alive
                && ctx.kind == EventKind::Damage
                && owner_is_source
                && card_is_sha
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|participant| participant.alive)
                && !state.hand_cards(active.owner).is_empty()
                && !state.hand_cards(ctx.player).is_empty()
        }

        TriggerFilter::OwnerAliveOtherNanmanCardsDiscard => {
            owner_alive
                && ctx.kind == EventKind::CardResolved
                && ctx.source != active.owner
                && ctx.card_kind == Some(24)
                && ctx.card.is_some_and(|card| {
                    state.cards.get(card).is_some_and(|card_state| {
                        state
                            .zones
                            .find(ZoneKind::DiscardPile, PlayerId::NONE)
                            .is_some_and(|zone| card_state.zone == zone)
                    })
                })
        }

        TriggerFilter::OwnerAliveSkillBoundTargetIsEventSource => {
            owner_alive
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|instance| instance.bound_target() == Some(ctx.source))
        }

        TriggerFilter::OwnerLostCardsOutsideTurn => {
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && state.turn.current != active.owner
                && ctx.player == active.owner
                && matches!(
                    ctx.from_zone,
                    Some(ZoneKind::Hand | ZoneKind::Equip | ZoneKind::Judge)
                )
                && ctx.to_zone.is_some_and(|to| {
                    state
                        .zones
                        .get(to)
                        .is_some_and(|zone| zone.owner != active.owner)
                })
                && !ctx.cards.is_empty()
        }

        TriggerFilter::OwnerPhaseBeginSkillPileGteThree(skill_def) => {
            owner_alive
                && ctx.kind == EventKind::PhasePrepare
                && owner_is_player
                && state.turn.current == active.owner
                && state.skill_pile_cards(active.owner, *skill_def).len() >= 3
        }

        TriggerFilter::OwnerPhaseBeginHandEmpty => {
            owner_alive
                && ctx.kind == EventKind::PhasePrepare
                && owner_is_player
                && state.turn.current == active.owner
                && state.hand_cards(active.owner).is_empty()
        }

        TriggerFilter::OwnerRespondedShan => {
            owner_alive
                && ctx.kind == EventKind::CardResponded
                && owner_is_player
                && ctx.card_kind == Some(3)
        }

        TriggerFilter::OwnerAliveOtherQunPlayPhaseCanGiveOwner => {
            owner_alive
                && ctx.kind == EventKind::PhasePlay
                && ctx.player != active.owner
                && ctx.player == state.turn.current
                && state.player_group(ctx.player) == Some("qun")
                && state.hand_cards(ctx.player).iter().any(|card| {
                    state.cards.get(*card).is_some_and(|value| {
                        let kind = crate::query::effective_card_kind(state, ctx.player, value);
                        matches!(kind, CardKind::Shan | CardKind::TrickShandian)
                    })
                })
        }

        TriggerFilter::OwnerAlivePlayPhaseActorHandNonemptyPotjiejieUnused => {
            const POTJIEJIE_USED_DEF: u32 = 4_200_000_554;
            owner_alive
                && ctx.kind == EventKind::PhasePlay
                && ctx.player == state.turn.current
                && event_player_alive
                && !state.hand_cards(ctx.player).is_empty()
                && !state
                    .players
                    .get(ctx.player)
                    .into_iter()
                    .flat_map(|player| player.skills.iter())
                    .any(|skill| {
                        state
                            .skills
                            .get(*skill)
                            .is_some_and(|instance| instance.def.raw() == POTJIEJIE_USED_DEF)
                    })
        }

        TriggerFilter::OwnerSourceShaDamageBeforePotyinzhanCondition => {
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && owner_is_source
                && card_is_sha
                && state.players.get(ctx.player).is_some_and(|target| {
                    target.alive
                        && state.players.get(active.owner).is_some_and(|owner| {
                            owner.hp <= target.hp
                                || state.hand_count(active.owner) <= state.hand_count(ctx.player)
                        })
                })
        }

        TriggerFilter::OwnerSourceDamageWithinOne => {
            owner_alive
                && ctx.kind == EventKind::Damage
                && owner_is_source
                && ctx.amount > 0
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|target| target.alive)
                && matches!(
                    crate::query::query(
                        state,
                        &QueryRequest::Distance {
                            from: active.owner,
                            to: ctx.player,
                        },
                        &mut QueryContext::default(),
                    ),
                    QueryResponse::Int(distance) if distance <= 1
                )
        }

        TriggerFilter::OwnerGainedCardsPotzhouyu => {
            let gained = match ctx.kind {
                EventKind::CardsDrawn => owner_is_player,
                EventKind::CardsMoved => ctx
                    .to_zone
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| zone.owner == active.owner && zone.kind == ZoneKind::Hand),
                _ => false,
            };
            owner_alive
                && gained
                && ctx.amount > 0
                && state.hand_count(active.owner) > 0
                && state.skills.get(active.skill_id).is_some_and(|skill| {
                    skill.content_flags & 0b110 == 0 || state.turn.current == active.owner
                })
        }

        TriggerFilter::OwnerSourceFirstTargetHasHandPotyanhui => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_source
                && ctx.targets.first() == Some(&ctx.player)
                && ctx
                    .targets
                    .iter()
                    .any(|target| state.hand_count(*target) > 0)
                && state.skills.get(active.skill_id).is_some_and(|skill| {
                    skill.content_flags & 0b110 == 0 || state.turn.current == active.owner
                })
        }

        TriggerFilter::OwnerAliveFireDamagePotfentao => {
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && ctx.nature == rule_api::damage_nature::FIRE
                && state.skills.get(active.skill_id).is_some_and(|skill| {
                    skill.content_flags & 0b110 == 0 || state.turn.current == active.owner
                })
        }

        TriggerFilter::OwnerAlivePotfentaoPendingDamage => {
            owner_alive
                && matches!(ctx.kind, EventKind::Damage | EventKind::DamagePrevented)
                && state.skills.get(active.skill_id).is_some_and(|skill| {
                    skill.state_blob.starts_with(b"PFT1")
                        && skill.state_blob.get(4..8).is_some_and(|raw| {
                            u32::from_le_bytes(raw.try_into().unwrap()) == ctx.player.raw()
                        })
                })
        }
        _ => unreachable!("filter routed to a different predicate group"),
    }
}
