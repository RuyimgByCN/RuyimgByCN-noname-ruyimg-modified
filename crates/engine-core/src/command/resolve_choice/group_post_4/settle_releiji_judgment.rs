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

/// 奇制：选定目标后弃其一张牌并令其摸一张，计数 +1。
pub(crate) fn resolve_qizhi_target(
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
            "奇制 choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let target = PlayerId::new(selections[0]);
    if target == player || !tx.state().players.get(target).is_some_and(|p| p.alive) {
        return Err(EffectError::Illegal(
            "奇制 target is no longer legal".into(),
        ));
    }
    let mut options = tx.state().hand_cards(target);
    options.extend(
        tx.state()
            .zones
            .find(crate::state::ZoneKind::Equip, target)
            .map(|zone| {
                tx.state()
                    .zones
                    .get(zone)
                    .map(|zone| zone.cards.clone())
                    .unwrap_or_default()
            })
            .unwrap_or_default(),
    );
    if options.is_empty() {
        return Ok(());
    }
    crate::effect::open_choice(
        tx,
        player,
        1,
        1,
        true,
        options.into_iter().map(engine_api::CardId::raw).collect(),
        engine_api::ChoiceKind::SelectCards,
        crate::state::ContinuationKind::Custom,
        vec![
            crate::effect::RESOLVE_QIZHI_CARD,
            2,
            player.raw(),
            target.raw(),
            skill.raw(),
        ],
    )
    .map(|_| ())
}

/// 奇制：弃置选定牌并令目标摸一张。
pub(crate) fn resolve_qizhi_card(
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
            "奇制 card choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
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
        return Err(EffectError::Illegal("奇制 card is no longer legal".into()));
    }
    let from_zone = tx.state().cards.get(picked).map(|value| value.zone);
    let discard = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    crate::effect::run_effects(
        tx,
        vec![
            EffectRequest::MoveCards {
                cards: vec![picked],
                to: discard,
                from: from_zone,
            },
            EffectRequest::DrawCards {
                player: target,
                count: 1,
            },
        ],
    )?;
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::ResolveQizhi {
            player,
            targets: Vec::new(),
            stage: 2,
            skill,
        }],
    )
    .map(|_| ())
}

/// 进趋：弃置至奇制次数张。
pub(crate) fn resolve_jinqu_discard(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let qizhi_def = engine_api::SkillDefId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "进趋 choice must be made by the owner".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    let keep = crate::effect::skill_instance_by_def(tx, player, qizhi_def)
        .and_then(|id| tx.state().skills.get(id))
        .map(|instance| crate::effect::qizhi_count(tx, instance.id))
        .unwrap_or(0);
    let expected = tx.state().hand_count(player).saturating_sub(keep);
    if selections.len() != expected as usize {
        return Err(EffectError::Illegal("进趋 discard count changed".into()));
    }
    let cards: Vec<_> = selections.iter().map(|&raw| CardId::new(raw)).collect();
    let hand = tx.state().hand_cards(player);
    if cards.iter().any(|card| !hand.contains(card)) {
        return Err(EffectError::Illegal(
            "进趋 card is no longer in hand".into(),
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
            cards,
            to: discard,
            from: Some(hand_zone),
        }],
    )
    .map(|_| ())
}

/// 荐相：目标确认后摸一张。
pub(crate) fn resolve_jianxiang_target(
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
            "荐相 choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let target = PlayerId::new(selections[0]);
    if !tx.state().players.get(target).is_some_and(|p| p.alive) {
        return Err(EffectError::Illegal(
            "荐相 target is no longer legal".into(),
        ));
    }
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::DrawCards {
            player: target,
            count: 1,
        }],
    )
    .map(|_| ())
}

/// 审时：交牌给目标（阴=来源并记录追踪；阳=手牌最多者）。
pub(crate) fn resolve_shenshi_give(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.get(1).unwrap_or(&0);
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let source = PlayerId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "审时 choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let picked = CardId::new(selections[0]);
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
    if !owner_cards.contains(&picked) && !equip_cards.contains(&picked) {
        return Err(EffectError::Illegal("审时 card is no longer legal".into()));
    }
    let from_zone = tx.state().cards.get(picked).map(|value| value.zone);
    if stage == 1 {
        // 阴：交给来源并记录追踪牌。
        let source_hand = tx
            .state()
            .zones
            .find(crate::state::ZoneKind::Hand, source)
            .ok_or_else(|| EffectError::Illegal("no source hand zone".into()))?;
        crate::effect::run_effects(
            tx,
            vec![EffectRequest::MoveCards {
                cards: vec![picked],
                to: source_hand,
                from: from_zone,
            }],
        )?;
        let mut blob = b"SS1".to_vec();
        blob.push(1);
        blob.extend_from_slice(&source.raw().to_le_bytes());
        blob.extend_from_slice(&picked.raw().to_le_bytes());
        if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
            instance.state_blob = blob;
        }
        return Ok(());
    }
    // 阳：选手牌最多者，交牌并造成 1 点伤害。
    let max_hand = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .map(|candidate| tx.state().hand_count(candidate))
        .max()
        .unwrap_or(0);
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| candidate != player && tx.state().hand_count(candidate) == max_hand)
        .collect();
    if options.is_empty() {
        return Ok(());
    }
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
            crate::effect::RESOLVE_SHENSHI_TARGET,
            3,
            player.raw(),
            skill.raw(),
            picked.raw(),
        ],
    )
    .map(|_| ())
}

/// 审时（阳）：交牌 + 伤害，若目标死亡可令一名角色摸至四张。
pub(crate) fn resolve_shenshi_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    let picked = CardId::new(*payload.get(4).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "审时 target choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let target = PlayerId::new(selections[0]);
    if !tx.state().players.get(target).is_some_and(|p| p.alive) {
        return Err(EffectError::Illegal(
            "审时 target is no longer legal".into(),
        ));
    }
    let target_hand = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("no target hand zone".into()))?;
    let from_zone = tx.state().cards.get(picked).map(|value| value.zone);
    crate::effect::run_effects(
        tx,
        vec![
            EffectRequest::MoveCards {
                cards: vec![picked],
                to: target_hand,
                from: from_zone,
            },
            EffectRequest::DealDamage {
                source: player,
                target,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            },
        ],
    )?;
    if tx.state().players.get(target).is_some_and(|p| p.alive) {
        return Ok(());
    }
    // 目标死亡：可选令一名角色手牌摸至四张。
    let options: Vec<_> = tx.state().players.alive_by_seat().into_iter().collect();
    if options.is_empty() {
        return Ok(());
    }
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
            crate::effect::RESOLVE_SHENSHI_REWARD,
            4,
            player.raw(),
            skill.raw(),
        ],
    )
    .map(|_| ())
}

/// 审时（阳）：死亡奖励——目标手牌摸至四张。
pub(crate) fn resolve_shenshi_reward(
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
            "审时 reward choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let target = PlayerId::new(selections[0]);
    let draw = 4u32.saturating_sub(tx.state().hand_count(target));
    if draw > 0 {
        crate::effect::run_effects(
            tx,
            vec![EffectRequest::DrawCards {
                player: target,
                count: draw,
            }],
        )?;
    }
    Ok(())
}

/// 明任：将一张手牌置于「任」技能牌堆。
pub(crate) fn resolve_mingren_place(
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
            "明任 choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let picked = CardId::new(selections[0]);
    let hand = tx.state().hand_cards(player);
    if !hand.contains(&picked) {
        return Err(EffectError::Illegal(
            "明任 card is no longer in hand".into(),
        ));
    }
    let hand_zone = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let def = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.def.raw())
        .ok_or_else(|| EffectError::EngineFault("明任 skill disappeared".into()))?;
    let pile = crate::effect::ensure_skill_pile_zone(tx, player, def)?;
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![picked],
            to: pile,
            from: Some(hand_zone),
        }],
    )?;
    if stage == 3 {
        // 替换：原「任」回到手牌。
        let old: Vec<_> = tx
            .state()
            .skill_piles
            .iter()
            .find(|pile| pile.owner == player && pile.skill_def == def)
            .and_then(|pile| tx.state().zones.get(pile.zone))
            .map(|zone| {
                zone.cards
                    .iter()
                    .filter(|card| **card != picked)
                    .copied()
                    .collect()
            })
            .unwrap_or_default();
        if let Some(old_card) = old.first().copied() {
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![old_card],
                    to: hand_zone,
                    from: None,
                }],
            )?;
        }
    }
    Ok(())
}

/// 贞良（阴）：弃同色牌并开攻击范围内目标选择。
pub(crate) fn resolve_zhenliang_discard(
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
            "贞良 choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let picked = CardId::new(selections[0]);
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
    if !owner_cards.contains(&picked) && !equip_cards.contains(&picked) {
        return Err(EffectError::Illegal("贞良 card is no longer legal".into()));
    }
    let from_zone = tx.state().cards.get(picked).map(|value| value.zone);
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
            from: from_zone,
        }],
    )?;
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| {
            candidate != player && crate::effect::dcchiying_in_range(tx.state(), player, candidate)
        })
        .collect();
    if options.is_empty() {
        return Ok(());
    }
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
            crate::effect::RESOLVE_ZHENLIANG_DAMAGE,
            2,
            player.raw(),
            skill.raw(),
        ],
    )
    .map(|_| ())
}

/// 贞良（阴）：对攻击范围内目标造成 1 点伤害。
pub(crate) fn resolve_zhenliang_damage(
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
            "贞良 target choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let target = PlayerId::new(selections[0]);
    if !tx.state().players.get(target).is_some_and(|p| p.alive) {
        return Err(EffectError::Illegal(
            "贞良 target is no longer legal".into(),
        ));
    }
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source: player,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .map(|_| ())
}

/// 贞良（阳）：目标摸一张牌。
pub(crate) fn resolve_zhenliang_draw(
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
            "贞良 draw choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let target = PlayerId::new(selections[0]);
    if !tx.state().players.get(target).is_some_and(|p| p.alive) {
        return Err(EffectError::Illegal(
            "贞良 draw target is no longer legal".into(),
        ));
    }
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::DrawCards {
            player: target,
            count: 1,
        }],
    )
    .map(|_| ())
}

/// 溃诛：选择摸牌或伤害分支并开目标选择。
pub(crate) fn resolve_kuizhu_dispatch(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    let x = *payload.get(4).unwrap_or(&0);
    if observer != player {
        return Err(EffectError::Illegal(
            "溃诛 choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let damage_branch = selections[0] == 1;
    let options: Vec<_> = tx.state().players.alive_by_seat();
    if options.is_empty() {
        return Ok(());
    }
    let tag = if damage_branch {
        crate::effect::RESOLVE_KUIZHU_DAMAGE
    } else {
        crate::effect::RESOLVE_KUIZHU_DRAW
    };
    crate::effect::open_choice(
        tx,
        player,
        0,
        x.min(8) as u8,
        true,
        options.into_iter().map(PlayerId::raw).collect(),
        engine_api::ChoiceKind::SelectTargets,
        crate::state::ContinuationKind::Custom,
        vec![tag, 2, player.raw(), skill.raw(), x],
    )
    .map(|_| ())
}

/// 溃诛（摸牌分支）：至多 X 名角色各摸一张。
pub(crate) fn resolve_kuizhu_draw(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    let x = *payload.get(4).unwrap_or(&0) as usize;
    if observer != player {
        return Err(EffectError::Illegal(
            "溃诛 draw choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    if selections.len() > x || selections.len() > 8 {
        return Err(EffectError::Illegal(
            "溃诛 draw target count exceeded".into(),
        ));
    }
    let requests: Vec<_> = selections
        .iter()
        .map(|&raw| EffectRequest::DrawCards {
            player: PlayerId::new(raw),
            count: 1,
        })
        .collect();
    crate::effect::run_effects(tx, requests).map(|_| ())
}
