//! Command operations for `resolve_jiebing` and related helpers.

use super::*;

pub(crate) fn resolve_jiebing(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [marker, player_raw, source_raw, skill_raw] = payload else {
        return Err(EffectError::EngineFault("bad Jiebing continuation".into()));
    };
    if *marker != crate::effect::JIEBING_CUSTOM_CHOICE || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal("Jiebing requires one target".into()));
    }
    let player = PlayerId::new(*player_raw);
    let source = PlayerId::new(*source_raw);
    let target = PlayerId::new(selections[0]);
    let skill = SkillId::new(*skill_raw);
    if observer != player
        || target == player
        || target == source
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal("Jiebing context changed".into()));
    }
    let cards = crate::query::gainable_target_cards(
        tx.state(),
        player,
        target,
        rule_api::TargetCardScope::HandEquip,
    );
    if cards.is_empty() {
        return Err(EffectError::Illegal(
            "Jiebing target has no gainable card".into(),
        ));
    }
    let mut rng = crate::rng::RuleRng::from_state(tx.state().rng);
    let card = cards[rng.gen_range_u32(cards.len() as u32) as usize];
    tx.state_mut().rng = rng.into_state();
    let card_state = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("Jiebing card missing".into()))?;
    let from = card_state.zone;
    let equip = card_state.kind.is_equip();
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("Jiebing owner hand missing".into()))?;
    if equip {
        tx.state_mut().resolution.deferred_tail_effects.push(
            EffectRequest::ChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip {
                player,
                source,
                card,
                stage: 1,
                skill,
            },
        );
    }
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: hand,
            from: Some(from),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_mobiledanshou(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [marker, player_raw, target_raw, skill_raw, count] = payload else {
        return Err(EffectError::EngineFault(
            "bad Mobile Danshou continuation".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    if *marker != crate::effect::MOBILEDANSHOU_CUSTOM_CHOICE || observer != player {
        return Err(EffectError::Illegal(
            "invalid Mobile Danshou chooser".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    let target = PlayerId::new(*target_raw);
    let skill = SkillId::new(*skill_raw);
    if selections.len() != *count as usize
        || tx.state().turn.current != target
        || tx.state().turn.phase != Phase::End
        || tx.state().turn.card_target_count(target, player) != *count
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "Mobile Danshou context changed".into(),
        ));
    }
    let allowed: std::collections::BTreeSet<_> =
        crate::query::discardable_hand_equip_cards(tx.state(), player)
            .into_iter()
            .map(CardId::raw)
            .collect();
    let mut unique = std::collections::BTreeSet::new();
    if !selections
        .iter()
        .all(|card| allowed.contains(card) && unique.insert(*card))
    {
        return Err(EffectError::Illegal(
            "Mobile Danshou discard cards changed".into(),
        ));
    }
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::DealDamage {
            source: player,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        });
    run_effects(
        tx,
        vec![EffectRequest::DiscardCards {
            player,
            cards: selections.iter().copied().map(CardId::new).collect(),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_jsrgchiying(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 7 {
        return Err(EffectError::EngineFault(
            "bad JSRG Chiying continuation".into(),
        ));
    }
    let remaining_len = payload[5] as usize;
    if payload.len() < 7 + remaining_len {
        return Err(EffectError::EngineFault(
            "bad JSRG Chiying remaining players".into(),
        ));
    }
    let accum_len = payload[6 + remaining_len] as usize;
    if payload.len() != 7 + remaining_len + accum_len {
        return Err(EffectError::EngineFault(
            "bad JSRG Chiying accumulated cards".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    let skill = SkillId::new(payload[3]);
    let discarder = PlayerId::new(payload[4]);
    if payload[0] != crate::effect::RESOLVE_JSRGCHIYING_CUSTOM_CHOICE
        || observer != discarder
        || cancelled
        || selections.len() != 1
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("JSRG Chiying context changed".into()));
    }
    let card = CardId::new(selections[0]);
    if !crate::query::discardable_hand_equip_cards(tx.state(), discarder).contains(&card) {
        return Err(EffectError::Illegal(
            "JSRG Chiying discard card changed".into(),
        ));
    }
    let remaining = payload[6..6 + remaining_len]
        .iter()
        .copied()
        .map(PlayerId::new)
        .collect();
    let mut accum: Vec<CardId> = payload[7 + remaining_len..]
        .iter()
        .copied()
        .map(CardId::new)
        .collect();
    run_effects(
        tx,
        vec![EffectRequest::DiscardCards {
            player: discarder,
            cards: vec![card],
        }],
    )?;
    accum.push(card);
    run_effects(
        tx,
        vec![EffectRequest::ResolveJsrgchiying {
            player,
            target,
            started: true,
            remaining,
            accum,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn mbqingdao_payload(
    stage: u32,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    damaged: bool,
    skill: SkillId,
) -> Vec<u32> {
    vec![
        crate::effect::MBQINGDAO_CUSTOM_CHOICE,
        stage,
        player.raw(),
        source.raw(),
        card.raw(),
        u32::from(damaged),
        skill.raw(),
    ]
}

pub(crate) fn resolve_mbqingdao(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 7 || payload[0] != crate::effect::MBQINGDAO_CUSTOM_CHOICE {
        return Err(EffectError::EngineFault("bad 清蹈 continuation".into()));
    }
    let stage = payload[1];
    let player = PlayerId::new(payload[2]);
    let source = PlayerId::new(payload[3]);
    let card = CardId::new(payload[4]);
    let damaged = payload[5] != 0;
    let skill = SkillId::new(payload[6]);
    if observer != player
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal("清蹈 context changed".into()));
    }
    if cancelled {
        return Ok(());
    }
    match stage {
        0 => {
            let [branch] = selections else {
                return Err(EffectError::Illegal("清蹈 requires one branch".into()));
            };
            match (*branch, damaged) {
                (1, true) => run_effects(
                    tx,
                    vec![EffectRequest::GainCardByKind {
                        player,
                        kind: CardKind::Shan.tag(),
                    }],
                )
                .map(|_| ()),
                (1, false) => run_effects(
                    tx,
                    vec![EffectRequest::GainCardByKind {
                        player,
                        kind: CardKind::Sha.tag(),
                    }],
                )
                .map(|_| ()),
                (2, true) => {
                    let options: Vec<_> = tx
                        .state()
                        .players
                        .alive_by_seat()
                        .into_iter()
                        .filter(|target| {
                            !crate::command::play::discardable_target_zone_card_tokens(
                                tx, player, *target,
                            )
                            .is_empty()
                        })
                        .map(PlayerId::raw)
                        .collect();
                    open_choice(
                        tx,
                        player,
                        1,
                        1,
                        false,
                        options,
                        ChoiceKind::SelectTargets,
                        ContinuationKind::Custom,
                        mbqingdao_payload(1, player, source, card, damaged, skill),
                    )
                    .map(|_| ())
                }
                (2, false) => {
                    let options: Vec<_> = tx
                        .state()
                        .hand_cards(player)
                        .into_iter()
                        .filter(|candidate| {
                            crate::command::play::skill_card_target_spec_with_options(
                                tx, player, *candidate, true, true,
                            )
                            .is_ok()
                        })
                        .map(CardId::raw)
                        .collect();
                    open_choice(
                        tx,
                        player,
                        1,
                        1,
                        false,
                        options,
                        ChoiceKind::SelectCards,
                        ContinuationKind::Custom,
                        mbqingdao_payload(3, player, source, card, damaged, skill),
                    )
                    .map(|_| ())
                }
                _ => Err(EffectError::Illegal("invalid 清蹈 branch".into())),
            }
        }
        1 => {
            let [target] = selections else {
                return Err(EffectError::Illegal("清蹈 requires one target".into()));
            };
            let target = PlayerId::new(*target);
            let options =
                crate::command::play::discardable_target_zone_card_tokens(tx, player, target);
            if options.is_empty() {
                return Err(EffectError::Illegal(
                    "清蹈 target no longer has a card".into(),
                ));
            }
            let mut next = mbqingdao_payload(2, player, source, card, damaged, skill);
            next.push(target.raw());
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                next,
            )
            .map(|_| ())
        }
        2 => {
            if payload.len() != 8 || selections.len() != 1 {
                return Err(EffectError::EngineFault("bad 清蹈 discard payload".into()));
            }
            let target = PlayerId::new(payload[7]);
            let selected = CardId::new(selections[0]);
            if !crate::command::play::discardable_target_zone_card_tokens(tx, player, target)
                .contains(&selected.raw())
            {
                return Err(EffectError::Illegal("清蹈 discard changed".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: target,
                    cards: vec![selected],
                }],
            )
            .map(|_| ())
        }
        3 => {
            let [selected] = selections else {
                return Err(EffectError::Illegal("清蹈 requires one hand card".into()));
            };
            let selected = CardId::new(*selected);
            let spec = crate::command::play::skill_card_target_spec_with_options(
                tx, player, selected, true, true,
            )?;
            match spec {
                crate::command::play::SkillCardTargetSpec::Immediate(targets) => {
                    use_mbqingdao_card(tx, player, selected, &targets)
                }
                crate::command::play::SkillCardTargetSpec::Select { min, max, options } => {
                    let mut next = mbqingdao_payload(4, player, source, card, damaged, skill);
                    next.push(selected.raw());
                    open_choice(
                        tx,
                        player,
                        min,
                        max,
                        false,
                        options.into_iter().map(PlayerId::raw).collect(),
                        ChoiceKind::SelectTargets,
                        ContinuationKind::Custom,
                        next,
                    )
                    .map(|_| ())
                }
                crate::command::play::SkillCardTargetSpec::Pairs(pairs) => {
                    let mut next = mbqingdao_payload(5, player, source, card, damaged, skill);
                    next.push(selected.raw());
                    next.push(pairs.len() as u32);
                    for (left, right) in &pairs {
                        next.push(left.raw());
                        next.push(right.raw());
                    }
                    open_choice(
                        tx,
                        player,
                        1,
                        1,
                        false,
                        (1..=pairs.len() as u32).collect(),
                        ChoiceKind::Custom,
                        ContinuationKind::Custom,
                        next,
                    )
                    .map(|_| ())
                }
            }
        }
        4 => {
            if payload.len() != 8 {
                return Err(EffectError::EngineFault("bad 清蹈 target payload".into()));
            }
            let selected = CardId::new(payload[7]);
            use_mbqingdao_card(
                tx,
                player,
                selected,
                &selections
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect::<Vec<_>>(),
            )
        }
        5 => {
            if payload.len() < 9 || selections.len() != 1 {
                return Err(EffectError::EngineFault("bad 清蹈 pair payload".into()));
            }
            let selected = CardId::new(payload[7]);
            let count = payload[8] as usize;
            if payload.len() != 9 + count * 2 {
                return Err(EffectError::EngineFault("truncated 清蹈 pairs".into()));
            }
            let index = selections[0].checked_sub(1).map(|value| value as usize);
            let Some(index) = index.filter(|index| *index < count) else {
                return Err(EffectError::Illegal("invalid 清蹈 pair".into()));
            };
            use_mbqingdao_card(
                tx,
                player,
                selected,
                &[
                    PlayerId::new(payload[9 + index * 2]),
                    PlayerId::new(payload[10 + index * 2]),
                ],
            )
        }
        _ => Err(EffectError::EngineFault("unknown 清蹈 stage".into())),
    }
}

pub(crate) fn use_mbqingdao_card(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    targets: &[PlayerId],
) -> Result<(), EffectError> {
    let kind = tx
        .state()
        .cards
        .get(card)
        .map(|card| crate::query::effective_card_kind(tx.state(), player, card))
        .ok_or_else(|| EffectError::Illegal("清蹈 card disappeared".into()))?;
    crate::command::play::play_as_kind(tx, player, card, kind, targets, true, false, true, true)
}

pub(crate) fn resolve_mbxiugeng(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 5 || payload[0] != crate::effect::MBXIUGENG_CUSTOM_CHOICE {
        return Err(EffectError::EngineFault("bad 修耕 continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    if observer != player {
        return Err(EffectError::Illegal("修耕 observer changed".into()));
    }
    if cancelled {
        return Ok(());
    }
    if selections.is_empty() || selections.len() > 2 {
        return Err(EffectError::Illegal(
            "修耕 requires one or two targets".into(),
        ));
    }
    let targets: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
    if targets.iter().any(|target| {
        !tx.state()
            .players
            .get(*target)
            .is_some_and(|participant| participant.alive)
    }) {
        return Err(EffectError::Illegal("修耕 target changed".into()));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveMbxiugeng {
            player,
            helper_def: SkillDefId::new(payload[2]),
            hand_limit_def: SkillDefId::new(payload[3]),
            stage: 2,
            targets,
            skill: SkillId::new(payload[4]),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_mbchenshe(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 7 || payload[0] != crate::effect::MBCHENSHE_CUSTOM_CHOICE {
        return Err(EffectError::EngineFault("bad 陈赦 continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    let dying = PlayerId::new(payload[2]);
    let source = PlayerId::new(payload[3]);
    let skill = SkillId::new(payload[4]);
    let roles_len = payload[5] as usize;
    if payload.len() < 7 + roles_len {
        return Err(EffectError::EngineFault("truncated 陈赦 roles".into()));
    }
    let suits_len = payload[6 + roles_len] as usize;
    if payload.len() != 7 + roles_len + suits_len
        || observer != player
        || cancelled
        || selections.len() != 1
        || roles_len == 0
    {
        return Err(EffectError::Illegal("陈赦 context changed".into()));
    }
    let mut roles: Vec<_> = payload[6..6 + roles_len]
        .iter()
        .copied()
        .map(PlayerId::new)
        .collect();
    let target = roles.remove(0);
    let selected = CardId::new(selections[0]);
    let options = if target == player {
        crate::query::discardable_hand_equip_cards(tx.state(), player)
            .into_iter()
            .map(CardId::raw)
            .collect()
    } else {
        crate::command::play::discardable_target_zone_card_tokens(tx, player, target)
    };
    if !options.contains(&selected.raw()) {
        return Err(EffectError::Illegal("陈赦 discard changed".into()));
    }
    let suit = tx
        .state()
        .cards
        .get(selected)
        .map(|card| crate::command::judge::suit_tag(card.suit) as u8)
        .ok_or_else(|| EffectError::Illegal("陈赦 card disappeared".into()))?;
    let mut suits: Vec<_> = payload[7 + roles_len..]
        .iter()
        .copied()
        .map(|value| value as u8)
        .collect();
    suits.push(suit);
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveMbchenshe {
            player,
            dying,
            source,
            stage: 1,
            roles,
            suits,
            skill,
        });
    run_effects(
        tx,
        vec![EffectRequest::DiscardCards {
            player: target,
            cards: vec![selected],
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_mbrenxing(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5
        || payload[0] != crate::effect::MBRENXING_CUSTOM_CHOICE
        || observer != PlayerId::new(payload[2])
        || cancelled
        || selections.len() != 1
    {
        return Err(EffectError::Illegal("任行 context changed".into()));
    }
    let stage = payload[1];
    let player = PlayerId::new(payload[2]);
    let current = PlayerId::new(payload[3]);
    let skill = SkillId::new(payload[4]);
    let targets = &payload[5..];
    match (stage, selections[0]) {
        (0, 1) => {
            let mut effects = vec![EffectRequest::DrawCards { player, count: 1 }];
            if current != player
                && tx
                    .state()
                    .players
                    .get(current)
                    .is_some_and(|participant| participant.alive)
            {
                effects.push(EffectRequest::DrawCards {
                    player: current,
                    count: 1,
                });
            }
            run_effects(tx, effects).map(|_| ())
        }
        (0, 2) if !targets.is_empty() => open_choice(
            tx,
            player,
            1,
            1,
            false,
            targets.to_vec(),
            ChoiceKind::SelectTargets,
            ContinuationKind::Custom,
            vec![
                crate::effect::MBRENXING_CUSTOM_CHOICE,
                1,
                player.raw(),
                current.raw(),
                skill.raw(),
            ],
        )
        .map(|_| ()),
        (1, target) => run_effects(
            tx,
            vec![EffectRequest::ResolveMbrenxing {
                player,
                stage: 1,
                target: PlayerId::new(target),
                skill,
            }],
        )
        .map(|_| ()),
        _ => Err(EffectError::Illegal("invalid 任行 branch".into())),
    }
}

pub(crate) fn resolve_mbchizhang(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 || payload[0] != crate::effect::MBCHIZHANG_CUSTOM_CHOICE {
        return Err(EffectError::EngineFault("bad 鸱张 continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    if observer != player {
        return Err(EffectError::Illegal("鸱张 observer changed".into()));
    }
    if cancelled {
        return Ok(());
    }
    if selections.is_empty() {
        return Err(EffectError::Illegal(
            "鸱张 requires at least one card".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveMbchizhang {
            player,
            card: CardId::new(payload[2]),
            stage: 1,
            cards: selections.iter().copied().map(CardId::new).collect(),
            skill: SkillId::new(payload[3]),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_mbduanyang(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 5
        || payload[0] != crate::effect::MBDUANYANG_CUSTOM_CHOICE
        || observer != PlayerId::new(payload[2])
    {
        return Err(EffectError::EngineFault("bad 断鞅 continuation".into()));
    }
    if cancelled {
        return if payload[1] == 2 {
            Ok(())
        } else {
            Err(EffectError::Illegal("断鞅 cannot cancel".into()))
        };
    }
    if selections.is_empty() || (payload[1] == 1 && selections.len() != 1) || selections.len() > 2 {
        return Err(EffectError::Illegal("invalid 断鞅 selection".into()));
    }
    let player = PlayerId::new(payload[2]);
    let (target, card, mode) = match payload[1] {
        1 => (PlayerId::new(selections[0]), CardId::new(payload[3]), 1),
        2 => (PlayerId::new(payload[3]), CardId::NONE, 2),
        _ => return Err(EffectError::EngineFault("unknown 断鞅 stage".into())),
    };
    run_effects(
        tx,
        vec![EffectRequest::ResolveMbduanyang {
            player,
            target,
            card,
            mode,
            stage: 1,
            cards: if mode == 2 {
                selections.iter().copied().map(CardId::new).collect()
            } else {
                Vec::new()
            },
            skill: SkillId::new(payload[4]),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_mbrunwei(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4
        || payload[0] != crate::effect::MBRUNWEI_CUSTOM_CHOICE
        || observer != PlayerId::new(payload[2])
    {
        return Err(EffectError::EngineFault("bad 润微 continuation".into()));
    }
    let stage = payload[1] as u8;
    let player = PlayerId::new(payload[2]);
    let skill = SkillId::new(payload[3]);
    if cancelled {
        if stage == 0 && selections.is_empty() {
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.play_phase_use_count = instance.play_phase_use_count.saturating_sub(1);
            }
            return Ok(());
        }
        return Err(EffectError::Illegal("润微 cannot cancel now".into()));
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal("润微 requires one selection".into()));
    }
    let (target, value, cards, next_stage) = match stage {
        0 if (1..=5).contains(&selections[0]) => (PlayerId::NONE, selections[0], Vec::new(), 1),
        1 if matches!(selections[0], 1 | 2) => (
            PlayerId::NONE,
            selections[0],
            payload[4..].iter().copied().map(CardId::new).collect(),
            2,
        ),
        2 => (
            PlayerId::new(selections[0]),
            *payload.get(4).unwrap_or(&0),
            payload[5..].iter().copied().map(CardId::new).collect(),
            3,
        ),
        _ => return Err(EffectError::Illegal("invalid 润微 branch".into())),
    };
    run_effects(
        tx,
        vec![EffectRequest::ResolveMbrunwei {
            player,
            target,
            stage: next_stage,
            value,
            cards,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_mbshuanghuai(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4
        || payload[0] != crate::effect::MBSHUANGHUAI_CUSTOM_CHOICE
        || observer != PlayerId::new(payload[1])
        || cancelled
        || selections.len() != 1
    {
        return Err(EffectError::Illegal("bad 霜怀 continuation".into()));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveMbshuanghuai {
            player: PlayerId::new(payload[1]),
            target: PlayerId::new(payload[2]),
            stage: 1,
            choice: selections[0] as u8,
            skill: SkillId::new(payload[3]),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_mbxiongtu(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5
        || payload[0] != crate::effect::MBXIONGTU_CUSTOM_CHOICE
        || observer != PlayerId::new(payload[2])
    {
        return Err(EffectError::EngineFault("bad 凶图 continuation".into()));
    }
    let player = PlayerId::new(payload[2]);
    let target = PlayerId::new(payload[3]);
    let (stage, card, cards, skill) = match payload[1] {
        0 if !cancelled && selections.len() == 1 => (
            1,
            CardId::new(selections[0]),
            Vec::new(),
            SkillId::new(payload[4]),
        ),
        1 if cancelled && selections.is_empty() => (
            3,
            CardId::new(payload[4]),
            Vec::new(),
            SkillId::new(payload[5]),
        ),
        1 if !cancelled && !selections.is_empty() => (
            2,
            CardId::new(payload[4]),
            selections.iter().copied().map(CardId::new).collect(),
            SkillId::new(payload[5]),
        ),
        _ => return Err(EffectError::Illegal("invalid 凶图 selection".into())),
    };
    run_effects(
        tx,
        vec![EffectRequest::ResolveMbxiongtu {
            player,
            target,
            stage,
            card,
            cards,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_mbsijian(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5
        || payload[0] != crate::effect::MBSIJIAN_CUSTOM_CHOICE
        || observer != PlayerId::new(payload[2])
    {
        return Err(EffectError::EngineFault("bad 死谏 continuation".into()));
    }
    let player = PlayerId::new(payload[2]);
    if payload[1] == 0 && cancelled && selections.is_empty() {
        let skill = SkillId::new(payload[4]);
        if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
            instance.turn_fire_count = instance.turn_fire_count.saturating_sub(1);
        }
        return Ok(());
    }
    let (stage, target, choice, helper_def, skill) = match payload[1] {
        0 if !cancelled && selections.len() == 1 && matches!(selections[0], 1 | 2 | 3) => (
            1,
            PlayerId::NONE,
            selections[0] as u8,
            SkillDefId::new(payload[3]),
            SkillId::new(payload[4]),
        ),
        1 if !cancelled && selections.len() == 1 => (
            2,
            PlayerId::new(selections[0]),
            payload[3] as u8,
            SkillDefId::new(payload[4]),
            SkillId::new(payload[5]),
        ),
        _ => return Err(EffectError::Illegal("invalid 死谏 selection".into())),
    };
    run_effects(
        tx,
        vec![EffectRequest::ResolveMbsijian {
            player,
            target,
            stage,
            choice,
            helper_def,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_mbfeijing(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 6 || payload[0] != crate::effect::MBFEIJING_CUSTOM_CHOICE {
        return Err(EffectError::EngineFault("bad 飞径 continuation".into()));
    }
    let player = PlayerId::new(payload[2]);
    match payload[1] {
        0 if observer == player && !cancelled && selections.len() == 1 => {
            let target = PlayerId::new(payload[3]);
            let card = CardId::new(payload[4]);
            let skill = SkillId::new(payload[5]);
            let mut order = tx.state().players.alive_by_seat();
            let start = order.iter().position(|role| *role == player);
            let end = order.iter().position(|role| *role == target);
            let (Some(start), Some(end)) = (start, end) else {
                return Err(EffectError::Illegal("飞径 endpoints changed".into()));
            };
            let step = match selections[0] {
                1 => 1,
                2 => -1,
                _ => return Err(EffectError::Illegal("invalid 飞径 path".into())),
            };
            let mut path = Vec::new();
            let mut index = start;
            loop {
                index = (index as isize + step).rem_euclid(order.len() as isize) as usize;
                if index == end {
                    break;
                }
                path.push(order[index]);
            }
            order.clear();
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::ResolveMbfeijing {
                    player,
                    target,
                    card,
                    stage: 1,
                    targets: path,
                    cards: Vec::new(),
                    skill,
                }],
            )
            .map(|_| ())
        }
        1 if !cancelled && selections.len() == 1 && payload.len() >= 7 => {
            let count = payload[6] as usize;
            if payload.len() < 7 + count {
                return Err(EffectError::EngineFault("truncated 飞径 path".into()));
            }
            let targets: Vec<_> = payload[7..7 + count]
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect();
            let mut cards: Vec<_> = payload[7 + count..]
                .iter()
                .copied()
                .map(CardId::new)
                .collect();
            let chooser = targets
                .get(cards.len())
                .copied()
                .ok_or_else(|| EffectError::Illegal("飞径 choice index overflow".into()))?;
            let selected = CardId::new(selections[0]);
            if observer != chooser
                || !crate::query::discardable_target_cards(
                    tx.state(),
                    chooser,
                    chooser,
                    rule_api::TargetCardScope::Hand,
                )
                .contains(&selected)
            {
                return Err(EffectError::Illegal("invalid 飞径 hand card".into()));
            }
            cards.push(selected);
            crate::effect::continue_mbfeijing_card_choices(
                tx,
                player,
                PlayerId::new(payload[3]),
                CardId::new(payload[4]),
                SkillId::new(payload[5]),
                targets,
                cards,
            )
            .map(|_| ())
        }
        2 if observer == player && payload.len() >= 6 => {
            if cancelled {
                return Ok(());
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal("invalid 飞径 color".into()));
            }
            let count = payload[5] as usize;
            if payload.len() != 6 + count * 2 {
                return Err(EffectError::EngineFault(
                    "truncated 飞径 color context".into(),
                ));
            }
            let targets = payload[6..6 + count]
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect();
            let cards = payload[6 + count..]
                .iter()
                .copied()
                .map(CardId::new)
                .collect();
            crate::effect::commit_mbfeijing_color(
                tx,
                player,
                CardId::new(payload[3]),
                SkillId::new(payload[4]),
                selections[0] as u8,
                targets,
                cards,
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal("invalid 飞径 selection".into())),
    }
}

pub(crate) fn resolve_mbxuye(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4
        || payload[0] != crate::effect::MBXUYE_CUSTOM_CHOICE
        || observer.raw() != payload[1]
        || cancelled
        || selections.len() != 1
    {
        return Err(EffectError::Illegal("invalid 蓄业 selection".into()));
    }
    let target = PlayerId::new(payload[2]);
    let card = CardId::new(selections[0]);
    let from = [ZoneKind::Hand, ZoneKind::Equip, ZoneKind::Judge]
        .into_iter()
        .filter_map(|kind| tx.state().zones.find(kind, target))
        .find(|zone| {
            tx.state()
                .zones
                .get(*zone)
                .is_some_and(|zone| zone.cards.contains(&card))
        })
        .ok_or_else(|| EffectError::Illegal("蓄业 card left target region".into()))?;
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("draw pile missing".into()))?;
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: draw,
            from: Some(from),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_spmiaolve_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 || observer.raw() != payload[1] {
        return Err(EffectError::Illegal("invalid 妙略 chooser".into()));
    }
    if cancelled {
        return Ok(());
    }
    let [choice] = selections else {
        return Err(EffectError::Illegal("妙略 requires one branch".into()));
    };
    if ![
        0,
        CardKind::TrickWuzhong.tag() as u32,
        CardKind::TrickGuohe.tag() as u32,
        CardKind::TrickWuxie.tag() as u32,
    ]
    .contains(choice)
    {
        return Err(EffectError::Illegal("invalid 妙略 branch".into()));
    }
    let player = PlayerId::new(payload[1]);
    run_effects(
        tx,
        vec![if *choice == 0 {
            EffectRequest::DrawCards { player, count: 2 }
        } else {
            EffectRequest::GainCardByKind {
                player,
                kind: *choice as u8,
            }
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_spyingjia_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 || observer.raw() != payload[2] {
        return Err(EffectError::Illegal("invalid 迎驾 chooser".into()));
    }
    let player = PlayerId::new(payload[2]);
    let helper_def = engine_api::SkillDefId::new(payload[3]);
    let skill = engine_api::SkillId::new(payload[4]);
    match payload[1] {
        0 => {
            if cancelled {
                return Ok(());
            }
            let [card_raw] = selections else {
                return Err(EffectError::Illegal("迎驾 requires one hand card".into()));
            };
            let card = CardId::new(*card_raw);
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            if tx
                .state()
                .cards
                .get(card)
                .is_none_or(|card| card.owner != player || Some(card.zone) != hand)
            {
                return Err(EffectError::Illegal("invalid 迎驾 discard card".into()));
            }
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(PlayerId::raw)
                .collect();
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
                    crate::effect::SPYINGJIA_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    helper_def.raw(),
                    skill.raw(),
                    card.raw(),
                ],
            )?;
            Ok(())
        }
        1 => {
            if payload.len() != 6 || cancelled {
                return Err(EffectError::Illegal("invalid 迎驾 target choice".into()));
            }
            let [target_raw] = selections else {
                return Err(EffectError::Illegal("迎驾 requires one target".into()));
            };
            let target = PlayerId::new(*target_raw);
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("迎驾 target is not alive".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveSpyingjia {
                    player,
                    action: 4,
                    helper_def,
                    card: CardId::new(payload[5]),
                    card_kind: 0,
                    target,
                    skill,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault("unknown 迎驾 choice stage".into())),
    }
}

pub(crate) fn resolve_pottuntian_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 || observer.raw() != payload[1] || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal("invalid 势屯田蓄力选择".into()));
    }
    let count = payload[4] as usize;
    if count == 0 || payload.len() != 5 + count {
        return Err(EffectError::EngineFault("malformed 势屯田目标列表".into()));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    let zaoxian_def = engine_api::SkillDefId::new(payload[3]);
    let amount = u8::try_from(selections[0])
        .map_err(|_| EffectError::Illegal("势屯田蓄力数量过大".into()))?;
    if usize::from(amount) < count {
        return Err(EffectError::Illegal("势屯田蓄力少于目标数".into()));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolvePottuntian {
            player,
            action: 3,
            amount,
            targets: payload[5..].iter().copied().map(PlayerId::new).collect(),
            zaoxian_def,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_mbzhonghui_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 5 || observer.raw() != payload[1] || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal("invalid 俟资蓄力选择".into()));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveMbzhonghui {
            player: PlayerId::new(payload[1]),
            target: PlayerId::NONE,
            action: 8,
            amount: selections[0],
            mbsizi_def: SkillDefId::new(payload[3]),
            mbkechang_def: SkillDefId::new(payload[4]),
            skill: SkillId::new(payload[2]),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_potjiejie_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 10 || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal("invalid 诫节花色选择".into()));
    }
    let player = PlayerId::new(payload[1]);
    let actor = PlayerId::new(payload[2]);
    let skill = SkillId::new(payload[3]);
    if observer != player
        || tx.state().turn.current != actor
        || tx.state().turn.phase != Phase::Play
        || !tx
            .state()
            .players
            .get(actor)
            .is_some_and(|state| state.alive)
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal("诫节上下文已变化".into()));
    }
    let suit_index = usize::try_from(selections[0])
        .ok()
        .filter(|index| *index < 4)
        .ok_or_else(|| EffectError::Illegal("invalid 诫节花色".into()))?;
    let suit = [
        engine_api::Suit::Spade,
        engine_api::Suit::Heart,
        engine_api::Suit::Club,
        engine_api::Suit::Diamond,
    ][suit_index];
    let hand = tx.state().hand_cards(actor);
    let has_suit = hand.iter().any(|card| {
        tx.state()
            .cards
            .get(*card)
            .is_some_and(|card| crate::query::effective_card_suit(tx.state(), actor, card) == suit)
    });
    let mut effects = Vec::new();
    if has_suit {
        effects.push(EffectRequest::AddSkillUntilTurnEnd {
            player: actor,
            skill_def: SkillDefId::new(payload[5 + suit_index]),
        });
        let discardable: std::collections::BTreeSet<_> =
            crate::query::discardable_hand_equip_cards(tx.state(), actor)
                .into_iter()
                .collect();
        let cards: Vec<_> = hand
            .into_iter()
            .filter(|card| {
                discardable.contains(card)
                    && tx.state().cards.get(*card).is_some_and(|card| {
                        crate::query::effective_card_suit(tx.state(), actor, card) != suit
                    })
            })
            .collect();
        if !cards.is_empty() {
            effects.push(EffectRequest::DiscardCards {
                player: actor,
                cards,
            });
        }
    } else {
        let draw = tx
            .state()
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::EngineFault("draw pile missing".into()))?;
        let card = tx.state().zones.get(draw).and_then(|zone| {
            zone.cards.iter().rev().copied().find(|card| {
                tx.state().cards.get(*card).is_some_and(|card| {
                    crate::query::effective_card_suit(tx.state(), actor, card) == suit
                })
            })
        });
        if let Some(card) = card {
            let hand_zone = tx
                .state()
                .zones
                .find(ZoneKind::Hand, actor)
                .ok_or_else(|| EffectError::EngineFault("hand zone missing".into()))?;
            effects.push(EffectRequest::MoveCards {
                cards: vec![card],
                to: hand_zone,
                from: Some(draw),
            });
        }
    }
    if payload[9] != 0 {
        let qingshi_def = SkillDefId::new(payload[4]);
        if let Some(qingshi) = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .find(|instance| {
                instance.owner == player
                    && instance.def == qingshi_def
                    && tx.state().skill_effectively_enabled(instance)
            })
            .map(|instance| instance.id)
        {
            effects.push(EffectRequest::ResolvePotqingshi {
                player,
                skill: qingshi,
            });
        }
    }
    run_effects(tx, effects).map(|_| ())
}
