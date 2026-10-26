#!/usr/bin/env python3
"""Tests for preflight.py."""

from __future__ import annotations

import importlib.util
import io
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from unittest.mock import patch

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

    def test_line_gate_failure_blocks_preflight(self):
        def result(name, _argv):
            return name, int(name == "行数门禁"), "oversized source"

        errors = io.StringIO()
        with patch.object(self.mod, "run", side_effect=result), \
                patch.object(self.mod.sys, "argv", [str(SCRIPT)]), \
                redirect_stdout(io.StringIO()), redirect_stderr(errors):
            self.assertEqual(self.mod.main(), 1)
        self.assertIn("preflight 失败 1 项：行数门禁", errors.getvalue())

    def test_all_passing_checks_succeed(self):
        with patch.object(self.mod, "run", side_effect=lambda name, _argv: (name, 0, "")), \
                patch.object(self.mod.sys, "argv", [str(SCRIPT)]), \
                redirect_stdout(io.StringIO()), redirect_stderr(io.StringIO()):
            self.assertEqual(self.mod.main(), 0)

    def test_all_checks_use_python_B(self):
        # `-B` 防止脚本生成 __pycache__（本仓库禁止提交缓存）
        for name, argv, _slow in self.mod.CHECKS:
            self.assertIn("-B", argv, name)


if __name__ == "__main__":
    unittest.main()
