//! NC08 第⑦层（上游行为对照）样本 70：**来源 U05（杀海拾遗·运筹）** 武将技
//! `yaliang`（雅量）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js`，`yaliang`）：
//!
//! ```js
//! yaliang: {
//!     trigger: { player: "damageEnd", source: "damageSource" },   // **受伤者/来源皆可触发**
//!     getIndex: event => event.num,                                // 按伤害点数取索引
//!     filter(event) {
//!         if (event._notrigger.includes(event.player)) return false;
//!         return event.num && event.source?.isIn() && event.player?.isIn()
//!             && event.source !== event.player;                    // **来源≠受伤者**
//!     },
//!     async content(event, trigger, player) {
//!         await game.asyncDraw([trigger.player, trigger.source].sortBySeat());   // **双方按座位序各摸 1**
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机**：**伤害结算结束**（受伤者与来源**双方**都可作为持技者触发）；
//! 2. **门槛**：本次有伤害数、来源与受伤者**均存活**、且**来源≠受伤者**；
//! 3. **效果**：**受伤者与来源按座位序各摸 1 张**（`asyncDraw` 同时摸）。
//!
//! 原生实现引用：`content/host/step/yunchou.rs:14`（`seats = targets` → `sort_unstable` + `dedup`
//! → 存活者各 `draw_cards(s, 1)`）；注册/过滤见 `content/host/behavior.rs`（tag 244）。
//!
//! **登记差异**：本样本**未发现**差异（座位序、双方各摸 1 与上游一致）；
//! 保留项：`ai.maixie` 策略面；**「来源==受伤者」分支**（自伤）未构造（登记待办）；
//! `getIndex: event.num`（**多点伤害**时按点数取索引）未覆盖（登记待办）。
//!
//! 未覆盖（登记）：自伤分支；多点伤害的索引语义；`preHidden` 表现面。

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

/// 上游点 3（**行为面**）：来源造成伤害后，**双方各摸 1**。
#[test]
fn upstream_yaliang_both_draw_one() {
    let mut e = yunchou_engine(20270201);
    e.debug_grant(0, "ycnyaliang"); // 持技者=来源（0 号）
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let hand0 = e.debug_hand(0).len();
    let hand1 = e.debug_hand(1).len();
    // 0 号对 1 号造成 1 点伤害（真实伤害路径）
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    // 驱动：确认触发（雅量非锁定 → 可能出现 trigger_confirm）
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
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
    eprintln!(
        "DBG 雅量：来源手牌 {hand0} → {}；受伤者手牌 {hand1} → {}",
        e.debug_hand(0).len(),
        e.debug_hand(1).len()
    );
    assert_eq!(
        e.debug_hand(0).len(),
        hand0 + 1,
        "雅量：**来源**应摸 1 张（{hand0} → {}）",
        e.debug_hand(0).len()
    );
    assert_eq!(
        e.debug_hand(1).len(),
        hand1 + 1,
        "雅量：**受伤者**应摸 1 张（{hand1} → {}）",
        e.debug_hand(1).len()
    );
}

/// 上游点 1+2（**源码事实**）：时机、门槛与座位序效果式。
#[test]
fn upstream_yaliang_source_matches() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/host/step/yunchou.rs"
    ))
    .expect("读 yunchou");
    let idx = src.find("YunchouYaliang =>").expect("雅量分支");
    let seg: String = src[idx..].chars().take(400).collect();
    assert!(
        seg.contains("sort_unstable") && seg.contains("draw_cards(s, 1)"),
        "原生依据：座位序 + 逐座位各摸 1：{seg}"
    );
    let up = std::fs::read_to_string(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../../archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js",
    ))
    .expect("读上游 yunchou/skill.js");
    let idx = up.find("\tyaliang: {").expect("上游雅量");
    let seg: String = up[idx..].chars().take(1100).collect();
    assert!(
        seg.contains("game.asyncDraw([trigger.player, trigger.source].sortBySeat())"),
        "上游依据：双方按座位序各摸 1：{seg}"
    );
    assert!(
        seg.contains("event.source !== event.player"),
        "上游依据：来源≠受伤者（自伤不触发）：{seg}"
    );
}
