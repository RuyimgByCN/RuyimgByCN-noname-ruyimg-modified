#!/usr/bin/env python3
"""`check_card_kind_compare.py` 的单测（第 1002 轮）。"""
import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts"))

import check_card_kind_compare as gate  # noqa: E402


class GateTests(unittest.TestCase):
    def test_runs_clean_on_current_tree(self):
        proc = subprocess.run(
            [sys.executable, "-B", str(ROOT / "scripts/check_card_kind_compare.py")],
            capture_output=True,
            text=True,
        )
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)
        self.assertIn("✅", proc.stdout)

    def test_allowlist_covers_only_known_behavior(self):
        self.assertEqual(gate.ALLOWED, {"XianjianLiguang"})

    def test_scan_finds_the_known_site(self):
        hits = gate.scan()
        self.assertTrue(any(b == "XianjianLiguang" for b, _, _ in hits), hits)

    def test_patterns_detect_a_synthetic_site(self):
        sample = "HostBehavior::Foo => {\n  let k = ctx.catalog.card_kind(*c);\n  if ctx.catalog.card_kind(*o) == k { }\n}"
        matched = any(p.search(sample) for p in gate.PATTERNS)
        self.assertFalse(matched, "把 k 存中间变量后不匹配是已知限制（文档化）")
        sample2 = "any(|o| ctx.catalog.card_kind(*o) == ctx.catalog.card_kind(*c))"
        self.assertTrue(any(p.search(sample2) for p in gate.PATTERNS))


if __name__ == "__main__":
    unittest.main()
