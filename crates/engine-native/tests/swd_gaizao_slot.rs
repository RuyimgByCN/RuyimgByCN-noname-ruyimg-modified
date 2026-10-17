//! swd deferred 消解批 143：改造（`gaizao`）——装备牌改放到空栏位。
//!
//! 上游 `character/swd.js`：`trigger {player:'useCardToBegin'}`，`direct:true`
//! ——自己使用装备牌且其固有栏位已被占用时可选：改放到一个**空栏位**
//! （牌种不变故装备技能保留）。

use engine_native::catalog::EquipSlot;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn repo_extensions_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn swd_engine(seed: u64) -> Engine {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .expect("读 swd 包数据");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接 swd 包").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        let has_pass = c.spec.options.contains(&ChoiceOption::Pass);
        let sel: Vec<ChoiceOption> = if has_pass {
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

fn next_phase(e: &mut Engine) {
    let actor = e.turn().current;
    let out = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
}

fn advance_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..12 {
        if e.turn().current == seat && e.turn().phase == Phase::Play {
            return;
        }
        next_phase(e);
        drain(e);
    }
    panic!("未到 {seat} 号出牌阶段");
}

/// 改造：固有栏位（武器）已被占用 → 选空栏位（防具）→ 入防具栏且技能保留。
#[test]
fn gaizao_moves_equip_to_empty_slot() {
    let mut e = swd_engine(120001);
    e.debug_grant(0, "gaizao");
    assert!(e.catalog().skill_by_name("gaizao").is_some());
    advance_to_play(&mut e, 0);
    // 0 号已装备一把武器（占用武器栏）
    let old_weapon = e.debug_place(0, "weapon2");
    {
        let core = e.state_core_mut();
        core.players[0].hand.retain(|c| *c != old_weapon);
        core.players[0].equips.push(old_weapon);
    }
    // 再用一把武器 → 改造窗
    let weapon = e.debug_place(0, "weapon2");
    e.debug_give_hand_card(0, 0, weapon);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: weapon, targets: Vec::new() },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    // 确认门 + 栏位窗（选防具=Tag 1）
    let mut answered = false;
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "trigger_confirm" {
            let out = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Confirm],
                },
            });
            assert!(
                matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
                "{out:?}"
            );
            continue;
        }
        if c.spec.prompt == "gaizao_slot" {
            assert!(
                c.spec.options.contains(&ChoiceOption::Tag(1)),
                "空栏位含防具：{:?}",
                c.spec.options
            );
            let out = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Tag(1)],
                },
            });
            assert!(
                matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
                "{out:?}"
            );
            answered = true;
            break;
        }
        break;
    }
    assert!(answered, "出现改造栏位窗");
    drain(&mut e);
    // 新武器占防具栏、旧武器仍在武器栏
    assert_eq!(
        e.catalog().card_slot_of(e.state_core(), weapon),
        Some(EquipSlot::Armor),
        "改造后占防具栏"
    );
    assert!(e.state_core().player(0).equips.contains(&weapon));
    assert!(e.state_core().player(0).equips.contains(&old_weapon), "旧武器未被替换");
}

/// 改造（对照）：固有栏位空着 → 不开窗（正常入槽）。
#[test]
fn gaizao_no_window_when_slot_free() {
    let mut e = swd_engine(120002);
    e.debug_grant(0, "gaizao");
    advance_to_play(&mut e, 0);
    let weapon = e.debug_place(0, "weapon2");
    e.debug_give_hand_card(0, 0, weapon);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: weapon, targets: Vec::new() },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let c = e.open_choice().cloned();
    assert!(
        c.as_ref().map(|c| c.spec.prompt != "gaizao_slot").unwrap_or(true),
        "栏位空着不开窗：{:?}",
        c.map(|c| c.spec.prompt)
    );
    drain(&mut e);
    assert_eq!(
        e.catalog().card_slot_of(e.state_core(), weapon),
        Some(EquipSlot::Weapon)
    );
}

/// 改造（对照）：放弃改造 → 正常替换原武器。
#[test]
fn gaizao_decline_replaces_normally() {
    let mut e = swd_engine(120003);
    e.debug_grant(0, "gaizao");
    advance_to_play(&mut e, 0);
    let old_weapon = e.debug_place(0, "weapon2");
    {
        let core = e.state_core_mut();
        core.players[0].hand.retain(|c| *c != old_weapon);
        core.players[0].equips.push(old_weapon);
    }
    let weapon = e.debug_place(0, "weapon2");
    e.debug_give_hand_card(0, 0, weapon);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: weapon, targets: Vec::new() },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "trigger_confirm" {
            let out = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer { choice_id: c.id, selection: Vec::new() },
            });
            assert!(
                matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
                "{out:?}"
            );
            break;
        }
        break;
    }
    drain(&mut e);
    assert!(
        !e.state_core().player(0).equips.contains(&old_weapon),
        "放弃改造：原武器被替换"
    );
    assert!(e.state_core().player(0).equips.contains(&weapon));
}

/// 改造：`declared_slots` 存档往返。
#[test]
fn gaizao_state_roundtrip() {
    let mut e = swd_engine(120004);
    e.state_core_mut().declared_slots.insert(5, EquipSlot::Armor);
    let snap = e.save();
    assert_eq!(snap.core.declared_slots.get(&5), Some(&EquipSlot::Armor));
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .expect("读包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    let restored = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复");
    assert_eq!(
        restored.state_core().declared_slots.get(&5),
        Some(&EquipSlot::Armor)
    );
}
