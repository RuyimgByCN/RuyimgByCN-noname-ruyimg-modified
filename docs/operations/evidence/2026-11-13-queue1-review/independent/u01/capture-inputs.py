"""Capture frozen HEAD inputs and literal test references for this independent review."""
from pathlib import Path
import hashlib
import json
import re
import runpy
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[5]
HEAD = "bd58ef06d59940baed8d7f93e61ce571b152fb81"
MORE = [
    "crates/content-native-loader/src/host_skill_bindings.rs",
    "crates/engine-native-content/src/content/host/activation/common.rs",
    "crates/engine-native-content/src/content/host/filtering/common.rs",
    "crates/engine-native-content/src/content/host/step/skills_damage.rs",
    "crates/engine-native-content/src/content/host/step/skills_targets.rs",
    "crates/engine-native-content/src/content/host/support.rs",
    "crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_view_as.rs",
    "crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_flags.rs",
    "crates/engine-native-kernel/src/rvm_bridge/handlers.rs",
    "crates/engine-native-kernel/src/kernel/events.rs",
    "crates/engine-native-kernel/src/kernel/events/turn.rs",
    "crates/engine-native-kernel/src/kernel/duel.rs",
    "crates/engine-native-kernel/src/kernel/equipment.rs",
    "crates/engine-native-kernel/src/catalog/card_kind.rs",
    "crates/engine-native-kernel/src/vocabulary/generals.rs",
    "crates/engine-native-kernel/src/vocabulary/host/metadata.rs",
    "crates/engine-native-kernel/src/vocabulary/host/metadata/common.rs",
    "crates/engine-native-kernel/src/vocabulary/host/metadata/source_order.rs",
    "crates/engine-native-kernel/src/state/general_assembly.rs",
    "crates/engine-native/src/engine/general_select.rs",
    "crates/engine-native-kernel/src/resolution.rs",
    "crates/engine-native-kernel/src/resolution/trigger_order.rs",
    "crates/engine-native/tests/decision34_pack9_closure.rs",
    "crates/engine-native/tests/g1_d3.rs",
    "crates/engine-native/tests/l11_taixu_boss_events.rs",
    "crates/engine-native/tests/nc10_live_hidden_cards.rs",
]


def git(*args):
    return subprocess.run(["git", *args], cwd=ROOT, check=True, capture_output=True).stdout


def capture():
    manifest = []
    for path in MORE:
        data = git("show", f"{HEAD}:{path}")
        dest = HERE / "inputs-additional" / path
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_bytes(data)
        manifest.append(dict(path=path, snapshot_path=str(dest.relative_to(HERE)),
                             sha256=hashlib.sha256(data).hexdigest(), lines=len(data.splitlines()),
                             source=f"git show {HEAD}:{path}"))
    upstream = ROOT.parent / "archive/upstream/noname"
    for path in ["apps/core/noname/get/index.js", "apps/core/noname/library/element/player.js",
                 "apps/core/noname/game/check.js", "apps/core/noname/library/element/content.ts"]:
        data = (upstream / path).read_bytes()
        dest = HERE / "inputs-additional/upstream" / path
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_bytes(data)
        manifest.append(dict(path=str((upstream / path).relative_to(ROOT.parent)),
                             snapshot_path=str(dest.relative_to(HERE)),
                             sha256=hashlib.sha256(data).hexdigest(), lines=len(data.splitlines()),
                             source="actual read-only upstream file"))
    original = json.loads((HERE / "inputs.json").read_text())
    verification = []
    for item in original["inputs"]:
        path = item["path"]
        snapshot = (HERE / "inputs/upstream" / Path(path).name if path.startswith("../archive/")
                    else HERE / "inputs" / path)
        actual = hashlib.sha256(snapshot.read_bytes()).hexdigest()
        assert actual == item["sha256"], path
        if not path.startswith("../archive/"):
            assert hashlib.sha256(git("show", f"{HEAD}:{path}")).hexdigest() == actual, path
        verification.append(dict(path=path, snapshot_preserved=True, matches_original_sha256=True))
    (HERE / "inputs-additional.json").write_text(json.dumps(dict(
        head=HEAD,
        basis="native files captured from frozen HEAD; original snapshots preserved; worktree repair captured separately",
        inputs=manifest, original_27_integrity=verification), ensure_ascii=False, indent=2) + "\n")
    rows = runpy.run_path(str(HERE / "audit-data.py"))["DATA"]
    names = {row["skill"] for row in rows}
    refs = {}
    for path in git("ls-tree", "-r", "--name-only", HEAD, "crates/engine-native/tests").decode().splitlines():
        if not path.endswith(".rs"):
            continue
        for line_no, line in enumerate(git("show", f"{HEAD}:{path}").decode().splitlines(), 1):
            for skill in set(re.findall(r"\bddd[a-z_]+\b", line)) & names:
                refs.setdefault(skill, {}).setdefault(path, []).append(line_no)
    (HERE / "test-reference-index.json").write_text(json.dumps(dict(
        basis="git show frozen HEAD; literal references only, not assertion coverage",
        skills=refs), ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(dict(additional_inputs=len(manifest), original_snapshots_preserved=len(verification),
                          literal_test_names=len(refs),
                          test_files=sorted({p for v in refs.values() for p in v})), ensure_ascii=False))


if __name__ == "__main__":
    capture()
