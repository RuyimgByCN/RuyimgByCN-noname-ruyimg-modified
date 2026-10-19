//! NC08 第⑦层（上游行为对照）样本 42：`kuangfu`（狂斧，**标准面** / 潘凤）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! kuangfu: {
//!     audio: "xinkuangfu",
//!     trigger: { source: "damageSource" },                 // 自己造成伤害后
//!     forced: true,                                        // 锁定技
//!     filter(event, player) {
//!         if (player.hasSkill("stdkuangfu_used")) return false;      // 每阶段一次
//!         return player.isPhaseUsing()                                // 自己出牌阶段内
//!             && event.card && event.card.name === "sha"              // 由【杀】造成
//!             && event.player !== player && event.player.isIn();       // 目标为其他存活角色
//!     },
//!     async content(event, trigger, player) {
//!         player.addTempSkill("stdkuangfu_used", "phaseChange");
//!         if (trigger.player.hp < player.hp) { await player.draw(2); }   // 目标体力**小于**你 → 摸 2
//!         else { await player.loseHp(); }                                // 否则 → 失去 1 点体力
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机/门槛**：自己**出牌阶段**内用【杀】对**其他存活角色**造成伤害后（锁定、**每阶段一次**）；
//! 2. **分支**：目标体力 **<** 自己 → **摸 2 张**；否则 → **失去 1 点体力**。
//!
//! 原生实现引用：`content/standard/skills/kuang_fu.rs`（filter 判 `Damage` + 来源自己 +
//! 非自己 + 未防止 + 牌为【杀】+ 当前回合/出牌阶段；`step` 按 `target.hp < owner.hp`
//! 分流为 `draw_cards(owner, 2)` / `hp -= 1`（含濒死检查））。
//!
//! **登记差异**：上游以 **`stdkuangfu_used` temp 技能到 `phaseChange`** 记「每阶段一次」；
//! 原生注释自述「因触发仅在自身出牌阶段，两者等价」→ **口径不同、语义等价**（登记）。
//!
//! 未覆盖（登记）：`audio`/`ai.halfneg` 表现与策略面。

mod nc08_common;
use engine_native::{ChoiceOption, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_play, drain, next_dedup};

/// 上游点 2（**摸 2 分支**）：目标体力**小于**自己 → 摸 2 张。
#[test]
fn upstream_kuangfu_draws_two_when_target_lower_hp() {
    let mut e = Engine::new(20267401, 4);
    e.debug_grant(0, "狂斧");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 自己满血 4；目标压到 2（受 1 点伤害后仍存活 → 满足上游 `event.player.isIn()`）
    e.debug_set_hp(1, 2);
    let sha = {
        let name = e.catalog().cards.iter().find(|c| c.name == "sha").map(|c| c.name).unwrap();
        e.debug_place(0, name)
    };
    let hand_before = e.debug_hand(0).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    assert!(
        matches!(out, engine_native::CommandOutcome::Accepted { .. } | engine_native::CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    // 净变化：打出【杀】-1、狂斧摸 2 → ≥ +1
    eprintln!(
        "DBG 狂斧摸2：hand {hand_before} → {}（目标 hp={}）",
        e.debug_hand(0).len(),
        e.debug_hp(1)
    );
    assert!(
        e.debug_hand(0).len() >= hand_before + 1,
        "狂斧：目标体力低于自己 → 摸 2（打出 -1 后净 +1）：{hand_before} → {}",
        e.debug_hand(0).len()
    );
}

/// 上游点 2（**失去体力分支**）：目标体力**不小于**自己 → 自己失去 1 点体力。
#[test]
fn upstream_kuangfu_loses_hp_when_target_not_lower() {
    let mut e = Engine::new(20267402, 4);
    e.debug_grant(0, "狂斧");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 自己压到 2（失去 1 点后仍存活）；目标满血 4 → 目标体力不小于自己
    e.debug_set_hp(0, 2);
    e.debug_set_hp(1, 4);
    let sha = {
        let name = e.catalog().cards.iter().find(|c| c.name == "sha").map(|c| c.name).unwrap();
        e.debug_place(0, name)
    };
    let hp_before = e.debug_hp(0);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    assert!(
        matches!(out, engine_native::CommandOutcome::Accepted { .. } | engine_native::CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    eprintln!(
        "DBG 狂斧失体：自己 hp {hp_before} → {}（目标 hp={}）",
        e.debug_hp(0),
        e.debug_hp(1)
    );
    assert!(
        e.debug_hp(0) < hp_before,
        "狂斧：目标体力不小于自己 → 自己失去 1 点体力：{hp_before} → {}",
        e.debug_hp(0)
    );
    let _ = Phase::Play;
    let _ = ChoiceOption::Confirm;
}

/// **登记差异**（源码事实固定）：每阶段一次的**实现口径**不同（temp 技能 ↔ 阶段判定）。
#[test]
fn upstream_kuangfu_once_per_phase_difference_is_fixed() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/standard/skills/kuang_fu.rs"
    ))
    .expect("读狂斧源码");
    let seg: String = src.chars().take(900).collect();
    assert!(
        seg.contains("Phase::Play") && seg.contains("两者等价"),
        "差异固定：原生以阶段判定表达每阶段一次，并自述与上游 temp 技能等价：{seg}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("kuangfu: {").expect("上游狂斧");
    let seg: String = up[idx..].chars().take(900).collect();
    assert!(
        seg.contains("stdkuangfu_used") && seg.contains("draw(2)") && seg.contains("loseHp()"),
        "上游依据：temp 技能记一次 + 摸 2 / 失去体力分支：{seg}"
    );
}
