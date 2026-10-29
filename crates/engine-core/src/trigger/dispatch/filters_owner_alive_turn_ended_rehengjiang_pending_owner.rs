//! 触发过滤器谓词：owner 族效果。

use super::*;

pub(super) fn filter_match(
    state: &crate::state::GameState,
    active: &ActiveTrigger,
    ctx: &TriggerContext,
    view_as_skill: u32,
    view_as_card: u32,
    history: Option<&TriggerHistorySnapshot>,
    def: &crate::trigger::TriggerDefinition,
) -> bool {
    use crate::state::ZoneKind;
    let owner_is_player = active.owner == ctx.player;
    let owner_is_source = active.owner == ctx.source;
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
    let owner_hand_below_alive_count_capped_seven = owner_alive
        && state.hand_count(active.owner)
            < state
                .players
                .iter_live()
                .filter(|player| player.alive)
                .count()
                .min(7) as u32;
    let owner_hand_below_alive_count_capped_eight = owner_alive
        && state.hand_count(active.owner)
            < state
                .players
                .iter_live()
                .filter(|player| player.alive)
                .count()
                .min(8) as u32;
    let event_player_alive = state
        .players
        .get(ctx.player)
        .map(|player| player.alive)
        .unwrap_or(false);
    let _event_player_distance_le_owner_hp = state
        .players
        .get(active.owner)
        .filter(|owner| owner.alive)
        .is_some_and(|owner| {
            event_player_alive
                && matches!(
                    query(
                        state,
                        &QueryRequest::Distance {
                            from: ctx.player,
                            to: active.owner,
                        },
                        &mut QueryContext::new(),
                    ),
                    QueryResponse::Int(distance) if distance <= owner.hp
                )
        });
    let unique_max_hand = |player_id| {
        state
            .players
            .get(player_id)
            .filter(|player| player.alive)
            .is_some_and(|_| {
                let hand = state.hand_count(player_id);
                state
                    .players
                    .iter_live()
                    .filter(|other| other.alive && other.id != player_id)
                    .all(|other| state.hand_count(other.id) < hand)
            })
    };
    let equip_count = |player_id| {
        state
            .zones
            .find(ZoneKind::Equip, player_id)
            .and_then(|zone| state.zones.get(zone))
            .map_or(0, |zone| zone.cards.len())
    };
    let source_is_max_equip = state
        .players
        .get(ctx.source)
        .filter(|source| source.alive)
        .is_some_and(|_| {
            let source_count = equip_count(ctx.source);
            state
                .players
                .iter_live()
                .filter(|player| player.alive)
                .all(|player| equip_count(player.id) <= source_count)
        });
    let turn_fires = state
        .skills
        .get(active.skill_id)
        .map(|s| s.turn_fire_count)
        .unwrap_or(0);
    let _play_phase_fires = state
        .skills
        .get(active.skill_id)
        .map(|s| s.play_phase_fire_count)
        .unwrap_or(0);
    let card_kind = ctx.card_kind.and_then(CardKind::from_tag);
    let card_is_ordinary_trick =
        card_kind.is_some_and(|kind| kind.is_trick() && !kind.is_delayed_trick());
    let card_is_sha = matches!(ctx.card_kind, Some(1 | 5 | 6));
    let owner_hand = state.hand_cards(active.owner);
    let _owner_hand_basic_count = owner_hand
        .iter()
        .filter(|card_id| {
            state.cards.get(**card_id).is_some_and(|card| {
                crate::query::effective_card_kind(state, active.owner, card).is_basic()
            })
        })
        .count();
    let suit_owner = if ctx.source.is_some() {
        ctx.source
    } else {
        ctx.player
    };
    let card_suit = ctx
        .card
        .and_then(|card| state.cards.get(card))
        .map(|card| crate::query::effective_card_suit(state, suit_owner, card));
    let _card_is_black = matches!(
        card_suit,
        Some(engine_api::Suit::Spade | engine_api::Suit::Club)
    );
    let previous_card_kind = history
        .map(|snapshot| snapshot.previous_card_kind)
        .or_else(|| u8::try_from(state.turn.flag(turn_flag::LAST_CARD_KIND)).ok())
        .and_then(CardKind::from_tag);
    let previous_card_def = history
        .map(|snapshot| snapshot.previous_card_def)
        .unwrap_or_else(|| state.turn.flag(turn_flag::LAST_CARD_DEF) as u32);
    let current_card_def = ctx
        .card
        .and_then(|card| state.cards.get(card))
        .map(|card| card.def.raw());
    let _repeats_previous_card_name = previous_card_kind
        .zip(card_kind)
        .zip(current_card_def)
        .is_some_and(|((previous_kind, current_kind), current_def)| {
            same_logical_card_name(previous_kind, previous_card_def, current_kind, current_def)
        });
    let _event_players_same_sex = state
        .players
        .get(ctx.source)
        .zip(state.players.get(ctx.player))
        .and_then(|(source, target)| {
            state
                .content
                .character(source.character_def)
                .zip(state.content.character(target.character_def))
        })
        .is_some_and(|(source, target)| same_sex(&source.sex, &target.sex));
    let _event_source_has_male_sex = state
        .players
        .get(ctx.source)
        .and_then(|player| state.content.character(player.character_def))
        .is_some_and(|character| matches!(character.sex.as_str(), "male" | "double"));
    let _owner_has_discardable_equipment =
        crate::query::discardable_hand_equip_cards(state, active.owner)
            .into_iter()
            .any(|card| {
                state.cards.get(card).is_some_and(|card_state| {
                    crate::query::effective_card_kind(state, active.owner, card_state).is_equip()
                })
            });
    let _owner_has_weapon = state
        .zones
        .find(ZoneKind::Equip, active.owner)
        .and_then(|zone| state.zones.get(zone))
        .is_some_and(|zone| {
            zone.cards.iter().any(|card| {
                state
                    .cards
                    .get(*card)
                    .is_some_and(|card| card.kind == CardKind::Weapon)
            })
        });
    let _owner_event_player_same_group = state
        .player_group(active.owner)
        .is_some_and(|group| state.player_group(ctx.player) == Some(group));
    let _owner_wounded = state
        .players
        .get(active.owner)
        .is_some_and(|owner| owner.alive && owner.hp < owner.max_hp as i32);
    let owner_history_bit = active
        .owner
        .raw()
        .checked_sub(1)
        .and_then(|shift| 1_u32.checked_shl(shift));
    match &def.filter {
        TriggerFilter::OwnerAliveTurnEndedRehengjiangPending => {
            owner_alive
                && ctx.kind == EventKind::TurnEnded
                && state.skills.get(active.skill_id).is_some_and(|skill| {
                    skill.state_blob.len() == 12
                        && &skill.state_blob[..4] == b"RHJ1"
                        && u32::from_le_bytes(skill.state_blob[4..8].try_into().unwrap())
                            == ctx.player.raw()
                        && u32::from_le_bytes(skill.state_blob[8..12].try_into().unwrap()) > 0
                })
        }

        TriggerFilter::OwnerAlivePhasePlayEndZhengbiPending => {
            owner_alive
                && ctx.kind == EventKind::PhasePlayEnd
                && ctx.player == active.owner
                && state.skills.get(active.skill_id).is_some_and(|skill| {
                    skill.state_blob.len() == 8 && &skill.state_blob[..4] == b"ZBI1" && {
                        let target = PlayerId::new(u32::from_le_bytes(
                            skill.state_blob[4..8].try_into().unwrap(),
                        ));
                        target.is_some()
                    }
                })
        }

        TriggerFilter::OwnerIsTargetSourceHandNonemptyPerDamagePoint => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::Damage
                && ctx.amount > 0
                && source_alive
                && !state.hand_cards(ctx.source).is_empty()
        }

        TriggerFilter::OwnerXichaWatchedSourceCardUsed => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && ctx.source.is_some()
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| xicha_state_has_source(&skill.state_blob, ctx.source))
        }

        TriggerFilter::OwnerQiaojianTrackedCardLeftHand => {
            let tracked = state
                .skills
                .get(active.skill_id)
                .map_or_else(Vec::new, |skill| skill.tracked_cards());
            let owner_hand = state.zones.find(ZoneKind::Hand, active.owner);
            ctx.kind == EventKind::CardsMoved
                && ctx.cards.iter().any(|card| {
                    tracked.contains(card)
                        && state
                            .cards
                            .get(*card)
                            .is_some_and(|card| Some(card.zone) != owner_hand)
                })
        }

        TriggerFilter::OwnerAliveDamageTargetSelfOrInRangeOtherSourceNumericCard => {
            owner_alive
                && ctx.kind == EventKind::Damage
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|target| target.alive)
                && (ctx.player == active.owner
                    || crate::query::player_in_attack_range(state, active.owner, ctx.player))
                && ctx.source != active.owner
                && ctx.card.is_some_and(|card| {
                    crate::query::physical_card_number_sum(state, card, &ctx.cards).is_some()
                })
        }

        TriggerFilter::OwnerAliveNanmanResolvedTargetDeadOrDamagedGtThree => {
            owner_alive
                && ctx.kind == EventKind::CardResolved
                && ctx.card_kind == Some(24)
                && (ctx.targets.iter().any(|target| {
                    state
                        .players
                        .get(*target)
                        .is_some_and(|participant| !participant.alive)
                }) || {
                    // amount 为受创目标位掩码（dispatch 对 CardResolved 的编码）。
                    let damaged_count = (0..ctx.targets.len())
                        .filter(|index| ctx.amount & (1u32 << index) != 0)
                        .count();
                    damaged_count > 3
                })
        }

        TriggerFilter::OwnerLostNonDamageCardPottuntianChargeNotFull => {
            let max_charge = state
                .skills
                .get(active.skill_id)
                .and_then(|skill| skill.state_blob.get(..4))
                .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
                .unwrap_or(0);
            let charge = state
                .players
                .get(active.owner)
                .and_then(|player| {
                    player
                        .marks
                        .iter()
                        .find(|(mark, _)| *mark == active.skill_def.raw())
                        .map(|(_, count)| (*count).max(0) as u32)
                })
                .unwrap_or(0);
            let moved: Vec<_> = ctx
                .cards
                .iter()
                .filter_map(|card| state.cards.get(*card))
                .collect();
            owner_alive
                && ctx.kind == EventKind::CardsMoved
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
                && charge < max_charge
                && moved
                    .iter()
                    .any(|card| !card.kind.is_sha() && !card.kind.is_damage_trick())
                && !(ctx.move_reason == CardMoveReason::Use
                    && moved.iter().all(|card| card.kind.is_equip()))
        }

        TriggerFilter::OwnerAliveShaEnteredDiscardPotzhanlieAvailable => {
            let (quota, seen) = state
                .skills
                .get(active.skill_id)
                .and_then(|skill| {
                    (skill.state_blob.len() >= 3)
                        .then(|| (skill.state_blob[1], skill.state_blob[2]))
                })
                .unwrap_or((0, 0));
            let marks = state
                .players
                .get(active.owner)
                .and_then(|owner| {
                    owner
                        .marks
                        .iter()
                        .find(|(mark, _)| *mark == active.skill_def.raw())
                        .map(|(_, count)| (*count).max(0) as u32)
                })
                .unwrap_or(0);
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && seen < quota
                && marks < 6
                && ctx.cards.iter().any(|card| {
                    state.cards.get(*card).is_some_and(|card| {
                        card.kind.is_sha()
                            && state
                                .zones
                                .get(card.zone)
                                .is_some_and(|zone| zone.kind == ZoneKind::DiscardPile)
                    })
                })
        }

        TriggerFilter::OwnerIsTargetDiscardPhaseOweLimitOtherQun => {
            owner_alive
                && ctx.kind == EventKind::PhaseDiscard
                && owner_is_player
                && ctx.player == state.turn.current
                && {
                    let limit = crate::query::query(
                        state,
                        &rule_api::QueryRequest::HandLimit {
                            player: active.owner,
                        },
                        &mut crate::query::QueryContext::new(),
                    );
                    let limit = match limit {
                        rule_api::QueryResponse::Int(value) => value.max(0) as u32,
                        _ => 0,
                    };
                    state.hand_count(active.owner) > limit
                        && state.players.alive_by_seat().into_iter().any(|candidate| {
                            candidate != active.owner
                                && state.player_group(candidate) == Some("qun")
                        })
                }
        }

        TriggerFilter::OwnerAlivePhaseTargetedHpChangedCanUseWanjian => {
            owner_alive
                && ctx.kind == EventKind::PhaseAnyEnd
                && owner_history_bit.is_some_and(|bit| {
                    ctx.turn_damage_sources & bit != 0 && ctx.turn_damaged_players & bit != 0
                })
                && crate::command::play::source_free_card_has_legal_use(
                    state,
                    active.owner,
                    CardKind::TrickWanjian,
                )
        }

        TriggerFilter::OwnerAlivePhaseTargetedHpUnchangedCanUseJuedou => {
            owner_alive
                && ctx.kind == EventKind::PhaseAnyEnd
                && owner_history_bit.is_some_and(|bit| {
                    ctx.turn_damage_sources & bit != 0 && ctx.turn_damaged_players & bit == 0
                })
                && crate::command::play::source_free_card_has_legal_use(
                    state,
                    active.owner,
                    CardKind::TrickJuedou,
                )
        }

        TriggerFilter::OwnerIsTargetDiscardableHandEquip => {
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && owner_is_player
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
        }

        TriggerFilter::OwnerHandCompareParticipant => {
            owner_alive
                && ctx.kind == EventKind::HandCompareStarted
                && (owner_is_player || owner_is_source)
        }

        TriggerFilter::OwnerHandCompareFinishedParticipant => {
            owner_alive
                && ctx.kind == EventKind::HandCompareFinished
                && (owner_is_player || owner_is_source)
        }

        TriggerFilter::OwnerHandCompareParticipantHasSha => {
            owner_alive
                && ctx.kind == EventKind::HandCompareFinished
                && (owner_is_player || owner_is_source)
                && ctx.cards.iter().any(|card| {
                    state
                        .cards
                        .get(*card)
                        .is_some_and(|card| card.kind.is_sha())
                })
        }

        TriggerFilter::OwnerHandCompareParticipantResultCardInDiscard => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            owner_alive
                && ctx.kind == EventKind::HandCompareFinished
                && hand_compare_result_card(
                    active.owner,
                    ctx.player,
                    ctx.source,
                    ctx.respond_to,
                    &ctx.cards,
                )
                .and_then(|card| state.cards.get(card))
                .is_some_and(|card| Some(card.zone) == discard)
        }

        TriggerFilter::OwnerDiscardedBlackBasicToDiscard => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && owner_is_player
                && matches!(ctx.from_zone, Some(ZoneKind::Hand | ZoneKind::Equip))
                && ctx.move_reason == CardMoveReason::Discard
                && ctx.to_zone == discard
                && ctx.cards.iter().any(|card| {
                    state.cards.get(*card).is_some_and(|card_state| {
                        Some(card_state.zone) == discard
                            && crate::query::effective_card_kind(state, active.owner, card_state)
                                .is_basic()
                            && crate::query::effective_card_color_mask(state, active.owner, *card)
                                == 0b10
                    })
                })
        }

        TriggerFilter::OwnerAliveDiscardedBlackBasicToDiscard => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.move_reason == CardMoveReason::Discard
                && ctx.to_zone == discard
                && ctx.cards.iter().any(|card| {
                    state.cards.get(*card).is_some_and(|card_state| {
                        Some(card_state.zone) == discard
                            && crate::query::effective_card_kind(state, active.owner, card_state)
                                .is_basic()
                            && crate::query::effective_card_color_mask(state, active.owner, *card)
                                == 0b10
                    })
                })
        }

        TriggerFilter::OwnerCardRespondedToByPlayerInRange => {
            if ctx.kind != EventKind::CardResponded
                || !owner_alive
                || !event_player_alive
                || ctx.respond_to != active.owner
            {
                return false;
            }
            let mut query_ctx = QueryContext::new();
            let distance = query(
                state,
                &QueryRequest::Distance {
                    from: ctx.player,
                    to: active.owner,
                },
                &mut query_ctx,
            );
            let range = query(
                state,
                &QueryRequest::AttackRange { player: ctx.player },
                &mut query_ctx,
            );
            matches!(
                (distance, range),
                (QueryResponse::Int(distance), QueryResponse::Int(range)) if distance <= range
            )
        }

        TriggerFilter::OwnerAliveFirstCardsMovedToHand => {
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx
                    .to_zone
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| zone.kind == ZoneKind::Hand)
                && turn_fires == 0
        }

        TriggerFilter::OwnerAliveNeighborGainedCardsOutsideOwnDrawOncePerTurn => {
            let target = ctx
                .to_zone
                .and_then(|zone| state.zones.get(zone))
                .and_then(|zone| {
                    (zone.kind == ZoneKind::Hand && zone.owner != active.owner)
                        .then_some(zone.owner)
                });
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.amount > 0
                && turn_fires == 0
                && target.is_some_and(|target| {
                    state
                        .players
                        .get(target)
                        .is_some_and(|participant| participant.alive)
                        && crate::query::alive_seat_neighbors(state, active.owner).contains(&target)
                        && !(state.turn.current == target && state.turn.phase == Phase::Draw)
                        && !crate::query::gainable_target_cards(
                            state,
                            active.owner,
                            target,
                            rule_api::TargetCardScope::HandEquip,
                        )
                        .is_empty()
                })
        }

        TriggerFilter::OwnerDrawsDuringOwnPlayNotSelf => {
            owner_alive
                && ctx.kind == EventKind::CardsDrawn
                && owner_is_player
                && state.turn.current == active.owner
                && state.turn.phase == Phase::Play
                && state.resolution.effect_source_skill != active.skill_id.raw()
        }

        TriggerFilter::OwnerEquipZoneChangedAtLeastTwoWounded => {
            let entered_owner_equip = ctx
                .to_zone
                .and_then(|zone| state.zones.get(zone))
                .is_some_and(|zone| zone.kind == ZoneKind::Equip && zone.owner == active.owner);
            let left_owner_equip = owner_is_player && ctx.from_zone == Some(ZoneKind::Equip);
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.amount > 0
                && crate::query::alive_wounded_count(state) >= 2
                && (entered_owner_equip || left_owner_equip)
        }

        TriggerFilter::OwnerAliveEquipUsed => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && card_kind.is_some_and(CardKind::is_equip)
        }

        TriggerFilter::OwnerAliveEquipUsedSkillMarkBelowThree => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && card_kind.is_some_and(CardKind::is_equip)
                && state.players.get(active.owner).is_some_and(|owner| {
                    owner
                        .marks
                        .iter()
                        .find(|(mark, _)| *mark == active.skill_def.raw())
                        .map_or(0, |(_, count)| *count)
                        < 3
                })
        }

        TriggerFilter::OwnerAliveOncePerRound => owner_alive,

        TriggerFilter::OwnerAlive => owner_alive,

        TriggerFilter::OwnerAliveKilledThisTurn => {
            owner_alive
                && ctx.kind == EventKind::TurnEnded
                && state
                    .turn
                    .flag(turn_flag::kill_count_by_player(active.owner))
                    > 0
        }

        TriggerFilter::OwnerAliveHandBelowAliveCountCappedSeven => {
            owner_hand_below_alive_count_capped_seven
        }

        TriggerFilter::OwnerAliveHandBelowAliveCountCappedEight => {
            owner_hand_below_alive_count_capped_eight
        }

        TriggerFilter::OwnerAliveDelayedTrickTargetWithoutPendingJudgeSkip => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardTargeted
                && card_kind.is_some_and(CardKind::is_delayed_trick)
                && !state.pending_judge_skips.contains(&active.owner)
        }

        TriggerFilter::OwnerLongdanViewAsFirstCounterpartHandNonempty => {
            let view_card = view_as_card;
            let source = history
                .map(|snapshot| {
                    (
                        SkillDefId::new(snapshot.view_as_skill_def),
                        PlayerId::new(snapshot.view_as_skill_owner),
                    )
                })
                .or_else(|| {
                    state
                        .skills
                        .get(SkillId::new(view_as_skill))
                        .map(|skill| (skill.def, skill.owner))
                });
            let converted_by_longdan = source.is_some_and(|(skill_def, skill_owner)| {
                skill_owner == active.owner
                    && state
                        .content
                        .skill_metadata(skill_def)
                        .is_some_and(|metadata| {
                            metadata.name.contains("longdan") || metadata.name.contains("fanghun")
                        })
            });
            let event_card_matches = ctx
                .card
                .map_or(view_card == 0, |card| card.raw() == view_card);
            let first_counterpart = ctx.targets.first().copied();
            owner_alive
                && event_player_alive
                && converted_by_longdan
                && event_card_matches
                && matches!(card_kind, Some(CardKind::Sha | CardKind::Shan))
                && first_counterpart.is_some_and(|counterpart| {
                    counterpart != active.owner
                        && state
                            .players
                            .get(counterpart)
                            .is_some_and(|player| player.alive)
                        && state.hand_count(counterpart) > 0
                })
                && match ctx.kind {
                    EventKind::CardTargeted => {
                        owner_is_source && first_counterpart == Some(ctx.player)
                    }
                    EventKind::CardResponded => {
                        owner_is_player && first_counterpart == Some(ctx.respond_to)
                    }
                    _ => false,
                }
        }

        TriggerFilter::OwnerAliveHandHasSameKindAsEventCard => {
            owner_alive && hand_has_same_kind_as_event_card(state, active.owner, ctx)
        }

        TriggerFilter::OwnerAliveHasSkillTaggedCards => {
            owner_alive
                && state
                    .hand_cards(active.owner)
                    .into_iter()
                    .any(|card| state.card_has_content_tag(card, active.skill_def.raw()))
        }

        TriggerFilter::OwnerAliveTaggedCardLostThisTurn => {
            owner_alive && state.turn.lost_card_tag(active.skill_def.raw())
        }

        TriggerFilter::OwnerAliveShaTargetNoOtherRedCardThisTurn => {
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::CardTargeted
                && card_is_sha
                && !history
                    .map(|snapshot| snapshot.event_player_had_other_red_card_target)
                    .unwrap_or_else(|| {
                        ctx.card.is_some_and(|card| {
                            state.turn.was_targeted_by_other_red_card(ctx.player, card)
                        })
                    })
        }

        TriggerFilter::OwnerAliveEquipSourceMaxEquip => {
            owner_alive && card_kind.is_some_and(CardKind::is_equip) && source_is_max_equip
        }

        TriggerFilter::OwnerAliveNotEventPlayerNeitherUniqueMaxHand => {
            owner_alive
                && event_player_alive
                && !owner_is_player
                && !unique_max_hand(active.owner)
                && !unique_max_hand(ctx.player)
        }

        TriggerFilter::OwnerTrackedJuedouOriginalTargetDamageRecoverableInRange => {
            let tracked = state.skills.get(active.skill_id).is_some_and(|instance| {
                ctx.card
                    .is_some_and(|card| instance.tracked_cards().contains(&card))
                    && instance
                        .active_play_phase_targets()
                        .is_some_and(|targets| targets.contains(&ctx.player))
            });
            owner_alive
                && owner_is_source
                && event_player_alive
                && ctx.kind == EventKind::Damage
                && card_kind == Some(CardKind::TrickJuedou)
                && tracked
                && state.players.iter_live().any(|candidate| {
                    candidate.alive
                        && candidate.id != active.owner
                        && candidate.hp < candidate.max_hp as i32
                        && crate::query::player_in_attack_range(state, ctx.player, candidate.id)
                })
        }

        TriggerFilter::OwnerIsTargetPhaseEndHandNonemptyOtherHandNonempty => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseEnd
                && state.hand_count(active.owner) > 0
                && state.players.iter_live().any(|candidate| {
                    candidate.alive
                        && candidate.id != active.owner
                        && state.hand_count(candidate.id) > 0
                })
        }

        TriggerFilter::OwnerIsTargetOrdinaryTrickCilvAvailable => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardTargeted
                && card_is_ordinary_trick
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|instance| crate::effect::cilv_used_mask(instance) != 0b111)
        }

        TriggerFilter::OwnerDyingZhupeilanOncePerGame => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::Dying
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|owner| owner.dying && owner.hp <= 0)
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|instance| instance.lifetime_fire_count == 0)
        }

        TriggerFilter::OwnerPhaseEndDcsbhaoyiAvailable => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseEnd
                && state.turn.entered_discard_cards.iter().any(|card| {
                    !state.turn.damage_card_materials.contains(card)
                        && state.cards.get(*card).is_some_and(|card| {
                            Some(card.zone) == discard
                                && (card.kind.is_sha() || card.kind.is_damage_trick())
                        })
                })
        }

        TriggerFilter::OwnerAliveGlobalFireDamageDcsbronghuo => {
            owner_alive
                && ctx.kind == EventKind::Damage
                && ctx.nature == rule_api::damage_nature::FIRE
        }

        TriggerFilter::OwnerHandTransferOtherDcsbmengmouBranchAvailable => {
            let destination = ctx.to_zone.and_then(|zone| state.zones.get(zone));
            let destination_owner = destination.map(|zone| zone.owner).unwrap_or(PlayerId::NONE);
            let counterpart = if ctx.source == active.owner {
                destination_owner
            } else if destination_owner == active.owner {
                ctx.source
            } else {
                PlayerId::NONE
            };
            let branch = state
                .skills
                .get(active.skill_id)
                .map_or(0, |instance| instance.view_as_branch.min(1));
            let used = state
                .turn
                .flag(crate::state::turn_flag::dcsbmengmou_used_by_player(
                    active.owner,
                )) as u32;
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.from_zone == Some(ZoneKind::Hand)
                && destination.is_some_and(|zone| zone.kind == ZoneKind::Hand)
                && counterpart.is_some()
                && counterpart != active.owner
                && state
                    .players
                    .get(counterpart)
                    .is_some_and(|participant| participant.alive)
                && used & (1u32 << branch) == 0
                && !ctx.cards.is_empty()
        }

        TriggerFilter::OwnerIsTargetCardDamageBefore => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::DamageBefore
                && (ctx.card.is_some() || ctx.card_kind.is_some())
        }

        _ => unreachable!("filter routed to a different predicate group"),
    }
}
