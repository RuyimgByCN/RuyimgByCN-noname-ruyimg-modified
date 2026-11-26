//! Content Package Test Runner (architecture §35).
//!
//! Validates a compiled DSL package: compiles DSL, verifies bytecode modules,
//! executes VM modules cleanly, and checks native engine start + archive invariants.

use engine_native::archive::{decode_archive, encode_archive};
use engine_native::Engine;
use rule_vm::{CollectingHost, ExecResult, HostContext, RuleVm, VerifiedModule};

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
                    "triggers={} modifiers={} modules={}",
                    p.trigger_index.len(),
                    p.modifier_index.len(),
                    p.modules.len()
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

    // 3. Bytecode verification + RuleVm execution test.
    let mut verified_count = 0usize;
    let mut vm_detail = String::from("ok");
    let mut vm_ok = true;
    for module in &pkg.modules {
        match VerifiedModule::verify(module.clone()) {
            Ok(verified) => {
                let mut vm = RuleVm::new();
                let mut host = CollectingHost {
                    ctx: HostContext {
                        skill_id: 1,
                        self_id: 1,
                        event_player: 1,
                        event_source: 2,
                        event_card: 1,
                        event_amount: 0,
                    },
                    ..CollectingHost::default()
                };
                match vm.execute(&verified, &mut host, 10_000) {
                    Ok(ExecResult::Halted { .. } | ExecResult::Paused { .. }) => {
                        verified_count += 1;
                    }
                    Ok(other) => {
                        vm_ok = false;
                        vm_detail = format!("{} unexpected vm result: {other:?}", module.name);
                        break;
                    }
                    Err(e) => {
                        vm_ok = false;
                        vm_detail = format!("{} vm error: {e}", module.name);
                        break;
                    }
                }
            }
            Err(e) => {
                vm_ok = false;
                vm_detail = format!("{} verify error: {e}", module.name);
                break;
            }
        }
    }
    results.push(PackageTestResult {
        test: "vm_verify_and_exec".into(),
        passed: vm_ok,
        detail: if vm_ok {
            format!("verified={verified_count} modules")
        } else {
            vm_detail
        },
    });

    // 4. Native Engine start check.
    let eng = Engine::new(42, 4);
    results.push(PackageTestResult {
        test: "native_engine_start".into(),
        passed: eng.revision() == 0,
        detail: format!("revision={} hash={}", eng.revision(), hex_short(&eng.state_hash())),
    });

    // 5. Manifest logic hash pinned check.
    results.push(PackageTestResult {
        test: "manifest_pinned".into(),
        passed: pkg.manifest.logic_hash != [0u8; 32],
        detail: format!("hash={}", hex_short(&pkg.manifest.logic_hash)),
    });

    // 6. Native snapshot/archive invariants hold across roundtrip.
    let bytes = encode_archive(&eng.save());
    let inv = decode_archive(&bytes)
        .map_err(|e| format!("decode error: {e:?}"))
        .and_then(|snap| Engine::restore(&snap).map_err(|e| format!("restore error: {e:?}")))
        .and_then(|restored| {
            if restored.state_hash() == eng.state_hash() {
                Ok(())
            } else {
                Err("state_hash mismatch after restore".into())
            }
        });
    results.push(PackageTestResult {
        test: "invariants".into(),
        passed: inv.is_ok(),
        detail: match &inv {
            Ok(()) => "native archive + restore invariants pass".into(),
            Err(e) => e.clone(),
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
