//! T8 增量 1 端到端：engine-native 触发调度接 rule-vm——扩展包声明的
//! VM 技能（数据驱动，不重编译）经目录 SkillDef 触发，效果按两层提交
//! 协议落账（Halted 提交 / fuel 耗尽回滚）。语义样本=酒尾同款
//! 「受伤回 1 / 满血摸 1」（原型 t8_backend_prototype 对照）。

use content_compiler::compile_source_named;
use engine_native::catalog::TriggerMeta;
use engine_native::event::{EventKind, Timing};
use engine_native::content::extension::{ExtensionDef, ExtensionPack, VmSkillDecl};
use engine_native::event::DamageNature;
use engine_native::event::Phase;
use engine_native::{ChoiceOption, Engine, GameAction, GameCommand};

/// 酒尾对照样本（同原型 JIUWEI_DSL 语义）。
const JIUWEI_DSL: &str = "\
skill hnl_jiuwei_t8(990001) {
  trigger phase_end owner_is_target {
    if hp(self) < max_hp(self) {
      effect recover_hp(self, 1);
    } else {
      effect draw_cards(self, 1);
    }
  }
}
";

/// 超限样本：先发效果再无限循环——fuel 耗尽 → 排队效果整批丢弃。
const OVERRUN_DSL: &str = "\
skill hnl_overrun_t8(990003) {
  trigger phase_end owner_is_target {
    effect draw_cards(self, 1);
    loop 1000000 {
      effect draw_cards(self, 1);
    }
  }
}
";

fn module_of(
    pkg: &content_schema::ContentPackage,
    skill: &str,
) -> rule_vm::BytecodeModule {
    let trig = pkg
        .trigger_index
        .iter()
        .find(|t| t.module.contains(skill))
        .unwrap_or_else(|| panic!("trigger for {skill} not found"));
    pkg.modules
        .iter()
        .find(|m| &m.name == &trig.module)
        .unwrap_or_else(|| panic!("module {} not found", trig.module))
        .clone()
}

fn pack_with(skills: Vec<VmSkillDecl>) -> ExtensionPack {
    ExtensionPack {
        def: ExtensionDef {
            name: "t8-rvm-bridge",
            version: 1,
            description: "T8 增量 1 端到端桥测",
        },
        generals: Vec::new(),
        cards: Vec::new(),
        host_skills: Vec::new(),
        vm_skills: skills,
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    }
}

fn phase_end_trigger() -> TriggerMeta {
    TriggerMeta {
        event: EventKind::PhaseEnd,
        timing: Timing::Normal,
        priority: 0,
        optional: false,
        extra_events: Vec::new(),
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

fn np_settled(e: &mut Engine) {
    let actor = e.turn().current;
    submit(e, actor, GameAction::NextPhase);
    // VM 挂起（rvm_choice）不代答：PA12④ 修复首答丢失后，这里代答会真的
    // 续跑落账，吞掉挂起测试要校验的状态；留给调用者显式处理。
    if e.open_choice().is_some_and(|c| c.spec.prompt == "rvm_choice") {
        return;
    }
    if let Some(c) = e.open_choice() {
        let sel: Vec<ChoiceOption> = c
            .spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Card(_)))
            .copied()
            .take(c.spec.min.max(1))
            .collect();
        let (cid, obs) = (c.id, c.observer);
        if !sel.is_empty() {
            submit(e, obs, GameAction::Answer { choice_id: cid, selection: sel });
        } else if !c.spec.options.is_empty() {
            submit(
                e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: vec![c.spec.options[0].clone()],
                },
            );
        }
    }
}

fn hp(e: &Engine, p: u8) -> i32 {
    e.debug_hp(p)
}

fn hand_len(e: &Engine, p: u8) -> usize {
    e.state_core().player(p).hand.len()
}

/// 恢复分支：hp<max 的持有者阶段结束回 1 体力（数据驱动，不重编译）。
#[test]
fn t8_vm_skill_recover_branch_end_to_end() {
    let pkg = compile_source_named("t8-rvm-bridge", JIUWEI_DSL).expect("DSL 编译");
    let module = module_of(&pkg, "hnl_jiuwei_t8");
    let name = "t8酒尾";
    let pack = pack_with(vec![VmSkillDecl {
        name: name.to_string(),
        module,
        trigger: phase_end_trigger(),
        filter: engine_native::rvm_bridge::VmFilterPolicy::OwnerIsTarget,
    }]);
    let mut e = Engine::new_with_extensions(910001, 4, &[&pack]).expect("建局");
    e.debug_grant(1, name);
    e.debug_set_hp(1, 2);
    assert_eq!(hp(&e, 1), 2);
    // 推进到 1 号阶段结束之后（PhaseEnd 触发 VM → RecoverHp 提交）
    for _ in 0..40 {
        if e.turn().current == 1 && hp(&e, 1) == 3 {
            break;
        }
        if e.outcome().is_some() {
            panic!("对局提前结束");
        }
        np_settled(&mut e);
    }
    assert_eq!(hp(&e, 1), 3, "VM 技能：阶段结束受伤回 1（数据驱动）");
}

/// 摸牌分支：满血持有者阶段结束摸 1。
#[test]
fn t8_vm_skill_draw_branch_end_to_end() {
    let pkg = compile_source_named("t8-rvm-bridge", JIUWEI_DSL).expect("DSL 编译");
    let module = module_of(&pkg, "hnl_jiuwei_t8");
    let name = "t8酒尾";
    let pack = pack_with(vec![VmSkillDecl {
        name: name.to_string(),
        module,
        trigger: phase_end_trigger(),
        filter: engine_native::rvm_bridge::VmFilterPolicy::OwnerIsTarget,
    }]);
    let mut e = Engine::new_with_extensions(910002, 4, &[&pack]).expect("建局");
    e.debug_grant(1, name);
    assert_eq!(hp(&e, 1), 4);
    let h0 = hand_len(&e, 1);
    for _ in 0..40 {
        if e.turn().current == 1 && hand_len(&e, 1) > h0 {
            break;
        }
        if e.outcome().is_some() {
            panic!("对局提前结束");
        }
        np_settled(&mut e);
    }
    assert!(
        hand_len(&e, 1) > h0,
        "VM 技能：阶段结束满血摸 1（手牌 {}→{}）",
        h0,
        hand_len(&e, 1)
    );
}

/// 回滚协议：fuel 耗尽（无限循环样本）→ 排队效果整批丢弃，无摸牌。
#[test]
fn t8_vm_skill_fuel_exhaustion_rolls_back() {
    let pkg = compile_source_named("t8-rvm-bridge", OVERRUN_DSL).expect("DSL 编译");
    let module = module_of(&pkg, "hnl_overrun_t8");
    let name = "t8超限";
    let pack = pack_with(vec![VmSkillDecl {
        name: name.to_string(),
        module,
        trigger: phase_end_trigger(),
        filter: engine_native::rvm_bridge::VmFilterPolicy::OwnerIsTarget,
    }]);
    let mut e = Engine::new_with_extensions(910003, 4, &[&pack]).expect("建局");
    e.debug_grant(1, name);
    let h0 = hand_len(&e, 1);
    for _ in 0..40 {
        if e.turn().current != 0 {
            break;
        }
        if e.outcome().is_some() {
            panic!("对局提前结束");
        }
        np_settled(&mut e);
    }
    // 0 号回合收束后 1 号回合的阶段结束已过（VM trap 回滚）——手牌无
    // VM 增量（阶段推进的正常摸牌不计：满血分支同基数下对照）。
    let baseline = {
        // 无技能对照：同 seed 建局不授技，同推进后的手牌数。
        let pack0 = pack_with(vec![]);
        let mut e0 = Engine::new_with_extensions(910003, 4, &[&pack0]).expect("对照建局");
        for _ in 0..40 {
            if e0.turn().current != 0 {
                break;
            }
            np_settled(&mut e0);
        }
        hand_len(&e0, 1)
    };
    assert_eq!(
        hand_len(&e, 1),
        baseline,
        "fuel 耗尽：排队效果整批丢弃（无半次落账）"
    );
    let _ = h0;
}

/// 选择挂起样本：choose(1,2) 二选一（1=回血，2=摸牌）。
const CHOICE_DSL: &str = "\
skill hnl_jiuwei_choice_t8(990002) {
  trigger phase_end owner_is_target {
    let option = choose(1, 2);
    if option == 1 {
      effect recover_hp(self, 1);
    }
    if option == 2 {
      effect draw_cards(self, 1);
    }
  }
}
";

/// 选择挂起/恢复端到端：阶段结束 VM 挂起（rvm_choice）→ 应答续跑落账；
/// 挂起中存档→恢复→续跑（continuation 跨存档，内容锁绑定）。
#[test]
fn t8_vm_skill_choice_suspend_resume_midrestore() {
    let pkg = compile_source_named("t8-rvm-bridge", CHOICE_DSL).expect("DSL 编译");
    let module = module_of(&pkg, "hnl_jiuwei_choice_t8");
    let name = "t8酒尾选择";
    let pack = pack_with(vec![VmSkillDecl {
        name: name.to_string(),
        module,
        trigger: phase_end_trigger(),
        filter: engine_native::rvm_bridge::VmFilterPolicy::OwnerIsTarget,
    }]);
    let mut e = Engine::new_with_extensions(910005, 4, &[&pack]).expect("建局");
    e.debug_grant(1, name);
    e.debug_set_hp(1, 2);
    // 推进到 1 号阶段结束：VM 挂起（rvm_choice）
    for _ in 0..40 {
        if let Some(c) = e.open_choice() {
            if c.spec.prompt == "rvm_choice" {
                break;
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
            let selection = if sel.is_empty() && !c.spec.options.is_empty() {
                vec![c.spec.options[0].clone()]
            } else {
                sel
            };
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection });
            continue;
        }
        if e.outcome().is_some() {
            panic!("对局提前结束");
        }
        np_settled(&mut e);
    }
    let c = e.open_choice().expect("挂起询问");
    assert_eq!(c.spec.prompt, "rvm_choice", "VM 选择挂起");
    assert_eq!(c.spec.options.len(), 2, "choose(1,2) 两选项");
    // 挂起中存档 → 恢复（同一扩展集合，内容锁）→ 续跑选 1=回血
    let snap = e.save();
    let mut r = Engine::restore_with_extensions(&snap, &[&pack]).expect("挂起态往返");
    let c = r.open_choice().expect("恢复后挂起询问在位");
    assert_eq!(c.spec.prompt, "rvm_choice", "恢复后 VM 挂起态保留");
    let cid = c.id;
    let obs = c.observer;
    submit(
        &mut r,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Tag(1)],
        },
    );
    assert_eq!(hp(&r, 1), 3, "续跑落账：选择 1=回血（2→3）");
}

/// 内容锁负例：扩展包版本变化后挂起态存档拒绝恢复（名称+版本指纹）。
#[test]
fn t8_vm_skill_content_lock_rejects_pack_drift() {
    let pkg = compile_source_named("t8-rvm-bridge", CHOICE_DSL).expect("DSL 编译");
    let module = module_of(&pkg, "hnl_jiuwei_choice_t8");
    let name = "t8酒尾选择";
    let pack = pack_with(vec![VmSkillDecl {
        name: name.to_string(),
        module,
        trigger: phase_end_trigger(),
        filter: engine_native::rvm_bridge::VmFilterPolicy::OwnerIsTarget,
    }]);
    let mut e = Engine::new_with_extensions(910006, 4, &[&pack]).expect("建局");
    e.debug_grant(1, name);
    // 推进到挂起
    for _ in 0..40 {
        let pending = e.open_choice().map(|c| {
            (c.id, c.observer, c.spec.prompt, c.spec.options.clone(), c.spec.min)
        });
        match pending {
            Some((cid, obs, prompt, options, min)) => {
                if prompt == "rvm_choice" {
                    break;
                }
                let sel: Vec<ChoiceOption> = options
                    .iter()
                    .filter(|o| matches!(o, ChoiceOption::Card(_)))
                    .copied()
                    .take(min.max(1))
                    .collect();
                let selection = if sel.is_empty() && !options.is_empty() {
                    vec![options[0].clone()]
                } else {
                    sel
                };
                submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection });
            }
            None => np_settled(&mut e),
        }
    }
    assert!(e.open_choice().is_some(), "挂起在位");
    let snap = e.save();
    // 版本 +1 的同包（版本指纹漂移）
    let mut drifted = pack_with(vec![VmSkillDecl {
        name: name.to_string(),
        module: module_of(
            &compile_source_named("t8-rvm-bridge", CHOICE_DSL).expect("重编译"),
            "hnl_jiuwei_choice_t8",
        ),
        trigger: phase_end_trigger(),
        filter: engine_native::rvm_bridge::VmFilterPolicy::OwnerIsTarget,
    }]);
    drifted.def.version += 1;
    assert!(
        Engine::restore_with_extensions(&snap, &[&drifted]).is_err(),
        "内容锁：包版本漂移拒绝恢复"
    );
}

/// 增量 3 端到端：DSL→编译 npack→**装载器 bridge_vm_skill_decls**→
/// ExtensionPack→建局→触发→落账（C0 装载器在环，全数据驱动）。
#[test]
fn t8_vm_skill_loader_bridge_end_to_end() {
    let pkg = compile_source_named("t8-rvm-bridge-loader", JIUWEI_DSL).expect("DSL 编译");
    let outcome =
        content_native_loader::bridge_vm_skill_decls(&pkg).expect("装载器桥接");
    let decls = outcome.decls;
    assert_eq!(decls.len(), 1, "触发索引 1 项→1 声明");
    // 声明名=模块名（编译器按 技能_事件 命名模块；符号表 intern）。
    assert_eq!(decls[0].name, "hnl_jiuwei_t8_phase_end", "声明名=模块名");
    let pack = pack_with(decls);
    let mut e = Engine::new_with_extensions(910007, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "hnl_jiuwei_t8_phase_end");
    e.debug_set_hp(1, 2);
    for _ in 0..40 {
        if e.turn().current == 1 && hp(&e, 1) == 3 {
            break;
        }
        if e.outcome().is_some() {
            panic!("对局提前结束");
        }
        np_settled(&mut e);
    }
    assert_eq!(hp(&e, 1), 3, "装载器在环：阶段结束受伤回 1");
}

/// 未映射效果样本：change_max_hp（增量 5 桥映射面之外）。
const UNMAPPED_EFFECT_DSL: &str = "\
skill hnl_capped_t8(990004) {
  trigger phase_end owner_is_target {
    effect change_max_hp(self, 1);
  }
}
";

// ---------------------------------------------------------------------------
// 恶意字节码拒收矩阵（T8 增量 5）：全部经数据路径注入，断言「无效果
// 落账、对局不 panic、机器回滚收尾」。
// ---------------------------------------------------------------------------

/// 推进到持有者阶段结束之后（VM 触发已发生），返回触发前后的手牌差。
fn drive_past_holder_phase_end(e: &mut Engine) {
    for _ in 0..40 {
        if e.turn().current != 0 {
            // 0 号回合收束后至少跨入 1 号回合（触发面已过）
            return;
        }
        if e.outcome().is_some() {
            panic!("对局提前结束");
        }
        np_settled(e);
    }
}

/// 拒收 ①：未映射效果（change_max_hp）——emit 时 InvalidRequest → 回滚，
/// 无效果落账。
#[test]
fn t8_rejection_unmapped_effect_rolls_back() {
    let pkg = compile_source_named("t8-rvm-bridge", UNMAPPED_EFFECT_DSL).expect("DSL 编译");
    let module = module_of(&pkg, "hnl_capped_t8");
    let name = "t8改上限";
    let pack = pack_with(vec![VmSkillDecl {
        name: name.to_string(),
        module,
        trigger: phase_end_trigger(),
        filter: engine_native::rvm_bridge::VmFilterPolicy::OwnerIsTarget,
    }]);
    let mut e = engine_setup(&pack, name, 930001);
    let h0 = hand_len(&e, 1);
    drive_past_holder_phase_end(&mut e);
    assert_eq!(hand_len(&e, 1), h0, "未映射效果：无落账（回滚协议）");
    assert_eq!(hp(&e, 1), 4, "体力上限未被改动");
}

/// 拒收 ②：字节码载体篡改（数据路径注入坏码）——PA06 起 verify_module
/// 在**装载期**（建目录）拒绝：含坏模块的包根本建不出对局，比原
/// 「首触发时执行期拒绝+回滚」更早；语义变化见台帐 PA06 证据。
#[test]
fn t8_rejection_corrupted_bytecode_rejected_at_load() {
    let pkg = compile_source_named("t8-rvm-bridge", JIUWEI_DSL).expect("DSL 编译");
    let mut module = module_of(&pkg, "hnl_jiuwei_t8");
    // 破坏码段首字节（装载期校验拒绝）
    if !module.code.is_empty() {
        module.code[0] ^= 0xFF;
    }
    let name = "t8坏码";
    let pack = pack_with(vec![VmSkillDecl {
        name: name.to_string(),
        module,
        trigger: phase_end_trigger(),
        filter: engine_native::rvm_bridge::VmFilterPolicy::OwnerIsTarget,
    }]);
    let err = match Engine::new_with_extensions(930002, 4, &[&pack]) {
        Ok(_) => panic!("含畸形 VM 模块的包应在装载期（建目录）被拒绝"),
        Err(e) => e,
    };
    assert_eq!(
        err,
        engine_native::content::extension::ExtensionError::InvalidVmModule {
            name: name.to_string()
        },
        "装载期以 InvalidVmModule 拒绝"
    );
}

/// 拒收 ③：能力越权（caps_mask 置未知位）——check_caps 拒绝，回滚。
#[test]
fn t8_rejection_caps_escalation_denied() {
    let pkg = compile_source_named("t8-rvm-bridge", JIUWEI_DSL).expect("DSL 编译");
    let mut module = module_of(&pkg, "hnl_jiuwei_t8");
    module.caps_mask |= 0xFFFF_0000; // 未知能力位
    let name = "t8越权";
    let pack = pack_with(vec![VmSkillDecl {
        name: name.to_string(),
        module,
        trigger: phase_end_trigger(),
        filter: engine_native::rvm_bridge::VmFilterPolicy::OwnerIsTarget,
    }]);
    let mut e = engine_setup(&pack, name, 930003);
    let h0 = hand_len(&e, 1);
    drive_past_holder_phase_end(&mut e);
    assert_eq!(hand_len(&e, 1), h0, "能力越权：check_caps 拒绝，无落账");
}

/// 拒收 ④：存档载体篡改——Rvm 快照模块字节被改后 restore 拒绝
///（from_bytes/哈希核对）。
#[test]
fn t8_rejection_tampered_snapshot_module_rejected() {
    let pkg = compile_source_named("t8-rvm-bridge", JIUWEI_DSL).expect("DSL 编译");
    let module = module_of(&pkg, "hnl_jiuwei_t8");
    let name = "t8酒尾";
    let pack = pack_with(vec![VmSkillDecl {
        name: name.to_string(),
        module,
        trigger: phase_end_trigger(),
        filter: engine_native::rvm_bridge::VmFilterPolicy::OwnerIsTarget,
    }]);
    let e = engine_setup(&pack, name, 930004);
    let snap = e.save();
    // NGSA 存档字节层面：翻转有效负载中部字节后解码须失败（魔数/校验
    // 和防线；针对 Rvm 载体字节的针尖级篡改由 check_machine 的
    // from_bytes/哈希核对承担——auth_archive 负例族同口径）。
    let mut bytes = engine_native::archive::encode_archive(&snap);
    let mid = bytes.len() / 2;
    bytes[mid] = bytes[mid].wrapping_add(1);
    assert!(
        engine_native::archive::decode_archive(&bytes).is_err(),
        "篡改存档须拒绝解码"
    );
}

fn engine_setup(pack: &ExtensionPack, name: &str, seed: u64) -> Engine {
    let mut e = Engine::new_with_extensions(seed, 4, &[pack]).expect("建局");
    e.debug_grant(1, name);
    e
}

/// owner_is_source 样本：持有者造成伤害后摸 1。
const SOURCE_DSL: &str = "\
skill hnl_src_t8(990005) {
  trigger damage owner_is_source {
    effect draw_cards(self, 1);
  }
}
";

/// always 样本：任意角色阶段结束摸 1（不受 owner 域限定）。
const ALWAYS_DSL: &str = "\
skill hnl_always_t8(990006) {
  trigger phase_end always {
    effect draw_cards(self, 1);
  }
}
";

/// 具名组合 filter 样本（装载器跳过面）。
const NAMED_FILTER_DSL: &str = "\
skill hnl_named_t8(990007) {
  trigger damage owner_is_target_wounded_sha {
    effect draw_cards(self, 1);
  }
}
";

/// 增量 6 过滤面：owner_is_source——持有者造成伤害触发、承受伤害不触
/// 发。
#[test]
fn t8_vm_skill_filter_owner_is_source() {
    let pkg = compile_source_named("t8-rvm-bridge", SOURCE_DSL).expect("DSL 编译");
    let module = module_of(&pkg, "hnl_src_t8");
    let name = "t8源";
    let pack = pack_with(vec![VmSkillDecl {
        name: name.to_string(),
        module,
        trigger: TriggerMeta {
            event: EventKind::Damage,
            timing: Timing::Normal,
            priority: 0,
            optional: false,
            extra_events: Vec::new(),
        },
        filter: engine_native::rvm_bridge::VmFilterPolicy::OwnerIsSource,
    }]);
    let mut e = Engine::new_with_extensions(930005, 4, &[&pack]).expect("建局");
    e.debug_grant(1, name);
    let h0 = hand_len(&e, 1);
    // 持有者造成伤害 → 触发摸 1
    let out = e.debug_deal_damage(Some(1), 2, 1, DamageNature::Normal);
    let _ = out;
    assert_eq!(hand_len(&e, 1), h0 + 1, "owner_is_source：造成伤害后摸 1");
    // 持有者承受伤害 → 不触发
    let out = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    let _ = out;
    assert_eq!(hand_len(&e, 1), h0 + 1, "owner_is_source：承受伤害不触发");
}

/// 增量 6 过滤面：always——任意角色阶段结束都触发（他人阶段尾即摸 1）。
#[test]
fn t8_vm_skill_filter_always() {
    let pkg = compile_source_named("t8-rvm-bridge", ALWAYS_DSL).expect("DSL 编译");
    let module = module_of(&pkg, "hnl_always_t8");
    let name = "t8常";
    let pack = pack_with(vec![VmSkillDecl {
        name: name.to_string(),
        module,
        trigger: phase_end_trigger(),
        filter: engine_native::rvm_bridge::VmFilterPolicy::Always,
    }]);
    let mut e = Engine::new_with_extensions(930006, 4, &[&pack]).expect("建局");
    e.debug_grant(1, name);
    let h0 = hand_len(&e, 1);
    // 推进到 0 号阶段结束之后：always 域即触发
    for _ in 0..20 {
        if hand_len(&e, 1) > h0 {
            break;
        }
        if e.outcome().is_some() {
            panic!("对局提前结束");
        }
        np_settled(&mut e);
    }
    assert!(hand_len(&e, 1) > h0, "always：他人阶段结束即触发");
}

/// 装载器：无法承载的 filter 名**装载期拒绝整包**并列出名称（PA12 批①；
/// 此前静默跳过会让技能装上但不触发）。
#[test]
fn t8_vm_skill_loader_rejects_unsupported_filters() {
    let mut pkg = compile_source_named("t8-rvm-bridge", NAMED_FILTER_DSL).expect("DSL 编译");
    pkg.trigger_index[0].filter = "not_a_real_filter".to_string();
    match content_native_loader::bridge_vm_skill_decls(&pkg) {
        Err(content_native_loader::BridgeError::UnsupportedVmFilters { filters }) => {
            assert_eq!(filters, vec!["not_a_real_filter".to_string()]);
        }
        other => panic!("应拒装不承载滤镜，实际 {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 跨进程恢复（T8 增量 7）：同宿主双进程存档接力——父进程挂起→NGSA 字
// 节落盘→子进程（本测试二进制自启，T8_WORKER 环境变量识别）恢复→续跑
// 落账→存档回写→父进程校验。C1 验收「跨进程恢复」面。
// ---------------------------------------------------------------------------

/// 跨进程恢复接力：父进程挂起→子进程续跑落账→回写→父进程校验。
#[test]
fn t8_vm_skill_cross_process_resume() {
    const DSL: &str = "\
skill hnl_xproc_t8(990008) {
  trigger phase_end owner_is_target {
    let option = choose(1, 2);
    if option == 1 {
      effect recover_hp(self, 1);
    }
    if option == 2 {
      effect draw_cards(self, 1);
    }
  }
}
";
    // 子进程模式：读存档→恢复→应答 Tag(1)→存档回写→退出
    if let Ok(Paths { archive, reply }) = worker_paths() {
        let bytes = std::fs::read(&archive).expect("子进程读存档");
        let snap = engine_native::archive::decode_archive(&bytes).expect("子进程解码");
        let pkg = compile_source_named("t8-rvm-bridge", DSL).expect("子进程 DSL 编译");
        let module = module_of(&pkg, "hnl_xproc_t8");
        let name = "t8跨进程";
        let pack = pack_with(vec![VmSkillDecl {
            name: name.to_string(),
            module,
            trigger: phase_end_trigger(),
            filter: engine_native::rvm_bridge::VmFilterPolicy::OwnerIsTarget,
        }]);
        let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("子进程恢复");
        let c = e.open_choice().expect("子进程挂起询问在位");
        assert_eq!(c.spec.prompt, "rvm_choice");
        let (cid, obs) = (c.id, c.observer);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Tag(1)],
            },
        );
        assert_eq!(hp(&e, 1), 3, "子进程续跑落账（2→3）");
        let out = engine_native::archive::encode_archive(&e.save());
        std::fs::write(&reply, out).expect("子进程回写");
        return;
    }

    // 父进程：建局→推进到挂起→落盘→自启子进程→校验
    let pkg = compile_source_named("t8-rvm-bridge", DSL).expect("DSL 编译");
    let module = module_of(&pkg, "hnl_xproc_t8");
    let name = "t8跨进程";
    let pack = pack_with(vec![VmSkillDecl {
        name: name.to_string(),
        module,
        trigger: phase_end_trigger(),
        filter: engine_native::rvm_bridge::VmFilterPolicy::OwnerIsTarget,
    }]);
    let mut e = Engine::new_with_extensions(930007, 4, &[&pack]).expect("建局");
    e.debug_grant(1, name);
    e.debug_set_hp(1, 2);
    for _ in 0..40 {
        let pending = e.open_choice().map(|c| {
            (c.id, c.observer, c.spec.prompt, c.spec.options.clone(), c.spec.min)
        });
        match pending {
            Some((cid, obs, prompt, options, min)) => {
                if prompt == "rvm_choice" {
                    break;
                }
                let sel: Vec<ChoiceOption> = options
                    .iter()
                    .filter(|o| matches!(o, ChoiceOption::Card(_)))
                    .copied()
                    .take(min.max(1))
                    .collect();
                let selection = if sel.is_empty() && !options.is_empty() {
                    vec![options[0].clone()]
                } else {
                    sel
                };
                submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection });
            }
            None => {
                if e.outcome().is_some() {
                    panic!("对局提前结束");
                }
                np_settled(&mut e);
            }
        }
    }
    assert!(e.open_choice().is_some(), "父进程挂起在位");

    let dir = tempfile::tempdir().expect("临时目录");
    let archive_path = dir.path().join("relay.ngsa");
    let reply_path = dir.path().join("relay.result.ngsa");
    std::fs::write(&archive_path, engine_native::archive::encode_archive(&e.save()))
        .expect("父进程落盘");

    // 自启子进程（同测试二进制，T8_WORKER 环境变量识别）
    let exe = std::env::current_exe().expect("测试二进制");
    let status = std::process::Command::new(exe)
        .args(["--exact", "t8_vm_skill_cross_process_resume", "--nocapture"])
        .env("T8_WORKER_ARCHIVE", archive_path.display().to_string())
        .env("T8_WORKER_REPLY", reply_path.display().to_string())
        .output()
        .expect("自启子进程");
    assert!(
        status.status.success(),
        "子进程续跑失败: {}",
        String::from_utf8_lossy(&status.stderr)
    );

    // 父进程校验回写存档：挂起已消费、hp 3
    let reply_bytes = std::fs::read(&reply_path).expect("读回写存档");
    let snap2 = engine_native::archive::decode_archive(&reply_bytes).expect("解码回写");
    let r = Engine::restore_with_extensions(&snap2, &[&pack]).expect("父进程恢复回写");
    assert_eq!(hp(&r, 1), 3, "跨进程接力：续跑效果在回写存档中");
}

/// 子进程接力路径参数（T8_WORKER_ARCHIVE/T8_WORKER_REPLY）。
struct Paths {
    archive: std::path::PathBuf,
    reply: std::path::PathBuf,
}

fn worker_paths() -> Result<Paths, std::env::VarError> {
    let archive = std::env::var("T8_WORKER_ARCHIVE")?;
    let reply = std::env::var("T8_WORKER_REPLY")?;
    Ok(Paths {
        archive: archive.into(),
        reply: reply.into(),
    })
}

/// 存档往返：VM 技能提交后对局可存档恢复（内容锁哈希入快照面）。
#[test]
fn t8_vm_skill_save_restore_roundtrip() {
    let pkg = compile_source_named("t8-rvm-bridge", JIUWEI_DSL).expect("DSL 编译");
    let module = module_of(&pkg, "hnl_jiuwei_t8");
    let name = "t8酒尾";
    let pack = pack_with(vec![VmSkillDecl {
        name: name.to_string(),
        module,
        trigger: phase_end_trigger(),
        filter: engine_native::rvm_bridge::VmFilterPolicy::OwnerIsTarget,
    }]);
    let mut e = Engine::new_with_extensions(910004, 4, &[&pack]).expect("建局");
    e.debug_grant(1, name);
    let snap = e.save();
    // 扩展局恢复须带同一扩展集合（内容锁：名称+版本指纹）
    let r = Engine::restore_with_extensions(&snap, &[&pack]).expect("往返");
    assert_eq!(
        r.catalog().skill_by_name(name),
        e.catalog().skill_by_name(name),
        "VM 技能 def 经内容锁恢复"
    );
}

/// T8 增量 8 定向：HostContext.skill_id 经目录反查回填（非 0）。
/// 观测面：DSL 侧无 skill_id 查询——以目录 def 查询同键核对装配一致性
/// （装配 def 名=声明名，program.skill_name 同源）。
#[test]
fn t8_vm_skill_id_reverse_lookup_wired() {
    let pkg = compile_source_named("t8-rvm-bridge", JIUWEI_DSL).expect("DSL 编译");
    let outcome = content_native_loader::bridge_vm_skill_decls(&pkg).expect("装载器桥接");
    let decls = outcome.decls;
    let pack = {
        // 复用通道测试的装配路径：npack 声明经 standard_catalog_with_extensions
        use engine_native::content::extension::{ExtensionDef, ExtensionPack, VmSkillDecl};
        use engine_native::catalog::TriggerMeta;
        use engine_native::event::{EventKind, Timing};
        ExtensionPack {
            def: ExtensionDef {
                name: "t8-skillid",
                version: 1,
                description: "skill_id 反查",
            },
            generals: Vec::new(),
            cards: Vec::new(),
            host_skills: Vec::new(),
            vm_skills: vec![VmSkillDecl {
                name: decls[0].name.clone(),
                module: decls[0].module.clone(),
                trigger: TriggerMeta {
                    event: EventKind::PhaseEnd,
                    timing: Timing::Normal,
                    priority: 0,
                    optional: false,
                    extra_events: Vec::new(),
                },
                filter: engine_native::rvm_bridge::VmFilterPolicy::OwnerIsTarget,
            }],
            init_hook: engine_native::content::extension::InitHook::None,
            enabled: true,
        }
    };
    let e = Engine::new_with_extensions(930001, 3, &[&pack]).expect("建局");
    let def = e
        .catalog()
        .skill_by_name(&decls[0].name)
        .unwrap_or_else(|| panic!("VM def 应在目录"));
    assert_ne!(def, 0, "def id 非 NONE（反查键有效）");
}

// ---------------------------------------------------------------------------
// 增量 13 过滤面收敛：`owner_is_target_*` 家族权威语义（编译器 lsp 权威
// 描述 "the skill owner is the event player and …"）=「事件主玩家=持有者」
// ∧ 状态谓词。12 策略逐策略双闸：非持有者事件不过闸（负例）、持有者事
// 件过闸（正例）；turn_damage 系另测「造成」口径（kernel 记来源侧），
// AtLeastHpOtherAlive 另测「手牌≥体力」半边（增量 11 实现曾丢）。
// ---------------------------------------------------------------------------

/// 家族探针样本：效果=摸 1；filter 由声明侧逐策略注入（DSL 侧 always
/// 仅占位）。
const GATE_DSL: &str = "\
skill hnl_gate_t8(990009) {
  trigger damage always {
    effect draw_cards(self, 1);
  }
}
";

#[test]
fn t8_filter_family_owner_gate_and_predicates() {
    use engine_native::rvm_bridge::VmFilterPolicy as P;

    let pkg = compile_source_named("t8-rvm-bridge", GATE_DSL).expect("DSL 编译");
    let module = module_of(&pkg, "hnl_gate_t8");
    let trig = TriggerMeta {
        event: EventKind::Damage,
        timing: Timing::Normal,
        priority: 0,
        optional: false,
        extra_events: Vec::new(),
    };
    let name = "t8闸";

    // 每策略一局：setup 布置谓词态 → 负例（非持有者事件不过闸）→ 正例
    // （本人为事件主玩家时过闸）。
    let cases: Vec<(P, fn(&mut Engine))> = vec![
        (P::OwnerWounded, |e: &mut Engine| e.debug_set_hp(1, 2)),
        (
            P::OwnerLinked,
            |e: &mut Engine| e.state_core_mut().players[1].linked = true,
        ),
        (P::OwnerHandNonempty, |_| {}),
        (P::OwnerHandEmpty, |e: &mut Engine| e.debug_clear_hand(1)),
        (P::OwnerNoTurnDamage, |_| {}),
        (P::OwnerTurnDamage, |_| {}),
        (P::OwnerTurnDamageAtLeastThree, |_| {}),
        (P::OwnerHandBelowMaxHp, |e: &mut Engine| e.debug_clear_hand(1)),
        (P::OwnerHandAboveHp, |e: &mut Engine| e.debug_set_hp(1, 2)),
        (P::OwnerHandAtLeastHp, |e: &mut Engine| e.debug_set_hp(1, 2)),
        (P::OwnerOtherHandNonempty, |_| {}),
        (
            P::OwnerHandAtLeastHpOtherAlive,
            |e: &mut Engine| e.debug_set_hp(1, 2),
        ),
    ];

    for (idx, (filter, setup)) in cases.iter().enumerate() {
        let pack = pack_with(vec![VmSkillDecl {
            name: name.to_string(),
            module: module.clone(),
            trigger: trig.clone(),
            filter: filter.clone(),
        }]);
        let mut e =
            Engine::new_with_extensions(941000 + idx as u64, 4, &[&pack]).expect("建局");
        e.debug_grant(1, name);
        setup(&mut e);
        let h0 = hand_len(&e, 1);

        match filter {
            // 造成伤害系：负例=持有者对他人造成伤害（事件主=受伤者≠持有
            // 者，同时把 turn_damage 布置过阈值）；正例=他人对持有者造成
            // 伤害。
            P::OwnerTurnDamage => {
                e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
                assert_eq!(
                    hand_len(&e, 1),
                    h0,
                    "turn_damage：持有者对他人造成伤害不过闸"
                );
                e.debug_deal_damage(Some(2), 1, 1, DamageNature::Normal);
                assert_eq!(hand_len(&e, 1), h0 + 1, "turn_damage：本人受伤过闸");
            }
            P::OwnerTurnDamageAtLeastThree => {
                e.debug_deal_damage(Some(1), 0, 3, DamageNature::Normal);
                assert_eq!(
                    hand_len(&e, 1),
                    h0,
                    "turn_damage_3：持有者对他人造成伤害不过闸"
                );
                e.debug_deal_damage(Some(2), 1, 1, DamageNature::Normal);
                assert_eq!(
                    hand_len(&e, 1),
                    h0 + 1,
                    "turn_damage_3：本人受伤过闸（已造成 3）"
                );
            }
            // 手牌≥体力半边专项：先令谓词假（体力 6 > 手牌 4）——本人受
            // 伤也不触发（增量 11 丢半边反例）；再回体力 2 走双闸。
            P::OwnerHandAtLeastHpOtherAlive => {
                e.debug_set_hp(1, 6);
                e.debug_deal_damage(Some(2), 1, 1, DamageNature::Normal);
                assert_eq!(
                    hand_len(&e, 1),
                    h0,
                    "at_least_hp_other_alive：手牌<体力谓词假不触发"
                );
                e.debug_set_hp(1, 2);
                e.debug_deal_damage(Some(2), 1, 1, DamageNature::Normal);
                assert_eq!(
                    hand_len(&e, 1),
                    h0 + 1,
                    "at_least_hp_other_alive：本人受伤过闸"
                );
                e.debug_deal_damage(Some(2), 0, 1, DamageNature::Normal);
                assert_eq!(
                    hand_len(&e, 1),
                    h0 + 1,
                    "at_least_hp_other_alive：他人受伤不过闸"
                );
            }
            _ => {
                e.debug_deal_damage(Some(2), 0, 1, DamageNature::Normal);
                assert_eq!(hand_len(&e, 1), h0, "{filter:?}：他人受伤不过闸");
                e.debug_deal_damage(Some(2), 1, 1, DamageNature::Normal);
                assert_eq!(hand_len(&e, 1), h0 + 1, "{filter:?}：本人受伤过闸");
            }
        }
    }
}

/// 增量 13 装载器映射正向锁面：词表中已承载的 11 个具名组合 filter 逐
/// 名映射到对应策略（映射遗漏现在会装载期拒装）。
#[test]
fn t8_loader_maps_all_carried_named_filters() {
    use engine_native::rvm_bridge::VmFilterPolicy as P;

    let mut dsl = String::new();
    // 词表差集说明：`owner_is_target_hand_empty` 不在编译器 FILTERS 词表
    // （装载器映射为前进兼容死臂，策略行为由家族直注测试覆盖）。
    let named: [(&str, &str, P); 19] = [
        ("wounded", "owner_is_target_wounded", P::OwnerWounded),
        ("linked", "owner_is_target_linked", P::OwnerLinked),
        (
            "hand_nonempty",
            "owner_is_target_hand_nonempty",
            P::OwnerHandNonempty,
        ),
        (
            "no_turn_damage",
            "owner_is_target_no_turn_damage",
            P::OwnerNoTurnDamage,
        ),
        ("turn_damage", "owner_is_target_turn_damage", P::OwnerTurnDamage),
        (
            "turn_damage_at_least_three",
            "owner_is_target_turn_damage_at_least_three",
            P::OwnerTurnDamageAtLeastThree,
        ),
        (
            "hand_at_least_hp_other_alive",
            "owner_is_target_hand_at_least_hp_other_alive",
            P::OwnerHandAtLeastHpOtherAlive,
        ),
        (
            "hand_below_max_hp",
            "owner_is_target_hand_below_max_hp",
            P::OwnerHandBelowMaxHp,
        ),
        (
            "hand_above_hp",
            "owner_is_target_hand_above_hp",
            P::OwnerHandAboveHp,
        ),
        (
            "other_hand_nonempty",
            "owner_is_target_other_hand_nonempty",
            P::OwnerOtherHandNonempty,
        ),
        (
            "play_bte",
            "owner_is_target_play_phase_card_category_count_positive",
            P::OwnerPlayPhaseBteUsed,
        ),
        (
            "alive_play_bte",
            "owner_alive_play_phase_card_category_count_positive",
            P::OwnerAlivePlayPhaseBteUsed,
        ),
        (
            "dbf_hand_empty",
            "owner_target_hand_empty_elemental_or_trick_damage_before",
            P::OwnerHandEmptyElementalOrTrickDamageBefore,
        ),
        (
            "movable_field",
            "owner_is_target_source_movable_field_card",
            P::OwnerSourceMovableFieldCard,
        ),
        (
            "tied_max_hand",
            "owner_is_target_tied_max_hand",
            P::OwnerTiedMaxHand,
        ),
        (
            "tied_max_hp",
            "owner_is_target_tied_max_hp",
            P::OwnerTiedMaxHp,
        ),
        (
            "tied_max_equip",
            "owner_is_target_tied_max_equip_nonempty",
            P::OwnerTiedMaxEquipNonempty,
        ),
        (
            "not_unique_max_hp",
            "owner_is_target_not_unique_max_current_hp",
            P::OwnerNotUniqueMaxHp,
        ),
        (
            "targeted_by_sha",
            "owner_targeted_by_other_sha",
            P::OwnerTargetedByOtherSha,
        ),
    ];
    for (idx, (tag, filter, _)) in named.iter().enumerate() {
        // dbf_hand_empty 走 damage_before 触发面（ABI 358 语义事件种）
        let event = if *tag == "dbf_hand_empty" { "damage_before" } else { "damage" };
        dsl.push_str(&format!(
            "skill hnl_ld_{tag}(99{:02}) {{\n  trigger {event} {filter} {{\n    effect draw_cards(self, 1);\n  }}\n}}\n",
            10 + idx
        ));
    }
    let pkg = compile_source_named("t8-rvm-bridge", &dsl).expect("DSL 编译");
    let outcome = content_native_loader::bridge_vm_skill_decls(&pkg).expect("桥接");
    assert_eq!(outcome.decls.len(), 19, "词表内 19 名全产出声明");
    for (tag, _, want) in named.iter() {
        let decl = outcome
            .decls
            .iter()
            .find(|d| d.name.contains(&format!("hnl_ld_{tag}")))
            .unwrap_or_else(|| panic!("hnl_ld_{tag} 声明缺失"));
        assert_eq!(decl.filter, *want, "hnl_ld_{tag} 策略映射");
    }
}

// ---------------------------------------------------------------------------
// 增量 14：出牌阶段基本/锦囊/装备三类计数（TurnState.bte_used_play_phase，
// 存档 v65/规则 v278）承载 `*_play_phase_card_category_count_positive` 双
// 策略。零询问布置（装备/延时种）；阶段种负例=弃牌阶段结束时计数仍非零
// 但事件非 Play 结束。
// ---------------------------------------------------------------------------

/// 出牌阶段结束探针：效果=摸 1（filter 由声明侧注入）。
const BTE_DSL: &str = "\
skill hnl_bte_t8(990010) {
  trigger phase_end always {
    effect draw_cards(self, 1);
  }
}
";

/// 推进到指定玩家的指定阶段（途中询问以空/首选应答排空）。
fn advance_to(e: &mut Engine, player: u8, phase: Phase) {
    for _ in 0..200 {
        if e.turn().current == player && e.turn().phase == phase {
            return;
        }
        assert!(e.outcome().is_none(), "对局提前结束");
        np_settled(e);
    }
    panic!("200 步未到 p{player} 目标阶段");
}

#[test]
fn t8_bte_play_phase_filter_dual_gate_and_phase_kind() {
    use engine_native::rvm_bridge::VmFilterPolicy as P;

    let pkg = compile_source_named("t8-rvm-bridge", BTE_DSL).expect("DSL 编译");
    let module = module_of(&pkg, "hnl_bte_t8");
    let trig = TriggerMeta {
        event: EventKind::PhaseEnd,
        timing: Timing::Normal,
        priority: 0,
        optional: false,
        extra_events: Vec::new(),
    };
    let name = "t8阵";

    for (idx, filter) in [P::OwnerPlayPhaseBteUsed, P::OwnerAlivePlayPhaseBteUsed]
        .into_iter()
        .enumerate()
    {
        let pack = pack_with(vec![VmSkillDecl {
            name: name.to_string(),
            module: module.clone(),
            trigger: trig.clone(),
            filter: filter.clone(),
        }]);
        let mut e = Engine::new_with_extensions(951000 + idx as u64, 4, &[&pack])
            .expect("建局");
        e.debug_grant(1, name);

        // ① gate 负例：玩家 0 出牌阶段出杀（bte=1；装备牌走 Equipped 帧
        //    不经 commit_card_used，与 cards_used_play_phase 同集）→ 出牌
        //    阶段结束时事件主玩家=0≠持有者 1，不触发。目标选 3（非持有
        //    者，距离 1）。
        advance_to(&mut e, 0, Phase::Play);
        let sha0 = e.debug_place(0, "sha");
        submit(&mut e, 0, GameAction::PlayCard { card: sha0, targets: vec![3] });
        assert_eq!(
            e.state_core().turn.bte_used_play_phase, 1,
            "基本牌计入三类计数"
        );
        let h1 = hand_len(&e, 1);
        advance_to(&mut e, 0, Phase::Discard);
        assert_eq!(hand_len(&e, 1), h1, "他人 Play 结束不过闸（双策略）");

        // ② 正例：玩家 1 出牌阶段出杀 → Play 结束触发摸 1。
        advance_to(&mut e, 1, Phase::Play);
        let sha1 = e.debug_place(1, "sha");
        submit(&mut e, 1, GameAction::PlayCard { card: sha1, targets: vec![0] });
        // 结算链时序（np_settled 同链）：Play 结束触发入队 → 阶段推进弃
        // 牌 1 张 → 队首触发机结算摸 1。手牌 4(初始)+2(摸)-1(杀)-1(弃)
        // +1(触发)=5。
        advance_to(&mut e, 1, Phase::Discard);
        assert_eq!(hand_len(&e, 1), 5, "{filter:?}: Play 结束触发摸 1");
        // 阶段种负例：计数仍为 1，但弃牌阶段结束事件非 Play 结束 → 不再
        // 触发（若误触发此处为 6）。
        advance_to(&mut e, 1, Phase::End);
        assert_eq!(hand_len(&e, 1), 5, "{filter:?}: 弃牌阶段结束不再触发");
    }
}

#[test]
fn t8_bte_zero_count_negative() {
    use engine_native::rvm_bridge::VmFilterPolicy as P;

    let pkg = compile_source_named("t8-rvm-bridge", BTE_DSL).expect("DSL 编译");
    let module = module_of(&pkg, "hnl_bte_t8");
    let pack = pack_with(vec![VmSkillDecl {
        name: "t8阵".to_string(),
        module,
        trigger: TriggerMeta {
            event: EventKind::PhaseEnd,
            timing: Timing::Normal,
            priority: 0,
            optional: false,
            extra_events: Vec::new(),
        },
        filter: P::OwnerPlayPhaseBteUsed,
    }]);
    let mut e = Engine::new_with_extensions(952000, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "t8阵");
    // 玩家 1 出牌阶段不出牌直接结束：计数 0 → 不触发（摸 2 张原样保留，
    // 弃牌回上限后=4；若误触发=5）。
    advance_to(&mut e, 1, Phase::Play);
    assert_eq!(e.state_core().turn.bte_used_play_phase, 0);
    advance_to(&mut e, 1, Phase::Discard);
    assert_eq!(hand_len(&e, 1), 4, "计数 0 不触发");
}

/// 三类分类判定单测：异类（地十种+机关牌）排除、基本/装备/锦囊计入。
#[test]
fn t8_bte_kind_classification() {
    use engine_native::catalog::CardKind as K;
    assert!(K::Sha.is_basic_trick_or_equip());
    assert!(K::Jiu.is_basic_trick_or_equip());
    assert!(K::Weapon.is_basic_trick_or_equip());
    assert!(K::Armor.is_basic_trick_or_equip());
    assert!(K::ArcheryAttack.is_basic_trick_or_equip(), "锦囊计入");
    assert!(K::Le.is_basic_trick_or_equip(), "延时锦囊计入");
    for land in [
        K::Longlushanfeng,
        K::Haidao,
        K::Shuimomuxue,
        K::Feixu,
        K::Yixialan,
        K::Shamolvzhou,
        K::Linzhongjianta,
        K::Bingheyaosai,
        K::Youlin,
        K::Lindixiliu,
    ] {
        assert!(!land.is_basic_trick_or_equip(), "{land:?} 地不计入");
    }
    assert!(!K::Jiguan.is_basic_trick_or_equip(), "机关牌不计入");
}

// ---------------------------------------------------------------------------
// 增量 15：参数化/计数族 filter——hp_below_{N}（hp<N）、play_card_count_eq_{N}
// /eq_hp（第 N 张主动用牌，commit 后序数口径）、card_used_at_least_hp（主动
// 用牌 ≥ 体力，出牌阶段计数化归）。触发效果=摸 1，以日志 CardsDrawn(p1,1)
// 计数锚定（规避无懈/摸牌阶段账目干扰）。
// ---------------------------------------------------------------------------

/// 触发探针构造：filter 由声明侧注入。
fn param_probe_pack(
    module: &rule_vm::BytecodeModule,
    filter: engine_native::rvm_bridge::VmFilterPolicy,
    event: EventKind,
) -> ExtensionPack {
    pack_with(vec![VmSkillDecl {
        name: "t8参".to_string(),
        module: module.clone(),
        trigger: TriggerMeta {
            event,
            timing: Timing::Normal,
            priority: 0,
            optional: false,
            extra_events: Vec::new(),
        },
        filter,
    }])
}

/// 日志中某座位 n=1 摸牌次数（触发效果计数）。
fn trigger_draws(e: &Engine) -> usize {
    trigger_draws_of(e, 1)
}

fn trigger_draws_of(e: &Engine, player: u8) -> usize {
    e.log()
        .iter()
        .filter(|ev| {
            matches!(
                ev,
                engine_native::machine::LogEvent::CardsDrawn { player: p, cards } if *p == player && cards.len() == 1
            )
        })
        .count()
}

#[test]
fn t8_filter_param_hp_below_dual_gate_and_threshold() {
    use engine_native::rvm_bridge::VmFilterPolicy as P;

    let pkg = compile_source_named(
        "t8-rvm-bridge",
        "skill hnl_php_t8(990011) {\n  trigger damage always {\n    effect draw_cards(self, 1);\n  }\n}\n",
    )
    .expect("DSL 编译");
    let module = module_of(&pkg, "hnl_php_t8");
    let pack = param_probe_pack(&module, P::OwnerHpBelow(2), EventKind::Damage);
    let mut e = Engine::new_with_extensions(953000, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "t8参");
    // 阈值边（口径=事件时点体力，damage 帧在 commit 扣血后评估 filter）：
    // 扣血后 hp=2 不满足 2<2 → 不触发
    e.debug_set_hp(1, 3);
    e.debug_deal_damage(Some(2), 1, 1, DamageNature::Normal);
    assert_eq!(trigger_draws(&e), 0, "hp=N 阈值边不触发");
    // 扣血后 hp=1 < 2 → 触发
    e.debug_set_hp(1, 2);
    e.debug_deal_damage(Some(2), 1, 1, DamageNature::Normal);
    assert_eq!(trigger_draws(&e), 1, "hp<N 触发");
    // owner 门：他人受伤不触发（谓词对持有者成立）
    e.debug_deal_damage(Some(2), 0, 1, DamageNature::Normal);
    assert_eq!(trigger_draws(&e), 1, "他人受伤不过闸");
}

/// 出两张牌（无中生有+闪电：均自目标、计数面走 commit_card_used），
/// 第 2 张 commit 后序数=2。
/// 只应答开放选择、不推进阶段（出牌后挂起窗排空）。
fn settle_choices(e: &mut Engine) {
    for _ in 0..8 {
        let Some(c) = e.open_choice() else { return };
        let sel: Vec<ChoiceOption> = c
            .spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Card(_)))
            .copied()
            .take(c.spec.min.max(1))
            .collect();
        let (cid, obs) = (c.id, c.observer);
        if sel.is_empty() && !c.spec.options.is_empty() {
            submit(e, obs, GameAction::Answer { choice_id: cid, selection: vec![c.spec.options[0].clone()] });
        } else if !sel.is_empty() {
            submit(e, obs, GameAction::Answer { choice_id: cid, selection: sel });
        }
    }
}

fn play_two_cards(e: &mut Engine) {
    advance_to(e, 1, Phase::Play);
    for name in ["wuzhong", "shandian"] {
        let id = e.debug_place(1, name);
        submit(e, 1, GameAction::PlayCard { card: id, targets: vec![] });
        settle_choices(e);
    }
}

#[test]
fn t8_filter_param_play_count_and_at_least_hp() {
    use engine_native::rvm_bridge::VmFilterPolicy as P;

    let dsl = "\
skill hnl_pcnt_t8(990012) {
  trigger card_used always {
    effect draw_cards(self, 1);
  }
}
";
    let pkg = compile_source_named("t8-rvm-bridge", dsl).expect("DSL 编译");
    let module = module_of(&pkg, "hnl_pcnt_t8");

    // eq_hp：hp=2 → 第 2 张主动用牌时触发（commit 后序数口径）
    let pack = param_probe_pack(&module, P::OwnerPlayCountEqHp, EventKind::CardUsed);
    let mut e = Engine::new_with_extensions(954000, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "t8参");
    e.debug_set_hp(1, 2);
    play_two_cards(&mut e);
    assert_eq!(trigger_draws(&e), 1, "eq_hp：第 hp 张牌使用时触发");

    // eq_2：同口径参数化
    let pack = param_probe_pack(&module, P::OwnerPlayCountEq(2), EventKind::CardUsed);
    let mut e = Engine::new_with_extensions(954001, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "t8参");
    play_two_cards(&mut e);
    assert_eq!(trigger_draws(&e), 1, "eq_2：第 2 张牌使用时触发");

    // at_least_hp：hp=2 → 第 2 张触发；hp=3 → 第 2 张不触发
    let pack = param_probe_pack(&module, P::OwnerCardUsedAtLeastHp, EventKind::CardUsed);
    let mut e = Engine::new_with_extensions(954002, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "t8参");
    e.debug_set_hp(1, 3);
    play_two_cards(&mut e);
    assert_eq!(trigger_draws(&e), 0, "at_least_hp：2<3 不触发");
    // 推进到持有者下一回合（计数随 Play 开始清零），hp 改 2：第 1 张
    // count=1<2 不触发、第 2 张 count=2≥2 触发。
    np_settled(&mut e);
    e.debug_set_hp(1, 2);
    play_two_cards(&mut e);
    assert_eq!(trigger_draws(&e), 1, "at_least_hp：达阈值牌触发");
}

/// 装载器参数化解析锁面：4 名（2 精确+2 参数化）经 DSL 编译→桥接产出
/// 声明、deferred 空、变体逐一对。
#[test]
fn t8_loader_maps_parameterized_filters() {
    use engine_native::rvm_bridge::VmFilterPolicy as P;

    let dsl = "\
skill hnl_pz_a(990013) {
  trigger damage owner_is_target_hp_below_2 {
    effect draw_cards(self, 1);
  }
}
skill hnl_pz_b(990014) {
  trigger card_used owner_source_play_card_count_eq_2 {
    effect draw_cards(self, 1);
  }
}
skill hnl_pz_c(990015) {
  trigger card_used owner_source_play_card_count_eq_hp {
    effect draw_cards(self, 1);
  }
}
skill hnl_pz_d(990016) {
  trigger card_used owner_is_target_card_used_at_least_hp {
    effect draw_cards(self, 1);
  }
}
";
    let pkg = compile_source_named("t8-rvm-bridge", dsl).expect("DSL 编译");
    let outcome = content_native_loader::bridge_vm_skill_decls(&pkg).expect("桥接");
    let want = [
        ("hnl_pz_a", P::OwnerHpBelow(2)),
        ("hnl_pz_b", P::OwnerPlayCountEq(2)),
        ("hnl_pz_c", P::OwnerPlayCountEqHp),
        ("hnl_pz_d", P::OwnerCardUsedAtLeastHp),
    ];
    assert_eq!(outcome.decls.len(), 4);
    for (tag, policy) in want {
        let decl = outcome
            .decls
            .iter()
            .find(|d| d.name.contains(tag))
            .unwrap_or_else(|| panic!("{tag} 声明缺失"));
        assert_eq!(decl.filter, policy, "{tag} 策略映射");
    }
}

// ---------------------------------------------------------------------------
// 增量 16：伤害面前置 filter——hand_empty_elemental_or_trick_damage_before
// （ABI 358：手空 ∧ 元素性 ∨ 事件卡普通锦囊）与 source_movable_field_card
// （ABI 202：来源存活 ∧ 他人场牌可移至来源）。装载器事件面补 damage_before。
// ---------------------------------------------------------------------------

const DBF_DSL: &str = "\
skill hnl_dbf_t8(990017) {
  trigger damage_before always {
    effect draw_cards(self, 1);
  }
}
";

#[test]
fn t8_filter_damage_before_hand_empty_elemental_or_trick() {
    use engine_native::rvm_bridge::VmFilterPolicy as P;

    let pkg = compile_source_named("t8-rvm-bridge", DBF_DSL).expect("DSL 编译");
    let module = module_of(&pkg, "hnl_dbf_t8");
    let pack = pack_with(vec![VmSkillDecl {
        name: "t8伤".to_string(),
        module,
        trigger: TriggerMeta {
            event: EventKind::DamageBefore,
            timing: Timing::Normal,
            priority: 0,
            optional: false,
            extra_events: Vec::new(),
        },
        filter: P::OwnerHandEmptyElementalOrTrickDamageBefore,
    }]);
    let mut e = Engine::new_with_extensions(955000, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "t8伤");
    e.debug_set_hp(1, 8);
    e.debug_clear_hand(1);

    // 正例（元素性）：火伤 → 触发
    e.debug_deal_damage(Some(2), 1, 1, DamageNature::Fire);
    assert_eq!(trigger_draws(&e), 1, "元素性伤害前置触发");
    // 谓词负例：手牌非空
    let tao = e.debug_place(1, "tao");
    e.debug_deal_damage(Some(2), 1, 1, DamageNature::Fire);
    assert_eq!(trigger_draws(&e), 1, "手牌非空不触发");
    let _ = tao;
    e.debug_clear_hand(1);
    // 谓词负例：普通伤害无卡（非元素非锦囊）
    e.debug_deal_damage(Some(2), 1, 1, DamageNature::Normal);
    assert_eq!(trigger_draws(&e), 1, "无卡普通伤害不触发");
    // 正例（普通锦囊卡）：事件卡=无中生有（锦囊）→ 触发
    let wuzhong = e.debug_place(0, "wuzhong");
    e.debug_deal_damage_with_card(Some(2), 1, 1, DamageNature::Normal, wuzhong);
    assert_eq!(trigger_draws(&e), 2, "事件卡普通锦囊触发");
    // 谓词负例：延时锦囊卡（乐）非普通锦囊
    let le = e.debug_place(0, "le");
    e.debug_deal_damage_with_card(Some(2), 1, 1, DamageNature::Normal, le);
    assert_eq!(trigger_draws(&e), 2, "延时锦囊卡不触发");
    // owner 门：他人受伤（手空、火伤）
    e.debug_deal_damage(Some(2), 0, 1, DamageNature::Fire);
    assert_eq!(trigger_draws(&e), 2, "他人受伤不过闸");
}

#[test]
fn t8_filter_damage_source_movable_field_card() {
    use engine_native::rvm_bridge::VmFilterPolicy as P;

    let pkg = compile_source_named(
        "t8-rvm-bridge",
        "skill hnl_mvf_t8(990018) {\n  trigger damage always {\n    effect draw_cards(self, 1);\n  }\n}\n",
    )
    .expect("DSL 编译");
    let module = module_of(&pkg, "hnl_mvf_t8");
    let pack = pack_with(vec![VmSkillDecl {
        name: "t8移".to_string(),
        module,
        trigger: TriggerMeta {
            event: EventKind::Damage,
            timing: Timing::Normal,
            priority: 0,
            optional: false,
            extra_events: Vec::new(),
        },
        filter: P::OwnerSourceMovableFieldCard,
    }]);
    let mut e = Engine::new_with_extensions(956000, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "t8移");
    e.debug_set_hp(1, 8);

    // 负例：全场无场牌
    e.debug_deal_damage(Some(2), 1, 1, DamageNature::Normal);
    assert_eq!(trigger_draws(&e), 0, "无场牌不触发");
    // 正例：p3 场上有武器、来源 p2 武器栏空 → 可移
    let zhuge = e.debug_place(3, "zhuge");
    e.state_core_mut().players[3].equips.push(zhuge);
    e.debug_deal_damage(Some(2), 1, 1, DamageNature::Normal);
    assert_eq!(trigger_draws(&e), 1, "他人装备可移至来源触发");
    // 负例：来源同栏被占（p2 也挂武器）
    let zhuge2 = e.debug_place(2, "zhuge");
    e.state_core_mut().players[2].equips.push(zhuge2);
    e.debug_deal_damage(Some(2), 1, 1, DamageNature::Normal);
    assert_eq!(trigger_draws(&e), 1, "来源栏被占不可移");
    // 正例：判定区延时可移（来源判定区无同名种）
    let shandian = e.debug_place(3, "shandian");
    e.state_core_mut().players[3].judge.push(shandian);
    e.debug_deal_damage(Some(2), 1, 1, DamageNature::Normal);
    assert_eq!(trigger_draws(&e), 2, "判定区延时可移触发");
    // 负例：来源即持牌者（p3 自己受伤）——"another"不含来源
    e.debug_deal_damage(Some(3), 1, 1, DamageNature::Normal);
    assert_eq!(trigger_draws(&e), 2, "来源自己的场牌不计");
    // owner 门：受伤者非持有者
    e.debug_deal_damage(Some(2), 0, 1, DamageNature::Normal);
    assert_eq!(trigger_draws(&e), 2, "他人受伤不过闸");
}

// ---------------------------------------------------------------------------
// 增量 17：并列最大查询族（ABI 55 三兄弟 + not_unique_max_hp）与
// targeted_by_other_sha（原生以 CardUsed+targets 表达，登记差异）。
// ---------------------------------------------------------------------------

#[test]
fn t8_filter_tied_max_family() {
    use engine_native::rvm_bridge::VmFilterPolicy as P;

    let pkg = compile_source_named("t8-rvm-bridge", BTE_DSL).expect("DSL 编译");
    let module = module_of(&pkg, "hnl_bte_t8");
    let trig = TriggerMeta {
        event: EventKind::PhaseEnd,
        timing: Timing::Normal,
        priority: 0,
        optional: false,
        extra_events: Vec::new(),
    };
    let name = "t8并";

    // 断言用日志摸牌计数（trigger_draws）而非手牌增量——np_settled 的
    // 一步会推进阶段并触发 Draw 摸牌，污染手牌账。
    let case = |filter: P, setup: fn(&mut Engine)| -> bool {
        let pack = pack_with(vec![VmSkillDecl {
            name: name.to_string(),
            module: module.clone(),
            trigger: trig.clone(),
            filter,
        }]);
        let mut e = Engine::new_with_extensions(957000, 4, &[&pack]).expect("建局");
        e.debug_grant(0, name);
        setup(&mut e);
        np_settled(&mut e);
        trigger_draws_of(&e, 0) == 1
    };

    // tied_max_hand：初始 4 人各 4 张并列 → 触发；p1 手牌压到 8（p0 即便
    // 经 Draw 摸 2 至 6 仍非并列）→ 恒不触发
    assert!(case(P::OwnerTiedMaxHand, |_| {}), "手牌并列最大触发");
    assert!(!case(P::OwnerTiedMaxHand, |e: &mut Engine| {
        for card_name in ["tao", "sha", "wuzhong", "shandian"] {
            let extra = e.debug_place(1, card_name);
            e.state_core_mut().players[1].hand.push(extra);
        }
    }), "他人手牌更多不触发");

    // tied_max_hp：初始全 4 并列 → 触发；p1 体力更高 → 不触发
    assert!(case(P::OwnerTiedMaxHp, |_| {}), "体力并列最大触发");
    assert!(!case(P::OwnerTiedMaxHp, |e: &mut Engine| {
        e.debug_set_hp(1, 6);
    }), "他人体力更高不触发");

    // tied_max_equip_nonempty：空装备不触发；p0 挂武器并列最大（唯一）触发
    assert!(!case(P::OwnerTiedMaxEquipNonempty, |_| {}), "无装备不触发");
    assert!(case(P::OwnerTiedMaxEquipNonempty, |e: &mut Engine| {
        let z = e.debug_place(0, "zhuge");
        e.state_core_mut().players[0].equips.push(z);
    }), "有装备且并列最大触发");

    // not_unique_max_hp：全 4 并列 → 触发；p0 严格最高 → 不触发
    assert!(case(P::OwnerNotUniqueMaxHp, |_| {}), "体力非严格最高触发");
    assert!(!case(P::OwnerNotUniqueMaxHp, |e: &mut Engine| {
        e.debug_set_hp(1, 3);
        e.debug_set_hp(2, 3);
        e.debug_set_hp(3, 3);
    }), "严格最高不触发");
}

#[test]
fn t8_filter_targeted_by_other_sha() {
    use engine_native::rvm_bridge::VmFilterPolicy as P;

    let pkg = compile_source_named(
        "t8-rvm-bridge",
        "skill hnl_tgt_t8(990019) {\n  trigger card_used always {\n    effect draw_cards(self, 1);\n  }\n}\n",
    )
    .expect("DSL 编译");
    let module = module_of(&pkg, "hnl_tgt_t8");
    let pack = pack_with(vec![VmSkillDecl {
        name: "t8指".to_string(),
        module,
        trigger: TriggerMeta {
            event: EventKind::CardUsed,
            timing: Timing::Normal,
            priority: 0,
            optional: false,
            extra_events: Vec::new(),
        },
        filter: P::OwnerTargetedByOtherSha,
    }]);
    let mut e = Engine::new_with_extensions(958000, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "t8指");
    e.debug_set_hp(1, 8);

    // p2 杀 p1（持有者为目标）→ 触发
    advance_to(&mut e, 2, Phase::Play);
    let sha = e.debug_place(2, "sha");
    submit(&mut e, 2, GameAction::PlayCard { card: sha, targets: vec![1] });
    settle_choices(&mut e);
    assert_eq!(trigger_draws(&e), 1, "被他人杀指定触发");

    // p2 杀 p0（非持有者）→ 不触发
    advance_to(&mut e, 2, Phase::Play);
    let sha = e.debug_place(2, "sha");
    submit(&mut e, 2, GameAction::PlayCard { card: sha, targets: vec![0] });
    settle_choices(&mut e);
    assert_eq!(trigger_draws(&e), 1, "目标非持有者不触发");

    // p1 自己出杀指 p0（使用者=持有者）→ 不触发
    advance_to(&mut e, 1, Phase::Play);
    let sha = e.debug_place(1, "sha");
    submit(&mut e, 1, GameAction::PlayCard { card: sha, targets: vec![0] });
    settle_choices(&mut e);
    assert_eq!(trigger_draws(&e), 1, "使用者本人不计");
}

fn install_dsl(dsl: &str) -> ExtensionPack {
    let pkg = compile_source_named("t8-rvm-bridge", dsl).expect("DSL 编译");
    let outcome = content_native_loader::bridge_vm_skill_decls(&pkg).expect("装载器");
    assert!(!outcome.decls.is_empty(), "装载器应产出声明");
    pack_with(outcome.decls)
}

fn keep_only(e: &mut Engine, seat: u8, names: &[&str]) {
    let ids: Vec<_> = names
        .iter()
        .map(|n| e.catalog().skill_by_name(n).unwrap_or_else(|| panic!("无技能 {n}")))
        .collect();
    e.state_core_mut()
        .skills
        .retain(|(o, d), _| *o != seat || ids.contains(d));
}

fn sha_id(e: &mut Engine) -> u32 {
    e.debug_place(0, "sha")
}

/// 词表里的每个 filter 都能映射；非 owner 未知名不能。
#[test]
fn t8_named_filter_vocabulary_maps() {
    use engine_native::rvm_bridge::{map_vm_filter, VmFilterPolicy};
    use engine_native::rvm_named::Atom;
    for name in content_compiler::filter_vocabulary() {
        let mapped = map_vm_filter(name).unwrap_or_else(|| panic!("未映射 {name}"));
        if let VmFilterPolicy::Named(prog) = &mapped {
            if name.starts_with("owner_is_target_") {
                assert!(
                    prog.groups.iter().all(|g| {
                        g.iter().any(|c| !c.neg && c.atom == Atom::EvPlayer)
                    }),
                    "{name} 缺少事件主玩家合取"
                );
            }
        }
    }
    assert!(map_vm_filter("not_a_real_filter").is_none());
}

/// 受伤且伤害来自杀：摸 1。满血、他人受伤、非杀都不摸。
///
/// lsp：`owner_is_target_wounded_sha` 是伤害前置窗。`damage` 在扣血之后
/// 才过滤，满血承受 1 点伤害时已经受伤，所以这里走 `damage_before`。
#[test]
fn t8_named_wounded_sha_dual_gate() {
    let dsl = "\
skill hnl_wsha(990020) {
  trigger damage_before owner_is_target_wounded_sha {
    effect draw_cards(self, 1);
  }
}
";
    let pack = install_dsl(dsl);
    let mut e = Engine::new_with_extensions(961001, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "hnl_wsha_damage_before");
    keep_only(&mut e, 1, &["hnl_wsha_damage_before"]);
    let max = e.state_core().player(1).max_hp as i32;
    e.debug_set_hp(1, max);
    let h = hand_len(&e, 1);
    let sha = sha_id(&mut e);
    e.debug_deal_damage_with_card(Some(0), 1, 1, DamageNature::Normal, sha);
    assert_eq!(hand_len(&e, 1), h, "满血杀伤前置不摸");
    e.debug_set_hp(1, max - 1);
    e.debug_deal_damage_with_card(Some(0), 2, 1, DamageNature::Normal, sha);
    assert_eq!(hand_len(&e, 1), h, "他人承受杀伤不摸");
    e.debug_deal_damage_with_card(Some(0), 1, 1, DamageNature::Normal, sha);
    assert_eq!(hand_len(&e, 1), h + 1, "受伤且杀伤摸 1");
    let shan = e.debug_place(0, "shan");
    e.debug_deal_damage_with_card(Some(0), 1, 1, DamageNature::Normal, shan);
    assert_eq!(hand_len(&e, 1), h + 1, "非杀伤害不摸");
}

/// 被他人杀伤害：本回合第一次摸牌并记 once，第二次不摸。
#[test]
fn t8_named_sha_once_dual_gate() {
    let dsl = "\
skill hnl_once(990023) {
  trigger damage owner_targeted_by_other_sha_once {
    effect draw_cards(self, 1);
  }
}
";
    let pack = install_dsl(dsl);
    let mut e = Engine::new_with_extensions(961002, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "hnl_once_damage");
    keep_only(&mut e, 1, &["hnl_once_damage"]);
    let h = hand_len(&e, 1);
    let sha = sha_id(&mut e);
    e.debug_deal_damage_with_card(Some(0), 1, 1, DamageNature::Normal, sha);
    assert_eq!(hand_len(&e, 1), h + 1, "第一次杀伤害摸 1");
    assert!(
        e.state_core().vm_aux.once.contains(&(1, "hnl_once_damage".to_string(), 1)),
        "once 已记"
    );
    e.debug_deal_damage_with_card(Some(0), 1, 1, DamageNature::Normal, sha);
    assert_eq!(hand_len(&e, 1), h + 1, "同回合第二次不摸");
}

/// 牌堆顶进技能牌堆；下一事件上「牌堆非空」才摸牌。
#[test]
fn t8_skill_pile_then_nonempty_filter() {
    let dsl = "\
skill hnl_pile(990021) {
  trigger damage owner_is_target {
    effect gain_top_card_to_skill_pile(self);
  }
}
skill hnl_pile_see(990024) {
  trigger damage owner_alive_other_phase_play_skill_pile_nonempty {
    effect draw_cards(self, 1);
  }
}
";
    let pack = install_dsl(dsl);
    let mut e = Engine::new_with_extensions(961003, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "hnl_pile_damage");
    e.debug_grant(1, "hnl_pile_see_damage");
    keep_only(&mut e, 1, &["hnl_pile_damage", "hnl_pile_see_damage"]);
    e.debug_set_turn(1, Phase::Play, 1);
    let deck0 = e.state_core().deck.len();
    let top = *e.state_core().deck.last().expect("牌堆");
    let h = hand_len(&e, 1);
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    let pile = e
        .state_core()
        .vm_aux
        .piles
        .get(&(1, "hnl_pile_damage".to_string()))
        .map(|c| c.len())
        .unwrap_or(0);
    assert_eq!(pile, 1, "牌堆顶进入技能牌堆");
    assert_eq!(e.state_core().deck.len(), deck0 - 1);
    assert_eq!(
        e.state_core().vm_aux.piles.get(&(1, "hnl_pile_damage".to_string())).unwrap()[0],
        top
    );
    assert_eq!(hand_len(&e, 1), h, "同波伤害时尚无牌堆，不摸");
    e.debug_set_turn(1, Phase::Judge, 1);
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(hand_len(&e, 1), h, "非出牌阶段不摸");
    e.debug_set_turn(1, Phase::Play, 1);
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(hand_len(&e, 1), h + 1, "出牌阶段且牌堆非空后摸 1");
}

/// add_mark 写入标记。
#[test]
fn t8_add_mark_writes() {
    let dsl = "\
skill hnl_mark(990022) {
  trigger damage owner_is_target {
    effect add_mark(self, 7, 2);
  }
}
";
    let pack = install_dsl(dsl);
    let mut e = Engine::new_with_extensions(961004, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "hnl_mark_damage");
    keep_only(&mut e, 1, &["hnl_mark_damage"]);
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(
        e.state_core().vm_aux.marks.get(&(1, 7)).copied(),
        Some(2)
    );
}

/// change_max_hp 改变体力上限。
#[test]
fn t8_change_max_hp_writes() {
    let dsl = "\
skill hnl_max(990025) {
  trigger damage owner_is_target {
    effect change_max_hp(self, 1);
  }
}
";
    let pack = install_dsl(dsl);
    let mut e = Engine::new_with_extensions(961005, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "hnl_max_damage");
    keep_only(&mut e, 1, &["hnl_max_damage"]);
    let before = e.state_core().player(1).max_hp;
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(e.state_core().player(1).max_hp, before + 1);
}

/// hand_count 查询为真才摸牌；空手不摸。
#[test]
fn t8_hand_count_query_gates_draw() {
    let dsl = "\
skill hnl_hc(990030) {
  trigger damage owner_is_target {
    if hand_count(self) > 0 {
      effect draw_cards(self, 1);
    }
  }
}
";
    let pack = install_dsl(dsl);
    let mut e = Engine::new_with_extensions(961006, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "hnl_hc_damage");
    keep_only(&mut e, 1, &["hnl_hc_damage"]);
    let h = hand_len(&e, 1);
    assert!(h > 0);
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(hand_len(&e, 1), h + 1, "有手牌时查询放行摸 1");
    let gone: Vec<_> = e.state_core_mut().player_mut(1).hand.drain(..).collect();
    e.state_core_mut().discard.extend(gone);
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(hand_len(&e, 1), 0, "空手查询为假，不摸");
}

/// 本回合丢失史：同一波弃牌尚不可见；下一事件才放行。他人事件不放行。
#[test]
fn t8_lost_history_dual_gate() {
    let dsl = "\
skill hnl_lost(990040) {
  trigger damage owner_alive_lost_hand_this_turn {
    effect draw_cards(self, 1);
  }
}
skill hnl_drop(990041) {
  trigger damage owner_is_target {
    effect discard_hand(self, 1);
  }
}
";
    let pack = install_dsl(dsl);
    let mut e = Engine::new_with_extensions(961007, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "hnl_lost_damage");
    e.debug_grant(1, "hnl_drop_damage");
    keep_only(&mut e, 1, &["hnl_lost_damage", "hnl_drop_damage"]);
    let h = hand_len(&e, 1);
    assert!(h > 0);
    e.debug_deal_damage(Some(0), 2, 1, DamageNature::Normal);
    assert_eq!(hand_len(&e, 1), h, "尚无丢失史，不摸");
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(hand_len(&e, 1), h - 1, "同波只弃牌，丢失史尚未放行");
    assert!(
        e.state_core().vm_aux.lost.get(&1).is_some_and(|c| !c.is_empty()),
        "弃牌写入丢失史"
    );
    e.debug_deal_damage(Some(0), 2, 1, DamageNature::Normal);
    assert_eq!(hand_len(&e, 1), h, "下一事件看见丢失史后摸回 1");
}

/// 移牌批次：真实 CardsMoved 放行；关掉技能后同一路径不再摸。
#[test]
fn t8_move_batch_dual_gate() {
    let dsl = "\
skill hnl_batch(990042) {
  trigger cards_moved owner_alive_discard_batch {
    effect draw_cards(self, 1);
  }
}
";
    let pack = install_dsl(dsl);
    let mut e = Engine::new_with_extensions(961008, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "hnl_batch_cards_moved");
    keep_only(&mut e, 1, &["hnl_batch_cards_moved"]);
    let h = hand_len(&e, 1);
    let card = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 2, card);
    assert_eq!(hand_len(&e, 1), h + 1, "移牌批次摸 1");
    e.debug_set_skill_enabled(1, "hnl_batch_cards_moved", false);
    let card2 = e.debug_place(0, "shan");
    e.debug_give_hand_card(0, 2, card2);
    assert_eq!(hand_len(&e, 1), h + 1, "技能关闭后不再摸");
}

/// 技能实例剩余次数：once_used 之前摸 1，置位后不摸。
#[test]
fn t8_skill_uses_remaining_dual_gate() {
    let dsl = "\
skill hnl_uses(990050) {
  trigger damage owner_is_target {
    let remaining = skill_uses_remaining(self_skill);
    if remaining > 0 {
      effect draw_cards(self, 1);
    }
  }
}
";
    let pack = install_dsl(dsl);
    let mut e = Engine::new_with_extensions(961009, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "hnl_uses_damage");
    keep_only(&mut e, 1, &["hnl_uses_damage"]);
    let h = hand_len(&e, 1);
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(hand_len(&e, 1), h + 1, "剩余次数为正时摸 1");
    let def = e.catalog().skill_by_name("hnl_uses_damage").expect("技能");
    e.state_core_mut()
        .skills
        .get_mut(&(1, def))
        .expect("实例")
        .once_used = true;
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(hand_len(&e, 1), h + 1, "次数用尽后不摸");
}
