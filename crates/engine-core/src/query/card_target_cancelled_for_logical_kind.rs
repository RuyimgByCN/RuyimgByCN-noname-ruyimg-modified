//! Query operations for `card_target_cancelled_for_logical_kind` and related helpers.

use super::*;

/// Whether one concrete/logical card may select `target`.
pub(crate) fn card_target_cancelled_for_logical_kind(
    state: &GameState,
    user: PlayerId,
    target: PlayerId,
    card: &Card,
    logical_kind: CardKind,
) -> bool {
    let mut mods = state.modifiers.for_card_target(
        user,
        target,
        logical_kind,
        effective_card_suit(state, user, card),
        user == state.turn.current,
        &state.turn.use_ban_pairs,
    );
    let card_suit = effective_card_suit(state, user, card);
    if user != target && !armor_effect_ignored_for(state, user, target) {
        mods.extend(
            state
                .modifiers
                .for_armor_card_target(logical_kind, card_suit),
        );
    }
    if !state.hand_cards(target).into_iter().any(|hand_card| {
        state
            .cards
            .get(hand_card)
            .is_some_and(|held| effective_card_suit(state, target, held) == card_suit)
    }) {
        mods.extend(
            state
                .modifiers
                .for_target(ModifierTarget::CardTargetCancelIfTargetLacksMatchingSuit),
        );
    }
    if user != target
        && state.hand_count(user) >= state.hand_count(target)
        && !has_equip_kind(state, target, CardKind::Armor)
    {
        mods.extend(
            state
                .modifiers
                .for_card_target_if_other_user_hand_gte_target_and_armor_empty(
                    logical_kind,
                    card_suit,
                ),
        );
    }
    if user != target
        && state
            .players
            .get(user)
            .zip(state.players.get(target))
            .is_some_and(|(user, target)| user.hp >= target.hp)
    {
        mods.extend(
            state
                .modifiers
                .for_target(ModifierTarget::CardTargetCancelIfOtherUserHpGteTarget),
        );
    }
    let mark_cancel_mods = if user != target {
        state
            .modifiers
            .for_target(ModifierTarget::CardTargetCancelIfOtherUserCardUseCountLtOwnerMark)
    } else {
        Vec::new()
    };
    if user != target {
        let distance_mods = state
            .modifiers
            .for_card_target_if_other_user_distance_gt_one(logical_kind, card_suit);
        if !distance_mods.is_empty()
            && matches!(
                distance_explain(state, user, target).response,
                QueryResponse::Int(distance) if distance > 1
            )
        {
            mods.extend(distance_mods);
        }
    }
    let conditional_veto = mark_cancel_mods.iter().any(|modifier| {
        modifier.target
            == ModifierTarget::CardTargetCancelIfOtherUserCardUseCountLtOwnerMark
            && state.turn.turn_card_use_count(user)
                < state
                    .players
                    .get(target)
                    .and_then(|player| {
                        player
                            .marks
                            .iter()
                            .find(|(mark, _)| *mark == modifier.skill_def.raw())
                            .map(|(_, count)| (*count).max(0) as u32)
                    })
                    .unwrap_or(0)
    });
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(target));
    vetoed || value != 0 || conditional_veto
}

/// Whether source-side armor bypass or target-side armor invalidation suppresses armor effects.
pub(crate) fn armor_effect_ignored_for(
    state: &GameState,
    source: PlayerId,
    target: PlayerId,
) -> bool {
    let source_ignores = if source.is_some() {
        let mut mods = state.modifiers.for_target(ModifierTarget::IgnoreArmor);
        mods.extend(
            state
                .modifiers
                .for_target(ModifierTarget::IgnoreArmorForSkillBoundTarget)
                .into_iter()
                .filter(|modifier| {
                    skill_bound_target_modifier_applies(state, modifier, source, target)
                }),
        );
        let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(source));
        vetoed || value != 0
    } else {
        false
    };
    let mods = state
        .modifiers
        .for_target(ModifierTarget::ArmorEffectDisabled);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(target));
    source_ignores || vetoed || value != 0
}

/// Whether armor is suppressed for one concrete card's source-side effects.
pub(crate) fn armor_effect_ignored_for_card(
    state: &GameState,
    source: PlayerId,
    target: PlayerId,
    card: engine_api::CardId,
) -> bool {
    if armor_effect_ignored_for(state, source, target) {
        return true;
    }
    if source.is_none() || card.is_none() {
        return false;
    }
    let mods: Vec<_> = state
        .modifiers
        .for_target(ModifierTarget::IgnoreArmorForSkillCardPolicy)
        .into_iter()
        .filter(|modifier| {
            skill_card_policy_modifier_applies(
                state,
                modifier,
                source,
                card,
                crate::state::SkillInstance::CARD_USE_POLICY_IGNORE_ARMOR,
            )
        })
        .collect();
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(source));
    vetoed || value != 0
}

/// Whether an unlinked player is protected by active armor from gaining the chain mark.
pub fn armor_link_gain_cancelled_for(state: &GameState, player: PlayerId) -> bool {
    if armor_effect_ignored_for(state, PlayerId::NONE, player) {
        return false;
    }
    let mods = state
        .modifiers
        .for_target(ModifierTarget::ArmorLinkGainCancel);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether a linked player is prevented from losing the chain mark.
pub fn link_loss_cancelled_for(state: &GameState, player: PlayerId) -> bool {
    if !state
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return false;
    }
    let mods = state.modifiers.for_target(ModifierTarget::LinkLossCancel);
    let (value, _, vetoed) = apply_int_modifiers_for_state(state, 0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether 杀 cards are excluded from this player's hand-limit accounting.
pub fn sha_ignored_for_hand_limit(state: &GameState, player: PlayerId) -> bool {
    let mut mods = state
        .modifiers
        .for_target(ModifierTarget::ShaIgnoredHandLimit);
    mods.extend(
        state
            .modifiers
            .for_target(ModifierTarget::CardIgnoredHandLimit(CardSelector::Sha)),
    );
    let (value, _, vetoed) = apply_int_modifiers(0, &mods, Some(player));
    vetoed || value != 0
}

/// Whether one concrete card is excluded from this player's hand-limit accounting.
pub fn card_ignored_for_hand_limit(
    state: &GameState,
    player: PlayerId,
    card: engine_api::CardId,
) -> bool {
    if state.turn.hand_limit_exempt_cards.contains(&(player, card)) {
        return true;
    }
    let Some(card) = state.cards.get(card) else {
        return false;
    };
    let kind = effective_card_kind(state, player, card);
    if kind == CardKind::TrickMantianguohai {
        return true;
    }
    let mods = state.modifiers.for_hand_limit_card(
        state,
        player,
        card,
        kind,
        effective_card_suit(state, player, card),
    );
    let (value, _, vetoed) = apply_int_modifiers(0, &mods, Some(player));
    vetoed || value != 0
}

/// Hand cards eligible for the ordinary discard-to-limit choice.
pub fn discard_to_limit_tokens(state: &GameState, player: PlayerId) -> Vec<engine_api::CardId> {
    let mut cards = state.hand_cards(player);
    if discard_phase_includes_equip(state, player) {
        if let Some(equip) = state
            .zones
            .find(ZoneKind::Equip, player)
            .and_then(|zone| state.zones.get(zone))
        {
            cards.extend(equip.cards.iter().copied());
        }
    }
    cards
        .into_iter()
        .filter(|card| !card_ignored_for_hand_limit(state, player, *card))
        .collect()
}

/// Effective hand size used by the discard-to-limit rule.
pub fn hand_count_for_limit(state: &GameState, player: PlayerId) -> u32 {
    discard_to_limit_tokens(state, player).len() as u32
}

/// Suit seen by rules for one player; the physical card remains unchanged.
pub fn effective_card_suit(state: &GameState, player: PlayerId, card: &Card) -> engine_api::Suit {
    if player.is_none() {
        return card.suit;
    }
    let mods = state
        .modifiers
        .for_target(ModifierTarget::CardSuit(card.suit));
    let base = match card.suit {
        engine_api::Suit::None => 0,
        engine_api::Suit::Spade => 1,
        engine_api::Suit::Heart => 2,
        engine_api::Suit::Club => 3,
        engine_api::Suit::Diamond => 4,
    };
    let (value, _, vetoed) = apply_int_modifiers(base, &mods, Some(player));
    if vetoed {
        return card.suit;
    }
    match value {
        0 => engine_api::Suit::None,
        1 => engine_api::Suit::Spade,
        2 => engine_api::Suit::Heart,
        3 => engine_api::Suit::Club,
        4 => engine_api::Suit::Diamond,
        _ => card.suit,
    }
}

/// Available physical cards grouped by their non-virtual effective suit.
pub fn effective_suit_card_groups(
    state: &GameState,
    player: PlayerId,
    cards: &[engine_api::CardId],
) -> [Vec<engine_api::CardId>; 4] {
    let mut groups = std::array::from_fn(|_| Vec::new());
    for card_id in cards {
        let Some(card) = state.cards.get(*card_id) else {
            continue;
        };
        let index = match effective_card_suit(state, player, card) {
            engine_api::Suit::Spade => 0,
            engine_api::Suit::Heart => 1,
            engine_api::Suit::Club => 2,
            engine_api::Suit::Diamond => 3,
            engine_api::Suit::None => continue,
        };
        groups[index].push(*card_id);
    }
    groups
}

/// Number seen by rules for one player's card; the physical rank remains unchanged.
pub fn effective_card_number(state: &GameState, player: PlayerId, card: &Card) -> u8 {
    if player.is_none() {
        return card.rank;
    }
    let tagged_in_hand = state.card_has_content_tag(card.id, crate::state::SHEQUAN_CARD_TAG)
        && state.zones.get(card.zone).is_some_and(|zone| {
            zone.kind == ZoneKind::Hand && zone.owner == player && card.owner == player
        });
    if tagged_in_hand {
        let has_jianzhuan = state
            .players
            .get(player)
            .into_iter()
            .flat_map(|owner| owner.skills.iter())
            .filter_map(|skill| state.skills.get(*skill))
            .any(|skill| {
                skill.def.raw() == crate::state::POTJIANZHUAN_DEF
                    && state.skill_effectively_enabled(skill)
            });
        return if has_jianzhuan { 13 } else { 1 };
    }
    let mods = state.modifiers.for_target(ModifierTarget::CardNumber);
    let (value, _, vetoed) =
        apply_int_modifiers_for_state(state, i32::from(card.rank), &mods, Some(player));
    if vetoed {
        return card.rank;
    }
    u8::try_from(value)
        .ok()
        .filter(|number| (1..=13).contains(number))
        .unwrap_or(card.rank)
}

/// Physical number used by converted-card rules: sum every material for a multi-card use,
/// otherwise use the carrier's printed rank.
pub fn physical_card_number_sum(
    state: &GameState,
    carrier: engine_api::CardId,
    sources: &[engine_api::CardId],
) -> Option<u32> {
    if sources.len() > 1 {
        return sources.iter().try_fold(0u32, |sum, card| {
            let rank = state.cards.get(*card)?.rank;
            (rank > 0).then(|| sum.saturating_add(u32::from(rank)))
        });
    }
    let rank = state.cards.get(carrier)?.rank;
    (rank > 0).then_some(u32::from(rank))
}

/// Rule-visible length of the card's localized name, matching upstream `get.cardNameLength`.
pub fn card_name_length(
    state: &GameState,
    player: PlayerId,
    card: engine_api::CardId,
) -> Option<u8> {
    let card = state.cards.get(card)?;
    let kind = effective_card_kind(state, player, card);
    let fixed = match kind {
        CardKind::Ying | CardKind::TrickChuqibuyi => 0,
        CardKind::Sha | CardKind::ShaFire | CardKind::ShaThunder => 1,
        CardKind::Tao | CardKind::Shan | CardKind::Jiu => 1,
        CardKind::TrickJuedou | CardKind::TrickShandian | CardKind::TrickHuogong => 2,
        CardKind::TrickWuzhong
        | CardKind::TrickGuohe
        | CardKind::TrickShunshou
        | CardKind::TrickNanman
        | CardKind::TrickWanjian
        | CardKind::TrickTaoyuan
        | CardKind::TrickWugu
        | CardKind::TrickLe
        | CardKind::TrickBingliang
        | CardKind::TrickJiedao
        | CardKind::TrickTiesuo
        | CardKind::TrickWuxie
        | CardKind::TrickYuanjiao
        | CardKind::TrickTiaojiyanmei
        | CardKind::TrickMantianguohai
        | CardKind::TrickYoudishenru => 4,
        CardKind::TrickDiaobingqianjiang | CardKind::TrickWangmeizhike => 4,
        CardKind::TrickYiyi => 4,
        CardKind::TrickShuiyanqijunx => 4,
        CardKind::TrickLulitongxin => 4,
        CardKind::TrickKefuzhongyuan => 4,
        CardKind::TrickGuguoanbang => 4,
        CardKind::TrickHuoshaolianying => 4,
        CardKind::TrickLianjunshengyan | CardKind::TrickWenheluanwu => 4,
        CardKind::TrickHaolingtianxia => 4,
        CardKind::TrickChiling => 4,
        CardKind::Weapon | CardKind::Armor | CardKind::HorseOff | CardKind::HorseDef => {
            match card.def.raw() {
                100 => 5,                   // 雌雄双股剑
                101 => 4,                   // 诸葛连弩
                102 | 105 | 110 => 3,       // 青釭剑、麒麟弓、八卦阵
                103 | 104 | 122 => 4,       // 丈八蛇矛、方天画戟、爪黄飞电
                111 | 120 | 121 | 123 => 2, // 藤甲与标准坐骑
                _ => 0,
            }
        }
        CardKind::Treasure | CardKind::Other => 0,
    };
    (fixed > 0).then_some(fixed).or_else(|| {
        state
            .content
            .card_metadata(card.def)
            .map(|card| card.name_length)
            .filter(|n| *n > 0)
    })
}

/// Whether a physical card carries the upstream `gifts` tag.
pub fn card_is_gift(state: &GameState, card: engine_api::CardId) -> bool {
    state
        .cards
        .get(card)
        .and_then(|card| state.content.card_metadata(card.def))
        .is_some_and(|metadata| metadata.gift)
}

/// Enumerate bounded, stable card subsets whose effective numbers total exactly `sum`.
pub fn effective_number_sum_subsets(
    state: &GameState,
    player: PlayerId,
    cards: &[engine_api::CardId],
    min_count: usize,
    max_count: usize,
    sum: u16,
    limit: usize,
) -> Vec<Vec<u32>> {
    fn visit(
        entries: &[(u32, u16)],
        index: usize,
        min_count: usize,
        max_count: usize,
        remaining: u16,
        limit: usize,
        current: &mut Vec<u32>,
        out: &mut Vec<Vec<u32>>,
        dead: &mut HashSet<(usize, usize, u16)>,
    ) {
        if out.len() >= limit {
            return;
        }
        let state = (index, current.len(), remaining);
        if dead.contains(&state) {
            return;
        }
        if remaining == 0 {
            if current.len() >= min_count {
                out.push(current.clone());
            }
            return;
        }
        if current.len() == max_count {
            dead.insert(state);
            return;
        }
        let before = out.len();
        for next in index..entries.len() {
            let (card, number) = entries[next];
            if number > remaining {
                continue;
            }
            current.push(card);
            visit(
                entries,
                next + 1,
                min_count,
                max_count,
                remaining - number,
                limit,
                current,
                out,
                dead,
            );
            current.pop();
            if out.len() >= limit {
                return;
            }
        }
        if out.len() == before {
            dead.insert(state);
        }
    }

    if limit == 0 || min_count > max_count || sum == 0 {
        return Vec::new();
    }
    let entries: Vec<_> = cards
        .iter()
        .filter_map(|card| {
            state.cards.get(*card).map(|state_card| {
                (
                    card.raw(),
                    u16::from(effective_card_number(state, player, state_card)),
                )
            })
        })
        .collect();
    let mut out = Vec::new();
    visit(
        &entries,
        0,
        min_count,
        max_count.min(entries.len()),
        sum,
        limit,
        &mut Vec::new(),
        &mut out,
        &mut HashSet::new(),
    );
    out
}

/// Rule category seen by one player; the physical card remains unchanged.
pub fn effective_card_kind(state: &GameState, player: PlayerId, card: &Card) -> CardKind {
    if player.is_none() {
        return card.kind;
    }
    let mods = state.modifiers.for_card_name(
        state,
        player,
        card,
        effective_card_suit(state, player, card),
    );
    let (tracked, owned): (Vec<_>, Vec<_>) = mods.into_iter().partition(|modifier| {
        matches!(
            modifier.target,
            ModifierTarget::CardName(CardSelector::SkillTracked)
        )
    });
    let (value, _, tracked_vetoed) = apply_int_modifiers(card.kind.tag() as i32, &tracked, None);
    let (value, _, owned_vetoed) = apply_int_modifiers(value, &owned, Some(player));
    let vetoed = tracked_vetoed || owned_vetoed;
    if vetoed {
        return card.kind;
    }
    u8::try_from(value)
        .ok()
        .and_then(CardKind::from_tag)
        .unwrap_or(card.kind)
}

/// Damage nature seen for one card used by a player; the physical card remains unchanged.
pub fn effective_card_nature(
    state: &GameState,
    player: PlayerId,
    card: &Card,
    logical_kind: CardKind,
) -> u8 {
    if player.is_none() {
        return logical_kind.sha_nature();
    }
    let mods = state
        .modifiers
        .for_card_nature(logical_kind, effective_card_suit(state, player, card));
    let base = i32::from(logical_kind.sha_nature());
    let (value, _, vetoed) = apply_int_modifiers(base, &mods, Some(player));
    if vetoed {
        return logical_kind.sha_nature();
    }
    u8::try_from(value).unwrap_or_else(|_| logical_kind.sha_nature())
}

/// Check a fixed card target-distance limit through the shared modifier surface.
pub fn card_target_within_distance(
    state: &GameState,
    user: PlayerId,
    card: engine_api::CardId,
    target: PlayerId,
    limit: i32,
) -> bool {
    if card_distance_unlimited_for_target(state, user, card, target) {
        return true;
    }
    let Some(card_state) = state.cards.get(card) else {
        return false;
    };
    let kind = effective_card_kind(state, user, card_state);
    let limit_mods = state
        .modifiers
        .for_card_distance_limit(kind, effective_card_suit(state, user, card_state));
    let (limit, _, vetoed) = apply_int_modifiers(limit, &limit_mods, Some(user));
    if vetoed {
        return false;
    }
    matches!(
        query(
            state,
            &QueryRequest::Distance {
                from: user,
                to: target,
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(distance) if distance <= limit.max(0)
    )
}

/// Whether the player has a content-defined draw-replacement damage offer.
pub fn has_draw_replacement(state: &GameState, player: PlayerId) -> bool {
    let mods = state
        .modifiers
        .for_target(ModifierTarget::DrawReplacementDamage);
    let (value, _, vetoed) = apply_int_modifiers(0, &mods, Some(player));
    !vetoed && value > 0
}

/// Content-defined prepare-phase scry count, capped by alive players.
pub fn prepare_scry_count(state: &GameState, player: PlayerId) -> u32 {
    let mods = state.modifiers.for_target(ModifierTarget::PrepareScryCount);
    let (value, _, vetoed) = apply_int_modifiers(0, &mods, Some(player));
    if vetoed {
        0
    } else {
        (value.max(0) as u32).min(state.players.alive_by_seat().len() as u32)
    }
}

/// Whether package content permits this player to rewrite a judgment.
pub fn can_rewrite_judgment(state: &GameState, player: PlayerId) -> bool {
    judgment_rewrite_unrestricted(state, player)
        || judgment_rewrite_red_hand_equip_expansion(state, player)
        || state
            .modifiers
            .for_target(ModifierTarget::JudgmentRewriteBlackHandEquip)
            .iter()
            .any(|modifier| {
                modifier.owner == player
                    && !crate::query::judgment_black_rewrite_cards(state, player).is_empty()
            })
}

/// Black hand/equipment cards eligible for a 鬼道 rewrite.
pub(crate) fn judgment_black_rewrite_cards(state: &GameState, player: PlayerId) -> Vec<CardId> {
    let in_zone = |card: &Card| {
        let in_hand = state.zones.find(ZoneKind::Hand, player) == Some(card.zone);
        let in_equip = state.zones.find(ZoneKind::Equip, player) == Some(card.zone);
        (in_hand || in_equip)
            && matches!(
                effective_card_suit(state, player, card),
                Suit::Spade | Suit::Club
            )
    };
    state
        .cards
        .sorted_live()
        .into_iter()
        .filter(|card| card.owner == player && in_zone(card))
        .map(|card| card.id)
        .collect()
}

pub(crate) fn judgment_rewrite_unrestricted(state: &GameState, player: PlayerId) -> bool {
    let hand_mods = state.modifiers.for_target(ModifierTarget::JudgmentRewrite);
    let hand_equip_mods = state
        .modifiers
        .for_target(ModifierTarget::JudgmentRewriteHandEquip);
    let (hand_value, _, hand_vetoed) = apply_int_modifiers(0, &hand_mods, Some(player));
    let (hand_equip_value, _, hand_equip_vetoed) =
        apply_int_modifiers(0, &hand_equip_mods, Some(player));
    let paid_hand_equip = judgment_rewrite_mark_cost(state, player).is_some_and(|mark| {
        state
            .players
            .get(player)
            .into_iter()
            .flat_map(|player| player.marks.iter())
            .any(|(definition, count)| *definition == mark && *count > 0)
    });
    (!hand_vetoed && hand_value > 0)
        || (!hand_equip_vetoed && hand_equip_value > 0)
        || paid_hand_equip
}

pub(crate) fn judgment_rewrite_red_hand_equip_expansion(
    state: &GameState,
    player: PlayerId,
) -> bool {
    let mods = state
        .modifiers
        .for_target(ModifierTarget::JudgmentRewriteRedHandEquipExpansion);
    let (value, _, vetoed) = apply_int_modifiers(0, &mods, Some(player));
    !vetoed && value > 0
}

/// Whether judgment rewrite may also consume one of the player's equipment cards.
pub fn judgment_rewrite_includes_equip(state: &GameState, player: PlayerId) -> bool {
    let mods = state
        .modifiers
        .for_target(ModifierTarget::JudgmentRewriteHandEquip);
    let (value, _, vetoed) = apply_int_modifiers(0, &mods, Some(player));
    (!vetoed && value > 0)
        || judgment_rewrite_mark_cost(state, player).is_some()
        || judgment_rewrite_red_hand_equip_expansion(state, player)
}

/// Whether one owned personal-zone card may currently replace a judgment.
pub(crate) fn judgment_rewrite_card_matches(
    state: &GameState,
    player: PlayerId,
    card: &Card,
) -> bool {
    if card.owner != player {
        return false;
    }
    let in_hand = state.zones.find(ZoneKind::Hand, player) == Some(card.zone);
    let in_equip = state.zones.find(ZoneKind::Equip, player) == Some(card.zone);
    let in_expansion = state.zones.find(ZoneKind::Expansion, player) == Some(card.zone);
    let hand_mods = state.modifiers.for_target(ModifierTarget::JudgmentRewrite);
    let hand_equip_mods = state
        .modifiers
        .for_target(ModifierTarget::JudgmentRewriteHandEquip);
    let (hand, _, hand_vetoed) = apply_int_modifiers(0, &hand_mods, Some(player));
    let (hand_equip, _, hand_equip_vetoed) = apply_int_modifiers(0, &hand_equip_mods, Some(player));
    let black_hand_equip_mods = state
        .modifiers
        .for_target(ModifierTarget::JudgmentRewriteBlackHandEquip);
    let (black_hand_equip, _, black_hand_equip_vetoed) =
        apply_int_modifiers(0, &black_hand_equip_mods, Some(player));
    let free = (in_hand && !hand_vetoed && hand > 0)
        || ((in_hand || in_equip) && !hand_equip_vetoed && hand_equip > 0)
        || ((in_hand || in_equip)
            && !black_hand_equip_vetoed
            && black_hand_equip > 0
            && matches!(
                effective_card_suit(state, player, card),
                Suit::Spade | Suit::Club
            ))
        || ((in_hand || in_equip || in_expansion)
            && judgment_rewrite_red_hand_equip_expansion(state, player)
            && matches!(
                effective_card_suit(state, player, card),
                Suit::Heart | Suit::Diamond
            ));
    let paid = (in_hand || in_equip)
        && judgment_rewrite_mark_cost(state, player).is_some_and(|mark| {
            state
                .players
                .get(player)
                .into_iter()
                .flat_map(|player| player.marks.iter())
                .any(|(definition, count)| *definition == mark && *count > 0)
        });
    free || paid
}

/// Mark consumed by this exact rewrite card, if it is legal only through a paid channel.
pub(crate) fn judgment_rewrite_mark_cost_for_card(
    state: &GameState,
    player: PlayerId,
    card: &Card,
) -> Option<u32> {
    let in_hand = state.zones.find(ZoneKind::Hand, player) == Some(card.zone);
    let in_equip = state.zones.find(ZoneKind::Equip, player) == Some(card.zone);
    let red_free = judgment_rewrite_red_hand_equip_expansion(state, player)
        && matches!(
            effective_card_suit(state, player, card),
            Suit::Heart | Suit::Diamond
        );
    let hand_free = in_hand && {
        let mods = state.modifiers.for_target(ModifierTarget::JudgmentRewrite);
        let (value, _, vetoed) = apply_int_modifiers(0, &mods, Some(player));
        !vetoed && value > 0
    };
    let hand_equip_free = (in_hand || in_equip) && {
        let mods = state
            .modifiers
            .for_target(ModifierTarget::JudgmentRewriteHandEquip);
        let (value, _, vetoed) = apply_int_modifiers(0, &mods, Some(player));
        !vetoed && value > 0
    };
    if hand_free || hand_equip_free || red_free {
        None
    } else {
        judgment_rewrite_mark_cost(state, player)
    }
}

/// Mark consumed when this player successfully rewrites a judgment, if any.
pub fn judgment_rewrite_mark_cost(state: &GameState, player: PlayerId) -> Option<u32> {
    for target in [
        ModifierTarget::JudgmentRewrite,
        ModifierTarget::JudgmentRewriteHandEquip,
    ] {
        let mods = state.modifiers.for_target(target);
        let (value, _, vetoed) = apply_int_modifiers(0, &mods, Some(player));
        if !vetoed && value > 0 {
            return None;
        }
    }
    state.modifiers.judgment_rewrite_cost_mark(player)
}

/// Whether the player's judgment should use the most favorable card in the draw pile.
pub fn judgment_uses_best_draw_pile_card(state: &GameState, player: PlayerId) -> bool {
    let mods = state
        .modifiers
        .for_target(ModifierTarget::JudgmentBestFromDrawPile);
    let (value, _, vetoed) = apply_int_modifiers(0, &mods, Some(player));
    !vetoed && value > 0
}

/// Required response-card count after source-owned content modifiers.
pub fn response_count_for(state: &GameState, source: PlayerId, base_count: u32) -> u32 {
    let mods = state.modifiers.for_target(ModifierTarget::ResponseCount);
    let (value, _, vetoed) =
        apply_int_modifiers_for_state(state, base_count as i32, &mods, Some(source));
    if vetoed {
        0
    } else {
        value.max(0) as u32
    }
}

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
