//! Hearthstone 扩展判定挂点（`fulei` 浮雷、`xianzhi` 判定前调序、`mdzhoufu2` 改判）。

use crate::catalog::Suit;
use crate::event::EventData;
use crate::ids::{CardId, PlayerId};
use crate::machine::{damage_data_with, Ctx};

pub(super) fn clear_xianzhi_asked(ctx: &mut Ctx<'_>, owner: PlayerId) {
    ctx.core
        .vm_aux
        .flags
        .remove(&(owner, "hs_xianzhi_asked".to_string()));
}

pub(super) fn is_fulei_card(catalog: &crate::catalog::Catalog, cid: CardId) -> bool {
    let n = catalog.cards[cid as usize].name;
    n == "fulei" || n.starts_with("fulei_")
}

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_fulei_judge(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    trick: CardId,
    judge_card: CardId,
    suit: Suit,
    shenyi_voids: bool,
    machine_holds: bool,
) {
    let next = ctx.core.next_alive(owner);
    if !shenyi_voids && suit.is_spade() {
        let key = format!("hs_fulei_hits:{trick}");
        let hits = ctx
            .core
            .vm_aux
            .flags
            .get(&(0, key.clone()))
            .copied()
            .unwrap_or(0)
            + 1;
        ctx.core.vm_aux.flags.insert((0, key), hits);
        if !machine_holds {
            ctx.to_discard(judge_card);
        }
        ctx.core.table.push(trick);
        ctx.core.vm_aux.flags.insert(
            (0, format!("hs_fulei_pending_move:{trick}")),
            u32::from(next) + 1,
        );
        ctx.emit(EventData::Damage(damage_data_with(
            None,
            owner,
            hits as i32,
            crate::event::DamageNature::Thunder,
            Some(trick),
        )));
        return;
    }
    let next_has_fulei = ctx
        .core
        .player(next)
        .judge
        .iter()
        .any(|c| is_fulei_card(ctx.catalog, *c));
    if next_has_fulei {
        ctx.core
            .vm_aux
            .flags
            .remove(&(0, format!("hs_fulei_hits:{trick}")));
        ctx.to_discard(trick);
    } else {
        ctx.core.player_mut(next).judge.push(trick);
    }
    if !machine_holds {
        ctx.to_discard(judge_card);
    }
}
