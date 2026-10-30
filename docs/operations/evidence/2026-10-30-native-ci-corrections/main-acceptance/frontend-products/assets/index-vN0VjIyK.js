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
}, Ol = Object.prototype.hasOwnProperty, de = (e, n) => Ol.call(e, n), Z = Array.isArray, ct = (e) => ts(e) === "[object Map]", wt = (e) => ts(e) === "[object Set]", hi = (e) => ts(e) === "[object Date]", ne = (e) => typeof e == "function", xe = (e) => typeof e == "string", bn = (e) => typeof e == "symbol", pe = (e) => e !== null && typeof e == "object", Wo = (e) => (pe(e) || ne(e)) && ne(e.then) && ne(e.catch), Bo = Object.prototype.toString, ts = (e) => Bo.call(e), Nl = (e) => ts(e).slice(8, -1), Ko = (e) => ts(e) === "[object Object]", Fa = (e) => xe(e) && e !== "NaN" && e[0] !== "-" && "" + parseInt(e, 10) === e, Lt = La(",key,ref,ref_for,ref_key,onVnodeBeforeMount,onVnodeMounted,onVnodeBeforeUpdate,onVnodeUpdated,onVnodeBeforeUnmount,onVnodeUnmounted"), qs = (e) => {
  const n = /* @__PURE__ */ Object.create(null);
  return ((t) => n[t] || (n[t] = e(t)));
}, ql = /-\w/g, Ge = qs((e) => e.replace(ql, (n) => n.slice(1).toUpperCase())), Ml = /\B([A-Z])/g, Jn = qs((e) => e.replace(Ml, "-$1").toLowerCase()), Va = qs((e) => e.charAt(0).toUpperCase() + e.slice(1)), Qs = qs((e) => e ? `on${Va(e)}` : ""), _n = (e, n) => !Object.is(e, n), gs = (e, ...n) => {
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
  if (Z(e)) {
    const n = {};
    for (let t = 0; t < e.length; t++) {
      const s = e[t], a = xe(s) ? Fl(s) : Kn(s);
      if (a) for (const i in a) n[i] = a[i];
    }
    return n;
  } else if (xe(e) || pe(e)) return e;
}
const Dl = /;(?![^(]*\))/g, Ll = /:([^]+)/, Ul = /\/\*[^]*?\*\//g;
function Fl(e) {
  const n = {};
  return e.replace(Ul, "").split(Dl).forEach((t) => {
    if (t) {
      const s = t.split(Ll);
      s.length > 1 && (n[s[0].trim()] = s[1].trim());
    }
  }), n;
}
function Re(e) {
  let n = "";
  if (xe(e)) n = e;
  else if (Z(e)) for (let t = 0; t < e.length; t++) {
    const s = Re(e[t]);
    s && (n += s + " ");
  }
  else if (pe(e)) for (const t in e) e[t] && (n += t + " ");
  return n.trim();
}
const Vl = "itemscope,allowfullscreen,formnovalidate,ismap,nomodule,novalidate,readonly", Hl = La(Vl);
function Jo(e) {
  return !!e || e === "";
}
function Wl(e, n) {
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
  if (t = Z(e), s = Z(n), t || s) return t && s ? Wl(e, n) : false;
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
const Yo = (e) => !!(e && e.__v_isRef === true), b = (e) => xe(e) ? e : e == null ? "" : Z(e) || pe(e) && (e.toString === Bo || !ne(e.toString)) ? Yo(e) ? b(e.value) : JSON.stringify(e, Qo, 2) : String(e), Qo = (e, n) => Yo(n) ? Qo(e, n.value) : ct(n) ? { [`Map(${n.size})`]: [...n.entries()].reduce((t, [s, a], i) => (t[Xs(s, i) + " =>"] = a, t), {}) } : wt(n) ? { [`Set(${n.size})`]: [...n.values()].map((t) => Xs(t)) } : bn(n) ? Xs(n) : pe(n) && !Z(n) && !Ko(n) ? String(n) : n, Xs = (e, n = "") => {
  var t;
  return bn(e) ? `Symbol(${(t = e.description) != null ? t : n})` : e;
};
/**
* @vue/reactivity v3.5.39
* (c) 2018-present Yuxi (Evan) You and Vue contributors
* @license MIT
**/
let Ce;
class Bl {
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
function Kl() {
  return Ce;
}
let _e;
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
    const n = _e, t = tn;
    _e = this, tn = true;
    try {
      return this.fn();
    } finally {
      tr(this), _e = n, tn = t, this.flags &= -3;
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
let Zo = 0, Ut, Ft;
function er(e, n = false) {
  if (e.flags |= 8, n) {
    e.next = Ft, Ft = e;
    return;
  }
  e.next = Ut, Ut = e;
}
function Wa() {
  Zo++;
}
function Ba() {
  if (--Zo > 0) return;
  if (Ft) {
    let n = Ft;
    for (Ft = void 0; n; ) {
      const t = n.next;
      n.next = void 0, n.flags &= -9, n = t;
    }
  }
  let e;
  for (; Ut; ) {
    let n = Ut;
    for (Ut = void 0; n; ) {
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
    s.version === -1 ? (s === t && (t = a), Ka(s), Gl(s)) : n = s, s.dep.activeLink = s.prevActiveLink, s.prevActiveLink = void 0, s = a;
  }
  e.deps = n, e.depsTail = t;
}
function fa(e) {
  for (let n = e.deps; n; n = n.nextDep) if (n.dep.version !== n.version || n.dep.computed && (sr(n.dep.computed) || n.dep.version !== n.version)) return true;
  return !!e._dirty;
}
function sr(e) {
  if (e.flags & 4 && !(e.flags & 16) || (e.flags &= -17, e.globalVersion === Jt) || (e.globalVersion = Jt, !e.isSSR && e.flags & 128 && (!e.deps && !e._dirty || !fa(e)))) return;
  e.flags |= 2;
  const n = e.dep, t = _e, s = tn;
  _e = e, tn = true;
  try {
    nr(e);
    const a = e.fn(e._value);
    (n.version === 0 || _n(a, e._value)) && (e.flags |= 128, e._value = a, n.version++);
  } catch (a) {
    throw n.version++, a;
  } finally {
    _e = t, tn = s, tr(e), e.flags &= -3;
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
function Gl(e) {
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
    const t = _e;
    _e = void 0;
    try {
      n();
    } finally {
      _e = t;
    }
  }
}
let Jt = 0;
class Jl {
  constructor(n, t) {
    this.sub = n, this.dep = t, this.version = t.version, this.nextDep = this.prevDep = this.nextSub = this.prevSub = this.prevActiveLink = void 0;
  }
}
class Ga {
  constructor(n) {
    this.computed = n, this.version = 0, this.activeLink = void 0, this.subs = void 0, this.map = void 0, this.key = void 0, this.sc = 0, this.__v_skip = true;
  }
  track(n) {
    if (!_e || !tn || _e === this.computed) return;
    let t = this.activeLink;
    if (t === void 0 || t.sub !== _e) t = this.activeLink = new Jl(_e, this), _e.deps ? (t.prevDep = _e.depsTail, _e.depsTail.nextDep = t, _e.depsTail = t) : _e.deps = _e.depsTail = t, ir(t);
    else if (t.version === -1 && (t.version = this.version, t.nextDep)) {
      const s = t.nextDep;
      s.prevDep = t.prevDep, t.prevDep && (t.prevDep.nextDep = s), t.prevDep = _e.depsTail, t.nextDep = void 0, _e.depsTail.nextDep = t, _e.depsTail = t, _e.deps === t && (_e.deps = s);
    }
    return t;
  }
  trigger(n) {
    this.version++, Jt++, this.notify(n);
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
const ma = /* @__PURE__ */ new WeakMap(), et = Symbol(""), ga = Symbol(""), Yt = Symbol("");
function Ie(e, n, t) {
  if (tn && _e) {
    let s = ma.get(e);
    s || ma.set(e, s = /* @__PURE__ */ new Map());
    let a = s.get(t);
    a || (s.set(t, a = new Ga()), a.map = s, a.key = t), a.track();
  }
}
function In(e, n, t, s, a, i) {
  const o = ma.get(e);
  if (!o) {
    Jt++;
    return;
  }
  const r = (l) => {
    l && l.trigger();
  };
  if (Wa(), n === "clear") o.forEach(r);
  else {
    const l = Z(e), c = l && Fa(t);
    if (l && t === "length") {
      const d = Number(s);
      o.forEach((p, m) => {
        (m === "length" || m === Yt || !bn(m) && m >= d) && r(p);
      });
    } else switch ((t !== void 0 || o.has(void 0)) && r(o.get(t)), c && r(o.get(Yt)), n) {
      case "add":
        l ? c && r(o.get("length")) : (r(o.get(et)), ct(e) && r(o.get(ga)));
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
  return n === e ? n : (Ie(n, "iterate", Yt), Ze(e) ? n : n.map(an));
}
function Ls(e) {
  return Ie(e = ce(e), "iterate", Yt), e;
}
function mn(e, n) {
  return On(e) ? ft(nt(e) ? an(n) : n) : an(n);
}
const Yl = { __proto__: null, [Symbol.iterator]() {
  return ea(this, Symbol.iterator, (e) => mn(this, e));
}, concat(...e) {
  return at(this).concat(...e.map((n) => Z(n) ? at(n) : n));
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
const Ql = Array.prototype;
function En(e, n, t, s, a, i) {
  const o = Ls(e), r = o !== e && !Ze(e), l = o[n];
  if (l !== Ql[n]) {
    const p = l.apply(e, i);
    return r ? an(p) : p;
  }
  let c = t;
  o !== e && (r ? c = function(p, m) {
    return t.call(this, mn(e, p), m, e);
  } : t.length > 2 && (c = function(p, m) {
    return t.call(this, p, m, e);
  }));
  const d = l.call(o, c, s);
  return r && a ? a(d) : d;
}
function gi(e, n, t, s) {
  const a = Ls(e), i = a !== e && !Ze(e);
  let o = t, r = false;
  a !== e && (i ? (r = s.length === 0, o = function(c, d, p) {
    return r && (r = false, c = mn(e, c)), t.call(this, c, mn(e, d), p, e);
  }) : t.length > 3 && (o = function(c, d, p) {
    return t.call(this, c, d, p, e);
  }));
  const l = a[n](o, ...s);
  return r ? mn(e, l) : l;
}
function na(e, n, t) {
  const s = ce(e);
  Ie(s, "iterate", Yt);
  const a = s[n](...t);
  return (a === -1 || a === false) && Qa(t[0]) ? (t[0] = ce(t[0]), s[n](...t)) : a;
}
function Et(e, n, t = []) {
  xn(), Wa();
  const s = ce(e)[n].apply(e, t);
  return Ba(), $n(), s;
}
const Xl = La("__proto__,__v_isRef,__isVue"), or = new Set(Object.getOwnPropertyNames(Symbol).filter((e) => e !== "arguments" && e !== "caller").map((e) => Symbol[e]).filter(bn));
function Zl(e) {
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
    if (t === "__v_raw") return s === (a ? i ? uu : dr : i ? cr : ur).get(n) || Object.getPrototypeOf(n) === Object.getPrototypeOf(s) ? n : void 0;
    const o = Z(n);
    if (!a) {
      let l;
      if (o && (l = Yl[t])) return l;
      if (t === "hasOwnProperty") return Zl;
    }
    const r = Reflect.get(n, t, Pe(n) ? n : s);
    if ((bn(t) ? or.has(t) : Xl(t)) || (a || Ie(n, "get", t), i)) return r;
    if (Pe(r)) {
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
    const o = Z(n) && Fa(t);
    if (!this._isShallow) {
      const c = On(i);
      if (!Ze(s) && !On(s) && (i = ce(i), s = ce(s)), !o && Pe(i) && !Pe(s)) return c || (i.value = s), true;
    }
    const r = o ? Number(t) < n.length : de(n, t), l = Reflect.set(n, t, s, Pe(n) ? n : a);
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
    return Ie(n, "iterate", Z(n) ? "length" : et), Reflect.ownKeys(n);
  }
}
class eu extends rr {
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
const nu = new lr(), tu = new eu(), su = new lr(true);
const _a = (e) => e, cs = (e) => Reflect.getPrototypeOf(e);
function au(e, n, t) {
  return function(...s) {
    const a = this.__v_raw, i = ce(a), o = ct(i), r = e === "entries" || e === Symbol.iterator && o, l = e === "keys" && o, c = a[e](...s), d = t ? _a : n ? ft : an;
    return !n && Ie(i, "iterate", l ? ga : et), ze(Object.create(c), { next() {
      const { value: p, done: m } = c.next();
      return m ? { value: p, done: m } : { value: r ? [d(p[0]), d(p[1])] : d(p), done: m };
    } });
  };
}
function ds(e) {
  return function(...n) {
    return e === "delete" ? false : e === "clear" ? void 0 : this;
  };
}
function iu(e, n) {
  const t = { get(a) {
    const i = this.__v_raw, o = ce(i), r = ce(a);
    e || (_n(a, r) && Ie(o, "get", a), Ie(o, "get", r));
    const { has: l } = cs(o), c = n ? _a : e ? ft : an;
    if (l.call(o, a)) return c(i.get(a));
    if (l.call(o, r)) return c(i.get(r));
    i !== o && i.get(a);
  }, get size() {
    const a = this.__v_raw;
    return !e && Ie(ce(a), "iterate", et), a.size;
  }, has(a) {
    const i = this.__v_raw, o = ce(i), r = ce(a);
    return e || (_n(a, r) && Ie(o, "has", a), Ie(o, "has", r)), a === r ? i.has(a) : i.has(a) || i.has(r);
  }, forEach(a, i) {
    const o = this, r = o.__v_raw, l = ce(r), c = n ? _a : e ? ft : an;
    return !e && Ie(l, "iterate", et), r.forEach((d, p) => a.call(i, c(d), c(p), o));
  } };
  return ze(t, e ? { add: ds("add"), set: ds("set"), delete: ds("delete"), clear: ds("clear") } : { add(a) {
    const i = ce(this), o = cs(i), r = ce(a), l = !n && !Ze(a) && !On(a) ? r : a;
    return o.has.call(i, l) || _n(a, l) && o.has.call(i, a) || _n(r, l) && o.has.call(i, r) || (i.add(l), In(i, "add", l, l)), this;
  }, set(a, i) {
    !n && !Ze(i) && !On(i) && (i = ce(i));
    const o = ce(this), { has: r, get: l } = cs(o);
    let c = r.call(o, a);
    c || (a = ce(a), c = r.call(o, a));
    const d = l.call(o, a);
    return o.set(a, i), c ? _n(i, d) && In(o, "set", a, i) : In(o, "add", a, i), this;
  }, delete(a) {
    const i = ce(this), { has: o, get: r } = cs(i);
    let l = o.call(i, a);
    l || (a = ce(a), l = o.call(i, a)), r && r.call(i, a);
    const c = i.delete(a);
    return l && In(i, "delete", a, void 0), c;
  }, clear() {
    const a = ce(this), i = a.size !== 0, o = a.clear();
    return i && In(a, "clear", void 0, void 0), o;
  } }), ["keys", "values", "entries", Symbol.iterator].forEach((a) => {
    t[a] = au(a, e, n);
  }), t;
}
function Ja(e, n) {
  const t = iu(e, n);
  return (s, a, i) => a === "__v_isReactive" ? !e : a === "__v_isReadonly" ? e : a === "__v_raw" ? s : Reflect.get(de(t, a) && a in s ? t : s, a, i);
}
const ou = { get: Ja(false, false) }, ru = { get: Ja(false, true) }, lu = { get: Ja(true, false) };
const ur = /* @__PURE__ */ new WeakMap(), cr = /* @__PURE__ */ new WeakMap(), dr = /* @__PURE__ */ new WeakMap(), uu = /* @__PURE__ */ new WeakMap();
function cu(e) {
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
  return On(e) ? e : Ya(e, false, nu, ou, ur);
}
function pr(e) {
  return Ya(e, false, su, ru, cr);
}
function va(e) {
  return Ya(e, true, tu, lu, dr);
}
function Ya(e, n, t, s, a) {
  if (!pe(e) || e.__v_raw && !(n && e.__v_isReactive) || e.__v_skip || !Object.isExtensible(e)) return e;
  const i = a.get(e);
  if (i) return i;
  const o = cu(Nl(e));
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
function du(e) {
  return !de(e, "__v_skip") && Object.isExtensible(e) && Go(e, "__v_skip", true), e;
}
const an = (e) => pe(e) ? xt(e) : e, ft = (e) => pe(e) ? va(e) : e;
function Pe(e) {
  return e ? e.__v_isRef === true : false;
}
function B(e) {
  return hr(e, false);
}
function pu(e) {
  return hr(e, true);
}
function hr(e, n) {
  return Pe(e) ? e : new hu(e, n);
}
class hu {
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
function L(e) {
  return Pe(e) ? e.value : e;
}
const fu = { get: (e, n, t) => n === "__v_raw" ? e : L(Reflect.get(e, n, t)), set: (e, n, t, s) => {
  const a = e[n];
  return Pe(a) && !Pe(t) ? (a.value = t, true) : Reflect.set(e, n, t, s);
} };
function fr(e) {
  return nt(e) ? e : new Proxy(e, fu);
}
class mu {
  constructor(n, t, s) {
    this.fn = n, this.setter = t, this._value = void 0, this.dep = new Ga(this), this.__v_isRef = true, this.deps = void 0, this.depsTail = void 0, this.flags = 16, this.globalVersion = Jt - 1, this.next = void 0, this.effect = this, this.__v_isReadonly = !t, this.isSSR = s;
  }
  notify() {
    if (this.flags |= 16, !(this.flags & 8) && _e !== this) return er(this, true), true;
  }
  get value() {
    const n = this.dep.track();
    return sr(this), n && (n.version = this.dep.version), this._value;
  }
  set value(n) {
    this.setter && this.setter(n);
  }
}
function gu(e, n, t = false) {
  let s, a;
  return ne(e) ? s = e : (s = e.get, a = e.set), new mu(s, a, t);
}
const ps = {}, ws = /* @__PURE__ */ new WeakMap();
let Xn;
function _u(e, n = false, t = Xn) {
  if (t) {
    let s = ws.get(t);
    s || ws.set(t, s = []), s.push(e);
  }
}
function vu(e, n, t = me) {
  const { immediate: s, deep: a, once: i, scheduler: o, augmentJob: r, call: l } = t, c = (P) => a ? P : Ze(P) || a === false || a === 0 ? Tn(P, 1) : Tn(P);
  let d, p, m, f, k = false, T = false;
  if (Pe(e) ? (p = () => e.value, k = Ze(e)) : nt(e) ? (p = () => c(e), k = true) : Z(e) ? (T = true, k = e.some((P) => nt(P) || Ze(P)), p = () => e.map((P) => {
    if (Pe(P)) return P.value;
    if (nt(P)) return c(P);
    if (ne(P)) return l ? l(P, 2) : P();
  })) : ne(e) ? n ? p = l ? () => l(e, 2) : e : p = () => {
    if (m) {
      xn();
      try {
        m();
      } finally {
        $n();
      }
    }
    const P = Xn;
    Xn = d;
    try {
      return l ? l(e, 3, [f]) : e(f);
    } finally {
      Xn = P;
    }
  } : p = yn, n && a) {
    const P = p, O = a === true ? 1 / 0 : a;
    p = () => Tn(P(), O);
  }
  const W = Kl(), j = () => {
    d.stop(), W && W.active && Ua(W.effects, d);
  };
  if (i && n) {
    const P = n;
    n = (...O) => {
      const G = P(...O);
      return j(), G;
    };
  }
  let $ = T ? new Array(e.length).fill(ps) : ps;
  const A = (P) => {
    if (!(!(d.flags & 1) || !d.dirty && !P)) if (n) {
      const O = d.run();
      if (P || a || k || (T ? O.some((G, N) => _n(G, $[N])) : _n(O, $))) {
        m && m();
        const G = Xn;
        Xn = d;
        try {
          const N = [O, $ === ps ? void 0 : T && $[0] === ps ? [] : $, f];
          $ = O, l ? l(n, 3, N) : n(...N);
        } finally {
          Xn = G;
        }
      }
    } else d.run();
  };
  return r && r(A), d = new Xo(p), d.scheduler = o ? () => o(A, false) : A, f = (P) => _u(P, false, d), m = d.onStop = () => {
    const P = ws.get(d);
    if (P) {
      if (l) l(P, 4);
      else for (const O of P) O();
      ws.delete(d);
    }
  }, n ? s ? A(true) : $ = d.run() : o ? o(A.bind(null, true), true) : d.run(), j.pause = d.pause.bind(d), j.resume = d.resume.bind(d), j.stop = j, j;
}
function Tn(e, n = 1 / 0, t) {
  if (n <= 0 || !pe(e) || e.__v_skip || (t = t || /* @__PURE__ */ new Map(), (t.get(e) || 0) >= n)) return e;
  if (t.set(e, n), n--, Pe(e)) Tn(e.value, n, t);
  else if (Z(e)) for (let s = 0; s < e.length; s++) Tn(e[s], n, t);
  else if (wt(e) || ct(e)) e.forEach((s) => {
    Tn(s, n, t);
  });
  else if (Ko(e)) {
    for (const s in e) Tn(e[s], n, t);
    for (const s of Object.getOwnPropertySymbols(e)) Object.prototype.propertyIsEnumerable.call(e, s) && Tn(e[s], n, t);
  }
  return e;
}
/**
* @vue/runtime-core v3.5.39
* (c) 2018-present Yuxi (Evan) You and Vue contributors
* @license MIT
**/
function ss(e, n, t, s) {
  try {
    return s ? e(...s) : e();
  } catch (a) {
    Us(a, n, t);
  }
}
function on(e, n, t, s) {
  if (ne(e)) {
    const a = ss(e, n, t, s);
    return a && Wo(a) && a.catch((i) => {
      Us(i, n, t);
    }), a;
  }
  if (Z(e)) {
    const a = [];
    for (let i = 0; i < e.length; i++) a.push(on(e[i], n, t, s));
    return a;
  }
}
function Us(e, n, t, s = true) {
  const a = n ? n.vnode : null, { errorHandler: i, throwUnhandledErrorInProduction: o } = n && n.appContext.config || me;
  if (n) {
    let r = n.parent;
    const l = n.proxy, c = `https://vuejs.org/error-reference/#runtime-${t}`;
    for (; r; ) {
      const d = r.ec;
      if (d) {
        for (let p = 0; p < d.length; p++) if (d[p](e, l, c) === false) return;
      }
      r = r.parent;
    }
    if (i) {
      xn(), ss(i, null, 10, [e, l, c]), $n();
      return;
    }
  }
  yu(e, t, a, s, o);
}
function yu(e, n, t, s = true, a = false) {
  if (a) throw e;
  console.error(e);
}
const Me = [];
let fn = -1;
const dt = [];
let Fn = null, ot = 0;
const mr = Promise.resolve();
let bs = null;
function as(e) {
  const n = bs || mr;
  return e ? n.then(this ? e.bind(this) : e) : n;
}
function ku(e) {
  let n = fn + 1, t = Me.length;
  for (; n < t; ) {
    const s = n + t >>> 1, a = Me[s], i = Qt(a);
    i < e || i === e && a.flags & 2 ? n = s + 1 : t = s;
  }
  return n;
}
function Xa(e) {
  if (!(e.flags & 1)) {
    const n = Qt(e), t = Me[Me.length - 1];
    !t || !(e.flags & 2) && n >= Qt(t) ? Me.push(e) : Me.splice(ku(n), 0, e), e.flags |= 1, gr();
  }
}
function gr() {
  bs || (bs = mr.then(vr));
}
function wu(e) {
  Z(e) ? dt.push(...e) : Fn && e.id === -1 ? Fn.splice(ot + 1, 0, e) : e.flags & 1 || (dt.push(e), e.flags |= 1), gr();
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
    const n = [...new Set(dt)].sort((t, s) => Qt(t) - Qt(s));
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
const Qt = (e) => e.id == null ? e.flags & 2 ? -1 : 1 / 0 : e.id;
function vr(e) {
  try {
    for (fn = 0; fn < Me.length; fn++) {
      const n = Me[fn];
      n && !(n.flags & 8) && (n.flags & 4 && (n.flags &= -2), ss(n, n.i, n.i ? 15 : 14), n.flags & 4 || (n.flags &= -2));
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
    i && (ne(i) && (i = { mounted: i, updated: i }), i.deep && Tn(o), s.push({ dir: i, instance: t, value: o, oldValue: void 0, arg: r, modifiers: l }));
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
  if (Te) {
    let t = Te.provides;
    const s = Te.parent && Te.parent.provides;
    s === t && (t = Te.provides = Object.create(s)), t[e] = n;
  }
}
function sn(e, n, t = false) {
  const s = wc();
  if (s || pt) {
    let a = pt ? pt._context.provides : s ? s.parent == null || s.ce ? s.vnode.appContext && s.vnode.appContext.provides : s.parent.provides : void 0;
    if (a && e in a) return a[e];
    if (arguments.length > 1) return t && ne(n) ? n.call(s && s.proxy) : n;
  }
}
const bu = Symbol.for("v-scx"), xu = () => sn(bu);
function kn(e, n, t) {
  return kr(e, n, t);
}
function kr(e, n, t = me) {
  const { immediate: s, deep: a, flush: i, once: o } = t, r = ze({}, t), l = n && s || !n && i !== "post";
  let c;
  if (Zt) {
    if (i === "sync") {
      const f = xu();
      c = f.__watcherHandles || (f.__watcherHandles = []);
    } else if (!l) {
      const f = () => {
      };
      return f.stop = yn, f.resume = yn, f.pause = yn, f;
    }
  }
  const d = Te;
  r.call = (f, k, T) => on(f, d, k, T);
  let p = false;
  i === "post" ? r.scheduler = (f) => {
    Ve(f, d && d.suspense);
  } : i !== "sync" && (p = true, r.scheduler = (f, k) => {
    k ? f() : Xa(f);
  }), r.augmentJob = (f) => {
    n && (f.flags |= 4), p && (f.flags |= 2, d && (f.id = d.uid, f.i = d));
  };
  const m = vu(e, n, r);
  return Zt && (c ? c.push(m) : l && m()), m;
}
function $u(e, n, t) {
  const s = this.proxy, a = xe(e) ? e.includes(".") ? wr(s, e) : () => s[e] : e.bind(s, s);
  let i;
  ne(n) ? i = n : (i = n.handler, t = n);
  const o = os(this), r = kr(a, i.bind(s), t);
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
const ju = Symbol("_vte"), Su = (e) => e.__isTeleport, ta = Symbol("_leaveCb");
function Za(e, n) {
  e.shapeFlag & 6 && e.component ? (e.transition = n, Za(e.component.subTree, n)) : e.shapeFlag & 128 ? (e.ssContent.transition = n.clone(e.ssContent), e.ssFallback.transition = n.clone(e.ssFallback)) : e.transition = n;
}
function De(e, n) {
  return ne(e) ? ze({ name: e.name }, n, { setup: e }) : e;
}
function br(e) {
  e.ids = [e.ids[0] + e.ids[2]++ + "-", 0, 0];
}
function vi(e, n) {
  let t;
  return !!((t = Object.getOwnPropertyDescriptor(e, n)) && !t.configurable);
}
const $s = /* @__PURE__ */ new WeakMap();
function Vt(e, n, t, s, a = false) {
  if (Z(e)) {
    e.forEach((T, W) => Vt(T, n && (Z(n) ? n[W] : n), t, s, a));
    return;
  }
  if (Ht(s) && !a) {
    s.shapeFlag & 512 && s.type.__asyncResolved && s.component.subTree.component && Vt(e, n, t, s.component.subTree);
    return;
  }
  const i = s.shapeFlag & 4 ? Ws(s.component) : s.el, o = a ? null : i, { i: r, r: l } = e, c = n && n.r, d = r.refs === me ? r.refs = {} : r.refs, p = r.setupState, m = ce(p), f = p === me ? Ho : (T) => vi(d, T) ? false : de(m, T), k = (T, W) => !(W && vi(d, W));
  if (c != null && c !== l) {
    if (yi(n), xe(c)) d[c] = null, f(c) && (p[c] = null);
    else if (Pe(c)) {
      const T = n;
      k(c, T.k) && (c.value = null), T.k && (d[T.k] = null);
    }
  }
  if (ne(l)) {
    xn();
    try {
      ss(l, r, 12, [o, d]);
    } finally {
      $n();
    }
  } else {
    const T = xe(l), W = Pe(l);
    if (T || W) {
      const j = () => {
        if (e.f) {
          const $ = T ? f(l) ? p[l] : d[l] : k() || !e.k ? l.value : d[e.k];
          if (a) Z($) && Ua($, i);
          else if (Z($)) $.includes(i) || $.push(i);
          else if (T) d[l] = [i], f(l) && (p[l] = d[l]);
          else {
            const A = [i];
            k(l, e.k) && (l.value = A), e.k && (d[e.k] = A);
          }
        } else T ? (d[l] = o, f(l) && (p[l] = o)) : W && (k(l, e.k) && (l.value = o), e.k && (d[e.k] = o));
      };
      if (o) {
        const $ = () => {
          j(), $s.delete(e);
        };
        $.id = -1, $s.set(e, $), Ve($, t);
      } else yi(e), j();
    }
  }
}
function yi(e) {
  const n = $s.get(e);
  n && (n.flags |= 8, $s.delete(e));
}
Ds().requestIdleCallback;
Ds().cancelIdleCallback;
const Ht = (e) => !!e.type.__asyncLoader, xr = (e) => e.type.__isKeepAlive;
function Eu(e, n) {
  $r(e, "a", n);
}
function Au(e, n) {
  $r(e, "da", n);
}
function $r(e, n, t = Te) {
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
    for (; a && a.parent; ) xr(a.parent.vnode) && Cu(s, n, t, a), a = a.parent;
  }
}
function Cu(e, n, t, s) {
  const a = Fs(n, e, s, true);
  is(() => {
    Ua(s[n], a);
  }, t);
}
function Fs(e, n, t = Te, s = false) {
  if (t) {
    const a = t[e] || (t[e] = []), i = n.__weh || (n.__weh = (...o) => {
      xn();
      const r = os(t), l = on(n, t, e, o);
      return r(), $n(), l;
    });
    return s ? a.unshift(i) : a.push(i), i;
  }
}
const Nn = (e) => (n, t = Te) => {
  (!Zt || e === "sp") && Fs(e, (...s) => n(...s), t);
}, Ru = Nn("bm"), $t = Nn("m"), zu = Nn("bu"), Iu = Nn("u"), Tu = Nn("bum"), is = Nn("um"), Pu = Nn("sp"), Ou = Nn("rtg"), Nu = Nn("rtc");
function qu(e, n = Te) {
  Fs("ec", e, n);
}
const Mu = "directives", Du = Symbol.for("v-ndc");
function Lu(e) {
  return Uu(Mu, e);
}
function Uu(e, n, t = true, s = false) {
  const a = Be || Te;
  if (a) {
    const i = a.type, o = ki(a[e] || i[e], n) || ki(a.appContext[e], n);
    return !o && s ? i : o;
  }
}
function ki(e, n) {
  return e && (e[n] || e[Ge(n)] || e[Va(Ge(n))]);
}
function ge(e, n, t, s) {
  let a;
  const i = t, o = Z(e);
  if (o || xe(e)) {
    const r = o && nt(e);
    let l = false, c = false;
    r && (l = !Ze(e), c = On(e), e = Ls(e)), a = new Array(e.length);
    for (let d = 0, p = e.length; d < p; d++) a[d] = n(l ? c ? ft(an(e[d])) : an(e[d]) : e[d], d, void 0, i);
  } else if (typeof e == "number") {
    a = new Array(e);
    for (let r = 0; r < e; r++) a[r] = n(r + 1, r, void 0, i);
  } else if (pe(e)) if (e[Symbol.iterator]) a = Array.from(e, (r, l) => n(r, l, void 0, i));
  else {
    const r = Object.keys(e);
    a = new Array(r.length);
    for (let l = 0, c = r.length; l < c; l++) {
      const d = r[l];
      a[l] = n(e[d], d, l, i);
    }
  }
  else a = [];
  return a;
}
const ya = (e) => e ? Wr(e) ? Ws(e) : ya(e.parent) : null, Wt = ze(/* @__PURE__ */ Object.create(null), { $: (e) => e, $el: (e) => e.vnode.el, $data: (e) => e.data, $props: (e) => e.props, $attrs: (e) => e.attrs, $slots: (e) => e.slots, $refs: (e) => e.refs, $parent: (e) => ya(e.parent), $root: (e) => ya(e.root), $host: (e) => e.ce, $emit: (e) => e.emit, $options: (e) => Sr(e), $forceUpdate: (e) => e.f || (e.f = () => {
  Xa(e.update);
}), $nextTick: (e) => e.n || (e.n = as.bind(e.proxy)), $watch: (e) => $u.bind(e) }), sa = (e, n) => e !== me && !e.__isScriptSetup && de(e, n), Fu = { get({ _: e }, n) {
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
  const c = Wt[n];
  let d, p;
  if (c) return n === "$attrs" && Ie(e.attrs, "get", ""), c(e);
  if ((d = r.__cssModules) && (d = d[n])) return d;
  if (t !== me && de(t, n)) return o[n] = 4, t[n];
  if (p = l.config.globalProperties, de(p, n)) return p[n];
}, set({ _: e }, n, t) {
  const { data: s, setupState: a, ctx: i } = e;
  return sa(a, n) ? (a[n] = t, true) : s !== me && de(s, n) ? (s[n] = t, true) : de(e.props, n) || n[0] === "$" && n.slice(1) in e ? false : (i[n] = t, true);
}, has({ _: { data: e, setupState: n, accessCache: t, ctx: s, appContext: a, props: i, type: o } }, r) {
  let l;
  return !!(t[r] || e !== me && r[0] !== "$" && de(e, r) || sa(n, r) || de(i, r) || de(s, r) || de(Wt, r) || de(a.config.globalProperties, r) || (l = o.__cssModules) && l[r]);
}, defineProperty(e, n, t) {
  return t.get != null ? e._.accessCache[n] = 0 : de(t, "value") && this.set(e, n, t.value, null), Reflect.defineProperty(e, n, t);
} };
function wi(e) {
  return Z(e) ? e.reduce((n, t) => (n[t] = null, n), {}) : e;
}
let ka = true;
function Vu(e) {
  const n = Sr(e), t = e.proxy, s = e.ctx;
  ka = false, n.beforeCreate && bi(n.beforeCreate, e, "bc");
  const { data: a, computed: i, methods: o, watch: r, provide: l, inject: c, created: d, beforeMount: p, mounted: m, beforeUpdate: f, updated: k, activated: T, deactivated: W, beforeDestroy: j, beforeUnmount: $, destroyed: A, unmounted: P, render: O, renderTracked: G, renderTriggered: N, errorCaptured: ke, serverPrefetch: Ee, expose: Se, inheritAttrs: Ye, components: He, directives: Le, filters: en } = n;
  if (c && Hu(c, s, null), o) for (const re in o) {
    const ie = o[re];
    ne(ie) && (s[re] = ie.bind(t));
  }
  if (a) {
    const re = a.call(t, t);
    pe(re) && (e.data = xt(re));
  }
  if (ka = true, i) for (const re in i) {
    const ie = i[re], Oe = ne(ie) ? ie.bind(t, t) : ne(ie.get) ? ie.get.bind(t, t) : yn, q = !ne(ie) && ne(ie.set) ? ie.set.bind(t) : yn, F = X({ get: Oe, set: q });
    Object.defineProperty(s, re, { enumerable: true, configurable: true, get: () => F.value, set: (J) => F.value = J });
  }
  if (r) for (const re in r) jr(r[re], s, t, re);
  if (l) {
    const re = ne(l) ? l.call(t) : l;
    Reflect.ownKeys(re).forEach((ie) => {
      _s(ie, re[ie]);
    });
  }
  d && bi(d, e, "c");
  function we(re, ie) {
    Z(ie) ? ie.forEach((Oe) => re(Oe.bind(t))) : ie && re(ie.bind(t));
  }
  if (we(Ru, p), we($t, m), we(zu, f), we(Iu, k), we(Eu, T), we(Au, W), we(qu, ke), we(Nu, G), we(Ou, N), we(Tu, $), we(is, P), we(Pu, Ee), Z(Se)) if (Se.length) {
    const re = e.exposed || (e.exposed = {});
    Se.forEach((ie) => {
      Object.defineProperty(re, ie, { get: () => t[ie], set: (Oe) => t[ie] = Oe, enumerable: true });
    });
  } else e.exposed || (e.exposed = {});
  O && e.render === yn && (e.render = O), Ye != null && (e.inheritAttrs = Ye), He && (e.components = He), Le && (e.directives = Le), Ee && br(e);
}
function Hu(e, n, t = yn) {
  Z(e) && (e = wa(e));
  for (const s in e) {
    const a = e[s];
    let i;
    pe(a) ? "default" in a ? i = sn(a.from || s, a.default, true) : i = sn(a.from || s) : i = sn(a), Pe(i) ? Object.defineProperty(n, s, { enumerable: true, configurable: true, get: () => i.value, set: (o) => i.value = o }) : n[s] = i;
  }
}
function bi(e, n, t) {
  on(Z(e) ? e.map((s) => s.bind(n.proxy)) : e.bind(n.proxy), n, t);
}
function jr(e, n, t, s) {
  let a = s.includes(".") ? wr(t, s) : () => t[s];
  if (xe(e)) {
    const i = n[e];
    ne(i) && kn(a, i);
  } else if (ne(e)) kn(a, e.bind(t));
  else if (pe(e)) if (Z(e)) e.forEach((i) => jr(i, n, t, s));
  else {
    const i = ne(e.handler) ? e.handler.bind(t) : n[e.handler];
    ne(i) && kn(a, i, e);
  }
}
function Sr(e) {
  const n = e.type, { mixins: t, extends: s } = n, { mixins: a, optionsCache: i, config: { optionMergeStrategies: o } } = e.appContext, r = i.get(n);
  let l;
  return r ? l = r : !a.length && !t && !s ? l = n : (l = {}, a.length && a.forEach((c) => js(l, c, o, true)), js(l, n, o)), pe(n) && i.set(n, l), l;
}
function js(e, n, t, s = false) {
  const { mixins: a, extends: i } = n;
  i && js(e, i, t, true), a && a.forEach((o) => js(e, o, t, true));
  for (const o in n) if (!(s && o === "expose")) {
    const r = Wu[o] || t && t[o];
    e[o] = r ? r(e[o], n[o]) : n[o];
  }
  return e;
}
const Wu = { data: xi, props: $i, emits: $i, methods: Pt, computed: Pt, beforeCreate: Ne, created: Ne, beforeMount: Ne, mounted: Ne, beforeUpdate: Ne, updated: Ne, beforeDestroy: Ne, beforeUnmount: Ne, destroyed: Ne, unmounted: Ne, activated: Ne, deactivated: Ne, errorCaptured: Ne, serverPrefetch: Ne, components: Pt, directives: Pt, watch: Ku, provide: xi, inject: Bu };
function xi(e, n) {
  return n ? e ? function() {
    return ze(ne(e) ? e.call(this, this) : e, ne(n) ? n.call(this, this) : n);
  } : n : e;
}
function Bu(e, n) {
  return Pt(wa(e), wa(n));
}
function wa(e) {
  if (Z(e)) {
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
  return e ? Z(e) && Z(n) ? [.../* @__PURE__ */ new Set([...e, ...n])] : ze(/* @__PURE__ */ Object.create(null), wi(e), wi(n ?? {})) : n;
}
function Ku(e, n) {
  if (!e) return n;
  if (!n) return e;
  const t = ze(/* @__PURE__ */ Object.create(null), e);
  for (const s in n) t[s] = Ne(e[s], n[s]);
  return t;
}
function Er() {
  return { app: null, config: { isNativeTag: Ho, performance: false, globalProperties: {}, optionMergeStrategies: {}, errorHandler: void 0, warnHandler: void 0, compilerOptions: {} }, mixins: [], components: {}, directives: {}, provides: /* @__PURE__ */ Object.create(null), optionsCache: /* @__PURE__ */ new WeakMap(), propsCache: /* @__PURE__ */ new WeakMap(), emitsCache: /* @__PURE__ */ new WeakMap() };
}
let Gu = 0;
function Ju(e, n) {
  return function(s, a = null) {
    ne(s) || (s = ze({}, s)), a != null && !pe(a) && (a = null);
    const i = Er(), o = /* @__PURE__ */ new WeakSet(), r = [];
    let l = false;
    const c = i.app = { _uid: Gu++, _component: s, _props: a, _container: null, _context: i, _instance: null, version: Ec, get config() {
      return i.config;
    }, set config(d) {
    }, use(d, ...p) {
      return o.has(d) || (d && ne(d.install) ? (o.add(d), d.install(c, ...p)) : ne(d) && (o.add(d), d(c, ...p))), c;
    }, mixin(d) {
      return i.mixins.includes(d) || i.mixins.push(d), c;
    }, component(d, p) {
      return p ? (i.components[d] = p, c) : i.components[d];
    }, directive(d, p) {
      return p ? (i.directives[d] = p, c) : i.directives[d];
    }, mount(d, p, m) {
      if (!l) {
        const f = c._ceVNode || ve(s, a);
        return f.appContext = i, m === true ? m = "svg" : m === false && (m = void 0), e(f, d, m), l = true, c._container = d, d.__vue_app__ = c, Ws(f.component);
      }
    }, onUnmount(d) {
      r.push(d);
    }, unmount() {
      l && (on(r, c._instance, 16), e(null, c._container), delete c._container.__vue_app__);
    }, provide(d, p) {
      return i.provides[d] = p, c;
    }, runWithContext(d) {
      const p = pt;
      pt = c;
      try {
        return d();
      } finally {
        pt = p;
      }
    } };
    return c;
  };
}
let pt = null;
const Yu = (e, n) => n === "modelValue" || n === "model-value" ? e.modelModifiers : e[`${n}Modifiers`] || e[`${Ge(n)}Modifiers`] || e[`${Jn(n)}Modifiers`];
function Qu(e, n, ...t) {
  if (e.isUnmounted) return;
  const s = e.vnode.props || me;
  let a = t;
  const i = n.startsWith("update:"), o = i && Yu(s, n.slice(7));
  o && (o.trim && (a = t.map((d) => xe(d) ? d.trim() : d)), o.number && (a = t.map(Ms)));
  let r, l = s[r = Qs(n)] || s[r = Qs(Ge(n))];
  !l && i && (l = s[r = Qs(Jn(n))]), l && on(l, e, 6, a);
  const c = s[r + "Once"];
  if (c) {
    if (!e.emitted) e.emitted = {};
    else if (e.emitted[r]) return;
    e.emitted[r] = true, on(c, e, 6, a);
  }
}
const Xu = /* @__PURE__ */ new WeakMap();
function Ar(e, n, t = false) {
  const s = t ? Xu : n.emitsCache, a = s.get(e);
  if (a !== void 0) return a;
  const i = e.emits;
  let o = {}, r = false;
  if (!ne(e)) {
    const l = (c) => {
      const d = Ar(c, n, true);
      d && (r = true, ze(o, d));
    };
    !t && n.mixins.length && n.mixins.forEach(l), e.extends && l(e.extends), e.mixins && e.mixins.forEach(l);
  }
  return !i && !r ? (pe(e) && s.set(e, null), null) : (Z(i) ? i.forEach((l) => o[l] = null) : ze(o, i), pe(e) && s.set(e, o), o);
}
function Vs(e, n) {
  return !e || !Os(n) ? false : (n = n.slice(2), n = n === "Once" ? n : n.replace(/Once$/, ""), de(e, n[0].toLowerCase() + n.slice(1)) || de(e, Jn(n)) || de(e, n));
}
function ji(e) {
  const { type: n, vnode: t, proxy: s, withProxy: a, propsOptions: [i], slots: o, attrs: r, emit: l, render: c, renderCache: d, props: p, data: m, setupState: f, ctx: k, inheritAttrs: T } = e, W = xs(e);
  let j, $;
  try {
    if (t.shapeFlag & 4) {
      const P = a || s, O = P;
      j = gn(c.call(O, P, d, p, f, m, k)), $ = r;
    } else {
      const P = n;
      j = gn(P.length > 1 ? P(p, { attrs: r, slots: o, emit: l }) : P(p, null)), $ = n.props ? r : Zu(r);
    }
  } catch (P) {
    Bt.length = 0, Us(P, e, 1), j = ve(Gn);
  }
  let A = j;
  if ($ && T !== false) {
    const P = Object.keys($), { shapeFlag: O } = A;
    P.length && O & 7 && (i && P.some(Ns) && ($ = ec($, i)), A = mt(A, $, false, true));
  }
  return t.dirs && (A = mt(A, null, false, true), A.dirs = A.dirs ? A.dirs.concat(t.dirs) : t.dirs), t.transition && Za(A, t.transition), j = A, xs(W), j;
}
const Zu = (e) => {
  let n;
  for (const t in e) (t === "class" || t === "style" || Os(t)) && ((n || (n = {}))[t] = e[t]);
  return n;
}, ec = (e, n) => {
  const t = {};
  for (const s in e) (!Ns(s) || !(s.slice(9) in n)) && (t[s] = e[s]);
  return t;
};
function nc(e, n, t) {
  const { props: s, children: a, component: i } = e, { props: o, children: r, patchFlag: l } = n, c = i.emitsOptions;
  if (n.dirs || n.transition) return true;
  if (t && l >= 0) {
    if (l & 1024) return true;
    if (l & 16) return s ? Si(s, o, c) : !!o;
    if (l & 8) {
      const d = n.dynamicProps;
      for (let p = 0; p < d.length; p++) {
        const m = d[p];
        if (Cr(o, s, m) && !Vs(c, m)) return true;
      }
    }
  } else return (a || r) && (!r || !r.$stable) ? true : s === o ? false : s ? o ? Si(s, o, c) : true : !!o;
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
function tc({ vnode: e, parent: n, suspense: t }, s) {
  for (; n; ) {
    const a = n.subTree;
    if (a.suspense && a.suspense.activeBranch === e && (a.suspense.vnode.el = a.el = s, e = a), a === e) (e = n.vnode).el = s, n = n.parent;
    else break;
  }
  t && t.activeBranch === e && (t.vnode.el = s);
}
const Rr = {}, zr = () => Object.create(Rr), Ir = (e) => Object.getPrototypeOf(e) === Rr;
function sc(e, n, t, s = false) {
  const a = {}, i = zr();
  e.propsDefaults = /* @__PURE__ */ Object.create(null), Tr(e, n, a, i);
  for (const o in e.propsOptions[0]) o in a || (a[o] = void 0);
  t ? e.props = s ? a : pr(a) : e.type.props ? e.props = a : e.props = i, e.attrs = i;
}
function ac(e, n, t, s) {
  const { props: a, attrs: i, vnode: { patchFlag: o } } = e, r = ce(a), [l] = e.propsOptions;
  let c = false;
  if ((s || o > 0) && !(o & 16)) {
    if (o & 8) {
      const d = e.vnode.dynamicProps;
      for (let p = 0; p < d.length; p++) {
        let m = d[p];
        if (Vs(e.emitsOptions, m)) continue;
        const f = n[m];
        if (l) if (de(i, m)) f !== i[m] && (i[m] = f, c = true);
        else {
          const k = Ge(m);
          a[k] = ba(l, r, k, f, e, false);
        }
        else f !== i[m] && (i[m] = f, c = true);
      }
    }
  } else {
    Tr(e, n, a, i) && (c = true);
    let d;
    for (const p in r) (!n || !de(n, p) && ((d = Jn(p)) === p || !de(n, d))) && (l ? t && (t[p] !== void 0 || t[d] !== void 0) && (a[p] = ba(l, r, p, void 0, e, true)) : delete a[p]);
    if (i !== r) for (const p in i) (!n || !de(n, p)) && (delete i[p], c = true);
  }
  c && In(e.attrs, "set", "");
}
function Tr(e, n, t, s) {
  const [a, i] = e.propsOptions;
  let o = false, r;
  if (n) for (let l in n) {
    if (Lt(l)) continue;
    const c = n[l];
    let d;
    a && de(a, d = Ge(l)) ? !i || !i.includes(d) ? t[d] = c : (r || (r = {}))[d] = c : Vs(e.emitsOptions, l) || (!(l in s) || c !== s[l]) && (s[l] = c, o = true);
  }
  if (i) {
    const l = ce(t), c = r || me;
    for (let d = 0; d < i.length; d++) {
      const p = i[d];
      t[p] = ba(a, l, p, c[p], e, !de(c, p));
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
      if (o.type !== Function && !o.skipFactory && ne(l)) {
        const { propsDefaults: c } = a;
        if (t in c) s = c[t];
        else {
          const d = os(a);
          s = c[t] = l.call(null, n), d();
        }
      } else s = l;
      a.ce && a.ce._setProp(t, s);
    }
    o[0] && (i && !r ? s = false : o[1] && (s === "" || s === Jn(t)) && (s = true));
  }
  return s;
}
const ic = /* @__PURE__ */ new WeakMap();
function Pr(e, n, t = false) {
  const s = t ? ic : n.propsCache, a = s.get(e);
  if (a) return a;
  const i = e.props, o = {}, r = [];
  let l = false;
  if (!ne(e)) {
    const d = (p) => {
      l = true;
      const [m, f] = Pr(p, n, true);
      ze(o, m), f && r.push(...f);
    };
    !t && n.mixins.length && n.mixins.forEach(d), e.extends && d(e.extends), e.mixins && e.mixins.forEach(d);
  }
  if (!i && !l) return pe(e) && s.set(e, ut), ut;
  if (Z(i)) for (let d = 0; d < i.length; d++) {
    const p = Ge(i[d]);
    Ei(p) && (o[p] = me);
  }
  else if (i) for (const d in i) {
    const p = Ge(d);
    if (Ei(p)) {
      const m = i[d], f = o[p] = Z(m) || ne(m) ? { type: m } : ze({}, m), k = f.type;
      let T = false, W = true;
      if (Z(k)) for (let j = 0; j < k.length; ++j) {
        const $ = k[j], A = ne($) && $.name;
        if (A === "Boolean") {
          T = true;
          break;
        } else A === "String" && (W = false);
      }
      else T = ne(k) && k.name === "Boolean";
      f[0] = T, f[1] = W, (T || de(f, "default")) && r.push(p);
    }
  }
  const c = [o, r];
  return pe(e) && s.set(e, c), c;
}
function Ei(e) {
  return e[0] !== "$" && !Lt(e);
}
const ei = (e) => e === "_" || e === "_ctx" || e === "$stable", ni = (e) => Z(e) ? e.map(gn) : [gn(e)], oc = (e, n, t) => {
  if (n._n) return n;
  const s = Ln((...a) => ni(n(...a)), t);
  return s._c = false, s;
}, Or = (e, n, t) => {
  const s = e._ctx;
  for (const a in e) {
    if (ei(a)) continue;
    const i = e[a];
    if (ne(i)) n[a] = oc(a, i, s);
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
}, rc = (e, n, t) => {
  const s = e.slots = zr();
  if (e.vnode.shapeFlag & 32) {
    const a = n._;
    a ? (qr(s, n, t), t && Go(s, "_", a, true)) : Or(n, s);
  } else n && Nr(e, n);
}, lc = (e, n, t) => {
  const { vnode: s, slots: a } = e;
  let i = true, o = me;
  if (s.shapeFlag & 32) {
    const r = n._;
    r ? t && r === 1 ? i = false : qr(a, n, t) : (i = !n.$stable, Or(n, a)), o = n;
  } else n && (Nr(e, n), o = { default: 1 });
  if (i) for (const r in a) !ei(r) && o[r] == null && delete a[r];
}, Ve = hc;
function uc(e) {
  return cc(e);
}
function cc(e, n) {
  const t = Ds();
  t.__VUE__ = true;
  const { insert: s, remove: a, patchProp: i, createElement: o, createText: r, createComment: l, setText: c, setElementText: d, parentNode: p, nextSibling: m, setScopeId: f = yn, insertStaticContent: k } = e, T = (h, g, y, R = null, I = null, C = null, M = void 0, _ = null, v = !!g.dynamicChildren) => {
    if (h === g) return;
    h && !At(h, g) && (R = z(h), J(h, I, C, true), h = null), g.patchFlag === -2 && (v = false, g.dynamicChildren = null);
    const { type: E, ref: S, shapeFlag: D } = g;
    switch (E) {
      case Hs:
        W(h, g, y, R);
        break;
      case Gn:
        j(h, g, y, R);
        break;
      case vs:
        h == null && $(g, y, R, M);
        break;
      case te:
        He(h, g, y, R, I, C, M, _, v);
        break;
      default:
        D & 1 ? O(h, g, y, R, I, C, M, _, v) : D & 6 ? Le(h, g, y, R, I, C, M, _, v) : (D & 64 || D & 128) && E.process(h, g, y, R, I, C, M, _, v, K);
    }
    S != null && I ? Vt(S, h && h.ref, C, g || h, !g) : S == null && h && h.ref != null && Vt(h.ref, null, C, h, true);
  }, W = (h, g, y, R) => {
    if (h == null) s(g.el = r(g.children), y, R);
    else {
      const I = g.el = h.el;
      g.children !== h.children && c(I, g.children);
    }
  }, j = (h, g, y, R) => {
    h == null ? s(g.el = l(g.children || ""), y, R) : g.el = h.el;
  }, $ = (h, g, y, R) => {
    [h.el, h.anchor] = k(h.children, g, y, R, h.el, h.anchor);
  }, A = ({ el: h, anchor: g }, y, R) => {
    let I;
    for (; h && h !== g; ) I = m(h), s(h, y, R), h = I;
    s(g, y, R);
  }, P = ({ el: h, anchor: g }) => {
    let y;
    for (; h && h !== g; ) y = m(h), a(h), h = y;
    a(g);
  }, O = (h, g, y, R, I, C, M, _, v) => {
    if (g.type === "svg" ? M = "svg" : g.type === "math" && (M = "mathml"), h == null) G(g, y, R, I, C, M, _, v);
    else {
      const E = h.el && h.el._isVueCE ? h.el : null;
      try {
        E && E._beginPatch(), Ee(h, g, I, C, M, _, v);
      } finally {
        E && E._endPatch();
      }
    }
  }, G = (h, g, y, R, I, C, M, _) => {
    let v, E;
    const { props: S, shapeFlag: D, transition: Y, dirs: ee } = h;
    if (v = h.el = o(h.type, C, S && S.is, S), D & 8 ? d(v, h.children) : D & 16 && ke(h.children, v, null, R, I, aa(h, C), M, _), ee && Yn(h, null, R, "created"), N(v, h, h.scopeId, M, R), S) {
      for (const fe in S) fe !== "value" && !Lt(fe) && i(v, fe, null, S[fe], C, R);
      "value" in S && i(v, "value", null, S.value, C), (E = S.onVnodeBeforeMount) && hn(E, R, h);
    }
    ee && Yn(h, null, R, "beforeMount");
    const le = dc(I, Y);
    le && Y.beforeEnter(v), s(v, g, y), ((E = S && S.onVnodeMounted) || le || ee) && Ve(() => {
      try {
        E && hn(E, R, h), le && Y.enter(v), ee && Yn(h, null, R, "mounted");
      } finally {
      }
    }, I);
  }, N = (h, g, y, R, I) => {
    if (y && f(h, y), R) for (let C = 0; C < R.length; C++) f(h, R[C]);
    if (I) {
      let C = I.subTree;
      if (g === C || Ur(C.type) && (C.ssContent === g || C.ssFallback === g)) {
        const M = I.vnode;
        N(h, M, M.scopeId, M.slotScopeIds, I.parent);
      }
    }
  }, ke = (h, g, y, R, I, C, M, _, v = 0) => {
    for (let E = v; E < h.length; E++) {
      const S = h[E] = _ ? zn(h[E]) : gn(h[E]);
      T(null, S, g, y, R, I, C, M, _);
    }
  }, Ee = (h, g, y, R, I, C, M) => {
    const _ = g.el = h.el;
    let { patchFlag: v, dynamicChildren: E, dirs: S } = g;
    v |= h.patchFlag & 16;
    const D = h.props || me, Y = g.props || me;
    let ee;
    if (y && Qn(y, false), (ee = Y.onVnodeBeforeUpdate) && hn(ee, y, g, h), S && Yn(g, h, y, "beforeUpdate"), y && Qn(y, true), E && (!h.dynamicChildren || h.dynamicChildren.length !== E.length) && (v = 0, M = false, E = null), (D.innerHTML && Y.innerHTML == null || D.textContent && Y.textContent == null) && d(_, ""), E ? Se(h.dynamicChildren, E, _, y, R, aa(g, I), C) : M || ie(h, g, _, null, y, R, aa(g, I), C, false), v > 0) {
      if (v & 16) Ye(_, D, Y, y, I);
      else if (v & 2 && D.class !== Y.class && i(_, "class", null, Y.class, I), v & 4 && i(_, "style", D.style, Y.style, I), v & 8) {
        const le = g.dynamicProps;
        for (let fe = 0; fe < le.length; fe++) {
          const he = le[fe], $e = D[he], Ae = Y[he];
          (Ae !== $e || he === "value") && i(_, he, $e, Ae, I, y);
        }
      }
      v & 1 && h.children !== g.children && d(_, g.children);
    } else !M && E == null && Ye(_, D, Y, y, I);
    ((ee = Y.onVnodeUpdated) || S) && Ve(() => {
      ee && hn(ee, y, g, h), S && Yn(g, h, y, "updated");
    }, R);
  }, Se = (h, g, y, R, I, C, M) => {
    for (let _ = 0; _ < g.length; _++) {
      const v = h[_], E = g[_], S = v.el && (v.type === te || !At(v, E) || v.shapeFlag & 198) ? p(v.el) : y;
      T(v, E, S, null, R, I, C, M, true);
    }
  }, Ye = (h, g, y, R, I) => {
    if (g !== y) {
      if (g !== me) for (const C in g) !Lt(C) && !(C in y) && i(h, C, g[C], null, I, R);
      for (const C in y) {
        if (Lt(C)) continue;
        const M = y[C], _ = g[C];
        M !== _ && C !== "value" && i(h, C, _, M, I, R);
      }
      "value" in y && i(h, "value", g.value, y.value, I);
    }
  }, He = (h, g, y, R, I, C, M, _, v) => {
    const E = g.el = h ? h.el : r(""), S = g.anchor = h ? h.anchor : r("");
    let { patchFlag: D, dynamicChildren: Y, slotScopeIds: ee } = g;
    ee && (_ = _ ? _.concat(ee) : ee), h == null ? (s(E, y, R), s(S, y, R), ke(g.children || [], y, S, I, C, M, _, v)) : D > 0 && D & 64 && Y && h.dynamicChildren && h.dynamicChildren.length === Y.length ? (Se(h.dynamicChildren, Y, y, I, C, M, _), (g.key != null || I && g === I.subTree) && Mr(h, g, true)) : ie(h, g, y, S, I, C, M, _, v);
  }, Le = (h, g, y, R, I, C, M, _, v) => {
    g.slotScopeIds = _, h == null ? g.shapeFlag & 512 ? I.ctx.activate(g, y, R, M, v) : en(g, y, R, I, C, M, v) : un(h, g, v);
  }, en = (h, g, y, R, I, C, M) => {
    const _ = h.component = kc(h, R, I);
    if (xr(h) && (_.ctx.renderer = K), bc(_, false, M), _.asyncDep) {
      if (I && I.registerDep(_, we, M), !h.el) {
        const v = _.subTree = ve(Gn);
        j(null, v, g, y), h.placeholder = v.el;
      }
    } else we(_, h, g, y, I, C, M);
  }, un = (h, g, y) => {
    const R = g.component = h.component;
    if (nc(h, g, y)) if (R.asyncDep && !R.asyncResolved) {
      re(R, g, y);
      return;
    } else R.next = g, R.update();
    else g.el = h.el, R.vnode = g;
  }, we = (h, g, y, R, I, C, M) => {
    const _ = () => {
      if (h.isMounted) {
        let { next: D, bu: Y, u: ee, parent: le, vnode: fe } = h;
        {
          const dn = Dr(h);
          if (dn) {
            D && (D.el = fe.el, re(h, D, M)), dn.asyncDep.then(() => {
              Ve(() => {
                h.isUnmounted || E();
              }, I);
            });
            return;
          }
        }
        let he = D, $e;
        Qn(h, false), D ? (D.el = fe.el, re(h, D, M)) : D = fe, Y && gs(Y), ($e = D.props && D.props.onVnodeBeforeUpdate) && hn($e, le, D, fe), Qn(h, true);
        const Ae = ji(h), cn = h.subTree;
        h.subTree = Ae, T(cn, Ae, p(cn.el), z(cn), h, I, C), D.el = Ae.el, he === null && tc(h, Ae.el), ee && Ve(ee, I), ($e = D.props && D.props.onVnodeUpdated) && Ve(() => hn($e, le, D, fe), I);
      } else {
        let D;
        const { el: Y, props: ee } = g, { bm: le, m: fe, parent: he, root: $e, type: Ae } = h, cn = Ht(g);
        Qn(h, false), le && gs(le), !cn && (D = ee && ee.onVnodeBeforeMount) && hn(D, he, g), Qn(h, true);
        {
          $e.ce && $e.ce._hasShadowRoot() && $e.ce._injectChildStyle(Ae, h.parent ? h.parent.type : void 0);
          const dn = h.subTree = ji(h);
          T(null, dn, y, R, h, I, C), g.el = dn.el;
        }
        if (fe && Ve(fe, I), !cn && (D = ee && ee.onVnodeMounted)) {
          const dn = g;
          Ve(() => hn(D, he, dn), I);
        }
        (g.shapeFlag & 256 || he && Ht(he.vnode) && he.vnode.shapeFlag & 256) && h.a && Ve(h.a, I), h.isMounted = true, g = y = R = null;
      }
    };
    h.scope.on();
    const v = h.effect = new Xo(_);
    h.scope.off();
    const E = h.update = v.run.bind(v), S = h.job = v.runIfDirty.bind(v);
    S.i = h, S.id = h.uid, v.scheduler = () => Xa(S), Qn(h, true), E();
  }, re = (h, g, y) => {
    g.component = h;
    const R = h.vnode.props;
    h.vnode = g, h.next = null, ac(h, g.props, R, y), lc(h, g.children, y), xn(), _i(h), $n();
  }, ie = (h, g, y, R, I, C, M, _, v = false) => {
    const E = h && h.children, S = h ? h.shapeFlag : 0, D = g.children, { patchFlag: Y, shapeFlag: ee } = g;
    if (Y > 0) {
      if (Y & 128) {
        q(E, D, y, R, I, C, M, _, v);
        return;
      } else if (Y & 256) {
        Oe(E, D, y, R, I, C, M, _, v);
        return;
      }
    }
    ee & 8 ? (S & 16 && Fe(E, I, C), D !== E && d(y, D)) : S & 16 ? ee & 16 ? q(E, D, y, R, I, C, M, _, v) : Fe(E, I, C, true) : (S & 8 && d(y, ""), ee & 16 && ke(D, y, R, I, C, M, _, v));
  }, Oe = (h, g, y, R, I, C, M, _, v) => {
    h = h || ut, g = g || ut;
    const E = h.length, S = g.length, D = Math.min(E, S);
    let Y;
    for (Y = 0; Y < D; Y++) {
      const ee = g[Y] = v ? zn(g[Y]) : gn(g[Y]);
      T(h[Y], ee, y, null, I, C, M, _, v);
    }
    E > S ? Fe(h, I, C, true, false, D) : ke(g, y, R, I, C, M, _, v, D);
  }, q = (h, g, y, R, I, C, M, _, v) => {
    let E = 0;
    const S = g.length;
    let D = h.length - 1, Y = S - 1;
    for (; E <= D && E <= Y; ) {
      const ee = h[E], le = g[E] = v ? zn(g[E]) : gn(g[E]);
      if (At(ee, le)) T(ee, le, y, null, I, C, M, _, v);
      else break;
      E++;
    }
    for (; E <= D && E <= Y; ) {
      const ee = h[D], le = g[Y] = v ? zn(g[Y]) : gn(g[Y]);
      if (At(ee, le)) T(ee, le, y, null, I, C, M, _, v);
      else break;
      D--, Y--;
    }
    if (E > D) {
      if (E <= Y) {
        const ee = Y + 1, le = ee < S ? g[ee].el : R;
        for (; E <= Y; ) T(null, g[E] = v ? zn(g[E]) : gn(g[E]), y, le, I, C, M, _, v), E++;
      }
    } else if (E > Y) for (; E <= D; ) J(h[E], I, C, true), E++;
    else {
      const ee = E, le = E, fe = /* @__PURE__ */ new Map();
      for (E = le; E <= Y; E++) {
        const We = g[E] = v ? zn(g[E]) : gn(g[E]);
        We.key != null && fe.set(We.key, E);
      }
      let he, $e = 0;
      const Ae = Y - le + 1;
      let cn = false, dn = 0;
      const St = new Array(Ae);
      for (E = 0; E < Ae; E++) St[E] = 0;
      for (E = ee; E <= D; E++) {
        const We = h[E];
        if ($e >= Ae) {
          J(We, I, C, true);
          continue;
        }
        let pn;
        if (We.key != null) pn = fe.get(We.key);
        else for (he = le; he <= Y; he++) if (St[he - le] === 0 && At(We, g[he])) {
          pn = he;
          break;
        }
        pn === void 0 ? J(We, I, C, true) : (St[pn - le] = E + 1, pn >= dn ? dn = pn : cn = true, T(We, g[pn], y, null, I, C, M, _, v), $e++);
      }
      const ci = cn ? pc(St) : ut;
      for (he = ci.length - 1, E = Ae - 1; E >= 0; E--) {
        const We = le + E, pn = g[We], di = g[We + 1], pi = We + 1 < S ? di.el || Lr(di) : R;
        St[E] === 0 ? T(null, pn, y, pi, I, C, M, _, v) : cn && (he < 0 || E !== ci[he] ? F(pn, y, pi, 2) : he--);
      }
    }
  }, F = (h, g, y, R, I = null) => {
    const { el: C, type: M, transition: _, children: v, shapeFlag: E } = h;
    if (E & 6) {
      F(h.component.subTree, g, y, R);
      return;
    }
    if (E & 128) {
      h.suspense.move(g, y, R);
      return;
    }
    if (E & 64) {
      M.move(h, g, y, K);
      return;
    }
    if (M === te) {
      s(C, g, y);
      for (let D = 0; D < v.length; D++) F(v[D], g, y, R);
      s(h.anchor, g, y);
      return;
    }
    if (M === vs) {
      A(h, g, y);
      return;
    }
    if (R !== 2 && E & 1 && _) if (R === 0) _.persisted && !C[ta] ? s(C, g, y) : (_.beforeEnter(C), s(C, g, y), Ve(() => _.enter(C), I));
    else {
      const { leave: D, delayLeave: Y, afterLeave: ee } = _, le = () => {
        h.ctx.isUnmounted ? a(C) : s(C, g, y);
      }, fe = () => {
        const he = C._isLeaving || !!C[ta];
        C._isLeaving && C[ta](true), _.persisted && !he ? le() : D(C, () => {
          le(), ee && ee();
        });
      };
      Y ? Y(C, le, fe) : fe();
    }
    else s(C, g, y);
  }, J = (h, g, y, R = false, I = false) => {
    const { type: C, props: M, ref: _, children: v, dynamicChildren: E, shapeFlag: S, patchFlag: D, dirs: Y, cacheIndex: ee, memo: le } = h;
    if (D === -2 && (I = false), _ != null && (xn(), Vt(_, null, y, h, true), $n()), ee != null && (g.renderCache[ee] = void 0), S & 256) {
      g.ctx.deactivate(h);
      return;
    }
    const fe = S & 1 && Y, he = !Ht(h);
    let $e;
    if (he && ($e = M && M.onVnodeBeforeUnmount) && hn($e, g, h), S & 6) Sn(h.component, y, R);
    else {
      if (S & 128) {
        h.suspense.unmount(y, R);
        return;
      }
      fe && Yn(h, null, g, "beforeUnmount"), S & 64 ? h.type.remove(h, g, y, K, R) : E && !E.hasOnce && (C !== te || D > 0 && D & 64) ? Fe(E, g, y, false, true) : (C === te && D & 384 || !I && S & 16) && Fe(v, g, y), R && oe(h);
    }
    const Ae = le != null && ee == null;
    (he && ($e = M && M.onVnodeUnmounted) || fe || Ae) && Ve(() => {
      $e && hn($e, g, h), fe && Yn(h, null, g, "unmounted"), Ae && (h.el = null);
    }, y);
  }, oe = (h) => {
    const { type: g, el: y, anchor: R, transition: I } = h;
    if (g === te) {
      Ue(y, R);
      return;
    }
    if (g === vs) {
      P(h);
      return;
    }
    const C = () => {
      a(y), I && !I.persisted && I.afterLeave && I.afterLeave();
    };
    if (h.shapeFlag & 1 && I && !I.persisted) {
      const { leave: M, delayLeave: _ } = I, v = () => M(y, C);
      _ ? _(h.el, C, v) : v();
    } else C();
  }, Ue = (h, g) => {
    let y;
    for (; h !== g; ) y = m(h), a(h), h = y;
    a(g);
  }, Sn = (h, g, y) => {
    const { bum: R, scope: I, job: C, subTree: M, um: _, m: v, a: E } = h;
    Ai(v), Ai(E), R && gs(R), I.stop(), C && (C.flags |= 8, J(M, h, g, y)), _ && Ve(_, g), Ve(() => {
      h.isUnmounted = true;
    }, g);
  }, Fe = (h, g, y, R = false, I = false, C = 0) => {
    for (let M = C; M < h.length; M++) J(h[M], g, y, R, I);
  }, z = (h) => {
    if (h.shapeFlag & 6) return z(h.component.subTree);
    if (h.shapeFlag & 128) return h.suspense.next();
    const g = m(h.anchor || h.el), y = g && g[ju];
    return y ? m(y) : g;
  };
  let H = false;
  const U = (h, g, y) => {
    let R;
    h == null ? g._vnode && (J(g._vnode, null, null, true), R = g._vnode.component) : T(g._vnode || null, h, g, null, null, null, y), g._vnode = h, H || (H = true, _i(R), _r(), H = false);
  }, K = { p: T, um: J, m: F, r: oe, mt: en, mc: ke, pc: ie, pbc: Se, n: z, o: e };
  return { render: U, hydrate: void 0, createApp: Ju(U) };
}
function aa({ type: e, props: n }, t) {
  return t === "svg" && e === "foreignObject" || t === "mathml" && e === "annotation-xml" && n && n.encoding && n.encoding.includes("html") ? void 0 : t;
}
function Qn({ effect: e, job: n }, t) {
  t ? (e.flags |= 32, n.flags |= 4) : (e.flags &= -33, n.flags &= -5);
}
function dc(e, n) {
  return (!e || e && !e.pendingBranch) && n && !n.persisted;
}
function Mr(e, n, t = false) {
  const s = e.children, a = n.children;
  if (Z(s) && Z(a)) for (let i = 0; i < s.length; i++) {
    const o = s[i];
    let r = a[i];
    r.shapeFlag & 1 && !r.dynamicChildren && ((r.patchFlag <= 0 || r.patchFlag === 32) && (r = a[i] = zn(a[i]), r.el = o.el), !t && r.patchFlag !== -2 && Mr(o, r)), r.type === Hs && (r.patchFlag === -1 && (r = a[i] = zn(r)), r.el = o.el), r.type === Gn && !r.el && (r.el = o.el);
  }
}
function pc(e) {
  const n = e.slice(), t = [0];
  let s, a, i, o, r;
  const l = e.length;
  for (s = 0; s < l; s++) {
    const c = e[s];
    if (c !== 0) {
      if (a = t[t.length - 1], e[a] < c) {
        n[s] = a, t.push(s);
        continue;
      }
      for (i = 0, o = t.length - 1; i < o; ) r = i + o >> 1, e[t[r]] < c ? i = r + 1 : o = r;
      c < e[t[i]] && (i > 0 && (n[s] = t[i - 1]), t[i] = s);
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
function hc(e, n) {
  n && n.pendingBranch ? Z(e) ? n.effects.push(...e) : n.effects.push(e) : wu(e);
}
const te = Symbol.for("v-fgt"), Hs = Symbol.for("v-txt"), Gn = Symbol.for("v-cmt"), vs = Symbol.for("v-stc"), Bt = [];
let Ke = null;
function w(e = false) {
  Bt.push(Ke = e ? null : []);
}
function fc() {
  Bt.pop(), Ke = Bt[Bt.length - 1] || null;
}
let Xt = 1;
function Ss(e, n = false) {
  Xt += e, e < 0 && Ke && n && (Ke.hasOnce = true);
}
function Fr(e) {
  return e.dynamicChildren = Xt > 0 ? Ke || ut : null, fc(), Xt > 0 && Ke && Ke.push(e), e;
}
function x(e, n, t, s, a, i) {
  return Fr(u(e, n, t, s, a, i, true));
}
function ti(e, n, t, s, a) {
  return Fr(ve(e, n, t, s, a, true));
}
function Es(e) {
  return e ? e.__v_isVNode === true : false;
}
function At(e, n) {
  return e.type === n.type && e.key === n.key;
}
const Vr = ({ key: e }) => e ?? null, ys = ({ ref: e, ref_key: n, ref_for: t }) => (typeof e == "number" && (e = "" + e), e != null ? xe(e) || Pe(e) || ne(e) ? { i: Be, r: e, k: n, f: !!t } : e : null);
function u(e, n = null, t = null, s = 0, a = null, i = e === te ? 0 : 1, o = false, r = false) {
  const l = { __v_isVNode: true, __v_skip: true, type: e, props: n, key: n && Vr(n), ref: n && ys(n), scopeId: yr, slotScopeIds: null, children: t, component: null, suspense: null, ssContent: null, ssFallback: null, dirs: null, transition: null, el: null, anchor: null, target: null, targetStart: null, targetAnchor: null, staticCount: 0, shapeFlag: i, patchFlag: s, dynamicProps: a, dynamicChildren: null, appContext: null, ctx: Be };
  return r ? (As(l, t), i & 128 && e.normalize(l)) : t && (l.shapeFlag |= xe(t) ? 8 : 16), Xt > 0 && !o && Ke && (l.patchFlag > 0 || i & 6) && l.patchFlag !== 32 && Ke.push(l), l;
}
const ve = mc;
function mc(e, n = null, t = null, s = 0, a = null, i = false) {
  if ((!e || e === Du) && (e = Gn), Es(e)) {
    const r = mt(e, n, true);
    return t && As(r, t), Xt > 0 && !i && Ke && (r.shapeFlag & 6 ? Ke[Ke.indexOf(e)] = r : Ke.push(r)), r.patchFlag = -2, r;
  }
  if (Sc(e) && (e = e.__vccOpts), n) {
    n = gc(n);
    let { class: r, style: l } = n;
    r && !xe(r) && (n.class = Re(r)), pe(l) && (Qa(l) && !Z(l) && (l = ze({}, l)), n.style = Kn(l));
  }
  const o = xe(e) ? 1 : Ur(e) ? 128 : Su(e) ? 64 : pe(e) ? 4 : ne(e) ? 2 : 0;
  return u(e, n, t, s, a, o, i, true);
}
function gc(e) {
  return e ? Qa(e) || Ir(e) ? ze({}, e) : e : null;
}
function mt(e, n, t = false, s = false) {
  const { props: a, ref: i, patchFlag: o, children: r, transition: l } = e, c = n ? _c(a || {}, n) : a, d = { __v_isVNode: true, __v_skip: true, type: e.type, props: c, key: c && Vr(c), ref: n && n.ref ? t && i ? Z(i) ? i.concat(ys(n)) : [i, ys(n)] : ys(n) : i, scopeId: e.scopeId, slotScopeIds: e.slotScopeIds, children: r, target: e.target, targetStart: e.targetStart, targetAnchor: e.targetAnchor, staticCount: e.staticCount, shapeFlag: e.shapeFlag, patchFlag: n && e.type !== te ? o === -1 ? 16 : o | 16 : o, dynamicProps: e.dynamicProps, dynamicChildren: e.dynamicChildren, appContext: e.appContext, dirs: e.dirs, transition: l, component: e.component, suspense: e.suspense, ssContent: e.ssContent && mt(e.ssContent), ssFallback: e.ssFallback && mt(e.ssFallback), placeholder: e.placeholder, el: e.el, anchor: e.anchor, ctx: e.ctx, ce: e.ce };
  return l && s && Za(d, l.clone(d)), d;
}
function ae(e = " ", n = 0) {
  return ve(Hs, null, e, n);
}
function Hr(e, n) {
  const t = ve(vs, null, e);
  return t.staticCount = n, t;
}
function V(e = "", n = false) {
  return n ? (w(), ti(Gn, null, e)) : ve(Gn, null, e);
}
function gn(e) {
  return e == null || typeof e == "boolean" ? ve(Gn) : Z(e) ? ve(te, null, e.slice()) : Es(e) ? zn(e) : ve(Hs, null, String(e));
}
function zn(e) {
  return e.el === null && e.patchFlag !== -1 || e.memo ? e : mt(e);
}
function As(e, n) {
  let t = 0;
  const { shapeFlag: s } = e;
  if (n == null) n = null;
  else if (Z(n)) t = 16;
  else if (typeof n == "object") if (s & 65) {
    const a = n.default;
    a && (a._c && (a._d = false), As(e, a()), a._c && (a._d = true));
    return;
  } else {
    t = 32;
    const a = n._;
    !a && !Ir(n) ? n._ctx = Be : a === 3 && Be && (Be.slots._ === 1 ? n._ = 1 : (n._ = 2, e.patchFlag |= 1024));
  }
  else if (ne(n)) {
    if (s & 65) {
      As(e, { default: n });
      return;
    }
    n = { default: n, _ctx: Be }, t = 32;
  } else n = String(n), s & 64 ? (t = 16, n = [ae(n)]) : t = 8;
  e.children = n, e.shapeFlag |= t;
}
function _c(...e) {
  const n = {};
  for (let t = 0; t < e.length; t++) {
    const s = e[t];
    for (const a in s) if (a === "class") n.class !== s.class && (n.class = Re([n.class, s.class]));
    else if (a === "style") n.style = Kn([n.style, s.style]);
    else if (Os(a)) {
      const i = n[a], o = s[a];
      o && i !== o && !(Z(i) && i.includes(o)) ? n[a] = i ? [].concat(i, o) : o : o == null && i == null && !Ns(a) && (n[a] = o);
    } else a !== "" && (n[a] = s[a]);
  }
  return n;
}
function hn(e, n, t, s = null) {
  on(e, n, 7, [t, s]);
}
const vc = Er();
let yc = 0;
function kc(e, n, t) {
  const s = e.type, a = (n ? n.appContext : e.appContext) || vc, i = { uid: yc++, vnode: e, type: s, parent: n, appContext: a, root: null, next: null, subTree: null, effect: null, update: null, job: null, scope: new Bl(true), render: null, proxy: null, exposed: null, exposeProxy: null, withProxy: null, provides: n ? n.provides : Object.create(a.provides), ids: n ? n.ids : ["", 0, 0], accessCache: null, renderCache: [], components: null, directives: null, propsOptions: Pr(s, a), emitsOptions: Ar(s, a), emit: null, emitted: null, propsDefaults: me, inheritAttrs: s.inheritAttrs, ctx: me, data: me, props: me, attrs: me, slots: me, refs: me, setupState: me, setupContext: null, suspense: t, suspenseId: t ? t.pendingId : 0, asyncDep: null, asyncResolved: false, isMounted: false, isUnmounted: false, isDeactivated: false, bc: null, c: null, bm: null, m: null, bu: null, u: null, um: null, bum: null, da: null, a: null, rtg: null, rtc: null, ec: null, sp: null };
  return i.ctx = { _: i }, i.root = n ? n.root : i, i.emit = Qu.bind(null, i), e.ce && e.ce(i), i;
}
let Te = null;
const wc = () => Te || Be;
let Cs, xa;
{
  const e = Ds(), n = (t, s) => {
    let a;
    return (a = e[t]) || (a = e[t] = []), a.push(s), (i) => {
      a.length > 1 ? a.forEach((o) => o(i)) : a[0](i);
    };
  };
  Cs = n("__VUE_INSTANCE_SETTERS__", (t) => Te = t), xa = n("__VUE_SSR_SETTERS__", (t) => Zt = t);
}
const os = (e) => {
  const n = Te;
  return Cs(e), e.scope.on(), () => {
    e.scope.off(), Cs(n);
  };
}, Ci = () => {
  Te && Te.scope.off(), Cs(null);
};
function Wr(e) {
  return e.vnode.shapeFlag & 4;
}
let Zt = false;
function bc(e, n = false, t = false) {
  n && xa(n);
  const { props: s, children: a } = e.vnode, i = Wr(e);
  sc(e, s, i, n), rc(e, a, t || n);
  const o = i ? xc(e, n) : void 0;
  return n && xa(false), o;
}
function xc(e, n) {
  const t = e.type;
  e.accessCache = /* @__PURE__ */ Object.create(null), e.proxy = new Proxy(e.ctx, Fu);
  const { setup: s } = t;
  if (s) {
    xn();
    const a = e.setupContext = s.length > 1 ? jc(e) : null, i = os(e), o = ss(s, e, 0, [e.props, a]), r = Wo(o);
    if ($n(), i(), (r || e.sp) && !Ht(e) && br(e), r) {
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
  ne(n) ? e.type.__ssrInlineRender ? e.ssrRender = n : e.render = n : pe(n) && (e.setupState = fr(n)), Br(e);
}
function Br(e, n, t) {
  const s = e.type;
  e.render || (e.render = s.render || yn);
  {
    const a = os(e);
    xn();
    try {
      Vu(e);
    } finally {
      $n(), a();
    }
  }
}
const $c = { get(e, n) {
  return Ie(e, "get", ""), e[n];
} };
function jc(e) {
  const n = (t) => {
    e.exposed = t || {};
  };
  return { attrs: new Proxy(e.attrs, $c), slots: e.slots, emit: e.emit, expose: n };
}
function Ws(e) {
  return e.exposed ? e.exposeProxy || (e.exposeProxy = new Proxy(fr(du(e.exposed)), { get(n, t) {
    if (t in n) return n[t];
    if (t in Wt) return Wt[t](e);
  }, has(n, t) {
    return t in n || t in Wt;
  } })) : e.proxy;
}
function Sc(e) {
  return ne(e) && "__vccOpts" in e;
}
const X = (e, n) => gu(e, n, Zt);
function Kr(e, n, t) {
  try {
    Ss(-1);
    const s = arguments.length;
    return s === 2 ? pe(n) && !Z(n) ? Es(n) ? ve(e, null, [n]) : ve(e, n) : ve(e, null, n) : (s > 3 ? t = Array.prototype.slice.call(arguments, 2) : s === 3 && Es(t) && (t = [t]), ve(e, n, t));
  } finally {
    Ss(1);
  }
}
const Ec = "3.5.39";
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
const Gr = $a ? (e) => $a.createHTML(e) : (e) => e, Ac = "http://www.w3.org/2000/svg", Cc = "http://www.w3.org/1998/Math/MathML", Rn = typeof document < "u" ? document : null, Ii = Rn && Rn.createElement("template"), Rc = { insert: (e, n, t) => {
  n.insertBefore(e, t || null);
}, remove: (e) => {
  const n = e.parentNode;
  n && n.removeChild(e);
}, createElement: (e, n, t, s) => {
  const a = n === "svg" ? Rn.createElementNS(Ac, e) : n === "mathml" ? Rn.createElementNS(Cc, e) : t ? Rn.createElement(e, { is: t }) : Rn.createElement(e);
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
} }, zc = Symbol("_vtc");
function Ic(e, n, t) {
  const s = e[zc];
  s && (n = (n ? [n, ...s] : [...s]).join(" ")), n == null ? e.removeAttribute("class") : t ? e.setAttribute("class", n) : e.className = n;
}
const Ti = Symbol("_vod"), Tc = Symbol("_vsh"), Pc = Symbol(""), Oc = /(?:^|;)\s*display\s*:/;
function Nc(e, n, t) {
  const s = e.style, a = xe(t);
  let i = false;
  if (t && !a) {
    if (n) if (xe(n)) for (const o of n.split(";")) {
      const r = o.slice(0, o.indexOf(":")).trim();
      t[r] == null && Ot(s, r, "");
    }
    else for (const o in n) t[o] == null && Ot(s, o, "");
    for (const o in t) {
      o === "display" && (i = true);
      const r = t[o];
      r != null ? Mc(e, o, !xe(n) && n ? n[o] : void 0, r) || Ot(s, o, r) : Ot(s, o, "");
    }
  } else if (a) {
    if (n !== t) {
      const o = s[Pc];
      o && (t += ";" + o), s.cssText = t, i = Oc.test(t);
    }
  } else n && e.removeAttribute("style");
  Ti in e && (e[Ti] = i ? s.display : "", e[Tc] && (s.display = "none"));
}
const Pi = /\s*!important$/;
function Ot(e, n, t) {
  if (Z(t)) t.forEach((s) => Ot(e, n, s));
  else if (t == null && (t = ""), n.startsWith("--")) e.setProperty(n, t);
  else {
    const s = qc(e, n);
    Pi.test(t) ? e.setProperty(Jn(s), t.replace(Pi, ""), "important") : e[s] = t;
  }
}
const Oi = ["Webkit", "Moz", "ms"], ia = {};
function qc(e, n) {
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
function Mc(e, n, t, s) {
  return e.tagName === "TEXTAREA" && (n === "width" || n === "height") && xe(s) && t === s;
}
const Ni = "http://www.w3.org/1999/xlink";
function qi(e, n, t, s, a, i = Hl(n)) {
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
function Dc(e, n, t, s) {
  e.removeEventListener(n, t, s);
}
const Di = Symbol("_vei");
function Lc(e, n, t, s, a = null) {
  const i = e[Di] || (e[Di] = {}), o = i[n];
  if (s && o) o.value = s;
  else {
    const [r, l] = Vc(n);
    if (s) {
      const c = i[n] = Bc(s, a);
      Hn(e, r, c, l);
    } else o && (Dc(e, r, o, l), i[n] = void 0);
  }
}
const Uc = /(Once|Passive|Capture)$/, Fc = /^on:?(?:Once|Passive|Capture)$/;
function Vc(e) {
  let n, t;
  for (; (t = e.match(Uc)) && !Fc.test(e); ) n || (n = {}), e = e.slice(0, e.length - t[1].length), n[t[1].toLowerCase()] = true;
  return [e[2] === ":" ? e.slice(3) : Jn(e.slice(2)), n];
}
let oa = 0;
const Hc = Promise.resolve(), Wc = () => oa || (Hc.then(() => oa = 0), oa = Date.now());
function Bc(e, n) {
  const t = (s) => {
    if (!s._vts) s._vts = Date.now();
    else if (s._vts <= t.attached) return;
    const a = t.value;
    if (Z(a)) {
      const i = s.stopImmediatePropagation;
      s.stopImmediatePropagation = () => {
        i.call(s), s._stopped = true;
      };
      const o = a.slice(), r = [s];
      for (let l = 0; l < o.length && !s._stopped; l++) {
        const c = o[l];
        c && on(c, n, 5, r);
      }
    } else on(a, n, 5, [s]);
  };
  return t.value = e, t.attached = Wc(), t;
}
const Li = (e) => e.charCodeAt(0) === 111 && e.charCodeAt(1) === 110 && e.charCodeAt(2) > 96 && e.charCodeAt(2) < 123, Kc = (e, n, t, s, a, i) => {
  const o = a === "svg";
  n === "class" ? Ic(e, s, o) : n === "style" ? Nc(e, t, s) : Os(n) ? Ns(n) || Lc(e, n, t, s, i) : (n[0] === "." ? (n = n.slice(1), true) : n[0] === "^" ? (n = n.slice(1), false) : Gc(e, n, s, o)) ? (Mi(e, n, s), !e.tagName.includes("-") && (n === "value" || n === "checked" || n === "selected") && qi(e, n, s, o, i, n !== "value")) : e._isVueCE && (Jc(e, n) || e._def.__asyncLoader && (/[A-Z]/.test(n) || !xe(s))) ? Mi(e, Ge(n), s, i, n) : (n === "true-value" ? e._trueValue = s : n === "false-value" && (e._falseValue = s), qi(e, n, s, o));
};
function Gc(e, n, t, s) {
  if (s) return !!(n === "innerHTML" || n === "textContent" || n in e && Li(n) && ne(t));
  if (n === "spellcheck" || n === "draggable" || n === "translate" || n === "autocorrect" || n === "sandbox" && e.tagName === "IFRAME" || n === "form" || n === "list" && e.tagName === "INPUT" || n === "type" && e.tagName === "TEXTAREA") return false;
  if (n === "width" || n === "height") {
    const a = e.tagName;
    if (a === "IMG" || a === "VIDEO" || a === "CANVAS" || a === "SOURCE") return false;
  }
  return Li(n) && xe(t) ? false : n in e;
}
function Jc(e, n) {
  const t = e._def.props;
  if (!t) return false;
  const s = Ge(n);
  return Array.isArray(t) ? t.some((a) => Ge(a) === s) : Object.keys(t).some((a) => Ge(a) === s);
}
const gt = (e) => {
  const n = e.props["onUpdate:modelValue"] || false;
  return Z(n) ? (t) => gs(n, t) : n;
};
function Yc(e) {
  e.target.composing = true;
}
function Ui(e) {
  const n = e.target;
  n.composing && (n.composing = false, n.dispatchEvent(new Event("input")));
}
const Pn = Symbol("_assign");
function Fi(e, n, t) {
  return n && (e = e.trim()), t && (e = Ms(e)), e;
}
const Wn = { created(e, { modifiers: { lazy: n, trim: t, number: s } }, a) {
  e[Pn] = gt(a);
  const i = s || a.props && a.props.type === "number";
  Hn(e, n ? "change" : "input", (o) => {
    o.target.composing || e[Pn](Fi(e.value, t, i));
  }), (t || i) && Hn(e, "change", () => {
    e.value = Fi(e.value, t, i);
  }), n || (Hn(e, "compositionstart", Yc), Hn(e, "compositionend", Ui), Hn(e, "change", Ui));
}, mounted(e, { value: n }) {
  e.value = n ?? "";
}, beforeUpdate(e, { value: n, oldValue: t, modifiers: { lazy: s, trim: a, number: i } }, o) {
  if (e[Pn] = gt(o), e.composing) return;
  const r = (i || e.type === "number") && !/^0\d/.test(e.value) ? Ms(e.value) : e.value, l = n ?? "";
  if (r === l) return;
  const c = e.getRootNode();
  (c instanceof Document || c instanceof ShadowRoot) && c.activeElement === e && e.type !== "range" && (s && n === t || a && e.value.trim() === l) || (e.value = l);
} }, ks = { deep: true, created(e, n, t) {
  e[Pn] = gt(t), Hn(e, "change", () => {
    const s = e._modelValue, a = es(e), i = e.checked, o = e[Pn];
    if (Z(s)) {
      const r = Ha(s, a), l = r !== -1;
      if (i && !l) o(s.concat(a));
      else if (!i && l) {
        const c = [...s];
        c.splice(r, 1), o(c);
      }
    } else if (wt(s)) {
      const r = new Set(s);
      i ? r.add(a) : r.delete(a), o(r);
    } else o(Jr(e, i));
  });
}, mounted: Vi, beforeUpdate(e, n, t) {
  e[Pn] = gt(t), Vi(e, n, t);
} };
function Vi(e, { value: n, oldValue: t }, s) {
  e._modelValue = n;
  let a;
  if (Z(n)) a = Ha(n, s.props.value) > -1;
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
    const i = Array.prototype.filter.call(e.options, (o) => o.selected).map((o) => t ? Ms(es(o)) : es(o));
    e[Pn](e.multiple ? a ? new Set(i) : i : i[0]), e._assigning = true, as(() => {
      e._assigning = false;
    });
  }), e[Pn] = gt(s);
}, mounted(e, { value: n }) {
  Hi(e, n);
}, beforeUpdate(e, n, t) {
  e[Pn] = gt(t);
}, updated(e, { value: n }) {
  e._assigning || Hi(e, n);
} };
function Hi(e, n) {
  const t = e.multiple, s = Z(n);
  if (!(t && !s && !wt(n))) {
    for (let a = 0, i = e.options.length; a < i; a++) {
      const o = e.options[a], r = es(o);
      if (t) if (s) {
        const l = typeof r;
        l === "string" || l === "number" ? o.selected = n.some((c) => String(c) === String(r)) : o.selected = Ha(n, r) > -1;
      } else o.selected = n.has(r);
      else if (bt(es(o), n)) {
        e.selectedIndex !== a && (e.selectedIndex = a);
        return;
      }
    }
    !t && e.selectedIndex !== -1 && (e.selectedIndex = -1);
  }
}
function es(e) {
  return "_value" in e ? e._value : e.value;
}
function Jr(e, n) {
  const t = n ? "_trueValue" : "_falseValue";
  return t in e ? e[t] : n;
}
const Qc = ["ctrl", "shift", "alt", "meta"], Xc = { stop: (e) => e.stopPropagation(), prevent: (e) => e.preventDefault(), self: (e) => e.target !== e.currentTarget, ctrl: (e) => !e.ctrlKey, shift: (e) => !e.shiftKey, alt: (e) => !e.altKey, meta: (e) => !e.metaKey, left: (e) => "button" in e && e.button !== 0, middle: (e) => "button" in e && e.button !== 1, right: (e) => "button" in e && e.button !== 2, exact: (e, n) => Qc.some((t) => e[`${t}Key`] && !n.includes(t)) }, ja = (e, n) => {
  if (!e) return e;
  const t = e._withMods || (e._withMods = {}), s = n.join(".");
  return t[s] || (t[s] = ((a, ...i) => {
    for (let o = 0; o < n.length; o++) {
      const r = Xc[n[o]];
      if (r && r(a, n)) return;
    }
    return e(a, ...i);
  }));
}, Zc = { esc: "escape", space: " ", up: "arrow-up", left: "arrow-left", right: "arrow-right", down: "arrow-down", delete: "backspace" }, ed = (e, n) => {
  const t = e._withKeys || (e._withKeys = {}), s = n.join(".");
  return t[s] || (t[s] = ((a) => {
    if (!("key" in a)) return;
    const i = Jn(a.key);
    if (n.some((o) => o === i || Zc[o] === i)) return e(a);
  }));
}, nd = ze({ patchProp: Kc }, Rc);
let Wi;
function td() {
  return Wi || (Wi = uc(nd));
}
const sd = ((...e) => {
  const n = td().createApp(...e), { mount: t } = n;
  return n.mount = (s) => {
    const a = id(s);
    if (!a) return;
    const i = n._component;
    !ne(i) && !i.render && !i.template && (i.template = a.innerHTML), a.nodeType === 1 && (a.textContent = "");
    const o = t(a, false, ad(a));
    return a instanceof Element && (a.removeAttribute("v-cloak"), a.setAttribute("data-v-app", "")), o;
  }, n;
});
function ad(e) {
  if (e instanceof SVGElement) return "svg";
  if (typeof MathMLElement == "function" && e instanceof MathMLElement) return "mathml";
}
function id(e) {
  return xe(e) ? document.querySelector(e) : e;
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
function od(e) {
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
const Kt = () => {
}, rn = Array.isArray;
function Bi(e, n) {
  const t = {};
  for (const s in e) t[s] = s in n ? n[s] : e[s];
  return t;
}
const Qr = /#/g, rd = /&/g, ld = /\//g, ud = /=/g, cd = /\?/g, Xr = /\+/g, dd = /%5B/g, pd = /%5D/g, Zr = /%5E/g, hd = /%60/g, el = /%7B/g, fd = /%7C/g, nl = /%7D/g, md = /%20/g;
function si(e) {
  return e == null ? "" : encodeURI("" + e).replace(fd, "|").replace(dd, "[").replace(pd, "]");
}
function gd(e) {
  return si(e).replace(el, "{").replace(nl, "}").replace(Zr, "^");
}
function Sa(e) {
  return si(e).replace(Xr, "%2B").replace(md, "+").replace(Qr, "%23").replace(rd, "%26").replace(hd, "`").replace(el, "{").replace(nl, "}").replace(Zr, "^");
}
function _d(e) {
  return Sa(e).replace(ud, "%3D");
}
function vd(e) {
  return si(e).replace(Qr, "%23").replace(cd, "%3F");
}
function yd(e) {
  return vd(e).replace(ld, "%2F");
}
function ns(e) {
  if (e == null) return null;
  try {
    return decodeURIComponent("" + e);
  } catch {
  }
  return "" + e;
}
const kd = /\/$/, wd = (e) => e.replace(kd, "");
function la(e, n, t = "/") {
  let s, a = {}, i = "", o = "";
  const r = n.indexOf("#");
  let l = n.indexOf("?");
  return l = r >= 0 && l > r ? -1 : l, l >= 0 && (s = n.slice(0, l), i = n.slice(l, r > 0 ? r : n.length), a = e(i.slice(1))), r >= 0 && (s = s || n.slice(0, r), o = n.slice(r, n.length)), s = jd(s ?? n, t), { fullPath: s + i + o, path: s, query: a, hash: ns(o) };
}
function bd(e, n) {
  const t = n.query ? e(n.query) : "";
  return n.path + (t && "?") + t + (n.hash || "");
}
function Ki(e, n) {
  return !n || !e.toLowerCase().startsWith(n.toLowerCase()) ? e : e.slice(n.length) || "/";
}
function xd(e, n, t) {
  const s = n.matched.length - 1, a = t.matched.length - 1;
  return s > -1 && s === a && _t(n.matched[s], t.matched[a]) && tl(n.params, t.params) && e(n.query) === e(t.query) && n.hash === t.hash;
}
function _t(e, n) {
  return (e.aliasOf || e) === (n.aliasOf || n);
}
function tl(e, n) {
  if (Object.keys(e).length !== Object.keys(n).length) return false;
  for (var t in e) if (!$d(e[t], n[t])) return false;
  return true;
}
function $d(e, n) {
  return rn(e) ? Gi(e, n) : rn(n) ? Gi(n, e) : (e == null ? void 0 : e.valueOf()) === (n == null ? void 0 : n.valueOf());
}
function Gi(e, n) {
  return rn(n) ? e.length === n.length && e.every((t, s) => t === n[s]) : e.length === 1 && e[0] === n;
}
function jd(e, n) {
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
function Sd(e) {
  if (!e) if (rt) {
    const n = document.querySelector("base");
    e = n && n.getAttribute("href") || "/", e = e.replace(/^\w+:\/\/[^\/]+/, "");
  } else e = "/";
  return e[0] !== "/" && e[0] !== "#" && (e = "/" + e), wd(e);
}
const Ed = /^[^#]+#/;
function Ad(e, n) {
  return e.replace(Ed, "#") + n;
}
function Cd(e, n) {
  const t = document.documentElement.getBoundingClientRect(), s = e.getBoundingClientRect();
  return { behavior: n.behavior, left: s.left - t.left - (n.left || 0), top: s.top - t.top - (n.top || 0) };
}
const Bs = () => ({ left: window.scrollX, top: window.scrollY });
function Rd(e) {
  let n;
  if ("el" in e) {
    const t = e.el, s = typeof t == "string" && t.startsWith("#"), a = typeof t == "string" ? s ? document.getElementById(t.slice(1)) : document.querySelector(t) : t;
    if (!a) return;
    n = Cd(a, e);
  } else n = e;
  "scrollBehavior" in document.documentElement.style ? window.scrollTo(n) : window.scrollTo(n.left != null ? n.left : window.scrollX, n.top != null ? n.top : window.scrollY);
}
function Ji(e, n) {
  return (history.state ? history.state.position - n : -1) + e;
}
const Aa = /* @__PURE__ */ new Map();
function zd(e, n) {
  Aa.set(e, n);
}
function Id(e) {
  const n = Aa.get(e);
  return Aa.delete(e), n;
}
function Td(e) {
  return typeof e == "string" || e && typeof e == "object";
}
function sl(e) {
  return typeof e == "string" || typeof e == "symbol";
}
let be = (function(e) {
  return e[e.MATCHER_NOT_FOUND = 1] = "MATCHER_NOT_FOUND", e[e.NAVIGATION_GUARD_REDIRECT = 2] = "NAVIGATION_GUARD_REDIRECT", e[e.NAVIGATION_ABORTED = 4] = "NAVIGATION_ABORTED", e[e.NAVIGATION_CANCELLED = 8] = "NAVIGATION_CANCELLED", e[e.NAVIGATION_DUPLICATED = 16] = "NAVIGATION_DUPLICATED", e;
})({});
const al = Symbol("");
be.MATCHER_NOT_FOUND + "", be.NAVIGATION_GUARD_REDIRECT + "", be.NAVIGATION_ABORTED + "", be.NAVIGATION_CANCELLED + "", be.NAVIGATION_DUPLICATED + "";
function vt(e, n) {
  return ue(new Error(), { type: e, [al]: true }, n);
}
function An(e, n) {
  return e instanceof Error && al in e && (n == null || !!(e.type & n));
}
const Pd = ["params", "query", "hash"];
function Od(e) {
  if (typeof e == "string") return e;
  if (e.path != null) return e.path;
  const n = {};
  for (const t of Pd) t in e && (n[t] = e[t]);
  return JSON.stringify(n, null, 2);
}
function Nd(e) {
  const n = {};
  if (e === "" || e === "?") return n;
  const t = (e[0] === "?" ? e.slice(1) : e).split("&");
  for (let s = 0; s < t.length; ++s) {
    const a = t[s].replace(Xr, " "), i = a.indexOf("="), o = ns(i < 0 ? a : a.slice(0, i)), r = i < 0 ? null : ns(a.slice(i + 1));
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
    if (t = _d(t), s == null) {
      s !== void 0 && (n += (n.length ? "&" : "") + t);
      continue;
    }
    (rn(s) ? s.map((a) => a && Sa(a)) : [s && Sa(s)]).forEach((a) => {
      a !== void 0 && (n += (n.length ? "&" : "") + t, a != null && (n += "=" + a));
    });
  }
  return n;
}
function qd(e) {
  const n = {};
  for (const t in e) {
    const s = e[t];
    s !== void 0 && (n[t] = rn(s) ? s.map((a) => a == null ? null : "" + a) : s == null ? s : "" + s);
  }
  return n;
}
const Md = Symbol(""), Qi = Symbol(""), Ks = Symbol(""), ai = Symbol(""), Ca = Symbol("");
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
    const c = (m) => {
      m === false ? l(vt(be.NAVIGATION_ABORTED, { from: t, to: n })) : m instanceof Error ? l(m) : Td(m) ? l(vt(be.NAVIGATION_GUARD_REDIRECT, { from: n, to: m })) : (o && s.enterCallbacks[a] === o && typeof m == "function" && o.push(m), r());
    }, d = i(() => e.call(s && s.instances[a], n, t, c));
    let p = Promise.resolve(d);
    e.length < 3 && (p = p.then(c)), p.catch((m) => l(m));
  });
}
function ca(e, n, t, s, a = (i) => i()) {
  const i = [];
  for (const o of e) for (const r in o.components) {
    let l = o.components[r];
    if (!(n !== "beforeRouteEnter" && !o.instances[r])) if (Yr(l)) {
      const c = (l.__vccOpts || l)[n];
      c && i.push(Vn(c, t, s, o, r, a));
    } else {
      let c = l();
      i.push(() => c.then((d) => {
        if (!d) throw new Error(`Couldn't resolve component "${r}" at "${o.path}"`);
        const p = od(d) ? d.default : d;
        o.mods[r] = d, o.components[r] = p;
        const m = (p.__vccOpts || p)[n];
        return m && Vn(m, t, s, o, r, a)();
      }));
    }
  }
  return i;
}
function Dd(e, n) {
  const t = [], s = [], a = [], i = Math.max(n.matched.length, e.matched.length);
  for (let o = 0; o < i; o++) {
    const r = n.matched[o];
    r && (e.matched.find((c) => _t(c, r)) ? s.push(r) : t.push(r));
    const l = e.matched[o];
    l && (n.matched.find((c) => _t(c, l)) || a.push(l));
  }
  return [t, s, a];
}
/*!
* vue-router v4.6.4
* (c) 2025 Eduardo San Martin Morote
* @license MIT
*/
let Ld = () => location.protocol + "//" + location.host;
function il(e, n) {
  const { pathname: t, search: s, hash: a } = n, i = e.indexOf("#");
  if (i > -1) {
    let o = a.includes(e.slice(i)) ? e.slice(i).length : 1, r = a.slice(o);
    return r[0] !== "/" && (r = "/" + r), Ki(r, "");
  }
  return Ki(t, e) + s + a;
}
function Ud(e, n, t, s) {
  let a = [], i = [], o = null;
  const r = ({ state: m }) => {
    const f = il(e, location), k = t.value, T = n.value;
    let W = 0;
    if (m) {
      if (t.value = f, n.value = m, o && o === k) {
        o = null;
        return;
      }
      W = T ? m.position - T.position : 0;
    } else s(f);
    a.forEach((j) => {
      j(t.value, k, { delta: W, type: Ea.pop, direction: W ? W > 0 ? ua.forward : ua.back : ua.unknown });
    });
  };
  function l() {
    o = t.value;
  }
  function c(m) {
    a.push(m);
    const f = () => {
      const k = a.indexOf(m);
      k > -1 && a.splice(k, 1);
    };
    return i.push(f), f;
  }
  function d() {
    if (document.visibilityState === "hidden") {
      const { history: m } = window;
      if (!m.state) return;
      m.replaceState(ue({}, m.state, { scroll: Bs() }), "");
    }
  }
  function p() {
    for (const m of i) m();
    i = [], window.removeEventListener("popstate", r), window.removeEventListener("pagehide", d), document.removeEventListener("visibilitychange", d);
  }
  return window.addEventListener("popstate", r), window.addEventListener("pagehide", d), document.addEventListener("visibilitychange", d), { pauseListeners: l, listen: c, destroy: p };
}
function Xi(e, n, t, s = false, a = false) {
  return { back: e, current: n, forward: t, replaced: s, position: window.history.length, scroll: a ? Bs() : null };
}
function Fd(e) {
  const { history: n, location: t } = window, s = { value: il(e, t) }, a = { value: n.state };
  a.value || i(s.value, { back: null, current: s.value, forward: null, position: n.length - 1, replaced: true, scroll: null }, true);
  function i(l, c, d) {
    const p = e.indexOf("#"), m = p > -1 ? (t.host && document.querySelector("base") ? e : e.slice(p)) + l : Ld() + e + l;
    try {
      n[d ? "replaceState" : "pushState"](c, "", m), a.value = c;
    } catch (f) {
      console.error(f), t[d ? "replace" : "assign"](m);
    }
  }
  function o(l, c) {
    i(l, ue({}, n.state, Xi(a.value.back, l, a.value.forward, true), c, { position: a.value.position }), true), s.value = l;
  }
  function r(l, c) {
    const d = ue({}, a.value, n.state, { forward: l, scroll: Bs() });
    i(d.current, d, true), i(l, ue({}, Xi(s.value, l, null), { position: d.position + 1 }, c), false), s.value = l;
  }
  return { location: s, state: a, push: r, replace: o };
}
function ol(e) {
  e = Sd(e);
  const n = Fd(e), t = Ud(e, n.state, n.location, n.replace);
  function s(i, o = true) {
    o || t.pauseListeners(), history.go(i);
  }
  const a = ue({ location: "", base: e, go: s, createHref: Ad.bind(null, e) }, n, t);
  return Object.defineProperty(a, "location", { enumerable: true, get: () => n.location.value }), Object.defineProperty(a, "state", { enumerable: true, get: () => n.state.value }), a;
}
function Vd(e) {
  return e = location.host ? e || location.pathname + location.search : "", e.includes("#") || (e += "#"), ol(e);
}
let Zn = (function(e) {
  return e[e.Static = 0] = "Static", e[e.Param = 1] = "Param", e[e.Group = 2] = "Group", e;
})({});
var je = (function(e) {
  return e[e.Static = 0] = "Static", e[e.Param = 1] = "Param", e[e.ParamRegExp = 2] = "ParamRegExp", e[e.ParamRegExpEnd = 3] = "ParamRegExpEnd", e[e.EscapeNext = 4] = "EscapeNext", e;
})(je || {});
const Hd = { type: Zn.Static, value: "" }, Wd = /[a-zA-Z0-9_]/;
function Bd(e) {
  if (!e) return [[]];
  if (e === "/") return [[Hd]];
  if (!e.startsWith("/")) throw new Error(`Invalid path "${e}"`);
  function n(f) {
    throw new Error(`ERR (${t})/"${c}": ${f}`);
  }
  let t = je.Static, s = t;
  const a = [];
  let i;
  function o() {
    i && a.push(i), i = [];
  }
  let r = 0, l, c = "", d = "";
  function p() {
    c && (t === je.Static ? i.push({ type: Zn.Static, value: c }) : t === je.Param || t === je.ParamRegExp || t === je.ParamRegExpEnd ? (i.length > 1 && (l === "*" || l === "+") && n(`A repeatable param (${c}) must be alone in its segment. eg: '/:ids+.`), i.push({ type: Zn.Param, value: c, regexp: d, repeatable: l === "*" || l === "+", optional: l === "*" || l === "?" })) : n("Invalid state to consume buffer"), c = "");
  }
  function m() {
    c += l;
  }
  for (; r < e.length; ) {
    if (l = e[r++], l === "\\" && t !== je.ParamRegExp) {
      s = t, t = je.EscapeNext;
      continue;
    }
    switch (t) {
      case je.Static:
        l === "/" ? (c && p(), o()) : l === ":" ? (p(), t = je.Param) : m();
        break;
      case je.EscapeNext:
        m(), t = s;
        break;
      case je.Param:
        l === "(" ? t = je.ParamRegExp : Wd.test(l) ? m() : (p(), t = je.Static, l !== "*" && l !== "?" && l !== "+" && r--);
        break;
      case je.ParamRegExp:
        l === ")" ? d[d.length - 1] == "\\" ? d = d.slice(0, -1) + l : t = je.ParamRegExpEnd : d += l;
        break;
      case je.ParamRegExpEnd:
        p(), t = je.Static, l !== "*" && l !== "?" && l !== "+" && r--, d = "";
        break;
      default:
        n("Unknown state");
        break;
    }
  }
  return t === je.ParamRegExp && n(`Unfinished custom RegExp for param "${c}"`), p(), o(), a;
}
const Zi = "[^/]+?", Kd = { sensitive: false, strict: false, start: true, end: true };
var qe = (function(e) {
  return e[e._multiplier = 10] = "_multiplier", e[e.Root = 90] = "Root", e[e.Segment = 40] = "Segment", e[e.SubSegment = 30] = "SubSegment", e[e.Static = 40] = "Static", e[e.Dynamic = 20] = "Dynamic", e[e.BonusCustomRegExp = 10] = "BonusCustomRegExp", e[e.BonusWildcard = -50] = "BonusWildcard", e[e.BonusRepeatable = -20] = "BonusRepeatable", e[e.BonusOptional = -8] = "BonusOptional", e[e.BonusStrict = 0.7000000000000001] = "BonusStrict", e[e.BonusCaseSensitive = 0.25] = "BonusCaseSensitive", e;
})(qe || {});
const Gd = /[.+*?^${}()[\]/\\]/g;
function Jd(e, n) {
  const t = ue({}, Kd, n), s = [];
  let a = t.start ? "^" : "";
  const i = [];
  for (const c of e) {
    const d = c.length ? [] : [qe.Root];
    t.strict && !c.length && (a += "/");
    for (let p = 0; p < c.length; p++) {
      const m = c[p];
      let f = qe.Segment + (t.sensitive ? qe.BonusCaseSensitive : 0);
      if (m.type === Zn.Static) p || (a += "/"), a += m.value.replace(Gd, "\\$&"), f += qe.Static;
      else if (m.type === Zn.Param) {
        const { value: k, repeatable: T, optional: W, regexp: j } = m;
        i.push({ name: k, repeatable: T, optional: W });
        const $ = j || Zi;
        if ($ !== Zi) {
          f += qe.BonusCustomRegExp;
          try {
            `${$}`;
          } catch (P) {
            throw new Error(`Invalid custom RegExp for param "${k}" (${$}): ` + P.message);
          }
        }
        let A = T ? `((?:${$})(?:/(?:${$}))*)` : `(${$})`;
        p || (A = W && c.length < 2 ? `(?:/${A})` : "/" + A), W && (A += "?"), a += A, f += qe.Dynamic, W && (f += qe.BonusOptional), T && (f += qe.BonusRepeatable), $ === ".*" && (f += qe.BonusWildcard);
      }
      d.push(f);
    }
    s.push(d);
  }
  if (t.strict && t.end) {
    const c = s.length - 1;
    s[c][s[c].length - 1] += qe.BonusStrict;
  }
  t.strict || (a += "/?"), t.end ? a += "$" : t.strict && !a.endsWith("/") && (a += "(?:/|$)");
  const o = new RegExp(a, t.sensitive ? "" : "i");
  function r(c) {
    const d = c.match(o), p = {};
    if (!d) return null;
    for (let m = 1; m < d.length; m++) {
      const f = d[m] || "", k = i[m - 1];
      p[k.name] = f && k.repeatable ? f.split("/") : f;
    }
    return p;
  }
  function l(c) {
    let d = "", p = false;
    for (const m of e) {
      (!p || !d.endsWith("/")) && (d += "/"), p = false;
      for (const f of m) if (f.type === Zn.Static) d += f.value;
      else if (f.type === Zn.Param) {
        const { value: k, repeatable: T, optional: W } = f, j = k in c ? c[k] : "";
        if (rn(j) && !T) throw new Error(`Provided param "${k}" is an array but it is not repeatable (* or + modifiers)`);
        const $ = rn(j) ? j.join("/") : j;
        if (!$) if (W) m.length < 2 && (d.endsWith("/") ? d = d.slice(0, -1) : p = true);
        else throw new Error(`Missing required param "${k}"`);
        d += $;
      }
    }
    return d || "/";
  }
  return { re: o, score: s, keys: i, parse: r, stringify: l };
}
function Yd(e, n) {
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
    const i = Yd(s[t], a[t]);
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
const Qd = { strict: false, end: true, sensitive: false };
function Xd(e, n, t) {
  const s = Jd(Bd(e.path), t), a = ue(s, { record: e, parent: n, children: [], alias: [] });
  return n && !a.record.aliasOf == !n.record.aliasOf && n.children.push(a), a;
}
function Zd(e, n) {
  const t = [], s = /* @__PURE__ */ new Map();
  n = Bi(Qd, n);
  function a(p) {
    return s.get(p);
  }
  function i(p, m, f) {
    const k = !f, T = to(p);
    T.aliasOf = f && f.record;
    const W = Bi(n, p), j = [T];
    if ("alias" in p) {
      const P = typeof p.alias == "string" ? [p.alias] : p.alias;
      for (const O of P) j.push(to(ue({}, T, { components: f ? f.record.components : T.components, path: O, aliasOf: f ? f.record : T })));
    }
    let $, A;
    for (const P of j) {
      const { path: O } = P;
      if (m && O[0] !== "/") {
        const G = m.record.path, N = G[G.length - 1] === "/" ? "" : "/";
        P.path = m.record.path + (O && N + O);
      }
      if ($ = Xd(P, m, W), f ? f.alias.push($) : (A = A || $, A !== $ && A.alias.push($), k && p.name && !so($) && o(p.name)), ll($) && l($), T.children) {
        const G = T.children;
        for (let N = 0; N < G.length; N++) i(G[N], $, f && f.children[N]);
      }
      f = f || $;
    }
    return A ? () => {
      o(A);
    } : Kt;
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
    const m = tp(p, t);
    t.splice(m, 0, p), p.record.name && !so(p) && s.set(p.record.name, p);
  }
  function c(p, m) {
    let f, k = {}, T, W;
    if ("name" in p && p.name) {
      if (f = s.get(p.name), !f) throw vt(be.MATCHER_NOT_FOUND, { location: p });
      W = f.record.name, k = ue(no(m.params, f.keys.filter((A) => !A.optional).concat(f.parent ? f.parent.keys.filter((A) => A.optional) : []).map((A) => A.name)), p.params && no(p.params, f.keys.map((A) => A.name))), T = f.stringify(k);
    } else if (p.path != null) T = p.path, f = t.find((A) => A.re.test(T)), f && (k = f.parse(T), W = f.record.name);
    else {
      if (f = m.name ? s.get(m.name) : t.find((A) => A.re.test(m.path)), !f) throw vt(be.MATCHER_NOT_FOUND, { location: p, currentLocation: m });
      W = f.record.name, k = ue({}, m.params, p.params), T = f.stringify(k);
    }
    const j = [];
    let $ = f;
    for (; $; ) j.unshift($.record), $ = $.parent;
    return { name: W, path: T, params: k, matched: j, meta: np(j) };
  }
  e.forEach((p) => i(p));
  function d() {
    t.length = 0, s.clear();
  }
  return { addRoute: i, resolve: c, removeRoute: o, clearRoutes: d, getRoutes: r, getRecordMatcher: a };
}
function no(e, n) {
  const t = {};
  for (const s of n) s in e && (t[s] = e[s]);
  return t;
}
function to(e) {
  const n = { path: e.path, redirect: e.redirect, name: e.name, meta: e.meta || {}, aliasOf: e.aliasOf, beforeEnter: e.beforeEnter, props: ep(e), children: e.children || [], instances: {}, leaveGuards: /* @__PURE__ */ new Set(), updateGuards: /* @__PURE__ */ new Set(), enterCallbacks: {}, components: "components" in e ? e.components || null : e.component && { default: e.component } };
  return Object.defineProperty(n, "mods", { value: {} }), n;
}
function ep(e) {
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
function np(e) {
  return e.reduce((n, t) => ue(n, t.meta), {});
}
function tp(e, n) {
  let t = 0, s = n.length;
  for (; t !== s; ) {
    const i = t + s >> 1;
    rl(e, n[i]) < 0 ? s = i : t = i + 1;
  }
  const a = sp(e);
  return a && (s = n.lastIndexOf(a, s - 1)), s;
}
function sp(e) {
  let n = e;
  for (; n = n.parent; ) if (ll(n) && rl(e, n) === 0) return n;
}
function ll({ record: e }) {
  return !!(e.name || e.components && Object.keys(e.components).length || e.redirect);
}
function ao(e) {
  const n = sn(Ks), t = sn(ai), s = X(() => {
    const l = L(e.to);
    return n.resolve(l);
  }), a = X(() => {
    const { matched: l } = s.value, { length: c } = l, d = l[c - 1], p = t.matched;
    if (!d || !p.length) return -1;
    const m = p.findIndex(_t.bind(null, d));
    if (m > -1) return m;
    const f = io(l[c - 2]);
    return c > 1 && io(d) === f && p[p.length - 1].path !== f ? p.findIndex(_t.bind(null, l[c - 2])) : m;
  }), i = X(() => a.value > -1 && rp(t.params, s.value.params)), o = X(() => a.value > -1 && a.value === t.matched.length - 1 && tl(t.params, s.value.params));
  function r(l = {}) {
    if (op(l)) {
      const c = n[L(e.replace) ? "replace" : "push"](L(e.to)).catch(Kt);
      return e.viewTransition && typeof document < "u" && "startViewTransition" in document && document.startViewTransition(() => c), c;
    }
    return Promise.resolve();
  }
  return { route: s, href: X(() => s.value.href), isActive: i, isExactActive: o, navigate: r };
}
function ap(e) {
  return e.length === 1 ? e[0] : e;
}
const ip = De({ name: "RouterLink", compatConfig: { MODE: 3 }, props: { to: { type: [String, Object], required: true }, replace: Boolean, activeClass: String, exactActiveClass: String, custom: Boolean, ariaCurrentValue: { type: String, default: "page" }, viewTransition: Boolean }, useLink: ao, setup(e, { slots: n }) {
  const t = xt(ao(e)), { options: s } = sn(Ks), a = X(() => ({ [oo(e.activeClass, s.linkActiveClass, "router-link-active")]: t.isActive, [oo(e.exactActiveClass, s.linkExactActiveClass, "router-link-exact-active")]: t.isExactActive }));
  return () => {
    const i = n.default && ap(n.default(t));
    return e.custom ? i : Kr("a", { "aria-current": t.isExactActive ? e.ariaCurrentValue : null, href: t.href, onClick: t.navigate, class: a.value }, i);
  };
} }), Un = ip;
function op(e) {
  if (!(e.metaKey || e.altKey || e.ctrlKey || e.shiftKey) && !e.defaultPrevented && !(e.button !== void 0 && e.button !== 0)) {
    if (e.currentTarget && e.currentTarget.getAttribute) {
      const n = e.currentTarget.getAttribute("target");
      if (/\b_blank\b/i.test(n)) return;
    }
    return e.preventDefault && e.preventDefault(), true;
  }
}
function rp(e, n) {
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
const oo = (e, n, t) => e ?? n ?? t, lp = De({ name: "RouterView", inheritAttrs: false, props: { name: { type: String, default: "default" }, route: Object }, compatConfig: { MODE: 3 }, setup(e, { attrs: n, slots: t }) {
  const s = sn(Ca), a = X(() => e.route || s.value), i = sn(Qi, 0), o = X(() => {
    let c = L(i);
    const { matched: d } = a.value;
    let p;
    for (; (p = d[c]) && !p.components; ) c++;
    return c;
  }), r = X(() => a.value.matched[o.value]);
  _s(Qi, X(() => o.value + 1)), _s(Md, r), _s(Ca, a);
  const l = B();
  return kn(() => [l.value, r.value, e.name], ([c, d, p], [m, f, k]) => {
    d && (d.instances[p] = c, f && f !== d && c && c === m && (d.leaveGuards.size || (d.leaveGuards = f.leaveGuards), d.updateGuards.size || (d.updateGuards = f.updateGuards))), c && d && (!f || !_t(d, f) || !m) && (d.enterCallbacks[p] || []).forEach((T) => T(c));
  }, { flush: "post" }), () => {
    const c = a.value, d = e.name, p = r.value, m = p && p.components[d];
    if (!m) return ro(t.default, { Component: m, route: c });
    const f = p.props[d], k = f ? f === true ? c.params : typeof f == "function" ? f(c) : f : null, W = Kr(m, ue({}, k, n, { onVnodeUnmounted: (j) => {
      j.component.isUnmounted && (p.instances[d] = null);
    }, ref: l }));
    return ro(t.default, { Component: W, route: c }) || W;
  };
} });
function ro(e, n) {
  if (!e) return null;
  const t = e(n);
  return t.length === 1 ? t[0] : t;
}
const ul = lp;
function up(e) {
  const n = Zd(e.routes, e), t = e.parseQuery || Nd, s = e.stringifyQuery || Yi, a = e.history, i = Ct(), o = Ct(), r = Ct(), l = pu(Mn);
  let c = Mn;
  rt && e.scrollBehavior && "scrollRestoration" in history && (history.scrollRestoration = "manual");
  const d = ra.bind(null, (z) => "" + z), p = ra.bind(null, yd), m = ra.bind(null, ns);
  function f(z, H) {
    let U, K;
    return sl(z) ? (U = n.getRecordMatcher(z), K = H) : K = z, n.addRoute(K, U);
  }
  function k(z) {
    const H = n.getRecordMatcher(z);
    H && n.removeRoute(H);
  }
  function T() {
    return n.getRoutes().map((z) => z.record);
  }
  function W(z) {
    return !!n.getRecordMatcher(z);
  }
  function j(z, H) {
    if (H = ue({}, H || l.value), typeof z == "string") {
      const y = la(t, z, H.path), R = n.resolve({ path: y.path }, H), I = a.createHref(y.fullPath);
      return ue(y, R, { params: m(R.params), hash: ns(y.hash), redirectedFrom: void 0, href: I });
    }
    let U;
    if (z.path != null) U = ue({}, z, { path: la(t, z.path, H.path).path });
    else {
      const y = ue({}, z.params);
      for (const R in y) y[R] == null && delete y[R];
      U = ue({}, z, { params: p(y) }), H.params = p(H.params);
    }
    const K = n.resolve(U, H), se = z.hash || "";
    K.params = d(m(K.params));
    const h = bd(s, ue({}, z, { hash: gd(se), path: K.path })), g = a.createHref(h);
    return ue({ fullPath: h, hash: se, query: s === Yi ? qd(z.query) : z.query || {} }, K, { redirectedFrom: void 0, href: g });
  }
  function $(z) {
    return typeof z == "string" ? la(t, z, l.value.path) : ue({}, z);
  }
  function A(z, H) {
    if (c !== z) return vt(be.NAVIGATION_CANCELLED, { from: H, to: z });
  }
  function P(z) {
    return N(z);
  }
  function O(z) {
    return P(ue($(z), { replace: true }));
  }
  function G(z, H) {
    const U = z.matched[z.matched.length - 1];
    if (U && U.redirect) {
      const { redirect: K } = U;
      let se = typeof K == "function" ? K(z, H) : K;
      return typeof se == "string" && (se = se.includes("?") || se.includes("#") ? se = $(se) : { path: se }, se.params = {}), ue({ query: z.query, hash: z.hash, params: se.path != null ? {} : z.params }, se);
    }
  }
  function N(z, H) {
    const U = c = j(z), K = l.value, se = z.state, h = z.force, g = z.replace === true, y = G(U, K);
    if (y) return N(ue($(y), { state: typeof y == "object" ? ue({}, se, y.state) : se, force: h, replace: g }), H || U);
    const R = U;
    R.redirectedFrom = H;
    let I;
    return !h && xd(s, K, U) && (I = vt(be.NAVIGATION_DUPLICATED, { to: R, from: K }), F(K, K, true, false)), (I ? Promise.resolve(I) : Se(R, K)).catch((C) => An(C) ? An(C, be.NAVIGATION_GUARD_REDIRECT) ? C : q(C) : ie(C, R, K)).then((C) => {
      if (C) {
        if (An(C, be.NAVIGATION_GUARD_REDIRECT)) return N(ue({ replace: g }, $(C.to), { state: typeof C.to == "object" ? ue({}, se, C.to.state) : se, force: h }), H || R);
      } else C = He(R, K, true, g, se);
      return Ye(R, K, C), C;
    });
  }
  function ke(z, H) {
    const U = A(z, H);
    return U ? Promise.reject(U) : Promise.resolve();
  }
  function Ee(z) {
    const H = Ue.values().next().value;
    return H && typeof H.runWithContext == "function" ? H.runWithContext(z) : z();
  }
  function Se(z, H) {
    let U;
    const [K, se, h] = Dd(z, H);
    U = ca(K.reverse(), "beforeRouteLeave", z, H);
    for (const y of K) y.leaveGuards.forEach((R) => {
      U.push(Vn(R, z, H));
    });
    const g = ke.bind(null, z, H);
    return U.push(g), Fe(U).then(() => {
      U = [];
      for (const y of i.list()) U.push(Vn(y, z, H));
      return U.push(g), Fe(U);
    }).then(() => {
      U = ca(se, "beforeRouteUpdate", z, H);
      for (const y of se) y.updateGuards.forEach((R) => {
        U.push(Vn(R, z, H));
      });
      return U.push(g), Fe(U);
    }).then(() => {
      U = [];
      for (const y of h) if (y.beforeEnter) if (rn(y.beforeEnter)) for (const R of y.beforeEnter) U.push(Vn(R, z, H));
      else U.push(Vn(y.beforeEnter, z, H));
      return U.push(g), Fe(U);
    }).then(() => (z.matched.forEach((y) => y.enterCallbacks = {}), U = ca(h, "beforeRouteEnter", z, H, Ee), U.push(g), Fe(U))).then(() => {
      U = [];
      for (const y of o.list()) U.push(Vn(y, z, H));
      return U.push(g), Fe(U);
    }).catch((y) => An(y, be.NAVIGATION_CANCELLED) ? y : Promise.reject(y));
  }
  function Ye(z, H, U) {
    r.list().forEach((K) => Ee(() => K(z, H, U)));
  }
  function He(z, H, U, K, se) {
    const h = A(z, H);
    if (h) return h;
    const g = H === Mn, y = rt ? history.state : {};
    U && (K || g ? a.replace(z.fullPath, ue({ scroll: g && y && y.scroll }, se)) : a.push(z.fullPath, se)), l.value = z, F(z, H, U, g), q();
  }
  let Le;
  function en() {
    Le || (Le = a.listen((z, H, U) => {
      if (!Sn.listening) return;
      const K = j(z), se = G(K, Sn.currentRoute.value);
      if (se) {
        N(ue(se, { replace: true, force: true }), K).catch(Kt);
        return;
      }
      c = K;
      const h = l.value;
      rt && zd(Ji(h.fullPath, U.delta), Bs()), Se(K, h).catch((g) => An(g, be.NAVIGATION_ABORTED | be.NAVIGATION_CANCELLED) ? g : An(g, be.NAVIGATION_GUARD_REDIRECT) ? (N(ue($(g.to), { force: true }), K).then((y) => {
        An(y, be.NAVIGATION_ABORTED | be.NAVIGATION_DUPLICATED) && !U.delta && U.type === Ea.pop && a.go(-1, false);
      }).catch(Kt), Promise.reject()) : (U.delta && a.go(-U.delta, false), ie(g, K, h))).then((g) => {
        g = g || He(K, h, false), g && (U.delta && !An(g, be.NAVIGATION_CANCELLED) ? a.go(-U.delta, false) : U.type === Ea.pop && An(g, be.NAVIGATION_ABORTED | be.NAVIGATION_DUPLICATED) && a.go(-1, false)), Ye(K, h, g);
      }).catch(Kt);
    }));
  }
  let un = Ct(), we = Ct(), re;
  function ie(z, H, U) {
    q(z);
    const K = we.list();
    return K.length ? K.forEach((se) => se(z, H, U)) : console.error(z), Promise.reject(z);
  }
  function Oe() {
    return re && l.value !== Mn ? Promise.resolve() : new Promise((z, H) => {
      un.add([z, H]);
    });
  }
  function q(z) {
    return re || (re = !z, en(), un.list().forEach(([H, U]) => z ? U(z) : H()), un.reset()), z;
  }
  function F(z, H, U, K) {
    const { scrollBehavior: se } = e;
    if (!rt || !se) return Promise.resolve();
    const h = !U && Id(Ji(z.fullPath, 0)) || (K || !U) && history.state && history.state.scroll || null;
    return as().then(() => se(z, H, h)).then((g) => g && Rd(g)).catch((g) => ie(g, z, H));
  }
  const J = (z) => a.go(z);
  let oe;
  const Ue = /* @__PURE__ */ new Set(), Sn = { currentRoute: l, listening: true, addRoute: f, removeRoute: k, clearRoutes: n.clearRoutes, hasRoute: W, getRoutes: T, resolve: j, options: e, push: P, replace: O, go: J, back: () => J(-1), forward: () => J(1), beforeEach: i.add, beforeResolve: o.add, afterEach: r.add, onError: we.add, isReady: Oe, install(z) {
    z.component("RouterLink", Un), z.component("RouterView", ul), z.config.globalProperties.$router = Sn, Object.defineProperty(z.config.globalProperties, "$route", { enumerable: true, get: () => L(l) }), rt && !oe && l.value === Mn && (oe = true, P(a.location).catch((K) => {
    }));
    const H = {};
    for (const K in Mn) Object.defineProperty(H, K, { get: () => l.value[K], enumerable: true });
    z.provide(Ks, Sn), z.provide(ai, pr(H)), z.provide(Ca, l);
    const U = z.unmount;
    Ue.add(z), z.unmount = function() {
      Ue.delete(z), Ue.size < 1 && (c = Mn, Le && Le(), Le = null, l.value = Mn, oe = false, re = false), U();
    };
  } };
  function Fe(z) {
    return z.reduce((H, U) => H.then(() => Ee(U)), Promise.resolve());
  }
  return Sn;
}
function cl() {
  return sn(Ks);
}
function cp(e) {
  return sn(ai);
}
const dp = { class: "app-shell" }, pp = { class: "topbar", role: "banner" }, hp = { class: "nav", "aria-label": "\u4E3B\u5BFC\u822A" }, fp = { class: "main", id: "main", tabindex: "-1" }, mp = De({ __name: "App", setup(e) {
  return (n, t) => (w(), x("div", dp, [u("header", pp, [ve(L(Un), { class: "brand", to: "/", "aria-label": "\u65E0\u540D\u6740 Next \u9996\u9875" }, { default: Ln(() => [...t[0] || (t[0] = [ae(" \u65E0\u540D\u6740 ", -1), u("span", { class: "accent" }, "Next", -1)])]), _: 1 }), u("nav", hp, [ve(L(Un), { to: "/" }, { default: Ln(() => [...t[1] || (t[1] = [ae("\u5927\u5385", -1)])]), _: 1 }), ve(L(Un), { to: "/battle" }, { default: Ln(() => [...t[2] || (t[2] = [ae("\u5BF9\u5C40", -1)])]), _: 1 }), ve(L(Un), { to: "/extensions" }, { default: Ln(() => [...t[3] || (t[3] = [ae("\u6269\u5C55", -1)])]), _: 1 }), ve(L(Un), { to: "/settings" }, { default: Ln(() => [...t[4] || (t[4] = [ae("\u8BBE\u7F6E", -1)])]), _: 1 }), ve(L(Un), { to: "/replay" }, { default: Ln(() => [...t[5] || (t[5] = [ae("\u5F55\u50CF", -1)])]), _: 1 }), ve(L(Un), { to: "/collection" }, { default: Ln(() => [...t[6] || (t[6] = [ae("\u6B66\u5C06\u5E93", -1)])]), _: 1 })])]), u("main", fp, [ve(L(ul))])]));
} });
async function jt(e, n = {}, t) {
  return window.__TAURI_INTERNALS__.invoke(e, n, t);
}
function gp(e, n = "asset") {
  return window.__TAURI_INTERNALS__.convertFileSrc(e, n);
}
function jn() {
  return typeof window < "u" && "__TAURI_INTERNALS__" in window;
}
function _p() {
  return jn() ? jt("platform_info") : Promise.resolve(null);
}
function lo(e) {
  return jn() ? jt("set_match_frozen", { frozen: e }) : Promise.resolve();
}
function vp() {
  return jn() ? jt("content_bundle") : Promise.resolve(null);
}
function yp(e, n) {
  return jn() ? jt("content_catalog_page", { offset: e, limit: n }) : Promise.resolve(null);
}
async function kp(e) {
  if (!jn()) return null;
  const n = await jt("resolve_content_asset", { namespace: e.namespace, packageId: e.package_id, assetId: e.asset_id });
  return gp(n);
}
const wp = { class: "stack" }, bp = { class: "panel hero" }, xp = { key: 0, class: "shell-badge" }, $p = { class: "row form" }, jp = ["value"], Sp = { class: "field" }, Ep = { class: "panel" }, Ap = { class: "online-grid" }, Cp = { class: "wide" }, Rp = ["value"], zp = { class: "row online-actions" }, Ip = { key: 0, class: "error", role: "alert" }, Tp = De({ __name: "HomeView", setup(e) {
  const n = cl(), t = B(4), s = B(42), a = B("prototype"), i = B(`player-${Math.floor(Math.random() * 900 + 100)}`), o = B(""), r = B(1), l = B(30), c = B(k()), d = B(""), p = B(null);
  $t(async () => {
    p.value = await _p();
  });
  function m() {
    n.push({ path: "/battle", query: { players: String(t.value), seed: String(s.value), ...a.value === "identity" ? { mode: "identity" } : {} } });
  }
  function f(W) {
    if (d.value = "", !c.value.trim()) {
      d.value = "\u8BF7\u586B\u5199 WebSocket \u5730\u5740";
      return;
    }
    if (W !== "create" && r.value < 1) {
      d.value = "\u623F\u95F4\u53F7\u5FC5\u987B\u5927\u4E8E 0";
      return;
    }
    sessionStorage.setItem("noname-next.online-intent", JSON.stringify({ action: W, wsUrl: c.value, displayName: i.value, password: o.value, roomId: r.value, players: t.value, seed: s.value, choiceTimeoutSecs: l.value })), n.push({ path: "/battle", query: { mode: "online", action: W, ...W === "create" ? {} : { room: String(r.value) } } });
  }
  function k() {
    if (typeof window > "u") return "ws://127.0.0.1:7700/ws";
    const W = window.location.protocol === "https:" ? "wss:" : "ws:", j = window.location.hostname || "127.0.0.1";
    return `${W}//${j}:7700/ws`;
  }
  const T = X(() => r.value > 0 ? `#${r.value}` : "\u2014");
  return (W, j) => (w(), x("div", wp, [u("section", bp, [j[15] || (j[15] = u("h1", null, "\u786E\u5B9A\u6027\u8EAB\u4EFD\u5C40", -1)), j[16] || (j[16] = u("p", { class: "muted" }, [ae(" \u9ED8\u8BA4\u539F\u751F\u5F15\u64CE\u540E\u7AEF\uFF08engine-native-wasm \u7A84\u58F3\uFF0CWorker \u5185\u8FD0\u884C\uFF09\u3002\u754C\u9762\u53EA\u6D88\u8D39\u89C2\u5BDF\u8005\u6295\u5F71\uFF0C\u4E0D\u6301\u6709\u5B8C\u6574\u6743\u5A01\u72B6\u6001\u3002\u53EF\u7528 "), u("code", null, "VITE_ENGINE_BACKEND=legacy"), ae(" \u56DE\u9000\u65E7\u7EBF\u3002 ")], -1)), p.value ? (w(), x("p", xp, " Tauri 2 \xB7 " + b(p.value.os) + " \xB7 " + b(p.value.arch), 1)) : V("", true), u("div", $p, [u("label", null, [j[11] || (j[11] = ae(" \u4EBA\u6570 ", -1)), ye(u("select", { "onUpdate:modelValue": j[0] || (j[0] = ($) => t.value = $), "aria-label": "\u73A9\u5BB6\u4EBA\u6570" }, [(w(), x(te, null, ge([2, 3, 4, 5, 6, 7, 8], ($) => u("option", { key: $, value: $ }, b($), 9, jp)), 64))], 512), [[vn, t.value, void 0, { number: true }]])]), u("label", null, [j[12] || (j[12] = ae(" \u79CD\u5B50 ", -1)), ye(u("input", { "onUpdate:modelValue": j[1] || (j[1] = ($) => s.value = $), type: "number", min: "0", "aria-label": "\u968F\u673A\u79CD\u5B50" }, null, 512), [[Wn, s.value, void 0, { number: true }]])]), u("label", Sp, [j[14] || (j[14] = u("span", null, "\u6A21\u5F0F", -1)), ye(u("select", { "onUpdate:modelValue": j[2] || (j[2] = ($) => a.value = $), "aria-label": "\u5BF9\u5C40\u6A21\u5F0F" }, [...j[13] || (j[13] = [u("option", { value: "prototype" }, "\u539F\u578B\uFF08\u552F\u4E00\u5B58\u6D3B\u8005\uFF09", -1), u("option", { value: "identity" }, "\u8EAB\u4EFD\u5C40\uFF08\u4E3B\u5FE0/\u53CD\u8D3C/\u5185\u5978\uFF09", -1)])], 512), [[vn, a.value]])]), u("button", { class: "primary", type: "button", onClick: m }, " \u672C\u5730\u5F00\u5C40 ")])]), u("section", Ep, [j[21] || (j[21] = u("h2", null, "\u8054\u673A\u5927\u5385", -1)), j[22] || (j[22] = u("p", { class: "muted" }, " \u901A\u8FC7 game-server WebSocket \u52A0\u5165\u6743\u5A01\u623F\u95F4\u3002\u6D4F\u89C8\u5668\u4EC5\u63D0\u4EA4\u547D\u4EE4\u5E76\u63A5\u6536\u81EA\u5DF1\u7684\u89C2\u5BDF\u8005\u6295\u5F71\u3002 ", -1)), u("div", Ap, [u("label", Cp, [j[17] || (j[17] = ae(" \u670D\u52A1\u7AEF ", -1)), ye(u("input", { "onUpdate:modelValue": j[3] || (j[3] = ($) => c.value = $), type: "url", autocomplete: "url" }, null, 512), [[Wn, c.value, void 0, { trim: true }]])]), u("label", null, [j[18] || (j[18] = ae(" \u6635\u79F0 ", -1)), ye(u("input", { "onUpdate:modelValue": j[4] || (j[4] = ($) => i.value = $), maxlength: "24", autocomplete: "nickname" }, null, 512), [[Wn, i.value, void 0, { trim: true }]])]), u("label", null, [j[19] || (j[19] = ae(" \u9009\u62E9\u65F6\u9650 ", -1)), ye(u("select", { "onUpdate:modelValue": j[5] || (j[5] = ($) => l.value = $), "aria-label": "\u9009\u62E9\u65F6\u9650" }, [(w(), x(te, null, ge([10, 15, 30, 60, 90], ($) => u("option", { key: $, value: $ }, b($) + " \u79D2", 9, Rp)), 64))], 512), [[vn, l.value, void 0, { number: true }]])]), u("label", null, [j[20] || (j[20] = ae(" \u623F\u95F4\u5BC6\u7801 ", -1)), ye(u("input", { "onUpdate:modelValue": j[6] || (j[6] = ($) => o.value = $), type: "password", autocomplete: "current-password" }, null, 512), [[Wn, o.value]])]), u("label", null, [ae(" \u623F\u95F4\u53F7 " + b(T.value) + " ", 1), ye(u("input", { "onUpdate:modelValue": j[7] || (j[7] = ($) => r.value = $), type: "number", min: "1", inputmode: "numeric" }, null, 512), [[Wn, r.value, void 0, { number: true }]])])]), u("div", zp, [u("button", { class: "primary", type: "button", onClick: j[8] || (j[8] = ($) => f("create")) }, "\u521B\u5EFA\u5E76\u5165\u5EA7"), u("button", { type: "button", onClick: j[9] || (j[9] = ($) => f("join")) }, "\u52A0\u5165\u623F\u95F4"), u("button", { type: "button", onClick: j[10] || (j[10] = ($) => f("spectate")) }, "\u65C1\u89C2")]), d.value ? (w(), x("p", Ip, b(d.value), 1)) : V("", true)]), j[23] || (j[23] = u("section", { class: "panel" }, [u("h2", null, "\u67B6\u6784\u8981\u70B9"), u("ul", { class: "muted list" }, [u("li", null, "\u4E3B\u7EBF\u7A0B\uFF1AVue Shell + DOM \u6218\u6597\u5C42 + \u52A8\u753B\u65F6\u95F4\u7EBF"), u("li", null, "\u672C\u5730\uFF1AWorker + \u539F\u751F\u7A84\u58F3\uFF08\u9ED8\u8BA4\uFF09/ \u65E7\u7EBF WASM\uFF1B\u8054\u673A\uFF1AWebSocket + \u6743\u5A01\u670D\u52A1\u5668"), u("li", null, "\u6269\u5C55\u5305\u7BA1\u7406\uFF1A\u5B89\u88C5 / \u542F\u505C / \u5347\u7EA7 / \u5378\u8F7D\uFF08\u5BFC\u822A\u300C\u6269\u5C55\u300D\uFF09"), u("li", null, "\u652F\u6301\u81EA\u52A8\u6B65\u8FDB\u3001\u5B58\u8BFB\u6863\u3001\u500D\u901F\u3001\u8DF3\u8FC7\u52A8\u753B\u3001\u51CF\u5C11\u52A8\u6001\u6548\u679C")])], -1))]));
} }), ln = (e, n) => {
  const t = e.__vccOpts || e;
  for (const [s, a] of n) t[s] = a;
  return t;
}, Pp = ln(Tp, [["__scopeId", "data-v-bb2f538c"]]), uo = { confirm: "Confirm", select_cards: "SelectCards", select_players: "SelectTargets", respond_card: "Custom", pick_tag: "Custom" };
function Op(e) {
  if (e === void 0) return [null, null];
  if (!Array.isArray(e) || e.length !== 2 || !e.every((n) => n === null || typeof n == "object" && !Array.isArray(n) && Number.isInteger(n.def) && n.def >= 0 && n.def <= 4294967295 && typeof n.name == "string" && typeof n.faction == "string" && typeof n.revealed == "boolean")) throw new Error("native: generals metadata invalid");
  return e;
}
function Np(e) {
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
function qp(e) {
  return e.prompt && Object.hasOwn(po, e.prompt) ? po[e.prompt] : Ra(e);
}
function dl(e, n) {
  for (const t of e.seats ?? []) Op(t.generals);
  if (n && (Np(n), n.deadline_unix_ms != null && (!Number.isSafeInteger(n.deadline_unix_ms) || n.deadline_unix_ms <= 0))) throw new Error("native: choice deadline invalid");
}
const ht = (e) => new Promise((n) => setTimeout(n, e)), Gs = "\u81EA\u52A8\u4E00\u6B65", Mp = "\u8FDE\u7EED\u81EA\u52A8", Js = /终局/;
let pl = [], za = 0;
function rs(e) {
  return Array.from(document.querySelectorAll("button")).find((n) => (n.innerText || "").trim() === e) ?? null;
}
function ho() {
  return new Promise((e) => {
    const n = () => {
      const a = rs(Gs);
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
function Dp() {
  const e = Ia(), n = document.body.innerText.match(/终局[^\n]*/) ?? document.body.innerText.match(/rev \d+[^\n]*/);
  return { frames: (e == null ? void 0 : e.frames()) ?? [], longTasks: (e == null ? void 0 : e.longTasks()) ?? [], eventTimings: (e == null ? void 0 : e.eventTimings()) ?? [], interactions: (e == null ? void 0 : e.interactions()) ?? [], status: (n == null ? void 0 : n[0]) ?? null, ua: navigator.userAgent, dpr: window.devicePixelRatio, viewport: [window.innerWidth, window.innerHeight], href: location.href, engine_transport: globalThis.__engineTransport ?? null, memory: null, memory_series: [] };
}
async function Lp() {
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
async function Up(e, n) {
  try {
    await fetch(e, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(n) });
  } catch {
  }
}
async function Fp(e, n) {
  for (let t = 0; t < e; t++) {
    const s = rs(Gs);
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
async function Vp() {
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
    const l = rs(Mp);
    l && l.click();
    let c = 0;
    for (; Date.now() < t && !s(); ) await ht(1e3), Date.now() - c > 3e4 && (c = Date.now(), await r());
    if (!e || Date.now() >= t) return pl = a, s() ? "finished" : "auto_timeout";
    (_a2 = Array.from(document.querySelectorAll("button")).find((p) => (p.innerText || "").includes("\u91CD\u65B0\u5F00\u5C40"))) == null ? void 0 : _a2.click(), await ht(1500);
  }
}
function Hp(e, n) {
  return new Promise((t) => {
    let s = 0;
    const a = (r) => {
      window.removeEventListener("click", i, true), t(r);
    }, i = (r) => {
      const l = r.target, c = rs(Gs);
      c && (l === c || c.contains(l)) && (s += 1, s >= e && setTimeout(() => a("manual_steps"), 800));
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
function Wp() {
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
      const m = rs(Gs);
      if (m && Ia() && !m.disabled) break;
      await ht(200);
    }
    (_a2 = Ia()) == null ? void 0 : _a2.reset(), await ht(300);
    const r = Date.now();
    za = r;
    let l = "unknown";
    n === "click" ? l = await Fp(s, a) : n === "auto" ? l = await Vp() : n === "manual" && (l = await Hp(s, i)), await ht(600);
    const c = Dp();
    c.memory = await Lp(), c.memory_series = pl, t && await Up(t, { ...c, end_state: l, wall_ms: Date.now() - r });
    const d = window;
    d.__nonameAutoDriveDone = l;
  })();
}
function Bp(e) {
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
  return dl(e.view, e.choice), { ...e, view: { ...e.view, outcome: Bp(e.view.outcome) } };
}
function Kp(e, n) {
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
class Gp {
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
        if (t = await this.call("native_engine_request", { resourceId: s, request: n }), t = Kp(t, n), t.id !== n.id || typeof t.ok != "boolean") throw new Error("invalid IPC response");
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
function Jp(e) {
  const n = [];
  for (let t = 1; t < e.length; t++) n.push(e[t] - e[t - 1]);
  return n;
}
function Yp(e) {
  const n = Rs(e), t = (a) => e.length ? e.filter((i) => i > a).length / e.length : null, s = n.p50;
  return { ...n, refresh_estimate_ms: s, over_8_33_ratio: t(1e3 / 120), over_16_67_ratio: t(1e3 / 60), over_1_5x_refresh_ratio: s === null ? null : t(s * 1.5) };
}
const Cn = (e, n) => e === null || n === null ? null : e - n;
function mo(e) {
  const n = e.worker, t = e.input_ts ?? e.sent;
  return { id: e.id, type: e.type, input_kind: e.input_kind, input_to_send_ms: Cn(e.sent, e.input_ts), roundtrip_ms: Cn(e.received, e.sent), worker_total_ms: (n == null ? void 0 : n.total_ms) ?? null, dispatch_ms: (n == null ? void 0 : n.dispatch_ms) ?? null, view_json_ms: (n == null ? void 0 : n.view_json_ms) ?? null, view_parse_ms: (n == null ? void 0 : n.view_parse_ms) ?? null, adapt_ms: (n == null ? void 0 : n.adapt_ms) ?? null, post_ms: (n == null ? void 0 : n.post_ms) ?? null, transfer_ms: Cn(e.received_epoch_ms, (n == null ? void 0 : n.post_epoch_ms) ?? null), receive_to_apply_ms: Cn(e.apply_start, e.received), apply_ms: Cn(e.apply_end, e.apply_start), vue_patch_ms: Cn(e.patched, e.apply_end), patch_to_frame_ms: Cn(e.frame, e.patched), frame_to_paint_ms: Cn(e.after_paint, e.frame), input_to_paint_ms: Cn(e.after_paint, t), view_json_chars: (n == null ? void 0 : n.view_json_chars) ?? null, envelope_json_chars: (n == null ? void 0 : n.envelope_json_chars) ?? null };
}
function Qp(e) {
  const n = ["input_to_send_ms", "roundtrip_ms", "worker_total_ms", "dispatch_ms", "view_json_ms", "view_parse_ms", "adapt_ms", "post_ms", "transfer_ms", "receive_to_apply_ms", "apply_ms", "vue_patch_ms", "patch_to_frame_ms", "frame_to_paint_ms", "input_to_paint_ms", "view_json_chars", "envelope_json_chars"], t = {};
  for (const s of n) {
    const a = e.map((i) => i[s]).filter((i) => typeof i == "number" && Number.isFinite(i));
    t[s] = Rs(a);
  }
  return t;
}
const Xp = "noname-next-perf", Zp = 3e5, eh = 2e4, Ta = 5e4, nh = 100;
let go = false, Xe = false;
const Rt = [], zt = [], It = [], wn = /* @__PURE__ */ new Map(), yt = /* @__PURE__ */ new Map();
let Nt = null, kt = null;
function th() {
  var _a2;
  try {
    if (((_a2 = globalThis.localStorage) == null ? void 0 : _a2.getItem(Xp)) === "1") return true;
  } catch {
  }
  if (typeof location > "u") return false;
  if (new URLSearchParams(location.search).get("perf") === "1") return true;
  const e = location.hash.split("?")[1];
  return e ? new URLSearchParams(e).get("perf") === "1" : false;
}
const sh = () => performance.timeOrigin + performance.now();
function _o(e, n, t) {
  var _a2;
  if (typeof PerformanceObserver > "u" || !((_a2 = PerformanceObserver.supportedEntryTypes) == null ? void 0 : _a2.includes(e))) return;
  new PerformanceObserver((a) => {
    for (const i of a.getEntries()) t(i);
  }).observe({ type: e, buffered: true, ...n });
}
function vo(e) {
  Nt = { ts: e.timeStamp, kind: e.type, used: false };
}
function ah() {
  if (go || (go = true, Xe = th(), !Xe || typeof window > "u")) return Xe;
  const e = (n) => {
    Rt.length < Zp && Rt.push(n), requestAnimationFrame(e);
  };
  return requestAnimationFrame(e), window.addEventListener("click", vo, { capture: true }), window.addEventListener("keydown", vo, { capture: true }), _o("longtask", {}, (n) => {
    zt.length < Ta && zt.push({ start: n.startTime, duration: n.duration });
  }), _o("event", { durationThreshold: 16 }, (n) => {
    const t = n;
    It.length < Ta && It.push({ name: t.name, start: t.startTime, duration: t.duration, input_delay: t.processingStart - t.startTime, processing: t.processingEnd - t.processingStart });
  }), window.__nonamePerf = { version: 1, enabled: true, frames: () => Rt.slice(), longTasks: () => zt.slice(), eventTimings: () => It.slice(), interactions: () => [...wn.values()].map(mo), summary: () => {
    const n = [...wn.values()].map(mo);
    return { frames: Yp(Jp(Rt)), interactions: Qp(n), long_tasks: Rs(zt.map((t) => t.duration)), event_timings: Rs(It.map((t) => t.duration)) };
  }, reset: () => {
    Rt.length = 0, zt.length = 0, It.length = 0, wn.clear(), yt.clear(), kt = null;
  } }, Xe && Wp(), Xe;
}
function ih(e, n) {
  if (!Xe || wn.size >= eh) return;
  const t = performance.now(), s = Nt && !Nt.used && t - Nt.ts <= nh ? Nt : null;
  s && (s.used = true), wn.set(e, { id: e, type: n, input_ts: (s == null ? void 0 : s.ts) ?? null, input_kind: (s == null ? void 0 : s.kind) ?? null, sent: t, received: null, received_epoch_ms: null, apply_start: null, apply_end: null, patched: null, frame: null, after_paint: null, worker: yt.get(e) ?? null }), yt.delete(e);
}
function yo(e) {
  Xe && (wn.delete(e), yt.delete(e));
}
function ko(e) {
  if (!Xe) return;
  const n = wn.get(e);
  n && (n.received = performance.now(), n.received_epoch_ms = sh(), kt = e);
}
function oh(e, n) {
  if (!Xe) return;
  const t = wn.get(e);
  t ? t.worker = n : yt.size < Ta && yt.set(e, n);
}
function rh() {
  if (kt === null) return null;
  const e = wn.get(kt);
  return e && e.apply_start === null ? e : null;
}
function lh() {
  if (!Xe) return;
  const e = rh();
  e && (e.apply_start = performance.now());
}
function uh() {
  if (!Xe || kt === null) return;
  const e = wn.get(kt);
  !e || e.apply_start === null || e.apply_end !== null || (e.apply_end = performance.now(), as(() => {
    e.patched = performance.now(), requestAnimationFrame(() => {
      e.frame = performance.now();
      const n = new MessageChannel();
      n.port1.onmessage = () => {
        e.after_paint = performance.now(), n.port1.close();
      }, n.port2.postMessage(0);
    });
  }));
}
function ch(e) {
  return new Worker("/assets/engine.native.worker-BIwpD907.js", { type: "module", name: e == null ? void 0 : e.name });
}
function dh(e) {
  return new Worker("/assets/engine.worker-DU8mrB9S.js", { type: "module", name: e == null ? void 0 : e.name });
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
    const t = !this.native && jn() ? await vp() : void 0, s = this.native && jn() && fl() === "native";
    this.transportKind = s ? "desktop-native" : this.native ? "wasm-worker" : "legacy-worker", globalThis.__engineTransport = this.transportKind;
    const a = s ? new Gp() : this.native ? new ch() : new dh();
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
        const c = i.data;
        oh(c.id, c.timings);
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
      await this.request({ id: this.nextId(), type: "init", bundle: t ?? void 0, perf: ah(), latencySab: s ? void 0 : this.initSabMirror() });
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
    }, l = { runs: 0, seen: 0, parseErr: 0, miss: 0 }, c = () => {
      l.runs += 1;
      const d = Atomics.load(i, 0);
      if (d !== this.sabPollSeq) {
        l.seen += 1, this.sabPollSeq = d;
        const p = Atomics.load(i, 1);
        if (p > 0 && 8 + p <= s) try {
          const m = new Uint8Array(p);
          m.set(new Uint8Array(a, 8, p));
          const f = JSON.parse(o.decode(m)), k = this.pending.get(f.id);
          if (k) {
            ko(f.id), this.pending.delete(f.id), r();
            const T = globalThis.__latencySabDebug;
            T && (T.resolvedEarly += 1), f.ok ? k.resolve(f.data) : k.reject(new Error(f.error ?? "worker error"));
          } else l.miss += 1;
        } catch {
          l.parseErr += 1;
        }
      }
      requestAnimationFrame(c);
    };
    return requestAnimationFrame(c), globalThis.__latencySabDebug = { seq: () => Atomics.load(i, 0), raw: () => {
      const d = Atomics.load(i, 1), p = new Uint8Array(Math.min(d, 120));
      return p.set(new Uint8Array(a, 8, p.length)), { len: d, head: o.decode(p) };
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
      this.pending.set(n.id, { resolve: t, reject: s }), ih(n.id, n.type), (_b2 = (_a2 = globalThis.__latencySabDebug) == null ? void 0 : _a2.wake) == null ? void 0 : _b2.call(_a2);
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
const Pa = 200, Q = xt({ contentLockHash: "", total: 0, pages: /* @__PURE__ */ new Map(), characters: /* @__PURE__ */ new Map(), skins: /* @__PURE__ */ new Map(), translations: {}, selectedSkins: /* @__PURE__ */ new Map(), portraitUrls: /* @__PURE__ */ new Map(), imageUrls: /* @__PURE__ */ new Map(), substituteUrls: /* @__PURE__ */ new Map() });
let Bn = null;
const qt = /* @__PURE__ */ new Map(), Mt = /* @__PURE__ */ new Map(), Oa = /* @__PURE__ */ new Map();
function ph(e) {
  if (jn()) {
    Ma({ kind: "desktop", key: "desktop" });
    return;
  }
  const n = `${window.location.protocol === "https:" ? "https:" : "http:"}//${window.location.hostname || "127.0.0.1"}:7700`;
  Ma({ kind: "http", key: `http:${n}`, baseUrl: n });
}
function hh(e) {
  const n = new URL(e.trim(), window.location.href);
  n.protocol = n.protocol === "wss:" || n.protocol === "https:" ? "https:" : "http:";
  const t = n.origin;
  Ma({ kind: "http", key: `http:${t}`, baseUrl: t });
}
async function fh(e) {
  const n = Q.characters.get(e);
  if (n) return Q.imageUrls.has(e) || await qa(e), n;
  if (!Bn) return;
  const t = Mt.get(e);
  if (t) return t;
  const s = Bn.key, a = (async () => {
    let i = 0;
    for (; (Bn == null ? void 0 : Bn.key) === s; ) {
      const o = await ml(i);
      if (!o) break;
      const r = Q.characters.get(e);
      if (r) return await qa(e), r;
      const l = o.offset + o.characters.length;
      if (l >= o.total || l <= i) break;
      i = l;
    }
  })();
  Mt.set(e, a);
  try {
    return await a;
  } finally {
    Mt.get(e) === a && Mt.delete(e);
  }
}
function mh(e) {
  var _a2;
  const n = (_a2 = Q.characters.get(e)) == null ? void 0 : _a2.definition.name_key;
  return n ? gl(n) : void 0;
}
function gh(e) {
  var _a2;
  return ((_a2 = Q.characters.get(e)) == null ? void 0 : _a2.skin_ids.flatMap((t) => {
    const s = Q.skins.get(t);
    return s && s.presentation.kind !== "substitute" ? [s] : [];
  })) ?? [];
}
function _h(e) {
  return gl(e.name_key);
}
function Na(e) {
  const n = Q.characters.get(e);
  if (!n || !Q.contentLockHash) return null;
  if (Q.selectedSkins.has(e)) return Q.selectedSkins.get(e) ?? null;
  const t = _l(e);
  let s = null;
  if (t) try {
    const a = Number(localStorage.getItem(t));
    Number.isInteger(a) && vl(n).includes(a) && (s = a);
  } catch {
  }
  return Q.selectedSkins.set(e, s), s;
}
function vh(e, n) {
  const t = Q.characters.get(e);
  if (n != null && (!t || !vl(t).includes(n))) return;
  Q.selectedSkins.set(e, n);
  const s = _l(e);
  if (s) try {
    n == null ? localStorage.removeItem(s) : localStorage.setItem(s, String(n));
  } catch {
  }
  qa(e);
}
function yh(e) {
  return Q.imageUrls.get(e) ?? Ys(e);
}
function kh(e) {
  return Q.portraitUrls.get(e) ?? Ys(e);
}
function wh(e, n) {
  return Q.substituteUrls.get(`${e}:${n}`) ?? null;
}
function Ys(e) {
  return `/original/portraits/${e}.jpg`;
}
async function ml(e) {
  const n = Q.pages.get(e);
  if (n) return n;
  const t = qt.get(e);
  if (t) return t;
  const s = Bn;
  if (!s) return null;
  const a = (async () => {
    let i;
    if (s.kind === "desktop") i = await yp(e, Pa);
    else {
      const o = new URL("/content-catalog", s.baseUrl);
      o.searchParams.set("offset", String(e)), o.searchParams.set("limit", String(Pa));
      const r = await fetch(o);
      if (!r.ok) throw new Error(`\u5185\u5BB9\u76EE\u5F55\u52A0\u8F7D\u5931\u8D25 (${r.status})`);
      i = await r.json();
    }
    if (Bn !== s || i == null) return null;
    if (xh(i), i.offset !== Math.min(e, i.total)) throw new Error("\u5185\u5BB9\u76EE\u5F55\u5206\u9875\u504F\u79FB\u4E0D\u4E00\u81F4");
    return e !== 0 && Q.contentLockHash && Q.contentLockHash !== i.content_lock_hash ? (oi(), ml(0)) : (bh(i), i);
  })();
  qt.set(e, a);
  try {
    return await a;
  } finally {
    qt.get(e) === a && qt.delete(e);
  }
}
function bh(e) {
  Q.contentLockHash && Q.contentLockHash !== e.content_lock_hash && oi(), Q.contentLockHash = e.content_lock_hash, Q.total = e.total, Q.pages.set(e.offset, e);
  for (const n of e.characters) Q.characters.set(n.definition.id, n);
  for (const n of e.skins) Q.skins.set(n.id, n);
  for (const [n, t] of Object.entries(e.translations)) Q.translations[n] = { ...Q.translations[n], ...t };
}
async function qa(e) {
  const n = Q.characters.get(e);
  if (!n) return;
  const t = Na(e), s = t == null ? void 0 : Q.skins.get(t), a = (s == null ? void 0 : s.presentation.kind) === "static" ? s.presentation.asset : null, i = Ys(e);
  Q.portraitUrls.set(e, i), Q.imageUrls.set(e, i);
  const o = `${Q.contentLockHash}:${t ?? "base"}`, r = n.portrait_asset ? await pa(n.portrait_asset).catch(() => null) : null, l = a ? await pa(a).catch(() => null) : null, c = n.skin_ids.map((m) => Q.skins.get(m)).find((m) => (m == null ? void 0 : m.presentation.kind) === "substitute" && m.presentation.rule === "dead"), d = (c == null ? void 0 : c.presentation.kind) === "substitute" ? c.presentation.asset : null, p = d ? await pa(d).catch(() => null) : null;
  if (o === `${Q.contentLockHash}:${Na(e) ?? "base"}`) {
    const m = r ?? i;
    Q.portraitUrls.set(e, m), Q.imageUrls.set(e, l ?? m), p && Q.substituteUrls.set(`${e}:dead`, p);
  }
}
function pa(e) {
  if (!$h(e) || !Bn) return Promise.resolve(null);
  const n = Bn, t = [n.key, e.namespace, e.package_id, e.asset_id].join("\0"), s = Oa.get(t);
  if (s) return s;
  const a = n.kind === "desktop" ? kp(e) : Promise.resolve(`${n.baseUrl}/package-assets/${[e.namespace, e.package_id].map(encodeURIComponent).join("/")}/${e.asset_id.split("/").map(encodeURIComponent).join("/")}?v=${encodeURIComponent(Q.contentLockHash)}`);
  return Oa.set(t, a), a;
}
function gl(e) {
  var _a2;
  const n = typeof navigator > "u" ? "" : navigator.language, t = [n, n.replace("-", "_"), "zh-CN", "zh_CN", "zh", "en"];
  for (const s of t) {
    const a = (_a2 = Q.translations[s]) == null ? void 0 : _a2[e];
    if (a) return a;
  }
  return e;
}
function _l(e) {
  return Q.contentLockHash ? `noname-next.skin.${Q.contentLockHash}.${e}` : null;
}
function Ma(e) {
  Bn = e, oi();
}
function oi() {
  Q.contentLockHash = "", Q.total = 0, Q.pages.clear(), Q.characters.clear(), Q.skins.clear(), Q.translations = {}, Q.selectedSkins.clear(), Q.portraitUrls.clear(), Q.imageUrls.clear(), Q.substituteUrls.clear(), qt.clear(), Mt.clear(), Oa.clear();
}
function xh(e) {
  const n = e;
  if (!n || typeof n.content_lock_hash != "string" || !Dn(n.offset) || !Dn(n.limit) || !Dn(n.total) || n.limit < 1 || n.limit > Pa || n.offset > n.total || !Array.isArray(n.characters) || n.characters.length > n.limit || !Array.isArray(n.skins) || !Array.isArray(n.aliases) || !n.translations || typeof n.translations != "object" || !n.characters.every((t) => {
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
    return ((_a2 = Q.skins.get(n)) == null ? void 0 : _a2.presentation.kind) !== "substitute";
  });
}
function bo(e) {
  return !!e && typeof e == "object" && !Array.isArray(e) && Object.values(e).every((n) => typeof n == "string");
}
function $h(e) {
  return typeof e.namespace == "string" && e.namespace.length > 0 && typeof e.package_id == "string" && e.package_id.length > 0 && typeof e.asset_id == "string" && e.asset_id.length > 0;
}
const Gt = [{ dir: "cardpile-upstream-liuli", packageId: "cardpile", namespace: "upstream-liuli", version: "1.0.0", title: "\u724C\u5806\uFF08\u7409\u7483\uFF09", logicFingerprint: "8dd99eb2" }, { dir: "cardpile-upstream-noname", packageId: "cardpile", namespace: "upstream-noname", version: "1.0.0", title: "\u724C\u5806\uFF08\u65E0\u540D\uFF09", logicFingerprint: "83aa81c4" }, { dir: "d3-upstream-noname", packageId: "d3-select", namespace: "upstream-noname", version: "1.0.0", title: "3D\u7CBE\u9009", logicFingerprint: "6f257a96" }, { dir: "huanle-upstream-noname", packageId: "huanle-cards", namespace: "upstream-noname", version: "1.0.0", title: "\u6B22\u4E50\u5361\u724C", logicFingerprint: "6ae1907c" }, { dir: "shaai-ow-upstream-noname", packageId: "shaai-ow", namespace: "upstream-noname", version: "1.0.0", title: "\u5B88\u671B\u5148\u950B", logicFingerprint: "6117a13e" }, { dir: "shaai-xianjian-upstream-noname", packageId: "shaai-xianjian", namespace: "upstream-noname", version: "1.0.0", title: "\u4ED9\u5251\u5947\u4FA0\u4F20", logicFingerprint: "d9ee1863" }, { dir: "shahai-gujian-upstream-noname", packageId: "shahai-gujian", namespace: "upstream-noname", version: "1.0.0", title: "\u53E4\u5251\u5947\u8C2D", logicFingerprint: "ae13018f" }, { dir: "shahai-gwent-upstream-noname", packageId: "shahai-gwent", namespace: "upstream-noname", version: "1.0.0", title: "\u6606\u7279\u724C", logicFingerprint: "0bd2569a" }, { dir: "shahai-hearth-upstream-noname", packageId: "shahai-hearth", namespace: "upstream-noname", version: "1.0.0", title: "\u7089\u77F3\u4F20\u8BF4", logicFingerprint: "deafc31c" }, { dir: "shahai-mtg-upstream-noname", packageId: "shahai-mtg", namespace: "upstream-noname", version: "1.0.0", title: "\u4E07\u667A\u724C", logicFingerprint: "6417be73" }, { dir: "shahai-swd-upstream-noname", packageId: "shahai-swd", namespace: "upstream-noname", version: "1.0.0", title: "\u8F69\u8F95\u5251", logicFingerprint: "31ccea98" }, { dir: "shahai-yunchou-upstream-noname", packageId: "shahai-yunchou", namespace: "upstream-noname", version: "1.0.0", title: "\u8FD0\u7B79\u5E37\u5E44", logicFingerprint: "57491089" }, { dir: "shahai-zhenfa-upstream-noname", packageId: "shahai-zhenfa", namespace: "upstream-noname", version: "1.0.0", title: "\u9635\u6CD5", logicFingerprint: "53945b47" }, { dir: "wudian-upstream-noname", packageId: "wudian", namespace: "upstream-noname", version: "1.0.0", title: "\u73A9\u70B9\u8BBA\u6740", logicFingerprint: "e0d488b6" }, { dir: "wuxing-upstream-liuli", packageId: "wuxing", namespace: "upstream-liuli", version: "1.0.0", title: "\u4E94\u884C", logicFingerprint: "845ea6fa" }];
function ri(e) {
  return `${e.namespace}/${e.packageId}`;
}
const jh = { package_format_version: 1, namespace: "upstream-liuli", package_id: "cardpile", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 753, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [141, 217, 158, 178, 102, 250, 23, 48, 46, 154, 229, 230, 185, 21, 142, 207, 121, 191, 21, 160, 114, 238, 113, 65, 184, 219, 63, 10, 143, 211, 102, 196], asset_hash: [5, 133, 68, 112, 135, 182, 90, 250, 76, 92, 107, 127, 63, 60, 145, 39, 21, 177, 136, 200, 170, 224, 209, 228, 167, 139, 124, 218, 97, 232, 139, 249], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, Sh = [], Eh = [], Ah = [], Ch = [], Rh = [], zh = [], Ih = [], Th = [], Ph = { "zh-CN": { cardpile: "\u724C\u5806", huosha: "\u706B\u6740", jiu: "\u9152", leisha: "\u96F7\u6740" } }, Oh = { schema_version: 1, package_id: "cardpile", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "cardpile:pile", source_hash: [223, 149, 103, 52, 153, 51, 90, 98, 219, 54, 199, 28, 186, 38, 121, 216, 91, 164, 250, 84, 188, 239, 98, 143, 239, 121, 48, 90, 56, 228, 25, 104], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, Nh = { "cardpile:pile": "assets/pile.json" }, qh = "gen-1", xo = { manifest: jh, modules: Sh, skill_index: Eh, character_index: Ah, card_index: Ch, trigger_index: Rh, modifier_index: zh, active_index: Ih, view_as_index: Th, translations: Ph, asset_manifest: Oh, asset_paths: Nh, compiler_version: qh }, Mh = "noname-liuli", Dh = "noname/extension/cardpile/extension.js", Lh = "8485dbb29ff16f9b258a766b1a17ef75f883aade", Uh = 160, Fh = [{ card: "sha", counts: { diamond: 6, club: 14, heart: 3, spade: 7 } }, { card: "huosha", counts: { diamond: 2, heart: 3 } }, { card: "leisha", counts: { spade: 5, club: 4 } }, { card: "shan", counts: { heart: 6, diamond: 18 } }, { card: "jiu", counts: { diamond: 1, spade: 2, club: 2 } }, { card: "tao", counts: { heart: 9, diamond: 3 } }, { card: "wanjian", counts: { heart: 1 } }, { card: "nanman", counts: { spade: 2, club: 1 } }, { card: "guohe", counts: { spade: 3, club: 2, heart: 1 } }, { card: "shunshou", counts: { spade: 3, diamond: 2 } }, { card: "wuxie", counts: { heart: 2, diamond: 1, spade: 2, club: 2 } }, { card: "tiesuo", counts: { spade: 2, club: 4 } }], Vh = { source: Mh, upstream_path: Dh, upstream_commit: Lh, total: Uh, entries: Fh }, Hh = { package_format_version: 1, namespace: "upstream-noname", package_id: "cardpile", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 753, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [131, 170, 129, 196, 137, 101, 152, 220, 92, 26, 119, 144, 188, 18, 226, 118, 124, 202, 213, 56, 37, 221, 167, 227, 108, 52, 173, 68, 188, 187, 99, 220], asset_hash: [128, 188, 216, 204, 239, 216, 63, 133, 226, 76, 81, 90, 3, 222, 209, 209, 187, 152, 128, 170, 74, 141, 193, 125, 195, 157, 166, 254, 81, 218, 111, 12], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, Wh = [], Bh = [], Kh = [], Gh = [], Jh = [], Yh = [], Qh = [], Xh = [], Zh = { "zh-CN": { cardpile: "\u724C\u5806", huosha: "\u706B\u6740", jiu: "\u9152", leisha: "\u96F7\u6740" } }, ef = { schema_version: 1, package_id: "cardpile", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "cardpile:pile", source_hash: [107, 194, 232, 96, 169, 46, 18, 235, 51, 137, 142, 3, 98, 227, 253, 110, 137, 37, 246, 88, 93, 161, 193, 213, 246, 39, 29, 229, 204, 37, 149, 34], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, nf = { "cardpile:pile": "assets/pile.json" }, tf = "gen-1", $o = { manifest: Hh, modules: Wh, skill_index: Bh, character_index: Kh, card_index: Gh, trigger_index: Jh, modifier_index: Yh, active_index: Qh, view_as_index: Xh, translations: Zh, asset_manifest: ef, asset_paths: nf, compiler_version: tf }, sf = "noname", af = "apps/core/extension/cardpile/extension.js", of = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", rf = 160, lf = [{ card: "sha", counts: { diamond: 6, club: 14, heart: 3, spade: 7 } }, { card: "huosha", counts: { diamond: 2, heart: 3 } }, { card: "leisha", counts: { spade: 5, club: 4 } }, { card: "shan", counts: { heart: 6, diamond: 18 } }, { card: "jiu", counts: { diamond: 1, spade: 2, club: 2 } }, { card: "tao", counts: { heart: 9, diamond: 3 } }, { card: "wanjian", counts: { heart: 1 } }, { card: "nanman", counts: { spade: 2, club: 1 } }, { card: "guohe", counts: { spade: 3, club: 2, heart: 1 } }, { card: "shunshou", counts: { spade: 3, diamond: 2 } }, { card: "wuxie", counts: { heart: 2, diamond: 1, spade: 2, club: 2 } }, { card: "tiesuo", counts: { spade: 2, club: 4 } }], uf = { source: sf, upstream_path: af, upstream_commit: of, total: rf, entries: lf }, cf = { package_format_version: 1, namespace: "upstream-noname", package_id: "d3-select", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [111, 37, 122, 150, 119, 48, 147, 56, 241, 49, 200, 32, 57, 124, 107, 75, 23, 118, 103, 218, 206, 252, 0, 59, 192, 24, 0, 59, 108, 78, 62, 128], asset_hash: [16, 5, 163, 133, 161, 153, 19, 47, 137, 85, 182, 46, 7, 149, 75, 218, 226, 85, 46, 131, 186, 196, 41, 78, 176, 6, 210, 119, 138, 54, 121, 189], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, df = [], pf = [], hf = [], ff = [], mf = [], gf = [], _f = [], vf = [], yf = { "zh-CN": { ddd_liuchong: "ddd\u5218\u51B2", dddjinggou: "\u7CBE\u5F40", dddmoyan: "\u672B\u7131" } }, kf = { schema_version: 1, package_id: "d3-select", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "d3-select:characters", source_hash: [50, 247, 124, 76, 209, 138, 160, 245, 215, 11, 118, 39, 184, 31, 122, 238, 107, 84, 39, 122, 206, 75, 86, 186, 245, 248, 189, 22, 28, 40, 135, 250], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, wf = { "d3-select:characters": "assets/character.json" }, bf = "gen-1", jo = { manifest: cf, modules: df, skill_index: pf, character_index: hf, card_index: ff, trigger_index: mf, modifier_index: gf, active_index: _f, view_as_index: vf, translations: yf, asset_manifest: kf, asset_paths: wf, compiler_version: bf }, xf = "noname", $f = "apps/core/extension/3D\u7CBE\u9009/", jf = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", Sf = { name: "d3-upstream-noname", version: 1, description: "G1 3D\u7CBE\u9009", card_prefix: "ddd_" }, Ef = [{ id: "ddd_liuchong", name: "ddd\u5218\u51B2", faction: "qun", hp: 4, gender: "male", skills: [{ name: "dddjinggou", title: "\u7CBE\u5F40", mapping: "host:max_range_weapon_damage" }, { name: "dddmoyan", title: "\u672B\u7131", mapping: "host:d3_moyan" }] }], Af = { source: xf, upstream_path: $f, upstream_commit: jf, pack: Sf, generals: Ef }, Cf = { package_format_version: 1, namespace: "upstream-noname", package_id: "huanle-cards", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [106, 225, 144, 124, 144, 13, 45, 180, 159, 95, 125, 53, 98, 74, 29, 117, 24, 49, 199, 10, 60, 244, 167, 94, 32, 142, 255, 23, 184, 119, 161, 55], asset_hash: [25, 84, 214, 199, 128, 168, 24, 45, 132, 188, 58, 79, 25, 130, 141, 162, 163, 191, 146, 92, 141, 235, 240, 14, 64, 255, 184, 47, 114, 101, 54, 250], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, Rf = [], zf = [], If = [], Tf = [], Pf = [], Of = [], Nf = [], qf = [], Mf = { "zh-CN": { huanle: "\u6B22\u4E50\u5361\u724C", jiuwei: "\u9152\u5C3E", kuwu: "\u82E6\u65E0", shoulijian: "\u624B\u91CC\u5251" } }, Df = { schema_version: 1, package_id: "huanle-cards", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "huanle-cards:cards", source_hash: [8, 4, 171, 213, 237, 11, 78, 79, 232, 10, 93, 164, 129, 27, 245, 153, 138, 180, 85, 242, 69, 243, 148, 105, 243, 156, 24, 165, 76, 211, 77, 17], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, Lf = { "huanle-cards:cards": "assets/cards.json" }, Uf = "gen-1", So = { manifest: Cf, modules: Rf, skill_index: zf, character_index: If, card_index: Tf, trigger_index: Pf, modifier_index: Of, active_index: Nf, view_as_index: qf, translations: Mf, asset_manifest: Df, asset_paths: Lf, compiler_version: Uf }, Ff = "noname", Vf = "apps/core/extension/\u6B22\u4E50\u5361\u724C/", Hf = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", Wf = { name: "huanle-upstream-noname", version: 1, description: "G1 \u6B22\u4E50\u5361\u724C", card_prefix: "hnl_" }, Bf = [{ name: "kuwu", kind: "weapon", range: 2, equip_skills: ["huanle_kuwu"], suit: "spade", rank: 6 }, { name: "shoulijian", kind: "shoulijian", suit: "diamond", rank: 4 }, { name: "shoulijian", kind: "shoulijian", suit: "spade", rank: 4 }, { name: "monkey", kind: "monkey", suit: "diamond", rank: 5 }, { name: "xuelunyang", kind: "xuelunyang", suit: "heart", rank: 2 }, { name: "mianju", kind: "mianju", suit: "club", rank: 3 }, { name: "jiuwei", kind: "jiuwei", suit: "heart", rank: 9, equip_skills: ["huanle_jiuwei"] }], Kf = [{ name: "huanle_kuwu", mapping: "host:sha_damage_force_discard" }, { name: "huanle_jiuwei", mapping: "host:phase_end_recover_or_draw" }], Gf = { source: Ff, upstream_path: Vf, upstream_commit: Hf, pack: Wf, cards: Bf, skills: Kf }, Jf = { package_format_version: 1, namespace: "upstream-noname", package_id: "shaai-ow", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [97, 23, 161, 62, 250, 238, 201, 81, 165, 68, 177, 185, 128, 62, 23, 122, 149, 20, 247, 219, 47, 165, 34, 207, 63, 74, 111, 21, 134, 193, 80, 44], asset_hash: [234, 208, 248, 98, 248, 214, 252, 238, 255, 142, 242, 125, 150, 77, 115, 114, 87, 46, 204, 14, 51, 9, 132, 104, 243, 158, 19, 237, 178, 87, 25, 1], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, Yf = [], Qf = [], Xf = [], Zf = [], em = [], nm = [], tm = [], sm = [], am = { "zh-CN": { diaobingqianjiang: "\u8C03\u5175\u9063\u5C06", sanjian: "\u4E09\u5C16", wangmeizhike: "\u671B\u6885\u6B62\u6E34", youdishenru: "\u8BF1\u654C\u6DF1\u5165" } }, im = { schema_version: 1, package_id: "shaai-ow", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "shaai-ow:cards", source_hash: [156, 170, 113, 113, 238, 249, 132, 22, 250, 155, 70, 184, 125, 243, 73, 245, 56, 16, 110, 98, 179, 110, 249, 104, 140, 229, 198, 10, 8, 170, 103, 219], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, om = { "shaai-ow:cards": "assets/cards.json" }, rm = "gen-1", Eo = { manifest: Jf, modules: Yf, skill_index: Qf, character_index: Xf, card_index: Zf, trigger_index: em, modifier_index: nm, active_index: tm, view_as_index: sm, translations: am, asset_manifest: im, asset_paths: om, compiler_version: rm }, lm = "qita/sanguosha/noname/extension/\u6740\u6D77\u62FE\u9057/character/ow", um = "extension/\u6740\u6D77\u62FE\u9057/character/ow/", cm = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", dm = { name: "shaai-ow-upstream-noname", version: 1, description: "G1 \u7EB5\u5207\uFF1A\u4E0A\u6E38 ow \u58EB\u517576\uFF08\u7A81\u51FB+\u76EE\u955C\uFF09", card_prefix: "ow_" }, pm = [], hm = [], fm = [], mm = [{ name: "guangshu", general: "ow_zhixuzhiguang", mapping: "host:wudian_guangshu" }, { name: "guangshu_heart", general: "ow_zhixuzhiguang", mapping: "host:wudian_guangshu_heart" }, { name: "guangshu_diamond", general: "ow_zhixuzhiguang", mapping: "host:wudian_guangshu_diamond" }, { name: "guangshu_club", general: "ow_zhixuzhiguang", mapping: "host:wudian_guangshu_club" }, { name: "guangshu_spade", general: "ow_zhixuzhiguang", mapping: "host:wudian_guangshu_spade" }, { name: "ziyu", general: "ow_chanyata", mapping: "host:wudian_ziyu" }, { name: "shouhu", general: "ow_tianshi", mapping: "host:wudian_shouhu" }, { name: "duwen", general: "ow_heibaihe", mapping: "host:wudian_duwen" }, { name: "paotai", general: "ow_tuobiang", mapping: "host:wudian_paotai" }, { name: "paotai2", general: "ow_tuobiang", mapping: "host:wudian_paotai_fire" }, { name: "bshaowei", general: "ow_baolei", mapping: "host:wudian_baolei_switch" }], gm = [{ name: "\u58EB\u517576", faction: "shu", hp: 4, gender: "male", skills: [{ name: "tuji", mapping: "host:wudian_tuji" }, { name: "mujing", mapping: "host:wudian_mujing" }, { name: "lichang", mapping: null, reason: "\u4E0B\u51C6\u5907\u9636\u6BB5\u5B50\u56DE\u5408\u65F6\u70B9\u672A\u5EFA\u6A21\u2014\u20142026-10-08 \u6682\u7F13" }] }, { name: "\u865A\u65E0\u4E4B\u5149", faction: "qun", hp: 3, gender: "female", skills: [{ name: "guangshu", mapping: "host:wudian_guangshu" }] }, { name: "\u7985\u96C5\u5854", faction: "qun", hp: 3, gender: "male", skills: [{ name: "ziyu", mapping: "host:wudian_ziyu" }] }, { name: "\u5929\u4F7F", faction: "shu", hp: 3, gender: "female", skills: [{ name: "shouhu", mapping: "host:wudian_shouhu" }, { name: "ziyu", mapping: "host:wudian_ziyu" }, { name: "feiying", mapping: null, reason: "\u6E90\u5B9A\u4E49\u4E0D\u5728 ow/skill.js\uFF08\u5171\u4EAB\u6280\u80FD\u6587\u4EF6\uFF09\u2014\u20142026-10-08 \u6682\u7F13" }] }, { name: "\u9ED1\u767E\u5408", faction: "wei", hp: 3, gender: "female", skills: [{ name: "duwen", mapping: "host:wudian_duwen" }] }, { name: "\u6258\u6BD4\u6602", faction: "shu", hp: 4, gender: "male", skills: [{ name: "paotai", mapping: "host:wudian_paotai" }, { name: "paotai2", mapping: "host:wudian_paotai_fire" }, { name: "maoding", mapping: null, reason: "\u96F6\u4EF6\u724C\u6C60\uFF08hslingjian\uFF09\u4E0D\u5728\u539F\u751F\u76EE\u5F55\u2014\u20142026-10-08 \u6682\u7F13" }] }, { name: "\u5821\u5792", faction: "shu", hp: 4, gender: "male", skills: [{ name: "bshaowei", mapping: "host:wudian_baolei_switch" }, { name: "zhencha", mapping: "host:wudian_baolei_switch" }, { name: "zhencha_draw", mapping: "host:wudian_zhencha_draw" }] }, { name: "\u5362\u897F\u5965", faction: "wu", hp: 3, gender: "male", skills: [{ name: "yuedong", mapping: "host:wudian_yuedong" }, { name: "kuoyin", mapping: "host:wudian_kuoyin" }, { name: "huhuan", mapping: "host:wudian_huhuan" }] }, { name: "\u7F8E", faction: "wei", hp: 3, gender: "female", skills: [{ name: "jidong", mapping: "host:wudian_jidong" }, { name: "baoxue", mapping: "host:wudian_baoxue" }, { name: "bingqiang", mapping: null, reason: "\u5EA7\u4F4D\u76F8\u90BB/\u8DDD\u79BB\u4FEE\u9970\u591A\u76EE\u6807\uFF08bingqiang2-5 \u5B50\u6280\u65CF+changeTarget \u76F8\u90BB\u6269\u5C55\uFF09\u2014\u20142026-10-08 \u6682\u7F13" }] }, { name: "\u8DEF\u9738", faction: "shu", hp: 4, gender: "male", skills: [{ name: "liangou", mapping: "host:wudian_liangou" }, { name: "xiyang", mapping: "host:wudian_xiyang" }] }, { name: "\u6E90\u6C0F", faction: "qun", hp: 3, gender: "male", skills: [{ name: "feiren", mapping: "host:wudian_feiren" }, { name: "lianpo", mapping: null, reason: "\u6E90\u5B9A\u4E49\u4E0D\u5728 ow/skill.js\uFF08\u5171\u4EAB\u6280\u80FD\u6587\u4EF6\uFF0Cfeiying \u540C\u6B3E\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }, { name: "zhanlong", mapping: null, reason: "limited+createCard \u51ED\u7A7A\u9020\u724C 3 \u6740+zhanlong2 \u4E34\u65F6\u6280\u94FE\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u6CD5\u8001\u4E4B\u9E70", faction: "shu", hp: 3, gender: "female", skills: [{ name: "feidan", mapping: "host:wudian_feidan" }, { name: "huoyu", mapping: null, reason: "limited+\u865A\u62DF useCard\uFF08chiyuxi \u51ED\u7A7A\u7528\u724C\xD72\uFF09\u2014\u2014\u865A\u62DF\u7528\u724C\u539F\u8BED\u540C youzhan \u7F3A\u53E3\uFF0C2026-10-09 \u6682\u7F13" }, { name: "feiying", mapping: null, reason: "\u6E90\u5B9A\u4E49\u4E0D\u5728 ow/skill.js\uFF08\u5171\u4EAB\u6280\u80FD\u6587\u4EF6\uFF09\u2014\u20142026-10-08 \u6682\u7F13" }] }, { name: "\u5B89\u5A1C", faction: "wei", hp: 3, gender: "female", skills: [{ name: "zhiyuan", mapping: "host:wudian_zhiyuan" }, { name: "mianzhen", mapping: null, reason: "\u76EE\u6807\u4FA7 chooseToRespond \u54CD\u5E94\u8BE2\u95EE+mianzhen2 \u6C38\u4E45\u5B50\u6280\u2014\u20142026-10-09 \u6682\u7F13" }, { name: "aqianghua", mapping: null, reason: "\u7ED9\u724C+\u62A4\u7532+aqianghua2 \u5B50\u6280\u94FE\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u9EA6\u514B\u96F7", faction: "shu", hp: 4, gender: "male", skills: [{ name: "shenqiang", mapping: "host:wudian_shenqiang" }, { name: "shanguang", mapping: null, reason: "shanguang2 \u5168\u5361\u724C\u7981\u7528\u5B50\u6280\uFF08mod cardEnabled/cardUsable/cardRespondable/cardSavable \u5168 false\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }, { name: "tiandan", mapping: null, reason: "createCard \u51ED\u7A7A\u9020\u6740\xD7N+\u4E3B\u52A8\u8DF3\u8FC7\u51FA\u724C/\u5F03\u724C\u9636\u6BB5\uFF08skip\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u72C2\u9F20", faction: "shu", hp: 3, gender: "male", skills: [{ name: "shihuo", mapping: "host:wudian_shihuo" }, { name: "liudan", mapping: null, reason: "\u6740\u4F7F\u7528\u65F6 50% \u6982\u7387\u968F\u673A\u8FFD\u52A0\u5408\u6CD5\u76EE\u6807\uFF08RNG \u76EE\u6807\u8FFD\u52A0+lib.filter.targetEnabled \u590D\u7528\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }, { name: "shoujia", mapping: null, reason: "\u7ED9\u724C+shoujia2 \u6302\u724C\u5B50\u6280\uFF08storage \u53CC\u5411\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u83B1\u56E0\u54C8\u7279", faction: "qun", hp: 4, gender: "male", skills: [{ name: "mengji", mapping: "host:wudian_mengji" }, { name: "zhongdun", mapping: null, reason: "init2 \u5F00\u5C40 +8 \u62A4\u7532\uFF08\u65E0 GameStart \u4E8B\u4EF6\u79CD\u7C7B=\u5F00\u5C40\u4E8B\u4EF6\u57FA\u5EFA\u7F3A\u53E3\uFF09+\u4E3B\u52A8\u8F6C\u7532\u6280\u2014\u20142026-10-09 \u6682\u7F13\uFF1Bmengji \u7684 storage.zhongdun \u7F6E\u4F4D\u4EE5\u6301\u6280\u7B49\u4EF7" }] }, { name: "\u534A\u85CF", faction: "qun", hp: 4, gender: "male", skills: [{ name: "yinbo", mapping: "host:wudian_yinbo" }, { name: "bfengshi", mapping: null, reason: "shaBegin Math.random \u6982\u7387\u76F4\u51FB+countUsed \u653B\u8DDD+\u7EC4\u6280 bfengshi2/3/4\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u5965\u4E3D\u838E", faction: "wu", hp: 4, gender: "female", skills: [{ name: "qianggu", mapping: "host:wudian_qianggu" }, { name: "qianggu2", mapping: "host:wudian_qianggu_guard" }, { name: "woliu", mapping: null, reason: "\u56DE\u5408\u7ED3\u675F\u9009 1-2 \u76EE\u6807\u5171\u6302 woliu2 \u7EC4\u8DDD\u79BB/\u6740\u9650\u5B9A+dieBegin/phaseBegin \u6E05\u9664\u7EC4\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u6E29\u65AF\u987F", faction: "shu", hp: 4, gender: "male", skills: [{ name: "feitiao", mapping: "host:wudian_feitiao" }, { name: "dianji", mapping: null, reason: "viewAs \u60CA\u96F7\u6740\uFF08jingleishan \u724C\u79CD\u4E0D\u5728\u539F\u751F\u76EE\u5F55\uFF09+\u8DDD\u79BB 2 \u9650\u5B9A\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u67E5\u8389\u5A05", faction: "wei", hp: 4, gender: "female", skills: [{ name: "pingzhang", mapping: "host:wudian_pingzhang" }, { name: "owliyong", mapping: "host:wudian_owliyong" }] }, { name: "\u9ED1\u5F71", faction: "wei", hp: 3, gender: "female", skills: [{ name: "yinshen", mapping: "host:wudian_yinshen" }, { name: "qinru", mapping: null, reason: "useCardToBegin \u5224\u5B9A+\u8DE8\u56DE\u5408\u76EE\u6807\u5B58\u50A8\u5217\u8868\uFF08qinru_turn \u5206\u6863\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }, { name: "maichong", mapping: null, reason: "qinru \u5B58\u50A8\u94FE\uFF08\u5BF9\u5DF2\u5165\u4FB5\u8005\u968F\u673A\u5F03\uFF09\u2014\u2014\u968F qinru \u6682\u7F13" }] }, { name: "\u6B7B\u795E", faction: "shu", hp: 4, gender: "male", skills: [{ name: "xiandan", mapping: "host:wudian_xiandan" }, { name: "yihun", mapping: null, reason: "\u865A\u62DF useCard\uFF08useCard({name:sha}) \u6307\u5B9A\u76EE\u6807\u56DE\u5408\uFF09\u2014\u2014\u865A\u62DF\u7528\u724C\u539F\u8BED\u540C youzhan/huoyu \u7F3A\u53E3\uFF0C2026-10-09 \u6682\u7F13" }, { name: "shouge", mapping: null, reason: "createCard \u6CBB\u7597\u6CE2\uFF08\u724C\u79CD\u8DE8\u5305\u4F9D\u8D56\u7089\u77F3\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "D.Va", faction: "shu", hp: 2, gender: "female", skills: [{ name: "jijia", mapping: "host:wudian_jijia" }, { name: "tuijin", mapping: null, reason: "\u673A\u7532\u8840>0 \u65F6\u8DDD\u79BB>1 \u76EE\u6807\u63A8\u79FB\uFF08tuijin2 \u8DDD\u79BB\u9650\u5B9A\u5B50\u6280\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }, { name: "zihui", mapping: null, reason: "\u81EA\u7206 AOE\uFF08chooseToDiscard n \u5F20\u6216 2 \u706B\u4F24\uFF0C\u673A\u7532\u8840=\u5F03\u724C\u6570\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }, { name: "chongzhuang", mapping: null, reason: "\u91CD\u88C5\u7D2F\u8BA1\uFF08\u673A\u7532\u8840\u5F52\u96F6\u540E\u4F24\u5BB3\u8BA1\u6570 4 \u590D\u6D3B\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u730E\u7A7A", faction: "shu", hp: 3, gender: "female", skills: [{ name: "shanxian", mapping: null, reason: "\u533A\u57DF\u79BB\u573A\uFF08he \u533A\u724C\u5165 storage \u79BB\u573A\u533A\uFF0Cshanhui \u56DE\u5F52\u590D\u6D3B\u542B\u4F4D\u7F6E s \u7684 createCard \u91CD\u9020\uFF09\u2014\u2014\u724C\u5B88\u6052\u7EAA\u5F8B\u7ED3\u6784\u6027\u4F9D\u8D56\uFF0C2026-10-09 \u6682\u7F13" }, { name: "shanhui", mapping: null, reason: "\u968F shanxian \u79BB\u573A\u5B58\u50A8\u94FE\u2014\u20142026-10-09 \u6682\u7F13" }] }], _m = { source: lm, upstream_path: um, upstream_commit: cm, pack: dm, cards: pm, host_skills: hm, deferred: fm, skills: mm, generals: gm }, vm = { package_format_version: 1, namespace: "upstream-noname", package_id: "shaai-xianjian", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [217, 238, 24, 99, 75, 40, 128, 69, 19, 80, 34, 62, 215, 207, 28, 63, 239, 132, 117, 217, 39, 40, 203, 249, 147, 20, 32, 1, 10, 43, 15, 178], asset_hash: [139, 23, 211, 24, 76, 140, 174, 144, 67, 188, 243, 216, 101, 242, 48, 219, 197, 44, 103, 185, 96, 125, 164, 50, 213, 22, 31, 210, 83, 3, 86, 181], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, ym = [], km = [], wm = [], bm = [], xm = [], $m = [], jm = [], Sm = [], Em = { "zh-CN": { diaobingqianjiang: "\u8C03\u5175\u9063\u5C06", sanjian: "\u4E09\u5C16", wangmeizhike: "\u671B\u6885\u6B62\u6E34", youdishenru: "\u8BF1\u654C\u6DF1\u5165" } }, Am = { schema_version: 1, package_id: "shaai-xianjian", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "shaai-xianjian:cards", source_hash: [125, 162, 65, 35, 137, 186, 77, 70, 223, 174, 61, 71, 106, 251, 20, 203, 164, 54, 152, 191, 36, 6, 206, 58, 190, 205, 23, 208, 222, 190, 253, 65], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, Cm = { "shaai-xianjian:cards": "assets/cards.json" }, Rm = "gen-1", Ao = { manifest: vm, modules: ym, skill_index: km, character_index: wm, card_index: bm, trigger_index: xm, modifier_index: $m, active_index: jm, view_as_index: Sm, translations: Em, asset_manifest: Am, asset_paths: Cm, compiler_version: Rm }, zm = "qita/sanguosha/noname/extension/\u6740\u6D77\u62FE\u9057/character/xianjian", Im = "extension/\u6740\u6D77\u62FE\u9057/character/xianjian/", Tm = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", Pm = { name: "shaai-xianjian-upstream-noname", version: 1, description: "G1 \u7EB5\u5207\uFF1A\u4E0A\u6E38 xianjian \u661F\u7487\uFF08\u7075\u96EA\uFF09", card_prefix: "xj_" }, Om = [], Nm = [], qm = [], Mm = [{ name: "lingxue", general: "pal_xingxuan", mapping: "host:xj_lingxue" }, { name: "huimeng", general: "pal_zhaoliner", mapping: "host:xj_huimeng" }, { name: "tianshe", general: "pal_zhaoliner", mapping: "host:xj_tianshe" }, { name: "tianshe2", general: "pal_zhaoliner", mapping: "host:xj_tianshe2" }, { name: "fanling", general: "pal_xia", mapping: "host:xj_fanling" }, { name: "guiyuan", general: "pal_linyueru", mapping: "host:xj_guiyuan" }, { name: "tannang", general: "pal_hanlingsha", mapping: "host:xj_tannang" }, { name: "feixu", general: "pal_wangpengxu", mapping: "host:xj_feixu" }, { name: "feng", general: "pal_leiyuange", mapping: "host:xj_feng" }, { name: "ya", general: "pal_leiyuange", mapping: "host:xj_ya" }, { name: "song", general: "pal_leiyuange", mapping: "host:xj_song" }, { name: "xjyufeng", general: "pal_lixiaoyao", mapping: "host:xj_xjyufeng" }, { name: "xuanyan", general: "pal_xuanxiao", mapping: "host:xj_xuanyan" }, { name: "xuanyan2", general: "pal_xuanxiao", mapping: "host:xj_xuanyan2" }, { name: "xuanyan3", general: "pal_xuanxiao", mapping: "host:xj_xuanyan3" }, { name: "ningbin", general: "pal_xuanxiao", mapping: "host:xj_ningbin" }, { name: "diewu", general: "pal_longkui", mapping: "host:xj_diewu" }, { name: "lingyu", general: "pal_longkui", mapping: "host:xj_lingyu" }, { name: "lueying", general: "pal_muchanglan", mapping: "host:xj_lueying" }, { name: "anwugu", general: "pal_anu", mapping: "host:xj_anwugu" }, { name: "anwugu2", general: "pal_anu", mapping: "host:xj_anwugu2" }, { name: "txianqu", general: "pal_tangyurou", mapping: "host:xj_txianqu" }], Dm = [{ name: "\u661F\u7487", faction: "qun", hp: 3, gender: "male", skills: [{ name: "lingxue", mapping: "host:xj_lingxue" }, { name: "feizhua", mapping: null, reason: "useCard \u6740\u5355\u76EE\u6807+\u7EAF\u8DDD 1 \u5185\u73A9\u5BB6\u8BE2\u95EE\u6362\u4F4D\u2014\u20142026-10-09 \u6682\u7F13" }, { name: "leiyu", mapping: null, reason: "phaseEnd \u96F7\u57DF\u5B58\u50A8\u5217\u8868+\u8DE8\u56DE\u5408\u6807\u8BB0\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u6C88\u6B3A\u971C", faction: "wei", hp: 3, gender: "female", skills: [{ name: "binxin", mapping: "host:xj_binxin" }, { name: "qixia", mapping: null, reason: "\u82B1\u8272\u96C6\u96C6\u9F50 4 \u8272 insertPhase \u989D\u5916\u56DE\u5408\uFF08\u5F15\u64CE\u7F3A\u53E3 ADR0646\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }, { name: "jianzhen", mapping: null, reason: "shaAfter \u9010\u4EBA chooseToUse \u5BF9\u540C\u4E00\u76EE\u6807\u7528\u6740\u8BE2\u95EE\u94FE\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u8D75\u7075\u513F", faction: "wei", hp: 3, gender: "female", skills: [{ name: "huimeng", mapping: "host:xj_huimeng" }, { name: "tianshe", mapping: "host:xj_tianshe" }, { name: "tianshe2", mapping: "host:xj_tianshe2" }] }, { name: "\u7455", faction: "wu", hp: 3, gender: "female", skills: [{ name: "xjzongyu", mapping: null, reason: "\u865A\u62DF useCard \u98DE\u9556\uFF08useCard({name:feibiao}) \u968F\u673A 2 \u654C\uFF09\u2014\u2014\u865A\u62DF\u7528\u724C\u539F\u8BED\u7F3A\u53E3\uFF0C2026-10-09 \u6682\u7F13" }, { name: "fanling", mapping: "host:xj_fanling" }] }, { name: "\u6E29\u6167", faction: "wei", hp: 4, gender: "male", skills: [{ name: "longxiang", mapping: "host:xj_longxiang" }, { name: "huxi", mapping: null, reason: "chooseToUse viewAs sha\uFF08\u65E0\u5B9E\u4F53\u724C\u865A\u62DF\u7528\u724C\uFF09+respondSha\u2014\u2014\u865A\u62DF\u7528\u724C\u539F\u8BED\u7F3A\u53E3\uFF0C2026-10-09 \u6682\u7F13" }] }, { name: "\u6797\u6708\u5982", faction: "wei", hp: 3, gender: "female", skills: [{ name: "guiyuan", mapping: "host:xj_guiyuan" }, { name: "qijian", mapping: null, reason: "phaseDiscardEnd \u5F03\u724C\u2192\u865A\u62DF useCard \u6740\uFF08\u65E0\u5B9E\u4F53\u724C\uFF09\u2014\u2014\u865A\u62DF\u7528\u724C\u539F\u8BED\u7F3A\u53E3\uFF0C2026-10-09 \u6682\u7F13" }] }, { name: "\u97E9\u83F1\u7EB1", faction: "qun", hp: 3, gender: "female", skills: [{ name: "tannang", mapping: "host:xj_tannang" }, { name: "tuoqiao", mapping: null, reason: "\u9ED1\u724C\u5F53\u77F3\u4F1A\u7C89\uFF08\u77F3\u4F1A\u7C89\u4E3A\u53E4\u5251\u5361\u7247\u2014\u2014\u8DE8\u5305\u724C\u79CD\u4F9D\u8D56\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u738B\u84EC\u7D6E", faction: "wei", hp: 3, gender: "female", skills: [{ name: "feixu", mapping: "host:xj_feixu" }, { name: "duxinshu", mapping: null, reason: "\u8BFB\u5FC3\uFF1A\u770B\u76EE\u6807\u624B\u724C chooseCardButton \u9009\u4E00\u5F20\u7528\u81EA\u5DF1\u7684\u624B\u724C\u66FF\u6362\uFF08\u6309\u94AE\u9009\u62E9+\u53CC\u5411\u6362\u724C\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u96F7\u5143\u6208", faction: "qun", hp: 4, gender: "male", skills: [{ name: "feng", mapping: "host:xj_feng" }, { name: "ya", mapping: "host:xj_ya" }, { name: "song", mapping: "host:xj_song" }] }, { name: "\u674E\u900D\u9065", faction: "qun", hp: 4, gender: "male", skills: [{ name: "xjyufeng", mapping: "host:xj_xjyufeng" }, { name: "tianjian", mapping: null, reason: "viewAs \u4E07\u7BAD\u9F50\u53D1\uFF08\u6740\u724C\u5F53 AOE\uFF09+tianjian_discard \u4F24\u5BB3\u5F52\u5C5E\u2014\u2014AOE \u865A\u62DF\u7528\u724C+\u5F52\u5C5E\u7F3A\u53E3\uFF0C2026-10-09 \u6682\u7F13" }] }, { name: "\u7384\u9704", faction: "qun", hp: 4, gender: "male", skills: [{ name: "xuanyan", mapping: "host:xj_xuanyan" }, { name: "xuanyan2", mapping: "host:xj_xuanyan2" }, { name: "xuanyan3", mapping: "host:xj_xuanyan3" }, { name: "ningbin", mapping: "host:xj_ningbin" }, { name: "xfenxin", mapping: null, reason: "changeHp \u4E8B\u4EF6\uFF08\u6389\u8840/\u56DE\u590D\u589E\u91CF\u62BD\u724C\uFF09\u2014\u2014\u539F\u751F\u65E0 changeHp \u4E8B\u4EF6\u79CD\uFF08jijia \u7528 commit_damage \u62E6\u622A\u66FF\u4EE3\u53E3\u5F84\uFF09\uFF0C2026-10-09 \u6682\u7F13" }] }, { name: "\u9F99\u8475", faction: "wu", hp: 3, gender: "female", skills: [{ name: "diewu", mapping: "host:xj_diewu" }, { name: "lingyu", mapping: "host:xj_lingyu" }, { name: "fenxing", mapping: null, reason: "\u53CC\u5F62\u6001\u53D8\u8EAB\uFF08Math.random \u5207\u6362 avatar+\u884D\u751F\u6280\u7EC4\uFF09\u2014\u2014\u7ED3\u6784\u6027\u4F9D\u8D56\uFF0C2026-10-09 \u6682\u7F13" }] }, { name: "\u6155\u957F\u5170", faction: "wei", hp: 3, gender: "female", skills: [{ name: "lueying", mapping: "host:xj_lueying" }, { name: "feixia", mapping: null, reason: "\u865A\u62DF useCard \u6740\uFF08\u968F\u673A\u654C\uFF09\u2014\u2014\u865A\u62DF\u7528\u724C\u539F\u8BED\u7F3A\u53E3\uFF0C2026-10-09 \u6682\u7F13" }] }, { name: "\u963F\u5974", faction: "wu", hp: 3, gender: "female", skills: [{ name: "anwugu", mapping: "host:xj_anwugu" }, { name: "anwugu2", mapping: "host:xj_anwugu2" }, { name: "lingdi", mapping: null, reason: "\u8DDD\u79BB\u9012\u589E\u76EE\u6807+\u82B1\u8272\u53BB\u91CD\u5B58\u50A8\u2014\u2014\u591A\u94A9\u5B58\u50A8\u7CFB\uFF0C2026-10-09 \u6682\u7F13" }] }, { name: "\u5510\u96E8\u67D4", faction: "wu", hp: 3, gender: "female", skills: [{ name: "txianqu", mapping: "host:xj_txianqu" }, { name: "qiongguang", mapping: null, reason: "\u7EAF\u8868\u73B0\u6280\uFF08line/expose/delay\uFF0C\u65E0\u673A\u5236\u672C\u4F53\uFF09\u2014\u2014\u8868\u73B0\u5C42 noop\uFF0C2026-10-09 \u767B\u8BB0\u4E0D\u8F6C" }] }, { name: "\u59DC\u57CE", faction: "qun", hp: 4, gender: "male", skills: [{ name: "yanzhan", mapping: null, reason: "\u7EA2\u724C\u8F6C\u706B\u6740 usable:1+\u76EE\u6807 yanzhan3 \u54CD\u5E94\u82B1\u8272 mod+yanzhan2\u21924 \u6740\u9650\u6B21 mod\u2014\u2014viewAs/mod \u7CFB\uFF0C2026-10-09 \u6682\u7F13" }, { name: "fenshi", mapping: "host:xj_fenshi" }, { name: "longhuo", mapping: "host:xj_longhuo", reason_ref: "\u884D\u751F\u6280\u5E38\u9A7B\u5B9E\u4F8B+PlayerState.jiangcheng_longhuo \u95E8\u63A7\u5EFA\u6A21\uFF08\u4E0A\u6E38 addSkill \u52A8\u6001\u83B7\u5F97\uFF09\uFF0C\u89C1\u89C4\u5219\u57FA\u51C6\u767B\u8BB0" }] }, { name: "\u666F\u5929", faction: "wu", hp: 3, gender: "male", skills: [{ name: "sajin", mapping: "host:xj_sajin" }, { name: "jtjubao", mapping: "host:xj_jtjubao" }] }, { name: "\u9F99\u5E7D", faction: "wei", hp: 4, gender: "male", skills: [{ name: "yuexing", mapping: "host:xj_yuexing" }, { name: "minsha", mapping: "host:xj_minsha" }] }, { name: "\u660E\u7EE3", faction: "shu", hp: 3, gender: "female", skills: [{ name: "linghuo", mapping: "host:xj_linghuo" }, { name: "guijin", mapping: "host:xj_guijin" }, { name: "chengxin", mapping: "host:xj_chengxin" }] }, { name: "\u4E91\u5929\u6CB3", faction: "wu", hp: 4, gender: "male", skills: [{ name: "longxi", mapping: null, reason: "chooseToRespond/chooseToUse \u54CD\u5E94\u8BE2\u95EE\u524D\u7F6E\u94A9\u7F3A\u53E3\uFF08\u8BE2\u95EE\u94FE\u6539\u5199\u65CF\uFF09\uFF0C2026-10-09 \u6682\u7F13" }, { name: "zhuyue", mapping: "host:xj_zhuyue" }, { name: "guanri", mapping: "host:xj_guanri" }] }, { name: "\u7D2B\u8431", faction: "shu", hp: 3, gender: "female", skills: [{ name: "shuiyun", mapping: "host:xj_shuiyun" }, { name: "shuiyun5", mapping: "host:xj_shuiyun5", reason_ref: "shuiyun \u5B50\u6280\u5E38\u9A7B\u5B9E\u4F8B\u5316\uFF08\u4E0A\u6E38 subSkill \u4E0D\u5217\u5C06\u8868\uFF09\uFF0C\u6FD2\u6B7B\u6551\u63F4\u901A\u9053\u9996\u4F8B" }, { name: "wangyou", mapping: "host:xj_wangyou" }, { name: "wangyou2", mapping: "host:xj_wangyou2", reason_ref: "wangyou \u5B50\u6280\u5E38\u9A7B\u5B9E\u4F8B\u5316\uFF08silent \u6807\u8BB0\uFF09" }, { name: "changnian", mapping: "host:xj_changnian" }] }, { name: "\u5C0F\u86EE", faction: "shu", hp: 3, gender: "female", skills: [{ name: "anwugu", mapping: "host:xj_anwugu", reason_ref: "\u4E0E\u963F\u5974\u540C\u6280\u2014\u2014mapping \u590D\u7528" }, { name: "lingquan", mapping: "host:xj_lingquan" }, { name: "shenwu", mapping: "host:xj_shenwu" }] }, { name: "\u96EA\u89C1", faction: "shu", hp: 3, gender: "female", skills: [{ name: "xshuangren", mapping: "host:xj_xshuangren" }, { name: "shenmu", mapping: "host:xj_shenmu" }, { name: "duci", mapping: "host:xj_duci" }, { name: "xshuangren2", mapping: "host:xj_xshuangren2", reason_ref: "xshuangren \u5B50\u6280\u5E38\u9A7B\u5B9E\u4F8B\u5316\uFF08dieBegin \u5F03\u7F6E\uFF09" }] }, { name: "\u67F3\u68A6\u7483", faction: "shu", hp: 3, gender: "female", skills: [{ name: "tianxian", mapping: "host:xj_tianxian" }, { name: "runxin", mapping: "host:xj_runxin" }, { name: "xjzhimeng", mapping: "host:xj_xjzhimeng" }, { name: "xjzhimeng2", mapping: "host:xj_xjzhimeng2", reason_ref: "xjzhimeng \u5B50\u6280\u5E38\u9A7B\u5B9E\u4F8B\u5316\uFF08\u76EE\u6807 runtime \u6388\u6280 def \u6E90\uFF09" }, { name: "changnian2", mapping: "host:xj_changnian2", reason_ref: "changnian \u5B50\u6280\u5E38\u9A7B\u5B9E\u4F8B\u5316\uFF08runtime \u6388\u6280 def \u6E90\uFF09" }] }, { name: "\u5357\u5BAB\u714C", faction: "wei", hp: 3, gender: "male", skills: [{ name: "zhaoyao", mapping: "host:xj_zhaoyao" }, { name: "sheling", mapping: "host:xj_sheling" }, { name: "zhangmu", mapping: "host:xj_zhangmu" }] }, { name: "\u738B\u5C0F\u864E", faction: "qun", hp: 4, gender: "male", skills: [{ name: "husha_count", mapping: "host:xj_husha_count", reason_ref: "husha \u5B50\u6280\u5E38\u9A7B\u5B9E\u4F8B\u5316\uFF08source damageEnd \u96C6\u6807\u8BB0\uFF09" }, { name: "husha", mapping: "host:xj_husha", reason_ref: "\u5143\u7D20\u6BC1\u706D\uFF083 \u679A\u9009\u9879\uFF09\u724C\u79CD\u4E0D\u5B58\u5728\u76EE\u5F55\u767B\u8BB0\u4E0D\u8F6C" }] }, { name: "\u6708\u7426", faction: "wei", hp: 3, gender: "female", skills: [{ name: "tianwu", mapping: null, reason: "\u7EAF\u8868\u73B0\u6280\uFF08getDebuff+addTempSkill tianwu2 \u7A7A\u58F3\uFF09\u2014\u2014\u8868\u73B0\u5C42 noop\uFF0C2026-10-09 \u767B\u8BB0\u4E0D\u8F6C" }, { name: "liguang", mapping: "host:xj_liguang" }, { name: "shiying", mapping: "host:xj_shiying" }] }, { name: "\u590F\u4FAF\u747E\u8F69", faction: "wei", hp: 3, gender: "male", skills: [{ name: "xuanmo", mapping: "host:xj_xuanmo" }, { name: "danqing", mapping: null, reason: "\u56DB\u82B1\u8272\u6536\u96C6\u2192getBuff(false) \u7EAF\u8868\u73B0\u76EE\u6807\u9009\u62E9\uFF0C\u65E0\u673A\u5236\u672C\u4F53\u2014\u2014\u8868\u73B0\u5C42\u767B\u8BB0\u4E0D\u8F6C\uFF0C2026-10-09" }] }, { name: "\u82CF\u5A9A", faction: "shu", hp: 3, gender: "female", skills: [{ name: "sheying", mapping: "host:xj_sheying" }, { name: "dujiang", mapping: null, reason: "\u83B7\u5F97\u3010\u6BD2\u3011\u724C\u2014\u2014\u6BD2\u724C\u79CD\u4E0D\u5728\u76EE\u5F55\uFF08createCard \u5B9E\u4F53\u53D1\u73B0\u4EA6\u4E0D\u53EF\u7528\uFF09\uFF0C2026-10-09 \u6682\u7F13" }, { name: "huahu", mapping: "host:xj_huahu" }] }, { name: "\u4ED9\u537F", faction: "qun", hp: 4, gender: "male", skills: [{ name: "xtanxi", mapping: "host:xj_xtanxi" }, { name: "xiaoyue", mapping: null, reason: "roundStart \u4E8B\u4EF6\u7F3A\u53E3+\u81EA\u52A8\u865A\u62DF\u7528\u724C\uFF08useCard \u6740+\u65E0\u8DDD\u79BB\u9650\u5236 mod\uFF09\u590D\u5408\u7F3A\u53E3\uFF0C2026-10-09 \u6682\u7F13" }] }, { name: "\u7F57\u62DB\u5AE3", faction: "shu", hp: 4, gender: "female", skills: [{ name: "fenglue", mapping: "host:xj_fenglue" }, { name: "tanhua", mapping: null, reason: "Recover \u524D\u7F6E\u7A97\u53E3\u7F3A\u53E3\uFF08\u56DE\u590D\u6539\u4E3A\u6478\u724C\u9700\u56DE\u590D\u53D6\u6D88\u8BED\u4E49\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u59DC\u4E91\u51E1", faction: "wei", hp: 4, gender: "male", skills: [{ name: "xunying", mapping: "host:xj_xunying" }, { name: "liefeng", mapping: "host:xj_liefeng" }, { name: "tianjian", mapping: "host:xj_tianjian", reason_ref: "liefeng runtime \u6388\u6280\u76EE\u6807 def \u6E90\uFF08\u76EE\u5F55\u7ECF mapping \u6CE8\u518C\uFF09" }] }, { name: "\u6155\u5BB9\u7D2B\u82F1", faction: "wei", hp: 4, gender: "male", skills: [{ name: "poyun", mapping: "host:xj_poyun" }, { name: "qianfang", mapping: "host:xj_qianfang" }, { name: "xuanning", mapping: null, reason: "\u8BA1\u6570\u6807\u8BB0\u83B7\u5F97\u8DEF\u5F84\u4E0A\u6E38\u7F3A\u5931\uFF08storage.xuanning \u65E0\u6280\u80FD\u5199\u5165\uFF09\u2014\u2014\u539F\u751F\u4EC5\u652F\u6301\u8C03\u8BD5/\u5BBF\u4E3B\u6CE8\u5165\uFF0C2026-10-09 \u767B\u8BB0\u5DEE\u5F02" }] }, { name: "\u8D8A\u4ECA\u671D", faction: "wei", hp: 4, gender: "male", skills: [{ name: "ywuhun", mapping: null, reason: "\u56DE\u5408\u5F00\u59CB\u524D\u989D\u5916\u56DE\u5408+\u56DE\u5408\u540E\u590D\u539F\u573A\u4E0A\u53CA\u724C\u5806\u5168\u90E8\u724C\u2014\u2014\u53CC\u5F15\u64CE\u7F3A\u53E3\uFF08\u989D\u5916\u56DE\u5408=ADR0646 \u767B\u8BB0\uFF09\uFF0C2026-10-10 \u6682\u7F13" }, { name: "xjyingfeng", mapping: "host:xj_xjyingfeng" }] }, { name: "\u5C45\u5341\u65B9", faction: "shu", hp: 3, gender: "male", skills: [{ name: "xiepan", mapping: "host:xj_xiepan" }, { name: "yujia", mapping: null, reason: "\u673A\u5173\u724C\u79CD\u4E0D\u5728\u76EE\u5F55\uFF08\u89C2\u770B/\u8BA1\u6570\u4F9D\u8D56\u673A\u5173\u724C\u5B9E\u4F53\u6C60\uFF09\u2014\u2014dujiang \u540C\u65CF\u7F3A\u53E3\uFF0C2026-10-10 \u6682\u7F13" }, { name: "yanshi", mapping: null, reason: "\u673A\u5173\u724C\u79CD\u4E0D\u5728\u76EE\u5F55\uFF08\u7D2F\u8BA1\u4F7F\u7528\u673A\u5173\u724C\u8BA1\u6570+\u968F\u673A\u88C5\u5907\u586B\u6EE1\u53EF\u6302\u5B9E\u4F53\u53D1\u73B0\uFF0C\u524D\u7F6E=\u673A\u5173\u724C\u79CD\u57FA\u5EFA\uFF09\uFF0C2026-10-10 \u6682\u7F13" }] }, { name: "\u957F\u537F", faction: "wei", hp: 4, gender: "male", skills: [{ name: "luanjian", mapping: "host:xj_luanjian" }, { name: "luanjian2", mapping: "host:xj_luanjian2", reason_ref: "luanjian \u589E\u4F24 subskill\uFF08DamageBefore forced 50%\uFF0C\u8F7D\u4F53\u724C\u951A\u5B9A\uFF09" }, { name: "ctianfu", mapping: "host:xj_ctianfu" }, { name: "ctianfu2", mapping: "host:xj_ctianfu2", reason_ref: "\u5929\u7B26\xB7\u53CD\u566C subskill\uFF08Damage \u4F24\u5BB3\u540E\u7A97\uFF0C\u6302\u9644\u6807\u8BB0+\u5B9E\u4F8B\u6210\u5BF9\uFF09" }, { name: "ctianfu3", mapping: "host:xj_ctianfu3", reason_ref: "\u5929\u7B26\xB7\u6E05\u6302 subskill\uFF08Death \u7A97\u6E05\u7406\uFF09" }] }], Lm = { source: zm, upstream_path: Im, upstream_commit: Tm, pack: Pm, cards: Om, host_skills: Nm, deferred: qm, skills: Mm, generals: Dm }, Um = { package_format_version: 1, namespace: "upstream-noname", package_id: "shahai-gujian", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [174, 19, 1, 143, 26, 75, 62, 188, 159, 67, 126, 42, 118, 12, 107, 113, 203, 163, 207, 147, 194, 208, 143, 29, 74, 238, 216, 95, 37, 52, 158, 6], asset_hash: [64, 167, 122, 159, 149, 48, 209, 126, 144, 16, 221, 246, 166, 188, 56, 151, 231, 155, 253, 19, 203, 98, 144, 68, 6, 23, 156, 243, 43, 177, 22, 35], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, Fm = [], Vm = [], Hm = [], Wm = [], Bm = [], Km = [], Gm = [], Jm = [], Ym = { "zh-CN": { bingpotong: "\u51B0\u9B44\u901A", chunbing: "\u6625\u997C", dinvxuanshuang: "\u5E1D\u5973\u7384\u971C", feibiao: "\u98DE\u9556", gjyuheng: "\u7389\u8861", gudonggeng: "\u5495\u549A\u7FB9", gujian: "\u53E4\u5251\u5947\u8C2D", heilonglinpian: "\u9ED1\u9F99\u9CDE\u7247", jinlianzhu: "\u91D1\u83B2\u70DB", liufengsan: "\u516D\u98CE\u6563", liutouge: "\u786B\u5934\u9601", liyutang: "\u9CA4\u9C7C\u6C64", longxugou: "\u9F99\u987B\u94A9", luyugeng: "\u5364\u7389\u7FB9", mapodoufu: "\u9EBB\u5A46\u8C46\u8150", mianlijinzhen: "\u9762\u91CC\u91D1\u9488", mizhilianou: "\u871C\u94FE\u85D5", molicha: "\u9EBB\u8336", mutoumianju: "\u6728\u5934\u9762\u5177", qiankunbiao: "\u4E7E\u5764\u9556", qingtuan: "\u9752\u56E2", shatang: "\u7802\u7CD6", shenhuofeiya: "\u795E\u706B\u98DE\u9E26", shihuifen: "\u77F3\u4F1A\u7C89", shujinsan: "\u8212\u7B4B\u6563", tanhuadong: "\u644A\u9EC4\u51BB", xiajiao: "\u83F1\u89D2", yougeng: "\u6709\u66F4", yuanbaorou: "\u5143\u5B9D\u8089", yunvyuanshen: "\u7389\u5973\u5143\u795E", ziyangdan: "\u7D2B\u9633\u4E39" } }, Qm = { schema_version: 1, package_id: "shahai-gujian", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "shahai-gujian:cards", source_hash: [59, 14, 97, 152, 140, 199, 211, 59, 2, 113, 112, 213, 241, 164, 249, 98, 134, 209, 174, 76, 85, 135, 152, 123, 139, 5, 42, 72, 174, 40, 183, 10], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, Xm = { "shahai-gujian:cards": "assets/cards.json" }, Zm = "gen-1", Co = { manifest: Um, modules: Fm, skill_index: Vm, character_index: Hm, card_index: Wm, trigger_index: Bm, modifier_index: Km, active_index: Gm, view_as_index: Jm, translations: Ym, asset_manifest: Qm, asset_paths: Xm, compiler_version: Zm }, eg = "noname", ng = "apps/core/extension/\u6740\u6D77\u62FE\u9057/card/gujian/", tg = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", sg = { name: "shahai-gujian-upstream-noname", version: 1, description: "G1 \u6740\u6D77\u62FE\u9057\xB7\u53E4\u5251\u5207\u7247", card_prefix: "gj_" }, ag = [{ name: "shujinsan", kind: "shujinsan", suit: "diamond", rank: 6 }, { name: "shujinsan", kind: "shujinsan", suit: "spade", rank: 2 }, { name: "mianlijinzhen", kind: "mianlijinzhen", suit: "spade", rank: 1 }, { name: "feibiao", kind: "feibiao", suit: "club", rank: 8 }, { name: "feibiao", kind: "feibiao", suit: "diamond", rank: 9 }, { name: "jinlianzhu", kind: "jinlianzhu", suit: "heart", rank: 9 }, { name: "jinlianzhu", kind: "jinlianzhu", suit: "spade", rank: 7 }, { name: "tanhuadong", kind: "tanhuadong", suit: "spade", rank: 2 }, { name: "yougeng", kind: "yougeng", suit: "heart", rank: 12 }, { name: "qingtuan", kind: "qingtuan", suit: "spade", rank: 3 }, { name: "yuanbaorou", kind: "yuanbaorou", suit: "spade", rank: 8 }, { name: "gudonggeng", kind: "gudonggeng", suit: "heart", rank: 8 }, { name: "mapodoufu", kind: "mapodoufu", suit: "heart", rank: 4 }, { name: "chunbing", kind: "chunbing", suit: "club", rank: 3 }, { name: "liyutang", kind: "liyutang", suit: "heart", rank: 1 }, { name: "molicha", kind: "molicha", suit: "club", rank: 1 }, { name: "xiajiao", kind: "xiajiao", suit: "diamond", rank: 6 }, { name: "mizhilianou", kind: "mizhilianou", suit: "diamond", rank: 4 }, { name: "luyugeng", kind: "luyugeng", suit: "club", rank: 11 }, { name: "bingpotong", kind: "bingpotong", suit: "spade", rank: 3 }, { name: "bingpotong", kind: "bingpotong", suit: "club", rank: 12 }, { name: "qiankunbiao", kind: "qiankunbiao", suit: "spade", rank: 9 }, { name: "qiankunbiao", kind: "qiankunbiao", suit: "club", rank: 13 }, { name: "shenhuofeiya", kind: "shenhuofeiya", suit: "diamond", rank: 9 }, { name: "longxugou", kind: "longxugou", suit: "spade", rank: 7 }, { name: "ziyangdan", kind: "ziyangdan", suit: "diamond", rank: 6 }, { name: "ziyangdan", kind: "ziyangdan", suit: "heart", rank: 1 }, { name: "yunvyuanshen", kind: "yunvyuanshen", suit: "heart", rank: 13 }, { name: "shihuifen", kind: "shihuifen", suit: "club", rank: 5 }, { name: "shihuifen", kind: "shihuifen", suit: "club", rank: 1 }, { name: "shihuifen", kind: "shihuifen", suit: "spade", rank: 13 }, { name: "liufengsan", kind: "liufengsan", suit: "club", rank: 6 }, { name: "liufengsan", kind: "liufengsan", suit: "club", rank: 3 }, { name: "shatang", kind: "shatang", suit: "heart", rank: 13 }, { name: "heilonglinpian", kind: "heilonglinpian", suit: "spade", rank: 2 }, { name: "liutouge", kind: "liutouge", suit: "heart", rank: 6 }, { name: "liutouge", kind: "liutouge", suit: "club", rank: 6 }, { name: "mutoumianju", kind: "mutoumianju", suit: "club", rank: 4 }, { name: "dinvxuanshuang", kind: "dinvxuanshuang", suit: "heart", rank: 9 }, { name: "gjyuheng", kind: "gjyuheng", suit: "spade", rank: 7 }], ig = [], og = { source: eg, upstream_path: ng, upstream_commit: tg, pack: sg, cards: ag, deferred: ig }, rg = { package_format_version: 1, namespace: "upstream-noname", package_id: "shahai-gwent", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [11, 210, 86, 154, 147, 71, 161, 144, 244, 225, 236, 145, 131, 209, 115, 183, 233, 66, 140, 230, 24, 60, 18, 155, 254, 3, 178, 198, 44, 142, 171, 205], asset_hash: [81, 129, 91, 229, 139, 77, 173, 158, 200, 141, 186, 65, 162, 83, 196, 174, 250, 135, 67, 199, 255, 82, 208, 132, 191, 127, 138, 83, 57, 29, 198, 231], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, lg = [], ug = [], cg = [], dg = [], pg = [], hg = [], fg = [], mg = [], gg = { "zh-CN": { aozuzhilei: "\u5965\u7956\u4E4B\u96F7", baoxueyaoshui: "\u66B4\u96EA\u836F\u6C34", birinongwu: "\u78A7\u65E5\u6D53\u96FE", ciguhanshuang: "\u523A\u9AA8\u5BD2\u971C", gwent: "gwent", kunenfayin: "\u6606\u6069\u6CD5\u5370", poxiao: "\u7834\u6653", qinpendayu: "\u503E\u76C6\u5927\u96E8", shanbengshu: "\u5C71\u5D29\u672F", wenyi: "\u761F\u75AB", yanziyaoshui: "\u71D5\u5B50\u836F\u6C34", zhihuanjun: "\u7F6E\u6362\u519B", zumoshoukao: "\u7956\u58A8\u624B\u94E0" } }, _g = { schema_version: 1, package_id: "shahai-gwent", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "shahai-gwent:cards", source_hash: [166, 48, 247, 15, 211, 138, 196, 50, 148, 242, 250, 36, 174, 15, 79, 156, 9, 156, 56, 156, 45, 66, 129, 66, 55, 226, 111, 253, 188, 124, 214, 218], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, vg = { "shahai-gwent:cards": "assets/cards.json" }, yg = "gen-1", Ro = { manifest: rg, modules: lg, skill_index: ug, character_index: cg, card_index: dg, trigger_index: pg, modifier_index: hg, active_index: fg, view_as_index: mg, translations: gg, asset_manifest: _g, asset_paths: vg, compiler_version: yg }, kg = "noname", wg = "apps/core/extension/\u6740\u6D77\u62FE\u9057/card/gwent/", bg = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", xg = { name: "shahai-gwent-upstream-noname", version: 1, description: "G1 \u6740\u6D77\u62FE\u9057\xB7gwent \u5207\u7247", card_prefix: "gw_" }, $g = [{ name: "yanziyaoshui", kind: "yanziyaoshui", suit: "heart", rank: 8 }, { name: "yanziyaoshui", kind: "yanziyaoshui", suit: "club", rank: 7 }, { name: "aozuzhilei", kind: "aozuzhilei", suit: "spade", rank: 9 }, { name: "aozuzhilei", kind: "aozuzhilei", suit: "club", rank: 3 }, { name: "wenyi", kind: "wenyi", suit: "club", rank: 5 }, { name: "zhihuanjun", kind: "zhihuanjun", suit: "club", rank: 1 }, { name: "zhihuanjun", kind: "zhihuanjun", suit: "spade", rank: 3 }, { name: "baoxueyaoshui", kind: "baoxueyaoshui", suit: "heart", rank: 4 }, { name: "baoxueyaoshui", kind: "baoxueyaoshui", suit: "spade", rank: 8 }, { name: "qinpendayu", kind: "qinpendayu", suit: "diamond", rank: 5 }, { name: "qinpendayu", kind: "qinpendayu", suit: "club", rank: 7 }, { name: "birinongwu", kind: "birinongwu", suit: "spade", rank: 9 }, { name: "birinongwu", kind: "birinongwu", suit: "heart", rank: 13 }, { name: "ciguhanshuang", kind: "ciguhanshuang", suit: "diamond", rank: 11 }, { name: "ciguhanshuang", kind: "ciguhanshuang", suit: "club", rank: 7 }, { name: "kunenfayin", kind: "kunenfayin", suit: "spade", rank: 11 }, { name: "shanbengshu", kind: "shanbengshu", suit: "spade", rank: 8 }, { name: "poxiao", kind: "poxiao", suit: "heart", rank: 7 }, { name: "poxiao", kind: "poxiao", suit: "diamond", rank: 4 }, { name: "zumoshoukao", kind: "zumoshoukao", suit: "spade", rank: 2 }, { name: "zumoshoukao", kind: "zumoshoukao", suit: "club", rank: 5 }], jg = [], Sg = { source: kg, upstream_path: wg, upstream_commit: bg, pack: xg, cards: $g, deferred: jg }, Eg = { package_format_version: 1, namespace: "upstream-noname", package_id: "shahai-hearth", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [222, 175, 195, 28, 126, 229, 205, 192, 134, 104, 195, 255, 215, 43, 199, 51, 28, 232, 140, 32, 174, 71, 203, 169, 230, 49, 163, 84, 168, 127, 59, 118], asset_hash: [19, 173, 66, 158, 166, 22, 115, 40, 86, 41, 2, 15, 142, 182, 166, 113, 120, 248, 54, 209, 90, 199, 119, 75, 143, 71, 43, 245, 55, 14, 41, 93], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, Ag = [], Cg = [], Rg = [], zg = [], Ig = [], Tg = [], Pg = [], Og = [], Ng = { "zh-CN": { chuansongmen: "\u4F20\u9001\u95E8", dunpaigedang: "\u76FE\u724C\u683C\u6321", hearth: "\u7089\u77F3\u4F20\u8BF4", jihuocard: "\u6025\u8017\u5361", linghunzhihuo: "\u7075\u9B42\u4E4B\u706B", shandianjian: "\u95EA\u7535\u5251", shenenshu: "\u795E\u6069\u672F", tanshezhiren: "\u5766\u86C7\u81F3\u4EBA", xingjiegoutong: "\u661F\u754C\u6C9F\u901A", yuansuhuimie: "\u5143\u7D20\u6BC1\u706D", zhaomingdan: "\u7167\u660E\u5F39", zhiliaobo: "\u6CBB\u7597\u6CE2" } }, qg = { schema_version: 1, package_id: "shahai-hearth", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "shahai-hearth:cards", source_hash: [224, 12, 104, 161, 208, 213, 217, 129, 88, 249, 189, 119, 106, 114, 77, 244, 33, 155, 19, 99, 87, 24, 224, 112, 101, 175, 244, 191, 178, 203, 181, 243], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, Mg = { "shahai-hearth:cards": "assets/cards.json" }, Dg = "gen-1", zo = { manifest: Eg, modules: Ag, skill_index: Cg, character_index: Rg, card_index: zg, trigger_index: Ig, modifier_index: Tg, active_index: Pg, view_as_index: Og, translations: Ng, asset_manifest: qg, asset_paths: Mg, compiler_version: Dg }, Lg = "noname", Ug = "apps/core/extension/\u6740\u6D77\u62FE\u9057/card/hearth/", Fg = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", Vg = { name: "shahai-hearth-upstream-noname", version: 1, description: "G1 \u7089\u77F3\u4F20\u8BF4\u5207\u7247", card_prefix: "ht_" }, Hg = [{ name: "linghunzhihuo", kind: "linghunzhihuo", suit: "diamond", rank: 2 }, { name: "zhaomingdan", kind: "zhaomingdan", suit: "diamond", rank: 5 }, { name: "zhaomingdan", kind: "zhaomingdan", suit: "heart", rank: 10 }, { name: "jihuocard", kind: "jihuocard", suit: "diamond", rank: 2 }, { name: "shandianjian", kind: "leisha", suit: "club", rank: 3 }, { name: "shandianjian", kind: "leisha", suit: "spade", rank: 1 }, { name: "shenenshu", kind: "jihuocard", suit: "heart", rank: 2 }, { name: "shenenshu", kind: "jihuocard", suit: "diamond", rank: 12 }, { name: "dunpaigedang", kind: "dunpaigedang", suit: "club", rank: 3 }, { name: "zhiliaobo", kind: "zhiliaobo", suit: "club", rank: 7 }, { name: "zhiliaobo", kind: "zhiliaobo", suit: "spade", rank: 1 }, { name: "yuansuhuimie", kind: "yuansuhuimie", suit: "spade", rank: 13 }, { name: "xingjiegoutong", kind: "xingjiegoutong", suit: "spade", rank: 13 }, { name: "chuansongmen", kind: "chuansongmen", suit: "diamond", rank: 2 }, { name: "chuansongmen", kind: "chuansongmen", suit: "heart", rank: 2 }, { name: "tanshezhiren", kind: "tanshezhiren", suit: "diamond", rank: 2 }, { name: "shijieshu", kind: "shijieshu", suit: "spade", rank: 7 }], Wg = [], Bg = { source: Lg, upstream_path: Ug, upstream_commit: Fg, pack: Vg, cards: Hg, deferred: Wg }, Kg = { package_format_version: 1, namespace: "upstream-noname", package_id: "shahai-mtg", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [100, 23, 190, 115, 240, 231, 83, 4, 12, 207, 180, 163, 203, 162, 240, 254, 137, 75, 251, 157, 145, 232, 234, 173, 222, 68, 39, 243, 228, 135, 129, 98], asset_hash: [138, 246, 212, 134, 176, 90, 183, 36, 117, 169, 120, 78, 6, 33, 28, 70, 108, 98, 68, 51, 36, 115, 35, 162, 65, 214, 7, 150, 89, 14, 168, 195], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, Gg = [], Jg = [], Yg = [], Qg = [], Xg = [], Zg = [], e_ = [], n_ = [], t_ = { schema_version: 1, package_id: "shahai-mtg", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "shahai-mtg:cards", source_hash: [90, 88, 252, 38, 173, 223, 90, 144, 111, 137, 77, 35, 246, 175, 221, 252, 104, 231, 28, 131, 135, 65, 225, 157, 151, 243, 9, 35, 13, 249, 156, 220], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, s_ = { "shahai-mtg:cards": "assets/cards.json" }, a_ = "test", Io = { manifest: Kg, modules: Gg, skill_index: Jg, character_index: Yg, card_index: Qg, trigger_index: Xg, modifier_index: Zg, active_index: e_, view_as_index: n_, asset_manifest: t_, asset_paths: s_, compiler_version: a_ }, i_ = "qita/sanguosha/noname/extension/\u6740\u6D77\u62FE\u9057/card/mtg", o_ = "extension/\u6740\u6D77\u62FE\u9057/card/mtg/", r_ = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", l_ = { name: "shahai-mtg-upstream-noname", version: 1, description: "G1 \u8F6C\u6362\uFF1A\u4E0A\u6E38 mtg \u9996\u5207\u7247\uFF08\u9F99\u9885\u5C71\u5CF0\uFF09", card_prefix: "mtg_" }, u_ = [{ name: "mtg_longlushanfeng", kind: "longlushanfeng", suit: "club", rank: 10 }, { name: "mtg_haidao", kind: "haidao", suit: "diamond", rank: 5 }, { name: "mtg_shuimomuxue", kind: "shuimomuxue", suit: "spade", rank: 11 }, { name: "mtg_feixu", kind: "feixu", suit: "club", rank: 8 }, { name: "mtg_yixialan", kind: "yixialan", suit: "club", rank: 12 }, { name: "mtg_shamolvzhou", kind: "shamolvzhou", suit: "heart", rank: 6 }, { name: "mtg_linzhongjianta", kind: "linzhongjianta", suit: "heart", rank: 6 }, { name: "mtg_bingheyaosai", kind: "bingheyaosai", suit: "diamond", rank: 5 }, { name: "mtg_youlin", kind: "youlin", suit: "club", rank: 10 }, { name: "mtg_lindixiliu", kind: "lindixiliu", suit: "spade", rank: 11 }], c_ = [], d_ = [{ name: "mtg_duzhao", reason: "\u4F9D\u8D56 xianxia \u5305\u3010\u6BD2\u3011\u5361\uFF08\u8DE8\u5305\u724C\u79CD\u4F9D\u8D56\u540C tuoqiao \u5148\u4F8B\uFF09\u2014\u20142026-10-11 \u6682\u7F13\uFF0C\u968F\u6BD2\u5361\u6E90\u8F6C\u6362\u89E3\u9501" }, { name: "mtg_cangbaohaiwan", reason: "\u4F9D\u8D56\u6D3E\u751F\u5305\u9009\u62E9 UI\uFF08derivation \u5305\u5217\u8868\u4EA4\u4E92\u9009\u724C\uFF09\u2014\u2014\u539F\u751F\u65E0\u8BE5\u673A\u5236\uFF0C2026-10-11 \u6682\u7F13" }], p_ = { source: i_, upstream_path: o_, upstream_commit: r_, pack: l_, cards: u_, host_skills: c_, deferred: d_ }, h_ = { package_format_version: 1, namespace: "upstream-noname", package_id: "shahai-swd", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [49, 204, 234, 152, 242, 141, 74, 177, 190, 148, 159, 91, 147, 112, 4, 119, 125, 19, 159, 58, 106, 197, 84, 110, 16, 191, 141, 30, 209, 141, 195, 127], asset_hash: [199, 119, 92, 146, 166, 89, 110, 115, 216, 130, 121, 121, 62, 47, 65, 102, 152, 166, 25, 69, 169, 82, 236, 165, 122, 112, 127, 20, 153, 39, 8, 196], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, f_ = [], m_ = [], g_ = [], __ = [], v_ = [], y_ = [], k_ = [], w_ = [], b_ = { schema_version: 1, package_id: "shahai-swd", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "shahai-swd:cards", source_hash: [70, 23, 137, 164, 203, 28, 38, 181, 129, 120, 81, 208, 138, 2, 175, 224, 228, 196, 230, 118, 108, 225, 193, 254, 181, 155, 126, 170, 60, 93, 71, 82], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, x_ = { "shahai-swd:cards": "assets/cards.json" }, $_ = "test", To = { manifest: h_, modules: f_, skill_index: m_, character_index: g_, card_index: __, trigger_index: v_, modifier_index: y_, active_index: k_, view_as_index: w_, asset_manifest: b_, asset_paths: x_, compiler_version: $_ }, j_ = "noname", S_ = "apps/core/extension/\u6740\u6D77\u62FE\u9057/card/swd/", E_ = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", A_ = { name: "shahai-swd-upstream-noname", version: 1, description: "G1 \u8F6C\u6362\uFF1A\u4E0A\u6E38 swd \u9996\u5207\u7247\uFF08\u62A4\u7B26\uFF09", card_prefix: "swd_", init_hook: "swd_jiqi_held" }, C_ = [{ name: "swd_hufu", kind: "hufu", suit: "spade", rank: 1 }, { name: "swd_hufu2", kind: "hufu", suit: "heart", rank: 1 }, { name: "swd_liuxinghuoyu1", kind: "liuxinghuoyu", suit: "diamond", rank: 3 }, { name: "swd_liuxinghuoyu2", kind: "liuxinghuoyu", suit: "heart", rank: 6 }, { name: "swd_liuxinghuoyu3", kind: "liuxinghuoyu", suit: "heart", rank: 9 }, { name: "swd_caoyao1", kind: "caoyao", suit: "club", rank: 3 }, { name: "swd_caoyao2", kind: "caoyao", suit: "club", rank: 4 }, { name: "swd_caoyao3", kind: "caoyao", suit: "club", rank: 5 }, { name: "swd_chilongya", kind: "chilongya", suit: "diamond", rank: 3 }, { name: "swd_tianxianjiu1", kind: "tianxianjiu", suit: "heart", rank: 9 }, { name: "swd_tianxianjiu2", kind: "tianxianjiu", suit: "diamond", rank: 2 }, { name: "swd_xixueguizhihuan", kind: "xixueguizhihuan", suit: "spade", rank: 5 }, { name: "swd_guiyoujie1", kind: "guiyoujie", suit: "spade", rank: 3 }, { name: "swd_guiyoujie2", kind: "guiyoujie", suit: "club", rank: 9 }, { name: "swd_shentoumianju", kind: "shentoumianju", suit: "club", rank: 6 }, { name: "swd_xingjunyan", kind: "xingjunyan", suit: "spade", rank: 7 }, { name: "swd_qinglonglingzhu", kind: "qinglonglingzhu", suit: "spade", rank: 2 }, { name: "swd_zhufangshenshi", kind: "zhufangshenshi", suit: "spade", rank: 4 }, { name: "swd_fengxueren", kind: "fengxueren", suit: "club", rank: 1 }, { name: "swd_yufulu", kind: "yufulu", suit: "spade", rank: 6 }, { name: "swd_yangpijuan", kind: "yangpijuan", suit: "spade", rank: 9 }, { name: "swd_daihuofenglun", kind: "daihuofenglun", suit: "heart", rank: 1 }, { name: "swd_xiayuncailing", kind: "xiayuncailing", suit: "diamond", rank: 2 }, { name: "swd_langeguaiyi", kind: "langeguaiyi", suit: "diamond", rank: 1 }, { name: "swd_zhiluxiaohu", kind: "jiguan", suit: "heart", rank: 3 }, { name: "swd_mujiaren", kind: "jiguan", suit: "club", rank: 7 }, { name: "swd_gouhunluo1", kind: "jiguan", suit: "spade", rank: 9 }, { name: "swd_gouhunluo2", kind: "jiguan", suit: "club", rank: 7 }, { name: "swd_jiguantong1", kind: "jiguan", suit: "diamond", rank: 4 }, { name: "swd_jiguantong2", kind: "jiguan", suit: "club", rank: 7 }, { name: "swd_jiguanfeng1", kind: "jiguan", suit: "club", rank: 3 }, { name: "swd_jiguanfeng2", kind: "jiguan", suit: "spade", rank: 4 }, { name: "swd_jiguanyuan1", kind: "jiguan", suit: "club", rank: 3 }, { name: "swd_jiguanyuan2", kind: "jiguan", suit: "diamond", rank: 2 }, { name: "swd_jiguanyaoshu1", kind: "jiguan", suit: "club", rank: 3 }, { name: "swd_jiguanyaoshu2", kind: "jiguan", suit: "spade", rank: 3 }, { name: "swd_xuanfengzhiren1", kind: "hslingjian", suit: "diamond", rank: 4 }, { name: "swd_zhongxinghujia1", kind: "hslingjian", suit: "heart", rank: 4 }, { name: "swd_xingtigaizao1", kind: "hslingjian", suit: "spade", rank: 6 }, { name: "swd_shijianhuisu1", kind: "hslingjian", suit: "diamond", rank: 6 }, { name: "swd_jinjilengdong1", kind: "hslingjian", suit: "spade", rank: 8 }, { name: "swd_shengxiuhaojiao1", kind: "hslingjian", suit: "heart", rank: 8 }, { name: "swd_yinmilichang1", kind: "hslingjian", suit: "club", rank: 10 }, { name: "swd_jiguanshu1", kind: "jiguan", suit: "club", rank: 2 }, { name: "swd_jiguanshu2", kind: "jiguan", suit: "diamond", rank: 2 }, { name: "swd_zhufangshenshi2", kind: "zhufangshenshi", suit: "spade", rank: 8 }, { name: "swd_zhiluxiaohu2", kind: "jiguan", suit: "diamond", rank: 4 }, { name: "swd_mujiaren2", kind: "jiguan", suit: "heart", rank: 6 }, { name: "swd_mujiaren3", kind: "jiguan", suit: "diamond", rank: 11 }, { name: "swd_yangpijuan2", kind: "yangpijuan", suit: "club", rank: 7 }, { name: "swd_xuejibingbao", kind: "xuejibingbao", suit: "club", rank: 1 }, { name: "swd_xuejibingbao2", kind: "xuejibingbao", suit: "spade", rank: 1 }, { name: "swd_fengyinzhidan", kind: "fengyinzhidan", suit: "spade", rank: 1 }, { name: "swd_shenmiguo", kind: "shenmiguo", suit: "spade", rank: 2 }, { name: "swd_shenmiguo2", kind: "shenmiguo", suit: "heart", rank: 1 }, { name: "swd_lianyaohu", kind: "lianyaohu", suit: "club", rank: 13, equip_skills: ["lianyaohu_shouna", "lianyaohu_lianhua"] }, { name: "swd_qiankundai", kind: "qiankundai", suit: "club", rank: 1 }, { name: "swd_shennongding", kind: "shennongding", suit: "club", rank: 13, equip_skills: ["shennongding"] }, { name: "swd_nvwashi", kind: "nvwashi", suit: "heart", rank: 13, equip_skills: ["nvwashi"] }, { name: "swd_kunlunjingc", kind: "kunlunjingc", suit: "spade", rank: 13, equip_skills: ["kunlunjingc"] }, { name: "swd_donghuangzhong", kind: "donghuangzhong", suit: "diamond", rank: 13, equip_skills: ["donghuangzhong"] }, { name: "swd_kongdongyin", kind: "kongdongyin", suit: "heart", rank: 13, equip_skills: ["kongdongyin"] }, { name: "swd_jingleishan", kind: "jingleishan", suit: "club", rank: 7 }, { name: "swd_chiyuxi", kind: "chiyuxi", suit: "diamond", rank: 7 }, { name: "swd_guisheqi", kind: "guisheqi", suit: "spade", rank: 9 }, { name: "swd_guisheqi2", kind: "guisheqi", suit: "club", rank: 7 }, { name: "swd_xianluhui", kind: "xianluhui", suit: "spade", rank: 13 }, { name: "swd_guilingzhitao", kind: "guilingzhitao", suit: "spade", rank: 7, equip_skills: ["nigong_accumulate", "nigong_strike"] }, { name: "swd_sifeizhenmian", kind: "sifeizhenmian", suit: "spade", rank: 4, equip_skills: ["yiluan"] }, { name: "swd_qinglongzhigui", kind: "qinglongzhigui", suit: "heart", rank: 6 }, { name: "swd_zhuquezhizhang", kind: "zhuquezhizhang", suit: "diamond", rank: 6 }, { name: "swd_baishouzhihu", kind: "baishouzhihu", suit: "spade", rank: 6 }, { name: "swd_xuanwuzhihuang", kind: "xuanwuzhihuang", suit: "club", rank: 6 }, { name: "swd_huanglinzhicong", kind: "huanglinzhicong", suit: "heart", rank: 5 }, { name: "swd_shuchui", kind: "shuchui", suit: "club", rank: 6, equip_skills: ["shuchui"] }, { name: "swd_sadengjinhuan", kind: "sadengjinhuan", suit: "club", rank: 13 }, { name: "swd_haotianta", kind: "haotianta", suit: "diamond", rank: 13 }, { name: "swd_yuruyi", kind: "yuruyi", suit: "heart", rank: 9 }, { name: "swd_yuchanqian", kind: "yuchanqian", suit: "spade", rank: 1, deferred_reason: "\u6362\u5F62\u6001\u968F\u673A mutation+\u7145\u9020\u88C5\u5907\u9762\u6302\u6279\u246C\uFF08autoViewAs \u6740\u672C\u6279\u5DF2\u8F6C\u6B63\uFF09" }, { name: "swd_yuchankun", kind: "yuchankun", suit: "club", rank: 2, deferred_reason: "\u6362\u5F62\u6001\u968F\u673A mutation+\u7145\u9020\u88C5\u5907\u9762\u6302\u6279\u246C\uFF08autoViewAs \u8349\u836F\u672C\u6279\u5DF2\u8F6C\u6B63\uFF09" }, { name: "swd_yuchanzhen", kind: "yuchanzhen", suit: "diamond", rank: 3, deferred_reason: "\u6362\u5F62\u6001\u968F\u673A mutation+\u7145\u9020\u88C5\u5907\u9762\u6302\u6279\u246C\uFF08autoViewAs \u9152\u672C\u6279\u5DF2\u8F6C\u6B63\uFF09" }, { name: "swd_yuchanxun", kind: "yuchanxun", suit: "heart", rank: 4, deferred_reason: "\u6362\u5F62\u6001\u968F\u673A mutation+\u7145\u9020\u88C5\u5907\u9762\u6302\u6279\u246C\uFF08autoViewAs \u6843\u672C\u6279\u5DF2\u8F6C\u6B63\uFF09" }, { name: "swd_yuchankan", kind: "yuchankan", suit: "spade", rank: 5, deferred_reason: "\u6362\u5F62\u6001\u968F\u673A mutation+\u7145\u9020\u88C5\u5907\u9762\u6302\u6279\u246C" }, { name: "swd_yuchanli", kind: "yuchanli", suit: "club", rank: 6, deferred_reason: "\u6362\u5F62\u6001\u968F\u673A mutation+\u7145\u9020\u88C5\u5907\u9762\u6302\u6279\u246C\uFF08autoViewAs \u5929\u4ED9\u9152\u672C\u6279\u5DF2\u8F6C\u6B63\uFF09" }, { name: "swd_yuchangen", kind: "yuchangen", suit: "diamond", rank: 7, deferred_reason: "\u6362\u5F62\u6001\u968F\u673A mutation+\u7145\u9020\u88C5\u5907\u9762\u6302\u6279\u246C\uFF08autoViewAs \u5C01\u5370\u4E4B\u4E39\u672C\u6279\u5DF2\u8F6C\u6B63\uFF09" }, { name: "swd_yuchandui", kind: "yuchandui", suit: "heart", rank: 8, deferred_reason: "\u6362\u5F62\u6001\u968F\u673A mutation+\u7145\u9020\u88C5\u5907\u9762\u6302\u6279\u246C\uFF08autoViewAs \u96EA\u9701\u51B0\u96F9\u672C\u6279\u5DF2\u8F6C\u6B63\uFF09" }, { name: "swd_shencaojie", kind: "shencaojie", suit: "club", rank: 4 }, { name: "swd_shencaojie2", kind: "shencaojie", suit: "diamond", rank: 4 }, { name: "swd_shencaojie3", kind: "shencaojie", suit: "spade", rank: 4 }, { name: "xuanyuanjian", kind: "xuanyuanjian", suit: "spade", rank: 1 }], R_ = [], z_ = [], I_ = JSON.parse('[{"name":"swd\u97E9\u7F57","faction":"qun","hp":5,"gender":"male","skills":[{"name":"tiebi","mapping":"host:tiebi"}]},{"name":"swd\u4E60\u5AE3","faction":"shu","hp":3,"gender":"female","skills":[{"name":"jiefen","mapping":"host:jiefen"},{"name":"datong","mapping":"host:datong"}]},{"name":"swd\u8F69\u8F95\u5251\u4F8D","faction":"qun","hp":3,"gender":"male","skills":[{"name":"chengjian","mapping":"host:chengjian"},{"name":"huanling","mapping":"host:huanling"}]},{"name":"swd\u7A0B\u54AC\u91D1","faction":"qun","hp":4,"gender":"male","skills":[{"name":"jiuchi","mapping":"builtin:\u9152\u6C60"},{"name":"jufu","mapping":"host:jufu"}]},{"name":"swd\u79E6\u53D4\u5B9D","faction":"qun","hp":3,"gender":"male","skills":[{"name":"huajing","mapping":"host:huajing"},{"name":"pingxu","mapping":"host:pingxu"}]},{"name":"swd\u5B50\u8FBE\u4E66","faction":"wei","hp":3,"gender":"male","skills":[{"name":"shoulie","mapping":"host:shoulie"},{"name":"hudun","mapping":"host:hudun"}]},{"name":"swd\u5F20\u9AD8","faction":"shu","hp":4,"gender":"male","skills":[{"name":"yicong","mapping":"builtin:\u4E49\u4ECE"},{"name":"poxing","mapping":"host:poxing"}]},{"name":"swd\u62D3\u8DCB\u7389\u513F","faction":"shu","hp":4,"gender":"female","skills":[{"name":"swdliuhong","mapping":"host:swdliuhong"},{"name":"poyue","mapping":"host:poyue"},{"name":"niepan","mapping":"host:niepan"}]},{"name":"swd\u6C34\u955C","faction":"qun","hp":4,"gender":"female","skills":[{"name":"mojian","mapping":"host:mojian"},{"name":"duanyue","mapping":"host:duanyue"},{"name":"tuzhen","mapping":"host:tuzhen"}]},{"name":"swd\u5C48\u5A34","faction":"qun","hp":3,"gender":"female","skills":[{"name":"mojian","mapping":"host:mojian"},{"name":"huanxia","mapping":"host:huanxia"}]},{"name":"swd\u58F6\u4E2D\u4ED9","faction":"wu","hp":3,"gender":"male","skills":[{"name":"daofa","mapping":"host:daofa"},{"name":"xielv","mapping":"host:xielv"},{"name":"hujing","mapping":"host:hujing"}]},{"name":"swd\u5170\u8335","faction":"wei","hp":3,"gender":"female","skills":[{"name":"xingdian","mapping":"host:xingdian"},{"name":"yulin","mapping":"host:yulin"},{"name":"luomei","mapping":"host:luomei"}]},{"name":"swd\u829D\u8335","faction":"wei","hp":3,"gender":"female","skills":[{"name":"xuehuang","mapping":"host:xuehuang"},{"name":"ningshuang","mapping":"host:ningshuang"},{"name":"zhuyu","mapping":"host:zhuyu"}]},{"name":"swd\u9EC4\u96F7","faction":"qun","hp":3,"gender":"male","skills":[{"name":"jilve","mapping":"host:jilve"},{"name":"gongshen","mapping":"host:gongshen"},{"name":"gaizao","mapping":"host:gaizao"}]},{"name":"swd_septem","faction":"qun","hp":4,"gender":"male","skills":[{"name":"jiying","mapping":"host:jiying"},{"name":"liaoyuan","mapping":"host:liaoyuan"},{"name":"yishan","mapping":"host:yishan"}]},{"name":"swd_huanyuanzhi","faction":"qun","hp":3,"gender":"male","skills":[{"name":"swdtianshu","mapping":"host:swdtianshu"},{"name":"lanzhi","mapping":"host:lanzhi"},{"name":"mufeng","mapping":"host:mufeng"}]},{"name":"swd_guyue","faction":"wei","hp":3,"gender":"male","skills":[{"name":"gtiandao","mapping":"host:g_tiandao"},{"name":"gxianyin","mapping":"host:gxianyin"},{"name":"wangchen","mapping":"host:wangchen"}]},{"name":"swd_shangzhang","faction":"shu","hp":4,"gender":"male","skills":[{"name":"lianwu","mapping":"host:lianwu"}]},{"name":"swd_youzhao","faction":"shu","hp":4,"gender":"male","skills":[{"name":"longdan","mapping":"builtin:\u9F99\u80C6"},{"name":"yuchen","mapping":"host:yuchen"}]},{"name":"swd_wangsiyue","faction":"wei","hp":3,"gender":"female","skills":[{"name":"duishi","mapping":"host:duishi"},{"name":"biyue","mapping":"builtin:\u95ED\u6708"}]},{"name":"swd_yuwentuo","faction":"shu","hp":4,"gender":"male","skills":[{"name":"wushuang","mapping":"builtin:\u65E0\u53CC"},{"name":"xielei","mapping":"host:xielei"},{"name":"kunlunjing","mapping":"host:kunlunjing"}]},{"name":"swd_fuyan","faction":"qun","hp":4,"gender":"male","skills":[{"name":"lianda","mapping":"host:lianda"}]},{"name":"swd_rongshuang","faction":"wu","hp":3,"gender":"female","skills":[{"name":"suiyan","mapping":"host:suiyan"},{"name":"duanxing","mapping":"host:duanxing"}]},{"name":"swd_muyun","faction":"wei","hp":4,"gender":"male","skills":[{"name":"zhuhai","mapping":"host:zhuhai"},{"name":"polang","mapping":"host:polang"},{"name":"jikong","mapping":"host:jikong"}]},{"name":"swd_chenjingchou","faction":"wu","hp":3,"gender":"male","skills":[{"name":"youyin","mapping":"host:youyin"},{"name":"yihua","mapping":"host:yihua"}]},{"name":"swd_tuwei","faction":"shu","hp":3,"gender":"female","skills":[{"name":"zhanlu","mapping":"host:zhanlu"},{"name":"susheng","mapping":"host:susheng"}]},{"name":"swd_xiarou","faction":"shu","hp":3,"gender":"male","skills":[{"name":"xianghui","mapping":"host:xianghui"},{"name":"huiqi","mapping":"host:huiqi"}]},{"name":"swd_zhuoshanzhu","faction":"wu","hp":4,"gender":"male","skills":[{"name":"suiyan","mapping":"host:suiyan"},{"name":"wanjun","mapping":"host:wanjun"}]},{"name":"swd_moye","faction":"wu","hp":3,"gender":"female","skills":[{"name":"rexue","mapping":"host:rexue"},{"name":"liuli","mapping":"builtin:\u6D41\u79BB"},{"name":"senluo","mapping":"host:senluo"}]},{"name":"swd_yeyaxi","faction":"shu","hp":3,"gender":"female","skills":[{"name":"rexue","mapping":"host:rexue"},{"name":"huopu","mapping":"host:huopu"},{"name":"shenyan","mapping":"host:shenyan"}]},{"name":"swd_xiaohuanglong","faction":"wei","hp":3,"gender":"male","skills":[{"name":"yeying","mapping":"host:yeying"},{"name":"juxi","mapping":"host:juxi"},{"name":"juxi_count","mapping":"host:juxi_count"}]},{"name":"swd_hupo","faction":"wu","hp":3,"gender":"male","skills":[{"name":"dunxing","mapping":"host:dunxing"},{"name":"guiying","mapping":"host:guiying"}]},{"name":"swd_situqiang","faction":"shu","hp":3,"gender":"female","skills":[{"name":"fengze","mapping":"host:fengze"},{"name":"lingyue","mapping":"host:lingyue"},{"name":"jinlin","mapping":"host:jinlin"}]},{"name":"swd_anka","faction":"qun","hp":3,"gender":"male","skills":[{"name":"songci","mapping":"host:songci"},{"name":"anlianying","mapping":"host:anlianying"}]},{"name":"swd_luchengxuan","faction":"wu","hp":4,"gender":"male","skills":[{"name":"ljifeng","mapping":"host:ljifeng"},{"name":"lxianglong","mapping":"host:lxianglong"}]},{"name":"swd_murongshi","faction":"shu","hp":3,"gender":"female","skills":[{"name":"duanyi","mapping":"host:duanyi"},{"name":"guxing","mapping":"host:guxing"},{"name":"guxing3","mapping":"host:guxing3"}]},{"name":"swd_yuxiaoxue","faction":"wei","hp":3,"gender":"female","skills":[{"name":"huanhun","mapping":"host:huanhun"},{"name":"daixing","mapping":"host:daixing"},{"name":"yinyue","mapping":"host:yinyue"},{"name":"daixing2","mapping":"host:daixing2"},{"name":"daixing3","mapping":"host:daixing3"}]},{"name":"swd_yuli","faction":"wu","hp":3,"gender":"female","skills":[{"name":"lingxin","mapping":"host:lingxin"},{"name":"tianxiang","mapping":"host:tianxiang"}]},{"name":"swd_kama","faction":"qun","hp":3,"gender":"female","skills":[{"name":"yueren","mapping":"host:yueren"},{"name":"shangshi","mapping":"builtin:\u4F24\u901D"}]},{"name":"swd_nicole","faction":"qun","hp":3,"gender":"female","skills":[{"name":"huanjian","mapping":"host:huanjian"},{"name":"lingwu","mapping":"host:lingwu"},{"name":"minjing","mapping":"host:minjing"}]},{"name":"swd_weida","faction":"qun","hp":3,"gender":"female","skills":[{"name":"yueren","mapping":"host:yueren"},{"name":"zhenlie","mapping":"host:zhenlie"},{"name":"duijue","mapping":null}]},{"name":"swd_chunyuheng","faction":"wei","hp":2,"gender":"male","skills":[{"name":"jueqing","mapping":"builtin:\u7EDD\u60C5"},{"name":"shengshou","mapping":"host:shengshou"},{"name":"xuying","mapping":"host:xuying"}]},{"name":"swd_ziqiao","faction":"shu","hp":3,"gender":"female","skills":[{"name":"guaili","mapping":"host:guaili"},{"name":"fuyan","mapping":"host:fuyan"}]},{"name":"swd_lilian","faction":"qun","hp":3,"gender":"female","skills":[{"name":"swd_wuxie","mapping":"host:wuxie"},{"name":"lqingcheng","mapping":"host:lqingcheng"}]},{"name":"swd_jiting","faction":"wei","hp":4,"gender":"female","skills":[{"name":"guanhu","mapping":"host:guanhu"},{"name":"lingshi","mapping":"host:lingshi"}]},{"name":"swd_hengai","faction":"shu","hp":3,"gender":"female","skills":[{"name":"hlingbo","mapping":"host:hlingbo"},{"name":"funiao","mapping":"host:funiao"},{"name":"ningxian","mapping":"host:ningxian"}]},{"name":"swd_miles","faction":"qun","hp":4,"gender":"male","skills":[{"name":"aojian","mapping":"host:aojian"},{"name":"miles_xueyi","mapping":"host:miles_xueyi"},{"name":"mohua2","mapping":"host:mohua2"}]},{"name":"swd_kangnalishi","faction":"qun","hp":1,"gender":"male","skills":[{"name":"busi","mapping":"host:busi"},{"name":"xuying","mapping":"host:xuying"},{"name":"yinguo","mapping":"host:yinguo"}]},{"name":"swd_zhaoyun","faction":"shu","hp":4,"gender":"male","skills":[{"name":"longdan","mapping":"builtin:\u9F99\u80C6"},{"name":"pozhen","mapping":"host:pozhen"},{"name":"tanlin","mapping":null}]},{"name":"swd_huyue","faction":"wu","hp":3,"gender":"female","skills":[{"name":"hyunshen","mapping":"host:hyunshen"},{"name":"fengming","mapping":"host:fengming"},{"name":"hyunshen2","mapping":"host:hyunshen2"}]},{"name":"swd_shanxiaoxiao","faction":"wu","hp":3,"gender":"female","skills":[{"name":"shehun","mapping":"host:shehun"},{"name":"xiaomoyu","mapping":"host:xiaomoyu"}]},{"name":"swd_duguningke","faction":"qun","hp":3,"gender":"female","skills":[{"name":"nlianji","mapping":"host:nlianji"},{"name":"touxi","mapping":"builtin:\u7A81\u88AD"}]},{"name":"swd_haidapang","faction":"wu","hp":3,"gender":"female","skills":[{"name":"bingjian","mapping":"host:bingjian"},{"name":"rumeng","mapping":"host:rumeng"}]},{"name":"swd_maixing","faction":"wu","hp":3,"gender":"male","skills":[{"name":"toudan","mapping":"host:toudan"},{"name":"shending","mapping":"host:shending"}]},{"name":"swd_fengtianling","faction":"shu","hp":4,"gender":"male","skills":[{"name":"guiyan","mapping":"host:guiyan"},{"name":"jiang","mapping":"host:jiang"}]},{"name":"swd_fengyu","faction":"shu","hp":4,"gender":"male","skills":[{"name":"fzhenwei","mapping":"host:fzhenwei"},{"name":"shangxi","mapping":"host:shangxi"}]},{"name":"swd_muyue","faction":"wei","hp":3,"gender":"female","skills":[{"name":"xingzhui","mapping":"host:xingzhui"},{"name":"lingxian","mapping":"host:lingxian"},{"name":"shouyin","mapping":"host:shouyin"}]},{"name":"swd_shaowei","faction":"shu","hp":3,"gender":"female","skills":[{"name":"jianji","mapping":"host:jianji"},{"name":"huangyu","mapping":"host:huangyu"}]},{"name":"swd_jialanduo","faction":"qun","hp":4,"gender":"male","skills":[{"name":"xianyin","mapping":"host:xianyin"},{"name":"mailun","mapping":"host:mailun"}]},{"name":"swd_qi","faction":"qun","hp":3,"gender":"male","skills":[{"name":"yaotong","mapping":"host:yaotong"},{"name":"heihuo","mapping":"host:heihuo"},{"name":"pojian","mapping":"host:pojian"}]},{"name":"swd_huiyan","faction":"qun","hp":4,"gender":"male","skills":[{"name":"hwendao","mapping":"host:hwendao"},{"name":"lingfeng","mapping":"host:lingfeng"},{"name":"hxunzhi","mapping":"host:hxunzhi"}]},{"name":"swd_qiner","faction":"wei","hp":3,"gender":"female","skills":[{"name":"huanyin","mapping":"host:huanyin"},{"name":"tianhuo","mapping":"host:tianhuo"},{"name":"xuanzhou","mapping":"host:xuanzhou"}]},{"name":"swd_jiliang","faction":"wu","hp":3,"gender":"male","skills":[{"name":"yunchou","mapping":"host:yunchou"},{"name":"gongxin","mapping":"host:gongxin"},{"name":"jqimou","mapping":"host:jqimou"}]},{"name":"swd_zhanglie","faction":"qun","hp":4,"gender":"male","skills":[{"name":"huajin","mapping":"host:huajin"},{"name":"poxiao","mapping":"host:poxiao"}]},{"name":"swd_xuanyuanjianxian","faction":"qun","hp":4,"gender":"male","skills":[{"name":"pozhou","mapping":"host:pozhou"},{"name":"huajian","mapping":"host:huajian"},{"name":"xuanyuan","mapping":"host:xuanyuan"},{"name":"pozhou2","mapping":"host:pozhou2"}]},{"name":"swd_sikongyu","faction":"qun","hp":4,"gender":"male","skills":[{"name":"sliufeng","mapping":"host:sliufeng"},{"name":"linyun","mapping":"host:linyun"},{"name":"hutian","mapping":"host:hutian"}]},{"name":"swd_jipeng","faction":"wu","hp":3,"gender":"male","skills":[{"name":"reyingzi","mapping":"host:reyingzi"},{"name":"guozao","mapping":"host:guozao"}]},{"name":"swd_duopeng","faction":"wu","hp":3,"gender":"male","skills":[{"name":"luanji","mapping":"host:luanji"},{"name":"reyingzi","mapping":"host:reyingzi"}]},{"name":"swd_fu","faction":"qun","hp":5,"gender":"male","skills":[{"name":"yudun","mapping":"host:yudun"}]},{"name":"swd_kendi","faction":"qun","hp":3,"gender":"male","skills":[{"name":"zhanxing","mapping":"host:zhanxing"},{"name":"kbolan","mapping":"host:kbolan"}]},{"name":"swd_jiuyou","faction":"wei","hp":3,"gender":"male","skills":[{"name":"lexue","mapping":"host:lexue"}]},{"name":"swd_linyue","faction":"wei","hp":3,"gender":"male","skills":[{"name":"zhenjiu","mapping":"host:zhenjiu"},{"name":"lmazui","mapping":"host:lmazui"}]},{"name":"swd_shuwaner","faction":"shu","hp":3,"gender":"female","skills":[{"name":"sxianjing","mapping":"host:sxianjing"},{"name":"huodan","mapping":"host:huodan"},{"name":"sxianjing_gain","mapping":"host:sxianjing_gain"},{"name":"sxianjing_damage","mapping":"host:sxianjing_damage"}]},{"name":"swd_jiangwu","faction":"shu","hp":4,"gender":"male","skills":[{"name":"yijue","mapping":"host:yijue"},{"name":"dangping","mapping":"host:dangping"}]},{"name":"swd_yuchiyanhong","faction":"shu","hp":3,"gender":"female","skills":[{"name":"huanxing","mapping":"host:huanxing"},{"name":"meihuo","mapping":"host:meihuo"}]},{"name":"swd_jiangziya","faction":"wu","hp":3,"gender":"male","skills":[{"name":"mingfu","mapping":"host:mingfu"},{"name":"tianlun","mapping":"host:tianlun"}]},{"name":"swd_cheyun","faction":"wu","hp":3,"gender":"female","skills":[{"name":"cyxianjiang","mapping":"host:cyxianjiang"},{"name":"cyqiaoxie","mapping":"host:cyqiaoxie"},{"name":"shengong","mapping":"host:shengong"}]},{"name":"swd_duanmeng","faction":"shu","hp":4,"gender":"female","skills":[{"name":"xuanying","mapping":"host:xuanying"},{"name":"lieren","mapping":"host:lieren"}]},{"name":"swd_chenfu","faction":"qun","hp":4,"gender":"male","skills":[{"name":"xuanzhou","mapping":"host:xuanzhou"},{"name":"bingfeng","mapping":"host:bingfeng"}]}]'), T_ = [{ name: "swd_zhenjiu2", mapping: "host:zhenjiu2" }, { name: "swd_lmazui2", mapping: "host:lmazui2" }, { name: "swd_huanxing_state", mapping: "host:huanxing2" }, { name: "swd_guiyan2", mapping: "host:guiyan2" }, { name: "swd_huajin2", mapping: "host:huajin2" }, { name: "swd_jinlin2", mapping: "host:jinlin2" }, { name: "tianxiang2", mapping: "host:tianxiang2" }, { name: "hxunzhi2", mapping: "host:hxunzhi2" }, { name: "bingfeng2", mapping: "host:bingfeng2" }, { name: "xuanyuanjian2", mapping: "host:xuanyuanjian2" }, { name: "xuanyuanjian3", mapping: "host:xuanyuanjian3" }, { name: "xuanyuan", mapping: "host:xuanyuan" }, { name: "jifeng", mapping: "host:jifeng" }, { name: "moyan", mapping: "host:moyan" }, { name: "miedao", mapping: "host:miedao" }, { name: "swd_xiuluo", mapping: "host:swd_xiuluo" }], P_ = { source: j_, upstream_path: S_, upstream_commit: E_, pack: A_, cards: C_, host_skills: R_, deferred: z_, generals: I_, skills: T_ }, O_ = { package_format_version: 1, namespace: "upstream-noname", package_id: "shahai-yunchou", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [87, 73, 16, 137, 17, 228, 185, 148, 29, 192, 248, 121, 46, 123, 51, 178, 99, 59, 227, 5, 238, 24, 234, 223, 153, 43, 225, 121, 118, 175, 237, 81], asset_hash: [34, 111, 194, 231, 244, 54, 173, 146, 254, 193, 36, 14, 84, 194, 87, 207, 61, 22, 100, 151, 193, 186, 122, 45, 222, 132, 90, 79, 201, 18, 92, 112], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, N_ = [], q_ = [], M_ = [], D_ = [], L_ = [], U_ = [], F_ = [], V_ = [], H_ = { "zh-CN": { fudichouxin: "\u91DC\u5E95\u62BD\u85AA", huoshan: "\u706B\u6740", shuigong: "\u6C34\u653B", yunchou: "\u8FD0\u7B79\u5E37\u5E44" } }, W_ = { schema_version: 1, package_id: "shahai-yunchou", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "shahai-yunchou:cards", source_hash: [242, 7, 161, 150, 106, 106, 85, 215, 72, 11, 50, 144, 208, 98, 108, 44, 119, 194, 137, 133, 206, 167, 18, 228, 203, 241, 192, 44, 92, 190, 91, 119], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, B_ = { "shahai-yunchou:cards": "assets/cards.json" }, K_ = "gen-1", Po = { manifest: O_, modules: N_, skill_index: q_, character_index: M_, card_index: D_, trigger_index: L_, modifier_index: U_, active_index: F_, view_as_index: V_, translations: H_, asset_manifest: W_, asset_paths: B_, compiler_version: K_ }, G_ = "noname", J_ = "apps/core/extension/\u6740\u6D77\u62FE\u9057/card/yunchou/", Y_ = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", Q_ = { name: "shahai-yunchou-upstream-noname", version: 2, description: "G1 \u6740\u6D77\u62FE\u9057\xB7\u8FD0\u7B79\u5E37\u5E44\u5207\u7247", card_prefix: "yc_" }, X_ = [{ name: "huoshan", kind: "huosha", suit: "heart", rank: 6 }, { name: "guohe", kind: "guohe", suit: "diamond", rank: 3 }, { name: "guohe", kind: "guohe", suit: "club", rank: 8 }, { name: "guohe", kind: "guohe", suit: "spade", rank: 13 }, { name: "shuigong", kind: "shuigong", suit: "club", rank: 7 }, { name: "shuigong", kind: "shuigong", suit: "club", rank: 7 }, { name: "shuigong", kind: "shuigong", suit: "spade", rank: 9 }, { name: "fudichouxin", kind: "fudichouxin", suit: "diamond", rank: 4 }, { name: "fudichouxin", kind: "fudichouxin", suit: "club", rank: 6 }, { name: "fudichouxin", kind: "fudichouxin", suit: "spade", rank: 1 }, { name: "chenhuodajie", kind: "chenhuodajie", suit: "diamond", rank: 6 }, { name: "chenhuodajie", kind: "chenhuodajie", suit: "diamond", rank: 9 }, { name: "chenhuodajie", kind: "chenhuodajie", suit: "club", rank: 3 }, { name: "suolianjia", kind: "suolianjia", suit: "club", rank: 13, link_on_equip: true }, { name: "hongshui", kind: "hongshui", suit: "club", rank: 7 }, { name: "toulianghuanzhu", kind: "toulianghuanzhu", suit: "heart", rank: 9 }, { name: "toulianghuanzhu", kind: "toulianghuanzhu", suit: "club", rank: 10 }, { name: "geanguanhuo", kind: "geanguanhuo", suit: "heart", rank: 1 }, { name: "geanguanhuo", kind: "geanguanhuo", suit: "spade", rank: 6 }, { name: "caochuanjiejian", kind: "caochuanjiejian", suit: "spade", rank: 7 }, { name: "caochuanjiejian", kind: "caochuanjiejian", suit: "spade", rank: 7 }, { name: "shezhanqunru", kind: "shezhanqunru", suit: "heart", rank: 4 }, { name: "shezhanqunru", kind: "shezhanqunru", suit: "club", rank: 8 }], Z_ = [], e1 = { source: G_, upstream_path: J_, upstream_commit: Y_, pack: Q_, cards: X_, deferred: Z_ }, n1 = "noname", t1 = "apps/core/extension/\u6740\u6D77\u62FE\u9057/character/yunchou/", s1 = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", a1 = { name: "shahai-yunchou-generals-b9", version: 1, description: "G1 \u8FD0\u7B79\u6B66\u5C06\u9762\u62799\uFF08\u4E8E\u5409\u6574\u5C06\uFF1A\u8749\u7F18 max_hp \u51CF\u9762\u8F6C\u6B63\uFF09", card_prefix: "ycn_" }, i1 = [{ id: "ycn_diy_caocao", name: "ycn\u66F9\u64CD", faction: "wei", hp: 4, gender: "male", skills: [{ name: "ycnxicai", title: "\u620F\u624D", mapping: "host:xicai" }, { name: "ycndiyjianxiong", title: "\u5978\u96C4", mapping: null }, { name: "hujia", title: "\u62A4\u9A7E", mapping: "builtin:\u62A4\u9A7E" }] }, { id: "ycn_diy_weiyan", name: "ycn\u9B4F\u5EF6", faction: "shu", hp: 4, gender: "male", skills: [{ name: "ycndiykuanggu", title: "\u72C2\u9AA8", mapping: "host:diykuanggu" }] }, { id: "ycn_diy_sp_zhouyu", name: "ycn\u5468\u745C", faction: "wu", hp: 3, gender: "male", skills: [{ name: "ycnxiongzi", title: "\u96C4\u59FF", mapping: "host:xiongzi" }, { name: "ycnyaliang", title: "\u96C5\u91CF", mapping: "host:yaliang" }] }, { id: "ycn_diy_luxun", name: "ycn\u9646\u900A", faction: "wu", hp: 3, gender: "male", skills: [{ name: "ycnzonghuo", title: "\u7EB5\u706B", mapping: "host:zonghuo" }, { name: "ycnshaoying", title: "\u70E7\u8425", mapping: "host:shaoying" }] }, { id: "ycn_diy_yuji", name: "ycn\u4E8E\u5409", faction: "qun", hp: 3, gender: "male", skills: [{ name: "ycndiyguhuo", title: "\u86CA\u60D1", mapping: "host:diyguhuo" }, { name: "ycndiychanyuan", title: "\u8749\u7F18", mapping: "host:diychanyuan" }] }, { id: "ycn_diy_zhouyu", name: "ycn\u5468\u745C", faction: "wu", hp: 3, gender: "male", skills: [{ name: "ycnjieyan", title: "\u9699\u7130", mapping: "host:jieyan" }, { name: "ycnhonglian", title: "\u7EA2\u83B2", mapping: "host:honglian" }] }, { id: "ycn_diy_zhenji", name: "ycn\u7504\u59EC", faction: "wei", hp: 3, gender: "female", skills: [{ name: "ycnjiaoxia", title: "\u72E1\u9EE0", mapping: "host:jiaoxia" }, { name: "ycnyiesheng", title: "\u91CE\u5347", mapping: "host:yiesheng" }] }, { id: "ycn_diy_caiwenji", name: "ycn\u8521\u6587\u59EC", faction: "qun", hp: 3, gender: "female", skills: [{ name: "beige", title: "\u60B2\u6B4C", mapping: "host:beige" }, { name: "ycnguihan", title: "\u5F52\u6C49", mapping: "host:guihan" }] }, { id: "ycn_diy_lukang", name: "ycn\u9646\u6297", faction: "wu", hp: 4, gender: "male", skills: [{ name: "ycnluweiyan", title: "\u5389\u7130", mapping: "host:luweiyan" }, { name: "ycnluweiyan2", title: "\u5389\u7130\xB7\u94FE\u6740", mapping: "host:luweiyan2" }, { name: "qianxun", title: "\u8C26\u900A", mapping: "builtin:\u8C26\u900A" }] }, { id: "ycn_old_majun", name: "ycn\u9A6C\u94A7", faction: "wei", hp: 3, gender: "male", skills: [{ name: "xinfu_jingxie", title: "\u7CBE\u68B0", mapping: null }, { name: "ycnxinfu_qiaosi", title: "\u5DE7\u601D", mapping: null }] }], o1 = { source: n1, upstream_path: t1, upstream_commit: s1, pack: a1, generals: i1 }, r1 = { package_format_version: 1, namespace: "upstream-noname", package_id: "shahai-zhenfa", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [83, 148, 91, 71, 176, 142, 85, 235, 241, 190, 176, 53, 230, 223, 245, 74, 197, 187, 203, 191, 104, 12, 134, 170, 95, 74, 42, 149, 171, 33, 19, 178], asset_hash: [2, 250, 107, 122, 186, 246, 73, 201, 2, 120, 23, 125, 30, 171, 190, 145, 67, 30, 150, 250, 27, 69, 102, 73, 94, 115, 5, 218, 205, 27, 76, 4], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, l1 = [], u1 = [], c1 = [], d1 = [], p1 = [], h1 = [], f1 = [], m1 = [], g1 = { schema_version: 1, package_id: "shahai-zhenfa", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "shahai-zhenfa:cards", source_hash: [212, 242, 140, 245, 41, 62, 165, 171, 122, 180, 21, 0, 119, 163, 234, 243, 149, 248, 103, 207, 9, 191, 218, 139, 73, 11, 55, 57, 198, 30, 168, 136], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, _1 = { "shahai-zhenfa:cards": "assets/cards.json" }, v1 = "test", Oo = { manifest: r1, modules: l1, skill_index: u1, character_index: c1, card_index: d1, trigger_index: p1, modifier_index: h1, active_index: f1, view_as_index: m1, asset_manifest: g1, asset_paths: _1, compiler_version: v1 }, y1 = "qita/sanguosha/noname/extension/\u6740\u6D77\u62FE\u9057/card/zhenfa", k1 = "extension/\u6740\u6D77\u62FE\u9057/card/zhenfa/", w1 = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", b1 = { name: "shahai-zhenfa-upstream-noname", version: 1, description: "G1 \u8F6C\u6362\uFF1A\u4E0A\u6E38 zhenfa \u9996\u5207\u7247\uFF08\u5730\u8F7D\u9635\uFF09", card_prefix: "zf_" }, x1 = [{ name: "zf_dizaizhen", kind: "dizaizhen", suit: "diamond", rank: 3 }, { name: "zf_dizaizhen", kind: "dizaizhen", suit: "spade", rank: 3 }, { name: "zf_dizaizhen", kind: "dizaizhen", suit: "heart", rank: 3 }, { name: "swd_tianfuzhen1", kind: "tianfuzhen", suit: "spade", rank: 1 }, { name: "swd_qixingzhen1", kind: "qixingzhen", suit: "club", rank: 9 }, { name: "swd_niaoxiangzhen1", kind: "niaoxiangzhen", suit: "heart", rank: 3 }, { name: "swd_changshezhen1", kind: "changshezhen", suit: "diamond", rank: 1 }, { name: "swd_changshezhen2", kind: "changshezhen", suit: "club", rank: 1 }], $1 = [], j1 = [{ name: "pozhenjue", reason: "\u4F9D\u8D56\u56FD\u6218\u961F\u5217/\u56F4\u653B/\u5EA7\u4F4D\u91CD\u6392\u4F53\u7CFB\uFF08inline/isFriendOf/swapSeat\uFF09\uFF0C\u539F\u751F\u56FD\u6218\u539F\u578B\u672A\u5EFA\u6A21\u2014\u2014\u7ED3\u6784\u6027\u4FDD\u7559\u9879\u540C\u65CF" }], S1 = { source: y1, upstream_path: k1, upstream_commit: w1, pack: b1, cards: x1, host_skills: $1, deferred: j1 }, E1 = { package_format_version: 1, namespace: "upstream-noname", package_id: "wudian", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [224, 212, 136, 182, 142, 127, 174, 224, 64, 168, 249, 131, 136, 232, 56, 12, 82, 126, 204, 103, 169, 162, 33, 160, 174, 233, 63, 60, 228, 9, 190, 247], asset_hash: [101, 186, 252, 146, 248, 33, 244, 97, 247, 227, 110, 37, 19, 234, 69, 143, 65, 21, 226, 79, 251, 191, 128, 16, 8, 88, 15, 225, 249, 137, 146, 161], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, A1 = [], C1 = [], R1 = [], z1 = [], I1 = [], T1 = [], P1 = [], O1 = [], N1 = { "zh-CN": { diaobingqianjiang: "\u8C03\u5175\u9063\u5C06", sanjian: "\u4E09\u5C16", wangmeizhike: "\u671B\u6885\u6B62\u6E34", youdishenru: "\u8BF1\u654C\u6DF1\u5165" } }, q1 = { schema_version: 1, package_id: "wudian", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "wudian:cards", source_hash: [243, 4, 96, 226, 174, 224, 22, 240, 77, 131, 190, 254, 60, 103, 228, 236, 232, 29, 21, 165, 250, 11, 71, 138, 97, 179, 174, 47, 29, 135, 117, 67], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, M1 = { "wudian:cards": "assets/cards.json" }, D1 = "gen-1", No = { manifest: E1, modules: A1, skill_index: C1, character_index: R1, card_index: z1, trigger_index: I1, modifier_index: T1, active_index: P1, view_as_index: O1, translations: N1, asset_manifest: q1, asset_paths: M1, compiler_version: D1 }, L1 = "noname", U1 = "apps/core/extension/\u73A9\u70B9\u8BBA\u6740/", F1 = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", V1 = { name: "wudian-upstream-noname", version: 2, description: "G1 \u73A9\u70B9\u8BBA\u6740", card_prefix: "wd_" }, H1 = [{ name: "sha", kind: "sha", suit: "diamond", rank: 9 }, { name: "sha", kind: "sha", suit: "spade", rank: 6 }, { name: "shan", kind: "shan", suit: "heart", rank: 6 }, { name: "shan", kind: "shan", suit: "diamond", rank: 3 }, { name: "shan", kind: "shan", suit: "diamond", rank: 11 }, { name: "tao", kind: "tao", suit: "heart", rank: 5 }, { name: "tao", kind: "tao", suit: "heart", rank: 7 }, { name: "wangmeizhike", kind: "wangmeizhike", suit: "club", rank: 8 }, { name: "wangmeizhike", kind: "wangmeizhike", suit: "club", rank: 11 }, { name: "youdishenru", kind: "youdishenru", suit: "club", rank: 9 }, { name: "youdishenru", kind: "youdishenru", suit: "spade", rank: 11 }, { name: "youdishenru", kind: "youdishenru", suit: "spade", rank: 12 }, { name: "sanjian", kind: "sanjian", suit: "club", rank: 5, range: 3, equip_skills: ["wd_sanjian"] }, { name: "tiesuo", kind: "tiesuo", suit: "spade", rank: 4 }], W1 = [{ name: "diaobingqianjiang", count: 2, reason: "\u591A\u4EBA\u65B9\u5411\u9009\u62E9\u591A\u6B65\u7ED3\u7B97" }], B1 = [{ name: "wd_wangmeizhike", card: "wangmeizhike", mapping: "host:wangmeizhike_card" }, { name: "wd_sanjian", card: "sanjian", mapping: "host:sanjian_weapon" }, { name: "moshou", general: "wdb_feishi", mapping: "host:moshou_immunity" }, { name: "ciqiu", general: "wdb_hanlong", mapping: "host:wudian_ciqiu" }, { name: "nsshuaiyan", general: "wdb_feishi", mapping: "host:wudian_shuaiyan" }, { name: "zhucheng", general: "wdb_liufu", mapping: "host:wudian_zhucheng" }, { name: "zhucheng2", general: "wdb_liufu", mapping: "host:wudian_zhucheng_guard" }, { name: "kangyin", general: "wdb_liuzan", mapping: "host:wudian_kangyin" }, { name: "chezhen", general: "wdb_tianyu", mapping: "host:wudian_chezhen" }, { name: "juedao", general: "wdb_liuyan", mapping: "host:wudian_juedao" }, { name: "geju", general: "wdb_liuyan", mapping: "host:wudian_geju" }, { name: "duoqi", general: "wdb_liufu", mapping: null, reason: "phaseUse \u4E2D\u6B62\u672A\u5EFA\u6A21+\u4F9D\u8D56\u7B51\u57CE\u6807\u8BB0\u2014\u20142026-10-08 \u6B63\u5F0F\u6682\u7F13" }, { name: "youzhan", general: "wdb_tianyu", mapping: null, reason: "shaBefore \u65F6\u70B9\u6620\u5C04\u504F\u665A+\u865A\u62DF useCard \u7EC4\u5408\u2014\u20142026-10-08 \u6B63\u5F0F\u6682\u7F13" }], K1 = [{ name: "\u8D39\u8BD7", faction: "shu", hp: 3, gender: "male", skills: [{ name: "nsshuaiyan", mapping: "host:wudian_shuaiyan" }, { name: "moshou", mapping: "host:moshou_immunity" }] }, { name: "\u97E9\u9F99", faction: "wei", hp: 4, gender: "male", skills: [{ name: "siji", mapping: "host:siji_discard_draw" }, { name: "ciqiu", mapping: "host:wudian_ciqiu" }] }, { name: "\u5218\u99A5", faction: "wei", hp: 3, gender: "male", skills: [{ name: "zhucheng", mapping: "host:wudian_zhucheng" }, { name: "zhucheng2", mapping: "host:wudian_zhucheng_guard" }, { name: "duoqi", mapping: null, reason: "phaseUse \u4E2D\u6B62\uFF08evt.skipped\uFF09\u672A\u5EFA\u6A21+\u4F9D\u8D56\u7B51\u57CE\u6807\u8BB0\u2014\u20142026-10-08 \u6B63\u5F0F\u6682\u7F13" }] }, { name: "\u5218\u7109", faction: "qun", hp: 3, gender: "male", skills: [{ name: "juedao", mapping: "host:wudian_juedao" }, { name: "geju", mapping: "host:wudian_geju" }] }, { name: "\u7559\u8D5E", faction: "wu", hp: 4, gender: "male", skills: [{ name: "kangyin", mapping: "host:wudian_kangyin" }] }, { name: "\u7530\u8C6B", faction: "wei", hp: 4, gender: "male", skills: [{ name: "chezhen", mapping: "host:wudian_chezhen" }, { name: "youzhan", mapping: null, reason: "shaBefore \u65F6\u70B9\uFF08\u539F\u751F\u6620\u5C04 AfterTriggers \u504F\u665A\uFF09+\u4EE4\u76EE\u6807\u865A\u62DF useCard \u8BF1\u654C\u6DF1\u5165\u7EC4\u5408\u2014\u20142026-10-08 \u6B63\u5F0F\u6682\u7F13" }] }, { name: "\u897F\u771F\u5915\u7EA2", faction: "shu", hp: 4, gender: "male", skills: [{ name: "jinyan", mapping: "host:wudian_jinyan" }, { name: "fuchou", mapping: "host:fuchou" }, { name: "fuchou2", mapping: "host:fuchou2" }, { name: "jianbi", mapping: "host:wudian_jianbi" }] }, { name: "\u9020\u4E4B\u4EC1\u541B", faction: "wei", hp: 3, gender: "male", skills: [{ name: "liangce", mapping: "host:liangce_remain" }, { name: "jianbi", mapping: null }, { name: "diyjuntun", mapping: "host:wudian_diyjuntun" }] }, { name: "\u6768\u4EEA", faction: "shu", hp: 3, gender: "male", skills: [{ name: "choudu", mapping: null }, { name: "liduan", mapping: "host:wudian_liduan" }] }], G1 = { source: L1, upstream_path: U1, upstream_commit: F1, pack: V1, cards: H1, deferred: W1, skills: B1, generals: K1 }, J1 = { package_format_version: 1, namespace: "upstream-liuli", package_id: "wuxing", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [132, 94, 166, 250, 221, 202, 90, 254, 41, 167, 212, 147, 169, 28, 169, 39, 198, 78, 32, 87, 170, 107, 26, 155, 120, 216, 242, 221, 92, 75, 95, 25], asset_hash: [60, 107, 102, 15, 60, 169, 0, 83, 63, 35, 181, 217, 139, 24, 141, 42, 38, 138, 111, 102, 49, 137, 183, 48, 164, 187, 206, 145, 39, 244, 93, 134], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, Y1 = [], Q1 = [], X1 = [], Z1 = [], ev = [], nv = [], tv = [], sv = [], av = { "zh-CN": { fire: "\u706B", metal: "\u91D1", soil: "\u571F", water: "\u6C34", wood: "\u6728", wuxingpan: "\u4E94\u884C\u76D8", wuxingpan_skill: "\u4E94\u884C" } }, iv = { schema_version: 1, package_id: "wuxing", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "wuxing:config", source_hash: [209, 188, 252, 3, 138, 195, 23, 199, 174, 66, 240, 192, 138, 68, 185, 89, 196, 5, 25, 128, 172, 49, 209, 89, 211, 121, 60, 76, 41, 255, 75, 222], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }, { asset_id: "wuxing:cards", source_hash: [239, 161, 205, 89, 208, 240, 33, 205, 87, 200, 152, 104, 171, 72, 97, 172, 223, 238, 146, 97, 40, 117, 119, 75, 157, 86, 203, 219, 159, 135, 28, 177], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, ov = { "wuxing:cards": "assets/cards.json", "wuxing:config": "assets/wuxing.json" }, rv = "gen-1", qo = { manifest: J1, modules: Y1, skill_index: Q1, character_index: X1, card_index: Z1, trigger_index: ev, modifier_index: nv, active_index: tv, view_as_index: sv, translations: av, asset_manifest: iv, asset_paths: ov, compiler_version: rv }, lv = "noname-liuli", uv = "noname/extension/wuxing/extension.js", cv = "8485dbb29ff16f9b258a766b1a17ef75f883aade", dv = { name: "wuxing-upstream-liuli", version: 1, description: "G1 wuxing\uFF1A\u4E94\u884C\u5C5E\u6027\u4E0E\u751F\u514B", card_prefix: "wxl_", init_hook: "wuxing_natures" }, pv = [{ name: "wuxingpan", kind: "wuxingpan", suit: "spade", rank: 5, equip_skills: ["wuxingpan_skill"] }], hv = [{ name: "wuxing_shengke", mapping: "host:shengke_check" }, { name: "wuxingpan_skill", mapping: "host:change_card_nature" }], fv = { source: lv, upstream_path: uv, upstream_commit: cv, pack: dv, cards: pv, skills: hv }, mv = { "cardpile-upstream-liuli": { package: xo, payload: { manifest: xo.manifest, pile: Vh } }, "cardpile-upstream-noname": { package: $o, payload: { manifest: $o.manifest, pile: uf } }, "d3-upstream-noname": { package: jo, payload: { manifest: jo.manifest, characters: Af } }, "huanle-upstream-noname": { package: So, payload: { manifest: So.manifest, cards: Gf } }, "shaai-ow-upstream-noname": { package: Eo, payload: { manifest: Eo.manifest, cards: _m } }, "shaai-xianjian-upstream-noname": { package: Ao, payload: { manifest: Ao.manifest, cards: Lm } }, "shahai-gujian-upstream-noname": { package: Co, payload: { manifest: Co.manifest, cards: og } }, "shahai-gwent-upstream-noname": { package: Ro, payload: { manifest: Ro.manifest, cards: Sg } }, "shahai-hearth-upstream-noname": { package: zo, payload: { manifest: zo.manifest, cards: Bg } }, "shahai-mtg-upstream-noname": { package: Io, payload: { manifest: Io.manifest, cards: p_ } }, "shahai-swd-upstream-noname": { package: To, payload: { manifest: To.manifest, cards: P_ } }, "shahai-yunchou-upstream-noname": { package: Po, payload: { manifest: Po.manifest, cards: e1, characters: o1 } }, "shahai-zhenfa-upstream-noname": { package: Oo, payload: { manifest: Oo.manifest, cards: S1 } }, "wudian-upstream-noname": { package: No, payload: { manifest: No.manifest, cards: G1 } }, "wuxing-upstream-liuli": { package: qo, payload: { manifest: qo.manifest, cards: fv } } };
function zs(e) {
  return typeof e == "string" && e.trim().length > 0 && e !== "." && e !== ".." && !e.includes("/") && !e.includes("\\");
}
function Da(e) {
  return Array.isArray(e) && e.length === 32 && e.every((n) => Number.isInteger(n) && n >= 0 && n <= 255);
}
function ls(e) {
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
  (n == null ? void 0 : n.modules) !== void 0 && gv(n);
  const s = [n == null ? void 0 : n.cards, n == null ? void 0 : n.characters, n == null ? void 0 : n.pile].filter((a) => a !== void 0);
  if (!s.length && (n == null ? void 0 : n.modules) === void 0 || s.some((a) => a === null || typeof a != "object" || Array.isArray(a))) throw new Error("\u6269\u5C55\u5305\u7F3A\u5C11\u53EF\u88C5\u8F7D\u7684\u5361\u724C\u3001\u6B66\u5C06\u3001\u724C\u5806\u6216 VM \u6570\u636E");
  return n;
}
function hs(e) {
  return Number.isInteger(e) && e >= 0;
}
function gv(e) {
  const n = e.modules, t = /* @__PURE__ */ new Map();
  for (const o of n) {
    const r = o;
    if (!r || typeof r != "object" || Array.isArray(r)) throw new Error("\u6269\u5C55\u5305 VM \u6A21\u5757\u683C\u5F0F\u65E0\u6548");
    if (!zs(r.name) || t.has(r.name)) throw new Error("\u6269\u5C55\u5305 VM \u6A21\u5757\u540D\u7F3A\u5931\u6216\u91CD\u590D");
    const l = r.code;
    if (!Array.isArray(l) || !l.length || !l.every((c) => Number.isInteger(c) && c >= 0 && c <= 255)) throw new Error(`\u6269\u5C55\u5305 VM \u6A21\u5757\u5B57\u8282\u7801\u65E0\u6548: ${r.name}`);
    if (![r.local_count, r.max_stack, r.entry_pc, r.caps_mask].every(hs) || r.entry_pc >= l.length) throw new Error(`\u6269\u5C55\u5305 VM \u6A21\u5757\u5165\u53E3\u6216\u9884\u7B97\u5B57\u6BB5\u65E0\u6548: ${r.name}`);
    if (!Da(r.logic_hash)) throw new Error(`\u6269\u5C55\u5305 VM \u6A21\u5757\u6307\u7EB9\u683C\u5F0F\u65E0\u6548: ${r.name}`);
    t.set(r.name, l.length);
  }
  const s = /* @__PURE__ */ new Set(), a = e.skill_index;
  if (a !== void 0) {
    if (!Array.isArray(a)) throw new Error("\u6269\u5C55\u5305\u6280\u80FD\u7D22\u5F15\u683C\u5F0F\u65E0\u6548");
    for (const o of a) {
      const r = o;
      if (!r || typeof r != "object" || !hs(r.skill_def_id) || !zs(r.name) || s.has(r.skill_def_id)) throw new Error("\u6269\u5C55\u5305\u6280\u80FD\u7D22\u5F15\u6761\u76EE\u65E0\u6548\u6216\u91CD\u590D");
      s.add(r.skill_def_id);
    }
  }
  const i = e.trigger_index;
  if (i !== void 0) {
    if (!Array.isArray(i)) throw new Error("\u6269\u5C55\u5305\u89E6\u53D1\u7D22\u5F15\u683C\u5F0F\u65E0\u6548");
    for (const o of i) {
      const r = o;
      if (!r || typeof r != "object" || typeof r.module != "string" || !t.has(r.module)) throw new Error("\u6269\u5C55\u5305\u89E6\u53D1\u7D22\u5F15\u5F15\u7528\u4E86\u4E0D\u5B58\u5728\u7684 VM \u6A21\u5757");
      if (!hs(r.entry_pc) || r.entry_pc >= t.get(r.module) || !hs(r.skill_def_id) || a !== void 0 && !s.has(r.skill_def_id)) throw new Error(`\u6269\u5C55\u5305\u89E6\u53D1\u7D22\u5F15\u5165\u53E3\u6216\u6280\u80FD\u5F15\u7528\u65E0\u6548: ${r.module}`);
    }
  }
}
function tt(e) {
  return `${e.manifest.namespace}/${e.manifest.package_id}`;
}
function us(e) {
  return e.manifest.logic_hash.map((n) => n.toString(16).padStart(2, "0")).join("");
}
function yl(e) {
  const n = mv[e.dir];
  if (!n) throw new Error(`\u65E0\u53EF\u5B89\u88C5\u5305\u6570\u636E: ${e.dir}`);
  if (!Array.isArray(n.package.modules)) throw new Error(`\u6269\u5C55\u5305 VM \u6A21\u5757\u683C\u5F0F\u65E0\u6548: ${e.dir}`);
  const t = n.package.modules.length ? { ...n.package, ...n.payload } : n.payload, s = ls(t);
  if (tt(s) !== `${e.namespace}/${e.packageId}` || s.manifest.version !== e.version || !us(s).startsWith(e.logicFingerprint)) throw new Error(`\u5305\u76EE\u5F55\u4E0E\u5B9E\u9645\u5B89\u88C5\u5185\u5BB9\u4E0D\u4E00\u81F4: ${e.dir}`);
  return JSON.stringify(s);
}
const kl = "noname-next.extension-store.v1", wl = "noname-next.native-saves.v1";
function fs() {
  return { installed: [], contentLock: [], matchFrozen: false };
}
function Je() {
  if (typeof localStorage > "u") return fs();
  try {
    const e = localStorage.getItem(kl);
    return e ? { ...fs(), ...JSON.parse(e) } : fs();
  } catch {
    return fs();
  }
}
function qn(e) {
  typeof localStorage > "u" || localStorage.setItem(kl, JSON.stringify(e));
}
function st() {
  return (/* @__PURE__ */ new Date()).toISOString();
}
function bl(e) {
  return Gt.find((n) => ri(n) === e);
}
function li(e, n, t) {
  const s = e == null ? void 0 : e[n];
  if (s === void 0) return;
  const a = (i) => JSON.stringify(JSON.parse(i), (o, r) => r === null || typeof r != "object" || Array.isArray(r) ? r : Object.fromEntries(Object.entries(r).sort(([l], [c]) => l < c ? -1 : l > c ? 1 : 0)));
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
async function _v(e, n) {
  const t = vv(e);
  return await n(t.canonical), yv(t);
}
function vv(e) {
  const n = Je();
  if (n.matchFrozen) throw new Error("\u5BF9\u5C40\u4E2D\u4E0D\u53EF\u5B89\u88C5\u6216\u66FF\u6362\u5305\u5185\u5BB9");
  let t;
  try {
    t = JSON.parse(e);
  } catch {
    throw new Error("\u6269\u5C55\u5305\u4E0D\u662F\u5408\u6CD5 JSON");
  }
  const s = ls(t), a = tt(s), i = s.manifest.version, o = JSON.stringify(s);
  return $l(n, a, i, o), { payload: s, key: a, version: i, canonical: o, fingerprint: us(s).slice(0, 8) };
}
function $l(e, n, t, s) {
  var _a2;
  if (e.matchFrozen) throw new Error("\u5BF9\u5C40\u4E2D\u4E0D\u53EF\u5B89\u88C5\u6216\u66FF\u6362\u5305\u5185\u5BB9");
  if ((_a2 = e.revokedKeys) == null ? void 0 : _a2.includes(n)) throw new Error(`\u5DF2\u64A4\u9500\u5305\u4E0D\u53EF\u91CD\u88C5: ${n}`);
  const a = e.installed.find((i) => i.key === n);
  if ((a == null ? void 0 : a.lifecycle) === "revoked") throw new Error(`\u5DF2\u64A4\u9500\u5305\u4E0D\u53EF\u91CD\u88C5: ${n}\uFF08\u9700\u6E05\u9664\u64A4\u9500\u8BB0\u5F55\uFF09`);
  a && li(a.nativeVersions, t, s);
}
function yv(e) {
  var _a2;
  const n = Je(), { payload: t, key: s, version: a, canonical: i, fingerprint: o } = e;
  $l(n, s, a, i);
  const r = n.installed.find((c) => c.key === s), l = st();
  return r ? (r.nativeVersions ?? (r.nativeVersions = {}), (_a2 = r.nativeVersions)[a] ?? (_a2[a] = i), r.version !== a && (r.history.includes(r.version) || r.history.push(r.version), r.version = a, r.history.includes(a) || r.history.push(a)), r.logicFingerprint = o, r.updatedAt = l, r.signature = "unsigned", r.lifecycle === "available" && (r.lifecycle = "installed")) : n.installed.push({ key: s, dir: "imported", packageId: t.manifest.package_id, namespace: t.manifest.namespace, version: a, history: [a], lifecycle: "installed", installedAt: l, updatedAt: l, logicFingerprint: o, nativeVersions: { [a]: i }, signature: "unsigned" }), qn(n), n;
}
function jl(e) {
  const n = Je();
  if (n.matchFrozen) throw new Error("\u5BF9\u5C40\u4E2D\u4E0D\u53EF\u53D8\u66F4\u5185\u5BB9\u9501");
  const t = n.installed.find((s) => s.key === e);
  if (!t || t.lifecycle === "revoked") throw new Error(`\u672A\u5B89\u88C5\u6216\u5DF2\u64A4\u9500: ${e}`);
  return t.lifecycle = "active", t.updatedAt = st(), n.contentLock.includes(e) || n.contentLock.push(e), n.contentLock.sort(), qn(n), n;
}
function kv(e) {
  const n = Je();
  if (n.matchFrozen) throw new Error("\u5BF9\u5C40\u4E2D\u4E0D\u53EF\u53D8\u66F4\u5185\u5BB9\u9501");
  const t = n.installed.find((s) => s.key === e);
  if (!t) throw new Error(`\u672A\u5B89\u88C5: ${e}`);
  if (t.lifecycle === "revoked") throw new Error(`\u5DF2\u64A4\u9500: ${e}`);
  return t.lifecycle = "disabled", t.updatedAt = st(), n.contentLock = n.contentLock.filter((s) => s !== e), qn(n), n;
}
function wv(e, n) {
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
function bv(e, n) {
  var _a2;
  const t = Je();
  if (t.matchFrozen) throw new Error("\u5BF9\u5C40\u4E2D\u4E0D\u53EF\u56DE\u9000");
  const s = t.installed.find((i) => i.key === e);
  if (!s || s.lifecycle === "revoked") throw new Error(`\u672A\u5B89\u88C5\u6216\u5DF2\u64A4\u9500: ${e}`);
  if (!s.history.includes(n)) throw new Error(`\u65E0\u5386\u53F2\u7248\u672C ${n}`);
  if (!((_a2 = s.nativeVersions) == null ? void 0 : _a2[n])) throw new Error(`\u7F3A\u5C11\u5DF2\u5B89\u88C5\u7684\u7248\u672C\u5185\u5BB9 ${n}`);
  const a = ls(JSON.parse(s.nativeVersions[n]));
  if (tt(a) !== e || a.manifest.version !== n) throw new Error(`\u5386\u53F2\u5305\u8EAB\u4EFD\u6216\u7248\u672C\u4E0D\u5339\u914D: ${e}`);
  return s.version = n, s.logicFingerprint = us(a).slice(0, 8), s.updatedAt = st(), qn(t), t;
}
function xv(e) {
  const n = Je();
  if (n.matchFrozen) throw new Error("\u5BF9\u5C40\u4E2D\u4E0D\u53EF\u5378\u8F7D");
  return n.installed.some((t) => t.key === e && t.lifecycle === "revoked") && (n.revokedKeys ?? (n.revokedKeys = []), n.revokedKeys.includes(e) || n.revokedKeys.push(e)), n.contentLock = n.contentLock.filter((t) => t !== e), n.installed = n.installed.filter((t) => t.key !== e), qn(n), n;
}
function $v(e) {
  const n = Je();
  if (n.matchFrozen) throw new Error("\u5BF9\u5C40\u4E2D\u4E0D\u53EF\u64A4\u9500");
  const t = n.installed.find((s) => s.key === e);
  if (!t) throw new Error(`\u672A\u5B89\u88C5: ${e}`);
  return t.lifecycle = "revoked", n.revokedKeys ?? (n.revokedKeys = []), n.revokedKeys.includes(e) || n.revokedKeys.push(e), t.updatedAt = st(), n.contentLock = n.contentLock.filter((s) => s !== e), qn(n), n;
}
function jv(e = Je()) {
  const n = e.contentLock.map((s) => {
    const a = e.installed.find((i) => i.key === s);
    return a ? { key: s, version: a.version, fingerprint: a.logicFingerprint } : null;
  }).filter(Boolean), t = n.map((s) => `${s.key}@${s.version}:${s.fingerprint}`).join("|");
  return { packages: n, hashSeed: t };
}
function Sv() {
  let e = Je();
  for (const n of Gt) {
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
function Ev(e) {
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
    if (o.manifest.version !== a.version || (r == null ? void 0 : r.some((l) => l !== 0)) && r.some((l, c) => l !== o.manifest.logic_hash[c])) throw new Error(`\u6269\u5C55\u5305\u4F9D\u8D56\u7248\u672C\u6216\u6307\u7EB9\u4E0D\u5339\u914D: ${i}@${a.version}`);
  }
  return { packsJson: e.length ? JSON.stringify(e) : void 0, contentLock: n, lockSeed: e.map((s) => `${tt(s)}@${s.manifest.version}:${us(s)}`).join("|") };
}
function Av(e = Is()) {
  const n = e.contentLock.map((t) => {
    var _a2;
    const s = e.installed.find((o) => o.key === t);
    if (!s || s.lifecycle !== "active") throw new Error(`\u5185\u5BB9\u9501\u4E2D\u7684\u5305\u672A\u542F\u7528: ${t}`);
    const a = (_a2 = s.nativeVersions) == null ? void 0 : _a2[s.version];
    if (!a) throw new Error(`\u7F3A\u5C11\u5DF2\u5B89\u88C5\u5305\u5185\u5BB9: ${t}\uFF0C\u8BF7\u5148\u5728\u6269\u5C55\u7BA1\u7406\u9875\u5347\u7EA7\u6216\u91CD\u65B0\u5B89\u88C5`);
    const i = ls(JSON.parse(a));
    if (tt(i) !== t || i.manifest.version !== s.version || !/^[a-f0-9]{8,64}$/.test(s.logicFingerprint) || !us(i).startsWith(s.logicFingerprint)) throw new Error(`\u5DF2\u5B89\u88C5\u5305\u7248\u672C\u6216\u6307\u7EB9\u4E0D\u5339\u914D: ${t}`);
    return i;
  });
  return El(n);
}
function Cv(e, n = Is()) {
  var _a2;
  const t = e.packsJson ? JSON.parse(e.packsJson) : [];
  if (!Array.isArray(t)) throw new Error("\u5B58\u6863\u6269\u5C55\u5185\u5BB9\u5FC5\u987B\u662F\u6570\u7EC4");
  const s = El(t.map(ls));
  if (s.lockSeed !== e.lockSeed || JSON.stringify(s.contentLock) !== JSON.stringify(e.contentLock)) throw new Error("\u5B58\u6863\u5185\u5BB9\u9501\u4E0E\u5305\u526F\u672C\u4E0D\u5339\u914D\uFF1B\u65E7\u6269\u5C55\u5B58\u6863\u7F3A\u5C11\u5B9E\u9645\u5305\u5185\u5BB9\u65F6\u4E0D\u80FD\u6062\u590D");
  for (const a of s.contentLock) if (((_a2 = n.revokedKeys) == null ? void 0 : _a2.includes(a)) || n.installed.some((i) => i.key === a && i.lifecycle === "revoked")) throw new Error(`\u5B58\u6863\u5305\u542B\u5DF2\u64A4\u9500\u5305: ${a}`);
  return s;
}
const Rv = { 1: "\u5BF9\u5C40\u5F00\u59CB", 2: "\u6478\u724C", 3: "\u4F7F\u7528\u6216\u54CD\u5E94\u724C", 4: "\u4F24\u5BB3\u7ED3\u7B97", 5: "\u56DE\u590D\u4F53\u529B", 6: "\u6FD2\u6B7B\u72B6\u6001\u53D8\u5316", 7: "\u73A9\u5BB6\u9635\u4EA1", 8: "\u9636\u6BB5\u6216\u56DE\u5408\u63A8\u8FDB", 9: "\u5BF9\u5C40\u7ED3\u675F", 10: "\u9009\u62E9\u72B6\u6001\u53D8\u5316" };
function zv(e) {
  return Rv[e] ?? `\u4E8B\u4EF6 ${e}`;
}
const Tt = 6, Ts = "noname-next.pending-command", Iv = 600 * 1e3;
function Tv() {
  try {
    if (typeof sessionStorage > "u") return null;
    const e = sessionStorage.getItem(Ts);
    if (!e) return null;
    const n = JSON.parse(e);
    return !n || typeof n.roomId != "number" || typeof n.key != "number" || typeof n.createdAt != "number" || !n.kind || Date.now() - n.createdAt > Iv ? null : { roomId: n.roomId, key: n.key, kind: n.kind, createdAt: n.createdAt };
  } catch {
    return null;
  }
}
function Pv(e, n, t) {
  try {
    if (typeof sessionStorage > "u" || !t) return;
    sessionStorage.setItem(Ts, JSON.stringify({ roomId: t, key: n, kind: e, createdAt: Date.now() }));
  } catch {
  }
}
function ms(e) {
  try {
    if (typeof sessionStorage > "u") return;
    const n = sessionStorage.getItem(Ts);
    if (!n) return;
    JSON.parse(n).key === e && sessionStorage.removeItem(Ts);
  } catch {
  }
}
class Ov {
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
    __publicField(this, "persistedPending", Tv());
    __publicField(this, "snapshotWaiters", /* @__PURE__ */ new Map());
    __publicField(this, "envelopeListeners", /* @__PURE__ */ new Set());
    __publicField(this, "rosterListeners", /* @__PURE__ */ new Set());
    __publicField(this, "closeListeners", /* @__PURE__ */ new Set());
    __publicField(this, "heartbeat", null);
    __publicField(this, "clockProbe", null);
    __publicField(this, "serverTimeAnchor", null);
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
    this.disposed = false, this.clockProbe = null, this.serverTimeAnchor = null, this.lastUrl = n, this.lastName = t;
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
    }), await a, await s, this.send({ type: "Hello", payload: { versions: { engine_api: 1, rule_vm_abi: 167, content_schema: 3, network_protocol: Tt, replay_schema: 1, snapshot_schema: 1, package_format: 1, engine_build_id: "noname-next-web" }, display_name: t } }), this.sendHeartbeat();
  }
  secondsUntilServerDeadline(n) {
    if (n == null || !this.serverTimeAnchor) return null;
    const t = performance.now() - this.serverTimeAnchor.monotonicMs;
    return Math.max(0, Math.ceil((n - this.serverTimeAnchor.unixMs - t) / 1e3));
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
    this.disposed = true, this.reconnectTimer != null && (window.clearTimeout(this.reconnectTimer), this.reconnectTimer = null), this.reconnectAttempts = 0, this.heartbeat != null && typeof window < "u" && window.clearInterval(this.heartbeat), this.heartbeat = null, this.clockProbe = null, this.serverTimeAnchor = null;
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
    return { protocol_version: Tt, room_id: this.roomId, conn_seq: ++this.connSeq, revision: this.revision };
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
        if (s.protocol_version !== Tt || ((_a2 = s.versions) == null ? void 0 : _a2.network_protocol) !== Tt) {
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
      case "Pong": {
        const a = s.header, i = this.clockProbe, o = performance.now(), r = s.server_time_ms;
        i && (a == null ? void 0 : a.protocol_version) === Tt && a.conn_seq === i.connSeq && a.room_id === i.roomId && typeof r == "number" && Number.isSafeInteger(r) && r > 0 && o >= i.sentAt && (this.serverTimeAnchor = { unixMs: r + (o - i.sentAt) / 2, monotonicMs: o }, this.clockProbe = null);
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
      const l = Mv(n.result);
      if (l) {
        for (const c of this.envelopeListeners) c(i);
        r.reject(new Error(l));
      } else r.resolve(i);
    } else for (const l of this.envelopeListeners) l(i);
    s > 0 && t.revision > s + 1 && this.requestSnapshot().catch(() => {
    });
  }
  startHeartbeat() {
    this.heartbeat != null && window.clearInterval(this.heartbeat), this.heartbeat = window.setInterval(() => {
      var _a2;
      ((_a2 = this.socket) == null ? void 0 : _a2.readyState) === WebSocket.OPEN && this.sendHeartbeat();
    }, 15e3);
  }
  sendHeartbeat() {
    const n = this.nextHeader();
    this.clockProbe = { connSeq: n.conn_seq, roomId: n.room_id, sentAt: performance.now() }, this.send({ type: "Heartbeat", payload: { header: n } });
  }
  handleClose() {
    this.clockProbe = null, this.serverTimeAnchor = null, this.heartbeat != null && window.clearInterval(this.heartbeat);
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
  return { result: a, events: t.map((i) => ({ PublicEvent: { code: i, label: zv(i) } })), choice: n, view: e, legal_actions: s, hash_hex: Nv(e.state_hash), revision: e.revision, finished: qv(e.outcome) };
}
function Nv(e) {
  return typeof e == "string" ? e : e.map((n) => n.toString(16).padStart(2, "0")).join("");
}
function qv(e) {
  return e == null || e === "InProgress" ? false : !(typeof e == "object" && "InProgress" in e);
}
function Mv(e) {
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
function Dv(e) {
  return Cl(e).label;
}
function Lv(e, n, t) {
  var _a2;
  let s = e == null ? void 0 : e.outcome;
  if (s == null) for (const a of n) {
    if (!a || typeof a != "object") continue;
    const i = a;
    if (s = i.type === "MatchEnded" ? i.outcome : (_a2 = i.MatchEnded) == null ? void 0 : _a2.outcome, s != null) break;
  }
  return Cl(s, e == null ? void 0 : e.seats.find((a) => a.player === t));
}
function Uv(e) {
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
let Fv = 1;
function Vv() {
  var _a2;
  const e = typeof window < "u" && ((_a2 = window.matchMedia) == null ? void 0 : _a2.call(window, "(prefers-reduced-motion: reduce)").matches);
  return xt({ connected: false, busy: false, error: null, revision: 0, hash: "", finished: false, view: null, choice: null, legalActions: [], lastEvents: [], log: [], effects: [], speed: 1, skipAnims: false, reduceMotion: !!e });
}
let it = 1;
function Lo(e, n) {
  lh(), Hv(e, n), uh();
}
function Hv(e, n) {
  e.revision = n.revision, e.hash = n.hash_hex, e.finished = n.finished, e.view = n.view, e.choice = n.choice, e.legalActions = n.legal_actions ?? [], e.lastEvents = n.events ?? [], e.error = null;
  const t = Wv(n.events ?? []);
  e.effects = t;
  const s = Kv(n.events ?? []);
  for (const a of s) Qe(e, a);
}
function Wv(e) {
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
function Bv(e) {
  e.effects = [];
}
function Qe(e, n) {
  e.log.unshift({ id: Fv++, text: n, at: Date.now() }), e.log.length > 80 && (e.log.length = 80);
}
function Kv(e) {
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
        n.push(`\u7EC8\u5C40 \xB7 ${Dv(i == null ? void 0 : i.outcome)}`);
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
function Gv(e, n) {
  if (e.skipAnims || e.reduceMotion || e.speed <= 0) return Promise.resolve();
  const t = Math.max(0, n / e.speed);
  return new Promise((s) => setTimeout(s, t));
}
function lt(e) {
  return typeof e == "object" && e !== null && !Array.isArray(e);
}
function Ps(e) {
  return typeof e == "string" ? e : e instanceof Error ? e.message : String(e);
}
function Dt(e) {
  try {
    const n = JSON.stringify(e);
    return (n === void 0 ? String(e) : n).slice(0, 160);
  } catch {
    return String(e);
  }
}
function Jv(e) {
  return Array.isArray(e) && e.every((n) => typeof n == "string");
}
function Yv(e) {
  if (!lt(e)) throw new Error(`\u9009\u5C06\u5019\u9009\u7ED3\u6784\u4E0D\u7B26\uFF08\u9700 {count, candidates}\uFF09\uFF1A${Dt(e)}`);
  const n = e.count, t = e.candidates;
  if (typeof n != "number" || !Number.isInteger(n) || n < 0 || !Array.isArray(t)) throw new Error(`\u9009\u5C06\u5019\u9009\u7ED3\u6784\u4E0D\u7B26\uFF08\u9700 {count:\u975E\u8D1F\u6574\u6570, candidates:\u6570\u7EC4}\uFF09\uFF1A${Dt(e)}`);
  const s = [], a = /* @__PURE__ */ new Set();
  for (const [i, o] of t.entries()) {
    if (!lt(o)) throw new Error(`\u9009\u5C06\u5019\u9009\u7B2C ${i} \u9879\u7ED3\u6784\u4E0D\u7B26\uFF08\u9700\u5BF9\u8C61\uFF09\uFF1A${Dt(o)}`);
    const r = o.def, l = o.name, c = o.skills;
    if (typeof r != "number" || !Number.isInteger(r) || r < 0 || typeof l != "string" || l.trim().length === 0 || !Jv(c)) throw new Error(`\u9009\u5C06\u5019\u9009\u7B2C ${i} \u9879\u7ED3\u6784\u4E0D\u7B26\uFF08\u9700 {def:\u975E\u8D1F\u6574\u6570, name:\u975E\u7A7A\u5B57\u7B26\u4E32, skills:\u5B57\u7B26\u4E32\u6570\u7EC4}\uFF09\uFF1A${Dt(o)}`);
    if (a.has(r)) throw new Error(`\u9009\u5C06\u5019\u9009\u7B2C ${i} \u9879 def=${r} \u4E0E\u524D\u9762\u91CD\u590D`);
    a.add(r), s.push({ def: r, name: l, skills: [...c], skillLabel: c.length > 0 ? c.join(" \xB7 ") : "\u65E0\u6280\u80FD" });
  }
  if (s.length !== n) throw new Error(`\u9009\u5C06\u5019\u9009 count=${n} \u4E0E\u5019\u9009\u6570 ${s.length} \u4E0D\u4E00\u81F4`);
  return s;
}
function ui() {
  return { characterDef: null, skillIds: [], skillNames: [] };
}
function Qv(e, n = ui()) {
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
    const c = r == null ? void 0 : r.name;
    typeof c == "string" && c.length > 0 && !i.includes(c) && i.push(c);
  }
  return { characterDef: typeof s.character_def == "number" ? s.character_def : null, skillIds: a, skillNames: i };
}
function Il(e, n) {
  return { ...e, projection: Rl(n) };
}
function Uo(e, n) {
  return { ...e, seat: n, projection: ui(), assignment: null, lastStateHash: null, selectionStatus: "idle", pendingName: null, error: null };
}
function Xv(e) {
  return { ...e, loadStatus: "loading", error: null };
}
function Zv(e, n) {
  try {
    const t = Yv(n);
    return { ...e, loadStatus: t.length > 0 ? "ready" : "empty", candidates: t, error: null };
  } catch (t) {
    return { ...e, loadStatus: "error", candidates: [], error: `\u9009\u5C06\u5019\u9009\u4E0D\u53EF\u7528\uFF1A${Ps(t)}` };
  }
}
function ey(e, n) {
  return { ...e, loadStatus: "error", candidates: [], error: `\u9009\u5C06\u5019\u9009\u52A0\u8F7D\u5931\u8D25\uFF1A${Ps(n)}` };
}
function ny(e, n) {
  if (!lt(e)) return `\u56DE\u5305\u4E0D\u662F\u5BF9\u8C61\uFF1A${Dt(e)}`;
  const t = e.assignment, s = e.envelope;
  return lt(t) ? t.seat !== n.seat ? `\u5EA7\u4F4D\u56DE\u663E ${String(t.seat)} \u2260 \u8BF7\u6C42 ${n.seat}` : t.name !== n.name ? `\u6B66\u5C06\u56DE\u663E ${String(t.name)} \u2260 \u8BF7\u6C42 ${n.name}` : typeof t.state_hash != "string" || t.state_hash.length === 0 ? "\u7F3A assignment.state_hash" : !lt(s) || !lt(s.view) ? "\u7F3A\u5237\u65B0\u4FE1\u5C01 view" : null : "\u7F3A assignment";
}
async function ty(e, n, t, s) {
  var _a2;
  if (typeof n != "string" || n.trim().length === 0) return { ...e, selectionStatus: "rejected", pendingName: null, error: "\u9009\u5C06\u5931\u8D25\uFF1A\u5019\u9009\u540D\u4E3A\u7A7A" };
  const a = { ...e, selectionStatus: "pending", pendingName: n, error: null };
  s == null ? void 0 : s(a);
  let i;
  try {
    i = await t.chooseGeneral(e.seat, n);
  } catch (p) {
    return { ...e, selectionStatus: "rejected", pendingName: null, error: `\u9009\u5C06\u5931\u8D25\uFF08seat=${e.seat}, name=${n}\uFF09\uFF1A${Ps(p)}` };
  }
  const o = ny(i, { seat: e.seat, name: n });
  if (o) return { ...e, selectionStatus: "rejected", pendingName: null, error: `\u9009\u5C06\u8FD4\u56DE\u65E0\u6CD5\u786E\u8BA4\u88C5\u914D\uFF1A${o}` };
  const r = i.assignment, l = i.envelope, c = Il({ ...a, assignment: r, lastStateHash: r.state_hash }, zl(l.view, e.seat));
  let d = null;
  try {
    (_a2 = t.applyEnvelope) == null ? void 0 : _a2.call(t, l);
  } catch (p) {
    d = `\u9009\u5C06\u5DF2\u88C5\u914D\uFF0C\u4F46\u89C6\u56FE\u5237\u65B0\u5931\u8D25\uFF1A${Ps(p)}`;
  }
  return { ...c, selectionStatus: "accepted", pendingName: null, error: d };
}
const sy = ["data-slot", "aria-label"], ay = ["src", "alt"], iy = ["aria-label"], oy = { class: "name" }, ry = { key: 2, class: "muted" }, ly = ["value", "aria-label"], uy = ["value"], cy = De({ __name: "GeneralPortrait", props: { general: {}, legacyDef: {}, alive: { type: Boolean }, slot: {} }, setup(e) {
  const n = e, t = X(() => {
    var _a2;
    return ((_a2 = n.general) == null ? void 0 : _a2.def) ?? n.legacyDef ?? 0;
  }), s = X(() => n.general !== null || n.legacyDef !== void 0), a = X(() => {
    var _a2;
    return s.value ? ((_a2 = n.general) == null ? void 0 : _a2.name) || mh(t.value) || `#${t.value}` : "\u6697\u5C06\uFF0F\u672A\u914D\u7F6E";
  }), i = { wei: "\u9B4F", shu: "\u8700", wu: "\u5434", qun: "\u7FA4" }, o = X(() => {
    var _a2;
    const j = ((_a2 = n.general) == null ? void 0 : _a2.faction) ?? "";
    return Object.hasOwn(i, j) ? i[j] : "\u672A\u77E5\u52BF\u529B";
  }), r = X(() => gh(t.value)), l = X(() => Na(t.value)), c = X(() => yh(t.value)), d = X(() => kh(t.value)), p = X(() => Ys(t.value)), m = X(() => Array.from(/* @__PURE__ */ new Set([...n.alive ? [] : [wh(t.value, "dead")], c.value, d.value, p.value])).filter((j) => !!j)), f = B([]), k = X(() => m.value.find((j) => !f.value.includes(j)) ?? "");
  kn([t, s], ([j, $]) => {
    $ && fh(j).catch(() => {
    });
  }, { immediate: true }), kn(m, () => {
    f.value = [];
  });
  function T() {
    k.value && (f.value = [...f.value, k.value]);
  }
  function W(j) {
    const $ = j.target.value;
    vh(t.value, $ === "" ? null : Number($));
  }
  return (j, $) => (w(), x("div", { class: "general-slot", "data-slot": e.slot, "aria-label": `${e.slot === 0 ? "\u4E3B\u5C06" : "\u526F\u5C06"} ${a.value}` }, [s.value && k.value ? (w(), x("img", { key: 0, class: "portrait", src: k.value, alt: `${a.value}\u6B66\u5C06\u5934\u50CF`, onError: T }, null, 40, ay)) : (w(), x("div", { key: 1, class: "portrait no-image", role: "img", "aria-label": `${a.value}\u6682\u65E0\u5934\u50CF` }, b(s.value ? "\u65E0\u56FE" : "\u672A\u77E5"), 9, iy)), u("span", oy, b(a.value), 1), e.general ? (w(), x("span", ry, b(o.value) + " \xB7 " + b(e.general.revealed ? "\u660E\u7F6E" : "\u6697\u7F6E"), 1)) : V("", true), s.value && r.value.length ? (w(), x("select", { key: 3, class: "skin-select", value: l.value ?? "", "aria-label": `${a.value}\u76AE\u80A4`, onClick: $[0] || ($[0] = ja(() => {
  }, ["stop"])), onKeydown: $[1] || ($[1] = ja(() => {
  }, ["stop"])), onChange: W }, [$[2] || ($[2] = u("option", { value: "" }, "\u9ED8\u8BA4\u76AE\u80A4", -1)), (w(true), x(te, null, ge(r.value, (A) => (w(), x("option", { key: A.id, value: A.id }, b(L(_h)(A)) + b(A.presentation.kind === "spine" ? "\uFF08\u9759\u6001\u56DE\u9000\uFF09" : ""), 9, uy))), 128))], 40, ly)) : V("", true)], 8, sy));
} }), dy = ln(cy, [["__scopeId", "data-v-689e5d5b"]]), py = ["aria-label", "tabindex"], hy = { key: 0, class: "tag" }, fy = { key: 1, class: "tag", "aria-label": "\u6B66\u5C06\u724C\u80CC\u9762" }, my = { class: "generals", "aria-label": "\u53CC\u5C06" }, gy = { class: "hp-row" }, _y = ["aria-label"], vy = { class: "hp", "aria-label": "\u4F53\u529B" }, yy = ["aria-valuenow", "aria-valuemax", "aria-label"], ky = { class: "meta muted" }, wy = { key: 0, class: "equip-zone", "aria-label": "\u88C5\u5907\u533A" }, by = ["title"], xy = { class: "equip-suit" }, $y = { class: "equip-id" }, jy = { key: 1, class: "dead-badge" }, Sy = { key: 2, class: "dying-badge" }, Ey = { key: 3, class: "target-hint" }, Ay = De({ __name: "SeatCard", props: { seat: {}, self: { type: Boolean }, identityLabel: {}, isTargetMode: { type: Boolean }, isSelectableTarget: { type: Boolean }, isDropTarget: { type: Boolean }, equipCards: {} }, emits: ["select"], setup(e, { emit: n }) {
  const t = e, s = n, a = X(() => t.seat.generals ?? [null, null]), i = X(() => !t.seat.generals || a.value.every((c) => c === null) && t.seat.character_def !== 0 ? t.seat.character_def : void 0), o = X(() => a.value.map((c, d) => (c == null ? void 0 : c.name) || (d === 0 && i.value !== void 0 ? `#${i.value}` : "\u6697\u5C06\uFF0F\u672A\u914D\u7F6E")).join(" / "));
  function r(c, d, p) {
    return p ? c <= 0 ? "\u26A0" : "\u2713" : "\u2717";
  }
  function l(c, d, p) {
    return p ? c <= 0 ? "var(--color-warning)" : "var(--color-positive)" : "var(--color-negative)";
  }
  return (c, d) => (w(), x("article", { class: Re(["seat has-portrait", { self: e.self, dead: !e.seat.alive, dying: e.seat.alive && e.seat.hp <= 0, "target-mode": e.isTargetMode, "target-selectable": e.isTargetMode && e.isSelectableTarget, "drop-target": e.isDropTarget, "current-turn": false }]), role: "listitem", "aria-label": `\u5EA7\u4F4D ${e.seat.seat} \u73A9\u5BB6 ${e.seat.player} \u6B66\u5C06 ${o.value}`, tabindex: e.isTargetMode && e.isSelectableTarget ? 0 : -1, onClick: d[0] || (d[0] = (p) => e.isTargetMode && e.isSelectableTarget && s("select", e.seat.player)), onKeydown: d[1] || (d[1] = ed((p) => e.isTargetMode && e.isSelectableTarget && s("select", e.seat.player), ["enter"])) }, [u("header", null, [e.self ? (w(), x("span", hy, "\u4F60")) : V("", true), e.seat.turned_over ? (w(), x("span", fy, "\u7FFB\u9762")) : V("", true)]), u("div", my, [(w(true), x(te, null, ge(a.value, (p, m) => (w(), ti(dy, { key: m, general: p, slot: m, "legacy-def": m === 0 ? i.value : void 0, alive: e.seat.alive }, null, 8, ["general", "slot", "legacy-def", "alive"]))), 128))]), u("div", gy, [u("span", { class: "state-icon", style: Kn({ color: l(e.seat.hp, e.seat.max_hp, e.seat.alive) }), "aria-label": e.seat.alive ? e.seat.hp <= 0 ? "\u6FD2\u6B7B" : "\u5B58\u6D3B" : "\u9635\u4EA1" }, b(r(e.seat.hp, e.seat.max_hp, e.seat.alive)), 13, _y), u("span", vy, b(e.seat.hp) + " / " + b(e.seat.max_hp), 1)]), u("div", { class: "hp-bar", role: "progressbar", "aria-valuenow": Math.max(e.seat.hp, 0), "aria-valuemax": e.seat.max_hp, "aria-label": `\u4F53\u529B ${e.seat.hp}/${e.seat.max_hp}` }, [u("div", { class: "hp-bar-fill", style: Kn({ width: Math.max(Math.min(e.seat.hp / e.seat.max_hp * 100, 100), 0) + "%", background: e.seat.alive ? e.seat.hp <= 0 ? "var(--color-warning)" : "var(--color-positive)" : "var(--color-negative)" }) }, null, 4)], 8, yy), u("div", ky, [u("span", null, "\u624B\u724C " + b(e.seat.hand_count), 1), u("span", null, b(e.identityLabel), 1)]), e.equipCards.length > 0 ? (w(), x("div", wy, [(w(true), x(te, null, ge(e.equipCards, (p) => (w(), x("div", { key: p.id, class: "equip-card", title: `\u5361\u724C #${p.id}${p.def != null ? " (def:" + p.def + ")" : ""}` }, [u("span", xy, b(p.suit === "Spade" ? "\u2660" : p.suit === "Heart" ? "\u2665" : p.suit === "Club" ? "\u2663" : p.suit === "Diamond" ? "\u2666" : "?"), 1), u("span", $y, "#" + b(p.id), 1)], 8, by))), 128))])) : V("", true), e.seat.alive ? e.seat.hp <= 0 ? (w(), x("div", Sy, "\u26A0 \u6FD2\u6B7B")) : V("", true) : (w(), x("div", jy, "\u2717 \u9635\u4EA1")), e.isTargetMode && e.isSelectableTarget ? (w(), x("div", Ey, " \u25B8 \u70B9\u51FB\u9009\u62E9\u4E3A\u76EE\u6807 ")) : V("", true)], 42, py));
} }), Cy = ln(Ay, [["__scopeId", "data-v-fdcad0a8"]]), Ry = { class: "hand", "aria-label": "\u624B\u724C" }, zy = { class: "muted title" }, Iy = { class: "strip", role: "listbox", "aria-label": "\u624B\u724C\u5217\u8868" }, Ty = ["disabled", "aria-selected", "aria-label", "onClick", "onDragstart"], Py = ["data-suit"], Oy = { class: "rank" }, Ny = { class: "id muted" }, qy = { key: 0, class: "card-known" }, My = { key: 0, class: "muted empty" }, Dy = { key: 0, class: "hint" }, Ly = { key: 1, class: "hint drag-hint" }, Uy = De({ __name: "HandStrip", props: { cards: {}, actionableIds: {} }, emits: ["select", "dragStart"], setup(e, { emit: n }) {
  const t = e, s = n, a = B(null), i = B(null);
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
  function c(p, m) {
    if (!t.actionableIds.includes(m)) {
      p.preventDefault();
      return;
    }
    i.value = m, p.dataTransfer && (p.dataTransfer.setData("text/plain", String(m)), p.dataTransfer.effectAllowed = "move"), s("dragStart", m);
  }
  function d() {
    i.value = null;
  }
  return (p, m) => (w(), x("section", Ry, [u("h2", zy, " \u624B\u724C\uFF08" + b(e.cards.length) + " \u5F20" + b(a.value != null ? " \xB7 \u5DF2\u9009\u4E2D #" + a.value : " \xB7 \u70B9\u51FB\u6216\u62D6\u62FD") + "\uFF09 ", 1), u("div", Iy, [(w(true), x(te, null, ge(e.cards, (f) => (w(), x("button", { key: f.id, type: "button", class: Re(["card", { selected: f.id === a.value, dragging: f.id === i.value }]), disabled: !e.actionableIds.includes(f.id), role: "option", "aria-selected": f.id === a.value, "aria-label": `\u5361\u724C ${f.id}${f.def != null ? " \u5DF2\u77E5" : " \u672A\u77E5"}`, draggable: "true", onClick: (k) => l(f.id), onDragstart: (k) => c(k, f.id), onDragend: d }, [u("span", { class: "suit", "data-suit": f.suit }, b(o(f.suit)), 9, Py), u("span", Oy, b(r(f.rank)), 1), u("span", Ny, "#" + b(f.id), 1), f.def != null ? (w(), x("span", qy, "\u25B8")) : V("", true)], 42, Ty))), 128)), e.cards.length ? V("", true) : (w(), x("p", My, "\u65E0\u53EF\u89C1\u624B\u724C"))]), a.value != null ? (w(), x("p", Dy, " \u2713 \u5DF2\u9009\u4E2D #" + b(a.value) + " \u2014 \u5728\u68CB\u76D8\u4E0A\u9009\u62E9\u76EE\u6807\uFF0C\u6216\u70B9\u51FB\u300C\u7ED3\u675F\u9636\u6BB5\u300D\u76F4\u63A5\u6253\u51FA ", 1)) : V("", true), i.value != null ? (w(), x("p", Ly, " \u{1F0A0} \u62D6\u62FD\u4E2D #" + b(i.value) + " \u2014 \u62D6\u5230\u76EE\u6807\u73A9\u5BB6\u91CA\u653E\u51FA\u724C ", 1)) : V("", true)]));
} }), Fy = ln(Uy, [["__scopeId", "data-v-b2b16095"]]), Vy = { class: "board", role: "region", "aria-label": "\u6218\u573A" }, Hy = { key: 0, class: "action-bar panel", role: "toolbar", "aria-label": "\u884C\u52A8\u64CD\u4F5C" }, Wy = { class: "muted" }, By = { key: 2 }, Ky = { key: 3 }, Gy = { class: "row" }, Jy = ["disabled"], Yy = ["disabled"], Qy = { key: 1, class: "skill-bar panel", "aria-label": "\u53EF\u53D1\u52A8\u6280\u80FD" }, Xy = ["onClick"], Zy = { class: "seats", role: "list", "aria-label": "\u5EA7\u4F4D\u5217\u8868" }, ek = ["onDragover", "onDragleave", "onDrop"], nk = { key: 2, class: "judgment-zone", "aria-label": "\u5224\u5B9A\u533A" }, tk = { class: "jz-entries" }, sk = ["title"], ak = { class: "jz-suit" }, ik = { class: "jz-player" }, ok = { key: 3, class: "discard-pile", "aria-label": "\u5F03\u724C\u5806" }, rk = { class: "muted discard-title" }, lk = { class: "discard-strip" }, uk = ["title"], ck = { class: "d-suit" }, dk = { key: 0, class: "muted discard-more" }, pk = De({ __name: "BattleBoard", props: { view: {}, selfId: {}, legalActions: {} }, emits: ["playCard", "activateSkill"], setup(e, { emit: n }) {
  const t = e, s = n, a = B(null), i = B(null), o = B(false), r = B([]), l = B(null), c = B(null);
  function d() {
    var _a2;
    return ((_a2 = t.view) == null ? void 0 : _a2.seats) ?? [];
  }
  function p() {
    return t.view ? t.view.known_cards.filter((q) => q.owner === t.selfId && q.zone === "Hand" && q.def != null) : [];
  }
  function m() {
    var _a2;
    const q = new Set(t.legalActions.flatMap((F) => typeof F == "object" && "ActivateSkill" in F ? [F.ActivateSkill.skill] : []));
    return ((_a2 = d().find((F) => F.player === t.selfId)) == null ? void 0 : _a2.skills.filter((F) => F.enabled && (F.active || F.view_as) && q.has(F.id))) ?? [];
  }
  function f(q) {
    switch (q) {
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
  function k() {
    return o.value ? new Set(O().flat()) : /* @__PURE__ */ new Set();
  }
  function T() {
    return t.view ? t.view.known_cards.filter((q) => q.zone === "DiscardPile" && q.def != null) : [];
  }
  function W(q) {
    return t.view ? t.view.known_cards.filter((F) => F.owner === q && F.zone === "Judge" && F.def != null) : [];
  }
  const j = X(() => k()), $ = X(() => i.value ? Array.from(new Set(P(i.value.id).flatMap((q) => q.card_targets))) : Array.from(new Set(A().map((q) => q.card))));
  function A(q) {
    return t.legalActions.flatMap((F) => typeof F != "object" || !("PlayCard" in F) ? [] : q != null && F.PlayCard.card !== q ? [] : [F.PlayCard]);
  }
  function P(q) {
    return t.legalActions.flatMap((F) => typeof F != "object" || !("ActivateSkill" in F) ? [] : F.ActivateSkill.skill === q ? [F.ActivateSkill] : []);
  }
  function O() {
    return i.value ? P(i.value.id).filter((q) => a.value == null ? q.card_targets.length === 0 : q.card_targets.length === 1 && q.card_targets[0] === a.value).map((q) => q.targets) : a.value == null ? [] : A(a.value).map((q) => q.targets);
  }
  function G(q, F) {
    return q.length === F.length && q.every((J, oe) => J === F[oe]);
  }
  const N = X(() => O().some((q) => G(q, r.value))), ke = X(() => O().some((q) => q.length === 0));
  function Ee(q) {
    return t.view ? t.view.known_cards.filter((F) => F.owner === q && F.zone === "Equip" && F.def != null) : [];
  }
  function Se(q) {
    if ($.value.includes(q)) {
      if (a.value === q) {
        a.value = null, o.value = false, r.value = [];
        return;
      }
      a.value = q, i.value || (i.value = null), o.value = true, r.value = [];
    }
  }
  function Ye(q) {
    var _a2;
    if (((_a2 = i.value) == null ? void 0 : _a2.id) === q.id) {
      Oe();
      return;
    }
    i.value = q, a.value = null, o.value = true, r.value = [];
  }
  function He(q) {
    $.value.includes(q) && (l.value = q, a.value = q, o.value = true);
  }
  function Le(q) {
    if (!(!o.value || !j.value.has(q))) if (r.value.includes(q)) r.value = r.value.filter((F) => F !== q);
    else {
      const F = Math.max(0, ...O().map((J) => J.length));
      r.value = [...r.value, q].slice(0, F);
    }
  }
  function en(q, F) {
    l.value != null && j.value.has(F) && (q.preventDefault(), q.dataTransfer.dropEffect = "move", c.value = F);
  }
  function un(q) {
    c.value === q && (c.value = null);
  }
  function we(q, F) {
    var _a2;
    q.preventDefault();
    const J = l.value;
    if (c.value = null, l.value = null, J != null && j.value.has(F)) {
      if ((_a2 = i.value) == null ? void 0 : _a2.view_as) {
        const oe = P(i.value.id).find((Ue) => G(Ue.targets, [F]) && Ue.card_targets.length === 1 && Ue.card_targets[0] === J);
        if (!oe) return;
        s("activateSkill", oe.skill, oe.targets, oe.card_targets);
      } else {
        const oe = A(J).find((Ue) => G(Ue.targets, [F]));
        if (!oe) return;
        s("playCard", oe.card, oe.targets);
      }
      a.value = null, i.value = null, o.value = false, r.value = [];
    }
  }
  function re() {
    if (N.value) {
      if (i.value) {
        const q = P(i.value.id).find((F) => G(F.targets, r.value) && (a.value == null ? F.card_targets.length === 0 : F.card_targets.length === 1 && F.card_targets[0] === a.value));
        if (!q) return;
        s("activateSkill", q.skill, q.targets, q.card_targets);
      } else {
        if (a.value == null) return;
        const q = A(a.value).find((F) => G(F.targets, r.value));
        if (!q) return;
        s("playCard", q.card, q.targets);
      }
      a.value = null, i.value = null, o.value = false, r.value = [];
    }
  }
  function ie() {
    r.value = [], re();
  }
  function Oe() {
    a.value = null, i.value = null, o.value = false, r.value = [];
  }
  return (q, F) => {
    var _a2;
    return w(), x("div", Vy, [a.value != null || i.value ? (w(), x("div", Hy, [u("span", Wy, [F[0] || (F[0] = ae(" \u25B8 ", -1)), i.value ? (w(), x(te, { key: 0 }, [ae("\u6280\u80FD #" + b(i.value.def), 1)], 64)) : (w(), x(te, { key: 1 }, [ae("\u624B\u724C #" + b(a.value), 1)], 64)), ((_a2 = i.value) == null ? void 0 : _a2.view_as) && a.value != null ? (w(), x("span", By, " \xB7 \u7D20\u6750 #" + b(a.value), 1)) : V("", true), o.value && r.value.length > 0 ? (w(), x("span", Ky, " \u2192 \u76EE\u6807 P" + b(r.value[0]), 1)) : V("", true)]), u("div", Gy, [u("button", { class: "primary", type: "button", onClick: re, disabled: !N.value, "aria-label": "\u786E\u8BA4\u884C\u52A8" }, b(i.value ? "\u53D1\u52A8\u6280\u80FD" : "\u786E\u8BA4\u51FA\u724C"), 9, Jy), u("button", { type: "button", disabled: !ke.value, onClick: ie, "aria-label": "\u65E0\u76EE\u6807\u76F4\u63A5\u6253\u51FA" }, b(i.value ? "\u65E0\u76EE\u6807\u53D1\u52A8" : "\u76F4\u63A5\u6253\u51FA"), 9, Yy), u("button", { type: "button", onClick: Oe, "aria-label": "\u53D6\u6D88\u9009\u62E9" }, " \u53D6\u6D88 ")])])) : V("", true), m().length ? (w(), x("section", Qy, [F[1] || (F[1] = u("span", { class: "muted" }, "\u6280\u80FD", -1)), (w(true), x(te, null, ge(m(), (J) => {
      var _a3;
      return w(), x("button", { key: J.id, type: "button", class: Re({ primary: ((_a3 = i.value) == null ? void 0 : _a3.id) === J.id }), onClick: (oe) => Ye(J) }, " #" + b(J.def) + b(J.view_as ? " \xB7 \u89C6\u4E3A" : ""), 11, Xy);
    }), 128))])) : V("", true), u("div", Zy, [(w(true), x(te, null, ge(d(), (J) => (w(), x("div", { key: J.player, class: "seat-wrapper", onDragover: (oe) => en(oe, J.player), onDragleave: (oe) => un(J.player), onDrop: (oe) => we(oe, J.player) }, [ve(Cy, { seat: J, self: J.player === e.selfId, "identity-label": f(J.identity), "is-target-mode": o.value, "is-selectable-target": j.value.has(J.player), "is-drop-target": c.value === J.player, "equip-cards": Ee(J.player), onSelect: Le }, null, 8, ["seat", "self", "identity-label", "is-target-mode", "is-selectable-target", "is-drop-target", "equip-cards"])], 40, ek))), 128))]), ve(Fy, { cards: p(), "actionable-ids": $.value, onSelect: Se, onDragStart: He }, null, 8, ["cards", "actionable-ids"]), d().some((J) => W(J.player).length > 0) ? (w(), x("section", nk, [F[2] || (F[2] = u("h3", { class: "muted jz-title" }, "\u2696 \u5224\u5B9A\u533A", -1)), u("div", tk, [(w(true), x(te, null, ge(d(), (J) => (w(), x("div", { key: "jz-" + J.player, class: "jz-entry" }, [(w(true), x(te, null, ge(W(J.player), (oe) => (w(), x("span", { key: oe.id, class: "jz-card", title: `P${J.player} #${oe.id}` }, [u("span", ak, b(oe.suit === "Spade" ? "\u2660" : oe.suit === "Heart" ? "\u2665" : oe.suit === "Club" ? "\u2663" : oe.suit === "Diamond" ? "\u2666" : "?"), 1), u("span", ik, "P" + b(J.player), 1)], 8, sk))), 128))]))), 128))])])) : V("", true), T().length > 0 ? (w(), x("section", ok, [u("h3", rk, "\u5F03\u724C\u5806\uFF08" + b(T().length) + "\uFF09", 1), u("div", lk, [(w(true), x(te, null, ge(T().slice(-10), (J) => (w(), x("div", { key: J.id, class: "discard-card", title: `#${J.id} def:${J.def}` }, [u("span", ck, b(J.suit === "Spade" ? "\u2660" : J.suit === "Heart" ? "\u2665" : J.suit === "Club" ? "\u2663" : J.suit === "Diamond" ? "\u2666" : "?"), 1)], 8, uk))), 128)), T().length > 10 ? (w(), x("span", dk, "+" + b(T().length - 10), 1)) : V("", true)])])) : V("", true)]);
  };
} }), hk = ln(pk, [["__scopeId", "data-v-9182a2db"]]), fk = { class: "panel general-select", "aria-label": "\u9009\u5C06\u9762\u677F" }, mk = { class: "gs-head" }, gk = { class: "muted tiny" }, _k = ["disabled"], vk = { key: 0, class: "muted", role: "status" }, yk = { key: 1, class: "muted empty", role: "status" }, kk = { key: 2, class: "error", role: "alert" }, wk = { key: 3, class: "success", role: "status" }, bk = { class: "muted" }, xk = { class: "muted" }, $k = { key: 4, class: "gs-grid" }, jk = ["disabled", "aria-pressed", "aria-label", "onClick"], Sk = { class: "candidate-name" }, Ek = { class: "candidate-skills" }, Ak = { class: "candidate-def muted" }, Ck = De({ __name: "GeneralSelectPanel", props: { state: {}, busy: { type: Boolean } }, emits: ["choose", "reload"], setup(e, { emit: n }) {
  const t = e, s = n, a = X(() => t.busy || t.state.selectionStatus === "pending"), i = X(() => {
    var _a2;
    return ((_a2 = t.state.assignment) == null ? void 0 : _a2.name) ?? null;
  }), o = X(() => t.state.projection.skillNames.length > 0 ? t.state.projection.skillNames.join(" \xB7 ") : "\u89C6\u56FE\u672A\u66B4\u9732\u6280\u80FD\u540D"), r = X(() => {
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
  return (l, c) => (w(), x("section", fk, [u("div", mk, [u("div", null, [c[1] || (c[1] = u("h2", null, "\u9009\u5C06", -1)), u("p", gk, " NC05 \xB7 \u76EE\u6807\u5EA7\u4F4D P" + b(e.state.seat) + " \xB7 " + b(r.value) + " \xB7 \u70B9\u51FB\u5019\u9009\u5373\u6309\u540D\u88C5\u914D \uFF08\u539F\u751F\u6B63\u5F0F\u5165\u53E3\uFF0C\u975E debug\uFF09 ", 1)]), u("button", { type: "button", disabled: a.value || e.state.loadStatus === "loading", "aria-label": "\u5237\u65B0\u9009\u5C06\u5019\u9009", onClick: c[0] || (c[0] = (d) => s("reload")) }, " \u5237\u65B0\u5019\u9009 ", 8, _k)]), e.state.loadStatus === "loading" && e.state.candidates.length === 0 ? (w(), x("p", vk, " \u6B63\u5728\u8BFB\u53D6\u539F\u751F\u76EE\u5F55\u5019\u9009\u2026 ")) : e.state.loadStatus === "empty" ? (w(), x("p", yk, " \u5019\u9009\u4E3A\u7A7A\uFF1A\u5185\u6838\u76EE\u5F55\u6CA1\u6709\u8FD4\u56DE\u4EFB\u4F55\u6B66\u5C06\uFF08\u8FD9\u662F\u5408\u6CD5\u7684\u7A7A\u76EE\u5F55\uFF0C\u4E0D\u662F\u52A0\u8F7D\u5931\u8D25\uFF09\u3002 ")) : V("", true), e.state.error ? (w(), x("p", kk, b(e.state.error), 1)) : V("", true), e.state.selectionStatus === "accepted" && e.state.assignment ? (w(), x("p", wk, [ae(" \u2705 \u5DF2\u4E3A P" + b(e.state.assignment.seat) + " \u88C5\u914D ", 1), u("strong", null, b(e.state.assignment.name), 1), u("span", bk, " \xB7 state_hash " + b(e.state.assignment.state_hash.slice(0, 16)) + "\u2026 ", 1), c[2] || (c[2] = u("br", null, null, -1)), u("span", xk, "\u5237\u65B0\u540E\u89C6\u56FE\u6280\u80FD\uFF1A" + b(o.value), 1)])) : V("", true), e.state.candidates.length > 0 ? (w(), x("ul", $k, [(w(true), x(te, null, ge(e.state.candidates, (d) => (w(), x("li", { key: d.def }, [u("button", { type: "button", class: Re(["candidate", { selected: i.value === d.name }]), disabled: a.value, "aria-pressed": i.value === d.name, "aria-label": `\u9009\u62E9 ${d.name}\uFF08${d.skillLabel}\uFF09`, onClick: (p) => s("choose", d.name) }, [u("span", Sk, b(d.name), 1), u("span", Ek, b(d.skillLabel), 1), u("span", Ak, "#" + b(d.def), 1)], 10, jk)]))), 128))])) : V("", true)]));
} }), Rk = ln(Ck, [["__scopeId", "data-v-9fa84d6d"]]), zk = { class: "battle stack" }, Ik = { key: 0, class: "panel hotkey-overlay", role: "dialog", "aria-label": "\u5FEB\u6377\u952E" }, Tk = { class: "outcome-content" }, Pk = { class: "outcome-icon" }, Ok = { class: "outcome-label" }, Nk = { class: "muted" }, qk = { key: 2, class: "panel turn-bar", role: "status", "aria-live": "polite" }, Mk = { class: "turn-player" }, Dk = ["data-phase"], Lk = { class: "panel toolbar", "aria-label": "\u5BF9\u5C40\u63A7\u5236" }, Uk = { class: "row" }, Fk = ["disabled"], Vk = { key: 1, class: "online-badge" }, Hk = ["disabled"], Wk = ["disabled"], Bk = ["disabled"], Kk = ["disabled"], Gk = ["disabled"], Jk = ["disabled"], Yk = { class: "row muted controls" }, Qk = { class: "check" }, Xk = { class: "status", role: "status", "aria-live": "polite" }, Zk = { key: 0, class: "error", role: "alert" }, ew = { key: 4, class: "panel online-roster", "aria-label": "\u8054\u673A\u623F\u95F4\u5EA7\u4F4D" }, nw = { class: "row online-roster-head" }, tw = { key: 0 }, sw = { class: "row online-seats" }, aw = { key: 0 }, iw = { key: 5, class: "panel choice", "aria-label": "\u5F85\u9009\u62E9" }, ow = { key: 0, role: "timer" }, rw = { class: "muted" }, lw = { key: 1, class: "row" }, uw = ["disabled", "onClick"], cw = { key: 2, class: "muted" }, dw = { key: 6, class: "panel roster", "aria-label": "\u73A9\u5BB6\u6982\u51B5" }, pw = { class: "roster-grid" }, hw = { class: "roster-player" }, fw = { class: "roster-hand muted" }, mw = ["data-id"], gw = { key: 1, class: "roster-id muted" }, _w = { class: "panel battle-section" }, vw = { key: 0, class: "effect-overlay", "aria-hidden": "true" }, yw = { class: "effect-icon" }, kw = { class: "effect-text" }, ww = { key: 0, class: "effect-player" }, bw = { key: 7, class: "panel match-stats", "aria-label": "\u5BF9\u5C40\u7EDF\u8BA1" }, xw = { class: "stats-grid" }, $w = { class: "stat" }, jw = { class: "stat-value mono" }, Sw = { class: "stat" }, Ew = { class: "stat-value mono" }, Aw = { class: "stat" }, Cw = { class: "stat-value" }, Rw = { class: "stat" }, zw = { class: "stat-value" }, Iw = { key: 8, class: "panel seat-status", "aria-label": "\u5EA7\u4F4D\u72B6\u6001\u4E00\u89C8" }, Tw = { class: "seat-status-grid" }, Pw = { class: "ss-header" }, Ow = { class: "ss-player" }, Nw = { key: 0, class: "ss-tag" }, qw = { class: "ss-hp-bar" }, Mw = { class: "ss-hp-text" }, Dw = { class: "ss-icon" }, Lw = { class: "ss-meta muted" }, Uw = ["data-id"], Fw = { key: 1, class: "muted" }, Vw = { class: "panel log", "aria-label": "\u4E8B\u4EF6\u65E5\u5FD7" }, Hw = { class: "log-header" }, Ww = { class: "log-list" }, Bw = { class: "log-rev" }, Kw = { class: "log-text" }, Gw = { key: 0, class: "muted empty-log" }, Jw = De({ __name: "BattleView", setup(e) {
  var _a2;
  const n = cp(), t = cl(), s = Vv(), a = new ii();
  let i;
  const o = new Ov(), r = B(1);
  try {
    ((_a2 = globalThis.localStorage) == null ? void 0 : _a2.getItem("noname-next-skip-anims")) === "1" && (s.skipAnims = true);
  } catch {
  }
  if (n.query.skipAnims === "1" && (s.skipAnims = true), n.query.speed !== void 0) {
    const _ = Number(n.query.speed);
    Number.isFinite(_) && _ > 0 && (s.speed = _);
  }
  const l = B(Number(n.query.players ?? 4) || 4), c = B(Number(n.query.seed ?? 42) || 42), d = B(null), p = B(false), m = B("\u672A\u8FDE\u63A5"), f = B(null), k = B(0), T = setInterval(() => {
    k.value++;
  }, 250), W = X(() => {
    var _a3;
    return k.value, o.secondsUntilServerDeadline((_a3 = s.choice) == null ? void 0 : _a3.deadline_unix_ms);
  }), j = B(0), $ = B(false), A = X(() => n.query.mode === "online"), P = X(() => A.value && r.value === 0), O = B(Qv(r.value)), G = X(() => !A.value && s.connected && a.isNative());
  async function N(_) {
    s.busy = true, s.error = null;
    try {
      return await _();
    } catch (v) {
      s.error = v instanceof Error ? v.message : String(v), m.value = s.error;
      return;
    } finally {
      s.busy = false;
    }
  }
  function ke(_) {
    globalThis.__nativeStartMarks ?? (globalThis.__nativeStartMarks = {}), globalThis.__nativeStartMarks[_] = Math.round(performance.now() * 100) / 100;
  }
  async function Ee(_) {
    const v = performance.now();
    Lo(s, _);
    const E = performance.now();
    ke("t5_apply_envelope"), await Gv(s, 120);
    const S = performance.now();
    if (globalThis.__nativeEnvelopeTimings ?? (globalThis.__nativeEnvelopeTimings = []), globalThis.__nativeEnvelopeTimings.push({ apply_ms: Math.round((E - v) * 100) / 100, delay_ms: Math.round((S - E) * 100) / 100 }), A.value && (!s.connected || s.revision !== _.revision || s.hash !== _.hash_hex)) return;
    Bv(s);
    const D = A.value ? `\u623F\u95F4 #${j.value} \xB7 ` : "", Y = (_.hash_hex || "").slice(0, 12);
    m.value = _.finished ? `${D}\u7EC8\u5C40 \xB7 ${I.value.label} \xB7 rev ${_.revision}` : `${D}rev ${_.revision}${Y ? ` \xB7 ${Y}\u2026` : ""}${a.isNative() ? " \xB7 native" : ""}`, Ye();
  }
  async function Se(_) {
    if (await Ee(_), !A.value && a.isNative()) {
      const v = Uv(_.result);
      if (v) throw new Error(v);
    }
  }
  function Ye() {
    O.value = Il(O.value, zl(s.view, O.value.seat));
  }
  async function He() {
    if (!(A.value || !a.isNative())) {
      O.value = Xv(O.value);
      try {
        const _ = await a.generalCandidates();
        O.value = Zv(O.value, _);
      } catch (_) {
        O.value = ey(O.value, _);
      }
    }
  }
  async function Le(_) {
    if (A.value || !a.isNative() || s.busy || O.value.selectionStatus === "pending") return;
    const v = O.value;
    await N(async () => {
      if (O.value = await ty(v, _, { chooseGeneral: (E, S) => a.chooseGeneral(E, S), applyEnvelope: (E) => {
        Lo(s, E), Qe(s, `\u9009\u5C06 P${v.seat} \u2192 ${_}`);
      } }, (E) => {
        O.value = E;
      }), O.value.selectionStatus === "accepted") {
        const E = O.value.lastStateHash ?? "";
        m.value = `\u9009\u5C06 P${v.seat} \xB7 ${_}${E ? ` \xB7 ${E.slice(0, 12)}\u2026` : ""}`;
      } else O.value.selectionStatus === "rejected" && (m.value = O.value.error ?? "\u9009\u5C06\u5931\u8D25");
    });
  }
  async function en() {
    await N(async () => {
      try {
        if (await lo(true), Mo(true), A.value) {
          const S = q();
          hh(S.wsUrl), await re(S);
          return;
        }
        ph(), ke("t0_click"), await a.connect(), ke("t1_worker_ready"), s.connected = true, m.value = a.isNative() ? "\u539F\u751F Worker \u5C31\u7EEA\uFF0C\u5F00\u5C40\u4E2D\u2026" : "Worker \u5C31\u7EEA\uFF0C\u5F00\u5C40\u4E2D\u2026", a.isNative() && (r.value = Math.max(0, Math.min(r.value, l.value - 1)), O.value = Uo(O.value, r.value));
        const _ = Av();
        if (!a.isNative() && _.contentLock.length) throw new Error("\u5DF2\u5B89\u88C5\u6269\u5C55\u7684\u6B63\u5F0F\u5F00\u5C40\u9700\u8981\u539F\u751F\u540E\u7AEF");
        const v = await a.start(l.value, c.value, r.value, _.packsJson, n.query.mode === "identity" ? "identity" : void 0);
        i = _, ke("t2_start_done");
        const E = (a.isNative() ? He() : Promise.resolve()).then((S) => (ke("t3_candidates_loaded"), S));
        await Ee(v), await as(), ke("t4_first_frame"), await E, Qe(s, `\u5F00\u5C40 ${l.value} \u4EBA seed=${c.value}${_.contentLock.length ? ` \xB7 \u5185\u5BB9\u9501 ${_.contentLock.length} \u5305` : ""}`);
      } catch (_) {
        throw s.connected = false, y(), _;
      }
    });
  }
  async function un() {
    await N(async () => {
      if (A.value) throw new Error("\u8054\u673A\u6A21\u5F0F\u8BF7\u4F7F\u7528\u670D\u52A1\u7AEF\u5B58\u6863");
      if (!a.isNative()) throw new Error("\u5B58\u6863\u4EC5\u539F\u751F\u540E\u7AEF\u652F\u6301");
      if (!i) throw new Error("\u5C1A\u672A\u5EFA\u7ACB\u5BF9\u5C40\u5185\u5BB9\u9501");
      const _ = await a.saveHex(), v = `save-${Date.now()}`;
      Ev({ id: v, label: `rev${s.revision} \xB7 ${l.value}p seed=${c.value}`, hex: _, seed: c.value, players: l.value, ...i, savedAt: (/* @__PURE__ */ new Date()).toISOString(), revision: s.revision }), Qe(s, `\u5DF2\u5B58\u6863 ${v}\uFF08${_.length / 2} \u5B57\u8282\uFF09`), m.value = `\u5DF2\u5B58\u6863 \xB7 ${v}`;
    });
  }
  async function we() {
    await N(async () => {
      if (A.value) throw new Error("\u8054\u673A\u6A21\u5F0F\u4E0D\u652F\u6301\u672C\u5730\u8BFB\u6863");
      if (!a.isNative()) throw new Error("\u8BFB\u6863\u4EC5\u539F\u751F\u540E\u7AEF\u652F\u6301");
      const v = Sl()[0];
      if (!v) throw new Error("\u65E0\u672C\u5730\u5B58\u6863");
      const E = Cv(v), S = await a.restoreHex(v.hex, E.packsJson);
      i = E, l.value = v.players, c.value = v.seed, O.value = Uo(O.value, O.value.seat), await Ee(S), await He(), Qe(s, `\u8BFB\u6863 ${v.id} \xB7 rev ${v.revision}`);
    });
  }
  async function re(_) {
    o.onEnvelope((E) => {
      Ee(E);
    }), o.onClose(() => {
      m.value = "\u8FDE\u63A5\u5DF2\u65AD\u5F00\uFF08\u670D\u52A1\u7AEF\u4E0D\u53EF\u8FBE\uFF09", s.connected = false, Qe(s, "\u8FDE\u63A5\u5DF2\u65AD\u5F00\uFF1A\u670D\u52A1\u7AEF\u4E0D\u53EF\u8FBE\u6216\u5DF2\u505C\u6B62");
    }), o.onReconnect((E) => {
      E === "reconnecting" ? (m.value = "\u8FDE\u63A5\u5DF2\u65AD\u5F00\uFF0C\u6B63\u5728\u91CD\u8FDE\u2026", s.connected = false) : E === "reconnected" ? (m.value = "\u5DF2\u91CD\u8FDE\uFF08\u7B49\u5F85\u623F\u95F4\u72B6\u6001\uFF09", s.connected = false, o.rejoinRoom().then((S) => (r.value = S.playerId, j.value = S.roomId, Oe(_, S), s.connected = true, m.value = `\u623F\u95F4 #${S.roomId} \xB7 \u5DF2\u91CD\u8FDE`, Ee(S.envelope))).catch((S) => {
        s.connected = false;
        const D = S instanceof Error ? S.message : String(S);
        m.value = `\u91CD\u8FDE\u540E\u56DE\u623F\u5931\u8D25\uFF1A${D}`, Qe(s, `\u91CD\u8FDE\u540E\u56DE\u623F\u5931\u8D25\uFF1A${D}`);
      })) : (m.value = "\u91CD\u8FDE\u5931\u8D25\uFF08\u670D\u52A1\u7AEF\u4E0D\u53EF\u8FBE\uFF09", s.connected = false);
    }), o.onRoster((E) => {
      f.value = E, j.value = E.room_id;
      const S = E.started && !$.value;
      $.value = E.started, m.value = E.started ? `\u623F\u95F4 #${E.room_id} \xB7 \u5DF2\u5F00\u5C40` : `\u623F\u95F4 #${E.room_id} \xB7 \u7B49\u5F85\u5165\u5EA7 ${E.seats.length}/${l.value}`, S && o.requestSnapshot().catch(() => {
      });
    }), m.value = `\u8FDE\u63A5 ${_.wsUrl}\u2026`, await o.connect(_.wsUrl, _.displayName);
    let v;
    if (_.action === "create") v = await o.createRoom(_.players, _.seed, _.password, _.choiceTimeoutSecs ?? 30);
    else if (_.action === "spectate") v = await o.spectate(_.roomId);
    else {
      const E = localStorage.getItem(Do(_.wsUrl, _.roomId));
      v = await o.joinRoom(_.roomId, _.password, E);
    }
    j.value = v.roomId, r.value = v.playerId, l.value = v.view.seats.length, Oe(_, v), s.connected = true, await Ee(v.envelope), Qe(s, v.role === "Spectator" ? `\u65C1\u89C2\u623F\u95F4 #${v.roomId}` : `\u52A0\u5165\u623F\u95F4 #${v.roomId} \xB7 \u5EA7\u4F4D ${v.seat + 1}`);
  }
  function ie() {
    o.cancelPendingCommands(), m.value = "\u5DF2\u53D6\u6D88\u5F85\u786E\u8BA4\u547D\u4EE4\uFF08\u670D\u52A1\u7AEF\u53EF\u80FD\u5DF2\u63D0\u4EA4\uFF0C\u4EE5\u6743\u5A01\u72B6\u6001\u4E3A\u51C6\uFF09", Qe(s, "\u5DF2\u53D6\u6D88\u5F85\u786E\u8BA4\u547D\u4EE4\uFF1B\u7ED3\u679C\u53EF\u80FD\u5DF2\u63D0\u4EA4\uFF0C\u7B49\u5F85/\u5237\u65B0\u540E\u4EE5\u670D\u52A1\u7AEF\u72B6\u6001\u4E3A\u51C6");
  }
  function Oe(_, v) {
    v.role === "Player" && v.reconnectToken && localStorage.setItem(Do(_.wsUrl, v.roomId), v.reconnectToken), sessionStorage.setItem("noname-next.online-intent", JSON.stringify({ ..._, action: v.role === "Spectator" ? "spectate" : "join", roomId: v.roomId }));
  }
  function q() {
    const _ = sessionStorage.getItem("noname-next.online-intent");
    if (_) try {
      return JSON.parse(_);
    } catch {
      sessionStorage.removeItem("noname-next.online-intent");
    }
    const v = window.location.protocol === "https:";
    return { action: String(n.query.action ?? "join"), wsUrl: `${v ? "wss" : "ws"}://${window.location.hostname || "127.0.0.1"}:7700/ws`, displayName: "player", password: "", roomId: Number(n.query.room ?? 1) || 1, players: l.value, seed: c.value };
  }
  function F() {
    return A.value ? o : a;
  }
  async function J() {
    await N(async () => {
      const _ = await F().endPhase(r.value);
      await Se(_);
    });
  }
  async function oe() {
    await N(async () => {
      if (A.value) throw new Error("\u8054\u673A\u6A21\u5F0F\u4E0D\u63D0\u4F9B\u5BA2\u6237\u7AEF\u81EA\u52A8\u6B65\u8FDB");
      const v = await a.autoStep();
      return await Se(v), true;
    }) || K();
  }
  async function Ue(_, v) {
    await N(async () => {
      const E = await F().playCard(r.value, _, v);
      await Se(E), Qe(s, `\u51FA\u724C #${_}${v.length > 0 ? ` \u2192 P${v[0]}` : ""}`);
    });
  }
  async function Sn(_, v, E) {
    await N(async () => {
      const S = await F().activateSkill(r.value, _, v, E);
      await Se(S), Qe(s, `\u53D1\u52A8\u6280\u80FD #${_}${v.length > 0 ? ` \u2192 P${v[0]}` : ""}`);
    });
  }
  async function Fe(_) {
    await N(async () => {
      const v = await F().resolveChoice(r.value, _.ResolveChoice.choice_id, _.ResolveChoice.selections, _.ResolveChoice.cancelled);
      await Se(v);
    });
  }
  const z = X(() => s.legalActions.filter((_) => typeof _ == "object" && "ResolveChoice" in _)), H = X(() => s.connected && s.legalActions.includes("EndPhase"));
  function U() {
    if (p.value) {
      K();
      return;
    }
    p.value = true;
    const _ = async () => {
      if (!p.value || s.finished) {
        K();
        return;
      }
      if (await oe(), p.value && !s.finished) {
        const v = s.skipAnims || s.reduceMotion ? 30 : 280 / s.speed;
        d.value = window.setTimeout(_, v);
      } else K();
    };
    _();
  }
  function K() {
    p.value = false, d.value != null && (clearTimeout(d.value), d.value = null);
  }
  function se(_) {
    return _.includes("\u4F24\u5BB3") || _.includes("\u9635\u4EA1") ? "log-damage" : _.includes("\u56DE\u590D") || _.includes("\u6478\u724C") ? "log-positive" : _.includes("\u6FD2\u6B7B") || _.includes("\u7B49\u5F85") ? "log-warning" : _.includes("\u56DE\u5408") || _.includes("\u4F7F\u7528") ? "log-action" : "";
  }
  const h = B(false);
  function g(_) {
    if (_.key === "?") {
      h.value = !h.value;
      return;
    }
    if (_.key === "Escape") {
      h.value = false;
      return;
    }
    s.busy || s.finished || !s.connected || (_.key === " " ? (_.preventDefault(), A.value || oe()) : _.key === "e" || _.key === "E" ? H.value && J() : (_.key === "s" || _.key === "S") && (s.skipAnims = !s.skipAnims));
  }
  function y() {
    K(), a.dispose(), o.dispose(), i = void 0, lo(false), Mo(false);
  }
  $t(() => {
    window.addEventListener("keydown", g), window.addEventListener("pagehide", y), en().catch((_) => {
      const v = _ instanceof Error ? _.message : String(_);
      m.value = `\u8FDE\u63A5\u5931\u8D25\uFF1A${v}`, Qe(s, `\u8FDE\u63A5\u5931\u8D25\uFF1A${v}`), s.connected = false;
    });
  }), is(() => {
    clearInterval(T), window.removeEventListener("keydown", g), window.removeEventListener("pagehide", y), y();
  }), kn(() => s.speed, () => {
  });
  const R = { Prepare: "\u51C6\u5907", Draw: "\u6478\u724C", Play: "\u51FA\u724C", Discard: "\u5F03\u724C", Judge: "\u5224\u5B9A", End: "\u7ED3\u675F" }, I = X(() => Lv(s.view, s.lastEvents ?? [], r.value));
  function C() {
    t.push("/");
  }
  const M = X(() => {
    var _a3;
    if (s.view && s.view.phase !== "Idle" && (s.view.current_player !== 0 || s.view.seats.some((v) => v.player === 0))) return { currentPlayer: s.view.current_player, phase: s.view.phase, phaseLabel: R[s.view.phase] ?? s.view.phase, round: s.view.round };
    const _ = s.lastEvents ?? [];
    for (const v of _) {
      const E = v;
      if ((E == null ? void 0 : E.type) === "TurnStateChanged") return { currentPlayer: E.current ?? 0, phase: E.phase ?? "?", phaseLabel: R[E.phase ?? ""] ?? E.phase ?? "?", round: ((_a3 = s.view) == null ? void 0 : _a3.round) ?? 0 };
    }
    return null;
  });
  return (_, v) => {
    const E = Lu("auto-scroll");
    return w(), x("div", zk, [h.value ? (w(), x("section", Ik, [...v[4] || (v[4] = [Hr('<h2 data-v-14eda8d9>\u2328 \u5FEB\u6377\u952E</h2><dl class="hotkey-list" data-v-14eda8d9><dt data-v-14eda8d9><kbd data-v-14eda8d9>Space</kbd></dt><dd data-v-14eda8d9>\u81EA\u52A8\u4E00\u6B65</dd><dt data-v-14eda8d9><kbd data-v-14eda8d9>E</kbd></dt><dd data-v-14eda8d9>\u7ED3\u675F\u9636\u6BB5</dd><dt data-v-14eda8d9><kbd data-v-14eda8d9>S</kbd></dt><dd data-v-14eda8d9>\u5207\u6362\u8DF3\u8FC7\u52A8\u753B</dd><dt data-v-14eda8d9><kbd data-v-14eda8d9>?</kbd></dt><dd data-v-14eda8d9>\u663E\u793A/\u9690\u85CF\u6B64\u9762\u677F</dd><dt data-v-14eda8d9><kbd data-v-14eda8d9>Esc</kbd></dt><dd data-v-14eda8d9>\u5173\u95ED\u6B64\u9762\u677F</dd></dl>', 2)])])) : V("", true), L(s).finished && I.value ? (w(), x("section", { key: 1, class: Re(["panel outcome-banner", { win: I.value.isSelfWin === true, lose: I.value.isSelfWin === false }]), role: "alert", "aria-live": "assertive" }, [u("div", Tk, [u("span", Pk, b(I.value.isSelfWin === null ? "\u2691" : I.value.isSelfWin ? "\u{1F389}" : "\u{1F480}"), 1), u("div", null, [u("h2", Ok, b(I.value.label), 1), u("p", Nk, "rev " + b(L(s).revision), 1)])]), u("button", { class: "primary", type: "button", onClick: v[0] || (v[0] = (S) => A.value ? C() : en()) }, b(A.value ? "\u8FD4\u56DE\u5927\u5385" : "\u518D\u6765\u4E00\u5C40"), 1)], 2)) : V("", true), M.value ? (w(), x("section", qk, [u("span", Mk, "\u25BA \u7B2C " + b(M.value.round) + " \u8F6E \xB7 P" + b(M.value.currentPlayer), 1), u("span", { class: "phase-badge", "data-phase": M.value.phase }, b(M.value.phaseLabel), 9, Dk)])) : V("", true), u("section", Lk, [u("div", Uk, [A.value ? (w(), x("span", Vk, "\u8054\u673A\u623F\u95F4 #" + b(j.value || "\u2026"), 1)) : (w(), x("button", { key: 0, class: "primary", type: "button", disabled: L(s).busy, onClick: en }, " \u91CD\u65B0\u5F00\u5C40 ", 8, Fk)), u("button", { type: "button", disabled: L(s).busy || L(s).finished || P.value || A.value && !$.value || !H.value, onClick: J }, " \u7ED3\u675F\u9636\u6BB5 ", 8, Hk), A.value ? V("", true) : (w(), x("button", { key: 2, type: "button", disabled: L(s).busy || L(s).finished, onClick: oe }, " \u81EA\u52A8\u4E00\u6B65 ", 8, Wk)), A.value ? V("", true) : (w(), x("button", { key: 3, type: "button", disabled: L(s).busy, onClick: U }, b(p.value ? "\u505C\u6B62\u81EA\u52A8" : "\u8FDE\u7EED\u81EA\u52A8"), 9, Bk)), A.value ? (w(), x("button", { key: 4, type: "button", disabled: !L(s).busy, onClick: ie }, " \u53D6\u6D88\u5F85\u786E\u8BA4 ", 8, Kk)) : V("", true), A.value ? V("", true) : (w(), x("button", { key: 5, type: "button", disabled: L(s).busy || !L(s).connected, onClick: un }, " \u5B58\u6863 ", 8, Gk)), A.value ? V("", true) : (w(), x("button", { key: 6, type: "button", disabled: L(s).busy || !L(s).connected, onClick: we }, " \u8BFB\u6863 ", 8, Jk))]), u("div", Yk, [u("label", null, [v[6] || (v[6] = ae(" \u500D\u901F ", -1)), ye(u("select", { "onUpdate:modelValue": v[1] || (v[1] = (S) => L(s).speed = S), "aria-label": "\u64AD\u653E\u500D\u901F" }, [...v[5] || (v[5] = [u("option", { value: 0.5 }, "0.5\xD7", -1), u("option", { value: 1 }, "1\xD7", -1), u("option", { value: 2 }, "2\xD7", -1), u("option", { value: 4 }, "4\xD7", -1)])], 512), [[vn, L(s).speed, void 0, { number: true }]])]), u("label", Qk, [ye(u("input", { "onUpdate:modelValue": v[2] || (v[2] = (S) => L(s).skipAnims = S), type: "checkbox" }, null, 512), [[ks, L(s).skipAnims]]), v[7] || (v[7] = ae(" \u8DF3\u8FC7\u52A8\u753B ", -1))]), u("span", null, b(P.value ? "\u516C\u5171\u65C1\u89C2\u6295\u5F71" : `\u89C2\u5BDF\u8005 P${r.value}`), 1)]), u("p", Xk, b(m.value), 1), L(s).error ? (w(), x("p", Zk, b(L(s).error), 1)) : V("", true)]), G.value ? (w(), ti(Rk, { key: 3, state: O.value, busy: L(s).busy, onChoose: Le, onReload: He }, null, 8, ["state", "busy"])) : V("", true), A.value && f.value ? (w(), x("section", ew, [u("div", nw, [u("h2", null, "\u623F\u95F4 #" + b(f.value.room_id), 1), f.value.choice_timeout_secs ? (w(), x("span", tw, "\u9009\u62E9\u65F6\u9650 " + b(f.value.choice_timeout_secs) + " \u79D2", 1)) : V("", true), u("span", { class: Re(f.value.started ? "started" : "waiting") }, b(f.value.started ? "\u5BF9\u5C40\u8FDB\u884C\u4E2D" : "\u7B49\u5F85\u73A9\u5BB6"), 3)]), u("div", sw, [(w(true), x(te, null, ge(f.value.seats, (S) => (w(), x("span", { key: S.seat, class: "online-seat" }, [ae(b(S.seat + 1) + " \xB7 " + b(S.display_name) + " ", 1), S.managed ? (w(), x("strong", aw, "\u6258\u7BA1\u4E2D")) : V("", true), u("i", { class: Re(S.connected ? "connected" : "offline") }, b(S.connected ? "\u5728\u7EBF" : "\u79BB\u7EBF"), 3)]))), 128))])])) : V("", true), L(s).choice ? (w(), x("section", iw, [u("h2", null, b(L(qp)(L(s).choice)), 1), L(s).choice.deadline_unix_ms != null ? (w(), x("p", ow, b(W.value === null ? "\u6B63\u5728\u6821\u51C6\u670D\u52A1\u5668\u65F6\u95F4\u2026" : `\u5269\u4F59 ${W.value} \u79D2 \xB7 \u5230\u65F6\u6258\u7BA1`), 1)) : V("", true), u("p", rw, " choice #" + b(L(s).choice.choice_id) + " \xB7 \u73A9\u5BB6 " + b(L(s).choice.observer) + " \xB7 " + b(L(Ra)(L(s).choice)), 1), z.value.length ? (w(), x("div", lw, [(w(true), x(te, null, ge(z.value, (S, D) => (w(), x("button", { key: D, type: "button", disabled: L(s).busy || !L(s).connected || P.value, onClick: (Y) => Fe(S) }, b(S.ResolveChoice.cancelled ? "\u53D6\u6D88" : `\u9009\u62E9 ${S.ResolveChoice.selections.join(", ") || "\u7A7A"}`), 9, uw))), 128))])) : (w(), x("p", cw, "\u5F53\u524D\u89C2\u5BDF\u8005\u65E0\u6743\u5904\u7406\u6B64\u9009\u62E9\u3002"))])) : V("", true), L(s).view ? (w(), x("section", dw, [v[8] || (v[8] = u("h2", null, "\u73A9\u5BB6\u6982\u51B5", -1)), u("div", pw, [(w(true), x(te, null, ge(L(s).view.seats, (S) => (w(), x("div", { key: S.player, class: Re(["roster-entry", { dead: !S.alive, self: S.player === r.value }]) }, [u("span", hw, "P" + b(S.player) + b(S.turned_over ? " \xB7 \u7FFB\u9762" : ""), 1), u("span", { class: "roster-hp", style: Kn({ color: S.alive ? S.hp <= 0 ? "var(--color-warning)" : "inherit" : "var(--color-negative)" }) }, b(S.alive ? S.hp <= 0 ? "\u26A0" : "" : "\u2717") + " " + b(S.hp) + "/" + b(S.max_hp), 5), u("span", fw, b(S.hand_count) + "\u{1F0A0}", 1), S.identity ? (w(), x("span", { key: 0, class: "roster-id", "data-id": S.identity }, b(S.identity === "Lord" ? "\u4E3B" : S.identity === "Loyalist" ? "\u5FE0" : S.identity === "Rebel" ? "\u53CD" : "\u5978"), 9, mw)) : (w(), x("span", gw, "?"))], 2))), 128))])])) : V("", true), u("section", _w, [ve(hk, { view: L(s).view, "self-id": r.value, "legal-actions": L(s).connected && !L(s).busy ? L(s).legalActions : [], onPlayCard: Ue, onActivateSkill: Sn }, null, 8, ["view", "self-id", "legal-actions"]), L(s).effects.length ? (w(), x("div", vw, [(w(true), x(te, null, ge(L(s).effects, (S) => (w(), x("div", { key: S.id, class: Re(["effect-popup", S.type]) }, [u("span", yw, b(S.type === "damage" ? "\u2694" : S.type === "heal" ? "\u271A" : S.type === "draw" ? "\u{1F0A0}" : S.type === "play" ? "\u25B8" : S.type === "dying" ? "\u26A0" : "\u2717"), 1), u("span", kw, b(S.text), 1), S.player != null ? (w(), x("span", ww, "P" + b(S.player), 1)) : V("", true)], 2))), 128))])) : V("", true)]), L(s).view ? (w(), x("section", bw, [u("div", xw, [u("div", $w, [v[9] || (v[9] = u("span", { class: "stat-label muted" }, "Revision", -1)), u("span", jw, b(L(s).revision), 1)]), u("div", Sw, [v[10] || (v[10] = u("span", { class: "stat-label muted" }, "State Hash", -1)), u("span", Ew, b(L(s).hash.slice(0, 16)) + "\u2026", 1)]), u("div", Aw, [v[11] || (v[11] = u("span", { class: "stat-label muted" }, "\u5DF2\u77E5\u5361\u724C", -1)), u("span", Cw, b(L(s).view.known_cards.length), 1)]), u("div", Rw, [v[12] || (v[12] = u("span", { class: "stat-label muted" }, "\u5B58\u6D3B\u73A9\u5BB6", -1)), u("span", zw, b(L(s).view.seats.filter((S) => S.alive).length) + " / " + b(L(s).view.seats.length), 1)])])])) : V("", true), L(s).view ? (w(), x("section", Iw, [v[13] || (v[13] = u("h2", null, "\u5EA7\u4F4D\u72B6\u6001", -1)), u("div", Tw, [(w(true), x(te, null, ge(L(s).view.seats, (S) => (w(), x("div", { key: "ss-" + S.player, class: Re(["ss-entry", { "ss-dead": !S.alive, "ss-self": S.player === r.value }]) }, [u("div", Pw, [u("span", Ow, "P" + b(S.player), 1), S.player === r.value ? (w(), x("span", Nw, "\u4F60")) : V("", true)]), u("div", qw, [u("div", { class: "ss-hp-fill", style: Kn({ width: Math.max(Math.min(S.hp / S.max_hp * 100, 100), 0) + "%", background: S.alive ? S.hp <= 0 ? "var(--color-warning)" : "var(--color-positive)" : "var(--color-negative)" }) }, null, 4)]), u("div", Mw, [u("span", Dw, b(S.alive ? S.hp <= 0 ? "\u26A0" : "" : "\u2717"), 1), ae(" " + b(Math.max(S.hp, 0)) + "/" + b(S.max_hp), 1)]), u("div", Lw, [u("span", null, b(S.hand_count) + "\u{1F0A0}", 1), S.identity ? (w(), x("span", { key: 0, class: "ss-id", "data-id": S.identity }, b(S.identity === "Lord" ? "\u4E3B" : S.identity === "Loyalist" ? "\u5FE0" : S.identity === "Rebel" ? "\u53CD" : "\u5978"), 9, Uw)) : (w(), x("span", Fw, "?"))])], 2))), 128))])])) : V("", true), u("section", Vw, [u("div", Hw, [v[14] || (v[14] = u("h2", null, "\u6218\u6597\u65E5\u5FD7", -1)), L(s).log.length > 0 ? (w(), x("button", { key: 0, type: "button", class: "clear-btn", onClick: v[3] || (v[3] = (S) => L(s).log = []), "aria-label": "\u6E05\u7A7A\u65E5\u5FD7" }, "\u6E05\u7A7A")) : V("", true)]), v[15] || (v[15] = u("p", { class: "muted tiny" }, " \u65E5\u5FD7\u6765\u81EA DomainEvent \u6458\u8981\uFF0C\u4E0D\u662F\u72B6\u6001\u771F\u76F8\u6E90\u3002\u771F\u76F8\u6E90\u4E3A Command Log + \u6295\u5F71\uFF08\xA722\uFF09\u3002 ", -1)), ye((w(), x("ol", Ww, [(w(true), x(te, null, ge(L(s).log.slice().reverse(), (S) => (w(), x("li", { key: S.id, class: Re(["log-entry", se(S.text)]) }, [u("span", Bw, "rev" + b(S.at), 1), u("span", Kw, b(S.text), 1)], 2))), 128))])), [[E]]), L(s).log.length === 0 ? (w(), x("p", Gw, "\u6682\u65E0\u4E8B\u4EF6")) : V("", true)])]);
  };
} }), Yw = ln(Jw, [["__scopeId", "data-v-14eda8d9"]]), Qw = { class: "a11y-group" }, Xw = { class: "row" }, Zw = ["disabled"], eb = ["disabled"], nb = { class: "legend-list", "aria-live": "polite" }, tb = ["disabled", "onClick"], sb = ["disabled", "onClick"], ab = ["disabled", "onClick"], ib = { key: 0, class: "muted" }, ob = { class: "status", role: "status" }, Fo = "noname-next.native-server", rb = De({ __name: "NativePackagesPanel", setup(e) {
  const n = B(typeof localStorage < "u" && localStorage.getItem(Fo) || "http://127.0.0.1:7700"), t = B([]), s = B(false), a = B("");
  function i() {
    try {
      localStorage.setItem(Fo, n.value);
    } catch {
    }
  }
  async function o(d, p) {
    const m = await fetch(new URL(d, n.value), { ...p, headers: { "content-type": "application/json", ...(p == null ? void 0 : p.headers) ?? {} } });
    if (!m.ok) throw new Error(await m.text() || `HTTP ${m.status}`);
    return m;
  }
  async function r() {
    s.value = true;
    try {
      const d = await o("/native/packages");
      t.value = await d.json(), a.value = `\u5DF2\u8F7D\u5165 ${t.value.length} \u4E2A\u539F\u751F\u5305`;
    } catch (d) {
      a.value = `\u8BFB\u53D6\u5931\u8D25\uFF1A${d instanceof Error ? d.message : String(d)}`;
    } finally {
      s.value = false;
    }
  }
  async function l() {
    s.value = true;
    try {
      await o("/native/packages/import", { method: "POST", body: "{}" }), await r(), a.value = "\u5B89\u88C5\u626B\u63CF\u5B8C\u6210";
    } catch (d) {
      a.value = `\u5B89\u88C5\u5931\u8D25\uFF1A${d instanceof Error ? d.message : String(d)}`;
    } finally {
      s.value = false;
    }
  }
  async function c(d, p) {
    if (!(d === "uninstall" && !window.confirm(`\u5378\u8F7D ${p.package_id} \u7684\u6240\u6709\u7248\u672C\uFF1F`))) {
      s.value = true;
      try {
        await o(`/native/packages/${d}`, { method: "POST", body: JSON.stringify({ namespace: p.namespace, package_id: p.package_id, version: p.version }) }), await r(), a.value = `${d} \u5B8C\u6210\uFF1A${p.package_id}`;
      } catch (m) {
        a.value = `${d} \u5931\u8D25\uFF1A${m instanceof Error ? m.message : String(m)}`;
      } finally {
        s.value = false;
      }
    }
  }
  return $t(r), (d, p) => (w(), x("fieldset", Qw, [p[2] || (p[2] = u("legend", null, "\u539F\u751F\u6269\u5C55\u5305", -1)), p[3] || (p[3] = u("p", { class: "muted", role: "note" }, " \u901A\u8FC7\u670D\u52A1\u7AEF\u6B63\u5F0F\u5165\u53E3\u5B89\u88C5\u3001\u542F\u505C\u3001\u5347\u7EA7\u6216\u5378\u8F7D\u539F\u751F\u5305\uFF1B\u5BA2\u6237\u7AEF\u65E0\u9700\u91CD\u65B0\u6784\u5EFA\u3002 ", -1)), u("div", Xw, [u("label", null, [p[1] || (p[1] = ae(" \u670D\u52A1\u7AEF ", -1)), ye(u("input", { "onUpdate:modelValue": p[0] || (p[0] = (m) => n.value = m), type: "url", "aria-label": "\u539F\u751F\u670D\u52A1\u7AEF\u5730\u5740", onChange: i }, null, 544), [[Wn, n.value]])]), u("button", { type: "button", disabled: s.value, onClick: r }, "\u5237\u65B0", 8, Zw), u("button", { type: "button", disabled: s.value, onClick: l }, "\u626B\u63CF\u5B89\u88C5", 8, eb)]), u("ul", nb, [(w(true), x(te, null, ge(t.value, (m) => (w(), x("li", { key: `${m.namespace}/${m.package_id}@${m.version}` }, [u("code", null, b(m.namespace) + "/" + b(m.package_id) + "@" + b(m.version), 1), u("button", { type: "button", disabled: s.value, onClick: (f) => c("activate", m) }, "\u6FC0\u6D3B", 8, tb), u("button", { type: "button", disabled: s.value, onClick: (f) => c("disable", m) }, "\u505C\u7528", 8, sb), u("button", { type: "button", disabled: s.value, onClick: (f) => c("uninstall", m) }, "\u5378\u8F7D", 8, ab)]))), 128))]), t.value.length === 0 ? (w(), x("p", ib, "\u5C1A\u672A\u8BFB\u53D6\u5230\u5DF2\u5B89\u88C5\u5305\u3002")) : V("", true), u("p", ob, b(a.value), 1)]));
} }), lb = { class: "panel stack", "aria-labelledby": "settings-title" }, ub = { key: 0, class: "a11y-group" }, cb = { class: "select-row" }, db = { class: "a11y-group" }, pb = { class: "check" }, hb = { class: "check" }, fb = { class: "check" }, mb = { class: "select-row" }, gb = { class: "select-row" }, _b = { class: "select-row" }, Vo = "noname-next-a11y", vb = De({ __name: "SettingsView", setup(e) {
  const n = jn(), t = B(fl());
  kn(t, (m) => localStorage.setItem(hl, m));
  const s = B(typeof window < "u" && window.matchMedia("(prefers-reduced-motion: reduce)").matches), a = B(false), i = B(false), o = B("normal"), r = B("none"), l = B("default");
  function c() {
    if (typeof window > "u") return null;
    try {
      const m = localStorage.getItem(Vo);
      return m ? JSON.parse(m) : null;
    } catch {
      return null;
    }
  }
  function d() {
    if (typeof window > "u") return;
    const m = { highContrast: a.value, skipAnimations: i.value, fontScale: o.value, colorBlind: r.value, screenReader: l.value };
    localStorage.setItem(Vo, JSON.stringify(m));
  }
  function p() {
    const m = document.documentElement;
    m.dataset.contrast = a.value ? "high" : "", m.dataset.skipAnimations = i.value ? "true" : "false", m.dataset.fontScale = o.value, m.dataset.colorBlind = r.value !== "none" ? r.value : "";
  }
  return $t(() => {
    const m = c();
    m && (a.value = m.highContrast, i.value = m.skipAnimations, o.value = m.fontScale || "normal", r.value = m.colorBlind || "none", l.value = m.screenReader || "default"), p();
  }), kn([a, i, o, r, l], () => {
    p(), d();
  }), (m, f) => (w(), x("section", lb, [f[23] || (f[23] = u("h1", { id: "settings-title" }, "\u8BBE\u7F6E", -1)), f[24] || (f[24] = u("p", { class: "muted", role: "note" }, " \u8868\u73B0\u5C42\u8BBE\u7F6E\u4E0D\u5F71\u54CD\u89C4\u5219\u6838\u5FC3\uFF08\xA730 \u53EF\u8BBF\u95EE\u6027\uFF09\u3002\u6240\u6709\u8BBE\u7F6E\u5373\u65F6\u751F\u6548\u5E76\u4FDD\u5B58\u5230\u672C\u5730\u3002 ", -1)), L(n) ? (w(), x("fieldset", ub, [f[9] || (f[9] = u("legend", null, "\u684C\u9762\u5F15\u64CE", -1)), u("label", cb, [f[8] || (f[8] = u("span", null, "\u4E0B\u6B21\u8FDE\u63A5\u4F7F\u7528", -1)), ye(u("select", { "onUpdate:modelValue": f[0] || (f[0] = (k) => t.value = k), "aria-label": "\u684C\u9762\u5F15\u64CE" }, [...f[7] || (f[7] = [u("option", { value: "wasm" }, "WASM Worker\uFF08\u9ED8\u8BA4 / \u56DE\u9000\uFF09", -1), u("option", { value: "native" }, "\u539F\u751F\u5F15\u64CE\uFF08\u5B9E\u9A8C\u539F\u578B\uFF09", -1)])], 512), [[vn, t.value]])]), f[10] || (f[10] = u("p", { class: "muted" }, "\u5F53\u524D\u5BF9\u5C40\u4E0D\u5207\u6362\u3002\u64CD\u4F5C\u5931\u8D25\u65F6\u4FDD\u7559\u5F53\u524D\u5F15\u64CE\uFF0C\u8BF7\u5148\u6838\u5BF9\u5C40\u9762\u3002", -1))])) : V("", true), u("fieldset", db, [f[20] || (f[20] = u("legend", null, "\u53EF\u8BBF\u95EE\u6027\uFF08\xA730\uFF09", -1)), u("label", pb, [ye(u("input", { "onUpdate:modelValue": f[1] || (f[1] = (k) => s.value = k), type: "checkbox", disabled: "", "aria-describedby": "reduce-motion-desc" }, null, 512), [[ks, s.value]]), f[11] || (f[11] = u("span", null, "\u51CF\u5C11\u52A8\u6001\u6548\u679C\uFF08\u7CFB\u7EDF\u504F\u597D\uFF0C\u53EA\u8BFB\uFF09", -1))]), f[21] || (f[21] = u("p", { id: "reduce-motion-desc", class: "sr-only" }, " \u9075\u5FAA\u7CFB\u7EDF prefers-reduced-motion \u8BBE\u7F6E\u3002 ", -1)), u("label", hb, [ye(u("input", { "onUpdate:modelValue": f[2] || (f[2] = (k) => i.value = k), type: "checkbox", "aria-describedby": "skip-anim-desc" }, null, 512), [[ks, i.value]]), f[12] || (f[12] = u("span", null, "\u8DF3\u8FC7\u52A8\u753B\uFF08\u8DF3\u8FC7\u540E\u4ECD\u53EF\u7406\u89E3\u89C4\u5219\u7ED3\u679C\uFF09", -1))]), f[22] || (f[22] = u("p", { id: "skip-anim-desc", class: "sr-only" }, " \u8DF3\u8FC7\u6240\u6709\u52A8\u753B\uFF1B\u89C4\u5219\u7ED3\u679C\u4ECD\u901A\u8FC7\u6587\u5B57\u65E5\u5FD7\u548C\u56FE\u6807\u5B8C\u6574\u4F20\u8FBE\u3002 ", -1)), u("label", fb, [ye(u("input", { "onUpdate:modelValue": f[3] || (f[3] = (k) => a.value = k), type: "checkbox" }, null, 512), [[ks, a.value]]), f[13] || (f[13] = u("span", null, "\u9AD8\u5BF9\u6BD4\u5EA6\u6A21\u5F0F", -1))]), u("label", mb, [f[15] || (f[15] = u("span", null, "\u5B57\u53F7\u7F29\u653E", -1)), ye(u("select", { "onUpdate:modelValue": f[4] || (f[4] = (k) => o.value = k), "aria-label": "\u5B57\u53F7\u7F29\u653E" }, [...f[14] || (f[14] = [u("option", { value: "small" }, "\u5C0F (85%)", -1), u("option", { value: "normal" }, "\u6807\u51C6 (100%)", -1), u("option", { value: "large" }, "\u5927 (125%)", -1), u("option", { value: "xlarge" }, "\u7279\u5927 (150%)", -1)])], 512), [[vn, o.value]])]), u("label", gb, [f[17] || (f[17] = u("span", null, "\u8272\u89C9\u53CB\u597D\u6A21\u5F0F", -1)), ye(u("select", { "onUpdate:modelValue": f[5] || (f[5] = (k) => r.value = k), "aria-label": "\u8272\u89C9\u53CB\u597D\u6A21\u5F0F" }, [...f[16] || (f[16] = [Hr('<option value="none" data-v-637706ba>\u5173\u95ED</option><option value="protanopia" data-v-637706ba>\u7EA2\u8272\u76F2\uFF08Protanopia\uFF09</option><option value="deuteranopia" data-v-637706ba>\u7EFF\u8272\u76F2\uFF08Deuteranopia\uFF09</option><option value="tritanopia" data-v-637706ba>\u84DD\u8272\u76F2\uFF08Tritanopia\uFF09</option><option value="achromatopsia" data-v-637706ba>\u5168\u8272\u76F2\uFF08Achromatopsia\uFF09</option>', 5)])], 512), [[vn, r.value]])]), u("label", _b, [f[19] || (f[19] = u("span", null, "\u5C4F\u5E55\u9605\u8BFB\u5668\u8BE6\u7EC6\u5EA6", -1)), ye(u("select", { "onUpdate:modelValue": f[6] || (f[6] = (k) => l.value = k), "aria-label": "\u5C4F\u5E55\u9605\u8BFB\u5668\u8BE6\u7EC6\u5EA6" }, [...f[18] || (f[18] = [u("option", { value: "off" }, "\u5173\u95ED", -1), u("option", { value: "default" }, "\u6807\u51C6", -1), u("option", { value: "verbose" }, "\u8BE6\u7EC6\uFF08\u64AD\u62A5\u6240\u6709\u72B6\u6001\u53D8\u5316\uFF09", -1)])], 512), [[vn, l.value]])])]), f[25] || (f[25] = u("div", { class: "state-legend", role: "region", "aria-label": "\u72B6\u6001\u4FE1\u53F7\u56FE\u4F8B" }, [u("h2", null, "\u72B6\u6001\u4FE1\u53F7\u8BF4\u660E"), u("p", { class: "muted" }, " \u6240\u6709\u6E38\u620F\u72B6\u6001\u901A\u8FC7\u56FE\u6807 + \u6587\u5B57\u4F20\u8FBE\uFF0C\u4E0D\u53EA\u4F9D\u8D56\u989C\u8272\uFF08\xA730: \u4E0D\u53EA\u4F9D\u8D56\u989C\u8272\u8868\u8FBE\u72B6\u6001\uFF09\u3002 "), u("ul", { class: "legend-list" }, [u("li", null, [u("span", { class: "state-icon", style: { color: "var(--color-positive)" } }, "\u2713"), ae(" \u5B58\u6D3B")]), u("li", null, [u("span", { class: "state-icon", style: { color: "var(--color-warning)" } }, "\u26A0"), ae(" \u6FD2\u6B7B")]), u("li", null, [u("span", { class: "state-icon", style: { color: "var(--color-negative)" } }, "\u2717"), ae(" \u9635\u4EA1")]), u("li", null, [u("span", { class: "state-icon", style: { color: "var(--color-positive)" } }, "\u25B8"), ae(" \u53EF\u51FA\u724C")]), u("li", null, [u("span", { class: "state-icon", style: { color: "var(--color-negative)" } }, "\u2715"), ae(" \u4E0D\u53EF\u7528")]), u("li", null, [u("span", { class: "state-icon", style: { color: "var(--color-accent)" } }, "\u25BA"), ae(" \u5F53\u524D\u56DE\u5408")]), u("li", null, [u("span", { class: "state-icon", style: { color: "var(--color-warning)" } }, "?"), ae(" \u8BF7\u9009\u62E9")])])], -1)), ve(rb)]));
} }), yb = ln(vb, [["__scopeId", "data-v-637706ba"]]), kb = { class: "replay stack" }, wb = { key: 0, class: "muted" }, bb = { key: 1, class: "error", role: "alert" }, xb = { key: 0, class: "panel" }, $b = { class: "info-grid" }, jb = { class: "mono" }, Sb = { class: "mono" }, Eb = ["disabled"], Ab = { key: 1, class: "panel" }, Cb = { class: "row playback-controls" }, Rb = ["disabled"], zb = ["disabled"], Ib = { class: "step-display mono" }, Tb = ["disabled"], Pb = ["disabled"], Ob = ["aria-valuenow", "aria-valuemax"], Nb = { class: "info-grid frame-info" }, qb = { class: "mono" }, Mb = { key: 2, class: "panel" }, Db = { key: 0, class: "seat-grid" }, Lb = { key: 1, class: "muted" }, Ub = { key: 3, class: "panel privacy-notice", role: "note" }, Fb = De({ __name: "ReplayView", setup(e) {
  const n = new ii(), t = B(null), s = B(null), a = B(null), i = B(null), o = B(false), r = B("full");
  async function l(f) {
    o.value = true, i.value = null;
    try {
      const k = JSON.parse(f);
      await n.connect(), s.value = await n.loadReplay(f), t.value = k, a.value = await n.seekReplay(0, r.value);
    } catch (k) {
      t.value = null, s.value = null, a.value = null, i.value = k instanceof Error ? k.message : String(k);
    } finally {
      o.value = false;
    }
  }
  function c(f) {
    const k = new FileReader();
    k.onload = () => void l(k.result), k.onerror = () => {
      i.value = "\u6587\u4EF6\u8BFB\u53D6\u5931\u8D25";
    }, k.readAsText(f);
  }
  function d(f) {
    var _a2;
    const k = (_a2 = f.target.files) == null ? void 0 : _a2[0];
    k && c(k);
  }
  function p(f) {
    var _a2, _b2;
    f.preventDefault();
    const k = (_b2 = (_a2 = f.dataTransfer) == null ? void 0 : _a2.files) == null ? void 0 : _b2[0];
    k && c(k);
  }
  async function m(f) {
    if (!(!s.value || o.value)) {
      o.value = true, i.value = null;
      try {
        a.value = await n.seekReplay(Math.max(0, Math.min(f, s.value.total)), r.value);
      } catch (k) {
        i.value = k instanceof Error ? k.message : String(k);
      } finally {
        o.value = false;
      }
    }
  }
  return kn(r, () => {
    a.value && m(a.value.step);
  }), is(() => n.dispose()), (f, k) => {
    var _a2;
    return w(), x("div", kb, [k[22] || (k[22] = u("section", { class: "panel" }, [u("h1", null, "\u5F55\u50CF\u56DE\u653E"), u("p", { class: "muted" }, "\u5BFC\u5165 Command Log \u540E\uFF0C\u89C4\u5219\u5F15\u64CE\u4F1A\u771F\u5B9E\u91CD\u6F14\u5E76\u9010\u6B65\u6838\u5BF9\u72B6\u6001\u54C8\u5E0C\u3002")], -1)), u("section", { class: "panel drop-zone", onDrop: p, onDragover: k[0] || (k[0] = ja(() => {
    }, ["prevent"])), "aria-label": "\u5BFC\u5165\u5F55\u50CF\u6587\u4EF6" }, [k[6] || (k[6] = u("h2", null, "\u5BFC\u5165\u5F55\u50CF", -1)), k[7] || (k[7] = u("p", { class: "muted" }, "\u62D6\u62FD .json \u5F55\u50CF\u6587\u4EF6\u5230\u6B64\u5904\uFF0C\u6216\u70B9\u51FB\u9009\u62E9", -1)), u("input", { type: "file", accept: ".json,application/json", onChange: d, "aria-label": "\u9009\u62E9\u5F55\u50CF\u6587\u4EF6" }, null, 32), o.value ? (w(), x("p", wb, "\u6B63\u5728\u9A8C\u8BC1\u5E76\u91CD\u6F14\u2026")) : V("", true), i.value ? (w(), x("p", bb, b(i.value), 1)) : V("", true)], 32), t.value && s.value ? (w(), x("section", xb, [k[15] || (k[15] = u("h2", null, "\u5F55\u50CF\u4FE1\u606F", -1)), u("dl", $b, [k[9] || (k[9] = u("dt", null, "Manifest Hash", -1)), u("dd", jb, b(s.value.manifest_hash), 1), k[10] || (k[10] = u("dt", null, "\u5F15\u64CE\u6784\u5EFA", -1)), u("dd", Sb, b(((_a2 = t.value.header) == null ? void 0 : _a2.engine_build_id) ?? "\u2014"), 1), k[11] || (k[11] = u("dt", null, "\u603B\u6B65\u6570", -1)), u("dd", null, b(s.value.total), 1), k[12] || (k[12] = u("dt", null, "\u5B8C\u6574\u6027", -1)), k[13] || (k[13] = u("dd", { class: "verified" }, "\u2713 \u54C8\u5E0C\u94FE\u4E0E\u5168\u90E8\u72B6\u6001\u68C0\u67E5\u70B9\u5DF2\u9A8C\u8BC1", -1)), k[14] || (k[14] = u("dt", null, "\u9690\u79C1\u7B49\u7EA7", -1)), u("dd", null, [ye(u("select", { "onUpdate:modelValue": k[1] || (k[1] = (T) => r.value = T), disabled: o.value, "aria-label": "\u9690\u79C1\u7B49\u7EA7" }, [...k[8] || (k[8] = [u("option", { value: "full" }, "\u5B8C\u6574\uFF08\u88C1\u5224\u89C6\u89D2\uFF09", -1), u("option", { value: "player" }, "\u73A9\u5BB6 1 \u89C6\u89D2", -1), u("option", { value: "public" }, "\u516C\u5F00\u89C6\u89D2", -1)])], 8, Eb), [[vn, r.value]])])])])) : V("", true), a.value && s.value ? (w(), x("section", Ab, [k[19] || (k[19] = u("h2", null, "\u56DE\u653E\u63A7\u5236", -1)), u("div", Cb, [u("button", { type: "button", onClick: k[2] || (k[2] = (T) => m(0)), disabled: o.value || a.value.step === 0 }, "\u23EE \u5F00\u5934", 8, Rb), u("button", { type: "button", onClick: k[3] || (k[3] = (T) => m(a.value.step - 1)), disabled: o.value || a.value.step === 0 }, "\u25C0 \u540E\u9000", 8, zb), u("span", Ib, b(a.value.step) + " / " + b(a.value.total), 1), u("button", { type: "button", onClick: k[4] || (k[4] = (T) => m(a.value.step + 1)), disabled: o.value || a.value.step >= a.value.total }, "\u524D\u8FDB \u25B6", 8, Tb), u("button", { type: "button", onClick: k[5] || (k[5] = (T) => m(a.value.total)), disabled: o.value || a.value.step >= a.value.total }, "\u7ED3\u5C3E \u23ED", 8, Pb)]), u("div", { class: "progress-bar", role: "progressbar", "aria-valuenow": a.value.step, "aria-valuemax": a.value.total }, [u("div", { class: "progress-fill", style: Kn({ width: a.value.step / Math.max(a.value.total, 1) * 100 + "%" }) }, null, 4)], 8, Ob), u("dl", Nb, [k[16] || (k[16] = u("dt", null, "Revision", -1)), u("dd", null, b(a.value.revision), 1), k[17] || (k[17] = u("dt", null, "State Hash", -1)), u("dd", qb, b(a.value.hash_hex), 1), k[18] || (k[18] = u("dt", null, "\u6BD4\u8D5B\u72B6\u6001", -1)), u("dd", null, b(a.value.finished ? "\u5DF2\u7ED3\u675F" : "\u8FDB\u884C\u4E2D"), 1)])])) : V("", true), a.value ? (w(), x("section", Mb, [k[20] || (k[20] = u("h2", null, "\u5F53\u524D\u5C40\u9762", -1)), a.value.view.seats.length ? (w(), x("div", Db, [(w(true), x(te, null, ge(a.value.view.seats, (T) => (w(), x("article", { key: T.player, class: "seat" }, [u("strong", null, "\u5EA7\u4F4D " + b(T.seat) + " \xB7 \u73A9\u5BB6 " + b(T.player), 1), u("span", null, "\u4F53\u529B " + b(T.hp) + " / " + b(T.max_hp), 1), u("span", null, "\u624B\u724C " + b(T.hand_count) + " \xB7 " + b(T.alive ? "\u5B58\u6D3B" : "\u9635\u4EA1") + b(T.turned_over ? " \xB7 \u7FFB\u9762" : ""), 1), u("span", null, "\u8EAB\u4EFD " + b(T.identity ?? "\u672A\u77E5"), 1)]))), 128))])) : (w(), x("p", Lb, "\u6BD4\u8D5B\u5C1A\u672A\u5F00\u59CB\u3002"))])) : V("", true), t.value ? (w(), x("section", Ub, [...k[21] || (k[21] = [u("h2", null, "\u9690\u79C1\u5206\u7EA7\uFF08\xA722.5\uFF09", -1), u("p", { class: "muted tiny" }, "\u6BCF\u4E2A\u753B\u9762\u76F4\u63A5\u4ECE\u5BF9\u5E94\u7684\u88C1\u5224\u3001\u73A9\u5BB6\u6216\u516C\u5F00 Observer \u6295\u5F71\u6784\u5EFA\uFF0C\u4E0D\u4F1A\u5148\u751F\u6210\u5B8C\u6574\u89C6\u56FE\u518D\u5220\u5B57\u6BB5\u3002", -1)])])) : V("", true)]);
  };
} }), Vb = ln(Fb, [["__scopeId", "data-v-91fcaceb"]]);
function Tl(e, n) {
  return typeof n != "string" || !n || !n.split("/").every((t) => t && t !== "." && t !== ".." && !/[\\%?#:\u0000-\u001f]/.test(t)) ? "" : `${e}/${n.split("/").map(encodeURIComponent).join("/")}`;
}
function Hb(e, n) {
  return e.assets.filter((t) => !t || typeof t.path != "string" || typeof t.sha256 != "string" || t.kind !== "audio" || t.source_category !== "core" || t.path !== t.source_path || !/^[a-f0-9]{64}$/.test(t.sha256) || !Number.isSafeInteger(t.size) || t.size <= 0 || !Tl("", t.path) || !t.path.endsWith(".mp3") ? false : t.path === `audio/die/${n.name}.mp3` ? true : n.skills.some((s) => {
    const a = `audio/skill/${s}`;
    return t.path.startsWith(a) && /^(?:[1-9][0-9]*)?\.mp3$/.test(t.path.slice(a.length));
  }));
}
const Wb = { class: "stack" }, Bb = { class: "panel" }, Kb = { key: 0, class: "counts" }, Gb = { key: 1, class: "muted", role: "status" }, Jb = { key: 2, class: "error", role: "alert" }, Yb = { key: 0, class: "browser" }, Qb = { class: "panel roster" }, Xb = { class: "filters" }, Zb = ["value"], e0 = ["onClick"], n0 = ["src"], t0 = { key: 0, class: "muted limit" }, s0 = { key: 0, class: "panel detail" }, a0 = ["src", "alt"], i0 = ["title"], o0 = { class: "muted" }, r0 = { key: 0, class: "skills" }, l0 = ["title"], u0 = { key: 1, class: "muted" }, c0 = { key: 2, class: "error", role: "alert" }, d0 = { key: 3, class: "skills" }, p0 = ["src", "aria-label", "onError", "onLoadeddata"], h0 = { key: 0, class: "error", role: "alert" }, f0 = { key: 4, class: "muted" }, m0 = { key: 5, class: "skins" }, g0 = ["src", "alt"], _0 = { key: 6, class: "muted" }, v0 = { class: "muted" }, y0 = De({ __name: "CollectionView", setup(e) {
  const n = B(null), t = B({ assets: [] }), s = B(""), a = B(/* @__PURE__ */ new Set()), i = B(""), o = B(false), r = B(""), l = B(""), c = B(null), d = P(), p = X(() => {
    var _a2;
    return new Map((_a2 = n.value) == null ? void 0 : _a2.skills.map((O) => [O.id, O]));
  }), m = X(() => {
    var _a2;
    const O = r.value.trim().toLowerCase();
    return (((_a2 = n.value) == null ? void 0 : _a2.characters) ?? []).filter((G) => !l.value || G.pack === l.value).filter((G) => !O || G.name.toLowerCase().includes(O) || G.display_name.toLowerCase().includes(O)).slice(0, 80);
  }), f = X(() => {
    var _a2;
    return ((_a2 = c.value) == null ? void 0 : _a2.resolved_skills.map((O) => p.value.get(O)).filter(Boolean)) ?? [];
  }), k = X(() => {
    var _a2;
    return (((_a2 = n.value) == null ? void 0 : _a2.skins.static) ?? []).filter((O) => {
      var _a3;
      return O.character === ((_a3 = c.value) == null ? void 0 : _a3.name);
    });
  }), T = X(() => {
    var _a2;
    return (((_a2 = n.value) == null ? void 0 : _a2.skins.dynamic) ?? []).filter((O) => {
      var _a3;
      return O.character === ((_a3 = c.value) == null ? void 0 : _a3.name);
    });
  }), W = X(() => c.value ? Hb(t.value, c.value) : []), j = X(() => t.value.assets.filter((O) => (O == null ? void 0 : O.kind) === "audio").length);
  $t($);
  async function $() {
    o.value = true, i.value = "";
    try {
      const O = await fetch(`${d}/catalog.json`);
      if (!O.ok) throw new Error(`${O.status} ${O.statusText}`);
      n.value = await O.json(), c.value = n.value.characters[0] ?? null;
      try {
        const G = await fetch(`${d}/asset-manifest.json`);
        if (!G.ok) throw new Error(`${G.status} ${G.statusText}`);
        const N = await G.json();
        if (N.schema_version !== 2 || !Array.isArray(N.assets)) throw new Error("\u8D44\u6E90\u6E05\u5355\u65E0\u6548");
        t.value = N;
      } catch (G) {
        s.value = `\u65E0\u6CD5\u8BFB\u53D6\u8BED\u97F3\u8D44\u6E90\u6E05\u5355\uFF08${String(G)}\uFF09\u3002`;
      }
    } catch (O) {
      i.value = `\u65E0\u6CD5\u8BFB\u53D6\u539F\u4ED3\u5E93\u5185\u5BB9\u76EE\u5F55\uFF08${String(O)}\uFF09\u3002\u8BF7\u5148\u542F\u52A8 game-server\u3002`;
    } finally {
      o.value = false;
    }
  }
  function A(O) {
    return Tl(d, O);
  }
  function P() {
    const G = !window.location.hostname || window.location.hostname.endsWith("tauri.localhost") ? "127.0.0.1" : window.location.hostname;
    return `${window.location.protocol === "https:" ? "https:" : "http:"}//${G}:7700/original-content`;
  }
  return (O, G) => (w(), x("div", Wb, [u("section", Bb, [G[2] || (G[2] = u("h1", null, "\u539F\u4ED3\u5E93\u6B66\u5C06\u5E93", -1)), G[3] || (G[3] = u("p", { class: "muted" }, " \u6B66\u5C06\u3001\u6280\u80FD\u6E90\u7801\u3001\u9759\u6001\u76AE\u80A4\u548C\u52A8\u6001\u76AE\u80A4\u5747\u4ECE\u672C\u5730\u539F\u4ED3\u5E93\u76EE\u5F55\u8BFB\u53D6\uFF1B\u65E7 JavaScript \u4E0D\u4F1A\u5728\u5BA2\u6237\u7AEF\u6267\u884C\u3002 ", -1)), n.value ? (w(), x("p", Kb, b(n.value.counts.packs) + " \u5305 \xB7 " + b(n.value.counts.characters) + " \u6B66\u5C06 \xB7 " + b(n.value.counts.skills) + " \u6280\u80FD \xB7 " + b(n.value.counts.static_skins) + " \u9759\u6001\u76AE\u80A4 \xB7 " + b(n.value.counts.dynamic_skins) + " \u52A8\u6001\u76AE\u80A4 \xB7 " + b(j.value) + " \u97F3\u9891\u8D44\u6E90 ", 1)) : V("", true), o.value ? (w(), x("p", Gb, "\u6B63\u5728\u52A0\u8F7D\u76EE\u5F55\u2026")) : V("", true), i.value ? (w(), x("p", Jb, b(i.value), 1)) : V("", true)]), n.value ? (w(), x("section", Yb, [u("aside", Qb, [u("div", Xb, [ye(u("input", { "onUpdate:modelValue": G[0] || (G[0] = (N) => r.value = N), type: "search", placeholder: "\u641C\u7D22\u6B66\u5C06", "aria-label": "\u641C\u7D22\u6B66\u5C06" }, null, 512), [[Wn, r.value]]), ye(u("select", { "onUpdate:modelValue": G[1] || (G[1] = (N) => l.value = N), "aria-label": "\u6B66\u5C06\u5305" }, [G[4] || (G[4] = u("option", { value: "" }, "\u5168\u90E8\u6B66\u5C06\u5305", -1)), (w(true), x(te, null, ge(n.value.packages, (N) => (w(), x("option", { key: N, value: N }, b(N), 9, Zb))), 128))], 512), [[vn, l.value]])]), (w(true), x(te, null, ge(m.value, (N) => {
    var _a2;
    return w(), x("button", { key: N.id, type: "button", class: Re(["character-row", { selected: ((_a2 = c.value) == null ? void 0 : _a2.id) === N.id }]), onClick: (ke) => c.value = N }, [N.portrait ? (w(), x("img", { key: 0, src: A(N.portrait), alt: "", loading: "lazy" }, null, 8, n0)) : V("", true), u("span", null, [u("strong", null, b(N.display_name), 1), u("small", null, b(N.pack) + " \xB7 " + b(N.name), 1)])], 10, e0);
  }), 128)), m.value.length === 80 ? (w(), x("p", t0, "\u4EC5\u663E\u793A\u524D 80 \u9879\uFF0C\u8BF7\u7EE7\u7EED\u7B5B\u9009\u3002")) : V("", true)]), c.value ? (w(), x("article", s0, [u("header", null, [c.value.portrait ? (w(), x("img", { key: 0, src: A(c.value.portrait), alt: `${c.value.display_name}\u6B66\u5C06\u5934\u50CF` }, null, 8, a0)) : V("", true), u("div", null, [u("h2", null, b(c.value.display_name), 1), u("span", { class: Re(["character-status", c.value.migration_status]), title: c.value.migration_status === "ported" ? "\u5B8C\u6574\u6B66\u5C06\u5B9A\u4E49\u5DF2\u63A5\u5165\u786E\u5B9A\u6027\u5185\u5BB9\u5305" : "\u4EC5\u540C\u6B65\u76EE\u5F55\u3001\u6E90\u7801\u4E0E\u8D44\u6E90\uFF0C\u5C1A\u4E0D\u80FD\u8FDB\u5165\u786E\u5B9A\u6027\u5BF9\u5C40" }, b(c.value.migration_status === "ported" ? "\u6B66\u5C06\u5305\u5DF2\u63A5\u5165" : "\u5F85\u5B8C\u6574\u79FB\u690D"), 11, i0), u("p", o0, b(c.value.pack) + " \xB7 " + b(c.value.name) + " \xB7 " + b(c.value.group ?? "\u672A\u77E5\u52BF\u529B") + " \xB7 " + b(c.value.hp) + " \u4F53\u529B", 1)])]), G[5] || (G[5] = u("h3", null, "\u6280\u80FD", -1)), f.value.length ? (w(), x("ul", r0, [(w(true), x(te, null, ge(f.value, (N) => (w(), x("li", { key: N.id }, [u("strong", null, b(N.display_name), 1), u("span", { class: Re(N.migration_status), title: N.migration_status === "ported" ? "\u5DF2\u63A5\u5165\u786E\u5B9A\u6027\u5185\u5BB9\u5305\uFF1B\u662F\u5426\u4E0E\u539F\u6280\u80FD\u5B8C\u5168\u7B49\u4EF7\u4ECD\u4EE5 Golden \u4E3A\u51C6" : "\u4EC5\u4FDD\u7559\u539F\u6E90\u7801\u4E0E\u76EE\u5F55\uFF0C\u4E0D\u80FD\u8FDB\u5165\u786E\u5B9A\u6027\u5BF9\u5C40" }, b(N.migration_status === "ported" ? "VM \u5DF2\u63A5\u5165" : "\u5F85 DSL \u91CD\u5199"), 11, l0), u("p", null, b(N.description ?? N.name), 1)]))), 128))])) : (w(), x("p", u0, "\u6CA1\u6709\u53EF\u89E3\u6790\u7684\u6B66\u5C06\u5305\u6280\u80FD\u3002")), u("h3", null, "\u8BED\u97F3\u8BD5\u542C\uFF08" + b(W.value.length) + "\uFF09", 1), s.value ? (w(), x("p", c0, b(s.value), 1)) : V("", true), W.value.length ? (w(), x("ul", d0, [(w(true), x(te, null, ge(W.value, (N) => (w(), x("li", { key: N.path }, [u("label", null, [u("span", null, b(N.source_path.startsWith("audio/die/") ? "\u9635\u4EA1" : N.source_path.slice(12, -4)), 1), u("audio", { controls: "", preload: "none", src: A(`assets/${N.path}`), "aria-label": `${c.value.display_name} \xB7 ${N.source_path}`, onError: (ke) => a.value.add(N.path), onLoadeddata: (ke) => a.value.delete(N.path) }, null, 40, p0)]), a.value.has(N.path) ? (w(), x("div", h0, "\u8BED\u97F3\u52A0\u8F7D\u6216\u89E3\u7801\u5931\u8D25\uFF0C\u8BF7\u68C0\u67E5\u672C\u5730\u8D44\u6E90\u3002")) : V("", true)]))), 128))])) : (w(), x("p", f0, "\u5F53\u524D\u8D44\u6E90\u6E05\u5355\u672A\u63D0\u4F9B\u8BE5\u6B66\u5C06\u7684\u76F4\u63A5\u6280\u80FD\u6216\u9635\u4EA1\u8BED\u97F3\u3002")), u("h3", null, "\u9759\u6001\u76AE\u80A4\uFF08" + b(k.value.length) + "\uFF09", 1), k.value.length ? (w(), x("div", m0, [(w(true), x(te, null, ge(k.value, (N) => (w(), x("figure", { key: N.id }, [u("img", { src: A(N.asset), alt: `${c.value.display_name} \xB7 ${N.name}`, loading: "lazy" }, null, 8, g0), u("figcaption", null, b(N.name), 1)]))), 128))])) : (w(), x("p", _0, "\u539F\u4ED3\u5E93\u672A\u63D0\u4F9B\u9759\u6001\u76AE\u80A4\u3002")), u("h3", null, "\u52A8\u6001\u76AE\u80A4\uFF08" + b(T.value.length) + "\uFF09", 1), u("p", v0, b(T.value.map((N) => N.name).join("\u3001") || "\u539F\u4ED3\u5E93\u672A\u63D0\u4F9B\u52A8\u6001\u76AE\u80A4\u3002"), 1)])) : V("", true)])) : V("", true)]));
} }), k0 = ln(y0, [["__scopeId", "data-v-d9802947"]]), w0 = { class: "stack" }, b0 = { class: "panel" }, x0 = { class: "muted" }, $0 = { class: "row" }, j0 = { key: 0, class: "ok", role: "status" }, S0 = { key: 1, class: "error", role: "alert" }, E0 = { key: 2, class: "warn", role: "status" }, A0 = { class: "panel" }, C0 = { class: "row" }, R0 = ["disabled"], z0 = ["disabled"], I0 = { class: "panel" }, T0 = { class: "muted mono" }, P0 = { key: 0, class: "lock-list" }, O0 = { key: 1, class: "muted" }, N0 = { class: "panel" }, q0 = { class: "table", role: "table" }, M0 = { key: 0, class: "badge-unsigned", title: "\u672C\u5730\u5BFC\u5165\u7684\u5305\u672A\u7ECF\u7B7E\u540D\u6821\u9A8C" }, D0 = { class: "mono" }, L0 = { class: "mono" }, U0 = { class: "actions" }, F0 = ["disabled", "onClick"], V0 = ["disabled", "onClick"], H0 = ["disabled", "onClick"], W0 = ["disabled", "title", "onClick"], B0 = ["disabled", "onClick"], K0 = ["disabled", "onClick"], G0 = ["disabled", "onClick"], J0 = De({ __name: "ExtensionsView", setup(e) {
  const n = B(Is()), t = B(""), s = B(""), a = B(""), i = B(false);
  let o = null;
  is(() => {
    o == null ? void 0 : o.then((j) => j.dispose(), () => {
    });
  });
  function r() {
    return o ?? (o = (async () => {
      const j = new ii();
      return await j.connect(), j;
    })().catch((j) => {
      throw o = null, j;
    })), o;
  }
  async function l(j) {
    await (await r()).validatePackage(j);
  }
  function c() {
    n.value = Is(), t.value = "";
  }
  function d(j) {
    t.value = "", s.value = "";
    try {
      j(), c();
    } catch ($) {
      t.value = $ instanceof Error ? $.message : String($);
    }
  }
  async function p() {
    if (!i.value) {
      t.value = "", s.value = "", i.value = true;
      try {
        const j = a.value;
        await _v(j, l);
        const $ = JSON.parse(j).manifest;
        s.value = `\u5DF2\u5BFC\u5165 ${$.namespace}/${$.package_id} @ ${$.version}\uFF08\u672A\u7B7E\u540D\uFF09`, a.value = "", c();
      } catch (j) {
        t.value = j instanceof Error ? j.message : String(j);
      } finally {
        i.value = false;
      }
    }
  }
  async function m(j) {
    var _a2;
    const $ = j.target, A = (_a2 = $.files) == null ? void 0 : _a2[0];
    if (A) try {
      a.value = await A.text(), await p();
    } catch (P) {
      t.value = P instanceof Error ? P.message : String(P);
    } finally {
      $.value = "";
    }
  }
  function f(j) {
    return n.value.installed.find(($) => $.key === j);
  }
  function k(j) {
    const $ = f(j);
    return $ ? $.lifecycle === "active" ? "\u5DF2\u542F\u7528" : $.lifecycle === "disabled" ? "\u5DF2\u505C\u7528" : $.lifecycle === "revoked" ? "\u5DF2\u64A4\u9500" : "\u5DF2\u5B89\u88C5" : "\u672A\u5B89\u88C5";
  }
  const T = X(() => jv(n.value)), W = X(() => {
    const j = Gt.map((P) => {
      const O = ri(P), G = f(O);
      return { title: P.title, version: P.version, key: O, record: G, status: k(O), imported: false };
    }), $ = new Set(j.map((P) => P.key)), A = n.value.installed.filter((P) => !$.has(P.key)).map((P) => ({ title: `${P.packageId}\uFF08\u5BFC\u5165\uFF09`, version: P.version, key: P.key, record: P, status: k(P.key), imported: true }));
    return [...j, ...A];
  });
  return (j, $) => (w(), x("div", w0, [u("section", b0, [$[2] || ($[2] = u("h1", null, "\u6269\u5C55\u5305\u7BA1\u7406", -1)), u("p", x0, " \u5BF9\u9F50\u5BBF\u4E3B\u5951\u7EA6\uFF1A\u5B89\u88C5 / \u542F\u7528 / \u505C\u7528 / \u5347\u7EA7 / \u56DE\u9000 / \u5378\u8F7D / \u64A4\u9500\u3002\u5F53\u524D\u76EE\u5F55 " + b(L(Gt).length) + " \u4E2A G1 \u8F6C\u6362\u5305\u3002\u5F00\u5C40\u5185\u5BB9\u9501\u7531\u542F\u7528\u96C6\u6784\u6210\uFF1B\u5BF9\u5C40\u4E2D\u51BB\u7ED3\u53D8\u66F4\u3002 \u5B89\u88C5\u65F6\u4FDD\u5B58\u5B9E\u9645\u5305\u5185\u5BB9\uFF0C\u672C\u5730\u539F\u751F\u5BF9\u5C40\u76F4\u63A5\u88C5\u8F7D\uFF1B\u5B58\u6863\u4FDD\u7559\u539F\u5C40\u5305\u526F\u672C\u3002 ", 1), u("div", $0, [u("button", { class: "primary", type: "button", onClick: $[0] || ($[0] = (A) => {
    d(() => {
      L(Sv)(), s.value = `\u5DF2\u5B89\u88C5\u5E76\u542F\u7528\u5168\u90E8 ${L(Gt).length} \u5305`;
    });
  }) }, " \u5168\u90E8\u5B89\u88C5\u5E76\u542F\u7528 "), u("button", { type: "button", onClick: c }, "\u5237\u65B0")]), s.value ? (w(), x("p", j0, b(s.value), 1)) : V("", true), t.value ? (w(), x("p", S0, b(t.value), 1)) : V("", true), n.value.matchFrozen ? (w(), x("p", E0, "\u5BF9\u5C40\u8FDB\u884C\u4E2D \xB7 \u5185\u5BB9\u9501\u5DF2\u51BB\u7ED3")) : V("", true)]), u("section", A0, [$[4] || ($[4] = u("h2", null, "\u5BFC\u5165\u6269\u5C55\u5305", -1)), $[5] || ($[5] = u("p", { class: "muted" }, " \u6B63\u5F0F\u5BFC\u5165\u5165\u53E3\uFF1A\u7C98\u8D34\u6216\u9009\u62E9\u4F5C\u8005\u4EA7\u7269\uFF08\u5B8C\u6574 npack JSON\uFF0C\u53EF\u542B VM \u6A21\u5757\u4E0E\u5361\u724C/\u6B66\u5C06/\u724C\u5806\u8D44\u4EA7\uFF09\u3002 \u540C\u5305\u65B0\u7248\u672C\u6309\u5347\u7EA7\u5904\u7406\u5E76\u4FDD\u7559\u65E7\u7248\u672C\u526F\u672C\uFF1B\u540C\u7248\u672C\u5185\u5BB9\u51B2\u7A81\u3001\u5DF2\u64A4\u9500\u3001\u5BF9\u5C40\u51BB\u7ED3\u3001\u7ED3\u6784\u6821\u9A8C\u6216 Rust \u6DF1\u5C42\u6821\u9A8C\uFF08\u5305/\u6A21\u5757\u6307\u7EB9\u3001\u5B57\u8282\u7801\u3001\u5BBF\u4E3B ABI \u7A97\u53E3\uFF09\u5931\u8D25\u65F6\u62D2\u7EDD\u4E14\u4E0D\u6539\u5199\u5DF2\u5B89\u88C5\u5185\u5BB9\u3002 \u672C\u5730\u5BFC\u5165\u7684\u5305\u672A\u7ECF\u7B7E\u540D\u6821\u9A8C\uFF0C\u5217\u8868\u4E2D\u6807\u4E3A\u300C\u672A\u7B7E\u540D\u300D\u3002 ", -1)), ye(u("textarea", { "onUpdate:modelValue": $[1] || ($[1] = (A) => a.value = A), class: "import-box", rows: "4", spellcheck: "false", "aria-label": "\u6269\u5C55\u5305 JSON", placeholder: '{"manifest": {...}, "modules": [...], ...}' }, null, 512), [[Wn, a.value]]), u("div", C0, [u("button", { class: "primary", type: "button", disabled: !a.value.trim() || n.value.matchFrozen || i.value, onClick: p }, b(i.value ? "\u6821\u9A8C\u4E2D\u2026" : "\u5BFC\u5165"), 9, R0), u("label", null, [$[3] || ($[3] = ae(" \u9009\u62E9\u6587\u4EF6 ", -1)), u("input", { type: "file", accept: ".json,application/json", "aria-label": "\u9009\u62E9\u6269\u5C55\u5305\u6587\u4EF6", disabled: n.value.matchFrozen, onChange: m }, null, 40, z0)])])]), u("section", I0, [$[6] || ($[6] = u("h2", null, "\u5185\u5BB9\u9501", -1)), u("p", T0, b(T.value.hashSeed || "\uFF08\u7A7A\uFF09"), 1), T.value.packages.length ? (w(), x("ul", P0, [(w(true), x(te, null, ge(T.value.packages, (A) => (w(), x("li", { key: A.key }, b(A.key) + " @ " + b(A.version) + " \xB7 " + b(A.fingerprint), 1))), 128))])) : (w(), x("p", O0, "\u5C1A\u672A\u542F\u7528\u4EFB\u4F55\u6269\u5C55\u3002"))]), u("section", N0, [$[8] || ($[8] = u("h2", null, "\u5305\u5217\u8868", -1)), u("div", q0, [$[7] || ($[7] = u("div", { class: "thead", role: "row" }, [u("span", null, "\u540D\u79F0"), u("span", null, "\u6807\u8BC6"), u("span", null, "\u7248\u672C"), u("span", null, "\u72B6\u6001"), u("span", null, "\u64CD\u4F5C")], -1)), (w(true), x(te, null, ge(W.value, (A) => {
    var _a2, _b2, _c2;
    return w(), x("div", { key: A.key, class: "trow", role: "row" }, [u("span", null, [ae(b(A.title) + " ", 1), ((_a2 = A.record) == null ? void 0 : _a2.signature) === "unsigned" ? (w(), x("span", M0, "\u672A\u7B7E\u540D")) : V("", true)]), u("span", D0, b(A.key), 1), u("span", L0, b(((_b2 = A.record) == null ? void 0 : _b2.version) ?? A.version), 1), u("span", null, b(A.status), 1), u("span", U0, [u("button", { type: "button", disabled: A.imported || !!A.record && A.record.lifecycle !== "revoked", onClick: (P) => d(() => L(xl)(A.key)) }, " \u5B89\u88C5 ", 8, F0), u("button", { type: "button", disabled: !A.record || A.record.lifecycle === "active" || A.record.lifecycle === "revoked", onClick: (P) => d(() => L(jl)(A.key)) }, " \u542F\u7528 ", 8, V0), u("button", { type: "button", disabled: !A.record || A.record.lifecycle !== "active", onClick: (P) => d(() => L(kv)(A.key)) }, " \u505C\u7528 ", 8, H0), u("button", { type: "button", disabled: A.imported || !A.record || A.record.lifecycle === "revoked", title: A.imported ? "\u5BFC\u5165\u5305\u8BF7\u5BFC\u5165\u65B0\u7248\u672C\u5305\u4EE5\u5347\u7EA7" : "", onClick: (P) => d(() => L(wv)(A.key)) }, " \u5347\u7EA7 ", 8, W0), u("button", { type: "button", disabled: !A.record || (((_c2 = A.record.history) == null ? void 0 : _c2.length) ?? 0) < 2, onClick: (P) => {
      d(() => {
        const O = A.record.history.filter((G) => G !== A.record.version);
        O.length && L(bv)(A.key, O[O.length - 1]);
      });
    } }, " \u56DE\u9000 ", 8, B0), u("button", { type: "button", disabled: !A.record, onClick: (P) => d(() => L(xv)(A.key)) }, " \u5378\u8F7D ", 8, K0), u("button", { type: "button", disabled: !A.record || A.record.lifecycle === "revoked", onClick: (P) => d(() => L($v)(A.key)) }, " \u64A4\u9500 ", 8, G0)])]);
  }), 128))])])]));
} }), Y0 = ln(J0, [["__scopeId", "data-v-2bd63404"]]), Pl = up({ history: jn() ? Vd() : ol(), routes: [{ path: "/", name: "home", component: Pp }, { path: "/battle", name: "battle", component: Yw }, { path: "/extensions", name: "extensions", component: Y0 }, { path: "/settings", name: "settings", component: yb }, { path: "/replay", name: "replay", component: Vb }, { path: "/collection", name: "collection", component: k0 }], scrollBehavior: () => ({ top: 0 }) });
function Q0() {
  const e = sessionStorage.getItem("noname-next.online-intent");
  if (e) try {
    const n = JSON.parse(e);
    if (n.action === "spectate") return;
    Pl.push({ path: "/battle", query: { mode: "online", action: n.action === "create" ? "join" : n.action ?? "join", room: String(n.roomId ?? 1), players: String(n.players ?? 4), seed: String(n.seed ?? 42) } });
  } catch {
    sessionStorage.removeItem("noname-next.online-intent");
  }
}
const X0 = sd(mp).use(Pl);
X0.mount("#app");
Q0();
