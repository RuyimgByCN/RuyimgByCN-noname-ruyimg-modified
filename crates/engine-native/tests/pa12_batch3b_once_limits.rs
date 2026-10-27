//! PA12 迁移批③b（限次系语义验收，2026-10-27）：旧线 `*_once` / `*_once_per_round`
//! 滤名在原生由具名程序（`rvm_named`：`once`→Seed::Once(1)，`once_per_round`→Once(2)）
//! 承载，消费记录在 `vm_aux.once`（座位×键×范围），Halt 时写入。窗口对齐依据：
//! 旧线 `turn_fire_count` 在**每个回合开始对全部技能实例清零**（engine-core
//! command/turn.rs），原生 `commit_turn_ended → vm_aux.clear_turn()` 对全部座位清
//! scope 1——两线都是「任一回合内一次」；轮范围对应 `clear_round`。
//! 真实技能全链：良范（offline，`damage owner_is_target_once optional` 摸 1）、
//! 反转（collab，`phase_begin owner_alive_once_per_round optional`）。

use content_compiler::compile_source_named;
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::event::{DamageNature, Phase};
use engine_native::{ChoiceOption, Engine, GameAction, GameCommand};

const LIANGFAN2: &str = include_str!("../../../content/original/ported/offline/liangfan2.skill");
const DCFANZHUAN: &str = include_str!("../../../content/original/ported/collab/dcfanzhuan.skill");

fn pack_for(source: &str, id: &str) -> ExtensionPack {
    let pkg = compile_source_named(id, source).expect("编译 ported .skill");
    let out = content_native_loader::bridge_vm_skill_decls(&pkg).expect("正式桥接（once 系经具名程序）");
    assert_eq!(out.decls.len(), 1);
    ExtensionPack {
        def: ExtensionDef {
            name: "pa12-batch3b",
            version: 1,
            description: "PA12 批③b 限次系",
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

/// 结算当前所有询问；遇 `trigger_confirm` 按 `confirm` 作答并计数。返回确认门出现次数。
fn drain(e: &mut Engine, confirm: bool) -> usize {
    let mut gates = 0;
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "trigger_confirm" {
            gates += 1;
            let sel = if confirm {
                c.spec.options.iter().find(|o| matches!(o, ChoiceOption::Confirm)).copied().unwrap_or(c.spec.options[0])
            } else {
                c.spec.options[0]
            };
            submit(e, c.observer, GameAction::Answer { choice_id: c.id, selection: vec![sel] });
            continue;
        }
        settle_plain(e, &c);
    }
    gates
}

fn hand_len(e: &Engine, p: u8) -> usize {
    e.state_core().player(p).hand.len()
}

fn next_turn(e: &mut Engine) {
    let cur = e.turn().current;
    for _ in 0..40 {
        if e.turn().current != cur {
            return;
        }
        drain(e, false);
        let c = e.turn().current;
        submit(e, c, GameAction::NextPhase);
    }
    panic!("40 步未换回合");
}

/// 良范：`damage owner_is_target_once optional`——同一回合内第二次受伤不再问（once 已消费），
/// 换回合后再次可触发（任一回合窗口，与旧线 turn_fire_count 全局清零一致）。
#[test]
fn pa12_batch3b_once_per_turn_window() {
    let pack = pack_for(LIANGFAN2, "offline-liangfan2");
    let name = pack.vm_skills[0].name.clone();
    let mut e = Engine::new_with_extensions(990_001, 4, &[&pack]).expect("建局");
    e.debug_grant(1, &name);
    e.debug_set_hp(1, 4);
    // 0 号回合内（非持有者回合）：第一次受伤 → 门 + 摸 1
    let h0 = hand_len(&e, 1);
    e.debug_deal_damage(Some(0u8.into()), 1u8.into(), 1, DamageNature::Normal);
    assert_eq!(drain(&mut e, true), 1, "首次受伤开确认门");
    assert_eq!(hand_len(&e, 1), h0 + 1, "确认后摸 1");
    // 同回合第二次受伤 → 无门、不摸（once 已消费）
    let h1 = hand_len(&e, 1);
    e.debug_deal_damage(Some(0u8.into()), 1u8.into(), 1, DamageNature::Normal);
    assert_eq!(drain(&mut e, true), 0, "同回合第二次不再问");
    assert_eq!(hand_len(&e, 1), h1, "同回合第二次不摸");
    assert!(
        e.state_core().vm_aux.once.iter().any(|(seat, _, scope)| *seat == 1 && *scope == 1),
        "once 消费记录在 vm_aux.once（scope 1）"
    );
    // 换到下一回合（任一玩家）：scope 1 清空 → 再次受伤可触发
    next_turn(&mut e);
    assert!(
        !e.state_core().vm_aux.once.iter().any(|(_, _, scope)| *scope == 1),
        "回合结束清空 scope 1"
    );
    e.debug_set_hp(1, 4);
    let h2 = hand_len(&e, 1);
    e.debug_deal_damage(Some(0u8.into()), 1u8.into(), 1, DamageNature::Normal);
    assert_eq!(drain(&mut e, true), 1, "新回合再次开门");
    assert_eq!(hand_len(&e, 1), h2 + 1);
}

/// 反转：`phase_begin owner_alive_once_per_round optional`——一轮内只在第一个回合开始
/// 问一次；换轮后再问。
#[test]
fn pa12_batch3b_once_per_round_window() {
    let pack = pack_for(DCFANZHUAN, "collab-dcfanzhuan");
    let name = pack.vm_skills[0].name.clone();
    let mut e = Engine::new_with_extensions(990_002, 4, &[&pack]).expect("建局");
    e.debug_grant(1, &name);
    e.debug_set_max_hp(1, 5);
    e.debug_set_hp(1, 2); // delta = 5-2-2 = 1 → 回 1 摸 1
    let round0 = e.state_core().turn.round;
    // 首回合（座位 0，construction 不发 Judge begin 帧——登记差异）；推进到座位 1 回合开始
    let mut gates_this_round = 0;
    let mut gated_at: Vec<(u8, Phase, u32)> = Vec::new();
    for _ in 0..200 {
        if e.state_core().turn.round != round0 {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt == "trigger_confirm" {
                gates_this_round += 1;
                gated_at.push((e.turn().current, e.turn().phase, e.state_core().turn.round));
                submit(&mut e, c.observer, GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![c.spec.options.iter().find(|o| matches!(o, ChoiceOption::Confirm)).copied().unwrap()],
                });
                continue;
            }
            settle_plain(&mut e, &c);
            continue;
        }
        let cur = e.turn().current;
        submit(&mut e, cur, GameAction::NextPhase);
    }
    assert_eq!(gates_this_round, 1, "一轮内只问一次（{gated_at:?}）");
    assert_eq!(gated_at[0].1, Phase::Judge, "回合开始帧");
    assert_eq!(e.debug_hp(1), 3, "回 1");
    // 换轮：scope 2 清空 → 再问一次
    assert!(e.state_core().turn.round > round0, "已进入下一轮");
    let round1 = e.state_core().turn.round;
    let mut gates_next = 0;
    for _ in 0..200 {
        if e.state_core().turn.round != round1 {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt == "trigger_confirm" {
                gates_next += 1;
                submit(&mut e, c.observer, GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![c.spec.options[0]],
                });
                continue;
            }
            settle_plain(&mut e, &c);
            continue;
        }
        let cur = e.turn().current;
        submit(&mut e, cur, GameAction::NextPhase);
    }
    assert_eq!(gates_next, 1, "下一轮再问一次");
}

