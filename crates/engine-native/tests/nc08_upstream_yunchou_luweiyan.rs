//! NC08 第⑦层（上游行为对照）样本 77：**来源 U05（杀海拾遗·运筹）** 武将技
//! `luweiyan`（厉焰，含 `luweiyan2` 链杀）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js`，`luweiyan`）：
//!
//! ```js
//! luweiyan: {
//!     enable: "phaseUse", usable: 1,                    // **出牌阶段限一次**
//!     filterCard(card) { return get.type(card) !== "basic"; },   // **非基本牌**
//!     position: "hse",                                  // **手牌 + 装备区 + 判定区**
//!     filter(event, player) { return player.hasCard(c => get.type(c) !== "basic", "hes"); },
//!     viewAs: { name: "shuigong" },                     // **当【水攻】使用**
//!     prompt: "将一张非基本牌当【水攻】使用",
//!     group: "luweiyan2",                               // 链杀子技
//! },
//! luweiyan2: {
//!     trigger: { player: "useCardAfter" },
//!     filter(event, player) { return event.skill === "luweiyan" && …目标可杀…; },
//!     content() { … chooseTarget("是否视为使用一张杀？", 限本牌目标) … },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **转化**：**非基本牌**（`type !== "basic"`）当**【水攻】**使用；出牌阶段限一次；
//! 2. **牌区**：**`hse`（手牌 + 装备区 + 判定区）**；
//! 3. **链杀**（`luweiyan2`）：使用后，可对**本牌目标中可被【杀】指定者**视为使用一张【杀】。
//!
//! 原生实现引用：`engine/play_card.rs:189`（持 `ycnluweiyan` 且 `!card_kind(card).is_basic_kind()`
//! → `kind = CardKind::Shuigong`）；`play_card.rs:1253`（水攻目标门槛：**所有目标须有装备**）；
//! `content/host/factory.rs:420`（`YunchouLuweiyan2` 链杀机器）。
//!
//! **登记差异（牌区族第 7 例）**：上游 `position: "hse"`（**手牌 + 装备区 + 判定区**）↔ 原生转化
//! 发生在**出牌命令面**（`PlayCard` 的 `card` 须为**当前回合角色手牌**，见 `play_card.rs` 的手牌校验）
//! → **装备区/判定区的非基本牌不能当【水攻】**（与 `zhiheng`/`qixi`/`guose`/`wusheng`/`lijian`/
//! `qingguo` **同族**）。
//!
//! 未覆盖（登记）：**链杀**（`luweiyan2`）行为面；`check`/`ai` 策略面；水攻自身的伤害结算细节。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand};
use nc08_common::{advance_to_play, drain, next_dedup};

/// 运筹帷幄：**双产物桥接**（武将面在 `character.json`）。
fn yunchou_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json =
        std::fs::read_to_string(root.join("shahai-yunchou-upstream-noname/assets/character.json"))
            .expect("读运筹 character.json");
    let mut pack = content_native_loader::bridge_pack(&json).expect("桥接 pack 段").pack;
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接 generals 段");
    pack.generals = bridged.generals;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 1（**行为面**）：**非基本手牌**可当【水攻】使用（目标须有装备）。
#[test]
fn upstream_luweiyan_nonbasic_as_shuigong() {
    let mut e = yunchou_engine(20270901);
    e.debug_grant(0, "ycnluweiyan");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 1 号装备一件（水攻的合法目标须有装备）
    let equip = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("armor") || c.name.contains("weapon"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("装备牌");
        nc08_common::put_equips(&mut e, 1, id);
        id
    };
    // 0 号持一张**非基本**牌（锦囊）
    let trick = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("dismantle"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("锦囊牌");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: trick, targets: vec![1] },
    });
    eprintln!("DBG 厉焰：非基本牌当水攻 → {out:?}（1 号装备={equip}）");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "厉焰：非基本牌应可当【水攻】使用（目标有装备）：{out:?}"
    );
    drain(&mut e);
}

/// 上游点 1（**目标门槛反例**）：目标**无装备** → 不可作为水攻目标。
#[test]
fn upstream_luweiyan_shuigong_target_needs_equip() {
    let mut e = yunchou_engine(20270902);
    e.debug_grant(0, "ycnluweiyan");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 1 号**无装备**
    let trick = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("dismantle"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("锦囊牌");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: trick, targets: vec![1] },
    });
    eprintln!("DBG 厉焰：目标无装备 → {out:?}");
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "厉焰：水攻目标**须有装备**（1 号无装备应被拒）：{out:?}"
    );
    drain(&mut e);
}

/// **登记差异（牌区族第 7 例）**（源码事实固定）。
#[test]
fn upstream_luweiyan_zone_difference_is_registered() {
    let pc = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/engine/play_card.rs"
    ))
    .expect("读 play_card");
    let idx = pc.find("ycnluweiyan").expect("厉焰转化点");
    // 取锚点**之后**的窗口（转化条件写在 skill_by_name 之后）
    let seg: String = pc[idx..].chars().take(400).collect();
    assert!(
        seg.contains("is_basic_kind"),
        "原生依据：非基本牌 → 水攻：{seg}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../../archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js",
        ),
    )
    .expect("读上游 yunchou/skill.js");
    let idx = up.find("\tluweiyan: {").expect("上游厉焰");
    let seg: String = up[idx..].chars().take(700).collect();
    assert!(
        seg.contains("position: \"hse\"") && seg.contains("viewAs: { name: \"shuigong\" }"),
        "上游依据：position hse（含装备区/判定区）+ 当水攻：{seg}"
    );
    assert!(
        seg.contains("get.type(card) !== \"basic\""),
        "上游依据：非基本牌：{seg}"
    );
}
