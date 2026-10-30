const cp = require('node:child_process');
const exec = cp.execFileSync;
cp.execFileSync = function(file, args, options) {
  if (options?.env?.CARGO_TARGET_DIR) options = { ...options, env: { ...options.env, CARGO_TARGET_DIR: '/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-f7-stage3.czy5e9gq/target' } };
  return exec.call(this, file, args, options);
};
require('node:module').syncBuiltinESMExports();
