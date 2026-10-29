use super::*;

/// 无中生有结算：使用者摸 2。
#[derive(Clone)]
pub struct WuzhongMachine {
    pub owner: PlayerId,
    pub card: Option<crate::ids::CardId>,
}

impl crate::catalog::SkillMachine for WuzhongMachine {
    fn fork(&self) -> Box<dyn crate::catalog::SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::Wuzhong { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        ctx.draw_cards(self.owner, 2);
        // 弃置锦囊本体（按牌种扫表：恢复期安全，不依赖快照携带 card）。
        if let Some(pos) = ctx
            .core
            .table
            .iter()
            .position(|c| ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Wuzhong)
        {
            let card = ctx.core.table.remove(pos);
            ctx.to_discard(card);
        }
        Flow::Done
    }
}

/// 桃园结义结算：所有其他存活角色回复 1 点体力。
#[derive(Clone)]
pub struct TaoyuanMachine {
    pub owner: PlayerId,
    pub card: Option<crate::ids::CardId>,
}

impl crate::catalog::SkillMachine for TaoyuanMachine {
    fn fork(&self) -> Box<dyn crate::catalog::SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::Taoyuan { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let others: Vec<PlayerId> = ctx
            .core
            .alive_players()
            .filter(|p| *p != self.owner)
            .collect();
        for p in others {
            let (hp, max) = {
                let pl = ctx.core.player(p);
                (pl.hp, pl.max_hp)
            };
            if hp < max as i32 {
                ctx.recover(p, 1, None);
            }
        }
        if let Some(pos) = ctx
            .core
            .table
            .iter()
            .position(|c| ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Taoyuan)
        {
            let card = ctx.core.table.remove(pos);
            ctx.to_discard(card);
        }
        Flow::Done
    }
}

/// 地载阵结算（zhenfa 首切片，规则 v110）：所有非大势力存活角色各摸
/// 一张（上游 target.draw(false) 同口径；大势力=is_major_faction）。
#[derive(Clone)]
pub struct DizaizhenMachine {
    pub owner: PlayerId,
}

impl crate::catalog::SkillMachine for DizaizhenMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        // 无询问机器（逐目标摸牌，Noop 同款快照形态）
        crate::snap::MachineSnapshot::Noop
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let targets: Vec<PlayerId> = ctx
            .core
            .alive_players()
            .filter(|p| !ctx.core.is_major_faction(ctx.catalog, *p))
            .collect();
        for t in targets {
            ctx.draw_cards(t, 1);
        }
        // 本体牌离结算区入弃堆（Wugu 同款收尾）
        if let Some(pos) = ctx
            .core
            .table
            .iter()
            .position(|c| ctx.catalog.card_kind(*c) == CardKind::Dizaizhen)
        {
            let c = ctx.core.table.remove(pos);
            ctx.to_discard(c);
        }
        Flow::Done
    }
}

/// 五谷丰登结算：亮 min(存活数, 牌堆) 张，从使用者起按座位序逐人择一。
#[derive(Clone)]
pub struct WuguMachine {
    pub owner: PlayerId,
    pub cards: Vec<crate::ids::CardId>,
    pub pickers: Vec<PlayerId>,
    pub stage: WuguStage,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WuguStage {
    /// 亮牌并确定择序（首个询问）。
    Init,
    /// 逐人读应答/开下一询问。
    Pick,
}

impl crate::catalog::SkillMachine for WuguMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::Wugu {
            owner: self.owner,
            cards: self.cards.iter().copied().map(|c| c as u32).collect(),
            pickers: self.pickers.clone(),
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            WuguStage::Init => {
                let alive: Vec<PlayerId> = ctx.core.alive_players().collect();
                let take = alive.len().min(ctx.core.deck.len());
                if take == 0 || self.cards.is_empty() == false {
                    return Flow::Done;
                }
                let new_len = ctx.core.deck.len() - take;
                self.cards = ctx.core.deck.split_off(new_len);
                self.pickers = alive;
                self.stage = WuguStage::Pick;
                self.open_pick(ctx)
            }
            WuguStage::Pick => {
                // 读当前择牌者应答，交牌并推进。
                let picker = match self.pickers.first().copied() {
                    Some(p) => p,
                    None => return self.finish_wugu(ctx),
                };
                if let Some(ChoiceOption::Card(card)) = ctx.answer().first().copied() {
                    if let Some(pos) = self.cards.iter().position(|c| *c == card) {
                        self.cards.remove(pos);
                        if ctx.core.player(picker).alive {
                            ctx.core.player_mut(picker).hand.push(card);
                            ctx.out.push(crate::LogEvent::CardsDrawn {
                                player: picker,
                                cards: vec![card],
                            });
                        }
                    }
                }
                self.pickers.remove(0);
                if self.pickers.is_empty() || self.cards.is_empty() {
                    return self.finish_wugu(ctx);
                }
                self.open_pick(ctx)
            }
        }
    }
}

impl WuguMachine {
    fn open_pick(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let Some(picker) = self.pickers.first().copied() else {
            return self.finish_wugu(ctx);
        };
        if !ctx.core.player(picker).alive || self.cards.is_empty() {
            self.pickers.remove(0);
            if self.pickers.is_empty() {
                return self.finish_wugu(ctx);
            }
            return self.open_pick(ctx);
        }
        let options: Vec<ChoiceOption> =
            self.cards.iter().copied().map(ChoiceOption::Card).collect();
        Flow::WaitChoice {
            observer: picker,
            spec: crate::choice::ChoiceSpec {
                kind: crate::choice::ChoiceKind::SelectCards,
                prompt: "wugu_pick",
                options,
                min: 1,
                max: 1,
                allow_cancel: false,
            },
        }
    }
    fn finish_wugu(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // 良策·余获（玩点论杀批 15）：剩余牌归持有者（登记差异：上游
        // 玩家选目标）。结构性说明：4 人局五谷亮牌数=存活数=择牌数，
        // **永无剩余**——本近似与「可选目标」在可达面等价（无剩余时均
        // 无效果）；多亮/减员等剩余场景上游亦罕见，登记边界。
        let _beneficiary = ctx.catalog.wugu_remain_beneficiary(ctx.core);
        for c in self.cards.drain(..) {
            ctx.to_discard(c);
        }
        if let Some(c) = ctx
            .core
            .table
            .iter()
            .position(|c| ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Wugu)
        {
            let card = ctx.core.table.remove(c);
            ctx.to_discard(card);
        }
        Flow::Done
    }
}

/// 借刀杀人结算：武器持有者出杀（登记差异：直接伤害，无出闪窗口）
/// 或把武器交給另一名目标。
#[derive(Clone)]
pub struct JiedaoMachine {
    pub owner: PlayerId,
    pub holder: PlayerId,
    pub victim: PlayerId,
    pub card: Option<crate::ids::CardId>,
    pub stage: JiedaoStage,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum JiedaoStage {
    Ask,
    Read,
}

impl crate::catalog::SkillMachine for JiedaoMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::Jiedao {
            owner: self.owner,
            holder: self.holder,
            victim: self.victim,
            stage: match self.stage {
                JiedaoStage::Ask => crate::snap::JiedaoStageSnap::Ask,
                JiedaoStage::Read => crate::snap::JiedaoStageSnap::Read,
            },
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            JiedaoStage::Ask => {
                // 资格复核：持有者仍在且武器仍在。
                let has_weapon = ctx
                    .core
                    .player(self.holder)
                    .equips
                    .iter()
                    .any(|c| ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Weapon);
                if !ctx.core.player(self.holder).alive || !has_weapon {
                    return self.finish_jiedao(ctx);
                }
                self.stage = JiedaoStage::Read;
                Flow::WaitChoice {
                    observer: self.holder,
                    spec: crate::choice::ChoiceSpec {
                        kind: crate::choice::ChoiceKind::PickTag,
                        prompt: "jiedao_sha_or_give",
                        options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                }
            }
            JiedaoStage::Read => {
                match ctx.answer_first() {
                    // 出杀：直接伤害结算（登记差异：无出闪窗口）。
                    ChoiceOption::Tag(0) => {
                        if ctx.core.player(self.victim).alive {
                            ctx.emit(crate::event::EventData::Damage(
                                crate::machine::damage_data(Some(self.holder), self.victim, 1),
                            ));
                        }
                    }
                    // 交出武器：武器移入另一目标装备区（原武器若有则替换
                    // 入弃牌堆）。
                    _ => {
                        let weapon =
                            ctx.core
                                .player(self.holder)
                                .equips
                                .iter()
                                .copied()
                                .find(|c| {
                                    ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Weapon
                                });
                        if let Some(w) = weapon {
                            ctx.core.player_mut(self.holder).equips.retain(|c| *c != w);
                            crate::kernel::revoke_equip_skills(
                                ctx.core,
                                ctx.catalog,
                                self.holder,
                                w,
                            );
                            let victim = self.victim;
                            let old =
                                ctx.core
                                    .player_mut(victim)
                                    .equips
                                    .iter()
                                    .copied()
                                    .find(|c| {
                                        ctx.catalog.card_kind(*c)
                                            == crate::catalog::CardKind::Weapon
                                    });
                            if let Some(old_w) = old {
                                ctx.core.player_mut(victim).equips.retain(|c| *c != old_w);
                                crate::kernel::revoke_equip_skills(
                                    ctx.core,
                                    ctx.catalog,
                                    victim,
                                    old_w,
                                );
                                ctx.to_discard(old_w);
                            }
                            ctx.core.player_mut(victim).equips.push(w);
                        }
                    }
                }
                self.finish_jiedao(ctx)
            }
        }
    }
}

impl JiedaoMachine {
    fn finish_jiedao(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        self.card = None;
        if let Some(pos) = ctx
            .core
            .table
            .iter()
            .position(|c| ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Jiedao)
        {
            let card = ctx.core.table.remove(pos);
            ctx.to_discard(card);
        }
        Flow::Done
    }
}

/// 过河拆桥：弃置目标角色一张牌（手牌优先，其次装备区，再次判定区）。
#[derive(Clone)]
pub struct DismantleMachine {
    pub card: crate::ids::CardId,
    pub source: PlayerId,
    pub target: PlayerId,
    pub asked: bool,
}

impl crate::catalog::SkillMachine for DismantleMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::Dismantle {
            card: self.card,
            source: self.source,
            target: self.target,
            asked: self.asked,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if !self.asked {
            self.asked = true;
            // 选牌：手牌优先，其次装备区，再次判定区
            let t = ctx.core.player(self.target);
            let pick = t
                .hand
                .first()
                .copied()
                .or_else(|| t.equips.first().copied())
                .or_else(|| t.judge.first().copied());
            if let Some(c) = pick {
                // 从所在区移除
                let from_hand = t.hand.contains(&c);
                {
                    let was_equip = ctx.catalog.card_kind(c).is_equip();
                    let p = ctx.core.player_mut(self.target);
                    p.hand.retain(|x| *x != c);
                    p.equips.retain(|x| *x != c);
                    p.judge.retain(|x| *x != c);
                    if was_equip {
                        revoke_equip_skills(ctx.core, ctx.catalog, self.target, c);
                    }
                }
                ctx.to_discard(c);
                ctx.log(LogEvent::CardDiscarded {
                    player: self.target,
                    cards: vec![c],
                });
                if from_hand {
                    // 弃置事件：连营类「失去手牌」触发依赖此事件（ADR 0763）
                    ctx.emit(EventData::CardsDiscarded {
                        player: self.target,
                        cards: vec![c],
                    });
                }
            }
            let card = self.card;
            ctx.to_discard(card);
            ctx.core.table.retain(|x| *x != card);
        }
        Flow::Done
    }
}

/// 顺手牵羊结算：获得一名其他角色的一张牌（手牌优先、其次装备，
/// 各区首张确定性取牌；获得进来源手牌——与拆桥弃置不同）。
#[derive(Clone)]
pub struct SnatchMachine {
    pub card: crate::ids::CardId,
    pub source: PlayerId,
    pub target: PlayerId,
    pub asked: bool,
}

impl crate::catalog::SkillMachine for SnatchMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::Snatch {
            card: self.card,
            source: self.source,
            target: self.target,
            asked: self.asked,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if !self.asked {
            self.asked = true;
            let t = ctx.core.player(self.target);
            let pick = t
                .hand
                .first()
                .copied()
                .or_else(|| t.equips.first().copied());
            if let Some(c) = pick {
                {
                    let was_equip = ctx.catalog.card_kind(c).is_equip();
                    let p = ctx.core.player_mut(self.target);
                    p.hand.retain(|x| *x != c);
                    p.equips.retain(|x| *x != c);
                    if was_equip {
                        revoke_equip_skills(ctx.core, ctx.catalog, self.target, c);
                    }
                }
                ctx.core.player_mut(self.source).hand.push(c);
                ctx.emit(EventData::CardsMoved {
                    cards: vec![c],
                    from: self.target,
                    to: self.source,
                });
                ctx.log(LogEvent::CardsGiven {
                    from: self.target,
                    to: self.source,
                    cards: vec![c],
                });
            }
            let card = self.card;
            ctx.to_discard(card);
            ctx.core.table.retain(|x| *x != card);
        }
        Flow::Done
    }
}
