//! swd deferred 消解批 122：入梦（`rumeng`）——他人出牌阶段前弃牌/跳阶段。
//!
//! 上游 `character/swd.js`（`rumeng` 39 行）：`global phaseUseBefore`，
//! 自己手/装备中有非基本牌且非自己回合时可发动——弃一张非基本牌，令回合
//! 角色弃一张基本牌，否则其跳过出牌与弃牌阶段（`trigger.cancel()` +
//! `skip('phaseDiscard')`）。

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
    for _ in 0..48 {
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

/// 应答指定 prompt 的选择（断言被接受）。
fn answer(e: &mut Engine, prompt: &str, sel: Vec<ChoiceOption>) -> bool {
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == prompt {
            let out = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer { choice_id: c.id, selection: sel },
            });
            assert!(
                matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
                "{out:?}"
            );
            return true;
        }
        let has_pass = c.spec.options.contains(&ChoiceOption::Pass);
        let fallback: Vec<ChoiceOption> = if has_pass {
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
            action: GameAction::Answer { choice_id: c.id, selection: fallback },
        });
    }
    false
}

/// 入梦（跳过路径）：1 号持技且有非基本牌 → 0 号出牌阶段前发动、弃非基本牌
/// → 0 号不弃基本牌 → 其出牌阶段立即收束且弃牌阶段被跳过。
#[test]
fn rumeng_skip_play_and_discard_when_target_declines() {
    let mut e = swd_engine(99001);
    e.debug_grant(1, "rumeng");
    assert!(e.catalog().skill_by_name("rumeng").is_some());
    // 1 号：一张非基本牌（杀之外用桃? 需非基本 → 用装备/锦囊）
    e.debug_clear_hand(1);
    let trick = e.debug_place(1, "wuzhong");
    e.debug_give_hand_card(0, 1, trick);
    // 0 号：一张基本牌（杀）用于对照其「弃基本牌」分支
    e.debug_clear_hand(0);
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);

    // 推进到 0 号出牌阶段前（入梦窗在 Play begin）
    for _ in 0..10 {
        if e.turn().current == 0 && e.turn().phase == Phase::Draw {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    assert_eq!(e.turn().current, 0);
    next_phase(&mut e); // Draw → Play begin（入梦窗）
    assert!(
        answer(&mut e, "trigger_confirm", vec![ChoiceOption::Confirm]),
        "入梦确认窗"
    );
    // 1 号弃非基本牌
    let hand1 = e.debug_hand(1).len();
    assert!(
        answer(
            &mut e,
            "rumeng_discard",
            vec![ChoiceOption::Card(trick)]
        ),
        "入梦弃非基本牌窗"
    );
    assert_eq!(e.debug_hand(1).len(), hand1 - 1, "1 号弃 1 张");
    // 0 号取消 → 跳过出牌与弃牌阶段
    let hand0_before = e.debug_hand(0).len();
    assert!(
        answer(&mut e, "rumeng_basic_or_skip", Vec::new()),
        "目标弃基本牌或跳过窗"
    );
    drain(&mut e);
    // 出牌阶段已收束（或已进入弃牌阶段并跳过）→ 最终到结束阶段
    for _ in 0..4 {
        if e.turn().phase == Phase::End || e.turn().current != 0 {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    assert!(
        e.turn().current != 0 || e.turn().phase == Phase::End,
        "跳过出牌与弃牌阶段后收束：{:?}/{:?}",
        e.turn().current,
        e.turn().phase
    );
    assert_eq!(
        e.debug_hand(0).len(),
        hand0_before,
        "0 号未弃基本牌（手牌数不变）"
    );
    assert!(!e.state_core().player(0).skip_discard_phase, "跳位用后清零");
}

/// 入梦（弃基本牌路径）：目标弃一张基本牌 → 不跳过阶段，照常出牌/弃牌。
#[test]
fn rumeng_target_discards_basic_card_no_skip() {
    let mut e = swd_engine(99002);
    e.debug_grant(1, "rumeng");
    e.debug_clear_hand(1);
    let trick = e.debug_place(1, "wuzhong");
    e.debug_give_hand_card(0, 1, trick);
    e.debug_clear_hand(0);
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    for _ in 0..10 {
        if e.turn().current == 0 && e.turn().phase == Phase::Draw {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    next_phase(&mut e);
    assert!(
        answer(&mut e, "trigger_confirm", vec![ChoiceOption::Confirm]),
        "入梦确认窗"
    );
    assert!(
        answer(&mut e, "rumeng_discard", vec![ChoiceOption::Card(trick)]),
        "入梦弃非基本牌窗"
    );
    let hand0_before = e.debug_hand(0).len();
    assert!(
        answer(
            &mut e,
            "rumeng_basic_or_skip",
            vec![ChoiceOption::Card(sha)]
        ),
        "目标弃基本牌窗"
    );
    drain(&mut e);
    assert_eq!(e.turn().current, 0);
    assert_eq!(e.turn().phase, Phase::Play, "未跳过出牌阶段");
    assert_eq!(
        e.debug_hand(0).len(),
        hand0_before - 1,
        "0 号弃掉一张基本牌"
    );
    assert!(!e.state_core().player(0).skip_discard_phase);
}

/// 入梦（对照）：1 号只有基本牌 → 过滤不成立，无入梦窗。
#[test]
fn rumeng_no_window_without_non_basic() {
    let mut e = swd_engine(99003);
    e.debug_grant(1, "rumeng");
    e.debug_clear_hand(1);
    let sha1 = e.debug_place(1, "sha");
    e.debug_give_hand_card(0, 1, sha1);
    for _ in 0..10 {
        if e.turn().current == 0 && e.turn().phase == Phase::Draw {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    next_phase(&mut e);
    let c = e.open_choice().cloned();
    assert!(
        c.as_ref()
            .map(|c| c.spec.prompt != "trigger_confirm")
            .unwrap_or(true),
        "无非基本牌不应开窗：{:?}",
        c.map(|c| c.spec.prompt)
    );
    drain(&mut e);
    assert_eq!(e.turn().phase, Phase::Play, "照常出牌阶段");
}

/// 入梦（对照）：跳过位只影响一次——用后清零，下一回合弃牌阶段照常。
#[test]
fn rumeng_skip_flag_one_shot() {
    let mut e = swd_engine(99004);
    {
        let p = e.state_core_mut().player_mut(0);
        p.skip_discard_phase = true;
    }
    // 推进 0 号回合到弃牌阶段：应被跳过（直接 End 或已过 Discard）
    for _ in 0..12 {
        if e.turn().current == 0 && matches!(e.turn().phase, Phase::Discard | Phase::End) {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    assert!(!e.state_core().player(0).skip_discard_phase, "用后清零");
    // 存档往返保持位（置位后立即存）
    e.state_core_mut().player_mut(0).skip_discard_phase = true;
    let snap = e.save();
    assert!(snap.core.players[0].skip_discard_phase);
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .expect("读包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    let restored = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复");
    assert!(restored.state_core().player(0).skip_discard_phase);
}
