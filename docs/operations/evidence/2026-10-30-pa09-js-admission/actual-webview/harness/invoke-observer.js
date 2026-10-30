import { invoke as tauriInvoke } from '/tmp/pa09-js-bound-4306bc00/apps/web-client/node_modules/@tauri-apps/api/core.js';
export * from '/tmp/pa09-js-bound-4306bc00/apps/web-client/node_modules/@tauri-apps/api/core.js';
// Observation only: preserve arguments, original invoke promise, result and rejection.
export function invoke(...args) {
  const promise = tauriInvoke(...args);
  globalThis.__pa09ObserveInvoke?.(args, promise);
  return promise;
}
