//! T11 Boss 庞统族批 118：涅槃（`niepan`）+ 浴火（`boss_yuhuo`）+ 神威
//! （`shenwei`）。
//!
//! 上游：`character/shenhua.js:5592`（涅槃）、`mode/boss.js:8530`（浴火）、
//! `mode/boss.js:8779`（神威）；Boss 条目 `boss_pangtong:['male','shen',
//! 4,['boss_tianyu','qiwu','niepan','boss_yuhuo'],...]`（`mode/boss.js:1305`）。
//! 原生目录：`BOSS_GENERALS` 庞统条目（天狱/栖梧/涅槃/浴火；神威由浴火授予）。

use engine_native::event::DamageNature;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn boss_engine(seed: u64) -> Engine {
    Engine::new_boss_stage(seed, 4, 1, 3, 6, 4, 0)
}

fn drain(e: &mut Engine) {
    for _ in 0..48 {
        let Some(c) = e.open_choice().cloned() else { break };
        let has_pass = c.spec.options.contains(&engine_native::ChoiceOption::Pass);
        let sel: Vec<engine_native::ChoiceOption> = if has_pass {
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
    drain(e);
}

/// 庞统（Boss 关卡）装配：主将槽 + 目录四技实例。
fn assemble_pangtong(e: &mut Engine) {
    let def = engine_native::content::standard::BOSS_GENERALS
        .iter()
        .position(|(n, ..)| *n == "庞统")
        .expect("庞统在 Boss 目录") as u16;
    let idx = engine_native::content::standard::STANDARD_GENERALS.len() as u16 + def;
    e.debug_set_general_def(0, idx);
    for name in ["天狱", "栖梧", "涅槃", "浴火"] {
        e.debug_grant(0, name);
    }
    e.debug_set_max_hp(0, 4);
    e.debug_set_hp(0, 4);
}

/// 涅槃（出牌阶段面）：弃全部牌（手/装/判）+ 解横置 + 翻回正面 + 摸 3 +
/// 回复至 3；限定一次；发动后触发浴火（授予神威 + 关卡循环型 1）。
#[test]
fn niepan_play_phase_effect_and_yuhuo_chain() {
    let mut e = boss_engine(95001);
    assemble_pangtong(&mut e);
    // 推进到 0 号出牌阶段（摸牌阶段已过），再布阵
    for _ in 0..8 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play {
            break;
        }
        next_phase(&mut e);
    }
    assert_eq!(e.turn().current, 0);
    assert_eq!(e.turn().phase, Phase::Play);
    // 布阵：手牌 2、装备 1、判定区 1、已横置、已翻面、体力 1
    e.debug_clear_hand(0);
    let c1 = e.debug_place(0, "sha");
    let c2 = e.debug_place(0, "shan");
    e.debug_give_hand_card(0, 0, c1);
    e.debug_give_hand_card(0, 0, c2);
    let equip = e.debug_place(0, "armor");
    {
        let core = e.state_core_mut();
        core.players[0].hand.retain(|c| *c != equip);
        core.players[0].equips.push(equip);
    }
    e.debug_place_judge(0, "le");
    e.debug_set_hp(0, 1);
    e.debug_set_face_down(0, true);
    e.state_core_mut().players[0].linked = true;
    assert!(e.state_core().player(0).face_down, "已翻面");
    assert!(e.state_core().player(0).linked, "已横置");
    let hand_before = e.debug_hand(0).len();
    assert_eq!(hand_before, 2, "手牌 2");
    assert_eq!(e.debug_equips(0).len(), 1, "装备 1");
    assert_eq!(e.debug_judge(0).len(), 1, "判定区 1");

    let niepan = e.catalog().skill_by_name("涅槃").expect("涅槃在目录");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: niepan },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.debug_hand(0).len(), 3, "弃全部牌后摸 3");
    assert_eq!(e.debug_equips(0).len(), 0, "装备全弃");
    assert_eq!(e.debug_judge(0).len(), 0, "判定区全弃");
    assert_eq!(e.debug_hp(0), 3, "回复至 3");
    assert!(!e.state_core().player(0).linked, "解除横置");
    assert!(!e.state_core().player(0).face_down, "翻回正面");
    assert!(e.state_core().player(0).niepan_used, "限定技已发动");
    // 浴火：授予神威 + 关卡循环型 1
    let sw = e.catalog().skill_by_name("神威").expect("神威在目录");
    assert!(
        e.state_core().skills.contains_key(&(0, sw)),
        "浴火授予神威"
    );
    assert_eq!(
        e.state_core().boss_stage().map(|s| s.loop_type),
        Some(1),
        "浴火置关卡循环型 1"
    );
    // 限定一次：再次发动被拒
    let again = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: niepan },
    });
    assert!(
        matches!(again, CommandOutcome::Rejected(_)),
        "限定技不可二次发动：{again:?}"
    );
}

/// 涅槃（濒死面）：濒死时可选发动自救（回复至 3），放弃则正常进入救援。
#[test]
fn niepan_dying_save_or_decline() {
    // 路径 A：濒死时发动涅槃自救
    let mut e = boss_engine(95002);
    assemble_pangtong(&mut e);
    e.debug_set_hp(0, 1);
    let out = e.debug_deal_damage(Some(1), 0, 2, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let c = e.open_choice().cloned().expect("濒死选择窗");
    assert_eq!(c.spec.prompt, "trigger_confirm", "涅槃可选触发确认门");
    let out = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: c.id,
            selection: vec![engine_native::ChoiceOption::Confirm],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(0), 3, "涅槃回复至 3");
    assert!(e.debug_alive(0), "自救成功");
    assert!(e.state_core().player(0).niepan_used);

    // 路径 B：放弃涅槃 → 正常救援（无桃则阵亡）
    let mut e = boss_engine(95003);
    assemble_pangtong(&mut e);
    e.debug_clear_hand(0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(0, 1);
    let out = e.debug_deal_damage(Some(1), 0, 2, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let c = e.open_choice().cloned().expect("濒死选择窗");
    let _ = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer { choice_id: c.id, selection: Vec::new() },
    });
    drain(&mut e);
    assert!(!e.debug_alive(0), "放弃涅槃且无救援 → 阵亡");
    assert!(!e.state_core().player(0).niepan_used, "未发动");
}

/// 神威（浴火授予）：摸牌阶段多摸 min(3, 其他角色数)、手牌上限同量加成。
#[test]
fn shenwei_draw_and_hand_limit_bonus() {
    let mut e = boss_engine(95004);
    assemble_pangtong(&mut e);
    e.debug_grant(0, "神威");
    let sw = e.catalog().skill_by_name("神威").expect("神威");
    assert!(e.state_core().skills.contains_key(&(0, sw)));
    // 手牌上限：体力 4 + min(3, 其他 3) = 7
    e.debug_set_hp(0, 4);
    assert_eq!(
        e.state_core().hand_limit(e.catalog(), 0),
        7,
        "手牌上限 +3"
    );
    // 摸牌阶段：0 号摸 2 + 3 = 5
    e.debug_clear_hand(0);
    for _ in 0..8 {
        if e.turn().current == 0 && e.turn().phase == Phase::Draw {
            break;
        }
        next_phase(&mut e);
    }
    assert_eq!(e.turn().current, 0);
    assert_eq!(e.turn().phase, Phase::Draw);
    next_phase(&mut e);
    assert_eq!(e.debug_hand(0).len(), 5, "摸牌 2+3=5");
    // 对照：无神威者摸 2
    let mut e2 = boss_engine(95005);
    assemble_pangtong(&mut e2);
    e2.debug_clear_hand(0);
    for _ in 0..8 {
        if e2.turn().current == 0 && e2.turn().phase == Phase::Draw {
            break;
        }
        next_phase(&mut e2);
    }
    next_phase(&mut e2);
    assert_eq!(e2.debug_hand(0).len(), 2, "对照：无神威摸 2");
}

/// 存档往返：涅槃限定位与关卡循环型保持。
#[test]
fn niepan_state_roundtrip() {
    let mut e = boss_engine(95006);
    assemble_pangtong(&mut e);
    e.debug_set_hp(0, 4);
    e.debug_grant(0, "神威");
    for _ in 0..8 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play {
            break;
        }
        next_phase(&mut e);
    }
    let niepan = e.catalog().skill_by_name("涅槃").expect("涅槃在目录");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: niepan },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    let snap = e.save();
    assert!(snap.core.players[0].niepan_used);
    assert_eq!(
        match &snap.core.mode {
            engine_native::state::GameMode::Boss { stage, .. } => Some(stage.loop_type),
            _ => None,
        },
        Some(1)
    );
    let restored = Engine::restore(&snap).expect("恢复");
    assert!(restored.state_core().player(0).niepan_used);
    assert_eq!(
        restored.state_core().boss_stage().map(|s| s.loop_type),
        Some(1)
    );
}
