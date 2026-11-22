use super::*;

/// 记录规则 VM 历史（包含 `dddanzhi` 末牌追踪，不依赖 VM 牌史开关）。
pub(in crate::kernel) fn record_vm_history(ctx: &mut Ctx<'_>, data: &EventData) {
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
    // 标准局面没有 VM 技能时不记。空 VmAux 不写摘要，v295 golden 保持。
    if !ctx.catalog.vm_card_history {
        return;
    }
    match data {
        EventData::CardsDiscarded { player, cards } => {
            let entry = ctx.core.vm_aux.lost.entry(*player).or_default();
            for &c in cards {
                if !entry.contains(&c) {
                    entry.push(c);
                }
            }
        }
        EventData::CardsLost {
            player,
            cards,
            origins,
        } if origins.contains(&crate::event::LostOrigin::Judge) => {
            for (&c, &orig) in cards.iter().zip(origins.iter()) {
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
            let effective = u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog));
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
        EventData::Equipped { player, .. }
            if ctx.core.turn.phase == Phase::Play && *player == ctx.core.turn.current =>
        {
            *ctx.core
                .vm_aux
                .flags
                .entry((*player, "play_cat_mask".to_string()))
                .or_insert(0) |= 4;
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
