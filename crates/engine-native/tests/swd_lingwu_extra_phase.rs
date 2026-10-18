//! swd deferred 消解批 120：灵武（`lingwu`）+ `insertPhase` 额外阶段原语。
//!
//! 上游 `character/swd.js:6325`：`phaseAfter` 频繁技——本回合已用牌数
//! ≥ 体力则插入一个额外出牌阶段（`player.insertPhase()`）；插入阶段的
//! 收束不再触发（上游 `event.skill != 'lingwu'`）。

use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn repo_extensions_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

/// 带 swd 包建局（`lingwu` 属 swd 扩展技能）。
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

fn advance_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..12 {
        if e.turn().current == seat && e.turn().phase == Phase::Play {
            return;
        }
        next_phase(e);
    }
    panic!("未到 {seat} 号出牌阶段：{:?}/{:?}", e.turn().current, e.turn().phase);
}

/// 灵武：阶段结束且本回合已用牌数 ≥ 体力 → 插入额外出牌阶段；插入阶段
/// 收束不再插入（防递归）；未达标不插入（对照）。
#[test]
fn lingwu_inserts_extra_play_phase_when_used_reaches_hp() {
    let mut e = swd_engine(97001);
    e.debug_set_hp(0, 1);
    e.debug_grant(0, "lingwu");
    assert!(
        e.catalog().skill_by_name("lingwu").is_some(),
        "灵武在目录（宿主行为）"
    );
    advance_to_play(&mut e, 0);
    // 清空其他玩家技能：武将分配随种子变化，受击方可能持反击技令 p0 失血
    // 乃至死亡（本用例只验证 p0 灵武条件，故先摘除他方技能）
    e.state_core_mut().skills.retain(|(o, _), _| *o == 0);
    e.debug_set_hp(0, 1);
    assert!(e.debug_alive(0), "p0 存活");
    assert_eq!(e.state_core().turn.cards_used_turn, 0, "开局未用牌");

    // 用 1 张牌（摸牌阶段已过：清手后手动置牌）
    e.debug_clear_hand(0);
    let sha = e.debug_place(0, "sha");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.state_core().turn.cards_used_turn, 1, "本回合已用 1");
    e.debug_set_hp(0, 1);
    assert_eq!(e.state_core().player(0).hp, 1, "体力 1");
    e.debug_grant(0, "lingwu");

    // 出牌阶段结束 → 灵武插入额外出牌阶段
    next_phase(&mut e);
    assert_eq!(e.turn().current, 0, "仍在 0 号回合");
    assert_eq!(e.turn().phase, Phase::Play, "插入额外出牌阶段");
    // 插入阶段收束：不再插入（used 仍 ≥ hp），正常进入弃牌阶段
    next_phase(&mut e);
    assert_eq!(e.turn().phase, Phase::Discard, "插入阶段收束不再插入");
}

/// 对照：已用牌数 < 体力 → 不插入，正常进入弃牌阶段。
#[test]
fn lingwu_no_insert_below_hp() {
    let mut e = swd_engine(97002);
    e.debug_set_hp(0, 4);
    e.debug_grant(0, "lingwu");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    let sha = e.debug_place(0, "sha");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.state_core().turn.cards_used_turn, 1);
    next_phase(&mut e);
    assert_eq!(e.turn().phase, Phase::Discard, "未达标不插入");
}

/// 对照：无灵武者即使已用牌数 ≥ 体力也不插入。
#[test]
fn lingwu_absent_no_insert() {
    let mut e = Engine::new(97003, 4);
    e.debug_set_hp(0, 1);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    let sha = e.debug_place(0, "sha");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert!(e.catalog().skill_by_name("lingwu").is_none());
    next_phase(&mut e);
    assert_eq!(e.turn().phase, Phase::Discard, "无灵武不插入");
}

/// 额外阶段队列与插入位入存档/哈希：往返保持（插入阶段挂起中恢复）。
#[test]
fn lingwu_extra_phase_state_roundtrip() {
    let mut e = swd_engine(97004);
    e.debug_set_hp(0, 1);
    e.debug_grant(0, "lingwu");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    let sha = e.debug_place(0, "sha");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    // 手工压入两个额外阶段并置插入位，验证存档往返
    {
        let turn = &mut e.state_core_mut().turn;
        turn.extra_phases = vec![2, 3];
        turn.extra_phase_lingwu = true;
    }
    let snap = e.save();
    assert_eq!(snap.core.turn.extra_phases, vec![2, 3]);
    assert!(snap.core.turn.extra_phase_lingwu);
    assert_eq!(snap.core.turn.cards_used_turn, 1);
    // 扩展局须带包恢复（规则版本含扩展内容锁）
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .expect("读 swd 包数据");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接 swd 包").pack;
    let restored = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复");
    assert_eq!(restored.state_core().turn.extra_phases, vec![2, 3]);
    assert!(restored.state_core().turn.extra_phase_lingwu);
    assert_eq!(restored.state_core().turn.cards_used_turn, 1);
    // 恢复后推进：先弹出出牌阶段，再弹出弃牌阶段
    let mut e = restored;
    next_phase(&mut e);
    assert_eq!(e.turn().phase, Phase::Play, "队列首个=出牌阶段");
    next_phase(&mut e);
    assert_eq!(e.turn().phase, Phase::Discard, "队列次个=弃牌阶段");
}

/// 本回合已用牌数在换回合时清零。
#[test]
fn cards_used_turn_resets_next_turn() {
    let mut e = swd_engine(97005);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    let sha = e.debug_place(0, "sha");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.state_core().turn.cards_used_turn, 1);
    // 结束 0 号回合 → 1 号回合
    for _ in 0..6 {
        if e.turn().current != 0 {
            break;
        }
        next_phase(&mut e);
    }
    assert_ne!(e.turn().current, 0, "回合已易主");
    assert_eq!(e.state_core().turn.cards_used_turn, 0, "换回合清零");
}
