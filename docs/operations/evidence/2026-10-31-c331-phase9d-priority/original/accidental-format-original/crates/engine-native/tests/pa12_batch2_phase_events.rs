//! PA12 迁移批②（阶段/事件帧补齐，2026-10-27）：
//! - DSL `phase_begin`（旧线 = `TurnBegan` 回合开始）→ 原生 `PhaseBegin` + `AtPhase(Judge)`
//!   （原生以首个阶段 Judge 的 begin 帧表示回合开始；此前映到任一阶段开始帧，每回合 5 次）；
//! - DSL `sha_dodged` → 原生 `EventKind::ShaDodged`（新增 `trigger_ctx_of` 臂：来源=杀的使用者）。
//! 以真实 ported .skill 全链验收：五脉（key，`phase_begin owner_is_target optional` 摸 1）、
//! 蹙国（newjiang，`sha_dodged owner_is_source` 摸 1）。
//! 登记差异：construction 置 `turn_opened=true`，第一个回合的 Judge begin 帧不发出——
//! 首回合（座位 0 的第一回合）不触发回合开始类技能（见迁移计划 §2）。

use content_compiler::compile_source_named;
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::event::Phase;
use engine_native::{ChoiceOption, Engine, GameAction, GameCommand};

const WUMAI: &str = include_str!("../../../content/original/ported/key/liyingxia_wumai.skill");
const CUGUO: &str = include_str!("../../../content/original/ported/newjiang/cuguo.skill");

fn pack_for(source: &str, id: &str) -> ExtensionPack {
    let pkg = compile_source_named(id, source).expect("编译 ported .skill");
    assert!(pkg.manifest.rule_vm_abi_version <= rule_vm::RULE_VM_ABI_VERSION);
    let out = content_native_loader::bridge_vm_skill_decls(&pkg).expect("正式桥接");
    assert_eq!(out.decls.len(), 1);
    ExtensionPack {
        def: ExtensionDef {
            name: "pa12-batch2",
            version: 1,
            description: "PA12 批② 阶段/事件帧",
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
    let sel = if sel.is_empty() {
        vec![c.spec.options[0].clone()]
    } else {
        sel
    };
    submit(
        e,
        c.observer,
        GameAction::Answer {
            choice_id: c.id,
            selection: sel,
        },
    );
}

fn hand_len(e: &Engine, p: u8) -> usize {
    e.state_core().player(p).hand.len()
}

/// 推进一步：有普通询问就结算（`trigger_confirm` 交给回调决定），否则 NextPhase。
fn step(e: &mut Engine, on_confirm: &mut dyn FnMut(&Engine, &engine_native::OpenChoice) -> bool) {
    if let Some(c) = e.open_choice() {
        let c = c.clone();
        if c.spec.prompt == "trigger_confirm" {
            let yes = on_confirm(e, &c);
            let sel = if yes {
                c.spec
                    .options
                    .iter()
                    .find(|o| matches!(o, ChoiceOption::Confirm))
                    .copied()
                    .unwrap_or(c.spec.options[0])
            } else {
                c.spec.options[0]
            };
            submit(
                e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![sel],
                },
            );
            return;
        }
        settle_plain(e, &c);
        return;
    }
    let cur = e.turn().current;
    submit(e, cur, GameAction::NextPhase);
}

/// 五脉：`phase_begin owner_is_target optional`——只在持有者（座位 1）自己回合开始
/// （Judge begin 帧）开确认门，确认后摸 1；他人回合开始不问；每回合恰好一次。
#[test]
fn pa12_batch2_phase_begin_is_turn_start_of_owner_only() {
    let pack = pack_for(WUMAI, "key-wumai");
    let name = pack.vm_skills[0].name.clone();
    let mut e = Engine::new_with_extensions(970_001, 4, &[&pack]).expect("建局");
    e.debug_grant(1, &name);
    let mut confirms: Vec<(u8, Phase)> = Vec::new();
    let mut hand_before_confirm = 0usize;
    let mut hand_after_first = None;
    // 推进两整圈（4 座 × 5 阶段 ≈ 40 步/圈）
    for _ in 0..120 {
        if e.outcome().is_some() {
            break;
        }
        let before = hand_len(&e, 1);
        step(&mut e, &mut |eng, _c| {
            confirms.push((eng.turn().current, eng.turn().phase));
            hand_before_confirm = hand_len(eng, 1);
            true
        });
        if hand_after_first.is_none()
            && !confirms.is_empty()
            && hand_len(&e, 1) == hand_before_confirm + 1
        {
            hand_after_first = Some(hand_len(&e, 1));
        }
        let _ = before;
    }
    assert!(!confirms.is_empty(), "持有者回合开始应开确认门");
    for (cur, ph) in &confirms {
        assert_eq!(*cur, 1, "只在持有者自己回合");
        assert_eq!(*ph, Phase::Judge, "回合开始=首个阶段 Judge 的 begin 帧");
    }
    assert!(
        confirms.len() >= 2,
        "两圈内至少两次自己回合开始（实际 {}）",
        confirms.len()
    );
    assert!(hand_after_first.is_some(), "确认后摸 1 落账");
}

/// 蹙国：`sha_dodged owner_is_source`——持有者（座位 1）的杀被目标（座位 2，持闪）闪避
/// → 摸 1；他人的杀被闪避不触发。
#[test]
fn pa12_batch2_sha_dodged_owner_is_source() {
    let pack = pack_for(CUGUO, "newjiang-cuguo");
    let name = pack.vm_skills[0].name.clone();
    let mut e = Engine::new_with_extensions(970_002, 4, &[&pack]).expect("建局");
    e.debug_grant(1, &name);

    // 负例：0 号出杀打 1 号，1 号持闪闪避——事件来源是 0 号，蹙国（1 号）不触发
    advance_to(&mut e, 0, Phase::Play);
    let shan1 = e.debug_place(1, "shan");
    let sha0 = e.debug_place(0, "sha");
    let h1 = hand_len(&e, 1);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha0,
            targets: vec![1],
        },
    );
    answer_with_card(&mut e, 1, shan1);
    drain_plain(&mut e);
    assert_eq!(
        hand_len(&e, 1),
        h1 - 1,
        "1 号只打出了闪，无摸牌（来源≠持有者）"
    );

    // 正例：1 号出杀打 2 号，2 号持闪闪避 → ShaDodged{source=1} → 蹙国摸 1
    advance_to(&mut e, 1, Phase::Play);
    let shan2 = e.debug_place(2, "shan");
    let sha1 = e.debug_place(1, "sha");
    let base = hand_len(&e, 1);
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![2],
        },
    );
    answer_with_card(&mut e, 2, shan2);
    drain_plain(&mut e);
    assert_eq!(hand_len(&e, 1), base, "打出杀 -1、蹙国摸 1 → 持平");
}

fn advance_to(e: &mut Engine, player: u8, phase: Phase) {
    for _ in 0..300 {
        if e.turn().current == player && e.turn().phase == phase {
            return;
        }
        assert!(e.outcome().is_none(), "对局提前结束");
        step(e, &mut |_, _| false);
    }
    panic!("300 步未到 p{player} 目标阶段");
}

/// 让 `who` 用指定牌应答当前询问（闪窗口）；若当前询问不是他的，先结算其它询问。
fn answer_with_card(e: &mut Engine, who: u8, card: engine_native::CardId) {
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else {
            return;
        };
        if c.observer == who && c.spec.options.contains(&ChoiceOption::Card(card)) {
            submit(
                e,
                who,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Card(card)],
                },
            );
            return;
        }
        settle_plain(e, &c);
    }
}

fn drain_plain(e: &mut Engine) {
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else {
            return;
        };
        settle_plain(e, &c);
    }
}
