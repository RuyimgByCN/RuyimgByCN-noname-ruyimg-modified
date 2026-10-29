//! Effect primitives for `refresh_skill` and related helpers.

use super::*;

/// `effect refresh_skill(self_skill)` — upstream `player.refreshSkill()`.
///
/// Resets one skill instance's per-turn / per-phase usage and trigger counters (mirroring
/// `reset_turn_state` scoped to a single instance) so a locked clan skill such as 中流
/// (clanzhongliu) can fire/activate again within the same turn. Content-owned persistent
/// state (`lifetime_fire_count`, `mark`, `content_flags`, `view_as_*`) is left untouched.
pub(crate) fn refresh_skill(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
) -> Result<Vec<engine_api::DomainEvent>, EffectError> {
    let changed = tx.state_mut().skills.refresh_skill(skill);
    if !changed {
        return Err(EffectError::Illegal("refresh_skill: bad skill".into()));
    }
    // Counter writes can re-enable a previously-disabled skill; rebuild derived indexes so
    // the next dispatch observes the refreshed state (matches reset_turn_state's epilogue).
    tx.state_mut().rebuild_derived_indexes();
    Ok(Vec::new())
}

/// Resolve a content-tag value from the skill instance id the VM pushed. The VM pushes the
/// current skill *instance*; the stable tag convention (see `SHEQUAN_CARD_TAG`/`POTFUJI_CARD_TAG`)
/// uses the skill *definition* id so multiple instances across players share one tag namespace.
/// If the id is not a live skill instance (e.g. content passed a literal def id), use it as-is.
pub(crate) fn resolve_tag(tx: &Transaction, tag: u32) -> u32 {
    if let Some(inst) = tx.state().skills.get(engine_api::SkillId::new(tag)) {
        inst.def.raw()
    } else {
        tag
    }
}

/// `effect tag_event_cards(self_skill)` — upstream `player.addGaintag(cards, tag)`.
///
/// Attaches the owning skill's definition id as a persistent content tag to each event card.
pub(crate) fn tag_cards(
    tx: &mut Transaction,
    cards: Vec<engine_api::CardId>,
    tag: u32,
) -> Result<Vec<engine_api::DomainEvent>, EffectError> {
    let tag = resolve_tag(tx, tag);
    if tag == 0 {
        return Err(EffectError::Illegal("tag_card: zero tag".into()));
    }
    for card in cards {
        tx.state_mut().add_content_card_tag(card, tag);
    }
    Ok(Vec::new())
}

/// `effect untag_event_cards(self_skill)` — upstream `player.removeGaintag(tag, cards)`.
pub(crate) fn untag_cards(
    tx: &mut Transaction,
    cards: Vec<engine_api::CardId>,
    tag: u32,
) -> Result<Vec<engine_api::DomainEvent>, EffectError> {
    let tag = resolve_tag(tx, tag);
    for card in cards {
        tx.state_mut().remove_content_card_tag(card, tag);
    }
    Ok(Vec::new())
}

/// `effect storage_set(self_skill, key, value)` — upstream
/// `player.storage.<skill>[key] = value`. Writes one key into the skill instance's keyed
/// content storage map. Returns an error if the skill instance does not exist.
pub(crate) fn storage_set(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    key: u32,
    value: u32,
) -> Result<Vec<engine_api::DomainEvent>, EffectError> {
    if !tx.state_mut().skills.storage_set(skill, key, value) {
        return Err(EffectError::Illegal("storage_set: bad skill".into()));
    }
    Ok(Vec::new())
}

/// `effect draw_all_event_targets(count)` — upstream `game.asyncDraw([player, ...targets])`.
/// Draws `count` cards to each listed player (event player + declared targets).
pub(crate) fn draw_all_event_targets(
    tx: &mut Transaction,
    players: Vec<engine_api::PlayerId>,
    count: u32,
) -> Result<Vec<engine_api::DomainEvent>, EffectError> {
    let mut events = Vec::new();
    for player in players {
        if player == engine_api::PlayerId::NONE {
            continue;
        }
        events.extend(draw_cards(tx, player, count)?);
    }
    Ok(events)
}

pub(crate) fn increase_bound_damage_by_one(
    tx: &mut Transaction,
    damage_id: u32,
) -> Result<(u32, u32), EffectError> {
    increase_bound_damage(tx, damage_id, 1)
}

pub(crate) fn increase_bound_damage(
    tx: &mut Transaction,
    damage_id: u32,
    delta: u32,
) -> Result<(u32, u32), EffectError> {
    if damage_id == 0 || tx.state().resolution.current_damage_id != damage_id {
        return Err(EffectError::Illegal(
            "damage increase context changed".into(),
        ));
    }
    match tx
        .state_mut()
        .resolution
        .deferred_tail_effects
        .iter_mut()
        .find(|request| {
            matches!(
                request,
                EffectRequest::CommitDamage { damage_id: saved, .. }
                    | EffectRequest::CommitDamageBanActiveIfDying {
                        damage_id: saved,
                        ..
                    } if *saved == damage_id
            )
        }) {
        Some(request) => {
            let (amount, base_amount) = match request {
                EffectRequest::CommitDamage {
                    amount,
                    base_amount,
                    ..
                }
                | EffectRequest::CommitDamageBanActiveIfDying {
                    amount,
                    base_amount,
                    ..
                } => (amount, base_amount),
                _ => unreachable!(),
            };
            let before = *amount;
            let after = before.saturating_add(delta);
            *amount = after;
            *base_amount = base_amount.saturating_add(delta);
            Ok((before, after))
        }
        // 批 282：触发执行期 CommitDamage 尚未入列，增量暂存 deferral 时套用。
        None => {
            let delta_i = i32::try_from(delta).unwrap_or(i32::MAX);
            tx.state_mut()
                .resolution
                .pending_bound_damage_deltas
                .push((damage_id, delta_i));
            Ok((0, 0))
        }
    }
}

pub(crate) fn set_bound_damage_nature(
    tx: &mut Transaction,
    damage_id: u32,
    nature: u8,
) -> Result<(), EffectError> {
    if damage_id == 0 || tx.state().resolution.current_damage_id != damage_id {
        return Err(EffectError::Illegal("damage nature context changed".into()));
    }
    let request = tx
        .state_mut()
        .resolution
        .deferred_tail_effects
        .iter_mut()
        .find(|request| {
            matches!(
                request,
                EffectRequest::CommitDamage { damage_id: saved, .. }
                    | EffectRequest::CommitDamageBanActiveIfDying {
                        damage_id: saved,
                        ..
                    } if *saved == damage_id
            )
        })
        .ok_or_else(|| EffectError::EngineFault("bound damage commit is missing".into()))?;
    match request {
        EffectRequest::CommitDamage { nature: saved, .. }
        | EffectRequest::CommitDamageBanActiveIfDying { nature: saved, .. } => *saved = nature,
        _ => unreachable!(),
    }
    Ok(())
}

pub(crate) fn bound_damage_source_matches(
    tx: &Transaction,
    damage_id: u32,
    player: PlayerId,
) -> bool {
    damage_id != 0
        && tx.state().resolution.current_damage_id == damage_id
        && tx
            .state()
            .resolution
            .deferred_tail_effects
            .iter()
            .any(|request| {
                matches!(
                    request,
                    EffectRequest::CommitDamage {
                        damage_id: saved,
                        source,
                        is_lose_hp: false,
                        ..
                    } if *saved == damage_id && *source == player
                ) || matches!(
                    request,
                    EffectRequest::CommitDamageBanActiveIfDying {
                        damage_id: saved,
                        source,
                        ..
                    } if *saved == damage_id && *source == player
                )
            })
}

pub(crate) fn choose_prevent_damage_or_lose_max_hp_increase_damage(
    tx: &mut Transaction,
    player: PlayerId,
    damage_id: u32,
    amount: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if amount == 0 || !bound_damage_source_matches(tx, damage_id, player) {
        return Err(EffectError::ContentFault(
            "damage branch used outside its damage_before event".into(),
        ));
    }
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        vec![1, 2],
        ChoiceKind::Custom,
        ContinuationKind::ChoosePreventDamageOrLoseMaxHpIncreaseDamage,
        vec![player.raw(), damage_id, amount, skill.raw()],
    )
}

pub(crate) fn discard_cards_from_exact_group(
    tx: &Transaction,
    cards: impl IntoIterator<Item = CardId>,
) -> Vec<CardId> {
    let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
    let mut seen = std::collections::BTreeSet::new();
    cards
        .into_iter()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|state| Some(state.zone) == discard)
        })
        .filter(|card| seen.insert(card.raw()))
        .collect()
}

pub(crate) fn choose_gain_resolved_sha_or_shan_then_optional_use_sha(
    tx: &mut Transaction,
    player: PlayerId,
    sha_card: CardId,
    shan_cards: Vec<CardId>,
    excluded_recipients: Vec<PlayerId>,
    skill: engine_api::SkillId,
    give_all_with_color_rewards: bool,
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
    let sha = discard_cards_from_exact_group(tx, [sha_card])
        .into_iter()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|card| card.def != CardDefId::NONE)
        })
        .collect::<Vec<_>>();
    let shan = discard_cards_from_exact_group(tx, shan_cards);
    if sha.is_empty() && shan.is_empty() {
        return Ok(Vec::new());
    }
    let mut excluded = excluded_recipients;
    excluded.push(player);
    excluded.sort_by_key(|target| target.raw());
    excluded.dedup();
    let recipients: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| !excluded.contains(target))
        .collect();
    if recipients.is_empty() {
        return Ok(Vec::new());
    }
    let mut payload = vec![
        u32::from(give_all_with_color_rewards) * 4,
        player.raw(),
        skill.raw(),
        sha.len() as u32,
    ];
    payload.extend(sha.iter().map(|card| card.raw()));
    payload.push(shan.len() as u32);
    payload.extend(shan.iter().map(|card| card.raw()));
    payload.push(excluded.len() as u32);
    payload.extend(excluded.iter().map(|target| target.raw()));
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        recipients.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseGainResolvedShaOrShanThenOptionalUseSha,
        payload,
    )
}

pub(crate) fn choose_use_sha_after_resolved_gift(
    tx: &mut Transaction,
    owner: PlayerId,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(owner)
        .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive && !participant.dying)
    {
        return Ok(Vec::new());
    }
    let cards: Vec<_> = crate::command::play::prompted_sha_source_cards(tx.state(), player)
        .into_iter()
        .filter(|card| {
            !crate::command::play::prompted_sha_targets(tx, owner, player, *card).is_empty()
        })
        .collect();
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        0,
        1,
        true,
        cards.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseGainResolvedShaOrShanThenOptionalUseSha,
        vec![2, owner.raw(), player.raw(), skill.raw()],
    )
}

pub(crate) fn discarded_black_basic_is_eligible(
    tx: &Transaction,
    player: PlayerId,
    card: CardId,
) -> bool {
    let Some(discard) = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE) else {
        return false;
    };
    tx.state().cards.get(card).is_some_and(|card_state| {
        card_state.zone == discard
            && card_state.owner.is_none()
            && crate::query::effective_card_kind(tx.state(), player, card_state).is_basic()
            && crate::query::effective_card_color_mask(tx.state(), player, card) == 0b10
            && !crate::command::play::view_as_bingliang_targets(tx, player, card).is_empty()
    })
}

pub(crate) fn choose_discarded_black_basic_as_bingliang(
    tx: &mut Transaction,
    player: PlayerId,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Err(EffectError::ContentFault(
            "discard-to-Bingliang choice requires its originating skill instance".into(),
        ));
    }
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let mut seen = std::collections::BTreeSet::new();
    let cards: Vec<_> = cards
        .into_iter()
        .filter(|card| seen.insert(card.raw()))
        .filter(|card| discarded_black_basic_is_eligible(tx, player, *card))
        .collect();
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let mut payload = vec![0, player.raw(), skill.raw(), cards.len() as u32];
    payload.extend(cards.iter().map(|card| card.raw()));
    open_choice(
        tx,
        player,
        0,
        1,
        true,
        cards.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseDiscardedBlackBasicAsBingliang,
        payload,
    )
}

pub(crate) fn player_in_attack_range(tx: &Transaction, player: PlayerId, target: PlayerId) -> bool {
    let distance = crate::query::query(
        tx.state(),
        &rule_api::QueryRequest::Distance {
            from: player,
            to: target,
        },
        &mut crate::query::QueryContext::new(),
    );
    let range = crate::query::query(
        tx.state(),
        &rule_api::QueryRequest::AttackRange { player },
        &mut crate::query::QueryContext::new(),
    );
    matches!(
        (distance, range),
        (rule_api::QueryResponse::Int(distance), rule_api::QueryResponse::Int(range))
            if distance <= range
    )
}

pub(crate) fn choose_discard_judge_use_virtual_sha_draw_if_no_damage(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Err(EffectError::ContentFault(
            "judge-discard virtual-Sha choice requires its originating skill instance".into(),
        ));
    }
    if player == target
        || ![player, target].into_iter().all(|participant| {
            tx.state()
                .players
                .get(participant)
                .is_some_and(|state| state.alive)
        })
        || !player_in_attack_range(tx, player, target)
    {
        return Ok(Vec::new());
    }
    let options = crate::query::discardable_target_cards(
        tx.state(),
        player,
        target,
        rule_api::TargetCardScope::Judge,
    );
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        0,
        1,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseDiscardJudgeUseVirtualShaDrawIfNoDamage,
        vec![player.raw(), target.raw(), skill.raw()],
    )
}

pub(crate) fn choose_discard_one_then_gain_target_equipment(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    slot: u8,
) -> Result<Vec<DomainEvent>, EffectError> {
    if EquipSlot::from_tag(slot).is_none() {
        return Err(EffectError::ContentFault(
            "invalid equipment slot for discard-then-gain".into(),
        ));
    }
    let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        0,
        1,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseDiscardOneThenGainTargetEquipment,
        vec![player.raw(), target.raw(), u32::from(slot)],
    )
}

pub(crate) fn gain_target_equipment(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    slot: u8,
) -> Result<Vec<DomainEvent>, EffectError> {
    let slot = EquipSlot::from_tag(slot)
        .ok_or_else(|| EffectError::ContentFault("invalid equipment slot to gain".into()))?;
    let Some(from) = tx.state().zones.find(ZoneKind::Equip, target) else {
        return Ok(Vec::new());
    };
    let cards: Vec<_> = tx
        .state()
        .zones
        .get(from)
        .map(|zone| {
            zone.cards
                .iter()
                .copied()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .and_then(|card| card.kind.equip_slot())
                        == Some(slot)
                })
                .collect()
        })
        .unwrap_or_default();
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("equipment recipient has no hand zone".into()))?;
    give_cards(tx, cards, to, from)
}

pub(crate) fn discard_all_target_equipment(
    tx: &mut Transaction,
    actor: PlayerId,
    target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if ![actor, target].into_iter().all(|player| {
        tx.state()
            .players
            .get(player)
            .is_some_and(|state| state.alive)
    }) {
        return Ok(Vec::new());
    }
    let cards = crate::query::discardable_equipment_cards(tx.state(), actor, target);
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    discard_target_cards(tx, actor, target, cards, rule_api::TargetCardScope::Equip)
}

pub(crate) fn adjust_hand_to(
    tx: &mut Transaction,
    player: PlayerId,
    count: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let hand = tx.state().hand_cards(player);
    let target = usize::try_from(count).unwrap_or(usize::MAX);
    if hand.len() < target {
        return draw_cards(tx, player, count.saturating_sub(hand.len() as u32));
    }
    let need = hand.len().saturating_sub(target);
    if need == 0 {
        return Ok(Vec::new());
    }
    let need = u8::try_from(need)
        .map_err(|_| EffectError::Illegal("adjust-hand discard exceeds choice limit".into()))?;
    open_choice(
        tx,
        player,
        need,
        need,
        false,
        hand.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::AdjustHandTo,
        vec![player.raw(), u32::from(need)],
    )
}

pub(crate) fn choose_draw_or_recover(
    tx: &mut Transaction,
    player: PlayerId,
    draw_count: u32,
    recover_count: u32,
    allow_cancel: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    let participant = tx
        .state()
        .players
        .get(player)
        .ok_or_else(|| EffectError::Illegal("bad draw-or-recover player".into()))?;
    if !participant.alive {
        return Ok(Vec::new());
    }
    if (participant.hp >= participant.max_hp as i32 || recover_count == 0) && allow_cancel {
        return open_choice(
            tx,
            player,
            1,
            1,
            true,
            vec![1],
            ChoiceKind::Custom,
            ContinuationKind::ChooseDrawOrRecover,
            vec![player.raw(), draw_count, recover_count, 1],
        );
    }
    if participant.hp >= participant.max_hp as i32 || recover_count == 0 {
        return draw_cards(tx, player, draw_count);
    }
    if draw_count == 0 {
        return recover_hp(tx, player, player, recover_count);
    }
    open_choice(
        tx,
        player,
        1,
        1,
        allow_cancel,
        vec![1, 2],
        ChoiceKind::Custom,
        ContinuationKind::ChooseDrawOrRecover,
        vec![
            player.raw(),
            draw_count,
            recover_count,
            u32::from(allow_cancel),
        ],
    )
}

pub(crate) fn choose_recover_target(
    tx: &mut Transaction,
    chooser: PlayerId,
    target: PlayerId,
    amount: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let chooser_alive = tx
        .state()
        .players
        .get(chooser)
        .is_some_and(|player| player.alive);
    let target_wounded = tx
        .state()
        .players
        .get(target)
        .is_some_and(|player| player.alive && player.hp < player.max_hp as i32);
    if amount == 0 || !chooser_alive || !target_wounded {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        chooser,
        1,
        1,
        true,
        vec![2],
        ChoiceKind::Custom,
        ContinuationKind::ChooseDrawOrRecover,
        vec![chooser.raw(), 0, amount, 1, target.raw()],
    )
}

pub(crate) fn choose_other_draw_by_hand_count(
    tx: &mut Transaction,
    player: PlayerId,
    normal_count: u32,
    empty_count: u32,
    times: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if times == 0
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
        .filter(|&target| target != player)
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
        targets.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseOtherDrawByHandCount,
        vec![player.raw(), normal_count, empty_count, times, skill.raw()],
    )
}

pub(crate) fn choose_other_draw_or_recover(
    tx: &mut Transaction,
    player: PlayerId,
    draw_count: u32,
    recover_count: u32,
    times: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if times == 0
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
        true,
        targets,
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseOtherDrawOrRecover,
        vec![
            0,
            player.raw(),
            draw_count,
            recover_count,
            times,
            skill.raw(),
        ],
    )
}
