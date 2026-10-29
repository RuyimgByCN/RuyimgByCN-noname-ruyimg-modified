//! 触发过滤器谓词：owner 族效果。

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
    let _play_phase_fires = state
        .skills
        .get(active.skill_id)
        .map(|s| s.play_phase_fire_count)
        .unwrap_or(0);
    let card_kind = ctx.card_kind.and_then(CardKind::from_tag);
    let card_is_trick = card_kind.is_some_and(CardKind::is_trick);
    let _card_is_ordinary_trick =
        card_kind.is_some_and(|kind| kind.is_trick() && !kind.is_delayed_trick());
    // The modelled ordinary tricks have no upstream `multitarget` property; selectTarget alone does not make them multitarget.
    let card_is_non_multitarget_trick =
        card_kind.is_some_and(|kind| kind.is_trick() && !kind.is_delayed_trick());
    let card_is_sha = matches!(ctx.card_kind, Some(1 | 5 | 6));
    let _card_nature = ctx
        .card
        .and_then(|card| state.cards.get(card))
        .zip(card_kind)
        .map(|(card, kind)| crate::query::effective_card_nature(state, ctx.source, card, kind));
    let _card_is_juedou = ctx.card_kind == Some(CardKind::TrickJuedou.tag());
    let owner_hand = state.hand_cards(active.owner);
    let owner_hand_basic_count = owner_hand
        .iter()
        .filter(|card_id| {
            state.cards.get(**card_id).is_some_and(|card| {
                crate::query::effective_card_kind(state, active.owner, card).is_basic()
            })
        })
        .count();
    let _owner_hand_basic_equals_non_basic = owner_hand_basic_count * 2 == owner_hand.len();
    let suit_owner = if ctx.source.is_some() {
        ctx.source
    } else {
        ctx.player
    };
    let card_suit = ctx
        .card
        .and_then(|card| state.cards.get(card))
        .map(|card| crate::query::effective_card_suit(state, suit_owner, card));
    let _card_is_red = matches!(
        card_suit,
        Some(engine_api::Suit::Heart | engine_api::Suit::Diamond)
    );
    let _card_is_black = matches!(
        card_suit,
        Some(engine_api::Suit::Spade | engine_api::Suit::Club)
    );
    let previous_card_meta = history
        .map(|snapshot| snapshot.previous_card_meta)
        .unwrap_or_else(|| state.turn.flag(turn_flag::LAST_CARD_META) as u32);
    let _previous_suit = previous_card_meta & 0xff;
    let previous_play_card_meta = history
        .map(|snapshot| snapshot.previous_play_card_meta)
        .unwrap_or_else(|| state.turn.flag(turn_flag::LAST_PLAY_CARD_META) as u32);
    let _previous_play_suit = previous_play_card_meta & 0xff;
    let _previous_play_rank = (previous_play_card_meta >> 8) & 0xff;
    let previous_game_card_meta = history
        .map(|snapshot| snapshot.previous_game_card_meta)
        .unwrap_or_else(|| {
            state
                .turn
                .flag(turn_flag::last_game_card_meta_by_player(active.owner)) as u32
        });
    let _previous_game_suit = previous_game_card_meta & 0xff;
    let _previous_game_rank = (previous_game_card_meta >> 8) & 0xff;
    let _current_suit = card_suit.map(crate::command::judge::suit_tag);
    let _current_rank = ctx
        .card
        .and_then(|card| state.cards.get(card))
        .map(|card| u32::from(card.rank));
    let _suit_is_red = |suit| matches!(suit, 2 | 4);
    let owner_wounded = state
        .players
        .get(active.owner)
        .is_some_and(|owner| owner.alive && owner.hp < owner.max_hp as i32);
    match &def.filter {
        TriggerFilter::OwnerAliveSkillBoundTargetIsEventPlayer => {
            owner_alive
                && state
                    .skills
                    .get(active.skill_id)
                    .and_then(|skill| skill.bound_target())
                    == Some(ctx.player)
        }

        TriggerFilter::OwnerIsEventPlayerEventCardSha => owner_is_player && card_is_sha,

        TriggerFilter::OwnerIsEventPlayerEventCardShaOrTrick => {
            owner_is_player && (card_is_sha || card_is_trick)
        }

        TriggerFilter::OwnerTargetSha => owner_is_player && card_is_sha,

        TriggerFilter::OwnerAliveGainedThisTurnCanCompareOther => {
            let gained = state.turn.gained_hand_cards.iter().any(|(owner, card)| {
                *owner == active.owner && state.hand_cards(active.owner).contains(card)
            });
            let has_other = state.players.alive_by_seat().into_iter().any(|candidate| {
                candidate != active.owner && !state.hand_cards(candidate).is_empty()
            });
            owner_alive && gained && has_other
        }

        TriggerFilter::OwnerAliveCardTargetedMarkedPeer(mark_def) => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && ctx.source == state.turn.current
                && ctx.source != ctx.player
                && state
                    .players
                    .get(ctx.source)
                    .is_some_and(|p| p.marks.iter().any(|(m, n)| *m == *mark_def && *n > 0))
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|p| p.marks.iter().any(|(m, n)| *m == *mark_def && *n > 0))
        }

        TriggerFilter::OwnerIsEventPlayerHandShaSkillPileEmpty => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseEnd
                && state
                    .skill_pile_cards(active.owner, active.skill_def.raw())
                    .is_empty()
                && state.hand_cards(active.owner).into_iter().any(|card| {
                    state.cards.get(card).is_some_and(|card| {
                        crate::query::effective_card_kind(state, active.owner, card).is_sha()
                    })
                })
        }

        TriggerFilter::OwnerAliveDyingHasSkillPileCards => {
            owner_alive
                && ctx.kind == EventKind::Dying
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|player| player.alive && player.dying && player.hp <= 0)
                && !state
                    .skill_pile_cards(active.owner, active.skill_def.raw())
                    .is_empty()
        }

        TriggerFilter::OwnerIsEventPlayerSourceAliveMarkPositive(mark) => {
            owner_is_player
                && source_alive
                && state.players.get(active.owner).is_some_and(|owner| {
                    owner
                        .marks
                        .iter()
                        .find(|(found, _)| found == mark)
                        .is_some_and(|(_, count)| *count > 0)
                })
        }

        TriggerFilter::OwnerIsEventPlayerAnyFieldCardNoSkillPile(skill_def) => {
            owner_is_player
                && owner_alive
                && state.players.alive_by_seat().into_iter().any(|target| {
                    [ZoneKind::Hand, ZoneKind::Equip, ZoneKind::Judge]
                        .into_iter()
                        .filter_map(|kind| state.zones.find(kind, target))
                        .filter_map(|zone| state.zones.get(zone))
                        .any(|zone| !zone.cards.is_empty())
                })
                && state
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .all(|target| state.skill_pile_cards(target, *skill_def).is_empty())
        }

        TriggerFilter::OwnerAliveDyingSkillPile(skill_def) => {
            owner_alive
                && ctx.kind == EventKind::Dying
                && event_player_alive
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|target| target.dying && target.hp <= 0)
                && !state.skill_pile_cards(ctx.player, *skill_def).is_empty()
        }

        TriggerFilter::OwnerIsEventPlayerTwxiaolianRedirectedDamage => {
            owner_is_player
                && ctx.kind == EventKind::Damage
                && ctx.card.is_some_and(|card| {
                    state
                        .skills
                        .get(active.skill_id)
                        .and_then(|skill| skill.twxiaolian_redirect())
                        .is_some_and(|(remembered, _)| remembered == card)
                })
        }

        TriggerFilter::OwnerIsEventPlayerOtherHigherHandOncePerGame => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhasePrepare
                && state.players.alive_by_seat().into_iter().any(|player| {
                    player != active.owner
                        && state.hand_count(player) > state.hand_count(active.owner)
                })
        }

        TriggerFilter::OwnerIsEventPlayerHpOdd => {
            owner_is_player
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|owner| owner.alive && owner.hp % 2 != 0)
        }

        TriggerFilter::OwnerIsEventPlayerHpEvenWounded => {
            owner_is_player
                && owner_wounded
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|owner| owner.hp % 2 == 0)
        }

        TriggerFilter::OwnerAlivePreparePlayerHpGtOwner
        | TriggerFilter::OwnerAlivePrepareLordHpGtOwner => {
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::PhasePrepare
                && state
                    .players
                    .get(active.owner)
                    .zip(state.players.get(ctx.player))
                    .is_some_and(|(owner, event_player)| {
                        event_player.hp > owner.hp
                            && (!matches!(
                                def.filter,
                                TriggerFilter::OwnerAlivePrepareLordHpGtOwner
                            ) || event_player.identity == Identity::Lord)
                    })
        }

        TriggerFilter::OwnerShaTargetCanDiscardRedirectToOtherInAttackRange => {
            owner_alive
                && owner_is_player
                && source_alive
                && ctx.kind == EventKind::CardTargeted
                && card_kind == Some(CardKind::Sha)
                && ctx.source != active.owner
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
                && ctx
                    .card
                    .and_then(|card| state.cards.get(card))
                    .is_some_and(|card_state| {
                        state.players.iter_live().any(|candidate| {
                            candidate.id != active.owner
                                && candidate.id != ctx.source
                                && !ctx.targets.contains(&candidate.id)
                                && crate::query::player_in_attack_range(
                                    state,
                                    active.owner,
                                    candidate.id,
                                )
                                && !crate::query::card_target_cancelled_for_logical_kind(
                                    state,
                                    ctx.source,
                                    candidate.id,
                                    card_state,
                                    CardKind::Sha,
                                )
                        })
                    })
        }

        TriggerFilter::OwnerShaTargetCanDiscardRedirectToZhengjianMarked => {
            owner_alive
                && owner_is_player
                && source_alive
                && ctx.kind == EventKind::CardTargeted
                && card_kind == Some(CardKind::Sha)
                && ctx.source != active.owner
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
                && ctx
                    .card
                    .and_then(|card| state.cards.get(card))
                    .is_some_and(|card_state| {
                        state.players.iter_live().any(|candidate| {
                            candidate.id != active.owner
                                && candidate.id != ctx.source
                                && !ctx.targets.contains(&candidate.id)
                                && candidate.marks.iter().any(|(mark, count)| {
                                    *mark == crate::effect::ZHENGJIAN_MARK && *count > 0
                                })
                                && crate::query::player_in_attack_range(
                                    state,
                                    ctx.source,
                                    candidate.id,
                                )
                                && !crate::query::card_target_cancelled_for_logical_kind(
                                    state,
                                    ctx.source,
                                    candidate.id,
                                    card_state,
                                    CardKind::Sha,
                                )
                        })
                    })
        }

        TriggerFilter::OwnerIsTargetShaSourceAliveHandEquipNonemptyOncePerTurn => {
            owner_alive
                && owner_is_player
                && source_alive
                && ctx.kind == EventKind::CardTargeted
                && card_kind == Some(CardKind::Sha)
                && ctx.source != active.owner
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
                && turn_fires == 0
        }

        TriggerFilter::OwnerIsTargetTrickTargetsGtSourceHpHandEquipNonempty => {
            owner_alive
                && owner_is_player
                && source_alive
                && ctx.kind == EventKind::CardTargeted
                && card_kind.is_some_and(|kind| kind.is_trick())
                && !ctx.targets.is_empty()
                && ctx.targets.len()
                    > state
                        .players
                        .get(ctx.source)
                        .map(|source| source.hp.max(1))
                        .unwrap_or(1) as usize
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
        }

        TriggerFilter::OwnerIsResponderSha => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardResponded
                && card_kind == Some(CardKind::Sha)
        }

        TriggerFilter::OwnerIsTargetDamageAmountGtOne => {
            owner_alive && owner_is_player && ctx.kind == EventKind::DamageBefore && ctx.amount > 1
        }

        TriggerFilter::OwnerIsTargetSourceHpGtOwnerPerRoleTurn => {
            owner_alive
                && owner_is_player
                && source_alive
                && ctx.kind == EventKind::CardTargeted
                && ctx.source.ne(&active.owner)
                && state
                    .players
                    .get(ctx.source)
                    .zip(state.players.get(active.owner))
                    .is_some_and(|(src, own)| src.hp > own.hp)
                && turn_fires == 0
        }

        TriggerFilter::OwnerIsEventPlayerCanCompareOther => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhasePlay
                && state.hand_count(active.owner) > 0
                && state.players.iter_live().any(|player| {
                    player.alive && player.id != active.owner && state.hand_count(player.id) > 0
                })
        }

        TriggerFilter::OwnerHandBelowLostHpAfterRelevantChange => {
            let moved_from_owner = ctx.player == active.owner
                && matches!(
                    ctx.from_zone,
                    Some(ZoneKind::Hand | ZoneKind::Equip | ZoneKind::Judge)
                );
            let moved_to_owner = ctx
                .to_zone
                .and_then(|zone| state.zones.get(zone))
                .is_some_and(|zone| {
                    zone.owner == active.owner
                        && matches!(
                            zone.kind,
                            ZoneKind::Hand | ZoneKind::Equip | ZoneKind::Judge
                        )
                });
            let personal_cards_changed =
                ctx.kind == EventKind::CardsMoved && (moved_from_owner || moved_to_owner);
            let owner_vitals_changed = owner_is_player
                && matches!(
                    ctx.kind,
                    EventKind::Damage
                        | EventKind::HpLost
                        | EventKind::HpRecovered
                        | EventKind::MaxHpChanged
                );
            owner_alive
                && (personal_cards_changed || owner_vitals_changed)
                && state.players.get(active.owner).is_some_and(|owner| {
                    state.hand_count(active.owner)
                        < owner.max_hp.saturating_sub(owner.hp.max(0) as u32)
                })
        }

        TriggerFilter::OwnerAliveEventPlayerPlayPhaseSameSuitAtLeastTwoOncePerTurn => {
            owner_alive
                && event_player_alive
                && turn_fires == 0
                && history
                    .map(|snapshot| snapshot.event_player_play_phase_card_used)
                    .unwrap_or_else(|| state.turn.play_phase_card_use_count(ctx.player))
                    > 1
                && history
                    .map(|snapshot| snapshot.event_player_play_phase_suit_count)
                    .unwrap_or_else(|| state.turn.distinct_play_phase_card_suits(ctx.player))
                    == 1
        }

        TriggerFilter::OwnerIsEventPlayerDiscardPileFourSuitsBelowAliveCap => {
            let (round, count) = state
                .skills
                .get(active.skill_id)
                .and_then(|skill| {
                    (skill.state_blob.len() == 12 && &skill.state_blob[..4] == b"BTU1").then(|| {
                        (
                            u32::from_le_bytes(skill.state_blob[4..8].try_into().unwrap()),
                            u32::from_le_bytes(skill.state_blob[8..12].try_into().unwrap()),
                        )
                    })
                })
                .unwrap_or((0, 0));
            let count = if round == state.turn.round { count } else { 0 };
            owner_is_player
                && state.turn.flag(turn_flag::DISCARD_PILE_SUIT_MASK) & 0b1111 == 0b1111
                && count < state.players.alive_by_seat().len().min(3) as u32
        }

        TriggerFilter::OwnerIsEventPlayerOncePerTurn => owner_is_player && turn_fires == 0,

        TriggerFilter::OwnerAliveShaTargetDistanceLeOne => {
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::CardTargeted
                && card_is_sha
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
                )
        }

        TriggerFilter::OwnerAliveShaTargetDistanceLeOneHasEquipCard => {
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::CardTargeted
                && card_is_sha
                && crate::query::discardable_hand_equip_cards(state, active.owner)
                    .into_iter()
                    .any(|card| {
                        state
                            .cards
                            .get(card)
                            .is_some_and(|card| card.kind.is_equip())
                    })
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
                )
        }

        TriggerFilter::OwnerIsEventPlayerShaSourceAliveHandEquipNonempty => {
            owner_alive
                && owner_is_player
                && source_alive
                && card_is_sha
                && (state.hand_count(active.owner) > 0
                    || state
                        .zones
                        .find(ZoneKind::Equip, active.owner)
                        .and_then(|zone| state.zones.get(zone))
                        .is_some_and(|zone| !zone.cards.is_empty()))
        }

        TriggerFilter::OwnerAliveWdbFuchouPendingLiving => {
            owner_alive
                && state.skills.get(active.skill_id).is_some_and(|skill| {
                    skill.state_blob.starts_with(b"WFU1")
                        && (skill.state_blob.len() - 4) % 4 == 0
                        && skill.state_blob[4..].chunks_exact(4).any(|raw| {
                            let source = PlayerId::new(u32::from_le_bytes(raw.try_into().unwrap()));
                            state
                                .players
                                .get(source)
                                .is_some_and(|participant| participant.alive)
                        })
                })
        }

        TriggerFilter::OwnerAliveWuguRemainedInDiscard => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.move_reason == CardMoveReason::WuguRemained
                && !ctx.cards.is_empty()
                && discard.is_some_and(|discard| {
                    ctx.cards.iter().any(|card| {
                        state
                            .cards
                            .get(*card)
                            .is_some_and(|card| card.zone == discard)
                    })
                })
        }

        TriggerFilter::OwnerIsTargetNonMultitargetTrickMultipleTargets => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardTargeted
                && card_is_non_multitarget_trick
                && ctx.target_count >= 2
        }

        TriggerFilter::OwnerAliveHpChangedThisTurnEnteredDiscardAvailable => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && state.turn.hp_change_count(active.owner) > 0
                && state.turn.entered_discard_cards.iter().any(|card| {
                    state
                        .cards
                        .get(*card)
                        .is_some_and(|card| Some(card.zone) == discard)
                })
        }

        TriggerFilter::OwnerAliveDddDongchaPileNonempty => {
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && !state
                    .skill_pile_cards(active.owner, crate::effect::DDD_DONGCHA_DEF)
                    .is_empty()
        }

        TriggerFilter::OwnerPlayDddZhengjunHpAvailable => {
            let used = state
                .skills
                .get(active.skill_id)
                .map(|skill| skill.play_phase_category_mask)
                .unwrap_or(u8::MAX);
            owner_alive
                && event_player_alive
                && state.turn.current == active.owner
                && state.turn.phase == Phase::Play
                && used & DDD_ZHENGJUN_HP == 0
                && matches!(
                    ctx.kind,
                    EventKind::Damage | EventKind::HpLost | EventKind::HpRecovered
                )
                && state
                    .players
                    .get(ctx.player)
                    .zip(state.players.get(active.owner))
                    .is_some_and(|(changed, owner)| changed.hp == owner.hp)
        }

        TriggerFilter::OwnerPlayDddZhengjunCardsMovedAvailable => {
            if !owner_alive
                || ctx.kind != EventKind::CardsMoved
                || state.turn.current != active.owner
                || state.turn.phase != Phase::Play
            {
                false
            } else {
                let used = state
                    .skills
                    .get(active.skill_id)
                    .map(|skill| skill.play_phase_category_mask)
                    .unwrap_or(u8::MAX);
                let move_mask = ddd_zhengjun_move_mask(state, ctx);
                let destination = ctx
                    .to_zone
                    .and_then(|zone| state.zones.get(zone))
                    .map(|zone| zone.owner)
                    .unwrap_or(PlayerId::NONE);
                let affected = |from_bit: u8, to_bit: u8| {
                    let mut players = Vec::new();
                    if move_mask & from_bit != 0 && ctx.player.is_some() {
                        players.push(ctx.player);
                    }
                    if move_mask & to_bit != 0
                        && destination.is_some()
                        && !players.contains(&destination)
                    {
                        players.push(destination);
                    }
                    players
                };
                let hand = used & DDD_ZHENGJUN_HAND == 0
                    && affected(1, 2).into_iter().any(|changed| {
                        state
                            .players
                            .get(changed)
                            .is_some_and(|participant| participant.alive)
                            && state.hand_count(changed) == state.hand_count(active.owner)
                    });
                let equip = used & DDD_ZHENGJUN_EQUIP == 0
                    && affected(4, 8).into_iter().any(|changed| {
                        state
                            .players
                            .get(changed)
                            .is_some_and(|participant| participant.alive)
                            && equip_count(changed) == equip_count(active.owner)
                    });
                hand || equip
            }
        }

        TriggerFilter::OwnerAliveDddQiceReady => {
            owner_alive
                && ctx.kind == EventKind::PhaseBegin
                && active.owner == ctx.player
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.content_flag(0) == Some(false))
        }

        TriggerFilter::OwnerAliveDddFenjiAvailable => {
            let phase_matches = (ctx.kind == EventKind::PhaseDraw
                && state.turn.phase == Phase::Draw
                && state.turn.draw_phase_fixed_count().is_none())
                || (ctx.kind == EventKind::PhaseDiscard && state.turn.phase == Phase::Discard);
            let used_mask = state
                .skills
                .get(active.skill_id)
                .map_or(0, |skill| skill.content_flags);
            owner_alive
                && active.owner == ctx.player
                && state.turn.current == active.owner
                && phase_matches
                && !crate::command::play::source_free_ordinary_trick_kinds(
                    state,
                    active.owner,
                    used_mask,
                )
                .is_empty()
        }
        _ => unreachable!("filter routed to a different predicate group"),
    }
}
