# ADR 0017: Turn card-use history

## Decision

- Record the current turn player's successful proactive card uses as an authoritative turn fact.
- Increment the fact in the shared `UseCard` effect primitive after intent cancellation and card validation, so command, view-as, and skill-generated uses share one definition.
- Reset the fact at normal turn reset and whenever a direct turn-state transition changes the current player.
- Expose comparisons through reusable trigger filters; legacy JavaScript remains migration evidence and never executes in a match.
- Permit empty DSL skills only when the TypeScript AST contains strictly allowlisted audio/`charlotte` metadata and the legacy analyzer reports no rule surface.

## Compatibility

The new fact is serialized in canonical `GameState` once a card is used, so completed-match state hashes intentionally change. `EngineSemanticsId` advances from revision 2 to 3, the affected Native match vectors are re-pinned, and the Native/browser-WASM cross-target gate must agree before release. Existing snapshots remain readable because the fact uses the existing sorted turn-flag representation.
