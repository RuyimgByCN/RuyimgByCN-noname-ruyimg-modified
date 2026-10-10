# ADR 0033: Target-card suit and sole-equipment queries

- Status: Accepted
- Date: 2026-07-15
- Engine semantics: 17

## Decision

Add the exact query surfaces required by `xixiu`:

- `OwnerAliveTargetedByOtherMatchingEquipSuit` compares the targeted card's raw suit with the owner's equipped cards' effective suits.
- `OwnerEquipCountExactlyOne` supplies the existing `card_discard_protected(equip)` modifier with a boolean value.

Both enum variants are appended so existing serialized ordinals remain unchanged. No legacy JavaScript executes at runtime.
