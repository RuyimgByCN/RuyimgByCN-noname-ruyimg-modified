# ADR 0007: Authoritative Sha limit and migration event surfaces

## Decision

- The play command and AI legal-action generator share `sha_use_limit_for`; its base is 1.
- Skills change the limit through `sha_use_limit`. Standard `paoxiao` overrides it, while
  `jsrgbahu` and `zhuishe` add one.
- `CardUsed` preserves declared targets, and the trigger layer exposes leaving-dying and
  max-HP-changed events. These are deterministic domain data, not legacy-JS callbacks.
- Phase transitions emit `PhaseEnded` before `TurnStateChanged`; `phase_play_end` maps the former
  only when leaving the play phase, so phase-use-end skills do not overload end-phase-begin.
- Hand-limit base replacement is separate from ordinary hand-limit bonuses.

## Compatibility

This intentionally changes the standard package logic hash and match behavior: the engine no
longer permits unlimited proactive Sha by default. Conformance vectors were regenerated after
the command path, AI/headless legal-action path, replay, and Paoxiao override tests passed.
Additive event fields use serde defaults so older serialized `CardUsed` events remain readable.
