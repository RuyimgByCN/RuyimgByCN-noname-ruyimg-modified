//! VM 技能机器（VmMachine）的执行、挂起恢复、快照与提交窗口。

use super::*;

/// 效果提交（`accept_effect` 已验证映射面；此处只落账）。

impl VmMachine {
    pub(crate) fn with_commit_state(
        mut self,
        pending: &[u8],
        state: Vec<u32>,
        committing: bool,
    ) -> Result<Self, ()> {
        if committing {
            self.pending_effects = dec_effects(pending).ok_or(())?;
        }
        self.committing = committing;
        self.commit_state = state;
        Ok(self)
    }

    pub(crate) fn with_following(mut self, following: Vec<MachineSnapshot>) -> Self {
        self.following = following;
        self
    }

    /// 存档重建（非挂起态）：从模块载体字节还原（快照点不会停在
    /// start→step 中段——此态重建后即待触发执行）。PA06：装载期
    /// 校验——恢复载体畸形即拒（等价于原执行期校验的前移）。
    /// B7（规则 v324）：技能身份/触发语境/限次/命中旗标从快照还原，
    /// 不再从模块名重建或重置。
    pub(crate) fn from_module(
        owner: PlayerId,
        module: BytecodeModule,
        skill_name: String,
        event: EventData,
        consume: Vec<(String, u8)>,
        count_hits: bool,
    ) -> Result<Self, ()> {
        if skill_name.is_empty() {
            return Err(());
        }
        let program = VmSkillProgram::new(module, skill_name).map_err(|_| ())?;
        Ok(Self {
            owner,
            program: Arc::new(program),
            trig: trigger_ctx_of(&event),
            event,
            pending_effects: Vec::new(),
            paused: None,
            asked: false,
            settled: false,
            consume,
            count_hits,
            commit_pause: None,
            following: Vec::new(),
            commit_state: Vec::new(),
            committing: false,
        })
    }

    /// 存档重建（挂起态）：continuation + 选项 + 排队效果还原；内容锁
    /// 核对（continuation 绑定模块 logic_hash，换模块续跑被拒）。
    /// B7（规则 v324）：声明技能名、触发事件、限次清单与命中旗标一并
    /// 从快照还原——此前技能名取模块名、触发语境重置为默认，续行的
    /// STORAGE_SET/once/hits 键与故障身份偏离未中断执行。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn restore_paused(
        owner: PlayerId,
        module: BytecodeModule,
        frames: Vec<u8>,
        pc: u32,
        fuel_remaining: u32,
        id: u64,
        module_hash: [u8; 32],
        options: Vec<u32>,
        pending_bytes: Vec<u8>,
        skill_name: String,
        event: EventData,
        consume: Vec<(String, u8)>,
        count_hits: bool,
    ) -> Result<Self, ()> {
        if module.logic_hash != module_hash {
            return Err(());
        }
        if skill_name.is_empty() {
            return Err(());
        }
        let pending = dec_effects(&pending_bytes).ok_or(())?;
        let program = VmSkillProgram::new(module, skill_name).map_err(|_| ())?;
        Ok(Self {
            owner,
            program: Arc::new(program),
            trig: trigger_ctx_of(&event),
            event,
            pending_effects: pending,
            paused: Some(VmPaused {
                continuation: Continuation {
                    id,
                    frames,
                    pc,
                    fuel_remaining,
                    module_hash,
                },
                options,
            }),
            // 挂起态快照 = 问询已发出（active_ask 派生同一询问）——下一
            // 应答即续跑，不重问。
            asked: true,
            settled: false,
            consume,
            count_hits,
            commit_pause: None,
            following: Vec::new(),
            commit_state: Vec::new(),
            committing: false,
        })
    }

    /// J1 存档重建（提交期挂起）：剩余队列 + 开窗参数还原。问询由
    /// active_ask 按推进后状态派生，`asked=true`——下一应答即读答继续
    /// 落账，不重问。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn restore_commit_paused(
        owner: PlayerId,
        module: BytecodeModule,
        commit_pending_bytes: Vec<u8>,
        player: PlayerId,
        count: u32,
        skill_name: String,
        event: EventData,
        consume: Vec<(String, u8)>,
        count_hits: bool,
    ) -> Result<Self, ()> {
        if skill_name.is_empty() {
            return Err(());
        }
        let pending = dec_effects(&commit_pending_bytes).ok_or(())?;
        if pending.is_empty() {
            return Err(());
        }
        let program = VmSkillProgram::new(module, skill_name).map_err(|_| ())?;
        Ok(Self {
            owner,
            program: Arc::new(program),
            trig: trigger_ctx_of(&event),
            event,
            pending_effects: pending,
            paused: None,
            asked: true,
            settled: false,
            consume,
            count_hits,
            commit_pause: Some(VmCommitPause { player, count }),
            following: Vec::new(),
            commit_state: Vec::new(),
            committing: false,
        })
    }
}

pub(super) fn enc_effects(w: &mut Vec<u8>, effects: &[EffectRequest]) {
    crate::rvm_ops::enc_effects(w, effects);
}

pub(super) fn dec_effects(r: &[u8]) -> Option<Vec<EffectRequest>> {
    crate::rvm_ops::dec_effects(r)
}

impl SkillMachine for VmMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }

    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::Rvm {
            owner: self.owner,
            module: self.program.module.to_bytes(),
            paused: self.paused.as_ref().map(|p| crate::snap::RvmPaused {
                frames: p.continuation.frames.clone(),
                pc: p.continuation.pc,
                fuel_remaining: p.continuation.fuel_remaining,
                id: p.continuation.id,
                module_hash: p.continuation.module_hash,
                options: p.options.clone(),
                pending: {
                    let mut b = Vec::new();
                    enc_effects(&mut b, &self.pending_effects);
                    b
                },
            }),
            skill_name: self.program.skill_name.clone(),
            event: self.event.clone(),
            consume: self.consume.clone(),
            count_hits: self.count_hits,
            // J1：提交期挂起——剩余队列与开窗参数（未挂起为空/None）。
            commit_pending: {
                if self.committing {
                    let mut b = Vec::new();
                    enc_effects(&mut b, &self.pending_effects);
                    b
                } else {
                    Vec::new()
                }
            },
            following: self.following.clone(),
            commit_state: self.commit_state.clone(),
            committing: self.committing,
            commit_pick: self.commit_pause.as_ref().map(|cp| (cp.player, cp.count)),
        }
    }

    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if self.settled {
            return Flow::Done;
        }
        // J1 提交期挂起：先问（WaitChoice），已答则按所选弃置并继续落账
        // （机器契约：WaitChoice 后下一 step 读应答）。
        if let Some(cp) = self.commit_pause.clone() {
            if !self.asked {
                let exact = commit_window_exact(ctx, cp.player, cp.count);
                if exact == 0 {
                    // 无可弃牌：开窗效果空过（与旧近似「0 张尽弃」同收尾）。
                    self.commit_pause = None;
                    let _ = self.pending_effects.remove(0);
                    return self.pump_commit(ctx);
                }
                self.asked = true;
                let options = commit_window_options(ctx, cp.player);
                return Flow::WaitChoice {
                    observer: cp.player,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "rvm_discard_pick",
                        options,
                        min: exact,
                        max: exact,
                        allow_cancel: false,
                    },
                };
            }
            self.asked = false;
            self.commit_pause = None;
            // 开窗效果完成（无论空过与否都已消费头元素）。
            let effect = self.pending_effects.remove(0);
            let picked: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .collect();
            if !discard_picked(ctx, cp.player, &effect, picked) {
                // 防御：应答与开窗不符（引擎按 options/min/max 校验，正常
                // 不可达）——整段回滚，半次效果不落账。
                return self.trap_done();
            }
            // Settle discard children before advancing queued effects or following handlers.
            return Flow::Yield;
        }
        if self.committing {
            return self.pump_commit(ctx);
        }
        // 挂起态：先问（WaitChoice），已答则续跑
        if let Some(p) = self.paused.clone() {
            if !self.asked {
                self.asked = true;
                let options: Vec<ChoiceOption> = p
                    .options
                    .iter()
                    .map(|t| ChoiceOption::Tag(*t as u8))
                    .collect();
                return Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::PickTag,
                        prompt: "rvm_choice",
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                };
            }
            self.asked = false;
            self.paused = None;
            let token = match ctx.answer().first() {
                Some(ChoiceOption::Tag(t)) => *t as u32,
                _ => return self.trap_done(),
            };
            let program = self.program.clone();
            let cont = p.continuation;
            let resumed = move |vm: &mut RuleVm, host: &mut EngineVmHost<'_>| {
                vm.resume(&program.module, host, cont, token)
            };
            return self.run(ctx, resumed);
        }
        let program = self.program.clone();
        let module = program.module.clone();
        let executed = move |vm: &mut RuleVm, host: &mut EngineVmHost<'_>| {
            vm.execute(&module, host, DEFAULT_FUEL)
        };
        self.run(ctx, executed)
    }
}

impl VmMachine {
    /// 执行/续跑公共尾：Halted 提交排队效果；Paused 转挂起询问（排队
    /// 效果保留至续跑后一并落账）；Err 回滚（排队效果整批丢弃）。
    fn run(
        &mut self,
        ctx: &mut Ctx<'_>,
        f: impl FnOnce(&mut RuleVm, &mut EngineVmHost<'_>) -> Result<ExecResult, rule_vm::VmError>,
    ) -> Flow {
        let seats = ctx.core.players.len() as u8;
        let damage = ctx.current_damage().ok().cloned();
        let outcome = {
            let targets = self
                .trig
                .targets
                .iter()
                .map(|t| to_rule_pid(*t).raw())
                .collect::<Vec<_>>();
            let selected_cards = ctx.core.vm_aux.selected_cards(self.owner);
            let mut selected_targets = ctx.core.vm_aux.selected_targets(self.owner);
            if selected_targets.is_empty() {
                if let Some(&at) = ctx
                    .core
                    .vm_aux
                    .flags
                    .get(&(
                        self.owner,
                        format!("active_target:{}", self.program.skill_name),
                    ))
                    .or_else(|| {
                        ctx.core
                            .vm_aux
                            .flags
                            .get(&(self.owner, "active_target".to_string()))
                    })
                {
                    if at != 0 {
                        selected_targets.push(at);
                    }
                }
            }
            let event_source = if let Some(d) = damage.as_ref() {
                d.source.map(|p| to_rule_pid(p).raw()).unwrap_or(0)
            } else if matches!(
                self.event,
                EventData::Phase {
                    phase: crate::event::Phase::Play,
                    begin: true,
                    ..
                }
            ) && !selected_targets.is_empty()
            {
                selected_targets[0]
            } else {
                self.trig
                    .event_source
                    .map(|p| to_rule_pid(p).raw())
                    .or_else(|| selected_targets.first().copied())
                    .unwrap_or(0)
            };
            let fallback_judge_card = if matches!(self.event, EventData::OrdinaryJudge { .. })
                && self.trig.event_card.is_none()
            {
                ctx.core.discard.last().copied()
            } else {
                None
            };
            let mut cards = self.trig.event_cards.clone();
            if cards.is_empty() {
                if let Some(c) = fallback_judge_card {
                    cards.push(c);
                }
            }
            let mut host = EngineVmHost {
                targets,
                cards,
                selected_cards,
                selected_targets,
                ctx: HostContext {
                    // T8 增量 8：skill_id 经目录反查回填（查不到保持 0=NONE）。
                    skill_id: ctx
                        .catalog
                        .skill_by_name(&self.program.skill_name)
                        .map(|d| d as u32)
                        .unwrap_or(0),
                    self_id: to_rule_pid(self.owner).raw(),
                    event_player: damage
                        .as_ref()
                        .map(|d| d.target)
                        .or(self.trig.event_player)
                        .map(to_rule_pid)
                        .map(|p| p.raw())
                        .unwrap_or(0),
                    event_source,
                    event_card: damage
                        .as_ref()
                        .map(|d| d.card)
                        .unwrap_or(self.trig.event_card.or(fallback_judge_card))
                        .unwrap_or(0),
                    event_amount: damage
                        .as_ref()
                        .map(|d| d.amount as u32)
                        .unwrap_or(self.trig.event_amount),
                },
                seats,
                core: ctx.core,
                catalog: ctx.catalog,
                pending: &mut self.pending_effects,
            };
            let mut vm = RuleVm::new();
            f(&mut vm, &mut host)
        };
        match outcome {
            Ok(ExecResult::Halted { .. }) => {
                self.committing = true;
                for (key, scope) in &self.consume {
                    let k = if key.is_empty() {
                        self.program.skill_name.clone()
                    } else {
                        key.clone()
                    };
                    ctx.core.vm_aux.once.insert((self.owner, k, *scope));
                }
                if self.count_hits && matches!(self.trig.kind, 1 | 2) {
                    *ctx.core
                        .vm_aux
                        .hits
                        .entry((self.owner, self.program.skill_name.clone()))
                        .or_default() += 1;
                }
                // J1：落账改走提交泵——遇开窗效果转入提交期挂起（真实
                // 选牌窗），否则逐效果落账至队列耗尽。
                self.pump_commit(ctx)
            }
            Ok(ExecResult::Paused {
                continuation,
                options,
            }) => {
                // 选项超 u8 域（ChoiceOption::Tag 容量）→ 回滚。
                if options.iter().any(|t| *t > 255) {
                    return self.trap_done();
                }
                self.paused = Some(VmPaused {
                    continuation,
                    options,
                });
                // 本次返回的 WaitChoice 就是问询（机器契约：WaitChoice 后下一
                // step 读应答）。PA12④ 原型实证：此处若置 false，下一 step 会
                // 再问一次、首答被丢（恢复路径 restore_paused 早已置 true）。
                self.asked = true;
                let options: Vec<ChoiceOption> = self
                    .paused
                    .as_ref()
                    .expect("just set")
                    .options
                    .iter()
                    .map(|t| ChoiceOption::Tag(*t as u8))
                    .collect();
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::PickTag,
                        prompt: "rvm_choice",
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                }
            }
            Err(e) => {
                // NC04：故障必须可定位并向上传播——记录技能/座位/事件/错误类别，
                // 由调度循环（StepOutcome::Fault）整段回滚并以 VmFault 拒绝命令。
                ctx.res.fault = Some(crate::machine::VmFault {
                    skill: self.program.skill_name.clone(),
                    owner: self.owner,
                    event: format!("{:?}", self.trig.kind),
                    kind: match &e {
                        rule_vm::VmError::OutOfFuel => "OutOfFuel",
                        rule_vm::VmError::StackOverflow => "StackOverflow",
                        rule_vm::VmError::MemoryLimit => "MemoryLimit",
                        rule_vm::VmError::InvalidBytecode(_) => "InvalidBytecode",
                        rule_vm::VmError::CapabilityDenied(_) => "CapabilityDenied",
                        rule_vm::VmError::Host(_) => "Host",
                        rule_vm::VmError::Arithmetic(_) => "Arithmetic",
                        rule_vm::VmError::BadControlFlow(_) => "BadControlFlow",
                        rule_vm::VmError::Stack(_) => "Stack",
                    }
                    .to_string(),
                    detail: e.to_string(),
                });
                self.trap_done()
            }
        }
    }

    /// J1 提交泵：顺序落账排队效果；队头是开窗型效果（`discard_hand_equip_count`
    /// N>0 且开窗座位有牌）时转入提交期挂起并经 `step` 的开窗分支发出问询。
    /// once/命中计数已在 Halted 尾写入（幂等或一次性，恢复续跑不重记）。
    fn pump_commit(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let seats = ctx.core.players.len() as u8;
        if !self.pending_effects.is_empty() {
            let hook = ctx.catalog.hooks.vm_commit;
            if let Some(flow) = hook(
                ctx,
                &self.program.skill_name,
                &self.pending_effects[0],
                &mut self.commit_state,
            ) {
                if matches!(flow, Flow::Done) {
                    self.pending_effects.remove(0);
                    self.commit_state.clear();
                    return Flow::Yield;
                }
                return flow;
            }
            let window = match &self.pending_effects[0] {
                EffectRequest::DiscardHandEquipCount { player, count, .. } if *count > 0 => {
                    match crate::rvm_ops::seat_of(*player, seats) {
                        Some(p) if commit_window_exact(ctx, p, *count) > 0 => Some((p, *count)),
                        _ => None,
                    }
                }
                _ => None,
            };
            if let Some((player, count)) = window {
                self.commit_pause = Some(VmCommitPause { player, count });
                self.asked = false;
                return self.step(ctx);
            }
            let e = self.pending_effects.remove(0);
            let skill = self.program.skill_name.clone();
            crate::rvm_ops::commit_effect(ctx, &skill, self.owner, e);
            return Flow::Yield;
        }
        if !self.following.is_empty() {
            let next = self.following.remove(0);
            let MachineSnapshot::Rvm {
                owner,
                module,
                skill_name,
                event,
                consume,
                count_hits,
                ..
            } = next
            else {
                return self.trap_done();
            };
            let Some(def) = ctx.catalog.skill_by_name(&skill_name) else {
                return self.trap_done();
            };
            let instance_ok = ctx.core.player(owner).alive
                && ctx
                    .core
                    .skills
                    .get(&(owner, def))
                    .is_some_and(|instance| instance.enabled && instance.owner == owner);
            if !instance_ok {
                return self.pump_commit(ctx);
            }
            if !ctx.catalog.skills[def as usize]
                .behavior
                .vm_trigger_still_matches(&module, &ctx.query(), owner, &event, ctx.res.stack.last().map(crate::resolution::trigger_window_kind).unwrap_or_else(|| event.kind()))
            {
                return self.pump_commit(ctx);
            }
            let Ok(module) = BytecodeModule::from_bytes(&module) else {
                return self.trap_done();
            };
            let Ok(mut next) =
                Self::from_module(owner, module, skill_name, event, consume, count_hits)
            else {
                return self.trap_done();
            };
            next.following = std::mem::take(&mut self.following);
            *self = next;
            return Flow::Yield;
        }
        self.settled = true;
        Flow::Done
    }

    /// 回滚收尾：排队效果整批丢弃（半次效果不落账，原型同款）。
    fn trap_done(&mut self) -> Flow {
        self.pending_effects.clear();
        self.paused = None;
        self.settled = true;
        Flow::Done
    }
}

/// J1：开窗可弃牌数（手牌+装备，不足以 count 封顶）。
pub(super) fn commit_window_exact(ctx: &Ctx<'_>, player: PlayerId, count: u32) -> usize {
    let p = ctx.core.player(player);
    let available = p.hand.len() + p.equips.len();
    (count as usize).min(available)
}

/// J1：开窗候选（手牌在前、装备在后——与旧近似的区优先序一致）。
pub(super) fn commit_window_options(ctx: &Ctx<'_>, player: PlayerId) -> Vec<ChoiceOption> {
    let p = ctx.core.player(player);
    p.hand
        .iter()
        .chain(p.equips.iter())
        .map(|c| ChoiceOption::Card(*c))
        .collect()
}

/// J1：按所选弃置（先从所属区移除再入弃堆，`discard_some` 同口径）。
/// 返回 false = 应答与开窗不符（防御路径，正常不可达）。
pub(super) fn discard_picked(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
    effect: &EffectRequest,
    picked: Vec<CardId>,
) -> bool {
    let expect = match effect {
        EffectRequest::DiscardHandEquipCount { count, .. } => {
            commit_window_exact(ctx, player, *count)
        }
        _ => return false,
    };
    if picked.len() != expect || picked.is_empty() {
        return false;
    }
    for c in &picked {
        let in_hand = ctx.core.player(player).hand.contains(c);
        let in_equip = ctx.core.player(player).equips.contains(c);
        if !in_hand && !in_equip {
            return false;
        }
    }
    for c in &picked {
        if ctx.core.player(player).hand.contains(c) {
            ctx.take_hand_card(player, *c);
        } else {
            ctx.take_owned_card(player, *c);
        }
    }
    crate::rvm_ops::discard_some(ctx, player, picked);
    true
}
