var __defProp = Object.defineProperty;
var __defNormalProp = (obj, key, value) => key in obj ? __defProp(obj, key, { enumerable: true, configurable: true, writable: true, value }) : obj[key] = value;
var __publicField = (obj, key, value) => __defNormalProp(obj, typeof key !== "symbol" ? key + "" : key, value);
(function() {
  const t = document.createElement("link").relList;
  if (t && t.supports && t.supports("modulepreload")) return;
  for (const i of document.querySelectorAll('link[rel="modulepreload"]')) s(i);
  new MutationObserver((i) => {
    for (const r of i) if (r.type === "childList") for (const o of r.addedNodes) o.tagName === "LINK" && o.rel === "modulepreload" && s(o);
  }).observe(document, { childList: true, subtree: true });
  function n(i) {
    const r = {};
    return i.integrity && (r.integrity = i.integrity), i.referrerPolicy && (r.referrerPolicy = i.referrerPolicy), i.crossOrigin === "use-credentials" ? r.credentials = "include" : i.crossOrigin === "anonymous" ? r.credentials = "omit" : r.credentials = "same-origin", r;
  }
  function s(i) {
    if (i.ep) return;
    i.ep = true;
    const r = n(i);
    fetch(i.href, r);
  }
})();
/**
* @vue/shared v3.5.39
* (c) 2018-present Yuxi (Evan) You and Vue contributors
* @license MIT
**/
function ti(e) {
  const t = /* @__PURE__ */ Object.create(null);
  for (const n of e.split(",")) t[n] = 1;
  return (n) => n in t;
}
const me = {}, Xt = [], pt = () => {
}, kr = () => false, ss = (e) => e.charCodeAt(0) === 111 && e.charCodeAt(1) === 110 && (e.charCodeAt(2) > 122 || e.charCodeAt(2) < 97), is = (e) => e.startsWith("onUpdate:"), Oe = Object.assign, ni = (e, t) => {
  const n = e.indexOf(t);
  n > -1 && e.splice(n, 1);
}, Qo = Object.prototype.hasOwnProperty, de = (e, t) => Qo.call(e, t), Q = Array.isArray, Zt = (e) => Mn(e) === "[object Map]", an = (e) => Mn(e) === "[object Set]", Ei = (e) => Mn(e) === "[object Date]", ee = (e) => typeof e == "function", ke = (e) => typeof e == "string", ht = (e) => typeof e == "symbol", pe = (e) => e !== null && typeof e == "object", Er = (e) => (pe(e) || ee(e)) && ee(e.then) && ee(e.catch), Cr = Object.prototype.toString, Mn = (e) => Cr.call(e), Yo = (e) => Mn(e).slice(8, -1), xr = (e) => Mn(e) === "[object Object]", si = (e) => ke(e) && e !== "NaN" && e[0] !== "-" && "" + parseInt(e, 10) === e, bn = ti(",key,ref,ref_for,ref_key,onVnodeBeforeMount,onVnodeMounted,onVnodeBeforeUpdate,onVnodeUpdated,onVnodeBeforeUnmount,onVnodeUnmounted"), rs = (e) => {
  const t = /* @__PURE__ */ Object.create(null);
  return ((n) => t[n] || (t[n] = e(n)));
}, Xo = /-\w/g, Ge = rs((e) => e.replace(Xo, (t) => t.slice(1).toUpperCase())), Zo = /\B([A-Z])/g, Mt = rs((e) => e.replace(Zo, "-$1").toLowerCase()), ii = rs((e) => e.charAt(0).toUpperCase() + e.slice(1)), ys = rs((e) => e ? `on${ii(e)}` : ""), dt = (e, t) => !Object.is(e, t), Wn = (e, ...t) => {
  for (let n = 0; n < e.length; n++) e[n](...t);
}, Ar = (e, t, n, s = false) => {
  Object.defineProperty(e, t, { configurable: true, enumerable: false, writable: s, value: n });
}, os = (e) => {
  const t = parseFloat(e);
  return isNaN(t) ? e : t;
};
let Ci;
const ls = () => Ci || (Ci = typeof globalThis < "u" ? globalThis : typeof self < "u" ? self : typeof window < "u" ? window : typeof global < "u" ? global : {});
function Tt(e) {
  if (Q(e)) {
    const t = {};
    for (let n = 0; n < e.length; n++) {
      const s = e[n], i = ke(s) ? sl(s) : Tt(s);
      if (i) for (const r in i) t[r] = i[r];
    }
    return t;
  } else if (ke(e) || pe(e)) return e;
}
const el = /;(?![^(]*\))/g, tl = /:([^]+)/, nl = /\/\*[^]*?\*\//g;
function sl(e) {
  const t = {};
  return e.replace(nl, "").split(el).forEach((n) => {
    if (n) {
      const s = n.split(tl);
      s.length > 1 && (t[s[0].trim()] = s[1].trim());
    }
  }), t;
}
function $e(e) {
  let t = "";
  if (ke(e)) t = e;
  else if (Q(e)) for (let n = 0; n < e.length; n++) {
    const s = $e(e[n]);
    s && (t += s + " ");
  }
  else if (pe(e)) for (const n in e) e[n] && (t += n + " ");
  return t.trim();
}
const il = "itemscope,allowfullscreen,formnovalidate,ismap,nomodule,novalidate,readonly", rl = ti(il);
function Rr(e) {
  return !!e || e === "";
}
function ol(e, t) {
  if (e.length !== t.length) return false;
  let n = true;
  for (let s = 0; n && s < e.length; s++) n = cn(e[s], t[s]);
  return n;
}
function cn(e, t) {
  if (e === t) return true;
  let n = Ei(e), s = Ei(t);
  if (n || s) return n && s ? e.getTime() === t.getTime() : false;
  if (n = ht(e), s = ht(t), n || s) return e === t;
  if (n = Q(e), s = Q(t), n || s) return n && s ? ol(e, t) : false;
  if (n = pe(e), s = pe(t), n || s) {
    if (!n || !s) return false;
    const i = Object.keys(e).length, r = Object.keys(t).length;
    if (i !== r) return false;
    for (const o in e) {
      const l = e.hasOwnProperty(o), a = t.hasOwnProperty(o);
      if (l && !a || !l && a || !cn(e[o], t[o])) return false;
    }
  }
  return String(e) === String(t);
}
function ri(e, t) {
  return e.findIndex((n) => cn(n, t));
}
const Pr = (e) => !!(e && e.__v_isRef === true), S = (e) => ke(e) ? e : e == null ? "" : Q(e) || pe(e) && (e.toString === Cr || !ee(e.toString)) ? Pr(e) ? S(e.value) : JSON.stringify(e, Or, 2) : String(e), Or = (e, t) => Pr(t) ? Or(e, t.value) : Zt(t) ? { [`Map(${t.size})`]: [...t.entries()].reduce((n, [s, i], r) => (n[_s(s, r) + " =>"] = i, n), {}) } : an(t) ? { [`Set(${t.size})`]: [...t.values()].map((n) => _s(n)) } : ht(t) ? _s(t) : pe(t) && !Q(t) && !xr(t) ? String(t) : t, _s = (e, t = "") => {
  var n;
  return ht(e) ? `Symbol(${(n = e.description) != null ? n : t})` : e;
};
/**
* @vue/reactivity v3.5.39
* (c) 2018-present Yuxi (Evan) You and Vue contributors
* @license MIT
**/
let Pe;
class ll {
  constructor(t = false) {
    this.detached = t, this._active = true, this._on = 0, this.effects = [], this.cleanups = [], this._isPaused = false, this._warnOnRun = true, this.__v_skip = true, !t && Pe && (Pe.active ? (this.parent = Pe, this.index = (Pe.scopes || (Pe.scopes = [])).push(this) - 1) : (this._active = false, this._warnOnRun = false));
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
      const n = Pe;
      try {
        return Pe = this, t();
      } finally {
        Pe = n;
      }
    }
  }
  on() {
    ++this._on === 1 && (this.prevScope = Pe, Pe = this);
  }
  off() {
    if (this._on > 0 && --this._on === 0) {
      if (Pe === this) Pe = this.prevScope;
      else {
        let t = Pe;
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
        const i = this.parent.scopes.pop();
        i && i !== this && (this.parent.scopes[this.index] = i, i.index = this.index);
      }
      this.parent = void 0;
    }
  }
}
function al() {
  return Pe;
}
let _e;
const bs = /* @__PURE__ */ new WeakSet();
class $r {
  constructor(t) {
    this.fn = t, this.deps = void 0, this.depsTail = void 0, this.flags = 5, this.next = void 0, this.cleanup = void 0, this.scheduler = void 0, Pe && (Pe.active ? Pe.effects.push(this) : this.flags &= -2);
  }
  pause() {
    this.flags |= 64;
  }
  resume() {
    this.flags & 64 && (this.flags &= -65, bs.has(this) && (bs.delete(this), this.trigger()));
  }
  notify() {
    this.flags & 2 && !(this.flags & 32) || this.flags & 8 || Tr(this);
  }
  run() {
    if (!(this.flags & 1)) return this.fn();
    this.flags |= 2, xi(this), Nr(this);
    const t = _e, n = Ye;
    _e = this, Ye = true;
    try {
      return this.fn();
    } finally {
      Mr(this), _e = t, Ye = n, this.flags &= -3;
    }
  }
  stop() {
    if (this.flags & 1) {
      for (let t = this.deps; t; t = t.nextDep) ai(t);
      this.deps = this.depsTail = void 0, xi(this), this.onStop && this.onStop(), this.flags &= -2;
    }
  }
  trigger() {
    this.flags & 64 ? bs.add(this) : this.scheduler ? this.scheduler() : this.runIfDirty();
  }
  runIfDirty() {
    Ns(this) && this.run();
  }
  get dirty() {
    return Ns(this);
  }
}
let Ir = 0, wn, Sn;
function Tr(e, t = false) {
  if (e.flags |= 8, t) {
    e.next = Sn, Sn = e;
    return;
  }
  e.next = wn, wn = e;
}
function oi() {
  Ir++;
}
function li() {
  if (--Ir > 0) return;
  if (Sn) {
    let t = Sn;
    for (Sn = void 0; t; ) {
      const n = t.next;
      t.next = void 0, t.flags &= -9, t = n;
    }
  }
  let e;
  for (; wn; ) {
    let t = wn;
    for (wn = void 0; t; ) {
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
function Nr(e) {
  for (let t = e.deps; t; t = t.nextDep) t.version = -1, t.prevActiveLink = t.dep.activeLink, t.dep.activeLink = t;
}
function Mr(e) {
  let t, n = e.depsTail, s = n;
  for (; s; ) {
    const i = s.prevDep;
    s.version === -1 ? (s === n && (n = i), ai(s), cl(s)) : t = s, s.dep.activeLink = s.prevActiveLink, s.prevActiveLink = void 0, s = i;
  }
  e.deps = t, e.depsTail = n;
}
function Ns(e) {
  for (let t = e.deps; t; t = t.nextDep) if (t.dep.version !== t.version || t.dep.computed && (Dr(t.dep.computed) || t.dep.version !== t.version)) return true;
  return !!e._dirty;
}
function Dr(e) {
  if (e.flags & 4 && !(e.flags & 16) || (e.flags &= -17, e.globalVersion === Rn) || (e.globalVersion = Rn, !e.isSSR && e.flags & 128 && (!e.deps && !e._dirty || !Ns(e)))) return;
  e.flags |= 2;
  const t = e.dep, n = _e, s = Ye;
  _e = e, Ye = true;
  try {
    Nr(e);
    const i = e.fn(e._value);
    (t.version === 0 || dt(i, e._value)) && (e.flags |= 128, e._value = i, t.version++);
  } catch (i) {
    throw t.version++, i;
  } finally {
    _e = n, Ye = s, Mr(e), e.flags &= -3;
  }
}
function ai(e, t = false) {
  const { dep: n, prevSub: s, nextSub: i } = e;
  if (s && (s.nextSub = i, e.prevSub = void 0), i && (i.prevSub = s, e.nextSub = void 0), n.subs === e && (n.subs = s, !s && n.computed)) {
    n.computed.flags &= -5;
    for (let r = n.computed.deps; r; r = r.nextDep) ai(r, true);
  }
  !t && !--n.sc && n.map && n.map.delete(n.key);
}
function cl(e) {
  const { prevDep: t, nextDep: n } = e;
  t && (t.nextDep = n, e.prevDep = void 0), n && (n.prevDep = t, e.nextDep = void 0);
}
let Ye = true;
const Lr = [];
function vt() {
  Lr.push(Ye), Ye = false;
}
function gt() {
  const e = Lr.pop();
  Ye = e === void 0 ? true : e;
}
function xi(e) {
  const { cleanup: t } = e;
  if (e.cleanup = void 0, t) {
    const n = _e;
    _e = void 0;
    try {
      t();
    } finally {
      _e = n;
    }
  }
}
let Rn = 0;
class ul {
  constructor(t, n) {
    this.sub = t, this.dep = n, this.version = n.version, this.nextDep = this.prevDep = this.nextSub = this.prevSub = this.prevActiveLink = void 0;
  }
}
class ci {
  constructor(t) {
    this.computed = t, this.version = 0, this.activeLink = void 0, this.subs = void 0, this.map = void 0, this.key = void 0, this.sc = 0, this.__v_skip = true;
  }
  track(t) {
    if (!_e || !Ye || _e === this.computed) return;
    let n = this.activeLink;
    if (n === void 0 || n.sub !== _e) n = this.activeLink = new ul(_e, this), _e.deps ? (n.prevDep = _e.depsTail, _e.depsTail.nextDep = n, _e.depsTail = n) : _e.deps = _e.depsTail = n, jr(n);
    else if (n.version === -1 && (n.version = this.version, n.nextDep)) {
      const s = n.nextDep;
      s.prevDep = n.prevDep, n.prevDep && (n.prevDep.nextDep = s), n.prevDep = _e.depsTail, n.nextDep = void 0, _e.depsTail.nextDep = n, _e.depsTail = n, _e.deps === n && (_e.deps = s);
    }
    return n;
  }
  trigger(t) {
    this.version++, Rn++, this.notify(t);
  }
  notify(t) {
    oi();
    try {
      for (let n = this.subs; n; n = n.prevSub) n.sub.notify() && n.sub.dep.notify();
    } finally {
      li();
    }
  }
}
function jr(e) {
  if (e.dep.sc++, e.sub.flags & 4) {
    const t = e.dep.computed;
    if (t && !e.dep.subs) {
      t.flags |= 20;
      for (let s = t.deps; s; s = s.nextDep) jr(s);
    }
    const n = e.dep.subs;
    n !== e && (e.prevSub = n, n && (n.nextSub = e)), e.dep.subs = e;
  }
}
const Ms = /* @__PURE__ */ new WeakMap(), Bt = Symbol(""), Ds = Symbol(""), Pn = Symbol("");
function Ie(e, t, n) {
  if (Ye && _e) {
    let s = Ms.get(e);
    s || Ms.set(e, s = /* @__PURE__ */ new Map());
    let i = s.get(n);
    i || (s.set(n, i = new ci()), i.map = s, i.key = n), i.track();
  }
}
function wt(e, t, n, s, i, r) {
  const o = Ms.get(e);
  if (!o) {
    Rn++;
    return;
  }
  const l = (a) => {
    a && a.trigger();
  };
  if (oi(), t === "clear") o.forEach(l);
  else {
    const a = Q(e), p = a && si(n);
    if (a && n === "length") {
      const u = Number(s);
      o.forEach((c, v) => {
        (v === "length" || v === Pn || !ht(v) && v >= u) && l(c);
      });
    } else switch ((n !== void 0 || o.has(void 0)) && l(o.get(n)), p && l(o.get(Pn)), t) {
      case "add":
        a ? p && l(o.get("length")) : (l(o.get(Bt)), Zt(e) && l(o.get(Ds)));
        break;
      case "delete":
        a || (l(o.get(Bt)), Zt(e) && l(o.get(Ds)));
        break;
      case "set":
        Zt(e) && l(o.get(Bt));
        break;
    }
  }
  li();
}
function Gt(e) {
  const t = fe(e);
  return t === e ? t : (Ie(t, "iterate", Pn), Je(e) ? t : t.map(Ze));
}
function as(e) {
  return Ie(e = fe(e), "iterate", Pn), e;
}
function ut(e, t) {
  return Ct(e) ? nn(qt(e) ? Ze(t) : t) : Ze(t);
}
const fl = { __proto__: null, [Symbol.iterator]() {
  return ws(this, Symbol.iterator, (e) => ut(this, e));
}, concat(...e) {
  return Gt(this).concat(...e.map((t) => Q(t) ? Gt(t) : t));
}, entries() {
  return ws(this, "entries", (e) => (e[1] = ut(this, e[1]), e));
}, every(e, t) {
  return mt(this, "every", e, t, void 0, arguments);
}, filter(e, t) {
  return mt(this, "filter", e, t, (n) => n.map((s) => ut(this, s)), arguments);
}, find(e, t) {
  return mt(this, "find", e, t, (n) => ut(this, n), arguments);
}, findIndex(e, t) {
  return mt(this, "findIndex", e, t, void 0, arguments);
}, findLast(e, t) {
  return mt(this, "findLast", e, t, (n) => ut(this, n), arguments);
}, findLastIndex(e, t) {
  return mt(this, "findLastIndex", e, t, void 0, arguments);
}, forEach(e, t) {
  return mt(this, "forEach", e, t, void 0, arguments);
}, includes(...e) {
  return Ss(this, "includes", e);
}, indexOf(...e) {
  return Ss(this, "indexOf", e);
}, join(e) {
  return Gt(this).join(e);
}, lastIndexOf(...e) {
  return Ss(this, "lastIndexOf", e);
}, map(e, t) {
  return mt(this, "map", e, t, void 0, arguments);
}, pop() {
  return dn(this, "pop");
}, push(...e) {
  return dn(this, "push", e);
}, reduce(e, ...t) {
  return Ai(this, "reduce", e, t);
}, reduceRight(e, ...t) {
  return Ai(this, "reduceRight", e, t);
}, shift() {
  return dn(this, "shift");
}, some(e, t) {
  return mt(this, "some", e, t, void 0, arguments);
}, splice(...e) {
  return dn(this, "splice", e);
}, toReversed() {
  return Gt(this).toReversed();
}, toSorted(e) {
  return Gt(this).toSorted(e);
}, toSpliced(...e) {
  return Gt(this).toSpliced(...e);
}, unshift(...e) {
  return dn(this, "unshift", e);
}, values() {
  return ws(this, "values", (e) => ut(this, e));
} };
function ws(e, t, n) {
  const s = as(e), i = s[t]();
  return s !== e && !Je(e) && (i._next = i.next, i.next = () => {
    const r = i._next();
    return r.done || (r.value = n(r.value)), r;
  }), i;
}
const dl = Array.prototype;
function mt(e, t, n, s, i, r) {
  const o = as(e), l = o !== e && !Je(e), a = o[t];
  if (a !== dl[t]) {
    const c = a.apply(e, r);
    return l ? Ze(c) : c;
  }
  let p = n;
  o !== e && (l ? p = function(c, v) {
    return n.call(this, ut(e, c), v, e);
  } : n.length > 2 && (p = function(c, v) {
    return n.call(this, c, v, e);
  }));
  const u = a.call(o, p, s);
  return l && i ? i(u) : u;
}
function Ai(e, t, n, s) {
  const i = as(e), r = i !== e && !Je(e);
  let o = n, l = false;
  i !== e && (r ? (l = s.length === 0, o = function(p, u, c) {
    return l && (l = false, p = ut(e, p)), n.call(this, p, ut(e, u), c, e);
  }) : n.length > 3 && (o = function(p, u, c) {
    return n.call(this, p, u, c, e);
  }));
  const a = i[t](o, ...s);
  return l ? ut(e, a) : a;
}
function Ss(e, t, n) {
  const s = fe(e);
  Ie(s, "iterate", Pn);
  const i = s[t](...n);
  return (i === -1 || i === false) && di(n[0]) ? (n[0] = fe(n[0]), s[t](...n)) : i;
}
function dn(e, t, n = []) {
  vt(), oi();
  const s = fe(e)[t].apply(e, n);
  return li(), gt(), s;
}
const pl = ti("__proto__,__v_isRef,__isVue"), Ur = new Set(Object.getOwnPropertyNames(Symbol).filter((e) => e !== "arguments" && e !== "caller").map((e) => Symbol[e]).filter(ht));
function hl(e) {
  ht(e) || (e = String(e));
  const t = fe(this);
  return Ie(t, "has", e), t.hasOwnProperty(e);
}
class Vr {
  constructor(t = false, n = false) {
    this._isReadonly = t, this._isShallow = n;
  }
  get(t, n, s) {
    if (n === "__v_skip") return t.__v_skip;
    const i = this._isReadonly, r = this._isShallow;
    if (n === "__v_isReactive") return !i;
    if (n === "__v_isReadonly") return i;
    if (n === "__v_isShallow") return r;
    if (n === "__v_raw") return s === (i ? r ? El : Br : r ? Wr : Fr).get(t) || Object.getPrototypeOf(t) === Object.getPrototypeOf(s) ? t : void 0;
    const o = Q(t);
    if (!i) {
      let a;
      if (o && (a = fl[n])) return a;
      if (n === "hasOwnProperty") return hl;
    }
    const l = Reflect.get(t, n, Ne(t) ? t : s);
    if ((ht(n) ? Ur.has(n) : pl(n)) || (i || Ie(t, "get", n), r)) return l;
    if (Ne(l)) {
      const a = o && si(n) ? l : l.value;
      return i && pe(a) ? js(a) : a;
    }
    return pe(l) ? i ? js(l) : un(l) : l;
  }
}
class Hr extends Vr {
  constructor(t = false) {
    super(false, t);
  }
  set(t, n, s, i) {
    let r = t[n];
    const o = Q(t) && si(n);
    if (!this._isShallow) {
      const p = Ct(r);
      if (!Je(s) && !Ct(s) && (r = fe(r), s = fe(s)), !o && Ne(r) && !Ne(s)) return p || (r.value = s), true;
    }
    const l = o ? Number(n) < t.length : de(t, n), a = Reflect.set(t, n, s, Ne(t) ? t : i);
    return t === fe(i) && a && (l ? dt(s, r) && wt(t, "set", n, s) : wt(t, "add", n, s)), a;
  }
  deleteProperty(t, n) {
    const s = de(t, n);
    t[n];
    const i = Reflect.deleteProperty(t, n);
    return i && s && wt(t, "delete", n, void 0), i;
  }
  has(t, n) {
    const s = Reflect.has(t, n);
    return (!ht(n) || !Ur.has(n)) && Ie(t, "has", n), s;
  }
  ownKeys(t) {
    return Ie(t, "iterate", Q(t) ? "length" : Bt), Reflect.ownKeys(t);
  }
}
class vl extends Vr {
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
const gl = new Hr(), ml = new vl(), yl = new Hr(true);
const Ls = (e) => e, Vn = (e) => Reflect.getPrototypeOf(e);
function _l(e, t, n) {
  return function(...s) {
    const i = this.__v_raw, r = fe(i), o = Zt(r), l = e === "entries" || e === Symbol.iterator && o, a = e === "keys" && o, p = i[e](...s), u = n ? Ls : t ? nn : Ze;
    return !t && Ie(r, "iterate", a ? Ds : Bt), Oe(Object.create(p), { next() {
      const { value: c, done: v } = p.next();
      return v ? { value: c, done: v } : { value: l ? [u(c[0]), u(c[1])] : u(c), done: v };
    } });
  };
}
function Hn(e) {
  return function(...t) {
    return e === "delete" ? false : e === "clear" ? void 0 : this;
  };
}
function bl(e, t) {
  const n = { get(i) {
    const r = this.__v_raw, o = fe(r), l = fe(i);
    e || (dt(i, l) && Ie(o, "get", i), Ie(o, "get", l));
    const { has: a } = Vn(o), p = t ? Ls : e ? nn : Ze;
    if (a.call(o, i)) return p(r.get(i));
    if (a.call(o, l)) return p(r.get(l));
    r !== o && r.get(i);
  }, get size() {
    const i = this.__v_raw;
    return !e && Ie(fe(i), "iterate", Bt), i.size;
  }, has(i) {
    const r = this.__v_raw, o = fe(r), l = fe(i);
    return e || (dt(i, l) && Ie(o, "has", i), Ie(o, "has", l)), i === l ? r.has(i) : r.has(i) || r.has(l);
  }, forEach(i, r) {
    const o = this, l = o.__v_raw, a = fe(l), p = t ? Ls : e ? nn : Ze;
    return !e && Ie(a, "iterate", Bt), l.forEach((u, c) => i.call(r, p(u), p(c), o));
  } };
  return Oe(n, e ? { add: Hn("add"), set: Hn("set"), delete: Hn("delete"), clear: Hn("clear") } : { add(i) {
    const r = fe(this), o = Vn(r), l = fe(i), a = !t && !Je(i) && !Ct(i) ? l : i;
    return o.has.call(r, a) || dt(i, a) && o.has.call(r, i) || dt(l, a) && o.has.call(r, l) || (r.add(a), wt(r, "add", a, a)), this;
  }, set(i, r) {
    !t && !Je(r) && !Ct(r) && (r = fe(r));
    const o = fe(this), { has: l, get: a } = Vn(o);
    let p = l.call(o, i);
    p || (i = fe(i), p = l.call(o, i));
    const u = a.call(o, i);
    return o.set(i, r), p ? dt(r, u) && wt(o, "set", i, r) : wt(o, "add", i, r), this;
  }, delete(i) {
    const r = fe(this), { has: o, get: l } = Vn(r);
    let a = o.call(r, i);
    a || (i = fe(i), a = o.call(r, i)), l && l.call(r, i);
    const p = r.delete(i);
    return a && wt(r, "delete", i, void 0), p;
  }, clear() {
    const i = fe(this), r = i.size !== 0, o = i.clear();
    return r && wt(i, "clear", void 0, void 0), o;
  } }), ["keys", "values", "entries", Symbol.iterator].forEach((i) => {
    n[i] = _l(i, e, t);
  }), n;
}
function ui(e, t) {
  const n = bl(e, t);
  return (s, i, r) => i === "__v_isReactive" ? !e : i === "__v_isReadonly" ? e : i === "__v_raw" ? s : Reflect.get(de(n, i) && i in s ? n : s, i, r);
}
const wl = { get: ui(false, false) }, Sl = { get: ui(false, true) }, kl = { get: ui(true, false) };
const Fr = /* @__PURE__ */ new WeakMap(), Wr = /* @__PURE__ */ new WeakMap(), Br = /* @__PURE__ */ new WeakMap(), El = /* @__PURE__ */ new WeakMap();
function Cl(e) {
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
function un(e) {
  return Ct(e) ? e : fi(e, false, gl, wl, Fr);
}
function qr(e) {
  return fi(e, false, yl, Sl, Wr);
}
function js(e) {
  return fi(e, true, ml, kl, Br);
}
function fi(e, t, n, s, i) {
  if (!pe(e) || e.__v_raw && !(t && e.__v_isReactive) || e.__v_skip || !Object.isExtensible(e)) return e;
  const r = i.get(e);
  if (r) return r;
  const o = Cl(Yo(e));
  if (o === 0) return e;
  const l = new Proxy(e, o === 2 ? s : n);
  return i.set(e, l), l;
}
function qt(e) {
  return Ct(e) ? qt(e.__v_raw) : !!(e && e.__v_isReactive);
}
function Ct(e) {
  return !!(e && e.__v_isReadonly);
}
function Je(e) {
  return !!(e && e.__v_isShallow);
}
function di(e) {
  return e ? !!e.__v_raw : false;
}
function fe(e) {
  const t = e && e.__v_raw;
  return t ? fe(t) : e;
}
function xl(e) {
  return !de(e, "__v_skip") && Object.isExtensible(e) && Ar(e, "__v_skip", true), e;
}
const Ze = (e) => pe(e) ? un(e) : e, nn = (e) => pe(e) ? js(e) : e;
function Ne(e) {
  return e ? e.__v_isRef === true : false;
}
function X(e) {
  return Kr(e, false);
}
function Al(e) {
  return Kr(e, true);
}
function Kr(e, t) {
  return Ne(e) ? e : new Rl(e, t);
}
class Rl {
  constructor(t, n) {
    this.dep = new ci(), this.__v_isRef = true, this.__v_isShallow = false, this._rawValue = n ? t : fe(t), this._value = n ? t : Ze(t), this.__v_isShallow = n;
  }
  get value() {
    return this.dep.track(), this._value;
  }
  set value(t) {
    const n = this._rawValue, s = this.__v_isShallow || Je(t) || Ct(t);
    t = s ? t : fe(t), dt(t, n) && (this._rawValue = t, this._value = s ? t : Ze(t), this.dep.trigger());
  }
}
function z(e) {
  return Ne(e) ? e.value : e;
}
const Pl = { get: (e, t, n) => t === "__v_raw" ? e : z(Reflect.get(e, t, n)), set: (e, t, n, s) => {
  const i = e[t];
  return Ne(i) && !Ne(n) ? (i.value = n, true) : Reflect.set(e, t, n, s);
} };
function Gr(e) {
  return qt(e) ? e : new Proxy(e, Pl);
}
class Ol {
  constructor(t, n, s) {
    this.fn = t, this.setter = n, this._value = void 0, this.dep = new ci(this), this.__v_isRef = true, this.deps = void 0, this.depsTail = void 0, this.flags = 16, this.globalVersion = Rn - 1, this.next = void 0, this.effect = this, this.__v_isReadonly = !n, this.isSSR = s;
  }
  notify() {
    if (this.flags |= 16, !(this.flags & 8) && _e !== this) return Tr(this, true), true;
  }
  get value() {
    const t = this.dep.track();
    return Dr(this), t && (t.version = this.dep.version), this._value;
  }
  set value(t) {
    this.setter && this.setter(t);
  }
}
function $l(e, t, n = false) {
  let s, i;
  return ee(e) ? s = e : (s = e.get, i = e.set), new Ol(s, i, n);
}
const Fn = {}, zn = /* @__PURE__ */ new WeakMap();
let Vt;
function Il(e, t = false, n = Vt) {
  if (n) {
    let s = zn.get(n);
    s || zn.set(n, s = []), s.push(e);
  }
}
function Tl(e, t, n = me) {
  const { immediate: s, deep: i, once: r, scheduler: o, augmentJob: l, call: a } = n, p = (F) => i ? F : Je(F) || i === false || i === 0 ? St(F, 1) : St(F);
  let u, c, v, m, b = false, _ = false;
  if (Ne(e) ? (c = () => e.value, b = Je(e)) : qt(e) ? (c = () => p(e), b = true) : Q(e) ? (_ = true, b = e.some((F) => qt(F) || Je(F)), c = () => e.map((F) => {
    if (Ne(F)) return F.value;
    if (qt(F)) return p(F);
    if (ee(F)) return a ? a(F, 2) : F();
  })) : ee(e) ? t ? c = a ? () => a(e, 2) : e : c = () => {
    if (v) {
      vt();
      try {
        v();
      } finally {
        gt();
      }
    }
    const F = Vt;
    Vt = u;
    try {
      return a ? a(e, 3, [m]) : e(m);
    } finally {
      Vt = F;
    }
  } : c = pt, t && i) {
    const F = c, ie = i === true ? 1 / 0 : i;
    c = () => St(F(), ie);
  }
  const P = al(), O = () => {
    u.stop(), P && P.active && ni(P.effects, u);
  };
  if (r && t) {
    const F = t;
    t = (...ie) => {
      const he = F(...ie);
      return O(), he;
    };
  }
  let w = _ ? new Array(e.length).fill(Fn) : Fn;
  const I = (F) => {
    if (!(!(u.flags & 1) || !u.dirty && !F)) if (t) {
      const ie = u.run();
      if (F || i || b || (_ ? ie.some((he, ae) => dt(he, w[ae])) : dt(ie, w))) {
        v && v();
        const he = Vt;
        Vt = u;
        try {
          const ae = [ie, w === Fn ? void 0 : _ && w[0] === Fn ? [] : w, m];
          w = ie, a ? a(t, 3, ae) : t(...ae);
        } finally {
          Vt = he;
        }
      }
    } else u.run();
  };
  return l && l(I), u = new $r(c), u.scheduler = o ? () => o(I, false) : I, m = (F) => Il(F, false, u), v = u.onStop = () => {
    const F = zn.get(u);
    if (F) {
      if (a) a(F, 4);
      else for (const ie of F) ie();
      zn.delete(u);
    }
  }, t ? s ? I(true) : w = u.run() : o ? o(I.bind(null, true), true) : u.run(), O.pause = u.pause.bind(u), O.resume = u.resume.bind(u), O.stop = O, O;
}
function St(e, t = 1 / 0, n) {
  if (t <= 0 || !pe(e) || e.__v_skip || (n = n || /* @__PURE__ */ new Map(), (n.get(e) || 0) >= t)) return e;
  if (n.set(e, t), t--, Ne(e)) St(e.value, t, n);
  else if (Q(e)) for (let s = 0; s < e.length; s++) St(e[s], t, n);
  else if (an(e) || Zt(e)) e.forEach((s) => {
    St(s, t, n);
  });
  else if (xr(e)) {
    for (const s in e) St(e[s], t, n);
    for (const s of Object.getOwnPropertySymbols(e)) Object.prototype.propertyIsEnumerable.call(e, s) && St(e[s], t, n);
  }
  return e;
}
/**
* @vue/runtime-core v3.5.39
* (c) 2018-present Yuxi (Evan) You and Vue contributors
* @license MIT
**/
function Dn(e, t, n, s) {
  try {
    return s ? e(...s) : e();
  } catch (i) {
    cs(i, t, n);
  }
}
function et(e, t, n, s) {
  if (ee(e)) {
    const i = Dn(e, t, n, s);
    return i && Er(i) && i.catch((r) => {
      cs(r, t, n);
    }), i;
  }
  if (Q(e)) {
    const i = [];
    for (let r = 0; r < e.length; r++) i.push(et(e[r], t, n, s));
    return i;
  }
}
function cs(e, t, n, s = true) {
  const i = t ? t.vnode : null, { errorHandler: r, throwUnhandledErrorInProduction: o } = t && t.appContext.config || me;
  if (t) {
    let l = t.parent;
    const a = t.proxy, p = `https://vuejs.org/error-reference/#runtime-${n}`;
    for (; l; ) {
      const u = l.ec;
      if (u) {
        for (let c = 0; c < u.length; c++) if (u[c](e, a, p) === false) return;
      }
      l = l.parent;
    }
    if (r) {
      vt(), Dn(r, null, 10, [e, a, p]), gt();
      return;
    }
  }
  Nl(e, n, i, s, o);
}
function Nl(e, t, n, s = true, i = false) {
  if (i) throw e;
  console.error(e);
}
const Ve = [];
let ct = -1;
const en = [];
let Pt = null, Jt = 0;
const zr = Promise.resolve();
let Jn = null;
function pi(e) {
  const t = Jn || zr;
  return e ? t.then(this ? e.bind(this) : e) : t;
}
function Ml(e) {
  let t = ct + 1, n = Ve.length;
  for (; t < n; ) {
    const s = t + n >>> 1, i = Ve[s], r = On(i);
    r < e || r === e && i.flags & 2 ? t = s + 1 : n = s;
  }
  return t;
}
function hi(e) {
  if (!(e.flags & 1)) {
    const t = On(e), n = Ve[Ve.length - 1];
    !n || !(e.flags & 2) && t >= On(n) ? Ve.push(e) : Ve.splice(Ml(t), 0, e), e.flags |= 1, Jr();
  }
}
function Jr() {
  Jn || (Jn = zr.then(Yr));
}
function Dl(e) {
  Q(e) ? en.push(...e) : Pt && e.id === -1 ? Pt.splice(Jt + 1, 0, e) : e.flags & 1 || (en.push(e), e.flags |= 1), Jr();
}
function Ri(e, t, n = ct + 1) {
  for (; n < Ve.length; n++) {
    const s = Ve[n];
    if (s && s.flags & 2) {
      if (e && s.id !== e.uid) continue;
      Ve.splice(n, 1), n--, s.flags & 4 && (s.flags &= -2), s(), s.flags & 4 || (s.flags &= -2);
    }
  }
}
function Qr(e) {
  if (en.length) {
    const t = [...new Set(en)].sort((n, s) => On(n) - On(s));
    if (en.length = 0, Pt) {
      Pt.push(...t);
      return;
    }
    for (Pt = t, Jt = 0; Jt < Pt.length; Jt++) {
      const n = Pt[Jt];
      n.flags & 4 && (n.flags &= -2), n.flags & 8 || n(), n.flags &= -2;
    }
    Pt = null, Jt = 0;
  }
}
const On = (e) => e.id == null ? e.flags & 2 ? -1 : 1 / 0 : e.id;
function Yr(e) {
  try {
    for (ct = 0; ct < Ve.length; ct++) {
      const t = Ve[ct];
      t && !(t.flags & 8) && (t.flags & 4 && (t.flags &= -2), Dn(t, t.i, t.i ? 15 : 14), t.flags & 4 || (t.flags &= -2));
    }
  } finally {
    for (; ct < Ve.length; ct++) {
      const t = Ve[ct];
      t && (t.flags &= -2);
    }
    ct = -1, Ve.length = 0, Qr(), Jn = null, (Ve.length || en.length) && Yr();
  }
}
let qe = null, Xr = null;
function Qn(e) {
  const t = qe;
  return qe = e, Xr = e && e.type.__scopeId || null, t;
}
function Ht(e, t = qe, n) {
  if (!t || e._n) return e;
  const s = (...i) => {
    s._d && Zn(-1);
    const r = Qn(t);
    let o;
    try {
      o = e(...i);
    } finally {
      Qn(r), s._d && Zn(1);
    }
    return o;
  };
  return s._n = true, s._c = true, s._d = true, s;
}
function Ae(e, t) {
  if (qe === null) return e;
  const n = hs(qe), s = e.dirs || (e.dirs = []);
  for (let i = 0; i < t.length; i++) {
    let [r, o, l, a = me] = t[i];
    r && (ee(r) && (r = { mounted: r, updated: r }), r.deep && St(o), s.push({ dir: r, instance: n, value: o, oldValue: void 0, arg: l, modifiers: a }));
  }
  return e;
}
function jt(e, t, n, s) {
  const i = e.dirs, r = t && t.dirs;
  for (let o = 0; o < i.length; o++) {
    const l = i[o];
    r && (l.oldValue = r[o].value);
    let a = l.dir[s];
    a && (vt(), et(a, n, 8, [e.el, l, e, t]), gt());
  }
}
function Bn(e, t) {
  if (Te) {
    let n = Te.provides;
    const s = Te.parent && Te.parent.provides;
    s === n && (n = Te.provides = Object.create(s)), n[e] = t;
  }
}
function Xe(e, t, n = false) {
  const s = La();
  if (s || tn) {
    let i = tn ? tn._context.provides : s ? s.parent == null || s.ce ? s.vnode.appContext && s.vnode.appContext.provides : s.parent.provides : void 0;
    if (i && e in i) return i[e];
    if (arguments.length > 1) return n && ee(t) ? t.call(s && s.proxy) : t;
  }
}
const Ll = Symbol.for("v-scx"), jl = () => Xe(Ll);
function kt(e, t, n) {
  return Zr(e, t, n);
}
function Zr(e, t, n = me) {
  const { immediate: s, deep: i, flush: r, once: o } = n, l = Oe({}, n), a = t && s || !t && r !== "post";
  let p;
  if (In) {
    if (r === "sync") {
      const m = jl();
      p = m.__watcherHandles || (m.__watcherHandles = []);
    } else if (!a) {
      const m = () => {
      };
      return m.stop = pt, m.resume = pt, m.pause = pt, m;
    }
  }
  const u = Te;
  l.call = (m, b, _) => et(m, u, b, _);
  let c = false;
  r === "post" ? l.scheduler = (m) => {
    We(m, u && u.suspense);
  } : r !== "sync" && (c = true, l.scheduler = (m, b) => {
    b ? m() : hi(m);
  }), l.augmentJob = (m) => {
    t && (m.flags |= 4), c && (m.flags |= 2, u && (m.id = u.uid, m.i = u));
  };
  const v = Tl(e, t, l);
  return In && (p ? p.push(v) : a && v()), v;
}
function Ul(e, t, n) {
  const s = this.proxy, i = ke(e) ? e.includes(".") ? eo(s, e) : () => s[e] : e.bind(s, s);
  let r;
  ee(t) ? r = t : (r = t.handler, n = t);
  const o = jn(this), l = Zr(i, r.bind(s), n);
  return o(), l;
}
function eo(e, t) {
  const n = t.split(".");
  return () => {
    let s = e;
    for (let i = 0; i < n.length && s; i++) s = s[n[i]];
    return s;
  };
}
const Vl = Symbol("_vte"), Hl = (e) => e.__isTeleport, ks = Symbol("_leaveCb");
function vi(e, t) {
  e.shapeFlag & 6 && e.component ? (e.transition = t, vi(e.component.subTree, t)) : e.shapeFlag & 128 ? (e.ssContent.transition = t.clone(e.ssContent), e.ssFallback.transition = t.clone(e.ssFallback)) : e.transition = t;
}
function nt(e, t) {
  return ee(e) ? Oe({ name: e.name }, t, { setup: e }) : e;
}
function to(e) {
  e.ids = [e.ids[0] + e.ids[2]++ + "-", 0, 0];
}
function Pi(e, t) {
  let n;
  return !!((n = Object.getOwnPropertyDescriptor(e, t)) && !n.configurable);
}
const Yn = /* @__PURE__ */ new WeakMap();
function kn(e, t, n, s, i = false) {
  if (Q(e)) {
    e.forEach((_, P) => kn(_, t && (Q(t) ? t[P] : t), n, s, i));
    return;
  }
  if (En(s) && !i) {
    s.shapeFlag & 512 && s.type.__asyncResolved && s.component.subTree.component && kn(e, t, n, s.component.subTree);
    return;
  }
  const r = s.shapeFlag & 4 ? hs(s.component) : s.el, o = i ? null : r, { i: l, r: a } = e, p = t && t.r, u = l.refs === me ? l.refs = {} : l.refs, c = l.setupState, v = fe(c), m = c === me ? kr : (_) => Pi(u, _) ? false : de(v, _), b = (_, P) => !(P && Pi(u, P));
  if (p != null && p !== a) {
    if (Oi(t), ke(p)) u[p] = null, m(p) && (c[p] = null);
    else if (Ne(p)) {
      const _ = t;
      b(p, _.k) && (p.value = null), _.k && (u[_.k] = null);
    }
  }
  if (ee(a)) {
    vt();
    try {
      Dn(a, l, 12, [o, u]);
    } finally {
      gt();
    }
  } else {
    const _ = ke(a), P = Ne(a);
    if (_ || P) {
      const O = () => {
        if (e.f) {
          const w = _ ? m(a) ? c[a] : u[a] : b() || !e.k ? a.value : u[e.k];
          if (i) Q(w) && ni(w, r);
          else if (Q(w)) w.includes(r) || w.push(r);
          else if (_) u[a] = [r], m(a) && (c[a] = u[a]);
          else {
            const I = [r];
            b(a, e.k) && (a.value = I), e.k && (u[e.k] = I);
          }
        } else _ ? (u[a] = o, m(a) && (c[a] = o)) : P && (b(a, e.k) && (a.value = o), e.k && (u[e.k] = o));
      };
      if (o) {
        const w = () => {
          O(), Yn.delete(e);
        };
        w.id = -1, Yn.set(e, w), We(w, n);
      } else Oi(e), O();
    }
  }
}
function Oi(e) {
  const t = Yn.get(e);
  t && (t.flags |= 8, Yn.delete(e));
}
ls().requestIdleCallback;
ls().cancelIdleCallback;
const En = (e) => !!e.type.__asyncLoader, no = (e) => e.type.__isKeepAlive;
function Fl(e, t) {
  so(e, "a", t);
}
function Wl(e, t) {
  so(e, "da", t);
}
function so(e, t, n = Te) {
  const s = e.__wdc || (e.__wdc = () => {
    let i = n;
    for (; i; ) {
      if (i.isDeactivated) return;
      i = i.parent;
    }
    return e();
  });
  if (us(t, s, n), n) {
    let i = n.parent;
    for (; i && i.parent; ) no(i.parent.vnode) && Bl(s, t, n, i), i = i.parent;
  }
}
function Bl(e, t, n, s) {
  const i = us(t, e, s, true);
  fs(() => {
    ni(s[t], i);
  }, n);
}
function us(e, t, n = Te, s = false) {
  if (n) {
    const i = n[e] || (n[e] = []), r = t.__weh || (t.__weh = (...o) => {
      vt();
      const l = jn(n), a = et(t, n, e, o);
      return l(), gt(), a;
    });
    return s ? i.unshift(r) : i.push(r), r;
  }
}
const xt = (e) => (t, n = Te) => {
  (!In || e === "sp") && us(e, (...s) => t(...s), n);
}, ql = xt("bm"), Ln = xt("m"), Kl = xt("bu"), Gl = xt("u"), zl = xt("bum"), fs = xt("um"), Jl = xt("sp"), Ql = xt("rtg"), Yl = xt("rtc");
function Xl(e, t = Te) {
  us("ec", e, t);
}
const Zl = "directives", ea = Symbol.for("v-ndc");
function ta(e) {
  return na(Zl, e);
}
function na(e, t, n = true, s = false) {
  const i = qe || Te;
  if (i) {
    const r = i.type, o = $i(i[e] || r[e], t) || $i(i.appContext[e], t);
    return !o && s ? r : o;
  }
}
function $i(e, t) {
  return e && (e[t] || e[Ge(t)] || e[ii(Ge(t))]);
}
function Ce(e, t, n, s) {
  let i;
  const r = n, o = Q(e);
  if (o || ke(e)) {
    const l = o && qt(e);
    let a = false, p = false;
    l && (a = !Je(e), p = Ct(e), e = as(e)), i = new Array(e.length);
    for (let u = 0, c = e.length; u < c; u++) i[u] = t(a ? p ? nn(Ze(e[u])) : Ze(e[u]) : e[u], u, void 0, r);
  } else if (typeof e == "number") {
    i = new Array(e);
    for (let l = 0; l < e; l++) i[l] = t(l + 1, l, void 0, r);
  } else if (pe(e)) if (e[Symbol.iterator]) i = Array.from(e, (l, a) => t(l, a, void 0, r));
  else {
    const l = Object.keys(e);
    i = new Array(l.length);
    for (let a = 0, p = l.length; a < p; a++) {
      const u = l[a];
      i[a] = t(e[u], u, a, r);
    }
  }
  else i = [];
  return i;
}
const Us = (e) => e ? Co(e) ? hs(e) : Us(e.parent) : null, Cn = Oe(/* @__PURE__ */ Object.create(null), { $: (e) => e, $el: (e) => e.vnode.el, $data: (e) => e.data, $props: (e) => e.props, $attrs: (e) => e.attrs, $slots: (e) => e.slots, $refs: (e) => e.refs, $parent: (e) => Us(e.parent), $root: (e) => Us(e.root), $host: (e) => e.ce, $emit: (e) => e.emit, $options: (e) => ro(e), $forceUpdate: (e) => e.f || (e.f = () => {
  hi(e.update);
}), $nextTick: (e) => e.n || (e.n = pi.bind(e.proxy)), $watch: (e) => Ul.bind(e) }), Es = (e, t) => e !== me && !e.__isScriptSetup && de(e, t), sa = { get({ _: e }, t) {
  if (t === "__v_skip") return true;
  const { ctx: n, setupState: s, data: i, props: r, accessCache: o, type: l, appContext: a } = e;
  if (t[0] !== "$") {
    const v = o[t];
    if (v !== void 0) switch (v) {
      case 1:
        return s[t];
      case 2:
        return i[t];
      case 4:
        return n[t];
      case 3:
        return r[t];
    }
    else {
      if (Es(s, t)) return o[t] = 1, s[t];
      if (i !== me && de(i, t)) return o[t] = 2, i[t];
      if (de(r, t)) return o[t] = 3, r[t];
      if (n !== me && de(n, t)) return o[t] = 4, n[t];
      Vs && (o[t] = 0);
    }
  }
  const p = Cn[t];
  let u, c;
  if (p) return t === "$attrs" && Ie(e.attrs, "get", ""), p(e);
  if ((u = l.__cssModules) && (u = u[t])) return u;
  if (n !== me && de(n, t)) return o[t] = 4, n[t];
  if (c = a.config.globalProperties, de(c, t)) return c[t];
}, set({ _: e }, t, n) {
  const { data: s, setupState: i, ctx: r } = e;
  return Es(i, t) ? (i[t] = n, true) : s !== me && de(s, t) ? (s[t] = n, true) : de(e.props, t) || t[0] === "$" && t.slice(1) in e ? false : (r[t] = n, true);
}, has({ _: { data: e, setupState: t, accessCache: n, ctx: s, appContext: i, props: r, type: o } }, l) {
  let a;
  return !!(n[l] || e !== me && l[0] !== "$" && de(e, l) || Es(t, l) || de(r, l) || de(s, l) || de(Cn, l) || de(i.config.globalProperties, l) || (a = o.__cssModules) && a[l]);
}, defineProperty(e, t, n) {
  return n.get != null ? e._.accessCache[t] = 0 : de(n, "value") && this.set(e, t, n.value, null), Reflect.defineProperty(e, t, n);
} };
function Ii(e) {
  return Q(e) ? e.reduce((t, n) => (t[n] = null, t), {}) : e;
}
let Vs = true;
function ia(e) {
  const t = ro(e), n = e.proxy, s = e.ctx;
  Vs = false, t.beforeCreate && Ti(t.beforeCreate, e, "bc");
  const { data: i, computed: r, methods: o, watch: l, provide: a, inject: p, created: u, beforeMount: c, mounted: v, beforeUpdate: m, updated: b, activated: _, deactivated: P, beforeDestroy: O, beforeUnmount: w, destroyed: I, unmounted: F, render: ie, renderTracked: he, renderTriggered: ae, errorCaptured: Me, serverPrefetch: He, expose: Fe, inheritAttrs: ze, components: Qe, directives: De, filters: st } = t;
  if (p && ra(p, s, null), o) for (const oe in o) {
    const te = o[oe];
    ee(te) && (s[oe] = te.bind(n));
  }
  if (i) {
    const oe = i.call(n, n);
    pe(oe) && (e.data = un(oe));
  }
  if (Vs = true, r) for (const oe in r) {
    const te = r[oe], Le = ee(te) ? te.bind(n, n) : ee(te.get) ? te.get.bind(n, n) : pt, N = !ee(te) && ee(te.set) ? te.set.bind(n) : pt, j = se({ get: Le, set: N });
    Object.defineProperty(s, oe, { enumerable: true, configurable: true, get: () => j.value, set: (W) => j.value = W });
  }
  if (l) for (const oe in l) io(l[oe], s, n, oe);
  if (a) {
    const oe = ee(a) ? a.call(n) : a;
    Reflect.ownKeys(oe).forEach((te) => {
      Bn(te, oe[te]);
    });
  }
  u && Ti(u, e, "c");
  function be(oe, te) {
    Q(te) ? te.forEach((Le) => oe(Le.bind(n))) : te && oe(te.bind(n));
  }
  if (be(ql, c), be(Ln, v), be(Kl, m), be(Gl, b), be(Fl, _), be(Wl, P), be(Xl, Me), be(Yl, he), be(Ql, ae), be(zl, w), be(fs, F), be(Jl, He), Q(Fe)) if (Fe.length) {
    const oe = e.exposed || (e.exposed = {});
    Fe.forEach((te) => {
      Object.defineProperty(oe, te, { get: () => n[te], set: (Le) => n[te] = Le, enumerable: true });
    });
  } else e.exposed || (e.exposed = {});
  ie && e.render === pt && (e.render = ie), ze != null && (e.inheritAttrs = ze), Qe && (e.components = Qe), De && (e.directives = De), He && to(e);
}
function ra(e, t, n = pt) {
  Q(e) && (e = Hs(e));
  for (const s in e) {
    const i = e[s];
    let r;
    pe(i) ? "default" in i ? r = Xe(i.from || s, i.default, true) : r = Xe(i.from || s) : r = Xe(i), Ne(r) ? Object.defineProperty(t, s, { enumerable: true, configurable: true, get: () => r.value, set: (o) => r.value = o }) : t[s] = r;
  }
}
function Ti(e, t, n) {
  et(Q(e) ? e.map((s) => s.bind(t.proxy)) : e.bind(t.proxy), t, n);
}
function io(e, t, n, s) {
  let i = s.includes(".") ? eo(n, s) : () => n[s];
  if (ke(e)) {
    const r = t[e];
    ee(r) && kt(i, r);
  } else if (ee(e)) kt(i, e.bind(n));
  else if (pe(e)) if (Q(e)) e.forEach((r) => io(r, t, n, s));
  else {
    const r = ee(e.handler) ? e.handler.bind(n) : t[e.handler];
    ee(r) && kt(i, r, e);
  }
}
function ro(e) {
  const t = e.type, { mixins: n, extends: s } = t, { mixins: i, optionsCache: r, config: { optionMergeStrategies: o } } = e.appContext, l = r.get(t);
  let a;
  return l ? a = l : !i.length && !n && !s ? a = t : (a = {}, i.length && i.forEach((p) => Xn(a, p, o, true)), Xn(a, t, o)), pe(t) && r.set(t, a), a;
}
function Xn(e, t, n, s = false) {
  const { mixins: i, extends: r } = t;
  r && Xn(e, r, n, true), i && i.forEach((o) => Xn(e, o, n, true));
  for (const o in t) if (!(s && o === "expose")) {
    const l = oa[o] || n && n[o];
    e[o] = l ? l(e[o], t[o]) : t[o];
  }
  return e;
}
const oa = { data: Ni, props: Mi, emits: Mi, methods: vn, computed: vn, beforeCreate: je, created: je, beforeMount: je, mounted: je, beforeUpdate: je, updated: je, beforeDestroy: je, beforeUnmount: je, destroyed: je, unmounted: je, activated: je, deactivated: je, errorCaptured: je, serverPrefetch: je, components: vn, directives: vn, watch: aa, provide: Ni, inject: la };
function Ni(e, t) {
  return t ? e ? function() {
    return Oe(ee(e) ? e.call(this, this) : e, ee(t) ? t.call(this, this) : t);
  } : t : e;
}
function la(e, t) {
  return vn(Hs(e), Hs(t));
}
function Hs(e) {
  if (Q(e)) {
    const t = {};
    for (let n = 0; n < e.length; n++) t[e[n]] = e[n];
    return t;
  }
  return e;
}
function je(e, t) {
  return e ? [...new Set([].concat(e, t))] : t;
}
function vn(e, t) {
  return e ? Oe(/* @__PURE__ */ Object.create(null), e, t) : t;
}
function Mi(e, t) {
  return e ? Q(e) && Q(t) ? [.../* @__PURE__ */ new Set([...e, ...t])] : Oe(/* @__PURE__ */ Object.create(null), Ii(e), Ii(t ?? {})) : t;
}
function aa(e, t) {
  if (!e) return t;
  if (!t) return e;
  const n = Oe(/* @__PURE__ */ Object.create(null), e);
  for (const s in t) n[s] = je(e[s], t[s]);
  return n;
}
function oo() {
  return { app: null, config: { isNativeTag: kr, performance: false, globalProperties: {}, optionMergeStrategies: {}, errorHandler: void 0, warnHandler: void 0, compilerOptions: {} }, mixins: [], components: {}, directives: {}, provides: /* @__PURE__ */ Object.create(null), optionsCache: /* @__PURE__ */ new WeakMap(), propsCache: /* @__PURE__ */ new WeakMap(), emitsCache: /* @__PURE__ */ new WeakMap() };
}
let ca = 0;
function ua(e, t) {
  return function(s, i = null) {
    ee(s) || (s = Oe({}, s)), i != null && !pe(i) && (i = null);
    const r = oo(), o = /* @__PURE__ */ new WeakSet(), l = [];
    let a = false;
    const p = r.app = { _uid: ca++, _component: s, _props: i, _container: null, _context: r, _instance: null, version: Wa, get config() {
      return r.config;
    }, set config(u) {
    }, use(u, ...c) {
      return o.has(u) || (u && ee(u.install) ? (o.add(u), u.install(p, ...c)) : ee(u) && (o.add(u), u(p, ...c))), p;
    }, mixin(u) {
      return r.mixins.includes(u) || r.mixins.push(u), p;
    }, component(u, c) {
      return c ? (r.components[u] = c, p) : r.components[u];
    }, directive(u, c) {
      return c ? (r.directives[u] = c, p) : r.directives[u];
    }, mount(u, c, v) {
      if (!a) {
        const m = p._ceVNode || we(s, i);
        return m.appContext = r, v === true ? v = "svg" : v === false && (v = void 0), e(m, u, v), a = true, p._container = u, u.__vue_app__ = p, hs(m.component);
      }
    }, onUnmount(u) {
      l.push(u);
    }, unmount() {
      a && (et(l, p._instance, 16), e(null, p._container), delete p._container.__vue_app__);
    }, provide(u, c) {
      return r.provides[u] = c, p;
    }, runWithContext(u) {
      const c = tn;
      tn = p;
      try {
        return u();
      } finally {
        tn = c;
      }
    } };
    return p;
  };
}
let tn = null;
const fa = (e, t) => t === "modelValue" || t === "model-value" ? e.modelModifiers : e[`${t}Modifiers`] || e[`${Ge(t)}Modifiers`] || e[`${Mt(t)}Modifiers`];
function da(e, t, ...n) {
  if (e.isUnmounted) return;
  const s = e.vnode.props || me;
  let i = n;
  const r = t.startsWith("update:"), o = r && fa(s, t.slice(7));
  o && (o.trim && (i = n.map((u) => ke(u) ? u.trim() : u)), o.number && (i = n.map(os)));
  let l, a = s[l = ys(t)] || s[l = ys(Ge(t))];
  !a && r && (a = s[l = ys(Mt(t))]), a && et(a, e, 6, i);
  const p = s[l + "Once"];
  if (p) {
    if (!e.emitted) e.emitted = {};
    else if (e.emitted[l]) return;
    e.emitted[l] = true, et(p, e, 6, i);
  }
}
const pa = /* @__PURE__ */ new WeakMap();
function lo(e, t, n = false) {
  const s = n ? pa : t.emitsCache, i = s.get(e);
  if (i !== void 0) return i;
  const r = e.emits;
  let o = {}, l = false;
  if (!ee(e)) {
    const a = (p) => {
      const u = lo(p, t, true);
      u && (l = true, Oe(o, u));
    };
    !n && t.mixins.length && t.mixins.forEach(a), e.extends && a(e.extends), e.mixins && e.mixins.forEach(a);
  }
  return !r && !l ? (pe(e) && s.set(e, null), null) : (Q(r) ? r.forEach((a) => o[a] = null) : Oe(o, r), pe(e) && s.set(e, o), o);
}
function ds(e, t) {
  return !e || !ss(t) ? false : (t = t.slice(2), t = t === "Once" ? t : t.replace(/Once$/, ""), de(e, t[0].toLowerCase() + t.slice(1)) || de(e, Mt(t)) || de(e, t));
}
function Di(e) {
  const { type: t, vnode: n, proxy: s, withProxy: i, propsOptions: [r], slots: o, attrs: l, emit: a, render: p, renderCache: u, props: c, data: v, setupState: m, ctx: b, inheritAttrs: _ } = e, P = Qn(e);
  let O, w;
  try {
    if (n.shapeFlag & 4) {
      const F = i || s, ie = F;
      O = ft(p.call(ie, F, u, c, m, v, b)), w = l;
    } else {
      const F = t;
      O = ft(F.length > 1 ? F(c, { attrs: l, slots: o, emit: a }) : F(c, null)), w = t.props ? l : ha(l);
    }
  } catch (F) {
    xn.length = 0, cs(F, e, 1), O = we(Nt);
  }
  let I = O;
  if (w && _ !== false) {
    const F = Object.keys(w), { shapeFlag: ie } = I;
    F.length && ie & 7 && (r && F.some(is) && (w = va(w, r)), I = sn(I, w, false, true));
  }
  return n.dirs && (I = sn(I, null, false, true), I.dirs = I.dirs ? I.dirs.concat(n.dirs) : n.dirs), n.transition && vi(I, n.transition), O = I, Qn(P), O;
}
const ha = (e) => {
  let t;
  for (const n in e) (n === "class" || n === "style" || ss(n)) && ((t || (t = {}))[n] = e[n]);
  return t;
}, va = (e, t) => {
  const n = {};
  for (const s in e) (!is(s) || !(s.slice(9) in t)) && (n[s] = e[s]);
  return n;
};
function ga(e, t, n) {
  const { props: s, children: i, component: r } = e, { props: o, children: l, patchFlag: a } = t, p = r.emitsOptions;
  if (t.dirs || t.transition) return true;
  if (n && a >= 0) {
    if (a & 1024) return true;
    if (a & 16) return s ? Li(s, o, p) : !!o;
    if (a & 8) {
      const u = t.dynamicProps;
      for (let c = 0; c < u.length; c++) {
        const v = u[c];
        if (ao(o, s, v) && !ds(p, v)) return true;
      }
    }
  } else return (i || l) && (!l || !l.$stable) ? true : s === o ? false : s ? o ? Li(s, o, p) : true : !!o;
  return false;
}
function Li(e, t, n) {
  const s = Object.keys(t);
  if (s.length !== Object.keys(e).length) return true;
  for (let i = 0; i < s.length; i++) {
    const r = s[i];
    if (ao(t, e, r) && !ds(n, r)) return true;
  }
  return false;
}
function ao(e, t, n) {
  const s = e[n], i = t[n];
  return n === "style" && pe(s) && pe(i) ? !cn(s, i) : s !== i;
}
function ma({ vnode: e, parent: t, suspense: n }, s) {
  for (; t; ) {
    const i = t.subTree;
    if (i.suspense && i.suspense.activeBranch === e && (i.suspense.vnode.el = i.el = s, e = i), i === e) (e = t.vnode).el = s, t = t.parent;
    else break;
  }
  n && n.activeBranch === e && (n.vnode.el = s);
}
const co = {}, uo = () => Object.create(co), fo = (e) => Object.getPrototypeOf(e) === co;
function ya(e, t, n, s = false) {
  const i = {}, r = uo();
  e.propsDefaults = /* @__PURE__ */ Object.create(null), po(e, t, i, r);
  for (const o in e.propsOptions[0]) o in i || (i[o] = void 0);
  n ? e.props = s ? i : qr(i) : e.type.props ? e.props = i : e.props = r, e.attrs = r;
}
function _a(e, t, n, s) {
  const { props: i, attrs: r, vnode: { patchFlag: o } } = e, l = fe(i), [a] = e.propsOptions;
  let p = false;
  if ((s || o > 0) && !(o & 16)) {
    if (o & 8) {
      const u = e.vnode.dynamicProps;
      for (let c = 0; c < u.length; c++) {
        let v = u[c];
        if (ds(e.emitsOptions, v)) continue;
        const m = t[v];
        if (a) if (de(r, v)) m !== r[v] && (r[v] = m, p = true);
        else {
          const b = Ge(v);
          i[b] = Fs(a, l, b, m, e, false);
        }
        else m !== r[v] && (r[v] = m, p = true);
      }
    }
  } else {
    po(e, t, i, r) && (p = true);
    let u;
    for (const c in l) (!t || !de(t, c) && ((u = Mt(c)) === c || !de(t, u))) && (a ? n && (n[c] !== void 0 || n[u] !== void 0) && (i[c] = Fs(a, l, c, void 0, e, true)) : delete i[c]);
    if (r !== l) for (const c in r) (!t || !de(t, c)) && (delete r[c], p = true);
  }
  p && wt(e.attrs, "set", "");
}
function po(e, t, n, s) {
  const [i, r] = e.propsOptions;
  let o = false, l;
  if (t) for (let a in t) {
    if (bn(a)) continue;
    const p = t[a];
    let u;
    i && de(i, u = Ge(a)) ? !r || !r.includes(u) ? n[u] = p : (l || (l = {}))[u] = p : ds(e.emitsOptions, a) || (!(a in s) || p !== s[a]) && (s[a] = p, o = true);
  }
  if (r) {
    const a = fe(n), p = l || me;
    for (let u = 0; u < r.length; u++) {
      const c = r[u];
      n[c] = Fs(i, a, c, p[c], e, !de(p, c));
    }
  }
  return o;
}
function Fs(e, t, n, s, i, r) {
  const o = e[n];
  if (o != null) {
    const l = de(o, "default");
    if (l && s === void 0) {
      const a = o.default;
      if (o.type !== Function && !o.skipFactory && ee(a)) {
        const { propsDefaults: p } = i;
        if (n in p) s = p[n];
        else {
          const u = jn(i);
          s = p[n] = a.call(null, t), u();
        }
      } else s = a;
      i.ce && i.ce._setProp(n, s);
    }
    o[0] && (r && !l ? s = false : o[1] && (s === "" || s === Mt(n)) && (s = true));
  }
  return s;
}
const ba = /* @__PURE__ */ new WeakMap();
function ho(e, t, n = false) {
  const s = n ? ba : t.propsCache, i = s.get(e);
  if (i) return i;
  const r = e.props, o = {}, l = [];
  let a = false;
  if (!ee(e)) {
    const u = (c) => {
      a = true;
      const [v, m] = ho(c, t, true);
      Oe(o, v), m && l.push(...m);
    };
    !n && t.mixins.length && t.mixins.forEach(u), e.extends && u(e.extends), e.mixins && e.mixins.forEach(u);
  }
  if (!r && !a) return pe(e) && s.set(e, Xt), Xt;
  if (Q(r)) for (let u = 0; u < r.length; u++) {
    const c = Ge(r[u]);
    ji(c) && (o[c] = me);
  }
  else if (r) for (const u in r) {
    const c = Ge(u);
    if (ji(c)) {
      const v = r[u], m = o[c] = Q(v) || ee(v) ? { type: v } : Oe({}, v), b = m.type;
      let _ = false, P = true;
      if (Q(b)) for (let O = 0; O < b.length; ++O) {
        const w = b[O], I = ee(w) && w.name;
        if (I === "Boolean") {
          _ = true;
          break;
        } else I === "String" && (P = false);
      }
      else _ = ee(b) && b.name === "Boolean";
      m[0] = _, m[1] = P, (_ || de(m, "default")) && l.push(c);
    }
  }
  const p = [o, l];
  return pe(e) && s.set(e, p), p;
}
function ji(e) {
  return e[0] !== "$" && !bn(e);
}
const gi = (e) => e === "_" || e === "_ctx" || e === "$stable", mi = (e) => Q(e) ? e.map(ft) : [ft(e)], wa = (e, t, n) => {
  if (t._n) return t;
  const s = Ht((...i) => mi(t(...i)), n);
  return s._c = false, s;
}, vo = (e, t, n) => {
  const s = e._ctx;
  for (const i in e) {
    if (gi(i)) continue;
    const r = e[i];
    if (ee(r)) t[i] = wa(i, r, s);
    else if (r != null) {
      const o = mi(r);
      t[i] = () => o;
    }
  }
}, go = (e, t) => {
  const n = mi(t);
  e.slots.default = () => n;
}, mo = (e, t, n) => {
  for (const s in t) (n || !gi(s)) && (e[s] = t[s]);
}, Sa = (e, t, n) => {
  const s = e.slots = uo();
  if (e.vnode.shapeFlag & 32) {
    const i = t._;
    i ? (mo(s, t, n), n && Ar(s, "_", i, true)) : vo(t, s);
  } else t && go(e, t);
}, ka = (e, t, n) => {
  const { vnode: s, slots: i } = e;
  let r = true, o = me;
  if (s.shapeFlag & 32) {
    const l = t._;
    l ? n && l === 1 ? r = false : mo(i, t, n) : (r = !t.$stable, vo(t, i)), o = t;
  } else t && (go(e, t), o = { default: 1 });
  if (r) for (const l in i) !gi(l) && o[l] == null && delete i[l];
}, We = Ra;
function Ea(e) {
  return Ca(e);
}
function Ca(e, t) {
  const n = ls();
  n.__VUE__ = true;
  const { insert: s, remove: i, patchProp: r, createElement: o, createText: l, createComment: a, setText: p, setElementText: u, parentNode: c, nextSibling: v, setScopeId: m = pt, insertStaticContent: b } = e, _ = (d, h, y, x = null, A = null, E = null, U = void 0, D = null, M = !!h.dynamicChildren) => {
    if (d === h) return;
    d && !pn(d, h) && (x = g(d), W(d, A, E, true), d = null), h.patchFlag === -2 && (M = false, h.dynamicChildren = null);
    const { type: R, ref: G, shapeFlag: H } = h;
    switch (R) {
      case ps:
        P(d, h, y, x);
        break;
      case Nt:
        O(d, h, y, x);
        break;
      case qn:
        d == null && w(h, y, x, U);
        break;
      case ce:
        Qe(d, h, y, x, A, E, U, D, M);
        break;
      default:
        H & 1 ? ie(d, h, y, x, A, E, U, D, M) : H & 6 ? De(d, h, y, x, A, E, U, D, M) : (H & 64 || H & 128) && R.process(d, h, y, x, A, E, U, D, M, B);
    }
    G != null && A ? kn(G, d && d.ref, E, h || d, !h) : G == null && d && d.ref != null && kn(d.ref, null, E, d, true);
  }, P = (d, h, y, x) => {
    if (d == null) s(h.el = l(h.children), y, x);
    else {
      const A = h.el = d.el;
      h.children !== d.children && p(A, h.children);
    }
  }, O = (d, h, y, x) => {
    d == null ? s(h.el = a(h.children || ""), y, x) : h.el = d.el;
  }, w = (d, h, y, x) => {
    [d.el, d.anchor] = b(d.children, h, y, x, d.el, d.anchor);
  }, I = ({ el: d, anchor: h }, y, x) => {
    let A;
    for (; d && d !== h; ) A = v(d), s(d, y, x), d = A;
    s(h, y, x);
  }, F = ({ el: d, anchor: h }) => {
    let y;
    for (; d && d !== h; ) y = v(d), i(d), d = y;
    i(h);
  }, ie = (d, h, y, x, A, E, U, D, M) => {
    if (h.type === "svg" ? U = "svg" : h.type === "math" && (U = "mathml"), d == null) he(h, y, x, A, E, U, D, M);
    else {
      const R = d.el && d.el._isVueCE ? d.el : null;
      try {
        R && R._beginPatch(), He(d, h, A, E, U, D, M);
      } finally {
        R && R._endPatch();
      }
    }
  }, he = (d, h, y, x, A, E, U, D) => {
    let M, R;
    const { props: G, shapeFlag: H, transition: q, dirs: Y } = d;
    if (M = d.el = o(d.type, E, G && G.is, G), H & 8 ? u(M, d.children) : H & 16 && Me(d.children, M, null, x, A, Cs(d, E), U, D), Y && jt(d, null, x, "created"), ae(M, d, d.scopeId, U, x), G) {
      for (const ge in G) ge !== "value" && !bn(ge) && r(M, ge, null, G[ge], E, x);
      "value" in G && r(M, "value", null, G.value, E), (R = G.onVnodeBeforeMount) && at(R, x, d);
    }
    Y && jt(d, null, x, "beforeMount");
    const le = xa(A, q);
    le && q.beforeEnter(M), s(M, h, y), ((R = G && G.onVnodeMounted) || le || Y) && We(() => {
      try {
        R && at(R, x, d), le && q.enter(M), Y && jt(d, null, x, "mounted");
      } finally {
      }
    }, A);
  }, ae = (d, h, y, x, A) => {
    if (y && m(d, y), x) for (let E = 0; E < x.length; E++) m(d, x[E]);
    if (A) {
      let E = A.subTree;
      if (h === E || wo(E.type) && (E.ssContent === h || E.ssFallback === h)) {
        const U = A.vnode;
        ae(d, U, U.scopeId, U.slotScopeIds, A.parent);
      }
    }
  }, Me = (d, h, y, x, A, E, U, D, M = 0) => {
    for (let R = M; R < d.length; R++) {
      const G = d[R] = D ? bt(d[R]) : ft(d[R]);
      _(null, G, h, y, x, A, E, U, D);
    }
  }, He = (d, h, y, x, A, E, U) => {
    const D = h.el = d.el;
    let { patchFlag: M, dynamicChildren: R, dirs: G } = h;
    M |= d.patchFlag & 16;
    const H = d.props || me, q = h.props || me;
    let Y;
    if (y && Ut(y, false), (Y = q.onVnodeBeforeUpdate) && at(Y, y, h, d), G && jt(h, d, y, "beforeUpdate"), y && Ut(y, true), R && (!d.dynamicChildren || d.dynamicChildren.length !== R.length) && (M = 0, U = false, R = null), (H.innerHTML && q.innerHTML == null || H.textContent && q.textContent == null) && u(D, ""), R ? Fe(d.dynamicChildren, R, D, y, x, Cs(h, A), E) : U || te(d, h, D, null, y, x, Cs(h, A), E, false), M > 0) {
      if (M & 16) ze(D, H, q, y, A);
      else if (M & 2 && H.class !== q.class && r(D, "class", null, q.class, A), M & 4 && r(D, "style", H.style, q.style, A), M & 8) {
        const le = h.dynamicProps;
        for (let ge = 0; ge < le.length; ge++) {
          const ve = le[ge], Ee = H[ve], Re = q[ve];
          (Re !== Ee || ve === "value") && r(D, ve, Ee, Re, A, y);
        }
      }
      M & 1 && d.children !== h.children && u(D, h.children);
    } else !U && R == null && ze(D, H, q, y, A);
    ((Y = q.onVnodeUpdated) || G) && We(() => {
      Y && at(Y, y, h, d), G && jt(h, d, y, "updated");
    }, x);
  }, Fe = (d, h, y, x, A, E, U) => {
    for (let D = 0; D < h.length; D++) {
      const M = d[D], R = h[D], G = M.el && (M.type === ce || !pn(M, R) || M.shapeFlag & 198) ? c(M.el) : y;
      _(M, R, G, null, x, A, E, U, true);
    }
  }, ze = (d, h, y, x, A) => {
    if (h !== y) {
      if (h !== me) for (const E in h) !bn(E) && !(E in y) && r(d, E, h[E], null, A, x);
      for (const E in y) {
        if (bn(E)) continue;
        const U = y[E], D = h[E];
        U !== D && E !== "value" && r(d, E, D, U, A, x);
      }
      "value" in y && r(d, "value", h.value, y.value, A);
    }
  }, Qe = (d, h, y, x, A, E, U, D, M) => {
    const R = h.el = d ? d.el : l(""), G = h.anchor = d ? d.anchor : l("");
    let { patchFlag: H, dynamicChildren: q, slotScopeIds: Y } = h;
    Y && (D = D ? D.concat(Y) : Y), d == null ? (s(R, y, x), s(G, y, x), Me(h.children || [], y, G, A, E, U, D, M)) : H > 0 && H & 64 && q && d.dynamicChildren && d.dynamicChildren.length === q.length ? (Fe(d.dynamicChildren, q, y, A, E, U, D), (h.key != null || A && h === A.subTree) && yo(d, h, true)) : te(d, h, y, G, A, E, U, D, M);
  }, De = (d, h, y, x, A, E, U, D, M) => {
    h.slotScopeIds = D, d == null ? h.shapeFlag & 512 ? A.ctx.activate(h, y, x, U, M) : st(h, y, x, A, E, U, M) : it(d, h, M);
  }, st = (d, h, y, x, A, E, U) => {
    const D = d.component = Da(d, x, A);
    if (no(d) && (D.ctx.renderer = B), ja(D, false, U), D.asyncDep) {
      if (A && A.registerDep(D, be, U), !d.el) {
        const M = D.subTree = we(Nt);
        O(null, M, h, y), d.placeholder = M.el;
      }
    } else be(D, d, h, y, A, E, U);
  }, it = (d, h, y) => {
    const x = h.component = d.component;
    if (ga(d, h, y)) if (x.asyncDep && !x.asyncResolved) {
      oe(x, h, y);
      return;
    } else x.next = h, x.update();
    else h.el = d.el, x.vnode = h;
  }, be = (d, h, y, x, A, E, U) => {
    const D = () => {
      if (d.isMounted) {
        let { next: H, bu: q, u: Y, parent: le, vnode: ge } = d;
        {
          const ot = _o(d);
          if (ot) {
            H && (H.el = ge.el, oe(d, H, U)), ot.asyncDep.then(() => {
              We(() => {
                d.isUnmounted || R();
              }, A);
            });
            return;
          }
        }
        let ve = H, Ee;
        Ut(d, false), H ? (H.el = ge.el, oe(d, H, U)) : H = ge, q && Wn(q), (Ee = H.props && H.props.onVnodeBeforeUpdate) && at(Ee, le, H, ge), Ut(d, true);
        const Re = Di(d), rt = d.subTree;
        d.subTree = Re, _(rt, Re, c(rt.el), g(rt), d, A, E), H.el = Re.el, ve === null && ma(d, Re.el), Y && We(Y, A), (Ee = H.props && H.props.onVnodeUpdated) && We(() => at(Ee, le, H, ge), A);
      } else {
        let H;
        const { el: q, props: Y } = h, { bm: le, m: ge, parent: ve, root: Ee, type: Re } = d, rt = En(h);
        Ut(d, false), le && Wn(le), !rt && (H = Y && Y.onVnodeBeforeMount) && at(H, ve, h), Ut(d, true);
        {
          Ee.ce && Ee.ce._hasShadowRoot() && Ee.ce._injectChildStyle(Re, d.parent ? d.parent.type : void 0);
          const ot = d.subTree = Di(d);
          _(null, ot, y, x, d, A, E), h.el = ot.el;
        }
        if (ge && We(ge, A), !rt && (H = Y && Y.onVnodeMounted)) {
          const ot = h;
          We(() => at(H, ve, ot), A);
        }
        (h.shapeFlag & 256 || ve && En(ve.vnode) && ve.vnode.shapeFlag & 256) && d.a && We(d.a, A), d.isMounted = true, h = y = x = null;
      }
    };
    d.scope.on();
    const M = d.effect = new $r(D);
    d.scope.off();
    const R = d.update = M.run.bind(M), G = d.job = M.runIfDirty.bind(M);
    G.i = d, G.id = d.uid, M.scheduler = () => hi(G), Ut(d, true), R();
  }, oe = (d, h, y) => {
    h.component = d;
    const x = d.vnode.props;
    d.vnode = h, d.next = null, _a(d, h.props, x, y), ka(d, h.children, y), vt(), Ri(d), gt();
  }, te = (d, h, y, x, A, E, U, D, M = false) => {
    const R = d && d.children, G = d ? d.shapeFlag : 0, H = h.children, { patchFlag: q, shapeFlag: Y } = h;
    if (q > 0) {
      if (q & 128) {
        N(R, H, y, x, A, E, U, D, M);
        return;
      } else if (q & 256) {
        Le(R, H, y, x, A, E, U, D, M);
        return;
      }
    }
    Y & 8 ? (G & 16 && Z(R, A, E), H !== R && u(y, H)) : G & 16 ? Y & 16 ? N(R, H, y, x, A, E, U, D, M) : Z(R, A, E, true) : (G & 8 && u(y, ""), Y & 16 && Me(H, y, x, A, E, U, D, M));
  }, Le = (d, h, y, x, A, E, U, D, M) => {
    d = d || Xt, h = h || Xt;
    const R = d.length, G = h.length, H = Math.min(R, G);
    let q;
    for (q = 0; q < H; q++) {
      const Y = h[q] = M ? bt(h[q]) : ft(h[q]);
      _(d[q], Y, y, null, A, E, U, D, M);
    }
    R > G ? Z(d, A, E, true, false, H) : Me(h, y, x, A, E, U, D, M, H);
  }, N = (d, h, y, x, A, E, U, D, M) => {
    let R = 0;
    const G = h.length;
    let H = d.length - 1, q = G - 1;
    for (; R <= H && R <= q; ) {
      const Y = d[R], le = h[R] = M ? bt(h[R]) : ft(h[R]);
      if (pn(Y, le)) _(Y, le, y, null, A, E, U, D, M);
      else break;
      R++;
    }
    for (; R <= H && R <= q; ) {
      const Y = d[H], le = h[q] = M ? bt(h[q]) : ft(h[q]);
      if (pn(Y, le)) _(Y, le, y, null, A, E, U, D, M);
      else break;
      H--, q--;
    }
    if (R > H) {
      if (R <= q) {
        const Y = q + 1, le = Y < G ? h[Y].el : x;
        for (; R <= q; ) _(null, h[R] = M ? bt(h[R]) : ft(h[R]), y, le, A, E, U, D, M), R++;
      }
    } else if (R > q) for (; R <= H; ) W(d[R], A, E, true), R++;
    else {
      const Y = R, le = R, ge = /* @__PURE__ */ new Map();
      for (R = le; R <= q; R++) {
        const Be = h[R] = M ? bt(h[R]) : ft(h[R]);
        Be.key != null && ge.set(Be.key, R);
      }
      let ve, Ee = 0;
      const Re = q - le + 1;
      let rt = false, ot = 0;
      const fn = new Array(Re);
      for (R = 0; R < Re; R++) fn[R] = 0;
      for (R = Y; R <= H; R++) {
        const Be = d[R];
        if (Ee >= Re) {
          W(Be, A, E, true);
          continue;
        }
        let lt;
        if (Be.key != null) lt = ge.get(Be.key);
        else for (ve = le; ve <= q; ve++) if (fn[ve - le] === 0 && pn(Be, h[ve])) {
          lt = ve;
          break;
        }
        lt === void 0 ? W(Be, A, E, true) : (fn[lt - le] = R + 1, lt >= ot ? ot = lt : rt = true, _(Be, h[lt], y, null, A, E, U, D, M), Ee++);
      }
      const wi = rt ? Aa(fn) : Xt;
      for (ve = wi.length - 1, R = Re - 1; R >= 0; R--) {
        const Be = le + R, lt = h[Be], Si = h[Be + 1], ki = Be + 1 < G ? Si.el || bo(Si) : x;
        fn[R] === 0 ? _(null, lt, y, ki, A, E, U, D, M) : rt && (ve < 0 || R !== wi[ve] ? j(lt, y, ki, 2) : ve--);
      }
    }
  }, j = (d, h, y, x, A = null) => {
    const { el: E, type: U, transition: D, children: M, shapeFlag: R } = d;
    if (R & 6) {
      j(d.component.subTree, h, y, x);
      return;
    }
    if (R & 128) {
      d.suspense.move(h, y, x);
      return;
    }
    if (R & 64) {
      U.move(d, h, y, B);
      return;
    }
    if (U === ce) {
      s(E, h, y);
      for (let H = 0; H < M.length; H++) j(M[H], h, y, x);
      s(d.anchor, h, y);
      return;
    }
    if (U === qn) {
      I(d, h, y);
      return;
    }
    if (x !== 2 && R & 1 && D) if (x === 0) D.persisted && !E[ks] ? s(E, h, y) : (D.beforeEnter(E), s(E, h, y), We(() => D.enter(E), A));
    else {
      const { leave: H, delayLeave: q, afterLeave: Y } = D, le = () => {
        d.ctx.isUnmounted ? i(E) : s(E, h, y);
      }, ge = () => {
        const ve = E._isLeaving || !!E[ks];
        E._isLeaving && E[ks](true), D.persisted && !ve ? le() : H(E, () => {
          le(), Y && Y();
        });
      };
      q ? q(E, le, ge) : ge();
    }
    else s(E, h, y);
  }, W = (d, h, y, x = false, A = false) => {
    const { type: E, props: U, ref: D, children: M, dynamicChildren: R, shapeFlag: G, patchFlag: H, dirs: q, cacheIndex: Y, memo: le } = d;
    if (H === -2 && (A = false), D != null && (vt(), kn(D, null, y, d, true), gt()), Y != null && (h.renderCache[Y] = void 0), G & 256) {
      h.ctx.deactivate(d);
      return;
    }
    const ge = G & 1 && q, ve = !En(d);
    let Ee;
    if (ve && (Ee = U && U.onVnodeBeforeUnmount) && at(Ee, h, d), G & 6) $(d.component, y, x);
    else {
      if (G & 128) {
        d.suspense.unmount(y, x);
        return;
      }
      ge && jt(d, null, h, "beforeUnmount"), G & 64 ? d.type.remove(d, h, y, B, x) : R && !R.hasOnce && (E !== ce || H > 0 && H & 64) ? Z(R, h, y, false, true) : (E === ce && H & 384 || !A && G & 16) && Z(M, h, y), x && ne(d);
    }
    const Re = le != null && Y == null;
    (ve && (Ee = U && U.onVnodeUnmounted) || ge || Re) && We(() => {
      Ee && at(Ee, h, d), ge && jt(d, null, h, "unmounted"), Re && (d.el = null);
    }, y);
  }, ne = (d) => {
    const { type: h, el: y, anchor: x, transition: A } = d;
    if (h === ce) {
      T(y, x);
      return;
    }
    if (h === qn) {
      F(d);
      return;
    }
    const E = () => {
      i(y), A && !A.persisted && A.afterLeave && A.afterLeave();
    };
    if (d.shapeFlag & 1 && A && !A.persisted) {
      const { leave: U, delayLeave: D } = A, M = () => U(y, E);
      D ? D(d.el, E, M) : M();
    } else E();
  }, T = (d, h) => {
    let y;
    for (; d !== h; ) y = v(d), i(d), d = y;
    i(h);
  }, $ = (d, h, y) => {
    const { bum: x, scope: A, job: E, subTree: U, um: D, m: M, a: R } = d;
    Ui(M), Ui(R), x && Wn(x), A.stop(), E && (E.flags |= 8, W(U, d, h, y)), D && We(D, h), We(() => {
      d.isUnmounted = true;
    }, h);
  }, Z = (d, h, y, x = false, A = false, E = 0) => {
    for (let U = E; U < d.length; U++) W(d[U], h, y, x, A);
  }, g = (d) => {
    if (d.shapeFlag & 6) return g(d.component.subTree);
    if (d.shapeFlag & 128) return d.suspense.next();
    const h = v(d.anchor || d.el), y = h && h[Vl];
    return y ? v(y) : h;
  };
  let V = false;
  const L = (d, h, y) => {
    let x;
    d == null ? h._vnode && (W(h._vnode, null, null, true), x = h._vnode.component) : _(h._vnode || null, d, h, null, null, null, y), h._vnode = d, V || (V = true, Ri(x), Qr(), V = false);
  }, B = { p: _, um: W, m: j, r: ne, mt: st, mc: Me, pc: te, pbc: Fe, n: g, o: e };
  return { render: L, hydrate: void 0, createApp: ua(L) };
}
function Cs({ type: e, props: t }, n) {
  return n === "svg" && e === "foreignObject" || n === "mathml" && e === "annotation-xml" && t && t.encoding && t.encoding.includes("html") ? void 0 : n;
}
function Ut({ effect: e, job: t }, n) {
  n ? (e.flags |= 32, t.flags |= 4) : (e.flags &= -33, t.flags &= -5);
}
function xa(e, t) {
  return (!e || e && !e.pendingBranch) && t && !t.persisted;
}
function yo(e, t, n = false) {
  const s = e.children, i = t.children;
  if (Q(s) && Q(i)) for (let r = 0; r < s.length; r++) {
    const o = s[r];
    let l = i[r];
    l.shapeFlag & 1 && !l.dynamicChildren && ((l.patchFlag <= 0 || l.patchFlag === 32) && (l = i[r] = bt(i[r]), l.el = o.el), !n && l.patchFlag !== -2 && yo(o, l)), l.type === ps && (l.patchFlag === -1 && (l = i[r] = bt(l)), l.el = o.el), l.type === Nt && !l.el && (l.el = o.el);
  }
}
function Aa(e) {
  const t = e.slice(), n = [0];
  let s, i, r, o, l;
  const a = e.length;
  for (s = 0; s < a; s++) {
    const p = e[s];
    if (p !== 0) {
      if (i = n[n.length - 1], e[i] < p) {
        t[s] = i, n.push(s);
        continue;
      }
      for (r = 0, o = n.length - 1; r < o; ) l = r + o >> 1, e[n[l]] < p ? r = l + 1 : o = l;
      p < e[n[r]] && (r > 0 && (t[s] = n[r - 1]), n[r] = s);
    }
  }
  for (r = n.length, o = n[r - 1]; r-- > 0; ) n[r] = o, o = t[o];
  return n;
}
function _o(e) {
  const t = e.subTree.component;
  if (t) return t.asyncDep && !t.asyncResolved ? t : _o(t);
}
function Ui(e) {
  if (e) for (let t = 0; t < e.length; t++) e[t].flags |= 8;
}
function bo(e) {
  if (e.placeholder) return e.placeholder;
  const t = e.component;
  return t ? bo(t.subTree) : null;
}
const wo = (e) => e.__isSuspense;
function Ra(e, t) {
  t && t.pendingBranch ? Q(e) ? t.effects.push(...e) : t.effects.push(e) : Dl(e);
}
const ce = Symbol.for("v-fgt"), ps = Symbol.for("v-txt"), Nt = Symbol.for("v-cmt"), qn = Symbol.for("v-stc"), xn = [];
let Ke = null;
function k(e = false) {
  xn.push(Ke = e ? null : []);
}
function Pa() {
  xn.pop(), Ke = xn[xn.length - 1] || null;
}
let $n = 1;
function Zn(e, t = false) {
  $n += e, e < 0 && Ke && t && (Ke.hasOnce = true);
}
function So(e) {
  return e.dynamicChildren = $n > 0 ? Ke || Xt : null, Pa(), $n > 0 && Ke && Ke.push(e), e;
}
function C(e, t, n, s, i, r) {
  return So(f(e, t, n, s, i, r, true));
}
function Oa(e, t, n, s, i) {
  return So(we(e, t, n, s, i, true));
}
function es(e) {
  return e ? e.__v_isVNode === true : false;
}
function pn(e, t) {
  return e.type === t.type && e.key === t.key;
}
const ko = ({ key: e }) => e ?? null, Kn = ({ ref: e, ref_key: t, ref_for: n }) => (typeof e == "number" && (e = "" + e), e != null ? ke(e) || Ne(e) || ee(e) ? { i: qe, r: e, k: t, f: !!n } : e : null);
function f(e, t = null, n = null, s = 0, i = null, r = e === ce ? 0 : 1, o = false, l = false) {
  const a = { __v_isVNode: true, __v_skip: true, type: e, props: t, key: t && ko(t), ref: t && Kn(t), scopeId: Xr, slotScopeIds: null, children: n, component: null, suspense: null, ssContent: null, ssFallback: null, dirs: null, transition: null, el: null, anchor: null, target: null, targetStart: null, targetAnchor: null, staticCount: 0, shapeFlag: r, patchFlag: s, dynamicProps: i, dynamicChildren: null, appContext: null, ctx: qe };
  return l ? (ts(a, n), r & 128 && e.normalize(a)) : n && (a.shapeFlag |= ke(n) ? 8 : 16), $n > 0 && !o && Ke && (a.patchFlag > 0 || r & 6) && a.patchFlag !== 32 && Ke.push(a), a;
}
const we = $a;
function $a(e, t = null, n = null, s = 0, i = null, r = false) {
  if ((!e || e === ea) && (e = Nt), es(e)) {
    const l = sn(e, t, true);
    return n && ts(l, n), $n > 0 && !r && Ke && (l.shapeFlag & 6 ? Ke[Ke.indexOf(e)] = l : Ke.push(l)), l.patchFlag = -2, l;
  }
  if (Fa(e) && (e = e.__vccOpts), t) {
    t = Ia(t);
    let { class: l, style: a } = t;
    l && !ke(l) && (t.class = $e(l)), pe(a) && (di(a) && !Q(a) && (a = Oe({}, a)), t.style = Tt(a));
  }
  const o = ke(e) ? 1 : wo(e) ? 128 : Hl(e) ? 64 : pe(e) ? 4 : ee(e) ? 2 : 0;
  return f(e, t, n, s, i, o, r, true);
}
function Ia(e) {
  return e ? di(e) || fo(e) ? Oe({}, e) : e : null;
}
function sn(e, t, n = false, s = false) {
  const { props: i, ref: r, patchFlag: o, children: l, transition: a } = e, p = t ? Ta(i || {}, t) : i, u = { __v_isVNode: true, __v_skip: true, type: e.type, props: p, key: p && ko(p), ref: t && t.ref ? n && r ? Q(r) ? r.concat(Kn(t)) : [r, Kn(t)] : Kn(t) : r, scopeId: e.scopeId, slotScopeIds: e.slotScopeIds, children: l, target: e.target, targetStart: e.targetStart, targetAnchor: e.targetAnchor, staticCount: e.staticCount, shapeFlag: e.shapeFlag, patchFlag: t && e.type !== ce ? o === -1 ? 16 : o | 16 : o, dynamicProps: e.dynamicProps, dynamicChildren: e.dynamicChildren, appContext: e.appContext, dirs: e.dirs, transition: a, component: e.component, suspense: e.suspense, ssContent: e.ssContent && sn(e.ssContent), ssFallback: e.ssFallback && sn(e.ssFallback), placeholder: e.placeholder, el: e.el, anchor: e.anchor, ctx: e.ctx, ce: e.ce };
  return a && s && vi(u, a.clone(u)), u;
}
function ye(e = " ", t = 0) {
  return we(ps, null, e, t);
}
function Eo(e, t) {
  const n = we(qn, null, e);
  return n.staticCount = t, n;
}
function J(e = "", t = false) {
  return t ? (k(), Oa(Nt, null, e)) : we(Nt, null, e);
}
function ft(e) {
  return e == null || typeof e == "boolean" ? we(Nt) : Q(e) ? we(ce, null, e.slice()) : es(e) ? bt(e) : we(ps, null, String(e));
}
function bt(e) {
  return e.el === null && e.patchFlag !== -1 || e.memo ? e : sn(e);
}
function ts(e, t) {
  let n = 0;
  const { shapeFlag: s } = e;
  if (t == null) t = null;
  else if (Q(t)) n = 16;
  else if (typeof t == "object") if (s & 65) {
    const i = t.default;
    i && (i._c && (i._d = false), ts(e, i()), i._c && (i._d = true));
    return;
  } else {
    n = 32;
    const i = t._;
    !i && !fo(t) ? t._ctx = qe : i === 3 && qe && (qe.slots._ === 1 ? t._ = 1 : (t._ = 2, e.patchFlag |= 1024));
  }
  else if (ee(t)) {
    if (s & 65) {
      ts(e, { default: t });
      return;
    }
    t = { default: t, _ctx: qe }, n = 32;
  } else t = String(t), s & 64 ? (n = 16, t = [ye(t)]) : n = 8;
  e.children = t, e.shapeFlag |= n;
}
function Ta(...e) {
  const t = {};
  for (let n = 0; n < e.length; n++) {
    const s = e[n];
    for (const i in s) if (i === "class") t.class !== s.class && (t.class = $e([t.class, s.class]));
    else if (i === "style") t.style = Tt([t.style, s.style]);
    else if (ss(i)) {
      const r = t[i], o = s[i];
      o && r !== o && !(Q(r) && r.includes(o)) ? t[i] = r ? [].concat(r, o) : o : o == null && r == null && !is(i) && (t[i] = o);
    } else i !== "" && (t[i] = s[i]);
  }
  return t;
}
function at(e, t, n, s = null) {
  et(e, t, 7, [n, s]);
}
const Na = oo();
let Ma = 0;
function Da(e, t, n) {
  const s = e.type, i = (t ? t.appContext : e.appContext) || Na, r = { uid: Ma++, vnode: e, type: s, parent: t, appContext: i, root: null, next: null, subTree: null, effect: null, update: null, job: null, scope: new ll(true), render: null, proxy: null, exposed: null, exposeProxy: null, withProxy: null, provides: t ? t.provides : Object.create(i.provides), ids: t ? t.ids : ["", 0, 0], accessCache: null, renderCache: [], components: null, directives: null, propsOptions: ho(s, i), emitsOptions: lo(s, i), emit: null, emitted: null, propsDefaults: me, inheritAttrs: s.inheritAttrs, ctx: me, data: me, props: me, attrs: me, slots: me, refs: me, setupState: me, setupContext: null, suspense: n, suspenseId: n ? n.pendingId : 0, asyncDep: null, asyncResolved: false, isMounted: false, isUnmounted: false, isDeactivated: false, bc: null, c: null, bm: null, m: null, bu: null, u: null, um: null, bum: null, da: null, a: null, rtg: null, rtc: null, ec: null, sp: null };
  return r.ctx = { _: r }, r.root = t ? t.root : r, r.emit = da.bind(null, r), e.ce && e.ce(r), r;
}
let Te = null;
const La = () => Te || qe;
let ns, Ws;
{
  const e = ls(), t = (n, s) => {
    let i;
    return (i = e[n]) || (i = e[n] = []), i.push(s), (r) => {
      i.length > 1 ? i.forEach((o) => o(r)) : i[0](r);
    };
  };
  ns = t("__VUE_INSTANCE_SETTERS__", (n) => Te = n), Ws = t("__VUE_SSR_SETTERS__", (n) => In = n);
}
const jn = (e) => {
  const t = Te;
  return ns(e), e.scope.on(), () => {
    e.scope.off(), ns(t);
  };
}, Vi = () => {
  Te && Te.scope.off(), ns(null);
};
function Co(e) {
  return e.vnode.shapeFlag & 4;
}
let In = false;
function ja(e, t = false, n = false) {
  t && Ws(t);
  const { props: s, children: i } = e.vnode, r = Co(e);
  ya(e, s, r, t), Sa(e, i, n || t);
  const o = r ? Ua(e, t) : void 0;
  return t && Ws(false), o;
}
function Ua(e, t) {
  const n = e.type;
  e.accessCache = /* @__PURE__ */ Object.create(null), e.proxy = new Proxy(e.ctx, sa);
  const { setup: s } = n;
  if (s) {
    vt();
    const i = e.setupContext = s.length > 1 ? Ha(e) : null, r = jn(e), o = Dn(s, e, 0, [e.props, i]), l = Er(o);
    if (gt(), r(), (l || e.sp) && !En(e) && to(e), l) {
      if (o.then(Vi, Vi), t) return o.then((a) => {
        Hi(e, a);
      }).catch((a) => {
        cs(a, e, 0);
      });
      e.asyncDep = o;
    } else Hi(e, o);
  } else xo(e);
}
function Hi(e, t, n) {
  ee(t) ? e.type.__ssrInlineRender ? e.ssrRender = t : e.render = t : pe(t) && (e.setupState = Gr(t)), xo(e);
}
function xo(e, t, n) {
  const s = e.type;
  e.render || (e.render = s.render || pt);
  {
    const i = jn(e);
    vt();
    try {
      ia(e);
    } finally {
      gt(), i();
    }
  }
}
const Va = { get(e, t) {
  return Ie(e, "get", ""), e[t];
} };
function Ha(e) {
  const t = (n) => {
    e.exposed = n || {};
  };
  return { attrs: new Proxy(e.attrs, Va), slots: e.slots, emit: e.emit, expose: t };
}
function hs(e) {
  return e.exposed ? e.exposeProxy || (e.exposeProxy = new Proxy(Gr(xl(e.exposed)), { get(t, n) {
    if (n in t) return t[n];
    if (n in Cn) return Cn[n](e);
  }, has(t, n) {
    return n in t || n in Cn;
  } })) : e.proxy;
}
function Fa(e) {
  return ee(e) && "__vccOpts" in e;
}
const se = (e, t) => $l(e, t, In);
function Ao(e, t, n) {
  try {
    Zn(-1);
    const s = arguments.length;
    return s === 2 ? pe(t) && !Q(t) ? es(t) ? we(e, null, [t]) : we(e, t) : we(e, null, t) : (s > 3 ? n = Array.prototype.slice.call(arguments, 2) : s === 3 && es(n) && (n = [n]), we(e, t, n));
  } finally {
    Zn(1);
  }
}
const Wa = "3.5.39";
/**
* @vue/runtime-dom v3.5.39
* (c) 2018-present Yuxi (Evan) You and Vue contributors
* @license MIT
**/
let Bs;
const Fi = typeof window < "u" && window.trustedTypes;
if (Fi) try {
  Bs = Fi.createPolicy("vue", { createHTML: (e) => e });
} catch {
}
const Ro = Bs ? (e) => Bs.createHTML(e) : (e) => e, Ba = "http://www.w3.org/2000/svg", qa = "http://www.w3.org/1998/Math/MathML", _t = typeof document < "u" ? document : null, Wi = _t && _t.createElement("template"), Ka = { insert: (e, t, n) => {
  t.insertBefore(e, n || null);
}, remove: (e) => {
  const t = e.parentNode;
  t && t.removeChild(e);
}, createElement: (e, t, n, s) => {
  const i = t === "svg" ? _t.createElementNS(Ba, e) : t === "mathml" ? _t.createElementNS(qa, e) : n ? _t.createElement(e, { is: n }) : _t.createElement(e);
  return e === "select" && s && s.multiple != null && i.setAttribute("multiple", s.multiple), i;
}, createText: (e) => _t.createTextNode(e), createComment: (e) => _t.createComment(e), setText: (e, t) => {
  e.nodeValue = t;
}, setElementText: (e, t) => {
  e.textContent = t;
}, parentNode: (e) => e.parentNode, nextSibling: (e) => e.nextSibling, querySelector: (e) => _t.querySelector(e), setScopeId(e, t) {
  e.setAttribute(t, "");
}, insertStaticContent(e, t, n, s, i, r) {
  const o = n ? n.previousSibling : t.lastChild;
  if (i && (i === r || i.nextSibling)) for (; t.insertBefore(i.cloneNode(true), n), !(i === r || !(i = i.nextSibling)); ) ;
  else {
    Wi.innerHTML = Ro(s === "svg" ? `<svg>${e}</svg>` : s === "mathml" ? `<math>${e}</math>` : e);
    const l = Wi.content;
    if (s === "svg" || s === "mathml") {
      const a = l.firstChild;
      for (; a.firstChild; ) l.appendChild(a.firstChild);
      l.removeChild(a);
    }
    t.insertBefore(l, n);
  }
  return [o ? o.nextSibling : t.firstChild, n ? n.previousSibling : t.lastChild];
} }, Ga = Symbol("_vtc");
function za(e, t, n) {
  const s = e[Ga];
  s && (t = (t ? [t, ...s] : [...s]).join(" ")), t == null ? e.removeAttribute("class") : n ? e.setAttribute("class", t) : e.className = t;
}
const Bi = Symbol("_vod"), Ja = Symbol("_vsh"), Qa = Symbol(""), Ya = /(?:^|;)\s*display\s*:/;
function Xa(e, t, n) {
  const s = e.style, i = ke(n);
  let r = false;
  if (n && !i) {
    if (t) if (ke(t)) for (const o of t.split(";")) {
      const l = o.slice(0, o.indexOf(":")).trim();
      n[l] == null && gn(s, l, "");
    }
    else for (const o in t) n[o] == null && gn(s, o, "");
    for (const o in n) {
      o === "display" && (r = true);
      const l = n[o];
      l != null ? ec(e, o, !ke(t) && t ? t[o] : void 0, l) || gn(s, o, l) : gn(s, o, "");
    }
  } else if (i) {
    if (t !== n) {
      const o = s[Qa];
      o && (n += ";" + o), s.cssText = n, r = Ya.test(n);
    }
  } else t && e.removeAttribute("style");
  Bi in e && (e[Bi] = r ? s.display : "", e[Ja] && (s.display = "none"));
}
const qi = /\s*!important$/;
function gn(e, t, n) {
  if (Q(n)) n.forEach((s) => gn(e, t, s));
  else if (n == null && (n = ""), t.startsWith("--")) e.setProperty(t, n);
  else {
    const s = Za(e, t);
    qi.test(n) ? e.setProperty(Mt(s), n.replace(qi, ""), "important") : e[s] = n;
  }
}
const Ki = ["Webkit", "Moz", "ms"], xs = {};
function Za(e, t) {
  const n = xs[t];
  if (n) return n;
  let s = Ge(t);
  if (s !== "filter" && s in e) return xs[t] = s;
  s = ii(s);
  for (let i = 0; i < Ki.length; i++) {
    const r = Ki[i] + s;
    if (r in e) return xs[t] = r;
  }
  return t;
}
function ec(e, t, n, s) {
  return e.tagName === "TEXTAREA" && (t === "width" || t === "height") && ke(s) && n === s;
}
const Gi = "http://www.w3.org/1999/xlink";
function zi(e, t, n, s, i, r = rl(t)) {
  s && t.startsWith("xlink:") ? n == null ? e.removeAttributeNS(Gi, t.slice(6, t.length)) : e.setAttributeNS(Gi, t, n) : n == null || r && !Rr(n) ? e.removeAttribute(t) : e.setAttribute(t, r ? "" : ht(n) ? String(n) : n);
}
function Ji(e, t, n, s, i) {
  if (t === "innerHTML" || t === "textContent") {
    n != null && (e[t] = t === "innerHTML" ? Ro(n) : n);
    return;
  }
  const r = e.tagName;
  if (t === "value" && r !== "PROGRESS" && !r.includes("-")) {
    const l = r === "OPTION" ? e.getAttribute("value") || "" : e.value, a = n == null ? e.type === "checkbox" ? "on" : "" : String(n);
    (l !== a || !("_value" in e)) && (e.value = a), n == null && e.removeAttribute(t), e._value = n;
    return;
  }
  let o = false;
  if (n === "" || n == null) {
    const l = typeof e[t];
    l === "boolean" ? n = Rr(n) : n == null && l === "string" ? (n = "", o = true) : l === "number" && (n = 0, o = true);
  }
  try {
    e[t] = n;
  } catch {
  }
  o && e.removeAttribute(i || t);
}
function $t(e, t, n, s) {
  e.addEventListener(t, n, s);
}
function tc(e, t, n, s) {
  e.removeEventListener(t, n, s);
}
const Qi = Symbol("_vei");
function nc(e, t, n, s, i = null) {
  const r = e[Qi] || (e[Qi] = {}), o = r[t];
  if (s && o) o.value = s;
  else {
    const [l, a] = rc(t);
    if (s) {
      const p = r[t] = ac(s, i);
      $t(e, l, p, a);
    } else o && (tc(e, l, o, a), r[t] = void 0);
  }
}
const sc = /(Once|Passive|Capture)$/, ic = /^on:?(?:Once|Passive|Capture)$/;
function rc(e) {
  let t, n;
  for (; (n = e.match(sc)) && !ic.test(e); ) t || (t = {}), e = e.slice(0, e.length - n[1].length), t[n[1].toLowerCase()] = true;
  return [e[2] === ":" ? e.slice(3) : Mt(e.slice(2)), t];
}
let As = 0;
const oc = Promise.resolve(), lc = () => As || (oc.then(() => As = 0), As = Date.now());
function ac(e, t) {
  const n = (s) => {
    if (!s._vts) s._vts = Date.now();
    else if (s._vts <= n.attached) return;
    const i = n.value;
    if (Q(i)) {
      const r = s.stopImmediatePropagation;
      s.stopImmediatePropagation = () => {
        r.call(s), s._stopped = true;
      };
      const o = i.slice(), l = [s];
      for (let a = 0; a < o.length && !s._stopped; a++) {
        const p = o[a];
        p && et(p, t, 5, l);
      }
    } else et(i, t, 5, [s]);
  };
  return n.value = e, n.attached = lc(), n;
}
const Yi = (e) => e.charCodeAt(0) === 111 && e.charCodeAt(1) === 110 && e.charCodeAt(2) > 96 && e.charCodeAt(2) < 123, cc = (e, t, n, s, i, r) => {
  const o = i === "svg";
  t === "class" ? za(e, s, o) : t === "style" ? Xa(e, n, s) : ss(t) ? is(t) || nc(e, t, n, s, r) : (t[0] === "." ? (t = t.slice(1), true) : t[0] === "^" ? (t = t.slice(1), false) : uc(e, t, s, o)) ? (Ji(e, t, s), !e.tagName.includes("-") && (t === "value" || t === "checked" || t === "selected") && zi(e, t, s, o, r, t !== "value")) : e._isVueCE && (fc(e, t) || e._def.__asyncLoader && (/[A-Z]/.test(t) || !ke(s))) ? Ji(e, Ge(t), s, r, t) : (t === "true-value" ? e._trueValue = s : t === "false-value" && (e._falseValue = s), zi(e, t, s, o));
};
function uc(e, t, n, s) {
  if (s) return !!(t === "innerHTML" || t === "textContent" || t in e && Yi(t) && ee(n));
  if (t === "spellcheck" || t === "draggable" || t === "translate" || t === "autocorrect" || t === "sandbox" && e.tagName === "IFRAME" || t === "form" || t === "list" && e.tagName === "INPUT" || t === "type" && e.tagName === "TEXTAREA") return false;
  if (t === "width" || t === "height") {
    const i = e.tagName;
    if (i === "IMG" || i === "VIDEO" || i === "CANVAS" || i === "SOURCE") return false;
  }
  return Yi(t) && ke(n) ? false : t in e;
}
function fc(e, t) {
  const n = e._def.props;
  if (!n) return false;
  const s = Ge(t);
  return Array.isArray(n) ? n.some((i) => Ge(i) === s) : Object.keys(n).some((i) => Ge(i) === s);
}
const rn = (e) => {
  const t = e.props["onUpdate:modelValue"] || false;
  return Q(t) ? (n) => Wn(t, n) : t;
};
function dc(e) {
  e.target.composing = true;
}
function Xi(e) {
  const t = e.target;
  t.composing && (t.composing = false, t.dispatchEvent(new Event("input")));
}
const Et = Symbol("_assign");
function Zi(e, t, n) {
  return t && (e = e.trim()), n && (e = os(e)), e;
}
const Qt = { created(e, { modifiers: { lazy: t, trim: n, number: s } }, i) {
  e[Et] = rn(i);
  const r = s || i.props && i.props.type === "number";
  $t(e, t ? "change" : "input", (o) => {
    o.target.composing || e[Et](Zi(e.value, n, r));
  }), (n || r) && $t(e, "change", () => {
    e.value = Zi(e.value, n, r);
  }), t || ($t(e, "compositionstart", dc), $t(e, "compositionend", Xi), $t(e, "change", Xi));
}, mounted(e, { value: t }) {
  e.value = t ?? "";
}, beforeUpdate(e, { value: t, oldValue: n, modifiers: { lazy: s, trim: i, number: r } }, o) {
  if (e[Et] = rn(o), e.composing) return;
  const l = (r || e.type === "number") && !/^0\d/.test(e.value) ? os(e.value) : e.value, a = t ?? "";
  if (l === a) return;
  const p = e.getRootNode();
  (p instanceof Document || p instanceof ShadowRoot) && p.activeElement === e && e.type !== "range" && (s && t === n || i && e.value.trim() === a) || (e.value = a);
} }, Gn = { deep: true, created(e, t, n) {
  e[Et] = rn(n), $t(e, "change", () => {
    const s = e._modelValue, i = Tn(e), r = e.checked, o = e[Et];
    if (Q(s)) {
      const l = ri(s, i), a = l !== -1;
      if (r && !a) o(s.concat(i));
      else if (!r && a) {
        const p = [...s];
        p.splice(l, 1), o(p);
      }
    } else if (an(s)) {
      const l = new Set(s);
      r ? l.add(i) : l.delete(i), o(l);
    } else o(Po(e, r));
  });
}, mounted: er, beforeUpdate(e, t, n) {
  e[Et] = rn(n), er(e, t, n);
} };
function er(e, { value: t, oldValue: n }, s) {
  e._modelValue = t;
  let i;
  if (Q(t)) i = ri(t, s.props.value) > -1;
  else if (an(t)) i = t.has(s.props.value);
  else {
    if (t === n) return;
    i = cn(t, Po(e, true));
  }
  e.checked !== i && (e.checked = i);
}
const Kt = { deep: true, created(e, { value: t, modifiers: { number: n } }, s) {
  const i = an(t);
  $t(e, "change", () => {
    const r = Array.prototype.filter.call(e.options, (o) => o.selected).map((o) => n ? os(Tn(o)) : Tn(o));
    e[Et](e.multiple ? i ? new Set(r) : r : r[0]), e._assigning = true, pi(() => {
      e._assigning = false;
    });
  }), e[Et] = rn(s);
}, mounted(e, { value: t }) {
  tr(e, t);
}, beforeUpdate(e, t, n) {
  e[Et] = rn(n);
}, updated(e, { value: t }) {
  e._assigning || tr(e, t);
} };
function tr(e, t) {
  const n = e.multiple, s = Q(t);
  if (!(n && !s && !an(t))) {
    for (let i = 0, r = e.options.length; i < r; i++) {
      const o = e.options[i], l = Tn(o);
      if (n) if (s) {
        const a = typeof l;
        a === "string" || a === "number" ? o.selected = t.some((p) => String(p) === String(l)) : o.selected = ri(t, l) > -1;
      } else o.selected = t.has(l);
      else if (cn(Tn(o), t)) {
        e.selectedIndex !== i && (e.selectedIndex = i);
        return;
      }
    }
    !n && e.selectedIndex !== -1 && (e.selectedIndex = -1);
  }
}
function Tn(e) {
  return "_value" in e ? e._value : e.value;
}
function Po(e, t) {
  const n = t ? "_trueValue" : "_falseValue";
  return n in e ? e[n] : t;
}
const pc = ["ctrl", "shift", "alt", "meta"], hc = { stop: (e) => e.stopPropagation(), prevent: (e) => e.preventDefault(), self: (e) => e.target !== e.currentTarget, ctrl: (e) => !e.ctrlKey, shift: (e) => !e.shiftKey, alt: (e) => !e.altKey, meta: (e) => !e.metaKey, left: (e) => "button" in e && e.button !== 0, middle: (e) => "button" in e && e.button !== 1, right: (e) => "button" in e && e.button !== 2, exact: (e, t) => pc.some((n) => e[`${n}Key`] && !t.includes(n)) }, qs = (e, t) => {
  if (!e) return e;
  const n = e._withMods || (e._withMods = {}), s = t.join(".");
  return n[s] || (n[s] = ((i, ...r) => {
    for (let o = 0; o < t.length; o++) {
      const l = hc[t[o]];
      if (l && l(i, t)) return;
    }
    return e(i, ...r);
  }));
}, vc = { esc: "escape", space: " ", up: "arrow-up", left: "arrow-left", right: "arrow-right", down: "arrow-down", delete: "backspace" }, gc = (e, t) => {
  const n = e._withKeys || (e._withKeys = {}), s = t.join(".");
  return n[s] || (n[s] = ((i) => {
    if (!("key" in i)) return;
    const r = Mt(i.key);
    if (t.some((o) => o === r || vc[o] === r)) return e(i);
  }));
}, mc = Oe({ patchProp: cc }, Ka);
let nr;
function yc() {
  return nr || (nr = Ea(mc));
}
const _c = ((...e) => {
  const t = yc().createApp(...e), { mount: n } = t;
  return t.mount = (s) => {
    const i = wc(s);
    if (!i) return;
    const r = t._component;
    !ee(r) && !r.render && !r.template && (r.template = i.innerHTML), i.nodeType === 1 && (i.textContent = "");
    const o = n(i, false, bc(i));
    return i instanceof Element && (i.removeAttribute("v-cloak"), i.setAttribute("data-v-app", "")), o;
  }, t;
});
function bc(e) {
  if (e instanceof SVGElement) return "svg";
  if (typeof MathMLElement == "function" && e instanceof MathMLElement) return "mathml";
}
function wc(e) {
  return ke(e) ? document.querySelector(e) : e;
}
/*!
* vue-router v4.6.4
* (c) 2025 Eduardo San Martin Morote
* @license MIT
*/
const Yt = typeof document < "u";
function Oo(e) {
  return typeof e == "object" || "displayName" in e || "props" in e || "__vccOpts" in e;
}
function Sc(e) {
  return e.__esModule || e[Symbol.toStringTag] === "Module" || e.default && Oo(e.default);
}
const ue = Object.assign;
function Rs(e, t) {
  const n = {};
  for (const s in t) {
    const i = t[s];
    n[s] = tt(i) ? i.map(e) : e(i);
  }
  return n;
}
const An = () => {
}, tt = Array.isArray;
function sr(e, t) {
  const n = {};
  for (const s in e) n[s] = s in t ? t[s] : e[s];
  return n;
}
const $o = /#/g, kc = /&/g, Ec = /\//g, Cc = /=/g, xc = /\?/g, Io = /\+/g, Ac = /%5B/g, Rc = /%5D/g, To = /%5E/g, Pc = /%60/g, No = /%7B/g, Oc = /%7C/g, Mo = /%7D/g, $c = /%20/g;
function yi(e) {
  return e == null ? "" : encodeURI("" + e).replace(Oc, "|").replace(Ac, "[").replace(Rc, "]");
}
function Ic(e) {
  return yi(e).replace(No, "{").replace(Mo, "}").replace(To, "^");
}
function Ks(e) {
  return yi(e).replace(Io, "%2B").replace($c, "+").replace($o, "%23").replace(kc, "%26").replace(Pc, "`").replace(No, "{").replace(Mo, "}").replace(To, "^");
}
function Tc(e) {
  return Ks(e).replace(Cc, "%3D");
}
function Nc(e) {
  return yi(e).replace($o, "%23").replace(xc, "%3F");
}
function Mc(e) {
  return Nc(e).replace(Ec, "%2F");
}
function Nn(e) {
  if (e == null) return null;
  try {
    return decodeURIComponent("" + e);
  } catch {
  }
  return "" + e;
}
const Dc = /\/$/, Lc = (e) => e.replace(Dc, "");
function Ps(e, t, n = "/") {
  let s, i = {}, r = "", o = "";
  const l = t.indexOf("#");
  let a = t.indexOf("?");
  return a = l >= 0 && a > l ? -1 : a, a >= 0 && (s = t.slice(0, a), r = t.slice(a, l > 0 ? l : t.length), i = e(r.slice(1))), l >= 0 && (s = s || t.slice(0, l), o = t.slice(l, t.length)), s = Hc(s ?? t, n), { fullPath: s + r + o, path: s, query: i, hash: Nn(o) };
}
function jc(e, t) {
  const n = t.query ? e(t.query) : "";
  return t.path + (n && "?") + n + (t.hash || "");
}
function ir(e, t) {
  return !t || !e.toLowerCase().startsWith(t.toLowerCase()) ? e : e.slice(t.length) || "/";
}
function Uc(e, t, n) {
  const s = t.matched.length - 1, i = n.matched.length - 1;
  return s > -1 && s === i && on(t.matched[s], n.matched[i]) && Do(t.params, n.params) && e(t.query) === e(n.query) && t.hash === n.hash;
}
function on(e, t) {
  return (e.aliasOf || e) === (t.aliasOf || t);
}
function Do(e, t) {
  if (Object.keys(e).length !== Object.keys(t).length) return false;
  for (var n in e) if (!Vc(e[n], t[n])) return false;
  return true;
}
function Vc(e, t) {
  return tt(e) ? rr(e, t) : tt(t) ? rr(t, e) : (e == null ? void 0 : e.valueOf()) === (t == null ? void 0 : t.valueOf());
}
function rr(e, t) {
  return tt(t) ? e.length === t.length && e.every((n, s) => n === t[s]) : e.length === 1 && e[0] === t;
}
function Hc(e, t) {
  if (e.startsWith("/")) return e;
  if (!e) return t;
  const n = t.split("/"), s = e.split("/"), i = s[s.length - 1];
  (i === ".." || i === ".") && s.push("");
  let r = n.length - 1, o, l;
  for (o = 0; o < s.length; o++) if (l = s[o], l !== ".") if (l === "..") r > 1 && r--;
  else break;
  return n.slice(0, r).join("/") + "/" + s.slice(o).join("/");
}
const At = { path: "/", name: void 0, params: {}, query: {}, hash: "", fullPath: "/", matched: [], meta: {}, redirectedFrom: void 0 };
let Gs = (function(e) {
  return e.pop = "pop", e.push = "push", e;
})({}), Os = (function(e) {
  return e.back = "back", e.forward = "forward", e.unknown = "", e;
})({});
function Fc(e) {
  if (!e) if (Yt) {
    const t = document.querySelector("base");
    e = t && t.getAttribute("href") || "/", e = e.replace(/^\w+:\/\/[^\/]+/, "");
  } else e = "/";
  return e[0] !== "/" && e[0] !== "#" && (e = "/" + e), Lc(e);
}
const Wc = /^[^#]+#/;
function Bc(e, t) {
  return e.replace(Wc, "#") + t;
}
function qc(e, t) {
  const n = document.documentElement.getBoundingClientRect(), s = e.getBoundingClientRect();
  return { behavior: t.behavior, left: s.left - n.left - (t.left || 0), top: s.top - n.top - (t.top || 0) };
}
const vs = () => ({ left: window.scrollX, top: window.scrollY });
function Kc(e) {
  let t;
  if ("el" in e) {
    const n = e.el, s = typeof n == "string" && n.startsWith("#"), i = typeof n == "string" ? s ? document.getElementById(n.slice(1)) : document.querySelector(n) : n;
    if (!i) return;
    t = qc(i, e);
  } else t = e;
  "scrollBehavior" in document.documentElement.style ? window.scrollTo(t) : window.scrollTo(t.left != null ? t.left : window.scrollX, t.top != null ? t.top : window.scrollY);
}
function or(e, t) {
  return (history.state ? history.state.position - t : -1) + e;
}
const zs = /* @__PURE__ */ new Map();
function Gc(e, t) {
  zs.set(e, t);
}
function zc(e) {
  const t = zs.get(e);
  return zs.delete(e), t;
}
function Jc(e) {
  return typeof e == "string" || e && typeof e == "object";
}
function Lo(e) {
  return typeof e == "string" || typeof e == "symbol";
}
let Se = (function(e) {
  return e[e.MATCHER_NOT_FOUND = 1] = "MATCHER_NOT_FOUND", e[e.NAVIGATION_GUARD_REDIRECT = 2] = "NAVIGATION_GUARD_REDIRECT", e[e.NAVIGATION_ABORTED = 4] = "NAVIGATION_ABORTED", e[e.NAVIGATION_CANCELLED = 8] = "NAVIGATION_CANCELLED", e[e.NAVIGATION_DUPLICATED = 16] = "NAVIGATION_DUPLICATED", e;
})({});
const jo = Symbol("");
Se.MATCHER_NOT_FOUND + "", Se.NAVIGATION_GUARD_REDIRECT + "", Se.NAVIGATION_ABORTED + "", Se.NAVIGATION_CANCELLED + "", Se.NAVIGATION_DUPLICATED + "";
function ln(e, t) {
  return ue(new Error(), { type: e, [jo]: true }, t);
}
function yt(e, t) {
  return e instanceof Error && jo in e && (t == null || !!(e.type & t));
}
const Qc = ["params", "query", "hash"];
function Yc(e) {
  if (typeof e == "string") return e;
  if (e.path != null) return e.path;
  const t = {};
  for (const n of Qc) n in e && (t[n] = e[n]);
  return JSON.stringify(t, null, 2);
}
function Xc(e) {
  const t = {};
  if (e === "" || e === "?") return t;
  const n = (e[0] === "?" ? e.slice(1) : e).split("&");
  for (let s = 0; s < n.length; ++s) {
    const i = n[s].replace(Io, " "), r = i.indexOf("="), o = Nn(r < 0 ? i : i.slice(0, r)), l = r < 0 ? null : Nn(i.slice(r + 1));
    if (o in t) {
      let a = t[o];
      tt(a) || (a = t[o] = [a]), a.push(l);
    } else t[o] = l;
  }
  return t;
}
function lr(e) {
  let t = "";
  for (let n in e) {
    const s = e[n];
    if (n = Tc(n), s == null) {
      s !== void 0 && (t += (t.length ? "&" : "") + n);
      continue;
    }
    (tt(s) ? s.map((i) => i && Ks(i)) : [s && Ks(s)]).forEach((i) => {
      i !== void 0 && (t += (t.length ? "&" : "") + n, i != null && (t += "=" + i));
    });
  }
  return t;
}
function Zc(e) {
  const t = {};
  for (const n in e) {
    const s = e[n];
    s !== void 0 && (t[n] = tt(s) ? s.map((i) => i == null ? null : "" + i) : s == null ? s : "" + s);
  }
  return t;
}
const eu = Symbol(""), ar = Symbol(""), gs = Symbol(""), _i = Symbol(""), Js = Symbol("");
function hn() {
  let e = [];
  function t(s) {
    return e.push(s), () => {
      const i = e.indexOf(s);
      i > -1 && e.splice(i, 1);
    };
  }
  function n() {
    e = [];
  }
  return { add: t, list: () => e.slice(), reset: n };
}
function Ot(e, t, n, s, i, r = (o) => o()) {
  const o = s && (s.enterCallbacks[i] = s.enterCallbacks[i] || []);
  return () => new Promise((l, a) => {
    const p = (v) => {
      v === false ? a(ln(Se.NAVIGATION_ABORTED, { from: n, to: t })) : v instanceof Error ? a(v) : Jc(v) ? a(ln(Se.NAVIGATION_GUARD_REDIRECT, { from: t, to: v })) : (o && s.enterCallbacks[i] === o && typeof v == "function" && o.push(v), l());
    }, u = r(() => e.call(s && s.instances[i], t, n, p));
    let c = Promise.resolve(u);
    e.length < 3 && (c = c.then(p)), c.catch((v) => a(v));
  });
}
function $s(e, t, n, s, i = (r) => r()) {
  const r = [];
  for (const o of e) for (const l in o.components) {
    let a = o.components[l];
    if (!(t !== "beforeRouteEnter" && !o.instances[l])) if (Oo(a)) {
      const p = (a.__vccOpts || a)[t];
      p && r.push(Ot(p, n, s, o, l, i));
    } else {
      let p = a();
      r.push(() => p.then((u) => {
        if (!u) throw new Error(`Couldn't resolve component "${l}" at "${o.path}"`);
        const c = Sc(u) ? u.default : u;
        o.mods[l] = u, o.components[l] = c;
        const v = (c.__vccOpts || c)[t];
        return v && Ot(v, n, s, o, l, i)();
      }));
    }
  }
  return r;
}
function tu(e, t) {
  const n = [], s = [], i = [], r = Math.max(t.matched.length, e.matched.length);
  for (let o = 0; o < r; o++) {
    const l = t.matched[o];
    l && (e.matched.find((p) => on(p, l)) ? s.push(l) : n.push(l));
    const a = e.matched[o];
    a && (t.matched.find((p) => on(p, a)) || i.push(a));
  }
  return [n, s, i];
}
/*!
* vue-router v4.6.4
* (c) 2025 Eduardo San Martin Morote
* @license MIT
*/
let nu = () => location.protocol + "//" + location.host;
function Uo(e, t) {
  const { pathname: n, search: s, hash: i } = t, r = e.indexOf("#");
  if (r > -1) {
    let o = i.includes(e.slice(r)) ? e.slice(r).length : 1, l = i.slice(o);
    return l[0] !== "/" && (l = "/" + l), ir(l, "");
  }
  return ir(n, e) + s + i;
}
function su(e, t, n, s) {
  let i = [], r = [], o = null;
  const l = ({ state: v }) => {
    const m = Uo(e, location), b = n.value, _ = t.value;
    let P = 0;
    if (v) {
      if (n.value = m, t.value = v, o && o === b) {
        o = null;
        return;
      }
      P = _ ? v.position - _.position : 0;
    } else s(m);
    i.forEach((O) => {
      O(n.value, b, { delta: P, type: Gs.pop, direction: P ? P > 0 ? Os.forward : Os.back : Os.unknown });
    });
  };
  function a() {
    o = n.value;
  }
  function p(v) {
    i.push(v);
    const m = () => {
      const b = i.indexOf(v);
      b > -1 && i.splice(b, 1);
    };
    return r.push(m), m;
  }
  function u() {
    if (document.visibilityState === "hidden") {
      const { history: v } = window;
      if (!v.state) return;
      v.replaceState(ue({}, v.state, { scroll: vs() }), "");
    }
  }
  function c() {
    for (const v of r) v();
    r = [], window.removeEventListener("popstate", l), window.removeEventListener("pagehide", u), document.removeEventListener("visibilitychange", u);
  }
  return window.addEventListener("popstate", l), window.addEventListener("pagehide", u), document.addEventListener("visibilitychange", u), { pauseListeners: a, listen: p, destroy: c };
}
function cr(e, t, n, s = false, i = false) {
  return { back: e, current: t, forward: n, replaced: s, position: window.history.length, scroll: i ? vs() : null };
}
function iu(e) {
  const { history: t, location: n } = window, s = { value: Uo(e, n) }, i = { value: t.state };
  i.value || r(s.value, { back: null, current: s.value, forward: null, position: t.length - 1, replaced: true, scroll: null }, true);
  function r(a, p, u) {
    const c = e.indexOf("#"), v = c > -1 ? (n.host && document.querySelector("base") ? e : e.slice(c)) + a : nu() + e + a;
    try {
      t[u ? "replaceState" : "pushState"](p, "", v), i.value = p;
    } catch (m) {
      console.error(m), n[u ? "replace" : "assign"](v);
    }
  }
  function o(a, p) {
    r(a, ue({}, t.state, cr(i.value.back, a, i.value.forward, true), p, { position: i.value.position }), true), s.value = a;
  }
  function l(a, p) {
    const u = ue({}, i.value, t.state, { forward: a, scroll: vs() });
    r(u.current, u, true), r(a, ue({}, cr(s.value, a, null), { position: u.position + 1 }, p), false), s.value = a;
  }
  return { location: s, state: i, push: l, replace: o };
}
function Vo(e) {
  e = Fc(e);
  const t = iu(e), n = su(e, t.state, t.location, t.replace);
  function s(r, o = true) {
    o || n.pauseListeners(), history.go(r);
  }
  const i = ue({ location: "", base: e, go: s, createHref: Bc.bind(null, e) }, t, n);
  return Object.defineProperty(i, "location", { enumerable: true, get: () => t.location.value }), Object.defineProperty(i, "state", { enumerable: true, get: () => t.state.value }), i;
}
function ru(e) {
  return e = location.host ? e || location.pathname + location.search : "", e.includes("#") || (e += "#"), Vo(e);
}
let Wt = (function(e) {
  return e[e.Static = 0] = "Static", e[e.Param = 1] = "Param", e[e.Group = 2] = "Group", e;
})({});
var xe = (function(e) {
  return e[e.Static = 0] = "Static", e[e.Param = 1] = "Param", e[e.ParamRegExp = 2] = "ParamRegExp", e[e.ParamRegExpEnd = 3] = "ParamRegExpEnd", e[e.EscapeNext = 4] = "EscapeNext", e;
})(xe || {});
const ou = { type: Wt.Static, value: "" }, lu = /[a-zA-Z0-9_]/;
function au(e) {
  if (!e) return [[]];
  if (e === "/") return [[ou]];
  if (!e.startsWith("/")) throw new Error(`Invalid path "${e}"`);
  function t(m) {
    throw new Error(`ERR (${n})/"${p}": ${m}`);
  }
  let n = xe.Static, s = n;
  const i = [];
  let r;
  function o() {
    r && i.push(r), r = [];
  }
  let l = 0, a, p = "", u = "";
  function c() {
    p && (n === xe.Static ? r.push({ type: Wt.Static, value: p }) : n === xe.Param || n === xe.ParamRegExp || n === xe.ParamRegExpEnd ? (r.length > 1 && (a === "*" || a === "+") && t(`A repeatable param (${p}) must be alone in its segment. eg: '/:ids+.`), r.push({ type: Wt.Param, value: p, regexp: u, repeatable: a === "*" || a === "+", optional: a === "*" || a === "?" })) : t("Invalid state to consume buffer"), p = "");
  }
  function v() {
    p += a;
  }
  for (; l < e.length; ) {
    if (a = e[l++], a === "\\" && n !== xe.ParamRegExp) {
      s = n, n = xe.EscapeNext;
      continue;
    }
    switch (n) {
      case xe.Static:
        a === "/" ? (p && c(), o()) : a === ":" ? (c(), n = xe.Param) : v();
        break;
      case xe.EscapeNext:
        v(), n = s;
        break;
      case xe.Param:
        a === "(" ? n = xe.ParamRegExp : lu.test(a) ? v() : (c(), n = xe.Static, a !== "*" && a !== "?" && a !== "+" && l--);
        break;
      case xe.ParamRegExp:
        a === ")" ? u[u.length - 1] == "\\" ? u = u.slice(0, -1) + a : n = xe.ParamRegExpEnd : u += a;
        break;
      case xe.ParamRegExpEnd:
        c(), n = xe.Static, a !== "*" && a !== "?" && a !== "+" && l--, u = "";
        break;
      default:
        t("Unknown state");
        break;
    }
  }
  return n === xe.ParamRegExp && t(`Unfinished custom RegExp for param "${p}"`), c(), o(), i;
}
const ur = "[^/]+?", cu = { sensitive: false, strict: false, start: true, end: true };
var Ue = (function(e) {
  return e[e._multiplier = 10] = "_multiplier", e[e.Root = 90] = "Root", e[e.Segment = 40] = "Segment", e[e.SubSegment = 30] = "SubSegment", e[e.Static = 40] = "Static", e[e.Dynamic = 20] = "Dynamic", e[e.BonusCustomRegExp = 10] = "BonusCustomRegExp", e[e.BonusWildcard = -50] = "BonusWildcard", e[e.BonusRepeatable = -20] = "BonusRepeatable", e[e.BonusOptional = -8] = "BonusOptional", e[e.BonusStrict = 0.7000000000000001] = "BonusStrict", e[e.BonusCaseSensitive = 0.25] = "BonusCaseSensitive", e;
})(Ue || {});
const uu = /[.+*?^${}()[\]/\\]/g;
function fu(e, t) {
  const n = ue({}, cu, t), s = [];
  let i = n.start ? "^" : "";
  const r = [];
  for (const p of e) {
    const u = p.length ? [] : [Ue.Root];
    n.strict && !p.length && (i += "/");
    for (let c = 0; c < p.length; c++) {
      const v = p[c];
      let m = Ue.Segment + (n.sensitive ? Ue.BonusCaseSensitive : 0);
      if (v.type === Wt.Static) c || (i += "/"), i += v.value.replace(uu, "\\$&"), m += Ue.Static;
      else if (v.type === Wt.Param) {
        const { value: b, repeatable: _, optional: P, regexp: O } = v;
        r.push({ name: b, repeatable: _, optional: P });
        const w = O || ur;
        if (w !== ur) {
          m += Ue.BonusCustomRegExp;
          try {
            `${w}`;
          } catch (F) {
            throw new Error(`Invalid custom RegExp for param "${b}" (${w}): ` + F.message);
          }
        }
        let I = _ ? `((?:${w})(?:/(?:${w}))*)` : `(${w})`;
        c || (I = P && p.length < 2 ? `(?:/${I})` : "/" + I), P && (I += "?"), i += I, m += Ue.Dynamic, P && (m += Ue.BonusOptional), _ && (m += Ue.BonusRepeatable), w === ".*" && (m += Ue.BonusWildcard);
      }
      u.push(m);
    }
    s.push(u);
  }
  if (n.strict && n.end) {
    const p = s.length - 1;
    s[p][s[p].length - 1] += Ue.BonusStrict;
  }
  n.strict || (i += "/?"), n.end ? i += "$" : n.strict && !i.endsWith("/") && (i += "(?:/|$)");
  const o = new RegExp(i, n.sensitive ? "" : "i");
  function l(p) {
    const u = p.match(o), c = {};
    if (!u) return null;
    for (let v = 1; v < u.length; v++) {
      const m = u[v] || "", b = r[v - 1];
      c[b.name] = m && b.repeatable ? m.split("/") : m;
    }
    return c;
  }
  function a(p) {
    let u = "", c = false;
    for (const v of e) {
      (!c || !u.endsWith("/")) && (u += "/"), c = false;
      for (const m of v) if (m.type === Wt.Static) u += m.value;
      else if (m.type === Wt.Param) {
        const { value: b, repeatable: _, optional: P } = m, O = b in p ? p[b] : "";
        if (tt(O) && !_) throw new Error(`Provided param "${b}" is an array but it is not repeatable (* or + modifiers)`);
        const w = tt(O) ? O.join("/") : O;
        if (!w) if (P) v.length < 2 && (u.endsWith("/") ? u = u.slice(0, -1) : c = true);
        else throw new Error(`Missing required param "${b}"`);
        u += w;
      }
    }
    return u || "/";
  }
  return { re: o, score: s, keys: r, parse: l, stringify: a };
}
function du(e, t) {
  let n = 0;
  for (; n < e.length && n < t.length; ) {
    const s = t[n] - e[n];
    if (s) return s;
    n++;
  }
  return e.length < t.length ? e.length === 1 && e[0] === Ue.Static + Ue.Segment ? -1 : 1 : e.length > t.length ? t.length === 1 && t[0] === Ue.Static + Ue.Segment ? 1 : -1 : 0;
}
function Ho(e, t) {
  let n = 0;
  const s = e.score, i = t.score;
  for (; n < s.length && n < i.length; ) {
    const r = du(s[n], i[n]);
    if (r) return r;
    n++;
  }
  if (Math.abs(i.length - s.length) === 1) {
    if (fr(s)) return 1;
    if (fr(i)) return -1;
  }
  return i.length - s.length;
}
function fr(e) {
  const t = e[e.length - 1];
  return e.length > 0 && t[t.length - 1] < 0;
}
const pu = { strict: false, end: true, sensitive: false };
function hu(e, t, n) {
  const s = fu(au(e.path), n), i = ue(s, { record: e, parent: t, children: [], alias: [] });
  return t && !i.record.aliasOf == !t.record.aliasOf && t.children.push(i), i;
}
function vu(e, t) {
  const n = [], s = /* @__PURE__ */ new Map();
  t = sr(pu, t);
  function i(c) {
    return s.get(c);
  }
  function r(c, v, m) {
    const b = !m, _ = pr(c);
    _.aliasOf = m && m.record;
    const P = sr(t, c), O = [_];
    if ("alias" in c) {
      const F = typeof c.alias == "string" ? [c.alias] : c.alias;
      for (const ie of F) O.push(pr(ue({}, _, { components: m ? m.record.components : _.components, path: ie, aliasOf: m ? m.record : _ })));
    }
    let w, I;
    for (const F of O) {
      const { path: ie } = F;
      if (v && ie[0] !== "/") {
        const he = v.record.path, ae = he[he.length - 1] === "/" ? "" : "/";
        F.path = v.record.path + (ie && ae + ie);
      }
      if (w = hu(F, v, P), m ? m.alias.push(w) : (I = I || w, I !== w && I.alias.push(w), b && c.name && !hr(w) && o(c.name)), Fo(w) && a(w), _.children) {
        const he = _.children;
        for (let ae = 0; ae < he.length; ae++) r(he[ae], w, m && m.children[ae]);
      }
      m = m || w;
    }
    return I ? () => {
      o(I);
    } : An;
  }
  function o(c) {
    if (Lo(c)) {
      const v = s.get(c);
      v && (s.delete(c), n.splice(n.indexOf(v), 1), v.children.forEach(o), v.alias.forEach(o));
    } else {
      const v = n.indexOf(c);
      v > -1 && (n.splice(v, 1), c.record.name && s.delete(c.record.name), c.children.forEach(o), c.alias.forEach(o));
    }
  }
  function l() {
    return n;
  }
  function a(c) {
    const v = yu(c, n);
    n.splice(v, 0, c), c.record.name && !hr(c) && s.set(c.record.name, c);
  }
  function p(c, v) {
    let m, b = {}, _, P;
    if ("name" in c && c.name) {
      if (m = s.get(c.name), !m) throw ln(Se.MATCHER_NOT_FOUND, { location: c });
      P = m.record.name, b = ue(dr(v.params, m.keys.filter((I) => !I.optional).concat(m.parent ? m.parent.keys.filter((I) => I.optional) : []).map((I) => I.name)), c.params && dr(c.params, m.keys.map((I) => I.name))), _ = m.stringify(b);
    } else if (c.path != null) _ = c.path, m = n.find((I) => I.re.test(_)), m && (b = m.parse(_), P = m.record.name);
    else {
      if (m = v.name ? s.get(v.name) : n.find((I) => I.re.test(v.path)), !m) throw ln(Se.MATCHER_NOT_FOUND, { location: c, currentLocation: v });
      P = m.record.name, b = ue({}, v.params, c.params), _ = m.stringify(b);
    }
    const O = [];
    let w = m;
    for (; w; ) O.unshift(w.record), w = w.parent;
    return { name: P, path: _, params: b, matched: O, meta: mu(O) };
  }
  e.forEach((c) => r(c));
  function u() {
    n.length = 0, s.clear();
  }
  return { addRoute: r, resolve: p, removeRoute: o, clearRoutes: u, getRoutes: l, getRecordMatcher: i };
}
function dr(e, t) {
  const n = {};
  for (const s of t) s in e && (n[s] = e[s]);
  return n;
}
function pr(e) {
  const t = { path: e.path, redirect: e.redirect, name: e.name, meta: e.meta || {}, aliasOf: e.aliasOf, beforeEnter: e.beforeEnter, props: gu(e), children: e.children || [], instances: {}, leaveGuards: /* @__PURE__ */ new Set(), updateGuards: /* @__PURE__ */ new Set(), enterCallbacks: {}, components: "components" in e ? e.components || null : e.component && { default: e.component } };
  return Object.defineProperty(t, "mods", { value: {} }), t;
}
function gu(e) {
  const t = {}, n = e.props || false;
  if ("component" in e) t.default = n;
  else for (const s in e.components) t[s] = typeof n == "object" ? n[s] : n;
  return t;
}
function hr(e) {
  for (; e; ) {
    if (e.record.aliasOf) return true;
    e = e.parent;
  }
  return false;
}
function mu(e) {
  return e.reduce((t, n) => ue(t, n.meta), {});
}
function yu(e, t) {
  let n = 0, s = t.length;
  for (; n !== s; ) {
    const r = n + s >> 1;
    Ho(e, t[r]) < 0 ? s = r : n = r + 1;
  }
  const i = _u(e);
  return i && (s = t.lastIndexOf(i, s - 1)), s;
}
function _u(e) {
  let t = e;
  for (; t = t.parent; ) if (Fo(t) && Ho(e, t) === 0) return t;
}
function Fo({ record: e }) {
  return !!(e.name || e.components && Object.keys(e.components).length || e.redirect);
}
function vr(e) {
  const t = Xe(gs), n = Xe(_i), s = se(() => {
    const a = z(e.to);
    return t.resolve(a);
  }), i = se(() => {
    const { matched: a } = s.value, { length: p } = a, u = a[p - 1], c = n.matched;
    if (!u || !c.length) return -1;
    const v = c.findIndex(on.bind(null, u));
    if (v > -1) return v;
    const m = gr(a[p - 2]);
    return p > 1 && gr(u) === m && c[c.length - 1].path !== m ? c.findIndex(on.bind(null, a[p - 2])) : v;
  }), r = se(() => i.value > -1 && ku(n.params, s.value.params)), o = se(() => i.value > -1 && i.value === n.matched.length - 1 && Do(n.params, s.value.params));
  function l(a = {}) {
    if (Su(a)) {
      const p = t[z(e.replace) ? "replace" : "push"](z(e.to)).catch(An);
      return e.viewTransition && typeof document < "u" && "startViewTransition" in document && document.startViewTransition(() => p), p;
    }
    return Promise.resolve();
  }
  return { route: s, href: se(() => s.value.href), isActive: r, isExactActive: o, navigate: l };
}
function bu(e) {
  return e.length === 1 ? e[0] : e;
}
const wu = nt({ name: "RouterLink", compatConfig: { MODE: 3 }, props: { to: { type: [String, Object], required: true }, replace: Boolean, activeClass: String, exactActiveClass: String, custom: Boolean, ariaCurrentValue: { type: String, default: "page" }, viewTransition: Boolean }, useLink: vr, setup(e, { slots: t }) {
  const n = un(vr(e)), { options: s } = Xe(gs), i = se(() => ({ [mr(e.activeClass, s.linkActiveClass, "router-link-active")]: n.isActive, [mr(e.exactActiveClass, s.linkExactActiveClass, "router-link-exact-active")]: n.isExactActive }));
  return () => {
    const r = t.default && bu(t.default(n));
    return e.custom ? r : Ao("a", { "aria-current": n.isExactActive ? e.ariaCurrentValue : null, href: n.href, onClick: n.navigate, class: i.value }, r);
  };
} }), Ft = wu;
function Su(e) {
  if (!(e.metaKey || e.altKey || e.ctrlKey || e.shiftKey) && !e.defaultPrevented && !(e.button !== void 0 && e.button !== 0)) {
    if (e.currentTarget && e.currentTarget.getAttribute) {
      const t = e.currentTarget.getAttribute("target");
      if (/\b_blank\b/i.test(t)) return;
    }
    return e.preventDefault && e.preventDefault(), true;
  }
}
function ku(e, t) {
  for (const n in t) {
    const s = t[n], i = e[n];
    if (typeof s == "string") {
      if (s !== i) return false;
    } else if (!tt(i) || i.length !== s.length || s.some((r, o) => r.valueOf() !== i[o].valueOf())) return false;
  }
  return true;
}
function gr(e) {
  return e ? e.aliasOf ? e.aliasOf.path : e.path : "";
}
const mr = (e, t, n) => e ?? t ?? n, Eu = nt({ name: "RouterView", inheritAttrs: false, props: { name: { type: String, default: "default" }, route: Object }, compatConfig: { MODE: 3 }, setup(e, { attrs: t, slots: n }) {
  const s = Xe(Js), i = se(() => e.route || s.value), r = Xe(ar, 0), o = se(() => {
    let p = z(r);
    const { matched: u } = i.value;
    let c;
    for (; (c = u[p]) && !c.components; ) p++;
    return p;
  }), l = se(() => i.value.matched[o.value]);
  Bn(ar, se(() => o.value + 1)), Bn(eu, l), Bn(Js, i);
  const a = X();
  return kt(() => [a.value, l.value, e.name], ([p, u, c], [v, m, b]) => {
    u && (u.instances[c] = p, m && m !== u && p && p === v && (u.leaveGuards.size || (u.leaveGuards = m.leaveGuards), u.updateGuards.size || (u.updateGuards = m.updateGuards))), p && u && (!m || !on(u, m) || !v) && (u.enterCallbacks[c] || []).forEach((_) => _(p));
  }, { flush: "post" }), () => {
    const p = i.value, u = e.name, c = l.value, v = c && c.components[u];
    if (!v) return yr(n.default, { Component: v, route: p });
    const m = c.props[u], b = m ? m === true ? p.params : typeof m == "function" ? m(p) : m : null, P = Ao(v, ue({}, b, t, { onVnodeUnmounted: (O) => {
      O.component.isUnmounted && (c.instances[u] = null);
    }, ref: a }));
    return yr(n.default, { Component: P, route: p }) || P;
  };
} });
function yr(e, t) {
  if (!e) return null;
  const n = e(t);
  return n.length === 1 ? n[0] : n;
}
const Wo = Eu;
function Cu(e) {
  const t = vu(e.routes, e), n = e.parseQuery || Xc, s = e.stringifyQuery || lr, i = e.history, r = hn(), o = hn(), l = hn(), a = Al(At);
  let p = At;
  Yt && e.scrollBehavior && "scrollRestoration" in history && (history.scrollRestoration = "manual");
  const u = Rs.bind(null, (g) => "" + g), c = Rs.bind(null, Mc), v = Rs.bind(null, Nn);
  function m(g, V) {
    let L, B;
    return Lo(g) ? (L = t.getRecordMatcher(g), B = V) : B = g, t.addRoute(B, L);
  }
  function b(g) {
    const V = t.getRecordMatcher(g);
    V && t.removeRoute(V);
  }
  function _() {
    return t.getRoutes().map((g) => g.record);
  }
  function P(g) {
    return !!t.getRecordMatcher(g);
  }
  function O(g, V) {
    if (V = ue({}, V || a.value), typeof g == "string") {
      const y = Ps(n, g, V.path), x = t.resolve({ path: y.path }, V), A = i.createHref(y.fullPath);
      return ue(y, x, { params: v(x.params), hash: Nn(y.hash), redirectedFrom: void 0, href: A });
    }
    let L;
    if (g.path != null) L = ue({}, g, { path: Ps(n, g.path, V.path).path });
    else {
      const y = ue({}, g.params);
      for (const x in y) y[x] == null && delete y[x];
      L = ue({}, g, { params: c(y) }), V.params = c(V.params);
    }
    const B = t.resolve(L, V), re = g.hash || "";
    B.params = u(v(B.params));
    const d = jc(s, ue({}, g, { hash: Ic(re), path: B.path })), h = i.createHref(d);
    return ue({ fullPath: d, hash: re, query: s === lr ? Zc(g.query) : g.query || {} }, B, { redirectedFrom: void 0, href: h });
  }
  function w(g) {
    return typeof g == "string" ? Ps(n, g, a.value.path) : ue({}, g);
  }
  function I(g, V) {
    if (p !== g) return ln(Se.NAVIGATION_CANCELLED, { from: V, to: g });
  }
  function F(g) {
    return ae(g);
  }
  function ie(g) {
    return F(ue(w(g), { replace: true }));
  }
  function he(g, V) {
    const L = g.matched[g.matched.length - 1];
    if (L && L.redirect) {
      const { redirect: B } = L;
      let re = typeof B == "function" ? B(g, V) : B;
      return typeof re == "string" && (re = re.includes("?") || re.includes("#") ? re = w(re) : { path: re }, re.params = {}), ue({ query: g.query, hash: g.hash, params: re.path != null ? {} : g.params }, re);
    }
  }
  function ae(g, V) {
    const L = p = O(g), B = a.value, re = g.state, d = g.force, h = g.replace === true, y = he(L, B);
    if (y) return ae(ue(w(y), { state: typeof y == "object" ? ue({}, re, y.state) : re, force: d, replace: h }), V || L);
    const x = L;
    x.redirectedFrom = V;
    let A;
    return !d && Uc(s, B, L) && (A = ln(Se.NAVIGATION_DUPLICATED, { to: x, from: B }), j(B, B, true, false)), (A ? Promise.resolve(A) : Fe(x, B)).catch((E) => yt(E) ? yt(E, Se.NAVIGATION_GUARD_REDIRECT) ? E : N(E) : te(E, x, B)).then((E) => {
      if (E) {
        if (yt(E, Se.NAVIGATION_GUARD_REDIRECT)) return ae(ue({ replace: h }, w(E.to), { state: typeof E.to == "object" ? ue({}, re, E.to.state) : re, force: d }), V || x);
      } else E = Qe(x, B, true, h, re);
      return ze(x, B, E), E;
    });
  }
  function Me(g, V) {
    const L = I(g, V);
    return L ? Promise.reject(L) : Promise.resolve();
  }
  function He(g) {
    const V = T.values().next().value;
    return V && typeof V.runWithContext == "function" ? V.runWithContext(g) : g();
  }
  function Fe(g, V) {
    let L;
    const [B, re, d] = tu(g, V);
    L = $s(B.reverse(), "beforeRouteLeave", g, V);
    for (const y of B) y.leaveGuards.forEach((x) => {
      L.push(Ot(x, g, V));
    });
    const h = Me.bind(null, g, V);
    return L.push(h), Z(L).then(() => {
      L = [];
      for (const y of r.list()) L.push(Ot(y, g, V));
      return L.push(h), Z(L);
    }).then(() => {
      L = $s(re, "beforeRouteUpdate", g, V);
      for (const y of re) y.updateGuards.forEach((x) => {
        L.push(Ot(x, g, V));
      });
      return L.push(h), Z(L);
    }).then(() => {
      L = [];
      for (const y of d) if (y.beforeEnter) if (tt(y.beforeEnter)) for (const x of y.beforeEnter) L.push(Ot(x, g, V));
      else L.push(Ot(y.beforeEnter, g, V));
      return L.push(h), Z(L);
    }).then(() => (g.matched.forEach((y) => y.enterCallbacks = {}), L = $s(d, "beforeRouteEnter", g, V, He), L.push(h), Z(L))).then(() => {
      L = [];
      for (const y of o.list()) L.push(Ot(y, g, V));
      return L.push(h), Z(L);
    }).catch((y) => yt(y, Se.NAVIGATION_CANCELLED) ? y : Promise.reject(y));
  }
  function ze(g, V, L) {
    l.list().forEach((B) => He(() => B(g, V, L)));
  }
  function Qe(g, V, L, B, re) {
    const d = I(g, V);
    if (d) return d;
    const h = V === At, y = Yt ? history.state : {};
    L && (B || h ? i.replace(g.fullPath, ue({ scroll: h && y && y.scroll }, re)) : i.push(g.fullPath, re)), a.value = g, j(g, V, L, h), N();
  }
  let De;
  function st() {
    De || (De = i.listen((g, V, L) => {
      if (!$.listening) return;
      const B = O(g), re = he(B, $.currentRoute.value);
      if (re) {
        ae(ue(re, { replace: true, force: true }), B).catch(An);
        return;
      }
      p = B;
      const d = a.value;
      Yt && Gc(or(d.fullPath, L.delta), vs()), Fe(B, d).catch((h) => yt(h, Se.NAVIGATION_ABORTED | Se.NAVIGATION_CANCELLED) ? h : yt(h, Se.NAVIGATION_GUARD_REDIRECT) ? (ae(ue(w(h.to), { force: true }), B).then((y) => {
        yt(y, Se.NAVIGATION_ABORTED | Se.NAVIGATION_DUPLICATED) && !L.delta && L.type === Gs.pop && i.go(-1, false);
      }).catch(An), Promise.reject()) : (L.delta && i.go(-L.delta, false), te(h, B, d))).then((h) => {
        h = h || Qe(B, d, false), h && (L.delta && !yt(h, Se.NAVIGATION_CANCELLED) ? i.go(-L.delta, false) : L.type === Gs.pop && yt(h, Se.NAVIGATION_ABORTED | Se.NAVIGATION_DUPLICATED) && i.go(-1, false)), ze(B, d, h);
      }).catch(An);
    }));
  }
  let it = hn(), be = hn(), oe;
  function te(g, V, L) {
    N(g);
    const B = be.list();
    return B.length ? B.forEach((re) => re(g, V, L)) : console.error(g), Promise.reject(g);
  }
  function Le() {
    return oe && a.value !== At ? Promise.resolve() : new Promise((g, V) => {
      it.add([g, V]);
    });
  }
  function N(g) {
    return oe || (oe = !g, st(), it.list().forEach(([V, L]) => g ? L(g) : V()), it.reset()), g;
  }
  function j(g, V, L, B) {
    const { scrollBehavior: re } = e;
    if (!Yt || !re) return Promise.resolve();
    const d = !L && zc(or(g.fullPath, 0)) || (B || !L) && history.state && history.state.scroll || null;
    return pi().then(() => re(g, V, d)).then((h) => h && Kc(h)).catch((h) => te(h, g, V));
  }
  const W = (g) => i.go(g);
  let ne;
  const T = /* @__PURE__ */ new Set(), $ = { currentRoute: a, listening: true, addRoute: m, removeRoute: b, clearRoutes: t.clearRoutes, hasRoute: P, getRoutes: _, resolve: O, options: e, push: F, replace: ie, go: W, back: () => W(-1), forward: () => W(1), beforeEach: r.add, beforeResolve: o.add, afterEach: l.add, onError: be.add, isReady: Le, install(g) {
    g.component("RouterLink", Ft), g.component("RouterView", Wo), g.config.globalProperties.$router = $, Object.defineProperty(g.config.globalProperties, "$route", { enumerable: true, get: () => z(a) }), Yt && !ne && a.value === At && (ne = true, F(i.location).catch((B) => {
    }));
    const V = {};
    for (const B in At) Object.defineProperty(V, B, { get: () => a.value[B], enumerable: true });
    g.provide(gs, $), g.provide(_i, qr(V)), g.provide(Js, a);
    const L = g.unmount;
    T.add(g), g.unmount = function() {
      T.delete(g), T.size < 1 && (p = At, De && De(), De = null, a.value = At, ne = false, oe = false), L();
    };
  } };
  function Z(g) {
    return g.reduce((V, L) => V.then(() => He(L)), Promise.resolve());
  }
  return $;
}
function Bo() {
  return Xe(gs);
}
function xu(e) {
  return Xe(_i);
}
const Au = { class: "app-shell" }, Ru = { class: "topbar", role: "banner" }, Pu = { class: "nav", "aria-label": "\u4E3B\u5BFC\u822A" }, Ou = { class: "main", id: "main", tabindex: "-1" }, $u = nt({ __name: "App", setup(e) {
  return (t, n) => (k(), C("div", Au, [f("header", Ru, [we(z(Ft), { class: "brand", to: "/", "aria-label": "\u65E0\u540D\u6740 Next \u9996\u9875" }, { default: Ht(() => [...n[0] || (n[0] = [ye(" \u65E0\u540D\u6740 ", -1), f("span", { class: "accent" }, "Next", -1)])]), _: 1 }), f("nav", Pu, [we(z(Ft), { to: "/" }, { default: Ht(() => [...n[1] || (n[1] = [ye("\u5927\u5385", -1)])]), _: 1 }), we(z(Ft), { to: "/battle" }, { default: Ht(() => [...n[2] || (n[2] = [ye("\u5BF9\u5C40", -1)])]), _: 1 }), we(z(Ft), { to: "/settings" }, { default: Ht(() => [...n[3] || (n[3] = [ye("\u8BBE\u7F6E", -1)])]), _: 1 }), we(z(Ft), { to: "/replay" }, { default: Ht(() => [...n[4] || (n[4] = [ye("\u5F55\u50CF", -1)])]), _: 1 }), we(z(Ft), { to: "/collection" }, { default: Ht(() => [...n[5] || (n[5] = [ye("\u6B66\u5C06\u5E93", -1)])]), _: 1 })])]), f("main", Ou, [we(z(Wo))])]));
} });
async function Un(e, t = {}, n) {
  return window.__TAURI_INTERNALS__.invoke(e, t, n);
}
function Iu(e, t = "asset") {
  return window.__TAURI_INTERNALS__.convertFileSrc(e, t);
}
function Dt() {
  return typeof window < "u" && "__TAURI_INTERNALS__" in window;
}
function Tu() {
  return Dt() ? Un("platform_info") : Promise.resolve(null);
}
function _r(e) {
  return Dt() ? Un("set_match_frozen", { frozen: e }) : Promise.resolve();
}
function Nu() {
  return Dt() ? Un("content_bundle") : Promise.resolve(null);
}
function Mu(e, t) {
  return Dt() ? Un("content_catalog_page", { offset: e, limit: t }) : Promise.resolve(null);
}
async function Du(e) {
  if (!Dt()) return null;
  const t = await Un("resolve_content_asset", { namespace: e.namespace, packageId: e.package_id, assetId: e.asset_id });
  return Iu(t);
}
const Lu = { class: "stack" }, ju = { class: "panel hero" }, Uu = { key: 0, class: "shell-badge" }, Vu = { class: "row form" }, Hu = ["value"], Fu = { class: "panel" }, Wu = { class: "online-grid" }, Bu = { class: "wide" }, qu = { class: "row online-actions" }, Ku = { key: 0, class: "error", role: "alert" }, Gu = nt({ __name: "HomeView", setup(e) {
  const t = Bo(), n = X(4), s = X(42), i = X(`player-${Math.floor(Math.random() * 900 + 100)}`), r = X(""), o = X(1), l = X(v()), a = X(""), p = X(null);
  Ln(async () => {
    p.value = await Tu();
  });
  function u() {
    t.push({ path: "/battle", query: { players: String(n.value), seed: String(s.value) } });
  }
  function c(b) {
    if (a.value = "", !l.value.trim()) {
      a.value = "\u8BF7\u586B\u5199 WebSocket \u5730\u5740";
      return;
    }
    if (b !== "create" && o.value < 1) {
      a.value = "\u623F\u95F4\u53F7\u5FC5\u987B\u5927\u4E8E 0";
      return;
    }
    sessionStorage.setItem("noname-next.online-intent", JSON.stringify({ action: b, wsUrl: l.value, displayName: i.value, password: r.value, roomId: o.value, players: n.value, seed: s.value })), t.push({ path: "/battle", query: { mode: "online", action: b, ...b === "create" ? {} : { room: String(o.value) } } });
  }
  function v() {
    if (typeof window > "u") return "ws://127.0.0.1:7700/ws";
    const b = window.location.protocol === "https:" ? "wss:" : "ws:", _ = window.location.hostname || "127.0.0.1";
    return `${b}//${_}:7700/ws`;
  }
  const m = se(() => o.value > 0 ? `#${o.value}` : "\u2014");
  return (b, _) => (k(), C("div", Lu, [f("section", ju, [_[11] || (_[11] = f("h1", null, "\u786E\u5B9A\u6027\u8EAB\u4EFD\u5C40", -1)), _[12] || (_[12] = f("p", { class: "muted" }, " \u89C4\u5219\u6838\u5FC3\u8FD0\u884C\u5728 Web Worker \u4E2D\u7684 WASM\u3002\u754C\u9762\u53EA\u6D88\u8D39\u89C2\u5BDF\u8005\u6295\u5F71\uFF08Observer View\uFF09\uFF0C\u4E0D\u6301\u6709\u5B8C\u6574\u6743\u5A01\u72B6\u6001\u3002 ", -1)), p.value ? (k(), C("p", Uu, " Tauri 2 \xB7 " + S(p.value.os) + " \xB7 " + S(p.value.arch), 1)) : J("", true), f("div", Vu, [f("label", null, [_[9] || (_[9] = ye(" \u4EBA\u6570 ", -1)), Ae(f("select", { "onUpdate:modelValue": _[0] || (_[0] = (P) => n.value = P), "aria-label": "\u73A9\u5BB6\u4EBA\u6570" }, [(k(), C(ce, null, Ce([2, 3, 4, 5, 6, 7, 8], (P) => f("option", { key: P, value: P }, S(P), 9, Hu)), 64))], 512), [[Kt, n.value, void 0, { number: true }]])]), f("label", null, [_[10] || (_[10] = ye(" \u79CD\u5B50 ", -1)), Ae(f("input", { "onUpdate:modelValue": _[1] || (_[1] = (P) => s.value = P), type: "number", min: "0", "aria-label": "\u968F\u673A\u79CD\u5B50" }, null, 512), [[Qt, s.value, void 0, { number: true }]])]), f("button", { class: "primary", type: "button", onClick: u }, " \u672C\u5730\u5F00\u5C40 ")])]), f("section", Fu, [_[16] || (_[16] = f("h2", null, "\u8054\u673A\u5927\u5385", -1)), _[17] || (_[17] = f("p", { class: "muted" }, " \u901A\u8FC7 game-server WebSocket \u52A0\u5165\u6743\u5A01\u623F\u95F4\u3002\u6D4F\u89C8\u5668\u4EC5\u63D0\u4EA4\u547D\u4EE4\u5E76\u63A5\u6536\u81EA\u5DF1\u7684\u89C2\u5BDF\u8005\u6295\u5F71\u3002 ", -1)), f("div", Wu, [f("label", Bu, [_[13] || (_[13] = ye(" \u670D\u52A1\u7AEF ", -1)), Ae(f("input", { "onUpdate:modelValue": _[2] || (_[2] = (P) => l.value = P), type: "url", autocomplete: "url" }, null, 512), [[Qt, l.value, void 0, { trim: true }]])]), f("label", null, [_[14] || (_[14] = ye(" \u6635\u79F0 ", -1)), Ae(f("input", { "onUpdate:modelValue": _[3] || (_[3] = (P) => i.value = P), maxlength: "24", autocomplete: "nickname" }, null, 512), [[Qt, i.value, void 0, { trim: true }]])]), f("label", null, [_[15] || (_[15] = ye(" \u623F\u95F4\u5BC6\u7801 ", -1)), Ae(f("input", { "onUpdate:modelValue": _[4] || (_[4] = (P) => r.value = P), type: "password", autocomplete: "current-password" }, null, 512), [[Qt, r.value]])]), f("label", null, [ye(" \u623F\u95F4\u53F7 " + S(m.value) + " ", 1), Ae(f("input", { "onUpdate:modelValue": _[5] || (_[5] = (P) => o.value = P), type: "number", min: "1", inputmode: "numeric" }, null, 512), [[Qt, o.value, void 0, { number: true }]])])]), f("div", qu, [f("button", { class: "primary", type: "button", onClick: _[6] || (_[6] = (P) => c("create")) }, "\u521B\u5EFA\u5E76\u5165\u5EA7"), f("button", { type: "button", onClick: _[7] || (_[7] = (P) => c("join")) }, "\u52A0\u5165\u623F\u95F4"), f("button", { type: "button", onClick: _[8] || (_[8] = (P) => c("spectate")) }, "\u65C1\u89C2")]), a.value ? (k(), C("p", Ku, S(a.value), 1)) : J("", true)]), _[18] || (_[18] = f("section", { class: "panel" }, [f("h2", null, "\u67B6\u6784\u8981\u70B9"), f("ul", { class: "muted list" }, [f("li", null, "\u4E3B\u7EBF\u7A0B\uFF1AVue Shell + DOM \u6218\u6597\u5C42 + \u52A8\u753B\u65F6\u95F4\u7EBF"), f("li", null, "\u672C\u5730\uFF1AWorker + WASM Engine\uFF1B\u8054\u673A\uFF1AWebSocket + \u6743\u5A01\u670D\u52A1\u5668"), f("li", null, "\u652F\u6301\u81EA\u52A8\u6B65\u8FDB\u3001\u500D\u901F\u3001\u8DF3\u8FC7\u52A8\u753B\u3001\u51CF\u5C11\u52A8\u6001\u6548\u679C")])], -1))]));
} }), Lt = (e, t) => {
  const n = e.__vccOpts || e;
  for (const [s, i] of t) n[s] = i;
  return n;
}, zu = Lt(Gu, [["__scopeId", "data-v-2f7eee7f"]]);
class qo {
  constructor() {
    __publicField(this, "worker", null);
    __publicField(this, "seq", 1);
    __publicField(this, "pending", /* @__PURE__ */ new Map());
  }
  async connect() {
    if (this.worker) return;
    const t = Dt() ? await Nu() : void 0;
    this.worker = new Worker(new URL("/assets/engine.worker-6iNxSyL2.js", import.meta.url), { type: "module" }), this.worker.onmessage = (n) => {
      const s = n.data, i = this.pending.get(s.id);
      i && (this.pending.delete(s.id), s.ok ? i.resolve(s.data) : i.reject(new Error(s.error)));
    }, this.worker.onerror = (n) => {
      console.error("engine worker error", n);
    };
    try {
      await this.request({ id: this.nextId(), type: "init", bundle: t ?? void 0 });
    } catch (n) {
      throw this.dispose(), n;
    }
  }
  dispose() {
    var _a2;
    (_a2 = this.worker) == null ? void 0 : _a2.terminate(), this.worker = null, this.pending.clear();
  }
  request(t) {
    return this.worker ? new Promise((n, s) => {
      this.pending.set(t.id, { resolve: n, reject: s }), this.worker.postMessage(t);
    }) : Promise.reject(new Error("worker not connected"));
  }
  nextId() {
    return this.seq++;
  }
  start(t, n, s = 1) {
    return this.request({ id: this.nextId(), type: "start", players: t, seed: n, observer: s });
  }
  endPhase(t = 0) {
    return this.request({ id: this.nextId(), type: "endPhase", actor: t });
  }
  playCard(t, n, s) {
    return this.request({ id: this.nextId(), type: "playCard", actor: t, card: n, targets: s });
  }
  activateSkill(t, n, s, i) {
    return this.request({ id: this.nextId(), type: "activateSkill", actor: t, skill: n, targets: s, cardTargets: i });
  }
  useSkillCard(t, n, s, i, r) {
    return this.request({ id: this.nextId(), type: "useSkillCard", actor: t, skill: n, cardKind: s, targets: i, equipSlot: r });
  }
  resolveChoice(t, n, s, i = false) {
    return this.request({ id: this.nextId(), type: "resolveChoice", actor: t, choiceId: n, selections: s, cancelled: i });
  }
  autoStep() {
    return this.request({ id: this.nextId(), type: "autoStep" });
  }
  observer() {
    return this.request({ id: this.nextId(), type: "observer" });
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
const Qs = 200, K = un({ contentLockHash: "", total: 0, pages: /* @__PURE__ */ new Map(), characters: /* @__PURE__ */ new Map(), skins: /* @__PURE__ */ new Map(), translations: {}, selectedSkins: /* @__PURE__ */ new Map(), portraitUrls: /* @__PURE__ */ new Map(), imageUrls: /* @__PURE__ */ new Map(), substituteUrls: /* @__PURE__ */ new Map() });
let It = null;
const mn = /* @__PURE__ */ new Map(), yn = /* @__PURE__ */ new Map(), Ys = /* @__PURE__ */ new Map();
function Ju(e) {
  if (Dt()) {
    ei({ kind: "desktop", key: "desktop" });
    return;
  }
  const t = `${window.location.protocol === "https:" ? "https:" : "http:"}//${window.location.hostname || "127.0.0.1"}:7700`;
  ei({ kind: "http", key: `http:${t}`, baseUrl: t });
}
function Qu(e) {
  const t = new URL(e.trim(), window.location.href);
  t.protocol = t.protocol === "wss:" || t.protocol === "https:" ? "https:" : "http:";
  const n = t.origin;
  ei({ kind: "http", key: `http:${n}`, baseUrl: n });
}
async function Yu(e) {
  const t = K.characters.get(e);
  if (t) return K.imageUrls.has(e) || await Zs(e), t;
  if (!It) return;
  const n = yn.get(e);
  if (n) return n;
  const s = It.key, i = (async () => {
    let r = 0;
    for (; (It == null ? void 0 : It.key) === s; ) {
      const o = await Ko(r);
      if (!o) break;
      const l = K.characters.get(e);
      if (l) return await Zs(e), l;
      const a = o.offset + o.characters.length;
      if (a >= o.total || a <= r) break;
      r = a;
    }
  })();
  yn.set(e, i);
  try {
    return await i;
  } finally {
    yn.get(e) === i && yn.delete(e);
  }
}
function Xu(e) {
  var _a2;
  const t = (_a2 = K.characters.get(e)) == null ? void 0 : _a2.definition.name_key;
  return t ? Go(t) : void 0;
}
function Zu(e) {
  var _a2;
  return ((_a2 = K.characters.get(e)) == null ? void 0 : _a2.skin_ids.flatMap((n) => {
    const s = K.skins.get(n);
    return s && s.presentation.kind !== "substitute" ? [s] : [];
  })) ?? [];
}
function ef(e) {
  return Go(e.name_key);
}
function Xs(e) {
  const t = K.characters.get(e);
  if (!t || !K.contentLockHash) return null;
  if (K.selectedSkins.has(e)) return K.selectedSkins.get(e) ?? null;
  const n = zo(e);
  let s = null;
  if (n) try {
    const i = Number(localStorage.getItem(n));
    Number.isInteger(i) && Jo(t).includes(i) && (s = i);
  } catch {
  }
  return K.selectedSkins.set(e, s), s;
}
function tf(e, t) {
  const n = K.characters.get(e);
  if (t != null && (!n || !Jo(n).includes(t))) return;
  K.selectedSkins.set(e, t);
  const s = zo(e);
  if (s) try {
    t == null ? localStorage.removeItem(s) : localStorage.setItem(s, String(t));
  } catch {
  }
  Zs(e);
}
function nf(e) {
  return K.imageUrls.get(e) ?? ms(e);
}
function sf(e) {
  return K.portraitUrls.get(e) ?? ms(e);
}
function rf(e, t) {
  return K.substituteUrls.get(`${e}:${t}`) ?? null;
}
function ms(e) {
  return `/original/portraits/${e}.jpg`;
}
async function Ko(e) {
  const t = K.pages.get(e);
  if (t) return t;
  const n = mn.get(e);
  if (n) return n;
  const s = It;
  if (!s) return null;
  const i = (async () => {
    let r;
    if (s.kind === "desktop") r = await Mu(e, Qs);
    else {
      const o = new URL("/content-catalog", s.baseUrl);
      o.searchParams.set("offset", String(e)), o.searchParams.set("limit", String(Qs));
      const l = await fetch(o);
      if (!l.ok) throw new Error(`\u5185\u5BB9\u76EE\u5F55\u52A0\u8F7D\u5931\u8D25 (${l.status})`);
      r = await l.json();
    }
    if (It !== s || r == null) return null;
    if (lf(r), r.offset !== Math.min(e, r.total)) throw new Error("\u5185\u5BB9\u76EE\u5F55\u5206\u9875\u504F\u79FB\u4E0D\u4E00\u81F4");
    return e !== 0 && K.contentLockHash && K.contentLockHash !== r.content_lock_hash ? (bi(), Ko(0)) : (of(r), r);
  })();
  mn.set(e, i);
  try {
    return await i;
  } finally {
    mn.get(e) === i && mn.delete(e);
  }
}
function of(e) {
  K.contentLockHash && K.contentLockHash !== e.content_lock_hash && bi(), K.contentLockHash = e.content_lock_hash, K.total = e.total, K.pages.set(e.offset, e);
  for (const t of e.characters) K.characters.set(t.definition.id, t);
  for (const t of e.skins) K.skins.set(t.id, t);
  for (const [t, n] of Object.entries(e.translations)) K.translations[t] = { ...K.translations[t], ...n };
}
async function Zs(e) {
  const t = K.characters.get(e);
  if (!t) return;
  const n = Xs(e), s = n == null ? void 0 : K.skins.get(n), i = (s == null ? void 0 : s.presentation.kind) === "static" ? s.presentation.asset : null, r = ms(e);
  K.portraitUrls.set(e, r), K.imageUrls.set(e, r);
  const o = `${K.contentLockHash}:${n ?? "base"}`, l = t.portrait_asset ? await Is(t.portrait_asset).catch(() => null) : null, a = i ? await Is(i).catch(() => null) : null, p = t.skin_ids.map((v) => K.skins.get(v)).find((v) => (v == null ? void 0 : v.presentation.kind) === "substitute" && v.presentation.rule === "dead"), u = (p == null ? void 0 : p.presentation.kind) === "substitute" ? p.presentation.asset : null, c = u ? await Is(u).catch(() => null) : null;
  if (o === `${K.contentLockHash}:${Xs(e) ?? "base"}`) {
    const v = l ?? r;
    K.portraitUrls.set(e, v), K.imageUrls.set(e, a ?? v), c && K.substituteUrls.set(`${e}:dead`, c);
  }
}
function Is(e) {
  if (!af(e) || !It) return Promise.resolve(null);
  const t = It, n = [t.key, e.namespace, e.package_id, e.asset_id].join("\0"), s = Ys.get(n);
  if (s) return s;
  const i = t.kind === "desktop" ? Du(e) : Promise.resolve(`${t.baseUrl}/package-assets/${[e.namespace, e.package_id].map(encodeURIComponent).join("/")}/${e.asset_id.split("/").map(encodeURIComponent).join("/")}?v=${encodeURIComponent(K.contentLockHash)}`);
  return Ys.set(n, i), i;
}
function Go(e) {
  var _a2;
  const t = typeof navigator > "u" ? "" : navigator.language, n = [t, t.replace("-", "_"), "zh-CN", "zh_CN", "zh", "en"];
  for (const s of n) {
    const i = (_a2 = K.translations[s]) == null ? void 0 : _a2[e];
    if (i) return i;
  }
  return e;
}
function zo(e) {
  return K.contentLockHash ? `noname-next.skin.${K.contentLockHash}.${e}` : null;
}
function ei(e) {
  It = e, bi();
}
function bi() {
  K.contentLockHash = "", K.total = 0, K.pages.clear(), K.characters.clear(), K.skins.clear(), K.translations = {}, K.selectedSkins.clear(), K.portraitUrls.clear(), K.imageUrls.clear(), K.substituteUrls.clear(), mn.clear(), yn.clear(), Ys.clear();
}
function lf(e) {
  const t = e;
  if (!t || typeof t.content_lock_hash != "string" || !Rt(t.offset) || !Rt(t.limit) || !Rt(t.total) || t.limit < 1 || t.limit > Qs || t.offset > t.total || !Array.isArray(t.characters) || t.characters.length > t.limit || !Array.isArray(t.skins) || !Array.isArray(t.aliases) || !t.translations || typeof t.translations != "object" || !t.characters.every((n) => {
    var _a2, _b;
    return Rt((_a2 = n == null ? void 0 : n.definition) == null ? void 0 : _a2.id) && n.definition.id > 0 && typeof ((_b = n == null ? void 0 : n.definition) == null ? void 0 : _b.name_key) == "string" && Array.isArray(n == null ? void 0 : n.skin_ids) && n.skin_ids.every(Rt);
  }) || !t.skins.every((n) => {
    var _a2;
    return Rt(n == null ? void 0 : n.id) && n.id > 0 && typeof (n == null ? void 0 : n.name_key) == "string" && ["static", "spine", "substitute"].includes((_a2 = n == null ? void 0 : n.presentation) == null ? void 0 : _a2.kind);
  }) || !t.aliases.every((n) => Rt(n == null ? void 0 : n.character_id) && Rt(n == null ? void 0 : n.source_character_id) && br(n == null ? void 0 : n.voice_mappings)) || !Object.values(t.translations).every(br)) throw new Error("\u5185\u5BB9\u76EE\u5F55\u54CD\u5E94\u65E0\u6548");
}
function Rt(e) {
  return Number.isSafeInteger(e) && Number(e) >= 0;
}
function Jo(e) {
  return e.skin_ids.filter((t) => {
    var _a2;
    return ((_a2 = K.skins.get(t)) == null ? void 0 : _a2.presentation.kind) !== "substitute";
  });
}
function br(e) {
  return !!e && typeof e == "object" && !Array.isArray(e) && Object.values(e).every((t) => typeof t == "string");
}
function af(e) {
  return typeof e.namespace == "string" && e.namespace.length > 0 && typeof e.package_id == "string" && e.package_id.length > 0 && typeof e.asset_id == "string" && e.asset_id.length > 0;
}
const wr = 5;
class cf {
  constructor() {
    __publicField(this, "socket", null);
    __publicField(this, "connectionId", 0);
    __publicField(this, "roomId", 0);
    __publicField(this, "playerId", 0);
    __publicField(this, "revision", 0);
    __publicField(this, "connSeq", 0);
    __publicField(this, "idemSeq", 1);
    __publicField(this, "choice", null);
    __publicField(this, "welcomeWaiter", null);
    __publicField(this, "joinWaiter", null);
    __publicField(this, "pending", /* @__PURE__ */ new Map());
    __publicField(this, "snapshotWaiters", /* @__PURE__ */ new Map());
    __publicField(this, "envelopeListeners", /* @__PURE__ */ new Set());
    __publicField(this, "rosterListeners", /* @__PURE__ */ new Set());
    __publicField(this, "heartbeat", null);
    __publicField(this, "disposed", false);
  }
  async connect(t, n) {
    var _a2;
    if (((_a2 = this.socket) == null ? void 0 : _a2.readyState) === WebSocket.OPEN) return;
    this.disposed = false;
    const s = new Promise((r, o) => {
      this.welcomeWaiter = { resolve: r, reject: o };
    });
    await new Promise((r, o) => {
      const l = new WebSocket(uf(t));
      this.socket = l, l.onopen = () => r(), l.onerror = () => o(new Error("WebSocket \u8FDE\u63A5\u5931\u8D25")), l.onmessage = (a) => this.handleMessage(String(a.data)), l.onclose = () => this.handleClose();
    }), await s, this.send({ type: "Hello", payload: { versions: { engine_api: 1, rule_vm_abi: 167, content_schema: 3, network_protocol: wr, replay_schema: 1, snapshot_schema: 1, package_format: 1, engine_build_id: "noname-next-web" }, display_name: n } });
  }
  createRoom(t, n, s = "") {
    const i = this.waitForJoin();
    return this.send({ type: "CreateRoom", payload: { players: t, seed: n, password: s } }), i;
  }
  joinRoom(t, n = "", s = null) {
    this.roomId = t;
    const i = this.waitForJoin();
    return this.send({ type: "JoinRoom", payload: { room_id: t, password: n, preferred_seat: null, reconnect_token: s } }), i;
  }
  spectate(t) {
    this.roomId = t;
    const n = this.waitForJoin();
    return this.send({ type: "Spectate", payload: { room_id: t } }), n;
  }
  onEnvelope(t) {
    return this.envelopeListeners.add(t), () => this.envelopeListeners.delete(t);
  }
  onRoster(t) {
    return this.rosterListeners.add(t), () => this.rosterListeners.delete(t);
  }
  endPhase(t = 0) {
    return this.submit("EndPhase");
  }
  playCard(t, n, s) {
    return this.submit({ PlayCard: { card: n, targets: s, card_targets: [] } });
  }
  activateSkill(t, n, s, i) {
    return this.submit({ ActivateSkill: { skill: n, targets: s, card_targets: i } });
  }
  useSkillCard(t, n, s, i, r) {
    return this.submit({ UseSkillCard: { skill: n, card_kind: s, targets: i, equip_slot: r } });
  }
  resolveChoice(t, n, s, i = false) {
    return this.submit({ ResolveChoice: { choice_id: n, selections: s, cancelled: i } });
  }
  surrender() {
    return this.submit("Surrender");
  }
  requestSnapshot() {
    const t = this.nextHeader(), n = new Promise((s, i) => {
      this.snapshotWaiters.set(t.conn_seq, { resolve: s, reject: i });
    });
    return this.send({ type: "RequestSnapshot", payload: { header: t, last_revision: this.revision } }), n;
  }
  dispose() {
    var _a2;
    this.disposed = true, this.heartbeat != null && window.clearInterval(this.heartbeat), this.heartbeat = null, (_a2 = this.socket) == null ? void 0 : _a2.close(1e3, "client dispose"), this.socket = null, this.rejectOutstanding(new Error("\u8FDE\u63A5\u5DF2\u5173\u95ED")), this.envelopeListeners.clear(), this.rosterListeners.clear();
  }
  waitForJoin() {
    return this.joinWaiter ? Promise.reject(new Error("\u5DF2\u6709\u5165\u623F\u8BF7\u6C42")) : new Promise((t, n) => {
      this.joinWaiter = { resolve: t, reject: n };
    });
  }
  submit(t) {
    if (!this.roomId || !this.playerId) return Promise.reject(new Error("\u5F53\u524D\u4E0D\u662F\u5DF2\u5165\u5EA7\u73A9\u5BB6"));
    const n = this.nextHeader(), s = this.connectionId * 1e6 + this.idemSeq++, i = new Promise((r, o) => {
      this.pending.set(s, { resolve: r, reject: o });
    });
    try {
      this.send({ type: "SubmitCommand", payload: { header: n, command: { actor: this.playerId, expected_revision: this.revision, idempotency_key: s, kind: t }, idempotency_key: s } });
    } catch (r) {
      return this.pending.delete(s), Promise.reject(r);
    }
    return i;
  }
  nextHeader() {
    return { protocol_version: wr, room_id: this.roomId, conn_seq: ++this.connSeq, revision: this.revision };
  }
  send(t) {
    var _a2;
    if (((_a2 = this.socket) == null ? void 0 : _a2.readyState) !== WebSocket.OPEN) throw new Error("WebSocket \u672A\u8FDE\u63A5");
    this.socket.send(JSON.stringify(t));
  }
  handleMessage(t) {
    var _a2, _b;
    let n;
    try {
      n = JSON.parse(t);
    } catch {
      this.rejectOutstanding(new Error("\u670D\u52A1\u7AEF\u8FD4\u56DE\u4E86\u65E0\u6548 JSON"));
      return;
    }
    const s = n.payload ?? {};
    switch (n.type) {
      case "Welcome":
        this.connectionId = Number(s.connection_id ?? 0), (_a2 = this.welcomeWaiter) == null ? void 0 : _a2.resolve(), this.welcomeWaiter = null;
        break;
      case "RoomCreated":
        this.roomId = Number(s.room_id ?? 0);
        break;
      case "JoinOk": {
        this.roomId = Number(s.room_id ?? this.roomId), this.playerId = Number(s.player_id ?? 0);
        const i = s.view;
        this.choice = s.choice ?? null, this.revision = i.revision;
        const r = { roomId: this.roomId, playerId: this.playerId, seat: Number(s.seat ?? 255), role: String(s.role ?? "Spectator"), reconnectToken: String(s.reconnect_token ?? ""), view: i, envelope: Ts(i, this.choice, [], s.legal_actions ?? [], null) };
        (_b = this.joinWaiter) == null ? void 0 : _b.resolve(r), this.joinWaiter = null, this.startHeartbeat();
        break;
      }
      case "RoomRoster": {
        const i = s;
        for (const r of this.rosterListeners) r(i);
        break;
      }
      case "CommandOutcome":
        this.handleOutcome(s);
        break;
      case "Snapshot": {
        const i = s.header, r = s.view;
        this.choice = s.choice ?? null, this.revision = r.revision;
        const o = this.snapshotWaiters.get(i.conn_seq);
        o && (this.snapshotWaiters.delete(i.conn_seq), o.resolve(r));
        const l = Ts(r, this.choice, [], s.legal_actions ?? [], null);
        for (const a of this.envelopeListeners) a(l);
        break;
      }
      case "Error": {
        const i = new Error(String(s.message ?? "\u670D\u52A1\u7AEF\u9519\u8BEF"));
        this.joinWaiter ? (this.joinWaiter.reject(i), this.joinWaiter = null) : this.rejectOutstanding(i);
        break;
      }
    }
  }
  handleOutcome(t) {
    const n = t.view, s = this.revision;
    this.revision = n.revision, this.choice = t.choice ?? null;
    const i = t.public_event_codes ?? [], r = Ts(n, this.choice, i, t.legal_actions ?? [], t.result ?? null), o = Number(t.idempotency_key ?? 0), l = this.pending.get(o);
    if (l) {
      this.pending.delete(o);
      const a = pf(t.result);
      a ? l.reject(new Error(a)) : l.resolve(r);
    } else for (const a of this.envelopeListeners) a(r);
    s > 0 && n.revision > s + 1 && this.requestSnapshot().catch(() => {
    });
  }
  startHeartbeat() {
    this.heartbeat != null && window.clearInterval(this.heartbeat), this.heartbeat = window.setInterval(() => {
      var _a2;
      if (((_a2 = this.socket) == null ? void 0 : _a2.readyState) !== WebSocket.OPEN) return;
      const t = this.nextHeader();
      this.send({ type: "Heartbeat", payload: { header: t } });
    }, 15e3);
  }
  handleClose() {
    this.heartbeat != null && window.clearInterval(this.heartbeat), this.heartbeat = null, this.socket = null, this.disposed || this.rejectOutstanding(new Error("\u4E0E\u670D\u52A1\u7AEF\u7684\u8FDE\u63A5\u5DF2\u65AD\u5F00"));
  }
  rejectOutstanding(t) {
    var _a2, _b;
    (_a2 = this.welcomeWaiter) == null ? void 0 : _a2.reject(t), this.welcomeWaiter = null, (_b = this.joinWaiter) == null ? void 0 : _b.reject(t), this.joinWaiter = null;
    for (const n of this.pending.values()) n.reject(t);
    this.pending.clear();
    for (const n of this.snapshotWaiters.values()) n.reject(t);
    this.snapshotWaiters.clear();
  }
}
function uf(e) {
  const t = e.trim();
  return t.startsWith("http://") ? `ws://${t.slice(7)}` : t.startsWith("https://") ? `wss://${t.slice(8)}` : t;
}
function Ts(e, t, n, s, i) {
  return { result: i, events: n.map((r) => ({ PublicEvent: { code: r, label: hf[r] ?? `\u4E8B\u4EF6 ${r}` } })), choice: t, view: e, legal_actions: s, hash_hex: ff(e.state_hash), revision: e.revision, finished: df(e.outcome) };
}
function ff(e) {
  return typeof e == "string" ? e : e.map((t) => t.toString(16).padStart(2, "0")).join("");
}
function df(e) {
  return e == null || e === "InProgress" ? false : !(typeof e == "object" && "InProgress" in e);
}
function pf(e) {
  if (!e || typeof e != "object" || !("Rejected" in e)) return null;
  const t = e.Rejected;
  return `\u547D\u4EE4\u88AB\u62D2\u7EDD\uFF1A${JSON.stringify(t)}`;
}
const hf = { 1: "\u5BF9\u5C40\u5F00\u59CB", 2: "\u6478\u724C", 3: "\u4F7F\u7528\u5361\u724C", 4: "\u9020\u6210\u4F24\u5BB3", 5: "\u56DE\u590D\u4F53\u529B", 6: "\u8FDB\u5165\u6FD2\u6B7B", 7: "\u73A9\u5BB6\u9635\u4EA1", 8: "\u56DE\u5408\u63A8\u8FDB", 9: "\u5BF9\u5C40\u7ED3\u675F", 10: "\u7B49\u5F85\u9009\u62E9" };
let vf = 1;
function gf() {
  var _a2;
  const e = typeof window < "u" && ((_a2 = window.matchMedia) == null ? void 0 : _a2.call(window, "(prefers-reduced-motion: reduce)").matches);
  return un({ connected: false, busy: false, error: null, revision: 0, hash: "", finished: false, view: null, choice: null, legalActions: [], lastEvents: [], log: [], effects: [], speed: 1, skipAnims: false, reduceMotion: !!e });
}
let zt = 1;
function mf(e, t) {
  e.revision = t.revision, e.hash = t.hash_hex, e.finished = t.finished, e.view = t.view, e.choice = t.choice, e.legalActions = t.legal_actions ?? [], e.lastEvents = t.events ?? [], e.error = null;
  const n = yf(t.events ?? []);
  e.effects = n;
  const s = bf(t.events ?? []);
  for (const i of s) _n(e, i);
}
function yf(e) {
  const t = [];
  for (const n of e) {
    if (!n || typeof n != "object") continue;
    const s = n, i = Object.keys(s)[0];
    if (!i) continue;
    const r = s[i];
    switch (i) {
      case "DamageDealt":
        t.push({ id: zt++, type: "damage", player: r == null ? void 0 : r.target, amount: r == null ? void 0 : r.amount, text: `-${r == null ? void 0 : r.amount}` });
        break;
      case "HpRecovered":
        t.push({ id: zt++, type: "heal", player: r == null ? void 0 : r.player, amount: r == null ? void 0 : r.amount, text: `+${r == null ? void 0 : r.amount}` });
        break;
      case "CardsDrawn":
        t.push({ id: zt++, type: "draw", player: r == null ? void 0 : r.player, text: `\u6478${Array.isArray(r == null ? void 0 : r.cards) ? r.cards.length : 1}` });
        break;
      case "CardUsed":
        t.push({ id: zt++, type: "play", player: r == null ? void 0 : r.player, text: "\u51FA\u724C" });
        break;
      case "EnteredDying":
        t.push({ id: zt++, type: "dying", player: r == null ? void 0 : r.player, text: "\u6FD2\u6B7B" });
        break;
      case "PlayerKilled":
        t.push({ id: zt++, type: "death", player: r == null ? void 0 : r.player, text: "\u9635\u4EA1" });
        break;
    }
  }
  return t;
}
function _f(e) {
  e.effects = [];
}
function _n(e, t) {
  e.log.unshift({ id: vf++, text: t, at: Date.now() }), e.log.length > 80 && (e.log.length = 80);
}
function bf(e) {
  const t = [];
  for (const n of e) {
    if (!n || typeof n != "object") continue;
    const s = n, i = Object.keys(s)[0];
    if (!i) continue;
    const r = s[i];
    switch (i) {
      case "GameStarted":
        t.push(`\u5BF9\u5C40\u5F00\u59CB \xB7 ${(r == null ? void 0 : r.player_count) ?? "?"} \u4EBA`);
        break;
      case "CardsDrawn":
        t.push(`\u73A9\u5BB6 ${r == null ? void 0 : r.player} \u6478\u724C \xD7${Array.isArray(r == null ? void 0 : r.cards) ? r.cards.length : "?"}`);
        break;
      case "CardUsed":
        t.push(`\u73A9\u5BB6 ${r == null ? void 0 : r.player} \u4F7F\u7528\u5361\u724C ${r == null ? void 0 : r.card}`);
        break;
      case "DamageDealt":
        t.push(`\u4F24\u5BB3 ${r == null ? void 0 : r.source} \u2192 ${r == null ? void 0 : r.target} (${r == null ? void 0 : r.amount})`);
        break;
      case "HpRecovered":
        t.push(`\u73A9\u5BB6 ${r == null ? void 0 : r.player} \u56DE\u590D ${r == null ? void 0 : r.amount}`);
        break;
      case "EnteredDying":
        t.push(`\u73A9\u5BB6 ${r == null ? void 0 : r.player} \u8FDB\u5165\u6FD2\u6B7B`);
        break;
      case "PlayerKilled":
        t.push(`\u73A9\u5BB6 ${r == null ? void 0 : r.player} \u9635\u4EA1`);
        break;
      case "TurnStateChanged":
        t.push(`\u56DE\u5408 \xB7 \u73A9\u5BB6 ${r == null ? void 0 : r.current} \xB7 ${r == null ? void 0 : r.phase}`);
        break;
      case "MatchEnded":
        t.push(`\u7EC8\u5C40 \xB7 ${JSON.stringify(r == null ? void 0 : r.outcome)}`);
        break;
      case "ChoiceRequested":
        t.push(`\u7B49\u5F85\u9009\u62E9 \xB7 \u73A9\u5BB6 ${r == null ? void 0 : r.observer}`);
        break;
      case "PublicEvent":
        t.push(String((r == null ? void 0 : r.label) ?? `\u516C\u5171\u4E8B\u4EF6 ${(r == null ? void 0 : r.code) ?? "?"}`));
        break;
      default:
        t.push(i);
    }
  }
  return t;
}
function wf(e, t) {
  if (e.skipAnims || e.reduceMotion || e.speed <= 0) return Promise.resolve();
  const n = Math.max(0, t / e.speed);
  return new Promise((s) => setTimeout(s, n));
}
const Sf = ["aria-label", "tabindex"], kf = ["src", "alt"], Ef = ["aria-label"], Cf = { class: "name" }, xf = { key: 0, class: "tag" }, Af = { key: 1, class: "tag", "aria-label": "\u6B66\u5C06\u724C\u80CC\u9762" }, Rf = { class: "hp-row" }, Pf = ["aria-label"], Of = { class: "hp", "aria-label": "\u4F53\u529B" }, $f = ["aria-valuenow", "aria-valuemax", "aria-label"], If = { class: "meta muted" }, Tf = ["value", "aria-label"], Nf = ["value"], Mf = { key: 3, class: "equip-zone", "aria-label": "\u88C5\u5907\u533A" }, Df = ["title"], Lf = { class: "equip-suit" }, jf = { class: "equip-id" }, Uf = { key: 4, class: "dead-badge" }, Vf = { key: 5, class: "dying-badge" }, Hf = { key: 6, class: "target-hint" }, Ff = nt({ __name: "SeatCard", props: { seat: {}, self: { type: Boolean }, identityLabel: {}, isTargetMode: { type: Boolean }, isSelectableTarget: { type: Boolean }, isDropTarget: { type: Boolean }, equipCards: {} }, emits: ["select"], setup(e, { emit: t }) {
  const n = e, s = t, i = se(() => Xu(n.seat.character_def) ?? `#${n.seat.character_def}`), r = se(() => Zu(n.seat.character_def)), o = se(() => Xs(n.seat.character_def)), l = se(() => nf(n.seat.character_def)), a = se(() => sf(n.seat.character_def)), p = se(() => ms(n.seat.character_def)), u = se(() => Array.from(/* @__PURE__ */ new Set([...n.seat.alive ? [] : [rf(n.seat.character_def, "dead")], l.value, a.value, p.value])).filter((O) => !!O)), c = X([]), v = se(() => u.value.find((O) => !c.value.includes(O)) ?? "");
  kt(() => n.seat.character_def, (O) => {
    Yu(O).catch(() => {
    });
  }, { immediate: true }), kt(u, () => {
    c.value = [];
  });
  function m() {
    v.value && (c.value = [...c.value, v.value]);
  }
  function b(O) {
    const w = O.target.value;
    tf(n.seat.character_def, w === "" ? null : Number(w));
  }
  function _(O, w, I) {
    return I ? O <= 0 ? "\u26A0" : "\u2713" : "\u2717";
  }
  function P(O, w, I) {
    return I ? O <= 0 ? "var(--color-warning)" : "var(--color-positive)" : "var(--color-negative)";
  }
  return (O, w) => (k(), C("article", { class: $e(["seat has-portrait", { self: e.self, dead: !e.seat.alive, dying: e.seat.alive && e.seat.hp <= 0, "target-mode": e.isTargetMode, "target-selectable": e.isTargetMode && e.isSelectableTarget, "drop-target": e.isDropTarget, "current-turn": false }]), role: "listitem", "aria-label": `\u5EA7\u4F4D ${e.seat.seat} \u73A9\u5BB6 ${e.seat.player} \u6B66\u5C06 ${i.value}`, tabindex: e.isTargetMode && e.isSelectableTarget ? 0 : -1, onClick: w[2] || (w[2] = (I) => e.isTargetMode && e.isSelectableTarget && s("select", e.seat.player)), onKeydown: w[3] || (w[3] = gc((I) => e.isTargetMode && e.isSelectableTarget && s("select", e.seat.player), ["enter"])) }, [v.value ? (k(), C("img", { key: 0, class: "portrait", src: v.value, alt: `${i.value}\u6B66\u5C06\u5934\u50CF`, onError: m }, null, 40, kf)) : (k(), C("div", { key: 1, class: "portrait no-image", role: "img", "aria-label": `${i.value}\u6682\u65E0\u5934\u50CF` }, " \u65E0\u56FE ", 8, Ef)), f("header", null, [f("span", Cf, S(i.value), 1), e.self ? (k(), C("span", xf, "\u4F60")) : J("", true), e.seat.turned_over ? (k(), C("span", Af, "\u7FFB\u9762")) : J("", true)]), f("div", Rf, [f("span", { class: "state-icon", style: Tt({ color: P(e.seat.hp, e.seat.max_hp, e.seat.alive) }), "aria-label": e.seat.alive ? e.seat.hp <= 0 ? "\u6FD2\u6B7B" : "\u5B58\u6D3B" : "\u9635\u4EA1" }, S(_(e.seat.hp, e.seat.max_hp, e.seat.alive)), 13, Pf), f("span", Of, S(e.seat.hp) + " / " + S(e.seat.max_hp), 1)]), f("div", { class: "hp-bar", role: "progressbar", "aria-valuenow": Math.max(e.seat.hp, 0), "aria-valuemax": e.seat.max_hp, "aria-label": `\u4F53\u529B ${e.seat.hp}/${e.seat.max_hp}` }, [f("div", { class: "hp-bar-fill", style: Tt({ width: Math.max(Math.min(e.seat.hp / e.seat.max_hp * 100, 100), 0) + "%", background: e.seat.alive ? e.seat.hp <= 0 ? "var(--color-warning)" : "var(--color-positive)" : "var(--color-negative)" }) }, null, 4)], 8, $f), f("div", If, [f("span", null, "\u624B\u724C " + S(e.seat.hand_count), 1), f("span", null, S(e.identityLabel), 1)]), r.value.length > 0 ? (k(), C("select", { key: 2, class: "skin-select", value: o.value ?? "", "aria-label": `${i.value}\u76AE\u80A4`, onClick: w[0] || (w[0] = qs(() => {
  }, ["stop"])), onKeydown: w[1] || (w[1] = qs(() => {
  }, ["stop"])), onChange: b }, [w[4] || (w[4] = f("option", { value: "" }, "\u9ED8\u8BA4\u76AE\u80A4", -1)), (k(true), C(ce, null, Ce(r.value, (I) => (k(), C("option", { key: I.id, value: I.id }, S(z(ef)(I)) + S(I.presentation.kind === "spine" ? "\uFF08\u9759\u6001\u56DE\u9000\uFF09" : ""), 9, Nf))), 128))], 40, Tf)) : J("", true), e.equipCards.length > 0 ? (k(), C("div", Mf, [(k(true), C(ce, null, Ce(e.equipCards, (I) => (k(), C("div", { key: I.id, class: "equip-card", title: `\u5361\u724C #${I.id}${I.def != null ? " (def:" + I.def + ")" : ""}` }, [f("span", Lf, S(I.suit === "Spade" ? "\u2660" : I.suit === "Heart" ? "\u2665" : I.suit === "Club" ? "\u2663" : I.suit === "Diamond" ? "\u2666" : "?"), 1), f("span", jf, "#" + S(I.id), 1)], 8, Df))), 128))])) : J("", true), e.seat.alive ? e.seat.hp <= 0 ? (k(), C("div", Vf, "\u26A0 \u6FD2\u6B7B")) : J("", true) : (k(), C("div", Uf, "\u2717 \u9635\u4EA1")), e.isTargetMode && e.isSelectableTarget ? (k(), C("div", Hf, " \u25B8 \u70B9\u51FB\u9009\u62E9\u4E3A\u76EE\u6807 ")) : J("", true)], 42, Sf));
} }), Wf = Lt(Ff, [["__scopeId", "data-v-cb77ea59"]]), Bf = { class: "hand", "aria-label": "\u624B\u724C" }, qf = { class: "muted title" }, Kf = { class: "strip", role: "listbox", "aria-label": "\u624B\u724C\u5217\u8868" }, Gf = ["disabled", "aria-selected", "aria-label", "onClick", "onDragstart"], zf = ["data-suit"], Jf = { class: "rank" }, Qf = { class: "id muted" }, Yf = { key: 0, class: "card-known" }, Xf = { key: 0, class: "muted empty" }, Zf = { key: 0, class: "hint" }, ed = { key: 1, class: "hint drag-hint" }, td = nt({ __name: "HandStrip", props: { cards: {}, actionableIds: {} }, emits: ["select", "dragStart"], setup(e, { emit: t }) {
  const n = e, s = t, i = X(null), r = X(null);
  function o(c) {
    switch (c) {
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
  function l(c) {
    return c == null ? "?" : c === 1 ? "A" : c === 11 ? "J" : c === 12 ? "Q" : c === 13 ? "K" : String(c);
  }
  function a(c) {
    n.actionableIds.includes(c) && (i.value = c, s("select", c));
  }
  function p(c, v) {
    if (!n.actionableIds.includes(v)) {
      c.preventDefault();
      return;
    }
    r.value = v, c.dataTransfer && (c.dataTransfer.setData("text/plain", String(v)), c.dataTransfer.effectAllowed = "move"), s("dragStart", v);
  }
  function u() {
    r.value = null;
  }
  return (c, v) => (k(), C("section", Bf, [f("h2", qf, " \u624B\u724C\uFF08" + S(e.cards.length) + " \u5F20" + S(i.value != null ? " \xB7 \u5DF2\u9009\u4E2D #" + i.value : " \xB7 \u70B9\u51FB\u6216\u62D6\u62FD") + "\uFF09 ", 1), f("div", Kf, [(k(true), C(ce, null, Ce(e.cards, (m) => (k(), C("button", { key: m.id, type: "button", class: $e(["card", { selected: m.id === i.value, dragging: m.id === r.value }]), disabled: !e.actionableIds.includes(m.id), role: "option", "aria-selected": m.id === i.value, "aria-label": `\u5361\u724C ${m.id}${m.def != null ? " \u5DF2\u77E5" : " \u672A\u77E5"}`, draggable: "true", onClick: (b) => a(m.id), onDragstart: (b) => p(b, m.id), onDragend: u }, [f("span", { class: "suit", "data-suit": m.suit }, S(o(m.suit)), 9, zf), f("span", Jf, S(l(m.rank)), 1), f("span", Qf, "#" + S(m.id), 1), m.def != null ? (k(), C("span", Yf, "\u25B8")) : J("", true)], 42, Gf))), 128)), e.cards.length ? J("", true) : (k(), C("p", Xf, "\u65E0\u53EF\u89C1\u624B\u724C"))]), i.value != null ? (k(), C("p", Zf, " \u2713 \u5DF2\u9009\u4E2D #" + S(i.value) + " \u2014 \u5728\u68CB\u76D8\u4E0A\u9009\u62E9\u76EE\u6807\uFF0C\u6216\u70B9\u51FB\u300C\u7ED3\u675F\u9636\u6BB5\u300D\u76F4\u63A5\u6253\u51FA ", 1)) : J("", true), r.value != null ? (k(), C("p", ed, " \u{1F0A0} \u62D6\u62FD\u4E2D #" + S(r.value) + " \u2014 \u62D6\u5230\u76EE\u6807\u73A9\u5BB6\u91CA\u653E\u51FA\u724C ", 1)) : J("", true)]));
} }), nd = Lt(td, [["__scopeId", "data-v-b2b16095"]]), sd = { class: "board", role: "region", "aria-label": "\u6218\u573A" }, id = { key: 0, class: "action-bar panel", role: "toolbar", "aria-label": "\u884C\u52A8\u64CD\u4F5C" }, rd = { class: "muted" }, od = { key: 2 }, ld = { key: 3 }, ad = { class: "row" }, cd = ["disabled"], ud = ["disabled"], fd = { key: 1, class: "skill-bar panel", "aria-label": "\u53EF\u53D1\u52A8\u6280\u80FD" }, dd = ["onClick"], pd = { class: "seats", role: "list", "aria-label": "\u5EA7\u4F4D\u5217\u8868" }, hd = ["onDragover", "onDragleave", "onDrop"], vd = { key: 2, class: "judgment-zone", "aria-label": "\u5224\u5B9A\u533A" }, gd = { class: "jz-entries" }, md = ["title"], yd = { class: "jz-suit" }, _d = { class: "jz-player" }, bd = { key: 3, class: "discard-pile", "aria-label": "\u5F03\u724C\u5806" }, wd = { class: "muted discard-title" }, Sd = { class: "discard-strip" }, kd = ["title"], Ed = { class: "d-suit" }, Cd = { key: 0, class: "muted discard-more" }, xd = nt({ __name: "BattleBoard", props: { view: {}, selfId: {}, legalActions: {} }, emits: ["playCard", "activateSkill"], setup(e, { emit: t }) {
  const n = e, s = t, i = X(null), r = X(null), o = X(false), l = X([]), a = X(null), p = X(null);
  function u() {
    var _a2;
    return ((_a2 = n.view) == null ? void 0 : _a2.seats) ?? [];
  }
  function c() {
    return n.view ? n.view.known_cards.filter((N) => N.owner === n.selfId && N.zone === "Hand" && N.def != null) : [];
  }
  function v() {
    var _a2;
    const N = new Set(n.legalActions.flatMap((j) => typeof j == "object" && "ActivateSkill" in j ? [j.ActivateSkill.skill] : []));
    return ((_a2 = u().find((j) => j.player === n.selfId)) == null ? void 0 : _a2.skills.filter((j) => j.enabled && (j.active || j.view_as) && N.has(j.id))) ?? [];
  }
  function m(N) {
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
    return o.value ? new Set(ie().flat()) : /* @__PURE__ */ new Set();
  }
  function _() {
    return n.view ? n.view.known_cards.filter((N) => N.zone === "DiscardPile" && N.def != null) : [];
  }
  function P(N) {
    return n.view ? n.view.known_cards.filter((j) => j.owner === N && j.zone === "Judge" && j.def != null) : [];
  }
  const O = se(() => b()), w = se(() => r.value ? Array.from(new Set(F(r.value.id).flatMap((N) => N.card_targets))) : Array.from(new Set(I().map((N) => N.card))));
  function I(N) {
    return n.legalActions.flatMap((j) => typeof j != "object" || !("PlayCard" in j) ? [] : N != null && j.PlayCard.card !== N ? [] : [j.PlayCard]);
  }
  function F(N) {
    return n.legalActions.flatMap((j) => typeof j != "object" || !("ActivateSkill" in j) ? [] : j.ActivateSkill.skill === N ? [j.ActivateSkill] : []);
  }
  function ie() {
    return r.value ? F(r.value.id).filter((N) => i.value == null ? N.card_targets.length === 0 : N.card_targets.length === 1 && N.card_targets[0] === i.value).map((N) => N.targets) : i.value == null ? [] : I(i.value).map((N) => N.targets);
  }
  function he(N, j) {
    return N.length === j.length && N.every((W, ne) => W === j[ne]);
  }
  const ae = se(() => ie().some((N) => he(N, l.value))), Me = se(() => ie().some((N) => N.length === 0));
  function He(N) {
    return n.view ? n.view.known_cards.filter((j) => j.owner === N && j.zone === "Equip" && j.def != null) : [];
  }
  function Fe(N) {
    if (w.value.includes(N)) {
      if (i.value === N) {
        i.value = null, o.value = false, l.value = [];
        return;
      }
      i.value = N, r.value || (r.value = null), o.value = true, l.value = [];
    }
  }
  function ze(N) {
    var _a2;
    if (((_a2 = r.value) == null ? void 0 : _a2.id) === N.id) {
      Le();
      return;
    }
    r.value = N, i.value = null, o.value = true, l.value = [];
  }
  function Qe(N) {
    w.value.includes(N) && (a.value = N, i.value = N, o.value = true);
  }
  function De(N) {
    if (!(!o.value || !O.value.has(N))) if (l.value.includes(N)) l.value = l.value.filter((j) => j !== N);
    else {
      const j = Math.max(0, ...ie().map((W) => W.length));
      l.value = [...l.value, N].slice(0, j);
    }
  }
  function st(N, j) {
    a.value != null && O.value.has(j) && (N.preventDefault(), N.dataTransfer.dropEffect = "move", p.value = j);
  }
  function it(N) {
    p.value === N && (p.value = null);
  }
  function be(N, j) {
    var _a2;
    N.preventDefault();
    const W = a.value;
    if (p.value = null, a.value = null, W != null && O.value.has(j)) {
      if ((_a2 = r.value) == null ? void 0 : _a2.view_as) {
        const ne = F(r.value.id).find((T) => he(T.targets, [j]) && T.card_targets.length === 1 && T.card_targets[0] === W);
        if (!ne) return;
        s("activateSkill", ne.skill, ne.targets, ne.card_targets);
      } else {
        const ne = I(W).find((T) => he(T.targets, [j]));
        if (!ne) return;
        s("playCard", ne.card, ne.targets);
      }
      i.value = null, r.value = null, o.value = false, l.value = [];
    }
  }
  function oe() {
    if (ae.value) {
      if (r.value) {
        const N = F(r.value.id).find((j) => he(j.targets, l.value) && (i.value == null ? j.card_targets.length === 0 : j.card_targets.length === 1 && j.card_targets[0] === i.value));
        if (!N) return;
        s("activateSkill", N.skill, N.targets, N.card_targets);
      } else {
        if (i.value == null) return;
        const N = I(i.value).find((j) => he(j.targets, l.value));
        if (!N) return;
        s("playCard", N.card, N.targets);
      }
      i.value = null, r.value = null, o.value = false, l.value = [];
    }
  }
  function te() {
    l.value = [], oe();
  }
  function Le() {
    i.value = null, r.value = null, o.value = false, l.value = [];
  }
  return (N, j) => {
    var _a2;
    return k(), C("div", sd, [i.value != null || r.value ? (k(), C("div", id, [f("span", rd, [j[0] || (j[0] = ye(" \u25B8 ", -1)), r.value ? (k(), C(ce, { key: 0 }, [ye("\u6280\u80FD #" + S(r.value.def), 1)], 64)) : (k(), C(ce, { key: 1 }, [ye("\u624B\u724C #" + S(i.value), 1)], 64)), ((_a2 = r.value) == null ? void 0 : _a2.view_as) && i.value != null ? (k(), C("span", od, " \xB7 \u7D20\u6750 #" + S(i.value), 1)) : J("", true), o.value && l.value.length > 0 ? (k(), C("span", ld, " \u2192 \u76EE\u6807 P" + S(l.value[0]), 1)) : J("", true)]), f("div", ad, [f("button", { class: "primary", type: "button", onClick: oe, disabled: !ae.value, "aria-label": "\u786E\u8BA4\u884C\u52A8" }, S(r.value ? "\u53D1\u52A8\u6280\u80FD" : "\u786E\u8BA4\u51FA\u724C"), 9, cd), f("button", { type: "button", disabled: !Me.value, onClick: te, "aria-label": "\u65E0\u76EE\u6807\u76F4\u63A5\u6253\u51FA" }, S(r.value ? "\u65E0\u76EE\u6807\u53D1\u52A8" : "\u76F4\u63A5\u6253\u51FA"), 9, ud), f("button", { type: "button", onClick: Le, "aria-label": "\u53D6\u6D88\u9009\u62E9" }, " \u53D6\u6D88 ")])])) : J("", true), v().length ? (k(), C("section", fd, [j[1] || (j[1] = f("span", { class: "muted" }, "\u6280\u80FD", -1)), (k(true), C(ce, null, Ce(v(), (W) => {
      var _a3;
      return k(), C("button", { key: W.id, type: "button", class: $e({ primary: ((_a3 = r.value) == null ? void 0 : _a3.id) === W.id }), onClick: (ne) => ze(W) }, " #" + S(W.def) + S(W.view_as ? " \xB7 \u89C6\u4E3A" : ""), 11, dd);
    }), 128))])) : J("", true), f("div", pd, [(k(true), C(ce, null, Ce(u(), (W) => (k(), C("div", { key: W.player, class: "seat-wrapper", onDragover: (ne) => st(ne, W.player), onDragleave: (ne) => it(W.player), onDrop: (ne) => be(ne, W.player) }, [we(Wf, { seat: W, self: W.player === e.selfId, "identity-label": m(W.identity), "is-target-mode": o.value, "is-selectable-target": O.value.has(W.player), "is-drop-target": p.value === W.player, "equip-cards": He(W.player), onSelect: De }, null, 8, ["seat", "self", "identity-label", "is-target-mode", "is-selectable-target", "is-drop-target", "equip-cards"])], 40, hd))), 128))]), we(nd, { cards: c(), "actionable-ids": w.value, onSelect: Fe, onDragStart: Qe }, null, 8, ["cards", "actionable-ids"]), u().some((W) => P(W.player).length > 0) ? (k(), C("section", vd, [j[2] || (j[2] = f("h3", { class: "muted jz-title" }, "\u2696 \u5224\u5B9A\u533A", -1)), f("div", gd, [(k(true), C(ce, null, Ce(u(), (W) => (k(), C("div", { key: "jz-" + W.player, class: "jz-entry" }, [(k(true), C(ce, null, Ce(P(W.player), (ne) => (k(), C("span", { key: ne.id, class: "jz-card", title: `P${W.player} #${ne.id}` }, [f("span", yd, S(ne.suit === "Spade" ? "\u2660" : ne.suit === "Heart" ? "\u2665" : ne.suit === "Club" ? "\u2663" : ne.suit === "Diamond" ? "\u2666" : "?"), 1), f("span", _d, "P" + S(W.player), 1)], 8, md))), 128))]))), 128))])])) : J("", true), _().length > 0 ? (k(), C("section", bd, [f("h3", wd, "\u5F03\u724C\u5806\uFF08" + S(_().length) + "\uFF09", 1), f("div", Sd, [(k(true), C(ce, null, Ce(_().slice(-10), (W) => (k(), C("div", { key: W.id, class: "discard-card", title: `#${W.id} def:${W.def}` }, [f("span", Ed, S(W.suit === "Spade" ? "\u2660" : W.suit === "Heart" ? "\u2665" : W.suit === "Club" ? "\u2663" : W.suit === "Diamond" ? "\u2666" : "?"), 1)], 8, kd))), 128)), _().length > 10 ? (k(), C("span", Cd, "+" + S(_().length - 10), 1)) : J("", true)])])) : J("", true)]);
  };
} }), Ad = Lt(xd, [["__scopeId", "data-v-9182a2db"]]), Rd = { class: "battle stack" }, Pd = { key: 0, class: "panel hotkey-overlay", role: "dialog", "aria-label": "\u5FEB\u6377\u952E" }, Od = { class: "outcome-content" }, $d = { class: "outcome-icon" }, Id = { class: "outcome-label" }, Td = { class: "muted" }, Nd = { key: 2, class: "panel turn-bar", role: "status", "aria-live": "polite" }, Md = { class: "turn-player" }, Dd = ["data-phase"], Ld = { class: "panel toolbar", "aria-label": "\u5BF9\u5C40\u63A7\u5236" }, jd = { class: "row" }, Ud = ["disabled"], Vd = { key: 1, class: "online-badge" }, Hd = ["disabled"], Fd = ["disabled"], Wd = ["disabled"], Bd = { class: "row muted controls" }, qd = { class: "check" }, Kd = { class: "status", role: "status", "aria-live": "polite" }, Gd = { key: 0, class: "error", role: "alert" }, zd = { key: 3, class: "panel online-roster", "aria-label": "\u8054\u673A\u623F\u95F4\u5EA7\u4F4D" }, Jd = { class: "row online-roster-head" }, Qd = { class: "row online-seats" }, Yd = { key: 4, class: "panel choice", "aria-label": "\u5F85\u9009\u62E9" }, Xd = { class: "muted" }, Zd = { key: 0, class: "row" }, ep = ["onClick"], tp = { key: 1, class: "muted" }, np = { key: 5, class: "panel roster", "aria-label": "\u73A9\u5BB6\u6982\u51B5" }, sp = { class: "roster-grid" }, ip = { class: "roster-player" }, rp = { class: "roster-hand muted" }, op = ["data-id"], lp = { key: 1, class: "roster-id muted" }, ap = { class: "panel battle-section" }, cp = { key: 0, class: "effect-overlay", "aria-hidden": "true" }, up = { class: "effect-icon" }, fp = { class: "effect-text" }, dp = { key: 0, class: "effect-player" }, pp = { key: 6, class: "panel match-stats", "aria-label": "\u5BF9\u5C40\u7EDF\u8BA1" }, hp = { class: "stats-grid" }, vp = { class: "stat" }, gp = { class: "stat-value mono" }, mp = { class: "stat" }, yp = { class: "stat-value mono" }, _p = { class: "stat" }, bp = { class: "stat-value" }, wp = { class: "stat" }, Sp = { class: "stat-value" }, kp = { key: 7, class: "panel seat-status", "aria-label": "\u5EA7\u4F4D\u72B6\u6001\u4E00\u89C8" }, Ep = { class: "seat-status-grid" }, Cp = { class: "ss-header" }, xp = { class: "ss-player" }, Ap = { key: 0, class: "ss-tag" }, Rp = { class: "ss-hp-bar" }, Pp = { class: "ss-hp-text" }, Op = { class: "ss-icon" }, $p = { class: "ss-meta muted" }, Ip = ["data-id"], Tp = { key: 1, class: "muted" }, Np = { class: "panel log", "aria-label": "\u4E8B\u4EF6\u65E5\u5FD7" }, Mp = { class: "log-header" }, Dp = { class: "log-list" }, Lp = { class: "log-rev" }, jp = { class: "log-text" }, Up = { key: 0, class: "muted empty-log" }, Vp = nt({ __name: "BattleView", setup(e) {
  const t = xu(), n = Bo(), s = gf(), i = new qo(), r = new cf(), o = X(1), l = X(Number(t.query.players ?? 4) || 4), a = X(Number(t.query.seed ?? 42) || 42), p = X(null), u = X(false), c = X("\u672A\u8FDE\u63A5"), v = X(null), m = X(0), b = X(false), _ = se(() => t.query.mode === "online"), P = se(() => _.value && o.value === 0);
  async function O(T) {
    s.busy = true, s.error = null;
    try {
      return await T();
    } catch ($) {
      s.error = $ instanceof Error ? $.message : String($), c.value = s.error;
      return;
    } finally {
      s.busy = false;
    }
  }
  async function w(T) {
    mf(s, T), await wf(s, 120), _f(s);
    const $ = _.value ? `\u623F\u95F4 #${m.value} \xB7 ` : "";
    c.value = T.finished ? `${$}\u7EC8\u5C40 \xB7 rev ${T.revision}` : `${$}rev ${T.revision} \xB7 ${T.hash_hex.slice(0, 12)}\u2026`;
  }
  async function I() {
    await O(async () => {
      if (await _r(true), _.value) {
        const $ = ie();
        Qu($.wsUrl), await F($);
        return;
      }
      Ju(), await i.connect(), s.connected = true, c.value = "Worker \u5C31\u7EEA\uFF0C\u5F00\u5C40\u4E2D\u2026";
      const T = await i.start(l.value, a.value, o.value);
      await w(T), _n(s, `\u5F00\u5C40 ${l.value} \u4EBA seed=${a.value}`);
    });
  }
  async function F(T) {
    r.onEnvelope((Z) => {
      w(Z);
    }), r.onRoster((Z) => {
      v.value = Z, m.value = Z.room_id;
      const g = Z.started && !b.value;
      b.value = Z.started, c.value = Z.started ? `\u623F\u95F4 #${Z.room_id} \xB7 \u5DF2\u5F00\u5C40` : `\u623F\u95F4 #${Z.room_id} \xB7 \u7B49\u5F85\u5165\u5EA7 ${Z.seats.length}/${l.value}`, g && r.requestSnapshot().catch(() => {
      });
    }), c.value = `\u8FDE\u63A5 ${T.wsUrl}\u2026`, await r.connect(T.wsUrl, T.displayName);
    let $;
    if (T.action === "create") $ = await r.createRoom(T.players, T.seed, T.password);
    else if (T.action === "spectate") $ = await r.spectate(T.roomId);
    else {
      const Z = localStorage.getItem(he(T.roomId));
      $ = await r.joinRoom(T.roomId, T.password, Z);
    }
    m.value = $.roomId, o.value = $.playerId, l.value = T.players, $.role === "Player" && $.reconnectToken && localStorage.setItem(he($.roomId), $.reconnectToken), s.connected = true, await w($.envelope), _n(s, $.role === "Spectator" ? `\u65C1\u89C2\u623F\u95F4 #${$.roomId}` : `\u52A0\u5165\u623F\u95F4 #${$.roomId} \xB7 \u5EA7\u4F4D ${$.seat + 1}`);
  }
  function ie() {
    const T = sessionStorage.getItem("noname-next.online-intent");
    if (T) try {
      return JSON.parse(T);
    } catch {
      sessionStorage.removeItem("noname-next.online-intent");
    }
    const $ = window.location.protocol === "https:";
    return { action: String(t.query.action ?? "join"), wsUrl: `${$ ? "wss" : "ws"}://${window.location.hostname || "127.0.0.1"}:7700/ws`, displayName: "player", password: "", roomId: Number(t.query.room ?? 1) || 1, players: l.value, seed: a.value };
  }
  function he(T) {
    return `noname-next.reconnect.${T}`;
  }
  function ae() {
    return _.value ? r : i;
  }
  async function Me() {
    await O(async () => {
      const T = await ae().endPhase(o.value);
      await w(T);
    });
  }
  async function He() {
    await O(async () => {
      if (_.value) throw new Error("\u8054\u673A\u6A21\u5F0F\u4E0D\u63D0\u4F9B\u5BA2\u6237\u7AEF\u81EA\u52A8\u6B65\u8FDB");
      const T = await i.autoStep();
      await w(T);
    });
  }
  async function Fe(T, $) {
    await O(async () => {
      const Z = await ae().playCard(o.value, T, $);
      await w(Z), _n(s, `\u51FA\u724C #${T}${$.length > 0 ? ` \u2192 P${$[0]}` : ""}`);
    });
  }
  async function ze(T, $, Z) {
    await O(async () => {
      const g = await ae().activateSkill(o.value, T, $, Z);
      await w(g), _n(s, `\u53D1\u52A8\u6280\u80FD #${T}${$.length > 0 ? ` \u2192 P${$[0]}` : ""}`);
    });
  }
  async function Qe(T) {
    await O(async () => {
      const $ = await ae().resolveChoice(o.value, T.ResolveChoice.choice_id, T.ResolveChoice.selections, T.ResolveChoice.cancelled);
      await w($);
    });
  }
  const De = se(() => s.legalActions.filter((T) => typeof T == "object" && "ResolveChoice" in T)), st = se(() => s.legalActions.includes("EndPhase"));
  function it() {
    if (u.value) {
      be();
      return;
    }
    u.value = true;
    const T = async () => {
      if (!u.value || s.finished) {
        be();
        return;
      }
      if (await He(), u.value && !s.finished) {
        const $ = s.skipAnims || s.reduceMotion ? 30 : 280 / s.speed;
        p.value = window.setTimeout(T, $);
      } else be();
    };
    T();
  }
  function be() {
    u.value = false, p.value != null && (clearTimeout(p.value), p.value = null);
  }
  function oe(T) {
    return T.includes("\u4F24\u5BB3") || T.includes("\u9635\u4EA1") ? "log-damage" : T.includes("\u56DE\u590D") || T.includes("\u6478\u724C") ? "log-positive" : T.includes("\u6FD2\u6B7B") || T.includes("\u7B49\u5F85") ? "log-warning" : T.includes("\u56DE\u5408") || T.includes("\u4F7F\u7528") ? "log-action" : "";
  }
  const te = X(false);
  function Le(T) {
    if (T.key === "?") {
      te.value = !te.value;
      return;
    }
    if (T.key === "Escape") {
      te.value = false;
      return;
    }
    s.busy || s.finished || (T.key === " " ? (T.preventDefault(), _.value || He()) : T.key === "e" || T.key === "E" ? st.value && Me() : (T.key === "s" || T.key === "S") && (s.skipAnims = !s.skipAnims));
  }
  Ln(() => {
    window.addEventListener("keydown", Le), I();
  }), fs(() => {
    window.removeEventListener("keydown", Le), be(), i.dispose(), r.dispose(), _r(false);
  }), kt(() => s.speed, () => {
  });
  const N = { Draw: "\u6478\u724C", Play: "\u51FA\u724C", Discard: "\u5F03\u724C", Judge: "\u5224\u5B9A" }, j = se(() => {
    var _a2;
    const T = s.lastEvents ?? [];
    for (const g of T) {
      const V = g;
      if ((V == null ? void 0 : V.type) === "MatchEnded" && V.outcome) {
        const L = Object.keys(V.outcome)[0];
        if (L === "RebelWin") return { winner: "\u53CD\u8D3C", label: "\u53CD\u8D3C\u80DC\u5229\uFF01", isSelfWin: false };
        if (L === "LoyalistWin") return { winner: "\u5FE0\u81E3", label: "\u5FE0\u81E3/\u4E3B\u516C\u80DC\u5229\uFF01", isSelfWin: true };
        if (L === "RenegadeWin") return { winner: "\u5185\u5978", label: "\u5185\u5978\u80DC\u5229\uFF01", isSelfWin: false };
        if (L === "Draw") return { winner: "\u5E73\u5C40", label: "\u5E73\u5C40", isSelfWin: false };
        if (L === "Aborted") return { winner: "\u4E2D\u6B62", label: "\u5BF9\u5C40\u4E2D\u6B62", isSelfWin: false };
      }
    }
    const $ = (_a2 = s.view) == null ? void 0 : _a2.outcome, Z = typeof $ == "string" ? $ : Object.keys($ ?? {})[0];
    return Z === "RebelWin" ? { winner: "\u53CD\u8D3C", label: "\u53CD\u8D3C\u80DC\u5229\uFF01", isSelfWin: false } : Z === "LoyalistWin" ? { winner: "\u5FE0\u81E3", label: "\u5FE0\u81E3/\u4E3B\u516C\u80DC\u5229\uFF01", isSelfWin: true } : Z === "RenegadeWin" ? { winner: "\u5185\u5978", label: "\u5185\u5978\u80DC\u5229\uFF01", isSelfWin: false } : Z === "Draw" ? { winner: "\u5E73\u5C40", label: "\u5E73\u5C40", isSelfWin: false } : Z === "Aborted" ? { winner: "\u4E2D\u6B62", label: "\u5BF9\u5C40\u4E2D\u6B62", isSelfWin: false } : null;
  });
  function W() {
    n.push("/");
  }
  const ne = se(() => {
    var _a2;
    if (s.view && s.view.current_player !== 0 && s.view.phase !== "Idle") return { currentPlayer: s.view.current_player, phase: s.view.phase, phaseLabel: N[s.view.phase] ?? s.view.phase, round: s.view.round };
    const T = s.lastEvents ?? [];
    for (const $ of T) {
      const Z = $;
      if ((Z == null ? void 0 : Z.type) === "TurnStateChanged") return { currentPlayer: Z.current ?? 0, phase: Z.phase ?? "?", phaseLabel: N[Z.phase ?? ""] ?? Z.phase ?? "?", round: ((_a2 = s.view) == null ? void 0 : _a2.round) ?? 0 };
    }
    return null;
  });
  return (T, $) => {
    const Z = ta("auto-scroll");
    return k(), C("div", Rd, [te.value ? (k(), C("section", Pd, [...$[4] || ($[4] = [Eo('<h2 data-v-40fe76d3>\u2328 \u5FEB\u6377\u952E</h2><dl class="hotkey-list" data-v-40fe76d3><dt data-v-40fe76d3><kbd data-v-40fe76d3>Space</kbd></dt><dd data-v-40fe76d3>\u81EA\u52A8\u4E00\u6B65</dd><dt data-v-40fe76d3><kbd data-v-40fe76d3>E</kbd></dt><dd data-v-40fe76d3>\u7ED3\u675F\u9636\u6BB5</dd><dt data-v-40fe76d3><kbd data-v-40fe76d3>S</kbd></dt><dd data-v-40fe76d3>\u5207\u6362\u8DF3\u8FC7\u52A8\u753B</dd><dt data-v-40fe76d3><kbd data-v-40fe76d3>?</kbd></dt><dd data-v-40fe76d3>\u663E\u793A/\u9690\u85CF\u6B64\u9762\u677F</dd><dt data-v-40fe76d3><kbd data-v-40fe76d3>Esc</kbd></dt><dd data-v-40fe76d3>\u5173\u95ED\u6B64\u9762\u677F</dd></dl>', 2)])])) : J("", true), z(s).finished && j.value ? (k(), C("section", { key: 1, class: $e(["panel outcome-banner", { win: j.value.isSelfWin, lose: !j.value.isSelfWin }]), role: "alert", "aria-live": "assertive" }, [f("div", Od, [f("span", $d, S(j.value.isSelfWin ? "\u{1F389}" : "\u{1F480}"), 1), f("div", null, [f("h2", Id, S(j.value.label), 1), f("p", Td, S(j.value.winner) + "\u9635\u8425\u83B7\u80DC \xB7 rev " + S(z(s).revision), 1)])]), f("button", { class: "primary", type: "button", onClick: $[0] || ($[0] = (g) => _.value ? W() : I()) }, S(_.value ? "\u8FD4\u56DE\u5927\u5385" : "\u518D\u6765\u4E00\u5C40"), 1)], 2)) : J("", true), ne.value ? (k(), C("section", Nd, [f("span", Md, "\u25BA \u7B2C " + S(ne.value.round) + " \u8F6E \xB7 P" + S(ne.value.currentPlayer), 1), f("span", { class: "phase-badge", "data-phase": ne.value.phase }, S(ne.value.phaseLabel), 9, Dd)])) : J("", true), f("section", Ld, [f("div", jd, [_.value ? (k(), C("span", Vd, "\u8054\u673A\u623F\u95F4 #" + S(m.value || "\u2026"), 1)) : (k(), C("button", { key: 0, class: "primary", type: "button", disabled: z(s).busy, onClick: I }, " \u91CD\u65B0\u5F00\u5C40 ", 8, Ud)), f("button", { type: "button", disabled: z(s).busy || z(s).finished || P.value || _.value && !b.value || !st.value, onClick: Me }, " \u7ED3\u675F\u9636\u6BB5 ", 8, Hd), _.value ? J("", true) : (k(), C("button", { key: 2, type: "button", disabled: z(s).busy || z(s).finished, onClick: He }, " \u81EA\u52A8\u4E00\u6B65 ", 8, Fd)), _.value ? J("", true) : (k(), C("button", { key: 3, type: "button", disabled: z(s).busy, onClick: it }, S(u.value ? "\u505C\u6B62\u81EA\u52A8" : "\u8FDE\u7EED\u81EA\u52A8"), 9, Wd))]), f("div", Bd, [f("label", null, [$[6] || ($[6] = ye(" \u500D\u901F ", -1)), Ae(f("select", { "onUpdate:modelValue": $[1] || ($[1] = (g) => z(s).speed = g), "aria-label": "\u64AD\u653E\u500D\u901F" }, [...$[5] || ($[5] = [f("option", { value: 0.5 }, "0.5\xD7", -1), f("option", { value: 1 }, "1\xD7", -1), f("option", { value: 2 }, "2\xD7", -1), f("option", { value: 4 }, "4\xD7", -1)])], 512), [[Kt, z(s).speed, void 0, { number: true }]])]), f("label", qd, [Ae(f("input", { "onUpdate:modelValue": $[2] || ($[2] = (g) => z(s).skipAnims = g), type: "checkbox" }, null, 512), [[Gn, z(s).skipAnims]]), $[7] || ($[7] = ye(" \u8DF3\u8FC7\u52A8\u753B ", -1))]), f("span", null, S(P.value ? "\u516C\u5171\u65C1\u89C2\u6295\u5F71" : `\u89C2\u5BDF\u8005 P${o.value}`), 1)]), f("p", Kd, S(c.value), 1), z(s).error ? (k(), C("p", Gd, S(z(s).error), 1)) : J("", true)]), _.value && v.value ? (k(), C("section", zd, [f("div", Jd, [f("h2", null, "\u623F\u95F4 #" + S(v.value.room_id), 1), f("span", { class: $e(v.value.started ? "started" : "waiting") }, S(v.value.started ? "\u5BF9\u5C40\u8FDB\u884C\u4E2D" : "\u7B49\u5F85\u73A9\u5BB6"), 3)]), f("div", Qd, [(k(true), C(ce, null, Ce(v.value.seats, (g) => (k(), C("span", { key: g.seat, class: "online-seat" }, [ye(S(g.seat + 1) + " \xB7 " + S(g.display_name) + " ", 1), f("i", { class: $e(g.connected ? "connected" : "offline") }, S(g.connected ? "\u5728\u7EBF" : "\u79BB\u7EBF"), 3)]))), 128))])])) : J("", true), z(s).choice ? (k(), C("section", Yd, [$[8] || ($[8] = f("h2", null, "\u9009\u62E9\u5C4F\u969C", -1)), f("p", Xd, " choice #" + S(z(s).choice.choice_id) + " \xB7 \u73A9\u5BB6 " + S(z(s).choice.observer) + " \xB7 " + S(z(s).choice.kind ?? "Custom"), 1), De.value.length ? (k(), C("div", Zd, [(k(true), C(ce, null, Ce(De.value, (g, V) => (k(), C("button", { key: V, type: "button", onClick: (L) => Qe(g) }, S(g.ResolveChoice.cancelled ? "\u53D6\u6D88" : `\u9009\u62E9 ${g.ResolveChoice.selections.join(", ") || "\u7A7A"}`), 9, ep))), 128))])) : (k(), C("p", tp, "\u5F53\u524D\u89C2\u5BDF\u8005\u65E0\u6743\u5904\u7406\u6B64\u9009\u62E9\u3002"))])) : J("", true), z(s).view ? (k(), C("section", np, [$[9] || ($[9] = f("h2", null, "\u73A9\u5BB6\u6982\u51B5", -1)), f("div", sp, [(k(true), C(ce, null, Ce(z(s).view.seats, (g) => (k(), C("div", { key: g.player, class: $e(["roster-entry", { dead: !g.alive, self: g.player === o.value }]) }, [f("span", ip, "P" + S(g.player) + S(g.turned_over ? " \xB7 \u7FFB\u9762" : ""), 1), f("span", { class: "roster-hp", style: Tt({ color: g.alive ? g.hp <= 0 ? "var(--color-warning)" : "inherit" : "var(--color-negative)" }) }, S(g.alive ? g.hp <= 0 ? "\u26A0" : "" : "\u2717") + " " + S(g.hp) + "/" + S(g.max_hp), 5), f("span", rp, S(g.hand_count) + "\u{1F0A0}", 1), g.identity ? (k(), C("span", { key: 0, class: "roster-id", "data-id": g.identity }, S(g.identity === "Lord" ? "\u4E3B" : g.identity === "Loyalist" ? "\u5FE0" : g.identity === "Rebel" ? "\u53CD" : "\u5978"), 9, op)) : (k(), C("span", lp, "?"))], 2))), 128))])])) : J("", true), f("section", ap, [we(Ad, { view: z(s).view, "self-id": o.value, "legal-actions": z(s).legalActions, onPlayCard: Fe, onActivateSkill: ze }, null, 8, ["view", "self-id", "legal-actions"]), z(s).effects.length ? (k(), C("div", cp, [(k(true), C(ce, null, Ce(z(s).effects, (g) => (k(), C("div", { key: g.id, class: $e(["effect-popup", g.type]) }, [f("span", up, S(g.type === "damage" ? "\u2694" : g.type === "heal" ? "\u271A" : g.type === "draw" ? "\u{1F0A0}" : g.type === "play" ? "\u25B8" : g.type === "dying" ? "\u26A0" : "\u2717"), 1), f("span", fp, S(g.text), 1), g.player != null ? (k(), C("span", dp, "P" + S(g.player), 1)) : J("", true)], 2))), 128))])) : J("", true)]), z(s).view ? (k(), C("section", pp, [f("div", hp, [f("div", vp, [$[10] || ($[10] = f("span", { class: "stat-label muted" }, "Revision", -1)), f("span", gp, S(z(s).revision), 1)]), f("div", mp, [$[11] || ($[11] = f("span", { class: "stat-label muted" }, "State Hash", -1)), f("span", yp, S(z(s).hash.slice(0, 16)) + "\u2026", 1)]), f("div", _p, [$[12] || ($[12] = f("span", { class: "stat-label muted" }, "\u5DF2\u77E5\u5361\u724C", -1)), f("span", bp, S(z(s).view.known_cards.length), 1)]), f("div", wp, [$[13] || ($[13] = f("span", { class: "stat-label muted" }, "\u5B58\u6D3B\u73A9\u5BB6", -1)), f("span", Sp, S(z(s).view.seats.filter((g) => g.alive).length) + " / " + S(z(s).view.seats.length), 1)])])])) : J("", true), z(s).view ? (k(), C("section", kp, [$[14] || ($[14] = f("h2", null, "\u5EA7\u4F4D\u72B6\u6001", -1)), f("div", Ep, [(k(true), C(ce, null, Ce(z(s).view.seats, (g) => (k(), C("div", { key: "ss-" + g.player, class: $e(["ss-entry", { "ss-dead": !g.alive, "ss-self": g.player === o.value }]) }, [f("div", Cp, [f("span", xp, "P" + S(g.player), 1), g.player === o.value ? (k(), C("span", Ap, "\u4F60")) : J("", true)]), f("div", Rp, [f("div", { class: "ss-hp-fill", style: Tt({ width: Math.max(Math.min(g.hp / g.max_hp * 100, 100), 0) + "%", background: g.alive ? g.hp <= 0 ? "var(--color-warning)" : "var(--color-positive)" : "var(--color-negative)" }) }, null, 4)]), f("div", Pp, [f("span", Op, S(g.alive ? g.hp <= 0 ? "\u26A0" : "" : "\u2717"), 1), ye(" " + S(Math.max(g.hp, 0)) + "/" + S(g.max_hp), 1)]), f("div", $p, [f("span", null, S(g.hand_count) + "\u{1F0A0}", 1), g.identity ? (k(), C("span", { key: 0, class: "ss-id", "data-id": g.identity }, S(g.identity === "Lord" ? "\u4E3B" : g.identity === "Loyalist" ? "\u5FE0" : g.identity === "Rebel" ? "\u53CD" : "\u5978"), 9, Ip)) : (k(), C("span", Tp, "?"))])], 2))), 128))])])) : J("", true), f("section", Np, [f("div", Mp, [$[15] || ($[15] = f("h2", null, "\u6218\u6597\u65E5\u5FD7", -1)), z(s).log.length > 0 ? (k(), C("button", { key: 0, type: "button", class: "clear-btn", onClick: $[3] || ($[3] = (g) => z(s).log = []), "aria-label": "\u6E05\u7A7A\u65E5\u5FD7" }, "\u6E05\u7A7A")) : J("", true)]), $[16] || ($[16] = f("p", { class: "muted tiny" }, " \u65E5\u5FD7\u6765\u81EA DomainEvent \u6458\u8981\uFF0C\u4E0D\u662F\u72B6\u6001\u771F\u76F8\u6E90\u3002\u771F\u76F8\u6E90\u4E3A Command Log + \u6295\u5F71\uFF08\xA722\uFF09\u3002 ", -1)), Ae((k(), C("ol", Dp, [(k(true), C(ce, null, Ce(z(s).log.slice().reverse(), (g) => (k(), C("li", { key: g.id, class: $e(["log-entry", oe(g.text)]) }, [f("span", Lp, "rev" + S(g.at), 1), f("span", jp, S(g.text), 1)], 2))), 128))])), [[Z]]), z(s).log.length === 0 ? (k(), C("p", Up, "\u6682\u65E0\u4E8B\u4EF6")) : J("", true)])]);
  };
} }), Hp = Lt(Vp, [["__scopeId", "data-v-40fe76d3"]]), Fp = { class: "panel stack", "aria-labelledby": "settings-title" }, Wp = { class: "a11y-group" }, Bp = { class: "check" }, qp = { class: "check" }, Kp = { class: "check" }, Gp = { class: "select-row" }, zp = { class: "select-row" }, Jp = { class: "select-row" }, Sr = "noname-next-a11y", Qp = nt({ __name: "SettingsView", setup(e) {
  const t = X(typeof window < "u" && window.matchMedia("(prefers-reduced-motion: reduce)").matches), n = X(false), s = X(false), i = X("normal"), r = X("none"), o = X("default");
  function l() {
    if (typeof window > "u") return null;
    try {
      const u = localStorage.getItem(Sr);
      return u ? JSON.parse(u) : null;
    } catch {
      return null;
    }
  }
  function a() {
    if (typeof window > "u") return;
    const u = { highContrast: n.value, skipAnimations: s.value, fontScale: i.value, colorBlind: r.value, screenReader: o.value };
    localStorage.setItem(Sr, JSON.stringify(u));
  }
  function p() {
    const u = document.documentElement;
    u.dataset.contrast = n.value ? "high" : "", u.dataset.skipAnimations = s.value ? "true" : "false", u.dataset.fontScale = i.value, u.dataset.colorBlind = r.value !== "none" ? r.value : "";
  }
  return Ln(() => {
    const u = l();
    u && (n.value = u.highContrast, s.value = u.skipAnimations, i.value = u.fontScale || "normal", r.value = u.colorBlind || "none", o.value = u.screenReader || "default"), p();
  }), kt([n, s, i, r, o], () => {
    p(), a();
  }), (u, c) => (k(), C("section", Fp, [c[18] || (c[18] = f("h1", { id: "settings-title" }, "\u8BBE\u7F6E", -1)), c[19] || (c[19] = f("p", { class: "muted", role: "note" }, " \u8868\u73B0\u5C42\u8BBE\u7F6E\u4E0D\u5F71\u54CD\u89C4\u5219\u6838\u5FC3\uFF08\xA730 \u53EF\u8BBF\u95EE\u6027\uFF09\u3002\u6240\u6709\u8BBE\u7F6E\u5373\u65F6\u751F\u6548\u5E76\u4FDD\u5B58\u5230\u672C\u5730\u3002 ", -1)), f("fieldset", Wp, [c[15] || (c[15] = f("legend", null, "\u53EF\u8BBF\u95EE\u6027\uFF08\xA730\uFF09", -1)), f("label", Bp, [Ae(f("input", { "onUpdate:modelValue": c[0] || (c[0] = (v) => t.value = v), type: "checkbox", disabled: "", "aria-describedby": "reduce-motion-desc" }, null, 512), [[Gn, t.value]]), c[6] || (c[6] = f("span", null, "\u51CF\u5C11\u52A8\u6001\u6548\u679C\uFF08\u7CFB\u7EDF\u504F\u597D\uFF0C\u53EA\u8BFB\uFF09", -1))]), c[16] || (c[16] = f("p", { id: "reduce-motion-desc", class: "sr-only" }, " \u9075\u5FAA\u7CFB\u7EDF prefers-reduced-motion \u8BBE\u7F6E\u3002 ", -1)), f("label", qp, [Ae(f("input", { "onUpdate:modelValue": c[1] || (c[1] = (v) => s.value = v), type: "checkbox", "aria-describedby": "skip-anim-desc" }, null, 512), [[Gn, s.value]]), c[7] || (c[7] = f("span", null, "\u8DF3\u8FC7\u52A8\u753B\uFF08\u8DF3\u8FC7\u540E\u4ECD\u53EF\u7406\u89E3\u89C4\u5219\u7ED3\u679C\uFF09", -1))]), c[17] || (c[17] = f("p", { id: "skip-anim-desc", class: "sr-only" }, " \u8DF3\u8FC7\u6240\u6709\u52A8\u753B\uFF1B\u89C4\u5219\u7ED3\u679C\u4ECD\u901A\u8FC7\u6587\u5B57\u65E5\u5FD7\u548C\u56FE\u6807\u5B8C\u6574\u4F20\u8FBE\u3002 ", -1)), f("label", Kp, [Ae(f("input", { "onUpdate:modelValue": c[2] || (c[2] = (v) => n.value = v), type: "checkbox" }, null, 512), [[Gn, n.value]]), c[8] || (c[8] = f("span", null, "\u9AD8\u5BF9\u6BD4\u5EA6\u6A21\u5F0F", -1))]), f("label", Gp, [c[10] || (c[10] = f("span", null, "\u5B57\u53F7\u7F29\u653E", -1)), Ae(f("select", { "onUpdate:modelValue": c[3] || (c[3] = (v) => i.value = v), "aria-label": "\u5B57\u53F7\u7F29\u653E" }, [...c[9] || (c[9] = [f("option", { value: "small" }, "\u5C0F (85%)", -1), f("option", { value: "normal" }, "\u6807\u51C6 (100%)", -1), f("option", { value: "large" }, "\u5927 (125%)", -1), f("option", { value: "xlarge" }, "\u7279\u5927 (150%)", -1)])], 512), [[Kt, i.value]])]), f("label", zp, [c[12] || (c[12] = f("span", null, "\u8272\u89C9\u53CB\u597D\u6A21\u5F0F", -1)), Ae(f("select", { "onUpdate:modelValue": c[4] || (c[4] = (v) => r.value = v), "aria-label": "\u8272\u89C9\u53CB\u597D\u6A21\u5F0F" }, [...c[11] || (c[11] = [Eo('<option value="none" data-v-cbc7cec4>\u5173\u95ED</option><option value="protanopia" data-v-cbc7cec4>\u7EA2\u8272\u76F2\uFF08Protanopia\uFF09</option><option value="deuteranopia" data-v-cbc7cec4>\u7EFF\u8272\u76F2\uFF08Deuteranopia\uFF09</option><option value="tritanopia" data-v-cbc7cec4>\u84DD\u8272\u76F2\uFF08Tritanopia\uFF09</option><option value="achromatopsia" data-v-cbc7cec4>\u5168\u8272\u76F2\uFF08Achromatopsia\uFF09</option>', 5)])], 512), [[Kt, r.value]])]), f("label", Jp, [c[14] || (c[14] = f("span", null, "\u5C4F\u5E55\u9605\u8BFB\u5668\u8BE6\u7EC6\u5EA6", -1)), Ae(f("select", { "onUpdate:modelValue": c[5] || (c[5] = (v) => o.value = v), "aria-label": "\u5C4F\u5E55\u9605\u8BFB\u5668\u8BE6\u7EC6\u5EA6" }, [...c[13] || (c[13] = [f("option", { value: "off" }, "\u5173\u95ED", -1), f("option", { value: "default" }, "\u6807\u51C6", -1), f("option", { value: "verbose" }, "\u8BE6\u7EC6\uFF08\u64AD\u62A5\u6240\u6709\u72B6\u6001\u53D8\u5316\uFF09", -1)])], 512), [[Kt, o.value]])])]), c[20] || (c[20] = f("div", { class: "state-legend", role: "region", "aria-label": "\u72B6\u6001\u4FE1\u53F7\u56FE\u4F8B" }, [f("h2", null, "\u72B6\u6001\u4FE1\u53F7\u8BF4\u660E"), f("p", { class: "muted" }, " \u6240\u6709\u6E38\u620F\u72B6\u6001\u901A\u8FC7\u56FE\u6807 + \u6587\u5B57\u4F20\u8FBE\uFF0C\u4E0D\u53EA\u4F9D\u8D56\u989C\u8272\uFF08\xA730: \u4E0D\u53EA\u4F9D\u8D56\u989C\u8272\u8868\u8FBE\u72B6\u6001\uFF09\u3002 "), f("ul", { class: "legend-list" }, [f("li", null, [f("span", { class: "state-icon", style: { color: "var(--color-positive)" } }, "\u2713"), ye(" \u5B58\u6D3B")]), f("li", null, [f("span", { class: "state-icon", style: { color: "var(--color-warning)" } }, "\u26A0"), ye(" \u6FD2\u6B7B")]), f("li", null, [f("span", { class: "state-icon", style: { color: "var(--color-negative)" } }, "\u2717"), ye(" \u9635\u4EA1")]), f("li", null, [f("span", { class: "state-icon", style: { color: "var(--color-positive)" } }, "\u25B8"), ye(" \u53EF\u51FA\u724C")]), f("li", null, [f("span", { class: "state-icon", style: { color: "var(--color-negative)" } }, "\u2715"), ye(" \u4E0D\u53EF\u7528")]), f("li", null, [f("span", { class: "state-icon", style: { color: "var(--color-accent)" } }, "\u25BA"), ye(" \u5F53\u524D\u56DE\u5408")]), f("li", null, [f("span", { class: "state-icon", style: { color: "var(--color-warning)" } }, "?"), ye(" \u8BF7\u9009\u62E9")])])], -1))]));
} }), Yp = Lt(Qp, [["__scopeId", "data-v-cbc7cec4"]]), Xp = { class: "replay stack" }, Zp = { key: 0, class: "muted" }, eh = { key: 1, class: "error", role: "alert" }, th = { key: 0, class: "panel" }, nh = { class: "info-grid" }, sh = { class: "mono" }, ih = { class: "mono" }, rh = ["disabled"], oh = { key: 1, class: "panel" }, lh = { class: "row playback-controls" }, ah = ["disabled"], ch = ["disabled"], uh = { class: "step-display mono" }, fh = ["disabled"], dh = ["disabled"], ph = ["aria-valuenow", "aria-valuemax"], hh = { class: "info-grid frame-info" }, vh = { class: "mono" }, gh = { key: 2, class: "panel" }, mh = { key: 0, class: "seat-grid" }, yh = { key: 1, class: "muted" }, _h = { key: 3, class: "panel privacy-notice", role: "note" }, bh = nt({ __name: "ReplayView", setup(e) {
  const t = new qo(), n = X(null), s = X(null), i = X(null), r = X(null), o = X(false), l = X("full");
  async function a(m) {
    o.value = true, r.value = null;
    try {
      const b = JSON.parse(m);
      await t.connect(), s.value = await t.loadReplay(m), n.value = b, i.value = await t.seekReplay(0, l.value);
    } catch (b) {
      n.value = null, s.value = null, i.value = null, r.value = b instanceof Error ? b.message : String(b);
    } finally {
      o.value = false;
    }
  }
  function p(m) {
    const b = new FileReader();
    b.onload = () => void a(b.result), b.onerror = () => {
      r.value = "\u6587\u4EF6\u8BFB\u53D6\u5931\u8D25";
    }, b.readAsText(m);
  }
  function u(m) {
    var _a2;
    const b = (_a2 = m.target.files) == null ? void 0 : _a2[0];
    b && p(b);
  }
  function c(m) {
    var _a2, _b;
    m.preventDefault();
    const b = (_b = (_a2 = m.dataTransfer) == null ? void 0 : _a2.files) == null ? void 0 : _b[0];
    b && p(b);
  }
  async function v(m) {
    if (!(!s.value || o.value)) {
      o.value = true, r.value = null;
      try {
        i.value = await t.seekReplay(Math.max(0, Math.min(m, s.value.total)), l.value);
      } catch (b) {
        r.value = b instanceof Error ? b.message : String(b);
      } finally {
        o.value = false;
      }
    }
  }
  return kt(l, () => {
    i.value && v(i.value.step);
  }), fs(() => t.dispose()), (m, b) => {
    var _a2;
    return k(), C("div", Xp, [b[22] || (b[22] = f("section", { class: "panel" }, [f("h1", null, "\u5F55\u50CF\u56DE\u653E"), f("p", { class: "muted" }, "\u5BFC\u5165 Command Log \u540E\uFF0C\u89C4\u5219\u5F15\u64CE\u4F1A\u771F\u5B9E\u91CD\u6F14\u5E76\u9010\u6B65\u6838\u5BF9\u72B6\u6001\u54C8\u5E0C\u3002")], -1)), f("section", { class: "panel drop-zone", onDrop: c, onDragover: b[0] || (b[0] = qs(() => {
    }, ["prevent"])), "aria-label": "\u5BFC\u5165\u5F55\u50CF\u6587\u4EF6" }, [b[6] || (b[6] = f("h2", null, "\u5BFC\u5165\u5F55\u50CF", -1)), b[7] || (b[7] = f("p", { class: "muted" }, "\u62D6\u62FD .json \u5F55\u50CF\u6587\u4EF6\u5230\u6B64\u5904\uFF0C\u6216\u70B9\u51FB\u9009\u62E9", -1)), f("input", { type: "file", accept: ".json,application/json", onChange: u, "aria-label": "\u9009\u62E9\u5F55\u50CF\u6587\u4EF6" }, null, 32), o.value ? (k(), C("p", Zp, "\u6B63\u5728\u9A8C\u8BC1\u5E76\u91CD\u6F14\u2026")) : J("", true), r.value ? (k(), C("p", eh, S(r.value), 1)) : J("", true)], 32), n.value && s.value ? (k(), C("section", th, [b[15] || (b[15] = f("h2", null, "\u5F55\u50CF\u4FE1\u606F", -1)), f("dl", nh, [b[9] || (b[9] = f("dt", null, "Manifest Hash", -1)), f("dd", sh, S(s.value.manifest_hash), 1), b[10] || (b[10] = f("dt", null, "\u5F15\u64CE\u6784\u5EFA", -1)), f("dd", ih, S(((_a2 = n.value.header) == null ? void 0 : _a2.engine_build_id) ?? "\u2014"), 1), b[11] || (b[11] = f("dt", null, "\u603B\u6B65\u6570", -1)), f("dd", null, S(s.value.total), 1), b[12] || (b[12] = f("dt", null, "\u5B8C\u6574\u6027", -1)), b[13] || (b[13] = f("dd", { class: "verified" }, "\u2713 \u54C8\u5E0C\u94FE\u4E0E\u5168\u90E8\u72B6\u6001\u68C0\u67E5\u70B9\u5DF2\u9A8C\u8BC1", -1)), b[14] || (b[14] = f("dt", null, "\u9690\u79C1\u7B49\u7EA7", -1)), f("dd", null, [Ae(f("select", { "onUpdate:modelValue": b[1] || (b[1] = (_) => l.value = _), disabled: o.value, "aria-label": "\u9690\u79C1\u7B49\u7EA7" }, [...b[8] || (b[8] = [f("option", { value: "full" }, "\u5B8C\u6574\uFF08\u88C1\u5224\u89C6\u89D2\uFF09", -1), f("option", { value: "player" }, "\u73A9\u5BB6 1 \u89C6\u89D2", -1), f("option", { value: "public" }, "\u516C\u5F00\u89C6\u89D2", -1)])], 8, rh), [[Kt, l.value]])])])])) : J("", true), i.value && s.value ? (k(), C("section", oh, [b[19] || (b[19] = f("h2", null, "\u56DE\u653E\u63A7\u5236", -1)), f("div", lh, [f("button", { type: "button", onClick: b[2] || (b[2] = (_) => v(0)), disabled: o.value || i.value.step === 0 }, "\u23EE \u5F00\u5934", 8, ah), f("button", { type: "button", onClick: b[3] || (b[3] = (_) => v(i.value.step - 1)), disabled: o.value || i.value.step === 0 }, "\u25C0 \u540E\u9000", 8, ch), f("span", uh, S(i.value.step) + " / " + S(i.value.total), 1), f("button", { type: "button", onClick: b[4] || (b[4] = (_) => v(i.value.step + 1)), disabled: o.value || i.value.step >= i.value.total }, "\u524D\u8FDB \u25B6", 8, fh), f("button", { type: "button", onClick: b[5] || (b[5] = (_) => v(i.value.total)), disabled: o.value || i.value.step >= i.value.total }, "\u7ED3\u5C3E \u23ED", 8, dh)]), f("div", { class: "progress-bar", role: "progressbar", "aria-valuenow": i.value.step, "aria-valuemax": i.value.total }, [f("div", { class: "progress-fill", style: Tt({ width: i.value.step / Math.max(i.value.total, 1) * 100 + "%" }) }, null, 4)], 8, ph), f("dl", hh, [b[16] || (b[16] = f("dt", null, "Revision", -1)), f("dd", null, S(i.value.revision), 1), b[17] || (b[17] = f("dt", null, "State Hash", -1)), f("dd", vh, S(i.value.hash_hex), 1), b[18] || (b[18] = f("dt", null, "\u6BD4\u8D5B\u72B6\u6001", -1)), f("dd", null, S(i.value.finished ? "\u5DF2\u7ED3\u675F" : "\u8FDB\u884C\u4E2D"), 1)])])) : J("", true), i.value ? (k(), C("section", gh, [b[20] || (b[20] = f("h2", null, "\u5F53\u524D\u5C40\u9762", -1)), i.value.view.seats.length ? (k(), C("div", mh, [(k(true), C(ce, null, Ce(i.value.view.seats, (_) => (k(), C("article", { key: _.player, class: "seat" }, [f("strong", null, "\u5EA7\u4F4D " + S(_.seat) + " \xB7 \u73A9\u5BB6 " + S(_.player), 1), f("span", null, "\u4F53\u529B " + S(_.hp) + " / " + S(_.max_hp), 1), f("span", null, "\u624B\u724C " + S(_.hand_count) + " \xB7 " + S(_.alive ? "\u5B58\u6D3B" : "\u9635\u4EA1") + S(_.turned_over ? " \xB7 \u7FFB\u9762" : ""), 1), f("span", null, "\u8EAB\u4EFD " + S(_.identity ?? "\u672A\u77E5"), 1)]))), 128))])) : (k(), C("p", yh, "\u6BD4\u8D5B\u5C1A\u672A\u5F00\u59CB\u3002"))])) : J("", true), n.value ? (k(), C("section", _h, [...b[21] || (b[21] = [f("h2", null, "\u9690\u79C1\u5206\u7EA7\uFF08\xA722.5\uFF09", -1), f("p", { class: "muted tiny" }, "\u6BCF\u4E2A\u753B\u9762\u76F4\u63A5\u4ECE\u5BF9\u5E94\u7684\u88C1\u5224\u3001\u73A9\u5BB6\u6216\u516C\u5F00 Observer \u6295\u5F71\u6784\u5EFA\uFF0C\u4E0D\u4F1A\u5148\u751F\u6210\u5B8C\u6574\u89C6\u56FE\u518D\u5220\u5B57\u6BB5\u3002", -1)])])) : J("", true)]);
  };
} }), wh = Lt(bh, [["__scopeId", "data-v-91fcaceb"]]), Sh = { class: "stack" }, kh = { class: "panel" }, Eh = { key: 0, class: "counts" }, Ch = { key: 1, class: "muted", role: "status" }, xh = { key: 2, class: "error", role: "alert" }, Ah = { key: 0, class: "browser" }, Rh = { class: "panel roster" }, Ph = { class: "filters" }, Oh = ["value"], $h = ["onClick"], Ih = ["src"], Th = { key: 0, class: "muted limit" }, Nh = { key: 0, class: "panel detail" }, Mh = ["src", "alt"], Dh = ["title"], Lh = { class: "muted" }, jh = { key: 0, class: "skills" }, Uh = ["title"], Vh = { key: 1, class: "muted" }, Hh = { key: 2, class: "skins" }, Fh = ["src", "alt"], Wh = { key: 3, class: "muted" }, Bh = { class: "muted" }, qh = nt({ __name: "CollectionView", setup(e) {
  const t = X(null), n = X(""), s = X(false), i = X(""), r = X(""), o = X(null), l = _(), a = se(() => {
    var _a2;
    return new Map((_a2 = t.value) == null ? void 0 : _a2.skills.map((P) => [P.id, P]));
  }), p = se(() => {
    var _a2;
    const P = i.value.trim().toLowerCase();
    return (((_a2 = t.value) == null ? void 0 : _a2.characters) ?? []).filter((O) => !r.value || O.pack === r.value).filter((O) => !P || O.name.toLowerCase().includes(P) || O.display_name.toLowerCase().includes(P)).slice(0, 80);
  }), u = se(() => {
    var _a2;
    return ((_a2 = o.value) == null ? void 0 : _a2.resolved_skills.map((P) => a.value.get(P)).filter(Boolean)) ?? [];
  }), c = se(() => {
    var _a2;
    return (((_a2 = t.value) == null ? void 0 : _a2.skins.static) ?? []).filter((P) => {
      var _a3;
      return P.character === ((_a3 = o.value) == null ? void 0 : _a3.name);
    });
  }), v = se(() => {
    var _a2;
    return (((_a2 = t.value) == null ? void 0 : _a2.skins.dynamic) ?? []).filter((P) => {
      var _a3;
      return P.character === ((_a3 = o.value) == null ? void 0 : _a3.name);
    });
  });
  Ln(m);
  async function m() {
    s.value = true, n.value = "";
    try {
      const P = await fetch(`${l}/catalog.json`);
      if (!P.ok) throw new Error(`${P.status} ${P.statusText}`);
      t.value = await P.json(), o.value = t.value.characters[0] ?? null;
    } catch (P) {
      n.value = `\u65E0\u6CD5\u8BFB\u53D6\u539F\u4ED3\u5E93\u5185\u5BB9\u76EE\u5F55\uFF08${String(P)}\uFF09\u3002\u8BF7\u5148\u542F\u52A8 game-server\u3002`;
    } finally {
      s.value = false;
    }
  }
  function b(P) {
    return P ? encodeURI(`${l}/${P}`) : "";
  }
  function _() {
    const O = !window.location.hostname || window.location.hostname.endsWith("tauri.localhost") ? "127.0.0.1" : window.location.hostname;
    return `${window.location.protocol === "https:" ? "https:" : "http:"}//${O}:7700/original-content`;
  }
  return (P, O) => (k(), C("div", Sh, [f("section", kh, [O[2] || (O[2] = f("h1", null, "\u539F\u4ED3\u5E93\u6B66\u5C06\u5E93", -1)), O[3] || (O[3] = f("p", { class: "muted" }, " \u6B66\u5C06\u3001\u6280\u80FD\u6E90\u7801\u3001\u9759\u6001\u76AE\u80A4\u548C\u52A8\u6001\u76AE\u80A4\u5747\u4ECE\u672C\u5730\u539F\u4ED3\u5E93\u76EE\u5F55\u8BFB\u53D6\uFF1B\u65E7 JavaScript \u4E0D\u4F1A\u5728\u5BA2\u6237\u7AEF\u6267\u884C\u3002 ", -1)), t.value ? (k(), C("p", Eh, S(t.value.counts.packs) + " \u5305 \xB7 " + S(t.value.counts.characters) + " \u6B66\u5C06 \xB7 " + S(t.value.counts.skills) + " \u6280\u80FD \xB7 " + S(t.value.counts.static_skins) + " \u9759\u6001\u76AE\u80A4 \xB7 " + S(t.value.counts.dynamic_skins) + " \u52A8\u6001\u76AE\u80A4 ", 1)) : J("", true), s.value ? (k(), C("p", Ch, "\u6B63\u5728\u52A0\u8F7D\u76EE\u5F55\u2026")) : J("", true), n.value ? (k(), C("p", xh, S(n.value), 1)) : J("", true)]), t.value ? (k(), C("section", Ah, [f("aside", Rh, [f("div", Ph, [Ae(f("input", { "onUpdate:modelValue": O[0] || (O[0] = (w) => i.value = w), type: "search", placeholder: "\u641C\u7D22\u6B66\u5C06", "aria-label": "\u641C\u7D22\u6B66\u5C06" }, null, 512), [[Qt, i.value]]), Ae(f("select", { "onUpdate:modelValue": O[1] || (O[1] = (w) => r.value = w), "aria-label": "\u6B66\u5C06\u5305" }, [O[4] || (O[4] = f("option", { value: "" }, "\u5168\u90E8\u6B66\u5C06\u5305", -1)), (k(true), C(ce, null, Ce(t.value.packages, (w) => (k(), C("option", { key: w, value: w }, S(w), 9, Oh))), 128))], 512), [[Kt, r.value]])]), (k(true), C(ce, null, Ce(p.value, (w) => {
    var _a2;
    return k(), C("button", { key: w.id, type: "button", class: $e(["character-row", { selected: ((_a2 = o.value) == null ? void 0 : _a2.id) === w.id }]), onClick: (I) => o.value = w }, [w.portrait ? (k(), C("img", { key: 0, src: b(w.portrait), alt: "", loading: "lazy" }, null, 8, Ih)) : J("", true), f("span", null, [f("strong", null, S(w.display_name), 1), f("small", null, S(w.pack) + " \xB7 " + S(w.name), 1)])], 10, $h);
  }), 128)), p.value.length === 80 ? (k(), C("p", Th, "\u4EC5\u663E\u793A\u524D 80 \u9879\uFF0C\u8BF7\u7EE7\u7EED\u7B5B\u9009\u3002")) : J("", true)]), o.value ? (k(), C("article", Nh, [f("header", null, [o.value.portrait ? (k(), C("img", { key: 0, src: b(o.value.portrait), alt: `${o.value.display_name}\u6B66\u5C06\u5934\u50CF` }, null, 8, Mh)) : J("", true), f("div", null, [f("h2", null, S(o.value.display_name), 1), f("span", { class: $e(["character-status", o.value.migration_status]), title: o.value.migration_status === "ported" ? "\u5B8C\u6574\u6B66\u5C06\u5B9A\u4E49\u5DF2\u63A5\u5165\u786E\u5B9A\u6027\u5185\u5BB9\u5305" : "\u4EC5\u540C\u6B65\u76EE\u5F55\u3001\u6E90\u7801\u4E0E\u8D44\u6E90\uFF0C\u5C1A\u4E0D\u80FD\u8FDB\u5165\u786E\u5B9A\u6027\u5BF9\u5C40" }, S(o.value.migration_status === "ported" ? "\u6B66\u5C06\u5305\u5DF2\u63A5\u5165" : "\u5F85\u5B8C\u6574\u79FB\u690D"), 11, Dh), f("p", Lh, S(o.value.pack) + " \xB7 " + S(o.value.name) + " \xB7 " + S(o.value.group ?? "\u672A\u77E5\u52BF\u529B") + " \xB7 " + S(o.value.hp) + " \u4F53\u529B", 1)])]), O[5] || (O[5] = f("h3", null, "\u6280\u80FD", -1)), u.value.length ? (k(), C("ul", jh, [(k(true), C(ce, null, Ce(u.value, (w) => (k(), C("li", { key: w.id }, [f("strong", null, S(w.display_name), 1), f("span", { class: $e(w.migration_status), title: w.migration_status === "ported" ? "\u5DF2\u63A5\u5165\u786E\u5B9A\u6027\u5185\u5BB9\u5305\uFF1B\u662F\u5426\u4E0E\u539F\u6280\u80FD\u5B8C\u5168\u7B49\u4EF7\u4ECD\u4EE5 Golden \u4E3A\u51C6" : "\u4EC5\u4FDD\u7559\u539F\u6E90\u7801\u4E0E\u76EE\u5F55\uFF0C\u4E0D\u80FD\u8FDB\u5165\u786E\u5B9A\u6027\u5BF9\u5C40" }, S(w.migration_status === "ported" ? "VM \u5DF2\u63A5\u5165" : "\u5F85 DSL \u91CD\u5199"), 11, Uh), f("p", null, S(w.description ?? w.name), 1)]))), 128))])) : (k(), C("p", Vh, "\u6CA1\u6709\u53EF\u89E3\u6790\u7684\u6B66\u5C06\u5305\u6280\u80FD\u3002")), f("h3", null, "\u9759\u6001\u76AE\u80A4\uFF08" + S(c.value.length) + "\uFF09", 1), c.value.length ? (k(), C("div", Hh, [(k(true), C(ce, null, Ce(c.value, (w) => (k(), C("figure", { key: w.id }, [f("img", { src: b(w.asset), alt: `${o.value.display_name} \xB7 ${w.name}`, loading: "lazy" }, null, 8, Fh), f("figcaption", null, S(w.name), 1)]))), 128))])) : (k(), C("p", Wh, "\u539F\u4ED3\u5E93\u672A\u63D0\u4F9B\u9759\u6001\u76AE\u80A4\u3002")), f("h3", null, "\u52A8\u6001\u76AE\u80A4\uFF08" + S(v.value.length) + "\uFF09", 1), f("p", Bh, S(v.value.map((w) => w.name).join("\u3001") || "\u539F\u4ED3\u5E93\u672A\u63D0\u4F9B\u52A8\u6001\u76AE\u80A4\u3002"), 1)])) : J("", true)])) : J("", true)]));
} }), Kh = Lt(qh, [["__scopeId", "data-v-bc3fbd15"]]), Gh = Cu({ history: Dt() ? ru() : Vo(), routes: [{ path: "/", name: "home", component: zu }, { path: "/battle", name: "battle", component: Hp }, { path: "/settings", name: "settings", component: Yp }, { path: "/replay", name: "replay", component: wh }, { path: "/collection", name: "collection", component: Kh }], scrollBehavior: () => ({ top: 0 }) });
_c($u).use(Gh).mount("#app");
