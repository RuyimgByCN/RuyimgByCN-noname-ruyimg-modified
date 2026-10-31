//! swd deferred 消解批 146：轩辕剑（卡 + 三子技）+ 轩辕（取剑）。
//!
//! 上游 `card/swd.js`：`xuanyuanjian`（独特武器，攻击范围 +2，装备时 +1
//! 护甲；体力 ≤2 且无「轩辕」则弃置）+ `xuanyuanjian2`（造成伤害 +1 且转
//! 雷属性）+ `xuanyuanjian3`（加伤后无「轩辕」则失去 1 体力）；`swd.js`
//! `xuanyuan`（结束阶段限定技：弃黑桃牌，从弃牌堆取【轩辕剑】装备）。

use engine_native::event::DamageNature;
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
    let pack = content_native_loader::bridge_pack(&json)
        .expect("桥接 swd 包")
        .pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        let has_pass = c.spec.options.contains(&ChoiceOption::Pass);
        let sel: Vec<ChoiceOption> = if has_pass {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec
                .options
                .iter()
                .take(c.spec.min.max(1))
                .copied()
                .collect()
        };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        });
    }
}

/// 按牌种取一张牌入手（包内牌名带前缀/序号，按种定位更稳）。
fn place_by_kind(e: &mut Engine, player: u8, kind: engine_native::catalog::CardKind) -> u32 {
    let found = e
        .state_core()
        .deck
        .iter()
        .copied()
        .find(|c| e.catalog().cards[*c as usize].kind == kind)
        .expect("牌堆中有该牌种");
    let core = e.state_core_mut();
    core.deck.retain(|c| *c != found);
    core.players[player as usize].hand.push(found);
    found
}

/// 轩辕剑（对照）：持「轩辕」时无反噬。
#[test]
fn xuanyuanjian_no_backlash_with_xuanyuan() {
    let mut e = swd_engine(123002);
    e.debug_grant(0, "xuanyuanjian2");
    e.debug_grant(0, "xuanyuanjian3");
    e.debug_grant(0, "xuanyuan");
    let hp2 = e.debug_hp(2);
    let hp0 = e.debug_hp(0);
    let out = e.debug_deal_damage(Some(0), 2, 1, DamageNature::Normal);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(2), hp2 - 2, "加伤 1");
    assert_eq!(e.debug_hp(0), hp0, "持「轩辕」不反噬");
}

/// 轩辕剑：`xuanyuanjian_marked` 字段存档往返。
#[test]
fn xuanyuanjian_state_roundtrip() {
    let mut e = swd_engine(123004);
    e.state_core_mut().turn.xuanyuanjian_marked = true;
    let snap = e.save();
    assert!(snap.core.turn.xuanyuanjian_marked);
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .expect("读包");
    let pack = content_native_loader::bridge_pack(&json)
        .expect("桥接")
        .pack;
    let restored = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复");
    assert!(restored.state_core().turn.xuanyuanjian_marked);
}

/// 轩辕剑：装备时 +1 护甲（体力 4 > 2 → 正常装备）。
#[test]
fn xuanyuanjian_equip_grants_hujia() {
    let mut e = swd_engine(123005);
    for _ in 0..12 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play {
            break;
        }
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        drain(&mut e);
    }
    let sword = place_by_kind(&mut e, 0, engine_native::catalog::CardKind::Xuanyuanjian);
    let hujia = e.state_core().player(0).hujia;
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: sword,
            targets: Vec::new(),
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    drain(&mut e);
    assert!(e.state_core().player(0).equips.contains(&sword), "已装备");
    assert_eq!(e.state_core().player(0).hujia, hujia + 1, "装备时 +1 护甲");
}

/// 轩辕：结束阶段弃黑桃牌 → 从弃牌堆取【轩辕剑】装备。
#[test]
fn xuanyuan_retrieves_sword_from_discard() {
    let mut e = swd_engine(123006);
    e.debug_grant(0, "xuanyuan");
    for _ in 0..12 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play {
            break;
        }
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        drain(&mut e);
    }
    let sword = place_by_kind(&mut e, 0, engine_native::catalog::CardKind::Xuanyuanjian);
    {
        let core = e.state_core_mut();
        core.players[0].hand.retain(|c| *c != sword);
        core.discard.push(sword);
    }
    e.debug_clear_hand(0);
    let spade = {
        let found = e
            .state_core()
            .deck
            .iter()
            .copied()
            .find(|c| {
                e.catalog().cards[*c as usize].suit == engine_native::catalog::Suit::Spade
                    && e.catalog().cards[*c as usize].kind.is_basic_kind()
            })
            .expect("黑桃基本牌");
        let core = e.state_core_mut();
        core.deck.retain(|c| *c != found);
        core.players[0].hand.push(found);
        found
    };
    let mut found = false;
    for _ in 0..24 {
        if let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt == "xuanyuan_discard" {
                let out = e.submit(GameCommand {
                    actor: c.observer,
                    expected_revision: e.revision(),
                    dedup_id: next_dedup(),
                    action: GameAction::Answer {
                        choice_id: c.id,
                        selection: vec![ChoiceOption::Card(spade)],
                    },
                });
                assert!(
                    matches!(
                        out,
                        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
                    ),
                    "{out:?}"
                );
                found = true;
                break;
            }
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
                    matches!(
                        out,
                        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
                    ),
                    "{out:?}"
                );
                continue;
            }
            let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else if c.spec.allow_cancel {
                Vec::new()
            } else {
                c.spec
                    .options
                    .iter()
                    .take(c.spec.min.max(1))
                    .copied()
                    .collect()
            };
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            });
            continue;
        }
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
    }
    assert!(found, "出现轩辕弃牌窗");
    drain(&mut e);
    assert!(
        e.state_core().player(0).equips.contains(&sword),
        "轩辕剑已从弃牌堆装备"
    );
    assert!(!e.state_core().discard.contains(&sword), "已离开弃牌堆");
}
