use super::*;

// 激将（刘备）：主公技，出牌阶段限一次——请求一名已声明蜀势力的
// 其他角色打出一张杀（其确认后置入你的手牌，视为由你使用；
// 原生简化：获得后走正常出杀流程与次数校验，见 ADR 0760）。
#[derive(Clone)]
pub(in super::super) struct JijiangBehavior;

impl SkillBehavior for JijiangBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, _owner: PlayerId, _event: &EventData) -> bool {
        false
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(JijiangMachine {
            owner,
            stage: JijiangStage::HelperAsk,
            queue: None,
            helper: None,
            sha: None,
            inner: None,
        })
    }

    fn activate(&self, ctx: &QueryCtx<'_>, owner: PlayerId) -> Option<Box<dyn SkillMachine>> {
        // 资格（ADR 0776 对齐上游 zhuSkill + filter）：主公资格
        // （身份局须主公身份）+ 存在蜀势力角色。**不以暗手牌预筛**——
        // 有无杀由被询问者逐人应答时才知道。
        if !ctx.state.is_lord(owner) {
            return None;
        }
        let has_shu = ctx.state.alive_players().any(|p| {
            p != owner
                && ctx.state.effective_faction(ctx.catalog, p) == Some(crate::catalog::Faction::Shu)
        });
        if !has_shu {
            return None; // 无可请求的蜀势力角色
        }
        Some(Box::new(JijiangMachine {
            owner,
            stage: JijiangStage::HelperAsk,
            queue: None,
            helper: None,
            sha: None,
            inner: None,
        }))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in super::super) enum JijiangStage {
    /// 逐人询问蜀势力角色（挂起 = jijiang_respond；helper 记当前被问者）。
    HelperAsk,
    /// 消费 jijiang_target 应答：借牌直结算（内嵌 ShaResolve，视为使用）。
    TargetAsk,
}

pub(in super::super) struct JijiangMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) stage: JijiangStage,
    /// 逐人询问队列（ADR 0776）：`None` = 未开始；`Some(q)` = 剩余待问
    /// 的蜀势力角色（owner 下家起按座位序）；`Some(空)` = 全部拒绝。
    /// 不以暗手牌预筛。
    pub(in super::super) queue: Option<Vec<PlayerId>>,
    /// 当前被询问/已响应的蜀势力角色。
    pub(in super::super) helper: Option<PlayerId>,
    /// 借出的杀（TargetAsk 起生效，置于桌面）。
    pub(in super::super) sha: Option<CardId>,
    /// 直结算内嵌机器（Launch 后委托）。
    pub(in super::super) inner: Option<Box<dyn SkillMachine>>,
}

impl JijiangMachine {
    /// 询问队列：owner 下家起按座位序的蜀势力存活角色。
    fn build_queue(&self, ctx: &Ctx<'_>) -> Vec<PlayerId> {
        ctx.core
            .seat_order_from(ctx.core.next_alive(self.owner))
            .into_iter()
            .filter(|p| {
                *p != self.owner
                    && ctx.core.player(*p).alive
                    && ctx.core.effective_faction(ctx.catalog, *p)
                        == Some(crate::catalog::Faction::Shu)
            })
            .collect()
    }
}

impl SkillMachine for JijiangMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(Self {
            owner: self.owner,
            stage: self.stage,
            queue: self.queue.clone(),
            helper: self.helper,
            sha: self.sha,
            inner: self.inner.as_ref().map(|m| m.fork()),
        })
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::Jijiang {
            owner: self.owner,
            stage: match self.stage {
                JijiangStage::HelperAsk => JijiangStageSnap::HelperAsk,
                JijiangStage::TargetAsk => JijiangStageSnap::TargetAsk,
            },
            queue: self.queue.clone(),
            helper: self.helper,
            sha: self.sha,
            inner: self.inner.as_ref().map(|m| Box::new(m.snapshot())),
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // 内嵌直结算机器存在即全权委托（OptionalGate 同款）
        if let Some(inner) = self.inner.as_mut() {
            let flow = inner.step(ctx);
            if matches!(flow, Flow::Done) {
                self.inner = None;
                return Flow::Done;
            }
            return flow;
        }
        match self.stage {
            JijiangStage::HelperAsk => {
                // 本窗口首次进入：构建逐人询问队列
                if self.queue.is_none() {
                    self.queue = Some(self.build_queue(ctx));
                }
                // 消费当前被询问者应答（若有）
                if let Some(helper) = self.helper.take() {
                    let confirmed = ctx.answer().first() == Some(&ChoiceOption::Confirm);
                    let alive = ctx.core.player(helper).alive;
                    let has_sha = alive
                        && ctx
                            .core
                            .player(helper)
                            .hand
                            .iter()
                            .any(|c| ctx.catalog.card_kind(*c) == CardKind::Sha);
                    if confirmed && has_sha {
                        // 响应：借杀上桌面并询问目标（同 ADR 0760 直结算）
                        self.helper = Some(helper);
                        let sha = ctx
                            .core
                            .player(helper)
                            .hand
                            .iter()
                            .copied()
                            .find(|c| ctx.catalog.card_kind(*c) == CardKind::Sha)
                            .expect("has_sha checked above");
                        {
                            let p = ctx.core.player_mut(helper);
                            p.hand.retain(|x| *x != sha);
                        }
                        ctx.core.table.push(sha);
                        self.sha = Some(sha);
                        // 目标候选：其他存活且距离 ≤ 射程（借牌仍守目标合法性）
                        let candidates: Vec<ChoiceOption> = ctx
                            .core
                            .alive_players()
                            .filter(|p| {
                                *p != self.owner
                                    && ctx.core.computed_distance(ctx.catalog, self.owner, *p)
                                        <= ctx.core.attack_range(ctx.catalog, self.owner)
                            })
                            .map(ChoiceOption::Player)
                            .collect();
                        if candidates.is_empty() {
                            // 无合法目标：借牌归还 helper，问下一位
                            let c = self.sha.take().expect("sha set above");
                            ctx.core.table.retain(|x| *x != c);
                            ctx.core.player_mut(helper).hand.push(c);
                        } else {
                            self.stage = JijiangStage::TargetAsk;
                            return Flow::WaitChoice {
                                observer: self.owner,
                                spec: ChoiceSpec {
                                    kind: ChoiceKind::SelectPlayers,
                                    prompt: "jijiang_target",
                                    options: candidates,
                                    min: 1,
                                    max: 1,
                                    allow_cancel: true,
                                },
                            };
                        }
                    }
                    // Pass / 确认但无杀：继续问下一位
                }
                // 逐人询问下一位
                let queue = self.queue.as_mut().expect("queue built above");
                if let Some(pos) = queue.iter().position(|p| ctx.core.player(*p).alive) {
                    let next = queue.remove(pos);
                    self.helper = Some(next);
                    return Flow::WaitChoice {
                        observer: next,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::Confirm,
                            prompt: "jijiang_respond",
                            options: vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    };
                }
                // 全部拒绝：取消本次发动（上游语义——无人响应则使用取消，
                // 不消耗每回合一次）
                let jj = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::JI_JIANG);
                if let Some(d) = jj {
                    if let Some(inst) = ctx.core.skills.get_mut(&(self.owner, d)) {
                        inst.once_used = false;
                    }
                }
                Flow::Done
            }
            JijiangStage::TargetAsk => {
                // 消费 jijiang_target：借牌直结算（视为刘备使用杀）
                let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
                    // 弃权：借牌归还 helper
                    if let Some(helper) = self.helper {
                        if let Some(c) = self.sha.take() {
                            ctx.core.table.retain(|x| *x != c);
                            ctx.core.player_mut(helper).hand.push(c);
                        }
                    }
                    return Flow::Done;
                };
                let sha = self.sha.take().expect("sha set at TargetAsk");
                // card, source, target, nature, asked, armor_judged,
                // dodges_used, tieji_red, liuli_done, liuli_stage, liuli_card,
                // hujia_queue, hujia_holder, jiu_bonus, asking_qilin,
                // qilin_horse, asking_hanbing, asking_guanshi, guanshi_cards,
                // asking_qinglong, asking_cixiong, cixiong_done,
                // extra_targets, hanbing_pending
                // card, source, target, nature, asked, armor_judged,
                // dodges, tieji, liuli_done, liuli_stage, liuli_card,
                // hujia_queue, hujia_holder, jiu_bonus, asking_qilin,
                // qilin_horse, asking_hanbing, asking_fengxueren,
                // asking_guanshi, guanshi_cards, asking_qinglong,
                // asking_cixiong, cixiong_done, extra_targets,
                // hanbing_pending
                let machine = crate::kernel::ShaResolveMachine::new(
                    sha,
                    self.owner,
                    target,
                    crate::event::DamageNature::Normal,
                    false,
                    false,
                    0,
                    None,
                    false,
                    0,
                    None,
                    None,
                    None,
                    false,
                    false,
                    None,
                    false,
                    false,
                    false,
                    Vec::new(),
                    false,
                    false,
                    false,
                    Vec::new(),
                    0,
                    false,
                    false,
                    false,
                    false,
                    None,
                    false,
                    false,
                    None,
                    false,
                    false,
                    false,
                );
                self.inner = Some(Box::new(machine));
                let flow = self.inner.as_mut().expect("just set").step(ctx);
                if matches!(flow, Flow::Done) {
                    self.inner = None;
                    return Flow::Done;
                }
                flow
            }
        }
    }
}
