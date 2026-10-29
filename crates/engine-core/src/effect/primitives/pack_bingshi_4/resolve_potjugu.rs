//! Effect primitives for `resolve_mbfeijing` and related helpers.

use super::*;

pub(crate) fn resolve_potjugu(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let count = {
        let instance = tx
            .state_mut()
            .skills
            .get_mut(skill)
            .ok_or_else(|| EffectError::EngineFault("拒孤 skill disappeared".into()))?;
        instance.turn_fire_count = instance.turn_fire_count.saturating_add(1);
        instance.turn_fire_count
    };
    let mut events = run_effects(tx, vec![EffectRequest::DrawCards { player, count: 2 }])?;
    let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
    let need = options.len().min(count as usize);
    if need == 0 {
        return Ok(events);
    }
    let need_u8 = u8::try_from(need)
        .map_err(|_| EffectError::Illegal("拒孤 discard count exceeds 255".into()))?;
    let mut payload = vec![
        POTJUGU_CUSTOM_CHOICE,
        player.raw(),
        skill.raw(),
        need as u32,
    ];
    payload.extend(options.iter().map(|card| card.raw()));
    events.extend(open_choice(
        tx,
        player,
        need_u8,
        need_u8,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        payload,
    )?);
    Ok(events)
}

pub(crate) fn resolve_potsifeng(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let skill_def = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.def.raw())
        .ok_or_else(|| EffectError::EngineFault("伺锋 skill disappeared".into()))?;
    match stage {
        0 => {
            if !tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .any(|target| target != player)
            {
                return Ok(Vec::new());
            }
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("draw pile missing".into()))?;
            let cards: Vec<_> = tx
                .state()
                .zones
                .get(draw)
                .into_iter()
                .flat_map(|zone| zone.cards.iter().rev())
                .take(3)
                .copied()
                .collect();
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let processing = tx
                .state()
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
            let mut events = move_cards(tx, cards.clone(), processing, Some(draw))?;
            events.extend(begin_potsifeng_distribution(tx, player, skill, cards)?);
            Ok(events)
        }
        3 => {
            let cards = tx.state().skill_pile_cards(target, skill_def);
            let Some(first) = cards.first().copied() else {
                return Ok(Vec::new());
            };
            let pile = ensure_skill_pile_zone(tx, target, skill_def)?;
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
            let different = crate::query::effective_card_color_mask(tx.state(), target, card)
                != crate::query::effective_card_color_mask(tx.state(), target, first);
            let mut events = move_cards(tx, vec![first], discard, Some(pile))?;
            if different {
                let options = crate::query::discardable_target_cards(
                    tx.state(),
                    target,
                    target,
                    TargetCardScope::Hand,
                );
                if !options.is_empty() {
                    let mut payload = vec![POTSIFENG_CUSTOM_CHOICE, 2, target.raw()];
                    payload.extend(options.iter().map(|card| card.raw()));
                    events.extend(open_choice(
                        tx,
                        target,
                        1,
                        1,
                        false,
                        options.into_iter().map(CardId::raw).collect(),
                        ChoiceKind::SelectCards,
                        ContinuationKind::Custom,
                        payload,
                    )?);
                }
            }
            Ok(events)
        }
        4 => open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![0, 1],
            ChoiceKind::Custom,
            ContinuationKind::Custom,
            vec![
                POTSIFENG_CUSTOM_CHOICE,
                3,
                player.raw(),
                target.raw(),
                skill.raw(),
            ],
        ),
        5 | 6 => {
            let cards = tx.state().skill_pile_cards(target, skill_def);
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let pile = ensure_skill_pile_zone(tx, target, skill_def)?;
            if stage == 6 {
                let hand = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, player)
                    .ok_or_else(|| EffectError::EngineFault("伺锋 owner hand missing".into()))?;
                return move_cards(tx, cards, hand, Some(pile));
            }
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
            let mut events = move_cards(tx, cards, discard, Some(pile))?;
            events.extend(run_effects(
                tx,
                vec![EffectRequest::DealDamage {
                    source: player,
                    target,
                    amount: 1,
                    is_lose_hp: false,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )?);
            Ok(events)
        }
        _ => Err(EffectError::ContentFault("bad 伺锋 stage".into())),
    }
}

pub(crate) fn potfaen_state(tx: &Transaction, skill: engine_api::SkillId) -> (PlayerId, u8) {
    let Some(blob) = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| &instance.state_blob)
    else {
        return (PlayerId::NONE, 0);
    };
    if blob.len() != 9 || &blob[..4] != b"FEN1" {
        return (PlayerId::NONE, 0);
    }
    (
        PlayerId::new(u32::from_le_bytes(blob[4..8].try_into().unwrap())),
        blob[8],
    )
}

pub(crate) fn set_potfaen_state(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    last_user: PlayerId,
    pending: u8,
) {
    let mut blob = b"FEN1".to_vec();
    blob.extend_from_slice(&last_user.raw().to_le_bytes());
    blob.push(pending);
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob = blob;
}

pub(crate) fn open_potfaen_primary(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let mut options = vec![1];
    if !crate::query::discardable_hand_equip_cards(tx.state(), target).is_empty() {
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
        ContinuationKind::Custom,
        vec![
            POTCHENQUN_CUSTOM_CHOICE,
            0,
            player.raw(),
            target.raw(),
            skill.raw(),
        ],
    )
}

pub(crate) fn resolve_potchenqun(
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
            let (previous, pending) = potfaen_state(tx, skill);
            set_potfaen_state(tx, skill, target, 0);
            let offer = previous == player;
            match pending {
                1 => {
                    let mut effects = vec![EffectRequest::DrawCards {
                        player: target,
                        count: 1,
                    }];
                    if offer {
                        effects.push(EffectRequest::ResolvePotchenqun {
                            player,
                            target,
                            stage: 2,
                            skill,
                        });
                    }
                    run_effects(tx, effects)
                }
                2 => {
                    let options = crate::query::discardable_hand_equip_cards(tx.state(), target);
                    if options.is_empty() {
                        return if offer {
                            open_potfaen_primary(tx, player, target, skill)
                        } else {
                            Ok(Vec::new())
                        };
                    }
                    let mut payload = vec![
                        POTCHENQUN_CUSTOM_CHOICE,
                        2,
                        player.raw(),
                        target.raw(),
                        skill.raw(),
                        u32::from(offer),
                    ];
                    payload.extend(options.iter().map(|card| card.raw()));
                    open_choice(
                        tx,
                        target,
                        1,
                        1,
                        false,
                        options.into_iter().map(CardId::raw).collect(),
                        ChoiceKind::SelectCards,
                        ContinuationKind::Custom,
                        payload,
                    )
                }
                _ if offer => open_potfaen_primary(tx, player, target, skill),
                _ => Ok(Vec::new()),
            }
        }
        1 => {
            let losses: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(|target| (target, tx.state().turn.card_loss_count(target)))
                .collect();
            let max = losses.iter().map(|(_, count)| *count).max().unwrap_or(0);
            if max == 0 {
                return Ok(Vec::new());
            }
            let options: Vec<_> = losses
                .into_iter()
                .filter_map(|(target, count)| (count == max).then_some(target.raw()))
                .collect();
            let mut payload = vec![POTCHENQUN_CUSTOM_CHOICE, 3, player.raw(), skill.raw()];
            payload.extend_from_slice(&options);
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )
        }
        2 => open_potfaen_primary(tx, player, target, skill),
        _ => Err(EffectError::ContentFault("bad 势陈群 stage".into())),
    }
}

pub(crate) fn resolve_potwanglie(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    helper_skill_def: engine_api::SkillDefId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let cards = tx.state().hand_cards(player);
            if cards.is_empty() || helper_skill_def.is_none() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                POTCHENDAO_CUSTOM_CHOICE,
                0,
                player.raw(),
                helper_skill_def.raw(),
                skill.raw(),
            ];
            payload.extend(cards.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                cards.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                payload,
            )
        }
        1 => {
            let armed = tx
                .state()
                .skills
                .get(skill)
                .and_then(|instance| instance.phase_card_use_policy())
                .is_some_and(|(armed, _, _)| armed == card);
            if !armed {
                return Ok(Vec::new());
            }
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .clear_phase_card_use_policy_state();
            add_skill(tx, player, helper_skill_def)
        }
        2 => {
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .clear_phase_card_use_policy_state();
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault("bad 势陈到往烈 stage".into())),
    }
}

pub(crate) fn pothongyi_mark(
    tx: &Transaction,
    skill: engine_api::SkillId,
) -> engine_api::MarkDefId {
    tx.state()
        .skills
        .get(skill)
        .map(|instance| engine_api::MarkDefId::new(instance.def.raw()))
        .unwrap_or(engine_api::MarkDefId::NONE)
}

pub(crate) fn pothongyi_pending(tx: &Transaction, skill: engine_api::SkillId) -> Option<(u8, u32)> {
    let blob = &tx.state().skills.get(skill)?.state_blob;
    (blob.len() == 9 && blob.get(..4) == Some(b"PHY1"))
        .then(|| (blob[4], u32::from_le_bytes(blob[5..9].try_into().unwrap())))
}

pub(crate) fn pothongyi_mark_count(
    tx: &Transaction,
    player: PlayerId,
    mark: engine_api::MarkDefId,
) -> u32 {
    tx.state()
        .players
        .get(player)
        .and_then(|player| {
            player
                .marks
                .iter()
                .find(|(id, _)| *id == mark.raw())
                .map(|(_, count)| (*count).max(0) as u32)
        })
        .unwrap_or(0)
}

pub(crate) fn resolve_pothongyi(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let mark = pothongyi_mark(tx, skill);
    let count = pothongyi_mark_count(tx, player, mark).min(5);
    match stage {
        0 => add_mark(tx, player, mark.raw(), 4_i32.saturating_sub(count as i32)),
        1 if count < 5 => add_mark(tx, player, mark.raw(), 1),
        1 => Ok(Vec::new()),
        2 if count > 0 => open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![0, 1],
            ChoiceKind::Custom,
            ContinuationKind::Custom,
            vec![POTCHENDAO_CUSTOM_CHOICE, 1, player.raw(), skill.raw()],
        ),
        2 => Ok(Vec::new()),
        3 => {
            let Some((branch, captured)) = pothongyi_pending(tx, skill) else {
                return Ok(Vec::new());
            };
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .state_blob
                .clear();
            if branch == 0 {
                remove_mark(tx, player, mark.raw(), 0)
            } else {
                run_effects(
                    tx,
                    vec![EffectRequest::DrawCards {
                        player,
                        count: captured,
                    }],
                )
            }
        }
        _ => Err(EffectError::ContentFault("bad 势陈到弘毅 stage".into())),
    }
}

pub(crate) fn commit_pothongyi_prepare(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    branch: u8,
) -> Result<Vec<DomainEvent>, EffectError> {
    if branch > 1 || !originating_skill_is_enabled(tx, player, skill) {
        return Err(EffectError::Illegal("invalid 弘毅 branch".into()));
    }
    let mark = pothongyi_mark(tx, skill);
    let count = pothongyi_mark_count(tx, player, mark).min(5);
    if count == 0 {
        return Ok(Vec::new());
    }
    let mut state = b"PHY1".to_vec();
    state.push(branch);
    state.extend_from_slice(&count.to_le_bytes());
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob = state;
    if branch == 0 {
        run_effects(tx, vec![EffectRequest::DrawCards { player, count }])
    } else {
        remove_mark(tx, player, mark.raw(), 0)
    }
}

pub(crate) fn arm_potwanglie(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<(), EffectError> {
    if !tx.state().hand_cards(player).contains(&card)
        || !originating_skill_is_enabled(tx, player, skill)
    {
        return Err(EffectError::Illegal("invalid 往烈 hand card".into()));
    }
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob =
        crate::state::SkillInstance::phase_card_use_policy_state(
            card,
            crate::state::SkillInstance::CARD_USE_POLICY_UNRESPONDABLE
                | crate::state::SkillInstance::CARD_USE_POLICY_DISTANCE_UNLIMITED,
            0,
        );
    Ok(())
}

pub(crate) fn first_shared_pile_card_by_category(
    tx: &Transaction,
    player: PlayerId,
    allowed: impl Fn(u8) -> bool,
) -> Result<Option<(ZoneId, CardId)>, EffectError> {
    for zone_kind in [ZoneKind::DrawPile, ZoneKind::DiscardPile] {
        let from = tx
            .state()
            .zones
            .find(zone_kind, PlayerId::NONE)
            .ok_or_else(|| EffectError::Illegal("shared card zone missing".into()))?;
        if let Some(card) = tx.state().zones.get(from).and_then(|zone| {
            zone.cards
                .iter()
                .rev()
                .copied()
                .find(|card| allowed(effective_card_category(tx.state(), player, *card)))
        }) {
            return Ok(Some((from, card)));
        }
    }
    Ok(None)
}

pub(crate) fn resolve_potguansha(
    tx: &mut Transaction,
    player: PlayerId,
    helper_skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) || helper_skill_def.is_none() {
        return Ok(Vec::new());
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let equip = tx
        .state()
        .zones
        .find(ZoneKind::Equip, player)
        .ok_or_else(|| EffectError::Illegal("no equip zone".into()))?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let hand_cards = tx.state().zones.get(hand).unwrap().cards.clone();
    let equip_cards = tx.state().zones.get(equip).unwrap().cards.clone();
    let replace_count = hand_cards.len().saturating_add(equip_cards.len());
    if replace_count == 0 {
        return Ok(Vec::new());
    }
    let mut events = Vec::new();
    if !hand_cards.is_empty() {
        events.extend(move_cards(tx, hand_cards, discard, Some(hand))?);
    }
    if !equip_cards.is_empty() {
        events.extend(move_cards(tx, equip_cards, discard, Some(equip))?);
    }
    let mut gained = 0u32;
    while gained < replace_count.min(u32::MAX as usize) as u32 {
        let Some((from, card)) =
            first_shared_pile_card_by_category(tx, player, |category| category == 1)?
        else {
            break;
        };
        events.extend(move_cards(tx, vec![card], hand, Some(from))?);
        gained += 1;
    }
    if gained > 0 {
        events.extend(add_skill(tx, player, helper_skill_def)?);
        events.extend(add_mark(tx, player, helper_skill_def.raw(), gained as i32)?);
    }
    Ok(events)
}

pub(crate) fn potjiyu_state(tx: &Transaction, skill: engine_api::SkillId) -> (u8, Vec<CardId>) {
    let Some(blob) = tx.state().skills.get(skill).map(|skill| &skill.state_blob) else {
        return (0, Vec::new());
    };
    if blob.len() < 5 || (blob.len() - 5) % 4 != 0 || blob.get(..4) != Some(b"PJY1") {
        return (0, Vec::new());
    }
    (
        blob[4].min(2),
        blob[5..]
            .chunks_exact(4)
            .map(|raw| CardId::new(u32::from_le_bytes(raw.try_into().unwrap())))
            .filter(|card| card.is_some())
            .collect(),
    )
}

pub(crate) fn set_potjiyu_state(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    refreshes: u8,
    cards: &[CardId],
) {
    let mut blob = b"PJY1".to_vec();
    blob.push(refreshes.min(2));
    for card in cards {
        blob.extend_from_slice(&card.raw().to_le_bytes());
    }
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob = blob;
}

pub(crate) fn resolve_potjiyu(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let source_category = effective_card_category(tx.state(), player, card);
            if source_category == 0
                || !tx.state().hand_cards(player).contains(&card)
                || !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&card)
            {
                return Err(EffectError::Illegal("invalid 急御 hand card".into()));
            }
            let hand = tx.state().cards.get(card).unwrap().zone;
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
            let mut events = move_cards(tx, vec![card], discard, Some(hand))?;
            let (refreshes, _) = potjiyu_state(tx, skill);
            let mut categories = 1u8 << source_category;
            let mut gained = Vec::new();
            loop {
                let Some((from, gain)) =
                    first_shared_pile_card_by_category(tx, player, |category| {
                        category > 0 && categories & (1u8 << category) == 0
                    })?
                else {
                    break;
                };
                let category = effective_card_category(tx.state(), player, gain);
                categories |= 1u8 << category;
                events.extend(move_cards(tx, vec![gain], hand, Some(from))?);
                gained.push(gain);
                if categories & 0b1110 == 0b1110 {
                    break;
                }
            }
            if !gained.is_empty() {
                set_potjiyu_state(tx, skill, refreshes, &gained);
            }
            Ok(events)
        }
        1 => {
            let (refreshes, tracked) = potjiyu_state(tx, skill);
            if tracked.is_empty() {
                return Ok(Vec::new());
            }
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            if tracked.iter().any(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|card| Some(card.zone) == hand)
            }) {
                return Ok(Vec::new());
            }
            let next = refreshes.saturating_add(1).min(2);
            set_potjiyu_state(tx, skill, next, &[]);
            if refreshes < 2 {
                tx.state_mut().skills.get_mut(skill).unwrap().use_count = 0;
            }
            Ok(Vec::new())
        }
        2 => {
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .state_blob
                .clear();
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault("bad 势娄圭急御 stage".into())),
    }
}
