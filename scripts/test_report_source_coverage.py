#!/usr/bin/env python3
"""Tests for report_source_coverage.py."""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts" / "report_source_coverage.py"


def load():
    spec = importlib.util.spec_from_file_location("report_source_coverage", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class CoverageTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.mod = load()
        cls.rows = cls.mod.rows()
        cls.by_class = {}
        for row in cls.rows:
            cls.by_class.setdefault(row[4], []).append(row[0])

    def test_covers_all_37_sources(self):
        self.assertEqual(len(self.rows), 37)

    def test_class_counts(self):
        self.assertEqual(len(self.by_class["converted"]), 12)
        self.assertEqual(len(self.by_class["installed-empty"]), 25)
        self.assertEqual(len(self.by_class.get("no-package", [])), 0)

    def test_converted_sources_all_have_entries(self):
        for row in self.rows:
            if row[4] == "converted":
                self.assertGreater(int(row[2]), 0, row)

    def test_empty_sources_have_zero_entries(self):
        for row in self.rows:
            if row[4] == "installed-empty":
                self.assertEqual(row[2], "0", row)

    def test_upstream_path_present_except_pure_ui(self):
        # 三个纯配置型扩展没有任何 `upstream_path`（包内只有 config.json）：
        #   L07 omnisearch   -> { css, background }          （搜索 UI）
        #   L23 desktop-adapt -> { device_profile, layout }   （桌面适配）
        #   U04 coin          -> { coin, snowFall }           （金币特效）
        # 它们既无可转换条目，也无可追溯上游 —— 是允许的例外集合。
        missing = sorted(row[0] for row in self.rows if row[4] == "installed-empty" and not row[3])
        self.assertEqual(missing, ["L07", "L23", "U04"])

    def test_mode_sources_are_installed_empty(self):
        # 模式型来源：包在、upstream 在，但条目没搬
        for source in ("L21", "L28", "U02"):
            self.assertIn(source, self.by_class["installed-empty"])

    def test_no_package_sources(self):
        self.assertEqual(sorted(self.by_class.get("no-package", [])), [])


if __name__ == "__main__":
    unittest.main()
