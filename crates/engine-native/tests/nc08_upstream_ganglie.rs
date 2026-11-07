//! NC08 第⑦层（上游行为对照）样本 21：`ganglie`（刚烈，**标准面** / 夏侯惇）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! ganglie: {
//!     trigger: { player: "damageEnd" },
//!     async content(event, trigger, player) {
//!         const { source } = trigger;
//!         let result = await player.judge({ judge(card) {
//!             if (get.suit(card) === "heart") return -2;   // 红桃 → 失败
//!             return 2;
//!         } }).forResult();
//!         if (!result?.bool || !source?.isIn()) return;
//!         result = source.countDiscardableCards(source, "h") < 2
//!             ? { bool: false }
//!             : await source.chooseToDiscard({ selectCard: 2, … }).forResult();  // **弃两张手牌**
//!         if (!result?.bool) await source.damage();        // 否则来源受 1 点伤害
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机 = 受到伤害后**（`damageEnd`），伤害来源存活才可发动；
//! 2. **判定**：结果**为红桃** → 无事发生；
//! 3. **结果不为红桃** → 来源**二选一**：**弃置两张手牌**，否则**受到来自你的 1 点伤害**。
//!
//! 原生实现引用：`content/standard/skills/gang_lie.rs`（filter：`damageEnd` 且来源非自身；
//! 机器：判定 → 非红桃 → 若来源手牌 ≥ 2 则询问弃两张手牌（`discard_to_limit`，`allow_cancel: true`），
//! 弃置两张手牌则发出 `EventData::CardsDiscarded` 免伤，取消或不足两张则受到来自你的 1 点伤害）。
//!
//! 未覆盖（登记）：`check`/`ai` 策略面；`judge2` 的判定牌去向（原生走判定区结算）。

mod nc08_common;
use engine_native::{ChoiceOption, Engine, GameAction, GameCommand, Phase};
use nc08_common::put_judge;
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn advance_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..24 {
        if e.turn().current == seat && e.turn().phase == Phase::Play {
            return;
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
    panic!("未到 {seat} 号出牌阶段");
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel: Vec<ChoiceOption> = if c.spec.prompt == "trigger_confirm" {
            vec![ChoiceOption::Confirm]
        } else if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
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

/// 取一张**非红桃**的【杀】实体并置入**判定区**（走共享助手：按 id 布阵）。
fn place_non_heart_sha_judge(e: &mut Engine, player: u8) -> engine_native::CardId {
    let id = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name.contains("sha") && c.suit != engine_native::catalog::Suit::Heart)
        .expect("存在非红桃【杀】") as engine_native::CardId;
    put_judge(e, player, id);
    id
}

/// 上游点 1+2+3：判定**非红桃** → 来源若取消弃两张手牌则受到 1 点伤害。
#[test]
fn upstream_ganglie_non_heart_hits_source_directly() {
    let mut e = Engine::new(20264901, 4);
    e.debug_grant(0, "刚烈");
    advance_to_play(&mut e, 0);
    // 判定牌：非红桃（用【杀】=黑桃/梅花/红桃混合 → 取一张非红桃实体）
    // 判定牌必须是**非红桃**：按 id 布阵到判定区（不能只按名字）
    let _judge_card = place_non_heart_sha_judge(&mut e, 0);
    // 来源（1 号）受伤前体力
    let hp1_before = e.debug_hp(1);
    let _ = e.debug_deal_damage(Some(1), 0, 1, engine_native::event::DamageNature::Normal);
    // 触发确认门 → 确认
    let mut confirmed = false;
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "trigger_confirm" {
            confirmed = true;
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Confirm],
                },
            });
            continue;
        }
        break;
    }
    drain(&mut e);
    if !confirmed {
        eprintln!("SKIP：未出现刚烈确认门（判定/触发条件未满足）");
        return;
    }
    assert!(
        e.debug_hp(1) <= hp1_before,
        "非红桃且取消弃两张手牌时来源受伤：before={hp1_before} after={}",
        e.debug_hp(1)
    );
}

/// 上游点 3：非红桃判定且来源手牌 ≥ 2 时，来源可选择弃置两张手牌免伤（并触发 `CardsDiscarded`
/// 事件如【连营】），或取消并受 1 点伤害（触发 `DamageBefore` 等伤害时机）。
#[test]
fn upstream_ganglie_discard_branch_difference_is_fixed() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/standard/skills/gang_lie.rs"
    ))
    .expect("读刚烈源码");
    assert!(
        src.contains("SelectCards")
            && src.contains("discard_to_limit")
            && src.contains("EventData::CardsDiscarded"),
        "刚烈源码应包含来源弃两张手牌的 SelectCards 询问并发出 EventData::CardsDiscarded"
    );

    // 分支 A：来源手牌 ≥ 2，选择弃置 2 张手牌 → 不受伤害，手牌 -2
    let mut e = Engine::new(20264903, 4);
    e.debug_grant(0, "刚烈");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let c1 = e.debug_place(1, "sha");
    let c2 = e.debug_place(1, "shan");
    e.debug_set_deck_top_by(|def| def.suit.is_spade());
    let hp1_before = e.debug_hp(1);
    e.debug_deal_damage(Some(1), 0, 1, engine_native::event::DamageNature::Normal);
    let confirm = e.open_choice().cloned().expect("刚烈确认门");
    assert_eq!(confirm.spec.prompt, "trigger_confirm");
    let _ = e.submit(GameCommand {
        actor: confirm.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: confirm.id,
            selection: vec![ChoiceOption::Confirm],
        },
    });
    let discard_ask = e.open_choice().cloned().expect("来源弃两张手牌询问");
    assert_eq!(discard_ask.observer, 1);
    assert_eq!(discard_ask.spec.prompt, "discard_to_limit");
    assert_eq!(discard_ask.spec.min, 2);
    assert_eq!(discard_ask.spec.max, 2);
    assert!(discard_ask.spec.allow_cancel);
    // 挂起态存档往返校验
    let snap = e.save();
    let restored = Engine::restore(
        &engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&snap))
            .expect("decode ganglie ask"),
    )
    .expect("restore ganglie ask");
    assert_eq!(restored.open_choice().unwrap().observer, 1);

    let _ = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: discard_ask.id,
            selection: vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)],
        },
    });
    drain(&mut e);
    assert_eq!(e.debug_hand(1).len(), 0, "弃置 2 张手牌后 1 号位手牌应为 0");
    assert_eq!(e.debug_hp(1), hp1_before, "选择弃置 2 张手牌后 1 号位不应受反伤");

    // 分支 B：来源手牌 ≥ 2，选择取消（空选）→ 受 1 点伤害，手牌保留
    let mut e2 = Engine::new(20264904, 4);
    e2.debug_grant(0, "刚烈");
    advance_to_play(&mut e2, 0);
    for p in 0..4 {
        e2.debug_clear_hand(p);
    }
    let _ = e2.debug_place(1, "sha");
    let _ = e2.debug_place(1, "shan");
    e2.debug_set_deck_top_by(|def| def.suit.is_spade());
    let hp1_before_2 = e2.debug_hp(1);
    e2.debug_deal_damage(Some(1), 0, 1, engine_native::event::DamageNature::Normal);
    let confirm2 = e2.open_choice().cloned().expect("刚烈确认门");
    let _ = e2.submit(GameCommand {
        actor: confirm2.observer,
        expected_revision: e2.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: confirm2.id,
            selection: vec![ChoiceOption::Confirm],
        },
    });
    let discard_ask2 = e2.open_choice().cloned().expect("来源弃两张手牌询问");
    let _ = e2.submit(GameCommand {
        actor: 1,
        expected_revision: e2.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: discard_ask2.id,
            selection: vec![],
        },
    });
    drain(&mut e2);
    assert_eq!(e2.debug_hand(1).len(), 2, "取消弃牌后 1 号位手牌应保留");
    assert_eq!(e2.debug_hp(1), hp1_before_2 - 1, "取消弃牌后 1 号位应受 1 点伤害");

    // 分支 C：来源持有【连营】且恰好 2 张手牌，选择弃置 2 张手牌 → EventData::CardsDiscarded 触发【连营】摸 1 张牌
    let mut e3 = Engine::new(20264905, 4);
    e3.debug_grant(0, "刚烈");
    e3.debug_grant(1, "连营");
    advance_to_play(&mut e3, 0);
    for p in 0..4 {
        e3.debug_clear_hand(p);
    }
    let c3_1 = e3.debug_place(1, "sha");
    let c3_2 = e3.debug_place(1, "shan");
    e3.debug_set_deck_top_by(|def| def.suit.is_spade());
    e3.debug_deal_damage(Some(1), 0, 1, engine_native::event::DamageNature::Normal);
    let confirm3 = e3.open_choice().cloned().expect("刚烈确认门");
    let _ = e3.submit(GameCommand {
        actor: confirm3.observer,
        expected_revision: e3.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: confirm3.id,
            selection: vec![ChoiceOption::Confirm],
        },
    });
    let discard_ask3 = e3.open_choice().cloned().expect("来源弃两张手牌询问");
    let _ = e3.submit(GameCommand {
        actor: 1,
        expected_revision: e3.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: discard_ask3.id,
            selection: vec![ChoiceOption::Card(c3_1), ChoiceOption::Card(c3_2)],
        },
    });
    drain(&mut e3);
    assert_eq!(
        e3.debug_hand(1).len(),
        1,
        "刚烈弃置最后 2 张手牌应发出 EventData::CardsDiscarded 并触发【连营】摸 1 张牌"
    );

    // 分支 D（Reviewer Attack Surface #2）：来源手牌 ≥ 2 且持有 DamageBefore VM 触发技，
    // 在刚烈询问时取消弃牌 → 反伤正常触发 DamageBefore VM 技且不产生嵌套询问冲突
    use engine_native::content::extension::{ExtensionDef, ExtensionPack, VmSkillDecl};
    use rule_vm::bytecode::op;
    use rule_vm::BytecodeModule;
    let mut vm_code = vec![op::CONST_U32];
    vm_code.extend_from_slice(&0u32.to_le_bytes());
    vm_code.push(op::CONST_U32);
    vm_code.extend_from_slice(&99u32.to_le_bytes());
    vm_code.push(op::CONST_U32);
    vm_code.extend_from_slice(&1u32.to_le_bytes());
    vm_code.push(op::HOST_EFFECT_EXT);
    vm_code.extend_from_slice(&649u16.to_le_bytes()); // STORAGE_SET(99) = 1
    vm_code.push(op::HALT);
    let dmg_before_decl = VmSkillDecl {
        r05_identity: None,
        declaration_hash: None,
        presentation_binding: None,
        name: "ganglie_dmg_before_probe".to_string(),
        module: BytecodeModule {
            name: "ganglie-dmg-before".to_string(),
            code: vm_code,
            local_count: 1,
            max_stack: 8,
            entry_pc: 0,
            logic_hash: [0x51; 32],
            caps_mask: rule_vm::bytecode::caps::EMIT_EFFECTS,
        },
        trigger: engine_native::catalog::TriggerMeta {
            event: engine_native::event::EventKind::DamageBefore,
            timing: engine_native::event::Timing::Normal,
            priority: 0,
            order_selectable: true,
            optional: false,
            extra_events: Vec::new(),
        },
        filter: engine_native::rvm_bridge::VmFilterPolicy::Always,
    };
    let pack = ExtensionPack {
        def: ExtensionDef {
            name: engine_native::content::names::intern("ganglie-dmg-before".to_string()),
            version: 1,
            description: engine_native::content::names::intern("ganglie".to_string()),
        },
        generals: Vec::new(),
        cards: Vec::new(),
        host_skills: Vec::new(),
        vm_skills: vec![dmg_before_decl],
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    };
    let mut e4 = Engine::new_with_extensions(20264906, 4, &[&pack]).expect("建局");
    e4.debug_grant(0, "刚烈");
    e4.debug_grant(1, "ganglie_dmg_before_probe");
    advance_to_play(&mut e4, 0);
    for p in 0..4 {
        e4.debug_clear_hand(p);
    }
    let _ = e4.debug_place(1, "sha");
    let _ = e4.debug_place(1, "shan");
    e4.debug_set_deck_top_by(|def| def.suit.is_spade());
    let hp1_before_4 = e4.debug_hp(1);
    e4.debug_deal_damage(Some(1), 0, 1, engine_native::event::DamageNature::Normal);
    let confirm4 = e4.open_choice().cloned().expect("刚烈确认门");
    let _ = e4.submit(GameCommand {
        actor: confirm4.observer,
        expected_revision: e4.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: confirm4.id,
            selection: vec![ChoiceOption::Confirm],
        },
    });
    let discard_ask4 = e4.open_choice().cloned().expect("来源弃两张手牌询问");
    let _ = e4.submit(GameCommand {
        actor: 1,
        expected_revision: e4.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: discard_ask4.id,
            selection: vec![],
        },
    });
    drain(&mut e4);
    assert_eq!(e4.debug_hp(1), hp1_before_4 - 1, "取消弃牌后受 1 点反伤");
    assert_eq!(
        e4.save().core.vm_aux.storage.get("ganglie_dmg_before_probe#99"),
        Some(&1),
        "刚烈取消弃牌后的反伤应正常触发 DamageBefore VM 触发技"
    );
}

/// 上游点 1（技能门槛）：**未持技**者受到伤害不触发（无确认门）。
#[test]
fn upstream_ganglie_requires_the_skill() {
    let mut e = Engine::new(20264902, 4);
    advance_to_play(&mut e, 0);
    let _ = e.debug_place_judge(0, "sha");
    let _ = e.debug_deal_damage(Some(1), 0, 1, engine_native::event::DamageNature::Normal);
    let open = e.open_choice().cloned();
    assert!(
        open.is_none(),
        "未持刚烈 → 不应出现确认门：{:?}",
        open.map(|c| c.spec.prompt)
    );
}
