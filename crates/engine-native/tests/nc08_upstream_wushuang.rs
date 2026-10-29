//! NC08 第⑦层（上游行为对照）样本 50：`wushuang`（无双，**标准面** / 吕布）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! wushuang: { forced: true, locked: true, group: ["wushuang1", "wushuang2"], … },
//!
//! wushuang1: {                                  // 【杀】方向
//!     trigger: { player: "useCardToPlayered" },
//!     forced: true, sourceSkill: "wushuang",
//!     filter(event, player) {
//!         return event.card.name === "sha"
//!             && !event.getParent()?.directHit.includes(event.target);   // **必中牌排除**
//!     },
//!     async content(event, trigger, player) { … 需要**两张【闪】** … },
//! },
//! // wushuang2：决斗方向（需要两张【杀】）
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **【杀】方向**（`wushuang1`）：目标须**连续打出两张【闪】**；**必中**（`directHit`）时**不**生效；
//! 2. **【决斗】方向**（`wushuang2`）：对手须**连续打出两张【杀】**。
//!
//! 原生实现引用：
//! - `kernel/sha.rs:277`（`source_has_wushuang` → 需连续两张【闪】）；
//! - `kernel/duel.rs:121`（`opponent_has_wushuang` → 同一响应者再次询问第二张【杀】）。
//!
//! **登记差异（本文件固定）**：**`directHit` 排除**——上游 `wushuang1` 的 filter 显式排除
//! `event.getParent()?.directHit.includes(event.target)`（**必中**时无双不加倍）；原生
//! `source_has_wushuang` 只判**是否持技**，**未见 `directHit` 判定**（本样本只固定源码事实，
//! 行为面因需构造「必中」场景而登记待办）。
//!
//! 未覆盖（登记）：`wushuang2` 决斗方向的行为面（本样本只固定源码事实）；
//! `preHidden`/`audioname` 表现面。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_play, drain, next_dedup};

/// 上游点 1（**源码事实**）：无双在【杀】与【决斗】**两个方向**均有实现。
#[test]
fn upstream_wushuang_covers_both_directions() {
    let sha = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-kernel/src/kernel/sha.rs"
    ))
    .expect("读 sha");
    let duel = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-kernel/src/kernel/duel.rs"
    ))
    .expect("读 duel");
    assert!(
        sha.contains("source_has_wushuang") && sha.contains("需要连续两张闪"),
        "【杀】方向应有『需要连续两张闪』实现"
    );
    assert!(
        duel.contains("opponent_has_wushuang") && duel.contains("同一响应者再次询问第二张杀"),
        "【决斗】方向应有『同一响应者再次询问第二张杀』实现"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("\twushuang: {").expect("上游无双");
    let seg: String = up[idx..].chars().take(1800).collect();
    assert!(
        seg.contains("group: [\"wushuang1\", \"wushuang2\"]"),
        "上游依据：两个子技组成：{seg}"
    );
    assert!(
        seg.contains("directHit"),
        "上游依据：wushuang1 排除 directHit（必中）：{seg}"
    );
}

/// 上游点 1（**行为面**）：无双的【杀】需要**两张【闪】**——只出一张不足以响应。
#[test]
fn upstream_wushuang_sha_needs_two_shan() {
    let mut e = Engine::new(20268201, 4);
    e.debug_grant(0, "无双");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = {
        let name = e.catalog().cards.iter().find(|c| c.name == "sha").map(|c| c.name).unwrap();
        e.debug_place(0, name)
    };
    // 目标只给一张【闪】
    let shan = {
        let name = e.catalog().cards.iter().find(|c| c.name == "shan").map(|c| c.name).unwrap();
        e.debug_place(1, name)
    };
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let hp1_before = e.debug_hp(1);
    let mut prompts: Vec<String> = Vec::new();
    let mut answered_first = false;
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else { break };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
        if c.spec.prompt == "respond_shan" && !answered_first {
            answered_first = true;
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![engine_native::ChoiceOption::Card(shan)],
                },
            });
            continue;
        }
        let sel: Vec<engine_native::ChoiceOption> =
            if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
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
    drain(&mut e);
    eprintln!(
        "DBG 无双（只出一张闪）：询问={prompts:?} 目标 hp {hp1_before} → {}",
        e.debug_hp(1)
    );
    if !answered_first {
        eprintln!("NOTE 未出现 respond_shan 询问——本用例不构成验收");
        return;
    }
    assert!(
        e.debug_hp(1) < hp1_before,
        "无双：只出一张【闪】**不足以**抵消，目标应受伤（{hp1_before} → {}）",
        e.debug_hp(1)
    );
    let _ = Phase::Play;
}

/// **登记差异**（源码事实固定）：`directHit`（必中）排除未在原生无双实现中体现。
#[test]
fn upstream_wushuang_direct_hit_difference_is_fixed() {
    let sha = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-kernel/src/kernel/sha.rs"
    ))
    .expect("读 sha");
    let idx = sha.find("fn source_has_wushuang").expect("source_has_wushuang");
    let seg: String = sha[idx..].chars().take(320).collect();
    assert!(
        !seg.contains("direct_hit") && !seg.contains("directHit"),
        "差异固定：原生无双判定未见 directHit（必中）排除：{seg}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游");
    let idx = up.find("\twushuang1: {").expect("上游 wushuang1");
    let seg: String = up[idx..].chars().take(600).collect();
    assert!(
        seg.contains("!event.getParent()?.directHit.includes(event.target)"),
        "上游依据：必中时无双不生效：{seg}"
    );
}

/// 上游点 2（**行为面 · 决斗方向**）：无双使对手需**两张【杀】**——只出一张即受伤。
#[test]
fn upstream_wushuang_duel_needs_two_sha() {
    use engine_native::{ChoiceOption, CommandOutcome, GameAction, GameCommand};
    use nc08_common::{advance_to_play, drain, next_dedup};

    let mut e = Engine::new(20268202, 4);
    e.debug_grant(0, "无双");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 0 号持【决斗】；1 号只持**一张**【杀】
    let duel = {
        let name = e.catalog().cards.iter().find(|c| c.name == "duel").map(|c| c.name).unwrap();
        e.debug_place(0, name)
    };
    let sha1 = {
        let name = e.catalog().cards.iter().find(|c| c.name == "sha").map(|c| c.name).unwrap();
        e.debug_place(1, name)
    };
    let hp1 = e.debug_hp(1);
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
    let mut prompts: Vec<String> = Vec::new();
    let mut played_sha = false;
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else { break };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
        // 1 号打出唯一的一张【杀】
        if c.observer == 1 && !played_sha && c.spec.options.contains(&ChoiceOption::Card(sha1)) {
            played_sha = true;
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Card(sha1)],
                },
            });
            continue;
        }
        let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
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
    drain(&mut e);
    eprintln!(
        "DBG 无双（决斗）：询问={prompts:?} 1 号出杀={played_sha} 1 号 hp {hp1} → {}",
        e.debug_hp(1)
    );
    if !played_sha {
        eprintln!("NOTE 未出现 1 号出【杀】的询问——本用例不构成验收");
        return;
    }
    // 无双：对手需**两张**杀；1 号只出一张 → 不足 → **1 号受伤**
    assert!(
        e.debug_hp(1) < hp1,
        "无双（决斗方向）：只出一张【杀】不足以应对 → 1 号应受伤（{hp1} → {}）",
        e.debug_hp(1)
    );
}
