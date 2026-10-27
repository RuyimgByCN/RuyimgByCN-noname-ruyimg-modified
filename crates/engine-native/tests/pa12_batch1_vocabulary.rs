//! PA12 迁移批①（桥接词汇对齐，2026-10-27）：装载器/桥接新增两个高频
//! 旧线滤名——`owner_is_source`（别名，inventory 206 处）与 `owner_alive`
//! （持有者存活、不含事件主玩家合取，754 处）——并把「不承载滤名」从
//! 静默跳过改为装载期拒装（`BridgeError::UnsupportedVmFilters`）。
//! 以两个**真实 ported .skill** 全链验收（正式装载路径 → 原生整局）：
//! 清励（clan，他人结束阶段摸至体力上限）与 悟道（collab，自己用牌后
//! 摸 1，optional 确认门）。计划：docs/architecture/native-dsl-migration-plan.md。

use content_compiler::compile_source_named;
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::event::Phase;
use engine_native::{ChoiceOption, Engine, GameAction, GameCommand};

const CLANQINGLI: &str = include_str!("../../../content/original/ported/clan/clanqingli.skill");
const DCWUDAO: &str = include_str!("../../../content/original/ported/collab/dcwudao.skill");

fn pack_for(source: &str, id: &str) -> ExtensionPack {
    let pkg = compile_source_named(id, source).expect("编译 ported .skill");
    assert!(
        pkg.manifest.rule_vm_abi_version <= rule_vm::RULE_VM_ABI_VERSION,
        "批①不引入新 ABI"
    );
    let out = content_native_loader::bridge_vm_skill_decls(&pkg).expect("正式桥接");
    assert_eq!(out.decls.len(), 1, "单技能包产出 1 条声明");
    ExtensionPack {
        def: ExtensionDef {
            name: "pa12-batch1",
            version: 1,
            description: "PA12 批① 桥接词汇对齐",
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

fn advance_to(e: &mut Engine, player: u8, phase: Phase) {
    for _ in 0..300 {
        if e.turn().current == player && e.turn().phase == phase {
            return;
        }
        assert!(e.outcome().is_none(), "对局提前结束");
        if let Some(c) = e.open_choice() {
            let c = c.clone();
            settle_plain(e, &c);
            continue;
        }
        let cur = e.turn().current;
        submit(e, cur, GameAction::NextPhase);
    }
    panic!("300 步未到 p{player} 目标阶段");
}

fn hand_len(e: &Engine, p: u8) -> usize {
    e.state_core().player(p).hand.len()
}

/// 映射面：`owner_is_source` 别名与 `owner_alive` 精确策略；未知名装载期拒装。
#[test]
fn pa12_batch1_vocabulary_mapping_and_strict_reject() {
    use engine_native::rvm_bridge::{map_vm_filter, VmFilterPolicy as P};
    assert_eq!(map_vm_filter("owner_is_source"), Some(P::OwnerIsSource));
    assert_eq!(map_vm_filter("owner_source"), Some(P::OwnerIsSource));
    assert_eq!(map_vm_filter("owner_alive"), Some(P::OwnerAlive));
    // 拒装：装载器对不承载滤名报错并列出（不再静默跳过）。
    let mut pkg = compile_source_named("pa12-batch1-reject", DCWUDAO).expect("编译");
    pkg.trigger_index[0].filter = "pa12_not_a_native_filter".to_string();
    match content_native_loader::bridge_vm_skill_decls(&pkg) {
        Err(content_native_loader::BridgeError::UnsupportedVmFilters { filters }) => {
            assert_eq!(filters, vec!["pa12_not_a_native_filter".to_string()]);
        }
        other => panic!("应拒装，实际 {other:?}"),
    }
}

/// 清励（真实 .skill，`phase_end owner_alive`）：**他人**（座位 0）结束阶段，
/// 持有者（座位 1）存活即触发，摸至体力上限——验证 `owner_alive` 不含事件
/// 主玩家合取（`owner_is_target` 语义下不会在他人回合触发）。
#[test]
fn pa12_batch1_owner_alive_triggers_on_others_end_phase() {
    let pack = pack_for(CLANQINGLI, "clan-qingli");
    let name = pack.vm_skills[0].name.clone();
    let mut e = Engine::new_with_extensions(960_001, 4, &[&pack]).expect("建局");
    e.debug_grant(1, &name);
    e.debug_set_max_hp(1, 4);
    e.debug_clear_hand(1);
    assert_eq!(hand_len(&e, 1), 0);
    // 座位 0 的结束阶段（不是持有者的回合）：phase_end@End + owner_alive 过闸
    advance_to(&mut e, 0, Phase::End);
    // 进入 0 号结束阶段的那一步即触发（PhaseBegin@End + owner_alive）。
    // 注意落账值为 5 而非「摸至体力上限 4」：VM 宿主两线（原生 pending_effects /
    // 旧线 host.effects）都在模块 Halt 后才落账，模块内 `hand_count(self)` 再查询
    // 看到的仍是执行前手牌 0，`loop 5 { if hand<max … }` 恒摸 5。这是 ported DSL
    // 的保真度问题（两线一致，见同名 legacy 差分），不是原生桥接缺口；已登记
    // 迁移计划 §2 H 类。
    assert_eq!(hand_len(&e, 1), 5, "他人结束阶段触发；两线同为 5 张（再查询陈旧）");
    assert_eq!(e.turn().current, 0, "触发时仍是 0 号回合——owner_alive 不要求事件主玩家=持有者");
}

/// 悟道（真实 .skill，`card_used owner_is_source optional`）：持有者自己用牌
/// → 确认门（optional）→ 摸 1；他人用牌不触发。
#[test]
fn pa12_batch1_owner_is_source_after_own_card_used() {
    let pack = pack_for(DCWUDAO, "collab-wudao");
    let name = pack.vm_skills[0].name.clone();
    let mut e = Engine::new_with_extensions(960_002, 4, &[&pack]).expect("建局");
    e.debug_grant(1, &name);
    // 他人（0 号）先用一张杀：不触发（来源≠持有者）
    advance_to(&mut e, 0, Phase::Play);
    let sha0 = e.debug_place(0, "sha");
    let h1 = hand_len(&e, 1);
    submit(&mut e, 0, GameAction::PlayCard { card: sha0, targets: vec![2] });
    for _ in 0..8 {
        if let Some(c) = e.open_choice() {
            let c = c.clone();
            settle_plain(&mut e, &c);
        } else {
            break;
        }
    }
    assert_eq!(hand_len(&e, 1), h1, "他人用牌不过闸（owner_is_source）");
    // 持有者自己用牌 → optional 确认门 → 确认后摸 1
    advance_to(&mut e, 1, Phase::Play);
    let sha1 = e.debug_place(1, "sha");
    let base = hand_len(&e, 1);
    submit(&mut e, 1, GameAction::PlayCard { card: sha1, targets: vec![0] });
    let mut confirmed = false;
    for _ in 0..8 {
        let Some(c) = e.open_choice() else { break };
        let c = c.clone();
        if c.spec.prompt == "trigger_confirm" && !confirmed {
            let sel = c
                .spec
                .options
                .iter()
                .find(|o| matches!(o, ChoiceOption::Confirm))
                .copied()
                .unwrap_or(c.spec.options[0]);
            submit(&mut e, c.observer, GameAction::Answer { choice_id: c.id, selection: vec![sel] });
            confirmed = true;
            continue;
        }
        settle_plain(&mut e, &c);
    }
    assert!(confirmed, "optional 技能应先开确认门");
    // 手牌 = base -1（打出杀）+1（悟道摸 1）；杀结算目标已闪/受伤与本断言无关
    assert_eq!(hand_len(&e, 1), base, "用牌后摸 1 补回打出的杀");
}
