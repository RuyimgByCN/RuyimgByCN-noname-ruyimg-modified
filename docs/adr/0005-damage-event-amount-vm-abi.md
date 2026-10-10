# ADR 0005: Damage Event Amount in Rule VM ABI

## Status

Accepted (2026-07-13)

## Context

Historical skills frequently scale effects from the triggering damage. The
Rule VM exposed the damage source and target but not its amount, so `olsishe`
could not faithfully implement equal retaliation.

## Decision

1. Add `event_amount` to damage trigger context and `HOST_EVENT_AMOUNT` to the
   bytecode ISA; bump Rule VM ABI from 2 to 3.
2. Allow `event_amount` as a `deal_damage` amount expression only inside
   `damage` triggers.
3. Until VM continuation context carries the amount, reject modules combining
   `event_amount` with `choose()` instead of silently resuming with a wrong value.
4. Hash the damage amount and logical card kind in deferred trigger context.

## Consequences

- Schema-v3 packages remain structurally valid but must be rebuilt for VM ABI
  v3 before installation.
- Content/state Golden hashes are intentionally re-pinned.
- `ol_sishe` now covers optional equal retaliation and ignores dead sources.
