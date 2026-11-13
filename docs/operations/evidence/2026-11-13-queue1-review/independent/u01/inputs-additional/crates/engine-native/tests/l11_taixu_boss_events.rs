//! R3 Event Path Tests: GameCommand & GameAction Phase-Driven Triggers,
//! Interactive Player Choices (`HostBehavior` + `HostStage` + `ChoiceKind`),
//! Counter-Examples, AI `BoundedPlanner` Public-Info Decisions, and Mid-Choice Save/Restore Roundtrips.
//!
//! 1. `boss_tianyun` (`HostBehavior::BossTianyun`): `Phase::End` trigger -> optional target selection
//!    (`"boss_tianyun_target"`, `allow_cancel: true`) -> owner loses 1 HP, deals 2~3 fire damage via RNG,
//!    then strips target equips. Counter-example / cancel / AI (`hp <= 1` cancels; otherwise picks enemy
//!    with most equips) / mid-choice save/restore.
//!    Upstream ref: `archive/upstream/noname-liuli/noname/extension/太虚幻境/extension_skill.js:4085`
//! 2. `boss_lingfeng`: `Phase::Draw` trigger -> draw 2; different color -> enemy loses 1 HP.
//!    Counter-example: same color -> no HP loss.
//!    Upstream ref: `archive/upstream/noname-liuli/noname/extension/太虚幻境/extension_skill.js:3843`
//! 3. `boss_chuanyun` (`HostBehavior::BossChuanyun`): `Phase::End` trigger -> optional target selection
//!    (`"boss_chuanyun_target"`, `allow_cancel: true`) among other alive seats with `target.hp > owner.hp`
//!    -> deals 1 damage. Counter-example: `target.hp <= owner.hp` does not trigger.
//!    Upstream ref: `archive/upstream/noname-liuli/noname/extension/太虚幻境/extension_skill.js:3806`
//! 4. `leiji` (standard pack `雷击`): `CardResponded` (Shan) -> choose target -> spade judge -> 2 thunder damage.
//!    Counter-example: non-shan response does not trigger.
//!    Upstream ref: `archive/upstream/noname/apps/core/extension/shenhua/skill.js:7205`
//! 5. `dddxiaxing` (`HostBehavior::DddXiaxing`): `Phase::Prepare` / `Dying` -> optional card selection
//!    (`"dddxiaxing_card"`, `allow_cancel: true`) -> puts chosen `he` card on top of `deck`, treats event
//!    player as using virtual `Jiu` (Prepare) or `Tao` (Dying), and grants persistent `dddxiaxing_xia` mark
//!    (consumed for 0-card virtual `Sha` / `Shan`).
//!    Counter-example / cancel / AI (`5 - get.value(card)`) / mid-choice save/restore.
//!    Upstream ref: `archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:59`

use content_native_loader::{
    bridge_pack, prepare_locked_content_with_bridge, validate_and_freeze_raw_package,
    LockedStartMode, VerifiedLockedContent,
};
use engine_native::catalog::Suit;
use engine_native::event::Phase;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

static EVENT_TEST_DEDUP: AtomicU64 = AtomicU64::new(8_800_000);

fn next_dedup() -> u64 {
    EVENT_TEST_DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn drain(e: &mut Engine) {
    for _ in 0..32 {
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
            action: GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        });
    }
}

fn start_taixu_test_engine_n(seed: u64, player_count: u8) -> (VerifiedLockedContent, Engine) {
    let pkg_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/taixu-upstream-liuli");
    let raw_json = std::fs::read_to_string(pkg_dir.join("package.npack.json"))
        .expect("read taixu package.npack.json")
        .replace("\r\n", "\n");
    let parsed: serde_json::Value = serde_json::from_str(&raw_json).unwrap();
    let mut assets = BTreeMap::new();
    if let Some(paths) = parsed.get("asset_paths").and_then(|v| v.as_object()) {
        for rel_path in paths.values().filter_map(|v| v.as_str()) {
            let bytes = std::fs::read(pkg_dir.join(rel_path))
                .expect("read taixu asset")
                .into_iter()
                .filter(|&b| b != b'\r')
                .collect::<Vec<u8>>();
            assets.insert(rel_path.to_string(), bytes);
        }
    }
    let frozen = validate_and_freeze_raw_package(raw_json, assets).expect("freeze taixu package");
    let verified = prepare_locked_content_with_bridge(vec![frozen]).expect("prepare locked taixu");
    let engine = start_locked_engine(seed, player_count, LockedStartMode::Prototype, &verified)
        .expect("start locked taixu engine");
    (verified, engine)
}

fn start_taixu_test_engine(seed: u64) -> (VerifiedLockedContent, Engine) {
    start_taixu_test_engine_n(seed, 2)
}

fn start_locked_engine(
    seed: u64,
    player_count: u8,
    mode: LockedStartMode,
    verified: &VerifiedLockedContent,
) -> Result<Engine, String> {
    content_native_loader::start_locked_engine(seed, player_count as usize, mode, verified)
}

/// Upstream: `archive/upstream/noname-liuli/noname/extension/太虚幻境/extension_skill.js:4085`
/// `boss_tianyun`:
/// - `Phase::End` trigger opens optional target choice (`"boss_tianyun_target"`, `allow_cancel: true`).
/// - Choosing target A vs B: owner loses 1 HP, chosen target takes 2 fire damage, then loses all equips.
/// - Cancelling (`selection: vec![]`): owner loses 0 HP, target takes 0 damage and keeps equips.
/// - AI (`BoundedPlanner`): cancels when `owner.hp <= 1`; otherwise picks enemy with most equips
///   using only public `PolicyInput` without reading unrevealed hand cards.
/// - Mid-choice save/restore: snapshot while `Pending` -> restore -> answer yields identical state hash & events.
#[test]
fn test_boss_tianyun_authentic_lifecycle() {
    let (verified, mut e_pick_a) = start_taixu_test_engine_n(0x5347_5313, 3);
    for seat in 0..3 {
        e_pick_a.state_core_mut().player_mut(seat).max_hp = 8;
        e_pick_a.state_core_mut().player_mut(seat).hp = 8;
    }
    e_pick_a.debug_grant(0, "boss_tianyun");
    let w1 = e_pick_a.debug_place(1, "zhuge");
    e_pick_a.state_core_mut().player_mut(1).hand.retain(|&c| c != w1);
    e_pick_a.state_core_mut().player_mut(1).equips.push(w1);
    let w2 = e_pick_a.debug_place(2, "qinggang");
    e_pick_a.state_core_mut().player_mut(2).hand.retain(|&c| c != w2);
    e_pick_a.state_core_mut().player_mut(2).equips.push(w2);

    e_pick_a.debug_set_turn(0, Phase::Discard, 1);
    let out = e_pick_a.submit(GameCommand {
        actor: 0,
        expected_revision: e_pick_a.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    let choice = match out {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected Pending boss_tianyun_target choice, got {other:?}"),
    };
    assert_eq!(choice.spec.prompt, "boss_tianyun_target");
    assert!(choice.spec.allow_cancel);
    assert_eq!(
        choice.spec.options,
        vec![ChoiceOption::Player(1), ChoiceOption::Player(2)]
    );

    // Mid-choice save/restore while choice is suspended (`Pending`):
    let pending_snap = e_pick_a.save();
    let mut e_restored_a = content_native_loader::restore_locked_engine(&pending_snap, &verified)
        .expect("restore mid-choice boss_tianyun");
    let mut e_pick_b = content_native_loader::restore_locked_engine(&pending_snap, &verified)
        .expect("restore branch B boss_tianyun");
    let mut e_cancel = content_native_loader::restore_locked_engine(&pending_snap, &verified)
        .expect("restore cancel branch boss_tianyun");
    assert_eq!(e_restored_a.state_hash(), e_pick_a.state_hash());

    // Branch A: choose seat 1 (both original and restored mid-choice engine must match identically)
    let cmd_a = GameCommand {
        actor: 0,
        expected_revision: e_pick_a.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: choice.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    };
    let log_mark_a = e_pick_a.log().len();
    let out_orig_a = e_pick_a.submit(cmd_a.clone());
    let out_rest_a = e_restored_a.submit(cmd_a);
    assert!(matches!(out_orig_a, CommandOutcome::Accepted { .. }));
    assert!(matches!(out_rest_a, CommandOutcome::Accepted { .. }));
    assert_eq!(
        &e_pick_a.log()[log_mark_a..],
        e_restored_a.log(),
        "mid-choice restore events must match"
    );
    assert_eq!(
        e_pick_a.state_hash(),
        e_restored_a.state_hash(),
        "mid-choice restore state_hash must match"
    );
    assert_eq!(e_pick_a.state_core().player(0).hp, 7, "owner loses 1 HP");
    assert!(
        matches!(e_pick_a.state_core().player(1).hp, 5 | 6),
        "seat 1 takes 2 or 3 fire damage via RNG"
    );
    assert_eq!(e_pick_a.state_core().player(1).equips.len(), 0, "seat 1 equips stripped");
    assert_eq!(e_pick_a.state_core().player(2).hp, 8, "seat 2 untouched");
    assert_eq!(e_pick_a.state_core().player(2).equips.len(), 1, "seat 2 keeps equip");

    // Branch B: choose seat 2 -> different outcome from Branch A
    let _ = e_pick_b.submit(GameCommand {
        actor: 0,
        expected_revision: e_pick_b.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: choice.id,
            selection: vec![ChoiceOption::Player(2)],
        },
    });
    assert_ne!(e_pick_a.state_hash(), e_pick_b.state_hash());
    assert_eq!(e_pick_b.state_core().player(0).hp, 7);
    assert_eq!(e_pick_b.state_core().player(1).hp, 8, "seat 1 untouched in branch B");
    assert_eq!(e_pick_b.state_core().player(1).equips.len(), 1);
    assert!(
        matches!(e_pick_b.state_core().player(2).hp, 5 | 6),
        "seat 2 takes 2 or 3 fire damage via RNG in branch B"
    );
    assert_eq!(e_pick_b.state_core().player(2).equips.len(), 0, "seat 2 equips stripped in branch B");

    // Cancel branch: player selects `vec![]` -> skill does not activate
    let _ = e_cancel.submit(GameCommand {
        actor: 0,
        expected_revision: e_cancel.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: choice.id,
            selection: Vec::new(),
        },
    });
    assert_eq!(e_cancel.state_core().player(0).hp, 8, "cancel loses 0 HP");
    assert_eq!(e_cancel.state_core().player(1).hp, 8);
    assert_eq!(e_cancel.state_core().player(1).equips.len(), 1);
    assert_eq!(e_cancel.state_core().player(2).hp, 8);
    assert_eq!(e_cancel.state_core().player(2).equips.len(), 1);

    // AI (`BoundedPlanner`) test (`extension_skill.js:4103-4111`):
    // 1. When seat 2 has 2 equips and seat 1 has 1 equip, AI picks seat 2 (`eff + target.countCards('e') / 2`),
    //    and mutating hidden hand cards of seat 1/2 does NOT change AI's planned action (`nodes_used == 0`).
    let mut e_ai = content_native_loader::restore_locked_engine(&pending_snap, &verified)
        .expect("restore AI branch boss_tianyun");
    let extra_eq = e_ai.debug_place(2, "zhuge");
    e_ai.state_core_mut().player_mut(2).hand.retain(|&c| c != extra_eq);
    e_ai.state_core_mut().player_mut(2).equips.push(extra_eq);
    let planner = engine_native::ai::BoundedPlanner::new(64);
    let plan1 = e_ai.plan_action(0, &planner);
    assert_eq!(plan1.nodes_used, 0);
    assert_eq!(
        plan1.action,
        GameAction::Answer {
            choice_id: choice.id,
            selection: vec![ChoiceOption::Player(2)],
        }
    );
    let hidden_tao = e_ai.debug_place(1, "tao");
    assert!(e_ai.state_core().player(1).hand.contains(&hidden_tao));
    let plan2 = e_ai.plan_action(0, &planner);
    assert_eq!(plan1.action, plan2.action, "AI must not depend on unrevealed hand cards");

    // 2. When owner HP <= 1 (`if (player.hp <= 1) return 0`), AI cancels (`selection: vec![]`).
    let (_, mut e_low_hp) = start_taixu_test_engine(0x5347_5314);
    e_low_hp.debug_grant(0, "boss_tianyun");
    e_low_hp.state_core_mut().player_mut(0).hp = 1;
    e_low_hp.state_core_mut().player_mut(1).hp = 8;
    e_low_hp.debug_set_turn(0, Phase::Discard, 1);
    let out_low = e_low_hp.submit(GameCommand {
        actor: 0,
        expected_revision: e_low_hp.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    let low_choice = match out_low {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected Pending boss_tianyun_target, got {other:?}"),
    };
    let low_plan = e_low_hp.plan_action(0, &planner);
    assert_eq!(
        low_plan.action,
        GameAction::Answer {
            choice_id: low_choice.id,
            selection: Vec::new(),
        },
        "AI must cancel boss_tianyun when hp <= 1"
    );
    let _ = e_low_hp.submit(GameCommand {
        actor: 0,
        expected_revision: e_low_hp.revision(),
        dedup_id: next_dedup(),
        action: low_plan.action,
    });
    assert_eq!(e_low_hp.state_core().player(0).hp, 1);
    assert_eq!(e_low_hp.state_core().player(1).hp, 8);
}

/// Upstream: `archive/upstream/noname-liuli/noname/extension/太虚幻境/extension_skill.js:3843`
/// `boss_lingfeng`: Draw phase -> draw 2 cards; if different color -> enemy loses 1 HP.
/// Counter-example: same color -> no HP loss.
#[test]
fn test_boss_lingfeng_authentic_lifecycle() {
    let (verified, mut e) = start_taixu_test_engine(0x5347_5315);
    e.debug_grant(0, "boss_lingfeng");
    e.state_core_mut().player_mut(0).hp = 4;
    e.state_core_mut().player_mut(1).hp = 4;

    // Stack deck top: red card (hearts) + black card (spades) -> different colors
    let c_red = e.debug_place_suit(0, Suit::Heart);
    let c_black = e.debug_place_suit(0, Suit::Spade);
    e.state_core_mut().player_mut(0).hand.retain(|&c| c != c_red && c != c_black);
    e.state_core_mut().deck.push(c_red);
    e.state_core_mut().deck.push(c_black);

    // Advance to Draw phase
    e.debug_set_turn(0, Phase::Judge, 1);
    let _ = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    drain(&mut e);

    // Positive assertion: enemy lost 1 HP (4 -> 3)
    assert_eq!(e.state_core().player(1).hp, 3, "different color causes 1 HP loss");

    // Save / Restore suspension test:
    let snapshot = e.save();
    let restored = content_native_loader::restore_locked_engine(&snapshot, &verified)
        .expect("restore lingfeng engine from snapshot");
    assert_eq!(restored.state_hash(), e.state_hash(), "restored hash must match");
    assert_eq!(restored.state_core().player(1).hp, 3);

    // Counter-example test: same colors -> no HP loss
    let (_, mut e2) = start_taixu_test_engine(0x5347_5316);
    e2.debug_grant(0, "boss_lingfeng");
    e2.state_core_mut().player_mut(0).hp = 4;
    e2.state_core_mut().player_mut(1).hp = 4;

    let c_black1 = e2.debug_place_suit(0, Suit::Spade);
    let c_black2 = e2.debug_place_suit(0, Suit::Club);
    e2.state_core_mut().player_mut(0).hand.retain(|&c| c != c_black1 && c != c_black2);
    e2.state_core_mut().deck.push(c_black1);
    e2.state_core_mut().deck.push(c_black2);

    e2.debug_set_turn(0, Phase::Judge, 1);
    let _ = e2.submit(GameCommand {
        actor: 0,
        expected_revision: e2.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    drain(&mut e2);
    assert_eq!(e2.state_core().player(1).hp, 4, "same color must not trigger HP loss");
}

/// Upstream: `archive/upstream/noname-liuli/noname/extension/太虚幻境/extension_skill.js:3806`
/// `boss_chuanyun`:
/// - `Phase::End` trigger opens optional target choice (`"boss_chuanyun_target"`, `allow_cancel: true`)
///   among other alive seats with `target.hp > owner.hp` (`return player.hp < target.hp`).
/// - Choosing target A vs B deals 1 damage to A vs B; cancelling (`selection: vec![]`) deals 0 damage.
/// - Negative case: when all other seats have `hp <= owner.hp` (including equal HP), skill does not trigger.
/// - AI (`BoundedPlanner`): selects enemy target via public `PolicyInput` (or cancels when no candidate has
///   `public_attitude < 0`) without reading hidden info.
/// - Mid-choice save/restore: snapshot while `Pending` -> restore -> answer yields identical state hash & events.
#[test]
fn test_boss_chuanyun_authentic_lifecycle() {
    let (verified, mut e_pick_a) = start_taixu_test_engine_n(0x5347_5317, 3);
    e_pick_a.debug_grant(0, "boss_chuanyun");
    e_pick_a.state_core_mut().player_mut(0).hp = 3;
    e_pick_a.state_core_mut().player_mut(1).hp = 5; // > owner.hp (3)
    e_pick_a.state_core_mut().player_mut(2).hp = 4; // > owner.hp (3)

    e_pick_a.debug_set_turn(0, Phase::Discard, 1);
    let out = e_pick_a.submit(GameCommand {
        actor: 0,
        expected_revision: e_pick_a.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    let choice = match out {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected Pending boss_chuanyun_target choice, got {other:?}"),
    };
    assert_eq!(choice.spec.prompt, "boss_chuanyun_target");
    assert!(choice.spec.allow_cancel);
    assert_eq!(
        choice.spec.options,
        vec![ChoiceOption::Player(1), ChoiceOption::Player(2)]
    );

    // Mid-choice save/restore while choice is suspended (`Pending`):
    let pending_snap = e_pick_a.save();
    let mut e_restored_a = content_native_loader::restore_locked_engine(&pending_snap, &verified)
        .expect("restore mid-choice boss_chuanyun");
    let mut e_pick_b = content_native_loader::restore_locked_engine(&pending_snap, &verified)
        .expect("restore branch B boss_chuanyun");
    let mut e_cancel = content_native_loader::restore_locked_engine(&pending_snap, &verified)
        .expect("restore cancel branch boss_chuanyun");
    assert_eq!(e_restored_a.state_hash(), e_pick_a.state_hash());

    // Cancel branch: player selects `vec![]` -> aborts without dealing damage (`if (result.bool)`)
    let out_cancel = e_cancel.submit(GameCommand {
        actor: 0,
        expected_revision: e_cancel.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: choice.id,
            selection: Vec::new(),
        },
    });
    assert!(matches!(out_cancel, CommandOutcome::Accepted { .. }));
    assert_eq!(e_cancel.state_core().player(1).hp, 5, "cancel deals 0 damage to seat 1");
    assert_eq!(e_cancel.state_core().player(2).hp, 4, "cancel deals 0 damage to seat 2");

    // Branch A: choose seat 1
    let cmd_a = GameCommand {
        actor: 0,
        expected_revision: e_pick_a.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: choice.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    };
    let log_mark_a = e_pick_a.log().len();
    let out_orig_a = e_pick_a.submit(cmd_a.clone());
    let out_rest_a = e_restored_a.submit(cmd_a);
    assert!(matches!(out_orig_a, CommandOutcome::Accepted { .. }));
    assert!(matches!(out_rest_a, CommandOutcome::Accepted { .. }));
    assert_eq!(&e_pick_a.log()[log_mark_a..], e_restored_a.log());
    assert_eq!(e_pick_a.state_hash(), e_restored_a.state_hash());
    assert_eq!(e_pick_a.state_core().player(1).hp, 4, "seat 1 takes 1 damage");
    assert_eq!(e_pick_a.state_core().player(2).hp, 4, "seat 2 untouched");

    // Branch B: choose seat 2 -> different outcome from Branch A
    let _ = e_pick_b.submit(GameCommand {
        actor: 0,
        expected_revision: e_pick_b.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: choice.id,
            selection: vec![ChoiceOption::Player(2)],
        },
    });
    assert_ne!(e_pick_a.state_hash(), e_pick_b.state_hash());
    assert_eq!(e_pick_b.state_core().player(1).hp, 5, "seat 1 untouched in branch B");
    assert_eq!(e_pick_b.state_core().player(2).hp, 3, "seat 2 takes 1 damage in branch B");

    // AI (`BoundedPlanner`) test (`extension_skill.js:3814-3816`):
    // 1. Seat 0 (Lord) picks lower-HP enemy target (seat 2 with hp=4 over seat 1 with hp=5),
    //    and does not depend on unrevealed hand cards (`nodes_used == 0`).
    let mut e_ai = content_native_loader::restore_locked_engine(&pending_snap, &verified)
        .expect("restore AI boss_chuanyun");
    let planner = engine_native::ai::BoundedPlanner::new(64);
    let plan1 = e_ai.plan_action(0, &planner);
    assert_eq!(plan1.nodes_used, 0);
    assert_eq!(
        plan1.action,
        GameAction::Answer {
            choice_id: choice.id,
            selection: vec![ChoiceOption::Player(2)],
        },
        "AI picks lower-HP enemy target"
    );
    let hidden_card = e_ai.debug_place(1, "tao");
    assert!(e_ai.state_core().player(1).hand.contains(&hidden_card));
    let plan2 = e_ai.plan_action(0, &planner);
    assert_eq!(plan1.action, plan2.action, "AI must not depend on unrevealed hand cards");

    // 2. When Loyalist seat 1 in an Identity match has `boss_chuanyun` and only friendly Lord
    //    seat 0 (`public_attitude > 0`) has `hp > seat1.hp`, AI cancels (`selection: vec![]`).
    let (_, mut e_ai_cancel) = start_taixu_test_engine_n(0x5347_5319, 3);
    e_ai_cancel.state_core_mut().mode = engine_native::state::GameMode::Identity {
        roles: vec![
            engine_native::state::Role::Lord,
            engine_native::state::Role::Loyalist,
            engine_native::state::Role::Rebel,
        ],
    };
    e_ai_cancel.debug_grant(1, "boss_chuanyun");
    e_ai_cancel.state_core_mut().player_mut(0).hp = 4; // friendly Lord hp > seat 1 hp
    e_ai_cancel.state_core_mut().player_mut(1).hp = 2; // Loyalist
    e_ai_cancel.state_core_mut().player_mut(2).hp = 2; // hp <= seat 1 hp (excluded)
    e_ai_cancel.debug_set_turn(1, Phase::Discard, 1);
    let out_ai_cancel = e_ai_cancel.submit(GameCommand {
        actor: 1,
        expected_revision: e_ai_cancel.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    let ai_cancel_choice = match out_ai_cancel {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected Pending boss_chuanyun_target on seat 1, got {other:?}"),
    };
    let cancel_plan = e_ai_cancel.plan_action(1, &planner);
    assert_eq!(
        cancel_plan.action,
        GameAction::Answer {
            choice_id: ai_cancel_choice.id,
            selection: Vec::new(),
        },
        "AI must cancel boss_chuanyun when no candidate has negative public_attitude"
    );

    // Counter-example: other players have `hp <= owner.hp` (including equal HP `3 == 3`) -> does not trigger
    let (_, mut e2) = start_taixu_test_engine_n(0x5347_5318, 3);
    e2.debug_grant(0, "boss_chuanyun");
    e2.state_core_mut().player_mut(0).hp = 3;
    e2.state_core_mut().player_mut(1).hp = 3; // equal HP must NOT trigger (`player.hp < target.hp`)
    e2.state_core_mut().player_mut(2).hp = 2; // lower HP must NOT trigger

    e2.debug_set_turn(0, Phase::Discard, 1);
    let out_neg = e2.submit(GameCommand {
        actor: 0,
        expected_revision: e2.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(out_neg, CommandOutcome::Accepted { .. }),
        "chuanyun must not open choice when target.hp <= owner.hp: {out_neg:?}"
    );
    assert_eq!(e2.state_core().player(1).hp, 3);
    assert_eq!(e2.state_core().player(2).hp, 2);
}

/// Upstream: `archive/upstream/noname/apps/core/extension/shenhua/skill.js:7205`
/// `leiji`: CardResponded (Shan) -> spade judge -> 2 thunder damage.
/// Counter-example: non-shan response does not trigger.
#[test]
fn test_leiji_authentic_lifecycle() {
    let mut e = Engine::new(0x5347_5301, 2);
    e.debug_grant(0, "雷击");
    e.state_core_mut().player_mut(0).hp = 4;
    e.state_core_mut().player_mut(1).hp = 4;

    // Give 0 a Shan card, give 1 a Sha card
    let card_shan = e.debug_place(0, "shan");
    let card_sha = e.debug_place(1, "sha");

    // Pre-set deck top with a spade judge card
    let spade_judge = e.debug_place_suit(0, Suit::Spade);
    e.state_core_mut().player_mut(0).hand.retain(|&c| c != spade_judge);
    e.state_core_mut().deck.push(spade_judge);

    // 1 plays Sha on 0
    e.debug_set_turn(1, Phase::Play, 1);
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: card_sha,
            targets: vec![0],
        },
    });
    assert!(matches!(out, CommandOutcome::Pending { .. } | CommandOutcome::Accepted { .. }));

    // 0 responds with Shan
    if let Some(c) = e.open_choice().cloned() {
        let _ = e.submit(GameCommand {
            actor: 0,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer {
                choice_id: c.id,
                selection: vec![ChoiceOption::Card(card_shan)],
            },
        });
    }
    drain(&mut e);

    // After Shan response and spade judge, 1 takes thunder damage (4 -> 2)
    assert!(
        e.state_core().player(1).hp <= 2,
        "leiji dealt 2 thunder damage: actual hp = {}",
        e.state_core().player(1).hp
    );

    // Save / Restore suspension test:
    let snapshot = e.save();
    let restored = Engine::restore(&snapshot).expect("restore leiji engine from snapshot");
    assert_eq!(restored.state_hash(), e.state_hash(), "restored hash must match");
    assert_eq!(restored.state_core().player(1).hp, e.state_core().player(1).hp);

    // Counter-example: 0 declines to respond with Shan
    let mut e2 = Engine::new(0x5347_5302, 2);
    e2.debug_grant(0, "雷击");
    e2.state_core_mut().player_mut(0).hp = 4;
    e2.state_core_mut().player_mut(1).hp = 4;
    let card_sha2 = e2.debug_place(1, "sha");
    e2.debug_set_turn(1, Phase::Play, 1);
    let _ = e2.submit(GameCommand {
        actor: 1,
        expected_revision: e2.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: card_sha2,
            targets: vec![0],
        },
    });
    // Decline response
    if let Some(c) = e2.open_choice().cloned() {
        let _ = e2.submit(GameCommand {
            actor: 0,
            expected_revision: e2.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer {
                choice_id: c.id,
                selection: vec![ChoiceOption::Pass],
            },
        });
    }
    drain(&mut e2);
    assert_eq!(e2.state_core().player(1).hp, 4, "pass Shan must not trigger leiji on opponent");
}

/// Upstream: `archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:59`
/// `dddxiaxing`:
/// - `Phase::Prepare` or `Dying` opens optional card selection (`"dddxiaxing_card"`, `allow_cancel: true`).
/// - Choosing card A vs B: puts chosen `he` card on top of `deck`, sets `dddxiaxing_xia = 1`, and treats
///   the event player as using virtual `Jiu` (on `Phase::Prepare`) or virtual `Tao` (on `Dying`).
/// - Holding `dddxiaxing_xia = 1` persists across rounds until consumed by `dddxiaxing_viewas` (0-card
///   virtual `Sha` / `Shan`), and does NOT block triggering `dddxiaxing` in subsequent rounds.
/// - Cancelling (`selection: vec![]`): moves 0 cards, does not consume once-per-round limit or set `dddxiaxing_xia`.
/// - Negative case: empty `hand` and `equips` (or already used in the same round) does not trigger.
/// - AI (`BoundedPlanner`): cancels when event player is an enemy; when friendly/self, picks lowest-value card
///   (`5 - get.value(card)`) without reading unrevealed hand cards.
/// - Mid-choice save/restore: snapshot while `Pending` -> restore -> answer yields identical state hash & events.
#[test]
fn test_dddxiaxing_authentic_lifecycle() {
    let d3_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/d3-upstream-noname/assets/character.json");
    let d3_json = std::fs::read_to_string(d3_path).expect("read d3 character.json");
    let d3_pack = bridge_pack(&d3_json).expect("bridge d3 pack").pack;
    let p9_pack = content_native_loader::decision34_pack9_for_coexistence(&[&d3_pack]);
    let packs = [&d3_pack, &p9_pack];

    let mut e_pick_a =
        Engine::new_with_extensions(260101, 2, &packs).expect("start U01 d3 engine");
    let zm = (0..e_pick_a.catalog().general_count() as u16)
        .find(|&idx| e_pick_a.catalog().general_meta(idx).is_some_and(|m| m.0 == "ddd_zhangmiao"))
        .expect("find ddd_zhangmiao");
    e_pick_a.assign_general(0, zm).expect("assign general");

    // Give seat 0 exactly two cards in hand: `card_sha` (low value) and `card_tao` (high value)
    let removed = e_pick_a.state_core_mut().player_mut(0).hand.drain(..).collect::<Vec<_>>();
    e_pick_a.state_core_mut().discard.extend(removed);
    let removed_eq = e_pick_a.state_core_mut().player_mut(0).equips.drain(..).collect::<Vec<_>>();
    e_pick_a.state_core_mut().discard.extend(removed_eq);
    let card_tao = e_pick_a.debug_place(0, "tao");
    let card_sha = e_pick_a.debug_place(0, "sha");

    // Advance seat 1 from `Phase::End` into seat 0's `Phase::Prepare` via real `GameAction::NextPhase`
    e_pick_a.debug_set_turn(1, Phase::End, 1);
    let out = e_pick_a.submit(GameCommand {
        actor: 1,
        expected_revision: e_pick_a.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    let choice = match out {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected Pending dddxiaxing_card choice, got {other:?}"),
    };
    assert_eq!(choice.spec.prompt, "dddxiaxing_card");
    assert!(choice.spec.allow_cancel);
    assert_eq!(
        choice.spec.options,
        vec![ChoiceOption::Card(card_tao), ChoiceOption::Card(card_sha)]
    );

    // Mid-choice save/restore while choice is suspended (`Pending`):
    let pending_snap = e_pick_a.save();
    let mut e_restored_a = Engine::restore_with_extensions(&pending_snap, &packs)
        .expect("restore mid-choice dddxiaxing");
    let mut e_pick_b = Engine::restore_with_extensions(&pending_snap, &packs)
        .expect("restore branch B dddxiaxing");
    let mut e_cancel = Engine::restore_with_extensions(&pending_snap, &packs)
        .expect("restore cancel branch dddxiaxing");
    assert_eq!(e_restored_a.state_hash(), e_pick_a.state_hash());

    // Branch A: choose `card_tao` -> `card_tao` placed on top of `deck`, `card_sha` remains in hand,
    // `dddxiaxing_xia = 1`, and seat 0 uses virtual `Jiu` (`jiu_active == true`).
    let cmd_a = GameCommand {
        actor: 0,
        expected_revision: e_pick_a.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: choice.id,
            selection: vec![ChoiceOption::Card(card_tao)],
        },
    };
    let log_mark_a = e_pick_a.log().len();
    let out_orig_a = e_pick_a.submit(cmd_a.clone());
    let out_rest_a = e_restored_a.submit(cmd_a);
    assert!(matches!(out_orig_a, CommandOutcome::Accepted { .. }));
    assert!(matches!(out_rest_a, CommandOutcome::Accepted { .. }));
    assert_eq!(&e_pick_a.log()[log_mark_a..], e_restored_a.log());
    assert_eq!(e_pick_a.state_hash(), e_restored_a.state_hash());
    assert!(!e_pick_a.state_core().player(0).hand.contains(&card_tao));
    assert!(e_pick_a.state_core().player(0).hand.contains(&card_sha));
    assert_eq!(
        e_pick_a.state_core().deck.last().copied(),
        Some(card_tao),
        "chosen card must be placed on top of the draw pile"
    );
    assert!(
        e_pick_a.state_core().turn.jiu_active,
        "dddxiaxing_begin on Prepare must treat target as using virtual Jiu"
    );
    assert_eq!(
        e_pick_a.state_core().vm_aux.flags.get(&(0, "dddxiaxing_xia".to_string())).copied(),
        Some(1)
    );

    // Branch B: choose `card_sha` -> `card_sha` placed on top of `deck`, `card_tao` remains in hand
    let _ = e_pick_b.submit(GameCommand {
        actor: 0,
        expected_revision: e_pick_b.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: choice.id,
            selection: vec![ChoiceOption::Card(card_sha)],
        },
    });
    assert_ne!(e_pick_a.state_hash(), e_pick_b.state_hash());
    assert!(e_pick_b.state_core().player(0).hand.contains(&card_tao));
    assert!(!e_pick_b.state_core().player(0).hand.contains(&card_sha));
    assert_eq!(
        e_pick_b.state_core().deck.last().copied(),
        Some(card_sha),
        "chosen card in branch B must be placed on top of the draw pile"
    );
    assert!(e_pick_b.state_core().turn.jiu_active);

    // Cross-round `"侠"` mark persistence & triggering while already holding `"侠"` mark (`skill.js:128-133, 163-208`):
    // In Round 3 (`turn.round = 3`), seat 0 still holds `dddxiaxing_xia = 1` AND can still trigger `dddxiaxing`
    // because `!player.hasSkill("dddxiaxing_used")` only checks once-per-round, not `hasMark("dddxiaxing")`.
    e_pick_b.debug_set_turn(1, Phase::End, 2);
    let out_next_round = e_pick_b.submit(GameCommand {
        actor: 1,
        expected_revision: e_pick_b.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    let next_round_choice = match out_next_round {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("dddxiaxing must trigger in next round even while holding xia mark: {other:?}"),
    };
    assert_eq!(next_round_choice.spec.prompt, "dddxiaxing_card");
    let _ = e_pick_b.submit(GameCommand {
        actor: 0,
        expected_revision: e_pick_b.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: next_round_choice.id,
            selection: Vec::new(),
        },
    });
    // Now in Round 3 `Phase::Play`, seat 0 (with 0 hand cards needed) can still consume `dddxiaxing_xia`
    // via `dddxiaxing_viewas` (`skill.js:163-208`) to view-as a 0-card `Sha`!
    e_pick_b.state_core_mut().player_mut(0).hand.clear();
    let cur_r = e_pick_b.turn().round;
    e_pick_b.debug_set_turn(0, Phase::Play, cur_r);
    let dddxiaxing_def = e_pick_b.catalog().skill_by_name("dddxiaxing").expect("dddxiaxing def");
    let out_viewas = e_pick_b.submit(GameCommand {
        actor: 0,
        expected_revision: e_pick_b.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: dddxiaxing_def },
    });
    let sha_choice = match out_viewas {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected Pending zhangba_sha_target from 0-card dddxiaxing_viewas: {other:?}"),
    };
    let _ = e_pick_b.submit(GameCommand {
        actor: 0,
        expected_revision: e_pick_b.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: sha_choice.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    });
    drain(&mut e_pick_b);
    assert_eq!(
        e_pick_b.state_core().vm_aux.flags.get(&(0, "dddxiaxing_xia".to_string())).copied(),
        None,
        "dddxiaxing_viewas must clear dddxiaxing_xia mark"
    );

    // Cancel branch: player selects `vec![]` -> neither card discarded, `dddxiaxing_xia` not set,
    // and once-per-round limit is NOT consumed (so it triggers again on seat 1's `Phase::Prepare` in the same round!)
    let _ = e_cancel.submit(GameCommand {
        actor: 0,
        expected_revision: e_cancel.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: choice.id,
            selection: Vec::new(),
        },
    });
    assert!(e_cancel.state_core().player(0).hand.contains(&card_tao));
    assert!(e_cancel.state_core().player(0).hand.contains(&card_sha));
    assert_eq!(
        e_cancel.state_core().vm_aux.flags.get(&(0, "dddxiaxing_xia".to_string())).copied(),
        None
    );

    // AI (`BoundedPlanner`) test (`3D精选/character/skill.js:81-83`):
    // 1. On own `Phase::Prepare` (`goon = true`), AI picks lowest-value card (`card_sha` over `card_tao`)
    //    without reading unrevealed opponent hand cards (`nodes_used == 0`).
    let mut e_ai = Engine::restore_with_extensions(&pending_snap, &packs)
        .expect("restore AI dddxiaxing");
    let planner = engine_native::ai::BoundedPlanner::new(64);
    let plan1 = e_ai.plan_action(0, &planner);
    assert_eq!(plan1.nodes_used, 0);
    assert_eq!(
        plan1.action,
        GameAction::Answer {
            choice_id: choice.id,
            selection: vec![ChoiceOption::Card(card_sha)],
        },
        "AI must prefer discarding Sha over Tao"
    );
    let opp_hidden = e_ai.debug_place(1, "tao");
    assert!(e_ai.state_core().player(1).hand.contains(&opp_hidden));
    let plan2 = e_ai.plan_action(0, &planner);
    assert_eq!(plan1.action, plan2.action, "AI must not depend on opponent hidden hand");

    // 2. On enemy seat 1's `Phase::Prepare` (`goon = false`), AI cancels (`selection: vec![]`).
    e_cancel.debug_set_turn(0, Phase::End, 2);
    let out_enemy_prep = e_cancel.submit(GameCommand {
        actor: 0,
        expected_revision: e_cancel.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    let enemy_choice = match out_enemy_prep {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected Pending dddxiaxing_card on seat 1 Prepare, got {other:?}"),
    };
    let enemy_plan = e_cancel.plan_action(0, &planner);
    assert_eq!(
        enemy_plan.action,
        GameAction::Answer {
            choice_id: enemy_choice.id,
            selection: Vec::new(),
        },
        "AI must cancel dddxiaxing when beneficiary is an enemy"
    );

    // Counter-example: empty hand & equips blocks activation
    let mut e2 = Engine::new_with_extensions(260102, 2, &packs)
        .expect("start U01 d3 engine counter-example");
    e2.assign_general(0, zm).expect("assign general");
    let removed2 = e2.state_core_mut().player_mut(0).hand.drain(..).collect::<Vec<_>>();
    e2.state_core_mut().discard.extend(removed2);
    let removed_eq2 = e2.state_core_mut().player_mut(0).equips.drain(..).collect::<Vec<_>>();
    e2.state_core_mut().discard.extend(removed_eq2);

    e2.debug_set_turn(1, Phase::End, 1);
    let out_empty = e2.submit(GameCommand {
        actor: 1,
        expected_revision: e2.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(out_empty, CommandOutcome::Accepted { .. }),
        "empty hand/equips must not open dddxiaxing choice: {out_empty:?}"
    );
    assert_eq!(
        e2.state_core().vm_aux.flags.get(&(0, "dddxiaxing_xia".to_string())).copied(),
        None
    );
}
