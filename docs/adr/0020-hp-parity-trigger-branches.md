# ADR 0020: HP-parity trigger branches

## Decision

- Represent legacy HP-parity branches as two mutually exclusive trigger filters instead of adding general modulo arithmetic to the DSL.
- Require a living event-player owner; the even branch additionally requires the owner to be wounded.
- Append serialized filter variants to preserve existing package enum ordinals.

## Compatibility

This filter-only addition is bundled into `EngineSemanticsId` revision 5 with ADR 0019. Native/browser-WASM conformance must agree before release.
