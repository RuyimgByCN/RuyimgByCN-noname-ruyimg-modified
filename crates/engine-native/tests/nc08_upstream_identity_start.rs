//! NC08 第⑦层：**身份局开局**的上游判定（对照 `nc08_cross_line` 登记的 4 处差异）。
//!
//! 上游依据（归档快照，逐字引用）：
//! `archive/upstream/noname/apps/core/mode/identity.js` 的 `start` 尾部：
//!
//! ```js
//! event.beginner = _status.firstAct2 || game.zhong || game.zhu || _status.firstAct || game.me;
//! game.gameDraw(event.beginner, player => {
//!     if (_status.mode == "purple" && player.seatNum > 5) { return 5; }
//!     return 4;                     // ← 每人 4 张（含主公）
//! });
//! ...
//! game.phaseLoop(event.beginner);
//! ```
//!
//! 另据归档上游文档 `docs/game-startup-flow.md`：阶段顺序为
//! `phaseZhunbei / phaseJudge / phaseDraw / phaseUse / phaseDiscard / phaseJieshu`
//! → 一个**新回合从「准备阶段」开始**。
//!
//! 由上游推导的判定：
//! 1. **首回合玩家 = 主公**（`game.zhu` 优先）→ 原生 `new_identity` 主公为座位 0 且由
//!    座位 0 起手：**与上游一致**；
//! 2. **初始手牌每人 4 张（含主公）** → 原生 `[4,4,4,4]`：**与上游一致**；旧线的
//!    `[6,4,4,4]`（主公多 2 张）**与上游不一致**；
//! 3. **阶段顺序含「准备阶段」** → 原生只有 `Judge/Draw/Play/Discard/End`，**缺 Prepare**：
//!    **登记差异**（原生把准备阶段类触发并入判定/回合起点，需按技能逐条核对）。
//!
//! 未覆盖：`_status.mode == "purple"` 等模式分支、`replaceHandcards`（手气卡）属模式/表现面。

use engine_native::{Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

/// 上游判定 1+2：主公起手、每人 4 张（含主公）。
#[test]
fn upstream_identity_start_lord_first_and_four_cards_each() {
    let e = Engine::new_identity(20262401, 4);
    // 主公（座位 0）：体力上限 5（身份局主公 +1）
    assert_eq!(
        e.state_core().player(0).max_hp,
        5,
        "身份局主公 +1 体力上限"
    );
    for seat in 1..4u8 {
        assert_eq!(e.state_core().player(seat).max_hp, 4, "非主公上限 4");
    }
    // 每人 4 张（上游 `gameDraw` 返回 4，含主公）
    for seat in 0..4u8 {
        assert_eq!(
            e.debug_hand(seat).len(),
            4,
            "上游 gameDraw 每人 4 张（座位 {seat}）"
        );
    }
    // 首回合玩家 = 主公（座位 0）
    assert_eq!(e.turn().current, 0, "首回合由主公起手（上游 game.zhu 优先）");
    assert_eq!(e.turn().round, 1, "第 1 回合");
}

/// R05（A2，规则 333）：登记差异反转——准备阶段已作为独立相位落地
///（Phase::Prepare；回合序 准备→判定→…）。上游判定 3 的「不含准备阶段」
/// 差异不再成立；本锁面改为固定**含** Prepare 且身份局起手仍在判定前
/// 经历准备阶段序列（首个提交前帧待处理，见 K22b 搁置说明）。
#[test]
fn upstream_identity_start_prepare_phase_now_present() {
    let phases: Vec<String> = Phase::ALL.iter().map(|p| format!("{p:?}")).collect();
    assert!(
        phases.iter().any(|p| p == "Prepare"),
        "原生阶段集合含 Prepare（R05 反转登记差异）：{phases:?}"
    );
    let e = Engine::new_identity(20262402, 4);
    assert_eq!(e.turn().phase, Phase::Judge, "身份局起手阶段 Judge（准备阶段帧序列待 K22b）");
}

/// 上游判定 2 的反向对照：旧线的「主公多 2 张」**不符**上游 `gameDraw` 返回值 4。
///
/// 本用例不依赖旧线实现，只把上游依据固定为可执行断言（4 张），供 `nc08_cross_line`
/// 的差异清单引用；旧线偏差本身在其对照日志中记录。
#[test]
fn upstream_identity_draw_is_uniform_four() {
    for seed in [20262403u64, 20262404, 20262405] {
        let mut e = Engine::new_identity(seed, 4);
        for seat in 0..4u8 {
            assert_eq!(
                e.debug_hand(seat).len(),
                4,
                "seed={seed}：上游 gameDraw 每人 4 张（座位 {seat}）"
            );
        }
        // 推进一个阶段后仍可继续（开局状态自洽）
        let actor = e.turn().current;
        let out = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        assert!(
            matches!(
                out,
                engine_native::CommandOutcome::Accepted { .. }
                    | engine_native::CommandOutcome::Pending { .. }
            ),
            "{out:?}"
        );
    }
}
