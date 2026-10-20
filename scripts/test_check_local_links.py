#!/usr/bin/env python3
"""`check_local_links.py` 的行为测试（随 `unittest discover -s scripts` 一起跑）。"""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

import check_local_links as cl


class CheckTest(unittest.TestCase):
    def test_existing_and_missing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "target.md").write_text("x", encoding="utf-8")
            doc = root / "doc.md"
            doc.write_text(
                "[ok](target.md) [bad](missing.md) [web](https://example.com) [a](#sec)",
                encoding="utf-8",
            )
            self.assertEqual(cl.check(doc), ["missing.md"])

    def test_nested_parent(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "a").mkdir()
            (root / "b").mkdir()
            (root / "b" / "t.md").write_text("x", encoding="utf-8")
            doc = root / "a" / "doc.md"
            doc.write_text("[up](../b/t.md)", encoding="utf-8")
            self.assertEqual(cl.check(doc), [])


if __name__ == "__main__":
    unittest.main()
