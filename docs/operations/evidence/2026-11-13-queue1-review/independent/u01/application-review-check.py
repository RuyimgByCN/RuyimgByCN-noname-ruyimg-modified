"""Read-only recomputation; writes only this review's new prefixed evidence files."""
import collections
import datetime
import hashlib
import json
import pathlib
import subprocess
from zoneinfo import ZoneInfo

HERE = pathlib.Path(__file__).resolve().parent
BASE = HERE.parents[1]
ROOT = BASE.parents[3]
STAMP = datetime.datetime.now(ZoneInfo("Asia/Taipei")).isoformat()


def read(path):
    return json.loads(path.read_text())


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def lines(path):
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def time(value):
    return datetime.datetime.fromisoformat(value.replace("Z", "+00:00"))


checks = []


def check(name, condition):
    checks.append({"check": name, "pass": bool(condition)})
    assert condition, name


def write(suffix, value):
    with (HERE / ("application-review-" + suffix + ".json")).open("x") as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write("\n")


invpath = "docs/migration/g0/inventory_status.json"
headbytes = subprocess.check_output(["git", "show", "HEAD:" + invpath], cwd=ROOT)
head = json.loads(headbytes)
current = read(ROOT / invpath)
app = read(BASE / "inventory-application.json")
changes = {c["key"]: c for c in app["changes"]}
skills = read(HERE / "skills.json")
characters = read(HERE / "characters.json")
findings = read(HERE / "findings.json")
u01keys = {r["inventory_key"] for r in skills + characters}
check("before inventory SHA equals HEAD bytes", hashlib.sha256(headbytes).hexdigest() == app["before_sha256"])
check("exactly 95 U01 keys", len(u01keys) == 95 and u01keys == {k for k in current["items"] if k.startswith("U01:")})
check("application contains all U01 keys", u01keys <= changes.keys())
check("all 378 application before records equal HEAD", len(changes) == 378 and all(c["before"] == head["items"][k] for k, c in changes.items()))
check("current U01 equals recorded after", all(current["items"][k] == changes[k]["after"] for k in u01keys))
withdrawn = {k for k in u01keys if head["items"][k]["status"] != current["items"][k]["status"]}
check("exact 61 strict U01 downgrades", withdrawn == set(findings["strict_downgrade_candidates"]) and len(withdrawn) == 61)
metadata = ["status", "reviewer", "review_date", "upstream_ref", "upstream_file_line", "test_name", "native", "target", "source_id", "category", "id"]
preserved = {r["inventory_key"] for r in characters} | {r["inventory_key"] for r in skills if r["classification"] == "potential_gap"}
check("33 retained records preserve original status and metadata", len(preserved) == 33 and all(all(current["items"][k].get(f) == head["items"][k].get(f) for f in metadata) for k in preserved))
check("all U01 old notes retained as prefix", all(not head["items"][k].get("note") or current["items"][k]["note"].startswith(head["items"][k]["note"]) for k in u01keys))
anzhi = next(r["inventory_key"] for r in skills if r["skill"] == "dddanzhi")
check("dddanzhi remains approximate with old reviewer/date", all(current["items"][anzhi][f] == head["items"][anzhi][f] for f in ["status", "reviewer", "review_date"]) and current["items"][anzhi]["status"] == "简化近似")
check("current global counts equal actual item statuses", all(current["counts"][k] == sum(r["status"] == k for r in current["items"].values()) for k in current["counts"]))
check("no U01 new equivalence signoffs", not any(head["items"][k]["status"] != "已复核等价" and current["items"][k]["status"] == "已复核等价" for k in u01keys))
record_rows = []
for r in skills + characters:
    k = r["inventory_key"]
    old, new = head["items"][k], current["items"][k]
    record_rows.append({"key": k, "name": r.get("skill", r.get("name")), "classification": r.get("classification", "core_fields_only"), "before_status": old["status"], "current_status": new["status"], "after_matches_application": new == changes[k]["after"], "changed_fields": [f for f in new if old.get(f) != new[f]], "current_difference": new.get("difference"), "current_reviewer": new.get("reviewer"), "current_review_date": new.get("review_date")})
non_u01_after_changes = [k for k, c in changes.items() if k not in u01keys and current["items"][k] != c["after"]]
write("inventory", {"at": STAMP, "u01_records": record_rows, "u01_status_counts": dict(collections.Counter(current["items"][k]["status"] for k in u01keys)), "current_inventory_sha256": sha(ROOT / invpath), "original_application_after_sha256": app["after_sha256"], "current_global_counts_for_identification_only": current["counts"], "non_u01_after_changes_not_semantically_reviewed": non_u01_after_changes})

# Reimplement the limited projection without importing or executing root's comparator.
aliases = {"horse_minus": "chitu", "wxl_wuxingpan_0": "wuxingpan"}
elements = {"metal": 0, "wood": 1, "water": 2, "fire": 3, "soil": 4}


def cards(values):
    out = []
    for value in values:
        element = value.get("wunature")
        out.append((aliases.get(value["name"], value["name"]), value["suit"].lower(), value["rank"], -1 if element is None else elements.get(element, element)))
    return sorted(out)


def projection(value, native, modern):
    out = []
    for p in value["players"]:
        v = [p["seat"], p["hp"], cards(p["hand"]), cards(p["equips"])]
        if modern:
            v += [bool(p["juxian_ban"] if native else p["juxianBan"]), bool(p["xuanlun_triggered"] if native else p["xuanlunChoice"])]
        out.append(v)
    return out


def branch(row):
    label = row["case"]
    for prefix in ("exact-", "ordered-", "blind-repeat-1-"):
        if label.startswith(prefix):
            label = label[len(prefix):]
            break
    if label.endswith("-final"):
        label = label[:-6]
    if label in ("juxian-top-black", "juxian-top-red"):
        receipt = row["result"]["players"][0]["hand"]
        assert len(receipt) == 1
        face = (receipt[0]["name"], receipt[0]["suit"], receipt[0]["rank"])
        assert face in (("sha", "club", 7), ("tao", "heart", 3))
        label = "juxian-top-" + ("red" if face == ("tao", "heart", 3) else "black")
    return label


native_rows = read(BASE / "native-comparison-final-results.json")
native = {r["case"]: r for r in native_rows}
native_log = [json.loads(line) for line in (BASE / "native-comparison-final.log").read_text().splitlines() if line.startswith("{")]
raw = lines(BASE / "browser/case-results.jsonl")
check("native 13 rows unique and equal raw command log", len(native) == len(native_rows) == 13 and native_log == native_rows)
check("all 27 recorded native pending checks succeeded", sum(len(r["choices"]) for r in native_rows) == 27 and all(c["restored_before_and_after_hash_equal"] for r in native_rows for c in r["choices"]))
check("native comparison flags are actual 0/1", all(p[f] in (0, 1) for r in native_rows for p in r["players"] for f in ("juxian_ban", "xuanlun_triggered")))
check("green and final actual fixture lists identical", [r["fixtures"] for r in read(BASE / "native-comparison-green-first-results.json")] == [r["fixtures"] for r in native_rows])

cdp = []
requests = {}
for path in (BASE / "browser").glob("*-replies.jsonl"):
    reqpath = path.with_name(path.name.replace("-replies", "-requests"))
    reqs = lines(reqpath)
    replies = lines(path)
    assert len(reqs) == len(replies)
    requests[path.name] = reqs
    for i, reply in enumerate(replies):
        assert reqs[i]["at"] == reply["at"]
        value = reply.get("result", {}).get("result", {}).get("value")
        if isinstance(value, str):
            try:
                value = json.loads(value)
            except json.JSONDecodeError:
                continue
        if isinstance(value, dict) and value.get("done") is True and "players" in value:
            cdp.append({"file": path.name, "line": i + 1, "at": reply["at"], "value": value})

allrows = []
used = set()
for number, row in enumerate(raw, 1):
    matches = [c for c in cdp if c["value"] == row["result"] and time(c["at"]) <= time(row["at"])]
    assert matches
    match = max(matches, key=lambda c: time(c["at"]))
    assert (match["file"], match["line"]) not in used
    used.add((match["file"], match["line"]))
    label = row["case"]
    reviewed = label.startswith(("exact-", "ordered-", "blind-repeat-1-", "wuxingpan-")) or label in ("shengke-sha-final", "shengke-tao-final")
    original = reviewed and not label.startswith("exact-")
    key = branch(row) if reviewed else None
    equality = projection(native[key], True, row["engine"] == "modern") == projection(row["result"], False, row["engine"] == "modern") if reviewed else None
    fixture_match = None
    start_hash = None
    if reviewed and row["engine"] == "modern":
        reqs = requests[match["file"]][:match["line"]]
        setters = [r["expression"] for r in reqs if r["expression"].startswith("window.queue1Fixture=")]
        fixture, _ = json.JSONDecoder().raw_decode(setters[-1].split("=", 1)[1])
        fixture_match = fixture == native[key]["fixtures"]
        starts = [r["expression"] for r in reqs if "_status.eventManager = new EventManager();" in r["expression"]]
        start_hash = hashlib.sha256(starts[-1].encode()).hexdigest()
    allrows.append({"raw_line": number, "case": label, "engine": row["engine"], "recorded_at": row["at"], "cdp_reply_file": match["file"], "cdp_reply_line": match["line"], "cdp_at": match["at"], "cohort": "reviewed" if reviewed else "exploratory_different_fixture", "original16_member": original, "receipt_branch": key, "projection_equal": equality, "fixture_faces_equal_final_native": fixture_match, "actual_browser_start_sha256": start_hash})
check("all 31 rich result records match distinct prior CDP replies", len(raw) == len(allrows) == len(cdp) == len(used) == 31)
check("reviewed cohort 24 of 31 with 7 explorations retained", sum(r["cohort"] == "reviewed" for r in allrows) == 24 and sum(r["cohort"] != "reviewed" for r in allrows) == 7)
check("all reviewed24 projections equal", all(r["projection_equal"] for r in allrows if r["cohort"] == "reviewed"))
check("all original16 projections equal", sum(r["original16_member"] for r in allrows) == 16 and all(r["projection_equal"] for r in allrows if r["original16_member"]))
check("all 19 modern reviewed fixture face objects match native", sum(r["fixture_faces_equal_final_native"] is True for r in allrows) == 19)
for file, original in (("runtime-comparison-final.json", True), ("runtime-comparison-reviewed.json", False)):
    summary = read(BASE / file)
    selected = [r for r in allrows if r["original16_member"]] if original else [r for r in allrows if r["cohort"] == "reviewed"]
    recorded = [(u["raw_case_label"], u["at"], r["case"]) for r in summary["results"] for u in r["upstream_repeats"]]
    recomputed = [(r["case"], r["recorded_at"], r["receipt_branch"]) for r in selected]
    check(file + " branch/sample manifest exactly matches independent grouping", sorted(recorded) == sorted(recomputed) and len(summary["results"]) == 13 and summary["compared_upstream_runs"] == len(selected))
    check(file + " result projections match independent computation", all(projection({"players": [{**p, "xuanlun_triggered": p.get("xuanlun_choice")} for p in r["players"]]}, True, r["engine"] == "modern") == projection(native[r["case"]], True, r["engine"] == "modern") for r in summary["results"]))
write("runtime", {"at": STAMP, "raw_rich_records": allrows, "reviewed_counts_by_branch": dict(collections.Counter(r["receipt_branch"] for r in allrows if r["cohort"] == "reviewed")), "old16_preserved": True, "reviewed24_equal": True, "reviewer_cargo_runs": 0, "reviewer_browser_runs": 0, "full_event_rng_or_skill_equivalence": False})

frozen = read(BASE / "frozen-product-inputs.json")
drift = [p for p, expected in frozen["files"].items() if not (ROOT / p).is_file() or sha(ROOT / p) != expected]
check("all 32259 frozen product inputs still match", len(frozen["files"]) == 32259 and not drift)
setup = read(BASE / "browser/modern-setup.json")
temp, source = pathlib.Path(setup["temporary_workspace"]), pathlib.Path(setup["source"])
upstream_drift = [p for p in setup["copied_files"] if not (temp / p).is_file() or (temp / p).read_bytes() != (source / p).read_bytes()]
check("850 copied modern upstream inputs match actual source", len(setup["copied_files"]) == 850 and not upstream_drift)
fixturepath = "crates/engine-native/tests/fixtures/u01-dddanzhi-pending-v107.ngsa"
oldfixture = subprocess.check_output(["git", "show", "HEAD:" + fixturepath], cwd=ROOT)
check("existing dddanzhi107 fixture unchanged from HEAD", (ROOT / fixturepath).read_bytes() == oldfixture)
check("native probe source matches recorded cleanup SHA", sha(BASE / "native-comparison-final-probe.rs") == read(BASE / "probe-final-source-and-cleanup.json")["source_sha256"])

inputpaths = [ROOT / invpath, pathlib.Path(__file__), BASE / "inventory-application.json", BASE / "inventory-correction-01.json", BASE / "frozen-product-inputs.json", BASE / "compare-runtime-final.py", BASE / "runtime-comparison-final.json", BASE / "runtime-comparison-reviewed.json", BASE / "native-comparison-final-results.json", BASE / "native-comparison-final.json", BASE / "native-comparison-final.log", BASE / "native-comparison-final-probe.rs", BASE / "native-comparison-green-first-results.json", BASE / "probe-final-source-and-cleanup.json", ROOT / fixturepath]
inputpaths += [HERE / n for n in ("skills.json", "characters.json", "findings.json")]
inputpaths += [BASE / "browser" / n for n in ("case-results.jsonl", "run-u01-exact.py", "modern-u01-start.js", "modern-choice.js", "modern-finish.js", "liuli-shengke-start.js", "liuli-wuxingpan-start.js", "liuli-choice.js", "liuli-finish.js", "modern-setup.json", "active-session-modern.json", "launch.mjs")]
inputpaths += list((BASE / "browser").glob("*-requests.jsonl")) + list((BASE / "browser").glob("*-replies.jsonl"))
inputpaths += [ROOT / r["path"] for r in read(HERE / "j06-followup-inputs.json")["inputs"] if r["scope"] == "requested_patch"]
inputpaths += [source / n for n in ("apps/core/extension/3D精选/character/skill.js", "apps/core/noname/library/element/content.ts", "apps/core/card/standard.js", "apps/core/noname/library/element/player.js")]
write("inputs", {"captured_at": STAMP, "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(), "tracked_dirty": bool(subprocess.check_output(["git", "status", "--porcelain", "--untracked-files=no"], cwd=ROOT)), "versions": app["versions"], "inputs": [{"path": str(p), "sha256": sha(p), "bytes": p.stat().st_size} for p in sorted(set(inputpaths))], "product_frozen_hashes_checked": len(frozen["files"]), "product_frozen_mismatches": drift, "copied_modern_source_files_checked": len(setup["copied_files"]), "copied_modern_source_mismatches": upstream_drift})
write("checks", {"at": STAMP, "command": "python3 -B " + str(pathlib.Path(__file__)), "checks": checks, "passed": len(checks), "failed": 0, "reviewer_cargo_runs": 0, "reviewer_browser_runs": 0})
print(json.dumps({"passed": len(checks), "failed": 0, "U01_records": 95, "U01_withdrawals": 61, "upstream_reviewed_runs": 24, "native_pending_restores_read": 27, "product_hashes_checked": len(frozen["files"])}))
