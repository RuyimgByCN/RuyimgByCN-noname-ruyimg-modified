//! Effect primitives for `resolve_potzhouyu_choice` and related helpers.

use super::*;

/// 从弃牌堆顶（向量尾）向堆底扫描，取至多 n 张牌名互不相同的基本牌。
pub(crate) fn zuozhan_distinct_basics_from_discard(
    tx: &Transaction,
    n: usize,
) -> Vec<engine_api::CardId> {
    let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
    let Some(discard) = discard else {
        return Vec::new();
    };
    let cards = tx.state().zones.get(discard).unwrap().cards.clone();
    let mut picked = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for card in cards.iter().rev() {
        if picked.len() >= n {
            break;
        }
        let Some(group) = tx
            .state()
            .cards
            .get(*card)
            .and_then(|state| zuozhan_basic_name_group(state.kind))
        else {
            continue;
        };
        if seen.insert(group) {
            picked.push(*card);
        }
    }
    picked
}

/// 专属状态机 (批 243, ADR 0641 §8.38)：坐瞻组技补牌——「坐瞻」标记角色死亡后
/// （全局 dieAfter），令一名存活的「坐瞻」角色从弃牌堆获得至多 X（射程标记）
/// 张牌名各不相同的基本牌。
pub(crate) fn resolve_jsrgzuozhan_gain(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    event_player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    // 权威校验：owner 存活；死亡者记有「坐瞻」标记；存在存活的坐瞻角色；
    // 弃牌堆存在可取的基本牌。
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || event_player == PlayerId::NONE
        || !tx
            .state()
            .players
            .get(event_player)
            .is_some_and(|participant| {
                participant
                    .marks
                    .iter()
                    .any(|(m, c)| *m == ZUOZHAN_SIT_MARK && *c > 0)
            })
    {
        return Ok(Vec::new());
    }
    let candidates: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| {
            candidate != event_player
                && tx
                    .state()
                    .players
                    .get(candidate)
                    .is_some_and(|participant| {
                        participant
                            .marks
                            .iter()
                            .any(|(m, c)| *m == ZUOZHAN_SIT_MARK && *c > 0)
                    })
        })
        .map(PlayerId::raw)
        .collect();
    if candidates.is_empty() {
        return Ok(Vec::new());
    }
    let skill_def = skill_pile_def(tx, player, skill)?;
    let quota = tx
        .state()
        .players
        .get(player)
        .and_then(|p| p.marks.iter().find(|(m, _)| *m == skill_def))
        .map_or(0, |(_, c)| *c)
        .max(0) as usize;
    if quota == 0 || zuozhan_distinct_basics_from_discard(tx, quota).is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        candidates,
        ChoiceKind::SelectTargets,
        ContinuationKind::ResolveJsrgzuozhanGainStage,
        vec![
            RESOLVE_ZUOZHAN_GAIN_CUSTOM_CHOICE,
            player.raw(),
            skill.raw(),
            event_player.raw(),
        ],
    )
}

/// 续体：selected[0] = 获得补牌的存活「坐瞻」角色。
pub(crate) fn resolve_jsrgzuozhan_gain_stage(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    _event_player: PlayerId,
    selected: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    let skill_def = skill_pile_def(tx, player, skill)?;
    let quota = tx
        .state()
        .players
        .get(player)
        .and_then(|p| p.marks.iter().find(|(m, _)| *m == skill_def))
        .map_or(0, |(_, c)| *c)
        .max(0) as usize;
    let cards = zuozhan_distinct_basics_from_discard(tx, quota);
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let target =
        PlayerId::new(*selected.first().ok_or_else(|| {
            EffectError::Illegal("resolve-jsrgzuozhan-gain target required".into())
        })?);
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("no target hand".into()))?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let mut events = Vec::new();
    for card in cards {
        tx.state_mut()
            .zones
            .get_mut(discard)
            .unwrap()
            .cards
            .retain(|c| *c != card);
        tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
        if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
            card_state.zone = hand;
            card_state.owner = target;
        }
        events.push(engine_api::DomainEvent::CardsDrawn {
            player: target,
            cards: vec![card],
        });
    }
    Ok(events)
}

/// 专属状态机 (批 241, ADR 0641 §8.36)：潜龙——受到伤害后，观看牌堆顶三张牌，
/// 获得至多已损失体力值张牌，其余置于牌堆底。
pub(crate) fn resolve_qianlong(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    let cards = tx.state().zones.get(draw).unwrap().cards.clone();
    if cards.len() < 3 {
        return Ok(Vec::new());
    }
    let top3 = [
        cards[cards.len() - 1],
        cards[cards.len() - 2],
        cards[cards.len() - 3],
    ];
    let participant = tx.state().players.get(player).unwrap();
    let lost = (participant.max_hp as i32 - participant.hp).max(0) as usize;
    let max_gain = lost.min(3);
    if max_gain == 0 {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        0,
        max_gain as u8,
        true,
        top3.iter().map(|card| card.raw()).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveQianlongStage,
        vec![
            RESOLVE_QIANLONG_MAGIC,
            player.raw(),
            skill.raw(),
            top3[0].raw(),
            top3[1].raw(),
            top3[2].raw(),
        ],
    )
}

/// 续体：selected = 获得牌；顶三张中未选的移至牌堆底（向量头部）。
pub(crate) fn resolve_qianlong_stage(
    tx: &mut Transaction,
    player: PlayerId,
    top3: [u32; 3],
    selected: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let chosen: std::collections::BTreeSet<u32> = selected.iter().copied().collect();
    let mut events = Vec::new();
    // 获得牌：从牌堆移入手牌。
    for raw in selected {
        let card = CardId::new(*raw);
        tx.state_mut()
            .zones
            .get_mut(draw)
            .unwrap()
            .cards
            .retain(|c| *c != card);
        tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
        if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
            card_state.zone = hand;
            card_state.owner = player;
        }
        events.push(engine_api::DomainEvent::CardsDrawn {
            player,
            cards: vec![card],
        });
    }
    // 未选的顶三张按原次序压入牌堆底（向量头部，先压最顶者使末选最靠底侧）。
    for raw in top3 {
        let card = CardId::new(raw);
        if chosen.contains(&raw) {
            continue;
        }
        if let Some(pos) = tx
            .state()
            .zones
            .get(draw)
            .unwrap()
            .cards
            .iter()
            .position(|c| *c == card)
        {
            let card_id = tx.state().zones.get(draw).unwrap().cards[pos];
            tx.state_mut()
                .zones
                .get_mut(draw)
                .unwrap()
                .cards
                .remove(pos);
            tx.state_mut()
                .zones
                .get_mut(draw)
                .unwrap()
                .cards
                .insert(0, card_id);
        }
    }
    Ok(events)
}

/// 专属状态机 (批 240, ADR 0641 §8.35)：蹈仁——出牌阶段限一次，将一张手牌交给
/// 一名其他角色，然后对处于你与其攻击范围交集内的所有角色各造成 1 点无来源
/// 伤害。载荷 6 槽：[magic, player, skill, stage, card, target]。
pub(crate) fn resolve_jsrgdaoren(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || tx.state().turn.phase != crate::state::Phase::Play
        || tx.state().hand_cards(player).is_empty()
        || !tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .any(|candidate| candidate != player)
    {
        return Ok(Vec::new());
    }
    let hand: Vec<u32> = tx
        .state()
        .hand_cards(player)
        .iter()
        .map(|card| card.raw())
        .collect();
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        hand,
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveJsrgdaorenStage,
        vec![
            RESOLVE_JSRGDAOREN_CUSTOM_CHOICE,
            player.raw(),
            skill.raw(),
            1,
            0,
            0,
        ],
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn jsrgdaoren_step(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
    card: u32,
    target: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let payload = |stage: u32, card: u32, target: u32| {
        vec![
            RESOLVE_JSRGDAOREN_CUSTOM_CHOICE,
            player.raw(),
            skill.raw(),
            stage,
            card,
            target,
        ]
    };
    match stage {
        1 => {
            // 手牌已选：选目标（其他存活角色）。
            let targets: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&candidate| candidate != player)
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
                false,
                targets,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveJsrgdaorenStage,
                payload(2, card, 0),
            )
        }
        2 => {
            // 目标已选：交牌 + 射程交集伤害。
            let target = PlayerId::new(target);
            let card_id = CardId::new(card);
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
            let target_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("no target hand".into()))?;
            if !tx.state().zones.get(hand).unwrap().cards.contains(&card_id) {
                return Err(EffectError::Illegal("jsrgdaoren card not in hand".into()));
            }
            tx.state_mut()
                .zones
                .get_mut(hand)
                .unwrap()
                .cards
                .retain(|c| *c != card_id);
            tx.state_mut()
                .zones
                .get_mut(target_hand)
                .unwrap()
                .cards
                .push(card_id);
            if let Some(card_state) = tx.state_mut().cards.get_mut(card_id) {
                card_state.zone = target_hand;
                card_state.owner = target;
            }
            // 射程交集：双方攻击范围均可及的所有存活角色（含自己/目标若在范围内）。
            let victims: Vec<PlayerId> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&candidate| {
                    crate::query::player_in_attack_range(tx.state(), player, candidate)
                        && crate::query::player_in_attack_range(tx.state(), target, candidate)
                })
                .collect();
            let mut events = Vec::new();
            for victim in victims {
                events.extend(super::super::pack_mobile::deal_damage(
                    tx,
                    PlayerId::NONE,
                    victim,
                    1,
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

/// 状态机续体入口（resolver 经由 crate::effect 重导出调用）。
pub(crate) fn jsrgdaoren_entry(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
    card: u32,
    target: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    jsrgdaoren_step(tx, player, skill, stage, card, target)
}

/// 专属状态机 (批 237, ADR 0641 §8.32)：拳甲——一名其他角色对另一角色造成伤害后
/// （来源为当前回合角色且有手牌），随机观看来源 (2+标记数) 张手牌：其中有【杀】
/// 则对其使用虚拟【杀】；否则获得 1 枚标记（下次多看一张）。
pub(crate) fn resolve_quanjia(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    source: PlayerId,
    target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    // 权威校验：来源非自己、目标非来源、来源为当前回合角色、来源存活且有手牌。
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || source == player
        || source == target
        || tx.state().turn.current != source
        || !tx.state().players.get(source).is_some_and(|participant| {
            participant.alive && !tx.state().hand_cards(source).is_empty()
        })
    {
        return Ok(Vec::new());
    }
    let marks = tx
        .state()
        .players
        .get(player)
        .and_then(|p| p.marks.iter().find(|(m, _)| *m == QUANJIA_MARK))
        .map_or(0, |(_, c)| *c) as u32;
    let num = 2 + marks;
    let mut hand = tx.state().hand_cards(source);
    // RNG 随机取样（ Fisher–Yates 局部 + 取前 num 张）。
    let mut rng = crate::rng::RuleRng::from_state(tx.state().rng);
    rng.shuffle(&mut hand);
    let sampled: Vec<engine_api::CardId> = hand.into_iter().take(num as usize).collect();
    tx.state_mut().rng = rng.into_state();
    // 其中有【杀】则对其使用虚拟【杀】；否则获得 1 枚标记。
    let has_sha = sampled.iter().any(|card| {
        tx.state()
            .cards
            .get(*card)
            .is_some_and(|state| state.kind == crate::state::CardKind::Sha)
    });
    if has_sha {
        let card =
            crate::effect::materialize_virtual_card(tx, player, crate::state::CardKind::Sha)?;
        crate::command::play::play_as_kind(
            tx,
            player,
            card,
            crate::state::CardKind::Sha,
            &[source],
            false,
            false,
            true,
            false,
        )?;
    } else {
        crate::effect::add_mark(tx, player, QUANJIA_MARK, 1)?;
    }
    Ok(Vec::new())
}

/// 变张置入（A0 置入面，ADR 0641 §8.12）：从自身手牌选 [min_count, max_count]

/// 专属状态机 (批 236, ADR 0641 §8.31)：祁县——出牌阶段限一次，重铸一张装备牌
/// （置入弃牌堆）并视为对一名其他角色使用一张【杀】或【过河拆桥】。
/// 载荷 7 槽：[magic, player, skill, stage, kind, material, target]。
pub(crate) fn resolve_pxqixian(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || tx.state().turn.phase != crate::state::Phase::Play
    {
        return Ok(Vec::new());
    }
    if pxqixian_equip_materials(tx, player).is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        vec![1, 2],
        ChoiceKind::Custom,
        ContinuationKind::ResolvePxqixianStage,
        vec![
            RESOLVE_PXQIXIAN_CUSTOM_CHOICE,
            player.raw(),
            skill.raw(),
            1,
            0,
            0,
            0,
        ],
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn pxqixian_step(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
    kind: u32,
    material: u32,
    target: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let payload = |stage: u32, kind: u32, material: u32, target: u32| {
        vec![
            RESOLVE_PXQIXIAN_CUSTOM_CHOICE,
            player.raw(),
            skill.raw(),
            stage,
            kind,
            material,
            target,
        ]
    };
    match stage {
        2 => {
            // kind 已选（1=sha, 2=guohe）：选重铸素材（装备牌）。
            let materials = pxqixian_equip_materials(tx, player);
            if materials.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                materials.into_iter().map(|card| card.raw()).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolvePxqixianStage,
                payload(2, kind, 0, 0),
            )
        }
        3 => {
            // 素材已选（material）：选目标。
            let targets: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&candidate| candidate != player)
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
                false,
                targets,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolvePxqixianStage,
                payload(3, kind, material, 0),
            )
        }
        4 => {
            // 目标已选：重铸素材置入弃牌堆 + 物化虚拟牌按规则使用。
            let material = CardId::new(material);
            let target = PlayerId::new(target);
            let card_kind = if kind == 1 {
                crate::state::CardKind::Sha
            } else {
                crate::state::CardKind::TrickGuohe
            };
            if let Some(from) = tx.state().cards.get(material).map(|state| state.zone) {
                let discard = tx
                    .state()
                    .zones
                    .find(crate::state::ZoneKind::DiscardPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
                tx.state_mut()
                    .zones
                    .get_mut(from)
                    .unwrap()
                    .cards
                    .retain(|c| *c != material);
                tx.state_mut()
                    .zones
                    .get_mut(discard)
                    .unwrap()
                    .cards
                    .push(material);
                if let Some(card_state) = tx.state_mut().cards.get_mut(material) {
                    card_state.zone = discard;
                    card_state.owner = PlayerId::NONE;
                }
            }
            let card = crate::effect::materialize_virtual_card(tx, player, card_kind)?;
            crate::command::play::play_as_kind(
                tx,
                player,
                card,
                card_kind,
                &[target],
                false,
                false,
                true,
                false,
            )?;
            Ok(Vec::new())
        }
        _ => Ok(Vec::new()),
    }
}

/// 手牌+装备区中的装备牌（重铸素材）。
pub(crate) fn pxqixian_equip_materials(
    tx: &Transaction,
    player: PlayerId,
) -> Vec<engine_api::CardId> {
    let mut materials = Vec::new();
    for zone_kind in [crate::state::ZoneKind::Hand, crate::state::ZoneKind::Equip] {
        if let Some(zone) = tx.state().zones.find(zone_kind, player) {
            for card in tx.state().zones.get(zone).unwrap().cards.clone() {
                if let Some(state) = tx.state().cards.get(card) {
                    if matches!(
                        state.kind,
                        crate::state::CardKind::Weapon
                            | crate::state::CardKind::Armor
                            | crate::state::CardKind::HorseOff
                            | crate::state::CardKind::HorseDef
                            | crate::state::CardKind::Treasure
                    ) {
                        materials.push(card);
                    }
                }
            }
        }
    }
    materials
}

/// 状态机续体入口（resolver 经由 crate::effect 重导出调用）。
pub(crate) fn resolve_dcbingji_entry(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
    kind: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    resolve_dcbingji(tx, player, skill, stage, kind)
}
