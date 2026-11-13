use super::*;

/// 杀的结算：询问目标闪，未闪则造成伤害子事件。
/// 裸衣伤害加成：来源本回合已发动裸衣（`once_used` 旗标，判定阶段
/// 开始时随 `reset_turn_limits` 清除）→ 杀/决斗伤害 +1。
pub(super) fn luoyi_bonus(ctx: &Ctx<'_>, source: Option<PlayerId>) -> u32 {
    let Some(s) = source else { return 0 };
    let Some(def) = ctx.catalog.skill_by_name(crate::vocabulary::names::LUO_YI) else {
        return 0;
    };
    match ctx.core.skills.get(&(s, def)) {
        Some(inst) if inst.enabled && inst.once_used => 1,
        _ => 0,
    }
}

/// 凶兽（Boss 模式共有技，规则 v208）：来源持凶兽且目标体力小于来源
/// 时，来源使用【杀】造成的伤害 +1（上游 boss_xiongshou_damage
/// `damageBegin1` filter hp< 同口径）。
pub(super) fn xiongshou_bonus(ctx: &Ctx<'_>, source: PlayerId, target: PlayerId) -> u32 {
    let Some(def) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_XIONGSHOU)
    else {
        return 0;
    };
    let enabled = ctx
        .core
        .skills
        .get(&(source, def))
        .map(|i| i.enabled)
        .unwrap_or(false);
    if enabled && ctx.core.player(target).hp < ctx.core.player(source).hp {
        1
    } else {
        0
    }
}

/// 贪欲手牌「全场最多」判定：持有者手牌数不少于任何其他存活角色。
pub(crate) fn tanyu_is_max_hand(core: &crate::state::GameStateCore, holder: PlayerId) -> bool {
    let my = core.player(holder).hand.len();
    core.alive_players()
        .all(|o| o == holder || core.player(o).hand.len() <= my)
}

/// 覆泰2（Boss 穷奇，规则 v214）：场上存有持覆泰的穷奇、用/救桃者非
/// 穷奇且当前回合非穷奇时，桃不可用/不可救（上游 boss_futai2
/// cardSavable/cardEnabled 双 mod 同口径）。
pub fn tao_futai2_banned(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    user: PlayerId,
) -> bool {
    let Some(fdef) = catalog.skill_by_name(crate::vocabulary::names::BOSS_FUTAI) else {
        return false;
    };
    let holders: Vec<PlayerId> = core
        .skills
        .iter()
        .filter(|((_, dd), i)| *dd == fdef && i.enabled)
        .map(|((o, _), _)| *o)
        .filter(|o| core.player(*o).alive && *o != user)
        .collect();
    match holders.first() {
        Some(&boss) => core.turn.current != boss,
        None => false,
    }
}

/// P1/N08 急救（华佗）：持有者濒死结算时红色手牌可当【桃】使用。
pub(super) fn is_tao_like(
    ctx: &Ctx<'_>,
    responder: PlayerId,
    card: crate::ids::CardId,
    dying: PlayerId,
) -> bool {
    if crate::rvm_bridge::modifiers::eval_dying_peach_target_canceled(
        ctx.core,
        ctx.catalog,
        responder,
        dying,
        card,
    ) || crate::rvm_bridge::modifiers::eval_card_use_canceled(
        ctx.core,
        ctx.catalog,
        responder,
        card,
        ctx.catalog.card_kind(card),
        crate::event::DamageNature::Normal,
    ) {
        return false;
    }
    if ctx.catalog.card_kind(card) == CardKind::Tao {
        // 覆泰2（规则 v214）：全场禁桃时真桃不可救
        return !tao_futai2_banned(ctx.core, ctx.catalog, responder);
    }
    // 酒（G1 cardpile 前置）：濒死时对自身使用回复 1 点体力
    if ctx.catalog.card_kind(card) == CardKind::Jiu {
        return true;
    }
    // 护符（U05 swd 首切片，规则 v85）：仅濒死者本人的桃形救援
    //（上游 savable: dying === player）
    if ctx.catalog.card_kind(card) == CardKind::Hufu {
        return responder == dying;
    }
    // 天仙酒（U05 swd 第五批，规则 v89）：酒的 swd 变体——濒死自救
    if ctx.catalog.card_kind(card) == CardKind::Tianxianjiu {
        return true;
    }
    // 急救（v202 对齐上游 viewAsFilter）：限持有者回合外——自己是当前
    // 回合角色时不可发动
    let jijiu = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::JI_JIU)
        .map(|d| ctx.core.has_skill(responder, d) && ctx.core.turn.current != responder)
        .unwrap_or(false);
    if jijiu
        && matches!(
            ctx.catalog.cards[card as usize].suit,
            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
        )
    {
        return true;
    }
    // 复燃（Boss 焰灵，规则 v241）：焰灵濒死时其敌方可用红色牌当桃救
    //（上游 boss_furan2 dying chooseToUse viewAs tao 同口径）
    let furan = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_FURAN)
        .map(|d| ctx.core.has_skill(dying, d))
        .unwrap_or(false);
    if furan
        && ctx.core.is_boss_side(responder) != ctx.core.is_boss_side(dying)
        && matches!(
            ctx.catalog.cards[card as usize].suit,
            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
        )
    {
        return true;
    }
    // 蜜链藕（G1 古剑食物）：持有者红桃手牌可当桃（濒死响应）
    let mizhilianou = ctx
        .catalog
        .skill_by_name(crate::vocabulary::host::GJ_FOOD_MIZHLIANOU_KEY)
        .map(|d| ctx.core.has_skill(responder, d))
        .unwrap_or(false);
    if mizhilianou && ctx.catalog.cards[card as usize].suit == crate::catalog::Suit::Heart {
        return true;
    }
    if ctx
        .catalog
        .find_player_vm_view_as(ctx.core, responder, card, false, Some(CardKind::Tao))
        .is_some()
        && !tao_futai2_banned(ctx.core, ctx.catalog, responder)
    {
        return true;
    }
    if responder == dying
        && ctx
            .catalog
            .find_player_vm_view_as(ctx.core, responder, card, false, Some(CardKind::Jiu))
            .is_some()
    {
        return true;
    }
    // 帝女玄霜（G1 古剑，濒死可救基本牌）：按桃结算回复 1 体力
    //（弃摸骑乘由 DyingSaveMachine rider 承担）
    ctx.catalog.card_kind(card) == CardKind::Dinvxuanshuang
}

pub(super) fn dying_tao_cards(
    ctx: &Ctx<'_>,
    responder: PlayerId,
    dying: PlayerId,
) -> Vec<crate::ids::CardId> {
    let p = ctx.core.player(responder);
    let mut out: Vec<crate::ids::CardId> = p
        .hand
        .iter()
        .copied()
        .filter(|&c| is_tao_like(ctx, responder, c, dying))
        .collect();
    for c in ctx
        .catalog
        .player_vm_view_as_extra_cards(ctx.core, responder, false, None)
    {
        if is_tao_like(ctx, responder, c, dying) && !out.contains(&c) {
            out.push(c);
        }
    }
    out
}

pub(super) fn response_kind_cards(
    ctx: &Ctx<'_>,
    responder: PlayerId,
    need: CardKind,
) -> Vec<crate::ids::CardId> {
    let hand_banned = (need.is_basic_kind() || need.is_equip())
        && ctx
            .core
            .vm_aux
            .flags
            .get(&(responder, "mbcmfangzhu_ban".to_string()))
            .copied()
            .unwrap_or(0)
            > 0;
    let p = ctx.core.player(responder);
    let mut out: Vec<crate::ids::CardId> = if hand_banned {
        Vec::new()
    } else {
        p.hand
            .iter()
            .copied()
            .filter(|&c| {
                ctx.catalog.card_kind(c) == need
                    || ctx
                        .catalog
                        .find_player_vm_view_as(ctx.core, responder, c, false, Some(need))
                        .is_some()
            })
            .collect()
    };
    for c in ctx
        .catalog
        .player_vm_view_as_extra_cards(ctx.core, responder, false, Some(need))
    {
        if !out.contains(&c) {
            out.push(c);
        }
    }
    out
}

pub(super) fn discard_response_card(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
    card: crate::ids::CardId,
    target_kind: CardKind,
) {
    let in_hand = ctx.core.player(player).hand.contains(&card);
    let mut from_none = false;
    if ctx.catalog.card_kind(card) != target_kind || !in_hand {
        if let Some(spec) = ctx
            .catalog
            .find_player_vm_view_as(ctx.core, player, card, false, Some(target_kind))
            .cloned()
        {
            let extra = spec.consume_and_record(ctx.core, ctx.catalog, player, card);
            for c2 in extra {
                ctx.to_discard(c2);
            }
            if spec.zone == crate::rvm_bridge::VmViewAsZone::None {
                from_none = true;
            }
        }
    } else {
        ctx.core
            .vm_aux
            .flags
            .remove(&(player, "event_view_as_skill".to_string()));
    }
    if from_none {
        return;
    }
    if ctx.core.player(player).hand.contains(&card) {
        ctx.discard_from_hand(player, card);
    } else if ctx.core.player(player).equips.contains(&card) {
        ctx.core.player_mut(player).equips.retain(|x| *x != card);
        ctx.core.declared_slots.remove(&card);
        revoke_equip_skills(ctx.core, ctx.catalog, player, card);
        ctx.to_discard(card);
        ctx.log(LogEvent::CardDiscarded {
            player,
            cards: vec![card],
        });
        ctx.emit(EventData::CardsLost {
            player,
            cards: vec![card],
            origins: vec![crate::event::LostOrigin::Equip],
        });
    } else if ctx
        .core
        .vm_aux
        .piles
        .iter_mut()
        .filter(|((owner, _), _)| *owner == player)
        .any(|(_, pile)| {
            let len_before = pile.len();
            pile.retain(|&x| x != card);
            pile.len() != len_before
        })
    {
        ctx.to_discard(card);
        ctx.log(LogEvent::CardDiscarded {
            player,
            cards: vec![card],
        });
    } else {
        ctx.to_discard(card);
    }
}

pub(super) fn discard_dying_card(
    ctx: &mut Ctx<'_>,
    responder: PlayerId,
    dying: PlayerId,
    card: crate::ids::CardId,
) {
    let in_hand = ctx.core.player(responder).hand.contains(&card);
    let mut from_none = false;
    if !matches!(
        ctx.catalog.card_kind(card),
        CardKind::Tao | CardKind::Jiu | CardKind::Hufu | CardKind::Tianxianjiu | CardKind::Dinvxuanshuang
    ) || !in_hand
    {
        let spec = ctx
            .catalog
            .find_player_vm_view_as(ctx.core, responder, card, false, Some(CardKind::Tao))
            .or_else(|| {
                (responder == dying)
                    .then(|| {
                        ctx.catalog.find_player_vm_view_as(
                            ctx.core,
                            responder,
                            card,
                            false,
                            Some(CardKind::Jiu),
                        )
                    })
                    .flatten()
            })
            .cloned();
        if let Some(s) = spec {
            let extra = s.consume_and_record(ctx.core, ctx.catalog, responder, card);
            for c2 in extra {
                ctx.to_discard(c2);
            }
            if s.zone == crate::rvm_bridge::VmViewAsZone::None {
                from_none = true;
            }
        }
    }
    if from_none {
        return;
    }
    if ctx.core.player(responder).hand.contains(&card) {
        ctx.discard_from_hand(responder, card);
    } else if ctx.core.player(responder).equips.contains(&card) {
        ctx.core.player_mut(responder).equips.retain(|x| *x != card);
        ctx.core.declared_slots.remove(&card);
        revoke_equip_skills(ctx.core, ctx.catalog, responder, card);
        ctx.to_discard(card);
        ctx.log(LogEvent::CardDiscarded {
            player: responder,
            cards: vec![card],
        });
        ctx.emit(EventData::CardsLost {
            player: responder,
            cards: vec![card],
            origins: vec![crate::event::LostOrigin::Equip],
        });
    } else if ctx
        .core
        .vm_aux
        .piles
        .iter_mut()
        .filter(|((owner, _), _)| *owner == responder)
        .any(|(_, pile)| {
            let len_before = pile.len();
            pile.retain(|&x| x != card);
            pile.len() != len_before
        })
    {
        ctx.to_discard(card);
        ctx.log(LogEvent::CardDiscarded {
            player: responder,
            cards: vec![card],
        });
    } else {
        ctx.to_discard(card);
    }
}
