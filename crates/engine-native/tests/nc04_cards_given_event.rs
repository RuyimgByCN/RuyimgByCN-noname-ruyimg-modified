//! NC04 未映射事件批：`cards_given`（给牌触发窗）接入原生。
//!
//! 修复前反例（[red-loader.log] 于本批证据目录）：DSL `trigger cards_given
//! owner_alive_gave_cards_to_other_alive` 的包被装载器整包拒绝（「VM 技能
//! 事件名 cards_given 未映射」）——即台账「未映射事件阻断完整 shiji」
//! （shiji/ejian、sp/weicheng、标准 xinfu_jiyuan 同用该事件）。
//!
//! 修复后断言反转：DSL 包装载通过；仁德（rende）经共享原语 `give_hand_card`
//! 给牌后，订阅 `cards_given` 的 VM 技能真实触发——事件主玩家=recipient
//! （旧线 dispatch `TriggerContext{player: recipient, source: giver}` 同口径）；
//! 滤镜方向错误（持有者=接受者）不触发；optional 技能在确认窗暂停，存档
//! （含给牌帧的归档编码）往返后应答仍完成同一效果。
//!
//! [red-loader.log]: ../../../../docs/operations/evidence/2026-10-28-nc04-cards-given/red-loader.log

use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook, VmSkillDecl};
use engine_native::content::names;
use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase,
};
use rule_vm::bytecode::op;
use rule_vm::BytecodeModule;
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);
fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    let out = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action,
    });
    assert!(
        !matches!(out, CommandOutcome::Rejected(_)),
        "unexpected rejection: {out:?}"
    );
    out
}

/// draw_cards(event_player, 1)：`HOST_EVENT_PLAYER` 读触发上下文的事件主玩家
/// （给牌帧=recipient）。
fn draw_event_player_module(name: &str) -> BytecodeModule {
    let mut c = vec![op::HOST_EVENT_PLAYER];
    c.push(op::CONST_U32);
    c.extend_from_slice(&1u32.to_le_bytes());
    c.push(op::HOST_EFFECT_EXT);
    c.extend_from_slice(&1u16.to_le_bytes()); // DRAW_CARDS
    c.push(op::HALT);
    BytecodeModule {
        name: name.to_string(),
        code: c,
        local_count: 1,
        max_stack: 8,
        entry_pc: 0,
        logic_hash: [9u8; 32],
        caps_mask: rule_vm::bytecode::caps::EMIT_EFFECTS,
    }
}

fn cards_given_pack(optional: bool) -> ExtensionPack {
    let decl = VmSkillDecl {
        name: "cg_probe".to_string(),
        module: draw_event_player_module("cg_probe"),
        trigger: engine_native::catalog::TriggerMeta {
            event: engine_native::event::EventKind::CardsGiven,
            timing: engine_native::event::Timing::Normal,
            priority: 0,
            optional,
            extra_events: Vec::new(),
        },
        filter: engine_native::rvm_bridge::VmFilterPolicy::OwnerAliveGaveCardsToOtherAlive,
    };
    ExtensionPack {
        def: ExtensionDef {
            name: names::intern("nc04-cards-given".to_string()),
            version: 1,
            description: names::intern("nc04".to_string()),
        },
        generals: Vec::new(),
        cards: Vec::new(),
        host_skills: Vec::new(),
        vm_skills: vec![decl],
        init_hook: InitHook::None,
        enabled: true,
    }
}

fn answer(e: &mut Engine, selection: Vec<ChoiceOption>) {
    let choice = e.open_choice().cloned().expect("choice open");
    submit(
        e,
        choice.observer,
        GameAction::Answer {
            choice_id: choice.id,
            selection,
        },
    );
}

fn advance_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..64 {
        if e.turn().current == seat && e.turn().phase == Phase::Play {
            return;
        }
        if let Some(choice) = e.open_choice().cloned() {
            let sel: Vec<ChoiceOption> = choice
                .spec
                .options
                .iter()
                .take(choice.spec.min.max(1))
                .copied()
                .collect();
            let id = choice.id;
            let observer = choice.observer;
            submit(e, observer, GameAction::Answer { choice_id: id, selection: sel });
            continue;
        }
        let actor = e.turn().current;
        submit(e, actor, GameAction::NextPhase);
    }
    panic!("advance_to_play did not converge");
}

fn hand_len(e: &Engine, p: u8) -> usize {
    e.state_core().player(p).hand.len()
}

/// 仁德给 1 张牌给 1 号；返回给出去的那张牌。
fn rende_give_one(e: &mut Engine) -> engine_native::CardId {
    let def = e.catalog().skill_by_name("仁德").expect("rende def");
    submit(e, 0, GameAction::ActivateSkill { skill: def });
    let cards = e.state_core().player(0).hand.clone();
    let given = cards[0];
    answer(e, vec![ChoiceOption::Card(given)]);
    answer(e, vec![ChoiceOption::Player(1)]);
    given
}

/// 修复的反转：DSL `cards_given` 包经装载器映射为原生 EventKind 与滤镜。
#[test]
fn cards_given_dsl_package_bridges_to_native_event() {
    let source = "skill cg_dsl(990031) {\n  trigger cards_given owner_alive_gave_cards_to_other_alive {\n    effect draw_cards(event_player, 1);\n  }\n}\n";
    let mut pkg = content_compiler::compile_source_named("cg-dsl", source).unwrap();
    pkg.manifest.logic_hash = pkg.compute_logic_hash();
    let outcome = content_native_loader::bridge_vm_skill_decls(&pkg)
        .expect("cards_given 事件与滤镜已映射");
    assert_eq!(outcome.decls.len(), 1);
    let decl = &outcome.decls[0];
    assert_eq!(
        decl.trigger.event,
        engine_native::event::EventKind::CardsGiven
    );
    assert!(!decl.trigger.optional);
    assert!(matches!(
        decl.filter,
        engine_native::rvm_bridge::VmFilterPolicy::OwnerAliveGaveCardsToOtherAlive
    ));
}

/// 仁德给牌真实触发：事件主玩家=recipient（1 号收牌再摸 1），持有者=给牌者。
#[test]
fn rende_give_triggers_cards_given_probe_for_recipient() {
    let pack = cards_given_pack(false);
    let mut e = Engine::new_with_extensions(23001, 4, &[&pack]).unwrap();
    advance_to_play(&mut e, 0);
    e.debug_grant(0, "cg_probe");
    e.debug_grant(0, "仁德");
    let (h0, h1, deck) = (
        hand_len(&e, 0),
        hand_len(&e, 1),
        e.state_core().deck.len(),
    );
    assert!(h0 >= 1);
    rende_give_one(&mut e);
    // 0 号给出 1 张；1 号收入 1 张并被探针补摸 1 张（从牌堆）。
    assert_eq!(hand_len(&e, 0), h0 - 1);
    assert_eq!(hand_len(&e, 1), h1 + 2);
    assert_eq!(e.state_core().deck.len(), deck - 1);
    // 无挂起选择残留（锁定技自动结算）。
    assert!(e.open_choice().is_none());
}

/// 滤镜方向：持有者=接受者（非给牌者）不触发——owner==source 谓词。
#[test]
fn probe_on_recipient_side_does_not_trigger() {
    let pack = cards_given_pack(false);
    let mut e = Engine::new_with_extensions(23002, 4, &[&pack]).unwrap();
    advance_to_play(&mut e, 0);
    e.debug_grant(0, "仁德");
    e.debug_grant(1, "cg_probe");
    let (h0, h1, deck) = (
        hand_len(&e, 0),
        hand_len(&e, 1),
        e.state_core().deck.len(),
    );
    rende_give_one(&mut e);
    assert_eq!(hand_len(&e, 0), h0 - 1);
    assert_eq!(hand_len(&e, 1), h1 + 1, "无探针补摸");
    assert_eq!(e.state_core().deck.len(), deck);
}

/// optional 技能在确认窗暂停；含给牌帧的存档往返后应答完成同一效果。
#[test]
fn optional_probe_confirms_and_restores_across_save() {
    let pack = cards_given_pack(true);
    let mut e = Engine::new_with_extensions(23003, 4, &[&pack]).unwrap();
    advance_to_play(&mut e, 0);
    e.debug_grant(0, "cg_probe");
    e.debug_grant(0, "仁德");
    let (h0, h1) = (hand_len(&e, 0), hand_len(&e, 1));

    let def = e.catalog().skill_by_name("仁德").unwrap();
    submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    let cards = e.state_core().player(0).hand.clone();
    answer(&mut e, vec![ChoiceOption::Card(cards[0])]);
    answer(&mut e, vec![ChoiceOption::Player(1)]);
    // 给牌帧已在结算栈上等待确认（optional → trigger_confirm）。
    let choice = e.open_choice().cloned().expect("confirm gate open");
    assert_eq!((choice.observer, choice.spec.prompt), (0, "trigger_confirm"));
    let snapshot = e.save();
    let bytes = engine_native::archive::encode_archive(&snapshot);
    let decoded = engine_native::archive::decode_archive(&bytes).unwrap();
    let mut restored = Engine::restore_with_extensions(&decoded, &[&pack]).unwrap();
    assert_eq!(restored.state_hash(), e.state_hash());
    // 恢复后应答确认：recipient 补摸 1 张，与未中断路径一致。
    for game in [&mut restored, &mut e] {
        answer(game, vec![ChoiceOption::Confirm]);
        assert_eq!(hand_len(game, 0), h0 - 1);
        assert_eq!(hand_len(game, 1), h1 + 2);
        assert!(game.open_choice().is_none());
    }
}
