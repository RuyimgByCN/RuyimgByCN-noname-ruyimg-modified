import importlib.util
from pathlib import Path
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location(
    "native_layering", Path(__file__).with_name("check-native-layering.py")
)
CHECK = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECK)


class NativeLayeringTest(unittest.TestCase):
    def test_counts_kernel_layer_code_but_not_comments_or_content(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name, text in {
                "kernel.rs": "use crate::content::host::HostMachine;\n",
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
            ],
        )


if __name__ == "__main__":
    unittest.main()
