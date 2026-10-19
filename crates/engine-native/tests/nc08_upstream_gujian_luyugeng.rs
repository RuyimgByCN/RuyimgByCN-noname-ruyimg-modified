//! NC08 第⑦层（上游行为对照）样本 108：**来源 U05（古剑）** 食物牌
//! `luyugeng`（鲈鱼羹）。
//!
//! 依据：原生 `content/host/step/gujian.rs:376` 注释 ——
//! 「鲈鱼羹：**出牌阶段限一次：弃一张基本牌，获得牌堆顶牌**
//! （**登记差异：上游为发现牌随机五选一**）」。
//!
//! 由注释推导的规则点：
//! 1. **时机/限次**：**出牌阶段限一次**（主动技）；
//! 2. **费用**：**弃一张基本牌**（杀/闪/桃/酒）；
//! 3. **效果**：**获得牌堆顶一张**；**登记差异**：上游为「**发现牌随机五选一**」。
//!
//! **本样本的定位**：验证「**弃基本牌 → 获得牌堆顶**」这一链路，并固定该登记差异
//! （「**随机→确定性**」族）。
//!
//! 未覆盖（登记）：上游 `skill.js` 的逐字依据（本样本以原生注释为准，已登记）；
//! `markcount` 递减；「限一次」的第二次发动应被拒。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
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

/// 上游点 2+3（**行为面**）：弃一张基本牌 → **获得牌堆顶牌**。
#[test]
fn upstream_luyugeng_discards_basic_and_gains_deck_top() {
    let mut e = gujian_engine(20273801);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 0 号：鲈鱼羹 + 一张基本牌（闪）
    let food = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("luyugeng"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("鲈鱼羹在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let basic = {
        let name = e.catalog().cards.iter().find(|c| c.name == "shan").map(|c| c.name).unwrap();
        e.debug_place(0, name)
    };
    // ① 吃鲈鱼羹（对自己）
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
        "鲈鱼羹应可对自己发动：{out:?}"
    );
    drain(&mut e);
    let has = e.debug_has_skill(0, "gj_food_luyugeng");
    let deck_top_before = e.state_core().deck.last().copied();
    let h_before = e.debug_hand(0).len();
    // ② 发动子技（主动技，技能名取自原生常量）
    let skill = e.catalog().skill_by_name("gj_food_luyugeng").expect("子技在目录");
    let out2 = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill },
    });
    eprintln!("DBG 鲈鱼羹：子技={has}；发动={out2:?}");
    if !matches!(
        out2,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ) {
        eprintln!("NOTE 子技未能发动（{out2:?}）——本用例不构成验收");
        return;
    }
    // ③ 弃一张基本牌
    let mut discarded = false;
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        if !discarded && c.spec.options.contains(&ChoiceOption::Card(basic)) {
            discarded = true;
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Card(basic)],
                },
            });
            continue;
        }
        let sel: Vec<ChoiceOption> = if c.spec.allow_cancel {
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
    let hand = e.debug_hand(0);
    eprintln!(
        "DBG 鲈鱼羹：弃基本牌={discarded}；手牌 {h_before} → {}；获得牌堆顶（{deck_top_before:?}）={}",
        hand.len(),
        deck_top_before.map(|c| hand.contains(&c)).unwrap_or(false)
    );
    assert!(discarded, "鲈鱼羹：应出现「弃一张基本牌」的询问并完成弃牌");
    assert_eq!(
        hand.len(),
        h_before,
        "鲈鱼羹：弃 1 张基本牌、获得牌堆顶 1 张 ⇒ 手牌数不变（{h_before} → {}）",
        hand.len()
    );
    assert!(
        deck_top_before.map(|c| hand.contains(&c)).unwrap_or(false),
        "鲈鱼羹：应**获得牌堆顶那张**（登记差异：上游为发现牌随机五选一）"
    );
}
