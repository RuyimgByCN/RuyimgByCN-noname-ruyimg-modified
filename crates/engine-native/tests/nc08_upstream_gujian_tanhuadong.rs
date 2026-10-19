//! NC08 第⑦层（上游行为对照）样本 107：**来源 U05（古剑）** 食物牌
//! `tanhuadong`（昙花冻）。
//!
//! 依据：原生 `content/host/step/gujian.rs:176` 注释 ——
//! 「昙花冻：**出牌阶段结束：摸 1**；倒计时扣减，归零移除状态」。
//!
//! 由注释推导的规则点：
//! 1. **时机**：**出牌阶段结束**（`phaseUseAfter`）；
//! 2. **效果**：**摸 1 张**；并有**倒计时**（归零移除状态）。
//!
//! **本样本的定位**：验证「出牌阶段结束摸 1」这一**时机面**（食物族 tick 的抽样）。
//!
//! 未覆盖（登记）：倒计时（`markcount`）的**归零移除**；上游对应实现的逐字依据（本样本以原生注释为准）。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

/// 载入 **古剑包**建局。
fn gujian_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("shahai-gujian-upstream-noname/assets/cards.json"))
        .expect("读古剑包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 1+2（**行为面**）：**出牌阶段结束 → 摸 1**。
#[test]
fn upstream_tanhuadong_draws_at_play_phase_end() {
    let mut e = gujian_engine(20273701);
    assert!(
        advance_to_phase(&mut e, 0, Phase::Play),
        "应到达 0 号出牌阶段"
    );
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let food = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("tanhuadong"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("昙花冻在目录");
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
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "昙花冻应可对自己发动：{out:?}"
    );
    drain(&mut e);
    let has = e.debug_has_skill(0, "gj_food_tanhuadong");
    let h_before = e.debug_hand(0).len();
    // 结束出牌阶段（下一阶段）→ 应触发「出牌阶段结束：摸 1」
    let out_next = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    drain(&mut e);
    let h_after = e.debug_hand(0).len();
    eprintln!(
        "DBG 昙花冻：子技={has}；结束出牌阶段={out_next:?}；手牌 {h_before} → {h_after}"
    );
    assert!(has, "昙花冻：应获得子技 gj_food_tanhuadong");
    assert_eq!(
        h_after,
        h_before + 1,
        "昙花冻：**出牌阶段结束应摸 1**（{h_before} → {h_after}）"
    );
}
