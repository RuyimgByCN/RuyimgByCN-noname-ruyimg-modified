//! P2/N10 有界任务调度与 AI 基线（ADR 0772）。
//!
//! 规划器在**克隆引擎**上模拟候选动作（计算与规则推进隔离），以
//! 节点预算与时限约束搜索规模；产出携带提交时修订号的计划，应用侧
//! 对修订号陈旧的计划拒绝。取消标志在节点边界检查。

use crate::engine::Engine;
use crate::{CardKind, ChoiceOption, ChoiceView, CommandOutcome, GameAction, GameCommand};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// 单选候选枚举上限（防超大选项集撑爆候选向量；真实询问的选项数
/// 通常 ≤ 手牌数 + 1）。
const MAX_SINGLE_CANDIDATES: usize = 32;

/// 有界规划器配置。
#[derive(Clone)]
pub struct BoundedPlanner {
    /// 节点预算：模拟提交次数上限。
    pub node_budget: usize,
    /// 时限（毫秒）；None = 不限时。
    pub deadline_ms: Option<u64>,
    /// 取消标志（节点边界检查）。
    pub cancel: std::sync::Arc<AtomicBool>,
    /// 搜索深度：1 = 单层启发式（现状）；2 = 两层（我方候选后再模拟
    /// 对手最优应对）。T13 更强候选。
    pub lookahead_depth: u8,
    /// 策略变体：0 = 现状（仅杀候选 + 体力差评分）；1 = 更强候选
    /// （全手牌候选 + 牌值/存活/护甲/装备评分）；2 = 变体 1 + **内容调参**
    /// （L01 AI优化：按席位/技能威胁度加权）。T13 更强候选。
    pub policy_variant: u8,
    /// 内容调参（L01 AI优化装载面）：`Some` 且变体 2 时参与评分。
    pub content_tuning: Option<ContentTuning>,
}

/// L01 AI优化（noname-liuli）的参数消费面。
///
/// 由装载面（`content-native-loader::ai_tuning`）的配置映射而来：席位权重
/// （由武将权重表或技能威胁度表解析）、技能威胁度表（供宿主按需查询）、
/// `qzCf`（权重参与效益结算）。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ContentTuning {
    /// 席位权重（缺省 1.0）。调用方按武将/技能解析后注入。
    pub seat_weights: Vec<(u8, f64)>,
    /// 技能威胁度表（技能名 → 威胁度）。
    pub skill_threat: Vec<(String, f64)>,
    /// 权重参与效益结算（上游 `qzCf`）。
    pub qz_cf: bool,
}

impl ContentTuning {
    /// 席位权重（缺省 1.0；越界钳制到 `[0.1, 10.0]`）。
    pub fn weight_of(&self, seat: u8) -> f64 {
        self.seat_weights
            .iter()
            .find(|(s, _)| *s == seat)
            .map(|(_, w)| w.clamp(0.1, 10.0))
            .unwrap_or(1.0)
    }

    /// 技能集合的威胁度（取最大值；无记录 → 1.0）——对应装载面
    /// `weight(..., QzMode::Threat)`。
    pub fn threat_of_skills<'a>(&self, skills: impl IntoIterator<Item = &'a str>) -> f64 {
        skills
            .into_iter()
            .filter_map(|s| {
                self.skill_threat
                    .iter()
                    .find(|(k, _)| k == s)
                    .map(|(_, v)| *v)
            })
            .fold(None, |acc: Option<f64>, v| {
                Some(acc.map_or(v, |a| a.max(v)))
            })
            .unwrap_or(1.0)
    }

    /// 效益结算：`qzCf` 开启时「权重 × 结果」。
    pub fn benefit(&self, weight: f64, value: f64) -> f64 {
        if self.qz_cf {
            weight * value
        } else {
            value
        }
    }
}

impl BoundedPlanner {
    pub fn new(node_budget: usize) -> Self {
        Self {
            node_budget,
            deadline_ms: None,
            cancel: std::sync::Arc::new(AtomicBool::new(false)),
            lookahead_depth: 1,
            policy_variant: 0,
            content_tuning: None,
        }
    }

    pub fn with_deadline_ms(mut self, ms: u64) -> Self {
        self.deadline_ms = Some(ms);
        self
    }

    /// 两层搜索（T13 更强候选）：每个候选后再模拟对手最优应对。
    pub fn with_lookahead(mut self, depth: u8) -> Self {
        self.lookahead_depth = depth.max(1);
        self
    }

    /// 策略变体选择（T13 更强候选）：1 = 全手牌候选 + 牌值评分。
    pub fn with_policy_variant(mut self, variant: u8) -> Self {
        self.policy_variant = variant;
        self
    }

    /// 注入内容调参（L01 AI优化装载面）；变体 2 时参与评分。
    pub fn with_content_tuning(mut self, tuning: ContentTuning) -> Self {
        self.content_tuning = Some(tuning);
        self
    }

    pub fn cancel_flag(&self) -> std::sync::Arc<AtomicBool> {
        self.cancel.clone()
    }

    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }

    fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }
}

/// T13 工作池：并行多座规划——固定 worker 线程分桶消费座位队列，各
/// 线程独立克隆引擎（规划隔离已克隆化），结果按输入座位序返回。规划
/// 为确定性纯函数（无共享 RNG/可变状态），并行结果与逐座顺序规划逐一
/// 相等（回归固化）。
#[derive(Debug, Clone, Copy)]
pub struct PlanPool {
    /// 并发 worker 数（≥1；超过座位数时按座位数截断）。
    pub workers: usize,
}

impl PlanPool {
    pub fn new(workers: usize) -> Self {
        Self {
            workers: workers.max(1),
        }
    }

    /// 并行规划多个座位（同一引擎快照基线；输入序返回）。
    pub fn plan_seats(
        &self,
        engine: &Engine,
        seats: &[u8],
        planner: &BoundedPlanner,
    ) -> Vec<Plan> {
        if seats.is_empty() {
            return Vec::new();
        }
        let workers = self.workers.min(seats.len()).max(1);
        // 座位按 round-robin 分桶；每线程顺序消费自己的桶
        let mut buckets: Vec<Vec<u8>> = vec![Vec::new(); workers];
        for (i, &seat) in seats.iter().enumerate() {
            buckets[i % workers].push(seat);
        }
        std::thread::scope(|scope| {
            let handles: Vec<_> = buckets
                .into_iter()
                .map(|bucket| {
                    scope.spawn(move || {
                        bucket
                            .into_iter()
                            .map(|seat| engine.plan_action(seat, planner))
                            .collect::<Vec<Plan>>()
                    })
                })
                .collect();
            let parts: Vec<Vec<Plan>> = handles
                .into_iter()
                .map(|h| h.join().expect("规划工作线程"))
                .collect();
            // round-robin 逆操作：按输入序拼接
            let mut out = Vec::with_capacity(seats.len());
            let mut cursors = vec![0usize; workers];
            for (i, _) in seats.iter().enumerate() {
                let b = i % workers;
                out.push(parts[b][cursors[b]].clone());
                cursors[b] += 1;
            }
            out
        })
    }
}

/// 规划产物：携带提交时修订号，供应用侧做陈旧拒绝。
#[derive(Debug, Clone)]
pub struct Plan {
    pub seat: u8,
    pub revision: u64,
    pub action: GameAction,
    pub nodes_used: usize,
    pub budget_exhausted: bool,
    pub timed_out: bool,
    pub cancelled: bool,
}

/// 应用计划的失败方式。
#[derive(Debug, Clone)]
pub enum ApplyPlanError {
    /// 引擎修订号已前进：计划陈旧，必须重新规划。
    Stale { submitted: u64, current: u64 },
    /// 计划在生成时即被取消。
    Cancelled,
}


impl Engine {
    /// 策略驱动：组装座位输入（只读投影）→ 策略决策 → 主通道提交。
    /// None 动作回退为推进阶段。修订号取自输入组装时刻；
    /// 去重键由规划器侧单调分配（与规则推进解耦）。
    pub fn run_policy(&mut self, seat: u8, policy: &mut dyn crate::view::SeatPolicy) -> CommandOutcome {
        let input = self.ai_session(seat).input();
        let revision = input.revision;
        let action =
            policy.act(&input).unwrap_or(GameAction::NextPhase);
        // N10 命令序号（ADR 0775）：使用会话高水位，不使用全局计数器。
        // 高水位耗尽（u64::MAX）时 saturating 保持 MAX——引擎按重复命令
        // 干净拒绝，不回绕为 0 或 panic。
        self.submit(GameCommand {
            actor: seat,
            expected_revision: revision,
            dedup_id: self.last_dedup.saturating_add(1),
            action,
        })
    }
}

/// 两层搜索中对手应对的候选评估上限（防组合爆炸）。
const MAX_LOOKAHEAD_CANDIDATES: usize = 8;

/// 更强候选（策略变体 1）的出牌候选上限（防手牌×目标组合爆炸）。
const MAX_PLAY_CANDIDATES_V2: usize = 96;

/// 候选枚举（按策略变体分派）。
///
/// - 变体 0（现状）：询问回退 + 出牌阶段的【杀】。
/// - 变体 1（T13 更强候选）：询问回退增补「全部选项」组合；出牌阶段
///   枚举**全部手牌**（自目标用空目标表、非杀牌逐一尝试其他存活座位），
///   非法候选由模拟提交拒绝。候选按「杀 → 其他牌 → NextPhase」排序，
///   让节点预算优先覆盖最具进攻性的候选。
fn enumerate_candidates_variant(sim: &Engine, seat: u8, variant: u8) -> Vec<GameAction> {
    let input = sim.ai_session(seat).input();
    let mut candidates: Vec<GameAction> = Vec::new();
    if let Some(ChoiceView::Own {
        id,
        options,
        min,
        allow_cancel,
        ..
    }) = input.choice
    {
        for sel in options.iter().take(MAX_SINGLE_CANDIDATES).copied() {
            candidates.push(GameAction::Answer {
                choice_id: id,
                selection: vec![sel],
            });
        }
        if min >= 2 {
            let sel: Vec<ChoiceOption> = options.iter().take(min).copied().collect();
            candidates.push(GameAction::Answer {
                choice_id: id,
                selection: sel,
            });
            // 变体 1：小选项集时补「全选」组合（弃多张/多目标询问的
            // 一次性应答；组合爆炸由选项数上限挡住）。
            if variant >= 1
                && options.len() <= 8
                && options.len() > min
            {
                candidates.push(GameAction::Answer {
                    choice_id: id,
                    selection: options.iter().take(8).copied().collect(),
                });
            }
        } else if min == 0 && allow_cancel && !options.is_empty() {
            candidates.push(GameAction::Answer {
                choice_id: id,
                selection: vec![],
            });
        }
    } else if input.view.turn == seat && input.view.phase == crate::event::Phase::Play {
        let others: Vec<u8> = input
            .view
            .players
            .iter()
            .filter(|p| p.alive && p.seat != seat)
            .map(|p| p.seat)
            .collect();
        if variant == 0 {
            for (card, kind) in &input.own_hand_kinds {
                if *kind == CardKind::Sha {
                    for t in &others {
                        candidates.push(GameAction::PlayCard {
                            card: *card,
                            targets: vec![*t],
                        });
                    }
                }
            }
        } else {
            // 杀优先（进攻候选先占节点预算）
            for (card, kind) in &input.own_hand_kinds {
                if kind.is_sha_like() {
                    for t in &others {
                        candidates.push(GameAction::PlayCard {
                            card: *card,
                            targets: vec![*t],
                        });
                    }
                }
            }
            for (card, kind) in &input.own_hand_kinds {
                if candidates.len() >= MAX_PLAY_CANDIDATES_V2 {
                    break;
                }
                if kind.is_sha_like() {
                    continue;
                }
                // 自目标/无目标形态（桃、无中生有、装备、AOE 等）
                candidates.push(GameAction::PlayCard {
                    card: *card,
                    targets: vec![],
                });
                // 单目标形态（顺牵/拆桥/决斗/乐/兵粮等；非法由模拟拒绝）
                for t in &others {
                    if candidates.len() >= MAX_PLAY_CANDIDATES_V2 {
                        break;
                    }
                    candidates.push(GameAction::PlayCard {
                        card: *card,
                        targets: vec![*t],
                    });
                }
            }
        }
    }
    candidates.push(GameAction::NextPhase);
    candidates
}


/// 启发式评分（策略变体分派）。`base` = 规划前基线投影，`after` = 模拟
/// 提交后的投影；两者都是**观察者视角**的合法信息集。
///
/// - 变体 0（现状）：`(敌方体力下降) − (己方体力下降)`——与历史行为
///   逐值一致（同 seed 计划不变）。
/// - 变体 1（T13 更强候选）：定点整数加权——己方体力 8/手牌 3/装备 4，
///   敌方体力 8/手牌 1/装备 2（防御性资源），击杀 +24、己方死亡 -1000；
///   模拟出终局时以胜负巨值收口（胜 +100_000，非胜 -100_000）。
///
/// 手牌项天然抑制「白打一张牌」：出牌使手牌 -1（-3），无收益的牌因此
/// 劣于 `NextPhase`；摸牌/顺牵等净增手牌的牌为正收益。
fn policy_score(
    base: &crate::view::PlayerView,
    after: &crate::view::PlayerView,
    seat: u8,
    variant: u8,
    tuning: Option<&ContentTuning>,
) -> i64 {
    let hp_of = |v: &crate::view::PlayerView, s: u8| v.players[s as usize].hp as i64;
    let enemy_hp = |v: &crate::view::PlayerView| -> i64 {
        v.players
            .iter()
            .filter(|p| p.alive && p.seat != seat)
            .map(|p| p.hp as i64)
            .sum()
    };
    if variant == 0 {
        return (enemy_hp(base) - enemy_hp(after)) - (hp_of(base, seat) - hp_of(after, seat));
    }
    if let Some(winner) = &after.outcome {
        let win = matches!(winner, crate::state::Winner::Player(p) if *p == seat);
        return if win { 100_000 } else { -100_000 };
    }
    let me_b = &base.players[seat as usize];
    let me_a = &after.players[seat as usize];
    if me_b.alive && !me_a.alive {
        return -1000;
    }
    let mut score = 8 * (me_a.hp as i64 - me_b.hp as i64)
        + 3 * (me_a.hand_count as i64 - me_b.hand_count as i64)
        + 4 * (me_a.equips.len() as i64 - me_b.equips.len() as i64);
    // 变体 2：内容调参——敌方损失按**席位权重**加权（L01 AI优化）。
    let weighted = variant >= 2;
    for (pb, pa) in base.players.iter().zip(after.players.iter()) {
        if pb.seat == seat {
            continue;
        }
        let w = if weighted {
            tuning.map(|t| t.weight_of(pa.seat)).unwrap_or(1.0)
        } else {
            1.0
        };
        score -= (8.0 * w * (pa.hp as i64 - pb.hp as i64) as f64).round() as i64;
        score -= (w * (pa.hand_count as i64 - pb.hand_count as i64) as f64).round() as i64;
        score -= (2.0 * w * (pa.equips.len() as i64 - pb.equips.len() as i64) as f64).round()
            as i64;
        if pb.alive && !pa.alive {
            score += (24.0 * w).round() as i64;
        }
    }
    score
}

impl Engine {
    /// 有界规划：在克隆引擎上模拟候选动作（计算与规则推进隔离），
    /// 以启发式评分（敌方体力下降 − 己方体力下降）选最优动作。
    /// 不修改权威状态；克隆隔离保证规划中途的模拟不外泄。
    pub fn plan_action(&self, seat: u8, planner: &BoundedPlanner) -> Plan {
        let submitted_revision = self.revision();
        // wasm 目标无 wall-clock 时间源（Instant::now panic）：deadline
        // 静默失效=纯节点预算面（engine-native-wasm 壳契约，登记证据）；
        // 原生目标语义不变。
        #[cfg(target_arch = "wasm32")]
        let started: Option<Instant> = None;
        #[cfg(not(target_arch = "wasm32"))]
        let started: Option<Instant> = Some(Instant::now());
        // 计算与规则推进隔离：全部模拟发生在克隆上
        let mut sim = self.clone();
        // N10 规划信息集（ADR 0774/v29）：未知牌（对手暗手牌+牌库）
        // 确定性重分配——搜索不使用真实暗牌归属，且物理牌集合守恒
        crate::ai::redact_opponent_hands(&mut sim, seat);
        let baseline_view = sim.ai_session(seat).view();

        // 候选生成（只依赖只读投影）
        let candidates = enumerate_candidates_variant(&sim, seat, planner.policy_variant);

        let mut nodes = 0usize;
        let mut best: Option<(i64, GameAction)> = None;
        let mut budget_exhausted = false;
        let mut timed_out = false;
        for action in candidates {
            if planner.cancelled() {
                return Plan {
                    seat,
                    revision: submitted_revision,
                    action,
                    nodes_used: nodes,
                    budget_exhausted,
                    timed_out,
                    cancelled: true,
                };
            }
            if let (Some(t), Some(ms)) = (started, planner.deadline_ms) {
                if t.elapsed() > Duration::from_millis(ms) {
                    timed_out = true;
                    break;
                }
            }
            if nodes >= planner.node_budget {
                budget_exhausted = true;
                break;
            }
            nodes += 1;
            // 单节点：克隆上模拟提交并评分。
            // N10 命令序号（ADR 0776）：模拟提交与真实提交同源——用克隆
            // 自身的会话高水位推进，不再用固定 `1_000_000 + nodes` 偏移
            // （真实高水位越过该区间时合法候选全部被当重复拒绝，
            // 规划退化为 NextPhase）。
            let mut trial = sim.clone();
            let out = trial.submit(GameCommand {
                actor: seat,
                expected_revision: trial.revision(),
                dedup_id: trial.last_dedup.saturating_add(1),
                action: action.clone(),
            });
            if matches!(out, CommandOutcome::Rejected(_)) {
                continue; // 非法候选不计分
            }
            let after = trial.ai_session(seat).view();
            let mut score = policy_score(
                &baseline_view,
                &after,
                seat,
                planner.policy_variant,
                planner.content_tuning.as_ref(),
            );
            // 两层搜索（T13 更强候选）：再模拟对手最优应对，取最差结果
            //（对手最大化其收益）作为本候选得分。节点预算与时限仍受控。
            if planner.lookahead_depth >= 2 && trial.outcome().is_none() {
                let opp = match trial.open_choice() {
                    Some(open) => open.observer,
                    None => trial.turn().current,
                };
                if opp != seat {
                    let opp_candidates =
                        enumerate_candidates_variant(&trial, opp, planner.policy_variant);
                    let mut worst: Option<i64> = None;
                    for opp_action in opp_candidates.into_iter().take(MAX_LOOKAHEAD_CANDIDATES) {
                        if planner.cancelled() || nodes >= planner.node_budget {
                            budget_exhausted = true;
                            break;
                        }
                        if let (Some(t), Some(ms)) = (started, planner.deadline_ms) {
                            if t.elapsed() > Duration::from_millis(ms) {
                                timed_out = true;
                                break;
                            }
                        }
                        nodes += 1;
                        let mut t2 = trial.clone();
                        let o = t2.submit(GameCommand {
                            actor: opp,
                            expected_revision: t2.revision(),
                            dedup_id: t2.last_dedup.saturating_add(1),
                            action: opp_action,
                        });
                        if matches!(o, CommandOutcome::Rejected(_)) {
                            continue;
                        }
                        let v = t2.ai_session(seat).view();
                        let s = policy_score(
                            &baseline_view,
                            &v,
                            seat,
                            planner.policy_variant,
                            planner.content_tuning.as_ref(),
                        );
                        if worst.map(|w| s < w).unwrap_or(true) {
                            worst = Some(s);
                        }
                    }
                    if let Some(w) = worst {
                        score = w;
                    }
                }
            }
            if best.as_ref().map(|(s, _)| score > *s).unwrap_or(true) {
                best = Some((score, action.clone()));
            }
            // 模拟推进回退：单层评分不需要回退——每个候选独立从 sim 克隆
            let _ = &mut sim; // sim 保持基线（候选都从同一基线出发）
        }
        let action = best
            .map(|(_, a)| a)
            .unwrap_or(GameAction::NextPhase);
        Plan {
            seat,
            revision: submitted_revision,
            action,
            nodes_used: nodes,
            budget_exhausted,
            timed_out,
            cancelled: false,
        }
    }

    /// 应用计划：修订号陈旧或计划取消时拒绝；否则走主提交通道。
    pub fn apply_plan(&mut self, plan: &Plan) -> Result<CommandOutcome, ApplyPlanError> {
        if plan.cancelled {
            return Err(ApplyPlanError::Cancelled);
        }
        let current = self.revision();
        if current != plan.revision {
            return Err(ApplyPlanError::Stale {
                submitted: plan.revision,
                current,
            });
        }
        Ok(self.submit(GameCommand {
            actor: plan.seat,
            expected_revision: current,
            dedup_id: self.last_dedup.saturating_add(1),
            action: plan.action.clone(),
        }))
    }
}

/// N10 规划信息集（ADR 0774，v29 修订为合法信息集）：把「对手暗手牌 +
/// 牌库」整体视为观察者的**未知牌池**，确定性洗牌后按原数量重新分配
/// （各对手手牌数、牌库数不变，其余回牌库）。
///
/// 合法性：观察者知道对手的手牌**数**与牌库**数**，但不知道其中具体
/// 是哪些牌——未知牌在「各对手手牌 + 牌库」之间的任何一种分配都与
/// 观察者知识一致，采样其中一种即合法的完全信息对局状态。
/// 守恒性：池内牌只被**移动**、从不复制——物理牌归属核算
/// （每张目录牌恰好出现一次）在重分配后依然成立，规划克隆可直接
/// 存档/恢复；旧实现从牌堆采样并复制 ID，会制造重复物理归属。
/// 退化：牌堆为空时池=对手暗牌，重排后仍不复制、不读入任何观察者
/// 不可知信息；确定性由座位号播种的 LCG 保证（无 OS 熵）。
pub fn redact_opponent_hands(e: &mut Engine, seat: u8) {
    use crate::ids::CardId;
    let core = &mut e.core_mut();
    let n = core.players.len();
    let opponents: Vec<usize> = (0..n).filter(|&i| i as u8 != seat).collect();
    let mut hand_lens: Vec<usize> = Vec::with_capacity(opponents.len());
    let mut pool: Vec<CardId> = Vec::new();
    for &i in &opponents {
        hand_lens.push(core.players[i].hand.len());
        pool.append(&mut core.players[i].hand);
    }
    pool.append(&mut core.deck);
    // 确定性 Fisher–Yates（LCG：座位号播种，可重放）。
    let mut rng_state: u64 = 0x5EED_CAFE_0000 ^ (seat as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    for k in (1..pool.len()).rev() {
        rng_state = rng_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let j = ((rng_state >> 33) as usize) % (k + 1);
        pool.swap(j, k);
    }
    // 按原手牌数重新分配；剩余全部回牌库（牌库顺序本就不可见）。
    let mut cursor = 0usize;
    for (&i, &len) in opponents.iter().zip(hand_lens.iter()) {
        core.players[i].hand = pool[cursor..cursor + len].to_vec();
        cursor += len;
    }
    core.deck = pool[cursor..].to_vec();
}
