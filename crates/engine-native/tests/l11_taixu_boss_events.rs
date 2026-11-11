//! R3 Event Path Tests: GameCommand & GameAction Phase-Driven Triggers,
//! Counter-Examples, and Save/Restore Snapshot Roundtrips.
//!
//! Note: The 4 extension skills below (`boss_tianyun`, `boss_lingfeng`, `boss_chuanyun`,
//! `dddxiaxing`) currently resolve via deterministic fallback without opening interactive
//! player choices (`ChoiceKind`), and `test_leiji_authentic_lifecycle` exercises the
//! standard-pack `雷击` (`HostBehavior`) rather than the L11 VM binding.
//! 1. `boss_tianyun`: `Phase::End` trigger -> owner loses 1 HP, strips target equips,
//!    deals random 2~3 fire damage. Counter-example: hp <= 1 does not trigger.
//!    Save/restore: snapshot roundtrip continuity.
//!    Upstream ref: `archive/upstream/noname-liuli/noname/extension/太虚幻境/extension_skill.js:4085`
//! 2. `boss_lingfeng`: `Phase::Draw` trigger -> draw 2; different color -> enemy loses 1 HP.
//!    Counter-example: same color -> no HP loss.
//!    Upstream ref: `archive/upstream/noname-liuli/noname/extension/太虚幻境/extension_skill.js:3843`
//! 3. `boss_chuanyun`: `Phase::End` trigger -> target.hp > player.hp deals 1 damage.
//!    Counter-example: target.hp <= player.hp does not trigger.
//!    Upstream ref: `archive/upstream/noname-liuli/noname/extension/太虚幻境/extension_skill.js:3806`
//! 4. `leiji` (standard pack `雷击`): `CardResponded` (Shan) -> choose target -> spade judge -> 2 thunder damage.
//!    Counter-example: non-shan response does not trigger.
//!    Upstream ref: `archive/upstream/noname/apps/core/extension/shenhua/skill.js:7205`
//! 5. `dddxiaxing`: `Phase::Prepare` / `Dying` -> put card to deck top -> viewas tao/jiu.
//!    Counter-example: empty hand and equips blocks activation.
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

fn start_taixu_test_engine(seed: u64) -> (VerifiedLockedContent, Engine) {
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
    let engine = start_locked_engine(seed, 2, LockedStartMode::Prototype, &verified)
        .expect("start locked taixu engine");
    (verified, engine)
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
/// `boss_tianyun`: End phase -> owner loses 1 HP, target loses all equips, takes 2~3 fire damage.
/// Counter-example: owner HP <= 1 -> does not trigger.
/// Suspension & Resumption: save/restore continuity during game state.
#[test]
fn test_boss_tianyun_authentic_lifecycle() {
    let (_verified, mut e) = start_taixu_test_engine(0x5347_5313);

    // Grant boss_tianyun to player 0
    e.debug_grant(0, "boss_tianyun");
    e.state_core_mut().player_mut(0).max_hp = 8;
    e.state_core_mut().player_mut(0).hp = 8;
    e.state_core_mut().player_mut(1).max_hp = 8;
    e.state_core_mut().player_mut(1).hp = 8;

    // Equip player 1 with a weapon and armor
    let w_card = e.debug_place(1, "zhuge");
    e.state_core_mut().player_mut(1).hand.retain(|&c| c != w_card);
    e.state_core_mut().player_mut(1).equips.push(w_card);
    assert_eq!(e.state_core().player(1).equips.len(), 1, "P1 has 1 equip");

    // Positive test: Advance player 0 to Phase::End via GameAction::NextPhase
    e.debug_set_turn(0, Phase::Discard, 1);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "NextPhase to End must succeed: {out:?}"
    );
    drain(&mut e);

    // Verify consequences:
    // 1. Owner loses 1 HP (8 -> 7)
    assert_eq!(e.state_core().player(0).hp, 7, "owner loses 1 HP");
    // 2. Target loses all equips
    assert_eq!(e.state_core().player(1).equips.len(), 0, "target equips stripped");
    // 3. Target takes 2~3 fire damage (8 -> 6 or 5)
    let p1_hp = e.state_core().player(1).hp;
    assert!(
        p1_hp == 6 || p1_hp == 5,
        "target takes 2~3 fire damage: actual HP = {p1_hp}"
    );

    // Save / Restore suspension test:
    let snapshot = e.save();
    let restored = content_native_loader::restore_locked_engine(&snapshot, &_verified)
        .expect("restore taixu engine from snapshot");
    assert_eq!(restored.state_hash(), e.state_hash(), "restored hash must match");
    assert_eq!(restored.state_core().player(0).hp, 7);
    assert_eq!(restored.state_core().player(1).hp, p1_hp);

    // Counter-example test: owner HP <= 1 does not self-immolate
    let (_, mut e2) = start_taixu_test_engine(0x5347_5314);
    e2.debug_grant(0, "boss_tianyun");
    e2.state_core_mut().player_mut(0).hp = 1;
    e2.state_core_mut().player_mut(1).hp = 8;
    e2.debug_set_turn(0, Phase::Discard, 1);
    let _ = e2.submit(GameCommand {
        actor: 0,
        expected_revision: e2.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    drain(&mut e2);
    assert_eq!(e2.state_core().player(0).hp, 1, "hp <= 1 must not trigger tianyun");
    assert_eq!(e2.state_core().player(1).hp, 8, "target takes no damage");
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
/// `boss_chuanyun`: End phase -> target.hp > owner.hp deals 1 damage.
/// Counter-example: target.hp <= owner.hp does not trigger.
#[test]
fn test_boss_chuanyun_authentic_lifecycle() {
    let (verified, mut e) = start_taixu_test_engine(0x5347_5317);
    e.debug_grant(0, "boss_chuanyun");
    e.state_core_mut().player_mut(0).hp = 3;
    e.state_core_mut().player_mut(1).hp = 5; // target.hp (5) > owner.hp (3)

    e.debug_set_turn(0, Phase::Discard, 1);
    let _ = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    drain(&mut e);

    assert_eq!(e.state_core().player(1).hp, 4, "chuanyun deals 1 damage when target hp > owner hp");

    // Save / Restore suspension test:
    let snapshot = e.save();
    let restored = content_native_loader::restore_locked_engine(&snapshot, &verified)
        .expect("restore chuanyun engine from snapshot");
    assert_eq!(restored.state_hash(), e.state_hash(), "restored hash must match");
    assert_eq!(restored.state_core().player(1).hp, 4);

    // Counter-example: target.hp (3) <= owner.hp (5)
    let (_, mut e2) = start_taixu_test_engine(0x5347_5318);
    e2.debug_grant(0, "boss_chuanyun");
    e2.state_core_mut().player_mut(0).hp = 5;
    e2.state_core_mut().player_mut(1).hp = 3;

    e2.debug_set_turn(0, Phase::Discard, 1);
    let _ = e2.submit(GameCommand {
        actor: 0,
        expected_revision: e2.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    drain(&mut e2);

    assert_eq!(e2.state_core().player(1).hp, 3, "chuanyun must not trigger when target hp <= owner hp");
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
/// `dddxiaxing`: Prepare phase -> puts card to deck top -> viewas tao/jiu.
/// Counter-example: empty hand and equips blocks activation.
#[test]
fn test_dddxiaxing_authentic_lifecycle() {
    let d3_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/d3-upstream-noname/assets/character.json");
    let d3_json = std::fs::read_to_string(d3_path).expect("read d3 character.json");
    let d3_pack = bridge_pack(&d3_json).expect("bridge d3 pack").pack;
    let p9_pack = content_native_loader::decision34_pack9_for_coexistence(&[&d3_pack]);

    let mut e = Engine::new_with_extensions(260101, 2, &[&d3_pack, &p9_pack])
        .expect("start U01 d3 engine");

    // Assign ddd_zhangmiao (owner of dddxiaxing) to seat 0
    let zm = (0..e.catalog().general_count() as u16)
        .find(|&idx| e.catalog().general_meta(idx).is_some_and(|m| m.0 == "ddd_zhangmiao"))
        .expect("find ddd_zhangmiao");
    e.assign_general(0, zm).expect("assign general");

    // Positive authentic lifecycle test: Seat 0 has a card in hand
    let removed = e.state_core_mut().player_mut(0).hand.drain(..).collect::<Vec<_>>();
    e.state_core_mut().discard.extend(removed);
    let removed_eq = e.state_core_mut().player_mut(0).equips.drain(..).collect::<Vec<_>>();
    e.state_core_mut().discard.extend(removed_eq);
    let card_shan = e.debug_place(0, "shan");
    assert!(e.state_core().player(0).hand.contains(&card_shan));

    // Advance to Judge phase via GameAction::NextPhase
    e.debug_set_turn(0, Phase::Prepare, 1);
    let _ = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    drain(&mut e);

    // Assert authentic consequences:
    // 1. One card placed on top of the deck
    assert_eq!(
        e.state_core().deck.last().copied(),
        Some(card_shan),
        "card placed on deck top"
    );
    // 2. Gains "侠" mark
    assert_eq!(
        e.state_core().vm_aux.flags.get(&(0, "dddxiaxing_xia".to_string())).copied(),
        Some(1),
        "gains xia mark"
    );
    // 3. Activates tao/jiu viewas
    assert_eq!(
        e.state_core().vm_aux.flags.get(&(0, "dddxiaxing_viewas_taojiu".to_string())).copied(),
        Some(1),
        "activates tao/jiu viewas"
    );

    // Save / Restore suspension test:
    let snapshot = e.save();
    let restored = Engine::restore_with_extensions(&snapshot, &[&d3_pack, &p9_pack])
        .expect("restore dddxiaxing engine from snapshot");
    assert_eq!(restored.state_hash(), e.state_hash(), "restored hash must match");
    assert_eq!(
        restored.state_core().vm_aux.flags.get(&(0, "dddxiaxing_xia".to_string())),
        Some(&1)
    );
    assert_eq!(
        restored.state_core().vm_aux.flags.get(&(0, "dddxiaxing_viewas_taojiu".to_string())),
        Some(&1)
    );

    // Counter-example test: empty hand & equips blocks activation
    let mut e2 = Engine::new_with_extensions(260102, 2, &[&d3_pack, &p9_pack])
        .expect("start U01 d3 engine counter-example");
    e2.assign_general(0, zm).expect("assign general");
    let removed2 = e2.state_core_mut().player_mut(0).hand.drain(..).collect::<Vec<_>>();
    e2.state_core_mut().discard.extend(removed2);
    let removed_eq2 = e2.state_core_mut().player_mut(0).equips.drain(..).collect::<Vec<_>>();
    e2.state_core_mut().discard.extend(removed_eq2);

    e2.debug_set_turn(0, Phase::Prepare, 1);
    let _ = e2.submit(GameCommand {
        actor: 0,
        expected_revision: e2.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    drain(&mut e2);

    let xia_flag = e2
        .state_core()
        .vm_aux
        .flags
        .get(&(0, "dddxiaxing_xia".to_string()))
        .copied()
        .unwrap_or(0);
    assert_eq!(xia_flag, 0, "empty hand/equips must block dddxiaxing");
    assert_eq!(
        e2.state_core().vm_aux.flags.get(&(0, "dddxiaxing_blocked_empty".to_string())).copied(),
        Some(1),
        "blocked flag set when empty"
    );
}
