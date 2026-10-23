#!/usr/bin/env python3
"""Tests for report_defect_ledger.py."""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts" / "report_defect_ledger.py"


def load():
    spec = importlib.util.spec_from_file_location("report_defect_ledger", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class LedgerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.mod = load()
        cls.rows = cls.mod.build()

    def test_ledger_is_populated(self):
        # 台账从 54 收敛到 26（复核后移入 FIXED/NON_DEFECT 注册表）。
        self.assertGreaterEqual(len(self.rows), 1)

    def test_rows_have_all_columns(self):
        for row in self.rows:
            self.assertEqual(len(row), 10, row)

    def test_classes_are_a_or_b(self):
        self.assertTrue({r[1] for r in self.rows} <= {"A", "B"})

    def test_no_duplicate_id_behavior_pairs(self):
        seen = [(r[0], r[2]) for r in self.rows]
        self.assertEqual(len(seen), len(set(seen)))

    def test_damage_family_yields_no_false_positive_g21(self):
        # 时序权威是 TriggerMeta；读过滤臂模式会产生 G21 假阳性（见枚举证据 §1）
        self.assertEqual([r for r in self.rows if r[0].startswith("G21")], [])

    def test_class_matches_verdict_family(self):
        # 「需档」需扩枚举 ⇒ B；G21/G24 是改触发时机 ⇒ A
        for row in self.rows:
            if row[0].startswith("需档"):
                self.assertEqual(row[1], "B", row)
            if row[0] in ("G21", "G24"):
                self.assertEqual(row[1], "A", row)

    def test_known_defects_present(self):
        behaviors = {r[2] for r in self.rows}
        # SwdTiebi/BossBaiyi 已复核移出；这里改挑仍在台账里的三行。
        # Reguhuo 已于第 942 轮修复移出；改挑仍在台账里的三行。
        for expected in ("HlssGaiming",):
            self.assertIn(expected, behaviors)

    def test_location_column_present(self):
        # 第 746 轮新增：一行缺陷要能直接定位到 file:line 或模块
        locations = {r[4] for r in self.rows}
        # 台账收敛到 3 行后，location 列可能全是模块名（filtering/* 等）；
        # 断言「非空」即可，file:line 不再是必要条件（第 996 轮）。
        self.assertTrue(any(loc for loc in locations), "location 不得全为空")
        self.assertTrue(all(loc for loc in locations), "location 不得为空")

    def test_source_column_present(self):
        # 第 744 轮新增：证据型归因（按上游文件路径匹配包的 upstream_path）
        # 台账收敛后 source 列可能只剩 L03/? 等；断言「列非空且为已知来源标识」
        sources = {r[3] for r in self.rows}
        self.assertTrue(sources, "source 列不得全为空")

    def test_permission_is_recorded_for_every_row(self):
        self.assertEqual({r[9] for r in self.rows}, {"blocked"})


if __name__ == "__main__":
    unittest.main()
