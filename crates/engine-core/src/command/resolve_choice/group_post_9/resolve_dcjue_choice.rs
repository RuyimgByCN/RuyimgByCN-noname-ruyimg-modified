//! Command operations for `resolve_choose_spxizhan_discard_or_lose` and related helpers.

use super::*;

/// 举讹确认/选目标结算（ABI 583）。
/// 自己的结束阶段 payload=[magic, player, skill]；其他角色的结束阶段
/// payload=[magic, player, fixed_target, skill]。
pub(crate) fn resolve_dcjue_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if !matches!(payload.len(), 3 | 4) {
        return Err(EffectError::EngineFault("bad dcjue continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    if observer != player || cancelled || (payload.len() == 3 && selections.is_empty()) {
        return Ok(());
    }
    let (target, skill) = if payload.len() == 3 {
        let [selected] = selections else {
            return Err(EffectError::Illegal("dcjue requires one target".into()));
        };
        let target = PlayerId::new(*selected);
        if target == player
            || !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
        {
            return Err(EffectError::Illegal("invalid dcjue target".into()));
        }
        (target, engine_api::SkillId::new(payload[2]))
    } else {
        if selections == [0] {
            return Ok(());
        }
        if selections != [1] {
            return Err(EffectError::Illegal("invalid dcjue confirmation".into()));
        }
        (
            PlayerId::new(payload[2]),
            engine_api::SkillId::new(payload[3]),
        )
    };
    run_effects(
        tx,
        vec![EffectRequest::ResolveDcjue {
            player,
            target,
            stage: 1,
            remaining: 0,
            skill,
        }],
    )
    .map(|_| ())
}

/// 阮籍「酒遁」上下文选择（ABI 584）。mode 0 确认摸牌饮酒；mode 1 可取消弃手牌并
/// 在弃置触发完成后令当前黑牌对自己无效。
pub(crate) fn resolve_ruanji_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [_, player, source, card, skill, mode] = payload else {
        return Err(EffectError::EngineFault("bad Ruanji continuation".into()));
    };
    let player = PlayerId::new(*player);
    let source = PlayerId::new(*source);
    let card = CardId::new(*card);
    let skill = engine_api::SkillId::new(*skill);
    if observer != player || cancelled {
        return Ok(());
    }
    match *mode {
        0 => {
            if selections == [0] {
                return Ok(());
            }
            if selections != [1] {
                return Err(EffectError::Illegal("invalid 酒遁 confirmation".into()));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::DrawCards { player, count: 1 },
                    EffectRequest::UseVirtualCardOn {
                        player,
                        kind: CardKind::Jiu.tag(),
                        target: player,
                    },
                ],
            )?;
        }
        1 => {
            let [selected] = selections else {
                return Ok(());
            };
            let selected = CardId::new(*selected);
            let legal = crate::query::discardable_hand_equip_cards(tx.state(), player)
                .into_iter()
                .any(|candidate| {
                    candidate == selected
                        && tx.state().cards.get(candidate).is_some_and(|card| {
                            tx.state()
                                .zones
                                .get(card.zone)
                                .is_some_and(|zone| zone.kind == ZoneKind::Hand)
                        })
                });
            if !legal {
                return Err(EffectError::Illegal(
                    "酒遁 discard card became illegal".into(),
                ));
            }
            let from = tx
                .state()
                .cards
                .get(selected)
                .map(|card| card.zone)
                .ok_or_else(|| EffectError::Illegal("酒遁 discard card disappeared".into()))?;
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
            run_effects(
                tx,
                vec![
                    EffectRequest::MoveCardsWithReason {
                        cards: vec![selected],
                        to: discard,
                        from: Some(from),
                        reason: CardMoveReason::Discard,
                    },
                    EffectRequest::ResolveRuanji {
                        player,
                        source,
                        card,
                        action: 4,
                        cards: Vec::new(),
                        skill,
                    },
                ],
            )?;
        }
        _ => return Err(EffectError::EngineFault("bad Ruanji choice mode".into())),
    }
    Ok(())
}

/// 崔琰毛玠「征辟」选择续体（ABI 585）。
pub(crate) fn resolve_zhengbi_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 || payload[0] != crate::effect::RESOLVE_ZHENGBI_CHOICE || cancelled {
        return Ok(());
    }
    let stage = payload[1];
    let player = PlayerId::new(payload[2]);
    let skill = engine_api::SkillId::new(if stage == 0 { payload[3] } else { payload[4] });
    match stage {
        0 => {
            if observer != player {
                return Err(EffectError::Illegal("征辟目标选择者不匹配".into()));
            }
            let [target] = selections else {
                return Err(EffectError::Illegal("征辟需要一名目标".into()));
            };
            let target = PlayerId::new(*target);
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("征辟目标已失效".into()));
            }
            let mut branches = vec![0];
            if tx.state().hand_cards(player).into_iter().any(|card| {
                tx.state().cards.get(card).is_some_and(|card| {
                    crate::query::effective_card_kind(tx.state(), player, card).is_basic()
                })
            }) {
                branches.push(1);
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                branches,
                ChoiceKind::Confirm,
                ContinuationKind::Custom,
                vec![
                    crate::effect::RESOLVE_ZHENGBI_CHOICE,
                    1,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )?;
        }
        1 => {
            if payload.len() != 5 || observer != player {
                return Err(EffectError::Illegal("征辟分支选择者不匹配".into()));
            }
            let target = PlayerId::new(payload[3]);
            let [branch] = selections else {
                return Err(EffectError::Illegal("征辟需要选择一项".into()));
            };
            match *branch {
                0 => {
                    run_effects(
                        tx,
                        vec![EffectRequest::ResolveZhengbi {
                            player,
                            target,
                            card: CardId::NONE,
                            action: 4,
                            skill,
                        }],
                    )?;
                }
                1 => {
                    let options: Vec<_> = tx
                        .state()
                        .hand_cards(player)
                        .into_iter()
                        .filter(|card| {
                            tx.state().cards.get(*card).is_some_and(|card| {
                                crate::query::effective_card_kind(tx.state(), player, card)
                                    .is_basic()
                            })
                        })
                        .map(CardId::raw)
                        .collect();
                    if options.is_empty() {
                        return Err(EffectError::Illegal("征辟已无可交出的基本牌".into()));
                    }
                    open_choice(
                        tx,
                        player,
                        1,
                        1,
                        false,
                        options,
                        ChoiceKind::SelectCards,
                        ContinuationKind::Custom,
                        vec![
                            crate::effect::RESOLVE_ZHENGBI_CHOICE,
                            2,
                            player.raw(),
                            target.raw(),
                            skill.raw(),
                        ],
                    )?;
                }
                _ => return Err(EffectError::Illegal("征辟分支无效".into())),
            }
        }
        2 => {
            if payload.len() != 5 || observer != player {
                return Err(EffectError::Illegal("征辟赠牌选择者不匹配".into()));
            }
            let [card] = selections else {
                return Err(EffectError::Illegal("征辟需要交出一张基本牌".into()));
            };
            run_effects(
                tx,
                vec![EffectRequest::ResolveZhengbi {
                    player,
                    target: PlayerId::new(payload[3]),
                    card: CardId::new(*card),
                    action: 2,
                    skill,
                }],
            )?;
        }
        3 => {
            if payload.len() != 5 {
                return Err(EffectError::EngineFault("征辟回礼分支数据损坏".into()));
            }
            let target = PlayerId::new(payload[3]);
            if observer != target {
                return Err(EffectError::Illegal("征辟回礼选择者不匹配".into()));
            }
            let [mode @ 0..=1] = selections else {
                return Err(EffectError::Illegal("征辟回礼分支无效".into()));
            };
            let hand = tx.state().zones.find(ZoneKind::Hand, target);
            let options: Vec<_> = crate::query::discardable_hand_equip_cards(tx.state(), target)
                .into_iter()
                .filter(|card| {
                    tx.state().cards.get(*card).is_some_and(|card| {
                        let basic =
                            crate::query::effective_card_kind(tx.state(), target, card).is_basic();
                        if *mode == 0 {
                            !basic
                        } else {
                            basic && Some(card.zone) == hand
                        }
                    })
                })
                .map(CardId::raw)
                .collect();
            let count = if *mode == 0 { 1 } else { 2 };
            if options.len() < count as usize {
                return Err(EffectError::Illegal("征辟回礼牌不足".into()));
            }
            open_choice(
                tx,
                target,
                count,
                count,
                false,
                options,
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    crate::effect::RESOLVE_ZHENGBI_CHOICE,
                    4,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                    *mode,
                ],
            )?;
        }
        4 => {
            if payload.len() != 6 {
                return Err(EffectError::EngineFault("征辟回礼数据损坏".into()));
            }
            let target = PlayerId::new(payload[3]);
            if observer != target {
                return Err(EffectError::Illegal("征辟回礼选择者不匹配".into()));
            }
            let mode = payload[5];
            let expected = if mode == 0 { 1 } else { 2 };
            if selections.len() != expected {
                return Err(EffectError::Illegal("征辟回礼数量无效".into()));
            }
            let hand = tx.state().zones.find(ZoneKind::Hand, target);
            let allowed: Vec<_> = crate::query::discardable_hand_equip_cards(tx.state(), target)
                .into_iter()
                .filter(|card| {
                    tx.state().cards.get(*card).is_some_and(|card| {
                        let basic =
                            crate::query::effective_card_kind(tx.state(), target, card).is_basic();
                        if mode == 0 {
                            !basic
                        } else {
                            basic && Some(card.zone) == hand
                        }
                    })
                })
                .collect();
            let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
            if cards.iter().any(|card| !allowed.contains(card)) {
                return Err(EffectError::Illegal("征辟回礼牌已失效".into()));
            }
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("征辟拥有者手牌区不存在".into()))?;
            let from = tx
                .state()
                .cards
                .get(cards[0])
                .map(|card| card.zone)
                .ok_or_else(|| EffectError::Illegal("征辟回礼牌不存在".into()))?;
            if cards.iter().any(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_none_or(|card| card.zone != from)
            }) {
                return Err(EffectError::Illegal("征辟回礼牌不在同一区域".into()));
            }
            run_effects(tx, vec![EffectRequest::GiveCards { cards, to, from }])?;
        }
        5 => {
            if payload.len() != 5 || observer != player {
                return Err(EffectError::Illegal("征辟获牌选择者不匹配".into()));
            }
            let target = PlayerId::new(payload[3]);
            let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
            let allowed = crate::query::gainable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::HandEquip,
            );
            if cards.is_empty() || cards.iter().any(|card| !allowed.contains(card)) {
                return Err(EffectError::Illegal("征辟所选牌已失效".into()));
            }
            let mut zones = std::collections::BTreeSet::new();
            for card in &cards {
                let zone = tx
                    .state()
                    .cards
                    .get(*card)
                    .map(|card| card.zone)
                    .ok_or_else(|| EffectError::Illegal("征辟所选牌不存在".into()))?;
                if !zones.insert(zone) {
                    return Err(EffectError::Illegal("征辟每个区域至多获得一张牌".into()));
                }
            }
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("征辟拥有者手牌区不存在".into()))?;
            let effects = cards
                .into_iter()
                .map(|card| EffectRequest::MoveCards {
                    cards: vec![card],
                    to,
                    from: tx.state().cards.get(card).map(|card| card.zone),
                })
                .collect();
            run_effects(tx, effects)?;
        }
        _ => return Err(EffectError::EngineFault("未知征辟选择阶段".into())),
    }
    Ok(())
}

/// 追锋选目标结算（ABI 419，payload[0]=RESOLVE_DBZHUIFENG_TARGET）。
/// payload: [magic, player, skill]；selections[0] = 决斗目标。
/// 对目标视为使用虚拟【决斗】。
pub(crate) fn resolve_dbzhuifeng_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 {
        return Err(EffectError::EngineFault(
            "bad Dbzhuifeng continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    if observer != player || cancelled {
        return Ok(());
    }
    let [target_raw] = selections else {
        return Err(EffectError::Illegal(
            "Dbzhuifeng requires one target".into(),
        ));
    };
    let target = PlayerId::new(*target_raw);
    if target == player
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("Dbzhuifeng target invalid".into()));
    }
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::UseVirtualCardOn {
            player,
            kind: u8::from(crate::state::CardKind::TrickJuedou.tag()),
            target,
        }],
    )
    .map(|_| ())
}

/// 曲迪三选一结算（ABI 420，payload[0]=RESOLVE_DBQUEDI_CHOICE）。
/// payload: [magic, player, target, card, skill]；selections[0] ∈ {0=获牌, 1=弃基本加伤, 2=背水}。
pub(crate) fn resolve_dbquedi_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 5 {
        return Err(EffectError::EngineFault("bad Dbquedi continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    let card = CardId::new(payload[3]);
    if observer != player || cancelled {
        return Ok(());
    }
    let [option] = selections else {
        return Err(EffectError::Illegal("Dbquedi requires one option".into()));
    };
    let backwater = *option == 2;
    let do_gain = *option == 0 || backwater;
    let do_discard_boost = *option == 1 || backwater;
    if backwater {
        crate::effect::run_effects(tx, vec![EffectRequest::ChangeMaxHp { player, delta: -1 }])?;
    }
    if do_gain {
        let options = crate::query::gainable_target_cards(
            tx.state(),
            player,
            target,
            rule_api::TargetCardScope::Hand,
        );
        if let Some(gain) = options.into_iter().next() {
            let from = tx
                .state()
                .cards
                .get(gain)
                .map(|c| c.zone)
                .ok_or_else(|| EffectError::Illegal("Dbquedi card missing".into()))?;
            let to_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?;
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![gain],
                    to: to_hand,
                    from: Some(from),
                }],
            )?;
        }
    }
    if do_discard_boost {
        let basic = tx.state().hand_cards(player).into_iter().find(|candidate| {
            tx.state()
                .cards
                .get(*candidate)
                .is_some_and(|value| value.kind.is_basic())
        });
        if let Some(basic) = basic {
            crate::effect::discard_cards(tx, player, vec![basic])?;
            // 此牌对目标伤害 +1。
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::IncreaseCurrentCardBaseDamage {
                    player,
                    card,
                    amount: 1,
                    skill: engine_api::SkillId::NONE,
                }],
            )?;
        }
    }
    Ok(())
}

/// 隐据二选一结算（ABI 421，payload[0]=RESOLVE_SPYINJU_CHOICE）。
/// payload: [magic, player, target, skill]；selections[0] ∈ {0=用杀, 1=跳过}。
pub(crate) fn resolve_spyinju_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    const SKIP_MARK: u32 = 2_728_392_676;
    if payload.len() != 4 {
        return Err(EffectError::EngineFault("bad Spyinju continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    if observer != target || cancelled {
        return Ok(());
    }
    let [option] = selections else {
        return Err(EffectError::Illegal("Spyinju requires one option".into()));
    };
    match *option {
        0 => {
            // 对拥有者使用一张手牌【杀】（有杀时；无则视为跳过）。
            let sha = tx.state().hand_cards(target).into_iter().find(|candidate| {
                tx.state()
                    .cards
                    .get(*candidate)
                    .is_some_and(|value| value.kind.is_sha())
            });
            let Some(sha) = sha else {
                return Ok(());
            };
            let r = crate::effect::run_effects(
                tx,
                vec![EffectRequest::UseCard {
                    player: target,
                    card: sha,
                    targets: vec![player],
                }],
            );
            r.map(|_| ())
        }
        1 => crate::effect::run_effects(
            tx,
            vec![EffectRequest::AddMark {
                player: target,
                mark: engine_api::MarkDefId::new(SKIP_MARK),
                delta: 1,
            }],
        )
        .map(|_| ()),
        _ => Err(EffectError::Illegal("unknown Spyinju option".into())),
    }
}

/// 上议选牌结算（ABI 423，payload[0]=RESOLVE_SPSHANGYI_CHOICE）。
/// payload: [magic, player, target, skill]；selections[0] = 获得的目标手牌。
pub(crate) fn resolve_spshangyi_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault(
            "bad Spshangyi continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    if observer != player || cancelled {
        return Ok(());
    }
    let [card_raw] = selections else {
        return Err(EffectError::Illegal("Spshangyi requires one card".into()));
    };
    let card = CardId::new(*card_raw);
    if !tx.state().hand_cards(target).contains(&card) {
        return Err(EffectError::Illegal(
            "Spshangyi card not in target hand".into(),
        ));
    }
    let from = tx
        .state()
        .cards
        .get(card)
        .map(|c| c.zone)
        .ok_or_else(|| EffectError::Illegal("Spshangyi card missing".into()))?;
    let to_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?;
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: to_hand,
            from: Some(from),
        }],
    )
    .map(|_| ())
}

/// 合击二选一结算（ABI 427，payload[0]=RESOLVE_HEJI_CHOICE）。
/// payload: [magic, player, target, skill]；selections[0] ∈ {0=杀, 1=决斗}。
pub(crate) fn resolve_heji_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault("bad Heji continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    if observer != player || cancelled {
        return Ok(());
    }
    let [option] = selections else {
        return Err(EffectError::Illegal("Heji requires one option".into()));
    };
    let kind = if *option == 0 {
        crate::state::CardKind::Sha
    } else {
        crate::state::CardKind::TrickJuedou
    };
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::UseVirtualCardOn {
            player,
            kind: u8::from(kind.tag()),
            target,
        }],
    )
    .map(|_| ())
}

/// 清雨防伤弃牌续体（ABI 431，payload[0]=RESOLVE_QINGYU_CUSTOM_CHOICE）：
/// 弃置所选 2 张 he 并防止当前伤害。
pub(crate) fn resolve_qingyu_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 {
        return Err(EffectError::EngineFault("bad Qingyu continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    if observer != player || cancelled {
        return Ok(());
    }
    let cards: Vec<_> = selections.iter().map(|raw| CardId::new(*raw)).collect();
    let allowed = crate::query::discardable_hand_equip_cards(tx.state(), player);
    if cards.len() != 2 || cards.iter().any(|card| !allowed.contains(card)) {
        return Err(EffectError::Illegal("Qingyu discard card invalid".into()));
    }
    crate::effect::run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }])?;
    let damage_id = tx.state().resolution.current_damage_id;
    if damage_id != 0 {
        crate::effect::run_effects(tx, vec![EffectRequest::PreventDamage { damage_id }])?;
    }
    let _ = skill;
    Ok(())
}

/// 清雨使命失败授技目标续体（ABI 431，payload[0]=RESOLVE_QINGYU_FAIL_TARGET）。
pub(crate) fn resolve_qingyu_fail_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault(
            "bad Qingyu fail continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let mbyongjue_def = engine_api::SkillDefId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    if observer != player || cancelled {
        return Ok(());
    }
    let [target_raw] = selections else {
        return Err(EffectError::Illegal(
            "Qingyu fail requires one target".into(),
        ));
    };
    let target = PlayerId::new(*target_raw);
    if target == player
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "Qingyu fail target no longer legal".into(),
        ));
    }
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::ResolveQingyu {
            player,
            action: 3,
            xuancun_def: engine_api::SkillDefId::NONE,
            mbyongjue_def,
            target,
            skill,
        }],
    )
    .map(|_| ())
}
