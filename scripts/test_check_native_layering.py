import importlib.util
from pathlib import Path
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location(
    "native_layering", Path(__file__).with_name("check-native-layering.py")
)
assert SPEC is not None and SPEC.loader is not None
CHECK = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECK)


class NativeLayeringTest(unittest.TestCase):
    def test_current_kernel_has_no_upward_references(self):
        root = Path(__file__).resolve().parents[1]
        refs = list(CHECK.upward_references(root))
        self.assertEqual(len(refs), 0, f"kernel references content/engine {len(refs)} times")

    def test_counts_kernel_layer_code_but_not_comments_or_content(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name, text in {
                "kernel.rs": "use crate::content::host::HostMachine;\n",
                "vocabulary/host.rs": "use crate::content as content_alias;\n",
                "kernel/damage.rs": "x(crate::content::standard::JUE_QING); crate::engine::f();\n",
                "snap/restore.rs": "/// see content::standard::restore_content_machine\n",
                "content/host.rs": "use crate::content::standard::X;\n",
                "engine.rs": "use crate::content::host::Y;\n",
            }.items():
                path = root / CHECK.SRC / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(text, encoding="utf-8")

            refs = sorted(str(path) for path, _ in CHECK.upward_references(root))

        self.assertEqual(
            refs,
            [
                "crates/engine-native/src/kernel.rs",
                "crates/engine-native/src/kernel/damage.rs",
                "crates/engine-native/src/kernel/damage.rs",
                "crates/engine-native/src/vocabulary/host.rs",
            ],
        )


    def test_grouped_imports_and_crate_aliases_cannot_hide_upward_paths(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / CHECK.SRC / "vocabulary/shared.rs"
            path.parent.mkdir(parents=True)
            path.write_text(
                "use crate::{content::{host::HostBehavior, standard as s}, engine as facade};\n"
                "use crate::{self as root, catalog::{self as catalog, Catalog}};\n"
                "root::content::host::f();\n"
                "use root::{engine::Engine, content as data};\n"
                "use super::super::{content as relative_content};\n"
                "catalog::CardKind::Sha; // crate::content::host::ignored\n",
                encoding="utf-8",
            )
            refs = list(CHECK.upward_references(root))
            self.assertEqual([line for _, line in refs], [1, 1, 1, 4, 4, 5, 3])
            self.assertTrue(all(file == path.relative_to(root) for file, _ in refs))


    def test_raw_identifiers_cannot_hide_upward_paths(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / CHECK.SRC / "vocabulary/shared.rs"
            path.parent.mkdir(parents=True)
            path.write_text(
                "use crate::r#content::host::HostBehavior;\n"
                "use crate::{r#content::{host::HostBehavior as r#behavior}, r#engine as facade};\n"
                "crate::r#engine::f();\n"
                "r#facade::f();\n"
                "r#behavior::f();\n"
                "crate::r#content::host::f();\n"
                "r#content::host::f();\n",
                encoding="utf-8",
            )
            refs = list(CHECK.upward_references(root))
            self.assertEqual([line for _, line in refs], [1, 2, 2, 3, 4, 5, 6, 7])
            self.assertTrue(all(file == path.relative_to(root) for file, _ in refs))


if __name__ == "__main__":
    unittest.main()
