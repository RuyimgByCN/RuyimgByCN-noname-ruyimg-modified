# ADR 0013: Conditional card-target cancellation

## Decision

- `card_target_cancel(selector)` accepts the shared card-selector vocabulary instead of a 桃-only special case.
- Modifier value source `owner_hand_empty` resolves to 1 exactly while the skill owner's hand is empty.
- Target legality queries, proactive commands, AI and headless runs continue through the same authoritative target check; conditional values are resolved from the current `GameState` at query time.
- Standard「空城」uses two declarations for【杀】and【决斗】and is restored to 诸葛亮's packaged skill list.

## Compatibility

The standard package logic hash and 诸葛亮's initial skill set intentionally change. StartGame, complete-match conformance vectors, and historical-character state-lock Goldens are re-pinned from the native reference run; the dedicated Golden also covers legal-query transitions and forged command rejection. Old npacks remain readable because the new selector and value-source enum variants are appended.
