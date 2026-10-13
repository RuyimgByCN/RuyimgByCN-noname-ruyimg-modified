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
use rule_vm::{BytecodeModule, ExecResult, HostContext, RuleVm, VmHost, DEFAULT_FUEL};

use crate::catalog::{SkillBehavior, SkillMachine};
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

/// VM 技能行为：触发即执行模块（事件门由 def 的 TriggerMeta 与行为总线
/// 承载；DSL filter 字段的映射属增量 2，登记）。
#[derive(Debug, Clone)]
pub struct VmSkillBehavior {
    pub program: Arc<VmSkillProgram>,
}

impl SkillBehavior for VmSkillBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        // 增量 1 过滤面：恒定 owner 域（event_player==owner，DSL
        // owner_is_target 族）；其余 filter 表达式映射属增量 2。
        trigger_ctx_of(event).event_player == Some(owner)
    }

    fn start(&self, owner: PlayerId, event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(VmMachine {
            owner,
            program: Some(self.program.clone()),
            trig: trigger_ctx_of(event),
            pending_effects: Vec::new(),
            settled: false,
        })
    }
}

/// VM 技能机器：一次 dispatch 内执行完毕（本增量无内部挂起；快照只
/// 出现在「start→step 原子完成」间隙之外，重建为直接 Done 的占位机）。
#[derive(Debug, Clone)]
pub struct VmMachine {
    owner: PlayerId,
    /// None = 恢复重建的 detached 占位（settled=true，直接 Done）。
    program: Option<Arc<VmSkillProgram>>,
    trig: VmTriggerCtx,
    /// 排队未落账效果（Halted 提交 / Paused、Err 丢弃）。
    pending_effects: Vec<EffectRequest>,
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

impl SkillMachine for VmMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }

    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::Rvm {
            owner: self.owner,
            module_hash: self
                .program
                .as_ref()
                .map(|p| p.logic_hash())
                .unwrap_or([0; 32]),
        }
    }

    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if self.settled {
            return Flow::Done;
        }
        self.settled = true;
        let Some(program) = self.program.clone() else {
            return Flow::Done;
        };
        let seats = ctx.core.players.len() as u8;
        {
            let mut host = EngineVmHost {
                ctx: HostContext {
                    // 增量 1：def id 未回填（catalog def 反查属增量 2）。
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
            let out = vm.execute(&program.module, &mut host, DEFAULT_FUEL);
            match out {
                Ok(ExecResult::Halted { .. }) => {}
                Ok(ExecResult::Paused { .. }) | Err(_) => {
                    // Paused（增量 2）/Err：排队效果整批丢弃（回滚协议）。
                    self.pending_effects.clear();
                }
            }
        }
        let pending = std::mem::take(&mut self.pending_effects);
        for e in pending {
            commit_effect(ctx, e);
        }
        Flow::Done
    }
}
