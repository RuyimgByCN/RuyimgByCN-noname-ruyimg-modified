//! swd deferred 消解批 132：魂殉志（`hxunzhi`）+ 殉（`hxunzhi2`）。
//!
//! 上游 `character/swd.js`：出牌阶段限定技——视为对全部其他角色使用
//! 【万箭齐发】并授予武圣/咆哮/殉；殉（`hxunzhi2`）在出牌阶段结束时
//! `player.die()`（直接死亡）。

use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn repo_extensions_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn swd_engine(seed: u64) -> Engine {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .expect("读 swd 包数据");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接 swd 包").pack;
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

/// 魂殉志：发动 → 万箭齐发（无闪者各受 1）→ 授予武圣/咆哮 → 出牌阶段末死亡。
#[test]
fn hxunzhi_wanjian_grants_and_death() {
    let mut e = swd_engine(109001);
    e.debug_grant(0, "hxunzhi");
    assert!(e.catalog().skill_by_name("hxunzhi").is_some());
    assert!(e.catalog().skill_by_name("hxunzhi2").is_some(), "殉在目录");
    // 推进到 0 号出牌阶段
    for _ in 0..12 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    assert_eq!(e.turn().current, 0);
    assert_eq!(e.turn().phase, Phase::Play);
    for p in 1..4 {
        e.debug_clear_hand(p);
    }
    let hps: Vec<i32> = (1..4).map(|p| e.debug_hp(p)).collect();
    let def = e.catalog().skill_by_name("hxunzhi").expect("魂殉志");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    let after: Vec<i32> = (1..4).map(|p| e.debug_hp(p)).collect();
    assert_eq!(
        hps.iter().zip(after.iter()).map(|(a, b)| a - b).sum::<i32>(),
        3,
        "万箭齐发：三名其他角色各受 1（均无闪）"
    );
    // 授予武圣/咆哮 + 殉
    for name in ["武圣", "咆哮", "hxunzhi2"] {
        let d = e.catalog().skill_by_name(name).expect(name);
        assert!(
            e.state_core().skills.contains_key(&(0, d)),
            "{name} 已授予"
        );
    }
    // 觉醒：自身实例禁用（再次发动被拒）
    let again = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(
        matches!(again, CommandOutcome::Rejected(_)),
        "限定技不可二次发动：{again:?}"
    );
    // 出牌阶段收束 → 死亡
    next_phase(&mut e);
    drain(&mut e);
    assert!(!e.debug_alive(0), "出牌阶段结束死亡（殉）");
    let d2 = e.catalog().skill_by_name("hxunzhi2").expect("殉");
    assert!(!e.state_core().skills.contains_key(&(0, d2)), "殉已移除");
}

/// 魂殉志（对照）：非出牌阶段不可发动；未持技不可发动。
#[test]
fn hxunzhi_phase_gate_and_absent() {
    let mut e = swd_engine(109002);
    e.debug_grant(0, "hxunzhi");
    for _ in 0..12 {
        if e.turn().current == 0 && e.turn().phase == Phase::Draw {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    assert_eq!(e.turn().phase, Phase::Draw);
    let def = e.catalog().skill_by_name("hxunzhi").expect("魂殉志");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(matches!(out, CommandOutcome::Rejected(_)), "非出牌阶段拒绝：{out:?}");
    // 未持技者：出牌阶段发动被拒
    let mut e2 = swd_engine(109003);
    for _ in 0..12 {
        if e2.turn().current == 0 && e2.turn().phase == Phase::Play {
            break;
        }
        next_phase(&mut e2);
        drain(&mut e2);
    }
    let out2 = e2.submit(GameCommand {
        actor: 0,
        expected_revision: e2.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(matches!(out2, CommandOutcome::Rejected(_)), "未持技拒绝：{out2:?}");
}

/// 魂殉志（对照）：无「殉」者出牌阶段收束不死亡。
#[test]
fn hxunzhi_without_marker_survives_phase_end() {
    let mut e = swd_engine(109004);
    for _ in 0..12 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    assert_eq!(e.turn().phase, Phase::Play);
    next_phase(&mut e);
    drain(&mut e);
    assert!(e.debug_alive(0), "无殉不死亡");
}
