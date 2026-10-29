//! 触发过滤器谓词：source、other 族效果。

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
    let event_player_alive = state
        .players
        .get(ctx.player)
        .map(|player| player.alive)
        .unwrap_or(false);
    let _equip_count = |player_id| {
        state
            .zones
            .find(ZoneKind::Equip, player_id)
            .and_then(|zone| state.zones.get(zone))
            .map_or(0, |zone| zone.cards.len())
    };
    let _hand_empty = state.hand_count(active.owner) == 0;
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

        TriggerFilter::OwnerAliveOtherTargetedBySha => {
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && card_is_sha
                && ctx.player != active.owner
        }

        TriggerFilter::OwnerAliveOtherSourceDamage => {
            owner_alive
                && ctx.kind == EventKind::Damage
                && ctx.source != PlayerId::NONE
                && ctx.source != active.owner
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

        TriggerFilter::OwnerSourceDamagePotcaoshuangAssignedUntriggered => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::Damage
                && ctx.amount > 0
                && potcaoshuang_timing_ready(state, active, 0b001)
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

        _ => unreachable!("filter routed to a different predicate group"),
    }
}
