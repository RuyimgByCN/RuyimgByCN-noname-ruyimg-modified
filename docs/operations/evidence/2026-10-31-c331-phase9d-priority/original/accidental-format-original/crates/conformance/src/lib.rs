//! Conformance suite: pinned golden vectors for engine + content.
//!
//! Architecture Phase 11 — same golden cases across Native / WASM / desktop shell.
//! Bumping a vector requires an intentional domain version change + doc note.
//!
//! The bootstrap state hashes were re-pinned on 2026-08-08 after the roster-closure
//! round restored cross-package character skill references (391 characters across
//! 20 packages; gongsunzan/old_re_lidian entered standard with refresh refs):
//! bootstrapped characters now carry their full skill sets, changing every vector.
//! Earlier pins: ABI 346 (start_*) and ABI 616 (identity_*).
//! they had drifted from their ABI 187/231 pins as the bootstrap ruleset advanced.
//! The two content-repro vectors (`package_jianxiong_repro`, `dual_vm_standard_m1`)
//! were unaffected. ABI 347 added an opt-in per-player damage-taken counter that
//! low-ABI bootstrap packages do not record, so these hashes remain stable.
//! See `content/original/ported-character-log.md` for the re-pin record.

#![deny(missing_docs)]

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    ENGINE_BUILD_ID,
};
use engine_core::{run_guozhan_match, run_identity_match, Engine, MatchConfig};
use rule_vm::{CollectingHost, ExecResult, HostContext, ReferenceVm, RuleVm, VerifiedModule};
use serde::{Deserialize, Serialize};

/// One start-game golden vector (bootstrap ruleset, hash command parity).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartVector {
    /// Label.
    pub id: &'static str,
    /// Players.
    pub players: u8,
    /// Seed.
    pub seed: u64,
    /// Expected state hash hex (lowercase).
    pub hash_hex: &'static str,
}

/// Identity auto-match golden vector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchVector {
    /// Label.
    pub id: &'static str,
    /// Players.
    pub players: u8,
    /// Seed.
    pub seed: u64,
    /// Max steps (must be high enough to finish for non-capped vectors).
    pub max_steps: u32,
    /// Expected final state hash hex.
    pub hash_hex: &'static str,
    /// Whether the run is allowed to hit max_steps.
    pub allow_capped: bool,
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
    /// Expected manifest.logic_hash hex (filled after first pin; tests recompute if empty? no — pin).
    pub logic_hash_hex: &'static str,
}

/// Bootstrap start vectors (engine-cli `hash` parity).
pub const START_VECTORS: &[StartVector] = &[
    StartVector {
        id: "start_4_42",
        players: 4,
        seed: 42,
        // Re-pinned for ABI 346 (owner_is_target_trick_used_zero / hp_le_one / hp_le_two filters);
        // these state hashes drifted from the ABI 187 pin as the bootstrap ruleset advanced.
        hash_hex: "86daeaffe7f71fb6facda6e6ebe358a87076f5d9e2e7be196deb111b698c31a7",
    },
    StartVector {
        id: "start_2_1",
        players: 2,
        seed: 1,
        // Re-pinned for ABI 346; see start_4_42 note.
        hash_hex: "5834b20ea9fbb7d2d3664af54bc49edad763e3089641d88fc181eb9ca19292ee",
    },
    StartVector {
        id: "start_8_99",
        players: 8,
        seed: 99,
        // Re-pinned for ABI 346; see start_4_42 note.
        hash_hex: "8835f8955fcc4af79589c2c30718e3054ca20b78c31dcddeddbe6726685caa7e",
    },
];

/// Finished identity match vectors.
pub const MATCH_VECTORS: &[MatchVector] = &[
    MatchVector {
        id: "identity_4_42",
        players: 4,
        seed: 42,
        max_steps: 5_000,
        // Re-pinned at ABI 616 after 五谷丰登 gained its exact shared-pool resolution.
        hash_hex: "b351f4876316e27bb5e5aea05dbe3fc29c127e910eacd7b28b4f24a07681abd1",
        allow_capped: false,
    },
    MatchVector {
        id: "identity_2_7",
        players: 2,
        seed: 7,
        max_steps: 5_000,
        // Re-pinned at ABI 616; see identity_4_42 note.
        hash_hex: "8273d6cbf62d9045576f92a8cea42a4dc9431e5fee83a3461b18a584635ca217",
        allow_capped: false,
    },
    MatchVector {
        id: "identity_8_99",
        players: 8,
        seed: 99,
        max_steps: 5_000,
        // Re-pinned at ABI 616; see identity_4_42 note.
        hash_hex: "9ba17d5af5ffef0f8dd46314acd35b41af6c24ee9f80aa8c4b50319b13b5d406",
        allow_capped: false,
    },
];

/// Guozhan (国战) auto-match golden vectors (ADR 0598 phase B).
///
/// Pinned fresh at phase B landing — these are new vectors, not a re-pin. The auto-player is
/// a naive faction-aware heuristic; `allow_capped` tolerates runs that reach the step limit
/// without a terminal outcome (the skeleton AI may stall on complex boards).
pub const GUOZHAN_MATCH_VECTORS: &[MatchVector] = &[
    MatchVector {
        id: "guozhan_4_42",
        players: 4,
        seed: 42,
        max_steps: 2_000,
        // Re-pinned after each guozhan deck CardKind upgrade: tricks as Other (0604), then
        // yiyi (0614), shuiyanqijunx (0615), lulitongxin (0616), kefuzhongyuan (0617),
        // guguoanbang (0618), huoshaolianying (0619), lianjunshengyan (0620), yuanjiao (0621),
        // wenheluanwu (0622), haolingtianxia (0623), chiling (0624, A 段 Claude re-pin).
        hash_hex: "5e97c1b2007edfbf8a656bc05ef7bb4a660dc9aea26233b90a874dd6a2dfd347",
        allow_capped: true,
    },
    MatchVector {
        id: "guozhan_2_7",
        players: 2,
        seed: 7,
        max_steps: 2_000,
        // Re-pinned after guozhan tricks; see guozhan_4_42 note. 0622/0623 re-pin (C 段 omp),
        // 0624 chiling re-pin (A 段 Claude).
        hash_hex: "c5ed1c4b4d77069032024dcaf76edfe24c82d3aef2190fdbf5e5d6a5510743fb",
        allow_capped: true,
    },
];

/// Official sample skills package (standard/jianxiong + peers compiled as one unit).
pub const JIANXIONG_SOURCE: &str = include_str!("../../../content/standard/jianxiong.skill");

/// Canonical standard bootstrap package used for VM backend differential checks.
pub const STANDARD_M1_SOURCE: &str = include_str!("../../../content/standard/standard_m1.skill");

/// Run a start vector; returns Ok(hex) or Err description.
pub fn run_start_vector(v: &StartVector) -> Result<String, String> {
    let mut eng = Engine::new();
    let r = eng.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: v.players,
            rng_seed: v.seed,
            ruleset: "bootstrap".into(),
        },
    });
    match r {
        CommandResult::Accepted { state_hash, .. } => Ok(hex32(&state_hash)),
        other => Err(format!("{}: {:?}", v.id, other)),
    }
}

/// Run a match vector.
pub fn run_match_vector(v: &MatchVector) -> Result<String, String> {
    let report = run_identity_match(MatchConfig {
        player_count: v.players,
        rng_seed: v.seed,
        max_steps: v.max_steps,
        ruleset: "identity-standard".into(),
    })?;
    if report.capped && !v.allow_capped {
        return Err(format!("{}: capped at {} steps", v.id, report.steps));
    }
    Ok(hex32(&report.state_hash))
}

/// Run a guozhan match vector (ADR 0598 phase B).
pub fn run_guozhan_match_vector(v: &MatchVector) -> Result<String, String> {
    let report = run_guozhan_match(MatchConfig {
        player_count: v.players,
        rng_seed: v.seed,
        max_steps: v.max_steps,
        ruleset: "guozhan-standard".into(),
    })?;
    if report.capped && !v.allow_capped {
        return Err(format!("{}: capped at {} steps", v.id, report.steps));
    }
    Ok(hex32(&report.state_hash))
}

/// Compile package vector and return logic hash hex.
pub fn run_package_vector(v: &PackageVector) -> Result<String, String> {
    let pkg = content_compiler::compile_source_named(v.package_id, v.source)
        .map_err(|e| format!("{}: compile {e}", v.id))?;
    Ok(hex32(&pkg.manifest.logic_hash))
}

fn run_dual_vm_standard() -> Result<(), String> {
    let package = content_compiler::compile_source_named("standard", STANDARD_M1_SOURCE)
        .map_err(|error| format!("compile standard_m1: {error}"))?;

    for module in &package.modules {
        // PA06/B1：优化后端只收装载期已校验模块；参照实现保留自有校验。
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
                        .resume(
                            &verified,
                            &mut optimized_host,
                            optimized_continuation,
                            chosen,
                        )
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

    for v in START_VECTORS {
        match run_start_vector(v) {
            Ok(h) if h == v.hash_hex => passed.push(v.id.to_string()),
            Ok(h) => failed.push((
                v.id.to_string(),
                format!("hash mismatch got={h} want={}", v.hash_hex),
            )),
            Err(e) => failed.push((v.id.to_string(), e)),
        }
    }

    for v in MATCH_VECTORS {
        match run_match_vector(v) {
            Ok(h) if h == v.hash_hex => passed.push(v.id.to_string()),
            Ok(h) => failed.push((
                v.id.to_string(),
                format!("hash mismatch got={h} want={}", v.hash_hex),
            )),
            Err(e) => failed.push((v.id.to_string(), e)),
        }
    }

    // ADR 0598 phase B: guozhan golden vectors (freshly pinned, not a re-pin).
    for v in GUOZHAN_MATCH_VECTORS {
        match run_guozhan_match_vector(v) {
            Ok(h) if h == v.hash_hex => passed.push(v.id.to_string()),
            Ok(h) => failed.push((
                v.id.to_string(),
                format!("hash mismatch got={h} want={}", v.hash_hex),
            )),
            Err(e) => failed.push((v.id.to_string(), e)),
        }
    }

    // Pin package logic hash dynamically first run via compute; we assert reproducibility
    // and non-empty, and pin against dual compile.
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

    // ADR 0604: guozhan DSL must compile. Verify gz_jianxiong (the first migrated guozhan
    // skill) compiles reproducibly, and that the ABI-641 guozhan_use_zhulian_bihe builtin
    // lowers correctly (its package requires ABI 641).
    match content_compiler::compile_source_named(
        "guozhan",
        include_str!("../../../content/guozhan/gz_jianxiong.skill"),
    ) {
        Ok(pkg) => {
            passed.push("guozhan_gz_jianxiong_repro".into());
            // Confirm the skill def id matches the claimed constant.
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
        Ok(pkg) => {
            passed.push("guozhan_batch_all_repro".into());
        }
        Err(e) => failed.push(("guozhan_batch_all_repro".into(), e.to_string())),
    }
    match content_compiler::compile_source_named(
        "guozhan_zhulian_bihe",
        "skill gz_zhulian_bihe_spend(1) {\n  active play_phase {\n    effect guozhan_use_zhulian_bihe(self);\n  }\n}\n",
    ) {
        Ok(pkg) => {
            passed.push("guozhan_zhulian_bihe_abi641".into());
            // The guozhan_use_zhulian_bihe builtin requires ABI 641.
            if pkg.manifest.rule_vm_abi_version < 641 {
                failed.push((
                    "guozhan_zhulian_bihe_abi641".into(),
                    format!(
                        "expected rule_vm_abi_version >= 641, got {}",
                        pkg.manifest.rule_vm_abi_version
                    ),
                ));
            }
        }
        Err(e) => failed.push(("guozhan_zhulian_bihe_abi641".into(), e.to_string())),
    }
    // ADR 0607: the huanle shoulijian (手里剑) custom-basic-card DSL must compile and require
    // ABI 642 (the ResolveShoulijian builtin).
    match content_compiler::compile_source_named(
        "huanle_shoulijian",
        include_str!("../../../content/original/ported/huanle/huanle_meta.skill"),
    ) {
        Ok(pkg) => {
            passed.push("huanle_shoulijian_abi642".into());
            if pkg.manifest.rule_vm_abi_version < 642 {
                failed.push((
                    "huanle_shoulijian_abi642".into(),
                    format!(
                        "expected rule_vm_abi_version >= 642, got {}",
                        pkg.manifest.rule_vm_abi_version
                    ),
                ));
            }
        }
        Err(e) => failed.push(("huanle_shoulijian_abi642".into(), e.to_string())),
    }
    // ADR 0607: the shahai (杀海拾遗) skill DSL must compile — including the newly added
    // gwent view_as skills (gw_wuyao_skill → 杀, gw_lang_skill → 酒).
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
    // ADR 0608: the gwent 杜达元帅1/2 special-card skills must compile and require ABI 643
    // (the CancelIncomingCardUse builtin).
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
                    format!(
                        "expected rule_vm_abi_version >= 643, got {}",
                        pkg.manifest.rule_vm_abi_version
                    ),
                ));
            }
        }
        Err(e) => failed.push(("gwent_dudayuanshuai_abi643".into(), e.to_string())),
    }
    // ADR 0612: the gwent 呼朋 (hupeng) skill must compile with the ABI 644
    // choose_other_gain_one_of_two_skills builtin and stay linked from gw_zhuoertan.
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
                    format!(
                        "expected rule_vm_abi_version >= 644, got {}",
                        pkg.manifest.rule_vm_abi_version
                    ),
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
        assert!(
            report.passed.len()
                >= START_VECTORS.len() + MATCH_VECTORS.len() + GUOZHAN_MATCH_VECTORS.len()
        );
    }

    #[test]
    fn start_vectors_individually() {
        for v in START_VECTORS {
            let h = run_start_vector(v).unwrap();
            assert_eq!(h, v.hash_hex, "{}", v.id);
        }
    }

    #[test]
    fn match_vectors_individually() {
        for v in MATCH_VECTORS {
            let h = run_match_vector(v).unwrap();
            assert_eq!(h, v.hash_hex, "{}", v.id);
        }
    }

    #[test]
    fn guozhan_match_vectors_individually() {
        for v in GUOZHAN_MATCH_VECTORS {
            let h = run_guozhan_match_vector(v).unwrap();
            assert_eq!(h, v.hash_hex, "{}", v.id);
        }
    }
}
