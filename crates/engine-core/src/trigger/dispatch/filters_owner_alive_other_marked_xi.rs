//! Trigger predicates for `OwnerAliveOtherMarkedXi` and related requests.

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
    let card_is_non_multitarget_trick =
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

        TriggerFilter::OwnerAliveSkillBoundTargetIsEventPlayer => {
            owner_alive
                && state
                    .skills
                    .get(active.skill_id)
                    .and_then(|skill| skill.bound_target())
                    == Some(ctx.player)
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

        TriggerFilter::OwnerIsEventPlayerEventCardSha => owner_is_player && card_is_sha,

        TriggerFilter::OwnerIsEventPlayerEventCardShaOrTrick => {
            owner_is_player && (card_is_sha || card_is_trick)
        }

        TriggerFilter::OwnerSourceOrWithinDistanceOneShaOrJuedou => {
            owner_alive
                && (card_is_sha || card_is_juedou)
                && (owner_is_source
                    || (ctx.source.is_some()
                        && crate::query::player_in_attack_range(state, ctx.source, active.owner)))
        }

        TriggerFilter::OwnerSourceSha => owner_is_source && card_is_sha,

        TriggerFilter::OwnerTargetSha => owner_is_player && card_is_sha,

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

        TriggerFilter::OwnerAliveGainedThisTurnCanCompareOther => {
            let gained = state.turn.gained_hand_cards.iter().any(|(owner, card)| {
                *owner == active.owner && state.hand_cards(active.owner).contains(card)
            });
            let has_other = state.players.alive_by_seat().into_iter().any(|candidate| {
                candidate != active.owner && !state.hand_cards(candidate).is_empty()
            });
            owner_alive && gained && has_other
        }

        TriggerFilter::OwnerAliveOtherSingleTargetJuedouOrRedSha => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && !owner_is_source
                && ctx.target_count == 1
                && ctx.player != active.owner
                && (card_is_juedou || (card_is_sha && card_is_red))
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

        TriggerFilter::OwnerAliveSourceShaHandEquipSkillPile(skill_def) => {
            owner_alive
                && source_alive
                && ctx.kind == EventKind::CardUsed
                && card_is_sha
                && (state.hand_count(ctx.source) > 0 || equip_count(ctx.source) > 0)
                && !state.skill_pile_cards(ctx.source, *skill_def).is_empty()
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

        TriggerFilter::OwnerSourceJiu => owner_is_source && card_kind == Some(CardKind::Jiu),

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

        TriggerFilter::OwnerIsSourceShaUsed => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::CardUsed
                && card_kind == Some(CardKind::Sha)
        }

        TriggerFilter::OwnerIsResponderSha => {
            owner_alive
                && owner_is_player
                && ctx.kind == EventKind::CardResponded
                && card_kind == Some(CardKind::Sha)
        }

        TriggerFilter::OwnerIsSourceShaCardDamageBefore => {
            owner_alive
                && owner_is_source
                && ctx.kind == EventKind::DamageBefore
                && ctx.card.is_some()
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

        TriggerFilter::OwnerSourceBasicOrTrick => {
            ctx.kind == EventKind::CardUsed
                && owner_is_source
                && card_kind.is_some_and(|kind| kind.is_basic() || kind.is_trick())
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

        TriggerFilter::OwnerSourcePhysicalOrdinaryTrick => {
            owner_is_source
                && card_is_ordinary_trick
                && ctx.card.is_some_and(|card| view_as_card != card.raw())
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

        TriggerFilter::OncePerTurn => owner_alive && turn_fires == 0,

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

        TriggerFilter::OwnerSourceOncePerTurn => owner_is_source && turn_fires == 0,

        TriggerFilter::OwnerIsEventPlayerOncePerTurn => owner_is_player && turn_fires == 0,

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

        TriggerFilter::DddXuyuPending => {
            ctx.kind == EventKind::TurnEnded
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

        TriggerFilter::DddShijianWatchedSourceCardUsedTargeted => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && ctx.target_count > 0
                && crate::effect::ddd_shijian_watched(state, active.skill_id) == Some(ctx.source)
        }

        TriggerFilter::DddShijianPending => {
            ctx.kind == EventKind::TurnEnded
                && crate::effect::ddd_shijian_watched(state, active.skill_id).is_some()
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

        TriggerFilter::OwnerAliveOtherPhaseBeginBothHandNonempty => {
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::PhaseBegin
                && active.owner != ctx.player
                && state.turn.current == ctx.player
                && !state.hand_cards(active.owner).is_empty()
                && !state.hand_cards(ctx.player).is_empty()
        }

        TriggerFilter::OwnerAliveDddDongchaPileNonempty => {
            owner_alive
                && ctx.kind == EventKind::PhaseEnd
                && !state
                    .skill_pile_cards(active.owner, crate::effect::DDD_DONGCHA_DEF)
                    .is_empty()
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

        TriggerFilter::OwnerAliveOtherPhasePlayDddChashiAvailable => {
            owner_alive
                && event_player_alive
                && ctx.kind == EventKind::PhasePlay
                && active.owner != ctx.player
                && state.turn.current == ctx.player
                && state.turn.phase == Phase::Play
                && !crate::query::discardable_hand_equip_cards(state, active.owner).is_empty()
        }

        TriggerFilter::DddChashiMatchingCard => {
            let pending = state
                .skills
                .get(active.skill_id)
                .and_then(|skill| skill.ddd_chashi_state());
            let suit = ctx
                .card
                .and_then(|card| state.cards.get(card))
                .map(|card| crate::query::effective_card_suit(state, ctx.source, card))
                .map(crate::command::judge::suit_tag)
                .and_then(|value| u8::try_from(value).ok())
                .unwrap_or(0);
            let category = card_kind.map_or(0, |kind| {
                if kind.is_basic() {
                    1
                } else if kind.is_trick() {
                    2
                } else if kind.is_equip() {
                    3
                } else {
                    0
                }
            });
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && pending.is_some_and(|(target, expected_suit, expected_category)| {
                    target == ctx.source
                        && state.turn.current == target
                        && state.turn.phase == Phase::Play
                        && ((suit != 0 && suit == expected_suit)
                            || (category != 0 && category == expected_category))
                })
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
