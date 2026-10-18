//! L27 选将美化批 3：每步（`new_meibu`）——弃牌令他人本回合止息。
//!
//! 上游 `extension/选将美化/extension.js`：`trigger {global:'phaseUseBegin'}`
//! direct——目标≠自己、存活、自己有手/装备牌、且**目标在攻击范围内能及自己**；
//! 弃一张牌（he）→ 目标获得临时 `new_zhixi`（止息，至其出牌阶段结束）；
//! 若弃牌非【杀】非锦囊且非黑 → 另获得 `new_meibu_range`（其到你距离视为 1）。

use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn repo_extensions_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn sb_engine(seed: u64) -> Engine {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("select-beautify-upstream-liuli/assets/cards.json"),
    )
    .expect("读选将美化包");
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

fn answer(e: &mut Engine, prompt: &str, sel: Vec<ChoiceOption>) -> bool {
    for _ in 0..32 {
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
        let fallback: Vec<ChoiceOption> = if c.spec.prompt == "trigger_confirm" {
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
            action: GameAction::Answer { choice_id: c.id, selection: fallback },
        });
    }
    false
}

/// 每步：0 号在 1 号出牌阶段开始时弃一张红桃基本牌 → 1 号本回合不能使用牌。
#[test]
fn new_meibu_locks_target_for_play_phase() {
    let mut e = sb_engine(152001);
    e.debug_grant(0, "new_meibu");
    assert!(e.catalog().skill_by_name("new_meibu").is_some());
    // 0 号备一张牌用于弃置（红桃基本牌：非杀/非锦囊/非黑 → 同时给距离效果）
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let card = {
        let found = e
            .state_core()
            .deck
            .iter()
            .copied()
            .find(|c| {
                let d = &e.catalog().cards[*c as usize];
                d.kind.is_basic_kind()
                    && matches!(
                        d.suit,
                        engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
                    )
                    && !d.kind.is_sha_like()
            })
            .expect("红色基本牌（非杀）");
        let core = e.state_core_mut();
        core.deck.retain(|c| *c != found);
        core.players[0].hand.push(found);
        found
    };
    // 1 号备一张【杀】，供其出牌阶段使用（应被止息封锁）
    let sha = e.debug_place(1, "sha");
    e.debug_give_hand_card(1, 1, sha);
    // 推进到 1 号**判定阶段**（出牌阶段的前一阶段），避免 drain 吃掉触发窗
    for _ in 0..16 {
        if e.turn().current == 1 && e.turn().phase == Phase::Judge {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    assert_eq!(e.turn().phase, Phase::Judge);
    next_phase(&mut e); // Judge 收束 → Draw
    drain(&mut e);
    assert_eq!(e.turn().phase, Phase::Draw);
    next_phase(&mut e); // Draw 收束 → Play 开始（每步触发）
    // 每步确认门（0 号观察者）→ 弃牌窗
    let mut triggered = false;
    triggered = answer(&mut e, "trigger_confirm", vec![ChoiceOption::Confirm]) || triggered;
    triggered = answer(&mut e, "new_meibu_discard", vec![ChoiceOption::Card(card)]) || triggered;
    drain(&mut e);
    assert!(triggered, "每步触发（确认 + 弃牌窗）");
    assert!(!e.debug_hand(0).contains(&card), "弃牌已离手");
    // 1 号现在不能使用【杀】
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![0] },
    });
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "止息封锁 1 号出牌：{out:?}"
    );
}

/// 每步（对照）：未持技者不触发（无确认门）。
#[test]
fn new_meibu_absent_no_window() {
    let mut e = sb_engine(152002);
    for _ in 0..16 {
        if e.turn().current == 1 && e.turn().phase == Phase::Play {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    let c = e.open_choice().cloned();
    assert!(
        c.as_ref()
            .map(|c| c.spec.prompt != "new_meibu_discard")
            .unwrap_or(true),
        "未持技无每步弃牌窗：{:?}",
        c.map(|c| c.spec.prompt)
    );
    drain(&mut e);
}
