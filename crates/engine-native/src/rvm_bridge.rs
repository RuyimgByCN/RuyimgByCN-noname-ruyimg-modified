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
    /// 目录 def 名（T8 增量 8：HostContext.skill_id 反查键）。
    pub skill_name: String,
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
    /// 阶段事件携带（阶段种，begin 由 EventKind 区分；增量 14）。
    phase: Option<crate::event::Phase>,
}

fn trigger_ctx_of(event: &EventData) -> VmTriggerCtx {
    match event {
        EventData::Damage(d) => VmTriggerCtx {
            event_player: Some(d.target),
            event_source: d.source,
            event_card: d.card,
            event_amount: d.amount,
            ..VmTriggerCtx::default()
        },
        EventData::Phase {
            player, phase, ..
        } => VmTriggerCtx {
            event_player: Some(*player),
            phase: Some(*phase),
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

/// DSL filter 映射（增量 6 三基 + 增量 9 基元扩展；增量 13 语义收敛——
/// `owner_is_target_*` 家族全部含「事件主玩家=持有者」合取段）：具名
/// 组合 filter 由装载器跳过登记（deferred）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VmFilterPolicy {
    /// DSL `owner_is_target`：事件主玩家=持有者。
    #[default]
    OwnerIsTarget,
    /// DSL `owner_source`：事件来源=持有者。
    OwnerIsSource,
    /// DSL `always`：无条件。
    Always,
    /// DSL `owner_is_target_wounded`：事件主玩家=持有者，且持有者已受伤
    /// （0<hp<maxHp）。
    OwnerWounded,
    /// DSL `owner_is_target_linked`：事件主玩家=持有者，且持有者横置。
    OwnerLinked,
    /// DSL `owner_is_target_hand_nonempty`：事件主玩家=持有者，且持有者
    /// 手牌非空。
    OwnerHandNonempty,
    /// DSL `owner_is_target_hand_empty`：事件主玩家=持有者，且持有者手牌
    /// 为空。
    OwnerHandEmpty,
    /// DSL `owner_is_target_no_turn_damage`：事件主玩家=持有者，且持有者
    /// 本回合未造成伤害。
    OwnerNoTurnDamage,
    /// DSL `owner_is_target_turn_damage`：事件主玩家=持有者，且持有者本
    /// 回合已造成伤害。
    OwnerTurnDamage,
    /// DSL `owner_is_target_hand_below_max_hp`：事件主玩家=持有者，且手
    /// 牌数 < 体力上限。
    OwnerHandBelowMaxHp,
    /// DSL `owner_is_target_hand_above_hp`：事件主玩家=持有者，且手牌数
    /// > 体力。
    OwnerHandAboveHp,
    /// DSL `owner_is_target_hand_at_least_hp`：事件主玩家=持有者，且手牌
    /// 数 >= 体力。
    OwnerHandAtLeastHp,
    /// DSL `owner_is_target_other_hand_nonempty`：事件主玩家=持有者，且
    /// 任一其他存活角色手牌非空。
    OwnerOtherHandNonempty,
    /// DSL `owner_is_target_turn_damage_at_least_three`：事件主玩家=持有
    /// 者，且本回合造成伤害 ≥3。
    OwnerTurnDamageAtLeastThree,
    /// DSL `owner_is_target_hand_at_least_hp_other_alive`：事件主玩家=持
    /// 有者，手牌数 ≥ 体力，且存在其他存活角色。
    OwnerHandAtLeastHpOtherAlive,
    /// DSL `owner_is_target_play_phase_card_category_count_positive`：事
    /// 件主玩家=持有者，事件为出牌阶段结束，且本出牌阶段已使用 ≥1 张基
    /// 本/锦囊/装备牌（增量 14，ABI 42）。
    OwnerPlayPhaseBteUsed,
    /// DSL `owner_alive_play_phase_card_category_count_positive`：事件为
    /// 出牌阶段结束，且持有者在该出牌阶段已使用 ≥1 张基本/锦囊/装备牌
    ///（仅当前回合玩家可用牌，故事件主玩家=持有者；增量 14，ABI 293）。
    OwnerAlivePlayPhaseBteUsed,
    /// DSL `owner_is_target_hp_below_{N}`：事件主玩家=持有者，且当前体力
    /// < N（增量 15，参数化）。
    OwnerHpBelow(u32),
    /// DSL `owner_source_play_card_count_eq_{N}`：事件主玩家=持有者（使
    /// 用者），且本次为其本出牌阶段第 N 张主动用牌（响应不计、出牌阶段
    /// 开始清零；增量 15，参数化）。
    OwnerPlayCountEq(u32),
    /// DSL `owner_source_play_card_count_eq_hp`：同上，N=当前体力（增量
    /// 15）。
    OwnerPlayCountEqHp,
    /// DSL `owner_is_target_card_used_at_least_hp`：事件主玩家=持有者，
    /// 且本回合（口径化归：出牌阶段计数，Play 开始清零=换回合归零）已
    /// 主动用牌 ≥ 当前体力（增量 15；响应不计）。
    OwnerCardUsedAtLeastHp,
}

/// VM 技能行为：触发即执行模块（事件门由 def 的 TriggerMeta、行为总线
/// 与 filter 策略共同承载）。
#[derive(Debug, Clone)]
pub struct VmSkillBehavior {
    pub program: Arc<VmSkillProgram>,
    pub filter: VmFilterPolicy,
}

impl SkillBehavior for VmSkillBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        // 增量 13 语义收敛：`owner_is_target_*` 家族的权威语义（编译器
        // lsp 权威描述 "the skill owner is the event player and …"）是
        // 「事件主玩家=持有者」与状态谓词的合取——filter 是触发调度里
        // 唯一的持有者域闸（plan_triggers 只按 EventKind 找候选），漏掉
        // 合取段会让他人事件过闸（炙焚 holder 门同类缺陷）。turn_damage
        // 为本回合「造成」伤害计数（kernel 记在来源侧），与 lsp 一致。
        let trig = trigger_ctx_of(event);
        let is_event_player = trig.event_player == Some(owner);
        let holder_state = |s: u8| ctx.state.player(s);
        match self.filter {
            VmFilterPolicy::OwnerIsTarget => is_event_player,
            VmFilterPolicy::OwnerIsSource => trig.event_source == Some(owner),
            VmFilterPolicy::Always => true,
            VmFilterPolicy::OwnerWounded => {
                is_event_player
                    && {
                        let p = holder_state(owner);
                        p.hp > 0 && p.hp < p.max_hp as i32
                    }
            }
            VmFilterPolicy::OwnerLinked => {
                is_event_player && holder_state(owner).linked
            }
            VmFilterPolicy::OwnerHandNonempty => {
                is_event_player && !holder_state(owner).hand.is_empty()
            }
            VmFilterPolicy::OwnerHandEmpty => {
                is_event_player && holder_state(owner).hand.is_empty()
            }
            VmFilterPolicy::OwnerNoTurnDamage => {
                is_event_player && holder_state(owner).turn_damage == 0
            }
            VmFilterPolicy::OwnerTurnDamage => {
                is_event_player && holder_state(owner).turn_damage > 0
            }
            VmFilterPolicy::OwnerHandBelowMaxHp => {
                is_event_player && {
                    let p = holder_state(owner);
                    p.hand.len() < p.max_hp as usize
                }
            }
            VmFilterPolicy::OwnerHandAboveHp => {
                is_event_player && {
                    let p = holder_state(owner);
                    p.hand.len() > p.hp as usize
                }
            }
            VmFilterPolicy::OwnerHandAtLeastHp => {
                is_event_player && {
                    let p = holder_state(owner);
                    p.hand.len() >= p.hp as usize
                }
            }
            VmFilterPolicy::OwnerOtherHandNonempty => {
                is_event_player
                    && (0..ctx.state.players.len()).any(|i| {
                        i as u8 != owner
                            && ctx.state.players[i].alive
                            && !ctx.state.players[i].hand.is_empty()
                    })
            }
            VmFilterPolicy::OwnerTurnDamageAtLeastThree => {
                is_event_player && holder_state(owner).turn_damage >= 3
            }
            VmFilterPolicy::OwnerHandAtLeastHpOtherAlive => {
                is_event_player && {
                    let p = holder_state(owner);
                    p.hand.len() >= p.hp as usize
                } && (0..ctx.state.players.len()).any(|i| {
                    i as u8 != owner && ctx.state.players[i].alive
                })
            }
            // 出牌阶段结束 && 本阶段三类牌已用 ≥1（计数随 Play 开始清
            // 零，事件时点值即本阶段值；ABI 293 的持有者归因由「仅当前
            // 回合玩家可用牌」化归为事件主玩家合取）。
            VmFilterPolicy::OwnerPlayPhaseBteUsed
            | VmFilterPolicy::OwnerAlivePlayPhaseBteUsed => {
                is_event_player
                    && trig.phase == Some(crate::event::Phase::Play)
                    && ctx.state.turn.bte_used_play_phase > 0
            }
            VmFilterPolicy::OwnerHpBelow(n) => {
                is_event_player && {
                    let p = holder_state(owner);
                    p.hp > 0 && p.hp < n as i32
                }
            }
            // 用牌序数：commit_card_used 尾计数=本次用牌的序数（响应走
            // CardResponded 不经此点=「Responses do not count」同口径）。
            VmFilterPolicy::OwnerPlayCountEq(n) => {
                is_event_player && ctx.state.turn.cards_used_play_phase == n
            }
            VmFilterPolicy::OwnerPlayCountEqHp => {
                let p = holder_state(owner);
                is_event_player
                    && ctx.state.turn.cards_used_play_phase == p.hp as u32
            }
            VmFilterPolicy::OwnerCardUsedAtLeastHp => {
                let p = holder_state(owner);
                is_event_player
                    && ctx.state.turn.cards_used_play_phase >= p.hp as u32
            }
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
            program: Arc::new(VmSkillProgram {
                skill_name: module.name.clone(),
                module,
            }),
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
            program: Arc::new(VmSkillProgram {
                skill_name: module.name.clone(),
                module,
            }),
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
                    // T8 增量 8：skill_id 经目录反查回填（查不到保持 0=NONE）。
                    skill_id: ctx
                        .catalog
                        .skill_by_name(&self.program.skill_name)
                        .map(|d| d as u32)
                        .unwrap_or(0),
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
