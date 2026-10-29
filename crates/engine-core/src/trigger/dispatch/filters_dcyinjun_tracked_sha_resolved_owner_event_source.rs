//! 触发过滤器谓词：event、source 族效果。

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
    let owner_source_target_distance_gt_one = owner_is_source
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
    let owner_source_wounded_distance_le_one = owner_source_distance_le_one
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
    let owner_hand_below_alive_count_capped_seven = owner_alive
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
    let owner_is_min_hand = state
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
    let _card_is_trick = card_kind.is_some_and(CardKind::is_trick);
    let card_is_ordinary_trick =
        card_kind.is_some_and(|kind| kind.is_trick() && !kind.is_delayed_trick());
    // The modelled ordinary tricks have no upstream `multitarget` property; selectTarget alone does not make them multitarget.
    let _card_is_non_multitarget_trick =
        card_kind.is_some_and(|kind| kind.is_trick() && !kind.is_delayed_trick());
    let card_is_sha = matches!(ctx.card_kind, Some(1 | 5 | 6));
    let single_sha_source_kind = ctx
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
    let _owner_equip_empty = state
        .zones
        .find(ZoneKind::Equip, active.owner)
        .and_then(|zone| state.zones.get(zone))
        .is_none_or(|zone| zone.cards.is_empty());
    let _owner_hand_below_hp = state
        .players
        .get(active.owner)
        .is_some_and(|player| state.hand_count(active.owner) < player.hp.max(0) as u32);
    let owner_hand_above_hp = state
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
    let _card_is_black = matches!(
        card_suit,
        Some(engine_api::Suit::Spade | engine_api::Suit::Club)
    );
    let _judgment_is_black = ctx.kind == EventKind::JudgmentResolved
        && ctx.cards.first().is_some_and(|result_card| {
            state.cards.get(*result_card).is_some_and(|card| {
                matches!(
                    crate::query::effective_card_suit(state, ctx.player, card),
                    engine_api::Suit::Spade | engine_api::Suit::Club
                )
            })
        });
    let card_is_diamond = card_suit == Some(engine_api::Suit::Diamond);
    let card_is_club = card_suit == Some(engine_api::Suit::Club);
    let event_players_equal_hp = state
        .players
        .get(ctx.source)
        .zip(state.players.get(ctx.player))
        .is_some_and(|(source, target)| source.hp == target.hp);
    let event_player_has_female_sex = state
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
    let _owner_damaged_hp_le_source = state
        .players
        .get(active.owner)
        .zip(state.players.get(ctx.source))
        .is_some_and(|(owner, source)| owner.hp < owner.max_hp as i32 && owner.hp <= source.hp);
    match &def.filter {
        TriggerFilter::OwnerSourceOwnViewAsCardTargetedTargetSkillPile(skill_def) => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardTargeted
                && view_as_skill == active.skill_id.raw()
                && !state.skill_pile_cards(ctx.player, *skill_def).is_empty()
        }

        TriggerFilter::OwnerSourceShaDamageBeforeTargetHealthy => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::DamageBefore
                && event_player_alive
                && card_kind.is_some_and(|kind| kind.is_sha())
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|target| target.hp > 0 && target.hp == target.max_hp as i32)
        }

        TriggerFilter::OwnerAliveEventCardsNonempty => owner_alive && !ctx.cards.is_empty(),

        TriggerFilter::OwnerAliveEventPlayerGroup(group) => {
            owner_alive
                && event_player_alive
                && event_player_group.is_some_and(|event_group| event_group == group)
        }

        TriggerFilter::OwnerIsEventPlayerNoClanLostHandThisTurn(clan) => {
            // Clan 中流 (clanzhongliu): owner is the event player, and no OTHER living member
            // of `clan` lost hand cards this turn. Per-turn approximation of upstream's
            // per-useEvent `relatedEvent` correlation (the engine tracks per-turn card loss,
            // not per-event-parent loss; see docs/migration/upstream-1.11.5-delta.md).
            owner_is_player
                && owner_alive
                && !state.players.iter_live().any(|other| {
                    other.id != active.owner
                        && state.player_has_clan(other.id, clan)
                        && state.turn.card_loss_count(other.id) > 0
                })
        }

        TriggerFilter::OwnerAliveEventPlayerSelfOrClan(clan) => {
            // 保族 (clanbaozu): the dying/event player is the owner or a member of `clan`.
            // The upstream hp<=0 gate is inherent to the dying event; the "not linked" gate
            // is expressed as a body-side mark guard in the DSL.
            owner_alive
                && event_player_alive
                && (ctx.player == active.owner || state.player_has_clan(ctx.player, clan))
        }

        TriggerFilter::OwnerIsEventPlayerWounded => owner_is_player && owner_wounded,

        TriggerFilter::OwnerIsEventPlayerLinked => {
            owner_alive
                && owner_is_player
                && state.players.get(active.owner).is_some_and(|owner| {
                    owner
                        .marks
                        .iter()
                        .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
                })
        }

        TriggerFilter::OwnerIsEventPlayerAliveWoundedOrGuimingWuExists => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseDraw
                && crate::query::alive_wounded_or_guiming_wu_count(state, active.owner) > 0
        }

        TriggerFilter::OwnerSourceShaOrOrdinaryTrickTargetWoundedHasDiscardableHandEquip => {
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_source
                && (card_is_sha || card_is_ordinary_trick)
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|target| target.hp < target.max_hp as i32)
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
        }

        TriggerFilter::OwnerSourceShaOrOrdinaryTrickHasDiscardableHandEquip => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && owner_is_source
                && (card_is_sha || card_is_ordinary_trick)
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
        }

        TriggerFilter::OwnerSourceShaOrOrdinaryTrick => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && owner_is_source
                && (card_is_sha || card_is_ordinary_trick)
        }

        TriggerFilter::OwnerSourceWuxieUsed => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && owner_is_source
                && card_kind == Some(CardKind::TrickWuxie)
                && turn_fires == 0
        }

        TriggerFilter::OwnerSourceShaOncePerTurn => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_source
                && card_is_sha
                && turn_fires == 0
        }

        TriggerFilter::OwnerSourceShaRepeatsPreviousShaTargets => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && owner_is_source
                && card_is_sha
                && turn_fires == 0
                && {
                    let previous = state.turn.flag(turn_flag::LAST_SHA_TARGETS).max(0) as u32;
                    previous != 0
                        && !ctx.targets.is_empty()
                        && ctx.targets.iter().any(|target| {
                            target
                                .raw()
                                .checked_sub(1)
                                .and_then(|shift| 1_u32.checked_shl(shift))
                                .is_some_and(|bit| previous & bit != 0)
                        })
                }
        }

        TriggerFilter::OwnerSourcePlayCardRepeatsPreviousTargets => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && owner_is_source
                && active.owner == state.turn.current
                && state.turn.phase == Phase::Play
                && turn_fires < 3
                && {
                    let previous = state.turn.flag(turn_flag::LAST_CARD_TARGETS).max(0) as u32;
                    previous != 0
                        && !ctx.targets.is_empty()
                        && ctx.targets.iter().any(|target| {
                            target
                                .raw()
                                .checked_sub(1)
                                .and_then(|shift| 1_u32.checked_shl(shift))
                                .is_some_and(|bit| previous & bit != 0)
                        })
                }
        }

        TriggerFilter::OwnerIsEventPlayerHandEquipJudgeAboveHp => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhasePrepare
                && (state.hand_count(active.owner)
                    + state
                        .zones
                        .find(ZoneKind::Equip, active.owner)
                        .and_then(|id| state.zones.get(id))
                        .map_or(0, |zone| zone.cards.len() as u32)
                    + state
                        .zones
                        .find(ZoneKind::Judge, active.owner)
                        .and_then(|id| state.zones.get(id))
                        .map_or(0, |zone| zone.cards.len() as u32))
                    > state
                        .players
                        .get(active.owner)
                        .map_or(0, |participant| participant.hp.max(0) as u32)
        }

        TriggerFilter::OwnerSourceJiuBoostedShaDamageSkillEnabled(skill_def) => {
            owner_alive
                && ctx.kind == EventKind::Damage
                && owner_is_source
                && card_is_sha
                && ctx.card_flags & rule_api::damage_card_flag::JIU_BOOSTED != 0
                && state.skills.sorted_live().into_iter().any(|skill| {
                    skill.owner == active.owner
                        && skill.def == *skill_def
                        && state.skill_effectively_enabled(skill)
                })
        }

        TriggerFilter::OwnerAliveEventPlayerOwnerOrFemale => {
            owner_alive && event_player_alive && (owner_is_player || event_player_has_female_sex)
        }

        TriggerFilter::OwnerIsEventPlayerNotMinHp => {
            owner_is_player && owner_alive && !owner_is_min_hp
        }

        TriggerFilter::OwnerIsEventPlayerMinHand => {
            owner_alive && owner_is_player && owner_is_min_hand
        }

        TriggerFilter::OwnerIsEventPlayerNotMinHand => {
            owner_alive && owner_is_player && !owner_is_min_hand
        }

        TriggerFilter::OwnerIsEventPlayer => owner_is_player,

        TriggerFilter::OwnerAliveIsEventPlayerCurrentPlayPhase => {
            owner_alive
                && owner_is_player
                && state.turn.current == active.owner
                && state.turn.phase == Phase::Play
        }

        TriggerFilter::OwnerIsEventPlayerOtherAlive => {
            owner_is_player
                && state
                    .players
                    .iter_live()
                    .any(|player| player.alive && player.id != active.owner)
        }

        TriggerFilter::OwnerIsEventPlayerNotUniqueMaxCurrentHp => {
            owner_is_player
                && crate::query::unique_max_current_hp(state, active.owner) == Some(false)
        }

        TriggerFilter::OwnerIsEventPlayerHandNonempty => owner_is_player && !hand_empty,

        TriggerFilter::OwnerIsEventPlayerHpBelow(threshold) => {
            owner_is_player
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|owner| owner.alive && i64::from(owner.hp) < i64::from(*threshold))
        }

        TriggerFilter::OwnerIsEventPlayerMarkPositive(mark) => {
            owner_alive
                && owner_is_player
                && state.players.get(active.owner).is_some_and(|owner| {
                    owner
                        .marks
                        .iter()
                        .find(|(id, _)| id == mark)
                        .is_some_and(|(_, count)| *count > 0)
                })
        }

        TriggerFilter::OwnerSourceTargetMarkPositive(mark) => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_source
                && state.players.get(ctx.player).is_some_and(|target| {
                    target.alive
                        && target
                            .marks
                            .iter()
                            .find(|(id, _)| id == mark)
                            .is_some_and(|(_, count)| *count > 0)
                })
        }

        TriggerFilter::OwnerIsEventPlayerHandBelowMaxHp => {
            owner_alive && owner_is_player && owner_hand_below_max_hp
        }

        TriggerFilter::OwnerIsEventPlayerFirstHpChangeTurnUniqueTrickMajority => {
            let mut categories = [0_u32; 3];
            for card in state.hand_cards(active.owner) {
                if let Some(card) = state.cards.get(card) {
                    let kind = crate::query::effective_card_kind(state, active.owner, card);
                    let index = if kind.is_basic() {
                        Some(0)
                    } else if kind.is_trick() {
                        Some(1)
                    } else if kind.is_equip() {
                        Some(2)
                    } else {
                        None
                    };
                    if let Some(index) = index {
                        categories[index] = categories[index].saturating_add(1);
                    }
                }
            }
            owner_alive
                && owner_is_player
                && matches!(
                    ctx.kind,
                    EventKind::Damage
                        | EventKind::HpLost
                        | EventKind::HpRecovered
                        | EventKind::MaxHpChanged
                )
                && history.is_some_and(|snapshot| snapshot.event_player_hp_change_count == 1)
                && categories[1] > categories[0]
                && categories[1] > categories[2]
        }

        TriggerFilter::OwnerIsEventPlayerHandAboveHp => {
            owner_alive && owner_is_player && owner_hand_above_hp
        }

        TriggerFilter::OwnerIsEventPlayerHandAboveHpNoLiubeiLord => {
            let liubei_lord_exists = state.players.iter_live().any(|player| {
                player.alive
                    && player.identity == Identity::Lord
                    && state
                        .content
                        .character(player.character_def)
                        .is_some_and(|character| {
                            character
                                .name_key
                                .rsplit('_')
                                .next()
                                .is_some_and(|name| name.starts_with("liubei"))
                        })
            });
            owner_alive && owner_is_player && owner_hand_above_hp && !liubei_lord_exists
        }

        TriggerFilter::OwnerIsEventPlayerHandAtLeastHpOtherAlive => {
            owner_alive
                && owner_is_player
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|owner| state.hand_count(active.owner) >= owner.hp.max(0) as u32)
                && state
                    .players
                    .iter_live()
                    .any(|player| player.alive && player.id != active.owner)
        }

        TriggerFilter::OwnerIsEventPlayerLostCardsAtMostHpGainable => {
            let count = state.turn.card_loss_count(active.owner);
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            owner_alive
                && owner_is_player
                && count > 0
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|owner| count <= owner.hp.max(0) as u32)
                && state.turn.lost_cards_for(active.owner).any(|card| {
                    state
                        .cards
                        .get(card)
                        .is_some_and(|card| Some(card.zone) == discard)
                })
        }

        TriggerFilter::OwnerIsEventPlayerSourceMovableFieldCard => {
            owner_alive
                && owner_is_player
                && source_alive
                && !crate::query::movable_field_cards_to(state, ctx.source).is_empty()
        }

        TriggerFilter::OwnerIsEventPlayerNoTurnDamage => {
            owner_is_player
                && active.owner == state.turn.current
                && state.turn.flag(crate::state::turn_flag::DAMAGE_DEALT) == 0
        }

        TriggerFilter::OwnerIsEventPlayerTurnDamage => {
            owner_is_player
                && active.owner == state.turn.current
                && state.turn.flag(crate::state::turn_flag::DAMAGE_DEALT) > 0
        }

        TriggerFilter::OwnerIsEventPlayerTurnDamageAtLeastThree => {
            owner_is_player
                && active.owner == state.turn.current
                && state.turn.flag(crate::state::turn_flag::DAMAGE_DEALT) >= 3
        }

        TriggerFilter::OwnerIsEventPlayerCardUsedAtLeastHp => {
            owner_is_player
                && active.owner == state.turn.current
                && state.players.get(active.owner).is_some_and(|player| {
                    state.turn.flag(crate::state::turn_flag::CARD_USED) >= player.hp
                })
        }

        TriggerFilter::OwnerIsEventPlayerPlayPhaseCardCategoryCountPositive => {
            owner_alive
                && owner_is_player
                && active.owner == state.turn.current
                && state.turn.play_card_category_count(active.owner) > 0
        }

        TriggerFilter::OwnerIsEventPlayerActiveUsedAtLeastThisPlayPhase(threshold) => {
            owner_alive
                && owner_is_player
                && active.owner == state.turn.current
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.play_phase_use_count >= *threshold)
        }

        TriggerFilter::OwnerIsEventPlayerUseResponseDiscardedAtLeastHp => {
            owner_is_player
                && active.owner == state.turn.current
                && state.players.get(active.owner).is_some_and(|player| {
                    state
                        .turn
                        .flag(crate::state::turn_flag::USE_RESPONSE_DISCARDED)
                        >= player.hp
                })
        }

        TriggerFilter::OwnerIsEventPlayerDamagedOtherQun => {
            owner_is_player
                && active.owner == state.turn.current
                && state
                    .turn
                    .flag(crate::state::turn_flag::DAMAGE_DEALT_OTHER_QUN)
                    != 0
        }

        TriggerFilter::OwnerIsEventPlayerSourceAlive => owner_is_player && source_alive,

        TriggerFilter::OwnerIsEventPlayerSourceAliveOncePerTurn => {
            owner_is_player && source_alive && turn_fires == 0
        }

        TriggerFilter::OwnerIsEventPlayerSourceAliveDisabledWeapon => {
            owner_is_player
                && source_alive
                && state.players.get(ctx.source).is_some_and(|source| {
                    source.disabled_equip_slots[crate::state::EquipSlot::Weapon.index()] > 0
                })
        }

        TriggerFilter::OwnerIsEventPlayerSourceHasWeapon => {
            owner_is_player
                && ctx.source.is_some()
                && state
                    .zones
                    .find(ZoneKind::Equip, ctx.source)
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| {
                        zone.cards.iter().any(|card| {
                            state
                                .cards
                                .get(*card)
                                .and_then(|card| card.kind.equip_slot())
                                == Some(crate::state::EquipSlot::Weapon)
                        })
                    })
        }

        TriggerFilter::OwnerIsEventPlayerOtherSourceAlive => {
            owner_is_player && source_alive && !owner_is_source
        }

        TriggerFilter::OwnerIsEventPlayerSourceGroupQunDiscardableHandEquipGteTwo => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::Damage
                && event_source_group == Some("qun")
                && crate::query::discardable_hand_equip_cards(state, active.owner).len() >= 2
        }

        TriggerFilter::OwnerIsEventPlayerOtherSourceAlivePerDamagePoint => {
            ctx.amount > 0 && owner_is_player && source_alive && !owner_is_source
        }

        TriggerFilter::OwnerIsEventPlayerPerDamagePoint => ctx.amount > 0 && owner_is_player,

        TriggerFilter::OwnerIsEventPlayerPerDamagePointOtherHandEquipJudgeNonempty => {
            ctx.amount > 0
                && owner_alive
                && owner_is_player
                && state.players.iter_live().any(|player| {
                    player.alive
                        && player.id != active.owner
                        && [ZoneKind::Hand, ZoneKind::Equip, ZoneKind::Judge]
                            .into_iter()
                            .any(|kind| {
                                state
                                    .zones
                                    .find(kind, player.id)
                                    .and_then(|zone| state.zones.get(zone))
                                    .is_some_and(|zone| !zone.cards.is_empty())
                            })
                })
        }

        TriggerFilter::OwnerIsEventPlayerSourceAliveGainableCardPerDamagePoint => {
            ctx.amount > 0
                && owner_is_player
                && source_alive
                && if owner_is_source {
                    equip_count(ctx.source) > 0
                } else {
                    state.hand_count(ctx.source) > 0 || equip_count(ctx.source) > 0
                }
        }

        TriggerFilter::OwnerIsEventPlayerSourceAliveGainableCard => {
            ctx.amount > 0
                && owner_is_player
                && source_alive
                && if owner_is_source {
                    equip_count(ctx.source) > 0
                } else {
                    state.hand_count(ctx.source) > 0 || equip_count(ctx.source) > 0
                }
        }

        TriggerFilter::OwnerIsEventPlayerCardInProcessing => {
            owner_is_player
                && ctx.card.is_some_and(|card| {
                    state.cards.get(card).is_some_and(|card_state| {
                        state
                            .zones
                            .get(card_state.zone)
                            .is_some_and(|zone| zone.kind == ZoneKind::Processing)
                    })
                })
        }

        TriggerFilter::OwnerIsEventPlayerRedShaSourceAlive => {
            owner_is_player && source_alive && card_is_sha && card_is_red
        }

        TriggerFilter::OwnerIsEventPlayerNonRedSha => {
            owner_is_player && card_is_sha && !card_is_red
        }

        TriggerFilter::OwnerIsEventPlayerRedOrJiuSha => {
            owner_is_player
                && card_is_sha
                && (card_is_red || ctx.card_flags & rule_api::damage_card_flag::JIU_BOOSTED != 0)
        }

        TriggerFilter::OwnerIsEventPlayerShaSourceHasDiscardableHandEquip => {
            owner_is_player
                && ctx.source.is_some()
                && card_is_sha
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
        }

        TriggerFilter::OwnerIsEventPlayerRedCardSourceAlive => {
            owner_is_player
                && source_alive
                && (ctx.card.is_some() || ctx.card_kind.is_some())
                && card_is_red
        }

        TriggerFilter::OwnerIsEventPlayerNonRedCard => {
            owner_is_player && (ctx.card.is_some() || ctx.card_kind.is_some()) && !card_is_red
        }

        TriggerFilter::OwnerIsEventPlayerFirstDamagePerTurn => owner_is_player && turn_fires == 0,

        TriggerFilter::OwnerIsEventPlayerSecondDamagePerTurn => owner_is_player && turn_fires == 1,

        TriggerFilter::OwnerIsEventPlayerAfterFirstDamagePerTurn => {
            owner_is_player && turn_fires > 0
        }

        TriggerFilter::OwnerIsEventSource => owner_is_source,

        TriggerFilter::OwnerAliveSourceTargetWasLinked => {
            owner_alive
                && owner_is_source
                && ctx.card_flags & rule_api::damage_card_flag::TARGET_WAS_LINKED != 0
        }

        TriggerFilter::OwnerAliveIsEventSource => owner_alive && owner_is_source,

        TriggerFilter::OwnerSourceHandBelowAliveCountCappedSeven => {
            owner_is_source && owner_hand_below_alive_count_capped_seven
        }

        TriggerFilter::OwnerIsEventSourceOrPlayer => owner_is_source || owner_is_player,

        TriggerFilter::OwnerSourceOrTargetOtherAlivePerDamagePoint => {
            ctx.amount > 0
                && owner_alive
                && event_player_alive
                && source_alive
                && ctx.player != ctx.source
                && (owner_is_source || owner_is_player)
        }

        TriggerFilter::OwnerSourceHealthy => {
            owner_is_source
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|player| player.hp == player.max_hp as i32)
        }

        TriggerFilter::OwnerSourceAmountGtOne => owner_is_source && ctx.amount > 1,

        TriggerFilter::OwnerSourceFire => {
            owner_is_source && ctx.nature == rule_api::damage_nature::FIRE
        }

        TriggerFilter::OwnerSourceThunder => {
            owner_is_source && ctx.nature == rule_api::damage_nature::THUNDER
        }

        TriggerFilter::OwnerSourceOtherTwicePerTurn => {
            owner_is_source && !owner_is_player && turn_fires < 2
        }

        TriggerFilter::OwnerSourceWoundedDistanceLeOne => owner_source_wounded_distance_le_one,

        TriggerFilter::OwnerSourceDistanceLeOnePerDamagePoint => owner_source_distance_le_one,

        TriggerFilter::OwnerSourceTargetDistanceGtOne => owner_source_target_distance_gt_one,

        TriggerFilter::OwnerSourceTargetNotGroup(group) => {
            owner_is_source && event_player_group.is_some_and(|event_group| event_group != group)
        }

        TriggerFilter::OwnerSourceOrTargetedByJuedouOrRedSha => {
            owner_alive
                && (owner_is_source || owner_was_targeted)
                && (card_is_juedou || (card_is_sha && card_is_red))
        }

        TriggerFilter::OwnerSourceOrTargetedByJuedou => {
            owner_alive && (owner_is_source || owner_was_targeted) && card_is_juedou
        }

        TriggerFilter::OwnerSourceOrTargetedByShaEqualHp => {
            owner_alive
                && (owner_is_source || owner_is_player)
                && card_is_sha
                && event_players_equal_hp
        }

        TriggerFilter::OwnerSourceOrTargetedByOtherDiamond => {
            owner_alive
                && card_is_diamond
                && (owner_is_source || (owner_was_targeted && !owner_is_source))
        }

        TriggerFilter::OwnerSourceOrTargetedClubLinkedExistsOncePerTurn => {
            owner_alive
                && card_is_club
                && (owner_is_source || owner_was_targeted)
                && turn_fires == 0
                && state.players.iter_live().any(|player| {
                    player.alive
                        && player
                            .marks
                            .iter()
                            .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
                })
        }

        TriggerFilter::OwnerSourceShaSingleTrickSourceCard => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardUsed
                && single_sha_source_kind.is_some_and(CardKind::is_trick)
        }

        TriggerFilter::OwnerSourceShaSingleEquipSourceCard => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardUsed
                && single_sha_source_kind.is_some_and(CardKind::is_equip)
        }

        TriggerFilter::OwnerSourceRed => owner_alive && owner_is_source && card_is_red,

        _ => unreachable!("filter routed to a different predicate group"),
    }
}
