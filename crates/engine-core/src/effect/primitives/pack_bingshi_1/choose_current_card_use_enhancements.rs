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

pub(crate) fn set_potyinzhan_entries(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    card: CardId,
    entries: &[(PlayerId, bool)],
) {
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        let mut blob = b"PYZ1".to_vec();
        blob.extend_from_slice(&card.raw().to_le_bytes());
        blob.push(entries.len().min(u8::MAX as usize) as u8);
        for (target, both) in entries.iter().take(u8::MAX as usize) {
            blob.extend_from_slice(&target.raw().to_le_bytes());
            blob.push(u8::from(*both));
        }
        instance.state_blob = blob;
    }
}

pub(crate) fn resolve_potweiyan(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let kuanggu_def = engine_api::SkillDefId::new(POTWEIYAN_KUANGGU_DEF);
    let zhuangshi_def = engine_api::SkillDefId::new(POTWEIYAN_ZHUANGSHI_DEF);
    let kunfen_def = engine_api::SkillDefId::new(POTWEIYAN_KUNFEN_DEF);
    let direct_def = engine_api::SkillDefId::new(POTWEIYAN_DIRECT_DEF);
    let limit_def = engine_api::SkillDefId::new(POTWEIYAN_LIMIT_DEF);
    let kuanggu_limit_def = engine_api::SkillDefId::new(POTWEIYAN_KUANGGU_LIMIT_DEF);
    if action <= 10 && !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            if potweiyan_status(tx, skill) == 0
                && skill_instance_by_def(tx, player, kuanggu_def).is_none()
            {
                add_skill(tx, player, kuanggu_def)
            } else {
                Ok(Vec::new())
            }
        }
        1 => {
            if potweiyan_status(tx, skill) != 0 {
                return Ok(Vec::new());
            }
            set_potweiyan_status(tx, skill, 1);
            if let Some(kuanggu) = skill_instance_by_def(tx, player, kuanggu_def) {
                set_potkuanggu_upgraded(tx, kuanggu);
            }
            let direct_left = skill_instance_by_def(tx, player, direct_def)
                .and_then(|id| tx.state().skills.get(id))
                .map_or(0, |instance| instance.mark);
            let limit_left = skill_instance_by_def(tx, player, limit_def)
                .and_then(|id| tx.state().skills.get(id))
                .map_or(0, |instance| instance.mark);
            let mut effects = Vec::new();
            if limit_left > 0 {
                effects.push(EffectRequest::DrawCards { player, count: 1 });
            }
            if direct_left > 0 {
                let damaged = tx
                    .state()
                    .players
                    .get(player)
                    .is_some_and(|participant| participant.hp < participant.max_hp as i32);
                effects.push(if damaged {
                    EffectRequest::RecoverHp {
                        player,
                        amount: 1,
                        source: PlayerId::NONE,
                    }
                } else {
                    EffectRequest::DrawCards { player, count: 1 }
                });
            }
            run_effects(tx, effects)?;
            Ok(vec![DomainEvent::RuleNote {
                code: 558,
                payload: vec![player.raw(), 1],
            }])
        }
        2 => {
            if potweiyan_status(tx, skill) != 0
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || skill_instance_by_def(tx, player, zhuangshi_def).is_none()
            {
                return Ok(Vec::new());
            }
            let hand = crate::query::discardable_hand_equip_cards(tx.state(), player)
                .into_iter()
                .filter(|card| tx.state().hand_cards(player).contains(card))
                .collect::<Vec<_>>();
            if hand.is_empty() {
                return open_potweiyan_hp_choice(
                    tx,
                    player,
                    skill,
                    kuanggu_def,
                    zhuangshi_def,
                    kunfen_def,
                    direct_def,
                    limit_def,
                    kuanggu_limit_def,
                    0,
                );
            }
            open_choice(
                tx,
                player,
                1,
                hand.len().min(u8::MAX as usize) as u8,
                true,
                hand.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    POTWEIYAN_CUSTOM_CHOICE,
                    0,
                    player.raw(),
                    skill.raw(),
                    kuanggu_def.raw(),
                    zhuangshi_def.raw(),
                    kunfen_def.raw(),
                    direct_def.raw(),
                    limit_def.raw(),
                    kuanggu_limit_def.raw(),
                ],
            )
        }
        3 => fail_potweiyan_mission(tx, player, skill, zhuangshi_def, kunfen_def),
        4 | 5 => {
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.mark = instance.mark.saturating_sub(1);
                if instance.mark == 0 {
                    return remove_skill(tx, skill);
                }
            }
            Ok(Vec::new())
        }
        6 => remove_skill(tx, skill),
        7 => {
            let owner = tx
                .state()
                .players
                .get(player)
                .ok_or_else(|| EffectError::Illegal("bad 饮战 owner".into()))?;
            let victim = tx
                .state()
                .players
                .get(target)
                .ok_or_else(|| EffectError::Illegal("bad 饮战 target".into()))?;
            let hp_branch = owner.hp <= victim.hp;
            let hand_branch = tx.state().hand_count(player) <= tx.state().hand_count(target);
            if hp_branch {
                increase_bound_damage_by_one(tx, tx.state().resolution.current_damage_id)?;
            }
            if hand_branch {
                let (stored_card, mut entries) = potyinzhan_entries(tx, skill);
                if entries.is_empty() || stored_card == card {
                    entries.push((target, hp_branch));
                    set_potyinzhan_entries(tx, skill, card, &entries);
                }
            }
            Ok(Vec::new())
        }
        8 => continue_potyinzhan(tx, player, card, skill),
        9 => {
            let damaged = tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.hp < participant.max_hp as i32);
            let mut options = vec![1];
            if damaged {
                options.insert(0, 0);
            }
            if potkuanggu_upgraded(tx, skill)
                && !crate::query::discardable_hand_equip_cards(tx.state(), player).is_empty()
            {
                options.push(2);
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                options,
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    POTWEIYAN_CUSTOM_CHOICE,
                    3,
                    player.raw(),
                    skill.raw(),
                    kuanggu_limit_def.raw(),
                ],
            )
        }
        10 => {
            run_effects(
                tx,
                vec![
                    EffectRequest::DealDamage {
                        source: PlayerId::NONE,
                        target: player,
                        amount: 1,
                        is_lose_hp: true,
                        nature: 0,
                    },
                    EffectRequest::DrawCards { player, count: 2 },
                ],
            )?;
            Ok(Vec::new())
        }
        11 => finish_potyinzhan_entry(tx, player, target, card, skill),
        12 => {
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if options.is_empty() {
                return ensure_marked_skill(tx, player, kuanggu_limit_def, 1);
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    POTWEIYAN_CUSTOM_CHOICE,
                    4,
                    player.raw(),
                    skill.raw(),
                    kuanggu_limit_def.raw(),
                ],
            )
        }
        13 => ensure_marked_skill(tx, player, kuanggu_limit_def, 1),
        14 => {
            let count = card.raw();
            let mut events = ensure_marked_skill(tx, player, direct_def, count)?;
            events.extend(open_potweiyan_hp_choice(
                tx,
                player,
                skill,
                kuanggu_def,
                zhuangshi_def,
                kunfen_def,
                direct_def,
                limit_def,
                kuanggu_limit_def,
                count,
            )?);
            Ok(events)
        }
        15 => ensure_marked_skill(tx, player, limit_def, card.raw()),
        _ => Err(EffectError::ContentFault("unknown 势魏延 action".into())),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn open_potweiyan_hp_choice(
    tx: &mut Transaction,
    player: PlayerId,
    mission: engine_api::SkillId,
    kuanggu_def: engine_api::SkillDefId,
    zhuangshi_def: engine_api::SkillDefId,
    kunfen_def: engine_api::SkillDefId,
    direct_def: engine_api::SkillDefId,
    limit_def: engine_api::SkillDefId,
    kuanggu_limit_def: engine_api::SkillDefId,
    discarded: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let hp = tx
        .state()
        .players
        .get(player)
        .map_or(0, |participant| participant.hp.max(0) as u32);
    if hp == 0 {
        return fail_potweiyan_mission(tx, player, mission, zhuangshi_def, kunfen_def);
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        (1..=hp).collect(),
        ChoiceKind::Custom,
        ContinuationKind::Custom,
        vec![
            POTWEIYAN_CUSTOM_CHOICE,
            1,
            player.raw(),
            mission.raw(),
            kuanggu_def.raw(),
            zhuangshi_def.raw(),
            kunfen_def.raw(),
            direct_def.raw(),
            limit_def.raw(),
            kuanggu_limit_def.raw(),
            discarded,
        ],
    )
}

pub(crate) fn continue_potyinzhan(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let (stored_card, entries) = potyinzhan_entries(tx, skill);
    if entries.is_empty() || stored_card != card {
        return Ok(Vec::new());
    }
    let (target, _) = entries[0];
    let options = crate::query::discardable_hand_equip_cards(tx.state(), target);
    if options.is_empty() {
        return finish_potyinzhan_entry(tx, player, target, CardId::NONE, skill);
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        vec![
            POTWEIYAN_CUSTOM_CHOICE,
            2,
            player.raw(),
            skill.raw(),
            target.raw(),
        ],
    )
}

pub(crate) fn finish_potyinzhan_entry(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    discarded: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let (card, mut entries) = potyinzhan_entries(tx, skill);
    let Some((expected, both)) = entries.first().copied() else {
        return Ok(Vec::new());
    };
    if expected != target {
        return Err(EffectError::Illegal("饮战 target changed".into()));
    }
    entries.remove(0);
    set_potyinzhan_entries(tx, skill, card, &entries);
    let mut effects = Vec::new();
    if both {
        effects.push(EffectRequest::RecoverHp {
            player,
            amount: 1,
            source: PlayerId::NONE,
        });
        if discarded.is_some()
            && tx.state().cards.get(discarded).is_some_and(|card| {
                tx.state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .is_some_and(|discard| card.zone == discard)
            })
        {
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("hand missing".into()))?;
            effects.push(EffectRequest::MoveCards {
                cards: vec![discarded],
                to: hand,
                from: tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE),
            });
        }
    }
    effects.push(EffectRequest::ResolvePotweiyan {
        player,
        target: PlayerId::NONE,
        card,
        action: 8,
        skill,
    });
    run_effects(tx, effects)?;
    Ok(Vec::new())
}

pub(crate) fn potzhouyu_turn_allowed(tx: &Transaction, skill: engine_api::SkillId) -> bool {
    tx.state().skills.get(skill).is_some_and(|instance| {
        instance.content_flags & (POTZHOUYU_DISABLE_ONE | POTZHOUYU_DISABLE_TWO) == 0
            || tx.state().turn.current == instance.owner
    })
}

pub(crate) fn potzhouyu_branch_allowed(
    tx: &Transaction,
    skill: engine_api::SkillId,
    branch: u8,
) -> bool {
    tx.state().skills.get(skill).is_some_and(|instance| {
        let flag = if branch == 1 {
            POTZHOUYU_DISABLE_ONE
        } else {
            POTZHOUYU_DISABLE_TWO
        };
        instance.content_flags & flag == 0
    })
}

pub(crate) fn potyanhui_state(tx: &Transaction, skill: engine_api::SkillId) -> PotyanhuiState {
    let Some(blob) = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.state_blob.as_slice())
    else {
        return PotyanhuiState::default();
    };
    if !blob.starts_with(b"PYH1") || blob.len() < 6 {
        return PotyanhuiState::default();
    }
    let armed = blob[4] != 0;
    let revealed_len = blob[5] as usize;
    let mut offset = 6;
    if blob.len() < offset + revealed_len * 8 + 1 {
        return PotyanhuiState::default();
    }
    let mut revealed = Vec::with_capacity(revealed_len);
    for chunk in blob[offset..offset + revealed_len * 8].chunks_exact(8) {
        revealed.push((
            CardId::new(u32::from_le_bytes(chunk[..4].try_into().unwrap())),
            PlayerId::new(u32::from_le_bytes(chunk[4..].try_into().unwrap())),
        ));
    }
    offset += revealed_len * 8;
    let discarded_len = blob[offset] as usize;
    offset += 1;
    if blob.len() != offset + discarded_len * 4 {
        return PotyanhuiState::default();
    }
    let discarded = blob[offset..]
        .chunks_exact(4)
        .map(|raw| PlayerId::new(u32::from_le_bytes(raw.try_into().unwrap())))
        .collect();
    PotyanhuiState {
        armed,
        revealed,
        discarded,
    }
}

pub(crate) fn set_potyanhui_state(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    state: &PotyanhuiState,
) {
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        let mut blob = b"PYH1".to_vec();
        blob.push(u8::from(state.armed));
        blob.push(state.revealed.len().min(u8::MAX as usize) as u8);
        for (card, player) in state.revealed.iter().take(u8::MAX as usize) {
            blob.extend_from_slice(&card.raw().to_le_bytes());
            blob.extend_from_slice(&player.raw().to_le_bytes());
        }
        blob.push(state.discarded.len().min(u8::MAX as usize) as u8);
        for player in state.discarded.iter().take(u8::MAX as usize) {
            blob.extend_from_slice(&player.raw().to_le_bytes());
        }
        instance.state_blob = blob;
    }
}

pub(crate) fn set_potfentao_pending(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    target: PlayerId,
) {
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob = [b"PFT1".as_slice(), &target.raw().to_le_bytes()].concat();
    }
}

pub(crate) fn add_potfentao_damage(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    propagated: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    let delta = if propagated {
        let instance = tx
            .state_mut()
            .skills
            .get_mut(skill)
            .ok_or_else(|| EffectError::Illegal("焚涛技能已失效".into()))?;
        instance.mark = instance.mark.saturating_add(1);
        instance.mark
    } else {
        if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
            instance.mark = 0;
        }
        1
    };
    increase_bound_damage(tx, tx.state().resolution.current_damage_id, delta)?;
    Ok(vec![DomainEvent::RuleNote {
        code: 561,
        payload: vec![skill.raw(), delta],
    }])
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_potzhouyu(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    action: u8,
    card_flags: u8,
    targets: &[PlayerId],
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if action <= 7 && !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            if !potzhouyu_turn_allowed(tx, skill) {
                return Ok(Vec::new());
            }
            let Some(instance) = tx.state_mut().skills.get_mut(skill) else {
                return Ok(Vec::new());
            };
            if instance.content_flags & 1 != 0 {
                return Ok(Vec::new());
            }
            instance.content_flags |= 1;
            let cards = tx.state().hand_cards(player);
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                cards.len().min(u8::MAX as usize) as u8,
                true,
                cards.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![POTZHOUYU_CUSTOM_CHOICE, 0, player.raw(), skill.raw()],
            )
        }
        1 => {
            if !potzhouyu_turn_allowed(tx, skill) {
                return Ok(Vec::new());
            }
            let eligible: Vec<_> = targets
                .iter()
                .copied()
                .filter(|candidate| {
                    tx.state()
                        .players
                        .get(*candidate)
                        .is_some_and(|role| role.alive)
                        && tx.state().hand_count(*candidate) > 0
                })
                .collect();
            if eligible.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                eligible.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![POTZHOUYU_CUSTOM_CHOICE, 3, player.raw(), skill.raw()],
            )
        }
        2 => {
            let mut state = potyanhui_state(tx, skill);
            if !state.armed {
                return Ok(Vec::new());
            }
            state.armed = false;
            let damage_targets: Vec<_> = state
                .discarded
                .iter()
                .copied()
                .filter(|target| {
                    tx.state()
                        .players
                        .get(*target)
                        .is_some_and(|role| role.alive)
                })
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect();
            state.discarded.clear();
            let draw_count = state
                .revealed
                .iter()
                .map(|(_, role)| *role)
                .collect::<std::collections::BTreeSet<_>>()
                .len() as u32;
            set_potyanhui_state(tx, skill, &state);
            let mut options = Vec::new();
            if draw_count > 0 && potzhouyu_branch_allowed(tx, skill, 2) {
                options.push(0);
            }
            if potzhouyu_branch_allowed(tx, skill, 1) {
                options.extend(damage_targets.iter().map(|target| target.raw()));
            }
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                POTZHOUYU_CUSTOM_CHOICE,
                5,
                player.raw(),
                skill.raw(),
                draw_count,
                damage_targets.len() as u32,
            ];
            payload.extend(damage_targets.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                payload,
            )
        }
        3 => {
            let propagated = card_flags & rule_api::damage_card_flag::CHAIN_PROPAGATED != 0;
            if !propagated {
                if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                    instance.mark = 0;
                }
            }
            if !potzhouyu_turn_allowed(tx, skill) {
                return Ok(Vec::new());
            }
            let linked_other = tx.state().players.get(target).is_some_and(|role| {
                role.alive
                    && target != player
                    && role
                        .marks
                        .iter()
                        .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
            });
            if !linked_other {
                return Ok(Vec::new());
            }
            let allow_one = potzhouyu_branch_allowed(tx, skill, 1);
            let allow_two = potzhouyu_branch_allowed(tx, skill, 2);
            let discardable = crate::query::discardable_hand_equip_cards(tx.state(), target);
            let equip_count = tx
                .state()
                .zones
                .find(ZoneKind::Equip, target)
                .and_then(|zone| tx.state().zones.get(zone))
                .map_or(0, |zone| zone.cards.len());
            let required = (tx.state().hand_count(target) + equip_count as u32).div_ceil(2);
            let can_discard = required > 0 && discardable.len() as u32 >= required;
            if !allow_two || !can_discard {
                return if allow_one {
                    add_potfentao_damage(tx, skill, propagated)
                } else {
                    Ok(Vec::new())
                };
            }
            let required_u8 = required.min(u8::MAX as u32) as u8;
            open_choice(
                tx,
                target,
                required_u8,
                required_u8,
                allow_one,
                discardable.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    POTZHOUYU_CUSTOM_CHOICE,
                    6,
                    player.raw(),
                    skill.raw(),
                    target.raw(),
                    u32::from(propagated),
                    required as u32,
                ],
            )
        }
        4 => {
            let pending = tx.state().skills.get(skill).is_some_and(|instance| {
                instance.state_blob.starts_with(b"PFT1")
                    && instance.state_blob.get(4..8).is_some_and(|raw| {
                        u32::from_le_bytes(raw.try_into().unwrap()) == target.raw()
                    })
            });
            if !pending {
                return Ok(Vec::new());
            }
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.state_blob.clear();
            }
            let linked = tx.state().players.get(target).is_some_and(|role| {
                role.marks
                    .iter()
                    .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
            });
            if linked {
                Ok(Vec::new())
            } else {
                add_mark(tx, target, crate::content::MARK_CHAIN, 1)
            }
        }
        5 => {
            let mut state = potyanhui_state(tx, skill);
            state.revealed.clear();
            state.discarded.clear();
            state.armed = false;
            set_potyanhui_state(tx, skill, &state);
            Ok(Vec::new())
        }
        6 => {
            if tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Prepare
                || tx
                    .state()
                    .skills
                    .get(skill)
                    .is_none_or(|instance| instance.lifetime_fire_count > 0)
            {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                vec![1, 2],
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![POTZHOUYU_CUSTOM_CHOICE, 8, player.raw(), skill.raw()],
            )
        }
        7 => {
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.content_flags &= !1;
            }
            Ok(Vec::new())
        }
        8 => {
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|role| role.alive)
            {
                return Ok(Vec::new());
            }
            let color_mask = card.raw() as u8;
            let matching: Vec<_> = tx
                .state()
                .hand_cards(target)
                .into_iter()
                .filter(|candidate| {
                    crate::query::effective_card_color_mask(tx.state(), target, *candidate)
                        & color_mask
                        != 0
                })
                .collect();
            let mut options = Vec::new();
            if !matching.is_empty() && potzhouyu_branch_allowed(tx, skill, 1) {
                options.push(1);
            }
            if potzhouyu_branch_allowed(tx, skill, 2) {
                options.push(2);
            }
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                options,
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    POTZHOUYU_CUSTOM_CHOICE,
                    2,
                    player.raw(),
                    skill.raw(),
                    target.raw(),
                    u32::from(color_mask),
                ],
            )
        }
        _ => Err(EffectError::ContentFault("unknown 势周瑜 action".into())),
    }
}
