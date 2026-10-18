//! swd deferred 消解批 148：神弓（`shengong`）——借用他人装备响应。
//!
//! 上游 `character/swd.js`：`chooseToRespondBegin` 可选——需要打出【闪】
//! 时借用**其他角色**的防具、需要【杀】时借用其武器作为响应（原牌从持有
//! 者装备区取走）。

use engine_native::catalog::{CardKind, Suit};
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

/// 取一张指定牌种的牌放入某玩家装备区（按种定位；包内牌名带序号）。
fn equip_by_kind(e: &mut Engine, player: u8, kind: CardKind) -> u32 {
    let found = e
        .state_core()
        .deck
        .iter()
        .copied()
        .find(|c| e.catalog().cards[*c as usize].kind == kind)
        .expect("牌堆中有该牌种");
    let core = e.state_core_mut();
    core.deck.retain(|c| *c != found);
    core.players[player as usize].hand.retain(|c| *c != found);
    core.players[player as usize].equips.push(found);
    found
}

/// 神弓：被【杀】指定为目标且无闪 → 借用他人防具闪避（防具从其装备区取走）。
#[test]
fn shengong_borrows_armor_to_dodge() {
    let mut e = swd_engine(124001);
    e.debug_grant(1, "shengong");
    assert!(e.catalog().skill_by_name("shengong").is_some());
    advance_to_play(&mut e, 0);
    // 2 号装防具；1 号无闪
    let armor = equip_by_kind(&mut e, 2, CardKind::Armor);
    e.debug_clear_hand(1);
    e.debug_clear_hand(0);
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let hp1 = e.debug_hp(1);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    // 闪响应窗应含借用的防具
    let c = e.open_choice().cloned().expect("闪响应窗");
    assert_eq!(c.spec.prompt, "respond_shan");
    assert!(
        c.spec.options.contains(&ChoiceOption::Card(armor)),
        "候选含借用防具：{:?}",
        c.spec.options
    );
    let out = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: c.id,
            selection: vec![ChoiceOption::Card(armor)],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1, "借防具闪避成功");
    assert!(
        !e.state_core().player(2).equips.contains(&armor),
        "防具已从原持有者装备区取走"
    );
    assert!(e.state_core().discard.contains(&armor), "防具入弃牌堆");
}

/// 神弓（对照）：未持技者无借用候选。
#[test]
fn shengong_absent_no_borrow_option() {
    let mut e = swd_engine(124002);
    advance_to_play(&mut e, 0);
    let armor = equip_by_kind(&mut e, 2, CardKind::Armor);
    e.debug_clear_hand(1);
    e.debug_clear_hand(0);
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let hp1 = e.debug_hp(1);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let c = e.open_choice().cloned();
    if let Some(c) = &c {
        assert!(
            !c.spec.options.contains(&ChoiceOption::Card(armor)),
            "未持技不含借用候选：{:?}",
            c.spec.options
        );
    }
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1 - 1, "未持技：被杀命中");
    assert!(e.state_core().player(2).equips.contains(&armor), "防具未被取走");
}

/// 神弓（对照）：自己装备区的牌仍可正常打出（非借用路径）。
#[test]
fn shengong_own_equip_still_usable() {
    let mut e = swd_engine(124003);
    e.debug_grant(1, "shengong");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let _ = Suit::Spade;
    let hp1 = e.debug_hp(1);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1 - 1, "无借用来源：照常命中");
}
