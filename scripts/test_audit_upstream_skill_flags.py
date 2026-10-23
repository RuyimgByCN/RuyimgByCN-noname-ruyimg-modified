#!/usr/bin/env python3
"""`audit_upstream_skill_flags.py` 的解析单测（第 1017 轮）。

只测**解析**（括号配对 / 块切分），不测判定——判定必须人工逐条核对。
"""
import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts"))

import audit_upstream_skill_flags as tool  # noqa: E402


class ParserTests(unittest.TestCase):
    def test_brace_matching_respects_nested_objects(self):
        txt = '\n\tskill_a: {\n\t\tfilter(e, p) {\n\t\t\tif (p) { return 1; }\n\t\t},\n\t\tusable: 1,\n\t},\n\tskill_b: { enable: "phaseUse" },\n'
        blocks = tool.skill_blocks(txt)
        self.assertIn("skill_a", blocks)
        self.assertIn("usable: 1", blocks["skill_a"][0])
        self.assertNotIn("skill_b", blocks["skill_a"][0], "块不得溢出到下一个技能")
        self.assertIn("skill_b", blocks)

    def test_brace_matching_skips_string_literals(self):
        txt = '\n\ts: {\n\t\tprompt: "a { brace } inside",\n\t},\n'
        blocks = tool.skill_blocks(txt)
        self.assertEqual(len(blocks["s"]), 1)
        self.assertIn("a { brace } inside", blocks["s"][0])

    def test_inspector_runs_on_known_skill(self):
        proc = subprocess.run(
            [sys.executable, "-B", str(ROOT / "scripts/audit_upstream_skill_flags.py"), "jiefen"],
            capture_output=True,
            text=True,
        )
        self.assertEqual(proc.returncode, 0, proc.stderr)
        self.assertIn("in once_per_turn 允许表: True", proc.stdout)

    def test_usage_without_args(self):
        proc = subprocess.run(
            [sys.executable, "-B", str(ROOT / "scripts/audit_upstream_skill_flags.py")],
            capture_output=True,
            text=True,
        )
        self.assertEqual(proc.returncode, 2)


if __name__ == "__main__":
    unittest.main()
