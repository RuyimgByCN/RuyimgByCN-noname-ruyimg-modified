//! T8 增量 1（ADR 0782 定案后的引擎接线）：engine-native 触发调度接
//! rule-vm——扩展包数据声明的 VM 技能（`VmSkillDecl`）经目录 SkillDef
//! 进入行为总线，触发时以内核状态为只读查询面执行字节码，效果按原型
//! 同款**两层提交协议**落账：
//!
//! - VM 排队效果仅在 `Halted` 时提交（recover/draw/damage 映射）；
//! - `Paused`（选择挂起）本增量未接——排队效果不落账（增量 2）；
//! - `Err`（fuel 耗尽/trap）排队效果整批丢弃——半次效果不落账（回滚）。
//!   两种情况在事件面表现为「无效果落账」（可断言）。
//!
//! 查询面增量 1 只映射 `Hp`/`MaxHp`/`Distance`；未映射查询/效果在
//! emit/query 时即返回 `InvalidRequest`（模块 trap → 回滚协议）。
//!
//! 玩家 id 映射：rule-api `PlayerId` 为 1 基（`NONE=0` 哨兵），内核座位
//! 为 0 基——桥内统一 `内核座位 s ↔ rule_id = s+1`。

use std::sync::Arc;

use rule_api::{CapabilitySet, EffectRequest, HostTrap, QueryRequest, QueryResponse};
use rule_vm::{BytecodeModule, Continuation, ExecResult, HostContext, RuleVm, VmHost, DEFAULT_FUEL};

use crate::catalog::{SkillBehavior, SkillMachine};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::{DamageNature, EventData};
use crate::ids::{CardId, PlayerId};
use crate::catalog::QueryCtx;
use crate::machine::{damage_data_with, Ctx, Flow};
use crate::snap::MachineSnapshot;

/// 数据驱动的 VM 技能程序（扩展包声明携带；目录 def 的 behavior 持有）。
#[derive(Debug, Clone)]
pub struct VmSkillProgram {
    /// 已验证字节码模块（装载/编译产物；`logic_hash` 为内容锁锚）。
    pub module: BytecodeModule,
}

impl VmSkillProgram {
    /// 内容锁锚（模块逻辑哈希）。
    pub fn logic_hash(&self) -> [u8; 32] {
        self.module.logic_hash
    }
}

/// VM 技能触发时捕获的宿主上下文（从触发事件提取）。
#[derive(Debug, Clone, Copy, Default)]
struct VmTriggerCtx {
    event_player: Option<PlayerId>,
    event_source: Option<PlayerId>,
    event_card: Option<CardId>,
    event_amount: u32,
}

fn trigger_ctx_of(event: &EventData) -> VmTriggerCtx {
    match event {
        EventData::Damage(d) => VmTriggerCtx {
            event_player: Some(d.target),
            event_source: d.source,
            event_card: d.card,
            event_amount: d.amount,
        },
        EventData::Phase { player, .. } => VmTriggerCtx {
            event_player: Some(*player),
            ..VmTriggerCtx::default()
        },
        EventData::CardUsed(u) => VmTriggerCtx {
            event_player: Some(u.player),
            event_card: Some(u.card),
            ..VmTriggerCtx::default()
        },
        EventData::Death { player, killer } => VmTriggerCtx {
            event_player: Some(*player),
            event_source: *killer,
            ..VmTriggerCtx::default()
        },
        _ => VmTriggerCtx::default(),
    }
}

/// DSL filter 映射（增量 6）：三基语义严格落；具名组合 filter 由装载
/// 器跳过登记（deferred）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VmFilterPolicy {
    /// DSL `owner_is_target`：事件主玩家=持有者。
    #[default]
    OwnerIsTarget,
    /// DSL `owner_is_source`：事件来源=持有者。
    OwnerIsSource,
    /// DSL `always`：无条件。
    Always,
}

/// VM 技能行为：触发即执行模块（事件门由 def 的 TriggerMeta、行为总线
/// 与 filter 策略共同承载）。
#[derive(Debug, Clone)]
pub struct VmSkillBehavior {
    pub program: Arc<VmSkillProgram>,
    pub filter: VmFilterPolicy,
}

impl SkillBehavior for VmSkillBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        // 增量 6 过滤面：三基语义（owner_is_target/owner_is_source/
        // always）；其余具名组合 filter 装载器跳过（deferred 登记）。
        let trig = trigger_ctx_of(event);
        match self.filter {
            VmFilterPolicy::OwnerIsTarget => trig.event_player == Some(owner),
            VmFilterPolicy::OwnerIsSource => trig.event_source == Some(owner),
            VmFilterPolicy::Always => true,
        }
    }

    fn start(&self, owner: PlayerId, event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(VmMachine {
            owner,
            program: self.program.clone(),
            trig: trigger_ctx_of(event),
            pending_effects: Vec::new(),
            paused: None,
            asked: false,
            settled: false,
        })
    }
}

/// VM 挂起态（HOST_CHOICE）：continuation + 选项 tokens。
#[derive(Debug, Clone)]
pub struct VmPaused {
    pub continuation: Continuation,
    pub options: Vec<u32>,
}

/// VM 技能机器：可挂起（HOST_CHOICE）——挂起期间排队效果保留（原型
/// 同款「Paused 层保留至续跑后落账」），存档快照携带模块字节+挂起态+
/// 排队效果（增量 2）。
#[derive(Debug, Clone)]
pub struct VmMachine {
    owner: PlayerId,
    program: Arc<VmSkillProgram>,
    trig: VmTriggerCtx,
    /// 排队未落账效果（Halted 提交 / Paused 保留 / Err 丢弃）。
    pending_effects: Vec<EffectRequest>,
    /// 挂起态（含已捕获 continuation 与选项）。
    paused: Option<VmPaused>,
    /// 挂起询问已答（下一步续跑）。
    asked: bool,
    settled: bool,
}

/// 内核座位 ↔ rule-api id（1 基，NONE=0）。
fn to_rule_pid(p: PlayerId) -> rule_api::PlayerId {
    rule_api::PlayerId::new(p as u32 + 1)
}

fn from_rule_pid(p: rule_api::PlayerId) -> Option<PlayerId> {
    if !p.is_some() {
        return None;
    }
    let raw = p.raw() - 1;
    u8::try_from(raw).ok()
}

/// 未映射查询/效果：立即拒绝（模块 trap → 回滚协议，绝不半提交）。
fn reject_unmapped(name: &str) -> HostTrap {
    HostTrap::InvalidRequest {
        message: format!("rvm_bridge 增量1 未映射: {name}"),
    }
}

/// 内核只读查询 + 排队效果的 VmHost 适配器。
struct EngineVmHost<'a> {
    ctx: HostContext,
    seats: u8,
    core: &'a crate::state::GameStateCore,
    catalog: &'a crate::catalog::Catalog,
    pending: &'a mut Vec<EffectRequest>,
}

impl VmHost for EngineVmHost<'_> {
    fn capabilities(&self) -> &CapabilitySet {
        // 标准技能能力面（emit/查询/选择申请）；选择挂起本增量未接。
        static CAPS: std::sync::OnceLock<CapabilitySet> = std::sync::OnceLock::new();
        CAPS.get_or_init(CapabilitySet::standard_skill)
    }

    fn context(&self) -> HostContext {
        self.ctx
    }

    fn emit_effect(&mut self, req: EffectRequest) -> Result<(), HostTrap> {
        // 增量 1 映射面校验（玩家界内 + 值合法），未映射变体拒绝。
        match &req {
            EffectRequest::RecoverHp { player, amount, .. } => {
                let p = from_rule_pid(*player)
                    .ok_or_else(|| reject_unmapped("RecoverHp.NONE"))?;
                if p >= self.seats || *amount == 0 {
                    return Err(reject_unmapped("RecoverHp 越界/零量"));
                }
            }
            EffectRequest::DrawCards { player, count } => {
                let p = from_rule_pid(*player)
                    .ok_or_else(|| reject_unmapped("DrawCards.NONE"))?;
                if p >= self.seats || *count == 0 {
                    return Err(reject_unmapped("DrawCards 越界/零量"));
                }
            }
            EffectRequest::DealDamage { target, amount, nature, .. } => {
                let t = from_rule_pid(*target)
                    .ok_or_else(|| reject_unmapped("DealDamage.NONE"))?;
                if t >= self.seats || *amount == 0 || *nature > 2 {
                    return Err(reject_unmapped("DealDamage 越界/nature"));
                }
            }
            other => {
                let name = format!("{other:?}");
                return Err(reject_unmapped(name.split([' ', '(', '{']).next().unwrap_or("?")));
            }
        }
        self.pending.push(req);
        Ok(())
    }

    fn run_query(&mut self, req: QueryRequest) -> Result<QueryResponse, HostTrap> {
        match req {
            QueryRequest::Hp { player } => {
                let p = from_rule_pid(player).ok_or_else(|| reject_unmapped("Hp.NONE"))?;
                if p >= self.seats {
                    return Err(reject_unmapped("Hp 越界"));
                }
                Ok(QueryResponse::Int(self.core.player(p).hp as i32))
            }
            QueryRequest::MaxHp { player } => {
                let p = from_rule_pid(player).ok_or_else(|| reject_unmapped("MaxHp.NONE"))?;
                if p >= self.seats {
                    return Err(reject_unmapped("MaxHp 越界"));
                }
                Ok(QueryResponse::Int(self.core.player(p).max_hp as i32))
            }
            QueryRequest::Distance { from, to } => {
                let a = from_rule_pid(from).ok_or_else(|| reject_unmapped("Distance.NONE"))?;
                let b = from_rule_pid(to).ok_or_else(|| reject_unmapped("Distance.NONE"))?;
                if a >= self.seats || b >= self.seats {
                    return Err(reject_unmapped("Distance 越界"));
                }
                Ok(QueryResponse::Int(
                    self.core.computed_distance(self.catalog, a, b) as i32,
                ))
            }
            other => {
                let name = format!("{other:?}");
                Err(reject_unmapped(name.split([' ', '(', '{']).next().unwrap_or("?")))
            }
        }
    }
}

/// 效果提交（emit_effect 已验证映射面；此处只落账）。
fn commit_effect(ctx: &mut Ctx<'_>, e: EffectRequest) {
    match e {
        EffectRequest::RecoverHp { player, amount, .. } => {
            if let Some(p) = from_rule_pid(player) {
                ctx.recover(p, amount, None);
            }
        }
        EffectRequest::DrawCards { player, count } => {
            if let Some(p) = from_rule_pid(player) {
                ctx.draw_cards(p, count as usize);
            }
        }
        EffectRequest::DealDamage { source, target, amount, is_lose_hp, nature } => {
            let Some(t) = from_rule_pid(target) else {
                return;
            };
            let src = if source.is_some() {
                from_rule_pid(source)
            } else {
                None
            };
            if is_lose_hp {
                ctx.emit(EventData::LoseHp { player: t, amount });
            } else {
                let nature = match nature {
                    1 => DamageNature::Fire,
                    2 => DamageNature::Thunder,
                    _ => DamageNature::Normal,
                };
                ctx.emit(EventData::Damage(damage_data_with(
                    src, t, amount, nature, None,
                )));
            }
        }
        _ => {}
    }
}

impl VmMachine {
    /// 存档重建（非挂起态）：从模块载体字节还原（快照点不会停在
    /// start→step 中段——此态重建后即待触发执行）。
    pub(crate) fn from_module(owner: PlayerId, module: BytecodeModule) -> Self {
        Self {
            owner,
            program: Arc::new(VmSkillProgram { module }),
            trig: VmTriggerCtx::default(),
            pending_effects: Vec::new(),
            paused: None,
            asked: false,
            settled: false,
        }
    }

    /// 存档重建（挂起态）：continuation + 选项 + 排队效果还原；内容锁
    /// 核对（continuation 绑定模块 logic_hash，换模块续跑被拒）。
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
    ) -> Result<Self, ()> {
        if module.logic_hash != module_hash {
            return Err(());
        }
        let pending = dec_effects(&pending_bytes).ok_or(())?;
        Ok(Self {
            owner,
            program: Arc::new(VmSkillProgram { module }),
            trig: VmTriggerCtx::default(),
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
        })
    }
}

/// 排队效果自定义编码（映射面仅三种；emit_effect 已验证）。
fn enc_effects(w: &mut Vec<u8>, effects: &[EffectRequest]) {
    fn u32le(w: &mut Vec<u8>, v: u32) {
        w.extend_from_slice(&v.to_le_bytes());
    }
    w.extend_from_slice(&(effects.len() as u32).to_le_bytes());
    for e in effects {
        match e {
            EffectRequest::RecoverHp { source, player, amount } => {
                w.push(1);
                u32le(w, source.raw());
                u32le(w, player.raw());
                u32le(w, *amount);
            }
            EffectRequest::DrawCards { player, count } => {
                w.push(2);
                u32le(w, player.raw());
                u32le(w, *count);
            }
            EffectRequest::DealDamage { source, target, amount, is_lose_hp, nature } => {
                w.push(3);
                u32le(w, source.raw());
                u32le(w, target.raw());
                u32le(w, *amount);
                w.push(u8::from(*is_lose_hp));
                w.push(*nature);
            }
            _ => unreachable!("emit_effect 面已拒未映射变体"),
        }
    }
}

/// 逐段 u32 读取器（dec_effects 用）。
struct U32Reader<'a> {
    r: &'a [u8],
}

impl U32Reader<'_> {
    fn next_u32(&mut self) -> Option<u32> {
        if self.r.len() < 4 {
            return None;
        }
        let (a, rest) = self.r.split_at(4);
        self.r = rest;
        Some(u32::from_le_bytes(a.try_into().unwrap()))
    }
    fn next_u8(&mut self) -> Option<u8> {
        let v = *self.r.first()?;
        self.r = &self.r[1..];
        Some(v)
    }
}

fn dec_effects(mut r: &[u8]) -> Option<Vec<EffectRequest>> {
    let mut rd = U32Reader { r: &mut r };
    let n = rd.next_u32()? as usize;
    let mut out = Vec::new();
    for _ in 0..n {
        let tag = rd.next_u8()?;
        match tag {
            1 => {
                let source = rd.next_u32()?;
                let player = rd.next_u32()?;
                let amount = rd.next_u32()?;
                out.push(EffectRequest::RecoverHp {
                    source: rule_api::PlayerId::new(source),
                    player: rule_api::PlayerId::new(player),
                    amount,
                });
            }
            2 => {
                let player = rd.next_u32()?;
                let count = rd.next_u32()?;
                out.push(EffectRequest::DrawCards {
                    player: rule_api::PlayerId::new(player),
                    count,
                });
            }
            3 => {
                let source = rd.next_u32()?;
                let target = rd.next_u32()?;
                let amount = rd.next_u32()?;
                let is_lose_hp = rd.next_u8()? == 1;
                let nature = rd.next_u8()?;
                out.push(EffectRequest::DealDamage {
                    source: rule_api::PlayerId::new(source),
                    target: rule_api::PlayerId::new(target),
                    amount,
                    is_lose_hp,
                    nature,
                });
            }
            _ => return None,
        }
    }
    if !rd.r.is_empty() {
        return None;
    }
    Some(out)
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
        }
    }

    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if self.settled {
            return Flow::Done;
        }
        // 挂起态：先问（WaitChoice），已答则续跑
        if let Some(p) = self.paused.clone() {
            if !self.asked {
                self.asked = true;
                let options: Vec<ChoiceOption> =
                    p.options.iter().map(|t| ChoiceOption::Tag(*t as u8)).collect();
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
        let outcome = {
            let mut host = EngineVmHost {
                ctx: HostContext {
                    // def id 反查回填属增量 3（登记）。
                    skill_id: 0,
                    self_id: to_rule_pid(self.owner).raw(),
                    event_player: self
                        .trig
                        .event_player
                        .map(to_rule_pid)
                        .map(|p| p.raw())
                        .unwrap_or(0),
                    event_source: self
                        .trig
                        .event_source
                        .map(to_rule_pid)
                        .map(|p| p.raw())
                        .unwrap_or(0),
                    event_card: self.trig.event_card.map(|c| c as u32).unwrap_or(0),
                    event_amount: self.trig.event_amount,
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
                let pending = std::mem::take(&mut self.pending_effects);
                for e in pending {
                    commit_effect(ctx, e);
                }
                self.settled = true;
                Flow::Done
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
                self.asked = false;
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
            Err(_) => self.trap_done(),
        }
    }

    /// 回滚收尾：排队效果整批丢弃（半次效果不落账，原型同款）。
    fn trap_done(&mut self) -> Flow {
        self.pending_effects.clear();
        self.paused = None;
        self.settled = true;
        Flow::Done
    }
}
