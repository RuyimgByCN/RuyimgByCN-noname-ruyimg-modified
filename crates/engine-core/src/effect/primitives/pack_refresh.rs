//! Effect primitive implementations — refresh cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

/// 迂志 storage keys (per-skill keyed content storage).
const YUZHI_KEY_DRAWN_ROUND: u32 = 1;
/// Previous round's 迂志 draw count.
const YUZHI_KEY_DRAWN_LAST: u32 = 2;
/// This round's revealed-card name length mark.
const YUZHI_KEY_MARK: u32 = 3;
/// 保族 (clanbaozu) skill definition id — the skill 迂志 can remove.
const YUZHI_CLAN_BAOZU: u32 = 3_600_751_244;

/// 惬诣 per-suit first-use storage keys (suit tag offset).
const QIEYI_SUIT_KEY_BASE: u32 = 10;

/// 惬诣 (clanqieyi) skill definition id — 谏直 counts its fires this turn.
const JIANZHI_CLANQIEYI: u32 = 3_347_941_832;

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
fn resolve_tag(tx: &Transaction, tag: u32) -> u32 {
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

pub(crate) fn discarded_black_basic_is_eligible(tx: &Transaction, player: PlayerId, card: CardId) -> bool {
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

pub(crate) fn choose_other_recover(
    tx: &mut Transaction,
    player: PlayerId,
    amount: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if amount == 0
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
        false,
        targets,
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseOtherDrawOrRecover,
        vec![2, player.raw(), amount],
    )
}

pub(crate) fn choose_discard_hand_equip_then_remove_self_skill(
    tx: &mut Transaction,
    player: PlayerId,
    count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Err(EffectError::Illegal(
            "dynamic discard requires the owner's enabled originating skill".into(),
        ));
    }
    let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
    let required = usize::try_from(count)
        .unwrap_or(usize::MAX)
        .min(options.len());
    if required == 0 {
        return remove_skill(tx, skill);
    }
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::RemoveSkill { skill });
    if required == options.len() {
        return discard_cards(tx, player, options);
    }
    let required = u8::try_from(required)
        .map_err(|_| EffectError::ContentFault("dynamic discard count exceeds 255".into()))?;
    let mut payload = vec![
        player.raw(),
        skill.raw(),
        u32::from(required),
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
        ContinuationKind::ChooseDiscardHandEquipThenRemoveSelfSkill,
        payload,
    )
}

pub(crate) fn choose_discard_hand_equip_then_recover(
    tx: &mut Transaction,
    player: PlayerId,
    count: u32,
    amount: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if count == 0 || !originating_skill_is_enabled(tx, player, skill) {
        return Err(EffectError::Illegal(
            "discard-and-recover requires a positive count and the owner's enabled skill".into(),
        ));
    }
    let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
    let required = usize::try_from(count).unwrap_or(usize::MAX);
    if options.len() < required {
        return Ok(Vec::new());
    }
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::RecoverHp {
            source: player,
            player,
            amount,
        });
    if required == options.len() {
        return discard_cards(tx, player, options);
    }
    let required = u8::try_from(required)
        .map_err(|_| EffectError::ContentFault("discard-and-recover count exceeds 255".into()))?;
    let mut payload = vec![
        player.raw(),
        skill.raw(),
        u32::from(required),
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
        ContinuationKind::ChooseDiscardHandEquipThenRemoveSelfSkill,
        payload,
    )
}

pub(crate) fn choose_discard_hand_equip_then_recover_target(
    tx: &mut Transaction,
    chooser: PlayerId,
    target: PlayerId,
    count: u32,
    amount: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if count == 0 || amount == 0 || !originating_skill_is_enabled(tx, chooser, skill) {
        return Err(EffectError::Illegal(
            "optional discard-and-recover requires positive values and the chooser's enabled skill"
                .into(),
        ));
    }
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|player| player.alive && player.hp < player.max_hp as i32)
    {
        return Ok(Vec::new());
    }
    let options = crate::query::discardable_hand_equip_cards(tx.state(), chooser);
    let required = usize::try_from(count).unwrap_or(usize::MAX);
    if options.len() < required {
        return Ok(Vec::new());
    }
    let required = u8::try_from(required).map_err(|_| {
        EffectError::ContentFault("optional discard-and-recover count exceeds 255".into())
    })?;
    let mut payload = vec![
        chooser.raw(),
        skill.raw(),
        u32::from(required),
        options.len().min(u32::MAX as usize) as u32,
    ];
    payload.extend(options.iter().map(|card| card.raw()));
    payload.extend([target.raw(), amount]);
    open_choice(
        tx,
        chooser,
        required,
        required,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseDiscardHandEquipThenRemoveSelfSkill,
        payload,
    )
}

pub(crate) fn choose_target_draw(
    tx: &mut Transaction,
    player: PlayerId,
    count: u32,
    scope: DrawTargetScope,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    choose_targets_draw(tx, player, count, 1, scope, skill)
}

pub(crate) fn choose_target_draw_to_max_hp_cap(
    tx: &mut Transaction,
    player: PlayerId,
    cap: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if cap == 0
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
        false,
        targets,
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseTargetDraw,
        vec![player.raw(), cap, skill.raw()],
    )
}

pub(crate) fn choose_target_draw_then_owner_if_below_max_hp(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    count: u32,
    skill: engine_api::SkillId,
    ready_to_check: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if count == 0 {
        return Err(EffectError::ContentFault(
            "target draw count must be positive".into(),
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
    if target.is_none() {
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
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            targets,
            ChoiceKind::SelectTargets,
            ContinuationKind::ChooseTargetDrawThenOwnerIfBelowMaxHp,
            vec![player.raw(), count, skill.raw()],
        );
    }
    let target_exists = tx.state().players.get(target).is_some();
    let target_alive = tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive);
    if !target_exists || (!ready_to_check && !target_alive) {
        return if ready_to_check {
            Ok(Vec::new())
        } else {
            Err(EffectError::Illegal(
                "selected draw target is no longer alive".into(),
            ))
        };
    }
    if !ready_to_check {
        let events = draw_cards(tx, target, count)?;
        tx.state_mut().resolution.deferred_effects.insert(
            0,
            EffectRequest::ChooseTargetDrawThenOwnerIfBelowMaxHp {
                player,
                target,
                count,
                skill,
                ready_to_check: true,
            },
        );
        return Ok(events);
    }
    let below_max_hp = tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| tx.state().hand_count(target) < participant.max_hp);
    if below_max_hp {
        draw_cards(tx, player, 1)
    } else {
        Ok(Vec::new())
    }
}

pub(crate) fn choose_target_draw_max_hp_cap_then_discard_excess(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    cap: u32,
    skill: engine_api::SkillId,
    ready_to_discard: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if cap == 0 {
        return Err(EffectError::ContentFault(
            "maximum-HP draw/discard cap must be positive".into(),
        ));
    }
    if !originating_skill_is_enabled(tx, player, skill) || tx.state().players.get(player).is_none()
    {
        return Ok(Vec::new());
    }
    if target.is_none() {
        let targets: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|target| {
                tx.state()
                    .players
                    .get(*target)
                    .is_some_and(|participant| participant.max_hp > 0)
            })
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
            ContinuationKind::ChooseTargetDrawMaxHpCapThenDiscardExcess,
            vec![player.raw(), cap, skill.raw()],
        );
    }
    let participant = tx.state().players.get(target);
    let target_alive_with_positive_max_hp =
        participant.is_some_and(|participant| participant.alive && participant.max_hp > 0);
    if !ready_to_discard && !target_alive_with_positive_max_hp {
        return Err(EffectError::Illegal(
            "selected capped draw target is no longer legal".into(),
        ));
    }
    if ready_to_discard {
        let Some(participant) = participant.filter(|participant| participant.alive) else {
            return Ok(Vec::new());
        };
        let limit = participant.max_hp.min(cap);
        if tx.state().hand_count(target) > limit {
            return adjust_hand_to(tx, target, limit);
        }
        return Ok(Vec::new());
    }
    let count = participant
        .expect("validated capped draw target")
        .max_hp
        .min(cap);
    let events = draw_cards(tx, target, count)?;
    tx.state_mut().resolution.deferred_effects.insert(
        0,
        EffectRequest::ChooseTargetDrawMaxHpCapThenDiscardExcess {
            player,
            target,
            cap,
            skill,
            ready_to_discard: true,
        },
    );
    Ok(events)
}

/// Custom-continuation marker for ABI 455 `choose_target_draw_to_max_hp_cap_then_add_skill`.
pub(crate) const CHOOSE_TARGET_DRAW_MAX_HP_CAP_THEN_ADD_SKILL_CUSTOM_CHOICE: u32 = 0xDCA5_4550;

pub(crate) fn choose_target_draw_max_hp_cap_then_add_skill(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    cap: u32,
    skill: engine_api::SkillId,
    add_def: engine_api::SkillDefId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if cap == 0
        || add_def.raw() == 0
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    if target.is_none() {
        let targets: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|target| {
                tx.state()
                    .players
                    .get(*target)
                    .is_some_and(|target| target.max_hp > 0)
            })
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
            ContinuationKind::Custom,
            vec![
                CHOOSE_TARGET_DRAW_MAX_HP_CAP_THEN_ADD_SKILL_CUSTOM_CHOICE,
                player.raw(),
                cap,
                add_def.raw(),
            ],
        );
    }
    let Some(participant) = tx
        .state()
        .players
        .get(target)
        .filter(|participant| participant.alive)
    else {
        return Ok(Vec::new());
    };
    let count = participant
        .max_hp
        .min(cap)
        .saturating_sub(tx.state().hand_count(target));
    let mut events = super::run_effects(
        tx,
        vec![EffectRequest::DrawCardsFromSkill {
            player: target,
            count,
            skill,
        }],
    )?;
    events.extend(super::run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: target,
            skill_def: add_def,
        }],
    )?);
    Ok(events)
}

pub(crate) fn give_selected_then_target_choose_discard_same_suit_or_lose_hp(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    suit: engine_api::Suit,
    skill: engine_api::SkillId,
    ready_to_choose: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if ready_to_choose {
        if !tx
            .state()
            .players
            .get(target)
            .is_some_and(|state| state.alive)
        {
            return Ok(Vec::new());
        }
        if tx.state().hand_count(target) == 0 {
            return super::run_effects(
                tx,
                vec![EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target,
                    amount: 1,
                    is_lose_hp: true,
                    nature: 0,
                }],
            )
            .map(|_| Vec::new());
        }
        return open_choice(
            tx,
            target,
            1,
            1,
            false,
            vec![0, 1],
            ChoiceKind::Custom,
            ContinuationKind::GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp,
            vec![target.raw(), crate::command::judge::suit_tag(suit)],
        );
    }
    if !originating_skill_is_enabled(tx, player, skill)
        || player == target
        || ![player, target].into_iter().all(|participant| {
            tx.state()
                .players
                .get(participant)
                .is_some_and(|state| state.alive)
        })
    {
        return Err(EffectError::Illegal(
            "selected-card gift requires distinct living participants".into(),
        ));
    }
    let from = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("missing source hand zone".into()))?;
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("missing recipient hand zone".into()))?;
    let captured_suit = tx
        .state()
        .cards
        .get(card)
        .filter(|state| state.owner == player && state.zone == from)
        .map(|state| state.suit)
        .ok_or_else(|| {
            EffectError::Illegal("selected gift card is no longer in the owner's hand".into())
        })?;
    super::run_effects(
        tx,
        vec![
            EffectRequest::RevealCards {
                cards: vec![card],
                to: Vec::new(),
            },
            EffectRequest::GiveCards {
                cards: vec![card],
                to,
                from,
            },
            EffectRequest::GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp {
                player,
                target,
                card,
                suit: captured_suit,
                skill,
                ready_to_choose: true,
            },
        ],
    )
    .map(|_| Vec::new())
}

pub(crate) fn target_choose_suit_gain_owner_hand_damage_if_mismatch(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || player == target
        || ![player, target].into_iter().all(|participant| {
            tx.state()
                .players
                .get(participant)
                .is_some_and(|state| state.alive)
        })
        || tx.state().hand_count(player) == 0
    {
        return Err(EffectError::Illegal(
            "suit guess requires distinct living participants and an owner hand card".into(),
        ));
    }
    open_choice(
        tx,
        target,
        1,
        1,
        false,
        vec![2, 4, 3, 1],
        ChoiceKind::Custom,
        ContinuationKind::FanjianChooseSuitOrHand,
        vec![0, player.raw(), target.raw(), skill.raw()],
    )
}

pub(crate) fn choose_other_draw_owner_lost_hp_turn_over(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
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
        false,
        targets,
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseOtherDrawOwnerLostHpTurnOver,
        vec![player.raw(), skill.raw()],
    )
}

pub(crate) fn choose_targets_draw(
    tx: &mut Transaction,
    player: PlayerId,
    count: u32,
    target_count: u8,
    scope: DrawTargetScope,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if count == 0
        || target_count == 0
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
        .filter(|&target| player_target_matches(tx.state(), player, target, scope))
        .collect();
    if targets.len() < usize::from(target_count) {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        target_count,
        target_count,
        false,
        targets.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseTargetDraw,
        if target_count == 1 {
            vec![player.raw(), count, u32::from(scope.tag()), skill.raw()]
        } else {
            vec![
                player.raw(),
                count,
                u32::from(target_count),
                u32::from(scope.tag()),
                skill.raw(),
            ]
        },
    )
}

pub(crate) fn choose_targets_damage(
    tx: &mut Transaction,
    player: PlayerId,
    min_count: u8,
    max_count: u8,
    scope: DrawTargetScope,
    amount: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if min_count == 0
        || min_count > max_count
        || amount == 0
        || !originating_skill_is_enabled(tx, player, skill)
    {
        return Err(EffectError::Illegal(
            "target-damage choice requires a valid range, positive damage, and enabled skill"
                .into(),
        ));
    }
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| player_target_matches(tx.state(), player, target, scope))
        .collect();
    if targets.len() < usize::from(min_count) {
        return Ok(Vec::new());
    }
    let available = targets.len().min(usize::from(u8::MAX)) as u8;
    open_choice(
        tx,
        player,
        min_count,
        max_count.min(available),
        false,
        targets.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseTargetsDamage,
        vec![
            player.raw(),
            u32::from(min_count),
            u32::from(max_count),
            u32::from(scope.tag()),
            amount,
            skill.raw(),
        ],
    )
}

pub(crate) fn choose_optional_targets_draw(
    tx: &mut Transaction,
    player: PlayerId,
    count: u32,
    min_count: u8,
    max_count: u8,
    scope: DrawTargetScope,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if count == 0
        || min_count == 0
        || min_count > max_count
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
        .filter(|&target| player_target_matches(tx.state(), player, target, scope))
        .collect();
    if targets.len() < usize::from(min_count) {
        return Ok(Vec::new());
    }
    let available = targets.len().min(usize::from(u8::MAX)) as u8;
    open_choice(
        tx,
        player,
        min_count,
        max_count.min(available),
        true,
        targets.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseTargetDraw,
        vec![
            player.raw(),
            count,
            u32::from(min_count),
            u32::from(max_count),
            u32::from(scope.tag()),
            skill.raw(),
        ],
    )
}

pub(crate) fn choose_repolu_targets_draw(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) || tx.state().players.get(player).is_none()
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
    let max_count = targets.len().min(usize::from(u8::MAX)) as u8;
    open_choice(
        tx,
        player,
        1,
        max_count,
        true,
        targets,
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseRepoluTargetsDraw,
        vec![player.raw(), skill.raw()],
    )
}

pub(crate) fn resolve_zhuiyi(
    tx: &mut Transaction,
    player: PlayerId,
    killer: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    // 追忆在拥有者阵亡时结算，只要求座位存在而非存活。
    if tx.state().players.get(player).is_none() {
        return Ok(Vec::new());
    }
    let others: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != player && target != killer)
        .collect();
    GuardedChoice::new(
        skill,
        Participants::owner_any_state(player),
        ContinuationKind::ResolveZhuiyi,
        vec![player.raw(), killer.raw(), skill.raw()],
    )
    .shape(ChoiceShape::one_optional())
    .open_targets(tx, others)
}

pub(crate) fn resolve_xinfu_guolun(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    player_card: CardId,
    target_card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let valid_context = originating_skill_is_enabled(tx, player, skill)
        && tx.state().turn.current == player
        && tx.state().turn.phase == Phase::Play
        && target != player
        && tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive);
    if !valid_context {
        return Err(EffectError::Illegal("Guolun context changed".into()));
    }
    if player_card.is_none() && target_card.is_none() {
        if tx.state().hand_count(player) == 0 {
            return Err(EffectError::Illegal("Guolun owner hand is empty".into()));
        }
        let options = tx.state().hand_cards(target);
        if options.is_empty() {
            return Err(EffectError::Illegal("Guolun target hand is empty".into()));
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            options.into_iter().map(CardId::raw).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::ResolveXinfuGuolun,
            vec![0, player.raw(), target.raw(), skill.raw()],
        );
    }
    if player_card.is_none() || target_card.is_none() {
        return Err(EffectError::EngineFault(
            "Guolun exchange requires both cards".into(),
        ));
    }

    let target_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::EngineFault("Guolun target hand zone missing".into()))?;
    let player_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("Guolun owner hand zone missing".into()))?;
    let target_card_state = tx
        .state()
        .cards
        .get(target_card)
        .filter(|card| card.owner == target && card.zone == target_hand)
        .cloned()
        .ok_or_else(|| EffectError::Illegal("Guolun target card moved".into()))?;
    let player_card_state = tx
        .state()
        .cards
        .get(player_card)
        .filter(|card| {
            card.owner == player
                && [ZoneKind::Hand, ZoneKind::Equip]
                    .into_iter()
                    .any(|kind| tx.state().zones.find(kind, player) == Some(card.zone))
        })
        .cloned()
        .ok_or_else(|| EffectError::Illegal("Guolun owner card moved".into()))?;
    let player_number = match crate::query::query(
        tx.state(),
        &QueryRequest::CardAttribute {
            card: player_card,
            attr: 1,
        },
        &mut crate::query::QueryContext::new(),
    ) {
        QueryResponse::Int(number) => number,
        _ => {
            return Err(EffectError::EngineFault(
                "Guolun owner card number query failed".into(),
            ));
        }
    };
    let target_number = match crate::query::query(
        tx.state(),
        &QueryRequest::CardAttribute {
            card: target_card,
            attr: 1,
        },
        &mut crate::query::QueryContext::new(),
    ) {
        QueryResponse::Int(number) => number,
        _ => {
            return Err(EffectError::EngineFault(
                "Guolun target card number query failed".into(),
            ));
        }
    };

    let mut events = move_cards(
        tx,
        vec![player_card],
        target_hand,
        Some(player_card_state.zone),
    )?;
    events.extend(move_cards(
        tx,
        vec![target_card],
        player_hand,
        Some(target_card_state.zone),
    )?);

    let drawer = if player_number < target_number {
        player
    } else if target_number < player_number {
        target
    } else {
        PlayerId::NONE
    };
    if drawer.is_some() {
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::DrawCardsFromSkill {
                player: drawer,
                count: 1,
                skill,
            });
    }
    Ok(events)
}

pub(crate) fn player_target_matches(
    state: &crate::state::GameState,
    chooser: PlayerId,
    target: PlayerId,
    scope: DrawTargetScope,
) -> bool {
    let Some(target_player) = state.players.get(target).filter(|player| player.alive) else {
        return false;
    };
    if !matches!(
        scope,
        DrawTargetScope::Any
            | DrawTargetScope::MinHand
            | DrawTargetScope::SameHp
            | DrawTargetScope::HandEmpty
            | DrawTargetScope::Wounded
    ) && target == chooser
    {
        return false;
    }
    let sex = state
        .content
        .character(target_player.character_def)
        .map(|character| character.sex.as_str())
        .unwrap_or("");
    match scope {
        DrawTargetScope::Other | DrawTargetScope::Any => true,
        DrawTargetScope::MaleOther => matches!(sex, "male" | "double"),
        DrawTargetScope::FemaleOther => matches!(sex, "female" | "double"),
        DrawTargetScope::MinHand => {
            let target_hand = state.hand_count(target);
            state
                .players
                .iter_live()
                .filter(|player| player.alive)
                .all(|player| target_hand <= state.hand_count(player.id))
        }
        DrawTargetScope::SameHp => state
            .players
            .get(chooser)
            .filter(|player| player.alive)
            .is_some_and(|player| target_player.hp == player.hp),
        DrawTargetScope::HandEmpty => state.hand_count(target) == 0,
        DrawTargetScope::Wounded => target_player.hp < target_player.max_hp as i32,
        DrawTargetScope::HandLteChooser => {
            state
                .players
                .get(chooser)
                .is_some_and(|player| player.alive)
                && state.hand_count(target) <= state.hand_count(chooser)
        }
    }
}

pub(crate) fn choose_discard_hand_add_skill_by_color(
    tx: &mut Transaction,
    player: PlayerId,
    red_skill_def: engine_api::SkillDefId,
    black_skill_def: engine_api::SkillDefId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
    let options: Vec<_> = crate::query::discardable_hand_equip_cards(tx.state(), player)
        .into_iter()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|card| card.zone == hand)
        })
        .collect();
    if options.is_empty() {
        return Ok(vec![]);
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::DiscardHandAddSkillByColor,
        vec![player.raw(), red_skill_def.raw(), black_skill_def.raw()],
    )
}

pub(crate) fn reveal_top_gain_non_basic_or_swap_basic(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|player| player.alive)
    {
        return Ok(Vec::new());
    }
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
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
        .and_then(|zone| zone.cards.last().copied())
    else {
        return Ok(Vec::new());
    };
    let is_basic = tx
        .state()
        .cards
        .get(card)
        .is_some_and(|card| card.kind.is_basic());
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    move_one_card(tx, card, draw, processing, PlayerId::NONE, true)?;
    let mut events = vec![
        DomainEvent::CardsMoved {
            cards: vec![card],
            from: draw,
            to: processing,
            reason: CardMoveReason::Unspecified,
        },
        DomainEvent::CardsRevealed {
            cards: vec![card],
            to: Vec::new(),
        },
    ];
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
    if !is_basic {
        events.extend(move_cards(tx, vec![card], hand, Some(processing))?);
        return Ok(events);
    }
    let options = tx.state().hand_cards(player);
    if options.is_empty() {
        let discard = tx
            .state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
        events.extend(move_cards(tx, vec![card], discard, Some(processing))?);
        return Ok(events);
    }
    events.extend(open_choice(
        tx,
        player,
        1,
        1,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::RevealTopGainNonBasicOrSwapBasic,
        vec![player.raw(), card.raw()],
    )?);
    Ok(events)
}

pub(crate) fn reveal_top_gain_if_color(
    tx: &mut Transaction,
    player: PlayerId,
    color_mask: u8,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !matches!(color_mask, 1 | 2) {
        return Err(EffectError::ContentFault(
            "top-card color must be red or black".into(),
        ));
    }
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|player| player.alive)
    {
        return Ok(Vec::new());
    }
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
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
        .and_then(|zone| zone.cards.last().copied())
    else {
        return Ok(Vec::new());
    };
    let matches_color = tx
        .state()
        .cards
        .get(card)
        .is_some_and(|card| crate::query::card_color_mask_for_suit(card.suit) == color_mask);
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    move_one_card(tx, card, draw, processing, PlayerId::NONE, true)?;
    let mut events = vec![
        DomainEvent::CardsMoved {
            cards: vec![card],
            from: draw,
            to: processing,
            reason: CardMoveReason::Unspecified,
        },
        DomainEvent::CardsRevealed {
            cards: vec![card],
            to: Vec::new(),
        },
    ];
    let destination = if matches_color {
        tx.state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::Illegal("no hand".into()))?
    } else {
        tx.state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?
    };
    events.extend(move_cards(tx, vec![card], destination, Some(processing))?);
    Ok(events)
}

pub(crate) fn choose_gain_top_cards_by_suit(
    tx: &mut Transaction,
    player: PlayerId,
    count: u8,
    suit: Suit,
) -> Result<Vec<DomainEvent>, EffectError> {
    if count == 0 || suit == Suit::None {
        return Err(EffectError::ContentFault(
            "top-card suit choice requires a positive count and concrete suit".into(),
        ));
    }
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|player| player.alive)
    {
        return Err(EffectError::Illegal(
            "top-card suit choice requires a living player".into(),
        ));
    }
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    let available = tx
        .state()
        .zones
        .get(draw)
        .map(|zone| zone.cards.len())
        .unwrap_or(0);
    if available < usize::from(count) {
        reshuffle_discard_into_draw(tx)?;
    }
    let zone = tx
        .state()
        .zones
        .get(draw)
        .ok_or_else(|| EffectError::EngineFault("draw zone missing".into()))?;
    let take = usize::from(count).min(zone.cards.len());
    if take == 0 {
        return Ok(Vec::new());
    }
    let top: Vec<CardId> = zone.cards[zone.cards.len() - take..].to_vec();
    let options: Vec<u32> = top
        .iter()
        .filter_map(|card| {
            tx.state()
                .cards
                .get(*card)
                .filter(|card| crate::query::effective_card_suit(tx.state(), player, card) == suit)
                .map(|card| card.id.raw())
        })
        .collect();
    let mut events = vec![DomainEvent::CardsRevealed {
        cards: top.clone(),
        to: vec![player],
    }];
    if options.is_empty() {
        return Ok(events);
    }
    let mut payload = vec![
        player.raw(),
        crate::command::judge::suit_tag(suit),
        take as u32,
    ];
    payload.extend(top.iter().map(|card| card.raw()));
    events.extend(open_choice(
        tx,
        player,
        1,
        options.len() as u8,
        true,
        options,
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseGainTopCardsBySuit,
        payload,
    )?);
    Ok(events)
}

pub(crate) fn discard_all_hand_and_equip(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let cards = crate::query::discardable_hand_equip_cards(tx.state(), player);
    if cards.is_empty() {
        return Ok(vec![]);
    }
    discard_cards(tx, player, cards)
}

pub(crate) fn discard_all_hand_equip_judge(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "dead or missing player zones cannot be discarded".into(),
        ));
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let batches: Vec<_> = [ZoneKind::Hand, ZoneKind::Equip, ZoneKind::Judge]
        .into_iter()
        .filter_map(|kind| {
            let zone = tx.state().zones.find(kind, player)?;
            let cards = tx.state().zones.get(zone)?.cards.clone();
            (!cards.is_empty()).then_some((zone, cards))
        })
        .collect();
    let mut events = Vec::new();
    for (from, cards) in batches {
        events.extend(move_cards_with_reason(
            tx,
            cards,
            discard,
            Some(from),
            CardMoveReason::Discard,
        )?);
    }
    Ok(events)
}

pub(crate) fn damage_then_discard_all_judge(
    tx: &mut Transaction,
    player: PlayerId,
    ready_to_discard: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(participant) = tx.state().players.get(player) else {
        return Err(EffectError::Illegal("missing player".into()));
    };
    if ready_to_discard {
        return discard_all_judge(tx, player);
    }
    if !participant.alive {
        return Err(EffectError::Illegal(
            "dead player cannot start self-damage".into(),
        ));
    }
    tx.state_mut().resolution.deferred_tail_effects.push(
        EffectRequest::DamageThenDiscardAllJudge {
            player,
            ready_to_discard: true,
        },
    );
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

pub(crate) fn replace_draw_gain_top_two_add_skills_if_different_colors(
    tx: &mut Transaction,
    player: PlayerId,
    first_skill_def: engine_api::SkillDefId,
    second_skill_def: engine_api::SkillDefId,
    stage: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if stage == 0 {
        if !originating_skill_is_enabled(tx, player, skill)
            || tx.state().turn.current != player
            || tx.state().turn.phase != Phase::Draw
            || tx.state().turn.draw_phase_is_fixed()
        {
            return Err(EffectError::Illegal(
                "invalid top-two draw replacement context".into(),
            ));
        }
        tx.state_mut().turn.set_draw_phase_fixed_count(0);
        let draw = tx
            .state()
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::EngineFault("draw pile missing".into()))?;
        if tx
            .state()
            .zones
            .get(draw)
            .map_or(0, |zone| zone.cards.len())
            < 2
        {
            reshuffle_discard_into_draw(tx)?;
        }
        let zone = tx
            .state()
            .zones
            .get(draw)
            .ok_or_else(|| EffectError::EngineFault("draw pile disappeared".into()))?;
        let take = 2.min(zone.cards.len());
        if take == 0 {
            return Ok(Vec::new());
        }
        let cards = zone.cards[zone.cards.len() - take..].to_vec();
        let processing = tx
            .state()
            .zones
            .find(ZoneKind::Processing, PlayerId::NONE)
            .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
        tx.state_mut().resolution.deferred_tail_effects.push(
            EffectRequest::ReplaceDrawGainTopTwoAddSkillsIfDifferentColors {
                player,
                first_skill_def,
                second_skill_def,
                stage: 1,
                cards: cards.clone(),
                skill,
            },
        );
        let mut events = move_cards(tx, cards.clone(), processing, Some(draw))?;
        events.push(DomainEvent::CardsRevealed {
            cards,
            to: Vec::new(),
        });
        return Ok(events);
    }
    if stage == 1 {
        let processing = tx
            .state()
            .zones
            .find(ZoneKind::Processing, PlayerId::NONE)
            .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
        if cards.is_empty()
            || cards.iter().any(|card| {
                !tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|card| card.zone == processing)
            })
        {
            return Err(EffectError::Illegal(
                "top-two revealed cards left processing".into(),
            ));
        }
        if cards.len() == 2
            && crate::query::effective_card_color_mask(tx.state(), player, cards[0])
                != crate::query::effective_card_color_mask(tx.state(), player, cards[1])
        {
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ReplaceDrawGainTopTwoAddSkillsIfDifferentColors {
                    player,
                    first_skill_def,
                    second_skill_def,
                    stage: 2,
                    cards: cards.clone(),
                    skill,
                },
            );
        }
        let hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::EngineFault("player hand missing".into()))?;
        return move_cards(tx, cards, hand, Some(processing));
    }
    if stage != 2 {
        return Err(EffectError::ContentFault(
            "bad top-two draw replacement stage".into(),
        ));
    }
    let mut events = add_skill_until_turn_end(tx, player, first_skill_def)?;
    events.extend(add_skill_until_turn_end(tx, player, second_skill_def)?);
    Ok(events)
}

pub(crate) fn resolve_new_reluoyi(
    tx: &mut Transaction,
    player: PlayerId,
    buff_skill_def: engine_api::SkillDefId,
    stage: u8,
    cards: Vec<CardId>,
    accepted: bool,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if buff_skill_def.is_none() {
        return Err(EffectError::ContentFault(
            "New Reluoyi requires a damage helper skill".into(),
        ));
    }
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
    if stage == 0 {
        if !originating_skill_is_enabled(tx, player, skill)
            || tx.state().turn.current != player
            || tx.state().turn.phase != Phase::Draw
            || tx.state().turn.draw_phase_is_fixed()
        {
            return Err(EffectError::Illegal(
                "invalid New Reluoyi draw replacement context".into(),
            ));
        }
        let draw = tx
            .state()
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::EngineFault("draw pile missing".into()))?;
        if tx
            .state()
            .zones
            .get(draw)
            .map_or(0, |zone| zone.cards.len())
            < 3
        {
            reshuffle_discard_into_draw(tx)?;
        }
        let zone = tx
            .state()
            .zones
            .get(draw)
            .ok_or_else(|| EffectError::EngineFault("draw pile disappeared".into()))?;
        let take = 3.min(zone.cards.len());
        if take == 0 {
            return Ok(Vec::new());
        }
        let cards = zone.cards[zone.cards.len() - take..].to_vec();
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::ResolveNewReluoyi {
                player,
                buff_skill_def,
                stage: 1,
                cards: cards.clone(),
                accepted: false,
                skill,
            });
        let mut events = move_cards(tx, cards.clone(), processing, Some(draw))?;
        events.push(DomainEvent::CardsRevealed {
            cards,
            to: Vec::new(),
        });
        return Ok(events);
    }
    if stage == 3 {
        return add_source_turn_bound_skill(tx, player, buff_skill_def, player);
    }
    if cards.is_empty()
        || cards.iter().any(|card| {
            !tx.state()
                .cards
                .get(*card)
                .is_some_and(|card| card.zone == processing)
        })
    {
        return Err(EffectError::Illegal(
            "New Reluoyi revealed cards left processing".into(),
        ));
    }
    if stage == 1 {
        let mut payload = vec![
            player.raw(),
            buff_skill_def.raw(),
            skill.raw(),
            cards.len() as u32,
        ];
        payload.extend(cards.iter().map(|card| card.raw()));
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![0, 1],
            ChoiceKind::Confirm,
            ContinuationKind::ResolveNewReluoyi,
            payload,
        );
    }
    if stage != 2 {
        return Err(EffectError::ContentFault(
            "bad New Reluoyi resolution stage".into(),
        ));
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
    if !accepted {
        return move_cards(tx, cards, discard, Some(processing));
    }

    tx.state_mut().turn.set_draw_phase_fixed_count(0);
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveNewReluoyi {
            player,
            buff_skill_def,
            stage: 3,
            cards: Vec::new(),
            accepted: true,
            skill,
        });
    let (gain, rest): (Vec<_>, Vec<_>) = cards.into_iter().partition(|card| {
        tx.state().cards.get(*card).is_some_and(|card| {
            card.kind.is_basic() || matches!(card.kind, CardKind::TrickJuedou | CardKind::Weapon)
        })
    });
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("player hand missing".into()))?;
    let mut events = Vec::new();
    if !gain.is_empty() {
        events.extend(move_cards(tx, gain, hand, Some(processing))?);
    }
    if !rest.is_empty() {
        events.extend(move_cards(tx, rest, discard, Some(processing))?);
    }
    Ok(events)
}

pub(crate) fn resolve_remoshi(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    helper_skill_def: engine_api::SkillDefId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let pile_skill_def = tx
        .state()
        .skills
        .get(skill)
        .filter(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        .map(|instance| instance.def.raw())
        .ok_or_else(|| EffectError::Illegal("invalid Remoshi skill context".into()))?;
    if !tx
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
    match action {
        0 => {
            if helper_skill_def.is_none() {
                return Err(EffectError::ContentFault(
                    "Remoshi requires its presentation helper".into(),
                ));
            }
            let source_cards: Vec<_> = tx
                .state()
                .resolution
                .sha_source_cards(player, card)
                .into_iter()
                .filter_map(|card| {
                    tx.state()
                        .cards
                        .get(card)
                        .filter(|card| card.def != engine_api::CardDefId::NONE)
                        .and_then(|card| {
                            tx.state()
                                .zones
                                .get(card.zone)
                                .filter(|zone| {
                                    matches!(
                                        zone.kind,
                                        ZoneKind::Processing | ZoneKind::DiscardPile
                                    )
                                })
                                .map(|_| (card.id, card.zone))
                        })
                })
                .collect();
            if source_cards.is_empty() {
                return Err(EffectError::Illegal(
                    "Remoshi Sha sources are no longer gainable".into(),
                ));
            }
            let has_armor_or_mount = tx
                .state()
                .zones
                .find(ZoneKind::Equip, target)
                .and_then(|zone| tx.state().zones.get(zone))
                .is_some_and(|zone| {
                    zone.cards.iter().any(|card| {
                        tx.state().cards.get(*card).is_some_and(|card| {
                            matches!(
                                card.kind,
                                CardKind::Armor | CardKind::HorseOff | CardKind::HorseDef
                            )
                        })
                    })
                });
            if !has_armor_or_mount {
                return Ok(Vec::new());
            }
            let mut events = add_skill(tx, target, helper_skill_def)?;
            let pile = ensure_skill_pile_zone(tx, target, pile_skill_def)?;
            for (card, from) in source_cards {
                events.extend(move_cards(tx, vec![card], pile, Some(from))?);
            }
            Ok(events)
        }
        1 => {
            let cards = tx.state().skill_pile_cards(target, pile_skill_def);
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let from = ensure_skill_pile_zone(tx, target, pile_skill_def)?;
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("Remoshi owner hand missing".into()))?;
            give_cards(tx, cards, to, from)
        }
        _ => Err(EffectError::ContentFault(
            "unknown Remoshi resolution action".into(),
        )),
    }
}

pub(crate) fn resolve_xinyaoming(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "invalid Xinyaoming skill context".into(),
        ));
    }
    let used_mask = tx
        .state()
        .skills
        .get(skill)
        .map_or(u8::MAX, |instance| instance.xinyaoming_used_mask());
    if target.is_none() {
        if action != 0 {
            return Err(EffectError::ContentFault(
                "Xinyaoming target choice requires action zero".into(),
            ));
        }
        let options: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|target| {
                !crate::query::xinyaoming_available_actions(tx.state(), player, *target, used_mask)
                    .is_empty()
            })
            .collect();
        if options.is_empty() {
            return Ok(Vec::new());
        }
        let mut payload = vec![0, player.raw(), skill.raw(), options.len() as u32];
        payload.extend(options.iter().map(|target| target.raw()));
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            options.into_iter().map(PlayerId::raw).collect(),
            ChoiceKind::SelectTargets,
            ContinuationKind::ResolveXinyaoming,
            payload,
        );
    }
    let actions = crate::query::xinyaoming_available_actions(tx.state(), player, target, used_mask);
    if action == 0 {
        return match actions.as_slice() {
            [] => Err(EffectError::Illegal(
                "Xinyaoming target has no available action".into(),
            )),
            [only] => resolve_xinyaoming(tx, player, target, *only, skill),
            _ => {
                let mut payload = vec![
                    1,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                    actions.len() as u32,
                ];
                payload.extend(actions.iter().map(|action| u32::from(*action)));
                open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    actions.into_iter().map(u32::from).collect(),
                    ChoiceKind::Custom,
                    ContinuationKind::ResolveXinyaoming,
                    payload,
                )
            }
        };
    }
    if !actions.contains(&action) {
        return Err(EffectError::Illegal(
            "Xinyaoming action is unavailable".into(),
        ));
    }
    tx.state_mut()
        .skills
        .get_mut(skill)
        .ok_or_else(|| EffectError::Illegal("Xinyaoming skill disappeared".into()))?
        .mark_xinyaoming_action(action);
    match action {
        crate::query::XINYAOMING_DRAW => draw_cards(tx, target, 1),
        crate::query::XINYAOMING_DISCARD_HAND => {
            choose_discard_target_card(tx, player, target, rule_api::TargetCardScope::Hand)
        }
        crate::query::XINYAOMING_DISCARD_DRAW => {
            let options = crate::query::discardable_target_cards(
                tx.state(),
                target,
                target,
                rule_api::TargetCardScope::HandEquip,
            );
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let max = options.len().min(2) as u8;
            let mut payload = vec![
                2,
                player.raw(),
                target.raw(),
                skill.raw(),
                options.len() as u32,
            ];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                target,
                1,
                max,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveXinyaoming,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault(
            "unknown Xinyaoming action".into(),
        )),
    }
}

pub(crate) fn rehuaiyi_eligible_colors(state: &crate::state::GameState, player: PlayerId) -> Vec<u8> {
    let hand = state.hand_cards(player);
    let discardable: std::collections::BTreeSet<_> =
        crate::query::discardable_hand_equip_cards(state, player)
            .into_iter()
            .collect();
    [0b01, 0b10]
        .into_iter()
        .filter(|color| {
            let matching: Vec<_> = hand
                .iter()
                .copied()
                .filter(|card| {
                    crate::query::effective_card_color_mask(state, player, *card) == *color
                })
                .collect();
            !matching.is_empty() && matching.iter().all(|card| discardable.contains(card))
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_rehuaiyi(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    color: u8,
    discarded_count: u32,
    targets: Vec<PlayerId>,
    mut index: u32,
    gained_count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let skill_valid = tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    });
    if !skill_valid
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive && !participant.dying)
    {
        return Err(EffectError::Illegal(
            "invalid Rehuaiyi skill context".into(),
        ));
    }

    match stage {
        0 if color == 0 => {
            if discarded_count != 0 || !targets.is_empty() || index != 0 || gained_count != 0 {
                return Err(EffectError::ContentFault(
                    "bad initial Rehuaiyi progress".into(),
                ));
            }
            let cards = tx.state().hand_cards(player);
            if cards.is_empty() {
                return Err(EffectError::Illegal(
                    "Rehuaiyi requires a nonempty hand".into(),
                ));
            }
            let color_mask = cards.iter().fold(0, |mask, card| {
                mask | crate::query::effective_card_color_mask(tx.state(), player, *card)
            }) & 0b11;
            let mut events = vec![DomainEvent::CardsRevealed {
                cards,
                to: Vec::new(),
            }];
            if matches!(color_mask, 0b01 | 0b10) {
                events.extend(draw_cards(tx, player, 1)?);
                return Ok(events);
            }
            if color_mask != 0b11 {
                return Ok(events);
            }

            let instance = tx
                .state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::Illegal("Rehuaiyi skill disappeared".into()))?;
            instance.play_phase_use_count = instance.play_phase_use_count.max(2);

            let colors = rehuaiyi_eligible_colors(tx.state(), player);
            match colors.as_slice() {
                [] => Ok(events),
                [only] => {
                    events.extend(resolve_rehuaiyi(
                        tx,
                        player,
                        0,
                        *only,
                        0,
                        Vec::new(),
                        0,
                        0,
                        skill,
                    )?);
                    Ok(events)
                }
                _ => {
                    let mut payload = vec![0, player.raw(), skill.raw(), colors.len() as u32];
                    payload.extend(colors.iter().map(|color| u32::from(*color)));
                    events.extend(open_choice(
                        tx,
                        player,
                        1,
                        1,
                        false,
                        colors.into_iter().map(u32::from).collect(),
                        ChoiceKind::Custom,
                        ContinuationKind::ResolveRehuaiyi,
                        payload,
                    )?);
                    Ok(events)
                }
            }
        }
        0 if matches!(color, 0b01 | 0b10) => {
            let eligible = rehuaiyi_eligible_colors(tx.state(), player);
            if !eligible.contains(&color) {
                return Err(EffectError::Illegal(
                    "selected Rehuaiyi color is no longer discardable".into(),
                ));
            }
            let cards: Vec<_> = tx
                .state()
                .hand_cards(player)
                .into_iter()
                .filter(|card| {
                    crate::query::effective_card_color_mask(tx.state(), player, *card) == color
                })
                .collect();
            if cards.is_empty() {
                return Err(EffectError::Illegal(
                    "selected Rehuaiyi color has no hand cards".into(),
                ));
            }
            let count = cards.len().min(u32::MAX as usize) as u32;
            run_effects(
                tx,
                vec![
                    EffectRequest::DiscardCards { player, cards },
                    EffectRequest::ResolveRehuaiyi {
                        player,
                        stage: 1,
                        color: 0,
                        discarded_count: count,
                        targets: Vec::new(),
                        index: 0,
                        gained_count: 0,
                        skill,
                    },
                ],
            )
        }
        1 => {
            if color != 0
                || discarded_count == 0
                || !targets.is_empty()
                || index != 0
                || gained_count != 0
            {
                return Err(EffectError::ContentFault(
                    "bad Rehuaiyi target-choice progress".into(),
                ));
            }
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&target| {
                    target != player
                        && !crate::query::gainable_target_cards(
                            tx.state(),
                            player,
                            target,
                            rule_api::TargetCardScope::HandEquip,
                        )
                        .is_empty()
                })
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let max = options
                .len()
                .min(discarded_count as usize)
                .min(u8::MAX as usize) as u8;
            let mut payload = vec![
                1,
                player.raw(),
                skill.raw(),
                discarded_count,
                options.len() as u32,
            ];
            payload.extend(options.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                1,
                max,
                true,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveRehuaiyi,
                payload,
            )
        }
        2 => {
            if color != 0 || discarded_count != 0 || targets.is_empty() {
                return Err(EffectError::ContentFault(
                    "bad Rehuaiyi gain progress".into(),
                ));
            }
            while let Some(target) = targets.get(index as usize).copied() {
                let options = crate::query::gainable_target_cards(
                    tx.state(),
                    player,
                    target,
                    rule_api::TargetCardScope::HandEquip,
                );
                if options.is_empty() {
                    index = index.saturating_add(1);
                    continue;
                }
                let mut payload = vec![
                    2,
                    player.raw(),
                    skill.raw(),
                    index,
                    gained_count,
                    targets.len() as u32,
                ];
                payload.extend(targets.iter().map(|target| target.raw()));
                payload.push(options.len() as u32);
                payload.extend(options.iter().map(|card| card.raw()));
                return open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    options.into_iter().map(CardId::raw).collect(),
                    ChoiceKind::SelectCards,
                    ContinuationKind::ResolveRehuaiyi,
                    payload,
                );
            }
            if gained_count >= 2 {
                return run_effects(
                    tx,
                    vec![EffectRequest::DealDamage {
                        source: PlayerId::NONE,
                        target: player,
                        amount: 1,
                        is_lose_hp: true,
                        nature: rule_api::damage_nature::NORMAL,
                    }],
                );
            }
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault(
            "unknown Rehuaiyi resolution stage".into(),
        )),
    }
}

pub(crate) fn choose_gain_target_card_alternate_zone_then_target_draw(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    selected: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    const HAND_USED: u8 = 1;
    const EQUIP_USED: u8 = 2;

    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || target == player
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "invalid alternate-zone target gain context".into(),
        ));
    }
    let used = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.play_phase_category_mask)
        .ok_or_else(|| EffectError::Illegal("alternate-zone skill is missing".into()))?;
    let hand = tx.state().zones.find(ZoneKind::Hand, target);
    let equip = tx.state().zones.find(ZoneKind::Equip, target);
    let mut options = Vec::new();
    if used & HAND_USED == 0 {
        options.extend(
            hand.and_then(|zone| tx.state().zones.get(zone))
                .into_iter()
                .flat_map(|zone| zone.cards.iter().copied()),
        );
    }
    if used & EQUIP_USED == 0 {
        options.extend(
            equip
                .and_then(|zone| tx.state().zones.get(zone))
                .into_iter()
                .flat_map(|zone| zone.cards.iter().copied()),
        );
    }
    if selected.is_none() {
        if options.is_empty() {
            return Err(EffectError::Illegal(
                "target has no card in an unused zone".into(),
            ));
        }
        let mut payload = vec![
            player.raw(),
            target.raw(),
            skill.raw(),
            options.len() as u32,
        ];
        payload.extend(options.iter().map(|card| card.raw()));
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            options.iter().map(|card| card.raw()).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::ChooseGainTargetCardAlternateZoneThenTargetDraw,
            payload,
        );
    }
    if !options.contains(&selected) {
        return Err(EffectError::Illegal(
            "selected target card is stale or from an already used zone".into(),
        ));
    }
    let card = tx
        .state()
        .cards
        .get(selected)
        .ok_or_else(|| EffectError::Illegal("selected target card is missing".into()))?;
    let (from, used_bit) = if Some(card.zone) == hand {
        (card.zone, HAND_USED)
    } else if Some(card.zone) == equip {
        (card.zone, EQUIP_USED)
    } else {
        return Err(EffectError::Illegal(
            "selected card left the target hand/equipment".into(),
        ));
    };
    tx.state_mut()
        .skills
        .get_mut(skill)
        .expect("validated alternate-zone skill")
        .play_phase_category_mask |= used_bit;
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::DrawCards {
            player: target,
            count: 1,
        });
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("skill owner hand missing".into()))?;
    move_cards(tx, vec![selected], to, Some(from))
}

/// 神君 (clanshenjun) reveal: reveal and gaintag every hand card sharing the
/// event card's effective kind (杀 or 锦囊). The tag is the skill's definition
/// id, so only 神君's own phase-end use consumes these cards.
pub(crate) fn clanshenjun_reveal(
    tx: &mut Transaction,
    player: engine_api::PlayerId,
    event_card: engine_api::CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<engine_api::DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || !originating_skill_is_enabled(tx, player, skill)
    {
        return Ok(Vec::new());
    }
    let Some(event) = tx.state().cards.get(event_card) else {
        return Ok(Vec::new());
    };
    let kind = crate::query::effective_card_kind(tx.state(), player, event);
    if !(kind.is_sha() || kind.is_trick()) {
        return Ok(Vec::new());
    }
    let matching: Vec<_> = tx
        .state()
        .hand_cards(player)
        .into_iter()
        .filter(|card| {
            tx.state().cards.get(*card).is_some_and(|candidate| {
                crate::query::effective_card_kind(tx.state(), player, candidate) == kind
            })
        })
        .collect();
    if matching.is_empty() {
        return Ok(Vec::new());
    }
    let tag = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.def.raw())
        .unwrap_or(0);
    for card in &matching {
        tx.state_mut().add_content_card_tag(*card, tag);
    }
    Ok(vec![engine_api::DomainEvent::CardsRevealed {
        cards: matching,
        to: Vec::new(),
    }])
}

/// 神君 (clanshenjun) phase-end use: open a choice over the gaintagged hand
/// cards (stage 0), then a target choice (stage 1), then play the selected card
/// as its own kind through the normal card pipeline.
///
/// NOTE: the choice continuation needs a `ContinuationKind` discriminant, but that enum is
/// `repr(u8)` and already full (0..=255). The phase-end use path is therefore stubbed until
/// the enum is widened to `repr(u16)`. The clanshenjun DSL remains a no-op, so this is never
/// reached in shipped content.
pub(crate) fn clanshenjun_use(
    tx: &mut Transaction,
    player: engine_api::PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<engine_api::DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || !originating_skill_is_enabled(tx, player, skill)
    {
        return Ok(Vec::new());
    }
    let Some(tag) = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.def.raw())
    else {
        return Ok(Vec::new());
    };
    let tagged: Vec<_> = tx
        .state()
        .hand_cards(player)
        .into_iter()
        .filter(|card| tx.state().card_has_content_tag(*card, tag))
        .collect();
    if tagged.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        tagged.into_iter().map(engine_api::CardId::raw).collect(),
        engine_api::ChoiceKind::SelectCards,
        crate::state::ContinuationKind::ClanshenjunUseCard,
        vec![0, player.raw(), skill.raw()],
    )
}

/// Choice continuation for 神君's phase-end use.
pub(crate) fn resolve_clanshenjun_use_choice(
    tx: &mut Transaction,
    observer: engine_api::PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.first().unwrap_or(&0);
    let player = engine_api::PlayerId::new(*payload.get(1).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(2).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "神君 use must be chosen by its owner".into(),
        ));
    }
    match stage {
        0 => {
            if cancelled {
                return Ok(());
            }
            let [card] = selections else {
                return Err(EffectError::Illegal(
                    "神君 requires exactly one tagged card".into(),
                ));
            };
            let card = engine_api::CardId::new(*card);
            let tag = tx
                .state()
                .skills
                .get(skill)
                .map(|instance| instance.def.raw())
                .unwrap_or(0);
            if !tx.state().hand_cards(player).contains(&card)
                || !tx.state().card_has_content_tag(card, tag)
            {
                return Err(EffectError::Illegal(
                    "神君 card is no longer tagged or in hand".into(),
                ));
            }
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| *target != player)
                .collect();
            if targets.is_empty() {
                return Err(EffectError::Illegal("神君 has no target".into()));
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                targets.into_iter().map(engine_api::PlayerId::raw).collect(),
                engine_api::ChoiceKind::SelectTargets,
                crate::state::ContinuationKind::ClanshenjunUseCard,
                vec![1, player.raw(), skill.raw(), card.raw()],
            )
            .map(|_| ())
        }
        1 => {
            if cancelled {
                return Ok(());
            }
            let [target] = selections else {
                return Err(EffectError::Illegal(
                    "神君 requires exactly one target".into(),
                ));
            };
            let card = engine_api::CardId::new(*payload.get(3).unwrap_or(&0));
            let target = engine_api::PlayerId::new(*target);
            let Some(card_state) = tx.state().cards.get(card) else {
                return Err(EffectError::Illegal("神君 card missing".into()));
            };
            let kind = crate::query::effective_card_kind(tx.state(), player, card_state);
            crate::command::play::play_as_kind(
                tx,
                player,
                card,
                kind,
                &[target],
                false,
                false,
                false,
                true,
            )
            .map(|_| ())
        }
        _ => Err(EffectError::ContentFault("神君 stage out of range".into())),
    }
}

/// 迂志 (clanyuzhi) round-start: open a choice over the owner's hand cards; the
/// continuation reveals + gaintags the chosen card, draws its name length in cards
/// (tagging the draws), and records the round draw counts in keyed storage.
pub(crate) fn clanyuzhi_reveal(
    tx: &mut Transaction,
    player: engine_api::PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<engine_api::DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || !originating_skill_is_enabled(tx, player, skill)
    {
        return Ok(Vec::new());
    }
    let options: Vec<u32> = tx
        .state()
        .hand_cards(player)
        .into_iter()
        .filter(|card| {
            crate::query::card_name_length(tx.state(), player, *card)
                .is_some_and(|len| len > 0)
        })
        .map(engine_api::CardId::raw)
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        options,
        engine_api::ChoiceKind::SelectCards,
        crate::state::ContinuationKind::ClanyuzhiRevealChoice,
        vec![0, player.raw(), skill.raw()],
    )
}

/// Stage 0 of the 迂志 round-start: reveal + gaintag the chosen hand card, draw its
/// name length (gaintagging every drawn card), rotate the round draw counters.
pub(crate) fn resolve_clanyuzhi_reveal_choice(
    tx: &mut Transaction,
    observer: engine_api::PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.first().unwrap_or(&0);
    let player = engine_api::PlayerId::new(*payload.get(1).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(2).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "迂志 must be chosen by its owner".into(),
        ));
    }
    match stage {
        0 => {
            if cancelled {
                return Ok(());
            }
            let [card] = selections else {
                return Err(EffectError::Illegal(
                    "迂志 requires exactly one hand card".into(),
                ));
            };
            let card = engine_api::CardId::new(*card);
            if !tx.state().hand_cards(player).contains(&card) {
                return Err(EffectError::Illegal(
                    "迂志 card is no longer in hand".into(),
                ));
            }
            let Some(tag) = tx
                .state()
                .skills
                .get(skill)
                .map(|instance| instance.def.raw())
            else {
                return Ok(());
            };
            tx.state_mut().add_content_card_tag(card, tag);
            let name_len =
                crate::query::card_name_length(tx.state(), player, card).unwrap_or(1) as u32;
            let draw_events = crate::effect::primitives::pack_sp_2::draw_cards(tx, player, name_len)?;
            let mut drawn = Vec::new();
            for event in &draw_events {
                if let engine_api::DomainEvent::CardsDrawn { player: drawer, cards } = event {
                    if *drawer == player {
                        drawn.extend(cards.iter().copied());
                    }
                }
            }
            for drawn_card in &drawn {
                tx.state_mut().add_content_card_tag(*drawn_card, tag);
            }
            let previous = tx.state().skills.storage_get(skill, YUZHI_KEY_DRAWN_ROUND);
            tx.state_mut()
                .skills
                .storage_set(skill, YUZHI_KEY_DRAWN_LAST, previous);
            tx.state_mut()
                .skills
                .storage_set(skill, YUZHI_KEY_DRAWN_ROUND, name_len);
            tx.state_mut().skills.storage_set(skill, YUZHI_KEY_MARK, name_len);
            tx.push_event(engine_api::DomainEvent::CardsRevealed {
                cards: vec![card],
                to: Vec::new(),
            });
            Ok(())
        }
        _ => Err(EffectError::ContentFault("迂志 stage out of range".into())),
    }
}

/// 迂志 (clanyuzhi) round-end: discard every gaintagged hand card, then if this
/// round's 迂志 draws exceeded the previous round's (both positive) or the round's
/// used-card count, the owner loses 保族 or takes 1 thunder damage.
pub(crate) fn clanyuzhi_round_end(
    tx: &mut Transaction,
    player: engine_api::PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<engine_api::DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || !originating_skill_is_enabled(tx, player, skill)
    {
        return Ok(Vec::new());
    }
    let Some(tag) = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.def.raw())
    else {
        return Ok(Vec::new());
    };
    let mut events = Vec::new();
    let tagged: Vec<_> = tx
        .state()
        .hand_cards(player)
        .into_iter()
        .filter(|card| tx.state().card_has_content_tag(*card, tag))
        .collect();
    if !tagged.is_empty() {
        events.extend(crate::effect::primitives::pack_sp_1::discard_target_cards(
            tx,
            player,
            player,
            tagged,
            rule_api::TargetCardScope::Hand,
        )?);
    }
    let num1 = tx.state().skills.storage_get(skill, YUZHI_KEY_DRAWN_ROUND);
    let num2 = tx.state().skills.storage_get(skill, YUZHI_KEY_DRAWN_LAST);
    let num3 = tx.state().turn.round_card_use_count(player);
    let ambitious = num1 > 0 && num2 > 0 && num1 > num2;
    let lagging = num1 > num3;
    if !(ambitious || lagging) {
        return Ok(events);
    }
    if !crate::effect::primitives::pack_old::player_has_skill_def(
        tx,
        player,
        engine_api::SkillDefId::new(YUZHI_CLAN_BAOZU),
        false,
    ) {
        events.extend(crate::effect::primitives::pack_mobile::deal_damage(
            tx,
            player,
            player,
            1,
            false,
            2,
            false,
            engine_api::CardId::NONE,
            0,
            0,
        )?);
        return Ok(events);
    }
    events.extend(open_choice(
        tx,
        player,
        1,
        1,
        false,
        vec![0, 1],
        engine_api::ChoiceKind::Confirm,
        crate::state::ContinuationKind::ClanyuzhiEndChoice,
        vec![0, player.raw(), skill.raw()],
    )?);
    Ok(events)
}

/// Stage 0 of the 迂志 round-end choice: selection 0 loses 保族, selection 1 takes
/// 1 thunder damage.
pub(crate) fn resolve_clanyuzhi_end_choice(
    tx: &mut Transaction,
    observer: engine_api::PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.first().unwrap_or(&0);
    let player = engine_api::PlayerId::new(*payload.get(1).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(2).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "迂志 round-end must be chosen by its owner".into(),
        ));
    }
    match stage {
        0 => {
            if cancelled {
                return Ok(());
            }
            let [choice] = selections else {
                return Err(EffectError::Illegal(
                    "迂志 round-end requires one selection".into(),
                ));
            };
            match *choice {
                0 => crate::effect::primitives::pack_tw_1::remove_skill_by_def(
                    tx,
                    player,
                    engine_api::SkillDefId::new(YUZHI_CLAN_BAOZU),
                )
                .map(|_| ()),
                1 => {
                    crate::effect::primitives::pack_mobile::deal_damage(
                        tx,
                        player,
                        player,
                        1,
                        false,
                        2,
                        false,
                        engine_api::CardId::NONE,
                        0,
                        0,
                    )
                    .map(|_| ())
                }
                _ => Err(EffectError::Illegal(
                    "迂志 round-end selection out of range".into(),
                )),
            }
        }
        _ => Err(EffectError::ContentFault("迂志 stage out of range".into())),
    }
}

/// 惬诣 (clanqieyi) play-phase start: reset the per-suit first-use storage keys and
/// privately view the top two draw-pile cards.
///
/// Upstream `phaseUseBegin` shows the top 2 without reordering (viewCards). The per-suit
/// first-use gate (upstream `clanqieyi_effect` temp skill tracking used suits) is
/// approximated with per-suit storage keys 10..13 (suit tag offset), cleared here so each
/// play phase restarts the gate.
pub(crate) fn qieyi_phase_start(
    tx: &mut Transaction,
    player: engine_api::PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<engine_api::DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || !originating_skill_is_enabled(tx, player, skill)
    {
        return Ok(Vec::new());
    }
    for suit in 0..4 {
        tx.state_mut()
            .skills
            .storage_set(skill, QIEYI_SUIT_KEY_BASE + suit, 0);
    }
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("惬诣 draw pile missing".into()))?;
    if tx
        .state()
        .zones
        .get(draw)
        .is_none_or(|zone| zone.cards.is_empty())
    {
        reshuffle_discard_into_draw(tx)?;
    }
    let top: Vec<_> = tx
        .state()
        .zones
        .get(draw)
        .map(|zone| zone.cards.iter().rev().take(2).copied().collect())
        .unwrap_or_default();
    if top.is_empty() {
        return Ok(Vec::new());
    }
    Ok(vec![engine_api::DomainEvent::CardsRevealed {
        cards: top,
        to: vec![player],
    }])
}

/// 惬诣 (clanqieyi) card-used: on the first use of a suit this play phase, reveal the top
/// draw-pile card; same color or same kind as the used card → gain it, otherwise open the
/// put-one-card-on-top choice.
pub(crate) fn qieyi_use_after(
    tx: &mut Transaction,
    player: engine_api::PlayerId,
    event_card: engine_api::CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<engine_api::DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || !originating_skill_is_enabled(tx, player, skill)
    {
        return Ok(Vec::new());
    }
    let suit_tag = {
        let Some(event) = tx.state().cards.get(event_card) else {
            return Ok(Vec::new());
        };
        crate::query::effective_card_suit(tx.state(), player, event) as u32
    };
    if suit_tag >= 4 {
        return Ok(Vec::new());
    }
    let key = QIEYI_SUIT_KEY_BASE + suit_tag;
    if tx.state().skills.storage_get(skill, key) != 0 {
        return Ok(Vec::new());
    }
    tx.state_mut().skills.storage_set(skill, key, 1);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("惬诣 draw pile missing".into()))?;
    if tx
        .state()
        .zones
        .get(draw)
        .is_none_or(|zone| zone.cards.is_empty())
    {
        reshuffle_discard_into_draw(tx)?;
    }
    let Some(top) = tx
        .state()
        .zones
        .get(draw)
        .and_then(|zone| zone.cards.last().copied())
    else {
        return Ok(Vec::new());
    };
    let mut events = vec![engine_api::DomainEvent::CardsRevealed {
        cards: vec![top],
        to: Vec::new(),
    }];
    let (same_color, used_kind, top_kind) = {
        let (Some(event), Some(top_card)) = (
            tx.state().cards.get(event_card),
            tx.state().cards.get(top),
        ) else {
            return Ok(events);
        };
        let used_kind = crate::query::effective_card_kind(tx.state(), player, event);
        let top_kind = crate::query::effective_card_kind(tx.state(), player, top_card);
        let top_suit = crate::query::effective_card_suit(tx.state(), player, top_card);
        let suit = match suit_tag {
            0 => engine_api::Suit::Spade,
            1 => engine_api::Suit::Heart,
            2 => engine_api::Suit::Club,
            _ => engine_api::Suit::Diamond,
        };
        let same_color = crate::query::card_color_mask_for_suit(suit) != 0
            && crate::query::card_color_mask_for_suit(suit)
                == crate::query::card_color_mask_for_suit(top_suit);
        (same_color, used_kind, top_kind)
    };
    if same_color || used_kind == top_kind {
        // Same color or same kind → gain the revealed card.
        let hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::Illegal("惬诣 hand zone missing".into()))?;
        crate::effect::primitives::pack_mobile::move_one_card(
            tx,
            top,
            draw,
            hand,
            player,
            true,
        )?;
        return Ok(events);
    }
    // Otherwise: choose one hand/equip card to put on top of the pile.
    let hand_zone = tx.state().zones.find(ZoneKind::Hand, player);
    let equip_zone = tx.state().zones.find(ZoneKind::Equip, player);
    let mut options: Vec<u32> = Vec::new();
    if let Some(zone) = hand_zone {
        if let Some(zone_state) = tx.state().zones.get(zone) {
            options.extend(zone_state.cards.iter().map(|card| card.raw()));
        }
    }
    if let Some(zone) = equip_zone {
        if let Some(zone_state) = tx.state().zones.get(zone) {
            options.extend(zone_state.cards.iter().map(|card| card.raw()));
        }
    }
    if options.is_empty() {
        return Ok(events);
    }
    events.extend(open_choice(
        tx,
        player,
        1,
        1,
        true,
        options,
        engine_api::ChoiceKind::SelectCards,
        crate::state::ContinuationKind::QieyiPutTopChoice,
        vec![0, player.raw(), skill.raw()],
    )?);
    Ok(events)
}

/// Stage 0 of the 惬诣 put-on-top choice: validate the selected hand/equip card and move
/// it to the top of the draw pile (the draw pile's top is its last element).
pub(crate) fn resolve_qieyi_put_top_choice(
    tx: &mut Transaction,
    observer: engine_api::PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.first().unwrap_or(&0);
    let player = engine_api::PlayerId::new(*payload.get(1).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(2).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "惬诣 must be chosen by its owner".into(),
        ));
    }
    match stage {
        0 => {
            if cancelled {
                return Ok(());
            }
            let [card] = selections else {
                return Err(EffectError::Illegal(
                    "惬诣 requires exactly one card".into(),
                ));
            };
            let card = engine_api::CardId::new(*card);
            let from = tx
                .state()
                .cards
                .get(card)
                .map(|state| state.zone)
                .ok_or_else(|| EffectError::Illegal("惬诣 card missing".into()))?;
            let from_zone = tx
                .state()
                .zones
                .get(from)
                .ok_or_else(|| EffectError::Illegal("惬诣 card zone missing".into()))?;
            if from_zone.owner != player
                || !matches!(from_zone.kind, ZoneKind::Hand | ZoneKind::Equip)
            {
                return Err(EffectError::Illegal(
                    "惬诣 card must be in the owner's hand or equipment".into(),
                ));
            }
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("惬诣 draw pile missing".into()))?;
            if !tx
                .state()
                .skills
                .get(skill)
                .is_some_and(|instance| instance.owner == player)
            {
                return Err(EffectError::Illegal("惬诣 skill missing".into()));
            }
            crate::effect::primitives::pack_mobile::move_one_card(
                tx,
                card,
                from,
                draw,
                PlayerId::NONE,
                true,
            )
            .map(|_| ())
        }
        _ => Err(EffectError::ContentFault("惬诣 stage out of range".into())),
    }
}

/// 谏直 (clanjianzhi) phase-end: if 惬诣 (clanqieyi) fired this turn, open the judgment
/// count choice (1..=惬诣 fires). The continuation runs the judgments, assigns
/// matching-suit judge cards to players, and deals 1 thunder damage per non-matching
/// judge card.
///
/// Cross-skill state query: the 惬诣 instance's `turn_fire_count` (incremented by the
/// trigger dispatcher on every fire) approximates upstream `getHistory("useSkill",
/// clanqieyi).length` for the current turn.
pub(crate) fn jianzhi_phase_end(
    tx: &mut Transaction,
    player: engine_api::PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<engine_api::DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || !originating_skill_is_enabled(tx, player, skill)
    {
        return Ok(Vec::new());
    }
    let qieyi_fires = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .filter(|instance| instance.owner == player && instance.def.raw() == JIANZHI_CLANQIEYI)
        .map(|instance| instance.turn_fire_count)
        .next()
        .unwrap_or(0);
    if qieyi_fires == 0 {
        return Ok(Vec::new());
    }
    let options: Vec<u32> = (1..=qieyi_fires).collect();
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        options,
        engine_api::ChoiceKind::Custom,
        crate::state::ContinuationKind::JianzhiCountChoice,
        vec![0, player.raw(), skill.raw()],
    )
}

/// Stage 0 of 谏直: run the chosen number of judgments (raw top-pile reveals), split the
/// judge cards by this play phase's used suits, open the assignment loop, and remember
/// whether any non-matching judge card appeared.
pub(crate) fn resolve_jianzhi_count_choice(
    tx: &mut Transaction,
    observer: engine_api::PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.first().unwrap_or(&0);
    let player = engine_api::PlayerId::new(*payload.get(1).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(2).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "谏直 must be chosen by its owner".into(),
        ));
    }
    match stage {
        0 => {
            if cancelled {
                return Ok(());
            }
            let [count] = selections else {
                return Err(EffectError::Illegal(
                    "谏直 requires one judgment count".into(),
                ));
            };
            let count = *count;
            if count == 0 || count > 16 {
                return Err(EffectError::Illegal(
                    "谏直 judgment count out of range".into(),
                ));
            }
            let mut judge_cards = Vec::new();
            for _ in 0..count {
                let draw = tx
                    .state()
                    .zones
                    .find(ZoneKind::DrawPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::Illegal("谏直 draw pile missing".into()))?;
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
                    .and_then(|zone| zone.cards.last().copied())
                else {
                    break;
                };
                tx.push_event(engine_api::DomainEvent::CardsRevealed {
                    cards: vec![card],
                    to: Vec::new(),
                });
                // Upstream `judge()` removes the judge card from the pile; the engine has
                // no expansion/limbo zone, so the card is held in the discard pile until the
                // assignment loop moves it to its recipient's hand.
                let discard = tx
                    .state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::Illegal("谏直 discard pile missing".into()))?;
                crate::effect::primitives::pack_mobile::move_one_card(
                    tx,
                    card,
                    draw,
                    discard,
                    PlayerId::NONE,
                    true,
                )?;
                judge_cards.push(card);
            }
            if judge_cards.is_empty() {
                return Ok(());
            }
            let suit_mask = tx.state().turn.play_phase_card_suit_mask(player);
            let mut matching = Vec::new();
            let mut has_non_matching = false;
            for card in &judge_cards {
                let Some(card_state) = tx.state().cards.get(*card) else {
                    continue;
                };
                let bit = match card_state.suit {
                    engine_api::Suit::Spade => 1,
                    engine_api::Suit::Heart => 2,
                    engine_api::Suit::Club => 4,
                    engine_api::Suit::Diamond => 8,
                    engine_api::Suit::None => 0,
                };
                if bit != 0 && suit_mask & bit != 0 {
                    matching.push(card.raw());
                } else {
                    has_non_matching = true;
                }
            }
            if matching.is_empty() {
                if has_non_matching {
                    crate::effect::primitives::pack_mobile::deal_damage(
                        tx,
                        engine_api::PlayerId::NONE,
                        player,
                        1,
                        false,
                        2,
                        false,
                        engine_api::CardId::NONE,
                        0,
                        0,
                    )?;
                }
                return Ok(());
            }
            let mut next_payload = vec![
                0,
                player.raw(),
                skill.raw(),
                u32::from(has_non_matching),
            ];
            next_payload.extend(matching.iter().copied());
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                matching,
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::JianzhiAssignCard,
                next_payload,
            )
            .map(|_| ())
        }
        _ => Err(EffectError::ContentFault("谏直 stage out of range".into())),
    }
}

/// 谏直 assignment loop: stage 0 selects one matching judge card, stage 1 a recipient;
/// the card is moved to the recipient's hand, the loop re-opens while cards remain, and
/// a non-matching judge card deals 1 thunder damage to the owner once the loop ends.
pub(crate) fn resolve_jianzhi_assign_card(
    tx: &mut Transaction,
    observer: engine_api::PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.first().unwrap_or(&0);
    let player = engine_api::PlayerId::new(*payload.get(1).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(2).unwrap_or(&0));
    let has_non_matching = *payload.get(3).unwrap_or(&0) != 0;
    let remaining: Vec<u32> = payload.get(4..).unwrap_or(&[]).to_vec();
    if observer != player {
        return Err(EffectError::Illegal(
            "谏直 must be chosen by its owner".into(),
        ));
    }
    match stage {
        0 => {
            if cancelled {
                return Ok(());
            }
            let [card] = selections else {
                return Err(EffectError::Illegal(
                    "谏直 requires exactly one judge card".into(),
                ));
            };
            if !remaining.contains(card) {
                return Err(EffectError::Illegal(
                    "谏直 card is no longer assignable".into(),
                ));
            }
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(engine_api::PlayerId::raw)
                .collect();
            if targets.is_empty() {
                return Err(EffectError::Illegal("谏直 has no recipient".into()));
            }
            let mut next_payload = vec![
                1,
                player.raw(),
                skill.raw(),
                u32::from(has_non_matching),
                *card,
            ];
            next_payload.extend(
                remaining
                    .iter()
                    .copied()
                    .filter(|candidate| candidate != card),
            );
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                targets,
                engine_api::ChoiceKind::SelectTargets,
                crate::state::ContinuationKind::JianzhiAssignCard,
                next_payload,
            )
            .map(|_| ())
        }
        1 => {
            if cancelled {
                return Ok(());
            }
            let [target] = selections else {
                return Err(EffectError::Illegal(
                    "谏直 requires exactly one recipient".into(),
                ));
            };
            let target = engine_api::PlayerId::new(*target);
            let card = engine_api::CardId::new(*payload.get(4).unwrap_or(&0));
            if card == engine_api::CardId::NONE || !tx.state().cards.get(card).is_some() {
                return Err(EffectError::Illegal("谏直 judge card missing".into()));
            }
            let rest: Vec<u32> = payload.get(5..).unwrap_or(&[]).to_vec();
            let from = tx
                .state()
                .cards
                .get(card)
                .map(|state| state.zone)
                .ok_or_else(|| EffectError::Illegal("谏直 card zone missing".into()))?;
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("谏直 recipient hand missing".into()))?;
            crate::effect::primitives::pack_mobile::move_one_card(
                tx,
                card,
                from,
                hand,
                target,
                true,
            )?;
            if !rest.is_empty() {
                let mut next_payload = vec![
                    0,
                    player.raw(),
                    skill.raw(),
                    u32::from(has_non_matching),
                ];
                next_payload.extend(rest.iter().copied());
                open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    rest,
                    engine_api::ChoiceKind::SelectCards,
                    crate::state::ContinuationKind::JianzhiAssignCard,
                    next_payload,
                )
                .map(|_| ())
            } else if has_non_matching {
                crate::effect::primitives::pack_mobile::deal_damage(
                    tx,
                    engine_api::PlayerId::NONE,
                    player,
                    1,
                    false,
                    2,
                    false,
                    engine_api::CardId::NONE,
                    0,
                    0,
                )
                .map(|_| ())
            } else {
                Ok(())
            }
        }
        _ => Err(EffectError::ContentFault("谏直 stage out of range".into())),
    }
}
