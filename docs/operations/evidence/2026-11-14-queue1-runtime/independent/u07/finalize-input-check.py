"""Only hash/read frozen inputs and check this independent report's local links."""
from datetime import datetime
import hashlib
import json
from pathlib import Path
import re
import subprocess

OWN = Path(__file__).resolve().parent
ROOT = OWN.parents[5]
U07 = OWN.parents[1] / "U07"


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    checks = []
    for manifest_name, expected_count in (
        ("initial-inputs.json", 19),
        ("final-inputs.json", 54),
        ("review-inputs.json", 16),
    ):
        manifest = json.loads((OWN / manifest_name).read_text())
        assert len(manifest["inputs"]) == expected_count
        for row in manifest["inputs"]:
            current_hash = digest(row["path"])
            snapshot_hash = digest(OWN / row["snapshot"]) if row.get("snapshot") else None
            checks.append({"manifest": manifest_name, "path": row["path"],
                           "expected": row["sha256"], "current": current_hash,
                           "snapshot": snapshot_hash})
            assert current_hash == row["sha256"], row["path"]
            assert snapshot_hash in (None, row["sha256"]), row["snapshot"]
    expected_index = "4bf0d611ca74bda600a1f1a3f7006fc75bd244447ad232acff656909762cda7c"
    expected_frozen = "cffbd31ac37c7d5cf0e252094cb33db87545bff2cb34dbc2ee7d4cb4634c0a35"
    assert digest(U07 / "final-input-index.json") == expected_index
    assert digest(U07 / "frozen-file-manifest.json") == expected_frozen
    frozen = json.loads((U07 / "frozen-file-manifest.json").read_text())
    assert len(frozen["sha256"]) == frozen["file_count_excluding_this_manifest"] == 298
    for relative, expected in frozen["sha256"].items():
        assert digest(U07 / relative) == expected, relative
    links = re.findall(r"\[[^]\n]+\]\(([^)\n]+)\)", (OWN / "review.md").read_text())
    for target in links:
        relative = target.split("#")[0]
        if relative != "final-input-check.json":
            assert (OWN / relative).exists(), target
    findings = json.loads((OWN / "findings.json").read_text())
    inventory = json.loads((ROOT / "docs/migration/g0/inventory_status.json").read_text())["items"]
    for item in findings["findings"]:
        for key in [item.get("inventory_key"), *item.get("affected_inventory_keys", [])]:
            assert key is None or key in inventory, key
        for position in [*item.get("upstream", []), *item.get("native", [])]:
            path = position["path"]
            file_path = ROOT.parent / path if path.startswith("archive/") else ROOT / path
            assert 1 <= position["line"] <= len(file_path.read_text().splitlines()), position
    head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    assert head == "605201f3f49e10cea61c31d52a61bbb66c85599d"
    product_status = subprocess.check_output(
        ["git", "status", "--porcelain=v1", "--untracked-files=no", "--",
         "crates", "apps", "packages", "scripts", "content", "fuzz", "tests"],
        cwd=ROOT, text=True)
    output = {"at": datetime.now().astimezone().isoformat(), "head": head,
              "dirty": True, "product_status_observed": product_status,
              "command": "python3 -B docs/operations/evidence/2026-11-14-queue1-runtime/independent/u07/finalize-input-check.py",
              "manifest_input_checks": checks, "manifest_input_changes": 0,
              "snapshot_changes": 0, "frozen_u07_files_checked": 298,
              "frozen_u07_file_changes": 0, "final_index_sha256": expected_index,
              "frozen_u07_manifest_sha256": expected_frozen,
              "local_report_links_checked": len(links), "bad_links": [],
              "findings_inventory_keys_exist": True, "source_line_bounds_valid": True,
              "cargo_or_browser_rerun": False, "passed": True}
    destination = OWN / "final-input-check.json"
    assert not destination.exists()
    destination.write_text(json.dumps(output, ensure_ascii=False, indent=2) + "\n")
    assert all((OWN / target.split("#")[0]).exists() for target in links)
    print(json.dumps({key: output[key] for key in (
        "head", "manifest_input_changes", "snapshot_changes", "frozen_u07_files_checked",
        "frozen_u07_file_changes", "local_report_links_checked", "passed")}, ensure_ascii=False))


if __name__ == "__main__":
    main()
