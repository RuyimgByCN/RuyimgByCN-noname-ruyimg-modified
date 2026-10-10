# ADR 0001: Core Technology Stack

## Status

Accepted

## Context

noname-next reimplements the card game platform rather than patching the legacy JavaScript engine. The architecture plan requires deterministic rules, native + WASM targets, and a single content execution model.

## Decision

| Area | Choice |
|---|---|
| Core language | Rust |
| Browser core | WebAssembly in a Web Worker |
| Native core | Rust library |
| Web shell | Vue 3 + TypeScript |
| Battle UI | DOM + Web Animations API |
| Desktop/mobile shell | Tauri 2 |
| Content | Strongly typed declarative DSL → Rule VM |
| Network | Versioned Protobuf |
| Replay truth | Accepted normalized command log |
| Server concurrency | Tokio; one serial task per room |

## Consequences

- No legacy JS skill execution in the new core.
- No dual script runtimes (Rhai + custom VM).
- Performance claims require reproducible benchmarks.
