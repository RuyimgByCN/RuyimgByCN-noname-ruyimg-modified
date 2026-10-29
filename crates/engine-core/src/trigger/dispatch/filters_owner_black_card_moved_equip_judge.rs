//! Trigger predicates for `OwnerBlackCardMovedEquipJudge` and related requests.

use super::*;

pub(super) fn filter_match(
    state: &crate::state::GameState,
    active: &ActiveTrigger,
    ctx: &TriggerContext,
    view_as_skill: u32,
    _view_as_card: u32,
    _history: Option<&TriggerHistorySnapshot>,
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
    let _card_is_ordinary_trick =
        card_kind.is_some_and(|kind| kind.is_trick() && !kind.is_delayed_trick());
    let card_is_sha = matches!(ctx.card_kind, Some(1 | 5 | 6));
    let suit_owner = if ctx.source.is_some() {
        ctx.source
    } else {
        ctx.player
    };
    let _card_suit = ctx
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

        TriggerFilter::CardsMovedDiscardAtLeastThree => {
            ctx.kind == EventKind::CardsMoved
                && ctx
                    .to_zone
                    .and_then(|zone| state.zones.get(zone).map(|z| z.kind))
                    == Some(ZoneKind::DiscardPile)
                && ctx.move_reason == CardMoveReason::Discard
                && ctx.cards.len() >= 3
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

        TriggerFilter::OwnerSkillAddedSelf(skill_def) => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::SkillAdded
                && ctx.added_skill_def == *skill_def
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

        TriggerFilter::OwnerAliveShanUsedByOwnerOrDuringOwnerTurn => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && card_kind == Some(CardKind::Shan)
                && (owner_is_source || state.turn.current == active.owner)
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
