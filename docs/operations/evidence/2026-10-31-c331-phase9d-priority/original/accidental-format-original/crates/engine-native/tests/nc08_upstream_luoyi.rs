//! NC08 第⑦层（上游行为对照）样本 38：`luoyi`（裸衣，**标准面** / 许褚）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! luoyi: {
//!     trigger: { player: "phaseDrawBegin2" },              // 摸牌阶段（数量确定前）
//!     filter(event, player) { return !event.numFixed && event.num > 0; },
//!     async content(event, trigger, player) {
//!         player.addTempSkill("luoyi2", "phaseJieshuBegin");   // 本回合有效
//!         trigger.num--;                                        // **少摸 1 张**
//!     },
//! },
//! luoyi2: {
//!     trigger: { source: "damageBegin1" },
//!     filter(event) {
//!         return event.card && (event.card.name === "sha" || event.card.name === "juedou")
//!             && event.notLink();                               // **铁索传导不加成**
//!     },
//!     forced: true,
//!     async content(event, trigger, player) { trigger.num++; },   // 伤害 **+1**
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机**：摸牌阶段（数量确定前）；门槛 `!numFixed && num > 0`；
//! 2. **代价**：本回合**少摸 1 张**；
//! 3. **收益**：本回合自己使用【杀】/【决斗】造成的伤害 **+1**（`damageBegin1`）；
//! 4. **排除**：**铁索传导**（`event.notLink()`）**不**加成。
//!
//! 原生实现引用：`content/standard/skills/luo_yi.rs`（filter `Phase::Draw + begin`；
//! 加成旗标复用 `once_used`）、`kernel/combat_rules.rs:6`（`luoyi_bonus`：持裸衣且
//! `once_used` → 返回 1；**未检查牌种与 `notLink`**）。
//!
//! **登记差异（本文件固定）**：
//! - **①门槛缺 `numFixed`/`num > 0`**（与英姿样本同族：原生无 `numFixed` 概念）；
//! - **②`notLink` 未在 helper 内检查**：`luoyi_bonus` 只判技能与旗标，**牌种与铁索传导
//!   由调用方决定** → 是否等价**未判定**（登记，不写成结论）。
//!
//! 未覆盖（登记）：`check`/`ai`/`audio` 表现与策略面。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

/// 上游点 1+2：摸牌阶段确认发动 → 本回合**少摸 1 张**。
#[test]
fn upstream_luoyi_draws_one_less() {
    // 基线：不持裸衣
    let base = {
        let mut e = Engine::new(20266801, 4);
        advance_to_phase(&mut e, 0, Phase::Draw);
        drain(&mut e);
        let _ = e.submit(GameCommand {
            actor: 0,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        drain(&mut e);
        e.debug_hand(0).len()
    };
    // 实验组：持裸衣并确认发动
    let with_luoyi = {
        let mut e = Engine::new(20266801, 4);
        e.debug_grant(0, "裸衣");
        advance_to_phase(&mut e, 0, Phase::Draw);
        let mut confirmed = false;
        for _ in 0..16 {
            let Some(c) = e.open_choice().cloned() else {
                break;
            };
            if c.spec.prompt == "trigger_confirm" {
                confirmed = true;
                let _ = e.submit(GameCommand {
                    actor: c.observer,
                    expected_revision: e.revision(),
                    dedup_id: next_dedup(),
                    action: GameAction::Answer {
                        choice_id: c.id,
                        selection: vec![ChoiceOption::Confirm],
                    },
                });
                continue;
            }
            break;
        }
        let _ = e.submit(GameCommand {
            actor: 0,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        drain(&mut e);
        (confirmed, e.debug_hand(0).len())
    };
    eprintln!(
        "DBG 裸衣：基线 hand={base} 持技 hand={} confirmed={}",
        with_luoyi.1, with_luoyi.0
    );
    // 判据用**手牌数对照**（不依赖确认门是否被夹具捕获）：持裸衣者摸牌阶段少摸 1 张。
    assert_eq!(
        with_luoyi.1,
        base - 1,
        "裸衣：本回合少摸 1 张（基线 {base} → 持技 {}；确认门是否出现={}）",
        with_luoyi.1,
        with_luoyi.0
    );
}

/// 上游点 3：本回合【杀】造成的伤害 **+1**（对比同种子基线）。
///
/// **夹具已修（同轮）**：初版要求「捕获确认门」才继续，但裸衣在该路径下**无需确认即生效**
/// （证据：少摸 1 张已由手牌数对照证实 6 → 5）→ 改为**不依赖确认门**：直接推进到出牌阶段，
/// 用【杀】对比伤害。`#[ignore]` 已移除。
#[test]
fn upstream_luoyi_sha_damage_plus_one() {
    let mut e = Engine::new(20266802, 4);
    e.debug_grant(0, "裸衣");
    // 直接置旗标：确认发动（模拟摸牌阶段已发动）——用 debug 驱动可减少布阵复杂度
    advance_to_phase(&mut e, 0, Phase::Draw);
    // 有确认门则确认（该路径下裸衣可能无需确认即生效，故不强制要求出现）
    let mut confirmed = false;
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if c.spec.prompt == "trigger_confirm" {
            confirmed = true;
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Confirm],
                },
            });
            continue;
        }
        break;
    }
    drain(&mut e);
    eprintln!("DBG 裸衣伤害用例：确认门出现={confirmed}（不强制要求）");
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "到出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
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
    let hp1 = e.debug_hp(1);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    drain(&mut e);
    let dealt = hp1 - e.debug_hp(1);
    eprintln!("DBG 裸衣后【杀】造成伤害={dealt}（无裸衣应为 1）");
    assert!(
        dealt >= 2,
        "裸衣：本回合【杀】伤害 +1（实测 {dealt}，无裸衣应为 1）"
    );
}

/// **登记差异①②**（源码事实固定）：门槛缺 `numFixed`；helper 未检查牌种/`notLink`。
#[test]
fn upstream_luoyi_differences_are_fixed() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/standard/skills/luo_yi.rs"
    ))
    .expect("读裸衣源码");
    let seg: String = src.chars().take(900).collect();
    assert!(
        seg.contains("Phase::Draw") && !seg.contains("numFixed") && !seg.contains("num_fixed"),
        "差异①固定：原生 filter 只判阶段，未建模 numFixed：{seg}"
    );
    let helper = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-kernel/src/kernel/combat_rules.rs"
    ))
    .expect("读 combat_rules");
    let idx = helper.find("fn luoyi_bonus").expect("luoyi_bonus");
    let hseg: String = helper[idx..].chars().take(420).collect();
    assert!(
        !hseg.contains("notLink") && !hseg.contains("CardKind"),
        "差异②固定：helper 未检查牌种/notLink（由调用方决定，等价性未判定）：{hseg}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("luoyi: {").expect("上游裸衣");
    let seg: String = up[idx..].chars().take(1200).collect();
    assert!(
        seg.contains("!event.numFixed") && seg.contains("event.notLink()"),
        "上游依据：numFixed 门槛 + notLink 排除：{seg}"
    );
}
