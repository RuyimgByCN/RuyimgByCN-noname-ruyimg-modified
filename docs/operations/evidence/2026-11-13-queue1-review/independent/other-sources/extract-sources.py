"""Read-only helpers for the independently selected source review (no verdicts)."""
from pathlib import Path
import json
import re

ROOT = Path(__file__).resolve().parents[6]
OUT = Path(__file__).parent


def block(text, start):
    opening = text.find("{", start)
    if opening < 0:
        return start, min(len(text), start + 2000)
    depth, state, i = 0, "", opening
    while i < len(text):
        c, pair = text[i], text[i:i + 2]
        if state in ("'", '"', "`"):
            if c == "\\":
                i += 2
                continue
            if c == state:
                state = ""
        elif state == "//":
            if c == "\n":
                state = ""
        elif state == "/*":
            if pair == "*/":
                state = ""
                i += 1
        elif pair in ("//", "/*"):
            state = pair
            i += 1
        elif c in ("'", '"', "`"):
            state = c
        elif c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return start, i + 1
        i += 1
    return start, min(len(text), start + 20000)


def upstream(item):
    ref = item["upstream_ref"].split(":")[0]
    path = ROOT.parent / ref if ref.startswith("archive/") else ROOT / ref.removeprefix("noname-next/")
    if item["source_id"] == "L29" and item["id"] in {"mbcmfangzhu", "mbcmjiushi", "mbcmqingzheng", "mbjuejin"}:
        path = ROOT.parent / "archive/upstream/noname-liuli/noname/character/mobile.js"
    text = path.read_text()
    ident = re.escape(item["id"])
    pattern = (r"(?:lib\.skill(?:\." + ident + r"|\[['\"]" + ident + r"['\"]\])\s*=\s*\{"
               r"|(?m:^[ \t]*['\"]?" + ident + r"['\"]?\s*:\s*\{))")
    matches = list(re.finditer(pattern, text))
    if matches:
        ref_line = item["upstream_ref"].split(":")[-1].split("-")[0]
        expected = int(ref_line) if ref_line.isdigit() else 1
        match = min(matches, key=lambda m: abs(text.count("\n", 0, m.start()) + 1 - expected))
        a, b = block(text, match.start())
        return str(path.relative_to(ROOT.parent)), text.count("\n", 0, a) + 1, text[a:b]
    suffix = item["upstream_ref"].split(":")[-1].split("-")[0]
    line = int(suffix) if suffix.isdigit() else 1
    lines = text.splitlines()
    code = "\n".join(lines[max(0, line - 1):line + 42])
    if len(code) > 20000:
        match = re.search(r"[\"']?" + ident + r"[\"']?\s*:", text)
        code = text[max(0, match.start() - 200):match.start() + 3000] if match else "[No named definition found; inspect original file directly.]"
    return str(path.relative_to(ROOT.parent)), line, code


def main():
    selected = json.loads((OUT / "selected-samples.json").read_text())
    for source, data in selected["sources"].items():
        chunks = []
        for item in data["random_samples"] + data["interactive_supplements"]:
            path, line, code = upstream(item)
            chunks.append(f"\n=== {item['category']} {item['id']} | {path}:{line} ===\n{code}\n")
        (OUT / f"{source}-upstream-selected.txt").write_text("".join(chunks))


if __name__ == "__main__":
    main()
