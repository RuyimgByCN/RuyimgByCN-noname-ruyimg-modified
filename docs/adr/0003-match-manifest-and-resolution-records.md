# ADR 0003: MatchManifest and Hash-Chained Resolution Records

## Status

Accepted (2026-07-11)

## Context

The architecture plan v2.0 review (§0) identified two foundational gaps in the
existing implementation that block freezing the architecture baseline:

1. **Conflated build identity.** The codebase carried a single flat
   `ENGINE_BUILD_ID` string used for both rule semantics and binary identity.
   Native and WASM binaries can never share an artifact hash, so the determinism
   contract must rest on a *semantic* identity (`EngineSemanticsId`) that two
   cross-target-equivalent builds share, plus a separate concrete
   `ArtifactBuildId` for deployment and rollback.

2. **No per-match immutable contract.** Nothing pinned the exact engine
   semantics, Rule IR / VM ABI, resolved ruleset, content lock, RNG spec or
   budgets for the lifetime of a match. A live room could drift if any of those
   changed mid-match.

3. **No tamper-evident log.** `AcceptedCommand` recorded command + post-revision
   + post-hash but carried no chain link, so a single tampered or reordered
   record would not be detectable without a full recompute.

The review also mandated (§22.1) that only *successfully committed* resolution
segments enter the canonical log — rejected commands go to a separate audit log
— and that each record classify its `ResolutionOutcome`.

## Decision

### 1. MatchManifest (plan §5)

Introduce `MatchManifest` as the immutable per-match contract, living in
`engine-api::manifest`. It captures:

- `EngineSemanticsId` — rule semantics (shared by Native/WASM once cross-target
  tests pass).
- `RuleIrVersion` / `VmAbiVersion` — independent version axes for the IR and VM
  ABI.
- `ResolvedRulesetRef` + `ContentLockHash` — exact content pinning.
- `RuleRngSpec` — algorithm + version + seed.
- `BudgetProfile` — effect / trigger / VM fuel caps (deterministic budget faults).
- `MatchConfig` + `InitialPlayer` order.
- `VersionBundle` snapshot.

`MatchManifest::manifest_hash()` produces a canonical `MatchManifestHash`
independent of any `GameState` or Protobuf encoding. The hash is pinned into the
`ReplayHeader` at `StartGame` and never changes for the rest of the match.

**Three identity axes remain strictly separate:**

| Axis | Type | Purpose |
|---|---|---|
| Rule semantics | `EngineSemanticsId` | Determinism contract; shared cross-target |
| Binary artifact | `ArtifactBuildId` | Deployment / rollback / exact reproduction |
| Match contract | `MatchManifestHash` | Replay + WAL + snapshot reference |

### 2. Hash-chained ResolutionRecord (plan §22.1–22.2)

`ResolutionRecord` records each committed segment with pre/post revision, pre/post
state hash, RNG checkpoints (`RuleRngCheckpoint`), a `ResolutionOutcome` tag, an
optional terminal `MatchOutcome`, and two chain fields: `previous_record_hash`
and `record_hash`. `record_hash` is recomputed from every field except itself;
each record's `previous_record_hash` must equal its predecessor's `record_hash`.

`ReplayLog::verify_chain()` walks the chain and returns the first broken link.
`ReplayLog::push_record()` derives the chain in lockstep with the compact
`AcceptedCommand` form (kept for golden-vector compatibility).

### 3. Engine integration

`Engine::submit` pins the manifest on the first `StartGame` and rebuilds the
`ReplayLog` header with the manifest hash. `Engine::manifest()` / `manifest_hash()`
expose the frozen contract. `restore_snapshot_with_manifest()` supports the
reconnect path where the manifest must survive a snapshot restore.

### 4. Independence from GameState hashing

`MatchManifest` is **not** part of `GameState` and does not enter the canonical
state hash. This guarantees all existing golden vectors remain valid — verified
by `phase14_manifest::manifest_independent_of_state_hash_keeps_golden_vectors`.

## Consequences

- Every official content release must pass cross-target conformance before
  bumping a shared `EngineSemanticsId`.
- Replays now carry a manifest hash; a replay whose manifest hash differs from
  the running engine's is rejected (replay viewer / reconnect).
- A tampered log is detectable in O(n) via `verify_chain` without a full recompute.
- The determinism contract (plan §6) now reads:

  ```
  相同 EngineSemanticsId
  + 相同 Rule IR / VM ABI
  + 相同 MatchManifestHash
  + 相同初始规范状态
  + 相同已提交 Command 序列
  = 相同 Revision、State Hash、DomainEvent 和最终结果
  ```

- Bumping `MANIFEST_SCHEMA_VERSION`, `RuleIrVersion`, or `VmAbiVersion` requires
  updating golden vectors and recording an ADR note.
