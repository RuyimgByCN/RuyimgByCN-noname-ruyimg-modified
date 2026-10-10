# ADR 0014: Turn damage facts and dynamic combat values

## Decision

- Record whether the current turn owner has damaged another living `qun` character as an authoritative turn fact, reset at the same boundaries as the existing per-turn facts.
- Expose the fact through a reusable trigger filter instead of a skill-specific branch.
- Resolve damage-source attack-range thresholds and the living field's mount-card count through shared query-time modifier primitives.
- Keep historical JavaScript as migration evidence only; rewritten skills are deterministic DSL modules packaged as npacks.

## Compatibility

The new turn fact is part of canonical `GameState`, so state hashes intentionally change even before a skill uses it. `EngineSemanticsId` revision advances from 1 to 2, Native conformance vectors are re-pinned, and cross-target conformance must pass. New serialized enum variants are appended so existing npacks remain readable.
