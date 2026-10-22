//! 史诗卡牌批 1：蛊惑（`reguhuo`）——弃一张手牌声明任意基本/锦囊牌使用。
//!
//! 上游 `extension/史诗卡牌/extension.js`：`enable:['chooseToUse','chooseToRespond']`
//! ——`backup.viewAs = {name: 声明名, nature, suit:'none', number:null}`：以一张
//! 手牌（位置 'hs'）转化为声明的**基本牌/锦囊牌**使用；`reguhuo_phase` 每阶段
//! 一次（`hasSkill('reguhuo_phase')` 门控）。

use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn repo_extensions_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

/// 史诗卡牌包（L10）作为宿主；另加载 swd 包以提供武将池广度。
fn engine(seed: u64) -> Engine {
    let epic = std::fs::read_to_string(
        repo_extensions_root().join("epic-cards-upstream-liuli/assets/cards.json"),
    )
    .expect("读史诗卡牌包数据");
    let epic_pack = content_native_loader::bridge_pack(&epic).expect("桥接史诗卡牌").pack;
    Engine::new_with_extensions(seed, 4, &[&epic_pack]).expect("建局")
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

/// 蛊惑：一张手牌声明为【桃】使用（转化声明；牌离手、体力回复）。
#[test]
fn reguhuo_declares_tao_from_hand() {
    let mut e = engine(132001);
    // 宿主包无该技：直接以目录名授予（行为已注册）
    e.debug_grant(0, "reguhuo");
    assert!(e.catalog().skill_by_name("reguhuo").is_some());
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    let card = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, card);
    e.debug_set_hp(0, 2);
    let hp0 = e.debug_hp(0);
    let def = e.catalog().skill_by_name("reguhuo").expect("蛊惑");
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
    // 选牌窗
    let pick = e.open_choice().cloned().expect("蛊惑选牌窗");
    assert_eq!(pick.spec.prompt, "reguhuo_card");
    let out = e.submit(GameCommand {
        actor: pick.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: pick.id,
            selection: vec![ChoiceOption::Card(card)],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    // 声明牌名窗（Tag 下标；桃 = 下标 4）
    let name = e.open_choice().cloned().expect("蛊惑声明窗");
    assert_eq!(name.spec.prompt, "reguhuo_name");
    let out = e.submit(GameCommand {
        actor: name.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: name.id,
            selection: vec![ChoiceOption::Tag(4)],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(0), hp0 + 1, "声明【桃】生效：回复 1");
    assert!(!e.debug_hand(0).contains(&card), "载体牌已使用离手");
}

/// 蛊惑：声明【杀】并对目标造成伤害（转化牌种路由到杀机器）。
#[test]
fn reguhuo_declares_sha_with_target() {
    let mut e = engine(132002);
    e.debug_grant(0, "reguhuo");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    for p in 1..4 {
        e.debug_clear_hand(p);
    }
    let card = e.debug_place(0, "tao");
    e.debug_give_hand_card(0, 0, card);
    let hp1 = e.debug_hp(1);
    let def = e.catalog().skill_by_name("reguhuo").expect("蛊惑");
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
    let pick = e.open_choice().cloned().expect("选牌窗");
    let out = e.submit(GameCommand {
        actor: pick.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: pick.id,
            selection: vec![ChoiceOption::Card(card)],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let name = e.open_choice().cloned().expect("声明窗");
    // 杀 = 下标 0
    let out = e.submit(GameCommand {
        actor: name.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: name.id,
            selection: vec![ChoiceOption::Tag(0)],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let tgt = e.open_choice().cloned().expect("目标窗");
    assert_eq!(tgt.spec.prompt, "reguhuo_targets");
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
    assert_eq!(e.debug_hp(1), hp1 - 1, "声明【杀】命中 1 点");
}

/// 蛊惑（对照）：未持技者不可发动。
#[test]
fn reguhuo_absent_rejected() {
    let mut e = engine(132003);
    advance_to_play(&mut e, 0);
    let def = e.catalog().skill_by_name("reguhuo").expect("蛊惑");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(matches!(out, CommandOutcome::Rejected(_)), "未持技拒绝：{out:?}");
}

/// 蛊惑：**每回合一次**（第 942 轮接线 `vm_aux.once` scope=1）。
///
/// 旧实现无任何标记 ⇒ 同一出牌阶段可以反复发动。现在第一次发动后即记账，
/// 第二次 `ActivateSkill` 必须被拒绝。
#[test]
fn reguhuo_is_once_per_turn() {
    let mut e = engine(132002);
    e.debug_grant(0, "reguhuo");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    let c1 = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, c1);
    e.debug_set_hp(0, 2);
    let def = e.catalog().skill_by_name("reguhuo").expect("蛊惑");

    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "首次发动应可用：{out:?}"
    );
    // 选牌窗出现即说明机器已启动（此时已记账）
    let pick = e.open_choice().cloned().expect("蛊惑选牌窗");
    assert_eq!(pick.spec.prompt, "reguhuo_card");

    // 放弃（取消选择）后再次发动应被拒绝
    let _ = e.submit(GameCommand {
        actor: pick.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: pick.id,
            selection: Vec::new(),
        },
    });
    drain(&mut e);
    let c2 = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, c2);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "同回合第二次发动应被拒绝：{out:?}"
    );
}
