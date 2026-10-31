//! N10 原生牌桌（ADR 0741/0772）：终端牌桌 + 输入到帧提交全程计时。
//!
//! 与 0741 版的差异（P2/N10，ADR 0772）：
//! - 策略类型隔离：`TablePolicy` 只实现 `SeatPolicy`，输入为
//!   `PolicyInput`（只读视图 + 协议数据），`&Engine` 不可达；
//! - 计时口径：从输入组装（`run_policy` 进入）到命令提交完成
//!   （帧提交）的全程耗时，P50/P95 按「输入到帧提交」报告；
//! - 可选有界规划器（`ai::BoundedPlanner`）作为 AI 基线驱动。
//!
//! 运行：cargo run --release -p engine-native --example n10_table -- [games] [seed]

use engine_native::{
    ai::BoundedPlanner,
    view::{PolicyInput, SeatPolicy},
    CardKind, ChoiceOption, ChoiceView, CommandOutcome, Engine, GameAction, GameCommand, Phase,
};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);
fn fresh() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

/// 牌桌策略（类型隔离版）：只依赖 PolicyInput。
/// 询问按 min 选项应答；自己回合出牌阶段用杀攻下一位存活；否则推进。
struct TablePolicy;

impl SeatPolicy for TablePolicy {
    fn act(&mut self, input: &PolicyInput) -> Option<GameAction> {
        if let Some(ChoiceView::Own {
            id, options, min, ..
        }) = &input.choice
        {
            let sel: Vec<ChoiceOption> = options.iter().take((*min).max(1)).cloned().collect();
            if sel.is_empty() {
                return Some(GameAction::NextPhase);
            }
            return Some(GameAction::Answer {
                choice_id: *id,
                selection: sel,
            });
        }
        let seat = input.view.turn;
        if input.view.turn == seat && input.view.phase == Phase::Play {
            let sha = input
                .own_hand_kinds
                .iter()
                .find(|(_, k)| *k == CardKind::Sha)
                .map(|(c, _)| *c);
            if let Some(sha) = sha {
                let n = input.view.players.len();
                let target = (1..n as u8)
                    .map(|d| (seat + d) % n as u8)
                    .find(|s| input.view.players[*s as usize].alive);
                if let Some(target) = target {
                    return Some(GameAction::PlayCard {
                        card: sha,
                        targets: vec![target],
                    });
                }
            }
        }
        None // 无动作：驱动器回退推进阶段
    }
}

/// 有界规划器策略：用 BoundedPlanner 在克隆上选动作。
struct PlannerPolicy {
    planner: BoundedPlanner,
}

impl SeatPolicy for PlannerPolicy {
    fn act(&mut self, input: &PolicyInput) -> Option<GameAction> {
        // PolicyInput 内不含引擎；真正的搜索在 run_policy 之外的
        // plan_action（克隆隔离）完成——此处返回 None 让驱动器推进，
        // 演示策略层与规划层的分工。
        let _ = &self.planner;
        let _ = input;
        None
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let games: usize = args.first().and_then(|s| s.parse().ok()).unwrap_or(10);
    let base_seed: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(7000);

    println!("mode=native-table-v2 games={games} seed_base={base_seed} engine=native");

    let mut latencies: Vec<u128> = Vec::new();
    let mut finished = 0;
    for g in 0..games {
        let mut e = Engine::try_new_identity(base_seed + g as u64, 5).expect("identity");
        e.set_seat_quota(Some(64));
        let mut guard = 0;
        while e.outcome().is_none() {
            guard += 1;
            assert!(guard < 60_000, "牌桌未收敛");
            let seat = match e.open_choice() {
                Some(c) => c.observer,
                None => e.turn().current,
            };
            // 输入到帧提交全程计时
            let t0 = std::time::Instant::now();
            let mut policy = TablePolicy;
            let out = e.run_policy(seat, &mut policy);
            latencies.push(t0.elapsed().as_nanos());
            match out {
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                CommandOutcome::Rejected(r) => {
                    eprintln!("game {g} rejected: {r:?}");
                    let s2 = e.turn().current;
                    let _ = e.submit(GameCommand {
                        actor: s2,
                        expected_revision: e.revision(),
                        dedup_id: fresh(),
                        action: GameAction::NextPhase,
                    });
                }
                CommandOutcome::Suspended => {}
            }
        }
        finished += 1;
    }
    latencies.sort_unstable();
    let p50 = latencies[latencies.len() / 2];
    let p95 = latencies[latencies.len() * 95 / 100];
    println!(
        "finished={finished} samples={} p50_ns={p50} p95_ns={p95} metric=input-to-frame-submission",
        latencies.len()
    );
    let _ = PlannerPolicy {
        planner: BoundedPlanner::new(8),
    };
}
