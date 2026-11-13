//! L20 标记补充批 3：烈弓（`fz_liegong`）+ 狂骨（`fz_xinkuanggu`）。
//!
//! 上游 `extension/标记补充/extension.js`：
//! - `fz_liegong`：`trigger {player:'useCardToPlayered'}`——对目标出【杀】且
//!   目标手牌数 ≥ 自己体力 或 ≤ 自己攻击范围 → `directHit.push(target)`；
//! - `fz_xinkuanggu`：`trigger {source:'damageSource'}` direct——距离 ≤1 且
//!   伤害 >0 → 回复 1 点体力或摸 1 张牌。

use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn repo_extensions_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn ms_engine(seed: u64) -> Engine {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("mark-supplement-upstream-liuli/assets/cards.json"),
    )
    .expect("读标记补充包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        let has_pass = c.spec.options.contains(&ChoiceOption::Pass);
        let sel: Vec<ChoiceOption> = if c.spec.prompt == "fz_liegong:confirm" {
            vec![ChoiceOption::Confirm]
        } else if has_pass {
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
    for _ in 0..16 {
        if e.turn().current == seat && e.turn().phase == Phase::Play {
            return;
        }
        next_phase(e);
        drain(e);
    }
    panic!("未到 {seat} 号出牌阶段");
}

/// 烈弓：目标手牌数 ≥ 出杀者体力 → 不可被闪避（跳过出闪直接伤害）。
#[test]
fn liegong_direct_hit_when_hand_ge_hp() {
    let mut e = ms_engine(161001);
    e.debug_grant(0, "fz_liegong");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 目标 1 号手牌 4 张（≥ 0 号体力 4）；0 号出【杀】
    for _ in 0..4 {
        let c = e.debug_place(1, "shan");
        e.debug_give_hand_card(1, 1, c);
    }
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let hp1_before = e.debug_hp(1);
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
    assert_eq!(e.debug_hp(1), hp1_before - 1, "烈弓：直伤命中");
}

/// 狂骨：造成伤害后选「摸 1」（距离 1）。
#[test]
fn xinkuanggu_draws_or_recovers() {
    let mut e = ms_engine(161002);
    e.debug_grant(0, "fz_xinkuanggu");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let hand_before = e.debug_hand(0).len();
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
    // 狂骨确认门 → 选择窗（摸 1）
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
        if c.spec.prompt == "fz_xinkuanggu_choice" || c.spec.prompt == "fz_xinkuanggu:choose_mode" {
            let out = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Tag(1)],
                },
            });
            assert!(
                matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
                "{out:?}"
            );
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
    assert_eq!(
        e.debug_hand(0).len(),
        hand_before - 1 + 1,
        "狂骨：出杀 -1 手牌，摸 1 → 净持平"
    );
}
