#!/usr/bin/env python3
"""`report_registered_deviations.py` 的单测。"""

from __future__ import annotations

import unittest
from pathlib import Path

import report_registered_deviations as rrd


class RegisteredDeviationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.files = {
            Path("a.rs"): 'HostBehavior::Foo => { /* 登记差异：上游自选 */ }\n'
                          'HostBehavior::Bar => { /* 无 */ }\n',
            Path("b.rs"): "HostBehavior::Baz => { // 差異：确定首张\n }\n",
        }

    def test_arm_text_collects_arms(self) -> None:
        self.assertIn("登记差异", rrd.arm_text("Foo", self.files))
        self.assertIn("差異", rrd.arm_text("Baz", self.files))

    def test_arm_text_missing_behavior(self) -> None:
        self.assertEqual(rrd.arm_text("Nope", self.files), "")

    def test_registered_splits(self) -> None:
        original = rrd._rows
        rrd._rows = lambda: ["Foo", "Bar", "Baz"]  # type: ignore[assignment]
        try:
            yes, no = rrd.registered(self.files)
        finally:
            rrd._rows = original  # type: ignore[assignment]
        self.assertEqual(yes, ["Foo", "Baz"])
        self.assertEqual(no, ["Bar"])

    def test_markers_cover_both_scripts(self) -> None:
        self.assertIn("差异", rrd.MARKERS)
        self.assertIn("差異", rrd.MARKERS)


if __name__ == "__main__":
    unittest.main()
