//! Command operations for `resolve_zhizheng_commit` and related helpers.

use super::*;

/// 踞襄（张绣，主公技）装备牌续体：栏位空闲/禁用时置入主公装备区并恢复栏位，否则获得。
pub(crate) fn resolve_twjuxiang_card(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let lord = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let event_player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != event_player {
        return Ok(());
    }
    if cancelled || selections.len() != 1 {
        return Ok(());
    }
    let card = CardId::new(selections[0]);
    if !tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == lord && tx.state().skill_effectively_enabled(instance)
    }) || !tx.state().players.get(lord).is_some_and(|p| p.alive)
    {
        return Ok(());
    }
    let event_equip = tx
        .state()
        .zones
        .find(ZoneKind::Equip, event_player)
        .ok_or_else(|| EffectError::Illegal("no event equip zone".into()))?;
    let in_equip = tx
        .state()
        .zones
        .get(event_equip)
        .is_some_and(|zone| zone.cards.contains(&card));
    if !in_equip {
        return Err(EffectError::Illegal("踞襄 card moved".into()));
    }
    let kind = tx
        .state()
        .cards
        .get(card)
        .map(|card| card.kind)
        .ok_or_else(|| EffectError::Illegal("踞襄 card missing".into()))?;
    if !kind.is_equip() {
        return Err(EffectError::Illegal(
            "踞襄 requires an equipment card".into(),
        ));
    }
    let lord_equip = tx
        .state()
        .zones
        .find(ZoneKind::Equip, lord)
        .ok_or_else(|| EffectError::Illegal("no lord equip zone".into()))?;
    let slot = kind.equip_slot();
    let slot_occupied = slot.is_some_and(|slot| {
        tx.state()
            .zones
            .get(lord_equip)
            .map(|zone| {
                zone.cards.iter().any(|existing| {
                    tx.state()
                        .cards
                        .get(*existing)
                        .is_some_and(|c| c.kind.equip_slot() == Some(slot))
                })
            })
            .unwrap_or(false)
    });
    let mut requests = Vec::new();
    if slot_occupied {
        let lord_hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, lord)
            .ok_or_else(|| EffectError::Illegal("no lord hand".into()))?;
        requests.push(EffectRequest::MoveCards {
            cards: vec![card],
            to: lord_hand,
            from: Some(event_equip),
        });
    } else {
        requests.push(EffectRequest::MoveCards {
            cards: vec![card],
            to: lord_equip,
            from: Some(event_equip),
        });
    }
    run_effects(tx, requests)?;
    if let Some(slot) = slot {
        if !tx
            .state()
            .players
            .get(lord)
            .is_some_and(|p| p.equip_slot_enabled(slot))
        {
            tx.push_event(engine_api::DomainEvent::RuleNote {
                code: 0x5901,
                payload: vec![lord.raw(), slot as u32],
            });
            tx.state_mut()
                .players
                .get_mut(lord)
                .expect("validated lord")
                .disabled_equip_slots[slot.index()] = 0;
        }
    }
    Ok(())
}

pub(crate) fn resolve_zhugemengxue_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 6 || payload[0] != crate::effect::RESOLVE_ZHUGEMENGXUE_CHOICE {
        return Err(EffectError::EngineFault(
            "bad 诸葛梦雪 choice payload".into(),
        ));
    }
    let stage = payload[1];
    let player = PlayerId::new(payload[2]);
    let skill = SkillId::new(payload[3]);
    if observer != player || cancelled {
        return Err(EffectError::Illegal("无效的诸葛梦雪选择".into()));
    }
    match stage {
        0 | 2 => {
            let card = CardId::new(payload[4]);
            let option_count = payload[5] as usize;
            if payload.len() != 6 + option_count || selections.len() != 1 {
                return Err(EffectError::Illegal("诸葛梦雪目标数量错误".into()));
            }
            let selected = selections[0];
            if !payload[6..].contains(&selected) {
                return Err(EffectError::Illegal("诸葛梦雪目标未在候选中".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveZhugemengxue {
                    player,
                    target: PlayerId::new(selected),
                    card,
                    action: if stage == 0 { 1 } else { 11 },
                    cards: Vec::new(),
                    skill,
                }],
            )
            .map(|_| ())
        }
        1 => {
            if payload.len() < 7 {
                return Err(EffectError::EngineFault(
                    "bad 寄春弃牌 choice payload".into(),
                ));
            }
            let target = PlayerId::new(payload[4]);
            let max = payload[5] as usize;
            let option_count = payload[6] as usize;
            if payload.len() != 7 + option_count || selections.is_empty() || selections.len() > max
            {
                return Err(EffectError::Illegal("寄春弃牌数量错误".into()));
            }
            let offered: std::collections::BTreeSet<_> = payload[7..].iter().copied().collect();
            let current: std::collections::BTreeSet<_> =
                crate::command::play::discardable_target_zone_card_tokens(tx, player, target)
                    .into_iter()
                    .collect();
            let mut unique = std::collections::BTreeSet::new();
            if !selections.iter().all(|card| {
                unique.insert(*card) && offered.contains(card) && current.contains(card)
            }) {
                return Err(EffectError::Illegal("寄春弃牌已失效".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: target,
                    cards: selections.iter().copied().map(CardId::new).collect(),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault("bad 诸葛梦雪 choice stage".into())),
    }
}

pub(crate) fn resolve_bailingyun_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 || payload[0] != crate::effect::RESOLVE_BAILINGYUN_CHOICE {
        return Err(EffectError::EngineFault("bad 柏灵筠 choice payload".into()));
    }
    let stage = payload[1];
    let player = PlayerId::new(payload[2]);
    let skill = SkillId::new(payload[3]);
    if observer != player {
        return Err(EffectError::Illegal("无效的柏灵筠选择者".into()));
    }
    let skill_enabled = |tx: &Transaction| {
        tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    };
    match stage {
        0 => {
            let all_count = payload[4] as usize;
            let all_end = 5usize.saturating_add(all_count);
            if payload.len() <= all_end {
                return Err(EffectError::EngineFault("bad 灵慧选牌 payload".into()));
            }
            let cards: Vec<_> = payload[5..all_end]
                .iter()
                .copied()
                .map(CardId::new)
                .collect();
            let offered_count = payload[all_end] as usize;
            if payload.len() != all_end + 1 + offered_count {
                return Err(EffectError::EngineFault("bad 灵慧候选牌 payload".into()));
            }
            if cancelled || selections.is_empty() {
                return run_effects(
                    tx,
                    vec![EffectRequest::ResolveBailingyun {
                        player,
                        target: PlayerId::NONE,
                        card: CardId::NONE,
                        action: 4,
                        cards,
                        skill,
                    }],
                )
                .map(|_| ());
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal("灵慧至多选择一张牌".into()));
            };
            if !payload[all_end + 1..].contains(selected) {
                return Err(EffectError::Illegal("灵慧所选牌不在候选中".into()));
            }
            let selected = CardId::new(*selected);
            let remaining = cards.into_iter().filter(|card| *card != selected).collect();
            run_effects(
                tx,
                vec![EffectRequest::ResolveBailingyun {
                    player,
                    target: PlayerId::NONE,
                    card: selected,
                    action: 2,
                    cards: remaining,
                    skill,
                }],
            )
            .map(|_| ())
        }
        1 => {
            if cancelled || payload.len() < 9 {
                return Err(EffectError::Illegal("灵慧用牌目标不可取消".into()));
            }
            let card = CardId::new(payload[4]);
            let min = payload[5] as usize;
            let max = payload[6] as usize;
            let remaining_count = payload[7] as usize;
            let remaining_end = 8usize.saturating_add(remaining_count);
            if payload.len() <= remaining_end {
                return Err(EffectError::EngineFault("bad 灵慧目标 payload".into()));
            }
            let remaining = payload[8..remaining_end]
                .iter()
                .copied()
                .map(CardId::new)
                .collect::<Vec<_>>();
            let option_count = payload[remaining_end] as usize;
            if payload.len() != remaining_end + 1 + option_count {
                return Err(EffectError::EngineFault("bad 灵慧目标候选 payload".into()));
            }
            let offered = &payload[remaining_end + 1..];
            let mut unique = std::collections::BTreeSet::new();
            if selections.len() < min
                || selections.len() > max
                || !selections
                    .iter()
                    .all(|target| unique.insert(*target) && offered.contains(target))
            {
                return Err(EffectError::Illegal("灵慧用牌目标非法".into()));
            }
            let crate::command::play::SkillCardTargetSpec::Select {
                min: current_min,
                max: current_max,
                options,
            } = crate::command::play::skill_card_target_spec(tx, player, card)?
            else {
                return Err(EffectError::Illegal("灵慧用牌目标形状已变化".into()));
            };
            let targets: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
            if targets.len() < current_min as usize
                || targets.len() > current_max as usize
                || targets.iter().any(|target| !options.contains(target))
            {
                return Err(EffectError::Illegal("灵慧用牌目标已失效".into()));
            }
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveBailingyun {
                    player,
                    target: PlayerId::NONE,
                    card: CardId::NONE,
                    action: 3,
                    cards: remaining,
                    skill,
                },
            );
            crate::command::play::handle_skill_card(tx, player, card, &targets)
        }
        2 => {
            if cancelled || selections.len() != 1 || payload.len() < 8 {
                return Err(EffectError::Illegal("灵慧目标对选择非法".into()));
            }
            let card = CardId::new(payload[4]);
            let remaining_count = payload[5] as usize;
            let remaining_end = 6usize.saturating_add(remaining_count);
            if payload.len() <= remaining_end {
                return Err(EffectError::EngineFault("bad 灵慧目标对 payload".into()));
            }
            let remaining = payload[6..remaining_end]
                .iter()
                .copied()
                .map(CardId::new)
                .collect::<Vec<_>>();
            let pair_count = payload[remaining_end] as usize;
            if payload.len() != remaining_end + 1 + pair_count * 2 {
                return Err(EffectError::EngineFault(
                    "bad 灵慧目标对候选 payload".into(),
                ));
            }
            let crate::command::play::SkillCardTargetSpec::Pairs(pairs) =
                crate::command::play::skill_card_target_spec(tx, player, card)?
            else {
                return Err(EffectError::Illegal("灵慧目标对形状已变化".into()));
            };
            let index = selections[0]
                .checked_sub(1)
                .map(|value| value as usize)
                .ok_or_else(|| EffectError::Illegal("灵慧目标对索引非法".into()))?;
            let pair = pairs
                .get(index)
                .copied()
                .ok_or_else(|| EffectError::Illegal("灵慧目标对已失效".into()))?;
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveBailingyun {
                    player,
                    target: PlayerId::NONE,
                    card: CardId::NONE,
                    action: 3,
                    cards: remaining,
                    skill,
                },
            );
            crate::command::play::handle_skill_card(tx, player, card, &[pair.0, pair.1])
        }
        3 => {
            if cancelled || selections.is_empty() {
                return Ok(());
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal("黠策须选择一名角色".into()));
            };
            let count = payload[4] as usize;
            if payload.len() != 5 + count || !payload[5..].contains(selected) {
                return Err(EffectError::Illegal("黠策目标不在候选中".into()));
            }
            let target = PlayerId::new(*selected);
            let key = crate::state::turn_flag::xiace_used_by_player(player);
            let mask = tx.state().turn.flag(key).clamp(0, u8::MAX as i32) as u8;
            if !skill_enabled(tx)
                || mask & 1 != 0
                || target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                || tx
                    .state()
                    .turn
                    .non_locked_skill_suppressed_players
                    .contains(&target)
            {
                return Err(EffectError::Illegal("黠策目标已失效".into()));
            }
            tx.state_mut().turn.set_flag(key, i32::from(mask | 1));
            run_effects(
                tx,
                vec![EffectRequest::SuppressNonLockedSkillsUntilTurnEnd { player: target }],
            )
            .map(|_| ())
        }
        4 => {
            if cancelled || selections.is_empty() {
                return Ok(());
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal("黠策须弃置一张牌".into()));
            };
            let count = payload[4] as usize;
            let card = CardId::new(*selected);
            let key = crate::state::turn_flag::xiace_used_by_player(player);
            let mask = tx.state().turn.flag(key).clamp(0, u8::MAX as i32) as u8;
            if payload.len() != 5 + count
                || !payload[5..].contains(selected)
                || !skill_enabled(tx)
                || mask & 2 != 0
                || !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&card)
                || !tx.state().players.get(player).is_some_and(|participant| {
                    participant.alive && participant.hp < participant.max_hp as i32
                })
            {
                return Err(EffectError::Illegal("黠策弃牌已失效".into()));
            }
            tx.state_mut().turn.set_flag(key, i32::from(mask | 2));
            run_effects(
                tx,
                vec![
                    EffectRequest::DiscardCards {
                        player,
                        cards: vec![card],
                    },
                    EffectRequest::RecoverHp {
                        source: player,
                        player,
                        amount: 1,
                    },
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault("bad 柏灵筠 choice stage".into())),
    }
}

pub(crate) fn resolve_dcdichou_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 || payload[0] != crate::effect::RESOLVE_DCDICHOU_CHOICE {
        return Err(EffectError::EngineFault("bad 涤仇 choice payload".into()));
    }
    let stage = payload[1];
    let player = PlayerId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    let helper_skill_def = engine_api::SkillDefId::new(payload[4]);
    if observer != player || cancelled {
        return Err(EffectError::Illegal("涤仇选择不可取消或代选".into()));
    }
    let mark = tx
        .state()
        .skills
        .get(skill)
        .filter(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        .map(|instance| engine_api::MarkDefId::new(instance.def.raw()))
        .ok_or_else(|| EffectError::Illegal("涤仇技能已失效".into()))?;
    let mark_count = |tx: &Transaction, target: PlayerId| {
        tx.state()
            .players
            .get(target)
            .and_then(|participant| participant.marks.iter().find(|(id, _)| *id == mark.raw()))
            .map(|(_, count)| *count)
            .unwrap_or(0)
    };

    match stage {
        0 => {
            let [slot] = selections else {
                return Err(EffectError::Illegal("涤仇须废除一个装备栏".into()));
            };
            let slot = u8::try_from(*slot)
                .ok()
                .and_then(crate::state::EquipSlot::from_tag)
                .ok_or_else(|| EffectError::Illegal("涤仇装备栏非法".into()))?;
            if !tx
                .state()
                .players
                .get(player)
                .is_some_and(|owner| owner.disabled_equip_slots[slot.index()] == 0)
            {
                return Err(EffectError::Illegal("涤仇装备栏已失效".into()));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::DisableEquipSlots {
                        player,
                        slots: vec![slot as u8],
                    },
                    EffectRequest::ResolveDcdichou {
                        player,
                        target: PlayerId::NONE,
                        card: CardId::NONE,
                        action: 1,
                        helper_skill_def,
                        skill,
                    },
                ],
            )
            .map(|_| ())
        }
        1 => {
            let [selected] = selections else {
                return Err(EffectError::Illegal("涤仇须选择一名有仇角色".into()));
            };
            let count = payload.get(5).copied().unwrap_or(0) as usize;
            if payload.len() != 6 + count || !payload[6..].contains(selected) {
                return Err(EffectError::Illegal("涤仇目标不在候选中".into()));
            }
            let target = PlayerId::new(*selected);
            if mark_count(tx, target) <= 0
                || crate::query::discardable_hand_equip_cards(tx.state(), target).is_empty()
            {
                return Err(EffectError::Illegal("涤仇目标已失效".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDcdichou {
                    player,
                    target,
                    card: CardId::NONE,
                    action: 2,
                    helper_skill_def,
                    skill,
                }],
            )
            .map(|_| ())
        }
        2 => {
            let [selected] = selections else {
                return Err(EffectError::Illegal("涤仇须弃置一张牌".into()));
            };
            let target = PlayerId::new(*payload.get(5).unwrap_or(&0));
            let count = payload.get(6).copied().unwrap_or(0) as usize;
            let card = CardId::new(*selected);
            if payload.len() != 7 + count
                || !payload[7..].contains(selected)
                || mark_count(tx, target) <= 0
                || !crate::query::discardable_hand_equip_cards(tx.state(), target).contains(&card)
            {
                return Err(EffectError::Illegal("涤仇弃牌已失效".into()));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::DiscardCards {
                        player: target,
                        cards: vec![card],
                    },
                    EffectRequest::ResolveDcdichou {
                        player,
                        target,
                        card,
                        action: 3,
                        helper_skill_def,
                        skill,
                    },
                ],
            )
            .map(|_| ())
        }
        10 => {
            let count = payload.get(5).copied().unwrap_or(0) as usize;
            if payload.len() != 6 + count || selections.is_empty() {
                return Err(EffectError::Illegal("涤仇结束阶段目标非法".into()));
            }
            let offered = &payload[6..];
            let selected: std::collections::BTreeSet<_> = selections.iter().copied().collect();
            if selected.len() != selections.len()
                || !selections.iter().all(|target| offered.contains(target))
            {
                return Err(EffectError::Illegal("涤仇结束阶段目标不在候选中".into()));
            }
            // 标记已在 action 10 开选择前全量清除；这里只按座位序结算所选
            // 目标的火焰伤害，不再二次清标记（未选者同样已被清除）。
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| selected.contains(&target.raw()))
                .collect();
            if targets.len() != selections.len() {
                return Err(EffectError::Illegal("涤仇结束阶段目标已失效".into()));
            }
            let mut effects = Vec::with_capacity(targets.len());
            for target in targets {
                effects.push(EffectRequest::DealDamage {
                    source: player,
                    target,
                    amount: 1,
                    is_lose_hp: false,
                    nature: rule_api::damage_nature::FIRE,
                });
            }
            run_effects(tx, effects).map(|_| ())
        }
        _ => Err(EffectError::EngineFault("bad 涤仇 choice stage".into())),
    }
}

pub(crate) fn resolve_dcdufeng_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 6 || payload[0] != crate::effect::RESOLVE_DCDUFENG_CHOICE {
        return Err(EffectError::EngineFault("bad 独锋 choice payload".into()));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    let helper_skill_def = engine_api::SkillDefId::new(payload[3]);
    let count = payload[4] as usize;
    let offered = &payload[5..];
    let selected: std::collections::BTreeSet<_> = selections.iter().copied().collect();
    if observer != player
        || cancelled
        || payload.len() != 5 + count
        || selections.is_empty()
        || selections.len() > 2
        || selected.len() != selections.len()
        || !selections.iter().all(|choice| offered.contains(choice))
        || selections.iter().filter(|choice| **choice != 0).count() > 1
    {
        return Err(EffectError::Illegal(
            "独锋须选择失去体力和/或一个装备栏".into(),
        ));
    }
    if !tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    }) {
        return Err(EffectError::Illegal("独锋技能已失效".into()));
    }

    let slot = selections.iter().copied().find(|choice| *choice != 0);
    if let Some(tag) = slot {
        let slot = u8::try_from(tag)
            .ok()
            .and_then(crate::state::EquipSlot::from_tag)
            .ok_or_else(|| EffectError::Illegal("独锋装备栏非法".into()))?;
        if !tx
            .state()
            .players
            .get(player)
            .is_some_and(|owner| owner.disabled_equip_slots[slot.index()] == 0)
        {
            return Err(EffectError::Illegal("独锋装备栏已失效".into()));
        }
    }

    let mut effects = Vec::with_capacity(3);
    if selections.contains(&0) {
        effects.push(EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: player,
            amount: 1,
            is_lose_hp: true,
            nature: rule_api::damage_nature::NORMAL,
        });
    }
    if let Some(slot) = slot {
        effects.push(EffectRequest::DisableEquipSlots {
            player,
            slots: vec![slot as u8],
        });
    }
    effects.push(EffectRequest::ResolveDcdufeng {
        player,
        action: 1,
        helper_skill_def,
        skill,
    });
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolve_kongsheng_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    let count = *payload.get(4).unwrap_or(&0) as usize;
    if observer != player {
        return Err(EffectError::Illegal(
            "箜声 target choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let target = PlayerId::new(selections[0]);
    if !tx.state().players.get(target).is_some_and(|p| p.alive) {
        return Err(EffectError::Illegal(
            "箜声 target is no longer legal".into(),
        ));
    }
    let def = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.def.raw())
        .ok_or_else(|| EffectError::EngineFault("箜声 skill disappeared".into()))?;
    let pile_zone = tx
        .state()
        .skill_piles
        .iter()
        .find(|pile| pile.owner == player && pile.skill_def == def)
        .map(|pile| pile.zone)
        .ok_or_else(|| EffectError::Illegal("no 箜 pile".into()))?;
    // 先压入第一件的移牌步骤，由 deferred 恢复驱动依次移入并使用。
    for i in 0..count {
        let card = CardId::new(*payload.get(5 + i).unwrap_or(&0));
        tx.state_mut()
            .resolution
            .deferred_effects
            .push(EffectRequest::ResolveKongsheng {
                player,
                stage: 4,
                target,
                card,
                skill,
            });
    }
    Ok(())
}

/// 巧变：跳过当前阶段（供各续体在收益结算后调用）。
pub(crate) fn qiaobian_skip_phase(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<(), EffectError> {
    crate::effect::run_effects(tx, vec![EffectRequest::SkipCurrentPhase { player }]).map(|_| ())
}
