use super::*;

/// 装备提交：牌入手牌区到对应装备栏，同栏旧牌替换入弃牌堆；
/// 替换结果写回帧载荷（哈希可见）。
/// G1 装备技能生命周期：装备 `card` 时授予其声明的目录技能。
pub fn grant_equip_skills(ctx: &mut Ctx<'_>, player: PlayerId, card: crate::ids::CardId) {
    let names = ctx.catalog.equip_card_skills(ctx.core, card);
    for name in names {
        if let Some(def) = ctx.catalog.skill_by_name(name) {
            ctx.ensure_skill(
                (player, def),
                crate::state::SkillInstance {
                    owner: player,
                    enabled: true,
                    once_used: false,
                    expires_end_of_turn: false,
                    rounds: 0,
                },
            );
        }
    }
    (ctx.catalog.hooks.on_equip_gained)(ctx, player, card);
}

/// G1 装备技能生命周期：装备 `card` 离场时撤销其技能——若该技能不同时
/// 由玩家剩余的其他装备提供（跨槽同名技能共存）。
pub fn revoke_equip_skills(
    core: &mut crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    player: PlayerId,
    card: crate::ids::CardId,
) {
    let names = catalog.equip_card_skills(core, card);
    for name in names {
        let Some(def) = catalog.skill_by_name(name) else {
            continue;
        };
        let still_provided = core
            .player(player)
            .equips
            .iter()
            .any(|c| catalog.equip_card_skills(core, *c).contains(&name));
        if !still_provided {
            core.skills.remove(&(player, def));
            if name == "txhj_chongyingshenfu"
                || catalog
                    .cards
                    .get(card as usize)
                    .is_some_and(|cd| cd.name == "txhj_chongyingshenfu")
            {
                if let Some(eff_def) = catalog.skill_by_name("txhj_chongyingshenfu_effect") {
                    core.skills.remove(&(player, eff_def));
                }
                core.vm_aux
                    .flags
                    .retain(|(p, k), _| !(*p == player && k.starts_with("l11_chongying_rec:")));
            }
            if name == "baiyin"
                && crate::rvm_bridge::modifiers::d34_l28_mods::has_rewritten_baiyin_on_lose(
                    core, catalog, player, card,
                )
            {
                core.vm_aux
                    .flags
                    .insert((player, "l28_rw_baiyin_pending_lose".to_string()), 1);
            }
            if name == "th_taipingyaoshu" {
                core.vm_aux
                    .flags
                    .remove(&(player, "l28_taipingyaoshu_equipped".to_string()));
                core.vm_aux
                    .flags
                    .remove(&(player, "l28_taipingyaoshu_revoke_after_lost".to_string()));
                if core.player(player).alive
                    && !crate::rvm_bridge::modifiers::is_target_unequip2(core, player)
                {
                    core.vm_aux
                        .flags
                        .insert((player, "l28_taipingyaoshu_pending_lose".to_string()), 1);
                }
            }
            if name == "tianjitu_skill" && core.player(player).alive {
                core.vm_aux
                    .flags
                    .insert((player, "l28_tianjitu_pending_lose".to_string()), 1);
            }
        }
    }
    // G1 索链甲：离场装备声明横置且玩家无其他同类装备 → 重置横置
    if catalog.equip_card_links(core, card)
        || catalog.resolved_card_kind(core, card) == crate::catalog::CardKind::Suolianjia
    {
        let still = core.player(player).equips.iter().any(|c| {
            catalog.equip_card_links(core, *c)
                || catalog.resolved_card_kind(core, *c) == crate::catalog::CardKind::Suolianjia
        });
        if !still
            && core.player(player).linked
            && crate::rvm_bridge::modifiers::can_lose_link(core, catalog, player)
        {
            core.player_mut(player).linked = false;
        }
    }
    (catalog.hooks.on_equip_revoked)(core, catalog, player, card);
}

/// L25 `ssr_luoyi` (`extension.js:15031-15037`): 禁用防具栏并弃置已有防具。
pub fn discard_armor_for_ssr_luoyi(
    core: &mut crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    player: PlayerId,
) -> Vec<crate::ids::CardId> {
    let armors: Vec<crate::ids::CardId> = core
        .player(player)
        .equips
        .iter()
        .copied()
        .filter(|&c| catalog.card_slot_of(core, c) == Some(crate::catalog::EquipSlot::Armor))
        .collect();
    if armors.is_empty() {
        return Vec::new();
    }
    for &c in &armors {
        core.player_mut(player).equips.retain(|x| *x != c);
        revoke_equip_skills(core, catalog, player, c);
        core.discard.push(c);
    }
    armors
}

pub(super) fn commit_equip(
    ctx: &mut Ctx<'_>,
    frame_id: EventId,
    player: PlayerId,
    card: crate::ids::CardId,
) {
    if ctx
        .core
        .vm_aux
        .flags
        .remove(&(player, format!("l11_move_equip_evt:{card}")))
        .is_some()
    {
        ctx.log(LogEvent::Equipped {
            player,
            card,
            replaced: None,
        });
        return;
    }
    // 实际装备栏（swd 批 143 改造：declared_slots 覆盖优先）
    let Some(slot) = ctx.catalog.card_slot_of(ctx.core, card) else {
        panic!("commit_equip on non-equip card");
    };
    let from_table = ctx
        .core
        .vm_aux
        .flags
        .remove(&(player, format!("u01_equip_from_table:{frame_id}")))
        .is_some();
    if ctx
        .core
        .vm_aux
        .flags
        .remove(&(player, format!("u01_equip_cancelled:{frame_id}")))
        .is_some()
    {
        ctx.core
            .vm_aux
            .flags
            .remove(&(player, format!("dddanzhi_equip_use:{frame_id}")));
        ctx.core
            .vm_aux
            .flags
            .remove(&(player, format!("zhilian_equip_use:{frame_id}")));
        if let Some(f) = ctx.res.frame_mut(frame_id) {
            f.stage = crate::event::Stage::Done;
        }
        return;
    }
    let from_hand = !from_table && ctx.core.player(player).hand.contains(&card);
    if from_table {
        ctx.core.table.retain(|&c| c != card);
    } else {
        ctx.take_hand_card(player, card);
    }
    if !ctx.catalog.is_equip_slot_enabled(ctx.core, player, slot) {
        ctx.to_discard(card);
        (ctx.catalog.hooks.on_cards_to_discard)(ctx, player, &[card], false);
        return;
    }
    if from_hand && ctx.catalog.vm_card_history {
        ctx.core
            .vm_aux
            .flags
            .insert((player, format!("l11_equip_from_hand:{card}")), 1);
    }
    let played = ctx
        .core
        .vm_aux
        .flags
        .remove(&(player, format!("dddanzhi_equip_use:{frame_id}")))
        .is_some();
    let l11_played = ctx
        .core
        .vm_aux
        .flags
        .remove(&(player, format!("l11_equip_use:{frame_id}")))
        .is_some();
    if played && consume_dddanzhi_ward(ctx, player, card) {
        ctx.core
            .vm_aux
            .flags
            .remove(&(player, format!("zhilian_equip_use:{frame_id}")));
        ctx.to_discard(card);
        ctx.log(LogEvent::CardNullified { player, card });
        // 未执行装备效果，不得再派发 equipAfter（如劫行重置）。
        ctx.res.frame_mut(frame_id).expect("equip frame").stage = crate::event::Stage::Done;
        return;
    }
    if ctx
        .core
        .vm_aux
        .flags
        .remove(&(player, format!("zhilian_equip_use:{frame_id}")))
        .is_some()
    {
        super::record_turn_use_resp(ctx, player);
    }
    if played || l11_played {
        let eff_kind = ctx.catalog.card_kind(card);
        let suit = ctx.catalog.effective_card_suit(ctx.core, player, card);
        super::events::record_l11_turn_card_used(
            ctx.core,
            ctx.catalog,
            player,
            Some(card),
            Some(suit),
            eff_kind,
        );
    }
    install_equip_card(ctx, Some(frame_id), player, card);
}

pub(super) fn install_equip_card(
    ctx: &mut Ctx<'_>,
    frame_id: Option<EventId>,
    player: PlayerId,
    card: crate::ids::CardId,
) {
    let Some(slot) = ctx.catalog.card_slot_of(ctx.core, card) else {
        return;
    };
    // 同**栏位**替换（改造后同栏位不同牌种亦替换）；六龙骖驾占双坐骑栏（customSwap: equip3|equip4|equip6）
    let is_liulong = ctx
        .catalog
        .cards
        .get(card as usize)
        .is_some_and(|cd| cd.name == "txhj_liulongcanjia");
    let is_mount = matches!(
        slot,
        crate::catalog::EquipSlot::HorsePlus | crate::catalog::EquipSlot::HorseMinus
    );
    let has_txhj_equip5 = slot == crate::catalog::EquipSlot::Treasure
        && ctx
            .catalog
            .skill_by_name("_txhj_equip5")
            .is_some_and(|def| ctx.core.has_skill(player, def));
    let to_replace: Vec<crate::ids::CardId> = if has_txhj_equip5 {
        let existing_treasures: Vec<crate::ids::CardId> = ctx
            .core
            .player(player)
            .equips
            .iter()
            .copied()
            .filter(|&c| {
                ctx.catalog.card_slot_of(ctx.core, c) == Some(crate::catalog::EquipSlot::Treasure)
            })
            .collect();
        if existing_treasures.len() >= 2 {
            vec![existing_treasures[0]]
        } else {
            Vec::new()
        }
    } else {
        ctx.core
            .player(player)
            .equips
            .iter()
            .copied()
            .filter(|&c| {
                let c_slot = ctx.catalog.card_slot_of(ctx.core, c);
                if is_liulong {
                    matches!(
                        c_slot,
                        Some(
                            crate::catalog::EquipSlot::HorsePlus
                                | crate::catalog::EquipSlot::HorseMinus
                        )
                    )
                } else if is_mount {
                    c_slot == Some(slot)
                        || ctx
                            .catalog
                            .cards
                            .get(c as usize)
                            .is_some_and(|cd| cd.name == "txhj_liulongcanjia")
                } else {
                    c_slot == Some(slot)
                }
            })
            .collect()
    };
    let replaced = to_replace.first().copied();
    for &old in &to_replace {
        ctx.core.player_mut(player).equips.retain(|c| *c != old);
        revoke_equip_skills(ctx.core, ctx.catalog, player, old);
        ctx.note_cards_lost(player, &[old]);
        ctx.to_discard(old);
        (ctx.catalog.hooks.on_cards_to_discard)(ctx, player, &[old], false);
    }
    if !to_replace.is_empty() && ctx.catalog.vm_card_history {
        let origins = vec![crate::event::LostOrigin::Equip; to_replace.len()];
        ctx.emit(EventData::CardsLost {
            player,
            cards: to_replace,
            origins,
        });
    }
    ctx.core.player_mut(player).equips.push(card);
    grant_equip_skills(ctx, player, card);
    // 轩辕剑装备钩（swd 批 146，规则 v320）：体力 ≤2 且无「轩辕」→ 弃置；
    // 否则 +1 护甲（上游 onEquip 同口径）
    if ctx.catalog.equip_card_has_kind(ctx.core, card, crate::catalog::CardKind::Xuanyuanjian) {
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
    if (ctx.catalog.equip_card_links(ctx.core, card)
        || ctx.catalog.resolved_card_kind(ctx.core, card) == crate::catalog::CardKind::Suolianjia)
        && crate::rvm_bridge::modifiers::can_gain_link(ctx.core, ctx.catalog, player)
    {
        let was = ctx.core.player(player).linked;
        ctx.core.player_mut(player).linked = true;
        if !was {
            ctx.log(LogEvent::Linked {
                player,
                linked: true,
            });
        }
    }
    if let Some(f) = frame_id.and_then(|id| ctx.res.frame_mut(id)) {
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

/// Clears `yange` cloned-card definition/slot overrides when a cloned card leaves its active zone.
pub fn clear_yange_clone_card(
    core: &mut crate::state::GameStateCore,
    card: crate::ids::CardId,
) {
    if core
        .vm_aux
        .flags
        .remove(&(0, format!("gj_yange_clone:{card}")))
        .is_some()
    {
        core.declared_kinds.remove(&card);
        core.declared_slots.remove(&card);
    }
}

/// Releases any `yanjia`-merged secondary card (`gj_yanjia_merged:{card}`)
/// from `detached_cards` back into `core.discard` once `card` is discarded.
pub fn release_yanjia_merged_to_discard(
    core: &mut crate::state::GameStateCore,
    card: crate::ids::CardId,
) {
    if let Some(val) = core
        .vm_aux
        .flags
        .remove(&(0, format!("gj_yanjia_merged:{card}")))
    {
        if let Some(c2) = val.checked_sub(1) {
            if let Some(pos) = core.detached_cards.iter().position(|(dc, _)| *dc == c2) {
                core.detached_cards.remove(pos);
                if !core.discard.contains(&c2) {
                    core.discard.push(c2);
                }
            }
        }
    }
}

/// Clears transient `yange` clone overrides and unmerges `yanjia` secondary cards
/// before `washed` cards are shuffled back into `core.deck`.
pub fn cleanup_discard_to_deck(
    core: &mut crate::state::GameStateCore,
    washed: &mut Vec<crate::ids::CardId>,
) {
    let mut extra = Vec::new();
    for &card in washed.iter() {
        clear_yange_clone_card(core, card);
        core.declared_kinds.remove(&card);
        core.declared_slots.remove(&card);
        if let Some(val) = core
            .vm_aux
            .flags
            .remove(&(0, format!("gj_yanjia_merged:{card}")))
        {
            if let Some(c2) = val.checked_sub(1) {
                if let Some(pos) = core.detached_cards.iter().position(|(dc, _)| *dc == c2) {
                    core.detached_cards.remove(pos);
                    core.declared_kinds.remove(&c2);
                    core.declared_slots.remove(&c2);
                    if !washed.contains(&c2) && !extra.contains(&c2) {
                        extra.push(c2);
                    }
                }
            }
        }
    }
    washed.extend(extra);
}
