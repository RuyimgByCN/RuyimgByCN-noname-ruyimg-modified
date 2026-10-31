//! swd deferred 消解批 134：冰封（`bingfeng`）+ 冰封·封（`bingfeng2`）。
//!
//! 上游 `character/swd.js`：出牌阶段限定技——翻面自己与至多 3 名未翻面
//! 其他角色，失去 1 点体力上限，移除玄咒，并给双方授予 `bingfeng2`
//! （`cardEnabled/cardUsable/cardRespondable/cardSavable` 全 false——
//! 翻面期间不能使用/打出/响应/求桃任何牌；翻回正面时自移除）。

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

/// 冰封：翻面自己与目标 + 失去 1 体力上限 + 授予封锁（封锁者不能出牌）。
#[test]
fn bingfeng_flips_loses_max_hp_and_locks_cards() {
    let mut e = swd_engine(111001);
    e.debug_grant(0, "bingfeng");
    assert!(e.catalog().skill_by_name("bingfeng").is_some());
    assert!(e.catalog().skill_by_name("bingfeng2").is_some());
    for _ in 0..12 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    assert_eq!(e.turn().current, 0);
    assert_eq!(e.turn().phase, Phase::Play);
    let max0 = e.state_core().player(0).max_hp;
    let def = e.catalog().skill_by_name("bingfeng").expect("冰封");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let pick = e.open_choice().cloned().expect("冰封目标窗");
    assert_eq!(pick.spec.prompt, "bingfeng_targets");
    let out = e.submit(GameCommand {
        actor: pick.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: pick.id,
            selection: vec![ChoiceOption::Player(1), ChoiceOption::Player(2)],
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    drain(&mut e);
    assert!(e.state_core().player(0).face_down, "自己翻面");
    assert!(e.state_core().player(1).face_down, "1 号翻面");
    assert!(e.state_core().player(2).face_down, "2 号翻面");
    assert!(!e.state_core().player(3).face_down, "未选目标不翻面");
    assert_eq!(e.state_core().player(0).max_hp, max0 - 1, "失去 1 体力上限");
    // 授予封锁：自己与两目标
    let b2 = e.catalog().skill_by_name("bingfeng2").expect("冰封·封");
    for p in [0u8, 1, 2] {
        assert!(
            e.state_core().skills.contains_key(&(p, b2)),
            "{p} 号获得封锁"
        );
    }
    assert!(!e.state_core().skills.contains_key(&(3, b2)));
    // 封锁者不能出牌（PlayCard 被拒）
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "封锁者出牌被拒：{out:?}"
    );
    // 限定技：二次发动被拒
    let again = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(
        matches!(again, CommandOutcome::Rejected(_)),
        "限定技：{again:?}"
    );
}

/// 冰封·封：封锁者不能打出闪（杀必中）。
#[test]
fn bingfeng_locked_player_cannot_dodge() {
    let mut e = swd_engine(111002);
    e.debug_grant(0, "bingfeng");
    // 直接给 1 号上封锁（模拟冰封命中）
    let b2 = e.catalog().skill_by_name("bingfeng2").expect("冰封·封");
    e.state_core_mut().skills.insert(
        (1, b2),
        engine_native::state::SkillInstance {
            owner: 1,
            enabled: true,
            once_used: false,
            expires_end_of_turn: false,
            rounds: 0,
        },
    );
    // 推进到 0 号出牌阶段
    for _ in 0..12 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    assert_eq!(e.turn().phase, Phase::Play);
    // 1 号手里有闪也不能打出
    e.debug_clear_hand(1);
    let shan = e.debug_place(1, "shan");
    e.debug_give_hand_card(0, 1, shan);
    e.debug_clear_hand(0);
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let hp1 = e.debug_hp(1);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1 - 1, "封锁者不能闪 → 命中");
    assert!(e.debug_hand(1).contains(&shan), "闪仍在手");
}

/// 冰封·封（对照）：未封锁者可正常出闪。
#[test]
fn bingfeng_unlocked_player_dodges() {
    let mut e = swd_engine(111003);
    for _ in 0..12 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    assert_eq!(e.turn().phase, Phase::Play);
    e.debug_clear_hand(1);
    let shan = e.debug_place(1, "shan");
    e.debug_give_hand_card(0, 1, shan);
    e.debug_clear_hand(0);
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let hp1 = e.debug_hp(1);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    // 打出闪
    let c = e.open_choice().cloned().expect("闪响应窗");
    assert_eq!(c.spec.prompt, "respond_shan");
    let out = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: c.id,
            selection: vec![ChoiceOption::Card(shan)],
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1, "未封锁者闪避成功");
}

/// 冰封·封：翻回正面时自移除（封锁解除）。
#[test]
fn bingfeng_lock_removed_on_turn_over() {
    let mut e = swd_engine(111004);
    let b2 = e.catalog().skill_by_name("bingfeng2").expect("冰封·封");
    e.state_core_mut().skills.insert(
        (0, b2),
        engine_native::state::SkillInstance {
            owner: 0,
            enabled: true,
            once_used: false,
            expires_end_of_turn: false,
            rounds: 0,
        },
    );
    e.debug_set_face_down(0, true);
    // 翻回正面（引擎翻面提交）
    e.debug_flip(0);
    assert!(!e.state_core().player(0).face_down, "已翻回正面");
    drain(&mut e);
    assert!(
        !e.state_core().skills.contains_key(&(0, b2)),
        "翻回正面后封锁移除"
    );
}
