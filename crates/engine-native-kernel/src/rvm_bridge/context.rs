//! VM 技能触发宿主上下文提取（`VmTriggerCtx`、`trigger_ctx_of`、`filter_view`）。

use crate::event::EventData;
use crate::ids::{CardId, PlayerId};

/// VM 技能触发时捕获的宿主上下文（从触发事件提取）。
/// targets 为堆面（增量 17），弃 Copy 保 Clone。
#[derive(Debug, Clone, Default)]
pub(super) struct VmTriggerCtx {
    pub(super) event_player: Option<PlayerId>,
    pub(super) event_source: Option<PlayerId>,
    pub(super) event_card: Option<CardId>,
    pub(super) event_cards: Vec<CardId>,
    pub(super) event_amount: u32,
    /// 阶段事件携带（阶段种，begin 由 EventKind 区分；增量 14）。
    pub(super) phase: Option<crate::event::Phase>,
    /// 伤害载荷携带（增量 16；非伤害帧为 None——串帧防护面）。
    pub(super) nature: Option<crate::event::DamageNature>,
    /// 用牌目标集（增量 17；CardUsed 载荷——card_targeted 语义以
    /// CardUsed+targets 表达，原生无独立 targeted 帧）。
    pub(super) targets: Vec<u8>,
    /// view-as 声明牌种。
    pub(super) use_as: Option<crate::catalog::CardKind>,
    /// 铁索传导。
    pub(super) from_chain: bool,
    /// 本批张数（弃牌/移牌）。
    pub(super) batch_len: u32,
    /// 移牌来源座位。
    pub(super) moved_from: Option<PlayerId>,
    /// 阶段开始/结束。
    pub(super) begin: Option<bool>,
    /// 获得技能的定义号（SkillToggled enabled=true）。
    pub(super) added_skill_def: Option<u16>,
    /// 1 伤害 2 用牌 3 阶段 4 弃牌 5 移牌 6 濒死 7 死亡 8 打出 9 给牌 12 武将状态 13 获得技能。
    pub(super) kind: u8,
}

/// 普通锦囊判定（增量 16）：锦囊三类（非基本/非装备/非异类）且非延时
///（乐/兵/闪电/洪水——G1 运筹洪水同为延时）。异类（地/机关）不计。
pub(super) fn kind_is_ordinary_trick(kind: crate::catalog::CardKind) -> bool {
    use crate::catalog::CardKind as K;
    if kind.is_basic_kind() || kind.equip_slot().is_some() {
        return false;
    }
    !matches!(kind, K::Le | K::Bing | K::ShanDian | K::HongShui) && kind.is_basic_trick_or_equip()
}

pub(super) fn trigger_ctx_of(event: &EventData) -> VmTriggerCtx {
    match event {
        EventData::Damage(d) => VmTriggerCtx {
            event_player: Some(d.target),
            event_source: d.source,
            event_card: d.card,
            event_amount: d.amount as u32,
            nature: Some(d.nature),
            from_chain: d.from_chain,
            kind: 1,
            ..VmTriggerCtx::default()
        },
        EventData::TurnBegin { player } => VmTriggerCtx {
            event_player: Some(*player),
            event_source: Some(*player),
            kind: 3,
            ..VmTriggerCtx::default()
        },
        EventData::TurnEndWindow { player, .. } => VmTriggerCtx {
            event_player: Some(*player),
            event_source: Some(*player),
            kind: 3,
            ..VmTriggerCtx::default()
        },
        EventData::Phase {
            player,
            phase,
            begin,
        } => VmTriggerCtx {
            event_player: Some(*player),
            event_source: Some(*player),
            phase: Some(*phase),
            begin: Some(*begin),
            kind: 3,
            ..VmTriggerCtx::default()
        },
        // 杀被闪避（PA12 批②）：来源=杀的使用者、事件主玩家=闪避的目标。
        EventData::ShaDodged {
            card,
            source,
            target,
        } => VmTriggerCtx {
            event_player: Some(*target),
            event_source: Some(*source),
            event_card: *card,
            kind: 8,
            ..VmTriggerCtx::default()
        },
        // 给牌（NC04）：事件主玩家=recipient、来源=giver（旧线 dispatch
        // TriggerContext{player: recipient, source: giver} 同口径）。
        EventData::CardsGiven {
            giver,
            recipient,
            cards,
        } => VmTriggerCtx {
            event_player: Some(*recipient),
            event_source: Some(*giver),
            event_card: cards.first().copied(),
            event_cards: cards.clone(),
            event_amount: cards.len() as u32,
            batch_len: cards.len() as u32,
            kind: 9,
            ..VmTriggerCtx::default()
        },
        // PA12 批⑥：摸牌（CardsDrawn）、体力恢复（Recover）、体力流失（LoseHp）、
        // 脱离濒死（DyingSaved）四事件此前在装载器已映射 EventKind，但
        // trigger_ctx_of 落入默认空臂（event_player=None、event_amount=0、
        // event_cards=[]），导致 owner_is_target* / owner_gained_* 与模块体内
        // event_player/event_amount/event_card_count 失真。
        EventData::CardsDrawn { player, cards } => VmTriggerCtx {
            event_player: Some(*player),
            event_source: Some(*player),
            event_card: cards.first().copied(),
            event_cards: cards.clone(),
            event_amount: cards.len() as u32,
            batch_len: cards.len() as u32,
            kind: 5,
            ..VmTriggerCtx::default()
        },
        EventData::Recover {
            player,
            amount,
            by_card,
        } => VmTriggerCtx {
            event_player: Some(*player),
            event_source: Some(*player),
            event_card: *by_card,
            event_amount: *amount,
            kind: 10,
            ..VmTriggerCtx::default()
        },
        EventData::LoseHp { player, amount } => VmTriggerCtx {
            event_player: Some(*player),
            event_source: Some(*player),
            event_amount: *amount,
            kind: 11,
            ..VmTriggerCtx::default()
        },
        EventData::DyingSaved { player, source } => VmTriggerCtx {
            event_player: Some(*player),
            event_source: *source,
            kind: 6,
            ..VmTriggerCtx::default()
        },
        EventData::CardUsed(u) => VmTriggerCtx {
            event_player: Some(u.player),
            // 旧线 CardUsed 的 TriggerContext.source = 用牌者；原生此前未填
            // 导致 `owner_is_source` 对用牌事件恒假（PA12 批①）。
            event_source: Some(u.player),
            event_card: u.card,
            event_cards: u.card.into_iter().collect(),
            targets: u.targets.clone(),
            use_as: u.use_as,
            nature: Some(u.nature),
            kind: 2,
            ..VmTriggerCtx::default()
        },
        EventData::Death { player, killer } => VmTriggerCtx {
            event_player: Some(*player),
            event_source: *killer,
            kind: 7,
            ..VmTriggerCtx::default()
        },
        EventData::CardsMoved { cards, from, to } => VmTriggerCtx {
            event_player: Some(*to),
            event_card: cards.first().copied(),
            event_cards: cards.clone(),
            event_amount: cards.len() as u32,
            moved_from: Some(*from),
            batch_len: cards.len() as u32,
            kind: 5,
            ..VmTriggerCtx::default()
        },
        EventData::CardsDiscarded { player, cards } => VmTriggerCtx {
            event_player: Some(*player),
            event_card: cards.first().copied(),
            event_cards: cards.clone(),
            event_amount: cards.len() as u32,
            batch_len: cards.len() as u32,
            kind: 4,
            ..VmTriggerCtx::default()
        },
        EventData::Dying { player, source } => VmTriggerCtx {
            event_player: Some(*player),
            event_source: *source,
            kind: 6,
            ..VmTriggerCtx::default()
        },
        EventData::CardResponded { card, player } => VmTriggerCtx {
            event_player: Some(*player),
            event_source: Some(*player),
            event_card: Some(*card),
            event_cards: vec![*card],
            kind: 8,
            ..VmTriggerCtx::default()
        },
        EventData::TurnOvered { player, face_down } => VmTriggerCtx {
            event_player: Some(*player),
            event_source: Some(*player),
            event_amount: if *face_down { 2 } else { 1 },
            kind: 12,
            ..VmTriggerCtx::default()
        },
        EventData::Linked { player, linked } => VmTriggerCtx {
            event_player: Some(*player),
            event_source: Some(*player),
            event_amount: if *linked { 3 } else { 4 },
            kind: 12,
            ..VmTriggerCtx::default()
        },
        EventData::SkillToggled {
            player,
            skill,
            enabled: true,
            ..
        } => VmTriggerCtx {
            event_player: Some(*player),
            event_source: Some(*player),
            added_skill_def: Some(*skill),
            kind: 13,
            ..VmTriggerCtx::default()
        },
        _ => VmTriggerCtx::default(),
    }
}

pub(super) fn filter_view(t: &VmTriggerCtx) -> crate::rvm_named::FilterView {
    crate::rvm_named::FilterView {
        event_player: t.event_player,
        event_source: t.event_source,
        event_card: t.event_card,
        event_amount: t.event_amount,
        phase: t.phase,
        nature: t.nature,
        targets: t.targets.clone(),
        use_as: t.use_as,
        from_chain: t.from_chain,
        batch_len: t.batch_len,
        moved_from: t.moved_from,
        begin: t.begin,
        kind: t.kind,
    }
}
