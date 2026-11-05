//! R05-H（裁决 28 合同 v3 §8）：公开分项投影与涕泣询问 context。
//!
//! - 红1：SeatView `r05_effects` 有序分项——多逻辑身份（两个包）四角色槽
//!   聚合，Combs/Limit 负值与 Some(0) 均非空项，死亡席位 Next 照显、
//!   disabled 实例保值；本人/他人/观战/裁判/AI 见相同公开值（含 linked）。
//! - 红2：涕泣符号选择的 Own context=`TiqiAdjustment{skill,target,delta}`
//!   （拥有者/裁判完整；他人仅询问存在；AI 同席位视图；delta 取自已验证
//!   内容局部 n，不重算）；确认询问无 context。
//! - 红3：隐藏面守卫——投影（视图与选择）不含 raw mark/helper 名、
//!   VmAux、pending 等内部信息。

use engine_native::catalog::QualifiedSkillRef;
use engine_native::content::extension::{ExtensionDef, ExtensionPack};
use engine_native::state::SkillInstance;
use engine_native::view::{ChoiceContext, ChoiceView, R05EffectView};
use engine_native::{
    ChoiceOption as O, CommandOutcome, Engine, GameAction, GameCommand, OpenChoice, Phase,
};
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

/// 单 resolver compound 模块（与 r05e 同形）。
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

/// 把每个触发模块替换为各自 compound 标签的单 resolver 体（R05-H：一个
/// 逻辑技能同时声明三 compound，装载器为其派生全部四角色 helper）。
/// `tags[i]` 与 `events[i]` 分别为该触发的 compound 标签与装载器事件名
/// （与 r05e 的 draw_phase_after/phase_prepare/damage_source 同键）。
fn retrigger_multi(pkg: &mut content_schema::ContentPackage, tags: &[u16], events: &[&str]) {
    assert_eq!(pkg.trigger_index.len(), tags.len());
    assert_eq!(tags.len(), events.len());
    for ((entry, tag), event) in pkg.trigger_index.iter_mut().zip(tags).zip(events) {
        let name = entry.module.clone();
        let module = pkg
            .modules
            .iter_mut()
            .find(|m| m.name == name)
            .expect("trigger module in package");
        *module = compound_module(&name, *tag);
        entry.event = event.to_string();
    }
}

fn compile_multi(
    package: &str,
    skill: &str,
    skill_id: u64,
    tags: &[u16],
    events: &[&str],
) -> content_schema::ContentPackage {
    let body: String = tags
        .iter()
        .map(|_| "  trigger phase_draw_end owner_alive {\n    effect emit_note(0);\n  }\n")
        .collect();
    let source = format!("skill {skill}({skill_id}) {{\n{body}}}");
    let mut pkg = content_compiler::compile_source_named(package, &source)
        .unwrap_or_else(|e| panic!("编译 {skill} 失败: {e:?}"));
    retrigger_multi(&mut pkg, tags, events);
    pkg.manifest.rule_vm_abi_version = 759;
    pkg.manifest.logic_hash = pkg.compute_logic_hash();
    pkg
}

fn pack_of(general: &str, package: &str, pkg: &content_schema::ContentPackage) -> ExtensionPack {
    let decls = content_native_loader::bridge_vm_skill_decls(pkg)
        .unwrap_or_else(|e| panic!("桥接 {package} 失败: {e:?}"))
        .decls;
    let known = decls.iter().map(|d| d.name.clone()).collect();
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
            description: "R05-H projection",
        },
        generals,
        cards: vec![],
        host_skills: vec![],
        vm_skills: decls,
        init_hook: Default::default(),
        enabled: true,
    }
}

fn default_answer(open: &OpenChoice) -> Vec<O> {
    if open.spec.options.contains(&O::Pass) {
        vec![O::Pass]
    } else if open.spec.allow_cancel {
        vec![]
    } else if open.spec.options.len() >= open.spec.min {
        open.spec.options.iter().take(open.spec.min).copied().collect()
    } else {
        panic!("无法默认应答: {:?}", open.spec)
    }
}

fn pump<F>(e: &mut Engine, mut script: F)
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
        return;
    }
    must(e, e.turn().current, GameAction::NextPhase);
}

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
    panic!("run_until 超预算: turn={:?}/{:?}", e.turn().current, e.turn().phase);
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

fn qsr(package: &str, def_id: u32) -> QualifiedSkillRef {
    QualifiedSkillRef {
        namespace: "official".to_string(),
        package_id: package.to_string(),
        logical_skill_def_id: def_id,
    }
}

/// 红1 布阵：包 A（一个逻辑身份，四角色）+ 包 B（仅 Limit）；手工布置
/// marks/死亡/disabled 实例——投影是只读汇总，不经真实结算。
fn projection_engine() -> Engine {
    let pkg_a = compile_multi(
        "r05h-a-mult",
        "r05h_multi_a",
        998003001,
        &[781, 782, 783],
        &["draw_phase_after", "phase_prepare", "damage_source"],
    );
    let pkg_b = compile_multi("r05h-b-lim", "r05h_lim_b", 998003002, &[781], &["draw_phase_after"]);
    let (pack_a, pack_b) = (
        pack_of("r05hAuthorA", "r05h-a-mult", &pkg_a),
        pack_of("r05hAuthorB", "r05h-b-lim", &pkg_b),
    );
    let mut e = Engine::new_with_extensions(34801, 3, &[&pack_b, &pack_a]).unwrap();
    e.assign_general_named(0, "r05hAuthorA").unwrap();
    let a_limit = helper_def(&e, "r05h_multi_a#r05:limit");
    let a_combs = helper_def(&e, "r05h_multi_a#r05:combs");
    let a_next = helper_def(&e, "r05h_multi_a#r05:next");
    let a_current = helper_def(&e, "r05h_multi_a#r05:current");
    let b_limit = helper_def(&e, "r05h_lim_b#r05:limit");
    // seat0：A 的负 Limit/负 Combs 与 Some(0) Current；B 的 Some(0) Limit。
    let core = e.state_core_mut();
    core.vm_aux.marks.insert((0, a_limit), -1);
    core.vm_aux.marks.insert((0, a_combs), -3);
    core.vm_aux.marks.insert((0, a_current), 0);
    core.vm_aux.marks.insert((0, b_limit), 0);
    // seat1：无任何槽（空分项不输出）；linked 公开位照投影。
    core.player_mut(1).linked = true;
    // seat2：死亡 Next 照显 + disabled 实例保值。
    core.vm_aux.marks.insert((2, a_next), 1);
    core.player_mut(2).alive = false;
    core.skills.insert(
        (2, a_next as u16),
        SkillInstance {
            owner: 2,
            enabled: false,
            once_used: false,
            expires_end_of_turn: false,
            rounds: 0,
        },
    );
    e
}

/// 各观察者对同一座位的 r05_effects/linked 必须逐字节相同（§8 公开值）。
fn assert_uniform_public_projection(e: &Engine) -> Vec<Vec<R05EffectView>> {
    let views = [
        e.view_for(0),
        e.view_for(1),
        e.view_spectator(),
        e.view_referee(),
        e.ai_session(1).view(),
    ];
    let baseline: Vec<Vec<R05EffectView>> = views[0]
        .players
        .iter()
        .map(|p| p.r05_effects.clone())
        .collect();
    for view in &views {
        let got: Vec<Vec<R05EffectView>> = view
            .players
            .iter()
            .map(|p| p.r05_effects.clone())
            .collect();
        assert_eq!(got, baseline, "公开 r05_effects 不随观察者变化");
        for cell in &view.players {
            assert_eq!(
                cell.linked,
                e.state_core().player(cell.seat).linked,
                "linked 公开位与权威状态一致"
            );
        }
    }
    baseline
}

#[test]
fn r05h_seat_view_r05_effects_ordered_and_uniform() {
    let e = projection_engine();
    let a = qsr("r05h-a-mult", 998003001);
    let b = qsr("r05h-b-lim", 998003002);
    let expected0 = vec![
        R05EffectView {
            skill: a.clone(),
            tiqi_limit: Some(-1),
            baoshu_combs: Some(-3),
            dcbihuo_next: None,
            dcbihuo_current: Some(0),
        },
        R05EffectView {
            skill: b.clone(),
            tiqi_limit: Some(0),
            baoshu_combs: None,
            dcbihuo_next: None,
            dcbihuo_current: None,
        },
    ];
    let expected2 = vec![R05EffectView {
        skill: a,
        tiqi_limit: None,
        baoshu_combs: None,
        dcbihuo_next: Some(1),
        dcbihuo_current: None,
    }];
    let per_seat = assert_uniform_public_projection(&e);
    assert_eq!(per_seat[0], expected0, "seat0：两身份有序分项（A 先于 B）");
    assert_eq!(per_seat[1], Vec::new(), "seat1：无槽不输出空项");
    assert_eq!(
        per_seat[2], expected2,
        "seat2：死亡 Next 照显、disabled 实例保值"
    );
    // 布阵自检：marks 与 helper defs 如实存在（负值/Some0 均保留条目）。
    let a_limit = helper_def(&e, "r05h_multi_a#r05:limit");
    let b_limit = helper_def(&e, "r05h_lim_b#r05:limit");
    assert_eq!(mark(&e, 0, a_limit), Some(-1));
    assert_eq!(mark(&e, 0, b_limit), Some(0));
}

#[test]
fn r05h_tiqi_adjustment_context_owner_full_other_existence_only() {
    let pkg = compile_multi("r05h-tiqi-a", "r05h_tiqi_a", 998004001, &[781], &["draw_phase_after"]);
    let pack = pack_of("r05hOwner", "r05h-tiqi-a", &pkg);
    let mut e = Engine::new_with_extensions(34802, 3, &[&pack]).unwrap();
    e.assign_general_named(0, "r05hOwner").unwrap();
    // 确认询问阶段：Own context 必须为 None（context 只随符号选择下发）。
    run_until(&mut e, |_| None, |e| at_phase(e, 1, Phase::Judge));
    e.debug_drain_deck_to_discard();
    e.state_core_mut().discard.clear();
    run_until(
        &mut e,
        |c| match c.spec.prompt {
            "rvm_tiqi_confirm" => Some(vec![O::Confirm]),
            other => panic!("意外询问 {other}"),
        },
        |e| e.open_choice().is_some_and(|c| c.spec.prompt == "rvm_tiqi_confirm"),
    );
    match e.choice_view_for(Some(0)) {
        Some(ChoiceView::Own { context, .. }) => assert!(
            context.is_none(),
            "确认询问不应携带 TiqiAdjustment context: {context:?}"
        ),
        other => panic!("owner 应见完整确认询问: {other:?}"),
    }
    let confirm_id = e.open_choice().unwrap().id;
    must(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: confirm_id,
            selection: vec![O::Confirm],
        },
    );
    // 符号询问开放：owner 完整、他人仅存在、裁判完整、AI 同席位。
    run_until(
        &mut e,
        |c| panic!("符号询问应已开放，未预期询问 {}", c.spec.prompt),
        |e| e.open_choice().is_some_and(|c| c.spec.prompt == "rvm_tiqi_sign"),
    );
    let expected = ChoiceContext::TiqiAdjustment {
        skill: qsr("r05h-tiqi-a", 998004001),
        target: 1, // seat1（0 基 PlayerId；服务端 native_view 边界归一为 API 1 基 = 2）
        delta: 2,  // 双枯竭 n=0 → d=|n−2|=2（取自已验证内容局部，不重算）
    };
    match e.choice_view_for(Some(0)) {
        Some(ChoiceView::Own { context, .. }) => {
            assert_eq!(context, Some(expected.clone()), "owner 见完整 context");
        }
        other => panic!("owner 应见完整符号询问: {other:?}"),
    }
    match e.choice_view_for(Some(2)) {
        Some(other @ ChoiceView::Other { .. }) => {
            // 他人视图类型上不含 context/options；Debug 亦不得携带细节。
            let debug = format!("{other:?}");
            assert!(!debug.contains("delta"), "他人视图泄露 delta: {debug}");
            assert!(!debug.contains("TiqiAdjustment"), "他人视图泄露 context: {debug}");
        }
        other => panic!("他人应仅见询问存在: {other:?}"),
    }
    match e.choice_view_referee() {
        Some(ChoiceView::Own { context, .. }) => {
            assert_eq!(context, Some(expected.clone()), "裁判见完整 context");
        }
        other => panic!("裁判应见完整询问: {other:?}"),
    }
    match e.ai_session(0).choice() {
        Some(ChoiceView::Own { context, .. }) => {
            assert_eq!(context, Some(expected), "AI 拿同席位（owner）视图");
        }
        other => panic!("AI 同席位视图应为 Own: {other:?}"),
    }
}

#[test]
fn r05h_projection_hides_internal_faces() {
    let e = projection_engine();
    for view in [
        e.view_for(0),
        e.view_for(1),
        e.view_spectator(),
        e.view_referee(),
    ] {
        let debug = format!("{view:?}");
        assert!(!debug.contains("#r05:"), "投影泄露 raw helper 名: {debug}");
        assert!(!debug.contains("vm_aux"), "投影泄露 VmAux: {debug}");
        assert!(!debug.contains("pending"), "投影泄露 pending: {debug}");
        for cell in &view.players {
            for name in &cell.skills {
                assert!(!name.contains('#'), "技能列表混入 helper 名: {name}");
            }
        }
    }
}
