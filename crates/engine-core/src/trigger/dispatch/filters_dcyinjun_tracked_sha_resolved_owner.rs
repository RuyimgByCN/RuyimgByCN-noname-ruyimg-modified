//! 触发过滤器谓词：owner 族效果。

use super::*;

pub(super) fn filter_match(
    state: &crate::state::GameState,
    active: &ActiveTrigger,
    ctx: &TriggerContext,
    _view_as_skill: u32,
    _view_as_card: u32,
    _history: Option<&TriggerHistorySnapshot>,
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
    let _owner_source_target_distance_gt_one = owner_is_source
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
            QueryResponse::Int(distance) if distance > 1
        );
    let _owner_source_wounded_distance_le_one = owner_source_distance_le_one
        && state
            .players
            .get(active.owner)
            .is_some_and(|player| player.hp < player.max_hp as i32);
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
    let _owner_hand_below_alive_count_capped_seven = owner_alive
        && state.hand_count(active.owner)
            < state
                .players
                .iter_live()
                .filter(|player| player.alive)
                .count()
                .min(7) as u32;
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
    let owner_is_min_hp = state
        .players
        .get(active.owner)
        .filter(|player| player.alive)
        .map(|owner| {
            state
                .players
                .iter_live()
                .filter(|player| player.alive)
                .all(|player| owner.hp <= player.hp)
        })
        .unwrap_or(false);
    let _owner_is_min_hand = state
        .players
        .get(active.owner)
        .filter(|player| player.alive)
        .map(|_| {
            let owner_hand = state.hand_count(active.owner);
            state
                .players
                .iter_live()
                .all(|player| owner_hand <= state.hand_count(player.id))
        })
        .unwrap_or(false);
    let hand_empty = state.hand_count(active.owner) == 0;
    let turn_fires = state
        .skills
        .get(active.skill_id)
        .map(|s| s.turn_fire_count)
        .unwrap_or(0);
    let card_kind = ctx.card_kind.and_then(CardKind::from_tag);
    let card_is_trick = card_kind.is_some_and(CardKind::is_trick);
    let _card_is_ordinary_trick =
        card_kind.is_some_and(|kind| kind.is_trick() && !kind.is_delayed_trick());
    // The modelled ordinary tricks have no upstream `multitarget` property; selectTarget alone does not make them multitarget.
    let card_is_non_multitarget_trick =
        card_kind.is_some_and(|kind| kind.is_trick() && !kind.is_delayed_trick());
    let card_is_sha = matches!(ctx.card_kind, Some(1 | 5 | 6));
    let _single_sha_source_kind = ctx
        .card
        .and_then(|card| {
            state
                .resolution
                .sha_resolution_records
                .iter()
                .find(|record| record.source == ctx.source && record.card == card)
                .and_then(|record| match record.source_cards.as_slice() {
                    [] => Some(card),
                    [source] => Some(*source),
                    _ => None,
                })
        })
        .and_then(|card| state.cards.get(card))
        .map(|card| card.kind);
    let card_is_juedou = ctx.card_kind == Some(CardKind::TrickJuedou.tag());
    let owner_was_targeted = ctx.targets.contains(&active.owner);
    let owner_equip_empty = state
        .zones
        .find(ZoneKind::Equip, active.owner)
        .and_then(|zone| state.zones.get(zone))
        .is_none_or(|zone| zone.cards.is_empty());
    let owner_hand_below_hp = state
        .players
        .get(active.owner)
        .is_some_and(|player| state.hand_count(active.owner) < player.hp.max(0) as u32);
    let _owner_hand_above_hp = state
        .players
        .get(active.owner)
        .is_some_and(|player| state.hand_count(active.owner) > player.hp.max(0) as u32);
    let owner_hand_below_max_hp = state
        .players
        .get(active.owner)
        .is_some_and(|player| state.hand_count(active.owner) < player.max_hp);
    let suit_owner = if ctx.source.is_some() {
        ctx.source
    } else {
        ctx.player
    };
    let card_suit = ctx
        .card
        .and_then(|card| state.cards.get(card))
        .map(|card| crate::query::effective_card_suit(state, suit_owner, card));
    let card_is_red = matches!(
        card_suit,
        Some(engine_api::Suit::Heart | engine_api::Suit::Diamond)
    );
    let card_is_black = matches!(
        card_suit,
        Some(engine_api::Suit::Spade | engine_api::Suit::Club)
    );
    let judgment_is_black = ctx.kind == EventKind::JudgmentResolved
        && ctx.cards.first().is_some_and(|result_card| {
            state.cards.get(*result_card).is_some_and(|card| {
                matches!(
                    crate::query::effective_card_suit(state, ctx.player, card),
                    engine_api::Suit::Spade | engine_api::Suit::Club
                )
            })
        });
    let _card_is_diamond = card_suit == Some(engine_api::Suit::Diamond);
    let _card_is_club = card_suit == Some(engine_api::Suit::Club);
    let _event_players_equal_hp = state
        .players
        .get(ctx.source)
        .zip(state.players.get(ctx.player))
        .is_some_and(|(source, target)| source.hp == target.hp);
    let _event_player_has_female_sex = state
        .players
        .get(ctx.player)
        .and_then(|player| state.content.character(player.character_def))
        .is_some_and(|character| matches!(character.sex.as_str(), "female" | "double"));
    let owner_wounded = state
        .players
        .get(active.owner)
        .is_some_and(|owner| owner.alive && owner.hp < owner.max_hp as i32);
    let event_player_group = state.player_group(ctx.player);
    let event_source_group = state.player_group(ctx.source);
    let owner_damaged_hp_le_source = state
        .players
        .get(active.owner)
        .zip(state.players.get(ctx.source))
        .is_some_and(|(owner, source)| owner.hp < owner.max_hp as i32 && owner.hp <= source.hp);
    match &def.filter {
        TriggerFilter::OwnerAliveOtherRecoveredOutsideOwnerTurn => {
            owner_alive
                && ctx.kind == EventKind::HpRecovered
                && ctx.player != active.owner
                && event_player_alive
                && state.turn.current != active.owner
        }

        TriggerFilter::OwnerIsTargetDiscardedShaInDiscardPhase => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseDiscardEnd
                && crate::effect::discard_phase_discarded_cards_for(state, active.owner).any(
                    |card| {
                        state.cards.get(card).is_some_and(|card_state| {
                            crate::query::effective_card_kind(state, active.owner, card_state)
                                .is_sha()
                        })
                    },
                )
        }

        TriggerFilter::OwnerPhaseEndOriginatingSkillPileEmpty => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseEnd
                && state
                    .skill_pile_cards(active.owner, active.skill_def.raw())
                    .is_empty()
        }

        TriggerFilter::OwnerIsTargetShaSkillPileNonempty(skill_def) => {
            owner_alive
                && owner_is_player
                && source_alive
                && ctx.kind == EventKind::CardTargeted
                && card_kind.is_some_and(|kind| kind.is_sha())
                && !state.skill_pile_cards(active.owner, *skill_def).is_empty()
        }

        TriggerFilter::OwnerAliveOtherPlayDiscardSkillPileNonempty(skill_def) => {
            owner_alive
                && state.turn.current != active.owner
                && state.turn.phase == crate::state::Phase::Play
                && ctx.kind == EventKind::CardsMoved
                && ctx.move_reason == CardMoveReason::Discard
                && !ctx.cards.is_empty()
                && !state.skill_pile_cards(active.owner, *skill_def).is_empty()
        }

        TriggerFilter::OwnerPrepareGroupOutsideAttackRangeExists => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseBegin
                && state.turn.current == active.owner
                && state.turn.phase == crate::state::Phase::Prepare
                && crate::query::other_group_count_outside_owner_attack_range(state, active.owner)
                    > 0
        }

        TriggerFilter::OwnerAliveTargetedByOther => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_player
                && ctx.source != active.owner
        }

        TriggerFilter::OwnerAliveTargetedByOtherBlackCard => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_player
                && ctx.source != active.owner
                && card_is_black
        }

        TriggerFilter::OwnerAliveTargetedByOtherRedCard => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_player
                && ctx.source != active.owner
                && card_is_red
        }

        TriggerFilter::OwnerAliveOtherSourceDeath => {
            owner_alive
                && ctx.kind == EventKind::Death
                && ctx.source != active.owner
                && ctx.source != PlayerId::NONE
        }

        TriggerFilter::OwnerIsTargetPlayPhaseHandNoSha => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhasePlay
                && state.turn.current == active.owner
                && !state.hand_cards(active.owner).into_iter().any(|card| {
                    state.cards.get(card).is_some_and(|candidate| {
                        crate::query::effective_card_kind(state, active.owner, candidate).is_sha()
                    })
                })
        }

        TriggerFilter::OwnerAliveOtherPhaseEndPlayerOutsideOwnerRange => {
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::PhaseEnd
                && active.owner != ctx.player
                && !crate::effect::dcchiying_in_range(state, ctx.player, active.owner)
        }

        TriggerFilter::OwnerIsRespondTo => {
            // 矢仄 (clanshize): someone responded to the owner's card. For CardResponded
            // events ctx.source is the responder and ctx.respond_to is the answered player.
            owner_alive && active.owner == ctx.respond_to
        }

        TriggerFilter::OwnerAliveOtherDamageSourceGroup(group) => {
            owner_alive
                && source_alive
                && ctx.kind == EventKind::Damage
                && !owner_is_source
                && event_source_group.is_some_and(|event_group| event_group == group)
        }

        TriggerFilter::OwnerDyingOtherTaoResponderGroup(group) => {
            owner_alive
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|owner| owner.dying && owner.hp <= 0)
                && source_alive
                && ctx.kind == EventKind::CardResponded
                && ctx.respond_to == active.owner
                && ctx.source != active.owner
                && card_kind == Some(CardKind::Tao)
                && event_source_group.is_some_and(|event_group| event_group == group)
        }

        TriggerFilter::OwnerAliveOtherJudgmentBlackEventPlayerGroup(group) => {
            owner_alive
                && event_player_alive
                && !owner_is_player
                && judgment_is_black
                && event_player_group.is_some_and(|event_group| event_group == group)
        }

        TriggerFilter::OwnerWoundedNotEventSource => owner_wounded && !owner_is_source,

        TriggerFilter::OwnerIsTargetChainDamageLinked => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::DamageBefore
                && ctx.card_flags & rule_api::damage_card_flag::CHAIN_PROPAGATED != 0
        }

        TriggerFilter::OwnerIsTargetWoundedSha => {
            owner_alive
                && owner_is_player
                && owner_wounded
                && ctx.kind == EventKind::DamageBefore
                && card_is_sha
        }

        TriggerFilter::OwnerIsTargetHealthyDrawNotFixed => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseDraw
                && !state.turn.draw_phase_is_fixed()
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|owner| owner.alive && owner.hp >= owner.max_hp as i32)
        }

        TriggerFilter::OwnerWoundedEventPlayerGroup(group) => {
            owner_wounded && event_player_group.is_some_and(|event_group| event_group == group)
        }

        TriggerFilter::OwnerAliveNotEventPlayer => owner_alive && !owner_is_player,

        TriggerFilter::OwnerAliveNotEventPlayerHandEquipNonempty => {
            owner_alive
                && event_player_alive
                && !owner_is_player
                && (state.hand_count(active.owner) > 0 || equip_count(active.owner) > 0)
        }

        TriggerFilter::OwnerEquipZoneChanged => {
            let entered_owner_equip = ctx
                .to_zone
                .and_then(|zone| state.zones.get(zone))
                .is_some_and(|zone| zone.kind == ZoneKind::Equip && zone.owner == active.owner);
            let left_owner_equip = owner_is_player && ctx.from_zone == Some(ZoneKind::Equip);
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.amount > 0
                && (entered_owner_equip || left_owner_equip)
        }

        TriggerFilter::OwnerAliveNotEventPlayerSourceSameGroup => {
            owner_alive
                && !owner_is_player
                && state
                    .player_group(active.owner)
                    .is_some_and(|group| state.player_group(ctx.source) == Some(group))
        }

        TriggerFilter::OwnerAliveNotEventPlayerEqualHand => {
            owner_alive
                && event_player_alive
                && !owner_is_player
                && state.hand_count(active.owner) == state.hand_count(ctx.player)
        }

        TriggerFilter::OwnerAliveNotEventPlayerHandBelowHp => {
            owner_alive
                && event_player_alive
                && !owner_is_player
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|owner| state.hand_count(active.owner) < owner.hp.max(0) as u32)
        }

        TriggerFilter::OwnerAliveNotEventPlayerNonnegativeHpGtEventPlayer => {
            owner_alive
                && !owner_is_player
                && state
                    .players
                    .get(active.owner)
                    .zip(state.players.get(ctx.player))
                    .is_some_and(|(owner, event_player)| owner.hp.max(0) > event_player.hp)
        }

        TriggerFilter::OwnerAliveCurrentTurnOtherEventPlayer => {
            owner_alive
                && active.owner == state.turn.current
                && !owner_is_player
                && event_player_alive
        }

        TriggerFilter::OwnerAliveCurrentTurnOtherEventPlayerFromHandEmpty => {
            owner_alive
                && active.owner == state.turn.current
                && !owner_is_player
                && event_player_alive
                && ctx.amount > 0
                && matches!(ctx.from_zone, Some(ZoneKind::Hand))
                && state.hand_count(ctx.player) == 0
        }

        TriggerFilter::OwnerGainsMultipleOutsideOwnDraw => {
            let owner_gained = match ctx.kind {
                EventKind::CardsDrawn => owner_is_player,
                EventKind::CardsMoved => ctx
                    .to_zone
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| zone.owner == active.owner && zone.kind == ZoneKind::Hand),
                _ => false,
            };
            owner_alive
                && owner_gained
                && ctx.amount > 1
                && !(state.turn.current == active.owner && state.turn.phase == Phase::Draw)
        }

        TriggerFilter::OwnerGainedOrLostAtLeastTwo => {
            let owner_gained = match ctx.kind {
                EventKind::CardsDrawn => owner_is_player,
                EventKind::CardsMoved => ctx
                    .to_zone
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| zone.owner == active.owner),
                _ => false,
            };
            let owner_lost = ctx.kind == EventKind::CardsMoved && owner_is_player;
            owner_alive && ctx.amount > 1 && (owner_gained || owner_lost)
        }

        TriggerFilter::OwnerGainsOutsideTurnMarkBelowAliveOncePerTurn(mark) => {
            let owner_gained_to_hand = match ctx.kind {
                EventKind::CardsDrawn => ctx.player == active.owner,
                EventKind::CardsMoved => ctx
                    .to_zone
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| zone.owner == active.owner && zone.kind == ZoneKind::Hand),
                _ => false,
            };
            let alive_count = state
                .players
                .iter_live()
                .filter(|player| player.alive)
                .count()
                .min(i32::MAX as usize) as i32;
            let mark_count = state
                .players
                .get(active.owner)
                .and_then(|owner| owner.marks.iter().find(|(id, _)| id == mark))
                .map(|(_, count)| *count)
                .unwrap_or(0);
            owner_alive
                && owner_gained_to_hand
                && ctx.amount > 0
                && active.owner != state.turn.current
                && mark_count < alive_count
                && turn_fires == 0
        }

        TriggerFilter::OwnerCurrentTurnOtherEquipmentLost => {
            owner_alive
                && active.owner == state.turn.current
                && ctx.kind == EventKind::CardsMoved
                && ctx.amount > 0
                && ctx.from_zone == Some(ZoneKind::Equip)
                && ctx.player.is_some()
                && ctx.player != active.owner
        }

        TriggerFilter::OwnerMarkPositive(mark) => {
            owner_alive
                && state.players.get(active.owner).is_some_and(|owner| {
                    owner
                        .marks
                        .iter()
                        .find(|(id, _)| id == mark)
                        .is_some_and(|(_, count)| *count > 0)
                })
        }

        TriggerFilter::OwnerAliveDamagedEventPlayer => {
            owner_alive && state.damage_history.contains(&(active.owner, ctx.player))
        }

        TriggerFilter::OwnerAliveNotEventPlayerOncePerTurn => {
            owner_alive && !owner_is_player && turn_fires == 0
        }

        TriggerFilter::OwnerAliveCurrentOrTurnFiresBelowHp => {
            owner_alive
                && (active.owner == state.turn.current
                    || state
                        .players
                        .get(active.owner)
                        .is_some_and(|owner| turn_fires < owner.hp.max(0) as u32))
        }

        TriggerFilter::OwnerAliveCardIsSha => owner_alive && card_is_sha,

        TriggerFilter::OwnerAliveOtherRespondedShaGainable => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            owner_alive
                && ctx.kind == EventKind::CardResponded
                && !owner_is_player
                && card_is_sha
                && ctx
                    .card
                    .and_then(|card| state.cards.get(card))
                    .is_some_and(|card| Some(card.zone) == discard)
        }

        TriggerFilter::OwnerAliveCardTargetsGtOne => owner_alive && ctx.target_count > 1,

        TriggerFilter::OwnerAliveNonMultitargetTrickTargetsGtOne => {
            owner_alive && ctx.target_count > 1 && card_is_non_multitarget_trick
        }

        TriggerFilter::OwnerAliveWoundedCardTargetsGtOne => {
            owner_alive
                && ctx.target_count > 1
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|owner| owner.hp < owner.max_hp as i32)
        }

        TriggerFilter::OwnerAliveTargetedByOtherSingleCardDiscardable => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && ctx.source != active.owner
                && ctx.targets.as_slice() == [active.owner]
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
        }

        TriggerFilter::OwnerAliveTargetedByTrickSingleOrDiscardable => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_player
                && card_is_trick
                && (ctx.target_count == 1
                    || !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty())
        }

        TriggerFilter::OwnerAliveTargetedByOtherTrick => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_player
                && ctx.source != active.owner
                && card_is_trick
        }

        TriggerFilter::OwnerAliveOtherDiscardedBlackToDiscard => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.player != PlayerId::NONE
                && !owner_is_player
                && ctx.move_reason == CardMoveReason::Discard
                && ctx.to_zone == discard
                && ctx.cards.iter().any(|card| {
                    state.cards.get(*card).is_some_and(|card_state| {
                        Some(card_state.zone) == discard
                            && crate::query::card_color_mask_for_suit(card_state.suit) == 0b10
                    })
                })
        }

        TriggerFilter::OwnerAliveTargetedByOtherMatchingEquipSuit => {
            let raw_suit = ctx
                .card
                .and_then(|card| state.cards.get(card))
                .map(|card| card.suit);
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_player
                && ctx.source != active.owner
                && raw_suit.is_some_and(|suit| {
                    state
                        .zones
                        .find(ZoneKind::Equip, active.owner)
                        .and_then(|zone| state.zones.get(zone))
                        .is_some_and(|zone| {
                            zone.cards.iter().any(|card| {
                                state.cards.get(*card).is_some_and(|card| {
                                    crate::query::effective_card_suit(state, active.owner, card)
                                        == suit
                                })
                            })
                        })
                })
        }

        TriggerFilter::OwnerMinHp => owner_is_min_hp,

        TriggerFilter::OwnerShaUsedAtLeastTwo => {
            owner_alive
                && owner_is_player
                && state.turn.flag(crate::state::turn_flag::SHA_PLAYED) >= 2
        }

        TriggerFilter::OwnerAliveUsedShaThisTurn => {
            owner_alive
                && state
                    .turn
                    .flag(crate::state::turn_flag::sha_used_by_player(active.owner))
                    > 0
        }

        TriggerFilter::OwnerMaxHpGtOneHandOrEquipEmpty => {
            owner_alive
                && owner_is_player
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|player| player.max_hp > 1)
                && (hand_empty
                    || state
                        .zones
                        .find(ZoneKind::Equip, active.owner)
                        .and_then(|zone| state.zones.get(zone))
                        .is_none_or(|zone| zone.cards.is_empty()))
        }

        TriggerFilter::OwnerAlivePlayPhaseCardCategoryCountPositive => {
            owner_alive && state.turn.play_card_category_count(active.owner) > 0
        }

        TriggerFilter::OwnerAlivePlayPhaseShaUserHasDiscardableHandEquip => {
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::CardUsed
                && card_is_sha
                && state.turn.current == ctx.player
                && state.turn.phase == Phase::Play
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
        }

        TriggerFilter::OwnerTargetFire => {
            owner_is_player && ctx.nature == rule_api::damage_nature::FIRE
        }

        TriggerFilter::OwnerTargetThunder => {
            owner_is_player && ctx.nature == rule_api::damage_nature::THUNDER
        }

        TriggerFilter::OwnerAliveOtherSourceShaTwicePerTurn => {
            owner_alive
                && source_alive
                && ctx.source != active.owner
                && card_is_sha
                && turn_fires < 2
        }

        TriggerFilter::OwnerHandEmpty => owner_is_player && hand_empty,

        TriggerFilter::OwnerTurnedOver => {
            owner_is_player
                && ctx.kind == EventKind::CharacterStateChanged
                && matches!(
                    ctx.amount,
                    CHARACTER_STATE_TURNED_FACE_UP | CHARACTER_STATE_TURNED_FACE_DOWN
                )
        }

        TriggerFilter::OwnerAliveOtherWeiRecoveryOwnPhaseOncePerTurn => {
            owner_alive
                && ctx.kind == EventKind::HpRecovered
                && !owner_is_player
                && state.player_group(ctx.player) == Some("wei")
                && state.turn.current == ctx.player
        }

        TriggerFilter::OwnerFromHand => {
            owner_is_player && matches!(ctx.from_zone, Some(ZoneKind::Hand))
        }

        TriggerFilter::OwnerFromEquip => {
            owner_is_player && matches!(ctx.from_zone, Some(ZoneKind::Equip))
        }

        TriggerFilter::OwnerFromEquipPerCard => {
            ctx.amount > 0 && owner_is_player && matches!(ctx.from_zone, Some(ZoneKind::Equip))
        }

        TriggerFilter::OwnerFromHandEmpty => {
            owner_is_player && matches!(ctx.from_zone, Some(ZoneKind::Hand)) && hand_empty
        }

        TriggerFilter::OwnerFromHandEmptyOtherDiscardableHandEquip => {
            owner_alive
                && owner_is_player
                && matches!(ctx.from_zone, Some(ZoneKind::Hand))
                && hand_empty
                && state
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .filter(|target| *target != active.owner)
                    .any(|target| {
                        !crate::query::discardable_target_cards(
                            state,
                            active.owner,
                            target,
                            rule_api::TargetCardScope::HandEquip,
                        )
                        .is_empty()
                    })
        }

        TriggerFilter::OwnerFromFieldEmpty => {
            owner_is_player
                && matches!(ctx.from_zone, Some(ZoneKind::Equip | ZoneKind::Judge))
                && [ZoneKind::Equip, ZoneKind::Judge].into_iter().all(|kind| {
                    state
                        .zones
                        .find(kind, active.owner)
                        .and_then(|zone| state.zones.get(zone))
                        .is_none_or(|zone| zone.cards.is_empty())
                })
        }

        TriggerFilter::OwnerTargetedByTrickEquipEmpty => {
            owner_alive && owner_was_targeted && card_is_trick && owner_equip_empty
        }

        TriggerFilter::OwnerTargetedByShaHandBelowHp => {
            owner_alive && owner_was_targeted && card_is_sha && owner_hand_below_hp
        }

        TriggerFilter::OwnerTargetedByShaOrJuedouDamagedHpLeSourceOnce => {
            owner_alive
                && owner_was_targeted
                && (card_is_sha || card_is_juedou)
                && owner_damaged_hp_le_source
                && turn_fires == 0
        }

        TriggerFilter::OwnerTargetedOutsideTurnByShaOrTrickHandEmpty => {
            owner_alive
                && owner_was_targeted
                && state.turn.current != active.owner
                && hand_empty
                && (card_is_sha || card_is_trick)
        }

        TriggerFilter::OwnerTargetedHandBelowMaxHp => {
            owner_alive && owner_was_targeted && owner_hand_below_max_hp
        }

        TriggerFilter::OwnerTargetedByOtherSingleTargetTrick => {
            owner_alive
                && owner_is_player
                && !owner_is_source
                && ctx.target_count == 1
                && card_is_trick
        }

        TriggerFilter::OwnerAliveGlobalJuedouOrRedShaUsed => {
            owner_alive && (card_is_juedou || (card_is_sha && card_is_red))
        }

        TriggerFilter::OwnerAliveOtherGlobalJuedouOrRedShaUsed => {
            owner_alive
                && ctx.player != active.owner
                && (card_is_juedou || (card_is_sha && card_is_red))
        }

        TriggerFilter::OwnerSkillPileNonempty(pile_def) => {
            owner_is_player && !state.skill_pile_cards(active.owner, *pile_def).is_empty()
        }

        TriggerFilter::OwnerLostOneHandEquipJudgeOtherCurrentPhaseOnce => {
            owner_alive
                && owner_is_player
                && ctx.amount == 1
                && matches!(
                    ctx.from_zone,
                    Some(ZoneKind::Hand | ZoneKind::Equip | ZoneKind::Judge)
                )
                && state.turn.current != active.owner
                && ctx.player == state.turn.current
                && turn_fires == 0
        }

        TriggerFilter::OwnerIsTargetSingleTargetOtherSourceOncePerTurn => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && ctx.player == active.owner
                && ctx.source != active.owner
                && ctx.target_count == 1
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.turn_fire_count == 0)
        }
        _ => unreachable!("filter routed to a different predicate group"),
    }
}
