//! 触发过滤器谓词：source、other 族效果。

use super::*;

pub(super) fn filter_match(
    state: &crate::state::GameState,
    active: &ActiveTrigger,
    ctx: &TriggerContext,
    _view_as_skill: u32,
    view_as_card: u32,
    history: Option<&TriggerHistorySnapshot>,
    def: &crate::trigger::TriggerDefinition,
) -> bool {
    use crate::state::ZoneKind;
    let _owner_is_player = active.owner == ctx.player;
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
    let card_is_trick = card_kind.is_some_and(CardKind::is_trick);
    let card_is_ordinary_trick =
        card_kind.is_some_and(|kind| kind.is_trick() && !kind.is_delayed_trick());
    // The modelled ordinary tricks have no upstream `multitarget` property; selectTarget alone does not make them multitarget.
    let _card_is_non_multitarget_trick =
        card_kind.is_some_and(|kind| kind.is_trick() && !kind.is_delayed_trick());
    let card_is_sha = matches!(ctx.card_kind, Some(1 | 5 | 6));
    let card_nature = ctx
        .card
        .and_then(|card| state.cards.get(card))
        .zip(card_kind)
        .map(|(card, kind)| crate::query::effective_card_nature(state, ctx.source, card, kind));
    let card_is_juedou = ctx.card_kind == Some(CardKind::TrickJuedou.tag());
    let owner_hand = state.hand_cards(active.owner);
    let owner_hand_basic_count = owner_hand
        .iter()
        .filter(|card_id| {
            state.cards.get(**card_id).is_some_and(|card| {
                crate::query::effective_card_kind(state, active.owner, card).is_basic()
            })
        })
        .count();
    let owner_hand_basic_equals_non_basic = owner_hand_basic_count * 2 == owner_hand.len();
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
    let previous_card_meta = history
        .map(|snapshot| snapshot.previous_card_meta)
        .unwrap_or_else(|| state.turn.flag(turn_flag::LAST_CARD_META) as u32);
    let previous_suit = previous_card_meta & 0xff;
    let previous_play_card_meta = history
        .map(|snapshot| snapshot.previous_play_card_meta)
        .unwrap_or_else(|| state.turn.flag(turn_flag::LAST_PLAY_CARD_META) as u32);
    let previous_play_suit = previous_play_card_meta & 0xff;
    let previous_play_rank = (previous_play_card_meta >> 8) & 0xff;
    let previous_game_card_meta = history
        .map(|snapshot| snapshot.previous_game_card_meta)
        .unwrap_or_else(|| {
            state
                .turn
                .flag(turn_flag::last_game_card_meta_by_player(active.owner)) as u32
        });
    let previous_game_suit = previous_game_card_meta & 0xff;
    let previous_game_rank = (previous_game_card_meta >> 8) & 0xff;
    let current_suit = card_suit.map(crate::command::judge::suit_tag);
    let current_rank = ctx
        .card
        .and_then(|card| state.cards.get(card))
        .map(|card| u32::from(card.rank));
    let suit_is_red = |suit| matches!(suit, 2 | 4);
    let owner_wounded = state
        .players
        .get(active.owner)
        .is_some_and(|owner| owner.alive && owner.hp < owner.max_hp as i32);
    match &def.filter {
        TriggerFilter::OwnerAliveOtherMarkedXi => {
            // 檄标记键 = 擅袭技能 def id（ABI 440）。
            const SPSHANXI_DEF: u32 = 2_378_382_374;
            owner_alive
                && ctx.kind == EventKind::HpRecovered
                && ctx.player != active.owner
                && state
                    .players
                    .get(ctx.player)
                    .is_some_and(|participant| participant.hp > 0)
                && state.players.get(ctx.player).is_some_and(|participant| {
                    participant
                        .marks
                        .iter()
                        .any(|(m, n)| *m == SPSHANXI_DEF && *n > 0)
                })
        }

        TriggerFilter::OwnerAliveOtherTrickFirstTargetSourceMaxHand => {
            let source_is_max_hand = state
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| *candidate != ctx.source)
                .all(|candidate| state.hand_count(candidate) <= state.hand_count(ctx.source));
            owner_alive
                && ctx.kind == EventKind::CardTargeted
                && ctx.source != active.owner
                && ctx.player == active.owner
                && ctx.targets.first() == Some(&ctx.player)
                && card_is_trick
                && source_is_max_hand
        }

        TriggerFilter::OwnerAliveOtherPlayPhaseDiscardableGteMark => {
            // 凶竖标记键 = 凶竖技能 def id（ABI 436）。
            const XIONGSHU_DEF: u32 = 3_731_458_110;
            let mark = state
                .players
                .get(active.owner)
                .and_then(|p| {
                    p.marks
                        .iter()
                        .find(|(m, _)| *m == XIONGSHU_DEF)
                        .map(|(_, n)| *n)
                })
                .unwrap_or(0)
                .max(0) as u32;
            owner_alive
                && ctx.kind == EventKind::PhasePlay
                && ctx.player != active.owner
                && ctx.player == state.turn.current
                && !state.hand_cards(ctx.player).is_empty()
                && crate::query::discardable_hand_equip_cards(state, active.owner).len() as u32
                    >= mark
        }

        TriggerFilter::OwnerSourceClub => {
            owner_alive
                && owner_is_source
                && current_suit == Some(crate::command::judge::suit_tag(engine_api::Suit::Club))
        }

        TriggerFilter::OwnerSourceAlternatesColor => {
            owner_alive
                && owner_is_source
                && active.owner == state.turn.current
                && previous_card_meta != 0
                && current_suit.is_some_and(|suit| suit_is_red(suit) != suit_is_red(previous_suit))
        }

        TriggerFilter::OwnerSourceRepeatsColorHasDiscardableHandEquip => {
            owner_alive
                && owner_is_source
                && active.owner == state.turn.current
                && previous_card_meta != 0
                && matches!(previous_suit, 1..=4)
                && current_suit.is_some_and(|suit| {
                    matches!(suit, 1..=4) && suit_is_red(suit) == suit_is_red(previous_suit)
                })
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
        }

        TriggerFilter::OwnerSourceMatchesPreviousSuitOrRank => {
            owner_alive
                && owner_is_source
                && active.owner == state.turn.current
                && state.turn.phase == Phase::Play
                && previous_play_card_meta != 0
                && (current_suit == Some(previous_play_suit)
                    || current_rank == Some(previous_play_rank))
        }

        TriggerFilter::OwnerSourcePlaySkillMarkMode(mark) => {
            let mode = state
                .players
                .get(active.owner)
                .and_then(|owner| owner.marks.iter().find(|(id, _)| id == mark))
                .map(|(_, count)| *count)
                .unwrap_or(0);
            owner_alive
                && owner_is_source
                && active.owner == state.turn.current
                && state.turn.phase == Phase::Play
                && matches!(mode, 1 | 2)
        }

        TriggerFilter::OwnerSourceHandBasicEqualsNonBasic => {
            owner_is_source && owner_hand_basic_equals_non_basic
        }

        TriggerFilter::OwnerSourceOrWithinDistanceOneShaOrJuedou => {
            owner_alive
                && (card_is_sha || card_is_juedou)
                && (owner_is_source
                    || (ctx.source.is_some()
                        && crate::query::player_in_attack_range(state, ctx.source, active.owner)))
        }

        TriggerFilter::OwnerSourceSha => owner_is_source && card_is_sha,

        TriggerFilter::OwnerSourceHeartSha => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardUsed
                && card_is_sha
                && current_suit == Some(crate::command::judge::suit_tag(engine_api::Suit::Heart))
        }

        TriggerFilter::OwnerSourceShaFirstTarget => {
            owner_is_source
                && card_is_sha
                && ctx.kind == EventKind::CardTargeted
                && ctx.targets.first() == Some(&ctx.player)
        }

        TriggerFilter::OwnerSourceShaOrJuedouSingleTarget => {
            owner_is_source
                && (card_is_sha || card_is_juedou)
                && ctx.kind == EventKind::CardTargeted
                && ctx.target_count == 1
        }

        TriggerFilter::OwnerAliveOtherSingleTargetJuedouOrRedSha => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && !owner_is_source
                && ctx.target_count == 1
                && ctx.player != active.owner
                && (card_is_juedou || (card_is_sha && card_is_red))
        }

        TriggerFilter::OwnerSourceNormalSha => {
            ctx.kind == EventKind::CardUsed
                && owner_is_source
                && card_is_sha
                && card_nature == Some(0)
        }

        TriggerFilter::OwnerSourceFireSha => {
            ctx.kind == EventKind::CardUsed
                && owner_is_source
                && card_is_sha
                && card_nature == Some(1)
        }

        TriggerFilter::OwnerSourceSingleTargetShaOrOrdinaryTrickCurrentTurn => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && owner_is_source
                && active.owner == state.turn.current
                && ctx.target_count == 1
                && (card_is_sha || card_is_ordinary_trick)
        }

        TriggerFilter::OwnerSourceEventCardPolicyDraw => {
            owner_alive
                && ctx.kind == EventKind::Damage
                && owner_is_source
                && ctx.card.is_some_and(|card| {
                    state.skills.get(active.skill_id).is_some_and(|skill| {
                        skill.card_use_policy(card)
                            & crate::state::SkillInstance::CARD_USE_POLICY_DRAW_ON_DAMAGE
                            != 0
                    })
                })
        }

        TriggerFilter::OwnerSourcePlayShaOtherAliveOncePerPlayPhase => {
            owner_alive
                && owner_is_source
                && active.owner == state.turn.current
                && state.turn.phase == Phase::Play
                && ctx.player != active.owner
                && event_player_alive
                && card_is_sha
                && play_phase_fires == 0
        }

        TriggerFilter::OwnerSourcePlayCardCountEquals(expected) => {
            let next_count = history
                .map(|snapshot| snapshot.owner_play_phase_card_used)
                .unwrap_or_else(|| {
                    state
                        .turn
                        .flag(turn_flag::play_phase_card_used_by_player(active.owner))
                })
                .saturating_add(1);
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && owner_is_source
                && active.owner == state.turn.current
                && state.turn.phase == Phase::Play
                && u32::try_from(next_count).ok() == Some(*expected)
        }

        TriggerFilter::OwnerSourcePlayCardCountEqualsHp => {
            let next_count = history
                .map(|snapshot| snapshot.owner_play_phase_card_used)
                .unwrap_or_else(|| {
                    state
                        .turn
                        .flag(turn_flag::play_phase_card_used_by_player(active.owner))
                })
                .saturating_add(1);
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && owner_is_source
                && active.owner == state.turn.current
                && state.turn.phase == Phase::Play
                && state
                    .players
                    .get(active.owner)
                    .is_some_and(|owner| owner.hp > 0 && next_count == owner.hp)
        }

        TriggerFilter::OwnerAliveSourceShaHandEquipSkillPile(skill_def) => {
            owner_alive
                && source_alive
                && ctx.kind == EventKind::CardUsed
                && card_is_sha
                && (state.hand_count(ctx.source) > 0 || equip_count(ctx.source) > 0)
                && !state.skill_pile_cards(ctx.source, *skill_def).is_empty()
        }

        TriggerFilter::OwnerAliveOtherSourceSingleShaTargetOther => {
            owner_alive
                && source_alive
                && ctx.kind == EventKind::CardTargeted
                && ctx.source != active.owner
                && ctx.player != active.owner
                && ctx.target_count == 1
                && card_is_sha
        }

        TriggerFilter::OwnerAliveOtherSourceSingleShaTargetOtherOwnerInAttackRange => {
            owner_alive
                && source_alive
                && ctx.kind == EventKind::CardTargeted
                && ctx.source != active.owner
                && ctx.player != active.owner
                && ctx.target_count == 1
                && card_is_sha
                && crate::query::player_in_attack_range(state, active.owner, ctx.source)
        }

        TriggerFilter::OwnerAliveOtherSourceShaResolvedTwxiaolianRedirectHasHandEquip => {
            owner_alive
                && source_alive
                && ctx.kind == EventKind::ShaResolved
                && ctx.source != active.owner
                && ctx.card.is_some_and(|card| {
                    state
                        .skills
                        .get(active.skill_id)
                        .and_then(|skill| skill.twxiaolian_redirect())
                        .is_some_and(|(remembered, _)| remembered == card)
                })
                && (state.hand_count(ctx.source) > 0 || equip_count(ctx.source) > 0)
        }

        TriggerFilter::OwnerSourceJiu => owner_is_source && card_kind == Some(CardKind::Jiu),

        TriggerFilter::OwnerSourceTrick => owner_is_source && card_is_trick,

        TriggerFilter::OwnerSourceShaOrTrickCurrentPlayPhase => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && owner_is_source
                && active.owner == state.turn.current
                && state.turn.phase == Phase::Play
                && (card_is_sha || card_is_trick)
        }

        TriggerFilter::OwnerSourceShaCurrentPlayPhase => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && owner_is_source
                && active.owner == state.turn.current
                && state.turn.phase == Phase::Play
                && card_is_sha
        }

        TriggerFilter::OwnerSourceCardHasOtherTargetCurrentPlayPhaseFirstTwo => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && owner_is_source
                && active.owner == state.turn.current
                && state.turn.phase == Phase::Play
                && ctx.targets.iter().any(|target| *target != active.owner)
        }

        TriggerFilter::OwnerSourceOrdinaryTrick => owner_is_source && card_is_ordinary_trick,

        TriggerFilter::OwnerSourceOrdinaryTrickMarkPositive(mark) => {
            owner_is_source
                && card_is_ordinary_trick
                && state.players.get(active.owner).is_some_and(|owner| {
                    owner
                        .marks
                        .iter()
                        .find(|(id, _)| id == mark)
                        .is_some_and(|(_, count)| *count > 0)
                })
        }

        TriggerFilter::OwnerSourceShaOrBlackTrickMarkPositive(mark) => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && owner_is_source
                && (card_is_sha || (card_is_ordinary_trick && card_is_black))
                && state.players.get(active.owner).is_some_and(|owner| {
                    owner
                        .marks
                        .iter()
                        .find(|(id, _)| id == mark)
                        .is_some_and(|(_, count)| *count > 0)
                })
        }

        TriggerFilter::OwnerSourceBasic => {
            ctx.kind == EventKind::CardUsed
                && owner_is_source
                && card_kind.is_some_and(CardKind::is_basic)
        }

        TriggerFilter::OwnerSourceNonBasic => {
            ctx.kind == EventKind::CardUsed
                && owner_is_source
                && card_kind.is_some_and(|kind| !kind.is_basic())
        }

        TriggerFilter::OwnerSourceWoundedNanmanWanjianOrNormalSpadeSha => {
            ctx.kind == EventKind::CardUsed
                && owner_is_source
                && owner_wounded
                && (matches!(
                    card_kind,
                    Some(CardKind::TrickNanman | CardKind::TrickWanjian)
                ) || (card_is_sha
                    && card_nature == Some(rule_api::damage_nature::NORMAL)
                    && card_suit == Some(engine_api::Suit::Spade)))
        }

        TriggerFilter::OwnerSourceDamageCardOncePerTurn => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && owner_is_source
                && card_kind.is_some_and(|kind| kind.is_sha() || kind.is_damage_trick())
        }

        TriggerFilter::OwnerAliveOtherPlayPhaseDuplicateCardCategoryHasHandEquip => {
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::PhasePlayEnd
                && active.owner != ctx.player
                && state.turn.current == ctx.player
                && state.turn.play_card_category_repeated(ctx.player)
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
        }

        TriggerFilter::OwnerAliveOtherPhaseEndDamageGtOneHasHandEquip => {
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::PhaseEnd
                && active.owner != ctx.player
                && state.turn.current == ctx.player
                && state.turn.flag(turn_flag::DAMAGE_DEALT) > 1
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
        }

        TriggerFilter::OwnerAliveOtherPhaseEndHasDiscardableBasicHand => {
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::PhaseEnd
                && active.owner != ctx.player
                && !crate::query::standard_xiaoguo_basic_hand_cards(state, active.owner).is_empty()
        }

        TriggerFilter::OwnerAliveOtherPhaseEndHandNonempty => {
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::PhaseEnd
                && active.owner != ctx.player
                && state.turn.current == ctx.player
                && !state.hand_cards(active.owner).is_empty()
        }

        TriggerFilter::OwnerAliveOtherShaTargetCanRedirectToOwner => {
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::CardTargeted
                && card_kind == Some(CardKind::Sha)
                && ctx.source.is_some()
                && ctx.source != active.owner
                && ctx.player != active.owner
                && !ctx.targets.contains(&active.owner)
                && crate::query::player_in_attack_range(state, ctx.player, active.owner)
                && !crate::query::discardable_hand_equip_cards(state, ctx.player).is_empty()
        }

        TriggerFilter::OwnerIsSourceShaUsed => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardUsed
                && card_kind == Some(CardKind::Sha)
        }

        TriggerFilter::OwnerIsSourceShaCardDamageBefore => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::DamageBefore
                && ctx.card.is_some()
                && card_kind == Some(CardKind::Sha)
        }

        TriggerFilter::OwnerSourceBasicOrTrick => {
            ctx.kind == EventKind::CardUsed
                && owner_is_source
                && card_kind.is_some_and(|kind| kind.is_basic() || kind.is_trick())
        }

        TriggerFilter::OwnerSourcePhysicalOrdinaryTrick => {
            owner_is_source
                && card_is_ordinary_trick
                && ctx.card.is_some_and(|card| view_as_card != card.raw())
        }

        TriggerFilter::OwnerAliveSourceFirstCardOrdinaryTrick => {
            owner_alive
                && card_is_ordinary_trick
                && history
                    .map(|snapshot| snapshot.source_card_used)
                    .unwrap_or_else(|| state.turn.flag(turn_flag::card_used_by_player(ctx.source)))
                    == 0
        }

        TriggerFilter::OwnerSourceMatchesLastUsedSuitOrRank => {
            owner_alive
                && owner_is_source
                && previous_game_card_meta != 0
                && (current_suit == Some(previous_game_suit)
                    || current_rank == Some(previous_game_rank))
        }

        TriggerFilter::OwnerSourceOncePerTurn => owner_is_source && turn_fires == 0,

        TriggerFilter::OwnerSourceTargetAliveHasEquip => {
            owner_alive && owner_is_source && event_player_alive && equip_count(ctx.player) > 0
        }

        TriggerFilter::OwnerSourceBlackShaTargetAliveHasDiscardableEquip => {
            owner_alive
                && owner_is_source
                && event_player_alive
                && card_is_sha
                && matches!(
                    card_suit,
                    Some(engine_api::Suit::Spade | engine_api::Suit::Club)
                )
                && !crate::query::discardable_equipment_cards(state, active.owner, ctx.player)
                    .is_empty()
        }

        TriggerFilter::OwnerSourceShaTargetAliveHasDiscardableRegionCard => {
            owner_alive
                && owner_is_source
                && event_player_alive
                && card_is_sha
                && !crate::query::discardable_target_cards(
                    state,
                    active.owner,
                    ctx.player,
                    rule_api::TargetCardScope::HandEquipJudge,
                )
                .is_empty()
        }

        TriggerFilter::OwnerSourceShaTargetAliveHasDiscardableHandEquip => {
            owner_alive
                && owner_is_source
                && event_player_alive
                && ctx.kind == EventKind::CardTargeted
                && card_is_sha
                && !crate::query::discardable_target_cards(
                    state,
                    active.owner,
                    ctx.player,
                    rule_api::TargetCardScope::HandEquip,
                )
                .is_empty()
        }

        TriggerFilter::OwnerSourceTiesuoTargetAliveUnlinkedHandEquipNonempty => {
            owner_alive
                && owner_is_source
                && event_player_alive
                && ctx.kind == EventKind::CardTargeted
                && card_kind == Some(CardKind::TrickTiesuo)
                && (state.hand_count(ctx.player) > 0 || equip_count(ctx.player) > 0)
                && state.players.get(ctx.player).is_some_and(|player| {
                    !player
                        .marks
                        .iter()
                        .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
                })
        }

        TriggerFilter::OwnerSourceShaTargetAliveHandNonempty => {
            owner_alive
                && owner_is_source
                && event_player_alive
                && ctx.kind == EventKind::CardTargeted
                && card_is_sha
                && state.hand_count(ctx.player) > 0
        }

        TriggerFilter::OwnerSourceShaTargetAliveDistanceLeOne => {
            owner_source_distance_le_one && event_player_alive && card_is_sha
        }

        TriggerFilter::OwnerAliveOtherGainedSingleEquipToHandOutsideTurnUsable => {
            let recipient = ctx
                .to_zone
                .and_then(|zone| state.zones.get(zone))
                .filter(|zone| zone.kind == ZoneKind::Hand)
                .map(|zone| zone.owner);
            let gained = ctx.cards.first().copied();
            owner_alive
                && ctx.kind == EventKind::CardsMoved
                && ctx.cards.len() == 1
                && recipient.is_some_and(|recipient| {
                    recipient != active.owner
                        && recipient != state.turn.current
                        && state.players.get(recipient).is_some_and(|player| {
                            player.alive
                                && !player.dying
                                && gained.is_some_and(|card| {
                                    state.cards.get(card).is_some_and(|card_state| {
                                        card_state.owner == recipient
                                            && Some(card_state.zone) == ctx.to_zone
                                            && card_state.kind.is_equip()
                                            && card_state
                                                .kind
                                                .equip_slot()
                                                .is_some_and(|slot| player.equip_slot_enabled(slot))
                                            && !crate::query::card_use_cancelled_for_card(
                                                state, recipient, card,
                                            )
                                    })
                                })
                        })
                })
        }

        TriggerFilter::OwnerSourcePlayPhaseShaHasAdditionalTargetIgnoringDistance => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardUsed
                && card_is_sha
                && state.turn.current == active.owner
                && state.turn.phase == crate::state::Phase::Play
                && ctx.card.is_some_and(|card| {
                    state.cards.get(card).is_some_and(|card_state| {
                        state.players.alive_by_seat().into_iter().any(|target| {
                            target != active.owner
                                && !ctx.targets.contains(&target)
                                && !crate::query::card_target_cancelled_for_logical_kind(
                                    state,
                                    active.owner,
                                    target,
                                    card_state,
                                    CardKind::Sha,
                                )
                        })
                    })
                })
        }

        TriggerFilter::OwnerSourceCardResolvedTrackedDddXianxi => {
            let tracked = state
                .skills
                .get(active.skill_id)
                .and_then(|skill| skill.state_blob.get(4..8))
                .and_then(|raw| raw.try_into().ok())
                .map(u32::from_le_bytes);
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardResolved
                && ctx.card.map(engine_api::CardId::raw) == tracked
                && state.skills.get(active.skill_id).is_some_and(|skill| {
                    skill.state_blob.get(0..4).is_some_and(|raw| {
                        u32::from_le_bytes(raw.try_into().unwrap()) == 0xDDD0_6170
                    })
                })
        }

        TriggerFilter::OwnerSourceCardResolvedDddXuyuPending => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardResolved
                && crate::effect::ddd_xuyu_pending(state, active.skill_id)
        }

        TriggerFilter::OwnerAliveOtherSourceSecondPlayCardResolvedHandEquipNonempty => {
            let owner_has_card = state
                .zones
                .find(ZoneKind::Hand, active.owner)
                .and_then(|zone| state.zones.get(zone))
                .is_some_and(|zone| !zone.cards.is_empty())
                || state
                    .zones
                    .find(ZoneKind::Equip, active.owner)
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| !zone.cards.is_empty());
            owner_alive
                && ctx.kind == EventKind::CardResolved
                && ctx.source != active.owner
                && event_player_alive
                && state.turn.current == ctx.source
                && state.turn.phase == Phase::Play
                && history
                    .map(|snapshot| snapshot.event_player_play_phase_card_used)
                    .unwrap_or_else(|| state.turn.play_phase_card_use_count(ctx.source))
                    == 2
                && owner_has_card
        }

        TriggerFilter::OwnerAliveOtherPhaseBeginBothHandNonempty => {
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::PhaseBegin
                && active.owner != ctx.player
                && state.turn.current == ctx.player
                && !state.hand_cards(active.owner).is_empty()
                && !state.hand_cards(ctx.player).is_empty()
        }

        TriggerFilter::OwnerAliveOtherPhaseBeginUnshownHandGteHp => {
            let need = state
                .players
                .get(ctx.player)
                .map(|participant| participant.hp.max(0) as usize)
                .unwrap_or(0);
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::PhaseBegin
                && active.owner != ctx.player
                && state.turn.current == ctx.player
                && need > 0
                && state
                    .hand_cards(active.owner)
                    .into_iter()
                    .filter(|card| state.cards.get(*card).is_some_and(|card| !card.face_up))
                    .count()
                    >= need
        }

        TriggerFilter::OwnerAliveOtherPhasePlayDddChashiAvailable => {
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::PhasePlay
                && active.owner != ctx.player
                && state.turn.current == ctx.player
                && state.turn.phase == Phase::Play
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
        }

        _ => unreachable!("filter routed to a different predicate group"),
    }
}
