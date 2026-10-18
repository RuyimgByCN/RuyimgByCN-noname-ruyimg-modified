//! U08 英雄杀批 1：反击（`yxs_fanji`）+ 反击·红杀直伤（`yxs_fanji2`）。
//!
//! 上游 `apps/core/extension/英雄杀/character/skill.js`：
//! - `yxs_fanji`：`trigger {player:'damageEnd'}` direct——手牌有【杀】且伤害载体
//!   为【杀】/【决斗】→ `addTempSkill('yxs_fanji2','shaAfter')` +
//!   `chooseToUse({name:'sha'}, trigger.source, ...)`；
//! - `yxs_fanji2`：`trigger {player:'shaBegin'}` direct——红色【杀】→
//!   `trigger.directHit = true`。

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


/// 反击：受到【杀】伤害后确认 → 选一张【杀】反击来源（来源受 1 点伤害）。
#[test]
fn yxs_fanji_counterattacks_source() {
    let mut e = yxs_engine(157001);
    e.debug_grant(0, "yxs_fanji");
    assert!(e.catalog().skill_by_name("yxs_fanji").is_some());
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 0 号手牌备一张【杀】（反击用）
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    // 1 号对 0 号造成 1 点伤害（无载体牌 → 用 debug_deal_damage 无法构造载体；
    // 改为让 1 号出【杀】：先推进到 1 号出牌阶段）
    for _ in 0..16 {
        if e.turn().current == 1 && e.turn().phase == Phase::Play {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    let sha1 = e.debug_place(1, "sha");
    e.debug_give_hand_card(1, 1, sha1);
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha1, targets: vec![0] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    // 不 drain：手动应答闪窗 → 反击确认 → 反击选杀
    let hp1_before = e.debug_hp(1);
    let mut ok = false;
    let mut ok2 = false;
    for _ in 0..12 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "yxs_fanji_sha" {
            let out = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Card(sha)],
                },
            });
            assert!(
                matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
                "{out:?}"
            );
            ok2 = true;
            continue;
        }
        let sel: Vec<ChoiceOption> = if c.spec.prompt == "trigger_confirm" {
            ok = true;
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
    drain(&mut e);
    assert!(ok || ok2, "反击触发（确认 + 选杀窗）");
    assert_eq!(e.debug_hp(1), hp1_before - 1, "来源被反击受到 1 点伤害");
    let _ = DamageNature::Normal;
}

/// 反击（对照）：未持技者受伤后无反击窗。
#[test]
fn yxs_fanji_absent_no_window() {
    let mut e = yxs_engine(157002);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    for _ in 0..16 {
        if e.turn().current == 1 && e.turn().phase == Phase::Play {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    let sha1 = e.debug_place(1, "sha");
    e.debug_give_hand_card(1, 1, sha1);
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha1, targets: vec![0] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    let c = e.open_choice().cloned();
    assert!(
        c.as_ref()
            .map(|c| c.spec.prompt != "yxs_fanji_sha")
            .unwrap_or(true),
        "未持技无反击选牌窗：{:?}",
        c.map(|c| c.spec.prompt)
    );
}
