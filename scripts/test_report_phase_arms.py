#!/usr/bin/env python3
"""Tests for report_phase_arms.py."""

from __future__ import annotations

import importlib.util
import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts" / "report_phase_arms.py"


def load():
    spec = importlib.util.spec_from_file_location("report_phase_arms", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class EnumerationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.mod = load()
        cls.rows = cls.mod.arms()

    def test_counts_every_phase_pinned_arm(self):
        self.assertEqual(len(self.rows), 53)

    def test_every_row_has_a_behavior(self):
        for behavior, *_ in self.rows:
            self.assertNotEqual(behavior, "?", f"未解析到 HostBehavior：{self.rows}")

    def test_phases_are_the_five_known_ones(self):
        self.assertEqual({r[1] for r in self.rows}, {"Judge", "Draw", "Play", "Discard", "End"})

    def test_begin_is_boolean_literal(self):
        self.assertEqual({r[2] for r in self.rows}, {"true", "false"})


class ClassificationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.mod = load()
        keys = cls.mod.host_keys()
        skills = cls.mod.skill_names()
        cls.verdicts = [
            cls.mod.classify(b, phase, bg, cls.mod.resolve_trigger(b, keys.get(b, ""), skills.get(keys.get(b, ""), "")))
            for b, phase, bg, _f, _l in cls.mod.arms()
        ]

    def test_g14_count_matches_manual_audit(self):
        # 14 = 11 + `SwdHujing`（相等比较写法）+ `Qinglongzhigui`/`Huanglinzhicong`
        # （效果挂在卡片 `global: g_<name>` 技能上，第 737 轮补查）
        self.assertEqual(sum(1 for v in self.verdicts if v.startswith("G14")), 1)

    def test_g20_count_matches_manual_audit(self):
        self.assertEqual(sum(1 for v in self.verdicts if v.startswith("G20")), 5)

    def test_phase_name_mapping(self):
        m = self.mod.PHASE_STEM
        self.assertEqual(m["Play"], "Use")
        self.assertEqual(m["End"], "Jieshu")

    def test_classify_judge_is_turn_start(self):
        # 原生无准备阶段 ⇒ Judge 是首相位 ⇒ 钉 Judge 等价于回合开始（第 804 轮）
        self.assertEqual(self.mod.classify("X", "Judge", "true", 'player: "phaseBegin"'), "OK")

    def test_classify_nonfirst_phase_is_g14(self):
        # 钉在 End（非首相位）才是真错位
        self.assertTrue(self.mod.classify("X", "End", "true", 'player: "phaseBegin"').startswith("G14"))

    def test_classify_phase_end_true_is_g20(self):
        self.assertTrue(self.mod.classify("X", "End", "true", 'player: "phaseEnd"').startswith("G20"))

    def test_classify_matching_begin_is_ok(self):
        self.assertEqual(self.mod.classify("X", "Play", "true", 'player: "phaseUseBegin"'), "OK")
        self.assertEqual(self.mod.classify("X", "End", "false", 'player: "phaseJieshuEnd"'), "OK")


class CliTests(unittest.TestCase):
    def test_check_passes_when_every_row_is_classified(self):
        # 第 768 轮：`miedao` 作为复合技补进 MANUAL 后，53 条全部有判定 ⇒ --check 归零
        proc = subprocess.run([sys.executable, "-B", str(SCRIPT), "--check"],
                              capture_output=True, text=True, cwd=ROOT)
        self.assertEqual(proc.returncode, 0, proc.stderr)
        self.assertIn("共 53 个相位钉死臂", proc.stderr)


if __name__ == "__main__":
    unittest.main()
