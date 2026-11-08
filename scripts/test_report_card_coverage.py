#!/usr/bin/env python3
"""Tests for report_card_coverage.py."""

from __future__ import annotations

import importlib.util
import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts" / "report_card_coverage.py"


def load():
    spec = importlib.util.spec_from_file_location("report_card_coverage", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class CardCoverageTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.mod = load()
        cls.per_pack = cls.mod.pack_cards()
        cls.kinds = cls.mod.kinds()
        cls.mapping = cls.mod.catalog_map()
        cls.wired = cls.mod.wired_kinds(cls.kinds, cls.mapping)

    def test_pack_names_match_real_directories(self):
        # 包名写错会静默数到 0（第 775 轮踩过），所以逐个校验目录存在
        for pack in self.mod.PACKS:
            self.assertTrue((ROOT / "packages" / "extensions" / pack).is_dir(), pack)

    def test_card_total(self):
        self.assertEqual(sum(self.per_pack.values()), 202)

    def test_unique_kinds(self):
        self.assertEqual(len(self.kinds), 129)

    def test_all_kinds_registered(self):
        self.assertEqual(len(self.mapping.keys() & self.kinds), 129)

    def test_all_kinds_wired(self):
        # 口径：`crates/**/*.rs`（排除 catalog.rs），接受变体名或 kind 字面量
        self.assertEqual(len(self.wired), 129)
        self.assertEqual(self.kinds - self.wired, set())

    def test_empty_packs_documented(self):
        # xianjian / ow 的 cards 为空，不应出现在 PACKS 里
        self.assertNotIn("shahai-xianjian-upstream-noname", self.mod.PACKS)

    def test_check_mode_passes(self):
        proc = subprocess.run([sys.executable, "-B", str(SCRIPT), "--check"],
                              capture_output=True, text=True, cwd=ROOT)
        self.assertEqual(proc.returncode, 0, proc.stderr)


if __name__ == "__main__":
    unittest.main()
