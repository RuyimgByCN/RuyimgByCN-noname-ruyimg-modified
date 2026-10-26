//! 架构评估用临时测量（不改仓库源码）：
//! - scale：未被任何玩家持有的触发技能数量 N 对权威提交/规划耗时的影响
//! - growth：对局推进（日志变长）时 Engine::clone / plan_action / 分配量的变化
use engine_native::ai::BoundedPlanner;
use engine_native::catalog::TriggerMeta;
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook, VmSkillDecl};
use engine_native::event::{EventKind, Timing};
use engine_native::{CommandOutcome, Engine};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use rule_api as rule_api_size;

struct Counting;
static ALLOCS: AtomicU64 = AtomicU64::new(0);
static BYTES: AtomicU64 = AtomicU64::new(0);
unsafe impl std::alloc::GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: std::alloc::Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        BYTES.fetch_add(l.size() as u64, Ordering::Relaxed);
        std::alloc::System.alloc(l)
    }
    unsafe fn dealloc(&self, p: *mut u8, l: std::alloc::Layout) {
        std::alloc::System.dealloc(p, l)
    }
    unsafe fn realloc(&self, p: *mut u8, l: std::alloc::Layout, n: usize) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        BYTES.fetch_add(n as u64, Ordering::Relaxed);
        std::alloc::System.realloc(p, l, n)
    }
}
#[global_allocator]
static GA: Counting = Counting;

fn allocs() -> (u64, u64) {
    (ALLOCS.load(Ordering::Relaxed), BYTES.load(Ordering::Relaxed))
}

const DSL: &str = "\
skill arch_dummy(990101) {
  trigger phase_end owner_is_target {
    effect draw_cards(self, 1);
  }
}
";

const KINDS: &[EventKind] = &[
    EventKind::PhaseBegin,
    EventKind::PhaseEnd,
    EventKind::CardUsed,
    EventKind::CardsMoved,
    EventKind::CardsDrawn,
    EventKind::Damage,
    EventKind::DamageBefore,
    EventKind::CardsDiscarded,
    EventKind::CardsLost,
    EventKind::CardResponded,
];

fn dummy_pack(n: usize) -> ExtensionPack {
    let pkg = content_compiler::compile_source_named("arch-dummy", DSL).expect("DSL");
    let trig = pkg.trigger_index.iter().find(|t| t.module.contains("arch_dummy")).expect("trig");
    let module = pkg.modules.iter().find(|m| m.name == trig.module).expect("module").clone();
    let vm_skills = (0..n)
        .map(|i| VmSkillDecl {
            name: format!("arch_dummy_{i}"),
            module: module.clone(),
            trigger: TriggerMeta {
                event: KINDS[i % KINDS.len()],
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            },
            filter: engine_native::rvm_bridge::VmFilterPolicy::OwnerIsTarget,
        })
        .collect();
    ExtensionPack {
        def: ExtensionDef { name: "arch-dummy", version: 1, description: "arch bench" },
        generals: Vec::new(),
        cards: Vec::new(),
        host_skills: Vec::new(),
        vm_skills,
        init_hook: InitHook::None,
        enabled: true,
    }
}

fn pct(v: &mut Vec<u128>, p: f64) -> u128 {
    if v.is_empty() {
        return 0;
    }
    v.sort_unstable();
    let i = ((p / 100.0) * v.len() as f64).ceil() as usize;
    v[i.saturating_sub(1).min(v.len() - 1)]
}

struct Step {
    log_len: usize,
    plan_ns: u128,
    submit_ns: u128,
    nodes: usize,
    plan_allocs: u64,
    plan_bytes: u64,
    submit_allocs: u64,
    submit_bytes: u64,
    clone_ns: u128,
    hash_ns: u128,
    fmt_ns: u128,
}

fn drive(e: &mut Engine, planner: &BoundedPlanner, max_steps: usize, deep: bool) -> Vec<Step> {
    let mut out = Vec::new();
    let mut rejects = 0;
    while e.outcome().is_none() && out.len() < max_steps {
        let seat = match e.open_choice() {
            Some(c) => c.observer,
            None => e.turn().current,
        };
        let (mut clone_ns, mut hash_ns, mut fmt_ns) = (0, 0, 0);
        if deep {
            let t = Instant::now();
            for _ in 0..5 {
                std::hint::black_box(e.clone());
            }
            clone_ns = t.elapsed().as_nanos() / 5;
            let t = Instant::now();
            for _ in 0..5 {
                std::hint::black_box(e.state_hash());
            }
            hash_ns = t.elapsed().as_nanos() / 5;
            // 复刻 plan_action 每节点无条件构造的 pre_context 诊断串
            let t = Instant::now();
            for _ in 0..5 {
                let res = e.resolution();
                let frames: Vec<String> = res
                    .stack
                    .iter()
                    .map(|f| format!("{:?}/{:?}/{:?}", f.id, f.kind, f.data))
                    .collect();
                let parks: Vec<String> =
                    res.parks.iter().map(|p| format!("{:?}", p.machine.snapshot())).collect();
                std::hint::black_box(format!(
                    "PRE rev={} phase={:?} frames=[{}] parks=[{}]",
                    e.revision(),
                    e.turn().phase,
                    frames.join(" ; "),
                    parks.join(" ; ")
                ));
            }
            fmt_ns = t.elapsed().as_nanos() / 5;
        }
        let log_len = e.log().len();
        let a0 = allocs();
        let t = Instant::now();
        let plan = e.plan_action(seat, planner);
        let plan_ns = t.elapsed().as_nanos();
        let a1 = allocs();
        let t = Instant::now();
        let mut r = e.apply_plan(&plan);
        while matches!(r, Ok(CommandOutcome::Suspended)) {
            r = Ok(e.resume());
        }
        let submit_ns = t.elapsed().as_nanos();
        let a2 = allocs();
        match r {
            Ok(CommandOutcome::Rejected(_)) | Err(_) => rejects += 1,
            _ => rejects = 0,
        }
        out.push(Step {
            log_len,
            plan_ns,
            submit_ns,
            nodes: plan.nodes_used,
            plan_allocs: a1.0 - a0.0,
            plan_bytes: a1.1 - a0.1,
            submit_allocs: a2.0 - a1.0,
            submit_bytes: a2.1 - a1.1,
            clone_ns,
            hash_ns,
            fmt_ns,
        });
        if rejects >= 64 {
            eprintln!("stalled");
            break;
        }
    }
    out
}

fn scale(ns: &[usize], seats: usize) {
    println!("# scale: 未持有触发技能 N 对每命令耗时的影响（{seats} 座，3 局，planner budget=32）");
    println!("N\tcatalog_skills\tsteps\tevents\tsubmit_p50\tsubmit_p95\tsubmit_total_ms\tplan_p50\tplan_p95\tplan_total_ms");
    for &n in ns {
        let pack = dummy_pack(n);
        let planner = BoundedPlanner::new(32);
        let (mut sub, mut pl) = (Vec::new(), Vec::new());
        let (mut steps, mut events, mut skills) = (0, 0, 0);
        for seed in 42007..42010u64 {
            let mut e = if n == 0 {
                Engine::new(seed, seats)
            } else {
                Engine::new_with_extensions(seed, seats, &[&pack]).expect("build")
            };
            if std::env::var("GRANT").is_ok() {
                let gens = engine_native::content::standard::STANDARD_GENERALS;
                for p in 0..seats {
                    let g = gens[(p * 5 + 1) % gens.len()].0;
                    e.debug_grant_general(p as u8, g);
                }
            }
            let instances = e.state_core().skills.len();
            if seed == 42007 { eprintln!("N={n} skill_instances={instances}"); }
            skills = e.catalog().skills.len();
            let run = drive(&mut e, &planner, 800, false);
            steps += run.len();
            events += e.log().len();
            sub.extend(run.iter().map(|s| s.submit_ns));
            pl.extend(run.iter().map(|s| s.plan_ns));
        }
        let st: u128 = sub.iter().sum();
        let pt: u128 = pl.iter().sum();
        println!(
            "{n}\t{skills}\t{steps}\t{events}\t{}\t{}\t{:.2}\t{}\t{}\t{:.2}",
            pct(&mut sub, 50.0),
            pct(&mut sub, 95.0),
            st as f64 / 1e6,
            pct(&mut pl, 50.0),
            pct(&mut pl, 95.0),
            pt as f64 / 1e6
        );
    }
}

fn growth(seats: usize, budget: usize, games: u64) {
    println!("# growth: 对局推进时的成本（{seats} 座，planner budget={budget}，{games} 局，标准目录）");
    let planner = BoundedPlanner::new(budget);
    let mut all = Vec::new();
    for seed in 0..games {
        let mut e = Engine::new(42007 + seed, seats);
        all.extend(drive(&mut e, &planner, 2000, true));
    }
    let mut maxlog = all.iter().map(|s| s.log_len).max().unwrap_or(0);
    if maxlog == 0 {
        maxlog = 1;
    }
    let buckets = 4;
    println!("log_len_bucket\tsteps\tclone_p50\thash_p50\tfmt_p50\tplan_p50\tnodes_p50\tplan_ns_per_node_p50\tplan_allocs_p50\tplan_KB_p50\tsubmit_p50\tsubmit_allocs_p50\tsubmit_KB_p50");
    for b in 0..buckets {
        let lo = maxlog * b / buckets;
        let hi = maxlog * (b + 1) / buckets;
        let sel: Vec<&Step> = all
            .iter()
            .filter(|s| s.log_len >= lo && (s.log_len < hi || (b == buckets - 1 && s.log_len <= hi)))
            .collect();
        if sel.is_empty() {
            continue;
        }
        let mut c: Vec<u128> = sel.iter().map(|s| s.clone_ns).collect();
        let mut h: Vec<u128> = sel.iter().map(|s| s.hash_ns).collect();
        let mut f: Vec<u128> = sel.iter().map(|s| s.fmt_ns).collect();
        let mut p: Vec<u128> = sel.iter().map(|s| s.plan_ns).collect();
        let mut nn: Vec<u128> = sel.iter().map(|s| s.nodes as u128).collect();
        let mut ppn: Vec<u128> =
            sel.iter().filter(|s| s.nodes > 0).map(|s| s.plan_ns / s.nodes as u128).collect();
        let mut pa: Vec<u128> = sel.iter().map(|s| s.plan_allocs as u128).collect();
        let mut pb: Vec<u128> = sel.iter().map(|s| s.plan_bytes as u128 / 1024).collect();
        let mut sn: Vec<u128> = sel.iter().map(|s| s.submit_ns).collect();
        let mut sa: Vec<u128> = sel.iter().map(|s| s.submit_allocs as u128).collect();
        let mut sb: Vec<u128> = sel.iter().map(|s| s.submit_bytes as u128 / 1024).collect();
        println!(
            "{lo}-{hi}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            sel.len(),
            pct(&mut c, 50.0),
            pct(&mut h, 50.0),
            pct(&mut f, 50.0),
            pct(&mut p, 50.0),
            pct(&mut nn, 50.0),
            pct(&mut ppn, 50.0),
            pct(&mut pa, 50.0),
            pct(&mut pb, 50.0),
            pct(&mut sn, 50.0),
            pct(&mut sa, 50.0),
            pct(&mut sb, 50.0)
        );
    }
    let total_plan: u128 = all.iter().map(|s| s.plan_ns).sum();
    let total_sub: u128 = all.iter().map(|s| s.submit_ns).sum();
    let total_allocs: u64 = all.iter().map(|s| s.plan_allocs + s.submit_allocs).sum();
    println!(
        "TOTAL steps={} plan_ms={:.1} submit_ms={:.1} allocs_per_step={:.0} max_log_len={maxlog}",
        all.len(),
        total_plan as f64 / 1e6,
        total_sub as f64 / 1e6,
        total_allocs as f64 / all.len() as f64
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("scale") => {
            let seats = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(4);
            scale(&[0, 500, 2000, 6000], seats);
        }
        Some("growth") => {
            let seats = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(4);
            let budget = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(64);
            let games = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(3);
            growth(seats, budget, games);
        }
        Some("sizes") => {
            use engine_native::state::{GameStateCore, PlayerState, TurnState};
            let e = Engine::new(42007, 8);
            println!("size_of GameStateCore={} PlayerState={} TurnState={} GameState={} Engine={}",
                std::mem::size_of::<GameStateCore>(),
                std::mem::size_of::<PlayerState>(),
                std::mem::size_of::<TurnState>(),
                std::mem::size_of::<engine_native::state::GameState>(),
                std::mem::size_of::<Engine>());
            println!("rule_api::EffectRequest={} EventData={} LogEvent={}",
                std::mem::size_of::<rule_api_size::EffectRequest>(),
                std::mem::size_of::<engine_native::EventData>(),
                std::mem::size_of::<engine_native::LogEvent>());
            println!("skills_instances_8p_start={}", e.state_core().skills.len());
        }
        _ => eprintln!("usage: archbench scale [seats] | growth [seats] [budget] [games]"),
    }
}
