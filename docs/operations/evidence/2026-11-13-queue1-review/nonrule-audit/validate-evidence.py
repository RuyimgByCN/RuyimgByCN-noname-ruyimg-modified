"""Validate this audit's coverage, source references and local evidence links.

These are evidence integrity checks, not product or rule equivalence tests.
"""
import collections
import hashlib
import json
import re
import subprocess
from pathlib import Path

BASE = Path(__file__).resolve().parent
ROOT = Path.cwd()
snapshot = json.loads((BASE / "inventory-snapshot.json").read_text())
findings = json.loads((BASE / "findings.json").read_text())
probes = json.loads((BASE / "consumer-probes.json").read_text())
assert (BASE / "findings.json").read_bytes() == (BASE / "recommendations.json").read_bytes()
expected = set(snapshot["items"])
actual = [r["key"] for r in findings["items"]]
assert set(actual) == expected and len(actual) == len(set(actual)) == 224
assert len([r for r in findings["items"] if r["category"] == "configs"]) == 115
assert len([r for r in findings["items"] if r["category"] == "ui_hooks"]) == 100
assert len([r for r in findings["items"] if r["category"] == "surfaces"]) == 9
allowed = {"已有原生落点", "简化近似", "已复核等价", "缺失"}
line_checks = 0
for row in findings["items"]:
    assert row["status"] in allowed
    assert row["difference"] == row["diff"] and row["note"]
    assert not row["new_equivalence_signoff"] and not row["observed_runtime_failure"]
    assert row["consumer_chain"] and row["evidence_level"]
    for kind in ["upstream_refs", "native_refs"]:
        for pointer in row[kind]:
            file = (ROOT.parent if kind == "upstream_refs" else ROOT) / pointer["path"]
            assert file.is_file(), file
            assert 1 <= pointer["line"] <= len(file.read_text().splitlines()), pointer
            line_checks += 1
    if "upstream_character_offsets" in row:
        pointer = row["upstream_refs"][0]
        text = (ROOT.parent / pointer["path"]).read_text()
        # Babel coordinates count UTF-16 code units.
        encoded = text.encode("utf-16-le")
        start, end = row["upstream_character_offsets"]
        fragment = encoded[start * 2:end * 2].decode("utf-16-le")
        assert row["id"] in fragment, row["key"]

drift = []
for source in probes["scope"]:
    file = ROOT / source["path"]
    if not file.is_file() or hashlib.sha256(file.read_text().encode()).hexdigest() != source["sha256"]:
        drift.append(source["path"])
assert not drift, f"Audited native source changed; coordinate with root before reuse: {drift}"
summary = json.loads((BASE / "upstream-parse-summary.json").read_text())
assert not any(x["parse_errors"] for x in summary["files"])
assert len(summary["commented_items"]) == 12
for source in summary["files"]:
    assert hashlib.sha256((ROOT.parent / source["path"]).read_bytes()).hexdigest() == source["sha256"]

link_checks = 0
whitespace_issues = []
# The report is also a documented local link target; the complete JSON replaces
# this placeholder after all integrity assertions pass.
(BASE / "validation.json").touch(exist_ok=True)
for file in BASE.iterdir():
    if not file.is_file() or file.suffix not in {".md", ".py", ".cjs", ".json"}:
        continue
    for n, line in enumerate(file.read_text().splitlines(), 1):
        if line.rstrip() != line:
            whitespace_issues.append(f"{file.name}:{n}")
    if file.suffix == ".md":
        for target in re.findall(r"\]\(([^)]+)\)", file.read_text()):
            if target.startswith(("https:", "http:", "#")):
                continue
            target = target.strip("<>").split("#")[0]
            if target:
                assert (file.parent / target).is_file(), (file, target)
                link_checks += 1
assert not whitespace_issues, whitespace_issues
result = {
    "evidence_integrity": "passed", "unique_inventory_keys": len(actual),
    "source_file_line_refs_checked": line_checks, "local_markdown_links_checked": link_checks,
    "original_js_files_parsed": len(summary["files"]), "parse_errors": 0,
    "native_source_files_searched": len(probes["scope"]), "native_source_drift": drift,
    "commented_items": len(summary["commented_items"]), "product_tests_run": 0,
    "upstream_browser_triggers_run": 0, "native_runtime_scenarios_run": 0,
    "status_recommendations_applied_to_inventory": False,
    "recommended_status": dict(collections.Counter(r["status"] for r in findings["items"])),
    "git_head_at_validation": subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip(),
    "tracked_dirty_paths_at_validation": subprocess.check_output(["git", "diff", "--name-only"], text=True).splitlines(),
}
(BASE / "validation.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
print(json.dumps(result, ensure_ascii=False))
