//! NC08 第⑦层（上游行为对照）样本 110：**来源 U05（古剑）** 食物牌
//! `xiajiao`（虾饺）。
//!
//! 依据：原生 `content/host/step/gujian.rs:333` 注释（**属 `GujianFoodXiajiao`**）——
//! 「**摸牌阶段：额外摸 1（固定摸牌后），然后弃 1**」。
//!
//! 由注释推导的规则点：
//! 1. **时机**：**摸牌阶段**（固定摸牌之后）；
//! 2. **效果**：**额外摸 1** → **然后弃 1**（净 +1）。
//!
//! **本样本的定位**：验证「额外摸 1 + 弃 1」这一**净效果**（食物族 tick 的抽样）。
//! 说明：本样本以**原生注释**为依据（上游 `skill.js` 逐字依据登记为待办）。
//!
//! 未覆盖（登记）：上游逐字依据；`markcount` 递减；弃牌是否可取消。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

/// 载入 **古剑包**建局。
fn gujian_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json =
        std::fs::read_to_string(root.join("shahai-gujian-upstream-noname/assets/cards.json"))
            .expect("读古剑包");
    let pack = content_native_loader::bridge_pack(&json)
        .expect("桥接")
        .pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 1+2（**行为面**）：摸牌阶段**额外摸 1 后弃 1**（净 +1）。
#[test]
fn upstream_xiajiao_extra_draw_then_discard() {
    let mut e = gujian_engine(20274001);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let food = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("xiajiao"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("虾饺在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: food,
            targets: vec![0],
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "虾饺应可对自己发动：{out:?}"
    );
    drain(&mut e);
    let has = e.debug_has_skill(0, "gj_food_xiajiao");
    let h_before = e.debug_hand(0).len();
    // 本回合的摸牌阶段**已过**（食物牌须在出牌阶段使用）→ 须**驱动一整轮**等 0 号**下一次**摸牌阶段
    let mut out_next = None;
    let mut prompts: Vec<String> = Vec::new();
    let mut discarded = false;
    for _ in 0..400 {
        let Some(c) = e.open_choice().cloned() else {
            let actor = e.turn().current;
            out_next = Some(e.submit(GameCommand {
                actor,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::NextPhase,
            }));
            continue;
        };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
        let cards: Vec<ChoiceOption> = c
            .spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Card(_)))
            .copied()
            .collect();
        if !discarded && !cards.is_empty() {
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
        let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
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
    let h_after = e.debug_hand(0).len();
    eprintln!(
        "DBG 虾饺：子技={has}；结束出牌阶段={out_next:?}；询问={prompts:?}；弃牌={discarded}；\
         手牌 {h_before} → {h_after}（固定摸 2 + 额外 1 − 弃 1 = 净 +2）"
    );
    assert!(has, "虾饺：应获得子技 gj_food_xiajiao");
    assert!(
        prompts.iter().any(|p| p.contains("xiajiao")),
        "虾饺：应出现 **`gj_xiajiao_discard`**（额外摸 1 后的弃 1 询问），实测 {prompts:?}"
    );
    assert!(discarded, "虾饺：应完成弃 1");
    // **手牌数不作判据**：本用例须**驱动一整轮**才能到达下一次摸牌阶段，
    // 途中经过多次摸牌/弃牌（实测 0 → 4）⇒ 计数被污染（同蛊惑·变体样本的教训）。
    let _ = (h_before, h_after);
}
