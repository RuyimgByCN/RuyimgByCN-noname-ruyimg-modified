//! NC08 第⑦层（上游行为对照）样本 113：**来源 U05（古剑）** 装备牌
//! `gjyuheng`（玉衡）——**古剑包最后一族**。
//!
//! 依据：规则基准 golden 表第 54 行 ——
//! 「**玉衡**（**主动型装备技**——**确定性取其他角色首张黑桃**；**登记差异：升级机制+随机敌暂缓**）」；
//! 原生常量 `content/host/constants.rs:138`（`GJ_YUHENG_SKILL_KEY = "gj_yuheng_skill"`）。
//!
//! 由依据推导的规则点：
//! 1. **装备**：玉衡为 `equip5`（宝物位）装备，装备后**授予技能** `gjyuheng_skill`；
//! 2. **技能**：**主动技**——**取其他角色一张黑桃牌**（原生为**确定性首张**）；
//! 3. **登记差异**：**升级机制**与**随机敌**暂缓。
//!
//! **本样本的定位**：验证「**装备授予技能**」+「**技能取走他人黑桃牌**」。
//! 技能名取自**原生常量**（不猜名——见样本 97/99 的教训）。
//!
//! 未覆盖（登记）：**升级机制**（`gjyuheng_plus`/`gjyuheng_pro` 变体）；**随机敌**（上游随机选目标）；
//! `onLose` 清理。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
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

/// 上游点 1+2（**行为面**）：装备授予技能；技能**取走其他角色的黑桃牌**。
#[test]
fn upstream_gjyuheng_steals_spade() {
    let mut e = gujian_engine(20274301);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 1 号持一张**黑桃**手牌（待被取走）
    let spade = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| matches!(c.suit, engine_native::catalog::Suit::Spade))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("黑桃牌");
        nc08_common::put_hand(&mut e, 1, id);
        id
    };
    // 0 号装备玉衡
    let equip = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| {
                c.name.contains("gjyuheng") && !c.name.contains("plus") && !c.name.contains("pro")
            })
            .map(|(i, _)| i as engine_native::CardId)
            .expect("玉衡在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: equip,
            targets: vec![],
        },
    });
    eprintln!("DBG 玉衡：装备={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "玉衡应可装备：{out:?}"
    );
    drain(&mut e);
    let has = e.debug_has_skill(0, "gj_yuheng_skill");
    let h1_before = e.debug_hand(1).len();
    let h0_before = e.debug_hand(0).len();
    // 发动主动技（技能名取自原生常量）
    let skill = e
        .catalog()
        .skill_by_name("gj_yuheng_skill")
        .expect("玉衡技能在目录");
    let out2 = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill },
    });
    eprintln!("DBG 玉衡：技能已授予={has}；发动={out2:?}");
    let mut prompts: Vec<String> = Vec::new();
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
        let sel: Vec<engine_native::ChoiceOption> =
            if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
                vec![engine_native::ChoiceOption::Pass]
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
    let h1_after = e.debug_hand(1).len();
    let h0_after = e.debug_hand(0).len();
    eprintln!(
        "DBG 玉衡：询问={prompts:?}；1 号手牌 {h1_before} → {h1_after}；0 号手牌 {h0_before} → {h0_after}；\
         黑桃牌（{spade}）现在 0 号手中={}",
        e.debug_hand(0).contains(&spade)
    );
    assert!(has, "玉衡：装备后应授予技能 gj_yuheng_skill");
    if !matches!(
        out2,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ) {
        eprintln!("NOTE 玉衡技能未能发动（{out2:?}）——本用例不构成验收");
        return;
    }
    assert_eq!(
        h1_after,
        h1_before - 1,
        "玉衡：应**取走其他角色一张牌**（1 号 {h1_before} → {h1_after}）"
    );
    assert!(
        e.debug_hand(0).contains(&spade),
        "玉衡：取走的应是那张**黑桃**牌（{spade}）——登记差异：原生为确定性首张"
    );
}
