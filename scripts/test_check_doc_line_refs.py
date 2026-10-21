#!/usr/bin/env python3
"""Tests for check_doc_line_refs.py."""

from __future__ import annotations

import importlib.util
import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts" / "check_doc_line_refs.py"


def load():
    spec = importlib.util.spec_from_file_location("check_doc_line_refs", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class DocLineRefTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.mod = load()

    def test_ref_regex_matches_forms(self):
        text = "见 `step/skills_cards.rs:2197` 与 `event.rs:12-20` 以及 `x.md`"
        hits = [m.group(0) for m in self.mod.REF_RE.finditer(text)]
        self.assertIn("step/skills_cards.rs:2197", hits)
        self.assertIn("event.rs:12-20", hits)

    def test_resolves_repo_file(self):
        target = self.mod.resolve("crates/engine-native/src/event.rs")
        self.assertIsNotNone(target)

    def test_resolves_upstream_archive(self):
        # 上游快照在仓库父目录的 archive/ 下
        target = self.mod.resolve("card/standard.js")
        self.assertIsNotNone(target, "应能在 ../archive 下解析到 standard.js")

    def test_report_mode_always_exits_zero(self):
        # 报告模式不阻塞：文档里现存 22 处失效引用（源码拆分导致），待清理
        proc = subprocess.run([sys.executable, "-B", str(SCRIPT)],
                              capture_output=True, text=True, cwd=ROOT)
        self.assertEqual(proc.returncode, 0, proc.stderr)

    def test_strict_mode_passes_after_cleanup(self):
        # 第 787 轮已清除全部失效行号引用 ⇒ 严格模式应为 0
        proc = subprocess.run([sys.executable, "-B", str(SCRIPT), "--strict"],
                              capture_output=True, text=True, cwd=ROOT)
        self.assertEqual(proc.returncode, 0, proc.stderr)


if __name__ == "__main__":
    unittest.main()
