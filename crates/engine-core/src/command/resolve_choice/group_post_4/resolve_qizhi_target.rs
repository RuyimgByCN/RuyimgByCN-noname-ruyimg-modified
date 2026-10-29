//! Command operations for `settle_releiji_judgment` and related helpers.

use super::*;

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
