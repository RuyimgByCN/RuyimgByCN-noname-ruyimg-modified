#!/usr/bin/env python3
"""`report_g14.py` 的行为测试（随 `unittest discover -s scripts` 一起跑）。"""

from __future__ import annotations

import unittest
from collections import Counter

import report_g14 as g14


class JudgeListTest(unittest.TestCase):
    def test_fourteen_judge_begin_true(self) -> None:
        # 实测：Phase::Judge 且 begin=true 共 14 条
        self.assertEqual(len(g14.judge_behaviors()), 14)

    def test_behavior_names_match_pattern(self) -> None:
        for beh in g14.judge_behaviors():
            self.assertRegex(beh, r"^\w+$")

    def test_key_map_covers_all(self) -> None:
        keys = g14.host_keys()
        missing = [b for b in g14.judge_behaviors() if b not in keys]
        self.assertEqual(missing, [], f"这些行为在 keys.rs 里没有 host: 键: {missing}")


class ManualTableTest(unittest.TestCase):
    def test_manual_entries_are_judge_behaviors(self) -> None:
        keys = g14.host_keys()
        beh_by_key = {v: k for k, v in keys.items()}
        for key in g14.MANUAL:
            self.assertIn(key, beh_by_key, f"MANUAL 里的 {key} 不是任何 filter 的 host 键")

    def test_manual_phasebegin_entries(self) -> None:
        # 人工表只补「自动链路查不到」的：其中 phaseBegin 类 5 条
        # （hyunshen2/daixing2/huanxing/qianfang/geju 由自动链路命中）
        #
        # 第 721 轮：`lexue` 由「无 trigger」更正为 `phaseBegin`。它的上游是复合技
        # （`group:["lexue1","lexue2"]`），真实触发面在子技能 `lexue1` 上；
        # 只读父技能会误判成「凭空新增触发」。
        g14_keys = {k for k, v in g14.MANUAL.items() if "phaseBegin" in v[1]}
        self.assertEqual(len(g14_keys), 5)
        self.assertEqual(
            g14_keys,
            {"diyguhuo", "gujian_food_yougeng", "huanle_xuelunyang", "zhenjiu2", "lexue"},
        )


class TraceTest(unittest.TestCase):
    def test_upstream_triggers_non_empty(self) -> None:
        tr = g14.upstream_triggers()
        self.assertIn("phaseBegin", tr.get("hyunshen2", ""))
        self.assertIn("phaseJieshuBegin", tr.get("fengqi", ""))

    def test_package_skill_names(self) -> None:
        names = g14.package_skill_names()
        self.assertEqual(names.get("hyunshen2"), "hyunshen2")


if __name__ == "__main__":
    unittest.main()
