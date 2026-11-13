use super::*;

/// 装备提交：牌入手牌区到对应装备栏，同栏旧牌替换入弃牌堆；
/// 替换结果写回帧载荷（哈希可见）。
/// G1 装备技能生命周期：装备 `card` 时授予其声明的目录技能。
pub fn grant_equip_skills(ctx: &mut Ctx<'_>, player: PlayerId, card: crate::ids::CardId) {
    let names: Vec<&'static str> = ctx.catalog.cards[card as usize].equip_skills.clone();
    for name in names {
        if let Some(def) = ctx.catalog.skill_by_name(name) {
            ctx.core
                .skills
                .entry((player, def))
                .or_insert(crate::state::SkillInstance {
                    owner: player,
                    enabled: true,
                    once_used: false,
                    expires_end_of_turn: false,
                    rounds: 0,
                });
        }
    }
}

/// G1 装备技能生命周期：装备 `card` 离场时撤销其技能——若该技能不同时
/// 由玩家剩余的其他装备提供（跨槽同名技能共存）。
pub fn revoke_equip_skills(
    core: &mut crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    player: PlayerId,
    card: crate::ids::CardId,
) {
    let names: Vec<&'static str> = catalog.cards[card as usize].equip_skills.clone();
    for name in names {
        let Some(def) = catalog.skill_by_name(name) else {
            continue;
        };
        let still_provided = core
            .player(player)
            .equips
            .iter()
            .any(|c| catalog.cards[*c as usize].equip_skills.contains(&name));
        if !still_provided {
            if name == "th_taipingyaoshu" {
                core.vm_aux
                    .flags
                    .insert((player, "l28_taipingyaoshu_revoke_after_lost".to_string()), 1);
            } else {
                core.skills.remove(&(player, def));
            }
        }
    }
    // G1 索链甲：离场装备声明横置且玩家无其他同类装备 → 重置横置
    if catalog.cards[card as usize].link_on_equip {
        let still = core
            .player(player)
            .equips
            .iter()
            .any(|c| catalog.cards[*c as usize].link_on_equip);
        if !still {
            core.player_mut(player).linked = false;
        }
    }
}

pub(super) fn commit_equip(
    ctx: &mut Ctx<'_>,
    frame_id: EventId,
    player: PlayerId,
    card: crate::ids::CardId,
) {
    // 实际装备栏（swd 批 143 改造：declared_slots 覆盖优先）
    let Some(slot) = ctx.catalog.card_slot_of(ctx.core, card) else {
        panic!("commit_equip on non-equip card");
    };
    ctx.take_hand_card(player, card);
    let played = ctx
        .core
        .vm_aux
        .flags
        .remove(&(player, format!("dddanzhi_equip_use:{frame_id}")))
        .is_some();
    if played && consume_dddanzhi_ward(ctx, player, card) {
        ctx.to_discard(card);
        ctx.log(LogEvent::CardNullified { player, card });
        // 未执行装备效果，不得再派发 equipAfter（如劫行重置）。
        ctx.res.frame_mut(frame_id).expect("equip frame").stage = crate::event::Stage::Done;
        return;
    }
    // 同**栏位**替换（改造后同栏位不同牌种亦替换）
    let replaced = ctx
        .core
        .player(player)
        .equips
        .iter()
        .copied()
        .find(|c| ctx.catalog.card_slot_of(ctx.core, *c) == Some(slot));
    if let Some(old) = replaced {
        ctx.core.player_mut(player).equips.retain(|c| *c != old);
        revoke_equip_skills(ctx.core, ctx.catalog, player, old);
        ctx.note_cards_lost(player, &[old]);
        ctx.to_discard(old);
        if ctx
            .catalog
            .cards
            .get(old as usize)
            .is_some_and(|cd| cd.name == "th_taipingyaoshu")
        {
            ctx.emit(EventData::CardsLost {
                player,
                cards: vec![old],
                origins: vec![crate::event::LostOrigin::Equip],
            });
        }
    }
    ctx.core.player_mut(player).equips.push(card);
    grant_equip_skills(ctx, player, card);
    // 轩辕剑装备钩（swd 批 146，规则 v320）：体力 ≤2 且无「轩辕」→ 弃置；
    // 否则 +1 护甲（上游 onEquip 同口径）
    if ctx.catalog.card_kind(card) == crate::catalog::CardKind::Xuanyuanjian {
        let has_xuanyuan = ctx
            .catalog
            .skill_by_name(crate::vocabulary::host::SWD_XUANYUAN_KEY)
            .map(|d| ctx.core.has_skill(player, d))
            .unwrap_or(false);
        if !has_xuanyuan && ctx.core.player(player).hp <= 2 {
            ctx.core.player_mut(player).equips.retain(|c| *c != card);
            revoke_equip_skills(ctx.core, ctx.catalog, player, card);
            ctx.to_discard(card);
        } else {
            ctx.change_hujia(player, 1);
        }
    }
    // G1 索链甲：装备时横置
    if ctx.catalog.cards[card as usize].link_on_equip {
        let was = ctx.core.player(player).linked;
        ctx.core.player_mut(player).linked = true;
        if !was {
            ctx.log(LogEvent::Linked {
                player,
                linked: true,
            });
        }
    }
    if let Some(f) = ctx.res.frame_mut(frame_id) {
        if let EventData::Equipped { replaced: r, .. } = &mut f.data {
            *r = replaced;
        }
    }
    ctx.log(LogEvent::Equipped {
        player,
        card,
        replaced,
    });
}
