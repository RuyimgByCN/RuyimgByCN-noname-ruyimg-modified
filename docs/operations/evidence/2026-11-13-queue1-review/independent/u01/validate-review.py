"""Validate only this review's evidence structure, hashes, and local links."""
from pathlib import Path
from urllib.parse import unquote
import hashlib
import json
import re
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[5]


def main():
    skills = json.loads((HERE / "skills.json").read_text())
    chars = json.loads((HERE / "characters.json").read_text())
    findings = json.loads((HERE / "findings.json").read_text())
    rows = json.loads((HERE / "historical-row-map.json").read_text())
    checks = []
    assert len(skills) == len({s["skill"] for s in skills}) == len({s["inventory_key"] for s in skills}) == 64
    assert all(all(s[k] for k in ["trigger", "conditions", "choices", "sequence", "native",
                                 "difference", "counterexample", "tests"]) for s in skills)
    checks.append("64 unique skills and real inventory keys, eight populated comparison fields")
    strict = [s for s in skills if s["classification"] == "verified_source_difference"]
    gaps = [s for s in skills if s["classification"] == "potential_gap"]
    assert len(strict) == 62 and len(gaps) == 2
    assert {s["skill"] for s in gaps} == {"dddyuanzhen", "dddqice"}
    assert len(findings["strict_downgrade_candidates"]) == 61 and len(findings["keep_simplified"]) == 1
    assert all(s["recommendation"] is None and not s["change_current_status"] for s in gaps)
    assert all(not s["runtime_verified_by_this_review"] and s["no_equivalence_signoff"] for s in skills)
    checks.append("62 concrete differences, 61 new candidates plus one retained approximation; two gaps excluded")
    assert len(chars) == len({c["name"] for c in chars}) == 31
    assert all(c["core_fields_equal"] and not c["rule_equivalence_signed"] for c in chars)
    assert rows["historical_full_table_rows"] == 95
    mapped = {r["inventory_key"] for row in rows["rows"] for r in row["covered_by"]}
    assert mapped == {s["inventory_key"] for s in skills} | {c["inventory_key"] for c in chars}
    checks.append("31 character core-field comparisons and all 95 old detailed row identifiers mapped")
    for item in json.loads((HERE / "inputs.json").read_text())["inputs"]:
        path = item["path"]
        snapshot = (HERE / "inputs/upstream" / Path(path).name if path.startswith("../archive/")
                    else HERE / "inputs" / path)
        assert hashlib.sha256(snapshot.read_bytes()).hexdigest() == item["sha256"], path
    additional = json.loads((HERE / "inputs-additional.json").read_text())
    for item in additional["inputs"]:
        assert hashlib.sha256((HERE / item["snapshot_path"]).read_bytes()).hexdigest() == item["sha256"]
    checks.append("27 original and 30 additional input snapshots preserve recorded SHA256")
    link_count = 0
    for file in HERE.glob("*.md"):
        for raw in re.findall(r"\]\(([^)]+)\)", file.read_text()):
            if re.match(r"[a-z][a-z+.-]*://", raw) or raw.startswith("#"):
                continue
            target = unquote(raw.split("#", 1)[0]).strip("<>")
            match = re.fullmatch(r"(.*):(\d+)", target)
            line = int(match[2]) if match else None
            target = match[1] if match else target
            path = (file.parent / target).resolve()
            if path == HERE / "validation.json" and line is None:
                # This command creates its own result after all checks succeed.
                link_count += 1
                continue
            assert path.exists(), (file.name, raw)
            if line is not None:
                assert path.is_file() and 1 <= line <= len(path.read_text().splitlines()), (file.name, raw)
            link_count += 1
    checks.append(f"{link_count} top-level report local links and cited line bounds")
    patch = json.loads((HERE / "implementation-inputs.json").read_text())
    live_mismatch = []
    for item in patch["inputs"]:
        assert hashlib.sha256((HERE / item["snapshot_path"]).read_bytes()).hexdigest() == item["sha256"]
        if hashlib.sha256((ROOT / item["path"]).read_bytes()).hexdigest() != item["sha256"]:
            live_mismatch.append(item["path"])
    assert not live_mismatch, live_mismatch
    assert hashlib.sha256((HERE / "implementation.diff").read_bytes()).hexdigest() == patch["diff_sha256"]
    checks.append("21 incremental implementation snapshots match captured hashes and current read-only input")
    result = subprocess.run(["git", "diff", "--check", "--", str(HERE)], cwd=ROOT,
                            capture_output=True, text=True)
    assert result.returncode == 0, result.stdout + result.stderr
    checks.append("git diff --check for owned evidence directory")
    out = dict(date="2026-11-13", command="python3 -B docs/operations/evidence/2026-11-13-queue1-review/independent/u01/validate-review.py",
               checks_passed=len(checks), checks_failed=0, checks=checks,
               local_links=link_count, game_tests_executed=0, cargo_runs=0, browser_runs=0,
               rule_equivalence_signoffs=0)
    (HERE / "validation.json").write_text(json.dumps(out, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(out, ensure_ascii=False))


if __name__ == "__main__":
    main()
