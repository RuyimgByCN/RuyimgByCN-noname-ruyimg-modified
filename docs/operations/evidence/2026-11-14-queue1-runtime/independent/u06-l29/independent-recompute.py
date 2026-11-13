"""Independent, read-only comparisons of raw U06/L29 runtime evidence.

Writes only beside this script. It does not import the producer's comparator.
"""
from __future__ import annotations

import hashlib
import json
from collections import Counter
from datetime import datetime
from pathlib import Path

OWN = Path(__file__).resolve().parent
BASE = OWN.parents[1]
REPO = BASE.parents[3]


def read(path):
    return json.loads(Path(path).read_text())


def lines(path):
    return [json.loads(line) for line in Path(path).read_text().splitlines() if line]


def canonical(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def digest(path):
    data = Path(path).read_bytes()
    return {"bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}


def frozen(relative):
    return OWN / "initial-inputs" / relative


ALIASES = {
    "hnl_monkey_3": "monkey", "hnl_kuwu_0": "kuwu",
    "hnl_xuelunyang_4": "xuelunyang", "duel": "juedou",
    "huanle:monkey": "monkey", "huanle_kuwu": "kuwu",
    "huanle:xuelunyang": "xuelunyang", "马术": "mashu",
    "咆哮": "paoxiao", "武圣": "wusheng",
}
HELPERS = {"player_when_flmwdj8h", "player_when_9xp8vfup", "player_when_1m6i6ing"}


def face(card, with_id=False):
    name = card.get("upstream_name") or ALIASES.get(card["name"], card["name"])
    result = (name, card["suit"].lower(), card["rank"], card.get("nature"))
    return (card["id"],) + result if with_id else result


def cards(items, with_id=False):
    return sorted((face(card, with_id) for card in items), key=repr)


def player(player, with_id=False):
    skills = [skill["name"] if isinstance(skill, dict) else skill for skill in player["skills"]]
    result = {
        "seat": player["seat"], "hp": player["hp"], "max_hp": player["max_hp"],
        "hand": cards(player["hand"], with_id), "equips": cards(player["equips"], with_id),
        "semantic_skills": sorted(ALIASES.get(s, s) for s in skills if s not in HELPERS),
    }
    if "face_down" in player:
        result["face_down"] = player["face_down"]
    return result


def cdp_values(prefix):
    requests = lines(Path(str(prefix) + "-requests.jsonl"))
    replies = lines(Path(str(prefix) + "-replies.jsonl"))
    assert len(requests) == len(replies)
    parsed = []
    for index, (request, reply) in enumerate(zip(requests, replies)):
        assert request["at"] == reply["at"]
        value = reply.get("result", {}).get("result", {}).get("value")
        if isinstance(value, str):
            try:
                value = json.loads(value)
            except ValueError:
                pass
        parsed.append({"index": index, "at": reply["at"], "value": value,
                       "expression": request.get("expression", "")})
    return parsed


def compare_u06():
    root = frozen("evidence/U06")
    native = read(root / "native-results-first.json")
    raw = lines(root / "browser/case-results.jsonl")
    browser = [row for row in raw if row.get("run_tag") == "fresh-nodes"]
    exploratory = [row for row in raw if row.get("run_tag") != "fresh-nodes"]
    assert len(native) == len(browser) == 13 and len(exploratory) == 13
    assert [row["case"] for row in native] == [row["case"] for row in browser]
    raw_native = [json.loads(line) for line in (root / "02-native-probe-first.log").read_text().splitlines()
                  if line.startswith("{")]
    assert raw_native == native
    cdp = cdp_values(root / "browser/session-modern-1794592005610")
    start_script = (root / "browser/start-case.js").read_text()
    choice_script = (root / "browser/await-choice.js").read_text()
    finish_script = (root / "browser/finish-case.js").read_text()
    result = []
    previous_finish = -1
    for n, b in zip(native, browser):
        initial = n["initial"].get("fixture", n["initial"])
        assert [player(p) for p in initial["players"]] == [player(p) for p in b["setup"]["initial"]]
        assert b["result"]["initial"] == b["setup"]["initial"]
        finishes = [r for r in cdp if r["value"] == b["result"]]
        finish = max(finishes, key=lambda r: r["index"])
        starts = [r for r in cdp if r["value"] == b["setup"]
                  and previous_finish < r["index"] < finish["index"]]
        start = max(starts, key=lambda r: r["index"])
        assert start["expression"] == start_script
        assert finish["expression"] == finish_script
        fixture_requests = [r for r in cdp[previous_finish + 1:start["index"]]
                            if r["expression"].startswith("window.runtimeFixture=")]
        injected = fixture_requests[-1]["expression"].removeprefix("window.runtimeFixture=").removesuffix(";true")
        assert json.loads(injected) == n
        choice_indices = []
        cursor = start["index"]
        for choice in b["choices"]:
            matches = [r for r in cdp[cursor + 1:finish["index"]] if r["value"] == choice]
            match = min(matches, key=lambda r: r["index"])
            assert match["expression"] == choice_script
            choice_indices.append(match["index"])
            cursor = match["index"]
        previous_finish = finish["index"]
        checks = {
            "initial_players": True,
            "final_players": [player(p) for p in n["players"]] == [player(p) for p in b["result"]["players"]],
            "final_discard": cards(n["discard"]) == cards(b["result"]["discard"]),
            "settled": n["settled"] == b["result"]["settled"],
        }
        helpers = [{"seat": p["seat"], "skills": [s for s in p["skills"] if s in HELPERS],
                    "additionalSkills": p.get("additionalSkills", {})}
                   for p in b["result"]["players"] if any(s in HELPERS for s in p["skills"])]
        native_choices = n["choices"]
        assert all(c["resumed_hash_and_log_equal"] for c in native_choices)
        result.append({"case": n["case"], "checks": checks,
                       "native_final": {"players": [player(p) for p in n["players"]],
                                        "discard": cards(n["discard"]), "settled": n["settled"], "pending": n["pending"]},
                       "upstream_final": {"players": [player(p) for p in b["result"]["players"]],
                                          "discard": cards(b["result"]["discard"]), "settled": b["result"]["settled"]},
                       "native_answer_attempts": native_choices, "upstream_choices": b["choices"],
                       "generated_cleanup_skills_separately_recorded": helpers,
                       "cdp_indices_zero_based": {"setup": start["index"], "choices": choice_indices,
                                                   "final": finish["index"],
                                                   "all_exact_final_matches_including_exploration": [r["index"] for r in finishes]}})
    different = [r["case"] for r in result if not all(r["checks"].values())]
    assert different == ["monkey-cancel", "xuelun-skill-cancel"]
    return {"native_cases": len(native), "raw_upstream_cases": len(raw), "valid_fresh_cases": len(browser),
            "exploratory_cases_preserved": len(exploratory), "initial_equal": len(browser),
            "all_projected_final_fields_equal": len(browser) - len(different), "different_cases": different,
            "native_answer_attempts_and_restore_checkpoints": sum(len(n["choices"]) for n in native),
            "native_raw_log_exactly_matches_json": True, "rows": result}


def compare_l29():
    root = BASE / "L29"
    before = OWN / "l29-own-before-run"
    after = OWN / "l29-own-run"
    rows = read(after / "old-ngsa-restore-results.json")
    producer_rows = read(root / "native-after-jiushi-fix/old-ngsa-restore-results.json")
    assert rows == producer_rows
    old_bytes_equal = []
    continuation_counts = Counter()
    for row in rows:
        path = Path(row["input"])
        assert row["error"] is None and row["snapshot_equal_except_auth_generation"]
        same_before_bytes = path.read_bytes() == (before / path.name).read_bytes()
        assert same_before_bytes
        old_bytes_equal.append({"file": str(path.relative_to(REPO)), **digest(path),
                                "exact_match_to_independently_executed_preserved_before_binary": True})
        cont = row["continuation"]
        assert cont["final_snapshot_equal_except_auth_generation"] and cont["final_state_hash_equal"]
        for cmd in cont["commands"]:
            assert cmd["request_equal_to_before"] and cmd["reply_equal_to_before"] and cmd["log_delta_equal_to_before"]
            continuation_counts["commands"] += 1
    direct_rows = []
    for path in sorted((root / "native-round4").glob("native-*.json")):
        if path.name in ("native-input-metadata.json", "native-pack-input.json"):
            continue
        old = read(path)
        if "direct" not in old:
            continue
        b, a = read(before / path.name), read(after / path.name)
        fields = ["initial", "commands", "final", "final_archive_hex"]
        checks = {field: b["direct"][field] == a["direct"][field] == old["direct"][field] for field in fields}
        assert all(checks.values())
        direct_rows.append({"case": old["case"], "checks": checks})
    assert len(direct_rows) == 22 and len(rows) == 68 and continuation_counts["commands"] == 136
    assert read(before / "native-input-metadata.json") == read(after / "native-input-metadata.json")
    cdp = []
    for name in ("session-l29-1794591931110", "session-l29b-1794592604612"):
        for value in cdp_values(root / name):
            cdp.append({**value, "session": name})
    raw_evidence = []
    for case in ("q_A", "q_B", "j_active_gain", "j_active_cancel_gain", "j_damage_back_accept_gain", "j_damage_back_cancel"):
        browser = read(root / f"browser-round8/upstream-{case}.json")
        native = read(root / f"native-round4/native-{case}.json")["direct"]
        assert [player(p, True) for p in native["initial"]["players"]] == [player(p, True) for p in browser["start"]["initial"]["players"]]
        verified = []
        for stage in browser["stages"]:
            matches = [r for r in cdp if r["value"] == stage["snapshot"]]
            assert matches, (case, stage["label"])
            verified.append({"label": stage["label"], "matches": [{"session": r["session"], "index": r["index"]} for r in matches]})
        raw_evidence.append({"case": case, "initial_players_id_face_hp_skills_equal": True, "stage_to_raw_cdp": verified})
    return {"own_before_exit": read(OWN / "l29-own-before-executable.json")["exit_code"],
            "own_after_exit": 0, "old_archives_restored": len(rows),
            "full_snapshot_equal_except_auth_generation": len(rows),
            "continuation_commands_request_reply_log_equal": continuation_counts["commands"],
            "final_full_snapshot_and_hash_equal": len(rows),
            "before_after_direct_cases_equal_including_final_archive_bytes": direct_rows,
            "same_pack_content_lock_and_ruleset": read(after / "native-input-metadata.json"),
            "old_archive_input_digests_and_before_binary_reproduction": old_bytes_equal,
            "own_old_restore_results_exactly_equal_producer_after_results": True,
            "l29_semantic_counterexample_raw_links": raw_evidence}


if __name__ == "__main__":
    result = {"at": datetime.now().astimezone().isoformat(),
              "method": "Independent functions in this script; no producer comparator imported; card/hand multisets only; full event/RNG/skill equivalence is not asserted.",
              "u06": compare_u06(), "l29": compare_l29()}
    (OWN / "independent-recomputation.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"u06": {k: v for k, v in result["u06"].items() if k != "rows"},
                      "l29_old_archives": result["l29"]["old_archives_restored"],
                      "l29_continuations": result["l29"]["continuation_commands_request_reply_log_equal"]}, ensure_ascii=False))
