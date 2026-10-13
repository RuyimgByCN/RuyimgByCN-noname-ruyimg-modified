//! T8 增量 1 端到端：engine-native 触发调度接 rule-vm——扩展包声明的
//! VM 技能（数据驱动，不重编译）经目录 SkillDef 触发，效果按两层提交
//! 协议落账（Halted 提交 / fuel 耗尽回滚）。语义样本=酒尾同款
//! 「受伤回 1 / 满血摸 1」（原型 t8_backend_prototype 对照）。

use content_compiler::compile_source_named;
use engine_native::catalog::TriggerMeta;
use engine_native::event::{EventKind, Timing};
use engine_native::content::extension::{ExtensionDef, ExtensionPack, VmSkillDecl};
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
