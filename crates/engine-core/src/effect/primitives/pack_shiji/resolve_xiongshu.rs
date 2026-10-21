//! Effect primitives for `resolve_xiongshu` and related helpers.

use super::*;

/// 凶竖（贾充，ABI 436）：其他角色出牌阶段开始时弃置 X 张牌（X=凶竖标记数，
/// 每轮递增）并展示其一张手牌，预测其是否会使用该牌；该出牌阶段结束时预测对
/// 则其受 1 点伤害，错则获得该牌（目标区域或弃牌堆）。
/// stage 0 弃牌（标记>0 时）+加标记 → 1 选目标手牌展示 → 2 预测 →
/// 3（phase_play_end）结算。
pub(crate) fn resolve_xiongshu(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    action: u8,
    stage: u8,
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
    match stage {
        0 => {
            if tx.state().turn.current != target || tx.state().turn.phase != Phase::Play {
                return Ok(Vec::new());
            }
            let mark = tx
                .state()
                .players
                .get(player)
                .and_then(|p| {
                    p.marks
                        .iter()
                        .find(|(m, _)| *m == XIONGSHU_MARK)
                        .map(|(_, n)| *n)
                })
                .unwrap_or(0)
                .max(0) as u32;
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if (options.len() as u32) < mark {
                return Ok(Vec::new());
            }
            if mark == 0 {
                // 无弃牌代价直接推进；发动即标记 +1（下次弃 1 张）。
                return run_effects(
                    tx,
                    vec![
                        EffectRequest::AddMark {
                            player,
                            mark: engine_api::MarkDefId::new(XIONGSHU_MARK),
                            delta: 1,
                        },
                        EffectRequest::ResolveXiongshu {
                            player,
                            target,
                            card: CardId::NONE,
                            action: 0,
                            stage: 1,
                            skill,
                        },
                    ],
                );
            }
            open_choice(
                tx,
                player,
                mark as u8,
                mark as u8,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_XIONGSHU_CHOICE,
                    0,
                    player.raw(),
                    target.raw(),
                    mark,
                    skill.raw(),
                ],
            )
        }
        1 => {
            // 选目标一张手牌展示。
            let hand = tx.state().hand_cards(target);
            if hand.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                hand.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_XIONGSHU_CHOICE,
                    1,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )
        }
        2 => {
            // 预测：0 会使用 / 1 不会使用。
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                vec![0, 1],
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_XIONGSHU_CHOICE,
                    2,
                    player.raw(),
                    target.raw(),
                    card.raw(),
                    skill.raw(),
                ],
            )
        }
        3 => {
            // 结算：预测对 → 目标 1 点伤害；错 → 获得该牌（目标区域或弃牌堆）。
            // blob "XS1" + target(4) + card(4) + action(1)。
            if tx.state().turn.current != target {
                return Ok(Vec::new());
            }
            let blob = tx
                .state()
                .skills
                .get(skill)
                .and_then(|instance| instance.state_blob.strip_prefix(b"XS1"))
                .map(|rest| rest.to_vec())
                .unwrap_or_default();
            if blob.len() < 9 {
                return Ok(Vec::new());
            }
            let saved_target =
                PlayerId::new(u32::from_le_bytes([blob[0], blob[1], blob[2], blob[3]]));
            let saved_card = CardId::new(u32::from_le_bytes([blob[4], blob[5], blob[6], blob[7]]));
            let predicted = blob[8];
            if saved_target != target {
                return Ok(Vec::new());
            }
            // 清空预测状态。
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.state_blob.clear();
            }
            // 近似：目标区域不再持有该牌视为已使用（旧 JS 以 useCard 历史判定）。
            let used = !tx.state().cards.get(saved_card).is_some_and(|value| {
                value.owner == target
                    && matches!(
                        tx.state().zones.get(value.zone).map(|z| z.kind),
                        Some(ZoneKind::Hand | ZoneKind::Equip | ZoneKind::Judge)
                    )
            });
            let correct = used == (predicted == 0);
            if correct {
                return deal_damage(
                    tx,
                    PlayerId::NONE,
                    target,
                    1,
                    false,
                    rule_api::damage_nature::NORMAL,
                    false,
                    CardId::NONE,
                    0,
                    0,
                );
            }
            // 预测错：获得该牌（目标区域或弃牌堆）。
            let from = tx.state().cards.get(saved_card).map(|value| value.zone);
            let Some(from) = from else {
                return Ok(Vec::new());
            };
            let to_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?;
            move_cards(tx, vec![saved_card], to_hand, Some(from))
        }
        _ => Err(EffectError::ContentFault("unknown Xiongshu stage".into())),
    }
}

/// 奸回（贾充，ABI 437）：伤害结算后——作为伤害来源且受伤者与上次相同则摸一张；
/// 受伤且来源与上次相同（连续同来源）则来源强制弃一张手牌或装备。
/// blob "JH1" + last_source(4) + last_target(4)。
pub(crate) fn resolve_jianhui(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || source == target
    {
        return Ok(Vec::new());
    }
    let blob = tx
        .state()
        .skills
        .get(skill)
        .and_then(|instance| instance.state_blob.strip_prefix(b"JH1"))
        .map(|rest| rest.to_vec())
        .unwrap_or_default();
    let last_source = if blob.len() >= 4 {
        PlayerId::new(u32::from_le_bytes([blob[0], blob[1], blob[2], blob[3]]))
    } else {
        PlayerId::NONE
    };
    let last_target = if blob.len() >= 8 {
        PlayerId::new(u32::from_le_bytes([blob[4], blob[5], blob[6], blob[7]]))
    } else {
        PlayerId::NONE
    };
    let mut effects = Vec::new();
    if player == source {
        // 作为来源：连续对同一角色造成伤害 → 摸一张。
        if target == last_target && source != target {
            effects.push(EffectRequest::DrawCards { player, count: 1 });
        }
    } else if player == target {
        // 受伤：来源与上次相同（连续同来源）→ 来源强制弃一张手牌或装备。
        if source == last_source && source.is_some() && source != player {
            effects.push(EffectRequest::ChooseDiscardOneHandOrEquip { player: source });
        }
    }
    // 更新记录。
    let mut out = Vec::with_capacity(8);
    out.extend_from_slice(&source.raw().to_le_bytes());
    out.extend_from_slice(&target.raw().to_le_bytes());
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob = [b"JH1".as_slice(), &out].concat();
    }
    run_effects(tx, effects)
}

/// 弃哀（王粲，ABI 439）：出牌阶段限一次将一张非基本牌交给一名其他角色——
/// 满血则摸两张，已受伤则目标二选一（令拥有者回复 1 点体力 / 摸两张）。
pub(crate) fn resolve_spqiai(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || target == player
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
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            if card.is_none() {
                return Ok(Vec::new());
            }
            let from = tx
                .state()
                .cards
                .get(card)
                .map(|card| card.zone)
                .ok_or_else(|| EffectError::Illegal("Spqiai gift card missing".into()))?;
            let to_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("recipient hand missing".into()))?;
            let mut events = move_cards_with_reason(
                tx,
                vec![card],
                to_hand,
                Some(from),
                engine_api::CardMoveReason::Give,
            )?;
            let wounded = tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.hp < participant.max_hp as i32);
            if !wounded {
                events.extend(draw_cards(tx, player, 2)?);
                return Ok(events);
            }
            events.extend(open_choice(
                tx,
                target,
                1,
                1,
                false,
                vec![0, 1],
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_SPQIAI_CHOICE,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )?);
            Ok(events)
        }
        1 => {
            // 目标二选一：0 令拥有者回复 1 / 1 令拥有者摸两张。
            let wounded = tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.hp < participant.max_hp as i32);
            if wounded {
                return recover_hp(tx, player, player, 1);
            }
            draw_cards(tx, player, 2)
        }
        _ => Err(EffectError::ContentFault("unknown Spqiai stage".into())),
    }
}

/// 擅袭（王粲，ABI 440）：出牌阶段开始选一名无“檄”标记的角色获得“檄”标记
/// （全局唯一，同时清除他人标记）。
pub(crate) fn resolve_spshanxi(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
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
    match stage {
        0 => {
            let candidates: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| {
                    *candidate != player
                        && !tx
                            .state()
                            .players
                            .get(*candidate)
                            .is_some_and(|participant| {
                                participant
                                    .marks
                                    .iter()
                                    .any(|(m, n)| *m == SPSHANXI_MARK && *n > 0)
                            })
                })
                .map(PlayerId::raw)
                .collect();
            if candidates.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                candidates,
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![RESOLVE_SPSHANXI_CHOICE, player.raw(), skill.raw()],
            )
        }
        1 => {
            // 檄标记全局唯一：清除所有角色的檄标记后授予目标。
            let mut effects = Vec::new();
            for participant in tx.state().players.iter_live() {
                if participant
                    .marks
                    .iter()
                    .any(|(m, n)| *m == SPSHANXI_MARK && *n > 0)
                {
                    effects.push(EffectRequest::RemoveMark {
                        player: participant.id,
                        mark: engine_api::MarkDefId::new(SPSHANXI_MARK),
                        amount: 0, // 0 = 移除整项（引擎语义）
                    });
                }
            }
            effects.push(EffectRequest::AddMark {
                player: target,
                mark: engine_api::MarkDefId::new(SPSHANXI_MARK),
                delta: 1,
            });
            run_effects(tx, effects)
        }
        _ => Err(EffectError::ContentFault("unknown Spshanxi stage".into())),
    }
}

/// 索命（王粲，ABI 441）：其他角色回复体力后（其有檄标记且体力 > 0）其交给你
/// 2 张牌或失去 1 点体力（可交不足时直接失体）。
pub(crate) fn resolve_spshanxi_suoming(
    tx: &mut Transaction,
    player: PlayerId,
    actor: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || actor == player
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(actor)
            .is_some_and(|participant| participant.alive && participant.hp > 0)
    {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let options = crate::query::discardable_hand_equip_cards(tx.state(), actor);
            if options.len() < 2 {
                // 可交不足：直接失去 1 点体力。
                return run_effects(
                    tx,
                    vec![EffectRequest::DealDamage {
                        source: PlayerId::NONE,
                        target: actor,
                        amount: 1,
                        is_lose_hp: true,
                        nature: rule_api::damage_nature::NORMAL,
                    }],
                );
            }
            open_choice(
                tx,
                actor,
                2,
                2,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_SUOMING_CHOICE,
                    player.raw(),
                    actor.raw(),
                    skill.raw(),
                ],
            )
        }
        1 => {
            // 交 2 张牌给拥有者；取消（未交）→ 失去 1 点体力由 handler 处理。
            let cards = {
                let mut got = Vec::new();
                if let Some(rest) = tx
                    .state()
                    .skills
                    .get(skill)
                    .and_then(|instance| instance.state_blob.strip_prefix(b"SM1"))
                {
                    for chunk in rest.chunks_exact(4) {
                        got.push(CardId::new(u32::from_le_bytes([
                            chunk[0], chunk[1], chunk[2], chunk[3],
                        ])));
                    }
                }
                got
            };
            if cards.len() != 2 {
                return Ok(Vec::new());
            }
            let to_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?;
            let mut effects = Vec::new();
            for card in &cards {
                if let Some(from) = tx.state().cards.get(*card).map(|value| value.zone) {
                    effects.push(EffectRequest::MoveCards {
                        cards: vec![*card],
                        to: to_hand,
                        from: Some(from),
                    });
                }
            }
            // 清空交牌记录。
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.state_blob.clear();
            }
            run_effects(tx, effects)
        }
        _ => Err(EffectError::ContentFault("unknown Suoming stage".into())),
    }
}

/// 通夺（刘巴，ABI 442）：成为其他角色单目标牌目标时（每回合一次）选一名有牌
/// 角色强制重铸一张牌。stage 0 选角色 → 1 该角色选牌 → 2 提交重铸。
pub(crate) fn resolve_tongduo(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    stage: u8,
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
    match stage {
        0 => {
            let candidates: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| {
                    *candidate != player && !candidate_hej_cards(tx, *candidate).is_empty()
                })
                .map(PlayerId::raw)
                .collect();
            if candidates.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                candidates,
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![RESOLVE_TONGDUO_CHOICE, 0, player.raw(), skill.raw()],
            )
        }
        1 => {
            let options = candidate_hej_cards(tx, target);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_TONGDUO_CHOICE,
                    1,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )
        }
        2 => {
            if !candidate_hej_cards(tx, target).contains(&card) {
                return Err(EffectError::Illegal(
                    "Tongduo recast card left zones".into(),
                ));
            }
            recast_cards(tx, target, vec![card])
        }
        _ => Err(EffectError::ContentFault("unknown Tongduo stage".into())),
    }
}

/// 一名角色手/装/判三区的全部牌（重铸可选范围）。
pub(crate) fn candidate_hej_cards(tx: &Transaction, player: PlayerId) -> Vec<CardId> {
    let mut cards = Vec::new();
    for kind in [ZoneKind::Hand, ZoneKind::Equip, ZoneKind::Judge] {
        if let Some(zone) = tx.state().zones.find(kind, player) {
            if let Some(zone) = tx.state().zones.get(zone) {
                cards.extend(zone.cards.iter().copied());
            }
        }
    }
    cards
}

/// 怨清（华歆，ABI 443）：出牌阶段结束时，本阶段用过的每种类别（基本/锦囊/
/// 装备）从弃牌堆权威随机取一张置入仁库（技能牌堆）。
pub(crate) fn resolve_yuanqing(
    tx: &mut Transaction,
    player: PlayerId,
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
    let mask = tx.state().turn.owner_card_category_mask(player);
    if mask == 0 {
        return Ok(Vec::new());
    }
    let skill_def = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.def.raw())
        .ok_or_else(|| EffectError::EngineFault("Yuanqing skill missing".into()))?;
    let pile = ensure_skill_pile_zone(tx, player, skill_def)?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let mut effects = Vec::new();
    for category in 0..3u32 {
        if mask & (1 << category) == 0 {
            continue;
        }
        // 弃牌堆该类牌随机取一张（权威 RNG）。
        let candidates: Vec<CardId> = tx
            .state()
            .zones
            .get(discard)
            .map(|zone| {
                zone.cards
                    .iter()
                    .copied()
                    .filter(|card| card_category_of(tx.state(), *card) == Some(category))
                    .collect()
            })
            .unwrap_or_default();
        if candidates.is_empty() {
            continue;
        }
        let mut rng = crate::rng::RuleRng::from_state(tx.state().rng);
        let picked = candidates[rng.gen_range_u32(candidates.len() as u32) as usize];
        tx.state_mut().rng = rng.into_state();
        effects.push(EffectRequest::MoveCards {
            cards: vec![picked],
            to: pile,
            from: Some(discard),
        });
    }
    run_effects(tx, effects)
}

/// 疏陈（华歆，ABI 444）：其他角色濒死时（每轮一次）若仁库（怨清技能牌堆）
/// > 3 张则获得全部并令其回复 1。
pub(crate) fn resolve_shuchen(
    tx: &mut Transaction,
    player: PlayerId,
    dying: PlayerId,
    renku_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || dying == player
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(dying)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let pile_cards = tx.state().skill_pile_cards(player, renku_def.raw());
    if pile_cards.len() <= 3 {
        return Ok(Vec::new());
    }
    let to_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?;
    let pile_zone = tx
        .state()
        .zones
        .find(ZoneKind::Expansion, player)
        .and_then(|zone| {
            tx.state()
                .skill_piles
                .iter()
                .find(|pile| pile.zone == zone && pile.skill_def == renku_def.raw())
                .map(|_| zone)
        });
    let mut effects = Vec::new();
    if let Some(from) = pile_zone {
        effects.push(EffectRequest::MoveCards {
            cards: pile_cards,
            to: to_hand,
            from: Some(from),
        });
    }
    effects.push(EffectRequest::RecoverHp {
        player: dying,
        amount: 1,
        source: PlayerId::NONE,
    });
    run_effects(tx, effects)
}

/// 断笔（刘巴，ABI 445）：限定技——全场手牌总数 > 人数×2 时可选任意名角色，
/// 逐目标强制弃 min(3, floor(手牌/2)) 张；全部弃完后再选一名角色权威随机获得
/// 至多 3 张仍处弃牌堆的被弃牌。被弃牌记录在 blob "DB1"。
pub(crate) fn resolve_duanbi(
    tx: &mut Transaction,
    player: PlayerId,
    targets: &[PlayerId],
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            // 校验全场手牌总数 > 人数×2。
            let total_hand: usize = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(|p| tx.state().hand_count(p) as usize)
                .sum();
            let players = tx.state().players.alive_by_seat().len();
            if total_hand <= players * 2 {
                return Ok(Vec::new());
            }
            let valid: Vec<PlayerId> = targets
                .iter()
                .copied()
                .filter(|t| {
                    *t != player
                        && tx
                            .state()
                            .players
                            .get(*t)
                            .is_some_and(|participant| participant.alive)
                        && !tx.state().hand_cards(*t).is_empty()
                })
                .collect();
            if valid.is_empty() {
                return Ok(Vec::new());
            }
            let first = valid[0];
            let count = duanbi_discard_count(tx, first);
            if count == 0 {
                return Ok(Vec::new());
            }
            // 技能启用 + owner/first 双方存活且异座由 GuardedChoice 守卫（ADR 0595）；
            // chooser 为 first，payload 尾部续带 valid[1..] 供续段按座次推进。
            let payload: Vec<u32> = vec![
                RESOLVE_DUANBI_CHOICE,
                0,
                player.raw(),
                skill.raw(),
                first.raw(),
            ]
            .into_iter()
            .chain(valid[1..].iter().map(|t| t.raw()))
            .collect();
            GuardedChoice::new(
                skill,
                Participants::distinct_pair(player, first),
                ContinuationKind::Custom,
                payload,
            )
            .chosen_by(first)
            .shape(ChoiceShape::exactly(count as u8))
            .open_cards(tx, tx.state().hand_cards(first))
        }
        _ => Err(EffectError::ContentFault("unknown Duanbi stage".into())),
    }
}

/// 断笔目标应弃张数：min(3, floor(手牌/2))。
pub(crate) fn duanbi_discard_count(tx: &Transaction, target: PlayerId) -> usize {
    (tx.state().hand_count(target) as usize / 2).min(3)
}

/// 躁厉（孙翊，ABI 446）：回合开始时有“躁厉”标记（使用/打出牌时 +1 至多 4，
/// DSL 表达）→ 弃至少一张牌（可全弃）并摸（标记数+弃牌数）张，总数 > 2 则
/// 失去 1 点体力。出牌阶段仅可使用本回合获得的牌由 DSL modifier
/// hand_card_gained_this_turn_only 表达。
pub(crate) fn resolve_zaoli(
    tx: &mut Transaction,
    player: PlayerId,
    mark: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || mark == 0
    {
        return Ok(Vec::new());
    }
    // 移除全部躁厉标记（amount 0 = 移除整项），开至少一张的可全弃弃牌 choice。
    let hand = tx.state().hand_cards(player);
    if hand.is_empty() {
        return run_effects(
            tx,
            vec![EffectRequest::RemoveMark {
                player,
                mark: engine_api::MarkDefId::new(ZAOLI_MARK),
                amount: 0,
            }],
        );
    }
    let events = run_effects(
        tx,
        vec![EffectRequest::RemoveMark {
            player,
            mark: engine_api::MarkDefId::new(ZAOLI_MARK),
            amount: 0,
        }],
    )?;
    let choice_events = open_choice(
        tx,
        player,
        1,
        hand.len() as u8,
        true,
        hand.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        vec![RESOLVE_ZAOLI_CHOICE, 0, player.raw(), mark, skill.raw()],
    )?;
    let mut all = events;
    all.extend(choice_events);
    Ok(all)
}

/// 孤影（向宠，ABI 447）：你失去一张手/装/判区牌后（每回合一次，当前回合角色为
/// 他人）其须二选一——随机交给你一张其可获得的牌 / 你收回失去的牌（若仍在弃牌堆）。
pub(crate) fn resolve_guying(
    tx: &mut Transaction,
    player: PlayerId,
    actor: PlayerId,
    card: CardId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || actor == player
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(actor)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            // 二选一：0 随机交给你一张可获得的牌 / 1 你收回失去的牌。
            let can_give = !crate::query::gainable_target_cards(
                tx.state(),
                player,
                actor,
                rule_api::TargetCardScope::HandEquip,
            )
            .is_empty();
            let lost_in_discard = tx.state().cards.get(card).is_some_and(|value| {
                tx.state()
                    .zones
                    .get(value.zone)
                    .is_some_and(|zone| zone.kind == ZoneKind::DiscardPile)
            });
            if !can_give && !lost_in_discard {
                return Ok(Vec::new());
            }
            let mut options = Vec::new();
            if can_give {
                options.push(0);
            }
            if lost_in_discard {
                options.push(1);
            }
            if options.len() == 1 {
                // 单选项直接结算。
                return resolve_guying_commit(tx, player, actor, card, options[0]);
            }
            open_choice(
                tx,
                actor,
                1,
                1,
                false,
                options,
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_GUYING_CHOICE,
                    player.raw(),
                    actor.raw(),
                    card.raw(),
                    skill.raw(),
                ],
            )
        }
        1 => {
            // 由续体直接结算。
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault("unknown Guying stage".into())),
    }
}

/// 孤影分支结算：0 随机交牌 / 1 收回弃牌堆的失去牌。
pub(crate) fn resolve_guying_commit(
    tx: &mut Transaction,
    player: PlayerId,
    actor: PlayerId,
    card: CardId,
    branch: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if branch == 0 {
        let gainable = crate::query::gainable_target_cards(
            tx.state(),
            player,
            actor,
            rule_api::TargetCardScope::HandEquip,
        );
        if gainable.is_empty() {
            return Ok(Vec::new());
        }
        let mut rng = crate::rng::RuleRng::from_state(tx.state().rng);
        let picked = gainable[rng.gen_range_u32(gainable.len() as u32) as usize];
        tx.state_mut().rng = rng.into_state();
        let from = tx
            .state()
            .cards
            .get(picked)
            .map(|value| value.zone)
            .ok_or_else(|| EffectError::Illegal("Guying gift card missing".into()))?;
        let to_hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?;
        return move_cards_with_reason(
            tx,
            vec![picked],
            to_hand,
            Some(from),
            engine_api::CardMoveReason::Give,
        );
    }
    // 收回失去的牌（若仍在弃牌堆）。
    let Some(from) = tx.state().cards.get(card).map(|value| value.zone) else {
        return Ok(Vec::new());
    };
    if !tx
        .state()
        .zones
        .get(from)
        .is_some_and(|zone| zone.kind == ZoneKind::DiscardPile)
    {
        return Ok(Vec::new());
    }
    let to_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?;
    move_cards(tx, vec![card], to_hand, Some(from))
}

/// 流冰（吴懿，ABI 426）：出牌阶段首次实体杀改为方块（直接修改实体牌花色）。
pub(crate) fn resolve_liubing(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) || card.is_none() {
        return Ok(Vec::new());
    }
    if let Some(value) = tx.state_mut().cards.get_mut(card) {
        if value.kind.is_sha() {
            value.suit = engine_api::Suit::Diamond;
        }
    }
    Ok(Vec::new())
}

/// 合击（吴懿，ABI 427）：其他角色单目标决斗/红杀后，可对原目标视为使用
/// 一张【杀】或【决斗】（二选一 choice）。
pub(crate) fn resolve_heji(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || target == player
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        vec![0, 1],
        ChoiceKind::Custom,
        ContinuationKind::Custom,
        vec![RESOLVE_HEJI_CHOICE, player.raw(), target.raw(), skill.raw()],
    )
}
