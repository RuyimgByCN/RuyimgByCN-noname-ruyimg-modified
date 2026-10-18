//! NC12 效能基线测量工具（2026-10-18，**只测量不改产品行为**）。
//!
//! 口径与[收敛方案 §NC12](../../../docs/architecture/native-convergence-plan.md#nc12)、
//! [性能架构 §11](../../../noname-performance-architecture.md)一致：
//! 同设备、同工具链、同负载，区分「无扩展 / 单扩展 / 常用组合」三档，
//! 分别测量启动建局、规则结算尾延迟、状态复制与模拟、存读档、AI、
//! 长局内存。所有产出以 `RESULT` 前缀的制表符行打印，便于落盘与汇总。
//!
//! 运行（离线 release）：
//!
//! ```text
//! cargo run --offline --release -p engine-native --example nc12_bench -- probe
//! cargo run --offline --release -p engine-native --example nc12_bench -- build tiers=none,swd,huanle,combo samples=30
//! cargo run --offline --release -p engine-native --example nc12_bench -- drive tier=none seats=4 budget=64 lookahead=1 variant=0 seed=42007
//! cargo run --offline --release -p engine-native --example nc12_bench -- copy tier=none budget=64 steps=60 samples=30
//! cargo run --offline --release -p engine-native --example nc12_bench -- archive tier=swd seats=4 samples=30
//! cargo run --offline --release -p engine-native --example nc12_bench -- ai tier=none budget=64 seed=42007 cmds=200
//! cargo run --offline --release -p engine-native --example nc12_bench -- long tier=none seats=4 budget=8 min_cmds=300 max_games=4
//! ```
//!
//! 通用参数：`csv=<path>` 把全部原始样本（metric,index,value_ns）落盘。
//! 失败分两类：**环境失败**（包导入/装载、目录不存在）与**测量失败**
//! （对局未收敛、命令被拒、恢复不一致），两者都在输出中分别标注，
//! 不做静默替换或外推。

use engine_native::ai::{BoundedPlanner, Plan};
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::content::extension::{content_lock_id, ExtensionPack};
use engine_native::content::standard::standard_catalog;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

/// 三档负载定义：目录名（= 安装来源目录）。
/// `probe` 会逐档实际导入并装载，冲突（重名牌/将/技能）如实报错。
const TIER_TABLE: &[(&str, &[&str])] = &[
    ("none", &[]),
    ("swd", &["shahai-swd-upstream-noname"]),
    ("huanle", &["huanle-upstream-noname"]),
    (
        "combo",
        &[
            "huanle-upstream-noname",
            "shahai-swd-upstream-noname",
            "shahai-gujian-upstream-noname",
        ],
    ),
];

// ---------------------------------------------------------------------------
// 参数解析
// ---------------------------------------------------------------------------

#[derive(Default)]
struct Args {
    kv: BTreeMap<String, String>,
    positional: Option<String>,
}

impl Args {
    fn parse() -> Self {
        let mut kv = BTreeMap::new();
        let mut positional = None;
        for a in std::env::args().skip(1) {
            if let Some((k, v)) = a.split_once('=') {
                kv.insert(k.to_string(), v.to_string());
            } else if positional.is_none() {
                positional = Some(a);
            }
        }
        Self { kv, positional }
    }

    fn get(&self, k: &str) -> Option<&str> {
        self.kv.get(k).map(|s| s.as_str()).filter(|s| !s.is_empty())
    }

    fn u64(&self, k: &str, d: u64) -> u64 {
        self.get(k).and_then(|s| s.parse().ok()).unwrap_or(d)
    }

    fn usize(&self, k: &str, d: usize) -> usize {
        self.u64(k, d as u64) as usize
    }

    fn u8(&self, k: &str, d: u8) -> u8 {
        self.u64(k, d as u64) as u8
    }

    fn str(&self, k: &str, d: &str) -> String {
        self.get(k).unwrap_or(d).to_string()
    }

    fn csv(&self, k: &str) -> Vec<String> {
        self.get(k)
            .map(|s| s.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect())
            .unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------
// 统计与输出
// ---------------------------------------------------------------------------

/// 分位数用 nearest-rank：`v[ceil(p/100·n)-1]`（样本已升序）。
struct Stats {
    n: usize,
    min: u128,
    p50: u128,
    p95: u128,
    max: u128,
    mean: u128,
    sum: u128,
}

fn stats(input: &[u128]) -> Stats {
    let mut v = input.to_vec();
    v.sort_unstable();
    let n = v.len();
    if n == 0 {
        return Stats { n: 0, min: 0, p50: 0, p95: 0, max: 0, mean: 0, sum: 0 };
    }
    let at = |p: f64| -> u128 {
        let rank = ((p / 100.0) * n as f64).ceil() as usize;
        v[rank.saturating_sub(1).min(n - 1)]
    };
    let sum: u128 = v.iter().sum();
    Stats {
        n,
        min: v[0],
        p50: at(50.0),
        p95: at(95.0),
        max: v[n - 1],
        mean: sum / n as u128,
        sum,
    }
}

/// 原始样本落盘（`csv=<path>`）：metric,index,value_ns。
struct CsvSink {
    path: Option<PathBuf>,
    lines: Vec<String>,
}

impl CsvSink {
    fn new(args: &Args) -> Self {
        Self {
            path: args.get("csv").map(PathBuf::from),
            lines: vec!["metric,index,value_ns".to_string()],
        }
    }

    fn push(&mut self, metric: &str, samples: &[u128]) {
        if self.path.is_none() {
            return;
        }
        for (i, v) in samples.iter().enumerate() {
            self.lines.push(format!("{metric},{i},{v}"));
        }
    }

    fn flush(&self) {
        if let Some(p) = &self.path {
            let _ = std::fs::write(p, self.lines.join("\n") + "\n");
            println!("RESULT\tcsv={}", p.display());
        }
    }
}

fn emit_stats(scenario: &str, tier: &str, config: &str, metric: &str, s: &Stats) {
    println!(
        "RESULT\tscenario={scenario}\ttier={tier}\tconfig={config}\tmetric={metric}\tn={}\tmin_ns={}\tp50_ns={}\tp95_ns={}\tmax_ns={}\tmean_ns={}\ttotal_ns={}",
        s.n, s.min, s.p50, s.p95, s.max, s.mean, s.sum
    );
}

/// 吞吐换算（naive 版本，保留供后续口径扩展；当前场景用 emit_kv 直接算）。
#[allow(dead_code)]
fn emit_throughput(scenario: &str, tier: &str, config: &str, metric: &str, s: &Stats) {
    println!(
        "RESULT\tscenario={scenario}\ttier={tier}\tconfig={config}\tmetric={metric}\tn={}\tns_per_op_p50={}\tns_per_op_mean={}\tops_per_sec_mean={}",
        s.n,
        s.p50,
        s.mean,
        if s.mean == 0 { 0 } else { 1_000_000_000 / s.mean }
    );
}

fn emit_kv(scenario: &str, tier: &str, config: &str, kv: &[(&str, String)]) {
    let mut line = format!("RESULT\tscenario={scenario}\ttier={tier}\tconfig={config}");
    for (k, v) in kv {
        line.push_str(&format!("\t{k}={v}"));
    }
    println!("{line}");
}

fn hex32(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

// ---------------------------------------------------------------------------
// 长局内存两条口径。
//
// 1) 进程 RSS：macOS/Linux `ps -o rss=`（KB）。本机沙箱禁止 `ps`/`top`
//    （`Operation not permitted`）——该函数此时返回 None，外部改用
//    `/usr/bin/time -l` 的 maximum resident set size 作为进程峰值 RSS。
// 2) 进程内计数分配器（`NC12_ALLOC=1` 打开）：堆分配/释放字节、分配次数
//    与峰值活字节。打开后计时含原子操作开销，该次运行的延迟数字**不作为
//    延迟口径**（沿用 n09_runner 的 PLAIN 对照结论）。
// ---------------------------------------------------------------------------

fn rss_kb() -> Option<u64> {
    let pid = std::process::id().to_string();
    let out = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &pid])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8_lossy(&out.stdout).trim().parse().ok()
}

/// 计数分配器：只统计「打开计数之后」的分配事件。
struct CountingAlloc;

static ALLOC_ON: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static HEAP_ALLOCATED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static HEAP_FREED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static HEAP_CALLS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static HEAP_PEAK_LIVE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn alloc_counting_on() -> bool {
    ALLOC_ON.load(std::sync::atomic::Ordering::Relaxed)
}

fn alloc_enable() {
    ALLOC_ON.store(true, std::sync::atomic::Ordering::Relaxed);
}

/// (累计分配字节, 累计释放字节, 分配次数, 峰值活字节)
fn heap_snapshot() -> (u64, u64, u64, u64) {
    use std::sync::atomic::Ordering::Relaxed;
    (
        HEAP_ALLOCATED.load(Relaxed),
        HEAP_FREED.load(Relaxed),
        HEAP_CALLS.load(Relaxed),
        HEAP_PEAK_LIVE.load(Relaxed),
    )
}

unsafe impl std::alloc::GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        use std::sync::atomic::Ordering::Relaxed;
        if alloc_counting_on() {
            HEAP_ALLOCATED.fetch_add(layout.size() as u64, Relaxed);
            HEAP_CALLS.fetch_add(1, Relaxed);
            let live = HEAP_ALLOCATED.load(Relaxed).saturating_sub(HEAP_FREED.load(Relaxed));
            HEAP_PEAK_LIVE.fetch_max(live, Relaxed);
        }
        std::alloc::System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        use std::sync::atomic::Ordering::Relaxed;
        if alloc_counting_on() {
            HEAP_FREED.fetch_add(layout.size() as u64, Relaxed);
        }
        std::alloc::System.dealloc(ptr, layout)
    }
}

#[global_allocator]
static NC12_ALLOCATOR: CountingAlloc = CountingAlloc;

// ---------------------------------------------------------------------------
// 扩展宿主：真实安装路径（import_extensions → 内容锁 → bridge_pack）
// ---------------------------------------------------------------------------

fn repo_extensions_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn install_options() -> content_runtime::InstallOptions {
    content_runtime::InstallOptions {
        activate: true,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    }
}

struct PackHost {
    /// 临时安装目录必须活到进程结束（内容锁与包资产从这里读）。
    _tmp: tempfile::TempDir,
    packs: Vec<ExtensionPack>,
    package_ids: Vec<String>,
    /// 全量 drop-in 根导入耗时（环境成本，不计入建局）。
    import_ns: u128,
    /// 请求包 `cards.json` 读取 + 桥接耗时。
    bridge_ns: u128,
    installed_total: usize,
    skipped_total: usize,
}

impl PackHost {
    fn load(dirs: &[&str]) -> Result<Self, String> {
        let root = repo_extensions_root();
        if !root.is_dir() {
            return Err(format!("环境失败：扩展根不存在 {}", root.display()));
        }
        let tmp = tempfile::tempdir().map_err(|e| format!("环境失败 tempdir: {e}"))?;
        let store = content_runtime::PackageStore::open(tmp.path().join("store"))
            .map_err(|e| format!("环境失败 PackageStore::open: {e}"))?;
        let t_import = Instant::now();
        let report = content_runtime::import_extensions(&store, &root, &install_options())
            .map_err(|e| format!("环境失败 import_extensions: {e}"))?;
        let import_ns = t_import.elapsed().as_nanos();
        // 目录名 → 安装元数据（extension_root 的末段即来源目录名）。
        let mut by_dir: BTreeMap<String, (String, PathBuf)> = BTreeMap::new();
        for meta in &report.installed {
            let Some(ext_root) = &meta.extension_root else { continue };
            let path = PathBuf::from(ext_root);
            let Some(name) = path.file_name().map(|s| s.to_string_lossy().into_owned()) else {
                continue;
            };
            by_dir.insert(name, (meta.package_id.clone(), path));
        }
        let t_bridge = Instant::now();
        let mut packs = Vec::new();
        let mut package_ids = Vec::new();
        for d in dirs {
            let (pid, path) = by_dir
                .get(*d)
                .ok_or_else(|| format!("环境失败：目录 {d} 未被 import_extensions 安装"))?;
            let cards = path.join("assets/cards.json");
            let json = std::fs::read_to_string(&cards)
                .map_err(|e| format!("环境失败：读 {} 失败 {e}", cards.display()))?;
            let out = content_native_loader::bridge_pack(&json)
                .map_err(|e| format!("环境失败：桥接 {d} 失败 {e}"))?;
            packs.push(out.pack);
            package_ids.push(pid.clone());
        }
        let bridge_ns = t_bridge.elapsed().as_nanos();
        Ok(Self {
            _tmp: tmp,
            packs,
            package_ids,
            import_ns,
            bridge_ns,
            installed_total: report.installed.len(),
            skipped_total: report.skipped.len(),
        })
    }

    fn refs(&self) -> Vec<&ExtensionPack> {
        self.packs.iter().collect()
    }

    fn lock_id(&self) -> u64 {
        content_lock_id(&self.refs())
    }
}

/// 三档建局：无扩展走 `Engine::new`，有扩展走 `Engine::new_with_extensions`。
fn build_engine(tier: &str, host: Option<&PackHost>, seed: u64, seats: usize) -> Result<Engine, String> {
    match host {
        None => Ok(Engine::new(seed, seats)),
        Some(h) => {
            let refs = h.refs();
            Engine::new_with_extensions(seed, seats, &refs)
                .map_err(|e| format!("环境失败：{tier} 建局被拒 {e:?}"))
        }
    }
}

fn load_tier(tier: &str) -> Result<Option<PackHost>, String> {
    let (_, dirs) = TIER_TABLE
        .iter()
        .find(|(name, _)| *name == tier)
        .ok_or_else(|| format!("未知档位 {tier}（可选 {}）", tier_names()))?;
    if dirs.is_empty() {
        Ok(None)
    } else {
        PackHost::load(dirs).map(Some)
    }
}

fn tier_names() -> String {
    TIER_TABLE.iter().map(|(n, _)| *n).collect::<Vec<_>>().join(",")
}

fn tier_dirs_csv(tier: &str) -> String {
    TIER_TABLE
        .iter()
        .find(|(n, _)| *n == tier)
        .map(|(_, d)| d.join("+"))
        .unwrap_or_else(|| "<unknown>".into())
}

// ---------------------------------------------------------------------------
// 对局形状
// ---------------------------------------------------------------------------

fn state_shape(e: &Engine) -> String {
    let core = e.state_core();
    let res = e.resolution();
    let hands: usize = core.players.iter().map(|p| p.hand.len()).sum();
    let equips: usize = core.players.iter().map(|p| p.equips.len()).sum();
    let judge: usize = core.players.iter().map(|p| p.judge.len()).sum();
    let off_court: usize = core.players.iter().map(|p| p.off_court.len()).sum();
    let lianyaohu: usize = core.players.iter().map(|p| p.lianyaohu_store.len()).sum();
    let zhucheng: usize = core.players.iter().map(|p| p.zhucheng.len()).sum();
    format!(
        "players={} frames={} parks={} skills={} vm_marks={} vm_piles={} vm_once={} vm_flags={} deck={} discard={} table={} hand={} equips={} judge={} off_court={} lianyaohu_store={} zhucheng={} log={} revision={}",
        core.players.len(),
        res.stack.len(),
        res.parks.len(),
        core.skills.len(),
        core.vm_aux.marks.len(),
        core.vm_aux.piles.len(),
        core.vm_aux.once.len(),
        core.vm_aux.flags.len(),
        core.deck.len(),
        core.discard.len(),
        core.table.len(),
        hands,
        equips,
        judge,
        off_court,
        lianyaohu,
        zhucheng,
        e.log().len(),
        e.revision()
    )
}

fn catalog_shape(tier: &str, e: &Engine) -> Vec<(&'static str, String)> {
    let c = e.catalog();
    vec![
        ("tier_dirs", tier_dirs_csv(tier)),
        ("cards", c.cards.len().to_string()),
        ("skills", c.skills.len().to_string()),
        ("extension_generals", c.extension_generals.len().to_string()),
        ("ruleset_version", e.ruleset_version().to_string()),
    ]
}

// ---------------------------------------------------------------------------
// 整局规划驱动（BoundedPlanner）
// ---------------------------------------------------------------------------

struct GameRun {
    steps: usize,
    /// NC12 长程防护：规划模拟提交触发内核不变量 panic 的候选数（Plan.sim_faults 汇总）。
    sim_faults: u32,
    accepted: usize,
    pending: usize,
    rejected: usize,
    suspended: usize,
    resumes: usize,
    reject_kinds: BTreeMap<String, usize>,
    finished: bool,
    stalled: bool,
    wall_exceeded: bool,
    step_capped: bool,
    events: usize,
    wall_ns: u128,
    plan_ns: Vec<u128>,
    submit_ns: Vec<u128>,
    cmd_ns: Vec<u128>,
    nodes: Vec<u128>,
    budget_exhausted: usize,
    timed_out: usize,
    /// 规划 panic（catch_panics=1 时捕获；默认直接进程中止）。
    panics: Vec<(usize, u8, String)>,
    final_shape: String,
}

impl GameRun {
    fn new() -> Self {
        Self {
            steps: 0,
            sim_faults: 0,
            accepted: 0,
            pending: 0,
            rejected: 0,
            suspended: 0,
            resumes: 0,
            reject_kinds: BTreeMap::new(),
            finished: false,
            stalled: false,
            wall_exceeded: false,
            step_capped: false,
            events: 0,
            wall_ns: 0,
            plan_ns: Vec::new(),
            submit_ns: Vec::new(),
            cmd_ns: Vec::new(),
            nodes: Vec::new(),
            budget_exhausted: 0,
            timed_out: 0,
            panics: Vec::new(),
            final_shape: String::new(),
        }
    }

    fn total_nodes(&self) -> u128 {
        self.nodes.iter().sum()
    }

    fn summary_kv(&self) -> Vec<(&'static str, String)> {
        vec![
            ("steps", self.steps.to_string()),
            ("accepted", self.accepted.to_string()),
            ("pending", self.pending.to_string()),
            ("suspended", self.suspended.to_string()),
            ("resumes", self.resumes.to_string()),
            ("rejected", self.rejected.to_string()),
            ("reject_kinds", if self.reject_kinds.is_empty() {
                "-".to_string()
            } else {
                self.reject_kinds
                    .iter()
                    .map(|(k, v)| format!("{k}:{v}"))
                    .collect::<Vec<_>>()
                    .join("|")
            }),
            ("finished", self.finished.to_string()),
            ("stalled", self.stalled.to_string()),
            ("wall_exceeded", self.wall_exceeded.to_string()),
            ("step_capped", self.step_capped.to_string()),
            ("events", self.events.to_string()),
            ("wall_ns", self.wall_ns.to_string()),
            ("nodes_total", self.total_nodes().to_string()),
            ("budget_exhausted_steps", self.budget_exhausted.to_string()),
            ("deadline_timeout_steps", self.timed_out.to_string()),
            ("plan_panics", self.panics.len().to_string()),
            ("panic_steps", if self.panics.is_empty() {
                "-".to_string()
            } else {
                self.panics.iter().map(|(s, seat, _)| format!("{s}@seat{seat}")).collect::<Vec<_>>().join("|")
            }),
            ("final_state", self.final_shape.clone()),
        ]
    }
}

fn reject_kind(r: &engine_native::RejectReason) -> String {
    let s = format!("{r:?}");
    s.split(['(', ' ', '{']).next().unwrap_or("Unknown").to_string()
}

/// 规划 panic 后的兜底动作（**降级模式专用**，只用于让测量继续）：
/// 有开放选择时按 `min` 取前若干选项，否则推进阶段。该动作不经过 AI
/// 评分，因而该次运行必须标注 `degraded=true`，不得当作干净基线。
fn fallback_action(e: &Engine) -> GameAction {
    if let Some(c) = e.open_choice() {
        let n = c.spec.min.max(1) as usize;
        let sel: Vec<ChoiceOption> = c.spec.options.iter().take(n).copied().collect();
        if !sel.is_empty() {
            return GameAction::Answer { choice_id: c.id, selection: sel };
        }
    }
    GameAction::NextPhase
}

/// 规划一次：`catch_panics` 时捕获 `plan_action` 内 panic 并返回兜底计划
/// （降级模式；`Err(msg)` 表示本次规划崩过）。默认不捕获——进程按首次
/// 失败中止，保留第一现场。
fn plan_with_catch(
    e: &Engine,
    seat: u8,
    planner: &BoundedPlanner,
    catch_panics: bool,
    step: usize,
) -> (Plan, Option<String>) {
    let attempt = if catch_panics {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| e.plan_action(seat, planner)))
    } else {
        Ok(e.plan_action(seat, planner))
    };
    match attempt {
        Ok(p) => (p, None),
        Err(payload) => {
            let msg = payload
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "<非字符串 panic 载荷>".into());
            eprintln!(
                "PANIC caught in plan_action: step={step} seat={seat} revision={} msg={msg}",
                e.revision()
            );
            (
                Plan {
                    sim_faults: 0,
                    seat,
                    revision: e.revision(),
                    action: fallback_action(e),
                    nodes_used: 0,
                    budget_exhausted: false,
                    timed_out: false,
                    cancelled: false,
                },
                Some(msg),
            )
        }
    }
}

/// 用有界规划器驱动对局：每命令 = `plan_action`（克隆模拟）+ `apply_plan`
/// （权威提交）。分别记录规划耗时、规则结算耗时与端到端命令耗时。
/// `catch_panics=1` 时捕获 `plan_action` 内 panic 并以兜底动作续行
///（降级模式，运行标注 `degraded=true`）；默认不捕获，进程按首次失败中止。
fn drive_game(
    e: &mut Engine,
    planner: &BoundedPlanner,
    max_steps: usize,
    wall_budget_ns: u128,
    sample_hook: &mut dyn FnMut(&Engine, usize),
    sample_every: usize,
    trace: bool,
    catch_panics: bool,
) -> GameRun {
    let t_wall = Instant::now();
    let mut run = GameRun::new();
    let mut consecutive_rejects = 0usize;
    while e.outcome().is_none() && run.steps < max_steps {
        if t_wall.elapsed().as_nanos() > wall_budget_ns {
            run.wall_exceeded = true;
            break;
        }
        if sample_every > 0 && run.steps % sample_every == 0 {
            sample_hook(e, run.steps);
        }
        let seat = match e.open_choice() {
            Some(c) => c.observer,
            None => e.turn().current,
        };
        if trace {
            // 崩溃定位：规划前打印步号/座位/修订号与状态形状（stderr 无缓冲）。
            eprintln!(
                "TRACE step={} seat={} revision={} choice={} phase={:?} turn={} {}",
                run.steps,
                seat,
                e.revision(),
                e.open_choice().is_some(),
                e.turn().phase,
                e.turn().current,
                state_shape(e)
            );
        }
        let t_plan = Instant::now();
        let (plan, panic_msg) = plan_with_catch(e, seat, planner, catch_panics, run.steps);
        if let Some(msg) = panic_msg {
            run.panics.push((run.steps, seat, msg));
        }
        // NC12 长程防护：把「规划模拟内部被捕获的 panic 数」计入运行统计（不静默）
        run.sim_faults = run.sim_faults.saturating_add(plan.sim_faults);
        let plan_ns = t_plan.elapsed().as_nanos();
        let t_submit = Instant::now();
        let out = e.apply_plan(&plan);
        let submit_ns = t_submit.elapsed().as_nanos();
        run.plan_ns.push(plan_ns);
        run.submit_ns.push(submit_ns);
        run.cmd_ns.push(plan_ns + submit_ns);
        run.nodes.push(plan.nodes_used as u128);
        if plan.budget_exhausted {
            run.budget_exhausted += 1;
        }
        if plan.timed_out {
            run.timed_out += 1;
        }
        run.steps += 1;
        // 时间片段语义：Suspended 需经 resume() 续行至终局结果。
        let mut settled = out;
        let mut resume_guard = 0;
        while matches!(settled, Ok(CommandOutcome::Suspended)) && resume_guard < 64 {
            settled = Ok(e.resume());
            run.resumes += 1;
            resume_guard += 1;
        }
        match settled {
            Ok(CommandOutcome::Accepted { .. }) => {
                run.accepted += 1;
                consecutive_rejects = 0;
            }
            Ok(CommandOutcome::Pending { .. }) => {
                run.pending += 1;
                consecutive_rejects = 0;
            }
            Ok(CommandOutcome::Suspended) => {
                run.suspended += 1;
                consecutive_rejects = 0;
            }
            Ok(CommandOutcome::Rejected(r)) => {
                run.rejected += 1;
                consecutive_rejects += 1;
                *run.reject_kinds.entry(reject_kind(&r)).or_default() += 1;
            }
            Err(err) => {
                run.rejected += 1;
                consecutive_rejects += 1;
                *run.reject_kinds.entry(format!("ApplyPlanError{err:?}")).or_default() += 1;
            }
        }
        if consecutive_rejects >= 64 {
            run.stalled = true;
            break;
        }
    }
    run.finished = e.outcome().is_some();
    run.step_capped = !run.finished && !run.stalled && !run.wall_exceeded && run.steps >= max_steps;
    run.wall_ns = t_wall.elapsed().as_nanos();
    run.events = e.log().len();
    run.final_shape = state_shape(e);
    run
}

fn planner_of(budget: usize, lookahead: u8, variant: u8) -> BoundedPlanner {
    BoundedPlanner::new(budget)
        .with_lookahead(lookahead)
        .with_policy_variant(variant)
}

fn config_str(seats: usize, budget: usize, lookahead: u8, variant: u8) -> String {
    format!("seats={seats},budget={budget},lookahead={lookahead},variant={variant}")
}

// ---------------------------------------------------------------------------
// 场景：probe —— 三档实际装载能力与目录规模
// ---------------------------------------------------------------------------

fn scenario_probe(args: &Args, csv: &mut CsvSink) {
    let wanted: Vec<String> = if args.csv("tiers").is_empty() {
        TIER_TABLE.iter().map(|(n, _)| n.to_string()).collect()
    } else {
        args.csv("tiers")
    };
    println!("# NC12 probe：逐档 import_extensions → bridge_pack → 建局 → 单次规划");
    for tier in &wanted {
        let t_load = Instant::now();
        let host = match load_tier(tier) {
            Ok(h) => h,
            Err(e) => {
                emit_kv("probe", tier, "", &[("result", "FAIL".into()), ("error", e)]);
                continue;
            }
        };
        let load_ns = t_load.elapsed().as_nanos();
        let mut kv: Vec<(&'static str, String)> = vec![
            ("result", "OK".into()),
            ("dirs", tier_dirs_csv(tier)),
            ("load_ns", load_ns.to_string()),
        ];
        if let Some(h) = &host {
            kv.push(("import_ns", h.import_ns.to_string()));
            kv.push(("bridge_ns", h.bridge_ns.to_string()));
            kv.push(("installed_packages", h.installed_total.to_string()));
            kv.push(("skipped_dirs", h.skipped_total.to_string()));
            kv.push(("package_ids", h.package_ids.join("+")));
            kv.push(("content_lock_id", h.lock_id().to_string()));
        }
        let seed = args.u64("seed", 42007);
        let seats = args.usize("seats", 4);
        let t_build = Instant::now();
        match build_engine(tier, host.as_ref(), seed, seats) {
            Ok(mut e) => {
                kv.push(("build_ns", t_build.elapsed().as_nanos().to_string()));
                for (k, v) in catalog_shape(tier, &e) {
                    kv.push((k, v));
                }
                kv.push(("state0", state_shape(&e)));
                let snap = e.save();
                let bytes = encode_archive(&snap);
                kv.push(("archive_bytes", bytes.len().to_string()));
                let planner = planner_of(args.usize("budget", 64), args.u8("lookahead", 1), args.u8("variant", 0));
                let seat = match e.open_choice() {
                    Some(c) => c.observer,
                    None => e.turn().current,
                };
                let t_plan = Instant::now();
                let plan = e.plan_action(seat, &planner);
                kv.push(("plan_smoke_ns", t_plan.elapsed().as_nanos().to_string()));
                kv.push(("plan_smoke_nodes", plan.nodes_used.to_string()));
                let out = e.apply_plan(&plan);
                kv.push(("apply_smoke", format!("{out:?}").split(['(', ' ']).next().unwrap_or("?").to_string()));
                // 建局计时（为 probe 提供最小样本，正式样本见 build 场景）。
                let samples = args.usize("samples", 5);
                let mut build_samples = Vec::new();
                for i in 0..samples {
                    let t = Instant::now();
                    let r = build_engine(tier, host.as_ref(), seed + i as u64, seats);
                    build_samples.push(t.elapsed().as_nanos());
                    std::hint::black_box(r.is_ok());
                }
                csv.push(&format!("probe_build_ns_{tier}"), &build_samples);
                emit_stats("probe", tier, "", "build_ns", &stats(&build_samples));
                emit_kv("probe", tier, "", &kv);
            }
            Err(e) => {
                kv.push(("build", "FAIL".into()));
                kv.push(("error", e));
                emit_kv("probe", tier, "", &kv);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 场景：build —— 启动/建局（无扩展 / 单扩展 / 常用组合）
// ---------------------------------------------------------------------------

fn scenario_build(args: &Args, csv: &mut CsvSink) {
    let tiers: Vec<String> = if args.csv("tiers").is_empty() {
        TIER_TABLE.iter().map(|(n, _)| n.to_string()).collect()
    } else {
        args.csv("tiers")
    };
    let samples = args.usize("samples", 30);
    let seats = args.usize("seats", 4);
    let seed = args.u64("seed", 42007);
    println!("# NC12 build：标准目录构建 + 三档建局（samples={samples}, seats={seats}, seed={seed}）");
    // 标准目录单独计时（`Engine::new` 内部同样构建一次）。
    let mut cat = Vec::new();
    for _ in 0..samples {
        let t = Instant::now();
        let c = standard_catalog();
        cat.push(t.elapsed().as_nanos());
        std::hint::black_box(c.ruleset_version);
    }
    csv.push("standard_catalog_ns", &cat);
    emit_stats("build", "none", &config_str(seats, 0, 0, 0), "standard_catalog_ns", &stats(&cat));

    for tier in &tiers {
        let t_load = Instant::now();
        let host = match load_tier(tier) {
            Ok(h) => h,
            Err(e) => {
                emit_kv("build", tier, "", &[("result", "FAIL".into()), ("error", e)]);
                continue;
            }
        };
        let load_ns = t_load.elapsed().as_nanos();
        let mut samples_ns = Vec::new();
        let mut first_err: Option<String> = None;
        let mut ruleset = 0u64;
        let mut bytes = 0usize;
        let mut shape = String::new();
        let mut cat_kv: Vec<(&'static str, String)> = Vec::new();
        for i in 0..samples {
            let t = Instant::now();
            let r = build_engine(tier, host.as_ref(), seed + i as u64, seats);
            samples_ns.push(t.elapsed().as_nanos());
            match r {
                Ok(e) => {
                    ruleset = e.ruleset_version();
                    if i == 0 {
                        bytes = encode_archive(&e.save()).len();
                        shape = state_shape(&e);
                        cat_kv = catalog_shape(tier, &e);
                    }
                }
                Err(err) => {
                    if first_err.is_none() {
                        first_err = Some(err);
                    }
                }
            }
        }
        let mut kv: Vec<(&'static str, String)> = vec![
            ("dirs", tier_dirs_csv(tier)),
            ("seats", seats.to_string()),
            ("samples", samples.to_string()),
            ("host_load_ns", load_ns.to_string()),
            ("ruleset_version", ruleset.to_string()),
            ("archive_bytes_at_open", bytes.to_string()),
            ("state_at_open", shape),
            ("first_error", first_err.unwrap_or_else(|| "-".into())),
        ];
        if let Some(h) = &host {
            kv.push(("import_ns", h.import_ns.to_string()));
            kv.push(("bridge_ns", h.bridge_ns.to_string()));
            kv.push(("package_ids", h.package_ids.join("+")));
            kv.push(("installed_packages", h.installed_total.to_string()));
            kv.push(("content_lock_id", h.lock_id().to_string()));
        }
        kv.extend(cat_kv);
        csv.push(&format!("build_ns_{tier}"), &samples_ns);
        emit_stats("build", tier, &config_str(seats, 0, 0, 0), "engine_build_ns", &stats(&samples_ns));
        emit_kv("build", tier, &config_str(seats, 0, 0, 0), &kv);
    }
}

// ---------------------------------------------------------------------------
// 场景：drive —— 整局规划驱动，每命令尾延迟
// ---------------------------------------------------------------------------

fn scenario_drive(args: &Args, csv: &mut CsvSink) {
    let tier = args.str("tier", "none");
    let seats = args.usize("seats", 4);
    let budget = args.usize("budget", 64);
    let lookahead = args.u8("lookahead", 1);
    let variant = args.u8("variant", 0);
    let games = args.usize("games", 1);
    let seed = args.u64("seed", 42007);
    let max_steps = args.usize("max_steps", 4000);
    let wall_secs = args.u64("wall_secs", 600);
    let cfg = config_str(seats, budget, lookahead, variant);
    let trace = args.get("trace").is_some();
    let catch_panics = args.get("catch_panics").is_some();
    println!(
        "# NC12 drive：tier={tier} dirs={} {cfg} games={games} seed={seed} max_steps={max_steps} wall_secs={wall_secs}",
        tier_dirs_csv(&tier)
    );
    let host = match load_tier(&tier) {
        Ok(h) => h,
        Err(e) => {
            emit_kv("drive", &tier, &cfg, &[("result", "FAIL".into()), ("error", e)]);
            return;
        }
    };
    let planner = planner_of(budget, lookahead, variant);
    let mut games_out: Vec<GameRun> = Vec::new();
    for g in 0..games {
        let mut e = match build_engine(&tier, host.as_ref(), seed + g as u64, seats) {
            Ok(e) => e,
            Err(err) => {
                emit_kv("drive", &tier, &format!("{cfg},game={g}"), &[("result", "FAIL".into()), ("error", err)]);
                continue;
            }
        };
        let run = drive_game(
            &mut e,
            &planner,
            max_steps,
            wall_secs as u128 * 1_000_000_000,
            &mut |_e, _step| {},
            0,
            trace,
            catch_panics,
        );
        println!(
            "RESULT\tscenario=drive\ttier={tier}\tconfig={cfg},game={g}\tgame={g}\tsteps={}\taccepted={}\tpending={}\trejected={}\tfinished={}\tstalled={}\tplan_panics={}\tdegraded={}\twall_ns={}\tevents={}\tnodes_total={}\tfinal_state={}",
            run.steps, run.accepted, run.pending, run.rejected, run.finished, run.stalled,
            run.panics.len(), !run.panics.is_empty(),
            run.wall_ns, run.events, run.total_nodes(), run.final_shape
        );
        if !run.plan_ns.is_empty() {
            csv.push(&format!("drive_{tier}_plan_ns_g{g}"), &run.plan_ns);
            csv.push(&format!("drive_{tier}_submit_ns_g{g}"), &run.submit_ns);
            csv.push(&format!("drive_{tier}_cmd_ns_g{g}"), &run.cmd_ns);
            emit_stats("drive", &tier, &format!("{cfg},game={g}"), "plan_ns", &stats(&run.plan_ns));
            emit_stats("drive", &tier, &format!("{cfg},game={g}"), "submit_ns", &stats(&run.submit_ns));
            emit_stats("drive", &tier, &format!("{cfg},game={g}"), "cmd_ns", &stats(&run.cmd_ns));
        }
        emit_kv("drive", &tier, &format!("{cfg},game={g}"), &run.summary_kv());
        games_out.push(run);
    }
    // 跨局汇总（样本拼接；不得与其它档位混算）。
    let mut plan_all = Vec::new();
    let mut submit_all = Vec::new();
    let mut cmd_all = Vec::new();
    for r in &games_out {
        plan_all.extend_from_slice(&r.plan_ns);
        submit_all.extend_from_slice(&r.submit_ns);
        cmd_all.extend_from_slice(&r.cmd_ns);
    }
    if !cmd_all.is_empty() {
        emit_stats("drive", &tier, &format!("{cfg},all_games"), "plan_ns", &stats(&plan_all));
        emit_stats("drive", &tier, &format!("{cfg},all_games"), "submit_ns", &stats(&submit_all));
        emit_stats("drive", &tier, &format!("{cfg},all_games"), "cmd_ns", &stats(&cmd_all));
        let total_wall: u128 = games_out.iter().map(|r| r.wall_ns).sum();
        let total_steps: usize = games_out.iter().map(|r| r.steps).sum();
        let total_nodes: u128 = games_out.iter().map(|r| r.total_nodes()).sum();
        emit_kv(
            "drive",
            &tier,
            &format!("{cfg},all_games"),
            &[
                ("result", "OK".into()),
                ("games", games_out.len().to_string()),
                ("steps", total_steps.to_string()),
                ("finished_games", games_out.iter().filter(|r| r.finished).count().to_string()),
                ("stalled_games", games_out.iter().filter(|r| r.stalled).count().to_string()),
                ("plan_panics", games_out.iter().map(|r| r.panics.len()).sum::<usize>().to_string()),
                ("degraded", games_out.iter().any(|r| !r.panics.is_empty()).to_string()),
                ("wall_ns", total_wall.to_string()),
                ("ns_per_step_wall", if total_steps == 0 { 0 } else { total_wall / total_steps as u128 }.to_string()),
                ("nodes_total", total_nodes.to_string()),
                ("nodes_per_sec_plan", {
                    let plan_sum: u128 = plan_all.iter().sum();
                    if plan_sum == 0 { 0 } else { total_nodes * 1_000_000_000 / plan_sum }
                }
                .to_string()),
                ("events", games_out.iter().map(|r| r.events).sum::<usize>().to_string()),
            ],
        );
    }
}

// ---------------------------------------------------------------------------
// 场景：copy —— 状态复制 / 模拟 / 状态规模增长
// ---------------------------------------------------------------------------

fn scenario_copy(args: &Args, csv: &mut CsvSink) {
    let tier = args.str("tier", "none");
    let seats = args.usize("seats", 4);
    let budget = args.usize("budget", 64);
    let lookahead = args.u8("lookahead", 1);
    let variant = args.u8("variant", 0);
    let steps = args.usize("steps", 60);
    let samples = args.usize("samples", 30);
    let seed = args.u64("seed", 42007);
    let max_steps = args.usize("max_steps", 4000);
    let wall_secs = args.u64("wall_secs", 600);
    let cfg = config_str(seats, budget, lookahead, variant);
    let trace = args.get("trace").is_some();
    let catch_panics = args.get("catch_panics").is_some();
    println!("# NC12 copy：tier={tier} dirs={} {cfg} steps={steps} samples={samples}", tier_dirs_csv(&tier));
    let host = match load_tier(&tier) {
        Ok(h) => h,
        Err(e) => {
            emit_kv("copy", &tier, &cfg, &[("result", "FAIL".into()), ("error", e)]);
            return;
        }
    };
    let mut e = match build_engine(&tier, host.as_ref(), seed, seats) {
        Ok(e) => e,
        Err(err) => {
            emit_kv("copy", &tier, &cfg, &[("result", "FAIL".into()), ("error", err)]);
            return;
        }
    };
    let planner = planner_of(budget, lookahead, variant);

    // 开局（步数=0）与推进 steps 步后的对照。
    let stages: Vec<(&str, usize)> = vec![("open", 0), ("mid", steps)];
    let mut done_steps = 0usize;
    for (stage, target) in stages {
        if target > done_steps {
            while done_steps < target && e.outcome().is_none() {
                let seat = match e.open_choice() {
                    Some(c) => c.observer,
                    None => e.turn().current,
                };
                let plan = e.plan_action(seat, &planner);
                if e.apply_plan(&plan).is_err() {
                    break;
                }
                done_steps += 1;
            }
        }
        let stage_cfg = format!("{cfg},stage={stage},steps={done_steps}");
        emit_kv(
            "copy",
            &tier,
            &stage_cfg,
            &[
                ("state", state_shape(&e)),
                ("finished", e.outcome().is_some().to_string()),
            ],
        );
        // save()：快照克隆
        let mut save_ns = Vec::new();
        let mut acc = 0usize;
        for _ in 0..samples {
            let t = Instant::now();
            let s = e.save();
            save_ns.push(t.elapsed().as_nanos());
            acc += s.core.players.len();
        }
        // Engine::clone()：规划隔离的实际复制面
        let mut clone_ns = Vec::new();
        for _ in 0..samples {
            let t = Instant::now();
            let c = e.clone();
            clone_ns.push(t.elapsed().as_nanos());
            acc += c.revision() as usize;
        }
        // plan_action：克隆 + 未知牌重分配 + 候选模拟
        let mut plan_ns = Vec::new();
        let mut nodes = Vec::new();
        let seat = match e.open_choice() {
            Some(c) => c.observer,
            None => e.turn().current,
        };
        for _ in 0..samples {
            let t = Instant::now();
            let p = e.plan_action(seat, &planner);
            plan_ns.push(t.elapsed().as_nanos());
            nodes.push(p.nodes_used as u128);
            std::hint::black_box(p.action.clone());
        }
        // 归档编码（字节存档）
        let mut enc_ns = Vec::new();
        for _ in 0..samples {
            let t = Instant::now();
            let s = e.save();
            let bytes = encode_archive(&s);
            enc_ns.push(t.elapsed().as_nanos());
            acc += bytes.len();
        }
        let bytes = encode_archive(&e.save()).len();
        csv.push(&format!("copy_{tier}_{stage}_save_ns"), &save_ns);
        csv.push(&format!("copy_{tier}_{stage}_clone_ns"), &clone_ns);
        csv.push(&format!("copy_{tier}_{stage}_plan_ns"), &plan_ns);
        csv.push(&format!("copy_{tier}_{stage}_encode_ns"), &enc_ns);
        emit_stats("copy", &tier, &stage_cfg, "save_ns", &stats(&save_ns));
        emit_stats("copy", &tier, &stage_cfg, "engine_clone_ns", &stats(&clone_ns));
        emit_stats("copy", &tier, &stage_cfg, "plan_action_ns", &stats(&plan_ns));
        emit_stats("copy", &tier, &stage_cfg, "encode_archive_ns", &stats(&enc_ns));
        emit_kv(
            "copy",
            &tier,
            &stage_cfg,
            &[
                ("archive_bytes", bytes.to_string()),
                ("plan_nodes_p50", stats(&nodes).p50.to_string()),
                ("plan_nodes_max", stats(&nodes).max.to_string()),
                ("nodes_per_sec_plan", {
                    let plan_sum: u128 = plan_ns.iter().sum();
                    if plan_sum == 0 { 0 } else { nodes.iter().sum::<u128>() * 1_000_000_000 / plan_sum }
                }
                .to_string()),
                ("black_box_acc", acc.to_string()),
            ],
        );
    }

    // 推进过程中状态规模增长曲线（players/frames/skills 等）。
    let mut e2 = match build_engine(&tier, host.as_ref(), seed, seats) {
        Ok(e) => e,
        Err(err) => {
            emit_kv("copy", &tier, &format!("{cfg},growth"), &[("result", "FAIL".into()), ("error", err)]);
            return;
        }
    };
    let every = args.usize("growth_every", 20);
    let mut grass = Vec::new();
    let mut r = drive_game(
        &mut e2,
        &planner,
        max_steps,
        wall_secs as u128 * 1_000_000_000,
        &mut |eng: &Engine, step: usize| {
            grass.push((step, state_shape(eng), encode_archive(&eng.save()).len()));
        },
        every,
        trace,
        catch_panics,
    );
    r.final_shape = state_shape(&e2);
    println!(
        "RESULT\tscenario=copy\ttier={tier}\tconfig={cfg},growth\tsteps={}\tfinished={}\tstalled={}\twall_ns={}\tfinal_state={}",
        r.steps, r.finished, r.stalled, r.wall_ns, r.final_shape
    );
    for (step, shape, bytes) in &grass {
        println!("RESULT\tscenario=copy\ttier={tier}\tconfig={cfg},growth\tstep={step}\tarchive_bytes={bytes}\tstate={shape}");
    }
}

/// 所有权校验**计入**的公开牌区（`snap/validation.rs::validate_card_ownership`
/// 口径）：手牌/装备/判定/离场区 + 牌库/弃牌堆/结算牌桌 + 挂起机器持牌。
/// 未计入的特殊区（炼妖壶收纳、筑城、卡牌改造表等）单独标注，避免把
/// 「校验器不认的区」误判成「牌区重复」。
fn counted_zones(snap: &engine_native::StateSnapshot, card: u32, name: &str) -> String {
    let count = |v: &Vec<u32>| v.iter().filter(|c| **c == card).count();
    let mut z: Vec<String> = Vec::new();
    let core = &snap.core;
    let push = |label: String, n: usize, out: &mut Vec<String>| {
        if n > 0 {
            out.push(format!("{label}={n}"));
        }
    };
    push("deck".into(), count(&core.deck), &mut z);
    push("discard".into(), count(&core.discard), &mut z);
    push("table".into(), count(&core.table), &mut z);
    let mut uncounted: Vec<String> = Vec::new();
    for (i, p) in core.players.iter().enumerate() {
        push(format!("hand[{i}]"), count(&p.hand), &mut z);
        push(format!("equips[{i}]"), count(&p.equips), &mut z);
        push(format!("judge[{i}]"), count(&p.judge), &mut z);
        push(format!("off_court[{i}]"), count(&p.off_court), &mut z);
        push(format!("lianyaohu_store[{i}]"), count(&p.lianyaohu_store), &mut uncounted);
        push(format!("zhucheng[{i}]"), count(&p.zhucheng), &mut uncounted);
    }
    if core.declared_kinds.contains_key(&card) {
        uncounted.push("declared_kinds".into());
    }
    if core.declared_slots.contains_key(&card) {
        uncounted.push("declared_slots".into());
    }
    if core.card_natures.contains_key(&card) {
        uncounted.push("card_natures".into());
    }
    format!(
        "counted=[{}] uncounted=[{}] parks={} card_name={}",
        z.join("+"),
        uncounted.join("+"),
        snap.resolution.parks.len(),
        name
    )
}

// ---------------------------------------------------------------------------
// 场景：archive —— 存读档（含扩展目录）与归档字节
// ---------------------------------------------------------------------------

fn scenario_archive(args: &Args, csv: &mut CsvSink) {
    let tier = args.str("tier", "none");
    let seats = args.usize("seats", 4);
    let samples = args.usize("samples", 30);
    let seed = args.u64("seed", 42007);
    let steps = args.usize("steps", 80);
    let budget = args.usize("budget", 64);
    let cfg = format!("seats={seats},samples={samples},steps={steps}");
    println!("# NC12 archive：tier={tier} dirs={} {cfg}", tier_dirs_csv(&tier));
    let host = match load_tier(&tier) {
        Ok(h) => h,
        Err(e) => {
            emit_kv("archive", &tier, &cfg, &[("result", "FAIL".into()), ("error", e)]);
            return;
        }
    };
    let mut e = match build_engine(&tier, host.as_ref(), seed, seats) {
        Ok(e) => e,
        Err(err) => {
            emit_kv("archive", &tier, &cfg, &[("result", "FAIL".into()), ("error", err)]);
            return;
        }
    };
    let planner = planner_of(budget, 1, 0);
    let mut stage_steps = 0usize;
    for stage in ["open", "played"] {
        if stage == "played" {
            while stage_steps < steps && e.outcome().is_none() {
                let seat = match e.open_choice() {
                    Some(c) => c.observer,
                    None => e.turn().current,
                };
                let plan = e.plan_action(seat, &planner);
                if e.apply_plan(&plan).is_err() {
                    break;
                }
                stage_steps += 1;
            }
        }
        let stage_cfg = format!("{cfg},stage={stage},steps={stage_steps}");
        let mut save_ns = Vec::new();
        let mut enc_ns = Vec::new();
        let mut dec_ns = Vec::new();
        let mut restore_ns = Vec::new();
        let mut roundtrip_fail = 0usize;
        let mut hash_mismatch = 0usize;
        let mut acc = 0usize;
        for _ in 0..samples {
            let t = Instant::now();
            let snap = e.save();
            save_ns.push(t.elapsed().as_nanos());
            let t = Instant::now();
            let bytes = encode_archive(&snap);
            enc_ns.push(t.elapsed().as_nanos());
            let t = Instant::now();
            let decoded = match decode_archive(&bytes) {
                Ok(d) => d,
                Err(err) => {
                    roundtrip_fail += 1;
                    eprintln!("decode_archive 失败: {err:?}");
                    continue;
                }
            };
            dec_ns.push(t.elapsed().as_nanos());
            let t = Instant::now();
            let restored = match host.as_ref() {
                None => Engine::restore(&decoded),
                Some(h) => {
                    let refs = h.refs();
                    Engine::restore_with_extensions(&decoded, &refs)
                }
            };
            restore_ns.push(t.elapsed().as_nanos());
            match restored {
                Ok(r) => {
                    if r.state_hash() != e.state_hash() {
                        hash_mismatch += 1;
                    }
                    acc += r.revision() as usize;
                }
                Err(err) => {
                    roundtrip_fail += 1;
                    eprintln!("restore 失败: {err:?}");
                    if let engine_native::SnapshotError::IdOutOfBounds { what, got } = &err {
                        let name = e
                            .catalog()
                            .cards
                            .get(*got as usize)
                            .map(|c| c.name)
                            .unwrap_or("<越界牌 id>");
                        println!(
                            "RESULT\tscenario=archive\ttier={tier}\tconfig={stage_cfg}\tdiagnostic=restore_reject\twhat={what}\tcard={got}\tcard_name={name}\t{}",
                            counted_zones(&decoded, *got as u32, name)
                        );
                    }
                }
            }
        }
        let bytes = encode_archive(&e.save()).len();
        csv.push(&format!("archive_{tier}_{stage}_save_ns"), &save_ns);
        csv.push(&format!("archive_{tier}_{stage}_encode_ns"), &enc_ns);
        csv.push(&format!("archive_{tier}_{stage}_decode_ns"), &dec_ns);
        csv.push(&format!("archive_{tier}_{stage}_restore_ns"), &restore_ns);
        emit_stats("archive", &tier, &stage_cfg, "save_ns", &stats(&save_ns));
        emit_stats("archive", &tier, &stage_cfg, "encode_archive_ns", &stats(&enc_ns));
        emit_stats("archive", &tier, &stage_cfg, "decode_archive_ns", &stats(&dec_ns));
        emit_stats("archive", &tier, &stage_cfg, "restore_ns", &stats(&restore_ns));
        emit_kv(
            "archive",
            &tier,
            &stage_cfg,
            &[
                ("result", if roundtrip_fail == 0 && hash_mismatch == 0 { "OK".into() } else { "FAIL".into() }),
                ("archive_bytes", bytes.to_string()),
                ("roundtrip_failures", roundtrip_fail.to_string()),
                ("state_hash_mismatch", hash_mismatch.to_string()),
                ("state_hash", hex32(&e.state_hash())),
                ("state", state_shape(&e)),
                ("roundtrip_token", acc.to_string()),
            ],
        );
    }
}

// ---------------------------------------------------------------------------
// 场景：ai —— 同种子同预算的策略矩阵（lookahead 1/2 × variant 0/1）
// ---------------------------------------------------------------------------

fn scenario_ai(args: &Args, csv: &mut CsvSink) {
    let tier = args.str("tier", "none");
    let seats = args.usize("seats", 4);
    let seed = args.u64("seed", 42007);
    let cmds = args.usize("cmds", 200);
    let budgets = if args.csv("budgets").is_empty() {
        vec![args.usize("budget", 64)]
    } else {
        args.csv("budgets").iter().filter_map(|s| s.parse().ok()).collect()
    };
    let configs_arg = args.csv("configs");
    let configs: Vec<(u8, u8)> = if configs_arg.is_empty() {
        vec![(1, 0), (1, 1), (2, 0), (2, 1)]
    } else {
        configs_arg
            .iter()
            .filter_map(|c| c.split_once('/').and_then(|(l, v)| Some((l.parse().ok()?, v.parse().ok()?))))
            .collect()
    };
    println!(
        "# NC12 ai：tier={tier} dirs={} seats={seats} seed={seed} cmds={cmds} budgets={:?} configs={:?}",
        tier_dirs_csv(&tier),
        budgets,
        configs
    );
    let catch_panics = args.get("catch_panics").is_some();
    let host = match load_tier(&tier) {
        Ok(h) => h,
        Err(e) => {
            emit_kv("ai", &tier, "", &[("result", "FAIL".into()), ("error", e)]);
            return;
        }
    };
    let mut decision_sequences: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for &budget in &budgets {
        for &(lookahead, variant) in &configs {
            let cfg = config_str(seats, budget, lookahead, variant);
            let mut e = match build_engine(&tier, host.as_ref(), seed, seats) {
                Ok(e) => e,
                Err(err) => {
                    emit_kv("ai", &tier, &cfg, &[("result", "FAIL".into()), ("error", err)]);
                    continue;
                }
            };
            let planner = planner_of(budget, lookahead, variant);
            let mut plan_ns = Vec::new();
            let mut nodes = Vec::new();
            let mut decisions = Vec::new();
            let mut accepted = 0usize;
            let mut rejected = 0usize;
            let mut steps = 0usize;
            let mut panics: Vec<(usize, u8, String)> = Vec::new();
            let t_all = Instant::now();
            while steps < cmds && e.outcome().is_none() {
                let seat = match e.open_choice() {
                    Some(c) => c.observer,
                    None => e.turn().current,
                };
                let t = Instant::now();
                let (plan, panic_msg) = plan_with_catch(&e, seat, &planner, catch_panics, steps);
                if let Some(msg) = panic_msg {
                    panics.push((steps, seat, msg));
                }
                plan_ns.push(t.elapsed().as_nanos());
                nodes.push(plan.nodes_used as u128);
                decisions.push(format!("{:?}", plan.action));
                match e.apply_plan(&plan) {
                    Ok(CommandOutcome::Rejected(_)) | Err(_) => rejected += 1,
                    Ok(_) => accepted += 1,
                }
                steps += 1;
            }
            let wall_ns = t_all.elapsed().as_nanos();
            let total_nodes: u128 = nodes.iter().sum();
            let plan_sum: u128 = plan_ns.iter().sum();
            csv.push(&format!("ai_{tier}_b{budget}_l{lookahead}_v{variant}_plan_ns"), &plan_ns);
            emit_stats("ai", &tier, &cfg, "plan_action_ns", &stats(&plan_ns));
            let ns = stats(&nodes);
            emit_kv(
                "ai",
                &tier,
                &cfg,
                &[
                    ("metric", "nodes_per_plan".into()),
                    ("n", ns.n.to_string()),
                    ("min", ns.min.to_string()),
                    ("p50", ns.p50.to_string()),
                    ("p95", ns.p95.to_string()),
                    ("max", ns.max.to_string()),
                    ("mean", ns.mean.to_string()),
                    ("total", ns.sum.to_string()),
                ],
            );
            emit_kv(
                "ai",
                &tier,
                &cfg,
                &[
                    ("result", "OK".into()),
                    ("steps", steps.to_string()),
                    ("accepted", accepted.to_string()),
                    ("rejected", rejected.to_string()),
                    ("finished", e.outcome().is_some().to_string()),
                    ("plan_panics", panics.len().to_string()),
                    ("degraded", (!panics.is_empty()).to_string()),
                    ("panic_steps", if panics.is_empty() {
                        "-".to_string()
                    } else {
                        panics.iter().map(|(s, seat, _)| format!("{s}@seat{seat}")).collect::<Vec<_>>().join("|")
                    }),
                    ("wall_ns", wall_ns.to_string()),
                    ("plan_total_ns", plan_sum.to_string()),
                    ("nodes_total", total_nodes.to_string()),
                    (
                        "nodes_per_sec",
                        if plan_sum == 0 { 0 } else { total_nodes * 1_000_000_000 / plan_sum }.to_string(),
                    ),
                    (
                        "ns_per_step_wall",
                        if steps == 0 { 0 } else { wall_ns / steps as u128 }.to_string(),
                    ),
                    ("final_state", state_shape(&e)),
                ],
            );
            decision_sequences.insert(cfg, decisions);
        }
    }
    // 同种子下策略差异：与「lookahead=1,variant=0」逐步对比。
    for &budget in &budgets {
        let base_key = config_str(seats, budget, 1, 0);
        let Some(base) = decision_sequences.get(&base_key) else { continue };
        for &(lookahead, variant) in &configs {
            let cfg = config_str(seats, budget, lookahead, variant);
            let Some(cur) = decision_sequences.get(&cfg) else { continue };
            let common = base.len().min(cur.len());
            let diff = (0..common).filter(|&i| base[i] != cur[i]).count();
            emit_kv(
                "ai",
                &tier,
                &cfg,
                &[
                    ("compare_base", base_key.clone()),
                    ("compare_steps", common.to_string()),
                    ("decision_diff_steps", diff.to_string()),
                    ("decision_len_base", base.len().to_string()),
                    ("decision_len_cur", cur.len().to_string()),
                ],
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 场景：long —— 长局内存（RSS 增长）
// ---------------------------------------------------------------------------

fn scenario_long(args: &Args, _csv: &mut CsvSink) {
    let tier = args.str("tier", "none");
    let seats = args.usize("seats", 4);
    let budget = args.usize("budget", 8);
    let min_cmds = args.usize("min_cmds", 300);
    let max_games = args.usize("max_games", 4);
    let seed = args.u64("seed", 42007);
    let every = args.usize("sample_every", 25);
    let trace = args.get("trace").is_some();
    let catch_panics = args.get("catch_panics").is_some();
    let cfg = config_str(seats, budget, 1, 0);
    println!(
        "# NC12 long：tier={tier} dirs={} {cfg} min_cmds={min_cmds} max_games={max_games} seed={seed} sample_every={every}",
        tier_dirs_csv(&tier)
    );
    let rss0 = rss_kb();
    let heap0 = heap_snapshot();
    emit_kv(
        "long",
        &tier,
        &cfg,
        &[
            ("phase", "process_start".into()),
            ("rss_kb", rss0.map(|v| v.to_string()).unwrap_or_else(|| "NA".into())),
            ("rss_source", "ps -o rss= (沙箱禁止时 NA；外部用 /usr/bin/time -l)".into()),
            ("alloc_counting", alloc_counting_on().to_string()),
            ("heap_allocated_bytes", heap0.0.to_string()),
            ("heap_freed_bytes", heap0.1.to_string()),
            ("heap_alloc_calls", heap0.2.to_string()),
            ("pid", std::process::id().to_string()),
        ],
    );
    let host = match load_tier(&tier) {
        Ok(h) => h,
        Err(e) => {
            emit_kv("long", &tier, &cfg, &[("result", "FAIL".into()), ("error", e)]);
            return;
        }
    };
    let planner = planner_of(budget, 1, 0);
    let mut total_cmds = 0usize;
    let mut rss_peak = rss0.unwrap_or(0);
    let mut rss_series: Vec<(usize, u64, String)> = Vec::new();
    for g in 0..max_games {
        if total_cmds >= min_cmds {
            break;
        }
        let mut e = match build_engine(&tier, host.as_ref(), seed + g as u64, seats) {
            Ok(e) => e,
            Err(err) => {
                emit_kv("long", &tier, &format!("{cfg},game={g}"), &[("result", "FAIL".into()), ("error", err)]);
                return;
            }
        };
        let rss_before = rss_kb().unwrap_or(0);
        let mut samples: Vec<(usize, u64, String)> = Vec::new();
        let mut run = drive_game(
            &mut e,
            &planner,
            args.usize("max_steps", 4000),
            args.u64("wall_secs", 600) as u128 * 1_000_000_000,
            &mut |eng: &Engine, step: usize| {
                if let Some(rss) = rss_kb() {
                    samples.push((step, rss, state_shape(eng)));
                }
            },
            every,
            trace,
            catch_panics,
        );
        run.final_shape = state_shape(&e);
        let rss_after = rss_kb().unwrap_or(0);
        rss_peak = rss_peak.max(rss_after);
        total_cmds += run.steps;
        let heap_game = heap_snapshot();
        println!(
            "RESULT\tscenario=long\ttier={tier}\tconfig={cfg},game={g}\tgame={g}\tsteps={}\taccepted={}\tfinished={}\tstalled={}\tplan_panics={}\tsim_faults={}\twall_ns={}\trss_before_kb={rss_before}\trss_after_kb={rss_after}\trss_delta_kb={}\theap_allocated_bytes={}\theap_freed_bytes={}\theap_live_bytes={}\tfinal_state={}",
            run.steps, run.accepted, run.finished, run.stalled, run.panics.len(), run.sim_faults, run.wall_ns,
            rss_after as i64 - rss_before as i64,
            heap_game.0, heap_game.1, heap_game.0.saturating_sub(heap_game.1),
            run.final_shape
        );
        for (step, rss, shape) in &samples {
            let heap = heap_snapshot();
            println!(
                "RESULT\tscenario=long\ttier={tier}\tconfig={cfg},game={g}\tstep={step}\trss_kb={rss}\theap_live_bytes={}\tstate={shape}",
                heap.0.saturating_sub(heap.1)
            );
            rss_series.push((*step, *rss, shape.clone()));
        }
        emit_stats(
            "long",
            &tier,
            &format!("{cfg},game={g}"),
            "plan_action_ns",
            &stats(&run.plan_ns),
        );
    }
    // 同场景重复后的资源累积检查：额外跑 20 次 存档+编码 往返。
    let mut acc = 0usize;
    if let Ok(e) = build_engine(&tier, host.as_ref(), seed, seats) {
        let t = Instant::now();
        for _ in 0..20 {
            let snap = e.save();
            let bytes = encode_archive(&snap);
            acc += bytes.len() + snap.core.players.len();
        }
        emit_kv(
            "long",
            &tier,
            &cfg,
            &[
                ("phase", "post_save_cycles".into()),
                ("save_cycle_total_ns", t.elapsed().as_nanos().to_string()),
                ("rss_kb", rss_kb().map(|v| v.to_string()).unwrap_or_else(|| "NA".into())),
                ("token", acc.to_string()),
            ],
        );
    }
    let rss_end = rss_kb().unwrap_or(0);
    rss_peak = rss_peak.max(rss_end);
    let heap1 = heap_snapshot();
    emit_kv(
        "long",
        &tier,
        &cfg,
        &[
            ("result", "OK".into()),
            ("total_cmds", total_cmds.to_string()),
            ("alloc_counting", alloc_counting_on().to_string()),
            ("heap_allocated_bytes", heap1.0.saturating_sub(heap0.0).to_string()),
            ("heap_freed_bytes", heap1.1.saturating_sub(heap0.1).to_string()),
            ("heap_live_delta_bytes", heap1.0.saturating_sub(heap0.0).saturating_sub(heap1.1.saturating_sub(heap0.1)).to_string()),
            ("heap_alloc_calls", heap1.2.to_string()),
            ("heap_peak_live_bytes", heap1.3.to_string()),
            ("rss_start_kb", rss0.map(|v| v.to_string()).unwrap_or_else(|| "NA".into())),
            ("rss_end_kb", rss_end.to_string()),
            ("rss_peak_kb", rss_peak.to_string()),
            (
                "rss_delta_kb",
                rss0.map(|v| rss_end as i64 - v as i64).unwrap_or(0).to_string(),
            ),
            ("rss_samples", rss_series.len().to_string()),
            ("min_cmds_reached", (total_cmds >= min_cmds).to_string()),
        ],
    );
}

// ---------------------------------------------------------------------------

fn main() {
    let args = Args::parse();
    let alloc_metric = args.get("alloc").map(|v| v == "1").unwrap_or(false);
    if alloc_metric {
        alloc_enable();
    }
    let scenario = args
        .get("scenario")
        .map(|s| s.to_string())
        .or_else(|| args.positional.clone())
        .unwrap_or_else(|| "help".into());
    if scenario == "help" {
        println!("scenarios: probe | build | drive | copy | archive | ai | long");
        println!("tiers: {}", tier_names());
        return;
    }
    println!(
        "# nc12_bench scenario={scenario} pid={} profile={}",
        std::process::id(),
        if cfg!(debug_assertions) { "debug" } else { "release" }
    );
    let mut csv = CsvSink::new(&args);
    match scenario.as_str() {
        "probe" => scenario_probe(&args, &mut csv),
        "build" => scenario_build(&args, &mut csv),
        "drive" => scenario_drive(&args, &mut csv),
        "copy" => scenario_copy(&args, &mut csv),
        "archive" => scenario_archive(&args, &mut csv),
        "ai" => scenario_ai(&args, &mut csv),
        "long" => scenario_long(&args, &mut csv),
        other => {
            eprintln!("未知场景 {other}（probe|build|drive|copy|archive|ai|long）");
            std::process::exit(2);
        }
    }
    csv.flush();
}
