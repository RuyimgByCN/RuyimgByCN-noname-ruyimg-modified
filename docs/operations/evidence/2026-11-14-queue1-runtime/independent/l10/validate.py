#!/usr/bin/env python3
"""Validate only this independent review and its frozen raw inputs."""
import datetime
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess

OWN = Path(__file__).resolve().parent
REPO = OWN.parents[5]
RAW = OWN.parents[1] / "L10"


def read(path):
    return json.loads(path.read_text())


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


report_path = OWN / "validation-first.json"
# Reserve this invocation's output before checking links to that output.
with report_path.open("x") as output:
    initial = read(OWN / "inputs-initial.json")
    changed_initial = [row["path"] for row in initial["files"]
                       if not (REPO / row["path"]).is_file()
                       or sha(REPO / row["path"]) != row["sha256"]]
    assert not changed_initial, changed_initial
    frozen = read(RAW / "frozen-manifest.json")
    assert len(frozen["files"]) == frozen["file_count"] == 318
    broken_raw = []
    for row in frozen["files"]:
        path = RAW / row["path"]
        if not path.is_file() or path.stat().st_size != row["bytes"] or sha(path) != row["sha256"]:
            broken_raw.append(row["path"])
    assert not broken_raw, broken_raw
    assert sum(row["bytes"] for row in frozen["files"]) == frozen["bytes"] == 47114568
    freeze2 = read(OWN / "formal-input-freeze-02.json")
    source = [r for r in freeze2["files"] if r["path"].startswith("crates/")]
    assert len(source) == 474
    source_changed = [r["path"] for r in source if sha(REPO / r["path"]) != r["sha256"]]
    assert not source_changed, source_changed
    spec = importlib.util.spec_from_file_location("repo_local_links", REPO / "scripts/check_local_links.py")
    links = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(links)
    markdown = sorted(OWN.rglob("*.md"))
    bad, oob, count, line_count = [], [], 0, 0
    for path in markdown:
        bad.extend({"doc": str(path.relative_to(OWN)), "href": href} for href in links.check(path))
        for match in links.LINK.finditer(path.read_text()):
            href = match[1]
            if href.startswith(("http://", "https://", "mailto:", "#")):
                continue
            count += 1
            target, _, fragment = href.partition("#")
            actual = (path.parent / target).resolve()
            assert actual.exists(), (path, href)
            line = re.fullmatch(r"L(\d+)", fragment)
            if line:
                line_count += 1
                at = int(line[1])
                if not actual.is_file() or not 0 < at <= len(actual.read_text(errors="replace").splitlines()):
                    oob.append({"doc": str(path.relative_to(OWN)), "href": href})
    assert not bad and not oob, (bad, oob)
    anchors = read(OWN / "source-anchors.json")
    for row in anchors:
        assert (REPO / row["path"]).read_text(errors="replace").splitlines()[row["line"] - 1] == row["text"]
    cases = read(OWN / "case-assessments.json")
    assert len(cases) == len({r["case"] for r in cases}) == 26
    assert sum(r["native_pending_archives_direct_plus_restored"] for r in cases) == 102
    assert sum(r["assessment"] == "已观察到结算反例" for r in cases) == 8
    assert sum(r["upstream_root_done"] for r in cases) == 14
    original_pending = len(list((RAW / "native-round2").glob("*-pending-*.ngsa")))
    replay_pending = len(list((OWN / "native-replay01").glob("*-pending-*.ngsa")))
    formal_pending = len(list((OWN / "formal-run01").glob("*-pending-*.ngsa")))
    follow_pending = len(list((OWN / "formal-consumption02").glob("*-pending-*.ngsa")))
    assert original_pending == replay_pending == formal_pending == 102 and follow_pending == 4
    staged = subprocess.check_output(["git", "diff", "--cached", "--name-only", "--", str(OWN)], cwd=REPO, text=True)
    assert not staged, staged
    report = {"at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
              "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True).strip(),
              "scope": "Only independent/l10 Markdown and declared source/raw inputs; not global doc gates or product acceptance",
              "command": "python3 -B docs/operations/evidence/2026-11-14-queue1-runtime/independent/l10/validate.py",
              "markdown_files": len(markdown), "local_links": count, "line_anchors": line_count,
              "broken_links": bad, "out_of_bounds_line_anchors": oob,
              "semantic_source_anchor_texts_unchanged": len(anchors),
              "initial_declared_inputs_unchanged": len(initial["files"]),
              "raw_frozen_manifest_sha256": sha(RAW / "frozen-manifest.json"),
              "raw_frozen_files_unchanged": frozen["file_count"], "raw_frozen_bytes": frozen["bytes"],
              "raw_author_explanation_files": "Checksum only; README/conclusions/comparison table/comparator not interpreted",
              "reviewed_native_product_rs_files_unchanged": len(source),
              "unique_cases": len(cases), "original_pending_archives": original_pending,
              "independent_replay_pending_archives": replay_pending,
              "formal_pending_archives": formal_pending, "extra_formal_continuation_pending_archives": follow_pending,
              "new_equivalence_signed": 0, "owned_files_staged": False,
              "product_status_after": subprocess.check_output(["git", "status", "--short", "--", "crates", "apps", "packages", "scripts", "content"], cwd=REPO, text=True).splitlines(),
              "private_target_created": False, "shared_target_cleaned": False,
              "product_tests_or_full_preflight_run": False, "result": "pass"}
    json.dump(report, output, ensure_ascii=False, indent=2)
    output.write("\n")
print(json.dumps({"result": "pass", "markdown_files": len(markdown), "local_links": count,
                  "line_anchors": line_count, "broken_links": len(bad), "out_of_bounds": len(oob),
                  "initial_inputs_unchanged": len(initial["files"]), "raw_frozen_files_unchanged": 318,
                  "product_rs_unchanged": len(source), "unique_cases": 26, "pending_original_replay_formal_followup": [102, 102, 102, 4]}, ensure_ascii=False))
