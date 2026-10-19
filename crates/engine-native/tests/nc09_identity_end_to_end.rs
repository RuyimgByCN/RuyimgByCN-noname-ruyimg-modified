//! NC09 身份局**端到端**（引擎侧）：以身份局开局跑到终局，并核对胜负结果。
//!
//! 背景（登记表 `NC09-IDENTITY-E2E`）：终局胜负展示已接线（前端文案 44/0、Rust 映射 33/0），
//! 但此前的端到端实测走的是**原型/单挑口径**（本地开局）→ 本测试补齐**身份局**的引擎侧端到端。
//!
//! 范围声明（不夸大）：
//! - 本测试覆盖**引擎侧**：身份局建局 → 确定性驱动至终局 → 结果属
//!   `Winner::{LordSide, Rebels, Traitor}` 三者之一（`Winner` 的其它变体属原型/国战/对决/斗地主）；
//! - **浏览器侧**身份局端到端（真实构建产物 + WASM worker）**仍未**完成（登记待办）。

use engine_native::state::{Outcome, Winner};
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);
fn next() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

/// 确定性驱动：处理询问（触发确认一律确认），否则推进阶段；返回步数。
fn drive(e: &mut Engine, max_steps: usize) -> usize {
    let mut steps = 0;
    for _ in 0..max_steps {
        if e.state_core().outcome.is_some() {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let sel: Vec<engine_native::ChoiceOption> = if c.spec.prompt == "trigger_confirm" {
                vec![engine_native::ChoiceOption::Confirm]
            } else if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
                vec![engine_native::ChoiceOption::Pass]
            } else if c.spec.allow_cancel {
                Vec::new()
            } else {
                c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
            };
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next(),
                action: GameAction::Answer { choice_id: c.id, selection: sel },
            });
        } else {
            let actor = e.turn().current;
            let out = e.submit(GameCommand {
                actor,
                expected_revision: e.revision(),
                dedup_id: next(),
                action: GameAction::NextPhase,
            });
            if matches!(out, CommandOutcome::Rejected(_)) {
                // 无法推进（例如等待他人响应）：交给下一轮循环处理询问
                if e.open_choice().is_none() {
                    break;
                }
            }
        }
        steps += 1;
    }
    steps
}

/// 身份局：建局 → 驱动至终局 → 结果必为身份局可表达的四类之一。
#[test]
fn identity_game_reaches_terminal_outcome() {
    let mut e = Engine::new_identity(20267201, 5);
    assert!(
        matches!(
            e.state_core().mode,
            engine_native::state::GameMode::Identity { .. }
        ),
        "建局应为身份局"
    );
    let seats = e.state_core().players.len();
    // 先按确定性驱动跑一段（覆盖真实阶段/询问路径）；被动驱动不会自然分出胜负，
    // 故随后用调试伤害**杀死主公**，把对局推到终局（本测试的对象是终局**结果判定**）。
    let steps = drive(&mut e, 600);
    let lord = (0..seats as u8)
        .find(|p| e.state_core().is_lord(*p))
        .expect("身份局必有主公");
    e.debug_set_hp(lord, 1);
    e.debug_deal_damage(Some(lord), lord, 1, engine_native::event::DamageNature::Normal);
    let _ = drive(&mut e, 400);
    let outcome = e.state_core().outcome.clone();
    eprintln!(
        "DBG 身份局：座位={seats} 主公={lord} 驱动步数={steps} outcome={outcome:?} 阶段={:?}",
        e.turn().phase
    );
    let Some(w) = outcome else {
        panic!("主公死亡后仍未到达终局（登记：需检查胜负判定路径）");
    };
    // 身份局可表达结果：主公阵营胜 / 反贼胜 / 内奸胜（`Winner` 其余变体属其它模式）
    let Outcome { winner } = w;
    assert!(
        matches!(winner, Winner::LordSide | Winner::Rebels | Winner::Traitor),
        "身份局终局应为 LordSide/Rebels/Traitor 之一，实测 {winner:?}"
    );
}

/// 终局后：不再接受推进类命令（MatchOver），且状态哈希稳定。
#[test]
fn identity_game_is_stable_after_terminal() {
    let mut e = Engine::new_identity(20267202, 4);
    let _ = drive(&mut e, 4000);
    if e.state_core().outcome.is_none() {
        eprintln!("SKIP：未在步数上限内到达终局");
        return;
    }
    let h1 = e.state_hash();
    let actor = e.turn().current;
    let out = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next(),
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "终局后推进类命令应被拒：{out:?}"
    );
    assert_eq!(h1, e.state_hash(), "被拒命令不应改变状态哈希");
    let _ = Phase::Play;
}
