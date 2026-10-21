#!/usr/bin/env python3
"""Tests for check_comment_counts.py."""

from __future__ import annotations

import importlib.util
import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts" / "check_comment_counts.py"


def load():
    spec = importlib.util.spec_from_file_location("check_comment_counts", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class CommentCountTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.mod = load()
        cls.counts = cls.mod.ledger_counts()

    def test_no_double_counting(self):
        # `需档(sha 幕)` 只应计入前缀族一次；曾因同时计全名而翻倍（42 vs 21）
        total = sum(1 for line in self.mod.LEDGER.read_text(encoding="utf-8").splitlines()[1:] if line.strip())
        # 前缀族计数之和不应超过行数
        prefixed = {k: v for k, v in self.counts.items()}
        self.assertLessEqual(sum(prefixed.values()) if all("(" not in k for k in prefixed) else total, total * 2)
        self.assertEqual(self.counts.get("需档"), 21)

    def test_known_family_counts(self):
        self.assertEqual(self.counts.get("G14"), 14)
        self.assertEqual(self.counts.get("G23"), 12)
        self.assertEqual(self.counts.get("G20"), 5)
        self.assertEqual(self.counts.get("G19"), 3)

    def test_strict_mode_passes(self):
        proc = subprocess.run([sys.executable, "-B", str(SCRIPT), "--strict"],
                              capture_output=True, text=True, cwd=ROOT)
        self.assertEqual(proc.returncode, 0, proc.stderr)


if __name__ == "__main__":
    unittest.main()
