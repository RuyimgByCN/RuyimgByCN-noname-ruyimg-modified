//! Command operations for `resolve_twchenzhen` and related helpers.

use super::*;

pub(crate) fn resolve_twshanghe(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal("bad TW Shanghe card choice".into()));
    }
    let player = PlayerId::new(payload[0]);
    let giver = PlayerId::new(payload[1]);
    let skill = SkillId::new(payload[2]);
    let count = payload[4] as usize;
    let remaining: Vec<_> = payload
        .get(5..5 + count)
        .ok_or_else(|| EffectError::EngineFault("truncated TW Shanghe queue".into()))?
        .iter()
        .copied()
        .map(PlayerId::new)
        .collect();
    let card = CardId::new(selections[0]);
    let in_giver_zone = [ZoneKind::Hand, ZoneKind::Equip].into_iter().any(|kind| {
        tx.state().zones.find(kind, giver).is_some_and(|zone| {
            tx.state()
                .cards
                .get(card)
                .is_some_and(|card_state| card_state.owner == giver && card_state.zone == zone)
        })
    });
    if observer != giver
        || !in_giver_zone
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|owner| owner.alive)
    {
        return Err(EffectError::Illegal("TW Shanghe context changed".into()));
    }
    let card_state = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("TW Shanghe card missing".into()))?;
    let from = card_state.zone;
    let got_jiu = payload[3] != 0
        || crate::query::effective_card_kind(tx.state(), player, card_state) == CardKind::Jiu;
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("TW Shanghe owner hand missing".into()))?;
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveTwshanghe {
            player,
            stage: 1,
            remaining,
            got_jiu,
            skill,
        });
    run_effects(
        tx,
        vec![EffectRequest::GiveCards {
            cards: vec![card],
            to,
            from,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_twwangchang(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let mode = *payload.first().unwrap_or(&u32::MAX);
    let player = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let skill = SkillId::new(if mode == 3 {
        *payload.get(2).unwrap_or(&0)
    } else {
        *payload.get(4).unwrap_or(&0)
    });
    if observer != player
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal("TW Wangchang context changed".into()));
    }
    if mode == 3 {
        if cancelled || payload.len() != 3 || selections.is_empty() {
            return Err(EffectError::Illegal("bad TW Kaiji target choice".into()));
        }
        let max = 1 + tx.state().lifetime_dying_players.len();
        let targets: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
        let mut unique = targets.clone();
        unique.sort_unstable_by_key(|target| target.raw());
        unique.dedup();
        if targets.len() > max
            || unique.len() != targets.len()
            || targets.iter().any(|target| {
                !tx.state()
                    .players
                    .get(*target)
                    .is_some_and(|participant| participant.alive)
            })
        {
            return Err(EffectError::Illegal("invalid TW Kaiji targets".into()));
        }
        return run_effects(
            tx,
            vec![EffectRequest::ResolveTwwangchang {
                player,
                target: PlayerId::NONE,
                card: CardId::NONE,
                selected_card: CardId::NONE,
                action: 0,
                stage: 1,
                targets,
                skill,
            }],
        )
        .map(|_| ());
    }
    if payload.len() != 5 {
        return Err(EffectError::EngineFault(
            "bad TW Shepan continuation".into(),
        ));
    }
    let target = PlayerId::new(payload[2]);
    let card = CardId::new(payload[3]);
    match mode {
        0 => {
            if cancelled || selections.len() != 1 || !matches!(selections[0], 0 | 1) {
                return Err(EffectError::Illegal("bad TW Shepan branch choice".into()));
            }
            if selections[0] == 0 {
                return run_effects(
                    tx,
                    vec![EffectRequest::ResolveTwwangchang {
                        player,
                        target,
                        card,
                        selected_card: CardId::NONE,
                        action: 1,
                        stage: 2,
                        targets: Vec::new(),
                        skill,
                    }],
                )
                .map(|_| ());
            }
            let options =
                crate::command::play::target_zone_card_tokens(tx, target, true, true, true);
            if options.is_empty() {
                return Err(EffectError::Illegal("TW Shepan target region empty".into()));
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwwangchang,
                vec![1, player.raw(), target.raw(), card.raw(), skill.raw()],
            )?;
            Ok(())
        }
        1 => {
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("bad TW Shepan card choice".into()));
            }
            let selected_card = CardId::new(selections[0]);
            if !crate::command::play::target_zone_card_tokens(tx, target, true, true, true)
                .contains(&selected_card.raw())
            {
                return Err(EffectError::Illegal("invalid TW Shepan target card".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwwangchang {
                    player,
                    target,
                    card,
                    selected_card,
                    action: 1,
                    stage: 4,
                    targets: Vec::new(),
                    skill,
                }],
            )
            .map(|_| ())
        }
        2 => {
            if cancelled {
                return Ok(());
            }
            if selections != [1] {
                return Err(EffectError::Illegal("bad TW Shepan nullify choice".into()));
            }
            crate::command::play::exclude_saved_card_target(tx, card, player)
        }
        _ => Err(EffectError::EngineFault(
            "unknown TW Wangchang continuation".into(),
        )),
    }
}

pub(crate) fn resolve_twcaozhao(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let mode = *payload.first().unwrap_or(&u32::MAX);
    let player = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let skill = SkillId::new(*payload.last().unwrap_or(&0));
    if !tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    }) {
        return Err(EffectError::Illegal("TW Caozhao context changed".into()));
    }
    match mode {
        0 => {
            if observer != player || cancelled || payload.len() != 5 || selections.len() != 1 {
                return Err(EffectError::Illegal("bad TW Fuzuan target choice".into()));
            }
            let target = PlayerId::new(selections[0]);
            let defs = crate::query::conversion_skill_defs(tx.state(), target);
            if defs.is_empty() {
                return Err(EffectError::Illegal(
                    "TW Fuzuan target has no conversion skill".into(),
                ));
            }
            if defs.len() == 1 {
                return run_effects(
                    tx,
                    vec![EffectRequest::ResolveTwcaozhao {
                        player,
                        target,
                        source: PlayerId::NONE,
                        card: CardId::NONE,
                        selected_skill_def: defs[0].raw(),
                        twfuzuan_skill_def: payload[2],
                        twfeifu_skill_def: payload[3],
                        action: 0,
                        stage: 1,
                        skill,
                    }],
                )
                .map(|_| ());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                defs.into_iter().map(engine_api::SkillDefId::raw).collect(),
                ChoiceKind::Custom,
                ContinuationKind::ResolveTwcaozhao,
                vec![
                    1,
                    player.raw(),
                    target.raw(),
                    payload[2],
                    payload[3],
                    skill.raw(),
                ],
            )?;
            Ok(())
        }
        1 => {
            if observer != player || cancelled || payload.len() != 6 || selections.len() != 1 {
                return Err(EffectError::Illegal("bad TW Fuzuan skill choice".into()));
            }
            let target = PlayerId::new(payload[2]);
            if !crate::query::conversion_skill_defs(tx.state(), target)
                .iter()
                .any(|skill| skill.raw() == selections[0])
            {
                return Err(EffectError::Illegal(
                    "invalid TW Fuzuan conversion skill".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwcaozhao {
                    player,
                    target,
                    source: PlayerId::NONE,
                    card: CardId::NONE,
                    selected_skill_def: selections[0],
                    twfuzuan_skill_def: payload[3],
                    twfeifu_skill_def: payload[4],
                    action: 0,
                    stage: 1,
                    skill,
                }],
            )
            .map(|_| ())
        }
        2 => {
            if observer != player || payload.len() != 5 {
                return Err(EffectError::Illegal("bad TW Chongqi target choice".into()));
            }
            if cancelled {
                return if selections.is_empty() {
                    Ok(())
                } else {
                    Err(EffectError::Illegal(
                        "cancelled TW Chongqi has a target".into(),
                    ))
                };
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal("TW Chongqi needs one target".into()));
            }
            let target = PlayerId::new(selections[0]);
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("invalid TW Chongqi target".into()));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::ChangeMaxHp { player, delta: -1 },
                    EffectRequest::AddSkill {
                        player: target,
                        skill_def: engine_api::SkillDefId::new(payload[2]),
                    },
                ],
            )
            .map(|_| ())
        }
        3 => {
            if payload.len() != 6 || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("bad TW Feifu card choice".into()));
            }
            let giver = PlayerId::new(payload[2]);
            let source = PlayerId::new(payload[3]);
            if observer != giver
                || !tx
                    .state()
                    .players
                    .get(source)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("TW Feifu participants changed".into()));
            }
            let card = CardId::new(selections[0]);
            let from = tx
                .state()
                .cards
                .get(card)
                .map(|state| state.zone)
                .ok_or_else(|| EffectError::Illegal("TW Feifu selected card missing".into()))?;
            if ![ZoneKind::Hand, ZoneKind::Equip]
                .into_iter()
                .any(|kind| tx.state().zones.find(kind, giver) == Some(from))
            {
                return Err(EffectError::Illegal("TW Feifu card left giver".into()));
            }
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, source)
                .ok_or_else(|| {
                    EffectError::EngineFault("TW Feifu recipient hand missing".into())
                })?;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveTwcaozhao {
                    player,
                    target: giver,
                    source,
                    card,
                    selected_skill_def: 0,
                    twfuzuan_skill_def: 0,
                    twfeifu_skill_def: 0,
                    action: 2,
                    stage: 1,
                    skill,
                });
            run_effects(
                tx,
                vec![EffectRequest::GiveCards {
                    cards: vec![card],
                    to,
                    from,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown TW Caozhao continuation".into(),
        )),
    }
}

pub(crate) fn resolve_twguohuai(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 6 || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal("bad TW Guohuai choice".into()));
    }
    let mode = payload[0];
    let player = PlayerId::new(payload[1]);
    let source = PlayerId::new(payload[2]);
    let mark_def = payload[3];
    let helper_skill_def = payload[4];
    let skill = SkillId::new(payload[5]);
    if observer != player
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal("TW Guohuai context changed".into()));
    }
    match (mode, selections[0]) {
        (0, 1) => run_effects(
            tx,
            vec![
                EffectRequest::RemoveMark {
                    player,
                    mark: engine_api::MarkDefId::new(mark_def),
                    amount: 1,
                },
                EffectRequest::AddSkillUntilTurnEnd {
                    player: source,
                    skill_def: engine_api::SkillDefId::new(helper_skill_def),
                },
            ],
        )
        .map(|_| ()),
        (0, 2) => {
            let cards = crate::query::discardable_hand_equip_cards(tx.state(), source);
            if cards.is_empty() {
                return Err(EffectError::Illegal(
                    "TW Yuzhang source has no cards".into(),
                ));
            }
            let count = cards.len().min(2) as u8;
            run_effects(
                tx,
                vec![EffectRequest::RemoveMark {
                    player,
                    mark: engine_api::MarkDefId::new(mark_def),
                    amount: 1,
                }],
            )?;
            open_choice(
                tx,
                source,
                count,
                count,
                false,
                cards.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwguohuai,
                vec![
                    1,
                    player.raw(),
                    source.raw(),
                    mark_def,
                    helper_skill_def,
                    skill.raw(),
                ],
            )?;
            Ok(())
        }
        (1, _) => {
            let allowed: std::collections::BTreeSet<_> =
                crate::query::discardable_hand_equip_cards(tx.state(), source)
                    .into_iter()
                    .collect();
            let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
            if cards.len() != allowed.len().min(2)
                || cards.iter().any(|card| !allowed.contains(card))
            {
                return Err(EffectError::Illegal("invalid TW Yuzhang discard".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: source,
                    cards,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal("invalid TW Guohuai branch".into())),
    }
}

pub(crate) fn resolve_twchengpu_custom(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&crate::effect::TWCHENGPU_CUSTOM_CHOICE)
        || cancelled
        || selections.len() != 1
    {
        return Err(EffectError::Illegal("bad TW Chengpu choice".into()));
    }
    let mode = *payload.get(1).unwrap_or(&u32::MAX);
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = SkillId::new(*payload.last().unwrap_or(&0));
    if !tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    }) {
        return Err(EffectError::Illegal("TW Chengpu context changed".into()));
    }
    match mode {
        0 => {
            if observer != player || payload.len() != 5 {
                return Err(EffectError::Illegal("bad TW Chunlao target choice".into()));
            }
            let target = PlayerId::new(selections[0]);
            let cards = crate::effect::twchengpu_field_cards(tx.state(), target);
            if cards.is_empty() {
                return Err(EffectError::Illegal(
                    "TW Chunlao target has no field card".into(),
                ));
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                cards.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    crate::effect::TWCHENGPU_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    target.raw(),
                    payload[3],
                    skill.raw(),
                ],
            )?;
            Ok(())
        }
        1 => {
            if observer != player || payload.len() != 6 {
                return Err(EffectError::Illegal("bad TW Chunlao card choice".into()));
            }
            let target = PlayerId::new(payload[3]);
            let skill_def = payload[4];
            let card = CardId::new(selections[0]);
            if !crate::effect::twchengpu_field_cards(tx.state(), target).contains(&card) {
                return Err(EffectError::Illegal("TW Chunlao card moved".into()));
            }
            let from = tx.state().cards.get(card).unwrap().zone;
            let to = crate::effect::ensure_skill_pile_zone(tx, target, skill_def)?;
            crate::effect::move_cards_with_reason(
                tx,
                vec![card],
                to,
                Some(from),
                engine_api::CardMoveReason::Give,
            )?;
            Ok(())
        }
        2 => {
            if payload.len() != 7 {
                return Err(EffectError::Illegal("bad TW Chunlao gift choice".into()));
            }
            let source = PlayerId::new(payload[3]);
            let sha = CardId::new(payload[4]);
            let card = CardId::new(selections[0]);
            if observer != source
                || !crate::effect::twchengpu_hand_equip_cards(tx.state(), source).contains(&card)
            {
                return Err(EffectError::Illegal("TW Chunlao gift card moved".into()));
            }
            let from = tx.state().cards.get(card).unwrap().zone;
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("TW Chunlao owner hand missing".into()))?;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveTwchengpu {
                    player,
                    target: source,
                    source,
                    card: sha,
                    chunlao_skill_def: payload[5],
                    action: 2,
                    stage: 1,
                    selected_card: card,
                    skill,
                });
            run_effects(
                tx,
                vec![EffectRequest::GiveCards {
                    cards: vec![card],
                    to,
                    from,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal(
            "unknown TW Chengpu custom choice".into(),
        )),
    }
}

pub(crate) fn resolve_twcaoang_custom(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 6
        || payload.first() != Some(&crate::effect::TWCAOANG_CUSTOM_CHOICE)
        || selections.len() != 1
    {
        return Err(EffectError::Illegal("bad TW Caoang horse choice".into()));
    }
    let stage = payload[1];
    let player = PlayerId::new(payload[2]);
    let target = PlayerId::new(payload[3]);
    let helper_skill_def = payload[4];
    let skill = SkillId::new(payload[5]);
    if observer != player
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|target| target.alive)
    {
        return Err(EffectError::Illegal(
            "TW Caoang horse context changed".into(),
        ));
    }
    if stage == 0 {
        if cancelled || selections == [0] {
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .clear_twxiaolian_redirect();
            return Ok(());
        }
        if selections != [1] {
            return Err(EffectError::Illegal("bad TW Caoang confirmation".into()));
        }
        let options = crate::effect::twchengpu_hand_equip_cards(tx.state(), player);
        let mut next = payload.to_vec();
        next[1] = 1;
        return crate::effect::open_choice(
            tx,
            player,
            1,
            1,
            false,
            options.into_iter().map(CardId::raw).collect(),
            engine_api::ChoiceKind::SelectCards,
            ContinuationKind::Custom,
            next,
        )
        .map(|_| ());
    }
    if stage != 1 || cancelled {
        return Err(EffectError::Illegal("bad TW Caoang horse stage".into()));
    }
    let card = CardId::new(selections[0]);
    if !crate::effect::twchengpu_hand_equip_cards(tx.state(), player).contains(&card) {
        return Err(EffectError::Illegal("TW Caoang horse card moved".into()));
    }
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: target,
            skill_def: engine_api::SkillDefId::new(helper_skill_def),
        }],
    )?;
    let from = tx.state().cards.get(card).unwrap().zone;
    let to = crate::effect::ensure_skill_pile_zone(tx, target, helper_skill_def)?;
    crate::effect::move_cards_with_reason(
        tx,
        vec![card],
        to,
        Some(from),
        engine_api::CardMoveReason::Unspecified,
    )?;
    tx.state_mut()
        .skills
        .get_mut(skill)
        .unwrap()
        .clear_twxiaolian_redirect();
    Ok(())
}
