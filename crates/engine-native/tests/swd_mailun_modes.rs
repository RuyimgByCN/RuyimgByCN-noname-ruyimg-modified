//! swd deferred 消解批 137：脉轮（`mailun`）——回合开始 8 选 1 效果。
//!
//! 上游 `character/swd.js`：`trigger {player:'phaseBegin'}`，`direct:true`——
//! 每回合开始（A2 H12：phaseBegin＝TurnBegin 窗，每回合一次；原锚任意
//! 自己阶段开始＝每回合 5→6 次）从 8 种效果中选一种（旧效果清除）：
//! 1 换体力上限、2 反向、3 首次造成/受到伤害 -1、4 首次伤害 +1、
//! 5 少摸一张且上限 +1、6 多摸一张且上限 -1、7 双向距离 -1、8 双向距离 +1。

use engine_native::event::DamageNature;
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


/// A2 H12：前进到 seat 下一回合开始（TurnBegin 窗挂起，脉轮确认窗/
/// 选择窗留给调用方；到达时不 drain）。
fn advance_to_turn_begin(e: &mut Engine, seat: u8) {
    while e.turn().current == seat {
        drain(e);
        next_phase(e);
    }
    for _ in 0..60 {
        if e.turn().current == seat && e.turn().phase == Phase::Judge {
            return;
        }
        drain(e);
        next_phase(e);
    }
    panic!("60 步内未到达 {seat} 号下一回合开始");
}

/// 打开脉轮窗并读取候选（先过确认门；不 drain 触发窗）。
fn mailun_options(e: &mut Engine) -> Option<Vec<ChoiceOption>> {
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else {
            next_phase(e);
            continue;
        };
        if c.spec.prompt == "mailun_mode" {
            return Some(c.spec.options.clone());
        }
        let sel: Vec<ChoiceOption> = if c.spec.prompt == "trigger_confirm" {
            vec![ChoiceOption::Confirm]
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
    None
}

/// 打开脉轮窗并选择指定效果（先过确认门）；返回是否成功。
fn pick_mailun(e: &mut Engine, mode: u8) -> bool {
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else {
            next_phase(e);
            continue;
        };
        if c.spec.prompt == "mailun_mode" {
            let out = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Tag(mode)],
                },
            });
            assert!(
                matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
                "{out:?}"
            );
            drain(e);
            return true;
        }
        let sel: Vec<ChoiceOption> = if c.spec.prompt == "trigger_confirm" {
            vec![ChoiceOption::Confirm]
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
    false
}

/// 选效果 5：摸牌阶段少摸一张（摸 1）且手牌上限 +1。
#[test]
fn mailun_mode5_draw_minus_and_limit_plus() {
    let mut e = swd_engine(114001);
    e.debug_grant(0, "mailun");
    assert!(e.catalog().skill_by_name("mailun").is_some());
    // A2 H12：推进到 0 号下一回合开始（TurnBegin 窗触发脉轮——确认后
    // 选效果 5），随后推进到本回合摸牌阶段验证少摸
    advance_to_turn_begin(&mut e, 0);
    let opts = mailun_options(&mut e).expect("脉轮选择窗");
    assert_eq!(opts.len(), 8, "8 种效果");
    assert!(pick_mailun(&mut e, 5), "选效果 5");
    assert_eq!(e.state_core().player(0).mailun_mode, 5);
    let limit = e.state_core().hand_limit(e.catalog(), 0);
    assert_eq!(limit, e.debug_hp(0).max(0) as usize + 1, "手牌上限 +1");
    // 记录手牌，推进到 0 号本回合的摸牌阶段（阶段开始即摸牌）
    let before = e.debug_hand(0).len();
    for _ in 0..6 {
        if e.turn().current == 0 && e.turn().phase == Phase::Draw {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    assert_eq!(e.turn().phase, Phase::Draw);
    assert_eq!(e.debug_hand(0).len(), before + 1, "少摸一张：摸 1");
}

/// 选效果 3：首次造成伤害 -1（一次性）；重选后清除。
#[test]
fn mailun_mode3_first_damage_minus() {
    // A2 H12：回合开始（TurnBegin 窗）选效果。同种子对照局分离脉轮
    // 的 -1（第二轮起其他随机将的修正可能叠加，断言差值而非绝对值）。
    let mut e = swd_engine(114002);
    e.debug_grant(0, "mailun");
    advance_to_turn_begin(&mut e, 0);
    assert!(pick_mailun(&mut e, 3), "选效果 3");
    assert_eq!(e.state_core().player(0).mailun_mode, 3);
    let hp2 = e.debug_hp(2);
    let out = e.debug_deal_damage(Some(0), 2, 3, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    let with_mode = hp2 - e.debug_hp(2);
    assert!(e.state_core().player(0).mailun_out_used, "一次性已用");
    // 对照局：同种子、同推进，回合开始**放弃**脉轮（不置模式）
    let mut e2 = swd_engine(114002);
    e2.debug_grant(0, "mailun");
    advance_to_turn_begin(&mut e2, 0);
    drain(&mut e2); // 取消确认门＝放弃
    assert_eq!(e2.state_core().player(0).mailun_mode, 0, "对照局未置模式");
    let hp2b = e2.debug_hp(2);
    let out = e2.debug_deal_damage(Some(0), 2, 3, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e2);
    let without_mode = hp2b - e2.debug_hp(2);
    assert_eq!(with_mode, without_mode - 1, "首次造成伤害 -1（仅脉轮贡献）");
    // 第二次伤害不再减免（脉轮一次性已用）
    let hp2c = e.debug_hp(2);
    let out = e.debug_deal_damage(Some(0), 2, 1, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    let second = hp2c - e.debug_hp(2);
    let hp2d = e2.debug_hp(2);
    let out = e2.debug_deal_damage(Some(0), 2, 1, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e2);
    let second_ctrl = hp2d - e2.debug_hp(2);
    assert_eq!(second, second_ctrl, "第二次不再减免（差值归零）");
}

/// 选效果 7：双向距离 -1（攻击范围与受击距离同变）。
#[test]
fn mailun_mode7_distance_minus() {
    let mut e = swd_engine(114003);
    e.debug_grant(0, "mailun");
    let base_to1 = e.state_core().computed_distance(e.catalog(), 0, 1);
    let base_from1 = e.state_core().computed_distance(e.catalog(), 1, 0);
    // A2 H12：回合开始（TurnBegin 窗）选效果
    advance_to_turn_begin(&mut e, 0);
    assert!(pick_mailun(&mut e, 7), "选效果 7");
    assert_eq!(e.state_core().player(0).mailun_mode, 7);
    let d01 = e.state_core().computed_distance(e.catalog(), 0, 1);
    let d10 = e.state_core().computed_distance(e.catalog(), 1, 0);
    assert_eq!(d01, (base_to1 as i32 - 1).max(1) as u32, "出向距离 -1");
    assert_eq!(d10, (base_from1 as i32 - 1).max(1) as u32, "受向距离 -1");
}

/// 选效果 1：失去 1 体力、上限 +1（maxHp==6 时不可选）。
#[test]
fn mailun_mode1_hp_shift_and_gate() {
    let mut e = swd_engine(114004);
    e.debug_grant(0, "mailun");
    // A2 H12：回合开始（TurnBegin 窗）选效果
    advance_to_turn_begin(&mut e, 0);
    let opts = mailun_options(&mut e).expect("脉轮选择窗");
    assert!(
        opts.contains(&ChoiceOption::Tag(1)),
        "上限非 6 时效果 1 可选"
    );
    let hp0 = e.debug_hp(0);
    let max0 = e.state_core().player(0).max_hp;
    assert!(pick_mailun(&mut e, 1), "选效果 1");
    assert_eq!(e.debug_hp(0), hp0 - 1, "失去 1 体力");
    assert_eq!(e.state_core().player(0).max_hp, max0 + 1, "上限 +1");
    // 上限 6 时效果 1 不可选（重开一局，把上限改到 6）
    let mut e2 = swd_engine(114005);
    e2.debug_grant(0, "mailun");
    e2.debug_set_max_hp(0, 6);
    // A2 H12：同样推进到回合开始再开窗
    advance_to_turn_begin(&mut e2, 0);
    let opts = mailun_options(&mut e2).expect("脉轮窗（上限 6）");
    assert!(!opts.contains(&ChoiceOption::Tag(1)), "上限 6 时效果 1 不可选");
    assert_eq!(opts.len(), 7, "其余 7 种可选");
}

/// 脉轮（对照）：未持技者阶段开始不开窗；放弃选择不改状态。
#[test]
fn mailun_absent_and_decline() {
    let mut e = swd_engine(114006);
    for _ in 0..8 {
        let c = e.open_choice().cloned();
        assert!(
            c.as_ref().map(|c| c.spec.prompt != "mailun_mode").unwrap_or(true),
            "未持技不开窗：{:?}",
            c.map(|c| c.spec.prompt)
        );
        next_phase(&mut e);
        drain(&mut e);
    }
    // 持技但放弃选择
    let mut e2 = swd_engine(114007);
    e2.debug_grant(0, "mailun");
    for _ in 0..8 {
        let Some(c) = e2.open_choice().cloned() else {
            next_phase(&mut e2);
            drain(&mut e2);
            continue;
        };
        if c.spec.prompt == "mailun_mode" {
            let out = e2.submit(GameCommand {
                actor: c.observer,
                expected_revision: e2.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer { choice_id: c.id, selection: Vec::new() },
            });
            assert!(
                matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
                "{out:?}"
            );
            break;
        }
        let sel: Vec<ChoiceOption> = if c.spec.prompt == "trigger_confirm" {
            vec![ChoiceOption::Confirm]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = e2.submit(GameCommand {
            actor: c.observer,
            expected_revision: e2.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer { choice_id: c.id, selection: sel },
        });
    }
    drain(&mut e2);
    assert_eq!(e2.state_core().player(0).mailun_mode, 0, "放弃不改状态");
}

/// 脉轮：三字段存档往返。
#[test]
fn mailun_state_roundtrip() {
    let mut e = swd_engine(114008);
    {
        let p = e.state_core_mut().player_mut(0);
        p.mailun_mode = 6;
        p.mailun_out_used = true;
        p.mailun_in_used = false;
    }
    let snap = e.save();
    assert_eq!(snap.core.players[0].mailun_mode, 6);
    assert!(snap.core.players[0].mailun_out_used);
    assert!(!snap.core.players[0].mailun_in_used);
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .expect("读包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    let restored = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复");
    assert_eq!(restored.state_core().player(0).mailun_mode, 6);
    assert!(restored.state_core().player(0).mailun_out_used);
}
