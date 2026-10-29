//! Command operations for `resolve_resolve_dctingxian` and related helpers.

use super::*;

pub(crate) fn resolve_resolve_zhiba(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    // payload: [magic, player, actor, skill, stage(, winner, player_card, target_card)]
    if payload.len() < 5 {
        return Err(EffectError::EngineFault("bad Zhiba continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    if observer != player {
        return Err(EffectError::Illegal("Zhiba observer mismatch".into()));
    }
    let actor = PlayerId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    let stage = payload[4];
    if cancelled || selections.len() != 1 {
        return Ok(());
    }
    if stage == 0 {
        // 觉醒拒绝选择：1 = 拒绝（停止），0 = 继续拼点。
        if selections[0] == 1 {
            return Ok(());
        }
        tx.state_mut().skills.get_mut(skill).map(|instance| {
            instance.set_zhiba_compare(actor);
        });
        let options: Vec<_> = tx
            .state()
            .hand_cards(actor)
            .into_iter()
            .map(CardId::raw)
            .collect();
        return crate::effect::open_choice(
            tx,
            actor,
            1,
            1,
            false,
            options,
            engine_api::ChoiceKind::SelectCards,
            crate::state::ContinuationKind::ChooseHandComparePlayerCard,
            vec![
                actor.raw(),
                player.raw(),
                engine_api::SkillDefId::NONE.raw(),
                engine_api::SkillDefId::NONE.raw(),
                skill.raw(),
            ],
        )
        .map(|_| ());
    }
    // stage 1: 取牌选择；1 = 获得两张拼点牌。
    if payload.len() != 8 {
        return Err(EffectError::EngineFault("bad Zhiba settle payload".into()));
    }
    if selections[0] != 1 {
        tx.state_mut().skills.get_mut(skill).map(|instance| {
            instance.clear_zhiba_compare();
        });
        return Ok(());
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveZhibaSettle {
            player,
            actor,
            winner: PlayerId::new(payload[5]),
            player_card: engine_api::CardId::new(payload[6]),
            target_card: engine_api::CardId::new(payload[7]),
            stage: 1,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_resolve_tiaoxin(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.get(1).unwrap_or(&0);
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let mut target = PlayerId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    let alive = |id: PlayerId| {
        tx.state()
            .players
            .get(id)
            .is_some_and(|participant| participant.alive)
    };
    match stage {
        1 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "挑衅 target choice must be made by the owner".into(),
                ));
            }
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "挑衅 requires exactly one target".into(),
                ));
            }
            let chosen = PlayerId::new(selections[0]);
            if !alive(chosen)
                || crate::query::discardable_hand_equip_cards(tx.state(), chosen).is_empty()
            {
                return Err(EffectError::Illegal(
                    "挑衅 target is no longer legal".into(),
                ));
            }
            crate::effect::open_choice(
                tx,
                chosen,
                1,
                1,
                true,
                vec![1, 0],
                engine_api::ChoiceKind::Custom,
                crate::state::ContinuationKind::Custom,
                vec![
                    crate::effect::RESOLVE_TIAOXIN_CUSTOM_CHOICE,
                    2,
                    player.raw(),
                    chosen.raw(),
                    skill.raw(),
                ],
            )
            .map(|_| ())
        }
        2 => {
            if observer != target || !alive(target) {
                return Err(EffectError::Illegal(
                    "挑衅 response must come from the challenged player".into(),
                ));
            }
            if cancelled || selections.first().copied() != Some(1) {
                return crate::effect::open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    crate::query::discardable_hand_equip_cards(tx.state(), target)
                        .into_iter()
                        .map(engine_api::CardId::raw)
                        .collect(),
                    engine_api::ChoiceKind::SelectCards,
                    crate::state::ContinuationKind::Custom,
                    vec![
                        crate::effect::RESOLVE_TIAOXIN_CUSTOM_CHOICE,
                        4,
                        player.raw(),
                        target.raw(),
                        skill.raw(),
                    ],
                )
                .map(|_| ());
            }
            let cards = crate::command::play::prompted_sha_source_cards(tx.state(), target);
            if cards.is_empty() {
                return crate::effect::open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    crate::query::discardable_hand_equip_cards(tx.state(), target)
                        .into_iter()
                        .map(engine_api::CardId::raw)
                        .collect(),
                    engine_api::ChoiceKind::SelectCards,
                    crate::state::ContinuationKind::Custom,
                    vec![
                        crate::effect::RESOLVE_TIAOXIN_CUSTOM_CHOICE,
                        4,
                        player.raw(),
                        target.raw(),
                        skill.raw(),
                    ],
                )
                .map(|_| ());
            }
            crate::effect::open_choice(
                tx,
                target,
                0,
                1,
                true,
                cards.into_iter().map(engine_api::CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::Custom,
                vec![
                    crate::effect::RESOLVE_TIAOXIN_CUSTOM_CHOICE,
                    3,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )
            .map(|_| ())
        }
        3 => {
            if observer != target || !alive(target) {
                return Err(EffectError::Illegal(
                    "挑衅 Sha source must come from the challenged player".into(),
                ));
            }
            let mut discard_choice = || {
                crate::effect::open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    crate::query::discardable_hand_equip_cards(tx.state(), target)
                        .into_iter()
                        .map(engine_api::CardId::raw)
                        .collect(),
                    engine_api::ChoiceKind::SelectCards,
                    crate::state::ContinuationKind::Custom,
                    vec![
                        crate::effect::RESOLVE_TIAOXIN_CUSTOM_CHOICE,
                        4,
                        player.raw(),
                        target.raw(),
                        skill.raw(),
                    ],
                )
                .map(|_| ())
            };
            if cancelled || selections.is_empty() {
                return discard_choice();
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "choose at most one 挑衅 Sha source".into(),
                ));
            };
            let card = engine_api::CardId::new(*selected);
            if !crate::command::play::prompted_sha_source_cards(tx.state(), target).contains(&card)
            {
                return Err(EffectError::Illegal(
                    "挑衅 Sha source is no longer legal".into(),
                ));
            }
            crate::command::play::play_prompted_sha_source_unlimited(tx, target, card, &[player])
        }
        4 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "挑衅 discard choice must be made by the owner".into(),
                ));
            }
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "挑衅 requires one discard card".into(),
                ));
            }
            let card = engine_api::CardId::new(selections[0]);
            if !crate::query::discardable_hand_equip_cards(tx.state(), target).contains(&card) {
                return Err(EffectError::Illegal(
                    "挑衅 discard card is no longer legal".into(),
                ));
            }
            let zone = tx
                .state()
                .cards
                .get(card)
                .map(|value| value.zone)
                .ok_or_else(|| EffectError::Illegal("挑衅 discard card missing".into()))?;
            let discard = tx
                .state()
                .zones
                .find(crate::state::ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![card],
                    to: discard,
                    from: Some(zone),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault("unknown 挑衅 choice stage".into())),
    }
}

pub(crate) fn resolve_resolve_zhengrong(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.get(1).unwrap_or(&0);
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let target = PlayerId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    match stage {
        1 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "征荣 target choice must be made by the owner".into(),
                ));
            }
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("征荣 requires one target".into()));
            }
            let chosen = PlayerId::new(selections[0]);
            let owner_hand = tx.state().hand_count(player);
            if chosen == player
                || !tx.state().players.get(chosen).is_some_and(|p| p.alive)
                || tx.state().hand_count(chosen) < owner_hand
                || crate::query::discardable_hand_equip_cards(tx.state(), chosen).is_empty()
            {
                return Err(EffectError::Illegal(
                    "征荣 target is no longer legal".into(),
                ));
            }
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                crate::query::discardable_hand_equip_cards(tx.state(), chosen)
                    .into_iter()
                    .map(engine_api::CardId::raw)
                    .collect(),
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::Custom,
                vec![
                    crate::effect::RESOLVE_ZHENGRONG_CUSTOM_CHOICE,
                    2,
                    player.raw(),
                    chosen.raw(),
                    skill.raw(),
                ],
            )
            .map(|_| ())
        }
        2 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "征荣 card choice must be made by the owner".into(),
                ));
            }
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("征荣 requires one card".into()));
            }
            let card = engine_api::CardId::new(selections[0]);
            if !crate::query::discardable_hand_equip_cards(tx.state(), target).contains(&card) {
                return Err(EffectError::Illegal("征荣 card is no longer legal".into()));
            }
            let zone = tx
                .state()
                .cards
                .get(card)
                .map(|value| value.zone)
                .ok_or_else(|| EffectError::Illegal("征荣 card missing".into()))?;
            let skill_def = crate::effect::skill_pile_def(tx, player, skill)?;
            let pile = crate::effect::ensure_skill_pile_zone(tx, player, skill_def)?;
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![card],
                    to: pile,
                    from: Some(zone),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault("unknown 征荣 choice stage".into())),
    }
}

pub(crate) fn resolve_resolve_hongju(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.get(1).unwrap_or(&0);
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let qingjian_def = engine_api::SkillDefId::new(*payload.get(3).unwrap_or(&0));
    let rong_def = engine_api::SkillDefId::new(*payload.get(4).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(5).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "鸿举 choice must be made by the owner".into(),
        ));
    }
    let rong = tx.state().skill_pile_cards(player, rong_def.raw());
    match stage {
        1 => {
            if cancelled || selections.first().copied() != Some(1) {
                return crate::effect::run_effects(
                    tx,
                    vec![
                        EffectRequest::ChangeMaxHp { player, delta: -1 },
                        EffectRequest::AddSkill {
                            player,
                            skill_def: qingjian_def,
                        },
                    ],
                )
                .map(|_| ());
            }
            let count = rong.len();
            let hand = tx.state().hand_cards(player);
            if count == 0 || hand.len() < count {
                return crate::effect::run_effects(
                    tx,
                    vec![
                        EffectRequest::ChangeMaxHp { player, delta: -1 },
                        EffectRequest::AddSkill {
                            player,
                            skill_def: qingjian_def,
                        },
                    ],
                )
                .map(|_| ());
            }
            crate::effect::open_choice(
                tx,
                player,
                count.min(u8::MAX as usize) as u8,
                count.min(u8::MAX as usize) as u8,
                false,
                hand.into_iter().map(engine_api::CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::Custom,
                vec![
                    crate::effect::RESOLVE_HONGJU_CUSTOM_CHOICE,
                    2,
                    player.raw(),
                    qingjian_def.raw(),
                    skill.raw(),
                ],
            )
            .map(|_| ())
        }
        2 => {
            if selections.len() != rong.len() {
                return Err(EffectError::Illegal(
                    "鸿举 hand selection count mismatch".into(),
                ));
            }
            let hand = tx.state().hand_cards(player);
            let selected: Vec<_> = selections
                .iter()
                .map(|&raw| engine_api::CardId::new(raw))
                .collect();
            if selected.iter().any(|card| !hand.contains(card)) {
                return Err(EffectError::Illegal(
                    "鸿举 hand card is no longer legal".into(),
                ));
            }
            let hand_zone = tx
                .state()
                .zones
                .find(crate::state::ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
            let pile = crate::effect::ensure_skill_pile_zone(tx, player, rong_def.raw())?;
            crate::effect::run_effects(
                tx,
                vec![
                    EffectRequest::MoveCards {
                        cards: rong.clone(),
                        to: hand_zone,
                        from: Some(pile),
                    },
                    EffectRequest::MoveCards {
                        cards: selected,
                        to: pile,
                        from: Some(hand_zone),
                    },
                ],
            )?;
            crate::effect::run_effects(
                tx,
                vec![
                    EffectRequest::ChangeMaxHp { player, delta: -1 },
                    EffectRequest::AddSkill {
                        player,
                        skill_def: qingjian_def,
                    },
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault("unknown 鸿举 choice stage".into())),
    }
}

pub(crate) fn resolve_resolve_qingjian(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.get(1).unwrap_or(&0);
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let rong_def = engine_api::SkillDefId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "清俭 choice must be made by the owner".into(),
        ));
    }
    match stage {
        1 => {
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("清俭 requires one 荣".into()));
            }
            let rong = tx.state().skill_pile_cards(player, rong_def.raw());
            if !rong.contains(&engine_api::CardId::new(selections[0])) {
                return Err(EffectError::Illegal("清俭 荣 is no longer legal".into()));
            }
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                tx.state()
                    .hand_cards(player)
                    .into_iter()
                    .map(engine_api::CardId::raw)
                    .collect(),
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::Custom,
                vec![
                    crate::effect::RESOLVE_QINGJIAN_CUSTOM_CHOICE,
                    2,
                    player.raw(),
                    rong_def.raw(),
                    selections[0],
                    skill.raw(),
                ],
            )
            .map(|_| ())
        }
        2 => {
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("清俭 requires one hand card".into()));
            }
            let hand_card = engine_api::CardId::new(selections[0]);
            if !tx.state().hand_cards(player).contains(&hand_card) {
                return Err(EffectError::Illegal(
                    "清俭 hand card is no longer legal".into(),
                ));
            }
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&candidate| candidate != player)
                .filter(|&candidate| {
                    !crate::query::discardable_target_cards(
                        tx.state(),
                        player,
                        candidate,
                        rule_api::TargetCardScope::EquipJudge,
                    )
                    .is_empty()
                })
                .collect();
            if options.is_empty() {
                return Err(EffectError::Illegal("清俭 has no legal target".into()));
            }
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                engine_api::ChoiceKind::SelectTargets,
                crate::state::ContinuationKind::Custom,
                vec![
                    crate::effect::RESOLVE_QINGJIAN_CUSTOM_CHOICE,
                    3,
                    player.raw(),
                    rong_def.raw(),
                    payload.get(4).copied().unwrap_or(0),
                    selections[0],
                    skill.raw(),
                ],
            )
            .map(|_| ())
        }
        3 => {
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("清俭 requires one target".into()));
            }
            let target = PlayerId::new(selections[0]);
            let cards = crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::EquipJudge,
            );
            if cards.is_empty() {
                return Err(EffectError::Illegal("清俭 target has no ej card".into()));
            }
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                cards.into_iter().map(engine_api::CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::Custom,
                vec![
                    crate::effect::RESOLVE_QINGJIAN_CUSTOM_CHOICE,
                    4,
                    player.raw(),
                    rong_def.raw(),
                    payload.get(4).copied().unwrap_or(0),
                    payload.get(5).copied().unwrap_or(0),
                    selections[0],
                    skill.raw(),
                ],
            )
            .map(|_| ())
        }
        4 => {
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("清俭 requires one ej card".into()));
            }
            let rong_card = engine_api::CardId::new(*payload.get(4).unwrap_or(&0));
            let hand_card = engine_api::CardId::new(*payload.get(5).unwrap_or(&0));
            let target = PlayerId::new(*payload.get(6).unwrap_or(&0));
            let ej_card = engine_api::CardId::new(selections[0]);
            if !crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::EquipJudge,
            )
            .contains(&ej_card)
            {
                return Err(EffectError::Illegal(
                    "清俭 ej card is no longer legal".into(),
                ));
            }
            let rong_zone = crate::effect::ensure_skill_pile_zone(tx, player, rong_def.raw())?;
            let hand_zone = tx
                .state()
                .zones
                .find(crate::state::ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
            let discard = tx
                .state()
                .zones
                .find(crate::state::ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
            let ej_zone = tx
                .state()
                .cards
                .get(ej_card)
                .map(|value| value.zone)
                .ok_or_else(|| EffectError::Illegal("清俭 ej card missing".into()))?;
            let hand_card_zone = tx
                .state()
                .cards
                .get(hand_card)
                .map(|value| value.zone)
                .ok_or_else(|| EffectError::Illegal("清俭 hand card missing".into()))?;
            crate::effect::run_effects(
                tx,
                vec![
                    EffectRequest::MoveCards {
                        cards: vec![rong_card],
                        to: hand_zone,
                        from: Some(rong_zone),
                    },
                    EffectRequest::MoveCards {
                        cards: vec![hand_card],
                        to: discard,
                        from: Some(hand_card_zone),
                    },
                    EffectRequest::MoveCards {
                        cards: vec![ej_card],
                        to: discard,
                        from: Some(ej_zone),
                    },
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault("unknown 清俭 choice stage".into())),
    }
}

pub(crate) fn resolve_resolve_releiji(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.get(1).unwrap_or(&0);
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "雷击 target choice must be made by the owner".into(),
        ));
    }
    match stage {
        1 => {
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("雷击 requires one target".into()));
            }
            let target = PlayerId::new(selections[0]);
            if target == player || !tx.state().players.get(target).is_some_and(|p| p.alive) {
                return Err(EffectError::Illegal(
                    "雷击 target is no longer legal".into(),
                ));
            }
            // 翻亮判定牌（留在 processing 供鬼道改判）。
            let draw = tx
                .state()
                .zones
                .find(crate::state::ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
            if tx
                .state()
                .zones
                .get(draw)
                .is_none_or(|zone| zone.cards.is_empty())
            {
                crate::effect::reshuffle_discard_into_draw(tx)?;
            }
            let result_card = tx
                .state()
                .zones
                .get(draw)
                .and_then(|zone| zone.cards.last().copied())
                .ok_or_else(|| EffectError::Illegal("draw empty".into()))?;
            let processing = tx
                .state()
                .zones
                .find(crate::state::ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
            crate::effect::move_one_card(tx, result_card, draw, processing, PlayerId::NONE, true)?;
            // 征询改判者（座次自目标起），复用 judgment_rewrite_cards 候选。
            let order = tx.state().players.alive_by_seat();
            let Some(start) = order.iter().position(|&p| p == target) else {
                return Err(EffectError::EngineFault(
                    "target missing from seat order".into(),
                ));
            };
            let rewriters: Vec<_> = (0..order.len())
                .map(|offset| order[(start + offset) % order.len()])
                .filter(|&p| crate::query::can_rewrite_judgment(tx.state(), p))
                .collect();
            if rewriters.is_empty() {
                return settle_releiji_judgment(tx, player, target, result_card, skill);
            }
            open_rewrite_releiji(tx, &rewriters, player, target, result_card, skill)
        }
        2 | 3 => {
            // 改判选择（stage 2）：换牌后继续下一改判者或结算。
            let target = PlayerId::new(*payload.get(3).unwrap_or(&0));
            let result_card = engine_api::CardId::new(*payload.get(5).unwrap_or(&0));
            let rewriter_len = *payload.get(6).unwrap_or(&0) as usize;
            let rewriters: Vec<_> = payload
                .get(7..7 + rewriter_len)
                .filter(|rest| rest.len() == rewriter_len)
                .ok_or_else(|| EffectError::EngineFault("truncated 雷击 rewriter queue".into()))?
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect();
            if !cancelled && selections.len() == 1 {
                let card = engine_api::CardId::new(selections[0]);
                if observer == rewriters[0] {
                    let legal = crate::command::judge::judgment_rewrite_cards(tx, rewriters[0])
                        .contains(&card);
                    if !legal {
                        return Err(EffectError::Illegal(
                            "雷击 rewrite card no longer legal".into(),
                        ));
                    }
                    let discard = tx
                        .state()
                        .zones
                        .find(crate::state::ZoneKind::DiscardPile, PlayerId::NONE)
                        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
                    let card_zone = tx
                        .state()
                        .cards
                        .get(card)
                        .map(|value| value.zone)
                        .ok_or_else(|| EffectError::Illegal("rewrite card missing".into()))?;
                    // 改判牌消费：原判定牌 → 弃牌堆；改判牌成为新判定牌（processing）。
                    let processing = tx
                        .state()
                        .zones
                        .find(crate::state::ZoneKind::Processing, PlayerId::NONE)
                        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
                    crate::effect::move_one_card(
                        tx,
                        result_card,
                        processing,
                        discard,
                        PlayerId::NONE,
                        false,
                    )?;
                    crate::effect::move_one_card(
                        tx,
                        card,
                        card_zone,
                        processing,
                        PlayerId::NONE,
                        true,
                    )?;
                    let new_result = card;
                    let rest = &rewriters[1..];
                    if rest.is_empty() {
                        return settle_releiji_judgment(tx, player, target, new_result, skill);
                    }
                    return open_rewrite_releiji(tx, rest, player, target, new_result, skill);
                }
            }
            // 放弃/无合法改判牌：继续下一改判者。
            let rest = &rewriters[1..];
            if rest.is_empty() {
                return settle_releiji_judgment(tx, player, target, result_card, skill);
            }
            open_rewrite_releiji(tx, rest, player, target, result_card, skill)
        }
        _ => Err(EffectError::EngineFault("unknown 雷击 choice stage".into())),
    }
}

/// Open the next rewriter's rewrite choice for the 雷击 judgment.
pub(crate) fn open_rewrite_releiji(
    tx: &mut Transaction,
    rewriters: &[PlayerId],
    player: PlayerId,
    target: PlayerId,
    result_card: engine_api::CardId,
    skill: engine_api::SkillId,
) -> Result<(), EffectError> {
    let Some(&rewriter) = rewriters.first() else {
        return settle_releiji_judgment(tx, player, target, result_card, skill);
    };
    let options = crate::command::judge::judgment_rewrite_cards(tx, rewriter);
    if options.is_empty() {
        return open_rewrite_releiji(tx, &rewriters[1..], player, target, result_card, skill);
    }
    let mut payload = vec![
        crate::effect::RESOLVE_RELEIJI_CUSTOM_CHOICE,
        2,
        player.raw(),
        target.raw(),
        skill.raw(),
        result_card.raw(),
        rewriters.len() as u32,
    ];
    payload.extend(rewriters.iter().map(|p| p.raw()));
    crate::effect::open_choice(
        tx,
        rewriter,
        0,
        1,
        true,
        options.into_iter().map(engine_api::CardId::raw).collect(),
        engine_api::ChoiceKind::SelectCards,
        crate::state::ContinuationKind::Custom,
        payload,
    )
    .map(|_| ())
}
