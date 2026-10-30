const cp = require('node:child_process');
const fs = require('node:fs');
const { syncBuiltinESMExports } = require('node:module');
const original = cp.execFileSync;
cp.execFileSync = function(file, args, options) {
  if (file === 'wasm-pack' && args?.[0] === 'build') {
    const before = options?.env ?? process.env;
    const env = { ...before, CARGO_TARGET_DIR: '/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-f7-stage3.czy5e9gq/target' };
    fs.appendFileSync('/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-10-30-pa09-desktop-native/validation-build-child-argv.jsonl', JSON.stringify({ file, args, before: { CARGO_TARGET_DIR: before.CARGO_TARGET_DIR, CARGO_INCREMENTAL: before.CARGO_INCREMENTAL, RUSTFLAGS: before.RUSTFLAGS ?? null }, after: { CARGO_TARGET_DIR: env.CARGO_TARGET_DIR, CARGO_INCREMENTAL: env.CARGO_INCREMENTAL, RUSTFLAGS: env.RUSTFLAGS ?? null }, changedKeys: ['CARGO_TARGET_DIR'] }) + '\n');
    return original.call(this, file, args, { ...options, env });
  }
  return original.call(this, file, args, options);
};
syncBuiltinESMExports();
