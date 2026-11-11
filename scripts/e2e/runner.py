#!/usr/bin/env python3
"""Unified E2E Test Runner CLI for noname-next.

Supports running the 4-tier opaque-box integration test suite:
- Tier 1: Feature Coverage (Category-Partition)
- Tier 2: Boundary Value Analysis & Fault Injection
- Tier 3: Pairwise Combinatorial Interactions
- Tier 4: Real-World Application Scenarios

Generates TAP and JSON reports conforming to TEST_INFRA.md and SCOPE.md.
"""

from __future__ import annotations

import argparse
import datetime
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time
from typing import Any, Dict, List, Optional, Tuple

TEST_LINE_RE = re.compile(r"^test\s+([\w:]+)\s+\.\.\.\s+(ok|FAILED|ignored)")
SUMMARY_LINE_RE = re.compile(
    r"test result:\s+(ok|FAILED)\.\s+(\d+)\s+passed;\s+(\d+)\s+failed;\s+(\d+)\s+ignored"
)


def find_repo_root() -> Path:
    """Find the noname-next workspace root."""
    current = Path.cwd().resolve()
    for candidate in [current, current / "noname-next", current.parent]:
        if (candidate / "crates" / "engine-native").is_dir() and (candidate / "Cargo.toml").is_file():
            return candidate
    # Fallback to searching upwards
    for parent in current.parents:
        if (parent / "crates" / "engine-native").is_dir() and (parent / "Cargo.toml").is_file():
            return parent
    return current


def parse_cargo_test_output(output: str) -> Tuple[List[Dict[str, Any]], Dict[str, int]]:
    """Parse rust test output into individual test results and summary counts."""
    tests: List[Dict[str, Any]] = []
    passed = 0
    failed = 0
    ignored = 0

    for line in output.splitlines():
        line = line.strip()
        match = TEST_LINE_RE.match(line)
        if match:
            test_name = match.group(1)
            status_str = match.group(2)
            if status_str == "ok":
                status = "passed"
                passed += 1
            elif status_str == "FAILED":
                status = "failed"
                failed += 1
            else:
                status = "ignored"
                ignored += 1
            tests.append({
                "name": test_name,
                "status": status,
            })
            continue

        sum_match = SUMMARY_LINE_RE.search(line)
        if sum_match:
            # Reconcile summary counts if needed
            p = int(sum_match.group(2))
            f = int(sum_match.group(3))
            i = int(sum_match.group(4))
            if p + f + i >= len(tests):
                passed, failed, ignored = p, f, i

    total = passed + failed + ignored
    summary = {
        "total": total,
        "passed": passed,
        "failed": failed,
        "ignored": ignored,
    }
    return tests, summary


def generate_tap_output(tests: List[Dict[str, Any]]) -> str:
    """Generate TAP v14 compliant output string."""
    lines = ["TAP version 14", f"1..{len(tests)}"]
    for idx, test in enumerate(tests, 1):
        status = test["status"]
        name = test["name"]
        if status == "passed":
            lines.append(f"ok {idx} - {name}")
        elif status == "ignored":
            lines.append(f"ok {idx} - {name} # SKIP")
        else:
            lines.append(f"not ok {idx} - {name}")
    return "\n".join(lines)


def build_cargo_filters(
    tier: Optional[int] = None,
    feature: Optional[int] = None,
    test_filter: Optional[str] = None,
) -> List[str]:
    """Construct cargo test filter tokens from tier, feature, and custom filter options.

    Cargo filters match test paths via exact substring match.
    In the E2E test suite:
    - Tier 1: test_t1_f{feature}_...
    - Tier 2: test_t2_f{feature}_...
    - Tier 3: test_t3_interaction_{feature}_...
    - Tier 4: test_t4_scenario_{scenario}_...

    To combine tier and feature with conjunction (AND) semantics, we synthesize
    a single substring token (e.g. 't1_f1_' or 't2_f10_') rather than passing
    space-separated tokens or multiple OR-disjoined tokens.
    """
    filters: List[str] = []

    if tier is not None and feature is not None:
        if tier in (1, 2):
            filters.append(f"t{tier}_f{feature}_")
        elif tier == 3:
            filters.append(f"t3_interaction_{feature}_")
        elif tier == 4:
            filters.append(f"t4_scenario_{feature}_")
    elif tier is not None:
        filters.append(f"tier{tier}")
    elif feature is not None:
        filters.append(f"_f{feature}_")

    if test_filter:
        filters.append(test_filter)

    return filters


def run_e2e_suite(
    repo_root: Path,
    tier: Optional[int] = None,
    feature: Optional[int] = None,
    quick: bool = False,
    test_filter: Optional[str] = None,
    report_path: Optional[Path] = None,
    tap_stdout: bool = False,
    extra_cargo_args: Optional[List[str]] = None,
) -> int:
    """Execute cargo test e2e suite and produce summary reports."""
    start_time = time.time()
    timestamp = datetime.datetime.now(datetime.timezone.utc).isoformat()

    # Determine test filter tokens
    filters = build_cargo_filters(tier=tier, feature=feature, test_filter=test_filter)
    combined_filter = " ".join(filters) if filters else ""

    cmd = [
        "cargo",
        "test",
        "--offline",
        "-p",
        "engine-native",
        "--test",
        "e2e",
    ]
    if extra_cargo_args:
        cmd.extend(extra_cargo_args)

    cmd.append("--")
    cmd.append("--nocapture")
    for f in filters:
        cmd.append(f)

    print(f"[*] Running E2E Test Suite in: {repo_root}")
    print(f"[*] Command: {' '.join(cmd)}")
    print(f"[*] Tier: {tier or 'All'}, Feature: {feature or 'All'}, Quick: {quick}")

    proc = subprocess.run(
        cmd,
        cwd=repo_root,
        capture_output=True,
        text=True,
    )

    duration = time.time() - start_time
    combined_output = proc.stdout + "\n" + proc.stderr

    # Parse results
    tests, summary = parse_cargo_test_output(combined_output)
    summary["duration_seconds"] = round(duration, 3)

    # TAP output if requested
    tap_str = generate_tap_output(tests)
    if tap_stdout:
        print("\n" + tap_str + "\n")

    # Determine final exit code
    if proc.returncode != 0:
        final_exit_code = proc.returncode
    elif summary["total"] == 0:
        final_exit_code = 1
    elif summary["failed"] > 0:
        final_exit_code = 1
    else:
        final_exit_code = 0

    # Target JSON report
    if report_path is None:
        report_path = repo_root / "target" / "e2e-report.json"

    report_path.parent.mkdir(parents=True, exist_ok=True)

    report_data = {
        "timestamp": timestamp,
        "suite": "noname-next-e2e",
        "tier": tier,
        "feature": feature,
        "quick": quick,
        "filter": combined_filter,
        "exit_code": final_exit_code,
        "summary": summary,
        "tests": tests,
    }

    report_path.write_text(json.dumps(report_data, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"[✓] Wrote structured JSON report to: {report_path}")

    # Console summary
    print("\n" + "=" * 48)
    print("           E2E TEST SUITE SUMMARY")
    print("=" * 48)
    print(f" Total Tests : {summary['total']}")
    print(f" Passed      : {summary['passed']}")
    print(f" Failed      : {summary['failed']}")
    print(f" Ignored     : {summary['ignored']}")
    print(f" Duration    : {summary['duration_seconds']}s")
    print("=" * 48)

    if proc.returncode != 0:
        print(f"[!] Test runner process failed with exit code: {proc.returncode}")
        if summary["failed"] > 0 or not tests:
            print("\n--- Failure Diagnostics ---")
            for line in combined_output.splitlines():
                if "FAILED" in line or "error:" in line or "panicked at" in line:
                    print(line)
            print("---------------------------\n")
        return final_exit_code

    if summary["total"] == 0:
        print("\n[!] Error: No tests matched the specified filter criteria (0 tests executed).")
        return 1

    if summary["failed"] > 0:
        return 1

    return 0


def run_self_test() -> int:
    """Run internal test parser, serializer, and filter generator unit checks."""
    sample_output = """
running 5 tests
test e2e_harness::tests::test_harness_match_startup ... ok
test e2e_harness::tests::test_harness_rejection_zero_mutation ... ok
test e2e_harness::tests::test_harness_step_resolution ... ok
test e2e_harness::tests::test_harness_snapshot_roundtrip ... ok
test e2e_harness::tests::test_harness_choice_answering ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
    """
    tests, summary = parse_cargo_test_output(sample_output)
    assert summary["total"] == 5, f"Expected 5 tests, got {summary['total']}"
    assert summary["passed"] == 5, f"Expected 5 passed, got {summary['passed']}"
    assert summary["failed"] == 0, f"Expected 0 failed, got {summary['failed']}"

    tap = generate_tap_output(tests)
    assert "TAP version 14" in tap, "TAP header missing"
    assert "ok 1 - e2e_harness::tests::test_harness_match_startup" in tap, "TAP test line missing"

    # Verify zero-test handling
    empty_output = "running 0 tests\ntest result: ok. 0 passed; 0 failed; 0 ignored\n"
    _, zero_summary = parse_cargo_test_output(empty_output)
    assert zero_summary["total"] == 0, f"Expected 0 tests, got {zero_summary['total']}"

    # Verify build_cargo_filters combinations
    assert build_cargo_filters(None, 1) == ["_f1_"], "Defect 1 regression: feature 1 must produce _f1_"
    assert build_cargo_filters(None, 9) == ["_f9_"], "Defect 1 regression: feature 9 must produce _f9_"
    assert build_cargo_filters(None, 10) == ["_f10_"], "Feature 10 must produce _f10_"
    assert build_cargo_filters(None, 23) == ["_f23_"], "Feature 23 must produce _f23_"

    assert build_cargo_filters(1, 1) == ["t1_f1_"], "Defect 2 regression: tier 1 feature 1 must be t1_f1_"
    assert build_cargo_filters(1, 10) == ["t1_f10_"], "Defect 2 regression: tier 1 feature 10 must be t1_f10_"
    assert build_cargo_filters(2, 5) == ["t2_f5_"], "Tier 2 feature 5 must be t2_f5_"
    assert build_cargo_filters(3, 1) == ["t3_interaction_1_"], "Tier 3 feature 1 must be t3_interaction_1_"
    assert build_cargo_filters(4, 2) == ["t4_scenario_2_"], "Tier 4 scenario 2 must be t4_scenario_2_"
    assert build_cargo_filters(1, None) == ["tier1"], "Tier 1 without feature must be tier1"
    assert build_cargo_filters(None, None) == [], "Empty filters when no args given"
    assert build_cargo_filters(None, None, "custom") == ["custom"], "Custom test filter must be preserved"

    # Verify argument parser boundaries (Defect 3)
    parser = argparse.ArgumentParser()
    parser.add_argument("--tier", type=int, choices=[1, 2, 3, 4])
    parser.add_argument("--feature", type=int, choices=range(1, 24), metavar="[1-23]")

    valid_1 = parser.parse_args(["--feature", "1"])
    assert valid_1.feature == 1
    valid_23 = parser.parse_args(["--feature", "23"])
    assert valid_23.feature == 23

    for invalid_val in ["0", "24", "99", "-1"]:
        try:
            parser.parse_args(["--feature", invalid_val])
            assert False, f"Parser should have rejected invalid feature value: {invalid_val}"
        except SystemExit as exc:
            assert exc.code == 2, f"Expected argparse exit code 2, got {exc.code}"

    print("[✓] runner.py self-test passed successfully.")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Unified E2E test runner for noname-next")
    parser.add_argument("--tier", type=int, choices=[1, 2, 3, 4], help="Run tests for specific Tier (1-4)")
    parser.add_argument(
        "--feature",
        type=int,
        choices=range(1, 24),
        metavar="[1-23]",
        help="Run tests for specific feature number (1-23)",
    )
    parser.add_argument("--quick", action="store_true", help="Run fast verification mode")
    parser.add_argument("--tap", action="store_true", help="Print TAP v14 output to stdout")
    parser.add_argument("--report", type=Path, help="Custom output path for target/e2e-report.json")
    parser.add_argument("--filter", dest="test_filter", type=str, help="Cargo test name filter pattern")
    parser.add_argument("--self-test", action="store_true", help="Run runner internal unit tests")

    args, extra_args = parser.parse_known_args()

    if args.self_test:
        return run_self_test()

    repo_root = find_repo_root()
    return run_e2e_suite(
        repo_root=repo_root,
        tier=args.tier,
        feature=args.feature,
        quick=args.quick,
        test_filter=args.test_filter,
        report_path=args.report,
        tap_stdout=args.tap,
        extra_cargo_args=extra_args if extra_args else None,
    )


if __name__ == "__main__":
    sys.exit(main())
