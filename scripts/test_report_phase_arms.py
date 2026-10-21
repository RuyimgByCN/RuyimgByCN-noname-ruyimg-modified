#!/usr/bin/env python3
"""Tests for report_phase_arms.py."""

from __future__ import annotations

import importlib.util
import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts" / "report_phase_arms.py"


def load():
    spec = importlib.util.spec_from_file_location("report_phase_arms", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class EnumerationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.mod = load()
        cls.rows = cls.mod.arms()

    def test_counts_every_phase_pinned_arm(self):
        self.assertEqual(len(self.rows), 48)

    def test_every_row_has_a_behavior(self):
        for behavior, *_ in self.rows:
            self.assertNotEqual(behavior, "?", f"未解析到 HostBehavior：{self.rows}")

    def test_phases_are_the_five_known_ones(self):
        self.assertEqual({r[1] for r in self.rows}, {"Judge", "Draw", "Play", "Discard", "End"})

    def test_begin_is_boolean_literal(self):
        self.assertEqual({r[2] for r in self.rows}, {"true", "false"})


class ClassificationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.mod = load()
        cls.verdicts = [
            cls.mod.classify(b, p, bg, cls.mod.upstream_trigger(cls.mod.skill_names().get(cls.mod.host_keys().get(b, ""), ""),
                                                             cls.mod.host_keys().get(b, "").removeprefix("host:")))
            for b, p, bg, _f, _l in cls.mod.arms()
        ]

    def test_g14_count_matches_manual_audit(self):
        self.assertEqual(sum(1 for v in self.verdicts if v.startswith("G14")), 11)

    def test_g20_count_matches_manual_audit(self):
        self.assertEqual(sum(1 for v in self.verdicts if v.startswith("G20")), 5)

    def test_phase_name_mapping(self):
        m = self.mod.PHASE_STEM
        self.assertEqual(m["Play"], "Use")
        self.assertEqual(m["End"], "Jieshu")

    def test_classify_phase_begin_is_g14(self):
        self.assertTrue(self.mod.classify("X", "Judge", "true", 'player: "phaseBegin"').startswith("G14"))

    def test_classify_phase_end_true_is_g20(self):
        self.assertTrue(self.mod.classify("X", "End", "true", 'player: "phaseEnd"').startswith("G20"))

    def test_classify_matching_begin_is_ok(self):
        self.assertEqual(self.mod.classify("X", "Play", "true", 'player: "phaseUseBegin"'), "OK")
        self.assertEqual(self.mod.classify("X", "End", "false", 'player: "phaseJieshuEnd"'), "OK")


class CliTests(unittest.TestCase):
    def test_check_exits_one_while_review_rows_remain(self):
        proc = subprocess.run([sys.executable, "-B", str(SCRIPT), "--check"],
                              capture_output=True, text=True, cwd=ROOT)
        self.assertEqual(proc.returncode, 1)
        self.assertIn("REVIEW", proc.stderr)


if __name__ == "__main__":
    unittest.main()
