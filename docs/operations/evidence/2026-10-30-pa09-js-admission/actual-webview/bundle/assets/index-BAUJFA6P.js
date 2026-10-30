const __vite__mapDeps=(i,m=__vite__mapDeps,d=(m.f||(m.f=["assets/main-CqWflYpH.js","assets/client-BZROKSmU.js","assets/main-jC10sqNn.css"])))=>i.map(i=>d[i]);
import { E as j, i as A, D as O } from "./client-BZROKSmU.js";
(async () => {
  var _a, _b;
  (function() {
    const s = document.createElement("link").relList;
    if (s && s.supports && s.supports("modulepreload")) return;
    for (const a of document.querySelectorAll('link[rel="modulepreload"]')) c(a);
    new MutationObserver((a) => {
      for (const i of a) if (i.type === "childList") for (const l of i.addedNodes) l.tagName === "LINK" && l.rel === "modulepreload" && c(l);
    }).observe(document, {
      childList: true,
      subtree: true
    });
    function n(a) {
      const i = {};
      return a.integrity && (i.integrity = a.integrity), a.referrerPolicy && (i.referrerPolicy = a.referrerPolicy), a.crossOrigin === "use-credentials" ? i.credentials = "include" : a.crossOrigin === "anonymous" ? i.credentials = "omit" : i.credentials = "same-origin", i;
    }
    function c(a) {
      if (a.ep) return;
      a.ep = true;
      const i = n(a);
      fetch(a.href, i);
    }
  })();
  const D = "modulepreload", z = function(e) {
    return "/" + e;
  }, x = {}, L = function(s, n, c) {
    let a = Promise.resolve();
    if (n && n.length > 0) {
      let l = function(d) {
        return Promise.all(d.map((f) => Promise.resolve(f).then((t) => ({
          status: "fulfilled",
          value: t
        }), (t) => ({
          status: "rejected",
          reason: t
        }))));
      };
      document.getElementsByTagName("link");
      const u = document.querySelector("meta[property=csp-nonce]"), r = (u == null ? void 0 : u.nonce) || (u == null ? void 0 : u.getAttribute("nonce"));
      a = l(n.map((d) => {
        if (d = z(d), d in x) return;
        x[d] = true;
        const f = d.endsWith(".css"), t = f ? '[rel="stylesheet"]' : "";
        if (document.querySelector(`link[href="${d}"]${t}`)) return;
        const p = document.createElement("link");
        if (p.rel = f ? "stylesheet" : D, f || (p.as = "script"), p.crossOrigin = "", p.href = d, r && p.setAttribute("nonce", r), document.head.appendChild(p), f) return new Promise((N, R) => {
          p.addEventListener("load", N), p.addEventListener("error", () => R(new Error(`Unable to preload CSS for ${d}`)));
        });
      }));
    }
    function i(l) {
      const u = new Event("vite:preloadError", {
        cancelable: true
      });
      if (u.payload = l, window.dispatchEvent(u), !u.defaultPrevented) throw l;
    }
    return a.then((l) => {
      for (const u of l || []) u.status === "rejected" && i(u.reason);
      return s().catch(i);
    });
  }, J = "http://127.0.0.1:19439", v = [], C = [], T = [], m = [];
  let g = "bootstrap", M = 0;
  const I = /* @__PURE__ */ new WeakMap(), q = (e) => (I.has(e) || I.set(e, ++M), I.get(e)), w = (e) => e === void 0 ? null : JSON.parse(JSON.stringify(e)), h = (e, s = {}) => v.push({
    index: v.length,
    phase: g,
    type: e,
    at: performance.now(),
    ...s
  });
  function o(e, s, n = {}) {
    if (C.push({
      message: s,
      passed: !!e,
      ...n
    }), !e) throw new Error(s + ": " + JSON.stringify(n));
  }
  const U = (e, s) => JSON.stringify(e) === JSON.stringify(s), y = (e, s = g) => m.filter((n) => n.phase === s && n.command === e), E = (e) => [
    ...e.pending.keys()
  ];
  async function _(e, s) {
    const n = performance.now() + 15e3;
    for (; !e(); ) {
      if (performance.now() >= n) throw new Error("Timed out waiting for " + s);
      await new Promise((c) => setTimeout(c, 10));
    }
  }
  function b() {
    const e = new j();
    return T.push(e), e;
  }
  function k(e) {
    const s = e.onmessage;
    e.onmessage = function(n) {
      return h("client-delivery", {
        transport: q(e),
        response: w(n.data)
      }), s.call(this, n);
    };
  }
  const B = O.prototype.postMessage;
  O.prototype.postMessage = function(e) {
    const s = q(this), n = this.tail;
    try {
      B.call(this, e), h("transport-accepted", {
        transport: s,
        request: w(e),
        closed: this.closed
      });
    } catch (c) {
      throw h("transport-rejected", {
        transport: s,
        request: w(e),
        error: String(c),
        tailUnchanged: this.tail === n
      }), c;
    }
  };
  globalThis.__pa09ObserveInvoke = (e, s) => {
    if (![
      "native_engine_open",
      "native_engine_request",
      "native_engine_close"
    ].includes(e[0])) return;
    const n = {
      phase: g,
      command: e[0],
      args: w(e[1]),
      settled: false,
      index: m.length
    };
    m.push(n), h("invoke-sent", {
      call: n.index,
      command: n.command,
      args: n.args
    }), s.then((c) => {
      n.settled = true, n.ok = true, n.response = w(c), h("invoke-reply", {
        call: n.index,
        command: n.command,
        response: n.response,
        sentPhase: n.phase
      });
    }, (c) => {
      n.settled = true, n.ok = false, n.error = String(c), h("invoke-error", {
        call: n.index,
        command: n.command,
        error: n.error,
        sentPhase: n.phase
      });
    });
  };
  async function P(e, s) {
    g = s + "-retired-probe";
    let n;
    try {
      await A("native_engine_request", {
        resourceId: e,
        request: {
          id: 900001,
          type: "status"
        }
      });
    } catch (c) {
      n = String(c);
    }
    o(!!n, "closed resource is refused by real ResourceTable", {
      resourceId: e,
      error: n
    });
  }
  async function W() {
    g = "burst-connect";
    const e = b();
    await e.connect();
    const s = e.worker;
    k(s), o(e.getTransportKind() === "desktop-native", "actual EngineClient selected desktop native");
    const n = await s.resource;
    g = "burst128-status";
    const c = [], a = [], i = [];
    for (let t = 0; t < 128; t++) a.push(e.status().then((p) => {
      c.push({
        ok: true,
        value: p
      }), h("caller-resolved", {
        revision: p.revision
      });
    }, (p) => {
      c.push({
        ok: false,
        error: String(p)
      }), h("caller-rejected", {
        error: String(p)
      });
    })), i.push({
      attempt: t + 1,
      pendingIDs: E(e),
      pendingCount: e.pending.size
    });
    o(i.every((t) => t.pendingCount <= 8), "pending map bounded after every call", {
      maximum: Math.max(...i.map((t) => t.pendingCount))
    }), h("pending-counts", {
      counts: i
    });
    const l = v.filter((t) => t.phase === g && t.type === "transport-accepted"), u = v.filter((t) => t.phase === g && t.type === "transport-rejected");
    o(l.length === 8 && u.length === 120, "128 actual client status calls admit eight and reject 120 synchronously", {
      accepted: l.length,
      rejected: u.length
    }), o(u.every((t) => t.tailUnchanged && /not submitted|未提交/.test(t.error) && !/unknown|未知/.test(t.error)), "all excess calls reject before tail growth with not-submitted text"), await Promise.resolve(), o(c.length === 120 && e.pending.size === 8, "only eight caller promises remain unresolved before returning to native callback event loop", {
      settled: c.length,
      pending: E(e)
    }), await Promise.all(a);
    const r = y("native_engine_request"), d = l.map((t) => t.request.id);
    o(U(r.map((t) => t.args.request.id), d), "real IPC receives only accepted IDs in order", {
      acceptedIDs: d,
      sentIDs: r.map((t) => t.args.request.id)
    }), o(r.every((t) => t.ok && t.response.ok && t.args.resourceId === n), "accepted IPC all succeed on the actual resource without unknown results"), o(c.filter((t) => t.ok).length === 8 && e.pending.size === 0, "accepted caller promises settle and pending is empty"), g = "continuous24-status";
    const f = [];
    for (let t = 0; t < 24; t++) f.push((await e.status()).revision);
    return o(f.length === 24 && e.pending.size === 0 && f.every((t) => t === 0), "24 sequential completions release admission continuously"), e.dispose(), await _(() => m.filter((t) => t.command === "native_engine_close" && t.args.resourceId === n).some((t) => t.settled), "burst resource close"), o(m.filter((t) => t.command === "native_engine_close" && t.args.resourceId === n).length === 1, "burst resource closes exactly once"), await P(n, "burst"), {
      resourceId: n,
      attempts: 128,
      accepted: d,
      notSubmitted: 120,
      maximumPending: 8,
      unresolvedAtTurnBoundary: 8,
      synchronousTurnOnly: true,
      sequentialSuccesses: 24
    };
  }
  async function K() {
    g = "opening-dispose";
    const e = b(), s = e.connect().then(() => ({
      ok: true
    }), (r) => ({
      ok: false,
      error: String(r)
    })), n = e.worker;
    for (let r = 0; r < 8 && !y("native_engine_open").length; r++) await Promise.resolve();
    const c = y("native_engine_open")[0];
    o(!!c && !c.settled, "real open IPC outstanding before dispose; no mocked or delayed writer");
    const a = [
      s
    ];
    for (let r = 0; r < 7; r++) a.push(e.status().then(() => ({
      ok: true
    }), (d) => ({
      ok: false,
      error: String(d)
    })));
    o(e.pending.size === 8, "opening plus seven queued requests occupy eight client slots"), k(n);
    const i = q(n);
    e.dispose(), e.dispose(), o(e.pending.size === 0, "opening dispose immediately rejects and clears client pending");
    const l = await Promise.all(a);
    o(l.every((r) => !r.ok && /结果未知/.test(r.error)), "dispose preserves existing unknown-result rejection for all pending calls");
    const u = await n.resource;
    return await _(() => m.some((r) => r.command === "native_engine_close" && r.args.resourceId === u && r.settled), "opening close after allocation"), o(y("native_engine_request", "opening-dispose").length === 0, "opening termination sends no queued request IPC"), o(m.filter((r) => r.command === "native_engine_close" && r.args.resourceId === u).length === 1, "opening resource closed exactly once after late allocation"), o(!v.some((r) => r.type === "client-delivery" && r.transport === i), "opening late allocation delivers no retired client response"), await P(u, "opening"), {
      resourceId: u,
      accepted: 8,
      queuedRequestIPC: 0,
      immediatePendingAfterDispose: 0,
      closeCount: 1
    };
  }
  async function $() {
    g = "active-connect";
    const e = b();
    await e.connect();
    const s = e.worker, n = await s.resource;
    k(s);
    const c = q(s);
    g = "active-dispose";
    const a = [
      e.status().then(() => ({
        ok: true
      }), (t) => ({
        ok: false,
        error: String(t)
      }))
    ];
    for (let t = 0; t < 8 && !y("native_engine_request").length; t++) await Promise.resolve();
    const i = y("native_engine_request")[0];
    o(!!i && !i.settled, "real request IPC outstanding before active dispose without native delay");
    for (let t = 0; t < 7; t++) a.push(e.status().then(() => ({
      ok: true
    }), (p) => ({
      ok: false,
      error: String(p)
    })));
    o(e.pending.size === 8, "active plus seven queued requests occupy eight slots"), e.dispose(), e.dispose(), h("old-disposed", {
      transport: c,
      resourceId: n,
      activeCall: i.index,
      activeSettled: i.settled,
      pending: E(e)
    }), o(e.pending.size === 0, "active dispose clears pending synchronously"), g = "fresh-resource";
    const l = b();
    await l.connect();
    const u = l.worker, r = await u.resource;
    k(u), o(r !== n, "fresh client opens independent resource", {
      resourceId: n,
      freshResourceId: r
    });
    const d = await l.status();
    o(d.revision === 0 && l.pending.size === 0, "fresh client works while old completion retires");
    const f = await Promise.all(a);
    return o(f.every((t) => !t.ok && /结果未知/.test(t.error)), "all eight old caller promises reject by production dispose"), await _(() => i.settled && m.some((t) => t.command === "native_engine_close" && t.args.resourceId === n && t.settled), "old request and close callbacks"), o(y("native_engine_request", "active-dispose").length === 1, "only active old request reached IPC; seven queued requests were never submitted"), o(!v.some((t) => t.type === "client-delivery" && t.transport === c), "old late raw IPC callback is suppressed before client delivery"), o(m.filter((t) => t.command === "native_engine_close" && t.args.resourceId === n).length === 1, "active old resource closes once"), o(e.pending.size === 0 && l.pending.size === 0, "old late response cannot add pending or contaminate fresh client"), await P(n, "active"), g = "fresh-close", l.dispose(), await _(() => m.some((t) => t.command === "native_engine_close" && t.args.resourceId === r && t.settled), "fresh resource close"), o(m.filter((t) => t.command === "native_engine_close" && t.args.resourceId === r).length === 1, "fresh independent resource closes once"), await P(r, "fresh"), {
      resourceId: n,
      freshResourceId: r,
      accepted: 8,
      oldRequestIPC: 1,
      oldQueuedNeverSubmitted: 7,
      oldLateRawReply: w(i),
      oldClientDeliveries: 0,
      closeCount: 1
    };
  }
  let S;
  try {
    o(typeof ((_a = window.__TAURI_INTERNALS__) == null ? void 0 : _a.invoke) == "function", "real Tauri bootstrap exists"), history.replaceState(null, "", "/?engineTransport=native"), await L(() => import("./main-CqWflYpH.js"), __vite__mapDeps([0,1,2])), h("vue-mounted", {
      text: (_b = document.querySelector(".brand")) == null ? void 0 : _b.textContent,
      ua: navigator.userAgent,
      location: location.href
    });
    const e = await W(), s = await K(), n = await $();
    S = {
      passed: true,
      jsBounded: true,
      qualification: "this real WKWebView fixture; same-event-turn JS admission, not OS scheduling pressure or performance",
      burst: e,
      opening: s,
      active: n,
      assertions: C,
      rawCalls: m,
      trace: v,
      userAgent: navigator.userAgent,
      tauri: true
    };
  } catch (e) {
    S = {
      passed: false,
      jsBounded: false,
      error: String(e),
      stack: e == null ? void 0 : e.stack,
      assertions: C,
      rawCalls: m,
      trace: v,
      userAgent: navigator.userAgent
    };
  } finally {
    for (const e of T) e.dispose();
  }
  await fetch(J + "/report", {
    method: "POST",
    body: JSON.stringify(S)
  });
  globalThis.__pa09ActualResult = S;
})();
