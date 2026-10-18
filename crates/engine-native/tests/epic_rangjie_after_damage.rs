//! 史诗卡牌批 2：让节（`rangjie`）——受伤后获得指定类型牌 / 移动场上牌。
//!
//! 上游 `extension/史诗卡牌/extension.js`：`trigger {player:'damageEnd'}`，
//! `direct:true`——按伤害点数逐次选择：「获得牌」（摸 1 + 从牌堆取一张指定
//! 类型牌）或「移动牌」（`player.moveCard(true)`）或取消。

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

fn engine(seed: u64) -> Engine {
    let epic = std::fs::read_to_string(
        repo_extensions_root().join("epic-cards-upstream-liuli/assets/cards.json"),
    )
    .expect("读史诗卡牌包数据");
    let pack = content_native_loader::bridge_pack(&epic).expect("桥接史诗卡牌").pack;
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

fn answer(e: &mut Engine, prompt: &str, sel: Vec<ChoiceOption>) -> bool {
    for _ in 0..32 {
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
        let fallback: Vec<ChoiceOption> = if c.spec.prompt == "trigger_confirm" {
            vec![ChoiceOption::Confirm]
        } else if has_pass {
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

/// 让节（获得牌分支）：受伤 1 点 → 确认 → 选「获得牌」→ 选「基本」→ 摸 1 + 得一张基本牌。
#[test]
fn rangjie_gains_basic_card() {
    let mut e = engine(133001);
    e.debug_grant(0, "rangjie");
    assert!(e.catalog().skill_by_name("rangjie").is_some());
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let out = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    assert!(
        answer(&mut e, "trigger_confirm", vec![ChoiceOption::Confirm]),
        "让节确认门"
    );
    assert!(
        answer(&mut e, "rangjie_choice", vec![ChoiceOption::Tag(0)]),
        "让节选择窗（获得牌）"
    );
    assert!(
        answer(&mut e, "rangjie_type", vec![ChoiceOption::Tag(0)]),
        "让节类型窗（基本）"
    );
    drain(&mut e);
    // 摸 1 + 得 1 基本牌 = 手牌 2（且均为基本牌）
    let hand = e.debug_hand(0);
    assert_eq!(hand.len(), 2, "摸 1 + 得 1：{hand:?}");
    // 摸 1 的类型不受限；「获得牌」所得必为基本牌
    assert!(
        hand.iter().any(|c| e.catalog().card_kind(*c).is_basic_kind()),
        "至少一张为所得基本牌：{hand:?}"
    );
}

/// 让节（移动牌分支）：把 2 号装备区一张牌移到 3 号（空栏位 → 装备）。
#[test]
fn rangjie_moves_card_between_players() {
    let mut e = engine(133002);
    e.debug_grant(0, "rangjie");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 2 号装备一把武器
    let weapon = e
        .state_core()
        .deck
        .iter()
        .copied()
        .find(|c| e.catalog().card_kind(*c).equip_slot().is_some())
        .expect("牌堆中有装备");
    {
        let core = e.state_core_mut();
        core.deck.retain(|c| *c != weapon);
        core.players[2].equips.push(weapon);
    }
    let out = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    assert!(
        answer(&mut e, "trigger_confirm", vec![ChoiceOption::Confirm]),
        "确认门"
    );
    assert!(
        answer(&mut e, "rangjie_choice", vec![ChoiceOption::Tag(1)]),
        "选择窗（移动牌）"
    );
    assert!(
        answer(&mut e, "rangjie_move_card", vec![ChoiceOption::Card(weapon)]),
        "移动选牌窗"
    );
    assert!(
        answer(&mut e, "rangjie_move_target", vec![ChoiceOption::Player(3)]),
        "移动目标窗"
    );
    drain(&mut e);
    assert!(!e.state_core().player(2).equips.contains(&weapon), "已离开 2 号");
    assert!(
        e.state_core().player(3).equips.contains(&weapon),
        "已入 3 号装备区（空栏位）"
    );
}

/// 让节（对照）：放弃 → 不获得/不移动；未持技者受伤无窗。
#[test]
fn rangjie_decline_and_absent() {
    let mut e = engine(133003);
    e.debug_grant(0, "rangjie");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let out = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    assert!(
        answer(&mut e, "trigger_confirm", Vec::new()),
        "确认门（放弃）"
    );
    drain(&mut e);
    assert_eq!(e.debug_hand(0).len(), 0, "放弃：无所得");
    // 未持技
    let mut e2 = engine(133004);
    for p in 0..4 {
        e2.debug_clear_hand(p);
    }
    let out2 = e2.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    assert!(
        matches!(out2, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out2:?}"
    );
    let c = e2.open_choice().cloned();
    assert!(
        c.as_ref().map(|c| c.spec.prompt != "rangjie_choice").unwrap_or(true),
        "未持技不开窗：{:?}",
        c.map(|c| c.spec.prompt)
    );
    drain(&mut e2);
}
