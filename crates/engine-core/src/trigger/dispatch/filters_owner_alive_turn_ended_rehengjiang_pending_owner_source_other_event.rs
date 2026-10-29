//! 触发过滤器谓词：source、other、event 族效果。

use super::*;

pub(super) fn filter_match(
    state: &crate::state::GameState,
    active: &ActiveTrigger,
    ctx: &TriggerContext,
    view_as_skill: u32,
    _view_as_card: u32,
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
    let _owner_hand_below_alive_count_capped_seven = owner_alive
        && state.hand_count(active.owner)
            < state
                .players
                .iter_live()
                .filter(|player| player.alive)
                .count()
                .min(7) as u32;
    let _owner_hand_below_alive_count_capped_eight = owner_alive
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
    let event_player_distance_le_owner_hp = state
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
    let _unique_max_hand = |player_id| {
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
    let _source_is_max_equip = state
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
    let play_phase_fires = state
        .skills
        .get(active.skill_id)
        .map(|s| s.play_phase_fire_count)
        .unwrap_or(0);
    let card_kind = ctx.card_kind.and_then(CardKind::from_tag);
    let card_is_ordinary_trick =
        card_kind.is_some_and(|kind| kind.is_trick() && !kind.is_delayed_trick());
    let card_is_sha = matches!(ctx.card_kind, Some(1 | 5 | 6));
    let owner_hand = state.hand_cards(active.owner);
    let owner_hand_basic_count = owner_hand
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
    let card_is_black = matches!(
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
    let repeats_previous_card_name = previous_card_kind
        .zip(card_kind)
        .zip(current_card_def)
        .is_some_and(|((previous_kind, current_kind), current_def)| {
            same_logical_card_name(previous_kind, previous_card_def, current_kind, current_def)
        });
    let event_players_same_sex = state
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
    let event_source_has_male_sex = state
        .players
        .get(ctx.source)
        .and_then(|player| state.content.character(player.character_def))
        .is_some_and(|character| matches!(character.sex.as_str(), "male" | "double"));
    let owner_has_discardable_equipment =
        crate::query::discardable_hand_equip_cards(state, active.owner)
            .into_iter()
            .any(|card| {
                state.cards.get(card).is_some_and(|card_state| {
                    crate::query::effective_card_kind(state, active.owner, card_state).is_equip()
                })
            });
    let owner_has_weapon = state
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
    let owner_event_player_same_group = state
        .player_group(active.owner)
        .is_some_and(|group| state.player_group(ctx.player) == Some(group));
    let owner_wounded = state
        .players
        .get(active.owner)
        .is_some_and(|owner| owner.alive && owner.hp < owner.max_hp as i32);
    let _owner_history_bit = active
        .owner
        .raw()
        .checked_sub(1)
        .and_then(|shift| 1_u32.checked_shl(shift));
    match &def.filter {
        TriggerFilter::OwnerSourceQiaojianExtraAvailable => {
            let category = card_kind.and_then(|kind| {
                if kind.is_sha() {
                    Some(1u8)
                } else if kind.is_trick() && !kind.is_delayed_trick() {
                    Some(2u8)
                } else {
                    None
                }
            });
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardUsed
                && state.skills.get(active.skill_id).is_some_and(|helper| {
                    helper.state_blob.len() == 6
                        && helper.temporary_until_turn_end_of == active.owner
                        && helper.state_blob[4] & 1 != 0
                        && category.is_some_and(|bit| helper.state_blob[5] & bit == 0)
                })
        }

        TriggerFilter::OwnerSourceWoundedExistsOncePerTurn => {
            owner_alive
                && ctx.kind == EventKind::Damage
                && owner_is_source
                && turn_fires == 0
                && state
                    .players
                    .iter_live()
                    .any(|target| target.alive && target.hp < target.max_hp as i32)
        }

        TriggerFilter::OwnerSourceFirstDamageCardTargetHandGteOwner => {
            let damage_tagged = ctx.card.is_some_and(|card| {
                state.cards.get(card).is_some_and(|value| {
                    let kind = crate::query::effective_card_kind(state, active.owner, value);
                    kind.is_sha() || kind.is_damage_trick()
                })
            });
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardTargeted
                && ctx.targets.first() == Some(&ctx.player)
                && damage_tagged
                && ctx.targets.iter().any(|target| {
                    *target != active.owner
                        && state
                            .players
                            .get(*target)
                            .is_some_and(|participant| participant.alive)
                        && state.hand_count(*target) >= state.hand_count(active.owner)
                        && !crate::query::discardable_hand_equip_cards(state, *target).is_empty()
                })
        }

        TriggerFilter::OwnerSourceShaResolvedTrackedPotceduanDealtDamage => {
            let tracked = state
                .skills
                .get(active.skill_id)
                .and_then(|skill| skill.state_blob.get(..4))
                .map(|bytes| CardId::new(u32::from_le_bytes(bytes.try_into().unwrap())));
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::ShaResolved
                && ctx.card_flags & rule_api::damage_card_flag::CARD_USE_DEALT_DAMAGE != 0
                && tracked.is_some_and(|tracked| ctx.card == Some(tracked))
        }

        TriggerFilter::OwnerSourceShaResolvedTrackedPotzhanlieDraw => {
            let tracked = state.skills.get(active.skill_id).and_then(|skill| {
                (skill.state_blob.len() >= 8 && skill.state_blob[3] != 0).then(|| {
                    CardId::new(u32::from_le_bytes(
                        skill.state_blob[4..8].try_into().unwrap(),
                    ))
                })
            });
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::ShaResolved
                && tracked.is_some_and(|tracked| ctx.card == Some(tracked))
        }

        TriggerFilter::OwnerSourceSingleTargetShaOrBlackOrdinaryTrickCurrentPlayPhaseUnused => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && owner_is_source
                && active.owner == state.turn.current
                && state.turn.phase == Phase::Play
                && ctx.target_count == 1
                && (card_is_sha || (card_is_ordinary_trick && card_is_black))
                && play_phase_fires == 0
        }

        TriggerFilter::OwnerIsEventPlayerMaleSourceHasDiscardableEquipment => {
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && owner_is_player
                && ctx.source.is_some()
                && event_source_has_male_sex
                && owner_has_discardable_equipment
        }

        TriggerFilter::OwnerAliveOtherTurnEndedNoCardTargetedOther => {
            owner_alive
                && ctx.kind == EventKind::TurnEnded
                && ctx.player != active.owner
                && event_player_alive
                && ctx.card_flags & 1 == 0
        }

        TriggerFilter::OwnerAliveOtherTurnEndedNoDamageOther => {
            owner_alive
                && ctx.kind == EventKind::TurnEnded
                && ctx.player != active.owner
                && event_player_alive
                && ctx.card_flags & 2 == 0
        }

        TriggerFilter::OwnerAliveOtherTurnEndedCardTargetedOtherNoDamageOther => {
            owner_alive
                && ctx.kind == EventKind::TurnEnded
                && ctx.player != active.owner
                && event_player_alive
                && ctx.card_flags & 1 != 0
                && ctx.card_flags & 2 == 0
        }

        TriggerFilter::OwnerSourceShaHasWeapon => {
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && owner_is_source
                && event_player_alive
                && card_is_sha
                && ctx.card_flags & rule_api::damage_card_flag::CHAIN_PROPAGATED == 0
                && owner_has_weapon
        }

        TriggerFilter::OwnerSourceShaDirectDamageBefore => {
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && owner_is_source
                && event_player_alive
                && card_is_sha
                && ctx.card_flags & rule_api::damage_card_flag::CHAIN_PROPAGATED == 0
        }

        TriggerFilter::OwnerSourceOtherDamageBefore => {
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && owner_is_source
                && event_player_alive
                && active.owner != ctx.player
        }

        TriggerFilter::OwnerSourceOtherDamageBeforeOncePerTurn => {
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && owner_is_source
                && event_player_alive
                && active.owner != ctx.player
                && turn_fires == 0
        }

        TriggerFilter::OwnerSourceTargetEarlierSeatHandBelowTargetMaxHpOncePerTurn => {
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && owner_is_source
                && event_player_alive
                && active.owner != ctx.player
                && turn_fires == 0
                && state.players.get(active.owner).is_some_and(|owner| {
                    state.players.get(ctx.player).is_some_and(|target| {
                        owner.seat.raw() > target.seat.raw()
                            && state.hand_count(active.owner) < target.max_hp
                    })
                })
        }

        TriggerFilter::OwnerSourceOtherDirectDamageBefore => {
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && owner_is_source
                && event_player_alive
                && active.owner != ctx.player
                && ctx.card_flags & rule_api::damage_card_flag::CHAIN_PROPAGATED == 0
        }

        TriggerFilter::OwnerSourceOrTargetHandCountDiffersOncePerTurn => {
            owner_alive
                && ctx.kind == EventKind::Damage
                && (owner_is_source || owner_is_player)
                && turn_fires == 0
                && state
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .any(|target| state.hand_count(target) != state.hand_count(active.owner))
        }

        TriggerFilter::OwnerIsEventPlayerAnyAliveHandNonempty => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhasePrepare
                && state
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .any(|target| state.hand_count(target) > 0)
        }

        TriggerFilter::OwnerIsEventPlayerAnyAliveHandEmpty => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseEnd
                && state
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .any(|target| state.hand_count(target) == 0)
        }

        TriggerFilter::OwnerIsEventPlayerOtherHandLteOwner => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseEnd
                && state.players.alive_by_seat().into_iter().any(|target| {
                    target != active.owner
                        && state.hand_count(target) <= state.hand_count(active.owner)
                })
        }

        TriggerFilter::OwnerIsEventPlayerDrawReshuffleZeroWounded => {
            owner_alive
                && owner_is_player
                && owner_wounded
                && ctx.kind == EventKind::PhasePrepare
                && state.draw_reshuffle_count == 0
        }

        TriggerFilter::OwnerIsEventPlayerDrawReshufflePositive => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhasePrepare
                && state.draw_reshuffle_count > 0
        }

        TriggerFilter::OwnerAliveOtherJudgePhaseInRangeHasJudgeCard => {
            if ctx.kind != EventKind::PhaseJudge
                || !owner_alive
                || !event_player_alive
                || owner_is_player
                || crate::query::discardable_target_cards(
                    state,
                    active.owner,
                    ctx.player,
                    rule_api::TargetCardScope::Judge,
                )
                .is_empty()
            {
                return false;
            }
            let distance = query(
                state,
                &QueryRequest::Distance {
                    from: active.owner,
                    to: ctx.player,
                },
                &mut QueryContext::new(),
            );
            let range = query(
                state,
                &QueryRequest::AttackRange {
                    player: active.owner,
                },
                &mut QueryContext::new(),
            );
            matches!(
                (distance, range),
                (QueryResponse::Int(distance), QueryResponse::Int(range)) if distance <= range
            )
        }

        TriggerFilter::OwnerSourceNonEquipNoBasicHandHasTargets => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && owner_is_source
                && ctx.target_count > 0
                && card_kind.is_some_and(|kind| !kind.is_equip())
                && owner_hand_basic_count == 0
        }

        TriggerFilter::OwnerAliveSourceAliveOncePerRound => owner_alive && source_alive,

        TriggerFilter::OwnerAliveSourceAliveEventPlayerDyingOncePerRound => {
            owner_alive
                && source_alive
                && ctx.kind == EventKind::Dying
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|target| target.alive && target.dying && target.hp <= 0)
        }

        TriggerFilter::OwnerIsEventPlayerDiscardableHandEquipGteTwoOncePerRound => {
            owner_alive
                && owner_is_player
                && crate::query::discardable_hand_equip_cards(state, active.owner).len() >= 2
        }

        TriggerFilter::OwnerIsEventPlayerOtherSourceAliveHandEquipNonemptyOldHuishengUnused => {
            owner_alive
                && owner_is_player
                && source_alive
                && ctx.source != active.owner
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| !skill.oldhuisheng_sources().contains(&ctx.source))
        }

        TriggerFilter::OwnerAliveOtherDyingOutsideTurnCanShaCurrent => {
            owner_alive
                && ctx.kind == EventKind::Dying
                && ctx.player != active.owner
                && state.turn.current != active.owner
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|player| player.alive && player.dying && player.hp <= 0)
                && !crate::command::play::oldjiefan_sha_source_cards(
                    state,
                    active.owner,
                    state.turn.current,
                )
                .is_empty()
        }

        TriggerFilter::OwnerSourceOldJiefanSha => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::DamageBefore
                && card_is_sha
                && ctx.card_flags & rule_api::damage_card_flag::OLDJIEFAN != 0
                && state
                    .skills
                    .get(active.skill_id)
                    .and_then(|skill| skill.oldjiefan_dying())
                    .is_some()
        }

        TriggerFilter::OwnerSourceCurrentTurnFirstTargetNonEquip => {
            owner_alive
                && owner_is_source
                && state.turn.current == active.owner
                && ctx.kind == EventKind::CardTargeted
                && !card_kind.is_some_and(CardKind::is_equip)
                && ctx.targets.first() == Some(&ctx.player)
        }

        TriggerFilter::OwnerSourceFaceDownViewAsChenweiAvailable => {
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
            let used_face_down = source.is_some_and(|(skill_def, skill_owner)| {
                skill_owner == active.owner
                    && state
                        .content
                        .view_as_defs_for_skill(skill_def)
                        .iter()
                        .any(|rule| rule.cost.as_deref() == Some("face_down_hand"))
            });
            let mode = state
                .skills
                .get(active.skill_id)
                .map_or(0, |skill| skill.view_as_branch);
            let branch_available = if mode == 0 {
                !state.hand_cards(active.owner).is_empty()
            } else {
                state.players.iter_live().any(|target| {
                    target.alive
                        && target.id != active.owner
                        && !crate::query::discardable_target_cards(
                            state,
                            active.owner,
                            target.id,
                            rule_api::TargetCardScope::HandEquip,
                        )
                        .is_empty()
                })
            };
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardUsed
                && used_face_down
                && branch_available
        }

        TriggerFilter::OwnerSourceShaOrJuedouSingleTargetFirstSameNameThisTurn => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardTargeted
                && ctx.target_count == 1
                && card_kind.is_some_and(|kind| {
                    if kind.is_sha() {
                        history
                            .map(|snapshot| snapshot.owner_sha_used)
                            .unwrap_or_else(|| {
                                state
                                    .turn
                                    .flag(turn_flag::sha_used_by_player(active.owner))
                                    .max(0) as u32
                            })
                            == 0
                    } else if kind == CardKind::TrickJuedou {
                        history
                            .map(|snapshot| snapshot.owner_juedou_used)
                            .unwrap_or_else(|| {
                                state
                                    .turn
                                    .flag(turn_flag::juedou_used_by_player(active.owner))
                                    .max(0) as u32
                            })
                            == 0
                    } else {
                        false
                    }
                })
        }

        TriggerFilter::OwnerSourceShaOrJuedouSingleTargetHandNonempty => {
            owner_alive
                && owner_is_source
                && event_player_alive
                && ctx.kind == EventKind::CardTargeted
                && ctx.target_count == 1
                && card_kind.is_some_and(|kind| kind.is_sha() || kind == CardKind::TrickJuedou)
                && state.hand_count(ctx.player) > 0
        }

        TriggerFilter::OwnerSourceShaDamageBeforeEndpointDiscardableHandEquip => {
            owner_alive
                && owner_is_source
                && event_player_alive
                && ctx.kind == EventKind::DamageBefore
                && card_is_sha
                && [active.owner, ctx.player].into_iter().any(|endpoint| {
                    !crate::query::discardable_target_cards(
                        state,
                        active.owner,
                        endpoint,
                        rule_api::TargetCardScope::HandEquip,
                    )
                    .is_empty()
                })
        }

        TriggerFilter::OwnerAliveEventPlayerAlive => owner_alive && event_player_alive,

        TriggerFilter::OwnerAliveEventPlayerEquipEmpty => {
            owner_alive && event_player_alive && equip_count(ctx.player) == 0
        }

        TriggerFilter::OwnerAliveEventPlayerFromEquipEmpty => {
            owner_alive
                && event_player_alive
                && ctx.amount > 0
                && matches!(ctx.from_zone, Some(ZoneKind::Equip))
                && equip_count(ctx.player) == 0
        }

        TriggerFilter::OwnerAliveEventPlayerHandEmpty => {
            owner_alive && event_player_alive && state.hand_count(ctx.player) == 0
        }

        TriggerFilter::OwnerAliveEventPlayerOutsideTurnFromHandEmpty => {
            owner_alive
                && event_player_alive
                && ctx.player != state.turn.current
                && ctx.amount > 0
                && matches!(ctx.from_zone, Some(ZoneKind::Hand))
                && state.hand_count(ctx.player) == 0
        }

        TriggerFilter::OwnerAliveEventPlayerOutsideTurnFromHandEmptyOncePerTurn => {
            owner_alive
                && event_player_alive
                && ctx.player != state.turn.current
                && ctx.amount > 0
                && matches!(ctx.from_zone, Some(ZoneKind::Hand))
                && state.hand_count(ctx.player) == 0
                && turn_fires == 0
        }

        TriggerFilter::OwnerAliveEventPlayerDistanceLeOwnerHp => event_player_distance_le_owner_hp,

        TriggerFilter::OwnerAliveOtherNormalDamageTargetUnlinked => {
            owner_alive
                && event_player_alive
                && !owner_is_player
                && ctx.kind == EventKind::DamageBefore
                && ctx.nature == rule_api::damage_nature::NORMAL
                && state.players.get(ctx.player).is_some_and(|player| {
                    !player
                        .marks
                        .iter()
                        .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
                })
        }

        TriggerFilter::OwnerAliveOtherCurrentSourceRepeatsCardName => {
            owner_alive
                && !owner_is_source
                && ctx.source == state.turn.current
                && repeats_previous_card_name
        }

        TriggerFilter::OwnerAliveSourceTargetSameSex => owner_alive && event_players_same_sex,

        TriggerFilter::OwnerAliveEventPlayerSameGroup => {
            owner_alive && owner_event_player_same_group
        }

        TriggerFilter::OwnerAliveEventPlayerSameGroupHandNonempty => {
            owner_alive && owner_event_player_same_group && state.hand_count(active.owner) > 0
        }

        TriggerFilter::OwnerAliveOtherPlayPhaseTiedMaxHandDcsbfuxiAvailable => {
            let event_hand = state.hand_count(ctx.player);
            let max_hand = state
                .players
                .alive_by_seat()
                .into_iter()
                .map(|player| state.hand_count(player))
                .max()
                .unwrap_or(0);
            let owner_cards = !state.hand_cards(active.owner).is_empty()
                || state
                    .zones
                    .find(ZoneKind::Equip, active.owner)
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| !zone.cards.is_empty());
            let target_cards = !crate::query::discardable_target_cards(
                state,
                active.owner,
                ctx.player,
                rule_api::TargetCardScope::HandEquip,
            )
            .is_empty();
            owner_alive
                && ctx.kind == EventKind::PhasePlay
                && ctx.player != active.owner
                && event_player_alive
                && event_hand == max_hand
                && (owner_cards
                    || target_cards
                    || crate::command::play::source_free_sha_can_target_without_distance(
                        state,
                        active.owner,
                        ctx.player,
                    ))
        }

        TriggerFilter::OwnerSourceFireShaOrHuogongDcsbronghuo => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardUsed
                && ctx.card.is_some_and(|card| {
                    state.cards.get(card).is_some_and(|card_state| {
                        let kind =
                            crate::query::effective_card_kind(state, active.owner, card_state);
                        kind == CardKind::TrickHuogong
                            || kind.is_sha()
                                && crate::query::effective_card_nature(
                                    state,
                                    active.owner,
                                    card_state,
                                    kind,
                                ) == rule_api::damage_nature::FIRE
                    })
                })
        }

        TriggerFilter::OwnerSourceCardResolvedHasOtherTargetDcsbyingmouOncePerTurn => {
            let other_targets: Vec<_> = ctx
                .targets
                .iter()
                .copied()
                .filter(|target| {
                    *target != active.owner
                        && state
                            .players
                            .get(*target)
                            .is_some_and(|participant| participant.alive)
                })
                .collect();
            let yin = state
                .skills
                .get(active.skill_id)
                .is_some_and(|instance| instance.view_as_branch.min(1) == 1);
            let has_yin_pair = !yin
                || other_targets.iter().any(|target| {
                    let max_hand = state
                        .players
                        .alive_by_seat()
                        .into_iter()
                        .map(|player| state.hand_count(player))
                        .max()
                        .unwrap_or(0);
                    state
                        .players
                        .alive_by_seat()
                        .into_iter()
                        .any(|source| source != *target && state.hand_count(source) == max_hand)
                });
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardResolved
                && turn_fires == 0
                && !other_targets.is_empty()
                && has_yin_pair
        }

        TriggerFilter::OwnerSourceSingleTargetShaOrTrickAllMaterialsFromHandCanSha => {
            let target = ctx.targets.first().copied().unwrap_or(PlayerId::NONE);
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardResolved
                && ctx.target_count == 1
                && ctx.card_flags & 1 != 0
                && !ctx.cards.is_empty()
                && card_kind.is_some_and(|kind| kind.is_sha() || kind.is_trick())
                && crate::command::play::source_free_sha_can_target_without_distance(
                    state,
                    active.owner,
                    target,
                )
        }
        _ => unreachable!("filter routed to a different predicate group"),
    }
}
