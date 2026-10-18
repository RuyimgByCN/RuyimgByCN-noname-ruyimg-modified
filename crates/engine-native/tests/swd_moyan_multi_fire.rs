//! swd 批 151：魔焰（`moyan`）——弃 N 张红牌对 N 名角色各 1 点火焰伤害。
//!
//! 上游 `character/swd.js`：`enable:'phaseUse'`，`usable:1`——`filter`：体力
//! 未满且有红色手牌；`selectCard`：1..min(其他角色数, 体力上限-体力)；
//! `selectTarget`：目标数 = 已选牌数；`content`：每个目标各 1 点火焰伤害。

use engine_native::catalog::Suit;
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

fn place_red_basic(e: &mut Engine, player: u8) -> u32 {
    let found = e
        .state_core()
        .deck
        .iter()
        .copied()
        .find(|c| {
            let def = &e.catalog().cards[*c as usize];
            def.kind.is_basic_kind()
                && matches!(def.suit, Suit::Heart | Suit::Diamond)
        })
        .expect("红桃/方块基本牌");
    let core = e.state_core_mut();
    core.deck.retain(|c| *c != found);
    core.players[player as usize].hand.push(found);
    found
}

/// 魔焰：弃 2 张红牌 → 选 2 名其他角色 → 各受 1 点火焰伤害。
#[test]
fn moyan_two_red_two_targets_fire_damage() {
    let mut e = swd_engine(127001);
    e.debug_grant(0, "moyan");
    assert!(e.catalog().skill_by_name("moyan").is_some());
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    // 体力上限 4，体力降到 2 → 可选至多 2 张
    e.debug_set_hp(0, 2);
    let r1 = place_red_basic(&mut e, 0);
    let r2 = place_red_basic(&mut e, 0);
    let hp1 = e.debug_hp(1);
    let hp2 = e.debug_hp(2);
    let def = e.catalog().skill_by_name("moyan").expect("魔焰");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let pick = e.open_choice().cloned().expect("魔焰选牌窗");
    assert_eq!(pick.spec.prompt, "moyan_cards");
    assert_eq!((pick.spec.min, pick.spec.max), (1, 2), "上限 = 体力差 2");
    let out = e.submit(GameCommand {
        actor: pick.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: pick.id,
            selection: vec![ChoiceOption::Card(r1), ChoiceOption::Card(r2)],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let tgt = e.open_choice().cloned().expect("魔焰目标窗");
    assert_eq!(tgt.spec.prompt, "moyan_targets");
    assert_eq!((tgt.spec.min, tgt.spec.max), (2, 2), "目标数 = 选牌数");
    let out = e.submit(GameCommand {
        actor: tgt.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: tgt.id,
            selection: vec![ChoiceOption::Player(1), ChoiceOption::Player(2)],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1 - 1, "1 号受 1 点火焰伤害");
    assert_eq!(e.debug_hp(2), hp2 - 1, "2 号受 1 点火焰伤害");
    assert!(!e.debug_hand(0).contains(&r1), "红牌已弃");
    assert!(!e.debug_hand(0).contains(&r2), "红牌已弃");
}

/// 魔焰（对照）：体力已满 → 不可发动；无红牌 → 不可发动。
#[test]
fn moyan_requires_wounded_and_red_card() {
    let mut e = swd_engine(127002);
    e.debug_grant(0, "moyan");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    let _ = place_red_basic(&mut e, 0);
    let def = e.catalog().skill_by_name("moyan").expect("魔焰");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(matches!(out, CommandOutcome::Rejected(_)), "体力已满拒绝：{out:?}");
    // 未满但无红牌
    e.debug_set_hp(0, 2);
    e.debug_clear_hand(0);
    let _black = {
        let found = e
            .state_core()
            .deck
            .iter()
            .copied()
            .find(|c| {
                let d = &e.catalog().cards[*c as usize];
                d.kind.is_basic_kind() && matches!(d.suit, Suit::Spade | Suit::Club)
            })
            .expect("黑色基本牌");
        let core = e.state_core_mut();
        core.deck.retain(|c| *c != found);
        core.players[0].hand.push(found);
        found
    };
    let out2 = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(matches!(out2, CommandOutcome::Rejected(_)), "无红牌拒绝：{out2:?}");
}

/// 魔焰（对照）：未持技者不可发动。
#[test]
fn moyan_absent_rejected() {
    let mut e = swd_engine(127003);
    advance_to_play(&mut e, 0);
    e.debug_set_hp(0, 2);
    let _ = place_red_basic(&mut e, 0);
    let def = e.catalog().skill_by_name("moyan").expect("魔焰");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(matches!(out, CommandOutcome::Rejected(_)), "未持技拒绝：{out:?}");
}
