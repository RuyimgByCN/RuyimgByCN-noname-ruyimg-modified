//! VM 历史追踪与回复后触发辅助（自 `kernel/events.rs` 拆分，行为零变化）。

use super::*;

/// 造异异变 def 查询（运行时授予技）。
pub(in crate::kernel) fn zhp_def(ctx: &Ctx<'_>) -> crate::ids::SkillDefId {
    ctx.catalog
        .skill_by_name(crate::vocabulary::names::BOSS_ZAOYI_HP)
        .expect("造异异变入目录")
}

fn track_l11_loss_dedup(catalog: &crate::catalog::Catalog) -> bool {
    catalog.skill_by_name("olshanjia").is_some()
        || catalog.skill_by_name("boss_moyany").is_some()
        || catalog.skill_by_name("boss_danshu").is_some()
        || catalog.skill_by_name("twyuhua").is_some()
        || catalog.skill_by_name("twxiangyu").is_some()
        || catalog.skill_by_name("meiyingqiang").is_some()
}

fn loss_handled_by_cards_lost(
    core: &crate::state::GameStateCore,
    stack: &[crate::event::Frame],
    frame_id: crate::ids::EventId,
    player: PlayerId,
    card: crate::ids::CardId,
) -> bool {
    core.vm_aux
        .flags
        .contains_key(&(player, format!("l11_loss_dedup:{frame_id}:{card}")))
        || stack.iter().any(|f| {
            f.id != frame_id
                && f.stage == crate::event::Stage::Commit
                && matches!(
                    &f.data,
                    EventData::CardsLost { player: p, cards, .. }
                        if *p == player && cards.contains(&card)
                )
        })
}

fn mark_sibling_discard_or_move_dedup(
    core: &mut crate::state::GameStateCore,
    stack: &[crate::event::Frame],
    frame_id: crate::ids::EventId,
    player: PlayerId,
    card: crate::ids::CardId,
) {
    for f in stack {
        if f.id == frame_id || f.stage != crate::event::Stage::Commit {
            continue;
        }
        let matches_sib = match &f.data {
            EventData::CardsDiscarded { player: dp, cards } => {
                *dp == player && cards.contains(&card)
            }
            EventData::CardsMoved { from, to, cards } => {
                from != to && *from == player && cards.contains(&card)
            }
            _ => false,
        };
        if matches_sib {
            core.vm_aux
                .flags
                .insert((player, format!("l11_loss_dedup:{}:{card}", f.id)), 1);
        }
    }
}

/// 回合收束：到期技能移除，然后交给下一位存活玩家并开启判定阶段。
pub(in crate::kernel) fn record_vm_history(
    ctx: &mut Ctx<'_>,
    frame_id: crate::ids::EventId,
    data: &EventData,
) {
    let pre_counted = matches!(
        data,
        EventData::CardUsed(u)
            if ctx
                .core
                .vm_aux
                .flags
                .contains_key(&(u.player, "monkey_pre_counted".to_string()))
    );
    // 暗织读全局末牌历史，即使当时未持有/被封禁；不依赖 VM 牌史开关。
    if !pre_counted {
        let card = match data {
            EventData::CardUsed(u) if !u.cancelled => Some(u.card),
            EventData::CardResponded { card, .. } => Some(Some(*card)),
            _ => None,
        };
        if let Some(card) = card {
            record_dddanzhi_card(ctx.core, ctx.catalog, card);
        }
    }
    if !pre_counted {
        if let EventData::CardUsed(u) = data {
            if !u.cancelled {
                let eff = u.effective_kind_in(ctx.core, ctx.catalog);
                let is_trick = !eff.is_basic_kind() && eff.equip_slot().is_none();
                let stamp = ctx.core.turn.round.saturating_mul(100)
                    + u32::from(ctx.core.turn.current)
                    + 1;
                if is_trick && ctx.catalog.skill_by_name("dddshixing").is_some() {
                    ctx.core
                        .vm_aux
                        .flags
                        .insert((0, "u01_turn_trick_used_stamp".to_string()), stamp);
                }
                if ctx.catalog.skill_by_name("dddzhuanshe").is_some() {
                    let key_kind = if eff.is_sha_like() {
                        crate::catalog::CardKind::Sha as u32
                    } else {
                        eff as u32
                    };
                    ctx.core.vm_aux.flags.insert(
                        (
                            u.player,
                            format!("u01_zhuanshe_used:{stamp}:{key_kind}"),
                        ),
                        1,
                    );
                }
                let is_delay = matches!(
                    eff,
                    CardKind::Le
                        | CardKind::Bing
                        | CardKind::ShanDian
                        | CardKind::HongShui
                        | CardKind::Guiyoujie
                );
                if (eff.is_basic_kind() || (is_trick && !is_delay))
                    && ctx.catalog.skill_by_name("dddchengjing").is_some()
                {
                    if let Some(cid) = u.card.filter(|_| u.use_as.is_none()) {
                        let len_key = (u.player, format!("u01_cj_len:{stamp}"));
                        let idx = ctx.core.vm_aux.flags.get(&len_key).copied().unwrap_or(0);
                        ctx.core.vm_aux.flags.insert(
                            (u.player, format!("u01_cj_card:{stamp}:{idx}")),
                            cid,
                        );
                        ctx.core.vm_aux.flags.insert(len_key, idx + 1);
                    }
                }
            }
        }
    }
    // 标准局面没有 VM 技能时不记。空 VmAux 不写摘要，v295 golden 保持。
    if !ctx.catalog.vm_card_history {
        return;
    }
    let track_olshanjia = ctx.catalog.skill_by_name("olshanjia").is_some();
    let track_dedup = track_l11_loss_dedup(ctx.catalog);
    match data {
        EventData::CardsDiscarded { player, cards } => {
            {
                let entry = ctx.core.vm_aux.lost.entry(*player).or_default();
                for &c in cards {
                    if !entry.contains(&c) {
                        entry.push(c);
                    }
                }
            }
            for &c in cards {
                if !ctx.core.player(*player).hand.contains(&c) {
                    ctx.core
                        .vm_aux
                        .flags
                        .remove(&(*player, format!("l11_gaintag_paiku:{c}")));
                }
                if track_olshanjia
                    && ctx.catalog.card_kind(c).equip_slot().is_some()
                    && !loss_handled_by_cards_lost(ctx.core, &ctx.res.stack, frame_id, *player, c)
                {
                    *ctx.core
                        .vm_aux
                        .flags
                        .entry((*player, "l11_olshanjia_lost_equips".to_string()))
                        .or_insert(0) += 1;
                }
            }
        }
        EventData::CardsLost {
            player,
            cards,
            origins,
        } => {
            for (&c, &orig) in cards.iter().zip(origins.iter()) {
                if !ctx.core.player(*player).hand.contains(&c) {
                    ctx.core
                        .vm_aux
                        .flags
                        .remove(&(*player, format!("l11_gaintag_paiku:{c}")));
                }
                if track_olshanjia && ctx.catalog.card_kind(c).equip_slot().is_some() {
                    *ctx.core
                        .vm_aux
                        .flags
                        .entry((*player, "l11_olshanjia_lost_equips".to_string()))
                        .or_insert(0) += 1;
                }
                if track_dedup {
                    mark_sibling_discard_or_move_dedup(
                        ctx.core,
                        &ctx.res.stack,
                        frame_id,
                        *player,
                        c,
                    );
                }
                if matches!(orig, crate::event::LostOrigin::Judge) {
                    let entry = ctx.core.vm_aux.lost.entry(*player).or_default();
                    if !entry.contains(&c) {
                        entry.push(c);
                    }
                    *ctx.core
                        .vm_aux
                        .flags
                        .entry((*player, "cards_lost".to_string()))
                        .or_insert(0) += 1;
                    *ctx.core
                        .vm_aux
                        .flags
                        .entry((*player, "judge_cards_lost".to_string()))
                        .or_insert(0) += 1;
                }
            }
        }
        EventData::CardsMoved { cards, from, to } if from != to => {
            for &c in cards {
                if !ctx.core.player(*from).hand.contains(&c) {
                    ctx.core
                        .vm_aux
                        .flags
                        .remove(&(*from, format!("l11_gaintag_paiku:{c}")));
                }
                if track_olshanjia
                    && ctx.catalog.card_kind(c).equip_slot().is_some()
                    && !loss_handled_by_cards_lost(ctx.core, &ctx.res.stack, frame_id, *from, c)
                {
                    *ctx.core
                        .vm_aux
                        .flags
                        .entry((*from, "l11_olshanjia_lost_equips".to_string()))
                        .or_insert(0) += 1;
                }
            }
            ctx.core
                .vm_aux
                .lost
                .entry(*from)
                .or_default()
                .extend(cards.iter().copied());
        }
        EventData::CardsDrawn { player, cards } => {
            *ctx.core.vm_aux.drawn.entry(*player).or_default() += cards.len() as u32;
        }
        EventData::CardUsed(u) => {
            if let Some(c) = u.card {
                if !ctx.core.player(u.player).hand.contains(&c) {
                    ctx.core
                        .vm_aux
                        .flags
                        .remove(&(u.player, format!("l11_gaintag_paiku:{c}")));
                }
            }
            let effective = u.effective_kind_in(ctx.core, ctx.catalog);
            if effective.is_sha_like()
                && ctx
                    .catalog
                    .skill_by_name("potcuifeng")
                    .is_some_and(|def| ctx.core.has_skill(u.player, def))
            {
                *ctx.core
                    .vm_aux
                    .flags
                    .entry((u.player, "sha_played_this_turn".to_string()))
                    .or_insert(0) += 1;
            }
            if !pre_counted
                && ctx.core.turn.phase == Phase::Play
                && u.player == ctx.core.turn.current
            {
                let bit = if effective.is_basic_kind() {
                    1u32
                } else if effective.equip_slot().is_some() {
                    4u32
                } else if effective.is_basic_trick_or_equip() {
                    2u32
                } else {
                    0u32
                };
                if bit != 0 {
                    *ctx.core
                        .vm_aux
                        .flags
                        .entry((u.player, "play_cat_mask".to_string()))
                        .or_insert(0) |= bit;
                }
            }
        }
        EventData::Equipped { player, card, .. } => {
            if !ctx.core.player(*player).hand.contains(card) {
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(*player, format!("l11_gaintag_paiku:{card}")));
            }
            if ctx.core.turn.phase == Phase::Play && *player == ctx.core.turn.current {
                *ctx.core
                    .vm_aux
                    .flags
                    .entry((*player, "play_cat_mask".to_string()))
                    .or_insert(0) |= 4;
            }
        }
        EventData::CardResponded { card, player } => {
            if !ctx.core.player(*player).hand.contains(card) {
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(*player, format!("l11_gaintag_paiku:{card}")));
            }
        }
        _ => {}
    }
}

/// 实际开始的使用/打出；装备 PlayCard 在其前置触发之前调用。
pub fn record_dddanzhi_card(
    core: &mut crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    card: Option<crate::ids::CardId>,
) {
    if catalog.skill_by_name("dddanzhi").is_none() {
        return;
    }
    let black = card.is_some_and(|c| {
        matches!(
            catalog.cards[c as usize].suit,
            crate::catalog::Suit::Spade | crate::catalog::Suit::Club
        )
    });
    for owner in 0..core.players.len() as PlayerId {
        core.vm_aux
            .flags
            .insert((owner, "dddanzhi_last_black".to_string()), u32::from(black));
    }
}

/// 积财（Boss 饕餮，规则 v216）：回复结算后——回复者==持有者 → 持有
/// 者摸 2；否则持有者与回复者各摸 1（上游 boss_jicai recoverAfter 同
/// 口径；仅实际回复量>0 的 Recover 事件进入）。
pub(in crate::kernel) fn commit_recover_skills(ctx: &mut Ctx<'_>, recovered: PlayerId) {
    if ctx.catalog.vm_card_history {
        ctx.core
            .vm_aux
            .flags
            .insert((recovered, "turn_change_hp".to_string()), 1);
    }
    // 忍性（年兽形态，规则 v249）：global recoverEnd——持有者非当前回
    // 合角色时，任意角色回复后其摸 1（上游 boss_renxing recoverEnd 同口
    // 径；damageEnd 半面见 commit_damage 忍性臂）
    if let Some(rxdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_RENXING)
    {
        let holders: Vec<PlayerId> = ctx
            .core
            .skills
            .iter()
            .filter(|((_, dd), i)| *dd == rxdef && i.enabled)
            .map(|((o, _), _)| *o)
            .filter(|o| ctx.core.player(*o).alive && ctx.core.turn.current != *o)
            .collect();
        for holder in holders {
            ctx.start_draw(holder, 1);
        }
    }
    let Some(jdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_JICAI)
    else {
        return;
    };
    let holders: Vec<PlayerId> = ctx
        .core
        .skills
        .iter()
        .filter(|((_, dd), i)| *dd == jdef && i.enabled)
        .map(|((o, _), _)| *o)
        .filter(|o| ctx.core.player(*o).alive)
        .collect();
    for holder in holders {
        if recovered == holder {
            ctx.start_draw(holder, 2);
        } else {
            ctx.start_draw(holder, 1);
            if ctx.core.player(recovered).alive {
                ctx.start_draw(recovered, 1);
            }
        }
    }
    // 伤逝（规则 v260）：体力变化（回复）后补牌
    shangshix_draw(ctx, recovered);
}

/// 记录当前回合玩家实际使用的牌花色/点数/类别掩码（`txmouduan`、`txhj_ditingSkill3`）。
pub(in crate::kernel) fn record_l11_turn_card_used(
    core: &mut crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    player: PlayerId,
    card: Option<crate::ids::CardId>,
    suit: Option<crate::catalog::Suit>,
    effective: CardKind,
) {
    if player != core.turn.current {
        return;
    }
    if effective.equip_slot().is_some() {
        if let Some(s) = suit {
            let color_bit = if matches!(s, crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond) {
                1u32
            } else {
                2u32
            };
            if core.turn.phase == crate::event::Phase::Play
                && catalog.skill_by_name("txkeji").is_some()
            {
                *core
                    .vm_aux
                    .flags
                    .entry((player, "skill_turn:l11_play_color_mask".to_string()))
                    .or_insert(0) |= color_bit;
            }
            if catalog.skill_by_name("txchengyuan").is_some() {
                *core
                    .vm_aux
                    .flags
                    .entry((player, "skill_turn:l11_chengyuan_used".to_string()))
                    .or_insert(0) |= color_bit;
            }
        }
    }
    if catalog.skill_by_name("txmouduan").is_none()
        && catalog.skill_by_name("txhj_mouduan").is_none()
        && catalog.skill_by_name("txhj_ditingSkill3").is_none()
    {
        return;
    }
    if let Some(s) = suit {
        let sbit = 1u32
            << match s {
                crate::catalog::Suit::Spade => 0,
                crate::catalog::Suit::Heart => 1,
                crate::catalog::Suit::Club => 2,
                crate::catalog::Suit::Diamond => 3,
            };
        *core
            .vm_aux
            .flags
            .entry((player, "l11_turn_used_suits_mask".to_string()))
            .or_insert(0) |= sbit;
    }
    if let Some(c) = card {
        let rank = catalog.cards.get(c as usize).map_or(0, |cd| cd.rank);
        if (1..=13).contains(&rank) {
            *core
                .vm_aux
                .flags
                .entry((player, "l11_turn_used_ranks_mask".to_string()))
                .or_insert(0) |= 1u32 << (rank - 1);
        }
    }
    let tbit = if effective.is_basic_kind() {
        1
    } else if effective.equip_slot().is_some() {
        4
    } else if matches!(
        effective,
        CardKind::Le
            | CardKind::Bing
            | CardKind::ShanDian
            | CardKind::HongShui
            | CardKind::Guiyoujie
    ) {
        8
    } else {
        2
    };
    *core
        .vm_aux
        .flags
        .entry((player, "l11_turn_used_types_mask".to_string()))
        .or_insert(0) |= tbit;
}
