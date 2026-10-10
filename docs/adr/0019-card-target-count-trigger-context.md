# ADR 0019: Card target count in trigger context

## Decision

- Expose a card event's declared target count as deterministic DSL value `event_target_count`.
- Reuse the VM's existing event-scalar opcode because damage amount and card target count are bound to disjoint event kinds.
- Add an owner-used-Sha trigger filter and append all serialized enum variants.

## Compatibility

The new binding and filter expand package rule semantics without changing canonical game state. `EngineSemanticsId` advances from revision 4 to 5; Native/browser-WASM conformance vectors must be re-pinned and agree before release.
