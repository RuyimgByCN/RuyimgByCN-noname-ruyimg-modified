#!/usr/bin/env python3
"""`check_stale_numbers.py` 的行为测试（随 `unittest discover -s scripts` 一起跑）。"""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

import check_stale_numbers as cs


def run(text: str) -> list[tuple[int, str, str, str]]:
    with tempfile.TemporaryDirectory() as tmp:
        f = Path(tmp) / "d.md"
        f.write_text(text, encoding="utf-8")
        return cs.violations(f)


class ViolationTest(unittest.TestCase):
    def test_plain_stale_hits(self) -> None:
        got = run("当前规则差异 5/32，另有组合 13/32。")
        self.assertEqual(len(got), 2)

    def test_round_log_excluded(self) -> None:
        self.assertEqual(run("> **第 12 轮**：当时规则差异 5/32"), [])
        self.assertEqual(run("> - **第 12 轮**：当时规则差异 5/32"), [])

    def test_correction_lines_excluded(self) -> None:
        for line in (
            "~~规则差异 5/32~~ → 0/32",
            "原写规则差异 5/32，已改为 0/32",
            "「规则差异 5/32」实际是生命周期覆盖数",
            "**因此（当时）**：已核 32",
        ):
            self.assertEqual(run(line), [], line)

    def test_clean_text(self) -> None:
        self.assertEqual(run("规则差异 0/32、组合 11/32"), [])


class RoundLogRangeTests(unittest.TestCase):
    def test_round_log_range_header_is_exempt(self) -> None:
        """`第 896–897 轮` 这类区间标题也算回合日志（否则其内的旧值会被误报）。"""
        import check_stale_numbers as csn

        self.assertIsNotNone(csn.ROUND_LOG.match("> **第 896–897 轮**：收尾"))
        self.assertIsNotNone(csn.ROUND_LOG.match("> **第 896-897 轮**：收尾"))
        self.assertIsNotNone(csn.ROUND_LOG.match("> **第 897 轮**：收尾"))


if __name__ == "__main__":
    unittest.main()
