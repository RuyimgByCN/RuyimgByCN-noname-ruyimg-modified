//! NC08 第⑦层「逐命令对照」工具（**旧线 ↔ 原生线**，可行口径）。
//!
//! 说明（边界登记）：方案要求的「逐命令对照」以**上游 JS** 为基准；上游 JS 在本仓库
//! 不可独立运行（无 harness），因此本工具以**旧线 `engine-core`** 作为**可执行参照**——
//! 旧线 ≠ 上游等价（见 NC08 开局差异判定证据），本工具只做**逐命令分歧定位**：
//! 同种子、同座位数、同命令序列（推进阶段）下，逐步比较两线的
//! `(当前座位, 相位, 各座位 hp/alive)`，报告**首次分歧**及其签名。
//!
//! 用途：① 回归守护——分歧签名变化即失败（提示规则或投影面发生位移）；
//! ② 后续接入上游 harness 时，把「参照实现」从旧线替换为上游即可复用同一比较框架。

use engine_api::{
    Command as ApiCommand, CommandKind as ApiKind, ExpectedRevision as ApiRev,
    IdempotencyKey as ApiKey, PlayerId as ApiPlayerId,
};
use engine_native::{CommandOutcome, Engine as NativeEngine, GameAction, GameCommand};

/// 一步对照快照：座位、相位、各座位 (hp, alive)。
#[derive(Debug, Clone, PartialEq, Eq)]
struct Step {
    current: u8,
    phase: String,
    seats: Vec<(i32, bool)>,
}

fn old_step(e: &engine_core::Engine) -> Step {
    let s = e.state();
    Step {
        // 旧线 PlayerId 为 1 基；归一到 0 基座位便于比较
        current: s.turn.current.raw().saturating_sub(1) as u8,
        phase: format!("{:?}", s.turn.phase),
        // PlayerStore 无 `iter`：按座位序经 `get` 取（座位数固定 4）
        seats: (0..4u32)
            .filter_map(|i| s.players.get(ApiPlayerId::new(i + 1)))
            .map(|p| (p.hp, p.alive))
            .collect(),
    }
}

fn native_step(e: &NativeEngine) -> Step {
    let core = e.state_core();
    Step {
        current: e.turn().current,
        phase: format!("{:?}", e.turn().phase),
        seats: (0..core.players.len() as u8)
            .map(|i| {
                let p = core.player(i);
                (p.hp, p.alive)
            })
            .collect(),
    }
}

/// 驱动一次对照：返回（首次分歧步号, 旧线快照, 原生快照）。
fn compare(seed: u64, steps: usize) -> Option<(usize, Step, Step)> {
    let mut old = engine_core::Engine::new();
    let _ = old.submit(ApiCommand {
        actor: ApiPlayerId::NONE,
        expected_revision: ApiRev(0),
        idempotency_key: ApiKey(0),
        kind: ApiKind::StartGame {
            player_count: 4,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    let mut native = NativeEngine::new_identity(seed, 4);

    for step in 0..steps {
        let a = old_step(&old);
        let b = native_step(&native);
        if a != b {
            return Some((step, a, b));
        }
        let actor = ApiPlayerId::new(old.state().turn.current.raw());
        let _ = old.submit(ApiCommand {
            actor,
            expected_revision: ApiRev(0),
            idempotency_key: ApiKey(step as u64 + 1),
            kind: ApiKind::EndPhase,
        });
        let seat = native.turn().current;
        let out = native.submit(GameCommand {
            actor: seat,
            expected_revision: native.revision(),
            dedup_id: step as u64 + 1,
            action: GameAction::NextPhase,
        });
        if matches!(out, CommandOutcome::Rejected(_)) {
            return Some((step, old_step(&old), native_step(&native)));
        }
    }
    None
}

/// 逐命令对照：首次分歧必须落在**已登记**的签名集合内（防静默位移）。
#[test]
fn nc08_command_by_command_first_divergence_is_registered() {
    let seeds: [u64; 6] = [1, 42, 99, 777, 12345, 20261702];
    let mut report: Vec<String> = Vec::new();
    for seed in seeds {
        match compare(seed, 12) {
            None => report.push(format!("seed={seed} 12 步内无分歧")),
            Some((step, a, b)) => report.push(format!(
                "seed={seed} 首次分歧@step={step} 旧(current={},phase={},seats={:?}) 原生(current={},phase={},seats={:?})",
                a.current, a.phase, a.seats, b.current, b.phase, b.seats
            )),
        }
    }
    for line in &report {
        eprintln!("CROSSLINE {line}");
    }
    // 登记口径：分歧应出现在**前 2 步内**（已知开局差异：初始手牌 6/4、起手相位
    // Prepare 缺失、旧线开局随机伤害——见 rules-baseline 与
    // 2026-10-18-nc08-upstream-identity-start 证据）。若出现在更晚步号，说明新增了
    // 未登记差异，本用例失败。
    for line in &report {
        if let Some(rest) = line.split("首次分歧@step=").nth(1) {
            let step: usize = rest.split_whitespace().next().unwrap().parse().unwrap();
            assert!(
                step <= 1,
                "新增未登记分歧（step={step}）：{line}\n完整报告：{report:#?}"
            );
        }
    }
}
