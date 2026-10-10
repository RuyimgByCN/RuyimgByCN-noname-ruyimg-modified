# Fuzz testing (architecture §37.7)

libFuzzer harnesses for the deterministic core. Targets every deserialization
boundary and parser listed in §37.7.

## Targets

| Target | Fuzz surface | §37.7 item |
|---|---|---|
| `fuzz_dsl_parser` | `content_compiler::parse_source` | DSL Parser |
| `fuzz_bytecode_module` | `BytecodeModule::from_bytes` | IR / bytecode decode |
| `fuzz_bytecode_verify` | `verify_module` on decoded modules | 字节码验证器 |
| `fuzz_snapshot_decode` | `Snapshot::from_bytes` | Replay / Snapshot |
| `fuzz_replay_decode` | `ReplayLog::from_bytes` | Replay / Snapshot |
| `fuzz_command_json` | `serde_json::from_slice::<Command>` | Command 解码 |
| `fuzz_content_package` | `ContentPackage::from_json_bytes` | 内容包和依赖解析 |

## Running (requires nightly Rust + cargo-fuzz)

```bash
rustup toolchain install nightly
cargo +nightly install cargo-fuzz

# Run a single target for 60 seconds
cargo +nightly fuzz run fuzz_dsl_parser -- -max_total_time=60

# Run with the provided corpus seeds
cargo +nightly fuzz run fuzz_bytecode_module fuzz/corpus/fuzz_bytecode_module/

# Run all targets (script)
for t in fuzz_dsl_parser fuzz_bytecode_module fuzz_bytecode_verify \
         fuzz_snapshot_decode fuzz_replay_decode fuzz_command_json \
         fuzz_content_package; do
  cargo +nightly fuzz run "$t" -- -max_total_time=30
done
```

## Corpus

Seed inputs live in `fuzz/corpus/<target>/`. libFuzzer will extend these with
coverage-guided mutations and write new finds to `fuzz/artifacts/<target>/`.

## CI integration

libFuzzer requires nightly + a sanitizer build, so it runs in a dedicated CI
stage, not `cargo test`. The stable-testable pseudo-fuzz lives in
`crates/engine-core/tests/fuzz_stable.rs` and runs under `cargo test --workspace`
as a smoke gate.

## Contract

Every fuzz target asserts only one thing: **no panic / abort / hang on arbitrary
input**. Structured errors are the correct response to malformed data. If a
target finds a panic, it is a real bug that must be fixed before release.
