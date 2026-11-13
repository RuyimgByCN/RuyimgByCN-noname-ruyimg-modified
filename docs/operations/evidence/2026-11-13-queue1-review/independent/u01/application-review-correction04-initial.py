"""Append-only review of the later 31 concrete character descriptions."""
import datetime
import hashlib
import json
import pathlib
import re
from zoneinfo import ZoneInfo

HERE = pathlib.Path(__file__).resolve().parent
BASE = HERE.parents[1]
ROOT = BASE.parents[3]
AT = datetime.datetime.now(ZoneInfo("Asia/Taipei")).isoformat()
read = lambda p: json.loads(p.read_text())
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
correction = read(BASE / "inventory-correction-04.json")
current_path = ROOT / "docs/migration/g0/inventory_status.json"
current_bytes = current_path.read_bytes()
current = json.loads(current_bytes)
app = read(BASE / "inventory-application.json")
after = {r["key"]: r["after"] for r in app["changes"]}
skills = read(HERE / "skills.json")
characters = read(HERE / "characters.json")
up_path = ROOT.parent / "archive/upstream/noname/apps/core/extension/3D精选/character/character.js"
native_path = ROOT / "crates/content-native-loader/src/decision34_pack9_data.rs"
assembly_path = ROOT / "crates/engine-native-kernel/src/state/general_assembly.rs"
select_path = ROOT / "crates/engine-native/src/engine/general_select.rs"
up = {}
for name, body in re.findall(r"\t(ddd_\w+): \{(.*?)\n\t\},", up_path.read_text(), re.S):
    sex = re.search(r'sex: "(\w+)"', body).group(1)
    group = re.search(r'group: "(\w+)"', body).group(1)
    hp = int(re.search(r"\bhp: (\d+)", body).group(1))
    max_hp = re.search(r"\bmaxHp: (\d+)", body)
    names = re.findall(r'"(\w+)"', re.search(r"skills: \[(.*?)\]", body).group(1))
    up[name] = {"sex": sex, "group": group, "hp": hp, "max_hp": int(max_hp.group(1)) if max_hp else hp, "skills": names}
native = {}
for name, faction, gender, hp, names in re.findall(r'\("(ddd_\w+)", Faction::(\w+), Gender::(\w+), (\d+), &\[(.*?)\]\)', native_path.read_text()):
    native[name] = {"sex": gender.lower(), "group": faction.lower(), "max_hp": int(hp), "skills": re.findall(r'"(\w+)"', names)}
assert len(up) == len(native) == 31
assert '"ddd_zhangkai" => 3,' in assembly_path.read_text()
assert '"ddd_zhangkai" => (p.max_hp as i32).saturating_sub(1),' in select_path.read_text()
changes = {r["key"]: r for r in correction["changes"] if r["key"].startswith("U01:")}
assert set(changes) == {r["inventory_key"] for r in characters}
rows = []
for character in characters:
    name, key = character["name"], character["inventory_key"]
    fields, native_fields = up[name], native[name]
    assert all(fields[f] == native_fields[f] for f in ("sex", "group", "max_hp", "skills"))
    assert fields["hp"] == (3 if name == "ddd_zhangkai" else native_fields["max_hp"])
    assert all(fields[f] == character["upstream"][f] for f in fields)
    prefix = f"本次实际静态字段：{fields['sex']}／{fields['group']}／{fields['hp']}起始、{fields['max_hp']}上限；" + "、".join(fields["skills"]) + "。"
    expected = {**after[key], "difference": prefix + after[key]["difference"], "diff": prefix + after[key]["diff"]}
    assert current["items"][key] == expected
    assert changes[key]["before_difference"] == after[key]["difference"]
    assert changes[key]["after_difference"] == expected["difference"]
    rows.append({"key": key, "source_fields": fields, "native_fields": native_fields, "description_matches_actual_fields": True, "only_difference_and_diff_changed": True, "status_reviewer_date_unchanged": True})
assert all(current["items"][r["inventory_key"]] == after[r["inventory_key"]] for r in skills)
assert hashlib.sha256(current_bytes).hexdigest() == correction["after_sha256"]
assert correction["before_sha256"] == read(HERE / "application-review-inventory.json")["current_inventory_sha256"]
frozen = read(BASE / "frozen-product-inputs.json")
drift = [p for p, h in frozen["files"].items() if not (ROOT / p).is_file() or sha(ROOT / p) != h]
assert not drift
paths = [pathlib.Path(__file__), current_path, BASE / "inventory-correction-04.json", up_path, native_path, assembly_path, select_path]
out = {"at": AT, "reviewer": "/root/queue0_independent_review", "conclusion": "limited_character_description_increment_pass", "current_inventory_sha256": hashlib.sha256(current_bytes).hexdigest(), "current_counts_identification_only": current["counts"], "u01_skills64_equal_original_application_after": True, "u01_generals31_only_concrete_field_prefix_added": True, "u01_status_reviewer_date_changed_by_correction04": 0, "characters": rows, "all32259_frozen_product_inputs_still_equal": True, "inputs": [{"path": str(p), "sha256": sha(p)} for p in paths], "executed_cargo": 0, "executed_browser": 0, "new_equivalence_signoffs": 0, "non_u01_224_description_changes_semantically_reviewed": False}
with (HERE / "application-review-correction04.json").open("x") as stream:
    json.dump(out, stream, ensure_ascii=False, indent=2)
    stream.write("\n")
print("correction04: 31 descriptions match actual source and native fields; 64 skill records unchanged; 32259 product hashes unchanged")
