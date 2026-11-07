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
        "/src/engine/play_rules.rs"
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

/// 上游点 3（**行为面 · 链杀**）：厉焰使用后**视为对目标使用一张【杀】**。
///
/// **本轮未验收（登记，含诊断）**：**根因已推进两步**——
/// ①**夹具缺口已修**：厉焰与**链杀子技**在包内是**两名技能**（`character.json`：
///   `ycnluweiyan` + `ycnluweiyan2`）→ 只授予前者时**完全不触发**（`询问=[]`）；同时授予后
///   **出现 `trigger_confirm`** ⇒ 链杀**确实被派发**；
/// ②**剩余问题**：确认后**虚拟杀未产生响应询问**（`respond_shan` 未出现、1 号手牌仍含【闪】，
///   而水攻本身已结算：1 号装备 1 → 0）→ 疑**嵌套结算**（虚拟杀在【水攻】自身结算内发起被丢弃）
///   → 详见修复清单第 31 项。
/// 用例以 `#[ignore]` 守护，**不写成通过**。
///
/// **登记差异（原生注释自述）**：上游 `luweiyan2` 为
/// `chooseTarget("是否视为使用一张杀？", …)` —— **玩家选目标 + 可取消**；
/// 原生为**自动取座位序首目标**（`yunchou.rs:280` 注释「登记差异：上游玩家选目标+可取消，
/// 原生自动首目标」）→ 本用例固定该差异（断言**无目标询问**、但**确实产生了【杀】**）。
#[test]
fn upstream_luweiyan_chain_kill_fires_on_first_target() {
    let mut e = yunchou_engine(20270903);
    // 厉焰与**链杀子技**在包内是**两名技能**（character.json：`ycnluweiyan` + `ycnluweiyan2`）
    // → 夹具须**同时授予**（上游以 `group: "luweiyan2"` 关联，包数据把它们并列列出）
    e.debug_grant(0, "ycnluweiyan");
    e.debug_grant(0, "ycnluweiyan2");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 1 号：有装备（水攻合法目标）+ 一张【闪】（用于观测「虚拟杀」确实指向它）
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
    let shan = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name == "shan")
            .map(|(i, _)| i as engine_native::CardId)
            .expect("闪");
        nc08_common::put_hand(&mut e, 1, id);
        id
    };
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
    let equips_before = e.debug_equips(1).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: trick, targets: vec![1] },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "厉焰应可发动：{out:?}（装备={equip}）"
    );
    let mut prompts: Vec<String> = Vec::new();
    let mut saw_shan_response = false;
    let mut saw_target_choice = false;
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else { break };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
        if c.spec.prompt.contains("respond_shan") {
            saw_shan_response = true;
        }
        // 「选目标」类询问（原生不应出现——差异点）
        if c.spec.options.iter().filter(|o| matches!(o, engine_native::ChoiceOption::Player(_))).count() > 0
            && c.spec.prompt.contains("sha")
        {
            saw_target_choice = true;
        }
        // 1 号打出【闪】以观测虚拟杀
        if c.observer == 1 && c.spec.options.contains(&engine_native::ChoiceOption::Card(shan)) {
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![engine_native::ChoiceOption::Card(shan)],
                },
            });
            continue;
        }
        let sel: Vec<engine_native::ChoiceOption> = if c.spec.prompt == "trigger_confirm" {
            vec![engine_native::ChoiceOption::Confirm]
        } else if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
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
    eprintln!(
        "DBG 厉焰·链杀：询问={prompts:?} 出现闪响应={saw_shan_response} 出现选目标={saw_target_choice} \
         1 号手牌={:?} 1 号装备 {equips_before} → {}（水攻是否结算）",
        e.debug_hand(1),
        e.debug_equips(1).len()
    );
    assert!(
        saw_shan_response,
        "链杀：厉焰结算后应对座位序首目标（1 号）**视为使用【杀】**（应出现【闪】响应询问），实测 {prompts:?}"
    );
    assert!(
        !saw_target_choice,
        "**差异登记**：原生自动取首目标（无「是否视为使用一张杀」的目标询问），上游为玩家选目标+可取消"
    );
    assert!(
        !e.debug_hand(1).contains(&shan),
        "链杀：1 号已用【闪】响应虚拟杀（手牌应不含该【闪】）"
    );
}
