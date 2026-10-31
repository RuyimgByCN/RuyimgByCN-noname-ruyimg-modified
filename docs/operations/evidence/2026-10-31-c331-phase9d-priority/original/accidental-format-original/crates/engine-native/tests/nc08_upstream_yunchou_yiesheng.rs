//! NC08 第⑦层（上游行为对照）样本 74：**来源 U05（杀海拾遗·运筹）** 武将技
//! `yiesheng`（野升）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js`，`yiesheng`）：
//!
//! ```js
//! yiesheng: {
//!     enable: "phaseUse",                                  // 出牌阶段
//!     filterCard: { color: "black" },                       // **仅黑色手牌**
//!     filter(event, player) { return player.countCards("h", { color: "black" }) > 0; },
//!     selectCard: [1, Infinity],                            // **任意张**
//!     prompt: "弃置任意张黑色手牌并摸等量的牌",
//!     content() { player.draw(cards.length); },             // **摸等量**
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机/门槛**：出牌阶段；须**有黑色手牌**；
//! 2. **费用**：**弃任意张黑色手牌**（`selectCard: [1, ∞]`；**红色手牌不可选**）；
//! 3. **效果**：**摸等量的牌**（手牌净变化 = 0）。
//!
//! 原生实现引用：`content/host/step/yunchou.rs:221`（`SelectCard` 段：候选 = **黑桃/草花**手牌，
//! `min 1 / max n / allow_cancel`，prompt `yiesheng_discard`；`Act` 段消费后摸等量）。
//!
//! **登记差异**：本样本**未发现**差异（候选仅黑色、弃 N 摸 N 与上游一致）；
//! 保留项：`check`（AI 弃牌评分）与 `ai.order` 策略面。
//!
//! 未覆盖（登记）：**取消**（`allow_cancel`）分支；`prompt` 文案表现面。

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
    let mut pack = content_native_loader::bridge_pack(&json)
        .expect("桥接 pack 段")
        .pack;
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接 generals 段");
    pack.generals = bridged.generals;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 2+3（**行为面**）：候选**仅黑色**；弃 2 张黑色 → **摸 2**（净变化 0、红牌留手）。
#[test]
fn upstream_yiesheng_discards_black_only_and_draws_equal() {
    let mut e = yunchou_engine(20270601);
    e.debug_grant(0, "ycnyiesheng");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 0 号：**两张不同的**黑色 + 一张红色手牌
    let black_ids: Vec<engine_native::CardId> = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .filter(|(_, c)| {
            matches!(
                c.suit,
                engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club
            )
        })
        .map(|(i, _)| i as engine_native::CardId)
        .take(2)
        .collect();
    assert_eq!(black_ids.len(), 2, "目录应有至少两张黑色牌");
    for id in &black_ids {
        nc08_common::put_hand(&mut e, 0, *id);
    }
    let blacks = black_ids.clone();
    let red = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| {
                matches!(
                    c.suit,
                    engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
                )
            })
            .map(|(i, _)| i as engine_native::CardId)
            .expect("任意红色牌");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let hand_before = e.debug_hand(0).len();
    let skill = e
        .catalog()
        .skill_by_name("ycnyiesheng")
        .expect("野升在目录");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "野升应可发动：{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    let mut offered_red = false;
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
        if c.spec.options.contains(&ChoiceOption::Card(red)) {
            offered_red = true;
        }
        // 弃掉两张黑色
        let sel: Vec<ChoiceOption> = blacks
            .iter()
            .filter(|b| c.spec.options.contains(&ChoiceOption::Card(**b)))
            .map(|b| ChoiceOption::Card(*b))
            .collect();
        if sel.len() == blacks.len() {
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            });
            continue;
        }
        let sel: Vec<ChoiceOption> = if c.spec.allow_cancel {
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
    let hand = e.debug_hand(0);
    eprintln!(
        "DBG 野升：询问={prompts:?} 红色入候选={offered_red} 手牌 {hand_before} → {}（红牌仍在={}，黑牌仍在={}）",
        hand.len(),
        hand.contains(&red),
        blacks.iter().any(|b| hand.contains(b))
    );
    assert!(
        prompts.iter().any(|p| p.contains("yiesheng")),
        "野升：应出现弃牌询问，实测 {prompts:?}"
    );
    assert!(!offered_red, "野升：**红色**手牌不应出现在候选中");
    assert_eq!(
        hand.len(),
        hand_before,
        "野升：弃 2 摸 2 → 手牌净变化应为 0（{hand_before} → {}）",
        hand.len()
    );
    assert!(hand.contains(&red), "野升：红色手牌应保留");
    assert!(
        !blacks.iter().any(|b| hand.contains(b)),
        "野升：被弃的两张黑色手牌应已离手"
    );
}

/// 上游点 1+2（**源码事实**）：候选花色与「弃 N 摸 N」。
#[test]
fn upstream_yiesheng_source_matches() {
    let step = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/host/step/yunchou.rs"
    ))
    .expect("读 yunchou");
    let idx = step.find("YunchouYiesheng =>").expect("野升分支");
    let seg: String = step[idx..].chars().take(1600).collect();
    assert!(
        seg.contains("Suit::Spade | crate::catalog::Suit::Club")
            && seg.contains("\"yiesheng_discard\""),
        "原生依据：候选仅黑桃/草花 + 询问名：{seg}"
    );
    let up = std::fs::read_to_string(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../../archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js",
    ))
    .expect("读上游 yunchou/skill.js");
    let idx = up.find("\tyiesheng: {").expect("上游野升");
    let seg: String = up[idx..].chars().take(520).collect();
    assert!(
        seg.contains("filterCard: { color: \"black\" }")
            && seg.contains("selectCard: [1, Infinity]")
            && seg.contains("player.draw(cards.length)"),
        "上游依据：仅黑色 + 任意张 + 摸等量：{seg}"
    );
}
