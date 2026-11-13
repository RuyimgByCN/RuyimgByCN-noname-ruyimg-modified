#!/usr/bin/env python3
"""Read only the L10 raw cohort and independently recalculate review facts."""
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
UPSTREAM = REPO.parent / "archive/upstream/noname-liuli/noname"
REPLAY = OWN / "native-replay01"


def read(path):
    return json.loads(path.read_text())


def write(name, value):
    path = OWN / name
    if path.exists():
        raise RuntimeError("Refusing to replace prior evidence: " + str(path))
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n")


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def canonical(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def face(c):
    return {"id": c.get("id"), "name": c.get("upstream_name", c.get("name")),
            "suit": c["suit"], "rank": c["rank"], "nature": c.get("nature")}


def cards(rows, ordered=False):
    rows = [face(c) for c in rows]
    return rows if ordered else sorted(rows, key=canonical)


def zones(state, native, excluded):
    players = []
    for p in state["players"]:
        q = {k: p[k] for k in ("seat", "hp", "max_hp", "face_down")}
        q.update({k: cards(p[k]) for k in ("hand", "equips", "judge")})
        players.append(q)
    return {"players": players,
            "deck_top_first": cards(state["deck_top_first" if native else "deck"], True),
            "discard_projection": cards([c for c in state["discard"] if c.get("id") not in excluded]),
            "table_or_ordering": cards(state["table" if native else "ordering"])}


def differences(a, b, prefix=""):
    if a == b:
        return []
    if isinstance(a, dict) and isinstance(b, dict):
        result = []
        for key in sorted(set(a) | set(b)):
            result += differences(a.get(key), b.get(key), prefix + "/" + str(key))
        return result
    # Array card zones are reported whole, retaining the actual differing faces.
    return [{"field": prefix, "native": a, "upstream": b}]


native = read(RAW / "native-round2/native-results.json")
browser = read(RAW / "browser-round4/results.json")
names = [row["case"] for row in native]
assert len(names) == len(set(names)) == 26
assert names == [row["case"] for row in browser]
assert native == read(REPLAY / "native-results.json")

requests = [json.loads(s) for s in (RAW / "session-l10-1794593358863-requests.jsonl").read_text().splitlines()]
replies = [json.loads(s) for s in (RAW / "session-l10-1794593358863-replies.jsonl").read_text().splitlines()]
assert len(requests) == len(replies) == 595
assert all(a["at"] == b["at"] for a, b in zip(requests, replies))
values = []
for reply in replies:
    v = reply.get("result", {}).get("result", {}).get("value")
    if isinstance(v, str):
        try:
            v = json.loads(v)
        except json.JSONDecodeError:
            pass
    values.append(v)

setups = []
for index, request in enumerate(requests):
    match = re.fullmatch(r'window\.l10Case=(".*?");window\.l10Fixture=(.*);true', request["expression"], re.S)
    if match:
        setups.append((index, json.loads(match[1]), json.loads(match[2])))
last = {name: (index, fixture) for index, name, fixture in setups if name in names}
assert [name for _, name in sorted((last[name][0], name) for name in names)] == names
assert len([x for x in setups if x[0] >= last[names[0]][0]]) == 26
setup_body = (RAW / "browser-round4-setup.js").read_text()
assert setup_body == (RAW / "upstream-setup.js").read_text()
loader = (RAW / "load-source.js").read_text()

streams = {}
for mode in ("direct", "restored"):
    rows = [json.loads(s) for s in (RAW / f"native-round2/native-{mode}-commands.jsonl").read_text().splitlines()]
    streams[mode] = collections.defaultdict(list)
    for row in rows:
        assert row["step"] == len(streams[mode][row["case"]])
        streams[mode][row["case"]].append(row["data"])

cases, cdp_cases, archives, blockers = [], [], [], []
for position, (n, u) in enumerate(zip(native, browser)):
    name = n["case"]
    assert n == read(RAW / f"native-round2/native-{name}.json")
    assert u == read(RAW / f"browser-round4/upstream-{name}.json")
    d, r = n["direct"], n["restored"]
    assert d["initial"] == r["initial"] == u["native_fixture"]
    assert d["commands"] == r["commands"]
    assert d["final"] == r["final"]
    for mode in ("direct", "restored"):
        run = n[mode]
        assert run["commands"] == streams[mode][name]
        for stage, check in enumerate(run["pending_archive_checks"]):
            path = RAW / "native-round2" / check["file"]
            data = path.read_bytes()
            assert data[:4] == b"NGSA" and int.from_bytes(data[4:6], "little") == 107
            assert len(data) == check["bytes"]
            assert check["error"] is None
            assert check["roundtrip_bytes_identical"] is False
            assert check["state_hash_identical"] is True
            assert check["snapshot_equal_except_auth_generation"] is True
            assert check["auth_after"] == check["auth_before"] + 1
            assert check["auth_before"] == (stage if mode == "restored" else 0)
            assert data == (REPLAY / path.name).read_bytes()
            archives.append({"case": name, "mode": mode, "stage": stage,
                             "file": str(path.relative_to(REPO)), "sha256": digest(path), **check})
        final_path = RAW / f"native-round2/{name}-{mode}-final.ngsa"
        assert final_path.read_bytes().hex() == run["final_archive_hex"]
    direct_debug = (RAW / f"native-round2/{name}-direct-final-snapshot-debug.txt").read_text()
    restored_debug = (RAW / f"native-round2/{name}-restored-final-snapshot-debug.txt").read_text()
    auth = lambda s: re.sub(r"auth_generation: \d+", "auth_generation: NORMALIZED", s)
    assert auth(direct_debug) == auth(restored_debug)
    assert n["final_ngsa_bytes_equal"] == (d["final_archive_hex"] == r["final_archive_hex"])

    start, fixture = last[name]
    end = last[names[position + 1]][0] if position + 1 < len(names) else len(requests)
    assert fixture == d["initial"]
    assert requests[start + 1]["expression"] == setup_body
    source_index = max(i for i in range(start) if requests[i]["expression"] == loader)
    assert values[source_index] == u["source_load"]
    snapshot_lines = collections.defaultdict(list)
    for i in range(start, end):
        snapshot_lines[canonical(values[i])].append(i + 1)
    if "start" in u:
        assert values[start + 1] == u["start"]
    initial_candidates = snapshot_lines[canonical(u["initial"])]
    if "start" in u:
        assert u["initial"] == values[start + 1]["initial"]
        initial_candidates = [start + 2]
    assert initial_candidates
    final_lines = snapshot_lines[canonical(u["final"])]
    assert final_lines
    stage_lines = []
    for stage in u["stages"]:
        lines = snapshot_lines[canonical(stage["snapshot"])]
        assert lines, (name, stage["label"])
        stage_lines.append({"label": stage["label"], "reply_lines": lines,
                            "choice": stage["snapshot"]["choice"],
                            "done": stage["snapshot"]["done"], "stack": stage["snapshot"]["stack"]})
    actions, exceptions = [], []
    for i in range(start + 1, end):
        exp = requests[i]["expression"]
        details = replies[i].get("result", {}).get("exceptionDetails")
        if details:
            exceptions.append({"request_reply_line": i + 1, "at": requests[i]["at"],
                               "expression_kind": "setup" if exp == setup_body else exp,
                               "raw_reply": replies[i]})
        if re.search(r"ui\.click\.(button|skill|card|control|target|ok|cancel)", exp):
            actions.append({"request_reply_line": i + 1, "at": requests[i]["at"],
                            "expression": exp, "reply_value": values[i],
                            "exception": details})
    cdp_cases.append({"case": name, "setup_request_line": start + 1,
                      "setup_body_request_line": start + 2,
                      "source_load_request_reply_line": source_index + 1,
                      "last_segment_in_exploration_and_final_transcript": True,
                      "initial_reply_lines": initial_candidates, "final_reply_lines": final_lines,
                      "stages": stage_lines, "actions": actions, "exceptions": exceptions})
    if name in ("k_A", "k_B", "k_cancel"):
        assert len(exceptions) == 2
        descriptions = [x["raw_reply"]["result"]["result"]["description"].splitlines()[0] for x in exceptions]
        assert descriptions == ["ReferenceError: dui is not defined", "ReferenceError: dcdAnim is not defined"]
        assert u["final"]["stack"][0] == {"name": "k_danggu", "step": 0}
        assert all(x is None or not x.get("candidates") for x in u["final"]["kUI"])
        blockers.append({"case": name, "exceptions": exceptions, "actual_final_stack": u["final"]["stack"],
                         "actual_trace": u["final"]["trace"], "actual_kUI": u["final"]["kUI"],
                         "upstream_partner_choice_executed": False,
                         "native_final_choice": d["final"]["choice"], "native_flags": d["final"]["flags"]})

    excluded = {c["id"] for c in u["initial"]["skippedInitialDiscard"]}
    assert len(excluded) == 21
    available_initial_n = zones(d["initial"], True, excluded)
    available_initial_u = zones(u["initial"], False, excluded)
    # Do not silently normalize the two initial Shandian nature representations.
    # Retain that initial mismatch, and separately compare the actually used
    # hand/deck entities. All original discard entities stay in discard here.
    initial_diffs = differences(available_initial_n, available_initial_u)
    original_discard = {c["id"] for c in d["initial"]["discard"]}
    initial_n = zones(d["initial"], True, original_discard)
    initial_u = zones(u["initial"], False, original_discard)
    assert initial_n == initial_u
    for before, after in ((d["initial"], d["final"]), (u["initial"], u["final"])):
        before_cards = {c["id"]: face(c) for c in before["discard"]}
        after_cards = {c["id"]: face(c) for c in after["discard"]}
        assert all(after_cards.get(card_id) == value for card_id, value in before_cards.items())
    primary_n = [[x["name"] for x in p["skills"]] for p in d["initial"]["players"]]
    primary_u = [[x["name"] for x in p["skills"]] for p in u["initial"]["players"]]
    assert primary_n == primary_u
    assert not u["initial"]["untaggedDeck"] and not u["final"]["untaggedDeck"]
    assert not u["initial"]["unmappedDeck"]
    final_n, final_u = zones(d["final"], True, original_discard), zones(u["final"], False, original_discard)
    diffs = []
    for np, up in zip(final_n["players"], final_u["players"]):
        diffs += differences(np, up, "/players/" + str(np["seat"]))
    for key in ("deck_top_first", "discard_projection", "table_or_ordering"):
        diffs += differences(final_n[key], final_u[key], "/" + key)
    native_choices = []
    for command in d["commands"]:
        ch = command["after"]["choice"]
        if ch and (not native_choices or native_choices[-1] != ch):
            native_choices.append(ch)
    cases.append({"case": name,
                  "initial_projected_fields_match": True, "initial_primary_skill_names_match": True,
                  "initial_projection": initial_n, "excluded_unmapped_discard": u["initial"]["skippedInitialDiscard"],
                  "available_initial_native_projection": available_initial_n,
                  "available_initial_upstream_projection": available_initial_u,
                  "available_initial_projection_differences": initial_diffs,
                  "all_initial_discard_ids_excluded_from_outcome_projection": sorted(original_discard),
                  "initial_discard_unchanged_within_each_runtime": True,
                  "native_choice_sequence": native_choices,
                  "native_requests_replies": [{"request": c["request"], "reply": c["reply"], "log_delta": c["log_delta"]} for c in d["commands"]],
                  "upstream_choices": [{"label": s["label"], "choice": s["snapshot"]["choice"]} for s in u["stages"]],
                  "final_native_projection": final_n, "final_upstream_projection": final_u,
                  "final_physical_projection_equal": not diffs, "physical_projection_differences": diffs,
                  "native_pending_at_stop": d["final"]["choice"],
                  "upstream_root_done": u["final"]["done"], "upstream_final_stack": u["final"]["stack"],
                  "upstream_target_branch_flag_only": u.get("upstream_completed"),
                  "upstream_trace": u["final"]["trace"],
                  "native_once": d["final"]["once"], "native_flags": d["final"]["flags"],
                  "upstream_effective_skills": [{"seat": p["seat"], "skills": p["skills"],
                                                "reguhuo_phase": p["reguhuo_phase"],
                                                "k_danggu": p.get("k_danggu"), "k_danggu_current": p.get("k_danggu_current")} for p in u["final"]["players"]],
                  "direct_restored_captured_state_and_commands_equal": True,
                  "direct_restored_final_debug_equal_except_auth_generation": True,
                  "pending_archives": len(d["pending_archive_checks"]) + len(r["pending_archive_checks"]),
                  "equivalence_signed": False})

assert len(archives) == len(list((RAW / "native-round2").glob("*-pending-*.ngsa"))) == 102
replayed_files = sorted(REPLAY.iterdir())
replayed = [{"file": p.name, "bytes": p.stat().st_size, "sha256": digest(p),
             "matches_original_raw_bytes": p.read_bytes() == (RAW / "native-round2" / p.name).read_bytes()} for p in replayed_files]
assert len(replayed) == 237 and all(x["matches_original_raw_bytes"] for x in replayed)
source_text = (UPSTREAM / "extension/史诗卡牌/extension.js").read_text()
source_bodies = []
for key in ("precontentBody", "rangjie", "k_danggu", "k_danggu_contentx"):
    body = browser[0]["source_load"][key]
    assert all(row["source_load"][key] == body for row in browser)
    offset = source_text.find(body)
    assert offset >= 0
    source_bodies.append({"key": key, "exact_upstream_substring": True,
                          "line": source_text.count("\n", 0, offset) + 1,
                          "bytes_utf8": len(body.encode()), "sha256": hashlib.sha256(body.encode()).hexdigest()})
http = [json.loads(s) for s in (RAW / "session-l10-1794593358863-http.jsonl").read_text().splitlines()]
spine_urls = [x for x in http if re.search("spine", x["path"], re.I)]
assert not spine_urls
head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True).strip()
summary = {"at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(), "head_at_recompute": head,
           "raw_unique_cases_native": len(names), "raw_unique_cases_upstream": len(set(x["case"] for x in browser)),
           "common_initial_projection_matches": sum(x["initial_projected_fields_match"] for x in cases),
           "common_projection_excludes": ["all initial discard entities; 21 are unmapped, Shandian77/78 nature is null native versus thunder upstream; complete available discard differences retained per case", "hand ordering", "native hujia", "native flags and once versus upstream storage/temp skills", "RNG stream", "auth/control metadata", "virtual equipment objects not explicitly sampled"],
           "available_initial_projection_mismatch_cases": [x["case"] for x in cases if x["available_initial_projection_differences"]],
           "initial_discard_unchanged_in_own_runtime_cases": sum(x["initial_discard_unchanged_within_each_runtime"] for x in cases),
           "equal_final_physical_projection_cases": [x["case"] for x in cases if x["final_physical_projection_equal"]],
           "different_final_physical_projection_cases": [x["case"] for x in cases if not x["final_physical_projection_equal"]],
           "native_pending_cases": [x["case"] for x in cases if x["native_pending_at_stop"]],
           "upstream_root_done_cases": [x["case"] for x in cases if x["upstream_root_done"]],
           "upstream_root_not_done_cases": [x["case"] for x in cases if not x["upstream_root_done"]],
           "actual_k_ui_blocked_cases": [x["case"] for x in blockers],
           "pending_archives": len(archives), "per_mode_pending_archives": dict(collections.Counter(x["mode"] for x in archives)),
           "direct_restored_final_state_and_commands_equal_cases": len(cases),
           "direct_restored_final_debug_equal_except_auth_generation_cases": len(cases),
           "checkpoint_state_hash_equal_and_auth_increment_one": len(archives),
           "roundtrip_ngsa_raw_byte_equal_checkpoints": sum(x["roundtrip_bytes_identical"] for x in archives),
           "direct_restored_final_ngsa_byte_equal_cases": sum(x["final_ngsa_bytes_equal"] for x in native),
           "replayed_raw_files_byte_identical": len(replayed), "raw_cdp_requests_replies_paired": len(requests),
           "final_cohort_first_setup_line": last[names[0]][0] + 1,
           "raw_http_requests": len(http), "local_spine_url_requests": len(spine_urls),
           "source_bodies_exact_upstream_substrings": source_bodies,
           "new_equivalence_signed": 0,
           "not_run": ["fresh independent Chrome rerun", "product tests/Cargo build", "full workspace/preflight", "UI dependency stubbing or prohibited Spine load", "full L10 source acceptance"]}
write("case-recalculation.json", cases)
write("cdp-provenance.json", cdp_cases)
write("archive-recalculation.json", archives)
write("native-replay01-file-check.json", replayed)
write("k-ui-blockers-raw.json", blockers)
write("summary-recalculation.json", summary)
print(canonical({key: summary[key] for key in ("raw_unique_cases_native", "common_initial_projection_matches", "pending_archives", "replayed_raw_files_byte_identical", "raw_cdp_requests_replies_paired", "actual_k_ui_blocked_cases", "native_pending_cases")}))
