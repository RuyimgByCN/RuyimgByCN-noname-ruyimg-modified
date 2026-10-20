#!/usr/bin/env python3
"""`map_package_sources.py` 的行为测试（随 `unittest discover -s scripts` 一起跑）。"""

from __future__ import annotations

import unittest

import map_package_sources as mps


class MappingTest(unittest.TestCase):
    def test_ledger_has_37_sources(self) -> None:
        self.assertEqual(len(mps.ledger_sources()), 37)

    def test_all_packages_mapped(self) -> None:
        by_source, unmapped = mps.mapping()
        self.assertEqual(unmapped, [])
        self.assertEqual(sum(len(v) for v in by_source.values()), 41)

    def test_32_sources_have_packages(self) -> None:
        by_source, _ = mps.mapping()
        self.assertEqual(len(by_source), mps.REQUIRED)

    def test_five_sources_have_no_package(self) -> None:
        by_source, _ = mps.mapping()
        self.assertEqual(set(mps.ledger_sources()) - set(by_source), mps.NO_PACKAGE)

    def test_disambiguation_by_suffix(self) -> None:
        by_source, _ = mps.mapping()
        # liuli 的 cardpile 不能记到 noname 的 U03
        self.assertIn("cardpile-upstream-liuli", by_source["L04"])
        self.assertIn("cardpile-upstream-noname", by_source["U03"])
        self.assertIn("boss-play-upstream-liuli", by_source["L03"])
        self.assertIn("boss-zhugeliang-upstream-liuli", by_source["L03"])

    def test_multi_package_sources(self) -> None:
        by_source, _ = mps.mapping()
        self.assertEqual(len(by_source["U05"]), 9)
        self.assertEqual(len(by_source["L03"]), 2)


if __name__ == "__main__":
    unittest.main()
