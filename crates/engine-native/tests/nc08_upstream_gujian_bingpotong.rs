//! NC08 第⑦层（上游行为对照）样本 102：**来源 U05（古剑）** 机关牌
//! `bingpotong`（天女散花）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`，`bingpotong`）：
//!
//! ```js
//! bingpotong: {
//!     type: "jiguan", enable: true, wuxieable: true,
//!     filterTarget(card, player, target) { return target.countCards("h") > 0; },
//!     selectTarget: [1, 3],                                   // 1–3 名目标
//!     content() {
//!         "step 0"; player.chooseCard("请展示一张手牌", true)…   // 使用者展示一张
//!         "step 1"; target.chooseCard("请展示一张手牌", true)…
//!                   .prompt2 = "若与…展示的牌相同，…弃置展示的牌，你失去1点体力";
//!         "step 3"; if (get.color(event.card2) === get.color(event.card1)) { … }
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：`he`/手牌有牌的角色（1–3 名）；
//! 2. **流程**：**使用者与目标各展示一张手牌** → **同色**则：
//!    **使用者弃置其展示的牌** + **目标失去 1 点体力**（`prompt2` 原文）；
//!    **异色无事**。
//!
//! 原生实现引用：`content/host/step/jiguan.rs:8-10`（注释：「使用者自选展示一张 → 目标自选展示一张
//! → **同色则使用者弃展示牌、目标失去 1 点体力**（规则 v77 原语）；**异色无事**。展示不移动牌」）。
//!
//! **本样本的定位**：验证**同色/异色两分支**，并用**护甲**判别「失去体力」与「伤害」的语义
//! （`loseHp` 不受护甲吸收；`damage` 会）。
//!
//! 未覆盖（登记）：`selectTarget: [1,3]` 的**多目标**面；`wuxieable` 无懈面。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
use nc08_common::{advance_to_play, drain, next_dedup};

/// 载入 **古剑包**建局。
fn gujian_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("shahai-gujian-upstream-noname/assets/cards.json"))
        .expect("读古剑包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 跑一次：`same_color` 决定双方展示牌是否同色；`hujia` 给目标预置护甲。
/// 返回 `(使用者手牌变化, 目标手牌变化, 目标 hp 变化, 目标护甲变化)`。
fn run(seed: u64, same_color: bool, hujia: u8) -> (i64, i64, i32, i32) {
    let mut e = gujian_engine(seed);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 使用者展示一张**黑**牌；目标按 same_color 展示黑或红
    let s0 = engine_native::catalog::Suit::Spade;
    let s1 = if same_color {
        engine_native::catalog::Suit::Club
    } else {
        engine_native::catalog::Suit::Heart
    };
    let _c0 = e.debug_place_suit(0, s0);
    let _c1 = e.debug_place_suit(1, s1);
    if hujia > 0 {
        e.debug_set_hujia(1, hujia);
    }
    let card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("bingpotong"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("天女散花在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let h0 = e.debug_hand(0).len() as i64;
    let h1 = e.debug_hand(1).len() as i64;
    let hp1 = e.debug_hp(1);
    let hj1 = e.state_core().player(1).hujia as i32;
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![1] },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "天女散花应可发动：{out:?}"
    );
    // 双方各有「展示一张」询问 → 各选第一张
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else { break };
        let cards: Vec<ChoiceOption> = c
            .spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Card(_)))
            .copied()
            .collect();
        if !cards.is_empty() {
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer { choice_id: c.id, selection: vec![cards[0]] },
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
    (
        e.debug_hand(0).len() as i64 - h0,
        e.debug_hand(1).len() as i64 - h1,
        e.debug_hp(1) - hp1,
        e.state_core().player(1).hujia as i32 - hj1,
    )
}

/// 上游点 2（**行为面 · 两分支**）：同色 → 使用者弃展示牌 + 目标失去 1 点体力；异色无事。
#[test]
fn upstream_bingpotong_same_color_branch() {
    let (dh0_same, dh1_same, dhp_same, _) = run(20273201, true, 0);
    let (dh0_diff, dh1_diff, dhp_diff, _) = run(20273202, false, 0);
    eprintln!(
        "DBG 天女散花：同色 → 使用者手牌{dh0_same:+}/目标手牌{dh1_same:+}/目标 hp{dhp_same:+}；\
         异色 → 使用者手牌{dh0_diff:+}/目标手牌{dh1_diff:+}/目标 hp{dhp_diff:+}"
    );
    // 同色：使用者打出本体 −1，再弃展示牌 −1 ⇒ −2；目标展示不移动牌 ⇒ 0；目标 hp −1
    assert_eq!(dh0_same, -2, "同色：使用者应弃置其展示的牌（打出本体 −1、弃展示牌 −1）");
    assert_eq!(dh1_same, 0, "同色：目标**展示不移动牌**（手牌不变）");
    assert_eq!(dhp_same, -1, "同色：目标应**失去 1 点体力**");
    // 异色：仅打出本体 −1；目标无事
    assert_eq!(dh0_diff, -1, "异色：仅打出本体（−1），不弃展示牌");
    assert_eq!(dh1_diff, 0, "异色：目标手牌不变");
    assert_eq!(dhp_diff, 0, "异色：目标**无事**（hp 不变）");
}

/// **语义判别**：目标**有护甲**时，同色分支是否仍掉血（`loseHp` 不被护甲吸收）。
#[test]
fn upstream_bingpotong_hp_loss_ignores_hujia() {
    let (_, _, dhp, dhj) = run(20273203, true, 2);
    eprintln!("DBG 天女散花（目标护甲 2）：目标 hp{dhp:+}、护甲{dhj:+}");
    assert_eq!(
        dhp, -1,
        "同色：目标**失去 1 点体力**应为 `loseHp` 语义（**不被护甲吸收**）——实测 hp{dhp:+}、护甲{dhj:+}"
    );
    assert_eq!(dhj, 0, "护甲不应被消耗（说明不是 damage）");
}
