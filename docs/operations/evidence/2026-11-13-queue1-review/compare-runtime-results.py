"""Compare only the documented shared projection of actual upstream/native runs."""
import datetime
import json
import pathlib

BASE = pathlib.Path(__file__).resolve().parent
native = {r["case"]: r for r in json.loads((BASE / "native-comparison-green-first-results.json").read_text())}
upstream = {}
for line in (BASE / "browser/case-results.jsonl").read_text().splitlines():
    row = json.loads(line)
    case = row["case"]
    if case.startswith("ordered-"):
        key = case.removeprefix("ordered-")
        if key in ("juxian-top-black", "juxian-top-red"):
            # Upstream content.ts:8962 randomizes hidden button order. Pair the
            # resolved receipt after the blind click, never preselect by face.
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
    assert key not in upstream, f"ambiguous repeated result: {key}"
    upstream[key] = row

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


assert len(native) == 13 and set(native) == set(upstream), (set(native) - set(upstream), set(upstream) - set(native))
results = []
for name, n in native.items():
    u = upstream[name]
    flags = u["engine"] == "modern"
    actual_native = projection(n, True, flags)
    actual_upstream = projection(u["result"], False, flags)
    assert actual_native == actual_upstream, json.dumps({"case": name, "native": actual_native, "upstream": actual_upstream}, ensure_ascii=False)
    results.append({"case": name, "raw_upstream_case_label": u["case"], "engine": u["engine"], "upstream_at": u["at"], "projection_equal": True,
                    "players": actual_native, "native_pending_restores": len(n["choices"]),
                    "native_prompts": [c["prompt"] for c in n["choices"]]})

output = {"at": datetime.datetime.now(datetime.timezone.utc).isoformat(), "compared_branches": len(results),
          "native_pending_restores": sum(r["native_pending_restores"] for r in results),
          "projection": "all seats: hp, hand/equipment face multisets, five-element card property; U01 additionally color-ban/玄论 pending-effect flag",
          "not_compared": ["full event order, RNG and complete deck/discard", "entry UI card/mode selection for direct useSkill branches", "all conditions, combinations, phase boundaries and cancellation branches"],
          "whole_skill_equivalence_signoffs": 0, "results": results}
with (BASE / "runtime-comparison.json").open("x") as stream:
    json.dump(output, stream, ensure_ascii=False, indent=2)
    stream.write("\n")
print(f"shared projections match: {len(results)} branches; native pending restores: {output['native_pending_restores']}; whole-skill signoffs: 0")
