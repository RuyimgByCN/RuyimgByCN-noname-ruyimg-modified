#!/usr/bin/env python3
"""`report_ledger_diff.py` 的单测。"""

from __future__ import annotations

import unittest

import report_ledger_diff as rld


class LedgerDiffTests(unittest.TestCase):
    def test_behaviors_skips_header(self) -> None:
        text = "id\tclass\tbehavior\nG1\tA\tFoo\nG2\tB\tBar\n"
        self.assertEqual(rld.behaviors(text), {"Foo": "G1", "Bar": "G2"})

    def test_behaviors_ignores_short_lines(self) -> None:
        text = "h\nG1\tA\nG1\tA\tFoo\n"
        self.assertEqual(rld.behaviors(text), {"Foo": "G1"})

    def test_diff_splits(self) -> None:
        old = {"A": "1", "B": "2", "C": "3"}
        new = {"B": "2", "C": "3", "D": "4"}
        removed, added, kept = rld.diff(old, new)
        self.assertEqual(removed, ["A"])
        self.assertEqual(added, ["D"])
        self.assertEqual(kept, ["B", "C"])

    def test_diff_identical(self) -> None:
        same = {"A": "1"}
        self.assertEqual(rld.diff(same, same), ([], [], ["A"]))


if __name__ == "__main__":
    unittest.main()
