#!/usr/bin/env python3
"""Expand skill_catalog.toml templates → DSL fragment (stdout).

Usage:
  python3 content/tools/gen_skill_dsl.py
  python3 content/tools/gen_skill_dsl.py > /tmp/generated.skill

Does not overwrite standard_m1.skill (hand-maintained complex skills remain there).
Use this to draft batches of template-shaped skills, then paste/review into the pack.
"""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]  # content/
CATALOG = ROOT / "standard" / "skill_catalog.toml"


def load_toml(path: Path) -> dict:
    text = path.read_text(encoding="utf-8")
    try:
        import tomllib

        return tomllib.loads(text)
    except ModuleNotFoundError:
        pass
    try:
        import tomli  # type: ignore

        return tomli.loads(text)
    except ModuleNotFoundError:
        pass
    # Minimal fallback: no external dep — use a tiny subset parser for our catalog shape
    return _minimal_toml_skills(text)


def _minimal_toml_skills(text: str) -> dict:
    """Parse only [[skills]] tables and [templates.*] dsl strings for this repo layout."""
    import re

    templates: dict = {}
    for m in re.finditer(
        r"\[templates\.(\w+)\](.*?)(?=\n\[|\Z)",
        text,
        re.S,
    ):
        name, body = m.group(1), m.group(2)
        dm = re.search(r'dsl\s*=\s*"""(.*?)"""', body, re.S)
        if dm:
            templates[name] = {"dsl": dm.group(1)}
    skills = []
    for block in re.split(r"\n\[\[skills\]\]\s*\n", text)[1:]:
        entry: dict = {}
        for line in block.splitlines():
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            if line.startswith("["):
                break
            if "=" not in line:
                continue
            k, v = line.split("=", 1)
            k, v = k.strip(), v.strip()
            if v.startswith('"') and v.endswith('"'):
                entry[k] = v[1:-1]
            else:
                try:
                    entry[k] = int(v)
                except ValueError:
                    entry[k] = v
        if entry:
            skills.append(entry)
    return {"templates": templates, "skills": skills}


def main() -> int:
    data = load_toml(CATALOG)
    templates = data.get("templates", {})
    skills = data.get("skills", [])
    out: list[str] = [
        "// AUTO-GENERATED from skill_catalog.toml — review before merging",
        f"// source: {CATALOG.relative_to(ROOT.parent)}",
        "",
    ]
    for sk in skills:
        tname = sk.get("template")
        if not tname or tname not in templates:
            print(f"skip {sk.get('name')}: unknown template {tname}", file=sys.stderr)
            continue
        tpl = templates[tname]["dsl"]
        params = {
            "name": sk["name"],
            "id": sk["id"],
            "n": sk.get("n", 1),
            "target": sk.get("target", "attack_range"),
            "as_kind": sk.get("as_kind", "sha"),
            "window": sk.get("window", "play_phase"),
            "filter": sk.get("filter", "any"),
        }
        try:
            block = tpl.format(**params).strip()
        except KeyError as e:
            print(f"skip {sk.get('name')}: missing param {e}", file=sys.stderr)
            continue
        note = sk.get("note", "")
        if note:
            out.append(f"// {note}")
        out.append(block)
        out.append("")
    sys.stdout.write("\n".join(out))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
