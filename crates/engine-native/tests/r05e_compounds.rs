//! R05-E（裁决 28 合同 v3 §2/§5）：三固定 compound（涕泣/宝梳/避祸）与
//! 四角色私有 helper 派生的分支矩阵验收。
//!
//! 覆盖（每技能）：正常路径、取消路径、marks 有符号/0 保留、身份隔离
//!（两个包不串槽）；另验：n==2/排除自己（涕泣）、m=0→+1 与 m<0（宝梳）、
//! Source −1/End +1 抵消 Some0、Next→Current 原子 take（避祸）、通用
//! AddMark 碰派生 helper 键被拒、装载器派生确定性与角色面。
//!
//! 触发面由测试包声明（npack trigger_index；DSL 词汇表未含三 compound，
//! 模块字节编译后替换为单 resolver 体——与胆守 `active_eligible` 认证的
//! 形状一致）。

use engine_native::content::extension::{ExtensionDef, ExtensionPack};
use engine_native::{ChoiceOption as O, CommandOutcome, Engine, GameAction, GameCommand, OpenChoice, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn must(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    let outcome = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action,
    });
    assert!(
        !matches!(outcome, CommandOutcome::Rejected(_)),
        "命令意外被拒: {outcome:?}"
    );
    outcome
}

/// 单 resolver compound 模块：`HOST_SELF, HOST_SKILL, HOST_EFFECT_EXT(tag),
/// HALT`（弹栈序 player, skill——与胆守编译产物同形）。
fn compound_module(name: &str, tag: u16) -> rule_vm::BytecodeModule {
    use rule_vm::bytecode::{caps, assemble, Op};
    let code = assemble(&[Op::HostSelf, Op::HostSkill, Op::HostEffect(tag), Op::Halt]);
    let caps_mask = caps::EMIT_EFFECTS;
    rule_vm::BytecodeModule {
        name: name.to_string(),
        logic_hash: rule_vm::BytecodeModule::compute_logic_hash(name, &code, 0, caps_mask, 0),
        code,
        local_count: 0,
        max_stack: 2,
        entry_pc: 0,
        caps_mask,
    }
}

/// 把包内指定触发/主动声明的模块替换为 compound 单 resolver 体，并可改写
/// 触发事件名（R05-E 装载面：`draw_phase_after`/`damage_source`/`damage_end`）。
fn retrigger(pkg: &mut content_schema::ContentPackage, tag: u16, events: &[&str]) {
    assert_eq!(pkg.trigger_index.len(), events.len());
    for (entry, event) in pkg.trigger_index.iter_mut().zip(events) {
        let name = entry.module.clone();
        let module = pkg
            .modules
            .iter_mut()
            .find(|m| m.name == name)
            .expect("trigger module in package");
        *module = compound_module(&name, tag);
        entry.event = event.to_string();
    }
    for active in &pkg.active_index {
        let name = active.module.clone();
        let module = pkg
            .modules
            .iter_mut()
            .find(|m| m.name == name)
            .expect("active module in package");
        *module = compound_module(&name, tag);
    }
}

/// 编译声明壳（DSL 触发段 + emit_note 占位）并替换为 compound 模块。
/// `triggers` 三元组＝(DSL 壳事件, filter, 改写后事件名)。
fn compile_compound(
    package: &str,
    skill: &str,
    skill_id: u64,
    tag: u16,
    triggers: &[(&str, &str, &str)],
) -> content_schema::ContentPackage {
    let body: String = triggers
        .iter()
        .map(|(event, filter, _)| {
            format!("  trigger {event} {filter} {{\n    effect emit_note(0);\n  }}\n")
        })
        .collect();
    let source = format!("skill {skill}({skill_id}) {{\n{body}}}");
    let mut pkg = content_compiler::compile_source_named(package, &source)
        .unwrap_or_else(|e| panic!("编译 {skill} 失败: {e:?}"));
    let rewritten: Vec<&str> = triggers.iter().map(|(_, _, event)| *event).collect();
    retrigger(&mut pkg, tag, &rewritten);
    // 模块替换后重算聚合 logic_hash（manifest 校验一致性）。
    pkg.manifest.logic_hash = pkg.compute_logic_hash();
    pkg
}

fn pack_of(general: &str, package: &str, pkg: &content_schema::ContentPackage) -> ExtensionPack {
    let decls = content_native_loader::bridge_vm_skill_decls(pkg)
        .unwrap_or_else(|e| panic!("桥接 {package} 失败: {e:?}"))
        .decls;
    let known = decls.iter().map(|d| d.name.clone()).collect();
    // 武将只授主技能；派生 helper 由 compound 机在目标上装（§2）。
    let granted: Vec<_> = decls
        .iter()
        .filter(|d| !d.name.contains("#r05:"))
        .map(|d| serde_json::json!({"name": d.name}))
        .collect();
    let generals = content_native_loader::bridge_generals_with_known(
        &serde_json::json!({
            "pack": {"name": package, "version": 1},
            "generals": [{"name": general, "hp": 4, "faction": "qun", "gender": "male",
                "skills": granted }]
        })
        .to_string(),
        &known,
    )
    .unwrap()
    .generals;
    let name = engine_native::content::names::intern(package.to_string());
    ExtensionPack {
        def: ExtensionDef {
            name,
            version: 1,
            description: "R05-E compound",
        },
        generals,
        cards: vec![],
        host_skills: vec![],
        vm_skills: decls,
        init_hook: Default::default(),
        enabled: true,
    }
}

/// 引擎泵：有开放询问先按脚本（rvm_* 前缀必须命中，否则按默认应答），
/// 无询问推进一次阶段。返回本次是否真的提交了 NextPhase。
fn pump<F>(e: &mut Engine, mut script: F) -> bool
where
    F: FnMut(&OpenChoice) -> Option<Vec<O>>,
{
    if let Some(open) = e.open_choice().cloned() {
        let selection = if open.spec.prompt.starts_with("rvm_") {
            script(&open).unwrap_or_else(|| panic!("未脚本化的 rvm 询问: {}", open.spec.prompt))
        } else {
            default_answer(&open)
        };
        must(e, open.observer, GameAction::Answer {
            choice_id: open.id,
            selection,
        });
        return false;
    }
    let actor = e.turn().current;
    must(e, actor, GameAction::NextPhase);
    true
}

fn default_answer(open: &OpenChoice) -> Vec<O> {
    if open.spec.options.contains(&O::Pass) {
        vec![O::Pass]
    }
    // 无 Pass 但可取消（RespondCard 无闪等）：取消。
    else if open.spec.allow_cancel {
        vec![]
    } else if open.spec.options.len() >= open.spec.min {
        open.spec.options.iter().take(open.spec.min).copied().collect()
    } else {
        panic!("无法默认应答: {:?}", open.spec)
    }
}

/// 推进直到谓词成立（每步先清询问）。
fn run_until<F, P>(e: &mut Engine, script: F, pred: P)
where
    F: FnMut(&OpenChoice) -> Option<Vec<O>>,
    P: Fn(&Engine) -> bool,
{
    let mut script = script;
    for _ in 0..256 {
        if pred(e) {
            return;
        }
        pump(e, |c| script(c));
    }
    let open = e
        .open_choice()
        .map(|c| (c.spec.prompt.to_string(), c.observer))
        .unwrap_or_else(|| ("无".to_string(), 255));
    panic!(
        "run_until 超预算: turn={:?}/{:?} open={open:?}",
        e.turn().current, e.turn().phase
    );
}

fn at_phase(e: &Engine, player: u8, phase: Phase) -> bool {
    e.turn().current == player && e.turn().phase == phase
}

fn mark(e: &Engine, seat: u8, def: u32) -> Option<i32> {
    e.state_core().vm_aux.marks.get(&(seat, def)).copied()
}

fn helper_def(e: &Engine, name: &str) -> u32 {
    e.catalog().skill_by_name(name).expect("派生 helper 入目录") as u32
}

// ---- 涕泣 ----

fn tiqi_engine(package: &str, skill: &str, seed: u64) -> Engine {
    let pkg = compile_compound(
        package,
        skill,
        998001001,
        781,
        &[("phase_draw_end", "owner_alive", "draw_phase_after")],
    );
    let pack = pack_of("author", package, &pkg);
    let mut e = Engine::new_with_extensions(seed, 3, &[&pack]).unwrap();
    e.assign_general_named(0, "author").unwrap();
    e
}

/// 牌库/弃牌堆双枯竭——下一位摸牌角色 R 帧 Exhausted、本次无 gain，
/// live 读 n=0（≠2 → d=2）。双枯竭的部分 held 转 detached、不入手。
fn deck_exhausted(e: &mut Engine) {
    e.debug_drain_deck_to_discard();
    e.state_core_mut().discard.clear();
}

#[test]
fn r05e_tiqi_confirm_draw_and_negative_limit() {
    let mut e = tiqi_engine("r05e-tiqi-a", "r05e_tiqi_a", 33901);
    // 走完 seat0 首回合（自己 D 被排除，无询问）。
    run_until(&mut e, |_| None, |e| at_phase(e, 1, Phase::Judge));
    deck_exhausted(&mut e);
    eprintln!(
        "AFTER-SETUP turn={:?}/{:?} deck={} discard={}",
        e.turn().current, e.turn().phase, e.state_core().deck.len(), e.state_core().discard.len()
    );
    let mut asked: Vec<&'static str> = Vec::new();
    run_until(
        &mut e,
        |c| {
            asked.push(match c.spec.prompt {
                "rvm_tiqi_confirm" => {
                    assert_eq!(c.observer, 0);
                    "confirm"
                }
                "rvm_tiqi_sign" => "sign",
                other => panic!("意外询问 {other}"),
            });
            match c.spec.prompt {
                "rvm_tiqi_confirm" => Some(vec![O::Confirm]),
                _ => Some(vec![O::Tag(2)]), // −d
            }
        },
        |e| at_phase(e, 1, Phase::Play),
    );
    assert_eq!(asked, vec!["confirm", "sign"], "确认→符号两问");
    let def = helper_def(&e, "r05e_tiqi_a#r05:limit");
    assert_eq!(mark(&e, 1, def), Some(-2), "n=0 → d=2 → 选 −d：Limit=-2");
    assert_eq!(
        e.state_core().players[1].hand.len(),
        4,
        "双枯竭无 gain：本 D n=0"
    );
}

#[test]
fn r05e_tiqi_cancel_confirm_and_cancel_sign_keep_no_slot() {
    // 确认即取消：无符号询问、无槽。
    let mut e = tiqi_engine("r05e-tiqi-b", "r05e_tiqi_b", 33902);
    run_until(&mut e, |_| None, |e| at_phase(e, 1, Phase::Judge));
    deck_exhausted(&mut e);
    let mut sign_seen = false;
    run_until(
        &mut e,
        |c| match c.spec.prompt {
            "rvm_tiqi_confirm" => Some(vec![]),
            "rvm_tiqi_sign" => {
                sign_seen = true;
                Some(vec![])
            }
            other => panic!("意外询问 {other}"),
        },
        |e| at_phase(e, 1, Phase::Play),
    );
    assert!(!sign_seen, "确认取消后不应再问符号");
    let def = helper_def(&e, "r05e_tiqi_b#r05:limit");
    assert_eq!(mark(&e, 1, def), None);
}

#[test]
fn r05e_tiqi_sign_cancel_keeps_drawn_and_writes_nothing() {
    let mut e = tiqi_engine("r05e-tiqi-c", "r05e_tiqi_c", 33903);
    run_until(&mut e, |_| None, |e| at_phase(e, 1, Phase::Judge));
    deck_exhausted(&mut e);
    run_until(
        &mut e,
        |c| match c.spec.prompt {
            "rvm_tiqi_confirm" => Some(vec![O::Confirm]),
            "rvm_tiqi_sign" => Some(vec![]),
            other => panic!("意外询问 {other}"),
        },
        |e| at_phase(e, 1, Phase::Play),
    );
    let def = helper_def(&e, "r05e_tiqi_c#r05:limit");
    assert_eq!(mark(&e, 1, def), None, "取消符号：不写 Limit");
}

#[test]
fn r05e_tiqi_n_equals_two_and_own_draw_are_noop() {
    let mut e = tiqi_engine("r05e-tiqi-d", "r05e_tiqi_d", 33904);
    // 完整走完 seat0 与 seat1 的回合（各自 D n=2）：全程无 rvm 询问。
    let mut prompts = 0;
    run_until(&mut e, |_| {
        prompts += 1;
        None
    }, |e| at_phase(e, 2, Phase::Judge));
    assert_eq!(prompts, 0, "n==2 与自己的 D 均不询问");
    let def = helper_def(&e, "r05e_tiqi_d#r05:limit");
    assert_eq!(mark(&e, 0, def), None);
    assert_eq!(mark(&e, 1, def), None);
}

#[test]
fn r05e_tiqi_two_packages_do_not_share_slots() {
    let pkg_a = compile_compound(
        "r05e-tiqi-x",
        "r05e_tiqi_ax",
        998001011,
        781,
        &[("phase_draw_end", "owner_alive", "draw_phase_after")],
    );
    let pkg_b = compile_compound(
        "r05e-tiqi-y",
        "r05e_tiqi_by",
        998001012,
        781,
        &[("phase_draw_end", "owner_alive", "draw_phase_after")],
    );
    let (pack_a, pack_b) = (pack_of("authorA", "r05e-tiqi-x", &pkg_a), pack_of("authorB", "r05e-tiqi-y", &pkg_b));
    let mut e = Engine::new_with_extensions(33905, 3, &[&pack_a, &pack_b]).unwrap();
    e.assign_general_named(0, "authorA").unwrap();
    e.assign_general_named(1, "authorB").unwrap();
    run_until(&mut e, |_| None, |e| at_phase(e, 2, Phase::Judge));
    deck_exhausted(&mut e);
    run_until(
        &mut e,
        |c| match c.spec.prompt {
            "rvm_tiqi_confirm" => Some(vec![O::Confirm]),
            "rvm_tiqi_sign" => Some(vec![O::Tag(1)]), // +d
            other => panic!("意外询问 {other}"),
        },
        |e| at_phase(e, 2, Phase::Play),
    );
    let def_a = helper_def(&e, "r05e_tiqi_ax#r05:limit");
    let def_b = helper_def(&e, "r05e_tiqi_by#r05:limit");
    assert_ne!(def_a, def_b, "两个逻辑身份派生不同 def");
    assert_eq!(mark(&e, 2, def_a), Some(2), "A 包（owner=0）n=0→d=2→+2");
    assert_eq!(mark(&e, 2, def_b), Some(2), "B 包（owner=1）写自己的槽");
}

// ---- 宝梳 ----

fn baoshu_engine(package: &str, skill: &str, skill_id: u64, seed: u64) -> Engine {
    let pkg = compile_compound(
        package,
        skill,
        skill_id,
        782,
        &[("phase_prepare", "owner_is_target", "phase_prepare")],
    );
    let pack = pack_of("author", package, &pkg);
    let mut e = Engine::new_with_extensions(seed, 3, &[&pack]).unwrap();
    e.assign_general_named(0, "author").unwrap();
    e
}

/// 推进到 seat0 下一回合的准备阶段（首回合从判定开始，无准备）。
fn to_own_prepare(e: &mut Engine) {
    run_until(e, |_| None, |e| at_phase(e, 0, Phase::Prepare));
}

#[test]
fn r05e_baoshu_cost_links_turns_over_and_combs_positive() {
    let mut e = baoshu_engine("r05e-baoshu-a", "r05e_baoshu_a", 998002001, 33911);
    to_own_prepare(&mut e);
    // 目标状态：1 号横置、2 号翻面。
    e.state_core_mut().player_mut(1).linked = true;
    e.debug_set_face_down(2, true);
    run_until(
        &mut e,
        |c| match c.spec.prompt {
            "rvm_baoshu_cost" => Some(vec![O::Player(1), O::Player(2)]),
            other => panic!("意外询问 {other}"),
        },
        |e| !at_phase(e, 0, Phase::Prepare) && e.turn().current == 0,
    );
    let def = helper_def(&e, "r05e_baoshu_a#r05:combs");
    // maxHp=4、k=2 → m=3：两目标各 +3；仅当前 linked 才解除、仅当前
    // face_down 才翻回。
    assert_eq!(mark(&e, 1, def), Some(3));
    assert_eq!(mark(&e, 2, def), Some(3));
    assert!(!e.state_core().player(1).linked, "横置目标被解除");
    assert!(!e.state_core().player(2).face_down, "翻面目标被翻回");
    // isIn 才装 helper：两目标都装上 Combs helper 实例。
    assert!(e.state_core().skills.contains_key(&(1, def as u16)));
    assert!(e.state_core().skills.contains_key(&(2, def as u16)));
}

#[test]
fn r05e_baoshu_cost_cancel_writes_nothing() {
    let mut e = baoshu_engine("r05e-baoshu-b", "r05e_baoshu_b", 998002002, 33912);
    to_own_prepare(&mut e);
    e.state_core_mut().player_mut(1).linked = true;
    run_until(
        &mut e,
        |c| match c.spec.prompt {
            "rvm_baoshu_cost" => Some(vec![]),
            other => panic!("意外询问 {other}"),
        },
        |e| e.turn().current == 0 && e.turn().phase == Phase::Judge,
    );
    let def = helper_def(&e, "r05e_baoshu_b#r05:combs");
    assert_eq!(mark(&e, 1, def), None);
    assert!(e.state_core().player(1).linked, "取消不改横置");
}

#[test]
fn r05e_baoshu_m_zero_and_negative_via_inter_window_max_hp_change() {
    // 反例面（合同 v3 §5 Finding 4）：成本与 content 之间经 SkillLog 窗
    // 改写 maxHp——m 不能在选择时冻结。主技＝宝梳；同包副技两个
    // skill_log_begin handler 各 −1 maxHp（真实 change_max_hp 提交）。
    // 轮序（F 批 Begin2 消费后）：先 m=−1 轮（负条目不被目标自己的
    // Begin2 消费），后 m=0→+1 轮——+1 与保留的 −1 抵消成 Some0。
    let source = "skill r05e_baoshu_c(998002003) {\n  trigger phase_prepare owner_is_target {\n    effect emit_note(0);\n  }\n}\nskill r05e_drop_c(998002004) {\n  trigger hp_lost owner_alive {\n    effect emit_note(0);\n  }\n  trigger hp_lost owner_alive {\n    effect emit_note(0);\n  }\n}\n";
    let mut pkg = content_compiler::compile_source_named("r05e-baoshu-c", source).unwrap();
    // 主技模块 → 宝梳 compound；副技两个 handler → change_max_hp(−1)。
    use rule_vm::bytecode::{caps, assemble, Op};
    for entry in pkg.trigger_index.iter_mut() {
        let name = entry.module.clone();
        let module = pkg.modules.iter_mut().find(|m| m.name == name).unwrap();
        if name.starts_with("r05e_baoshu_c") {
            *module = compound_module(&name, 782);
        } else {
            let code = assemble(&[
                Op::HostSelf,
                Op::ConstI32(-1),
                Op::HostEffect(5), // CHANGE_MAX_HP
                Op::Halt,
            ]);
            *module = rule_vm::BytecodeModule {
                name: name.clone(),
                logic_hash: rule_vm::BytecodeModule::compute_logic_hash(
                    &name, &code, 0, caps::EMIT_EFFECTS, 0,
                ),
                code,
                local_count: 0,
                max_stack: 2,
                entry_pc: 0,
                caps_mask: caps::EMIT_EFFECTS,
            };
            entry.event = "skill_log_begin".into();
        }
    }
    pkg.manifest.logic_hash = pkg.compute_logic_hash();
    let pack = pack_of("author", "r05e-baoshu-c", &pkg);
    let mut e = Engine::new_with_extensions(33913, 3, &[&pack]).unwrap();
    e.assign_general_named(0, "author").unwrap();
    let def = helper_def(&e, "r05e_baoshu_c#r05:combs");
    // 推进到 seat2 回合（seat0 下一回合前），布 maxHp=3。
    run_until(&mut e, |_| None, |e| at_phase(e, 2, Phase::Judge));
    e.debug_set_max_hp(0, 3);
    e.debug_set_hp(0, 3);
    // 第一轮：k=2 → SkillLog 窗 −2 → content maxHp=1 → m=0 → delta=+1。
    run_until(
        &mut e,
        |c| match c.spec.prompt {
            "rvm_baoshu_cost" => Some(vec![O::Player(1), O::Player(2)]),
            other => panic!("意外询问 {other}"),
        },
        |e| mark(e, 1, def) == Some(1),
    );
    assert_eq!(mark(&e, 1, def), Some(1), "m==0 → +1");
    assert_eq!(mark(&e, 2, def), Some(1));
    assert_eq!(e.state_core().players[0].max_hp, 1, "窗口间 maxHp 已改写");
    // 第二轮：maxHp 重置 3、k=3 → −2 → content maxHp=1 → m=−1 →
    // delta=−1：与上一轮 +1 抵消成 Some0 保留。
    e.debug_set_max_hp(0, 3);
    e.debug_set_hp(0, 3);
    // 第一轮（m<0 面）：k=3 → SkillLog 窗 −2 → content maxHp=1 →
    // m=1+1−3=−1 → delta=−1。负条目不消费（§5「stored≤0 不触发、不
    // 消费」）：中间两轮 pumping 经过 seat1/seat2 自己的摸牌 Begin2，
    // −1 槽原样保留（下轮断言兼证）。
    run_until(
        &mut e,
        |c| match c.spec.prompt {
            "rvm_baoshu_cost" => Some(vec![O::Player(1), O::Player(2), O::Player(0)]),
            other => panic!("意外询问 {other}"),
        },
        |e| mark(e, 1, def) == Some(-1),
    );
    assert_eq!(mark(&e, 1, def), Some(-1), "m<0：负条目保留");
    assert_eq!(mark(&e, 2, def), Some(-1));
    assert_eq!(e.state_core().players[0].max_hp, 1, "窗口间 maxHp 已改写");
    // 第二轮（m==0 面）：maxHp 重置 3、k=2 → −2 → content maxHp=1 →
    // m=0 → delta=+1：与保留的 −1 抵消成 Some0 保留（含 0 不删）。
    e.debug_set_max_hp(0, 3);
    e.debug_set_hp(0, 3);
    run_until(
        &mut e,
        |c| match c.spec.prompt {
            "rvm_baoshu_cost" => Some(vec![O::Player(1), O::Player(2)]),
            other => panic!("意外询问 {other}"),
        },
        |e| mark(e, 1, def) == Some(0),
    );
    assert_eq!(mark(&e, 1, def), Some(0), "+1 加 −1 → Some0 保留");
    assert_eq!(mark(&e, 2, def), Some(0));
}

#[test]
fn r05e_baoshu_two_packages_do_not_share_slots() {
    let pkg_a = compile_compound(
        "r05e-baoshu-x",
        "r05e_baoshu_ax",
        998002011,
        782,
        &[("phase_prepare", "owner_is_target", "phase_prepare")],
    );
    let pkg_b = compile_compound(
        "r05e-baoshu-y",
        "r05e_baoshu_by",
        998002012,
        782,
        &[("phase_prepare", "owner_is_target", "phase_prepare")],
    );
    let (pack_a, pack_b) = (pack_of("authorA", "r05e-baoshu-x", &pkg_a), pack_of("authorB", "r05e-baoshu-y", &pkg_b));
    let mut e = Engine::new_with_extensions(33914, 3, &[&pack_a, &pack_b]).unwrap();
    e.assign_general_named(0, "authorA").unwrap();
    e.assign_general_named(1, "authorB").unwrap();
    // 首回合（seat0）无准备阶段：B（owner=1）先在自己回合触发，A
    //（owner=0）到下一轮自己的准备阶段触发。
    run_until(
        &mut e,
        |c| match c.spec.prompt {
            "rvm_baoshu_cost" => Some(vec![O::Player(2)]),
            other => panic!("意外询问 {other}"),
        },
        |e| mark(e, 2, helper_def(e, "r05e_baoshu_by#r05:combs")) == Some(4),
    );
    let def_b = helper_def(&e, "r05e_baoshu_by#r05:combs");
    assert_eq!(mark(&e, 2, def_b), Some(4), "B 包 k=1→m=4");
    run_until(
        &mut e,
        |c| match c.spec.prompt {
            "rvm_baoshu_cost" => Some(vec![O::Player(2)]),
            other => panic!("意外询问 {other}"),
        },
        |e| mark(e, 2, helper_def(e, "r05e_baoshu_ax#r05:combs")) == Some(4),
    );
    let def_a = helper_def(&e, "r05e_baoshu_ax#r05:combs");
    assert_ne!(def_a, def_b);
    assert_eq!(mark(&e, 2, def_a), Some(4), "A 包独立写自己的槽");
    // F 批（§5）生命周期：B 写入 seat2 的 +4 在 seat2 自己的摸牌
    // Begin2 被消费清槽（正值→pending、槽清、helper 移除）——发生在
    // A 发动（seat0 下一轮 Prepare）之前的 pumping 里。隔离性不受影响：
    // A/B 各写各的 def；消费只作用于目标本人的正 Combs 槽，A 后写的
    // 槽（写于 seat2 该次摸牌之后）原样保留。
    assert_eq!(mark(&e, 2, def_b), None, "B 的 +4 已被 seat2 自己的 Begin2 消费");
}

// ---- 避祸 ----

fn dcbihuo_engine(package: &str, skill: &str, skill_id: u64, seed: u64) -> Engine {
    let pkg = compile_compound(
        package,
        skill,
        skill_id,
        783,
        &[
            ("damage", "owner_alive", "damage_source"),
            ("hp_lost", "owner_alive", "damage_end"),
            ("phase_begin", "owner_alive", "phase_begin"),
        ],
    );
    let pack = pack_of("author", package, &pkg);
    let mut e = Engine::new_with_extensions(seed, 3, &[&pack]).unwrap();
    e.assign_general_named(0, "author").unwrap();
    e
}

/// seat0 出杀 seat1（seat1 无手牌不能闪）——触发真实 Source/End 窗。
fn sha_once(e: &mut Engine) {
    let sha = e.debug_place(0, "sha");
    e.debug_clear_hand(1);
    must(e, 0, GameAction::PlayCard {
        card: sha,
        targets: vec![1],
    });
}

#[test]
fn r05e_dcbihuo_source_minus_end_plus_keeps_zero_then_take_moves_current() {
    let mut e = dcbihuo_engine("r05e-dcbihuo-a", "r05e_dcbihuo_a", 998003001, 33921);
    run_until(&mut e, |_| None, |e| at_phase(e, 0, Phase::Play));
    sha_once(&mut e);
    let mut windows: Vec<&'static str> = Vec::new();
    run_until(
        &mut e,
        |c| match c.spec.prompt {
            "rvm_dcbihuo_target" => {
                assert_eq!(c.observer, 0);
                windows.push("ask");
                Some(vec![O::Player(1)])
            }
            other => panic!("意外询问 {other}"),
        },
        |e| !at_phase(e, 0, Phase::Play),
    );
    assert_eq!(windows.len(), 2, "Source 与 End 各一问");
    let next_def = helper_def(&e, "r05e_dcbihuo_a#r05:next");
    assert_eq!(mark(&e, 1, next_def), Some(0), "−1+1 抵消：Some0 保留");
    // 推进到 seat1 回合开始：原子 take——Next 移除、Current 写入 0。
    run_until(&mut e, |_| None, |e| at_phase(e, 1, Phase::Judge));
    let current_def = helper_def(&e, "r05e_dcbihuo_a#r05:current");
    assert_eq!(mark(&e, 1, next_def), None, "Next 被 take 移除");
    assert_eq!(mark(&e, 1, current_def), Some(0), "Current 含 0 条目");
    assert!(e.state_core().skills.contains_key(&(1, current_def as u16)));
}

#[test]
fn r05e_dcbihuo_cancel_end_window_keeps_minus_then_take_moves_minus() {
    let mut e = dcbihuo_engine("r05e-dcbihuo-b", "r05e_dcbihuo_b", 998003002, 33922);
    run_until(&mut e, |_| None, |e| at_phase(e, 0, Phase::Play));
    sha_once(&mut e);
    let mut first = true;
    run_until(
        &mut e,
        |c| match c.spec.prompt {
            "rvm_dcbihuo_target" => {
                // Source 窗选目标，End 窗取消。
                let answer = if first { vec![O::Player(1)] } else { vec![] };
                first = false;
                Some(answer)
            }
            other => panic!("意外询问 {other}"),
        },
        |e| !at_phase(e, 0, Phase::Play),
    );
    let next_def = helper_def(&e, "r05e_dcbihuo_b#r05:next");
    assert_eq!(mark(&e, 1, next_def), Some(-1), "仅 Source −1 生效");
    run_until(&mut e, |_| None, |e| at_phase(e, 1, Phase::Judge));
    let current_def = helper_def(&e, "r05e_dcbihuo_b#r05:current");
    assert_eq!(mark(&e, 1, current_def), Some(-1), "take 写 Current=-1");
    assert_eq!(mark(&e, 1, next_def), None);
}

#[test]
fn r05e_dcbihuo_two_packages_do_not_share_slots() {
    let pkg_a = compile_compound(
        "r05e-dcbihuo-x",
        "r05e_dcbihuo_ax",
        998003011,
        783,
        &[
            ("damage", "owner_alive", "damage_source"),
            ("hp_lost", "owner_alive", "damage_end"),
            ("phase_begin", "owner_alive", "phase_begin"),
        ],
    );
    let pkg_b = compile_compound(
        "r05e-dcbihuo-y",
        "r05e_dcbihuo_by",
        998003012,
        783,
        &[
            ("damage", "owner_alive", "damage_source"),
            ("hp_lost", "owner_alive", "damage_end"),
            ("phase_begin", "owner_alive", "phase_begin"),
        ],
    );
    let (pack_a, pack_b) = (pack_of("authorA", "r05e-dcbihuo-x", &pkg_a), pack_of("authorB", "r05e-dcbihuo-y", &pkg_b));
    let mut e = Engine::new_with_extensions(33923, 3, &[&pack_a, &pack_b]).unwrap();
    e.assign_general_named(0, "authorA").unwrap();
    e.assign_general_named(1, "authorB").unwrap();
    run_until(&mut e, |_| None, |e| at_phase(e, 0, Phase::Play));
    sha_once(&mut e);
    let mut by_owner: Vec<(u8, u8)> = Vec::new();
    run_until(
        &mut e,
        |c| match c.spec.prompt {
            "rvm_dcbihuo_target" => {
                by_owner.push((c.observer, 2));
                Some(vec![O::Player(2)])
            }
            other => panic!("意外询问 {other}"),
        },
        |e| !at_phase(e, 0, Phase::Play),
    );
    // A（owner=0）与 B（owner=1）各在 Source/End 窗被问。
    assert_eq!(by_owner.len(), 4, "两包 × 两窗");
    let (next_a, next_b) = (
        helper_def(&e, "r05e_dcbihuo_ax#r05:next"),
        helper_def(&e, "r05e_dcbihuo_by#r05:next"),
    );
    assert_ne!(next_a, next_b);
    assert_eq!(mark(&e, 2, next_a), Some(0));
    assert_eq!(mark(&e, 2, next_b), Some(0));
    // seat2 回合开始只 take 各自的 Next。
    run_until(&mut e, |_| None, |e| at_phase(e, 2, Phase::Judge));
    let (cur_a, cur_b) = (
        helper_def(&e, "r05e_dcbihuo_ax#r05:current"),
        helper_def(&e, "r05e_dcbihuo_by#r05:current"),
    );
    assert_eq!(mark(&e, 2, cur_a), Some(0));
    assert_eq!(mark(&e, 2, cur_b), Some(0));
}

// ---- 通用 AddMark 拒绝 + 装载器派生 ----

#[test]
fn r05e_generic_add_mark_on_helper_key_is_rejected() {
    // 先建 A 包引擎拿派生 def，再用该 def 构造通用 add_mark 模块包。
    let pkg_a = compile_compound(
        "r05e-gate-a",
        "r05e_gate_a",
        998004001,
        782,
        &[("phase_prepare", "owner_is_target", "phase_prepare")],
    );
    let pack_a = pack_of("author", "r05e-gate-a", &pkg_a);
    let probe = Engine::new_with_extensions(33931, 3, &[&pack_a]).unwrap();
    let helper = helper_def(&probe, "r05e_gate_a#r05:combs");
    // 通用 add_mark(self, helper, 1)：[player, mark, delta] 依次压栈。
    use rule_vm::bytecode::{caps, assemble, Op};
    let code = assemble(&[
        Op::HostSelf,
        Op::ConstU32(helper),
        Op::ConstI32(1),
        Op::HostEffect(6), // ADD_MARK
        Op::Halt,
    ]);
    let source = "skill r05e_gate_mark(998004002) {\n  trigger phase_play owner_is_target {\n    effect emit_note(0);\n  }\n}\n";
    let mut pkg_m = content_compiler::compile_source_named("r05e-gate-m", source).unwrap();
    let module_name = pkg_m.trigger_index[0].module.clone();
    let module = rule_vm::BytecodeModule {
        logic_hash: rule_vm::BytecodeModule::compute_logic_hash(
            &module_name,
            &code,
            0,
            caps::EMIT_EFFECTS,
            0,
        ),
        name: module_name.clone(),
        code,
        local_count: 0,
        max_stack: 3,
        entry_pc: 0,
        caps_mask: caps::EMIT_EFFECTS,
    };
    // 模块整体替换（触发段唯一）；保留原模块名（身份一致性校验）。
    *pkg_m
        .modules
        .iter_mut()
        .find(|m| m.name == module_name)
        .expect("module") = module;
    pkg_m.manifest.logic_hash = pkg_m.compute_logic_hash();
    let pack_m = pack_of("marker", "r05e-gate-m", &pkg_m);
    // seat0 持 marker 武将（pack A 只为派生 def 存在目录）。
    let mut e = Engine::new_with_extensions(33932, 3, &[&pack_a, &pack_m]).unwrap();
    e.assign_general_named(0, "marker").unwrap();
    // 进入出牌阶段的命令触发 marker → AddMark 碰派生键 → 复检拒绝
    //（命令被 Fault 拒绝并回滚）。
    run_until(&mut e, |_| None, |e| at_phase(e, 0, Phase::Draw));
    let outcome = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(outcome, CommandOutcome::Rejected(_)),
        "通用 AddMark 碰派生 helper 键应被拒: {outcome:?}"
    );
    assert_eq!(mark(&e, 0, helper), None, "键未被写入");
}

#[test]
fn r05e_loader_derives_roles_deterministically() {
    let cases: [(&str, &str, u64, u16, &[&str], Vec<&str>); 3] = [
        (
            "r05e-der-t",
            "r05e_der_t",
            998005001,
            781,
            &["draw_phase_after"],
            vec!["r05e_der_t#r05:limit"],
        ),
        (
            "r05e-der-b",
            "r05e_der_b",
            998005002,
            782,
            &["phase_prepare"],
            vec!["r05e_der_b#r05:combs"],
        ),
        (
            "r05e-der-d",
            "r05e_der_d",
            998005003,
            783,
            &["damage_source", "damage_end", "phase_begin"],
            vec!["r05e_der_d#r05:next", "r05e_der_d#r05:current"],
        ),
    ];
    for (package, skill, id, tag, events, helpers) in cases {
        let triggers: Vec<(&str, &str, &str)> = events
            .iter()
            .map(|event| {
                let (shell, filter) = match *event {
                    "phase_prepare" => ("phase_prepare", "owner_is_target"),
                    "phase_begin" => ("phase_begin", "owner_alive"),
                    "damage_source" => ("damage", "owner_alive"),
                    "damage_end" => ("hp_lost", "owner_alive"),
                    _ => ("phase_draw_end", "owner_alive"),
                };
                (shell, filter, *event)
            })
            .collect();
        let pkg = compile_compound(package, skill, id, tag, &triggers);
        let first = content_native_loader::bridge_vm_skill_decls(&pkg).unwrap();
        let second = content_native_loader::bridge_vm_skill_decls(&pkg).unwrap();
        let names = |o: &content_native_loader::VmBridgeOutcome| {
            o.decls.iter().map(|d| d.name.clone()).collect::<Vec<_>>()
        };
        assert_eq!(names(&first), names(&second), "派生确定");
        for helper in helpers {
            assert!(
                first.decls.iter().any(|d| d.name == helper),
                "{package} 应派生 {helper}：{:?}",
                names(&first)
            );
        }
        // 非 compound 的普通包不派生任何 helper。
        let plain = content_compiler::compile_source_named(
            "r05e-der-plain",
            "skill r05e_der_plain(998005004) {\n  trigger phase_play owner_is_target {\n    effect emit_note(0);\n  }\n}\n",
        )
        .unwrap();
        let plain_out = content_native_loader::bridge_vm_skill_decls(&plain).unwrap();
        assert!(plain_out.decls.iter().all(|d| !d.name.contains("#r05:")));
    }
}


/// R05-E codec 挂起往返回归：三 compound 效果在挂起队列中编码→解码→
/// 逐字段相等（修复前 `enc_effects` 对新变体命中 `unreachable!()`，
/// 挂起存档/AI 规划会 panic）。
#[test]
fn r05e_compound_effects_roundtrip_through_suspended_codec() {
    use engine_native::rvm_ops::{dec_effects, enc_effects};
    use rule_api::{EffectRequest, PlayerId, SkillId};
    let cases = [
        EffectRequest::ResolveTiqi {
            player: PlayerId::new(2),
            skill: SkillId::new(77),
        },
        EffectRequest::ResolveBaoshu {
            player: PlayerId::new(1),
            skill: SkillId::new(88),
        },
        EffectRequest::ResolveDcbihuo {
            player: PlayerId::new(0),
            skill: SkillId::new(99),
        },
    ];
    let mut bytes = Vec::new();
    enc_effects(&mut bytes, &cases);
    let decoded = dec_effects(&bytes).expect("三 compound 挂起效果可解码");
    assert_eq!(decoded.len(), cases.len());
    for (want, got) in cases.iter().zip(&decoded) {
        assert_eq!(
            format!("{want:?}"),
            format!("{got:?}"),
            "挂起效果编解码后逐字段相等"
        );
    }
}

// ---- E 未尽项 3–7（F 批 kernel 回调）：真实行为红（接线前必失败） ----

/// 三 compound 全声明壳（四角色 helper 齐派生；到期/死亡/上限回调测试用）。
/// 不挂真实派发触发（事件名设为装载器接受但不派发的 `game_started`），
/// 测试直接布 marks/实例，只测 kernel 回调。
fn r05f_pack_all_roles() -> (Engine, u8) {
    let pkg = compile_compound("r05f-all", "tiqi_f", 950001, 781,
        &[("phase_end", "always", "hp_lost")]);
    let pkg2 = compile_compound("r05f-all2", "baoshu_f", 950002, 782,
        &[("phase_end", "always", "hp_lost")]);
    let pkg3 = compile_compound("r05f-all3", "dcbihuo_f", 950003, 783,
        &[("phase_end", "always", "hp_lost")]);
    let pack1 = pack_of("f_holder", "r05f-all", &pkg);
    let pack2 = pack_of("f_baoshu", "r05f-all2", &pkg2);
    let pack3 = pack_of("f_dcbihuo", "r05f-all3", &pkg3);
    let e = engine_native::Engine::new_with_extensions(96001, 3, &[&pack1, &pack2, &pack3])
        .unwrap();
    (e, 0)
}

fn role_def(e: &Engine, skill: &str, role: &str) -> u32 {
    e.catalog()
        .skill_by_name(&format!("{skill}#r05:{role}"))
        .unwrap_or_else(|| panic!("装载器派生 {skill}#{role}")) as u32
}

/// 未尽项 3a：Limit/Current 到期——任意角色整回合结束（全局 phaseAfter）
/// 后清 Limit，Next/Combs 保留（§6 全局 After expiry）。
#[test]
fn r05f_limit_current_expire_at_global_phase_after() {
    // 直接布 marks：Limit=2、Combs=-1、Next=1、Current=3（绕过 compound 机，
    // 只测到期回调——四角色到期是目录级回调，不是 compound 机行为）。
    let (mut e, seat) = r05f_pack_all_roles();
    let catalog = e.catalog().clone();
    let limit_def = role_def(&e, "tiqi_f", "limit");
    let (combs, next, current) = (
        role_def(&e, "baoshu_f", "combs"),
        role_def(&e, "dcbihuo_f", "next"),
        role_def(&e, "dcbihuo_f", "current"),
    );
    {
        let core = e.state_core_mut();
        for def in [limit_def, combs, next, current] {
            core.vm_aux.marks.insert((seat, def), 0);
        }
        core.vm_aux.marks.insert((seat, limit_def), 2);
        core.vm_aux.marks.insert((seat, combs), -1);
        core.vm_aux.marks.insert((seat, next), 1);
        core.vm_aux.marks.insert((seat, current), 3);
    }
    let _ = catalog;
    // 推进完整一回合＋下一人回合开始（全局 phaseAfter 在收束窗后）。
    for _ in 0..24 {
        if e.turn().current == 1 && e.turn().phase == Phase::Judge {
            break;
        }
        pump(&mut e, |_| None);
    }
    let core = e.state_core();
    assert!(
        !core.vm_aux.marks.contains_key(&(seat, limit_def)),
        "Limit 到期清零（全局 phaseAfter 后）"
    );
    assert!(
        !core.vm_aux.marks.contains_key(&(seat, current)),
        "Current 到期清零"
    );
    assert_eq!(
        core.vm_aux.marks.get(&(seat, combs)),
        Some(&-1),
        "Combs 保留（含负数）"
    );
    assert_eq!(core.vm_aux.marks.get(&(seat, next)), Some(&1), "Next 保留");
}

/// 未尽项 4：死亡清理——普通死亡清 Limit/Combs/Current 及 helper 实例，
/// Next 条目/值/实例原样保留（§6）。
#[test]
fn r05f_death_clears_limit_combs_current_keeps_next() {
    let (mut e, seat) = r05f_pack_all_roles();
    let limit_def = role_def(&e, "tiqi_f", "limit");
    let (combs, next, current) = (
        role_def(&e, "baoshu_f", "combs"),
        role_def(&e, "dcbihuo_f", "next"),
        role_def(&e, "dcbihuo_f", "current"),
    );
    {
        let core = e.state_core_mut();
        core.vm_aux.marks.insert((seat, limit_def), 1);
        core.vm_aux.marks.insert((seat, combs), 2);
        core.vm_aux.marks.insert((seat, next), 1);
        core.vm_aux.marks.insert((seat, current), 1);
        for def in [limit_def, combs, next, current] {
            if let Ok(id) = u16::try_from(def) {
                core.skills.insert(
                    (seat, id),
                    engine_native::state::SkillInstance {
                        owner: seat,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: false,
                        rounds: 0,
                    },
                );
            }
        }
        core.players[seat as usize].hp = 0;
        core.players[seat as usize].dying = false;
        core.players[seat as usize].alive = false;
    }
    // 死亡后的下一次收束触发清理回调（commit_death_skills 路径）。
    for _ in 0..12 {
        pump(&mut e, |_| None);
    }
    let core = e.state_core();
    assert!(!core.vm_aux.marks.contains_key(&(seat, limit_def)), "死亡清 Limit");
    assert!(!core.vm_aux.marks.contains_key(&(seat, combs)), "死亡清 Combs");
    assert!(!core.vm_aux.marks.contains_key(&(seat, current)), "死亡清 Current");
    assert_eq!(core.vm_aux.marks.get(&(seat, next)), Some(&1), "Next 值保留");
    if let Ok(id) = u16::try_from(next) {
        assert!(
            core.skills.contains_key(&(seat, id)),
            "Next 实例保留（仅 alive 门失效）"
        );
    }
}

/// 未尽项 5：手牌上限——弃牌阶段的最终上限在其他基数/修饰后加全部启用
/// Limit（有符号 clamp 0）。
#[test]
fn r05f_hand_limit_applies_enabled_limit_signed_clamped() {
    let (mut e, seat) = r05f_pack_all_roles();
    let limit_def = role_def(&e, "tiqi_f", "limit");
    {
        let core = e.state_core_mut();
        core.vm_aux.marks.insert((seat, limit_def), -3);
        if let Ok(id) = u16::try_from(limit_def) {
            core.skills.insert(
                (seat, id),
                engine_native::state::SkillInstance {
                    owner: seat,
                    enabled: true,
                    once_used: false,
                    expires_end_of_turn: false,
                    rounds: 0,
                },
            );
        }
        // 手牌 4、体力上限 4：Limit=-3 → 最终上限 1（4-3）。
        while core.player(seat).hand.len() < 4 {
            let c = core.deck.pop().unwrap();
            core.player_mut(seat).hand.push(c);
        }
    }
    // 推进到本人弃牌阶段末（PhaseEnd(Discard) 的弃到上限按 Limit 收敛）。
    let mut saw_discard = false;
    for _ in 0..40 {
        if e.turn().current == seat && e.turn().phase == Phase::Discard {
            saw_discard = true;
        }
        if saw_discard && e.turn().phase == Phase::End {
            break;
        }
        pump(&mut e, |_| None);
    }
    assert!(saw_discard, "到达本人弃牌阶段");
    assert_eq!(
        e.state_core().player(seat).hand.len(),
        1,
        "最终上限=基数-Limit（clamp 0）"
    );
}

/// 未尽项 3c：Combs 消费——目标本人的非 fixed D 到 Begin2 且 stored>0 才
/// pending＋全部正值、归零清槽与 helper；≤0（负数/0）不触发不消费不删除。
#[test]
fn r05f_combs_consumes_at_begin2_when_positive() {
    let (mut e, seat) = r05f_pack_all_roles();
    let combs = role_def(&e, "baoshu_f", "combs");
    {
        let core = e.state_core_mut();
        // 手工布一个 Combs=3 的槽；本人摸牌阶段的 Begin2 应消费为 pending+3。
        core.vm_aux.marks.insert((seat, combs), 3);
    }
    // 推进到本人摸牌阶段完成（D 机 Begin2 消费）。
    let mut consumed = false;
    for _ in 0..40 {
        if !e.state_core().vm_aux.marks.contains_key(&(seat, combs)) {
            consumed = true;
            break;
        }
        pump(&mut e, |_| None);
    }
    assert!(consumed, "Combs>0 在本人摸牌 Begin2 被消费");
    // 摸牌阶段基础 2＋Combs 3＝手牌应至少 +5（起手 4 → ≥9）。
    assert!(
        e.state_core().player(seat).hand.len() >= 9,
        "pending 加全部正值（手牌 {}）",
        e.state_core().player(seat).hand.len()
    );
    // 负值对照：布 -2 槽，下一轮摸牌不消费不删除。
    {
        let core = e.state_core_mut();
        core.vm_aux.marks.insert((seat, combs), -2);
    }
    for _ in 0..40 {
        pump(&mut e, |_| None);
    }
    assert_eq!(
        e.state_core().vm_aux.marks.get(&(seat, combs)),
        Some(&-2),
        "Combs ≤0 不消费不删除"
    );
}
