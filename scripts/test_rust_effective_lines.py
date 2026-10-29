from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
from rust_effective_lines import effective_lines  # noqa: E402


class RustEffectiveLinesTest(unittest.TestCase):
    def test_comments_blank_lines_and_test_items_do_not_count(self):
        source = "\n".join([
            "//! crate doc",
            "use std::fmt; // trailing",
            "",
            "/* block",
            "   still comment /* nested */ */",
            "fn a() -> &'static str {",
            '    "// not a comment /* either"',
            "}",
            "fn b() -> char { '/' }",
            'fn c() -> &\'static str { r#"/* raw */"# }',
            "#[cfg(test)]",
            "mod tests {",
            "    #[test]",
            "    fn t() { assert!(true); }",
            "}",
            "#[cfg(test)]",
            "mod more;",
            "struct Kept;",
        ])
        self.assertEqual(effective_lines(source), 7)


if __name__ == "__main__":
    unittest.main()
