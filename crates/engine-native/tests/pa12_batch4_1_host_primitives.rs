//! PA12 迁移批④-1（宿主原语：`turn_over`，2026-10-27）：DSL `turn_over(player)`
//!（tag `TURN_OVER` 433 → `EffectRequest::TurnOver`，旧线「武将牌翻面：背面跳过
//! 下个回合后翻回」）此前不在原生 `accept_effect` 允许面（装载整包拒装）。本批
//! 接入：允许面 + `commit_effect` 以 `TurnOverBefore` 子事件帧走内核翻面提交
//!（神裔「不能翻至背面」守卫、急冻冰封解除、`TurnOvered` 事件全部同口径）。
//! 真实技能全链：雪堕（伤害后翻面）/ 扶理所涉 `turn_over` 分支由模块差分覆盖。

use content_compiler::compile_source_named;
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::event::{DamageNature, Phase};
use engine_native::{ChoiceOption, Engine, GameAction, GameCommand};

fn settle(e: &mut Engine, confirm: bool) {
    for _ in 0..10 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel = if confirm && c.spec.prompt == "trigger_confirm" {
            vec![c.spec.options.iter().find(|o| matches!(o, ChoiceOption::Confirm)).copied().unwrap_or(c.spec.options[0])]
        } else {
            vec![c.spec.options[0].clone()]
        };
        submit(e, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
    }
}

const NSXUEZHU: &str = include_str!("../../../content/original/ported/diy/nsxuezhu.skill");

fn pack_for(source: &str, id: &str) -> ExtensionPack {
    let pkg = compile_source_named(id, source).expect("编译 ported .skill");
    assert!(pkg.manifest.rule_vm_abi_version <= rule_vm::RULE_VM_ABI_VERSION);
    let out = content_native_loader::bridge_vm_skill_decls(&pkg).expect("正式桥接");
    ExtensionPack {
        def: ExtensionDef {
            name: "pa12-batch4-1",
            version: 1,
            description: "PA12 批④-1 turn_over",
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

/// 雪堕（diy，`damage owner_is_source_or_target` → 摸 1 + 翻面）：持有者受到
/// 伤害后翻至背面；再受一次伤害翻回正面（内核翻面提交全口径——含 TurnOvered
/// 帧与背面跳回合由回合推进验证）。
#[test]
fn pa12_batch4_1_turn_over_toggles_face() {
    let pack = pack_for(NSXUEZHU, "diy-nsxuezhu");
    let name = pack.vm_skills[0].name.clone();
    let mut e = Engine::new_with_extensions(995_001, 4, &[&pack]).expect("建局");
    e.debug_grant(1, &name);
    assert!(!e.state_core().player(1).face_down, "初始正面");
    e.debug_deal_damage(Some(0u8.into()), 1u8.into(), 1, DamageNature::Normal);
    settle(&mut e, true);
    assert!(e.state_core().player(1).face_down, "受伤后翻至背面");
    e.debug_deal_damage(Some(0u8.into()), 1u8.into(), 1, DamageNature::Normal);
    settle(&mut e, true);
    assert!(!e.state_core().player(1).face_down, "再受伤翻回正面");

    // 背面跳回合（E03 内核路径）：翻至背面后，1 号下个回合的判定阶段开始
    // 即被跳过路径翻回（commit_phase：PhaseBegin(Judge) ∧ face_down →
    // TurnOverBefore 翻回 + 直达回合收束）。驱动注记：单元驱动在跳过窗口
    // 内注入 NextPhase 会与待提交的 TurnEnded 子帧竞争（实测该窗口见到
    // phase=Judge 且 face 已翻回），故以「首次到达 1 号回合时已翻回」为
    // 跳过路径执行的判据；整回合无阶段帧的全量验证属内核 E03 既有测试面。
    e.debug_deal_damage(Some(0u8.into()), 1u8.into(), 1, DamageNature::Normal);
    e.debug_set_hp(1, 10);
    settle(&mut e, true);
    assert!(e.state_core().player(1).face_down);
    let mut flipped_back_at_first_turn = None;
    for _ in 0..200 {
        if e.outcome().is_some() {
            break;
        }
        if e.turn().current == 1 {
            flipped_back_at_first_turn = Some(e.state_core().player(1).face_down);
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let sel: Vec<ChoiceOption> = c
                .spec
                .options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Card(_)))
                .copied()
                .take(c.spec.min.max(1))
                .collect();
            let sel = if sel.is_empty() { vec![c.spec.options[0].clone()] } else { sel };
            submit(&mut e, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
            continue;
        }
        let cur = e.turn().current;
        submit(&mut e, cur, GameAction::NextPhase);
    }
    assert_eq!(
        flipped_back_at_first_turn,
        Some(false),
        "1 号回合到达时已由跳过路径翻回正面"
    );
}
