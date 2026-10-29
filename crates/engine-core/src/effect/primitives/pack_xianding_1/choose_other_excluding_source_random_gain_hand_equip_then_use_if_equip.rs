//! Effect primitives for `set_potfaen_pending` and related helpers.

use super::*;

pub(crate) fn choose_other_excluding_source_random_gain_hand_equip_then_use_if_equip(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if stage == 1 {
        let Some(card_state) = tx.state().cards.get(card) else {
            return Ok(Vec::new());
        };
        let hand = tx.state().zones.find(ZoneKind::Hand, player);
        let kind = crate::query::effective_card_kind(tx.state(), player, card_state);
        let can_use = hand == Some(card_state.zone)
            && card_state.owner == player
            && tx.state().players.get(player).is_some_and(|participant| {
                participant.alive
                    && !participant.dying
                    && kind
                        .equip_slot()
                        .is_some_and(|slot| participant.equip_slot_enabled(slot))
            })
            && !crate::query::card_use_cancelled_for_card(tx.state(), player, card);
        if can_use {
            crate::command::play::use_equipment_from_hand(tx, player, card)?;
        }
        return Ok(Vec::new());
    }
    if stage != 0 {
        return Err(EffectError::ContentFault("bad Jiebing stage".into()));
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
    let options: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| {
            *target != player
                && *target != source
                && !crate::query::gainable_target_cards(
                    tx.state(),
                    player,
                    *target,
                    TargetCardScope::HandEquip,
                )
                .is_empty()
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
        ContinuationKind::Custom,
        vec![
            JIEBING_CUSTOM_CHOICE,
            player.raw(),
            source.raw(),
            skill.raw(),
        ],
    )
}

pub(crate) fn resolve_gzjinfa(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    cost_card: CardId,
    given_card: CardId,
    stage: u8,
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
    {
        return Err(EffectError::Illegal("invalid Jinfa context".into()));
    }
    match stage {
        0 => {
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            let equip = tx.state().zones.find(ZoneKind::Equip, player);
            if !tx.state().cards.get(cost_card).is_some_and(|state| {
                state.owner == player && (Some(state.zone) == hand || Some(state.zone) == equip)
            }) || crate::query::discardable_hand_equip_cards(tx.state(), target).is_empty()
            {
                return Err(EffectError::Illegal("invalid Jinfa cost or target".into()));
            }
            super::super::run_effects(
                tx,
                vec![
                    EffectRequest::DiscardCards {
                        player,
                        cards: vec![cost_card],
                    },
                    EffectRequest::ResolveGzjinfa {
                        player,
                        target,
                        cost_card,
                        given_card: CardId::NONE,
                        stage: 1,
                        skill,
                    },
                ],
            )
        }
        1 => {
            let hand = tx.state().zones.find(ZoneKind::Hand, target);
            let equip = tx.state().zones.find(ZoneKind::Equip, target);
            let mut options: Vec<_> = [hand, equip]
                .into_iter()
                .flatten()
                .flat_map(|zone| {
                    tx.state()
                        .zones
                        .get(zone)
                        .map(|state| state.cards.clone())
                        .unwrap_or_default()
                })
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|state| state.kind.is_equip())
                })
                .collect();
            options.sort_by_key(|card| card.raw());
            if options.is_empty() {
                let gainable = crate::query::gainable_target_cards(
                    tx.state(),
                    player,
                    target,
                    rule_api::TargetCardScope::HandEquip,
                );
                if gainable.is_empty() {
                    return Ok(Vec::new());
                }
                let mut payload = vec![
                    GZJINFA_CUSTOM_CHOICE,
                    2,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                    gainable.len() as u32,
                ];
                payload.extend(gainable.iter().map(|card| card.raw()));
                return open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    gainable.into_iter().map(CardId::raw).collect(),
                    ChoiceKind::SelectCards,
                    ContinuationKind::Custom,
                    payload,
                );
            }
            let mut payload = vec![
                GZJINFA_CUSTOM_CHOICE,
                1,
                player.raw(),
                target.raw(),
                skill.raw(),
                options.len() as u32,
            ];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                target,
                0,
                1,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                payload,
            )
        }
        2 => {
            if !given_card.is_none()
                && tx.state().cards.get(given_card).is_some_and(|card| {
                    crate::query::effective_card_suit(tx.state(), target, card)
                        == engine_api::Suit::Spade
                })
            {
                try_use_virtual_sha_on_without_distance(tx, target, player, true, 0)
            } else {
                Ok(Vec::new())
            }
        }
        _ => Err(EffectError::ContentFault("bad Jinfa stage".into())),
    }
}

pub(crate) fn resolve_wn_qifeng(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let lost = tx.state().turn.card_loss_count(player);
    if !originating_skill_is_enabled(tx, player, skill)
        || lost == 0
        || player == target
        || tx.state().turn.current != target
        || tx.state().turn.phase != Phase::End
        || ![player, target].into_iter().all(|participant| {
            tx.state()
                .players
                .get(participant)
                .is_some_and(|state| state.alive)
        })
    {
        return Ok(Vec::new());
    }
    try_use_virtual_sha_on_without_distance(tx, player, target, true, lost - 1)
}

pub(crate) fn dcjue_discard_count(tx: &Transaction) -> u32 {
    let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
    tx.state()
        .turn
        .discarded_cards
        .iter()
        .filter(|card| {
            tx.state()
                .cards
                .get(**card)
                .is_some_and(|card| Some(card.zone) == discard)
        })
        .count()
        .min(u32::MAX as usize) as u32
}

pub(crate) fn dcjue_virtual_targets(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    kind: CardKind,
) -> Result<Vec<PlayerId>, EffectError> {
    let card = materialize_virtual_card(tx, player, kind)?;
    let spec = crate::command::play::skill_card_target_spec(tx, player, card);
    let options = match spec {
        Ok(crate::command::play::SkillCardTargetSpec::Immediate(options))
        | Ok(crate::command::play::SkillCardTargetSpec::Select { options, .. }) => options,
        Ok(crate::command::play::SkillCardTargetSpec::Pairs(_)) | Err(EffectError::Illegal(_)) => {
            Vec::new()
        }
        Err(error) => {
            remove_unused_virtual_card(tx, card)?;
            return Err(error);
        }
    };
    let targets = [player, target]
        .into_iter()
        .filter(|candidate| {
            options.contains(candidate)
                && (!matches!(kind, CardKind::Sha | CardKind::TrickGuohe) || *candidate != player)
        })
        .collect();
    remove_unused_virtual_card(tx, card)?;
    Ok(targets)
}

/// 举讹（杨彪，ABI 583）：一次选择后按仍在弃牌堆的本回合弃牌数循环；每轮以权威
/// RNG 打乱三种虚拟牌，使用第一个对本人/目标至少有一个合法目标的牌。
pub(crate) fn resolve_dcjue(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    remaining: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.phase != Phase::End
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            if target != tx.state().turn.current
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                || dcjue_discard_count(tx) == 0
                || tx
                    .state()
                    .skills
                    .get(skill)
                    .is_some_and(|instance| instance.round_fire_round == tx.state().turn.round)
            {
                return Ok(Vec::new());
            }
            if target == player {
                let options = tx
                    .state()
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .filter(|candidate| *candidate != player)
                    .map(PlayerId::raw)
                    .collect::<Vec<_>>();
                open_choice(
                    tx,
                    player,
                    0,
                    1,
                    true,
                    options,
                    ChoiceKind::SelectTargets,
                    ContinuationKind::Custom,
                    vec![RESOLVE_DCJUE_CHOICE, player.raw(), skill.raw()],
                )
            } else {
                open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    vec![0, 1],
                    ChoiceKind::Confirm,
                    ContinuationKind::Custom,
                    vec![
                        RESOLVE_DCJUE_CHOICE,
                        player.raw(),
                        target.raw(),
                        skill.raw(),
                    ],
                )
            }
        }
        1 => {
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let round = tx.state().turn.round;
            if tx
                .state()
                .skills
                .get(skill)
                .is_some_and(|instance| instance.round_fire_round == round)
            {
                return Ok(Vec::new());
            }
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.round_fire_round = round;
            }
            let cap = tx
                .state()
                .players
                .get(target)
                .map(|participant| participant.max_hp)
                .unwrap_or(0);
            let count = dcjue_discard_count(tx).min(cap);
            run_effects(
                tx,
                vec![EffectRequest::ResolveDcjue {
                    player,
                    target,
                    stage: 2,
                    remaining: count,
                    skill,
                }],
            )
        }
        2 => {
            if remaining == 0
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let mut kinds = [CardKind::Sha, CardKind::TrickGuohe, CardKind::TrickWugu];
            let mut rng = RuleRng::from_state(tx.state().rng);
            tx.journal(JournalEntry::RngCheckpoint);
            rng.shuffle(&mut kinds);
            tx.state_mut().rng = rng.into_state();
            for kind in kinds {
                let targets = dcjue_virtual_targets(tx, player, target, kind)?;
                if targets.is_empty() {
                    continue;
                }
                return run_effects(
                    tx,
                    vec![
                        EffectRequest::UseVirtualCardOnSelectedTargets {
                            player,
                            kind: kind.tag(),
                            targets,
                        },
                        EffectRequest::ResolveDcjue {
                            player,
                            target,
                            stage: 2,
                            remaining: remaining - 1,
                            skill,
                        },
                    ],
                );
            }
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault("bad dcjue stage".into())),
    }
}

pub(crate) fn ruanji_has_jiu(tx: &Transaction, player: PlayerId) -> bool {
    tx.state().players.get(player).is_some_and(|player| {
        player
            .marks
            .iter()
            .any(|(mark, count)| *mark == crate::content::MARK_JIU && *count > 0)
    })
}

pub(crate) fn ruanji_discardable_hand(tx: &Transaction, player: PlayerId) -> Vec<CardId> {
    crate::query::discardable_hand_equip_cards(tx.state(), player)
        .into_iter()
        .filter(|card| {
            tx.state().cards.get(*card).is_some_and(|card| {
                tx.state()
                    .zones
                    .get(card.zone)
                    .is_some_and(|zone| zone.kind == ZoneKind::Hand)
            })
        })
        .collect()
}

/// 阮籍（ABI 584）：复用技能实体牌集合与 source-free 转化保存「昭文」展示牌；
/// 「酒遁」只改写当前【酒】的尾效果，或在黑牌目标窗口打开唯一一次上下文选择。
pub(crate) fn resolve_ruanji(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    action: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            if tx.state().turn.current != player || tx.state().turn.phase != Phase::Play {
                return Ok(Vec::new());
            }
            let hand = tx.state().hand_cards(player);
            if hand.is_empty() {
                return Ok(Vec::new());
            }
            let token = tx
                .state()
                .turn
                .round
                .saturating_mul(16)
                .saturating_add(tx.state().turn.current.raw());
            let instance = tx
                .state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::Illegal("昭文 skill disappeared".into()))?;
            let old = instance.tracked_cards();
            instance.take_tracked_cards(&old);
            if !instance.add_tracked_cards(&hand) {
                return Err(EffectError::ContentFault(
                    "昭文 cannot store tracked hand cards".into(),
                ));
            }
            instance.view_as_use_round = token;
            instance.view_as_used_branches = 0;
            Ok(Vec::new())
        }
        1 => {
            let tracked = tx
                .state()
                .skills
                .get(skill)
                .map_or_else(Vec::new, |skill| skill.tracked_cards());
            let count = cards
                .into_iter()
                .filter(|used| tracked.contains(used))
                .filter(|used| {
                    tx.state().cards.get(*used).is_some_and(|card| {
                        matches!(
                            crate::query::effective_card_suit(tx.state(), player, card),
                            engine_api::Suit::Heart | engine_api::Suit::Diamond
                        )
                    })
                })
                .count()
                .min(u32::MAX as usize) as u32;
            if count == 0 {
                return Ok(Vec::new());
            }
            run_effects(tx, vec![EffectRequest::DrawCards { player, count }])
        }
        2 => {
            if !card.is_some()
                || !tx.state().cards.get(card).is_some_and(|card| {
                    crate::query::effective_card_kind(tx.state(), source, card) == CardKind::Jiu
                })
            {
                return Ok(Vec::new());
            }
            let before = tx.state().resolution.deferred_tail_effects.len();
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .retain(|effect| match effect {
                    EffectRequest::RecoverHp {
                        source: effect_source,
                        player: target,
                        ..
                    } => !(*effect_source == source && *target == player),
                    EffectRequest::AddMark {
                        player: target,
                        mark,
                        ..
                    } => !(*target == player && mark.raw() == crate::content::MARK_JIU),
                    _ => true,
                });
            if before == tx.state().resolution.deferred_tail_effects.len() {
                return Err(EffectError::Illegal(
                    "酒遁 found no current Jiu effect to rewrite".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::AddMark {
                    player,
                    mark: engine_api::MarkDefId::new(crate::content::MARK_JIU),
                    delta: 1,
                }],
            )
        }
        3 => {
            if source == player || !card.is_some() {
                return Ok(Vec::new());
            }
            if ruanji_has_jiu(tx, player) {
                let options = ruanji_discardable_hand(tx, player);
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
                    ContinuationKind::Custom,
                    vec![
                        RESOLVE_RUANJI_CHOICE,
                        player.raw(),
                        source.raw(),
                        card.raw(),
                        skill.raw(),
                        1,
                    ],
                )
            } else {
                open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    vec![0, 1],
                    ChoiceKind::Confirm,
                    ContinuationKind::Custom,
                    vec![
                        RESOLVE_RUANJI_CHOICE,
                        player.raw(),
                        source.raw(),
                        card.raw(),
                        skill.raw(),
                        0,
                    ],
                )
            }
        }
        4 => {
            crate::command::play::exclude_saved_card_target(tx, card, player)?;
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault("bad Ruanji action".into())),
    }
}

pub(crate) fn zhengbi_payment_cards(
    tx: &Transaction,
    target: PlayerId,
    non_basic: bool,
) -> Vec<CardId> {
    let hand = tx.state().zones.find(ZoneKind::Hand, target);
    crate::query::discardable_hand_equip_cards(tx.state(), target)
        .into_iter()
        .filter(|card| {
            tx.state().cards.get(*card).is_some_and(|card| {
                let basic = crate::query::effective_card_kind(tx.state(), target, card).is_basic();
                if non_basic {
                    !basic
                } else {
                    basic && Some(card.zone) == hand
                }
            })
        })
        .collect()
}

pub(crate) fn write_zhengbi_target(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    target: PlayerId,
) {
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob = [b"ZBI1".as_slice(), &target.raw().to_le_bytes()].concat();
    }
}

pub(crate) fn take_zhengbi_target(tx: &mut Transaction, skill: engine_api::SkillId) -> PlayerId {
    let target = tx
        .state()
        .skills
        .get(skill)
        .filter(|skill| skill.state_blob.len() == 8 && &skill.state_blob[..4] == b"ZBI1")
        .map(|skill| {
            PlayerId::new(u32::from_le_bytes(
                skill.state_blob[4..8].try_into().unwrap(),
            ))
        })
        .unwrap_or(PlayerId::NONE);
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob.clear();
    }
    target
}
