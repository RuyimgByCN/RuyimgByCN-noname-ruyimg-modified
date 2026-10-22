#!/usr/bin/env python3
"""Tests for check_ledger_consistency.py."""

from __future__ import annotations

import importlib.util
import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts" / "check_ledger_consistency.py"


def load():
    spec = importlib.util.spec_from_file_location("check_ledger_consistency", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class ConsistencyTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.mod = load()
        cls.ledger = cls.mod.ledger()

    def test_ledger_has_rows(self):
        # 台账从 54 收敛到 26：大量行经上游原文/测试断言复核后移入 FIXED/NON_DEFECT 注册表。
        self.assertGreater(len(self.ledger), 15)

    def test_every_id_is_registered(self):
        registered = cls = self.mod.PHASE_IDS | self.mod.EVENT_IDS | self.mod.HANDWRITTEN | {self.mod.G23_ID}
        unknown = sorted({row[0] for row in self.ledger} - registered)
        self.assertEqual(unknown, [], f"未登记的 id: {unknown}")

    def test_no_duplicate_id_behavior_pairs(self):
        pairs = [(row[0], row[2]) for row in self.ledger]
        self.assertEqual(len(pairs), len(set(pairs)))

    def test_strict_mode_passes(self):
        proc = subprocess.run([sys.executable, "-B", str(SCRIPT), "--strict"],
                              capture_output=True, text=True, cwd=ROOT)
        self.assertEqual(proc.returncode, 0, proc.stderr)


if __name__ == "__main__":
    unittest.main()
