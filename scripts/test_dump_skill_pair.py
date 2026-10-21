#!/usr/bin/env python3
"""Tests for dump_skill_pair.py."""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts" / "dump_skill_pair.py"


def load():
    spec = importlib.util.spec_from_file_location("dump_skill_pair", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class DumpPairTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.mod = load()

    def test_upstream_block_found(self):
        block = self.mod.upstream_block("hutian", 400)
        self.assertIn("phaseEnd", block)

    def test_native_step_found(self):
        arm = self.mod.native_step("Hutian", 400)
        self.assertIn("Hutian", arm)
        # 应能下钻到共享步里该行为自己的分支
        self.assertIn("HostStage", arm)

    def test_missing_behaviour_is_reported(self):
        self.assertEqual(self.mod.native_step("NoSuchBehaviorXyz", 200), "(无 step 臂)")

    def test_missing_upstream_is_reported(self):
        self.assertEqual(self.mod.upstream_block("no_such_skill_xyz", 200), "(上游未找到)")


if __name__ == "__main__":
    unittest.main()
