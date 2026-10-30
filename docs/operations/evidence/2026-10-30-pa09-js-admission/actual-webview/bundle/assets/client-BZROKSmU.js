async function ei(e, t = {}, n) {
  return window.__TAURI_INTERNALS__.invoke(e, t, n);
}
function ti(e, t = "asset") {
  return window.__TAURI_INTERNALS__.convertFileSrc(e, t);
}
function ft(...e) {
  var _a;
  const t = ei(...e);
  return (_a = globalThis.__pa09ObserveInvoke) == null ? void 0 : _a.call(globalThis, e, t), t;
}
function Qe() {
  return typeof window < "u" && "__TAURI_INTERNALS__" in window;
}
function Nl() {
  return Qe() ? ft("platform_info") : Promise.resolve(null);
}
function Hl(e) {
  return Qe() ? ft("set_match_frozen", { frozen: e }) : Promise.resolve();
}
function ni() {
  return Qe() ? ft("content_bundle") : Promise.resolve(null);
}
function Ll(e, t) {
  return Qe() ? ft("content_catalog_page", { offset: e, limit: t }) : Promise.resolve(null);
}
async function Ul(e) {
  if (!Qe()) return null;
  const t = await ft("resolve_content_asset", { namespace: e.namespace, packageId: e.package_id, assetId: e.asset_id });
  return ti(t);
}
const tt = (e) => new Promise((t) => setTimeout(t, e)), ln = "\u81EA\u52A8\u4E00\u6B65", si = "\u8FDE\u7EED\u81EA\u52A8", cn = /终局/;
let Vs = [], On = 0;
function Dt(e) {
  return Array.from(document.querySelectorAll("button")).find((t) => (t.innerText || "").trim() === e) ?? null;
}
function gs() {
  return new Promise((e) => {
    const t = () => {
      const r = Dt(ln);
      return r ? cn.test(document.body.innerText) ? "finished" : r.disabled ? null : "ready" : "gone";
    }, n = t();
    if (n) {
      e(n);
      return;
    }
    const s = new MutationObserver(() => {
      const r = t();
      r && (s.disconnect(), e(r));
    });
    s.observe(document.body, { attributes: true, subtree: true, childList: true, attributeFilter: ["disabled"] }), setTimeout(() => {
      s.disconnect(), e("timeout");
    }, 15e3);
  });
}
function Mn() {
  return window.__nonamePerf ?? null;
}
function ri() {
  const e = Mn(), t = document.body.innerText.match(/终局[^\n]*/) ?? document.body.innerText.match(/rev \d+[^\n]*/);
  return { frames: (e == null ? void 0 : e.frames()) ?? [], longTasks: (e == null ? void 0 : e.longTasks()) ?? [], eventTimings: (e == null ? void 0 : e.eventTimings()) ?? [], interactions: (e == null ? void 0 : e.interactions()) ?? [], status: (t == null ? void 0 : t[0]) ?? null, ua: navigator.userAgent, dpr: window.devicePixelRatio, viewport: [window.innerWidth, window.innerHeight], href: location.href, engine_transport: globalThis.__engineTransport ?? null, memory: null, memory_series: [] };
}
async function ii() {
  const e = performance;
  try {
    if (e.memory) {
      const t = e.memory;
      return { used: t.usedJSHeapSize, total: t.totalJSHeapSize, limit: t.jsHeapSizeLimit, api: "performance.memory" };
    }
    if (e.measureUserAgentSpecificMemory && crossOriginIsolated) {
      const t = await e.measureUserAgentSpecificMemory();
      return { used: t.bytes, total: t.bytes, limit: 0, api: "measureUserAgentSpecificMemory" };
    }
  } catch {
  }
  return null;
}
async function oi(e, t) {
  try {
    await fetch(e, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(t) });
  } catch {
  }
}
async function li(e, t) {
  for (let n = 0; n < e; n++) {
    const s = Dt(ln);
    if (!s || cn.test(document.body.innerText)) return "finished";
    if (s.disabled) {
      if (await gs() === "finished") return "finished";
      continue;
    }
    if (s.click(), await gs() === "finished") return "finished";
    await tt(t);
  }
  return "max_steps";
}
async function ci() {
  var _a;
  const e = de("loop") === "1", t = Number(de("duration") ?? 0), n = Date.now() + (t > 0 ? t : 6e5), s = () => cn.test(document.body.innerText), r = [], i = performance.memory, o = performance.measureUserAgentSpecificMemory, l = async () => {
    if (i) {
      r.push({ t_ms: Date.now() - On, bytes: i.usedJSHeapSize });
      return;
    }
    if (!(!o || !crossOriginIsolated)) try {
      const a = await o();
      r.push({ t_ms: Date.now() - On, bytes: a.bytes });
    } catch {
    }
  };
  for (; ; ) {
    const a = Dt(si);
    a && a.click();
    let d = 0;
    for (; Date.now() < n && !s(); ) await tt(1e3), Date.now() - d > 3e4 && (d = Date.now(), await l());
    if (!e || Date.now() >= n) return Vs = r, s() ? "finished" : "auto_timeout";
    (_a = Array.from(document.querySelectorAll("button")).find((p) => (p.innerText || "").includes("\u91CD\u65B0\u5F00\u5C40"))) == null ? void 0 : _a.click(), await tt(1500);
  }
}
function ai(e, t) {
  return new Promise((n) => {
    let s = 0;
    const r = (l) => {
      window.removeEventListener("click", i, true), n(l);
    }, i = (l) => {
      const a = l.target, d = Dt(ln);
      d && (a === d || d.contains(a)) && (s += 1, s >= e && setTimeout(() => r("manual_steps"), 800));
    };
    window.addEventListener("click", i, true);
    const o = setInterval(() => {
      cn.test(document.body.innerText) && (clearInterval(o), r("finished"));
    }, 500);
    setTimeout(() => {
      clearInterval(o), r(s > 0 ? "manual_timeout" : "manual_no_input");
    }, t);
  });
}
function de(e) {
  const t = location.hash.split("?")[1], n = t ? new URLSearchParams(t) : null;
  return new URLSearchParams(location.search).get(e) ?? (n == null ? void 0 : n.get(e)) ?? null;
}
function fi() {
  if (typeof window > "u") return;
  const e = window;
  if (e.__nonameAutoDriveStarted) return;
  const t = de("autodrive");
  if (!t) return;
  e.__nonameAutoDriveStarted = true;
  const n = de("report"), s = Number(de("steps") ?? 1500), r = Number(de("gap") ?? 50), i = Number(de("timeout") ?? 12e4);
  (async () => {
    var _a;
    for (let p = 0; p < 150; p++) {
      const S = Dt(ln);
      if (S && Mn() && !S.disabled) break;
      await tt(200);
    }
    (_a = Mn()) == null ? void 0 : _a.reset(), await tt(300);
    const l = Date.now();
    On = l;
    let a = "unknown";
    t === "click" ? a = await li(s, r) : t === "auto" ? a = await ci() : t === "manual" && (a = await ai(s, i)), await tt(600);
    const d = ri();
    d.memory = await ii(), d.memory_series = Vs, n && await oi(n, { ...d, end_state: a, wall_ms: Date.now() - l });
    const u = window;
    u.__nonameAutoDriveDone = a;
  })();
}
const _s = { confirm: "Confirm", select_cards: "SelectCards", select_players: "SelectTargets", respond_card: "Custom", pick_tag: "Custom" };
function ui(e) {
  if (e === void 0) return [null, null];
  if (!Array.isArray(e) || e.length !== 2 || !e.every((t) => t === null || typeof t == "object" && !Array.isArray(t) && Number.isInteger(t.def) && t.def >= 0 && t.def <= 4294967295 && typeof t.name == "string" && typeof t.faction == "string" && typeof t.revealed == "boolean")) throw new Error("native: generals metadata invalid");
  return e;
}
function di(e) {
  if (e.prompt != null && typeof e.prompt != "string") throw new Error("native: prompt metadata invalid");
  if (e.native_kind != null && (typeof e.native_kind != "string" || !Object.hasOwn(_s, e.native_kind))) throw new Error("native: native_kind metadata invalid");
  const t = e.native_kind;
  return { ...e.prompt == null ? {} : { prompt: e.prompt }, ...t == null ? {} : { native_kind: t }, kind: t ? _s[t] : e.kind ?? "Custom" };
}
const ms = { confirm: "\u786E\u8BA4", select_cards: "\u9009\u62E9\u5361\u724C", select_players: "\u9009\u62E9\u89D2\u8272", respond_card: "\u54CD\u5E94\u51FA\u724C", pick_tag: "\u9009\u62E9\u9009\u9879", Confirm: "\u786E\u8BA4", SelectCards: "\u9009\u62E9\u5361\u724C", SelectTargets: "\u9009\u62E9\u89D2\u8272", Custom: "\u9009\u62E9\u9009\u9879", OrderTriggers: "\u6392\u5217\u6280\u80FD", DiscardToLimit: "\u5F03\u724C\u81F3\u624B\u724C\u4E0A\u9650" }, ys = { trigger_confirm: "\u662F\u5426\u53D1\u52A8\u6280\u80FD\uFF1F", respond_shan: "\u8BF7\u6253\u51FA\u95EA", respond_sha: "\u8BF7\u6253\u51FA\u6740", aoe_response: "\u8BF7\u54CD\u5E94\u7FA4\u4F53\u9526\u56CA", dying_save: "\u8BF7\u9009\u62E9\u6551\u63F4\u724C", discard_to_limit: "\u8BF7\u5F03\u724C\u81F3\u624B\u724C\u4E0A\u9650", tuxi_target: "\u7A81\u88AD\uFF1A\u9009\u62E9\u83B7\u5F97\u624B\u724C\u7684\u89D2\u8272", guanxing_arrange: "\u89C2\u661F\uFF1A\u6392\u5217\u724C\u5806\u9876\u7684\u5361\u724C", xunxun_arrange: "\u6042\u6042\uFF1A\u9009\u62E9\u83B7\u5F97\u7684\u5361\u724C", jijie_give_target: "\u673A\u6377\uFF1A\u9009\u62E9\u83B7\u5F97\u6B64\u724C\u7684\u89D2\u8272", gaiming_pick: "\u6539\u547D\uFF1A\u9009\u62E9\u5224\u5B9A\u724C", haotianta_pick: "\u660A\u5929\u5854\uFF1A\u9009\u62E9\u5224\u5B9A\u724C", liaoyuan_discard: "\u71CE\u539F\uFF1A\u9009\u62E9\u5F03\u7F6E\u7684\u5361\u724C", fanjian_target: "\u53CD\u95F4\uFF1A\u9009\u62E9\u76EE\u6807\u89D2\u8272", fanjian_suit: "\u53CD\u95F4\uFF1A\u9009\u62E9\u82B1\u8272", fanjian_pick: "\u53CD\u95F4\uFF1A\u9009\u62E9\u83B7\u5F97\u7684\u624B\u724C" };
function bs(e) {
  const t = e.native_kind ?? e.kind ?? "Custom";
  return Object.hasOwn(ms, t) ? ms[t] : "\u9009\u62E9\u9009\u9879";
}
function $l(e) {
  return e.prompt && Object.hasOwn(ys, e.prompt) ? ys[e.prompt] : bs(e);
}
function hi(e, t) {
  for (const n of e.seats ?? []) ui(n.generals);
  if (t && (di(t), t.deadline_unix_ms != null && (!Number.isSafeInteger(t.deadline_unix_ms) || t.deadline_unix_ms <= 0))) throw new Error("native: choice deadline invalid");
}
function pi(e) {
  if (e === null || typeof e != "object" || Array.isArray(e)) return e;
  const t = e.SingleWin;
  if (t === null || typeof t != "object" || Array.isArray(t)) return e;
  const n = t.player;
  return typeof n != "number" || !Number.isInteger(n) || n <= 0 ? e : { ...e, SingleWin: { ...t, player: n - 1 } };
}
const gi = "noname-next-desktop-engine", _i = 8;
function mi() {
  const e = typeof location > "u" ? null : de("engineTransport");
  if (e === "native" || e === "wasm") return e;
  try {
    return localStorage.getItem(gi) === "native" ? "native" : "wasm";
  } catch {
    return "wasm";
  }
}
function vs(e) {
  return hi(e.view, e.choice), { ...e, view: { ...e.view, outcome: pi(e.view.outcome) } };
}
function yi(e, t) {
  var _a;
  if (!e.ok || !e.data || typeof e.data != "object") return e;
  if (t.type === "chooseGeneral") {
    const n = e.data;
    if ((_a = n.envelope) == null ? void 0 : _a.view) return { ...e, data: { ...n, envelope: vs(n.envelope) } };
  } else {
    const n = e.data;
    if (n.view) return { ...e, data: vs(n) };
  }
  return e;
}
class bi {
  constructor(t = ft) {
    this.call = t, this.onmessage = null, this.onerror = null, this.closed = false, this.outstanding = 0, this.tail = Promise.resolve(), this.resource = null;
  }
  postMessage(t) {
    if (this.closed) throw new Error("Desktop engine transport closed");
    if (this.outstanding >= _i) throw new Error("Desktop engine busy; request was not submitted\uFF08\u672A\u63D0\u4EA4\uFF09");
    this.outstanding++, this.tail = this.tail.then(async () => {
      var _a;
      try {
        if (this.closed) return;
        let n;
        try {
          this.resource ?? (this.resource = this.call("native_engine_open"));
          const s = await this.resource;
          if (this.closed) return;
          if (n = await this.call("native_engine_request", { resourceId: s, request: t }), n = yi(n, t), n.id !== t.id || typeof n.ok != "boolean") throw new Error("invalid IPC response");
        } catch (s) {
          n = { id: t.id, ok: false, error: `Desktop IPC: ${String(s)}; \u64CD\u4F5C\u7ED3\u679C\u53EF\u80FD\u672A\u77E5\uFF0C\u8BF7\u5148\u786E\u8BA4\u5F53\u524D\u5C40\u9762` };
        }
        this.closed || ((_a = this.onmessage) == null ? void 0 : _a.call(this, { data: n }));
      } finally {
        this.closed || this.outstanding--;
      }
    }).catch((n) => console.error("Desktop engine response listener failed", n));
  }
  terminate() {
    var _a;
    this.closed || (this.closed = true, this.outstanding = 0, (_a = this.resource) == null ? void 0 : _a.then((t) => this.call("native_engine_close", { resourceId: t })).catch((t) => console.error("Desktop engine resource close failed", t)));
  }
  addEventListener(t, n) {
  }
}
/**
* @vue/shared v3.5.39
* (c) 2018-present Yuxi (Evan) You and Vue contributors
* @license MIT
**/
function Bn(e) {
  const t = /* @__PURE__ */ Object.create(null);
  for (const n of e.split(",")) t[n] = 1;
  return (n) => n in t;
}
const U = {}, nt = [], Ae = () => {
}, Js = () => false, qn = (e) => e.charCodeAt(0) === 111 && e.charCodeAt(1) === 110 && (e.charCodeAt(2) > 122 || e.charCodeAt(2) < 97), Vn = (e) => e.startsWith("onUpdate:"), ee = Object.assign, Jn = (e, t) => {
  const n = e.indexOf(t);
  n > -1 && e.splice(n, 1);
}, vi = Object.prototype.hasOwnProperty, j = (e, t) => vi.call(e, t), P = Array.isArray, st = (e) => Ft(e) === "[object Map]", zs = (e) => Ft(e) === "[object Set]", ws = (e) => Ft(e) === "[object Date]", O = (e) => typeof e == "function", re = (e) => typeof e == "string", je = (e) => typeof e == "symbol", N = (e) => e !== null && typeof e == "object", Gs = (e) => (N(e) || O(e)) && O(e.then) && O(e.catch), Ys = Object.prototype.toString, Ft = (e) => Ys.call(e), wi = (e) => Ft(e).slice(8, -1), Xs = (e) => Ft(e) === "[object Object]", zn = (e) => re(e) && e !== "NaN" && e[0] !== "-" && "" + parseInt(e, 10) === e, xt = Bn(",key,ref,ref_for,ref_key,onVnodeBeforeMount,onVnodeMounted,onVnodeBeforeUpdate,onVnodeUpdated,onVnodeBeforeUnmount,onVnodeUnmounted"), an = (e) => {
  const t = /* @__PURE__ */ Object.create(null);
  return ((n) => t[n] || (t[n] = e(n)));
}, xi = /-\w/g, Ke = an((e) => e.replace(xi, (t) => t.slice(1).toUpperCase())), Si = /\B([A-Z])/g, jt = an((e) => e.replace(Si, "-$1").toLowerCase()), Qs = an((e) => e.charAt(0).toUpperCase() + e.slice(1)), vn = an((e) => e ? `on${Qs(e)}` : ""), Ee = (e, t) => !Object.is(e, t), wn = (e, ...t) => {
  for (let n = 0; n < e.length; n++) e[n](...t);
}, Zs = (e, t, n, s = false) => {
  Object.defineProperty(e, t, { configurable: true, enumerable: false, writable: s, value: n });
}, Ti = (e) => {
  const t = parseFloat(e);
  return isNaN(t) ? e : t;
};
let xs;
const fn = () => xs || (xs = typeof globalThis < "u" ? globalThis : typeof self < "u" ? self : typeof window < "u" ? window : typeof global < "u" ? global : {});
function Gn(e) {
  if (P(e)) {
    const t = {};
    for (let n = 0; n < e.length; n++) {
      const s = e[n], r = re(s) ? ki(s) : Gn(s);
      if (r) for (const i in r) t[i] = r[i];
    }
    return t;
  } else if (re(e) || N(e)) return e;
}
const Ei = /;(?![^(]*\))/g, Ai = /:([^]+)/, Ii = /\/\*[^]*?\*\//g;
function ki(e) {
  const t = {};
  return e.replace(Ii, "").split(Ei).forEach((n) => {
    if (n) {
      const s = n.split(Ai);
      s.length > 1 && (t[s[0].trim()] = s[1].trim());
    }
  }), t;
}
function Yn(e) {
  let t = "";
  if (re(e)) t = e;
  else if (P(e)) for (let n = 0; n < e.length; n++) {
    const s = Yn(e[n]);
    s && (t += s + " ");
  }
  else if (N(e)) for (const n in e) e[n] && (t += n + " ");
  return t.trim();
}
const Ci = "itemscope,allowfullscreen,formnovalidate,ismap,nomodule,novalidate,readonly", Kl = Bn(Ci);
function Wl(e) {
  return !!e || e === "";
}
function Pi(e, t) {
  if (e.length !== t.length) return false;
  let n = true;
  for (let s = 0; n && s < e.length; s++) n = un(e[s], t[s]);
  return n;
}
function un(e, t) {
  if (e === t) return true;
  let n = ws(e), s = ws(t);
  if (n || s) return n && s ? e.getTime() === t.getTime() : false;
  if (n = je(e), s = je(t), n || s) return e === t;
  if (n = P(e), s = P(t), n || s) return n && s ? Pi(e, t) : false;
  if (n = N(e), s = N(t), n || s) {
    if (!n || !s) return false;
    const r = Object.keys(e).length, i = Object.keys(t).length;
    if (r !== i) return false;
    for (const o in e) {
      const l = e.hasOwnProperty(o), a = t.hasOwnProperty(o);
      if (l && !a || !l && a || !un(e[o], t[o])) return false;
    }
  }
  return String(e) === String(t);
}
function Bl(e, t) {
  return e.findIndex((n) => un(n, t));
}
const er = (e) => !!(e && e.__v_isRef === true), Oi = (e) => re(e) ? e : e == null ? "" : P(e) || N(e) && (e.toString === Ys || !O(e.toString)) ? er(e) ? Oi(e.value) : JSON.stringify(e, tr, 2) : String(e), tr = (e, t) => er(t) ? tr(e, t.value) : st(t) ? { [`Map(${t.size})`]: [...t.entries()].reduce((n, [s, r], i) => (n[xn(s, i) + " =>"] = r, n), {}) } : zs(t) ? { [`Set(${t.size})`]: [...t.values()].map((n) => xn(n)) } : je(t) ? xn(t) : N(t) && !P(t) && !Xs(t) ? String(t) : t, xn = (e, t = "") => {
  var n;
  return je(e) ? `Symbol(${(n = e.description) != null ? n : t})` : e;
};
/**
* @vue/reactivity v3.5.39
* (c) 2018-present Yuxi (Evan) You and Vue contributors
* @license MIT
**/
let Y;
class Mi {
  constructor(t = false) {
    this.detached = t, this._active = true, this._on = 0, this.effects = [], this.cleanups = [], this._isPaused = false, this._warnOnRun = true, this.__v_skip = true, !t && Y && (Y.active ? (this.parent = Y, this.index = (Y.scopes || (Y.scopes = [])).push(this) - 1) : (this._active = false, this._warnOnRun = false));
  }
  get active() {
    return this._active;
  }
  pause() {
    if (this._active) {
      this._isPaused = true;
      let t, n;
      if (this.scopes) for (t = 0, n = this.scopes.length; t < n; t++) this.scopes[t].pause();
      for (t = 0, n = this.effects.length; t < n; t++) this.effects[t].pause();
    }
  }
  resume() {
    if (this._active && this._isPaused) {
      this._isPaused = false;
      let t, n;
      if (this.scopes) for (t = 0, n = this.scopes.length; t < n; t++) this.scopes[t].resume();
      for (t = 0, n = this.effects.length; t < n; t++) this.effects[t].resume();
    }
  }
  run(t) {
    if (this._active) {
      const n = Y;
      try {
        return Y = this, t();
      } finally {
        Y = n;
      }
    }
  }
  on() {
    ++this._on === 1 && (this.prevScope = Y, Y = this);
  }
  off() {
    if (this._on > 0 && --this._on === 0) {
      if (Y === this) Y = this.prevScope;
      else {
        let t = Y;
        for (; t; ) {
          if (t.prevScope === this) {
            t.prevScope = this.prevScope;
            break;
          }
          t = t.prevScope;
        }
      }
      this.prevScope = void 0;
    }
  }
  stop(t) {
    if (this._active) {
      this._active = false;
      let n, s;
      for (n = 0, s = this.effects.length; n < s; n++) this.effects[n].stop();
      for (this.effects.length = 0, n = 0, s = this.cleanups.length; n < s; n++) this.cleanups[n]();
      if (this.cleanups.length = 0, this.scopes) {
        for (n = 0, s = this.scopes.length; n < s; n++) this.scopes[n].stop(true);
        this.scopes.length = 0;
      }
      if (!this.detached && this.parent && !t) {
        const r = this.parent.scopes.pop();
        r && r !== this && (this.parent.scopes[this.index] = r, r.index = this.index);
      }
      this.parent = void 0;
    }
  }
}
function Ri() {
  return Y;
}
let K;
const Sn = /* @__PURE__ */ new WeakSet();
class nr {
  constructor(t) {
    this.fn = t, this.deps = void 0, this.depsTail = void 0, this.flags = 5, this.next = void 0, this.cleanup = void 0, this.scheduler = void 0, Y && (Y.active ? Y.effects.push(this) : this.flags &= -2);
  }
  pause() {
    this.flags |= 64;
  }
  resume() {
    this.flags & 64 && (this.flags &= -65, Sn.has(this) && (Sn.delete(this), this.trigger()));
  }
  notify() {
    this.flags & 2 && !(this.flags & 32) || this.flags & 8 || rr(this);
  }
  run() {
    if (!(this.flags & 1)) return this.fn();
    this.flags |= 2, Ss(this), ir(this);
    const t = K, n = he;
    K = this, he = true;
    try {
      return this.fn();
    } finally {
      or(this), K = t, he = n, this.flags &= -3;
    }
  }
  stop() {
    if (this.flags & 1) {
      for (let t = this.deps; t; t = t.nextDep) Zn(t);
      this.deps = this.depsTail = void 0, Ss(this), this.onStop && this.onStop(), this.flags &= -2;
    }
  }
  trigger() {
    this.flags & 64 ? Sn.add(this) : this.scheduler ? this.scheduler() : this.runIfDirty();
  }
  runIfDirty() {
    Rn(this) && this.run();
  }
  get dirty() {
    return Rn(this);
  }
}
let sr = 0, St, Tt;
function rr(e, t = false) {
  if (e.flags |= 8, t) {
    e.next = Tt, Tt = e;
    return;
  }
  e.next = St, St = e;
}
function Xn() {
  sr++;
}
function Qn() {
  if (--sr > 0) return;
  if (Tt) {
    let t = Tt;
    for (Tt = void 0; t; ) {
      const n = t.next;
      t.next = void 0, t.flags &= -9, t = n;
    }
  }
  let e;
  for (; St; ) {
    let t = St;
    for (St = void 0; t; ) {
      const n = t.next;
      if (t.next = void 0, t.flags &= -9, t.flags & 1) try {
        t.trigger();
      } catch (s) {
        e || (e = s);
      }
      t = n;
    }
  }
  if (e) throw e;
}
function ir(e) {
  for (let t = e.deps; t; t = t.nextDep) t.version = -1, t.prevActiveLink = t.dep.activeLink, t.dep.activeLink = t;
}
function or(e) {
  let t, n = e.depsTail, s = n;
  for (; s; ) {
    const r = s.prevDep;
    s.version === -1 ? (s === n && (n = r), Zn(s), Di(s)) : t = s, s.dep.activeLink = s.prevActiveLink, s.prevActiveLink = void 0, s = r;
  }
  e.deps = t, e.depsTail = n;
}
function Rn(e) {
  for (let t = e.deps; t; t = t.nextDep) if (t.dep.version !== t.version || t.dep.computed && (lr(t.dep.computed) || t.dep.version !== t.version)) return true;
  return !!e._dirty;
}
function lr(e) {
  if (e.flags & 4 && !(e.flags & 16) || (e.flags &= -17, e.globalVersion === Ct) || (e.globalVersion = Ct, !e.isSSR && e.flags & 128 && (!e.deps && !e._dirty || !Rn(e)))) return;
  e.flags |= 2;
  const t = e.dep, n = K, s = he;
  K = e, he = true;
  try {
    ir(e);
    const r = e.fn(e._value);
    (t.version === 0 || Ee(r, e._value)) && (e.flags |= 128, e._value = r, t.version++);
  } catch (r) {
    throw t.version++, r;
  } finally {
    K = n, he = s, or(e), e.flags &= -3;
  }
}
function Zn(e, t = false) {
  const { dep: n, prevSub: s, nextSub: r } = e;
  if (s && (s.nextSub = r, e.prevSub = void 0), r && (r.prevSub = s, e.nextSub = void 0), n.subs === e && (n.subs = s, !s && n.computed)) {
    n.computed.flags &= -5;
    for (let i = n.computed.deps; i; i = i.nextDep) Zn(i, true);
  }
  !t && !--n.sc && n.map && n.map.delete(n.key);
}
function Di(e) {
  const { prevDep: t, nextDep: n } = e;
  t && (t.nextDep = n, e.prevDep = void 0), n && (n.prevDep = t, e.nextDep = void 0);
}
let he = true;
const cr = [];
function ke() {
  cr.push(he), he = false;
}
function Ce() {
  const e = cr.pop();
  he = e === void 0 ? true : e;
}
function Ss(e) {
  const { cleanup: t } = e;
  if (e.cleanup = void 0, t) {
    const n = K;
    K = void 0;
    try {
      t();
    } finally {
      K = n;
    }
  }
}
let Ct = 0;
class Fi {
  constructor(t, n) {
    this.sub = t, this.dep = n, this.version = n.version, this.nextDep = this.prevDep = this.nextSub = this.prevSub = this.prevActiveLink = void 0;
  }
}
class es {
  constructor(t) {
    this.computed = t, this.version = 0, this.activeLink = void 0, this.subs = void 0, this.map = void 0, this.key = void 0, this.sc = 0, this.__v_skip = true;
  }
  track(t) {
    if (!K || !he || K === this.computed) return;
    let n = this.activeLink;
    if (n === void 0 || n.sub !== K) n = this.activeLink = new Fi(K, this), K.deps ? (n.prevDep = K.depsTail, K.depsTail.nextDep = n, K.depsTail = n) : K.deps = K.depsTail = n, ar(n);
    else if (n.version === -1 && (n.version = this.version, n.nextDep)) {
      const s = n.nextDep;
      s.prevDep = n.prevDep, n.prevDep && (n.prevDep.nextDep = s), n.prevDep = K.depsTail, n.nextDep = void 0, K.depsTail.nextDep = n, K.depsTail = n, K.deps === n && (K.deps = s);
    }
    return n;
  }
  trigger(t) {
    this.version++, Ct++, this.notify(t);
  }
  notify(t) {
    Xn();
    try {
      for (let n = this.subs; n; n = n.prevSub) n.sub.notify() && n.sub.dep.notify();
    } finally {
      Qn();
    }
  }
}
function ar(e) {
  if (e.dep.sc++, e.sub.flags & 4) {
    const t = e.dep.computed;
    if (t && !e.dep.subs) {
      t.flags |= 20;
      for (let s = t.deps; s; s = s.nextDep) ar(s);
    }
    const n = e.dep.subs;
    n !== e && (e.prevSub = n, n && (n.nextSub = e)), e.dep.subs = e;
  }
}
const Dn = /* @__PURE__ */ new WeakMap(), Ye = Symbol(""), Fn = Symbol(""), Pt = Symbol("");
function X(e, t, n) {
  if (he && K) {
    let s = Dn.get(e);
    s || Dn.set(e, s = /* @__PURE__ */ new Map());
    let r = s.get(n);
    r || (s.set(n, r = new es()), r.map = s, r.key = n), r.track();
  }
}
function De(e, t, n, s, r, i) {
  const o = Dn.get(e);
  if (!o) {
    Ct++;
    return;
  }
  const l = (a) => {
    a && a.trigger();
  };
  if (Xn(), t === "clear") o.forEach(l);
  else {
    const a = P(e), d = a && zn(n);
    if (a && n === "length") {
      const u = Number(s);
      o.forEach((p, S) => {
        (S === "length" || S === Pt || !je(S) && S >= u) && l(p);
      });
    } else switch ((n !== void 0 || o.has(void 0)) && l(o.get(n)), d && l(o.get(Pt)), t) {
      case "add":
        a ? d && l(o.get("length")) : (l(o.get(Ye)), st(e) && l(o.get(Fn)));
        break;
      case "delete":
        a || (l(o.get(Ye)), st(e) && l(o.get(Fn)));
        break;
      case "set":
        st(e) && l(o.get(Ye));
        break;
    }
  }
  Qn();
}
function Ze(e) {
  const t = F(e);
  return t === e ? t : (X(t, "iterate", Pt), ue(e) ? t : t.map(pe));
}
function dn(e) {
  return X(e = F(e), "iterate", Pt), e;
}
function Se(e, t) {
  return Ne(e) ? ot(Xe(e) ? pe(t) : t) : pe(t);
}
const ji = { __proto__: null, [Symbol.iterator]() {
  return Tn(this, Symbol.iterator, (e) => Se(this, e));
}, concat(...e) {
  return Ze(this).concat(...e.map((t) => P(t) ? Ze(t) : t));
}, entries() {
  return Tn(this, "entries", (e) => (e[1] = Se(this, e[1]), e));
}, every(e, t) {
  return Pe(this, "every", e, t, void 0, arguments);
}, filter(e, t) {
  return Pe(this, "filter", e, t, (n) => n.map((s) => Se(this, s)), arguments);
}, find(e, t) {
  return Pe(this, "find", e, t, (n) => Se(this, n), arguments);
}, findIndex(e, t) {
  return Pe(this, "findIndex", e, t, void 0, arguments);
}, findLast(e, t) {
  return Pe(this, "findLast", e, t, (n) => Se(this, n), arguments);
}, findLastIndex(e, t) {
  return Pe(this, "findLastIndex", e, t, void 0, arguments);
}, forEach(e, t) {
  return Pe(this, "forEach", e, t, void 0, arguments);
}, includes(...e) {
  return En(this, "includes", e);
}, indexOf(...e) {
  return En(this, "indexOf", e);
}, join(e) {
  return Ze(this).join(e);
}, lastIndexOf(...e) {
  return En(this, "lastIndexOf", e);
}, map(e, t) {
  return Pe(this, "map", e, t, void 0, arguments);
}, pop() {
  return gt(this, "pop");
}, push(...e) {
  return gt(this, "push", e);
}, reduce(e, ...t) {
  return Ts(this, "reduce", e, t);
}, reduceRight(e, ...t) {
  return Ts(this, "reduceRight", e, t);
}, shift() {
  return gt(this, "shift");
}, some(e, t) {
  return Pe(this, "some", e, t, void 0, arguments);
}, splice(...e) {
  return gt(this, "splice", e);
}, toReversed() {
  return Ze(this).toReversed();
}, toSorted(e) {
  return Ze(this).toSorted(e);
}, toSpliced(...e) {
  return Ze(this).toSpliced(...e);
}, unshift(...e) {
  return gt(this, "unshift", e);
}, values() {
  return Tn(this, "values", (e) => Se(this, e));
} };
function Tn(e, t, n) {
  const s = dn(e), r = s[t]();
  return s !== e && !ue(e) && (r._next = r.next, r.next = () => {
    const i = r._next();
    return i.done || (i.value = n(i.value)), i;
  }), r;
}
const Ni = Array.prototype;
function Pe(e, t, n, s, r, i) {
  const o = dn(e), l = o !== e && !ue(e), a = o[t];
  if (a !== Ni[t]) {
    const p = a.apply(e, i);
    return l ? pe(p) : p;
  }
  let d = n;
  o !== e && (l ? d = function(p, S) {
    return n.call(this, Se(e, p), S, e);
  } : n.length > 2 && (d = function(p, S) {
    return n.call(this, p, S, e);
  }));
  const u = a.call(o, d, s);
  return l && r ? r(u) : u;
}
function Ts(e, t, n, s) {
  const r = dn(e), i = r !== e && !ue(e);
  let o = n, l = false;
  r !== e && (i ? (l = s.length === 0, o = function(d, u, p) {
    return l && (l = false, d = Se(e, d)), n.call(this, d, Se(e, u), p, e);
  }) : n.length > 3 && (o = function(d, u, p) {
    return n.call(this, d, u, p, e);
  }));
  const a = r[t](o, ...s);
  return l ? Se(e, a) : a;
}
function En(e, t, n) {
  const s = F(e);
  X(s, "iterate", Pt);
  const r = s[t](...n);
  return (r === -1 || r === false) && rs(n[0]) ? (n[0] = F(n[0]), s[t](...n)) : r;
}
function gt(e, t, n = []) {
  ke(), Xn();
  const s = F(e)[t].apply(e, n);
  return Qn(), Ce(), s;
}
const Hi = Bn("__proto__,__v_isRef,__isVue"), fr = new Set(Object.getOwnPropertyNames(Symbol).filter((e) => e !== "arguments" && e !== "caller").map((e) => Symbol[e]).filter(je));
function Li(e) {
  je(e) || (e = String(e));
  const t = F(this);
  return X(t, "has", e), t.hasOwnProperty(e);
}
class ur {
  constructor(t = false, n = false) {
    this._isReadonly = t, this._isShallow = n;
  }
  get(t, n, s) {
    if (n === "__v_skip") return t.__v_skip;
    const r = this._isReadonly, i = this._isShallow;
    if (n === "__v_isReactive") return !r;
    if (n === "__v_isReadonly") return r;
    if (n === "__v_isShallow") return i;
    if (n === "__v_raw") return s === (r ? i ? Gi : gr : i ? pr : hr).get(t) || Object.getPrototypeOf(t) === Object.getPrototypeOf(s) ? t : void 0;
    const o = P(t);
    if (!r) {
      let a;
      if (o && (a = ji[n])) return a;
      if (n === "hasOwnProperty") return Li;
    }
    const l = Reflect.get(t, n, Z(t) ? t : s);
    if ((je(n) ? fr.has(n) : Hi(n)) || (r || X(t, "get", n), i)) return l;
    if (Z(l)) {
      const a = o && zn(n) ? l : l.value;
      return r && N(a) ? Nn(a) : a;
    }
    return N(l) ? r ? Nn(l) : ns(l) : l;
  }
}
class dr extends ur {
  constructor(t = false) {
    super(false, t);
  }
  set(t, n, s, r) {
    let i = t[n];
    const o = P(t) && zn(n);
    if (!this._isShallow) {
      const d = Ne(i);
      if (!ue(s) && !Ne(s) && (i = F(i), s = F(s)), !o && Z(i) && !Z(s)) return d || (i.value = s), true;
    }
    const l = o ? Number(n) < t.length : j(t, n), a = Reflect.set(t, n, s, Z(t) ? t : r);
    return t === F(r) && a && (l ? Ee(s, i) && De(t, "set", n, s) : De(t, "add", n, s)), a;
  }
  deleteProperty(t, n) {
    const s = j(t, n);
    t[n];
    const r = Reflect.deleteProperty(t, n);
    return r && s && De(t, "delete", n, void 0), r;
  }
  has(t, n) {
    const s = Reflect.has(t, n);
    return (!je(n) || !fr.has(n)) && X(t, "has", n), s;
  }
  ownKeys(t) {
    return X(t, "iterate", P(t) ? "length" : Ye), Reflect.ownKeys(t);
  }
}
class Ui extends ur {
  constructor(t = false) {
    super(true, t);
  }
  set(t, n) {
    return true;
  }
  deleteProperty(t, n) {
    return true;
  }
}
const $i = new dr(), Ki = new Ui(), Wi = new dr(true);
const jn = (e) => e, Bt = (e) => Reflect.getPrototypeOf(e);
function Bi(e, t, n) {
  return function(...s) {
    const r = this.__v_raw, i = F(r), o = st(i), l = e === "entries" || e === Symbol.iterator && o, a = e === "keys" && o, d = r[e](...s), u = n ? jn : t ? ot : pe;
    return !t && X(i, "iterate", a ? Fn : Ye), ee(Object.create(d), { next() {
      const { value: p, done: S } = d.next();
      return S ? { value: p, done: S } : { value: l ? [u(p[0]), u(p[1])] : u(p), done: S };
    } });
  };
}
function qt(e) {
  return function(...t) {
    return e === "delete" ? false : e === "clear" ? void 0 : this;
  };
}
function qi(e, t) {
  const n = { get(r) {
    const i = this.__v_raw, o = F(i), l = F(r);
    e || (Ee(r, l) && X(o, "get", r), X(o, "get", l));
    const { has: a } = Bt(o), d = t ? jn : e ? ot : pe;
    if (a.call(o, r)) return d(i.get(r));
    if (a.call(o, l)) return d(i.get(l));
    i !== o && i.get(r);
  }, get size() {
    const r = this.__v_raw;
    return !e && X(F(r), "iterate", Ye), r.size;
  }, has(r) {
    const i = this.__v_raw, o = F(i), l = F(r);
    return e || (Ee(r, l) && X(o, "has", r), X(o, "has", l)), r === l ? i.has(r) : i.has(r) || i.has(l);
  }, forEach(r, i) {
    const o = this, l = o.__v_raw, a = F(l), d = t ? jn : e ? ot : pe;
    return !e && X(a, "iterate", Ye), l.forEach((u, p) => r.call(i, d(u), d(p), o));
  } };
  return ee(n, e ? { add: qt("add"), set: qt("set"), delete: qt("delete"), clear: qt("clear") } : { add(r) {
    const i = F(this), o = Bt(i), l = F(r), a = !t && !ue(r) && !Ne(r) ? l : r;
    return o.has.call(i, a) || Ee(r, a) && o.has.call(i, r) || Ee(l, a) && o.has.call(i, l) || (i.add(a), De(i, "add", a, a)), this;
  }, set(r, i) {
    !t && !ue(i) && !Ne(i) && (i = F(i));
    const o = F(this), { has: l, get: a } = Bt(o);
    let d = l.call(o, r);
    d || (r = F(r), d = l.call(o, r));
    const u = a.call(o, r);
    return o.set(r, i), d ? Ee(i, u) && De(o, "set", r, i) : De(o, "add", r, i), this;
  }, delete(r) {
    const i = F(this), { has: o, get: l } = Bt(i);
    let a = o.call(i, r);
    a || (r = F(r), a = o.call(i, r)), l && l.call(i, r);
    const d = i.delete(r);
    return a && De(i, "delete", r, void 0), d;
  }, clear() {
    const r = F(this), i = r.size !== 0, o = r.clear();
    return i && De(r, "clear", void 0, void 0), o;
  } }), ["keys", "values", "entries", Symbol.iterator].forEach((r) => {
    n[r] = Bi(r, e, t);
  }), n;
}
function ts(e, t) {
  const n = qi(e, t);
  return (s, r, i) => r === "__v_isReactive" ? !e : r === "__v_isReadonly" ? e : r === "__v_raw" ? s : Reflect.get(j(n, r) && r in s ? n : s, r, i);
}
const Vi = { get: ts(false, false) }, Ji = { get: ts(false, true) }, zi = { get: ts(true, false) };
const hr = /* @__PURE__ */ new WeakMap(), pr = /* @__PURE__ */ new WeakMap(), gr = /* @__PURE__ */ new WeakMap(), Gi = /* @__PURE__ */ new WeakMap();
function Yi(e) {
  switch (e) {
    case "Object":
    case "Array":
      return 1;
    case "Map":
    case "Set":
    case "WeakMap":
    case "WeakSet":
      return 2;
    default:
      return 0;
  }
}
function ns(e) {
  return Ne(e) ? e : ss(e, false, $i, Vi, hr);
}
function Xi(e) {
  return ss(e, false, Wi, Ji, pr);
}
function Nn(e) {
  return ss(e, true, Ki, zi, gr);
}
function ss(e, t, n, s, r) {
  if (!N(e) || e.__v_raw && !(t && e.__v_isReactive) || e.__v_skip || !Object.isExtensible(e)) return e;
  const i = r.get(e);
  if (i) return i;
  const o = Yi(wi(e));
  if (o === 0) return e;
  const l = new Proxy(e, o === 2 ? s : n);
  return r.set(e, l), l;
}
function Xe(e) {
  return Ne(e) ? Xe(e.__v_raw) : !!(e && e.__v_isReactive);
}
function Ne(e) {
  return !!(e && e.__v_isReadonly);
}
function ue(e) {
  return !!(e && e.__v_isShallow);
}
function rs(e) {
  return e ? !!e.__v_raw : false;
}
function F(e) {
  const t = e && e.__v_raw;
  return t ? F(t) : e;
}
function Qi(e) {
  return !j(e, "__v_skip") && Object.isExtensible(e) && Zs(e, "__v_skip", true), e;
}
const pe = (e) => N(e) ? ns(e) : e, ot = (e) => N(e) ? Nn(e) : e;
function Z(e) {
  return e ? e.__v_isRef === true : false;
}
function ql(e) {
  return _r(e, false);
}
function Vl(e) {
  return _r(e, true);
}
function _r(e, t) {
  return Z(e) ? e : new Zi(e, t);
}
class Zi {
  constructor(t, n) {
    this.dep = new es(), this.__v_isRef = true, this.__v_isShallow = false, this._rawValue = n ? t : F(t), this._value = n ? t : pe(t), this.__v_isShallow = n;
  }
  get value() {
    return this.dep.track(), this._value;
  }
  set value(t) {
    const n = this._rawValue, s = this.__v_isShallow || ue(t) || Ne(t);
    t = s ? t : F(t), Ee(t, n) && (this._rawValue = t, this._value = s ? t : pe(t), this.dep.trigger());
  }
}
function eo(e) {
  return Z(e) ? e.value : e;
}
const to = { get: (e, t, n) => t === "__v_raw" ? e : eo(Reflect.get(e, t, n)), set: (e, t, n, s) => {
  const r = e[t];
  return Z(r) && !Z(n) ? (r.value = n, true) : Reflect.set(e, t, n, s);
} };
function mr(e) {
  return Xe(e) ? e : new Proxy(e, to);
}
class no {
  constructor(t, n, s) {
    this.fn = t, this.setter = n, this._value = void 0, this.dep = new es(this), this.__v_isRef = true, this.deps = void 0, this.depsTail = void 0, this.flags = 16, this.globalVersion = Ct - 1, this.next = void 0, this.effect = this, this.__v_isReadonly = !n, this.isSSR = s;
  }
  notify() {
    if (this.flags |= 16, !(this.flags & 8) && K !== this) return rr(this, true), true;
  }
  get value() {
    const t = this.dep.track();
    return lr(this), t && (t.version = this.dep.version), this._value;
  }
  set value(t) {
    this.setter && this.setter(t);
  }
}
function so(e, t, n = false) {
  let s, r;
  return O(e) ? s = e : (s = e.get, r = e.set), new no(s, r, n);
}
const Vt = {}, Yt = /* @__PURE__ */ new WeakMap();
let Ge;
function ro(e, t = false, n = Ge) {
  if (n) {
    let s = Yt.get(n);
    s || Yt.set(n, s = []), s.push(e);
  }
}
function io(e, t, n = U) {
  const { immediate: s, deep: r, once: i, scheduler: o, augmentJob: l, call: a } = n, d = (k) => r ? k : ue(k) || r === false || r === 0 ? Fe(k, 1) : Fe(k);
  let u, p, S, T, R = false, I = false;
  if (Z(e) ? (p = () => e.value, R = ue(e)) : Xe(e) ? (p = () => d(e), R = true) : P(e) ? (I = true, R = e.some((k) => Xe(k) || ue(k)), p = () => e.map((k) => {
    if (Z(k)) return k.value;
    if (Xe(k)) return d(k);
    if (O(k)) return a ? a(k, 2) : k();
  })) : O(e) ? t ? p = a ? () => a(e, 2) : e : p = () => {
    if (S) {
      ke();
      try {
        S();
      } finally {
        Ce();
      }
    }
    const k = Ge;
    Ge = u;
    try {
      return a ? a(e, 3, [T]) : e(T);
    } finally {
      Ge = k;
    }
  } : p = Ae, t && r) {
    const k = p, z = r === true ? 1 / 0 : r;
    p = () => Fe(k(), z);
  }
  const V = Ri(), B = () => {
    u.stop(), V && V.active && Jn(V.effects, u);
  };
  if (i && t) {
    const k = t;
    t = (...z) => {
      const ge = k(...z);
      return B(), ge;
    };
  }
  let D = I ? new Array(e.length).fill(Vt) : Vt;
  const W = (k) => {
    if (!(!(u.flags & 1) || !u.dirty && !k)) if (t) {
      const z = u.run();
      if (k || r || R || (I ? z.some((ge, _e) => Ee(ge, D[_e])) : Ee(z, D))) {
        S && S();
        const ge = Ge;
        Ge = u;
        try {
          const _e = [z, D === Vt ? void 0 : I && D[0] === Vt ? [] : D, T];
          D = z, a ? a(t, 3, _e) : t(..._e);
        } finally {
          Ge = ge;
        }
      }
    } else u.run();
  };
  return l && l(W), u = new nr(p), u.scheduler = o ? () => o(W, false) : W, T = (k) => ro(k, false, u), S = u.onStop = () => {
    const k = Yt.get(u);
    if (k) {
      if (a) a(k, 4);
      else for (const z of k) z();
      Yt.delete(u);
    }
  }, t ? s ? W(true) : D = u.run() : o ? o(W.bind(null, true), true) : u.run(), B.pause = u.pause.bind(u), B.resume = u.resume.bind(u), B.stop = B, B;
}
function Fe(e, t = 1 / 0, n) {
  if (t <= 0 || !N(e) || e.__v_skip || (n = n || /* @__PURE__ */ new Map(), (n.get(e) || 0) >= t)) return e;
  if (n.set(e, t), t--, Z(e)) Fe(e.value, t, n);
  else if (P(e)) for (let s = 0; s < e.length; s++) Fe(e[s], t, n);
  else if (zs(e) || st(e)) e.forEach((s) => {
    Fe(s, t, n);
  });
  else if (Xs(e)) {
    for (const s in e) Fe(e[s], t, n);
    for (const s of Object.getOwnPropertySymbols(e)) Object.prototype.propertyIsEnumerable.call(e, s) && Fe(e[s], t, n);
  }
  return e;
}
/**
* @vue/runtime-core v3.5.39
* (c) 2018-present Yuxi (Evan) You and Vue contributors
* @license MIT
**/
function Nt(e, t, n, s) {
  try {
    return s ? e(...s) : e();
  } catch (r) {
    hn(r, t, n);
  }
}
function He(e, t, n, s) {
  if (O(e)) {
    const r = Nt(e, t, n, s);
    return r && Gs(r) && r.catch((i) => {
      hn(i, t, n);
    }), r;
  }
  if (P(e)) {
    const r = [];
    for (let i = 0; i < e.length; i++) r.push(He(e[i], t, n, s));
    return r;
  }
}
function hn(e, t, n, s = true) {
  const r = t ? t.vnode : null, { errorHandler: i, throwUnhandledErrorInProduction: o } = t && t.appContext.config || U;
  if (t) {
    let l = t.parent;
    const a = t.proxy, d = `https://vuejs.org/error-reference/#runtime-${n}`;
    for (; l; ) {
      const u = l.ec;
      if (u) {
        for (let p = 0; p < u.length; p++) if (u[p](e, a, d) === false) return;
      }
      l = l.parent;
    }
    if (i) {
      ke(), Nt(i, null, 10, [e, a, d]), Ce();
      return;
    }
  }
  oo(e, n, r, s, o);
}
function oo(e, t, n, s = true, r = false) {
  if (r) throw e;
  console.error(e);
}
const se = [];
let xe = -1;
const rt = [];
let $e = null, et = 0;
const yr = Promise.resolve();
let Xt = null;
function br(e) {
  const t = Xt || yr;
  return e ? t.then(this ? e.bind(this) : e) : t;
}
function lo(e) {
  let t = xe + 1, n = se.length;
  for (; t < n; ) {
    const s = t + n >>> 1, r = se[s], i = Ot(r);
    i < e || i === e && r.flags & 2 ? t = s + 1 : n = s;
  }
  return t;
}
function is(e) {
  if (!(e.flags & 1)) {
    const t = Ot(e), n = se[se.length - 1];
    !n || !(e.flags & 2) && t >= Ot(n) ? se.push(e) : se.splice(lo(t), 0, e), e.flags |= 1, vr();
  }
}
function vr() {
  Xt || (Xt = yr.then(xr));
}
function co(e) {
  P(e) ? rt.push(...e) : $e && e.id === -1 ? $e.splice(et + 1, 0, e) : e.flags & 1 || (rt.push(e), e.flags |= 1), vr();
}
function Es(e, t, n = xe + 1) {
  for (; n < se.length; n++) {
    const s = se[n];
    if (s && s.flags & 2) {
      if (e && s.id !== e.uid) continue;
      se.splice(n, 1), n--, s.flags & 4 && (s.flags &= -2), s(), s.flags & 4 || (s.flags &= -2);
    }
  }
}
function wr(e) {
  if (rt.length) {
    const t = [...new Set(rt)].sort((n, s) => Ot(n) - Ot(s));
    if (rt.length = 0, $e) {
      $e.push(...t);
      return;
    }
    for ($e = t, et = 0; et < $e.length; et++) {
      const n = $e[et];
      n.flags & 4 && (n.flags &= -2), n.flags & 8 || n(), n.flags &= -2;
    }
    $e = null, et = 0;
  }
}
const Ot = (e) => e.id == null ? e.flags & 2 ? -1 : 1 / 0 : e.id;
function xr(e) {
  try {
    for (xe = 0; xe < se.length; xe++) {
      const t = se[xe];
      t && !(t.flags & 8) && (t.flags & 4 && (t.flags &= -2), Nt(t, t.i, t.i ? 15 : 14), t.flags & 4 || (t.flags &= -2));
    }
  } finally {
    for (; xe < se.length; xe++) {
      const t = se[xe];
      t && (t.flags &= -2);
    }
    xe = -1, se.length = 0, wr(), Xt = null, (se.length || rt.length) && xr();
  }
}
let ce = null, Sr = null;
function Qt(e) {
  const t = ce;
  return ce = e, Sr = e && e.type.__scopeId || null, t;
}
function ao(e, t = ce, n) {
  if (!t || e._n) return e;
  const s = (...r) => {
    s._d && tn(-1);
    const i = Qt(t);
    let o;
    try {
      o = e(...r);
    } finally {
      Qt(i), s._d && tn(1);
    }
    return o;
  };
  return s._n = true, s._c = true, s._d = true, s;
}
function Jl(e, t) {
  if (ce === null) return e;
  const n = mn(ce), s = e.dirs || (e.dirs = []);
  for (let r = 0; r < t.length; r++) {
    let [i, o, l, a = U] = t[r];
    i && (O(i) && (i = { mounted: i, updated: i }), i.deep && Fe(o), s.push({ dir: i, instance: n, value: o, oldValue: void 0, arg: l, modifiers: a }));
  }
  return e;
}
function Je(e, t, n, s) {
  const r = e.dirs, i = t && t.dirs;
  for (let o = 0; o < r.length; o++) {
    const l = r[o];
    i && (l.oldValue = i[o].value);
    let a = l.dir[s];
    a && (ke(), He(a, n, 8, [e.el, l, e, t]), Ce());
  }
}
function fo(e, t) {
  if (Q) {
    let n = Q.provides;
    const s = Q.parent && Q.parent.provides;
    s === n && (n = Q.provides = Object.create(s)), n[e] = t;
  }
}
function Jt(e, t, n = false) {
  const s = hl();
  if (s || it) {
    let r = it ? it._context.provides : s ? s.parent == null || s.ce ? s.vnode.appContext && s.vnode.appContext.provides : s.parent.provides : void 0;
    if (r && e in r) return r[e];
    if (arguments.length > 1) return n && O(t) ? t.call(s && s.proxy) : t;
  }
}
const uo = Symbol.for("v-scx"), ho = () => Jt(uo);
function An(e, t, n) {
  return Tr(e, t, n);
}
function Tr(e, t, n = U) {
  const { immediate: s, deep: r, flush: i, once: o } = n, l = ee({}, n), a = t && s || !t && i !== "post";
  let d;
  if (Rt) {
    if (i === "sync") {
      const T = ho();
      d = T.__watcherHandles || (T.__watcherHandles = []);
    } else if (!a) {
      const T = () => {
      };
      return T.stop = Ae, T.resume = Ae, T.pause = Ae, T;
    }
  }
  const u = Q;
  l.call = (T, R, I) => He(T, u, R, I);
  let p = false;
  i === "post" ? l.scheduler = (T) => {
    ie(T, u && u.suspense);
  } : i !== "sync" && (p = true, l.scheduler = (T, R) => {
    R ? T() : is(T);
  }), l.augmentJob = (T) => {
    t && (T.flags |= 4), p && (T.flags |= 2, u && (T.id = u.uid, T.i = u));
  };
  const S = io(e, t, l);
  return Rt && (d ? d.push(S) : a && S()), S;
}
function po(e, t, n) {
  const s = this.proxy, r = re(e) ? e.includes(".") ? Er(s, e) : () => s[e] : e.bind(s, s);
  let i;
  O(t) ? i = t : (i = t.handler, n = t);
  const o = Ht(this), l = Tr(r, i.bind(s), n);
  return o(), l;
}
function Er(e, t) {
  const n = t.split(".");
  return () => {
    let s = e;
    for (let r = 0; r < n.length && s; r++) s = s[n[r]];
    return s;
  };
}
const go = Symbol("_vte"), _o = (e) => e.__isTeleport, In = Symbol("_leaveCb");
function os(e, t) {
  e.shapeFlag & 6 && e.component ? (e.transition = t, os(e.component.subTree, t)) : e.shapeFlag & 128 ? (e.ssContent.transition = t.clone(e.ssContent), e.ssFallback.transition = t.clone(e.ssFallback)) : e.transition = t;
}
function zl(e, t) {
  return O(e) ? ee({ name: e.name }, t, { setup: e }) : e;
}
function Ar(e) {
  e.ids = [e.ids[0] + e.ids[2]++ + "-", 0, 0];
}
function As(e, t) {
  let n;
  return !!((n = Object.getOwnPropertyDescriptor(e, t)) && !n.configurable);
}
const Zt = /* @__PURE__ */ new WeakMap();
function Et(e, t, n, s, r = false) {
  if (P(e)) {
    e.forEach((I, V) => Et(I, t && (P(t) ? t[V] : t), n, s, r));
    return;
  }
  if (At(s) && !r) {
    s.shapeFlag & 512 && s.type.__asyncResolved && s.component.subTree.component && Et(e, t, n, s.component.subTree);
    return;
  }
  const i = s.shapeFlag & 4 ? mn(s.component) : s.el, o = r ? null : i, { i: l, r: a } = e, d = t && t.r, u = l.refs === U ? l.refs = {} : l.refs, p = l.setupState, S = F(p), T = p === U ? Js : (I) => As(u, I) ? false : j(S, I), R = (I, V) => !(V && As(u, V));
  if (d != null && d !== a) {
    if (Is(t), re(d)) u[d] = null, T(d) && (p[d] = null);
    else if (Z(d)) {
      const I = t;
      R(d, I.k) && (d.value = null), I.k && (u[I.k] = null);
    }
  }
  if (O(a)) {
    ke();
    try {
      Nt(a, l, 12, [o, u]);
    } finally {
      Ce();
    }
  } else {
    const I = re(a), V = Z(a);
    if (I || V) {
      const B = () => {
        if (e.f) {
          const D = I ? T(a) ? p[a] : u[a] : R() || !e.k ? a.value : u[e.k];
          if (r) P(D) && Jn(D, i);
          else if (P(D)) D.includes(i) || D.push(i);
          else if (I) u[a] = [i], T(a) && (p[a] = u[a]);
          else {
            const W = [i];
            R(a, e.k) && (a.value = W), e.k && (u[e.k] = W);
          }
        } else I ? (u[a] = o, T(a) && (p[a] = o)) : V && (R(a, e.k) && (a.value = o), e.k && (u[e.k] = o));
      };
      if (o) {
        const D = () => {
          B(), Zt.delete(e);
        };
        D.id = -1, Zt.set(e, D), ie(D, n);
      } else Is(e), B();
    }
  }
}
function Is(e) {
  const t = Zt.get(e);
  t && (t.flags |= 8, Zt.delete(e));
}
fn().requestIdleCallback;
fn().cancelIdleCallback;
const At = (e) => !!e.type.__asyncLoader, Ir = (e) => e.type.__isKeepAlive;
function mo(e, t) {
  kr(e, "a", t);
}
function yo(e, t) {
  kr(e, "da", t);
}
function kr(e, t, n = Q) {
  const s = e.__wdc || (e.__wdc = () => {
    let r = n;
    for (; r; ) {
      if (r.isDeactivated) return;
      r = r.parent;
    }
    return e();
  });
  if (pn(t, s, n), n) {
    let r = n.parent;
    for (; r && r.parent; ) Ir(r.parent.vnode) && bo(s, t, n, r), r = r.parent;
  }
}
function bo(e, t, n, s) {
  const r = pn(t, e, s, true);
  Cr(() => {
    Jn(s[t], r);
  }, n);
}
function pn(e, t, n = Q, s = false) {
  if (n) {
    const r = n[e] || (n[e] = []), i = t.__weh || (t.__weh = (...o) => {
      ke();
      const l = Ht(n), a = He(t, n, e, o);
      return l(), Ce(), a;
    });
    return s ? r.unshift(i) : r.push(i), i;
  }
}
const Le = (e) => (t, n = Q) => {
  (!Rt || e === "sp") && pn(e, (...s) => t(...s), n);
}, vo = Le("bm"), wo = Le("m"), xo = Le("bu"), So = Le("u"), To = Le("bum"), Cr = Le("um"), Eo = Le("sp"), Ao = Le("rtg"), Io = Le("rtc");
function ko(e, t = Q) {
  pn("ec", e, t);
}
const Co = "directives", Po = Symbol.for("v-ndc");
function Gl(e) {
  return Oo(Co, e);
}
function Oo(e, t, n = true, s = false) {
  const r = ce || Q;
  if (r) {
    const i = r.type, o = ks(r[e] || i[e], t) || ks(r.appContext[e], t);
    return !o && s ? i : o;
  }
}
function ks(e, t) {
  return e && (e[t] || e[Ke(t)] || e[Qs(Ke(t))]);
}
function Yl(e, t, n, s) {
  let r;
  const i = n, o = P(e);
  if (o || re(e)) {
    const l = o && Xe(e);
    let a = false, d = false;
    l && (a = !ue(e), d = Ne(e), e = dn(e)), r = new Array(e.length);
    for (let u = 0, p = e.length; u < p; u++) r[u] = t(a ? d ? ot(pe(e[u])) : pe(e[u]) : e[u], u, void 0, i);
  } else if (typeof e == "number") {
    r = new Array(e);
    for (let l = 0; l < e; l++) r[l] = t(l + 1, l, void 0, i);
  } else if (N(e)) if (e[Symbol.iterator]) r = Array.from(e, (l, a) => t(l, a, void 0, i));
  else {
    const l = Object.keys(e);
    r = new Array(l.length);
    for (let a = 0, d = l.length; a < d; a++) {
      const u = l[a];
      r[a] = t(e[u], u, a, i);
    }
  }
  else r = [];
  return r;
}
const Hn = (e) => e ? Yr(e) ? mn(e) : Hn(e.parent) : null, It = ee(/* @__PURE__ */ Object.create(null), { $: (e) => e, $el: (e) => e.vnode.el, $data: (e) => e.data, $props: (e) => e.props, $attrs: (e) => e.attrs, $slots: (e) => e.slots, $refs: (e) => e.refs, $parent: (e) => Hn(e.parent), $root: (e) => Hn(e.root), $host: (e) => e.ce, $emit: (e) => e.emit, $options: (e) => Or(e), $forceUpdate: (e) => e.f || (e.f = () => {
  is(e.update);
}), $nextTick: (e) => e.n || (e.n = br.bind(e.proxy)), $watch: (e) => po.bind(e) }), kn = (e, t) => e !== U && !e.__isScriptSetup && j(e, t), Mo = { get({ _: e }, t) {
  if (t === "__v_skip") return true;
  const { ctx: n, setupState: s, data: r, props: i, accessCache: o, type: l, appContext: a } = e;
  if (t[0] !== "$") {
    const S = o[t];
    if (S !== void 0) switch (S) {
      case 1:
        return s[t];
      case 2:
        return r[t];
      case 4:
        return n[t];
      case 3:
        return i[t];
    }
    else {
      if (kn(s, t)) return o[t] = 1, s[t];
      if (r !== U && j(r, t)) return o[t] = 2, r[t];
      if (j(i, t)) return o[t] = 3, i[t];
      if (n !== U && j(n, t)) return o[t] = 4, n[t];
      Ln && (o[t] = 0);
    }
  }
  const d = It[t];
  let u, p;
  if (d) return t === "$attrs" && X(e.attrs, "get", ""), d(e);
  if ((u = l.__cssModules) && (u = u[t])) return u;
  if (n !== U && j(n, t)) return o[t] = 4, n[t];
  if (p = a.config.globalProperties, j(p, t)) return p[t];
}, set({ _: e }, t, n) {
  const { data: s, setupState: r, ctx: i } = e;
  return kn(r, t) ? (r[t] = n, true) : s !== U && j(s, t) ? (s[t] = n, true) : j(e.props, t) || t[0] === "$" && t.slice(1) in e ? false : (i[t] = n, true);
}, has({ _: { data: e, setupState: t, accessCache: n, ctx: s, appContext: r, props: i, type: o } }, l) {
  let a;
  return !!(n[l] || e !== U && l[0] !== "$" && j(e, l) || kn(t, l) || j(i, l) || j(s, l) || j(It, l) || j(r.config.globalProperties, l) || (a = o.__cssModules) && a[l]);
}, defineProperty(e, t, n) {
  return n.get != null ? e._.accessCache[t] = 0 : j(n, "value") && this.set(e, t, n.value, null), Reflect.defineProperty(e, t, n);
} };
function Cs(e) {
  return P(e) ? e.reduce((t, n) => (t[n] = null, t), {}) : e;
}
let Ln = true;
function Ro(e) {
  const t = Or(e), n = e.proxy, s = e.ctx;
  Ln = false, t.beforeCreate && Ps(t.beforeCreate, e, "bc");
  const { data: r, computed: i, methods: o, watch: l, provide: a, inject: d, created: u, beforeMount: p, mounted: S, beforeUpdate: T, updated: R, activated: I, deactivated: V, beforeDestroy: B, beforeUnmount: D, destroyed: W, unmounted: k, render: z, renderTracked: ge, renderTriggered: _e, errorCaptured: Ue, serverPrefetch: Lt, expose: Be, inheritAttrs: ut, components: Ut, directives: $t, filters: yn } = t;
  if (d && Do(d, s, null), o) for (const q in o) {
    const $ = o[q];
    O($) && (s[q] = $.bind(n));
  }
  if (r) {
    const q = r.call(n, n);
    N(q) && (e.data = ns(q));
  }
  if (Ln = true, i) for (const q in i) {
    const $ = i[q], qe = O($) ? $.bind(n, n) : O($.get) ? $.get.bind(n, n) : Ae, Kt = !O($) && O($.set) ? $.set.bind(n) : Ae, Ve = bl({ get: qe, set: Kt });
    Object.defineProperty(s, q, { enumerable: true, configurable: true, get: () => Ve.value, set: (me) => Ve.value = me });
  }
  if (l) for (const q in l) Pr(l[q], s, n, q);
  if (a) {
    const q = O(a) ? a.call(n) : a;
    Reflect.ownKeys(q).forEach(($) => {
      fo($, q[$]);
    });
  }
  u && Ps(u, e, "c");
  function te(q, $) {
    P($) ? $.forEach((qe) => q(qe.bind(n))) : $ && q($.bind(n));
  }
  if (te(vo, p), te(wo, S), te(xo, T), te(So, R), te(mo, I), te(yo, V), te(ko, Ue), te(Io, ge), te(Ao, _e), te(To, D), te(Cr, k), te(Eo, Lt), P(Be)) if (Be.length) {
    const q = e.exposed || (e.exposed = {});
    Be.forEach(($) => {
      Object.defineProperty(q, $, { get: () => n[$], set: (qe) => n[$] = qe, enumerable: true });
    });
  } else e.exposed || (e.exposed = {});
  z && e.render === Ae && (e.render = z), ut != null && (e.inheritAttrs = ut), Ut && (e.components = Ut), $t && (e.directives = $t), Lt && Ar(e);
}
function Do(e, t, n = Ae) {
  P(e) && (e = Un(e));
  for (const s in e) {
    const r = e[s];
    let i;
    N(r) ? "default" in r ? i = Jt(r.from || s, r.default, true) : i = Jt(r.from || s) : i = Jt(r), Z(i) ? Object.defineProperty(t, s, { enumerable: true, configurable: true, get: () => i.value, set: (o) => i.value = o }) : t[s] = i;
  }
}
function Ps(e, t, n) {
  He(P(e) ? e.map((s) => s.bind(t.proxy)) : e.bind(t.proxy), t, n);
}
function Pr(e, t, n, s) {
  let r = s.includes(".") ? Er(n, s) : () => n[s];
  if (re(e)) {
    const i = t[e];
    O(i) && An(r, i);
  } else if (O(e)) An(r, e.bind(n));
  else if (N(e)) if (P(e)) e.forEach((i) => Pr(i, t, n, s));
  else {
    const i = O(e.handler) ? e.handler.bind(n) : t[e.handler];
    O(i) && An(r, i, e);
  }
}
function Or(e) {
  const t = e.type, { mixins: n, extends: s } = t, { mixins: r, optionsCache: i, config: { optionMergeStrategies: o } } = e.appContext, l = i.get(t);
  let a;
  return l ? a = l : !r.length && !n && !s ? a = t : (a = {}, r.length && r.forEach((d) => en(a, d, o, true)), en(a, t, o)), N(t) && i.set(t, a), a;
}
function en(e, t, n, s = false) {
  const { mixins: r, extends: i } = t;
  i && en(e, i, n, true), r && r.forEach((o) => en(e, o, n, true));
  for (const o in t) if (!(s && o === "expose")) {
    const l = Fo[o] || n && n[o];
    e[o] = l ? l(e[o], t[o]) : t[o];
  }
  return e;
}
const Fo = { data: Os, props: Ms, emits: Ms, methods: vt, computed: vt, beforeCreate: ne, created: ne, beforeMount: ne, mounted: ne, beforeUpdate: ne, updated: ne, beforeDestroy: ne, beforeUnmount: ne, destroyed: ne, unmounted: ne, activated: ne, deactivated: ne, errorCaptured: ne, serverPrefetch: ne, components: vt, directives: vt, watch: No, provide: Os, inject: jo };
function Os(e, t) {
  return t ? e ? function() {
    return ee(O(e) ? e.call(this, this) : e, O(t) ? t.call(this, this) : t);
  } : t : e;
}
function jo(e, t) {
  return vt(Un(e), Un(t));
}
function Un(e) {
  if (P(e)) {
    const t = {};
    for (let n = 0; n < e.length; n++) t[e[n]] = e[n];
    return t;
  }
  return e;
}
function ne(e, t) {
  return e ? [...new Set([].concat(e, t))] : t;
}
function vt(e, t) {
  return e ? ee(/* @__PURE__ */ Object.create(null), e, t) : t;
}
function Ms(e, t) {
  return e ? P(e) && P(t) ? [.../* @__PURE__ */ new Set([...e, ...t])] : ee(/* @__PURE__ */ Object.create(null), Cs(e), Cs(t ?? {})) : t;
}
function No(e, t) {
  if (!e) return t;
  if (!t) return e;
  const n = ee(/* @__PURE__ */ Object.create(null), e);
  for (const s in t) n[s] = ne(e[s], t[s]);
  return n;
}
function Mr() {
  return { app: null, config: { isNativeTag: Js, performance: false, globalProperties: {}, optionMergeStrategies: {}, errorHandler: void 0, warnHandler: void 0, compilerOptions: {} }, mixins: [], components: {}, directives: {}, provides: /* @__PURE__ */ Object.create(null), optionsCache: /* @__PURE__ */ new WeakMap(), propsCache: /* @__PURE__ */ new WeakMap(), emitsCache: /* @__PURE__ */ new WeakMap() };
}
let Ho = 0;
function Lo(e, t) {
  return function(s, r = null) {
    O(s) || (s = ee({}, s)), r != null && !N(r) && (r = null);
    const i = Mr(), o = /* @__PURE__ */ new WeakSet(), l = [];
    let a = false;
    const d = i.app = { _uid: Ho++, _component: s, _props: r, _container: null, _context: i, _instance: null, version: vl, get config() {
      return i.config;
    }, set config(u) {
    }, use(u, ...p) {
      return o.has(u) || (u && O(u.install) ? (o.add(u), u.install(d, ...p)) : O(u) && (o.add(u), u(d, ...p))), d;
    }, mixin(u) {
      return i.mixins.includes(u) || i.mixins.push(u), d;
    }, component(u, p) {
      return p ? (i.components[u] = p, d) : i.components[u];
    }, directive(u, p) {
      return p ? (i.directives[u] = p, d) : i.directives[u];
    }, mount(u, p, S) {
      if (!a) {
        const T = d._ceVNode || oe(s, r);
        return T.appContext = i, S === true ? S = "svg" : S === false && (S = void 0), e(T, u, S), a = true, d._container = u, u.__vue_app__ = d, mn(T.component);
      }
    }, onUnmount(u) {
      l.push(u);
    }, unmount() {
      a && (He(l, d._instance, 16), e(null, d._container), delete d._container.__vue_app__);
    }, provide(u, p) {
      return i.provides[u] = p, d;
    }, runWithContext(u) {
      const p = it;
      it = d;
      try {
        return u();
      } finally {
        it = p;
      }
    } };
    return d;
  };
}
let it = null;
const Uo = (e, t) => t === "modelValue" || t === "model-value" ? e.modelModifiers : e[`${t}Modifiers`] || e[`${Ke(t)}Modifiers`] || e[`${jt(t)}Modifiers`];
function $o(e, t, ...n) {
  if (e.isUnmounted) return;
  const s = e.vnode.props || U;
  let r = n;
  const i = t.startsWith("update:"), o = i && Uo(s, t.slice(7));
  o && (o.trim && (r = n.map((u) => re(u) ? u.trim() : u)), o.number && (r = n.map(Ti)));
  let l, a = s[l = vn(t)] || s[l = vn(Ke(t))];
  !a && i && (a = s[l = vn(jt(t))]), a && He(a, e, 6, r);
  const d = s[l + "Once"];
  if (d) {
    if (!e.emitted) e.emitted = {};
    else if (e.emitted[l]) return;
    e.emitted[l] = true, He(d, e, 6, r);
  }
}
const Ko = /* @__PURE__ */ new WeakMap();
function Rr(e, t, n = false) {
  const s = n ? Ko : t.emitsCache, r = s.get(e);
  if (r !== void 0) return r;
  const i = e.emits;
  let o = {}, l = false;
  if (!O(e)) {
    const a = (d) => {
      const u = Rr(d, t, true);
      u && (l = true, ee(o, u));
    };
    !n && t.mixins.length && t.mixins.forEach(a), e.extends && a(e.extends), e.mixins && e.mixins.forEach(a);
  }
  return !i && !l ? (N(e) && s.set(e, null), null) : (P(i) ? i.forEach((a) => o[a] = null) : ee(o, i), N(e) && s.set(e, o), o);
}
function gn(e, t) {
  return !e || !qn(t) ? false : (t = t.slice(2), t = t === "Once" ? t : t.replace(/Once$/, ""), j(e, t[0].toLowerCase() + t.slice(1)) || j(e, jt(t)) || j(e, t));
}
function Rs(e) {
  const { type: t, vnode: n, proxy: s, withProxy: r, propsOptions: [i], slots: o, attrs: l, emit: a, render: d, renderCache: u, props: p, data: S, setupState: T, ctx: R, inheritAttrs: I } = e, V = Qt(e);
  let B, D;
  try {
    if (n.shapeFlag & 4) {
      const k = r || s, z = k;
      B = Te(d.call(z, k, u, p, T, S, R)), D = l;
    } else {
      const k = t;
      B = Te(k.length > 1 ? k(p, { attrs: l, slots: o, emit: a }) : k(p, null)), D = t.props ? l : Wo(l);
    }
  } catch (k) {
    kt.length = 0, hn(k, e, 1), B = oe(We);
  }
  let W = B;
  if (D && I !== false) {
    const k = Object.keys(D), { shapeFlag: z } = W;
    k.length && z & 7 && (i && k.some(Vn) && (D = Bo(D, i)), W = lt(W, D, false, true));
  }
  return n.dirs && (W = lt(W, null, false, true), W.dirs = W.dirs ? W.dirs.concat(n.dirs) : n.dirs), n.transition && os(W, n.transition), B = W, Qt(V), B;
}
const Wo = (e) => {
  let t;
  for (const n in e) (n === "class" || n === "style" || qn(n)) && ((t || (t = {}))[n] = e[n]);
  return t;
}, Bo = (e, t) => {
  const n = {};
  for (const s in e) (!Vn(s) || !(s.slice(9) in t)) && (n[s] = e[s]);
  return n;
};
function qo(e, t, n) {
  const { props: s, children: r, component: i } = e, { props: o, children: l, patchFlag: a } = t, d = i.emitsOptions;
  if (t.dirs || t.transition) return true;
  if (n && a >= 0) {
    if (a & 1024) return true;
    if (a & 16) return s ? Ds(s, o, d) : !!o;
    if (a & 8) {
      const u = t.dynamicProps;
      for (let p = 0; p < u.length; p++) {
        const S = u[p];
        if (Dr(o, s, S) && !gn(d, S)) return true;
      }
    }
  } else return (r || l) && (!l || !l.$stable) ? true : s === o ? false : s ? o ? Ds(s, o, d) : true : !!o;
  return false;
}
function Ds(e, t, n) {
  const s = Object.keys(t);
  if (s.length !== Object.keys(e).length) return true;
  for (let r = 0; r < s.length; r++) {
    const i = s[r];
    if (Dr(t, e, i) && !gn(n, i)) return true;
  }
  return false;
}
function Dr(e, t, n) {
  const s = e[n], r = t[n];
  return n === "style" && N(s) && N(r) ? !un(s, r) : s !== r;
}
function Vo({ vnode: e, parent: t, suspense: n }, s) {
  for (; t; ) {
    const r = t.subTree;
    if (r.suspense && r.suspense.activeBranch === e && (r.suspense.vnode.el = r.el = s, e = r), r === e) (e = t.vnode).el = s, t = t.parent;
    else break;
  }
  n && n.activeBranch === e && (n.vnode.el = s);
}
const Fr = {}, jr = () => Object.create(Fr), Nr = (e) => Object.getPrototypeOf(e) === Fr;
function Jo(e, t, n, s = false) {
  const r = {}, i = jr();
  e.propsDefaults = /* @__PURE__ */ Object.create(null), Hr(e, t, r, i);
  for (const o in e.propsOptions[0]) o in r || (r[o] = void 0);
  n ? e.props = s ? r : Xi(r) : e.type.props ? e.props = r : e.props = i, e.attrs = i;
}
function zo(e, t, n, s) {
  const { props: r, attrs: i, vnode: { patchFlag: o } } = e, l = F(r), [a] = e.propsOptions;
  let d = false;
  if ((s || o > 0) && !(o & 16)) {
    if (o & 8) {
      const u = e.vnode.dynamicProps;
      for (let p = 0; p < u.length; p++) {
        let S = u[p];
        if (gn(e.emitsOptions, S)) continue;
        const T = t[S];
        if (a) if (j(i, S)) T !== i[S] && (i[S] = T, d = true);
        else {
          const R = Ke(S);
          r[R] = $n(a, l, R, T, e, false);
        }
        else T !== i[S] && (i[S] = T, d = true);
      }
    }
  } else {
    Hr(e, t, r, i) && (d = true);
    let u;
    for (const p in l) (!t || !j(t, p) && ((u = jt(p)) === p || !j(t, u))) && (a ? n && (n[p] !== void 0 || n[u] !== void 0) && (r[p] = $n(a, l, p, void 0, e, true)) : delete r[p]);
    if (i !== l) for (const p in i) (!t || !j(t, p)) && (delete i[p], d = true);
  }
  d && De(e.attrs, "set", "");
}
function Hr(e, t, n, s) {
  const [r, i] = e.propsOptions;
  let o = false, l;
  if (t) for (let a in t) {
    if (xt(a)) continue;
    const d = t[a];
    let u;
    r && j(r, u = Ke(a)) ? !i || !i.includes(u) ? n[u] = d : (l || (l = {}))[u] = d : gn(e.emitsOptions, a) || (!(a in s) || d !== s[a]) && (s[a] = d, o = true);
  }
  if (i) {
    const a = F(n), d = l || U;
    for (let u = 0; u < i.length; u++) {
      const p = i[u];
      n[p] = $n(r, a, p, d[p], e, !j(d, p));
    }
  }
  return o;
}
function $n(e, t, n, s, r, i) {
  const o = e[n];
  if (o != null) {
    const l = j(o, "default");
    if (l && s === void 0) {
      const a = o.default;
      if (o.type !== Function && !o.skipFactory && O(a)) {
        const { propsDefaults: d } = r;
        if (n in d) s = d[n];
        else {
          const u = Ht(r);
          s = d[n] = a.call(null, t), u();
        }
      } else s = a;
      r.ce && r.ce._setProp(n, s);
    }
    o[0] && (i && !l ? s = false : o[1] && (s === "" || s === jt(n)) && (s = true));
  }
  return s;
}
const Go = /* @__PURE__ */ new WeakMap();
function Lr(e, t, n = false) {
  const s = n ? Go : t.propsCache, r = s.get(e);
  if (r) return r;
  const i = e.props, o = {}, l = [];
  let a = false;
  if (!O(e)) {
    const u = (p) => {
      a = true;
      const [S, T] = Lr(p, t, true);
      ee(o, S), T && l.push(...T);
    };
    !n && t.mixins.length && t.mixins.forEach(u), e.extends && u(e.extends), e.mixins && e.mixins.forEach(u);
  }
  if (!i && !a) return N(e) && s.set(e, nt), nt;
  if (P(i)) for (let u = 0; u < i.length; u++) {
    const p = Ke(i[u]);
    Fs(p) && (o[p] = U);
  }
  else if (i) for (const u in i) {
    const p = Ke(u);
    if (Fs(p)) {
      const S = i[u], T = o[p] = P(S) || O(S) ? { type: S } : ee({}, S), R = T.type;
      let I = false, V = true;
      if (P(R)) for (let B = 0; B < R.length; ++B) {
        const D = R[B], W = O(D) && D.name;
        if (W === "Boolean") {
          I = true;
          break;
        } else W === "String" && (V = false);
      }
      else I = O(R) && R.name === "Boolean";
      T[0] = I, T[1] = V, (I || j(T, "default")) && l.push(p);
    }
  }
  const d = [o, l];
  return N(e) && s.set(e, d), d;
}
function Fs(e) {
  return e[0] !== "$" && !xt(e);
}
const ls = (e) => e === "_" || e === "_ctx" || e === "$stable", cs = (e) => P(e) ? e.map(Te) : [Te(e)], Yo = (e, t, n) => {
  if (t._n) return t;
  const s = ao((...r) => cs(t(...r)), n);
  return s._c = false, s;
}, Ur = (e, t, n) => {
  const s = e._ctx;
  for (const r in e) {
    if (ls(r)) continue;
    const i = e[r];
    if (O(i)) t[r] = Yo(r, i, s);
    else if (i != null) {
      const o = cs(i);
      t[r] = () => o;
    }
  }
}, $r = (e, t) => {
  const n = cs(t);
  e.slots.default = () => n;
}, Kr = (e, t, n) => {
  for (const s in t) (n || !ls(s)) && (e[s] = t[s]);
}, Xo = (e, t, n) => {
  const s = e.slots = jr();
  if (e.vnode.shapeFlag & 32) {
    const r = t._;
    r ? (Kr(s, t, n), n && Zs(s, "_", r, true)) : Ur(t, s);
  } else t && $r(e, t);
}, Qo = (e, t, n) => {
  const { vnode: s, slots: r } = e;
  let i = true, o = U;
  if (s.shapeFlag & 32) {
    const l = t._;
    l ? n && l === 1 ? i = false : Kr(r, t, n) : (i = !t.$stable, Ur(t, r)), o = t;
  } else t && ($r(e, t), o = { default: 1 });
  if (i) for (const l in r) !ls(l) && o[l] == null && delete r[l];
}, ie = nl;
function Xl(e) {
  return Zo(e);
}
function Zo(e, t) {
  const n = fn();
  n.__VUE__ = true;
  const { insert: s, remove: r, patchProp: i, createElement: o, createText: l, createComment: a, setText: d, setElementText: u, parentNode: p, nextSibling: S, setScopeId: T = Ae, insertStaticContent: R } = e, I = (c, f, h, y = null, m = null, g = null, w = void 0, v = null, b = !!f.dynamicChildren) => {
    if (c === f) return;
    c && !_t(c, f) && (y = Wt(c), me(c, m, g, true), c = null), f.patchFlag === -2 && (b = false, f.dynamicChildren = null);
    const { type: _, ref: A, shapeFlag: x } = f;
    switch (_) {
      case _n:
        V(c, f, h, y);
        break;
      case We:
        B(c, f, h, y);
        break;
      case zt:
        c == null && D(f, h, y, w);
        break;
      case Me:
        Ut(c, f, h, y, m, g, w, v, b);
        break;
      default:
        x & 1 ? z(c, f, h, y, m, g, w, v, b) : x & 6 ? $t(c, f, h, y, m, g, w, v, b) : (x & 64 || x & 128) && _.process(c, f, h, y, m, g, w, v, b, ht);
    }
    A != null && m ? Et(A, c && c.ref, g, f || c, !f) : A == null && c && c.ref != null && Et(c.ref, null, g, c, true);
  }, V = (c, f, h, y) => {
    if (c == null) s(f.el = l(f.children), h, y);
    else {
      const m = f.el = c.el;
      f.children !== c.children && d(m, f.children);
    }
  }, B = (c, f, h, y) => {
    c == null ? s(f.el = a(f.children || ""), h, y) : f.el = c.el;
  }, D = (c, f, h, y) => {
    [c.el, c.anchor] = R(c.children, f, h, y, c.el, c.anchor);
  }, W = ({ el: c, anchor: f }, h, y) => {
    let m;
    for (; c && c !== f; ) m = S(c), s(c, h, y), c = m;
    s(f, h, y);
  }, k = ({ el: c, anchor: f }) => {
    let h;
    for (; c && c !== f; ) h = S(c), r(c), c = h;
    r(f);
  }, z = (c, f, h, y, m, g, w, v, b) => {
    if (f.type === "svg" ? w = "svg" : f.type === "math" && (w = "mathml"), c == null) ge(f, h, y, m, g, w, v, b);
    else {
      const _ = c.el && c.el._isVueCE ? c.el : null;
      try {
        _ && _._beginPatch(), Lt(c, f, m, g, w, v, b);
      } finally {
        _ && _._endPatch();
      }
    }
  }, ge = (c, f, h, y, m, g, w, v) => {
    let b, _;
    const { props: A, shapeFlag: x, transition: E, dirs: C } = c;
    if (b = c.el = o(c.type, g, A && A.is, A), x & 8 ? u(b, c.children) : x & 16 && Ue(c.children, b, null, y, m, Cn(c, g), w, v), C && Je(c, null, y, "created"), _e(b, c, c.scopeId, w, y), A) {
      for (const L in A) L !== "value" && !xt(L) && i(b, L, null, A[L], g, y);
      "value" in A && i(b, "value", null, A.value, g), (_ = A.onVnodeBeforeMount) && we(_, y, c);
    }
    C && Je(c, null, y, "beforeMount");
    const M = el(m, E);
    M && E.beforeEnter(b), s(b, f, h), ((_ = A && A.onVnodeMounted) || M || C) && ie(() => {
      try {
        _ && we(_, y, c), M && E.enter(b), C && Je(c, null, y, "mounted");
      } finally {
      }
    }, m);
  }, _e = (c, f, h, y, m) => {
    if (h && T(c, h), y) for (let g = 0; g < y.length; g++) T(c, y[g]);
    if (m) {
      let g = m.subTree;
      if (f === g || Vr(g.type) && (g.ssContent === f || g.ssFallback === f)) {
        const w = m.vnode;
        _e(c, w, w.scopeId, w.slotScopeIds, m.parent);
      }
    }
  }, Ue = (c, f, h, y, m, g, w, v, b = 0) => {
    for (let _ = b; _ < c.length; _++) {
      const A = c[_] = v ? Re(c[_]) : Te(c[_]);
      I(null, A, f, h, y, m, g, w, v);
    }
  }, Lt = (c, f, h, y, m, g, w) => {
    const v = f.el = c.el;
    let { patchFlag: b, dynamicChildren: _, dirs: A } = f;
    b |= c.patchFlag & 16;
    const x = c.props || U, E = f.props || U;
    let C;
    if (h && ze(h, false), (C = E.onVnodeBeforeUpdate) && we(C, h, f, c), A && Je(f, c, h, "beforeUpdate"), h && ze(h, true), _ && (!c.dynamicChildren || c.dynamicChildren.length !== _.length) && (b = 0, w = false, _ = null), (x.innerHTML && E.innerHTML == null || x.textContent && E.textContent == null) && u(v, ""), _ ? Be(c.dynamicChildren, _, v, h, y, Cn(f, m), g) : w || $(c, f, v, null, h, y, Cn(f, m), g, false), b > 0) {
      if (b & 16) ut(v, x, E, h, m);
      else if (b & 2 && x.class !== E.class && i(v, "class", null, E.class, m), b & 4 && i(v, "style", x.style, E.style, m), b & 8) {
        const M = f.dynamicProps;
        for (let L = 0; L < M.length; L++) {
          const H = M[L], J = x[H], G = E[H];
          (G !== J || H === "value") && i(v, H, J, G, m, h);
        }
      }
      b & 1 && c.children !== f.children && u(v, f.children);
    } else !w && _ == null && ut(v, x, E, h, m);
    ((C = E.onVnodeUpdated) || A) && ie(() => {
      C && we(C, h, f, c), A && Je(f, c, h, "updated");
    }, y);
  }, Be = (c, f, h, y, m, g, w) => {
    for (let v = 0; v < f.length; v++) {
      const b = c[v], _ = f[v], A = b.el && (b.type === Me || !_t(b, _) || b.shapeFlag & 198) ? p(b.el) : h;
      I(b, _, A, null, y, m, g, w, true);
    }
  }, ut = (c, f, h, y, m) => {
    if (f !== h) {
      if (f !== U) for (const g in f) !xt(g) && !(g in h) && i(c, g, f[g], null, m, y);
      for (const g in h) {
        if (xt(g)) continue;
        const w = h[g], v = f[g];
        w !== v && g !== "value" && i(c, g, v, w, m, y);
      }
      "value" in h && i(c, "value", f.value, h.value, m);
    }
  }, Ut = (c, f, h, y, m, g, w, v, b) => {
    const _ = f.el = c ? c.el : l(""), A = f.anchor = c ? c.anchor : l("");
    let { patchFlag: x, dynamicChildren: E, slotScopeIds: C } = f;
    C && (v = v ? v.concat(C) : C), c == null ? (s(_, h, y), s(A, h, y), Ue(f.children || [], h, A, m, g, w, v, b)) : x > 0 && x & 64 && E && c.dynamicChildren && c.dynamicChildren.length === E.length ? (Be(c.dynamicChildren, E, h, m, g, w, v), (f.key != null || m && f === m.subTree) && Wr(c, f, true)) : $(c, f, h, A, m, g, w, v, b);
  }, $t = (c, f, h, y, m, g, w, v, b) => {
    f.slotScopeIds = v, c == null ? f.shapeFlag & 512 ? m.ctx.activate(f, h, y, w, b) : yn(f, h, y, m, g, w, b) : as(c, f, b);
  }, yn = (c, f, h, y, m, g, w) => {
    const v = c.component = dl(c, y, m);
    if (Ir(c) && (v.ctx.renderer = ht), pl(v, false, w), v.asyncDep) {
      if (m && m.registerDep(v, te, w), !c.el) {
        const b = v.subTree = oe(We);
        B(null, b, f, h), c.placeholder = b.el;
      }
    } else te(v, c, f, h, m, g, w);
  }, as = (c, f, h) => {
    const y = f.component = c.component;
    if (qo(c, f, h)) if (y.asyncDep && !y.asyncResolved) {
      q(y, f, h);
      return;
    } else y.next = f, y.update();
    else f.el = c.el, y.vnode = f;
  }, te = (c, f, h, y, m, g, w) => {
    const v = () => {
      if (c.isMounted) {
        let { next: x, bu: E, u: C, parent: M, vnode: L } = c;
        {
          const be = Br(c);
          if (be) {
            x && (x.el = L.el, q(c, x, w)), be.asyncDep.then(() => {
              ie(() => {
                c.isUnmounted || _();
              }, m);
            });
            return;
          }
        }
        let H = x, J;
        ze(c, false), x ? (x.el = L.el, q(c, x, w)) : x = L, E && wn(E), (J = x.props && x.props.onVnodeBeforeUpdate) && we(J, M, x, L), ze(c, true);
        const G = Rs(c), ye = c.subTree;
        c.subTree = G, I(ye, G, p(ye.el), Wt(ye), c, m, g), x.el = G.el, H === null && Vo(c, G.el), C && ie(C, m), (J = x.props && x.props.onVnodeUpdated) && ie(() => we(J, M, x, L), m);
      } else {
        let x;
        const { el: E, props: C } = f, { bm: M, m: L, parent: H, root: J, type: G } = c, ye = At(f);
        ze(c, false), M && wn(M), !ye && (x = C && C.onVnodeBeforeMount) && we(x, H, f), ze(c, true);
        {
          J.ce && J.ce._hasShadowRoot() && J.ce._injectChildStyle(G, c.parent ? c.parent.type : void 0);
          const be = c.subTree = Rs(c);
          I(null, be, h, y, c, m, g), f.el = be.el;
        }
        if (L && ie(L, m), !ye && (x = C && C.onVnodeMounted)) {
          const be = f;
          ie(() => we(x, H, be), m);
        }
        (f.shapeFlag & 256 || H && At(H.vnode) && H.vnode.shapeFlag & 256) && c.a && ie(c.a, m), c.isMounted = true, f = h = y = null;
      }
    };
    c.scope.on();
    const b = c.effect = new nr(v);
    c.scope.off();
    const _ = c.update = b.run.bind(b), A = c.job = b.runIfDirty.bind(b);
    A.i = c, A.id = c.uid, b.scheduler = () => is(A), ze(c, true), _();
  }, q = (c, f, h) => {
    f.component = c;
    const y = c.vnode.props;
    c.vnode = f, c.next = null, zo(c, f.props, y, h), Qo(c, f.children, h), ke(), Es(c), Ce();
  }, $ = (c, f, h, y, m, g, w, v, b = false) => {
    const _ = c && c.children, A = c ? c.shapeFlag : 0, x = f.children, { patchFlag: E, shapeFlag: C } = f;
    if (E > 0) {
      if (E & 128) {
        Kt(_, x, h, y, m, g, w, v, b);
        return;
      } else if (E & 256) {
        qe(_, x, h, y, m, g, w, v, b);
        return;
      }
    }
    C & 8 ? (A & 16 && dt(_, m, g), x !== _ && u(h, x)) : A & 16 ? C & 16 ? Kt(_, x, h, y, m, g, w, v, b) : dt(_, m, g, true) : (A & 8 && u(h, ""), C & 16 && Ue(x, h, y, m, g, w, v, b));
  }, qe = (c, f, h, y, m, g, w, v, b) => {
    c = c || nt, f = f || nt;
    const _ = c.length, A = f.length, x = Math.min(_, A);
    let E;
    for (E = 0; E < x; E++) {
      const C = f[E] = b ? Re(f[E]) : Te(f[E]);
      I(c[E], C, h, null, m, g, w, v, b);
    }
    _ > A ? dt(c, m, g, true, false, x) : Ue(f, h, y, m, g, w, v, b, x);
  }, Kt = (c, f, h, y, m, g, w, v, b) => {
    let _ = 0;
    const A = f.length;
    let x = c.length - 1, E = A - 1;
    for (; _ <= x && _ <= E; ) {
      const C = c[_], M = f[_] = b ? Re(f[_]) : Te(f[_]);
      if (_t(C, M)) I(C, M, h, null, m, g, w, v, b);
      else break;
      _++;
    }
    for (; _ <= x && _ <= E; ) {
      const C = c[x], M = f[E] = b ? Re(f[E]) : Te(f[E]);
      if (_t(C, M)) I(C, M, h, null, m, g, w, v, b);
      else break;
      x--, E--;
    }
    if (_ > x) {
      if (_ <= E) {
        const C = E + 1, M = C < A ? f[C].el : y;
        for (; _ <= E; ) I(null, f[_] = b ? Re(f[_]) : Te(f[_]), h, M, m, g, w, v, b), _++;
      }
    } else if (_ > E) for (; _ <= x; ) me(c[_], m, g, true), _++;
    else {
      const C = _, M = _, L = /* @__PURE__ */ new Map();
      for (_ = M; _ <= E; _++) {
        const le = f[_] = b ? Re(f[_]) : Te(f[_]);
        le.key != null && L.set(le.key, _);
      }
      let H, J = 0;
      const G = E - M + 1;
      let ye = false, be = 0;
      const pt = new Array(G);
      for (_ = 0; _ < G; _++) pt[_] = 0;
      for (_ = C; _ <= x; _++) {
        const le = c[_];
        if (J >= G) {
          me(le, m, g, true);
          continue;
        }
        let ve;
        if (le.key != null) ve = L.get(le.key);
        else for (H = M; H <= E; H++) if (pt[H - M] === 0 && _t(le, f[H])) {
          ve = H;
          break;
        }
        ve === void 0 ? me(le, m, g, true) : (pt[ve - M] = _ + 1, ve >= be ? be = ve : ye = true, I(le, f[ve], h, null, m, g, w, v, b), J++);
      }
      const ds = ye ? tl(pt) : nt;
      for (H = ds.length - 1, _ = G - 1; _ >= 0; _--) {
        const le = M + _, ve = f[le], hs = f[le + 1], ps = le + 1 < A ? hs.el || qr(hs) : y;
        pt[_] === 0 ? I(null, ve, h, ps, m, g, w, v, b) : ye && (H < 0 || _ !== ds[H] ? Ve(ve, h, ps, 2) : H--);
      }
    }
  }, Ve = (c, f, h, y, m = null) => {
    const { el: g, type: w, transition: v, children: b, shapeFlag: _ } = c;
    if (_ & 6) {
      Ve(c.component.subTree, f, h, y);
      return;
    }
    if (_ & 128) {
      c.suspense.move(f, h, y);
      return;
    }
    if (_ & 64) {
      w.move(c, f, h, ht);
      return;
    }
    if (w === Me) {
      s(g, f, h);
      for (let x = 0; x < b.length; x++) Ve(b[x], f, h, y);
      s(c.anchor, f, h);
      return;
    }
    if (w === zt) {
      W(c, f, h);
      return;
    }
    if (y !== 2 && _ & 1 && v) if (y === 0) v.persisted && !g[In] ? s(g, f, h) : (v.beforeEnter(g), s(g, f, h), ie(() => v.enter(g), m));
    else {
      const { leave: x, delayLeave: E, afterLeave: C } = v, M = () => {
        c.ctx.isUnmounted ? r(g) : s(g, f, h);
      }, L = () => {
        const H = g._isLeaving || !!g[In];
        g._isLeaving && g[In](true), v.persisted && !H ? M() : x(g, () => {
          M(), C && C();
        });
      };
      E ? E(g, M, L) : L();
    }
    else s(g, f, h);
  }, me = (c, f, h, y = false, m = false) => {
    const { type: g, props: w, ref: v, children: b, dynamicChildren: _, shapeFlag: A, patchFlag: x, dirs: E, cacheIndex: C, memo: M } = c;
    if (x === -2 && (m = false), v != null && (ke(), Et(v, null, h, c, true), Ce()), C != null && (f.renderCache[C] = void 0), A & 256) {
      f.ctx.deactivate(c);
      return;
    }
    const L = A & 1 && E, H = !At(c);
    let J;
    if (H && (J = w && w.onVnodeBeforeUnmount) && we(J, f, c), A & 6) Zr(c.component, h, y);
    else {
      if (A & 128) {
        c.suspense.unmount(h, y);
        return;
      }
      L && Je(c, null, f, "beforeUnmount"), A & 64 ? c.type.remove(c, f, h, ht, y) : _ && !_.hasOnce && (g !== Me || x > 0 && x & 64) ? dt(_, f, h, false, true) : (g === Me && x & 384 || !m && A & 16) && dt(b, f, h), y && fs(c);
    }
    const G = M != null && C == null;
    (H && (J = w && w.onVnodeUnmounted) || L || G) && ie(() => {
      J && we(J, f, c), L && Je(c, null, f, "unmounted"), G && (c.el = null);
    }, h);
  }, fs = (c) => {
    const { type: f, el: h, anchor: y, transition: m } = c;
    if (f === Me) {
      Qr(h, y);
      return;
    }
    if (f === zt) {
      k(c);
      return;
    }
    const g = () => {
      r(h), m && !m.persisted && m.afterLeave && m.afterLeave();
    };
    if (c.shapeFlag & 1 && m && !m.persisted) {
      const { leave: w, delayLeave: v } = m, b = () => w(h, g);
      v ? v(c.el, g, b) : b();
    } else g();
  }, Qr = (c, f) => {
    let h;
    for (; c !== f; ) h = S(c), r(c), c = h;
    r(f);
  }, Zr = (c, f, h) => {
    const { bum: y, scope: m, job: g, subTree: w, um: v, m: b, a: _ } = c;
    js(b), js(_), y && wn(y), m.stop(), g && (g.flags |= 8, me(w, c, f, h)), v && ie(v, f), ie(() => {
      c.isUnmounted = true;
    }, f);
  }, dt = (c, f, h, y = false, m = false, g = 0) => {
    for (let w = g; w < c.length; w++) me(c[w], f, h, y, m);
  }, Wt = (c) => {
    if (c.shapeFlag & 6) return Wt(c.component.subTree);
    if (c.shapeFlag & 128) return c.suspense.next();
    const f = S(c.anchor || c.el), h = f && f[go];
    return h ? S(h) : f;
  };
  let bn = false;
  const us = (c, f, h) => {
    let y;
    c == null ? f._vnode && (me(f._vnode, null, null, true), y = f._vnode.component) : I(f._vnode || null, c, f, null, null, null, h), f._vnode = c, bn || (bn = true, Es(y), wr(), bn = false);
  }, ht = { p: I, um: me, m: Ve, r: fs, mt: yn, mc: Ue, pc: $, pbc: Be, n: Wt, o: e };
  return { render: us, hydrate: void 0, createApp: Lo(us) };
}
function Cn({ type: e, props: t }, n) {
  return n === "svg" && e === "foreignObject" || n === "mathml" && e === "annotation-xml" && t && t.encoding && t.encoding.includes("html") ? void 0 : n;
}
function ze({ effect: e, job: t }, n) {
  n ? (e.flags |= 32, t.flags |= 4) : (e.flags &= -33, t.flags &= -5);
}
function el(e, t) {
  return (!e || e && !e.pendingBranch) && t && !t.persisted;
}
function Wr(e, t, n = false) {
  const s = e.children, r = t.children;
  if (P(s) && P(r)) for (let i = 0; i < s.length; i++) {
    const o = s[i];
    let l = r[i];
    l.shapeFlag & 1 && !l.dynamicChildren && ((l.patchFlag <= 0 || l.patchFlag === 32) && (l = r[i] = Re(r[i]), l.el = o.el), !n && l.patchFlag !== -2 && Wr(o, l)), l.type === _n && (l.patchFlag === -1 && (l = r[i] = Re(l)), l.el = o.el), l.type === We && !l.el && (l.el = o.el);
  }
}
function tl(e) {
  const t = e.slice(), n = [0];
  let s, r, i, o, l;
  const a = e.length;
  for (s = 0; s < a; s++) {
    const d = e[s];
    if (d !== 0) {
      if (r = n[n.length - 1], e[r] < d) {
        t[s] = r, n.push(s);
        continue;
      }
      for (i = 0, o = n.length - 1; i < o; ) l = i + o >> 1, e[n[l]] < d ? i = l + 1 : o = l;
      d < e[n[i]] && (i > 0 && (t[s] = n[i - 1]), n[i] = s);
    }
  }
  for (i = n.length, o = n[i - 1]; i-- > 0; ) n[i] = o, o = t[o];
  return n;
}
function Br(e) {
  const t = e.subTree.component;
  if (t) return t.asyncDep && !t.asyncResolved ? t : Br(t);
}
function js(e) {
  if (e) for (let t = 0; t < e.length; t++) e[t].flags |= 8;
}
function qr(e) {
  if (e.placeholder) return e.placeholder;
  const t = e.component;
  return t ? qr(t.subTree) : null;
}
const Vr = (e) => e.__isSuspense;
function nl(e, t) {
  t && t.pendingBranch ? P(e) ? t.effects.push(...e) : t.effects.push(e) : co(e);
}
const Me = Symbol.for("v-fgt"), _n = Symbol.for("v-txt"), We = Symbol.for("v-cmt"), zt = Symbol.for("v-stc"), kt = [];
let ae = null;
function sl(e = false) {
  kt.push(ae = e ? null : []);
}
function rl() {
  kt.pop(), ae = kt[kt.length - 1] || null;
}
let Mt = 1;
function tn(e, t = false) {
  Mt += e, e < 0 && ae && t && (ae.hasOnce = true);
}
function Jr(e) {
  return e.dynamicChildren = Mt > 0 ? ae || nt : null, rl(), Mt > 0 && ae && ae.push(e), e;
}
function Ql(e, t, n, s, r, i) {
  return Jr(Gr(e, t, n, s, r, i, true));
}
function il(e, t, n, s, r) {
  return Jr(oe(e, t, n, s, r, true));
}
function nn(e) {
  return e ? e.__v_isVNode === true : false;
}
function _t(e, t) {
  return e.type === t.type && e.key === t.key;
}
const zr = ({ key: e }) => e ?? null, Gt = ({ ref: e, ref_key: t, ref_for: n }) => (typeof e == "number" && (e = "" + e), e != null ? re(e) || Z(e) || O(e) ? { i: ce, r: e, k: t, f: !!n } : e : null);
function Gr(e, t = null, n = null, s = 0, r = null, i = e === Me ? 0 : 1, o = false, l = false) {
  const a = { __v_isVNode: true, __v_skip: true, type: e, props: t, key: t && zr(t), ref: t && Gt(t), scopeId: Sr, slotScopeIds: null, children: n, component: null, suspense: null, ssContent: null, ssFallback: null, dirs: null, transition: null, el: null, anchor: null, target: null, targetStart: null, targetAnchor: null, staticCount: 0, shapeFlag: i, patchFlag: s, dynamicProps: r, dynamicChildren: null, appContext: null, ctx: ce };
  return l ? (sn(a, n), i & 128 && e.normalize(a)) : n && (a.shapeFlag |= re(n) ? 8 : 16), Mt > 0 && !o && ae && (a.patchFlag > 0 || i & 6) && a.patchFlag !== 32 && ae.push(a), a;
}
const oe = ol;
function ol(e, t = null, n = null, s = 0, r = null, i = false) {
  if ((!e || e === Po) && (e = We), nn(e)) {
    const l = lt(e, t, true);
    return n && sn(l, n), Mt > 0 && !i && ae && (l.shapeFlag & 6 ? ae[ae.indexOf(e)] = l : ae.push(l)), l.patchFlag = -2, l;
  }
  if (yl(e) && (e = e.__vccOpts), t) {
    t = ll(t);
    let { class: l, style: a } = t;
    l && !re(l) && (t.class = Yn(l)), N(a) && (rs(a) && !P(a) && (a = ee({}, a)), t.style = Gn(a));
  }
  const o = re(e) ? 1 : Vr(e) ? 128 : _o(e) ? 64 : N(e) ? 4 : O(e) ? 2 : 0;
  return Gr(e, t, n, s, r, o, i, true);
}
function ll(e) {
  return e ? rs(e) || Nr(e) ? ee({}, e) : e : null;
}
function lt(e, t, n = false, s = false) {
  const { props: r, ref: i, patchFlag: o, children: l, transition: a } = e, d = t ? al(r || {}, t) : r, u = { __v_isVNode: true, __v_skip: true, type: e.type, props: d, key: d && zr(d), ref: t && t.ref ? n && i ? P(i) ? i.concat(Gt(t)) : [i, Gt(t)] : Gt(t) : i, scopeId: e.scopeId, slotScopeIds: e.slotScopeIds, children: l, target: e.target, targetStart: e.targetStart, targetAnchor: e.targetAnchor, staticCount: e.staticCount, shapeFlag: e.shapeFlag, patchFlag: t && e.type !== Me ? o === -1 ? 16 : o | 16 : o, dynamicProps: e.dynamicProps, dynamicChildren: e.dynamicChildren, appContext: e.appContext, dirs: e.dirs, transition: a, component: e.component, suspense: e.suspense, ssContent: e.ssContent && lt(e.ssContent), ssFallback: e.ssFallback && lt(e.ssFallback), placeholder: e.placeholder, el: e.el, anchor: e.anchor, ctx: e.ctx, ce: e.ce };
  return a && s && os(u, a.clone(u)), u;
}
function cl(e = " ", t = 0) {
  return oe(_n, null, e, t);
}
function Zl(e, t) {
  const n = oe(zt, null, e);
  return n.staticCount = t, n;
}
function ec(e = "", t = false) {
  return t ? (sl(), il(We, null, e)) : oe(We, null, e);
}
function Te(e) {
  return e == null || typeof e == "boolean" ? oe(We) : P(e) ? oe(Me, null, e.slice()) : nn(e) ? Re(e) : oe(_n, null, String(e));
}
function Re(e) {
  return e.el === null && e.patchFlag !== -1 || e.memo ? e : lt(e);
}
function sn(e, t) {
  let n = 0;
  const { shapeFlag: s } = e;
  if (t == null) t = null;
  else if (P(t)) n = 16;
  else if (typeof t == "object") if (s & 65) {
    const r = t.default;
    r && (r._c && (r._d = false), sn(e, r()), r._c && (r._d = true));
    return;
  } else {
    n = 32;
    const r = t._;
    !r && !Nr(t) ? t._ctx = ce : r === 3 && ce && (ce.slots._ === 1 ? t._ = 1 : (t._ = 2, e.patchFlag |= 1024));
  }
  else if (O(t)) {
    if (s & 65) {
      sn(e, { default: t });
      return;
    }
    t = { default: t, _ctx: ce }, n = 32;
  } else t = String(t), s & 64 ? (n = 16, t = [cl(t)]) : n = 8;
  e.children = t, e.shapeFlag |= n;
}
function al(...e) {
  const t = {};
  for (let n = 0; n < e.length; n++) {
    const s = e[n];
    for (const r in s) if (r === "class") t.class !== s.class && (t.class = Yn([t.class, s.class]));
    else if (r === "style") t.style = Gn([t.style, s.style]);
    else if (qn(r)) {
      const i = t[r], o = s[r];
      o && i !== o && !(P(i) && i.includes(o)) ? t[r] = i ? [].concat(i, o) : o : o == null && i == null && !Vn(r) && (t[r] = o);
    } else r !== "" && (t[r] = s[r]);
  }
  return t;
}
function we(e, t, n, s = null) {
  He(e, t, 7, [n, s]);
}
const fl = Mr();
let ul = 0;
function dl(e, t, n) {
  const s = e.type, r = (t ? t.appContext : e.appContext) || fl, i = { uid: ul++, vnode: e, type: s, parent: t, appContext: r, root: null, next: null, subTree: null, effect: null, update: null, job: null, scope: new Mi(true), render: null, proxy: null, exposed: null, exposeProxy: null, withProxy: null, provides: t ? t.provides : Object.create(r.provides), ids: t ? t.ids : ["", 0, 0], accessCache: null, renderCache: [], components: null, directives: null, propsOptions: Lr(s, r), emitsOptions: Rr(s, r), emit: null, emitted: null, propsDefaults: U, inheritAttrs: s.inheritAttrs, ctx: U, data: U, props: U, attrs: U, slots: U, refs: U, setupState: U, setupContext: null, suspense: n, suspenseId: n ? n.pendingId : 0, asyncDep: null, asyncResolved: false, isMounted: false, isUnmounted: false, isDeactivated: false, bc: null, c: null, bm: null, m: null, bu: null, u: null, um: null, bum: null, da: null, a: null, rtg: null, rtc: null, ec: null, sp: null };
  return i.ctx = { _: i }, i.root = t ? t.root : i, i.emit = $o.bind(null, i), e.ce && e.ce(i), i;
}
let Q = null;
const hl = () => Q || ce;
let rn, Kn;
{
  const e = fn(), t = (n, s) => {
    let r;
    return (r = e[n]) || (r = e[n] = []), r.push(s), (i) => {
      r.length > 1 ? r.forEach((o) => o(i)) : r[0](i);
    };
  };
  rn = t("__VUE_INSTANCE_SETTERS__", (n) => Q = n), Kn = t("__VUE_SSR_SETTERS__", (n) => Rt = n);
}
const Ht = (e) => {
  const t = Q;
  return rn(e), e.scope.on(), () => {
    e.scope.off(), rn(t);
  };
}, Ns = () => {
  Q && Q.scope.off(), rn(null);
};
function Yr(e) {
  return e.vnode.shapeFlag & 4;
}
let Rt = false;
function pl(e, t = false, n = false) {
  t && Kn(t);
  const { props: s, children: r } = e.vnode, i = Yr(e);
  Jo(e, s, i, t), Xo(e, r, n || t);
  const o = i ? gl(e, t) : void 0;
  return t && Kn(false), o;
}
function gl(e, t) {
  const n = e.type;
  e.accessCache = /* @__PURE__ */ Object.create(null), e.proxy = new Proxy(e.ctx, Mo);
  const { setup: s } = n;
  if (s) {
    ke();
    const r = e.setupContext = s.length > 1 ? ml(e) : null, i = Ht(e), o = Nt(s, e, 0, [e.props, r]), l = Gs(o);
    if (Ce(), i(), (l || e.sp) && !At(e) && Ar(e), l) {
      if (o.then(Ns, Ns), t) return o.then((a) => {
        Hs(e, a);
      }).catch((a) => {
        hn(a, e, 0);
      });
      e.asyncDep = o;
    } else Hs(e, o);
  } else Xr(e);
}
function Hs(e, t, n) {
  O(t) ? e.type.__ssrInlineRender ? e.ssrRender = t : e.render = t : N(t) && (e.setupState = mr(t)), Xr(e);
}
function Xr(e, t, n) {
  const s = e.type;
  e.render || (e.render = s.render || Ae);
  {
    const r = Ht(e);
    ke();
    try {
      Ro(e);
    } finally {
      Ce(), r();
    }
  }
}
const _l = { get(e, t) {
  return X(e, "get", ""), e[t];
} };
function ml(e) {
  const t = (n) => {
    e.exposed = n || {};
  };
  return { attrs: new Proxy(e.attrs, _l), slots: e.slots, emit: e.emit, expose: t };
}
function mn(e) {
  return e.exposed ? e.exposeProxy || (e.exposeProxy = new Proxy(mr(Qi(e.exposed)), { get(t, n) {
    if (n in t) return t[n];
    if (n in It) return It[n](e);
  }, has(t, n) {
    return n in t || n in It;
  } })) : e.proxy;
}
function yl(e) {
  return O(e) && "__vccOpts" in e;
}
const bl = (e, t) => so(e, t, Rt);
function tc(e, t, n) {
  try {
    tn(-1);
    const s = arguments.length;
    return s === 2 ? N(t) && !P(t) ? nn(t) ? oe(e, null, [t]) : oe(e, t) : oe(e, null, t) : (s > 3 ? n = Array.prototype.slice.call(arguments, 2) : s === 3 && nn(n) && (n = [n]), oe(e, t, n));
  } finally {
    tn(1);
  }
}
const vl = "3.5.39";
function Pn(e, t) {
  if (e.length === 0) return null;
  const n = [...e].sort((r, i) => r - i), s = Math.ceil(t / 100 * n.length);
  return n[Math.min(n.length, Math.max(1, s)) - 1];
}
function on(e) {
  const t = e.filter((s) => Number.isFinite(s)), n = t.reduce((s, r) => s + r, 0);
  return { count: t.length, p50: Pn(t, 50), p95: Pn(t, 95), p99: Pn(t, 99), max: t.length ? Math.max(...t) : null, mean: t.length ? n / t.length : null };
}
function wl(e) {
  const t = [];
  for (let n = 1; n < e.length; n++) t.push(e[n] - e[n - 1]);
  return t;
}
function xl(e) {
  const t = on(e), n = (r) => e.length ? e.filter((i) => i > r).length / e.length : null, s = t.p50;
  return { ...t, refresh_estimate_ms: s, over_8_33_ratio: n(1e3 / 120), over_16_67_ratio: n(1e3 / 60), over_1_5x_refresh_ratio: s === null ? null : n(s * 1.5) };
}
const Oe = (e, t) => e === null || t === null ? null : e - t;
function Ls(e) {
  const t = e.worker, n = e.input_ts ?? e.sent;
  return { id: e.id, type: e.type, input_kind: e.input_kind, input_to_send_ms: Oe(e.sent, e.input_ts), roundtrip_ms: Oe(e.received, e.sent), worker_total_ms: (t == null ? void 0 : t.total_ms) ?? null, dispatch_ms: (t == null ? void 0 : t.dispatch_ms) ?? null, view_json_ms: (t == null ? void 0 : t.view_json_ms) ?? null, view_parse_ms: (t == null ? void 0 : t.view_parse_ms) ?? null, adapt_ms: (t == null ? void 0 : t.adapt_ms) ?? null, post_ms: (t == null ? void 0 : t.post_ms) ?? null, transfer_ms: Oe(e.received_epoch_ms, (t == null ? void 0 : t.post_epoch_ms) ?? null), receive_to_apply_ms: Oe(e.apply_start, e.received), apply_ms: Oe(e.apply_end, e.apply_start), vue_patch_ms: Oe(e.patched, e.apply_end), patch_to_frame_ms: Oe(e.frame, e.patched), frame_to_paint_ms: Oe(e.after_paint, e.frame), input_to_paint_ms: Oe(e.after_paint, n), view_json_chars: (t == null ? void 0 : t.view_json_chars) ?? null, envelope_json_chars: (t == null ? void 0 : t.envelope_json_chars) ?? null };
}
function Sl(e) {
  const t = ["input_to_send_ms", "roundtrip_ms", "worker_total_ms", "dispatch_ms", "view_json_ms", "view_parse_ms", "adapt_ms", "post_ms", "transfer_ms", "receive_to_apply_ms", "apply_ms", "vue_patch_ms", "patch_to_frame_ms", "frame_to_paint_ms", "input_to_paint_ms", "view_json_chars", "envelope_json_chars"], n = {};
  for (const s of t) {
    const r = e.map((i) => i[s]).filter((i) => typeof i == "number" && Number.isFinite(i));
    n[s] = on(r);
  }
  return n;
}
const Tl = "noname-next-perf", El = 3e5, Al = 2e4, Wn = 5e4, Il = 100;
let Us = false, fe = false;
const mt = [], yt = [], bt = [], Ie = /* @__PURE__ */ new Map(), ct = /* @__PURE__ */ new Map();
let wt = null, at = null;
function kl() {
  var _a;
  try {
    if (((_a = globalThis.localStorage) == null ? void 0 : _a.getItem(Tl)) === "1") return true;
  } catch {
  }
  if (typeof location > "u") return false;
  if (new URLSearchParams(location.search).get("perf") === "1") return true;
  const e = location.hash.split("?")[1];
  return e ? new URLSearchParams(e).get("perf") === "1" : false;
}
const Cl = () => performance.timeOrigin + performance.now();
function $s(e, t, n) {
  var _a;
  if (typeof PerformanceObserver > "u" || !((_a = PerformanceObserver.supportedEntryTypes) == null ? void 0 : _a.includes(e))) return;
  new PerformanceObserver((r) => {
    for (const i of r.getEntries()) n(i);
  }).observe({ type: e, buffered: true, ...t });
}
function Ks(e) {
  wt = { ts: e.timeStamp, kind: e.type, used: false };
}
function Pl() {
  if (Us || (Us = true, fe = kl(), !fe || typeof window > "u")) return fe;
  const e = (t) => {
    mt.length < El && mt.push(t), requestAnimationFrame(e);
  };
  return requestAnimationFrame(e), window.addEventListener("click", Ks, { capture: true }), window.addEventListener("keydown", Ks, { capture: true }), $s("longtask", {}, (t) => {
    yt.length < Wn && yt.push({ start: t.startTime, duration: t.duration });
  }), $s("event", { durationThreshold: 16 }, (t) => {
    const n = t;
    bt.length < Wn && bt.push({ name: n.name, start: n.startTime, duration: n.duration, input_delay: n.processingStart - n.startTime, processing: n.processingEnd - n.processingStart });
  }), window.__nonamePerf = { version: 1, enabled: true, frames: () => mt.slice(), longTasks: () => yt.slice(), eventTimings: () => bt.slice(), interactions: () => [...Ie.values()].map(Ls), summary: () => {
    const t = [...Ie.values()].map(Ls);
    return { frames: xl(wl(mt)), interactions: Sl(t), long_tasks: on(yt.map((n) => n.duration)), event_timings: on(bt.map((n) => n.duration)) };
  }, reset: () => {
    mt.length = 0, yt.length = 0, bt.length = 0, Ie.clear(), ct.clear(), at = null;
  } }, fe && fi(), fe;
}
function Ol(e, t) {
  if (!fe || Ie.size >= Al) return;
  const n = performance.now(), s = wt && !wt.used && n - wt.ts <= Il ? wt : null;
  s && (s.used = true), Ie.set(e, { id: e, type: t, input_ts: (s == null ? void 0 : s.ts) ?? null, input_kind: (s == null ? void 0 : s.kind) ?? null, sent: n, received: null, received_epoch_ms: null, apply_start: null, apply_end: null, patched: null, frame: null, after_paint: null, worker: ct.get(e) ?? null }), ct.delete(e);
}
function Ws(e) {
  fe && (Ie.delete(e), ct.delete(e));
}
function Bs(e) {
  if (!fe) return;
  const t = Ie.get(e);
  t && (t.received = performance.now(), t.received_epoch_ms = Cl(), at = e);
}
function Ml(e, t) {
  if (!fe) return;
  const n = Ie.get(e);
  n ? n.worker = t : ct.size < Wn && ct.set(e, t);
}
function Rl() {
  if (at === null) return null;
  const e = Ie.get(at);
  return e && e.apply_start === null ? e : null;
}
function nc() {
  if (!fe) return;
  const e = Rl();
  e && (e.apply_start = performance.now());
}
function sc() {
  if (!fe || at === null) return;
  const e = Ie.get(at);
  !e || e.apply_start === null || e.apply_end !== null || (e.apply_end = performance.now(), br(() => {
    e.patched = performance.now(), requestAnimationFrame(() => {
      e.frame = performance.now();
      const t = new MessageChannel();
      t.port1.onmessage = () => {
        e.after_paint = performance.now(), t.port1.close();
      }, t.port2.postMessage(0);
    });
  }));
}
function Dl(e) {
  return new Worker("/assets/engine.native.worker-BIwpD907.js", { type: "module", name: e == null ? void 0 : e.name });
}
function Fl(e) {
  return new Worker("/assets/engine.worker-DF5gc_e9.js", { type: "module", name: e == null ? void 0 : e.name });
}
function qs(e) {
  return fetch(e).then((t) => {
    if (!t.ok) throw new Error(`\u538B\u6D4B\u8F7D\u8377\u52A0\u8F7D\u5931\u8D25 ${e}: ${t.status}`);
    return t.text();
  });
}
class rc {
  constructor() {
    this.worker = null, this.transportKind = "wasm-worker", this.seq = 1, this.pending = /* @__PURE__ */ new Map(), this.native = true, this.sabPollSeq = -1;
  }
  getTransportKind() {
    return this.transportKind;
  }
  async connect() {
    if (this.worker) return;
    const t = "native".toLowerCase();
    this.native = t !== "legacy" && t !== "wasm" && t !== "old";
    const n = !this.native && Qe() ? await ni() : void 0, s = this.native && Qe() && mi() === "native";
    this.transportKind = s ? "desktop-native" : this.native ? "wasm-worker" : "legacy-worker", globalThis.__engineTransport = this.transportKind;
    const r = s ? new bi() : this.native ? new Dl() : new Fl();
    this.worker = r, r.onmessage = (i) => {
      var _a, _b;
      if (this.worker !== r) return;
      const o = i.data;
      if ((o == null ? void 0 : o.type) === "init_timings" && o.timings) {
        globalThis.__nativeInitTimings ?? (globalThis.__nativeInitTimings = []), globalThis.__nativeInitTimings.push(o.timings);
        return;
      }
      if ((o == null ? void 0 : o.type) === "candidate_timings" && o.timings) {
        globalThis.__nativeCandidateTimings ?? (globalThis.__nativeCandidateTimings = []), globalThis.__nativeCandidateTimings.push(o.timings);
        return;
      }
      if ((o == null ? void 0 : o.type) === "step_timings" && o.timings) {
        const d = i.data;
        Ml(d.id, d.timings);
        return;
      }
      const l = i.data, a = this.pending.get(l.id);
      a && (Bs(l.id), this.pending.delete(l.id), (_b = (_a = globalThis.__latencySabDebug) == null ? void 0 : _a.settle) == null ? void 0 : _b.call(_a), l.ok ? a.resolve(l.data) : a.reject(new Error(l.error)));
    }, r.onerror = (i) => {
      this.worker === r && (console.error("engine worker error", i), this.rejectPending(new Error("Worker \u53D1\u751F\u9519\u8BEF\uFF0C\u64CD\u4F5C\u7ED3\u679C\u672A\u77E5\uFF1B\u8BF7\u5148\u786E\u8BA4\u5F53\u524D\u5C40\u9762")));
    }, r.addEventListener("messageerror", () => {
      this.worker === r && this.rejectPending(new Error("Worker \u56DE\u5305\u65E0\u6CD5\u8BFB\u53D6\uFF0C\u64CD\u4F5C\u7ED3\u679C\u672A\u77E5\uFF1B\u8BF7\u5148\u786E\u8BA4\u5F53\u524D\u5C40\u9762"));
    });
    try {
      await this.request({ id: this.nextId(), type: "init", bundle: n ?? void 0, perf: Pl(), latencySab: s ? void 0 : this.initSabMirror() });
    } catch (i) {
      throw this.worker === r && this.dispose(), i;
    }
  }
  initSabMirror() {
    if (de("latency") !== "sab" || !crossOriginIsolated || typeof SharedArrayBuffer > "u") return;
    const t = document.createElement("style");
    t.textContent = ".sab-keepalive{animation:sab-pulse 8ms infinite}@keyframes sab-pulse{50%{margin-left:.5px}}", document.head.appendChild(t);
    const n = document.createElement("div");
    n.className = "sab-keepalive", n.style.display = "none", document.body.appendChild(n);
    const s = 256 * 1024, r = new SharedArrayBuffer(s), i = new Int32Array(r, 0, 2);
    Atomics.store(i, 0, 0), Atomics.store(i, 1, 0);
    const o = new TextDecoder(), l = () => {
      n.style.display = "none";
    }, a = { runs: 0, seen: 0, parseErr: 0, miss: 0 }, d = () => {
      a.runs += 1;
      const u = Atomics.load(i, 0);
      if (u !== this.sabPollSeq) {
        a.seen += 1, this.sabPollSeq = u;
        const p = Atomics.load(i, 1);
        if (p > 0 && 8 + p <= s) try {
          const S = new Uint8Array(p);
          S.set(new Uint8Array(r, 8, p));
          const T = JSON.parse(o.decode(S)), R = this.pending.get(T.id);
          if (R) {
            Bs(T.id), this.pending.delete(T.id), l();
            const I = globalThis.__latencySabDebug;
            I && (I.resolvedEarly += 1), T.ok ? R.resolve(T.data) : R.reject(new Error(T.error ?? "worker error"));
          } else a.miss += 1;
        } catch {
          a.parseErr += 1;
        }
      }
      requestAnimationFrame(d);
    };
    return requestAnimationFrame(d), globalThis.__latencySabDebug = { seq: () => Atomics.load(i, 0), raw: () => {
      const u = Atomics.load(i, 1), p = new Uint8Array(Math.min(u, 120));
      return p.set(new Uint8Array(r, 8, p.length)), { len: u, head: o.decode(p) };
    }, wake: () => {
      n.style.display = "block";
    }, settle: l, stats: a, resolvedEarly: 0, resolvedByMessage: 0 }, r;
  }
  isNative() {
    return this.native;
  }
  dispose() {
    const t = this.worker;
    this.worker = null, t == null ? void 0 : t.terminate(), this.rejectPending(new Error("Worker \u5DF2\u5173\u95ED\uFF0C\u64CD\u4F5C\u7ED3\u679C\u672A\u77E5\uFF1B\u8BF7\u5148\u786E\u8BA4\u5F53\u524D\u5C40\u9762"));
  }
  rejectPending(t) {
    var _a, _b;
    if (this.pending.size !== 0) {
      for (const [n, s] of this.pending) Ws(n), s.reject(t);
      this.pending.clear(), (_b = (_a = globalThis.__latencySabDebug) == null ? void 0 : _a.settle) == null ? void 0 : _b.call(_a);
    }
  }
  request(t) {
    return this.worker ? new Promise((n, s) => {
      var _a, _b, _c, _d;
      this.pending.set(t.id, { resolve: n, reject: s }), Ol(t.id, t.type), (_b = (_a = globalThis.__latencySabDebug) == null ? void 0 : _a.wake) == null ? void 0 : _b.call(_a);
      try {
        this.worker.postMessage(t);
      } catch (r) {
        this.pending.delete(t.id), Ws(t.id), this.pending.size === 0 && ((_d = (_c = globalThis.__latencySabDebug) == null ? void 0 : _c.settle) == null ? void 0 : _d.call(_c)), s(r);
      }
    }) : Promise.reject(new Error("worker not connected"));
  }
  nextId() {
    return this.seq++;
  }
  async start(t, n, s = 1, r, i) {
    const o = de("packsUrl"), l = de("hexUrl"), a = o ? await qs(o) : r;
    return l ? (await this.request({ id: this.nextId(), type: "start", players: t, seed: n, observer: s, packsJson: a, mode: i }), this.request({ id: this.nextId(), type: "restore", hex: await qs(l), packsJson: a })) : this.request({ id: this.nextId(), type: "start", players: t, seed: n, observer: s, packsJson: a, mode: i });
  }
  endPhase(t = 0) {
    return this.request({ id: this.nextId(), type: "endPhase", actor: t });
  }
  playCard(t, n, s) {
    return this.request({ id: this.nextId(), type: "playCard", actor: t, card: n, targets: [...s] });
  }
  activateSkill(t, n, s, r) {
    return this.request({ id: this.nextId(), type: "activateSkill", actor: t, skill: n, targets: [...s], cardTargets: [...r] });
  }
  useSkillCard(t, n, s, r, i) {
    return this.request({ id: this.nextId(), type: "useSkillCard", actor: t, skill: n, cardKind: s, targets: [...r], equipSlot: i });
  }
  resolveChoice(t, n, s, r = false) {
    return this.request({ id: this.nextId(), type: "resolveChoice", actor: t, choiceId: n, selections: [...s], cancelled: r });
  }
  autoStep() {
    return this.request({ id: this.nextId(), type: "autoStep" });
  }
  generalCandidates() {
    return this.request({ id: this.nextId(), type: "generalCandidates" });
  }
  chooseGeneral(t, n) {
    return this.request({ id: this.nextId(), type: "chooseGeneral", seat: t, name: n });
  }
  observer() {
    return this.native ? this.request({ id: this.nextId(), type: "observer" }).then((t) => {
      const n = t;
      if (n == null ? void 0 : n.view) return n.view;
      throw new Error("native observer: \u7F3A view");
    }) : this.request({ id: this.nextId(), type: "observer" });
  }
  saveHex() {
    return this.request({ id: this.nextId(), type: "save" }).then((t) => {
      const n = t;
      if (typeof n == "string") return n;
      if (n == null ? void 0 : n.hex) return n.hex;
      throw new Error("native save: \u7F3A hex");
    });
  }
  restoreHex(t, n) {
    return this.request({ id: this.nextId(), type: "restore", hex: t, packsJson: n });
  }
  async validatePackage(t) {
    await this.request({ id: this.nextId(), type: "validatePackage", packJson: t });
  }
  loadReplay(t) {
    return this.request({ id: this.nextId(), type: "loadReplay", json: t });
  }
  seekReplay(t, n, s = 1) {
    return this.request({ id: this.nextId(), type: "seekReplay", step: t, privacy: n, player: s });
  }
  status() {
    return this.request({ id: this.nextId(), type: "status" });
  }
}
export {
  Yn as $,
  ns as A,
  tc as B,
  bl as C,
  bi as D,
  rc as E,
  An as F,
  ql as G,
  fo as H,
  Ql as I,
  Gr as J,
  oe as K,
  ao as L,
  sl as M,
  cl as N,
  wo as O,
  Nl as P,
  Oi as Q,
  ec as R,
  Jl as S,
  Me as T,
  Yl as U,
  Qe as V,
  Ul as W,
  Ll as X,
  hi as Y,
  nc as Z,
  sc as _,
  O as a,
  il as a0,
  Gn as a1,
  Cr as a2,
  Zl as a3,
  $l as a4,
  bs as a5,
  Hl as a6,
  Gl as a7,
  mi as a8,
  gi as a9,
  re as b,
  Xl as c,
  zs as d,
  P as e,
  Bl as f,
  un as g,
  wn as h,
  ft as i,
  jt as j,
  ee as k,
  Ti as l,
  qn as m,
  br as n,
  Vn as o,
  Ke as p,
  Wl as q,
  je as r,
  Kl as s,
  Qs as t,
  He as u,
  eo as v,
  Xi as w,
  Vl as x,
  zl as y,
  Jt as z
};
