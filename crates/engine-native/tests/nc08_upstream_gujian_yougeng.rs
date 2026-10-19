//! NC08 第⑦层（上游行为对照）样本 111：**来源 U05（古剑）** 食物牌
//! `yougeng`（酉羹）。
//!
//! 依据：原生 `content/host/step/gujian.rs:182` 注释 ——
//! 「**阶段开始（判定阶段替位准备阶段）：已受伤且体力值为全场最少或之一 → 回复 1**；
//! 倒计时扣减（**登记差异：与上游同点扣减**）」。
//!
//! 由注释推导的规则点：
//! 1. **时机**：**阶段开始**（原生以**判定阶段替位准备阶段**）；
//! 2. **门槛**：**已受伤**且**体力值为全场最少（或之一）**；
//! 3. **效果**：**回复 1 点体力**；并有**倒计时**（同点扣减）。
//!
//! **本样本的定位**：验证「**受伤且体力最少 → 回复 1**」与**反例**（满血不回复）。
//!
//! 未覆盖（登记）：上游逐字依据（本样本以原生注释为准）；`markcount` 递减与归零移除。

mod nc08_common;
use engine_native::{Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

/// 载入 **古剑包**建局。
fn gujian_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("shahai-gujian-upstream-noname/assets/cards.json"))
        .expect("读古剑包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 跑一次：`wounded` 决定 0 号是否受伤（且是否为全场最少体力）；返回 0 号 hp 变化。
fn run(seed: u64, wounded: bool) -> i32 {
    let mut e = gujian_engine(seed);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    if wounded {
        let max0 = e.state_core().player(0).max_hp as i32;
        e.debug_set_hp(0, max0 - 1); // 受伤（且低于其他满血角色 ⇒ 全场最少）
    }
    let food = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("yougeng"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("酉羹在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: food, targets: vec![0] },
    });
    assert!(
        matches!(
            out,
            engine_native::CommandOutcome::Accepted { .. }
                | engine_native::CommandOutcome::Pending { .. }
        ),
        "酉羹应可对自己发动：{out:?}"
    );
    drain(&mut e);
    let has = e.debug_has_skill(0, "gj_food_yougeng");
    let hp_before = e.debug_hp(0);
    // 驱动一整轮 → 0 号**下一次阶段开始**（判定阶段替位准备阶段）应触发
    for _ in 0..400 {
        let Some(c) = e.open_choice().cloned() else {
            let actor = e.turn().current;
            let _ = e.submit(GameCommand {
                actor,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::NextPhase,
            });
            continue;
        };
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
    let hp_after = e.debug_hp(0);
    eprintln!("DBG 酉羹：子技={has}（wounded={wounded}）；hp {hp_before} → {hp_after}");
    hp_after - hp_before
}

/// 上游点 2+3（**行为面 · 正反例**）：受伤且体力最少 → **回复 1**；满血 → **不回复**。
#[test]
fn upstream_yougeng_recovers_when_lowest_and_wounded() {
    let d_wounded = run(20274101, true);
    let d_full = run(20274102, false);
    eprintln!("DBG 酉羹：受伤且最少 → hp{d_wounded:+}；满血 → hp{d_full:+}");
    assert_eq!(
        d_wounded, 1,
        "酉羹：**受伤且体力全场最少**时，阶段开始应**回复 1**（实测 hp{d_wounded:+}）"
    );
    assert_eq!(
        d_full, 0,
        "酉羹（反例）：**满血**时不应回复（实测 hp{d_full:+}）"
    );
}
