//! PA12④「DSL/IR→原生」生成原型（2026-10-26）：**现有 ported .skill**（sp 包
//! `olyinji` 隐绩——结束阶段体力非唯一最高时 `choose(1,2)`：回复 1 体力或
//! 体力上限 +1，含选择与续行）经 content-compiler 编译 →
//! content-native-loader **正式桥接**（`bridge_vm_skill_decls`，数据驱动不
//! 重编译）→ engine-native 整局：阶段结束触发 → `rvm_choice` 挂起 → 作答
//! 续行 → 效果落账；挂起中 `save()` → `restore_with_extensions` → 续行，
//! 同一快照两个分支各自落账。旧线差分见
//! `crates/engine-core/tests/pa12_dsl_legacy_differential.rs`（同一模块经
//! 旧线 vm_host，效果与落账一致）。迁移计划：docs/architecture/native-dsl-
//! migration-plan.md；批量迁移前须用户确认（台帐 PA12④）。

use content_compiler::compile_source_named;
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::{ChoiceOption, Engine, GameAction, GameCommand};

/// 真实移植产物（非手写样本）：sp 包 olyinji.skill 原文。
const OLYINJI: &str = include_str!("../../../content/original/ported/sp/olyinji.skill");

fn load_via_loader() -> ExtensionPack {
    let pkg = compile_source_named("sp-olyinji-prototype", OLYINJI).expect("编译 ported .skill");
    assert!(
        pkg.manifest.rule_vm_abi_version <= rule_vm::RULE_VM_ABI_VERSION,
        "原型不引入新 ABI：包要求 {} > 宿主 {}",
        pkg.manifest.rule_vm_abi_version,
        rule_vm::RULE_VM_ABI_VERSION
    );
    let out = content_native_loader::bridge_vm_skill_decls(&pkg).expect("正式桥接路径");
    assert_eq!(out.decls.len(), 1, "单技能包应产出 1 条 VM 技能声明");
    ExtensionPack {
        def: ExtensionDef {
            name: "sp-olyinji-prototype",
            version: 1,
            description: "PA12④ DSL→原生 生成原型",
        },
        generals: Vec::new(),
        cards: Vec::new(),
        host_skills: Vec::new(),
        vm_skills: out.decls,
        init_hook: InitHook::None,
        enabled: true,
    }
}

fn submit(e: &mut Engine, actor: u8, action: GameAction) {
    let rev = e.revision();
    let _ = e.submit(GameCommand {
        actor,
        expected_revision: rev,
        dedup_id: (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64)
            | 1,
        action,
    });
}

/// 推进并自动结算普通询问；VM 挂起（prompt=rvm_choice）原样留给调用者。
fn advance_until_rvm_pause(e: &mut Engine, owner: u8) -> engine_native::OpenChoice {
    for _ in 0..400 {
        if e.outcome().is_some() {
            panic!("对局提前结束，未到 olyinji 挂起");
        }
        if let Some(c) = e.open_choice() {
            if c.spec.prompt == "rvm_choice" {
                assert_eq!(e.turn().current, owner, "挂起应发生在持有者回合");
                return c.clone();
            }
            let sel: Vec<ChoiceOption> = c
                .spec
                .options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Card(_)))
                .copied()
                .take(c.spec.min.max(1))
                .collect();
            let (cid, obs) = (c.id, c.observer);
            let sel = if !sel.is_empty() {
                sel
            } else {
                vec![c.spec.options[0].clone()]
            };
            submit(e, obs, GameAction::Answer { choice_id: cid, selection: sel });
            continue;
        }
        let cur = e.turn().current;
        submit(e, cur, GameAction::NextPhase);
    }
    panic!("400 步内未出现 rvm_choice 挂起");
}

fn answer(e: &mut Engine, c: &engine_native::OpenChoice, token: u8) {
    let rev = e.revision();
    let out = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: rev,
        dedup_id: (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64)
            | 1,
        action: GameAction::Answer {
            choice_id: c.id,
            selection: vec![ChoiceOption::Tag(token)],
        },
    });
    assert!(
        matches!(out, engine_native::CommandOutcome::Accepted { .. } | engine_native::CommandOutcome::Pending { .. }),
        "应答应被接受: {out:?}"
    );
}


/// 结算一个普通询问（按 min 取 Card 选项；无 Card 则取首选项）。
fn settle_plain(e: &mut Engine, c: &engine_native::OpenChoice) {
    let sel: Vec<ChoiceOption> = c
        .spec
        .options
        .iter()
        .filter(|o| matches!(o, ChoiceOption::Card(_)))
        .copied()
        .take(c.spec.min.max(1))
        .collect();
    let sel = if sel.is_empty() { vec![c.spec.options[0].clone()] } else { sel };
    submit(e, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
}

fn hp(e: &Engine, p: u8) -> i32 {
    e.debug_hp(p)
}

fn max_hp(e: &Engine, p: u8) -> u32 {
    e.state_core().player(p).max_hp
}

/// 场景：4 座（座位 0..3），1 号持「olyinji」，体力 3 与 2 号并列最高（0、3 号压到 1）。
fn scenario(seed: u64) -> (ExtensionPack, Engine) {
    let pack = load_via_loader();
    let name = pack.vm_skills[0].name.clone();
    let mut e = Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局");
    e.debug_grant(1, &name);
    e.debug_set_hp(1, 3);
    e.debug_set_hp(2, 3);
    e.debug_set_hp(0, 1);
    e.debug_set_hp(3, 1);
    assert_eq!((hp(&e, 1), max_hp(&e, 1)), (3, 4));
    (pack, e)
}

/// 分支 1：choose→回复 1 体力（3→4）。
#[test]
fn pa12_dsl_choice_recover_branch() {
    let (pack, mut e) = scenario(930_001);
    let c = advance_until_rvm_pause(&mut e, 1);
    let options: Vec<u32> = c
        .spec
        .options
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Tag(t) => Some(*t as u32),
            _ => None,
        })
        .collect();
    assert_eq!(options, vec![1, 2], "DSL choose(1,2) → 两个 Tag 选项");
    answer(&mut e, &c, 1);
    assert_eq!(hp(&e, 1), 4, "选 1=回复 1 体力");
    assert_eq!(max_hp(&e, 1), 4, "上限不变");
    drop(pack);
}

/// 分支 2：choose→体力上限 +1（4→5，体力保持 3）。
#[test]
fn pa12_dsl_choice_max_hp_branch() {
    let (_pack, mut e) = scenario(930_002);
    let c = advance_until_rvm_pause(&mut e, 1);
    answer(&mut e, &c, 2);
    assert_eq!(max_hp(&e, 1), 5, "选 2=体力上限 +1");
    assert_eq!(hp(&e, 1), 3, "体力不变");
}

/// 挂起中存档 → 新引擎恢复 → 续行；同一快照两个分支各自落账，原引擎不受影响。
#[test]
fn pa12_dsl_pause_save_restore_resume_both_branches() {
    let (pack, mut e) = scenario(930_003);
    let c = advance_until_rvm_pause(&mut e, 1);
    let snap = e.save();

    let mut e1 = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复（分支 1）");
    let c1 = e1
        .open_choice()
        .expect("恢复后挂起询问仍在（active_ask 派生同一选择）")
        .clone();
    assert_eq!(c1.spec.prompt, "rvm_choice");
    answer(&mut e1, &c1, 1);
    assert_eq!(hp(&e1, 1), 4, "恢复后续行选 1=回复 1");

    let mut e2 = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复（分支 2）");
    let c2 = e2.open_choice().expect("同快照再恢复").clone();
    assert_eq!(c2.spec.prompt, "rvm_choice");
    answer(&mut e2, &c2, 2);
    assert_eq!(max_hp(&e2, 1), 5, "同一快照另一分支=上限 +1");

    answer(&mut e, &c, 1);
    assert_eq!(hp(&e, 1), 4, "快照不改变原引擎，续行同样落账");
}

/// 滤镜语义（体力非唯一最高才触发）：唯一最高时不挂起。
#[test]
fn pa12_dsl_filter_not_fired_when_unique_max() {
    let (_pack, mut e) = scenario(930_004);
    e.debug_set_hp(2, 2); // 1 号 3 成为唯一最高
    for _ in 0..400 {
        if e.outcome().is_some() {
            break;
        }
        if let Some(c) = e.open_choice() {
            assert_ne!(
                c.spec.prompt, "rvm_choice",
                "唯一最高体力时 olyinji 不应触发"
            );
            let c = c.clone();
            settle_plain(&mut e, &c);
            continue;
        }
        let cur = e.turn().current;
        submit(&mut e, cur, GameAction::NextPhase);
    }
    assert_eq!(hp(&e, 1), 3, "未触发，体力不变");
}

/// 触发时机与次数（PA12④ 发现的两处桥接缺陷的回归）：
/// ① 旧线 `phase_end` = 进入结束阶段——原生经 `AtPhase(End)` 门只在 End 阶段开头问
///   （修前：任一阶段结束都触发，每回合 5 次）；
/// ② 首答即续行（修前：`run()` 置 `asked=false` 导致再问一次、首答丢失）。
#[test]
fn pa12_dsl_fires_once_per_turn_at_end_phase_and_first_answer_lands() {
    let (_pack, mut e) = scenario(930_009);
    let c = advance_until_rvm_pause(&mut e, 1);
    assert_eq!(e.turn().phase, engine_native::Phase::End, "挂起发生在结束阶段");
    answer(&mut e, &c, 2); // 上限 +1，体力 3 保持与 2 号并列最高 → 下回合应再问
    assert_eq!(max_hp(&e, 1), 5, "首答即续行落账（不再二次询问）");
    assert!(
        e.open_choice().map(|c| c.spec.prompt != "rvm_choice").unwrap_or(true),
        "作答后无二次 rvm_choice"
    );
    // 继续推进到 1 号的下一个结束阶段：期间 rvm_choice 只能出现在 1 号 End 阶段，且恰好 1 次。
    let mut asks = 0;
    let mut steps = 0;
    while asks < 1 && steps < 400 {
        steps += 1;
        assert!(e.outcome().is_none(), "对局不应结束（仅推进阶段）");
        if let Some(c) = e.open_choice() {
            let c = c.clone();
            if c.spec.prompt == "rvm_choice" {
                asks += 1;
                assert_eq!(e.turn().current, 1, "只在持有者回合");
                assert_eq!(e.turn().phase, engine_native::Phase::End, "只在结束阶段");
                answer(&mut e, &c, 2);
                continue;
            }
            settle_plain(&mut e, &c);
            continue;
        }
        let cur = e.turn().current;
        submit(&mut e, cur, GameAction::NextPhase);
    }
    assert_eq!(asks, 1, "下一回合结束阶段恰好再问一次（{steps} 步）");
    assert_eq!(max_hp(&e, 1), 6);
    assert_eq!(hp(&e, 1), 3);
}
