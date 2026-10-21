#!/usr/bin/env python3
"""Tests for plan_effect_request_split.py."""

from __future__ import annotations

import importlib.util
import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts" / "plan_effect_request_split.py"


def load():
    spec = importlib.util.spec_from_file_location("plan_effect_request_split", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class ParseTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.mod = load()
        cls.variants, cls.body = load().parse_variants(load().EFFECT_RS)

    def test_variant_count_is_stable(self):
        # 815 is the measured count at rules v322 / effects file 10778 lines.
        self.assertEqual(len(self.variants), 815)

    def test_spans_cover_the_enum_body(self):
        total = sum(span for _n, _f, span in self.variants)
        self.assertGreaterEqual(total, self.body - 5)
        self.assertLessEqual(total, self.body + 5)

    def test_names_are_unique_and_capitalised(self):
        names = [n for n, _f, _s in self.variants]
        self.assertEqual(len(names), len(set(names)))
        for name in names:
            self.assertRegex(name, r"^[A-Z][A-Za-z0-9_]*$")

    def test_first_variant_is_draw_cards(self):
        self.assertEqual(self.variants[0][0], "DrawCards")

    def test_no_span_reaches_the_whole_enum(self):
        for name, _first, span in self.variants:
            self.assertLess(span, self.body, f"{name} span {span} looks like a parse failure")


class GroupingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.mod = load()
        cls.variants, _body = cls.mod.parse_variants(cls.mod.EFFECT_RS)
        cls.grouped = cls.mod.plan(cls.variants)
        cls.sizes = {m: sum(s for _n, s in items) for m, items in cls.grouped.items()}

    def test_every_variant_lands_in_exactly_one_group(self):
        self.assertEqual(sum(len(items) for items in self.grouped.values()), len(self.variants))

    def test_known_buckets_are_within_the_limit(self):
        for module in ("draw", "damage", "cards", "choice", "skills", "marks", "turn",
                       "life", "emit", "judge", "sha", "givegain", "target", "misc"):
            with self.subTest(module=module):
                self.assertLessEqual(self.sizes[module], self.mod.LIMIT)

    def test_resolve_is_the_only_oversized_bucket(self):
        over = [m for m, size in self.sizes.items() if size > self.mod.LIMIT]
        self.assertEqual(over, ["resolve"])

    def test_resolve_needs_at_least_three_way_split(self):
        self.assertGreater(self.sizes["resolve"] / self.mod.LIMIT, 2)


class CliTests(unittest.TestCase):
    def test_report_runs_and_exits_zero(self):
        proc = subprocess.run(
            [sys.executable, "-B", str(SCRIPT)], capture_output=True, text=True, cwd=ROOT
        )
        self.assertEqual(proc.returncode, 0, proc.stderr)
        self.assertIn("variants 815", proc.stdout)
        self.assertIn("resolve", proc.stdout)

    def test_check_flags_the_unsplit_resolve_bucket(self):
        proc = subprocess.run(
            [sys.executable, "-B", str(SCRIPT), "--check"], capture_output=True, text=True, cwd=ROOT
        )
        self.assertEqual(proc.returncode, 1)
        self.assertIn("exceed", proc.stdout)


if __name__ == "__main__":
    unittest.main()
