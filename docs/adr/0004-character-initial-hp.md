# ADR 0004: Character Initial HP in Content Schema

## Status

Accepted (2026-07-13)

## Context

The original character catalog contains characters whose starting HP differs
from maximum HP. `ol_chouniu` starts at 1/5 HP, while the existing package
schema only represented `max_hp`; treating it as 5/5 would change the skill's
minimum-HP trigger semantics.

## Decision

1. Add `initial_hp` to `PackageCharacterEntry` and bump the content schema from
   2 to 3.
2. An omitted or zero `initial_hp` preserves the previous behavior of starting
   at `max_hp`; a non-zero value must not exceed `max_hp`.
3. Bootstrap uses `initial_hp` when present. The Lord bonus increases both
   starting HP and maximum HP by one.
4. Include `initial_hp` in package and content-registry logic hashes.

## Consequences

- Existing schema-v2 packages must be rebuilt as schema v3 before installation.
- State and semantic Golden hashes are intentionally re-pinned because the
  standard package/registry hash changes even when `initial_hp` is omitted.
- The `ol_chouniu` Golden covers 1/5 startup, Lord bonus, minimum-HP recovery,
  and suppression when another living player has lower HP.
