//! `Ctx` 与事件载荷构造方法及通用辅助函数（含义山记录与 `guanfu` 回收阻断辅助）。

use super::*;
use crate::catalog::CardKind;
use crate::event::DamageNature;

/// `guanfu.global: ['guanfu_disrecover']`（`祖安武将/extension.js:1217-1288`）：
/// 任一存活角色持有 `guanfu` 时拦截恢复；非平衡改为摸等量牌，平衡逐点判定。
pub(super) fn apply_guanfu_disrecover(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
    mut amount: u32,
) -> Option<u32> {
    let has_guanfu = ctx.catalog.skill_by_name("guanfu").is_some_and(|d| {
        (0..ctx.core.players.len() as PlayerId)
            .any(|p| ctx.core.player(p).alive && ctx.core.has_skill(p, d))
    });
    if !has_guanfu || !ctx.core.player(player).alive || amount == 0 {
        return Some(amount);
    }
    let balance = ctx
        .core
        .vm_aux
        .flags
        .get(&(0, "l25_balance".to_string()))
        .copied()
        .unwrap_or(0)
        > 0;
    if ctx.core.player(player).hp <= 0 && !balance {
        return Some(amount);
    }
    if !balance {
        ctx.start_draw(player, amount);
        return None;
    }
    let mut rem = amount;
    while rem >= 1 && ctx.core.player(player).hp + (rem as i32) > 1 {
        rem -= 1;
        if ctx.core.deck.is_empty() && ctx.core.discard.is_empty() {
            break;
        }
        let jcard = ctx.draw_judge_card();
        let suit = ctx.catalog.cards[jcard as usize].suit;
        ctx.out.push(LogEvent::Judged {
            player,
            card: jcard,
        });
        let pile_key = (player, "guanfu_disrecover".to_string());
        let suit_seen = ctx.core.vm_aux.piles.get(&pile_key).is_some_and(|pile| {
            pile.iter()
                .any(|&c| ctx.catalog.cards[c as usize].suit == suit)
        });
        if suit == crate::catalog::Suit::Heart || suit_seen {
            ctx.to_discard(jcard);
            if let Some(stored) = ctx.core.vm_aux.piles.remove(&pile_key) {
                if !stored.is_empty() {
                    ctx.core.discard.extend(stored.iter().copied());
                    ctx.log(LogEvent::CardDiscarded {
                        player,
                        cards: stored,
                    });
                }
            }
        } else {
            amount = amount.saturating_sub(1);
            ctx.core
                .vm_aux
                .piles
                .entry(pile_key)
                .or_default()
                .push(jcard);
        }
    }
    (amount > 0).then_some(amount)
}

/// 便捷构造：普通伤害载荷。
pub fn damage_data(source: Option<PlayerId>, target: PlayerId, amount: i32) -> DamageData {
    damage_data_with(source, target, amount, DamageNature::Normal, None)
}

/// 便捷构造：显式属性伤害载荷。
pub fn damage_data_with(
    source: Option<PlayerId>,
    target: PlayerId,
    amount: i32,
    nature: DamageNature,
    card: Option<CardId>,
) -> DamageData {
    DamageData {
        settlement: Default::default(),
        source,
        target,
        amount,
        nature,
        prevented: false,
        redirect_chain: Vec::new(),
        card,
        from_chain: false,
    }
}

/// 卡牌使用载荷构造（普通属性）。
pub fn card_use(card: CardId, player: PlayerId, targets: Vec<PlayerId>) -> CardUseData {
    card_use_as(card, player, targets, None)
}

/// 带 view-as 声明的构造（转化使用时 `as_kind` = 声明牌种）。
pub fn card_use_as(
    card: CardId,
    player: PlayerId,
    targets: Vec<PlayerId>,
    as_kind: Option<CardKind>,
) -> CardUseData {
    CardUseData {
        card: Some(card),
        player,
        targets,
        nature: DamageNature::Normal,
        use_as: as_kind,
        cancelled: false,
    }
}

/// 卡牌使用载荷构造（显式属性，火杀用）。
pub fn card_use_with_nature(
    card: CardId,
    player: PlayerId,
    targets: Vec<PlayerId>,
    nature: DamageNature,
    use_as: Option<CardKind>,
) -> CardUseData {
    CardUseData {
        card: Some(card),
        player,
        targets,
        nature,
        use_as,
        cancelled: false,
    }
}

/// 义山记录（swd 批 123，规则 v310）自由函数：Ctx 与测试工具共用。
pub fn note_cards_lost(
    core: &mut GameStateCore,
    catalog: &crate::catalog::Catalog,
    player: PlayerId,
    cards: &[CardId],
) {
    let Some(def) = catalog.skill_by_name("yishan") else {
        return;
    };
    if !core.has_skill(player, def) {
        return;
    }
    let list = &mut core.player_mut(player).yishan_lost;
    for c in cards {
        list.insert(0, *c);
    }
    if list.len() > 16 {
        list.truncate(16);
    }
}
