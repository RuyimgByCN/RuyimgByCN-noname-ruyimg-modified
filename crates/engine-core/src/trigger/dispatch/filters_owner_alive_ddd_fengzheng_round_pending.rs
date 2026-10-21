//! Trigger predicates for `OwnerAliveDddFengzhengRoundPending` and related requests.

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
    let hand_empty = state.hand_count(active.owner) == 0;
    let turn_fires = state
        .skills
        .get(active.skill_id)
        .map(|s| s.turn_fire_count)
        .unwrap_or(0);
    let card_kind = ctx.card_kind.and_then(CardKind::from_tag);
    let card_is_ordinary_trick =
        card_kind.is_some_and(|kind| kind.is_trick() && !kind.is_delayed_trick());
    let card_is_sha = matches!(ctx.card_kind, Some(1 | 5 | 6));
    let card_is_juedou = ctx.card_kind == Some(CardKind::TrickJuedou.tag());
    let event_player_group = state.player_group(ctx.player);
    let owner_alive_other_discarded_basic_outside_turn = owner_alive
        && ctx.kind == EventKind::CardsMoved
        && ctx.player.is_some()
        && ctx.player != active.owner
        && state.turn.current != active.owner
        && matches!(ctx.from_zone, Some(ZoneKind::Hand | ZoneKind::Equip))
        && ctx.move_reason == CardMoveReason::Discard
        && ctx.moved_has_basic;
    let owner_received_gift_from_other = owner_alive
        && ctx.kind == EventKind::CardsMoved
        && ctx.source.is_some()
        && ctx.source != active.owner
        && ctx.move_reason == CardMoveReason::Give
        && ctx
            .to_zone
            .and_then(|zone| state.zones.get(zone))
            .is_some_and(|zone| zone.owner == active.owner && zone.kind == ZoneKind::Hand);
    let owner_dealt_or_received_damage_this_turn = owner_alive
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

        TriggerFilter::OwnerIsTargetFaceDown => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::Damage
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|player| player.turned_over)
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

        TriggerFilter::OwnerSourceOwnViewAsDddTongyuUsed => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardUsed
                && card_kind == Some(CardKind::TrickWugu)
                && view_as_skill == active.skill_id.raw()
        }

        TriggerFilter::OwnerSourceOwnViewAsDddTongyuTargeted => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardTargeted
                && card_kind == Some(CardKind::TrickWugu)
                && view_as_skill == active.skill_id.raw()
                && event_player_alive
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

        TriggerFilter::OwnerAliveSourceBasicDddFushiAvailable => {
            const DDD_MIAOXING_PILE_1: u32 = 919_529_013;
            const DDD_MIAOXING_PILE_2: u32 = 2_949_101_455;
            owner_alive
                && owner_is_source
                && matches!(ctx.kind, EventKind::CardResolved | EventKind::CardResponded)
                && card_kind.is_some_and(CardKind::is_basic)
                && turn_fires == 0
                && (state.hand_count(active.owner) > 0
                    || !state
                        .skill_pile_cards(active.owner, DDD_MIAOXING_PILE_1)
                        .is_empty()
                    || !state
                        .skill_pile_cards(active.owner, DDD_MIAOXING_PILE_2)
                        .is_empty())
        }

        TriggerFilter::OwnerSourceShaTargetHandGteHpOrLteAttackRange => {
            if !owner_alive
                || !owner_is_source
                || !event_player_alive
                || ctx.kind != EventKind::CardTargeted
                || !card_is_sha
            {
                return false;
            }
            let hand = state.hand_count(ctx.player).min(i32::MAX as u32) as i32;
            let hp = state
                .players
                .get(active.owner)
                .map_or(i32::MAX, |owner| owner.hp);
            hand >= hp
                || matches!(
                    query(
                        state,
                        &QueryRequest::AttackRange {
                            player: active.owner,
                        },
                        &mut QueryContext::new(),
                    ),
                    QueryResponse::Int(range) if hand <= range
                )
        }

        TriggerFilter::OwnerSourceShaTargetHandLteOwnerOrHpGteOwner => {
            owner_alive
                && owner_is_source
                && event_player_alive
                && ctx.kind == EventKind::CardTargeted
                && card_is_sha
                && state
                    .players
                    .get(ctx.player)
                    .zip(state.players.get(active.owner))
                    .is_some_and(|(target, owner)| {
                        state.hand_count(ctx.player) <= state.hand_count(active.owner)
                            || target.hp >= owner.hp
                    })
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

        TriggerFilter::OwnerSourceShaDodgedTargetAliveHasDiscardableHandEquip => {
            owner_alive
                && owner_is_source
                && event_player_alive
                && ctx.kind == EventKind::ShaDodged
                && !crate::query::discardable_target_cards(
                    state,
                    active.owner,
                    ctx.player,
                    rule_api::TargetCardScope::HandEquip,
                )
                .is_empty()
        }

        TriggerFilter::OwnerSourceShaDodgedTargetRecordedBySkillHasDiscardableHandEquip => {
            owner_alive
                && owner_is_source
                && event_player_alive
                && ctx.kind == EventKind::ShaDodged
                && ctx.card.is_some_and(|card| {
                    state.resolution.sha_skill_target_recorded(
                        active.owner,
                        card,
                        active.skill_id,
                        ctx.player,
                    )
                })
                && !crate::query::discardable_target_cards(
                    state,
                    ctx.player,
                    active.owner,
                    rule_api::TargetCardScope::HandEquip,
                )
                .is_empty()
        }

        TriggerFilter::OwnerSourceShaDodged => {
            owner_alive && owner_is_source && ctx.kind == EventKind::ShaDodged
        }

        TriggerFilter::OwnerSourceShaDamageRecordedBySkill => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::Damage
                && ctx.card.is_some_and(|card| {
                    state.resolution.sha_skill_target_recorded(
                        active.owner,
                        card,
                        active.skill_id,
                        ctx.player,
                    )
                })
        }

        TriggerFilter::OwnerAliveOtherDyingOncePerRound => {
            owner_alive && active.owner != ctx.player && ctx.kind == EventKind::Dying
        }

        TriggerFilter::OwnerIsTargetDiscardableHandEquipGteTwo => {
            owner_alive
                && owner_is_player
                && crate::query::discardable_target_cards(
                    state,
                    active.owner,
                    active.owner,
                    rule_api::TargetCardScope::HandEquip,
                )
                .len()
                    >= 2
        }

        TriggerFilter::OwnerSourceTargetAliveHasHandOrEquip => {
            owner_alive
                && owner_is_source
                && event_player_alive
                && (state.hand_count(ctx.player) > 0 || equip_count(ctx.player) > 0)
        }

        TriggerFilter::OwnerSourceOtherTargetAliveHasDiscardableHandEquip => {
            owner_alive
                && owner_is_source
                && event_player_alive
                && active.owner != ctx.player
                && !crate::query::discardable_target_cards(
                    state,
                    active.owner,
                    ctx.player,
                    rule_api::TargetCardScope::HandEquip,
                )
                .is_empty()
        }

        TriggerFilter::OwnerSourceShaTargetAliveOwnerOutsideTargetAttackRange => {
            owner_alive
                && owner_is_source
                && event_player_alive
                && ctx.kind == EventKind::CardTargeted
                && card_is_sha
                && !crate::query::player_in_attack_range(state, ctx.player, active.owner)
        }

        TriggerFilter::OwnerAliveFirstTargetVirtualOrConvertedCard => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && ctx.targets.first() == Some(&ctx.player)
                && ctx
                    .card
                    .and_then(|card| state.cards.get(card))
                    .is_some_and(|card| {
                        card.def.is_none()
                            || ctx.card_kind != Some(card.kind.tag())
                            || view_as_card == card.id.raw()
                    })
        }

        TriggerFilter::OwnerTargetedByOtherShaOncePerTurn => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_player
                && ctx.source != active.owner
                && card_is_sha
                && turn_fires == 0
        }

        TriggerFilter::OwnerTargetedByOtherCardOncePerTurn => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_player
                && ctx.source != active.owner
                && turn_fires == 0
        }

        TriggerFilter::OwnerTwfeifuSingleShaPartyTargetHasHandEquip => {
            let branch = state
                .skills
                .get(active.skill_id)
                .map_or(0, |skill| skill.view_as_branch.min(1));
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && card_is_sha
                && ctx.target_count == 1
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|target| target.alive)
                && (state.hand_count(ctx.player) > 0 || equip_count(ctx.player) > 0)
                && match branch {
                    0 => owner_is_player && ctx.source != active.owner,
                    _ => owner_is_source,
                }
        }

        TriggerFilter::OwnerTargetedByOtherSha => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_player
                && !owner_is_source
                && card_is_sha
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

        TriggerFilter::OwnerCardActionCountEqualsAttackRange => {
            if !owner_alive
                || !owner_is_source
                || !matches!(ctx.kind, EventKind::CardUsed | EventKind::CardResponded)
            {
                return false;
            }
            let next_count = history
                .map(|snapshot| snapshot.owner_card_action_count)
                .unwrap_or_else(|| {
                    state
                        .turn
                        .flag(turn_flag::card_action_count_by_player(active.owner))
                })
                .saturating_add(1);
            matches!(
                query(
                    state,
                    &QueryRequest::AttackRange {
                        player: active.owner,
                    },
                    &mut QueryContext::new(),
                ),
                QueryResponse::Int(range) if next_count == range
            )
        }

        TriggerFilter::OwnerLifetimeCardAction => {
            owner_alive
                && owner_is_source
                && matches!(ctx.kind, EventKind::CardUsed | EventKind::CardResponded)
        }

        TriggerFilter::OwnerAliveOtherShanOutsideTurnGroupOncePerTurn(group) => {
            owner_alive
                && event_player_alive
                && matches!(ctx.kind, EventKind::CardUsed | EventKind::CardResponded)
                && ctx.player != active.owner
                && ctx.player != state.turn.current
                && ctx.card_kind == Some(CardKind::Shan.tag())
                && event_player_group.is_some_and(|event_group| event_group == group)
                && turn_fires == 0
        }

        TriggerFilter::OwnerAliveOtherDiscardedBasicOutsideTurn => {
            owner_alive_other_discarded_basic_outside_turn
        }

        TriggerFilter::OwnerAliveOtherDiscardedBasicOutsideTurnOncePerTurn => {
            owner_alive_other_discarded_basic_outside_turn && turn_fires == 0
        }

        TriggerFilter::OwnerFirstHandDiscardHasShaInDiscard => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            let prior_hand_discards = history
                .map(|snapshot| snapshot.event_player_hand_discard_events)
                .unwrap_or_else(|| {
                    state
                        .turn
                        .flag(turn_flag::hand_discard_event_by_player(active.owner))
                        .max(0) as u32
                });
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardsMoved
                && ctx.from_zone == Some(ZoneKind::Hand)
                && ctx.move_reason == CardMoveReason::Discard
                && ctx.to_zone == discard
                && prior_hand_discards == 0
                && ctx.cards.iter().any(|card| {
                    state.cards.get(*card).is_some_and(|card_state| {
                        Some(card_state.zone) == discard
                            && crate::query::effective_card_kind(state, active.owner, card_state)
                                .is_sha()
                    })
                })
        }

        TriggerFilter::OwnerAliveDealtOrReceivedDamageThisTurn => {
            owner_dealt_or_received_damage_this_turn
        }

        TriggerFilter::OwnerReceivedGiftFromOther => owner_received_gift_from_other,

        TriggerFilter::OwnerInitiatedDuelTargetAliveLoserHasHandOrEquip => {
            let original_target_alive = ctx
                .targets
                .first()
                .and_then(|target| state.players.get(*target))
                .is_some_and(|player| player.alive);
            let loser_has_card = [ZoneKind::Hand, ZoneKind::Equip].into_iter().any(|kind| {
                state
                    .zones
                    .find(kind, ctx.source)
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| !zone.cards.is_empty())
            });
            owner_alive
                && ctx.kind == EventKind::DuelResolved
                && ctx.respond_to == active.owner
                && original_target_alive
                && ctx.player != ctx.source
                && loser_has_card
        }

        TriggerFilter::OwnerHasEnabledEquipSlot => {
            owner_alive
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|owner| owner.disabled_equip_slots.iter().any(|count| *count == 0))
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

        TriggerFilter::OwnerAliveOtherDyingHandNonemptyOncePerGame => {
            owner_alive
                && ctx.kind == EventKind::Dying
                && ctx.player != active.owner
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|target| target.alive && target.hp <= 0)
                && state.hand_count(active.owner) > 0
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.lifetime_fire_count == 0)
        }

        TriggerFilter::OwnerDyingOtherHandEquipNonemptyOncePerGame => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::Dying
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|owner| owner.hp <= 0)
                && state.players.alive_by_seat().into_iter().any(|target| {
                    target != active.owner
                        && [ZoneKind::Hand, ZoneKind::Equip].into_iter().any(|kind| {
                            state
                                .zones
                                .find(kind, target)
                                .and_then(|zone| state.zones.get(zone))
                                .is_some_and(|zone| !zone.cards.is_empty())
                        })
                })
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.lifetime_fire_count == 0)
        }

        TriggerFilter::OwnerAliveOtherDyingFirstPerTarget => {
            owner_alive
                && ctx.kind == EventKind::Dying
                && ctx.player != active.owner
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|target| target.alive && target.hp <= 0)
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| !skill.has_lifetime_target(ctx.player))
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

        TriggerFilter::OwnerSourceOrTargetEventPlayerSecondDamageThisTurn => {
            owner_alive
                && (owner_is_player || owner_is_source)
                && ctx.kind == EventKind::Damage
                && state.turn.damage_taken_count(ctx.player) == 2
        }

        TriggerFilter::OwnerSourceOrTargetEventPlayerFirstDamageThisTurn => {
            owner_alive
                && (owner_is_player || owner_is_source)
                && ctx.kind == EventKind::Damage
                && state.turn.damage_taken_count(ctx.player) == 1
        }

        TriggerFilter::OwnerSourceTargetDistanceLeOneFirstDamageToTargetThisTurn => {
            owner_source_distance_le_one
                && ctx.kind == EventKind::Damage
                && state.turn.damage_pair_count(active.owner, ctx.player) == 1
        }

        TriggerFilter::OwnerFirstTransformedShaUsedThisTurn => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardUsed
                && CardKind::from_tag(state.turn.flag(turn_flag::VIEW_AS_LOGICAL_KIND).max(0) as u8)
                    .is_some_and(CardKind::is_sha)
                && view_as_skill != 0
                && ctx.card.is_some_and(|card| {
                    state
                        .cards
                        .get(card)
                        .is_some_and(|card| card.def != engine_api::CardDefId::NONE)
                })
        }

        TriggerFilter::OwnerTargetHandEmptyElementalOrTrickDamageBefore => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::DamageBefore
                && hand_empty
                && (ctx.nature != 0 || card_is_ordinary_trick)
        }

        TriggerFilter::OwnerAliveDrawPlayerOwnerOrMinHpTwjiaohuaAvailable => {
            let target_is_min_hp = state
                .players
                .get(ctx.player)
                .filter(|target| target.alive)
                .is_some_and(|target| {
                    state
                        .players
                        .iter_live()
                        .filter(|other| other.alive)
                        .all(|other| target.hp <= other.hp)
                });
            let drawn_mask = ctx.cards.iter().fold(0u8, |mask, card| {
                mask | state.cards.get(*card).map_or(0, |card| {
                    if card.kind.is_basic() {
                        0b001
                    } else if card.kind.is_trick() {
                        0b010
                    } else if card.kind.is_equip() {
                        0b100
                    } else {
                        0
                    }
                })
            });
            let gained_mask =
                state
                    .turn
                    .flag(crate::state::turn_flag::twjiaohua_category_by_player(
                        ctx.player,
                    )) as u8;

            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::CardsDrawn
                && (owner_is_player || target_is_min_hp)
                && (drawn_mask | gained_mask) & 0b111 != 0b111
        }

        TriggerFilter::OwnerAliveShaOrDuelDamageSourceTargetAliveHandEquipNonempty => {
            owner_alive
                && ctx.kind == EventKind::Damage
                && source_alive
                && event_player_alive
                && (card_is_sha || card_is_juedou)
                && (state.hand_count(ctx.player) > 0
                    || equip_count(ctx.player) > 0
                    || state.hand_count(ctx.source) > 0
                    || equip_count(ctx.source) > 0)
        }

        TriggerFilter::OwnerAliveShaDamageSourceTargetAliveOwnerHandEquipNonempty => {
            owner_alive
                && ctx.kind == EventKind::Damage
                && card_is_sha
                && event_player_alive
                && ctx.source.is_some()
                && state.players.get(ctx.source).is_some()
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
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

        TriggerFilter::OwnerAliveGainedThreeThisTurnOtherAlive => {
            owner_alive
                && state.turn.card_gain_count(active.owner) >= 3
                && state
                    .players
                    .iter_live()
                    .any(|player| player.alive && player.id != active.owner)
        }

        TriggerFilter::OwnerAliveGainedTwoThisTurnCanCompareOther => {
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && state.turn.card_gain_count(active.owner) >= 2
                && state.hand_count(active.owner) > 0
                && state.players.iter_live().any(|player| {
                    player.alive && player.id != active.owner && state.hand_count(player.id) > 0
                })
        }

        TriggerFilter::OwnerAliveGaveCardsToOtherAlive => {
            owner_alive
                && ctx.kind == EventKind::CardsGiven
                && owner_is_source
                && event_player_alive
                && active.owner != ctx.player
                && ctx.amount > 0
        }

        TriggerFilter::OwnerAliveTargetedByOtherHiddenSkillPile(skill_def) => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_player
                && ctx.source != active.owner
                && !state
                    .hidden_skill_pile_cards(active.owner, *skill_def)
                    .is_empty()
        }

        TriggerFilter::OwnerIsEventPlayerHandNonemptyHiddenSkillPile(skill_def) => {
            owner_alive
                && owner_is_player
                && state.hand_count(active.owner) > 0
                && !state
                    .hidden_skill_pile_cards(active.owner, *skill_def)
                    .is_empty()
        }

        TriggerFilter::OwnerLostMatchingRevealedSkillPileSuit(skill_def) => {
            if !owner_alive
                || ctx.kind != EventKind::CardsMoved
                || !owner_is_player
                || !matches!(
                    ctx.from_zone,
                    Some(ZoneKind::Hand | ZoneKind::Equip | ZoneKind::Judge)
                )
            {
                false
            } else {
                let suits: Vec<_> = state
                    .revealed_skill_pile_cards(active.owner, *skill_def)
                    .into_iter()
                    .filter_map(|card| state.cards.get(card).map(|card| card.suit))
                    .collect();
                !suits.is_empty()
                    && ctx.cards.iter().any(|card| {
                        state
                            .cards
                            .get(*card)
                            .is_some_and(|card| suits.contains(&card.suit))
                    })
            }
        }

        TriggerFilter::OwnerAliveGroupCountAtMostTwo => {
            owner_alive && crate::query::alive_group_count(state) <= 2
        }

        TriggerFilter::OwnerAliveCurrentTurn => owner_alive && state.turn.current == active.owner,

        TriggerFilter::OwnerIsEventPlayerCurrentTurn => {
            owner_alive && owner_is_player && state.turn.current == active.owner
        }

        TriggerFilter::DebateOwnerIsInitiator => {
            owner_alive && ctx.kind == EventKind::DebateResolved && owner_is_player
        }

        TriggerFilter::DebateOwnerParticipated => {
            owner_alive
                && ctx.kind == EventKind::DebateResolved
                && (ctx.debate_red.contains(&active.owner)
                    || ctx.debate_black.contains(&active.owner))
        }

        TriggerFilter::OwnerDebateParticipant => {
            owner_alive
                && ctx.kind == EventKind::DebateStarted
                && ctx.targets.contains(&active.owner)
        }

        TriggerFilter::OwnerIsLord => {
            owner_alive
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|p| p.identity == crate::state::Identity::Lord)
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

        TriggerFilter::OwnerAliveTurnUsedAllSuits => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::TurnEnded
                && state.turn.distinct_used_card_suits(active.owner) >= 4
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

        TriggerFilter::OwnerAliveDamageBeforeEventTargetMarkPositive(mark) => {
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && state.players.get(ctx.player).is_some_and(|target| {
                    target.alive
                        && target
                            .marks
                            .iter()
                            .find(|(id, _)| id == mark)
                            .is_some_and(|(_, count)| *count > 0)
                })
        }

        TriggerFilter::OwnerAlivePhaseBeginEventTargetMarkPositive(mark) => {
            owner_alive
                && ctx.kind == EventKind::PhaseBegin
                && state.players.get(ctx.player).is_some_and(|target| {
                    target.alive
                        && target
                            .marks
                            .iter()
                            .find(|(id, _)| id == mark)
                            .is_some_and(|(_, count)| *count > 0)
                })
        }

        TriggerFilter::OwnerAlivePlayEndUsedCategoriesPositive => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhasePlayEnd
                && state
                    .turn
                    .flag(turn_flag::play_card_category_history_by_player(
                        active.owner,
                    ))
                    .count_ones()
                    > 0
        }

        TriggerFilter::OwnerGainedCardsDuringPlayPhaseWounded => {
            let own_play_phase =
                state.turn.current == active.owner && state.turn.phase == Phase::Play;
            let gained = match ctx.kind {
                EventKind::CardsDrawn => ctx.player == active.owner,
                EventKind::CardsMoved => {
                    ctx.player == active.owner
                        && ctx
                            .to_zone
                            .and_then(|zone| state.zones.get(zone))
                            .is_some_and(|zone| {
                                zone.owner == active.owner && zone.kind == ZoneKind::Hand
                            })
                }
                _ => false,
            };
            owner_alive
                && own_play_phase
                && gained
                && ctx.amount > 0
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|player| player.hp < player.max_hp as i32)
        }

        TriggerFilter::OwnerDiscardedOutsidePlayPhase => {
            let own_play_phase =
                state.turn.current == active.owner && state.turn.phase == Phase::Play;
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.player == active.owner
                && !own_play_phase
                && matches!(ctx.from_zone, Some(ZoneKind::Hand | ZoneKind::Equip))
                && ctx
                    .to_zone
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| zone.kind == ZoneKind::DiscardPile)
                && !ctx.cards.is_empty()
        }

        TriggerFilter::OwnerInMultitargetShaOrTrickTargets => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_player
                && ctx.target_count > 1
                && (card_is_sha || card_is_ordinary_trick)
        }

        TriggerFilter::OwnerTargetedByNanman => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_player
                && card_kind == Some(CardKind::TrickNanman)
        }
        _ => unreachable!("filter routed to a different predicate group"),
    }
}
