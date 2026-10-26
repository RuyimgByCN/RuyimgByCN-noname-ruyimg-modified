#!/usr/bin/env python3
"""Regression tests for the functional EffectRequest split auditor."""

from __future__ import annotations

import importlib.util
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts" / "plan_effect_request_split.py"
# Full documented definitions from the pre-split rules-v322 enum, in Serde order.
BASELINE_VARIANTS = 815
BASELINE_SHA256 = "f7d1026e7af40b0586e2e2ded9c70a702f9a084da667dce5f5f231663d2ef873"


def load():
    spec = importlib.util.spec_from_file_location("plan_effect_request_split", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def arm(name, extra=""):
    return """    (FIRST [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Documented FIRST.
    FIRST,
EXTRA        ] }
    };
""".replace("FIRST", name).replace("EXTRA", extra)


def family(*arms):
    return "macro_rules! variants {\n" + "".join(arms) + "}\npub(super) use variants;\n"


class SplitTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.mod = load()
        cls.variants, cls.sizes = cls.mod.read_split()

    def test_existing_variant_definitions_and_ordinals_are_unchanged(self):
        self.assertGreaterEqual(len(self.variants), BASELINE_VARIANTS)
        self.assertEqual(
            self.mod.variant_digest(self.variants[:BASELINE_VARIANTS]), BASELINE_SHA256
        )

    def test_names_remain_unique(self):
        names = [name for name, _definition in self.variants]
        self.assertEqual(len(names), len(set(names)))
        self.assertEqual(names[0], "DrawCards")

    def test_every_maintained_effect_file_is_within_limit(self):
        for path, lines in self.sizes.items():
            with self.subTest(path=path):
                self.assertLessEqual(lines, self.mod.LIMIT)

    def test_digest_detects_docs_defaults_and_field_order_changes(self):
        original = [("Alpha", '/// old\nAlpha { #[serde(default = "no_player")] player: PlayerId, count: u32 },')]
        for changed in (
            original[0][1].replace("old", "new"),
            original[0][1].replace("no_player", "other_player"),
            original[0][1].replace("player: PlayerId, count: u32", "count: u32, player: PlayerId"),
        ):
            self.assertNotEqual(
                self.mod.variant_digest(original), self.mod.variant_digest([("Alpha", changed)])
            )


class MalformedSplitTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.mod = load()

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.path = Path(self.temp.name) / "effect.rs"
        self.directory = self.path.with_suffix("")
        self.directory.mkdir()
        self.path.write_text(
            "mod cards;\nmod turns;\ncollect_variants! { [\n"
            "    (cards, Alpha),\n    (turns, Beta)\n] [] }\n",
            encoding="utf-8",
        )
        (self.directory / "cards.rs").write_text(family(arm("Alpha")), encoding="utf-8")
        (self.directory / "turns.rs").write_text(family(arm("Beta")), encoding="utf-8")

    def replace_registry(self, before, after):
        self.path.write_text(self.path.read_text().replace(before, after), encoding="utf-8")

    def test_fixture_reconstructs_registry_order(self):
        definitions, sizes = self.mod.read_split(self.path)
        self.assertEqual([name for name, _text in definitions], ["Alpha", "Beta"])
        self.assertEqual(len(sizes), 3)

    def test_duplicate_registry_segment_is_rejected(self):
        self.replace_registry("(cards, Alpha),", "(cards, Alpha), (cards, Alpha),")
        with self.assertRaisesRegex(ValueError, "duplicate registry segment"):
            self.mod.read_split(self.path)

    def test_missing_registry_segment_is_rejected(self):
        self.replace_registry("(cards, Alpha)", "(cards, Missing)")
        with self.assertRaisesRegex(ValueError, "missing segments:.*Missing"):
            self.mod.read_split(self.path)

    def test_unused_defined_segment_is_rejected(self):
        (self.directory / "cards.rs").write_text(family(arm("Alpha"), arm("Unused")))
        with self.assertRaisesRegex(ValueError, "unused segments:.*Unused"):
            self.mod.read_split(self.path)

    def test_duplicate_defined_segment_is_rejected(self):
        (self.directory / "cards.rs").write_text(family(arm("Alpha"), arm("Alpha")))
        with self.assertRaisesRegex(ValueError, "duplicate defined segment"):
            self.mod.read_split(self.path)

    def test_unused_module_is_rejected(self):
        self.replace_registry("mod cards;", "mod cards;\nmod unused;")
        with self.assertRaisesRegex(ValueError, "declared and used modules differ"):
            self.mod.read_split(self.path)

    def test_duplicate_variants_across_segments_are_rejected(self):
        (self.directory / "turns.rs").write_text(family(arm("Beta", "    Alpha,\n")))
        with self.assertRaisesRegex(ValueError, "duplicate variant definitions"):
            self.mod.read_split(self.path)

    def test_unparsed_macro_text_is_rejected(self):
        path = self.directory / "cards.rs"
        path.write_text(path.read_text().replace("    Alpha,", "    unsupported!(),"))
        with self.assertRaisesRegex(ValueError, "invalid variant definition"):
            self.mod.read_split(self.path)


class CliTests(unittest.TestCase):
    def run_cli(self, *args):
        return subprocess.run(
            [sys.executable, "-B", str(SCRIPT), *args], capture_output=True, text=True, cwd=ROOT
        )

    def test_report_and_check_succeed(self):
        for args in ((), ("--check",)):
            with self.subTest(args=args):
                proc = self.run_cli(*args)
                self.assertEqual(proc.returncode, 0, proc.stderr)
                self.assertIn("functional modules", proc.stdout)
                self.assertIn("ordered definition SHA256", proc.stdout)

    def test_check_enforces_actual_file_sizes(self):
        proc = self.run_cli("--check", "--limit", "1")
        self.assertEqual(proc.returncode, 1, proc.stderr)
        self.assertIn("OVER", proc.stdout)


if __name__ == "__main__":
    unittest.main()
