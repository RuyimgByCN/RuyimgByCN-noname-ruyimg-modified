# Cross-target tests

Phase 1 / 12 acceptance: same golden inputs on Native and WASM produce identical:

- State hash per revision
- Domain event sequence
- RNG call count
- ChoiceRequest payload
- Final outcome

## One-shot gate

```bash
bash scripts/cross-target-hash.sh
```

## Native (authoritative CI entry)

```bash
cargo run -p conformance
cargo run -p engine-cli -- hash 4 42
# expected: 371e24331d268a2be8d3ab3f26dc748c828fe1ea1148106fce06e0d4773cc200
```

Pinned vectors live in `crates/conformance`.

## WASM compile gate

```bash
rustup target add wasm32-unknown-unknown
cargo build -p engine-core --target wasm32-unknown-unknown
cargo build -p engine-wasm --target wasm32-unknown-unknown
```

`cargo test --target wasm32-unknown-unknown` does **not** run on host OS loaders
(exit 126). Use:

| Runner | Command |
|---|---|
| wasmtime | `wasmtime target/wasm32-unknown-unknown/release/…` with exported hash harness |
| Browser | `apps/web-client` Worker + same StartGame seed |

## Content package path

```bash
cargo run -p engine-cli -- skill-vm content/standard/jianxiong.skill
# hand draws +1 via RunVm after damage
```

Empty content registry preserves bootstrap-native handlers → golden suite unchanged.
