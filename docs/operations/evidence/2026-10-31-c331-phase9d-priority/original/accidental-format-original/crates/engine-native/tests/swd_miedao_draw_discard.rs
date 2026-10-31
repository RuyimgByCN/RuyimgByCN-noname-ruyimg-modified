//! swd 批 152：灭道（`miedao`）——摸牌多摸 / 弃牌多弃「已损失体力」。
//!
//! 上游 `character/swd.js`：组技两面——`miedao1`（`phaseDrawBegin` 锁定）
//! 体力未满时摸牌 +（maxHp-hp）；`miedao2`（`phaseDiscardBegin` 锁定）体力
//! 未满时强制弃（maxHp-hp）张（手/装备）。

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
    let pack = content_native_loader::bridge_pack(&json)
        .expect("桥接 swd 包")
        .pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        let has_pass = c.spec.options.contains(&ChoiceOption::Pass);
        let sel: Vec<ChoiceOption> = if has_pass {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec
                .options
                .iter()
                .take(c.spec.min.max(1))
                .copied()
                .collect()
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

fn next_phase(e: &mut Engine) {
    let actor = e.turn().current;
    let out = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
}

/// 灭道：体力差 2 → 摸牌 +2；弃牌阶段强制弃 2 张。
#[test]
fn miedao_draw_extra_and_forced_discard() {
    let mut e = swd_engine(128001);
    e.debug_grant(0, "miedao");
    assert!(e.catalog().skill_by_name("miedao").is_some());
    e.debug_set_hp(0, 2);
    // 推进到 0 号**判定阶段**（摸牌前一阶段；摸牌发生在摸牌阶段开始）
    for _ in 0..12 {
        if e.turn().current == 0 && e.turn().phase == Phase::Judge {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    assert_eq!(e.turn().phase, Phase::Judge);
    let before = e.debug_hand(0).len();
    next_phase(&mut e); // Judge 收束 → Draw 开始（摸牌）
    drain(&mut e);
    assert_eq!(e.turn().phase, Phase::Draw);
    assert_eq!(e.debug_hand(0).len(), before + 4, "多摸已损失体力 2 张");
    // 推进到弃牌阶段 → 强制弃 2
    let mut guard = 0;
    while e.turn().phase != Phase::Discard && guard < 8 {
        next_phase(&mut e);
        drain(&mut e);
        guard += 1;
    }
    assert_eq!(e.turn().phase, Phase::Discard);
    let hand = e.debug_hand(0).len();
    assert!(hand >= 2, "手牌足够弃置：{hand}");
    // 弃牌阶段开始 → 灭道强制弃窗（不可取消）
    let c = e.open_choice().cloned();
    if let Some(c) = c {
        if c.spec.prompt == "miedao_discard" {
            assert_eq!((c.spec.min, c.spec.max), (2, 2), "强制弃 2");
            assert!(!c.spec.allow_cancel, "不可取消");
            let sel: Vec<ChoiceOption> = c.spec.options.iter().take(2).copied().collect();
            let out = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            });
            assert!(
                matches!(
                    out,
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
                ),
                "{out:?}"
            );
        }
    }
    drain(&mut e);
    assert!(e.debug_hand(0).len() <= hand, "弃置后手牌不增");
}

/// 灭道（对照）：体力已满 → 无多摸、无强制弃窗。
#[test]
fn miedao_no_effect_at_full_hp() {
    let mut e = swd_engine(128002);
    e.debug_grant(0, "miedao");
    for _ in 0..12 {
        if e.turn().current == 0 && e.turn().phase == Phase::Judge {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    let before = e.debug_hand(0).len();
    next_phase(&mut e); // Judge 收束 → Draw 开始
    drain(&mut e);
    assert_eq!(e.debug_hand(0).len(), before + 2, "体力已满：正常摸 2");
    let mut guard = 0;
    while e.turn().phase != Phase::Discard && guard < 8 {
        next_phase(&mut e);
        drain(&mut e);
        guard += 1;
    }
    let c = e.open_choice().cloned();
    assert!(
        c.as_ref()
            .map(|c| c.spec.prompt != "miedao_discard")
            .unwrap_or(true),
        "体力已满不出现灭道弃牌窗：{:?}",
        c.map(|c| c.spec.prompt)
    );
    drain(&mut e);
}

/// 灭道（对照）：未持技者体力未满亦无多摸。
#[test]
fn miedao_absent_no_extra_draw() {
    let mut e = swd_engine(128003);
    e.debug_set_hp(0, 2);
    for _ in 0..12 {
        if e.turn().current == 0 && e.turn().phase == Phase::Judge {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    let before = e.debug_hand(0).len();
    next_phase(&mut e); // Judge 收束 → Draw 开始
    drain(&mut e);
    assert_eq!(e.debug_hand(0).len(), before + 2, "未持技：正常摸 2");
}
