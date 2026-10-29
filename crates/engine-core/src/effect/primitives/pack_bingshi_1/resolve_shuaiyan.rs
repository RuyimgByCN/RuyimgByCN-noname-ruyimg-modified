//! Effect primitives for `resolve_dcmoukui_stage` and related helpers.

use super::*;

/// 专属状态机 (批 266, ADR 0641 §8.54)：率言——弃牌阶段开始，若手牌数大于
/// 1：选择一名其他角色，其交给你一张牌（手牌或装备区）。
pub(crate) fn resolve_shuaiyan(
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
                || tx.state().hand_cards(player).len() <= 1
            {
                return Ok(Vec::new());
            }
            let candidates: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&candidate| {
                    candidate != player
                        && (tx.state().zones.find(ZoneKind::Hand, candidate).is_some()
                            && !tx.state().hand_cards(candidate).is_empty()
                            || tx
                                .state()
                                .zones
                                .find(ZoneKind::Equip, candidate)
                                .is_some_and(|zone| {
                                    !tx.state().zones.get(zone).unwrap().cards.is_empty()
                                }))
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
                ContinuationKind::ResolveShuaiyanStage,
                vec![RESOLVE_SHUAIYAN_MAGIC, player.raw(), skill.raw(), 0],
            )
        }
        2 => {
            // 目标已选：其从手牌/装备区选一张交给 owner。
            let target = tx
                .state()
                .resolution
                .continuation_payload
                .get(4)
                .map(|raw| PlayerId::new(*raw))
                .unwrap_or(PlayerId::NONE);
            let mut options: Vec<u32> = tx
                .state()
                .hand_cards(target)
                .iter()
                .map(|card| card.raw())
                .collect();
            if let Some(equip) = tx.state().zones.find(ZoneKind::Equip, target) {
                options.extend(
                    tx.state()
                        .zones
                        .get(equip)
                        .unwrap()
                        .cards
                        .iter()
                        .map(|card| card.raw()),
                );
            }
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let payload = vec![
                RESOLVE_SHUAIYAN_MAGIC,
                player.raw(),
                skill.raw(),
                3,
                target.raw(),
            ];
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveShuaiyanStage,
                payload,
            )?;
            Ok(Vec::new())
        }
        _ => Ok(Vec::new()),
    }
}

/// stage 3：目标选定的牌交给 owner。
pub(crate) fn resolve_shuaiyan_give(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    target_raw: u32,
    card_raw: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    let target = PlayerId::new(target_raw);
    let card = CardId::new(card_raw);
    // 从目标的手牌或装备区移除。
    let from = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .filter(|zone| tx.state().zones.get(*zone).unwrap().cards.contains(&card))
        .or_else(|| tx.state().zones.find(ZoneKind::Equip, target))
        .ok_or_else(|| EffectError::Illegal("shuaiyan card not found".into()))?;
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
    Ok(vec![engine_api::DomainEvent::CardsDrawn {
        player,
        cards: vec![card],
    }])
}

/// stage 2：目标已选 → 目标从手牌/装备区选一张交给 owner。
pub(crate) fn resolve_shuaiyan_stage(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    target_raw: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let target = PlayerId::new(target_raw);
    let mut options: Vec<u32> = tx
        .state()
        .hand_cards(target)
        .iter()
        .map(|card| card.raw())
        .collect();
    if let Some(equip) = tx.state().zones.find(ZoneKind::Equip, target) {
        options.extend(
            tx.state()
                .zones
                .get(equip)
                .unwrap()
                .cards
                .iter()
                .map(|card| card.raw()),
        );
    }
    if options.is_empty() {
        return Ok(Vec::new());
    }
    let payload = vec![
        RESOLVE_SHUAIYAN_MAGIC,
        player.raw(),
        skill.raw(),
        3,
        target_raw,
    ];
    open_choice(
        tx,
        target,
        1,
        1,
        false,
        options,
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveShuaiyanStage,
        payload,
    )?;
    Ok(Vec::new())
}

/// 专属状态机 (批 275, ADR 0641 §8.63)：鄙位——出牌阶段限一次，弃手牌中
/// 点数最大的一张，令一名有手牌的其他角色弃置所有点数≥该牌的手牌；若其
/// 未因此弃牌，重置技能的出牌阶段使用计数（上游 delete getStat("skill")）。
pub(crate) fn resolve_friendbiwei(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if target == player {
        return Err(EffectError::Illegal(
            "resolve-friendbiwei target must be another player".into(),
        ));
    }
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
        || tx.state().hand_count(target) == 0
    {
        return Ok(Vec::new());
    }
    // 成本牌必须是 owner 手牌中点数最大的一张。
    let hand = tx.state().hand_cards(player);
    if !hand.contains(&card) {
        return Err(EffectError::Illegal(
            "resolve-friendbiwei card is not in the owner hand".into(),
        ));
    }
    let cost_rank = tx
        .state()
        .cards
        .get(card)
        .map(|card| card.rank)
        .ok_or_else(|| EffectError::Illegal("resolve-friendbiwei card is missing".into()))?;
    let max_other = hand
        .iter()
        .filter(|other| **other != card)
        .filter_map(|other| tx.state().cards.get(*other).map(|card| card.rank))
        .max()
        .unwrap_or(0);
    if cost_rank < max_other {
        return Err(EffectError::Illegal(
            "resolve-friendbiwei card is not the highest rank".into(),
        ));
    }
    // 弃成本牌。
    let mut events = run_effects(
        tx,
        vec![EffectRequest::DiscardCards {
            player,
            cards: vec![card],
        }],
    )?;
    // 目标弃置所有点数 ≥ 成本点数的手牌。
    let to_discard: Vec<CardId> = tx
        .state()
        .hand_cards(target)
        .into_iter()
        .filter(|other| {
            tx.state()
                .cards
                .get(*other)
                .is_some_and(|card| card.rank >= cost_rank)
        })
        .collect();
    if to_discard.is_empty() {
        // 未因此弃牌 → 重置技能使用计数（可再次发动）。
        if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
            instance.use_count = 0;
            instance.play_phase_use_count = 0;
        }
        return Ok(events);
    }
    events.extend(run_effects(
        tx,
        vec![EffectRequest::DiscardCards {
            player: target,
            cards: to_discard,
        }],
    )?);
    Ok(events)
}

/// 专属状态机 (批 274, ADR 0641 §8.62)：黜老——出牌阶段限一次，弃一张牌
/// （黑桃则 owner 摸 1），对任意名势力各异角色逐个由 owner 选弃其一张牌
/// （被弃牌黑桃则该目标摸 1）。stage 0=弃成本牌 → 1=成本结算+对首目标开
/// 选牌 → 2=弃目标牌+黑桃摸 1+循环下一目标。
pub(crate) fn resolve_chulao(
    tx: &mut Transaction,
    player: PlayerId,
    targets: &[PlayerId],
) -> Result<Vec<DomainEvent>, EffectError> {
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    // 势力各异且非 unknown。
    let mut seen: Vec<String> = Vec::new();
    for target in targets {
        let group = tx.state().player_group(*target);
        match group {
            None => return Ok(Vec::new()),
            Some(group) => {
                if seen.iter().any(|existing| existing == group) {
                    return Err(EffectError::Illegal(
                        "resolve-chulao targets must have distinct groups".into(),
                    ));
                }
                seen.push(group.to_string());
            }
        }
        if !tx
            .state()
            .players
            .get(*target)
            .is_some_and(|participant| participant.alive)
        {
            return Err(EffectError::Illegal(
                "resolve-chulao targets must be alive".into(),
            ));
        }
    }
    let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
    if options.is_empty() {
        return Ok(Vec::new());
    }
    let mut payload = vec![RESOLVE_CHULAO_MAGIC, 1u32, player.raw()];
    payload.extend(targets.iter().map(|target| target.raw()));
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveChulaoStage,
        payload,
    )
}

/// 专属状态机 (批 281, ADR 0641 §8.69)：定策——受到伤害后，弃自己一张手牌
/// （可取消），再令伤害来源弃一张手牌（owner 代选，强制）。
/// stage 0=owner 选自己的手牌 → 1=弃置+对来源开选牌 → 2=弃置来源牌。
/// 同色分支（虚拟【洞烛先机】）因卡牌不在牌池登记债，暂不结算。
pub(crate) fn resolve_dingce(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(source)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let options: Vec<u32> = crate::query::discardable_hand_equip_cards(tx.state(), player)
        .into_iter()
        .map(CardId::raw)
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
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveDingceStage,
        vec![RESOLVE_DINGCE_MAGIC, 1, player.raw(), source.raw()],
    )
}

/// 专属状态机 (批 286, ADR 0641 §8.71)：巧略弃牌代价补全——用牌指定目标后，
/// 先弃自己一张牌（强制），再取事件目标一张随机手牌。
/// stage 0=owner 选成本牌（1..1 强制）→ 1=弃成本 + TakeOneHandCard。
pub(crate) fn resolve_dcqiaolve(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
        || tx.state().hand_count(target) == 0
    {
        return Ok(Vec::new());
    }
    let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveDcqiaolveStage,
        vec![RESOLVE_DCQIAOLVE_MAGIC, 1, player.raw(), target.raw()],
    )
}

pub(crate) fn resolve_twxiongjun(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let damage_token = tx.state().resolution.current_damage_id;
    if damage_token != 0 {
        if let Some(instance) = tx.state().players.get(player).and_then(|p| {
            p.skills.iter().find_map(|skill| {
                tx.state().skills.get(*skill).and_then(|instance| {
                    (instance.def.raw() == 2_909_562_544).then_some(instance.id)
                })
            })
        }) {
            if tx
                .state()
                .skills
                .storage_get(instance, RESOLVE_TW_XIONGJUN_LAST_DAMAGE)
                == damage_token
            {
                return Ok(Vec::new());
            }
            tx.state_mut().skills.storage_set(
                instance,
                RESOLVE_TW_XIONGJUN_LAST_DAMAGE,
                damage_token,
            );
        }
    }
    let mut events = Vec::new();
    for holder in tx.state().players.alive_by_seat() {
        let owns = tx.state().players.get(holder).is_some_and(|p| {
            p.skills.iter().any(|skill| {
                tx.state()
                    .skills
                    .get(*skill)
                    .is_some_and(|instance| instance.def.raw() == 2_909_562_544)
            })
        });
        if !owns {
            continue;
        }
        events.extend(crate::effect::draw_cards(tx, holder, 1)?);
    }
    let _ = player;
    Ok(events)
}

/// 专属状态机 (批 288, ADR 0641 §8.73)：义襄——每名角色回合限一次，成为
/// 使用牌目标时随机获得牌堆中一张自己没有的基本牌。单发无续体。
pub(crate) fn resolve_yixiang(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("draw pile missing".into()))?;
    let hand_kinds: Vec<crate::state::CardKind> = tx
        .state()
        .hand_cards(player)
        .iter()
        .filter_map(|card| tx.state().cards.get(*card))
        .map(|card| card.kind)
        .collect();
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
    let candidate = tx
        .state()
        .zones
        .get(draw)
        .unwrap()
        .cards
        .iter()
        .rev()
        .copied()
        .find(|card| {
            tx.state().cards.get(*card).is_some_and(|c| {
                c.kind.is_basic()
                    && !hand_kinds.contains(&c.kind)
                    && !tx.state().hand_cards(player).contains(card)
            })
        });
    let Some(card) = candidate else {
        return Ok(Vec::new());
    };
    move_cards(tx, vec![card], hand, Some(draw))
}

/// 求援 giver 扫描：座位序从 `after` 之后环绕，跳过 owner、已死与无
/// 手+装备牌者（上游 `game.filterPlayer(current != player).sortBySeat()`）。
pub(crate) fn xinfu_qiai_next_giver(
    state: &crate::state::GameState,
    owner: PlayerId,
    after: PlayerId,
) -> Option<PlayerId> {
    let alive = state.players.alive_by_seat();
    let start = alive
        .iter()
        .position(|id| *id == after)
        .map(|index| index + 1)
        .unwrap_or(0);
    for offset in 0..alive.len() {
        let candidate = alive[(start + offset) % alive.len()];
        if candidate == owner {
            return None;
        }
        if !crate::query::discardable_hand_equip_cards(state, candidate).is_empty() {
            return Some(candidate);
        }
    }
    None
}

/// 专属状态机 (批 291, ADR 0641 §8.74)：求援——限定技，濒死时其他角色依
/// 座次各交给 owner 一张牌（手+装备，强制，跳过无牌者）。stage 1=对 giver
/// 开选牌（强制 1..1）→ 续体=移交 owner 手牌区+循环下一 giver。
pub(crate) fn resolve_xinfu_qiai(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(giver) = xinfu_qiai_next_giver(tx.state(), player, player) else {
        return Ok(Vec::new());
    };
    let options: Vec<u32> = crate::query::discardable_hand_equip_cards(tx.state(), giver)
        .into_iter()
        .map(CardId::raw)
        .collect();
    open_choice(
        tx,
        giver,
        1,
        1,
        false,
        options,
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveXinfuQiaiStage,
        vec![RESOLVE_XINFU_QIAI_MAGIC, 1, player.raw(), giver.raw()],
    )
}

/// 专属状态机 (批 292, ADR 0641 §8.76)：温仁——出牌阶段限一次，任选多名
/// 目标，每名目标无手牌摸 1、手牌数不大于 owner 再摸 1（可叠 2，可摸 0）。
/// 单发无续体；目标按座位序去重结算（上游逐目标 await draw 同序）。
pub(crate) fn resolve_olwenren(
    tx: &mut Transaction,
    player: PlayerId,
    targets: &[PlayerId],
) -> Result<Vec<DomainEvent>, EffectError> {
    let order = tx.state().players.alive_by_seat();
    let mut sorted: Vec<PlayerId> = Vec::with_capacity(targets.len());
    for &target in targets {
        if sorted.contains(&target) {
            continue;
        }
        if !order.contains(&target) {
            continue;
        }
        sorted.push(target);
    }
    sorted.sort_by_key(|target| {
        order
            .iter()
            .position(|candidate| candidate == target)
            .expect("validated living target")
    });
    let owner_hand = tx.state().hand_count(player);
    let mut events = Vec::new();
    for target in sorted {
        let mut num: u32 = 0;
        if tx.state().hand_count(target) == 0 {
            num += 1;
        }
        if tx.state().hand_count(target) <= owner_hand {
            num += 1;
        }
        if num > 0 {
            events.extend(draw_cards(tx, target, num)?);
        }
    }
    Ok(events)
}

/// 专属状态机 (批 293, ADR 0641 §8.77)：霸业 spolaige——觉醒技：全局第二次
/// 濒死结算后，失去〖西向〗获得〖朔北〗，摸牌至体力上限并回复体力至体力
/// 上限（差值在机内计算：上游 drawTo/recoverTo 的等价量表达）。
/// 单发无续体；一次性由 owner_alive_second_global_dying（计数==2）承载。
pub(crate) fn resolve_spolaige(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let mut events = run_effects(
        tx,
        vec![
            EffectRequest::AddSkill {
                player,
                skill_def: engine_api::SkillDefId::new(SPOLZHUBEI_DEF),
            },
            EffectRequest::RemoveSkillByDef {
                player,
                skill_def: engine_api::SkillDefId::new(SPOLXIXIANG_DEF),
            },
        ],
    )?;
    let max_hp = tx
        .state()
        .players
        .get(player)
        .map(|participant| participant.max_hp)
        .unwrap_or(0);
    let draw = max_hp.saturating_sub(tx.state().hand_count(player));
    if draw > 0 {
        events.extend(draw_cards(tx, player, draw)?);
    }
    let hp = tx
        .state()
        .players
        .get(player)
        .map(|participant| participant.hp)
        .unwrap_or(0);
    let recover = i64::from(max_hp) - i64::from(hp);
    if recover > 0 {
        let amount = u32::try_from(recover).unwrap_or(0);
        events.extend(recover_hp(tx, PlayerId::NONE, player, amount)?);
    }
    Ok(events)
}

/// 专属状态机 (批 294, ADR 0641 §8.78)：直谏——任意角色结束阶段，选射程
/// 内一名角色对其造成 1 点伤害（forced）。stage 1=选射程内目标（强制
/// SelectTargets）→ 续体=对目标造成 1 点伤害。
pub(crate) fn resolve_olzhijie(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let candidates: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|candidate| {
            *candidate != player
                && crate::query::player_in_attack_range(tx.state(), player, *candidate)
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
        false,
        candidates,
        ChoiceKind::SelectTargets,
        ContinuationKind::ResolveOlzhijieStage,
        vec![RESOLVE_OLZHIJIE_MAGIC, 1, player.raw()],
    )
}

/// 专属状态机 (批 295, ADR 0641 §8.79)：俗义——出牌阶段限一次，选一名
/// 其他角色：牌堆顶向下检索第一张其可装备的装备牌，令其获得并装备；
/// 然后对其视为使用一张无来源消耗的【杀】（上游 useCard isCard:true）。
/// 单发无续体；上游 get.cardPile 随机检索以顺序检索近似（登记债）。
pub(crate) fn resolve_olsuyi(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("draw pile missing".into()))?;
    let target_slot_enabled = |tx: &Transaction, card: CardId| {
        tx.state()
            .cards
            .get(card)
            .and_then(|card_state| {
                crate::query::effective_card_kind(tx.state(), target, card_state).equip_slot()
            })
            .is_some_and(|slot| {
                tx.state()
                    .players
                    .get(target)
                    .is_some_and(|p| p.equip_slot_enabled(slot))
            })
    };
    let candidate = tx
        .state()
        .zones
        .get(draw)
        .unwrap()
        .cards
        .iter()
        .rev()
        .copied()
        .find(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|c| c.kind.is_equip() && target_slot_enabled(tx, *card))
        });
    let mut events = Vec::new();
    if let Some(card) = candidate {
        // 获得 → 装备（上游 $gain2 + chooseUseTarget forced 的等价量）。
        let hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, target)
            .ok_or_else(|| EffectError::Illegal("target hand zone missing".into()))?;
        events.extend(move_cards(tx, vec![card], hand, Some(draw))?);
        crate::command::play::use_equipment_from_hand(tx, target, card)?;
    }
    // owner 对目标视为使用【杀】（CardKind::Sha tag=1；上游 isCard:true
    // 实体化杀的等价表达，完整结算管道）。
    if tx.state().players.get(target).is_some_and(|p| p.alive) {
        events.extend(run_effects(
            tx,
            vec![EffectRequest::UseVirtualCardOnSelectedTargets {
                player,
                kind: crate::state::CardKind::Sha.tag(),
                targets: vec![target],
            }],
        )?);
    }
    Ok(events)
}
