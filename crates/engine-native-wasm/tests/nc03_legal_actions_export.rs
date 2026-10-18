//! NC03 前端接线：窄壳「权威动作查询」导出面（`legal_actions_json`）定向验收。
//!
//! 修复前：前端自己拼候选（`hand.slice(0, 8)` 出牌 + 合成一个 `ResolveChoice`），
//! 合法性定义散落在内核之外。本文件验收窄壳导出面把**内核候选**交给客户端：
//!
//! 1. 导出面存在且可调用（编译期取 `NativeEngine::legal_actions_json` 函数指针）；
//! 2. 返回非空且 schema 与 TS `LegalAction` 同形（`{seat, revision, actions}`）；
//! 3. **逐个候选可在内核上真实提交**（Accepted/Pending，绝不 Rejected）——
//!    即「前端拿到的动作都是内核认可的」；
//! 4. 候选来自内核而非视图拼接：12 张手牌（> 8）不被截断，不可用的手牌不出现在候选；
//! 5. 反向探针：伪造卡 id／越权座位被内核拒绝，座位越界查询报错（校验非恒真）。

use std::sync::atomic::{AtomicU64, Ordering};

use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use engine_native_wasm::{legal_actions_json_for, NativeEngine};
use serde_json::Value;
use wasm_bindgen::JsValue;

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

/// 提交阶段推进直到 `seat` 进入出牌阶段（与内核 NC03 测试同款驱动）。
fn advance_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..40 {
        if e.turn().current == seat && e.turn().phase == Phase::Play {
            return;
        }
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        drain(e);
    }
    panic!("未到 {seat} 号出牌阶段");
}

/// 自动应答开放选择（避免驱动时卡在选择上）。
fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
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
}

/// 解析窄壳动作 JSON 并**反向重建**内核动作（独立于投影实现，验证信息完备）。
fn parse_action(engine: &Engine, v: &Value) -> GameAction {
    if let Some(name) = v.as_str() {
        assert_eq!(name, "EndPhase", "未知字符串动作: {v}");
        return GameAction::NextPhase;
    }
    let obj = v.as_object().unwrap_or_else(|| panic!("动作须是字符串或对象: {v}"));
    assert_eq!(obj.len(), 1, "动作对象须恰好一个键: {v}");
    if let Some(play) = obj.get("PlayCard") {
        let card = play["card"].as_u64().expect("card 是数字") as u32;
        let targets = play["targets"]
            .as_array()
            .expect("targets 是数组")
            .iter()
            .map(|t| t.as_u64().expect("目标座位是数字") as u8)
            .collect();
        return GameAction::PlayCard { card, targets };
    }
    if let Some(act) = obj.get("ActivateSkill") {
        assert!(
            act["targets"].as_array().expect("targets").is_empty(),
            "主动技候选不应带目标（目标经开放选择应答）: {v}"
        );
        let skill = act["skill"].as_u64().expect("skill 是数字") as u16;
        return GameAction::ActivateSkill { skill };
    }
    if let Some(rc) = obj.get("ResolveChoice") {
        let choice_id = rc["choice_id"].as_u64().expect("choice_id 是数字");
        let open = engine.open_choice().expect("应答候选须对应开放选择");
        assert_eq!(open.id, choice_id, "候选必须指向当前开放选择: {v}");
        let spec = open.spec.clone();
        let idxs: Vec<u64> = rc["selections"]
            .as_array()
            .expect("selections 是数组")
            .iter()
            .map(|i| i.as_u64().expect("下标是数字"))
            .collect();
        let selection: Vec<ChoiceOption> = idxs
            .iter()
            .map(|i| {
                *spec
                    .options
                    .get(*i as usize)
                    .unwrap_or_else(|| panic!("下标 {i} 越界: {v}"))
            })
            .collect();
        assert_eq!(
            rc["cancelled"].as_bool().expect("cancelled 是布尔"),
            selection.is_empty(),
            "cancelled 与空选择同义: {v}"
        );
        return GameAction::Answer {
            choice_id,
            selection,
        };
    }
    panic!("未知动作形态: {v}");
}

/// 取权威查询的 actions 数组（顺带核对信封字段）。
fn query(engine: &Engine, seat: u8) -> Vec<Value> {
    let raw = legal_actions_json_for(engine, seat).expect("权威查询应成功");
    let v: Value = serde_json::from_str(&raw).expect("查询结果是 JSON");
    assert_eq!(v["seat"].as_u64(), Some(seat as u64), "seat 回显: {raw}");
    assert_eq!(
        v["revision"].as_u64(),
        Some(engine.revision()),
        "revision 是该候选成立的状态修订号: {raw}"
    );
    v["actions"].as_array().expect("actions 是数组").clone()
}

/// 在克隆上**真实提交**候选：不接受 Rejected/Suspended。
fn assert_submittable(engine: &Engine, seat: u8, action: &GameAction) {
    let mut probe = engine.clone();
    let out = probe.submit(GameCommand {
        actor: seat,
        expected_revision: engine.revision(),
        dedup_id: 1_000_000,
        action: action.clone(),
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "候选必须能被内核接受（{action:?} → {out:?}）"
    );
}

#[test]
fn export_surface_exists_and_returns_kernel_actions() {
    // 编译期：`legal_actions_json` 必须是 `NativeEngine` 的公开导出方法。
    let _export: fn(&NativeEngine, u8) -> Result<String, JsValue> =
        NativeEngine::legal_actions_json;
    let _skill_export: fn(&mut NativeEngine, u32, u16) -> Result<String, JsValue> =
        NativeEngine::activate_skill;

    let mut e = Engine::new(20261019, 5);
    advance_to_play(&mut e, 0);
    let actions = query(&e, 0);
    assert!(!actions.is_empty(), "出牌阶段候选不得为空");
    assert!(
        actions.iter().any(|a| a.as_str() == Some("EndPhase")),
        "阶段推进候选在列: {actions:?}"
    );
    assert!(
        actions.iter().any(|a| a.get("PlayCard").is_some()),
        "手牌可用时应有出牌候选: {actions:?}"
    );

    // 座位越界必须报错，而不是给别的座位的候选。
    assert!(legal_actions_json_for(&e, 9).is_err(), "座位越界应报错");
}

#[test]
fn every_projected_action_is_submittable() {
    let mut e = Engine::new(20261020, 5);
    advance_to_play(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);

    let actions = query(&e, 0);
    assert!(actions.len() >= 2, "至少出牌 + 阶段推进: {actions:?}");
    for action in &actions {
        let rebuilt = parse_action(&e, action);
        assert!(
            e.is_legal_action(0, &rebuilt),
            "候选须通过内核校验: {action}"
        );
        assert_submittable(&e, 0, &rebuilt);
    }
    // 射程：5 人局 0 号相邻 1/4；射程外目标不出现（与内核拒绝一致）。
    let targets: Vec<Vec<u64>> = actions
        .iter()
        .filter_map(|a| a.get("PlayCard"))
        .filter(|p| p["card"].as_u64() == Some(sha as u64))
        .map(|p| {
            p["targets"]
                .as_array()
                .unwrap()
                .iter()
                .map(|t| t.as_u64().unwrap())
                .collect()
        })
        .collect();
    assert!(targets.contains(&vec![1]), "相邻目标在列: {targets:?}");
    assert!(
        !targets.contains(&vec![2]) && !targets.contains(&vec![3]),
        "射程外目标不得出现: {targets:?}"
    );
    assert!(
        !targets.iter().any(|t| t.len() == 1 && t[0] == 0),
        "不能以自己为目标: {targets:?}"
    );

    // 与正式提交一致：射程外提交被拒（伪造动作不会因投影而变合法）。
    let mut probe = e.clone();
    let out = probe.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: 1_000_001,
        action: GameAction::PlayCard {
            card: sha,
            targets: vec![2],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "射程外目标应被内核拒绝: {out:?}"
    );
}

#[test]
fn candidates_are_not_hand_padding_nor_truncated_at_eight() {
    let mut e = Engine::new(20261021, 5);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);

    // 9 张【杀】（出牌阶段可用、需目标）：旧前端 `hand.slice(0, 8)` 只覆盖前
    // 8 张手牌；权威查询必须让第 9 张也出现（每张至少有相邻目标候选）。
    let mut ids = Vec::new();
    for _ in 0..9 {
        ids.push(e.debug_place(0, "sha"));
    }
    assert_eq!(e.debug_hand(0).len(), 9, "布阵应给到 9 张手牌");
    let actions = query(&e, 0);
    let played: Vec<u64> = actions
        .iter()
        .filter_map(|a| a.get("PlayCard"))
        .map(|p| p["card"].as_u64().unwrap())
        .collect();
    let distinct: std::collections::BTreeSet<u64> = played.iter().copied().collect();
    assert_eq!(distinct.len(), 9, "9 张可用手牌都应有候选（>8，不得截断）: {played:?}");
    for (i, id) in ids.iter().enumerate() {
        assert!(
            played.contains(&(*id as u64)),
            "第 {} 张【杀】（id {id}）必须出现在候选里: {played:?}",
            i + 1
        );
    }

    // 反向：手里只有不可主动使用的牌（自己回合的【闪】）时，视图有 12 张手牌，
    // 但内核候选里**没有**出牌项——候选来自内核校验，不是按手牌拼接。
    let mut e2 = Engine::new(20261022, 5);
    advance_to_play(&mut e2, 0);
    e2.debug_clear_hand(0);
    for _ in 0..12 {
        e2.debug_place(0, "shan");
    }
    assert_eq!(e2.debug_hand(0).len(), 12, "布阵应给到 12 张手牌");
    let actions = query(&e2, 0);
    assert!(
        !actions.iter().any(|a| a.get("PlayCard").is_some()),
        "自己回合的【闪】不可主动使用，候选不得出现出牌项: {actions:?}"
    );
    assert!(
        actions.iter().any(|a| a.as_str() == Some("EndPhase")),
        "阶段推进仍应可提交: {actions:?}"
    );
}

#[test]
fn open_choice_candidates_round_trip_option_indices() {
    let mut e = Engine::new(20261023, 4);
    // 推进到出现开放选择（弃牌阶段的手牌上限询问）。
    for _ in 0..30 {
        if e.open_choice().is_some() {
            break;
        }
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
    }
    let choice = e.open_choice().cloned().expect("应有开放选择");
    let observer = choice.observer;
    let actions = query(&e, observer);
    assert!(!actions.is_empty(), "观察者应有应答候选");
    assert!(
        actions
            .iter()
            .all(|a| a.get("ResolveChoice").is_some()),
        "开放选择期间只给应答候选: {actions:?}"
    );
    // 每个候选都能反向重建（下标 → 候选值）并被内核接受。
    for action in &actions {
        let rebuilt = parse_action(&e, action);
        assert_submittable(&e, observer, &rebuilt);
    }
    // 非观察者没有任何候选（旧前端仍会合成一个）。
    let other = (0..4u8).find(|p| *p != observer).expect("其他座位");
    assert!(query(&e, other).is_empty(), "非观察者无候选");
}

#[test]
fn forged_actions_are_rejected_by_kernel() {
    let mut e = Engine::new(20261024, 5);
    advance_to_play(&mut e, 0);

    // 伪造卡 id：不在手牌 → 内核拒绝（权威校验非恒真）。
    assert!(
        !e.is_legal_action(
            0,
            &GameAction::PlayCard {
                card: 9999,
                targets: vec![],
            }
        ),
        "伪造卡 id 不得通过权威校验"
    );
    // 伪造座位：越权推进阶段 → 拒绝。
    assert!(
        !e.is_legal_action(9, &GameAction::NextPhase),
        "越权座位不得通过权威校验"
    );
    // 投影面的候选里也不含伪造值。
    let actions = query(&e, 0);
    assert!(
        !actions.iter().any(|a| {
            a.get("PlayCard")
                .map(|p| p["card"].as_u64() == Some(9999))
                .unwrap_or(false)
        }),
        "候选不得包含伪造卡 id: {actions:?}"
    );

    // 命令面同样拒绝，且拒绝原因可见（窄壳 outcome_json 的 reason）。
    let mut probe = e.clone();
    let out = probe.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: 1_000_002,
        action: GameAction::PlayCard {
            card: 9999,
            targets: vec![],
        },
    });
    match out {
        CommandOutcome::Rejected(reason) => {
            assert_eq!(format!("{reason:?}"), "CardNotInHand");
        }
        other => panic!("伪造卡 id 应被拒绝: {other:?}"),
    }
}
