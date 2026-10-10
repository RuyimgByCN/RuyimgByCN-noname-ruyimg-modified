# ADR 0018: Legacy trigger-filter parity

## Decision

- Keep legacy JavaScript as migration evidence and express migrated trigger predicates in the deterministic engine.
- Add filters for HP-bounded per-turn firing, equal basic/non-basic hand counts, and another player's single-target trick.
- Reuse one `CardKind::is_basic` definition for modifier selectors and trigger filtering.
- Append serialized filter variants so existing package and snapshot enum ordinals remain stable.

## Compatibility

These filters expand rule semantics used by newly migrated packages without changing existing canonical state fields. `EngineSemanticsId` advances from revision 3 to 4, affected conformance manifest hashes are re-pinned, and Native/browser-WASM conformance must agree before release.
