# ADR 0016: Startup triggers and dynamic mark state

## Decision

- Dispatch a generic `game_started` trigger after opening hands are dealt and before the first turn phase begins.
- Permit `add_mark` deltas to use deterministic expressions such as `event_amount`, and expose mark counts through the read-only `mark_count(player, mark_id)` query.
- Preserve the existing literal `EffectAddMark` IR shape and append a separate expression variant so previously compiled extension packages remain readable.
- Add reusable `owner_is_source_or_target` and `owner_target_fire` trigger filters instead of skill-name branches.
- Add the compound `sha_red` card selector for rules that require both Sha identity and a red effective suit.
- Keep historical JavaScript as migration evidence only; rewritten skills are deterministic DSL modules packaged as npacks.

## Compatibility

New query, trigger, selector, filter, and statement variants are appended. Matches without loaded startup or mark skills retain their prior state and event order. Existing literal `add_mark` package IR remains deserializable, and the default Native/WASM conformance vectors do not change.
