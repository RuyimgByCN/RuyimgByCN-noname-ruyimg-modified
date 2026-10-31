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
#[path = "nc12_bench/scenarios.rs"]
mod scenarios;
use scenarios::*;

/// 三档负载定义：目录名（= 安装来源目录）。
/// `probe` 会逐档实际导入并装载，冲突（重名牌/将/技能）如实报错。
const TIER_TABLE: &[(&str, &[&str])] = &[
    ("none", &[]),
    ("swd", &["shahai-swd-upstream-noname"]),
    ("huanle", &["huanle-upstream-noname"]),
    // 逐来源档（NC08 第 8 项「逐来源性能」用）：一个来源目录一档，便于按来源签署。
    ("gujian", &["shahai-gujian-upstream-noname"]),
    ("mtg", &["shahai-mtg-upstream-noname"]),
    ("wudian", &["wudian-upstream-noname"]),
    ("hearth", &["shahai-hearth-upstream-noname"]),
    ("yunchou", &["shahai-yunchou-upstream-noname"]),
    ("zhenfa", &["shahai-zhenfa-upstream-noname"]),
    ("gwent", &["shahai-gwent-upstream-noname"]),
    ("wuxing", &["wuxing-upstream-liuli"]),
    ("d3", &["d3-upstream-noname"]),
    ("cardpile", &["cardpile-upstream-noname"]),
    ("cardpile_liuli", &["cardpile-upstream-liuli"]),
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
            .map(|s| {
                s.split(',')
                    .map(|x| x.trim().to_string())
                    .filter(|x| !x.is_empty())
                    .collect()
            })
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
        return Stats {
            n: 0,
            min: 0,
            p50: 0,
            p95: 0,
            max: 0,
            mean: 0,
            sum: 0,
        };
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
            let live = HEAP_ALLOCATED
                .load(Relaxed)
                .saturating_sub(HEAP_FREED.load(Relaxed));
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
            let Some(ext_root) = &meta.extension_root else {
                continue;
            };
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
            let pack = load_pack_dir(&path).map_err(|e| format!("环境失败：{d} {e}"))?;
            packs.push(pack);
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
fn build_engine(
    tier: &str,
    host: Option<&PackHost>,
    seed: u64,
    seats: usize,
) -> Result<Engine, String> {
    match host {
        None => Ok(Engine::new(seed, seats)),
        Some(h) => {
            let refs = h.refs();
            Engine::new_with_extensions(seed, seats, &refs)
                .map_err(|e| format!("环境失败：{tier} 建局被拒 {e:?}"))
        }
    }
}

/// 按**资产形态**装载一个来源目录（NC12 逐来源档用）：
/// 1. `cards.json`（多数卡牌包；若同目录还有 `character.json` 则并入武将）；
/// 2. 仅 `character.json`（纯武将包，如 `d3`）；
/// 3. `pile.json`（牌堆补丁包，如 `cardpile-*`；走 `extension_pack_from_pile`）。
///
/// 返回 `ExtensionPack`；**不改变产品行为**，只让 bench 覆盖更多来源。
fn load_pack_dir(
    path: &std::path::Path,
) -> Result<engine_native::content::extension::ExtensionPack, String> {
    let assets = path.join("assets");
    let cards = assets.join("cards.json");
    if cards.is_file() {
        let json = std::fs::read_to_string(&cards)
            .map_err(|e| format!("读 {} 失败 {e}", cards.display()))?;
        let mut out = content_native_loader::bridge_pack(&json)
            .map_err(|e| format!("桥接 cards.json 失败 {e}"))?;
        let chars = assets.join("character.json");
        if chars.is_file() {
            if let Ok(cj) = std::fs::read_to_string(&chars) {
                if let Ok(b) = content_native_loader::bridge_generals(&cj) {
                    out.pack.generals = b.generals;
                }
            }
        }
        return Ok(out.pack);
    }
    let chars = assets.join("character.json");
    if chars.is_file() {
        let json = std::fs::read_to_string(&chars)
            .map_err(|e| format!("读 {} 失败 {e}", chars.display()))?;
        let mut out = content_native_loader::bridge_pack(&json)
            .map_err(|e| format!("桥接 character.json 失败 {e}"))?;
        if let Ok(b) = content_native_loader::bridge_generals(&json) {
            out.pack.generals = b.generals;
        }
        return Ok(out.pack);
    }
    let pile = assets.join("pile.json");
    if pile.is_file() {
        let json = std::fs::read_to_string(&pile)
            .map_err(|e| format!("读 {} 失败 {e}", pile.display()))?;
        let v: serde_json::Value =
            serde_json::from_str(&json).map_err(|e| format!("解析 pile.json 失败 {e}"))?;
        let source = v
            .get("source")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        let total = v.get("total").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
        let mut entries = Vec::new();
        for e in v
            .get("entries")
            .and_then(|x| x.as_array())
            .cloned()
            .unwrap_or_default()
        {
            let card = e
                .get("card")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            let mut counts: Vec<(String, u32)> = Vec::new();
            if let Some(obj) = e.get("counts").and_then(|x| x.as_object()) {
                for suit in ["spade", "heart", "club", "diamond"] {
                    if let Some(n) = obj.get(suit).and_then(|x| x.as_u64()) {
                        counts.push((suit.to_string(), n as u32));
                    }
                }
            }
            entries.push(engine_native::content::pile::PileEntry { card, counts });
        }
        return engine_native::content::pile::extension_pack_from_pile(
            engine_native::content::extension::ExtensionDef {
                name: "bench-pile",
                version: 1,
                description: "bench pile",
            },
            "bench_pile",
            &engine_native::content::pile::PileSpec {
                source,
                total,
                entries,
            },
        )
        .map_err(|e| format!("pile 构包失败 {e:?}"));
    }
    Err("资产目录既无 cards.json / character.json / pile.json".to_string())
}

fn load_tier(tier: &str) -> Result<Option<PackHost>, String> {
    // **逐包档**：`dir:<包目录名>` —— 直接按 drop-in 目录名装载一个包，
    // 便于用 shell 循环把 `probe` 跑到**每一个**包上（不必往 TIER_TABLE 里逐个登记）。
    if let Some(dir) = tier.strip_prefix("dir:") {
        let one = [dir];
        return PackHost::load(&one).map(Some);
    }
    let (_, dirs) = TIER_TABLE
        .iter()
        .find(|(name, _)| *name == tier)
        .ok_or_else(|| format!("未知档位 {tier}（可选 {}，或 dir:<目录名>）", tier_names()))?;
    if dirs.is_empty() {
        Ok(None)
    } else {
        PackHost::load(dirs).map(Some)
    }
}

fn tier_names() -> String {
    TIER_TABLE
        .iter()
        .map(|(n, _)| *n)
        .collect::<Vec<_>>()
        .join(",")
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
            (
                "reject_kinds",
                if self.reject_kinds.is_empty() {
                    "-".to_string()
                } else {
                    self.reject_kinds
                        .iter()
                        .map(|(k, v)| format!("{k}:{v}"))
                        .collect::<Vec<_>>()
                        .join("|")
                },
            ),
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
            (
                "panic_steps",
                if self.panics.is_empty() {
                    "-".to_string()
                } else {
                    self.panics
                        .iter()
                        .map(|(s, seat, _)| format!("{s}@seat{seat}"))
                        .collect::<Vec<_>>()
                        .join("|")
                },
            ),
            ("final_state", self.final_shape.clone()),
        ]
    }
}

fn reject_kind(r: &engine_native::RejectReason) -> String {
    let s = format!("{r:?}");
    s.split(['(', ' ', '{'])
        .next()
        .unwrap_or("Unknown")
        .to_string()
}

/// 规划 panic 后的兜底动作（**降级模式专用**，只用于让测量继续）：
/// 有开放选择时按 `min` 取前若干选项，否则推进阶段。该动作不经过 AI
/// 评分，因而该次运行必须标注 `degraded=true`，不得当作干净基线。
fn fallback_action(e: &Engine) -> GameAction {
    if let Some(c) = e.open_choice() {
        let n = c.spec.min.max(1) as usize;
        let sel: Vec<ChoiceOption> = c.spec.options.iter().take(n).copied().collect();
        if !sel.is_empty() {
            return GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            };
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
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            e.plan_action(seat, planner)
        }))
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
                *run.reject_kinds
                    .entry(format!("ApplyPlanError{err:?}"))
                    .or_default() += 1;
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
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
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
