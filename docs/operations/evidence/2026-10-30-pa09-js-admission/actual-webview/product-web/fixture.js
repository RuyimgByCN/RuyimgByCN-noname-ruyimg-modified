import { EngineClient } from './src/engine/client';
import { DesktopEngineTransport } from './src/engine/desktopEngineTransport';
import { invoke } from '@tauri-apps/api/core';

const endpoint = 'http://127.0.0.1:19439';
const trace = [];
const assertions = [];
const clients = [];
const rawCalls = [];
let phase = 'bootstrap';
let serial = 0;
const transportIDs = new WeakMap();
const transportID = (transport) => {
  if (!transportIDs.has(transport)) transportIDs.set(transport, ++serial);
  return transportIDs.get(transport);
};
const clone = (value) => value === undefined ? null : JSON.parse(JSON.stringify(value));
const note = (type, data = {}) => trace.push({ index: trace.length, phase, type, at: performance.now(), ...data });
function check(value, message, data = {}) {
  assertions.push({ message, passed: !!value, ...data });
  if (!value) throw new Error(message + ': ' + JSON.stringify(data));
}
const same = (a, b) => JSON.stringify(a) === JSON.stringify(b);
const nativeCalls = (kind, label = phase) => rawCalls.filter((call) => call.phase === label && call.command === kind);
const pending = (client) => [...client.pending.keys()];
async function until(predicate, label) {
  const limit = performance.now() + 15000;
  while (!predicate()) {
    if (performance.now() >= limit) throw new Error('Timed out waiting for ' + label);
    await new Promise((resolve) => setTimeout(resolve, 10));
  }
}
function client() {
  const value = new EngineClient();
  clients.push(value);
  return value;
}
function observeDelivery(transport) {
  const original = transport.onmessage;
  transport.onmessage = function(event) {
    note('client-delivery', { transport: transportID(transport), response: clone(event.data) });
    return original.call(this, event);
  };
}
const originalPost = DesktopEngineTransport.prototype.postMessage;
DesktopEngineTransport.prototype.postMessage = function(request) {
  const id = transportID(this);
  const before = this.tail;
  try {
    originalPost.call(this, request);
    note('transport-accepted', { transport: id, request: clone(request), closed: this.closed });
  } catch (error) {
    note('transport-rejected', { transport: id, request: clone(request), error: String(error), tailUnchanged: this.tail === before });
    throw error;
  }
};
globalThis.__pa09ObserveInvoke = (args, promise) => {
  if (!['native_engine_open', 'native_engine_request', 'native_engine_close'].includes(args[0])) return;
  const call = { phase, command: args[0], args: clone(args[1]), settled: false, index: rawCalls.length };
  rawCalls.push(call);
  note('invoke-sent', { call: call.index, command: call.command, args: call.args });
  promise.then((response) => {
    call.settled = true;
    call.ok = true;
    call.response = clone(response);
    note('invoke-reply', { call: call.index, command: call.command, response: call.response, sentPhase: call.phase });
  }, (error) => {
    call.settled = true;
    call.ok = false;
    call.error = String(error);
    note('invoke-error', { call: call.index, command: call.command, error: call.error, sentPhase: call.phase });
  });
};
async function proveRetired(resourceId, label) {
  phase = label + '-retired-probe';
  let error;
  try { await invoke('native_engine_request', { resourceId, request: { id: 900001, type: 'status' } }); }
  catch (value) { error = String(value); }
  check(!!error, 'closed resource is refused by real ResourceTable', { resourceId, error });
}
async function burst() {
  phase = 'burst-connect';
  const c = client();
  await c.connect();
  const transport = c.worker;
  observeDelivery(transport);
  check(c.getTransportKind() === 'desktop-native', 'actual EngineClient selected desktop native');
  const resourceId = await transport.resource;
  phase = 'burst128-status';
  const callerResults = [];
  const promises = [];
  const counts = [];
  for (let i = 0; i < 128; i++) {
    promises.push(c.status().then((value) => {
      callerResults.push({ ok: true, value });
      note('caller-resolved', { revision: value.revision });
    }, (error) => {
      callerResults.push({ ok: false, error: String(error) });
      note('caller-rejected', { error: String(error) });
    }));
    counts.push({ attempt: i + 1, pendingIDs: pending(c), pendingCount: c.pending.size });
  }
  check(counts.every((row) => row.pendingCount <= 8), 'pending map bounded after every call', { maximum: Math.max(...counts.map((row) => row.pendingCount)) });
  note('pending-counts', { counts });
  const accepted = trace.filter((row) => row.phase === phase && row.type === 'transport-accepted');
  const rejected = trace.filter((row) => row.phase === phase && row.type === 'transport-rejected');
  check(accepted.length === 8 && rejected.length === 120, '128 actual client status calls admit eight and reject 120 synchronously', { accepted: accepted.length, rejected: rejected.length });
  check(rejected.every((row) => row.tailUnchanged && /not submitted|未提交/.test(row.error) && !/unknown|未知/.test(row.error)), 'all excess calls reject before tail growth with not-submitted text');
  await Promise.resolve();
  check(callerResults.length === 120 && c.pending.size === 8, 'only eight caller promises remain unresolved before returning to native callback event loop', { settled: callerResults.length, pending: pending(c) });
  await Promise.all(promises);
  const calls = nativeCalls('native_engine_request');
  const acceptedIDs = accepted.map((row) => row.request.id);
  check(same(calls.map((call) => call.args.request.id), acceptedIDs), 'real IPC receives only accepted IDs in order', { acceptedIDs, sentIDs: calls.map((call) => call.args.request.id) });
  check(calls.every((call) => call.ok && call.response.ok && call.args.resourceId === resourceId), 'accepted IPC all succeed on the actual resource without unknown results');
  check(callerResults.filter((row) => row.ok).length === 8 && c.pending.size === 0, 'accepted caller promises settle and pending is empty');
  phase = 'continuous24-status';
  const revisions = [];
  for (let i = 0; i < 24; i++) revisions.push((await c.status()).revision);
  check(revisions.length === 24 && c.pending.size === 0 && revisions.every((value) => value === 0), '24 sequential completions release admission continuously');
  c.dispose();
  await until(() => rawCalls.filter((call) => call.command === 'native_engine_close' && call.args.resourceId === resourceId).some((call) => call.settled), 'burst resource close');
  check(rawCalls.filter((call) => call.command === 'native_engine_close' && call.args.resourceId === resourceId).length === 1, 'burst resource closes exactly once');
  await proveRetired(resourceId, 'burst');
  return { resourceId, attempts: 128, accepted: acceptedIDs, notSubmitted: 120, maximumPending: 8, unresolvedAtTurnBoundary: 8, synchronousTurnOnly: true, sequentialSuccesses: 24 };
}
async function openingDispose() {
  phase = 'opening-dispose';
  const c = client();
  const connect = c.connect().then(() => ({ ok: true }), (error) => ({ ok: false, error: String(error) }));
  const transport = c.worker;
  for (let i = 0; i < 8 && !nativeCalls('native_engine_open').length; i++) await Promise.resolve();
  const open = nativeCalls('native_engine_open')[0];
  check(!!open && !open.settled, 'real open IPC outstanding before dispose; no mocked or delayed writer');
  const promises = [connect];
  for (let i = 0; i < 7; i++) promises.push(c.status().then(() => ({ ok: true }), (error) => ({ ok: false, error: String(error) })));
  check(c.pending.size === 8, 'opening plus seven queued requests occupy eight client slots');
  observeDelivery(transport);
  const tid = transportID(transport);
  c.dispose();
  c.dispose();
  check(c.pending.size === 0, 'opening dispose immediately rejects and clears client pending');
  const results = await Promise.all(promises);
  check(results.every((row) => !row.ok && /结果未知/.test(row.error)), 'dispose preserves existing unknown-result rejection for all pending calls');
  const resourceId = await transport.resource;
  await until(() => rawCalls.some((call) => call.command === 'native_engine_close' && call.args.resourceId === resourceId && call.settled), 'opening close after allocation');
  check(nativeCalls('native_engine_request', 'opening-dispose').length === 0, 'opening termination sends no queued request IPC');
  check(rawCalls.filter((call) => call.command === 'native_engine_close' && call.args.resourceId === resourceId).length === 1, 'opening resource closed exactly once after late allocation');
  check(!trace.some((row) => row.type === 'client-delivery' && row.transport === tid), 'opening late allocation delivers no retired client response');
  await proveRetired(resourceId, 'opening');
  return { resourceId, accepted: 8, queuedRequestIPC: 0, immediatePendingAfterDispose: 0, closeCount: 1 };
}
async function activeDispose() {
  phase = 'active-connect';
  const old = client();
  await old.connect();
  const transport = old.worker;
  const resourceId = await transport.resource;
  observeDelivery(transport);
  const tid = transportID(transport);
  phase = 'active-dispose';
  const promises = [old.status().then(() => ({ ok: true }), (error) => ({ ok: false, error: String(error) }))];
  for (let i = 0; i < 8 && !nativeCalls('native_engine_request').length; i++) await Promise.resolve();
  const active = nativeCalls('native_engine_request')[0];
  check(!!active && !active.settled, 'real request IPC outstanding before active dispose without native delay');
  for (let i = 0; i < 7; i++) promises.push(old.status().then(() => ({ ok: true }), (error) => ({ ok: false, error: String(error) })));
  check(old.pending.size === 8, 'active plus seven queued requests occupy eight slots');
  old.dispose();
  old.dispose();
  note('old-disposed', { transport: tid, resourceId, activeCall: active.index, activeSettled: active.settled, pending: pending(old) });
  check(old.pending.size === 0, 'active dispose clears pending synchronously');
  phase = 'fresh-resource';
  const fresh = client();
  await fresh.connect();
  const freshTransport = fresh.worker;
  const freshResourceId = await freshTransport.resource;
  observeDelivery(freshTransport);
  check(freshResourceId !== resourceId, 'fresh client opens independent resource', { resourceId, freshResourceId });
  const result = await fresh.status();
  check(result.revision === 0 && fresh.pending.size === 0, 'fresh client works while old completion retires');
  const results = await Promise.all(promises);
  check(results.every((row) => !row.ok && /结果未知/.test(row.error)), 'all eight old caller promises reject by production dispose');
  await until(() => active.settled && rawCalls.some((call) => call.command === 'native_engine_close' && call.args.resourceId === resourceId && call.settled), 'old request and close callbacks');
  check(nativeCalls('native_engine_request', 'active-dispose').length === 1, 'only active old request reached IPC; seven queued requests were never submitted');
  check(!trace.some((row) => row.type === 'client-delivery' && row.transport === tid), 'old late raw IPC callback is suppressed before client delivery');
  check(rawCalls.filter((call) => call.command === 'native_engine_close' && call.args.resourceId === resourceId).length === 1, 'active old resource closes once');
  check(old.pending.size === 0 && fresh.pending.size === 0, 'old late response cannot add pending or contaminate fresh client');
  await proveRetired(resourceId, 'active');
  phase = 'fresh-close';
  fresh.dispose();
  await until(() => rawCalls.some((call) => call.command === 'native_engine_close' && call.args.resourceId === freshResourceId && call.settled), 'fresh resource close');
  check(rawCalls.filter((call) => call.command === 'native_engine_close' && call.args.resourceId === freshResourceId).length === 1, 'fresh independent resource closes once');
  await proveRetired(freshResourceId, 'fresh');
  return { resourceId, freshResourceId, accepted: 8, oldRequestIPC: 1, oldQueuedNeverSubmitted: 7, oldLateRawReply: clone(active), oldClientDeliveries: 0, closeCount: 1 };
}
let report;
try {
  check(typeof window.__TAURI_INTERNALS__?.invoke === 'function', 'real Tauri bootstrap exists');
  history.replaceState(null, '', '/?engineTransport=native');
  await import('./src/main.ts');
  note('vue-mounted', { text: document.querySelector('.brand')?.textContent, ua: navigator.userAgent, location: location.href });
  const burstResult = await burst();
  const opening = await openingDispose();
  const active = await activeDispose();
  report = { passed: true, jsBounded: true, qualification: 'this real WKWebView fixture; same-event-turn JS admission, not OS scheduling pressure or performance', burst: burstResult, opening, active, assertions, rawCalls, trace, userAgent: navigator.userAgent, tauri: true };
} catch (error) {
  report = { passed: false, jsBounded: false, error: String(error), stack: error?.stack, assertions, rawCalls, trace, userAgent: navigator.userAgent };
} finally {
  for (const c of clients) c.dispose();
}
await fetch(endpoint + '/report', { method: 'POST', body: JSON.stringify(report) });
globalThis.__pa09ActualResult = report;
