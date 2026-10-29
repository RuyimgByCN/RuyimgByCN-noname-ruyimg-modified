//! Query operations for `card_target_cancelled_for_logical_kind` and related helpers.

use super::*;

/// Required Sha count for one Duel exchange after modifiers owned by the responder's opponent.
pub fn duel_response_count_for(
    state: &GameState,
    responder_opponent: PlayerId,
    base_count: u32,
) -> u32 {
    let mods = state
        .modifiers
        .for_target(ModifierTarget::DuelOpponentResponseCount);
    let (value, _, vetoed) =
        apply_int_modifiers_for_state(state, base_count as i32, &mods, Some(responder_opponent));
    if vetoed {
        0
    } else {
        value.max(0) as u32
    }
}

/// Required Shan count for one Sha target after source-owned general modifiers and modifiers
/// whose owner participates opposite a female character.
pub fn sha_response_count_for(
    state: &GameState,
    source: PlayerId,
    target: PlayerId,
    base_count: u32,
) -> u32 {
    let base_count = response_count_for(state, source, base_count);
    let has_female_sex = |player: PlayerId| {
        state
            .players
            .get(player)
            .filter(|player| player.alive)
            .and_then(|player| state.content.character(player.character_def))
            .is_some_and(|character| matches!(character.sex.as_str(), "female" | "double"))
    };
    let modifiers: Vec<_> = state
        .modifiers
        .for_target(ModifierTarget::ShaResponseCountIfOtherFemale)
        .into_iter()
        .filter(|modifier| {
            (modifier.owner == source && has_female_sex(target))
                || (modifier.owner == target && has_female_sex(source))
        })
        .collect();
    let (value, _, vetoed) =
        apply_int_modifiers_for_state(state, base_count as i32, &modifiers, None);
    let nearby_modifiers: Vec<_> = state
        .modifiers
        .for_target(ModifierTarget::ShaResponseCountIfSourceOwnerTargetDistanceLeOne)
        .into_iter()
        .filter(|modifier| {
            modifier.owner == source
                && matches!(
                    query(
                        state,
                        &QueryRequest::Distance {
                            from: source,
                            to: target,
                        },
                        &mut QueryContext::new(),
                    ),
                    QueryResponse::Int(distance) if distance <= 1
                )
        })
        .collect();
    let (value, _, nearby_vetoed) =
        apply_int_modifiers_for_state(state, value, &nearby_modifiers, None);
    if vetoed || nearby_vetoed {
        0
    } else {
        value.max(0) as u32
    }
}

/// Damage received from a logical Sha after target-owned Sha-specific modifiers.
pub fn sha_damage_taken_for(state: &GameState, target: PlayerId, base_amount: u32) -> u32 {
    let mods = state.modifiers.for_target(ModifierTarget::ShaDamageTaken);
    let (value, _, vetoed) =
        apply_int_modifiers_for_state(state, base_amount as i32, &mods, Some(target));
    if vetoed {
        0
    } else {
        value.max(0) as u32
    }
}

/// Sha damage after source-owned modifiers that require being outside the target's attack range.
pub fn sha_damage_outside_target_range_for(
    state: &GameState,
    source: PlayerId,
    target: PlayerId,
    base_amount: u32,
) -> u32 {
    let (distance, range) = match (
        distance_explain(state, target, source).response,
        attack_range_explain(state, target).response,
    ) {
        (QueryResponse::Int(distance), QueryResponse::Int(range)) => (distance, range),
        _ => return base_amount,
    };
    if distance <= range {
        return base_amount;
    }
    let mods = state
        .modifiers
        .for_target(ModifierTarget::ShaDamageIfOutsideTargetRange);
    let (value, _, vetoed) =
        apply_int_modifiers_for_state(state, base_amount as i32, &mods, Some(source));
    if vetoed {
        0
    } else {
        value.max(0) as u32
    }
}

pub(crate) fn sha_damage_source_range_threshold(
    state: &GameState,
    source: PlayerId,
) -> Option<(i32, i32)> {
    let mods = state
        .modifiers
        .for_target(ModifierTarget::ShaDamageSourceRangeThreshold);
    let (threshold, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(source));
    if !vetoed && threshold <= 0 {
        return None;
    }
    let range = match attack_range_explain(state, source).response {
        QueryResponse::Int(range) => range,
        _ => return None,
    };
    Some((range, threshold))
}

/// Early source-range bonus for logical Sha damage.
pub fn sha_damage_source_range_bonus_for(
    state: &GameState,
    source: PlayerId,
    base_amount: u32,
) -> u32 {
    if sha_damage_source_range_threshold(state, source)
        .is_some_and(|(range, threshold)| range > threshold)
    {
        base_amount.saturating_add(1)
    } else {
        base_amount
    }
}

/// Late source-range cap for logical Sha damage.
pub fn sha_damage_source_range_cap_for(
    state: &GameState,
    source: PlayerId,
    base_amount: u32,
) -> u32 {
    if base_amount > 1
        && sha_damage_source_range_threshold(state, source)
            .is_some_and(|(range, threshold)| range < threshold)
    {
        1
    } else {
        base_amount
    }
}

/// Damage received after target-owned amount modifiers for the resolved nature.
pub fn damage_taken_amount_for(
    state: &GameState,
    source: PlayerId,
    target: PlayerId,
    nature: u8,
    base_amount: u32,
) -> u32 {
    let range_mods = state
        .modifiers
        .for_target(ModifierTarget::DamageTakenSourceRangeThreshold);
    let (threshold, _, range_vetoed) =
        apply_int_modifiers_for_state(state, 0, &range_mods, Some(target));
    let source_range = if source.is_some() && (range_vetoed || threshold > 0) {
        match attack_range_explain(state, source).response {
            QueryResponse::Int(range) => Some(range),
            _ => None,
        }
    } else {
        None
    };
    let base_amount = if source_range.is_some_and(|range| range > threshold) {
        base_amount.saturating_add(1)
    } else {
        base_amount
    };
    let mods = state
        .modifiers
        .for_damage_taken_amount(nature, target, |owner| {
            state.players.get(owner).is_some_and(|player| player.alive)
        });
    let (mut value, _, vetoed) = apply_int_modifiers_for_state_with_context(
        state,
        base_amount as i32,
        &mods,
        None,
        Some(target),
        Some(source),
    );
    if vetoed {
        return 0;
    }
    if !armor_effect_ignored_for(state, source, target) {
        let mods = state.modifiers.for_armor_damage_taken_amount(nature);
        let (armor_value, _, vetoed) =
            apply_int_modifiers_for_state(state, value, &mods, Some(target));
        if vetoed {
            return 0;
        }
        value = armor_value;
    }
    if value > 1 && source_range.is_some_and(|range| range < threshold) {
        1
    } else {
        value.max(0) as u32
    }
}

/// Final damage received after equipment and ordinary target-owned amount modifiers.
pub fn damage_taken_final_amount_for(
    state: &GameState,
    target: PlayerId,
    nature: u8,
    base_amount: u32,
) -> u32 {
    let mods = state
        .modifiers
        .for_damage_taken_final_amount(nature, target, |owner| {
            state.players.get(owner).is_some_and(|player| player.alive)
        });
    let (value, _, vetoed) = apply_int_modifiers_for_state_with_context(
        state,
        base_amount as i32,
        &mods,
        None,
        Some(target),
        None,
    );
    if vetoed {
        0
    } else {
        value.max(0) as u32
    }
}

/// Resolved damage nature after target-owned nature modifiers.
pub fn damage_taken_nature_for(state: &GameState, target: PlayerId, base_nature: u8) -> u8 {
    let mods = state
        .modifiers
        .for_target(ModifierTarget::DamageTakenNature);
    let (value, _, vetoed) =
        apply_int_modifiers_for_state(state, base_nature as i32, &mods, Some(target));
    if vetoed {
        base_nature
    } else {
        value.clamp(0, u8::MAX as i32) as u8
    }
}

/// Maximum number of distinct targets one proactive 杀 may select.
pub fn sha_target_limit_for(state: &GameState, source: PlayerId) -> u32 {
    let mods = state.modifiers.for_target(ModifierTarget::ShaTargetLimit);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 1, &mods, Some(source));
    if vetoed {
        0
    } else {
        value.max(0) as u32
    }
}

/// Maximum number of independently resolved targets for one concrete card use.
pub fn card_target_limit_for_card(
    state: &GameState,
    source: PlayerId,
    card: engine_api::CardId,
    base: u32,
) -> u32 {
    let Some(card) = state.cards.get(card) else {
        return base;
    };
    let effective_kind = effective_card_kind(state, source, card);
    card_target_limit(
        state,
        source,
        card.kind,
        effective_kind,
        effective_card_suit(state, source, card),
        base,
    )
}

/// Maximum target count for a source-free logical card use.
pub fn card_target_limit_for_kind(
    state: &GameState,
    source: PlayerId,
    kind: CardKind,
    base: u32,
) -> u32 {
    card_target_limit(state, source, kind, kind, engine_api::Suit::None, base)
}

pub(crate) fn card_target_limit(
    state: &GameState,
    source: PlayerId,
    physical_kind: CardKind,
    effective_kind: CardKind,
    effective_suit: engine_api::Suit,
    base: u32,
) -> u32 {
    let mods = state
        .modifiers
        .for_card_target_limit(physical_kind, effective_kind, effective_suit);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, base as i32, &mods, Some(source));
    if vetoed {
        0
    } else {
        value.max(0) as u32
    }
}

/// Whether one proactive 杀 must target every otherwise-legal player.
pub fn sha_targets_all_for(state: &GameState, source: PlayerId) -> bool {
    let mods = state.modifiers.for_card_targets_all(CardSelector::Sha);
    let (value, _, vetoed) = apply_int_modifiers(0, &mods, Some(source));
    vetoed || value != 0
}

/// Sha uses charged against the current Play phase's allowance.
///
/// ABI-44 content separates this bounded phase counter from the turn-wide Sha history used by
/// skills such as Keji. Older content keeps the original single-counter state and hashes.
pub fn sha_play_phase_use_count(state: &GameState) -> u32 {
    let key = if state.content.tracks_opening_extra_play_lifecycle() {
        crate::state::turn_flag::SHA_PLAYED_IN_PLAY_PHASE
    } else {
        crate::state::turn_flag::SHA_PLAYED
    };
    state.turn.flag(key).max(0) as u32
}

/// Whether cards used by `source` may only target that same player.
pub fn card_targets_self_only_for(state: &GameState, source: PlayerId) -> bool {
    let mods = state
        .modifiers
        .for_target(ModifierTarget::CardTargetsSelfOnly);
    let (value, _, vetoed) = apply_int_modifiers(0, &mods, Some(source));
    vetoed || value != 0
}

/// Whether content policy skips this player's discard phase after a no-杀 play phase.
pub fn skip_discard_if_no_sha(state: &GameState, player: PlayerId) -> bool {
    if state.turn.flag(crate::state::turn_flag::SHA_PLAYED) != 0 {
        return false;
    }
    let mods = state
        .modifiers
        .for_target(ModifierTarget::SkipDiscardIfNoSha);
    let (value, _, vetoed) = apply_int_modifiers(0, &mods, Some(player));
    !vetoed && value > 0
}

/// Whether content policy skips this player's judgment phase.
pub fn skip_judge_phase(state: &GameState, player: PlayerId) -> bool {
    let mods = state.modifiers.for_target(ModifierTarget::SkipJudgePhase);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    !vetoed && value > 0
}

/// Whether content policy skips this player's discard phase unconditionally at query time.
pub fn skip_discard_phase(state: &GameState, player: PlayerId) -> bool {
    let mods = state.modifiers.for_target(ModifierTarget::SkipDiscardPhase);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    !vetoed && value > 0
}

/// Whether a skipped or cancelled phase is restored for this player after its audit event.
pub fn restore_skipped_phase(state: &GameState, player: PlayerId) -> bool {
    let mods = state
        .modifiers
        .for_target(ModifierTarget::RestoreSkippedPhase);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    !vetoed && value > 0
}

/// Black non-trick hand/equipment cards that can pay 3D精选丁奉「短兵」.
pub(crate) fn ddd_duanbing_cost_cards(state: &GameState, player: PlayerId) -> Vec<CardId> {
    if !ddd_duanbing_can_receive_bingliang(state, player) {
        return Vec::new();
    }
    let hand = state.zones.find(ZoneKind::Hand, player);
    let equip = state.zones.find(ZoneKind::Equip, player);
    state
        .cards
        .sorted_live()
        .into_iter()
        .filter(|card| Some(card.zone) == hand || Some(card.zone) == equip)
        .filter(|card| effective_card_color_mask(state, player, card.id) == 0b10)
        .filter(|card| !effective_card_kind(state, player, card).is_trick())
        .map(|card| card.id)
        .collect()
}

/// Owner judgment cards moved by 3D精选丁奉「短兵」 after card damage.
pub(crate) fn ddd_duanbing_transfer_cards(
    state: &GameState,
    owner: PlayerId,
    target: PlayerId,
) -> Vec<CardId> {
    if !ddd_duanbing_can_receive_bingliang(state, target) {
        return Vec::new();
    }
    let Some(judge) = state.zones.find(ZoneKind::Judge, owner) else {
        return Vec::new();
    };
    state
        .zones
        .get(judge)
        .into_iter()
        .flat_map(|zone| zone.cards.iter().copied())
        .filter(|card| {
            state
                .cards
                .get(*card)
                .is_some_and(|card| card.kind == CardKind::TrickBingliang)
        })
        .collect()
}

pub(crate) fn ddd_duanbing_can_receive_bingliang(state: &GameState, player: PlayerId) -> bool {
    if state.disabled_judge_zones.contains(&player)
        || !state
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return false;
    }
    state
        .zones
        .find(ZoneKind::Judge, player)
        .and_then(|zone| state.zones.get(zone))
        .is_some_and(|zone| {
            zone.cards.iter().all(|card| {
                state
                    .cards
                    .get(*card)
                    .is_none_or(|card| card.kind != CardKind::TrickBingliang)
            })
        })
}

/// Whether a face-up player is protected from being turned face-down.
pub fn turn_face_down_cancelled(state: &GameState, player: PlayerId) -> bool {
    let mods = state
        .modifiers
        .for_target(ModifierTarget::TurnFaceDownCancel);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    !vetoed && value > 0
}

/// Whether content policy prevents either face-up/face-down transition.
pub fn turn_over_cancelled(state: &GameState, player: PlayerId) -> bool {
    let mods = state.modifiers.for_target(ModifierTarget::TurnOverCancel);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    !vetoed && value > 0
}

/// Whether the ordinary discard phase counts and may discard the owner's equipment cards.
pub fn discard_phase_includes_equip(state: &GameState, player: PlayerId) -> bool {
    let mods: Vec<_> = state
        .modifiers
        .for_target(ModifierTarget::DiscardPhaseIncludesEquip)
        .into_iter()
        .filter(|modifier| {
            modifier.owner == player
                || modifier.value_source
                    == Some(ModifierValueSource::QueriedPlayerLinkedOtherThanOwner)
        })
        .collect();
    let (value, _, vetoed) =
        apply_int_modifiers_for_state_with_context(state, 0, &mods, None, Some(player), None);
    !vetoed && value > 0
}

/// Whether this seated observer may see the faces of cards in other players' hands.
pub fn observer_foreign_hand_faces_visible(state: &GameState, observer: PlayerId) -> bool {
    let mods = state
        .modifiers
        .for_target(ModifierTarget::ObserverForeignHandFacesVisible);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(observer));
    !vetoed && value > 0
}

/// Whether this seated observer may see identities that are otherwise private.
pub fn observer_hidden_identities_visible(state: &GameState, observer: PlayerId) -> bool {
    let mods = state
        .modifiers
        .for_target(ModifierTarget::ObserverHiddenIdentitiesVisible);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(observer));
    !vetoed && value > 0
}

/// Whether the player may perform the optional four-card draw-phase reorder.
pub fn has_draw_phase_scry_four_split_two(state: &GameState, player: PlayerId) -> bool {
    let mods = state
        .modifiers
        .for_target(ModifierTarget::DrawPhaseScryFourSplitTwo);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    !vetoed && value > 0
}

/// Initial hand size after package modifiers.
pub fn initial_hand_count_for(state: &GameState, player: PlayerId, base: u32) -> u32 {
    let mods = state.modifiers.for_target(ModifierTarget::InitialHandCount);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, base as i32, &mods, Some(player));
    if vetoed {
        0
    } else {
        value.max(0) as u32
    }
}

/// Whether `target` is immune to being chosen by a trick of the given
/// immunity channel (e.g. 谦逊 → 顺手牵羊 / 乐不思蜀). `channel` is one of
/// [`ModifierTarget::ImmuneShunshou`] / [`ModifierTarget::ImmuneLe`].
pub fn target_immune(state: &GameState, target: PlayerId, channel: ModifierTarget) -> bool {
    let mods = state.modifiers.for_target(channel);
    let (value, _, _) = apply_int_modifiers(0, &mods, Some(target));
    value > 0
}

pub(crate) fn relation(state: &GameState, a: PlayerId, b: PlayerId) -> QueryResponse {
    if a == b {
        return QueryResponse::Relation(RelationTag::Self_);
    }
    let Some(pa) = state.players.get(a) else {
        return QueryResponse::Error {
            code: QueryErrorCode::InvalidId,
        };
    };
    let Some(pb) = state.players.get(b) else {
        return QueryResponse::Error {
            code: QueryErrorCode::InvalidId,
        };
    };
    use crate::state::Identity::*;
    let tag = match (pa.identity, pb.identity) {
        (Lord, Loyalist) | (Loyalist, Lord) | (Loyalist, Loyalist) | (Lord, Lord) => {
            RelationTag::Ally
        }
        (Rebel, Rebel) => RelationTag::Ally,
        (Renegade, _) | (_, Renegade) => RelationTag::Enemy,
        _ => RelationTag::Enemy,
    };
    QueryResponse::Relation(tag)
}

pub(crate) fn card_usable_explain(
    state: &GameState,
    player: PlayerId,
    card: engine_api::CardId,
) -> QueryExplanation {
    let Some(p) = state.players.get(player) else {
        return err(QueryErrorCode::InvalidId);
    };
    if !p.alive || p.dying {
        return QueryExplanation {
            response: QueryResponse::Bool(false),
            base: None,
            sources: vec![],
        };
    }
    if state.turn.phase != Phase::Play || state.turn.current != player {
        return QueryExplanation {
            response: QueryResponse::Bool(false),
            base: None,
            sources: vec![],
        };
    }
    let Some(c) = state.cards.get(card) else {
        return err(QueryErrorCode::InvalidId);
    };
    let hand = state.zones.find(ZoneKind::Hand, player);
    if hand != Some(c.zone) || c.owner != player {
        return QueryExplanation {
            response: QueryResponse::Bool(false),
            base: None,
            sources: vec![],
        };
    }
    if card_use_cancelled_for_card(state, player, card) {
        return QueryExplanation {
            response: QueryResponse::Bool(false),
            base: Some(0),
            sources: vec![],
        };
    }
    let kind = effective_card_kind(state, player, c);
    let ok = match kind {
        CardKind::Ying => false,
        CardKind::Shan => false,
        // 无懈：仅响应窗口打出
        CardKind::TrickWuxie | CardKind::TrickYoudishenru => false,
        CardKind::Tao => {
            p.hp < p.max_hp as i32
                && !card_target_cancelled_for_logical_kind(state, player, player, c, CardKind::Tao)
        }
        // 酒：满血也可出（获得 JIU mark）
        CardKind::Jiu => true,
        CardKind::Sha
        | CardKind::ShaFire
        | CardKind::ShaThunder
        | CardKind::Weapon
        | CardKind::Armor
        | CardKind::HorseOff
        | CardKind::HorseDef
        | CardKind::Treasure
        | CardKind::TrickWuzhong
        | CardKind::TrickGuohe
        | CardKind::TrickShunshou
        | CardKind::TrickJuedou
        | CardKind::TrickNanman
        | CardKind::TrickWanjian
        | CardKind::TrickTaoyuan
        | CardKind::TrickWugu
        | CardKind::TrickLe
        | CardKind::TrickBingliang
        | CardKind::TrickShandian
        | CardKind::TrickJiedao
        | CardKind::TrickTiesuo
        | CardKind::TrickHuogong
        | CardKind::TrickYuanjiao
        | CardKind::TrickTiaojiyanmei
        | CardKind::TrickMantianguohai
        | CardKind::TrickDiaobingqianjiang
        | CardKind::TrickWangmeizhike
        | CardKind::TrickYiyi
        | CardKind::TrickShuiyanqijunx
        | CardKind::TrickLulitongxin
        | CardKind::TrickKefuzhongyuan
        | CardKind::TrickGuguoanbang
        | CardKind::TrickChuqibuyi
        | CardKind::TrickHuoshaolianying
        | CardKind::TrickLianjunshengyan
        | CardKind::TrickWenheluanwu
        | CardKind::TrickHaolingtianxia
        | CardKind::TrickChiling
        | CardKind::Other => true,
    };
    QueryExplanation {
        response: QueryResponse::Bool(ok),
        base: None,
        sources: vec![],
    }
}
