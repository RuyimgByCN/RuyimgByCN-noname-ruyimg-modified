//! Command operations for `settle_releiji_judgment` and related helpers.

use super::*;

/// Settle the 雷击 judgment: move the final card to discard and apply the suit branch.
pub(crate) fn settle_releiji_judgment(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    result_card: engine_api::CardId,
    skill: engine_api::SkillId,
) -> Result<(), EffectError> {
    let processing = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    let discard = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![result_card],
            to: discard,
            from: Some(processing),
        }],
    )?;
    let suit = tx
        .state()
        .cards
        .get(result_card)
        .map(|value| crate::query::effective_card_suit(tx.state(), target, value))
        .unwrap_or(engine_api::Suit::None);
    let rank = tx
        .state()
        .cards
        .get(result_card)
        .map(|value| value.rank)
        .unwrap_or(0);
    tx.push_event(engine_api::DomainEvent::RuleNote {
        code: 234,
        payload: vec![
            player.raw(),
            target.raw(),
            crate::command::judge::suit_tag(suit),
            rank as u32,
            skill.raw(),
        ],
    });
    match suit {
        engine_api::Suit::Spade => crate::effect::run_effects(
            tx,
            vec![EffectRequest::DealCardDamage {
                source: PlayerId::NONE,
                target,
                amount: 2,
                is_lose_hp: false,
                nature: rule_api::damage_nature::THUNDER,
                card: engine_api::CardId::NONE,
                card_kind: 0,
                card_flags: 0,
            }],
        )
        .map(|_| ()),
        engine_api::Suit::Club => crate::effect::run_effects(
            tx,
            vec![
                EffectRequest::RecoverHp {
                    source: PlayerId::NONE,
                    player,
                    amount: 1,
                },
                EffectRequest::DealCardDamage {
                    source: PlayerId::NONE,
                    target,
                    amount: 1,
                    is_lose_hp: false,
                    nature: rule_api::damage_nature::THUNDER,
                    card: engine_api::CardId::NONE,
                    card_kind: 0,
                    card_flags: 0,
                },
            ],
        )
        .map(|_| ()),
        _ => Ok(()),
    }
}

pub(crate) fn resolve_resolve_huangtian(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.get(1).unwrap_or(&0);
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let actor = PlayerId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    match stage {
        1 => {
            if observer != actor {
                return Err(EffectError::Illegal(
                    "黄天 choice must be made by the Qun player".into(),
                ));
            }
            if cancelled || selections.len() != 1 {
                return Ok(()); // 拒绝/取消：不赠送。
            }
            let card = engine_api::CardId::new(selections[0]);
            let legal = tx.state().hand_cards(actor).into_iter().any(|c| c == card)
                && tx.state().cards.get(card).is_some_and(|value| {
                    matches!(
                        crate::query::effective_card_kind(tx.state(), actor, value),
                        crate::state::CardKind::Shan | crate::state::CardKind::TrickShandian
                    )
                });
            if !legal {
                return Err(EffectError::Illegal("黄天 card is no longer legal".into()));
            }
            let zone = tx
                .state()
                .cards
                .get(card)
                .map(|value| value.zone)
                .ok_or_else(|| EffectError::Illegal("黄天 card missing".into()))?;
            let hand = tx
                .state()
                .zones
                .find(crate::state::ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("no lord hand zone".into()))?;
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![card],
                    to: hand,
                    from: Some(zone),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault("unknown 黄天 choice stage".into())),
    }
}

pub(crate) fn resolve_resolve_haoshi(
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
    if observer != player {
        return Err(EffectError::Illegal(
            "好施 choice must be made by the owner".into(),
        ));
    }
    match stage {
        2 => {
            if cancelled || selections.is_empty() {
                return Err(EffectError::Illegal(
                    "好施 requires the half hand cards".into(),
                ));
            }
            let give = tx.state().hand_count(player) / 2;
            if selections.len() != give as usize
                || !tx.state().players.get(target).is_some_and(|p| p.alive)
            {
                return Err(EffectError::Illegal("好施 target or count changed".into()));
            }
            let cards: Vec<_> = selections
                .iter()
                .map(|&raw| engine_api::CardId::new(raw))
                .collect();
            let hand = tx.state().hand_cards(player);
            if cards.iter().any(|card| !hand.contains(card)) {
                return Err(EffectError::Illegal("好施 card is no longer legal".into()));
            }
            let hand_zone = tx
                .state()
                .zones
                .find(crate::state::ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
            let target_hand = tx
                .state()
                .zones
                .find(crate::state::ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("no target hand zone".into()))?;
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards,
                    to: target_hand,
                    from: Some(hand_zone),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault("unknown 好施 choice stage".into())),
    }
}

pub(crate) fn resolve_resolve_dimeng(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.get(1).unwrap_or(&0);
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "缔盟 choice must be made by the owner".into(),
        ));
    }
    match stage {
        1 => {
            if cancelled || selections.len() != 2 {
                return Err(EffectError::Illegal("缔盟 requires two targets".into()));
            }
            let left = PlayerId::new(selections[0]);
            let right = PlayerId::new(selections[1]);
            if left == right
                || left == player
                || right == player
                || !tx.state().players.get(left).is_some_and(|p| p.alive)
                || !tx.state().players.get(right).is_some_and(|p| p.alive)
            {
                return Err(EffectError::Illegal(
                    "缔盟 targets are no longer legal".into(),
                ));
            }
            let diff = tx
                .state()
                .hand_count(left)
                .abs_diff(tx.state().hand_count(right));
            let options = tx
                .state()
                .hand_cards(player)
                .into_iter()
                .chain(
                    tx.state()
                        .zones
                        .find(crate::state::ZoneKind::Equip, player)
                        .map(|zone| {
                            tx.state()
                                .zones
                                .get(zone)
                                .map(|zone| zone.cards.clone())
                                .unwrap_or_default()
                        })
                        .unwrap_or_default(),
                )
                .collect::<Vec<_>>();
            if options.len() < diff as usize {
                return Err(EffectError::Illegal(
                    "缔盟 cannot pay the hand difference".into(),
                ));
            }
            if diff == 0 {
                // 手牌相等：直接交换，无需弃牌。
                return exchange_dimeng_hands(tx, player, left, right, &[], skill);
            }
            crate::effect::open_choice(
                tx,
                player,
                diff.min(u8::MAX as u32) as u8,
                diff.min(u8::MAX as u32) as u8,
                false,
                options.into_iter().map(engine_api::CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::Custom,
                vec![
                    crate::effect::RESOLVE_DIMENG_CUSTOM_CHOICE,
                    2,
                    player.raw(),
                    left.raw(),
                    right.raw(),
                    skill.raw(),
                ],
            )
            .map(|_| ())
        }
        2 => {
            if cancelled {
                return Err(EffectError::Illegal(
                    "缔盟 requires the discard payment".into(),
                ));
            }
            let left = PlayerId::new(*payload.get(3).unwrap_or(&0));
            let right = PlayerId::new(*payload.get(4).unwrap_or(&0));
            let diff = tx
                .state()
                .hand_count(left)
                .abs_diff(tx.state().hand_count(right));
            if selections.len() != diff as usize {
                return Err(EffectError::Illegal(
                    "缔盟 discard count must equal the difference".into(),
                ));
            }
            let cards: Vec<_> = selections
                .iter()
                .map(|&raw| engine_api::CardId::new(raw))
                .collect();
            let owner_cards = tx.state().hand_cards(player);
            let equip_cards = tx
                .state()
                .zones
                .find(crate::state::ZoneKind::Equip, player)
                .map(|zone| {
                    tx.state()
                        .zones
                        .get(zone)
                        .map(|zone| zone.cards.clone())
                        .unwrap_or_default()
                })
                .unwrap_or_default();
            let all = owner_cards
                .into_iter()
                .chain(equip_cards)
                .collect::<Vec<_>>();
            if cards.iter().any(|card| !all.contains(card)) {
                return Err(EffectError::Illegal("缔盟 card is no longer legal".into()));
            }
            exchange_dimeng_hands(tx, player, left, right, &cards, skill)
        }
        _ => Err(EffectError::EngineFault("unknown 缔盟 choice stage".into())),
    }
}

/// 缔盟结算：弃置代价牌后交换两名角色手牌。
pub(crate) fn exchange_dimeng_hands(
    tx: &mut Transaction,
    player: PlayerId,
    left: PlayerId,
    right: PlayerId,
    discard_cards: &[engine_api::CardId],
    skill: engine_api::SkillId,
) -> Result<(), EffectError> {
    let discard = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let mut requests = Vec::new();
    for card in discard_cards {
        let zone = tx
            .state()
            .cards
            .get(*card)
            .map(|value| value.zone)
            .ok_or_else(|| EffectError::Illegal("缔盟 discard card missing".into()))?;
        requests.push(EffectRequest::MoveCards {
            cards: vec![*card],
            to: discard,
            from: Some(zone),
        });
    }
    let left_hand = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Hand, left)
        .ok_or_else(|| EffectError::Illegal("no left hand zone".into()))?;
    let right_hand = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Hand, right)
        .ok_or_else(|| EffectError::Illegal("no right hand zone".into()))?;
    let left_cards = tx.state().hand_cards(left);
    let right_cards = tx.state().hand_cards(right);
    requests.push(EffectRequest::MoveCards {
        cards: left_cards,
        to: right_hand,
        from: Some(left_hand),
    });
    requests.push(EffectRequest::MoveCards {
        cards: right_cards,
        to: left_hand,
        from: Some(right_hand),
    });
    crate::effect::run_effects(tx, requests).map(|_| ())
}

/// 享乐：使用者弃一张基本牌，否则此杀对持有者无效。
pub(crate) fn resolve_resolve_xiangle(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let source = PlayerId::new(*payload.get(3).unwrap_or(&0));
    let card = CardId::new(*payload.get(4).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(5).unwrap_or(&0));
    if observer != source {
        return Err(EffectError::Illegal(
            "享乐 choice must be made by the Sha user".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        // 拒绝弃牌：此杀对持有者无效。
        return crate::effect::run_effects(
            tx,
            vec![EffectRequest::ExcludeSavedShaTarget {
                card,
                target: player,
            }],
        )
        .map(|_| ());
    }
    let picked = CardId::new(selections[0]);
    let hand = tx.state().hand_cards(source);
    let basic_ok = tx
        .state()
        .cards
        .get(picked)
        .is_some_and(|value| value.kind.is_basic());
    if !hand.contains(&picked) || !basic_ok {
        return Err(EffectError::Illegal("享乐 card is no longer legal".into()));
    }
    let hand_zone = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Hand, source)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let discard = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![picked],
            to: discard,
            from: Some(hand_zone),
        }],
    )
    .map(|_| ())
}

/// 放权：弃一张手牌后选择一名其他角色进行额外回合。
pub(crate) fn resolve_fangquan_discard(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "放权 choice must be made by the owner".into(),
        ));
    }
    if cancelled {
        // 放弃放权：清除标记，结束阶段不再授回合。
        if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
            instance.state_blob.clear();
        }
        return Ok(());
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal("放权 requires one hand card".into()));
    }
    let picked = CardId::new(selections[0]);
    let hand = tx.state().hand_cards(player);
    if !hand.contains(&picked) {
        return Err(EffectError::Illegal(
            "放权 card is no longer in hand".into(),
        ));
    }
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
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![picked],
            to: discard,
            from: Some(hand_zone),
        }],
    )?;
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| candidate != player)
        .collect();
    crate::effect::open_choice(
        tx,
        player,
        1,
        1,
        true,
        options.into_iter().map(PlayerId::raw).collect(),
        engine_api::ChoiceKind::SelectTargets,
        crate::state::ContinuationKind::Custom,
        vec![
            crate::effect::RESOLVE_FANGQUAN_TARGET,
            2,
            player.raw(),
            skill.raw(),
        ],
    )
    .map(|_| ())
}

/// 放权：授目标一个额外回合并清除标记。
pub(crate) fn resolve_fangquan_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "放权 target choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
            instance.state_blob.clear();
        }
        return Ok(());
    }
    let target = PlayerId::new(selections[0]);
    if target == player || !tx.state().players.get(target).is_some_and(|p| p.alive) {
        return Err(EffectError::Illegal(
            "放权 target is no longer legal".into(),
        ));
    }
    if tx.state().turn.extra_turn_queue.len() >= 8 {
        return Err(EffectError::Illegal("放权 extra turn queue is full".into()));
    }
    tx.state_mut().turn.extra_turn_queue.push(target);
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob.clear();
    }
    Ok(())
}

/// 罪论（0<N<3）：选 N 张置牌堆顶并获其余。
pub(crate) fn resolve_zuilun_pick(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    let count = *payload.get(4).unwrap_or(&0) as usize;
    if observer != player {
        return Err(EffectError::Illegal(
            "罪论 choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.len() != count || count == 0 || count >= 3 {
        return Err(EffectError::Illegal(
            "罪论 requires exactly the counted picks".into(),
        ));
    }
    let revealed: Vec<_> = payload
        .get(5..5 + 3)
        .unwrap_or(&[])
        .iter()
        .map(|&raw| CardId::new(raw))
        .collect();
    let picks: Vec<_> = selections.iter().map(|&raw| CardId::new(raw)).collect();
    let processing = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    let processing_cards = tx
        .state()
        .zones
        .get(processing)
        .map(|zone| zone.cards.clone())
        .unwrap_or_default();
    if picks
        .iter()
        .any(|card| !revealed.contains(card) || !processing_cards.contains(card))
    {
        return Err(EffectError::Illegal(
            "罪论 pick is no longer revealed".into(),
        ));
    }
    let draw = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    let hand = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let mut requests = Vec::new();
    for &card in &picks {
        requests.push(EffectRequest::MoveCards {
            cards: vec![card],
            to: draw,
            from: Some(processing),
        });
    }
    let keep: Vec<_> = revealed
        .iter()
        .copied()
        .filter(|card| !picks.contains(card))
        .collect();
    if !keep.is_empty() {
        requests.push(EffectRequest::MoveCards {
            cards: keep,
            to: hand,
            from: Some(processing),
        });
    }
    crate::effect::run_effects(tx, requests).map(|_| ())
}

/// 罪论（N==3）：目标确定后一起失去 1 点体力。
pub(crate) fn resolve_zuilun_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "罪论 target choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let target = PlayerId::new(selections[0]);
    if target == player || !tx.state().players.get(target).is_some_and(|p| p.alive) {
        return Err(EffectError::Illegal(
            "罪论 target is no longer legal".into(),
        ));
    }
    crate::effect::run_effects(
        tx,
        vec![
            EffectRequest::DealDamage {
                source: PlayerId::NONE,
                target: player,
                amount: 1,
                is_lose_hp: true,
                nature: 0,
            },
            EffectRequest::DealDamage {
                source: PlayerId::NONE,
                target,
                amount: 1,
                is_lose_hp: true,
                nature: 0,
            },
        ],
    )
    .map(|_| ())
}

/// 拒战（阳分支）：获得目标一张牌并禁己方用牌。
pub(crate) fn resolve_juzhan_gain(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let target = PlayerId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "拒战 gain choice must be made by the owner".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal("拒战 requires one gained card".into()));
    }
    let picked = CardId::new(selections[0]);
    let target_cards = tx.state().hand_cards(target);
    let equip_cards = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Equip, target)
        .map(|zone| {
            tx.state()
                .zones
                .get(zone)
                .map(|zone| zone.cards.clone())
                .unwrap_or_default()
        })
        .unwrap_or_default();
    if !target_cards.contains(&picked) && !equip_cards.contains(&picked) {
        return Err(EffectError::Illegal("拒战 card is no longer legal".into()));
    }
    let from_zone = tx.state().cards.get(picked).map(|value| value.zone);
    let owner_hand = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no owner hand zone".into()))?;
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![picked],
            to: owner_hand,
            from: from_zone,
        }],
    )?;
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::BanUseAgainst {
            user: player,
            target,
        }],
    )
    .map(|_| ())
}
