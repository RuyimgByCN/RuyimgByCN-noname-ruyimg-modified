# ADR 0015: Phase-skip and initial-hand modifiers

## Decision

- Resolve a player's initial hand count through the ordinary state-aware modifier pipeline before the opening deal.
- Resolve judgment-phase skipping through a passive modifier at the phase boundary; emit the normal `PhaseSkipped` event and leave delayed cards in the judgment zone.
- Keep both capabilities generic and data-driven so historical skills use DSL definitions instead of skill-name branches in Rust.
- Keep historical JavaScript as migration evidence only; rewritten skills are deterministic DSL modules packaged as npacks.

## Compatibility

Both modifier enum variants are appended. Matches without a loaded modifier retain the previous four-card opening hand and ordinary judgment-phase resolution, so the canonical default-state and conformance vectors do not change. Existing npacks remain readable.
