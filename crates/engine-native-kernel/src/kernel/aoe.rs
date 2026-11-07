use super::*;

/// 热闷（Boss 都市王，规则 v273）：防具槽空时南蛮/万箭对其无效
///（上游 useCardToBefore cancel 同口径；防具槽=equip_slot()==Armor 全族）。
pub(super) fn dushiwang_remen_blocks(ctx: &Ctx<'_>, target: PlayerId) -> bool {
    ctx.catalog
        .skill_by_name(crate::vocabulary::names::BOSS_REMEN)
        .map(|def| {
            ctx.core.has_skill(target, def)
                && !ctx.core.player(target).equips.iter().any(|c| {
                    ctx.catalog.card_slot_of(ctx.core, *c) == Some(crate::catalog::EquipSlot::Armor)
                })
        })
        .unwrap_or(false)
}

/// 帷幕（楚江王，规则 v275）：黑色锦囊牌对持有者无效（上游 mod
/// targetEnabled 同族；AOE 面在窗口前跳过——万箭为红/无指向锦囊天然不
/// 触发）。
pub(super) fn weimu_blocks_aoe(ctx: &Ctx<'_>, card: crate::ids::CardId, target: PlayerId) -> bool {
    let black = matches!(
        ctx.catalog.cards[card as usize].suit,
        crate::catalog::Suit::Spade | crate::catalog::Suit::Club
    );
    if !black {
        return false;
    }
    ctx.catalog
        .skill_by_name(crate::vocabulary::names::BOSS_WEIMU)
        .map(|def| ctx.core.has_skill(target, def))
        .unwrap_or(false)
}

/// AOE 单目标段（南蛮入侵/万箭齐发）：询问响应牌（杀/闪），Pass 或无牌
/// 则受 1 点伤害（完整伤害帧）。无牌者不询问直接受伤（跳过语义挂账见清单）。
#[derive(Clone)]
pub struct AoeTargetMachine {
    pub(super) source: PlayerId,
    pub(super) target: PlayerId,
    /// 响应需求（南蛮/惊雷山=杀，万箭/炽羽袭=闪）。
    pub(super) need: CardKind,
    pub(super) asked: bool,
    /// 造成伤害的锦囊牌（DamageData.card 透传）。
    pub(super) card: crate::ids::CardId,
    /// 伤害属性（惊雷山=雷、炽羽袭=火）。
    pub(super) nature: crate::event::DamageNature,
}

impl AoeTargetMachine {
    pub(crate) fn new(
        source: PlayerId,
        target: PlayerId,
        need: CardKind,
        asked: bool,
        card: crate::ids::CardId,
        nature: crate::event::DamageNature,
    ) -> Self {
        Self {
            source,
            target,
            need,
            asked,
            card,
            nature,
        }
    }
}

impl crate::catalog::SkillMachine for AoeTargetMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }

    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::AoeTarget {
            source: self.source,
            target: self.target,
            need: self.need,
            asked: self.asked,
            card: self.card,
            nature: self.nature,
        }
    }

    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if self.asked {
            self.asked = false;
            let answer = ctx.answer().clone();
            if let Some(ChoiceOption::Card(c)) = answer.first().copied() {
                // 出响应牌免伤
                discard_response_card(ctx, self.target, c, self.need);
                ctx.emit(EventData::CardResponded {
                    card: c,
                    player: self.target,
                });
                // 雷击（Boss 青龙，规则 v234）：万箭闪响应后判定雷击
                // （南蛮响应为杀，不触发）
                if self.need == CardKind::Shan || ctx.catalog.card_kind(c) == CardKind::Shan {
                    releiji_on_shan(ctx, self.target);
                }
            } else {
                // 放弃 → 1 点伤害（完整结算）
                let vm_card_bonus = ctx
                    .core
                    .vm_aux
                    .flags
                    .get(&(self.source, format!("card_damage_bonus:{}", self.card)))
                    .copied()
                    .unwrap_or(0);
                ctx.emit(EventData::Damage(damage_data_with(
                    Some(self.source),
                    self.target,
                    (1 + vm_card_bonus) as i32,
                    self.nature,
                    Some(self.card),
                )));
            }
            return Flow::Done;
        }
        let holders: Vec<crate::ids::CardId> = response_kind_cards(ctx, self.target, self.need);
        if holders.is_empty() {
            // 无响应牌：不询问，直接受伤（跳过语义挂账见清单）
            let vm_card_bonus = ctx
                .core
                .vm_aux
                .flags
                .get(&(self.source, format!("card_damage_bonus:{}", self.card)))
                .copied()
                .unwrap_or(0);
            ctx.emit(EventData::Damage(damage_data_with(
                Some(self.source),
                self.target,
                (1 + vm_card_bonus) as i32,
                self.nature,
                Some(self.card),
            )));
            return Flow::Done;
        }
        self.asked = true;
        let mut options = vec![ChoiceOption::Pass];
        if !ctx.catalog.blocks_all_cards(ctx.core, self.target) {
            options.extend(holders.into_iter().map(ChoiceOption::Card));
            // 神弓（swd 批 148）：按需求牌种借用他人装备
            options.extend((ctx.catalog.hooks.borrow_options)(
                ctx,
                self.target,
                self.need == CardKind::Shan,
            ));
        }
        Flow::WaitChoice {
            observer: self.target,
            spec: ChoiceSpec {
                kind: ChoiceKind::RespondCard,
                prompt: "aoe_response",
                options,
                min: 1,
                max: 1,
                allow_cancel: false,
            },
        }
    }
}

/// AOE 多目标外层机器：按冻结目标顺序逐个结算；每个目标结算前先开
/// **独立的无懈窗口**（逐目标窗口，抵消只作用于该目标——旧实现为整牌级
/// 单窗口，见覆盖清单已关闭的行为差异行）。目标中途死亡由存活过滤跳过，
/// 队列继续。AOE 牌由本机器在全部目标结算后一次性弃置。
pub struct AoeResolveMachine {
    pub(super) card: crate::ids::CardId,
    pub(super) source: PlayerId,
    /// 响应需求（南蛮/惊雷山=杀，万箭/炽羽袭=闪）。
    pub(super) need: CardKind,
    pub(super) targets: Vec<PlayerId>,
    pub(super) idx: usize,
    /// 当前目标的无懈窗口（内含响应段机器）。
    pub(super) inner: Option<Box<dyn SkillMachine>>,
    /// 伤害属性（惊雷山=雷、炽羽袭=火，其余普通）。
    pub(super) nature: crate::event::DamageNature,
}

impl Clone for AoeResolveMachine {
    fn clone(&self) -> Self {
        Self {
            card: self.card,
            source: self.source,
            need: self.need,
            targets: self.targets.clone(),
            idx: self.idx,
            inner: self.inner.as_ref().map(|m| m.fork()),
            nature: self.nature,
        }
    }
}

impl AoeResolveMachine {
    pub(crate) fn new(
        card: crate::ids::CardId,
        source: PlayerId,
        need: CardKind,
        targets: Vec<PlayerId>,
        idx: usize,
        inner: Option<Box<dyn SkillMachine>>,
        nature: crate::event::DamageNature,
    ) -> Self {
        Self {
            card,
            source,
            need,
            targets,
            idx,
            inner,
            nature,
        }
    }

    fn spawn_window(&self, ctx: &Ctx<'_>) -> Box<dyn SkillMachine> {
        let target = self.targets[self.idx];
        // AOE 逐目标窗口不拥有弃置权（N01）：外层在全部目标结算后统一弃置。
        // 询问队列自来源起（含本人），对齐旧线 alive_from_seat(actor)。
        Box::new(NullifyWindowMachine::new(
            self.card,
            self.source,
            false,
            ctx.core.seat_order_from(self.source),
            false,
            true,
            false,
            Some(Box::new(AoeTargetMachine::new(
                self.source,
                target,
                self.need,
                false,
                self.card,
                self.nature,
            ))),
        ))
    }
}

impl crate::catalog::SkillMachine for AoeResolveMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(Self {
            card: self.card,
            source: self.source,
            need: self.need,
            targets: self.targets.clone(),
            idx: self.idx,
            inner: self.inner.as_ref().map(|m| m.fork()),
            nature: self.nature,
        })
    }

    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::Aoe {
            card: self.card,
            source: self.source,
            need: self.need,
            targets: self.targets.clone(),
            idx: self.idx,
            inner: self.inner.as_ref().map(|m| Box::new(m.snapshot())),
            nature: self.nature,
        }
    }

    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        loop {
            // 跳过已死亡目标（窗口未开启时）；热闷（Boss 都市王，规则
            // v273）——防具槽空时南蛮/万箭对其无效（上游 useCardToBefore
            // cancel 同口径），窗口开启前跳过
            while self.inner.is_none()
                && self.idx < self.targets.len()
                && (!ctx.core.player(self.targets[self.idx]).alive
                    || dushiwang_remen_blocks(ctx, self.targets[self.idx])
                    || weimu_blocks_aoe(ctx, self.card, self.targets[self.idx])
                    || ctx
                        .catalog
                        .aoe_excludes_target(ctx.core, self.targets[self.idx]))
            {
                self.idx += 1;
            }
            if self.idx >= self.targets.len() {
                // 收尾：AOE 牌离桌入弃牌堆（唯一弃置点）
                let card = self.card;
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(self.source, format!("card_damage_bonus:{card}")));
                ctx.to_discard(card);
                ctx.core.table.retain(|c| *c != card);
                return Flow::Done;
            }
            if self.inner.is_none() {
                self.inner = Some(self.spawn_window(ctx));
            }
            let flow = self.inner.as_mut().expect("checked").step(ctx);
            if matches!(flow, Flow::Done) {
                self.inner = None;
                self.idx += 1;
                continue;
            }
            return flow;
        }
    }
}
