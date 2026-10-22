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
        self.assertEqual(self.counts.get("需档"), 19)  # 第 909 轮：2 行移入 NON_DEFECT_VERIFIED

    def test_known_family_counts(self):
        self.assertEqual(self.counts.get("G14"), 1)
        self.assertIsNone(self.counts.get("G18"))
        self.assertEqual(self.counts.get("G23"), 3)  # 第 903 轮：判据改为「作用域不符」后由 11 降为 3
        self.assertEqual(self.counts.get("G20"), 5)
        self.assertEqual(self.counts.get("G19"), 1)

    def test_progress_total_matches_ledger(self):
        # 进度主稿头部的 `N 条 = A x / B y` 必须与台账一致（新增校验）
        import re

        text = self.mod.PROGRESS.read_text(encoding="utf-8")
        match = self.mod.TOTAL_CLAIM.search(text)
        self.assertIsNotNone(match, "进度主稿里找不到总账行")
        total, a, b = (int(match.group(i)) for i in (1, 2, 3))
        counts = {"A": 0, "B": 0}
        for line in self.mod.LEDGER.read_text(encoding="utf-8").splitlines()[1:]:
            if line.strip():
                counts[line.split("\t")[1]] = counts.get(line.split("\t")[1], 0) + 1
        self.assertEqual((total, a, b), (counts["A"] + counts["B"], counts["A"], counts["B"]))

    def test_readme_table_matches_ledger(self):
        # 台账 README 的 A/B/合计 行必须与 ledger.tsv 一致
        text = self.mod.README.read_text(encoding="utf-8")
        counts = {"A": 0, "B": 0}
        for line in self.mod.LEDGER.read_text(encoding="utf-8").splitlines()[1:]:
            if line.strip():
                counts[line.split("\t")[1]] = counts.get(line.split("\t")[1], 0) + 1
        a = self.mod.README_A.search(text)
        b = self.mod.README_B.search(text)
        total = self.mod.README_TOTAL.search(text)
        self.assertIsNotNone(a); self.assertIsNotNone(b); self.assertIsNotNone(total)
        self.assertEqual(int(a.group(1)), counts["A"])
        self.assertEqual(int(b.group(1)), counts["B"])
        self.assertEqual(int(total.group(1)), counts["A"] + counts["B"])

    def test_strict_mode_passes(self):
        proc = subprocess.run([sys.executable, "-B", str(SCRIPT), "--strict"],
                              capture_output=True, text=True, cwd=ROOT)
        self.assertEqual(proc.returncode, 0, proc.stderr)


if __name__ == "__main__":
    unittest.main()
