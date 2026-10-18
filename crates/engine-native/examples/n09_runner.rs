//! N09 新线无界面性能 runner（release 运行）。
//!
//! 负载：确定性脚本对局——FFA 原型（出牌阶段对下一位存活玩家出杀、
//! 响应按策略放弃）、5 人身份局（同脚本）与鬼道/天妒专项判定局
//! （改判者主动出黑牌改判、天妒确认获得，ADR 0720 后的主动负载）。
//! 每条命令计时（纳秒原值），输出接受/拒绝（含拒绝原因分布）、终局、
//! 事件数、延迟分位（P50/P95/P99，真实 ns）与跨目标摘要；另测
//! `save()` 与 `restore()` 平均成本。原始逐命令样本经 `N09_SAMPLES`
//! 环境变量落盘 CSV（load,game,seq,ns），分配统计在全部负载结束后输出。
//!
//! 运行：cargo run --release -p engine-native --example n09_runner -- [games] [seed]

use engine_native::{
    CardKind, ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, OpenChoice, Phase,
    RejectReason,
};
use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// 分配统计分配器（N09 分配/内存统计）：累计分配字节数与次数。
/// 仅 example 内生效，不影响库。
struct CountingAlloc;

static ALLOC_BYTES: AtomicU64 = AtomicU64::new(0);
static ALLOC_COUNT: AtomicU64 = AtomicU64::new(0);
static LIVE_BYTES: AtomicU64 = AtomicU64::new(0);
static PEAK_LIVE_BYTES: AtomicU64 = AtomicU64::new(0);

/// 计量开关：`N09_PLAIN=1` 时计数钩子短路（近似「关闭计数器」对照——
/// 分支开销仍在，但不含原子操作），用于评估计数器本身的计时/内存影响。
static PLAIN_MODE: AtomicU64 = AtomicU64::new(0);

unsafe impl std::alloc::GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        if PLAIN_MODE.load(Ordering::Relaxed) == 0 {
            ALLOC_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
            ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
            let live = LIVE_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed)
                + layout.size() as u64;
            PEAK_LIVE_BYTES.fetch_max(live, Ordering::Relaxed);
        }
        std::alloc::System.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        if PLAIN_MODE.load(Ordering::Relaxed) == 0 {
            LIVE_BYTES.fetch_sub(layout.size() as u64, Ordering::Relaxed);
        }
        std::alloc::System.dealloc(ptr, layout)
    }
}

#[global_allocator]
static GLOBAL_ALLOC: CountingAlloc = CountingAlloc;

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn fresh() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

struct Metrics {
    commands: u64,
    accepted: u64,
    rejected: u64,
    rejects_by_reason: std::collections::BTreeMap<&'static str, u64>,
    events: u64,
    duration_ns: u128,
    samples_ns: Vec<u128>,
}

impl Metrics {
    fn new() -> Self {
        Self {
            commands: 0,
            accepted: 0,
            rejected: 0,
            rejects_by_reason: Default::default(),
            events: 0,
            duration_ns: 0,
            samples_ns: Vec::new(),
        }
    }
    fn note(&mut self, out: &CommandOutcome, dt_ns: u128) {
        self.commands += 1;
        self.duration_ns += dt_ns;
        self.samples_ns.push(dt_ns);
        match out {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {
                self.accepted += 1
            }
            CommandOutcome::Rejected(r) => {
                self.rejected += 1;
                let key: &'static str = match r {
                    RejectReason::ChoicePending => "choice_pending",
                    RejectReason::NoSuchChoice => "no_such_choice",
                    RejectReason::InvalidChoice => "invalid_choice",
                    RejectReason::NotYourTurn => "not_your_turn",
                    RejectReason::WrongPhase => "wrong_phase",
                    RejectReason::CardNotInHand => "card_not_in_hand",
                    RejectReason::IllegalTargets => "illegal_targets",
                    RejectReason::ShaLimit => "sha_limit",
                    RejectReason::HpFull => "hp_full",
                    RejectReason::MatchOver => "match_over",
                    RejectReason::BudgetExhausted => "budget_exhausted",
                    RejectReason::WrongPlayer => "wrong_player",
                    RejectReason::UnknownActor => "unknown_actor",
                    RejectReason::StaleRevision => "stale_revision",
                    RejectReason::DuplicateCommand => "duplicate_command",
                    RejectReason::SegmentSuspended => "segment_suspended",
                    RejectReason::NotSuspended => "not_suspended",
                    RejectReason::QuotaExhausted => "quota_exhausted",
                    RejectReason::NoSuchSession => "no_such_session",
                    RejectReason::Impersonation => "impersonation",
                    RejectReason::NoSuchSkill => "no_such_skill",
                    RejectReason::SkillDisabled => "skill_disabled",
                    RejectReason::SkillAlreadyUsed => "skill_already_used",
                    RejectReason::WrongMode => "wrong_mode",
                    RejectReason::NoSuchGeneral => "no_such_general",
                    RejectReason::AlreadyRevealed => "already_revealed",
                    RejectReason::VmFault { .. } => "vm_fault",
                    RejectReason::CardLimit => "card_limit",
                    RejectReason::CardBlocked => "card_blocked",
                };
                *self.rejects_by_reason.entry(key).or_insert(0) += 1;
            }
            CommandOutcome::Suspended => self.accepted += 1,
        }
    }
    fn merge(&mut self, other: &Metrics) {
        self.commands += other.commands;
        self.accepted += other.accepted;
        self.rejected += other.rejected;
        for (k, v) in &other.rejects_by_reason {
            *self.rejects_by_reason.entry(k).or_insert(0) += v;
        }
        self.events += other.events;
        self.duration_ns += other.duration_ns;
        self.samples_ns.extend(other.samples_ns.iter().copied());
    }
    fn percentile(sorted: &[u128], p: f64) -> u128 {
        if sorted.is_empty() {
            return 0;
        }
        let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
        sorted[idx.min(sorted.len() - 1)]
    }
    fn summary(&self) -> String {
        let mut sorted = self.samples_ns.clone();
        sorted.sort_unstable();
        format!(
            "commands={} accepted={} rejected={} rejects={:?} events={} wall_us={} p50_ns={} p95_ns={} p99_ns={}",
            self.commands,
            self.accepted,
            self.rejected,
            self.rejects_by_reason,
            self.events,
            self.duration_ns / 1000,
            Self::percentile(&sorted, 0.50),
            Self::percentile(&sorted, 0.95),
            Self::percentile(&sorted, 0.99),
        )
    }
}

fn timed(e: &mut Engine, cmd: GameCommand, m: &mut Metrics) -> CommandOutcome {
    let t0 = Instant::now();
    let out = e.submit(cmd);
    let dt = t0.elapsed().as_nanos();
    m.note(&out, dt);
    out
}

/// 应答策略（N09：主动负载而非全 Pass）：
/// - `gui_dao_rewrite`：主动打出第一张黑牌改判（无牌可选才 Pass）；
/// - `tian_du_gain`：确认获得；
/// - `discard_to_limit`：按 min 取前若干张牌弃置；
/// - `nullify_window`：一律 Pass（不抵消锦囊，保持脚本确定性）；
/// - 其余：取首选项。
fn selection_for(choice: &OpenChoice) -> Vec<ChoiceOption> {
    let card_opts = || {
        choice
            .spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Card(_)))
            .copied()
            .collect::<Vec<_>>()
    };
    match choice.spec.prompt {
        "gui_dao_rewrite" => {
            let cards = card_opts();
            if cards.is_empty() {
                vec![choice.spec.options[0]]
            } else {
                vec![cards[0]]
            }
        }
        "tian_du_gain" => vec![ChoiceOption::Confirm],
        "discard_to_limit" => {
            let take = choice.spec.min.max(1) as usize;
            let mut cards = card_opts();
            cards.truncate(take);
            cards
        }
        "nullify_window" => vec![ChoiceOption::Pass],
        _ => vec![choice.spec.options[0]],
    }
}

/// 确定性脚本驱动：段内链式选择（濒死/改判/天妒/弃牌/无懈）按策略排干，
/// Suspended 续行；Accepted/Rejected 返回。
fn drive(e: &mut Engine, mut out: CommandOutcome, m: &mut Metrics) {
    let mut guard = 0;
    loop {
        guard += 1;
        assert!(guard < 5000, "drive 未收敛");
        out = match out {
            CommandOutcome::Suspended => {
                let t0 = Instant::now();
                let r = e.resume();
                let dt = t0.elapsed().as_nanos();
                m.note(&r, dt);
                r
            }
            CommandOutcome::Pending { choice } => {
                let sel = selection_for(&choice);
                let a = choice.observer;
                let cid = choice.id;
                let cmd = GameCommand {
                    actor: a,
                    expected_revision: e.revision(),
                    dedup_id: fresh(),
                    action: GameAction::Answer {
                        choice_id: cid,
                        selection: sel,
                    },
                };
                timed(e, cmd, m)
            }
            CommandOutcome::Accepted { .. } | CommandOutcome::Rejected(_) => return,
        };
    }
}

fn advance(e: &mut Engine, m: &mut Metrics) {
    let out = next_phase(e, m);
    drive(e, out, m);
}

fn next_phase(e: &mut Engine, m: &mut Metrics) -> CommandOutcome {
    let a = e.turn().current;
    let cmd = GameCommand {
        actor: a,
        expected_revision: e.revision(),
        dedup_id: fresh(),
        action: GameAction::NextPhase,
    };
    timed(e, cmd, m)
}

/// 确定性脚本：出牌阶段对下一位存活玩家出杀（有杀才出），其余推进。
fn advance_or_sha(e: &mut Engine, m: &mut Metrics) {
    if e.turn().phase == Phase::Play {
        let actor = e.turn().current;
        let sha = e
            .debug_hand(actor)
            .iter()
            .copied()
            .find(|c| e.catalog().card_kind(*c) == CardKind::Sha);
        if let Some(sha) = sha {
            let target = e.state_core().next_alive(actor);
            let cmd = GameCommand {
                actor,
                expected_revision: e.revision(),
                dedup_id: fresh(),
                action: GameAction::PlayCard {
                    card: sha,
                    targets: vec![target],
                },
            };
            let out = timed(e, cmd, m);
            match out {
                CommandOutcome::Rejected(_) => advance(e, m),
                other => drive(e, other, m),
            }
            return;
        }
    }
    advance(e, m);
}

fn ffa_round(e: &mut Engine, m: &mut Metrics) {
    for _ in 0..64 {
        if e.outcome().is_some() {
            return;
        }
        advance_or_sha(e, m);
    }
}

/// 原始样本 CSV（N09 原始数据留存）：load,game,seq,ns 一行一样本。
struct SampleSink {
    path: Option<std::path::PathBuf>,
}

impl SampleSink {
    fn from_env() -> Self {
        Self {
            path: std::env::var("N09_SAMPLES").ok().map(std::path::PathBuf::from),
        }
    }
    fn dump(&self, load: &str, game: usize, m: &Metrics) {
        if let Some(p) = &self.path {
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(p)
            {
                for (i, ns) in m.samples_ns.iter().enumerate() {
                    let _ = writeln!(f, "{load},{game},{i},{ns}");
                }
            }
        }
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let games: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(20);
    let base_seed: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(9000);
    let sink = SampleSink::from_env();
    let plain = std::env::var("N09_PLAIN").map(|v| v == "1").unwrap_or(false);
    if plain {
        PLAIN_MODE.store(1, Ordering::Relaxed);
    }

    let ruleset = {
        let e = Engine::new(1, 4);
        e.catalog().ruleset_version
    };
    println!(
        "mode=load games={games} seed_base={base_seed} ruleset={ruleset} engine=native counting={}",
        if plain { "plain" } else { "on" }
    );

    // 每命令检查点成本：中局状态 save()/restore() 平均耗时。
    // 说明：单命令延迟（下文分位）已包含段内工作副本克隆与检查点总成本；
    // 此处单独给出 save/restore 原语成本供容量规划参考。
    {
        let mut e = Engine::new(base_seed, 4);
        for _ in 0..64 {
            if e.turn().current == 0 && e.turn().phase == Phase::Play {
                break;
            }
            advance(&mut e, &mut Metrics::new());
        }
        let n = 2000;
        let t0 = Instant::now();
        let mut sink_bytes = 0usize;
        for _ in 0..n {
            let snap = e.save();
            // 观测屏障：消费快照内容，防止克隆被优化删除
            sink_bytes = sink_bytes
                .wrapping_add(std::hint::black_box(&snap).core.deck.len());
        }
        let save_ns = t0.elapsed().as_nanos() / n as u128;
        let snap = e.save();
        let t0 = Instant::now();
        for _ in 0..n {
            let r = Engine::restore(&snap).expect("restore");
            sink_bytes =
                sink_bytes.wrapping_add(std::hint::black_box(&r).revision() as usize);
        }
        let restore_ns = t0.elapsed().as_nanos() / n as u128;
        println!(
            "checkpoint save_avg_ns={save_ns} restore_avg_ns={restore_ns} samples={n} sink={sink_bytes} (save=StateSnapshot 全量克隆)"
        );
    }

    let alloc_before_all = ALLOC_BYTES.load(Ordering::Relaxed);

    // 负载 1：FFA 原型局
    let mut total = Metrics::new();
    for g in 0..games {
        let mut e = Engine::new(base_seed + g as u64, 4);
        let mut m = Metrics::new();
        let mut rounds = 0;
        while e.outcome().is_none() {
            rounds += 1;
            assert!(rounds < 500, "FFA 未收敛");
            ffa_round(&mut e, &mut m);
        }
        m.events = e.log().len() as u64;
        total.merge(&m);
        sink.dump("ffa", g, &m);
        println!(
            "game=ffa g={g} rounds={rounds} outcome={:?} {}",
            e.outcome().map(|o| o.winner),
            m.summary()
        );
        println!(
            "digest ffa game={g} seed={} hash={}",
            base_seed + g as u64,
            hex(&e.state_hash())
        );
    }
    println!(
        "mode=ffa-aggregate {} alloc_delta_bytes={}",
        total.summary(),
        ALLOC_BYTES.load(Ordering::Relaxed) - alloc_before_all
    );

    // 负载 2：5 人身份局（纯命令脚本，至阵营终局或有界）
    let mut total_i = Metrics::new();
    let mut finished = 0;
    for g in 0..games {
        let mut e = Engine::try_new_identity(base_seed + 1000 + g as u64, 5).expect("identity");
        let mut m = Metrics::new();
        let mut cmds = 0usize;
        let mut converged = true;
        while e.outcome().is_none() {
            cmds += 1;
            if cmds > 6000 {
                // 被动脚本下的合法长局（如内奸 1v1）不收敛：记录后跳过
                converged = false;
                break;
            }
            advance_or_sha(&mut e, &mut m);
        }
        m.events = e.log().len() as u64;
        if converged {
            finished += 1;
        }
        total_i.merge(&m);
        sink.dump("identity", g, &m);
        println!(
            "game=identity g={g} converged={converged} outcome={:?} {}",
            e.outcome().map(|o| o.winner),
            m.summary()
        );
        println!(
            "digest identity g={g} seed={} hash={}",
            base_seed + 1000 + g as u64,
            hex(&e.state_hash())
        );
    }
    println!(
        "mode=identity-aggregate {} finished={finished}",
        total_i.summary()
    );

    // 负载 3：鬼道/天妒专项（主动改判 + 判定 + 天妒获得的密集判定对局）。
    // 2 号持鬼道主动出黑牌改判；0 号持天妒确认获得；布阵前抬高体力避免
    // 闪电命中提前终局。判定牌布顶黑桃 2–9：乐跳阶段、闪电必中，事件密集。
    let mut total_j = Metrics::new();
    for g in 0..10 {
        let mut e = Engine::new(base_seed + 2000 + g as u64, 4);
        e.debug_grant(0, "天妒");
        e.debug_grant(2, "鬼道");
        advance(&mut e, &mut Metrics::new()); // 预热驱动至回合内推进（丢弃预热指标）
        for p in 0..4 {
            e.debug_set_hp(p, 8);
            e.debug_clear_hand(p);
        }
        let mut m = Metrics::new();
        for round in 0..6 {
            let owner = (round % 4) as u8;
            let trick = if round % 2 == 0 { "le" } else { "shandian" };
            // 闪电未命中会转移到下家判定区；布阵前清场保证同名锦囊可回收
            e.debug_sweep_judge_zones();
            let _t = e.debug_place_judge(owner, trick);
            if round % 3 == 0 {
                // 改判者弹药：黑牌进鬼道持有者手牌
                e.debug_place(2, "sha");
            }
            e.debug_set_deck_top_by(|d| d.suit.is_spade() && (2..=9).contains(&d.rank));
            // 推进至该锦囊完成判定（判定区清空），途中改判/天妒/濒死按策略排干
            let mut guard = 0;
            while !e.debug_judge(owner).is_empty() {
                guard += 1;
                assert!(guard < 64, "judgement round {round} 未收敛");
                if e.outcome().is_some() {
                    break;
                }
                advance(&mut e, &mut m);
            }
            if e.outcome().is_some() {
                break;
            }
        }
        m.events = e.log().len() as u64;
        total_j.merge(&m);
        sink.dump("judgement", g, &m);
        println!(
            "game=judgement g={g} outcome={:?} {}",
            e.outcome().map(|o| o.winner),
            m.summary()
        );
        println!(
            "digest judgement g={g} seed={} hash={} rewrites={} finalized={} gained={}",
            base_seed + 2000 + g as u64,
            hex(&e.state_hash()),
            count_log(&e, |l| matches!(
                l,
                engine_native::LogEvent::JudgementRewritten { .. }
            )),
            count_log(&e, |l| matches!(
                l,
                engine_native::LogEvent::JudgementFinalized { .. }
            )),
            count_log(&e, |l| matches!(
                l,
                engine_native::LogEvent::JudgementGained { .. }
            )),
        );
    }
    println!("mode=judgement-aggregate {}", total_j.summary());

    // 分配统计（N09）：全部负载结束后输出，避免截断后续负载的分配；
    // 含 runner 自身（样本向量/字符串）开销。
    println!(
        "alloc total_bytes={} total_count={} peak_live_bytes={} live_at_exit_bytes={} (进程累计，含 runner 自身开销)",
        ALLOC_BYTES.load(Ordering::Relaxed),
        ALLOC_COUNT.load(Ordering::Relaxed),
        PEAK_LIVE_BYTES.load(Ordering::Relaxed),
        LIVE_BYTES.load(Ordering::Relaxed)
    );
    println!("mode=summary games={games} exit=ok");
}

fn count_log(e: &Engine, f: impl Fn(&engine_native::LogEvent) -> bool) -> usize {
    e.log().iter().filter(|l| f(l)).count()
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
