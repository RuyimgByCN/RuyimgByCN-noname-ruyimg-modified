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
    def make_kernel(self, root):
        src = root / CHECK.SRC
        src.mkdir(parents=True, exist_ok=True)
        for name in ("lib", *CHECK.KERNEL_MODULES):
            (src / f"{name}.rs").write_text("// fixture\n", encoding="utf-8")

    def test_missing_and_empty_mandatory_modules_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with self.assertRaises(ValueError):
                list(CHECK.upward_references(root))
            self.make_kernel(root)
            path = root / CHECK.SRC / "vocabulary.rs"
            path.unlink()
            with self.assertRaises(ValueError):
                list(CHECK.upward_references(root))
            path.write_text("", encoding="utf-8")
            with self.assertRaises(ValueError):
                list(CHECK.upward_references(root))

    def test_external_crates_grouped_raw_and_aliased_paths_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.make_kernel(root)
            path = root / CHECK.SRC / "lib.rs"
            path.write_text(
                "use engine_native_content::{content as data, self as r#upper};\n"
                "use r#engine_native::{engine::Engine as E, self as facade};\n"
                "data::f(); r#upper::f(); E::f(); facade::f();\n"
                "r#engine_native_content::content::f(); engine_native::engine::f();\n",
                encoding="utf-8",
            )
            refs = list(CHECK.upward_references(root))
            self.assertEqual([line for _, line in refs], [1, 1, 2, 2, 3, 3, 3, 3, 4, 4])

    def test_extern_crate_declarations_cannot_hide_upward_paths(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.make_kernel(root)
            path = root / CHECK.SRC / "lib.rs"
            path.write_text(
                "extern crate engine_native_content;\n"
                "extern crate engine_native as upper;\n"
                "extern crate r#engine_native_content as r#data;\n"
                "extern crate r#engine_native as _;\n"
                "extern crate engine_native_content as 上游;\n"
                "upper::Engine::new(42, 4);\n"
                "extern crate blake3 as hashing;\n"
                "// extern crate engine_native_content as ignored;\n",
                encoding="utf-8",
            )
            self.assertEqual([line for _, line in CHECK.upward_references(root)], [1, 2, 3, 4, 5])

    def test_normal_dependency_direction_including_aliases_and_indirection(self):
        def package(name, *dependencies):
            return {"name": name, "dependencies": list(dependencies)}

        def dependency(name, kind=None, **extra):
            return {"name": name, "kind": kind, **extra}

        packages = [
            package("engine-native-kernel"),
            package("engine-native-content", dependency("engine-native-kernel")),
            package("engine-native", dependency("engine-native-content")),
        ]
        self.assertEqual(CHECK.dependency_violations({"packages": packages}), [])
        packages[0]["dependencies"] = [dependency("engine-native", "dev")]
        self.assertEqual(CHECK.dependency_violations({"packages": packages}), [])
        packages[0]["dependencies"] = [dependency("helper", rename="lower")]
        packages.append(package("helper", dependency("engine-native-content", target="cfg(unix)")))
        self.assertTrue(CHECK.dependency_violations({"packages": packages}))
        packages[0]["dependencies"] = []
        packages[1]["dependencies"].append(dependency("engine-native", rename="facade"))
        self.assertTrue(CHECK.dependency_violations({"packages": packages}))
        with self.assertRaises(ValueError):
            CHECK.dependency_violations({"packages": []})

    def test_new_kernel_modules_are_scanned_without_registration(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.make_kernel(root)
            src = root / CHECK.SRC
            (src / "lib.rs").write_text("pub mod future;\n", encoding="utf-8")
            (src / "future.rs").write_text(
                "pub mod nested;\nuse engine_native::Engine;\n", encoding="utf-8"
            )
            (src / "future").mkdir()
            (src / "future/nested.rs").write_text(
                "use engine_native_content::content;\n", encoding="utf-8"
            )
            refs = list(CHECK.upward_references(root))
            self.assertEqual(len(refs), 2)
            self.assertEqual({path for path, _ in refs}, {
                CHECK.SRC / "future.rs", CHECK.SRC / "future/nested.rs",
            })

    def test_current_kernel_has_no_upward_references(self):
        root = Path(__file__).resolve().parents[1]
        refs = list(CHECK.upward_references(root))
        self.assertEqual(len(refs), 0, f"kernel references content/engine {len(refs)} times")

    def test_counts_kernel_layer_code_but_not_comments_or_content(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.make_kernel(root)
            for name, text in {
                "crates/engine-native-kernel/src/kernel.rs": "use crate::content::host::HostMachine;\n",
                "crates/engine-native-kernel/src/vocabulary/host.rs": "use crate::content as content_alias;\n",
                "crates/engine-native-kernel/src/kernel/damage.rs": "x(crate::content::standard::JUE_QING); crate::engine::f();\n",
                "crates/engine-native-kernel/src/snap/restore.rs": "/// see content::standard::restore_content_machine\n",
                "crates/engine-native-content/src/content/host.rs": "use crate::content::standard::X;\n",
                "crates/engine-native/src/engine.rs": "use crate::content::host::Y;\n",
            }.items():
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(text, encoding="utf-8")

            refs = sorted(str(path) for path, _ in CHECK.upward_references(root))

        self.assertEqual(
            refs,
            [
                "crates/engine-native-kernel/src/kernel.rs",
                "crates/engine-native-kernel/src/kernel/damage.rs",
                "crates/engine-native-kernel/src/kernel/damage.rs",
                "crates/engine-native-kernel/src/vocabulary/host.rs",
            ],
        )


    def test_grouped_imports_and_crate_aliases_cannot_hide_upward_paths(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.make_kernel(root)
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
            self.make_kernel(root)
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
