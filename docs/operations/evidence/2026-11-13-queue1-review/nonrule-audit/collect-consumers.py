"""Read-only native consumer search; output stays in this evidence directory.

Literal hits are leads, not proof of a consumer. Generic names such as mode/text
need manual review. Inline Rust test tails and all Spine/skeleton paths are skipped.
"""
import hashlib
import json
import re
from pathlib import Path

BASE = Path(__file__).resolve().parent
ROOT = Path.cwd()
ROOTS = [
    "crates/content-native-loader/src", "crates/engine-native-kernel/src",
    "crates/engine-native-content/src", "crates/engine-native/src",
    "crates/engine-native-wasm/src", "apps/web-client/src",
    "apps/game-server/src", "apps/desktop-client/src",
]
SUFFIXES = {".rs", ".ts", ".vue", ".js", ".css"}
snapshot = json.loads((BASE / "inventory-snapshot.json").read_text())
tokens = {item["id"] for item in snapshot["items"].values() if item["category"] != "surfaces"}
tokens.update({
    "parse_epic_config", "shichangshi_plan", "parse_select_beautify_config",
    "parse_thunder_config", "replacement_plan", "should_hide_pack",
    "load_mark_supplement_config", "parse_mark_supplement_config",
    "parse_longxue_config", "parse_zuan_brawl", "parse_zuan_config", "parse_elo_config",
    "elo_apply_engine_config", "elo_time_killer_delay_ms", "elo_time_killer_chat_male", "elo_time_killer_chat_female",
    "triggerL10EmojiRebate", "applyL20MarkCssTheme", "applyL27SelectBeautifyTheme",
    "triggerLongxueHolySwordEffect", "selectL27DualGenerals", "zuanUi",
    "judgmentOverlay", "bossDefeatedBanner", "reviveCountdownBySeat", "lastAudioCue",
    "l27-select-beautify", "longxue-shengjian-active", "l25_luoyi_double_req",
    "xianqu_viewed_main", "xianqu_viewed_deputy", "xianqu_inspected_by", "xingchong_card",
})
patterns = {token: re.compile(r"(?<![\w])" + re.escape(token) + r"(?![\w])") for token in tokens}
hits = {token: [] for token in sorted(tokens)}
vue_hits = {token: [] for token in ["zuanUi", "judgmentOverlay", "bossDefeatedBanner", "reviveCountdownBySeat", "lastAudioCue"]}
scope = []
for relroot in ROOTS:
    start = ROOT / relroot
    if not start.is_dir():
        continue
    for file in sorted(start.rglob("*")):
        if not file.is_file() or file.suffix not in SUFFIXES:
            continue
        rel = file.relative_to(ROOT).as_posix()
        if any(word in rel.lower() for word in ["spine", "skeleton", "/test", ".test.", ".spec."]):
            continue
        content = file.read_text()
        production = re.split(r"(?m)^#\[cfg\(test\)\]", content, maxsplit=1)[0]
        scope.append({"path": rel, "sha256": hashlib.sha256(content.encode()).hexdigest()})
        for lineno, line in enumerate(production.splitlines(), 1):
            if line.lstrip().startswith(("//", "/*", "*")):
                continue
            for token, pattern in patterns.items():
                if pattern.search(line):
                    hit = {"path": rel, "line": lineno, "text": line.strip()[:500]}
                    hits[token].append(hit)
                    if file.suffix == ".vue" and token in vue_hits:
                        vue_hits[token].append(hit)
result = {"roots": ROOTS, "scope": scope,
          "excluded": "Spine/skeleton paths, tests, inline cfg(test) tails, leading comment lines; no upstream runtime read",
          "warning": "Literal hits do not establish feature consumption; declarations, exports and unrelated generic identifiers remain included.",
          "matches": hits, "vue_state_consumers": vue_hits}
(BASE / "consumer-probes.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
print(json.dumps({"source_files": len(scope), "tokens": len(tokens),
                  "vue_state_consumer_hits": {k: len(v) for k, v in vue_hits.items()}}, ensure_ascii=False))
