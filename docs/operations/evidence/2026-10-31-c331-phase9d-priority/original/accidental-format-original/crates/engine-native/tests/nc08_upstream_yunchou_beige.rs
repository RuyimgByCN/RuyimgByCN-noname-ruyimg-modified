//! NC08 第⑦层（上游行为对照）样本 82：**来源 U05（杀海拾遗·运筹）** 武将技
//! `olbeige`（悲歌 / **refresh 变体**；运筹包蔡文姬技能之一）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/refresh/skill.js:3957`，`olbeige`）：
//!
//! ```js
//! olbeige: {
//!     trigger: { global: "damageEnd" }, logTarget: "player",
//!     filter(event, player) {
//!         return event.card && event.card.name == "sha"          // **因【杀】受到伤害**
//!             && event.player.isIn() && player.countCards("he") > 0;
//!     },
//!     prompt2: "令其进行判定，然后你可根据判定结果，弃置一张牌并令其执行对应效果。",
//!     async content(event, trigger, player) {
//!         let result = await trigger.player.judge().forResult();     // ① **受伤者判定**
//!         switch (result.suit) {                                     // ② 按**花色**分支
//!             case "heart": … 令其**回复 1 点体力** …
//!             …
//!         }
//!         // ②之前先问「是否弃置一张牌」（可取消）
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机/门槛**：任意角色**因【杀】受到伤害后**；持技者须有 `he` 牌；
//! 2. **流程**：受伤者**判定** → 持技者**可弃一张牌**（可取消）→ 按判定**花色**执行对应效果；
//! 3. **红桃分支**：令受伤者**回复 1 点体力**。
//!
//! 原生实现引用：`content/host/step/yunchou.rs`（`YunchouBeige`：`draw_judge_card()` → 弃 1 张（可取消）
//! → 按判定花色分支 + 收回奖励；注释自述「**登记差异：草花来源弃 2 取首两张确定性**；
//! `addExpose`（AI 面）不做」）。
//!
//! **登记差异（本文件固定）**：**草花分支的取牌** —— 上游为玩家自选（弃 2），原生取**首两张**（确定性）
//! → 与「**取牌选择权**」族**同族**（族内第 5 例）。
//!
//! 未覆盖（登记）：黑桃/方块/草花三个分支的**行为面**（本样本只覆盖**红桃**分支）；
//! `addExpose`（AI 面）。

mod nc08_common;
use engine_native::{ChoiceOption, Engine, GameAction, GameCommand};
use nc08_common::{advance_to_play, drain, next_dedup};

/// 运筹帷幄：**双产物桥接**（武将面在 `character.json`）。
fn yunchou_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json =
        std::fs::read_to_string(root.join("shahai-yunchou-upstream-noname/assets/character.json"))
            .expect("读运筹 character.json");
    let mut pack = content_native_loader::bridge_pack(&json)
        .expect("桥接 pack 段")
        .pack;
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接 generals 段");
    pack.generals = bridged.generals;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 1+2+3（**行为面 · 红桃分支**）：因【杀】受伤 → 判定红桃 → 弃 1 张 → **受伤者回复 1**。
#[test]
fn upstream_beige_heart_branch_recovers() {
    let mut e = yunchou_engine(20271301);
    e.debug_grant(2, "beige"); // 2 号持悲歌
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 2 号须有 he 牌（弃一张的费用）
    {
        let name = e
            .catalog()
            .cards
            .iter()
            .find(|c| c.name == "shan")
            .map(|c| c.name)
            .unwrap();
        let _ = e.debug_place(2, name);
    }
    // 判定牌：**确实为红桃**（置顶后回读 deck.last() 核对花色——新纪律）
    let heart_ok = {
        let mut ok = false;
        let names: Vec<&'static str> = e
            .catalog()
            .cards
            .iter()
            .filter(|c| matches!(c.suit, engine_native::catalog::Suit::Heart))
            .map(|c| c.name)
            .collect();
        for name in names.iter().take(40) {
            let _ = e.debug_set_deck_top(name);
            if matches!(
                e.state_core()
                    .deck
                    .last()
                    .map(|c| e.catalog().cards[*c as usize].suit),
                Some(engine_native::catalog::Suit::Heart)
            ) {
                ok = true;
                break;
            }
        }
        ok
    };
    assert!(heart_ok, "应能把一张**确实为红桃**的牌置于牌堆顶");
    // 0 号用【杀】打 1 号（1 号无手牌 → 直接受伤）
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
    let hp1_before = e.debug_hp(1);
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
            engine_native::CommandOutcome::Accepted { .. }
                | engine_native::CommandOutcome::Pending { .. }
        ),
        "出杀应被受理：{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    let mut discarded = false;
    for _ in 0..32 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
        // 弃一张牌（悲歌的费用；可能出现在 2 号身上）
        if !discarded {
            let cards: Vec<ChoiceOption> = c
                .spec
                .options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Card(_)))
                .copied()
                .collect();
            if !cards.is_empty() && c.observer == 2 {
                discarded = true;
                let _ = e.submit(GameCommand {
                    actor: c.observer,
                    expected_revision: e.revision(),
                    dedup_id: next_dedup(),
                    action: GameAction::Answer {
                        choice_id: c.id,
                        selection: vec![cards[0]],
                    },
                });
                continue;
            }
        }
        let sel: Vec<ChoiceOption> = if c.spec.prompt == "trigger_confirm" {
            vec![ChoiceOption::Confirm]
        } else if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec
                .options
                .iter()
                .take(c.spec.min.max(1))
                .copied()
                .collect()
        };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        });
    }
    drain(&mut e);
    let hp1_after = e.debug_hp(1);
    eprintln!("DBG 悲歌：询问={prompts:?} 弃牌={discarded} 1 号 hp {hp1_before} → {hp1_after}");
    assert!(
        prompts.iter().any(|p| p.contains("beige")),
        "悲歌：应出现悲歌相关询问，实测 {prompts:?}"
    );
    assert!(
        hp1_after >= hp1_before,
        "悲歌（红桃分支）：受伤者应先受伤（−1）再**回复 1** → 净效果应不低于初始（{hp1_before} → {hp1_after}）"
    );
}

/// **登记差异**（源码事实固定）：草花分支取牌为「首两张」（上游玩家自选）。
#[test]
fn upstream_beige_club_choice_difference_is_registered() {
    let step = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/host/step/yunchou.rs"
    ))
    .expect("读 yunchou");
    assert!(
        step.contains("草花来源弃 2 取首两张确定性"),
        "原生注释应自述草花分支的确定性取牌差异"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/refresh/skill.js"),
    )
    .expect("读上游 refresh/skill.js");
    let idx = up.find("\tolbeige: {").expect("上游 olbeige");
    let seg: String = up[idx..].chars().take(2000).collect();
    assert!(
        seg.contains("global: \"damageEnd\"")
            && seg.contains("event.card.name == \"sha\"")
            && seg.contains("trigger.player.judge()"),
        "上游依据：damageEnd + 因杀受伤 + 受伤者判定：{seg}"
    );
}
