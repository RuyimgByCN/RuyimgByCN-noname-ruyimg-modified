#!/usr/bin/env python3
"""`check_local_links.py` 的行为测试（随 `unittest discover -s scripts` 一起跑）。"""

from __future__ import annotations

import contextlib
import io
import tempfile
import unittest
from pathlib import Path
from unittest import mock

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

    def test_all_skips_evidence_raw_and_rejects_links_to_raw(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            docs = root / "docs"
            raw_file = docs / "operations" / "evidence-raw" / "ci" / "raw.md"
            raw_file.parent.mkdir(parents=True)
            raw_file.write_text("[broken](missing.md)", encoding="utf-8")
            ok_doc = docs / "operations" / "ok.md"
            ok_doc.write_text("[self](ok.md)", encoding="utf-8")
            bad_ref_doc = docs / "operations" / "bad-ref.md"
            bad_ref_doc.write_text("[raw](evidence-raw/ci/raw.md)", encoding="utf-8")

            with (
                mock.patch.object(cl, "ROOT", root),
                mock.patch.object(cl, "DOCS", docs),
                contextlib.redirect_stdout(io.StringIO()),
            ):
                self.assertEqual(cl.doc_files(), [bad_ref_doc, ok_doc])
                self.assertEqual(cl.check(bad_ref_doc), ["evidence-raw/ci/raw.md"])
                bad_ref_doc.unlink()
                self.assertEqual(cl.main(["--all"]), 0)


if __name__ == "__main__":
    unittest.main()
