//! Conformance suite: pinned native golden vectors + DSL/VM/loader consistency checks.
//!
//! Phase 6 migration: runs the 14 frozen standard native golden scenarios (Ruleset v333 /
//! NGSA v107 / ABI 759 / WS 8 — zero golden hash changes), archive roundtrip determinism,
//! `content-native-loader` locked package verification, and DSL/VM differential checks.

#![deny(missing_docs)]

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use content_native_loader::{
    prepare_locked_content_with_bridge, restore_locked_engine, start_locked_engine,
    validate_and_freeze_raw_package, LockedStartMode,
};
use engine_api::ENGINE_BUILD_ID;
use engine_native::archive::{decode_archive, encode_archive, ARCHIVE_FORMAT_VERSION};
use engine_native::catalog::CardKind;
use engine_native::content::standard::STANDARD_RULESET_VERSION;
use engine_native::event::DamageNature;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use rule_vm::{CollectingHost, ExecResult, HostContext, ReferenceVm, RuleVm, VerifiedModule};
use serde::{Deserialize, Serialize};

/// One frozen native golden scenario vector.
#[derive(Debug, Clone)]
pub struct NativeGoldenVector {
    /// Scenario id (registered in `docs/native-coverage-manifest.md`).
    pub id: &'static str,
    /// Runner producing final canonical state hash.
    pub run: fn() -> [u8; 32],
    /// Expected frozen state hash hex (Ruleset v333).
    pub hash_hex: &'static str,
}

/// Content package logic-hash golden (DSL compile reproducibility).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageVector {
    /// Label.
    pub id: &'static str,
    /// Package id passed to compiler.
    pub package_id: &'static str,
    /// DSL source.
    pub source: &'static str,
    /// Expected manifest.logic_hash hex.
    pub logic_hash_hex: &'static str,
}

/// Official sample skills package (standard/jianxiong + peers compiled as one unit).
pub const JIANXIONG_SOURCE: &str = include_str!("../../../content/standard/jianxiong.skill");

/// Canonical standard bootstrap package used for VM backend differential checks.
pub const STANDARD_M1_SOURCE: &str = include_str!("../../../content/standard/standard_m1.skill");

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn fresh_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn submit_cmd(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    let expected_revision = e.revision();
    e.submit(GameCommand {
        actor,
        expected_revision,
        dedup_id: fresh_dedup(),
        action,
    })
}

fn next_phase(e: &mut Engine) -> CommandOutcome {
    let actor = e.turn().current;
    submit_cmd(e, actor, GameAction::NextPhase)
}

fn answer_sel(e: &mut Engine, selection: Vec<ChoiceOption>) -> CommandOutcome {
    let choice = e.open_choice().expect("choice open").clone();
    submit_cmd(
        e,
        choice.observer,
        GameAction::Answer {
            choice_id: choice.id,
            selection,
        },
    )
}

fn answer(e: &mut Engine, opt: ChoiceOption) -> CommandOutcome {
    answer_sel(e, vec![opt])
}

fn answer_min_cards(e: &mut Engine, choice: &engine_native::OpenChoice) -> CommandOutcome {
    let sel: Vec<ChoiceOption> = choice
        .spec
        .options
        .iter()
        .filter(|o| matches!(o, ChoiceOption::Card(_)))
        .copied()
        .take(choice.spec.min.max(1))
        .collect();
    answer_sel(e, sel)
}

fn advance_to_play(e: &mut Engine, player: u8) {
    for _ in 0..64 {
        let t = e.turn();
        if t.current == player && t.phase == Phase::Play {
            return;
        }
        match next_phase(e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { choice } => {
                answer_min_cards(e, &choice);
            }
            other => panic!("advance rejected: {other:?}"),
        }
    }
    panic!("advance_to_play did not converge");
}

fn play(e: &mut Engine, card: u32, targets: Vec<u8>) -> CommandOutcome {
    let actor = e.turn().current;
    submit_cmd(e, actor, GameAction::PlayCard { card, targets })
}

fn play_sha_at(e: &mut Engine, attacker: u8, target: u8) -> CommandOutcome {
    let sha = e
        .debug_hand(attacker)
        .iter()
        .copied()
        .find(|c| e.catalog().card_kind(*c) == CardKind::Sha)
        .expect("attacker holds a sha");
    let actor = e.turn().current;
    submit_cmd(
        e,
        actor,
        GameAction::PlayCard {
            card: sha,
            targets: vec![target],
        },
    )
}

fn scenario_sha_dodge_then_damage() -> [u8; 32] {
    let mut e = Engine::new(11, 4);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let shan = e.debug_place(1, "shan");
    e.debug_place(0, "sha");
    play_sha_at(&mut e, 0, 1);
    answer(&mut e, ChoiceOption::Card(shan));
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_place(1, "sha");
    play_sha_at(&mut e, 1, 0);
    e.state_hash()
}

fn scenario_death_terminal_single() -> [u8; 32] {
    let mut e = Engine::new(71, 2);
    e.debug_set_hp(0, 1);
    advance_to_play(&mut e, 1);
    for p in 0..2 {
        e.debug_clear_hand(p);
    }
    let _sha = e.debug_place(1, "sha");
    play_sha_at(&mut e, 1, 0);
    assert!(e.outcome().is_some(), "single mode must end after kill");
    e.state_hash()
}

fn scenario_sancai_nested_chain() -> [u8; 32] {
    let mut e = Engine::new(23, 4);
    e.debug_grant(0, "散财");
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    e.debug_place(1, "sha");
    play_sha_at(&mut e, 1, 0);
    answer(&mut e, ChoiceOption::Confirm);
    answer(&mut e, ChoiceOption::Player(2));
    let card = match e.open_choice().expect("card choice").spec.options[0] {
        ChoiceOption::Card(c) => c,
        _ => panic!("first option must be a card"),
    };
    answer(&mut e, ChoiceOption::Card(card));
    e.state_hash()
}

fn scenario_yishang_redirect() -> [u8; 32] {
    let mut e = Engine::new(31, 4);
    e.debug_grant(0, "移伤");
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    e.debug_place(1, "sha");
    play_sha_at(&mut e, 1, 0);
    answer(&mut e, ChoiceOption::Confirm);
    answer(&mut e, ChoiceOption::Player(2));
    e.state_hash()
}

fn scenario_suidun_window_matrix() -> [u8; 32] {
    let mut e = Engine::new(53, 4);
    e.debug_grant(0, "碎盾");
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    e.debug_place(1, "sha");
    play_sha_at(&mut e, 1, 0);
    advance_to_play(&mut e, 3);
    e.debug_clear_hand(0);
    e.debug_place(3, "sha");
    play_sha_at(&mut e, 3, 0);
    e.debug_grant(2, "肃清");
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    e.debug_place(1, "sha");
    play_sha_at(&mut e, 1, 0);
    e.state_hash()
}

fn scenario_shouyi_dynamic_skill() -> [u8; 32] {
    let mut e = Engine::new(61, 4);
    e.debug_grant(0, "授艺");
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    e.debug_place(1, "sha");
    play_sha_at(&mut e, 1, 0);
    assert!(e.debug_has_skill(0, "移伤"));
    advance_to_play(&mut e, 3);
    e.debug_clear_hand(0);
    play_sha_at(&mut e, 3, 0);
    answer(&mut e, ChoiceOption::Confirm);
    answer(&mut e, ChoiceOption::Player(1));
    advance_to_play(&mut e, 0);
    assert!(e.debug_has_skill(0, "移伤"));
    for _ in 0..5 {
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { choice } => {
                answer_min_cards(&mut e, &choice);
            }
            other => panic!("{other:?}"),
        }
        if !e.debug_has_skill(0, "移伤") {
            break;
        }
    }
    assert!(!e.debug_has_skill(0, "移伤"));
    e.state_hash()
}

fn scenario_discard_to_limit() -> [u8; 32] {
    let mut e = Engine::new(91, 4);
    advance_to_play(&mut e, 0);
    e.debug_set_hp(0, 2);
    for _ in 0..4 {
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { choice } => {
                answer_min_cards(&mut e, &choice);
            }
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(e.debug_hand(0).len(), 2);
    e.state_hash()
}

fn scenario_rollback_invariants() -> [u8; 32] {
    let mut e = Engine::new(81, 4);
    e.debug_grant(0, "散财");
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    e.debug_place(1, "sha");
    play_sha_at(&mut e, 1, 0);
    let id = e.open_choice().expect("choice open").id;
    let before = e.state_hash();
    assert!(matches!(
        e.submit(GameCommand {
            actor: e.open_choice().unwrap().observer,
            expected_revision: e.revision(),
            dedup_id: fresh_dedup(),
            action: GameAction::Answer {
                choice_id: id,
                selection: vec![ChoiceOption::Card(9999)],
            },
        }),
        CommandOutcome::Rejected(engine_native::RejectReason::InvalidChoice)
    ));
    assert_eq!(e.state_hash(), before);
    e.debug_set_budget(2);
    let before_small_budget = e.state_hash();
    assert!(matches!(
        e.submit(GameCommand {
            actor: e.open_choice().unwrap().observer,
            expected_revision: e.revision(),
            dedup_id: fresh_dedup(),
            action: GameAction::Answer {
                choice_id: id,
                selection: vec![ChoiceOption::Confirm],
            },
        }),
        CommandOutcome::Rejected(engine_native::RejectReason::BudgetExhausted)
    ));
    assert_eq!(e.state_hash(), before_small_budget);
    e.debug_set_budget(4096);
    answer(&mut e, ChoiceOption::Confirm);
    e.state_hash()
}

fn scenario_equip_distance_matrix() -> [u8; 32] {
    let mut e = Engine::new(101, 4);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(2);
    let sha = e.debug_place(0, "sha");
    play(&mut e, sha, vec![2]);
    let hm = e.debug_place(0, "horse_minus");
    play(&mut e, hm, vec![]);
    play(&mut e, sha, vec![2]);
    let w2 = e.debug_place(0, "weapon2");
    play(&mut e, w2, vec![]);
    advance_to_play(&mut e, 2);
    e.debug_clear_hand(0);
    let hp_card = e.debug_place(2, "horse_plus");
    play(&mut e, hp_card, vec![]);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(2);
    let sha2 = e.debug_place(0, "sha");
    play(&mut e, sha2, vec![2]);
    e.debug_clear_hand(0);
    let w3 = e.debug_place(0, "weapon3");
    play(&mut e, w3, vec![]);
    e.state_hash()
}

fn scenario_chain_fire_propagation() -> [u8; 32] {
    let mut e = Engine::new(113, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let chain = e.debug_place(0, "chain");
    play(&mut e, chain, vec![1, 2]);
    let fsha = e.debug_place(0, "fire_sha");
    play(&mut e, fsha, vec![1]);
    let sha = e.debug_place(0, "sha");
    play(&mut e, sha, vec![1]);
    e.debug_deal_damage(None, 2, 1, DamageNature::Thunder);
    e.state_hash()
}

fn scenario_aoe_response_queue() -> [u8; 32] {
    let mut e = Engine::new(131, 4);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    for p in [1u8, 2, 3] {
        e.debug_clear_hand(p);
    }
    let savage = e.debug_place(0, "savage");
    let sha1 = e.debug_place(1, "sha");
    let _sha3 = e.debug_place(3, "sha");
    play(&mut e, savage, vec![]);
    answer(&mut e, ChoiceOption::Card(sha1));
    answer(&mut e, ChoiceOption::Pass);
    e.debug_clear_hand(1);
    let _shan1 = e.debug_place(1, "shan");
    let archery = e.debug_place(0, "archery");
    play(&mut e, archery, vec![]);
    answer(&mut e, ChoiceOption::Pass);
    e.state_hash()
}

fn scenario_duel_alternating_response() -> [u8; 32] {
    let mut e = Engine::new(141, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let duel = e.debug_place(0, "duel");
    let sha_t1 = e.debug_place(2, "sha");
    let sha_s = e.debug_place(0, "sha");
    play(&mut e, duel, vec![2]);
    answer(&mut e, ChoiceOption::Card(sha_t1));
    answer(&mut e, ChoiceOption::Card(sha_s));
    e.state_hash()
}

fn scenario_nullify_chain_on_savage() -> [u8; 32] {
    let mut e = Engine::new(153, 4);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    for p in [1u8, 2, 3] {
        e.debug_clear_hand(p);
    }
    let savage = e.debug_place(0, "savage");
    let nul1 = e.debug_place(1, "nullify");
    let nul2 = e.debug_place(2, "nullify");
    let sha1 = e.debug_place(1, "sha");
    play(&mut e, savage, vec![]);
    answer(&mut e, ChoiceOption::Card(nul1));
    answer(&mut e, ChoiceOption::Card(nul2));
    answer(&mut e, ChoiceOption::Card(sha1));
    e.state_hash()
}

fn scenario_judge_phase_delayed_tricks() -> [u8; 32] {
    let mut e = Engine::new(161, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_deck_top_by(|def| def.suit.is_spade() && def.kind == CardKind::Sha);
    let le = e.debug_place(0, "le");
    play(&mut e, le, vec![1]);
    loop {
        let t = e.turn();
        if t.current == 1 && t.phase == Phase::Draw {
            break;
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { choice } => {
                answer_min_cards(&mut e, &choice);
            }
            other => panic!("{other:?}"),
        }
    }
    next_phase(&mut e);
    e.state_hash()
}

/// All 14 frozen standard native golden vectors (Ruleset v333 / NGSA v107).
pub const NATIVE_GOLDEN_VECTORS: &[NativeGoldenVector] = &[
    NativeGoldenVector {
        id: "sha_dodge_then_damage",
        run: scenario_sha_dodge_then_damage,
        hash_hex: "d52004cdfbfc8afb10dfce81e10c540e17ac0ab2117b7c20ff94e8cca2fed561",
    },
    NativeGoldenVector {
        id: "death_terminal_single",
        run: scenario_death_terminal_single,
        hash_hex: "7d5d161c37a5c7c33536c5a23fc2ec9e6e43790118d77dc9182fb1b52d39d7f3",
    },
    NativeGoldenVector {
        id: "sancai_nested_chain",
        run: scenario_sancai_nested_chain,
        hash_hex: "69b5c88f517aa8200f9591eab5e3033018f6db76796babe08634edf276a24035",
    },
    NativeGoldenVector {
        id: "yishang_redirect",
        run: scenario_yishang_redirect,
        hash_hex: "4127b9090b15f868fceb6901190a1459f1d7ddebddc5e37d4fa74d6d814bef71",
    },
    NativeGoldenVector {
        id: "suidun_window_matrix",
        run: scenario_suidun_window_matrix,
        hash_hex: "cbb9b795c3ec1408dbd1fca30cbbcd8e6cd1dfd754958152f3058ea749a92805",
    },
    NativeGoldenVector {
        id: "shouyi_dynamic_skill",
        run: scenario_shouyi_dynamic_skill,
        hash_hex: "eb3f63e81a75d6981b53b11630fbd70f7708a72d836dab6e9cd34c40c6840c90",
    },
    NativeGoldenVector {
        id: "discard_to_limit",
        run: scenario_discard_to_limit,
        hash_hex: "5f600731fd0aa495ae2520e41b5f8744f1604305006f27ea5ee8c02b65165421",
    },
    NativeGoldenVector {
        id: "rollback_invariants",
        run: scenario_rollback_invariants,
        hash_hex: "a36cdff4c3649f77605cbbb0030056418b0bc676c3dc568d52f64e16357609e0",
    },
    NativeGoldenVector {
        id: "equip_distance_matrix",
        run: scenario_equip_distance_matrix,
        hash_hex: "9689e92df3b75d45455361c442145438c068641dd269ef8466a8871ab675f9a6",
    },
    NativeGoldenVector {
        id: "chain_fire_propagation",
        run: scenario_chain_fire_propagation,
        hash_hex: "f507cc04af331039bb023e964bfacf22ff1edeff6dbd17a1445a5902a565c9b9",
    },
    NativeGoldenVector {
        id: "aoe_response_queue",
        run: scenario_aoe_response_queue,
        hash_hex: "67fcb763b3fa874e17015bb3296c7e1ad6c1fd98f8a717443438397c13258f86",
    },
    NativeGoldenVector {
        id: "duel_alternating_response",
        run: scenario_duel_alternating_response,
        hash_hex: "6b12acc94788a9b607363eaf4b3e69fb370cdf91d8042b2eda7033dafc645423",
    },
    NativeGoldenVector {
        id: "nullify_chain_on_savage",
        run: scenario_nullify_chain_on_savage,
        hash_hex: "3501cda99198af8395905448800b16653fd726a5b97d20b4d53f91a2b51cf487",
    },
    NativeGoldenVector {
        id: "judge_phase_delayed_tricks",
        run: scenario_judge_phase_delayed_tricks,
        hash_hex: "5e675d48cc3b3b5f58233223764376a91542f01e2bbdcbd8f608ce9b9c26f4cb",
    },
];

/// Run one native golden vector; returns Ok(hex) or Err description.
pub fn run_native_golden_vector(v: &NativeGoldenVector) -> Result<String, String> {
    let actual = hex32(&(v.run)());
    if actual == v.hash_hex {
        Ok(actual)
    } else {
        Err(format!("{}: mismatch got={actual} want={}", v.id, v.hash_hex))
    }
}

/// Compile package vector and return logic hash hex.
pub fn run_package_vector(v: &PackageVector) -> Result<String, String> {
    let pkg = content_compiler::compile_source_named(v.package_id, v.source)
        .map_err(|e| format!("{}: compile {e}", v.id))?;
    Ok(hex32(&pkg.manifest.logic_hash))
}

fn check_native_start_and_archive_roundtrip() -> Result<(), String> {
    if STANDARD_RULESET_VERSION != 333 {
        return Err(format!("expected ruleset 333, got {STANDARD_RULESET_VERSION}"));
    }
    if ARCHIVE_FORMAT_VERSION != 107 {
        return Err(format!("expected NGSA 107, got {ARCHIVE_FORMAT_VERSION}"));
    }
    for (players, seed) in [(4usize, 42u64), (2, 1), (8, 99)] {
        let a = Engine::new(seed, players);
        let b = Engine::new(seed, players);
        if a.state_hash() != b.state_hash() {
            return Err(format!("start({players},{seed}) non-deterministic"));
        }
        let archive = encode_archive(&a.save());
        let decoded = decode_archive(&archive).map_err(|e| format!("decode({players},{seed}): {e:?}"))?;
        let restored = Engine::restore(&decoded).map_err(|e| format!("restore({players},{seed}): {e:?}"))?;
        if restored.state_hash() != a.state_hash() {
            return Err(format!("archive roundtrip({players},{seed}) hash mismatch"));
        }
    }
    Ok(())
}

fn check_content_native_loader_locked_roundtrip() -> Result<(), String> {
    let pkg_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions/huanle-upstream-noname");
    let package_json = std::fs::read_to_string(pkg_dir.join("package.npack.json"))
        .map_err(|e| format!("read huanle package.npack.json: {e}"))?;
    let cards_bytes = std::fs::read(pkg_dir.join("assets/cards.json"))
        .map_err(|e| format!("read huanle assets/cards.json: {e}"))?;
    let mut assets = BTreeMap::new();
    assets.insert("assets/cards.json".to_string(), cards_bytes);
    let frozen = validate_and_freeze_raw_package(package_json, assets)?;
    let verified = prepare_locked_content_with_bridge(vec![frozen])?;
    let e = start_locked_engine(42, 4, LockedStartMode::Identity, &verified)?;
    let bytes = encode_archive(&e.save());
    let snap = decode_archive(&bytes).map_err(|err| format!("decode locked archive: {err:?}"))?;
    let restored = restore_locked_engine(&snap, &verified)
        .map_err(|err| format!("restore locked engine: {err:?}"))?;
    if restored.state_hash() != e.state_hash() {
        return Err("locked restore state_hash mismatch".into());
    }
    Ok(())
}

fn run_dual_vm_standard() -> Result<(), String> {
    let package = content_compiler::compile_source_named("standard", STANDARD_M1_SOURCE)
        .map_err(|error| format!("compile standard_m1: {error}"))?;

    for module in &package.modules {
        let verified = VerifiedModule::verify(module.clone())
            .map_err(|error| format!("{} verify: {error}", module.name))?;
        let context = HostContext {
            skill_id: 1,
            self_id: 1,
            event_player: 1,
            event_source: 2,
            event_card: 1,
            event_amount: 0,
        };
        let mut optimized_host = CollectingHost {
            ctx: context,
            ..Default::default()
        };
        let mut reference_host = optimized_host.clone();
        let mut optimized = RuleVm::new();
        let mut reference = ReferenceVm::new();
        let mut optimized_result = optimized
            .execute(&verified, &mut optimized_host, 10_000)
            .map_err(|error| format!("{} optimized: {error}", module.name))?;
        let mut reference_result = reference
            .execute(module, &mut reference_host, 10_000)
            .map_err(|error| format!("{} reference: {error}", module.name))?;

        let mut completed = false;
        for _ in 0..16 {
            if optimized_result != reference_result {
                return Err(format!(
                    "{} result mismatch: optimized={optimized_result:?} reference={reference_result:?}",
                    module.name
                ));
            }
            if optimized_host.effects != reference_host.effects {
                return Err(format!("{} emitted effects diverged", module.name));
            }

            match (optimized_result, reference_result) {
                (ExecResult::Halted { .. }, ExecResult::Halted { .. }) => {
                    completed = true;
                    break;
                }
                (
                    ExecResult::Paused {
                        continuation: optimized_continuation,
                        options,
                    },
                    ExecResult::Paused {
                        continuation: reference_continuation,
                        ..
                    },
                ) => {
                    let chosen = options.first().copied().unwrap_or_default();
                    optimized_result = optimized
                        .resume(&verified, &mut optimized_host, optimized_continuation, chosen)
                        .map_err(|error| format!("{} optimized resume: {error}", module.name))?;
                    reference_result = reference
                        .resume(module, &mut reference_host, reference_continuation, chosen)
                        .map_err(|error| format!("{} reference resume: {error}", module.name))?;
                }
                _ => return Err(format!("{} backend state diverged", module.name)),
            }
        }
        if !completed {
            return Err(format!("{} exceeded 16 choice resumptions", module.name));
        }
    }
    Ok(())
}

/// Aggregate report for CLI / CI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConformanceReport {
    /// Engine build id.
    pub engine_build_id: String,
    /// Passed case ids.
    pub passed: Vec<String>,
    /// Failed (id, detail).
    pub failed: Vec<(String, String)>,
}

impl ConformanceReport {
    /// True if no failures.
    pub fn ok(&self) -> bool {
        self.failed.is_empty()
    }
}

/// Run all built-in vectors.
pub fn run_all() -> ConformanceReport {
    let mut passed = Vec::new();
    let mut failed = Vec::new();

    for v in NATIVE_GOLDEN_VECTORS {
        match run_native_golden_vector(v) {
            Ok(_) => passed.push(format!("native_golden_{}", v.id)),
            Err(e) => failed.push((format!("native_golden_{}", v.id), e)),
        }
    }

    match check_native_start_and_archive_roundtrip() {
        Ok(()) => passed.push("native_start_archive_roundtrip".into()),
        Err(e) => failed.push(("native_start_archive_roundtrip".into(), e)),
    }

    match check_content_native_loader_locked_roundtrip() {
        Ok(()) => passed.push("content_native_loader_locked_roundtrip".into()),
        Err(e) => failed.push(("content_native_loader_locked_roundtrip".into(), e)),
    }

    match content_compiler::compile_source_named("standard", JIANXIONG_SOURCE) {
        Ok(a) => match content_compiler::compile_source_named("standard", JIANXIONG_SOURCE) {
            Ok(b) if a.manifest.logic_hash == b.manifest.logic_hash => {
                passed.push("package_jianxiong_repro".into());
            }
            Ok(_) => failed.push((
                "package_jianxiong_repro".into(),
                "logic_hash not reproducible".into(),
            )),
            Err(e) => failed.push(("package_jianxiong_repro".into(), e.to_string())),
        },
        Err(e) => failed.push(("package_jianxiong_repro".into(), e.to_string())),
    }

    match run_dual_vm_standard() {
        Ok(()) => passed.push("dual_vm_standard_m1".into()),
        Err(error) => {
            failed.push(("dual_vm_standard_m1".into(), error));
        }
    }

    match content_compiler::compile_source_named(
        "guozhan",
        include_str!("../../../content/guozhan/gz_jianxiong.skill"),
    ) {
        Ok(pkg) => {
            passed.push("guozhan_gz_jianxiong_repro".into());
            if !pkg.skill_index.iter().any(|s| s.skill_def_id == 2398363843) {
                failed.push((
                    "guozhan_gz_jianxiong_repro".into(),
                    "skill def id 2398363843 not in skill_index".into(),
                ));
            }
        }
        Err(e) => failed.push(("guozhan_gz_jianxiong_repro".into(), e.to_string())),
    }
    match content_compiler::compile_source_named(
        "guozhan_wei_batch",
        include_str!("../../../content/guozhan/gz_wei_skills.skill"),
    ) {
        Ok(pkg) => {
            passed.push("guozhan_wei_batch_repro".into());
            if pkg.skill_index.len() < 6 {
                failed.push((
                    "guozhan_wei_batch_repro".into(),
                    format!("expected >=6 skills, got {}", pkg.skill_index.len()),
                ));
            }
        }
        Err(e) => failed.push(("guozhan_wei_batch_repro".into(), e.to_string())),
    }
    match content_compiler::compile_source_named(
        "guozhan_other_batch",
        include_str!("../../../content/guozhan/gz_other_skills.skill"),
    ) {
        Ok(pkg) => {
            passed.push("guozhan_other_batch_repro".into());
            if pkg.skill_index.len() < 9 {
                failed.push((
                    "guozhan_other_batch_repro".into(),
                    format!("expected >=9 skills, got {}", pkg.skill_index.len()),
                ));
            }
        }
        Err(e) => failed.push(("guozhan_other_batch_repro".into(), e.to_string())),
    }
    match content_compiler::compile_source_named(
        "guozhan_batch_all",
        include_str!("../../../content/guozhan/gz_batch_all.skill"),
    ) {
        Ok(_) => passed.push("guozhan_batch_all_repro".into()),
        Err(e) => failed.push(("guozhan_batch_all_repro".into(), e.to_string())),
    }
    match content_compiler::compile_source_named(
        "guozhan_zhulian_bihe",
        "skill gz_zhulian_bihe_spend(1) {\n  active play_phase {\n    effect guozhan_use_zhulian_bihe(self);\n  }\n}\n",
    ) {
        Ok(pkg) => {
            passed.push("guozhan_zhulian_bihe_abi641".into());
            if pkg.manifest.rule_vm_abi_version < 641 {
                failed.push((
                    "guozhan_zhulian_bihe_abi641".into(),
                    format!("expected rule_vm_abi_version >= 641, got {}", pkg.manifest.rule_vm_abi_version),
                ));
            }
        }
        Err(e) => failed.push(("guozhan_zhulian_bihe_abi641".into(), e.to_string())),
    }
    match content_compiler::compile_source_named(
        "huanle_shoulijian",
        include_str!("../../../content/original/ported/huanle/huanle_meta.skill"),
    ) {
        Ok(pkg) => {
            passed.push("huanle_shoulijian_abi642".into());
            if pkg.manifest.rule_vm_abi_version < 642 {
                failed.push((
                    "huanle_shoulijian_abi642".into(),
                    format!("expected rule_vm_abi_version >= 642, got {}", pkg.manifest.rule_vm_abi_version),
                ));
            }
        }
        Err(e) => failed.push(("huanle_shoulijian_abi642".into(), e.to_string())),
    }
    match content_compiler::compile_source_named(
        "shahai_impl",
        include_str!("../../../content/original/ported/shahai/shahai_impl.skill"),
    ) {
        Ok(pkg) => {
            passed.push("shahai_impl_repro".into());
            let gwent_ok = pkg
                .skill_index
                .iter()
                .any(|s| s.skill_def_id == 784509644 || s.skill_def_id == 2836710318);
            if !gwent_ok {
                failed.push((
                    "shahai_impl_repro".into(),
                    "gwent view_as skills missing from skill_index".into(),
                ));
            }
        }
        Err(e) => failed.push(("shahai_impl_repro".into(), e.to_string())),
    }
    match content_compiler::compile_source_named(
        "shahai_impl",
        include_str!("../../../content/original/ported/shahai/shahai_impl.skill"),
    ) {
        Ok(pkg) => {
            passed.push("gwent_dudayuanshuai_abi643".into());
            let dudayuanshuai_ok = pkg
                .skill_index
                .iter()
                .any(|s| s.skill_def_id == 3583584906 || s.skill_def_id == 3832623127);
            if !dudayuanshuai_ok {
                failed.push((
                    "gwent_dudayuanshuai_abi643".into(),
                    "gw_dudayuanshuai1/2 skills missing from skill_index".into(),
                ));
            }
            if pkg.manifest.rule_vm_abi_version < 643 {
                failed.push((
                    "gwent_dudayuanshuai_abi643".into(),
                    format!("expected rule_vm_abi_version >= 643, got {}", pkg.manifest.rule_vm_abi_version),
                ));
            }
        }
        Err(e) => failed.push(("gwent_dudayuanshuai_abi643".into(), e.to_string())),
    }
    match content_compiler::compile_source_named(
        "shahai_impl",
        include_str!("../../../content/original/ported/shahai/shahai_impl.skill"),
    ) {
        Ok(pkg) => {
            passed.push("shahai_hupeng_abi644".into());
            let hupeng_ok = pkg.skill_index.iter().any(|s| s.skill_def_id == 3607688462);
            if !hupeng_ok {
                failed.push((
                    "shahai_hupeng_abi644".into(),
                    "hupeng skill missing from skill_index".into(),
                ));
            }
            if pkg.manifest.rule_vm_abi_version < 644 {
                failed.push((
                    "shahai_hupeng_abi644".into(),
                    format!("expected rule_vm_abi_version >= 644, got {}", pkg.manifest.rule_vm_abi_version),
                ));
            }
        }
        Err(e) => failed.push(("shahai_hupeng_abi644".into(), e.to_string())),
    }

    ConformanceReport {
        engine_build_id: ENGINE_BUILD_ID.to_string(),
        passed,
        failed,
    }
}

/// Hex encode 32 bytes.
pub fn hex32(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_golden_vectors() {
        let report = run_all();
        if !report.ok() {
            for (id, detail) in &report.failed {
                eprintln!("FAIL {id}: {detail}");
            }
        }
        assert!(report.ok(), "conformance failures: {:?}", report.failed);
        assert!(report.passed.len() >= NATIVE_GOLDEN_VECTORS.len() + 2);
    }

    #[test]
    fn native_golden_vectors_individually() {
        assert_eq!(NATIVE_GOLDEN_VECTORS.len(), 14);
        for v in NATIVE_GOLDEN_VECTORS {
            let h = run_native_golden_vector(v).unwrap();
            assert_eq!(h, v.hash_hex, "{}", v.id);
        }
    }
}
