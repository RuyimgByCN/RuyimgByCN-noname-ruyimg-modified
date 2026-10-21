#!/usr/bin/env python3
"""Tests for report_event_arms.py."""

from __future__ import annotations

import importlib.util
import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts" / "report_event_arms.py"


def load():
    spec = importlib.util.spec_from_file_location("report_event_arms", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class MetadataTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.mod = load()
        cls.meta = cls.mod.metadata_events()

    def test_metadata_is_populated(self):
        self.assertGreater(len(self.meta), 200)

    def test_xunmeng_declares_damage_before(self):
        # 过滤臂写 EventData::Damage，但 meta 声明 DamageBefore —— 时序权威是 meta。
        self.assertEqual(self.meta.get("Xunmeng"), "DamageBefore")

    def test_known_meta_events(self):
        self.assertEqual(self.meta.get("Shishi"), "Death")
        for behavior, event in self.meta.items():
            self.assertRegex(event, r"^[A-Z][A-Za-z0-9]*$", f"{behavior} 的 event 不像 EventKind")


class MetadataClassTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.mod = load()
        cls.cls = cls.mod.metadata_classes()

    def test_two_kinds_only(self):
        self.assertEqual({k for k, _ in self.cls.values()}, {"literal", "none"})

    def test_literal_has_event_name(self):
        for behavior, (kind, event) in self.cls.items():
            if kind == "literal":
                self.assertRegex(event, r"^[A-Z][A-Za-z0-9]*$", behavior)

    def test_none_kind_has_no_event(self):
        for behavior, (kind, event) in self.cls.items():
            if kind == "none":
                self.assertEqual(event, "", behavior)

    def test_known_none_case(self):
        # 还侠：代码注释自述「无触发面（PlayCard 转化臂+kernel 跟踪钩承载）」
        self.assertEqual(self.cls["SwdHuanxia"], ("none", ""))

    def test_known_literal_case(self):
        self.assertEqual(self.cls["Shishi"], ("literal", "Death"))


class ArmTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.mod = load()
        cls.rows = cls.mod.arms()

    def test_damage_arms_are_enumerated(self):
        self.assertEqual(len(self.rows), 88)

    def test_every_row_has_behavior_and_kind(self):
        for behavior, which, *_ in self.rows:
            self.assertNotEqual(behavior, "?")
            self.assertIn(which, {"Damage", "DamageBefore"})


class ClassifyTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.mod = load()

    def test_begin_maps_to_before(self):
        self.assertEqual(self.mod.classify("DamageBefore", "source:'damageBegin'"), "OK")
        self.assertEqual(self.mod.classify("Damage", "player:'damageBegin'"), "G21(前/后错位)")

    def test_end_maps_to_after(self):
        self.assertEqual(self.mod.classify("Damage", "player:'damageEnd'"), "OK")
        self.assertEqual(self.mod.classify("DamageBefore", "global:'damageAfter'"), "G21(前/后错位)")

    def test_third_occurrence_needs_slot(self):
        self.assertEqual(self.mod.classify("DamageBefore", 'player:"damageBegin3"'), "第3次(需档位)")

    def test_no_trigger_is_review(self):
        self.assertTrue(self.mod.classify("Damage", "").startswith("REVIEW"))
        self.assertTrue(self.mod.classify("Damage", "player:'phaseEnd'").startswith("REVIEW"))


class CliTests(unittest.TestCase):
    def test_check_exits_one_while_review_rows_remain(self):
        proc = subprocess.run([sys.executable, "-B", str(SCRIPT), "--check"],
                              capture_output=True, text=True, cwd=ROOT)
        self.assertEqual(proc.returncode, 1)
        self.assertIn("REVIEW", proc.stderr)


if __name__ == "__main__":
    unittest.main()
