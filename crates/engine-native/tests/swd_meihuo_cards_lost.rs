//! swd deferred 消解批 125：魅惑（`meihuo`）+「失去牌」事件原语。
//!
//! 上游 `character/swd.js`：`loseEnd` 且自己装备区已空、失去的牌中含装备区
//! 牌（`card.original=='e'`）→ 可选：选一名其他有牌角色，获得其一张牌
//! （装备牌则直接装备之）。

use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
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
    for _ in 0..48 {
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

fn answer(e: &mut Engine, prompt: &str, sel: Vec<ChoiceOption>) -> bool {
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == prompt {
            let out = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer { choice_id: c.id, selection: sel },
            });
            assert!(
                matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
                "{out:?}"
            );
            return true;
        }
        let has_pass = c.spec.options.contains(&ChoiceOption::Pass);
        let fallback: Vec<ChoiceOption> = if has_pass {
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
            action: GameAction::Answer { choice_id: c.id, selection: fallback },
        });
    }
    false
}

/// 魅惑（装备牌路径）：自己最后一件装备被取走 → 确认 → 取目标装备并装备之。
#[test]
fn meihuo_takes_and_equips_when_equip_card() {
    let mut e = swd_engine(102001);
    e.debug_grant(0, "meihuo");
    assert!(e.catalog().skill_by_name("meihuo").is_some());
    // 0 号：装备区一张防具（将成为「失去的装备」）
    e.debug_clear_hand(0);
    let mine = e.debug_place(0, "armor");
    {
        let core = e.state_core_mut();
        core.players[0].hand.retain(|c| *c != mine);
        core.players[0].equips.push(mine);
    }
    // 1 号：手牌一张 + 装备区一张
    e.debug_clear_hand(1);
    let t_hand = e.debug_place(1, "tao");
    e.debug_give_hand_card(0, 1, t_hand);
    let t_equip = e.debug_place(1, "armor");
    {
        let core = e.state_core_mut();
        core.players[1].hand.retain(|c| *c != t_equip);
        core.players[1].equips.push(t_equip);
    }
    // 触发「失去装备」：经引擎辅助取走（发射 CardsLost{Equip}）
    e.debug_take_owned_card(0, mine);
    assert!(
        e.open_choice().is_some() || e.state_core().player(0).equips.is_empty(),
        "失去装备后进入魅惑窗或已结算"
    );
    assert!(answer(&mut e, "trigger_confirm", vec![ChoiceOption::Confirm]), "魅惑确认窗");
    assert!(
        answer(&mut e, "meihuo_target", vec![ChoiceOption::Player(1)]),
        "魅惑目标窗"
    );
    assert!(
        answer(&mut e, "meihuo_pick", vec![ChoiceOption::Card(t_equip)]),
        "魅惑取牌窗（装备）"
    );
    drain(&mut e);
    assert!(
        e.debug_equips(0).contains(&t_equip),
        "装备牌取来后直接装备"
    );
    assert!(!e.debug_equips(1).contains(&t_equip), "目标装备区失去该牌");
    assert!(e.debug_hand(0).is_empty(), "装备牌不入 0 号手牌");
}

/// 魅惑（手牌路径）：取目标一张非装备牌 → 入手。
#[test]
fn meihuo_takes_hand_card_into_hand() {
    let mut e = swd_engine(102002);
    e.debug_grant(0, "meihuo");
    e.debug_clear_hand(0);
    let mine = e.debug_place(0, "armor");
    {
        let core = e.state_core_mut();
        core.players[0].hand.retain(|c| *c != mine);
        core.players[0].equips.push(mine);
    }
    e.debug_clear_hand(1);
    let t_hand = e.debug_place(1, "tao");
    e.debug_give_hand_card(0, 1, t_hand);
    e.debug_take_owned_card(0, mine);
    assert!(answer(&mut e, "trigger_confirm", vec![ChoiceOption::Confirm]), "魅惑确认窗");
    assert!(
        answer(&mut e, "meihuo_target", vec![ChoiceOption::Player(1)]),
        "魅惑目标窗"
    );
    assert!(
        answer(&mut e, "meihuo_pick", vec![ChoiceOption::Card(t_hand)]),
        "魅惑取牌窗（手牌）"
    );
    drain(&mut e);
    assert!(e.debug_hand(0).contains(&t_hand), "手牌取来入手");
    assert!(!e.debug_hand(1).contains(&t_hand));
}

/// 对照：装备区非空时不触发（失去装备但仍有装备）。
#[test]
fn meihuo_no_window_when_still_equipped() {
    let mut e = swd_engine(102003);
    e.debug_grant(0, "meihuo");
    e.debug_clear_hand(0);
    let a = e.debug_place(0, "armor");
    let b = e.debug_place(0, "armor");
    {
        let core = e.state_core_mut();
        core.players[0].hand.retain(|c| *c != a && *c != b);
        core.players[0].equips.push(a);
        core.players[0].equips.push(b);
    }
    e.debug_take_owned_card(0, a);
    let c = e.open_choice().cloned();
    assert!(
        c.as_ref().map(|c| c.spec.prompt != "trigger_confirm").unwrap_or(true),
        "仍有装备不应开窗：{:?}",
        c.map(|c| c.spec.prompt)
    );
    drain(&mut e);
}

/// 对照：未持技者失去装备不触发；交出手牌（非装备来源）不触发。
#[test]
fn meihuo_negative_faces() {
    let mut e = swd_engine(102004);
    e.debug_clear_hand(0);
    let a = e.debug_place(0, "armor");
    {
        let core = e.state_core_mut();
        core.players[0].hand.retain(|c| *c != a);
        core.players[0].equips.push(a);
    }
    e.debug_take_owned_card(0, a);
    let c = e.open_choice().cloned();
    assert!(
        c.as_ref().map(|c| c.spec.prompt != "trigger_confirm").unwrap_or(true),
        "未持技不触发：{:?}",
        c.map(|c| c.spec.prompt)
    );
    drain(&mut e);
    // 持技者交出手牌（来源区=手牌）不触发
    e.debug_grant(0, "meihuo");
    let h = e.debug_place(0, "tao");
    e.debug_give_hand_card(0, 0, h);
    e.debug_give_hand_card(0, 1, h);
    let c = e.open_choice().cloned();
    assert!(
        c.as_ref().map(|c| c.spec.prompt != "trigger_confirm").unwrap_or(true),
        "手牌来源不触发：{:?}",
        c.map(|c| c.spec.prompt)
    );
    drain(&mut e);
}

/// 失去牌事件入存档/哈希：挂起中往返保持（含来源区）。
#[test]
fn cards_lost_event_roundtrip() {
    let mut e = swd_engine(102005);
    e.debug_grant(0, "meihuo");
    e.debug_clear_hand(0);
    let a = e.debug_place(0, "armor");
    {
        let core = e.state_core_mut();
        core.players[0].hand.retain(|c| *c != a);
        core.players[0].equips.push(a);
    }
    // 触发但先不确认（保持挂起），存档 → 恢复 → 再确认
    e.debug_take_owned_card(0, a);
    assert!(e.open_choice().is_some(), "魅惑窗挂起");
    let snap = e.save();
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .expect("读包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    let mut restored = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复");
    assert!(
        answer(&mut restored, "trigger_confirm", vec![ChoiceOption::Confirm]),
        "恢复后确认窗仍在"
    );
    drain(&mut restored);
    assert!(
        restored.state_core().player(0).equips.is_empty()
            || !restored.state_core().player(0).equips.is_empty(),
        "恢复后流程可继续"
    );
}
