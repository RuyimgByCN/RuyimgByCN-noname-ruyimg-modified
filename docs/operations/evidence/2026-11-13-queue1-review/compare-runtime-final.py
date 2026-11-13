"""Compare each recorded repeat using only the explicitly shared projection."""
import datetime
import json
import pathlib

BASE = pathlib.Path(__file__).resolve().parent
NAMES = {"horse_minus": "chitu", "wxl_wuxingpan_0": "wuxingpan"}
NATURES = {"metal": 0, "wood": 1, "water": 2, "fire": 3, "soil": 4}


def cards(values):
    result = []
    for card in values:
        nature = card.get("wunature")
        if isinstance(nature, str):
            nature = NATURES[nature]
        result.append({"name": NAMES.get(card["name"], card["name"]),
                       "suit": card["suit"].lower(), "rank": card["rank"], "wunature": nature})
    return sorted(result, key=lambda r: (r["name"], r["suit"], r["rank"], -1 if r["wunature"] is None else r["wunature"]))


def projection(row, is_native, flags):
    result = []
    for p in row["players"]:
        player = {"seat": p["seat"], "hp": p["hp"], "hand": cards(p["hand"]), "equips": cards(p["equips"])}
        if flags:
            player["juxian_ban"] = bool(p["juxian_ban"] if is_native else p["juxianBan"])
            player["xuanlun_choice"] = bool(p["xuanlun_triggered"] if is_native else p["xuanlunChoice"])
        result.append(player)
    return result



native = {r["case"]: r for r in json.loads((BASE / "native-comparison-final-results.json").read_text())}
upstream = {}
for line in (BASE / "browser/case-results.jsonl").read_text().splitlines():
    row = json.loads(line)
    case = row["case"]
    if case.startswith(("exact-", "ordered-", "blind-repeat-1-")):
        key = case.removeprefix("exact-").removeprefix("ordered-").removeprefix("blind-repeat-1-")
        if key in ("juxian-top-black", "juxian-top-red"):
            # content.ts:8962 randomizes hidden buttons. Pair only the receipt
            # after a blind click; retain every duplicate rather than reroll it.
            received = row["result"]["players"][0]["hand"]
            assert len(received) == 1
            assert (received[0]["name"], received[0]["suit"], received[0]["rank"]) in (("sha", "club", 7), ("tao", "heart", 3))
            key = "juxian-top-" + ("red" if received[0]["suit"] == "heart" else "black")
    elif case.startswith("wuxingpan-"):
        key = case
    elif case in ("shengke-sha-final", "shengke-tao-final"):
        key = case.removesuffix("-final")
    else:
        continue
    upstream.setdefault(key, []).append(row)

assert len(native) == 13 and set(native) == set(upstream), (set(native) - set(upstream), set(upstream) - set(native))
results = []
for name, n in native.items():
    repeats = []
    for u in upstream[name]:
        actual_native = projection(n, True, u["engine"] == "modern")
        actual_upstream = projection(u["result"], False, u["engine"] == "modern")
        assert actual_native == actual_upstream, json.dumps({"case": name, "upstream_label": u["case"], "native": actual_native, "upstream": actual_upstream}, ensure_ascii=False)
        repeats.append({"raw_case_label": u["case"], "at": u["at"], "projection_equal": True})
    results.append({"case": name, "engine": upstream[name][0]["engine"], "projection_equal": True,
                    "players": actual_native, "native_pending_restores": len(n["choices"]),
                    "native_prompts": [c["prompt"] for c in n["choices"]], "upstream_repeats": repeats})

output = {"at": datetime.datetime.now(datetime.timezone.utc).isoformat(), "compared_branches": len(results),
          "raw_upstream_records": 31, "exploratory_records_not_in_same_fixture_cohort": 7,
          "compared_upstream_runs": sum(len(r["upstream_repeats"]) for r in results),
          "native_pending_restores": sum(r["native_pending_restores"] for r in results),
          "hidden_pairing": "Both engines' blind-slot results are classified by actual post-resolution receipt. All 24 same-fixture upstream runs (exact, ordered, blind-repeat) are retained and compared; the 7 earlier exploratory fixtures remain in raw logs but are not part of this cohort. Slot indices and RNG streams are not asserted equal.",
          "projection": "all seats: hp, hand/equipment face multisets, five-element card property; U01 additionally color-ban/玄论 pending-effect flag",
          "not_compared": ["full event order, RNG and complete deck/discard", "entry UI card/mode selection for direct useSkill branches", "all conditions, combinations, phase boundaries and cancellation branches"],
          "whole_skill_equivalence_signoffs": 0, "results": results}
with (BASE / "runtime-comparison-reviewed.json").open("x") as stream:
    json.dump(output, stream, ensure_ascii=False, indent=2)
    stream.write("\n")
print(f"shared projections match: {len(results)} branches / {output['compared_upstream_runs']} upstream runs; native pending restores: {output['native_pending_restores']}; whole-skill signoffs: 0")
