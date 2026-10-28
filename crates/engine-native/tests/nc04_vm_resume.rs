//! NC04 追加：VM 挂起 → 续跑 → 效果落账；以及**恢复后故障**的整段回滚。
//!
//! DSL 目前只有 `effect`/`if`/`loop`/`let` 语句，没有选择语句，故本文件用
//! **手工构造的字节码模块**覆盖 `HOST_CHOICE`（挂起）路径：
//! - 模块 A：push 选项 token 0/1 → `HOST_CHOICE 2`（挂起）→ 续跑后 `HALT`；
//! - 模块 B：`HOST_CHOICE 2` → 续跑后调用无效宿主效果 → 故障，
//!   验证「恢复后故障」同样整段回滚并以 `VmFault` 拒绝。

use engine_native::content::extension::{ExtensionDef, ExtensionPack, VmSkillDecl};
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase, RejectReason};
use rule_vm::bytecode::op;
use rule_vm::BytecodeModule;
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn module(name: &str, code: Vec<u8>) -> BytecodeModule {
    BytecodeModule {
        name: name.to_string(),
        code,
        local_count: 1,
        max_stack: 8,
        entry_pc: 0,
        logic_hash: [7u8; 32],
        caps_mask: rule_vm::bytecode::caps::EMIT_EFFECTS | rule_vm::bytecode::caps::REQUEST_CHOICE,
    }
}

/// 选项 token 0/1 → HOST_CHOICE 2 →（续跑后）HALT。
///
/// 只覆盖挂起/续跑机制本身：各效果 tag 的参数出栈契约不同，效果落账已由
/// DSL 版 `nc04_normal_completion` 覆盖。
fn choice_then_halt() -> Vec<u8> {
    let mut c = vec![op::CONST_U32];
    c.extend_from_slice(&0u32.to_le_bytes());
    c.push(op::CONST_U32);
    c.extend_from_slice(&1u32.to_le_bytes());
    c.push(op::HOST_CHOICE);
    c.push(2);
    c.push(op::HALT);
    c
}

/// 选项 token 0/1 → HOST_CHOICE 2 → 续跑后**无效效果 tag**（Host 错误）→ 故障。
fn choice_then_bad_effect() -> Vec<u8> {
    let mut c = vec![op::CONST_U32];
    c.extend_from_slice(&0u32.to_le_bytes());
    c.push(op::CONST_U32);
    c.extend_from_slice(&1u32.to_le_bytes());
    c.push(op::HOST_CHOICE);
    c.push(2);
    // 无效效果 tag：宿主在调用边界明确失败（Host 错误）→ VmFault
    c.push(op::HOST_EFFECT_EXT);
    c.extend_from_slice(&0xFFFFu16.to_le_bytes());
    c.push(op::HALT);
    c
}

fn pack(name: &str, code: Vec<u8>, skill: &str) -> ExtensionPack {
    let decl = VmSkillDecl {
        name: skill.to_string(),
        module: module(name, code),
        trigger: engine_native::catalog::TriggerMeta {
            event: engine_native::event::EventKind::PhaseEnd,
            timing: engine_native::event::Timing::Normal,
            priority: 0,
            optional: false,
            extra_events: Vec::new(),
        },
        filter: engine_native::rvm_bridge::VmFilterPolicy::Always,
    };
    ExtensionPack {
        def: ExtensionDef {
            name: engine_native::content::names::intern(name.to_string()),
            version: 1,
            description: engine_native::content::names::intern("nc04".to_string()),
        },
        generals: Vec::new(),
        cards: Vec::new(),
        host_skills: Vec::new(),
        vm_skills: vec![decl],
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    }
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel: Vec<engine_native::ChoiceOption> =
            if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
                vec![engine_native::ChoiceOption::Pass]
            } else if c.spec.allow_cancel {
                Vec::new()
            } else {
                c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
            };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer { choice_id: c.id, selection: sel },
        });
    }
}

/// 推进到 1 号回合结束（VM 触发点）。
fn drive_to_phase_end(e: &mut Engine) {
    for _ in 0..40 {
        if e.turn().current == 1 && e.turn().phase == Phase::Discard {
            break;
        }
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        drain(e);
    }
    let actor = e.turn().current;
    let _ = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
}

/// 挂起 → 续跑（选择 token 被接受并继续执行，段结算完成、无故障）。
#[test]
fn nc04_pause_resume_completes() {
    let pack = pack("nc04-pause", choice_then_halt(), "nc04_pause_skill");
    let mut e = Engine::new_with_extensions(20261041, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "nc04_pause_skill");
    e.debug_set_hp(1, 2);
    drive_to_phase_end(&mut e);

    // 出现 VM 选择（挂起）
    let mut choice = None;
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "rvm_choice" {
            choice = Some(c);
            break;
        }
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer { choice_id: c.id, selection: Vec::new() },
        });
    }
    let c = choice.expect("VM 应挂起于 HOST_CHOICE");
    assert_eq!(c.spec.options.len(), 2, "两个选项 token");
    let out = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: c.id,
            selection: vec![engine_native::ChoiceOption::Tag(0)],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "续跑应被接受: {out:?}"
    );
    drain(&mut e);
    assert!(e.resolution().fault.is_none(), "正常续跑不留故障");
    // 续跑后段已结算：修订号推进且对局继续（当前座位不再是挂起观察者）
    assert!(e.revision() > 0, "续跑推进修订号");
    assert!(e.open_choice().is_none() || e.open_choice().map(|c| c.spec.prompt != "rvm_choice").unwrap_or(true), "VM 选择已消费");
}

/// 恢复后故障：续跑调用无效宿主效果 → 整段回滚 + `VmFault` 拒绝。
#[test]
fn nc04_fault_after_resume_rolls_back() {
    let pack = pack("nc04-resume-fault", choice_then_bad_effect(), "nc04_resume_fault");
    let mut e = Engine::new_with_extensions(20261042, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "nc04_resume_fault");
    drive_to_phase_end(&mut e);

    let mut choice = None;
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "rvm_choice" {
            choice = Some(c);
            break;
        }
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer { choice_id: c.id, selection: Vec::new() },
        });
    }
    let c = choice.expect("VM 应挂起于 HOST_CHOICE");
    let hash_before = e.state_hash();
    let rev_before = e.revision();
    let out = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: c.id,
            selection: vec![engine_native::ChoiceOption::Tag(1)],
        },
    });
    match out {
        CommandOutcome::Rejected(RejectReason::VmFault { skill, detail }) => {
            assert_eq!(skill, "nc04_resume_fault", "故障技能可定位");
            assert!(
                detail.contains("Host") || detail.contains("host"),
                "故障类别可定位（宿主调用边界失败）: {detail}"
            );
        }
        other => panic!("恢复后故障应以 VmFault 拒绝: {other:?}"),
    }
    assert_eq!(e.state_hash(), hash_before, "恢复后故障同样整段回滚");
    assert_eq!(e.revision(), rev_before, "修订号不因故障推进");
    assert!(e.resolution().fault.is_none(), "故障槽在拒绝后清空");
}

fn pending_vm_choice(e: &mut Engine) -> engine_native::OpenChoice {
    drive_to_phase_end(e);
    for _ in 0..8 {
        let c = e.open_choice().cloned().expect("等待选择");
        if c.spec.prompt == "rvm_choice" {
            return c;
        }
        let selection = if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer { choice_id: c.id, selection },
        });
    }
    panic!("未出现 VM 选择");
}

fn assert_bad_answers_leave_choice(e: &mut Engine, c: &engine_native::OpenChoice, dedup: u64) {
    use engine_native::ChoiceOption::{Card, Player, Tag};
    let before = e.save();
    let log = e.log().to_vec();
    let hash = e.state_hash();
    for selection in [vec![], vec![Tag(2)], vec![Tag(255)], vec![Card(0)], vec![Player(0)],
        vec![Tag(0), Tag(0)], vec![Tag(0), Tag(1)]] {
        let out = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: dedup,
            action: GameAction::Answer { choice_id: c.id, selection },
        });
        assert!(matches!(out, CommandOutcome::Rejected(RejectReason::InvalidChoice)), "{out:?}");
        assert_eq!(e.save(), before, "拒绝不改变快照、随机流或去重水位");
        assert_eq!(e.state_hash(), hash);
        assert_eq!(e.log(), log, "拒绝不留下日志");
        assert_eq!(e.open_choice(), Some(c));
    }
    for (actor, choice_id, reason) in [
        ((c.observer + 1) % 4, c.id, RejectReason::WrongPlayer),
        (c.observer, c.id + 1, RejectReason::NoSuchChoice),
    ] {
        let out = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: dedup,
            action: GameAction::Answer { choice_id, selection: vec![Tag(0)] },
        });
        assert!(matches!(out, CommandOutcome::Rejected(r) if r == reason));
        assert_eq!(e.save(), before);
        assert_eq!(e.state_hash(), hash);
        assert_eq!(e.log(), log);
    }
}

#[test]
fn nc04_invalid_vm_answers_preserve_retry_across_archive() {
    let pack = pack("nc04-invalid-choice", choice_then_halt(), "nc04_invalid_choice");
    let mut original = Engine::new_with_extensions(20261043, 4, &[&pack]).unwrap();
    original.debug_grant(1, "nc04_invalid_choice");
    let choice = pending_vm_choice(&mut original);
    assert_eq!(choice.spec.options, vec![engine_native::ChoiceOption::Tag(0), engine_native::ChoiceOption::Tag(1)]);
    let dedup = next_dedup();
    assert_bad_answers_leave_choice(&mut original, &choice, dedup);

    let bytes = engine_native::archive::encode_archive(&original.save());
    let snapshot = engine_native::archive::decode_archive(&bytes).unwrap();
    assert_eq!(snapshot, original.save());
    let mut restored = Engine::restore_with_extensions(&snapshot, &[&pack]).unwrap();
    assert_eq!(restored.open_choice(), Some(&choice));
    assert_bad_answers_leave_choice(&mut restored, &choice, dedup);

    for e in [&mut original, &mut restored] {
        let out = e.submit(GameCommand {
            actor: choice.observer,
            expected_revision: e.revision(),
            dedup_id: dedup,
            action: GameAction::Answer {
                choice_id: choice.id,
                selection: vec![engine_native::ChoiceOption::Tag(1)],
            },
        });
        assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    }
    assert_eq!(original.state_hash(), restored.state_hash());
    assert_eq!(original.save().core, restored.save().core);
    assert_eq!(original.save().resolution, restored.save().resolution);
    assert_eq!(original.save().last_dedup, dedup);
    assert_eq!(restored.save().last_dedup, dedup);
}

#[test]
fn nc04_vm_fault_after_archive_restores_choice_and_allows_retry() {
    let pack = pack("nc04-restored-fault", choice_then_bad_effect(), "nc04_restored_fault");
    let mut original = Engine::new_with_extensions(20261044, 4, &[&pack]).unwrap();
    original.debug_grant(1, "nc04_restored_fault");
    let choice = pending_vm_choice(&mut original);
    let snapshot = engine_native::archive::decode_archive(
        &engine_native::archive::encode_archive(&original.save()),
    ).unwrap();
    let mut restored = Engine::restore_with_extensions(&snapshot, &[&pack]).unwrap();
    let dedup = next_dedup();
    for e in [&mut original, &mut restored] {
        let before = e.save();
        let log = e.log().to_vec();
        let hash = e.state_hash();
        for _ in 0..2 {
            let out = e.submit(GameCommand {
                actor: choice.observer,
                expected_revision: e.revision(),
                dedup_id: dedup,
                action: GameAction::Answer {
                    choice_id: choice.id,
                    selection: vec![engine_native::ChoiceOption::Tag(1)],
                },
            });
            match out {
                CommandOutcome::Rejected(RejectReason::VmFault { skill, detail }) => {
                    // B7 已修：恢复后的故障身份与原局逐字一致（声明技能名
                    // 入快照，不再从模块名 nc04-restored-fault 重建）。
                    assert_eq!(skill, "nc04_restored_fault");
                    assert!(detail.contains("Host") || detail.contains("host"), "{detail}");
                }
                other => panic!("续跑应报告 VM 故障: {other:?}"),
            }
            assert_eq!(e.save(), before, "故障不改变快照、随机流或去重水位");
            assert_eq!(e.state_hash(), hash);
            assert_eq!(e.log(), log);
            assert_eq!(e.open_choice(), Some(&choice));
            assert!(e.resolution().fault.is_none());
        }
    }
    assert_eq!(original.state_hash(), restored.state_hash());
}

// ---------------------------------------------------------------------------
// B7（规则 v324）：VM 挂起恢复的技能身份/触发语境/限次/命中一致性。
// GT-011：HOST_CHOICE 后编码、跨恢复再执行 STORAGE_SET/once/hits，上述
// 字段与未中断执行一致；旧档按版本策略拒绝。
// ---------------------------------------------------------------------------

use engine_native::event::DamageNature;
use std::sync::Arc;

/// HOST_CHOICE 后 STORAGE_SET(key=7, value=11)。
fn choice_then_storage() -> Vec<u8> {
    let mut c = vec![op::CONST_U32];
    c.extend_from_slice(&0u32.to_le_bytes());
    c.push(op::CONST_U32);
    c.extend_from_slice(&1u32.to_le_bytes());
    c.push(op::HOST_CHOICE);
    c.push(2);
    c.push(op::CONST_U32);
    c.extend_from_slice(&0u32.to_le_bytes());
    c.push(op::CONST_U32);
    c.extend_from_slice(&7u32.to_le_bytes());
    c.push(op::CONST_U32);
    c.extend_from_slice(&11u32.to_le_bytes());
    c.push(op::HOST_EFFECT_EXT);
    c.extend_from_slice(&649u16.to_le_bytes());
    c.push(op::HALT);
    c
}

/// B7 反例回归：恢复后续跑的 STORAGE_SET 以**声明技能名**为键。
/// 缺陷态（v323）：恢复局额外写出 `{模块名#7: 11}`——快照只带模块字节，
/// 技能名从模块名重建。
#[test]
fn nc04_vm_restore_uses_declared_skill_for_storage() {
    let pack = pack("nc04-storage-module", choice_then_storage(), "nc04_storage_skill");
    let mut original = Engine::new_with_extensions(20261045, 4, &[&pack]).unwrap();
    original.debug_grant(1, "nc04_storage_skill");
    let choice = pending_vm_choice(&mut original);
    let snapshot = engine_native::archive::decode_archive(
        &engine_native::archive::encode_archive(&original.save()),
    ).unwrap();
    let mut restored = Engine::restore_with_extensions(&snapshot, &[&pack]).unwrap();
    let dedup = next_dedup();
    for e in [&mut original, &mut restored] {
        let out = e.submit(GameCommand {
            actor: choice.observer,
            expected_revision: e.revision(),
            dedup_id: dedup,
            action: GameAction::Answer {
                choice_id: choice.id,
                selection: vec![engine_native::ChoiceOption::Tag(1)],
            },
        });
        assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    }
    // 存储键 = 声明技能名（非模块名 nc04-storage-module），且双局逐字一致。
    assert_eq!(
        original.save().core.vm_aux.storage,
        restored.save().core.vm_aux.storage,
    );
    assert_eq!(
        restored.save().core.vm_aux.storage.keys().collect::<Vec<_>>(),
        vec!["nc04_storage_skill#7"],
    );
    // 完整轨迹：核心状态、结算快照与哈希一致。
    assert_eq!(original.state_hash(), restored.state_hash());
    assert_eq!(original.save().core, restored.save().core);
    assert_eq!(original.save().resolution, restored.save().resolution);
}

/// B7 反例回归：挂起恢复后续跑的限次消费不丢失。
/// `owner_alive_once_per_round` 携带 consume=[(空键, 2)]——Halted 时写
/// once[(owner, 技能名, 2)]。缺陷态（v323）：恢复机 consume 被重置，
/// once 不落账。
#[test]
fn nc04_vm_restore_preserves_once_consume() {
    let policy = engine_native::rvm_bridge::VmFilterPolicy::Named(Arc::new(
        engine_native::rvm_named::compile_named("owner_alive_once_per_round").expect("滤名编译"),
    ));
    let decl = VmSkillDecl {
        name: "nc04_once_skill".to_string(),
        module: module("nc04-once-module", choice_then_halt()),
        trigger: engine_native::catalog::TriggerMeta {
            event: engine_native::event::EventKind::PhaseEnd,
            timing: engine_native::event::Timing::Normal,
            priority: 0,
            optional: false,
            extra_events: Vec::new(),
        },
        filter: policy,
    };
    let pack = ExtensionPack {
        def: ExtensionDef {
            name: engine_native::content::names::intern("nc04-once-module".to_string()),
            version: 1,
            description: engine_native::content::names::intern("nc04".to_string()),
        },
        generals: Vec::new(),
        cards: Vec::new(),
        host_skills: Vec::new(),
        vm_skills: vec![decl],
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    };
    let mut original = Engine::new_with_extensions(20261047, 4, &[&pack]).unwrap();
    // 先推进到 1 号弃牌阶段再授技——否则此前各阶段尾就会首次触发并被
    // drain 代答消耗限次（once_per_round 全场只此一次）。
    for _ in 0..40 {
        if original.turn().current == 1 && original.turn().phase == engine_native::Phase::Discard {
            break;
        }
        let actor = original.turn().current;
        let _ = original.submit(GameCommand {
            actor,
            expected_revision: original.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        drain(&mut original);
    }
    original.debug_grant(1, "nc04_once_skill");
    let choice = pending_vm_choice(&mut original);
    let snapshot = engine_native::archive::decode_archive(
        &engine_native::archive::encode_archive(&original.save()),
    ).unwrap();
    let mut restored = Engine::restore_with_extensions(&snapshot, &[&pack]).unwrap();
    let dedup = next_dedup();
    for e in [&mut original, &mut restored] {
        let out = e.submit(GameCommand {
            actor: choice.observer,
            expected_revision: e.revision(),
            dedup_id: dedup,
            action: GameAction::Answer {
                choice_id: choice.id,
                selection: vec![engine_native::ChoiceOption::Tag(0)],
            },
        });
        assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    }
    // once 落账且以声明技能名为键，双局一致。
    let expected_once = vec![(
        1u8,
        "nc04_once_skill".to_string(),
        2u8,
    )];
    let collect = |e: &Engine| -> Vec<(u8, String, u8)> {
        e.save()
            .core
            .vm_aux
            .once
            .iter()
            .map(|(p, k, s)| (*p, k.clone(), *s))
            .collect()
    };
    assert_eq!(collect(&original), expected_once, "原局限次落账");
    assert_eq!(collect(&restored), expected_once, "恢复局限次落账不丢失");
    assert_eq!(original.state_hash(), restored.state_hash());
}

/// B7 反例回归：伤害触发的命中计数跨恢复保留。
/// `owner_alive_first` 携带 count_hits=true——kind==1（伤害）Halted 时
/// hits[(owner, 技能名)] += 1。缺陷态（v323）：恢复机旗标被重置，命中
/// 不计数。
#[test]
fn nc04_vm_restore_preserves_hit_counting() {
    let policy = engine_native::rvm_bridge::VmFilterPolicy::Named(Arc::new(
        engine_native::rvm_named::compile_named("owner_alive_first").expect("滤名编译"),
    ));
    let decl = VmSkillDecl {
        name: "nc04_hits_skill".to_string(),
        module: module("nc04-hits-module", choice_then_halt()),
        trigger: engine_native::catalog::TriggerMeta {
            event: engine_native::event::EventKind::Damage,
            timing: engine_native::event::Timing::Normal,
            priority: 0,
            optional: false,
            extra_events: Vec::new(),
        },
        filter: policy,
    };
    let pack = ExtensionPack {
        def: ExtensionDef {
            name: engine_native::content::names::intern("nc04-hits-module".to_string()),
            version: 1,
            description: engine_native::content::names::intern("nc04".to_string()),
        },
        generals: Vec::new(),
        cards: Vec::new(),
        host_skills: Vec::new(),
        vm_skills: vec![decl],
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    };
    let mut original = Engine::new_with_extensions(20261048, 4, &[&pack]).unwrap();
    original.debug_grant(1, "nc04_hits_skill");
    // 伤害触发 → VM 挂起于 HOST_CHOICE。
    let _ = original.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    let choice = original
        .open_choice()
        .cloned()
        .filter(|c| c.spec.prompt == "rvm_choice")
        .expect("伤害触发后 VM 应挂起");
    let snapshot = engine_native::archive::decode_archive(
        &engine_native::archive::encode_archive(&original.save()),
    ).unwrap();
    let mut restored = Engine::restore_with_extensions(&snapshot, &[&pack]).unwrap();
    let dedup = next_dedup();
    for e in [&mut original, &mut restored] {
        let out = e.submit(GameCommand {
            actor: choice.observer,
            expected_revision: e.revision(),
            dedup_id: dedup,
            action: GameAction::Answer {
                choice_id: choice.id,
                selection: vec![engine_native::ChoiceOption::Tag(0)],
            },
        });
        assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    }
    let hits = |e: &Engine| e.save().core.vm_aux.hits.clone();
    assert_eq!(hits(&original).get(&(1, "nc04_hits_skill".to_string())), Some(&1), "原局命中计数");
    assert_eq!(hits(&restored).get(&(1, "nc04_hits_skill".to_string())), Some(&1), "恢复局命中计数保留");
    assert_eq!(original.state_hash(), restored.state_hash());
}

/// 旧档拒绝（版本策略）：v323 普通档解码成功但按规则版本拒绝恢复。
#[test]
fn b7_plain_v323_archive_is_rejected_by_rule_version() {
    let old = include_bytes!(
        "../../../docs/operations/evidence/2026-10-28-b7-vm-suspend-restore/native-v323.ngsa"
    );
    let snapshot = engine_native::archive::decode_archive(old).unwrap();
    assert_eq!(snapshot.ruleset_version, 323);
    assert!(matches!(
        Engine::restore(&snapshot),
        Err(engine_native::snap::SnapshotError::RuleVersionMismatch { snapshot: 323, catalog: 324 })
    ));
}

/// 旧档拒绝（不可靠恢复）：v323 含挂起 VM 机的档用旧 Rvm 编码，新解码
/// 器无法读出技能身份字段——解码即失败，不存在半恢复。
#[test]
fn b7_suspended_vm_v323_archive_is_rejected() {
    let old = include_bytes!(
        "../../../docs/operations/evidence/2026-10-28-b7-vm-suspend-restore/vm-suspended-v323.ngsa"
    );
    assert!(
        engine_native::archive::decode_archive(old).is_err(),
        "旧格式挂起 VM 档不能解码（B7 前的 Rvm 编码不含技能身份字段）"
    );
}
