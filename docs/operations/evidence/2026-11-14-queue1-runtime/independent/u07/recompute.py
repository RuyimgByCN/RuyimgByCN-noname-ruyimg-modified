"""Independent U07 projection and raw-record checks. No producer comparator import."""
from __future__ import annotations

import hashlib
import json
import re
from collections import Counter
from datetime import datetime
from pathlib import Path

OWN = Path(__file__).resolve().parent
ROOT = OWN.parents[1] / "U07"
FROZEN = OWN / "final-inputs"


def read(path):
    return json.loads(Path(path).read_text())


def jsonl(path):
    return [json.loads(line) for line in Path(path).read_text().splitlines() if line.startswith("{")]


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


ALIASES = {
    "archery": "wanjian", "armor": "bagua", "bing": "bingliang", "chain": "tiesuo",
    "dismantle": "guohe", "duel": "juedou", "horse_minus": "chitu", "horse_plus": "jueying",
    "le": "lebu", "nullify": "wuxie", "savage": "nanman", "shandian_safe": "shandian",
    "snatch": "shunshou", "weapon2": "qinggang", "weapon2b": "cixiong", "weapon3": "qinglong",
    "fire_sha": "sha", "leisha": "sha",
}


def name(value):
    if value.startswith("wd_"):
        return re.sub(r"_\d+$", "", value[3:])
    return ALIASES.get(value, value)


def card(value):
    # Nature was not captured/created in these fixtures; it is explicitly outside this projection.
    return value["id"], name(value["name"]), value["suit"].lower(), value["rank"]


def player(value):
    return {
        **{key: value[key] for key in ("seat", "hp", "max_hp", "alive", "linked", "face_down", "hujia")},
        "skills": sorted("sanjian_skill" if skill == "wd_sanjian" else skill for skill in value["skills"]),
        "fuchou_marked": value["fuchou_marked"],
        **{zone: sorted((card(c) for c in value[zone]), key=repr) for zone in ("hand", "equips", "judge")},
    }


def projection(state):
    return {"players": [player(p) for p in state["players"]],
            "deck_draw_order": [card(c) for c in state["deck_draw_order"]],
            "discard": sorted((card(c) for c in state["discard"]), key=repr),
            "table": sorted((card(c) for c in state["table"]), key=repr)}


def cdp_values():
    meta = read(FROZEN / "browser/active-session-wudian.json")
    requests, replies = (jsonl(meta["transcript"] + "-" + suffix + ".jsonl") for suffix in ("requests", "replies"))
    assert len(requests) == len(replies)
    values = []
    for index, (request, reply) in enumerate(zip(requests, replies)):
        assert request["at"] == reply["at"]
        value = reply.get("result", {}).get("result", {}).get("value")
        if isinstance(value, str):
            try:
                value = json.loads(value)
            except ValueError:
                pass
        values.append({"line": index + 1, "value": value, "request": request, "reply": reply})
    return values


def first_differences(left, right):
    result = []
    for index, (a, b) in enumerate(zip(left, right)):
        if a != b:
            result.append({"index": index, "native": a, "upstream": b})
            if len(result) == 8:
                break
    return {"native_len": len(left), "upstream_len": len(right), "first_up_to_eight": result}


def compare():
    manifest = read(OWN / "final-inputs.json")
    assert all(sha(row["path"]) == row["sha256"] for row in manifest["inputs"])
    native = jsonl(FROZEN / "native/09-formal.log")
    browser = jsonl(FROZEN / "browser/case-results-twelfth.jsonl")
    assert len(native) == len(browser) == 24
    assert len({row["case"] for row in native}) == len({row["case"] for row in browser}) == 24
    assert [row["case"] for row in native] == [row["case"] for row in browser]
    assert all(row["run"] == "twelfth" and row["success"] for row in browser)
    supplied_index = read(FROZEN / "final-input-index.json")
    for path, expected in supplied_index["sha256"].items():
        assert sha(ROOT / path) == expected
    cdp = cdp_values()
    start_source = (FROZEN / "browser/start-case.js").read_text()
    rows, archive_rows, raw_excerpts = [], [], []
    previous_finish = 0
    for line, (n, b, indexed) in enumerate(zip(native, browser, supplied_index["cases"]), 1):
        assert indexed["case"] == n["case"]
        assert indexed["native_jsonl_line"] == indexed["browser_jsonl_line"] == line
        assert projection(n["initial"]) == projection(b["setup"]["initial"]), n["case"]
        assert b["setup"]["initial"] == b["result"]["initial"]
        assert not b["result"]["foreignPileMutations"]
        for p in b["setup"]["initial"]["players"]:
            assert len(p["virtualEquips"]) == len(p["equips"]) and not p["virtualJudges"]
        finishes = [v for v in cdp if v["value"] == b["result"]]
        finish = max(finishes, key=lambda v: v["line"])
        assert finish["line"] == indexed["raw_final_reply_line"] == indexed["raw_final_request_line"]
        starts = [v for v in cdp if previous_finish < v["line"] < finish["line"] and v["value"] == b["setup"]]
        start = max(starts, key=lambda v: v["line"])
        assert start["request"]["expression"] == start_source
        fixture = cdp[indexed["raw_fixture_request_line"] - 1]
        assert previous_finish < fixture["line"] < start["line"]
        expression = fixture["request"]["expression"]
        assert expression.startswith("window.runtimeFixture=") and expression.endswith(";true")
        assert json.loads(expression[len("window.runtimeFixture="):-len(";true")]) == n
        cursor = start["line"]
        choice_links = []
        for choice in b["choices"]:
            matches = [v for v in cdp[cursor:finish["line"] - 1] if v["value"] == choice]
            match = min(matches, key=lambda v: v["line"])
            choice_links.append(match["line"])
            cursor = match["line"]
        click_requests = [{"line": v["line"], "expression": v["request"]["expression"]}
                          for v in cdp[start["line"]:finish["line"] - 1]
                          if "q1.ui.click." in v["request"]["expression"]]
        # Preserve a small raw subset with both request and reply, not fabricated summary records.
        for number in {fixture["line"], start["line"], finish["line"], *choice_links}:
            raw_excerpts.append({"case": n["case"], **cdp[number - 1]})
        previous_finish = finish["line"]
        for choice in n["choices"]:
            assert choice["resume_state_hash_log_equal"]
            actual = Path(choice["archive_file"])
            saved = FROZEN / "native/formal-archives" / actual.name
            data = saved.read_bytes()
            assert data == actual.read_bytes() and len(data) == choice["archive_bytes"]
            assert data[:4] == b"NGSA" and int.from_bytes(data[4:6], "little") == 107
            assert int.from_bytes(data[14:22], "little") == len(data) - 54
            assert data[6:14] == data[22:30]
            archive_rows.append({"case": n["case"], "prompt": choice["prompt"], "observer": choice["observer"],
                                 "selection": choice["selection"], "bytes": len(data), "sha256": sha(saved),
                                 "archive": str(saved.relative_to(OWN)), "asserted_resume_outcome_hash_log_equal": True})
        stage_rows = {}
        for stage in ("after_attack", "final"):
            a, z = projection(n[stage]), projection(b["result"][stage])
            checks = {key: a[key] == z[key] for key in a}
            different_players = [{"seat": p["seat"], "native": p, "upstream": q}
                                 for p, q in zip(a["players"], z["players"]) if p != q]
            stage_rows[stage] = {"checks": checks, "different_players": different_players,
                                 "native_discard": a["discard"], "upstream_discard": z["discard"],
                                 "native_table": a["table"], "upstream_table": z["table"],
                                 "deck_difference": first_differences(a["deck_draw_order"], z["deck_draw_order"])}
        rows.append({"case": n["case"], "native_line": line, "upstream_line": line,
                     "initial_projection_equal": True, "stages": stage_rows,
                     "native_settled": n["settled"], "upstream_settled": b["result"]["settled"],
                     "native_choices": [{key: c[key] for key in ("prompt", "observer", "min", "max", "allow_cancel", "options", "selection", "outcome")} for c in n["choices"]],
                     "upstream_choices": b["choices"], "click_requests": click_requests,
                     "native_damage_logs": [s for s in n["log"] if "DamageDealt" in s or "HpLost" in s],
                     "raw_cdp_lines": {"fixture": fixture["line"], "setup": start["line"], "choices": choice_links,
                                       "final": finish["line"], "all_exact_final_matches": [v["line"] for v in finishes]}})
    assert len(archive_rows) == 43
    assert len({r["archive"] for r in archive_rows}) == 43
    catalog = read(FROZEN / "native/formal-catalog-without-placeholder.json")
    assert not catalog["placeholder_added"] and not catalog["real_diaobingqianjiang_cards"]
    assert all("diaobingqianjiang" not in c["name"] for c in native[0]["initial"]["deck_draw_order"])
    all_equal = [r["case"] for r in rows if all(r["stages"]["final"]["checks"].values())
                 and r["native_settled"] == r["upstream_settled"]]
    result = {"at": datetime.now().astimezone().isoformat(), "raw_native_cases": len(native), "raw_browser_cases": len(browser),
              "no_deduplication": True, "initial_projection_equal_cases": 24, "final_projection_equal_cases": all_equal,
              "final_projection_different_cases": [r["case"] for r in rows if r["case"] not in all_equal],
              "archived_restore_answer_checkpoints": len(archive_rows), "rows": rows, "archives": archive_rows,
              "scope": "IDs and name/suit/rank projection; nature and entire card semantics not compared; raw CDP linked exactly; no whole-skill equivalence signoff"}
    (OWN / "recomputed.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    with (OWN / "raw-cdp-excerpts.jsonl").open("w") as stream:
        for row in raw_excerpts:
            stream.write(json.dumps(row, ensure_ascii=False) + "\n")
    print(json.dumps({key: result[key] for key in ("raw_native_cases", "raw_browser_cases", "initial_projection_equal_cases", "final_projection_equal_cases", "final_projection_different_cases", "archived_restore_answer_checkpoints")}, ensure_ascii=False))


if __name__ == "__main__":
    compare()
