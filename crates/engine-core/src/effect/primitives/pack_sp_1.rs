//! Effect primitive implementations — sp 1 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

pub(crate) fn choose_move_field_card_to(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let options = crate::query::movable_field_cards_to(tx.state(), target);
    if options.is_empty() || !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseMoveFieldCardTo,
        vec![player.raw(), target.raw(), skill.raw()],
    )
}

pub(crate) fn draw_for_lifetime_card_action_divisors(
    tx: &mut Transaction,
    player: PlayerId,
    divisors: [u32; 3],
) -> Result<Vec<DomainEvent>, EffectError> {
    if tx.state().players.get(player).is_none() {
        return Err(EffectError::Illegal("bad player".into()));
    }
    if divisors.contains(&0) {
        return Err(EffectError::Illegal(
            "card-action divisor must be positive".into(),
        ));
    }
    let total = tx
        .state()
        .card_action_totals
        .get(&player)
        .copied()
        .unwrap_or(0);
    if total == 0 {
        return Ok(Vec::new());
    }
    let count = divisors
        .into_iter()
        .filter(|divisor| total % divisor == 0)
        .count() as u32;
    if count == 0 {
        Ok(Vec::new())
    } else {
        draw_cards(tx, player, count)
    }
}

pub(crate) fn gain_draw_cards_for_lifetime_card_action_divisors(
    tx: &mut Transaction,
    player: PlayerId,
    divisors: [u32; 3],
    groups: [u8; 3],
) -> Result<Vec<DomainEvent>, EffectError> {
    if tx.state().players.get(player).is_none() {
        return Err(EffectError::Illegal("bad player".into()));
    }
    if divisors.contains(&0)
        || groups.iter().any(|group| {
            !matches!(
                *group,
                rule_api::lifetime_card_reward_group::SHA_OR_SHAN
                    | rule_api::lifetime_card_reward_group::TAO_OR_JIU
                    | rule_api::lifetime_card_reward_group::JUE_DOU_OR_WU_ZHONG
            )
        })
    {
        return Err(EffectError::Illegal(
            "invalid lifetime card reward selector".into(),
        ));
    }
    let total = tx
        .state()
        .card_action_totals
        .get(&player)
        .copied()
        .unwrap_or(0);
    if total == 0 {
        return Ok(Vec::new());
    }
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("draw pile missing".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let pile = tx
        .state()
        .zones
        .get(draw)
        .map(|zone| zone.cards.clone())
        .unwrap_or_default();
    let mut cards = Vec::new();
    for (divisor, group) in divisors.into_iter().zip(groups) {
        if total % divisor != 0 {
            continue;
        }
        if let Some(card) = pile.iter().rev().copied().find(|card| {
            !cards.contains(card)
                && tx.state().cards.get(*card).is_some_and(|card| {
                    matches!(
                        (group, card.kind),
                        (
                            rule_api::lifetime_card_reward_group::SHA_OR_SHAN,
                            CardKind::Sha
                                | CardKind::ShaFire
                                | CardKind::ShaThunder
                                | CardKind::Shan
                        ) | (
                            rule_api::lifetime_card_reward_group::TAO_OR_JIU,
                            CardKind::Tao | CardKind::Jiu
                        ) | (
                            rule_api::lifetime_card_reward_group::JUE_DOU_OR_WU_ZHONG,
                            CardKind::TrickJuedou | CardKind::TrickWuzhong
                        )
                    )
                })
        }) {
            cards.push(card);
        }
    }
    if cards.is_empty() {
        Ok(Vec::new())
    } else {
        move_cards(tx, cards, hand, Some(draw))
    }
}

pub(crate) fn choose_add_missing_skill(
    tx: &mut Transaction,
    player: PlayerId,
    skill_defs: [engine_api::SkillDefId; 3],
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if skill_defs.iter().any(|skill_def| skill_def.raw() == 0)
        || skill_defs[0] == skill_defs[1]
        || skill_defs[0] == skill_defs[2]
        || skill_defs[1] == skill_defs[2]
    {
        return Err(EffectError::ContentFault(
            "missing-skill choice requires three distinct non-zero definitions".into(),
        ));
    }
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let options: Vec<_> = skill_defs
        .into_iter()
        .filter(|skill_def| {
            !tx.state()
                .players
                .get(player)
                .into_iter()
                .flat_map(|participant| participant.skills.iter())
                .any(|skill| {
                    tx.state()
                        .skills
                        .get(*skill)
                        .is_some_and(|instance| instance.def == *skill_def)
                })
        })
        .collect();
    match options.as_slice() {
        [] => Ok(Vec::new()),
        [skill_def] => add_skill(tx, player, *skill_def),
        _ => open_choice(
            tx,
            player,
            1,
            1,
            false,
            options.iter().map(|skill_def| skill_def.raw()).collect(),
            ChoiceKind::Custom,
            ContinuationKind::ChooseAddMissingSkill,
            vec![
                player.raw(),
                skill.raw(),
                skill_defs[0].raw(),
                skill_defs[1].raw(),
                skill_defs[2].raw(),
            ],
        ),
    }
}

pub(crate) fn add_first_missing_skill(
    tx: &mut Transaction,
    player: PlayerId,
    skill_defs: [engine_api::SkillDefId; 3],
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if skill_defs.iter().any(|skill_def| skill_def.raw() == 0)
        || skill_defs[0] == skill_defs[1]
        || skill_defs[0] == skill_defs[2]
        || skill_defs[1] == skill_defs[2]
    {
        return Err(EffectError::ContentFault(
            "first-missing-skill grant requires three distinct non-zero definitions".into(),
        ));
    }
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let Some(skill_def) = skill_defs.into_iter().find(|skill_def| {
        !tx.state()
            .players
            .get(player)
            .into_iter()
            .flat_map(|participant| participant.skills.iter())
            .any(|skill| {
                tx.state()
                    .skills
                    .get(*skill)
                    .is_some_and(|instance| instance.def == *skill_def)
            })
    }) else {
        return Ok(Vec::new());
    };
    add_skill(tx, player, skill_def)
}

pub(crate) fn record_current_sha_all_targets(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if card.is_none()
        || !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let targets = tx
        .state()
        .resolution
        .sha_resolution_records
        .iter()
        .find(|record| record.source == player && record.card == card)
        .map(|record| record.targets.clone())
        .unwrap_or_default();
    for target in targets {
        tx.state_mut()
            .resolution
            .record_sha_skill_target(player, card, skill, target);
    }
    Ok(Vec::new())
}

pub(crate) fn resolve_dccunsi(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    guixiu_def: engine_api::SkillDefId,
    yongjue_def: engine_api::SkillDefId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if guixiu_def.raw() == 0 || yongjue_def.raw() == 0 || guixiu_def == yongjue_def {
        return Err(EffectError::ContentFault(
            "dccunsi requires distinct non-zero Guixiu and Yongjue definitions".into(),
        ));
    }
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx
                    .state()
                    .players
                    .get(player)
                    .is_some_and(|participant| participant.alive)
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let mut events = add_skill(tx, target, yongjue_def)?;
            if target != player {
                events.extend(draw_cards(tx, player, 2)?);
            }
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveDccunsi {
                    player,
                    target,
                    guixiu_def,
                    yongjue_def,
                    stage: 1,
                    skill,
                });
            Ok(events)
        }
        1 => {
            let wounded = tx.state().players.get(player).is_some_and(|participant| {
                participant.alive && participant.hp < participant.max_hp as i32
            });
            if !wounded || !player_has_skill_def(tx, player, guixiu_def, true) {
                return Ok(Vec::new());
            }
            recover_hp(tx, player, player, 1)
        }
        _ => Err(EffectError::ContentFault("invalid dccunsi stage".into())),
    }
}

pub(crate) fn remove_cards_by_kind(tx: &mut Transaction, kind: u8) -> Result<Vec<DomainEvent>, EffectError> {
    let kind = crate::state::CardKind::from_tag(kind)
        .ok_or_else(|| EffectError::Illegal(format!("unknown card kind {kind}")))?;
    let removed = tx
        .state()
        .zones
        .find(ZoneKind::Removed, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no removed zone".into()))?;
    let groups: Vec<_> = tx
        .state()
        .zones
        .sorted_live()
        .into_iter()
        .filter(|zone| !matches!(zone.kind, ZoneKind::Processing | ZoneKind::Removed))
        .filter_map(|zone| {
            let cards: Vec<_> = zone
                .cards
                .iter()
                .copied()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|card| card.kind == kind)
                })
                .collect();
            (!cards.is_empty()).then_some((zone.id, cards))
        })
        .collect();

    let mut events = Vec::with_capacity(groups.len());
    for (from, cards) in groups {
        for card in &cards {
            move_one_card(tx, *card, from, removed, PlayerId::NONE, true)?;
        }
        events.push(DomainEvent::CardsMoved {
            cards,
            from,
            to: removed,
            reason: CardMoveReason::Unspecified,
        });
    }
    Ok(events)
}

pub(crate) fn discard_cards(
    tx: &mut Transaction,
    player: PlayerId,
    cards: Vec<CardId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    if cards.is_empty() {
        return Err(EffectError::Illegal("discard selection is empty".into()));
    }
    let allowed: std::collections::BTreeSet<_> =
        crate::query::discardable_hand_equip_cards(tx.state(), player)
            .into_iter()
            .map(CardId::raw)
            .collect();
    let mut seen = std::collections::BTreeSet::new();
    let mut by_zone = std::collections::BTreeMap::<u32, Vec<CardId>>::new();
    for card in &cards {
        if !seen.insert(card.raw()) {
            return Err(EffectError::Illegal("duplicate discard card".into()));
        }
        let card_state = tx
            .state()
            .cards
            .get(*card)
            .ok_or_else(|| EffectError::Illegal("bad discard card".into()))?;
        if card_state.owner != player || !allowed.contains(&card.raw()) {
            return Err(EffectError::Illegal(
                "discard card not in player hand or equipment".into(),
            ));
        }
        by_zone
            .entry(card_state.zone.raw())
            .or_default()
            .push(*card);
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let mut events = Vec::new();
    for (from, cards) in by_zone {
        events.extend(move_cards_with_reason(
            tx,
            cards,
            discard,
            Some(ZoneId::new(from)),
            CardMoveReason::Discard,
        )?);
    }
    Ok(events)
}

pub(crate) fn reveal_hand_discard_kind(
    tx: &mut Transaction,
    player: PlayerId,
    kind: u8,
) -> Result<Vec<DomainEvent>, EffectError> {
    let kind = CardKind::from_tag(kind)
        .ok_or_else(|| EffectError::Illegal(format!("unknown card kind {kind}")))?;
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let cards = tx.state().hand_cards(player);
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let discardable: std::collections::BTreeSet<_> =
        crate::query::discardable_hand_equip_cards(tx.state(), player)
            .into_iter()
            .collect();
    let matching: Vec<_> = cards
        .iter()
        .copied()
        .filter(|card| {
            discardable.contains(card)
                && tx.state().cards.get(*card).is_some_and(|card| {
                    crate::query::effective_card_kind(tx.state(), player, card) == kind
                })
        })
        .collect();
    let mut effects = vec![EffectRequest::RevealCards {
        cards,
        to: Vec::new(),
    }];
    if !matching.is_empty() {
        effects.push(EffectRequest::DiscardCards {
            player,
            cards: matching,
        });
    }
    crate::effect::run_effects(tx, effects)
}

pub(crate) fn reveal_hand_choose_discard_color(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let cards = tx.state().hand_cards(player);
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let color_mask = cards.iter().fold(0, |mask, card| {
        mask | crate::query::effective_card_color_mask(tx.state(), player, *card)
    }) & 0b11;
    let mut events = vec![DomainEvent::CardsRevealed {
        cards,
        to: Vec::new(),
    }];
    if matches!(color_mask, 0b01 | 0b10) {
        let discardable: std::collections::BTreeSet<_> =
            crate::query::discardable_hand_equip_cards(tx.state(), player)
                .into_iter()
                .collect();
        let matching: Vec<_> = tx
            .state()
            .hand_cards(player)
            .into_iter()
            .filter(|card| {
                discardable.contains(card)
                    && crate::query::effective_card_color_mask(tx.state(), player, *card)
                        == color_mask
            })
            .collect();
        if !matching.is_empty() {
            events.extend(crate::effect::run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player,
                    cards: matching,
                }],
            )?);
        }
        return Ok(events);
    }
    if color_mask == 0b11 {
        events.extend(open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![0b01, 0b10],
            ChoiceKind::Custom,
            ContinuationKind::RevealHandChooseDiscardColor,
            vec![player.raw(), u32::from(color_mask)],
        )?);
    }
    Ok(events)
}

pub(crate) fn discard_target_cards(
    tx: &mut Transaction,
    actor: PlayerId,
    target: PlayerId,
    cards: Vec<CardId>,
    scope: rule_api::TargetCardScope,
) -> Result<Vec<DomainEvent>, EffectError> {
    if cards.is_empty() {
        return Err(EffectError::Illegal(
            "target discard selection is empty".into(),
        ));
    }
    if ![actor, target].into_iter().all(|player| {
        tx.state()
            .players
            .get(player)
            .is_some_and(|state| state.alive)
    }) {
        return Err(EffectError::Illegal(
            "target discard participants are not alive".into(),
        ));
    }
    let allowed: std::collections::BTreeSet<_> =
        crate::query::discardable_target_cards(tx.state(), actor, target, scope)
            .into_iter()
            .map(CardId::raw)
            .collect();
    let mut seen = std::collections::BTreeSet::new();
    let mut by_zone = std::collections::BTreeMap::<u32, Vec<CardId>>::new();
    for card in &cards {
        if !seen.insert(card.raw()) {
            return Err(EffectError::Illegal("duplicate target discard card".into()));
        }
        let card_state = tx
            .state()
            .cards
            .get(*card)
            .ok_or_else(|| EffectError::Illegal("bad target discard card".into()))?;
        if card_state.owner != target || !allowed.contains(&card.raw()) {
            return Err(EffectError::Illegal(
                "target discard card is outside the allowed scope".into(),
            ));
        }
        by_zone
            .entry(card_state.zone.raw())
            .or_default()
            .push(*card);
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let mut events = Vec::new();
    for (from, cards) in by_zone {
        events.extend(move_cards_with_reason(
            tx,
            cards,
            discard,
            Some(ZoneId::new(from)),
            CardMoveReason::Discard,
        )?);
    }
    Ok(events)
}

pub(crate) fn target_discard_card_category(
    state: &crate::state::GameState,
    target: PlayerId,
    card: CardId,
) -> u8 {
    let Some(card) = state.cards.get(card) else {
        return 0;
    };
    let kind = if state
        .zones
        .get(card.zone)
        .is_some_and(|zone| zone.kind == ZoneKind::Hand)
    {
        crate::query::effective_card_kind(state, target, card)
    } else {
        card.kind
    };
    if kind.is_basic() {
        1
    } else if kind.is_trick() {
        2
    } else if kind.is_equip() {
        3
    } else {
        0
    }
}

pub(crate) fn target_discard_cards_share_category(
    state: &crate::state::GameState,
    target: PlayerId,
    cards: &[CardId],
) -> bool {
    let Some(category) = cards
        .first()
        .map(|card| target_discard_card_category(state, target, *card))
        .filter(|category| *category != 0)
    else {
        return false;
    };
    cards
        .iter()
        .all(|card| target_discard_card_category(state, target, *card) == category)
}

pub(crate) fn choose_discard_target_cards_recover_same_category(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    count: u32,
    amount: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if ![player, target].into_iter().all(|participant| {
        tx.state()
            .players
            .get(participant)
            .is_some_and(|state| state.alive)
    }) {
        return Ok(Vec::new());
    }
    if count == 0 || !originating_skill_is_enabled(tx, player, skill) {
        return Err(EffectError::Illegal(
            "target discard-and-recover requires a positive count and the owner's enabled skill"
                .into(),
        ));
    }
    let options = crate::query::discardable_target_cards(
        tx.state(),
        player,
        target,
        rule_api::TargetCardScope::HandEquip,
    );
    let required = usize::try_from(count)
        .unwrap_or(usize::MAX)
        .min(options.len());
    if required == 0 {
        return Ok(Vec::new());
    }
    if required == options.len() {
        if required == usize::try_from(count).unwrap_or(usize::MAX)
            && amount > 0
            && target_discard_cards_share_category(tx.state(), target, &options)
        {
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::RecoverHp {
                    source: player,
                    player,
                    amount,
                });
        }
        return discard_target_cards(
            tx,
            player,
            target,
            options,
            rule_api::TargetCardScope::HandEquip,
        );
    }
    let required = u8::try_from(required)
        .map_err(|_| EffectError::ContentFault("target discard count exceeds 255".into()))?;
    let mut payload = vec![
        player.raw(),
        target.raw(),
        count,
        amount,
        skill.raw(),
        options.len().min(u32::MAX as usize) as u32,
    ];
    payload.extend(options.iter().map(|card| card.raw()));
    open_choice(
        tx,
        player,
        required,
        required,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseDiscardTargetCardsRecoverSameCategory,
        payload,
    )
}

/// Emit a cross-hand distinct-suit discard choice for 「试探」 (scskuiji). The chooser (`player`)
/// sees its own and `target`'s hand cards combined, may pick up to `count` of them with distinct
/// effective suits, and all picked cards are then discarded (split by owner at resolve time).
pub(crate) fn choose_scskuiji_discard(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if ![player, target].into_iter().all(|participant| {
        tx.state()
            .players
            .get(participant)
            .is_some_and(|state| state.alive)
    }) {
        return Ok(Vec::new());
    }
    if count == 0 || !originating_skill_is_enabled(tx, player, skill) {
        return Err(EffectError::Illegal(
            "scskuiji discard requires a positive count and the owner's enabled skill".into(),
        ));
    }
    let mut options: Vec<engine_api::CardId> = tx.state().hand_cards(player);
    options.extend(tx.state().hand_cards(target));
    let max = usize::try_from(count)
        .unwrap_or(usize::MAX)
        .min(options.len());
    let max = u8::try_from(max)
        .map_err(|_| EffectError::ContentFault("scskuiji discard count exceeds 255".into()))?;
    let mut payload = vec![
        player.raw(),
        target.raw(),
        count,
        skill.raw(),
        options.len() as u32,
    ];
    payload.extend(options.iter().map(|card| card.raw()));
    open_choice(
        tx,
        player,
        0,
        max,
        true,
        options.into_iter().map(engine_api::CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseScskuijiDiscard,
        payload,
    )
}

/// Emit the 「破试探」 (pskuiji) cross-hand distinct-suit discard. Identical option presentation
/// to `choose_scskuiji_discard`; the asymmetric HP-loss is applied at resolve time.
pub(crate) fn choose_pskuiji_discard(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if ![player, target].into_iter().all(|participant| {
        tx.state()
            .players
            .get(participant)
            .is_some_and(|state| state.alive)
    }) {
        return Ok(Vec::new());
    }
    if count == 0 || !originating_skill_is_enabled(tx, player, skill) {
        return Err(EffectError::Illegal(
            "pskuiji discard requires a positive count and the owner's enabled skill".into(),
        ));
    }
    let mut options: Vec<engine_api::CardId> = tx.state().hand_cards(player);
    options.extend(tx.state().hand_cards(target));
    let max = usize::try_from(count)
        .unwrap_or(usize::MAX)
        .min(options.len());
    let max = u8::try_from(max)
        .map_err(|_| EffectError::ContentFault("pskuiji discard count exceeds 255".into()))?;
    let mut payload = vec![
        player.raw(),
        target.raw(),
        count,
        skill.raw(),
        options.len() as u32,
    ];
    payload.extend(options.iter().map(|card| card.raw()));
    open_choice(
        tx,
        player,
        0,
        max,
        true,
        options.into_iter().map(engine_api::CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChoosePskuijiDiscard,
        payload,
    )
}

pub(crate) fn discard_from_hand(
    tx: &mut Transaction,
    player: PlayerId,
    count: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if count == 0 {
        return Ok(vec![]);
    }
    let hand = tx.state().hand_cards(player);
    if hand.is_empty() {
        return Ok(vec![]);
    }
    let n = (count as usize).min(hand.len());
    let cards: Vec<CardId> = hand.into_iter().take(n).collect();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    move_cards_with_reason(tx, cards, discard, None, CardMoveReason::Discard)
}

/// Deterministically discard up to `count` cards from `player`'s hand then equipment (legacy `he`
/// scope), mirroring `discard_from_hand` over the combined discardable hand+equipment set.
pub(crate) fn discard_from_hand_equip(
    tx: &mut Transaction,
    player: PlayerId,
    count: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if count == 0 {
        return Ok(vec![]);
    }
    let cards = crate::query::discardable_hand_equip_cards(tx.state(), player);
    if cards.is_empty() {
        return Ok(vec![]);
    }
    let n = (count as usize).min(cards.len());
    let cards: Vec<CardId> = cards.into_iter().take(n).collect();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    move_cards_with_reason(tx, cards, discard, None, CardMoveReason::Discard)
}

pub(crate) fn choose_discard_one_hand_or_equip(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
    if options.is_empty() {
        return Ok(vec![]);
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::DiscardOneHandOrEquip,
        vec![player.raw()],
    )
}

pub(crate) fn choose_draw_or_discard_add_skill(
    tx: &mut Transaction,
    player: PlayerId,
    draw_skill_def: engine_api::SkillDefId,
    discard_skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
    three_way: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    let state = tx.state();
    if !state.skills.get(skill).is_some_and(|instance| {
        instance.owner == player && state.skill_effectively_enabled(instance)
    }) {
        return Err(EffectError::ContentFault(
            "draw-or-discard choice requires its originating skill instance".into(),
        ));
    }
    if !state
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || state.turn.current != player
        || state.turn.phase != Phase::Play
    {
        return Ok(Vec::new());
    }
    let mut options = if three_way { vec![1, 2] } else { vec![1] };
    if !crate::query::discardable_hand_equip_cards(state, player).is_empty() {
        options.push(if three_way { 3 } else { 2 });
    }
    open_choice(
        tx,
        player,
        u8::from(!three_way),
        1,
        three_way,
        options,
        ChoiceKind::Custom,
        ContinuationKind::ChooseDrawOrDiscardAddSkill,
        vec![
            0,
            player.raw(),
            draw_skill_def.raw(),
            discard_skill_def.raw(),
            skill.raw(),
            u32::from(three_way),
        ],
    )
}

pub(crate) fn choose_draw_or_recast_add_skill(
    tx: &mut Transaction,
    player: PlayerId,
    draw_skill_def: engine_api::SkillDefId,
    recast_skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let state = tx.state();
    if !originating_skill_is_enabled(tx, player, skill)
        || draw_skill_def.is_none()
        || recast_skill_def.is_none()
    {
        return Err(EffectError::ContentFault(
            "draw-or-recast choice requires its originating and temporary skills".into(),
        ));
    }
    if !state
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || state.turn.current != player
        || state.turn.phase != Phase::Play
    {
        return Ok(Vec::new());
    }
    let mut options = vec![1];
    if !crate::query::discardable_hand_equip_cards(state, player).is_empty() {
        options.push(2);
    }
    open_choice(
        tx,
        player,
        0,
        1,
        true,
        options,
        ChoiceKind::Custom,
        ContinuationKind::ChooseDrawOrRecastAddSkill,
        vec![
            0,
            player.raw(),
            draw_skill_def.raw(),
            recast_skill_def.raw(),
            skill.raw(),
        ],
    )
}

pub(crate) fn choose_draw_or_discard_target_card_record_sha(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if skill.is_none() || !originating_skill_is_enabled(tx, player, skill) {
        return Err(EffectError::ContentFault(
            "draw-or-discard Sha choice requires its originating skill".into(),
        ));
    }
    let state = tx.state();
    if ![player, target].into_iter().all(|participant| {
        state
            .players
            .get(participant)
            .is_some_and(|participant| participant.alive)
    }) || !state.resolution.sha_target_resolving(player, card, target)
    {
        return Ok(Vec::new());
    }
    let mut options = vec![1];
    if !crate::query::discardable_target_cards(
        state,
        player,
        target,
        rule_api::TargetCardScope::HandEquip,
    )
    .is_empty()
    {
        options.push(2);
    }
    open_choice(
        tx,
        player,
        0,
        1,
        true,
        options,
        ChoiceKind::Custom,
        ContinuationKind::ChooseDrawOrDiscardTargetCardRecordSha,
        vec![player.raw(), card.raw(), target.raw(), skill.raw()],
    )
}

pub(crate) fn choose_gain_sha_for_opening_extra_play(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Err(EffectError::ContentFault(
            "opening extra Play Sha gain requires its originating skill".into(),
        ));
    }
    if tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || tx.state().turn.opening_extra_play_active_skill != skill
    {
        return Ok(Vec::new());
    }
    tx.state_mut()
        .skills
        .get_mut(skill)
        .expect("validated originating skill")
        .clear_phase_card_use_policy_state();
    let Some((from, card)) = first_shared_pile_card_by_kind(tx, CardKind::Sha)? else {
        return Ok(Vec::new());
    };
    let damage_baseline = tx.state().turn.opening_extra_play_damage_baseline;
    open_choice(
        tx,
        player,
        0,
        1,
        true,
        vec![1],
        ChoiceKind::Confirm,
        ContinuationKind::ChooseGainShaForOpeningExtraPlay,
        vec![
            player.raw(),
            card.raw(),
            from.raw(),
            skill.raw(),
            damage_baseline,
        ],
    )
}

pub(crate) fn lose_hp_gain_discard_card_for_opening_extra_play(
    tx: &mut Transaction,
    player: PlayerId,
    kind: u8,
    mandatory_skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if CardKind::from_tag(kind).is_none() || mandatory_skill_def.is_none() {
        return Err(EffectError::ContentFault(
            "opening extra Play HP-loss gain has invalid content arguments".into(),
        ));
    }
    if !originating_skill_is_enabled(tx, player, skill) {
        return Err(EffectError::ContentFault(
            "opening extra Play HP-loss gain requires its originating skill".into(),
        ));
    }
    if tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || tx.state().turn.opening_extra_play_active_skill != skill
    {
        return Ok(Vec::new());
    }
    let mandatory = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .any(|instance| instance.owner == player && instance.def == mandatory_skill_def);
    if mandatory {
        return commit_opening_extra_play_hp_loss_gain(tx, player, kind);
    }
    open_choice(
        tx,
        player,
        0,
        1,
        true,
        vec![1],
        ChoiceKind::Confirm,
        ContinuationKind::LoseHpGainDiscardCardForOpeningExtraPlay,
        vec![
            player.raw(),
            u32::from(kind),
            mandatory_skill_def.raw(),
            skill.raw(),
        ],
    )
}

pub(crate) fn commit_opening_extra_play_hp_loss_gain(
    tx: &mut Transaction,
    player: PlayerId,
    kind: u8,
) -> Result<Vec<DomainEvent>, EffectError> {
    super::run_effects(
        tx,
        vec![
            EffectRequest::DealDamage {
                source: PlayerId::NONE,
                target: player,
                amount: 1,
                is_lose_hp: true,
                nature: rule_api::damage_nature::NORMAL,
            },
            EffectRequest::GainDiscardCardByKind { player, kind },
        ],
    )
}

pub(crate) fn finish_opening_extra_play_sha_gain(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let Some((_, policy, damage_baseline)) = tx
        .state()
        .skills
        .get(skill)
        .and_then(|instance| instance.phase_card_use_policy())
    else {
        return Ok(Vec::new());
    };
    if policy & crate::state::SkillInstance::CARD_USE_POLICY_DISTANCE_UNLIMITED == 0 {
        return Ok(Vec::new());
    }
    let dealt_damage = tx
        .state()
        .damage_dealt_totals
        .get(&player)
        .copied()
        .unwrap_or(0)
        > damage_baseline;
    tx.state_mut()
        .skills
        .get_mut(skill)
        .expect("validated originating skill")
        .clear_phase_card_use_policy_state();
    if dealt_damage
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    super::run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source: player,
            target: player,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
}

pub(crate) fn reveal_target_hand_gain_shan_armor_or_draw_two(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Err(EffectError::ContentFault(
            "hand reveal gain requires its originating skill".into(),
        ));
    }
    if player == target
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "hand reveal gain requires distinct living players".into(),
        ));
    }
    let cards = tx.state().hand_cards(target);
    if cards.is_empty() {
        return Err(EffectError::Illegal(
            "hand reveal gain target has no hand cards".into(),
        ));
    }
    let matching: Vec<_> = cards
        .iter()
        .copied()
        .filter(|card| {
            tx.state().cards.get(*card).is_some_and(|state| {
                state.kind == CardKind::Armor
                    || crate::query::effective_card_kind(tx.state(), target, state)
                        == CardKind::Shan
            })
        })
        .collect();
    let mut effects = vec![EffectRequest::RevealCards {
        cards,
        to: Vec::new(),
    }];
    if matching.is_empty() {
        effects.push(EffectRequest::DrawCardsFromSkill {
            player,
            count: 2,
            skill,
        });
    } else {
        let from = tx
            .state()
            .zones
            .find(ZoneKind::Hand, target)
            .ok_or_else(|| EffectError::EngineFault("reveal target hand zone missing".into()))?;
        let to = tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::EngineFault("reveal recipient hand zone missing".into()))?;
        effects.push(EffectRequest::GiveCards {
            cards: matching,
            to,
            from,
        });
    }
    super::run_effects(tx, effects)
}

pub(crate) fn originating_skill_is_enabled(
    tx: &Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> bool {
    tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    })
}

/// 临界 (sp/ollinjie): after the owner takes damage, the damage source is forced to
/// discard one hand card; if the discarded card is effectively a Sha, the owner draws one.
///
/// The engine rechecks the hand options when this effect starts and revalidates the selected
/// card at the choice barrier; the conditional draw is scheduled with the discard in the same
/// queue (both serialized inside `ResolveOllinjie`).
pub(crate) fn resolve_ollinjie(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || source.is_none()
        || player == source
        || !tx
            .state()
            .players
            .get(source)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let options: Vec<_> = tx.state().hand_cards(source).into_iter().map(CardId::raw).collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        source,
        1,
        1,
        false,
        options,
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveOllinjie,
        vec![player.raw(), source.raw(), skill.raw()],
    )
}

pub(crate) fn selected_discard_draw_count(
    tx: &Transaction,
    player: PlayerId,
    cards: &[CardId],
) -> Result<u32, EffectError> {
    if cards.is_empty() {
        return Err(EffectError::Illegal(
            "discard-and-draw selection must not be empty".into(),
        ));
    }
    let allowed: std::collections::BTreeSet<_> =
        crate::query::discardable_hand_equip_cards(tx.state(), player)
            .into_iter()
            .collect();
    let unique: std::collections::BTreeSet<_> = cards.iter().copied().collect();
    if unique.len() != cards.len() || !cards.iter().all(|card| allowed.contains(card)) {
        return Err(EffectError::Illegal(
            "discard-and-draw selected cards became stale".into(),
        ));
    }
    let hand = tx.state().hand_cards(player);
    let bonus = u32::from(!hand.is_empty() && hand.iter().all(|card| unique.contains(card)));
    Ok(unique.len() as u32 + bonus)
}

pub(crate) fn discard_selected_draw_all_hand_bonus(
    tx: &mut Transaction,
    player: PlayerId,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let count = selected_discard_draw_count(tx, player, &cards)?;
    run_effects(
        tx,
        vec![
            EffectRequest::DiscardCards { player, cards },
            EffectRequest::DrawCardsFromSkill {
                player,
                count,
                skill,
            },
        ],
    )?;
    Ok(Vec::new())
}

pub(crate) fn choose_other_yinghun(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    count: u32,
    branch: u8,
    ready_to_discard: bool,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    if target.is_none() {
        let count = tx
            .state()
            .players
            .get(player)
            .map(|owner| owner.max_hp.saturating_sub(owner.hp.max(0) as u32))
            .unwrap_or(0);
        if count == 0 {
            return Ok(Vec::new());
        }
        let targets: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|&target| target != player)
            .map(PlayerId::raw)
            .collect();
        if targets.is_empty() {
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            targets,
            ChoiceKind::SelectTargets,
            ContinuationKind::ChooseOtherYinghun,
            vec![0, player.raw(), count, skill.raw()],
        );
    }
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
        || count == 0
        || !matches!(branch, 1 | 2)
    {
        return Ok(Vec::new());
    }
    if !ready_to_discard {
        tx.state_mut().resolution.deferred_effects.insert(
            0,
            EffectRequest::ChooseOtherYinghun {
                player,
                target,
                count,
                branch,
                ready_to_discard: true,
                skill,
            },
        );
        return draw_cards(tx, target, if branch == 1 { count } else { 1 });
    }
    let options = crate::query::discardable_hand_equip_cards(tx.state(), target);
    let required = usize::try_from(if branch == 1 { 1 } else { count })
        .unwrap_or(usize::MAX)
        .min(options.len());
    if required == 0 {
        return Ok(Vec::new());
    }
    if required == options.len() {
        return discard_cards(tx, target, options);
    }
    let required = u8::try_from(required)
        .map_err(|_| EffectError::ContentFault("Yinghun discard count exceeds 255".into()))?;
    let mut payload = vec![
        2,
        player.raw(),
        target.raw(),
        count,
        u32::from(branch),
        skill.raw(),
        options.len().min(u32::MAX as usize) as u32,
    ];
    payload.extend(options.iter().map(|card| card.raw()));
    open_choice(
        tx,
        target,
        required,
        required,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseOtherYinghun,
        payload,
    )
}

pub(crate) fn choose_other_bizheng(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&target| target != player)
                .map(PlayerId::raw)
                .collect();
            if targets.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                targets,
                ChoiceKind::SelectTargets,
                ContinuationKind::ChooseOtherBizheng,
                vec![0, player.raw(), skill.raw()],
            )
        }
        1 => {
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            tx.state_mut().resolution.deferred_effects.insert(
                0,
                EffectRequest::ChooseOtherBizheng {
                    player,
                    target,
                    stage: 2,
                    skill,
                },
            );
            draw_cards(tx, target, 2)
        }
        2 | 3 => {
            let discarder = if stage == 2 { player } else { target };
            let needs_discard = tx
                .state()
                .players
                .get(discarder)
                .is_some_and(|participant| {
                    participant.alive && tx.state().hand_count(discarder) > participant.max_hp
                });
            if !needs_discard {
                return if stage == 2 {
                    choose_other_bizheng(tx, player, target, 3, skill)
                } else {
                    Ok(Vec::new())
                };
            }
            let options = crate::query::discardable_hand_equip_cards(tx.state(), discarder);
            let required = options.len().min(2);
            if stage == 2 {
                tx.state_mut().resolution.deferred_tail_effects.push(
                    EffectRequest::ChooseOtherBizheng {
                        player,
                        target,
                        stage: 3,
                        skill,
                    },
                );
            }
            if required == 0 {
                return Ok(Vec::new());
            }
            if required == options.len() {
                return discard_cards(tx, discarder, options);
            }
            let mut payload = vec![
                1,
                player.raw(),
                target.raw(),
                discarder.raw(),
                skill.raw(),
                options.len().min(u32::MAX as usize) as u32,
            ];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                discarder,
                required as u8,
                required as u8,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ChooseOtherBizheng,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault("invalid Bizheng stage".into())),
    }
}

pub(crate) fn choose_other_xiashu(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            if tx.state().hand_count(player) == 0 {
                return Ok(Vec::new());
            }
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&target| target != player)
                .map(PlayerId::raw)
                .collect();
            if targets.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                targets,
                ChoiceKind::SelectTargets,
                ContinuationKind::ChooseOtherXiashu,
                vec![0, player.raw(), skill.raw()],
            )
        }
        1 => {
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let cards = tx.state().hand_cards(player);
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let from = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("Xiashu owner hand missing".into()))?;
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::EngineFault("Xiashu target hand missing".into()))?;
            tx.state_mut().resolution.deferred_effects.insert(
                0,
                EffectRequest::ChooseOtherXiashu {
                    player,
                    target,
                    stage: 2,
                    cards: Vec::new(),
                    skill,
                },
            );
            give_cards(tx, cards, to, from)
        }
        2 => {
            let options = tx.state().hand_cards(target);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            // ponytail: choice counts are u8; widen ChoiceRequest if real hands can exceed 255.
            let mut payload = vec![
                1,
                player.raw(),
                target.raw(),
                skill.raw(),
                options.len().min(u32::MAX as usize) as u32,
            ];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                target,
                1,
                options.len().min(u8::MAX as usize) as u8,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ChooseOtherXiashu,
                payload,
            )
        }
        3 => {
            let mut payload = vec![
                2,
                player.raw(),
                target.raw(),
                skill.raw(),
                cards.len().min(u32::MAX as usize) as u32,
            ];
            payload.extend(cards.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                vec![1, 2],
                ChoiceKind::Custom,
                ContinuationKind::ChooseOtherXiashu,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault("invalid Xiashu stage".into())),
    }
}

pub(crate) fn resolve_guanchao_rank(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
    {
        return Ok(Vec::new());
    }
    let mark = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.def.raw())
        .ok_or_else(|| EffectError::ContentFault("Guanchao skill is missing".into()))?;
    let mode = tx
        .state()
        .players
        .get(player)
        .and_then(|owner| owner.marks.iter().find(|(id, _)| *id == mark))
        .map(|(_, count)| *count)
        .unwrap_or(0);
    if !matches!(mode, 1 | 2) {
        return Ok(Vec::new());
    }
    let previous = (tx.state().turn.flag(turn_flag::LAST_PLAY_CARD_META) as u32 >> 8) & 0xff;
    if previous == 0 {
        return Ok(Vec::new());
    }
    let current = tx
        .state()
        .cards
        .get(card)
        .map(|card| u32::from(card.rank))
        .unwrap_or(0);
    let valid = if mode == 1 {
        current > previous
    } else {
        current < previous
    };
    if valid {
        draw_cards(tx, player, 1)
    } else {
        add_mark(tx, player, mark, 3 - mode)
    }
}

pub(crate) fn resolve_guanwei(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                player.raw(),
                target.raw(),
                skill.raw(),
                options.len().min(u32::MAX as usize) as u32,
            ];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveGuanwei,
                payload,
            )
        }
        1 => {
            tx.state_mut().resolution.deferred_effects.insert(
                0,
                EffectRequest::ResolveGuanwei {
                    player,
                    target,
                    stage: 2,
                    card: CardId::NONE,
                    skill,
                },
            );
            discard_cards(tx, player, vec![card])
        }
        2 => {
            tx.state_mut().resolution.deferred_effects.insert(
                0,
                EffectRequest::ResolveGuanwei {
                    player,
                    target,
                    stage: 3,
                    card: CardId::NONE,
                    skill,
                },
            );
            draw_cards(tx, target, 2)
        }
        3 => insert_extra_play_after_completed_play(tx, target),
        _ => Err(EffectError::ContentFault("invalid Guanwei stage".into())),
    }
}

pub(crate) fn resolve_lanjiang(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    index: u32,
    _target: PlayerId,
    targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let owner_hand = tx.state().hand_count(player);
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&target| tx.state().hand_count(target) >= owner_hand)
                .collect();
            resolve_lanjiang(tx, player, 1, 0, PlayerId::NONE, targets, skill)
        }
        1 => {
            let Some(target) = targets.get(index as usize).copied() else {
                return resolve_lanjiang(tx, player, 2, 0, PlayerId::NONE, targets, skill);
            };
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return resolve_lanjiang(
                    tx,
                    player,
                    1,
                    index.saturating_add(1),
                    PlayerId::NONE,
                    targets,
                    skill,
                );
            }
            let mut payload = vec![
                0,
                player.raw(),
                index,
                skill.raw(),
                targets.len().min(u32::MAX as usize) as u32,
            ];
            payload.extend(targets.iter().map(|target| target.raw()));
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                vec![0, 1],
                ChoiceKind::Confirm,
                ContinuationKind::ResolveLanjiang,
                payload,
            )
        }
        2 => {
            let owner_hand = tx.state().hand_count(player);
            let options: Vec<_> = targets
                .iter()
                .copied()
                .filter(|target| {
                    tx.state()
                        .players
                        .get(*target)
                        .is_some_and(|participant| participant.alive)
                        && tx.state().hand_count(*target) == owner_hand
                })
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                1,
                player.raw(),
                skill.raw(),
                targets.len().min(u32::MAX as usize) as u32,
            ];
            payload.extend(targets.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveLanjiang,
                payload,
            )
        }
        3 => {
            let owner_hand = tx.state().hand_count(player);
            let options: Vec<_> = targets
                .iter()
                .copied()
                .filter(|target| {
                    tx.state()
                        .players
                        .get(*target)
                        .is_some_and(|participant| participant.alive)
                        && tx.state().hand_count(*target) < owner_hand
                })
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![2, player.raw(), skill.raw()];
            payload.extend(targets.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveLanjiang,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault("invalid Lanjiang stage".into())),
    }
}

pub(crate) fn resolve_rebingyi(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let cards = tx.state().hand_cards(player);
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let mut events = vec![DomainEvent::CardsRevealed {
        cards: cards.clone(),
        to: Vec::new(),
    }];
    let first_color = crate::query::effective_card_color_mask(tx.state(), player, cards[0]);
    let uniform_color = matches!(first_color, 0b01 | 0b10)
        && cards.iter().all(|card| {
            crate::query::effective_card_color_mask(tx.state(), player, *card) == first_color
        });
    if !uniform_color {
        return Ok(events);
    }
    let first_rank = tx.state().cards.get(cards[0]).map(|card| card.rank);
    let self_draw = first_rank.is_some()
        && cards
            .iter()
            .all(|card| tx.state().cards.get(*card).map(|card| card.rank) == first_rank);
    let options = tx.state().players.alive_by_seat();
    let max = cards.len().min(options.len()).min(usize::from(u8::MAX)) as u8;
    if max == 0 {
        if self_draw {
            events.extend(draw_cards(tx, player, 1)?);
        }
        return Ok(events);
    }
    events.extend(open_choice(
        tx,
        player,
        0,
        max,
        true,
        options.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ResolveRebingyi,
        vec![
            player.raw(),
            u32::from(max),
            u32::from(self_draw),
            skill.raw(),
        ],
    )?);
    Ok(events)
}

pub(crate) fn resolve_rebotu(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.extra_turn_queue.len() >= 8
    {
        return Ok(Vec::new());
    }
    let current_round = tx.state().turn.round;
    let previous_count = tx
        .state()
        .skills
        .get(skill)
        .and_then(|skill| {
            (skill.state_blob.len() == 12 && &skill.state_blob[..4] == b"BTU1").then(|| {
                (
                    u32::from_le_bytes(skill.state_blob[4..8].try_into().unwrap()),
                    u32::from_le_bytes(skill.state_blob[8..12].try_into().unwrap()),
                )
            })
        })
        .filter(|(round, _)| *round == current_round)
        .map(|(_, count)| count)
        .unwrap_or(0);
    let cap = tx.state().players.alive_by_seat().len().min(3) as u32;
    if previous_count >= cap {
        return Ok(Vec::new());
    }
    let mut blob = b"BTU1".to_vec();
    blob.extend_from_slice(&current_round.to_le_bytes());
    blob.extend_from_slice(&previous_count.saturating_add(1).to_le_bytes());
    tx.state_mut()
        .skills
        .get_mut(skill)
        .ok_or_else(|| EffectError::EngineFault("Rebotu skill disappeared".into()))?
        .state_blob = blob;
    tx.state_mut().turn.extra_turn_queue.push(player);
    Ok(Vec::new())
}

pub(crate) fn oldimeng_pair_legal(
    tx: &Transaction,
    player: PlayerId,
    first: PlayerId,
    second: PlayerId,
) -> bool {
    first != player
        && second != player
        && first != second
        && [first, second].into_iter().all(|target| {
            tx.state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
        })
        && (tx.state().hand_count(first) > 0 || tx.state().hand_count(second) > 0)
        && tx
            .state()
            .hand_count(first)
            .abs_diff(tx.state().hand_count(second))
            <= crate::query::discardable_hand_equip_cards(tx.state(), player).len() as u32
}

pub(crate) fn resolve_oldimeng(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    first: PlayerId,
    second: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let alive = tx.state().players.alive_by_seat();
            let options: Vec<_> = alive
                .iter()
                .copied()
                .filter(|first| {
                    *first != player
                        && alive
                            .iter()
                            .copied()
                            .any(|second| oldimeng_pair_legal(tx, player, *first, second))
                })
                .map(PlayerId::raw)
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveOldimeng,
                vec![0, player.raw(), skill.raw()],
            )
        }
        1 if second.is_none() => {
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|second| oldimeng_pair_legal(tx, player, first, *second))
                .map(PlayerId::raw)
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveOldimeng,
                vec![1, player.raw(), first.raw(), skill.raw()],
            )
        }
        1 => {
            if !oldimeng_pair_legal(tx, player, first, second) {
                return Err(EffectError::Illegal(
                    "Oldimeng target pair became illegal".into(),
                ));
            }
            let first_cards = tx.state().hand_cards(first);
            let second_cards = tx.state().hand_cards(second);
            let first_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, first)
                .ok_or_else(|| EffectError::EngineFault("Oldimeng first hand missing".into()))?;
            let second_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, second)
                .ok_or_else(|| EffectError::EngineFault("Oldimeng second hand missing".into()))?;
            let mut events = move_cards(tx, first_cards, second_hand, Some(first_hand))?;
            events.extend(move_cards(tx, second_cards, first_hand, Some(second_hand))?);
            let mut blob = b"ODM1".to_vec();
            blob.extend_from_slice(&first.raw().to_le_bytes());
            blob.extend_from_slice(&second.raw().to_le_bytes());
            tx.state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::EngineFault("Oldimeng skill disappeared".into()))?
                .state_blob = blob;
            Ok(events)
        }
        2 => {
            let pair = tx.state().skills.get(skill).and_then(|skill| {
                (skill.state_blob.len() == 12 && &skill.state_blob[..4] == b"ODM1").then(|| {
                    (
                        PlayerId::new(u32::from_le_bytes(
                            skill.state_blob[4..8].try_into().unwrap(),
                        )),
                        PlayerId::new(u32::from_le_bytes(
                            skill.state_blob[8..12].try_into().unwrap(),
                        )),
                    )
                })
            });
            tx.state_mut()
                .skills
                .get_mut(skill)
                .expect("validated skill")
                .state_blob
                .clear();
            let Some((first, second)) = pair else {
                return Ok(Vec::new());
            };
            let required = tx
                .state()
                .hand_count(first)
                .abs_diff(tx.state().hand_count(second)) as usize;
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            let required = required.min(options.len());
            if required == 0 {
                return Ok(Vec::new());
            }
            if required == options.len() {
                return discard_cards(tx, player, options);
            }
            let mut payload = vec![
                2,
                player.raw(),
                skill.raw(),
                options.len().min(u32::MAX as usize) as u32,
                required as u32,
            ];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                required as u8,
                required as u8,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveOldimeng,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault("invalid Oldimeng stage".into())),
    }
}

pub(crate) fn rejieyin_can_equip(tx: &Transaction, target: PlayerId, card: CardId) -> bool {
    tx.state()
        .cards
        .get(card)
        .and_then(|card| card.kind.equip_slot())
        .is_some_and(|slot| {
            tx.state()
                .players
                .get(target)
                .is_some_and(|target| target.alive && target.equip_slot_enabled(slot))
        })
}

pub(crate) fn resolve_rejieyin(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    card: CardId,
    equip: bool,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            if !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&card) {
                return Err(EffectError::Illegal(
                    "Rejieyin selected card became illegal".into(),
                ));
            }
            let source_is_equip = tx.state().cards.get(card).is_some_and(|card| {
                tx.state()
                    .zones
                    .get(card.zone)
                    .is_some_and(|zone| zone.kind == ZoneKind::Equip)
            });
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&target| {
                    let male = tx
                        .state()
                        .players
                        .get(target)
                        .and_then(|target| tx.state().content.character(target.character_def))
                        .is_some_and(|character| {
                            matches!(character.sex.as_str(), "male" | "double")
                        });
                    male && (!source_is_equip || rejieyin_can_equip(tx, target, card))
                })
                .map(PlayerId::raw)
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveRejieyin,
                vec![0, player.raw(), card.raw(), skill.raw()],
            )
        }
        1 => {
            let source_is_equip = tx.state().cards.get(card).is_some_and(|card| {
                tx.state()
                    .zones
                    .get(card.zone)
                    .is_some_and(|zone| zone.kind == ZoneKind::Equip)
            });
            if source_is_equip {
                return resolve_rejieyin(tx, player, target, 2, card, true, skill);
            }
            if rejieyin_can_equip(tx, target, card) {
                return open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    vec![0, 1],
                    ChoiceKind::Custom,
                    ContinuationKind::ResolveRejieyin,
                    vec![1, player.raw(), target.raw(), card.raw(), skill.raw()],
                );
            }
            resolve_rejieyin(tx, player, target, 2, card, false, skill)
        }
        2 => {
            tx.state_mut().resolution.deferred_effects.insert(
                0,
                EffectRequest::ResolveRejieyin {
                    player,
                    target,
                    stage: 3,
                    card: CardId::NONE,
                    equip: false,
                    skill,
                },
            );
            if !equip {
                return discard_cards(tx, player, vec![card]);
            }
            if !rejieyin_can_equip(tx, target, card) {
                return Err(EffectError::Illegal(
                    "Rejieyin equipment target became illegal".into(),
                ));
            }
            let card_state = tx
                .state()
                .cards
                .get(card)
                .cloned()
                .ok_or_else(|| EffectError::Illegal("missing Rejieyin card".into()))?;
            let slot = card_state.kind.equip_slot().unwrap();
            let target_equip = tx
                .state()
                .zones
                .find(ZoneKind::Equip, target)
                .ok_or_else(|| EffectError::EngineFault("target equip zone missing".into()))?;
            let existing: Vec<_> = tx
                .state()
                .zones
                .get(target_equip)
                .into_iter()
                .flat_map(|zone| zone.cards.iter())
                .copied()
                .filter(|existing| {
                    tx.state()
                        .cards
                        .get(*existing)
                        .and_then(|card| card.kind.equip_slot())
                        == Some(slot)
                })
                .collect();
            let mut events = Vec::new();
            if !existing.is_empty() {
                let discard = tx
                    .state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
                events.extend(move_cards(tx, existing, discard, Some(target_equip))?);
            }
            events.extend(move_cards(
                tx,
                vec![card],
                target_equip,
                Some(card_state.zone),
            )?);
            Ok(events)
        }
        3 => {
            let (drawer, recovering) = match (
                tx.state().players.get(player).map(|player| player.hp),
                tx.state().players.get(target).map(|target| target.hp),
            ) {
                (Some(left), Some(right)) if left > right => (player, target),
                (Some(left), Some(right)) if left < right => (target, player),
                _ => return Ok(Vec::new()),
            };
            if tx
                .state()
                .players
                .get(recovering)
                .is_some_and(|participant| participant.hp < participant.max_hp as i32)
            {
                tx.state_mut().resolution.deferred_effects.insert(
                    0,
                    EffectRequest::RecoverHp {
                        source: drawer,
                        player: recovering,
                        amount: 1,
                    },
                );
            }
            draw_cards(tx, drawer, 1)
        }
        _ => Err(EffectError::ContentFault("invalid Rejieyin stage".into())),
    }
}

pub(crate) fn reanxu_cards(tx: &Transaction, player: PlayerId) -> Vec<CardId> {
    let mut cards = tx.state().hand_cards(player);
    if let Some(equip) = tx.state().zones.find(ZoneKind::Equip, player) {
        cards.extend(
            tx.state()
                .zones
                .get(equip)
                .into_iter()
                .flat_map(|zone| zone.cards.iter())
                .copied(),
        );
    }
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    cards
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_reanxu(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    first: PlayerId,
    second: PlayerId,
    card: CardId,
    from_hand: bool,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let alive = tx.state().players.alive_by_seat();
            if alive.len() <= 2 {
                return Ok(Vec::new());
            }
            let options: Vec<_> = alive
                .into_iter()
                .filter(|&target| target != player && !reanxu_cards(tx, target).is_empty())
                .map(PlayerId::raw)
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveReanxu,
                vec![0, player.raw(), skill.raw()],
            )
        }
        1 if second.is_none() => {
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&target| target != player && target != first)
                .map(PlayerId::raw)
                .collect();
            if options.is_empty() || reanxu_cards(tx, first).is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveReanxu,
                vec![1, player.raw(), first.raw(), skill.raw()],
            )
        }
        1 => {
            if player == first
                || player == second
                || first == second
                || ![first, second].into_iter().all(|target| {
                    tx.state()
                        .players
                        .get(target)
                        .is_some_and(|participant| participant.alive)
                })
            {
                return Err(EffectError::Illegal(
                    "Reanxu target pair became illegal".into(),
                ));
            }
            let options = reanxu_cards(tx, first);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                second,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveReanxu,
                vec![2, player.raw(), first.raw(), second.raw(), skill.raw()],
            )
        }
        2 => {
            let card_state = tx
                .state()
                .cards
                .get(card)
                .cloned()
                .ok_or_else(|| EffectError::Illegal("missing Reanxu card".into()))?;
            if card_state.owner != first || !reanxu_cards(tx, first).contains(&card) {
                return Err(EffectError::Illegal(
                    "Reanxu selected card became illegal".into(),
                ));
            }
            let from_hand = tx
                .state()
                .zones
                .get(card_state.zone)
                .is_some_and(|zone| zone.kind == ZoneKind::Hand);
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, second)
                .ok_or_else(|| EffectError::EngineFault("Reanxu target hand missing".into()))?;
            tx.state_mut().resolution.deferred_effects.insert(
                0,
                EffectRequest::ResolveReanxu {
                    player,
                    stage: 3,
                    first,
                    second,
                    card: CardId::NONE,
                    from_hand,
                    skill,
                },
            );
            move_cards(tx, vec![card], hand, Some(card_state.zone))
        }
        3 => {
            if from_hand {
                tx.state_mut().resolution.deferred_effects.insert(
                    0,
                    EffectRequest::ResolveReanxu {
                        player,
                        stage: 4,
                        first,
                        second,
                        card: CardId::NONE,
                        from_hand: false,
                        skill,
                    },
                );
                return draw_cards(tx, player, 1);
            }
            resolve_reanxu(tx, player, 4, first, second, CardId::NONE, false, skill)
        }
        4 => {
            if ![first, second].into_iter().all(|target| {
                tx.state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            }) {
                return Ok(Vec::new());
            }
            let first_hand = tx.state().hand_count(first);
            let second_hand = tx.state().hand_count(second);
            if first_hand == second_hand {
                return Ok(Vec::new());
            }
            let lower = if first_hand < second_hand {
                first
            } else {
                second
            };
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                vec![0, 1],
                ChoiceKind::Confirm,
                ContinuationKind::ResolveReanxu,
                vec![3, player.raw(), lower.raw(), skill.raw()],
            )
        }
        _ => Err(EffectError::ContentFault("invalid Reanxu stage".into())),
    }
}

pub(crate) fn resolve_oldanxu(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    giver: PlayerId,
    receiver: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let alive = tx.state().players.alive_by_seat();
            let options = alive
                .iter()
                .copied()
                .filter(|giver| {
                    *giver != player
                        && alive.iter().copied().any(|receiver| {
                            receiver != player
                                && receiver != *giver
                                && tx.state().hand_count(receiver) < tx.state().hand_count(*giver)
                        })
                })
                .map(PlayerId::raw)
                .collect::<Vec<_>>();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveOldAnxu,
                vec![0, player.raw(), skill.raw()],
            )
        }
        1 => {
            let options: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&target| {
                    target != player
                        && target != giver
                        && tx.state().hand_count(target) < tx.state().hand_count(giver)
                })
                .map(PlayerId::raw)
                .collect::<Vec<_>>();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveOldAnxu,
                vec![1, player.raw(), giver.raw(), skill.raw()],
            )
        }
        2 => {
            if player == giver
                || player == receiver
                || giver == receiver
                || tx.state().hand_count(giver) <= tx.state().hand_count(receiver)
            {
                return Err(EffectError::Illegal(
                    "old Anxu target pair became illegal".into(),
                ));
            }
            let options = tx.state().hand_cards(giver);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                receiver,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveOldAnxu,
                vec![2, player.raw(), giver.raw(), receiver.raw(), skill.raw()],
            )
        }
        3 => {
            let card_state = tx
                .state()
                .cards
                .get(card)
                .cloned()
                .ok_or_else(|| EffectError::Illegal("missing old Anxu card".into()))?;
            let giver_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, giver)
                .ok_or_else(|| EffectError::EngineFault("old Anxu giver hand missing".into()))?;
            if card_state.owner != giver
                || card_state.zone != giver_hand
                || tx.state().hand_count(giver) <= tx.state().hand_count(receiver)
            {
                return Err(EffectError::Illegal(
                    "old Anxu selected card became illegal".into(),
                ));
            }
            let receiver_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, receiver)
                .ok_or_else(|| EffectError::EngineFault("old Anxu receiver hand missing".into()))?;
            if card_state.suit != Suit::Spade {
                tx.state_mut()
                    .resolution
                    .deferred_effects
                    .insert(0, EffectRequest::DrawCards { player, count: 1 });
            }
            move_cards(tx, vec![card], receiver_hand, Some(giver_hand))
        }
        _ => Err(EffectError::ContentFault("invalid old Anxu stage".into())),
    }
}

pub(crate) fn resolve_oldqinqing(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    lord: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let Some(lord) = tx
                .state()
                .players
                .iter_live()
                .find(|participant| participant.alive && participant.identity == Identity::Lord)
                .map(|participant| participant.id)
            else {
                return Ok(Vec::new());
            };
            let alive = tx.state().players.alive_by_seat();
            if !alive.iter().copied().any(|candidate| {
                candidate != lord
                    && candidate != player
                    && crate::query::player_in_attack_range(tx.state(), candidate, lord)
            }) {
                return Ok(Vec::new());
            }
            let options = alive
                .into_iter()
                .filter(|candidate| {
                    *candidate != player
                        && crate::query::player_in_attack_range(tx.state(), *candidate, lord)
                })
                .map(PlayerId::raw)
                .collect::<Vec<_>>();
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveOldQinqing,
                vec![player.raw(), lord.raw(), skill.raw()],
            )
        }
        1 => {
            if ![player, target, lord].into_iter().all(|participant| {
                tx.state()
                    .players
                    .get(participant)
                    .is_some_and(|participant| participant.alive)
            }) || target == player
                || !crate::query::player_in_attack_range(tx.state(), target, lord)
            {
                return Err(EffectError::Illegal(
                    "old Qinqing target became illegal".into(),
                ));
            }
            let mut effects = Vec::new();
            if !crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                TargetCardScope::HandEquip,
            )
            .is_empty()
            {
                effects.push(EffectRequest::ChooseDiscardTargetCard {
                    player,
                    target,
                    scope: TargetCardScope::HandEquip,
                });
            }
            effects.push(EffectRequest::DrawCards {
                player: target,
                count: 1,
            });
            effects.push(EffectRequest::ResolveOldQinqing {
                player,
                target,
                lord,
                stage: 2,
                skill,
            });
            run_effects(tx, effects)
        }
        2 => {
            if tx
                .state()
                .players
                .get(lord)
                .is_some_and(|participant| participant.alive)
                && tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                && tx.state().hand_count(target) > tx.state().hand_count(lord)
            {
                draw_cards(tx, player, 1)
            } else {
                Ok(Vec::new())
            }
        }
        _ => Err(EffectError::ContentFault(
            "invalid old Qinqing stage".into(),
        )),
    }
}

pub(crate) fn resolve_oldmingjian(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            if tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || tx.state().hand_count(player) == 0
            {
                return Ok(Vec::new());
            }
            let options: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&target| target != player)
                .map(PlayerId::raw)
                .collect::<Vec<_>>();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveOldMingjian,
                vec![player.raw(), skill.raw()],
            )
        }
        1 => {
            if tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || tx.state().hand_count(player) == 0
                || target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal(
                    "old Mingjian target became illegal".into(),
                ));
            }
            if tx.state().turn.extra_play_phase_queue.len() >= 8 {
                return Err(EffectError::ContentFault(
                    "extra Play phase queue exceeds the eight-segment bound".into(),
                ));
            }
            tx.state_mut().turn.extra_play_phase_queue.push(target);
            run_effects(
                tx,
                vec![
                    EffectRequest::GiveAllHand { player, target },
                    EffectRequest::AdvanceTurn,
                ],
            )
        }
        _ => Err(EffectError::ContentFault(
            "invalid old Mingjian stage".into(),
        )),
    }
}

pub(crate) fn resolve_oldxuanfeng(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    stage: u8,
    branch: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let processing = tx
                .state()
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
            let card = tx.state_mut().cards.alloc(Card {
                id: CardId::NONE,
                def: CardDefId::NONE,
                kind: CardKind::Sha,
                suit: Suit::None,
                rank: 0,
                weapon_range: 0,
                zone: processing,
                owner: PlayerId::NONE,
                face_up: true,
            });
            tx.state_mut()
                .zones
                .get_mut(processing)
                .unwrap()
                .cards
                .push(card);
            let options = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&target| {
                    target != player
                        && (oldxuanfeng_distance_one(tx, player, target)
                            || crate::command::play::validate_sha_target_with_options(
                                tx, player, card, target, false,
                            )
                            .is_ok())
                })
                .map(PlayerId::raw)
                .collect::<Vec<_>>();
            if options.is_empty() {
                remove_oldxuanfeng_virtual_sha(tx, card)?;
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveOldXuanfeng,
                vec![0, player.raw(), card.raw(), skill.raw()],
            )
        }
        1 => {
            let direct = oldxuanfeng_distance_one(tx, player, target);
            let sha = crate::command::play::validate_sha_target_with_options(
                tx, player, card, target, false,
            )
            .is_ok();
            match (sha, direct) {
                (true, true) => open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    vec![1, 2],
                    ChoiceKind::Custom,
                    ContinuationKind::ResolveOldXuanfeng,
                    vec![1, player.raw(), target.raw(), card.raw(), skill.raw()],
                ),
                (true, false) => {
                    crate::command::play::handle_skill_card_with_policies(
                        tx,
                        player,
                        card,
                        &[target],
                        false,
                        true,
                    )?;
                    Ok(Vec::new())
                }
                (false, true) => {
                    remove_oldxuanfeng_virtual_sha(tx, card)?;
                    run_effects(
                        tx,
                        vec![EffectRequest::DealDamage {
                            source: player,
                            target,
                            amount: 1,
                            is_lose_hp: false,
                            nature: rule_api::damage_nature::NORMAL,
                        }],
                    )
                }
                (false, false) => Err(EffectError::Illegal(
                    "old Xuanfeng target became illegal".into(),
                )),
            }
        }
        2 => match branch {
            1 => {
                crate::command::play::handle_skill_card_with_policies(
                    tx,
                    player,
                    card,
                    &[target],
                    false,
                    true,
                )?;
                Ok(Vec::new())
            }
            2 if oldxuanfeng_distance_one(tx, player, target) => {
                remove_oldxuanfeng_virtual_sha(tx, card)?;
                run_effects(
                    tx,
                    vec![EffectRequest::DealDamage {
                        source: player,
                        target,
                        amount: 1,
                        is_lose_hp: false,
                        nature: rule_api::damage_nature::NORMAL,
                    }],
                )
            }
            _ => Err(EffectError::Illegal("invalid old Xuanfeng branch".into())),
        },
        _ => Err(EffectError::ContentFault(
            "invalid old Xuanfeng stage".into(),
        )),
    }
}

pub(crate) fn choose_target_gain_max_hp(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .map(PlayerId::raw)
        .collect();
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        targets,
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseTargetGainMaxHp,
        vec![player.raw(), skill.raw()],
    )
}

pub(crate) fn gain_target_hand_card(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != player && !tx.state().hand_cards(target).is_empty())
        .map(PlayerId::raw)
        .collect();
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        targets,
        ChoiceKind::SelectTargets,
        ContinuationKind::GainTargetHandCard,
        vec![0, player.raw(), skill.raw()],
    )
}

/// 穆荫 (clan/clanmuyin, ABI 648): the owner selects a living player who is the
/// owner themself or shares a clan with the owner and whose hand limit is not
/// the field maximum; that player permanently gains +1 hand limit.
pub(crate) fn choose_clan_not_max_hand_limit_gain(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let owner_clans: std::collections::BTreeSet<String> = tx
        .state()
        .players
        .get(player)
        .map(|p| p.character_def)
        .and_then(|def| tx.state().content.character(def))
        .map(|c| c.clans.iter().cloned().collect())
        .unwrap_or_default();
    let hand_limit = |state: &crate::state::GameState, target: PlayerId| -> i32 {
        match crate::query::query(
            state,
            &rule_api::QueryRequest::HandLimit { player: target },
            &mut crate::query::QueryContext::new(),
        ) {
            rule_api::QueryResponse::Int(n) => n,
            _ => 0,
        }
    };
    let state = tx.state();
    let is_field_max = |target: PlayerId| -> bool {
        let own = hand_limit(state, target);
        state.players.alive_by_seat().into_iter().any(|other| {
            other != target && hand_limit(state, other) > own
        })
    };
    let targets: Vec<u32> = state
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| {
            if target == player {
                return !is_field_max(target);
            }
            let shares_clan = state
                .players
                .get(target)
                .map(|p| p.character_def)
                .and_then(|def| state.content.character(def))
                .is_some_and(|c| c.clans.iter().any(|clan| owner_clans.contains(clan)));
            shares_clan && !is_field_max(target)
        })
        .map(PlayerId::raw)
        .collect();
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        targets,
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseClanNotMaxHandLimitGain,
        vec![player.raw(), skill.raw()],
    )
}

/// 额外结算 (clanfenshi family, ABI 649): queue one extra effect pass for the
/// currently suspended trick. Upstream `trigger.effectCount++` runs inside the
/// CardUsed trigger window, i.e. while the trick sits in its WuxieWindow
/// continuation; the extra passes are applied by `apply_deferred_trick` via
/// `extra_effect_count` once the window clears.
pub(crate) fn extra_resolve_current_trick(
    tx: &mut Transaction,
    player: PlayerId,
    card: engine_api::CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    if tx.state().resolution.resume_kind != crate::state::ContinuationKind::WuxieWindow {
        return Err(EffectError::ContentFault(
            "extra resolution requires a suspended trick (WuxieWindow)".into(),
        ));
    }
    // Resolve the suspended source card from the wuxie payload marker
    // (same extraction as `increase_saved_card_base_damage`: rposition over the
    // whole ask-phase resume payload).
    let payload = &tx.state().resolution.resume_payload;
    let suspended_card = payload
        .iter()
        .rposition(|value| *value == crate::command::play::SAVED_WUXIE_SOURCE_CARD_MAGIC)
        .map(|index| {
            engine_api::CardId::new(*payload.get(index + 1).unwrap_or(&0))
        })
        .unwrap_or(engine_api::CardId::NONE);
    if card.is_none() || card != suspended_card {
        return Err(EffectError::Illegal(
            "extra resolution card does not match the suspended trick".into(),
        ));
    }
    let entry = tx.state_mut().extra_effect_count.entry(card.raw()).or_insert(0);
    if *entry >= 2 {
        return Err(EffectError::Illegal(
            "extra resolution pass limit reached".into(),
        ));
    }
    *entry += 1;
    Ok(Vec::new())
}
