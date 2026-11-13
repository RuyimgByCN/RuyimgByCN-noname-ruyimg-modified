#!/usr/bin/env python3
"""`report_phase_filters.py` 的行为测试（随 `unittest discover -s scripts` 一起跑）。"""

from __future__ import annotations

import unittest

import report_phase_filters as rp


class ScanTest(unittest.TestCase):
    def test_scans_phase_filters(self) -> None:
        rows = rp.scan()
        self.assertEqual(len(rows), 37, "阶段过滤条数变了 ⇒ 先核对是否有新增/删除")

    def test_rows_are_well_formed(self) -> None:
        for beh, key, phase, begin in rp.scan():
            self.assertRegex(beh, r"^\w+$")
            self.assertIn(phase, {"Prepare", "Judge", "Draw", "Play", "Discard", "End"})
            self.assertIn(begin, {"true", "false"})

    def test_begin_true_count(self) -> None:
        # 2026-11-13：两 Boss、侠行和魔王新增四个 begin=true 过滤分支。
        n = sum(1 for r in rp.scan() if r[3] == "true")
        self.assertEqual(n, 21, "begin=true 的条数变了")

    def test_phase_distribution(self) -> None:
        from collections import Counter

        c = Counter(r[2] for r in rp.scan())
        # 原32条保留，新增 Prepare 两条、End 三条。
        self.assertEqual(
            dict(c),
            {"Prepare": 4, "Judge": 1, "Draw": 7, "Play": 8, "Discard": 3, "End": 14},
        )


class RenderTest(unittest.TestCase):
    def test_render_keeps_status_columns(self) -> None:
        text = rp.render(rp.scan())
        lines = text.strip().splitlines()
        self.assertEqual(lines[0], "行为\tkey\tPhase\tbegin\t状态\t依据")
        self.assertEqual(len(lines), 38)
        for line in lines[1:]:
            cells = line.split("\t")
            self.assertEqual(len(cells), 6)
            self.assertIn(cells[4], {"已核", "不适用", "未核"})

    def test_status_counts(self) -> None:
        from collections import Counter

        c = Counter(line.split("\t")[4] for line in rp.render(rp.scan()).strip().splitlines()[1:])
        self.assertEqual(c["已核"] + c["不适用"] + c["未核"], 37)


if __name__ == "__main__":
    unittest.main()


class ArmSlicingTest(unittest.TestCase):
    """钉住「段切片」规则：37 条，且 `Xiefang` 的 Draw 分支必须在。"""

    def test_arms_count_matches(self) -> None:
        rows = rp.scan()
        self.assertEqual(len(rows), 37)

    def test_xiefang_has_draw_branch(self) -> None:
        got = [(p, b) for beh, _key, p, b in rp.scan() if beh == "Xiefang"]
        self.assertIn(("Draw", "false"), got, "Xiefang 的 Draw 分支丢了 ⇒ 切片规则被改坏")

    def test_slicing_rule_documented(self) -> None:
        doc = rp.__doc__ or ""
        self.assertIn("段切片", rp.arms.__doc__ or "")
