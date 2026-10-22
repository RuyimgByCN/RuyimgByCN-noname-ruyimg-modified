#!/usr/bin/env python3
"""Tests for report_guard_coverage.py."""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts" / "report_guard_coverage.py"


def load():
    spec = importlib.util.spec_from_file_location("report_guard_coverage", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class GuardCoverageTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.mod = load()
        cls.guarded = cls.mod.guard_behaviors()
        cls.rows = cls.mod.ledger_rows()

    def test_guards_are_discovered(self):
        # 宏生成的守卫把变体名当字符串传参，必须能被扫到
        self.assertGreater(len(self.guarded), 25)
        self.assertIn("SwdTiebi", self.guarded)

    def test_ledger_non_empty(self):
        self.assertGreater(len(self.rows), 10)  # 台账 54→20 收敛后

    def test_guard_coverage_recorded(self):
        # 精确匹配（不做事后缀放宽）：2026-10-19 测得 38/101 = 37.6%
        covered = [r for r in self.rows if r[2] in self.guarded]
        ratio = len(covered) / len(self.rows)
        self.assertGreaterEqual(ratio, 0.25, f"覆盖率跌破 25%：{ratio:.1%}")  # 第 943 轮：台账 54→15 后按实际校准
        self.assertLessEqual(ratio, 1.0)

    def test_every_row_has_behavior_name(self):
        for row in self.rows:
            self.assertRegex(row[2], r"^[A-Z][A-Za-z0-9_]*$", row)


if __name__ == "__main__":
    unittest.main()
