var __defProp = Object.defineProperty;
var __defNormalProp = (obj, key, value) => key in obj ? __defProp(obj, key, { enumerable: true, configurable: true, writable: true, value }) : obj[key] = value;
var __publicField = (obj, key, value) => __defNormalProp(obj, typeof key !== "symbol" ? key + "" : key, value);
(function() {
  const n = document.createElement("link").relList;
  if (n && n.supports && n.supports("modulepreload")) return;
  for (const a of document.querySelectorAll('link[rel="modulepreload"]')) s(a);
  new MutationObserver((a) => {
    for (const i of a) if (i.type === "childList") for (const o of i.addedNodes) o.tagName === "LINK" && o.rel === "modulepreload" && s(o);
  }).observe(document, { childList: true, subtree: true });
  function t(a) {
    const i = {};
    return a.integrity && (i.integrity = a.integrity), a.referrerPolicy && (i.referrerPolicy = a.referrerPolicy), a.crossOrigin === "use-credentials" ? i.credentials = "include" : a.crossOrigin === "anonymous" ? i.credentials = "omit" : i.credentials = "same-origin", i;
  }
  function s(a) {
    if (a.ep) return;
    a.ep = true;
    const i = t(a);
    fetch(a.href, i);
  }
})();
/**
* @vue/shared v3.5.39
* (c) 2018-present Yuxi (Evan) You and Vue contributors
* @license MIT
**/
function La(e) {
  const n = /* @__PURE__ */ Object.create(null);
  for (const t of e.split(",")) n[t] = 1;
  return (t) => t in n;
}
const me = {}, ut = [], yn = () => {
}, Ho = () => false, Os = (e) => e.charCodeAt(0) === 111 && e.charCodeAt(1) === 110 && (e.charCodeAt(2) > 122 || e.charCodeAt(2) < 97), Ns = (e) => e.startsWith("onUpdate:"), ze = Object.assign, Ua = (e, n) => {
  const t = e.indexOf(n);
  t > -1 && e.splice(t, 1);
}, Tl = Object.prototype.hasOwnProperty, de = (e, n) => Tl.call(e, n), Q = Array.isArray, ct = (e) => ns(e) === "[object Map]", wt = (e) => ns(e) === "[object Set]", hi = (e) => ns(e) === "[object Date]", Z = (e) => typeof e == "function", be = (e) => typeof e == "string", bn = (e) => typeof e == "symbol", pe = (e) => e !== null && typeof e == "object", Wo = (e) => (pe(e) || Z(e)) && Z(e.then) && Z(e.catch), Bo = Object.prototype.toString, ns = (e) => Bo.call(e), Ol = (e) => ns(e).slice(8, -1), Ko = (e) => ns(e) === "[object Object]", Fa = (e) => be(e) && e !== "NaN" && e[0] !== "-" && "" + parseInt(e, 10) === e, Dt = La(",key,ref,ref_for,ref_key,onVnodeBeforeMount,onVnodeMounted,onVnodeBeforeUpdate,onVnodeUpdated,onVnodeBeforeUnmount,onVnodeUnmounted"), qs = (e) => {
  const n = /* @__PURE__ */ Object.create(null);
  return ((t) => n[t] || (n[t] = e(t)));
}, Nl = /-\w/g, Ge = qs((e) => e.replace(Nl, (n) => n.slice(1).toUpperCase())), ql = /\B([A-Z])/g, Jn = qs((e) => e.replace(ql, "-$1").toLowerCase()), Va = qs((e) => e.charAt(0).toUpperCase() + e.slice(1)), Qs = qs((e) => e ? `on${Va(e)}` : ""), _n = (e, n) => !Object.is(e, n), gs = (e, ...n) => {
  for (let t = 0; t < e.length; t++) e[t](...n);
}, Go = (e, n, t, s = false) => {
  Object.defineProperty(e, n, { configurable: true, enumerable: false, writable: s, value: t });
}, Ms = (e) => {
  const n = parseFloat(e);
  return isNaN(n) ? e : n;
};
let fi;
const Ds = () => fi || (fi = typeof globalThis < "u" ? globalThis : typeof self < "u" ? self : typeof window < "u" ? window : typeof global < "u" ? global : {});
function Kn(e) {
  if (Q(e)) {
    const n = {};
    for (let t = 0; t < e.length; t++) {
      const s = e[t], a = be(s) ? Ul(s) : Kn(s);
      if (a) for (const i in a) n[i] = a[i];
    }
    return n;
  } else if (be(e) || pe(e)) return e;
}
const Ml = /;(?![^(]*\))/g, Dl = /:([^]+)/, Ll = /\/\*[^]*?\*\//g;
function Ul(e) {
  const n = {};
  return e.replace(Ll, "").split(Ml).forEach((t) => {
    if (t) {
      const s = t.split(Dl);
      s.length > 1 && (n[s[0].trim()] = s[1].trim());
    }
  }), n;
}
function Re(e) {
  let n = "";
  if (be(e)) n = e;
  else if (Q(e)) for (let t = 0; t < e.length; t++) {
    const s = Re(e[t]);
    s && (n += s + " ");
  }
  else if (pe(e)) for (const t in e) e[t] && (n += t + " ");
  return n.trim();
}
const Fl = "itemscope,allowfullscreen,formnovalidate,ismap,nomodule,novalidate,readonly", Vl = La(Fl);
function Jo(e) {
  return !!e || e === "";
}
function Hl(e, n) {
  if (e.length !== n.length) return false;
  let t = true;
  for (let s = 0; t && s < e.length; s++) t = bt(e[s], n[s]);
  return t;
}
function bt(e, n) {
  if (e === n) return true;
  let t = hi(e), s = hi(n);
  if (t || s) return t && s ? e.getTime() === n.getTime() : false;
  if (t = bn(e), s = bn(n), t || s) return e === n;
  if (t = Q(e), s = Q(n), t || s) return t && s ? Hl(e, n) : false;
  if (t = pe(e), s = pe(n), t || s) {
    if (!t || !s) return false;
    const a = Object.keys(e).length, i = Object.keys(n).length;
    if (a !== i) return false;
    for (const o in e) {
      const r = e.hasOwnProperty(o), l = n.hasOwnProperty(o);
      if (r && !l || !r && l || !bt(e[o], n[o])) return false;
    }
  }
  return String(e) === String(n);
}
function Ha(e, n) {
  return e.findIndex((t) => bt(t, n));
}
const Yo = (e) => !!(e && e.__v_isRef === true), $ = (e) => be(e) ? e : e == null ? "" : Q(e) || pe(e) && (e.toString === Bo || !Z(e.toString)) ? Yo(e) ? $(e.value) : JSON.stringify(e, Qo, 2) : String(e), Qo = (e, n) => Yo(n) ? Qo(e, n.value) : ct(n) ? { [`Map(${n.size})`]: [...n.entries()].reduce((t, [s, a], i) => (t[Xs(s, i) + " =>"] = a, t), {}) } : wt(n) ? { [`Set(${n.size})`]: [...n.values()].map((t) => Xs(t)) } : bn(n) ? Xs(n) : pe(n) && !Q(n) && !Ko(n) ? String(n) : n, Xs = (e, n = "") => {
  var t;
  return bn(e) ? `Symbol(${(t = e.description) != null ? t : n})` : e;
};
/**
* @vue/reactivity v3.5.39
* (c) 2018-present Yuxi (Evan) You and Vue contributors
* @license MIT
**/
let Ce;
class Wl {
  constructor(n = false) {
    this.detached = n, this._active = true, this._on = 0, this.effects = [], this.cleanups = [], this._isPaused = false, this._warnOnRun = true, this.__v_skip = true, !n && Ce && (Ce.active ? (this.parent = Ce, this.index = (Ce.scopes || (Ce.scopes = [])).push(this) - 1) : (this._active = false, this._warnOnRun = false));
  }
  get active() {
    return this._active;
  }
  pause() {
    if (this._active) {
      this._isPaused = true;
      let n, t;
      if (this.scopes) for (n = 0, t = this.scopes.length; n < t; n++) this.scopes[n].pause();
      for (n = 0, t = this.effects.length; n < t; n++) this.effects[n].pause();
    }
  }
  resume() {
    if (this._active && this._isPaused) {
      this._isPaused = false;
      let n, t;
      if (this.scopes) for (n = 0, t = this.scopes.length; n < t; n++) this.scopes[n].resume();
      for (n = 0, t = this.effects.length; n < t; n++) this.effects[n].resume();
    }
  }
  run(n) {
    if (this._active) {
      const t = Ce;
      try {
        return Ce = this, n();
      } finally {
        Ce = t;
      }
    }
  }
  on() {
    ++this._on === 1 && (this.prevScope = Ce, Ce = this);
  }
  off() {
    if (this._on > 0 && --this._on === 0) {
      if (Ce === this) Ce = this.prevScope;
      else {
        let n = Ce;
        for (; n; ) {
          if (n.prevScope === this) {
            n.prevScope = this.prevScope;
            break;
          }
          n = n.prevScope;
        }
      }
      this.prevScope = void 0;
    }
  }
  stop(n) {
    if (this._active) {
      this._active = false;
      let t, s;
      for (t = 0, s = this.effects.length; t < s; t++) this.effects[t].stop();
      for (this.effects.length = 0, t = 0, s = this.cleanups.length; t < s; t++) this.cleanups[t]();
      if (this.cleanups.length = 0, this.scopes) {
        for (t = 0, s = this.scopes.length; t < s; t++) this.scopes[t].stop(true);
        this.scopes.length = 0;
      }
      if (!this.detached && this.parent && !n) {
        const a = this.parent.scopes.pop();
        a && a !== this && (this.parent.scopes[this.index] = a, a.index = this.index);
      }
      this.parent = void 0;
    }
  }
}
function Bl() {
  return Ce;
}
let ge;
const Zs = /* @__PURE__ */ new WeakSet();
class Xo {
  constructor(n) {
    this.fn = n, this.deps = void 0, this.depsTail = void 0, this.flags = 5, this.next = void 0, this.cleanup = void 0, this.scheduler = void 0, Ce && (Ce.active ? Ce.effects.push(this) : this.flags &= -2);
  }
  pause() {
    this.flags |= 64;
  }
  resume() {
    this.flags & 64 && (this.flags &= -65, Zs.has(this) && (Zs.delete(this), this.trigger()));
  }
  notify() {
    this.flags & 2 && !(this.flags & 32) || this.flags & 8 || er(this);
  }
  run() {
    if (!(this.flags & 1)) return this.fn();
    this.flags |= 2, mi(this), nr(this);
    const n = ge, t = tn;
    ge = this, tn = true;
    try {
      return this.fn();
    } finally {
      tr(this), ge = n, tn = t, this.flags &= -3;
    }
  }
  stop() {
    if (this.flags & 1) {
      for (let n = this.deps; n; n = n.nextDep) Ka(n);
      this.deps = this.depsTail = void 0, mi(this), this.onStop && this.onStop(), this.flags &= -2;
    }
  }
  trigger() {
    this.flags & 64 ? Zs.add(this) : this.scheduler ? this.scheduler() : this.runIfDirty();
  }
  runIfDirty() {
    fa(this) && this.run();
  }
  get dirty() {
    return fa(this);
  }
}
let Zo = 0, Lt, Ut;
function er(e, n = false) {
  if (e.flags |= 8, n) {
    e.next = Ut, Ut = e;
    return;
  }
  e.next = Lt, Lt = e;
}
function Wa() {
  Zo++;
}
function Ba() {
  if (--Zo > 0) return;
  if (Ut) {
    let n = Ut;
    for (Ut = void 0; n; ) {
      const t = n.next;
      n.next = void 0, n.flags &= -9, n = t;
    }
  }
  let e;
  for (; Lt; ) {
    let n = Lt;
    for (Lt = void 0; n; ) {
      const t = n.next;
      if (n.next = void 0, n.flags &= -9, n.flags & 1) try {
        n.trigger();
      } catch (s) {
        e || (e = s);
      }
      n = t;
    }
  }
  if (e) throw e;
}
function nr(e) {
  for (let n = e.deps; n; n = n.nextDep) n.version = -1, n.prevActiveLink = n.dep.activeLink, n.dep.activeLink = n;
}
function tr(e) {
  let n, t = e.depsTail, s = t;
  for (; s; ) {
    const a = s.prevDep;
    s.version === -1 ? (s === t && (t = a), Ka(s), Kl(s)) : n = s, s.dep.activeLink = s.prevActiveLink, s.prevActiveLink = void 0, s = a;
  }
  e.deps = n, e.depsTail = t;
}
function fa(e) {
  for (let n = e.deps; n; n = n.nextDep) if (n.dep.version !== n.version || n.dep.computed && (sr(n.dep.computed) || n.dep.version !== n.version)) return true;
  return !!e._dirty;
}
function sr(e) {
  if (e.flags & 4 && !(e.flags & 16) || (e.flags &= -17, e.globalVersion === Gt) || (e.globalVersion = Gt, !e.isSSR && e.flags & 128 && (!e.deps && !e._dirty || !fa(e)))) return;
  e.flags |= 2;
  const n = e.dep, t = ge, s = tn;
  ge = e, tn = true;
  try {
    nr(e);
    const a = e.fn(e._value);
    (n.version === 0 || _n(a, e._value)) && (e.flags |= 128, e._value = a, n.version++);
  } catch (a) {
    throw n.version++, a;
  } finally {
    ge = t, tn = s, tr(e), e.flags &= -3;
  }
}
function Ka(e, n = false) {
  const { dep: t, prevSub: s, nextSub: a } = e;
  if (s && (s.nextSub = a, e.prevSub = void 0), a && (a.prevSub = s, e.nextSub = void 0), t.subs === e && (t.subs = s, !s && t.computed)) {
    t.computed.flags &= -5;
    for (let i = t.computed.deps; i; i = i.nextDep) Ka(i, true);
  }
  !n && !--t.sc && t.map && t.map.delete(t.key);
}
function Kl(e) {
  const { prevDep: n, nextDep: t } = e;
  n && (n.nextDep = t, e.prevDep = void 0), t && (t.prevDep = n, e.nextDep = void 0);
}
let tn = true;
const ar = [];
function xn() {
  ar.push(tn), tn = false;
}
function $n() {
  const e = ar.pop();
  tn = e === void 0 ? true : e;
}
function mi(e) {
  const { cleanup: n } = e;
  if (e.cleanup = void 0, n) {
    const t = ge;
    ge = void 0;
    try {
      n();
    } finally {
      ge = t;
    }
  }
}
let Gt = 0;
class Gl {
  constructor(n, t) {
    this.sub = n, this.dep = t, this.version = t.version, this.nextDep = this.prevDep = this.nextSub = this.prevSub = this.prevActiveLink = void 0;
  }
}
class Ga {
  constructor(n) {
    this.computed = n, this.version = 0, this.activeLink = void 0, this.subs = void 0, this.map = void 0, this.key = void 0, this.sc = 0, this.__v_skip = true;
  }
  track(n) {
    if (!ge || !tn || ge === this.computed) return;
    let t = this.activeLink;
    if (t === void 0 || t.sub !== ge) t = this.activeLink = new Gl(ge, this), ge.deps ? (t.prevDep = ge.depsTail, ge.depsTail.nextDep = t, ge.depsTail = t) : ge.deps = ge.depsTail = t, ir(t);
    else if (t.version === -1 && (t.version = this.version, t.nextDep)) {
      const s = t.nextDep;
      s.prevDep = t.prevDep, t.prevDep && (t.prevDep.nextDep = s), t.prevDep = ge.depsTail, t.nextDep = void 0, ge.depsTail.nextDep = t, ge.depsTail = t, ge.deps === t && (ge.deps = s);
    }
    return t;
  }
  trigger(n) {
    this.version++, Gt++, this.notify(n);
  }
  notify(n) {
    Wa();
    try {
      for (let t = this.subs; t; t = t.prevSub) t.sub.notify() && t.sub.dep.notify();
    } finally {
      Ba();
    }
  }
}
function ir(e) {
  if (e.dep.sc++, e.sub.flags & 4) {
    const n = e.dep.computed;
    if (n && !e.dep.subs) {
      n.flags |= 20;
      for (let s = n.deps; s; s = s.nextDep) ir(s);
    }
    const t = e.dep.subs;
    t !== e && (e.prevSub = t, t && (t.nextSub = e)), e.dep.subs = e;
  }
}
const ma = /* @__PURE__ */ new WeakMap(), et = Symbol(""), ga = Symbol(""), Jt = Symbol("");
function Ie(e, n, t) {
  if (tn && ge) {
    let s = ma.get(e);
    s || ma.set(e, s = /* @__PURE__ */ new Map());
    let a = s.get(t);
    a || (s.set(t, a = new Ga()), a.map = s, a.key = t), a.track();
  }
}
function In(e, n, t, s, a, i) {
  const o = ma.get(e);
  if (!o) {
    Gt++;
    return;
  }
  const r = (l) => {
    l && l.trigger();
  };
  if (Wa(), n === "clear") o.forEach(r);
  else {
    const l = Q(e), d = l && Fa(t);
    if (l && t === "length") {
      const c = Number(s);
      o.forEach((p, m) => {
        (m === "length" || m === Jt || !bn(m) && m >= c) && r(p);
      });
    } else switch ((t !== void 0 || o.has(void 0)) && r(o.get(t)), d && r(o.get(Jt)), n) {
      case "add":
        l ? d && r(o.get("length")) : (r(o.get(et)), ct(e) && r(o.get(ga)));
        break;
      case "delete":
        l || (r(o.get(et)), ct(e) && r(o.get(ga)));
        break;
      case "set":
        ct(e) && r(o.get(et));
        break;
    }
  }
  Ba();
}
function at(e) {
  const n = ce(e);
  return n === e ? n : (Ie(n, "iterate", Jt), Ze(e) ? n : n.map(an));
}
function Ls(e) {
  return Ie(e = ce(e), "iterate", Jt), e;
}
function mn(e, n) {
  return On(e) ? ft(nt(e) ? an(n) : n) : an(n);
}
const Jl = { __proto__: null, [Symbol.iterator]() {
  return ea(this, Symbol.iterator, (e) => mn(this, e));
}, concat(...e) {
  return at(this).concat(...e.map((n) => Q(n) ? at(n) : n));
}, entries() {
  return ea(this, "entries", (e) => (e[1] = mn(this, e[1]), e));
}, every(e, n) {
  return En(this, "every", e, n, void 0, arguments);
}, filter(e, n) {
  return En(this, "filter", e, n, (t) => t.map((s) => mn(this, s)), arguments);
}, find(e, n) {
  return En(this, "find", e, n, (t) => mn(this, t), arguments);
}, findIndex(e, n) {
  return En(this, "findIndex", e, n, void 0, arguments);
}, findLast(e, n) {
  return En(this, "findLast", e, n, (t) => mn(this, t), arguments);
}, findLastIndex(e, n) {
  return En(this, "findLastIndex", e, n, void 0, arguments);
}, forEach(e, n) {
  return En(this, "forEach", e, n, void 0, arguments);
}, includes(...e) {
  return na(this, "includes", e);
}, indexOf(...e) {
  return na(this, "indexOf", e);
}, join(e) {
  return at(this).join(e);
}, lastIndexOf(...e) {
  return na(this, "lastIndexOf", e);
}, map(e, n) {
  return En(this, "map", e, n, void 0, arguments);
}, pop() {
  return Et(this, "pop");
}, push(...e) {
  return Et(this, "push", e);
}, reduce(e, ...n) {
  return gi(this, "reduce", e, n);
}, reduceRight(e, ...n) {
  return gi(this, "reduceRight", e, n);
}, shift() {
  return Et(this, "shift");
}, some(e, n) {
  return En(this, "some", e, n, void 0, arguments);
}, splice(...e) {
  return Et(this, "splice", e);
}, toReversed() {
  return at(this).toReversed();
}, toSorted(e) {
  return at(this).toSorted(e);
}, toSpliced(...e) {
  return at(this).toSpliced(...e);
}, unshift(...e) {
  return Et(this, "unshift", e);
}, values() {
  return ea(this, "values", (e) => mn(this, e));
} };
function ea(e, n, t) {
  const s = Ls(e), a = s[n]();
  return s !== e && !Ze(e) && (a._next = a.next, a.next = () => {
    const i = a._next();
    return i.done || (i.value = t(i.value)), i;
  }), a;
}
const Yl = Array.prototype;
function En(e, n, t, s, a, i) {
  const o = Ls(e), r = o !== e && !Ze(e), l = o[n];
  if (l !== Yl[n]) {
    const p = l.apply(e, i);
    return r ? an(p) : p;
  }
  let d = t;
  o !== e && (r ? d = function(p, m) {
    return t.call(this, mn(e, p), m, e);
  } : t.length > 2 && (d = function(p, m) {
    return t.call(this, p, m, e);
  }));
  const c = l.call(o, d, s);
  return r && a ? a(c) : c;
}
function gi(e, n, t, s) {
  const a = Ls(e), i = a !== e && !Ze(e);
  let o = t, r = false;
  a !== e && (i ? (r = s.length === 0, o = function(d, c, p) {
    return r && (r = false, d = mn(e, d)), t.call(this, d, mn(e, c), p, e);
  }) : t.length > 3 && (o = function(d, c, p) {
    return t.call(this, d, c, p, e);
  }));
  const l = a[n](o, ...s);
  return r ? mn(e, l) : l;
}
function na(e, n, t) {
  const s = ce(e);
  Ie(s, "iterate", Jt);
  const a = s[n](...t);
  return (a === -1 || a === false) && Qa(t[0]) ? (t[0] = ce(t[0]), s[n](...t)) : a;
}
function Et(e, n, t = []) {
  xn(), Wa();
  const s = ce(e)[n].apply(e, t);
  return Ba(), $n(), s;
}
const Ql = La("__proto__,__v_isRef,__isVue"), or = new Set(Object.getOwnPropertyNames(Symbol).filter((e) => e !== "arguments" && e !== "caller").map((e) => Symbol[e]).filter(bn));
function Xl(e) {
  bn(e) || (e = String(e));
  const n = ce(this);
  return Ie(n, "has", e), n.hasOwnProperty(e);
}
class rr {
  constructor(n = false, t = false) {
    this._isReadonly = n, this._isShallow = t;
  }
  get(n, t, s) {
    if (t === "__v_skip") return n.__v_skip;
    const a = this._isReadonly, i = this._isShallow;
    if (t === "__v_isReactive") return !a;
    if (t === "__v_isReadonly") return a;
    if (t === "__v_isShallow") return i;
    if (t === "__v_raw") return s === (a ? i ? lu : dr : i ? cr : ur).get(n) || Object.getPrototypeOf(n) === Object.getPrototypeOf(s) ? n : void 0;
    const o = Q(n);
    if (!a) {
      let l;
      if (o && (l = Jl[t])) return l;
      if (t === "hasOwnProperty") return Xl;
    }
    const r = Reflect.get(n, t, Te(n) ? n : s);
    if ((bn(t) ? or.has(t) : Ql(t)) || (a || Ie(n, "get", t), i)) return r;
    if (Te(r)) {
      const l = o && Fa(t) ? r : r.value;
      return a && pe(l) ? va(l) : l;
    }
    return pe(r) ? a ? va(r) : xt(r) : r;
  }
}
class lr extends rr {
  constructor(n = false) {
    super(false, n);
  }
  set(n, t, s, a) {
    let i = n[t];
    const o = Q(n) && Fa(t);
    if (!this._isShallow) {
      const d = On(i);
      if (!Ze(s) && !On(s) && (i = ce(i), s = ce(s)), !o && Te(i) && !Te(s)) return d || (i.value = s), true;
    }
    const r = o ? Number(t) < n.length : de(n, t), l = Reflect.set(n, t, s, Te(n) ? n : a);
    return n === ce(a) && l && (r ? _n(s, i) && In(n, "set", t, s) : In(n, "add", t, s)), l;
  }
  deleteProperty(n, t) {
    const s = de(n, t);
    n[t];
    const a = Reflect.deleteProperty(n, t);
    return a && s && In(n, "delete", t, void 0), a;
  }
  has(n, t) {
    const s = Reflect.has(n, t);
    return (!bn(t) || !or.has(t)) && Ie(n, "has", t), s;
  }
  ownKeys(n) {
    return Ie(n, "iterate", Q(n) ? "length" : et), Reflect.ownKeys(n);
  }
}
class Zl extends rr {
  constructor(n = false) {
    super(true, n);
  }
  set(n, t) {
    return true;
  }
  deleteProperty(n, t) {
    return true;
  }
}
const eu = new lr(), nu = new Zl(), tu = new lr(true);
const _a = (e) => e, us = (e) => Reflect.getPrototypeOf(e);
function su(e, n, t) {
  return function(...s) {
    const a = this.__v_raw, i = ce(a), o = ct(i), r = e === "entries" || e === Symbol.iterator && o, l = e === "keys" && o, d = a[e](...s), c = t ? _a : n ? ft : an;
    return !n && Ie(i, "iterate", l ? ga : et), ze(Object.create(d), { next() {
      const { value: p, done: m } = d.next();
      return m ? { value: p, done: m } : { value: r ? [c(p[0]), c(p[1])] : c(p), done: m };
    } });
  };
}
function cs(e) {
  return function(...n) {
    return e === "delete" ? false : e === "clear" ? void 0 : this;
  };
}
function au(e, n) {
  const t = { get(a) {
    const i = this.__v_raw, o = ce(i), r = ce(a);
    e || (_n(a, r) && Ie(o, "get", a), Ie(o, "get", r));
    const { has: l } = us(o), d = n ? _a : e ? ft : an;
    if (l.call(o, a)) return d(i.get(a));
    if (l.call(o, r)) return d(i.get(r));
    i !== o && i.get(a);
  }, get size() {
    const a = this.__v_raw;
    return !e && Ie(ce(a), "iterate", et), a.size;
  }, has(a) {
    const i = this.__v_raw, o = ce(i), r = ce(a);
    return e || (_n(a, r) && Ie(o, "has", a), Ie(o, "has", r)), a === r ? i.has(a) : i.has(a) || i.has(r);
  }, forEach(a, i) {
    const o = this, r = o.__v_raw, l = ce(r), d = n ? _a : e ? ft : an;
    return !e && Ie(l, "iterate", et), r.forEach((c, p) => a.call(i, d(c), d(p), o));
  } };
  return ze(t, e ? { add: cs("add"), set: cs("set"), delete: cs("delete"), clear: cs("clear") } : { add(a) {
    const i = ce(this), o = us(i), r = ce(a), l = !n && !Ze(a) && !On(a) ? r : a;
    return o.has.call(i, l) || _n(a, l) && o.has.call(i, a) || _n(r, l) && o.has.call(i, r) || (i.add(l), In(i, "add", l, l)), this;
  }, set(a, i) {
    !n && !Ze(i) && !On(i) && (i = ce(i));
    const o = ce(this), { has: r, get: l } = us(o);
    let d = r.call(o, a);
    d || (a = ce(a), d = r.call(o, a));
    const c = l.call(o, a);
    return o.set(a, i), d ? _n(i, c) && In(o, "set", a, i) : In(o, "add", a, i), this;
  }, delete(a) {
    const i = ce(this), { has: o, get: r } = us(i);
    let l = o.call(i, a);
    l || (a = ce(a), l = o.call(i, a)), r && r.call(i, a);
    const d = i.delete(a);
    return l && In(i, "delete", a, void 0), d;
  }, clear() {
    const a = ce(this), i = a.size !== 0, o = a.clear();
    return i && In(a, "clear", void 0, void 0), o;
  } }), ["keys", "values", "entries", Symbol.iterator].forEach((a) => {
    t[a] = su(a, e, n);
  }), t;
}
function Ja(e, n) {
  const t = au(e, n);
  return (s, a, i) => a === "__v_isReactive" ? !e : a === "__v_isReadonly" ? e : a === "__v_raw" ? s : Reflect.get(de(t, a) && a in s ? t : s, a, i);
}
const iu = { get: Ja(false, false) }, ou = { get: Ja(false, true) }, ru = { get: Ja(true, false) };
const ur = /* @__PURE__ */ new WeakMap(), cr = /* @__PURE__ */ new WeakMap(), dr = /* @__PURE__ */ new WeakMap(), lu = /* @__PURE__ */ new WeakMap();
function uu(e) {
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
function xt(e) {
  return On(e) ? e : Ya(e, false, eu, iu, ur);
}
function pr(e) {
  return Ya(e, false, tu, ou, cr);
}
function va(e) {
  return Ya(e, true, nu, ru, dr);
}
function Ya(e, n, t, s, a) {
  if (!pe(e) || e.__v_raw && !(n && e.__v_isReactive) || e.__v_skip || !Object.isExtensible(e)) return e;
  const i = a.get(e);
  if (i) return i;
  const o = uu(Ol(e));
  if (o === 0) return e;
  const r = new Proxy(e, o === 2 ? s : t);
  return a.set(e, r), r;
}
function nt(e) {
  return On(e) ? nt(e.__v_raw) : !!(e && e.__v_isReactive);
}
function On(e) {
  return !!(e && e.__v_isReadonly);
}
function Ze(e) {
  return !!(e && e.__v_isShallow);
}
function Qa(e) {
  return e ? !!e.__v_raw : false;
}
function ce(e) {
  const n = e && e.__v_raw;
  return n ? ce(n) : e;
}
function cu(e) {
  return !de(e, "__v_skip") && Object.isExtensible(e) && Go(e, "__v_skip", true), e;
}
const an = (e) => pe(e) ? xt(e) : e, ft = (e) => pe(e) ? va(e) : e;
function Te(e) {
  return e ? e.__v_isRef === true : false;
}
function W(e) {
  return hr(e, false);
}
function du(e) {
  return hr(e, true);
}
function hr(e, n) {
  return Te(e) ? e : new pu(e, n);
}
class pu {
  constructor(n, t) {
    this.dep = new Ga(), this.__v_isRef = true, this.__v_isShallow = false, this._rawValue = t ? n : ce(n), this._value = t ? n : an(n), this.__v_isShallow = t;
  }
  get value() {
    return this.dep.track(), this._value;
  }
  set value(n) {
    const t = this._rawValue, s = this.__v_isShallow || Ze(n) || On(n);
    n = s ? n : ce(n), _n(n, t) && (this._rawValue = n, this._value = s ? n : an(n), this.dep.trigger());
  }
}
function D(e) {
  return Te(e) ? e.value : e;
}
const hu = { get: (e, n, t) => n === "__v_raw" ? e : D(Reflect.get(e, n, t)), set: (e, n, t, s) => {
  const a = e[n];
  return Te(a) && !Te(t) ? (a.value = t, true) : Reflect.set(e, n, t, s);
} };
function fr(e) {
  return nt(e) ? e : new Proxy(e, hu);
}
class fu {
  constructor(n, t, s) {
    this.fn = n, this.setter = t, this._value = void 0, this.dep = new Ga(this), this.__v_isRef = true, this.deps = void 0, this.depsTail = void 0, this.flags = 16, this.globalVersion = Gt - 1, this.next = void 0, this.effect = this, this.__v_isReadonly = !t, this.isSSR = s;
  }
  notify() {
    if (this.flags |= 16, !(this.flags & 8) && ge !== this) return er(this, true), true;
  }
  get value() {
    const n = this.dep.track();
    return sr(this), n && (n.version = this.dep.version), this._value;
  }
  set value(n) {
    this.setter && this.setter(n);
  }
}
function mu(e, n, t = false) {
  let s, a;
  return Z(e) ? s = e : (s = e.get, a = e.set), new fu(s, a, t);
}
const ds = {}, ws = /* @__PURE__ */ new WeakMap();
let Xn;
function gu(e, n = false, t = Xn) {
  if (t) {
    let s = ws.get(t);
    s || ws.set(t, s = []), s.push(e);
  }
}
function _u(e, n, t = me) {
  const { immediate: s, deep: a, once: i, scheduler: o, augmentJob: r, call: l } = t, d = (T) => a ? T : Ze(T) || a === false || a === 0 ? Pn(T, 1) : Pn(T);
  let c, p, m, f, b = false, P = false;
  if (Te(e) ? (p = () => e.value, b = Ze(e)) : nt(e) ? (p = () => d(e), b = true) : Q(e) ? (P = true, b = e.some((T) => nt(T) || Ze(T)), p = () => e.map((T) => {
    if (Te(T)) return T.value;
    if (nt(T)) return d(T);
    if (Z(T)) return l ? l(T, 2) : T();
  })) : Z(e) ? n ? p = l ? () => l(e, 2) : e : p = () => {
    if (m) {
      xn();
      try {
        m();
      } finally {
        $n();
      }
    }
    const T = Xn;
    Xn = c;
    try {
      return l ? l(e, 3, [f]) : e(f);
    } finally {
      Xn = T;
    }
  } : p = yn, n && a) {
    const T = p, H = a === true ? 1 / 0 : a;
    p = () => Pn(T(), H);
  }
  const O = Bl(), w = () => {
    c.stop(), O && O.active && Ua(O.effects, c);
  };
  if (i && n) {
    const T = n;
    n = (...H) => {
      const le = T(...H);
      return w(), le;
    };
  }
  let _ = P ? new Array(e.length).fill(ds) : ds;
  const A = (T) => {
    if (!(!(c.flags & 1) || !c.dirty && !T)) if (n) {
      const H = c.run();
      if (T || a || b || (P ? H.some((le, ne) => _n(le, _[ne])) : _n(H, _))) {
        m && m();
        const le = Xn;
        Xn = c;
        try {
          const ne = [H, _ === ds ? void 0 : P && _[0] === ds ? [] : _, f];
          _ = H, l ? l(n, 3, ne) : n(...ne);
        } finally {
          Xn = le;
        }
      }
    } else c.run();
  };
  return r && r(A), c = new Xo(p), c.scheduler = o ? () => o(A, false) : A, f = (T) => gu(T, false, c), m = c.onStop = () => {
    const T = ws.get(c);
    if (T) {
      if (l) l(T, 4);
      else for (const H of T) H();
      ws.delete(c);
    }
  }, n ? s ? A(true) : _ = c.run() : o ? o(A.bind(null, true), true) : c.run(), w.pause = c.pause.bind(c), w.resume = c.resume.bind(c), w.stop = w, w;
}
function Pn(e, n = 1 / 0, t) {
  if (n <= 0 || !pe(e) || e.__v_skip || (t = t || /* @__PURE__ */ new Map(), (t.get(e) || 0) >= n)) return e;
  if (t.set(e, n), n--, Te(e)) Pn(e.value, n, t);
  else if (Q(e)) for (let s = 0; s < e.length; s++) Pn(e[s], n, t);
  else if (wt(e) || ct(e)) e.forEach((s) => {
    Pn(s, n, t);
  });
  else if (Ko(e)) {
    for (const s in e) Pn(e[s], n, t);
    for (const s of Object.getOwnPropertySymbols(e)) Object.prototype.propertyIsEnumerable.call(e, s) && Pn(e[s], n, t);
  }
  return e;
}
/**
* @vue/runtime-core v3.5.39
* (c) 2018-present Yuxi (Evan) You and Vue contributors
* @license MIT
**/
function ts(e, n, t, s) {
  try {
    return s ? e(...s) : e();
  } catch (a) {
    Us(a, n, t);
  }
}
function on(e, n, t, s) {
  if (Z(e)) {
    const a = ts(e, n, t, s);
    return a && Wo(a) && a.catch((i) => {
      Us(i, n, t);
    }), a;
  }
  if (Q(e)) {
    const a = [];
    for (let i = 0; i < e.length; i++) a.push(on(e[i], n, t, s));
    return a;
  }
}
function Us(e, n, t, s = true) {
  const a = n ? n.vnode : null, { errorHandler: i, throwUnhandledErrorInProduction: o } = n && n.appContext.config || me;
  if (n) {
    let r = n.parent;
    const l = n.proxy, d = `https://vuejs.org/error-reference/#runtime-${t}`;
    for (; r; ) {
      const c = r.ec;
      if (c) {
        for (let p = 0; p < c.length; p++) if (c[p](e, l, d) === false) return;
      }
      r = r.parent;
    }
    if (i) {
      xn(), ts(i, null, 10, [e, l, d]), $n();
      return;
    }
  }
  vu(e, t, a, s, o);
}
function vu(e, n, t, s = true, a = false) {
  if (a) throw e;
  console.error(e);
}
const Me = [];
let fn = -1;
const dt = [];
let Fn = null, ot = 0;
const mr = Promise.resolve();
let bs = null;
function ss(e) {
  const n = bs || mr;
  return e ? n.then(this ? e.bind(this) : e) : n;
}
function yu(e) {
  let n = fn + 1, t = Me.length;
  for (; n < t; ) {
    const s = n + t >>> 1, a = Me[s], i = Yt(a);
    i < e || i === e && a.flags & 2 ? n = s + 1 : t = s;
  }
  return n;
}
function Xa(e) {
  if (!(e.flags & 1)) {
    const n = Yt(e), t = Me[Me.length - 1];
    !t || !(e.flags & 2) && n >= Yt(t) ? Me.push(e) : Me.splice(yu(n), 0, e), e.flags |= 1, gr();
  }
}
function gr() {
  bs || (bs = mr.then(vr));
}
function ku(e) {
  Q(e) ? dt.push(...e) : Fn && e.id === -1 ? Fn.splice(ot + 1, 0, e) : e.flags & 1 || (dt.push(e), e.flags |= 1), gr();
}
function _i(e, n, t = fn + 1) {
  for (; t < Me.length; t++) {
    const s = Me[t];
    if (s && s.flags & 2) {
      if (e && s.id !== e.uid) continue;
      Me.splice(t, 1), t--, s.flags & 4 && (s.flags &= -2), s(), s.flags & 4 || (s.flags &= -2);
    }
  }
}
function _r(e) {
  if (dt.length) {
    const n = [...new Set(dt)].sort((t, s) => Yt(t) - Yt(s));
    if (dt.length = 0, Fn) {
      Fn.push(...n);
      return;
    }
    for (Fn = n, ot = 0; ot < Fn.length; ot++) {
      const t = Fn[ot];
      t.flags & 4 && (t.flags &= -2), t.flags & 8 || t(), t.flags &= -2;
    }
    Fn = null, ot = 0;
  }
}
const Yt = (e) => e.id == null ? e.flags & 2 ? -1 : 1 / 0 : e.id;
function vr(e) {
  try {
    for (fn = 0; fn < Me.length; fn++) {
      const n = Me[fn];
      n && !(n.flags & 8) && (n.flags & 4 && (n.flags &= -2), ts(n, n.i, n.i ? 15 : 14), n.flags & 4 || (n.flags &= -2));
    }
  } finally {
    for (; fn < Me.length; fn++) {
      const n = Me[fn];
      n && (n.flags &= -2);
    }
    fn = -1, Me.length = 0, _r(), bs = null, (Me.length || dt.length) && vr();
  }
}
let Be = null, yr = null;
function xs(e) {
  const n = Be;
  return Be = e, yr = e && e.type.__scopeId || null, n;
}
function Ln(e, n = Be, t) {
  if (!n || e._n) return e;
  const s = (...a) => {
    s._d && Ss(-1);
    const i = xs(n);
    let o;
    try {
      o = e(...a);
    } finally {
      xs(i), s._d && Ss(1);
    }
    return o;
  };
  return s._n = true, s._c = true, s._d = true, s;
}
function ye(e, n) {
  if (Be === null) return e;
  const t = Ws(Be), s = e.dirs || (e.dirs = []);
  for (let a = 0; a < n.length; a++) {
    let [i, o, r, l = me] = n[a];
    i && (Z(i) && (i = { mounted: i, updated: i }), i.deep && Pn(o), s.push({ dir: i, instance: t, value: o, oldValue: void 0, arg: r, modifiers: l }));
  }
  return e;
}
function Yn(e, n, t, s) {
  const a = e.dirs, i = n && n.dirs;
  for (let o = 0; o < a.length; o++) {
    const r = a[o];
    i && (r.oldValue = i[o].value);
    let l = r.dir[s];
    l && (xn(), on(l, t, 8, [e.el, r, e, n]), $n());
  }
}
function _s(e, n) {
  if (Pe) {
    let t = Pe.provides;
    const s = Pe.parent && Pe.parent.provides;
    s === t && (t = Pe.provides = Object.create(s)), t[e] = n;
  }
}
function sn(e, n, t = false) {
  const s = kc();
  if (s || pt) {
    let a = pt ? pt._context.provides : s ? s.parent == null || s.ce ? s.vnode.appContext && s.vnode.appContext.provides : s.parent.provides : void 0;
    if (a && e in a) return a[e];
    if (arguments.length > 1) return t && Z(n) ? n.call(s && s.proxy) : n;
  }
}
const wu = Symbol.for("v-scx"), bu = () => sn(wu);
function kn(e, n, t) {
  return kr(e, n, t);
}
function kr(e, n, t = me) {
  const { immediate: s, deep: a, flush: i, once: o } = t, r = ze({}, t), l = n && s || !n && i !== "post";
  let d;
  if (Xt) {
    if (i === "sync") {
      const f = bu();
      d = f.__watcherHandles || (f.__watcherHandles = []);
    } else if (!l) {
      const f = () => {
      };
      return f.stop = yn, f.resume = yn, f.pause = yn, f;
    }
  }
  const c = Pe;
  r.call = (f, b, P) => on(f, c, b, P);
  let p = false;
  i === "post" ? r.scheduler = (f) => {
    Ve(f, c && c.suspense);
  } : i !== "sync" && (p = true, r.scheduler = (f, b) => {
    b ? f() : Xa(f);
  }), r.augmentJob = (f) => {
    n && (f.flags |= 4), p && (f.flags |= 2, c && (f.id = c.uid, f.i = c));
  };
  const m = _u(e, n, r);
  return Xt && (d ? d.push(m) : l && m()), m;
}
function xu(e, n, t) {
  const s = this.proxy, a = be(e) ? e.includes(".") ? wr(s, e) : () => s[e] : e.bind(s, s);
  let i;
  Z(n) ? i = n : (i = n.handler, t = n);
  const o = is(this), r = kr(a, i.bind(s), t);
  return o(), r;
}
function wr(e, n) {
  const t = n.split(".");
  return () => {
    let s = e;
    for (let a = 0; a < t.length && s; a++) s = s[t[a]];
    return s;
  };
}
const $u = Symbol("_vte"), ju = (e) => e.__isTeleport, ta = Symbol("_leaveCb");
function Za(e, n) {
  e.shapeFlag & 6 && e.component ? (e.transition = n, Za(e.component.subTree, n)) : e.shapeFlag & 128 ? (e.ssContent.transition = n.clone(e.ssContent), e.ssFallback.transition = n.clone(e.ssFallback)) : e.transition = n;
}
function De(e, n) {
  return Z(e) ? ze({ name: e.name }, n, { setup: e }) : e;
}
function br(e) {
  e.ids = [e.ids[0] + e.ids[2]++ + "-", 0, 0];
}
function vi(e, n) {
  let t;
  return !!((t = Object.getOwnPropertyDescriptor(e, n)) && !t.configurable);
}
const $s = /* @__PURE__ */ new WeakMap();
function Ft(e, n, t, s, a = false) {
  if (Q(e)) {
    e.forEach((P, O) => Ft(P, n && (Q(n) ? n[O] : n), t, s, a));
    return;
  }
  if (Vt(s) && !a) {
    s.shapeFlag & 512 && s.type.__asyncResolved && s.component.subTree.component && Ft(e, n, t, s.component.subTree);
    return;
  }
  const i = s.shapeFlag & 4 ? Ws(s.component) : s.el, o = a ? null : i, { i: r, r: l } = e, d = n && n.r, c = r.refs === me ? r.refs = {} : r.refs, p = r.setupState, m = ce(p), f = p === me ? Ho : (P) => vi(c, P) ? false : de(m, P), b = (P, O) => !(O && vi(c, O));
  if (d != null && d !== l) {
    if (yi(n), be(d)) c[d] = null, f(d) && (p[d] = null);
    else if (Te(d)) {
      const P = n;
      b(d, P.k) && (d.value = null), P.k && (c[P.k] = null);
    }
  }
  if (Z(l)) {
    xn();
    try {
      ts(l, r, 12, [o, c]);
    } finally {
      $n();
    }
  } else {
    const P = be(l), O = Te(l);
    if (P || O) {
      const w = () => {
        if (e.f) {
          const _ = P ? f(l) ? p[l] : c[l] : b() || !e.k ? l.value : c[e.k];
          if (a) Q(_) && Ua(_, i);
          else if (Q(_)) _.includes(i) || _.push(i);
          else if (P) c[l] = [i], f(l) && (p[l] = c[l]);
          else {
            const A = [i];
            b(l, e.k) && (l.value = A), e.k && (c[e.k] = A);
          }
        } else P ? (c[l] = o, f(l) && (p[l] = o)) : O && (b(l, e.k) && (l.value = o), e.k && (c[e.k] = o));
      };
      if (o) {
        const _ = () => {
          w(), $s.delete(e);
        };
        _.id = -1, $s.set(e, _), Ve(_, t);
      } else yi(e), w();
    }
  }
}
function yi(e) {
  const n = $s.get(e);
  n && (n.flags |= 8, $s.delete(e));
}
Ds().requestIdleCallback;
Ds().cancelIdleCallback;
const Vt = (e) => !!e.type.__asyncLoader, xr = (e) => e.type.__isKeepAlive;
function Su(e, n) {
  $r(e, "a", n);
}
function Eu(e, n) {
  $r(e, "da", n);
}
function $r(e, n, t = Pe) {
  const s = e.__wdc || (e.__wdc = () => {
    let a = t;
    for (; a; ) {
      if (a.isDeactivated) return;
      a = a.parent;
    }
    return e();
  });
  if (Fs(n, s, t), t) {
    let a = t.parent;
    for (; a && a.parent; ) xr(a.parent.vnode) && Au(s, n, t, a), a = a.parent;
  }
}
function Au(e, n, t, s) {
  const a = Fs(n, e, s, true);
  as(() => {
    Ua(s[n], a);
  }, t);
}
function Fs(e, n, t = Pe, s = false) {
  if (t) {
    const a = t[e] || (t[e] = []), i = n.__weh || (n.__weh = (...o) => {
      xn();
      const r = is(t), l = on(n, t, e, o);
      return r(), $n(), l;
    });
    return s ? a.unshift(i) : a.push(i), i;
  }
}
const Nn = (e) => (n, t = Pe) => {
  (!Xt || e === "sp") && Fs(e, (...s) => n(...s), t);
}, Cu = Nn("bm"), $t = Nn("m"), Ru = Nn("bu"), zu = Nn("u"), Iu = Nn("bum"), as = Nn("um"), Pu = Nn("sp"), Tu = Nn("rtg"), Ou = Nn("rtc");
function Nu(e, n = Pe) {
  Fs("ec", e, n);
}
const qu = "directives", Mu = Symbol.for("v-ndc");
function Du(e) {
  return Lu(qu, e);
}
function Lu(e, n, t = true, s = false) {
  const a = Be || Pe;
  if (a) {
    const i = a.type, o = ki(a[e] || i[e], n) || ki(a.appContext[e], n);
    return !o && s ? i : o;
  }
}
function ki(e, n) {
  return e && (e[n] || e[Ge(n)] || e[Va(Ge(n))]);
}
function ve(e, n, t, s) {
  let a;
  const i = t, o = Q(e);
  if (o || be(e)) {
    const r = o && nt(e);
    let l = false, d = false;
    r && (l = !Ze(e), d = On(e), e = Ls(e)), a = new Array(e.length);
    for (let c = 0, p = e.length; c < p; c++) a[c] = n(l ? d ? ft(an(e[c])) : an(e[c]) : e[c], c, void 0, i);
  } else if (typeof e == "number") {
    a = new Array(e);
    for (let r = 0; r < e; r++) a[r] = n(r + 1, r, void 0, i);
  } else if (pe(e)) if (e[Symbol.iterator]) a = Array.from(e, (r, l) => n(r, l, void 0, i));
  else {
    const r = Object.keys(e);
    a = new Array(r.length);
    for (let l = 0, d = r.length; l < d; l++) {
      const c = r[l];
      a[l] = n(e[c], c, l, i);
    }
  }
  else a = [];
  return a;
}
const ya = (e) => e ? Wr(e) ? Ws(e) : ya(e.parent) : null, Ht = ze(/* @__PURE__ */ Object.create(null), { $: (e) => e, $el: (e) => e.vnode.el, $data: (e) => e.data, $props: (e) => e.props, $attrs: (e) => e.attrs, $slots: (e) => e.slots, $refs: (e) => e.refs, $parent: (e) => ya(e.parent), $root: (e) => ya(e.root), $host: (e) => e.ce, $emit: (e) => e.emit, $options: (e) => Sr(e), $forceUpdate: (e) => e.f || (e.f = () => {
  Xa(e.update);
}), $nextTick: (e) => e.n || (e.n = ss.bind(e.proxy)), $watch: (e) => xu.bind(e) }), sa = (e, n) => e !== me && !e.__isScriptSetup && de(e, n), Uu = { get({ _: e }, n) {
  if (n === "__v_skip") return true;
  const { ctx: t, setupState: s, data: a, props: i, accessCache: o, type: r, appContext: l } = e;
  if (n[0] !== "$") {
    const m = o[n];
    if (m !== void 0) switch (m) {
      case 1:
        return s[n];
      case 2:
        return a[n];
      case 4:
        return t[n];
      case 3:
        return i[n];
    }
    else {
      if (sa(s, n)) return o[n] = 1, s[n];
      if (a !== me && de(a, n)) return o[n] = 2, a[n];
      if (de(i, n)) return o[n] = 3, i[n];
      if (t !== me && de(t, n)) return o[n] = 4, t[n];
      ka && (o[n] = 0);
    }
  }
  const d = Ht[n];
  let c, p;
  if (d) return n === "$attrs" && Ie(e.attrs, "get", ""), d(e);
  if ((c = r.__cssModules) && (c = c[n])) return c;
  if (t !== me && de(t, n)) return o[n] = 4, t[n];
  if (p = l.config.globalProperties, de(p, n)) return p[n];
}, set({ _: e }, n, t) {
  const { data: s, setupState: a, ctx: i } = e;
  return sa(a, n) ? (a[n] = t, true) : s !== me && de(s, n) ? (s[n] = t, true) : de(e.props, n) || n[0] === "$" && n.slice(1) in e ? false : (i[n] = t, true);
}, has({ _: { data: e, setupState: n, accessCache: t, ctx: s, appContext: a, props: i, type: o } }, r) {
  let l;
  return !!(t[r] || e !== me && r[0] !== "$" && de(e, r) || sa(n, r) || de(i, r) || de(s, r) || de(Ht, r) || de(a.config.globalProperties, r) || (l = o.__cssModules) && l[r]);
}, defineProperty(e, n, t) {
  return t.get != null ? e._.accessCache[n] = 0 : de(t, "value") && this.set(e, n, t.value, null), Reflect.defineProperty(e, n, t);
} };
function wi(e) {
  return Q(e) ? e.reduce((n, t) => (n[t] = null, n), {}) : e;
}
let ka = true;
function Fu(e) {
  const n = Sr(e), t = e.proxy, s = e.ctx;
  ka = false, n.beforeCreate && bi(n.beforeCreate, e, "bc");
  const { data: a, computed: i, methods: o, watch: r, provide: l, inject: d, created: c, beforeMount: p, mounted: m, beforeUpdate: f, updated: b, activated: P, deactivated: O, beforeDestroy: w, beforeUnmount: _, destroyed: A, unmounted: T, render: H, renderTracked: le, renderTriggered: ne, errorCaptured: $e, serverPrefetch: Ee, expose: Se, inheritAttrs: Ye, components: He, directives: Le, filters: en } = n;
  if (d && Vu(d, s, null), o) for (const oe in o) {
    const ae = o[oe];
    Z(ae) && (s[oe] = ae.bind(t));
  }
  if (a) {
    const oe = a.call(t, t);
    pe(oe) && (e.data = xt(oe));
  }
  if (ka = true, i) for (const oe in i) {
    const ae = i[oe], Oe = Z(ae) ? ae.bind(t, t) : Z(ae.get) ? ae.get.bind(t, t) : yn, N = !Z(ae) && Z(ae.set) ? ae.set.bind(t) : yn, U = Y({ get: Oe, set: N });
    Object.defineProperty(s, oe, { enumerable: true, configurable: true, get: () => U.value, set: (K) => U.value = K });
  }
  if (r) for (const oe in r) jr(r[oe], s, t, oe);
  if (l) {
    const oe = Z(l) ? l.call(t) : l;
    Reflect.ownKeys(oe).forEach((ae) => {
      _s(ae, oe[ae]);
    });
  }
  c && bi(c, e, "c");
  function ke(oe, ae) {
    Q(ae) ? ae.forEach((Oe) => oe(Oe.bind(t))) : ae && oe(ae.bind(t));
  }
  if (ke(Cu, p), ke($t, m), ke(Ru, f), ke(zu, b), ke(Su, P), ke(Eu, O), ke(Nu, $e), ke(Ou, le), ke(Tu, ne), ke(Iu, _), ke(as, T), ke(Pu, Ee), Q(Se)) if (Se.length) {
    const oe = e.exposed || (e.exposed = {});
    Se.forEach((ae) => {
      Object.defineProperty(oe, ae, { get: () => t[ae], set: (Oe) => t[ae] = Oe, enumerable: true });
    });
  } else e.exposed || (e.exposed = {});
  H && e.render === yn && (e.render = H), Ye != null && (e.inheritAttrs = Ye), He && (e.components = He), Le && (e.directives = Le), Ee && br(e);
}
function Vu(e, n, t = yn) {
  Q(e) && (e = wa(e));
  for (const s in e) {
    const a = e[s];
    let i;
    pe(a) ? "default" in a ? i = sn(a.from || s, a.default, true) : i = sn(a.from || s) : i = sn(a), Te(i) ? Object.defineProperty(n, s, { enumerable: true, configurable: true, get: () => i.value, set: (o) => i.value = o }) : n[s] = i;
  }
}
function bi(e, n, t) {
  on(Q(e) ? e.map((s) => s.bind(n.proxy)) : e.bind(n.proxy), n, t);
}
function jr(e, n, t, s) {
  let a = s.includes(".") ? wr(t, s) : () => t[s];
  if (be(e)) {
    const i = n[e];
    Z(i) && kn(a, i);
  } else if (Z(e)) kn(a, e.bind(t));
  else if (pe(e)) if (Q(e)) e.forEach((i) => jr(i, n, t, s));
  else {
    const i = Z(e.handler) ? e.handler.bind(t) : n[e.handler];
    Z(i) && kn(a, i, e);
  }
}
function Sr(e) {
  const n = e.type, { mixins: t, extends: s } = n, { mixins: a, optionsCache: i, config: { optionMergeStrategies: o } } = e.appContext, r = i.get(n);
  let l;
  return r ? l = r : !a.length && !t && !s ? l = n : (l = {}, a.length && a.forEach((d) => js(l, d, o, true)), js(l, n, o)), pe(n) && i.set(n, l), l;
}
function js(e, n, t, s = false) {
  const { mixins: a, extends: i } = n;
  i && js(e, i, t, true), a && a.forEach((o) => js(e, o, t, true));
  for (const o in n) if (!(s && o === "expose")) {
    const r = Hu[o] || t && t[o];
    e[o] = r ? r(e[o], n[o]) : n[o];
  }
  return e;
}
const Hu = { data: xi, props: $i, emits: $i, methods: Pt, computed: Pt, beforeCreate: Ne, created: Ne, beforeMount: Ne, mounted: Ne, beforeUpdate: Ne, updated: Ne, beforeDestroy: Ne, beforeUnmount: Ne, destroyed: Ne, unmounted: Ne, activated: Ne, deactivated: Ne, errorCaptured: Ne, serverPrefetch: Ne, components: Pt, directives: Pt, watch: Bu, provide: xi, inject: Wu };
function xi(e, n) {
  return n ? e ? function() {
    return ze(Z(e) ? e.call(this, this) : e, Z(n) ? n.call(this, this) : n);
  } : n : e;
}
function Wu(e, n) {
  return Pt(wa(e), wa(n));
}
function wa(e) {
  if (Q(e)) {
    const n = {};
    for (let t = 0; t < e.length; t++) n[e[t]] = e[t];
    return n;
  }
  return e;
}
function Ne(e, n) {
  return e ? [...new Set([].concat(e, n))] : n;
}
function Pt(e, n) {
  return e ? ze(/* @__PURE__ */ Object.create(null), e, n) : n;
}
function $i(e, n) {
  return e ? Q(e) && Q(n) ? [.../* @__PURE__ */ new Set([...e, ...n])] : ze(/* @__PURE__ */ Object.create(null), wi(e), wi(n ?? {})) : n;
}
function Bu(e, n) {
  if (!e) return n;
  if (!n) return e;
  const t = ze(/* @__PURE__ */ Object.create(null), e);
  for (const s in n) t[s] = Ne(e[s], n[s]);
  return t;
}
function Er() {
  return { app: null, config: { isNativeTag: Ho, performance: false, globalProperties: {}, optionMergeStrategies: {}, errorHandler: void 0, warnHandler: void 0, compilerOptions: {} }, mixins: [], components: {}, directives: {}, provides: /* @__PURE__ */ Object.create(null), optionsCache: /* @__PURE__ */ new WeakMap(), propsCache: /* @__PURE__ */ new WeakMap(), emitsCache: /* @__PURE__ */ new WeakMap() };
}
let Ku = 0;
function Gu(e, n) {
  return function(s, a = null) {
    Z(s) || (s = ze({}, s)), a != null && !pe(a) && (a = null);
    const i = Er(), o = /* @__PURE__ */ new WeakSet(), r = [];
    let l = false;
    const d = i.app = { _uid: Ku++, _component: s, _props: a, _container: null, _context: i, _instance: null, version: Sc, get config() {
      return i.config;
    }, set config(c) {
    }, use(c, ...p) {
      return o.has(c) || (c && Z(c.install) ? (o.add(c), c.install(d, ...p)) : Z(c) && (o.add(c), c(d, ...p))), d;
    }, mixin(c) {
      return i.mixins.includes(c) || i.mixins.push(c), d;
    }, component(c, p) {
      return p ? (i.components[c] = p, d) : i.components[c];
    }, directive(c, p) {
      return p ? (i.directives[c] = p, d) : i.directives[c];
    }, mount(c, p, m) {
      if (!l) {
        const f = d._ceVNode || _e(s, a);
        return f.appContext = i, m === true ? m = "svg" : m === false && (m = void 0), e(f, c, m), l = true, d._container = c, c.__vue_app__ = d, Ws(f.component);
      }
    }, onUnmount(c) {
      r.push(c);
    }, unmount() {
      l && (on(r, d._instance, 16), e(null, d._container), delete d._container.__vue_app__);
    }, provide(c, p) {
      return i.provides[c] = p, d;
    }, runWithContext(c) {
      const p = pt;
      pt = d;
      try {
        return c();
      } finally {
        pt = p;
      }
    } };
    return d;
  };
}
let pt = null;
const Ju = (e, n) => n === "modelValue" || n === "model-value" ? e.modelModifiers : e[`${n}Modifiers`] || e[`${Ge(n)}Modifiers`] || e[`${Jn(n)}Modifiers`];
function Yu(e, n, ...t) {
  if (e.isUnmounted) return;
  const s = e.vnode.props || me;
  let a = t;
  const i = n.startsWith("update:"), o = i && Ju(s, n.slice(7));
  o && (o.trim && (a = t.map((c) => be(c) ? c.trim() : c)), o.number && (a = t.map(Ms)));
  let r, l = s[r = Qs(n)] || s[r = Qs(Ge(n))];
  !l && i && (l = s[r = Qs(Jn(n))]), l && on(l, e, 6, a);
  const d = s[r + "Once"];
  if (d) {
    if (!e.emitted) e.emitted = {};
    else if (e.emitted[r]) return;
    e.emitted[r] = true, on(d, e, 6, a);
  }
}
const Qu = /* @__PURE__ */ new WeakMap();
function Ar(e, n, t = false) {
  const s = t ? Qu : n.emitsCache, a = s.get(e);
  if (a !== void 0) return a;
  const i = e.emits;
  let o = {}, r = false;
  if (!Z(e)) {
    const l = (d) => {
      const c = Ar(d, n, true);
      c && (r = true, ze(o, c));
    };
    !t && n.mixins.length && n.mixins.forEach(l), e.extends && l(e.extends), e.mixins && e.mixins.forEach(l);
  }
  return !i && !r ? (pe(e) && s.set(e, null), null) : (Q(i) ? i.forEach((l) => o[l] = null) : ze(o, i), pe(e) && s.set(e, o), o);
}
function Vs(e, n) {
  return !e || !Os(n) ? false : (n = n.slice(2), n = n === "Once" ? n : n.replace(/Once$/, ""), de(e, n[0].toLowerCase() + n.slice(1)) || de(e, Jn(n)) || de(e, n));
}
function ji(e) {
  const { type: n, vnode: t, proxy: s, withProxy: a, propsOptions: [i], slots: o, attrs: r, emit: l, render: d, renderCache: c, props: p, data: m, setupState: f, ctx: b, inheritAttrs: P } = e, O = xs(e);
  let w, _;
  try {
    if (t.shapeFlag & 4) {
      const T = a || s, H = T;
      w = gn(d.call(H, T, c, p, f, m, b)), _ = r;
    } else {
      const T = n;
      w = gn(T.length > 1 ? T(p, { attrs: r, slots: o, emit: l }) : T(p, null)), _ = n.props ? r : Xu(r);
    }
  } catch (T) {
    Wt.length = 0, Us(T, e, 1), w = _e(Gn);
  }
  let A = w;
  if (_ && P !== false) {
    const T = Object.keys(_), { shapeFlag: H } = A;
    T.length && H & 7 && (i && T.some(Ns) && (_ = Zu(_, i)), A = mt(A, _, false, true));
  }
  return t.dirs && (A = mt(A, null, false, true), A.dirs = A.dirs ? A.dirs.concat(t.dirs) : t.dirs), t.transition && Za(A, t.transition), w = A, xs(O), w;
}
const Xu = (e) => {
  let n;
  for (const t in e) (t === "class" || t === "style" || Os(t)) && ((n || (n = {}))[t] = e[t]);
  return n;
}, Zu = (e, n) => {
  const t = {};
  for (const s in e) (!Ns(s) || !(s.slice(9) in n)) && (t[s] = e[s]);
  return t;
};
function ec(e, n, t) {
  const { props: s, children: a, component: i } = e, { props: o, children: r, patchFlag: l } = n, d = i.emitsOptions;
  if (n.dirs || n.transition) return true;
  if (t && l >= 0) {
    if (l & 1024) return true;
    if (l & 16) return s ? Si(s, o, d) : !!o;
    if (l & 8) {
      const c = n.dynamicProps;
      for (let p = 0; p < c.length; p++) {
        const m = c[p];
        if (Cr(o, s, m) && !Vs(d, m)) return true;
      }
    }
  } else return (a || r) && (!r || !r.$stable) ? true : s === o ? false : s ? o ? Si(s, o, d) : true : !!o;
  return false;
}
function Si(e, n, t) {
  const s = Object.keys(n);
  if (s.length !== Object.keys(e).length) return true;
  for (let a = 0; a < s.length; a++) {
    const i = s[a];
    if (Cr(n, e, i) && !Vs(t, i)) return true;
  }
  return false;
}
function Cr(e, n, t) {
  const s = e[t], a = n[t];
  return t === "style" && pe(s) && pe(a) ? !bt(s, a) : s !== a;
}
function nc({ vnode: e, parent: n, suspense: t }, s) {
  for (; n; ) {
    const a = n.subTree;
    if (a.suspense && a.suspense.activeBranch === e && (a.suspense.vnode.el = a.el = s, e = a), a === e) (e = n.vnode).el = s, n = n.parent;
    else break;
  }
  t && t.activeBranch === e && (t.vnode.el = s);
}
const Rr = {}, zr = () => Object.create(Rr), Ir = (e) => Object.getPrototypeOf(e) === Rr;
function tc(e, n, t, s = false) {
  const a = {}, i = zr();
  e.propsDefaults = /* @__PURE__ */ Object.create(null), Pr(e, n, a, i);
  for (const o in e.propsOptions[0]) o in a || (a[o] = void 0);
  t ? e.props = s ? a : pr(a) : e.type.props ? e.props = a : e.props = i, e.attrs = i;
}
function sc(e, n, t, s) {
  const { props: a, attrs: i, vnode: { patchFlag: o } } = e, r = ce(a), [l] = e.propsOptions;
  let d = false;
  if ((s || o > 0) && !(o & 16)) {
    if (o & 8) {
      const c = e.vnode.dynamicProps;
      for (let p = 0; p < c.length; p++) {
        let m = c[p];
        if (Vs(e.emitsOptions, m)) continue;
        const f = n[m];
        if (l) if (de(i, m)) f !== i[m] && (i[m] = f, d = true);
        else {
          const b = Ge(m);
          a[b] = ba(l, r, b, f, e, false);
        }
        else f !== i[m] && (i[m] = f, d = true);
      }
    }
  } else {
    Pr(e, n, a, i) && (d = true);
    let c;
    for (const p in r) (!n || !de(n, p) && ((c = Jn(p)) === p || !de(n, c))) && (l ? t && (t[p] !== void 0 || t[c] !== void 0) && (a[p] = ba(l, r, p, void 0, e, true)) : delete a[p]);
    if (i !== r) for (const p in i) (!n || !de(n, p)) && (delete i[p], d = true);
  }
  d && In(e.attrs, "set", "");
}
function Pr(e, n, t, s) {
  const [a, i] = e.propsOptions;
  let o = false, r;
  if (n) for (let l in n) {
    if (Dt(l)) continue;
    const d = n[l];
    let c;
    a && de(a, c = Ge(l)) ? !i || !i.includes(c) ? t[c] = d : (r || (r = {}))[c] = d : Vs(e.emitsOptions, l) || (!(l in s) || d !== s[l]) && (s[l] = d, o = true);
  }
  if (i) {
    const l = ce(t), d = r || me;
    for (let c = 0; c < i.length; c++) {
      const p = i[c];
      t[p] = ba(a, l, p, d[p], e, !de(d, p));
    }
  }
  return o;
}
function ba(e, n, t, s, a, i) {
  const o = e[t];
  if (o != null) {
    const r = de(o, "default");
    if (r && s === void 0) {
      const l = o.default;
      if (o.type !== Function && !o.skipFactory && Z(l)) {
        const { propsDefaults: d } = a;
        if (t in d) s = d[t];
        else {
          const c = is(a);
          s = d[t] = l.call(null, n), c();
        }
      } else s = l;
      a.ce && a.ce._setProp(t, s);
    }
    o[0] && (i && !r ? s = false : o[1] && (s === "" || s === Jn(t)) && (s = true));
  }
  return s;
}
const ac = /* @__PURE__ */ new WeakMap();
function Tr(e, n, t = false) {
  const s = t ? ac : n.propsCache, a = s.get(e);
  if (a) return a;
  const i = e.props, o = {}, r = [];
  let l = false;
  if (!Z(e)) {
    const c = (p) => {
      l = true;
      const [m, f] = Tr(p, n, true);
      ze(o, m), f && r.push(...f);
    };
    !t && n.mixins.length && n.mixins.forEach(c), e.extends && c(e.extends), e.mixins && e.mixins.forEach(c);
  }
  if (!i && !l) return pe(e) && s.set(e, ut), ut;
  if (Q(i)) for (let c = 0; c < i.length; c++) {
    const p = Ge(i[c]);
    Ei(p) && (o[p] = me);
  }
  else if (i) for (const c in i) {
    const p = Ge(c);
    if (Ei(p)) {
      const m = i[c], f = o[p] = Q(m) || Z(m) ? { type: m } : ze({}, m), b = f.type;
      let P = false, O = true;
      if (Q(b)) for (let w = 0; w < b.length; ++w) {
        const _ = b[w], A = Z(_) && _.name;
        if (A === "Boolean") {
          P = true;
          break;
        } else A === "String" && (O = false);
      }
      else P = Z(b) && b.name === "Boolean";
      f[0] = P, f[1] = O, (P || de(f, "default")) && r.push(p);
    }
  }
  const d = [o, r];
  return pe(e) && s.set(e, d), d;
}
function Ei(e) {
  return e[0] !== "$" && !Dt(e);
}
const ei = (e) => e === "_" || e === "_ctx" || e === "$stable", ni = (e) => Q(e) ? e.map(gn) : [gn(e)], ic = (e, n, t) => {
  if (n._n) return n;
  const s = Ln((...a) => ni(n(...a)), t);
  return s._c = false, s;
}, Or = (e, n, t) => {
  const s = e._ctx;
  for (const a in e) {
    if (ei(a)) continue;
    const i = e[a];
    if (Z(i)) n[a] = ic(a, i, s);
    else if (i != null) {
      const o = ni(i);
      n[a] = () => o;
    }
  }
}, Nr = (e, n) => {
  const t = ni(n);
  e.slots.default = () => t;
}, qr = (e, n, t) => {
  for (const s in n) (t || !ei(s)) && (e[s] = n[s]);
}, oc = (e, n, t) => {
  const s = e.slots = zr();
  if (e.vnode.shapeFlag & 32) {
    const a = n._;
    a ? (qr(s, n, t), t && Go(s, "_", a, true)) : Or(n, s);
  } else n && Nr(e, n);
}, rc = (e, n, t) => {
  const { vnode: s, slots: a } = e;
  let i = true, o = me;
  if (s.shapeFlag & 32) {
    const r = n._;
    r ? t && r === 1 ? i = false : qr(a, n, t) : (i = !n.$stable, Or(n, a)), o = n;
  } else n && (Nr(e, n), o = { default: 1 });
  if (i) for (const r in a) !ei(r) && o[r] == null && delete a[r];
}, Ve = pc;
function lc(e) {
  return uc(e);
}
function uc(e, n) {
  const t = Ds();
  t.__VUE__ = true;
  const { insert: s, remove: a, patchProp: i, createElement: o, createText: r, createComment: l, setText: d, setElementText: c, parentNode: p, nextSibling: m, setScopeId: f = yn, insertStaticContent: b } = e, P = (h, g, k, R = null, I = null, C = null, q = void 0, v = null, y = !!g.dynamicChildren) => {
    if (h === g) return;
    h && !At(h, g) && (R = z(h), K(h, I, C, true), h = null), g.patchFlag === -2 && (y = false, g.dynamicChildren = null);
    const { type: E, ref: S, shapeFlag: M } = g;
    switch (E) {
      case Hs:
        O(h, g, k, R);
        break;
      case Gn:
        w(h, g, k, R);
        break;
      case vs:
        h == null && _(g, k, R, q);
        break;
      case ee:
        He(h, g, k, R, I, C, q, v, y);
        break;
      default:
        M & 1 ? H(h, g, k, R, I, C, q, v, y) : M & 6 ? Le(h, g, k, R, I, C, q, v, y) : (M & 64 || M & 128) && E.process(h, g, k, R, I, C, q, v, y, B);
    }
    S != null && I ? Ft(S, h && h.ref, C, g || h, !g) : S == null && h && h.ref != null && Ft(h.ref, null, C, h, true);
  }, O = (h, g, k, R) => {
    if (h == null) s(g.el = r(g.children), k, R);
    else {
      const I = g.el = h.el;
      g.children !== h.children && d(I, g.children);
    }
  }, w = (h, g, k, R) => {
    h == null ? s(g.el = l(g.children || ""), k, R) : g.el = h.el;
  }, _ = (h, g, k, R) => {
    [h.el, h.anchor] = b(h.children, g, k, R, h.el, h.anchor);
  }, A = ({ el: h, anchor: g }, k, R) => {
    let I;
    for (; h && h !== g; ) I = m(h), s(h, k, R), h = I;
    s(g, k, R);
  }, T = ({ el: h, anchor: g }) => {
    let k;
    for (; h && h !== g; ) k = m(h), a(h), h = k;
    a(g);
  }, H = (h, g, k, R, I, C, q, v, y) => {
    if (g.type === "svg" ? q = "svg" : g.type === "math" && (q = "mathml"), h == null) le(g, k, R, I, C, q, v, y);
    else {
      const E = h.el && h.el._isVueCE ? h.el : null;
      try {
        E && E._beginPatch(), Ee(h, g, I, C, q, v, y);
      } finally {
        E && E._endPatch();
      }
    }
  }, le = (h, g, k, R, I, C, q, v) => {
    let y, E;
    const { props: S, shapeFlag: M, transition: G, dirs: X } = h;
    if (y = h.el = o(h.type, C, S && S.is, S), M & 8 ? c(y, h.children) : M & 16 && $e(h.children, y, null, R, I, aa(h, C), q, v), X && Yn(h, null, R, "created"), ne(y, h, h.scopeId, q, R), S) {
      for (const fe in S) fe !== "value" && !Dt(fe) && i(y, fe, null, S[fe], C, R);
      "value" in S && i(y, "value", null, S.value, C), (E = S.onVnodeBeforeMount) && hn(E, R, h);
    }
    X && Yn(h, null, R, "beforeMount");
    const re = cc(I, G);
    re && G.beforeEnter(y), s(y, g, k), ((E = S && S.onVnodeMounted) || re || X) && Ve(() => {
      try {
        E && hn(E, R, h), re && G.enter(y), X && Yn(h, null, R, "mounted");
      } finally {
      }
    }, I);
  }, ne = (h, g, k, R, I) => {
    if (k && f(h, k), R) for (let C = 0; C < R.length; C++) f(h, R[C]);
    if (I) {
      let C = I.subTree;
      if (g === C || Ur(C.type) && (C.ssContent === g || C.ssFallback === g)) {
        const q = I.vnode;
        ne(h, q, q.scopeId, q.slotScopeIds, I.parent);
      }
    }
  }, $e = (h, g, k, R, I, C, q, v, y = 0) => {
    for (let E = y; E < h.length; E++) {
      const S = h[E] = v ? zn(h[E]) : gn(h[E]);
      P(null, S, g, k, R, I, C, q, v);
    }
  }, Ee = (h, g, k, R, I, C, q) => {
    const v = g.el = h.el;
    let { patchFlag: y, dynamicChildren: E, dirs: S } = g;
    y |= h.patchFlag & 16;
    const M = h.props || me, G = g.props || me;
    let X;
    if (k && Qn(k, false), (X = G.onVnodeBeforeUpdate) && hn(X, k, g, h), S && Yn(g, h, k, "beforeUpdate"), k && Qn(k, true), E && (!h.dynamicChildren || h.dynamicChildren.length !== E.length) && (y = 0, q = false, E = null), (M.innerHTML && G.innerHTML == null || M.textContent && G.textContent == null) && c(v, ""), E ? Se(h.dynamicChildren, E, v, k, R, aa(g, I), C) : q || ae(h, g, v, null, k, R, aa(g, I), C, false), y > 0) {
      if (y & 16) Ye(v, M, G, k, I);
      else if (y & 2 && M.class !== G.class && i(v, "class", null, G.class, I), y & 4 && i(v, "style", M.style, G.style, I), y & 8) {
        const re = g.dynamicProps;
        for (let fe = 0; fe < re.length; fe++) {
          const he = re[fe], xe = M[he], Ae = G[he];
          (Ae !== xe || he === "value") && i(v, he, xe, Ae, I, k);
        }
      }
      y & 1 && h.children !== g.children && c(v, g.children);
    } else !q && E == null && Ye(v, M, G, k, I);
    ((X = G.onVnodeUpdated) || S) && Ve(() => {
      X && hn(X, k, g, h), S && Yn(g, h, k, "updated");
    }, R);
  }, Se = (h, g, k, R, I, C, q) => {
    for (let v = 0; v < g.length; v++) {
      const y = h[v], E = g[v], S = y.el && (y.type === ee || !At(y, E) || y.shapeFlag & 198) ? p(y.el) : k;
      P(y, E, S, null, R, I, C, q, true);
    }
  }, Ye = (h, g, k, R, I) => {
    if (g !== k) {
      if (g !== me) for (const C in g) !Dt(C) && !(C in k) && i(h, C, g[C], null, I, R);
      for (const C in k) {
        if (Dt(C)) continue;
        const q = k[C], v = g[C];
        q !== v && C !== "value" && i(h, C, v, q, I, R);
      }
      "value" in k && i(h, "value", g.value, k.value, I);
    }
  }, He = (h, g, k, R, I, C, q, v, y) => {
    const E = g.el = h ? h.el : r(""), S = g.anchor = h ? h.anchor : r("");
    let { patchFlag: M, dynamicChildren: G, slotScopeIds: X } = g;
    X && (v = v ? v.concat(X) : X), h == null ? (s(E, k, R), s(S, k, R), $e(g.children || [], k, S, I, C, q, v, y)) : M > 0 && M & 64 && G && h.dynamicChildren && h.dynamicChildren.length === G.length ? (Se(h.dynamicChildren, G, k, I, C, q, v), (g.key != null || I && g === I.subTree) && Mr(h, g, true)) : ae(h, g, k, S, I, C, q, v, y);
  }, Le = (h, g, k, R, I, C, q, v, y) => {
    g.slotScopeIds = v, h == null ? g.shapeFlag & 512 ? I.ctx.activate(g, k, R, q, y) : en(g, k, R, I, C, q, y) : un(h, g, y);
  }, en = (h, g, k, R, I, C, q) => {
    const v = h.component = yc(h, R, I);
    if (xr(h) && (v.ctx.renderer = B), wc(v, false, q), v.asyncDep) {
      if (I && I.registerDep(v, ke, q), !h.el) {
        const y = v.subTree = _e(Gn);
        w(null, y, g, k), h.placeholder = y.el;
      }
    } else ke(v, h, g, k, I, C, q);
  }, un = (h, g, k) => {
    const R = g.component = h.component;
    if (ec(h, g, k)) if (R.asyncDep && !R.asyncResolved) {
      oe(R, g, k);
      return;
    } else R.next = g, R.update();
    else g.el = h.el, R.vnode = g;
  }, ke = (h, g, k, R, I, C, q) => {
    const v = () => {
      if (h.isMounted) {
        let { next: M, bu: G, u: X, parent: re, vnode: fe } = h;
        {
          const dn = Dr(h);
          if (dn) {
            M && (M.el = fe.el, oe(h, M, q)), dn.asyncDep.then(() => {
              Ve(() => {
                h.isUnmounted || E();
              }, I);
            });
            return;
          }
        }
        let he = M, xe;
        Qn(h, false), M ? (M.el = fe.el, oe(h, M, q)) : M = fe, G && gs(G), (xe = M.props && M.props.onVnodeBeforeUpdate) && hn(xe, re, M, fe), Qn(h, true);
        const Ae = ji(h), cn = h.subTree;
        h.subTree = Ae, P(cn, Ae, p(cn.el), z(cn), h, I, C), M.el = Ae.el, he === null && nc(h, Ae.el), X && Ve(X, I), (xe = M.props && M.props.onVnodeUpdated) && Ve(() => hn(xe, re, M, fe), I);
      } else {
        let M;
        const { el: G, props: X } = g, { bm: re, m: fe, parent: he, root: xe, type: Ae } = h, cn = Vt(g);
        Qn(h, false), re && gs(re), !cn && (M = X && X.onVnodeBeforeMount) && hn(M, he, g), Qn(h, true);
        {
          xe.ce && xe.ce._hasShadowRoot() && xe.ce._injectChildStyle(Ae, h.parent ? h.parent.type : void 0);
          const dn = h.subTree = ji(h);
          P(null, dn, k, R, h, I, C), g.el = dn.el;
        }
        if (fe && Ve(fe, I), !cn && (M = X && X.onVnodeMounted)) {
          const dn = g;
          Ve(() => hn(M, he, dn), I);
        }
        (g.shapeFlag & 256 || he && Vt(he.vnode) && he.vnode.shapeFlag & 256) && h.a && Ve(h.a, I), h.isMounted = true, g = k = R = null;
      }
    };
    h.scope.on();
    const y = h.effect = new Xo(v);
    h.scope.off();
    const E = h.update = y.run.bind(y), S = h.job = y.runIfDirty.bind(y);
    S.i = h, S.id = h.uid, y.scheduler = () => Xa(S), Qn(h, true), E();
  }, oe = (h, g, k) => {
    g.component = h;
    const R = h.vnode.props;
    h.vnode = g, h.next = null, sc(h, g.props, R, k), rc(h, g.children, k), xn(), _i(h), $n();
  }, ae = (h, g, k, R, I, C, q, v, y = false) => {
    const E = h && h.children, S = h ? h.shapeFlag : 0, M = g.children, { patchFlag: G, shapeFlag: X } = g;
    if (G > 0) {
      if (G & 128) {
        N(E, M, k, R, I, C, q, v, y);
        return;
      } else if (G & 256) {
        Oe(E, M, k, R, I, C, q, v, y);
        return;
      }
    }
    X & 8 ? (S & 16 && Fe(E, I, C), M !== E && c(k, M)) : S & 16 ? X & 16 ? N(E, M, k, R, I, C, q, v, y) : Fe(E, I, C, true) : (S & 8 && c(k, ""), X & 16 && $e(M, k, R, I, C, q, v, y));
  }, Oe = (h, g, k, R, I, C, q, v, y) => {
    h = h || ut, g = g || ut;
    const E = h.length, S = g.length, M = Math.min(E, S);
    let G;
    for (G = 0; G < M; G++) {
      const X = g[G] = y ? zn(g[G]) : gn(g[G]);
      P(h[G], X, k, null, I, C, q, v, y);
    }
    E > S ? Fe(h, I, C, true, false, M) : $e(g, k, R, I, C, q, v, y, M);
  }, N = (h, g, k, R, I, C, q, v, y) => {
    let E = 0;
    const S = g.length;
    let M = h.length - 1, G = S - 1;
    for (; E <= M && E <= G; ) {
      const X = h[E], re = g[E] = y ? zn(g[E]) : gn(g[E]);
      if (At(X, re)) P(X, re, k, null, I, C, q, v, y);
      else break;
      E++;
    }
    for (; E <= M && E <= G; ) {
      const X = h[M], re = g[G] = y ? zn(g[G]) : gn(g[G]);
      if (At(X, re)) P(X, re, k, null, I, C, q, v, y);
      else break;
      M--, G--;
    }
    if (E > M) {
      if (E <= G) {
        const X = G + 1, re = X < S ? g[X].el : R;
        for (; E <= G; ) P(null, g[E] = y ? zn(g[E]) : gn(g[E]), k, re, I, C, q, v, y), E++;
      }
    } else if (E > G) for (; E <= M; ) K(h[E], I, C, true), E++;
    else {
      const X = E, re = E, fe = /* @__PURE__ */ new Map();
      for (E = re; E <= G; E++) {
        const We = g[E] = y ? zn(g[E]) : gn(g[E]);
        We.key != null && fe.set(We.key, E);
      }
      let he, xe = 0;
      const Ae = G - re + 1;
      let cn = false, dn = 0;
      const St = new Array(Ae);
      for (E = 0; E < Ae; E++) St[E] = 0;
      for (E = X; E <= M; E++) {
        const We = h[E];
        if (xe >= Ae) {
          K(We, I, C, true);
          continue;
        }
        let pn;
        if (We.key != null) pn = fe.get(We.key);
        else for (he = re; he <= G; he++) if (St[he - re] === 0 && At(We, g[he])) {
          pn = he;
          break;
        }
        pn === void 0 ? K(We, I, C, true) : (St[pn - re] = E + 1, pn >= dn ? dn = pn : cn = true, P(We, g[pn], k, null, I, C, q, v, y), xe++);
      }
      const ci = cn ? dc(St) : ut;
      for (he = ci.length - 1, E = Ae - 1; E >= 0; E--) {
        const We = re + E, pn = g[We], di = g[We + 1], pi = We + 1 < S ? di.el || Lr(di) : R;
        St[E] === 0 ? P(null, pn, k, pi, I, C, q, v, y) : cn && (he < 0 || E !== ci[he] ? U(pn, k, pi, 2) : he--);
      }
    }
  }, U = (h, g, k, R, I = null) => {
    const { el: C, type: q, transition: v, children: y, shapeFlag: E } = h;
    if (E & 6) {
      U(h.component.subTree, g, k, R);
      return;
    }
    if (E & 128) {
      h.suspense.move(g, k, R);
      return;
    }
    if (E & 64) {
      q.move(h, g, k, B);
      return;
    }
    if (q === ee) {
      s(C, g, k);
      for (let M = 0; M < y.length; M++) U(y[M], g, k, R);
      s(h.anchor, g, k);
      return;
    }
    if (q === vs) {
      A(h, g, k);
      return;
    }
    if (R !== 2 && E & 1 && v) if (R === 0) v.persisted && !C[ta] ? s(C, g, k) : (v.beforeEnter(C), s(C, g, k), Ve(() => v.enter(C), I));
    else {
      const { leave: M, delayLeave: G, afterLeave: X } = v, re = () => {
        h.ctx.isUnmounted ? a(C) : s(C, g, k);
      }, fe = () => {
        const he = C._isLeaving || !!C[ta];
        C._isLeaving && C[ta](true), v.persisted && !he ? re() : M(C, () => {
          re(), X && X();
        });
      };
      G ? G(C, re, fe) : fe();
    }
    else s(C, g, k);
  }, K = (h, g, k, R = false, I = false) => {
    const { type: C, props: q, ref: v, children: y, dynamicChildren: E, shapeFlag: S, patchFlag: M, dirs: G, cacheIndex: X, memo: re } = h;
    if (M === -2 && (I = false), v != null && (xn(), Ft(v, null, k, h, true), $n()), X != null && (g.renderCache[X] = void 0), S & 256) {
      g.ctx.deactivate(h);
      return;
    }
    const fe = S & 1 && G, he = !Vt(h);
    let xe;
    if (he && (xe = q && q.onVnodeBeforeUnmount) && hn(xe, g, h), S & 6) Sn(h.component, k, R);
    else {
      if (S & 128) {
        h.suspense.unmount(k, R);
        return;
      }
      fe && Yn(h, null, g, "beforeUnmount"), S & 64 ? h.type.remove(h, g, k, B, R) : E && !E.hasOnce && (C !== ee || M > 0 && M & 64) ? Fe(E, g, k, false, true) : (C === ee && M & 384 || !I && S & 16) && Fe(y, g, k), R && ie(h);
    }
    const Ae = re != null && X == null;
    (he && (xe = q && q.onVnodeUnmounted) || fe || Ae) && Ve(() => {
      xe && hn(xe, g, h), fe && Yn(h, null, g, "unmounted"), Ae && (h.el = null);
    }, k);
  }, ie = (h) => {
    const { type: g, el: k, anchor: R, transition: I } = h;
    if (g === ee) {
      Ue(k, R);
      return;
    }
    if (g === vs) {
      T(h);
      return;
    }
    const C = () => {
      a(k), I && !I.persisted && I.afterLeave && I.afterLeave();
    };
    if (h.shapeFlag & 1 && I && !I.persisted) {
      const { leave: q, delayLeave: v } = I, y = () => q(k, C);
      v ? v(h.el, C, y) : y();
    } else C();
  }, Ue = (h, g) => {
    let k;
    for (; h !== g; ) k = m(h), a(h), h = k;
    a(g);
  }, Sn = (h, g, k) => {
    const { bum: R, scope: I, job: C, subTree: q, um: v, m: y, a: E } = h;
    Ai(y), Ai(E), R && gs(R), I.stop(), C && (C.flags |= 8, K(q, h, g, k)), v && Ve(v, g), Ve(() => {
      h.isUnmounted = true;
    }, g);
  }, Fe = (h, g, k, R = false, I = false, C = 0) => {
    for (let q = C; q < h.length; q++) K(h[q], g, k, R, I);
  }, z = (h) => {
    if (h.shapeFlag & 6) return z(h.component.subTree);
    if (h.shapeFlag & 128) return h.suspense.next();
    const g = m(h.anchor || h.el), k = g && g[$u];
    return k ? m(k) : g;
  };
  let F = false;
  const L = (h, g, k) => {
    let R;
    h == null ? g._vnode && (K(g._vnode, null, null, true), R = g._vnode.component) : P(g._vnode || null, h, g, null, null, null, k), g._vnode = h, F || (F = true, _i(R), _r(), F = false);
  }, B = { p: P, um: K, m: U, r: ie, mt: en, mc: $e, pc: ae, pbc: Se, n: z, o: e };
  return { render: L, hydrate: void 0, createApp: Gu(L) };
}
function aa({ type: e, props: n }, t) {
  return t === "svg" && e === "foreignObject" || t === "mathml" && e === "annotation-xml" && n && n.encoding && n.encoding.includes("html") ? void 0 : t;
}
function Qn({ effect: e, job: n }, t) {
  t ? (e.flags |= 32, n.flags |= 4) : (e.flags &= -33, n.flags &= -5);
}
function cc(e, n) {
  return (!e || e && !e.pendingBranch) && n && !n.persisted;
}
function Mr(e, n, t = false) {
  const s = e.children, a = n.children;
  if (Q(s) && Q(a)) for (let i = 0; i < s.length; i++) {
    const o = s[i];
    let r = a[i];
    r.shapeFlag & 1 && !r.dynamicChildren && ((r.patchFlag <= 0 || r.patchFlag === 32) && (r = a[i] = zn(a[i]), r.el = o.el), !t && r.patchFlag !== -2 && Mr(o, r)), r.type === Hs && (r.patchFlag === -1 && (r = a[i] = zn(r)), r.el = o.el), r.type === Gn && !r.el && (r.el = o.el);
  }
}
function dc(e) {
  const n = e.slice(), t = [0];
  let s, a, i, o, r;
  const l = e.length;
  for (s = 0; s < l; s++) {
    const d = e[s];
    if (d !== 0) {
      if (a = t[t.length - 1], e[a] < d) {
        n[s] = a, t.push(s);
        continue;
      }
      for (i = 0, o = t.length - 1; i < o; ) r = i + o >> 1, e[t[r]] < d ? i = r + 1 : o = r;
      d < e[t[i]] && (i > 0 && (n[s] = t[i - 1]), t[i] = s);
    }
  }
  for (i = t.length, o = t[i - 1]; i-- > 0; ) t[i] = o, o = n[o];
  return t;
}
function Dr(e) {
  const n = e.subTree.component;
  if (n) return n.asyncDep && !n.asyncResolved ? n : Dr(n);
}
function Ai(e) {
  if (e) for (let n = 0; n < e.length; n++) e[n].flags |= 8;
}
function Lr(e) {
  if (e.placeholder) return e.placeholder;
  const n = e.component;
  return n ? Lr(n.subTree) : null;
}
const Ur = (e) => e.__isSuspense;
function pc(e, n) {
  n && n.pendingBranch ? Q(e) ? n.effects.push(...e) : n.effects.push(e) : ku(e);
}
const ee = Symbol.for("v-fgt"), Hs = Symbol.for("v-txt"), Gn = Symbol.for("v-cmt"), vs = Symbol.for("v-stc"), Wt = [];
let Ke = null;
function x(e = false) {
  Wt.push(Ke = e ? null : []);
}
function hc() {
  Wt.pop(), Ke = Wt[Wt.length - 1] || null;
}
let Qt = 1;
function Ss(e, n = false) {
  Qt += e, e < 0 && Ke && n && (Ke.hasOnce = true);
}
function Fr(e) {
  return e.dynamicChildren = Qt > 0 ? Ke || ut : null, hc(), Qt > 0 && Ke && Ke.push(e), e;
}
function j(e, n, t, s, a, i) {
  return Fr(u(e, n, t, s, a, i, true));
}
function ti(e, n, t, s, a) {
  return Fr(_e(e, n, t, s, a, true));
}
function Es(e) {
  return e ? e.__v_isVNode === true : false;
}
function At(e, n) {
  return e.type === n.type && e.key === n.key;
}
const Vr = ({ key: e }) => e ?? null, ys = ({ ref: e, ref_key: n, ref_for: t }) => (typeof e == "number" && (e = "" + e), e != null ? be(e) || Te(e) || Z(e) ? { i: Be, r: e, k: n, f: !!t } : e : null);
function u(e, n = null, t = null, s = 0, a = null, i = e === ee ? 0 : 1, o = false, r = false) {
  const l = { __v_isVNode: true, __v_skip: true, type: e, props: n, key: n && Vr(n), ref: n && ys(n), scopeId: yr, slotScopeIds: null, children: t, component: null, suspense: null, ssContent: null, ssFallback: null, dirs: null, transition: null, el: null, anchor: null, target: null, targetStart: null, targetAnchor: null, staticCount: 0, shapeFlag: i, patchFlag: s, dynamicProps: a, dynamicChildren: null, appContext: null, ctx: Be };
  return r ? (As(l, t), i & 128 && e.normalize(l)) : t && (l.shapeFlag |= be(t) ? 8 : 16), Qt > 0 && !o && Ke && (l.patchFlag > 0 || i & 6) && l.patchFlag !== 32 && Ke.push(l), l;
}
const _e = fc;
function fc(e, n = null, t = null, s = 0, a = null, i = false) {
  if ((!e || e === Mu) && (e = Gn), Es(e)) {
    const r = mt(e, n, true);
    return t && As(r, t), Qt > 0 && !i && Ke && (r.shapeFlag & 6 ? Ke[Ke.indexOf(e)] = r : Ke.push(r)), r.patchFlag = -2, r;
  }
  if (jc(e) && (e = e.__vccOpts), n) {
    n = mc(n);
    let { class: r, style: l } = n;
    r && !be(r) && (n.class = Re(r)), pe(l) && (Qa(l) && !Q(l) && (l = ze({}, l)), n.style = Kn(l));
  }
  const o = be(e) ? 1 : Ur(e) ? 128 : ju(e) ? 64 : pe(e) ? 4 : Z(e) ? 2 : 0;
  return u(e, n, t, s, a, o, i, true);
}
function mc(e) {
  return e ? Qa(e) || Ir(e) ? ze({}, e) : e : null;
}
function mt(e, n, t = false, s = false) {
  const { props: a, ref: i, patchFlag: o, children: r, transition: l } = e, d = n ? gc(a || {}, n) : a, c = { __v_isVNode: true, __v_skip: true, type: e.type, props: d, key: d && Vr(d), ref: n && n.ref ? t && i ? Q(i) ? i.concat(ys(n)) : [i, ys(n)] : ys(n) : i, scopeId: e.scopeId, slotScopeIds: e.slotScopeIds, children: r, target: e.target, targetStart: e.targetStart, targetAnchor: e.targetAnchor, staticCount: e.staticCount, shapeFlag: e.shapeFlag, patchFlag: n && e.type !== ee ? o === -1 ? 16 : o | 16 : o, dynamicProps: e.dynamicProps, dynamicChildren: e.dynamicChildren, appContext: e.appContext, dirs: e.dirs, transition: l, component: e.component, suspense: e.suspense, ssContent: e.ssContent && mt(e.ssContent), ssFallback: e.ssFallback && mt(e.ssFallback), placeholder: e.placeholder, el: e.el, anchor: e.anchor, ctx: e.ctx, ce: e.ce };
  return l && s && Za(c, l.clone(c)), c;
}
function se(e = " ", n = 0) {
  return _e(Hs, null, e, n);
}
function Hr(e, n) {
  const t = _e(vs, null, e);
  return t.staticCount = n, t;
}
function V(e = "", n = false) {
  return n ? (x(), ti(Gn, null, e)) : _e(Gn, null, e);
}
function gn(e) {
  return e == null || typeof e == "boolean" ? _e(Gn) : Q(e) ? _e(ee, null, e.slice()) : Es(e) ? zn(e) : _e(Hs, null, String(e));
}
function zn(e) {
  return e.el === null && e.patchFlag !== -1 || e.memo ? e : mt(e);
}
function As(e, n) {
  let t = 0;
  const { shapeFlag: s } = e;
  if (n == null) n = null;
  else if (Q(n)) t = 16;
  else if (typeof n == "object") if (s & 65) {
    const a = n.default;
    a && (a._c && (a._d = false), As(e, a()), a._c && (a._d = true));
    return;
  } else {
    t = 32;
    const a = n._;
    !a && !Ir(n) ? n._ctx = Be : a === 3 && Be && (Be.slots._ === 1 ? n._ = 1 : (n._ = 2, e.patchFlag |= 1024));
  }
  else if (Z(n)) {
    if (s & 65) {
      As(e, { default: n });
      return;
    }
    n = { default: n, _ctx: Be }, t = 32;
  } else n = String(n), s & 64 ? (t = 16, n = [se(n)]) : t = 8;
  e.children = n, e.shapeFlag |= t;
}
function gc(...e) {
  const n = {};
  for (let t = 0; t < e.length; t++) {
    const s = e[t];
    for (const a in s) if (a === "class") n.class !== s.class && (n.class = Re([n.class, s.class]));
    else if (a === "style") n.style = Kn([n.style, s.style]);
    else if (Os(a)) {
      const i = n[a], o = s[a];
      o && i !== o && !(Q(i) && i.includes(o)) ? n[a] = i ? [].concat(i, o) : o : o == null && i == null && !Ns(a) && (n[a] = o);
    } else a !== "" && (n[a] = s[a]);
  }
  return n;
}
function hn(e, n, t, s = null) {
  on(e, n, 7, [t, s]);
}
const _c = Er();
let vc = 0;
function yc(e, n, t) {
  const s = e.type, a = (n ? n.appContext : e.appContext) || _c, i = { uid: vc++, vnode: e, type: s, parent: n, appContext: a, root: null, next: null, subTree: null, effect: null, update: null, job: null, scope: new Wl(true), render: null, proxy: null, exposed: null, exposeProxy: null, withProxy: null, provides: n ? n.provides : Object.create(a.provides), ids: n ? n.ids : ["", 0, 0], accessCache: null, renderCache: [], components: null, directives: null, propsOptions: Tr(s, a), emitsOptions: Ar(s, a), emit: null, emitted: null, propsDefaults: me, inheritAttrs: s.inheritAttrs, ctx: me, data: me, props: me, attrs: me, slots: me, refs: me, setupState: me, setupContext: null, suspense: t, suspenseId: t ? t.pendingId : 0, asyncDep: null, asyncResolved: false, isMounted: false, isUnmounted: false, isDeactivated: false, bc: null, c: null, bm: null, m: null, bu: null, u: null, um: null, bum: null, da: null, a: null, rtg: null, rtc: null, ec: null, sp: null };
  return i.ctx = { _: i }, i.root = n ? n.root : i, i.emit = Yu.bind(null, i), e.ce && e.ce(i), i;
}
let Pe = null;
const kc = () => Pe || Be;
let Cs, xa;
{
  const e = Ds(), n = (t, s) => {
    let a;
    return (a = e[t]) || (a = e[t] = []), a.push(s), (i) => {
      a.length > 1 ? a.forEach((o) => o(i)) : a[0](i);
    };
  };
  Cs = n("__VUE_INSTANCE_SETTERS__", (t) => Pe = t), xa = n("__VUE_SSR_SETTERS__", (t) => Xt = t);
}
const is = (e) => {
  const n = Pe;
  return Cs(e), e.scope.on(), () => {
    e.scope.off(), Cs(n);
  };
}, Ci = () => {
  Pe && Pe.scope.off(), Cs(null);
};
function Wr(e) {
  return e.vnode.shapeFlag & 4;
}
let Xt = false;
function wc(e, n = false, t = false) {
  n && xa(n);
  const { props: s, children: a } = e.vnode, i = Wr(e);
  tc(e, s, i, n), oc(e, a, t || n);
  const o = i ? bc(e, n) : void 0;
  return n && xa(false), o;
}
function bc(e, n) {
  const t = e.type;
  e.accessCache = /* @__PURE__ */ Object.create(null), e.proxy = new Proxy(e.ctx, Uu);
  const { setup: s } = t;
  if (s) {
    xn();
    const a = e.setupContext = s.length > 1 ? $c(e) : null, i = is(e), o = ts(s, e, 0, [e.props, a]), r = Wo(o);
    if ($n(), i(), (r || e.sp) && !Vt(e) && br(e), r) {
      if (o.then(Ci, Ci), n) return o.then((l) => {
        Ri(e, l);
      }).catch((l) => {
        Us(l, e, 0);
      });
      e.asyncDep = o;
    } else Ri(e, o);
  } else Br(e);
}
function Ri(e, n, t) {
  Z(n) ? e.type.__ssrInlineRender ? e.ssrRender = n : e.render = n : pe(n) && (e.setupState = fr(n)), Br(e);
}
function Br(e, n, t) {
  const s = e.type;
  e.render || (e.render = s.render || yn);
  {
    const a = is(e);
    xn();
    try {
      Fu(e);
    } finally {
      $n(), a();
    }
  }
}
const xc = { get(e, n) {
  return Ie(e, "get", ""), e[n];
} };
function $c(e) {
  const n = (t) => {
    e.exposed = t || {};
  };
  return { attrs: new Proxy(e.attrs, xc), slots: e.slots, emit: e.emit, expose: n };
}
function Ws(e) {
  return e.exposed ? e.exposeProxy || (e.exposeProxy = new Proxy(fr(cu(e.exposed)), { get(n, t) {
    if (t in n) return n[t];
    if (t in Ht) return Ht[t](e);
  }, has(n, t) {
    return t in n || t in Ht;
  } })) : e.proxy;
}
function jc(e) {
  return Z(e) && "__vccOpts" in e;
}
const Y = (e, n) => mu(e, n, Xt);
function Kr(e, n, t) {
  try {
    Ss(-1);
    const s = arguments.length;
    return s === 2 ? pe(n) && !Q(n) ? Es(n) ? _e(e, null, [n]) : _e(e, n) : _e(e, null, n) : (s > 3 ? t = Array.prototype.slice.call(arguments, 2) : s === 3 && Es(t) && (t = [t]), _e(e, n, t));
  } finally {
    Ss(1);
  }
}
const Sc = "3.5.39";
/**
* @vue/runtime-dom v3.5.39
* (c) 2018-present Yuxi (Evan) You and Vue contributors
* @license MIT
**/
let $a;
const zi = typeof window < "u" && window.trustedTypes;
if (zi) try {
  $a = zi.createPolicy("vue", { createHTML: (e) => e });
} catch {
}
const Gr = $a ? (e) => $a.createHTML(e) : (e) => e, Ec = "http://www.w3.org/2000/svg", Ac = "http://www.w3.org/1998/Math/MathML", Rn = typeof document < "u" ? document : null, Ii = Rn && Rn.createElement("template"), Cc = { insert: (e, n, t) => {
  n.insertBefore(e, t || null);
}, remove: (e) => {
  const n = e.parentNode;
  n && n.removeChild(e);
}, createElement: (e, n, t, s) => {
  const a = n === "svg" ? Rn.createElementNS(Ec, e) : n === "mathml" ? Rn.createElementNS(Ac, e) : t ? Rn.createElement(e, { is: t }) : Rn.createElement(e);
  return e === "select" && s && s.multiple != null && a.setAttribute("multiple", s.multiple), a;
}, createText: (e) => Rn.createTextNode(e), createComment: (e) => Rn.createComment(e), setText: (e, n) => {
  e.nodeValue = n;
}, setElementText: (e, n) => {
  e.textContent = n;
}, parentNode: (e) => e.parentNode, nextSibling: (e) => e.nextSibling, querySelector: (e) => Rn.querySelector(e), setScopeId(e, n) {
  e.setAttribute(n, "");
}, insertStaticContent(e, n, t, s, a, i) {
  const o = t ? t.previousSibling : n.lastChild;
  if (a && (a === i || a.nextSibling)) for (; n.insertBefore(a.cloneNode(true), t), !(a === i || !(a = a.nextSibling)); ) ;
  else {
    Ii.innerHTML = Gr(s === "svg" ? `<svg>${e}</svg>` : s === "mathml" ? `<math>${e}</math>` : e);
    const r = Ii.content;
    if (s === "svg" || s === "mathml") {
      const l = r.firstChild;
      for (; l.firstChild; ) r.appendChild(l.firstChild);
      r.removeChild(l);
    }
    n.insertBefore(r, t);
  }
  return [o ? o.nextSibling : n.firstChild, t ? t.previousSibling : n.lastChild];
} }, Rc = Symbol("_vtc");
function zc(e, n, t) {
  const s = e[Rc];
  s && (n = (n ? [n, ...s] : [...s]).join(" ")), n == null ? e.removeAttribute("class") : t ? e.setAttribute("class", n) : e.className = n;
}
const Pi = Symbol("_vod"), Ic = Symbol("_vsh"), Pc = Symbol(""), Tc = /(?:^|;)\s*display\s*:/;
function Oc(e, n, t) {
  const s = e.style, a = be(t);
  let i = false;
  if (t && !a) {
    if (n) if (be(n)) for (const o of n.split(";")) {
      const r = o.slice(0, o.indexOf(":")).trim();
      t[r] == null && Tt(s, r, "");
    }
    else for (const o in n) t[o] == null && Tt(s, o, "");
    for (const o in t) {
      o === "display" && (i = true);
      const r = t[o];
      r != null ? qc(e, o, !be(n) && n ? n[o] : void 0, r) || Tt(s, o, r) : Tt(s, o, "");
    }
  } else if (a) {
    if (n !== t) {
      const o = s[Pc];
      o && (t += ";" + o), s.cssText = t, i = Tc.test(t);
    }
  } else n && e.removeAttribute("style");
  Pi in e && (e[Pi] = i ? s.display : "", e[Ic] && (s.display = "none"));
}
const Ti = /\s*!important$/;
function Tt(e, n, t) {
  if (Q(t)) t.forEach((s) => Tt(e, n, s));
  else if (t == null && (t = ""), n.startsWith("--")) e.setProperty(n, t);
  else {
    const s = Nc(e, n);
    Ti.test(t) ? e.setProperty(Jn(s), t.replace(Ti, ""), "important") : e[s] = t;
  }
}
const Oi = ["Webkit", "Moz", "ms"], ia = {};
function Nc(e, n) {
  const t = ia[n];
  if (t) return t;
  let s = Ge(n);
  if (s !== "filter" && s in e) return ia[n] = s;
  s = Va(s);
  for (let a = 0; a < Oi.length; a++) {
    const i = Oi[a] + s;
    if (i in e) return ia[n] = i;
  }
  return n;
}
function qc(e, n, t, s) {
  return e.tagName === "TEXTAREA" && (n === "width" || n === "height") && be(s) && t === s;
}
const Ni = "http://www.w3.org/1999/xlink";
function qi(e, n, t, s, a, i = Vl(n)) {
  s && n.startsWith("xlink:") ? t == null ? e.removeAttributeNS(Ni, n.slice(6, n.length)) : e.setAttributeNS(Ni, n, t) : t == null || i && !Jo(t) ? e.removeAttribute(n) : e.setAttribute(n, i ? "" : bn(t) ? String(t) : t);
}
function Mi(e, n, t, s, a) {
  if (n === "innerHTML" || n === "textContent") {
    t != null && (e[n] = n === "innerHTML" ? Gr(t) : t);
    return;
  }
  const i = e.tagName;
  if (n === "value" && i !== "PROGRESS" && !i.includes("-")) {
    const r = i === "OPTION" ? e.getAttribute("value") || "" : e.value, l = t == null ? e.type === "checkbox" ? "on" : "" : String(t);
    (r !== l || !("_value" in e)) && (e.value = l), t == null && e.removeAttribute(n), e._value = t;
    return;
  }
  let o = false;
  if (t === "" || t == null) {
    const r = typeof e[n];
    r === "boolean" ? t = Jo(t) : t == null && r === "string" ? (t = "", o = true) : r === "number" && (t = 0, o = true);
  }
  try {
    e[n] = t;
  } catch {
  }
  o && e.removeAttribute(a || n);
}
function Hn(e, n, t, s) {
  e.addEventListener(n, t, s);
}
function Mc(e, n, t, s) {
  e.removeEventListener(n, t, s);
}
const Di = Symbol("_vei");
function Dc(e, n, t, s, a = null) {
  const i = e[Di] || (e[Di] = {}), o = i[n];
  if (s && o) o.value = s;
  else {
    const [r, l] = Fc(n);
    if (s) {
      const d = i[n] = Wc(s, a);
      Hn(e, r, d, l);
    } else o && (Mc(e, r, o, l), i[n] = void 0);
  }
}
const Lc = /(Once|Passive|Capture)$/, Uc = /^on:?(?:Once|Passive|Capture)$/;
function Fc(e) {
  let n, t;
  for (; (t = e.match(Lc)) && !Uc.test(e); ) n || (n = {}), e = e.slice(0, e.length - t[1].length), n[t[1].toLowerCase()] = true;
  return [e[2] === ":" ? e.slice(3) : Jn(e.slice(2)), n];
}
let oa = 0;
const Vc = Promise.resolve(), Hc = () => oa || (Vc.then(() => oa = 0), oa = Date.now());
function Wc(e, n) {
  const t = (s) => {
    if (!s._vts) s._vts = Date.now();
    else if (s._vts <= t.attached) return;
    const a = t.value;
    if (Q(a)) {
      const i = s.stopImmediatePropagation;
      s.stopImmediatePropagation = () => {
        i.call(s), s._stopped = true;
      };
      const o = a.slice(), r = [s];
      for (let l = 0; l < o.length && !s._stopped; l++) {
        const d = o[l];
        d && on(d, n, 5, r);
      }
    } else on(a, n, 5, [s]);
  };
  return t.value = e, t.attached = Hc(), t;
}
const Li = (e) => e.charCodeAt(0) === 111 && e.charCodeAt(1) === 110 && e.charCodeAt(2) > 96 && e.charCodeAt(2) < 123, Bc = (e, n, t, s, a, i) => {
  const o = a === "svg";
  n === "class" ? zc(e, s, o) : n === "style" ? Oc(e, t, s) : Os(n) ? Ns(n) || Dc(e, n, t, s, i) : (n[0] === "." ? (n = n.slice(1), true) : n[0] === "^" ? (n = n.slice(1), false) : Kc(e, n, s, o)) ? (Mi(e, n, s), !e.tagName.includes("-") && (n === "value" || n === "checked" || n === "selected") && qi(e, n, s, o, i, n !== "value")) : e._isVueCE && (Gc(e, n) || e._def.__asyncLoader && (/[A-Z]/.test(n) || !be(s))) ? Mi(e, Ge(n), s, i, n) : (n === "true-value" ? e._trueValue = s : n === "false-value" && (e._falseValue = s), qi(e, n, s, o));
};
function Kc(e, n, t, s) {
  if (s) return !!(n === "innerHTML" || n === "textContent" || n in e && Li(n) && Z(t));
  if (n === "spellcheck" || n === "draggable" || n === "translate" || n === "autocorrect" || n === "sandbox" && e.tagName === "IFRAME" || n === "form" || n === "list" && e.tagName === "INPUT" || n === "type" && e.tagName === "TEXTAREA") return false;
  if (n === "width" || n === "height") {
    const a = e.tagName;
    if (a === "IMG" || a === "VIDEO" || a === "CANVAS" || a === "SOURCE") return false;
  }
  return Li(n) && be(t) ? false : n in e;
}
function Gc(e, n) {
  const t = e._def.props;
  if (!t) return false;
  const s = Ge(n);
  return Array.isArray(t) ? t.some((a) => Ge(a) === s) : Object.keys(t).some((a) => Ge(a) === s);
}
const gt = (e) => {
  const n = e.props["onUpdate:modelValue"] || false;
  return Q(n) ? (t) => gs(n, t) : n;
};
function Jc(e) {
  e.target.composing = true;
}
function Ui(e) {
  const n = e.target;
  n.composing && (n.composing = false, n.dispatchEvent(new Event("input")));
}
const Tn = Symbol("_assign");
function Fi(e, n, t) {
  return n && (e = e.trim()), t && (e = Ms(e)), e;
}
const Wn = { created(e, { modifiers: { lazy: n, trim: t, number: s } }, a) {
  e[Tn] = gt(a);
  const i = s || a.props && a.props.type === "number";
  Hn(e, n ? "change" : "input", (o) => {
    o.target.composing || e[Tn](Fi(e.value, t, i));
  }), (t || i) && Hn(e, "change", () => {
    e.value = Fi(e.value, t, i);
  }), n || (Hn(e, "compositionstart", Jc), Hn(e, "compositionend", Ui), Hn(e, "change", Ui));
}, mounted(e, { value: n }) {
  e.value = n ?? "";
}, beforeUpdate(e, { value: n, oldValue: t, modifiers: { lazy: s, trim: a, number: i } }, o) {
  if (e[Tn] = gt(o), e.composing) return;
  const r = (i || e.type === "number") && !/^0\d/.test(e.value) ? Ms(e.value) : e.value, l = n ?? "";
  if (r === l) return;
  const d = e.getRootNode();
  (d instanceof Document || d instanceof ShadowRoot) && d.activeElement === e && e.type !== "range" && (s && n === t || a && e.value.trim() === l) || (e.value = l);
} }, ks = { deep: true, created(e, n, t) {
  e[Tn] = gt(t), Hn(e, "change", () => {
    const s = e._modelValue, a = Zt(e), i = e.checked, o = e[Tn];
    if (Q(s)) {
      const r = Ha(s, a), l = r !== -1;
      if (i && !l) o(s.concat(a));
      else if (!i && l) {
        const d = [...s];
        d.splice(r, 1), o(d);
      }
    } else if (wt(s)) {
      const r = new Set(s);
      i ? r.add(a) : r.delete(a), o(r);
    } else o(Jr(e, i));
  });
}, mounted: Vi, beforeUpdate(e, n, t) {
  e[Tn] = gt(t), Vi(e, n, t);
} };
function Vi(e, { value: n, oldValue: t }, s) {
  e._modelValue = n;
  let a;
  if (Q(n)) a = Ha(n, s.props.value) > -1;
  else if (wt(n)) a = n.has(s.props.value);
  else {
    if (n === t) return;
    a = bt(n, Jr(e, true));
  }
  e.checked !== a && (e.checked = a);
}
const vn = { deep: true, created(e, { value: n, modifiers: { number: t } }, s) {
  const a = wt(n);
  Hn(e, "change", () => {
    const i = Array.prototype.filter.call(e.options, (o) => o.selected).map((o) => t ? Ms(Zt(o)) : Zt(o));
    e[Tn](e.multiple ? a ? new Set(i) : i : i[0]), e._assigning = true, ss(() => {
      e._assigning = false;
    });
  }), e[Tn] = gt(s);
}, mounted(e, { value: n }) {
  Hi(e, n);
}, beforeUpdate(e, n, t) {
  e[Tn] = gt(t);
}, updated(e, { value: n }) {
  e._assigning || Hi(e, n);
} };
function Hi(e, n) {
  const t = e.multiple, s = Q(n);
  if (!(t && !s && !wt(n))) {
    for (let a = 0, i = e.options.length; a < i; a++) {
      const o = e.options[a], r = Zt(o);
      if (t) if (s) {
        const l = typeof r;
        l === "string" || l === "number" ? o.selected = n.some((d) => String(d) === String(r)) : o.selected = Ha(n, r) > -1;
      } else o.selected = n.has(r);
      else if (bt(Zt(o), n)) {
        e.selectedIndex !== a && (e.selectedIndex = a);
        return;
      }
    }
    !t && e.selectedIndex !== -1 && (e.selectedIndex = -1);
  }
}
function Zt(e) {
  return "_value" in e ? e._value : e.value;
}
function Jr(e, n) {
  const t = n ? "_trueValue" : "_falseValue";
  return t in e ? e[t] : n;
}
const Yc = ["ctrl", "shift", "alt", "meta"], Qc = { stop: (e) => e.stopPropagation(), prevent: (e) => e.preventDefault(), self: (e) => e.target !== e.currentTarget, ctrl: (e) => !e.ctrlKey, shift: (e) => !e.shiftKey, alt: (e) => !e.altKey, meta: (e) => !e.metaKey, left: (e) => "button" in e && e.button !== 0, middle: (e) => "button" in e && e.button !== 1, right: (e) => "button" in e && e.button !== 2, exact: (e, n) => Yc.some((t) => e[`${t}Key`] && !n.includes(t)) }, ja = (e, n) => {
  if (!e) return e;
  const t = e._withMods || (e._withMods = {}), s = n.join(".");
  return t[s] || (t[s] = ((a, ...i) => {
    for (let o = 0; o < n.length; o++) {
      const r = Qc[n[o]];
      if (r && r(a, n)) return;
    }
    return e(a, ...i);
  }));
}, Xc = { esc: "escape", space: " ", up: "arrow-up", left: "arrow-left", right: "arrow-right", down: "arrow-down", delete: "backspace" }, Zc = (e, n) => {
  const t = e._withKeys || (e._withKeys = {}), s = n.join(".");
  return t[s] || (t[s] = ((a) => {
    if (!("key" in a)) return;
    const i = Jn(a.key);
    if (n.some((o) => o === i || Xc[o] === i)) return e(a);
  }));
}, ed = ze({ patchProp: Bc }, Cc);
let Wi;
function nd() {
  return Wi || (Wi = lc(ed));
}
const td = ((...e) => {
  const n = nd().createApp(...e), { mount: t } = n;
  return n.mount = (s) => {
    const a = ad(s);
    if (!a) return;
    const i = n._component;
    !Z(i) && !i.render && !i.template && (i.template = a.innerHTML), a.nodeType === 1 && (a.textContent = "");
    const o = t(a, false, sd(a));
    return a instanceof Element && (a.removeAttribute("v-cloak"), a.setAttribute("data-v-app", "")), o;
  }, n;
});
function sd(e) {
  if (e instanceof SVGElement) return "svg";
  if (typeof MathMLElement == "function" && e instanceof MathMLElement) return "mathml";
}
function ad(e) {
  return be(e) ? document.querySelector(e) : e;
}
/*!
* vue-router v4.6.4
* (c) 2025 Eduardo San Martin Morote
* @license MIT
*/
const rt = typeof document < "u";
function Yr(e) {
  return typeof e == "object" || "displayName" in e || "props" in e || "__vccOpts" in e;
}
function id(e) {
  return e.__esModule || e[Symbol.toStringTag] === "Module" || e.default && Yr(e.default);
}
const ue = Object.assign;
function ra(e, n) {
  const t = {};
  for (const s in n) {
    const a = n[s];
    t[s] = rn(a) ? a.map(e) : e(a);
  }
  return t;
}
const Bt = () => {
}, rn = Array.isArray;
function Bi(e, n) {
  const t = {};
  for (const s in e) t[s] = s in n ? n[s] : e[s];
  return t;
}
const Qr = /#/g, od = /&/g, rd = /\//g, ld = /=/g, ud = /\?/g, Xr = /\+/g, cd = /%5B/g, dd = /%5D/g, Zr = /%5E/g, pd = /%60/g, el = /%7B/g, hd = /%7C/g, nl = /%7D/g, fd = /%20/g;
function si(e) {
  return e == null ? "" : encodeURI("" + e).replace(hd, "|").replace(cd, "[").replace(dd, "]");
}
function md(e) {
  return si(e).replace(el, "{").replace(nl, "}").replace(Zr, "^");
}
function Sa(e) {
  return si(e).replace(Xr, "%2B").replace(fd, "+").replace(Qr, "%23").replace(od, "%26").replace(pd, "`").replace(el, "{").replace(nl, "}").replace(Zr, "^");
}
function gd(e) {
  return Sa(e).replace(ld, "%3D");
}
function _d(e) {
  return si(e).replace(Qr, "%23").replace(ud, "%3F");
}
function vd(e) {
  return _d(e).replace(rd, "%2F");
}
function es(e) {
  if (e == null) return null;
  try {
    return decodeURIComponent("" + e);
  } catch {
  }
  return "" + e;
}
const yd = /\/$/, kd = (e) => e.replace(yd, "");
function la(e, n, t = "/") {
  let s, a = {}, i = "", o = "";
  const r = n.indexOf("#");
  let l = n.indexOf("?");
  return l = r >= 0 && l > r ? -1 : l, l >= 0 && (s = n.slice(0, l), i = n.slice(l, r > 0 ? r : n.length), a = e(i.slice(1))), r >= 0 && (s = s || n.slice(0, r), o = n.slice(r, n.length)), s = $d(s ?? n, t), { fullPath: s + i + o, path: s, query: a, hash: es(o) };
}
function wd(e, n) {
  const t = n.query ? e(n.query) : "";
  return n.path + (t && "?") + t + (n.hash || "");
}
function Ki(e, n) {
  return !n || !e.toLowerCase().startsWith(n.toLowerCase()) ? e : e.slice(n.length) || "/";
}
function bd(e, n, t) {
  const s = n.matched.length - 1, a = t.matched.length - 1;
  return s > -1 && s === a && _t(n.matched[s], t.matched[a]) && tl(n.params, t.params) && e(n.query) === e(t.query) && n.hash === t.hash;
}
function _t(e, n) {
  return (e.aliasOf || e) === (n.aliasOf || n);
}
function tl(e, n) {
  if (Object.keys(e).length !== Object.keys(n).length) return false;
  for (var t in e) if (!xd(e[t], n[t])) return false;
  return true;
}
function xd(e, n) {
  return rn(e) ? Gi(e, n) : rn(n) ? Gi(n, e) : (e == null ? void 0 : e.valueOf()) === (n == null ? void 0 : n.valueOf());
}
function Gi(e, n) {
  return rn(n) ? e.length === n.length && e.every((t, s) => t === n[s]) : e.length === 1 && e[0] === n;
}
function $d(e, n) {
  if (e.startsWith("/")) return e;
  if (!e) return n;
  const t = n.split("/"), s = e.split("/"), a = s[s.length - 1];
  (a === ".." || a === ".") && s.push("");
  let i = t.length - 1, o, r;
  for (o = 0; o < s.length; o++) if (r = s[o], r !== ".") if (r === "..") i > 1 && i--;
  else break;
  return t.slice(0, i).join("/") + "/" + s.slice(o).join("/");
}
const Mn = { path: "/", name: void 0, params: {}, query: {}, hash: "", fullPath: "/", matched: [], meta: {}, redirectedFrom: void 0 };
let Ea = (function(e) {
  return e.pop = "pop", e.push = "push", e;
})({}), ua = (function(e) {
  return e.back = "back", e.forward = "forward", e.unknown = "", e;
})({});
function jd(e) {
  if (!e) if (rt) {
    const n = document.querySelector("base");
    e = n && n.getAttribute("href") || "/", e = e.replace(/^\w+:\/\/[^\/]+/, "");
  } else e = "/";
  return e[0] !== "/" && e[0] !== "#" && (e = "/" + e), kd(e);
}
const Sd = /^[^#]+#/;
function Ed(e, n) {
  return e.replace(Sd, "#") + n;
}
function Ad(e, n) {
  const t = document.documentElement.getBoundingClientRect(), s = e.getBoundingClientRect();
  return { behavior: n.behavior, left: s.left - t.left - (n.left || 0), top: s.top - t.top - (n.top || 0) };
}
const Bs = () => ({ left: window.scrollX, top: window.scrollY });
function Cd(e) {
  let n;
  if ("el" in e) {
    const t = e.el, s = typeof t == "string" && t.startsWith("#"), a = typeof t == "string" ? s ? document.getElementById(t.slice(1)) : document.querySelector(t) : t;
    if (!a) return;
    n = Ad(a, e);
  } else n = e;
  "scrollBehavior" in document.documentElement.style ? window.scrollTo(n) : window.scrollTo(n.left != null ? n.left : window.scrollX, n.top != null ? n.top : window.scrollY);
}
function Ji(e, n) {
  return (history.state ? history.state.position - n : -1) + e;
}
const Aa = /* @__PURE__ */ new Map();
function Rd(e, n) {
  Aa.set(e, n);
}
function zd(e) {
  const n = Aa.get(e);
  return Aa.delete(e), n;
}
function Id(e) {
  return typeof e == "string" || e && typeof e == "object";
}
function sl(e) {
  return typeof e == "string" || typeof e == "symbol";
}
let we = (function(e) {
  return e[e.MATCHER_NOT_FOUND = 1] = "MATCHER_NOT_FOUND", e[e.NAVIGATION_GUARD_REDIRECT = 2] = "NAVIGATION_GUARD_REDIRECT", e[e.NAVIGATION_ABORTED = 4] = "NAVIGATION_ABORTED", e[e.NAVIGATION_CANCELLED = 8] = "NAVIGATION_CANCELLED", e[e.NAVIGATION_DUPLICATED = 16] = "NAVIGATION_DUPLICATED", e;
})({});
const al = Symbol("");
we.MATCHER_NOT_FOUND + "", we.NAVIGATION_GUARD_REDIRECT + "", we.NAVIGATION_ABORTED + "", we.NAVIGATION_CANCELLED + "", we.NAVIGATION_DUPLICATED + "";
function vt(e, n) {
  return ue(new Error(), { type: e, [al]: true }, n);
}
function An(e, n) {
  return e instanceof Error && al in e && (n == null || !!(e.type & n));
}
const Pd = ["params", "query", "hash"];
function Td(e) {
  if (typeof e == "string") return e;
  if (e.path != null) return e.path;
  const n = {};
  for (const t of Pd) t in e && (n[t] = e[t]);
  return JSON.stringify(n, null, 2);
}
function Od(e) {
  const n = {};
  if (e === "" || e === "?") return n;
  const t = (e[0] === "?" ? e.slice(1) : e).split("&");
  for (let s = 0; s < t.length; ++s) {
    const a = t[s].replace(Xr, " "), i = a.indexOf("="), o = es(i < 0 ? a : a.slice(0, i)), r = i < 0 ? null : es(a.slice(i + 1));
    if (o in n) {
      let l = n[o];
      rn(l) || (l = n[o] = [l]), l.push(r);
    } else n[o] = r;
  }
  return n;
}
function Yi(e) {
  let n = "";
  for (let t in e) {
    const s = e[t];
    if (t = gd(t), s == null) {
      s !== void 0 && (n += (n.length ? "&" : "") + t);
      continue;
    }
    (rn(s) ? s.map((a) => a && Sa(a)) : [s && Sa(s)]).forEach((a) => {
      a !== void 0 && (n += (n.length ? "&" : "") + t, a != null && (n += "=" + a));
    });
  }
  return n;
}
function Nd(e) {
  const n = {};
  for (const t in e) {
    const s = e[t];
    s !== void 0 && (n[t] = rn(s) ? s.map((a) => a == null ? null : "" + a) : s == null ? s : "" + s);
  }
  return n;
}
const qd = Symbol(""), Qi = Symbol(""), Ks = Symbol(""), ai = Symbol(""), Ca = Symbol("");
function Ct() {
  let e = [];
  function n(s) {
    return e.push(s), () => {
      const a = e.indexOf(s);
      a > -1 && e.splice(a, 1);
    };
  }
  function t() {
    e = [];
  }
  return { add: n, list: () => e.slice(), reset: t };
}
function Vn(e, n, t, s, a, i = (o) => o()) {
  const o = s && (s.enterCallbacks[a] = s.enterCallbacks[a] || []);
  return () => new Promise((r, l) => {
    const d = (m) => {
      m === false ? l(vt(we.NAVIGATION_ABORTED, { from: t, to: n })) : m instanceof Error ? l(m) : Id(m) ? l(vt(we.NAVIGATION_GUARD_REDIRECT, { from: n, to: m })) : (o && s.enterCallbacks[a] === o && typeof m == "function" && o.push(m), r());
    }, c = i(() => e.call(s && s.instances[a], n, t, d));
    let p = Promise.resolve(c);
    e.length < 3 && (p = p.then(d)), p.catch((m) => l(m));
  });
}
function ca(e, n, t, s, a = (i) => i()) {
  const i = [];
  for (const o of e) for (const r in o.components) {
    let l = o.components[r];
    if (!(n !== "beforeRouteEnter" && !o.instances[r])) if (Yr(l)) {
      const d = (l.__vccOpts || l)[n];
      d && i.push(Vn(d, t, s, o, r, a));
    } else {
      let d = l();
      i.push(() => d.then((c) => {
        if (!c) throw new Error(`Couldn't resolve component "${r}" at "${o.path}"`);
        const p = id(c) ? c.default : c;
        o.mods[r] = c, o.components[r] = p;
        const m = (p.__vccOpts || p)[n];
        return m && Vn(m, t, s, o, r, a)();
      }));
    }
  }
  return i;
}
function Md(e, n) {
  const t = [], s = [], a = [], i = Math.max(n.matched.length, e.matched.length);
  for (let o = 0; o < i; o++) {
    const r = n.matched[o];
    r && (e.matched.find((d) => _t(d, r)) ? s.push(r) : t.push(r));
    const l = e.matched[o];
    l && (n.matched.find((d) => _t(d, l)) || a.push(l));
  }
  return [t, s, a];
}
/*!
* vue-router v4.6.4
* (c) 2025 Eduardo San Martin Morote
* @license MIT
*/
let Dd = () => location.protocol + "//" + location.host;
function il(e, n) {
  const { pathname: t, search: s, hash: a } = n, i = e.indexOf("#");
  if (i > -1) {
    let o = a.includes(e.slice(i)) ? e.slice(i).length : 1, r = a.slice(o);
    return r[0] !== "/" && (r = "/" + r), Ki(r, "");
  }
  return Ki(t, e) + s + a;
}
function Ld(e, n, t, s) {
  let a = [], i = [], o = null;
  const r = ({ state: m }) => {
    const f = il(e, location), b = t.value, P = n.value;
    let O = 0;
    if (m) {
      if (t.value = f, n.value = m, o && o === b) {
        o = null;
        return;
      }
      O = P ? m.position - P.position : 0;
    } else s(f);
    a.forEach((w) => {
      w(t.value, b, { delta: O, type: Ea.pop, direction: O ? O > 0 ? ua.forward : ua.back : ua.unknown });
    });
  };
  function l() {
    o = t.value;
  }
  function d(m) {
    a.push(m);
    const f = () => {
      const b = a.indexOf(m);
      b > -1 && a.splice(b, 1);
    };
    return i.push(f), f;
  }
  function c() {
    if (document.visibilityState === "hidden") {
      const { history: m } = window;
      if (!m.state) return;
      m.replaceState(ue({}, m.state, { scroll: Bs() }), "");
    }
  }
  function p() {
    for (const m of i) m();
    i = [], window.removeEventListener("popstate", r), window.removeEventListener("pagehide", c), document.removeEventListener("visibilitychange", c);
  }
  return window.addEventListener("popstate", r), window.addEventListener("pagehide", c), document.addEventListener("visibilitychange", c), { pauseListeners: l, listen: d, destroy: p };
}
function Xi(e, n, t, s = false, a = false) {
  return { back: e, current: n, forward: t, replaced: s, position: window.history.length, scroll: a ? Bs() : null };
}
function Ud(e) {
  const { history: n, location: t } = window, s = { value: il(e, t) }, a = { value: n.state };
  a.value || i(s.value, { back: null, current: s.value, forward: null, position: n.length - 1, replaced: true, scroll: null }, true);
  function i(l, d, c) {
    const p = e.indexOf("#"), m = p > -1 ? (t.host && document.querySelector("base") ? e : e.slice(p)) + l : Dd() + e + l;
    try {
      n[c ? "replaceState" : "pushState"](d, "", m), a.value = d;
    } catch (f) {
      console.error(f), t[c ? "replace" : "assign"](m);
    }
  }
  function o(l, d) {
    i(l, ue({}, n.state, Xi(a.value.back, l, a.value.forward, true), d, { position: a.value.position }), true), s.value = l;
  }
  function r(l, d) {
    const c = ue({}, a.value, n.state, { forward: l, scroll: Bs() });
    i(c.current, c, true), i(l, ue({}, Xi(s.value, l, null), { position: c.position + 1 }, d), false), s.value = l;
  }
  return { location: s, state: a, push: r, replace: o };
}
function ol(e) {
  e = jd(e);
  const n = Ud(e), t = Ld(e, n.state, n.location, n.replace);
  function s(i, o = true) {
    o || t.pauseListeners(), history.go(i);
  }
  const a = ue({ location: "", base: e, go: s, createHref: Ed.bind(null, e) }, n, t);
  return Object.defineProperty(a, "location", { enumerable: true, get: () => n.location.value }), Object.defineProperty(a, "state", { enumerable: true, get: () => n.state.value }), a;
}
function Fd(e) {
  return e = location.host ? e || location.pathname + location.search : "", e.includes("#") || (e += "#"), ol(e);
}
let Zn = (function(e) {
  return e[e.Static = 0] = "Static", e[e.Param = 1] = "Param", e[e.Group = 2] = "Group", e;
})({});
var je = (function(e) {
  return e[e.Static = 0] = "Static", e[e.Param = 1] = "Param", e[e.ParamRegExp = 2] = "ParamRegExp", e[e.ParamRegExpEnd = 3] = "ParamRegExpEnd", e[e.EscapeNext = 4] = "EscapeNext", e;
})(je || {});
const Vd = { type: Zn.Static, value: "" }, Hd = /[a-zA-Z0-9_]/;
function Wd(e) {
  if (!e) return [[]];
  if (e === "/") return [[Vd]];
  if (!e.startsWith("/")) throw new Error(`Invalid path "${e}"`);
  function n(f) {
    throw new Error(`ERR (${t})/"${d}": ${f}`);
  }
  let t = je.Static, s = t;
  const a = [];
  let i;
  function o() {
    i && a.push(i), i = [];
  }
  let r = 0, l, d = "", c = "";
  function p() {
    d && (t === je.Static ? i.push({ type: Zn.Static, value: d }) : t === je.Param || t === je.ParamRegExp || t === je.ParamRegExpEnd ? (i.length > 1 && (l === "*" || l === "+") && n(`A repeatable param (${d}) must be alone in its segment. eg: '/:ids+.`), i.push({ type: Zn.Param, value: d, regexp: c, repeatable: l === "*" || l === "+", optional: l === "*" || l === "?" })) : n("Invalid state to consume buffer"), d = "");
  }
  function m() {
    d += l;
  }
  for (; r < e.length; ) {
    if (l = e[r++], l === "\\" && t !== je.ParamRegExp) {
      s = t, t = je.EscapeNext;
      continue;
    }
    switch (t) {
      case je.Static:
        l === "/" ? (d && p(), o()) : l === ":" ? (p(), t = je.Param) : m();
        break;
      case je.EscapeNext:
        m(), t = s;
        break;
      case je.Param:
        l === "(" ? t = je.ParamRegExp : Hd.test(l) ? m() : (p(), t = je.Static, l !== "*" && l !== "?" && l !== "+" && r--);
        break;
      case je.ParamRegExp:
        l === ")" ? c[c.length - 1] == "\\" ? c = c.slice(0, -1) + l : t = je.ParamRegExpEnd : c += l;
        break;
      case je.ParamRegExpEnd:
        p(), t = je.Static, l !== "*" && l !== "?" && l !== "+" && r--, c = "";
        break;
      default:
        n("Unknown state");
        break;
    }
  }
  return t === je.ParamRegExp && n(`Unfinished custom RegExp for param "${d}"`), p(), o(), a;
}
const Zi = "[^/]+?", Bd = { sensitive: false, strict: false, start: true, end: true };
var qe = (function(e) {
  return e[e._multiplier = 10] = "_multiplier", e[e.Root = 90] = "Root", e[e.Segment = 40] = "Segment", e[e.SubSegment = 30] = "SubSegment", e[e.Static = 40] = "Static", e[e.Dynamic = 20] = "Dynamic", e[e.BonusCustomRegExp = 10] = "BonusCustomRegExp", e[e.BonusWildcard = -50] = "BonusWildcard", e[e.BonusRepeatable = -20] = "BonusRepeatable", e[e.BonusOptional = -8] = "BonusOptional", e[e.BonusStrict = 0.7000000000000001] = "BonusStrict", e[e.BonusCaseSensitive = 0.25] = "BonusCaseSensitive", e;
})(qe || {});
const Kd = /[.+*?^${}()[\]/\\]/g;
function Gd(e, n) {
  const t = ue({}, Bd, n), s = [];
  let a = t.start ? "^" : "";
  const i = [];
  for (const d of e) {
    const c = d.length ? [] : [qe.Root];
    t.strict && !d.length && (a += "/");
    for (let p = 0; p < d.length; p++) {
      const m = d[p];
      let f = qe.Segment + (t.sensitive ? qe.BonusCaseSensitive : 0);
      if (m.type === Zn.Static) p || (a += "/"), a += m.value.replace(Kd, "\\$&"), f += qe.Static;
      else if (m.type === Zn.Param) {
        const { value: b, repeatable: P, optional: O, regexp: w } = m;
        i.push({ name: b, repeatable: P, optional: O });
        const _ = w || Zi;
        if (_ !== Zi) {
          f += qe.BonusCustomRegExp;
          try {
            `${_}`;
          } catch (T) {
            throw new Error(`Invalid custom RegExp for param "${b}" (${_}): ` + T.message);
          }
        }
        let A = P ? `((?:${_})(?:/(?:${_}))*)` : `(${_})`;
        p || (A = O && d.length < 2 ? `(?:/${A})` : "/" + A), O && (A += "?"), a += A, f += qe.Dynamic, O && (f += qe.BonusOptional), P && (f += qe.BonusRepeatable), _ === ".*" && (f += qe.BonusWildcard);
      }
      c.push(f);
    }
    s.push(c);
  }
  if (t.strict && t.end) {
    const d = s.length - 1;
    s[d][s[d].length - 1] += qe.BonusStrict;
  }
  t.strict || (a += "/?"), t.end ? a += "$" : t.strict && !a.endsWith("/") && (a += "(?:/|$)");
  const o = new RegExp(a, t.sensitive ? "" : "i");
  function r(d) {
    const c = d.match(o), p = {};
    if (!c) return null;
    for (let m = 1; m < c.length; m++) {
      const f = c[m] || "", b = i[m - 1];
      p[b.name] = f && b.repeatable ? f.split("/") : f;
    }
    return p;
  }
  function l(d) {
    let c = "", p = false;
    for (const m of e) {
      (!p || !c.endsWith("/")) && (c += "/"), p = false;
      for (const f of m) if (f.type === Zn.Static) c += f.value;
      else if (f.type === Zn.Param) {
        const { value: b, repeatable: P, optional: O } = f, w = b in d ? d[b] : "";
        if (rn(w) && !P) throw new Error(`Provided param "${b}" is an array but it is not repeatable (* or + modifiers)`);
        const _ = rn(w) ? w.join("/") : w;
        if (!_) if (O) m.length < 2 && (c.endsWith("/") ? c = c.slice(0, -1) : p = true);
        else throw new Error(`Missing required param "${b}"`);
        c += _;
      }
    }
    return c || "/";
  }
  return { re: o, score: s, keys: i, parse: r, stringify: l };
}
function Jd(e, n) {
  let t = 0;
  for (; t < e.length && t < n.length; ) {
    const s = n[t] - e[t];
    if (s) return s;
    t++;
  }
  return e.length < n.length ? e.length === 1 && e[0] === qe.Static + qe.Segment ? -1 : 1 : e.length > n.length ? n.length === 1 && n[0] === qe.Static + qe.Segment ? 1 : -1 : 0;
}
function rl(e, n) {
  let t = 0;
  const s = e.score, a = n.score;
  for (; t < s.length && t < a.length; ) {
    const i = Jd(s[t], a[t]);
    if (i) return i;
    t++;
  }
  if (Math.abs(a.length - s.length) === 1) {
    if (eo(s)) return 1;
    if (eo(a)) return -1;
  }
  return a.length - s.length;
}
function eo(e) {
  const n = e[e.length - 1];
  return e.length > 0 && n[n.length - 1] < 0;
}
const Yd = { strict: false, end: true, sensitive: false };
function Qd(e, n, t) {
  const s = Gd(Wd(e.path), t), a = ue(s, { record: e, parent: n, children: [], alias: [] });
  return n && !a.record.aliasOf == !n.record.aliasOf && n.children.push(a), a;
}
function Xd(e, n) {
  const t = [], s = /* @__PURE__ */ new Map();
  n = Bi(Yd, n);
  function a(p) {
    return s.get(p);
  }
  function i(p, m, f) {
    const b = !f, P = to(p);
    P.aliasOf = f && f.record;
    const O = Bi(n, p), w = [P];
    if ("alias" in p) {
      const T = typeof p.alias == "string" ? [p.alias] : p.alias;
      for (const H of T) w.push(to(ue({}, P, { components: f ? f.record.components : P.components, path: H, aliasOf: f ? f.record : P })));
    }
    let _, A;
    for (const T of w) {
      const { path: H } = T;
      if (m && H[0] !== "/") {
        const le = m.record.path, ne = le[le.length - 1] === "/" ? "" : "/";
        T.path = m.record.path + (H && ne + H);
      }
      if (_ = Qd(T, m, O), f ? f.alias.push(_) : (A = A || _, A !== _ && A.alias.push(_), b && p.name && !so(_) && o(p.name)), ll(_) && l(_), P.children) {
        const le = P.children;
        for (let ne = 0; ne < le.length; ne++) i(le[ne], _, f && f.children[ne]);
      }
      f = f || _;
    }
    return A ? () => {
      o(A);
    } : Bt;
  }
  function o(p) {
    if (sl(p)) {
      const m = s.get(p);
      m && (s.delete(p), t.splice(t.indexOf(m), 1), m.children.forEach(o), m.alias.forEach(o));
    } else {
      const m = t.indexOf(p);
      m > -1 && (t.splice(m, 1), p.record.name && s.delete(p.record.name), p.children.forEach(o), p.alias.forEach(o));
    }
  }
  function r() {
    return t;
  }
  function l(p) {
    const m = np(p, t);
    t.splice(m, 0, p), p.record.name && !so(p) && s.set(p.record.name, p);
  }
  function d(p, m) {
    let f, b = {}, P, O;
    if ("name" in p && p.name) {
      if (f = s.get(p.name), !f) throw vt(we.MATCHER_NOT_FOUND, { location: p });
      O = f.record.name, b = ue(no(m.params, f.keys.filter((A) => !A.optional).concat(f.parent ? f.parent.keys.filter((A) => A.optional) : []).map((A) => A.name)), p.params && no(p.params, f.keys.map((A) => A.name))), P = f.stringify(b);
    } else if (p.path != null) P = p.path, f = t.find((A) => A.re.test(P)), f && (b = f.parse(P), O = f.record.name);
    else {
      if (f = m.name ? s.get(m.name) : t.find((A) => A.re.test(m.path)), !f) throw vt(we.MATCHER_NOT_FOUND, { location: p, currentLocation: m });
      O = f.record.name, b = ue({}, m.params, p.params), P = f.stringify(b);
    }
    const w = [];
    let _ = f;
    for (; _; ) w.unshift(_.record), _ = _.parent;
    return { name: O, path: P, params: b, matched: w, meta: ep(w) };
  }
  e.forEach((p) => i(p));
  function c() {
    t.length = 0, s.clear();
  }
  return { addRoute: i, resolve: d, removeRoute: o, clearRoutes: c, getRoutes: r, getRecordMatcher: a };
}
function no(e, n) {
  const t = {};
  for (const s of n) s in e && (t[s] = e[s]);
  return t;
}
function to(e) {
  const n = { path: e.path, redirect: e.redirect, name: e.name, meta: e.meta || {}, aliasOf: e.aliasOf, beforeEnter: e.beforeEnter, props: Zd(e), children: e.children || [], instances: {}, leaveGuards: /* @__PURE__ */ new Set(), updateGuards: /* @__PURE__ */ new Set(), enterCallbacks: {}, components: "components" in e ? e.components || null : e.component && { default: e.component } };
  return Object.defineProperty(n, "mods", { value: {} }), n;
}
function Zd(e) {
  const n = {}, t = e.props || false;
  if ("component" in e) n.default = t;
  else for (const s in e.components) n[s] = typeof t == "object" ? t[s] : t;
  return n;
}
function so(e) {
  for (; e; ) {
    if (e.record.aliasOf) return true;
    e = e.parent;
  }
  return false;
}
function ep(e) {
  return e.reduce((n, t) => ue(n, t.meta), {});
}
function np(e, n) {
  let t = 0, s = n.length;
  for (; t !== s; ) {
    const i = t + s >> 1;
    rl(e, n[i]) < 0 ? s = i : t = i + 1;
  }
  const a = tp(e);
  return a && (s = n.lastIndexOf(a, s - 1)), s;
}
function tp(e) {
  let n = e;
  for (; n = n.parent; ) if (ll(n) && rl(e, n) === 0) return n;
}
function ll({ record: e }) {
  return !!(e.name || e.components && Object.keys(e.components).length || e.redirect);
}
function ao(e) {
  const n = sn(Ks), t = sn(ai), s = Y(() => {
    const l = D(e.to);
    return n.resolve(l);
  }), a = Y(() => {
    const { matched: l } = s.value, { length: d } = l, c = l[d - 1], p = t.matched;
    if (!c || !p.length) return -1;
    const m = p.findIndex(_t.bind(null, c));
    if (m > -1) return m;
    const f = io(l[d - 2]);
    return d > 1 && io(c) === f && p[p.length - 1].path !== f ? p.findIndex(_t.bind(null, l[d - 2])) : m;
  }), i = Y(() => a.value > -1 && op(t.params, s.value.params)), o = Y(() => a.value > -1 && a.value === t.matched.length - 1 && tl(t.params, s.value.params));
  function r(l = {}) {
    if (ip(l)) {
      const d = n[D(e.replace) ? "replace" : "push"](D(e.to)).catch(Bt);
      return e.viewTransition && typeof document < "u" && "startViewTransition" in document && document.startViewTransition(() => d), d;
    }
    return Promise.resolve();
  }
  return { route: s, href: Y(() => s.value.href), isActive: i, isExactActive: o, navigate: r };
}
function sp(e) {
  return e.length === 1 ? e[0] : e;
}
const ap = De({ name: "RouterLink", compatConfig: { MODE: 3 }, props: { to: { type: [String, Object], required: true }, replace: Boolean, activeClass: String, exactActiveClass: String, custom: Boolean, ariaCurrentValue: { type: String, default: "page" }, viewTransition: Boolean }, useLink: ao, setup(e, { slots: n }) {
  const t = xt(ao(e)), { options: s } = sn(Ks), a = Y(() => ({ [oo(e.activeClass, s.linkActiveClass, "router-link-active")]: t.isActive, [oo(e.exactActiveClass, s.linkExactActiveClass, "router-link-exact-active")]: t.isExactActive }));
  return () => {
    const i = n.default && sp(n.default(t));
    return e.custom ? i : Kr("a", { "aria-current": t.isExactActive ? e.ariaCurrentValue : null, href: t.href, onClick: t.navigate, class: a.value }, i);
  };
} }), Un = ap;
function ip(e) {
  if (!(e.metaKey || e.altKey || e.ctrlKey || e.shiftKey) && !e.defaultPrevented && !(e.button !== void 0 && e.button !== 0)) {
    if (e.currentTarget && e.currentTarget.getAttribute) {
      const n = e.currentTarget.getAttribute("target");
      if (/\b_blank\b/i.test(n)) return;
    }
    return e.preventDefault && e.preventDefault(), true;
  }
}
function op(e, n) {
  for (const t in n) {
    const s = n[t], a = e[t];
    if (typeof s == "string") {
      if (s !== a) return false;
    } else if (!rn(a) || a.length !== s.length || s.some((i, o) => i.valueOf() !== a[o].valueOf())) return false;
  }
  return true;
}
function io(e) {
  return e ? e.aliasOf ? e.aliasOf.path : e.path : "";
}
const oo = (e, n, t) => e ?? n ?? t, rp = De({ name: "RouterView", inheritAttrs: false, props: { name: { type: String, default: "default" }, route: Object }, compatConfig: { MODE: 3 }, setup(e, { attrs: n, slots: t }) {
  const s = sn(Ca), a = Y(() => e.route || s.value), i = sn(Qi, 0), o = Y(() => {
    let d = D(i);
    const { matched: c } = a.value;
    let p;
    for (; (p = c[d]) && !p.components; ) d++;
    return d;
  }), r = Y(() => a.value.matched[o.value]);
  _s(Qi, Y(() => o.value + 1)), _s(qd, r), _s(Ca, a);
  const l = W();
  return kn(() => [l.value, r.value, e.name], ([d, c, p], [m, f, b]) => {
    c && (c.instances[p] = d, f && f !== c && d && d === m && (c.leaveGuards.size || (c.leaveGuards = f.leaveGuards), c.updateGuards.size || (c.updateGuards = f.updateGuards))), d && c && (!f || !_t(c, f) || !m) && (c.enterCallbacks[p] || []).forEach((P) => P(d));
  }, { flush: "post" }), () => {
    const d = a.value, c = e.name, p = r.value, m = p && p.components[c];
    if (!m) return ro(t.default, { Component: m, route: d });
    const f = p.props[c], b = f ? f === true ? d.params : typeof f == "function" ? f(d) : f : null, O = Kr(m, ue({}, b, n, { onVnodeUnmounted: (w) => {
      w.component.isUnmounted && (p.instances[c] = null);
    }, ref: l }));
    return ro(t.default, { Component: O, route: d }) || O;
  };
} });
function ro(e, n) {
  if (!e) return null;
  const t = e(n);
  return t.length === 1 ? t[0] : t;
}
const ul = rp;
function lp(e) {
  const n = Xd(e.routes, e), t = e.parseQuery || Od, s = e.stringifyQuery || Yi, a = e.history, i = Ct(), o = Ct(), r = Ct(), l = du(Mn);
  let d = Mn;
  rt && e.scrollBehavior && "scrollRestoration" in history && (history.scrollRestoration = "manual");
  const c = ra.bind(null, (z) => "" + z), p = ra.bind(null, vd), m = ra.bind(null, es);
  function f(z, F) {
    let L, B;
    return sl(z) ? (L = n.getRecordMatcher(z), B = F) : B = z, n.addRoute(B, L);
  }
  function b(z) {
    const F = n.getRecordMatcher(z);
    F && n.removeRoute(F);
  }
  function P() {
    return n.getRoutes().map((z) => z.record);
  }
  function O(z) {
    return !!n.getRecordMatcher(z);
  }
  function w(z, F) {
    if (F = ue({}, F || l.value), typeof z == "string") {
      const k = la(t, z, F.path), R = n.resolve({ path: k.path }, F), I = a.createHref(k.fullPath);
      return ue(k, R, { params: m(R.params), hash: es(k.hash), redirectedFrom: void 0, href: I });
    }
    let L;
    if (z.path != null) L = ue({}, z, { path: la(t, z.path, F.path).path });
    else {
      const k = ue({}, z.params);
      for (const R in k) k[R] == null && delete k[R];
      L = ue({}, z, { params: p(k) }), F.params = p(F.params);
    }
    const B = n.resolve(L, F), te = z.hash || "";
    B.params = c(m(B.params));
    const h = wd(s, ue({}, z, { hash: md(te), path: B.path })), g = a.createHref(h);
    return ue({ fullPath: h, hash: te, query: s === Yi ? Nd(z.query) : z.query || {} }, B, { redirectedFrom: void 0, href: g });
  }
  function _(z) {
    return typeof z == "string" ? la(t, z, l.value.path) : ue({}, z);
  }
  function A(z, F) {
    if (d !== z) return vt(we.NAVIGATION_CANCELLED, { from: F, to: z });
  }
  function T(z) {
    return ne(z);
  }
  function H(z) {
    return T(ue(_(z), { replace: true }));
  }
  function le(z, F) {
    const L = z.matched[z.matched.length - 1];
    if (L && L.redirect) {
      const { redirect: B } = L;
      let te = typeof B == "function" ? B(z, F) : B;
      return typeof te == "string" && (te = te.includes("?") || te.includes("#") ? te = _(te) : { path: te }, te.params = {}), ue({ query: z.query, hash: z.hash, params: te.path != null ? {} : z.params }, te);
    }
  }
  function ne(z, F) {
    const L = d = w(z), B = l.value, te = z.state, h = z.force, g = z.replace === true, k = le(L, B);
    if (k) return ne(ue(_(k), { state: typeof k == "object" ? ue({}, te, k.state) : te, force: h, replace: g }), F || L);
    const R = L;
    R.redirectedFrom = F;
    let I;
    return !h && bd(s, B, L) && (I = vt(we.NAVIGATION_DUPLICATED, { to: R, from: B }), U(B, B, true, false)), (I ? Promise.resolve(I) : Se(R, B)).catch((C) => An(C) ? An(C, we.NAVIGATION_GUARD_REDIRECT) ? C : N(C) : ae(C, R, B)).then((C) => {
      if (C) {
        if (An(C, we.NAVIGATION_GUARD_REDIRECT)) return ne(ue({ replace: g }, _(C.to), { state: typeof C.to == "object" ? ue({}, te, C.to.state) : te, force: h }), F || R);
      } else C = He(R, B, true, g, te);
      return Ye(R, B, C), C;
    });
  }
  function $e(z, F) {
    const L = A(z, F);
    return L ? Promise.reject(L) : Promise.resolve();
  }
  function Ee(z) {
    const F = Ue.values().next().value;
    return F && typeof F.runWithContext == "function" ? F.runWithContext(z) : z();
  }
  function Se(z, F) {
    let L;
    const [B, te, h] = Md(z, F);
    L = ca(B.reverse(), "beforeRouteLeave", z, F);
    for (const k of B) k.leaveGuards.forEach((R) => {
      L.push(Vn(R, z, F));
    });
    const g = $e.bind(null, z, F);
    return L.push(g), Fe(L).then(() => {
      L = [];
      for (const k of i.list()) L.push(Vn(k, z, F));
      return L.push(g), Fe(L);
    }).then(() => {
      L = ca(te, "beforeRouteUpdate", z, F);
      for (const k of te) k.updateGuards.forEach((R) => {
        L.push(Vn(R, z, F));
      });
      return L.push(g), Fe(L);
    }).then(() => {
      L = [];
      for (const k of h) if (k.beforeEnter) if (rn(k.beforeEnter)) for (const R of k.beforeEnter) L.push(Vn(R, z, F));
      else L.push(Vn(k.beforeEnter, z, F));
      return L.push(g), Fe(L);
    }).then(() => (z.matched.forEach((k) => k.enterCallbacks = {}), L = ca(h, "beforeRouteEnter", z, F, Ee), L.push(g), Fe(L))).then(() => {
      L = [];
      for (const k of o.list()) L.push(Vn(k, z, F));
      return L.push(g), Fe(L);
    }).catch((k) => An(k, we.NAVIGATION_CANCELLED) ? k : Promise.reject(k));
  }
  function Ye(z, F, L) {
    r.list().forEach((B) => Ee(() => B(z, F, L)));
  }
  function He(z, F, L, B, te) {
    const h = A(z, F);
    if (h) return h;
    const g = F === Mn, k = rt ? history.state : {};
    L && (B || g ? a.replace(z.fullPath, ue({ scroll: g && k && k.scroll }, te)) : a.push(z.fullPath, te)), l.value = z, U(z, F, L, g), N();
  }
  let Le;
  function en() {
    Le || (Le = a.listen((z, F, L) => {
      if (!Sn.listening) return;
      const B = w(z), te = le(B, Sn.currentRoute.value);
      if (te) {
        ne(ue(te, { replace: true, force: true }), B).catch(Bt);
        return;
      }
      d = B;
      const h = l.value;
      rt && Rd(Ji(h.fullPath, L.delta), Bs()), Se(B, h).catch((g) => An(g, we.NAVIGATION_ABORTED | we.NAVIGATION_CANCELLED) ? g : An(g, we.NAVIGATION_GUARD_REDIRECT) ? (ne(ue(_(g.to), { force: true }), B).then((k) => {
        An(k, we.NAVIGATION_ABORTED | we.NAVIGATION_DUPLICATED) && !L.delta && L.type === Ea.pop && a.go(-1, false);
      }).catch(Bt), Promise.reject()) : (L.delta && a.go(-L.delta, false), ae(g, B, h))).then((g) => {
        g = g || He(B, h, false), g && (L.delta && !An(g, we.NAVIGATION_CANCELLED) ? a.go(-L.delta, false) : L.type === Ea.pop && An(g, we.NAVIGATION_ABORTED | we.NAVIGATION_DUPLICATED) && a.go(-1, false)), Ye(B, h, g);
      }).catch(Bt);
    }));
  }
  let un = Ct(), ke = Ct(), oe;
  function ae(z, F, L) {
    N(z);
    const B = ke.list();
    return B.length ? B.forEach((te) => te(z, F, L)) : console.error(z), Promise.reject(z);
  }
  function Oe() {
    return oe && l.value !== Mn ? Promise.resolve() : new Promise((z, F) => {
      un.add([z, F]);
    });
  }
  function N(z) {
    return oe || (oe = !z, en(), un.list().forEach(([F, L]) => z ? L(z) : F()), un.reset()), z;
  }
  function U(z, F, L, B) {
    const { scrollBehavior: te } = e;
    if (!rt || !te) return Promise.resolve();
    const h = !L && zd(Ji(z.fullPath, 0)) || (B || !L) && history.state && history.state.scroll || null;
    return ss().then(() => te(z, F, h)).then((g) => g && Cd(g)).catch((g) => ae(g, z, F));
  }
  const K = (z) => a.go(z);
  let ie;
  const Ue = /* @__PURE__ */ new Set(), Sn = { currentRoute: l, listening: true, addRoute: f, removeRoute: b, clearRoutes: n.clearRoutes, hasRoute: O, getRoutes: P, resolve: w, options: e, push: T, replace: H, go: K, back: () => K(-1), forward: () => K(1), beforeEach: i.add, beforeResolve: o.add, afterEach: r.add, onError: ke.add, isReady: Oe, install(z) {
    z.component("RouterLink", Un), z.component("RouterView", ul), z.config.globalProperties.$router = Sn, Object.defineProperty(z.config.globalProperties, "$route", { enumerable: true, get: () => D(l) }), rt && !ie && l.value === Mn && (ie = true, T(a.location).catch((B) => {
    }));
    const F = {};
    for (const B in Mn) Object.defineProperty(F, B, { get: () => l.value[B], enumerable: true });
    z.provide(Ks, Sn), z.provide(ai, pr(F)), z.provide(Ca, l);
    const L = z.unmount;
    Ue.add(z), z.unmount = function() {
      Ue.delete(z), Ue.size < 1 && (d = Mn, Le && Le(), Le = null, l.value = Mn, ie = false, oe = false), L();
    };
  } };
  function Fe(z) {
    return z.reduce((F, L) => F.then(() => Ee(L)), Promise.resolve());
  }
  return Sn;
}
function cl() {
  return sn(Ks);
}
function up(e) {
  return sn(ai);
}
const cp = { class: "app-shell" }, dp = { class: "topbar", role: "banner" }, pp = { class: "nav", "aria-label": "\u4E3B\u5BFC\u822A" }, hp = { class: "main", id: "main", tabindex: "-1" }, fp = De({ __name: "App", setup(e) {
  return (n, t) => (x(), j("div", cp, [u("header", dp, [_e(D(Un), { class: "brand", to: "/", "aria-label": "\u65E0\u540D\u6740 Next \u9996\u9875" }, { default: Ln(() => [...t[0] || (t[0] = [se(" \u65E0\u540D\u6740 ", -1), u("span", { class: "accent" }, "Next", -1)])]), _: 1 }), u("nav", pp, [_e(D(Un), { to: "/" }, { default: Ln(() => [...t[1] || (t[1] = [se("\u5927\u5385", -1)])]), _: 1 }), _e(D(Un), { to: "/battle" }, { default: Ln(() => [...t[2] || (t[2] = [se("\u5BF9\u5C40", -1)])]), _: 1 }), _e(D(Un), { to: "/extensions" }, { default: Ln(() => [...t[3] || (t[3] = [se("\u6269\u5C55", -1)])]), _: 1 }), _e(D(Un), { to: "/settings" }, { default: Ln(() => [...t[4] || (t[4] = [se("\u8BBE\u7F6E", -1)])]), _: 1 }), _e(D(Un), { to: "/replay" }, { default: Ln(() => [...t[5] || (t[5] = [se("\u5F55\u50CF", -1)])]), _: 1 }), _e(D(Un), { to: "/collection" }, { default: Ln(() => [...t[6] || (t[6] = [se("\u6B66\u5C06\u5E93", -1)])]), _: 1 })])]), u("main", hp, [_e(D(ul))])]));
} });
async function jt(e, n = {}, t) {
  return window.__TAURI_INTERNALS__.invoke(e, n, t);
}
function mp(e, n = "asset") {
  return window.__TAURI_INTERNALS__.convertFileSrc(e, n);
}
function jn() {
  return typeof window < "u" && "__TAURI_INTERNALS__" in window;
}
function gp() {
  return jn() ? jt("platform_info") : Promise.resolve(null);
}
function lo(e) {
  return jn() ? jt("set_match_frozen", { frozen: e }) : Promise.resolve();
}
function _p() {
  return jn() ? jt("content_bundle") : Promise.resolve(null);
}
function vp(e, n) {
  return jn() ? jt("content_catalog_page", { offset: e, limit: n }) : Promise.resolve(null);
}
async function yp(e) {
  if (!jn()) return null;
  const n = await jt("resolve_content_asset", { namespace: e.namespace, packageId: e.package_id, assetId: e.asset_id });
  return mp(n);
}
const kp = { class: "stack" }, wp = { class: "panel hero" }, bp = { key: 0, class: "shell-badge" }, xp = { class: "row form" }, $p = ["value"], jp = { class: "field" }, Sp = { class: "panel" }, Ep = { class: "online-grid" }, Ap = { class: "wide" }, Cp = ["value"], Rp = { class: "row online-actions" }, zp = { key: 0, class: "error", role: "alert" }, Ip = De({ __name: "HomeView", setup(e) {
  const n = cl(), t = W(4), s = W(42), a = W("prototype"), i = W(`player-${Math.floor(Math.random() * 900 + 100)}`), o = W(""), r = W(1), l = W(30), d = W(b()), c = W(""), p = W(null);
  $t(async () => {
    p.value = await gp();
  });
  function m() {
    n.push({ path: "/battle", query: { players: String(t.value), seed: String(s.value), ...a.value === "identity" ? { mode: "identity" } : {} } });
  }
  function f(O) {
    if (c.value = "", !d.value.trim()) {
      c.value = "\u8BF7\u586B\u5199 WebSocket \u5730\u5740";
      return;
    }
    if (O !== "create" && r.value < 1) {
      c.value = "\u623F\u95F4\u53F7\u5FC5\u987B\u5927\u4E8E 0";
      return;
    }
    sessionStorage.setItem("noname-next.online-intent", JSON.stringify({ action: O, wsUrl: d.value, displayName: i.value, password: o.value, roomId: r.value, players: t.value, seed: s.value, choiceTimeoutSecs: l.value })), n.push({ path: "/battle", query: { mode: "online", action: O, ...O === "create" ? {} : { room: String(r.value) } } });
  }
  function b() {
    if (typeof window > "u") return "ws://127.0.0.1:7700/ws";
    const O = window.location.protocol === "https:" ? "wss:" : "ws:", w = window.location.hostname || "127.0.0.1";
    return `${O}//${w}:7700/ws`;
  }
  const P = Y(() => r.value > 0 ? `#${r.value}` : "\u2014");
  return (O, w) => (x(), j("div", kp, [u("section", wp, [w[15] || (w[15] = u("h1", null, "\u786E\u5B9A\u6027\u8EAB\u4EFD\u5C40", -1)), w[16] || (w[16] = u("p", { class: "muted" }, [se(" \u9ED8\u8BA4\u539F\u751F\u5F15\u64CE\u540E\u7AEF\uFF08engine-native-wasm \u7A84\u58F3\uFF0CWorker \u5185\u8FD0\u884C\uFF09\u3002\u754C\u9762\u53EA\u6D88\u8D39\u89C2\u5BDF\u8005\u6295\u5F71\uFF0C\u4E0D\u6301\u6709\u5B8C\u6574\u6743\u5A01\u72B6\u6001\u3002\u53EF\u7528 "), u("code", null, "VITE_ENGINE_BACKEND=legacy"), se(" \u56DE\u9000\u65E7\u7EBF\u3002 ")], -1)), p.value ? (x(), j("p", bp, " Tauri 2 \xB7 " + $(p.value.os) + " \xB7 " + $(p.value.arch), 1)) : V("", true), u("div", xp, [u("label", null, [w[11] || (w[11] = se(" \u4EBA\u6570 ", -1)), ye(u("select", { "onUpdate:modelValue": w[0] || (w[0] = (_) => t.value = _), "aria-label": "\u73A9\u5BB6\u4EBA\u6570" }, [(x(), j(ee, null, ve([2, 3, 4, 5, 6, 7, 8], (_) => u("option", { key: _, value: _ }, $(_), 9, $p)), 64))], 512), [[vn, t.value, void 0, { number: true }]])]), u("label", null, [w[12] || (w[12] = se(" \u79CD\u5B50 ", -1)), ye(u("input", { "onUpdate:modelValue": w[1] || (w[1] = (_) => s.value = _), type: "number", min: "0", "aria-label": "\u968F\u673A\u79CD\u5B50" }, null, 512), [[Wn, s.value, void 0, { number: true }]])]), u("label", jp, [w[14] || (w[14] = u("span", null, "\u6A21\u5F0F", -1)), ye(u("select", { "onUpdate:modelValue": w[2] || (w[2] = (_) => a.value = _), "aria-label": "\u5BF9\u5C40\u6A21\u5F0F" }, [...w[13] || (w[13] = [u("option", { value: "prototype" }, "\u539F\u578B\uFF08\u552F\u4E00\u5B58\u6D3B\u8005\uFF09", -1), u("option", { value: "identity" }, "\u8EAB\u4EFD\u5C40\uFF08\u4E3B\u5FE0/\u53CD\u8D3C/\u5185\u5978\uFF09", -1)])], 512), [[vn, a.value]])]), u("button", { class: "primary", type: "button", onClick: m }, " \u672C\u5730\u5F00\u5C40 ")])]), u("section", Sp, [w[21] || (w[21] = u("h2", null, "\u8054\u673A\u5927\u5385", -1)), w[22] || (w[22] = u("p", { class: "muted" }, " \u901A\u8FC7 game-server WebSocket \u52A0\u5165\u6743\u5A01\u623F\u95F4\u3002\u6D4F\u89C8\u5668\u4EC5\u63D0\u4EA4\u547D\u4EE4\u5E76\u63A5\u6536\u81EA\u5DF1\u7684\u89C2\u5BDF\u8005\u6295\u5F71\u3002 ", -1)), u("div", Ep, [u("label", Ap, [w[17] || (w[17] = se(" \u670D\u52A1\u7AEF ", -1)), ye(u("input", { "onUpdate:modelValue": w[3] || (w[3] = (_) => d.value = _), type: "url", autocomplete: "url" }, null, 512), [[Wn, d.value, void 0, { trim: true }]])]), u("label", null, [w[18] || (w[18] = se(" \u6635\u79F0 ", -1)), ye(u("input", { "onUpdate:modelValue": w[4] || (w[4] = (_) => i.value = _), maxlength: "24", autocomplete: "nickname" }, null, 512), [[Wn, i.value, void 0, { trim: true }]])]), u("label", null, [w[19] || (w[19] = se(" \u9009\u62E9\u65F6\u9650 ", -1)), ye(u("select", { "onUpdate:modelValue": w[5] || (w[5] = (_) => l.value = _), "aria-label": "\u9009\u62E9\u65F6\u9650" }, [(x(), j(ee, null, ve([10, 15, 30, 60, 90], (_) => u("option", { key: _, value: _ }, $(_) + " \u79D2", 9, Cp)), 64))], 512), [[vn, l.value, void 0, { number: true }]])]), u("label", null, [w[20] || (w[20] = se(" \u623F\u95F4\u5BC6\u7801 ", -1)), ye(u("input", { "onUpdate:modelValue": w[6] || (w[6] = (_) => o.value = _), type: "password", autocomplete: "current-password" }, null, 512), [[Wn, o.value]])]), u("label", null, [se(" \u623F\u95F4\u53F7 " + $(P.value) + " ", 1), ye(u("input", { "onUpdate:modelValue": w[7] || (w[7] = (_) => r.value = _), type: "number", min: "1", inputmode: "numeric" }, null, 512), [[Wn, r.value, void 0, { number: true }]])])]), u("div", Rp, [u("button", { class: "primary", type: "button", onClick: w[8] || (w[8] = (_) => f("create")) }, "\u521B\u5EFA\u5E76\u5165\u5EA7"), u("button", { type: "button", onClick: w[9] || (w[9] = (_) => f("join")) }, "\u52A0\u5165\u623F\u95F4"), u("button", { type: "button", onClick: w[10] || (w[10] = (_) => f("spectate")) }, "\u65C1\u89C2")]), c.value ? (x(), j("p", zp, $(c.value), 1)) : V("", true)]), w[23] || (w[23] = u("section", { class: "panel" }, [u("h2", null, "\u67B6\u6784\u8981\u70B9"), u("ul", { class: "muted list" }, [u("li", null, "\u4E3B\u7EBF\u7A0B\uFF1AVue Shell + DOM \u6218\u6597\u5C42 + \u52A8\u753B\u65F6\u95F4\u7EBF"), u("li", null, "\u672C\u5730\uFF1AWorker + \u539F\u751F\u7A84\u58F3\uFF08\u9ED8\u8BA4\uFF09/ \u65E7\u7EBF WASM\uFF1B\u8054\u673A\uFF1AWebSocket + \u6743\u5A01\u670D\u52A1\u5668"), u("li", null, "\u6269\u5C55\u5305\u7BA1\u7406\uFF1A\u5B89\u88C5 / \u542F\u505C / \u5347\u7EA7 / \u5378\u8F7D\uFF08\u5BFC\u822A\u300C\u6269\u5C55\u300D\uFF09"), u("li", null, "\u652F\u6301\u81EA\u52A8\u6B65\u8FDB\u3001\u5B58\u8BFB\u6863\u3001\u500D\u901F\u3001\u8DF3\u8FC7\u52A8\u753B\u3001\u51CF\u5C11\u52A8\u6001\u6548\u679C")])], -1))]));
} }), ln = (e, n) => {
  const t = e.__vccOpts || e;
  for (const [s, a] of n) t[s] = a;
  return t;
}, Pp = ln(Ip, [["__scopeId", "data-v-bb2f538c"]]), uo = { confirm: "Confirm", select_cards: "SelectCards", select_players: "SelectTargets", respond_card: "Custom", pick_tag: "Custom" };
function Tp(e) {
  if (e === void 0) return [null, null];
  if (!Array.isArray(e) || e.length !== 2 || !e.every((n) => n === null || typeof n == "object" && !Array.isArray(n) && Number.isInteger(n.def) && n.def >= 0 && n.def <= 4294967295 && typeof n.name == "string" && typeof n.faction == "string" && typeof n.revealed == "boolean")) throw new Error("native: generals metadata invalid");
  return e;
}
function Op(e) {
  if (e.prompt != null && typeof e.prompt != "string") throw new Error("native: prompt metadata invalid");
  if (e.native_kind != null && (typeof e.native_kind != "string" || !Object.hasOwn(uo, e.native_kind))) throw new Error("native: native_kind metadata invalid");
  const n = e.native_kind;
  return { ...e.prompt == null ? {} : { prompt: e.prompt }, ...n == null ? {} : { native_kind: n }, kind: n ? uo[n] : e.kind ?? "Custom" };
}
const co = { confirm: "\u786E\u8BA4", select_cards: "\u9009\u62E9\u5361\u724C", select_players: "\u9009\u62E9\u89D2\u8272", respond_card: "\u54CD\u5E94\u51FA\u724C", pick_tag: "\u9009\u62E9\u9009\u9879", Confirm: "\u786E\u8BA4", SelectCards: "\u9009\u62E9\u5361\u724C", SelectTargets: "\u9009\u62E9\u89D2\u8272", Custom: "\u9009\u62E9\u9009\u9879", OrderTriggers: "\u6392\u5217\u6280\u80FD", DiscardToLimit: "\u5F03\u724C\u81F3\u624B\u724C\u4E0A\u9650" }, po = { trigger_confirm: "\u662F\u5426\u53D1\u52A8\u6280\u80FD\uFF1F", respond_shan: "\u8BF7\u6253\u51FA\u95EA", respond_sha: "\u8BF7\u6253\u51FA\u6740", aoe_response: "\u8BF7\u54CD\u5E94\u7FA4\u4F53\u9526\u56CA", dying_save: "\u8BF7\u9009\u62E9\u6551\u63F4\u724C", discard_to_limit: "\u8BF7\u5F03\u724C\u81F3\u624B\u724C\u4E0A\u9650", tuxi_target: "\u7A81\u88AD\uFF1A\u9009\u62E9\u83B7\u5F97\u624B\u724C\u7684\u89D2\u8272", guanxing_arrange: "\u89C2\u661F\uFF1A\u6392\u5217\u724C\u5806\u9876\u7684\u5361\u724C", xunxun_arrange: "\u6042\u6042\uFF1A\u9009\u62E9\u83B7\u5F97\u7684\u5361\u724C", jijie_give_target: "\u673A\u6377\uFF1A\u9009\u62E9\u83B7\u5F97\u6B64\u724C\u7684\u89D2\u8272", gaiming_pick: "\u6539\u547D\uFF1A\u9009\u62E9\u5224\u5B9A\u724C", haotianta_pick: "\u660A\u5929\u5854\uFF1A\u9009\u62E9\u5224\u5B9A\u724C", liaoyuan_discard: "\u71CE\u539F\uFF1A\u9009\u62E9\u5F03\u7F6E\u7684\u5361\u724C", fanjian_target: "\u53CD\u95F4\uFF1A\u9009\u62E9\u76EE\u6807\u89D2\u8272", fanjian_suit: "\u53CD\u95F4\uFF1A\u9009\u62E9\u82B1\u8272", fanjian_pick: "\u53CD\u95F4\uFF1A\u9009\u62E9\u83B7\u5F97\u7684\u624B\u724C" };
function Ra(e) {
  const n = e.native_kind ?? e.kind ?? "Custom";
  return Object.hasOwn(co, n) ? co[n] : "\u9009\u62E9\u9009\u9879";
}
function Np(e) {
  return e.prompt && Object.hasOwn(po, e.prompt) ? po[e.prompt] : Ra(e);
}
function dl(e, n) {
  for (const t of e.seats ?? []) Tp(t.generals);
  if (n && (Op(n), n.deadline_unix_ms != null && (!Number.isSafeInteger(n.deadline_unix_ms) || n.deadline_unix_ms <= 0))) throw new Error("native: choice deadline invalid");
}
const ht = (e) => new Promise((n) => setTimeout(n, e)), Gs = "\u81EA\u52A8\u4E00\u6B65", qp = "\u8FDE\u7EED\u81EA\u52A8", Js = /终局/;
let pl = [], za = 0;
function os(e) {
  return Array.from(document.querySelectorAll("button")).find((n) => (n.innerText || "").trim() === e) ?? null;
}
function ho() {
  return new Promise((e) => {
    const n = () => {
      const a = os(Gs);
      return a ? Js.test(document.body.innerText) ? "finished" : a.disabled ? null : "ready" : "gone";
    }, t = n();
    if (t) {
      e(t);
      return;
    }
    const s = new MutationObserver(() => {
      const a = n();
      a && (s.disconnect(), e(a));
    });
    s.observe(document.body, { attributes: true, subtree: true, childList: true, attributeFilter: ["disabled"] }), setTimeout(() => {
      s.disconnect(), e("timeout");
    }, 15e3);
  });
}
function Ia() {
  return window.__nonamePerf ?? null;
}
function Mp() {
  const e = Ia(), n = document.body.innerText.match(/终局[^\n]*/) ?? document.body.innerText.match(/rev \d+[^\n]*/);
  return { frames: (e == null ? void 0 : e.frames()) ?? [], longTasks: (e == null ? void 0 : e.longTasks()) ?? [], eventTimings: (e == null ? void 0 : e.eventTimings()) ?? [], interactions: (e == null ? void 0 : e.interactions()) ?? [], status: (n == null ? void 0 : n[0]) ?? null, ua: navigator.userAgent, dpr: window.devicePixelRatio, viewport: [window.innerWidth, window.innerHeight], href: location.href, engine_transport: globalThis.__engineTransport ?? null, memory: null, memory_series: [] };
}
async function Dp() {
  const e = performance;
  try {
    if (e.memory) {
      const n = e.memory;
      return { used: n.usedJSHeapSize, total: n.totalJSHeapSize, limit: n.jsHeapSizeLimit, api: "performance.memory" };
    }
    if (e.measureUserAgentSpecificMemory && crossOriginIsolated) {
      const n = await e.measureUserAgentSpecificMemory();
      return { used: n.bytes, total: n.bytes, limit: 0, api: "measureUserAgentSpecificMemory" };
    }
  } catch {
  }
  return null;
}
async function Lp(e, n) {
  try {
    await fetch(e, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(n) });
  } catch {
  }
}
async function Up(e, n) {
  for (let t = 0; t < e; t++) {
    const s = os(Gs);
    if (!s || Js.test(document.body.innerText)) return "finished";
    if (s.disabled) {
      if (await ho() === "finished") return "finished";
      continue;
    }
    if (s.click(), await ho() === "finished") return "finished";
    await ht(n);
  }
  return "max_steps";
}
async function Fp() {
  var _a2;
  const e = nn("loop") === "1", n = Number(nn("duration") ?? 0), t = Date.now() + (n > 0 ? n : 6e5), s = () => Js.test(document.body.innerText), a = [], i = performance.memory, o = performance.measureUserAgentSpecificMemory, r = async () => {
    if (i) {
      a.push({ t_ms: Date.now() - za, bytes: i.usedJSHeapSize });
      return;
    }
    if (!(!o || !crossOriginIsolated)) try {
      const l = await o();
      a.push({ t_ms: Date.now() - za, bytes: l.bytes });
    } catch {
    }
  };
  for (; ; ) {
    const l = os(qp);
    l && l.click();
    let d = 0;
    for (; Date.now() < t && !s(); ) await ht(1e3), Date.now() - d > 3e4 && (d = Date.now(), await r());
    if (!e || Date.now() >= t) return pl = a, s() ? "finished" : "auto_timeout";
    (_a2 = Array.from(document.querySelectorAll("button")).find((p) => (p.innerText || "").includes("\u91CD\u65B0\u5F00\u5C40"))) == null ? void 0 : _a2.click(), await ht(1500);
  }
}
function Vp(e, n) {
  return new Promise((t) => {
    let s = 0;
    const a = (r) => {
      window.removeEventListener("click", i, true), t(r);
    }, i = (r) => {
      const l = r.target, d = os(Gs);
      d && (l === d || d.contains(l)) && (s += 1, s >= e && setTimeout(() => a("manual_steps"), 800));
    };
    window.addEventListener("click", i, true);
    const o = setInterval(() => {
      Js.test(document.body.innerText) && (clearInterval(o), a("finished"));
    }, 500);
    setTimeout(() => {
      clearInterval(o), a(s > 0 ? "manual_timeout" : "manual_no_input");
    }, n);
  });
}
function nn(e) {
  const n = location.hash.split("?")[1], t = n ? new URLSearchParams(n) : null;
  return new URLSearchParams(location.search).get(e) ?? (t == null ? void 0 : t.get(e)) ?? null;
}
function Hp() {
  if (typeof window > "u") return;
  const e = window;
  if (e.__nonameAutoDriveStarted) return;
  const n = nn("autodrive");
  if (!n) return;
  e.__nonameAutoDriveStarted = true;
  const t = nn("report"), s = Number(nn("steps") ?? 1500), a = Number(nn("gap") ?? 50), i = Number(nn("timeout") ?? 12e4);
  (async () => {
    var _a2;
    for (let p = 0; p < 150; p++) {
      const m = os(Gs);
      if (m && Ia() && !m.disabled) break;
      await ht(200);
    }
    (_a2 = Ia()) == null ? void 0 : _a2.reset(), await ht(300);
    const r = Date.now();
    za = r;
    let l = "unknown";
    n === "click" ? l = await Up(s, a) : n === "auto" ? l = await Fp() : n === "manual" && (l = await Vp(s, i)), await ht(600);
    const d = Mp();
    d.memory = await Dp(), d.memory_series = pl, t && await Lp(t, { ...d, end_state: l, wall_ms: Date.now() - r });
    const c = window;
    c.__nonameAutoDriveDone = l;
  })();
}
function Wp(e) {
  if (e === null || typeof e != "object" || Array.isArray(e)) return e;
  const n = e.SingleWin;
  if (n === null || typeof n != "object" || Array.isArray(n)) return e;
  const t = n.player;
  return typeof t != "number" || !Number.isInteger(t) || t <= 0 ? e : { ...e, SingleWin: { ...n, player: t - 1 } };
}
const hl = "noname-next-desktop-engine";
function fl() {
  const e = typeof location > "u" ? null : nn("engineTransport");
  if (e === "native" || e === "wasm") return e;
  try {
    return localStorage.getItem(hl) === "native" ? "native" : "wasm";
  } catch {
    return "wasm";
  }
}
function fo(e) {
  return dl(e.view, e.choice), { ...e, view: { ...e.view, outcome: Wp(e.view.outcome) } };
}
function Bp(e, n) {
  var _a2;
  if (!e.ok || !e.data || typeof e.data != "object") return e;
  if (n.type === "chooseGeneral") {
    const t = e.data;
    if ((_a2 = t.envelope) == null ? void 0 : _a2.view) return { ...e, data: { ...t, envelope: fo(t.envelope) } };
  } else {
    const t = e.data;
    if (t.view) return { ...e, data: fo(t) };
  }
  return e;
}
class Kp {
  constructor(n = jt) {
    __publicField(this, "onmessage", null);
    __publicField(this, "onerror", null);
    __publicField(this, "closed", false);
    __publicField(this, "tail", Promise.resolve());
    __publicField(this, "resource", null);
    this.call = n;
  }
  postMessage(n) {
    if (this.closed) throw new Error("Desktop engine transport closed");
    this.tail = this.tail.then(async () => {
      var _a2;
      if (this.closed) return;
      let t;
      try {
        this.resource ?? (this.resource = this.call("native_engine_open"));
        const s = await this.resource;
        if (this.closed) return;
        if (t = await this.call("native_engine_request", { resourceId: s, request: n }), t = Bp(t, n), t.id !== n.id || typeof t.ok != "boolean") throw new Error("invalid IPC response");
      } catch (s) {
        t = { id: n.id, ok: false, error: `Desktop IPC: ${String(s)}; \u64CD\u4F5C\u7ED3\u679C\u53EF\u80FD\u672A\u77E5\uFF0C\u8BF7\u5148\u786E\u8BA4\u5F53\u524D\u5C40\u9762` };
      }
      this.closed || ((_a2 = this.onmessage) == null ? void 0 : _a2.call(this, { data: t }));
    });
  }
  terminate() {
    var _a2;
    this.closed || (this.closed = true, (_a2 = this.resource) == null ? void 0 : _a2.then((n) => this.call("native_engine_close", { resourceId: n })).catch((n) => console.error("Desktop engine resource close failed", n)));
  }
  addEventListener(n, t) {
  }
}
function da(e, n) {
  if (e.length === 0) return null;
  const t = [...e].sort((a, i) => a - i), s = Math.ceil(n / 100 * t.length);
  return t[Math.min(t.length, Math.max(1, s)) - 1];
}
function Rs(e) {
  const n = e.filter((s) => Number.isFinite(s)), t = n.reduce((s, a) => s + a, 0);
  return { count: n.length, p50: da(n, 50), p95: da(n, 95), p99: da(n, 99), max: n.length ? Math.max(...n) : null, mean: n.length ? t / n.length : null };
}
function Gp(e) {
  const n = [];
  for (let t = 1; t < e.length; t++) n.push(e[t] - e[t - 1]);
  return n;
}
function Jp(e) {
  const n = Rs(e), t = (a) => e.length ? e.filter((i) => i > a).length / e.length : null, s = n.p50;
  return { ...n, refresh_estimate_ms: s, over_8_33_ratio: t(1e3 / 120), over_16_67_ratio: t(1e3 / 60), over_1_5x_refresh_ratio: s === null ? null : t(s * 1.5) };
}
const Cn = (e, n) => e === null || n === null ? null : e - n;
function mo(e) {
  const n = e.worker, t = e.input_ts ?? e.sent;
  return { id: e.id, type: e.type, input_kind: e.input_kind, input_to_send_ms: Cn(e.sent, e.input_ts), roundtrip_ms: Cn(e.received, e.sent), worker_total_ms: (n == null ? void 0 : n.total_ms) ?? null, dispatch_ms: (n == null ? void 0 : n.dispatch_ms) ?? null, view_json_ms: (n == null ? void 0 : n.view_json_ms) ?? null, view_parse_ms: (n == null ? void 0 : n.view_parse_ms) ?? null, adapt_ms: (n == null ? void 0 : n.adapt_ms) ?? null, post_ms: (n == null ? void 0 : n.post_ms) ?? null, transfer_ms: Cn(e.received_epoch_ms, (n == null ? void 0 : n.post_epoch_ms) ?? null), receive_to_apply_ms: Cn(e.apply_start, e.received), apply_ms: Cn(e.apply_end, e.apply_start), vue_patch_ms: Cn(e.patched, e.apply_end), patch_to_frame_ms: Cn(e.frame, e.patched), frame_to_paint_ms: Cn(e.after_paint, e.frame), input_to_paint_ms: Cn(e.after_paint, t), view_json_chars: (n == null ? void 0 : n.view_json_chars) ?? null, envelope_json_chars: (n == null ? void 0 : n.envelope_json_chars) ?? null };
}
function Yp(e) {
  const n = ["input_to_send_ms", "roundtrip_ms", "worker_total_ms", "dispatch_ms", "view_json_ms", "view_parse_ms", "adapt_ms", "post_ms", "transfer_ms", "receive_to_apply_ms", "apply_ms", "vue_patch_ms", "patch_to_frame_ms", "frame_to_paint_ms", "input_to_paint_ms", "view_json_chars", "envelope_json_chars"], t = {};
  for (const s of n) {
    const a = e.map((i) => i[s]).filter((i) => typeof i == "number" && Number.isFinite(i));
    t[s] = Rs(a);
  }
  return t;
}
const Qp = "noname-next-perf", Xp = 3e5, Zp = 2e4, Pa = 5e4, eh = 100;
let go = false, Xe = false;
const Rt = [], zt = [], It = [], wn = /* @__PURE__ */ new Map(), yt = /* @__PURE__ */ new Map();
let Ot = null, kt = null;
function nh() {
  var _a2;
  try {
    if (((_a2 = globalThis.localStorage) == null ? void 0 : _a2.getItem(Qp)) === "1") return true;
  } catch {
  }
  if (typeof location > "u") return false;
  if (new URLSearchParams(location.search).get("perf") === "1") return true;
  const e = location.hash.split("?")[1];
  return e ? new URLSearchParams(e).get("perf") === "1" : false;
}
const th = () => performance.timeOrigin + performance.now();
function _o(e, n, t) {
  var _a2;
  if (typeof PerformanceObserver > "u" || !((_a2 = PerformanceObserver.supportedEntryTypes) == null ? void 0 : _a2.includes(e))) return;
  new PerformanceObserver((a) => {
    for (const i of a.getEntries()) t(i);
  }).observe({ type: e, buffered: true, ...n });
}
function vo(e) {
  Ot = { ts: e.timeStamp, kind: e.type, used: false };
}
function sh() {
  if (go || (go = true, Xe = nh(), !Xe || typeof window > "u")) return Xe;
  const e = (n) => {
    Rt.length < Xp && Rt.push(n), requestAnimationFrame(e);
  };
  return requestAnimationFrame(e), window.addEventListener("click", vo, { capture: true }), window.addEventListener("keydown", vo, { capture: true }), _o("longtask", {}, (n) => {
    zt.length < Pa && zt.push({ start: n.startTime, duration: n.duration });
  }), _o("event", { durationThreshold: 16 }, (n) => {
    const t = n;
    It.length < Pa && It.push({ name: t.name, start: t.startTime, duration: t.duration, input_delay: t.processingStart - t.startTime, processing: t.processingEnd - t.processingStart });
  }), window.__nonamePerf = { version: 1, enabled: true, frames: () => Rt.slice(), longTasks: () => zt.slice(), eventTimings: () => It.slice(), interactions: () => [...wn.values()].map(mo), summary: () => {
    const n = [...wn.values()].map(mo);
    return { frames: Jp(Gp(Rt)), interactions: Yp(n), long_tasks: Rs(zt.map((t) => t.duration)), event_timings: Rs(It.map((t) => t.duration)) };
  }, reset: () => {
    Rt.length = 0, zt.length = 0, It.length = 0, wn.clear(), yt.clear(), kt = null;
  } }, Xe && Hp(), Xe;
}
function ah(e, n) {
  if (!Xe || wn.size >= Zp) return;
  const t = performance.now(), s = Ot && !Ot.used && t - Ot.ts <= eh ? Ot : null;
  s && (s.used = true), wn.set(e, { id: e, type: n, input_ts: (s == null ? void 0 : s.ts) ?? null, input_kind: (s == null ? void 0 : s.kind) ?? null, sent: t, received: null, received_epoch_ms: null, apply_start: null, apply_end: null, patched: null, frame: null, after_paint: null, worker: yt.get(e) ?? null }), yt.delete(e);
}
function yo(e) {
  Xe && (wn.delete(e), yt.delete(e));
}
function ko(e) {
  if (!Xe) return;
  const n = wn.get(e);
  n && (n.received = performance.now(), n.received_epoch_ms = th(), kt = e);
}
function ih(e, n) {
  if (!Xe) return;
  const t = wn.get(e);
  t ? t.worker = n : yt.size < Pa && yt.set(e, n);
}
function oh() {
  if (kt === null) return null;
  const e = wn.get(kt);
  return e && e.apply_start === null ? e : null;
}
function rh() {
  if (!Xe) return;
  const e = oh();
  e && (e.apply_start = performance.now());
}
function lh() {
  if (!Xe || kt === null) return;
  const e = wn.get(kt);
  !e || e.apply_start === null || e.apply_end !== null || (e.apply_end = performance.now(), ss(() => {
    e.patched = performance.now(), requestAnimationFrame(() => {
      e.frame = performance.now();
      const n = new MessageChannel();
      n.port1.onmessage = () => {
        e.after_paint = performance.now(), n.port1.close();
      }, n.port2.postMessage(0);
    });
  }));
}
function uh(e) {
  return new Worker("/assets/engine.native.worker-BIwpD907.js", { type: "module", name: e == null ? void 0 : e.name });
}
function ch(e) {
  return new Worker("/assets/engine.worker-DF5gc_e9.js", { type: "module", name: e == null ? void 0 : e.name });
}
function wo(e) {
  return fetch(e).then((n) => {
    if (!n.ok) throw new Error(`\u538B\u6D4B\u8F7D\u8377\u52A0\u8F7D\u5931\u8D25 ${e}: ${n.status}`);
    return n.text();
  });
}
class ii {
  constructor() {
    __publicField(this, "worker", null);
    __publicField(this, "transportKind", "wasm-worker");
    __publicField(this, "seq", 1);
    __publicField(this, "pending", /* @__PURE__ */ new Map());
    __publicField(this, "native", true);
    __publicField(this, "sabPollSeq", -1);
  }
  getTransportKind() {
    return this.transportKind;
  }
  async connect() {
    if (this.worker) return;
    const n = "native".toLowerCase();
    this.native = n !== "legacy" && n !== "wasm" && n !== "old";
    const t = !this.native && jn() ? await _p() : void 0, s = this.native && jn() && fl() === "native";
    this.transportKind = s ? "desktop-native" : this.native ? "wasm-worker" : "legacy-worker", globalThis.__engineTransport = this.transportKind;
    const a = s ? new Kp() : this.native ? new uh() : new ch();
    this.worker = a, a.onmessage = (i) => {
      var _a2, _b2;
      if (this.worker !== a) return;
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
        ih(d.id, d.timings);
        return;
      }
      const r = i.data, l = this.pending.get(r.id);
      l && (ko(r.id), this.pending.delete(r.id), (_b2 = (_a2 = globalThis.__latencySabDebug) == null ? void 0 : _a2.settle) == null ? void 0 : _b2.call(_a2), r.ok ? l.resolve(r.data) : l.reject(new Error(r.error)));
    }, a.onerror = (i) => {
      this.worker === a && (console.error("engine worker error", i), this.rejectPending(new Error("Worker \u53D1\u751F\u9519\u8BEF\uFF0C\u64CD\u4F5C\u7ED3\u679C\u672A\u77E5\uFF1B\u8BF7\u5148\u786E\u8BA4\u5F53\u524D\u5C40\u9762")));
    }, a.addEventListener("messageerror", () => {
      this.worker === a && this.rejectPending(new Error("Worker \u56DE\u5305\u65E0\u6CD5\u8BFB\u53D6\uFF0C\u64CD\u4F5C\u7ED3\u679C\u672A\u77E5\uFF1B\u8BF7\u5148\u786E\u8BA4\u5F53\u524D\u5C40\u9762"));
    });
    try {
      await this.request({ id: this.nextId(), type: "init", bundle: t ?? void 0, perf: sh(), latencySab: s ? void 0 : this.initSabMirror() });
    } catch (i) {
      throw this.worker === a && this.dispose(), i;
    }
  }
  initSabMirror() {
    if (nn("latency") !== "sab" || !crossOriginIsolated || typeof SharedArrayBuffer > "u") return;
    const n = document.createElement("style");
    n.textContent = ".sab-keepalive{animation:sab-pulse 8ms infinite}@keyframes sab-pulse{50%{margin-left:.5px}}", document.head.appendChild(n);
    const t = document.createElement("div");
    t.className = "sab-keepalive", t.style.display = "none", document.body.appendChild(t);
    const s = 256 * 1024, a = new SharedArrayBuffer(s), i = new Int32Array(a, 0, 2);
    Atomics.store(i, 0, 0), Atomics.store(i, 1, 0);
    const o = new TextDecoder(), r = () => {
      t.style.display = "none";
    }, l = { runs: 0, seen: 0, parseErr: 0, miss: 0 }, d = () => {
      l.runs += 1;
      const c = Atomics.load(i, 0);
      if (c !== this.sabPollSeq) {
        l.seen += 1, this.sabPollSeq = c;
        const p = Atomics.load(i, 1);
        if (p > 0 && 8 + p <= s) try {
          const m = new Uint8Array(p);
          m.set(new Uint8Array(a, 8, p));
          const f = JSON.parse(o.decode(m)), b = this.pending.get(f.id);
          if (b) {
            ko(f.id), this.pending.delete(f.id), r();
            const P = globalThis.__latencySabDebug;
            P && (P.resolvedEarly += 1), f.ok ? b.resolve(f.data) : b.reject(new Error(f.error ?? "worker error"));
          } else l.miss += 1;
        } catch {
          l.parseErr += 1;
        }
      }
      requestAnimationFrame(d);
    };
    return requestAnimationFrame(d), globalThis.__latencySabDebug = { seq: () => Atomics.load(i, 0), raw: () => {
      const c = Atomics.load(i, 1), p = new Uint8Array(Math.min(c, 120));
      return p.set(new Uint8Array(a, 8, p.length)), { len: c, head: o.decode(p) };
    }, wake: () => {
      t.style.display = "block";
    }, settle: r, stats: l, resolvedEarly: 0, resolvedByMessage: 0 }, a;
  }
  isNative() {
    return this.native;
  }
  dispose() {
    const n = this.worker;
    this.worker = null, n == null ? void 0 : n.terminate(), this.rejectPending(new Error("Worker \u5DF2\u5173\u95ED\uFF0C\u64CD\u4F5C\u7ED3\u679C\u672A\u77E5\uFF1B\u8BF7\u5148\u786E\u8BA4\u5F53\u524D\u5C40\u9762"));
  }
  rejectPending(n) {
    var _a2, _b2;
    if (this.pending.size !== 0) {
      for (const [t, s] of this.pending) yo(t), s.reject(n);
      this.pending.clear(), (_b2 = (_a2 = globalThis.__latencySabDebug) == null ? void 0 : _a2.settle) == null ? void 0 : _b2.call(_a2);
    }
  }
  request(n) {
    return this.worker ? new Promise((t, s) => {
      var _a2, _b2, _c2, _d2;
      this.pending.set(n.id, { resolve: t, reject: s }), ah(n.id, n.type), (_b2 = (_a2 = globalThis.__latencySabDebug) == null ? void 0 : _a2.wake) == null ? void 0 : _b2.call(_a2);
      try {
        this.worker.postMessage(n);
      } catch (a) {
        this.pending.delete(n.id), yo(n.id), this.pending.size === 0 && ((_d2 = (_c2 = globalThis.__latencySabDebug) == null ? void 0 : _c2.settle) == null ? void 0 : _d2.call(_c2)), s(a);
      }
    }) : Promise.reject(new Error("worker not connected"));
  }
  nextId() {
    return this.seq++;
  }
  async start(n, t, s = 1, a, i) {
    const o = nn("packsUrl"), r = nn("hexUrl"), l = o ? await wo(o) : a;
    return r ? (await this.request({ id: this.nextId(), type: "start", players: n, seed: t, observer: s, packsJson: l, mode: i }), this.request({ id: this.nextId(), type: "restore", hex: await wo(r), packsJson: l })) : this.request({ id: this.nextId(), type: "start", players: n, seed: t, observer: s, packsJson: l, mode: i });
  }
  endPhase(n = 0) {
    return this.request({ id: this.nextId(), type: "endPhase", actor: n });
  }
  playCard(n, t, s) {
    return this.request({ id: this.nextId(), type: "playCard", actor: n, card: t, targets: [...s] });
  }
  activateSkill(n, t, s, a) {
    return this.request({ id: this.nextId(), type: "activateSkill", actor: n, skill: t, targets: [...s], cardTargets: [...a] });
  }
  useSkillCard(n, t, s, a, i) {
    return this.request({ id: this.nextId(), type: "useSkillCard", actor: n, skill: t, cardKind: s, targets: [...a], equipSlot: i });
  }
  resolveChoice(n, t, s, a = false) {
    return this.request({ id: this.nextId(), type: "resolveChoice", actor: n, choiceId: t, selections: [...s], cancelled: a });
  }
  autoStep() {
    return this.request({ id: this.nextId(), type: "autoStep" });
  }
  generalCandidates() {
    return this.request({ id: this.nextId(), type: "generalCandidates" });
  }
  chooseGeneral(n, t) {
    return this.request({ id: this.nextId(), type: "chooseGeneral", seat: n, name: t });
  }
  observer() {
    return this.native ? this.request({ id: this.nextId(), type: "observer" }).then((n) => {
      const t = n;
      if (t == null ? void 0 : t.view) return t.view;
      throw new Error("native observer: \u7F3A view");
    }) : this.request({ id: this.nextId(), type: "observer" });
  }
  saveHex() {
    return this.request({ id: this.nextId(), type: "save" }).then((n) => {
      const t = n;
      if (typeof t == "string") return t;
      if (t == null ? void 0 : t.hex) return t.hex;
      throw new Error("native save: \u7F3A hex");
    });
  }
  restoreHex(n, t) {
    return this.request({ id: this.nextId(), type: "restore", hex: n, packsJson: t });
  }
  async validatePackage(n) {
    await this.request({ id: this.nextId(), type: "validatePackage", packJson: n });
  }
  loadReplay(n) {
    return this.request({ id: this.nextId(), type: "loadReplay", json: n });
  }
  seekReplay(n, t, s = 1) {
    return this.request({ id: this.nextId(), type: "seekReplay", step: n, privacy: t, player: s });
  }
  status() {
    return this.request({ id: this.nextId(), type: "status" });
  }
}
const Ta = 200, J = xt({ contentLockHash: "", total: 0, pages: /* @__PURE__ */ new Map(), characters: /* @__PURE__ */ new Map(), skins: /* @__PURE__ */ new Map(), translations: {}, selectedSkins: /* @__PURE__ */ new Map(), portraitUrls: /* @__PURE__ */ new Map(), imageUrls: /* @__PURE__ */ new Map(), substituteUrls: /* @__PURE__ */ new Map() });
let Bn = null;
const Nt = /* @__PURE__ */ new Map(), qt = /* @__PURE__ */ new Map(), Oa = /* @__PURE__ */ new Map();
function dh(e) {
  if (jn()) {
    Ma({ kind: "desktop", key: "desktop" });
    return;
  }
  const n = `${window.location.protocol === "https:" ? "https:" : "http:"}//${window.location.hostname || "127.0.0.1"}:7700`;
  Ma({ kind: "http", key: `http:${n}`, baseUrl: n });
}
function ph(e) {
  const n = new URL(e.trim(), window.location.href);
  n.protocol = n.protocol === "wss:" || n.protocol === "https:" ? "https:" : "http:";
  const t = n.origin;
  Ma({ kind: "http", key: `http:${t}`, baseUrl: t });
}
async function hh(e) {
  const n = J.characters.get(e);
  if (n) return J.imageUrls.has(e) || await qa(e), n;
  if (!Bn) return;
  const t = qt.get(e);
  if (t) return t;
  const s = Bn.key, a = (async () => {
    let i = 0;
    for (; (Bn == null ? void 0 : Bn.key) === s; ) {
      const o = await ml(i);
      if (!o) break;
      const r = J.characters.get(e);
      if (r) return await qa(e), r;
      const l = o.offset + o.characters.length;
      if (l >= o.total || l <= i) break;
      i = l;
    }
  })();
  qt.set(e, a);
  try {
    return await a;
  } finally {
    qt.get(e) === a && qt.delete(e);
  }
}
function fh(e) {
  var _a2;
  const n = (_a2 = J.characters.get(e)) == null ? void 0 : _a2.definition.name_key;
  return n ? gl(n) : void 0;
}
function mh(e) {
  var _a2;
  return ((_a2 = J.characters.get(e)) == null ? void 0 : _a2.skin_ids.flatMap((t) => {
    const s = J.skins.get(t);
    return s && s.presentation.kind !== "substitute" ? [s] : [];
  })) ?? [];
}
function gh(e) {
  return gl(e.name_key);
}
function Na(e) {
  const n = J.characters.get(e);
  if (!n || !J.contentLockHash) return null;
  if (J.selectedSkins.has(e)) return J.selectedSkins.get(e) ?? null;
  const t = _l(e);
  let s = null;
  if (t) try {
    const a = Number(localStorage.getItem(t));
    Number.isInteger(a) && vl(n).includes(a) && (s = a);
  } catch {
  }
  return J.selectedSkins.set(e, s), s;
}
function _h(e, n) {
  const t = J.characters.get(e);
  if (n != null && (!t || !vl(t).includes(n))) return;
  J.selectedSkins.set(e, n);
  const s = _l(e);
  if (s) try {
    n == null ? localStorage.removeItem(s) : localStorage.setItem(s, String(n));
  } catch {
  }
  qa(e);
}
function vh(e) {
  return J.imageUrls.get(e) ?? Ys(e);
}
function yh(e) {
  return J.portraitUrls.get(e) ?? Ys(e);
}
function kh(e, n) {
  return J.substituteUrls.get(`${e}:${n}`) ?? null;
}
function Ys(e) {
  return `/original/portraits/${e}.jpg`;
}
async function ml(e) {
  const n = J.pages.get(e);
  if (n) return n;
  const t = Nt.get(e);
  if (t) return t;
  const s = Bn;
  if (!s) return null;
  const a = (async () => {
    let i;
    if (s.kind === "desktop") i = await vp(e, Ta);
    else {
      const o = new URL("/content-catalog", s.baseUrl);
      o.searchParams.set("offset", String(e)), o.searchParams.set("limit", String(Ta));
      const r = await fetch(o);
      if (!r.ok) throw new Error(`\u5185\u5BB9\u76EE\u5F55\u52A0\u8F7D\u5931\u8D25 (${r.status})`);
      i = await r.json();
    }
    if (Bn !== s || i == null) return null;
    if (bh(i), i.offset !== Math.min(e, i.total)) throw new Error("\u5185\u5BB9\u76EE\u5F55\u5206\u9875\u504F\u79FB\u4E0D\u4E00\u81F4");
    return e !== 0 && J.contentLockHash && J.contentLockHash !== i.content_lock_hash ? (oi(), ml(0)) : (wh(i), i);
  })();
  Nt.set(e, a);
  try {
    return await a;
  } finally {
    Nt.get(e) === a && Nt.delete(e);
  }
}
function wh(e) {
  J.contentLockHash && J.contentLockHash !== e.content_lock_hash && oi(), J.contentLockHash = e.content_lock_hash, J.total = e.total, J.pages.set(e.offset, e);
  for (const n of e.characters) J.characters.set(n.definition.id, n);
  for (const n of e.skins) J.skins.set(n.id, n);
  for (const [n, t] of Object.entries(e.translations)) J.translations[n] = { ...J.translations[n], ...t };
}
async function qa(e) {
  const n = J.characters.get(e);
  if (!n) return;
  const t = Na(e), s = t == null ? void 0 : J.skins.get(t), a = (s == null ? void 0 : s.presentation.kind) === "static" ? s.presentation.asset : null, i = Ys(e);
  J.portraitUrls.set(e, i), J.imageUrls.set(e, i);
  const o = `${J.contentLockHash}:${t ?? "base"}`, r = n.portrait_asset ? await pa(n.portrait_asset).catch(() => null) : null, l = a ? await pa(a).catch(() => null) : null, d = n.skin_ids.map((m) => J.skins.get(m)).find((m) => (m == null ? void 0 : m.presentation.kind) === "substitute" && m.presentation.rule === "dead"), c = (d == null ? void 0 : d.presentation.kind) === "substitute" ? d.presentation.asset : null, p = c ? await pa(c).catch(() => null) : null;
  if (o === `${J.contentLockHash}:${Na(e) ?? "base"}`) {
    const m = r ?? i;
    J.portraitUrls.set(e, m), J.imageUrls.set(e, l ?? m), p && J.substituteUrls.set(`${e}:dead`, p);
  }
}
function pa(e) {
  if (!xh(e) || !Bn) return Promise.resolve(null);
  const n = Bn, t = [n.key, e.namespace, e.package_id, e.asset_id].join("\0"), s = Oa.get(t);
  if (s) return s;
  const a = n.kind === "desktop" ? yp(e) : Promise.resolve(`${n.baseUrl}/package-assets/${[e.namespace, e.package_id].map(encodeURIComponent).join("/")}/${e.asset_id.split("/").map(encodeURIComponent).join("/")}?v=${encodeURIComponent(J.contentLockHash)}`);
  return Oa.set(t, a), a;
}
function gl(e) {
  var _a2;
  const n = typeof navigator > "u" ? "" : navigator.language, t = [n, n.replace("-", "_"), "zh-CN", "zh_CN", "zh", "en"];
  for (const s of t) {
    const a = (_a2 = J.translations[s]) == null ? void 0 : _a2[e];
    if (a) return a;
  }
  return e;
}
function _l(e) {
  return J.contentLockHash ? `noname-next.skin.${J.contentLockHash}.${e}` : null;
}
function Ma(e) {
  Bn = e, oi();
}
function oi() {
  J.contentLockHash = "", J.total = 0, J.pages.clear(), J.characters.clear(), J.skins.clear(), J.translations = {}, J.selectedSkins.clear(), J.portraitUrls.clear(), J.imageUrls.clear(), J.substituteUrls.clear(), Nt.clear(), qt.clear(), Oa.clear();
}
function bh(e) {
  const n = e;
  if (!n || typeof n.content_lock_hash != "string" || !Dn(n.offset) || !Dn(n.limit) || !Dn(n.total) || n.limit < 1 || n.limit > Ta || n.offset > n.total || !Array.isArray(n.characters) || n.characters.length > n.limit || !Array.isArray(n.skins) || !Array.isArray(n.aliases) || !n.translations || typeof n.translations != "object" || !n.characters.every((t) => {
    var _a2, _b2;
    return Dn((_a2 = t == null ? void 0 : t.definition) == null ? void 0 : _a2.id) && t.definition.id > 0 && typeof ((_b2 = t == null ? void 0 : t.definition) == null ? void 0 : _b2.name_key) == "string" && Array.isArray(t == null ? void 0 : t.skin_ids) && t.skin_ids.every(Dn);
  }) || !n.skins.every((t) => {
    var _a2;
    return Dn(t == null ? void 0 : t.id) && t.id > 0 && typeof (t == null ? void 0 : t.name_key) == "string" && ["static", "spine", "substitute"].includes((_a2 = t == null ? void 0 : t.presentation) == null ? void 0 : _a2.kind);
  }) || !n.aliases.every((t) => Dn(t == null ? void 0 : t.character_id) && Dn(t == null ? void 0 : t.source_character_id) && bo(t == null ? void 0 : t.voice_mappings)) || !Object.values(n.translations).every(bo)) throw new Error("\u5185\u5BB9\u76EE\u5F55\u54CD\u5E94\u65E0\u6548");
}
function Dn(e) {
  return Number.isSafeInteger(e) && Number(e) >= 0;
}
function vl(e) {
  return e.skin_ids.filter((n) => {
    var _a2;
    return ((_a2 = J.skins.get(n)) == null ? void 0 : _a2.presentation.kind) !== "substitute";
  });
}
function bo(e) {
  return !!e && typeof e == "object" && !Array.isArray(e) && Object.values(e).every((n) => typeof n == "string");
}
function xh(e) {
  return typeof e.namespace == "string" && e.namespace.length > 0 && typeof e.package_id == "string" && e.package_id.length > 0 && typeof e.asset_id == "string" && e.asset_id.length > 0;
}
const Kt = [{ dir: "cardpile-upstream-liuli", packageId: "cardpile", namespace: "upstream-liuli", version: "1.0.0", title: "\u724C\u5806\uFF08\u7409\u7483\uFF09", logicFingerprint: "8dd99eb2" }, { dir: "cardpile-upstream-noname", packageId: "cardpile", namespace: "upstream-noname", version: "1.0.0", title: "\u724C\u5806\uFF08\u65E0\u540D\uFF09", logicFingerprint: "83aa81c4" }, { dir: "d3-upstream-noname", packageId: "d3-select", namespace: "upstream-noname", version: "1.0.0", title: "3D\u7CBE\u9009", logicFingerprint: "6f257a96" }, { dir: "huanle-upstream-noname", packageId: "huanle-cards", namespace: "upstream-noname", version: "1.0.0", title: "\u6B22\u4E50\u5361\u724C", logicFingerprint: "6ae1907c" }, { dir: "shaai-ow-upstream-noname", packageId: "shaai-ow", namespace: "upstream-noname", version: "1.0.0", title: "\u5B88\u671B\u5148\u950B", logicFingerprint: "6117a13e" }, { dir: "shaai-xianjian-upstream-noname", packageId: "shaai-xianjian", namespace: "upstream-noname", version: "1.0.0", title: "\u4ED9\u5251\u5947\u4FA0\u4F20", logicFingerprint: "d9ee1863" }, { dir: "shahai-gujian-upstream-noname", packageId: "shahai-gujian", namespace: "upstream-noname", version: "1.0.0", title: "\u53E4\u5251\u5947\u8C2D", logicFingerprint: "ae13018f" }, { dir: "shahai-gwent-upstream-noname", packageId: "shahai-gwent", namespace: "upstream-noname", version: "1.0.0", title: "\u6606\u7279\u724C", logicFingerprint: "0bd2569a" }, { dir: "shahai-hearth-upstream-noname", packageId: "shahai-hearth", namespace: "upstream-noname", version: "1.0.0", title: "\u7089\u77F3\u4F20\u8BF4", logicFingerprint: "deafc31c" }, { dir: "shahai-mtg-upstream-noname", packageId: "shahai-mtg", namespace: "upstream-noname", version: "1.0.0", title: "\u4E07\u667A\u724C", logicFingerprint: "6417be73" }, { dir: "shahai-swd-upstream-noname", packageId: "shahai-swd", namespace: "upstream-noname", version: "1.0.0", title: "\u8F69\u8F95\u5251", logicFingerprint: "31ccea98" }, { dir: "shahai-yunchou-upstream-noname", packageId: "shahai-yunchou", namespace: "upstream-noname", version: "1.0.0", title: "\u8FD0\u7B79\u5E37\u5E44", logicFingerprint: "57491089" }, { dir: "shahai-zhenfa-upstream-noname", packageId: "shahai-zhenfa", namespace: "upstream-noname", version: "1.0.0", title: "\u9635\u6CD5", logicFingerprint: "53945b47" }, { dir: "wudian-upstream-noname", packageId: "wudian", namespace: "upstream-noname", version: "1.0.0", title: "\u73A9\u70B9\u8BBA\u6740", logicFingerprint: "e0d488b6" }, { dir: "wuxing-upstream-liuli", packageId: "wuxing", namespace: "upstream-liuli", version: "1.0.0", title: "\u4E94\u884C", logicFingerprint: "845ea6fa" }];
function ri(e) {
  return `${e.namespace}/${e.packageId}`;
}
const $h = { package_format_version: 1, namespace: "upstream-liuli", package_id: "cardpile", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 753, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [141, 217, 158, 178, 102, 250, 23, 48, 46, 154, 229, 230, 185, 21, 142, 207, 121, 191, 21, 160, 114, 238, 113, 65, 184, 219, 63, 10, 143, 211, 102, 196], asset_hash: [5, 133, 68, 112, 135, 182, 90, 250, 76, 92, 107, 127, 63, 60, 145, 39, 21, 177, 136, 200, 170, 224, 209, 228, 167, 139, 124, 218, 97, 232, 139, 249], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, jh = [], Sh = [], Eh = [], Ah = [], Ch = [], Rh = [], zh = [], Ih = [], Ph = { "zh-CN": { cardpile: "\u724C\u5806", huosha: "\u706B\u6740", jiu: "\u9152", leisha: "\u96F7\u6740" } }, Th = { schema_version: 1, package_id: "cardpile", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "cardpile:pile", source_hash: [223, 149, 103, 52, 153, 51, 90, 98, 219, 54, 199, 28, 186, 38, 121, 216, 91, 164, 250, 84, 188, 239, 98, 143, 239, 121, 48, 90, 56, 228, 25, 104], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, Oh = { "cardpile:pile": "assets/pile.json" }, Nh = "gen-1", xo = { manifest: $h, modules: jh, skill_index: Sh, character_index: Eh, card_index: Ah, trigger_index: Ch, modifier_index: Rh, active_index: zh, view_as_index: Ih, translations: Ph, asset_manifest: Th, asset_paths: Oh, compiler_version: Nh }, qh = "noname-liuli", Mh = "noname/extension/cardpile/extension.js", Dh = "8485dbb29ff16f9b258a766b1a17ef75f883aade", Lh = 160, Uh = [{ card: "sha", counts: { diamond: 6, club: 14, heart: 3, spade: 7 } }, { card: "huosha", counts: { diamond: 2, heart: 3 } }, { card: "leisha", counts: { spade: 5, club: 4 } }, { card: "shan", counts: { heart: 6, diamond: 18 } }, { card: "jiu", counts: { diamond: 1, spade: 2, club: 2 } }, { card: "tao", counts: { heart: 9, diamond: 3 } }, { card: "wanjian", counts: { heart: 1 } }, { card: "nanman", counts: { spade: 2, club: 1 } }, { card: "guohe", counts: { spade: 3, club: 2, heart: 1 } }, { card: "shunshou", counts: { spade: 3, diamond: 2 } }, { card: "wuxie", counts: { heart: 2, diamond: 1, spade: 2, club: 2 } }, { card: "tiesuo", counts: { spade: 2, club: 4 } }], Fh = { source: qh, upstream_path: Mh, upstream_commit: Dh, total: Lh, entries: Uh }, Vh = { package_format_version: 1, namespace: "upstream-noname", package_id: "cardpile", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 753, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [131, 170, 129, 196, 137, 101, 152, 220, 92, 26, 119, 144, 188, 18, 226, 118, 124, 202, 213, 56, 37, 221, 167, 227, 108, 52, 173, 68, 188, 187, 99, 220], asset_hash: [128, 188, 216, 204, 239, 216, 63, 133, 226, 76, 81, 90, 3, 222, 209, 209, 187, 152, 128, 170, 74, 141, 193, 125, 195, 157, 166, 254, 81, 218, 111, 12], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, Hh = [], Wh = [], Bh = [], Kh = [], Gh = [], Jh = [], Yh = [], Qh = [], Xh = { "zh-CN": { cardpile: "\u724C\u5806", huosha: "\u706B\u6740", jiu: "\u9152", leisha: "\u96F7\u6740" } }, Zh = { schema_version: 1, package_id: "cardpile", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "cardpile:pile", source_hash: [107, 194, 232, 96, 169, 46, 18, 235, 51, 137, 142, 3, 98, 227, 253, 110, 137, 37, 246, 88, 93, 161, 193, 213, 246, 39, 29, 229, 204, 37, 149, 34], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, ef = { "cardpile:pile": "assets/pile.json" }, nf = "gen-1", $o = { manifest: Vh, modules: Hh, skill_index: Wh, character_index: Bh, card_index: Kh, trigger_index: Gh, modifier_index: Jh, active_index: Yh, view_as_index: Qh, translations: Xh, asset_manifest: Zh, asset_paths: ef, compiler_version: nf }, tf = "noname", sf = "apps/core/extension/cardpile/extension.js", af = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", of = 160, rf = [{ card: "sha", counts: { diamond: 6, club: 14, heart: 3, spade: 7 } }, { card: "huosha", counts: { diamond: 2, heart: 3 } }, { card: "leisha", counts: { spade: 5, club: 4 } }, { card: "shan", counts: { heart: 6, diamond: 18 } }, { card: "jiu", counts: { diamond: 1, spade: 2, club: 2 } }, { card: "tao", counts: { heart: 9, diamond: 3 } }, { card: "wanjian", counts: { heart: 1 } }, { card: "nanman", counts: { spade: 2, club: 1 } }, { card: "guohe", counts: { spade: 3, club: 2, heart: 1 } }, { card: "shunshou", counts: { spade: 3, diamond: 2 } }, { card: "wuxie", counts: { heart: 2, diamond: 1, spade: 2, club: 2 } }, { card: "tiesuo", counts: { spade: 2, club: 4 } }], lf = { source: tf, upstream_path: sf, upstream_commit: af, total: of, entries: rf }, uf = { package_format_version: 1, namespace: "upstream-noname", package_id: "d3-select", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [111, 37, 122, 150, 119, 48, 147, 56, 241, 49, 200, 32, 57, 124, 107, 75, 23, 118, 103, 218, 206, 252, 0, 59, 192, 24, 0, 59, 108, 78, 62, 128], asset_hash: [16, 5, 163, 133, 161, 153, 19, 47, 137, 85, 182, 46, 7, 149, 75, 218, 226, 85, 46, 131, 186, 196, 41, 78, 176, 6, 210, 119, 138, 54, 121, 189], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, cf = [], df = [], pf = [], hf = [], ff = [], mf = [], gf = [], _f = [], vf = { "zh-CN": { ddd_liuchong: "ddd\u5218\u51B2", dddjinggou: "\u7CBE\u5F40", dddmoyan: "\u672B\u7131" } }, yf = { schema_version: 1, package_id: "d3-select", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "d3-select:characters", source_hash: [50, 247, 124, 76, 209, 138, 160, 245, 215, 11, 118, 39, 184, 31, 122, 238, 107, 84, 39, 122, 206, 75, 86, 186, 245, 248, 189, 22, 28, 40, 135, 250], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, kf = { "d3-select:characters": "assets/character.json" }, wf = "gen-1", jo = { manifest: uf, modules: cf, skill_index: df, character_index: pf, card_index: hf, trigger_index: ff, modifier_index: mf, active_index: gf, view_as_index: _f, translations: vf, asset_manifest: yf, asset_paths: kf, compiler_version: wf }, bf = "noname", xf = "apps/core/extension/3D\u7CBE\u9009/", $f = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", jf = { name: "d3-upstream-noname", version: 1, description: "G1 3D\u7CBE\u9009", card_prefix: "ddd_" }, Sf = [{ id: "ddd_liuchong", name: "ddd\u5218\u51B2", faction: "qun", hp: 4, gender: "male", skills: [{ name: "dddjinggou", title: "\u7CBE\u5F40", mapping: "host:max_range_weapon_damage" }, { name: "dddmoyan", title: "\u672B\u7131", mapping: "host:d3_moyan" }] }], Ef = { source: bf, upstream_path: xf, upstream_commit: $f, pack: jf, generals: Sf }, Af = { package_format_version: 1, namespace: "upstream-noname", package_id: "huanle-cards", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [106, 225, 144, 124, 144, 13, 45, 180, 159, 95, 125, 53, 98, 74, 29, 117, 24, 49, 199, 10, 60, 244, 167, 94, 32, 142, 255, 23, 184, 119, 161, 55], asset_hash: [25, 84, 214, 199, 128, 168, 24, 45, 132, 188, 58, 79, 25, 130, 141, 162, 163, 191, 146, 92, 141, 235, 240, 14, 64, 255, 184, 47, 114, 101, 54, 250], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, Cf = [], Rf = [], zf = [], If = [], Pf = [], Tf = [], Of = [], Nf = [], qf = { "zh-CN": { huanle: "\u6B22\u4E50\u5361\u724C", jiuwei: "\u9152\u5C3E", kuwu: "\u82E6\u65E0", shoulijian: "\u624B\u91CC\u5251" } }, Mf = { schema_version: 1, package_id: "huanle-cards", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "huanle-cards:cards", source_hash: [8, 4, 171, 213, 237, 11, 78, 79, 232, 10, 93, 164, 129, 27, 245, 153, 138, 180, 85, 242, 69, 243, 148, 105, 243, 156, 24, 165, 76, 211, 77, 17], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, Df = { "huanle-cards:cards": "assets/cards.json" }, Lf = "gen-1", So = { manifest: Af, modules: Cf, skill_index: Rf, character_index: zf, card_index: If, trigger_index: Pf, modifier_index: Tf, active_index: Of, view_as_index: Nf, translations: qf, asset_manifest: Mf, asset_paths: Df, compiler_version: Lf }, Uf = "noname", Ff = "apps/core/extension/\u6B22\u4E50\u5361\u724C/", Vf = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", Hf = { name: "huanle-upstream-noname", version: 1, description: "G1 \u6B22\u4E50\u5361\u724C", card_prefix: "hnl_" }, Wf = [{ name: "kuwu", kind: "weapon", range: 2, equip_skills: ["huanle_kuwu"], suit: "spade", rank: 6 }, { name: "shoulijian", kind: "shoulijian", suit: "diamond", rank: 4 }, { name: "shoulijian", kind: "shoulijian", suit: "spade", rank: 4 }, { name: "monkey", kind: "monkey", suit: "diamond", rank: 5 }, { name: "xuelunyang", kind: "xuelunyang", suit: "heart", rank: 2 }, { name: "mianju", kind: "mianju", suit: "club", rank: 3 }, { name: "jiuwei", kind: "jiuwei", suit: "heart", rank: 9, equip_skills: ["huanle_jiuwei"] }], Bf = [{ name: "huanle_kuwu", mapping: "host:sha_damage_force_discard" }, { name: "huanle_jiuwei", mapping: "host:phase_end_recover_or_draw" }], Kf = { source: Uf, upstream_path: Ff, upstream_commit: Vf, pack: Hf, cards: Wf, skills: Bf }, Gf = { package_format_version: 1, namespace: "upstream-noname", package_id: "shaai-ow", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [97, 23, 161, 62, 250, 238, 201, 81, 165, 68, 177, 185, 128, 62, 23, 122, 149, 20, 247, 219, 47, 165, 34, 207, 63, 74, 111, 21, 134, 193, 80, 44], asset_hash: [234, 208, 248, 98, 248, 214, 252, 238, 255, 142, 242, 125, 150, 77, 115, 114, 87, 46, 204, 14, 51, 9, 132, 104, 243, 158, 19, 237, 178, 87, 25, 1], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, Jf = [], Yf = [], Qf = [], Xf = [], Zf = [], em = [], nm = [], tm = [], sm = { "zh-CN": { diaobingqianjiang: "\u8C03\u5175\u9063\u5C06", sanjian: "\u4E09\u5C16", wangmeizhike: "\u671B\u6885\u6B62\u6E34", youdishenru: "\u8BF1\u654C\u6DF1\u5165" } }, am = { schema_version: 1, package_id: "shaai-ow", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "shaai-ow:cards", source_hash: [156, 170, 113, 113, 238, 249, 132, 22, 250, 155, 70, 184, 125, 243, 73, 245, 56, 16, 110, 98, 179, 110, 249, 104, 140, 229, 198, 10, 8, 170, 103, 219], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, im = { "shaai-ow:cards": "assets/cards.json" }, om = "gen-1", Eo = { manifest: Gf, modules: Jf, skill_index: Yf, character_index: Qf, card_index: Xf, trigger_index: Zf, modifier_index: em, active_index: nm, view_as_index: tm, translations: sm, asset_manifest: am, asset_paths: im, compiler_version: om }, rm = "qita/sanguosha/noname/extension/\u6740\u6D77\u62FE\u9057/character/ow", lm = "extension/\u6740\u6D77\u62FE\u9057/character/ow/", um = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", cm = { name: "shaai-ow-upstream-noname", version: 1, description: "G1 \u7EB5\u5207\uFF1A\u4E0A\u6E38 ow \u58EB\u517576\uFF08\u7A81\u51FB+\u76EE\u955C\uFF09", card_prefix: "ow_" }, dm = [], pm = [], hm = [], fm = [{ name: "guangshu", general: "ow_zhixuzhiguang", mapping: "host:wudian_guangshu" }, { name: "guangshu_heart", general: "ow_zhixuzhiguang", mapping: "host:wudian_guangshu_heart" }, { name: "guangshu_diamond", general: "ow_zhixuzhiguang", mapping: "host:wudian_guangshu_diamond" }, { name: "guangshu_club", general: "ow_zhixuzhiguang", mapping: "host:wudian_guangshu_club" }, { name: "guangshu_spade", general: "ow_zhixuzhiguang", mapping: "host:wudian_guangshu_spade" }, { name: "ziyu", general: "ow_chanyata", mapping: "host:wudian_ziyu" }, { name: "shouhu", general: "ow_tianshi", mapping: "host:wudian_shouhu" }, { name: "duwen", general: "ow_heibaihe", mapping: "host:wudian_duwen" }, { name: "paotai", general: "ow_tuobiang", mapping: "host:wudian_paotai" }, { name: "paotai2", general: "ow_tuobiang", mapping: "host:wudian_paotai_fire" }, { name: "bshaowei", general: "ow_baolei", mapping: "host:wudian_baolei_switch" }], mm = [{ name: "\u58EB\u517576", faction: "shu", hp: 4, gender: "male", skills: [{ name: "tuji", mapping: "host:wudian_tuji" }, { name: "mujing", mapping: "host:wudian_mujing" }, { name: "lichang", mapping: null, reason: "\u4E0B\u51C6\u5907\u9636\u6BB5\u5B50\u56DE\u5408\u65F6\u70B9\u672A\u5EFA\u6A21\u2014\u20142026-10-08 \u6682\u7F13" }] }, { name: "\u865A\u65E0\u4E4B\u5149", faction: "qun", hp: 3, gender: "female", skills: [{ name: "guangshu", mapping: "host:wudian_guangshu" }] }, { name: "\u7985\u96C5\u5854", faction: "qun", hp: 3, gender: "male", skills: [{ name: "ziyu", mapping: "host:wudian_ziyu" }] }, { name: "\u5929\u4F7F", faction: "shu", hp: 3, gender: "female", skills: [{ name: "shouhu", mapping: "host:wudian_shouhu" }, { name: "ziyu", mapping: "host:wudian_ziyu" }, { name: "feiying", mapping: null, reason: "\u6E90\u5B9A\u4E49\u4E0D\u5728 ow/skill.js\uFF08\u5171\u4EAB\u6280\u80FD\u6587\u4EF6\uFF09\u2014\u20142026-10-08 \u6682\u7F13" }] }, { name: "\u9ED1\u767E\u5408", faction: "wei", hp: 3, gender: "female", skills: [{ name: "duwen", mapping: "host:wudian_duwen" }] }, { name: "\u6258\u6BD4\u6602", faction: "shu", hp: 4, gender: "male", skills: [{ name: "paotai", mapping: "host:wudian_paotai" }, { name: "paotai2", mapping: "host:wudian_paotai_fire" }, { name: "maoding", mapping: null, reason: "\u96F6\u4EF6\u724C\u6C60\uFF08hslingjian\uFF09\u4E0D\u5728\u539F\u751F\u76EE\u5F55\u2014\u20142026-10-08 \u6682\u7F13" }] }, { name: "\u5821\u5792", faction: "shu", hp: 4, gender: "male", skills: [{ name: "bshaowei", mapping: "host:wudian_baolei_switch" }, { name: "zhencha", mapping: "host:wudian_baolei_switch" }, { name: "zhencha_draw", mapping: "host:wudian_zhencha_draw" }] }, { name: "\u5362\u897F\u5965", faction: "wu", hp: 3, gender: "male", skills: [{ name: "yuedong", mapping: "host:wudian_yuedong" }, { name: "kuoyin", mapping: "host:wudian_kuoyin" }, { name: "huhuan", mapping: "host:wudian_huhuan" }] }, { name: "\u7F8E", faction: "wei", hp: 3, gender: "female", skills: [{ name: "jidong", mapping: "host:wudian_jidong" }, { name: "baoxue", mapping: "host:wudian_baoxue" }, { name: "bingqiang", mapping: null, reason: "\u5EA7\u4F4D\u76F8\u90BB/\u8DDD\u79BB\u4FEE\u9970\u591A\u76EE\u6807\uFF08bingqiang2-5 \u5B50\u6280\u65CF+changeTarget \u76F8\u90BB\u6269\u5C55\uFF09\u2014\u20142026-10-08 \u6682\u7F13" }] }, { name: "\u8DEF\u9738", faction: "shu", hp: 4, gender: "male", skills: [{ name: "liangou", mapping: "host:wudian_liangou" }, { name: "xiyang", mapping: "host:wudian_xiyang" }] }, { name: "\u6E90\u6C0F", faction: "qun", hp: 3, gender: "male", skills: [{ name: "feiren", mapping: "host:wudian_feiren" }, { name: "lianpo", mapping: null, reason: "\u6E90\u5B9A\u4E49\u4E0D\u5728 ow/skill.js\uFF08\u5171\u4EAB\u6280\u80FD\u6587\u4EF6\uFF0Cfeiying \u540C\u6B3E\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }, { name: "zhanlong", mapping: null, reason: "limited+createCard \u51ED\u7A7A\u9020\u724C 3 \u6740+zhanlong2 \u4E34\u65F6\u6280\u94FE\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u6CD5\u8001\u4E4B\u9E70", faction: "shu", hp: 3, gender: "female", skills: [{ name: "feidan", mapping: "host:wudian_feidan" }, { name: "huoyu", mapping: null, reason: "limited+\u865A\u62DF useCard\uFF08chiyuxi \u51ED\u7A7A\u7528\u724C\xD72\uFF09\u2014\u2014\u865A\u62DF\u7528\u724C\u539F\u8BED\u540C youzhan \u7F3A\u53E3\uFF0C2026-10-09 \u6682\u7F13" }, { name: "feiying", mapping: null, reason: "\u6E90\u5B9A\u4E49\u4E0D\u5728 ow/skill.js\uFF08\u5171\u4EAB\u6280\u80FD\u6587\u4EF6\uFF09\u2014\u20142026-10-08 \u6682\u7F13" }] }, { name: "\u5B89\u5A1C", faction: "wei", hp: 3, gender: "female", skills: [{ name: "zhiyuan", mapping: "host:wudian_zhiyuan" }, { name: "mianzhen", mapping: null, reason: "\u76EE\u6807\u4FA7 chooseToRespond \u54CD\u5E94\u8BE2\u95EE+mianzhen2 \u6C38\u4E45\u5B50\u6280\u2014\u20142026-10-09 \u6682\u7F13" }, { name: "aqianghua", mapping: null, reason: "\u7ED9\u724C+\u62A4\u7532+aqianghua2 \u5B50\u6280\u94FE\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u9EA6\u514B\u96F7", faction: "shu", hp: 4, gender: "male", skills: [{ name: "shenqiang", mapping: "host:wudian_shenqiang" }, { name: "shanguang", mapping: null, reason: "shanguang2 \u5168\u5361\u724C\u7981\u7528\u5B50\u6280\uFF08mod cardEnabled/cardUsable/cardRespondable/cardSavable \u5168 false\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }, { name: "tiandan", mapping: null, reason: "createCard \u51ED\u7A7A\u9020\u6740\xD7N+\u4E3B\u52A8\u8DF3\u8FC7\u51FA\u724C/\u5F03\u724C\u9636\u6BB5\uFF08skip\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u72C2\u9F20", faction: "shu", hp: 3, gender: "male", skills: [{ name: "shihuo", mapping: "host:wudian_shihuo" }, { name: "liudan", mapping: null, reason: "\u6740\u4F7F\u7528\u65F6 50% \u6982\u7387\u968F\u673A\u8FFD\u52A0\u5408\u6CD5\u76EE\u6807\uFF08RNG \u76EE\u6807\u8FFD\u52A0+lib.filter.targetEnabled \u590D\u7528\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }, { name: "shoujia", mapping: null, reason: "\u7ED9\u724C+shoujia2 \u6302\u724C\u5B50\u6280\uFF08storage \u53CC\u5411\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u83B1\u56E0\u54C8\u7279", faction: "qun", hp: 4, gender: "male", skills: [{ name: "mengji", mapping: "host:wudian_mengji" }, { name: "zhongdun", mapping: null, reason: "init2 \u5F00\u5C40 +8 \u62A4\u7532\uFF08\u65E0 GameStart \u4E8B\u4EF6\u79CD\u7C7B=\u5F00\u5C40\u4E8B\u4EF6\u57FA\u5EFA\u7F3A\u53E3\uFF09+\u4E3B\u52A8\u8F6C\u7532\u6280\u2014\u20142026-10-09 \u6682\u7F13\uFF1Bmengji \u7684 storage.zhongdun \u7F6E\u4F4D\u4EE5\u6301\u6280\u7B49\u4EF7" }] }, { name: "\u534A\u85CF", faction: "qun", hp: 4, gender: "male", skills: [{ name: "yinbo", mapping: "host:wudian_yinbo" }, { name: "bfengshi", mapping: null, reason: "shaBegin Math.random \u6982\u7387\u76F4\u51FB+countUsed \u653B\u8DDD+\u7EC4\u6280 bfengshi2/3/4\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u5965\u4E3D\u838E", faction: "wu", hp: 4, gender: "female", skills: [{ name: "qianggu", mapping: "host:wudian_qianggu" }, { name: "qianggu2", mapping: "host:wudian_qianggu_guard" }, { name: "woliu", mapping: null, reason: "\u56DE\u5408\u7ED3\u675F\u9009 1-2 \u76EE\u6807\u5171\u6302 woliu2 \u7EC4\u8DDD\u79BB/\u6740\u9650\u5B9A+dieBegin/phaseBegin \u6E05\u9664\u7EC4\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u6E29\u65AF\u987F", faction: "shu", hp: 4, gender: "male", skills: [{ name: "feitiao", mapping: "host:wudian_feitiao" }, { name: "dianji", mapping: null, reason: "viewAs \u60CA\u96F7\u6740\uFF08jingleishan \u724C\u79CD\u4E0D\u5728\u539F\u751F\u76EE\u5F55\uFF09+\u8DDD\u79BB 2 \u9650\u5B9A\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u67E5\u8389\u5A05", faction: "wei", hp: 4, gender: "female", skills: [{ name: "pingzhang", mapping: "host:wudian_pingzhang" }, { name: "owliyong", mapping: "host:wudian_owliyong" }] }, { name: "\u9ED1\u5F71", faction: "wei", hp: 3, gender: "female", skills: [{ name: "yinshen", mapping: "host:wudian_yinshen" }, { name: "qinru", mapping: null, reason: "useCardToBegin \u5224\u5B9A+\u8DE8\u56DE\u5408\u76EE\u6807\u5B58\u50A8\u5217\u8868\uFF08qinru_turn \u5206\u6863\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }, { name: "maichong", mapping: null, reason: "qinru \u5B58\u50A8\u94FE\uFF08\u5BF9\u5DF2\u5165\u4FB5\u8005\u968F\u673A\u5F03\uFF09\u2014\u2014\u968F qinru \u6682\u7F13" }] }, { name: "\u6B7B\u795E", faction: "shu", hp: 4, gender: "male", skills: [{ name: "xiandan", mapping: "host:wudian_xiandan" }, { name: "yihun", mapping: null, reason: "\u865A\u62DF useCard\uFF08useCard({name:sha}) \u6307\u5B9A\u76EE\u6807\u56DE\u5408\uFF09\u2014\u2014\u865A\u62DF\u7528\u724C\u539F\u8BED\u540C youzhan/huoyu \u7F3A\u53E3\uFF0C2026-10-09 \u6682\u7F13" }, { name: "shouge", mapping: null, reason: "createCard \u6CBB\u7597\u6CE2\uFF08\u724C\u79CD\u8DE8\u5305\u4F9D\u8D56\u7089\u77F3\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "D.Va", faction: "shu", hp: 2, gender: "female", skills: [{ name: "jijia", mapping: "host:wudian_jijia" }, { name: "tuijin", mapping: null, reason: "\u673A\u7532\u8840>0 \u65F6\u8DDD\u79BB>1 \u76EE\u6807\u63A8\u79FB\uFF08tuijin2 \u8DDD\u79BB\u9650\u5B9A\u5B50\u6280\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }, { name: "zihui", mapping: null, reason: "\u81EA\u7206 AOE\uFF08chooseToDiscard n \u5F20\u6216 2 \u706B\u4F24\uFF0C\u673A\u7532\u8840=\u5F03\u724C\u6570\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }, { name: "chongzhuang", mapping: null, reason: "\u91CD\u88C5\u7D2F\u8BA1\uFF08\u673A\u7532\u8840\u5F52\u96F6\u540E\u4F24\u5BB3\u8BA1\u6570 4 \u590D\u6D3B\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u730E\u7A7A", faction: "shu", hp: 3, gender: "female", skills: [{ name: "shanxian", mapping: null, reason: "\u533A\u57DF\u79BB\u573A\uFF08he \u533A\u724C\u5165 storage \u79BB\u573A\u533A\uFF0Cshanhui \u56DE\u5F52\u590D\u6D3B\u542B\u4F4D\u7F6E s \u7684 createCard \u91CD\u9020\uFF09\u2014\u2014\u724C\u5B88\u6052\u7EAA\u5F8B\u7ED3\u6784\u6027\u4F9D\u8D56\uFF0C2026-10-09 \u6682\u7F13" }, { name: "shanhui", mapping: null, reason: "\u968F shanxian \u79BB\u573A\u5B58\u50A8\u94FE\u2014\u20142026-10-09 \u6682\u7F13" }] }], gm = { source: rm, upstream_path: lm, upstream_commit: um, pack: cm, cards: dm, host_skills: pm, deferred: hm, skills: fm, generals: mm }, _m = { package_format_version: 1, namespace: "upstream-noname", package_id: "shaai-xianjian", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [217, 238, 24, 99, 75, 40, 128, 69, 19, 80, 34, 62, 215, 207, 28, 63, 239, 132, 117, 217, 39, 40, 203, 249, 147, 20, 32, 1, 10, 43, 15, 178], asset_hash: [139, 23, 211, 24, 76, 140, 174, 144, 67, 188, 243, 216, 101, 242, 48, 219, 197, 44, 103, 185, 96, 125, 164, 50, 213, 22, 31, 210, 83, 3, 86, 181], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, vm = [], ym = [], km = [], wm = [], bm = [], xm = [], $m = [], jm = [], Sm = { "zh-CN": { diaobingqianjiang: "\u8C03\u5175\u9063\u5C06", sanjian: "\u4E09\u5C16", wangmeizhike: "\u671B\u6885\u6B62\u6E34", youdishenru: "\u8BF1\u654C\u6DF1\u5165" } }, Em = { schema_version: 1, package_id: "shaai-xianjian", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "shaai-xianjian:cards", source_hash: [125, 162, 65, 35, 137, 186, 77, 70, 223, 174, 61, 71, 106, 251, 20, 203, 164, 54, 152, 191, 36, 6, 206, 58, 190, 205, 23, 208, 222, 190, 253, 65], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, Am = { "shaai-xianjian:cards": "assets/cards.json" }, Cm = "gen-1", Ao = { manifest: _m, modules: vm, skill_index: ym, character_index: km, card_index: wm, trigger_index: bm, modifier_index: xm, active_index: $m, view_as_index: jm, translations: Sm, asset_manifest: Em, asset_paths: Am, compiler_version: Cm }, Rm = "qita/sanguosha/noname/extension/\u6740\u6D77\u62FE\u9057/character/xianjian", zm = "extension/\u6740\u6D77\u62FE\u9057/character/xianjian/", Im = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", Pm = { name: "shaai-xianjian-upstream-noname", version: 1, description: "G1 \u7EB5\u5207\uFF1A\u4E0A\u6E38 xianjian \u661F\u7487\uFF08\u7075\u96EA\uFF09", card_prefix: "xj_" }, Tm = [], Om = [], Nm = [], qm = [{ name: "lingxue", general: "pal_xingxuan", mapping: "host:xj_lingxue" }, { name: "huimeng", general: "pal_zhaoliner", mapping: "host:xj_huimeng" }, { name: "tianshe", general: "pal_zhaoliner", mapping: "host:xj_tianshe" }, { name: "tianshe2", general: "pal_zhaoliner", mapping: "host:xj_tianshe2" }, { name: "fanling", general: "pal_xia", mapping: "host:xj_fanling" }, { name: "guiyuan", general: "pal_linyueru", mapping: "host:xj_guiyuan" }, { name: "tannang", general: "pal_hanlingsha", mapping: "host:xj_tannang" }, { name: "feixu", general: "pal_wangpengxu", mapping: "host:xj_feixu" }, { name: "feng", general: "pal_leiyuange", mapping: "host:xj_feng" }, { name: "ya", general: "pal_leiyuange", mapping: "host:xj_ya" }, { name: "song", general: "pal_leiyuange", mapping: "host:xj_song" }, { name: "xjyufeng", general: "pal_lixiaoyao", mapping: "host:xj_xjyufeng" }, { name: "xuanyan", general: "pal_xuanxiao", mapping: "host:xj_xuanyan" }, { name: "xuanyan2", general: "pal_xuanxiao", mapping: "host:xj_xuanyan2" }, { name: "xuanyan3", general: "pal_xuanxiao", mapping: "host:xj_xuanyan3" }, { name: "ningbin", general: "pal_xuanxiao", mapping: "host:xj_ningbin" }, { name: "diewu", general: "pal_longkui", mapping: "host:xj_diewu" }, { name: "lingyu", general: "pal_longkui", mapping: "host:xj_lingyu" }, { name: "lueying", general: "pal_muchanglan", mapping: "host:xj_lueying" }, { name: "anwugu", general: "pal_anu", mapping: "host:xj_anwugu" }, { name: "anwugu2", general: "pal_anu", mapping: "host:xj_anwugu2" }, { name: "txianqu", general: "pal_tangyurou", mapping: "host:xj_txianqu" }], Mm = [{ name: "\u661F\u7487", faction: "qun", hp: 3, gender: "male", skills: [{ name: "lingxue", mapping: "host:xj_lingxue" }, { name: "feizhua", mapping: null, reason: "useCard \u6740\u5355\u76EE\u6807+\u7EAF\u8DDD 1 \u5185\u73A9\u5BB6\u8BE2\u95EE\u6362\u4F4D\u2014\u20142026-10-09 \u6682\u7F13" }, { name: "leiyu", mapping: null, reason: "phaseEnd \u96F7\u57DF\u5B58\u50A8\u5217\u8868+\u8DE8\u56DE\u5408\u6807\u8BB0\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u6C88\u6B3A\u971C", faction: "wei", hp: 3, gender: "female", skills: [{ name: "binxin", mapping: "host:xj_binxin" }, { name: "qixia", mapping: null, reason: "\u82B1\u8272\u96C6\u96C6\u9F50 4 \u8272 insertPhase \u989D\u5916\u56DE\u5408\uFF08\u5F15\u64CE\u7F3A\u53E3 ADR0646\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }, { name: "jianzhen", mapping: null, reason: "shaAfter \u9010\u4EBA chooseToUse \u5BF9\u540C\u4E00\u76EE\u6807\u7528\u6740\u8BE2\u95EE\u94FE\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u8D75\u7075\u513F", faction: "wei", hp: 3, gender: "female", skills: [{ name: "huimeng", mapping: "host:xj_huimeng" }, { name: "tianshe", mapping: "host:xj_tianshe" }, { name: "tianshe2", mapping: "host:xj_tianshe2" }] }, { name: "\u7455", faction: "wu", hp: 3, gender: "female", skills: [{ name: "xjzongyu", mapping: null, reason: "\u865A\u62DF useCard \u98DE\u9556\uFF08useCard({name:feibiao}) \u968F\u673A 2 \u654C\uFF09\u2014\u2014\u865A\u62DF\u7528\u724C\u539F\u8BED\u7F3A\u53E3\uFF0C2026-10-09 \u6682\u7F13" }, { name: "fanling", mapping: "host:xj_fanling" }] }, { name: "\u6E29\u6167", faction: "wei", hp: 4, gender: "male", skills: [{ name: "longxiang", mapping: "host:xj_longxiang" }, { name: "huxi", mapping: null, reason: "chooseToUse viewAs sha\uFF08\u65E0\u5B9E\u4F53\u724C\u865A\u62DF\u7528\u724C\uFF09+respondSha\u2014\u2014\u865A\u62DF\u7528\u724C\u539F\u8BED\u7F3A\u53E3\uFF0C2026-10-09 \u6682\u7F13" }] }, { name: "\u6797\u6708\u5982", faction: "wei", hp: 3, gender: "female", skills: [{ name: "guiyuan", mapping: "host:xj_guiyuan" }, { name: "qijian", mapping: null, reason: "phaseDiscardEnd \u5F03\u724C\u2192\u865A\u62DF useCard \u6740\uFF08\u65E0\u5B9E\u4F53\u724C\uFF09\u2014\u2014\u865A\u62DF\u7528\u724C\u539F\u8BED\u7F3A\u53E3\uFF0C2026-10-09 \u6682\u7F13" }] }, { name: "\u97E9\u83F1\u7EB1", faction: "qun", hp: 3, gender: "female", skills: [{ name: "tannang", mapping: "host:xj_tannang" }, { name: "tuoqiao", mapping: null, reason: "\u9ED1\u724C\u5F53\u77F3\u4F1A\u7C89\uFF08\u77F3\u4F1A\u7C89\u4E3A\u53E4\u5251\u5361\u7247\u2014\u2014\u8DE8\u5305\u724C\u79CD\u4F9D\u8D56\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u738B\u84EC\u7D6E", faction: "wei", hp: 3, gender: "female", skills: [{ name: "feixu", mapping: "host:xj_feixu" }, { name: "duxinshu", mapping: null, reason: "\u8BFB\u5FC3\uFF1A\u770B\u76EE\u6807\u624B\u724C chooseCardButton \u9009\u4E00\u5F20\u7528\u81EA\u5DF1\u7684\u624B\u724C\u66FF\u6362\uFF08\u6309\u94AE\u9009\u62E9+\u53CC\u5411\u6362\u724C\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u96F7\u5143\u6208", faction: "qun", hp: 4, gender: "male", skills: [{ name: "feng", mapping: "host:xj_feng" }, { name: "ya", mapping: "host:xj_ya" }, { name: "song", mapping: "host:xj_song" }] }, { name: "\u674E\u900D\u9065", faction: "qun", hp: 4, gender: "male", skills: [{ name: "xjyufeng", mapping: "host:xj_xjyufeng" }, { name: "tianjian", mapping: null, reason: "viewAs \u4E07\u7BAD\u9F50\u53D1\uFF08\u6740\u724C\u5F53 AOE\uFF09+tianjian_discard \u4F24\u5BB3\u5F52\u5C5E\u2014\u2014AOE \u865A\u62DF\u7528\u724C+\u5F52\u5C5E\u7F3A\u53E3\uFF0C2026-10-09 \u6682\u7F13" }] }, { name: "\u7384\u9704", faction: "qun", hp: 4, gender: "male", skills: [{ name: "xuanyan", mapping: "host:xj_xuanyan" }, { name: "xuanyan2", mapping: "host:xj_xuanyan2" }, { name: "xuanyan3", mapping: "host:xj_xuanyan3" }, { name: "ningbin", mapping: "host:xj_ningbin" }, { name: "xfenxin", mapping: null, reason: "changeHp \u4E8B\u4EF6\uFF08\u6389\u8840/\u56DE\u590D\u589E\u91CF\u62BD\u724C\uFF09\u2014\u2014\u539F\u751F\u65E0 changeHp \u4E8B\u4EF6\u79CD\uFF08jijia \u7528 commit_damage \u62E6\u622A\u66FF\u4EE3\u53E3\u5F84\uFF09\uFF0C2026-10-09 \u6682\u7F13" }] }, { name: "\u9F99\u8475", faction: "wu", hp: 3, gender: "female", skills: [{ name: "diewu", mapping: "host:xj_diewu" }, { name: "lingyu", mapping: "host:xj_lingyu" }, { name: "fenxing", mapping: null, reason: "\u53CC\u5F62\u6001\u53D8\u8EAB\uFF08Math.random \u5207\u6362 avatar+\u884D\u751F\u6280\u7EC4\uFF09\u2014\u2014\u7ED3\u6784\u6027\u4F9D\u8D56\uFF0C2026-10-09 \u6682\u7F13" }] }, { name: "\u6155\u957F\u5170", faction: "wei", hp: 3, gender: "female", skills: [{ name: "lueying", mapping: "host:xj_lueying" }, { name: "feixia", mapping: null, reason: "\u865A\u62DF useCard \u6740\uFF08\u968F\u673A\u654C\uFF09\u2014\u2014\u865A\u62DF\u7528\u724C\u539F\u8BED\u7F3A\u53E3\uFF0C2026-10-09 \u6682\u7F13" }] }, { name: "\u963F\u5974", faction: "wu", hp: 3, gender: "female", skills: [{ name: "anwugu", mapping: "host:xj_anwugu" }, { name: "anwugu2", mapping: "host:xj_anwugu2" }, { name: "lingdi", mapping: null, reason: "\u8DDD\u79BB\u9012\u589E\u76EE\u6807+\u82B1\u8272\u53BB\u91CD\u5B58\u50A8\u2014\u2014\u591A\u94A9\u5B58\u50A8\u7CFB\uFF0C2026-10-09 \u6682\u7F13" }] }, { name: "\u5510\u96E8\u67D4", faction: "wu", hp: 3, gender: "female", skills: [{ name: "txianqu", mapping: "host:xj_txianqu" }, { name: "qiongguang", mapping: null, reason: "\u7EAF\u8868\u73B0\u6280\uFF08line/expose/delay\uFF0C\u65E0\u673A\u5236\u672C\u4F53\uFF09\u2014\u2014\u8868\u73B0\u5C42 noop\uFF0C2026-10-09 \u767B\u8BB0\u4E0D\u8F6C" }] }, { name: "\u59DC\u57CE", faction: "qun", hp: 4, gender: "male", skills: [{ name: "yanzhan", mapping: null, reason: "\u7EA2\u724C\u8F6C\u706B\u6740 usable:1+\u76EE\u6807 yanzhan3 \u54CD\u5E94\u82B1\u8272 mod+yanzhan2\u21924 \u6740\u9650\u6B21 mod\u2014\u2014viewAs/mod \u7CFB\uFF0C2026-10-09 \u6682\u7F13" }, { name: "fenshi", mapping: "host:xj_fenshi" }, { name: "longhuo", mapping: "host:xj_longhuo", reason_ref: "\u884D\u751F\u6280\u5E38\u9A7B\u5B9E\u4F8B+PlayerState.jiangcheng_longhuo \u95E8\u63A7\u5EFA\u6A21\uFF08\u4E0A\u6E38 addSkill \u52A8\u6001\u83B7\u5F97\uFF09\uFF0C\u89C1\u89C4\u5219\u57FA\u51C6\u767B\u8BB0" }] }, { name: "\u666F\u5929", faction: "wu", hp: 3, gender: "male", skills: [{ name: "sajin", mapping: "host:xj_sajin" }, { name: "jtjubao", mapping: "host:xj_jtjubao" }] }, { name: "\u9F99\u5E7D", faction: "wei", hp: 4, gender: "male", skills: [{ name: "yuexing", mapping: "host:xj_yuexing" }, { name: "minsha", mapping: "host:xj_minsha" }] }, { name: "\u660E\u7EE3", faction: "shu", hp: 3, gender: "female", skills: [{ name: "linghuo", mapping: "host:xj_linghuo" }, { name: "guijin", mapping: "host:xj_guijin" }, { name: "chengxin", mapping: "host:xj_chengxin" }] }, { name: "\u4E91\u5929\u6CB3", faction: "wu", hp: 4, gender: "male", skills: [{ name: "longxi", mapping: null, reason: "chooseToRespond/chooseToUse \u54CD\u5E94\u8BE2\u95EE\u524D\u7F6E\u94A9\u7F3A\u53E3\uFF08\u8BE2\u95EE\u94FE\u6539\u5199\u65CF\uFF09\uFF0C2026-10-09 \u6682\u7F13" }, { name: "zhuyue", mapping: "host:xj_zhuyue" }, { name: "guanri", mapping: "host:xj_guanri" }] }, { name: "\u7D2B\u8431", faction: "shu", hp: 3, gender: "female", skills: [{ name: "shuiyun", mapping: "host:xj_shuiyun" }, { name: "shuiyun5", mapping: "host:xj_shuiyun5", reason_ref: "shuiyun \u5B50\u6280\u5E38\u9A7B\u5B9E\u4F8B\u5316\uFF08\u4E0A\u6E38 subSkill \u4E0D\u5217\u5C06\u8868\uFF09\uFF0C\u6FD2\u6B7B\u6551\u63F4\u901A\u9053\u9996\u4F8B" }, { name: "wangyou", mapping: "host:xj_wangyou" }, { name: "wangyou2", mapping: "host:xj_wangyou2", reason_ref: "wangyou \u5B50\u6280\u5E38\u9A7B\u5B9E\u4F8B\u5316\uFF08silent \u6807\u8BB0\uFF09" }, { name: "changnian", mapping: "host:xj_changnian" }] }, { name: "\u5C0F\u86EE", faction: "shu", hp: 3, gender: "female", skills: [{ name: "anwugu", mapping: "host:xj_anwugu", reason_ref: "\u4E0E\u963F\u5974\u540C\u6280\u2014\u2014mapping \u590D\u7528" }, { name: "lingquan", mapping: "host:xj_lingquan" }, { name: "shenwu", mapping: "host:xj_shenwu" }] }, { name: "\u96EA\u89C1", faction: "shu", hp: 3, gender: "female", skills: [{ name: "xshuangren", mapping: "host:xj_xshuangren" }, { name: "shenmu", mapping: "host:xj_shenmu" }, { name: "duci", mapping: "host:xj_duci" }, { name: "xshuangren2", mapping: "host:xj_xshuangren2", reason_ref: "xshuangren \u5B50\u6280\u5E38\u9A7B\u5B9E\u4F8B\u5316\uFF08dieBegin \u5F03\u7F6E\uFF09" }] }, { name: "\u67F3\u68A6\u7483", faction: "shu", hp: 3, gender: "female", skills: [{ name: "tianxian", mapping: "host:xj_tianxian" }, { name: "runxin", mapping: "host:xj_runxin" }, { name: "xjzhimeng", mapping: "host:xj_xjzhimeng" }, { name: "xjzhimeng2", mapping: "host:xj_xjzhimeng2", reason_ref: "xjzhimeng \u5B50\u6280\u5E38\u9A7B\u5B9E\u4F8B\u5316\uFF08\u76EE\u6807 runtime \u6388\u6280 def \u6E90\uFF09" }, { name: "changnian2", mapping: "host:xj_changnian2", reason_ref: "changnian \u5B50\u6280\u5E38\u9A7B\u5B9E\u4F8B\u5316\uFF08runtime \u6388\u6280 def \u6E90\uFF09" }] }, { name: "\u5357\u5BAB\u714C", faction: "wei", hp: 3, gender: "male", skills: [{ name: "zhaoyao", mapping: "host:xj_zhaoyao" }, { name: "sheling", mapping: "host:xj_sheling" }, { name: "zhangmu", mapping: "host:xj_zhangmu" }] }, { name: "\u738B\u5C0F\u864E", faction: "qun", hp: 4, gender: "male", skills: [{ name: "husha_count", mapping: "host:xj_husha_count", reason_ref: "husha \u5B50\u6280\u5E38\u9A7B\u5B9E\u4F8B\u5316\uFF08source damageEnd \u96C6\u6807\u8BB0\uFF09" }, { name: "husha", mapping: "host:xj_husha", reason_ref: "\u5143\u7D20\u6BC1\u706D\uFF083 \u679A\u9009\u9879\uFF09\u724C\u79CD\u4E0D\u5B58\u5728\u76EE\u5F55\u767B\u8BB0\u4E0D\u8F6C" }] }, { name: "\u6708\u7426", faction: "wei", hp: 3, gender: "female", skills: [{ name: "tianwu", mapping: null, reason: "\u7EAF\u8868\u73B0\u6280\uFF08getDebuff+addTempSkill tianwu2 \u7A7A\u58F3\uFF09\u2014\u2014\u8868\u73B0\u5C42 noop\uFF0C2026-10-09 \u767B\u8BB0\u4E0D\u8F6C" }, { name: "liguang", mapping: "host:xj_liguang" }, { name: "shiying", mapping: "host:xj_shiying" }] }, { name: "\u590F\u4FAF\u747E\u8F69", faction: "wei", hp: 3, gender: "male", skills: [{ name: "xuanmo", mapping: "host:xj_xuanmo" }, { name: "danqing", mapping: null, reason: "\u56DB\u82B1\u8272\u6536\u96C6\u2192getBuff(false) \u7EAF\u8868\u73B0\u76EE\u6807\u9009\u62E9\uFF0C\u65E0\u673A\u5236\u672C\u4F53\u2014\u2014\u8868\u73B0\u5C42\u767B\u8BB0\u4E0D\u8F6C\uFF0C2026-10-09" }] }, { name: "\u82CF\u5A9A", faction: "shu", hp: 3, gender: "female", skills: [{ name: "sheying", mapping: "host:xj_sheying" }, { name: "dujiang", mapping: null, reason: "\u83B7\u5F97\u3010\u6BD2\u3011\u724C\u2014\u2014\u6BD2\u724C\u79CD\u4E0D\u5728\u76EE\u5F55\uFF08createCard \u5B9E\u4F53\u53D1\u73B0\u4EA6\u4E0D\u53EF\u7528\uFF09\uFF0C2026-10-09 \u6682\u7F13" }, { name: "huahu", mapping: "host:xj_huahu" }] }, { name: "\u4ED9\u537F", faction: "qun", hp: 4, gender: "male", skills: [{ name: "xtanxi", mapping: "host:xj_xtanxi" }, { name: "xiaoyue", mapping: null, reason: "roundStart \u4E8B\u4EF6\u7F3A\u53E3+\u81EA\u52A8\u865A\u62DF\u7528\u724C\uFF08useCard \u6740+\u65E0\u8DDD\u79BB\u9650\u5236 mod\uFF09\u590D\u5408\u7F3A\u53E3\uFF0C2026-10-09 \u6682\u7F13" }] }, { name: "\u7F57\u62DB\u5AE3", faction: "shu", hp: 4, gender: "female", skills: [{ name: "fenglue", mapping: "host:xj_fenglue" }, { name: "tanhua", mapping: null, reason: "Recover \u524D\u7F6E\u7A97\u53E3\u7F3A\u53E3\uFF08\u56DE\u590D\u6539\u4E3A\u6478\u724C\u9700\u56DE\u590D\u53D6\u6D88\u8BED\u4E49\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u59DC\u4E91\u51E1", faction: "wei", hp: 4, gender: "male", skills: [{ name: "xunying", mapping: "host:xj_xunying" }, { name: "liefeng", mapping: "host:xj_liefeng" }, { name: "tianjian", mapping: "host:xj_tianjian", reason_ref: "liefeng runtime \u6388\u6280\u76EE\u6807 def \u6E90\uFF08\u76EE\u5F55\u7ECF mapping \u6CE8\u518C\uFF09" }] }, { name: "\u6155\u5BB9\u7D2B\u82F1", faction: "wei", hp: 4, gender: "male", skills: [{ name: "poyun", mapping: "host:xj_poyun" }, { name: "qianfang", mapping: "host:xj_qianfang" }, { name: "xuanning", mapping: null, reason: "\u8BA1\u6570\u6807\u8BB0\u83B7\u5F97\u8DEF\u5F84\u4E0A\u6E38\u7F3A\u5931\uFF08storage.xuanning \u65E0\u6280\u80FD\u5199\u5165\uFF09\u2014\u2014\u539F\u751F\u4EC5\u652F\u6301\u8C03\u8BD5/\u5BBF\u4E3B\u6CE8\u5165\uFF0C2026-10-09 \u767B\u8BB0\u5DEE\u5F02" }] }, { name: "\u8D8A\u4ECA\u671D", faction: "wei", hp: 4, gender: "male", skills: [{ name: "ywuhun", mapping: null, reason: "\u56DE\u5408\u5F00\u59CB\u524D\u989D\u5916\u56DE\u5408+\u56DE\u5408\u540E\u590D\u539F\u573A\u4E0A\u53CA\u724C\u5806\u5168\u90E8\u724C\u2014\u2014\u53CC\u5F15\u64CE\u7F3A\u53E3\uFF08\u989D\u5916\u56DE\u5408=ADR0646 \u767B\u8BB0\uFF09\uFF0C2026-10-10 \u6682\u7F13" }, { name: "xjyingfeng", mapping: "host:xj_xjyingfeng" }] }, { name: "\u5C45\u5341\u65B9", faction: "shu", hp: 3, gender: "male", skills: [{ name: "xiepan", mapping: "host:xj_xiepan" }, { name: "yujia", mapping: null, reason: "\u673A\u5173\u724C\u79CD\u4E0D\u5728\u76EE\u5F55\uFF08\u89C2\u770B/\u8BA1\u6570\u4F9D\u8D56\u673A\u5173\u724C\u5B9E\u4F53\u6C60\uFF09\u2014\u2014dujiang \u540C\u65CF\u7F3A\u53E3\uFF0C2026-10-10 \u6682\u7F13" }, { name: "yanshi", mapping: null, reason: "\u673A\u5173\u724C\u79CD\u4E0D\u5728\u76EE\u5F55\uFF08\u7D2F\u8BA1\u4F7F\u7528\u673A\u5173\u724C\u8BA1\u6570+\u968F\u673A\u88C5\u5907\u586B\u6EE1\u53EF\u6302\u5B9E\u4F53\u53D1\u73B0\uFF0C\u524D\u7F6E=\u673A\u5173\u724C\u79CD\u57FA\u5EFA\uFF09\uFF0C2026-10-10 \u6682\u7F13" }] }, { name: "\u957F\u537F", faction: "wei", hp: 4, gender: "male", skills: [{ name: "luanjian", mapping: "host:xj_luanjian" }, { name: "luanjian2", mapping: "host:xj_luanjian2", reason_ref: "luanjian \u589E\u4F24 subskill\uFF08DamageBefore forced 50%\uFF0C\u8F7D\u4F53\u724C\u951A\u5B9A\uFF09" }, { name: "ctianfu", mapping: "host:xj_ctianfu" }, { name: "ctianfu2", mapping: "host:xj_ctianfu2", reason_ref: "\u5929\u7B26\xB7\u53CD\u566C subskill\uFF08Damage \u4F24\u5BB3\u540E\u7A97\uFF0C\u6302\u9644\u6807\u8BB0+\u5B9E\u4F8B\u6210\u5BF9\uFF09" }, { name: "ctianfu3", mapping: "host:xj_ctianfu3", reason_ref: "\u5929\u7B26\xB7\u6E05\u6302 subskill\uFF08Death \u7A97\u6E05\u7406\uFF09" }] }], Dm = { source: Rm, upstream_path: zm, upstream_commit: Im, pack: Pm, cards: Tm, host_skills: Om, deferred: Nm, skills: qm, generals: Mm }, Lm = { package_format_version: 1, namespace: "upstream-noname", package_id: "shahai-gujian", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [174, 19, 1, 143, 26, 75, 62, 188, 159, 67, 126, 42, 118, 12, 107, 113, 203, 163, 207, 147, 194, 208, 143, 29, 74, 238, 216, 95, 37, 52, 158, 6], asset_hash: [64, 167, 122, 159, 149, 48, 209, 126, 144, 16, 221, 246, 166, 188, 56, 151, 231, 155, 253, 19, 203, 98, 144, 68, 6, 23, 156, 243, 43, 177, 22, 35], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, Um = [], Fm = [], Vm = [], Hm = [], Wm = [], Bm = [], Km = [], Gm = [], Jm = { "zh-CN": { bingpotong: "\u51B0\u9B44\u901A", chunbing: "\u6625\u997C", dinvxuanshuang: "\u5E1D\u5973\u7384\u971C", feibiao: "\u98DE\u9556", gjyuheng: "\u7389\u8861", gudonggeng: "\u5495\u549A\u7FB9", gujian: "\u53E4\u5251\u5947\u8C2D", heilonglinpian: "\u9ED1\u9F99\u9CDE\u7247", jinlianzhu: "\u91D1\u83B2\u70DB", liufengsan: "\u516D\u98CE\u6563", liutouge: "\u786B\u5934\u9601", liyutang: "\u9CA4\u9C7C\u6C64", longxugou: "\u9F99\u987B\u94A9", luyugeng: "\u5364\u7389\u7FB9", mapodoufu: "\u9EBB\u5A46\u8C46\u8150", mianlijinzhen: "\u9762\u91CC\u91D1\u9488", mizhilianou: "\u871C\u94FE\u85D5", molicha: "\u9EBB\u8336", mutoumianju: "\u6728\u5934\u9762\u5177", qiankunbiao: "\u4E7E\u5764\u9556", qingtuan: "\u9752\u56E2", shatang: "\u7802\u7CD6", shenhuofeiya: "\u795E\u706B\u98DE\u9E26", shihuifen: "\u77F3\u4F1A\u7C89", shujinsan: "\u8212\u7B4B\u6563", tanhuadong: "\u644A\u9EC4\u51BB", xiajiao: "\u83F1\u89D2", yougeng: "\u6709\u66F4", yuanbaorou: "\u5143\u5B9D\u8089", yunvyuanshen: "\u7389\u5973\u5143\u795E", ziyangdan: "\u7D2B\u9633\u4E39" } }, Ym = { schema_version: 1, package_id: "shahai-gujian", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "shahai-gujian:cards", source_hash: [59, 14, 97, 152, 140, 199, 211, 59, 2, 113, 112, 213, 241, 164, 249, 98, 134, 209, 174, 76, 85, 135, 152, 123, 139, 5, 42, 72, 174, 40, 183, 10], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, Qm = { "shahai-gujian:cards": "assets/cards.json" }, Xm = "gen-1", Co = { manifest: Lm, modules: Um, skill_index: Fm, character_index: Vm, card_index: Hm, trigger_index: Wm, modifier_index: Bm, active_index: Km, view_as_index: Gm, translations: Jm, asset_manifest: Ym, asset_paths: Qm, compiler_version: Xm }, Zm = "noname", eg = "apps/core/extension/\u6740\u6D77\u62FE\u9057/card/gujian/", ng = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", tg = { name: "shahai-gujian-upstream-noname", version: 1, description: "G1 \u6740\u6D77\u62FE\u9057\xB7\u53E4\u5251\u5207\u7247", card_prefix: "gj_" }, sg = [{ name: "shujinsan", kind: "shujinsan", suit: "diamond", rank: 6 }, { name: "shujinsan", kind: "shujinsan", suit: "spade", rank: 2 }, { name: "mianlijinzhen", kind: "mianlijinzhen", suit: "spade", rank: 1 }, { name: "feibiao", kind: "feibiao", suit: "club", rank: 8 }, { name: "feibiao", kind: "feibiao", suit: "diamond", rank: 9 }, { name: "jinlianzhu", kind: "jinlianzhu", suit: "heart", rank: 9 }, { name: "jinlianzhu", kind: "jinlianzhu", suit: "spade", rank: 7 }, { name: "tanhuadong", kind: "tanhuadong", suit: "spade", rank: 2 }, { name: "yougeng", kind: "yougeng", suit: "heart", rank: 12 }, { name: "qingtuan", kind: "qingtuan", suit: "spade", rank: 3 }, { name: "yuanbaorou", kind: "yuanbaorou", suit: "spade", rank: 8 }, { name: "gudonggeng", kind: "gudonggeng", suit: "heart", rank: 8 }, { name: "mapodoufu", kind: "mapodoufu", suit: "heart", rank: 4 }, { name: "chunbing", kind: "chunbing", suit: "club", rank: 3 }, { name: "liyutang", kind: "liyutang", suit: "heart", rank: 1 }, { name: "molicha", kind: "molicha", suit: "club", rank: 1 }, { name: "xiajiao", kind: "xiajiao", suit: "diamond", rank: 6 }, { name: "mizhilianou", kind: "mizhilianou", suit: "diamond", rank: 4 }, { name: "luyugeng", kind: "luyugeng", suit: "club", rank: 11 }, { name: "bingpotong", kind: "bingpotong", suit: "spade", rank: 3 }, { name: "bingpotong", kind: "bingpotong", suit: "club", rank: 12 }, { name: "qiankunbiao", kind: "qiankunbiao", suit: "spade", rank: 9 }, { name: "qiankunbiao", kind: "qiankunbiao", suit: "club", rank: 13 }, { name: "shenhuofeiya", kind: "shenhuofeiya", suit: "diamond", rank: 9 }, { name: "longxugou", kind: "longxugou", suit: "spade", rank: 7 }, { name: "ziyangdan", kind: "ziyangdan", suit: "diamond", rank: 6 }, { name: "ziyangdan", kind: "ziyangdan", suit: "heart", rank: 1 }, { name: "yunvyuanshen", kind: "yunvyuanshen", suit: "heart", rank: 13 }, { name: "shihuifen", kind: "shihuifen", suit: "club", rank: 5 }, { name: "shihuifen", kind: "shihuifen", suit: "club", rank: 1 }, { name: "shihuifen", kind: "shihuifen", suit: "spade", rank: 13 }, { name: "liufengsan", kind: "liufengsan", suit: "club", rank: 6 }, { name: "liufengsan", kind: "liufengsan", suit: "club", rank: 3 }, { name: "shatang", kind: "shatang", suit: "heart", rank: 13 }, { name: "heilonglinpian", kind: "heilonglinpian", suit: "spade", rank: 2 }, { name: "liutouge", kind: "liutouge", suit: "heart", rank: 6 }, { name: "liutouge", kind: "liutouge", suit: "club", rank: 6 }, { name: "mutoumianju", kind: "mutoumianju", suit: "club", rank: 4 }, { name: "dinvxuanshuang", kind: "dinvxuanshuang", suit: "heart", rank: 9 }, { name: "gjyuheng", kind: "gjyuheng", suit: "spade", rank: 7 }], ag = [], ig = { source: Zm, upstream_path: eg, upstream_commit: ng, pack: tg, cards: sg, deferred: ag }, og = { package_format_version: 1, namespace: "upstream-noname", package_id: "shahai-gwent", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [11, 210, 86, 154, 147, 71, 161, 144, 244, 225, 236, 145, 131, 209, 115, 183, 233, 66, 140, 230, 24, 60, 18, 155, 254, 3, 178, 198, 44, 142, 171, 205], asset_hash: [81, 129, 91, 229, 139, 77, 173, 158, 200, 141, 186, 65, 162, 83, 196, 174, 250, 135, 67, 199, 255, 82, 208, 132, 191, 127, 138, 83, 57, 29, 198, 231], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, rg = [], lg = [], ug = [], cg = [], dg = [], pg = [], hg = [], fg = [], mg = { "zh-CN": { aozuzhilei: "\u5965\u7956\u4E4B\u96F7", baoxueyaoshui: "\u66B4\u96EA\u836F\u6C34", birinongwu: "\u78A7\u65E5\u6D53\u96FE", ciguhanshuang: "\u523A\u9AA8\u5BD2\u971C", gwent: "gwent", kunenfayin: "\u6606\u6069\u6CD5\u5370", poxiao: "\u7834\u6653", qinpendayu: "\u503E\u76C6\u5927\u96E8", shanbengshu: "\u5C71\u5D29\u672F", wenyi: "\u761F\u75AB", yanziyaoshui: "\u71D5\u5B50\u836F\u6C34", zhihuanjun: "\u7F6E\u6362\u519B", zumoshoukao: "\u7956\u58A8\u624B\u94E0" } }, gg = { schema_version: 1, package_id: "shahai-gwent", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "shahai-gwent:cards", source_hash: [166, 48, 247, 15, 211, 138, 196, 50, 148, 242, 250, 36, 174, 15, 79, 156, 9, 156, 56, 156, 45, 66, 129, 66, 55, 226, 111, 253, 188, 124, 214, 218], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, _g = { "shahai-gwent:cards": "assets/cards.json" }, vg = "gen-1", Ro = { manifest: og, modules: rg, skill_index: lg, character_index: ug, card_index: cg, trigger_index: dg, modifier_index: pg, active_index: hg, view_as_index: fg, translations: mg, asset_manifest: gg, asset_paths: _g, compiler_version: vg }, yg = "noname", kg = "apps/core/extension/\u6740\u6D77\u62FE\u9057/card/gwent/", wg = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", bg = { name: "shahai-gwent-upstream-noname", version: 1, description: "G1 \u6740\u6D77\u62FE\u9057\xB7gwent \u5207\u7247", card_prefix: "gw_" }, xg = [{ name: "yanziyaoshui", kind: "yanziyaoshui", suit: "heart", rank: 8 }, { name: "yanziyaoshui", kind: "yanziyaoshui", suit: "club", rank: 7 }, { name: "aozuzhilei", kind: "aozuzhilei", suit: "spade", rank: 9 }, { name: "aozuzhilei", kind: "aozuzhilei", suit: "club", rank: 3 }, { name: "wenyi", kind: "wenyi", suit: "club", rank: 5 }, { name: "zhihuanjun", kind: "zhihuanjun", suit: "club", rank: 1 }, { name: "zhihuanjun", kind: "zhihuanjun", suit: "spade", rank: 3 }, { name: "baoxueyaoshui", kind: "baoxueyaoshui", suit: "heart", rank: 4 }, { name: "baoxueyaoshui", kind: "baoxueyaoshui", suit: "spade", rank: 8 }, { name: "qinpendayu", kind: "qinpendayu", suit: "diamond", rank: 5 }, { name: "qinpendayu", kind: "qinpendayu", suit: "club", rank: 7 }, { name: "birinongwu", kind: "birinongwu", suit: "spade", rank: 9 }, { name: "birinongwu", kind: "birinongwu", suit: "heart", rank: 13 }, { name: "ciguhanshuang", kind: "ciguhanshuang", suit: "diamond", rank: 11 }, { name: "ciguhanshuang", kind: "ciguhanshuang", suit: "club", rank: 7 }, { name: "kunenfayin", kind: "kunenfayin", suit: "spade", rank: 11 }, { name: "shanbengshu", kind: "shanbengshu", suit: "spade", rank: 8 }, { name: "poxiao", kind: "poxiao", suit: "heart", rank: 7 }, { name: "poxiao", kind: "poxiao", suit: "diamond", rank: 4 }, { name: "zumoshoukao", kind: "zumoshoukao", suit: "spade", rank: 2 }, { name: "zumoshoukao", kind: "zumoshoukao", suit: "club", rank: 5 }], $g = [], jg = { source: yg, upstream_path: kg, upstream_commit: wg, pack: bg, cards: xg, deferred: $g }, Sg = { package_format_version: 1, namespace: "upstream-noname", package_id: "shahai-hearth", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [222, 175, 195, 28, 126, 229, 205, 192, 134, 104, 195, 255, 215, 43, 199, 51, 28, 232, 140, 32, 174, 71, 203, 169, 230, 49, 163, 84, 168, 127, 59, 118], asset_hash: [19, 173, 66, 158, 166, 22, 115, 40, 86, 41, 2, 15, 142, 182, 166, 113, 120, 248, 54, 209, 90, 199, 119, 75, 143, 71, 43, 245, 55, 14, 41, 93], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, Eg = [], Ag = [], Cg = [], Rg = [], zg = [], Ig = [], Pg = [], Tg = [], Og = { "zh-CN": { chuansongmen: "\u4F20\u9001\u95E8", dunpaigedang: "\u76FE\u724C\u683C\u6321", hearth: "\u7089\u77F3\u4F20\u8BF4", jihuocard: "\u6025\u8017\u5361", linghunzhihuo: "\u7075\u9B42\u4E4B\u706B", shandianjian: "\u95EA\u7535\u5251", shenenshu: "\u795E\u6069\u672F", tanshezhiren: "\u5766\u86C7\u81F3\u4EBA", xingjiegoutong: "\u661F\u754C\u6C9F\u901A", yuansuhuimie: "\u5143\u7D20\u6BC1\u706D", zhaomingdan: "\u7167\u660E\u5F39", zhiliaobo: "\u6CBB\u7597\u6CE2" } }, Ng = { schema_version: 1, package_id: "shahai-hearth", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "shahai-hearth:cards", source_hash: [224, 12, 104, 161, 208, 213, 217, 129, 88, 249, 189, 119, 106, 114, 77, 244, 33, 155, 19, 99, 87, 24, 224, 112, 101, 175, 244, 191, 178, 203, 181, 243], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, qg = { "shahai-hearth:cards": "assets/cards.json" }, Mg = "gen-1", zo = { manifest: Sg, modules: Eg, skill_index: Ag, character_index: Cg, card_index: Rg, trigger_index: zg, modifier_index: Ig, active_index: Pg, view_as_index: Tg, translations: Og, asset_manifest: Ng, asset_paths: qg, compiler_version: Mg }, Dg = "noname", Lg = "apps/core/extension/\u6740\u6D77\u62FE\u9057/card/hearth/", Ug = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", Fg = { name: "shahai-hearth-upstream-noname", version: 1, description: "G1 \u7089\u77F3\u4F20\u8BF4\u5207\u7247", card_prefix: "ht_" }, Vg = [{ name: "linghunzhihuo", kind: "linghunzhihuo", suit: "diamond", rank: 2 }, { name: "zhaomingdan", kind: "zhaomingdan", suit: "diamond", rank: 5 }, { name: "zhaomingdan", kind: "zhaomingdan", suit: "heart", rank: 10 }, { name: "jihuocard", kind: "jihuocard", suit: "diamond", rank: 2 }, { name: "shandianjian", kind: "leisha", suit: "club", rank: 3 }, { name: "shandianjian", kind: "leisha", suit: "spade", rank: 1 }, { name: "shenenshu", kind: "jihuocard", suit: "heart", rank: 2 }, { name: "shenenshu", kind: "jihuocard", suit: "diamond", rank: 12 }, { name: "dunpaigedang", kind: "dunpaigedang", suit: "club", rank: 3 }, { name: "zhiliaobo", kind: "zhiliaobo", suit: "club", rank: 7 }, { name: "zhiliaobo", kind: "zhiliaobo", suit: "spade", rank: 1 }, { name: "yuansuhuimie", kind: "yuansuhuimie", suit: "spade", rank: 13 }, { name: "xingjiegoutong", kind: "xingjiegoutong", suit: "spade", rank: 13 }, { name: "chuansongmen", kind: "chuansongmen", suit: "diamond", rank: 2 }, { name: "chuansongmen", kind: "chuansongmen", suit: "heart", rank: 2 }, { name: "tanshezhiren", kind: "tanshezhiren", suit: "diamond", rank: 2 }, { name: "shijieshu", kind: "shijieshu", suit: "spade", rank: 7 }], Hg = [], Wg = { source: Dg, upstream_path: Lg, upstream_commit: Ug, pack: Fg, cards: Vg, deferred: Hg }, Bg = { package_format_version: 1, namespace: "upstream-noname", package_id: "shahai-mtg", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [100, 23, 190, 115, 240, 231, 83, 4, 12, 207, 180, 163, 203, 162, 240, 254, 137, 75, 251, 157, 145, 232, 234, 173, 222, 68, 39, 243, 228, 135, 129, 98], asset_hash: [138, 246, 212, 134, 176, 90, 183, 36, 117, 169, 120, 78, 6, 33, 28, 70, 108, 98, 68, 51, 36, 115, 35, 162, 65, 214, 7, 150, 89, 14, 168, 195], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, Kg = [], Gg = [], Jg = [], Yg = [], Qg = [], Xg = [], Zg = [], e_ = [], n_ = { schema_version: 1, package_id: "shahai-mtg", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "shahai-mtg:cards", source_hash: [90, 88, 252, 38, 173, 223, 90, 144, 111, 137, 77, 35, 246, 175, 221, 252, 104, 231, 28, 131, 135, 65, 225, 157, 151, 243, 9, 35, 13, 249, 156, 220], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, t_ = { "shahai-mtg:cards": "assets/cards.json" }, s_ = "test", Io = { manifest: Bg, modules: Kg, skill_index: Gg, character_index: Jg, card_index: Yg, trigger_index: Qg, modifier_index: Xg, active_index: Zg, view_as_index: e_, asset_manifest: n_, asset_paths: t_, compiler_version: s_ }, a_ = "qita/sanguosha/noname/extension/\u6740\u6D77\u62FE\u9057/card/mtg", i_ = "extension/\u6740\u6D77\u62FE\u9057/card/mtg/", o_ = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", r_ = { name: "shahai-mtg-upstream-noname", version: 1, description: "G1 \u8F6C\u6362\uFF1A\u4E0A\u6E38 mtg \u9996\u5207\u7247\uFF08\u9F99\u9885\u5C71\u5CF0\uFF09", card_prefix: "mtg_" }, l_ = [{ name: "mtg_longlushanfeng", kind: "longlushanfeng", suit: "club", rank: 10 }, { name: "mtg_haidao", kind: "haidao", suit: "diamond", rank: 5 }, { name: "mtg_shuimomuxue", kind: "shuimomuxue", suit: "spade", rank: 11 }, { name: "mtg_feixu", kind: "feixu", suit: "club", rank: 8 }, { name: "mtg_yixialan", kind: "yixialan", suit: "club", rank: 12 }, { name: "mtg_shamolvzhou", kind: "shamolvzhou", suit: "heart", rank: 6 }, { name: "mtg_linzhongjianta", kind: "linzhongjianta", suit: "heart", rank: 6 }, { name: "mtg_bingheyaosai", kind: "bingheyaosai", suit: "diamond", rank: 5 }, { name: "mtg_youlin", kind: "youlin", suit: "club", rank: 10 }, { name: "mtg_lindixiliu", kind: "lindixiliu", suit: "spade", rank: 11 }], u_ = [], c_ = [{ name: "mtg_duzhao", reason: "\u4F9D\u8D56 xianxia \u5305\u3010\u6BD2\u3011\u5361\uFF08\u8DE8\u5305\u724C\u79CD\u4F9D\u8D56\u540C tuoqiao \u5148\u4F8B\uFF09\u2014\u20142026-10-11 \u6682\u7F13\uFF0C\u968F\u6BD2\u5361\u6E90\u8F6C\u6362\u89E3\u9501" }, { name: "mtg_cangbaohaiwan", reason: "\u4F9D\u8D56\u6D3E\u751F\u5305\u9009\u62E9 UI\uFF08derivation \u5305\u5217\u8868\u4EA4\u4E92\u9009\u724C\uFF09\u2014\u2014\u539F\u751F\u65E0\u8BE5\u673A\u5236\uFF0C2026-10-11 \u6682\u7F13" }], d_ = { source: a_, upstream_path: i_, upstream_commit: o_, pack: r_, cards: l_, host_skills: u_, deferred: c_ }, p_ = { package_format_version: 1, namespace: "upstream-noname", package_id: "shahai-swd", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [49, 204, 234, 152, 242, 141, 74, 177, 190, 148, 159, 91, 147, 112, 4, 119, 125, 19, 159, 58, 106, 197, 84, 110, 16, 191, 141, 30, 209, 141, 195, 127], asset_hash: [199, 119, 92, 146, 166, 89, 110, 115, 216, 130, 121, 121, 62, 47, 65, 102, 152, 166, 25, 69, 169, 82, 236, 165, 122, 112, 127, 20, 153, 39, 8, 196], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, h_ = [], f_ = [], m_ = [], g_ = [], __ = [], v_ = [], y_ = [], k_ = [], w_ = { schema_version: 1, package_id: "shahai-swd", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "shahai-swd:cards", source_hash: [70, 23, 137, 164, 203, 28, 38, 181, 129, 120, 81, 208, 138, 2, 175, 224, 228, 196, 230, 118, 108, 225, 193, 254, 181, 155, 126, 170, 60, 93, 71, 82], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, b_ = { "shahai-swd:cards": "assets/cards.json" }, x_ = "test", Po = { manifest: p_, modules: h_, skill_index: f_, character_index: m_, card_index: g_, trigger_index: __, modifier_index: v_, active_index: y_, view_as_index: k_, asset_manifest: w_, asset_paths: b_, compiler_version: x_ }, $_ = "noname", j_ = "apps/core/extension/\u6740\u6D77\u62FE\u9057/card/swd/", S_ = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", E_ = { name: "shahai-swd-upstream-noname", version: 1, description: "G1 \u8F6C\u6362\uFF1A\u4E0A\u6E38 swd \u9996\u5207\u7247\uFF08\u62A4\u7B26\uFF09", card_prefix: "swd_", init_hook: "swd_jiqi_held" }, A_ = [{ name: "swd_hufu", kind: "hufu", suit: "spade", rank: 1 }, { name: "swd_hufu2", kind: "hufu", suit: "heart", rank: 1 }, { name: "swd_liuxinghuoyu1", kind: "liuxinghuoyu", suit: "diamond", rank: 3 }, { name: "swd_liuxinghuoyu2", kind: "liuxinghuoyu", suit: "heart", rank: 6 }, { name: "swd_liuxinghuoyu3", kind: "liuxinghuoyu", suit: "heart", rank: 9 }, { name: "swd_caoyao1", kind: "caoyao", suit: "club", rank: 3 }, { name: "swd_caoyao2", kind: "caoyao", suit: "club", rank: 4 }, { name: "swd_caoyao3", kind: "caoyao", suit: "club", rank: 5 }, { name: "swd_chilongya", kind: "chilongya", suit: "diamond", rank: 3 }, { name: "swd_tianxianjiu1", kind: "tianxianjiu", suit: "heart", rank: 9 }, { name: "swd_tianxianjiu2", kind: "tianxianjiu", suit: "diamond", rank: 2 }, { name: "swd_xixueguizhihuan", kind: "xixueguizhihuan", suit: "spade", rank: 5 }, { name: "swd_guiyoujie1", kind: "guiyoujie", suit: "spade", rank: 3 }, { name: "swd_guiyoujie2", kind: "guiyoujie", suit: "club", rank: 9 }, { name: "swd_shentoumianju", kind: "shentoumianju", suit: "club", rank: 6 }, { name: "swd_xingjunyan", kind: "xingjunyan", suit: "spade", rank: 7 }, { name: "swd_qinglonglingzhu", kind: "qinglonglingzhu", suit: "spade", rank: 2 }, { name: "swd_zhufangshenshi", kind: "zhufangshenshi", suit: "spade", rank: 4 }, { name: "swd_fengxueren", kind: "fengxueren", suit: "club", rank: 1 }, { name: "swd_yufulu", kind: "yufulu", suit: "spade", rank: 6 }, { name: "swd_yangpijuan", kind: "yangpijuan", suit: "spade", rank: 9 }, { name: "swd_daihuofenglun", kind: "daihuofenglun", suit: "heart", rank: 1 }, { name: "swd_xiayuncailing", kind: "xiayuncailing", suit: "diamond", rank: 2 }, { name: "swd_langeguaiyi", kind: "langeguaiyi", suit: "diamond", rank: 1 }, { name: "swd_zhiluxiaohu", kind: "jiguan", suit: "heart", rank: 3 }, { name: "swd_mujiaren", kind: "jiguan", suit: "club", rank: 7 }, { name: "swd_gouhunluo1", kind: "jiguan", suit: "spade", rank: 9 }, { name: "swd_gouhunluo2", kind: "jiguan", suit: "club", rank: 7 }, { name: "swd_jiguantong1", kind: "jiguan", suit: "diamond", rank: 4 }, { name: "swd_jiguantong2", kind: "jiguan", suit: "club", rank: 7 }, { name: "swd_jiguanfeng1", kind: "jiguan", suit: "club", rank: 3 }, { name: "swd_jiguanfeng2", kind: "jiguan", suit: "spade", rank: 4 }, { name: "swd_jiguanyuan1", kind: "jiguan", suit: "club", rank: 3 }, { name: "swd_jiguanyuan2", kind: "jiguan", suit: "diamond", rank: 2 }, { name: "swd_jiguanyaoshu1", kind: "jiguan", suit: "club", rank: 3 }, { name: "swd_jiguanyaoshu2", kind: "jiguan", suit: "spade", rank: 3 }, { name: "swd_xuanfengzhiren1", kind: "hslingjian", suit: "diamond", rank: 4 }, { name: "swd_zhongxinghujia1", kind: "hslingjian", suit: "heart", rank: 4 }, { name: "swd_xingtigaizao1", kind: "hslingjian", suit: "spade", rank: 6 }, { name: "swd_shijianhuisu1", kind: "hslingjian", suit: "diamond", rank: 6 }, { name: "swd_jinjilengdong1", kind: "hslingjian", suit: "spade", rank: 8 }, { name: "swd_shengxiuhaojiao1", kind: "hslingjian", suit: "heart", rank: 8 }, { name: "swd_yinmilichang1", kind: "hslingjian", suit: "club", rank: 10 }, { name: "swd_jiguanshu1", kind: "jiguan", suit: "club", rank: 2 }, { name: "swd_jiguanshu2", kind: "jiguan", suit: "diamond", rank: 2 }, { name: "swd_zhufangshenshi2", kind: "zhufangshenshi", suit: "spade", rank: 8 }, { name: "swd_zhiluxiaohu2", kind: "jiguan", suit: "diamond", rank: 4 }, { name: "swd_mujiaren2", kind: "jiguan", suit: "heart", rank: 6 }, { name: "swd_mujiaren3", kind: "jiguan", suit: "diamond", rank: 11 }, { name: "swd_yangpijuan2", kind: "yangpijuan", suit: "club", rank: 7 }, { name: "swd_xuejibingbao", kind: "xuejibingbao", suit: "club", rank: 1 }, { name: "swd_xuejibingbao2", kind: "xuejibingbao", suit: "spade", rank: 1 }, { name: "swd_fengyinzhidan", kind: "fengyinzhidan", suit: "spade", rank: 1 }, { name: "swd_shenmiguo", kind: "shenmiguo", suit: "spade", rank: 2 }, { name: "swd_shenmiguo2", kind: "shenmiguo", suit: "heart", rank: 1 }, { name: "swd_lianyaohu", kind: "lianyaohu", suit: "club", rank: 13, equip_skills: ["lianyaohu_shouna", "lianyaohu_lianhua"] }, { name: "swd_qiankundai", kind: "qiankundai", suit: "club", rank: 1 }, { name: "swd_shennongding", kind: "shennongding", suit: "club", rank: 13, equip_skills: ["shennongding"] }, { name: "swd_nvwashi", kind: "nvwashi", suit: "heart", rank: 13, equip_skills: ["nvwashi"] }, { name: "swd_kunlunjingc", kind: "kunlunjingc", suit: "spade", rank: 13, equip_skills: ["kunlunjingc"] }, { name: "swd_donghuangzhong", kind: "donghuangzhong", suit: "diamond", rank: 13, equip_skills: ["donghuangzhong"] }, { name: "swd_kongdongyin", kind: "kongdongyin", suit: "heart", rank: 13, equip_skills: ["kongdongyin"] }, { name: "swd_jingleishan", kind: "jingleishan", suit: "club", rank: 7 }, { name: "swd_chiyuxi", kind: "chiyuxi", suit: "diamond", rank: 7 }, { name: "swd_guisheqi", kind: "guisheqi", suit: "spade", rank: 9 }, { name: "swd_guisheqi2", kind: "guisheqi", suit: "club", rank: 7 }, { name: "swd_xianluhui", kind: "xianluhui", suit: "spade", rank: 13 }, { name: "swd_guilingzhitao", kind: "guilingzhitao", suit: "spade", rank: 7, equip_skills: ["nigong_accumulate", "nigong_strike"] }, { name: "swd_sifeizhenmian", kind: "sifeizhenmian", suit: "spade", rank: 4, equip_skills: ["yiluan"] }, { name: "swd_qinglongzhigui", kind: "qinglongzhigui", suit: "heart", rank: 6 }, { name: "swd_zhuquezhizhang", kind: "zhuquezhizhang", suit: "diamond", rank: 6 }, { name: "swd_baishouzhihu", kind: "baishouzhihu", suit: "spade", rank: 6 }, { name: "swd_xuanwuzhihuang", kind: "xuanwuzhihuang", suit: "club", rank: 6 }, { name: "swd_huanglinzhicong", kind: "huanglinzhicong", suit: "heart", rank: 5 }, { name: "swd_shuchui", kind: "shuchui", suit: "club", rank: 6, equip_skills: ["shuchui"] }, { name: "swd_sadengjinhuan", kind: "sadengjinhuan", suit: "club", rank: 13 }, { name: "swd_haotianta", kind: "haotianta", suit: "diamond", rank: 13 }, { name: "swd_yuruyi", kind: "yuruyi", suit: "heart", rank: 9 }, { name: "swd_yuchanqian", kind: "yuchanqian", suit: "spade", rank: 1, deferred_reason: "\u6362\u5F62\u6001\u968F\u673A mutation+\u7145\u9020\u88C5\u5907\u9762\u6302\u6279\u246C\uFF08autoViewAs \u6740\u672C\u6279\u5DF2\u8F6C\u6B63\uFF09" }, { name: "swd_yuchankun", kind: "yuchankun", suit: "club", rank: 2, deferred_reason: "\u6362\u5F62\u6001\u968F\u673A mutation+\u7145\u9020\u88C5\u5907\u9762\u6302\u6279\u246C\uFF08autoViewAs \u8349\u836F\u672C\u6279\u5DF2\u8F6C\u6B63\uFF09" }, { name: "swd_yuchanzhen", kind: "yuchanzhen", suit: "diamond", rank: 3, deferred_reason: "\u6362\u5F62\u6001\u968F\u673A mutation+\u7145\u9020\u88C5\u5907\u9762\u6302\u6279\u246C\uFF08autoViewAs \u9152\u672C\u6279\u5DF2\u8F6C\u6B63\uFF09" }, { name: "swd_yuchanxun", kind: "yuchanxun", suit: "heart", rank: 4, deferred_reason: "\u6362\u5F62\u6001\u968F\u673A mutation+\u7145\u9020\u88C5\u5907\u9762\u6302\u6279\u246C\uFF08autoViewAs \u6843\u672C\u6279\u5DF2\u8F6C\u6B63\uFF09" }, { name: "swd_yuchankan", kind: "yuchankan", suit: "spade", rank: 5, deferred_reason: "\u6362\u5F62\u6001\u968F\u673A mutation+\u7145\u9020\u88C5\u5907\u9762\u6302\u6279\u246C" }, { name: "swd_yuchanli", kind: "yuchanli", suit: "club", rank: 6, deferred_reason: "\u6362\u5F62\u6001\u968F\u673A mutation+\u7145\u9020\u88C5\u5907\u9762\u6302\u6279\u246C\uFF08autoViewAs \u5929\u4ED9\u9152\u672C\u6279\u5DF2\u8F6C\u6B63\uFF09" }, { name: "swd_yuchangen", kind: "yuchangen", suit: "diamond", rank: 7, deferred_reason: "\u6362\u5F62\u6001\u968F\u673A mutation+\u7145\u9020\u88C5\u5907\u9762\u6302\u6279\u246C\uFF08autoViewAs \u5C01\u5370\u4E4B\u4E39\u672C\u6279\u5DF2\u8F6C\u6B63\uFF09" }, { name: "swd_yuchandui", kind: "yuchandui", suit: "heart", rank: 8, deferred_reason: "\u6362\u5F62\u6001\u968F\u673A mutation+\u7145\u9020\u88C5\u5907\u9762\u6302\u6279\u246C\uFF08autoViewAs \u96EA\u9701\u51B0\u96F9\u672C\u6279\u5DF2\u8F6C\u6B63\uFF09" }, { name: "swd_shencaojie", kind: "shencaojie", suit: "club", rank: 4 }, { name: "swd_shencaojie2", kind: "shencaojie", suit: "diamond", rank: 4 }, { name: "swd_shencaojie3", kind: "shencaojie", suit: "spade", rank: 4 }, { name: "xuanyuanjian", kind: "xuanyuanjian", suit: "spade", rank: 1 }], C_ = [], R_ = [], z_ = JSON.parse('[{"name":"swd\u97E9\u7F57","faction":"qun","hp":5,"gender":"male","skills":[{"name":"tiebi","mapping":"host:tiebi"}]},{"name":"swd\u4E60\u5AE3","faction":"shu","hp":3,"gender":"female","skills":[{"name":"jiefen","mapping":"host:jiefen"},{"name":"datong","mapping":"host:datong"}]},{"name":"swd\u8F69\u8F95\u5251\u4F8D","faction":"qun","hp":3,"gender":"male","skills":[{"name":"chengjian","mapping":"host:chengjian"},{"name":"huanling","mapping":"host:huanling"}]},{"name":"swd\u7A0B\u54AC\u91D1","faction":"qun","hp":4,"gender":"male","skills":[{"name":"jiuchi","mapping":"builtin:\u9152\u6C60"},{"name":"jufu","mapping":"host:jufu"}]},{"name":"swd\u79E6\u53D4\u5B9D","faction":"qun","hp":3,"gender":"male","skills":[{"name":"huajing","mapping":"host:huajing"},{"name":"pingxu","mapping":"host:pingxu"}]},{"name":"swd\u5B50\u8FBE\u4E66","faction":"wei","hp":3,"gender":"male","skills":[{"name":"shoulie","mapping":"host:shoulie"},{"name":"hudun","mapping":"host:hudun"}]},{"name":"swd\u5F20\u9AD8","faction":"shu","hp":4,"gender":"male","skills":[{"name":"yicong","mapping":"builtin:\u4E49\u4ECE"},{"name":"poxing","mapping":"host:poxing"}]},{"name":"swd\u62D3\u8DCB\u7389\u513F","faction":"shu","hp":4,"gender":"female","skills":[{"name":"swdliuhong","mapping":"host:swdliuhong"},{"name":"poyue","mapping":"host:poyue"},{"name":"niepan","mapping":"host:niepan"}]},{"name":"swd\u6C34\u955C","faction":"qun","hp":4,"gender":"female","skills":[{"name":"mojian","mapping":"host:mojian"},{"name":"duanyue","mapping":"host:duanyue"},{"name":"tuzhen","mapping":"host:tuzhen"}]},{"name":"swd\u5C48\u5A34","faction":"qun","hp":3,"gender":"female","skills":[{"name":"mojian","mapping":"host:mojian"},{"name":"huanxia","mapping":"host:huanxia"}]},{"name":"swd\u58F6\u4E2D\u4ED9","faction":"wu","hp":3,"gender":"male","skills":[{"name":"daofa","mapping":"host:daofa"},{"name":"xielv","mapping":"host:xielv"},{"name":"hujing","mapping":"host:hujing"}]},{"name":"swd\u5170\u8335","faction":"wei","hp":3,"gender":"female","skills":[{"name":"xingdian","mapping":"host:xingdian"},{"name":"yulin","mapping":"host:yulin"},{"name":"luomei","mapping":"host:luomei"}]},{"name":"swd\u829D\u8335","faction":"wei","hp":3,"gender":"female","skills":[{"name":"xuehuang","mapping":"host:xuehuang"},{"name":"ningshuang","mapping":"host:ningshuang"},{"name":"zhuyu","mapping":"host:zhuyu"}]},{"name":"swd\u9EC4\u96F7","faction":"qun","hp":3,"gender":"male","skills":[{"name":"jilve","mapping":"host:jilve"},{"name":"gongshen","mapping":"host:gongshen"},{"name":"gaizao","mapping":"host:gaizao"}]},{"name":"swd_septem","faction":"qun","hp":4,"gender":"male","skills":[{"name":"jiying","mapping":"host:jiying"},{"name":"liaoyuan","mapping":"host:liaoyuan"},{"name":"yishan","mapping":"host:yishan"}]},{"name":"swd_huanyuanzhi","faction":"qun","hp":3,"gender":"male","skills":[{"name":"swdtianshu","mapping":"host:swdtianshu"},{"name":"lanzhi","mapping":"host:lanzhi"},{"name":"mufeng","mapping":"host:mufeng"}]},{"name":"swd_guyue","faction":"wei","hp":3,"gender":"male","skills":[{"name":"gtiandao","mapping":"host:g_tiandao"},{"name":"gxianyin","mapping":"host:gxianyin"},{"name":"wangchen","mapping":"host:wangchen"}]},{"name":"swd_shangzhang","faction":"shu","hp":4,"gender":"male","skills":[{"name":"lianwu","mapping":"host:lianwu"}]},{"name":"swd_youzhao","faction":"shu","hp":4,"gender":"male","skills":[{"name":"longdan","mapping":"builtin:\u9F99\u80C6"},{"name":"yuchen","mapping":"host:yuchen"}]},{"name":"swd_wangsiyue","faction":"wei","hp":3,"gender":"female","skills":[{"name":"duishi","mapping":"host:duishi"},{"name":"biyue","mapping":"builtin:\u95ED\u6708"}]},{"name":"swd_yuwentuo","faction":"shu","hp":4,"gender":"male","skills":[{"name":"wushuang","mapping":"builtin:\u65E0\u53CC"},{"name":"xielei","mapping":"host:xielei"},{"name":"kunlunjing","mapping":"host:kunlunjing"}]},{"name":"swd_fuyan","faction":"qun","hp":4,"gender":"male","skills":[{"name":"lianda","mapping":"host:lianda"}]},{"name":"swd_rongshuang","faction":"wu","hp":3,"gender":"female","skills":[{"name":"suiyan","mapping":"host:suiyan"},{"name":"duanxing","mapping":"host:duanxing"}]},{"name":"swd_muyun","faction":"wei","hp":4,"gender":"male","skills":[{"name":"zhuhai","mapping":"host:zhuhai"},{"name":"polang","mapping":"host:polang"},{"name":"jikong","mapping":"host:jikong"}]},{"name":"swd_chenjingchou","faction":"wu","hp":3,"gender":"male","skills":[{"name":"youyin","mapping":"host:youyin"},{"name":"yihua","mapping":"host:yihua"}]},{"name":"swd_tuwei","faction":"shu","hp":3,"gender":"female","skills":[{"name":"zhanlu","mapping":"host:zhanlu"},{"name":"susheng","mapping":"host:susheng"}]},{"name":"swd_xiarou","faction":"shu","hp":3,"gender":"male","skills":[{"name":"xianghui","mapping":"host:xianghui"},{"name":"huiqi","mapping":"host:huiqi"}]},{"name":"swd_zhuoshanzhu","faction":"wu","hp":4,"gender":"male","skills":[{"name":"suiyan","mapping":"host:suiyan"},{"name":"wanjun","mapping":"host:wanjun"}]},{"name":"swd_moye","faction":"wu","hp":3,"gender":"female","skills":[{"name":"rexue","mapping":"host:rexue"},{"name":"liuli","mapping":"builtin:\u6D41\u79BB"},{"name":"senluo","mapping":"host:senluo"}]},{"name":"swd_yeyaxi","faction":"shu","hp":3,"gender":"female","skills":[{"name":"rexue","mapping":"host:rexue"},{"name":"huopu","mapping":"host:huopu"},{"name":"shenyan","mapping":"host:shenyan"}]},{"name":"swd_xiaohuanglong","faction":"wei","hp":3,"gender":"male","skills":[{"name":"yeying","mapping":"host:yeying"},{"name":"juxi","mapping":"host:juxi"},{"name":"juxi_count","mapping":"host:juxi_count"}]},{"name":"swd_hupo","faction":"wu","hp":3,"gender":"male","skills":[{"name":"dunxing","mapping":"host:dunxing"},{"name":"guiying","mapping":"host:guiying"}]},{"name":"swd_situqiang","faction":"shu","hp":3,"gender":"female","skills":[{"name":"fengze","mapping":"host:fengze"},{"name":"lingyue","mapping":"host:lingyue"},{"name":"jinlin","mapping":"host:jinlin"}]},{"name":"swd_anka","faction":"qun","hp":3,"gender":"male","skills":[{"name":"songci","mapping":"host:songci"},{"name":"anlianying","mapping":"host:anlianying"}]},{"name":"swd_luchengxuan","faction":"wu","hp":4,"gender":"male","skills":[{"name":"ljifeng","mapping":"host:ljifeng"},{"name":"lxianglong","mapping":"host:lxianglong"}]},{"name":"swd_murongshi","faction":"shu","hp":3,"gender":"female","skills":[{"name":"duanyi","mapping":"host:duanyi"},{"name":"guxing","mapping":"host:guxing"},{"name":"guxing3","mapping":"host:guxing3"}]},{"name":"swd_yuxiaoxue","faction":"wei","hp":3,"gender":"female","skills":[{"name":"huanhun","mapping":"host:huanhun"},{"name":"daixing","mapping":"host:daixing"},{"name":"yinyue","mapping":"host:yinyue"},{"name":"daixing2","mapping":"host:daixing2"},{"name":"daixing3","mapping":"host:daixing3"}]},{"name":"swd_yuli","faction":"wu","hp":3,"gender":"female","skills":[{"name":"lingxin","mapping":"host:lingxin"},{"name":"tianxiang","mapping":"host:tianxiang"}]},{"name":"swd_kama","faction":"qun","hp":3,"gender":"female","skills":[{"name":"yueren","mapping":"host:yueren"},{"name":"shangshi","mapping":"builtin:\u4F24\u901D"}]},{"name":"swd_nicole","faction":"qun","hp":3,"gender":"female","skills":[{"name":"huanjian","mapping":"host:huanjian"},{"name":"lingwu","mapping":"host:lingwu"},{"name":"minjing","mapping":"host:minjing"}]},{"name":"swd_weida","faction":"qun","hp":3,"gender":"female","skills":[{"name":"yueren","mapping":"host:yueren"},{"name":"zhenlie","mapping":"host:zhenlie"},{"name":"duijue","mapping":null}]},{"name":"swd_chunyuheng","faction":"wei","hp":2,"gender":"male","skills":[{"name":"jueqing","mapping":"builtin:\u7EDD\u60C5"},{"name":"shengshou","mapping":"host:shengshou"},{"name":"xuying","mapping":"host:xuying"}]},{"name":"swd_ziqiao","faction":"shu","hp":3,"gender":"female","skills":[{"name":"guaili","mapping":"host:guaili"},{"name":"fuyan","mapping":"host:fuyan"}]},{"name":"swd_lilian","faction":"qun","hp":3,"gender":"female","skills":[{"name":"swd_wuxie","mapping":"host:wuxie"},{"name":"lqingcheng","mapping":"host:lqingcheng"}]},{"name":"swd_jiting","faction":"wei","hp":4,"gender":"female","skills":[{"name":"guanhu","mapping":"host:guanhu"},{"name":"lingshi","mapping":"host:lingshi"}]},{"name":"swd_hengai","faction":"shu","hp":3,"gender":"female","skills":[{"name":"hlingbo","mapping":"host:hlingbo"},{"name":"funiao","mapping":"host:funiao"},{"name":"ningxian","mapping":"host:ningxian"}]},{"name":"swd_miles","faction":"qun","hp":4,"gender":"male","skills":[{"name":"aojian","mapping":"host:aojian"},{"name":"miles_xueyi","mapping":"host:miles_xueyi"},{"name":"mohua2","mapping":"host:mohua2"}]},{"name":"swd_kangnalishi","faction":"qun","hp":1,"gender":"male","skills":[{"name":"busi","mapping":"host:busi"},{"name":"xuying","mapping":"host:xuying"},{"name":"yinguo","mapping":"host:yinguo"}]},{"name":"swd_zhaoyun","faction":"shu","hp":4,"gender":"male","skills":[{"name":"longdan","mapping":"builtin:\u9F99\u80C6"},{"name":"pozhen","mapping":"host:pozhen"},{"name":"tanlin","mapping":null}]},{"name":"swd_huyue","faction":"wu","hp":3,"gender":"female","skills":[{"name":"hyunshen","mapping":"host:hyunshen"},{"name":"fengming","mapping":"host:fengming"},{"name":"hyunshen2","mapping":"host:hyunshen2"}]},{"name":"swd_shanxiaoxiao","faction":"wu","hp":3,"gender":"female","skills":[{"name":"shehun","mapping":"host:shehun"},{"name":"xiaomoyu","mapping":"host:xiaomoyu"}]},{"name":"swd_duguningke","faction":"qun","hp":3,"gender":"female","skills":[{"name":"nlianji","mapping":"host:nlianji"},{"name":"touxi","mapping":"builtin:\u7A81\u88AD"}]},{"name":"swd_haidapang","faction":"wu","hp":3,"gender":"female","skills":[{"name":"bingjian","mapping":"host:bingjian"},{"name":"rumeng","mapping":"host:rumeng"}]},{"name":"swd_maixing","faction":"wu","hp":3,"gender":"male","skills":[{"name":"toudan","mapping":"host:toudan"},{"name":"shending","mapping":"host:shending"}]},{"name":"swd_fengtianling","faction":"shu","hp":4,"gender":"male","skills":[{"name":"guiyan","mapping":"host:guiyan"},{"name":"jiang","mapping":"host:jiang"}]},{"name":"swd_fengyu","faction":"shu","hp":4,"gender":"male","skills":[{"name":"fzhenwei","mapping":"host:fzhenwei"},{"name":"shangxi","mapping":"host:shangxi"}]},{"name":"swd_muyue","faction":"wei","hp":3,"gender":"female","skills":[{"name":"xingzhui","mapping":"host:xingzhui"},{"name":"lingxian","mapping":"host:lingxian"},{"name":"shouyin","mapping":"host:shouyin"}]},{"name":"swd_shaowei","faction":"shu","hp":3,"gender":"female","skills":[{"name":"jianji","mapping":"host:jianji"},{"name":"huangyu","mapping":"host:huangyu"}]},{"name":"swd_jialanduo","faction":"qun","hp":4,"gender":"male","skills":[{"name":"xianyin","mapping":"host:xianyin"},{"name":"mailun","mapping":"host:mailun"}]},{"name":"swd_qi","faction":"qun","hp":3,"gender":"male","skills":[{"name":"yaotong","mapping":"host:yaotong"},{"name":"heihuo","mapping":"host:heihuo"},{"name":"pojian","mapping":"host:pojian"}]},{"name":"swd_huiyan","faction":"qun","hp":4,"gender":"male","skills":[{"name":"hwendao","mapping":"host:hwendao"},{"name":"lingfeng","mapping":"host:lingfeng"},{"name":"hxunzhi","mapping":"host:hxunzhi"}]},{"name":"swd_qiner","faction":"wei","hp":3,"gender":"female","skills":[{"name":"huanyin","mapping":"host:huanyin"},{"name":"tianhuo","mapping":"host:tianhuo"},{"name":"xuanzhou","mapping":"host:xuanzhou"}]},{"name":"swd_jiliang","faction":"wu","hp":3,"gender":"male","skills":[{"name":"yunchou","mapping":"host:yunchou"},{"name":"gongxin","mapping":"host:gongxin"},{"name":"jqimou","mapping":"host:jqimou"}]},{"name":"swd_zhanglie","faction":"qun","hp":4,"gender":"male","skills":[{"name":"huajin","mapping":"host:huajin"},{"name":"poxiao","mapping":"host:poxiao"}]},{"name":"swd_xuanyuanjianxian","faction":"qun","hp":4,"gender":"male","skills":[{"name":"pozhou","mapping":"host:pozhou"},{"name":"huajian","mapping":"host:huajian"},{"name":"xuanyuan","mapping":"host:xuanyuan"},{"name":"pozhou2","mapping":"host:pozhou2"}]},{"name":"swd_sikongyu","faction":"qun","hp":4,"gender":"male","skills":[{"name":"sliufeng","mapping":"host:sliufeng"},{"name":"linyun","mapping":"host:linyun"},{"name":"hutian","mapping":"host:hutian"}]},{"name":"swd_jipeng","faction":"wu","hp":3,"gender":"male","skills":[{"name":"reyingzi","mapping":"host:reyingzi"},{"name":"guozao","mapping":"host:guozao"}]},{"name":"swd_duopeng","faction":"wu","hp":3,"gender":"male","skills":[{"name":"luanji","mapping":"host:luanji"},{"name":"reyingzi","mapping":"host:reyingzi"}]},{"name":"swd_fu","faction":"qun","hp":5,"gender":"male","skills":[{"name":"yudun","mapping":"host:yudun"}]},{"name":"swd_kendi","faction":"qun","hp":3,"gender":"male","skills":[{"name":"zhanxing","mapping":"host:zhanxing"},{"name":"kbolan","mapping":"host:kbolan"}]},{"name":"swd_jiuyou","faction":"wei","hp":3,"gender":"male","skills":[{"name":"lexue","mapping":"host:lexue"}]},{"name":"swd_linyue","faction":"wei","hp":3,"gender":"male","skills":[{"name":"zhenjiu","mapping":"host:zhenjiu"},{"name":"lmazui","mapping":"host:lmazui"}]},{"name":"swd_shuwaner","faction":"shu","hp":3,"gender":"female","skills":[{"name":"sxianjing","mapping":"host:sxianjing"},{"name":"huodan","mapping":"host:huodan"},{"name":"sxianjing_gain","mapping":"host:sxianjing_gain"},{"name":"sxianjing_damage","mapping":"host:sxianjing_damage"}]},{"name":"swd_jiangwu","faction":"shu","hp":4,"gender":"male","skills":[{"name":"yijue","mapping":"host:yijue"},{"name":"dangping","mapping":"host:dangping"}]},{"name":"swd_yuchiyanhong","faction":"shu","hp":3,"gender":"female","skills":[{"name":"huanxing","mapping":"host:huanxing"},{"name":"meihuo","mapping":"host:meihuo"}]},{"name":"swd_jiangziya","faction":"wu","hp":3,"gender":"male","skills":[{"name":"mingfu","mapping":"host:mingfu"},{"name":"tianlun","mapping":"host:tianlun"}]},{"name":"swd_cheyun","faction":"wu","hp":3,"gender":"female","skills":[{"name":"cyxianjiang","mapping":"host:cyxianjiang"},{"name":"cyqiaoxie","mapping":"host:cyqiaoxie"},{"name":"shengong","mapping":"host:shengong"}]},{"name":"swd_duanmeng","faction":"shu","hp":4,"gender":"female","skills":[{"name":"xuanying","mapping":"host:xuanying"},{"name":"lieren","mapping":"host:lieren"}]},{"name":"swd_chenfu","faction":"qun","hp":4,"gender":"male","skills":[{"name":"xuanzhou","mapping":"host:xuanzhou"},{"name":"bingfeng","mapping":"host:bingfeng"}]}]'), I_ = [{ name: "swd_zhenjiu2", mapping: "host:zhenjiu2" }, { name: "swd_lmazui2", mapping: "host:lmazui2" }, { name: "swd_huanxing_state", mapping: "host:huanxing2" }, { name: "swd_guiyan2", mapping: "host:guiyan2" }, { name: "swd_huajin2", mapping: "host:huajin2" }, { name: "swd_jinlin2", mapping: "host:jinlin2" }, { name: "tianxiang2", mapping: "host:tianxiang2" }, { name: "hxunzhi2", mapping: "host:hxunzhi2" }, { name: "bingfeng2", mapping: "host:bingfeng2" }, { name: "xuanyuanjian2", mapping: "host:xuanyuanjian2" }, { name: "xuanyuanjian3", mapping: "host:xuanyuanjian3" }, { name: "xuanyuan", mapping: "host:xuanyuan" }, { name: "jifeng", mapping: "host:jifeng" }, { name: "moyan", mapping: "host:moyan" }, { name: "miedao", mapping: "host:miedao" }, { name: "swd_xiuluo", mapping: "host:swd_xiuluo" }], P_ = { source: $_, upstream_path: j_, upstream_commit: S_, pack: E_, cards: A_, host_skills: C_, deferred: R_, generals: z_, skills: I_ }, T_ = { package_format_version: 1, namespace: "upstream-noname", package_id: "shahai-yunchou", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [87, 73, 16, 137, 17, 228, 185, 148, 29, 192, 248, 121, 46, 123, 51, 178, 99, 59, 227, 5, 238, 24, 234, 223, 153, 43, 225, 121, 118, 175, 237, 81], asset_hash: [34, 111, 194, 231, 244, 54, 173, 146, 254, 193, 36, 14, 84, 194, 87, 207, 61, 22, 100, 151, 193, 186, 122, 45, 222, 132, 90, 79, 201, 18, 92, 112], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, O_ = [], N_ = [], q_ = [], M_ = [], D_ = [], L_ = [], U_ = [], F_ = [], V_ = { "zh-CN": { fudichouxin: "\u91DC\u5E95\u62BD\u85AA", huoshan: "\u706B\u6740", shuigong: "\u6C34\u653B", yunchou: "\u8FD0\u7B79\u5E37\u5E44" } }, H_ = { schema_version: 1, package_id: "shahai-yunchou", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "shahai-yunchou:cards", source_hash: [242, 7, 161, 150, 106, 106, 85, 215, 72, 11, 50, 144, 208, 98, 108, 44, 119, 194, 137, 133, 206, 167, 18, 228, 203, 241, 192, 44, 92, 190, 91, 119], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, W_ = { "shahai-yunchou:cards": "assets/cards.json" }, B_ = "gen-1", To = { manifest: T_, modules: O_, skill_index: N_, character_index: q_, card_index: M_, trigger_index: D_, modifier_index: L_, active_index: U_, view_as_index: F_, translations: V_, asset_manifest: H_, asset_paths: W_, compiler_version: B_ }, K_ = "noname", G_ = "apps/core/extension/\u6740\u6D77\u62FE\u9057/card/yunchou/", J_ = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", Y_ = { name: "shahai-yunchou-upstream-noname", version: 2, description: "G1 \u6740\u6D77\u62FE\u9057\xB7\u8FD0\u7B79\u5E37\u5E44\u5207\u7247", card_prefix: "yc_" }, Q_ = [{ name: "huoshan", kind: "huosha", suit: "heart", rank: 6 }, { name: "guohe", kind: "guohe", suit: "diamond", rank: 3 }, { name: "guohe", kind: "guohe", suit: "club", rank: 8 }, { name: "guohe", kind: "guohe", suit: "spade", rank: 13 }, { name: "shuigong", kind: "shuigong", suit: "club", rank: 7 }, { name: "shuigong", kind: "shuigong", suit: "club", rank: 7 }, { name: "shuigong", kind: "shuigong", suit: "spade", rank: 9 }, { name: "fudichouxin", kind: "fudichouxin", suit: "diamond", rank: 4 }, { name: "fudichouxin", kind: "fudichouxin", suit: "club", rank: 6 }, { name: "fudichouxin", kind: "fudichouxin", suit: "spade", rank: 1 }, { name: "chenhuodajie", kind: "chenhuodajie", suit: "diamond", rank: 6 }, { name: "chenhuodajie", kind: "chenhuodajie", suit: "diamond", rank: 9 }, { name: "chenhuodajie", kind: "chenhuodajie", suit: "club", rank: 3 }, { name: "suolianjia", kind: "suolianjia", suit: "club", rank: 13, link_on_equip: true }, { name: "hongshui", kind: "hongshui", suit: "club", rank: 7 }, { name: "toulianghuanzhu", kind: "toulianghuanzhu", suit: "heart", rank: 9 }, { name: "toulianghuanzhu", kind: "toulianghuanzhu", suit: "club", rank: 10 }, { name: "geanguanhuo", kind: "geanguanhuo", suit: "heart", rank: 1 }, { name: "geanguanhuo", kind: "geanguanhuo", suit: "spade", rank: 6 }, { name: "caochuanjiejian", kind: "caochuanjiejian", suit: "spade", rank: 7 }, { name: "caochuanjiejian", kind: "caochuanjiejian", suit: "spade", rank: 7 }, { name: "shezhanqunru", kind: "shezhanqunru", suit: "heart", rank: 4 }, { name: "shezhanqunru", kind: "shezhanqunru", suit: "club", rank: 8 }], X_ = [], Z_ = { source: K_, upstream_path: G_, upstream_commit: J_, pack: Y_, cards: Q_, deferred: X_ }, e1 = "noname", n1 = "apps/core/extension/\u6740\u6D77\u62FE\u9057/character/yunchou/", t1 = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", s1 = { name: "shahai-yunchou-generals-b9", version: 1, description: "G1 \u8FD0\u7B79\u6B66\u5C06\u9762\u62799\uFF08\u4E8E\u5409\u6574\u5C06\uFF1A\u8749\u7F18 max_hp \u51CF\u9762\u8F6C\u6B63\uFF09", card_prefix: "ycn_" }, a1 = [{ id: "ycn_diy_caocao", name: "ycn\u66F9\u64CD", faction: "wei", hp: 4, gender: "male", skills: [{ name: "ycnxicai", title: "\u620F\u624D", mapping: "host:xicai" }, { name: "ycndiyjianxiong", title: "\u5978\u96C4", mapping: null }, { name: "hujia", title: "\u62A4\u9A7E", mapping: "builtin:\u62A4\u9A7E" }] }, { id: "ycn_diy_weiyan", name: "ycn\u9B4F\u5EF6", faction: "shu", hp: 4, gender: "male", skills: [{ name: "ycndiykuanggu", title: "\u72C2\u9AA8", mapping: "host:diykuanggu" }] }, { id: "ycn_diy_sp_zhouyu", name: "ycn\u5468\u745C", faction: "wu", hp: 3, gender: "male", skills: [{ name: "ycnxiongzi", title: "\u96C4\u59FF", mapping: "host:xiongzi" }, { name: "ycnyaliang", title: "\u96C5\u91CF", mapping: "host:yaliang" }] }, { id: "ycn_diy_luxun", name: "ycn\u9646\u900A", faction: "wu", hp: 3, gender: "male", skills: [{ name: "ycnzonghuo", title: "\u7EB5\u706B", mapping: "host:zonghuo" }, { name: "ycnshaoying", title: "\u70E7\u8425", mapping: "host:shaoying" }] }, { id: "ycn_diy_yuji", name: "ycn\u4E8E\u5409", faction: "qun", hp: 3, gender: "male", skills: [{ name: "ycndiyguhuo", title: "\u86CA\u60D1", mapping: "host:diyguhuo" }, { name: "ycndiychanyuan", title: "\u8749\u7F18", mapping: "host:diychanyuan" }] }, { id: "ycn_diy_zhouyu", name: "ycn\u5468\u745C", faction: "wu", hp: 3, gender: "male", skills: [{ name: "ycnjieyan", title: "\u9699\u7130", mapping: "host:jieyan" }, { name: "ycnhonglian", title: "\u7EA2\u83B2", mapping: "host:honglian" }] }, { id: "ycn_diy_zhenji", name: "ycn\u7504\u59EC", faction: "wei", hp: 3, gender: "female", skills: [{ name: "ycnjiaoxia", title: "\u72E1\u9EE0", mapping: "host:jiaoxia" }, { name: "ycnyiesheng", title: "\u91CE\u5347", mapping: "host:yiesheng" }] }, { id: "ycn_diy_caiwenji", name: "ycn\u8521\u6587\u59EC", faction: "qun", hp: 3, gender: "female", skills: [{ name: "beige", title: "\u60B2\u6B4C", mapping: "host:beige" }, { name: "ycnguihan", title: "\u5F52\u6C49", mapping: "host:guihan" }] }, { id: "ycn_diy_lukang", name: "ycn\u9646\u6297", faction: "wu", hp: 4, gender: "male", skills: [{ name: "ycnluweiyan", title: "\u5389\u7130", mapping: "host:luweiyan" }, { name: "ycnluweiyan2", title: "\u5389\u7130\xB7\u94FE\u6740", mapping: "host:luweiyan2" }, { name: "qianxun", title: "\u8C26\u900A", mapping: "builtin:\u8C26\u900A" }] }, { id: "ycn_old_majun", name: "ycn\u9A6C\u94A7", faction: "wei", hp: 3, gender: "male", skills: [{ name: "xinfu_jingxie", title: "\u7CBE\u68B0", mapping: null }, { name: "ycnxinfu_qiaosi", title: "\u5DE7\u601D", mapping: null }] }], i1 = { source: e1, upstream_path: n1, upstream_commit: t1, pack: s1, generals: a1 }, o1 = { package_format_version: 1, namespace: "upstream-noname", package_id: "shahai-zhenfa", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [83, 148, 91, 71, 176, 142, 85, 235, 241, 190, 176, 53, 230, 223, 245, 74, 197, 187, 203, 191, 104, 12, 134, 170, 95, 74, 42, 149, 171, 33, 19, 178], asset_hash: [2, 250, 107, 122, 186, 246, 73, 201, 2, 120, 23, 125, 30, 171, 190, 145, 67, 30, 150, 250, 27, 69, 102, 73, 94, 115, 5, 218, 205, 27, 76, 4], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, r1 = [], l1 = [], u1 = [], c1 = [], d1 = [], p1 = [], h1 = [], f1 = [], m1 = { schema_version: 1, package_id: "shahai-zhenfa", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "shahai-zhenfa:cards", source_hash: [212, 242, 140, 245, 41, 62, 165, 171, 122, 180, 21, 0, 119, 163, 234, 243, 149, 248, 103, 207, 9, 191, 218, 139, 73, 11, 55, 57, 198, 30, 168, 136], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, g1 = { "shahai-zhenfa:cards": "assets/cards.json" }, _1 = "test", Oo = { manifest: o1, modules: r1, skill_index: l1, character_index: u1, card_index: c1, trigger_index: d1, modifier_index: p1, active_index: h1, view_as_index: f1, asset_manifest: m1, asset_paths: g1, compiler_version: _1 }, v1 = "qita/sanguosha/noname/extension/\u6740\u6D77\u62FE\u9057/card/zhenfa", y1 = "extension/\u6740\u6D77\u62FE\u9057/card/zhenfa/", k1 = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", w1 = { name: "shahai-zhenfa-upstream-noname", version: 1, description: "G1 \u8F6C\u6362\uFF1A\u4E0A\u6E38 zhenfa \u9996\u5207\u7247\uFF08\u5730\u8F7D\u9635\uFF09", card_prefix: "zf_" }, b1 = [{ name: "zf_dizaizhen", kind: "dizaizhen", suit: "diamond", rank: 3 }, { name: "zf_dizaizhen", kind: "dizaizhen", suit: "spade", rank: 3 }, { name: "zf_dizaizhen", kind: "dizaizhen", suit: "heart", rank: 3 }, { name: "swd_tianfuzhen1", kind: "tianfuzhen", suit: "spade", rank: 1 }, { name: "swd_qixingzhen1", kind: "qixingzhen", suit: "club", rank: 9 }, { name: "swd_niaoxiangzhen1", kind: "niaoxiangzhen", suit: "heart", rank: 3 }, { name: "swd_changshezhen1", kind: "changshezhen", suit: "diamond", rank: 1 }, { name: "swd_changshezhen2", kind: "changshezhen", suit: "club", rank: 1 }], x1 = [], $1 = [{ name: "pozhenjue", reason: "\u4F9D\u8D56\u56FD\u6218\u961F\u5217/\u56F4\u653B/\u5EA7\u4F4D\u91CD\u6392\u4F53\u7CFB\uFF08inline/isFriendOf/swapSeat\uFF09\uFF0C\u539F\u751F\u56FD\u6218\u539F\u578B\u672A\u5EFA\u6A21\u2014\u2014\u7ED3\u6784\u6027\u4FDD\u7559\u9879\u540C\u65CF" }], j1 = { source: v1, upstream_path: y1, upstream_commit: k1, pack: w1, cards: b1, host_skills: x1, deferred: $1 }, S1 = { package_format_version: 1, namespace: "upstream-noname", package_id: "wudian", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [224, 212, 136, 182, 142, 127, 174, 224, 64, 168, 249, 131, 136, 232, 56, 12, 82, 126, 204, 103, 169, 162, 33, 160, 174, 233, 63, 60, 228, 9, 190, 247], asset_hash: [101, 186, 252, 146, 248, 33, 244, 97, 247, 227, 110, 37, 19, 234, 69, 143, 65, 21, 226, 79, 251, 191, 128, 16, 8, 88, 15, 225, 249, 137, 146, 161], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, E1 = [], A1 = [], C1 = [], R1 = [], z1 = [], I1 = [], P1 = [], T1 = [], O1 = { "zh-CN": { diaobingqianjiang: "\u8C03\u5175\u9063\u5C06", sanjian: "\u4E09\u5C16", wangmeizhike: "\u671B\u6885\u6B62\u6E34", youdishenru: "\u8BF1\u654C\u6DF1\u5165" } }, N1 = { schema_version: 1, package_id: "wudian", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "wudian:cards", source_hash: [243, 4, 96, 226, 174, 224, 22, 240, 77, 131, 190, 254, 60, 103, 228, 236, 232, 29, 21, 165, 250, 11, 71, 138, 97, 179, 174, 47, 29, 135, 117, 67], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, q1 = { "wudian:cards": "assets/cards.json" }, M1 = "gen-1", No = { manifest: S1, modules: E1, skill_index: A1, character_index: C1, card_index: R1, trigger_index: z1, modifier_index: I1, active_index: P1, view_as_index: T1, translations: O1, asset_manifest: N1, asset_paths: q1, compiler_version: M1 }, D1 = "noname", L1 = "apps/core/extension/\u73A9\u70B9\u8BBA\u6740/", U1 = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", F1 = { name: "wudian-upstream-noname", version: 2, description: "G1 \u73A9\u70B9\u8BBA\u6740", card_prefix: "wd_" }, V1 = [{ name: "sha", kind: "sha", suit: "diamond", rank: 9 }, { name: "sha", kind: "sha", suit: "spade", rank: 6 }, { name: "shan", kind: "shan", suit: "heart", rank: 6 }, { name: "shan", kind: "shan", suit: "diamond", rank: 3 }, { name: "shan", kind: "shan", suit: "diamond", rank: 11 }, { name: "tao", kind: "tao", suit: "heart", rank: 5 }, { name: "tao", kind: "tao", suit: "heart", rank: 7 }, { name: "wangmeizhike", kind: "wangmeizhike", suit: "club", rank: 8 }, { name: "wangmeizhike", kind: "wangmeizhike", suit: "club", rank: 11 }, { name: "youdishenru", kind: "youdishenru", suit: "club", rank: 9 }, { name: "youdishenru", kind: "youdishenru", suit: "spade", rank: 11 }, { name: "youdishenru", kind: "youdishenru", suit: "spade", rank: 12 }, { name: "sanjian", kind: "sanjian", suit: "club", rank: 5, range: 3, equip_skills: ["wd_sanjian"] }, { name: "tiesuo", kind: "tiesuo", suit: "spade", rank: 4 }], H1 = [{ name: "diaobingqianjiang", count: 2, reason: "\u591A\u4EBA\u65B9\u5411\u9009\u62E9\u591A\u6B65\u7ED3\u7B97" }], W1 = [{ name: "wd_wangmeizhike", card: "wangmeizhike", mapping: "host:wangmeizhike_card" }, { name: "wd_sanjian", card: "sanjian", mapping: "host:sanjian_weapon" }, { name: "moshou", general: "wdb_feishi", mapping: "host:moshou_immunity" }, { name: "ciqiu", general: "wdb_hanlong", mapping: "host:wudian_ciqiu" }, { name: "nsshuaiyan", general: "wdb_feishi", mapping: "host:wudian_shuaiyan" }, { name: "zhucheng", general: "wdb_liufu", mapping: "host:wudian_zhucheng" }, { name: "zhucheng2", general: "wdb_liufu", mapping: "host:wudian_zhucheng_guard" }, { name: "kangyin", general: "wdb_liuzan", mapping: "host:wudian_kangyin" }, { name: "chezhen", general: "wdb_tianyu", mapping: "host:wudian_chezhen" }, { name: "juedao", general: "wdb_liuyan", mapping: "host:wudian_juedao" }, { name: "geju", general: "wdb_liuyan", mapping: "host:wudian_geju" }, { name: "duoqi", general: "wdb_liufu", mapping: null, reason: "phaseUse \u4E2D\u6B62\u672A\u5EFA\u6A21+\u4F9D\u8D56\u7B51\u57CE\u6807\u8BB0\u2014\u20142026-10-08 \u6B63\u5F0F\u6682\u7F13" }, { name: "youzhan", general: "wdb_tianyu", mapping: null, reason: "shaBefore \u65F6\u70B9\u6620\u5C04\u504F\u665A+\u865A\u62DF useCard \u7EC4\u5408\u2014\u20142026-10-08 \u6B63\u5F0F\u6682\u7F13" }], B1 = [{ name: "\u8D39\u8BD7", faction: "shu", hp: 3, gender: "male", skills: [{ name: "nsshuaiyan", mapping: "host:wudian_shuaiyan" }, { name: "moshou", mapping: "host:moshou_immunity" }] }, { name: "\u97E9\u9F99", faction: "wei", hp: 4, gender: "male", skills: [{ name: "siji", mapping: "host:siji_discard_draw" }, { name: "ciqiu", mapping: "host:wudian_ciqiu" }] }, { name: "\u5218\u99A5", faction: "wei", hp: 3, gender: "male", skills: [{ name: "zhucheng", mapping: "host:wudian_zhucheng" }, { name: "zhucheng2", mapping: "host:wudian_zhucheng_guard" }, { name: "duoqi", mapping: null, reason: "phaseUse \u4E2D\u6B62\uFF08evt.skipped\uFF09\u672A\u5EFA\u6A21+\u4F9D\u8D56\u7B51\u57CE\u6807\u8BB0\u2014\u20142026-10-08 \u6B63\u5F0F\u6682\u7F13" }] }, { name: "\u5218\u7109", faction: "qun", hp: 3, gender: "male", skills: [{ name: "juedao", mapping: "host:wudian_juedao" }, { name: "geju", mapping: "host:wudian_geju" }] }, { name: "\u7559\u8D5E", faction: "wu", hp: 4, gender: "male", skills: [{ name: "kangyin", mapping: "host:wudian_kangyin" }] }, { name: "\u7530\u8C6B", faction: "wei", hp: 4, gender: "male", skills: [{ name: "chezhen", mapping: "host:wudian_chezhen" }, { name: "youzhan", mapping: null, reason: "shaBefore \u65F6\u70B9\uFF08\u539F\u751F\u6620\u5C04 AfterTriggers \u504F\u665A\uFF09+\u4EE4\u76EE\u6807\u865A\u62DF useCard \u8BF1\u654C\u6DF1\u5165\u7EC4\u5408\u2014\u20142026-10-08 \u6B63\u5F0F\u6682\u7F13" }] }, { name: "\u897F\u771F\u5915\u7EA2", faction: "shu", hp: 4, gender: "male", skills: [{ name: "jinyan", mapping: "host:wudian_jinyan" }, { name: "fuchou", mapping: "host:fuchou" }, { name: "fuchou2", mapping: "host:fuchou2" }, { name: "jianbi", mapping: "host:wudian_jianbi" }] }, { name: "\u9020\u4E4B\u4EC1\u541B", faction: "wei", hp: 3, gender: "male", skills: [{ name: "liangce", mapping: "host:liangce_remain" }, { name: "jianbi", mapping: null }, { name: "diyjuntun", mapping: "host:wudian_diyjuntun" }] }, { name: "\u6768\u4EEA", faction: "shu", hp: 3, gender: "male", skills: [{ name: "choudu", mapping: null }, { name: "liduan", mapping: "host:wudian_liduan" }] }], K1 = { source: D1, upstream_path: L1, upstream_commit: U1, pack: F1, cards: V1, deferred: H1, skills: W1, generals: B1 }, G1 = { package_format_version: 1, namespace: "upstream-liuli", package_id: "wuxing", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [132, 94, 166, 250, 221, 202, 90, 254, 41, 167, 212, 147, 169, 28, 169, 39, 198, 78, 32, 87, 170, 107, 26, 155, 120, 216, 242, 221, 92, 75, 95, 25], asset_hash: [60, 107, 102, 15, 60, 169, 0, 83, 63, 35, 181, 217, 139, 24, 141, 42, 38, 138, 111, 102, 49, 137, 183, 48, 164, 187, 206, 145, 39, 244, 93, 134], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, J1 = [], Y1 = [], Q1 = [], X1 = [], Z1 = [], ev = [], nv = [], tv = [], sv = { "zh-CN": { fire: "\u706B", metal: "\u91D1", soil: "\u571F", water: "\u6C34", wood: "\u6728", wuxingpan: "\u4E94\u884C\u76D8", wuxingpan_skill: "\u4E94\u884C" } }, av = { schema_version: 1, package_id: "wuxing", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "wuxing:config", source_hash: [209, 188, 252, 3, 138, 195, 23, 199, 174, 66, 240, 192, 138, 68, 185, 89, 196, 5, 25, 128, 172, 49, 209, 89, 211, 121, 60, 76, 41, 255, 75, 222], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }, { asset_id: "wuxing:cards", source_hash: [239, 161, 205, 89, 208, 240, 33, 205, 87, 200, 152, 104, 171, 72, 97, 172, 223, 238, 146, 97, 40, 117, 119, 75, 157, 86, 203, 219, 159, 135, 28, 177], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, iv = { "wuxing:cards": "assets/cards.json", "wuxing:config": "assets/wuxing.json" }, ov = "gen-1", qo = { manifest: G1, modules: J1, skill_index: Y1, character_index: Q1, card_index: X1, trigger_index: Z1, modifier_index: ev, active_index: nv, view_as_index: tv, translations: sv, asset_manifest: av, asset_paths: iv, compiler_version: ov }, rv = "noname-liuli", lv = "noname/extension/wuxing/extension.js", uv = "8485dbb29ff16f9b258a766b1a17ef75f883aade", cv = { name: "wuxing-upstream-liuli", version: 1, description: "G1 wuxing\uFF1A\u4E94\u884C\u5C5E\u6027\u4E0E\u751F\u514B", card_prefix: "wxl_", init_hook: "wuxing_natures" }, dv = [{ name: "wuxingpan", kind: "wuxingpan", suit: "spade", rank: 5, equip_skills: ["wuxingpan_skill"] }], pv = [{ name: "wuxing_shengke", mapping: "host:shengke_check" }, { name: "wuxingpan_skill", mapping: "host:change_card_nature" }], hv = { source: rv, upstream_path: lv, upstream_commit: uv, pack: cv, cards: dv, skills: pv }, fv = { "cardpile-upstream-liuli": { package: xo, payload: { manifest: xo.manifest, pile: Fh } }, "cardpile-upstream-noname": { package: $o, payload: { manifest: $o.manifest, pile: lf } }, "d3-upstream-noname": { package: jo, payload: { manifest: jo.manifest, characters: Ef } }, "huanle-upstream-noname": { package: So, payload: { manifest: So.manifest, cards: Kf } }, "shaai-ow-upstream-noname": { package: Eo, payload: { manifest: Eo.manifest, cards: gm } }, "shaai-xianjian-upstream-noname": { package: Ao, payload: { manifest: Ao.manifest, cards: Dm } }, "shahai-gujian-upstream-noname": { package: Co, payload: { manifest: Co.manifest, cards: ig } }, "shahai-gwent-upstream-noname": { package: Ro, payload: { manifest: Ro.manifest, cards: jg } }, "shahai-hearth-upstream-noname": { package: zo, payload: { manifest: zo.manifest, cards: Wg } }, "shahai-mtg-upstream-noname": { package: Io, payload: { manifest: Io.manifest, cards: d_ } }, "shahai-swd-upstream-noname": { package: Po, payload: { manifest: Po.manifest, cards: P_ } }, "shahai-yunchou-upstream-noname": { package: To, payload: { manifest: To.manifest, cards: Z_, characters: i1 } }, "shahai-zhenfa-upstream-noname": { package: Oo, payload: { manifest: Oo.manifest, cards: j1 } }, "wudian-upstream-noname": { package: No, payload: { manifest: No.manifest, cards: K1 } }, "wuxing-upstream-liuli": { package: qo, payload: { manifest: qo.manifest, cards: hv } } };
function zs(e) {
  return typeof e == "string" && e.trim().length > 0 && e !== "." && e !== ".." && !e.includes("/") && !e.includes("\\");
}
function Da(e) {
  return Array.isArray(e) && e.length === 32 && e.every((n) => Number.isInteger(n) && n >= 0 && n <= 255);
}
function rs(e) {
  const n = e, t = n == null ? void 0 : n.manifest;
  if (!t || ![t.namespace, t.package_id, t.version].every(zs) || !Da(t.logic_hash)) throw new Error("\u6269\u5C55\u5305\u8EAB\u4EFD\u6216\u903B\u8F91\u6307\u7EB9\u65E0\u6548");
  if (t.dependencies !== void 0) {
    if (!Array.isArray(t.dependencies)) throw new Error("\u6269\u5C55\u5305\u4F9D\u8D56\u683C\u5F0F\u65E0\u6548");
    const a = /* @__PURE__ */ new Set();
    for (const i of t.dependencies) {
      if (!i || typeof i != "object" || Array.isArray(i) || ![i.namespace, i.package_id, i.version].every(zs) || i.logic_hash !== void 0 && !Da(i.logic_hash)) throw new Error("\u6269\u5C55\u5305\u4F9D\u8D56\u683C\u5F0F\u65E0\u6548");
      const o = `${i.namespace}/${i.package_id}`;
      if (a.has(o) || o === `${t.namespace}/${t.package_id}`) throw new Error(`\u6269\u5C55\u5305\u4F9D\u8D56\u91CD\u590D\u6216\u6307\u5411\u81EA\u8EAB: ${o}`);
      a.add(o);
    }
  }
  if ((n == null ? void 0 : n.modules) !== void 0 && !Array.isArray(n.modules)) throw new Error("\u6269\u5C55\u5305 VM \u6A21\u5757\u683C\u5F0F\u65E0\u6548");
  (n == null ? void 0 : n.modules) !== void 0 && mv(n);
  const s = [n == null ? void 0 : n.cards, n == null ? void 0 : n.characters, n == null ? void 0 : n.pile].filter((a) => a !== void 0);
  if (!s.length && (n == null ? void 0 : n.modules) === void 0 || s.some((a) => a === null || typeof a != "object" || Array.isArray(a))) throw new Error("\u6269\u5C55\u5305\u7F3A\u5C11\u53EF\u88C5\u8F7D\u7684\u5361\u724C\u3001\u6B66\u5C06\u3001\u724C\u5806\u6216 VM \u6570\u636E");
  return n;
}
function ps(e) {
  return Number.isInteger(e) && e >= 0;
}
function mv(e) {
  const n = e.modules, t = /* @__PURE__ */ new Map();
  for (const o of n) {
    const r = o;
    if (!r || typeof r != "object" || Array.isArray(r)) throw new Error("\u6269\u5C55\u5305 VM \u6A21\u5757\u683C\u5F0F\u65E0\u6548");
    if (!zs(r.name) || t.has(r.name)) throw new Error("\u6269\u5C55\u5305 VM \u6A21\u5757\u540D\u7F3A\u5931\u6216\u91CD\u590D");
    const l = r.code;
    if (!Array.isArray(l) || !l.length || !l.every((d) => Number.isInteger(d) && d >= 0 && d <= 255)) throw new Error(`\u6269\u5C55\u5305 VM \u6A21\u5757\u5B57\u8282\u7801\u65E0\u6548: ${r.name}`);
    if (![r.local_count, r.max_stack, r.entry_pc, r.caps_mask].every(ps) || r.entry_pc >= l.length) throw new Error(`\u6269\u5C55\u5305 VM \u6A21\u5757\u5165\u53E3\u6216\u9884\u7B97\u5B57\u6BB5\u65E0\u6548: ${r.name}`);
    if (!Da(r.logic_hash)) throw new Error(`\u6269\u5C55\u5305 VM \u6A21\u5757\u6307\u7EB9\u683C\u5F0F\u65E0\u6548: ${r.name}`);
    t.set(r.name, l.length);
  }
  const s = /* @__PURE__ */ new Set(), a = e.skill_index;
  if (a !== void 0) {
    if (!Array.isArray(a)) throw new Error("\u6269\u5C55\u5305\u6280\u80FD\u7D22\u5F15\u683C\u5F0F\u65E0\u6548");
    for (const o of a) {
      const r = o;
      if (!r || typeof r != "object" || !ps(r.skill_def_id) || !zs(r.name) || s.has(r.skill_def_id)) throw new Error("\u6269\u5C55\u5305\u6280\u80FD\u7D22\u5F15\u6761\u76EE\u65E0\u6548\u6216\u91CD\u590D");
      s.add(r.skill_def_id);
    }
  }
  const i = e.trigger_index;
  if (i !== void 0) {
    if (!Array.isArray(i)) throw new Error("\u6269\u5C55\u5305\u89E6\u53D1\u7D22\u5F15\u683C\u5F0F\u65E0\u6548");
    for (const o of i) {
      const r = o;
      if (!r || typeof r != "object" || typeof r.module != "string" || !t.has(r.module)) throw new Error("\u6269\u5C55\u5305\u89E6\u53D1\u7D22\u5F15\u5F15\u7528\u4E86\u4E0D\u5B58\u5728\u7684 VM \u6A21\u5757");
      if (!ps(r.entry_pc) || r.entry_pc >= t.get(r.module) || !ps(r.skill_def_id) || a !== void 0 && !s.has(r.skill_def_id)) throw new Error(`\u6269\u5C55\u5305\u89E6\u53D1\u7D22\u5F15\u5165\u53E3\u6216\u6280\u80FD\u5F15\u7528\u65E0\u6548: ${r.module}`);
    }
  }
}
function tt(e) {
  return `${e.manifest.namespace}/${e.manifest.package_id}`;
}
function ls(e) {
  return e.manifest.logic_hash.map((n) => n.toString(16).padStart(2, "0")).join("");
}
function yl(e) {
  const n = fv[e.dir];
  if (!n) throw new Error(`\u65E0\u53EF\u5B89\u88C5\u5305\u6570\u636E: ${e.dir}`);
  if (!Array.isArray(n.package.modules)) throw new Error(`\u6269\u5C55\u5305 VM \u6A21\u5757\u683C\u5F0F\u65E0\u6548: ${e.dir}`);
  const t = n.package.modules.length ? { ...n.package, ...n.payload } : n.payload, s = rs(t);
  if (tt(s) !== `${e.namespace}/${e.packageId}` || s.manifest.version !== e.version || !ls(s).startsWith(e.logicFingerprint)) throw new Error(`\u5305\u76EE\u5F55\u4E0E\u5B9E\u9645\u5B89\u88C5\u5185\u5BB9\u4E0D\u4E00\u81F4: ${e.dir}`);
  return JSON.stringify(s);
}
const kl = "noname-next.extension-store.v1", wl = "noname-next.native-saves.v1";
function hs() {
  return { installed: [], contentLock: [], matchFrozen: false };
}
function Je() {
  if (typeof localStorage > "u") return hs();
  try {
    const e = localStorage.getItem(kl);
    return e ? { ...hs(), ...JSON.parse(e) } : hs();
  } catch {
    return hs();
  }
}
function qn(e) {
  typeof localStorage > "u" || localStorage.setItem(kl, JSON.stringify(e));
}
function st() {
  return (/* @__PURE__ */ new Date()).toISOString();
}
function bl(e) {
  return Kt.find((n) => ri(n) === e);
}
function li(e, n, t) {
  const s = e == null ? void 0 : e[n];
  if (s === void 0) return;
  const a = (i) => JSON.stringify(JSON.parse(i), (o, r) => r === null || typeof r != "object" || Array.isArray(r) ? r : Object.fromEntries(Object.entries(r).sort(([l], [d]) => l < d ? -1 : l > d ? 1 : 0)));
  if (a(s) !== a(t)) throw new Error(`\u540C\u7248\u672C\u5305\u5185\u5BB9\u51B2\u7A81: ${n}`);
}
function Is() {
  return Je();
}
function Mo(e) {
  const n = Je();
  return n.matchFrozen = e, qn(n), n;
}
function xl(e) {
  var _a2;
  const n = bl(e);
  if (!n) throw new Error(`\u672A\u77E5\u6269\u5C55: ${e}`);
  const t = Je();
  if (t.matchFrozen) throw new Error("\u5BF9\u5C40\u4E2D\u4E0D\u53EF\u5B89\u88C5\u6216\u66FF\u6362\u5305\u5185\u5BB9");
  if ((_a2 = t.revokedKeys) == null ? void 0 : _a2.includes(e)) throw new Error(`\u5DF2\u64A4\u9500\u5305\u4E0D\u53EF\u91CD\u88C5: ${e}`);
  const s = yl(n), a = t.installed.find((o) => o.key === e), i = st();
  if (a) {
    if (a.lifecycle === "revoked") throw new Error(`\u5DF2\u64A4\u9500\u5305\u4E0D\u53EF\u91CD\u88C5: ${e}\uFF08\u9700\u6E05\u9664\u64A4\u9500\u8BB0\u5F55\uFF09`);
    li(a.nativeVersions, n.version, s), a.updatedAt = i, a.nativeVersions ?? (a.nativeVersions = {}), a.version === n.version && !a.nativeVersions[n.version] && (a.nativeVersions[n.version] = s), a.lifecycle === "available" && (a.lifecycle = "installed");
  } else t.installed.push({ key: e, dir: n.dir, packageId: n.packageId, namespace: n.namespace, version: n.version, history: [n.version], lifecycle: "installed", installedAt: i, updatedAt: i, logicFingerprint: n.logicFingerprint, nativeVersions: { [n.version]: s } });
  return qn(t), t;
}
async function gv(e, n) {
  const t = _v(e);
  return await n(t.canonical), vv(t);
}
function _v(e) {
  const n = Je();
  if (n.matchFrozen) throw new Error("\u5BF9\u5C40\u4E2D\u4E0D\u53EF\u5B89\u88C5\u6216\u66FF\u6362\u5305\u5185\u5BB9");
  let t;
  try {
    t = JSON.parse(e);
  } catch {
    throw new Error("\u6269\u5C55\u5305\u4E0D\u662F\u5408\u6CD5 JSON");
  }
  const s = rs(t), a = tt(s), i = s.manifest.version, o = JSON.stringify(s);
  return $l(n, a, i, o), { payload: s, key: a, version: i, canonical: o, fingerprint: ls(s).slice(0, 8) };
}
function $l(e, n, t, s) {
  var _a2;
  if (e.matchFrozen) throw new Error("\u5BF9\u5C40\u4E2D\u4E0D\u53EF\u5B89\u88C5\u6216\u66FF\u6362\u5305\u5185\u5BB9");
  if ((_a2 = e.revokedKeys) == null ? void 0 : _a2.includes(n)) throw new Error(`\u5DF2\u64A4\u9500\u5305\u4E0D\u53EF\u91CD\u88C5: ${n}`);
  const a = e.installed.find((i) => i.key === n);
  if ((a == null ? void 0 : a.lifecycle) === "revoked") throw new Error(`\u5DF2\u64A4\u9500\u5305\u4E0D\u53EF\u91CD\u88C5: ${n}\uFF08\u9700\u6E05\u9664\u64A4\u9500\u8BB0\u5F55\uFF09`);
  a && li(a.nativeVersions, t, s);
}
function vv(e) {
  var _a2;
  const n = Je(), { payload: t, key: s, version: a, canonical: i, fingerprint: o } = e;
  $l(n, s, a, i);
  const r = n.installed.find((d) => d.key === s), l = st();
  return r ? (r.nativeVersions ?? (r.nativeVersions = {}), (_a2 = r.nativeVersions)[a] ?? (_a2[a] = i), r.version !== a && (r.history.includes(r.version) || r.history.push(r.version), r.version = a, r.history.includes(a) || r.history.push(a)), r.logicFingerprint = o, r.updatedAt = l, r.signature = "unsigned", r.lifecycle === "available" && (r.lifecycle = "installed")) : n.installed.push({ key: s, dir: "imported", packageId: t.manifest.package_id, namespace: t.manifest.namespace, version: a, history: [a], lifecycle: "installed", installedAt: l, updatedAt: l, logicFingerprint: o, nativeVersions: { [a]: i }, signature: "unsigned" }), qn(n), n;
}
function jl(e) {
  const n = Je();
  if (n.matchFrozen) throw new Error("\u5BF9\u5C40\u4E2D\u4E0D\u53EF\u53D8\u66F4\u5185\u5BB9\u9501");
  const t = n.installed.find((s) => s.key === e);
  if (!t || t.lifecycle === "revoked") throw new Error(`\u672A\u5B89\u88C5\u6216\u5DF2\u64A4\u9500: ${e}`);
  return t.lifecycle = "active", t.updatedAt = st(), n.contentLock.includes(e) || n.contentLock.push(e), n.contentLock.sort(), qn(n), n;
}
function yv(e) {
  const n = Je();
  if (n.matchFrozen) throw new Error("\u5BF9\u5C40\u4E2D\u4E0D\u53EF\u53D8\u66F4\u5185\u5BB9\u9501");
  const t = n.installed.find((s) => s.key === e);
  if (!t) throw new Error(`\u672A\u5B89\u88C5: ${e}`);
  if (t.lifecycle === "revoked") throw new Error(`\u5DF2\u64A4\u9500: ${e}`);
  return t.lifecycle = "disabled", t.updatedAt = st(), n.contentLock = n.contentLock.filter((s) => s !== e), qn(n), n;
}
function kv(e, n) {
  var _a2;
  const t = bl(e), s = Je();
  if (!t) throw s.installed.some((r) => r.key === e) ? new Error(`\u5BFC\u5165\u5305\u6CA1\u6709\u5185\u7F6E\u76EE\u5F55\u7248\u672C\uFF0C\u8BF7\u5BFC\u5165\u65B0\u7248\u672C\u5305\u4EE5\u5347\u7EA7: ${e}`) : new Error(`\u672A\u77E5\u6269\u5C55: ${e}`);
  if (s.matchFrozen) throw new Error("\u5BF9\u5C40\u4E2D\u4E0D\u53EF\u5347\u7EA7\uFF08\u907F\u514D\u5185\u5BB9\u9501\u6F02\u79FB\uFF09");
  const a = s.installed.find((r) => r.key === e);
  if (!a || a.lifecycle === "revoked") throw new Error(`\u672A\u5B89\u88C5\u6216\u5DF2\u64A4\u9500: ${e}`);
  const i = n ?? t.version;
  if (i !== t.version) throw new Error(`\u65E0\u53EF\u5B89\u88C5\u7684\u7248\u672C ${i}`);
  const o = yl(t);
  return li(a.nativeVersions, i, o), a.nativeVersions ?? (a.nativeVersions = {}), (_a2 = a.nativeVersions)[i] ?? (_a2[i] = o), a.version !== i && (a.history.includes(a.version) || a.history.push(a.version), a.version = i, a.history.includes(i) || a.history.push(i)), a.logicFingerprint = t.logicFingerprint, a.updatedAt = st(), a.lifecycle === "installed" || a.lifecycle, qn(s), s;
}
function wv(e, n) {
  var _a2;
  const t = Je();
  if (t.matchFrozen) throw new Error("\u5BF9\u5C40\u4E2D\u4E0D\u53EF\u56DE\u9000");
  const s = t.installed.find((i) => i.key === e);
  if (!s || s.lifecycle === "revoked") throw new Error(`\u672A\u5B89\u88C5\u6216\u5DF2\u64A4\u9500: ${e}`);
  if (!s.history.includes(n)) throw new Error(`\u65E0\u5386\u53F2\u7248\u672C ${n}`);
  if (!((_a2 = s.nativeVersions) == null ? void 0 : _a2[n])) throw new Error(`\u7F3A\u5C11\u5DF2\u5B89\u88C5\u7684\u7248\u672C\u5185\u5BB9 ${n}`);
  const a = rs(JSON.parse(s.nativeVersions[n]));
  if (tt(a) !== e || a.manifest.version !== n) throw new Error(`\u5386\u53F2\u5305\u8EAB\u4EFD\u6216\u7248\u672C\u4E0D\u5339\u914D: ${e}`);
  return s.version = n, s.logicFingerprint = ls(a).slice(0, 8), s.updatedAt = st(), qn(t), t;
}
function bv(e) {
  const n = Je();
  if (n.matchFrozen) throw new Error("\u5BF9\u5C40\u4E2D\u4E0D\u53EF\u5378\u8F7D");
  return n.installed.some((t) => t.key === e && t.lifecycle === "revoked") && (n.revokedKeys ?? (n.revokedKeys = []), n.revokedKeys.includes(e) || n.revokedKeys.push(e)), n.contentLock = n.contentLock.filter((t) => t !== e), n.installed = n.installed.filter((t) => t.key !== e), qn(n), n;
}
function xv(e) {
  const n = Je();
  if (n.matchFrozen) throw new Error("\u5BF9\u5C40\u4E2D\u4E0D\u53EF\u64A4\u9500");
  const t = n.installed.find((s) => s.key === e);
  if (!t) throw new Error(`\u672A\u5B89\u88C5: ${e}`);
  return t.lifecycle = "revoked", n.revokedKeys ?? (n.revokedKeys = []), n.revokedKeys.includes(e) || n.revokedKeys.push(e), t.updatedAt = st(), n.contentLock = n.contentLock.filter((s) => s !== e), qn(n), n;
}
function $v(e = Je()) {
  const n = e.contentLock.map((s) => {
    const a = e.installed.find((i) => i.key === s);
    return a ? { key: s, version: a.version, fingerprint: a.logicFingerprint } : null;
  }).filter(Boolean), t = n.map((s) => `${s.key}@${s.version}:${s.fingerprint}`).join("|");
  return { packages: n, hashSeed: t };
}
function jv() {
  let e = Je();
  for (const n of Kt) {
    const t = ri(n);
    e = xl(t), e = jl(t);
  }
  return e;
}
function Sl() {
  if (typeof localStorage > "u") return [];
  try {
    const e = localStorage.getItem(wl);
    return e ? JSON.parse(e) : [];
  } catch {
    return [];
  }
}
function Sv(e) {
  const n = Sl().filter((t) => t.id !== e.id);
  return n.unshift(e), localStorage.setItem(wl, JSON.stringify(n.slice(0, 20))), n;
}
function El(e) {
  const n = e.map(tt);
  if (new Set(n).size !== n.length) throw new Error("\u5185\u5BB9\u9501\u5305\u542B\u91CD\u590D\u6269\u5C55");
  const t = new Map(e.map((s) => [tt(s), s]));
  for (const s of e) for (const a of s.manifest.dependencies ?? []) {
    const i = `${a.namespace}/${a.package_id}`, o = t.get(i);
    if (!o) throw new Error(`\u6269\u5C55\u5305\u7F3A\u5C11\u4F9D\u8D56: ${i}@${a.version}`);
    const r = a.logic_hash;
    if (o.manifest.version !== a.version || (r == null ? void 0 : r.some((l) => l !== 0)) && r.some((l, d) => l !== o.manifest.logic_hash[d])) throw new Error(`\u6269\u5C55\u5305\u4F9D\u8D56\u7248\u672C\u6216\u6307\u7EB9\u4E0D\u5339\u914D: ${i}@${a.version}`);
  }
  return { packsJson: e.length ? JSON.stringify(e) : void 0, contentLock: n, lockSeed: e.map((s) => `${tt(s)}@${s.manifest.version}:${ls(s)}`).join("|") };
}
function Ev(e = Is()) {
  const n = e.contentLock.map((t) => {
    var _a2;
    const s = e.installed.find((o) => o.key === t);
    if (!s || s.lifecycle !== "active") throw new Error(`\u5185\u5BB9\u9501\u4E2D\u7684\u5305\u672A\u542F\u7528: ${t}`);
    const a = (_a2 = s.nativeVersions) == null ? void 0 : _a2[s.version];
    if (!a) throw new Error(`\u7F3A\u5C11\u5DF2\u5B89\u88C5\u5305\u5185\u5BB9: ${t}\uFF0C\u8BF7\u5148\u5728\u6269\u5C55\u7BA1\u7406\u9875\u5347\u7EA7\u6216\u91CD\u65B0\u5B89\u88C5`);
    const i = rs(JSON.parse(a));
    if (tt(i) !== t || i.manifest.version !== s.version || !/^[a-f0-9]{8,64}$/.test(s.logicFingerprint) || !ls(i).startsWith(s.logicFingerprint)) throw new Error(`\u5DF2\u5B89\u88C5\u5305\u7248\u672C\u6216\u6307\u7EB9\u4E0D\u5339\u914D: ${t}`);
    return i;
  });
  return El(n);
}
function Av(e, n = Is()) {
  var _a2;
  const t = e.packsJson ? JSON.parse(e.packsJson) : [];
  if (!Array.isArray(t)) throw new Error("\u5B58\u6863\u6269\u5C55\u5185\u5BB9\u5FC5\u987B\u662F\u6570\u7EC4");
  const s = El(t.map(rs));
  if (s.lockSeed !== e.lockSeed || JSON.stringify(s.contentLock) !== JSON.stringify(e.contentLock)) throw new Error("\u5B58\u6863\u5185\u5BB9\u9501\u4E0E\u5305\u526F\u672C\u4E0D\u5339\u914D\uFF1B\u65E7\u6269\u5C55\u5B58\u6863\u7F3A\u5C11\u5B9E\u9645\u5305\u5185\u5BB9\u65F6\u4E0D\u80FD\u6062\u590D");
  for (const a of s.contentLock) if (((_a2 = n.revokedKeys) == null ? void 0 : _a2.includes(a)) || n.installed.some((i) => i.key === a && i.lifecycle === "revoked")) throw new Error(`\u5B58\u6863\u5305\u542B\u5DF2\u64A4\u9500\u5305: ${a}`);
  return s;
}
const Cv = { 1: "\u5BF9\u5C40\u5F00\u59CB", 2: "\u6478\u724C", 3: "\u4F7F\u7528\u6216\u54CD\u5E94\u724C", 4: "\u4F24\u5BB3\u7ED3\u7B97", 5: "\u56DE\u590D\u4F53\u529B", 6: "\u6FD2\u6B7B\u72B6\u6001\u53D8\u5316", 7: "\u73A9\u5BB6\u9635\u4EA1", 8: "\u9636\u6BB5\u6216\u56DE\u5408\u63A8\u8FDB", 9: "\u5BF9\u5C40\u7ED3\u675F", 10: "\u9009\u62E9\u72B6\u6001\u53D8\u5316" };
function Rv(e) {
  return Cv[e] ?? `\u4E8B\u4EF6 ${e}`;
}
const fs = 6, Ps = "noname-next.pending-command", zv = 600 * 1e3;
function Iv() {
  try {
    if (typeof sessionStorage > "u") return null;
    const e = sessionStorage.getItem(Ps);
    if (!e) return null;
    const n = JSON.parse(e);
    return !n || typeof n.roomId != "number" || typeof n.key != "number" || typeof n.createdAt != "number" || !n.kind || Date.now() - n.createdAt > zv ? null : { roomId: n.roomId, key: n.key, kind: n.kind, createdAt: n.createdAt };
  } catch {
    return null;
  }
}
function Pv(e, n, t) {
  try {
    if (typeof sessionStorage > "u" || !t) return;
    sessionStorage.setItem(Ps, JSON.stringify({ roomId: t, key: n, kind: e, createdAt: Date.now() }));
  } catch {
  }
}
function ms(e) {
  try {
    if (typeof sessionStorage > "u") return;
    const n = sessionStorage.getItem(Ps);
    if (!n) return;
    JSON.parse(n).key === e && sessionStorage.removeItem(Ps);
  } catch {
  }
}
class Tv {
  constructor() {
    __publicField(this, "socket", null);
    __publicField(this, "connectionId", 0);
    __publicField(this, "roomId", 0);
    __publicField(this, "playerId", 0);
    __publicField(this, "revision", 0);
    __publicField(this, "connSeq", 0);
    __publicField(this, "idemSeq", 1);
    __publicField(this, "choice", null);
    __publicField(this, "session", null);
    __publicField(this, "roomPassword", "");
    __publicField(this, "welcomeWaiter", null);
    __publicField(this, "joinWaiter", null);
    __publicField(this, "pending", /* @__PURE__ */ new Map());
    __publicField(this, "persistedPending", Iv());
    __publicField(this, "snapshotWaiters", /* @__PURE__ */ new Map());
    __publicField(this, "envelopeListeners", /* @__PURE__ */ new Set());
    __publicField(this, "rosterListeners", /* @__PURE__ */ new Set());
    __publicField(this, "closeListeners", /* @__PURE__ */ new Set());
    __publicField(this, "heartbeat", null);
    __publicField(this, "disposed", false);
    __publicField(this, "lastUrl", null);
    __publicField(this, "lastName", "");
    __publicField(this, "reconnectTimer", null);
    __publicField(this, "reconnectAttempts", 0);
    __publicField(this, "reconnectListeners", /* @__PURE__ */ new Set());
  }
  async connect(n, t) {
    var _a2;
    if (((_a2 = this.socket) == null ? void 0 : _a2.readyState) === WebSocket.OPEN) return;
    this.disposed = false, this.lastUrl = n, this.lastName = t;
    const s = new Promise((i, o) => {
      this.welcomeWaiter = { resolve: i, reject: o };
    }), a = new Promise((i, o) => {
      const r = new WebSocket(Al(n));
      this.socket = r, r.onopen = () => {
        if (this.socket !== r) {
          o(new Error("WebSocket \u8FDE\u63A5\u5DF2\u66FF\u6362"));
          return;
        }
        this.connSeq = 0, this.idemSeq = 1, i();
      }, r.onerror = () => o(new Error("WebSocket \u8FDE\u63A5\u5931\u8D25")), r.onmessage = (l) => {
        this.socket === r && this.handleMessage(String(l.data));
      }, r.onclose = () => {
        o(new Error("WebSocket \u8FDE\u63A5\u5DF2\u5173\u95ED")), this.socket === r && this.handleClose();
      };
    });
    s.catch(() => {
    }), await a, await s, this.send({ type: "Hello", payload: { versions: { engine_api: 1, rule_vm_abi: 167, content_schema: 3, network_protocol: fs, replay_schema: 1, snapshot_schema: 1, package_format: 1, engine_build_id: "noname-next-web" }, display_name: t } });
  }
  createRoom(n, t, s = "", a = 30) {
    return this.joinRequest({ type: "CreateRoom", payload: { players: n, seed: t, password: s, choice_timeout_secs: a } }, s);
  }
  joinRoom(n, t = "", s = null) {
    return this.roomId = n, this.joinRequest({ type: "JoinRoom", payload: { room_id: n, password: t, preferred_seat: null, reconnect_token: s } }, t);
  }
  spectate(n) {
    return this.roomId = n, this.joinRequest({ type: "Spectate", payload: { room_id: n } }, "");
  }
  rejoinRoom() {
    const n = this.session;
    return n ? n.role === "Spectator" ? this.spectate(n.roomId) : this.joinRoom(n.roomId, this.roomPassword, n.reconnectToken) : Promise.reject(new Error("\u5C1A\u672A\u52A0\u5165\u623F\u95F4"));
  }
  onEnvelope(n) {
    return this.envelopeListeners.add(n), () => this.envelopeListeners.delete(n);
  }
  onClose(n) {
    return this.closeListeners.add(n), () => this.closeListeners.delete(n);
  }
  onRoster(n) {
    return this.rosterListeners.add(n), () => this.rosterListeners.delete(n);
  }
  cancelPendingCommands(n = "\u5DF2\u53D6\u6D88\u5F85\u786E\u8BA4\u547D\u4EE4") {
    for (const t of this.pending.values()) ms(t.key), t.reject(new Error(n));
    this.pending.clear(), this.persistedPending && (ms(this.persistedPending.key), this.persistedPending = null);
  }
  endPhase(n = 0) {
    return this.submit("EndPhase");
  }
  playCard(n, t, s) {
    return this.submit({ PlayCard: { card: t, targets: s, card_targets: [] } });
  }
  activateSkill(n, t, s, a) {
    return this.submit({ ActivateSkill: { skill: t, targets: s, card_targets: a } });
  }
  useSkillCard(n, t, s, a, i) {
    return this.submit({ UseSkillCard: { skill: t, card_kind: s, targets: a, equip_slot: i } });
  }
  resolveChoice(n, t, s, a = false) {
    return this.submit({ ResolveChoice: { choice_id: t, selections: s, cancelled: a } });
  }
  surrender() {
    return this.submit("Surrender");
  }
  requestSnapshot() {
    const n = this.nextHeader();
    return new Promise((t, s) => {
      this.snapshotWaiters.set(n.conn_seq, { resolve: t, reject: s });
      try {
        this.send({ type: "RequestSnapshot", payload: { header: n, last_revision: this.revision } });
      } catch (a) {
        this.snapshotWaiters.delete(n.conn_seq), s(a);
      }
    });
  }
  dispose() {
    this.disposed = true, this.reconnectTimer != null && (window.clearTimeout(this.reconnectTimer), this.reconnectTimer = null), this.reconnectAttempts = 0, this.heartbeat != null && typeof window < "u" && window.clearInterval(this.heartbeat), this.heartbeat = null;
    const n = this.socket;
    this.socket = null, n == null ? void 0 : n.close(1e3, "client dispose"), this.session = null, this.roomPassword = "", this.playerId = 0, this.rejectOutstanding(new Error("\u8FDE\u63A5\u5DF2\u5173\u95ED")), this.envelopeListeners.clear(), this.rosterListeners.clear(), this.closeListeners.clear(), this.reconnectListeners.clear();
  }
  async joinRequest(n, t) {
    var _a2;
    const s = this.waitForJoin();
    try {
      this.send(n), this.roomPassword = t;
    } catch (a) {
      (_a2 = this.joinWaiter) == null ? void 0 : _a2.reject(a instanceof Error ? a : new Error(String(a))), this.joinWaiter = null;
    }
    return s;
  }
  waitForJoin() {
    if (this.joinWaiter) throw new Error("\u5DF2\u6709\u5165\u623F\u8BF7\u6C42");
    return new Promise((n, t) => {
      this.joinWaiter = { resolve: n, reject: t };
    });
  }
  submit(n) {
    if (!this.roomId || !this.playerId) return Promise.reject(new Error("\u5F53\u524D\u4E0D\u662F\u5DF2\u5165\u5EA7\u73A9\u5BB6"));
    const t = this.connectionId * 1e6 + this.idemSeq++;
    let s;
    const a = new Promise((i, o) => {
      s = { kind: n, key: t, resolve: i, reject: o }, this.pending.set(t, s);
    });
    try {
      this.sendSubmit(s), this.persistedPending = { roomId: this.roomId, key: t, kind: n, createdAt: Date.now() }, Pv(n, t, this.roomId);
    } catch (i) {
      return this.pending.delete(t), ms(t), Promise.reject(i);
    }
    return a;
  }
  sendSubmit(n, t = false) {
    this.send({ type: "SubmitCommand", payload: { header: this.nextHeader(), command: { actor: this.playerId, expected_revision: this.revision, idempotency_key: n.key, kind: n.kind }, idempotency_key: n.key, replay: t } });
  }
  replayPending() {
    const n = this.persistedPending && this.persistedPending.roomId === this.roomId ? this.persistedPending : null;
    if (n && !this.pending.has(n.key)) {
      const t = { kind: n.kind, key: n.key, resolve: (s) => {
        for (const a of this.envelopeListeners) a(s);
      }, reject: () => {
      } };
      this.pending.set(t.key, t);
    }
    if (this.pending.size) for (const t of [...this.pending.values()]) try {
      this.sendSubmit(t, true);
    } catch {
      return;
    }
  }
  nextHeader() {
    return { protocol_version: fs, room_id: this.roomId, conn_seq: ++this.connSeq, revision: this.revision };
  }
  send(n) {
    var _a2;
    if (((_a2 = this.socket) == null ? void 0 : _a2.readyState) !== WebSocket.OPEN) throw new Error("WebSocket \u672A\u8FDE\u63A5");
    this.socket.send(JSON.stringify(n));
  }
  handleMessage(n) {
    var _a2, _b2, _c2, _d2;
    let t;
    try {
      t = JSON.parse(n);
    } catch {
      this.rejectOutstanding(new Error("\u670D\u52A1\u7AEF\u8FD4\u56DE\u4E86\u65E0\u6548 JSON"));
      return;
    }
    const s = t.payload ?? {};
    if (s.view) try {
      dl(s.view, s.choice ?? null);
    } catch (a) {
      this.rejectOutstanding(new Error(`\u670D\u52A1\u7AEF\u5143\u6570\u636E\u65E0\u6548\uFF1A${String(a)}`));
      return;
    }
    switch (t.type) {
      case "Welcome":
        if (s.protocol_version !== fs || ((_a2 = s.versions) == null ? void 0 : _a2.network_protocol) !== fs) {
          this.rejectOutstanding(new Error("\u670D\u52A1\u7AEF\u534F\u8BAE\u7248\u672C\u4E0D\u517C\u5BB9\uFF1A\u9700\u8981 WS6")), (_b2 = this.socket) == null ? void 0 : _b2.close();
          return;
        }
        this.connectionId = Number(s.connection_id ?? 0), (_c2 = this.welcomeWaiter) == null ? void 0 : _c2.resolve(), this.welcomeWaiter = null;
        break;
      case "RoomCreated":
        this.roomId = Number(s.room_id ?? 0);
        break;
      case "JoinOk": {
        this.roomId = Number(s.room_id ?? this.roomId), this.playerId = Number(s.player_id ?? 0);
        const a = s.view;
        this.choice = s.choice ?? null, this.revision = a.revision;
        const i = { roomId: this.roomId, playerId: this.playerId, seat: Number(s.seat ?? 255), role: String(s.role ?? "Spectator"), reconnectToken: String(s.reconnect_token ?? ""), view: a, envelope: ha(a, this.choice, [], s.legal_actions ?? [], null) };
        this.session = i, this.replayPending(), (_d2 = this.joinWaiter) == null ? void 0 : _d2.resolve(i), this.joinWaiter = null, this.startHeartbeat();
        break;
      }
      case "RoomRoster": {
        const a = s;
        if (a.choice_timeout_secs !== null && ![10, 15, 30, 60, 90].includes(a.choice_timeout_secs) || !Array.isArray(a.seats) || a.seats.some((i) => typeof i.managed != "boolean")) {
          this.rejectOutstanding(new Error("\u670D\u52A1\u7AEF\u623F\u95F4\u8D85\u65F6\u914D\u7F6E\u65E0\u6548"));
          return;
        }
        for (const i of this.rosterListeners) i(a);
        break;
      }
      case "CommandOutcome":
        this.handleOutcome(s);
        break;
      case "Snapshot": {
        const a = s.header, i = s.view;
        this.choice = s.choice ?? null, this.revision = i.revision;
        const o = this.snapshotWaiters.get(a.conn_seq);
        o && (this.snapshotWaiters.delete(a.conn_seq), o.resolve(i));
        const r = ha(i, this.choice, [], s.legal_actions ?? [], null);
        for (const l of this.envelopeListeners) l(r);
        break;
      }
      case "Error": {
        const a = new Error(String(s.message ?? "\u670D\u52A1\u7AEF\u9519\u8BEF"));
        this.joinWaiter ? (this.joinWaiter.reject(a), this.joinWaiter = null) : this.rejectOutstanding(a);
        break;
      }
    }
  }
  handleOutcome(n) {
    var _a2;
    const t = n.view, s = this.revision;
    this.revision = t.revision, this.choice = n.choice ?? null;
    const a = n.public_event_codes ?? [], i = ha(t, this.choice, a, n.legal_actions ?? [], n.result ?? null), o = Number(n.idempotency_key ?? 0);
    ms(o), ((_a2 = this.persistedPending) == null ? void 0 : _a2.key) === o && (this.persistedPending = null);
    const r = this.pending.get(o);
    if (r) {
      this.pending.delete(o);
      const l = qv(n.result);
      if (l) {
        for (const d of this.envelopeListeners) d(i);
        r.reject(new Error(l));
      } else r.resolve(i);
    } else for (const l of this.envelopeListeners) l(i);
    s > 0 && t.revision > s + 1 && this.requestSnapshot().catch(() => {
    });
  }
  startHeartbeat() {
    this.heartbeat != null && window.clearInterval(this.heartbeat), this.heartbeat = window.setInterval(() => {
      var _a2;
      if (((_a2 = this.socket) == null ? void 0 : _a2.readyState) !== WebSocket.OPEN) return;
      const n = this.nextHeader();
      this.send({ type: "Heartbeat", payload: { header: n } });
    }, 15e3);
  }
  handleClose() {
    this.heartbeat != null && window.clearInterval(this.heartbeat);
    for (const n of this.closeListeners) try {
      n();
    } catch {
    }
    this.heartbeat = null, this.socket = null, this.playerId = 0, this.disposed || (this.suspendOutstanding(new Error("\u4E0E\u670D\u52A1\u7AEF\u7684\u8FDE\u63A5\u5DF2\u65AD\u5F00")), this.scheduleReconnect());
  }
  scheduleReconnect() {
    if (this.reconnectTimer != null || !this.lastUrl) return;
    if (this.reconnectAttempts >= 5) {
      this.failPendingUncertain();
      for (const t of this.reconnectListeners) t("failed");
      return;
    }
    const n = Math.min(1e3 * 2 ** this.reconnectAttempts, 8e3);
    this.reconnectAttempts += 1;
    for (const t of this.reconnectListeners) t("reconnecting");
    this.reconnectTimer = window.setTimeout(() => {
      this.reconnectTimer = null;
      const t = this.lastUrl;
      !t || this.disposed || this.connect(t, this.lastName).then(() => {
        this.reconnectAttempts = 0;
        for (const s of this.reconnectListeners) s("reconnected");
      }).catch(() => {
        this.scheduleReconnect();
      });
    }, n);
  }
  onReconnect(n) {
    return this.reconnectListeners.add(n), () => this.reconnectListeners.delete(n);
  }
  suspendOutstanding(n) {
    var _a2, _b2;
    (_a2 = this.welcomeWaiter) == null ? void 0 : _a2.reject(n), this.welcomeWaiter = null, (_b2 = this.joinWaiter) == null ? void 0 : _b2.reject(n), this.joinWaiter = null;
    for (const t of this.snapshotWaiters.values()) t.reject(n);
    this.snapshotWaiters.clear();
  }
  failPendingUncertain() {
    if (this.pending.size) {
      for (const n of this.pending.values()) n.reject(new Error("\u63D0\u4EA4\u7ED3\u679C\u4E0D\u786E\u5B9A\uFF1A\u8FDE\u63A5\u5DF2\u65AD\u5F00\u4E14\u81EA\u52A8\u91CD\u8FDE\u5931\u8D25\uFF0C\u8BF7\u91CD\u65B0\u8FDB\u5165\u623F\u95F4\u540E\u518D\u64CD\u4F5C"));
      this.pending.clear();
    }
  }
  rejectOutstanding(n) {
    this.suspendOutstanding(n);
    for (const t of this.pending.values()) t.reject(n);
    this.pending.clear();
  }
}
function Al(e) {
  const n = e.trim();
  return n.startsWith("http://") ? `ws://${n.slice(7)}` : n.startsWith("https://") ? `wss://${n.slice(8)}` : n;
}
function Do(e, n) {
  const t = new URL(Al(e)).origin;
  return `noname-next.reconnect.${encodeURIComponent(t)}.${n}`;
}
function ha(e, n, t, s, a) {
  return { result: a, events: t.map((i) => ({ PublicEvent: { code: i, label: Rv(i) } })), choice: n, view: e, legal_actions: s, hash_hex: Ov(e.state_hash), revision: e.revision, finished: Nv(e.outcome) };
}
function Ov(e) {
  return typeof e == "string" ? e : e.map((n) => n.toString(16).padStart(2, "0")).join("");
}
function Nv(e) {
  return e == null || e === "InProgress" ? false : !(typeof e == "object" && "InProgress" in e);
}
function qv(e) {
  var _a2;
  if (!e || typeof e != "object" || !("Rejected" in e)) return null;
  const n = e.Rejected;
  return (n && typeof n == "object" && "IllegalAction" in n ? (_a2 = n.IllegalAction) == null ? void 0 : _a2.code : void 0) === "legacy_replay_no_receipt" ? "\u63D0\u4EA4\u7ED3\u679C\u4E0D\u786E\u5B9A\uFF1A\u65E7\u623F\u95F4\u6CA1\u6709\u53EF\u6838\u5BF9\u7684\u56DE\u6267\uFF0C\u5DF2\u505C\u6B62\u91CD\u653E\uFF1B\u8BF7\u4EE5\u623F\u95F4\u5F53\u524D\u72B6\u6001\u4E3A\u51C6" : `\u547D\u4EE4\u88AB\u62D2\u7EDD\uFF1A${JSON.stringify(n)}`;
}
function Cl(e, n) {
  var _a2, _b2;
  const t = (i) => ({ label: i, isSelfWin: null });
  if (e == null) return t("\u80DC\u8D1F\u672A\u77E5");
  if (e === "finished") return t("\u5DF2\u7ED3\u675F\uFF08\u80DC\u8D1F\u7C7B\u578B\u672A\u5EFA\u6A21\uFF09");
  const s = typeof e == "object" ? e : {};
  switch (typeof e == "string" ? e : Object.keys(s)[0]) {
    case "LoyalistWin":
      return { label: "\u4E3B\u516C\u65B9\u80DC", isSelfWin: (n == null ? void 0 : n.identity) ? n.identity === "Lord" || n.identity === "Loyalist" : null };
    case "RebelWin":
      return { label: "\u53CD\u8D3C\u80DC", isSelfWin: (n == null ? void 0 : n.identity) ? n.identity === "Rebel" : null };
    case "RenegadeWin":
      return { label: "\u5185\u5978\u80DC", isSelfWin: (n == null ? void 0 : n.identity) ? n.identity === "Renegade" : null };
    case "SingleWin": {
      const i = (_a2 = s.SingleWin) == null ? void 0 : _a2.player;
      if (typeof i == "number" && Number.isInteger(i) && i >= 0) return { label: `\u5355\u6311\u80DC\u8005 P${i}`, isSelfWin: n ? n.player === i : null };
      break;
    }
    case "GroupWin": {
      const i = (_b2 = s.GroupWin) == null ? void 0 : _b2.group;
      if (typeof i == "string") return t(`\u52BF\u529B\u80DC\uFF08${i}\uFF09`);
      break;
    }
    case "Draw":
      return t("\u5E73\u5C40");
    case "Aborted":
      return t("\u5BF9\u5C40\u4E2D\u6B62");
  }
  return t(`\u5DF2\u7ED3\u675F\uFF08${JSON.stringify(e)}\uFF09`);
}
function Mv(e) {
  return Cl(e).label;
}
function Dv(e, n, t) {
  var _a2;
  let s = e == null ? void 0 : e.outcome;
  if (s == null) for (const a of n) {
    if (!a || typeof a != "object") continue;
    const i = a;
    if (s = i.type === "MatchEnded" ? i.outcome : (_a2 = i.MatchEnded) == null ? void 0 : _a2.outcome, s != null) break;
  }
  return Cl(s, e == null ? void 0 : e.seats.find((a) => a.player === t));
}
function Lv(e) {
  if (!e || typeof e != "object") return "\u547D\u4EE4\u7ED3\u679C\u65E0\u6CD5\u8BFB\u53D6";
  let n = e;
  if (n.kind === "plan" && typeof n.error == "string") return `\u81EA\u52A8\u64CD\u4F5C\u5931\u8D25\uFF1A${n.error === "stale" ? "\u5C40\u9762\u5DF2\u53D8\u5316\uFF0C\u8BF7\u91CD\u8BD5" : n.error === "cancelled" ? "\u5DF2\u53D6\u6D88" : n.error}`;
  if (n.kind === "plan" || n.kind === "answer") try {
    const a = typeof n.outcome == "string" ? JSON.parse(n.outcome) : null;
    if (!a || typeof a != "object" || Array.isArray(a)) return "\u81EA\u52A8\u64CD\u4F5C\u7ED3\u679C\u65E0\u6CD5\u8BFB\u53D6";
    n = a;
  } catch {
    return "\u81EA\u52A8\u64CD\u4F5C\u7ED3\u679C\u65E0\u6CD5\u8BFB\u53D6";
  }
  if (n.done === true || n.outcome === "accepted" || n.outcome === "pending") return null;
  if (n.outcome === "suspended") return "\u64CD\u4F5C\u5DF2\u6682\u505C\uFF0C\u6682\u65E0\u6CD5\u7EE7\u7EED\u7ED3\u7B97";
  if (n.outcome !== "rejected") return "\u547D\u4EE4\u7ED3\u679C\u65E0\u6CD5\u8BFB\u53D6";
  const t = { ChoicePending: "\u8BF7\u5148\u5B8C\u6210\u5F53\u524D\u9009\u62E9", NoSuchChoice: "\u5F53\u524D\u9009\u62E9\u5DF2\u5931\u6548", InvalidChoice: "\u9009\u62E9\u65E0\u6548", NotYourTurn: "\u8FD8\u6CA1\u6709\u8F6E\u5230\u8BE5\u73A9\u5BB6", WrongPhase: "\u5F53\u524D\u9636\u6BB5\u4E0D\u80FD\u6267\u884C\u6B64\u64CD\u4F5C", CardNotInHand: "\u8FD9\u5F20\u724C\u4E0D\u5728\u624B\u724C\u4E2D", IllegalTargets: "\u76EE\u6807\u4E0D\u5408\u6CD5", WrongPlayer: "\u5F53\u524D\u73A9\u5BB6\u65E0\u6743\u5904\u7406\u6B64\u9009\u62E9", StaleRevision: "\u5C40\u9762\u5DF2\u53D8\u5316\uFF0C\u8BF7\u91CD\u8BD5", MatchOver: "\u5BF9\u5C40\u5DF2\u7ED3\u675F" }, s = typeof n.reason == "string" && n.reason ? n.reason : "\u539F\u56E0\u672A\u77E5";
  return `\u64CD\u4F5C\u88AB\u62D2\u7EDD\uFF1A${t[s] ?? s}`;
}
let Uv = 1;
function Fv() {
  var _a2;
  const e = typeof window < "u" && ((_a2 = window.matchMedia) == null ? void 0 : _a2.call(window, "(prefers-reduced-motion: reduce)").matches);
  return xt({ connected: false, busy: false, error: null, revision: 0, hash: "", finished: false, view: null, choice: null, legalActions: [], lastEvents: [], log: [], effects: [], speed: 1, skipAnims: false, reduceMotion: !!e });
}
let it = 1;
function Lo(e, n) {
  rh(), Vv(e, n), lh();
}
function Vv(e, n) {
  e.revision = n.revision, e.hash = n.hash_hex, e.finished = n.finished, e.view = n.view, e.choice = n.choice, e.legalActions = n.legal_actions ?? [], e.lastEvents = n.events ?? [], e.error = null;
  const t = Hv(n.events ?? []);
  e.effects = t;
  const s = Bv(n.events ?? []);
  for (const a of s) Qe(e, a);
}
function Hv(e) {
  const n = [];
  for (const t of e) {
    if (!t || typeof t != "object") continue;
    const s = t, a = Object.keys(s)[0];
    if (!a) continue;
    const i = s[a];
    switch (a) {
      case "DamageDealt":
        n.push({ id: it++, type: "damage", player: i == null ? void 0 : i.target, amount: i == null ? void 0 : i.amount, text: `-${i == null ? void 0 : i.amount}` });
        break;
      case "HpRecovered":
        n.push({ id: it++, type: "heal", player: i == null ? void 0 : i.player, amount: i == null ? void 0 : i.amount, text: `+${i == null ? void 0 : i.amount}` });
        break;
      case "CardsDrawn":
        n.push({ id: it++, type: "draw", player: i == null ? void 0 : i.player, text: `\u6478${Array.isArray(i == null ? void 0 : i.cards) ? i.cards.length : 1}` });
        break;
      case "CardUsed":
        n.push({ id: it++, type: "play", player: i == null ? void 0 : i.player, text: "\u51FA\u724C" });
        break;
      case "EnteredDying":
        n.push({ id: it++, type: "dying", player: i == null ? void 0 : i.player, text: "\u6FD2\u6B7B" });
        break;
      case "PlayerKilled":
        n.push({ id: it++, type: "death", player: i == null ? void 0 : i.player, text: "\u9635\u4EA1" });
        break;
    }
  }
  return n;
}
function Wv(e) {
  e.effects = [];
}
function Qe(e, n) {
  e.log.unshift({ id: Uv++, text: n, at: Date.now() }), e.log.length > 80 && (e.log.length = 80);
}
function Bv(e) {
  const n = [];
  for (const t of e) {
    if (!t || typeof t != "object") continue;
    const s = t, a = Object.keys(s)[0];
    if (!a) continue;
    const i = s[a];
    switch (a) {
      case "GameStarted":
        n.push(`\u5BF9\u5C40\u5F00\u59CB \xB7 ${(i == null ? void 0 : i.player_count) ?? "?"} \u4EBA`);
        break;
      case "CardsDrawn":
        n.push(`\u73A9\u5BB6 ${i == null ? void 0 : i.player} \u6478\u724C \xD7${Array.isArray(i == null ? void 0 : i.cards) ? i.cards.length : "?"}`);
        break;
      case "CardUsed":
        n.push(`\u73A9\u5BB6 ${i == null ? void 0 : i.player} \u4F7F\u7528\u5361\u724C ${i == null ? void 0 : i.card}`);
        break;
      case "DamageDealt":
        n.push(`\u4F24\u5BB3 ${i == null ? void 0 : i.source} \u2192 ${i == null ? void 0 : i.target} (${i == null ? void 0 : i.amount})`);
        break;
      case "HpRecovered":
        n.push(`\u73A9\u5BB6 ${i == null ? void 0 : i.player} \u56DE\u590D ${i == null ? void 0 : i.amount}`);
        break;
      case "EnteredDying":
        n.push(`\u73A9\u5BB6 ${i == null ? void 0 : i.player} \u8FDB\u5165\u6FD2\u6B7B`);
        break;
      case "PlayerKilled":
        n.push(`\u73A9\u5BB6 ${i == null ? void 0 : i.player} \u9635\u4EA1`);
        break;
      case "TurnStateChanged":
        n.push(`\u56DE\u5408 \xB7 \u73A9\u5BB6 ${i == null ? void 0 : i.current} \xB7 ${i == null ? void 0 : i.phase}`);
        break;
      case "MatchEnded":
        n.push(`\u7EC8\u5C40 \xB7 ${Mv(i == null ? void 0 : i.outcome)}`);
        break;
      case "ChoiceRequested":
        n.push(`\u7B49\u5F85\u9009\u62E9 \xB7 \u73A9\u5BB6 ${i == null ? void 0 : i.observer}`);
        break;
      case "PublicEvent":
        n.push(String((i == null ? void 0 : i.label) ?? `\u516C\u5171\u4E8B\u4EF6 ${(i == null ? void 0 : i.code) ?? "?"}`));
        break;
      default:
        n.push(a);
    }
  }
  return n;
}
function Kv(e, n) {
  if (e.skipAnims || e.reduceMotion || e.speed <= 0) return Promise.resolve();
  const t = Math.max(0, n / e.speed);
  return new Promise((s) => setTimeout(s, t));
}
function lt(e) {
  return typeof e == "object" && e !== null && !Array.isArray(e);
}
function Ts(e) {
  return typeof e == "string" ? e : e instanceof Error ? e.message : String(e);
}
function Mt(e) {
  try {
    const n = JSON.stringify(e);
    return (n === void 0 ? String(e) : n).slice(0, 160);
  } catch {
    return String(e);
  }
}
function Gv(e) {
  return Array.isArray(e) && e.every((n) => typeof n == "string");
}
function Jv(e) {
  if (!lt(e)) throw new Error(`\u9009\u5C06\u5019\u9009\u7ED3\u6784\u4E0D\u7B26\uFF08\u9700 {count, candidates}\uFF09\uFF1A${Mt(e)}`);
  const n = e.count, t = e.candidates;
  if (typeof n != "number" || !Number.isInteger(n) || n < 0 || !Array.isArray(t)) throw new Error(`\u9009\u5C06\u5019\u9009\u7ED3\u6784\u4E0D\u7B26\uFF08\u9700 {count:\u975E\u8D1F\u6574\u6570, candidates:\u6570\u7EC4}\uFF09\uFF1A${Mt(e)}`);
  const s = [], a = /* @__PURE__ */ new Set();
  for (const [i, o] of t.entries()) {
    if (!lt(o)) throw new Error(`\u9009\u5C06\u5019\u9009\u7B2C ${i} \u9879\u7ED3\u6784\u4E0D\u7B26\uFF08\u9700\u5BF9\u8C61\uFF09\uFF1A${Mt(o)}`);
    const r = o.def, l = o.name, d = o.skills;
    if (typeof r != "number" || !Number.isInteger(r) || r < 0 || typeof l != "string" || l.trim().length === 0 || !Gv(d)) throw new Error(`\u9009\u5C06\u5019\u9009\u7B2C ${i} \u9879\u7ED3\u6784\u4E0D\u7B26\uFF08\u9700 {def:\u975E\u8D1F\u6574\u6570, name:\u975E\u7A7A\u5B57\u7B26\u4E32, skills:\u5B57\u7B26\u4E32\u6570\u7EC4}\uFF09\uFF1A${Mt(o)}`);
    if (a.has(r)) throw new Error(`\u9009\u5C06\u5019\u9009\u7B2C ${i} \u9879 def=${r} \u4E0E\u524D\u9762\u91CD\u590D`);
    a.add(r), s.push({ def: r, name: l, skills: [...d], skillLabel: d.length > 0 ? d.join(" \xB7 ") : "\u65E0\u6280\u80FD" });
  }
  if (s.length !== n) throw new Error(`\u9009\u5C06\u5019\u9009 count=${n} \u4E0E\u5019\u9009\u6570 ${s.length} \u4E0D\u4E00\u81F4`);
  return s;
}
function ui() {
  return { characterDef: null, skillIds: [], skillNames: [] };
}
function Yv(e, n = ui()) {
  return { seat: e, loadStatus: "idle", candidates: [], error: null, selectionStatus: "idle", pendingName: null, assignment: null, lastStateHash: null, projection: Rl(n) };
}
function Rl(e) {
  return { characterDef: e.characterDef, skillIds: [...e.skillIds], skillNames: [...e.skillNames] };
}
function zl(e, n) {
  const s = (Array.isArray(e == null ? void 0 : e.seats) ? e.seats : []).find((r) => r && r.player === n);
  if (!s) return ui();
  const a = [], i = [], o = Array.isArray(s.skills) ? s.skills : [];
  for (const r of o) {
    const l = r == null ? void 0 : r.id;
    typeof l == "number" && Number.isInteger(l) && !a.includes(l) && a.push(l);
    const d = r == null ? void 0 : r.name;
    typeof d == "string" && d.length > 0 && !i.includes(d) && i.push(d);
  }
  return { characterDef: typeof s.character_def == "number" ? s.character_def : null, skillIds: a, skillNames: i };
}
function Il(e, n) {
  return { ...e, projection: Rl(n) };
}
function Uo(e, n) {
  return { ...e, seat: n, projection: ui(), assignment: null, lastStateHash: null, selectionStatus: "idle", pendingName: null, error: null };
}
function Qv(e) {
  return { ...e, loadStatus: "loading", error: null };
}
function Xv(e, n) {
  try {
    const t = Jv(n);
    return { ...e, loadStatus: t.length > 0 ? "ready" : "empty", candidates: t, error: null };
  } catch (t) {
    return { ...e, loadStatus: "error", candidates: [], error: `\u9009\u5C06\u5019\u9009\u4E0D\u53EF\u7528\uFF1A${Ts(t)}` };
  }
}
function Zv(e, n) {
  return { ...e, loadStatus: "error", candidates: [], error: `\u9009\u5C06\u5019\u9009\u52A0\u8F7D\u5931\u8D25\uFF1A${Ts(n)}` };
}
function ey(e, n) {
  if (!lt(e)) return `\u56DE\u5305\u4E0D\u662F\u5BF9\u8C61\uFF1A${Mt(e)}`;
  const t = e.assignment, s = e.envelope;
  return lt(t) ? t.seat !== n.seat ? `\u5EA7\u4F4D\u56DE\u663E ${String(t.seat)} \u2260 \u8BF7\u6C42 ${n.seat}` : t.name !== n.name ? `\u6B66\u5C06\u56DE\u663E ${String(t.name)} \u2260 \u8BF7\u6C42 ${n.name}` : typeof t.state_hash != "string" || t.state_hash.length === 0 ? "\u7F3A assignment.state_hash" : !lt(s) || !lt(s.view) ? "\u7F3A\u5237\u65B0\u4FE1\u5C01 view" : null : "\u7F3A assignment";
}
async function ny(e, n, t, s) {
  var _a2;
  if (typeof n != "string" || n.trim().length === 0) return { ...e, selectionStatus: "rejected", pendingName: null, error: "\u9009\u5C06\u5931\u8D25\uFF1A\u5019\u9009\u540D\u4E3A\u7A7A" };
  const a = { ...e, selectionStatus: "pending", pendingName: n, error: null };
  s == null ? void 0 : s(a);
  let i;
  try {
    i = await t.chooseGeneral(e.seat, n);
  } catch (p) {
    return { ...e, selectionStatus: "rejected", pendingName: null, error: `\u9009\u5C06\u5931\u8D25\uFF08seat=${e.seat}, name=${n}\uFF09\uFF1A${Ts(p)}` };
  }
  const o = ey(i, { seat: e.seat, name: n });
  if (o) return { ...e, selectionStatus: "rejected", pendingName: null, error: `\u9009\u5C06\u8FD4\u56DE\u65E0\u6CD5\u786E\u8BA4\u88C5\u914D\uFF1A${o}` };
  const r = i.assignment, l = i.envelope, d = Il({ ...a, assignment: r, lastStateHash: r.state_hash }, zl(l.view, e.seat));
  let c = null;
  try {
    (_a2 = t.applyEnvelope) == null ? void 0 : _a2.call(t, l);
  } catch (p) {
    c = `\u9009\u5C06\u5DF2\u88C5\u914D\uFF0C\u4F46\u89C6\u56FE\u5237\u65B0\u5931\u8D25\uFF1A${Ts(p)}`;
  }
  return { ...d, selectionStatus: "accepted", pendingName: null, error: c };
}
const ty = ["data-slot", "aria-label"], sy = ["src", "alt"], ay = ["aria-label"], iy = { class: "name" }, oy = { key: 2, class: "muted" }, ry = ["value", "aria-label"], ly = ["value"], uy = De({ __name: "GeneralPortrait", props: { general: {}, legacyDef: {}, alive: { type: Boolean }, slot: {} }, setup(e) {
  const n = e, t = Y(() => {
    var _a2;
    return ((_a2 = n.general) == null ? void 0 : _a2.def) ?? n.legacyDef ?? 0;
  }), s = Y(() => n.general !== null || n.legacyDef !== void 0), a = Y(() => {
    var _a2;
    return s.value ? ((_a2 = n.general) == null ? void 0 : _a2.name) || fh(t.value) || `#${t.value}` : "\u6697\u5C06\uFF0F\u672A\u914D\u7F6E";
  }), i = { wei: "\u9B4F", shu: "\u8700", wu: "\u5434", qun: "\u7FA4" }, o = Y(() => {
    var _a2;
    const w = ((_a2 = n.general) == null ? void 0 : _a2.faction) ?? "";
    return Object.hasOwn(i, w) ? i[w] : "\u672A\u77E5\u52BF\u529B";
  }), r = Y(() => mh(t.value)), l = Y(() => Na(t.value)), d = Y(() => vh(t.value)), c = Y(() => yh(t.value)), p = Y(() => Ys(t.value)), m = Y(() => Array.from(/* @__PURE__ */ new Set([...n.alive ? [] : [kh(t.value, "dead")], d.value, c.value, p.value])).filter((w) => !!w)), f = W([]), b = Y(() => m.value.find((w) => !f.value.includes(w)) ?? "");
  kn([t, s], ([w, _]) => {
    _ && hh(w).catch(() => {
    });
  }, { immediate: true }), kn(m, () => {
    f.value = [];
  });
  function P() {
    b.value && (f.value = [...f.value, b.value]);
  }
  function O(w) {
    const _ = w.target.value;
    _h(t.value, _ === "" ? null : Number(_));
  }
  return (w, _) => (x(), j("div", { class: "general-slot", "data-slot": e.slot, "aria-label": `${e.slot === 0 ? "\u4E3B\u5C06" : "\u526F\u5C06"} ${a.value}` }, [s.value && b.value ? (x(), j("img", { key: 0, class: "portrait", src: b.value, alt: `${a.value}\u6B66\u5C06\u5934\u50CF`, onError: P }, null, 40, sy)) : (x(), j("div", { key: 1, class: "portrait no-image", role: "img", "aria-label": `${a.value}\u6682\u65E0\u5934\u50CF` }, $(s.value ? "\u65E0\u56FE" : "\u672A\u77E5"), 9, ay)), u("span", iy, $(a.value), 1), e.general ? (x(), j("span", oy, $(o.value) + " \xB7 " + $(e.general.revealed ? "\u660E\u7F6E" : "\u6697\u7F6E"), 1)) : V("", true), s.value && r.value.length ? (x(), j("select", { key: 3, class: "skin-select", value: l.value ?? "", "aria-label": `${a.value}\u76AE\u80A4`, onClick: _[0] || (_[0] = ja(() => {
  }, ["stop"])), onKeydown: _[1] || (_[1] = ja(() => {
  }, ["stop"])), onChange: O }, [_[2] || (_[2] = u("option", { value: "" }, "\u9ED8\u8BA4\u76AE\u80A4", -1)), (x(true), j(ee, null, ve(r.value, (A) => (x(), j("option", { key: A.id, value: A.id }, $(D(gh)(A)) + $(A.presentation.kind === "spine" ? "\uFF08\u9759\u6001\u56DE\u9000\uFF09" : ""), 9, ly))), 128))], 40, ry)) : V("", true)], 8, ty));
} }), cy = ln(uy, [["__scopeId", "data-v-689e5d5b"]]), dy = ["aria-label", "tabindex"], py = { key: 0, class: "tag" }, hy = { key: 1, class: "tag", "aria-label": "\u6B66\u5C06\u724C\u80CC\u9762" }, fy = { class: "generals", "aria-label": "\u53CC\u5C06" }, my = { class: "hp-row" }, gy = ["aria-label"], _y = { class: "hp", "aria-label": "\u4F53\u529B" }, vy = ["aria-valuenow", "aria-valuemax", "aria-label"], yy = { class: "meta muted" }, ky = { key: 0, class: "equip-zone", "aria-label": "\u88C5\u5907\u533A" }, wy = ["title"], by = { class: "equip-suit" }, xy = { class: "equip-id" }, $y = { key: 1, class: "dead-badge" }, jy = { key: 2, class: "dying-badge" }, Sy = { key: 3, class: "target-hint" }, Ey = De({ __name: "SeatCard", props: { seat: {}, self: { type: Boolean }, identityLabel: {}, isTargetMode: { type: Boolean }, isSelectableTarget: { type: Boolean }, isDropTarget: { type: Boolean }, equipCards: {} }, emits: ["select"], setup(e, { emit: n }) {
  const t = e, s = n, a = Y(() => t.seat.generals ?? [null, null]), i = Y(() => !t.seat.generals || a.value.every((d) => d === null) && t.seat.character_def !== 0 ? t.seat.character_def : void 0), o = Y(() => a.value.map((d, c) => (d == null ? void 0 : d.name) || (c === 0 && i.value !== void 0 ? `#${i.value}` : "\u6697\u5C06\uFF0F\u672A\u914D\u7F6E")).join(" / "));
  function r(d, c, p) {
    return p ? d <= 0 ? "\u26A0" : "\u2713" : "\u2717";
  }
  function l(d, c, p) {
    return p ? d <= 0 ? "var(--color-warning)" : "var(--color-positive)" : "var(--color-negative)";
  }
  return (d, c) => (x(), j("article", { class: Re(["seat has-portrait", { self: e.self, dead: !e.seat.alive, dying: e.seat.alive && e.seat.hp <= 0, "target-mode": e.isTargetMode, "target-selectable": e.isTargetMode && e.isSelectableTarget, "drop-target": e.isDropTarget, "current-turn": false }]), role: "listitem", "aria-label": `\u5EA7\u4F4D ${e.seat.seat} \u73A9\u5BB6 ${e.seat.player} \u6B66\u5C06 ${o.value}`, tabindex: e.isTargetMode && e.isSelectableTarget ? 0 : -1, onClick: c[0] || (c[0] = (p) => e.isTargetMode && e.isSelectableTarget && s("select", e.seat.player)), onKeydown: c[1] || (c[1] = Zc((p) => e.isTargetMode && e.isSelectableTarget && s("select", e.seat.player), ["enter"])) }, [u("header", null, [e.self ? (x(), j("span", py, "\u4F60")) : V("", true), e.seat.turned_over ? (x(), j("span", hy, "\u7FFB\u9762")) : V("", true)]), u("div", fy, [(x(true), j(ee, null, ve(a.value, (p, m) => (x(), ti(cy, { key: m, general: p, slot: m, "legacy-def": m === 0 ? i.value : void 0, alive: e.seat.alive }, null, 8, ["general", "slot", "legacy-def", "alive"]))), 128))]), u("div", my, [u("span", { class: "state-icon", style: Kn({ color: l(e.seat.hp, e.seat.max_hp, e.seat.alive) }), "aria-label": e.seat.alive ? e.seat.hp <= 0 ? "\u6FD2\u6B7B" : "\u5B58\u6D3B" : "\u9635\u4EA1" }, $(r(e.seat.hp, e.seat.max_hp, e.seat.alive)), 13, gy), u("span", _y, $(e.seat.hp) + " / " + $(e.seat.max_hp), 1)]), u("div", { class: "hp-bar", role: "progressbar", "aria-valuenow": Math.max(e.seat.hp, 0), "aria-valuemax": e.seat.max_hp, "aria-label": `\u4F53\u529B ${e.seat.hp}/${e.seat.max_hp}` }, [u("div", { class: "hp-bar-fill", style: Kn({ width: Math.max(Math.min(e.seat.hp / e.seat.max_hp * 100, 100), 0) + "%", background: e.seat.alive ? e.seat.hp <= 0 ? "var(--color-warning)" : "var(--color-positive)" : "var(--color-negative)" }) }, null, 4)], 8, vy), u("div", yy, [u("span", null, "\u624B\u724C " + $(e.seat.hand_count), 1), u("span", null, $(e.identityLabel), 1)]), e.equipCards.length > 0 ? (x(), j("div", ky, [(x(true), j(ee, null, ve(e.equipCards, (p) => (x(), j("div", { key: p.id, class: "equip-card", title: `\u5361\u724C #${p.id}${p.def != null ? " (def:" + p.def + ")" : ""}` }, [u("span", by, $(p.suit === "Spade" ? "\u2660" : p.suit === "Heart" ? "\u2665" : p.suit === "Club" ? "\u2663" : p.suit === "Diamond" ? "\u2666" : "?"), 1), u("span", xy, "#" + $(p.id), 1)], 8, wy))), 128))])) : V("", true), e.seat.alive ? e.seat.hp <= 0 ? (x(), j("div", jy, "\u26A0 \u6FD2\u6B7B")) : V("", true) : (x(), j("div", $y, "\u2717 \u9635\u4EA1")), e.isTargetMode && e.isSelectableTarget ? (x(), j("div", Sy, " \u25B8 \u70B9\u51FB\u9009\u62E9\u4E3A\u76EE\u6807 ")) : V("", true)], 42, dy));
} }), Ay = ln(Ey, [["__scopeId", "data-v-fdcad0a8"]]), Cy = { class: "hand", "aria-label": "\u624B\u724C" }, Ry = { class: "muted title" }, zy = { class: "strip", role: "listbox", "aria-label": "\u624B\u724C\u5217\u8868" }, Iy = ["disabled", "aria-selected", "aria-label", "onClick", "onDragstart"], Py = ["data-suit"], Ty = { class: "rank" }, Oy = { class: "id muted" }, Ny = { key: 0, class: "card-known" }, qy = { key: 0, class: "muted empty" }, My = { key: 0, class: "hint" }, Dy = { key: 1, class: "hint drag-hint" }, Ly = De({ __name: "HandStrip", props: { cards: {}, actionableIds: {} }, emits: ["select", "dragStart"], setup(e, { emit: n }) {
  const t = e, s = n, a = W(null), i = W(null);
  function o(p) {
    switch (p) {
      case "Spade":
        return "\u2660";
      case "Heart":
        return "\u2665";
      case "Club":
        return "\u2663";
      case "Diamond":
        return "\u2666";
      default:
        return "?";
    }
  }
  function r(p) {
    return p == null ? "?" : p === 1 ? "A" : p === 11 ? "J" : p === 12 ? "Q" : p === 13 ? "K" : String(p);
  }
  function l(p) {
    t.actionableIds.includes(p) && (a.value = p, s("select", p));
  }
  function d(p, m) {
    if (!t.actionableIds.includes(m)) {
      p.preventDefault();
      return;
    }
    i.value = m, p.dataTransfer && (p.dataTransfer.setData("text/plain", String(m)), p.dataTransfer.effectAllowed = "move"), s("dragStart", m);
  }
  function c() {
    i.value = null;
  }
  return (p, m) => (x(), j("section", Cy, [u("h2", Ry, " \u624B\u724C\uFF08" + $(e.cards.length) + " \u5F20" + $(a.value != null ? " \xB7 \u5DF2\u9009\u4E2D #" + a.value : " \xB7 \u70B9\u51FB\u6216\u62D6\u62FD") + "\uFF09 ", 1), u("div", zy, [(x(true), j(ee, null, ve(e.cards, (f) => (x(), j("button", { key: f.id, type: "button", class: Re(["card", { selected: f.id === a.value, dragging: f.id === i.value }]), disabled: !e.actionableIds.includes(f.id), role: "option", "aria-selected": f.id === a.value, "aria-label": `\u5361\u724C ${f.id}${f.def != null ? " \u5DF2\u77E5" : " \u672A\u77E5"}`, draggable: "true", onClick: (b) => l(f.id), onDragstart: (b) => d(b, f.id), onDragend: c }, [u("span", { class: "suit", "data-suit": f.suit }, $(o(f.suit)), 9, Py), u("span", Ty, $(r(f.rank)), 1), u("span", Oy, "#" + $(f.id), 1), f.def != null ? (x(), j("span", Ny, "\u25B8")) : V("", true)], 42, Iy))), 128)), e.cards.length ? V("", true) : (x(), j("p", qy, "\u65E0\u53EF\u89C1\u624B\u724C"))]), a.value != null ? (x(), j("p", My, " \u2713 \u5DF2\u9009\u4E2D #" + $(a.value) + " \u2014 \u5728\u68CB\u76D8\u4E0A\u9009\u62E9\u76EE\u6807\uFF0C\u6216\u70B9\u51FB\u300C\u7ED3\u675F\u9636\u6BB5\u300D\u76F4\u63A5\u6253\u51FA ", 1)) : V("", true), i.value != null ? (x(), j("p", Dy, " \u{1F0A0} \u62D6\u62FD\u4E2D #" + $(i.value) + " \u2014 \u62D6\u5230\u76EE\u6807\u73A9\u5BB6\u91CA\u653E\u51FA\u724C ", 1)) : V("", true)]));
} }), Uy = ln(Ly, [["__scopeId", "data-v-b2b16095"]]), Fy = { class: "board", role: "region", "aria-label": "\u6218\u573A" }, Vy = { key: 0, class: "action-bar panel", role: "toolbar", "aria-label": "\u884C\u52A8\u64CD\u4F5C" }, Hy = { class: "muted" }, Wy = { key: 2 }, By = { key: 3 }, Ky = { class: "row" }, Gy = ["disabled"], Jy = ["disabled"], Yy = { key: 1, class: "skill-bar panel", "aria-label": "\u53EF\u53D1\u52A8\u6280\u80FD" }, Qy = ["onClick"], Xy = { class: "seats", role: "list", "aria-label": "\u5EA7\u4F4D\u5217\u8868" }, Zy = ["onDragover", "onDragleave", "onDrop"], ek = { key: 2, class: "judgment-zone", "aria-label": "\u5224\u5B9A\u533A" }, nk = { class: "jz-entries" }, tk = ["title"], sk = { class: "jz-suit" }, ak = { class: "jz-player" }, ik = { key: 3, class: "discard-pile", "aria-label": "\u5F03\u724C\u5806" }, ok = { class: "muted discard-title" }, rk = { class: "discard-strip" }, lk = ["title"], uk = { class: "d-suit" }, ck = { key: 0, class: "muted discard-more" }, dk = De({ __name: "BattleBoard", props: { view: {}, selfId: {}, legalActions: {} }, emits: ["playCard", "activateSkill"], setup(e, { emit: n }) {
  const t = e, s = n, a = W(null), i = W(null), o = W(false), r = W([]), l = W(null), d = W(null);
  function c() {
    var _a2;
    return ((_a2 = t.view) == null ? void 0 : _a2.seats) ?? [];
  }
  function p() {
    return t.view ? t.view.known_cards.filter((N) => N.owner === t.selfId && N.zone === "Hand" && N.def != null) : [];
  }
  function m() {
    var _a2;
    const N = new Set(t.legalActions.flatMap((U) => typeof U == "object" && "ActivateSkill" in U ? [U.ActivateSkill.skill] : []));
    return ((_a2 = c().find((U) => U.player === t.selfId)) == null ? void 0 : _a2.skills.filter((U) => U.enabled && (U.active || U.view_as) && N.has(U.id))) ?? [];
  }
  function f(N) {
    switch (N) {
      case "Lord":
        return "\u4E3B\u516C";
      case "Loyalist":
        return "\u5FE0\u81E3";
      case "Rebel":
        return "\u53CD\u8D3C";
      case "Renegade":
        return "\u5185\u5978";
      default:
        return "\u8EAB\u4EFD\u672A\u77E5";
    }
  }
  function b() {
    return o.value ? new Set(H().flat()) : /* @__PURE__ */ new Set();
  }
  function P() {
    return t.view ? t.view.known_cards.filter((N) => N.zone === "DiscardPile" && N.def != null) : [];
  }
  function O(N) {
    return t.view ? t.view.known_cards.filter((U) => U.owner === N && U.zone === "Judge" && U.def != null) : [];
  }
  const w = Y(() => b()), _ = Y(() => i.value ? Array.from(new Set(T(i.value.id).flatMap((N) => N.card_targets))) : Array.from(new Set(A().map((N) => N.card))));
  function A(N) {
    return t.legalActions.flatMap((U) => typeof U != "object" || !("PlayCard" in U) ? [] : N != null && U.PlayCard.card !== N ? [] : [U.PlayCard]);
  }
  function T(N) {
    return t.legalActions.flatMap((U) => typeof U != "object" || !("ActivateSkill" in U) ? [] : U.ActivateSkill.skill === N ? [U.ActivateSkill] : []);
  }
  function H() {
    return i.value ? T(i.value.id).filter((N) => a.value == null ? N.card_targets.length === 0 : N.card_targets.length === 1 && N.card_targets[0] === a.value).map((N) => N.targets) : a.value == null ? [] : A(a.value).map((N) => N.targets);
  }
  function le(N, U) {
    return N.length === U.length && N.every((K, ie) => K === U[ie]);
  }
  const ne = Y(() => H().some((N) => le(N, r.value))), $e = Y(() => H().some((N) => N.length === 0));
  function Ee(N) {
    return t.view ? t.view.known_cards.filter((U) => U.owner === N && U.zone === "Equip" && U.def != null) : [];
  }
  function Se(N) {
    if (_.value.includes(N)) {
      if (a.value === N) {
        a.value = null, o.value = false, r.value = [];
        return;
      }
      a.value = N, i.value || (i.value = null), o.value = true, r.value = [];
    }
  }
  function Ye(N) {
    var _a2;
    if (((_a2 = i.value) == null ? void 0 : _a2.id) === N.id) {
      Oe();
      return;
    }
    i.value = N, a.value = null, o.value = true, r.value = [];
  }
  function He(N) {
    _.value.includes(N) && (l.value = N, a.value = N, o.value = true);
  }
  function Le(N) {
    if (!(!o.value || !w.value.has(N))) if (r.value.includes(N)) r.value = r.value.filter((U) => U !== N);
    else {
      const U = Math.max(0, ...H().map((K) => K.length));
      r.value = [...r.value, N].slice(0, U);
    }
  }
  function en(N, U) {
    l.value != null && w.value.has(U) && (N.preventDefault(), N.dataTransfer.dropEffect = "move", d.value = U);
  }
  function un(N) {
    d.value === N && (d.value = null);
  }
  function ke(N, U) {
    var _a2;
    N.preventDefault();
    const K = l.value;
    if (d.value = null, l.value = null, K != null && w.value.has(U)) {
      if ((_a2 = i.value) == null ? void 0 : _a2.view_as) {
        const ie = T(i.value.id).find((Ue) => le(Ue.targets, [U]) && Ue.card_targets.length === 1 && Ue.card_targets[0] === K);
        if (!ie) return;
        s("activateSkill", ie.skill, ie.targets, ie.card_targets);
      } else {
        const ie = A(K).find((Ue) => le(Ue.targets, [U]));
        if (!ie) return;
        s("playCard", ie.card, ie.targets);
      }
      a.value = null, i.value = null, o.value = false, r.value = [];
    }
  }
  function oe() {
    if (ne.value) {
      if (i.value) {
        const N = T(i.value.id).find((U) => le(U.targets, r.value) && (a.value == null ? U.card_targets.length === 0 : U.card_targets.length === 1 && U.card_targets[0] === a.value));
        if (!N) return;
        s("activateSkill", N.skill, N.targets, N.card_targets);
      } else {
        if (a.value == null) return;
        const N = A(a.value).find((U) => le(U.targets, r.value));
        if (!N) return;
        s("playCard", N.card, N.targets);
      }
      a.value = null, i.value = null, o.value = false, r.value = [];
    }
  }
  function ae() {
    r.value = [], oe();
  }
  function Oe() {
    a.value = null, i.value = null, o.value = false, r.value = [];
  }
  return (N, U) => {
    var _a2;
    return x(), j("div", Fy, [a.value != null || i.value ? (x(), j("div", Vy, [u("span", Hy, [U[0] || (U[0] = se(" \u25B8 ", -1)), i.value ? (x(), j(ee, { key: 0 }, [se("\u6280\u80FD #" + $(i.value.def), 1)], 64)) : (x(), j(ee, { key: 1 }, [se("\u624B\u724C #" + $(a.value), 1)], 64)), ((_a2 = i.value) == null ? void 0 : _a2.view_as) && a.value != null ? (x(), j("span", Wy, " \xB7 \u7D20\u6750 #" + $(a.value), 1)) : V("", true), o.value && r.value.length > 0 ? (x(), j("span", By, " \u2192 \u76EE\u6807 P" + $(r.value[0]), 1)) : V("", true)]), u("div", Ky, [u("button", { class: "primary", type: "button", onClick: oe, disabled: !ne.value, "aria-label": "\u786E\u8BA4\u884C\u52A8" }, $(i.value ? "\u53D1\u52A8\u6280\u80FD" : "\u786E\u8BA4\u51FA\u724C"), 9, Gy), u("button", { type: "button", disabled: !$e.value, onClick: ae, "aria-label": "\u65E0\u76EE\u6807\u76F4\u63A5\u6253\u51FA" }, $(i.value ? "\u65E0\u76EE\u6807\u53D1\u52A8" : "\u76F4\u63A5\u6253\u51FA"), 9, Jy), u("button", { type: "button", onClick: Oe, "aria-label": "\u53D6\u6D88\u9009\u62E9" }, " \u53D6\u6D88 ")])])) : V("", true), m().length ? (x(), j("section", Yy, [U[1] || (U[1] = u("span", { class: "muted" }, "\u6280\u80FD", -1)), (x(true), j(ee, null, ve(m(), (K) => {
      var _a3;
      return x(), j("button", { key: K.id, type: "button", class: Re({ primary: ((_a3 = i.value) == null ? void 0 : _a3.id) === K.id }), onClick: (ie) => Ye(K) }, " #" + $(K.def) + $(K.view_as ? " \xB7 \u89C6\u4E3A" : ""), 11, Qy);
    }), 128))])) : V("", true), u("div", Xy, [(x(true), j(ee, null, ve(c(), (K) => (x(), j("div", { key: K.player, class: "seat-wrapper", onDragover: (ie) => en(ie, K.player), onDragleave: (ie) => un(K.player), onDrop: (ie) => ke(ie, K.player) }, [_e(Ay, { seat: K, self: K.player === e.selfId, "identity-label": f(K.identity), "is-target-mode": o.value, "is-selectable-target": w.value.has(K.player), "is-drop-target": d.value === K.player, "equip-cards": Ee(K.player), onSelect: Le }, null, 8, ["seat", "self", "identity-label", "is-target-mode", "is-selectable-target", "is-drop-target", "equip-cards"])], 40, Zy))), 128))]), _e(Uy, { cards: p(), "actionable-ids": _.value, onSelect: Se, onDragStart: He }, null, 8, ["cards", "actionable-ids"]), c().some((K) => O(K.player).length > 0) ? (x(), j("section", ek, [U[2] || (U[2] = u("h3", { class: "muted jz-title" }, "\u2696 \u5224\u5B9A\u533A", -1)), u("div", nk, [(x(true), j(ee, null, ve(c(), (K) => (x(), j("div", { key: "jz-" + K.player, class: "jz-entry" }, [(x(true), j(ee, null, ve(O(K.player), (ie) => (x(), j("span", { key: ie.id, class: "jz-card", title: `P${K.player} #${ie.id}` }, [u("span", sk, $(ie.suit === "Spade" ? "\u2660" : ie.suit === "Heart" ? "\u2665" : ie.suit === "Club" ? "\u2663" : ie.suit === "Diamond" ? "\u2666" : "?"), 1), u("span", ak, "P" + $(K.player), 1)], 8, tk))), 128))]))), 128))])])) : V("", true), P().length > 0 ? (x(), j("section", ik, [u("h3", ok, "\u5F03\u724C\u5806\uFF08" + $(P().length) + "\uFF09", 1), u("div", rk, [(x(true), j(ee, null, ve(P().slice(-10), (K) => (x(), j("div", { key: K.id, class: "discard-card", title: `#${K.id} def:${K.def}` }, [u("span", uk, $(K.suit === "Spade" ? "\u2660" : K.suit === "Heart" ? "\u2665" : K.suit === "Club" ? "\u2663" : K.suit === "Diamond" ? "\u2666" : "?"), 1)], 8, lk))), 128)), P().length > 10 ? (x(), j("span", ck, "+" + $(P().length - 10), 1)) : V("", true)])])) : V("", true)]);
  };
} }), pk = ln(dk, [["__scopeId", "data-v-9182a2db"]]), hk = { class: "panel general-select", "aria-label": "\u9009\u5C06\u9762\u677F" }, fk = { class: "gs-head" }, mk = { class: "muted tiny" }, gk = ["disabled"], _k = { key: 0, class: "muted", role: "status" }, vk = { key: 1, class: "muted empty", role: "status" }, yk = { key: 2, class: "error", role: "alert" }, kk = { key: 3, class: "success", role: "status" }, wk = { class: "muted" }, bk = { class: "muted" }, xk = { key: 4, class: "gs-grid" }, $k = ["disabled", "aria-pressed", "aria-label", "onClick"], jk = { class: "candidate-name" }, Sk = { class: "candidate-skills" }, Ek = { class: "candidate-def muted" }, Ak = De({ __name: "GeneralSelectPanel", props: { state: {}, busy: { type: Boolean } }, emits: ["choose", "reload"], setup(e, { emit: n }) {
  const t = e, s = n, a = Y(() => t.busy || t.state.selectionStatus === "pending"), i = Y(() => {
    var _a2;
    return ((_a2 = t.state.assignment) == null ? void 0 : _a2.name) ?? null;
  }), o = Y(() => t.state.projection.skillNames.length > 0 ? t.state.projection.skillNames.join(" \xB7 ") : "\u89C6\u56FE\u672A\u66B4\u9732\u6280\u80FD\u540D"), r = Y(() => {
    switch (t.state.loadStatus) {
      case "loading":
        return "\u8BFB\u53D6\u5019\u9009\u2026";
      case "error":
        return "\u5019\u9009\u4E0D\u53EF\u7528";
      case "empty":
        return "\u5019\u9009\u4E3A\u7A7A";
      default:
        return `${t.state.candidates.length} \u9879\u5019\u9009`;
    }
  });
  return (l, d) => (x(), j("section", hk, [u("div", fk, [u("div", null, [d[1] || (d[1] = u("h2", null, "\u9009\u5C06", -1)), u("p", mk, " NC05 \xB7 \u76EE\u6807\u5EA7\u4F4D P" + $(e.state.seat) + " \xB7 " + $(r.value) + " \xB7 \u70B9\u51FB\u5019\u9009\u5373\u6309\u540D\u88C5\u914D \uFF08\u539F\u751F\u6B63\u5F0F\u5165\u53E3\uFF0C\u975E debug\uFF09 ", 1)]), u("button", { type: "button", disabled: a.value || e.state.loadStatus === "loading", "aria-label": "\u5237\u65B0\u9009\u5C06\u5019\u9009", onClick: d[0] || (d[0] = (c) => s("reload")) }, " \u5237\u65B0\u5019\u9009 ", 8, gk)]), e.state.loadStatus === "loading" && e.state.candidates.length === 0 ? (x(), j("p", _k, " \u6B63\u5728\u8BFB\u53D6\u539F\u751F\u76EE\u5F55\u5019\u9009\u2026 ")) : e.state.loadStatus === "empty" ? (x(), j("p", vk, " \u5019\u9009\u4E3A\u7A7A\uFF1A\u5185\u6838\u76EE\u5F55\u6CA1\u6709\u8FD4\u56DE\u4EFB\u4F55\u6B66\u5C06\uFF08\u8FD9\u662F\u5408\u6CD5\u7684\u7A7A\u76EE\u5F55\uFF0C\u4E0D\u662F\u52A0\u8F7D\u5931\u8D25\uFF09\u3002 ")) : V("", true), e.state.error ? (x(), j("p", yk, $(e.state.error), 1)) : V("", true), e.state.selectionStatus === "accepted" && e.state.assignment ? (x(), j("p", kk, [se(" \u2705 \u5DF2\u4E3A P" + $(e.state.assignment.seat) + " \u88C5\u914D ", 1), u("strong", null, $(e.state.assignment.name), 1), u("span", wk, " \xB7 state_hash " + $(e.state.assignment.state_hash.slice(0, 16)) + "\u2026 ", 1), d[2] || (d[2] = u("br", null, null, -1)), u("span", bk, "\u5237\u65B0\u540E\u89C6\u56FE\u6280\u80FD\uFF1A" + $(o.value), 1)])) : V("", true), e.state.candidates.length > 0 ? (x(), j("ul", xk, [(x(true), j(ee, null, ve(e.state.candidates, (c) => (x(), j("li", { key: c.def }, [u("button", { type: "button", class: Re(["candidate", { selected: i.value === c.name }]), disabled: a.value, "aria-pressed": i.value === c.name, "aria-label": `\u9009\u62E9 ${c.name}\uFF08${c.skillLabel}\uFF09`, onClick: (p) => s("choose", c.name) }, [u("span", jk, $(c.name), 1), u("span", Sk, $(c.skillLabel), 1), u("span", Ek, "#" + $(c.def), 1)], 10, $k)]))), 128))])) : V("", true)]));
} }), Ck = ln(Ak, [["__scopeId", "data-v-9fa84d6d"]]), Rk = { class: "battle stack" }, zk = { key: 0, class: "panel hotkey-overlay", role: "dialog", "aria-label": "\u5FEB\u6377\u952E" }, Ik = { class: "outcome-content" }, Pk = { class: "outcome-icon" }, Tk = { class: "outcome-label" }, Ok = { class: "muted" }, Nk = { key: 2, class: "panel turn-bar", role: "status", "aria-live": "polite" }, qk = { class: "turn-player" }, Mk = ["data-phase"], Dk = { class: "panel toolbar", "aria-label": "\u5BF9\u5C40\u63A7\u5236" }, Lk = { class: "row" }, Uk = ["disabled"], Fk = { key: 1, class: "online-badge" }, Vk = ["disabled"], Hk = ["disabled"], Wk = ["disabled"], Bk = ["disabled"], Kk = ["disabled"], Gk = ["disabled"], Jk = { class: "row muted controls" }, Yk = { class: "check" }, Qk = { class: "status", role: "status", "aria-live": "polite" }, Xk = { key: 0, class: "error", role: "alert" }, Zk = { key: 4, class: "panel online-roster", "aria-label": "\u8054\u673A\u623F\u95F4\u5EA7\u4F4D" }, ew = { class: "row online-roster-head" }, nw = { key: 0 }, tw = { class: "row online-seats" }, sw = { key: 0 }, aw = { key: 5, class: "panel choice", "aria-label": "\u5F85\u9009\u62E9" }, iw = { key: 0, role: "timer" }, ow = { class: "muted" }, rw = { key: 1, class: "row" }, lw = ["disabled", "onClick"], uw = { key: 2, class: "muted" }, cw = { key: 6, class: "panel roster", "aria-label": "\u73A9\u5BB6\u6982\u51B5" }, dw = { class: "roster-grid" }, pw = { class: "roster-player" }, hw = { class: "roster-hand muted" }, fw = ["data-id"], mw = { key: 1, class: "roster-id muted" }, gw = { class: "panel battle-section" }, _w = { key: 0, class: "effect-overlay", "aria-hidden": "true" }, vw = { class: "effect-icon" }, yw = { class: "effect-text" }, kw = { key: 0, class: "effect-player" }, ww = { key: 7, class: "panel match-stats", "aria-label": "\u5BF9\u5C40\u7EDF\u8BA1" }, bw = { class: "stats-grid" }, xw = { class: "stat" }, $w = { class: "stat-value mono" }, jw = { class: "stat" }, Sw = { class: "stat-value mono" }, Ew = { class: "stat" }, Aw = { class: "stat-value" }, Cw = { class: "stat" }, Rw = { class: "stat-value" }, zw = { key: 8, class: "panel seat-status", "aria-label": "\u5EA7\u4F4D\u72B6\u6001\u4E00\u89C8" }, Iw = { class: "seat-status-grid" }, Pw = { class: "ss-header" }, Tw = { class: "ss-player" }, Ow = { key: 0, class: "ss-tag" }, Nw = { class: "ss-hp-bar" }, qw = { class: "ss-hp-text" }, Mw = { class: "ss-icon" }, Dw = { class: "ss-meta muted" }, Lw = ["data-id"], Uw = { key: 1, class: "muted" }, Fw = { class: "panel log", "aria-label": "\u4E8B\u4EF6\u65E5\u5FD7" }, Vw = { class: "log-header" }, Hw = { class: "log-list" }, Ww = { class: "log-rev" }, Bw = { class: "log-text" }, Kw = { key: 0, class: "muted empty-log" }, Gw = De({ __name: "BattleView", setup(e) {
  var _a2;
  const n = up(), t = cl(), s = Fv(), a = new ii();
  let i;
  const o = new Tv(), r = W(1);
  try {
    ((_a2 = globalThis.localStorage) == null ? void 0 : _a2.getItem("noname-next-skip-anims")) === "1" && (s.skipAnims = true);
  } catch {
  }
  if (n.query.skipAnims === "1" && (s.skipAnims = true), n.query.speed !== void 0) {
    const v = Number(n.query.speed);
    Number.isFinite(v) && v > 0 && (s.speed = v);
  }
  const l = W(Number(n.query.players ?? 4) || 4), d = W(Number(n.query.seed ?? 42) || 42), c = W(null), p = W(false), m = W("\u672A\u8FDE\u63A5"), f = W(null), b = W(Date.now()), P = setInterval(() => {
    b.value = Date.now();
  }, 250), O = Y(() => {
    var _a3;
    return ((_a3 = s.choice) == null ? void 0 : _a3.deadline_unix_ms) == null ? null : Math.max(0, Math.ceil((s.choice.deadline_unix_ms - b.value) / 1e3));
  }), w = W(0), _ = W(false), A = Y(() => n.query.mode === "online"), T = Y(() => A.value && r.value === 0), H = W(Yv(r.value)), le = Y(() => !A.value && s.connected && a.isNative());
  async function ne(v) {
    s.busy = true, s.error = null;
    try {
      return await v();
    } catch (y) {
      s.error = y instanceof Error ? y.message : String(y), m.value = s.error;
      return;
    } finally {
      s.busy = false;
    }
  }
  function $e(v) {
    globalThis.__nativeStartMarks ?? (globalThis.__nativeStartMarks = {}), globalThis.__nativeStartMarks[v] = Math.round(performance.now() * 100) / 100;
  }
  async function Ee(v) {
    const y = performance.now();
    Lo(s, v);
    const E = performance.now();
    $e("t5_apply_envelope"), await Kv(s, 120);
    const S = performance.now();
    if (globalThis.__nativeEnvelopeTimings ?? (globalThis.__nativeEnvelopeTimings = []), globalThis.__nativeEnvelopeTimings.push({ apply_ms: Math.round((E - y) * 100) / 100, delay_ms: Math.round((S - E) * 100) / 100 }), A.value && (!s.connected || s.revision !== v.revision || s.hash !== v.hash_hex)) return;
    Wv(s);
    const M = A.value ? `\u623F\u95F4 #${w.value} \xB7 ` : "", G = (v.hash_hex || "").slice(0, 12);
    m.value = v.finished ? `${M}\u7EC8\u5C40 \xB7 ${I.value.label} \xB7 rev ${v.revision}` : `${M}rev ${v.revision}${G ? ` \xB7 ${G}\u2026` : ""}${a.isNative() ? " \xB7 native" : ""}`, Ye();
  }
  async function Se(v) {
    if (await Ee(v), !A.value && a.isNative()) {
      const y = Lv(v.result);
      if (y) throw new Error(y);
    }
  }
  function Ye() {
    H.value = Il(H.value, zl(s.view, H.value.seat));
  }
  async function He() {
    if (!(A.value || !a.isNative())) {
      H.value = Qv(H.value);
      try {
        const v = await a.generalCandidates();
        H.value = Xv(H.value, v);
      } catch (v) {
        H.value = Zv(H.value, v);
      }
    }
  }
  async function Le(v) {
    if (A.value || !a.isNative() || s.busy || H.value.selectionStatus === "pending") return;
    const y = H.value;
    await ne(async () => {
      if (H.value = await ny(y, v, { chooseGeneral: (E, S) => a.chooseGeneral(E, S), applyEnvelope: (E) => {
        Lo(s, E), Qe(s, `\u9009\u5C06 P${y.seat} \u2192 ${v}`);
      } }, (E) => {
        H.value = E;
      }), H.value.selectionStatus === "accepted") {
        const E = H.value.lastStateHash ?? "";
        m.value = `\u9009\u5C06 P${y.seat} \xB7 ${v}${E ? ` \xB7 ${E.slice(0, 12)}\u2026` : ""}`;
      } else H.value.selectionStatus === "rejected" && (m.value = H.value.error ?? "\u9009\u5C06\u5931\u8D25");
    });
  }
  async function en() {
    await ne(async () => {
      try {
        if (await lo(true), Mo(true), A.value) {
          const S = N();
          ph(S.wsUrl), await oe(S);
          return;
        }
        dh(), $e("t0_click"), await a.connect(), $e("t1_worker_ready"), s.connected = true, m.value = a.isNative() ? "\u539F\u751F Worker \u5C31\u7EEA\uFF0C\u5F00\u5C40\u4E2D\u2026" : "Worker \u5C31\u7EEA\uFF0C\u5F00\u5C40\u4E2D\u2026", a.isNative() && (r.value = Math.max(0, Math.min(r.value, l.value - 1)), H.value = Uo(H.value, r.value));
        const v = Ev();
        if (!a.isNative() && v.contentLock.length) throw new Error("\u5DF2\u5B89\u88C5\u6269\u5C55\u7684\u6B63\u5F0F\u5F00\u5C40\u9700\u8981\u539F\u751F\u540E\u7AEF");
        const y = await a.start(l.value, d.value, r.value, v.packsJson, n.query.mode === "identity" ? "identity" : void 0);
        i = v, $e("t2_start_done");
        const E = (a.isNative() ? He() : Promise.resolve()).then((S) => ($e("t3_candidates_loaded"), S));
        await Ee(y), await ss(), $e("t4_first_frame"), await E, Qe(s, `\u5F00\u5C40 ${l.value} \u4EBA seed=${d.value}${v.contentLock.length ? ` \xB7 \u5185\u5BB9\u9501 ${v.contentLock.length} \u5305` : ""}`);
      } catch (v) {
        throw s.connected = false, k(), v;
      }
    });
  }
  async function un() {
    await ne(async () => {
      if (A.value) throw new Error("\u8054\u673A\u6A21\u5F0F\u8BF7\u4F7F\u7528\u670D\u52A1\u7AEF\u5B58\u6863");
      if (!a.isNative()) throw new Error("\u5B58\u6863\u4EC5\u539F\u751F\u540E\u7AEF\u652F\u6301");
      if (!i) throw new Error("\u5C1A\u672A\u5EFA\u7ACB\u5BF9\u5C40\u5185\u5BB9\u9501");
      const v = await a.saveHex(), y = `save-${Date.now()}`;
      Sv({ id: y, label: `rev${s.revision} \xB7 ${l.value}p seed=${d.value}`, hex: v, seed: d.value, players: l.value, ...i, savedAt: (/* @__PURE__ */ new Date()).toISOString(), revision: s.revision }), Qe(s, `\u5DF2\u5B58\u6863 ${y}\uFF08${v.length / 2} \u5B57\u8282\uFF09`), m.value = `\u5DF2\u5B58\u6863 \xB7 ${y}`;
    });
  }
  async function ke() {
    await ne(async () => {
      if (A.value) throw new Error("\u8054\u673A\u6A21\u5F0F\u4E0D\u652F\u6301\u672C\u5730\u8BFB\u6863");
      if (!a.isNative()) throw new Error("\u8BFB\u6863\u4EC5\u539F\u751F\u540E\u7AEF\u652F\u6301");
      const y = Sl()[0];
      if (!y) throw new Error("\u65E0\u672C\u5730\u5B58\u6863");
      const E = Av(y), S = await a.restoreHex(y.hex, E.packsJson);
      i = E, l.value = y.players, d.value = y.seed, H.value = Uo(H.value, H.value.seat), await Ee(S), await He(), Qe(s, `\u8BFB\u6863 ${y.id} \xB7 rev ${y.revision}`);
    });
  }
  async function oe(v) {
    o.onEnvelope((E) => {
      Ee(E);
    }), o.onClose(() => {
      m.value = "\u8FDE\u63A5\u5DF2\u65AD\u5F00\uFF08\u670D\u52A1\u7AEF\u4E0D\u53EF\u8FBE\uFF09", s.connected = false, Qe(s, "\u8FDE\u63A5\u5DF2\u65AD\u5F00\uFF1A\u670D\u52A1\u7AEF\u4E0D\u53EF\u8FBE\u6216\u5DF2\u505C\u6B62");
    }), o.onReconnect((E) => {
      E === "reconnecting" ? (m.value = "\u8FDE\u63A5\u5DF2\u65AD\u5F00\uFF0C\u6B63\u5728\u91CD\u8FDE\u2026", s.connected = false) : E === "reconnected" ? (m.value = "\u5DF2\u91CD\u8FDE\uFF08\u7B49\u5F85\u623F\u95F4\u72B6\u6001\uFF09", s.connected = false, o.rejoinRoom().then((S) => (r.value = S.playerId, w.value = S.roomId, Oe(v, S), s.connected = true, m.value = `\u623F\u95F4 #${S.roomId} \xB7 \u5DF2\u91CD\u8FDE`, Ee(S.envelope))).catch((S) => {
        s.connected = false;
        const M = S instanceof Error ? S.message : String(S);
        m.value = `\u91CD\u8FDE\u540E\u56DE\u623F\u5931\u8D25\uFF1A${M}`, Qe(s, `\u91CD\u8FDE\u540E\u56DE\u623F\u5931\u8D25\uFF1A${M}`);
      })) : (m.value = "\u91CD\u8FDE\u5931\u8D25\uFF08\u670D\u52A1\u7AEF\u4E0D\u53EF\u8FBE\uFF09", s.connected = false);
    }), o.onRoster((E) => {
      f.value = E, w.value = E.room_id;
      const S = E.started && !_.value;
      _.value = E.started, m.value = E.started ? `\u623F\u95F4 #${E.room_id} \xB7 \u5DF2\u5F00\u5C40` : `\u623F\u95F4 #${E.room_id} \xB7 \u7B49\u5F85\u5165\u5EA7 ${E.seats.length}/${l.value}`, S && o.requestSnapshot().catch(() => {
      });
    }), m.value = `\u8FDE\u63A5 ${v.wsUrl}\u2026`, await o.connect(v.wsUrl, v.displayName);
    let y;
    if (v.action === "create") y = await o.createRoom(v.players, v.seed, v.password, v.choiceTimeoutSecs ?? 30);
    else if (v.action === "spectate") y = await o.spectate(v.roomId);
    else {
      const E = localStorage.getItem(Do(v.wsUrl, v.roomId));
      y = await o.joinRoom(v.roomId, v.password, E);
    }
    w.value = y.roomId, r.value = y.playerId, l.value = y.view.seats.length, Oe(v, y), s.connected = true, await Ee(y.envelope), Qe(s, y.role === "Spectator" ? `\u65C1\u89C2\u623F\u95F4 #${y.roomId}` : `\u52A0\u5165\u623F\u95F4 #${y.roomId} \xB7 \u5EA7\u4F4D ${y.seat + 1}`);
  }
  function ae() {
    o.cancelPendingCommands(), m.value = "\u5DF2\u53D6\u6D88\u5F85\u786E\u8BA4\u547D\u4EE4\uFF08\u670D\u52A1\u7AEF\u53EF\u80FD\u5DF2\u63D0\u4EA4\uFF0C\u4EE5\u6743\u5A01\u72B6\u6001\u4E3A\u51C6\uFF09", Qe(s, "\u5DF2\u53D6\u6D88\u5F85\u786E\u8BA4\u547D\u4EE4\uFF1B\u7ED3\u679C\u53EF\u80FD\u5DF2\u63D0\u4EA4\uFF0C\u7B49\u5F85/\u5237\u65B0\u540E\u4EE5\u670D\u52A1\u7AEF\u72B6\u6001\u4E3A\u51C6");
  }
  function Oe(v, y) {
    y.role === "Player" && y.reconnectToken && localStorage.setItem(Do(v.wsUrl, y.roomId), y.reconnectToken), sessionStorage.setItem("noname-next.online-intent", JSON.stringify({ ...v, action: y.role === "Spectator" ? "spectate" : "join", roomId: y.roomId }));
  }
  function N() {
    const v = sessionStorage.getItem("noname-next.online-intent");
    if (v) try {
      return JSON.parse(v);
    } catch {
      sessionStorage.removeItem("noname-next.online-intent");
    }
    const y = window.location.protocol === "https:";
    return { action: String(n.query.action ?? "join"), wsUrl: `${y ? "wss" : "ws"}://${window.location.hostname || "127.0.0.1"}:7700/ws`, displayName: "player", password: "", roomId: Number(n.query.room ?? 1) || 1, players: l.value, seed: d.value };
  }
  function U() {
    return A.value ? o : a;
  }
  async function K() {
    await ne(async () => {
      const v = await U().endPhase(r.value);
      await Se(v);
    });
  }
  async function ie() {
    await ne(async () => {
      if (A.value) throw new Error("\u8054\u673A\u6A21\u5F0F\u4E0D\u63D0\u4F9B\u5BA2\u6237\u7AEF\u81EA\u52A8\u6B65\u8FDB");
      const y = await a.autoStep();
      return await Se(y), true;
    }) || B();
  }
  async function Ue(v, y) {
    await ne(async () => {
      const E = await U().playCard(r.value, v, y);
      await Se(E), Qe(s, `\u51FA\u724C #${v}${y.length > 0 ? ` \u2192 P${y[0]}` : ""}`);
    });
  }
  async function Sn(v, y, E) {
    await ne(async () => {
      const S = await U().activateSkill(r.value, v, y, E);
      await Se(S), Qe(s, `\u53D1\u52A8\u6280\u80FD #${v}${y.length > 0 ? ` \u2192 P${y[0]}` : ""}`);
    });
  }
  async function Fe(v) {
    await ne(async () => {
      const y = await U().resolveChoice(r.value, v.ResolveChoice.choice_id, v.ResolveChoice.selections, v.ResolveChoice.cancelled);
      await Se(y);
    });
  }
  const z = Y(() => s.legalActions.filter((v) => typeof v == "object" && "ResolveChoice" in v)), F = Y(() => s.connected && s.legalActions.includes("EndPhase"));
  function L() {
    if (p.value) {
      B();
      return;
    }
    p.value = true;
    const v = async () => {
      if (!p.value || s.finished) {
        B();
        return;
      }
      if (await ie(), p.value && !s.finished) {
        const y = s.skipAnims || s.reduceMotion ? 30 : 280 / s.speed;
        c.value = window.setTimeout(v, y);
      } else B();
    };
    v();
  }
  function B() {
    p.value = false, c.value != null && (clearTimeout(c.value), c.value = null);
  }
  function te(v) {
    return v.includes("\u4F24\u5BB3") || v.includes("\u9635\u4EA1") ? "log-damage" : v.includes("\u56DE\u590D") || v.includes("\u6478\u724C") ? "log-positive" : v.includes("\u6FD2\u6B7B") || v.includes("\u7B49\u5F85") ? "log-warning" : v.includes("\u56DE\u5408") || v.includes("\u4F7F\u7528") ? "log-action" : "";
  }
  const h = W(false);
  function g(v) {
    if (v.key === "?") {
      h.value = !h.value;
      return;
    }
    if (v.key === "Escape") {
      h.value = false;
      return;
    }
    s.busy || s.finished || !s.connected || (v.key === " " ? (v.preventDefault(), A.value || ie()) : v.key === "e" || v.key === "E" ? F.value && K() : (v.key === "s" || v.key === "S") && (s.skipAnims = !s.skipAnims));
  }
  function k() {
    B(), a.dispose(), o.dispose(), i = void 0, lo(false), Mo(false);
  }
  $t(() => {
    window.addEventListener("keydown", g), window.addEventListener("pagehide", k), en().catch((v) => {
      const y = v instanceof Error ? v.message : String(v);
      m.value = `\u8FDE\u63A5\u5931\u8D25\uFF1A${y}`, Qe(s, `\u8FDE\u63A5\u5931\u8D25\uFF1A${y}`), s.connected = false;
    });
  }), as(() => {
    clearInterval(P), window.removeEventListener("keydown", g), window.removeEventListener("pagehide", k), k();
  }), kn(() => s.speed, () => {
  });
  const R = { Prepare: "\u51C6\u5907", Draw: "\u6478\u724C", Play: "\u51FA\u724C", Discard: "\u5F03\u724C", Judge: "\u5224\u5B9A", End: "\u7ED3\u675F" }, I = Y(() => Dv(s.view, s.lastEvents ?? [], r.value));
  function C() {
    t.push("/");
  }
  const q = Y(() => {
    var _a3;
    if (s.view && s.view.phase !== "Idle" && (s.view.current_player !== 0 || s.view.seats.some((y) => y.player === 0))) return { currentPlayer: s.view.current_player, phase: s.view.phase, phaseLabel: R[s.view.phase] ?? s.view.phase, round: s.view.round };
    const v = s.lastEvents ?? [];
    for (const y of v) {
      const E = y;
      if ((E == null ? void 0 : E.type) === "TurnStateChanged") return { currentPlayer: E.current ?? 0, phase: E.phase ?? "?", phaseLabel: R[E.phase ?? ""] ?? E.phase ?? "?", round: ((_a3 = s.view) == null ? void 0 : _a3.round) ?? 0 };
    }
    return null;
  });
  return (v, y) => {
    const E = Du("auto-scroll");
    return x(), j("div", Rk, [h.value ? (x(), j("section", zk, [...y[4] || (y[4] = [Hr('<h2 data-v-b7aec031>\u2328 \u5FEB\u6377\u952E</h2><dl class="hotkey-list" data-v-b7aec031><dt data-v-b7aec031><kbd data-v-b7aec031>Space</kbd></dt><dd data-v-b7aec031>\u81EA\u52A8\u4E00\u6B65</dd><dt data-v-b7aec031><kbd data-v-b7aec031>E</kbd></dt><dd data-v-b7aec031>\u7ED3\u675F\u9636\u6BB5</dd><dt data-v-b7aec031><kbd data-v-b7aec031>S</kbd></dt><dd data-v-b7aec031>\u5207\u6362\u8DF3\u8FC7\u52A8\u753B</dd><dt data-v-b7aec031><kbd data-v-b7aec031>?</kbd></dt><dd data-v-b7aec031>\u663E\u793A/\u9690\u85CF\u6B64\u9762\u677F</dd><dt data-v-b7aec031><kbd data-v-b7aec031>Esc</kbd></dt><dd data-v-b7aec031>\u5173\u95ED\u6B64\u9762\u677F</dd></dl>', 2)])])) : V("", true), D(s).finished && I.value ? (x(), j("section", { key: 1, class: Re(["panel outcome-banner", { win: I.value.isSelfWin === true, lose: I.value.isSelfWin === false }]), role: "alert", "aria-live": "assertive" }, [u("div", Ik, [u("span", Pk, $(I.value.isSelfWin === null ? "\u2691" : I.value.isSelfWin ? "\u{1F389}" : "\u{1F480}"), 1), u("div", null, [u("h2", Tk, $(I.value.label), 1), u("p", Ok, "rev " + $(D(s).revision), 1)])]), u("button", { class: "primary", type: "button", onClick: y[0] || (y[0] = (S) => A.value ? C() : en()) }, $(A.value ? "\u8FD4\u56DE\u5927\u5385" : "\u518D\u6765\u4E00\u5C40"), 1)], 2)) : V("", true), q.value ? (x(), j("section", Nk, [u("span", qk, "\u25BA \u7B2C " + $(q.value.round) + " \u8F6E \xB7 P" + $(q.value.currentPlayer), 1), u("span", { class: "phase-badge", "data-phase": q.value.phase }, $(q.value.phaseLabel), 9, Mk)])) : V("", true), u("section", Dk, [u("div", Lk, [A.value ? (x(), j("span", Fk, "\u8054\u673A\u623F\u95F4 #" + $(w.value || "\u2026"), 1)) : (x(), j("button", { key: 0, class: "primary", type: "button", disabled: D(s).busy, onClick: en }, " \u91CD\u65B0\u5F00\u5C40 ", 8, Uk)), u("button", { type: "button", disabled: D(s).busy || D(s).finished || T.value || A.value && !_.value || !F.value, onClick: K }, " \u7ED3\u675F\u9636\u6BB5 ", 8, Vk), A.value ? V("", true) : (x(), j("button", { key: 2, type: "button", disabled: D(s).busy || D(s).finished, onClick: ie }, " \u81EA\u52A8\u4E00\u6B65 ", 8, Hk)), A.value ? V("", true) : (x(), j("button", { key: 3, type: "button", disabled: D(s).busy, onClick: L }, $(p.value ? "\u505C\u6B62\u81EA\u52A8" : "\u8FDE\u7EED\u81EA\u52A8"), 9, Wk)), A.value ? (x(), j("button", { key: 4, type: "button", disabled: !D(s).busy, onClick: ae }, " \u53D6\u6D88\u5F85\u786E\u8BA4 ", 8, Bk)) : V("", true), A.value ? V("", true) : (x(), j("button", { key: 5, type: "button", disabled: D(s).busy || !D(s).connected, onClick: un }, " \u5B58\u6863 ", 8, Kk)), A.value ? V("", true) : (x(), j("button", { key: 6, type: "button", disabled: D(s).busy || !D(s).connected, onClick: ke }, " \u8BFB\u6863 ", 8, Gk))]), u("div", Jk, [u("label", null, [y[6] || (y[6] = se(" \u500D\u901F ", -1)), ye(u("select", { "onUpdate:modelValue": y[1] || (y[1] = (S) => D(s).speed = S), "aria-label": "\u64AD\u653E\u500D\u901F" }, [...y[5] || (y[5] = [u("option", { value: 0.5 }, "0.5\xD7", -1), u("option", { value: 1 }, "1\xD7", -1), u("option", { value: 2 }, "2\xD7", -1), u("option", { value: 4 }, "4\xD7", -1)])], 512), [[vn, D(s).speed, void 0, { number: true }]])]), u("label", Yk, [ye(u("input", { "onUpdate:modelValue": y[2] || (y[2] = (S) => D(s).skipAnims = S), type: "checkbox" }, null, 512), [[ks, D(s).skipAnims]]), y[7] || (y[7] = se(" \u8DF3\u8FC7\u52A8\u753B ", -1))]), u("span", null, $(T.value ? "\u516C\u5171\u65C1\u89C2\u6295\u5F71" : `\u89C2\u5BDF\u8005 P${r.value}`), 1)]), u("p", Qk, $(m.value), 1), D(s).error ? (x(), j("p", Xk, $(D(s).error), 1)) : V("", true)]), le.value ? (x(), ti(Ck, { key: 3, state: H.value, busy: D(s).busy, onChoose: Le, onReload: He }, null, 8, ["state", "busy"])) : V("", true), A.value && f.value ? (x(), j("section", Zk, [u("div", ew, [u("h2", null, "\u623F\u95F4 #" + $(f.value.room_id), 1), f.value.choice_timeout_secs ? (x(), j("span", nw, "\u9009\u62E9\u65F6\u9650 " + $(f.value.choice_timeout_secs) + " \u79D2", 1)) : V("", true), u("span", { class: Re(f.value.started ? "started" : "waiting") }, $(f.value.started ? "\u5BF9\u5C40\u8FDB\u884C\u4E2D" : "\u7B49\u5F85\u73A9\u5BB6"), 3)]), u("div", tw, [(x(true), j(ee, null, ve(f.value.seats, (S) => (x(), j("span", { key: S.seat, class: "online-seat" }, [se($(S.seat + 1) + " \xB7 " + $(S.display_name) + " ", 1), S.managed ? (x(), j("strong", sw, "\u6258\u7BA1\u4E2D")) : V("", true), u("i", { class: Re(S.connected ? "connected" : "offline") }, $(S.connected ? "\u5728\u7EBF" : "\u79BB\u7EBF"), 3)]))), 128))])])) : V("", true), D(s).choice ? (x(), j("section", aw, [u("h2", null, $(D(Np)(D(s).choice)), 1), O.value !== null ? (x(), j("p", iw, "\u5269\u4F59 " + $(O.value) + " \u79D2 \xB7 \u5230\u65F6\u6258\u7BA1", 1)) : V("", true), u("p", ow, " choice #" + $(D(s).choice.choice_id) + " \xB7 \u73A9\u5BB6 " + $(D(s).choice.observer) + " \xB7 " + $(D(Ra)(D(s).choice)), 1), z.value.length ? (x(), j("div", rw, [(x(true), j(ee, null, ve(z.value, (S, M) => (x(), j("button", { key: M, type: "button", disabled: D(s).busy || !D(s).connected || T.value, onClick: (G) => Fe(S) }, $(S.ResolveChoice.cancelled ? "\u53D6\u6D88" : `\u9009\u62E9 ${S.ResolveChoice.selections.join(", ") || "\u7A7A"}`), 9, lw))), 128))])) : (x(), j("p", uw, "\u5F53\u524D\u89C2\u5BDF\u8005\u65E0\u6743\u5904\u7406\u6B64\u9009\u62E9\u3002"))])) : V("", true), D(s).view ? (x(), j("section", cw, [y[8] || (y[8] = u("h2", null, "\u73A9\u5BB6\u6982\u51B5", -1)), u("div", dw, [(x(true), j(ee, null, ve(D(s).view.seats, (S) => (x(), j("div", { key: S.player, class: Re(["roster-entry", { dead: !S.alive, self: S.player === r.value }]) }, [u("span", pw, "P" + $(S.player) + $(S.turned_over ? " \xB7 \u7FFB\u9762" : ""), 1), u("span", { class: "roster-hp", style: Kn({ color: S.alive ? S.hp <= 0 ? "var(--color-warning)" : "inherit" : "var(--color-negative)" }) }, $(S.alive ? S.hp <= 0 ? "\u26A0" : "" : "\u2717") + " " + $(S.hp) + "/" + $(S.max_hp), 5), u("span", hw, $(S.hand_count) + "\u{1F0A0}", 1), S.identity ? (x(), j("span", { key: 0, class: "roster-id", "data-id": S.identity }, $(S.identity === "Lord" ? "\u4E3B" : S.identity === "Loyalist" ? "\u5FE0" : S.identity === "Rebel" ? "\u53CD" : "\u5978"), 9, fw)) : (x(), j("span", mw, "?"))], 2))), 128))])])) : V("", true), u("section", gw, [_e(pk, { view: D(s).view, "self-id": r.value, "legal-actions": D(s).connected && !D(s).busy ? D(s).legalActions : [], onPlayCard: Ue, onActivateSkill: Sn }, null, 8, ["view", "self-id", "legal-actions"]), D(s).effects.length ? (x(), j("div", _w, [(x(true), j(ee, null, ve(D(s).effects, (S) => (x(), j("div", { key: S.id, class: Re(["effect-popup", S.type]) }, [u("span", vw, $(S.type === "damage" ? "\u2694" : S.type === "heal" ? "\u271A" : S.type === "draw" ? "\u{1F0A0}" : S.type === "play" ? "\u25B8" : S.type === "dying" ? "\u26A0" : "\u2717"), 1), u("span", yw, $(S.text), 1), S.player != null ? (x(), j("span", kw, "P" + $(S.player), 1)) : V("", true)], 2))), 128))])) : V("", true)]), D(s).view ? (x(), j("section", ww, [u("div", bw, [u("div", xw, [y[9] || (y[9] = u("span", { class: "stat-label muted" }, "Revision", -1)), u("span", $w, $(D(s).revision), 1)]), u("div", jw, [y[10] || (y[10] = u("span", { class: "stat-label muted" }, "State Hash", -1)), u("span", Sw, $(D(s).hash.slice(0, 16)) + "\u2026", 1)]), u("div", Ew, [y[11] || (y[11] = u("span", { class: "stat-label muted" }, "\u5DF2\u77E5\u5361\u724C", -1)), u("span", Aw, $(D(s).view.known_cards.length), 1)]), u("div", Cw, [y[12] || (y[12] = u("span", { class: "stat-label muted" }, "\u5B58\u6D3B\u73A9\u5BB6", -1)), u("span", Rw, $(D(s).view.seats.filter((S) => S.alive).length) + " / " + $(D(s).view.seats.length), 1)])])])) : V("", true), D(s).view ? (x(), j("section", zw, [y[13] || (y[13] = u("h2", null, "\u5EA7\u4F4D\u72B6\u6001", -1)), u("div", Iw, [(x(true), j(ee, null, ve(D(s).view.seats, (S) => (x(), j("div", { key: "ss-" + S.player, class: Re(["ss-entry", { "ss-dead": !S.alive, "ss-self": S.player === r.value }]) }, [u("div", Pw, [u("span", Tw, "P" + $(S.player), 1), S.player === r.value ? (x(), j("span", Ow, "\u4F60")) : V("", true)]), u("div", Nw, [u("div", { class: "ss-hp-fill", style: Kn({ width: Math.max(Math.min(S.hp / S.max_hp * 100, 100), 0) + "%", background: S.alive ? S.hp <= 0 ? "var(--color-warning)" : "var(--color-positive)" : "var(--color-negative)" }) }, null, 4)]), u("div", qw, [u("span", Mw, $(S.alive ? S.hp <= 0 ? "\u26A0" : "" : "\u2717"), 1), se(" " + $(Math.max(S.hp, 0)) + "/" + $(S.max_hp), 1)]), u("div", Dw, [u("span", null, $(S.hand_count) + "\u{1F0A0}", 1), S.identity ? (x(), j("span", { key: 0, class: "ss-id", "data-id": S.identity }, $(S.identity === "Lord" ? "\u4E3B" : S.identity === "Loyalist" ? "\u5FE0" : S.identity === "Rebel" ? "\u53CD" : "\u5978"), 9, Lw)) : (x(), j("span", Uw, "?"))])], 2))), 128))])])) : V("", true), u("section", Fw, [u("div", Vw, [y[14] || (y[14] = u("h2", null, "\u6218\u6597\u65E5\u5FD7", -1)), D(s).log.length > 0 ? (x(), j("button", { key: 0, type: "button", class: "clear-btn", onClick: y[3] || (y[3] = (S) => D(s).log = []), "aria-label": "\u6E05\u7A7A\u65E5\u5FD7" }, "\u6E05\u7A7A")) : V("", true)]), y[15] || (y[15] = u("p", { class: "muted tiny" }, " \u65E5\u5FD7\u6765\u81EA DomainEvent \u6458\u8981\uFF0C\u4E0D\u662F\u72B6\u6001\u771F\u76F8\u6E90\u3002\u771F\u76F8\u6E90\u4E3A Command Log + \u6295\u5F71\uFF08\xA722\uFF09\u3002 ", -1)), ye((x(), j("ol", Hw, [(x(true), j(ee, null, ve(D(s).log.slice().reverse(), (S) => (x(), j("li", { key: S.id, class: Re(["log-entry", te(S.text)]) }, [u("span", Ww, "rev" + $(S.at), 1), u("span", Bw, $(S.text), 1)], 2))), 128))])), [[E]]), D(s).log.length === 0 ? (x(), j("p", Kw, "\u6682\u65E0\u4E8B\u4EF6")) : V("", true)])]);
  };
} }), Jw = ln(Gw, [["__scopeId", "data-v-b7aec031"]]), Yw = { class: "a11y-group" }, Qw = { class: "row" }, Xw = ["disabled"], Zw = ["disabled"], eb = { class: "legend-list", "aria-live": "polite" }, nb = ["disabled", "onClick"], tb = ["disabled", "onClick"], sb = ["disabled", "onClick"], ab = { key: 0, class: "muted" }, ib = { class: "status", role: "status" }, Fo = "noname-next.native-server", ob = De({ __name: "NativePackagesPanel", setup(e) {
  const n = W(typeof localStorage < "u" && localStorage.getItem(Fo) || "http://127.0.0.1:7700"), t = W([]), s = W(false), a = W("");
  function i() {
    try {
      localStorage.setItem(Fo, n.value);
    } catch {
    }
  }
  async function o(c, p) {
    const m = await fetch(new URL(c, n.value), { ...p, headers: { "content-type": "application/json", ...(p == null ? void 0 : p.headers) ?? {} } });
    if (!m.ok) throw new Error(await m.text() || `HTTP ${m.status}`);
    return m;
  }
  async function r() {
    s.value = true;
    try {
      const c = await o("/native/packages");
      t.value = await c.json(), a.value = `\u5DF2\u8F7D\u5165 ${t.value.length} \u4E2A\u539F\u751F\u5305`;
    } catch (c) {
      a.value = `\u8BFB\u53D6\u5931\u8D25\uFF1A${c instanceof Error ? c.message : String(c)}`;
    } finally {
      s.value = false;
    }
  }
  async function l() {
    s.value = true;
    try {
      await o("/native/packages/import", { method: "POST", body: "{}" }), await r(), a.value = "\u5B89\u88C5\u626B\u63CF\u5B8C\u6210";
    } catch (c) {
      a.value = `\u5B89\u88C5\u5931\u8D25\uFF1A${c instanceof Error ? c.message : String(c)}`;
    } finally {
      s.value = false;
    }
  }
  async function d(c, p) {
    if (!(c === "uninstall" && !window.confirm(`\u5378\u8F7D ${p.package_id} \u7684\u6240\u6709\u7248\u672C\uFF1F`))) {
      s.value = true;
      try {
        await o(`/native/packages/${c}`, { method: "POST", body: JSON.stringify({ namespace: p.namespace, package_id: p.package_id, version: p.version }) }), await r(), a.value = `${c} \u5B8C\u6210\uFF1A${p.package_id}`;
      } catch (m) {
        a.value = `${c} \u5931\u8D25\uFF1A${m instanceof Error ? m.message : String(m)}`;
      } finally {
        s.value = false;
      }
    }
  }
  return $t(r), (c, p) => (x(), j("fieldset", Yw, [p[2] || (p[2] = u("legend", null, "\u539F\u751F\u6269\u5C55\u5305", -1)), p[3] || (p[3] = u("p", { class: "muted", role: "note" }, " \u901A\u8FC7\u670D\u52A1\u7AEF\u6B63\u5F0F\u5165\u53E3\u5B89\u88C5\u3001\u542F\u505C\u3001\u5347\u7EA7\u6216\u5378\u8F7D\u539F\u751F\u5305\uFF1B\u5BA2\u6237\u7AEF\u65E0\u9700\u91CD\u65B0\u6784\u5EFA\u3002 ", -1)), u("div", Qw, [u("label", null, [p[1] || (p[1] = se(" \u670D\u52A1\u7AEF ", -1)), ye(u("input", { "onUpdate:modelValue": p[0] || (p[0] = (m) => n.value = m), type: "url", "aria-label": "\u539F\u751F\u670D\u52A1\u7AEF\u5730\u5740", onChange: i }, null, 544), [[Wn, n.value]])]), u("button", { type: "button", disabled: s.value, onClick: r }, "\u5237\u65B0", 8, Xw), u("button", { type: "button", disabled: s.value, onClick: l }, "\u626B\u63CF\u5B89\u88C5", 8, Zw)]), u("ul", eb, [(x(true), j(ee, null, ve(t.value, (m) => (x(), j("li", { key: `${m.namespace}/${m.package_id}@${m.version}` }, [u("code", null, $(m.namespace) + "/" + $(m.package_id) + "@" + $(m.version), 1), u("button", { type: "button", disabled: s.value, onClick: (f) => d("activate", m) }, "\u6FC0\u6D3B", 8, nb), u("button", { type: "button", disabled: s.value, onClick: (f) => d("disable", m) }, "\u505C\u7528", 8, tb), u("button", { type: "button", disabled: s.value, onClick: (f) => d("uninstall", m) }, "\u5378\u8F7D", 8, sb)]))), 128))]), t.value.length === 0 ? (x(), j("p", ab, "\u5C1A\u672A\u8BFB\u53D6\u5230\u5DF2\u5B89\u88C5\u5305\u3002")) : V("", true), u("p", ib, $(a.value), 1)]));
} }), rb = { class: "panel stack", "aria-labelledby": "settings-title" }, lb = { key: 0, class: "a11y-group" }, ub = { class: "select-row" }, cb = { class: "a11y-group" }, db = { class: "check" }, pb = { class: "check" }, hb = { class: "check" }, fb = { class: "select-row" }, mb = { class: "select-row" }, gb = { class: "select-row" }, Vo = "noname-next-a11y", _b = De({ __name: "SettingsView", setup(e) {
  const n = jn(), t = W(fl());
  kn(t, (m) => localStorage.setItem(hl, m));
  const s = W(typeof window < "u" && window.matchMedia("(prefers-reduced-motion: reduce)").matches), a = W(false), i = W(false), o = W("normal"), r = W("none"), l = W("default");
  function d() {
    if (typeof window > "u") return null;
    try {
      const m = localStorage.getItem(Vo);
      return m ? JSON.parse(m) : null;
    } catch {
      return null;
    }
  }
  function c() {
    if (typeof window > "u") return;
    const m = { highContrast: a.value, skipAnimations: i.value, fontScale: o.value, colorBlind: r.value, screenReader: l.value };
    localStorage.setItem(Vo, JSON.stringify(m));
  }
  function p() {
    const m = document.documentElement;
    m.dataset.contrast = a.value ? "high" : "", m.dataset.skipAnimations = i.value ? "true" : "false", m.dataset.fontScale = o.value, m.dataset.colorBlind = r.value !== "none" ? r.value : "";
  }
  return $t(() => {
    const m = d();
    m && (a.value = m.highContrast, i.value = m.skipAnimations, o.value = m.fontScale || "normal", r.value = m.colorBlind || "none", l.value = m.screenReader || "default"), p();
  }), kn([a, i, o, r, l], () => {
    p(), c();
  }), (m, f) => (x(), j("section", rb, [f[23] || (f[23] = u("h1", { id: "settings-title" }, "\u8BBE\u7F6E", -1)), f[24] || (f[24] = u("p", { class: "muted", role: "note" }, " \u8868\u73B0\u5C42\u8BBE\u7F6E\u4E0D\u5F71\u54CD\u89C4\u5219\u6838\u5FC3\uFF08\xA730 \u53EF\u8BBF\u95EE\u6027\uFF09\u3002\u6240\u6709\u8BBE\u7F6E\u5373\u65F6\u751F\u6548\u5E76\u4FDD\u5B58\u5230\u672C\u5730\u3002 ", -1)), D(n) ? (x(), j("fieldset", lb, [f[9] || (f[9] = u("legend", null, "\u684C\u9762\u5F15\u64CE", -1)), u("label", ub, [f[8] || (f[8] = u("span", null, "\u4E0B\u6B21\u8FDE\u63A5\u4F7F\u7528", -1)), ye(u("select", { "onUpdate:modelValue": f[0] || (f[0] = (b) => t.value = b), "aria-label": "\u684C\u9762\u5F15\u64CE" }, [...f[7] || (f[7] = [u("option", { value: "wasm" }, "WASM Worker\uFF08\u9ED8\u8BA4 / \u56DE\u9000\uFF09", -1), u("option", { value: "native" }, "\u539F\u751F\u5F15\u64CE\uFF08\u5B9E\u9A8C\u539F\u578B\uFF09", -1)])], 512), [[vn, t.value]])]), f[10] || (f[10] = u("p", { class: "muted" }, "\u5F53\u524D\u5BF9\u5C40\u4E0D\u5207\u6362\u3002\u64CD\u4F5C\u5931\u8D25\u65F6\u4FDD\u7559\u5F53\u524D\u5F15\u64CE\uFF0C\u8BF7\u5148\u6838\u5BF9\u5C40\u9762\u3002", -1))])) : V("", true), u("fieldset", cb, [f[20] || (f[20] = u("legend", null, "\u53EF\u8BBF\u95EE\u6027\uFF08\xA730\uFF09", -1)), u("label", db, [ye(u("input", { "onUpdate:modelValue": f[1] || (f[1] = (b) => s.value = b), type: "checkbox", disabled: "", "aria-describedby": "reduce-motion-desc" }, null, 512), [[ks, s.value]]), f[11] || (f[11] = u("span", null, "\u51CF\u5C11\u52A8\u6001\u6548\u679C\uFF08\u7CFB\u7EDF\u504F\u597D\uFF0C\u53EA\u8BFB\uFF09", -1))]), f[21] || (f[21] = u("p", { id: "reduce-motion-desc", class: "sr-only" }, " \u9075\u5FAA\u7CFB\u7EDF prefers-reduced-motion \u8BBE\u7F6E\u3002 ", -1)), u("label", pb, [ye(u("input", { "onUpdate:modelValue": f[2] || (f[2] = (b) => i.value = b), type: "checkbox", "aria-describedby": "skip-anim-desc" }, null, 512), [[ks, i.value]]), f[12] || (f[12] = u("span", null, "\u8DF3\u8FC7\u52A8\u753B\uFF08\u8DF3\u8FC7\u540E\u4ECD\u53EF\u7406\u89E3\u89C4\u5219\u7ED3\u679C\uFF09", -1))]), f[22] || (f[22] = u("p", { id: "skip-anim-desc", class: "sr-only" }, " \u8DF3\u8FC7\u6240\u6709\u52A8\u753B\uFF1B\u89C4\u5219\u7ED3\u679C\u4ECD\u901A\u8FC7\u6587\u5B57\u65E5\u5FD7\u548C\u56FE\u6807\u5B8C\u6574\u4F20\u8FBE\u3002 ", -1)), u("label", hb, [ye(u("input", { "onUpdate:modelValue": f[3] || (f[3] = (b) => a.value = b), type: "checkbox" }, null, 512), [[ks, a.value]]), f[13] || (f[13] = u("span", null, "\u9AD8\u5BF9\u6BD4\u5EA6\u6A21\u5F0F", -1))]), u("label", fb, [f[15] || (f[15] = u("span", null, "\u5B57\u53F7\u7F29\u653E", -1)), ye(u("select", { "onUpdate:modelValue": f[4] || (f[4] = (b) => o.value = b), "aria-label": "\u5B57\u53F7\u7F29\u653E" }, [...f[14] || (f[14] = [u("option", { value: "small" }, "\u5C0F (85%)", -1), u("option", { value: "normal" }, "\u6807\u51C6 (100%)", -1), u("option", { value: "large" }, "\u5927 (125%)", -1), u("option", { value: "xlarge" }, "\u7279\u5927 (150%)", -1)])], 512), [[vn, o.value]])]), u("label", mb, [f[17] || (f[17] = u("span", null, "\u8272\u89C9\u53CB\u597D\u6A21\u5F0F", -1)), ye(u("select", { "onUpdate:modelValue": f[5] || (f[5] = (b) => r.value = b), "aria-label": "\u8272\u89C9\u53CB\u597D\u6A21\u5F0F" }, [...f[16] || (f[16] = [Hr('<option value="none" data-v-637706ba>\u5173\u95ED</option><option value="protanopia" data-v-637706ba>\u7EA2\u8272\u76F2\uFF08Protanopia\uFF09</option><option value="deuteranopia" data-v-637706ba>\u7EFF\u8272\u76F2\uFF08Deuteranopia\uFF09</option><option value="tritanopia" data-v-637706ba>\u84DD\u8272\u76F2\uFF08Tritanopia\uFF09</option><option value="achromatopsia" data-v-637706ba>\u5168\u8272\u76F2\uFF08Achromatopsia\uFF09</option>', 5)])], 512), [[vn, r.value]])]), u("label", gb, [f[19] || (f[19] = u("span", null, "\u5C4F\u5E55\u9605\u8BFB\u5668\u8BE6\u7EC6\u5EA6", -1)), ye(u("select", { "onUpdate:modelValue": f[6] || (f[6] = (b) => l.value = b), "aria-label": "\u5C4F\u5E55\u9605\u8BFB\u5668\u8BE6\u7EC6\u5EA6" }, [...f[18] || (f[18] = [u("option", { value: "off" }, "\u5173\u95ED", -1), u("option", { value: "default" }, "\u6807\u51C6", -1), u("option", { value: "verbose" }, "\u8BE6\u7EC6\uFF08\u64AD\u62A5\u6240\u6709\u72B6\u6001\u53D8\u5316\uFF09", -1)])], 512), [[vn, l.value]])])]), f[25] || (f[25] = u("div", { class: "state-legend", role: "region", "aria-label": "\u72B6\u6001\u4FE1\u53F7\u56FE\u4F8B" }, [u("h2", null, "\u72B6\u6001\u4FE1\u53F7\u8BF4\u660E"), u("p", { class: "muted" }, " \u6240\u6709\u6E38\u620F\u72B6\u6001\u901A\u8FC7\u56FE\u6807 + \u6587\u5B57\u4F20\u8FBE\uFF0C\u4E0D\u53EA\u4F9D\u8D56\u989C\u8272\uFF08\xA730: \u4E0D\u53EA\u4F9D\u8D56\u989C\u8272\u8868\u8FBE\u72B6\u6001\uFF09\u3002 "), u("ul", { class: "legend-list" }, [u("li", null, [u("span", { class: "state-icon", style: { color: "var(--color-positive)" } }, "\u2713"), se(" \u5B58\u6D3B")]), u("li", null, [u("span", { class: "state-icon", style: { color: "var(--color-warning)" } }, "\u26A0"), se(" \u6FD2\u6B7B")]), u("li", null, [u("span", { class: "state-icon", style: { color: "var(--color-negative)" } }, "\u2717"), se(" \u9635\u4EA1")]), u("li", null, [u("span", { class: "state-icon", style: { color: "var(--color-positive)" } }, "\u25B8"), se(" \u53EF\u51FA\u724C")]), u("li", null, [u("span", { class: "state-icon", style: { color: "var(--color-negative)" } }, "\u2715"), se(" \u4E0D\u53EF\u7528")]), u("li", null, [u("span", { class: "state-icon", style: { color: "var(--color-accent)" } }, "\u25BA"), se(" \u5F53\u524D\u56DE\u5408")]), u("li", null, [u("span", { class: "state-icon", style: { color: "var(--color-warning)" } }, "?"), se(" \u8BF7\u9009\u62E9")])])], -1)), _e(ob)]));
} }), vb = ln(_b, [["__scopeId", "data-v-637706ba"]]), yb = { class: "replay stack" }, kb = { key: 0, class: "muted" }, wb = { key: 1, class: "error", role: "alert" }, bb = { key: 0, class: "panel" }, xb = { class: "info-grid" }, $b = { class: "mono" }, jb = { class: "mono" }, Sb = ["disabled"], Eb = { key: 1, class: "panel" }, Ab = { class: "row playback-controls" }, Cb = ["disabled"], Rb = ["disabled"], zb = { class: "step-display mono" }, Ib = ["disabled"], Pb = ["disabled"], Tb = ["aria-valuenow", "aria-valuemax"], Ob = { class: "info-grid frame-info" }, Nb = { class: "mono" }, qb = { key: 2, class: "panel" }, Mb = { key: 0, class: "seat-grid" }, Db = { key: 1, class: "muted" }, Lb = { key: 3, class: "panel privacy-notice", role: "note" }, Ub = De({ __name: "ReplayView", setup(e) {
  const n = new ii(), t = W(null), s = W(null), a = W(null), i = W(null), o = W(false), r = W("full");
  async function l(f) {
    o.value = true, i.value = null;
    try {
      const b = JSON.parse(f);
      await n.connect(), s.value = await n.loadReplay(f), t.value = b, a.value = await n.seekReplay(0, r.value);
    } catch (b) {
      t.value = null, s.value = null, a.value = null, i.value = b instanceof Error ? b.message : String(b);
    } finally {
      o.value = false;
    }
  }
  function d(f) {
    const b = new FileReader();
    b.onload = () => void l(b.result), b.onerror = () => {
      i.value = "\u6587\u4EF6\u8BFB\u53D6\u5931\u8D25";
    }, b.readAsText(f);
  }
  function c(f) {
    var _a2;
    const b = (_a2 = f.target.files) == null ? void 0 : _a2[0];
    b && d(b);
  }
  function p(f) {
    var _a2, _b2;
    f.preventDefault();
    const b = (_b2 = (_a2 = f.dataTransfer) == null ? void 0 : _a2.files) == null ? void 0 : _b2[0];
    b && d(b);
  }
  async function m(f) {
    if (!(!s.value || o.value)) {
      o.value = true, i.value = null;
      try {
        a.value = await n.seekReplay(Math.max(0, Math.min(f, s.value.total)), r.value);
      } catch (b) {
        i.value = b instanceof Error ? b.message : String(b);
      } finally {
        o.value = false;
      }
    }
  }
  return kn(r, () => {
    a.value && m(a.value.step);
  }), as(() => n.dispose()), (f, b) => {
    var _a2;
    return x(), j("div", yb, [b[22] || (b[22] = u("section", { class: "panel" }, [u("h1", null, "\u5F55\u50CF\u56DE\u653E"), u("p", { class: "muted" }, "\u5BFC\u5165 Command Log \u540E\uFF0C\u89C4\u5219\u5F15\u64CE\u4F1A\u771F\u5B9E\u91CD\u6F14\u5E76\u9010\u6B65\u6838\u5BF9\u72B6\u6001\u54C8\u5E0C\u3002")], -1)), u("section", { class: "panel drop-zone", onDrop: p, onDragover: b[0] || (b[0] = ja(() => {
    }, ["prevent"])), "aria-label": "\u5BFC\u5165\u5F55\u50CF\u6587\u4EF6" }, [b[6] || (b[6] = u("h2", null, "\u5BFC\u5165\u5F55\u50CF", -1)), b[7] || (b[7] = u("p", { class: "muted" }, "\u62D6\u62FD .json \u5F55\u50CF\u6587\u4EF6\u5230\u6B64\u5904\uFF0C\u6216\u70B9\u51FB\u9009\u62E9", -1)), u("input", { type: "file", accept: ".json,application/json", onChange: c, "aria-label": "\u9009\u62E9\u5F55\u50CF\u6587\u4EF6" }, null, 32), o.value ? (x(), j("p", kb, "\u6B63\u5728\u9A8C\u8BC1\u5E76\u91CD\u6F14\u2026")) : V("", true), i.value ? (x(), j("p", wb, $(i.value), 1)) : V("", true)], 32), t.value && s.value ? (x(), j("section", bb, [b[15] || (b[15] = u("h2", null, "\u5F55\u50CF\u4FE1\u606F", -1)), u("dl", xb, [b[9] || (b[9] = u("dt", null, "Manifest Hash", -1)), u("dd", $b, $(s.value.manifest_hash), 1), b[10] || (b[10] = u("dt", null, "\u5F15\u64CE\u6784\u5EFA", -1)), u("dd", jb, $(((_a2 = t.value.header) == null ? void 0 : _a2.engine_build_id) ?? "\u2014"), 1), b[11] || (b[11] = u("dt", null, "\u603B\u6B65\u6570", -1)), u("dd", null, $(s.value.total), 1), b[12] || (b[12] = u("dt", null, "\u5B8C\u6574\u6027", -1)), b[13] || (b[13] = u("dd", { class: "verified" }, "\u2713 \u54C8\u5E0C\u94FE\u4E0E\u5168\u90E8\u72B6\u6001\u68C0\u67E5\u70B9\u5DF2\u9A8C\u8BC1", -1)), b[14] || (b[14] = u("dt", null, "\u9690\u79C1\u7B49\u7EA7", -1)), u("dd", null, [ye(u("select", { "onUpdate:modelValue": b[1] || (b[1] = (P) => r.value = P), disabled: o.value, "aria-label": "\u9690\u79C1\u7B49\u7EA7" }, [...b[8] || (b[8] = [u("option", { value: "full" }, "\u5B8C\u6574\uFF08\u88C1\u5224\u89C6\u89D2\uFF09", -1), u("option", { value: "player" }, "\u73A9\u5BB6 1 \u89C6\u89D2", -1), u("option", { value: "public" }, "\u516C\u5F00\u89C6\u89D2", -1)])], 8, Sb), [[vn, r.value]])])])])) : V("", true), a.value && s.value ? (x(), j("section", Eb, [b[19] || (b[19] = u("h2", null, "\u56DE\u653E\u63A7\u5236", -1)), u("div", Ab, [u("button", { type: "button", onClick: b[2] || (b[2] = (P) => m(0)), disabled: o.value || a.value.step === 0 }, "\u23EE \u5F00\u5934", 8, Cb), u("button", { type: "button", onClick: b[3] || (b[3] = (P) => m(a.value.step - 1)), disabled: o.value || a.value.step === 0 }, "\u25C0 \u540E\u9000", 8, Rb), u("span", zb, $(a.value.step) + " / " + $(a.value.total), 1), u("button", { type: "button", onClick: b[4] || (b[4] = (P) => m(a.value.step + 1)), disabled: o.value || a.value.step >= a.value.total }, "\u524D\u8FDB \u25B6", 8, Ib), u("button", { type: "button", onClick: b[5] || (b[5] = (P) => m(a.value.total)), disabled: o.value || a.value.step >= a.value.total }, "\u7ED3\u5C3E \u23ED", 8, Pb)]), u("div", { class: "progress-bar", role: "progressbar", "aria-valuenow": a.value.step, "aria-valuemax": a.value.total }, [u("div", { class: "progress-fill", style: Kn({ width: a.value.step / Math.max(a.value.total, 1) * 100 + "%" }) }, null, 4)], 8, Tb), u("dl", Ob, [b[16] || (b[16] = u("dt", null, "Revision", -1)), u("dd", null, $(a.value.revision), 1), b[17] || (b[17] = u("dt", null, "State Hash", -1)), u("dd", Nb, $(a.value.hash_hex), 1), b[18] || (b[18] = u("dt", null, "\u6BD4\u8D5B\u72B6\u6001", -1)), u("dd", null, $(a.value.finished ? "\u5DF2\u7ED3\u675F" : "\u8FDB\u884C\u4E2D"), 1)])])) : V("", true), a.value ? (x(), j("section", qb, [b[20] || (b[20] = u("h2", null, "\u5F53\u524D\u5C40\u9762", -1)), a.value.view.seats.length ? (x(), j("div", Mb, [(x(true), j(ee, null, ve(a.value.view.seats, (P) => (x(), j("article", { key: P.player, class: "seat" }, [u("strong", null, "\u5EA7\u4F4D " + $(P.seat) + " \xB7 \u73A9\u5BB6 " + $(P.player), 1), u("span", null, "\u4F53\u529B " + $(P.hp) + " / " + $(P.max_hp), 1), u("span", null, "\u624B\u724C " + $(P.hand_count) + " \xB7 " + $(P.alive ? "\u5B58\u6D3B" : "\u9635\u4EA1") + $(P.turned_over ? " \xB7 \u7FFB\u9762" : ""), 1), u("span", null, "\u8EAB\u4EFD " + $(P.identity ?? "\u672A\u77E5"), 1)]))), 128))])) : (x(), j("p", Db, "\u6BD4\u8D5B\u5C1A\u672A\u5F00\u59CB\u3002"))])) : V("", true), t.value ? (x(), j("section", Lb, [...b[21] || (b[21] = [u("h2", null, "\u9690\u79C1\u5206\u7EA7\uFF08\xA722.5\uFF09", -1), u("p", { class: "muted tiny" }, "\u6BCF\u4E2A\u753B\u9762\u76F4\u63A5\u4ECE\u5BF9\u5E94\u7684\u88C1\u5224\u3001\u73A9\u5BB6\u6216\u516C\u5F00 Observer \u6295\u5F71\u6784\u5EFA\uFF0C\u4E0D\u4F1A\u5148\u751F\u6210\u5B8C\u6574\u89C6\u56FE\u518D\u5220\u5B57\u6BB5\u3002", -1)])])) : V("", true)]);
  };
} }), Fb = ln(Ub, [["__scopeId", "data-v-91fcaceb"]]), Vb = { class: "stack" }, Hb = { class: "panel" }, Wb = { key: 0, class: "counts" }, Bb = { key: 1, class: "muted", role: "status" }, Kb = { key: 2, class: "error", role: "alert" }, Gb = { key: 0, class: "browser" }, Jb = { class: "panel roster" }, Yb = { class: "filters" }, Qb = ["value"], Xb = ["onClick"], Zb = ["src"], e0 = { key: 0, class: "muted limit" }, n0 = { key: 0, class: "panel detail" }, t0 = ["src", "alt"], s0 = ["title"], a0 = { class: "muted" }, i0 = { key: 0, class: "skills" }, o0 = ["title"], r0 = { key: 1, class: "muted" }, l0 = { key: 2, class: "skins" }, u0 = ["src", "alt"], c0 = { key: 3, class: "muted" }, d0 = { class: "muted" }, p0 = De({ __name: "CollectionView", setup(e) {
  const n = W(null), t = W(""), s = W(false), a = W(""), i = W(""), o = W(null), r = P(), l = Y(() => {
    var _a2;
    return new Map((_a2 = n.value) == null ? void 0 : _a2.skills.map((O) => [O.id, O]));
  }), d = Y(() => {
    var _a2;
    const O = a.value.trim().toLowerCase();
    return (((_a2 = n.value) == null ? void 0 : _a2.characters) ?? []).filter((w) => !i.value || w.pack === i.value).filter((w) => !O || w.name.toLowerCase().includes(O) || w.display_name.toLowerCase().includes(O)).slice(0, 80);
  }), c = Y(() => {
    var _a2;
    return ((_a2 = o.value) == null ? void 0 : _a2.resolved_skills.map((O) => l.value.get(O)).filter(Boolean)) ?? [];
  }), p = Y(() => {
    var _a2;
    return (((_a2 = n.value) == null ? void 0 : _a2.skins.static) ?? []).filter((O) => {
      var _a3;
      return O.character === ((_a3 = o.value) == null ? void 0 : _a3.name);
    });
  }), m = Y(() => {
    var _a2;
    return (((_a2 = n.value) == null ? void 0 : _a2.skins.dynamic) ?? []).filter((O) => {
      var _a3;
      return O.character === ((_a3 = o.value) == null ? void 0 : _a3.name);
    });
  });
  $t(f);
  async function f() {
    s.value = true, t.value = "";
    try {
      const O = await fetch(`${r}/catalog.json`);
      if (!O.ok) throw new Error(`${O.status} ${O.statusText}`);
      n.value = await O.json(), o.value = n.value.characters[0] ?? null;
    } catch (O) {
      t.value = `\u65E0\u6CD5\u8BFB\u53D6\u539F\u4ED3\u5E93\u5185\u5BB9\u76EE\u5F55\uFF08${String(O)}\uFF09\u3002\u8BF7\u5148\u542F\u52A8 game-server\u3002`;
    } finally {
      s.value = false;
    }
  }
  function b(O) {
    return O ? encodeURI(`${r}/${O}`) : "";
  }
  function P() {
    const w = !window.location.hostname || window.location.hostname.endsWith("tauri.localhost") ? "127.0.0.1" : window.location.hostname;
    return `${window.location.protocol === "https:" ? "https:" : "http:"}//${w}:7700/original-content`;
  }
  return (O, w) => (x(), j("div", Vb, [u("section", Hb, [w[2] || (w[2] = u("h1", null, "\u539F\u4ED3\u5E93\u6B66\u5C06\u5E93", -1)), w[3] || (w[3] = u("p", { class: "muted" }, " \u6B66\u5C06\u3001\u6280\u80FD\u6E90\u7801\u3001\u9759\u6001\u76AE\u80A4\u548C\u52A8\u6001\u76AE\u80A4\u5747\u4ECE\u672C\u5730\u539F\u4ED3\u5E93\u76EE\u5F55\u8BFB\u53D6\uFF1B\u65E7 JavaScript \u4E0D\u4F1A\u5728\u5BA2\u6237\u7AEF\u6267\u884C\u3002 ", -1)), n.value ? (x(), j("p", Wb, $(n.value.counts.packs) + " \u5305 \xB7 " + $(n.value.counts.characters) + " \u6B66\u5C06 \xB7 " + $(n.value.counts.skills) + " \u6280\u80FD \xB7 " + $(n.value.counts.static_skins) + " \u9759\u6001\u76AE\u80A4 \xB7 " + $(n.value.counts.dynamic_skins) + " \u52A8\u6001\u76AE\u80A4 ", 1)) : V("", true), s.value ? (x(), j("p", Bb, "\u6B63\u5728\u52A0\u8F7D\u76EE\u5F55\u2026")) : V("", true), t.value ? (x(), j("p", Kb, $(t.value), 1)) : V("", true)]), n.value ? (x(), j("section", Gb, [u("aside", Jb, [u("div", Yb, [ye(u("input", { "onUpdate:modelValue": w[0] || (w[0] = (_) => a.value = _), type: "search", placeholder: "\u641C\u7D22\u6B66\u5C06", "aria-label": "\u641C\u7D22\u6B66\u5C06" }, null, 512), [[Wn, a.value]]), ye(u("select", { "onUpdate:modelValue": w[1] || (w[1] = (_) => i.value = _), "aria-label": "\u6B66\u5C06\u5305" }, [w[4] || (w[4] = u("option", { value: "" }, "\u5168\u90E8\u6B66\u5C06\u5305", -1)), (x(true), j(ee, null, ve(n.value.packages, (_) => (x(), j("option", { key: _, value: _ }, $(_), 9, Qb))), 128))], 512), [[vn, i.value]])]), (x(true), j(ee, null, ve(d.value, (_) => {
    var _a2;
    return x(), j("button", { key: _.id, type: "button", class: Re(["character-row", { selected: ((_a2 = o.value) == null ? void 0 : _a2.id) === _.id }]), onClick: (A) => o.value = _ }, [_.portrait ? (x(), j("img", { key: 0, src: b(_.portrait), alt: "", loading: "lazy" }, null, 8, Zb)) : V("", true), u("span", null, [u("strong", null, $(_.display_name), 1), u("small", null, $(_.pack) + " \xB7 " + $(_.name), 1)])], 10, Xb);
  }), 128)), d.value.length === 80 ? (x(), j("p", e0, "\u4EC5\u663E\u793A\u524D 80 \u9879\uFF0C\u8BF7\u7EE7\u7EED\u7B5B\u9009\u3002")) : V("", true)]), o.value ? (x(), j("article", n0, [u("header", null, [o.value.portrait ? (x(), j("img", { key: 0, src: b(o.value.portrait), alt: `${o.value.display_name}\u6B66\u5C06\u5934\u50CF` }, null, 8, t0)) : V("", true), u("div", null, [u("h2", null, $(o.value.display_name), 1), u("span", { class: Re(["character-status", o.value.migration_status]), title: o.value.migration_status === "ported" ? "\u5B8C\u6574\u6B66\u5C06\u5B9A\u4E49\u5DF2\u63A5\u5165\u786E\u5B9A\u6027\u5185\u5BB9\u5305" : "\u4EC5\u540C\u6B65\u76EE\u5F55\u3001\u6E90\u7801\u4E0E\u8D44\u6E90\uFF0C\u5C1A\u4E0D\u80FD\u8FDB\u5165\u786E\u5B9A\u6027\u5BF9\u5C40" }, $(o.value.migration_status === "ported" ? "\u6B66\u5C06\u5305\u5DF2\u63A5\u5165" : "\u5F85\u5B8C\u6574\u79FB\u690D"), 11, s0), u("p", a0, $(o.value.pack) + " \xB7 " + $(o.value.name) + " \xB7 " + $(o.value.group ?? "\u672A\u77E5\u52BF\u529B") + " \xB7 " + $(o.value.hp) + " \u4F53\u529B", 1)])]), w[5] || (w[5] = u("h3", null, "\u6280\u80FD", -1)), c.value.length ? (x(), j("ul", i0, [(x(true), j(ee, null, ve(c.value, (_) => (x(), j("li", { key: _.id }, [u("strong", null, $(_.display_name), 1), u("span", { class: Re(_.migration_status), title: _.migration_status === "ported" ? "\u5DF2\u63A5\u5165\u786E\u5B9A\u6027\u5185\u5BB9\u5305\uFF1B\u662F\u5426\u4E0E\u539F\u6280\u80FD\u5B8C\u5168\u7B49\u4EF7\u4ECD\u4EE5 Golden \u4E3A\u51C6" : "\u4EC5\u4FDD\u7559\u539F\u6E90\u7801\u4E0E\u76EE\u5F55\uFF0C\u4E0D\u80FD\u8FDB\u5165\u786E\u5B9A\u6027\u5BF9\u5C40" }, $(_.migration_status === "ported" ? "VM \u5DF2\u63A5\u5165" : "\u5F85 DSL \u91CD\u5199"), 11, o0), u("p", null, $(_.description ?? _.name), 1)]))), 128))])) : (x(), j("p", r0, "\u6CA1\u6709\u53EF\u89E3\u6790\u7684\u6B66\u5C06\u5305\u6280\u80FD\u3002")), u("h3", null, "\u9759\u6001\u76AE\u80A4\uFF08" + $(p.value.length) + "\uFF09", 1), p.value.length ? (x(), j("div", l0, [(x(true), j(ee, null, ve(p.value, (_) => (x(), j("figure", { key: _.id }, [u("img", { src: b(_.asset), alt: `${o.value.display_name} \xB7 ${_.name}`, loading: "lazy" }, null, 8, u0), u("figcaption", null, $(_.name), 1)]))), 128))])) : (x(), j("p", c0, "\u539F\u4ED3\u5E93\u672A\u63D0\u4F9B\u9759\u6001\u76AE\u80A4\u3002")), u("h3", null, "\u52A8\u6001\u76AE\u80A4\uFF08" + $(m.value.length) + "\uFF09", 1), u("p", d0, $(m.value.map((_) => _.name).join("\u3001") || "\u539F\u4ED3\u5E93\u672A\u63D0\u4F9B\u52A8\u6001\u76AE\u80A4\u3002"), 1)])) : V("", true)])) : V("", true)]));
} }), h0 = ln(p0, [["__scopeId", "data-v-bc3fbd15"]]), f0 = { class: "stack" }, m0 = { class: "panel" }, g0 = { class: "muted" }, _0 = { class: "row" }, v0 = { key: 0, class: "ok", role: "status" }, y0 = { key: 1, class: "error", role: "alert" }, k0 = { key: 2, class: "warn", role: "status" }, w0 = { class: "panel" }, b0 = { class: "row" }, x0 = ["disabled"], $0 = ["disabled"], j0 = { class: "panel" }, S0 = { class: "muted mono" }, E0 = { key: 0, class: "lock-list" }, A0 = { key: 1, class: "muted" }, C0 = { class: "panel" }, R0 = { class: "table", role: "table" }, z0 = { key: 0, class: "badge-unsigned", title: "\u672C\u5730\u5BFC\u5165\u7684\u5305\u672A\u7ECF\u7B7E\u540D\u6821\u9A8C" }, I0 = { class: "mono" }, P0 = { class: "mono" }, T0 = { class: "actions" }, O0 = ["disabled", "onClick"], N0 = ["disabled", "onClick"], q0 = ["disabled", "onClick"], M0 = ["disabled", "title", "onClick"], D0 = ["disabled", "onClick"], L0 = ["disabled", "onClick"], U0 = ["disabled", "onClick"], F0 = De({ __name: "ExtensionsView", setup(e) {
  const n = W(Is()), t = W(""), s = W(""), a = W(""), i = W(false);
  let o = null;
  as(() => {
    o == null ? void 0 : o.then((w) => w.dispose(), () => {
    });
  });
  function r() {
    return o ?? (o = (async () => {
      const w = new ii();
      return await w.connect(), w;
    })().catch((w) => {
      throw o = null, w;
    })), o;
  }
  async function l(w) {
    await (await r()).validatePackage(w);
  }
  function d() {
    n.value = Is(), t.value = "";
  }
  function c(w) {
    t.value = "", s.value = "";
    try {
      w(), d();
    } catch (_) {
      t.value = _ instanceof Error ? _.message : String(_);
    }
  }
  async function p() {
    if (!i.value) {
      t.value = "", s.value = "", i.value = true;
      try {
        const w = a.value;
        await gv(w, l);
        const _ = JSON.parse(w).manifest;
        s.value = `\u5DF2\u5BFC\u5165 ${_.namespace}/${_.package_id} @ ${_.version}\uFF08\u672A\u7B7E\u540D\uFF09`, a.value = "", d();
      } catch (w) {
        t.value = w instanceof Error ? w.message : String(w);
      } finally {
        i.value = false;
      }
    }
  }
  async function m(w) {
    var _a2;
    const _ = w.target, A = (_a2 = _.files) == null ? void 0 : _a2[0];
    if (A) try {
      a.value = await A.text(), await p();
    } catch (T) {
      t.value = T instanceof Error ? T.message : String(T);
    } finally {
      _.value = "";
    }
  }
  function f(w) {
    return n.value.installed.find((_) => _.key === w);
  }
  function b(w) {
    const _ = f(w);
    return _ ? _.lifecycle === "active" ? "\u5DF2\u542F\u7528" : _.lifecycle === "disabled" ? "\u5DF2\u505C\u7528" : _.lifecycle === "revoked" ? "\u5DF2\u64A4\u9500" : "\u5DF2\u5B89\u88C5" : "\u672A\u5B89\u88C5";
  }
  const P = Y(() => $v(n.value)), O = Y(() => {
    const w = Kt.map((T) => {
      const H = ri(T), le = f(H);
      return { title: T.title, version: T.version, key: H, record: le, status: b(H), imported: false };
    }), _ = new Set(w.map((T) => T.key)), A = n.value.installed.filter((T) => !_.has(T.key)).map((T) => ({ title: `${T.packageId}\uFF08\u5BFC\u5165\uFF09`, version: T.version, key: T.key, record: T, status: b(T.key), imported: true }));
    return [...w, ...A];
  });
  return (w, _) => (x(), j("div", f0, [u("section", m0, [_[2] || (_[2] = u("h1", null, "\u6269\u5C55\u5305\u7BA1\u7406", -1)), u("p", g0, " \u5BF9\u9F50\u5BBF\u4E3B\u5951\u7EA6\uFF1A\u5B89\u88C5 / \u542F\u7528 / \u505C\u7528 / \u5347\u7EA7 / \u56DE\u9000 / \u5378\u8F7D / \u64A4\u9500\u3002\u5F53\u524D\u76EE\u5F55 " + $(D(Kt).length) + " \u4E2A G1 \u8F6C\u6362\u5305\u3002\u5F00\u5C40\u5185\u5BB9\u9501\u7531\u542F\u7528\u96C6\u6784\u6210\uFF1B\u5BF9\u5C40\u4E2D\u51BB\u7ED3\u53D8\u66F4\u3002 \u5B89\u88C5\u65F6\u4FDD\u5B58\u5B9E\u9645\u5305\u5185\u5BB9\uFF0C\u672C\u5730\u539F\u751F\u5BF9\u5C40\u76F4\u63A5\u88C5\u8F7D\uFF1B\u5B58\u6863\u4FDD\u7559\u539F\u5C40\u5305\u526F\u672C\u3002 ", 1), u("div", _0, [u("button", { class: "primary", type: "button", onClick: _[0] || (_[0] = (A) => {
    c(() => {
      D(jv)(), s.value = `\u5DF2\u5B89\u88C5\u5E76\u542F\u7528\u5168\u90E8 ${D(Kt).length} \u5305`;
    });
  }) }, " \u5168\u90E8\u5B89\u88C5\u5E76\u542F\u7528 "), u("button", { type: "button", onClick: d }, "\u5237\u65B0")]), s.value ? (x(), j("p", v0, $(s.value), 1)) : V("", true), t.value ? (x(), j("p", y0, $(t.value), 1)) : V("", true), n.value.matchFrozen ? (x(), j("p", k0, "\u5BF9\u5C40\u8FDB\u884C\u4E2D \xB7 \u5185\u5BB9\u9501\u5DF2\u51BB\u7ED3")) : V("", true)]), u("section", w0, [_[4] || (_[4] = u("h2", null, "\u5BFC\u5165\u6269\u5C55\u5305", -1)), _[5] || (_[5] = u("p", { class: "muted" }, " \u6B63\u5F0F\u5BFC\u5165\u5165\u53E3\uFF1A\u7C98\u8D34\u6216\u9009\u62E9\u4F5C\u8005\u4EA7\u7269\uFF08\u5B8C\u6574 npack JSON\uFF0C\u53EF\u542B VM \u6A21\u5757\u4E0E\u5361\u724C/\u6B66\u5C06/\u724C\u5806\u8D44\u4EA7\uFF09\u3002 \u540C\u5305\u65B0\u7248\u672C\u6309\u5347\u7EA7\u5904\u7406\u5E76\u4FDD\u7559\u65E7\u7248\u672C\u526F\u672C\uFF1B\u540C\u7248\u672C\u5185\u5BB9\u51B2\u7A81\u3001\u5DF2\u64A4\u9500\u3001\u5BF9\u5C40\u51BB\u7ED3\u3001\u7ED3\u6784\u6821\u9A8C\u6216 Rust \u6DF1\u5C42\u6821\u9A8C\uFF08\u5305/\u6A21\u5757\u6307\u7EB9\u3001\u5B57\u8282\u7801\u3001\u5BBF\u4E3B ABI \u7A97\u53E3\uFF09\u5931\u8D25\u65F6\u62D2\u7EDD\u4E14\u4E0D\u6539\u5199\u5DF2\u5B89\u88C5\u5185\u5BB9\u3002 \u672C\u5730\u5BFC\u5165\u7684\u5305\u672A\u7ECF\u7B7E\u540D\u6821\u9A8C\uFF0C\u5217\u8868\u4E2D\u6807\u4E3A\u300C\u672A\u7B7E\u540D\u300D\u3002 ", -1)), ye(u("textarea", { "onUpdate:modelValue": _[1] || (_[1] = (A) => a.value = A), class: "import-box", rows: "4", spellcheck: "false", "aria-label": "\u6269\u5C55\u5305 JSON", placeholder: '{"manifest": {...}, "modules": [...], ...}' }, null, 512), [[Wn, a.value]]), u("div", b0, [u("button", { class: "primary", type: "button", disabled: !a.value.trim() || n.value.matchFrozen || i.value, onClick: p }, $(i.value ? "\u6821\u9A8C\u4E2D\u2026" : "\u5BFC\u5165"), 9, x0), u("label", null, [_[3] || (_[3] = se(" \u9009\u62E9\u6587\u4EF6 ", -1)), u("input", { type: "file", accept: ".json,application/json", "aria-label": "\u9009\u62E9\u6269\u5C55\u5305\u6587\u4EF6", disabled: n.value.matchFrozen, onChange: m }, null, 40, $0)])])]), u("section", j0, [_[6] || (_[6] = u("h2", null, "\u5185\u5BB9\u9501", -1)), u("p", S0, $(P.value.hashSeed || "\uFF08\u7A7A\uFF09"), 1), P.value.packages.length ? (x(), j("ul", E0, [(x(true), j(ee, null, ve(P.value.packages, (A) => (x(), j("li", { key: A.key }, $(A.key) + " @ " + $(A.version) + " \xB7 " + $(A.fingerprint), 1))), 128))])) : (x(), j("p", A0, "\u5C1A\u672A\u542F\u7528\u4EFB\u4F55\u6269\u5C55\u3002"))]), u("section", C0, [_[8] || (_[8] = u("h2", null, "\u5305\u5217\u8868", -1)), u("div", R0, [_[7] || (_[7] = u("div", { class: "thead", role: "row" }, [u("span", null, "\u540D\u79F0"), u("span", null, "\u6807\u8BC6"), u("span", null, "\u7248\u672C"), u("span", null, "\u72B6\u6001"), u("span", null, "\u64CD\u4F5C")], -1)), (x(true), j(ee, null, ve(O.value, (A) => {
    var _a2, _b2, _c2;
    return x(), j("div", { key: A.key, class: "trow", role: "row" }, [u("span", null, [se($(A.title) + " ", 1), ((_a2 = A.record) == null ? void 0 : _a2.signature) === "unsigned" ? (x(), j("span", z0, "\u672A\u7B7E\u540D")) : V("", true)]), u("span", I0, $(A.key), 1), u("span", P0, $(((_b2 = A.record) == null ? void 0 : _b2.version) ?? A.version), 1), u("span", null, $(A.status), 1), u("span", T0, [u("button", { type: "button", disabled: A.imported || !!A.record && A.record.lifecycle !== "revoked", onClick: (T) => c(() => D(xl)(A.key)) }, " \u5B89\u88C5 ", 8, O0), u("button", { type: "button", disabled: !A.record || A.record.lifecycle === "active" || A.record.lifecycle === "revoked", onClick: (T) => c(() => D(jl)(A.key)) }, " \u542F\u7528 ", 8, N0), u("button", { type: "button", disabled: !A.record || A.record.lifecycle !== "active", onClick: (T) => c(() => D(yv)(A.key)) }, " \u505C\u7528 ", 8, q0), u("button", { type: "button", disabled: A.imported || !A.record || A.record.lifecycle === "revoked", title: A.imported ? "\u5BFC\u5165\u5305\u8BF7\u5BFC\u5165\u65B0\u7248\u672C\u5305\u4EE5\u5347\u7EA7" : "", onClick: (T) => c(() => D(kv)(A.key)) }, " \u5347\u7EA7 ", 8, M0), u("button", { type: "button", disabled: !A.record || (((_c2 = A.record.history) == null ? void 0 : _c2.length) ?? 0) < 2, onClick: (T) => {
      c(() => {
        const H = A.record.history.filter((le) => le !== A.record.version);
        H.length && D(wv)(A.key, H[H.length - 1]);
      });
    } }, " \u56DE\u9000 ", 8, D0), u("button", { type: "button", disabled: !A.record, onClick: (T) => c(() => D(bv)(A.key)) }, " \u5378\u8F7D ", 8, L0), u("button", { type: "button", disabled: !A.record || A.record.lifecycle === "revoked", onClick: (T) => c(() => D(xv)(A.key)) }, " \u64A4\u9500 ", 8, U0)])]);
  }), 128))])])]));
} }), V0 = ln(F0, [["__scopeId", "data-v-2bd63404"]]), Pl = lp({ history: jn() ? Fd() : ol(), routes: [{ path: "/", name: "home", component: Pp }, { path: "/battle", name: "battle", component: Jw }, { path: "/extensions", name: "extensions", component: V0 }, { path: "/settings", name: "settings", component: vb }, { path: "/replay", name: "replay", component: Fb }, { path: "/collection", name: "collection", component: h0 }], scrollBehavior: () => ({ top: 0 }) });
function H0() {
  const e = sessionStorage.getItem("noname-next.online-intent");
  if (e) try {
    const n = JSON.parse(e);
    if (n.action === "spectate") return;
    Pl.push({ path: "/battle", query: { mode: "online", action: n.action === "create" ? "join" : n.action ?? "join", room: String(n.roomId ?? 1), players: String(n.players ?? 4), seed: String(n.seed ?? 42) } });
  } catch {
    sessionStorage.removeItem("noname-next.online-intent");
  }
}
const W0 = td(fp).use(Pl);
W0.mount("#app");
H0();
