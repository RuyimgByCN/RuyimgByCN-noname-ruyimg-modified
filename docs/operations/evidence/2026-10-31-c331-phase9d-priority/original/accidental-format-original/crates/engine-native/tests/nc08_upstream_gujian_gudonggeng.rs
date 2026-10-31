//! NC08 第⑦层（上游行为对照）样本 106：**来源 U05（古剑）** 食物牌
//! `gudonggeng`（骨董羹）。
//!
//! 依据：原生 `content/host/step/gujian.rs:261` 注释 ——
//! 「骨董羹（古剑食物，规则 v48）：**受到杀伤害：伤害 -1；一次性状态，触发即移除**」；
//! 规则基准 golden 表第 45 行记：「咕咚羹（**受杀伤害 -1，一次性移除**）」。
//!
//! 由注释推导的规则点：
//! 1. **门槛**：**受到【杀】造成的伤害**时生效；
//! 2. **效果**：该次伤害 **−1**；且状态**一次性**（触发即移除 ⇒ 后续伤害**不再减免**）。
//!
//! **本样本的定位**：验证「**减 1**」与「**一次性**」两点 —— 后者需**同一回合两次杀伤害**，
//! 故用**同族食物牌元宝肉**（额外一张杀）解开 `ShaLimit` 阻塞（沿用样本 105 的方法）。
//!
//! 未覆盖（登记）：`markcount` 递减；非【杀】伤害是否触发（上游为「杀伤害」限定）。

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

fn find(e: &Engine, key: &str) -> engine_native::CardId {
    e.catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(_, c)| c.name.contains(key))
        .map(|(i, _)| i as engine_native::CardId)
        .unwrap_or_else(|| panic!("{key} 在目录"))
}

/// 上游点 2（**行为面**）：受杀伤害 **−1**；且**一次性**（第二次不再减免）。
#[test]
fn upstream_gudonggeng_reduces_once() {
    let mut e = gujian_engine(20273601);
    assert!(
        advance_to_phase(&mut e, 0, Phase::Play),
        "应到达 0 号出牌阶段（食物牌须在出牌阶段使用）"
    );
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 0 号吃骨董羹（对自己）
    let food = find(&e, "gudonggeng");
    nc08_common::put_hand(&mut e, 0, food);
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
        "骨董羹应可对自己发动：{out:?}"
    );
    drain(&mut e);
    let has = e.debug_has_skill(0, "gj_food_gudonggeng");
    // 轮到 1 号：吃元宝肉（额外一张杀）→ 对 0 号连出两张【杀】
    assert!(
        advance_to_phase(&mut e, 1, Phase::Play),
        "应到达 1 号出牌阶段"
    );
    let meat = find(&e, "yuanbaorou");
    nc08_common::put_hand(&mut e, 1, meat);
    let out_meat = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: meat,
            targets: vec![1],
        },
    });
    drain(&mut e);
    let shas: Vec<engine_native::CardId> = (0..2)
        .map(|_| {
            let name = e
                .catalog()
                .cards
                .iter()
                .find(|c| c.name == "sha")
                .map(|c| c.name)
                .unwrap();
            e.debug_place(1, name)
        })
        .collect();
    let hp0_before = e.debug_hp(0);
    let out1 = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: shas[0],
            targets: vec![0],
        },
    });
    drain(&mut e);
    let hp0_after1 = e.debug_hp(0);
    let out2 = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: shas[1],
            targets: vec![0],
        },
    });
    drain(&mut e);
    let hp0_after2 = e.debug_hp(0);
    eprintln!(
        "DBG 骨董羹：子技={has}；元宝肉发动={out_meat:?}；第一张杀={out1:?} hp {hp0_before} → {hp0_after1}；\
         第二张杀={out2:?} hp {hp0_after1} → {hp0_after2}"
    );
    assert!(has, "骨董羹：应获得子技 gj_food_gudonggeng");
    assert_eq!(
        hp0_after1, hp0_before,
        "骨董羹：受【杀】伤害应 **−1**（1 点伤害减为 0 ⇒ hp 不变；{hp0_before} → {hp0_after1}）"
    );
    assert_eq!(
        hp0_after2,
        hp0_after1 - 1,
        "骨董羹（**一次性**）：第二次【杀】伤害**不再减免**（应 −1；{hp0_after1} → {hp0_after2}）"
    );
}
