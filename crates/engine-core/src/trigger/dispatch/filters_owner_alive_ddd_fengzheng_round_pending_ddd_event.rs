//! 触发过滤器谓词：ddd、event 族效果。

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
    let _owner_source_distance_le_one = owner_is_source
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
    let _source_alive = state
        .players
        .get(ctx.source)
        .map(|player| player.alive)
        .unwrap_or(false);
    let owner_alive = state
        .players
        .get(active.owner)
        .map(|player| player.alive)
        .unwrap_or(false);
    let _event_player_alive = state
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
    let hand_empty = state.hand_count(active.owner) == 0;
    let _turn_fires = state
        .skills
        .get(active.skill_id)
        .map(|s| s.turn_fire_count)
        .unwrap_or(0);
    let card_kind = ctx.card_kind.and_then(CardKind::from_tag);
    let _card_is_ordinary_trick =
        card_kind.is_some_and(|kind| kind.is_trick() && !kind.is_delayed_trick());
    let _card_is_sha = matches!(ctx.card_kind, Some(1 | 5 | 6));
    let _card_is_juedou = ctx.card_kind == Some(CardKind::TrickJuedou.tag());
    let _event_player_group = state.player_group(ctx.player);
    let _owner_alive_other_discarded_basic_outside_turn = owner_alive
        && ctx.kind == EventKind::CardsMoved
        && ctx.player.is_some()
        && ctx.player != active.owner
        && state.turn.current != active.owner
        && matches!(ctx.from_zone, Some(ZoneKind::Hand | ZoneKind::Equip))
        && ctx.move_reason == CardMoveReason::Discard
        && ctx.moved_has_basic;
    let _owner_received_gift_from_other = owner_alive
        && ctx.kind == EventKind::CardsMoved
        && ctx.source.is_some()
        && ctx.source != active.owner
        && ctx.move_reason == CardMoveReason::Give
        && ctx
            .to_zone
            .and_then(|zone| state.zones.get(zone))
            .is_some_and(|zone| zone.owner == active.owner && zone.kind == ZoneKind::Hand);
    let _owner_dealt_or_received_damage_this_turn = owner_alive
        && active
            .owner
            .raw()
            .checked_sub(1)
            .and_then(|shift| 1_u32.checked_shl(shift))
            .is_some_and(|bit| (ctx.turn_damage_sources | ctx.turn_damaged_players) & bit != 0);
    match &def.filter {
        TriggerFilter::OwnerAliveDddFengzhengRoundPending => {
            const GLOBAL: u32 = crate::command::skill::DDD_FENGZHENG_GLOBAL_DEF;
            let round = state.turn.round;
            owner_alive
                && ctx.kind == EventKind::PhaseBegin
                && round > 1
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.round_fire_round != round)
                && state.skills.sorted_live().iter().any(|skill| {
                    skill.def.raw() == GLOBAL
                        && skill.round_fire_round != 0
                        && skill.round_fire_round.saturating_add(1) == round
                })
        }

        TriggerFilter::OwnerAliveDddYulvDiscardMatches => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            let yulv = state
                .skill_pile_cards(active.owner, 694_263_366)
                .first()
                .and_then(|card| state.cards.get(*card))
                .map(|card| card.rank);
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.to_zone == discard
                && state.turn.phase != Phase::Idle
                && state
                    .players
                    .get(state.turn.current)
                    .is_some_and(|current| current.alive)
                && yulv.is_some_and(|rank| {
                    rank > 0
                        && ctx.cards.iter().any(|card| {
                            state
                                .cards
                                .get(*card)
                                .is_some_and(|card| card.rank == rank && Some(card.zone) == discard)
                        })
                })
        }

        TriggerFilter::OwnerAliveDddYouxueRoundPending => {
            owner_alive
                && ctx.kind == EventKind::PhaseBegin
                && state.turn.round > 0
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.round_fire_round != state.turn.round)
        }

        TriggerFilter::OwnerAliveDddChengjingAvailable => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && ctx.player.is_some()
                && state.players.get(ctx.player).is_some_and(|player| {
                    player
                        .marks
                        .iter()
                        .any(|(mark, count)| *mark == 2_449_657_606 && *count > 0)
                })
                && state
                    .turn
                    .ddd_chengjing_eligible_uses
                    .iter()
                    .rev()
                    .any(|card| {
                        state
                            .cards
                            .get(*card)
                            .is_some_and(|card| Some(card.zone) == discard)
                    })
        }

        TriggerFilter::DddChashiPendingPhasePlayEnd => {
            ctx.kind == EventKind::PhasePlayEnd
                && state
                    .skills
                    .get(active.skill_id)
                    .and_then(|skill| skill.ddd_chashi_state())
                    .is_some_and(|(target, _, _)| target == ctx.player)
        }

        TriggerFilter::OwnerAliveDddZhilianAvailable => {
            let mut actors: Vec<_> = state
                .players
                .iter_live()
                .filter(|actor| {
                    state
                        .turn
                        .flag(turn_flag::card_action_count_by_player(actor.id))
                        > 0
                })
                .map(|actor| (actor.seat.raw(), actor.id, actor.alive))
                .collect();
            actors.sort_by_key(|(seat, _, _)| *seat);
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && actors.len() == 2
                && actors.iter().all(|(_, _, alive)| *alive)
                && actors.iter().any(|(_, actor, _)| *actor == active.owner)
                && actors.iter().any(|(_, actor, _)| {
                    *actor != active.owner
                        && state
                            .players
                            .get(*actor)
                            .and_then(|participant| {
                                state.content.character(participant.character_def)
                            })
                            .is_some_and(|character| {
                                matches!(character.sex.as_str(), "male" | "double")
                            })
                })
        }

        TriggerFilter::OwnerAliveDddZhuansheGift => {
            owner_alive
                && ctx.kind == EventKind::PhasePlay
                && ctx.player != active.owner
                && !state.hand_cards(active.owner).is_empty()
        }

        TriggerFilter::OwnerAliveDddZhuansheCardUsed => {
            let Some(kind) = ctx.card_kind.and_then(CardKind::from_tag) else {
                return false;
            };
            let def = if kind == CardKind::Other {
                ctx.card
                    .and_then(|card| state.cards.get(card))
                    .map(|card| card.def.raw())
                    .unwrap_or(0)
            } else {
                0
            };
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && (kind.is_basic() || kind.is_trick() && !kind.is_delayed_trick())
                && state.skills.get(active.skill_id).is_some_and(|skill| {
                    skill.ddd_zhuanshe_state().is_some_and(|(actor, names)| {
                        actor == ctx.player
                            && names.iter().any(|(stored_kind, stored_def, _)| {
                                *stored_kind == kind.tag() && *stored_def == def
                            })
                    })
                })
        }

        TriggerFilter::OwnerAliveDddZhuanshePhaseEnd => {
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && state.skills.get(active.skill_id).is_some_and(|skill| {
                    skill
                        .ddd_zhuanshe_state()
                        .is_some_and(|(actor, _)| actor == ctx.player)
                })
        }

        TriggerFilter::OwnerAliveDddWeiqiu => {
            owner_alive
                && ctx.kind == EventKind::RecoverBefore
                && state.hand_cards(active.owner).is_empty()
        }

        TriggerFilter::OwnerAliveDddLianerAvailable => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            owner_alive
                && ctx.kind == EventKind::CardResolved
                && ctx.source == active.owner
                && ctx.card.is_some_and(|card| {
                    crate::query::effective_card_color_mask(state, active.owner, card) == 0b01
                })
                && ctx.cards.iter().any(|card| {
                    state
                        .cards
                        .get(*card)
                        .is_some_and(|card| Some(card.zone) == discard)
                })
        }

        TriggerFilter::OwnerAliveDddLianerCeiling => {
            owner_alive
                && ctx.kind == EventKind::TurnEnded
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.mark > 0)
        }

        TriggerFilter::OwnerAliveDddAnzhiAvailable => {
            owner_alive
                && ctx.kind == EventKind::PhaseBegin
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.ddd_anzhi_state().0)
        }

        TriggerFilter::OwnerAliveDddAnzhiTracking => {
            owner_alive && matches!(ctx.kind, EventKind::CardUsed | EventKind::CardResponded)
        }

        TriggerFilter::OwnerAliveDddAnzhiTargetBlack => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && ctx.card.is_some_and(|card| {
                    crate::query::effective_card_color_mask(state, ctx.source, card) == 0b10
                })
                && state.skills.get(active.skill_id).is_some_and(|skill| {
                    let (_, actor, target) = skill.ddd_anzhi_state();
                    actor == state.turn.current && target == ctx.player
                })
        }

        TriggerFilter::OwnerAliveDddAnzhiTurnEnd => {
            owner_alive
                && ctx.kind == EventKind::TurnEnded
                && state.skills.get(active.skill_id).is_some_and(|skill| {
                    let (_, actor, target) = skill.ddd_anzhi_state();
                    actor == ctx.player && target.is_some()
                })
        }

        TriggerFilter::OwnerAliveDddShixingTurnRefresh => {
            let desired = u32::from(ddd_shixing_female_available(state)) | 2;
            owner_alive
                && matches!(ctx.kind, EventKind::GameStarted | EventKind::PhaseBegin)
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.mark != desired)
        }

        TriggerFilter::OwnerAliveDddShixingFemaleChanged => {
            let event_is_female = state.players.get(ctx.player).is_some_and(|participant| {
                state
                    .content
                    .character(participant.character_def)
                    .is_some_and(|character| matches!(character.sex.as_str(), "female" | "double"))
            });
            owner_alive
                && event_is_female
                && matches!(
                    ctx.kind,
                    EventKind::Damage
                        | EventKind::HpLost
                        | EventKind::HpRecovered
                        | EventKind::MaxHpChanged
                )
                && state.skills.get(active.skill_id).is_some_and(|skill| {
                    let desired_female = u32::from(ddd_shixing_female_available(state));
                    skill.mark & 1 != desired_female
                })
        }

        TriggerFilter::OwnerAliveDddShixingFirstTrick => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && ctx
                    .card_kind
                    .and_then(CardKind::from_tag)
                    .is_some_and(CardKind::is_trick)
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.mark & 2 != 0)
        }

        TriggerFilter::OwnerAliveDddDangguAvailable => {
            let alive = state.players.alive_by_seat();
            let linked = alive
                .iter()
                .filter(|player| ddd_player_linked(state, **player))
                .count();
            let qun = alive
                .iter()
                .filter(|player| state.player_group(**player) == Some("qun"))
                .count();
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && ctx.player == active.owner
                && linked < qun
        }

        TriggerFilter::OwnerAliveDddFuzongDeath => {
            owner_alive
                && ctx.kind == EventKind::Death
                && state.player_group(ctx.player) == Some("qun")
                && (!state.hand_cards(active.owner).is_empty()
                    || state
                        .zones
                        .find(ZoneKind::Equip, active.owner)
                        .and_then(|zone| state.zones.get(zone))
                        .is_some_and(|zone| !zone.cards.is_empty()))
        }

        TriggerFilter::OwnerAliveDddXuanlunPending => {
            owner_alive
                && ctx.kind == EventKind::TurnEnded
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.mark & 1 != 0)
        }

        TriggerFilter::OwnerAliveDddJiexingRecoverOncePerGame => {
            owner_alive
                && ctx.kind == EventKind::RecoverBefore
                && ctx.player != active.owner
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|owner| owner.hp < owner.max_hp as i32)
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.lifetime_fire_count == 0)
        }

        TriggerFilter::OwnerAliveDddJiexingEquipOncePerGame => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && ctx.player != active.owner
                && ctx
                    .card_kind
                    .and_then(CardKind::from_tag)
                    .is_some_and(CardKind::is_equip)
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.lifetime_fire_count == 0)
                && ctx.cards.iter().copied().any(|card| {
                    ddd_zhangkai_can_take_equipment(state, active.owner, ctx.player, card)
                })
        }

        TriggerFilter::OwnerAliveDddJiexingRecoveryReset => {
            owner_alive
                && ctx.kind == EventKind::HpRecovered
                && owner_is_player
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.lifetime_fire_count > 0)
        }

        TriggerFilter::OwnerAliveDddJiexingEquipReset => {
            owner_alive
                && ctx.kind == EventKind::CardResolved
                && ctx.source == active.owner
                && ctx
                    .card_kind
                    .and_then(CardKind::from_tag)
                    .is_some_and(CardKind::is_equip)
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.lifetime_fire_count > 0)
        }

        TriggerFilter::OwnerAliveDddBaileiAvailable => {
            owner_alive
                && ctx.kind == EventKind::PhasePrepare
                && owner_is_player
                && (state.players.alive_by_seat().into_iter().any(|target| {
                    state
                        .players
                        .get(target)
                        .is_some_and(|target| target.hp == 1)
                }) || ddd_zhangkai_unique_max_equip(state).is_some_and(|target| {
                    !crate::query::gainable_target_cards(
                        state,
                        active.owner,
                        target,
                        rule_api::TargetCardScope::HandEquip,
                    )
                    .is_empty()
                }))
        }

        TriggerFilter::OwnerAliveDddTongyuRemained => {
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.move_reason == CardMoveReason::WuguRemained
                && !ctx.cards.is_empty()
                && state
                    .resolution
                    .ddd_tongyu_rejectors(active.owner)
                    .next()
                    .is_some()
        }

        TriggerFilter::OwnerIsEventPlayerJudgeNonempty => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhasePrepare
                && state
                    .zones
                    .find(ZoneKind::Judge, active.owner)
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| !zone.cards.is_empty())
        }

        TriggerFilter::OwnerIsEventPlayerDrawNotReplaced => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseDraw
                && !state.turn.draw_phase_is_fixed()
        }

        TriggerFilter::OwnerIsEventPlayerDrawNotFixedHpOneOrHandEmpty => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseDraw
                && !state.turn.draw_phase_is_fixed()
                && (hand_empty
                    || state
                        .players
                        .get(active.owner)
                        .is_some_and(|owner| owner.hp == 1))
        }

        TriggerFilter::OwnerAliveEventPlayerTurnChangedOrBecameLinked => {
            owner_alive
                && ctx.kind == EventKind::CharacterStateChanged
                && matches!(
                    ctx.amount,
                    CHARACTER_STATE_TURNED_FACE_UP
                        | CHARACTER_STATE_TURNED_FACE_DOWN
                        | CHARACTER_STATE_BECAME_LINKED
                )
        }

        TriggerFilter::OwnerAliveEventPlayerBecameFaceUpOrLinked => {
            owner_alive
                && ctx.kind == EventKind::CharacterStateChanged
                && matches!(
                    ctx.amount,
                    CHARACTER_STATE_TURNED_FACE_UP | CHARACTER_STATE_BECAME_LINKED
                )
        }

        TriggerFilter::OwnerIsEventPlayerNoEnabledEquipSlotOncePerGame => {
            owner_alive
                && ctx.kind == EventKind::EquipSlotDisabled
                && owner_is_player
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|owner| owner.disabled_equip_slots.iter().all(|count| *count > 0))
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.lifetime_fire_count == 0)
        }

        TriggerFilter::OwnerIsEventPlayerHasMovableEquipment => {
            owner_alive
                && owner_is_player
                && !crate::query::movable_equipment_cards(state, active.owner).is_empty()
        }

        TriggerFilter::OwnerIsEventPlayerTrickUsedZero => {
            owner_alive
                && owner_is_player
                && state.turn.owner_card_category_mask(active.owner) & 0b010 == 0
        }

        TriggerFilter::OwnerIsEventPlayerHpLeOne => {
            owner_alive
                && owner_is_player
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|player| player.hp <= 1)
        }

        TriggerFilter::OwnerIsEventPlayerHpLeTwo => {
            owner_alive
                && owner_is_player
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|player| player.hp <= 2)
        }

        TriggerFilter::OwnerIsEventPlayerAnyAliveSelfDiscardableHandEquip => {
            owner_alive
                && owner_is_player
                && state.players.alive_by_seat().into_iter().any(|target| {
                    !crate::query::discardable_target_cards(
                        state,
                        target,
                        target,
                        rule_api::TargetCardScope::HandEquip,
                    )
                    .is_empty()
                })
        }

        TriggerFilter::OwnerIsEventPlayerAnyAliveDiscardableByOwnerHandEquip => {
            owner_alive
                && owner_is_player
                && state.players.alive_by_seat().into_iter().any(|target| {
                    !crate::query::discardable_target_cards(
                        state,
                        active.owner,
                        target,
                        rule_api::TargetCardScope::HandEquip,
                    )
                    .is_empty()
                })
        }

        TriggerFilter::OwnerIsEventPlayerAnyAliveSelfDiscardableHand => {
            owner_alive
                && owner_is_player
                && state.players.alive_by_seat().into_iter().any(|target| {
                    !crate::query::discardable_target_cards(
                        state,
                        target,
                        target,
                        rule_api::TargetCardScope::Hand,
                    )
                    .is_empty()
                })
        }

        TriggerFilter::OwnerIsEventPlayerAnyAliveWounded => {
            owner_alive && owner_is_player && crate::query::alive_wounded_count(state) > 0
        }

        TriggerFilter::OwnerIsEventPlayerHandEquipNonemptyOtherAlive => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::Damage
                && ctx.amount > 0
                && (state.hand_count(active.owner) > 0 || equip_count(active.owner) > 0)
                && state
                    .players
                    .iter_live()
                    .any(|player| player.alive && player.id != active.owner)
        }

        TriggerFilter::OwnerIsEventPlayerHandNonemptyHiddenSkillPile(skill_def) => {
            owner_alive
                && owner_is_player
                && state.hand_count(active.owner) > 0
                && !state
                    .hidden_skill_pile_cards(active.owner, *skill_def)
                    .is_empty()
        }

        TriggerFilter::OwnerIsEventPlayerCurrentTurn => {
            owner_alive && owner_is_player && state.turn.current == active.owner
        }

        TriggerFilter::OwnerAliveEventPlayerLord => {
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|p| p.identity == crate::state::Identity::Lord)
        }

        TriggerFilter::OwnerAliveEventPlayerLordNoDamage => {
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|p| p.identity == crate::state::Identity::Lord)
                && state.turn.flag(turn_flag::DAMAGE_DEALT) == 0
        }

        TriggerFilter::OwnerAliveEventPlayerLordDamaged => {
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|p| p.identity == crate::state::Identity::Lord)
                && state.turn.flag(turn_flag::DAMAGE_DEALT) > 0
        }

        TriggerFilter::OwnerAliveEventTargetMarkPositiveNoDamage(mark) => {
            let marked_no_damage = state.players.get(ctx.player).is_some_and(|target| {
                target.alive
                    && target
                        .marks
                        .iter()
                        .find(|(id, _)| id == mark)
                        .is_some_and(|(_, count)| *count > 0)
            }) && state.turn.flag(turn_flag::DAMAGE_DEALT) == 0;
            owner_alive && ctx.kind == EventKind::TurnEnded && marked_no_damage
        }

        TriggerFilter::OwnerAliveEventTargetMarkPositive(mark) => {
            owner_alive
                && ctx.kind == EventKind::TurnEnded
                && owner_is_player
                && state.players.get(ctx.player).is_some_and(|target| {
                    target.alive
                        && target
                            .marks
                            .iter()
                            .find(|(id, _)| id == mark)
                            .is_some_and(|(_, count)| *count > 0)
                })
        }

        _ => unreachable!("filter routed to a different predicate group"),
    }
}
