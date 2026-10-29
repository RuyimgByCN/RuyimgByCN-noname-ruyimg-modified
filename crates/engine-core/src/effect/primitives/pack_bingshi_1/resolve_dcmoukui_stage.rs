//! Effect primitives for `resolve_dcmoukui_stage` and related helpers.

use super::*;

/// stage 2：知情选择已定 → 应用。
pub(crate) fn resolve_dcmoukui_stage(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    target: PlayerId,
    selections: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    let mut events = Vec::new();
    let draw = selections.contains(&1);
    let discard = selections.contains(&2);
    if draw && discard {
        // 双选：记录追罚标记（此杀未命中 → 目标弃 owner 一张手牌）。
        tx.state_mut()
            .skills
            .storage_set(skill, DCMOUKUI_FLAG_KEY, 1);
    }
    if draw {
        events.extend(crate::effect::draw_cards(tx, player, 1)?);
    }
    if discard {
        // owner 从目标手牌选一张弃置。
        let options: Vec<u32> = tx
            .state()
            .hand_cards(target)
            .iter()
            .map(|card| card.raw())
            .collect();
        if options.is_empty() {
            return Ok(events);
        }
        let payload = vec![
            RESOLVE_DCMOUKUI_MAGIC,
            player.raw(),
            skill.raw(),
            3,
            target.raw(),
        ];
        open_choice(
            tx,
            player,
            1,
            1,
            false,
            options,
            ChoiceKind::SelectCards,
            ContinuationKind::ResolveDcmoukuiStage,
            payload,
        )?;
    }
    Ok(events)
}

/// stage 3：弃置目标手牌 selections[0]。
pub(crate) fn resolve_dcmoukui_discard(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    target: PlayerId,
    card_raw: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    let card = CardId::new(card_raw);
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("no target hand".into()))?;
    if !tx.state().zones.get(hand).unwrap().cards.contains(&card) {
        return Err(EffectError::Illegal(
            "dcmoukui card outside target hand".into(),
        ));
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

/// 专属状态机 (批 262, ADR 0641 §8.51)：忠言——出牌阶段限一次，亮出牌堆顶
/// 三张牌，一名有手牌的角色以其中一张与其一张手牌交换（chooseToMove 族）。
pub(crate) fn resolve_zhongyan(
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
            let targets: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&candidate| !tx.state().hand_cards(candidate).is_empty())
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
                ContinuationKind::ResolveZhongyanStage,
                vec![RESOLVE_ZHONGYAN_MAGIC, player.raw(), skill.raw(), 0],
            )
        }
        _ => Ok(Vec::new()),
    }
}

/// 目标已选（resolver 调用）：亮出牌堆顶三张，目标选其中一张。
pub(crate) fn resolve_zhongyan_stage(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    target_raw: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let target = PlayerId::new(target_raw);
    if tx.state().hand_cards(target).is_empty() {
        return Ok(Vec::new());
    }
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    let top = tx.state().zones.get(draw).unwrap().cards.clone();
    if top.is_empty() {
        return Ok(Vec::new());
    }
    // 牌堆顶 = 向量尾部。
    let take = top.len().min(3);
    let revealed: Vec<CardId> = top[top.len() - take..].to_vec();
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::RevealCards {
            cards: revealed.clone(),
            to: Vec::new(),
        }],
    )?;
    let mut payload = vec![
        RESOLVE_ZHONGYAN_MAGIC,
        player.raw(),
        skill.raw(),
        3,
        target_raw,
        revealed.len() as u32,
    ];
    payload.extend(revealed.iter().map(|c| c.raw()));
    open_choice(
        tx,
        target,
        1,
        1,
        false,
        revealed.iter().map(|c| c.raw()).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveZhongyanStage,
        payload,
    )?;
    Ok(Vec::new())
}

/// 目标已选亮牌（resolver 调用）：目标再选自己一张手牌交换。
pub(crate) fn resolve_zhongyan_pick(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    target_raw: u32,
    revealed: &[u32],
    revealed_pick: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let target = PlayerId::new(target_raw);
    let hand: Vec<u32> = tx
        .state()
        .hand_cards(target)
        .iter()
        .map(|card| card.raw())
        .collect();
    let mut payload = vec![
        RESOLVE_ZHONGYAN_MAGIC,
        player.raw(),
        skill.raw(),
        4,
        target_raw,
        revealed.len() as u32,
    ];
    payload.extend_from_slice(revealed);
    payload.push(revealed_pick);
    open_choice(
        tx,
        target,
        1,
        1,
        false,
        hand,
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveZhongyanStage,
        payload,
    )?;
    Ok(Vec::new())
}

/// 交换结算：所选亮牌入手牌，所选手牌置回牌堆顶（原亮牌位置）。
pub(crate) fn resolve_zhongyan_swap(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    target_raw: u32,
    revealed: &[u32],
    revealed_pick: u32,
    hand_card: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = (player, skill);
    let target = PlayerId::new(target_raw);
    let pick = CardId::new(revealed_pick);
    let hand_card = CardId::new(hand_card);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("no target hand".into()))?;
    // 亮牌 → 目标手牌。
    if let Some(pos) = tx
        .state()
        .zones
        .get(draw)
        .unwrap()
        .cards
        .iter()
        .position(|c| *c == pick)
    {
        tx.state_mut()
            .zones
            .get_mut(draw)
            .unwrap()
            .cards
            .remove(pos);
    }
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(pick);
    if let Some(card_state) = tx.state_mut().cards.get_mut(pick) {
        card_state.zone = hand;
        card_state.owner = target;
    }
    // 目标手牌 → 牌堆顶（原亮牌位置；找不到则置顶）。
    let insert_pos = {
        let cards = &tx.state().zones.get(draw).unwrap().cards;
        cards
            .iter()
            .position(|c| revealed.contains(&c.raw()))
            .unwrap_or(0)
    };
    if let Some(pos) = {
        let cards = &tx.state().zones.get(hand).unwrap().cards;
        cards.iter().position(|c| *c == hand_card)
    } {
        tx.state_mut()
            .zones
            .get_mut(hand)
            .unwrap()
            .cards
            .remove(pos);
    }
    tx.state_mut()
        .zones
        .get_mut(draw)
        .unwrap()
        .cards
        .insert(insert_pos, hand_card);
    if let Some(card_state) = tx.state_mut().cards.get_mut(hand_card) {
        card_state.zone = draw;
        card_state.owner = PlayerId::NONE;
    }
    Ok(Vec::new())
}

/// 专属状态机 (批 263, ADR 0641 §8.52)：狐魅——出牌阶段，选择一名体力不超过
/// owner 本回合造成伤害总数的角色，令其执行一项：摸一张牌／交给 owner 一张
/// 手牌／回复 1 点体力（受伤时）。
pub(crate) fn resolve_chumei(
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
            // 门控：本回合造成伤害总数 > 0（否则无目标满足体力 ≤ 该值）。
            let dealt = tx.state().turn.damage_dealt_count(player);
            if dealt == 0 {
                return Ok(Vec::new());
            }
            let dealt = dealt as i32;
            let targets: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&candidate| {
                    candidate != player
                        && tx
                            .state()
                            .players
                            .get(candidate)
                            .is_some_and(|p| p.alive && p.hp <= dealt)
                })
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
                ContinuationKind::ResolveChumeiStage,
                vec![RESOLVE_CHUMEI_MAGIC, player.raw(), skill.raw(), 0],
            )
        }
        _ => Ok(Vec::new()),
    }
}

/// stage 2：目标已定 → 三选一（1=目标摸 1；2=目标交 owner 一张手牌；
/// 3=目标回复 1，仅目标受伤时可选）。
pub(crate) fn resolve_chumei_stage(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    target_raw: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let target = PlayerId::new(target_raw);
    let target_damaged = tx
        .state()
        .players
        .get(target)
        .is_some_and(|p| p.hp < p.max_hp as i32);
    let mut options = vec![1u32];
    if !tx.state().hand_cards(target).is_empty() {
        options.push(2);
    }
    if target_damaged {
        options.push(3);
    }
    let payload = vec![
        RESOLVE_CHUMEI_MAGIC,
        player.raw(),
        skill.raw(),
        2,
        target_raw,
        options.len() as u32,
    ]
    .into_iter()
    .chain(options.iter().copied())
    .collect::<Vec<u32>>();
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options,
        ChoiceKind::Custom,
        ContinuationKind::ResolveChumeiStage,
        payload,
    )?;
    Ok(Vec::new())
}

/// stage 2 选「给牌」后的子选择：目标从手牌选一张交给 owner。
pub(crate) fn resolve_chumei_give_pick(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    target_raw: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let target = PlayerId::new(target_raw);
    let hand: Vec<u32> = tx
        .state()
        .hand_cards(target)
        .iter()
        .map(|card| card.raw())
        .collect();
    let payload = vec![
        RESOLVE_CHUMEI_MAGIC,
        player.raw(),
        skill.raw(),
        4,
        target_raw,
    ];
    open_choice(
        tx,
        target,
        1,
        1,
        false,
        hand,
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveChumeiStage,
        payload,
    )?;
    Ok(Vec::new())
}

/// stage 4：目标选定的手牌交给 owner。
pub(crate) fn resolve_chumei_give(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    target_raw: u32,
    card_raw: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    let target = PlayerId::new(target_raw);
    let card = CardId::new(card_raw);
    let from = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("no target hand".into()))?;
    if !tx.state().zones.get(from).unwrap().cards.contains(&card) {
        return Err(EffectError::Illegal(
            "chumei card outside target hand".into(),
        ));
    }
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no owner hand".into()))?;
    tx.state_mut()
        .zones
        .get_mut(from)
        .unwrap()
        .cards
        .retain(|c| *c != card);
    tx.state_mut().zones.get_mut(to).unwrap().cards.push(card);
    if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
        card_state.zone = to;
        card_state.owner = player;
    }
    Ok(Vec::new())
}

/// 追罚入口（stage 5）：杀未命中（ShaDodged）且双选标记在 → 目标（闪避者）
/// 弃置 owner 一张手牌。
pub(crate) fn resolve_dcmoukui_conseq(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    dodger: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if tx.state().skills.storage_get(skill, DCMOUKUI_FLAG_KEY) != 1 {
        return Ok(Vec::new());
    }
    // 消耗标记。
    tx.state_mut()
        .skills
        .storage_set(skill, DCMOUKUI_FLAG_KEY, 0);
    if !tx
        .state()
        .players
        .get(dodger)
        .is_some_and(|participant| participant.alive)
        || tx.state().hand_cards(player).is_empty()
    {
        return Ok(Vec::new());
    }
    let options: Vec<u32> = tx
        .state()
        .hand_cards(player)
        .iter()
        .map(|card| card.raw())
        .collect();
    let payload = vec![
        RESOLVE_DCMOUKUI_MAGIC,
        player.raw(),
        skill.raw(),
        6,
        dodger.raw(),
    ];
    open_choice(
        tx,
        dodger,
        1,
        1,
        false,
        options,
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveDcmoukuiStage,
        payload,
    )?;
    Ok(Vec::new())
}

/// stage 6：闪避者选定的 owner 手牌弃置。
pub(crate) fn resolve_dcmoukui_conseq_discard(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    dodger: PlayerId,
    card_raw: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    let card = CardId::new(card_raw);
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no owner hand".into()))?;
    if !tx.state().zones.get(hand).unwrap().cards.contains(&card) {
        return Err(EffectError::Illegal(
            "dcmoukui conseq card outside owner hand".into(),
        ));
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

/// 专属状态机 (批 265, ADR 0641 §8.53)：勤勉——mode 0：准备阶段摸 1 张或
/// 受到伤害后按伤害点数摸牌，摸得牌打「勤勉」标记；mode 1：全场结束阶段，
/// 可弃置手牌中全部标记牌并对当前回合角色造成等量伤害（若手牌全部为标记牌
/// 则等值为其张数，否则 1 点）。
pub(crate) fn resolve_qinmian(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    mode: u32,
    amount: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let mut events: Vec<DomainEvent> = Vec::new();
    match mode {
        0 => {
            let draw_n = amount.max(1);
            let mut drawn = crate::effect::draw_cards(tx, player, draw_n)?;
            if let Some(engine_api::DomainEvent::CardsDrawn { cards, .. }) = drawn.last() {
                for card in cards {
                    tx.state_mut().add_content_card_tag(*card, QINMIAN_TAG);
                }
            }
            events.extend(drawn);
            Ok(events)
        }
        1 => {
            // 全场结束阶段（当前回合角色 = state.turn.current）。
            let current = tx.state().turn.current;
            let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
            let tagged: Vec<CardId> = tx
                .state()
                .zones
                .get(hand)
                .unwrap()
                .cards
                .iter()
                .copied()
                .filter(|card| tx.state().card_has_content_tag(*card, QINMIAN_TAG))
                .collect();
            if tagged.is_empty() {
                return Ok(Vec::new());
            }
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
            for card in &tagged {
                tx.state_mut()
                    .zones
                    .get_mut(hand)
                    .unwrap()
                    .cards
                    .retain(|c| *c != *card);
                tx.state_mut()
                    .zones
                    .get_mut(discard)
                    .unwrap()
                    .cards
                    .push(*card);
                if let Some(card_state) = tx.state_mut().cards.get_mut(*card) {
                    card_state.zone = discard;
                }
            }
            // 伤害值：手牌全部为标记牌 → 等于张数；否则 1 点（上游 isAllQinmian）。
            let hand_now = tx.state().hand_cards(player).len();
            let damage = if hand_now == 0 {
                tagged.len() as u32
            } else {
                1
            };
            if current != PlayerId::NONE && damage > 0 {
                events.extend(super::super::pack_mobile::deal_damage(
                    tx,
                    player,
                    current,
                    damage,
                    false,
                    0,
                    false,
                    CardId::NONE,
                    0,
                    0,
                )?);
            }
            Ok(events)
        }
        _ => Ok(Vec::new()),
    }
}
