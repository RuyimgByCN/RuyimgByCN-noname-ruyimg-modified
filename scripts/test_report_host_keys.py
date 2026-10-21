#!/usr/bin/env python3
"""Tests for report_host_keys.py."""

from __future__ import annotations

import importlib.util
import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts" / "report_host_keys.py"


def load():
    spec = importlib.util.spec_from_file_location("report_host_keys", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class HostKeyTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.mod = load()
        cls.per_package = cls.mod.survey()

    def test_all_expected_packages_exist(self):
        for name in self.mod.EXPECTED_U05:
            self.assertTrue((ROOT / "packages" / "extensions" / name).is_dir(), name)

    def test_u05_has_nine_packages(self):
        # 第 776 轮更正：先前只算 8 个，漏了 shaai-xianjian(75) 与 shahai-zhenfa(0)
        self.assertEqual(len(self.mod.EXPECTED_U05), 9)

    def test_u05_total(self):
        total = sum(self.per_package.get(name, 0) for name in self.mod.EXPECTED_U05)
        self.assertEqual(total, 314)

    def test_xianjian_package_is_counted(self):
        # `shaai-xianjian`（75 键）曾被漏掉；`shahai-xianjian` 确实是 0
        self.assertEqual(self.per_package.get("shaai-xianjian-upstream-noname"), 75)
        self.assertEqual(self.per_package.get("shahai-xianjian-upstream-noname", 0), 0)

    def test_key_shape(self):
        keys = self.mod.keys_for("shahai-swd-upstream-noname")
        self.assertTrue(keys)
        for key in keys:
            self.assertRegex(key, r"^host:[a-z0-9_]+$")

    def test_check_mode_passes(self):
        proc = subprocess.run([sys.executable, "-B", str(SCRIPT), "--check"],
                              capture_output=True, text=True, cwd=ROOT)
        self.assertEqual(proc.returncode, 0, proc.stderr)


if __name__ == "__main__":
    unittest.main()
