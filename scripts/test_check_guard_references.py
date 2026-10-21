#!/usr/bin/env python3
"""Tests for check_guard_references.py."""

from __future__ import annotations

import importlib.util
import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts" / "check_guard_references.py"


def load():
    spec = importlib.util.spec_from_file_location("check_guard_references", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class GuardReferenceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.mod = load()

    def test_behaviour_variants_discovered(self):
        found = self.mod.variants(self.mod.KEYS, "HostBehavior")
        self.assertGreater(len(found), 400)
        self.assertIn("Rangjie", found)

    def test_stale_name_is_gone(self):
        # 守卫曾引用不存在的 `GujianFoodRangjie`，实际变体是 `Rangjie`
        text = self.mod.GUARDS.read_text(encoding="utf-8")
        self.assertNotIn("GujianFoodRangjie", text)

    def test_event_variants_discovered(self):
        found = self.mod.variants(self.mod.EVENT, "EventKind")
        self.assertGreater(len(found), 15)

    def test_strict_mode_passes(self):
        proc = subprocess.run([sys.executable, "-B", str(SCRIPT), "--strict"],
                              capture_output=True, text=True, cwd=ROOT)
        self.assertEqual(proc.returncode, 0, proc.stderr)


if __name__ == "__main__":
    unittest.main()
