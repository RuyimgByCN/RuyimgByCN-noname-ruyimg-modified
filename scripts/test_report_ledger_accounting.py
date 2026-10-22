#!/usr/bin/env python3
"""`report_ledger_accounting.py` 的单测。"""

from __future__ import annotations

import unittest

import report_ledger_accounting as acct


class BehaviorTests(unittest.TestCase):
    def test_skips_header_and_short_lines(self) -> None:
        text = "id\tclass\tbehavior\nG1\tA\tFoo\nbad\nG2\tB\tBar\n"
        self.assertEqual(acct.behaviors(text), {"Foo", "Bar"})

    def test_empty(self) -> None:
        self.assertEqual(acct.behaviors(""), set())


class UnaccountedTests(unittest.TestCase):
    def test_registered_row_is_fine(self) -> None:
        reg = {"Foo": "reason"}
        self.assertEqual(acct.unaccounted({"Foo"}, [reg]), [])

    def test_missing_row_is_reported(self) -> None:
        reg = {"Foo": "reason"}
        self.assertEqual(acct.unaccounted({"Foo", "Bar"}, [reg]), ["Bar"])

    def test_multiple_registries(self) -> None:
        self.assertEqual(acct.unaccounted({"A", "B"}, [{"A": ""}, {"B": ""}]), [])

    def test_no_registries_reports_all(self) -> None:
        self.assertEqual(acct.unaccounted({"A"}, []), ["A"])


if __name__ == "__main__":
    unittest.main()
