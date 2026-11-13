#!/usr/bin/env python3
"""Keep the formal registration run separate from the bridge-only cohort."""
import collections
import datetime
import hashlib
import json
from pathlib import Path
import re
import subprocess

OWN = Path(__file__).resolve().parent
REPO = OWN.parents[5]
RAW = OWN.parents[1] / "L10"
FORMAL = OWN / "formal-run01"
FOLLOW = OWN / "formal-consumption02"


def read(path):
    return json.loads(path.read_text())


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def dump(name, value):
    p = OWN / name
    assert not p.exists(), p
    p.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n")


def normalize_debug(value):
    return re.sub(r"auth_generation: \d+", "auth_generation: NORMALIZED", value)


def check_run(base, row):
    name = row["case"]
    d, r = row["direct"], row["restored"]
    assert d["initial"] == r["initial"]
    assert d["commands"] == r["commands"]
    assert d["final"] == r["final"]
    if "stages" in d:
        assert d["stages"] == r["stages"]
    count = 0
    for mode in ("direct", "restored"):
        run = row[mode]
        for i, c in enumerate(run["pending_archive_checks"]):
            data = (base / c["file"]).read_bytes()
            assert data[:4] == b"NGSA" and int.from_bytes(data[4:6], "little") == 107
            assert len(data) == c["bytes"] and c["error"] is None
            assert c["state_hash_identical"] and c["snapshot_equal_except_auth_generation"]
            assert not c["roundtrip_bytes_identical"]
            assert c["auth_after"] == c["auth_before"] + 1
            assert c["auth_before"] == (i if mode == "restored" else 0)
            count += 1
        assert (base / f"{name}-{mode}-final.ngsa").read_bytes().hex() == run["final_archive_hex"]
    ds = (base / f"{name}-direct-final-snapshot-debug.txt").read_text()
    rs = (base / f"{name}-restored-final-snapshot-debug.txt").read_text()
    assert normalize_debug(ds) == normalize_debug(rs)
    return count


old = read(RAW / "native-round2/native-results.json")
formal = read(FORMAL / "native-results.json")
assert len(formal) == 26 and [x["case"] for x in old] == [x["case"] for x in formal]
assert (RAW / "native-probe/src/main.rs").read_text().split("fn main(){")[0] == (OWN / "formal-probe-round1-source.rs").read_text().split("fn main(){")[0]
registration = read(FORMAL / "formal-registration.json")
catalog = read(FORMAL / "formal-catalog.json")
assert len(registration["host_skills"]) == 4 and len(registration["vm_skills"]) == 10
assert all(row["id"] is not None for row in catalog["companion_lookup"])
comparisons = []
for before, after in zip(old, formal):
    pending = check_run(FORMAL, after)
    a = {k: v for k, v in before["direct"]["final"].items() if k != "hash"}
    b = {k: v for k, v in after["direct"]["final"].items() if k != "hash"}
    comparisons.append({"case": after["case"], "pending_archives": pending,
                        "same_final_capture_except_content_bound_hash": a == b,
                        "changed_fields": [k for k in a if a[k] != b[k]],
                        "bridge_skills": before["direct"]["final"]["players"][0]["skills"],
                        "formal_skills": after["direct"]["final"]["players"][0]["skills"],
                        "formal_pending_at_stop": after["direct"]["final"]["choice"],
                        "direct_restored_final_and_commands_equal": True,
                        "direct_restored_full_debug_equal_except_auth_generation": True})
assert sum(x["pending_archives"] for x in comparisons) == 102
assert [x["case"] for x in comparisons if not x["same_final_capture_except_content_bound_hash"]] == ["k_A", "k_B"]

followups = read(FOLLOW / "formal-followup-results.json")
assert [x["case"] for x in followups] == ["k_A_followup", "k_B_followup"]
continuations = []
for row in followups:
    pending = check_run(FOLLOW, row)
    d = row["direct"]
    assert len(d["commands"]) == 4
    assert all("NextPhase" in c["request"] for c in [d["commands"][0], *d["commands"][2:]])
    assert "Answer" in d["commands"][1]["request"]
    assert all(c["after"]["choice"] is None for c in d["commands"][1:])
    hp1 = [s["state"]["players"][1]["hp"] for s in d["stages"]]
    assert hp1 == ([4, 3, 2] if row["case"] == "k_A_followup" else [4, 3, 1])
    debug = (FOLLOW / f"{row['case']}-direct-final-snapshot-debug.txt").read_text()
    flags = sorted(set(re.findall(r'"(l10_scs[^"\n]+)"', debug)))
    continuations.append({"case": row["case"], "pending_archives": pending,
                          "hp_seat1_after_grant_draw_play": hp1,
                          "actual_stages": d["stages"], "commands_and_logs": d["commands"],
                          "l10_scs_flag_names_in_full_final_snapshot": flags,
                          "only_answer_is_clique_partner": True,
                          "no_active_skill_or_card_use_command": True,
                          "no_native_choice_after_two_normal_nextphase_commands": True,
                          "direct_restored_commands_stages_final_equal": True,
                          "upstream_runtime_continuation_not_executed_due_to_k_ui_block": True})
assert sum(x["pending_archives"] for x in continuations) == 4

freeze = read(OWN / "formal-input-freeze-02.json")
product = [r for r in freeze["files"] if r["path"].startswith("crates/")]
assert len(product) == 474
changed = [r["path"] for r in product if sha(REPO / r["path"]) != r["sha256"]]
assert not changed
input_records = []
extra_source_files = [
    REPO.parent / "archive/upstream/noname-liuli/noname/character/mobile.js",
    REPO.parent / "archive/upstream/noname-liuli/noname/game/game.js",
    REPO.parent / "archive/upstream/noname-liuli/noname/card/standard.js",
    REPO / "crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_flags.rs",
    REPO / "crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34.rs",
    REPO / "crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_ext.rs",
    OWN / "formal-probe/Cargo.lock",
]
for p in extra_source_files:
    input_records.append({"path": str(p), "bytes": p.stat().st_size, "sha256": sha(p)})
summary = {"at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
           "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True).strip(),
           "route": registration["route"], "extra_sources_read": input_records,
           "formal_metadata": read(FORMAL / "native-input-metadata.json"),
           "original_bridge_metadata": read(RAW / "native-round2/native-input-metadata.json"),
           "formal_original_cases": len(formal), "formal_pending_archives_checked": 102,
           "formal_direct_restored_complete_debug_equal_except_auth_cases": 26,
           "formal_only_skill_list_change_at_clique_selection": ["k_A", "k_B"],
           "formal_companion_catalog": catalog["companion_lookup"],
           "extra_continuation_cases": len(continuations), "extra_continuation_pending_archives_checked": 4,
           "reviewed_product_rs_files_unchanged_after_build_run": len(product),
           "product_rs_changed": changed,
           "comparison_boundary": "The formal run uses existing registration, without manifest validation, installation, all-package coexistence, browser-side dependency stubs, or injected entities. Upstream full clique/continuation is statically reviewed, not dynamically completed.",
           "new_equivalence_signed": 0}
dump("formal-case-recalculation.json", comparisons)
dump("formal-continuation-recalculation.json", continuations)
dump("formal-summary-recalculation.json", summary)
print(json.dumps({"formal_cases": 26, "formal_pending": 102, "followup_cases": 2,
                  "followup_pending": 4, "product_rs_unchanged": len(product)}, ensure_ascii=False))
