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
