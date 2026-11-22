//! `Ctx` 与事件载荷构造方法及通用辅助函数。

use super::*;

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
