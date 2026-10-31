//! NC08 第⑦层（上游行为对照）样本 49：`paoxiao`（咆哮，**标准面** / 张飞）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! paoxiao: {
//!     audio: 2,
//!     firstDo: true,
//!     trigger: { player: "useCard1" },          // 第 2 张起的**语音**触发
//!     forced: true,
//!     filter(event, player) {
//!         return !event.audioed && event.card.name === "sha"
//!             && player.countUsed("sha", true) > 1
//!             && event.getParent()?.type === "phase";
//!     },
//!     mod: {
//!         cardUsable(card, player, num) {
//!             if (card.name == "sha") return Infinity;   // **【杀】次数无上限**
//!         },
//!     },
//!     ai: { unequip: true, … },
//! },
//! ```
//!
//! 由 JS 推导的规则点：持技者**出牌阶段使用【杀】不受次数限制**（`mod.cardUsable` → `Infinity`）；
//! `trigger` 只是第 2 张起的**语音**表现面。
//!
//! 原生实现引用：`engine/play_sha.rs:32`（读 `PAO_XIAO` 持有者）+ `:118`（
//! `if !paoxiao && … && core.turn.sha_used_play_phase { … 限次 … }` → **持咆哮者绕过限次**）。
//!
//! **登记差异（本文件固定）**：**拦截/放行方式**——上游 `mod.cardUsable`（**统一把上限设为
//! `Infinity`**）；原生在**限次校验处 bypass**（`!paoxiao && …`）→ 语义等价、**维护性风险**
//! 同 `kongcheng`/`qianxun`（**族第 3 例**）。
//!
//! 未覆盖（登记）：`firstDo`/`trigger`（语音面）、`ai.unequip` 策略面。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_play, drain, next_dedup};

/// 上游点（**主判据**）：持咆哮者出牌阶段可**连续使用两张【杀】**。
#[test]
fn upstream_paoxiao_allows_multiple_sha() {
    let mut e = Engine::new(20268101, 4);
    e.debug_grant(0, "咆哮");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let mut played = 0;
    for _ in 0..2 {
        let sha = {
            let name = e
                .catalog()
                .cards
                .iter()
                .find(|c| c.name == "sha")
                .map(|c| c.name)
                .unwrap();
            e.debug_place(0, name)
        };
        let out = e.submit(GameCommand {
            actor: 0,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::PlayCard {
                card: sha,
                targets: vec![1],
            },
        });
        eprintln!("DBG 咆哮第 {} 张【杀】：{out:?}", played + 1);
        if matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ) {
            played += 1;
        } else {
            break;
        }
        drain(&mut e);
    }
    assert_eq!(
        played, 2,
        "咆哮：出牌阶段应可使用两张【杀】（实测 {played} 张）"
    );
}

/// 上游点（**反例**）：**无咆哮**者同回合第二张【杀】被拒。
#[test]
fn upstream_paoxiao_requires_the_skill() {
    let mut e = Engine::new(20268102, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let first = {
        let name = e
            .catalog()
            .cards
            .iter()
            .find(|c| c.name == "sha")
            .map(|c| c.name)
            .unwrap();
        e.debug_place(0, name)
    };
    let out1 = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: first,
            targets: vec![1],
        },
    });
    assert!(
        matches!(
            out1,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "第一张【杀】应可用：{out1:?}"
    );
    drain(&mut e);
    let second = {
        let name = e
            .catalog()
            .cards
            .iter()
            .find(|c| c.name == "sha")
            .map(|c| c.name)
            .unwrap();
        e.debug_place(0, name)
    };
    let out2 = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: second,
            targets: vec![1],
        },
    });
    eprintln!("DBG 无咆哮第二张【杀】：{out2:?}");
    assert!(
        matches!(out2, CommandOutcome::Rejected(_)),
        "无咆哮者同回合第二张【杀】应被拒，实测 {out2:?}"
    );
    let _ = Phase::Play;
}

/// **登记差异**（源码事实固定）：`mod.cardUsable → Infinity` ↔ 限次校验处 bypass（族第 3 例）。
#[test]
fn upstream_paoxiao_bypass_difference_is_fixed() {
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("\tpaoxiao: {").expect("上游咆哮");
    let seg: String = up[idx..].chars().take(1000).collect();
    assert!(
        seg.contains("cardUsable") && seg.contains("Infinity"),
        "上游依据：cardUsable 对【杀】返回 Infinity：{seg}"
    );
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/engine/play_sha.rs"
    ))
    .expect("读 play_sha");
    let idx = src.find("if !paoxiao").expect("限次 bypass");
    let seg: String = src[idx..].chars().take(240).collect();
    assert!(
        seg.contains("sha_used_play_phase"),
        "原生依据：在限次校验处 bypass（`!paoxiao && … && sha_used_play_phase`）：{seg}"
    );
}
