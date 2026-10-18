import importlib.util
from pathlib import Path
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location(
    "source_lines", Path(__file__).with_name("check-source-lines.py")
)
CHECK = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECK)


class SourceLinesTest(unittest.TestCase):
    def test_limit_includes_nested_tests_and_skips_dependencies_and_snapshots(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name, count in {
                "crates/example/src/lib.rs": 2000,
                "crates/example/tests/scenarios/choices.rs": 2001,
                "apps/client/src/main.ts": 2001,
                "apps/client/node_modules/vendor/index.js": 2001,
                "content/original/legacy-source/skill.js": 2001,
                "content/standard/skills.skill": 2001,
                "crates/example/target/generated.rs": 2001,
                "fuzz/fuzz_targets/run_turn.rs": 2001,
                "tests/fixtures/extension/index.js": 2001,
            }.items():
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("// 測試\n" * (count - 1) + "// last", encoding="utf-8")
            self.assertEqual(
                dict(CHECK.oversized(root)),
                {
                    Path("crates/example/tests/scenarios/choices.rs"): 2001,
                    Path("apps/client/src/main.ts"): 2001,
                    Path("content/standard/skills.skill"): 2001,
                    Path("fuzz/fuzz_targets/run_turn.rs"): 2001,
                    Path("tests/fixtures/extension/index.js"): 2001,
                },
            )


if __name__ == "__main__":
    unittest.main()
