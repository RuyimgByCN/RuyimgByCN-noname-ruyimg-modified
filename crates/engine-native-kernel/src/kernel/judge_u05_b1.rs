//! U05-1 判定区延时锦囊扩展结算：`hongshui`（洪水）与 `huoshan`（火山）以及未命中/抵消传下家。

use super::*;

pub(crate) fn is_huoshan_card(catalog: &crate::catalog::Catalog, card: crate::ids::CardId) -> bool {
    let name = catalog.cards[card as usize].name;
    catalog.card_kind(card) == CardKind::Le
        && (name == "huoshan" || name.starts_with("yc_huoshan"))
}

pub(crate) fn is_hongshui_or_huoshan(
    catalog: &crate::catalog::Catalog,
    card: crate::ids::CardId,
) -> bool {
    catalog.card_kind(card) == CardKind::HongShui || is_huoshan_card(catalog, card)
}

/// 将【洪水】/【火山】传递给下家（或其后首个判定区不含同名延时锦囊者）；无人可接则弃置。
/// 对应上游 `card/yunchou/card.js:481-485, 551-555` 的 `addJudgeNext(card)`。
pub(crate) fn pass_delayed_to_next_or_discard(
    ctx: &mut Ctx<'_>,
    from: PlayerId,
    trick: crate::ids::CardId,
) {
    let seats = ctx.core.players.len();
    let huoshan = is_huoshan_card(ctx.catalog, trick);
    let mut cur = ctx.core.next_alive(from);
    for _ in 0..seats {
        if !ctx.core.player(cur).alive {
            break;
        }
        let already = ctx.core.player(cur).judge.iter().any(|c| {
            if huoshan {
                is_huoshan_card(ctx.catalog, *c)
            } else {
                ctx.catalog.card_kind(*c) == CardKind::HongShui
            }
        });
        if !already {
            ctx.core.table.retain(|c| *c != trick);
            ctx.core.player_mut(cur).judge.push(trick);
            if cur != from {
                crate::kernel::trigger_qiaoqiao_add_judge(ctx, cur);
            }
            return;
        }
        let next = ctx.core.next_alive(cur);
        if next == cur {
            break;
        }
        cur = next;
    }
    ctx.core.table.retain(|c| *c != trick);
    ctx.to_discard(trick);
}

fn discard_random_he_or_lose_hp(ctx: &mut Ctx<'_>, target: PlayerId, count: usize) {
    if !ctx.core.player(target).alive {
        return;
    }
    let total_he = ctx.core.player(target).hand.len() + ctx.core.player(target).equips.len();
    if total_he == 0 {
        ctx.emit(EventData::LoseHp {
            player: target,
            amount: 1,
        });
        return;
    }
    let n = count.min(total_he);
    for _ in 0..n {
        let hlen = ctx.core.player(target).hand.len();
        let elen = ctx.core.player(target).equips.len();
        let sum = hlen + elen;
        if sum == 0 {
            break;
        }
        let pick = ctx.core.rng.gen_range(sum as u32) as usize;
        if pick < hlen {
            let c = ctx.core.player(target).hand[pick];
            ctx.discard_from_hand(target, c);
            ctx.log(LogEvent::CardDiscarded {
                player: target,
                cards: vec![c],
            });
        } else {
            let c = ctx.core.player_mut(target).equips.remove(pick - hlen);
            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, target, c);
            ctx.to_discard(c);
            ctx.log(LogEvent::CardDiscarded {
                player: target,
                cards: vec![c],
            });
        }
    }
}

pub(in crate::kernel) fn resolve_hongshui_judge(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    trick: crate::ids::CardId,
    judge_card: crate::ids::CardId,
    machine_holds: bool,
    shenyi_voids: bool,
    suit: crate::catalog::Suit,
    rank: u8,
) {
    if !machine_holds {
        ctx.to_discard(judge_card);
    }
    // `card/yunchou/card.js:518-553`: 梅花 2–9 命中（弃随机至多 3 张或失 1 血；距离<=2 其他角色弃 3-max(1,dist) 或失 1 血）
    if !shenyi_voids && suit.is_club() && (2..=9).contains(&rank) {
        ctx.to_discard(trick);
        (ctx.catalog.hooks.on_cards_to_discard)(ctx, owner, &[trick], false);
        discard_random_he_or_lose_hp(ctx, owner, 3);
        let order: Vec<PlayerId> = ctx
            .core
            .seat_order_from(owner)
            .into_iter()
            .filter(|p| *p != owner && ctx.core.player(*p).alive)
            .collect();
        for p in order {
            let dist = ctx.core.computed_distance(ctx.catalog, owner, p);
            if dist <= 2 {
                let drop = 3usize.saturating_sub(dist.max(1) as usize);
                if drop > 0 {
                    discard_random_he_or_lose_hp(ctx, p, drop);
                }
            }
        }
    } else {
        pass_delayed_to_next_or_discard(ctx, owner, trick);
    }
}

pub(in crate::kernel) fn resolve_huoshan_judge(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    trick: crate::ids::CardId,
    judge_card: crate::ids::CardId,
    machine_holds: bool,
    shenyi_voids: bool,
    suit: crate::catalog::Suit,
    rank: u8,
) {
    if !machine_holds {
        ctx.to_discard(judge_card);
    }
    // `card/yunchou/card.js:458-482`: 红桃 2–9 命中（自己 2 点无源火焰伤害 + 距离<=1 其他角色各 1 点无源火焰伤害）
    if !shenyi_voids && suit.is_heart() && (2..=9).contains(&rank) {
        ctx.to_discard(trick);
        (ctx.catalog.hooks.on_cards_to_discard)(ctx, owner, &[trick], false);
        let splash: Vec<PlayerId> = ctx
            .core
            .seat_order_from(owner)
            .into_iter()
            .filter(|p| {
                *p != owner
                    && ctx.core.player(*p).alive
                    && ctx.core.computed_distance(ctx.catalog, owner, *p) <= 1
            })
            .collect();
        for p in splash.into_iter().rev() {
            ctx.emit(EventData::Damage(damage_data_with(
                None,
                p,
                1,
                crate::event::DamageNature::Fire,
                Some(trick),
            )));
        }
        ctx.emit(EventData::Damage(damage_data_with(
            None,
            owner,
            2,
            crate::event::DamageNature::Fire,
            Some(trick),
        )));
    } else {
        pass_delayed_to_next_or_discard(ctx, owner, trick);
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_le_bing_hongshui(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    trick: crate::ids::CardId,
    trick_kind: CardKind,
    judge_card: crate::ids::CardId,
    machine_holds: bool,
    shenyi_voids: bool,
    suit: crate::catalog::Suit,
    rank: u8,
) {
    match trick_kind {
        CardKind::Le if is_huoshan_card(ctx.catalog, trick) => {
            resolve_huoshan_judge(ctx, owner, trick, judge_card, machine_holds, shenyi_voids, suit, rank);
        }
        CardKind::Le => {
            if !shenyi_voids && !suit.is_heart() {
                ctx.core.turn.skip_play_phase = true;
            }
            ctx.to_discard(trick);
            (ctx.catalog.hooks.on_cards_to_discard)(ctx, owner, &[trick], false);
            if !machine_holds {
                ctx.to_discard(judge_card);
            }
        }
        CardKind::Bing => {
            if !shenyi_voids && !suit.is_club() {
                ctx.core.turn.skip_draw_phase = true;
            }
            ctx.to_discard(trick);
            (ctx.catalog.hooks.on_cards_to_discard)(ctx, owner, &[trick], false);
            if !machine_holds {
                ctx.to_discard(judge_card);
            }
        }
        CardKind::HongShui => {
            resolve_hongshui_judge(ctx, owner, trick, judge_card, machine_holds, shenyi_voids, suit, rank);
        }
        _ => {}
    }
}
