"""Compare only the captured U06 visible projection; never sign rule equivalence."""
from collections import Counter
from pathlib import Path
import datetime
import hashlib
import json

BASE = Path(__file__).resolve().parent
ALIASES = {
    "hnl_monkey_3": "monkey", "huanle:monkey": "monkey",
    "hnl_kuwu_0": "kuwu", "huanle_kuwu": "kuwu",
    "hnl_xuelunyang_4": "xuelunyang", "huanle:xuelunyang": "xuelunyang",
    "duel": "juedou",
    "马术": "mashu", "咆哮": "paoxiao", "武圣": "wusheng",
}


def name(value):
    return ALIASES.get(value, value)


def faces(cards):
    return sorted((name(c["name"]), c["suit"], c["rank"]) for c in cards)


def skills(player):
    raw = [s["name"] if isinstance(s, dict) else s for s in player["skills"]]
    return sorted(name(s) for s in raw if not s.startswith("player_when_"))


def projection(players):
    return [{"seat": p["seat"], "hp": p["hp"], "max_hp": p["max_hp"],
             "hand": faces(p["hand"]), "equips": faces(p["equips"]),
             "skills": skills(p)} for p in players]


def self_check():
    a = {"name": "hnl_monkey_3", "suit": "diamond", "rank": 5}
    b = {"name": "monkey", "suit": "diamond", "rank": 5}
    assert faces([a]) == faces([b])
    assert faces([{**a, "name": "duel"}]) == faces([{**a, "name": "juedou"}])
    assert faces([a, a]) != faces([b])  # Preserve card multiplicity.
    assert faces([a]) != faces([{**b, "rank": 6}])
    p = {"seat": 0, "hp": 4, "max_hp": 4, "hand": [a], "equips": [],
         "skills": [{"name": "马术"}]}
    assert projection([p]) != projection([{**p, "hp": 3}])
    assert skills({"skills": ["player_when_example", "mashu"]}) == ["mashu"]


self_check()
paths = [BASE / "native-results-first.json", BASE / "browser/case-results.jsonl"]
native = json.loads(paths[0].read_text())
all_browser = [json.loads(line) for line in paths[1].read_text().splitlines()]
browser = [r for r in all_browser if r["run_tag"] == "fresh-nodes"]
assert len(native) == len(browser) == 13
assert len({r["case"] for r in native}) == len({r["case"] for r in browser}) == 13
by_case = {r["case"]: r for r in browser}
assert set(by_case) == {r["case"] for r in native}
rows = []
for n in native:
    u = by_case[n["case"]]
    initial = n["initial"].get("fixture", n["initial"])
    ni, ui = projection(initial["players"]), projection(u["setup"]["initial"])
    assert ni == ui, (n["case"], "initial projection mismatch", ni, ui)
    assert u["setup"]["initial"] == u["result"]["initial"]
    np, up = projection(n["players"]), projection(u["result"]["players"])
    nd, ud = faces(n["discard"]), faces(u["result"]["discard"])
    choices = n["choices"]
    assert all(c["resumed_hash_and_log_equal"] for c in choices)
    rows.append({
        "case": n["case"], "initial_projection_equal": True,
        "native_final_projection": np, "upstream_final_projection": up,
        "final_projection_equal": np == up, "discard_projection_equal": nd == ud,
        "native_settled": n["settled"], "upstream_settled": u["result"]["settled"],
        "settlement_equal": n["settled"] == u["result"]["settled"],
        "projection_and_settlement_equal": np == up and nd == ud and n["settled"] == u["result"]["settled"],
        "native_choices_and_restore_attempts": choices,
        "upstream_choices": u["choices"],
        "upstream_generated_lifecycle_skills": [
            {"seat": p["seat"], "skills": [s for s in p["skills"] if s.startswith("player_when_")]}
            for p in u["result"]["players"] if any(s.startswith("player_when_") for s in p["skills"])],
    })
record = {
    "at": datetime.datetime.now().astimezone().isoformat(),
    "inputs": [{"path": str(p.relative_to(BASE)), "sha256": hashlib.sha256(p.read_bytes()).hexdigest()} for p in paths],
    "included_browser_tag": "fresh-nodes", "excluded_tags": dict(Counter(r["run_tag"] for r in all_browser if r["run_tag"] != "fresh-nodes")),
    "aliases": ALIASES, "cases": rows,
    "counts": {"cases": len(rows), "initial_equal": sum(r["initial_projection_equal"] for r in rows),
               "final_projection_equal": sum(r["final_projection_equal"] for r in rows),
               "discard_projection_equal": sum(r["discard_projection_equal"] for r in rows),
               "projection_and_settlement_equal": sum(r["projection_and_settlement_equal"] for r in rows),
               "restore_attempts": sum(len(r["native_choices_and_restore_attempts"]) for r in rows)},
    "uncompared": ["deck/order/RNG", "full event stream", "phase/identity and all state fields",
                   "generated player_when lifecycle and skill expiry", "all conditions and combinations"],
    "equivalence_signoffs": 0,
}
out = BASE / "comparison-fresh-nodes.json"
assert not out.exists(), "Preserve earlier comparison output"
out.write_text(json.dumps(record, ensure_ascii=False, indent=2) + "\n")
print(json.dumps(record["counts"]))
for row in rows:
    if not row["projection_and_settlement_equal"]:
        print("difference", row["case"], "native settled", row["native_settled"], "upstream settled", row["upstream_settled"])
