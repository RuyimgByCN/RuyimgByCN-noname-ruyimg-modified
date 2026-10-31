const cp = require('node:child_process');
const fs = require('node:fs');
const { syncBuiltinESMExports } = require('node:module');
const original = cp.execFileSync;
cp.execFileSync = function(file, args, options) {
  if (file === 'wasm-pack' && args?.[0] === 'build') {
    const before = options?.env ?? process.env;
    const env = { ...before, CARGO_TARGET_DIR: '/tmp/nc27-main-functional-458ed333/cargo-target' };
    fs.appendFileSync('/tmp/i02-main-typed-wire-functional-33d21ba5/build-child-argv.jsonl', JSON.stringify({ file, args, beforeTarget: before.CARGO_TARGET_DIR, afterTarget: env.CARGO_TARGET_DIR, changedKeys: ['CARGO_TARGET_DIR'] }) + '\n');
    return original.call(this, file, args, { ...options, env });
  }
  return original.call(this, file, args, options);
};
syncBuiltinESMExports();
