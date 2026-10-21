//! Effect primitives for `resolve_potzhouyu_choice` and related helpers.

use super::*;

pub(crate) fn resolve_potzhouyu_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 || payload[0] != POTZHOUYU_CUSTOM_CHOICE {
        return Err(EffectError::EngineFault("bad 势周瑜 payload".into()));
    }
    let player = PlayerId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    if !originating_skill_is_enabled(tx, player, skill) {
        return Err(EffectError::Illegal("势周瑜技能已失效".into()));
    }
    match payload[1] {
        0 => {
            if observer != player {
                return Err(EffectError::Illegal("炽沄选牌者变化".into()));
            }
            if cancelled {
                return Ok(());
            }
            if selections.is_empty() {
                return Err(EffectError::Illegal("炽沄至少交一张牌".into()));
            }
            let hand: std::collections::BTreeSet<_> =
                tx.state().hand_cards(player).into_iter().collect();
            let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
            if cards
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != cards.len()
                || cards.iter().any(|card| !hand.contains(card))
            {
                return Err(EffectError::Illegal("炽沄手牌已变化".into()));
            }
            let color_mask = cards.iter().fold(0, |mask, card| {
                mask | crate::query::effective_card_color_mask(tx.state(), player, *card)
            });
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| *target != player)
                .collect();
            let mut next = vec![
                POTZHOUYU_CUSTOM_CHOICE,
                1,
                player.raw(),
                skill.raw(),
                u32::from(color_mask),
                cards.len() as u32,
            ];
            next.extend(cards.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                targets.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                next,
            )
            .map(|_| ())
        }
        1 => {
            if observer != player || cancelled || selections.len() != 1 || payload.len() < 6 {
                return Err(EffectError::Illegal("炽沄目标选择无效".into()));
            }
            let count = payload[5] as usize;
            if payload.len() != 6 + count {
                return Err(EffectError::EngineFault("bad 炽沄 cards payload".into()));
            }
            let target = PlayerId::new(selections[0]);
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|role| role.alive)
            {
                return Err(EffectError::Illegal("炽沄目标已变化".into()));
            }
            let cards: Vec<_> = payload[6..].iter().copied().map(CardId::new).collect();
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            if cards.iter().any(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_none_or(|value| Some(value.zone) != hand || value.owner != player)
            }) {
                return Err(EffectError::Illegal("炽沄手牌已变化".into()));
            }
            let target_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::EngineFault("target hand missing".into()))?;
            run_effects(
                tx,
                vec![
                    EffectRequest::MoveCards {
                        cards,
                        to: target_hand,
                        from: hand,
                    },
                    EffectRequest::ResolvePotzhouyu {
                        player,
                        target,
                        card: CardId::new(payload[4]),
                        action: 8,
                        card_flags: 0,
                        targets: Vec::new(),
                        skill,
                    },
                ],
            )
            .map(|_| ())
        }
        2 => {
            if payload.len() != 6 || observer.raw() != payload[4] || cancelled {
                return Err(EffectError::Illegal("炽沄分支选择无效".into()));
            }
            let [choice] = selections else {
                return Err(EffectError::Illegal("炽沄分支缺失".into()));
            };
            let target = PlayerId::new(payload[4]);
            match *choice {
                1 if potzhouyu_branch_allowed(tx, skill, 1) => {
                    let mask = payload[5] as u8;
                    let cards: Vec<_> = tx
                        .state()
                        .hand_cards(target)
                        .into_iter()
                        .filter(|card| {
                            crate::query::effective_card_color_mask(tx.state(), target, *card)
                                & mask
                                != 0
                        })
                        .collect();
                    if cards.is_empty() {
                        return Err(EffectError::Illegal("炽沄同色手牌已变化".into()));
                    }
                    run_effects(
                        tx,
                        vec![
                            EffectRequest::RevealCards {
                                cards,
                                to: Vec::new(),
                            },
                            EffectRequest::DealDamage {
                                source: player,
                                target,
                                amount: 1,
                                is_lose_hp: false,
                                nature: rule_api::damage_nature::FIRE,
                            },
                        ],
                    )
                    .map(|_| ())
                }
                2 if potzhouyu_branch_allowed(tx, skill, 2) => {
                    let mut effects = vec![EffectRequest::DrawCards { player, count: 2 }];
                    let linked = tx.state().players.get(target).is_some_and(|role| {
                        role.marks
                            .iter()
                            .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
                    });
                    if !linked {
                        effects.push(EffectRequest::AddMark {
                            player: target,
                            mark: engine_api::MarkDefId::new(crate::content::MARK_CHAIN),
                            delta: 1,
                        });
                    }
                    run_effects(tx, effects).map(|_| ())
                }
                _ => Err(EffectError::Illegal("炽沄分支已禁用".into())),
            }
        }
        3 => {
            if observer != player || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("焰洄目标选择无效".into()));
            }
            let target = PlayerId::new(selections[0]);
            let cards = tx.state().hand_cards(target);
            if cards.is_empty() {
                return Err(EffectError::Illegal("焰洄目标已无手牌".into()));
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                cards.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    POTZHOUYU_CUSTOM_CHOICE,
                    4,
                    player.raw(),
                    skill.raw(),
                    target.raw(),
                ],
            )
            .map(|_| ())
        }
        4 => {
            if payload.len() != 5 || observer != player || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("焰洄展示选择无效".into()));
            }
            let target = PlayerId::new(payload[4]);
            let card = CardId::new(selections[0]);
            if !tx.state().hand_cards(target).contains(&card) {
                return Err(EffectError::Illegal("焰洄手牌已变化".into()));
            }
            let mut state = potyanhui_state(tx, skill);
            let seen = state.revealed.iter().any(|(shown, _)| *shown == card);
            if !state.revealed.iter().any(|entry| *entry == (card, target)) {
                state.revealed.push((card, target));
            }
            state.armed = true;
            if seen && !state.discarded.contains(&target) {
                state.discarded.push(target);
            }
            set_potyanhui_state(tx, skill, &state);
            let mut effects = vec![EffectRequest::RevealCards {
                cards: vec![card],
                to: Vec::new(),
            }];
            if seen {
                effects.push(EffectRequest::DiscardCards {
                    player: target,
                    cards: vec![card],
                });
            }
            run_effects(tx, effects).map(|_| ())
        }
        5 => {
            if payload.len() < 6 || observer != player || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("焰洄阶段结算选择无效".into()));
            }
            let target_count = payload[5] as usize;
            if payload.len() != 6 + target_count {
                return Err(EffectError::EngineFault("bad 焰洄阶段 payload".into()));
            }
            let choice = selections[0];
            if choice == 0 {
                if !potzhouyu_branch_allowed(tx, skill, 2) || payload[4] == 0 {
                    return Err(EffectError::Illegal("焰洄摸牌分支无效".into()));
                }
                run_effects(
                    tx,
                    vec![EffectRequest::DrawCards {
                        player,
                        count: payload[4],
                    }],
                )
                .map(|_| ())
            } else {
                let target = PlayerId::new(choice);
                if !potzhouyu_branch_allowed(tx, skill, 1)
                    || !payload[6..].contains(&choice)
                    || !tx
                        .state()
                        .players
                        .get(target)
                        .is_some_and(|role| role.alive)
                {
                    return Err(EffectError::Illegal("焰洄伤害目标无效".into()));
                }
                run_effects(
                    tx,
                    vec![EffectRequest::DealDamage {
                        source: player,
                        target,
                        amount: 1,
                        is_lose_hp: false,
                        nature: rule_api::damage_nature::FIRE,
                    }],
                )
                .map(|_| ())
            }
        }
        6 => {
            if payload.len() != 7 || observer.raw() != payload[4] {
                return Err(EffectError::Illegal("焚涛弃牌选择无效".into()));
            }
            let target = PlayerId::new(payload[4]);
            let propagated = payload[5] != 0;
            if cancelled {
                return add_potfentao_damage(tx, skill, propagated).map(|_| ());
            }
            let required = payload[6] as usize;
            if selections.len() != required {
                return Err(EffectError::Illegal("焚涛弃牌数量无效".into()));
            }
            let legal: std::collections::BTreeSet<_> =
                crate::query::discardable_hand_equip_cards(tx.state(), target)
                    .into_iter()
                    .collect();
            let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
            if cards
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != cards.len()
                || cards.iter().any(|card| !legal.contains(card))
            {
                return Err(EffectError::Illegal("焚涛弃牌已变化".into()));
            }
            set_potfentao_pending(tx, skill, target);
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: target,
                    cards,
                }],
            )
            .map(|_| ())
        }
        8 => {
            if observer != player {
                return Err(EffectError::Illegal("雄姿选择者变化".into()));
            }
            if cancelled {
                return Ok(());
            }
            let [choice] = selections else {
                return Err(EffectError::Illegal("雄姿分支缺失".into()));
            };
            if !matches!(*choice, 1 | 2) {
                return Err(EffectError::Illegal("雄姿分支无效".into()));
            }
            let disable = if *choice == 1 {
                POTZHOUYU_DISABLE_TWO
            } else {
                POTZHOUYU_DISABLE_ONE
            };
            for def in [
                POTZHOUYU_CHIYUN_DEF,
                POTZHOUYU_YANHUI_DEF,
                POTZHOUYU_FENTAO_DEF,
            ] {
                if let Some(id) =
                    skill_instance_by_def(tx, player, engine_api::SkillDefId::new(def))
                {
                    if let Some(instance) = tx.state_mut().skills.get_mut(id) {
                        instance.content_flags |= disable;
                    }
                }
            }
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.lifetime_fire_count = 1;
            }
            run_effects(tx, vec![EffectRequest::DrawCards { player, count: 2 }]).map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown 势周瑜 choice stage".into(),
        )),
    }
}

/// 置入 (expansion P2, ADR 0641, ABI 650): the owner selects one hand card
/// (any kind) and places it onto their skill pile, reusing the
/// CommitStoreHandShaInSkillPile pile-insertion machinery.
pub(crate) fn place_hand_card_to_expansion(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _skill_def = skill_pile_def(tx, player, skill)?;
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let cards: Vec<u32> = tx
        .state()
        .hand_cards(player)
        .iter()
        .map(|c| c.raw())
        .collect();
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        cards,
        ChoiceKind::SelectCards,
        ContinuationKind::PlaceHandCardToExpansion,
        vec![player.raw(), skill.raw()],
    )
}

/// 选他人授标记 (ABI 652): the chooser picks one other alive player and the
/// chosen player's mark is adjusted by `delta`.
pub(crate) fn choose_other_add_mark(
    tx: &mut Transaction,
    player: PlayerId,
    mark: u32,
    delta: i32,
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
    // Negative deltas are removal flows (jsrgpingrong removing 猎): only
    // targets that actually hold the mark are offered, mirroring upstream
    // hasMark-gated chooseTarget filters. Positive grants keep offering every
    // other living player.
    let clamp_negative = delta < 0;
    let current_count = |state: &crate::state::GameState, target: PlayerId| -> i32 {
        state
            .players
            .get(target)
            .and_then(|p| p.marks.iter().find(|(m, _)| *m == mark))
            .map_or(0, |(_, c)| *c)
    };
    let mut options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != player)
        .filter(|&target| !clamp_negative || current_count(tx.state(), target) > 0)
        .map(PlayerId::raw)
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        options,
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseOtherAddMark,
        vec![player.raw(), mark, delta as u32, skill.raw()],
    )
}

/// 取用 (expansion P1, ADR 0641, ABI 651): the owner selects one card from
/// their own skill pile (武将牌上) and gains it into hand.
pub(crate) fn place_top_deck_to_expansion(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    pile_def: u32,
    count: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    // Provenance gate on the originating skill instance; the pile is addressed by
    // the `pile_def` literal so sibling piles can be supplied.
    let _ = skill_pile_def(tx, player, skill)?;
    if count == 0
        || !tx
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
    let top: Vec<CardId> = {
        let cards = &tx.state().zones.get(draw).unwrap().cards;
        cards.iter().take(count as usize).copied().collect()
    };
    if top.is_empty() {
        return Ok(Vec::new());
    }
    let pile = ensure_skill_pile_zone(tx, player, pile_def)?;
    move_cards_with_reason(tx, top, pile, Some(draw), CardMoveReason::Unspecified)
}

pub(crate) fn choose_one_from_skill_pile_discard(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    pile_def: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill_pile_def(tx, player, skill)?;
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let cards = tx.state().skill_pile_cards(player, pile_def);
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        cards.iter().map(|card| card.raw()).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseOneFromSkillPileDiscard,
        vec![player.raw(), pile_def, skill.raw()],
    )
}

pub(crate) fn place_hand_cards_to_deck_top(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    count: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill_pile_def(tx, player, skill)?;
    if count == 0
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let hand_cards: Vec<u32> = {
        let hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
        tx.state()
            .zones
            .get(hand)
            .unwrap()
            .cards
            .iter()
            .map(|c| c.raw())
            .collect()
    };
    if hand_cards.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        count as u8,
        count as u8,
        true,
        hand_cards,
        ChoiceKind::SelectCards,
        ContinuationKind::PlaceHandCardsToDeckTop,
        vec![player.raw(), skill.raw(), count],
    )
}

pub(crate) fn gain_whole_skill_pile(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    pile_def: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill_pile_def(tx, player, skill)?;
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let cards = tx.state().skill_pile_cards(player, pile_def);
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let pile = tx
        .state()
        .skill_piles
        .iter()
        .find(|p| p.owner == player && p.skill_def == pile_def)
        .map(|p| p.zone)
        .ok_or_else(|| EffectError::Illegal("skill pile missing".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    move_cards_with_reason(tx, cards, hand, Some(pile), CardMoveReason::Unspecified)
}

pub(crate) fn choose_target_hand_card_to_pile(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    pile_def: u32,
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
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != player && !tx.state().hand_cards(target).is_empty())
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
        ContinuationKind::ChooseTargetHandCardToPile,
        vec![0, player.raw(), skill.raw(), pile_def],
    )
}

/// 专属状态机 (批 239, ADR 0641 §8.34)：度望——游戏开始时，将牌堆顶前 5 张
/// 非【杀】牌置于自身技能堆（「刺」）。距离修正面保守省略（上游为有刺即 +1，
/// DSL 无 min(1,count) 修饰源，+N 超集不建）。
pub(crate) fn resolve_duwang(
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
    let skill_def = crate::effect::skill_pile_def(tx, player, skill)?;
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    // 牌堆顶 = 向量尾部；取前 5 张非【杀】。
    let top_cards = tx.state().zones.get(draw).unwrap().cards.clone();
    let mut picked = Vec::new();
    for card in top_cards.iter().rev() {
        if picked.len() >= 5 {
            break;
        }
        if let Some(state) = tx.state().cards.get(*card) {
            if state.kind != crate::state::CardKind::Sha {
                picked.push(*card);
            }
        }
    }
    if picked.is_empty() {
        return Ok(Vec::new());
    }
    let pile = crate::effect::ensure_skill_pile_zone(tx, player, skill_def)?;
    crate::effect::move_cards(tx, picked, pile, Some(draw))?;
    Ok(Vec::new())
}

/// 专属状态机 (批 242, ADR 0641 §8.37)：坐瞻——游戏开始时，选择至多两名其他
/// 角色，获得 min(所选角色最高体力, 5) 枚「射程」标记（attack_range
/// owner_skill_mark 消费），所选角色各记一枚「坐瞻」标记。
pub(crate) fn resolve_jsrgzuozhan(
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
        2,
        true,
        targets,
        ChoiceKind::SelectTargets,
        ContinuationKind::ResolveJsrgzuozhanStage,
        vec![RESOLVE_ZUOZHAN_CUSTOM_CHOICE, player.raw(), skill.raw()],
    )
}

/// 续体：selected = 「坐瞻」角色（1..=2）；射程标记 = min(所选最高体力, 5)
/// 加于自身（标记 id = 技能 def，owner_skill_mark 修饰源按 def 读取；
/// self_skill 传入为实例 id，须经 skill_pile_def 解析）。
pub(crate) fn resolve_jsrgzuozhan_stage(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    selected: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    let skill_def = skill_pile_def(tx, player, skill)?;
    let mut best_hp = 0i32;
    for raw in selected {
        let hp = tx
            .state()
            .players
            .get(PlayerId::new(*raw))
            .filter(|participant| participant.alive)
            .map(|participant| participant.hp)
            .unwrap_or(0);
        best_hp = best_hp.max(hp);
    }
    let range = best_hp.clamp(0, 5) as u32;
    let mut events = Vec::new();
    if range > 0 {
        events.extend(crate::effect::add_mark(
            tx,
            player,
            skill_def,
            range as i32,
        )?);
    }
    for raw in selected {
        events.extend(crate::effect::add_mark(
            tx,
            PlayerId::new(*raw),
            ZUOZHAN_SIT_MARK,
            1,
        )?);
    }
    Ok(events)
}

/// 基本牌名分组（上游「牌名各不相同」口径：杀/火杀/雷杀同名 sha）。
pub(crate) fn zuozhan_basic_name_group(kind: crate::state::CardKind) -> Option<u8> {
    match kind {
        crate::state::CardKind::Sha
        | crate::state::CardKind::ShaFire
        | crate::state::CardKind::ShaThunder => Some(1),
        crate::state::CardKind::Tao => Some(2),
        crate::state::CardKind::Shan => Some(3),
        crate::state::CardKind::Jiu => Some(4),
        _ => None,
    }
}

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
