# ADR 0002: Determinism and Canonical State Hash

## Status

Accepted

## Context

Replays, multiplayer authority, and Native/WASM parity require bit-stable rule evolution.

## Decision

1. All rule randomness goes through `RuleRng` (xorshift64* with call counting).
2. Entity IDs are stable monotonic integers; freed IDs are never reused.
3. Semantic iteration uses explicit stable order (ID or seat), never raw `HashMap` iteration.
4. Canonical state hash is BLAKE3 over a fixed field order encoding of `GameState` (not Rust memory layout, not Protobuf wire bytes).
5. Caches are never part of hashed state.

## Consequences

- Snapshot restore must rebuild derived indexes and clear caches.
- Cross-target CI compares hash, events, RNG call count, and choices per revision.
