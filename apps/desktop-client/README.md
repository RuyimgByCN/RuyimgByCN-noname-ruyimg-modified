# desktop-client

Tauri 2 desktop shell plus the Phase 10 capability-gated **host bridge**.

## Role

```text
Web Battle UI  (apps/web-client)
      │  capability-gated IPC
      ▼
DesktopHost    (this crate)
      │
      ├─ content-runtime  (atomic install / rollback / revoke / lock)
      ├─ diagnostics      (redacted packs)
      └─ engine-core      (loaded by engine path, not by content packages)
```

Content packages **never** receive:

- raw filesystem paths outside the store
- network
- process spawn
- host secrets

## Tauri 2 integration

`tauri.conf.json` embeds the same `apps/web-client` build used in browsers. The
shell registers only the typed host commands in `src/main.rs`; its capability
file grants no filesystem, shell, or HTTP plugin. The Web UI freezes content on
match mount and unfreezes it on unmount.

Mobile (`mobile-shell`) reuses the same `DesktopHost` contract with a platform data directory.

The macOS aarch64 bundle was exercised through a full four-player seed-42 match on
2026-07-13 (revision 167, loyalist/lord victory). Cross-platform distribution remains M7 work.

## Dev

```bash
cargo test -p desktop-client
cargo check -p desktop-client --features tauri-shell --bin noname-next-desktop
pnpm --dir apps/web-client desktop
pnpm --dir apps/web-client desktop:build
cargo run -p content-cli -- --store /tmp/npack install pkg.json --allow-unsigned
```
