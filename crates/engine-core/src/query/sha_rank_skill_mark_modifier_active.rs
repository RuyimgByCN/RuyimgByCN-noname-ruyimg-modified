//! Query operations for `sha_rank_skill_mark_modifier_active` and related helpers.

use super::*;

pub(crate) fn sha_rank_skill_mark_modifier_active(
    state: &GameState,
    player: PlayerId,
    card: engine_api::CardId,
    target: ModifierTarget,
    rank_is_less: bool,
) -> bool {
    let Some(card) = state.cards.get(card) else {
        return false;
    };
    if !effective_card_kind(state, player, card).is_sha() {
        return false;
    }
    let rank = i32::from(card.rank);
    let mods: Vec<_> = state
        .modifiers
        .for_target(target)
        .into_iter()
        .filter(|modifier| {
            if modifier.owner != player {
                return false;
            }
            let threshold = state
                .players
                .get(player)
                .and_then(|owner| {
                    owner
                        .marks
                        .iter()
                        .find(|(mark, _)| *mark == modifier.skill_def.raw())
                })
                .map(|(_, count)| *count)
                .unwrap_or(0);
            threshold > 0
                && if rank_is_less {
                    rank < threshold
                } else {
                    rank == 0 || rank > threshold
                }
        })
        .collect();
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether this exact logical Sha bypasses and does not consume its ordinary use count.
pub fn sha_use_does_not_count_for_card(
    state: &GameState,
    player: PlayerId,
    card: engine_api::CardId,
) -> bool {
    sha_rank_skill_mark_modifier_active(
        state,
        player,
        card,
        ModifierTarget::ShaNoUseCountIfRankGtSkillMark,
        false,
    )
}

/// Marks carried by `target` as `(mark_def, count)` pairs.
pub(crate) fn target_marks(state: &GameState, target: PlayerId) -> Vec<(u32, i32)> {
    state
        .players
        .get(target)
        .map(|player| player.marks.clone())
        .unwrap_or_default()
}

pub(crate) fn skill_bound_target_modifier_applies(
    state: &GameState,
    modifier: &&ActiveModifier,
    owner: PlayerId,
    target: PlayerId,
) -> bool {
    modifier.owner == owner
        && state
            .skills
            .get(modifier.skill_id)
            .and_then(|skill| skill.bound_target())
            == Some(target)
}

pub(crate) fn skill_card_policy_modifier_applies(
    state: &GameState,
    modifier: &&ActiveModifier,
    owner: PlayerId,
    card: engine_api::CardId,
    policy: u8,
) -> bool {
    modifier.owner == owner
        && state
            .skills
            .get(modifier.skill_id)
            .is_some_and(|skill| skill.card_use_policy(card) & policy != 0)
}

/// Whether a proactive Sha may exceed its ordinary use count for this specific target.
pub fn sha_use_unlimited_for_target(state: &GameState, player: PlayerId, target: PlayerId) -> bool {
    let mut mods: Vec<_> = state
        .modifiers
        .for_target(ModifierTarget::ShaUseUnlimitedForSkillBoundTarget)
        .into_iter()
        .filter(|modifier| skill_bound_target_modifier_applies(state, modifier, player, target))
        .collect();
    // 政略②（jsrgzhenglve）：目标持有任一被声明豁免的标记即不计数。
    for (mark, _) in target_marks(state, target) {
        mods.extend(
            state
                .modifiers
                .for_target(ModifierTarget::ShaUseUnlimitedTargetMark(mark))
                .into_iter()
                .filter(|modifier| modifier.owner == player),
        );
    }
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether one concrete card, when resolved as Sha, cannot be answered by its target.
pub fn sha_unrespondable_for_card(
    state: &GameState,
    player: PlayerId,
    card: engine_api::CardId,
) -> bool {
    let Some(card) = state.cards.get(card) else {
        return false;
    };
    let kind = effective_card_kind(state, player, card);
    sha_unrespondable_for_logical_kind(state, player, card, kind)
}

/// Whether one concrete Sha cannot be answered by this specific target.
pub fn sha_unrespondable_for_card_target(
    state: &GameState,
    player: PlayerId,
    target: PlayerId,
    card: engine_api::CardId,
) -> bool {
    let Some(card) = state.cards.get(card) else {
        return false;
    };
    let kind = effective_card_kind(state, player, card);
    if !kind.is_sha() {
        return false;
    }
    if sha_unrespondable_for_logical_kind(state, player, card, kind) {
        return true;
    }
    if state.turn.current == player
        && state
            .players
            .get(player)
            .into_iter()
            .flat_map(|player| player.skills.iter())
            .filter_map(|skill| state.skills.get(*skill))
            .filter(|skill| state.skill_effectively_enabled(skill))
            .filter_map(|skill| skill.luanqun_helper())
            .any(|(behavior, sources)| behavior == 2 && sources.contains(&target))
    {
        return true;
    }
    let target_mods = state
        .modifiers
        .for_target(ModifierTarget::ShaTargetUnrespondable);
    let (target_locked, _, target_vetoed) =
        apply_int_modifiers_for_state(state, 0, &target_mods, Some(target));
    if target_vetoed || target_locked != 0 {
        return true;
    }
    let distance = match distance_explain(state, player, target).response {
        QueryResponse::Int(distance) => distance,
        _ => return false,
    };
    let mods = state
        .modifiers
        .for_target(ModifierTarget::ShaUnrespondableWithinDistance);
    let (threshold, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || (threshold > 0 && distance <= threshold)
}

/// Whether every target of the source's next Sha is one of its Luanqun owners.
pub(crate) fn luanqun_sha_targets_allowed(
    state: &GameState,
    player: PlayerId,
    targets: &[PlayerId],
) -> bool {
    if state.turn.current != player || state.turn.phase != crate::state::Phase::Play {
        return true;
    }
    let allowed: std::collections::BTreeSet<_> = state
        .players
        .get(player)
        .into_iter()
        .flat_map(|player| player.skills.iter())
        .filter_map(|skill| state.skills.get(*skill))
        .filter(|skill| state.skill_effectively_enabled(skill))
        .filter_map(|skill| skill.luanqun_helper())
        .filter(|(behavior, _)| *behavior == 1)
        .flat_map(|(_, sources)| sources)
        .collect();
    allowed.is_empty() || targets.iter().all(|target| allowed.contains(target))
}

/// Whether one whole Sha use is unrespondable because its source or any target owns a matching
/// mutual-distance modifier. Distance is measured from the other participant to the owner, which
/// preserves directional distance modifiers such as defensive mounts.
pub fn sha_use_unrespondable_within_distance(
    state: &GameState,
    source: PlayerId,
    targets: &[PlayerId],
) -> bool {
    state
        .modifiers
        .for_target(ModifierTarget::ShaUseUnrespondableWithinDistance)
        .into_iter()
        .any(|modifier| {
            let owner = modifier.owner;
            let threshold = modifier.value;
            if threshold <= 0 {
                return false;
            }
            let within = |from, to| {
                matches!(
                    distance_explain(state, from, to).response,
                    QueryResponse::Int(distance) if distance <= threshold
                )
            };
            if owner == source {
                targets.iter().any(|target| within(*target, owner))
            } else {
                targets.contains(&owner) && within(source, owner)
            }
        })
}

/// Whether `responder` may not answer the current card used by `source` because a source-owned
/// modifier covers the final directional distance between them.
pub fn card_use_unrespondable_within_distance(
    state: &GameState,
    source: PlayerId,
    responder: PlayerId,
) -> bool {
    if source == responder {
        return false;
    }
    let covered = |target, from, to| {
        let mods = state.modifiers.for_target(target);
        let (threshold, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(source));
        if vetoed {
            return true;
        }
        if threshold <= 0 {
            return false;
        }
        matches!(
            distance_explain(state, from, to).response,
            QueryResponse::Int(distance) if distance <= threshold
        )
    };
    covered(
        ModifierTarget::CardUseUnrespondableWithinResponderToOwnerDistance,
        responder,
        source,
    ) || covered(
        ModifierTarget::CardUseUnrespondableWithinOwnerToResponderDistance,
        source,
        responder,
    )
}

/// Whether the current logical card used by `source` cannot be answered by another player.
pub fn card_use_unrespondable_for_kind(
    state: &GameState,
    source: PlayerId,
    logical_kind: CardKind,
    suit: engine_api::Suit,
) -> bool {
    let mods = state
        .modifiers
        .for_card_use_unrespondable(logical_kind, suit);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(source));
    vetoed || value != 0
}

/// tanhu3（批 170，内容层变体）：目标集合中存在「拥有某 def 技能、且任意角色挂有
/// 对应 `CardUseUnrespondableSkillOnOwner(def)` 声明」的角色时，以该角色为目标的
/// 锦囊不可被无懈响应。以目标侧 def 反查声明（与 tanhu3 距离变体同构）。
pub fn wuxie_blocked_by_target_skill_declarations(state: &GameState, targets: &[PlayerId]) -> bool {
    targets.iter().any(|target| {
        state
            .skills
            .sorted_live()
            .iter()
            .filter(|instance| instance.owner == *target)
            .any(|instance| {
                !state
                    .modifiers
                    .for_target(ModifierTarget::CardUseUnrespondableSkillOnOwner(
                        instance.def.raw(),
                    ))
                    .is_empty()
            })
    })
}

/// Whether 势施 froze `responder` into the direct-hit set of this exact physical card use.
pub(crate) fn potshishi_blocks_response(
    state: &GameState,
    source: PlayerId,
    card: engine_api::CardId,
    responder: PlayerId,
) -> bool {
    card.is_some()
        && source != responder
        && state
            .players
            .get(source)
            .into_iter()
            .flat_map(|player| player.skills.iter())
            .filter_map(|skill| state.skills.get(*skill))
            .filter(|skill| state.skill_effectively_enabled(skill))
            .any(|skill| skill.potshishi_blocks(card, responder))
}

/// Colors that `player` discarded this round for the exact 颂谳 Wuzhong resolution.
pub(crate) fn potsongyan_response_colors(
    state: &GameState,
    card: engine_api::CardId,
    player: PlayerId,
) -> u8 {
    if card.is_none() {
        return 0;
    }
    state
        .skills
        .sorted_live()
        .into_iter()
        .filter(|skill| state.skill_effectively_enabled(skill))
        .fold(0, |mask, skill| {
            mask | skill.potsongyan_response_colors(state.turn.round, card, player)
        })
        & 0b11
}

/// Whether `card`, when converted to `logical_kind`, cannot be answered as Sha.
pub fn sha_unrespondable_for_logical_kind(
    state: &GameState,
    player: PlayerId,
    card: &crate::state::Card,
    logical_kind: CardKind,
) -> bool {
    if !logical_kind.is_sha() {
        return false;
    }
    let mut mods = state
        .modifiers
        .for_sha_unrespondable(logical_kind, effective_card_suit(state, player, card));
    mods.extend(
        state
            .modifiers
            .for_target(ModifierTarget::CardUseUnrespondableForSkillCardPolicy)
            .into_iter()
            .filter(|modifier| {
                skill_card_policy_modifier_applies(
                    state,
                    modifier,
                    player,
                    card.id,
                    crate::state::SkillInstance::CARD_USE_POLICY_UNRESPONDABLE,
                )
            }),
    );
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether one concrete card use cannot be answered by Wuxie.
pub fn card_use_unrespondable_for_card(
    state: &GameState,
    source: PlayerId,
    card: engine_api::CardId,
    logical_kind: CardKind,
    suit: engine_api::Suit,
) -> bool {
    let mut mods = state
        .modifiers
        .for_card_use_unrespondable(logical_kind, suit);
    mods.extend(
        state
            .modifiers
            .for_target(ModifierTarget::CardUseUnrespondableForSkillCardPolicy)
            .into_iter()
            .filter(|modifier| {
                skill_card_policy_modifier_applies(
                    state,
                    modifier,
                    source,
                    card,
                    crate::state::SkillInstance::CARD_USE_POLICY_UNRESPONDABLE,
                )
            }),
    );
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(source));
    vetoed || value != 0
}

/// Whether cards used by `player` ignore their ordinary target-distance limit.
pub fn card_distance_unlimited_for(state: &GameState, player: PlayerId) -> bool {
    let mods = state
        .modifiers
        .for_target(ModifierTarget::CardDistanceUnlimited);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether a use resolved as 杀 ignores distance, including view-as 杀.
pub fn sha_distance_unlimited_for(state: &GameState, player: PlayerId) -> bool {
    let mut mods = state
        .modifiers
        .for_target(ModifierTarget::CardDistanceUnlimited);
    mods.extend(
        state
            .modifiers
            .for_target(ModifierTarget::CardDistanceUnlimitedFor(CardSelector::Sha)),
    );
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether one concrete card used by `player` ignores its target-distance limit.
/// 祸首：南蛮实体牌伤害的来源重定向为修正器持有者（ABI 541）。
pub fn nanman_damage_source_for(
    state: &GameState,
    source: PlayerId,
    card: engine_api::CardId,
) -> Option<PlayerId> {
    let is_nanman = state.cards.get(card).is_some_and(|value| {
        crate::query::effective_card_kind(state, source, value) == CardKind::TrickNanman
    });
    if !is_nanman {
        return None;
    }
    state
        .modifiers
        .for_target(ModifierTarget::NanmanDamageSourceAsOwner)
        .into_iter()
        .filter(|modifier| {
            modifier.owner != source
                && state
                    .players
                    .get(modifier.owner)
                    .is_some_and(|owner| owner.alive)
        })
        .map(|modifier| modifier.owner)
        .next()
}

/// Whether one concrete card ignores its target-distance limit.
pub fn card_distance_unlimited_for_card(
    state: &GameState,
    player: PlayerId,
    card: engine_api::CardId,
) -> bool {
    if sha_rank_skill_mark_modifier_active(
        state,
        player,
        card,
        ModifierTarget::ShaDistanceUnlimitedIfRankLtSkillMark,
        true,
    ) {
        return true;
    }
    let Some(card) = state.cards.get(card) else {
        return false;
    };
    if chenglve_suit_unlimited_for(state, player, card) {
        return true;
    }
    let kind = effective_card_kind(state, player, card);
    let mut mods = state
        .modifiers
        .for_card_distance(kind, effective_card_suit(state, player, card));
    mods.extend(
        state
            .modifiers
            .for_target(ModifierTarget::CardDistanceUnlimitedForSkillCardPolicy)
            .into_iter()
            .filter(|modifier| {
                skill_card_policy_modifier_applies(
                    state,
                    modifier,
                    player,
                    card.id,
                    crate::state::SkillInstance::CARD_USE_POLICY_DISTANCE_UNLIMITED,
                )
            }),
    );
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether one concrete card ignores distance to this specific target.
pub fn card_distance_unlimited_for_target(
    state: &GameState,
    player: PlayerId,
    card: engine_api::CardId,
    target: PlayerId,
) -> bool {
    if card_distance_unlimited_for_card(state, player, card) {
        return true;
    }
    if sha_target_distance_lte_rank_modifier_active(state, player, card, target) {
        return true;
    }
    let bound_mods: Vec<_> = state
        .modifiers
        .for_target(ModifierTarget::CardDistanceUnlimitedForSkillBoundTarget)
        .into_iter()
        .filter(|modifier| skill_bound_target_modifier_applies(state, modifier, player, target))
        .collect();
    let (bound_value, _, bound_vetoed) =
        apply_int_modifiers_for_state(state, 0, &bound_mods, Some(player));
    if bound_vetoed || bound_value != 0 {
        return true;
    }
    // 政略②（jsrgzhenglve）：目标持有任一被声明豁免的标记即无视距离。
    let mut mark_mods = Vec::new();
    for (mark, _) in target_marks(state, target) {
        mark_mods.extend(
            state
                .modifiers
                .for_target(ModifierTarget::CardDistanceUnlimitedTargetMark(mark))
                .into_iter()
                .filter(|modifier| modifier.owner == player),
        );
    }
    let (mark_value, _, mark_vetoed) =
        apply_int_modifiers_for_state(state, 0, &mark_mods, Some(player));
    if mark_vetoed || mark_value != 0 {
        return true;
    }
    // tanhu3（批 168，ABI 679）：任意目标持有被声明的技能（def）时，任何使用者
    // 对其无视距离（与政略②方向相反：技能在目标侧；modifier 为全局声明，
    // 不限定使用方或持有方）。
    let mut on_owner_mods = Vec::new();
    for skill_def in state
        .skills
        .sorted_live()
        .iter()
        .filter(|instance| instance.owner == target)
        .map(|instance| instance.def.raw())
    {
        on_owner_mods.extend(
            state
                .modifiers
                .for_target(ModifierTarget::CardDistanceUnlimitedSkillOnOwner(skill_def)),
        );
    }
    // 目标侧声明：不做 owner_filter（modifier 挂在声明技持有者身上，作用于任意使用方）。
    let (on_owner_value, _, on_owner_vetoed) =
        apply_int_modifiers_for_state(state, 0, &on_owner_mods, None);
    if on_owner_vetoed || on_owner_value != 0 {
        return true;
    }
    if state.hand_cards(target).len() < state.hand_cards(player).len() {
        return false;
    }
    let Some(card) = state.cards.get(card) else {
        return false;
    };
    let kind = effective_card_kind(state, player, card);
    let mods = state
        .modifiers
        .for_card_distance_if_target_hand_gte_owner(kind, effective_card_suit(state, player, card));
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

pub(crate) fn sha_target_distance_lte_rank_modifier_active(
    state: &GameState,
    player: PlayerId,
    card: engine_api::CardId,
    target: PlayerId,
) -> bool {
    let Some(card) = state.cards.get(card) else {
        return false;
    };
    if card.rank == 0 || !effective_card_kind(state, player, card).is_sha() {
        return false;
    }
    let within_rank = matches!(
        query(
            state,
            &QueryRequest::Distance {
                from: player,
                to: target,
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(distance) if distance <= i32::from(card.rank)
    );
    if !within_rank {
        return false;
    }
    let mods: Vec<_> = state
        .modifiers
        .for_target(ModifierTarget::ShaDistanceUnlimitedIfTargetDistanceLteRank)
        .into_iter()
        .filter(|modifier| modifier.owner == player)
        .collect();
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether another player is forbidden from discarding this physical equipment card.
pub fn card_discard_protected_from(
    state: &GameState,
    actor: PlayerId,
    target: PlayerId,
    card: engine_api::CardId,
) -> bool {
    if actor == target {
        return false;
    }
    let Some(card) = state.cards.get(card) else {
        return false;
    };
    let Some(equip) = state.zones.find(ZoneKind::Equip, target) else {
        return false;
    };
    if card.owner != target || card.zone != equip {
        return false;
    }
    let mods = state
        .modifiers
        .for_card_discard_protection(card.kind, card.suit);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(target));
    vetoed || value != 0
}

/// Whether another player is forbidden from gaining this physical card from the target.
pub fn card_gain_protected_from(
    state: &GameState,
    actor: PlayerId,
    target: PlayerId,
    card: CardId,
) -> bool {
    if actor == target {
        return false;
    }
    let Some(card) = state.cards.get(card) else {
        return false;
    };
    let in_target_personal_zone = card.owner == target
        && [ZoneKind::Hand, ZoneKind::Equip, ZoneKind::Judge]
            .into_iter()
            .any(|kind| state.zones.find(kind, target) == Some(card.zone));
    if !in_target_personal_zone {
        return false;
    }
    let mods = state
        .modifiers
        .for_card_gain_protection(card.kind, card.suit);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(target));
    vetoed || value != 0
}

/// Cards the owner may choose for a self-discard from hand or equipment.
pub fn discardable_hand_equip_cards(state: &GameState, player: PlayerId) -> Vec<CardId> {
    let mut cards = state.hand_cards(player);
    if let Some(equip) = state
        .zones
        .find(ZoneKind::Equip, player)
        .and_then(|zone| state.zones.get(zone))
    {
        cards.extend(equip.cards.iter().copied());
    }
    cards
}

/// Current hand cards eligible for Yeyan's four-distinct-suit cost.
pub fn yeyan_hand_cards(state: &GameState, player: PlayerId) -> Vec<CardId> {
    state
        .hand_cards(player)
        .into_iter()
        .filter(|card| {
            state.cards.get(*card).is_some_and(|card| {
                effective_card_suit(state, player, card) != engine_api::Suit::None
            })
        })
        .collect()
}

/// Effective basic cards in a player's hand that may pay Standard Xiaoguo's cost.
pub fn standard_xiaoguo_basic_hand_cards(state: &GameState, player: PlayerId) -> Vec<CardId> {
    state
        .hand_cards(player)
        .into_iter()
        .filter(|card| {
            state
                .cards
                .get(*card)
                .is_some_and(|card| effective_card_kind(state, player, card).is_basic())
        })
        .collect()
}

/// Effective equipment cards in a player's hand/equipment zones for Standard Xiaoguo.
pub fn standard_xiaoguo_equipment_cards(state: &GameState, player: PlayerId) -> Vec<CardId> {
    discardable_hand_equip_cards(state, player)
        .into_iter()
        .filter(|card| {
            state
                .cards
                .get(*card)
                .is_some_and(|card| effective_card_kind(state, player, card).is_equip())
        })
        .collect()
}

/// Hand cards that may pay SP Xiaoguo's unrestricted self-discard cost.
pub fn sp_xiaoguo_hand_cards(state: &GameState, player: PlayerId) -> Vec<CardId> {
    state.hand_cards(player)
}

/// Effective equipment cards the phase player may self-discard for SP Xiaoguo.
pub fn sp_xiaoguo_equipment_cards(state: &GameState, player: PlayerId) -> Vec<CardId> {
    discardable_hand_equip_cards(state, player)
        .into_iter()
        .filter(|card| {
            state
                .cards
                .get(*card)
                .is_some_and(|card| effective_card_kind(state, player, card).is_equip())
        })
        .collect()
}

/// Discardable non-basic hand/equipment cards whose effective color matches at least one of the
/// player's currently equipped cards.
pub fn discardable_non_basic_matching_equip_color_cards(
    state: &GameState,
    player: PlayerId,
) -> Vec<CardId> {
    let equip_color_mask = state
        .zones
        .find(ZoneKind::Equip, player)
        .and_then(|zone| state.zones.get(zone))
        .map(|zone| {
            zone.cards.iter().fold(0, |mask, card| {
                mask | effective_card_color_mask(state, player, *card)
            })
        })
        .unwrap_or(0);
    if equip_color_mask == 0 {
        return Vec::new();
    }
    discardable_hand_equip_cards(state, player)
        .into_iter()
        .filter(|card| {
            state.cards.get(*card).is_some_and(|card_state| {
                !effective_card_kind(state, player, card_state).is_basic()
                    && effective_card_color_mask(state, player, *card) & equip_color_mask != 0
            })
        })
        .collect()
}

/// Equipment cards that `actor` may discard from `target`.
pub fn discardable_equipment_cards(
    state: &GameState,
    actor: PlayerId,
    target: PlayerId,
) -> Vec<CardId> {
    state
        .zones
        .find(ZoneKind::Equip, target)
        .and_then(|zone| state.zones.get(zone))
        .map(|zone| {
            zone.cards
                .iter()
                .copied()
                .filter(|card| !card_discard_protected_from(state, actor, target, *card))
                .collect()
        })
        .unwrap_or_default()
}

/// Cards that `actor` may discard from `target` within a canonical target-card scope.
///
/// The returned order is stable by zone order (hand, equipment, judgment) and then canonical
/// zone order. Equipment discard protection is applied through the shared modifier query.
pub fn discardable_target_cards(
    state: &GameState,
    actor: PlayerId,
    target: PlayerId,
    scope: rule_api::TargetCardScope,
) -> Vec<CardId> {
    let kinds = [
        scope.includes_hand().then_some(ZoneKind::Hand),
        scope.includes_equip().then_some(ZoneKind::Equip),
        scope.includes_judge().then_some(ZoneKind::Judge),
    ];
    let mut cards: Vec<_> = kinds
        .into_iter()
        .flatten()
        .filter_map(|kind| state.zones.find(kind, target))
        .filter_map(|zone| state.zones.get(zone))
        .flat_map(|zone| zone.cards.iter().copied())
        .filter(|card| !card_discard_protected_from(state, actor, target, *card))
        .collect();
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    cards
}

/// Cards that `actor` may gain from `target` within a canonical target-card scope.
///
/// The returned order is stable by physical card id. Target-owned gain protection is applied
/// through the shared modifier query for hand, equipment, and judgment cards alike.
pub fn gainable_target_cards(
    state: &GameState,
    actor: PlayerId,
    target: PlayerId,
    scope: rule_api::TargetCardScope,
) -> Vec<CardId> {
    let kinds = [
        scope.includes_hand().then_some(ZoneKind::Hand),
        scope.includes_equip().then_some(ZoneKind::Equip),
        scope.includes_judge().then_some(ZoneKind::Judge),
    ];
    let mut cards: Vec<_> = kinds
        .into_iter()
        .flatten()
        .filter_map(|kind| state.zones.find(kind, target))
        .filter_map(|zone| state.zones.get(zone))
        .flat_map(|zone| zone.cards.iter().copied())
        .filter(|card| !card_gain_protected_from(state, actor, target, *card))
        .collect();
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    cards
}

/// Living previous/next seats around `player`, deduplicated in a two-player game.
pub fn alive_seat_neighbors(state: &GameState, player: PlayerId) -> Vec<PlayerId> {
    let alive = state.players.alive_by_seat();
    let Some(index) = alive.iter().position(|candidate| *candidate == player) else {
        return Vec::new();
    };
    if alive.len() < 2 {
        return Vec::new();
    }
    let previous = alive[(index + alive.len() - 1) % alive.len()];
    let next = alive[(index + 1) % alive.len()];
    if previous == next {
        vec![previous]
    } else {
        vec![previous, next]
    }
}

/// Discardable armor equipment cards that `actor` may discard from `target`.
pub fn discardable_armor_equipment_cards(
    state: &GameState,
    actor: PlayerId,
    target: PlayerId,
) -> Vec<CardId> {
    discardable_equipment_cards(state, actor, target)
        .into_iter()
        .filter(|card| {
            state
                .cards
                .get(*card)
                .and_then(|card| card.kind.equip_slot())
                == Some(crate::state::EquipSlot::Armor)
        })
        .collect()
}

/// Whether two targets form a legal equipment-discard pair containing armor.
pub fn equipment_pair_with_armor_legal(
    state: &GameState,
    actor: PlayerId,
    targets: &[PlayerId],
) -> bool {
    targets.len() == 2
        && targets[0] != targets[1]
        && targets.iter().all(|target| {
            state
                .players
                .get(*target)
                .is_some_and(|player| player.alive)
                && !discardable_equipment_cards(state, actor, *target).is_empty()
        })
        && targets
            .iter()
            .any(|target| !discardable_armor_equipment_cards(state, actor, *target).is_empty())
}

/// Whether `player` is currently forbidden from responding with 杀.
pub fn sha_response_cancelled_for(state: &GameState, player: PlayerId) -> bool {
    let mods = state
        .modifiers
        .for_target(ModifierTarget::ShaResponseCancel);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether `player` is currently forbidden from proactively using any card.
pub fn card_use_cancelled_for(state: &GameState, player: PlayerId) -> bool {
    let mods = state.modifiers.for_target(ModifierTarget::CardUseCancel);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether `player` is forbidden from using one concrete card.
pub fn card_use_cancelled_for_card(
    state: &GameState,
    player: PlayerId,
    card: engine_api::CardId,
) -> bool {
    let Some(card) = state.cards.get(card) else {
        return card_use_cancelled_for(state, player);
    };
    if ddd_lianer_card_blocked(state, player, card) {
        return true;
    }
    card_use_cancelled_for_logical_kind(
        state,
        player,
        card,
        effective_card_kind(state, player, card),
    )
}

pub(crate) fn card_use_cancelled_for_logical_kind(
    state: &GameState,
    player: PlayerId,
    card: &crate::state::Card,
    logical_kind: crate::state::CardKind,
) -> bool {
    if logical_kind == crate::state::CardKind::Tao
        && tao_cancelled_for_non_dying_other_during_owner_turn(state, player, None)
    {
        return true;
    }
    let mut mods = state
        .modifiers
        .for_card_use(logical_kind, effective_card_suit(state, player, card));
    mods.extend(card_color_lock_modifiers_for_suit(
        state,
        player,
        effective_card_suit(state, player, card),
    ));
    if state
        .zones
        .find(ZoneKind::Hand, player)
        .is_some_and(|hand| card.zone == hand)
    {
        mods.extend(
            state
                .modifiers
                .for_target(ModifierTarget::HandCardUseResponseCancel),
        );
    }
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

/// Stable red/black bit used by versioned skill color-lock state.
pub(crate) const fn card_color_mask_for_suit(suit: engine_api::Suit) -> u8 {
    match suit {
        engine_api::Suit::Heart | engine_api::Suit::Diamond => 0b01,
        engine_api::Suit::Spade | engine_api::Suit::Club => 0b10,
        engine_api::Suit::None => 0,
    }
}

/// Effective red/black bit of one physical card for one player.
pub(crate) fn effective_card_color_mask(
    state: &GameState,
    player: PlayerId,
    card: engine_api::CardId,
) -> u8 {
    state
        .cards
        .get(card)
        .map(|card| card_color_mask_for_suit(effective_card_suit(state, player, card)))
        .unwrap_or(0)
}

/// Whether a physical response material matches the color lock on the innermost card use.
pub(crate) fn response_color_locked_for_current_card(
    state: &GameState,
    player: PlayerId,
    card: engine_api::CardId,
) -> bool {
    let Some(record) = state.resolution.pending_card_resolutions.last() else {
        return false;
    };
    player != record.source
        && record.response_color_lock_mask != 0
        && effective_card_color_mask(state, player, card) & record.response_color_lock_mask != 0
}

pub(crate) fn card_color_lock_modifiers_for_suit(
    state: &GameState,
    player: PlayerId,
    suit: engine_api::Suit,
) -> Vec<&ActiveModifier> {
    let color = card_color_mask_for_suit(suit);
    if color == 0 {
        return Vec::new();
    }
    state
        .modifiers
        .for_target(ModifierTarget::CardActionCancelForSkillColorMask)
        .into_iter()
        .filter(|modifier| {
            modifier.owner == player
                && state
                    .skills
                    .get(modifier.skill_id)
                    .and_then(|skill| skill.card_color_lock())
                    .is_some_and(|(_, mask)| mask & color != 0)
        })
        .collect()
}

/// Whether a physical card is blocked by a temporary skill-instance color or rank lock.
pub(crate) fn card_color_locked_for_card(
    state: &GameState,
    player: PlayerId,
    card: engine_api::CardId,
) -> bool {
    let Some(card) = state.cards.get(card) else {
        return false;
    };
    if ddd_lianer_card_blocked(state, player, card) {
        return true;
    }
    let mods =
        card_color_lock_modifiers_for_suit(state, player, effective_card_suit(state, player, card));
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

pub(crate) fn ddd_lianer_card_blocked(
    state: &GameState,
    player: PlayerId,
    card: &crate::state::Card,
) -> bool {
    state.players.get(player).is_some_and(|owner| {
        owner.skills.iter().any(|skill| {
            state.skills.get(*skill).is_some_and(|instance| {
                instance.def.raw() == 1_083_711_762
                    && state.skill_effectively_enabled(instance)
                    && instance.mark > 0
                    && (card.rank == 0 || u32::from(card.rank) >= instance.mark)
            })
        })
    })
}

/// Whether `player` is forbidden from using hand cards as use/response inputs.
pub fn hand_card_use_response_cancelled_for(state: &GameState, player: PlayerId) -> bool {
    let current = state.turn.current;
    if player != current
        && state.players.get(current).is_some_and(|participant| {
            participant.alive
                && participant.skills.iter().any(|skill| {
                    state.skills.get(*skill).is_some_and(|instance| {
                        instance.def.raw() == 1_632_230_116
                            && state.skill_effectively_enabled(instance)
                    })
                })
                && state
                    .hand_cards(current)
                    .into_iter()
                    .all(|card| state.cards.get(card).is_some_and(|card| card.face_up))
        })
    {
        return true;
    }
    let mods = state
        .modifiers
        .for_target(ModifierTarget::HandCardUseResponseCancel);
    let (value, _, vetoed) = apply_int_modifiers(0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether a Play-phase modifier restricts this player to hand cards gained this turn.
pub fn hand_card_gained_this_turn_only_for(state: &GameState, player: PlayerId) -> bool {
    if state.turn.current != player || state.turn.phase != crate::state::Phase::Play {
        return false;
    }
    let mods = state
        .modifiers
        .for_target(ModifierTarget::HandCardGainedThisTurnOnly);
    let (value, _, vetoed) = apply_int_modifiers(0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether `player` is currently forbidden from responding with any card.
pub fn card_response_cancelled_for(state: &GameState, player: PlayerId) -> bool {
    let mods = state
        .modifiers
        .for_target(ModifierTarget::CardResponseCancel);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether one physical card is forbidden as a response under its supplied logical kind.
pub(crate) fn card_response_cancelled_for_logical_kind(
    state: &GameState,
    player: PlayerId,
    card: &crate::state::Card,
    logical_kind: CardKind,
) -> bool {
    if ddd_lianer_card_blocked(state, player, card) {
        return true;
    }
    let mods = state
        .modifiers
        .for_card_response(logical_kind, effective_card_suit(state, player, card));
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether `player` is currently forbidden from using a card to save a dying player.
pub fn card_save_cancelled_for(state: &GameState, player: PlayerId) -> bool {
    card_save_cancelled_for_kind(state, player, CardKind::Tao)
}

/// Whether `player` is forbidden from using one logical card kind to save themselves.
pub fn card_save_cancelled_for_kind(state: &GameState, player: PlayerId, kind: CardKind) -> bool {
    card_save_cancelled_for_kind_to(state, player, player, kind)
}

/// Whether `player` is forbidden from using one logical card kind to save `dying`.
pub fn card_save_cancelled_for_kind_to(
    state: &GameState,
    player: PlayerId,
    dying: PlayerId,
    kind: CardKind,
) -> bool {
    card_save_cancelled_without_hand_source(state, player, dying, kind)
        || hand_card_use_response_cancelled_for(state, player)
}

pub(crate) fn card_save_cancelled_without_hand_source(
    state: &GameState,
    player: PlayerId,
    dying: PlayerId,
    kind: CardKind,
) -> bool {
    if kind == CardKind::Tao
        && tao_cancelled_for_non_dying_other_during_owner_turn(state, player, Some(dying))
    {
        return true;
    }
    if kind == CardKind::Tao && tao_cancelled_for_dying_beyond_distance(state, player, dying) {
        return true;
    }
    let mut mods = state.modifiers.for_target(ModifierTarget::CardSaveCancel);
    if let Some(selector) = match kind {
        CardKind::Tao => Some(CardSelector::Tao),
        CardKind::Jiu => Some(CardSelector::Jiu),
        _ => None,
    } {
        mods.extend(
            state
                .modifiers
                .for_target(ModifierTarget::CardSaveCancelFor(selector)),
        );
    }
    let (value, _, vetoed) = apply_int_modifiers(0, &mods, Some(player));
    vetoed || value != 0 || card_target_cancelled_for_kind(state, player, dying, kind)
}

pub(crate) fn tao_cancelled_for_non_dying_other_during_owner_turn(
    state: &GameState,
    player: PlayerId,
    dying: Option<PlayerId>,
) -> bool {
    let Some(participant) = state.players.get(player) else {
        return false;
    };
    let is_current_dying_player = dying.map_or(participant.dying, |dying| dying == player);
    if !participant.alive || is_current_dying_player {
        return false;
    }
    let owner = state.turn.current;
    if owner.is_none()
        || owner == player
        || !state
            .players
            .get(owner)
            .is_some_and(|participant| participant.alive)
    {
        return false;
    }
    let mods = state
        .modifiers
        .for_target(ModifierTarget::TaoUseSaveCancelForNonDyingOthersDuringOwnerTurn);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(owner));
    vetoed || value != 0
}

/// Whether `player`'s 桃 save is cancelled because the dying target is beyond distance 2 and
/// `player` carries a `tao_use_save_cancel_for_dying_beyond_distance` modifier. Powers 「泣梓」.
pub(crate) fn tao_cancelled_for_dying_beyond_distance(
    state: &GameState,
    player: PlayerId,
    dying: PlayerId,
) -> bool {
    if player == dying {
        return false;
    }
    let mods = state
        .modifiers
        .for_target(ModifierTarget::TaoUseSaveCancelForDyingBeyondDistance);
    if mods.is_empty() {
        return false;
    }
    let (_value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    if !(vetoed || _value != 0) {
        return false;
    }
    base_distance(state, player, dying).map_or(false, |d| d > 2)
}

/// Whether one physical card satisfies a declarative view-as filter.
pub(crate) fn view_as_filter_matches(
    state: &GameState,
    player: PlayerId,
    filter: &str,
    card: &Card,
) -> bool {
    let suit = effective_card_suit(state, player, card);
    let kind = effective_card_kind(state, player, card);
    match filter {
        "any" => true,
        "red" => matches!(suit, Suit::Heart | Suit::Diamond),
        "black" => matches!(suit, Suit::Spade | Suit::Club),
        "heart" => suit == Suit::Heart,
        "spade" => suit == Suit::Spade,
        "club" => suit == Suit::Club,
        "diamond" => suit == Suit::Diamond,
        "owner_dying_club" => {
            state.players.get(player).is_some_and(|owner| owner.hp <= 0) && suit == Suit::Club
        }
        "owner_outside_turn" => state.turn.current != player,
        "owner_outside_turn_red" => {
            state.turn.current != player && matches!(suit, Suit::Heart | Suit::Diamond)
        }
        "owner_single_hand" => state.hand_count(player) == 1,
        "owner_single_red_hand" => {
            matches!(suit, Suit::Heart | Suit::Diamond)
                && state
                    .hand_cards(player)
                    .into_iter()
                    .filter_map(|card| state.cards.get(card))
                    .filter(|card| {
                        matches!(
                            effective_card_suit(state, player, card),
                            Suit::Heart | Suit::Diamond
                        )
                    })
                    .count()
                    == 1
        }
        "owner_hand_gt_hp" => state
            .players
            .get(player)
            .is_some_and(|owner| state.hand_count(player) as i32 > owner.hp),
        "owner_hp_le_two_black_trick" => {
            state.players.get(player).is_some_and(|owner| owner.hp <= 2)
                && matches!(suit, Suit::Spade | Suit::Club)
                && kind.is_trick()
        }
        "card_sha" => kind.is_sha(),
        "card_shan" => kind == CardKind::Shan,
        "card_tao" => kind == CardKind::Tao,
        "card_jiu" => kind == CardKind::Jiu,
        "card_ying" => kind == CardKind::Ying,
        "card_basic" => kind.is_basic(),
        "card_equip" => kind.is_equip(),
        "card_trick" => kind.is_trick(),
        "damage_trick" => kind.is_damage_trick(),
        "non_basic" => !kind.is_basic(),
        "color_ne_shuangxiong" => {
            let recorded = state
                .players
                .get(player)
                .map(|owner| {
                    owner
                        .marks
                        .iter()
                        .find(|(mark, _)| *mark == crate::effect::SHUANGXIONG_COLOR_MARK)
                        .map(|(_, count)| *count)
                        .unwrap_or(0)
                })
                .unwrap_or(0);
            match recorded {
                // Red recorded: black cards qualify.
                1 => matches!(suit, Suit::Spade | Suit::Club),
                // Black recorded: red cards qualify.
                2 => matches!(suit, Suit::Heart | Suit::Diamond),
                _ => false,
            }
        }
        "black_basic_equip" => {
            matches!(suit, Suit::Spade | Suit::Club) && (kind.is_basic() || kind.is_equip())
        }
        _ => false,
    }
}

/// Whether a physical card is in the source zone declared by a view-as rule.
pub(crate) fn view_as_source_zone_matches(
    state: &GameState,
    player: PlayerId,
    source_zone: Option<&str>,
    card: &Card,
) -> bool {
    let borrowed_hand = borrowed_hand_source(state, player, card.id).is_some();
    let in_hand = state.zones.find(ZoneKind::Hand, player) == Some(card.zone) || borrowed_hand;
    let in_equip =
        card.owner == player && state.zones.find(ZoneKind::Equip, player) == Some(card.zone);
    let in_expansion = state.zones.get(card.zone).is_some_and(|zone| {
        card.owner == player && zone.kind == ZoneKind::Expansion && zone.owner == player
    });
    match source_zone.unwrap_or("hand") {
        "hand" => in_hand,
        "equip" => in_equip,
        "hand_equip" => in_hand || in_equip,
        "hand_expansion" => in_hand || in_expansion,
        "hand_equip_expansion" => in_hand || in_equip || in_expansion,
        _ => false,
    }
}

/// First deterministic self-save card: logical 桃 before 酒, then natural/skill/source order.
pub fn dying_save_card(state: &GameState, player: PlayerId) -> Option<CardId> {
    dying_save_card_for(state, player, player)
}

/// First deterministic card with which `player` may save `dying`. Other players may use Tao;
/// only the dying player may additionally self-save with Jiu.
pub fn dying_save_card_for(state: &GameState, player: PlayerId, dying: PlayerId) -> Option<CardId> {
    dying_save_card_and_kind_for(state, player, dying).map(|(card, _)| card)
}

/// First deterministic save card together with the logical Tao/Jiu kind it supplies.
pub(crate) fn dying_save_card_and_kind_for(
    state: &GameState,
    player: PlayerId,
    dying: PlayerId,
) -> Option<(CardId, CardKind)> {
    let hand = state.playable_hand_cards(player);
    [(CardKind::Tao, "tao"), (CardKind::Jiu, "jiu")]
        .into_iter()
        .filter(|(kind, _)| *kind != CardKind::Jiu || player == dying)
        .find_map(|(kind, as_name)| {
            if let Some(card) = hand.iter().copied().find(|card| {
                !card_save_cancelled_for_kind_to(state, player, dying, kind)
                    && !card_color_locked_for_card(state, player, *card)
                    && state
                        .cards
                        .get(*card)
                        .is_some_and(|card| effective_card_kind(state, player, card) == kind)
            }) {
                return Some((card, kind));
            }
            let owner = state.players.get(player)?;
            owner.skills.iter().find_map(|skill_id| {
                let skill = state.skills.get(*skill_id)?;
                if !state.skill_effectively_enabled(skill) {
                    return None;
                }
                state
                    .content
                    .view_as_defs_for_skill(skill.def)
                    .into_iter()
                    .filter(|view| {
                        view.as_kind == as_name
                            && matches!(view.window.as_str(), "any" | "response")
                            && view.max_cards == 1
                            && !view.owner_hp_min_one_cards
                            && !view.owner_basic_uses_plus_one_cards
                    })
                    .find_map(|view| {
                        state
                            .cards
                            .sorted_live()
                            .into_iter()
                            .find(|card| {
                                let in_hand =
                                    state.zones.find(ZoneKind::Hand, player) == Some(card.zone);
                                !card_save_cancelled_without_hand_source(state, player, dying, kind)
                                    && !card_color_locked_for_card(state, player, card.id)
                                    && (!in_hand
                                        || !hand_card_use_response_cancelled_for(state, player))
                                    && view_as_source_zone_matches(
                                        state,
                                        player,
                                        view.source_zone.as_deref(),
                                        card,
                                    )
                                    && view_as_filter_matches(state, player, &view.filter, card)
                            })
                            .map(|card| (card.id, kind))
                    })
            })
        })
}

/// Whether a logical card kind may select `target`.
pub fn card_target_cancelled_for_kind(
    state: &GameState,
    user: PlayerId,
    target: PlayerId,
    logical_kind: CardKind,
) -> bool {
    if logical_kind != CardKind::Tao {
        return false;
    }
    let mut mods = state
        .modifiers
        .for_target(ModifierTarget::CardTargetCancelFor(CardSelector::Tao));
    if user == state.turn.current {
        mods.extend(
            state
                .modifiers
                .for_target(ModifierTarget::CardTargetCancelFromCurrentFor(
                    CardSelector::Tao,
                )),
        );
    }
    if user != target && state.turn.use_ban_pairs.contains(&(user, target)) {
        mods.extend(
            state
                .modifiers
                .for_target(ModifierTarget::CardTargetCancelTurnBannedFor(
                    CardSelector::Tao,
                ))
                .into_iter()
                .chain(
                    state
                        .modifiers
                        .for_target(ModifierTarget::CardTargetCancelTurnBannedFor(
                            CardSelector::Any,
                        )),
                ),
        );
    }
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(target));
    vetoed || value != 0
}
