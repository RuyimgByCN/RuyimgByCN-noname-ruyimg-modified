use super::*;

/// 按 `card_mods` 中附着的灵剑与装备槽位（`equip1..=5`）推导锻造装备技键。
pub fn forged_lingjian_skill_name(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    player: PlayerId,
    card: crate::ids::CardId,
) -> Option<&'static str> {
    let attached = core.player(player).card_mods.get(&card).copied()?;
    let akind = catalog.card_kind(attached);
    let aname = catalog.cards.get(attached as usize)?.name;
    let slot_num = match catalog.card_slot_of(core, card)? {
        crate::catalog::EquipSlot::Weapon => 1,
        crate::catalog::EquipSlot::Armor => 2,
        crate::catalog::EquipSlot::HorsePlus => 3,
        crate::catalog::EquipSlot::HorseMinus => 4,
        crate::catalog::EquipSlot::Treasure => 5,
    };
    if akind.yuchan_auto_view_as().is_some() {
        let specific = format!("{aname}_equip{slot_num}");
        if let Some(id) = catalog.skill_by_name(&specific) {
            return Some(catalog.skills[id as usize].name);
        }
        return catalog
            .skill_by_name("g_yuchan_equip")
            .map(|id| catalog.skills[id as usize].name);
    }
    let base = [
        ("xuanfengzhiren", "hslingjian_xuanfengzhiren"),
        ("zhongxinghujia", "hslingjian_zhongxinghujia"),
        ("jinjilengdong", "hslingjian_jinjilengdong"),
        ("yinmilichang", "hslingjian_yinmilichang"),
        ("xingtigaizao", "hslingjian_xingtigaizao"),
        ("shengxiuhaojiao", "hslingjian_shengxiuhaojiao"),
        ("shijianhuisu", "hslingjian_shijianhuisu"),
        ("qinglongzhigui", "qinglongzhigui"),
        ("zhuquezhizhang", "zhuquezhizhang"),
        ("baishouzhihu", "baishouzhihu"),
        ("xuanwuzhihuang", "xuanwuzhihuang"),
        ("huanglinzhicong", "huanglinzhicong"),
        ("cangchizhibi", "cangchizhibi"),
    ]
    .iter()
    .find(|(pat, _)| aname.contains(pat))?
    .1;
    let full = format!("{base}_equip{slot_num}");
    let id = catalog.skill_by_name(&full)?;
    Some(catalog.skills[id as usize].name)
}

/// 装备提交：牌入手牌区到对应装备栏，同栏旧牌替换入弃牌堆；
/// 替换结果写回帧载荷（哈希可见）。
/// G1 装备技能生命周期：装备 `card` 时授予其声明的目录技能（含 `card_mods` 灵剑锻造技）。
pub fn grant_equip_skills(ctx: &mut Ctx<'_>, player: PlayerId, card: crate::ids::CardId) {
    let mut names = ctx.catalog.equip_card_skills(ctx.core, card);
    if let Some(forged) = forged_lingjian_skill_name(ctx.core, ctx.catalog, player, card) {
        if !names.contains(&forged) {
            names.push(forged);
        }
    }
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
/// 由玩家剩余的其他装备提供（跨槽同名技能共存），并记录 `onLose` 延迟钩。
pub fn revoke_equip_skills(
    core: &mut crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    player: PlayerId,
    card: crate::ids::CardId,
) {
    let mut names = catalog.equip_card_skills(core, card);
    if let Some(forged) = forged_lingjian_skill_name(core, catalog, player, card) {
        if !names.contains(&forged) {
            names.push(forged);
        }
    }
    for name in names {
        let Some(def) = catalog.skill_by_name(name) else {
            continue;
        };
        let still_provided = core.player(player).equips.iter().any(|c| {
            catalog.equip_card_skills(core, *c).contains(&name)
                || forged_lingjian_skill_name(core, catalog, player, *c) == Some(name)
        });
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
    // 乾坤袋 `onLose`（U05 swd 批 7a/7b 同源键名）：失去乾坤袋后摸 1。
    let has_qiankundai_skill = catalog.cards[card as usize]
        .equip_skills
        .iter()
        .any(|name| *name == "qiankundai");
    if (has_qiankundai_skill
        || catalog.card_kind(card) == crate::catalog::CardKind::Qiankundai)
        && core.player(player).alive
    {
        *core
            .vm_aux
            .flags
            .entry((player, "swd_qiankundai_pending_lose".to_string()))
            .or_insert(0) += 1;
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
    trigger_gw_dieyi_lost(core, catalog, player, card, false);
    // 清理锻造灵剑在离场装备上附加的伪装牌与声明槽位。
    core.player_mut(player).card_mods.remove(&card);
    core.declared_slots.remove(&card);
}

/// 蝶翼（U05 杀海拾遗·gwent `card/gwent/skill.js:253`）：`gw_dieyi_equip1..5`、
/// `gw_dieyi_judge` 或带 `gw_dieyi_card:{c}` 假标的牌离场时授予或递增 `gw_dieyi`。
pub fn trigger_gw_dieyi_lost(
    core: &mut crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    player: PlayerId,
    card: crate::ids::CardId,
    force_trigger: bool,
) {
    let cname = catalog.cards[card as usize].name;
    let masked = core
        .vm_aux
        .flags
        .remove(&(player, format!("gw_dieyi_card:{card}")))
        .is_some();
    if !force_trigger && !masked && !cname.starts_with("gw_dieyi_") {
        return;
    }
    if !core.player(player).alive {
        return;
    }
    let Some(def) = catalog.skill_by_name("gw_dieyi") else {
        return;
    };
    let key = (player, "gw_dieyi_storage".to_string());
    if core.has_skill(player, def) {
        let cur = core.vm_aux.flags.get(&key).copied().unwrap_or(1);
        core.vm_aux.flags.insert(key, cur.saturating_add(1));
    } else {
        core.skills.insert(
            (player, def),
            crate::state::SkillInstance {
                owner: player,
                enabled: true,
                once_used: false,
                expires_end_of_turn: false,
                rounds: 0,
            },
        );
        core.vm_aux.flags.insert(key, 1);
    }
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

/// Flushes pending `Qiankundai.onLose` draws immediately instead of waiting for outer settlement.
pub fn flush_qiankundai_pending_lose(ctx: &mut Ctx<'_>) {
    let seats = ctx.core.players.len() as PlayerId;
    for p in 0..seats {
        if let Some(n) = ctx
            .core
            .vm_aux
            .flags
            .remove(&(p, "swd_qiankundai_pending_lose".to_string()))
        {
            if n > 0 && ctx.core.player(p).alive {
                ctx.start_draw(p, n);
            }
        }
    }
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
        .is_some()
        || ctx.core.table.contains(&card);
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
