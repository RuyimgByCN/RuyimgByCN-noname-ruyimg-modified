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

// ---------------------------------------------------------------------------
// R05-C2 批红守卫：续行位（continuation-slot）迁移的 4 个代表 caller
// （boss 孟婆 / 照妖 / 灵泉 / 扶仇·清算）。修前（同步路径）全部为红。
// ---------------------------------------------------------------------------

/// 仙剑包（照妖/灵泉宿主行为的承载包）。
fn xianjian_pack() -> ExtensionPack {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shaai-xianjian-upstream-noname/assets/cards.json"),
    )
    .expect("读仙剑包");
    content_native_loader::bridge_pack(&json).expect("桥接").pack
}

/// 玩点论杀包（扶仇·清算宿主行为的承载包）。
fn wudian_pack() -> ExtensionPack {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("wudian-upstream-noname/assets/cards.json"),
    )
    .expect("读玩点论杀包");
    content_native_loader::bridge_pack(&json).expect("桥接").pack
}

/// 牌堆只留 `keep` 张（取原牌堆顶）、弃堆清空——制造「摸牌中途枯竭/
/// 双枯竭」的确定性环境（苦肉守卫同款手法）。
fn exhaust_deck(e: &mut Engine, keep: usize) {
    let mut tops: Vec<u32> = Vec::new();
    for _ in 0..keep {
        if let Some(c) = e.state_core().deck.last().copied() {
            tops.push(c);
            e.state_core_mut().deck.pop();
        }
    }
    e.debug_drain_deck_to_discard();
    let core = e.state_core_mut();
    core.discard.clear();
    let mut deck = std::mem::take(&mut core.deck);
    deck.clear();
    for c in tops.into_iter().rev() {
        deck.push(c);
    }
    core.deck = deck;
}

// ---- C2-1：boss 孟婆 level 0 乱入——上游对照「不摸牌」零扰动 ----

/// 上游 boss.js:3439-3455（boss_newhuanren 开局）＝game.addFellow 只入座
/// 不发牌；directgain(4) 仅 level!=0 转关臂（boss.js:3575-3577）。原生
/// 旧实现「摸 4 再逐张取回压栈」虽恢复牌序/手牌，但** CardsDrawn 日志
/// 已落**＝对日志面有扰动。正确迁移＝开局臂删摸牌整段（零日志零扰动）；
/// 修前红（日志含孟婆座位 CardsDrawn），迁移后绿。
#[test]
fn r05c2_mengpo_level0_spawn_is_card_silent() {
    fn general_first(e: &Engine, name: &str) -> u16 {
        (0..e.catalog().general_count() as u16)
            .find(|&d| e.catalog().general_meta(d).map(|g| g.0) == Some(name))
            .unwrap_or_else(|| panic!("general {name}"))
    }
    fn boot(seed: u64) -> Option<(Engine, u8)> {
        let mut e = Engine::new_boss_stage(seed, 4, 0, 0, 4, 4, 0);
        let qinguang = general_first(&e, "秦广王");
        e.debug_set_general_def(0, qinguang);
        let luxun = general_first(&e, "陆逊");
        e.debug_set_general_def(1, luxun);
        let out = submit(&mut e, 0, GameAction::NextPhase);
        drain(&mut e);
        let _ = out;
        if !e.state_core().boss_mengpo_spawned() {
            return None;
        }
        let seat = (0..e.state_core().players.len() as u8)
            .find(|&s| {
                e.state_core().players[s as usize]
                    .general_slots[0]
                    .and_then(|id| e.catalog().general_meta(id).map(|g| g.0))
                    == Some("孟婆")
            })
            .expect("乱入后应找到孟婆座位");
        Some((e, seat))
    }
    let mut checked = 0;
    for seed in 1..81u64 {
        let Some((e, seat)) = boot(seed) else {
            continue;
        };
        checked += 1;
        assert!(
            e.debug_hand(seat).is_empty(),
            "seed {seed}：level 0 孟婆手牌为空（上游不摸牌）"
        );
        assert!(
            !e.log()
                .iter()
                .any(|l| matches!(l, LogEvent::CardsDrawn { player, .. } if *player == seat)),
            "seed {seed}：level 0 乱入零摸牌日志（上游 addFellow 无发牌；旧同步摸 4+回滚已落日志＝红）"
        );
        if checked >= 3 {
            break;
        }
    }
    assert!(checked >= 1, "1..80 应出现孟婆乱入（40% 门）");
}

// ---- C2-2：照妖——枯竭摸牌走 R 面（detached/不入手），询问仍开后置 ----

/// 照妖拼点赢摸 2：牌堆仅 1 张、弃堆空。同步 draw_cards＝那张牌直接入
/// 手（红：手牌 +1、有 CardsDrawn）；异步 R 帧＝held 1 张后双枯竭
/// Exhausted、转 detached、不入手、无 CardsDrawn 日志。置顶询问（min2
/// max2）仍正常打开（选项=预置 2 张）——摸后依赖逻辑不回归。
#[test]
fn r05c2_zhaoyao_exhausted_draw_detaches_ask_opens() {
    let mut e = Engine::new_with_extensions(63201, 3, &[&xianjian_pack()]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "zhaoyao");
    // 布阵保证拼点赢：0 号首张高点数 > 1 号首张低点数（直接从牌堆按点数
    // 挑牌，避免 debug_place 依牌库序的随机性；拼点只看首张点数）
    let rank_of = |e: &Engine, c: u32| e.catalog().cards[c as usize].rank;
    let take_low = |e: &mut Engine| -> u32 {
        let pos = e
            .state_core()
            .deck
            .iter()
            .position(|c| rank_of(e, *c) <= 5)
            .expect("牌堆有点数≤5 的牌");
        e.state_core_mut().deck.remove(pos)
    };
    let low = take_low(&mut e);
    e.state_core_mut().player_mut(1).hand.push(low);
    let low_rank = rank_of(&e, low);
    let take_high = |e: &mut Engine| -> u32 {
        let pos = e
            .state_core()
            .deck
            .iter()
            .position(|c| rank_of(e, *c) > low_rank)
            .expect("牌堆有点数更高的牌");
        e.state_core_mut().deck.remove(pos)
    };
    let high = take_high(&mut e);
    e.state_core_mut().player_mut(0).hand.push(high);
    // 0 号第二张（凑置顶询问的 2 个选项）
    let second = e.debug_place(0, "tao");
    let hand0 = e.debug_hand(0).len();
    debug_assert_eq!(hand0, 2);
    // 0 号出牌→弃→结束；1 号判定→摸牌阶段开始触发照妖。
    // 照妖原生触发在 1 号主摸 R 之后（登记时序），故留 3 张：主摸 2 张
    // 交付 1 号，照妖摸 2 时仅剩 1 张——同步＝第 3 张入手（红）；异步
    // R＝held 1 后枯竭转 detached、不入手。
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    exhaust_deck(&mut e, 3);
    let detached0 = e.state_core().detached_cards.len();
    let log_from = e.log().len();
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw：主摸+照妖
    // 照妖询问挂起（min2max2，选项=预置 2 张——摸 2 已枯竭不入手）
    let c = e
        .open_choice()
        .expect("照妖置顶询问仍打开（摸后依赖逻辑）")
        .clone();
    assert_eq!(
        c.spec.prompt,
        engine_native::content::host::WUDIAN_XJ_ZHAOYAO_CARDS
    );
    assert_eq!((c.spec.min, c.spec.max), (2, 2));
    let opts: Vec<u32> = c
        .spec
        .options
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Card(x) => Some(*x),
            _ => None,
        })
        .collect();
    assert!(
        opts.contains(&high) && opts.contains(&second),
        "选项含预置两张（枯竭摸牌未入手）"
    );
    assert_eq!(
        e.debug_hand(0).len(),
        hand0,
        "双枯竭：R 帧 Exhausted、held 1 张转 detached、不入手（同步 +1＝红）"
    );
    assert!(
        !e.log()[log_from..]
            .iter()
            .any(|l| matches!(l, LogEvent::CardsDrawn { player: 0, .. })),
        "照妖枯竭摸牌无 CardsDrawn 日志（同步有＝红）"
    );
    assert_eq!(
        e.state_core().detached_cards.len(),
        detached0 + 1,
        "照妖 R 帧 held 1 张转 detached（同步入手＝红）"
    );
    let _ = submit(
        &mut e,
        c.observer,
        GameAction::Answer {
            choice_id: c.id,
            selection: vec![ChoiceOption::Card(high), ChoiceOption::Card(second)],
        },
    );
    drain(&mut e);
}

// ---- C2-3：灵泉——枯竭摸 3 走 R 面（detached），置牌询问仍打开 ----

/// 灵泉觉醒摸 3+内置水云置牌：牌堆仅 1 张、弃堆空。同步＝那张牌入手
/// （红：手牌 +1、CardsDrawn、无 detached）；异步 R＝held 1 后枯竭转
/// detached、不入手、无日志。置牌询问（min1max1）仍以既有手牌打开。
#[test]
fn r05c2_lingquan_exhausted_detached_ask_opens() {
    let mut e = Engine::new_with_extensions(63202, 3, &[&xianjian_pack()]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "lingquan");
    // 觉醒门槛：round≥3 且 total_used > hp（hp 压 1、用牌计数 2）
    e.debug_set_hp(0, 1);
    e.state_core_mut().player_mut(0).total_used = 2;
    let keep = e.debug_place(0, "tao");
    let hand0 = e.debug_hand(0).len();
    let detached0 = e.state_core().detached_cards.len();
    e.debug_set_turn(0, Phase::End, 3);
    exhaust_deck(&mut e, 1);
    let log_from = e.log().len();
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→收束：灵泉觉醒
    assert!(
        e.state_core().player(0).lingquan_awakened,
        "觉醒标记已置（两路径一致）"
    );
    assert_eq!(
        e.debug_hand(0).len(),
        hand0,
        "双枯竭：held 1 张转 detached、不入手（同步 +1＝红）"
    );
    assert!(
        !e.log()[log_from..]
            .iter()
            .any(|l| matches!(l, LogEvent::CardsDrawn { player: 0, .. })),
        "灵泉枯竭摸牌无 CardsDrawn 日志（同步有＝红）"
    );
    assert_eq!(
        e.state_core().detached_cards.len(),
        detached0 + 1,
        "held 1 张转 detached（Unclaimed）"
    );
    // 内置水云置牌询问仍打开（选项=既有手牌，min1max1）
    let c = e
        .open_choice()
        .expect("灵泉置牌询问仍打开（摸后依赖逻辑）")
        .clone();
    assert_eq!(
        c.spec.prompt,
        engine_native::content::host::WUDIAN_XJ_LINGQUAN_CARDS
    );
    assert_eq!((c.spec.min, c.spec.max), (1, 1));
    assert!(c.spec.options.contains(&ChoiceOption::Card(keep)));
    let _ = submit(
        &mut e,
        c.observer,
        GameAction::Answer {
            choice_id: c.id,
            selection: vec![ChoiceOption::Card(keep)],
        },
    );
    drain(&mut e);
}

// ---- C2-4：扶仇·清算——双枯竭不 panic（虚拟杀跳过）、标记仍消耗 ----

/// 清算对标记来源「摸 1+虚拟杀」：牌堆弃堆全空。同步 draw_cards 摸 0
/// 后 use_virtual_card 取 deck.first() **panic**（红＝测试崩）；异步 R
/// 双枯竭 Exhausted（0 held、无 detached），续行步守卫牌堆空跳过虚拟杀
/// ——不 panic、不凭空造牌、标记照常消耗、无 CardsDrawn。
#[test]
fn r05c2_fuchou2_exhausted_no_panic_virtual_sha_skipped() {
    let mut e = Engine::new_with_extensions(63203, 4, &[&wudian_pack()]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(1, "fuchou2");
    // 直接置标记：0 号在 1 号的复仇标记里
    e.state_core_mut().player_mut(1).fuchou_marked = vec![0];
    exhaust_deck(&mut e, 0);
    let hand1 = e.debug_hand(1).len();
    let log_from = e.log().len();
    // 结束 0 号整回合（TurnEndWindow after）→ 清算触发
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let out = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge：清算
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "清算双枯竭不得 panic（同步 use_virtual_card 空堆 panic＝红）：{out:?}"
    );
    drain(&mut e);
    assert!(
        e.state_core().player(1).fuchou_marked.is_empty(),
        "清算：标记已消耗（死亡移除语义不回归）"
    );
    assert_eq!(
        e.debug_hand(1).len(),
        hand1,
        "双枯竭摸 0：手牌不变（0 held 无 detached）"
    );
    assert!(
        !e.log()[log_from..]
            .iter()
            .any(|l| matches!(l, LogEvent::CardsDrawn { player: 1, .. })),
        "清算枯竭摸牌无 CardsDrawn 日志"
    );
}
