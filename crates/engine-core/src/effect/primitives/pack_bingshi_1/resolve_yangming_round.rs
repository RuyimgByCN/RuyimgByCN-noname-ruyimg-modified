//! Effect primitives for `resolve_twmitu_round` and related helpers.

use super::*;

/// 拼点轮各阶段确认（resolver 调用）。
pub(crate) fn resolve_yangming_round(
    tx: &mut Transaction,
    observer: PlayerId,
    player: PlayerId,
    skill: engine_api::SkillId,
    payload: &[u32],
    selections: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    const HEADER: usize = 6;
    if payload.len() < HEADER || payload[0] != RESOLVE_YANGMING_MAGIC {
        return Err(EffectError::EngineFault(
            "bad resolve-yangming continuation payload".into(),
        ));
    }
    let stage = payload[3];
    let target_raw = payload[4];
    let losses = payload[5];
    let pend: Vec<u32> = payload[HEADER..].to_vec();
    let target = PlayerId::new(target_raw);
    match stage {
        2 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "resolve-yangming compare card requires owner".into(),
                ));
            }
            let c_p = *selections.first().ok_or_else(|| {
                EffectError::Illegal("resolve-yangming compare card required".into())
            })?;
            let owner_hand: std::collections::BTreeSet<u32> = tx
                .state()
                .hand_cards(player)
                .iter()
                .map(|card| card.raw())
                .collect();
            if !owner_hand.contains(&c_p) {
                return Err(EffectError::Illegal(
                    "resolve-yangming compare card outside owner hand".into(),
                ));
            }
            let t_hand: Vec<u32> = tx
                .state()
                .hand_cards(target)
                .iter()
                .map(|card| card.raw())
                .collect();
            let payload = yangming_payload(player, skill, 3, target_raw, losses, &[c_p]);
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                t_hand,
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveYangmingStage,
                payload,
            )?;
            Ok(Vec::new())
        }
        3 => {
            if observer != target {
                return Err(EffectError::Illegal(
                    "resolve-yangming compare card requires the target".into(),
                ));
            }
            if pend.len() != 1 {
                return Err(EffectError::EngineFault(
                    "resolve-yangming missing owner compare card".into(),
                ));
            }
            let c_p = pend[0];
            let c_t = *selections.first().ok_or_else(|| {
                EffectError::Illegal("resolve-yangming compare card required".into())
            })?;
            let target_hand: std::collections::BTreeSet<u32> = tx
                .state()
                .hand_cards(target)
                .iter()
                .map(|card| card.raw())
                .collect();
            if !target_hand.contains(&c_t) {
                return Err(EffectError::Illegal(
                    "resolve-yangming compare card outside target hand".into(),
                ));
            }
            // 拼点牌置入弃牌堆（chooseToCompare 规则）。
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
            let mut events = Vec::new();
            for (holder, raw) in [(player, c_p), (target, c_t)] {
                let card = CardId::new(raw);
                if let Some(hand_zone) = tx.state().zones.find(ZoneKind::Hand, holder) {
                    tx.state_mut()
                        .zones
                        .get_mut(hand_zone)
                        .unwrap()
                        .cards
                        .retain(|c| *c != card);
                }
                tx.state_mut()
                    .zones
                    .get_mut(discard)
                    .unwrap()
                    .cards
                    .push(card);
                if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
                    card_state.zone = discard;
                }
            }
            let p_rank = tx
                .state()
                .cards
                .get(CardId::new(c_p))
                .map(|c| c.rank)
                .unwrap_or(0);
            let t_rank = tx
                .state()
                .cards
                .get(CardId::new(c_t))
                .map(|c| c.rank)
                .unwrap_or(0);
            if t_rank > p_rank {
                // 目标赢：摸已输次数张 + owner 回复 1，结束。
                if losses > 0 {
                    events.extend(crate::effect::draw_cards(tx, target, losses)?);
                }
                events.extend(super::super::recover_hp(tx, player, player, 1)?);
                return Ok(events);
            }
            // owner 赢或平：记一次输。
            let losses2 = losses + 1;
            if !yangming_can_compare(tx, player, target) {
                return Ok(events);
            }
            // stage 4：owner 选择是否重复。
            let payload = yangming_payload(player, skill, 4, target_raw, losses2, &[]);
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                vec![1, 2],
                ChoiceKind::Custom,
                ContinuationKind::ResolveYangmingStage,
                payload,
            )?;
            Ok(events)
        }
        4 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "resolve-yangming repeat requires owner".into(),
                ));
            }
            let repeat = *selections.first().ok_or_else(|| {
                EffectError::Illegal("resolve-yangming repeat choice required".into())
            })?;
            if repeat == 1 && yangming_can_compare(tx, player, target) {
                resolve_yangming_stage(tx, player, skill, target_raw, losses)?;
            }
            Ok(Vec::new())
        }
        _ => Err(EffectError::EngineFault(
            "bad resolve-yangming stage".into(),
        )),
    }
}

/// 装备区牌数。
pub(crate) fn canyun_equip_count(tx: &Transaction, player: PlayerId) -> usize {
    tx.state()
        .zones
        .find(ZoneKind::Equip, player)
        .map(|zone| tx.state().zones.get(zone).unwrap().cards.len())
        .unwrap_or(0)
}

/// 专属状态机 (批 255, ADR 0641 §8.44)：残殒——出牌阶段，可弃置一张牌并
/// 选择 1-2 名其他角色，比较装备数：owner 多则其回复 1 体力，相等则其摸
/// 1 张牌，其多则其失去 1 体力；目标体力为 1 时 owner 摸一张牌。
pub(crate) fn resolve_canyun(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if stage != 0 {
        return Ok(Vec::new());
    }
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || tx.state().hand_cards(player).is_empty()
    {
        return Ok(Vec::new());
    }
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
    // stage 0：弃置一张手牌（费用）。
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
        true,
        hand,
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveCanyunStage,
        vec![RESOLVE_CANYUN_MAGIC, player.raw(), skill.raw(), 2],
    )
}

/// stage 2：弃牌已定 → 选 1-2 名其他角色。
pub(crate) fn resolve_canyun_discarded(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    discard_card: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let targets: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| candidate != player)
        .map(PlayerId::raw)
        .collect();
    let mut payload = vec![
        RESOLVE_CANYUN_MAGIC,
        player.raw(),
        skill.raw(),
        3,
        discard_card,
    ];
    open_choice(
        tx,
        player,
        1,
        2,
        true,
        targets,
        ChoiceKind::SelectTargets,
        ContinuationKind::ResolveCanyunStage,
        payload,
    )
}

/// stage 3：目标已定 → 逐目标比较装备数结算；目标体力为 1 时 owner 摸 1。
pub(crate) fn resolve_canyun_resolve(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    discard_card: u32,
    targets: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    let owner_equip = canyun_equip_count(tx, player) as i32;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let player_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no player hand".into()))?;
    let mut events = Vec::new();
    let mut owner_draws = 0i32;

    // 弃置费用牌。
    let cost = CardId::new(discard_card);
    tx.state_mut()
        .zones
        .get_mut(player_hand)
        .unwrap()
        .cards
        .retain(|c| *c != cost);
    tx.state_mut()
        .zones
        .get_mut(discard)
        .unwrap()
        .cards
        .push(cost);
    if let Some(card_state) = tx.state_mut().cards.get_mut(cost) {
        card_state.zone = discard;
    }

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
        let target_equip = canyun_equip_count(tx, target) as i32;
        if owner_equip > target_equip {
            events.extend(super::super::recover_hp(tx, player, target, 1)?);
        } else if owner_equip == target_equip {
            events.extend(crate::effect::draw_cards(tx, target, 1)?);
        } else {
            // 目标多：其失去 1 体力（体力流失通道）。
            events.extend(super::super::deal_damage(
                tx,
                PlayerId::NONE,
                target,
                1,
                true,
                0,
                false,
                CardId::NONE,
                0,
                0,
            )?);
        }
        if hp_of_player(tx, target) == 1 {
            owner_draws += 1;
        }
    }
    if owner_draws > 0 {
        events.extend(crate::effect::draw_cards(tx, player, owner_draws as u32)?);
    }
    Ok(events)
}

pub(crate) fn hp_of_player(tx: &Transaction, player: PlayerId) -> i32 {
    tx.state()
        .players
        .get(player)
        .map(|participant| participant.hp)
        .unwrap_or(0)
}

/// 专属状态机 (批 256, ADR 0641 §8.45)：横骛——使用/打出牌后，若手牌无此
/// 花色且场上有同花色装备：摸「场上该花色装备数」张（frequent 锁定自动）。
pub(crate) fn resolve_hengwu(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    event_card: u32,
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
    // 事件牌花色（无牌/无色 → 结束）。
    let Some(suit) = tx
        .state()
        .cards
        .get(CardId::new(event_card))
        .map(|card| card.suit)
        .filter(|suit| {
            matches!(
                suit,
                engine_api::Suit::Spade
                    | engine_api::Suit::Heart
                    | engine_api::Suit::Club
                    | engine_api::Suit::Diamond
            )
        })
    else {
        return Ok(Vec::new());
    };
    // 门控：手牌无此花色。
    if tx.state().hand_cards(player).iter().any(|card| {
        tx.state()
            .cards
            .get(*card)
            .is_some_and(|card_state| card_state.suit == suit)
    }) {
        return Ok(Vec::new());
    }
    // 场上同花色装备数。
    let mut count = 0usize;
    for zone in tx.state().zones.sorted_live() {
        if zone.kind != ZoneKind::Equip {
            continue;
        }
        for card in &zone.cards {
            if tx
                .state()
                .cards
                .get(*card)
                .is_some_and(|card_state| card_state.suit == suit)
            {
                count += 1;
            }
        }
    }
    if count == 0 {
        return Ok(Vec::new());
    }
    crate::effect::draw_cards(tx, player, count as u32)
}

/// 专属状态机 (批 257, ADR 0641 §8.46)：表召·归——准备阶段若「表」堆有牌：
/// 弃置首张，令一名角色（可选自己，可取消）摸至全场最大手牌数（至多 5）并
/// 回复 1 点体力。全场最大手牌数机内计算。
pub(crate) fn resolve_biaozhao3(
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
            // 门控：「表」堆非空（forced 触发体内由 DSL 门控保证，机内再校验）。
            let pile = tx.state().skill_pile_cards(player, BIAOZHAO_PILE_DEF);
            if pile.is_empty() {
                return Ok(Vec::new());
            }
            // 弃置首张「表」。
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
            let card = pile[0];
            if let Some(pile_zone) = tx
                .state()
                .skill_piles
                .iter()
                .find(|p| p.owner == player && p.skill_def == BIAOZHAO_PILE_DEF)
                .map(|p| p.zone)
            {
                tx.state_mut()
                    .zones
                    .get_mut(pile_zone)
                    .unwrap()
                    .cards
                    .retain(|c| *c != card);
            }
            tx.state_mut()
                .zones
                .get_mut(discard)
                .unwrap()
                .cards
                .push(card);
            if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
                card_state.zone = discard;
            }
            // 全场最大手牌数（机内计算）。
            let mut num = 0usize;
            for participant in tx.state().players.alive_by_seat() {
                num = num.max(tx.state().hand_cards(participant).len());
            }
            let targets: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(PlayerId::raw)
                .collect();
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                targets,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveBiaozhao3Stage,
                vec![
                    RESOLVE_BIAOZHAO3_MAGIC,
                    player.raw(),
                    skill.raw(),
                    2,
                    num as u32,
                ],
            )
        }
        _ => Ok(Vec::new()),
    }
}

/// 续体：selections[0] = 受益角色。摸至 num（至多补 5 张）并回复 1 点体力。
pub(crate) fn resolve_biaozhao3_stage(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    num: u32,
    selected: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    if selected.is_empty() {
        return Ok(Vec::new());
    }
    let target = PlayerId::new(selected[0]);
    let mut events = Vec::new();
    if num > tx.state().hand_cards(target).len() as u32 {
        let draw = (num - tx.state().hand_cards(target).len() as u32).min(5);
        if draw > 0 {
            events.extend(crate::effect::draw_cards(tx, target, draw)?);
        }
    }
    events.extend(super::super::recover_hp(tx, player, target, 1)?);
    Ok(events)
}

/// 专属状态机 (批 258, ADR 0641 §8.47)：穿云——「传术」衍生技。杀指定目标后
/// 若其装备区有牌：随机弃置其中一张。
pub(crate) fn resolve_chuanyun(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    event_player: PlayerId,
    event_card: u32,
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
    // 事件牌须为【杀】（普/火/雷）。
    let is_sha = tx
        .state()
        .cards
        .get(CardId::new(event_card))
        .is_some_and(|card| {
            matches!(
                card.kind,
                crate::state::CardKind::Sha
                    | crate::state::CardKind::ShaFire
                    | crate::state::CardKind::ShaThunder
            )
        });
    if !is_sha || event_player == PlayerId::NONE || event_player == player {
        return Ok(Vec::new());
    }
    // 目标装备区有牌：RNG 随机弃置一张。
    let Some(equip_zone) = tx.state().zones.find(ZoneKind::Equip, event_player) else {
        return Ok(Vec::new());
    };
    let equips = tx.state().zones.get(equip_zone).unwrap().cards.clone();
    if equips.is_empty() {
        return Ok(Vec::new());
    }
    let mut rng = crate::rng::RuleRng::from_state(tx.state().rng);
    let idx = rng.gen_range_u32(equips.len() as u32) as usize;
    tx.state_mut().rng = rng.into_state();
    let card = equips[idx];
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    tx.state_mut()
        .zones
        .get_mut(equip_zone)
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

/// 专属状态机 (批 259, ADR 0641 §8.48)：表召·应——任意角色弃置与「表」
/// 同花色同点数的牌时（cards_moved 过滤器已判定），将「表」交给弃牌者，
/// owner 失去 1 点体力。无弃牌者（来源不可考）时「表」入弃牌堆。
pub(crate) fn resolve_biaozhao2(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    event_player: PlayerId,
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
    let pile = tx.state().skill_pile_cards(player, BIAOZHAO_PILE_DEF);
    if pile.is_empty() {
        return Ok(Vec::new());
    }
    let card = pile[0];
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    // 从「表」堆移除。
    if let Some(pile_zone) = tx
        .state()
        .skill_piles
        .iter()
        .find(|p| p.owner == player && p.skill_def == BIAOZHAO_PILE_DEF)
        .map(|p| p.zone)
    {
        tx.state_mut()
            .zones
            .get_mut(pile_zone)
            .unwrap()
            .cards
            .retain(|c| *c != card);
    }
    // 有存活弃牌者 → 「表」交给弃牌者；否则「表」入弃牌堆。
    let giver_target = if event_player != PlayerId::NONE
        && tx
            .state()
            .players
            .get(event_player)
            .is_some_and(|participant| participant.alive)
    {
        event_player
    } else {
        player
    };
    if giver_target == player {
        // 无可交付对象 → 「表」入弃牌堆。
        tx.state_mut()
            .zones
            .get_mut(discard)
            .unwrap()
            .cards
            .push(card);
        if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
            card_state.zone = discard;
        }
    } else {
        let hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, giver_target)
            .ok_or_else(|| EffectError::Illegal("no discarder hand".into()))?;
        tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
        if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
            card_state.zone = hand;
            card_state.owner = giver_target;
        }
    }
    // owner 失去 1 点体力。
    let lose = super::super::deal_damage(
        tx,
        PlayerId::NONE,
        player,
        1,
        true,
        0,
        false,
        CardId::NONE,
        0,
        0,
    )?;
    Ok(lose)
}

/// 专属状态机 (批 260, ADR 0641 §8.49)：逆击——成为非装备牌目标后摸 1 张牌
/// 并打上「逆击」标记；结束阶段弃置手牌中全部「逆击」牌。
pub(crate) fn resolve_dcniji(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    mode: u32,
    event_card: u32,
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
    match mode {
        0 => {
            // 非装备牌目标：事件牌须存在且非装备。
            let Some(targeting) = tx.state().cards.get(CardId::new(event_card)) else {
                return Ok(Vec::new());
            };
            if targeting.kind.equip_slot().is_some() {
                return Ok(Vec::new());
            }
            let mut events = crate::effect::draw_cards(tx, player, 1)?;
            if let Some(engine_api::DomainEvent::CardsDrawn { cards, .. }) = events.last() {
                for card in cards {
                    tx.state_mut().add_content_card_tag(*card, DCNIJI_TAG);
                }
            }
            Ok(events)
        }
        1 => {
            // 结束阶段：弃置手牌中全部「逆击」牌。
            let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
            let tagged: Vec<CardId> = tx
                .state()
                .zones
                .get(hand)
                .unwrap()
                .cards
                .iter()
                .copied()
                .filter(|card| tx.state().card_has_content_tag(*card, DCNIJI_TAG))
                .collect();
            for card in tagged {
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
            }
            Ok(Vec::new())
        }
        _ => Ok(Vec::new()),
    }
}

/// 专属状态机 (批 261, ADR 0641 §8.50)：谋溃——杀指定目标后，chooseButton
/// [1,2]：摸 1 张牌／弃置目标一张手牌（owner 从目标手牌选），可同选两项。
/// 未命中追罚面留 slice 2。
pub(crate) fn resolve_dcmoukui(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
    target: PlayerId,
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
            // 新一次杀指定：清除上一杀可能残留的追罚标记。
            tx.state_mut()
                .skills
                .storage_set(skill, DCMOUKUI_FLAG_KEY, 0);
            // Custom 双选：1=摸一张牌 2=弃目标一张牌；可选 1-2 项，不可取消。
            open_choice(
                tx,
                player,
                1,
                2,
                false,
                vec![1, 2],
                ChoiceKind::Custom,
                ContinuationKind::ResolveDcmoukuiStage,
                vec![
                    RESOLVE_DCMOUKUI_MAGIC,
                    player.raw(),
                    skill.raw(),
                    2,
                    target.raw(),
                ],
            )
        }
        5 => {
            // 追罚入口：杀未命中且双选标记在 → 闪避者选 owner 一张手牌弃置。
            resolve_dcmoukui_conseq(tx, player, skill, target)
        }
        _ => Ok(Vec::new()),
    }
}
