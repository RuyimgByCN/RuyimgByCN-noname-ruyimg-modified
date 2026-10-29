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
