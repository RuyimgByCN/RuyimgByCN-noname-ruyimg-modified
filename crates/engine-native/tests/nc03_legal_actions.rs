//! NC03 权威动作查询：合法动作只由内核定义（客户端/AI 不再镜像规则）。
//!
//! 审计反例（`adapter-probe.log`）与源码依据：前端按 `hand.slice(0, 8)` 拼接
//! `PlayCard`、`ai.rs::enumerate_candidates_variant` 注释即「非法由模拟拒绝」——
//! 规则合法性被复制到内核之外。本文件验收 `Engine::legal_actions` /
//! `is_legal_action` 的权威性：
//! 1. 射程外目标不出现（内核距离/射程校验为准）；
//! 2. 合法杀目标逐个出现；
//! 3. 非本人回合无出牌候选；
//! 4. 开放选择时只给观察者应答候选（单选逐项 + 取消）；
//! 5. 查询不改变引擎状态（修订号/摘要不变）。

use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel: Vec<engine_native::ChoiceOption> =
            if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
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
}

fn advance_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..20 {
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

/// 射程外目标不出现在候选；相邻目标出现（内核校验为准）。
#[test]
fn legal_actions_respect_attack_range() {
    let mut e = Engine::new(20261019, 5);
    advance_to_play(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);

    let actions = e.legal_actions(0);
    let sha_targets: Vec<Vec<u8>> = actions
        .iter()
        .filter_map(|a| match a {
            GameAction::PlayCard { card, targets } if *card == sha => Some(targets.clone()),
            _ => None,
        })
        .collect();
    assert!(!sha_targets.is_empty(), "持【杀】应有出牌候选");
    // 5 人局：0 号相邻为 1、4；2、3 距离 2 超出射程 1
    for t in &sha_targets {
        if t.len() == 1 {
            assert!(
                t[0] == 1 || t[0] == 4,
                "单目标候选须在射程内（实际 {t:?}）"
            );
        }
    }
    assert!(
        !sha_targets.contains(&vec![2]) && !sha_targets.contains(&vec![3]),
        "射程外目标不得出现"
    );
    assert!(sha_targets.contains(&vec![1]), "相邻目标应出现");
    // 与内核校验一致：射程外提交被拒
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![2] },
    });
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "射程外提交应被内核拒绝: {out:?}"
    );
}

/// 查询是只读的：修订号与摘要不变。
#[test]
fn legal_actions_do_not_mutate_engine() {
    let mut e = Engine::new(20261020, 4);
    advance_to_play(&mut e, 0);
    let rev = e.revision();
    let hash = e.state_hash();
    let actions = e.legal_actions(0);
    assert!(!actions.is_empty(), "出牌阶段应有候选（至少 NextPhase）");
    assert_eq!(e.revision(), rev, "修订号不变");
    assert_eq!(e.state_hash(), hash, "摘要不变");
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, GameAction::NextPhase)),
        "阶段推进候选在列"
    );
}

/// 非本人回合：无出牌候选（只可能有阶段推进/应答）。
#[test]
fn legal_actions_empty_for_other_seat() {
    let mut e = Engine::new(20261021, 4);
    advance_to_play(&mut e, 0);
    let actions = e.legal_actions(1);
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a, GameAction::PlayCard { .. })),
        "非本人回合不得给出出牌候选: {actions:?}"
    );
}

/// 开放选择：只给观察者应答候选，且每项都能被内核接受。
#[test]
fn legal_actions_cover_open_choice() {
    let mut e = Engine::new(20261022, 4);
    // 推进到出现开放选择（弃牌阶段的手牌上限询问）
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
    let actions = e.legal_actions(observer);
    assert!(!actions.is_empty(), "观察者应有应答候选");
    assert!(
        actions.iter().all(|a| matches!(a, GameAction::Answer { .. })),
        "开放选择期间只给应答候选: {actions:?}"
    );
    // 每个候选都能被内核接受（权威性：查询与提交同源）
    for a in actions.iter().take(3) {
        assert!(
            e.is_legal_action(observer, a),
            "候选应通过内核校验: {a:?}"
        );
    }
    // 非观察者无候选
    let other = (0..4u8).find(|p| *p != observer).expect("其他座位");
    assert!(e.legal_actions(other).is_empty(), "非观察者无应答候选");
}
