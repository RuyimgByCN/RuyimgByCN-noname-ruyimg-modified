//! swd deferred 消解批 127：林韵（`linyun`）——两张牌当【杀】。
//!
//! 上游 `character/swd.js`：`enable:'chooseToUse'`，`selectCard:2`，
//! `position:'he'`，`viewAs:{name:'sha'}`——两张牌（手牌或装备）当杀使用；
//! 组技 `linyun2`（`shaBegin`：目标须出闪，否则直击）在原生杀结算中语义
//! 等价（目标无闪即命中；`directHit` 的防具豁免面登记差异）。

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
    for _ in 0..48 {
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

fn next_phase(e: &mut Engine) {
    let actor = e.turn().current;
    let out = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
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

/// 林韵：两张手牌当杀 → 目标无闪 → 受 1 点伤害；两张素材离手。
#[test]
fn linyun_two_hand_cards_as_sha_hits() {
    let mut e = swd_engine(104001);
    e.debug_grant(0, "linyun");
    assert!(e.catalog().skill_by_name("linyun").is_some());
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    for p in 1..4 {
        e.debug_clear_hand(p);
    }
    let c1 = e.debug_place(0, "tao");
    let c2 = e.debug_place(0, "tao");
    e.debug_give_hand_card(0, 0, c1);
    e.debug_give_hand_card(0, 0, c2);
    assert_eq!(e.debug_hand(0).len(), 2);
    let linyun = e.catalog().skill_by_name("linyun").expect("林韵");
    let hp1 = e.debug_hp(1);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: linyun },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let pick = e.open_choice().cloned().expect("林韵选两张窗");
    assert_eq!(pick.spec.prompt, "linyun_pick_two");
    let out = e.submit(GameCommand {
        actor: pick.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: pick.id,
            selection: vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)],
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let tgt = e.open_choice().cloned().expect("林韵选目标窗");
    assert_eq!(tgt.spec.prompt, "linyun_sha_target");
    let out = e.submit(GameCommand {
        actor: tgt.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: tgt.id,
            selection: vec![ChoiceOption::Player(1)],
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
    assert_eq!(e.debug_hp(1), hp1 - 1, "无闪 → 受 1 点伤害");
    assert_eq!(e.debug_hand(0).len(), 0, "两张素材离手");
}

/// 林韵（装备素材）：一张手牌 + 一件装备当杀；装备离场（装备技撤销）。
#[test]
fn linyun_equip_material_allowed() {
    let mut e = swd_engine(104002);
    e.debug_grant(0, "linyun");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    for p in 1..4 {
        e.debug_clear_hand(p);
    }
    let hand = e.debug_place(0, "tao");
    e.debug_give_hand_card(0, 0, hand);
    let equip = e.debug_place(0, "armor");
    {
        let core = e.state_core_mut();
        core.players[0].hand.retain(|c| *c != equip);
        core.players[0].equips.push(equip);
    }
    let linyun = e.catalog().skill_by_name("linyun").expect("林韵");
    let hp1 = e.debug_hp(1);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: linyun },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let pick = e.open_choice().cloned().expect("选两张窗");
    let out = e.submit(GameCommand {
        actor: pick.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: pick.id,
            selection: vec![ChoiceOption::Card(equip), ChoiceOption::Card(hand)],
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let tgt = e.open_choice().cloned().expect("选目标窗");
    let out = e.submit(GameCommand {
        actor: tgt.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: tgt.id,
            selection: vec![ChoiceOption::Player(1)],
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
    assert_eq!(e.debug_hp(1), hp1 - 1, "装备作素材照常命中");
    assert!(e.debug_equips(0).is_empty(), "装备素材离场");
    assert_eq!(e.debug_hand(0).len(), 0, "手牌素材离场");
}

/// 林韵（对照）：不足两张牌时不出现主动面（activate 返回 None）。
#[test]
fn linyun_requires_two_cards() {
    let mut e = swd_engine(104003);
    e.debug_grant(0, "linyun");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    let only = e.debug_place(0, "tao");
    e.debug_give_hand_card(0, 0, only);
    let linyun = e.catalog().skill_by_name("linyun").expect("林韵");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: linyun },
    });
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "仅一张牌不可发动：{out:?}"
    );
    assert!(e.open_choice().is_none(), "无选择窗");
}

/// 林韵（对照）：非出牌阶段不可主动发动。
#[test]
fn linyun_requires_play_phase() {
    let mut e = swd_engine(104004);
    e.debug_grant(0, "linyun");
    for _ in 0..12 {
        if e.turn().current == 0 && e.turn().phase == Phase::Draw {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    assert_eq!(e.turn().phase, Phase::Draw);
    e.debug_clear_hand(0);
    let c1 = e.debug_place(0, "tao");
    let c2 = e.debug_place(0, "tao");
    e.debug_give_hand_card(0, 0, c1);
    e.debug_give_hand_card(0, 0, c2);
    let linyun = e.catalog().skill_by_name("linyun").expect("林韵");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: linyun },
    });
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "非出牌阶段不可发动：{out:?}"
    );
}
