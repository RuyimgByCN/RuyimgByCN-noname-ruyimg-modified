//! Content Package Test Runner (architecture §35).
//!
//! Validates a compiled content package: compiles DSL, checks trigger/modifier
//! indexes, loads into engine, and verifies the engine doesn't crash.

use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId};

/// Test result for one package test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageTestResult {
    /// Test name.
    pub test: String,
    /// Whether the test passed.
    pub passed: bool,
    /// Detail message (error description if failed).
    pub detail: String,
}

/// Run the full package test suite against a DSL source.
pub fn run_package_tests(source: &str) -> Vec<PackageTestResult> {
    let mut results = Vec::new();

    // 1. Compilation test.
    let pkg = match content_compiler::compile_source(source) {
        Ok(p) => {
            results.push(PackageTestResult {
                test: "compile".into(),
                passed: true,
                detail: format!(
                    "triggers={} modifiers={}",
                    p.trigger_index.len(),
                    p.modifier_index.len()
                ),
            });
            p
        }
        Err(e) => {
            results.push(PackageTestResult {
                test: "compile".into(),
                passed: false,
                detail: e.to_string(),
            });
            return results;
        }
    };

    // 2. Logic hash reproducibility test.
    let pkg2 = content_compiler::compile_source(source);
    let reproducible = match pkg2 {
        Ok(p2) => p2.manifest.logic_hash == pkg.manifest.logic_hash,
        Err(_) => false,
    };
    results.push(PackageTestResult {
        test: "logic_hash_reproducible".into(),
        passed: reproducible,
        detail: if reproducible {
            "logic hash matches on recompile".into()
        } else {
            "logic hash differs on recompile".into()
        },
    });

    // 3. Engine load test.
    let mut eng = engine_core::Engine::new();
    let load_result = eng.load_package(&pkg);
    results.push(PackageTestResult {
        test: "engine_load".into(),
        passed: load_result.is_ok(),
        detail: match &load_result {
            Ok(()) => format!("packages={}", eng.content().package_count()),
            Err(e) => e.to_string(),
        },
    });

    if load_result.is_err() {
        return results;
    }

    // 4. StartGame with package loaded.
    let start_result = eng.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 4,
            rng_seed: 42,
            ruleset: "identity-standard".into(),
        },
    });
    let started = matches!(start_result, engine_api::CommandResult::Accepted { .. });
    results.push(PackageTestResult {
        test: "start_game_with_package".into(),
        passed: started,
        detail: if started {
            format!("revision={}", eng.state().revision)
        } else {
            "StartGame rejected".into()
        },
    });

    // 5. Manifest pinned test.
    let manifest_ok = eng.manifest().is_some();
    results.push(PackageTestResult {
        test: "manifest_pinned".into(),
        passed: manifest_ok,
        detail: if manifest_ok {
            format!("hash={}", hex_short(&eng.manifest_hash().0))
        } else {
            "no manifest pinned".into()
        },
    });

    // 6. Invariants hold.
    let inv = engine_core::invariants::check_invariants(eng.state());
    results.push(PackageTestResult {
        test: "invariants".into(),
        passed: inv.is_ok(),
        detail: match &inv {
            Ok(()) => "all invariants pass".into(),
            Err(e) => e.to_string(),
        },
    });

    results
}

/// Render test results as text.
pub fn render_results(results: &[PackageTestResult]) -> String {
    let mut out = String::new();
    let passed = results.iter().filter(|r| r.passed).count();
    out.push_str(&format!(
        "package tests: {}/{} passed\n",
        passed,
        results.len()
    ));
    for r in results {
        let status = if r.passed { "✓" } else { "✗" };
        out.push_str(&format!("  {status} {:<30} {}\n", r.test, r.detail));
    }
    out
}

fn hex_short(bytes: &[u8]) -> String {
    bytes.iter().take(8).map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const JIANXIONG: &str = include_str!("../../../content/standard/jianxiong.skill");

    #[test]
    fn jianxiong_package_passes_tests() {
        let results = run_package_tests(JIANXIONG);
        assert!(results.iter().all(|r| r.passed), "{:?}", results);
        assert!(results.iter().any(|r| r.test == "compile"));
        assert!(results.iter().any(|r| r.test == "invariants"));
    }

    #[test]
    fn invalid_source_fails_compile() {
        let results = run_package_tests("not valid dsl {{{{");
        let compile = results.iter().find(|r| r.test == "compile").unwrap();
        assert!(!compile.passed);
    }
}
