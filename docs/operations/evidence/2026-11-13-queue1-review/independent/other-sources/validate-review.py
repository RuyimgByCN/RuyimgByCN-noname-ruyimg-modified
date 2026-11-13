"""Validate the independent review's coverage, provenance, and scoped assertions."""
import hashlib
import json
import math
import random
import re
from collections import Counter
from pathlib import Path

OUT = Path(__file__).resolve().parent
ROOT = OUT.parents[5]


def read(name):
    return json.loads((OUT / name).read_text())


def main():
    report = read("findings.json")
    selected = read("selected-samples.json")
    pointers = read("inventory-source-pointers.json")
    candidates = read("interactive-candidates.json")
    rows = {r["key"]: r for r in report["samples"]}
    expected = {r["key"]: r for data in selected["sources"].values()
                for r in data["random_samples"] + data["interactive_supplements"]}
    assert len(rows) == len(report["samples"]) == len(expected) == 141
    assert set(rows) == set(expected)
    assert set(selected["sources"]) == {"U06", "U07", "L06", "L22", "L29", "L10", "L28", "L27", "L19", "L20", "U08", "L25"}
    assert report["head"] == selected["head"] == "bd58ef06d59940baed8d7f93e61ce571b152fb81"
    assert report["versions"] == {"rules": 333, "ngsa": 107, "content_schema": 5, "abi": 759, "ws": 8}
    for source, pool in selected["sources"].items():
        all_keys = sorted(r["key"] for r in pointers[source])
        count = min(len(all_keys), max(10, math.ceil(len(all_keys) * .1)))
        assert set(random.Random(2026111301).sample(all_keys, count)) == {r["key"] for r in pool["random_samples"]}
        assert len(all_keys) == pool["total_inventory_items"]
        assert Counter(r["category"] for r in pointers[source]) == pool["category_counts"]
        assert all(c["key"] in rows for c in candidates[source]["candidates"])
        assert len(candidates[source]["candidates"]) == (2 if source == "L06" else 0 if source == "L22" else 3)
    ref_count = 0
    for key, row in rows.items():
        for field in ["key", "source_id", "category", "id", "upstream_ref"]:
            assert row[field] == expected[key][field], (key, field)
        for field in ["upstream_checked", "native_checked", "reproduction_or_next_check"]:
            assert row[field].strip(), (key, field)
        assert not row["executed_native_test"] and not row["executed_upstream_browser"]
        assert row["status_boundary"] != "已复核等价"
        assert bool(row["finding"]) == row["difference_confirmed_by_source"]
        assert not row["finding"].startswith("待核验")
        assert row["status_boundary"] == ("简化近似" if row["finding"] else "已有原生落点")
        if row["verification_gap"]:
            assert not row["finding"]
        for ref in row["native_refs"] + row["upstream_refs_reviewed"]:
            match = re.fullmatch(r"(.+):(\d+)(?:-(\d+))?", ref)
            assert match, ref
            name, start, end = match.groups()
            path = ROOT.parent / name if name.startswith(("archive/", "noname-next/")) else ROOT / name
            assert path.exists(), ref
            assert 1 <= int(start) <= int(end or start) <= len(path.read_text().splitlines()), ref
            ref_count += 1
    for name, digest in read("source-hashes.json").items():
        assert hashlib.sha256((ROOT.parent / name).read_bytes()).hexdigest() == digest, name
    proof = read("L29-raw-source-check.json")
    assert hashlib.sha256((ROOT.parent / proof["raw_path"]).read_bytes()).hexdigest() == proof["raw_sha256"]
    assert [(c["id"], c["exact_block_equal"]) for c in proof["checks"]] == [("mbqianlong", True), ("_CMeffects", False)]
    assert report["reviewed_sample_count"] == 141
    assert report["total_inventory_items"] == 765
    assert report["random_sample_count"] == 131 and report["interactive_supplement_count"] == 10
    assert report["accepted_equivalent_count"] == 0
    assert report["executed_browser_count_by_this_reviewer"] == report["executed_native_test_count_by_this_reviewer"] == 0
    totals = Counter(r["status_boundary"] for r in rows.values())
    assert totals == report["status_counts_in_sample"]
    for path in OUT.glob("*.json"):
        json.loads(path.read_text())
    link_count = 0
    for path in OUT.glob("*.md"):
        for link in re.findall(r"\[[^\]]+\]\(([^)]+)\)", path.read_text()):
            assert (path.parent / link.split("#", 1)[0]).exists(), (path.name, link)
            link_count += 1
    print(f"PASS: 12 source denominators, fixed-seed samples, 141 exact keys/categories, 32 interactive candidates, {ref_count} source references and source hashes.")
    print(f"PASS: {totals['简化近似']} confirmed differences; {report['verification_gap_count']} verification gaps; zero browser/Cargo executions or equivalent signoffs claimed.")
    print(f"PASS: all JSON parsed, {link_count} local evidence links resolved, Longxue raw-source provenance boundary retained.")


if __name__ == "__main__":
    main()
