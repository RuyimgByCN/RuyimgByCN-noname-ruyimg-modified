import importlib.util
from pathlib import Path
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location(
    "check_adr_index", Path(__file__).with_name("check-adr-index.py")
)
CHECK = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECK)


INDEX_HEADER = """# ADR 编号索引（INDEX）

## 编号冲突消歧表

| 编号 | 共用文件 |
|---|---|
"""


def write_adr(path: Path, name: str, heading: str) -> None:
    (path / name).write_text(f"{heading}\n\n正文\n", encoding="utf-8")


class AdrIndexCheckTest(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.adr = Path(self._tmp.name) / "docs" / "adr"
        self.adr.mkdir(parents=True)
        self.index = self.adr / "INDEX.md"

    def tearDown(self):
        self._tmp.cleanup()

    def write_index(self, rows: str = "", links: str = "") -> None:
        self.index.write_text(INDEX_HEADER + rows + "\n" + links, encoding="utf-8")

    def test_clean_tree_passes(self):
        write_adr(self.adr, "0001-first.md", "# 0001 first")
        write_adr(self.adr, "0002-second.md", "# ADR 0002：second")
        self.write_index(links="- [0001 first](0001-first.md)\n- [0002 second](0002-second.md)\n")
        report = CHECK.build_report(self.adr, self.index)
        self.assertEqual(report.failures, 0)
        self.assertEqual(report.numbered_files, 2)
        self.assertEqual(report.duplicates, {})

    def test_heading_number_mismatch_is_reported(self):
        write_adr(self.adr, "0446-clan.md", "# 0444 clan")
        self.write_index()
        report = CHECK.build_report(self.adr, self.index)
        self.assertEqual(len(report.heading_mismatches), 1)
        self.assertEqual(report.heading_mismatches[0]["heading_id"], "0444")
        self.assertEqual(report.failures, 1)

    def test_heading_without_number_is_not_a_mismatch(self):
        write_adr(self.adr, "0003-plain.md", "# 无编号标题")
        self.write_index()
        report = CHECK.build_report(self.adr, self.index)
        self.assertEqual(report.heading_mismatches, [])

    def test_unregistered_duplicate_is_a_failure(self):
        write_adr(self.adr, "0500-a.md", "# 0500 a")
        write_adr(self.adr, "0500-b.md", "# 0500 b")
        self.write_index()
        report = CHECK.build_report(self.adr, self.index)
        self.assertEqual(report.duplicates, {"0500": ["0500-a.md", "0500-b.md"]})
        self.assertEqual(report.registry_missing, ["0500"])
        self.assertEqual(report.failures, 1)

    def test_registered_duplicate_passes_and_stale_entry_fails(self):
        write_adr(self.adr, "0500-a.md", "# 0500 a")
        write_adr(self.adr, "0500-b.md", "# 0500 b")
        self.write_index(rows="| 0500 | `0500-a.md` / `0500-b.md` |\n")
        report = CHECK.build_report(self.adr, self.index)
        self.assertEqual(report.failures, 0)
        self.assertEqual(report.registry_duplicates["0500"], ["0500-a.md", "0500-b.md"])

        (self.adr / "0500-b.md").unlink()
        report = CHECK.build_report(self.adr, self.index)
        self.assertEqual(report.registry_stale, ["0500"])
        self.assertEqual(report.failures, 1)

    def test_registry_file_list_must_match_disk(self):
        write_adr(self.adr, "0500-a.md", "# 0500 a")
        write_adr(self.adr, "0500-b.md", "# 0500 b")
        self.write_index(rows="| 0500 | `0500-a.md` |\n")
        report = CHECK.build_report(self.adr, self.index)
        self.assertEqual(len(report.registry_mismatch), 1)
        self.assertEqual(report.failures, 1)

    def test_broken_index_link_is_a_failure(self):
        write_adr(self.adr, "0001-first.md", "# 0001 first")
        self.write_index(links="- [缺失](0999-missing.md)\n")
        report = CHECK.build_report(self.adr, self.index)
        self.assertEqual(report.link_missing, [{"index": "docs/adr/INDEX.md", "target": "0999-missing.md"}])
        self.assertEqual(report.failures, 1)

    def test_angle_bracket_and_anchor_links_are_handled(self):
        write_adr(self.adr, "0001-first.md", "# 0001 first")
        self.write_index(links="- [带空格](<0001-first.md#小节>)\n- [锚点](#编号冲突消歧表)\n")
        report = CHECK.build_report(self.adr, self.index)
        self.assertEqual(report.link_missing, [])

    def test_exit_code_and_json_report(self):
        write_adr(self.adr, "0001-first.md", "# 0001 first")
        write_adr(self.adr, "0001-second.md", "# 0001 second")
        self.write_index()
        self.assertEqual(CHECK.main(["--adr-dir", str(self.adr)]), 1)
        self.write_index(rows="| 0001 | `0001-first.md` / `0001-second.md` |\n")
        self.assertEqual(CHECK.main(["--adr-dir", str(self.adr), "--json"]), 0)


if __name__ == "__main__":
    unittest.main()
