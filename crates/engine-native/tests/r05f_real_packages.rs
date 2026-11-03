//! R05-F 真包数据批：xianding 的 tiqi／baoshu 与 sp2 的 dcbihuo 按**上游真实
//! trigger 声明**经 `packages/extensions/` 真包（package.npack.json＋
//! assets/characters.json＋src/*.skill 壳）装载→建局→整局触发→断言。
//!
//! 包数据生成（E 批 retrigger 模式的数据面落地）：src/*.skill DSL 壳只声明
//! trigger＋emit_note 占位；生成期把 trigger_index 事件名改写为原生装载面、
//! 模块体替换为含 HOST_EFFECT_EXT(tag) 的 compound 单 resolver——装载器
//! `derive_r05_helpers` 扫模块字节派生四角色私有 helper。`r05f_real_packages_
//! regeneration_matches_committed_npack` 回验盘上 npack 与壳再生结果一致
//!（数据来源可复现）。装载面对照上游声明：
//! · tiqi 上游 global:[phaseDrawEnd,phaseDrawSkipped,phaseDrawCancelled]——
//!   主 draw>0 时 End 窗被 R 子帧替代、恒发射终窗是 After，故声明
//!   `draw_phase_after`（E 批登记差异 7）；Skipped/Cancelled 无原生触发面
//!   （跳过路径 D 帧直接 Intrinsic→Done；Cancelled 无发射点）＝语义缺口。
//! · baoshu 上游 player:phaseZhunbeiBegin → `phase_prepare`（无改写）。
//! · dcbihuo 上游 player:damageEnd＋source:damageSource（＋subSkill effect
//!   phaseBegin／sha phaseDrawBegin2）→ `damage_source`＋`damage_end`＋
//!   `phase_begin` 三声明（join 成单 handler 集）；Begin2 额定摸牌 ± 落
//!   Current 槽——内核消费点未接线（语义缺口，见 f-real 报告）。

use engine_native::content::extension::{ExtensionDef, ExtensionPack};
use engine_native::{ChoiceOption as O, CommandOutcome, Engine, GameAction, GameCommand, OpenChoice, Phase};
use std::path::{Path, PathBuf};
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

/// 引擎泵：有开放询问先按脚本（rvm_* 前缀必须命中，否则按默认应答），无询问
/// 推进一次阶段。
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
    let actor = e.turn().current;
    must(e, actor, GameAction::NextPhase);
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
    panic!(
        "run_until 超预算: turn={:?}/{:?} open={:?}",
        e.turn().current,
        e.turn().phase,
        e.open_choice().map(|c| c.spec.prompt.to_string())
    );
}

fn at_phase(e: &Engine, player: u8, phase: Phase) -> bool {
    e.turn().current == player && e.turn().phase == phase
}

fn mark(e: &Engine, seat: u8, def: u32) -> Option<i32> {
    e.state_core().vm_aux.marks.get(&(seat, def)).copied()
}

fn helper_def(e: &Engine, name: &str) -> u32 {
    e.catalog().skill_by_name(name).unwrap_or_else(|| panic!("派生 helper 入目录: {name}")) as u32
}

// ---- 真包装载 ----

fn pkg_root(dir: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions")
        .join(dir)
}

/// 单 resolver compound 模块：`HOST_SELF, HOST_SKILL, HOST_EFFECT_EXT(tag),
/// HALT`（与 E 批同形；derive_r05_helpers 扫 0x5A 立即数标签）。
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

/// 每包再生成表：(技能名, compound tag, [(DSL 壳事件名, 原生装载面事件名)])。
type SkillSpec = (&'static str, u16, &'static [(&'static str, &'static str)]);

const XIANDING_SPEC: &[SkillSpec] = &[(
    "tiqi",
    781,
    &[("phase_draw_end", "draw_phase_after")],
), (
    "baoshu",
    782,
    &[("phase_prepare", "phase_prepare")],
)];

const SP2_SPEC: &[SkillSpec] = &[(
    "dcbihuo",
    783,
    &[
        ("damage", "damage_source"),
        ("hp_lost", "damage_end"),
        ("phase_begin", "phase_begin"),
    ],
)];

/// 从 src/*.skill 壳再生成完整包（retrigger 数据面）：编译→按表改写事件名＋
/// 替换模块体→改 manifest 身份→重算聚合 logic_hash。确定性：壳字节序稳定、
/// id/表为常量。
fn rebuild_package(dir: &str, package_id: &str, spec: &[SkillSpec]) -> content_schema::ContentPackage {
    let mut sources: Vec<(String, String)> = Vec::new();
    for entry in std::fs::read_dir(pkg_root(dir).join("src")).expect("src 目录存在") {
        let path = entry.expect("目录项").path();
        if path.extension().is_some_and(|ext| ext == "skill") {
            sources.push((
                path.file_name().unwrap().to_string_lossy().into_owned(),
                std::fs::read_to_string(&path).expect("读 .skill 壳"),
            ));
        }
    }
    sources.sort_by(|a, b| a.0.cmp(&b.0));
    assert!(!sources.is_empty(), "{dir} 无 .skill 壳");
    let source = sources.iter().map(|(_, s)| s.as_str()).collect::<Vec<_>>().join("\n");
    let mut pkg = content_compiler::compile_source_named(package_id, &source)
        .unwrap_or_else(|e| panic!("编译 {dir} 壳失败: {e:?}"));
    pkg.manifest.namespace = "upstream-noname".to_string();
    pkg.manifest.version = "1.0.0".to_string();
    for entry in pkg.trigger_index.iter_mut() {
        let skill = pkg
            .skill_index
            .iter()
            .find(|s| s.skill_def_id == entry.skill_def_id)
            .unwrap_or_else(|| panic!("trigger 引用未知技能 {}", entry.skill_def_id));
        let (_, tag, faces) = spec
            .iter()
            .find(|(name, _, _)| *name == skill.name)
            .unwrap_or_else(|| panic!("技能 {} 不在再生成表", skill.name));
        let (_, native) = faces
            .iter()
            .find(|(shell, _)| *shell == entry.event)
            .unwrap_or_else(|| panic!("技能 {} 壳事件 {} 无装载面映射", skill.name, entry.event));
        entry.event = native.to_string();
        let module = pkg
            .modules
            .iter_mut()
            .find(|m| m.name == entry.module)
            .expect("trigger 模块在包内");
        *module = compound_module(&entry.module, *tag);
    }
    pkg.manifest.logic_hash = pkg.compute_logic_hash();
    pkg
}

fn committed_package(dir: &str) -> content_schema::ContentPackage {
    let json = std::fs::read_to_string(pkg_root(dir).join("package.npack.json"))
        .unwrap_or_else(|e| panic!("读 {dir}/package.npack.json 失败（包数据未接入）: {e}"));
    serde_json::from_str(&json).expect("npack 反序列化")
}

/// 生产装载路径：npack（桥 VM 声明＋派生 helper）＋assets/characters.json
///（bridge_generals_with_vm_package＝Worker/服务端安装路径）→ ExtensionPack。
fn load_real_pack(dir: &str) -> ExtensionPack {
    let npack = committed_package(dir);
    let decls = content_native_loader::bridge_vm_skill_decls(&npack)
        .unwrap_or_else(|e| panic!("桥接 {dir} 失败: {e:?}"))
        .decls;
    let chars = std::fs::read_to_string(pkg_root(dir).join("assets/characters.json"))
        .expect("读 characters.json");
    let generals = content_native_loader::bridge_generals_with_vm_package(&chars, &npack)
        .unwrap_or_else(|e| panic!("{dir} 武将装载失败: {e:?}"))
        .generals;
    let meta: serde_json::Value = serde_json::from_str(&chars).expect("characters.json 解析");
    let pack_name = meta["pack"]["name"].as_str().expect("pack.name");
    ExtensionPack {
        def: ExtensionDef {
            name: engine_native::content::names::intern(pack_name.to_string()),
            version: 1,
            description: engine_native::content::names::intern(
                meta["pack"]["description"].as_str().unwrap_or("").to_string(),
            ),
        },
        generals,
        cards: vec![],
        host_skills: vec![],
        vm_skills: decls,
        init_hook: Default::default(),
        enabled: true,
    }
}

// ---- 再生与装载面登记 ----

#[test]
fn r05f_real_packages_regeneration_matches_committed_npack() {
    for (dir, package_id, spec) in [
        ("xianding-upstream-noname", "xianding", XIANDING_SPEC),
        ("sp2-upstream-noname", "sp2", SP2_SPEC),
    ] {
        let rebuilt = rebuild_package(dir, package_id, spec);
        let committed = committed_package(dir);
        // 聚合哈希与逐字段一致（数据来源＝壳再生成）。
        assert_eq!(
            rebuilt.manifest.logic_hash, committed.manifest.logic_hash,
            "{dir}: 盘上 npack 与壳再生 logic_hash 不一致"
        );
        assert_eq!(rebuilt, committed, "{dir}: 盘上 npack 与壳再生内容不一致");
        // 装载面事件名精确登记（对照上游声明；trigger_index 序随技能名字节序，
        // 故按集合比对）。
        let mut events: Vec<&str> = committed.trigger_index.iter().map(|t| t.event.as_str()).collect();
        events.sort_unstable();
        let mut expected: Vec<&str> = spec
            .iter()
            .flat_map(|(_, _, faces)| faces.iter().map(|(_, native)| *native))
            .collect();
        expected.sort_unstable();
        assert_eq!(events, expected, "{dir}: 触发面与装载面表不一致");
        // 桥接后派生四角色 helper（tiqi→limit；baoshu→combs；dcbihuo→next/current）。
        let decls = content_native_loader::bridge_vm_skill_decls(&committed).unwrap().decls;
        let names: Vec<&str> = decls.iter().map(|d| d.name.as_str()).collect();
        for helper in expected_helpers(spec) {
            assert!(names.contains(&helper.as_str()), "{dir}: 缺派生 helper {helper}");
        }
    }
}

fn expected_helpers(spec: &[SkillSpec]) -> Vec<String> {
    const ROLES: [(&str, u16, &[&str]); 3] = [
        ("tiqi", 781, &["limit"]),
        ("baoshu", 782, &["combs"]),
        ("dcbihuo", 783, &["next", "current"]),
    ];
    spec.iter()
        .filter_map(|(name, tag, _)| {
            ROLES
                .iter()
                .find(|(n, t, _)| n == name && t == tag)
                .map(|(_, _, roles)| roles.iter().map(|r| format!("{name}#r05:{r}")).collect::<Vec<_>>())
        })
        .flatten()
        .collect()
}

// ---- tiqi：他人摸牌 n≠2 → 确认窗＋±选择写目标 Limit ----

#[test]
fn r05f_real_tiqi_other_player_draw_confirm_sign_writes_limit() {
    let pack = load_real_pack("xianding-upstream-noname");
    let mut e = Engine::new_with_extensions(34901, 3, &[&pack]).unwrap();
    e.assign_general_named(0, "xnd冯妤").unwrap();
    // 走完 seat0 首回合（自己 D 被排除，无询问）。
    run_until(&mut e, |_| None, |e| at_phase(e, 1, Phase::Judge));
    // 牌库/弃牌堆双枯竭：seat1 的 D 无 gain → live n=0（≠2 → d=2）。
    e.debug_drain_deck_to_discard();
    e.state_core_mut().discard.clear();
    let mut asked: Vec<&'static str> = Vec::new();
    run_until(
        &mut e,
        |c| {
            asked.push(match c.spec.prompt {
                "rvm_tiqi_confirm" => "confirm",
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
    let def = helper_def(&e, "tiqi#r05:limit");
    assert_eq!(mark(&e, 1, def), Some(-2), "n=0 → d=2 → 选 −d：Limit=-2");
    // seat2 未到其 D：无跨目标串写。
    assert_eq!(mark(&e, 2, def), None);
}

// ---- baoshu：自己准备阶段成本窗 → 多目标重置＋梳标记 ----

#[test]
fn r05f_real_baoshu_prepare_cost_resets_and_combs_marks() {
    let pack = load_real_pack("xianding-upstream-noname");
    let mut e = Engine::new_with_extensions(34902, 3, &[&pack]).unwrap();
    e.assign_general_named(0, "xnd冯妤").unwrap();
    // 首回合无准备阶段：推进到 seat0 下一回合的准备阶段。
    run_until(&mut e, |_| None, |e| at_phase(e, 0, Phase::Prepare));
    // 目标状态：1 号横置、2 号翻面。
    e.state_core_mut().player_mut(1).linked = true;
    e.debug_set_face_down(2, true);
    run_until(
        &mut e,
        |c| match c.spec.prompt {
            "rvm_baoshu_cost" => Some(vec![O::Player(1), O::Player(2)]),
            other => panic!("意外询问 {other}"),
        },
        |e| !(e.turn().current == 0 && e.turn().phase == Phase::Prepare),
    );
    let def = helper_def(&e, "baoshu#r05:combs");
    // maxHp=3、k=2 → m=1+3−2=2：两目标各 +2。
    assert_eq!(mark(&e, 1, def), Some(2), "m=1+maxHp−k=2");
    assert_eq!(mark(&e, 2, def), Some(2));
    assert!(!e.state_core().player(1).linked, "横置目标被解除");
    assert!(!e.state_core().player(2).face_down, "翻面目标被翻回");
    assert!(e.state_core().skills.contains_key(&(1, def as u16)), "装 Combs helper");
    assert!(e.state_core().skills.contains_key(&(2, def as u16)));
}

// ---- dcbihuo：受伤/造伤两面 → 目标下回合额定摸牌数 ±（Next→Current take） ----

fn sha_once(e: &mut Engine) {
    let sha = e.debug_place(0, "sha");
    e.debug_clear_hand(1);
    must(e, 0, GameAction::PlayCard {
        card: sha,
        targets: vec![1],
    });
}

#[test]
fn r05f_real_dcbihuo_source_minus_end_plus_take_current() {
    let pack = load_real_pack("sp2-upstream-noname");
    let mut e = Engine::new_with_extensions(34903, 3, &[&pack]).unwrap();
    e.assign_general_named(0, "sp2冯方").unwrap();
    run_until(&mut e, |_| None, |e| at_phase(e, 0, Phase::Play));
    sha_once(&mut e);
    let mut windows = 0;
    run_until(
        &mut e,
        |c| match c.spec.prompt {
            "rvm_dcbihuo_target" => {
                windows += 1;
                Some(vec![O::Player(1)])
            }
            other => panic!("意外询问 {other}"),
        },
        |e| !at_phase(e, 0, Phase::Play),
    );
    assert_eq!(windows, 2, "Source 与 End 各一问");
    let next = helper_def(&e, "dcbihuo#r05:next");
    let current = helper_def(&e, "dcbihuo#r05:current");
    assert_eq!(mark(&e, 1, next), Some(0), "−1+1 抵消：Some0 保留");
    // seat1 回合开始：原子 take——Next 移除、Current 写 0、装默认 helper。
    run_until(&mut e, |_| None, |e| at_phase(e, 1, Phase::Judge));
    assert_eq!(mark(&e, 1, next), None, "Next 被 take 移除");
    assert_eq!(mark(&e, 1, current), Some(0), "Current 含 0 条目");
    assert!(e.state_core().skills.contains_key(&(1, current as u16)));
}

#[test]
fn r05f_real_dcbihuo_cancel_end_keeps_minus_take_signed() {
    let pack = load_real_pack("sp2-upstream-noname");
    let mut e = Engine::new_with_extensions(34904, 3, &[&pack]).unwrap();
    e.assign_general_named(0, "sp2冯方").unwrap();
    run_until(&mut e, |_| None, |e| at_phase(e, 0, Phase::Play));
    sha_once(&mut e);
    let mut first = true;
    run_until(
        &mut e,
        |c| match c.spec.prompt {
            "rvm_dcbihuo_target" => {
                let answer = if first { vec![O::Player(1)] } else { vec![] };
                first = false;
                Some(answer)
            }
            other => panic!("意外询问 {other}"),
        },
        |e| !at_phase(e, 0, Phase::Play),
    );
    let next = helper_def(&e, "dcbihuo#r05:next");
    assert_eq!(mark(&e, 1, next), Some(-1), "仅 Source −1 生效");
    run_until(&mut e, |_| None, |e| at_phase(e, 1, Phase::Judge));
    let current = helper_def(&e, "dcbihuo#r05:current");
    assert_eq!(mark(&e, 1, current), Some(-1), "take 写 Current=-1（含负值）");
    assert_eq!(mark(&e, 1, next), None);
}
