//! swd deferred 消解批 128：诛害（`zhuhai`）——结束阶段角色造成过伤害则用杀。
//!
//! 上游 `character/refresh.js:13643`：`global phaseJieshuBegin`，`direct:true`
//! ——若结束阶段角色本回合造成过伤害、自己手中有【杀】且其在自己攻击距离
//! 内，可对其使用一张【杀】。

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

/// 诛害：1 号回合造成过伤害 → 其结束阶段开始时 0 号（持技、有杀）可对其用杀。
#[test]
fn zhuhai_uses_sha_on_damaging_player() {
    let mut e = swd_engine(105001);
    e.debug_grant(0, "zhuhai");
    assert!(e.catalog().skill_by_name("zhuhai").is_some());
    // 0 号手中一张杀
    e.debug_clear_hand(0);
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    // 推进到 1 号出牌阶段（其回合已开始，本回合伤害统计已清零）
    let mut guard = 0;
    while !(e.turn().current == 1 && e.turn().phase == Phase::Play) && guard < 24 {
        next_phase(&mut e);
        drain(&mut e);
        guard += 1;
    }
    assert_eq!(e.turn().current, 1);
    assert_eq!(e.turn().phase, Phase::Play);
    // 1 号在其回合内造成伤害（打 2 号 1 点）
    let out = e.debug_deal_damage(Some(1), 2, 1, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert!(
        e.state_core().player(1).turn_damage > 0,
        "1 号本回合造成过伤害"
    );
    // 推进到 1 号弃牌阶段（其 End begin 前一步）
    guard = 0;
    while !(e.turn().current == 1 && e.turn().phase == Phase::Discard) && guard < 12 {
        next_phase(&mut e);
        drain(&mut e);
        guard += 1;
    }
    assert_eq!(e.turn().current, 1);
    assert_eq!(e.turn().phase, Phase::Discard);
    let hp1 = e.debug_hp(1);
    next_phase(&mut e); // Discard → End begin（诛害窗）
    assert!(
        answer(&mut e, "trigger_confirm", vec![ChoiceOption::Confirm]),
        "诛害确认窗"
    );
    assert!(
        answer(&mut e, "zhuhai_sha", vec![ChoiceOption::Card(sha)]),
        "诛害选杀窗"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1 - 1, "0 号对结束阶段角色用杀：1 号受 1 伤");
    assert!(!e.debug_hand(0).contains(&sha), "杀已使用离手");
}

/// 对照：结束阶段角色本回合未造成伤害 → 不开窗。
#[test]
fn zhuhai_no_window_without_damage() {
    let mut e = swd_engine(105002);
    e.debug_grant(0, "zhuhai");
    e.debug_clear_hand(0);
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let mut guard = 0;
    while !(e.turn().current == 1 && e.turn().phase == Phase::Discard) && guard < 24 {
        next_phase(&mut e);
        drain(&mut e);
        guard += 1;
    }
    next_phase(&mut e);
    let c = e.open_choice().cloned();
    assert!(
        c.as_ref().map(|c| c.spec.prompt != "trigger_confirm").unwrap_or(true),
        "未造成伤害不开窗：{:?}",
        c.map(|c| c.spec.prompt)
    );
    drain(&mut e);
}

/// 对照：自己手中无杀 → 不开窗（有伤害也不开）。
#[test]
fn zhuhai_no_window_without_sha() {
    let mut e = swd_engine(105003);
    e.debug_grant(0, "zhuhai");
    let out = e.debug_deal_damage(Some(1), 2, 1, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    e.debug_clear_hand(0);
    let mut guard = 0;
    while !(e.turn().current == 1 && e.turn().phase == Phase::Discard) && guard < 24 {
        next_phase(&mut e);
        drain(&mut e);
        guard += 1;
    }
    next_phase(&mut e);
    let c = e.open_choice().cloned();
    assert!(
        c.as_ref().map(|c| c.spec.prompt != "trigger_confirm").unwrap_or(true),
        "无杀不开窗：{:?}",
        c.map(|c| c.spec.prompt)
    );
    drain(&mut e);
}

/// 对照：未持技者不触发。
#[test]
fn zhuhai_absent_no_window() {
    let mut e = swd_engine(105004);
    let out = e.debug_deal_damage(Some(1), 2, 1, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    e.debug_clear_hand(0);
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let mut guard = 0;
    while !(e.turn().current == 1 && e.turn().phase == Phase::Discard) && guard < 24 {
        next_phase(&mut e);
        drain(&mut e);
        guard += 1;
    }
    next_phase(&mut e);
    let c = e.open_choice().cloned();
    assert!(
        c.as_ref().map(|c| c.spec.prompt != "trigger_confirm").unwrap_or(true),
        "未持技不开窗：{:?}",
        c.map(|c| c.spec.prompt)
    );
    drain(&mut e);
}
