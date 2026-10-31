//! NC08 第⑦层（上游行为对照）样本 123：**来源 U07（hearth）** 锦囊
//! `linghunzhihuo`（灵魂之火）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js`）：
//!
//! ```js
//! linghunzhihuo: {
//!     type: "trick", cardnature: "fire", enable: true, filterTarget: true,
//!     content() {
//!         "step 0"; target.damage("fire");                       // **1 点火焰伤害**
//!         "step 1"; var hs = player.getCards("h");
//!                   if (hs.length) player.discard(hs.randomGet());   // **使用者随机弃 1 张手牌**
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：任意角色（`filterTarget: true`）；
//! 2. **效果**：目标受 **1 点火焰伤害**；随后**使用者弃 1 张手牌**（上游为**随机**一张）。
//!
//! 原生实现引用：`kernel/intrinsic.rs:1264`（`CardKind::Linghunzhihuo` →
//! `HostBehavior::LinghunzhihuoCard`）；`content/host/step/skills_damage.rs:749`。
//! 原生 `content/standard.rs:52` 记：「v40：…**灵魂之火**/照明弹/急耗卡（CardKind 三种 + 宿主行为）」。
//!
//! **本样本的定位**：验证「**目标受 1 点火伤 + 使用者弃 1 张手牌**」的净效果，
//! 并以 `询问=[]` 行为面确认「**随机 → 确定性**」族内口径。
//!
//! 未覆盖（登记）：`cardnature: fire` 的**属性修饰面**（藤甲等载体）；弃牌**具体弃了哪张**。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

/// 载入 **hearth 包**建局。
fn hearth_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json =
        std::fs::read_to_string(root.join("shahai-hearth-upstream-noname/assets/cards.json"))
            .expect("读 hearth 包");
    let pack = content_native_loader::bridge_pack(&json)
        .expect("桥接")
        .pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 2（**行为面**）：**目标受 1 点火伤**；**使用者弃 1 张手牌**。
#[test]
fn upstream_linghunzhihuo_damages_and_user_discards() {
    let mut e = hearth_engine(20275301);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 0 号：本体 + 2 张手牌（供「弃 1」）；1 号无手牌（避免闪等干扰）
    let card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("linghunzhihuo"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("灵魂之火在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    for _ in 0..2 {
        let name = e
            .catalog()
            .cards
            .iter()
            .find(|c| c.name == "shan")
            .map(|c| c.name)
            .unwrap();
        e.debug_place(0, name);
    }
    let hp1_before = e.debug_hp(1);
    let h0_before = e.debug_hand(0).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card,
            targets: vec![1],
        },
    });
    eprintln!("DBG 灵魂之火：发动={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "灵魂之火应可发动：{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    for _ in 0..16 {
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
    let hp1_after = e.debug_hp(1);
    let h0_after = e.debug_hand(0).len();
    eprintln!(
        "DBG 灵魂之火：询问={prompts:?}；1 号 hp {hp1_before} → {hp1_after}；\
         0 号手牌 {h0_before} → {h0_after}（打出 −1、弃 1 ⇒ 净 −2）"
    );
    assert_eq!(
        hp1_after,
        hp1_before - 1,
        "灵魂之火：目标应受 **1 点火焰伤害**（hp {hp1_before} → {hp1_after}）"
    );
    assert_eq!(
        h0_after,
        h0_before - 2,
        "灵魂之火：**使用者应弃 1 张手牌**（打出 −1、弃 1 ⇒ 净 −2；{h0_before} → {h0_after}）"
    );
    eprintln!(
        "NOTE 灵魂之火：询问={prompts:?}（上游为 `hs.randomGet()` 随机 ⇒ 原生确定性属族内口径）"
    );
}
