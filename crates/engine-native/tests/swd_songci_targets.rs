//! swd deferred 消解批 129：颂词（`songci`）——选未记录角色：弃 2 或摸 2。
//!
//! 上游 `character/sp.js:22323`（+ 组技 `songci_draw`）：出牌阶段主动（可重复）
//! ——选一名未发动过的角色（含自己）：其手牌多于体力则弃 2 张（手/装备），
//! 否则摸 2 张；全部角色均已记录时弃牌阶段末摸 1。

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

fn advance_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..12 {
        if e.turn().current == seat && e.turn().phase == Phase::Play {
            return;
        }
        next_phase(e);
        drain(e);
    }
    panic!("未到 {seat} 号出牌阶段");
}

fn activate(e: &mut Engine, skill: &str) -> CommandOutcome {
    let def = e.catalog().skill_by_name(skill).expect("技能在目录");
    e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    })
}

/// 颂词（摸牌分支）：目标手牌 ≤ 体力 → 摸 2 并记录；同目标不可再选。
#[test]
fn songci_draws_two_and_records_target() {
    let mut e = swd_engine(106001);
    e.debug_grant(0, "songci");
    assert!(e.catalog().skill_by_name("songci").is_some());
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_set_hp(1, 4);
    let out = activate(&mut e, "songci");
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let tgt = e.open_choice().cloned().expect("颂词目标窗");
    assert_eq!(tgt.spec.prompt, "songci_target");
    let out = e.submit(GameCommand {
        actor: tgt.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: tgt.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.debug_hand(1).len(), 2, "手牌 ≤ 体力 → 摸 2");
    assert_eq!(e.state_core().player(0).songci_targets, vec![1], "记录目标");
    // 再次发动：目标窗不含 1 号
    let out = activate(&mut e, "songci");
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let tgt2 = e.open_choice().cloned().expect("颂词目标窗（二次）");
    assert!(
        !tgt2.spec.options.contains(&ChoiceOption::Player(1)),
        "已记录目标不再可选：{:?}",
        tgt2.spec.options
    );
    drain(&mut e);
}

/// 颂词（弃牌分支）：目标手牌 > 体力 → 强制弃 2 张（手/装备）并记录。
#[test]
fn songci_forces_discard_when_hand_over_hp() {
    let mut e = swd_engine(106002);
    e.debug_grant(0, "songci");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(1, 1);
    let mut cards = Vec::new();
    for _ in 0..3 {
        let c = e.debug_place(1, "tao");
        e.debug_give_hand_card(0, 1, c);
        cards.push(c);
    }
    assert_eq!(e.debug_hand(1).len(), 3);
    let out = activate(&mut e, "songci");
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let tgt = e.open_choice().cloned().expect("颂词目标窗");
    let out = e.submit(GameCommand {
        actor: tgt.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: tgt.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let pick = e.open_choice().cloned().expect("颂词弃牌窗");
    assert_eq!(pick.spec.prompt, "songci_discard");
    assert_eq!((pick.spec.min, pick.spec.max), (2, 2));
    let out = e.submit(GameCommand {
        actor: pick.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: pick.id,
            selection: vec![ChoiceOption::Card(cards[0]), ChoiceOption::Card(cards[1])],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.debug_hand(1).len(), 1, "强制弃 2 张");
    assert_eq!(e.state_core().player(0).songci_targets, vec![1]);
}

/// 颂词组技：全部角色均已记录 → 弃牌阶段末摸 1；未记录则不摸。
#[test]
fn songci_draw_group_skill_when_all_recorded() {
    let mut e = swd_engine(106003);
    e.debug_grant(0, "songci");
    advance_to_play(&mut e, 0);
    // 手工记录全部 4 名角色
    {
        let p = e.state_core_mut().player_mut(0);
        p.songci_targets = vec![0, 1, 2, 3];
    }
    e.debug_clear_hand(0);
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    // 推进到弃牌阶段（手牌 1 ≤ 体力 4，无弃牌）→ 收束应摸 1
    let mut guard = 0;
    while e.turn().phase != Phase::Discard && guard < 12 {
        next_phase(&mut e);
        drain(&mut e);
        guard += 1;
    }
    assert_eq!(e.turn().phase, Phase::Discard);
    let hand_before = e.debug_hand(0).len();
    next_phase(&mut e); // Discard 收束 → songci_draw
    drain(&mut e);
    assert_eq!(e.debug_hand(0).len(), hand_before + 1, "全员已记录 → 摸 1");
    // 对照：未记录全员者不摸（重开一局）
    let mut e2 = swd_engine(106004);
    e2.debug_grant(0, "songci");
    advance_to_play(&mut e2, 0);
    e2.debug_clear_hand(0);
    let sha2 = e2.debug_place(0, "sha");
    e2.debug_give_hand_card(0, 0, sha2);
    let mut guard = 0;
    while e2.turn().phase != Phase::Discard && guard < 12 {
        next_phase(&mut e2);
        drain(&mut e2);
        guard += 1;
    }
    let hand_before2 = e2.debug_hand(0).len();
    next_phase(&mut e2);
    drain(&mut e2);
    assert_eq!(e2.debug_hand(0).len(), hand_before2, "对照：未全员记录不摸");
}

/// 颂词（对照）：非出牌阶段不可发动；存档往返保持记录表。
#[test]
fn songci_phase_gate_and_roundtrip() {
    let mut e = swd_engine(106005);
    e.debug_grant(0, "songci");
    for _ in 0..12 {
        if e.turn().current == 0 && e.turn().phase == Phase::Draw {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    assert_eq!(e.turn().phase, Phase::Draw);
    let out = activate(&mut e, "songci");
    assert!(matches!(out, CommandOutcome::Rejected(_)), "非出牌阶段拒绝：{out:?}");
    {
        let p = e.state_core_mut().player_mut(0);
        p.songci_targets = vec![1, 2];
    }
    let snap = e.save();
    assert_eq!(snap.core.players[0].songci_targets, vec![1, 2]);
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .expect("读包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    let restored = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复");
    assert_eq!(restored.state_core().player(0).songci_targets, vec![1, 2]);
}
