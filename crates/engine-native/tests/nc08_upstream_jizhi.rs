//! NC08 第⑦层（上游行为对照）样本 19：`jizhi`（集智，**标准面** / 黄月英）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/sb/skill.js`）：
//!
//! ```js
//! jizhi: {
//!     trigger: { player: "useCard" },
//!     filter(event, player) { return get.type(event.card) === "trick"; },  // **任意锦囊**
//!     forced: true,                                                        // 锁定技
//!     async content(event, trigger, player) {
//!         const next = player.draw("nodelay");
//!         next.gaintag.add("sbjizhi");          // 摸到的牌打上标记
//!         await next;
//!         player.addTempSkill("sbjizhi_mark");  // 本回合该牌**不可被弃置**
//!     },
//!     subSkill: { mark: { mod: { ignoredHandcard(…) / cardDiscardable(…) } } },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **锁定技**（`forced: true` → 无确认门）；
//! 2. **门槛 = 使用任意锦囊**（`get.type(event.card) === "trick"`）；
//! 3. **效果 = 摸 1 张**，且该牌获得 `sbjizhi` 标记（**本回合不可弃置**）。
//!
//! 原生实现引用：`content/standard/skills/ji_zhi.rs`（filter 仅列 **5 种**锦囊：
//! 南蛮/万箭/决斗/铁索/无懈；效果 `draw_cards(owner, 1)`）。
//!
//! **口径更正（2026-10-19 同轮）**：本样本初版把 `character/sb/skill.js` 当作基准，其中
//! 集智带 `sbjizhi` 标记与子技。**规则基准来源是 `character/standard/skill.js`**（本地 standard
//! 规则），其集智为 `filter: get.type(event.card) === "trick"` + `draw`，**没有**标记/子技 →
//! 因此**「标记面未实现」不是差异**（已从基准差异表移除）。
//!
//! **登记差异（本文件固定，属规则损失）**：
//! - **①覆盖范围**：标准面上游为**任意锦囊**，原生只覆盖 5 种 → 例如【无中生有】【桃园结义】
//!   **不触发**集智（本文件以行为断言固定该现状）。
//!
//! 未覆盖（登记）：`audio` 等表现面。

use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn advance_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..24 {
        if e.turn().current == seat && e.turn().phase == Phase::Play {
            return;
        }
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        drain(e);
    }
    panic!("未到 {seat} 号出牌阶段");
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
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
            dedup_id: next_dedup(),
            action: GameAction::Answer { choice_id: c.id, selection: sel },
        });
    }
}

fn place_by_prefix(e: &mut Engine, prefix: &str, player: u8) -> engine_native::CardId {
    let name = e
        .catalog()
        .cards
        .iter()
        .map(|c| c.name)
        .find(|n| n.contains(prefix))
        .unwrap_or_else(|| panic!("{prefix} 在目录"));
    e.debug_place(player, name)
}

/// 上游点 1+3（**在原生已覆盖的锦囊上**）：使用【决斗】→ 摸 1 张，且无确认门。
#[test]
fn upstream_jizhi_draws_on_covered_trick() {
    let mut e = Engine::new(20264701, 4);
    e.debug_grant(0, "集智"); // 目录名为中文技能名
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let duel = place_by_prefix(&mut e, "duel", 0); // 决斗（原生已覆盖的 5 种之一；目录名为英文）
    let hand_before = e.debug_hand(0).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: duel, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    assert!(
        e.open_choice().is_none(),
        "集智为锁定技，不应出现确认门：{:?}",
        e.open_choice().map(|c| c.spec.prompt.clone())
    );
    drain(&mut e);
    assert!(
        e.debug_hand(0).len() >= hand_before,
        "使用【决斗】应摸 1 张（本体离手 +1 摸）：before={hand_before} after={}",
        e.debug_hand(0).len()
    );
}

/// 上游点 2 的**登记差异①（规则损失）**：使用【无中生有】（也是锦囊）时，
/// 上游 `get.type === "trick"` 成立 → 应触发集智；**原生不触发**（filter 只列 5 种）。
#[test]
fn upstream_jizhi_scope_difference_is_registered() {
    // 源码事实：原生 filter 只列 5 种锦囊
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/standard/skills/ji_zhi.rs"
    ))
    .expect("读集智源码");
    let seg: String = src.chars().take(1400).collect();
    assert!(
        seg.contains("SavageAttack") && seg.contains("Nullify"),
        "源码事实：原生集智只覆盖 5 种锦囊：{seg}"
    );
    // 行为面：使用【无中生有】→ 原生**不**触发集智（手牌不因集智增加）
    let mut e = Engine::new(20264702, 4);
    e.debug_grant(0, "集智"); // 目录名为中文技能名
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let wuzhong = place_by_prefix(&mut e, "wuzhong", 0);
    let hand_before = e.debug_hand(0).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: wuzhong, targets: vec![] },
    });
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    drain(&mut e);
    // 【无中生有】自身摸 2：本体离手（-1）+ 摸 2 = 净 +1；若集智也触发则净 +2
    assert_eq!(
        e.debug_hand(0).len(),
        hand_before + 1,
        "登记差异①：原生集智**不**覆盖【无中生有】（上游会触发，净应为 +2）"
    );
}

/// **口径更正**：`sbjizhi` 标记面属 `sb` 面（非规则基准来源）→ 不作为差异登记；
/// 本用例保留为「标准面定义核对」：标准面集智**无**标记/子技。
#[test]
fn upstream_jizhi_standard_face_has_no_gaintag() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/standard/skills/ji_zhi.rs"
    ))
    .expect("读集智源码");
    assert!(
        !src.contains("sbjizhi") && !src.contains("gaintag"),
        "差异②固定：原生未实现 sbjizhi 标记面（上游摸到的牌本回合不可弃置）"
    );
    // 标准面（规则基准来源）：集智无 sbjizhi 标记/子技
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("jizhi: {").expect("上游集智");
    let seg: String = up[idx..].chars().take(700).collect();
    assert!(
        seg.contains("get.type(event.card) === \"trick\"") && !seg.contains("gaintag"),
        "标准面依据：任意锦囊、无标记面：{seg}"
    );
}
