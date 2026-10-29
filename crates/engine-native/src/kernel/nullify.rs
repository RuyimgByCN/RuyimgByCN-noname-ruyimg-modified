use super::*;

/// 铁索连环结算：**逐目标**独立无懈窗口（对齐上游 `useCardToBegin` 逐目标
/// 窗口——`_wuxie` 触发于每目标的使用开始、无懈以 `neutralize()` 只作用于
/// 该目标；铁索 `complexTarget` 非 multitarget，故两个目标各有独立窗口）。
/// 抵消只作用于该目标，其余目标照常横置；全部目标结算后牌离桌入弃牌堆
/// （唯一弃置点）。无懈窗口原为整牌级、抵消即全场不横置，本批对齐上游并
/// 关闭覆盖清单行为差异「铁索无懈仍为整牌级」（N06）。
pub struct ChainMachine {
    pub(super) card: crate::ids::CardId,
    pub(super) source: PlayerId,
    pub(super) targets: Vec<PlayerId>,
    /// 已开窗（并结算）的目标数。
    pub(super) idx: usize,
    /// 当前目标的无懈窗口（内含单目标叶子机器）。
    pub(super) inner: Option<Box<dyn SkillMachine>>,
    /// true=整牌调度层（逐目标开窗 + 收尾弃置）；false=窗口内层单目标
    /// 叶子（未抵消时发 Linked 帧翻转横置，不持弃置权）。
    pub(super) dispatcher: bool,
}

impl Clone for ChainMachine {
    fn clone(&self) -> Self {
        Self {
            card: self.card,
            source: self.source,
            targets: self.targets.clone(),
            idx: self.idx,
            inner: self.inner.as_ref().map(|m| m.fork()),
            dispatcher: self.dispatcher,
        }
    }
}

impl ChainMachine {
    pub(crate) fn new(
        card: crate::ids::CardId,
        source: PlayerId,
        targets: Vec<PlayerId>,
        idx: usize,
        inner: Option<Box<dyn SkillMachine>>,
        dispatcher: bool,
    ) -> Self {
        Self {
            card,
            source,
            targets,
            idx,
            inner,
            dispatcher,
        }
    }

    /// 整牌调度层入口（命令层使用）。
    pub(crate) fn dispatcher(
        card: crate::ids::CardId,
        source: PlayerId,
        targets: Vec<PlayerId>,
    ) -> Self {
        Self::new(card, source, targets, 0, None, true)
    }

    /// 单目标叶子（窗口内层效果）。
    fn leaf(card: crate::ids::CardId, source: PlayerId, target: PlayerId) -> Self {
        Self::new(card, source, vec![target], 0, None, false)
    }

    /// 当前目标的独立无懈窗口；窗口不持弃置权（外层收尾统一弃置，N01 同款），
    /// 询问队列自来源起（含本人），对齐旧线 alive_from_seat(actor)。
    fn spawn_window(&self, ctx: &Ctx<'_>) -> Box<dyn SkillMachine> {
        let target = self.targets[self.idx];
        Box::new(NullifyWindowMachine::new(
            self.card,
            self.source,
            false,
            ctx.core.seat_order_from(self.source),
            false,
            true,
            false,
            Some(Box::new(ChainMachine::leaf(self.card, self.source, target))),
        ))
    }
}

impl crate::catalog::SkillMachine for ChainMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(Self {
            card: self.card,
            source: self.source,
            targets: self.targets.clone(),
            idx: self.idx,
            inner: self.inner.as_ref().map(|m| m.fork()),
            dispatcher: self.dispatcher,
        })
    }

    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::Chain {
            card: self.card,
            source: self.source,
            targets: self.targets.clone(),
            idx: self.idx,
            inner: self.inner.as_ref().map(|m| Box::new(m.snapshot())),
            dispatcher: self.dispatcher,
        }
    }

    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if !self.dispatcher {
            // 叶子：窗口未抵消（本机器只在未抵消路径被委托）→ 单目标横置。
            if self.idx == 0 {
                self.idx = 1;
                let t = self.targets[0];
                if ctx.core.player(t).alive {
                    ctx.emit(EventData::Linked {
                        player: t,
                        linked: false,
                    });
                    return Flow::Yield;
                }
            }
            return Flow::Done;
        }
        loop {
            // 跳过开窗前已死亡的目标（窗口期间死亡者由叶子跳过）
            while self.inner.is_none()
                && self.idx < self.targets.len()
                && !ctx.core.player(self.targets[self.idx]).alive
            {
                self.idx += 1;
            }
            if self.idx >= self.targets.len() {
                // 收尾：铁索离桌入弃牌堆（唯一弃置点）
                let card = self.card;
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

/// 无懈可击响应窗口：包裹锦囊效果机器。
/// 自来源下家起按座位顺序询问持无懈者；出无懈即反转「抵消」状态并自其下家
/// 重新询问；窗口关闭时未抵消则委托效果机器，已抵消则锦囊入弃牌堆。
/// 窗口适用范围/整牌级窗口/跳过语义见覆盖清单行为差异表。
pub struct NullifyWindowMachine {
    pub(super) card: crate::ids::CardId,
    pub(super) source: PlayerId,
    pub(super) nullified: bool,
    pub(super) queue: Vec<PlayerId>,
    pub(super) asked: bool,
    /// false = 窗口已关闭，进入效果委托阶段。
    pub(super) window_open: bool,
    /// 抵消关闭时是否由本窗口弃置锦囊：单次结算锦囊（决斗/铁索/延时锦囊）
    /// 为 true；AOE 逐目标窗口为 false——由外层机器在全部目标结算后
    /// 统一弃置（N01：否则「抵消一个目标」路径同一牌进弃牌堆两次）。
    pub(super) owns_disposal: bool,
    pub(super) effect: Option<Box<dyn crate::catalog::SkillMachine>>,
}

impl NullifyWindowMachine {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        card: crate::ids::CardId,
        source: PlayerId,
        nullified: bool,
        queue: Vec<PlayerId>,
        asked: bool,
        window_open: bool,
        owns_disposal: bool,
        effect: Option<Box<dyn crate::catalog::SkillMachine>>,
    ) -> Self {
        Self {
            card,
            source,
            nullified,
            queue,
            asked,
            window_open,
            owns_disposal,
            effect,
        }
    }
}

impl crate::catalog::SkillMachine for NullifyWindowMachine {
    fn fork(&self) -> Box<dyn crate::catalog::SkillMachine> {
        Box::new(Self {
            card: self.card,
            source: self.source,
            nullified: self.nullified,
            queue: self.queue.clone(),
            asked: self.asked,
            window_open: self.window_open,
            owns_disposal: self.owns_disposal,
            effect: self.effect.as_ref().map(|m| m.fork()),
        })
    }

    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::NullifyWindow {
            card: self.card,
            source: self.source,
            nullified: self.nullified,
            queue: self.queue.clone(),
            asked: self.asked,
            window_open: self.window_open,
            owns_disposal: self.owns_disposal,
            effect: self.effect.as_ref().map(|m| Box::new(m.snapshot())),
        }
    }

    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // 起风·闭（T11 Boss 诸葛亮族批 116，规则 v303 同批）：锁定——
        // 来源持技时其使用的牌不可被无懈响应（上游 hlss_fengqi2
        // `mod.wuxieRespondable = false` 同口径）：窗口不询问、不抵消，
        // 直接委托效果机器。
        if self.window_open
            && ctx
                .catalog
                .skill_by_name(crate::vocabulary::host::HLSS_FENGQI2_KEY)
                .map(|def| ctx.core.has_skill(self.source, def))
                .unwrap_or(false)
        {
            self.window_open = false;
            return self.step(ctx);
        }
        // 窗口阶段
        if self.window_open {
            // 询问队列清理：跳过死亡/无无懈持有者
            while let Some(p) = self.queue.first().copied() {
                let has_nullify = ctx.core.player(p).alive
                    && ctx
                        .core
                        .player(p)
                        .hand
                        .iter()
                        .any(|c| ctx.catalog.card_kind(*c) == CardKind::Nullify);
                if !has_nullify {
                    self.queue.remove(0);
                } else {
                    break;
                }
            }
            let Some(responder) = self.queue.first().copied() else {
                // 队列耗尽：窗口关闭。弃牌责任唯一化（ADR 0705 修复）：抵消时
                // 由窗口弃置锦囊；未抵消时效果机器（决斗 finish / AOE 收尾 /
                // 铁索机器）结算结束弃置——旧实现在此无条件弃牌，叠加效果
                // 机器的弃置使非抵消路径同一牌进弃牌堆两次。
                if self.nullified {
                    // 弃置责任唯一化（ADR 0705/0706）：owns_disposal 的窗口
                    // 抵消时弃置；AOE 逐目标窗口不弃置（外层收尾统一弃置）。
                    if self.owns_disposal {
                        let card = self.card;
                        ctx.to_discard(card);
                        ctx.core.table.retain(|c| *c != card);
                    }
                    ctx.log(LogEvent::CardNullified {
                        card: self.card,
                        player: self.source,
                    });
                    return Flow::Done;
                }
                self.window_open = false;
                return self.step(ctx); // 未抵消：委托效果机器
            };
            if self.asked {
                self.asked = false;
                self.queue.remove(0);
                let answer = ctx.answer().clone();
                if let Some(ChoiceOption::Card(c)) = answer.first().copied() {
                    // 出无懈：弃牌、反转抵消状态、自其下家重新询问
                    ctx.discard_from_hand(responder, c);
                    ctx.emit(EventData::CardResponded {
                        card: c,
                        player: responder,
                    });
                    self.nullified = !self.nullified;
                    self.queue = ctx.core.seat_order_from(ctx.core.next_alive(responder));
                }
                return self.step(ctx);
            }
            self.asked = true;
            let mut options = vec![ChoiceOption::Pass];
            options.extend(
                ctx.core
                    .player(responder)
                    .hand
                    .iter()
                    .filter(|c| ctx.catalog.card_kind(**c) == CardKind::Nullify)
                    .copied()
                    .map(ChoiceOption::Card),
            );
            return Flow::WaitChoice {
                observer: responder,
                spec: ChoiceSpec {
                    kind: ChoiceKind::Confirm,
                    prompt: "nullify_window",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            };
        }
        // 窗口已关闭且未抵消：全权委托效果机器
        if let Some(inner) = self.effect.as_mut() {
            let flow = inner.step(ctx);
            if matches!(flow, Flow::Done) {
                self.effect = None;
                return Flow::Done;
            }
            return flow;
        }
        Flow::Done
    }
}

/// 为锦囊构造无懈窗口并预载询问队列（无懈持有者过滤在运行时进行）。
pub(crate) fn wrap_nullify_window(
    ctx: &Ctx<'_>,
    card: crate::ids::CardId,
    source: PlayerId,
    effect: Box<dyn crate::catalog::SkillMachine>,
) -> NullifyWindowMachine {
    // 询问队列自来源起按座位顺序（含来源本人）——对齐旧线 alive_from_seat(actor)。
    NullifyWindowMachine {
        card,
        source,
        nullified: false,
        queue: ctx.core.seat_order_from(source),
        asked: false,
        window_open: true,
        owns_disposal: true,
        effect: Some(effect),
    }
}

/// 延时锦囊置入机器（使用时无懈窗口未抵消后启动）：
/// 将桌面上的锦囊置入目标判定区；目标在窗口期间死亡则弃置
/// （同名牌限制已在命令期校验，窗口期无改判类技能，不复查）。
#[derive(Clone)]
pub struct PlaceDelayedMachine {
    pub(super) card: crate::ids::CardId,
    pub(super) target: PlayerId,
    pub(super) placed: bool,
}

impl PlaceDelayedMachine {
    pub(crate) fn new(card: crate::ids::CardId, target: PlayerId, placed: bool) -> Self {
        Self {
            card,
            target,
            placed,
        }
    }
}

impl crate::catalog::SkillMachine for PlaceDelayedMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }

    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::PlaceDelayed {
            card: self.card,
            target: self.target,
            placed: self.placed,
        }
    }

    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if self.placed {
            return Flow::Done;
        }
        self.placed = true;
        let card = self.card;
        ctx.core.table.retain(|c| *c != card);
        if !ctx.core.player(self.target).alive {
            ctx.to_discard(card);
            return Flow::Done;
        }
        ctx.core.player_mut(self.target).judge.push(card);
        Flow::Done
    }
}

/// 权限型能力（无触发时机，如「鬼道」「天妒」）的空机器：
/// 永不进入触发队列与挂起槽，启动即直接结束。
#[derive(Clone)]
pub struct NoopMachine;

impl NoopMachine {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl crate::catalog::SkillMachine for NoopMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(NoopMachine)
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::Noop
    }
    fn step(&mut self, _ctx: &mut Ctx<'_>) -> Flow {
        Flow::Done
    }
}
