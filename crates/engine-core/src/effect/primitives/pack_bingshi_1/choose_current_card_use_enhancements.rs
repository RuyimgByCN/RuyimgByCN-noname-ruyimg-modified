//! Effect primitives for `choose_current_card_use_enhancements` and related helpers.

use super::*;

pub(crate) fn choose_current_card_use_enhancements(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    helper_skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if helper_skill_def.is_none()
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::ContentFault(
            "card-use enhancements require their originating skill and helper definition".into(),
        ));
    }
    let Some(extra_targets) =
        crate::command::play::current_card_use_enhancement_extra_targets(tx, player, card)?
    else {
        return Ok(Vec::new());
    };
    let mut options = Vec::with_capacity(4);
    if !extra_targets.is_empty() {
        options.push(0);
    }
    options.extend([1, 2, 3]);
    open_choice(
        tx,
        player,
        1,
        2,
        false,
        options,
        ChoiceKind::Custom,
        ContinuationKind::ChooseCurrentCardUseEnhancements,
        vec![
            player.raw(),
            card.raw(),
            helper_skill_def.raw(),
            skill.raw(),
        ],
    )
}

pub(crate) fn add_card_use_policy_skill(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    policy_mask: u8,
    skill_def: engine_api::SkillDefId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let policy_mask = policy_mask & 0b111;
    if policy_mask == 0 || card.is_none() || skill_def.is_none() {
        return Ok(Vec::new());
    }
    let mut entries = tx
        .state()
        .players
        .get(player)
        .into_iter()
        .flat_map(|participant| participant.skills.iter())
        .find_map(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .filter(|instance| instance.def == skill_def)
                .and_then(|instance| instance.card_use_policies())
        })
        .unwrap_or_default();
    if let Some((_, mask)) = entries.iter_mut().find(|(entry, _)| *entry == card) {
        *mask |= policy_mask;
    } else {
        entries.push((card, policy_mask));
    }
    add_skill_with_state(
        tx,
        player,
        skill_def,
        Some(crate::state::SkillInstance::card_use_policy_state(&entries)),
    )
}

pub(crate) fn skill_pile_def(
    tx: &Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<u32, EffectError> {
    let state = tx.state();
    state
        .skills
        .get(skill)
        .filter(|instance| instance.owner == player && state.skill_effectively_enabled(instance))
        .map(|instance| instance.def.raw())
        .ok_or_else(|| EffectError::ContentFault("invalid skill-pile owner or skill".into()))
}

pub(crate) fn hand_sha_cards(tx: &Transaction, player: PlayerId) -> Vec<CardId> {
    tx.state()
        .hand_cards(player)
        .into_iter()
        .filter(|card| {
            tx.state().cards.get(*card).is_some_and(|card| {
                crate::query::effective_card_kind(tx.state(), player, card).is_sha()
            })
        })
        .collect()
}

pub(crate) fn choose_store_hand_sha_in_skill_pile(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let skill_def = skill_pile_def(tx, player, skill)?;
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || !tx.state().skill_pile_cards(player, skill_def).is_empty()
    {
        return Ok(Vec::new());
    }
    let cards = hand_sha_cards(tx, player);
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        cards.len().min(u8::MAX as usize) as u8,
        false,
        cards.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseStoreHandShaInSkillPile,
        vec![player.raw(), skill.raw()],
    )
}

pub(crate) fn choose_spend_skill_pile_for_dying_jiu(
    tx: &mut Transaction,
    player: PlayerId,
    dying: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let skill_def = skill_pile_def(tx, player, skill)?;
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || !tx.state().players.get(dying).is_some_and(|participant| {
            participant.alive && participant.dying && participant.hp <= 0
        })
    {
        return Ok(Vec::new());
    }
    let cards = tx.state().skill_pile_cards(player, skill_def);
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        cards.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseSpendSkillPileForDyingJiu,
        vec![player.raw(), dying.raw(), skill.raw()],
    )
}

pub(crate) fn ensure_skill_pile_zone(
    tx: &mut Transaction,
    player: PlayerId,
    skill_def: u32,
) -> Result<ZoneId, EffectError> {
    if let Some(zone) = tx
        .state()
        .skill_piles
        .iter()
        .find(|pile| pile.owner == player && pile.skill_def == skill_def)
        .map(|pile| pile.zone)
    {
        return tx
            .state()
            .zones
            .get(zone)
            .filter(|zone| zone.kind == ZoneKind::Expansion && zone.owner == player)
            .map(|zone| zone.id)
            .ok_or_else(|| EffectError::EngineFault("invalid skill-pile zone".into()));
    }
    let zone = tx.state_mut().zones.alloc(Zone {
        id: ZoneId::NONE,
        kind: ZoneKind::Expansion,
        owner: player,
        cards: Vec::new(),
    });
    tx.state_mut().skill_piles.push(crate::state::SkillPileRef {
        owner: player,
        skill_def,
        zone,
        usable_as_hand: false,
        wants_lost_count: false,
    });
    tx.state_mut()
        .skill_piles
        .sort_by_key(|pile| (pile.owner.raw(), pile.skill_def));
    Ok(zone)
}

pub(crate) fn commit_store_hand_sha_in_skill_pile(
    tx: &mut Transaction,
    player: PlayerId,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let skill_def = skill_pile_def(tx, player, skill)?;
    if cards.is_empty() || !tx.state().skill_pile_cards(player, skill_def).is_empty() {
        return Err(EffectError::Illegal(
            "skill-pile storage requires selected cards and an empty pile".into(),
        ));
    }
    let legal: std::collections::BTreeSet<_> = hand_sha_cards(tx, player).into_iter().collect();
    let selected: std::collections::BTreeSet<_> = cards.iter().copied().collect();
    if selected.len() != cards.len() || !selected.is_subset(&legal) {
        return Err(EffectError::Illegal(
            "skill-pile storage must select distinct current hand Sha".into(),
        ));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let pile = ensure_skill_pile_zone(tx, player, skill_def)?;
    move_cards(tx, cards, pile, Some(hand))
}

pub(crate) fn store_selected_in_skill_pile(
    tx: &mut Transaction,
    player: PlayerId,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let skill_def = skill_pile_def(tx, player, skill)?;
    if cards.is_empty() {
        // 死腿修复（批 215）：DSL 直排语句没有前置选择阶段，cards 恒空导致该腿
        // 在已部署件中静默无效——改为自开 SelectCards 1/1（从当前手牌，必选）。
        if !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        {
            return Ok(Vec::new());
        }
        let hand_cards: Vec<u32> = tx
            .state()
            .hand_cards(player)
            .iter()
            .map(|card| card.raw())
            .collect();
        if hand_cards.is_empty() {
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            hand_cards,
            ChoiceKind::SelectCards,
            ContinuationKind::StoreSelectedInSkillPile,
            vec![player.raw(), skill.raw()],
        );
    }
    let legal: std::collections::BTreeSet<_> = tx.state().hand_cards(player).into_iter().collect();
    let selected: std::collections::BTreeSet<_> = cards.iter().copied().collect();
    if selected.len() != cards.len() || !selected.is_subset(&legal) {
        return Err(EffectError::Illegal(
            "skill-pile storage must select distinct current hand cards".into(),
        ));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let pile = ensure_skill_pile_zone(tx, player, skill_def)?;
    move_cards(tx, cards, pile, Some(hand))
}

pub(crate) fn gain_top_card_to_skill_pile(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let skill_def = skill_pile_def(tx, player, skill)?;
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("draw pile is missing".into()))?;
    if tx
        .state()
        .zones
        .get(draw)
        .is_none_or(|zone| zone.cards.is_empty())
    {
        reshuffle_discard_into_draw(tx)?;
    }
    let Some(card) = tx
        .state()
        .zones
        .get(draw)
        .and_then(|zone| zone.cards.last())
        .copied()
    else {
        return Ok(Vec::new());
    };
    let pile = ensure_skill_pile_zone(tx, player, skill_def)?;
    move_cards(tx, vec![card], pile, Some(draw))
}

pub(crate) fn choose_discard_skill_pile_card_add_marked_skill(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    helper_skill_def: engine_api::SkillDefId,
    mut selected: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let skill_def = skill_pile_def(tx, player, skill)?;
    if player == target
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("invalid marked-skill target".into()));
    }
    let cards = tx.state().skill_pile_cards(player, skill_def);
    if cards.is_empty() {
        return Err(EffectError::Illegal("current skill pile is empty".into()));
    }
    if selected.is_none() && cards.len() > 1 {
        open_choice(
            tx,
            player,
            1,
            1,
            false,
            cards.iter().map(|card| card.raw()).collect(),
            ChoiceKind::Custom,
            ContinuationKind::ChooseDiscardSkillPileCardAddMarkedSkill,
            vec![
                player.raw(),
                target.raw(),
                helper_skill_def.raw(),
                skill.raw(),
            ],
        )?;
        return Err(EffectError::ChoiceRequested);
    }
    if selected.is_none() {
        selected = cards[0];
    }
    if !cards.contains(&selected) {
        return Err(EffectError::Illegal(
            "selected card is not in the current skill pile".into(),
        ));
    }
    let from = tx
        .state()
        .cards
        .get(selected)
        .map(|card| card.zone)
        .ok_or_else(|| EffectError::Illegal("selected pile card is missing".into()))?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("discard pile is missing".into()))?;
    tx.state_mut().resolution.deferred_tail_effects.extend([
        EffectRequest::AddSkill {
            player: target,
            skill_def: helper_skill_def,
        },
        EffectRequest::AddMark {
            player: target,
            mark: engine_api::MarkDefId::new(helper_skill_def.raw()),
            delta: 1,
        },
    ]);
    move_cards_with_reason(
        tx,
        vec![selected],
        discard,
        Some(from),
        CardMoveReason::Discard,
    )
}

pub(crate) fn end_match_for_camp_if_skill_pile_at_least(
    tx: &mut Transaction,
    player: PlayerId,
    pile_skill_def: engine_api::SkillDefId,
    count: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if tx.state().outcome.is_finished()
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || tx
            .state()
            .skill_pile_cards(player, pile_skill_def.raw())
            .len()
            < count as usize
    {
        return Ok(Vec::new());
    }
    let identity = tx
        .state()
        .players
        .get(player)
        .ok_or_else(|| EffectError::Illegal("bad camp winner".into()))?
        .identity;
    let outcome = match identity {
        Identity::Lord | Identity::Loyalist => MatchOutcome::LoyalistWin,
        Identity::Rebel => MatchOutcome::RebelWin,
        Identity::Renegade => MatchOutcome::RenegadeWin { player },
    };
    tx.journal(JournalEntry::OutcomeCheckpoint);
    tx.state_mut().outcome = outcome.clone();
    Ok(vec![DomainEvent::MatchEnded { outcome }])
}

pub(crate) fn commit_spend_skill_pile_for_dying_jiu(
    tx: &mut Transaction,
    player: PlayerId,
    dying: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let skill_def = skill_pile_def(tx, player, skill)?;
    if !tx
        .state()
        .players
        .get(dying)
        .is_some_and(|participant| participant.alive && participant.dying && participant.hp <= 0)
        || !tx
            .state()
            .skill_pile_cards(player, skill_def)
            .contains(&card)
    {
        return Err(EffectError::Illegal(
            "dying Jiu requires the current dying player and one current skill-pile card".into(),
        ));
    }
    let pile = tx
        .state()
        .skill_piles
        .iter()
        .find(|pile| pile.owner == player && pile.skill_def == skill_def)
        .map(|pile| pile.zone)
        .ok_or_else(|| EffectError::EngineFault("skill-pile zone missing".into()))?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let mut events = move_cards(tx, vec![card], discard, Some(pile))?;
    events.extend(use_virtual_card_on(tx, dying, CardKind::Jiu.tag(), dying)?);
    Ok(events)
}

pub(crate) fn potweiyan_status(tx: &Transaction, skill: engine_api::SkillId) -> u8 {
    tx.state()
        .skills
        .get(skill)
        .and_then(|instance| instance.state_blob.strip_prefix(b"PWY1"))
        .and_then(|rest| rest.first())
        .copied()
        .unwrap_or(0)
}

pub(crate) fn set_potweiyan_status(tx: &mut Transaction, skill: engine_api::SkillId, status: u8) {
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob = [b"PWY1".as_slice(), &[status]].concat();
    }
}

pub(crate) fn set_potkuanggu_upgraded(tx: &mut Transaction, skill: engine_api::SkillId) {
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob = b"PKG1".to_vec();
    }
}

pub(crate) fn potkuanggu_upgraded(tx: &Transaction, skill: engine_api::SkillId) -> bool {
    tx.state()
        .skills
        .get(skill)
        .is_some_and(|instance| instance.state_blob == b"PKG1")
}

pub(crate) fn ensure_marked_skill(
    tx: &mut Transaction,
    player: PlayerId,
    def: engine_api::SkillDefId,
    count: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let mut events = if skill_instance_by_def(tx, player, def).is_none() {
        add_skill(tx, player, def)?
    } else {
        Vec::new()
    };
    let skill = skill_instance_by_def(tx, player, def)
        .ok_or_else(|| EffectError::EngineFault("marked helper was not added".into()))?;
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.mark = instance.mark.saturating_add(count);
    }
    events.push(DomainEvent::RuleNote {
        code: 558,
        payload: vec![player.raw(), def.raw(), count],
    });
    Ok(events)
}

pub(crate) fn fail_potweiyan_mission(
    tx: &mut Transaction,
    player: PlayerId,
    mission: engine_api::SkillId,
    zhuangshi_def: engine_api::SkillDefId,
    kunfen_def: engine_api::SkillDefId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if potweiyan_status(tx, mission) != 0 {
        return Ok(Vec::new());
    }
    set_potweiyan_status(tx, mission, 2);
    let mut events = remove_skill_by_def(tx, player, zhuangshi_def)?;
    if skill_instance_by_def(tx, player, kunfen_def).is_none() {
        events.extend(add_skill(tx, player, kunfen_def)?);
    }
    events.push(DomainEvent::RuleNote {
        code: 558,
        payload: vec![player.raw(), 2],
    });
    Ok(events)
}

pub(crate) fn potyinzhan_entries(
    tx: &Transaction,
    skill: engine_api::SkillId,
) -> (CardId, Vec<(PlayerId, bool)>) {
    let Some(blob) = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.state_blob.as_slice())
        .filter(|blob| blob.starts_with(b"PYZ1") && blob.len() >= 9)
    else {
        return (CardId::NONE, Vec::new());
    };
    let card = CardId::new(u32::from_le_bytes(blob[4..8].try_into().unwrap()));
    let count = blob[8] as usize;
    if blob.len() != 9 + count * 5 {
        return (CardId::NONE, Vec::new());
    }
    let entries = blob[9..]
        .chunks_exact(5)
        .map(|chunk| {
            (
                PlayerId::new(u32::from_le_bytes(chunk[..4].try_into().unwrap())),
                chunk[4] != 0,
            )
        })
        .collect();
    (card, entries)
}
