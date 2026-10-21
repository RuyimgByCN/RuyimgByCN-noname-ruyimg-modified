//! Trigger predicates for `OwnerBlackCardMovedEquipJudge` and related requests.

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
    let suit_owner = if ctx.source.is_some() {
        ctx.source
    } else {
        ctx.player
    };
    let card_suit = ctx
        .card
        .and_then(|card| state.cards.get(card))
        .map(|card| crate::query::effective_card_suit(state, suit_owner, card));
    match &def.filter {
        TriggerFilter::OwnerBlackCardMovedEquipJudge => {
            let black = ctx.cards.iter().any(|c| {
                state.cards.get(*c).is_some_and(|card| {
                    matches!(card.suit, engine_api::Suit::Spade | engine_api::Suit::Club)
                })
            });
            let leaves_owner_zone = ctx.player == active.owner
                && matches!(ctx.from_zone, Some(ZoneKind::Equip) | Some(ZoneKind::Judge));
            let enters_owner_zone = ctx
                .to_zone
                .and_then(|z| state.zones.get(z))
                .is_some_and(|z| {
                    z.owner == active.owner && matches!(z.kind, ZoneKind::Equip | ZoneKind::Judge)
                });
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && black
                && (leaves_owner_zone || enters_owner_zone)
        }

        TriggerFilter::OwnerAliveDamageCardNanman => {
            owner_alive && ctx.kind == EventKind::Damage && card_kind == Some(CardKind::TrickNanman)
        }

        TriggerFilter::OwnerDiscardSuitNoOwnFaluMark(suit_tag, mark_def) => {
            let suit = match *suit_tag {
                1 => engine_api::Suit::Spade,
                2 => engine_api::Suit::Heart,
                3 => engine_api::Suit::Club,
                4 => engine_api::Suit::Diamond,
                _ => engine_api::Suit::None,
            };
            let owner_has_mark = state.players.get(active.owner).is_some_and(|p| {
                p.marks
                    .iter()
                    .any(|(found, count)| *found == *mark_def && *count > 0)
            });
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.player == active.owner
                && ctx
                    .cards
                    .iter()
                    .any(|c| state.cards.get(*c).is_some_and(|card| card.suit == suit))
                && ctx
                    .to_zone
                    .and_then(|z| state.zones.get(z))
                    .is_some_and(|z| z.kind == ZoneKind::DiscardPile)
                && !owner_has_mark
        }

        TriggerFilter::OwnerAliveOtherDyingHasEquipCard => {
            let has_equip_card = state
                .hand_cards(active.owner)
                .iter()
                .any(|card| state.cards.get(*card).is_some_and(|c| c.kind.is_equip()))
                || state
                    .zones
                    .find(ZoneKind::Equip, active.owner)
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| !zone.cards.is_empty());
            owner_alive
                && ctx.kind == EventKind::Dying
                && ctx.player != active.owner
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|target| target.alive && target.hp <= 0)
                && has_equip_card
        }

        TriggerFilter::OwnerSourceTrickResolvedOwnPlayPhaseOncePerTurn => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardResolved
                && card_is_ordinary_trick
                && state.turn.current == active.owner
                && state.turn.phase == Phase::Play
                && turn_fires == 0
        }

        TriggerFilter::OwnerTargetedSourceHpHandDifferOncePerTurn => {
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && owner_is_player
                && source_alive
                && state
                    .players
                    .get(ctx.source)
                    .zip(state.players.get(active.owner))
                    .is_some_and(|(source, target)| {
                        source.hp != target.hp
                            && state.hand_count(ctx.source) != state.hand_count(active.owner)
                    })
                && turn_fires == 0
        }

        TriggerFilter::OwnerSourceHandGaintagAbsent(tag) => {
            // natuxing 兴门: owner uses a card while holding no card carrying
            // the given gaintag.
            let has_tag_card = state.hand_cards(active.owner).iter().any(|card| {
                state
                    .content_card_tags
                    .get(card)
                    .is_some_and(|tags| tags.contains(tag))
            });
            owner_alive && owner_is_source && ctx.kind == EventKind::CardUsed && !has_tag_card
        }

        TriggerFilter::CardsMovedDiscardAtLeastThree => {
            ctx.kind == EventKind::CardsMoved
                && ctx
                    .to_zone
                    .and_then(|zone| state.zones.get(zone).map(|z| z.kind))
                    == Some(ZoneKind::DiscardPile)
                && ctx.move_reason == CardMoveReason::Discard
                && ctx.cards.len() >= 3
        }

        TriggerFilter::OwnerSourceDamageTargetMarkPositiveOnce(mark) => {
            let carries_mark = |player: PlayerId| {
                state.players.get(player).is_some_and(|target| {
                    target.alive
                        && target
                            .marks
                            .iter()
                            .find(|(id, _)| id == mark)
                            .is_some_and(|(_, count)| *count > 0)
                })
            };
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::Damage
                && turn_fires == 0
                && carries_mark(ctx.player)
        }

        TriggerFilter::OwnerAliveMarkedAliveAboveTwo(mark) => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhasePrepare
                && state
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .filter(|player| {
                        state
                            .players
                            .get(*player)
                            .is_some_and(|holder| holder.alive)
                            && state
                                .players
                                .get(*player)
                                .and_then(|holder| {
                                    holder
                                        .marks
                                        .iter()
                                        .find(|(id, _)| id == mark)
                                        .map(|(_, count)| *count)
                                })
                                .unwrap_or(0)
                                > 0
                    })
                    .count()
                    > 2
        }

        TriggerFilter::CardsMovedIntoExpansion => {
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.to_zone.is_some_and(|zone| {
                    state
                        .zones
                        .get(zone)
                        .is_some_and(|z| z.kind == ZoneKind::Expansion)
                })
        }

        TriggerFilter::CardsMovedOutOfExpansion => {
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.from_zone == Some(ZoneKind::Expansion)
        }

        TriggerFilter::OwnerAliveDiscardMatchesOwnPileCard(pile_def) => {
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx
                    .to_zone
                    .and_then(|zone| state.zones.get(zone).map(|z| z.kind))
                    == Some(ZoneKind::DiscardPile)
                && {
                    let pile = state.skill_pile_cards(active.owner, *pile_def);
                    !pile.is_empty()
                }
                && {
                    let pile_card = state.skill_pile_cards(active.owner, *pile_def)[0];
                    let Some(pile_state) = state.cards.get(pile_card) else {
                        return false;
                    };
                    ctx.cards.iter().any(|card| {
                        state
                            .cards
                            .get(*card)
                            .is_some_and(|c| c.suit == pile_state.suit && c.rank == pile_state.rank)
                    })
                }
        }

        TriggerFilter::OwnerAliveSomeoneMarkPositive(mark) => {
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && state.players.alive_by_seat().into_iter().any(|player| {
                    state
                        .players
                        .get(player)
                        .and_then(|holder| {
                            holder
                                .marks
                                .iter()
                                .find(|(id, _)| id == mark)
                                .map(|(_, count)| *count)
                        })
                        .unwrap_or(0)
                        > 0
                })
        }

        TriggerFilter::OwnerAliveShaResolvedCity(skill_def) => {
            let has_city = state.skill_piles.iter().any(|pile| {
                pile.owner == active.owner
                    && pile.skill_def == *skill_def
                    && state
                        .zones
                        .get(pile.zone)
                        .is_some_and(|zone| !zone.cards.is_empty())
            });
            let any_lost = state.players.alive_by_seat().into_iter().any(|player| {
                player != active.owner
                    && state
                        .turn
                        .flag(crate::state::turn_flag::lost_card_count_by_player(player))
                        > 0
            });
            owner_alive && ctx.kind == EventKind::ShaResolved && has_city && any_lost
        }

        TriggerFilter::OwnerAliveOtherTargetedBySha => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && card_is_sha
                && ctx.player != active.owner
        }

        TriggerFilter::OwnerAliveTargetedByOtherShaJudgeXumou => {
            let has_xumou = state
                .zones
                .find(ZoneKind::Judge, active.owner)
                .map(|judge| {
                    state
                        .zones
                        .get(judge)
                        .map(|zone| {
                            zone.cards.iter().rev().any(|card| {
                                state
                                    .content_card_tags
                                    .get(card)
                                    .is_some_and(|tags| tags.contains(&crate::content::XUMOU_TAG))
                            })
                        })
                        .unwrap_or(false)
                })
                .unwrap_or(false);
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_player
                && !owner_is_source
                && card_is_sha
                && has_xumou
        }

        TriggerFilter::OwnerCardsMovedContainYing => {
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.to_zone.map(|z| state.zones.get(z).map(|z| z.kind))
                    == Some(Some(ZoneKind::DiscardPile))
                && ctx.cards.iter().any(|card| {
                    state
                        .cards
                        .get(*card)
                        .is_some_and(|card| card.kind == crate::state::CardKind::Ying)
                })
        }

        TriggerFilter::OwnerAliveOtherSourceDamage => {
            owner_alive
                && ctx.kind == EventKind::Damage
                && ctx.source != PlayerId::NONE
                && ctx.source != active.owner
        }

        TriggerFilter::DebateOwnerInitiatorResultRed => {
            owner_alive
                && ctx.kind == EventKind::DebateResolved
                && owner_is_player
                && ctx.amount == 1
        }

        TriggerFilter::DebateOwnerInitiatorResultBlack => {
            owner_alive
                && ctx.kind == EventKind::DebateResolved
                && owner_is_player
                && ctx.amount == 2
        }

        TriggerFilter::DebateOwnerOpinionDiffersInitiator => {
            owner_alive && ctx.kind == EventKind::DebateResolved && {
                let owner_red = ctx.debate_red.contains(&active.owner);
                let owner_black = ctx.debate_black.contains(&active.owner);
                let init_red = ctx.debate_red.contains(&ctx.player);
                let init_black = ctx.debate_black.contains(&ctx.player);
                (owner_red || owner_black) && (init_red || init_black) && owner_red != init_red
            }
        }

        TriggerFilter::OwnerLostPackageCardFromEquip => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardsMoved
                && ctx.from_zone == Some(ZoneKind::Equip)
                && ctx.cards.iter().any(|card| {
                    state
                        .cards
                        .get(*card)
                        .and_then(|card| state.content.card_metadata(card.def))
                        .is_some_and(|card| card.on_leave_equip_skill_def == active.skill_def.raw())
                })
        }

        TriggerFilter::OwnerAliveSourceAllyShaDamageBefore => {
            let allied = matches!(
                query(
                    state,
                    &QueryRequest::Relation {
                        a: active.owner,
                        b: ctx.source,
                    },
                    &mut QueryContext::default(),
                ),
                QueryResponse::Relation(RelationTag::Ally | RelationTag::Self_)
            );
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && card_is_sha
                && ctx.source.is_some()
                && allied
        }

        TriggerFilter::OwnerIsEventPlayerWoundedDeadAlly => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhasePrepare
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|player| player.hp < player.max_hp as i32)
                && state.players.iter_live().any(|player| {
                    !player.alive
                        && matches!(
                            query(
                                state,
                                &QueryRequest::Relation {
                                    a: active.owner,
                                    b: player.id,
                                },
                                &mut QueryContext::default(),
                            ),
                            QueryResponse::Relation(RelationTag::Ally)
                        )
                })
        }

        TriggerFilter::OwnerIsEventPlayerPhaseEndHandEquipNonemptyOtherAlive => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseEnd
                && (state.hand_count(active.owner) > 0 || equip_count(active.owner) > 0)
                && state
                    .players
                    .iter_live()
                    .any(|player| player.alive && player.id != active.owner)
        }

        TriggerFilter::OwnerSourceShaDamageTargetAliveHasDiscardableEquip => {
            owner_alive
                && owner_is_source
                && event_player_alive
                && ctx.kind == EventKind::Damage
                && card_is_sha
                && !crate::query::discardable_equipment_cards(state, active.owner, ctx.player)
                    .is_empty()
        }

        TriggerFilter::OwnerSourceShaTargetAliveHasDiscardableEquip => {
            owner_alive
                && owner_is_source
                && event_player_alive
                && ctx.kind == EventKind::CardTargeted
                && card_is_sha
                && !crate::query::discardable_equipment_cards(state, active.owner, ctx.player)
                    .is_empty()
        }

        TriggerFilter::OwnerSourceCardUsedTargetsSelf => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardUsed
                && ctx.targets.contains(&active.owner)
        }

        TriggerFilter::OwnerSourceCurrentPhaseCardTargetedOtherHealthy => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardTargeted
                && active.owner == state.turn.current
                && ctx.player != active.owner
                && event_player_alive
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|target| target.hp.max(0) as u32 == target.max_hp)
        }

        TriggerFilter::OwnerSkillAddedSelf(skill_def) => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::SkillAdded
                && ctx.added_skill_def == *skill_def
        }

        TriggerFilter::OwnerSourceFirstCardSuitThisTurn => {
            let matched = owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardUsed
                && card_suit.is_some_and(|suit| {
                    let bit = match suit {
                        engine_api::Suit::Spade => 1,
                        engine_api::Suit::Heart => 1 << 1,
                        engine_api::Suit::Club => 1 << 2,
                        engine_api::Suit::Diamond => 1 << 3,
                        engine_api::Suit::None => return false,
                    };
                    // 使用事件时快照（remember_card_action 在计划后执行，快照不含当前牌）
                    let history_flag = history
                        .map(|snapshot| snapshot.owner_used_card_suit_history)
                        .unwrap_or_else(|| {
                            state
                                .turn
                                .flag(turn_flag::used_card_suit_history_by_player(active.owner))
                                .max(0) as u32
                        });
                    history_flag & bit == 0
                });
            matched
        }

        TriggerFilter::OwnerLostHandNowEmpty => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardsMoved
                && ctx.from_zone == Some(ZoneKind::Hand)
                && ctx.move_reason == CardMoveReason::Discard
                && !ctx.cards.is_empty()
                && state.hand_cards(active.owner).is_empty()
        }

        TriggerFilter::OwnerLostSpadeTwoToNine => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardsMoved
                && ctx.from_zone == Some(ZoneKind::Hand)
                && ctx.cards.iter().any(|card| {
                    state.cards.get(*card).is_some_and(|card_state| {
                        card_state.suit == engine_api::Suit::Spade
                            && (2..=9).contains(&card_state.rank)
                    })
                })
        }

        TriggerFilter::OwnerSourceCardDamageTargetCanReceiveOwnerBingliang => {
            owner_alive
                && owner_is_source
                && event_player_alive
                && ctx.kind == EventKind::Damage
                && ctx.card.is_some()
                && !crate::query::ddd_duanbing_transfer_cards(state, active.owner, ctx.player)
                    .is_empty()
        }

        TriggerFilter::OwnerSourceEquipUsedSkillMarkBelowSeven => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardUsed
                && card_kind.is_some_and(CardKind::is_equip)
                && state.players.get(active.owner).is_some_and(|owner| {
                    owner
                        .marks
                        .iter()
                        .find(|(mark, _)| *mark == active.skill_def.raw())
                        .map_or(0, |(_, count)| *count)
                        < 7
                })
        }

        TriggerFilter::OwnerSourceOutsideTurnCardUsedOrResponded => {
            owner_alive
                && owner_is_source
                && active.owner != state.turn.current
                && matches!(ctx.kind, EventKind::CardUsed | EventKind::CardResponded)
                && ctx.card.is_some()
                && card_kind.is_some()
        }

        TriggerFilter::OwnerAliveShanUsedByOwnerOrDuringOwnerTurn => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && card_kind == Some(CardKind::Shan)
                && (owner_is_source || state.turn.current == active.owner)
        }

        TriggerFilter::OwnerAliveOtherPhasePlaySkillPileNonempty => {
            owner_alive
                && event_player_alive
                && !owner_is_player
                && ctx.kind == EventKind::PhasePlay
                && !state
                    .skill_pile_cards(active.owner, active.skill_def.raw())
                    .is_empty()
        }

        TriggerFilter::OwnerAliveOtherSourceDamageCardResolvedTargetedDamaged
        | TriggerFilter::OwnerAliveOtherSourceDamageCardResolvedTargetedNotDamaged => {
            let damage_card = card_kind.is_some_and(|kind| kind.is_sha() || kind.is_damage_trick());
            let damaged = ctx
                .targets
                .iter()
                .position(|target| *target == active.owner)
                .and_then(|index| 1u32.checked_shl(index as u32))
                .is_some_and(|bit| ctx.amount & bit != 0);
            owner_alive
                && ctx.kind == EventKind::CardResolved
                && !owner_is_player
                && active.owner != ctx.source
                && damage_card
                && ctx.targets.contains(&active.owner)
                && (damaged
                    == matches!(
                        def.filter,
                        TriggerFilter::OwnerAliveOtherSourceDamageCardResolvedTargetedDamaged
                    ))
        }

        TriggerFilter::OwnerAliveOtherDyingMbchensheAvailable => {
            owner_alive
                && ctx.kind == EventKind::Dying
                && !owner_is_player
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|participant| participant.alive && participant.dying)
                && [active.owner, ctx.player, ctx.source]
                    .into_iter()
                    .filter(|target| {
                        target.is_some()
                            && state
                                .players
                                .get(*target)
                                .is_some_and(|participant| participant.alive)
                    })
                    .any(|target| {
                        if target == active.owner {
                            !crate::query::discardable_hand_equip_cards(state, target).is_empty()
                        } else {
                            !crate::query::discardable_target_cards(
                                state,
                                active.owner,
                                target,
                                rule_api::TargetCardScope::HandEquipJudge,
                            )
                            .is_empty()
                        }
                    })
        }

        TriggerFilter::OwnerIsEventPlayerDiscardableHandEquipOncePerRound => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseEnd
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
        }

        TriggerFilter::OwnerAliveFirstNonDiscardPhaseDiscardMbrenxingAvailable => {
            let used = state
                .skills
                .get(active.skill_id)
                .and_then(|instance| {
                    (instance.state_blob.len() == 12 && &instance.state_blob[..4] == b"RNX1")
                        .then(|| {
                            let round =
                                u32::from_le_bytes(instance.state_blob[4..8].try_into().unwrap());
                            (round == state.turn.round).then(|| {
                                u32::from_le_bytes(instance.state_blob[8..12].try_into().unwrap())
                            })
                        })
                        .flatten()
                })
                .unwrap_or(0);
            let has_discard_target = state.players.alive_by_seat().into_iter().any(|target| {
                state.turn.flag(turn_flag::sha_used_by_player(target)) == 0
                    && !crate::query::discardable_target_cards(
                        state,
                        active.owner,
                        target,
                        rule_api::TargetCardScope::HandEquipJudge,
                    )
                    .is_empty()
            });
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.move_reason == CardMoveReason::Discard
                && state.turn.phase != Phase::Discard
                && state.turn.flag(turn_flag::NON_DISCARD_PHASE_DISCARD_SEEN) == 0
                && used < 2
                && (state
                    .players
                    .get(state.turn.current)
                    .is_some_and(|participant| participant.alive)
                    || has_discard_target)
        }

        TriggerFilter::OwnerSourceFirstTargetHandDamageCardDiscardable => {
            let current_use = ctx.card.and_then(|card| {
                state
                    .resolution
                    .pending_card_resolutions
                    .iter()
                    .rev()
                    .find(|record| record.card == card)
            });
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && owner_is_source
                && ctx.targets.first() == Some(&ctx.player)
                && card_kind.is_some_and(|kind| kind.is_sha() || kind.is_damage_trick())
                && current_use
                    .is_some_and(|record| record.source == active.owner && record.source_was_hand)
                && !crate::query::discardable_target_cards(
                    state,
                    active.owner,
                    active.owner,
                    rule_api::TargetCardScope::Hand,
                )
                .is_empty()
        }

        TriggerFilter::OwnerLostHandOwnerlessShaNotUseOncePerTurn => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardsMoved
                && ctx.from_zone == Some(ZoneKind::Hand)
                && !matches!(
                    ctx.move_reason,
                    CardMoveReason::UseOrRespond | CardMoveReason::Use
                )
                && ctx.cards.iter().any(|card| {
                    state
                        .cards
                        .get(*card)
                        .is_some_and(|card| card.owner.is_none() && card.kind.is_sha())
                })
        }

        TriggerFilter::OwnerAlivePhaseAnyEndOriginatingSkillPileNonempty => {
            owner_alive
                && ctx.kind == EventKind::PhaseAnyEnd
                && !state
                    .skill_pile_cards(active.owner, active.skill_def.raw())
                    .is_empty()
        }

        TriggerFilter::OwnerSourceDamageTrackedMbduanyangTargetRecastable => {
            let tracked = state.skills.get(active.skill_id).and_then(|instance| {
                (instance.state_blob.len() == 8 && &instance.state_blob[..4] == b"DYG1").then(
                    || {
                        CardId::new(u32::from_le_bytes(
                            instance.state_blob[4..8].try_into().unwrap(),
                        ))
                    },
                )
            });
            owner_alive
                && ctx.kind == EventKind::Damage
                && owner_is_source
                && event_player_alive
                && ctx.card.is_some_and(|card| Some(card) == tracked)
                && [ZoneKind::Hand, ZoneKind::Equip, ZoneKind::Judge]
                    .into_iter()
                    .any(|zone| {
                        state
                            .zones
                            .find(zone, ctx.player)
                            .and_then(|id| state.zones.get(id))
                            .is_some_and(|zone| !zone.cards.is_empty())
                    })
        }

        TriggerFilter::OwnerMbrunweiLossThreshold => {
            let armed = state.skills.get(active.skill_id).is_some_and(|instance| {
                instance.state_blob.len() >= 16
                    && &instance.state_blob[..4] == b"RNW1"
                    && u32::from_le_bytes(instance.state_blob[4..8].try_into().unwrap()) > 0
            });
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardsMoved
                && armed
                && !ctx.cards.is_empty()
                && !(ctx.move_reason == CardMoveReason::Use
                    && ctx.cards.iter().all(|card| {
                        state
                            .cards
                            .get(*card)
                            .is_some_and(|card| card.kind.is_equip())
                    }))
        }

        TriggerFilter::OwnerPhasePlayEndMbrunweiTrackedHand => {
            let hand = state.zones.find(ZoneKind::Hand, active.owner);
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhasePlayEnd
                && state.skills.get(active.skill_id).is_some_and(|instance| {
                    instance.state_blob.len() >= 16
                        && &instance.state_blob[..4] == b"RNW1"
                        && instance.state_blob[16..].chunks_exact(4).any(|raw| {
                            state
                                .cards
                                .get(CardId::new(u32::from_le_bytes(raw.try_into().unwrap())))
                                .is_some_and(|card| Some(card.zone) == hand)
                        })
                })
        }

        TriggerFilter::OwnerAliveDamageBeforeTargetDistanceLeOneOncePerTurn => {
            owner_alive
                && ctx.kind == EventKind::DamageBefore
                && event_player_alive
                && matches!(
                    query(
                        state,
                        &QueryRequest::Distance {
                            from: ctx.player,
                            to: active.owner,
                        },
                        &mut QueryContext::new(),
                    ),
                    QueryResponse::Int(distance) if distance <= 1
                )
        }

        TriggerFilter::OwnerSourceDamageMbxiongtuArmed => {
            let flags = state
                .skills
                .get(active.skill_id)
                .and_then(|instance| {
                    (instance.state_blob.len() == 8 && &instance.state_blob[..4] == b"XTG1")
                        .then(|| u32::from_le_bytes(instance.state_blob[4..8].try_into().unwrap()))
                })
                .unwrap_or(0);
            owner_alive
                && ctx.kind == EventKind::Damage
                && owner_is_source
                && flags & 0b001 != 0
                && flags & 0b100 == 0
        }

        TriggerFilter::OwnerPhasePlayEndMbxiongtuState => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhasePlayEnd
                && state.skills.get(active.skill_id).is_some_and(|instance| {
                    instance.state_blob.len() == 8 && &instance.state_blob[..4] == b"XTG1"
                })
        }

        TriggerFilter::OwnerPhaseEndMbganggengTargetAlive => {
            let target = state.skills.get(active.skill_id).and_then(|instance| {
                (instance.state_blob.len() == 8 && &instance.state_blob[..4] == b"GGG1").then(
                    || {
                        PlayerId::new(u32::from_le_bytes(
                            instance.state_blob[4..8].try_into().unwrap(),
                        ))
                    },
                )
            });
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhaseEnd
                && target.is_some_and(|target| {
                    state
                        .players
                        .get(target)
                        .is_some_and(|participant| participant.alive)
                })
        }

        TriggerFilter::OwnerFromHandEmptyOrDyingMbsijianTwicePerTurn => {
            owner_alive
                && owner_is_player
                && turn_fires < 2
                && (ctx.kind == EventKind::Dying
                    || (ctx.kind == EventKind::CardsMoved
                        && matches!(ctx.from_zone, Some(ZoneKind::Hand))
                        && hand_empty))
        }

        TriggerFilter::OwnerIsTargetCanMoveFieldCardMbyuanmoTwicePerTurn => {
            owner_alive
                && owner_is_player
                && turn_fires < 2
                && matches!(ctx.kind, EventKind::PhasePrepare | EventKind::Damage)
                && state.players.iter_live().any(|source| {
                    source.alive
                        && !crate::query::movable_field_cards_from(state, source.id).is_empty()
                })
        }

        TriggerFilter::OwnerSourceFirstSingleTargetShaMbfeijingTwicePerTurn => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardTargeted
                && card_is_sha
                && ctx.target_count == 1
                && ctx.targets.first() == Some(&ctx.player)
                && turn_fires < 2
                && state.players.iter_live().filter(|role| role.alive).count() > 2
        }

        TriggerFilter::OwnerSourceDamageBeforeMbfeijingExtra => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::DamageBefore
                && card_is_sha
                && ctx.card.is_some_and(|card| {
                    mbfeijing_record_matches(state, active.owner, card, Some(ctx.player))
                })
        }

        TriggerFilter::OwnerSourceShaResolvedOriginalSingleMbxiaoge => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardResolved
                && card_is_sha
                && ctx.card.is_some_and(|card| {
                    ctx.target_count == 1
                        || mbfeijing_record_matches(state, active.owner, card, None)
                })
        }

        TriggerFilter::OwnerAliveDamageTargetMinHandMbxuyeOncePerTurn => {
            let hand = state.hand_count(ctx.player);
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::Damage
                && turn_fires < 1
                && state
                    .players
                    .iter_live()
                    .all(|role| hand <= state.hand_count(role.id))
        }

        TriggerFilter::OwnerMbkuangxiangReceivedCardsExhausted => {
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.from_zone == Some(ZoneKind::Hand)
                && crate::effect::mbkuangxiang_exhausted_role(
                    state,
                    active.skill_id,
                    active.owner,
                    ctx.player,
                    &ctx.cards,
                )
        }

        TriggerFilter::OwnerPhasePlayMbkuangxiangActive => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhasePlay
                && state.skills.get(active.skill_id).is_some_and(|instance| {
                    instance.state_blob.len() >= 16 && &instance.state_blob[..4] == b"KXI1"
                })
        }

        TriggerFilter::OwnerGainedCardsPotduzuo => {
            let gained = match ctx.kind {
                EventKind::CardsDrawn => owner_is_player,
                EventKind::CardsMoved => ctx
                    .to_zone
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| zone.owner == active.owner && zone.kind == ZoneKind::Hand),
                _ => false,
            };
            owner_alive && gained && ctx.amount > 0 && ctx.move_reason != CardMoveReason::Potduzuo
        }

        TriggerFilter::OwnerGainedCardsPotduzuoOncePerTurn => {
            let gained = match ctx.kind {
                EventKind::CardsDrawn => owner_is_player,
                EventKind::CardsMoved => ctx
                    .to_zone
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| zone.owner == active.owner && zone.kind == ZoneKind::Hand),
                _ => false,
            };
            owner_alive && gained && ctx.amount > 0 && ctx.move_reason != CardMoveReason::Potduzuo
        }

        TriggerFilter::OwnerShaDamageBeforePotbihanAvailable => {
            let above_hp = |role| {
                state.players.get(role).is_some_and(|participant| {
                    participant.alive && state.hand_count(role) > participant.hp.max(0) as u32
                })
            };
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::DamageBefore
                && card_is_sha
                && (above_hp(active.owner) || above_hp(ctx.player))
        }

        TriggerFilter::OwnerAliveCardTargetedHealthy => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardTargeted
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|player| player.hp.max(0) as u32 == player.max_hp)
        }

        TriggerFilter::OwnerAliveCardResolvedSourceCurrentPotsifengPile => {
            owner_alive
                && ctx.kind == EventKind::CardResolved
                && ctx.source == state.turn.current
                && !state
                    .skill_pile_cards(ctx.source, active.skill_def.raw())
                    .is_empty()
        }

        TriggerFilter::OwnerAlivePhaseEndPlayerPotsifengPile => {
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && !state
                    .skill_pile_cards(ctx.player, active.skill_def.raw())
                    .is_empty()
        }

        TriggerFilter::OwnerAlivePotdingpinAvailableOncePerRound => {
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && state
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .any(|player| state.turn.card_loss_count(player) > 0)
        }

        TriggerFilter::OwnerSourceCardResolvedPhasePolicy => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardResolved
                && ctx.card.is_some_and(|card| {
                    state
                        .skills
                        .get(active.skill_id)
                        .and_then(|instance| instance.phase_card_use_policy())
                        .is_some_and(|(armed, _, _)| armed == card)
                })
        }

        TriggerFilter::OwnerPhasePlayEndPotguanshaAvailableOncePerGame => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::PhasePlayEnd
                && (state.hand_count(active.owner) > 0
                    || state
                        .zones
                        .find(ZoneKind::Equip, active.owner)
                        .and_then(|zone| state.zones.get(zone))
                        .is_some_and(|zone| !zone.cards.is_empty()))
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.lifetime_fire_count == 0)
        }

        TriggerFilter::OwnerDiscardedShaToDiscard => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardsMoved
                && ctx.move_reason == CardMoveReason::Discard
                && ctx.to_zone == discard
                && ctx.cards.iter().any(|card| {
                    state.cards.get(*card).is_some_and(|card_state| {
                        Some(card_state.zone) == discard
                            && crate::query::effective_card_kind(state, active.owner, card_state)
                                .is_sha()
                    })
                })
        }

        TriggerFilter::OwnerAliveOtherTurnDiscardedRecordedPotfuanSuit => {
            let recorded = state
                .skills
                .get(active.skill_id)
                .and_then(|skill| {
                    (skill.state_blob.len() == 32 && &skill.state_blob[..4] == b"FUA1")
                        .then(|| u32::from_le_bytes(skill.state_blob[4..8].try_into().unwrap()))
                })
                .unwrap_or(0);
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && state.turn.current != active.owner
                && recorded & state.turn.flag(turn_flag::DISCARD_PILE_SUIT_MASK) as u32 != 0
        }

        TriggerFilter::OwnerSourceShaResolvedPotfuanDealtDamage => {
            let pending = state.skills.get(active.skill_id).and_then(|skill| {
                (skill.state_blob.len() == 32 && &skill.state_blob[..4] == b"FUA1").then(|| {
                    (
                        CardId::new(u32::from_le_bytes(
                            skill.state_blob[20..24].try_into().unwrap(),
                        )),
                        u32::from_le_bytes(skill.state_blob[28..32].try_into().unwrap()),
                    )
                })
            });
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::ShaResolved
                && ctx.card_flags & rule_api::damage_card_flag::CARD_USE_DEALT_DAMAGE != 0
                && pending.is_some_and(|(pending, first)| ctx.card == Some(pending) && first != 0)
        }

        TriggerFilter::OwnerAliveOtherPlayPhasePothuilvAvailable => {
            owner_alive
                && ctx.kind == EventKind::PhasePlay
                && ctx.player != active.owner
                && event_player_alive
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.pothuilv_state(state.turn.round).0 != 0b11)
        }

        TriggerFilter::OwnerIsTargetCanUseWuzhong => {
            owner_alive
                && owner_is_player
                && matches!(ctx.kind, EventKind::PhasePrepare | EventKind::Damage)
                && crate::command::play::source_free_wuzhong_has_legal_use(state, active.owner)
        }

        TriggerFilter::OwnerAliveDiscardBatch => {
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.move_reason == CardMoveReason::Discard
                && event_player_alive
                && !ctx.cards.is_empty()
        }

        TriggerFilter::OwnerAliveOtherRespondedToOwner => {
            owner_alive
                && ctx.kind == EventKind::CardResponded
                && event_player_alive
                && ctx.player != active.owner
                && ctx.respond_to == active.owner
        }

        TriggerFilter::OwnerSourceDamagePotjianzhuanAvailable => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::Damage
                && ctx.amount > 0
                && potjianzhuan_timing_available(state, active, 0b001)
        }

        TriggerFilter::OwnerTargetDamagePotjianzhuanAvailable => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::Damage
                && ctx.amount > 0
                && potjianzhuan_timing_available(state, active, 0b010)
        }

        TriggerFilter::OwnerAliveDyingPotjianzhuanAvailable => {
            owner_alive
                && ctx.kind == EventKind::Dying
                && potjianzhuan_timing_available(state, active, 0b100)
        }

        TriggerFilter::OwnerSourceDamagePotcaoshuangAssignedUntriggered => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::Damage
                && ctx.amount > 0
                && potcaoshuang_timing_ready(state, active, 0b001)
        }

        TriggerFilter::OwnerTargetDamagePotcaoshuangAssignedUntriggered => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::Damage
                && ctx.amount > 0
                && potcaoshuang_timing_ready(state, active, 0b010)
        }

        TriggerFilter::OwnerAliveDyingPotcaoshuangAssignedUntriggered => {
            owner_alive
                && ctx.kind == EventKind::Dying
                && potcaoshuang_timing_ready(state, active, 0b100)
        }

        TriggerFilter::OwnerUsedPotnizunPileCard => {
            owner_alive
                && matches!(ctx.kind, EventKind::CardUsed | EventKind::CardResponded)
                && (ctx.source == active.owner || ctx.player == active.owner)
                && ctx.card.is_some_and(|card| {
                    state.turn.used_playable_skill_pile_card(active.owner, card)
                        && state.card_has_content_tag(card, crate::state::SHEQUAN_CARD_TAG)
                })
        }

        TriggerFilter::OwnerTargetDamageBeforePotnizunUsed => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::DamageBefore
                && state
                    .skills
                    .get(active.skill_id)
                    .is_some_and(|skill| skill.potnizun_use_count(state.turn.round) > 0)
        }

        TriggerFilter::OwnerAliveTurnEndedUsedCardTaggedDiscard => {
            let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            owner_alive
                && ctx.kind == EventKind::TurnEnded
                && state
                    .turn
                    .flag(turn_flag::card_used_by_player(active.owner))
                    > 0
                && discard.is_some_and(|discard| {
                    state.zones.get(discard).is_some_and(|zone| {
                        zone.cards.iter().any(|card| {
                            state.card_has_content_tag(*card, crate::state::SHEQUAN_CARD_TAG)
                        })
                    })
                })
        }

        TriggerFilter::OwnerSourceCardName(selector) => {
            // Owner used/responded a card whose logical kind/suit matches the selector.
            event_card_matches_selector(state, active.owner, ctx, *selector)
                && owner_is_source
                && matches!(
                    ctx.kind,
                    EventKind::CardUsed | EventKind::CardResolved | EventKind::CardResponded
                )
        }

        TriggerFilter::OwnerIsTargetCardName(selector) => {
            // Owner is the target of a card whose logical kind/suit matches the selector.
            owner_is_player
                && ctx.kind == EventKind::CardTargeted
                && event_card_matches_selector(state, active.owner, ctx, *selector)
        }

        TriggerFilter::OwnerIsTargetCardDamage(selector) => {
            // Owner took damage whose originating card matches the selector.
            owner_is_player
                && ctx.kind == EventKind::Damage
                && event_card_matches_selector(state, active.owner, ctx, *selector)
        }

        TriggerFilter::EventViewAsSkill(skill_def) => {
            // Event card originated from a view-as skill. `view_as_skill` is the skill INSTANCE id;
            // resolve to its definition id and compare. `_backup` view-as helpers (engine-internal)
            // also have resolvable instances, so this covers them too.
            view_as_skill != 0
                && state
                    .skills
                    .get(engine_api::SkillId::new(view_as_skill))
                    .is_some_and(|skill| skill.def.raw() == *skill_def)
        }

        TriggerFilter::EventViewAsSkillName(name) => {
            // Event card originated from a view-as skill whose content-metadata name matches.
            // Covers dynamic _backup helpers without a static def id (ADR 0639 architecture limit).
            view_as_skill != 0
                && state
                    .skills
                    .get(engine_api::SkillId::new(view_as_skill))
                    .and_then(|skill| state.content.skill_metadata(skill.def))
                    .is_some_and(|metadata| metadata.name == *name)
        }
        _ => unreachable!("filter routed to a different predicate group"),
    }
}
