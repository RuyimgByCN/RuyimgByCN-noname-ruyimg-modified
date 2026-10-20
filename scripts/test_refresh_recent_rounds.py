#!/usr/bin/env python3
"""`refresh_recent_rounds.py` 的行为测试（随 `unittest discover -s scripts` 一起跑）。"""

from __future__ import annotations

import unittest

import refresh_recent_rounds as rr


class RenderTest(unittest.TestCase):
    def test_render_contains_numbered_lines(self) -> None:
        items = [(1, "第一轮：做了一件事"), (2, "第二轮：又做了一件")]
        text = rr.render(items)
        self.assertIn("第 1 轮", text)
        self.assertIn("第 2 轮", text)
        self.assertIn(rr.BEGIN, text)
        self.assertIn(rr.END, text)

    def test_render_is_deterministic(self) -> None:
        items = [(3, "x"), (1, "y")]
        self.assertEqual(rr.render(items), rr.render(items))


class RoundsTest(unittest.TestCase):
    def test_rounds_takes_last_n_sorted(self) -> None:
        body = "\n".join(f"> **第 {i} 轮**：内容 {i}" for i in range(1, 30))
        got = rr.rounds(body)
        self.assertEqual(len(got), rr.N)
        self.assertEqual(got[-1][0], 29)
        self.assertEqual(got[0][0], 29 - rr.N + 1)

    def test_rounds_dedupes(self) -> None:
        body = "> **第 1 轮**：a\n> **第 1 轮**：a\n> **第 2 轮**：b"
        self.assertEqual(len(rr.rounds(body)), 2)


if __name__ == "__main__":
    unittest.main()
