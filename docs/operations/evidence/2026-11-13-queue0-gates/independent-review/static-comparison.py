#!/usr/bin/env python3
"""Independent static checks of the three queue-0 guard repairs."""
from datetime import datetime
import hashlib
import json
from pathlib import Path
import re
import subprocess


ROOT = Path(__file__).resolve().parents[5]
archive = ROOT / "crates/engine-native-kernel/src/archive"
expected = set()
old_guard = set()
new_guard = set()
modules = []
for path in sorted(archive.glob("known_prompts*.rs")):
    source = path.read_text()
    arrays = re.findall(r"&\[(.*?)\];", source, re.S)
    values = set(re.findall(r'"([^"\n]+)"', "\n".join(arrays)))
    narrow = set(re.findall(r'^    "([^"\n]+)",', source, re.M))
    broad = set(re.findall(r'"([^"\n]+)"\s*,', source))
    expected.update(values)
    old_guard.update(narrow)
    new_guard.update(broad)
    modules.append({
        "file": str(path.relative_to(ROOT)),
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "array_values": len(values),
        "old_line_guard": len(narrow),
        "new_guard": len(broad),
        "missing_new": sorted(values - broad),
    })

production_literals = set()
for crate in ("engine-native-kernel", "engine-native-content", "engine-native"):
    for path in (ROOT / "crates" / crate / "src").rglob("*.rs"):
        production_literals.update(re.findall(r'prompt:\s*"([^"\n]+)"', path.read_text()))

guard_path = ROOT / "apps/web-client/scripts/approved-b-metadata.test.mjs"
guard_source = guard_path.read_text()
assert 'matchAll(/"([^"\\n]+)"\\s*,/g)' in guard_source or 'matchAll(/"([^"]+)"\\s*,/g)' in guard_source
assert "assert(prompts.length>0" in guard_source
assert expected
assert not expected - new_guard

loader = ROOT / "crates/content-native-loader/src/decision34_pack9_data.rs"
loader_source = loader.read_text()
skill_array = loader_source.split("pub const U01_DDD_SKILLS: [&str; 64] = [", 1)[1].split("];", 1)[0]
native_names = set(re.findall(r'"([^"\n]+)"', skill_array))
upstream = ROOT.parent / "archive/upstream/noname/apps/core/extension/3D精选/character/skill.js"
upstream_names = set(re.findall(r"^\t(ddd[a-z_]+):\s*\{", upstream.read_text(), re.M))
assert len(native_names) == 64
assert native_names == upstream_names

result = {
    "reviewer": "queue0_independent_review",
    "at": datetime.now().astimezone().isoformat(),
    "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
    "tracked_dirty": bool(subprocess.check_output(["git", "status", "--porcelain", "--untracked-files=no"], cwd=ROOT, text=True)),
    "versions": {"rules": 333, "NGSA": 107, "schema": 5, "ABI": 759, "WS": 8},
    "prompts": {
        "modules": modules,
        "expected_unique": len(expected),
        "old_guard_unique": len(old_guard),
        "old_uncovered_including_production_scan": sorted(expected - old_guard - production_literals),
        "new_guard_unique": len(new_guard),
        "new_uncovered_including_production_scan": sorted(expected - new_guard - production_literals),
        "guard_sha256": hashlib.sha256(guard_path.read_bytes()).hexdigest(),
    },
    "U01_skill_names": {
        "native_count": len(native_names),
        "upstream_count": len(upstream_names),
        "native_only": sorted(native_names - upstream_names),
        "upstream_only": sorted(upstream_names - native_names),
        "loader_sha256": hashlib.sha256(loader.read_bytes()).hexdigest(),
        "upstream_sha256": hashlib.sha256(upstream.read_bytes()).hexdigest(),
        "scope": "Names only; no whole-source rule equivalence claim.",
    },
}
print(json.dumps(result, ensure_ascii=False, indent=2))
