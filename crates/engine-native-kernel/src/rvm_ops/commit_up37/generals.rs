//! Batch 18 (`UP37-N02..N07`): authoritative host-side skill resolvers for the 6 new
//! generals (`sp:tangtang`, `collab:tw_dm_zhangliang`, `sp:ol_sunhanhua`, `collab:ol_chitu`,
//! `collab:ol_le_menghuo`, `sp:ol_xushao`).

use super::*;

/// `UP37-N02` (`sp:zexing` `2331836028`):
/// Show 1 card & choose unvisited other player: branch 1 view hand & gain up to 2
/// diff-suit cards + target gains shown card; branch 2 target loses 1 HP & may repeat.
pub(super) fn resolve_zexing(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let Some(shown) = ctx.core.player(owner).hand.first().copied() else {
        return;
    };
    let branch = flag(ctx.core, owner, "zexing_branch").max(1);
    let unvisited: Vec<PlayerId> = ctx
        .core
        .seat_order_from(owner)
        .into_iter()
        .filter(|&q| {
            q != owner
                && ctx.core.player(q).alive
                && turn_var(ctx.core, owner, &format!("zexing_visited:{q}")) == 0
        })
        .collect();
    let Some(&first_t) = unvisited.first() else {
        return;
    };
    set_turn_var(ctx.core, owner, &format!("zexing_visited:{first_t}"), 1);
    if branch == 1 {
        let mut gained_suits: Vec<Suit> = Vec::new();
        let mut to_gain: Vec<CardId> = Vec::new();
        for &c in &ctx.core.player(first_t).hand {
            let s = ctx.catalog.cards[c as usize].suit;
            if !gained_suits.contains(&s) {
                gained_suits.push(s);
                to_gain.push(c);
                if to_gain.len() == 2 {
                    break;
                }
            }
        }
        ctx.give_hand_cards(first_t, owner, &to_gain);
        if ctx.core.player(owner).hand.contains(&shown) {
            ctx.give_hand_card(owner, first_t, shown);
        }
        set_flag(ctx.core, owner, "zexing_branch1_gained", to_gain.len() as u32);
    } else {
        ctx.emit(EventData::LoseHp {
            player: first_t,
            amount: 1,
        });
        let mut visits = 1u32;
        if flag(ctx.core, owner, "zexing_repeat") != 0 {
            if let Some(&second_t) = unvisited.get(1) {
                set_turn_var(ctx.core, owner, &format!("zexing_visited:{second_t}"), 1);
                ctx.emit(EventData::LoseHp {
                    player: second_t,
                    amount: 1,
                });
                visits += 1;
            }
        }
        set_flag(ctx.core, owner, "zexing_branch2_visits", visits);
    }
}

/// `UP37-N02` (`sp:tt_zhiyi` `3644472252`):
/// Once per turn on card gain/loss, if hand has another card of same color, choose 1 player to draw 1.
pub(super) fn resolve_tt_zhiyi(ctx: &mut Ctx<'_>, owner: PlayerId) {
    if turn_var(ctx.core, owner, "tt_zhiyi_used") != 0 {
        return;
    }
    if let Some(frame) = ctx.res.stack.last() {
        match &frame.data {
            EventData::CardsLost { player, .. } | EventData::CardsDiscarded { player, .. } => {
                if *player != owner {
                    return;
                }
            }
            EventData::CardsMoved { from, to, .. } => {
                if *from != owner && *to != owner {
                    return;
                }
            }
            _ => {}
        }
    }
    if ctx.core.player(owner).hand.is_empty() {
        return;
    }
    set_turn_var(ctx.core, owner, "tt_zhiyi_used", 1);
    let target = (flag(ctx.core, owner, "tt_zhiyi_target") as PlayerId)
        .min(ctx.core.players.len().saturating_sub(1) as PlayerId);
    ctx.start_draw(target, 1);
}

/// `UP37-N03` (`collab:twrenfang` `3456624936`):
/// Locked mark accumulation on card entering discard (`+marks` hand limit, `+(marks/5)` slash limit).
pub(super) fn resolve_twrenfang(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let explicit_add = flag(ctx.core, owner, "twrenfang_add");
    let delta = if explicit_add != 0 || ctx.res.stack.is_empty() {
        set_flag(ctx.core, owner, "twrenfang_add", 0);
        explicit_add.max(1) as i32
    } else if let Some(frame) = ctx.res.stack.last() {
        match &frame.data {
            EventData::CardsDiscarded { cards, .. } => {
                cards
                    .iter()
                    .filter(|&&c| {
                        ctx.core.discard.contains(&c)
                            && !ctx.res.stack.iter().rev().skip(1).any(|f| {
                                matches!(
                                    &f.data,
                                    EventData::CardsLost { cards: lc, .. } if lc.contains(&c)
                                )
                            })
                    })
                    .count() as i32
            }
            EventData::CardsLost { cards, .. } => {
                cards
                    .iter()
                    .filter(|&&c| ctx.core.discard.contains(&c))
                    .count() as i32
            }
            _ => 0,
        }
    } else {
        0
    };
    if delta <= 0 {
        return;
    }
    let cur = ctx
        .core
        .vm_aux
        .marks
        .get(&(owner, TWRENFANG_ID))
        .copied()
        .unwrap_or(0);
    ctx.core
        .vm_aux
        .marks
        .insert((owner, TWRENFANG_ID), (cur + delta).min(99));
}

/// `UP37-N03` (`collab:twjuemie` `2465023183`):
/// Consume `twrenfang` marks for damage and draw.
pub(super) fn resolve_twjuemie(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let marks = ctx
        .core
        .vm_aux
        .marks
        .get(&(owner, TWRENFANG_ID))
        .copied()
        .unwrap_or(0);
    if marks <= 0 {
        return;
    }
    let consume = (flag(ctx.core, owner, "twjuemie_consume").max(1) as i32).min(marks);
    let next = marks - consume;
    if next <= 0 {
        ctx.core.vm_aux.marks.remove(&(owner, TWRENFANG_ID));
    } else {
        ctx.core.vm_aux.marks.insert((owner, TWRENFANG_ID), next);
    }
    if let Some(target) = first_other_alive(ctx.core, owner) {
        ctx.emit(EventData::Damage(crate::machine::damage_data(
            Some(owner),
            target,
            1,
        )));
    }
    ctx.start_draw(owner, 1);
}

/// `UP37-N03` (`tw:twjianming` `2495679541`):
pub(super) fn resolve_twjianming(ctx: &mut Ctx<'_>, owner: PlayerId) {
    set_flag(ctx.core, owner, "twjianming_invoked", 1);
    ctx.start_draw(owner, 1);
}

/// `UP37-N03` (`extra:relonghun` `3894575372`):
pub(super) fn resolve_relonghun(ctx: &mut Ctx<'_>, owner: PlayerId) {
    set_flag(ctx.core, owner, "relonghun_invoked", 1);
    if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
        ctx.recover(owner, 1, None);
    } else if let Some(target) = first_other_alive(ctx.core, owner) {
        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
            Some(owner),
            target,
            1,
            DamageNature::Fire,
            None,
        )));
    }
}

/// `UP37-N04` (`sp:olhuaguang` `3010477673`):
/// `RoundStart`/`PhasePrepare` place `cailian` into weapon (`1`), armor (`2`), or
/// treasure (`5`) slot or change suit (`1..=4`).
pub(super) fn resolve_olhuaguang(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let slot = match flag(ctx.core, owner, "olhuaguang_target_slot") {
        2 => 2,
        5 => 5,
        _ => 1,
    };
    let suit = flag(ctx.core, owner, "olhuaguang_target_suit").clamp(1, 4);
    set_flag(ctx.core, owner, "cailian_slot", slot);
    set_flag(ctx.core, owner, "cailian_suit", suit);
    resolve_olxuanbai(ctx, owner);
}

/// `UP37-N04` (`sp:olxuanbai` `3762297239`):
/// Switch `dangmo` (`1`), `jihui` (`2`), `xiaju` (`3`) form by `cailian` slot.
pub(super) fn resolve_olxuanbai(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let slot = flag(ctx.core, owner, "cailian_slot").max(1);
    let form_id = match slot {
        1 => 1, // dangmo
        2 => 2, // jihui
        _ => 3, // xiaju
    };
    set_flag(ctx.core, owner, "olxuanbai_form", form_id);
    if form_id == 3 && ctx.core.turn.phase == Phase::Play {
        ctx.start_draw(owner, 1);
    }
}

/// `UP37-N05` (`collab:olfangma` `3715058816`):
/// `RoundStart` place `chitu` at left of hand; `left > right` unlimited `sha`,
/// `left < right` `sha` needs 2 `shan`.
pub(super) fn resolve_olfangma(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let idx = flag(ctx.core, owner, "olfangma_chitu_index");
    let hand_len = ctx.core.player(owner).hand.len() as u32;
    let left = idx.min(hand_len);
    let right = hand_len.saturating_sub(left);
    set_flag(ctx.core, owner, "olfangma_active", 1);
    if left > right {
        set_flag(ctx.core, owner, "olfangma_sha_unlimited", 1);
        set_flag(ctx.core, owner, "olfangma_sha_double_shan", 0);
    } else if left < right {
        set_flag(ctx.core, owner, "olfangma_sha_unlimited", 0);
        set_flag(ctx.core, owner, "olfangma_sha_double_shan", 1);
    } else {
        set_flag(ctx.core, owner, "olfangma_sha_unlimited", 0);
        set_flag(ctx.core, owner, "olfangma_sha_double_shan", 0);
    }
}

/// `UP37-N05` (`collab:oljingxian` `2801773264`):
/// Move `chitu` right 1 index on card use, overtaken card is `sha`, `RoundEnd` at
/// rightmost grants extra turn.
pub(super) fn resolve_oljingxian(ctx: &mut Ctx<'_>, owner: PlayerId) {
    if let Some(frame) = ctx.res.stack.last() {
        match &frame.data {
            EventData::CardUsed(u) if u.player != owner => return,
            EventData::Phase { player, .. } if *player != owner => return,
            _ => {}
        }
    }
    let hand_len = ctx.core.player(owner).hand.len();
    let idx = flag(ctx.core, owner, "olfangma_chitu_index") as usize;
    if idx < hand_len {
        let overtaken = ctx.core.player(owner).hand[idx];
        ctx.core.declared_kinds.insert(overtaken, CardKind::Sha);
        set_flag(ctx.core, owner, "oljingxian_overtaken_sha", overtaken);
        let next_idx = (idx + 1).min(hand_len);
        set_flag(ctx.core, owner, "olfangma_chitu_index", next_idx as u32);
    }
    let new_idx = flag(ctx.core, owner, "olfangma_chitu_index") as usize;
    if new_idx >= hand_len {
        set_flag(ctx.core, owner, "oljingxian_at_rightmost", 1);
        if ctx.core.turn.phase == Phase::End || flag(ctx.core, owner, "oljingxian_round_end") != 0
        {
            ctx.core.turn.extra_phases.push(2);
            set_flag(ctx.core, owner, "oljingxian_extra_turn_granted", 1);
        }
    }
    resolve_olfangma(ctx, owner);
}

/// `UP37-N06` (`collab:oljunzhu` `2795607434`):
/// Deterministic `PhasePrepare` mushroom roll (`1..=3`).
pub(super) fn resolve_oljunzhu(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let override_roll = flag(ctx.core, owner, "oljunzhu_roll_override");
    let mushroom = if (1..=3).contains(&override_roll) {
        override_roll
    } else {
        ((ctx.core.turn.round + u32::from(owner)) % 3) + 1
    };
    set_flag(ctx.core, owner, "oljunzhu_mushroom", mushroom);
    match mushroom {
        1 => ctx.start_draw(owner, 2),
        2 => {
            ctx.recover(owner, 1, None);
        }
        _ => {
            if let Some(target) = first_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    target,
                    1,
                )));
            }
        }
    }
}

/// `UP37-N06` (`collab:olrehuanshi` `3274133709`):
/// Once per suit per turn view-as hand card as same-suit basic/normal-trick in hand.
pub(super) fn resolve_olrehuanshi(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let Some(&card) = ctx.core.player(owner).hand.first() else {
        return;
    };
    let suit = ctx.catalog.cards[card as usize].suit;
    let suit_bit = match suit {
        Suit::Spade => 1u32,
        Suit::Heart => 2u32,
        Suit::Club => 4u32,
        Suit::Diamond => 8u32,
    };
    let mask = turn_var(ctx.core, owner, "olrehuanshi_suit_mask");
    if (mask & suit_bit) != 0 {
        set_flag(ctx.core, owner, "olrehuanshi_rejected_duplicate_suit", 1);
        return;
    }
    set_turn_var(ctx.core, owner, "olrehuanshi_suit_mask", mask | suit_bit);
    let cnt = flag(ctx.core, owner, "olrehuanshi_used_count") + 1;
    set_flag(ctx.core, owner, "olrehuanshi_used_count", cnt);
    ctx.start_draw(owner, 1);
}

/// `UP37-N07` (`sp:olshilun` `4097066031`):
/// Deterministic `pingjian` pool skill acquisition + invoke & revoke at `PhasePlay`/`PhaseEnd`/`DamageTaken`.
pub(super) fn resolve_olshilun(ctx: &mut Ctx<'_>, owner: PlayerId) {
    const PINGJIAN_POOL: [u32; 3] = [4_097_066_032, 4_097_066_033, 4_097_066_034];
    let idx = ((ctx.core.turn.round as usize) + (owner as usize)) % PINGJIAN_POOL.len();
    let pool_sk = {
        let ov = flag(ctx.core, owner, "olshilun_pool_skill");
        if ov != 0 {
            ov
        } else {
            PINGJIAN_POOL[idx]
        }
    };
    set_flag(ctx.core, owner, "pingjian_skill", pool_sk);
    set_flag(ctx.core, owner, "olshilun_acquired_skill", pool_sk);
    ctx.start_draw(owner, 1);
    if flag(ctx.core, owner, "olshilun_revoke_after_use") != 0 {
        set_flag(ctx.core, owner, "pingjian_skill", 0);
        set_flag(ctx.core, owner, "olshilun_revoked_skill", pool_sk);
        notify_pingjian_skill_lost(ctx, owner);
    }
}

fn notify_pingjian_skill_lost(ctx: &mut Ctx<'_>, lost_player: PlayerId) {
    let _ = lost_player;
    let holders: Vec<PlayerId> = ctx
        .core
        .alive_players()
        .filter(|&p| {
            ctx.catalog
                .skill_by_name("olzhuoming")
                .is_some_and(|def| ctx.core.has_skill(p, def))
                || flag(ctx.core, p, "has_olzhuoming") != 0
        })
        .collect();
    for h in holders {
        let cnt = flag(ctx.core, h, "olzhuoming_draw_on_lose") + 1;
        set_flag(ctx.core, h, "olzhuoming_draw_on_lose", cnt);
        ctx.start_draw(h, 1);
    }
}

/// `UP37-N07` (`sp:olzhuoming` `3643715420`):
/// First damage dealt/taken per turn transfer 1 `pingjian` skill to another player;
/// draw 1 whenever any player loses a `pingjian` skill.
pub(super) fn resolve_olzhuoming(ctx: &mut Ctx<'_>, owner: PlayerId) {
    if turn_var(ctx.core, owner, "olzhuoming_transferred") != 0 {
        return;
    }
    let sk = flag(ctx.core, owner, "pingjian_skill");
    if sk == 0 {
        return;
    }
    let Some(target) = first_other_alive(ctx.core, owner) else {
        return;
    };
    set_turn_var(ctx.core, owner, "olzhuoming_transferred", 1);
    set_flag(ctx.core, owner, "pingjian_skill", 0);
    set_flag(ctx.core, target, "pingjian_skill", sk);
    notify_pingjian_skill_lost(ctx, owner);
}
