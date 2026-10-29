//! Effect primitives for `choose_move_field_card_to` and related helpers.

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

pub(crate) fn remove_cards_by_kind(
    tx: &mut Transaction,
    kind: u8,
) -> Result<Vec<DomainEvent>, EffectError> {
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
