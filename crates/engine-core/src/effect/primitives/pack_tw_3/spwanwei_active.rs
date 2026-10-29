//! Effect primitives for `resolve_twchenxun` and related helpers.

use super::*;

/// 挽危（主动阶段）：目标回复 player.hp 点体力，拥有者失去 player.hp 点体力。
pub(crate) fn spwanwei_active(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let p = tx
        .state()
        .players
        .get(player)
        .ok_or_else(|| EffectError::Illegal("bad player".into()))?;
    let t = tx
        .state()
        .players
        .get(target)
        .ok_or_else(|| EffectError::Illegal("bad target".into()))?;
    if !p.alive || !t.alive || p.hp < 1 {
        return Ok(Vec::new());
    }
    let num = p.hp as u32;
    let mut events = Vec::new();
    events.extend(apply_effect(
        tx,
        EffectRequest::RecoverHp {
            source: player,
            player: target,
            amount: num,
        },
    )?);
    events.extend(apply_effect(
        tx,
        EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: player,
            amount: num,
            is_lose_hp: true,
            nature: rule_api::damage_nature::NORMAL,
        },
    )?);
    Ok(events)
}

/// 挽危（濒死救援）：目标回复 max(player.hp + 1, 1 - target.hp) 体力，拥有者失去 player.hp 点体力。
pub(crate) fn spwanwei_dying(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let p = tx
        .state()
        .players
        .get(player)
        .ok_or_else(|| EffectError::Illegal("bad player".into()))?;
    let t = tx
        .state()
        .players
        .get(target)
        .ok_or_else(|| EffectError::Illegal("bad target".into()))?;
    if !p.alive || !t.alive || p.hp < 1 {
        return Ok(Vec::new());
    }
    let num = p.hp as u32;
    let needed = if t.hp < 1 { (1 - t.hp) as u32 } else { 1 };
    let recover_amount = (num + 1).max(needed);
    let mut events = Vec::new();
    events.extend(apply_effect(
        tx,
        EffectRequest::RecoverHp {
            source: player,
            player: target,
            amount: recover_amount,
        },
    )?);
    events.extend(apply_effect(
        tx,
        EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: player,
            amount: num,
            is_lose_hp: true,
            nature: rule_api::damage_nature::NORMAL,
        },
    )?);
    Ok(events)
}

/// 殉义（王甫赵累，ABI 412）：mode 0 选择“义”目标（获义标记）；mode 1 伤害事件分流
/// ——拥有者/义目标任一造成伤害另一人摸 1、任一受伤另一人强制弃 1 张手牌/装备。
/// 选择义目标的 choice 由 resolve_choice 的 RESOLVE_XUNYI_SELECT 分支结算。
pub(crate) fn resolve_xunyi(
    tx: &mut Transaction,
    player: PlayerId,
    mode: u8,
    source: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    // “义”标记定义 ID（= xunyi 技能定义 ID）。
    const YI_MARK: u32 = 2_517_009_871;
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    match mode {
        0 => {
            // 已存在义标记者（含已死目标）则不重复选择。
            if tx
                .state()
                .players
                .iter_live()
                .any(|p| p.id != player && p.marks.iter().any(|(m, n)| *m == YI_MARK && *n > 0))
            {
                return Ok(Vec::new());
            }
            let candidates: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| *candidate != player)
                .map(|candidate| candidate.raw())
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
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![RESOLVE_XUNYI_SELECT, player.raw(), skill.raw()],
            )
        }
        1 => {
            let yi =
                tx.state()
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .find(|candidate| {
                        *candidate != player
                            && tx.state().players.get(*candidate).is_some_and(|p| {
                                p.marks.iter().any(|(m, n)| *m == YI_MARK && *n > 0)
                            })
                    });
            let mut effects = Vec::new();
            if source == player {
                // 王甫造成伤害 → 义目标摸 1。
                if let Some(yi) = yi {
                    effects.push(EffectRequest::DrawCards {
                        player: yi,
                        count: 1,
                    });
                }
            } else if yi == Some(source) {
                // 义目标造成伤害 → 王甫摸 1。
                effects.push(EffectRequest::DrawCards { player, count: 1 });
            } else if target == player {
                // 王甫受伤 → 义目标强制弃 1 张手牌/装备。
                if let Some(yi) = yi {
                    effects.push(EffectRequest::ChooseDiscardOneHandOrEquip { player: yi });
                }
            } else if yi == Some(target) {
                // 义目标受伤 → 王甫强制弃 1 张手牌/装备。
                effects.push(EffectRequest::ChooseDiscardOneHandOrEquip { player });
            }
            run_effects(tx, effects)
        }
        _ => Err(EffectError::ContentFault("unknown Xunyi mode".into())),
    }
}

/// 博明（许靖，ABI 413）：把主动技选中的牌赠给目标（触发 CardsGiven 事件）。
pub(crate) fn resolve_boming(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || card.is_none()
        || target == player
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let from = tx
        .state()
        .cards
        .get(card)
        .map(|card| card.zone)
        .ok_or_else(|| EffectError::Illegal("boming card missing".into()))?;
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("target hand missing".into()))?;
    give_cards_as(tx, player, vec![card], to, from)
}

/// 恶荐（许靖，ABI 414）：获牌者除赠牌外仍有同类（基本/锦囊/装备）手牌或装备牌时，
/// 二选一——受 1 点伤害 / 展示并弃所有同类牌。选择由 resolve_choice 的
/// RESOLVE_EJIAN_CHOICE 分支结算。
pub(crate) fn resolve_ejian(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) || card.is_none() {
        return Ok(Vec::new());
    }
    let category = card_category_of(tx.state(), card);
    let has_same = [ZoneKind::Hand, ZoneKind::Equip]
        .into_iter()
        .filter_map(|kind| tx.state().zones.find(kind, target))
        .filter_map(|zone| tx.state().zones.get(zone))
        .flat_map(|zone| zone.cards.iter().copied())
        .any(|candidate| candidate != card && card_category_of(tx.state(), candidate) == category);
    if !has_same {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        target,
        1,
        1,
        false,
        vec![0, 1],
        ChoiceKind::Custom,
        ContinuationKind::Custom,
        vec![RESOLVE_EJIAN_CHOICE, player.raw(), target.raw(), card.raw()],
    )
}

/// 异勇（王双，ABI 415）：受到【杀】伤害后获得该杀实体牌，若来源存活且实体牌在
/// 手牌中，立即用其反击来源（来源无武器则伤害基数 2；实体牌随反击消耗）。
pub(crate) fn resolve_yiyong(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    source: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || card.is_none()
        || source == player
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let mut effects = Vec::new();
    // 1. 获得弃牌堆中的杀实体。
    let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
    let gained = discard.is_some_and(|discard| {
        tx.state()
            .cards
            .get(card)
            .is_some_and(|c| c.zone == discard && c.kind == CardKind::Sha)
    });
    if gained {
        let hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?;
        effects.push(EffectRequest::MoveCards {
            cards: vec![card],
            to: hand,
            from: discard,
        });
    }
    // 2. 反击条件：来源存活、实体牌已到手。
    let source_alive = tx
        .state()
        .players
        .get(source)
        .is_some_and(|participant| participant.alive);
    if gained && source_alive {
        // 来源有武器则基数 1，否则基数 2。
        let source_armed = tx
            .state()
            .zones
            .find(ZoneKind::Equip, source)
            .is_some_and(|zone| {
                tx.state().zones.get(zone).is_some_and(|z| {
                    z.cards.iter().any(|c| {
                        tx.state()
                            .cards
                            .get(*c)
                            .is_some_and(|v| v.kind == CardKind::Weapon)
                    })
                })
            });
        let base = if source_armed { 1 } else { 2 };
        effects.push(EffectRequest::DealDamage {
            source: player,
            target: source,
            amount: base,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        });
        // 实体牌随反击消耗（移入弃牌堆）。
        if let Some(discard) = discard {
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            if let Some(hand) = hand {
                effects.push(EffectRequest::MoveCards {
                    cards: vec![card],
                    to: discard,
                    from: Some(hand),
                });
            }
        }
    }
    run_effects(tx, effects)
}

/// 善械（王双，ABI 416）：牌堆（仅 DrawPile）有武器则获得其中一张；否则随机夺一名
/// 有武器角色的武器（随机数只使用可回放 RuleRng）。
pub(crate) fn resolve_shanxie(
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
    let to_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?;
    // 牌堆（仅 DrawPile）武器检索。
    let draw = tx.state().zones.find(ZoneKind::DrawPile, PlayerId::NONE);
    let weapon_in_draw = draw.and_then(|zone| {
        tx.state().zones.get(zone).and_then(|z| {
            z.cards.iter().rev().copied().find(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|v| v.kind == CardKind::Weapon)
            })
        })
    });
    if let Some(card) = weapon_in_draw {
        let from = tx
            .state()
            .cards
            .get(card)
            .map(|card| card.zone)
            .ok_or_else(|| EffectError::Illegal("weapon card missing".into()))?;
        return run_effects(
            tx,
            vec![EffectRequest::MoveCards {
                cards: vec![card],
                to: to_hand,
                from: Some(from),
            }],
        );
    }
    // 随机夺一名有武器角色的武器。
    let armed: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|candidate| *candidate != player)
        .filter(|candidate| {
            tx.state()
                .zones
                .find(ZoneKind::Equip, *candidate)
                .is_some_and(|zone| {
                    tx.state().zones.get(zone).is_some_and(|z| {
                        z.cards.iter().any(|c| {
                            tx.state()
                                .cards
                                .get(*c)
                                .is_some_and(|v| v.kind == CardKind::Weapon)
                        })
                    })
                })
        })
        .collect();
    if armed.is_empty() {
        return Ok(Vec::new());
    }
    let mut rng = crate::rng::RuleRng::from_state(tx.state().rng);
    let victim = armed[rng.next_u64() as usize % armed.len()];
    let weapons: Vec<_> = tx
        .state()
        .zones
        .find(ZoneKind::Equip, victim)
        .map(|zone| {
            tx.state()
                .zones
                .get(zone)
                .map(|z| {
                    z.cards
                        .iter()
                        .copied()
                        .filter(|c| {
                            tx.state()
                                .cards
                                .get(*c)
                                .is_some_and(|v| v.kind == CardKind::Weapon)
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        })
        .unwrap_or_default();
    if weapons.is_empty() {
        tx.state_mut().rng = rng.into_state();
        return Ok(Vec::new());
    }
    let card = weapons[rng.next_u64() as usize % weapons.len()];
    tx.state_mut().rng = rng.into_state();
    let from = tx
        .state()
        .cards
        .get(card)
        .map(|card| card.zone)
        .ok_or_else(|| EffectError::Illegal("weapon card missing".into()))?;
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: to_hand,
            from: Some(from),
        }],
    )
}

/// 卡牌粗类别标签（0=基本 1=锦囊 2=装备），供 owner_card_category_mask 位运算。
pub(crate) fn card_category_tag(kind: CardKind) -> Option<u32> {
    if kind.is_basic() {
        Some(0)
    } else if kind.is_trick() {
        Some(1)
    } else if kind.is_equip() {
        Some(2)
    } else {
        None
    }
}

/// 舍医（蔡贞姬，ABI 417）：其他角色受伤前（每轮一次，DSL filter 限）交至少
/// max(1, hp) 张手牌/装备牌防伤。选牌 choice 由 resolve_choice 的
/// RESOLVE_SHEYI_CHOICE 分支结算（赠牌 + prevent）。
pub(crate) fn resolve_sheyi(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
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
    // 每轮一次（引擎内检查；DSL filter 移除 once_per_round 以避免与
    // optional 确认流程的交互问题）。
    let round = tx.state().turn.round;
    if tx
        .state()
        .skills
        .get(skill)
        .is_some_and(|instance| instance.round_fire_round == round)
    {
        return Ok(Vec::new());
    }
    tx.state_mut()
        .skills
        .get_mut(skill)
        .map(|instance| instance.round_fire_round = round);
    let hp = tx.state().players.get(player).map(|p| p.hp).unwrap_or(0);
    let need = hp.max(1) as u32;
    let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
    if (options.len() as u32) < need {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        need as u8,
        options.len() as u8,
        true,
        options.iter().map(|card| card.raw()).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        vec![
            RESOLVE_SHEYI_CHOICE,
            player.raw(),
            target.raw(),
            skill.raw(),
        ],
    )
}

/// 天荫（蔡贞姬，ABI 418）：结束阶段获得牌堆顶每个本阶段未用类别牌的第一张。
pub(crate) fn resolve_tianyin(
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
    let mut seen = tx.state().turn.owner_card_category_mask(player) as u32 & 0b111;
    let draw = tx.state().zones.find(ZoneKind::DrawPile, PlayerId::NONE);
    let Some(draw) = draw else {
        return Ok(Vec::new());
    };
    let mut collected = Vec::new();
    for card in tx
        .state()
        .zones
        .get(draw)
        .map(|z| z.cards.iter().rev().copied().collect::<Vec<_>>())
        .unwrap_or_default()
    {
        let Some(category) = tx
            .state()
            .cards
            .get(card)
            .and_then(|value| card_category_tag(value.kind))
        else {
            continue;
        };
        let bit = 1u32 << category;
        if seen & bit == 0 {
            collected.push(card);
            seen |= bit;
            if seen & 0b111 == 0b111 {
                break;
            }
        }
    }
    if collected.is_empty() {
        return Ok(Vec::new());
    }
    let to_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?;
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: collected,
            to: to_hand,
            from: Some(draw),
        }],
    )
}

/// 追锋（标文鸯，ABI 419）：失 1 点体力，然后选择一名其他角色并对其视为使用
/// 一张【决斗】（无距离限制近似）。
pub(crate) fn resolve_dbzhuifeng(
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
        || tx.state().players.get(player).map(|p| p.hp).unwrap_or(0) <= 0
    {
        return Ok(Vec::new());
    }
    let mut events = Vec::new();
    events.extend(run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: player,
            amount: 1,
            is_lose_hp: true,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )?);
    let candidates: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|candidate| *candidate != player)
        .map(|candidate| candidate.raw())
        .collect();
    if candidates.is_empty() {
        return Ok(events);
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        candidates,
        ChoiceKind::Custom,
        ContinuationKind::Custom,
        vec![RESOLVE_DBZHUIFENG_TARGET, player.raw(), skill.raw()],
    )?;
    Ok(events)
}

/// 曲迪（标文鸯，ABI 420）：单目标杀/决斗指定后三选一。
pub(crate) fn resolve_dbquedi(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if card.is_none() {
        return Ok(Vec::new());
    }
    // 三个选项至少要有一个可执行（获牌 / 弃基本牌），否则不开选择。
    let can_gain = !crate::query::gainable_target_cards(
        tx.state(),
        player,
        target,
        rule_api::TargetCardScope::Hand,
    )
    .is_empty();
    let has_basic = tx.state().hand_cards(player).into_iter().any(|candidate| {
        tx.state()
            .cards
            .get(candidate)
            .is_some_and(|value| value.kind.is_basic())
    });
    if !can_gain && !has_basic {
        return Ok(Vec::new());
    }
    GuardedChoice::new(
        skill,
        Participants::distinct_other_only(player, target),
        ContinuationKind::Custom,
        vec![
            RESOLVE_DBQUEDI_CHOICE,
            player.raw(),
            target.raw(),
            card.raw(),
            skill.raw(),
        ],
    )
    .shape(ChoiceShape::one_optional())
    .open_tokens(tx, vec![0, 1, 2], ChoiceKind::Custom)
}

/// 隐据（辛毗，ABI 421）：目标二选一——对拥有者使用【杀】/ 获得跳过标记。
pub(crate) fn resolve_spyinju(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    GuardedChoice::new(
        skill,
        Participants::distinct_other_only(player, target),
        ContinuationKind::Custom,
        vec![
            RESOLVE_SPYINJU_CHOICE,
            player.raw(),
            target.raw(),
            skill.raw(),
        ],
    )
    .chosen_by(target)
    .open_tokens(tx, vec![0, 1], ChoiceKind::Custom)
}
