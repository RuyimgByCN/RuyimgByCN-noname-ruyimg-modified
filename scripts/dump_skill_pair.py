#!/usr/bin/env python3
"""Dump both sides of one behaviour: upstream skill block + native step arm.

Verifying "effect equivalence" by hand means reading two files per behaviour.
This helper prints both in one place so a verification pass costs one command
per behaviour instead of four `grep`s.

Usage:
    python3 -B scripts/dump_skill_pair.py Hutian
    python3 -B scripts/dump_skill_pair.py Hutian --width 800
"""

from __future__ import annotations

import argparse
import importlib.util
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
HOST = ROOT / "crates" / "engine-native" / "src" / "content" / "host"


def _load_phase():
    spec = importlib.util.spec_from_file_location("report_phase_arms", ROOT / "scripts" / "report_phase_arms.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


PREFIXES = ("swd_", "xj_", "xianjian_", "wudian_", "hlss_", "gujian_", "yunchou_",
            "hearth_", "gwent_", "mtg_", "ow_", "boss_", "huanle_", "longxue_")


def _candidates(*names: str) -> list[str]:
    """与 `report_phase_arms.upstream_trigger_with_path` 的候选顺序保持一致。

    顺序不同会导致「工具显示的上游」与「台账解析的上游」不是同一个技能
    （第 834 轮发现：dumper 显示 `xianjian_ctianfu`，台账解析到 `dieBegin`）。
    """
    out: list[str] = []
    for name in names:
        if not name:
            continue
        out.extend([name, f"g_{name}"])
        for prefix in PREFIXES:
            if name.startswith(prefix) and len(name) > len(prefix):
                stripped = name[len(prefix):]
                out.extend([stripped, f"g_{stripped}"])
    return out


def upstream_block(name: str, width: int, key: str = "") -> str:
    phase = _load_phase()
    sources = phase.upstream_sources()
    for candidate in _candidates(name, key.removeprefix("host:")):
        patterns = [
            r'\n\s*"?' + re.escape(candidate) + r'"?\s*:\s*\{',
            r"lib\.skill\." + re.escape(candidate) + r"\s*=\s*\{",
        ]
        for _path, text in sources.items():
            for pattern in patterns:
                match = re.search(pattern, text)
                if not match:
                    continue
                block = phase._block(text, match.end())
                return " ".join(block.split())[:width]
    return "(上游未找到)"


def native_step(behavior: str, width: int) -> str:
    files = list(HOST.glob("step*.rs")) + list(HOST.glob("step/*.rs"))
    for path in files:
        text = path.read_text(encoding="utf-8")
        start = text.find(f"HostBehavior::{behavior} =>")
        if start < 0:
            continue
        end = text.find("HostBehavior::", start + 10)
        arm = " ".join(text[start : (end if end > 0 else start + 900)].split())
        # 沿共享步查一层（如 `self.step_skills_damage(ctx)`）
        shared = re.search(r"self\.(step_[a-z_]+)\(ctx\)", arm)
        note = ""
        if shared:
            name = shared.group(1)
            for other in files:
                body = other.read_text(encoding="utf-8")
                inner = body.find(f"fn {name}(")
                if inner < 0:
                    continue
                # 共享步内部通常再按 behavior 分派：取该行为自己的臂
                arm_inner = body.find(f"HostBehavior::{behavior} =>", inner)
                if arm_inner >= 0:
                    stop = body.find("HostBehavior::", arm_inner + 10)
                    note = "\n  ↳ " + " ".join(body[arm_inner : (stop if stop > 0 else arm_inner + width)].split())[:width]
                else:
                    note = "\n  ↳ 共享步 " + " ".join(body[inner : inner + width].split())
                break
        return f"[{path.name}] " + arm[:width] + note
    return "(无 step 臂)"


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("behavior")
    ap.add_argument("--skill", help="上游技能名（默认用行为名去前缀/去后缀后的 snake）")
    ap.add_argument("--width", type=int, default=500)
    args = ap.parse_args()

    skill = args.skill
    if not skill:
        # 优先用包内技能名（`host:` 键 → `skills[].name`）——这比从行为名推导可靠
        phase = _load_phase()
        key = phase.host_keys().get(args.behavior, "")
        skill = phase.skill_names().get(key, "")
    if not skill:
        name = args.behavior
        for suffix in ("Held", "Card", "Use", "Gain", "Remain", "2", "3"):
            if name.endswith(suffix):
                name = name[: -len(suffix)]
        skill = re.sub(r"(?<!^)(?=[A-Z])", "_", name).lower().lstrip("_")

    print(f"== {args.behavior}（上游 {skill}）")
    key = _load_phase().host_keys().get(args.behavior, "")
    print("上游:", upstream_block(skill, args.width, key))
    print("原生:", native_step(args.behavior, args.width))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
