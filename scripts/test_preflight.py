#!/usr/bin/env python3
"""Tests for preflight.py."""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts" / "preflight.py"


def load():
    spec = importlib.util.spec_from_file_location("preflight", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class PreflightTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.mod = load()

    def test_every_check_script_exists(self):
        for name, argv, _slow in self.mod.CHECKS:
            target = next((arg for arg in argv if arg.startswith("scripts/")), None)
            if target:
                self.assertTrue((ROOT / target).exists(), f"{name}: 缺 {target}")

    def test_expected_failures_are_declared_checks(self):
        names = {name for name, _argv, _slow in self.mod.CHECKS}
        for name in self.mod.EXPECTED_FAILURES:
            self.assertIn(name, names, f"预期失败项 {name} 不在检查列表里")

    def test_line_gate_is_the_known_failure(self):
        # 行数门禁目前因 `crates/rule-api/src/effect.rs` 超限而预期失败
        self.assertEqual(self.mod.EXPECTED_FAILURES, {"行数门禁"})

    def test_all_checks_use_python_B(self):
        # `-B` 防止脚本生成 __pycache__（本仓库禁止提交缓存）
        for name, argv, _slow in self.mod.CHECKS:
            self.assertIn("-B", argv, name)


if __name__ == "__main__":
    unittest.main()
