//! CLI: run golden conformance suite (CI entrypoint).

use conformance::run_all;

fn main() {
    let report = run_all();
    println!("engine_build_id={}", report.engine_build_id);
    println!("passed={}", report.passed.len());
    for id in &report.passed {
        println!("  OK {id}");
    }
    if !report.failed.is_empty() {
        println!("failed={}", report.failed.len());
        for (id, detail) in &report.failed {
            println!("  FAIL {id}: {detail}");
        }
        std::process::exit(1);
    }
    println!("ALL PASS");
}
