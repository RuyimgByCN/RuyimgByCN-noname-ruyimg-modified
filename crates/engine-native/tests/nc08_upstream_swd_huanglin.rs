//! NC08 第⑦层（上游行为对照）样本 157：**来源 U06（swd）** 法器
//! `huanglinzhicong`（黄麟之琮）——**`global` 卡窗口的第 6 张验证**（同族法器）。
//!
//! 上游源码（`skill.js` 的 **`g_huanglinzhicong`**，**逐字**）：
//!
//! ```js
//! huanglinzhicong: { type: "jiqi", global: "g_huanglinzhicong" },   // 卡本体无 content
//! g_huanglinzhicong: {
//!     trigger: { player: "phaseBegin" },                            // **本人阶段开始时**
//!     forced: true,
//!     filter(event, player) {
//!         return !player.hujia && player.countCards("h", "huanglinzhicong") > 0;  // **无护甲**且持有
//!     },
//!     content() { … player.changeHujia(); player.update(); },        // **+1 护甲**
//! },
//! ```
//!
//! **本样本的定位**：**无护甲的持有者阶段开始 ⇒ +1 护甲**（`forced`，无询问 ⇒ 可用常规推进助手）。
//!
//! 未覆盖（登记）：**已有护甲时不应触发**（`!player.hujia` 的负例）——建议下一轮补。

mod nc08_common;
use engine_native::{ChoiceOption, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, next_dedup, put_hand};

fn swd_engine(seed: u64) -> Engine {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/shahai-swd-upstream-noname/assets");
    let cards = std::fs::read_to_string(dir.join("cards.json")).expect("读 swd cards");
    let pack = content_native_loader::bridge_pack(&cards).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游 `g_huanglinzhicong`（**forced**）：无护甲的持有者阶段开始 ⇒ **+1 护甲**。
///
/// **本轮未通过（登记，含诊断）**：`hujia` **0 → 0**（应 **0 → 1**）⇒ **窗口未触发**。
///
/// **重要**：本文件的**负例（已有护甲 ⇒ 不变）也通过** ✗，但**窗口不触发时同样通过**
/// ⇒ **负例不是有效对照**（**通过的理由不成立**）。
///
/// **与第 55 项的共同点**：二者都依赖 **`phaseBegin`** 窗口；而 `qinglongzhigui` 的**摸 2 确实发生**
/// ⇒ **`phaseBegin` 本身会触发**，但**本卡的 `changeHujia` 分支未生效**——需与第 55 项一并排查
/// 「`phaseBegin` 机器的**分支选择**」与「`WaitChoice` 可见性」。
#[test]
fn upstream_huanglinzhicong_gains_hujia_on_phase_begin() {
    let mut e = swd_engine(20288901);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 0 号持黄麟之琮（无护甲）
    let _card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("huanglinzhicong"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("黄麟之琮在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    {
        let core = e.state_core_mut();
        if let Some(p) = core.players.get_mut(0) {
            p.hujia = 0; // 确保无护甲
        }
    }
    let hj_before = e.state_core().player(0).hujia;
    // **窗口在 `Judge`（准备）阶段**（上游 `phaseBegin` + 原生 `filtering/swd.rs:462` 要求 `Phase::Judge`）。
    //
    // ⚠ **第 55/56 项的更正经纬**：`Phase { begin: true }` 事件**只在回合起点的判定阶段发射**
    // （`kernel/events.rs:753/789`），而建局时**第 1 轮的判定开始已经发过** ⇒ 必须推进到**下一轮**
    // （`round >= 2`）的判定阶段。此前用例只推进 16–32 步、停在 `Play`/`End`，因此**看到 0→0**，
    // 被误判为「派发缺陷」。实测 **20 步**即可到达。
    let mut steps = 0;
    for _ in 0..260 {
        if e.turn().round >= 2 {
            break;
        }
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        steps += 1;
        while let Some(c) = e.open_choice().cloned() {
            let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else if c.spec.allow_cancel {
                Vec::new()
            } else {
                c.spec.options.iter().take(c.spec.max.max(1)).copied().collect()
            };
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer { choice_id: c.id, selection: sel },
            });
        }
    }
    let hj_after = e.state_core().player(0).hujia;
    eprintln!("DBG 黄麟之琮：推进 {steps} 步至 round {}", e.turn().round);
    eprintln!("DBG 黄麟之琮：护甲 {hj_before} → {hj_after}（应 +1）");
    assert_eq!(
        hj_after as i32 - hj_before as i32,
        1,
        "黄麟之琮（**forced**）：无护甲的持有者阶段开始应 **+1 护甲**（{hj_before} → {hj_after}）"
    );
}

/// **负例**（上游 `!player.hujia`）：**已有护甲**时**不应**再加。
///
/// ⚠ **本用例的通过不构成证据**：若窗口完全不触发，本用例也会通过 ⇒ 仅作**辅助**，
/// 有效性依赖「正例能触发」这一前提（当前**不成立**）。
#[test]
fn upstream_huanglinzhicong_no_gain_when_armored() {
    let mut e = swd_engine(20288902);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("huanglinzhicong"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("黄麟之琮在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    {
        let core = e.state_core_mut();
        if let Some(p) = core.players.get_mut(0) {
            p.hujia = 2; // **已有护甲** ⇒ 不应触发
        }
    }
    let hj_before = e.state_core().player(0).hujia;
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达 0 号出牌阶段");
    let hj_after = e.state_core().player(0).hujia;
    eprintln!("DBG 黄麟之琮（负例）：护甲 {hj_before} → {hj_after}（应不变）");
    assert_eq!(
        hj_after, hj_before,
        "黄麟之琮：**已有护甲**时不应触发（`!player.hujia` 条件；{hj_before} → {hj_after}）"
    );
}

/// **决定性探针**：停在 **`Judge`（准备）阶段**检查护甲——上游/原生 filter 都要求该阶段。
#[test]
fn probe_huanglin_at_judge_phase() {
    let mut e = swd_engine(20288903);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let id = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(_, c)| c.name.contains("huanglinzhicong"))
        .map(|(i, _)| i as engine_native::CardId)
        .expect("黄麟之琮在目录");
    nc08_common::put_hand(&mut e, 0, id);
    {
        let core = e.state_core_mut();
        if let Some(p) = core.players.get_mut(0) {
            p.hujia = 0;
        }
    }
    // 手动推进（不 drain），**一旦到 Judge 就停**
    for _ in 0..16 {
        if e.turn().current == 0 && e.turn().phase == Phase::Judge {
            break;
        }
        let actor = e.turn().current;
        let _ = e.submit(engine_native::GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: nc08_common::next_dedup(),
            action: engine_native::GameAction::NextPhase,
        });
        if e.open_choice().is_some() {
            break;
        }
    }
    let hj_judge = e.state_core().player(0).hujia;
    eprintln!(
        "DBG 黄麟之琮探针：当前阶段={:?}；护甲={hj_judge}（若在 Judge 且已触发应为 1）",
        e.turn().phase
    );
}

/// **决定性探针（只读）**：比对「放入手牌的该牌的 `catalog.card_kind`」与「期望的 `CardKind`」。
///
/// 背景：第 56 项的 filter 条件里唯一未验证的是 `hand_has_kind(ctx, owner, CardKind::Huanglinzhicong)`。
#[test]
fn probe_hand_has_kind_matches_catalog_kind() {
    let e = swd_engine(20288904);
    let id = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(_, c)| c.name.contains("huanglinzhicong"))
        .map(|(i, _)| i as engine_native::CardId)
        .expect("黄麟之琮在目录");
    let actual = e.catalog().card_kind(id);
    let expected = engine_native::CardKind::Huanglinzhicong;
    eprintln!(
        "DBG hand_has_kind 探针：目录名={}；card_kind={actual:?}；期望={expected:?}；相等={}",
        e.catalog().cards[id as usize].name,
        format!("{actual:?}") == format!("{expected:?}")
    );
    assert_eq!(
        format!("{actual:?}"),
        format!("{expected:?}"),
        "放入手牌的该牌的 catalog.card_kind 应等于 filter 中期望的 CardKind"
    );
}

/// **决定性只读探针（第 56 项剩余 2 项）**：
/// ①`core.skills` 中是否存在该行为的**实例**、其 `enabled` 为何；
/// ②`catalog.candidates(EventKind::PhaseBegin)` 是否**包含**该 `def_id`。
#[test]
fn probe_instance_and_candidates() {
    let e = swd_engine(20288905);
    // 找出「黄麟之琮手持技」的 def_id
    let def_id = e
        .catalog()
        .skills
        .iter()
        .position(|d| {
            matches!(
                d.behavior.as_host_behavior(),
                Some(engine_native::content::host::HostBehavior::SwdHuanglinzhicongHeld)
            )
        })
        .map(|i| i as engine_native::ids::SkillDefId);
    eprintln!("DBG 探针：黄麟之琮 def_id = {def_id:?}");
    let Some(def_id) = def_id else {
        eprintln!("NOTE 探针：目录中不存在该行为的 SkillDef——本探针不构成结论");
        return;
    };
    // ① core.skills 实例
    let mut instances = 0;
    let mut enabled_true = 0;
    for p in 0..4u8 {
        if let Some(inst) = e.state_core().skills.get(&(p, def_id)) {
            instances += 1;
            if inst.enabled {
                enabled_true += 1;
            }
        }
    }
    // ② candidates
    let cand = e
        .catalog()
        .candidates(engine_native::event::EventKind::PhaseBegin);
    let in_cand = cand.contains(&def_id);
    eprintln!(
        "DBG 探针：core.skills 实例 {instances}/4（enabled {enabled_true}）；\
         candidates(PhaseBegin) 共 {} 项，包含该 def = {in_cand}",
        cand.len()
    );
    assert!(
        instances > 0,
        "该行为的 SkillInstance 应存在于 core.skills（否则 plan_triggers 取不到）"
    );
    assert!(
        in_cand,
        "catalog.candidates(PhaseBegin) 应包含该 def（否则入队阶段就被跳过）"
    );
}

/// **决定性探针（关键更正）**：`Phase{begin:true}` 事件**只在回合起点的判定阶段发射**
/// （`kernel/events.rs:753/789`）。此前用例只推进 16–32 步，**从未到达下一轮判定阶段**，
/// 因此「窗口未触发」是**用例步数不足**，而非派发缺陷。
///
/// 本探针推进到 **round ≥ 2**（下一轮判定阶段），观察 `hujia` 是否 0 → 1。
#[test]
fn probe_huanglin_at_next_round_judge() {
    let mut e = swd_engine(20288906);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let card = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(_, c)| c.name.contains("huanglinzhicong"))
        .map(|(i, _)| i as engine_native::CardId)
        .expect("黄麟之琮在目录");
    put_hand(&mut e, 0, card);
    let before = e.state_core().player(0).hujia;
    let mut steps = 0;
    for _ in 0..260 {
        if e.turn().round >= 2 {
            break;
        }
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        steps += 1;
        while let Some(c) = e.open_choice().cloned() {
            let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else if c.spec.allow_cancel {
                Vec::new()
            } else {
                c.spec.options.iter().take(c.spec.max.max(1)).copied().collect()
            };
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer { choice_id: c.id, selection: sel },
            });
        }
    }
    let after = e.state_core().player(0).hujia;
    eprintln!(
        "DBG 黄麟之琮（下一轮判定）：步数 {steps}；回合 {} 阶段 {:?}；hujia {before} → {after}",
        e.turn().round,
        e.turn().phase
    );
    assert_eq!(
        after,
        before + 1,
        "到达下一轮判定阶段后，黄麟之琮应令 hujia +1（{before} → {after}）"
    );
}
