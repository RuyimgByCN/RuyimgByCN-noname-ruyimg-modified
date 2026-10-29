//! Effect primitives for `resolve_olqiaozhi` and related helpers.

use super::*;

/// 专属状态机 (批 276, ADR 0641 §8.64)：镇南——成为其他角色使用的锦囊目标
/// （目标数 > max(1, 使用者体力)）时，可弃一张牌并对任意角色造成 1 点伤害。
/// stage 0=选成本牌（可取消）→ 1=弃牌+选伤害目标（任意存活角色）→
/// 2=弃牌结算+伤害。
pub(crate) fn resolve_mbzhenann(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveMbzhenannStage,
        vec![RESOLVE_MBZHENANN_MAGIC, 1, player.raw()],
    )
}

/// 专属状态机 (批 273, ADR 0641 §8.61)：暗蓄——出牌阶段限一次，选两名手牌
/// 数不同的角色，手牌少者从多者处选得一张手牌；所得牌非黑桃则 owner 摸 1，
/// 此后两人手牌数相等则 owner 回 1。stage 0=少者从多者手牌选 1 →
/// 1=移牌+花色/相等结算。
pub(crate) fn resolve_dcanxu(
    tx: &mut Transaction,
    player: PlayerId,
    targets: &[PlayerId],
) -> Result<Vec<DomainEvent>, EffectError> {
    if targets.len() != 2 || targets[0] == targets[1] {
        return Err(EffectError::Illegal(
            "resolve-dcanxu requires exactly two distinct targets".into(),
        ));
    }
    let (a, b) = (targets[0], targets[1]);
    let (a_count, b_count) = (tx.state().hand_count(a), tx.state().hand_count(b));
    if a_count == b_count {
        return Err(EffectError::Illegal(
            "resolve-dcanxu targets must differ in hand count".into(),
        ));
    }
    let (gainner, giver) = if a_count < b_count { (a, b) } else { (b, a) };
    let options: Vec<u32> = tx
        .state()
        .hand_cards(giver)
        .iter()
        .map(|card| card.raw())
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        gainner,
        1,
        1,
        false,
        options,
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveDcanxuStage,
        vec![
            RESOLVE_DCANXU_MAGIC,
            player.raw(),
            gainner.raw(),
            giver.raw(),
            1,
        ],
    )
}

/// 专属状态机 (批 272, ADR 0641 §8.60)：度断——每回合一次，成为【杀】目标
/// 时可重铸一张牌；再选一项：①使用者摸 2 且此杀对 owner 无效；②使用者弃
/// 一张牌且 owner 不可响应此杀。stage 0=选重铸牌 → 1=重铸+双选（使用者无
/// 可弃牌时强制①）→ 2=分支执行/选使用者弃牌 → 3=使用者弃牌+不可响应。
pub(crate) fn resolve_duoduan(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveDuoduanStage,
        vec![
            RESOLVE_DUODUAN_MAGIC,
            player.raw(),
            source.raw(),
            card.raw(),
            skill.raw(),
            1,
        ],
    )
}

/// 选项①：使用者摸 2 张，此【杀】对 owner 无效。排除在解析期内同步生效
/// （直接改 sha 挂起载荷；经 CancelIncomingCardUse 尾随提交会在响应询问
/// 之后才落地，且附带「获得实体牌」的杜达语义，与度断不符）。
pub(crate) fn resolve_duoduan_draw_and_cancel(
    tx: &mut Transaction,
    source: PlayerId,
    player: PlayerId,
    card: CardId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let mut events = crate::effect::draw_cards(tx, source, 2)?;
    crate::command::play::exclude_saved_card_target(tx, card, player)?;
    Ok(events)
}

/// 告援候选：非 owner、非使用者的存活角色，持「诤」标，且为此【杀】在
/// 使用者射程内的合法目标（转移语义 targetEnabled）。
pub(crate) fn gaoyuan_redirect_candidates(
    tx: &Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
) -> Result<Vec<PlayerId>, EffectError> {
    let legal = crate::command::play::saved_sha_redirect_targets_in_attack_range(
        tx, source, card, player, source,
    )?;
    Ok(legal
        .into_iter()
        .filter(|candidate| {
            *candidate != source
                && tx
                    .state()
                    .players
                    .get(*candidate)
                    .is_some_and(|participant| {
                        participant.marks.iter().any(|(mark, count)| {
                            *mark == crate::effect::ZHENGJIAN_MARK && *count > 0
                        })
                    })
        })
        .collect())
}

/// 专属状态机 (批 271, ADR 0641 §8.59)：告援——成为【杀】目标时，可弃一张
/// 牌，将此【杀】转移给一名有「诤」标且为此杀合法目标的其他角色。
/// stage 0：选弃牌（0..1 可取消）→ stage 1：选转移目标（1..1 可取消）→
/// stage 2：弃牌 + redirect_saved_sha_target + CardTargetAdded。
pub(crate) fn resolve_gaoyuan(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
    let candidates = gaoyuan_redirect_candidates(tx, player, source, card)?;
    if options.is_empty() || candidates.is_empty() {
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
        ContinuationKind::ResolveGaoyuanStage,
        vec![
            RESOLVE_GAOYUAN_MAGIC,
            player.raw(),
            source.raw(),
            card.raw(),
            1,
        ],
    )
}

/// 专属状态机 (批 269, ADR 0641 §8.57)：孛明——游戏开始时，令至多两名角色各
/// 获得牌堆中一张攻击范围等于其手牌花色数的武器牌（无 distance 信息的武器
/// 视作范围 1）。
pub(crate) fn resolve_mbbeiming(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let candidates: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(PlayerId::raw)
                .collect();
            if candidates.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                2,
                true,
                candidates,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveMbbeimingStage,
                vec![RESOLVE_MBBEIMING_MAGIC, player.raw(), skill.raw(), 0],
            )
        }
        _ => Ok(Vec::new()),
    }
}

/// stage 2（解析器直调）：对每个选定目标，从牌堆搜索攻击范围 == 其手牌花色数
/// 的武器牌并置入其手牌区；找不到则跳过该目标。
pub(crate) fn resolve_mbbeiming_grant(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    targets: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = player;
    let _ = skill;
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("draw pile is missing".into()))?;
    let mut events = Vec::new();
    for raw in targets {
        let target = PlayerId::new(*raw);
        if !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        {
            continue;
        }
        let suits_count = {
            let mut seen: [bool; 4] = [false; 4];
            for card in tx.state().hand_cards(target) {
                if let Some(card) = tx.state().cards.get(card) {
                    match card.suit {
                        Suit::Spade => seen[0] = true,
                        Suit::Heart => seen[1] = true,
                        Suit::Club => seen[2] = true,
                        Suit::Diamond => seen[3] = true,
                        Suit::None => {}
                    }
                }
            }
            seen.into_iter().filter(|seen| *seen).count()
        };
        let equip = tx.state().zones.get(draw).and_then(|zone| {
            zone.cards.iter().rev().copied().find(|card| {
                tx.state().cards.get(*card).is_some_and(|card| {
                    card.kind == CardKind::Weapon
                        && u32::from(card.weapon_range.max(1)) == suits_count as u32
                })
            })
        });
        let Some(equip) = equip else {
            continue;
        };
        let hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, target)
            .ok_or_else(|| EffectError::Illegal("no target hand".into()))?;
        events.extend(move_cards(tx, vec![equip], hand, Some(draw))?);
    }
    Ok(events)
}

/// 专属状态机 (批 267, ADR 0641 §8.55)：诤荐——mode 0：结束阶段选一名角色
/// 授一枚「诤」标记；mode 1/2：该角色用牌/打出牌时标记 +1（机内直读直加，
/// 无选择）；mode 3：owner 回合开始，各有标角色摸 min(5, 上限, 标数) 张并
/// 清标。
pub(crate) fn resolve_zhengjian(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
    event_player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let targets: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(PlayerId::raw)
                .collect();
            if targets.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                targets,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveZhengjianStage,
                vec![RESOLVE_ZHENGJIAN_MAGIC, player.raw(), skill.raw(), 0],
            )
        }
        1 | 2 => {
            // 用牌/打出计数：目标带标则标数 +1（无选择，inline）。
            if event_player != PlayerId::NONE
                && tx
                    .state()
                    .players
                    .get(event_player)
                    .is_some_and(|p| p.marks.iter().any(|(m, c)| *m == ZHENGJIAN_MARK && *c > 0))
            {
                events_extend_add_mark(tx, event_player, ZHENGJIAN_MARK, 1)?;
            }
            Ok(Vec::new())
        }
        3 => {
            // owner 回合开始：各有标角色摸 min(5, 上限, 标数) 张并清标。
            let mut events = Vec::new();
            for participant in tx.state().players.alive_by_seat() {
                let marks = tx
                    .state()
                    .players
                    .get(participant)
                    .and_then(|p| p.marks.iter().find(|(m, _)| *m == ZHENGJIAN_MARK))
                    .map_or(0, |(_, c)| *c);
                if marks <= 0 {
                    continue;
                }
                let draw_n = marks.min(5).max(0) as u32;
                if draw_n > 0 {
                    events.extend(crate::effect::draw_cards(tx, participant, draw_n)?);
                }
                if let Some(p) = tx.state_mut().players.get_mut(participant) {
                    p.marks.retain(|(m, _)| *m != ZHENGJIAN_MARK);
                }
            }
            Ok(events)
        }
        _ => Ok(Vec::new()),
    }
}

pub(crate) fn events_extend_add_mark(
    tx: &mut Transaction,
    player: PlayerId,
    mark: u32,
    delta: i32,
) -> Result<(), EffectError> {
    crate::effect::add_mark(tx, player, mark, delta)?;
    Ok(())
}

/// 专属状态机 (批 268, ADR 0641 §8.56)：良姻——①牌被置于武将牌上时（cards_
/// moved 入 expansion 区），你可令一名角色摸一张牌；②牌从武将牌上被获得时
/// （cards_moved 出 expansion 区），你可令一名角色弃置一张手牌。
pub(crate) fn resolve_reliangyin(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    mode: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let targets: Vec<u32> = match mode {
        0 => tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .map(PlayerId::raw)
            .collect(),
        1 => tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|&candidate| !tx.state().hand_cards(candidate).is_empty())
            .map(PlayerId::raw)
            .collect(),
        _ => Vec::new(),
    };
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        targets,
        ChoiceKind::SelectTargets,
        ContinuationKind::ResolveReliangyinStage,
        vec![RESOLVE_LIANGYIN_MAGIC, player.raw(), skill.raw(), mode],
    )
}

/// 续体：selections[0] = 受益/弃牌角色。
pub(crate) fn resolve_reliangyin_stage(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    mode: u32,
    who_raw: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    let who = PlayerId::new(who_raw);
    match mode {
        0 => crate::effect::draw_cards(tx, who, 1),
        1 => {
            // 该角色弃置一张手牌（其自选，强制）。
            let hand: Vec<u32> = tx
                .state()
                .hand_cards(who)
                .iter()
                .map(|card| card.raw())
                .collect();
            if hand.is_empty() {
                return Ok(Vec::new());
            }
            let payload = vec![RESOLVE_LIANGYIN_MAGIC, who.raw(), 0, 9];
            open_choice(
                tx,
                who,
                1,
                1,
                false,
                hand,
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveReliangyinStage,
                payload,
            )?;
            Ok(Vec::new())
        }
        _ => Ok(Vec::new()),
    }
}

/// mode 1 子续体：who 选定的手牌弃置。
pub(crate) fn resolve_reliangyin_discard(
    tx: &mut Transaction,
    who: PlayerId,
    card_raw: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let card = CardId::new(card_raw);
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, who)
        .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
    if !tx.state().zones.get(hand).unwrap().cards.contains(&card) {
        return Err(EffectError::Illegal("liangyin card outside hand".into()));
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    tx.state_mut()
        .zones
        .get_mut(hand)
        .unwrap()
        .cards
        .retain(|c| *c != card);
    tx.state_mut()
        .zones
        .get_mut(discard)
        .unwrap()
        .cards
        .push(card);
    if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
        card_state.zone = discard;
    }
    Ok(Vec::new())
}
