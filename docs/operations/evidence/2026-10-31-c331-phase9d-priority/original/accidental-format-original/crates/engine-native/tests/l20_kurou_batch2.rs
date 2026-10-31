//! L20 标记补充批 2：苦肉（`kurou`）+ 主亡忠继（`_xigua_zhuwanzhongji_`）。
//!
//! 上游 `extension/标记补充/extension.js`：
//! - `kurou`：`enable:'phaseUse'` direct——`loseHp(1)` + `draw(2)`；
//! - `_xigua_zhuwanzhongji_`：`trigger {global:'dieBegin'}` 锁定——主公阵亡且
//!   自己为忠臣/明忠 → 身份互换（自己继位为主公）。

use engine_native::state::Role;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn repo_extensions_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn ms_pack() -> engine_native::content::extension::ExtensionPack {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("mark-supplement-upstream-liuli/assets/cards.json"),
    )
    .expect("读标记补充包");
    content_native_loader::bridge_pack(&json)
        .expect("桥接")
        .pack
}

fn ms_engine(seed: u64) -> Engine {
    Engine::new_with_extensions(seed, 4, &[&ms_pack()]).expect("建局")
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

/// 苦肉：发动 → 失去 1 点体力并摸 2 张。
#[test]
fn kurou_loses_hp_and_draws() {
    let mut e = ms_engine(160001);
    e.debug_grant(0, "kurou");
    assert!(e.catalog().skill_by_name("kurou").is_some());
    advance_to_play(&mut e, 0);
    let hp_before = e.debug_hp(0);
    let hand_before = e.debug_hand(0).len();
    let def = e.catalog().skill_by_name("kurou").expect("kurou");
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
    drain(&mut e);
    assert_eq!(e.debug_hp(0), hp_before - 1, "失去 1 点体力");
    assert_eq!(e.debug_hand(0).len(), hand_before + 2, "摸 2 张");
}

/// 苦肉（对照）：未持技者不可发动。
#[test]
fn kurou_absent_rejected() {
    let mut e = ms_engine(160002);
    advance_to_play(&mut e, 0);
    let def = e.catalog().skill_by_name("kurou").expect("kurou");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "未持技拒绝：{out:?}"
    );
}

/// 主亡忠继：身份局中主公阵亡 → 忠臣继位（身份互换）。
#[test]
fn zhuwanzhongji_succession() {
    // 5 人身份局（主/忠/反/反/内）；用带扩展的引擎构造身份局
    let mut e = Engine::new_with_extensions(160003, 5, &[&ms_pack()]).expect("建局");
    e.debug_set_identity_roles(vec![
        Role::Lord,
        Role::Loyalist,
        Role::Rebel,
        Role::Rebel,
        Role::Traitor,
    ]);
    e.debug_grant(1, "zhuwanzhongji");
    // 主公阵亡（1 点伤害致死）
    e.debug_set_hp(0, 1);
    let out = e.debug_deal_damage(Some(2), 0, 1, engine_native::event::DamageNature::Normal);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    drain(&mut e);
    assert!(!e.debug_alive(0), "主公已阵亡");
    let roles = e.state_core().identity_roles().expect("身份局").to_vec();
    assert_eq!(roles[1], Role::Lord, "1 号忠臣继位为主公");
    assert_eq!(roles[0], Role::Loyalist, "0 号（原主公）身份转为忠臣");
    assert_eq!(roles[2], Role::Rebel, "2 号不变");
}
