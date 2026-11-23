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

impl<'a> Ctx<'a> {
    /// 虚拟用牌原语（规则 v169 基建，上游 game.useCard({name}) 同族）：
    /// 以牌堆底一张实体为载体、use_as 声明牌种发起一次完整使用管线
    /// （响应/伤害/弃置全走 use_as 分派）。载体经使用者手牌通道取出、
    /// 结算后入弃堆（登记差异：上游凭空造牌无实体不入弃堆）。返回载体。
    pub fn use_virtual_card(
        &mut self,
        source: PlayerId,
        as_kind: crate::catalog::CardKind,
        targets: Vec<PlayerId>,
    ) -> CardId {
        self.use_virtual_card_with_suit_and_nature(
            source,
            as_kind,
            crate::event::DamageNature::Normal,
            false,
            targets,
        )
    }

    /// 扩展虚拟用牌原语：支持声明 `nature` 并在 `force_spade` 时优先择取牌堆黑桃载体或挂单帧黑桃覆写。
    pub fn use_virtual_card_with_suit_and_nature(
        &mut self,
        source: PlayerId,
        as_kind: crate::catalog::CardKind,
        nature: crate::event::DamageNature,
        force_spade: bool,
        targets: Vec<PlayerId>,
    ) -> CardId {
        assert!(!self.core.deck.is_empty(), "virtual use: deck empty");
        let pos = if force_spade {
            self.core
                .deck
                .iter()
                .position(|&c| self.catalog.cards[c as usize].suit == crate::catalog::Suit::Spade)
                .unwrap_or(0)
        } else {
            0
        };
        let carrier = self.core.deck.remove(pos);
        self.core.player_mut(source).hand.push(carrier);
        self.emit(EventData::CardUsed(crate::event::CardUseData {
            card: Some(carrier),
            player: source,
            targets,
            nature,
            use_as: Some(as_kind),
            cancelled: false,
        }));
        if force_spade {
            if let Some(fid) = self.res.stack.last().map(|f| f.id) {
                self.core
                    .vm_aux
                    .flags
                    .insert((source, format!("ow_card_use_suit_spade:{fid}")), 1);
            }
        }
        carrier
    }

    /// `ow::shanxian`（`global: "phaseBefore"`）在翻面跳过前确认劫持时启动己方插队回合。
    pub fn start_hijacked_turn(&mut self, owner: PlayerId) {
        self.core.turn.current = owner;
        self.core.turn.phase = crate::event::Phase::Judge;
        self.core.turn.turn_opened = false;
        crate::kernel::events::turn::reset_new_turn_state(self, owner);
        self.emit(EventData::TurnBegin { player: owner });
    }

    /// `ow::shanxian` 拒绝劫持背面朝上目标时，恢复该目标原本的翻面整回合跳过流程。
    pub fn resume_facedown_turn_skip(&mut self, victim: PlayerId) {
        self.core.player_mut(victim).jileng_active = false;
        self.emit(EventData::TurnEnded { player: victim });
        self.emit(EventData::TurnOverBefore {
            player: victim,
            cancelled: false,
        });
        self.core.player_mut(victim).boss_mad = false;
    }

    /// `ow::shanxian` 在劫持回合 `phaseAfter` 提交后完成自身翻面再归还原本回合交接。
    pub fn handoff_after_shanxian(&mut self, prev: PlayerId) {
        let next = self.core.next_alive(prev);
        crate::kernel::events::turn::turn_handoff(self, prev, next);
    }
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
