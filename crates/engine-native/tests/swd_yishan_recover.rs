//! swd deferred 消解批 123：义山（`yishan`）——受伤取回最近失去的牌。
//!
//! 上游 `character/swd.js`：`yishan2`（`loseEnd` 记录 `storage.yishan`，
//! 新在前）+ `yishan`（`damageEnd` 可选：取回其中「不属于自己且不在特殊区」
//! 的至多 2 张）。

use engine_native::event::DamageNature;
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

/// 应答指定 prompt 的确认门（其余按可选放弃）。
fn confirm(e: &mut Engine, prompt: &str) -> bool {
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == prompt {
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

/// 义山：受伤后取回最近失去且仍在弃牌堆的至多 2 张（新在前）。
#[test]
fn yishan_recovers_two_recent_lost_cards() {
    let mut e = swd_engine(100001);
    e.debug_grant(0, "yishan");
    assert!(e.catalog().skill_by_name("yishan").is_some());
    e.debug_clear_hand(0);
    let c1 = e.debug_place(0, "sha");
    let c2 = e.debug_place(0, "shan");
    let c3 = e.debug_place(0, "tao");
    e.debug_give_hand_card(0, 0, c1);
    e.debug_give_hand_card(0, 0, c2);
    e.debug_give_hand_card(0, 0, c3);
    assert_eq!(e.debug_hand(0).len(), 3);
    // 弃置 3 张（按序：c3 最后弃 → 记录表新在前 [c3,c2,c1]）
    for c in [c1, c2, c3] {
        e.debug_discard_card(0, c);
    }
    assert_eq!(e.debug_hand(0).len(), 0);
    assert_eq!(
        e.state_core().player(0).yishan_lost,
        vec![c3, c2, c1],
        "记录新在前"
    );
    // 受伤 → 义山确认 → 取回 2 张（c3,c2）
    let out = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    assert!(confirm(&mut e, "trigger_confirm"), "义山确认门");
    drain(&mut e);
    assert_eq!(e.debug_hand(0).len(), 2, "取回 2 张");
    let hand = e.debug_hand(0);
    assert!(hand.contains(&c3) && hand.contains(&c2), "取回最近两张：{hand:?}");
    assert!(!hand.contains(&c1), "第三张不取回");
    assert_eq!(
        e.state_core().player(0).yishan_lost,
        vec![c1],
        "已取回的从记录表移除"
    );
}

/// 义山（对照）：无「仍在他处」的已失去牌 → 不开窗（全部已在自己手里）。
#[test]
fn yishan_no_window_when_nothing_recoverable() {
    let mut e = swd_engine(100002);
    e.debug_grant(0, "yishan");
    e.debug_clear_hand(0);
    let c1 = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, c1);
    e.debug_discard_card(0, c1);
    assert_eq!(e.state_core().player(0).yishan_lost, vec![c1]);
    // 先把牌取回手里（模拟牌已回到自己手上）
    {
        let core = e.state_core_mut();
        core.discard.retain(|c| *c != c1);
        core.players[0].hand.push(c1);
    }
    let out = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let c = e.open_choice().cloned();
    assert!(
        c.as_ref().map(|c| c.spec.prompt != "trigger_confirm").unwrap_or(true),
        "无可取回牌不应开窗：{:?}",
        c.map(|c| c.spec.prompt)
    );
    drain(&mut e);
}

/// 义山（对照）：未持技者不记录（记录表恒空）。
#[test]
fn yishan_records_only_for_holder() {
    let mut e = swd_engine(100003);
    e.debug_clear_hand(1);
    let c1 = e.debug_place(1, "sha");
    e.debug_give_hand_card(0, 1, c1);
    e.debug_discard_card(1, c1);
    assert!(
        e.state_core().player(1).yishan_lost.is_empty(),
        "未持技不记录"
    );
}

/// 义山：取回他人装备区的牌（跨区取回）与存档往返。
#[test]
fn yishan_recovers_from_other_equip_and_roundtrips() {
    let mut e = swd_engine(100004);
    e.debug_grant(0, "yishan");
    e.debug_clear_hand(0);
    let equip = e.debug_place(0, "armor");
    {
        let core = e.state_core_mut();
        core.players[0].hand.retain(|c| *c != equip);
        core.players[0].equips.push(equip);
    }
    // 装备被替换 → 记录
    let new_equip = e.debug_place(0, "armor");
    {
        let core = e.state_core_mut();
        core.players[0].hand.retain(|c| *c != new_equip);
        core.players[0].equips.retain(|c| *c != equip);
        core.players[0].equips.push(new_equip);
        core.discard.push(equip);
        core.players[0].yishan_lost.insert(0, equip);
    }
    assert_eq!(e.state_core().player(0).yishan_lost, vec![equip]);
    // 他人把该牌拿走（交牌给 1 号）→ 义山仍可跨区取回
    {
        let core = e.state_core_mut();
        core.discard.retain(|c| *c != equip);
        core.players[1].hand.push(equip);
    }
    let out = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    assert!(confirm(&mut e, "trigger_confirm"), "义山确认门");
    drain(&mut e);
    assert!(e.debug_hand(0).contains(&equip), "跨区取回他人手牌");
    assert!(!e.debug_hand(1).contains(&equip));
    // 存档往返保持记录表
    {
        let core = e.state_core_mut();
        core.players[0].yishan_lost = vec![equip];
    }
    let snap = e.save();
    assert_eq!(snap.core.players[0].yishan_lost, vec![equip]);
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .expect("读包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    let restored = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复");
    assert_eq!(restored.state_core().player(0).yishan_lost, vec![equip]);
}
