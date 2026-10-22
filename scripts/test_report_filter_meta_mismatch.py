#!/usr/bin/env python3
"""`report_filter_meta_mismatch.py` 的单测。"""

from __future__ import annotations

import unittest

import report_filter_meta_mismatch as rfm


class FamilyTests(unittest.TestCase):
    def test_phase_begin(self) -> None:
        self.assertEqual(rfm.arm_family("EventData::Phase { begin: true }"), "PhaseBegin")

    def test_phase_end(self) -> None:
        self.assertEqual(rfm.arm_family("EventData::Phase { begin: false }"), "PhaseEnd")

    def test_phase_both(self) -> None:
        body = "EventData::Phase { begin: true } ... EventData::Phase { begin: false }"
        self.assertEqual(rfm.arm_family(body), "Phase")

    def test_card_used(self) -> None:
        self.assertEqual(rfm.arm_family("EventData::CardUsed(u)"), "CardUsed")

    def test_begin_star_positive(self) -> None:
        self.assertEqual(rfm.arm_family("EventData::Phase { begin, .. } if && *begin"), "PhaseBegin")

    def test_begin_star_negative(self) -> None:
        self.assertEqual(rfm.arm_family("EventData::Phase { begin, .. } if && !*begin"), "PhaseEnd")

    def test_unknown(self) -> None:
        self.assertEqual(rfm.arm_family("false"), "?")


class MismatchTests(unittest.TestCase):
    def test_strict_opposite_reported(self) -> None:
        metas = {"A": "PhaseEnd"}
        arms = {"A": "EventData::Phase { begin: true }"}
        self.assertEqual(rfm.mismatches(metas, arms), [("A", "PhaseEnd", "PhaseBegin")])

    def test_unspecified_filter_not_reported(self) -> None:
        metas = {"A": "PhaseEnd"}
        arms = {"A": "EventData::Phase { player }"}
        self.assertEqual(rfm.mismatches(metas, arms), [])

    def test_same_family_not_reported(self) -> None:
        metas = {"A": "PhaseBegin"}
        arms = {"A": "EventData::Phase { begin: true }"}
        self.assertEqual(rfm.mismatches(metas, arms), [])

    def test_missing_arm_skipped(self) -> None:
        self.assertEqual(rfm.mismatches({"A": "PhaseEnd"}, {}), [])


class StrictTests(unittest.TestCase):
    def test_allowed_conflicts_pass(self) -> None:
        bad = [("WudianZhucheng", "PhaseEnd", "PhaseBegin")]
        unexpected = [row for row in bad if row[0] not in rfm.ALLOWED]
        self.assertEqual(unexpected, [])

    def test_new_conflict_flagged(self) -> None:
        bad = [("SomethingNew", "PhaseEnd", "PhaseBegin")]
        unexpected = [row for row in bad if row[0] not in rfm.ALLOWED]
        self.assertEqual(len(unexpected), 1)


if __name__ == "__main__":
    unittest.main()
