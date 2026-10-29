//! NC08 第⑦层（上游行为对照）样本 71：**来源 U05（杀海拾遗·运筹）** 武将技
//! `diy_jiaoxia`（狡黠）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js`，`diy_jiaoxia`）：
//!
//! ```js
//! diy_jiaoxia: {
//!     trigger: { target: "useCardToBegin" },                      // **成为牌的目标时**
//!     filter(event, player) {
//!         return event.card && get.color(event.card) === "red";   // **红色牌**
//!     },
//!     frequent: true,                                             // **可多次**
//!     content() { player.draw(); },                               // **摸 1**
//!     ai: { effect: { target(card, player, target) { if (get.color(card) === "red") return [1, 1]; } } },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机**：**成为某张牌的目标**（`useCardToBegin`，使用者不限）；
//! 2. **门槛**：该牌为**红色**（♥/♦）；
//! 3. **效果**：**摸 1 张**（`frequent` → 每次成为红牌目标都可发动）。
//!
//! 原生实现引用：`content/host/step/yunchou.rs:317`（`draw_cards(owner, 1)`）；
//! 过滤 `content/host/filtering/common.rs:473`（`CardUsed` 且 `targets.contains(&owner)`
//! 且花色为 **Heart|Diamond**）。
//!
//! **登记差异**：本样本**未发现**差异（红牌触发 / 黑牌不触发两分支一致）；
//! 保留项：`ai.effect` 策略面；`frequent` 的**多次**语义（本样本每分支只发动一次）。
//!
//! 未覆盖（登记）：同一回合内**多次**成为红牌目标时的重复触发；装备牌/锦囊牌作为红牌来源。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
use nc08_common::{advance_to_play, drain, next_dedup};

/// 运筹帷幄：**双产物桥接**（武将面在 `character.json`）。
fn yunchou_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json =
        std::fs::read_to_string(root.join("shahai-yunchou-upstream-noname/assets/character.json"))
            .expect("读运筹 character.json");
    let mut pack = content_native_loader::bridge_pack(&json).expect("桥接 pack 段").pack;
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接 generals 段");
    pack.generals = bridged.generals;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 跑一次「0 号用某花色的【杀】打 1 号」，返回 1 号手牌净变化。
fn sha_on_holder(seed: u64, red: bool) -> i64 {
    let mut e = yunchou_engine(seed);
    e.debug_grant(1, "ycnjiaoxia"); // 持技者=被杀指定者
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = {
        let id = e
            .catalog()
            .cards
            .iter()
            .position(|c| {
                c.name == "sha"
                    && if red {
                        matches!(
                            c.suit,
                            engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
                        )
                    } else {
                        matches!(
                            c.suit,
                            engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club
                        )
                    }
            })
            .expect("对应花色的杀") as engine_native::CardId;
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let before = e.debug_hand(1).len() as i64;
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "出杀应被受理：{out:?}"
    );
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel: Vec<ChoiceOption> = if c.spec.prompt == "trigger_confirm" {
            vec![ChoiceOption::Confirm]
        } else if c.spec.options.contains(&ChoiceOption::Pass) {
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
    e.debug_hand(1).len() as i64 - before
}

/// 上游点 2+3（**行为面 · 两分支**）：红色牌 → 摸 1；黑色牌 → 不摸。
#[test]
fn upstream_jiaoxia_red_triggers_black_does_not() {
    let red = sha_on_holder(20270301, true);
    let black = sha_on_holder(20270302, false);
    eprintln!("DBG 狡黠：成为红色【杀】目标 → 手牌净增 {red}；黑色【杀】→ 净增 {black}");
    assert_eq!(red, 1, "狡黠：成为**红色**牌目标应摸 1（实测 {red}）");
    assert_eq!(black, 0, "狡黠：**黑色**牌不触发（实测 {black}）");
}

/// 上游点 1+2（**源码事实**）：时机与花色门槛。
#[test]
fn upstream_jiaoxia_source_matches() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/host/filtering/common.rs"
    ))
    .expect("读 filtering/common");
    let idx = src.find("HostBehavior::YunchouJiaoxia =>").expect("狡黠过滤");
    let seg: String = src[idx..].chars().take(420).collect();
    assert!(
        seg.contains("targets.contains(&owner)") && seg.contains("Suit::Heart | crate::catalog::Suit::Diamond"),
        "原生依据：成为目标 + 红色花色：{seg}"
    );
    let step = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/host/step/yunchou.rs"
    ))
    .expect("读 yunchou");
    let idx = step.find("YunchouJiaoxia =>").expect("狡黠效果");
    let seg: String = step[idx..].chars().take(160).collect();
    assert!(seg.contains("draw_cards(self.owner, 1)"), "原生依据：摸 1：{seg}");
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../../archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js",
        ),
    )
    .expect("读上游 yunchou/skill.js");
    let idx = up.find("\tdiy_jiaoxia: {").expect("上游狡黠");
    let seg: String = up[idx..].chars().take(600).collect();
    assert!(
        seg.contains("useCardToBegin")
            && seg.contains("get.color(event.card) === \"red\"")
            && seg.contains("frequent: true"),
        "上游依据：成为牌目标 + 红色 + frequent：{seg}"
    );
}
