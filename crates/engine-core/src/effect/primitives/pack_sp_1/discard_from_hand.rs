//! Effect primitives for `choose_move_field_card_to` and related helpers.

use super::*;

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
    super::super::run_effects(
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
    super::super::run_effects(
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
    super::super::run_effects(tx, effects)
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
    let options: Vec<_> = tx
        .state()
        .hand_cards(source)
        .into_iter()
        .map(CardId::raw)
        .collect();
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
