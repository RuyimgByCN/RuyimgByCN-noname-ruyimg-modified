//! R05-C 批红守卫：content 侧 `ctx.draw_cards`（同步直改）→ `ctx.start_draw`
//! （真实 R 帧异步摸牌，R05 合同 v3 §3「caller 让出后续行」）。
//!
//! 三个 caller 族各取一个代表技，断言**异步语义的正确形态**——修前
//! （同步路径）全部为红：
//!
//! · swd_status 族（森罗）：同 step 循环「弃 2+摸 1」逐目标交错；异步
//!   形态＝本机器步先完成全部弃置、R 子树随后统一交付摸牌 → 日志中
//!   全部 CardDiscarded 先于全部 CardsDrawn。
//! · standard skills 族（苦肉）：摸 2 遇牌堆仅剩 1 张且弃堆空——同步
//!   draw_cards 部分入手（+1 张、发 CardsDrawn）；异步 R 帧双枯竭
//!   Exhausted、held 转 detached（§4）→ 手牌不变、无 CardsDrawn 日志、
//!   detached_cards +1。
//! · skills_recovery 族（狂骨）：选「摸 1」臂内 draw_cards 在
//!   SkillTriggered 日志**之前**执行；异步形态＝触发日志随机器步先落、
//!   摸牌由 R 帧随后交付 → SkillTriggered(狂骨) 先于 CardsDrawn。
//!
//! R05-C 迁移后本文件必须全绿；若改回同步原语则相应测试转红。

use content_runtime::{import_extensions, InstallOptions, PackageStore};
use engine_native::content::extension::ExtensionPack;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, LogEvent, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action,
    })
}

fn repo_extensions_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn options() -> InstallOptions {
    InstallOptions {
        activate: true,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    }
}

/// 标记补充包（苦肉/狂骨宿主行为的承载包）。
fn ms_pack() -> ExtensionPack {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("mark-supplement-upstream-liuli/assets/cards.json"),
    )
    .expect("读标记补充包");
    content_native_loader::bridge_pack(&json).expect("桥接").pack
}

fn advance_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..96 {
        if e.turn().current == seat && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            return;
        }
        if let Some(c) = e.open_choice().cloned() {
            let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else if c.spec.allow_cancel {
                Vec::new()
            } else {
                c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
            };
            submit(
                e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            );
            continue;
        }
        submit(e, e.turn().current, GameAction::NextPhase);
    }
    panic!("未到 {seat} 号出牌阶段");
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = submit(
            e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        );
    }
}

// ---- 族 1：swd_status（森罗）——同步交错 vs 异步「先弃后摸」 ----

/// 森罗：其他有手牌角色各弃 2（不足全弃）+各摸 1。异步正确形态＝
/// 机器步内全部弃置完成，R 子树随后逐帧交付摸牌 → 日志中最后一张
/// CardDiscarded 先于第一张 CardsDrawn。同步 draw_cards 逐目标交错
/// （弃-摸-弃-摸）→ 红。
#[test]
fn r05c_senluo_discards_settle_before_async_draws() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shahai-swd")
        .expect("swd 安装")
        .clone();
    let json = std::fs::read_to_string(
        std::path::PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = content_native_loader::bridge_pack(&json).expect("swd 桥接").pack;
    let mut e = Engine::new_with_extensions(63101, 4, &[&pack]).expect("建局");
    e.debug_grant(0, "senluo");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // p0 手牌 1（并列最少）；p1 手牌 4；p2/p3 各 1（≥p0）——与既有森罗
    // 行为面同构：三个有手牌目标。
    let _keep0 = e.debug_place(0, "tao");
    let _f2 = e.debug_place(2, "tao");
    let _f3 = e.debug_place(3, "tao");
    for _ in 0..4 {
        e.debug_place(1, "tao");
    }
    let def = e.catalog().skill_by_name("senluo").expect("森罗在目录");
    let log_from = e.log().len();
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "森罗发动 {out:?}"
    );
    drain(&mut e);
    let slice = &e.log()[log_from..];
    let last_discard = slice
        .iter()
        .rposition(|l| matches!(l, LogEvent::CardDiscarded { .. }))
        .expect("森罗弃置日志");
    let first_draw = slice
        .iter()
        .position(|l| matches!(l, LogEvent::CardsDrawn { .. }))
        .expect("森罗摸牌日志");
    assert!(
        last_discard < first_draw,
        "异步形态：全部弃置先于全部摸牌（弃@{last_discard} 摸@{first_draw}）"
    );
    // 行为面不回归：各目标弃 2/1 摸 1 后手牌数与同步路径一致。
    assert_eq!(e.debug_hand(1).len(), 3, "p1 弃 2 摸 1（4→3）");
    assert_eq!(e.debug_hand(2).len(), 1, "p2 弃 1 摸 1（1→1）");
}

// ---- 族 2：standard skills（苦肉）——双枯竭 R 帧 Exhausted/detached ----

/// 苦肉：失去 1 点体力并摸 2。牌堆仅剩 1 张、弃堆清空：同步 draw_cards
/// 部分入手（+1 张）；异步 R 帧 held 1 张后双枯竭 → Exhausted，held 转
/// detached（§4）→ 手牌不变、无 CardsDrawn 日志、detached +1。同步 → 红。
#[test]
fn r05c_kurou_exhausted_draw_detaches_instead_of_partial_hand() {
    let mut e = Engine::new_with_extensions(63102, 4, &[&ms_pack()]).expect("建局");
    e.debug_grant(0, "kurou");
    advance_to_play(&mut e, 0);
    let hp0 = e.debug_hp(0);
    let hand0 = e.debug_hand(0).len();
    let detached0 = e.state_core().detached_cards.len();
    // 牌堆留 1 张、弃堆清空：摸 2 必然中途枯竭。
    let last = *e
        .state_core()
        .deck
        .last()
        .expect("advance 后牌堆非空");
    e.debug_drain_deck_to_discard();
    {
        let core = e.state_core_mut();
        core.discard.clear();
        core.deck.push(last);
    }
    let log_from = e.log().len();
    let def = e.catalog().skill_by_name("kurou").expect("kurou 在目录");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "苦肉发动 {out:?}"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(0), hp0 - 1, "失去 1 点体力（两路径一致）");
    assert_eq!(
        e.debug_hand(0).len(),
        hand0,
        "双枯竭：R 帧 Exhausted，held 全部 detached、不入手"
    );
    assert!(
        !e.log()[log_from..]
            .iter()
            .any(|l| matches!(l, LogEvent::CardsDrawn { .. })),
        "双枯竭无 gain：无 CardsDrawn 日志"
    );
    assert_eq!(
        e.state_core().detached_cards.len(),
        detached0 + 1,
        "held 1 张转 detached（Unclaimed）"
    );
}

// ---- 族 3：skills_recovery（狂骨）——触发日志与 R 帧交付的次序 ----

/// 狂骨选「摸 1」臂：同步 draw_cards 在 SkillTriggered 日志之前落
/// CardsDrawn；异步形态＝机器步先落触发日志，摸牌由 R 子树随后交付 →
/// SkillTriggered(狂骨) 先于 CardsDrawn。同步 → 红。
#[test]
fn r05c_kuanggu_trigger_log_precedes_async_draw_delivery() {
    let mut e = Engine::new_with_extensions(63103, 4, &[&ms_pack()]).expect("建局");
    e.debug_grant(0, "fz_xinkuanggu");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(0, 3);
    let hand0 = e.debug_hand(0).len();
    let log_from = e.log().len();
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    // 狂骨确认门 → 选择窗：选「摸 1」。
    let mut chose_draw = false;
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel = if c.spec.prompt == "trigger_confirm" {
            vec![ChoiceOption::Confirm]
        } else if c.spec.prompt == "fz_xinkuanggu_choice" {
            chose_draw = true;
            vec![ChoiceOption::Tag(1)]
        } else if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = submit(
            &mut e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        );
    }
    drain(&mut e);
    assert!(chose_draw, "狂骨选择窗已到达");
    let slice = &e.log()[log_from..];
    let skill_idx = slice
        .iter()
        .position(|l| matches!(l, LogEvent::SkillTriggered { skill: "狂骨", .. }))
        .expect("狂骨触发日志");
    let draw_idx = slice
        .iter()
        .position(|l| matches!(l, LogEvent::CardsDrawn { player: 0, .. }))
        .expect("狂骨摸 1 交付日志");
    assert!(
        skill_idx < draw_idx,
        "异步形态：触发日志先于 R 帧摸牌交付（技@{skill_idx} 摸@{draw_idx}）"
    );
    assert_eq!(e.debug_hand(0).len(), hand0 + 1, "摸 1 行为面不回归");
}
