//! Jiding's emitted card keeps source-specific receipts for later target changes.
use super::*;
use crate::event::{InvocationData, JidingCardChange, JidingCardCost};
use crate::snap::{ResolutionSnapshot, SnapshotError};

/// 该牌帧是否已绑定到真实「既定」invocation（`jiding_card == Some(card)`）。
///
/// 写入端必须先确认绑定再付代价/改写载荷：否则会出现「转移发生了，但没有
/// 来源绑定也没有收据」的档案（C9J P1「unpaid redirect / proof strip」与
/// 「origin 未绑定」）。
pub fn jiding_card_is_bound(ctx: &Ctx<'_>, card: EventId) -> bool {
    ctx.res
        .frame(card)
        .and_then(|f| f.parent)
        .and_then(|parent| ctx.res.frame(parent))
        .is_some_and(|frame| {
            matches!(&frame.data, EventData::SkillInvocation(data) if data.jiding_card == Some(card))
        })
}

/// 记录一次来源绑定的牌变更；返回是否真正记入既定 invocation。
pub fn record_jiding_card_change(ctx: &mut Ctx<'_>, change: JidingCardChange) -> bool {
    let Some(parent) = ctx.res.frame(change.card).and_then(|f| f.parent) else { return false };
    if let Some(frame) = ctx.res.frame_mut(parent) {
        if let EventData::SkillInvocation(data) = &mut frame.data {
            if data.jiding_card == Some(change.card) {
                data.jiding_changes.push(change);
                return true;
            }
        }
    }
    false
}

pub(super) fn validate_jiding_card(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    res: &ResolutionSnapshot,
    invocation: EventId,
    context: &InvocationData,
    child: EventId,
) -> Result<(), SnapshotError> {
    let bad = || SnapshotError::BadTag("jiding_card_change", 0);
    // 来源绑定：变更只能记在真实 jiding 牌帧上（origin 绑定），张数有上界。
    if context.jiding_card != Some(child)
        || context.jiding_changes.len() > core.players.len().saturating_mul(3).max(1)
    {
        return Err(bad());
    }
    // 付款的持久证明：牌已进弃牌堆且不在付款者的任何牌区。收据帧结算后
    // （叶子事件弹栈）仍可据此证明代价确已支付；收据帧仍在栈上时用逐字段
    // 精确核对（更严格）。两者都不成立 = 未付代价的转移，拒绝。
    let paid = |card: crate::ids::CardId, payer: crate::ids::PlayerId| {
        let p = core.player(payer);
        core.discard.contains(&card) && !p.hand.contains(&card) && !p.equips.contains(&card)
    };
    let mut player = context.owner;
    let mut targets: Vec<_> = context.source.into_iter().collect();
    let mut used = std::collections::BTreeSet::new();
    let mut last_child = child;
    for change in &context.jiding_changes {
        if change.actor as usize >= core.players.len()
            || change.before_player as usize >= core.players.len()
            || change.before_target as usize >= core.players.len()
            || change.card != child || change.before_player != player
            || targets.as_slice() != [change.before_target]
            || !used.insert((change.def, change.actor))
        {
            return Err(bad());
        }
        let def = catalog.skills.get(change.def as usize).ok_or_else(bad)?;
        if res.parks.iter().any(|park| matches!(&park.machine,
            crate::snap::MachineSnapshot::TriggerOrder { frame, done, .. }
                if *frame == child && !done.contains(&(change.actor, change.def))))
        {
            return Err(bad());
        }
        // 收据帧仍在栈上 → 精确核对（父帧/载荷/序号严格单调）；已结算 → Ok(false)。
        let mut check_child = |id: EventId, parent: EventId, data: &dyn Fn(&EventData) -> bool| {
            if id <= last_child || id >= res.next_event_id { return Err(bad()); }
            last_child = id;
            match res.stack.iter().find(|f| f.id == id) {
                Some(frame) => {
                    if frame.parent != Some(parent) || !data(&frame.data) { return Err(bad()); }
                    Ok(true)
                }
                None => Ok(false),
            }
        };
        match &change.cost {
            JidingCardCost::Zhenlie { lose_hp } => {
                if def.behavior.as_host_behavior() != Some(crate::vocabulary::host::HostBehavior::Zhenlie)
                    || change.actor != change.before_target || change.actor == player {
                    return Err(bad());
                }
                check_child(*lose_hp, child, &|d| matches!(d, EventData::LoseHp { player, amount: 1 } if *player == change.actor))?;
                targets.clear();
            }
            JidingCardCost::TongJi { card, loss } => {
                if def.name != crate::vocabulary::names::TONG_JI
                    || change.actor == player || change.actor == change.before_target
                    || *card as usize >= catalog.cards.len() { return Err(bad()); }
                // 未付代价（loss=None）或收据无法证明付款 → 拒绝。
                let Some(id) = loss else { return Err(bad()); };
                let present = check_child(*id, child, &|d| matches!(d, EventData::CardsLost { player, cards, .. } if *player == change.before_target && cards == &[*card]))?;
                if !present && !paid(*card, change.before_target) { return Err(bad()); }
                targets = vec![change.actor];
            }
            JidingCardCost::Yihua { cards, losses } => {
                if def.behavior.as_host_behavior() != Some(crate::vocabulary::host::HostBehavior::Yihua)
                    || change.actor != change.before_target || change.actor == player
                    || cards.len() != 2 || cards[0] == cards[1] || losses.len() != 2
                    || cards.iter().any(|c| *c as usize >= catalog.cards.len()) { return Err(bad()); }
                let mut parent = child;
                for (card, id) in cards.iter().zip(losses) {
                    let present = check_child(*id, parent, &|d| matches!(d, EventData::CardsLost { player, cards, .. } if *player == change.actor && cards == &[*card]))?;
                    if !present && !paid(*card, change.actor) { return Err(bad()); }
                    parent = *id;
                }
                targets = vec![player];
                player = change.actor;
            }
        }
    }
    if let Some(frame) = res.stack.iter().find(|f| f.id == child) {
        if frame.parent != Some(invocation)
            || !matches!(&frame.data, EventData::CardUsed(u) if u.player == player
                && u.use_as == Some(CardKind::LeiSha) && u.card.is_none()
                && u.nature == crate::event::DamageNature::Thunder && u.targets == targets) {
            return Err(bad());
        }
    }
    Ok(())
}
