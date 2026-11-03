//! U08 英雄杀批 2：门神（`yxs_menshen`）+ 门神·清除（`yxs_menshen3`）。
//!
//! 上游 `character/skill.js`：
//! - `yxs_menshen`：`trigger {player:'phaseEnd'}`（group `yxs_menshen3`）——
//!   >1 人时选一名其他角色 → `target.storage.yxs_menshen2 = player` +
//!   `target.addSkill('yxs_menshen2')`；
//! - `yxs_menshen3`：`trigger {player:['phaseBegin','dieBegin']}` silent——
//!   任一场上有 `yxs_menshen2` 时移除全场该技。

use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn repo_extensions_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn yxs_engine(seed: u64) -> Engine {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("hero-kill-upstream-noname/assets/cards.json"),
    )
    .expect("读英雄杀包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        let has_pass = c.spec.options.contains(&ChoiceOption::Pass);
        let sel: Vec<ChoiceOption> = if has_pass {
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
            action: GameAction::Answer { choice_id: c.id, selection: sel },
        });
    }
}

fn next_phase(e: &mut Engine) {
    let actor = e.turn().current;
    let out = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
}

/// 门神标记键（技能定义 u16）。
fn menshen2_def(e: &Engine) -> u16 {
    e.catalog().skill_by_name("yxs_menshen2").expect("yxs_menshen2")
}

/// 标记表键（u32）。
fn menshen2_mark_key(e: &Engine) -> u32 {
    u32::from(menshen2_def(e))
}

/// 门神：0 号结束阶段选 1 号 → 1 号获得门神标记（实例 + 标记=0）。
#[test]
fn menshen_marks_target() {
    let mut e = yxs_engine(158001);
    e.debug_grant(0, "yxs_menshen");
    e.debug_grant(0, "yxs_menshen3");
    assert!(e.catalog().skill_by_name("yxs_menshen").is_some());
    // 推进到 0 号结束阶段（Play → Discard → 回合结束即 PhaseEnd）
    for _ in 0..16 {
        if e.turn().current == 0 && e.turn().phase == Phase::Discard {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    assert_eq!(e.turn().phase, Phase::Discard);
    next_phase(&mut e); // Discard 收束 → 结束阶段
    drain(&mut e);
    // A2 H51：上游 player phaseEnd＝整回合结束——门神在整回合结束窗触发
    //（此后不 drain：确认门与目标窗由下方循环显式应答）
    next_phase(&mut e); // End 收束 → 整回合结束窗（门神触发）
    // 门神确认门 → 目标窗
    let mut picked = false;
    for _ in 0..12 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "trigger_confirm" {
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Confirm],
                },
            });
            continue;
        }
        if c.spec.prompt == "yxs_menshen_target" {
            let out = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Player(1)],
                },
            });
            assert!(
                matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
                "{out:?}"
            );
            picked = true;
            continue;
        }
        let has_pass = c.spec.options.contains(&ChoiceOption::Pass);
        let sel: Vec<ChoiceOption> = if has_pass {
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
            action: GameAction::Answer { choice_id: c.id, selection: sel },
        });
    }
    drain(&mut e);
    assert!(picked, "门神目标窗出现");
    let def = menshen2_def(&e);
    assert!(
        e.state_core().skills.contains_key(&(1, def)),
        "1 号获得门神实例"
    );
    assert_eq!(
        e.state_core().vm_aux.marks.get(&(1, menshen2_mark_key(&e))).copied(),
        Some(0),
        "门神标记 = 施予者座位（0）"
    );
}

/// 门神·清除：下个阶段开始时移除全场门神标记。
#[test]
fn menshen3_clears_marker() {
    let mut e = yxs_engine(158002);
    e.debug_grant(0, "yxs_menshen");
    e.debug_grant(0, "yxs_menshen3");
    // 直接手工置标记（模拟已施予），再推进阶段触发清除
    let def = menshen2_def(&e);
    {
        let core = e.state_core_mut();
        core.skills.insert(
            (1, def),
            engine_native::state::SkillInstance {
                owner: 1,
                enabled: true,
                once_used: false,
                expires_end_of_turn: false,
                rounds: 0,
            },
        );
        core.vm_aux.marks.insert((1, u32::from(def)), 0);
    }
    // A2 H06：上游 player phaseBegin＝持有者自己回合开始——推进到 0 号
    // 下一回合开始（约三轮）触发清除；中途他人回合/阶段不再清（原任意
    // 阶段开始当回合即清）。
    for _ in 0..48 {
        next_phase(&mut e);
        drain(&mut e);
        if !e.state_core().skills.contains_key(&(1, def)) {
            break;
        }
    }
    assert!(
        !e.state_core().skills.contains_key(&(1, def)),
        "门神实例已被清除"
    );
    assert!(
        e.state_core().vm_aux.marks.get(&(1, u32::from(def))).is_none(),
        "门神标记已被清除"
    );
}
