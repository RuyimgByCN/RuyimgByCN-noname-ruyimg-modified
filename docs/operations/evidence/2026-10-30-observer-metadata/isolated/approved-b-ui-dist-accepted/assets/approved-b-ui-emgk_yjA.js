var __defProp = Object.defineProperty;
var __defNormalProp = (obj, key, value) => key in obj ? __defProp(obj, key, { enumerable: true, configurable: true, writable: true, value }) : obj[key] = value;
var __publicField = (obj, key, value) => __defNormalProp(obj, typeof key !== "symbol" ? key + "" : key, value);
(async () => {
  function Cr(e) {
    const t = /* @__PURE__ */ Object.create(null);
    for (const n of e.split(",")) t[n] = 1;
    return (n) => n in t;
  }
  const fe = {}, Xt = [], ht = () => {
  }, mo = () => false, Ss = (e) => e.charCodeAt(0) === 111 && e.charCodeAt(1) === 110 && (e.charCodeAt(2) > 122 || e.charCodeAt(2) < 97), ks = (e) => e.startsWith("onUpdate:"), Ee = Object.assign, Rr = (e, t) => {
    const n = e.indexOf(t);
    n > -1 && e.splice(n, 1);
  }, nl = Object.prototype.hasOwnProperty, ae = (e, t) => nl.call(e, t), G = Array.isArray, Zt = (e) => Kn(e) === "[object Map]", fn = (e) => Kn(e) === "[object Set]", ti = (e) => Kn(e) === "[object Date]", Y = (e) => typeof e == "function", ve = (e) => typeof e == "string", gt = (e) => typeof e == "symbol", le = (e) => e !== null && typeof e == "object", vo = (e) => (le(e) || Y(e)) && Y(e.then) && Y(e.catch), yo = Object.prototype.toString, Kn = (e) => yo.call(e), sl = (e) => Kn(e).slice(8, -1), _o = (e) => Kn(e) === "[object Object]", Pr = (e) => ve(e) && e !== "NaN" && e[0] !== "-" && "" + parseInt(e, 10) === e, In = Cr(",key,ref,ref_for,ref_key,onVnodeBeforeMount,onVnodeMounted,onVnodeBeforeUpdate,onVnodeUpdated,onVnodeBeforeUnmount,onVnodeUnmounted"), Es = (e) => {
    const t = /* @__PURE__ */ Object.create(null);
    return ((n) => t[n] || (t[n] = e(n)));
  }, rl = /-\w/g, Be = Es((e) => e.replace(rl, (t) => t.slice(1).toUpperCase())), il = /\B([A-Z])/g, jt = Es((e) => e.replace(il, "-$1").toLowerCase()), Tr = Es((e) => e.charAt(0).toUpperCase() + e.slice(1)), Ds = Es((e) => e ? `on${Tr(e)}` : ""), dt = (e, t) => !Object.is(e, t), ss = (e, ...t) => {
    for (let n = 0; n < e.length; n++) e[n](...t);
  }, bo = (e, t, n, s = false) => {
    Object.defineProperty(e, t, {
      configurable: true,
      enumerable: false,
      writable: s,
      value: n
    });
  }, wo = (e) => {
    const t = parseFloat(e);
    return isNaN(t) ? e : t;
  };
  let ni;
  const As = () => ni || (ni = typeof globalThis < "u" ? globalThis : typeof self < "u" ? self : typeof window < "u" ? window : typeof global < "u" ? global : {});
  function Kt(e) {
    if (G(e)) {
      const t = {};
      for (let n = 0; n < e.length; n++) {
        const s = e[n], r = ve(s) ? cl(s) : Kt(s);
        if (r) for (const i in r) t[i] = r[i];
      }
      return t;
    } else if (ve(e) || le(e)) return e;
  }
  const ol = /;(?![^(]*\))/g, al = /:([^]+)/, ll = /\/\*[^]*?\*\//g;
  function cl(e) {
    const t = {};
    return e.replace(ll, "").split(ol).forEach((n) => {
      if (n) {
        const s = n.split(al);
        s.length > 1 && (t[s[0].trim()] = s[1].trim());
      }
    }), t;
  }
  function Fe(e) {
    let t = "";
    if (ve(e)) t = e;
    else if (G(e)) for (let n = 0; n < e.length; n++) {
      const s = Fe(e[n]);
      s && (t += s + " ");
    }
    else if (le(e)) for (const n in e) e[n] && (t += n + " ");
    return t.trim();
  }
  const ul = "itemscope,allowfullscreen,formnovalidate,ismap,nomodule,novalidate,readonly", fl = Cr(ul);
  function So(e) {
    return !!e || e === "";
  }
  function dl(e, t) {
    if (e.length !== t.length) return false;
    let n = true;
    for (let s = 0; n && s < e.length; s++) n = dn(e[s], t[s]);
    return n;
  }
  function dn(e, t) {
    if (e === t) return true;
    let n = ti(e), s = ti(t);
    if (n || s) return n && s ? e.getTime() === t.getTime() : false;
    if (n = gt(e), s = gt(t), n || s) return e === t;
    if (n = G(e), s = G(t), n || s) return n && s ? dl(e, t) : false;
    if (n = le(e), s = le(t), n || s) {
      if (!n || !s) return false;
      const r = Object.keys(e).length, i = Object.keys(t).length;
      if (r !== i) return false;
      for (const a in e) {
        const o = e.hasOwnProperty(a), l = t.hasOwnProperty(a);
        if (o && !l || !o && l || !dn(e[a], t[a])) return false;
      }
    }
    return String(e) === String(t);
  }
  function Ir(e, t) {
    return e.findIndex((n) => dn(n, t));
  }
  const ko = (e) => !!(e && e.__v_isRef === true), P = (e) => ve(e) ? e : e == null ? "" : G(e) || le(e) && (e.toString === yo || !Y(e.toString)) ? ko(e) ? P(e.value) : JSON.stringify(e, Eo, 2) : String(e), Eo = (e, t) => ko(t) ? Eo(e, t.value) : Zt(t) ? {
    [`Map(${t.size})`]: [
      ...t.entries()
    ].reduce((n, [s, r], i) => (n[Ls(s, i) + " =>"] = r, n), {})
  } : fn(t) ? {
    [`Set(${t.size})`]: [
      ...t.values()
    ].map((n) => Ls(n))
  } : gt(t) ? Ls(t) : le(t) && !G(t) && !_o(t) ? String(t) : t, Ls = (e, t = "") => {
    var n;
    return gt(e) ? `Symbol(${(n = e.description) != null ? n : t})` : e;
  };
  let Se;
  class hl {
    constructor(t = false) {
      this.detached = t, this._active = true, this._on = 0, this.effects = [], this.cleanups = [], this._isPaused = false, this._warnOnRun = true, this.__v_skip = true, !t && Se && (Se.active ? (this.parent = Se, this.index = (Se.scopes || (Se.scopes = [])).push(this) - 1) : (this._active = false, this._warnOnRun = false));
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
        const n = Se;
        try {
          return Se = this, t();
        } finally {
          Se = n;
        }
      }
    }
    on() {
      ++this._on === 1 && (this.prevScope = Se, Se = this);
    }
    off() {
      if (this._on > 0 && --this._on === 0) {
        if (Se === this) Se = this.prevScope;
        else {
          let t = Se;
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
  function pl() {
    return Se;
  }
  let he;
  const Us = /* @__PURE__ */ new WeakSet();
  class Ao {
    constructor(t) {
      this.fn = t, this.deps = void 0, this.depsTail = void 0, this.flags = 5, this.next = void 0, this.cleanup = void 0, this.scheduler = void 0, Se && (Se.active ? Se.effects.push(this) : this.flags &= -2);
    }
    pause() {
      this.flags |= 64;
    }
    resume() {
      this.flags & 64 && (this.flags &= -65, Us.has(this) && (Us.delete(this), this.trigger()));
    }
    notify() {
      this.flags & 2 && !(this.flags & 32) || this.flags & 8 || Co(this);
    }
    run() {
      if (!(this.flags & 1)) return this.fn();
      this.flags |= 2, si(this), Ro(this);
      const t = he, n = Xe;
      he = this, Xe = true;
      try {
        return this.fn();
      } finally {
        Po(this), he = t, Xe = n, this.flags &= -3;
      }
    }
    stop() {
      if (this.flags & 1) {
        for (let t = this.deps; t; t = t.nextDep) $r(t);
        this.deps = this.depsTail = void 0, si(this), this.onStop && this.onStop(), this.flags &= -2;
      }
    }
    trigger() {
      this.flags & 64 ? Us.add(this) : this.scheduler ? this.scheduler() : this.runIfDirty();
    }
    runIfDirty() {
      nr(this) && this.run();
    }
    get dirty() {
      return nr(this);
    }
  }
  let xo = 0, On, Nn;
  function Co(e, t = false) {
    if (e.flags |= 8, t) {
      e.next = Nn, Nn = e;
      return;
    }
    e.next = On, On = e;
  }
  function Or() {
    xo++;
  }
  function Nr() {
    if (--xo > 0) return;
    if (Nn) {
      let t = Nn;
      for (Nn = void 0; t; ) {
        const n = t.next;
        t.next = void 0, t.flags &= -9, t = n;
      }
    }
    let e;
    for (; On; ) {
      let t = On;
      for (On = void 0; t; ) {
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
  function Ro(e) {
    for (let t = e.deps; t; t = t.nextDep) t.version = -1, t.prevActiveLink = t.dep.activeLink, t.dep.activeLink = t;
  }
  function Po(e) {
    let t, n = e.depsTail, s = n;
    for (; s; ) {
      const r = s.prevDep;
      s.version === -1 ? (s === n && (n = r), $r(s), gl(s)) : t = s, s.dep.activeLink = s.prevActiveLink, s.prevActiveLink = void 0, s = r;
    }
    e.deps = t, e.depsTail = n;
  }
  function nr(e) {
    for (let t = e.deps; t; t = t.nextDep) if (t.dep.version !== t.version || t.dep.computed && (To(t.dep.computed) || t.dep.version !== t.version)) return true;
    return !!e._dirty;
  }
  function To(e) {
    if (e.flags & 4 && !(e.flags & 16) || (e.flags &= -17, e.globalVersion === Un) || (e.globalVersion = Un, !e.isSSR && e.flags & 128 && (!e.deps && !e._dirty || !nr(e)))) return;
    e.flags |= 2;
    const t = e.dep, n = he, s = Xe;
    he = e, Xe = true;
    try {
      Ro(e);
      const r = e.fn(e._value);
      (t.version === 0 || dt(r, e._value)) && (e.flags |= 128, e._value = r, t.version++);
    } catch (r) {
      throw t.version++, r;
    } finally {
      he = n, Xe = s, Po(e), e.flags &= -3;
    }
  }
  function $r(e, t = false) {
    const { dep: n, prevSub: s, nextSub: r } = e;
    if (s && (s.nextSub = r, e.prevSub = void 0), r && (r.prevSub = s, e.nextSub = void 0), n.subs === e && (n.subs = s, !s && n.computed)) {
      n.computed.flags &= -5;
      for (let i = n.computed.deps; i; i = i.nextDep) $r(i, true);
    }
    !t && !--n.sc && n.map && n.map.delete(n.key);
  }
  function gl(e) {
    const { prevDep: t, nextDep: n } = e;
    t && (t.nextDep = n, e.prevDep = void 0), n && (n.prevDep = t, e.nextDep = void 0);
  }
  let Xe = true;
  const Io = [];
  function mt() {
    Io.push(Xe), Xe = false;
  }
  function vt() {
    const e = Io.pop();
    Xe = e === void 0 ? true : e;
  }
  function si(e) {
    const { cleanup: t } = e;
    if (e.cleanup = void 0, t) {
      const n = he;
      he = void 0;
      try {
        t();
      } finally {
        he = n;
      }
    }
  }
  let Un = 0;
  class ml {
    constructor(t, n) {
      this.sub = t, this.dep = n, this.version = n.version, this.nextDep = this.prevDep = this.nextSub = this.prevSub = this.prevActiveLink = void 0;
    }
  }
  class Mr {
    constructor(t) {
      this.computed = t, this.version = 0, this.activeLink = void 0, this.subs = void 0, this.map = void 0, this.key = void 0, this.sc = 0, this.__v_skip = true;
    }
    track(t) {
      if (!he || !Xe || he === this.computed) return;
      let n = this.activeLink;
      if (n === void 0 || n.sub !== he) n = this.activeLink = new ml(he, this), he.deps ? (n.prevDep = he.depsTail, he.depsTail.nextDep = n, he.depsTail = n) : he.deps = he.depsTail = n, Oo(n);
      else if (n.version === -1 && (n.version = this.version, n.nextDep)) {
        const s = n.nextDep;
        s.prevDep = n.prevDep, n.prevDep && (n.prevDep.nextDep = s), n.prevDep = he.depsTail, n.nextDep = void 0, he.depsTail.nextDep = n, he.depsTail = n, he.deps === n && (he.deps = s);
      }
      return n;
    }
    trigger(t) {
      this.version++, Un++, this.notify(t);
    }
    notify(t) {
      Or();
      try {
        for (let n = this.subs; n; n = n.prevSub) n.sub.notify() && n.sub.dep.notify();
      } finally {
        Nr();
      }
    }
  }
  function Oo(e) {
    if (e.dep.sc++, e.sub.flags & 4) {
      const t = e.dep.computed;
      if (t && !e.dep.subs) {
        t.flags |= 20;
        for (let s = t.deps; s; s = s.nextDep) Oo(s);
      }
      const n = e.dep.subs;
      n !== e && (e.prevSub = n, n && (n.nextSub = e)), e.dep.subs = e;
    }
  }
  const sr = /* @__PURE__ */ new WeakMap(), Wt = Symbol(""), rr = Symbol(""), Hn = Symbol("");
  function Ae(e, t, n) {
    if (Xe && he) {
      let s = sr.get(e);
      s || sr.set(e, s = /* @__PURE__ */ new Map());
      let r = s.get(n);
      r || (s.set(n, r = new Mr()), r.map = s, r.key = n), r.track();
    }
  }
  function Et(e, t, n, s, r, i) {
    const a = sr.get(e);
    if (!a) {
      Un++;
      return;
    }
    const o = (l) => {
      l && l.trigger();
    };
    if (Or(), t === "clear") a.forEach(o);
    else {
      const l = G(e), c = l && Pr(n);
      if (l && n === "length") {
        const u = Number(s);
        a.forEach((f, m) => {
          (m === "length" || m === Hn || !gt(m) && m >= u) && o(f);
        });
      } else switch ((n !== void 0 || a.has(void 0)) && o(a.get(n)), c && o(a.get(Hn)), t) {
        case "add":
          l ? c && o(a.get("length")) : (o(a.get(Wt)), Zt(e) && o(a.get(rr)));
          break;
        case "delete":
          l || (o(a.get(Wt)), Zt(e) && o(a.get(rr)));
          break;
        case "set":
          Zt(e) && o(a.get(Wt));
          break;
      }
    }
    Nr();
  }
  function Gt(e) {
    const t = oe(e);
    return t === e ? t : (Ae(t, "iterate", Hn), ze(e) ? t : t.map(et));
  }
  function xs(e) {
    return Ae(e = oe(e), "iterate", Hn), e;
  }
  function ut(e, t) {
    return Ct(e) ? rn(Vt(e) ? et(t) : t) : et(t);
  }
  const vl = {
    __proto__: null,
    [Symbol.iterator]() {
      return Hs(this, Symbol.iterator, (e) => ut(this, e));
    },
    concat(...e) {
      return Gt(this).concat(...e.map((t) => G(t) ? Gt(t) : t));
    },
    entries() {
      return Hs(this, "entries", (e) => (e[1] = ut(this, e[1]), e));
    },
    every(e, t) {
      return _t(this, "every", e, t, void 0, arguments);
    },
    filter(e, t) {
      return _t(this, "filter", e, t, (n) => n.map((s) => ut(this, s)), arguments);
    },
    find(e, t) {
      return _t(this, "find", e, t, (n) => ut(this, n), arguments);
    },
    findIndex(e, t) {
      return _t(this, "findIndex", e, t, void 0, arguments);
    },
    findLast(e, t) {
      return _t(this, "findLast", e, t, (n) => ut(this, n), arguments);
    },
    findLastIndex(e, t) {
      return _t(this, "findLastIndex", e, t, void 0, arguments);
    },
    forEach(e, t) {
      return _t(this, "forEach", e, t, void 0, arguments);
    },
    includes(...e) {
      return Fs(this, "includes", e);
    },
    indexOf(...e) {
      return Fs(this, "indexOf", e);
    },
    join(e) {
      return Gt(this).join(e);
    },
    lastIndexOf(...e) {
      return Fs(this, "lastIndexOf", e);
    },
    map(e, t) {
      return _t(this, "map", e, t, void 0, arguments);
    },
    pop() {
      return vn(this, "pop");
    },
    push(...e) {
      return vn(this, "push", e);
    },
    reduce(e, ...t) {
      return ri(this, "reduce", e, t);
    },
    reduceRight(e, ...t) {
      return ri(this, "reduceRight", e, t);
    },
    shift() {
      return vn(this, "shift");
    },
    some(e, t) {
      return _t(this, "some", e, t, void 0, arguments);
    },
    splice(...e) {
      return vn(this, "splice", e);
    },
    toReversed() {
      return Gt(this).toReversed();
    },
    toSorted(e) {
      return Gt(this).toSorted(e);
    },
    toSpliced(...e) {
      return Gt(this).toSpliced(...e);
    },
    unshift(...e) {
      return vn(this, "unshift", e);
    },
    values() {
      return Hs(this, "values", (e) => ut(this, e));
    }
  };
  function Hs(e, t, n) {
    const s = xs(e), r = s[t]();
    return s !== e && !ze(e) && (r._next = r.next, r.next = () => {
      const i = r._next();
      return i.done || (i.value = n(i.value)), i;
    }), r;
  }
  const yl = Array.prototype;
  function _t(e, t, n, s, r, i) {
    const a = xs(e), o = a !== e && !ze(e), l = a[t];
    if (l !== yl[t]) {
      const f = l.apply(e, i);
      return o ? et(f) : f;
    }
    let c = n;
    a !== e && (o ? c = function(f, m) {
      return n.call(this, ut(e, f), m, e);
    } : n.length > 2 && (c = function(f, m) {
      return n.call(this, f, m, e);
    }));
    const u = l.call(a, c, s);
    return o && r ? r(u) : u;
  }
  function ri(e, t, n, s) {
    const r = xs(e), i = r !== e && !ze(e);
    let a = n, o = false;
    r !== e && (i ? (o = s.length === 0, a = function(c, u, f) {
      return o && (o = false, c = ut(e, c)), n.call(this, c, ut(e, u), f, e);
    }) : n.length > 3 && (a = function(c, u, f) {
      return n.call(this, c, u, f, e);
    }));
    const l = r[t](a, ...s);
    return o ? ut(e, l) : l;
  }
  function Fs(e, t, n) {
    const s = oe(e);
    Ae(s, "iterate", Hn);
    const r = s[t](...n);
    return (r === -1 || r === false) && Lr(n[0]) ? (n[0] = oe(n[0]), s[t](...n)) : r;
  }
  function vn(e, t, n = []) {
    mt(), Or();
    const s = oe(e)[t].apply(e, n);
    return Nr(), vt(), s;
  }
  const _l = Cr("__proto__,__v_isRef,__isVue"), No = new Set(Object.getOwnPropertyNames(Symbol).filter((e) => e !== "arguments" && e !== "caller").map((e) => Symbol[e]).filter(gt));
  function bl(e) {
    gt(e) || (e = String(e));
    const t = oe(this);
    return Ae(t, "has", e), t.hasOwnProperty(e);
  }
  class $o {
    constructor(t = false, n = false) {
      this._isReadonly = t, this._isShallow = n;
    }
    get(t, n, s) {
      if (n === "__v_skip") return t.__v_skip;
      const r = this._isReadonly, i = this._isShallow;
      if (n === "__v_isReactive") return !r;
      if (n === "__v_isReadonly") return r;
      if (n === "__v_isShallow") return i;
      if (n === "__v_raw") return s === (r ? i ? Tl : Lo : i ? Do : jo).get(t) || Object.getPrototypeOf(t) === Object.getPrototypeOf(s) ? t : void 0;
      const a = G(t);
      if (!r) {
        let l;
        if (a && (l = vl[n])) return l;
        if (n === "hasOwnProperty") return bl;
      }
      const o = Reflect.get(t, n, Re(t) ? t : s);
      if ((gt(n) ? No.has(n) : _l(n)) || (r || Ae(t, "get", n), i)) return o;
      if (Re(o)) {
        const l = a && Pr(n) ? o : o.value;
        return r && le(l) ? or(l) : l;
      }
      return le(o) ? r ? or(o) : hn(o) : o;
    }
  }
  class Mo extends $o {
    constructor(t = false) {
      super(false, t);
    }
    set(t, n, s, r) {
      let i = t[n];
      const a = G(t) && Pr(n);
      if (!this._isShallow) {
        const c = Ct(i);
        if (!ze(s) && !Ct(s) && (i = oe(i), s = oe(s)), !a && Re(i) && !Re(s)) return c || (i.value = s), true;
      }
      const o = a ? Number(n) < t.length : ae(t, n), l = Reflect.set(t, n, s, Re(t) ? t : r);
      return t === oe(r) && l && (o ? dt(s, i) && Et(t, "set", n, s) : Et(t, "add", n, s)), l;
    }
    deleteProperty(t, n) {
      const s = ae(t, n);
      t[n];
      const r = Reflect.deleteProperty(t, n);
      return r && s && Et(t, "delete", n, void 0), r;
    }
    has(t, n) {
      const s = Reflect.has(t, n);
      return (!gt(n) || !No.has(n)) && Ae(t, "has", n), s;
    }
    ownKeys(t) {
      return Ae(t, "iterate", G(t) ? "length" : Wt), Reflect.ownKeys(t);
    }
  }
  class wl extends $o {
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
  const Sl = new Mo(), kl = new wl(), El = new Mo(true);
  const ir = (e) => e, Qn = (e) => Reflect.getPrototypeOf(e);
  function Al(e, t, n) {
    return function(...s) {
      const r = this.__v_raw, i = oe(r), a = Zt(i), o = e === "entries" || e === Symbol.iterator && a, l = e === "keys" && a, c = r[e](...s), u = n ? ir : t ? rn : et;
      return !t && Ae(i, "iterate", l ? rr : Wt), Ee(Object.create(c), {
        next() {
          const { value: f, done: m } = c.next();
          return m ? {
            value: f,
            done: m
          } : {
            value: o ? [
              u(f[0]),
              u(f[1])
            ] : u(f),
            done: m
          };
        }
      });
    };
  }
  function Xn(e) {
    return function(...t) {
      return e === "delete" ? false : e === "clear" ? void 0 : this;
    };
  }
  function xl(e, t) {
    const n = {
      get(r) {
        const i = this.__v_raw, a = oe(i), o = oe(r);
        e || (dt(r, o) && Ae(a, "get", r), Ae(a, "get", o));
        const { has: l } = Qn(a), c = t ? ir : e ? rn : et;
        if (l.call(a, r)) return c(i.get(r));
        if (l.call(a, o)) return c(i.get(o));
        i !== a && i.get(r);
      },
      get size() {
        const r = this.__v_raw;
        return !e && Ae(oe(r), "iterate", Wt), r.size;
      },
      has(r) {
        const i = this.__v_raw, a = oe(i), o = oe(r);
        return e || (dt(r, o) && Ae(a, "has", r), Ae(a, "has", o)), r === o ? i.has(r) : i.has(r) || i.has(o);
      },
      forEach(r, i) {
        const a = this, o = a.__v_raw, l = oe(o), c = t ? ir : e ? rn : et;
        return !e && Ae(l, "iterate", Wt), o.forEach((u, f) => r.call(i, c(u), c(f), a));
      }
    };
    return Ee(n, e ? {
      add: Xn("add"),
      set: Xn("set"),
      delete: Xn("delete"),
      clear: Xn("clear")
    } : {
      add(r) {
        const i = oe(this), a = Qn(i), o = oe(r), l = !t && !ze(r) && !Ct(r) ? o : r;
        return a.has.call(i, l) || dt(r, l) && a.has.call(i, r) || dt(o, l) && a.has.call(i, o) || (i.add(l), Et(i, "add", l, l)), this;
      },
      set(r, i) {
        !t && !ze(i) && !Ct(i) && (i = oe(i));
        const a = oe(this), { has: o, get: l } = Qn(a);
        let c = o.call(a, r);
        c || (r = oe(r), c = o.call(a, r));
        const u = l.call(a, r);
        return a.set(r, i), c ? dt(i, u) && Et(a, "set", r, i) : Et(a, "add", r, i), this;
      },
      delete(r) {
        const i = oe(this), { has: a, get: o } = Qn(i);
        let l = a.call(i, r);
        l || (r = oe(r), l = a.call(i, r)), o && o.call(i, r);
        const c = i.delete(r);
        return l && Et(i, "delete", r, void 0), c;
      },
      clear() {
        const r = oe(this), i = r.size !== 0, a = r.clear();
        return i && Et(r, "clear", void 0, void 0), a;
      }
    }), [
      "keys",
      "values",
      "entries",
      Symbol.iterator
    ].forEach((r) => {
      n[r] = Al(r, e, t);
    }), n;
  }
  function jr(e, t) {
    const n = xl(e, t);
    return (s, r, i) => r === "__v_isReactive" ? !e : r === "__v_isReadonly" ? e : r === "__v_raw" ? s : Reflect.get(ae(n, r) && r in s ? n : s, r, i);
  }
  const Cl = {
    get: jr(false, false)
  }, Rl = {
    get: jr(false, true)
  }, Pl = {
    get: jr(true, false)
  };
  const jo = /* @__PURE__ */ new WeakMap(), Do = /* @__PURE__ */ new WeakMap(), Lo = /* @__PURE__ */ new WeakMap(), Tl = /* @__PURE__ */ new WeakMap();
  function Il(e) {
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
  function hn(e) {
    return Ct(e) ? e : Dr(e, false, Sl, Cl, jo);
  }
  function Uo(e) {
    return Dr(e, false, El, Rl, Do);
  }
  function or(e) {
    return Dr(e, true, kl, Pl, Lo);
  }
  function Dr(e, t, n, s, r) {
    if (!le(e) || e.__v_raw && !(t && e.__v_isReactive) || e.__v_skip || !Object.isExtensible(e)) return e;
    const i = r.get(e);
    if (i) return i;
    const a = Il(sl(e));
    if (a === 0) return e;
    const o = new Proxy(e, a === 2 ? s : n);
    return r.set(e, o), o;
  }
  function Vt(e) {
    return Ct(e) ? Vt(e.__v_raw) : !!(e && e.__v_isReactive);
  }
  function Ct(e) {
    return !!(e && e.__v_isReadonly);
  }
  function ze(e) {
    return !!(e && e.__v_isShallow);
  }
  function Lr(e) {
    return e ? !!e.__v_raw : false;
  }
  function oe(e) {
    const t = e && e.__v_raw;
    return t ? oe(t) : e;
  }
  function Ol(e) {
    return !ae(e, "__v_skip") && Object.isExtensible(e) && bo(e, "__v_skip", true), e;
  }
  const et = (e) => le(e) ? hn(e) : e, rn = (e) => le(e) ? or(e) : e;
  function Re(e) {
    return e ? e.__v_isRef === true : false;
  }
  function ye(e) {
    return Ho(e, false);
  }
  function Nl(e) {
    return Ho(e, true);
  }
  function Ho(e, t) {
    return Re(e) ? e : new $l(e, t);
  }
  class $l {
    constructor(t, n) {
      this.dep = new Mr(), this.__v_isRef = true, this.__v_isShallow = false, this._rawValue = n ? t : oe(t), this._value = n ? t : et(t), this.__v_isShallow = n;
    }
    get value() {
      return this.dep.track(), this._value;
    }
    set value(t) {
      const n = this._rawValue, s = this.__v_isShallow || ze(t) || Ct(t);
      t = s ? t : oe(t), dt(t, n) && (this._rawValue = t, this._value = s ? t : et(t), this.dep.trigger());
    }
  }
  function W(e) {
    return Re(e) ? e.value : e;
  }
  const Ml = {
    get: (e, t, n) => t === "__v_raw" ? e : W(Reflect.get(e, t, n)),
    set: (e, t, n, s) => {
      const r = e[t];
      return Re(r) && !Re(n) ? (r.value = n, true) : Reflect.set(e, t, n, s);
    }
  };
  function Fo(e) {
    return Vt(e) ? e : new Proxy(e, Ml);
  }
  class jl {
    constructor(t, n, s) {
      this.fn = t, this.setter = n, this._value = void 0, this.dep = new Mr(this), this.__v_isRef = true, this.deps = void 0, this.depsTail = void 0, this.flags = 16, this.globalVersion = Un - 1, this.next = void 0, this.effect = this, this.__v_isReadonly = !n, this.isSSR = s;
    }
    notify() {
      if (this.flags |= 16, !(this.flags & 8) && he !== this) return Co(this, true), true;
    }
    get value() {
      const t = this.dep.track();
      return To(this), t && (t.version = this.dep.version), this._value;
    }
    set value(t) {
      this.setter && this.setter(t);
    }
  }
  function Dl(e, t, n = false) {
    let s, r;
    return Y(e) ? s = e : (s = e.get, r = e.set), new jl(s, r, n);
  }
  const Zn = {}, ls = /* @__PURE__ */ new WeakMap();
  let Ht;
  function Ll(e, t = false, n = Ht) {
    if (n) {
      let s = ls.get(n);
      s || ls.set(n, s = []), s.push(e);
    }
  }
  function Ul(e, t, n = fe) {
    const { immediate: s, deep: r, once: i, scheduler: a, augmentJob: o, call: l } = n, c = ($) => r ? $ : ze($) || r === false || r === 0 ? At($, 1) : At($);
    let u, f, m, y, O = false, T = false;
    if (Re(e) ? (f = () => e.value, O = ze(e)) : Vt(e) ? (f = () => c(e), O = true) : G(e) ? (T = true, O = e.some(($) => Vt($) || ze($)), f = () => e.map(($) => {
      if (Re($)) return $.value;
      if (Vt($)) return c($);
      if (Y($)) return l ? l($, 2) : $();
    })) : Y(e) ? t ? f = l ? () => l(e, 2) : e : f = () => {
      if (m) {
        mt();
        try {
          m();
        } finally {
          vt();
        }
      }
      const $ = Ht;
      Ht = u;
      try {
        return l ? l(e, 3, [
          y
        ]) : e(y);
      } finally {
        Ht = $;
      }
    } : f = ht, t && r) {
      const $ = f, X = r === true ? 1 / 0 : r;
      f = () => At($(), X);
    }
    const D = pl(), L = () => {
      u.stop(), D && D.active && Rr(D.effects, u);
    };
    if (i && t) {
      const $ = t;
      t = (...X) => {
        const se = $(...X);
        return L(), se;
      };
    }
    let E = T ? new Array(e.length).fill(Zn) : Zn;
    const H = ($) => {
      if (!(!(u.flags & 1) || !u.dirty && !$)) if (t) {
        const X = u.run();
        if ($ || r || O || (T ? X.some((se, re) => dt(se, E[re])) : dt(X, E))) {
          m && m();
          const se = Ht;
          Ht = u;
          try {
            const re = [
              X,
              E === Zn ? void 0 : T && E[0] === Zn ? [] : E,
              y
            ];
            E = X, l ? l(t, 3, re) : t(...re);
          } finally {
            Ht = se;
          }
        }
      } else u.run();
    };
    return o && o(H), u = new Ao(f), u.scheduler = a ? () => a(H, false) : H, y = ($) => Ll($, false, u), m = u.onStop = () => {
      const $ = ls.get(u);
      if ($) {
        if (l) l($, 4);
        else for (const X of $) X();
        ls.delete(u);
      }
    }, t ? s ? H(true) : E = u.run() : a ? a(H.bind(null, true), true) : u.run(), L.pause = u.pause.bind(u), L.resume = u.resume.bind(u), L.stop = L, L;
  }
  function At(e, t = 1 / 0, n) {
    if (t <= 0 || !le(e) || e.__v_skip || (n = n || /* @__PURE__ */ new Map(), (n.get(e) || 0) >= t)) return e;
    if (n.set(e, t), t--, Re(e)) At(e.value, t, n);
    else if (G(e)) for (let s = 0; s < e.length; s++) At(e[s], t, n);
    else if (fn(e) || Zt(e)) e.forEach((s) => {
      At(s, t, n);
    });
    else if (_o(e)) {
      for (const s in e) At(e[s], t, n);
      for (const s of Object.getOwnPropertySymbols(e)) Object.prototype.propertyIsEnumerable.call(e, s) && At(e[s], t, n);
    }
    return e;
  }
  function Gn(e, t, n, s) {
    try {
      return s ? e(...s) : e();
    } catch (r) {
      Cs(r, t, n);
    }
  }
  function tt(e, t, n, s) {
    if (Y(e)) {
      const r = Gn(e, t, n, s);
      return r && vo(r) && r.catch((i) => {
        Cs(i, t, n);
      }), r;
    }
    if (G(e)) {
      const r = [];
      for (let i = 0; i < e.length; i++) r.push(tt(e[i], t, n, s));
      return r;
    }
  }
  function Cs(e, t, n, s = true) {
    const r = t ? t.vnode : null, { errorHandler: i, throwUnhandledErrorInProduction: a } = t && t.appContext.config || fe;
    if (t) {
      let o = t.parent;
      const l = t.proxy, c = `https://vuejs.org/error-reference/#runtime-${n}`;
      for (; o; ) {
        const u = o.ec;
        if (u) {
          for (let f = 0; f < u.length; f++) if (u[f](e, l, c) === false) return;
        }
        o = o.parent;
      }
      if (i) {
        mt(), Gn(i, null, 10, [
          e,
          l,
          c
        ]), vt();
        return;
      }
    }
    Hl(e, n, r, s, a);
  }
  function Hl(e, t, n, s = true, r = false) {
    if (r) throw e;
    console.error(e);
  }
  const Ne = [];
  let ct = -1;
  const en = [];
  let It = null, zt = 0;
  const qo = Promise.resolve();
  let cs = null;
  function pn(e) {
    const t = cs || qo;
    return e ? t.then(this ? e.bind(this) : e) : t;
  }
  function Fl(e) {
    let t = ct + 1, n = Ne.length;
    for (; t < n; ) {
      const s = t + n >>> 1, r = Ne[s], i = Fn(r);
      i < e || i === e && r.flags & 2 ? t = s + 1 : n = s;
    }
    return t;
  }
  function Ur(e) {
    if (!(e.flags & 1)) {
      const t = Fn(e), n = Ne[Ne.length - 1];
      !n || !(e.flags & 2) && t >= Fn(n) ? Ne.push(e) : Ne.splice(Fl(t), 0, e), e.flags |= 1, Wo();
    }
  }
  function Wo() {
    cs || (cs = qo.then(Bo));
  }
  function ql(e) {
    G(e) ? en.push(...e) : It && e.id === -1 ? It.splice(zt + 1, 0, e) : e.flags & 1 || (en.push(e), e.flags |= 1), Wo();
  }
  function ii(e, t, n = ct + 1) {
    for (; n < Ne.length; n++) {
      const s = Ne[n];
      if (s && s.flags & 2) {
        if (e && s.id !== e.uid) continue;
        Ne.splice(n, 1), n--, s.flags & 4 && (s.flags &= -2), s(), s.flags & 4 || (s.flags &= -2);
      }
    }
  }
  function Vo(e) {
    if (en.length) {
      const t = [
        ...new Set(en)
      ].sort((n, s) => Fn(n) - Fn(s));
      if (en.length = 0, It) {
        It.push(...t);
        return;
      }
      for (It = t, zt = 0; zt < It.length; zt++) {
        const n = It[zt];
        n.flags & 4 && (n.flags &= -2), n.flags & 8 || n(), n.flags &= -2;
      }
      It = null, zt = 0;
    }
  }
  const Fn = (e) => e.id == null ? e.flags & 2 ? -1 : 1 / 0 : e.id;
  function Bo(e) {
    try {
      for (ct = 0; ct < Ne.length; ct++) {
        const t = Ne[ct];
        t && !(t.flags & 8) && (t.flags & 4 && (t.flags &= -2), Gn(t, t.i, t.i ? 15 : 14), t.flags & 4 || (t.flags &= -2));
      }
    } finally {
      for (; ct < Ne.length; ct++) {
        const t = Ne[ct];
        t && (t.flags &= -2);
      }
      ct = -1, Ne.length = 0, Vo(), cs = null, (Ne.length || en.length) && Bo();
    }
  }
  let We = null, Ko = null;
  function us(e) {
    const t = We;
    return We = e, Ko = e && e.type.__scopeId || null, t;
  }
  function Wl(e, t = We, n) {
    if (!t || e._n) return e;
    const s = (...r) => {
      s._d && hs(-1);
      const i = us(t);
      let a;
      try {
        a = e(...r);
      } finally {
        us(i), s._d && hs(1);
      }
      return a;
    };
    return s._n = true, s._c = true, s._d = true, s;
  }
  function qs(e, t) {
    if (We === null) return e;
    const n = Is(We), s = e.dirs || (e.dirs = []);
    for (let r = 0; r < t.length; r++) {
      let [i, a, o, l = fe] = t[r];
      i && (Y(i) && (i = {
        mounted: i,
        updated: i
      }), i.deep && At(a), s.push({
        dir: i,
        instance: n,
        value: a,
        oldValue: void 0,
        arg: o,
        modifiers: l
      }));
    }
    return e;
  }
  function Lt(e, t, n, s) {
    const r = e.dirs, i = t && t.dirs;
    for (let a = 0; a < r.length; a++) {
      const o = r[a];
      i && (o.oldValue = i[a].value);
      let l = o.dir[s];
      l && (mt(), tt(l, n, 8, [
        e.el,
        o,
        e,
        t
      ]), vt());
    }
  }
  function rs(e, t) {
    if (Ce) {
      let n = Ce.provides;
      const s = Ce.parent && Ce.parent.provides;
      s === n && (n = Ce.provides = Object.create(s)), n[e] = t;
    }
  }
  function Ze(e, t, n = false) {
    const s = Vc();
    if (s || tn) {
      let r = tn ? tn._context.provides : s ? s.parent == null || s.ce ? s.vnode.appContext && s.vnode.appContext.provides : s.parent.provides : void 0;
      if (r && e in r) return r[e];
      if (arguments.length > 1) return n && Y(t) ? t.call(s && s.proxy) : t;
    }
  }
  const Vl = Symbol.for("v-scx"), Bl = () => Ze(Vl);
  function Bt(e, t, n) {
    return Go(e, t, n);
  }
  function Go(e, t, n = fe) {
    const { immediate: s, deep: r, flush: i, once: a } = n, o = Ee({}, n), l = t && s || !t && i !== "post";
    let c;
    if (Wn) {
      if (i === "sync") {
        const y = Bl();
        c = y.__watcherHandles || (y.__watcherHandles = []);
      } else if (!l) {
        const y = () => {
        };
        return y.stop = ht, y.resume = ht, y.pause = ht, y;
      }
    }
    const u = Ce;
    o.call = (y, O, T) => tt(y, u, O, T);
    let f = false;
    i === "post" ? o.scheduler = (y) => {
      He(y, u && u.suspense);
    } : i !== "sync" && (f = true, o.scheduler = (y, O) => {
      O ? y() : Ur(y);
    }), o.augmentJob = (y) => {
      t && (y.flags |= 4), f && (y.flags |= 2, u && (y.id = u.uid, y.i = u));
    };
    const m = Ul(e, t, o);
    return Wn && (c ? c.push(m) : l && m()), m;
  }
  function Kl(e, t, n) {
    const s = this.proxy, r = ve(e) ? e.includes(".") ? Jo(s, e) : () => s[e] : e.bind(s, s);
    let i;
    Y(t) ? i = t : (i = t.handler, n = t);
    const a = Jn(this), o = Go(r, i.bind(s), n);
    return a(), o;
  }
  function Jo(e, t) {
    const n = t.split(".");
    return () => {
      let s = e;
      for (let r = 0; r < n.length && s; r++) s = s[n[r]];
      return s;
    };
  }
  const Gl = Symbol("_vte"), Jl = (e) => e.__isTeleport, Ws = Symbol("_leaveCb");
  function Hr(e, t) {
    e.shapeFlag & 6 && e.component ? (e.transition = t, Hr(e.component.subTree, t)) : e.shapeFlag & 128 ? (e.ssContent.transition = t.clone(e.ssContent), e.ssFallback.transition = t.clone(e.ssFallback)) : e.transition = t;
  }
  function Dt(e, t) {
    return Y(e) ? Ee({
      name: e.name
    }, t, {
      setup: e
    }) : e;
  }
  function zo(e) {
    e.ids = [
      e.ids[0] + e.ids[2]++ + "-",
      0,
      0
    ];
  }
  function oi(e, t) {
    let n;
    return !!((n = Object.getOwnPropertyDescriptor(e, t)) && !n.configurable);
  }
  const fs = /* @__PURE__ */ new WeakMap();
  function $n(e, t, n, s, r = false) {
    if (G(e)) {
      e.forEach((T, D) => $n(T, t && (G(t) ? t[D] : t), n, s, r));
      return;
    }
    if (Mn(s) && !r) {
      s.shapeFlag & 512 && s.type.__asyncResolved && s.component.subTree.component && $n(e, t, n, s.component.subTree);
      return;
    }
    const i = s.shapeFlag & 4 ? Is(s.component) : s.el, a = r ? null : i, { i: o, r: l } = e, c = t && t.r, u = o.refs === fe ? o.refs = {} : o.refs, f = o.setupState, m = oe(f), y = f === fe ? mo : (T) => oi(u, T) ? false : ae(m, T), O = (T, D) => !(D && oi(u, D));
    if (c != null && c !== l) {
      if (ai(t), ve(c)) u[c] = null, y(c) && (f[c] = null);
      else if (Re(c)) {
        const T = t;
        O(c, T.k) && (c.value = null), T.k && (u[T.k] = null);
      }
    }
    if (Y(l)) {
      mt();
      try {
        Gn(l, o, 12, [
          a,
          u
        ]);
      } finally {
        vt();
      }
    } else {
      const T = ve(l), D = Re(l);
      if (T || D) {
        const L = () => {
          if (e.f) {
            const E = T ? y(l) ? f[l] : u[l] : O() || !e.k ? l.value : u[e.k];
            if (r) G(E) && Rr(E, i);
            else if (G(E)) E.includes(i) || E.push(i);
            else if (T) u[l] = [
              i
            ], y(l) && (f[l] = u[l]);
            else {
              const H = [
                i
              ];
              O(l, e.k) && (l.value = H), e.k && (u[e.k] = H);
            }
          } else T ? (u[l] = a, y(l) && (f[l] = a)) : D && (O(l, e.k) && (l.value = a), e.k && (u[e.k] = a));
        };
        if (a) {
          const E = () => {
            L(), fs.delete(e);
          };
          E.id = -1, fs.set(e, E), He(E, n);
        } else ai(e), L();
      }
    }
  }
  function ai(e) {
    const t = fs.get(e);
    t && (t.flags |= 8, fs.delete(e));
  }
  As().requestIdleCallback;
  As().cancelIdleCallback;
  const Mn = (e) => !!e.type.__asyncLoader, Yo = (e) => e.type.__isKeepAlive;
  function zl(e, t) {
    Qo(e, "a", t);
  }
  function Yl(e, t) {
    Qo(e, "da", t);
  }
  function Qo(e, t, n = Ce) {
    const s = e.__wdc || (e.__wdc = () => {
      let r = n;
      for (; r; ) {
        if (r.isDeactivated) return;
        r = r.parent;
      }
      return e();
    });
    if (Rs(t, s, n), n) {
      let r = n.parent;
      for (; r && r.parent; ) Yo(r.parent.vnode) && Ql(s, t, n, r), r = r.parent;
    }
  }
  function Ql(e, t, n, s) {
    const r = Rs(t, e, s, true);
    Fr(() => {
      Rr(s[t], r);
    }, n);
  }
  function Rs(e, t, n = Ce, s = false) {
    if (n) {
      const r = n[e] || (n[e] = []), i = t.__weh || (t.__weh = (...a) => {
        mt();
        const o = Jn(n), l = tt(t, n, e, a);
        return o(), vt(), l;
      });
      return s ? r.unshift(i) : r.push(i), i;
    }
  }
  const Rt = (e) => (t, n = Ce) => {
    (!Wn || e === "sp") && Rs(e, (...s) => t(...s), n);
  }, Xl = Rt("bm"), Xo = Rt("m"), Zl = Rt("bu"), ec = Rt("u"), tc = Rt("bum"), Fr = Rt("um"), nc = Rt("sp"), sc = Rt("rtg"), rc = Rt("rtc");
  function ic(e, t = Ce) {
    Rs("ec", e, t);
  }
  const oc = "directives", ac = Symbol.for("v-ndc");
  function lc(e) {
    return cc(oc, e);
  }
  function cc(e, t, n = true, s = false) {
    const r = We || Ce;
    if (r) {
      const i = r.type, a = li(r[e] || i[e], t) || li(r.appContext[e], t);
      return !a && s ? i : a;
    }
  }
  function li(e, t) {
    return e && (e[t] || e[Be(t)] || e[Tr(Be(t))]);
  }
  function xe(e, t, n, s) {
    let r;
    const i = n, a = G(e);
    if (a || ve(e)) {
      const o = a && Vt(e);
      let l = false, c = false;
      o && (l = !ze(e), c = Ct(e), e = xs(e)), r = new Array(e.length);
      for (let u = 0, f = e.length; u < f; u++) r[u] = t(l ? c ? rn(et(e[u])) : et(e[u]) : e[u], u, void 0, i);
    } else if (typeof e == "number") {
      r = new Array(e);
      for (let o = 0; o < e; o++) r[o] = t(o + 1, o, void 0, i);
    } else if (le(e)) if (e[Symbol.iterator]) r = Array.from(e, (o, l) => t(o, l, void 0, i));
    else {
      const o = Object.keys(e);
      r = new Array(o.length);
      for (let l = 0, c = o.length; l < c; l++) {
        const u = o[l];
        r[l] = t(e[u], u, l, i);
      }
    }
    else r = [];
    return r;
  }
  const ar = (e) => e ? ya(e) ? Is(e) : ar(e.parent) : null, jn = Ee(/* @__PURE__ */ Object.create(null), {
    $: (e) => e,
    $el: (e) => e.vnode.el,
    $data: (e) => e.data,
    $props: (e) => e.props,
    $attrs: (e) => e.attrs,
    $slots: (e) => e.slots,
    $refs: (e) => e.refs,
    $parent: (e) => ar(e.parent),
    $root: (e) => ar(e.root),
    $host: (e) => e.ce,
    $emit: (e) => e.emit,
    $options: (e) => ea(e),
    $forceUpdate: (e) => e.f || (e.f = () => {
      Ur(e.update);
    }),
    $nextTick: (e) => e.n || (e.n = pn.bind(e.proxy)),
    $watch: (e) => Kl.bind(e)
  }), Vs = (e, t) => e !== fe && !e.__isScriptSetup && ae(e, t), uc = {
    get({ _: e }, t) {
      if (t === "__v_skip") return true;
      const { ctx: n, setupState: s, data: r, props: i, accessCache: a, type: o, appContext: l } = e;
      if (t[0] !== "$") {
        const m = a[t];
        if (m !== void 0) switch (m) {
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
          if (Vs(s, t)) return a[t] = 1, s[t];
          if (r !== fe && ae(r, t)) return a[t] = 2, r[t];
          if (ae(i, t)) return a[t] = 3, i[t];
          if (n !== fe && ae(n, t)) return a[t] = 4, n[t];
          lr && (a[t] = 0);
        }
      }
      const c = jn[t];
      let u, f;
      if (c) return t === "$attrs" && Ae(e.attrs, "get", ""), c(e);
      if ((u = o.__cssModules) && (u = u[t])) return u;
      if (n !== fe && ae(n, t)) return a[t] = 4, n[t];
      if (f = l.config.globalProperties, ae(f, t)) return f[t];
    },
    set({ _: e }, t, n) {
      const { data: s, setupState: r, ctx: i } = e;
      return Vs(r, t) ? (r[t] = n, true) : s !== fe && ae(s, t) ? (s[t] = n, true) : ae(e.props, t) || t[0] === "$" && t.slice(1) in e ? false : (i[t] = n, true);
    },
    has({ _: { data: e, setupState: t, accessCache: n, ctx: s, appContext: r, props: i, type: a } }, o) {
      let l;
      return !!(n[o] || e !== fe && o[0] !== "$" && ae(e, o) || Vs(t, o) || ae(i, o) || ae(s, o) || ae(jn, o) || ae(r.config.globalProperties, o) || (l = a.__cssModules) && l[o]);
    },
    defineProperty(e, t, n) {
      return n.get != null ? e._.accessCache[t] = 0 : ae(n, "value") && this.set(e, t, n.value, null), Reflect.defineProperty(e, t, n);
    }
  };
  function ci(e) {
    return G(e) ? e.reduce((t, n) => (t[n] = null, t), {}) : e;
  }
  let lr = true;
  function fc(e) {
    const t = ea(e), n = e.proxy, s = e.ctx;
    lr = false, t.beforeCreate && ui(t.beforeCreate, e, "bc");
    const { data: r, computed: i, methods: a, watch: o, provide: l, inject: c, created: u, beforeMount: f, mounted: m, beforeUpdate: y, updated: O, activated: T, deactivated: D, beforeDestroy: L, beforeUnmount: E, destroyed: H, unmounted: $, render: X, renderTracked: se, renderTriggered: re, errorCaptured: $e, serverPrefetch: Me, expose: je, inheritAttrs: De, components: Ye, directives: Le, filters: yt } = t;
    if (c && dc(c, s, null), a) for (const te in a) {
      const ee = a[te];
      Y(ee) && (s[te] = ee.bind(n));
    }
    if (r) {
      const te = r.call(n, n);
      le(te) && (e.data = hn(te));
    }
    if (lr = true, i) for (const te in i) {
      const ee = i[te], Pe = Y(ee) ? ee.bind(n, n) : Y(ee.get) ? ee.get.bind(n, n) : ht, I = !Y(ee) && Y(ee.set) ? ee.set.bind(n) : ht, M = Q({
        get: Pe,
        set: I
      });
      Object.defineProperty(s, te, {
        enumerable: true,
        configurable: true,
        get: () => M.value,
        set: (F) => M.value = F
      });
    }
    if (o) for (const te in o) Zo(o[te], s, n, te);
    if (l) {
      const te = Y(l) ? l.call(n) : l;
      Reflect.ownKeys(te).forEach((ee) => {
        rs(ee, te[ee]);
      });
    }
    u && ui(u, e, "c");
    function ge(te, ee) {
      G(ee) ? ee.forEach((Pe) => te(Pe.bind(n))) : ee && te(ee.bind(n));
    }
    if (ge(Xl, f), ge(Xo, m), ge(Zl, y), ge(ec, O), ge(zl, T), ge(Yl, D), ge(ic, $e), ge(rc, se), ge(sc, re), ge(tc, E), ge(Fr, $), ge(nc, Me), G(je)) if (je.length) {
      const te = e.exposed || (e.exposed = {});
      je.forEach((ee) => {
        Object.defineProperty(te, ee, {
          get: () => n[ee],
          set: (Pe) => n[ee] = Pe,
          enumerable: true
        });
      });
    } else e.exposed || (e.exposed = {});
    X && e.render === ht && (e.render = X), De != null && (e.inheritAttrs = De), Ye && (e.components = Ye), Le && (e.directives = Le), Me && zo(e);
  }
  function dc(e, t, n = ht) {
    G(e) && (e = cr(e));
    for (const s in e) {
      const r = e[s];
      let i;
      le(r) ? "default" in r ? i = Ze(r.from || s, r.default, true) : i = Ze(r.from || s) : i = Ze(r), Re(i) ? Object.defineProperty(t, s, {
        enumerable: true,
        configurable: true,
        get: () => i.value,
        set: (a) => i.value = a
      }) : t[s] = i;
    }
  }
  function ui(e, t, n) {
    tt(G(e) ? e.map((s) => s.bind(t.proxy)) : e.bind(t.proxy), t, n);
  }
  function Zo(e, t, n, s) {
    let r = s.includes(".") ? Jo(n, s) : () => n[s];
    if (ve(e)) {
      const i = t[e];
      Y(i) && Bt(r, i);
    } else if (Y(e)) Bt(r, e.bind(n));
    else if (le(e)) if (G(e)) e.forEach((i) => Zo(i, t, n, s));
    else {
      const i = Y(e.handler) ? e.handler.bind(n) : t[e.handler];
      Y(i) && Bt(r, i, e);
    }
  }
  function ea(e) {
    const t = e.type, { mixins: n, extends: s } = t, { mixins: r, optionsCache: i, config: { optionMergeStrategies: a } } = e.appContext, o = i.get(t);
    let l;
    return o ? l = o : !r.length && !n && !s ? l = t : (l = {}, r.length && r.forEach((c) => ds(l, c, a, true)), ds(l, t, a)), le(t) && i.set(t, l), l;
  }
  function ds(e, t, n, s = false) {
    const { mixins: r, extends: i } = t;
    i && ds(e, i, n, true), r && r.forEach((a) => ds(e, a, n, true));
    for (const a in t) if (!(s && a === "expose")) {
      const o = hc[a] || n && n[a];
      e[a] = o ? o(e[a], t[a]) : t[a];
    }
    return e;
  }
  const hc = {
    data: fi,
    props: di,
    emits: di,
    methods: kn,
    computed: kn,
    beforeCreate: Ie,
    created: Ie,
    beforeMount: Ie,
    mounted: Ie,
    beforeUpdate: Ie,
    updated: Ie,
    beforeDestroy: Ie,
    beforeUnmount: Ie,
    destroyed: Ie,
    unmounted: Ie,
    activated: Ie,
    deactivated: Ie,
    errorCaptured: Ie,
    serverPrefetch: Ie,
    components: kn,
    directives: kn,
    watch: gc,
    provide: fi,
    inject: pc
  };
  function fi(e, t) {
    return t ? e ? function() {
      return Ee(Y(e) ? e.call(this, this) : e, Y(t) ? t.call(this, this) : t);
    } : t : e;
  }
  function pc(e, t) {
    return kn(cr(e), cr(t));
  }
  function cr(e) {
    if (G(e)) {
      const t = {};
      for (let n = 0; n < e.length; n++) t[e[n]] = e[n];
      return t;
    }
    return e;
  }
  function Ie(e, t) {
    return e ? [
      ...new Set([].concat(e, t))
    ] : t;
  }
  function kn(e, t) {
    return e ? Ee(/* @__PURE__ */ Object.create(null), e, t) : t;
  }
  function di(e, t) {
    return e ? G(e) && G(t) ? [
      .../* @__PURE__ */ new Set([
        ...e,
        ...t
      ])
    ] : Ee(/* @__PURE__ */ Object.create(null), ci(e), ci(t ?? {})) : t;
  }
  function gc(e, t) {
    if (!e) return t;
    if (!t) return e;
    const n = Ee(/* @__PURE__ */ Object.create(null), e);
    for (const s in t) n[s] = Ie(e[s], t[s]);
    return n;
  }
  function ta() {
    return {
      app: null,
      config: {
        isNativeTag: mo,
        performance: false,
        globalProperties: {},
        optionMergeStrategies: {},
        errorHandler: void 0,
        warnHandler: void 0,
        compilerOptions: {}
      },
      mixins: [],
      components: {},
      directives: {},
      provides: /* @__PURE__ */ Object.create(null),
      optionsCache: /* @__PURE__ */ new WeakMap(),
      propsCache: /* @__PURE__ */ new WeakMap(),
      emitsCache: /* @__PURE__ */ new WeakMap()
    };
  }
  let mc = 0;
  function vc(e, t) {
    return function(s, r = null) {
      Y(s) || (s = Ee({}, s)), r != null && !le(r) && (r = null);
      const i = ta(), a = /* @__PURE__ */ new WeakSet(), o = [];
      let l = false;
      const c = i.app = {
        _uid: mc++,
        _component: s,
        _props: r,
        _container: null,
        _context: i,
        _instance: null,
        version: Yc,
        get config() {
          return i.config;
        },
        set config(u) {
        },
        use(u, ...f) {
          return a.has(u) || (u && Y(u.install) ? (a.add(u), u.install(c, ...f)) : Y(u) && (a.add(u), u(c, ...f))), c;
        },
        mixin(u) {
          return i.mixins.includes(u) || i.mixins.push(u), c;
        },
        component(u, f) {
          return f ? (i.components[u] = f, c) : i.components[u];
        },
        directive(u, f) {
          return f ? (i.directives[u] = f, c) : i.directives[u];
        },
        mount(u, f, m) {
          if (!l) {
            const y = c._ceVNode || ke(s, r);
            return y.appContext = i, m === true ? m = "svg" : m === false && (m = void 0), e(y, u, m), l = true, c._container = u, u.__vue_app__ = c, Is(y.component);
          }
        },
        onUnmount(u) {
          o.push(u);
        },
        unmount() {
          l && (tt(o, c._instance, 16), e(null, c._container), delete c._container.__vue_app__);
        },
        provide(u, f) {
          return i.provides[u] = f, c;
        },
        runWithContext(u) {
          const f = tn;
          tn = c;
          try {
            return u();
          } finally {
            tn = f;
          }
        }
      };
      return c;
    };
  }
  let tn = null;
  const yc = (e, t) => t === "modelValue" || t === "model-value" ? e.modelModifiers : e[`${t}Modifiers`] || e[`${Be(t)}Modifiers`] || e[`${jt(t)}Modifiers`];
  function _c(e, t, ...n) {
    if (e.isUnmounted) return;
    const s = e.vnode.props || fe;
    let r = n;
    const i = t.startsWith("update:"), a = i && yc(s, t.slice(7));
    a && (a.trim && (r = n.map((u) => ve(u) ? u.trim() : u)), a.number && (r = n.map(wo)));
    let o, l = s[o = Ds(t)] || s[o = Ds(Be(t))];
    !l && i && (l = s[o = Ds(jt(t))]), l && tt(l, e, 6, r);
    const c = s[o + "Once"];
    if (c) {
      if (!e.emitted) e.emitted = {};
      else if (e.emitted[o]) return;
      e.emitted[o] = true, tt(c, e, 6, r);
    }
  }
  const bc = /* @__PURE__ */ new WeakMap();
  function na(e, t, n = false) {
    const s = n ? bc : t.emitsCache, r = s.get(e);
    if (r !== void 0) return r;
    const i = e.emits;
    let a = {}, o = false;
    if (!Y(e)) {
      const l = (c) => {
        const u = na(c, t, true);
        u && (o = true, Ee(a, u));
      };
      !n && t.mixins.length && t.mixins.forEach(l), e.extends && l(e.extends), e.mixins && e.mixins.forEach(l);
    }
    return !i && !o ? (le(e) && s.set(e, null), null) : (G(i) ? i.forEach((l) => a[l] = null) : Ee(a, i), le(e) && s.set(e, a), a);
  }
  function Ps(e, t) {
    return !e || !Ss(t) ? false : (t = t.slice(2), t = t === "Once" ? t : t.replace(/Once$/, ""), ae(e, t[0].toLowerCase() + t.slice(1)) || ae(e, jt(t)) || ae(e, t));
  }
  function hi(e) {
    const { type: t, vnode: n, proxy: s, withProxy: r, propsOptions: [i], slots: a, attrs: o, emit: l, render: c, renderCache: u, props: f, data: m, setupState: y, ctx: O, inheritAttrs: T } = e, D = us(e);
    let L, E;
    try {
      if (n.shapeFlag & 4) {
        const $ = r || s, X = $;
        L = ft(c.call(X, $, u, f, y, m, O)), E = o;
      } else {
        const $ = t;
        L = ft($.length > 1 ? $(f, {
          attrs: o,
          slots: a,
          emit: l
        }) : $(f, null)), E = t.props ? o : wc(o);
      }
    } catch ($) {
      Dn.length = 0, Cs($, e, 1), L = ke($t);
    }
    let H = L;
    if (E && T !== false) {
      const $ = Object.keys(E), { shapeFlag: X } = H;
      $.length && X & 7 && (i && $.some(ks) && (E = Sc(E, i)), H = on(H, E, false, true));
    }
    return n.dirs && (H = on(H, null, false, true), H.dirs = H.dirs ? H.dirs.concat(n.dirs) : n.dirs), n.transition && Hr(H, n.transition), L = H, us(D), L;
  }
  const wc = (e) => {
    let t;
    for (const n in e) (n === "class" || n === "style" || Ss(n)) && ((t || (t = {}))[n] = e[n]);
    return t;
  }, Sc = (e, t) => {
    const n = {};
    for (const s in e) (!ks(s) || !(s.slice(9) in t)) && (n[s] = e[s]);
    return n;
  };
  function kc(e, t, n) {
    const { props: s, children: r, component: i } = e, { props: a, children: o, patchFlag: l } = t, c = i.emitsOptions;
    if (t.dirs || t.transition) return true;
    if (n && l >= 0) {
      if (l & 1024) return true;
      if (l & 16) return s ? pi(s, a, c) : !!a;
      if (l & 8) {
        const u = t.dynamicProps;
        for (let f = 0; f < u.length; f++) {
          const m = u[f];
          if (sa(a, s, m) && !Ps(c, m)) return true;
        }
      }
    } else return (r || o) && (!o || !o.$stable) ? true : s === a ? false : s ? a ? pi(s, a, c) : true : !!a;
    return false;
  }
  function pi(e, t, n) {
    const s = Object.keys(t);
    if (s.length !== Object.keys(e).length) return true;
    for (let r = 0; r < s.length; r++) {
      const i = s[r];
      if (sa(t, e, i) && !Ps(n, i)) return true;
    }
    return false;
  }
  function sa(e, t, n) {
    const s = e[n], r = t[n];
    return n === "style" && le(s) && le(r) ? !dn(s, r) : s !== r;
  }
  function Ec({ vnode: e, parent: t, suspense: n }, s) {
    for (; t; ) {
      const r = t.subTree;
      if (r.suspense && r.suspense.activeBranch === e && (r.suspense.vnode.el = r.el = s, e = r), r === e) (e = t.vnode).el = s, t = t.parent;
      else break;
    }
    n && n.activeBranch === e && (n.vnode.el = s);
  }
  const ra = {}, ia = () => Object.create(ra), oa = (e) => Object.getPrototypeOf(e) === ra;
  function Ac(e, t, n, s = false) {
    const r = {}, i = ia();
    e.propsDefaults = /* @__PURE__ */ Object.create(null), aa(e, t, r, i);
    for (const a in e.propsOptions[0]) a in r || (r[a] = void 0);
    n ? e.props = s ? r : Uo(r) : e.type.props ? e.props = r : e.props = i, e.attrs = i;
  }
  function xc(e, t, n, s) {
    const { props: r, attrs: i, vnode: { patchFlag: a } } = e, o = oe(r), [l] = e.propsOptions;
    let c = false;
    if ((s || a > 0) && !(a & 16)) {
      if (a & 8) {
        const u = e.vnode.dynamicProps;
        for (let f = 0; f < u.length; f++) {
          let m = u[f];
          if (Ps(e.emitsOptions, m)) continue;
          const y = t[m];
          if (l) if (ae(i, m)) y !== i[m] && (i[m] = y, c = true);
          else {
            const O = Be(m);
            r[O] = ur(l, o, O, y, e, false);
          }
          else y !== i[m] && (i[m] = y, c = true);
        }
      }
    } else {
      aa(e, t, r, i) && (c = true);
      let u;
      for (const f in o) (!t || !ae(t, f) && ((u = jt(f)) === f || !ae(t, u))) && (l ? n && (n[f] !== void 0 || n[u] !== void 0) && (r[f] = ur(l, o, f, void 0, e, true)) : delete r[f]);
      if (i !== o) for (const f in i) (!t || !ae(t, f)) && (delete i[f], c = true);
    }
    c && Et(e.attrs, "set", "");
  }
  function aa(e, t, n, s) {
    const [r, i] = e.propsOptions;
    let a = false, o;
    if (t) for (let l in t) {
      if (In(l)) continue;
      const c = t[l];
      let u;
      r && ae(r, u = Be(l)) ? !i || !i.includes(u) ? n[u] = c : (o || (o = {}))[u] = c : Ps(e.emitsOptions, l) || (!(l in s) || c !== s[l]) && (s[l] = c, a = true);
    }
    if (i) {
      const l = oe(n), c = o || fe;
      for (let u = 0; u < i.length; u++) {
        const f = i[u];
        n[f] = ur(r, l, f, c[f], e, !ae(c, f));
      }
    }
    return a;
  }
  function ur(e, t, n, s, r, i) {
    const a = e[n];
    if (a != null) {
      const o = ae(a, "default");
      if (o && s === void 0) {
        const l = a.default;
        if (a.type !== Function && !a.skipFactory && Y(l)) {
          const { propsDefaults: c } = r;
          if (n in c) s = c[n];
          else {
            const u = Jn(r);
            s = c[n] = l.call(null, t), u();
          }
        } else s = l;
        r.ce && r.ce._setProp(n, s);
      }
      a[0] && (i && !o ? s = false : a[1] && (s === "" || s === jt(n)) && (s = true));
    }
    return s;
  }
  const Cc = /* @__PURE__ */ new WeakMap();
  function la(e, t, n = false) {
    const s = n ? Cc : t.propsCache, r = s.get(e);
    if (r) return r;
    const i = e.props, a = {}, o = [];
    let l = false;
    if (!Y(e)) {
      const u = (f) => {
        l = true;
        const [m, y] = la(f, t, true);
        Ee(a, m), y && o.push(...y);
      };
      !n && t.mixins.length && t.mixins.forEach(u), e.extends && u(e.extends), e.mixins && e.mixins.forEach(u);
    }
    if (!i && !l) return le(e) && s.set(e, Xt), Xt;
    if (G(i)) for (let u = 0; u < i.length; u++) {
      const f = Be(i[u]);
      gi(f) && (a[f] = fe);
    }
    else if (i) for (const u in i) {
      const f = Be(u);
      if (gi(f)) {
        const m = i[u], y = a[f] = G(m) || Y(m) ? {
          type: m
        } : Ee({}, m), O = y.type;
        let T = false, D = true;
        if (G(O)) for (let L = 0; L < O.length; ++L) {
          const E = O[L], H = Y(E) && E.name;
          if (H === "Boolean") {
            T = true;
            break;
          } else H === "String" && (D = false);
        }
        else T = Y(O) && O.name === "Boolean";
        y[0] = T, y[1] = D, (T || ae(y, "default")) && o.push(f);
      }
    }
    const c = [
      a,
      o
    ];
    return le(e) && s.set(e, c), c;
  }
  function gi(e) {
    return e[0] !== "$" && !In(e);
  }
  const qr = (e) => e === "_" || e === "_ctx" || e === "$stable", Wr = (e) => G(e) ? e.map(ft) : [
    ft(e)
  ], Rc = (e, t, n) => {
    if (t._n) return t;
    const s = Wl((...r) => Wr(t(...r)), n);
    return s._c = false, s;
  }, ca = (e, t, n) => {
    const s = e._ctx;
    for (const r in e) {
      if (qr(r)) continue;
      const i = e[r];
      if (Y(i)) t[r] = Rc(r, i, s);
      else if (i != null) {
        const a = Wr(i);
        t[r] = () => a;
      }
    }
  }, ua = (e, t) => {
    const n = Wr(t);
    e.slots.default = () => n;
  }, fa = (e, t, n) => {
    for (const s in t) (n || !qr(s)) && (e[s] = t[s]);
  }, Pc = (e, t, n) => {
    const s = e.slots = ia();
    if (e.vnode.shapeFlag & 32) {
      const r = t._;
      r ? (fa(s, t, n), n && bo(s, "_", r, true)) : ca(t, s);
    } else t && ua(e, t);
  }, Tc = (e, t, n) => {
    const { vnode: s, slots: r } = e;
    let i = true, a = fe;
    if (s.shapeFlag & 32) {
      const o = t._;
      o ? n && o === 1 ? i = false : fa(r, t, n) : (i = !t.$stable, ca(t, r)), a = t;
    } else t && (ua(e, t), a = {
      default: 1
    });
    if (i) for (const o in r) !qr(o) && a[o] == null && delete r[o];
  }, He = Mc;
  function Ic(e) {
    return Oc(e);
  }
  function Oc(e, t) {
    const n = As();
    n.__VUE__ = true;
    const { insert: s, remove: r, patchProp: i, createElement: a, createText: o, createComment: l, setText: c, setElementText: u, parentNode: f, nextSibling: m, setScopeId: y = ht, insertStaticContent: O } = e, T = (d, h, _, b = null, g = null, p = null, w = void 0, v = null, R = !!h.dynamicChildren) => {
      if (d === h) return;
      d && !yn(d, h) && (b = S(d), F(d, g, p, true), d = null), h.patchFlag === -2 && (R = false, h.dynamicChildren = null);
      const { type: A, ref: K, shapeFlag: j } = h;
      switch (A) {
        case Ts:
          D(d, h, _, b);
          break;
        case $t:
          L(d, h, _, b);
          break;
        case is:
          d == null && E(h, _, b, w);
          break;
        case de:
          Ye(d, h, _, b, g, p, w, v, R);
          break;
        default:
          j & 1 ? X(d, h, _, b, g, p, w, v, R) : j & 6 ? Le(d, h, _, b, g, p, w, v, R) : (j & 64 || j & 128) && A.process(d, h, _, b, g, p, w, v, R, q);
      }
      K != null && g ? $n(K, d && d.ref, p, h || d, !h) : K == null && d && d.ref != null && $n(d.ref, null, p, d, true);
    }, D = (d, h, _, b) => {
      if (d == null) s(h.el = o(h.children), _, b);
      else {
        const g = h.el = d.el;
        h.children !== d.children && c(g, h.children);
      }
    }, L = (d, h, _, b) => {
      d == null ? s(h.el = l(h.children || ""), _, b) : h.el = d.el;
    }, E = (d, h, _, b) => {
      [d.el, d.anchor] = O(d.children, h, _, b, d.el, d.anchor);
    }, H = ({ el: d, anchor: h }, _, b) => {
      let g;
      for (; d && d !== h; ) g = m(d), s(d, _, b), d = g;
      s(h, _, b);
    }, $ = ({ el: d, anchor: h }) => {
      let _;
      for (; d && d !== h; ) _ = m(d), r(d), d = _;
      r(h);
    }, X = (d, h, _, b, g, p, w, v, R) => {
      if (h.type === "svg" ? w = "svg" : h.type === "math" && (w = "mathml"), d == null) se(h, _, b, g, p, w, v, R);
      else {
        const A = d.el && d.el._isVueCE ? d.el : null;
        try {
          A && A._beginPatch(), Me(d, h, g, p, w, v, R);
        } finally {
          A && A._endPatch();
        }
      }
    }, se = (d, h, _, b, g, p, w, v) => {
      let R, A;
      const { props: K, shapeFlag: j, transition: V, dirs: J } = d;
      if (R = d.el = a(d.type, p, K && K.is, K), j & 8 ? u(R, d.children) : j & 16 && $e(d.children, R, null, b, g, Bs(d, p), w, v), J && Lt(d, null, b, "created"), re(R, d, d.scopeId, w, b), K) {
        for (const ue in K) ue !== "value" && !In(ue) && i(R, ue, null, K[ue], p, b);
        "value" in K && i(R, "value", null, K.value, p), (A = K.onVnodeBeforeMount) && lt(A, b, d);
      }
      J && Lt(d, null, b, "beforeMount");
      const ne = Nc(g, V);
      ne && V.beforeEnter(R), s(R, h, _), ((A = K && K.onVnodeMounted) || ne || J) && He(() => {
        try {
          A && lt(A, b, d), ne && V.enter(R), J && Lt(d, null, b, "mounted");
        } finally {
        }
      }, g);
    }, re = (d, h, _, b, g) => {
      if (_ && y(d, _), b) for (let p = 0; p < b.length; p++) y(d, b[p]);
      if (g) {
        let p = g.subTree;
        if (h === p || ga(p.type) && (p.ssContent === h || p.ssFallback === h)) {
          const w = g.vnode;
          re(d, w, w.scopeId, w.slotScopeIds, g.parent);
        }
      }
    }, $e = (d, h, _, b, g, p, w, v, R = 0) => {
      for (let A = R; A < d.length; A++) {
        const K = d[A] = v ? kt(d[A]) : ft(d[A]);
        T(null, K, h, _, b, g, p, w, v);
      }
    }, Me = (d, h, _, b, g, p, w) => {
      const v = h.el = d.el;
      let { patchFlag: R, dynamicChildren: A, dirs: K } = h;
      R |= d.patchFlag & 16;
      const j = d.props || fe, V = h.props || fe;
      let J;
      if (_ && Ut(_, false), (J = V.onVnodeBeforeUpdate) && lt(J, _, h, d), K && Lt(h, d, _, "beforeUpdate"), _ && Ut(_, true), A && (!d.dynamicChildren || d.dynamicChildren.length !== A.length) && (R = 0, w = false, A = null), (j.innerHTML && V.innerHTML == null || j.textContent && V.textContent == null) && u(v, ""), A ? je(d.dynamicChildren, A, v, _, b, Bs(h, g), p) : w || ee(d, h, v, null, _, b, Bs(h, g), p, false), R > 0) {
        if (R & 16) De(v, j, V, _, g);
        else if (R & 2 && j.class !== V.class && i(v, "class", null, V.class, g), R & 4 && i(v, "style", j.style, V.style, g), R & 8) {
          const ne = h.dynamicProps;
          for (let ue = 0; ue < ne.length; ue++) {
            const ce = ne[ue], _e = j[ce], we = V[ce];
            (we !== _e || ce === "value") && i(v, ce, _e, we, g, _);
          }
        }
        R & 1 && d.children !== h.children && u(v, h.children);
      } else !w && A == null && De(v, j, V, _, g);
      ((J = V.onVnodeUpdated) || K) && He(() => {
        J && lt(J, _, h, d), K && Lt(h, d, _, "updated");
      }, b);
    }, je = (d, h, _, b, g, p, w) => {
      for (let v = 0; v < h.length; v++) {
        const R = d[v], A = h[v], K = R.el && (R.type === de || !yn(R, A) || R.shapeFlag & 198) ? f(R.el) : _;
        T(R, A, K, null, b, g, p, w, true);
      }
    }, De = (d, h, _, b, g) => {
      if (h !== _) {
        if (h !== fe) for (const p in h) !In(p) && !(p in _) && i(d, p, h[p], null, g, b);
        for (const p in _) {
          if (In(p)) continue;
          const w = _[p], v = h[p];
          w !== v && p !== "value" && i(d, p, v, w, g, b);
        }
        "value" in _ && i(d, "value", h.value, _.value, g);
      }
    }, Ye = (d, h, _, b, g, p, w, v, R) => {
      const A = h.el = d ? d.el : o(""), K = h.anchor = d ? d.anchor : o("");
      let { patchFlag: j, dynamicChildren: V, slotScopeIds: J } = h;
      J && (v = v ? v.concat(J) : J), d == null ? (s(A, _, b), s(K, _, b), $e(h.children || [], _, K, g, p, w, v, R)) : j > 0 && j & 64 && V && d.dynamicChildren && d.dynamicChildren.length === V.length ? (je(d.dynamicChildren, V, _, g, p, w, v), (h.key != null || g && h === g.subTree) && da(d, h, true)) : ee(d, h, _, K, g, p, w, v, R);
    }, Le = (d, h, _, b, g, p, w, v, R) => {
      h.slotScopeIds = v, d == null ? h.shapeFlag & 512 ? g.ctx.activate(h, _, b, w, R) : yt(h, _, b, g, p, w, R) : st(d, h, R);
    }, yt = (d, h, _, b, g, p, w) => {
      const v = d.component = Wc(d, b, g);
      if (Yo(d) && (v.ctx.renderer = q), Bc(v, false, w), v.asyncDep) {
        if (g && g.registerDep(v, ge, w), !d.el) {
          const R = v.subTree = ke($t);
          L(null, R, h, _), d.placeholder = R.el;
        }
      } else ge(v, d, h, _, g, p, w);
    }, st = (d, h, _) => {
      const b = h.component = d.component;
      if (kc(d, h, _)) if (b.asyncDep && !b.asyncResolved) {
        te(b, h, _);
        return;
      } else b.next = h, b.update();
      else h.el = d.el, b.vnode = h;
    }, ge = (d, h, _, b, g, p, w) => {
      const v = () => {
        if (d.isMounted) {
          let { next: j, bu: V, u: J, parent: ne, vnode: ue } = d;
          {
            const ot = ha(d);
            if (ot) {
              j && (j.el = ue.el, te(d, j, w)), ot.asyncDep.then(() => {
                He(() => {
                  d.isUnmounted || A();
                }, g);
              });
              return;
            }
          }
          let ce = j, _e;
          Ut(d, false), j ? (j.el = ue.el, te(d, j, w)) : j = ue, V && ss(V), (_e = j.props && j.props.onVnodeBeforeUpdate) && lt(_e, ne, j, ue), Ut(d, true);
          const we = hi(d), it = d.subTree;
          d.subTree = we, T(it, we, f(it.el), S(it), d, g, p), j.el = we.el, ce === null && Ec(d, we.el), J && He(J, g), (_e = j.props && j.props.onVnodeUpdated) && He(() => lt(_e, ne, j, ue), g);
        } else {
          let j;
          const { el: V, props: J } = h, { bm: ne, m: ue, parent: ce, root: _e, type: we } = d, it = Mn(h);
          Ut(d, false), ne && ss(ne), !it && (j = J && J.onVnodeBeforeMount) && lt(j, ce, h), Ut(d, true);
          {
            _e.ce && _e.ce._hasShadowRoot() && _e.ce._injectChildStyle(we, d.parent ? d.parent.type : void 0);
            const ot = d.subTree = hi(d);
            T(null, ot, _, b, d, g, p), h.el = ot.el;
          }
          if (ue && He(ue, g), !it && (j = J && J.onVnodeMounted)) {
            const ot = h;
            He(() => lt(j, ce, ot), g);
          }
          (h.shapeFlag & 256 || ce && Mn(ce.vnode) && ce.vnode.shapeFlag & 256) && d.a && He(d.a, g), d.isMounted = true, h = _ = b = null;
        }
      };
      d.scope.on();
      const R = d.effect = new Ao(v);
      d.scope.off();
      const A = d.update = R.run.bind(R), K = d.job = R.runIfDirty.bind(R);
      K.i = d, K.id = d.uid, R.scheduler = () => Ur(K), Ut(d, true), A();
    }, te = (d, h, _) => {
      h.component = d;
      const b = d.vnode.props;
      d.vnode = h, d.next = null, xc(d, h.props, b, _), Tc(d, h.children, _), mt(), ii(d), vt();
    }, ee = (d, h, _, b, g, p, w, v, R = false) => {
      const A = d && d.children, K = d ? d.shapeFlag : 0, j = h.children, { patchFlag: V, shapeFlag: J } = h;
      if (V > 0) {
        if (V & 128) {
          I(A, j, _, b, g, p, w, v, R);
          return;
        } else if (V & 256) {
          Pe(A, j, _, b, g, p, w, v, R);
          return;
        }
      }
      J & 8 ? (K & 16 && Ue(A, g, p), j !== A && u(_, j)) : K & 16 ? J & 16 ? I(A, j, _, b, g, p, w, v, R) : Ue(A, g, p, true) : (K & 8 && u(_, ""), J & 16 && $e(j, _, b, g, p, w, v, R));
    }, Pe = (d, h, _, b, g, p, w, v, R) => {
      d = d || Xt, h = h || Xt;
      const A = d.length, K = h.length, j = Math.min(A, K);
      let V;
      for (V = 0; V < j; V++) {
        const J = h[V] = R ? kt(h[V]) : ft(h[V]);
        T(d[V], J, _, null, g, p, w, v, R);
      }
      A > K ? Ue(d, g, p, true, false, j) : $e(h, _, b, g, p, w, v, R, j);
    }, I = (d, h, _, b, g, p, w, v, R) => {
      let A = 0;
      const K = h.length;
      let j = d.length - 1, V = K - 1;
      for (; A <= j && A <= V; ) {
        const J = d[A], ne = h[A] = R ? kt(h[A]) : ft(h[A]);
        if (yn(J, ne)) T(J, ne, _, null, g, p, w, v, R);
        else break;
        A++;
      }
      for (; A <= j && A <= V; ) {
        const J = d[j], ne = h[V] = R ? kt(h[V]) : ft(h[V]);
        if (yn(J, ne)) T(J, ne, _, null, g, p, w, v, R);
        else break;
        j--, V--;
      }
      if (A > j) {
        if (A <= V) {
          const J = V + 1, ne = J < K ? h[J].el : b;
          for (; A <= V; ) T(null, h[A] = R ? kt(h[A]) : ft(h[A]), _, ne, g, p, w, v, R), A++;
        }
      } else if (A > V) for (; A <= j; ) F(d[A], g, p, true), A++;
      else {
        const J = A, ne = A, ue = /* @__PURE__ */ new Map();
        for (A = ne; A <= V; A++) {
          const qe = h[A] = R ? kt(h[A]) : ft(h[A]);
          qe.key != null && ue.set(qe.key, A);
        }
        let ce, _e = 0;
        const we = V - ne + 1;
        let it = false, ot = 0;
        const mn = new Array(we);
        for (A = 0; A < we; A++) mn[A] = 0;
        for (A = J; A <= j; A++) {
          const qe = d[A];
          if (_e >= we) {
            F(qe, g, p, true);
            continue;
          }
          let at;
          if (qe.key != null) at = ue.get(qe.key);
          else for (ce = ne; ce <= V; ce++) if (mn[ce - ne] === 0 && yn(qe, h[ce])) {
            at = ce;
            break;
          }
          at === void 0 ? F(qe, g, p, true) : (mn[at - ne] = A + 1, at >= ot ? ot = at : it = true, T(qe, h[at], _, null, g, p, w, v, R), _e++);
        }
        const Xr = it ? $c(mn) : Xt;
        for (ce = Xr.length - 1, A = we - 1; A >= 0; A--) {
          const qe = ne + A, at = h[qe], Zr = h[qe + 1], ei = qe + 1 < K ? Zr.el || pa(Zr) : b;
          mn[A] === 0 ? T(null, at, _, ei, g, p, w, v, R) : it && (ce < 0 || A !== Xr[ce] ? M(at, _, ei, 2) : ce--);
        }
      }
    }, M = (d, h, _, b, g = null) => {
      const { el: p, type: w, transition: v, children: R, shapeFlag: A } = d;
      if (A & 6) {
        M(d.component.subTree, h, _, b);
        return;
      }
      if (A & 128) {
        d.suspense.move(h, _, b);
        return;
      }
      if (A & 64) {
        w.move(d, h, _, q);
        return;
      }
      if (w === de) {
        s(p, h, _);
        for (let j = 0; j < R.length; j++) M(R[j], h, _, b);
        s(d.anchor, h, _);
        return;
      }
      if (w === is) {
        H(d, h, _);
        return;
      }
      if (b !== 2 && A & 1 && v) if (b === 0) v.persisted && !p[Ws] ? s(p, h, _) : (v.beforeEnter(p), s(p, h, _), He(() => v.enter(p), g));
      else {
        const { leave: j, delayLeave: V, afterLeave: J } = v, ne = () => {
          d.ctx.isUnmounted ? r(p) : s(p, h, _);
        }, ue = () => {
          const ce = p._isLeaving || !!p[Ws];
          p._isLeaving && p[Ws](true), v.persisted && !ce ? ne() : j(p, () => {
            ne(), J && J();
          });
        };
        V ? V(p, ne, ue) : ue();
      }
      else s(p, h, _);
    }, F = (d, h, _, b = false, g = false) => {
      const { type: p, props: w, ref: v, children: R, dynamicChildren: A, shapeFlag: K, patchFlag: j, dirs: V, cacheIndex: J, memo: ne } = d;
      if (j === -2 && (g = false), v != null && (mt(), $n(v, null, _, d, true), vt()), J != null && (h.renderCache[J] = void 0), K & 256) {
        h.ctx.deactivate(d);
        return;
      }
      const ue = K & 1 && V, ce = !Mn(d);
      let _e;
      if (ce && (_e = w && w.onVnodeBeforeUnmount) && lt(_e, h, d), K & 6) rt(d.component, _, b);
      else {
        if (K & 128) {
          d.suspense.unmount(_, b);
          return;
        }
        ue && Lt(d, null, h, "beforeUnmount"), K & 64 ? d.type.remove(d, h, _, q, b) : A && !A.hasOnce && (p !== de || j > 0 && j & 64) ? Ue(A, h, _, false, true) : (p === de && j & 384 || !g && K & 16) && Ue(R, h, _), b && ie(d);
      }
      const we = ne != null && J == null;
      (ce && (_e = w && w.onVnodeUnmounted) || ue || we) && He(() => {
        _e && lt(_e, h, d), ue && Lt(d, null, h, "unmounted"), we && (d.el = null);
      }, _);
    }, ie = (d) => {
      const { type: h, el: _, anchor: b, transition: g } = d;
      if (h === de) {
        Te(_, b);
        return;
      }
      if (h === is) {
        $(d);
        return;
      }
      const p = () => {
        r(_), g && !g.persisted && g.afterLeave && g.afterLeave();
      };
      if (d.shapeFlag & 1 && g && !g.persisted) {
        const { leave: w, delayLeave: v } = g, R = () => w(_, p);
        v ? v(d.el, p, R) : R();
      } else p();
    }, Te = (d, h) => {
      let _;
      for (; d !== h; ) _ = m(d), r(d), d = _;
      r(h);
    }, rt = (d, h, _) => {
      const { bum: b, scope: g, job: p, subTree: w, um: v, m: R, a: A } = d;
      mi(R), mi(A), b && ss(b), g.stop(), p && (p.flags |= 8, F(w, d, h, _)), v && He(v, h), He(() => {
        d.isUnmounted = true;
      }, h);
    }, Ue = (d, h, _, b = false, g = false, p = 0) => {
      for (let w = p; w < d.length; w++) F(d[w], h, _, b, g);
    }, S = (d) => {
      if (d.shapeFlag & 6) return S(d.component.subTree);
      if (d.shapeFlag & 128) return d.suspense.next();
      const h = m(d.anchor || d.el), _ = h && h[Gl];
      return _ ? m(_) : h;
    };
    let U = false;
    const N = (d, h, _) => {
      let b;
      d == null ? h._vnode && (F(h._vnode, null, null, true), b = h._vnode.component) : T(h._vnode || null, d, h, null, null, null, _), h._vnode = d, U || (U = true, ii(b), Vo(), U = false);
    }, q = {
      p: T,
      um: F,
      m: M,
      r: ie,
      mt: yt,
      mc: $e,
      pc: ee,
      pbc: je,
      n: S,
      o: e
    };
    return {
      render: N,
      hydrate: void 0,
      createApp: vc(N)
    };
  }
  function Bs({ type: e, props: t }, n) {
    return n === "svg" && e === "foreignObject" || n === "mathml" && e === "annotation-xml" && t && t.encoding && t.encoding.includes("html") ? void 0 : n;
  }
  function Ut({ effect: e, job: t }, n) {
    n ? (e.flags |= 32, t.flags |= 4) : (e.flags &= -33, t.flags &= -5);
  }
  function Nc(e, t) {
    return (!e || e && !e.pendingBranch) && t && !t.persisted;
  }
  function da(e, t, n = false) {
    const s = e.children, r = t.children;
    if (G(s) && G(r)) for (let i = 0; i < s.length; i++) {
      const a = s[i];
      let o = r[i];
      o.shapeFlag & 1 && !o.dynamicChildren && ((o.patchFlag <= 0 || o.patchFlag === 32) && (o = r[i] = kt(r[i]), o.el = a.el), !n && o.patchFlag !== -2 && da(a, o)), o.type === Ts && (o.patchFlag === -1 && (o = r[i] = kt(o)), o.el = a.el), o.type === $t && !o.el && (o.el = a.el);
    }
  }
  function $c(e) {
    const t = e.slice(), n = [
      0
    ];
    let s, r, i, a, o;
    const l = e.length;
    for (s = 0; s < l; s++) {
      const c = e[s];
      if (c !== 0) {
        if (r = n[n.length - 1], e[r] < c) {
          t[s] = r, n.push(s);
          continue;
        }
        for (i = 0, a = n.length - 1; i < a; ) o = i + a >> 1, e[n[o]] < c ? i = o + 1 : a = o;
        c < e[n[i]] && (i > 0 && (t[s] = n[i - 1]), n[i] = s);
      }
    }
    for (i = n.length, a = n[i - 1]; i-- > 0; ) n[i] = a, a = t[a];
    return n;
  }
  function ha(e) {
    const t = e.subTree.component;
    if (t) return t.asyncDep && !t.asyncResolved ? t : ha(t);
  }
  function mi(e) {
    if (e) for (let t = 0; t < e.length; t++) e[t].flags |= 8;
  }
  function pa(e) {
    if (e.placeholder) return e.placeholder;
    const t = e.component;
    return t ? pa(t.subTree) : null;
  }
  const ga = (e) => e.__isSuspense;
  function Mc(e, t) {
    t && t.pendingBranch ? G(e) ? t.effects.push(...e) : t.effects.push(e) : ql(e);
  }
  const de = Symbol.for("v-fgt"), Ts = Symbol.for("v-txt"), $t = Symbol.for("v-cmt"), is = Symbol.for("v-stc"), Dn = [];
  let Ve = null;
  function x(e = false) {
    Dn.push(Ve = e ? null : []);
  }
  function jc() {
    Dn.pop(), Ve = Dn[Dn.length - 1] || null;
  }
  let qn = 1;
  function hs(e, t = false) {
    qn += e, e < 0 && Ve && t && (Ve.hasOnce = true);
  }
  function ma(e) {
    return e.dynamicChildren = qn > 0 ? Ve || Xt : null, jc(), qn > 0 && Ve && Ve.push(e), e;
  }
  function C(e, t, n, s, r, i) {
    return ma(k(e, t, n, s, r, i, true));
  }
  function Vr(e, t, n, s, r) {
    return ma(ke(e, t, n, s, r, true));
  }
  function ps(e) {
    return e ? e.__v_isVNode === true : false;
  }
  function yn(e, t) {
    return e.type === t.type && e.key === t.key;
  }
  const va = ({ key: e }) => e ?? null, os = ({ ref: e, ref_key: t, ref_for: n }) => (typeof e == "number" && (e = "" + e), e != null ? ve(e) || Re(e) || Y(e) ? {
    i: We,
    r: e,
    k: t,
    f: !!n
  } : e : null);
  function k(e, t = null, n = null, s = 0, r = null, i = e === de ? 0 : 1, a = false, o = false) {
    const l = {
      __v_isVNode: true,
      __v_skip: true,
      type: e,
      props: t,
      key: t && va(t),
      ref: t && os(t),
      scopeId: Ko,
      slotScopeIds: null,
      children: n,
      component: null,
      suspense: null,
      ssContent: null,
      ssFallback: null,
      dirs: null,
      transition: null,
      el: null,
      anchor: null,
      target: null,
      targetStart: null,
      targetAnchor: null,
      staticCount: 0,
      shapeFlag: i,
      patchFlag: s,
      dynamicProps: r,
      dynamicChildren: null,
      appContext: null,
      ctx: We
    };
    return o ? (gs(l, n), i & 128 && e.normalize(l)) : n && (l.shapeFlag |= ve(n) ? 8 : 16), qn > 0 && !a && Ve && (l.patchFlag > 0 || i & 6) && l.patchFlag !== 32 && Ve.push(l), l;
  }
  const ke = Dc;
  function Dc(e, t = null, n = null, s = 0, r = null, i = false) {
    if ((!e || e === ac) && (e = $t), ps(e)) {
      const o = on(e, t, true);
      return n && gs(o, n), qn > 0 && !i && Ve && (o.shapeFlag & 6 ? Ve[Ve.indexOf(e)] = o : Ve.push(o)), o.patchFlag = -2, o;
    }
    if (zc(e) && (e = e.__vccOpts), t) {
      t = Lc(t);
      let { class: o, style: l } = t;
      o && !ve(o) && (t.class = Fe(o)), le(l) && (Lr(l) && !G(l) && (l = Ee({}, l)), t.style = Kt(l));
    }
    const a = ve(e) ? 1 : ga(e) ? 128 : Jl(e) ? 64 : le(e) ? 4 : Y(e) ? 2 : 0;
    return k(e, t, n, s, r, a, i, true);
  }
  function Lc(e) {
    return e ? Lr(e) || oa(e) ? Ee({}, e) : e : null;
  }
  function on(e, t, n = false, s = false) {
    const { props: r, ref: i, patchFlag: a, children: o, transition: l } = e, c = t ? Hc(r || {}, t) : r, u = {
      __v_isVNode: true,
      __v_skip: true,
      type: e.type,
      props: c,
      key: c && va(c),
      ref: t && t.ref ? n && i ? G(i) ? i.concat(os(t)) : [
        i,
        os(t)
      ] : os(t) : i,
      scopeId: e.scopeId,
      slotScopeIds: e.slotScopeIds,
      children: o,
      target: e.target,
      targetStart: e.targetStart,
      targetAnchor: e.targetAnchor,
      staticCount: e.staticCount,
      shapeFlag: e.shapeFlag,
      patchFlag: t && e.type !== de ? a === -1 ? 16 : a | 16 : a,
      dynamicProps: e.dynamicProps,
      dynamicChildren: e.dynamicChildren,
      appContext: e.appContext,
      dirs: e.dirs,
      transition: l,
      component: e.component,
      suspense: e.suspense,
      ssContent: e.ssContent && on(e.ssContent),
      ssFallback: e.ssFallback && on(e.ssFallback),
      placeholder: e.placeholder,
      el: e.el,
      anchor: e.anchor,
      ctx: e.ctx,
      ce: e.ce
    };
    return l && s && Hr(u, l.clone(u)), u;
  }
  function xt(e = " ", t = 0) {
    return ke(Ts, null, e, t);
  }
  function Uc(e, t) {
    const n = ke(is, null, e);
    return n.staticCount = t, n;
  }
  function z(e = "", t = false) {
    return t ? (x(), Vr($t, null, e)) : ke($t, null, e);
  }
  function ft(e) {
    return e == null || typeof e == "boolean" ? ke($t) : G(e) ? ke(de, null, e.slice()) : ps(e) ? kt(e) : ke(Ts, null, String(e));
  }
  function kt(e) {
    return e.el === null && e.patchFlag !== -1 || e.memo ? e : on(e);
  }
  function gs(e, t) {
    let n = 0;
    const { shapeFlag: s } = e;
    if (t == null) t = null;
    else if (G(t)) n = 16;
    else if (typeof t == "object") if (s & 65) {
      const r = t.default;
      r && (r._c && (r._d = false), gs(e, r()), r._c && (r._d = true));
      return;
    } else {
      n = 32;
      const r = t._;
      !r && !oa(t) ? t._ctx = We : r === 3 && We && (We.slots._ === 1 ? t._ = 1 : (t._ = 2, e.patchFlag |= 1024));
    }
    else if (Y(t)) {
      if (s & 65) {
        gs(e, {
          default: t
        });
        return;
      }
      t = {
        default: t,
        _ctx: We
      }, n = 32;
    } else t = String(t), s & 64 ? (n = 16, t = [
      xt(t)
    ]) : n = 8;
    e.children = t, e.shapeFlag |= n;
  }
  function Hc(...e) {
    const t = {};
    for (let n = 0; n < e.length; n++) {
      const s = e[n];
      for (const r in s) if (r === "class") t.class !== s.class && (t.class = Fe([
        t.class,
        s.class
      ]));
      else if (r === "style") t.style = Kt([
        t.style,
        s.style
      ]);
      else if (Ss(r)) {
        const i = t[r], a = s[r];
        a && i !== a && !(G(i) && i.includes(a)) ? t[r] = i ? [].concat(i, a) : a : a == null && i == null && !ks(r) && (t[r] = a);
      } else r !== "" && (t[r] = s[r]);
    }
    return t;
  }
  function lt(e, t, n, s = null) {
    tt(e, t, 7, [
      n,
      s
    ]);
  }
  const Fc = ta();
  let qc = 0;
  function Wc(e, t, n) {
    const s = e.type, r = (t ? t.appContext : e.appContext) || Fc, i = {
      uid: qc++,
      vnode: e,
      type: s,
      parent: t,
      appContext: r,
      root: null,
      next: null,
      subTree: null,
      effect: null,
      update: null,
      job: null,
      scope: new hl(true),
      render: null,
      proxy: null,
      exposed: null,
      exposeProxy: null,
      withProxy: null,
      provides: t ? t.provides : Object.create(r.provides),
      ids: t ? t.ids : [
        "",
        0,
        0
      ],
      accessCache: null,
      renderCache: [],
      components: null,
      directives: null,
      propsOptions: la(s, r),
      emitsOptions: na(s, r),
      emit: null,
      emitted: null,
      propsDefaults: fe,
      inheritAttrs: s.inheritAttrs,
      ctx: fe,
      data: fe,
      props: fe,
      attrs: fe,
      slots: fe,
      refs: fe,
      setupState: fe,
      setupContext: null,
      suspense: n,
      suspenseId: n ? n.pendingId : 0,
      asyncDep: null,
      asyncResolved: false,
      isMounted: false,
      isUnmounted: false,
      isDeactivated: false,
      bc: null,
      c: null,
      bm: null,
      m: null,
      bu: null,
      u: null,
      um: null,
      bum: null,
      da: null,
      a: null,
      rtg: null,
      rtc: null,
      ec: null,
      sp: null
    };
    return i.ctx = {
      _: i
    }, i.root = t ? t.root : i, i.emit = _c.bind(null, i), e.ce && e.ce(i), i;
  }
  let Ce = null;
  const Vc = () => Ce || We;
  let ms, fr;
  {
    const e = As(), t = (n, s) => {
      let r;
      return (r = e[n]) || (r = e[n] = []), r.push(s), (i) => {
        r.length > 1 ? r.forEach((a) => a(i)) : r[0](i);
      };
    };
    ms = t("__VUE_INSTANCE_SETTERS__", (n) => Ce = n), fr = t("__VUE_SSR_SETTERS__", (n) => Wn = n);
  }
  const Jn = (e) => {
    const t = Ce;
    return ms(e), e.scope.on(), () => {
      e.scope.off(), ms(t);
    };
  }, vi = () => {
    Ce && Ce.scope.off(), ms(null);
  };
  function ya(e) {
    return e.vnode.shapeFlag & 4;
  }
  let Wn = false;
  function Bc(e, t = false, n = false) {
    t && fr(t);
    const { props: s, children: r } = e.vnode, i = ya(e);
    Ac(e, s, i, t), Pc(e, r, n || t);
    const a = i ? Kc(e, t) : void 0;
    return t && fr(false), a;
  }
  function Kc(e, t) {
    const n = e.type;
    e.accessCache = /* @__PURE__ */ Object.create(null), e.proxy = new Proxy(e.ctx, uc);
    const { setup: s } = n;
    if (s) {
      mt();
      const r = e.setupContext = s.length > 1 ? Jc(e) : null, i = Jn(e), a = Gn(s, e, 0, [
        e.props,
        r
      ]), o = vo(a);
      if (vt(), i(), (o || e.sp) && !Mn(e) && zo(e), o) {
        if (a.then(vi, vi), t) return a.then((l) => {
          yi(e, l);
        }).catch((l) => {
          Cs(l, e, 0);
        });
        e.asyncDep = a;
      } else yi(e, a);
    } else _a(e);
  }
  function yi(e, t, n) {
    Y(t) ? e.type.__ssrInlineRender ? e.ssrRender = t : e.render = t : le(t) && (e.setupState = Fo(t)), _a(e);
  }
  function _a(e, t, n) {
    const s = e.type;
    e.render || (e.render = s.render || ht);
    {
      const r = Jn(e);
      mt();
      try {
        fc(e);
      } finally {
        vt(), r();
      }
    }
  }
  const Gc = {
    get(e, t) {
      return Ae(e, "get", ""), e[t];
    }
  };
  function Jc(e) {
    const t = (n) => {
      e.exposed = n || {};
    };
    return {
      attrs: new Proxy(e.attrs, Gc),
      slots: e.slots,
      emit: e.emit,
      expose: t
    };
  }
  function Is(e) {
    return e.exposed ? e.exposeProxy || (e.exposeProxy = new Proxy(Fo(Ol(e.exposed)), {
      get(t, n) {
        if (n in t) return t[n];
        if (n in jn) return jn[n](e);
      },
      has(t, n) {
        return n in t || n in jn;
      }
    })) : e.proxy;
  }
  function zc(e) {
    return Y(e) && "__vccOpts" in e;
  }
  const Q = (e, t) => Dl(e, t, Wn);
  function ba(e, t, n) {
    try {
      hs(-1);
      const s = arguments.length;
      return s === 2 ? le(t) && !G(t) ? ps(t) ? ke(e, null, [
        t
      ]) : ke(e, t) : ke(e, null, t) : (s > 3 ? n = Array.prototype.slice.call(arguments, 2) : s === 3 && ps(n) && (n = [
        n
      ]), ke(e, t, n));
    } finally {
      hs(1);
    }
  }
  const Yc = "3.5.39";
  let dr;
  const _i = typeof window < "u" && window.trustedTypes;
  if (_i) try {
    dr = _i.createPolicy("vue", {
      createHTML: (e) => e
    });
  } catch {
  }
  const wa = dr ? (e) => dr.createHTML(e) : (e) => e, Qc = "http://www.w3.org/2000/svg", Xc = "http://www.w3.org/1998/Math/MathML", St = typeof document < "u" ? document : null, bi = St && St.createElement("template"), Zc = {
    insert: (e, t, n) => {
      t.insertBefore(e, n || null);
    },
    remove: (e) => {
      const t = e.parentNode;
      t && t.removeChild(e);
    },
    createElement: (e, t, n, s) => {
      const r = t === "svg" ? St.createElementNS(Qc, e) : t === "mathml" ? St.createElementNS(Xc, e) : n ? St.createElement(e, {
        is: n
      }) : St.createElement(e);
      return e === "select" && s && s.multiple != null && r.setAttribute("multiple", s.multiple), r;
    },
    createText: (e) => St.createTextNode(e),
    createComment: (e) => St.createComment(e),
    setText: (e, t) => {
      e.nodeValue = t;
    },
    setElementText: (e, t) => {
      e.textContent = t;
    },
    parentNode: (e) => e.parentNode,
    nextSibling: (e) => e.nextSibling,
    querySelector: (e) => St.querySelector(e),
    setScopeId(e, t) {
      e.setAttribute(t, "");
    },
    insertStaticContent(e, t, n, s, r, i) {
      const a = n ? n.previousSibling : t.lastChild;
      if (r && (r === i || r.nextSibling)) for (; t.insertBefore(r.cloneNode(true), n), !(r === i || !(r = r.nextSibling)); ) ;
      else {
        bi.innerHTML = wa(s === "svg" ? `<svg>${e}</svg>` : s === "mathml" ? `<math>${e}</math>` : e);
        const o = bi.content;
        if (s === "svg" || s === "mathml") {
          const l = o.firstChild;
          for (; l.firstChild; ) o.appendChild(l.firstChild);
          o.removeChild(l);
        }
        t.insertBefore(o, n);
      }
      return [
        a ? a.nextSibling : t.firstChild,
        n ? n.previousSibling : t.lastChild
      ];
    }
  }, eu = Symbol("_vtc");
  function tu(e, t, n) {
    const s = e[eu];
    s && (t = (t ? [
      t,
      ...s
    ] : [
      ...s
    ]).join(" ")), t == null ? e.removeAttribute("class") : n ? e.setAttribute("class", t) : e.className = t;
  }
  const wi = Symbol("_vod"), nu = Symbol("_vsh"), su = Symbol(""), ru = /(?:^|;)\s*display\s*:/;
  function iu(e, t, n) {
    const s = e.style, r = ve(n);
    let i = false;
    if (n && !r) {
      if (t) if (ve(t)) for (const a of t.split(";")) {
        const o = a.slice(0, a.indexOf(":")).trim();
        n[o] == null && En(s, o, "");
      }
      else for (const a in t) n[a] == null && En(s, a, "");
      for (const a in n) {
        a === "display" && (i = true);
        const o = n[a];
        o != null ? au(e, a, !ve(t) && t ? t[a] : void 0, o) || En(s, a, o) : En(s, a, "");
      }
    } else if (r) {
      if (t !== n) {
        const a = s[su];
        a && (n += ";" + a), s.cssText = n, i = ru.test(n);
      }
    } else t && e.removeAttribute("style");
    wi in e && (e[wi] = i ? s.display : "", e[nu] && (s.display = "none"));
  }
  const Si = /\s*!important$/;
  function En(e, t, n) {
    if (G(n)) n.forEach((s) => En(e, t, s));
    else if (n == null && (n = ""), t.startsWith("--")) e.setProperty(t, n);
    else {
      const s = ou(e, t);
      Si.test(n) ? e.setProperty(jt(s), n.replace(Si, ""), "important") : e[s] = n;
    }
  }
  const ki = [
    "Webkit",
    "Moz",
    "ms"
  ], Ks = {};
  function ou(e, t) {
    const n = Ks[t];
    if (n) return n;
    let s = Be(t);
    if (s !== "filter" && s in e) return Ks[t] = s;
    s = Tr(s);
    for (let r = 0; r < ki.length; r++) {
      const i = ki[r] + s;
      if (i in e) return Ks[t] = i;
    }
    return t;
  }
  function au(e, t, n, s) {
    return e.tagName === "TEXTAREA" && (t === "width" || t === "height") && ve(s) && n === s;
  }
  const Ei = "http://www.w3.org/1999/xlink";
  function Ai(e, t, n, s, r, i = fl(t)) {
    s && t.startsWith("xlink:") ? n == null ? e.removeAttributeNS(Ei, t.slice(6, t.length)) : e.setAttributeNS(Ei, t, n) : n == null || i && !So(n) ? e.removeAttribute(t) : e.setAttribute(t, i ? "" : gt(n) ? String(n) : n);
  }
  function xi(e, t, n, s, r) {
    if (t === "innerHTML" || t === "textContent") {
      n != null && (e[t] = t === "innerHTML" ? wa(n) : n);
      return;
    }
    const i = e.tagName;
    if (t === "value" && i !== "PROGRESS" && !i.includes("-")) {
      const o = i === "OPTION" ? e.getAttribute("value") || "" : e.value, l = n == null ? e.type === "checkbox" ? "on" : "" : String(n);
      (o !== l || !("_value" in e)) && (e.value = l), n == null && e.removeAttribute(t), e._value = n;
      return;
    }
    let a = false;
    if (n === "" || n == null) {
      const o = typeof e[t];
      o === "boolean" ? n = So(n) : n == null && o === "string" ? (n = "", a = true) : o === "number" && (n = 0, a = true);
    }
    try {
      e[t] = n;
    } catch {
    }
    a && e.removeAttribute(r || t);
  }
  function Br(e, t, n, s) {
    e.addEventListener(t, n, s);
  }
  function lu(e, t, n, s) {
    e.removeEventListener(t, n, s);
  }
  const Ci = Symbol("_vei");
  function cu(e, t, n, s, r = null) {
    const i = e[Ci] || (e[Ci] = {}), a = i[t];
    if (s && a) a.value = s;
    else {
      const [o, l] = du(t);
      if (s) {
        const c = i[t] = gu(s, r);
        Br(e, o, c, l);
      } else a && (lu(e, o, a, l), i[t] = void 0);
    }
  }
  const uu = /(Once|Passive|Capture)$/, fu = /^on:?(?:Once|Passive|Capture)$/;
  function du(e) {
    let t, n;
    for (; (n = e.match(uu)) && !fu.test(e); ) t || (t = {}), e = e.slice(0, e.length - n[1].length), t[n[1].toLowerCase()] = true;
    return [
      e[2] === ":" ? e.slice(3) : jt(e.slice(2)),
      t
    ];
  }
  let Gs = 0;
  const hu = Promise.resolve(), pu = () => Gs || (hu.then(() => Gs = 0), Gs = Date.now());
  function gu(e, t) {
    const n = (s) => {
      if (!s._vts) s._vts = Date.now();
      else if (s._vts <= n.attached) return;
      const r = n.value;
      if (G(r)) {
        const i = s.stopImmediatePropagation;
        s.stopImmediatePropagation = () => {
          i.call(s), s._stopped = true;
        };
        const a = r.slice(), o = [
          s
        ];
        for (let l = 0; l < a.length && !s._stopped; l++) {
          const c = a[l];
          c && tt(c, t, 5, o);
        }
      } else tt(r, t, 5, [
        s
      ]);
    };
    return n.value = e, n.attached = pu(), n;
  }
  const Ri = (e) => e.charCodeAt(0) === 111 && e.charCodeAt(1) === 110 && e.charCodeAt(2) > 96 && e.charCodeAt(2) < 123, mu = (e, t, n, s, r, i) => {
    const a = r === "svg";
    t === "class" ? tu(e, s, a) : t === "style" ? iu(e, n, s) : Ss(t) ? ks(t) || cu(e, t, n, s, i) : (t[0] === "." ? (t = t.slice(1), true) : t[0] === "^" ? (t = t.slice(1), false) : vu(e, t, s, a)) ? (xi(e, t, s), !e.tagName.includes("-") && (t === "value" || t === "checked" || t === "selected") && Ai(e, t, s, a, i, t !== "value")) : e._isVueCE && (yu(e, t) || e._def.__asyncLoader && (/[A-Z]/.test(t) || !ve(s))) ? xi(e, Be(t), s, i, t) : (t === "true-value" ? e._trueValue = s : t === "false-value" && (e._falseValue = s), Ai(e, t, s, a));
  };
  function vu(e, t, n, s) {
    if (s) return !!(t === "innerHTML" || t === "textContent" || t in e && Ri(t) && Y(n));
    if (t === "spellcheck" || t === "draggable" || t === "translate" || t === "autocorrect" || t === "sandbox" && e.tagName === "IFRAME" || t === "form" || t === "list" && e.tagName === "INPUT" || t === "type" && e.tagName === "TEXTAREA") return false;
    if (t === "width" || t === "height") {
      const r = e.tagName;
      if (r === "IMG" || r === "VIDEO" || r === "CANVAS" || r === "SOURCE") return false;
    }
    return Ri(t) && ve(n) ? false : t in e;
  }
  function yu(e, t) {
    const n = e._def.props;
    if (!n) return false;
    const s = Be(t);
    return Array.isArray(n) ? n.some((r) => Be(r) === s) : Object.keys(n).some((r) => Be(r) === s);
  }
  const vs = (e) => {
    const t = e.props["onUpdate:modelValue"] || false;
    return G(t) ? (n) => ss(t, n) : t;
  }, nn = Symbol("_assign"), _u = {
    deep: true,
    created(e, t, n) {
      e[nn] = vs(n), Br(e, "change", () => {
        const s = e._modelValue, r = Vn(e), i = e.checked, a = e[nn];
        if (G(s)) {
          const o = Ir(s, r), l = o !== -1;
          if (i && !l) a(s.concat(r));
          else if (!i && l) {
            const c = [
              ...s
            ];
            c.splice(o, 1), a(c);
          }
        } else if (fn(s)) {
          const o = new Set(s);
          i ? o.add(r) : o.delete(r), a(o);
        } else a(Sa(e, i));
      });
    },
    mounted: Pi,
    beforeUpdate(e, t, n) {
      e[nn] = vs(n), Pi(e, t, n);
    }
  };
  function Pi(e, { value: t, oldValue: n }, s) {
    e._modelValue = t;
    let r;
    if (G(t)) r = Ir(t, s.props.value) > -1;
    else if (fn(t)) r = t.has(s.props.value);
    else {
      if (t === n) return;
      r = dn(t, Sa(e, true));
    }
    e.checked !== r && (e.checked = r);
  }
  const bu = {
    deep: true,
    created(e, { value: t, modifiers: { number: n } }, s) {
      const r = fn(t);
      Br(e, "change", () => {
        const i = Array.prototype.filter.call(e.options, (a) => a.selected).map((a) => n ? wo(Vn(a)) : Vn(a));
        e[nn](e.multiple ? r ? new Set(i) : i : i[0]), e._assigning = true, pn(() => {
          e._assigning = false;
        });
      }), e[nn] = vs(s);
    },
    mounted(e, { value: t }) {
      Ti(e, t);
    },
    beforeUpdate(e, t, n) {
      e[nn] = vs(n);
    },
    updated(e, { value: t }) {
      e._assigning || Ti(e, t);
    }
  };
  function Ti(e, t) {
    const n = e.multiple, s = G(t);
    if (!(n && !s && !fn(t))) {
      for (let r = 0, i = e.options.length; r < i; r++) {
        const a = e.options[r], o = Vn(a);
        if (n) if (s) {
          const l = typeof o;
          l === "string" || l === "number" ? a.selected = t.some((c) => String(c) === String(o)) : a.selected = Ir(t, o) > -1;
        } else a.selected = t.has(o);
        else if (dn(Vn(a), t)) {
          e.selectedIndex !== r && (e.selectedIndex = r);
          return;
        }
      }
      !n && e.selectedIndex !== -1 && (e.selectedIndex = -1);
    }
  }
  function Vn(e) {
    return "_value" in e ? e._value : e.value;
  }
  function Sa(e, t) {
    const n = t ? "_trueValue" : "_falseValue";
    return n in e ? e[n] : t;
  }
  const wu = [
    "ctrl",
    "shift",
    "alt",
    "meta"
  ], Su = {
    stop: (e) => e.stopPropagation(),
    prevent: (e) => e.preventDefault(),
    self: (e) => e.target !== e.currentTarget,
    ctrl: (e) => !e.ctrlKey,
    shift: (e) => !e.shiftKey,
    alt: (e) => !e.altKey,
    meta: (e) => !e.metaKey,
    left: (e) => "button" in e && e.button !== 0,
    middle: (e) => "button" in e && e.button !== 1,
    right: (e) => "button" in e && e.button !== 2,
    exact: (e, t) => wu.some((n) => e[`${n}Key`] && !t.includes(n))
  }, Ii = (e, t) => {
    if (!e) return e;
    const n = e._withMods || (e._withMods = {}), s = t.join(".");
    return n[s] || (n[s] = ((r, ...i) => {
      for (let a = 0; a < t.length; a++) {
        const o = Su[t[a]];
        if (o && o(r, t)) return;
      }
      return e(r, ...i);
    }));
  }, ku = {
    esc: "escape",
    space: " ",
    up: "arrow-up",
    left: "arrow-left",
    right: "arrow-right",
    down: "arrow-down",
    delete: "backspace"
  }, Eu = (e, t) => {
    const n = e._withKeys || (e._withKeys = {}), s = t.join(".");
    return n[s] || (n[s] = ((r) => {
      if (!("key" in r)) return;
      const i = jt(r.key);
      if (t.some((a) => a === i || ku[a] === i)) return e(r);
    }));
  }, Au = Ee({
    patchProp: mu
  }, Zc);
  let Oi;
  function xu() {
    return Oi || (Oi = Ic(Au));
  }
  const Ni = ((...e) => {
    const t = xu().createApp(...e), { mount: n } = t;
    return t.mount = (s) => {
      const r = Ru(s);
      if (!r) return;
      const i = t._component;
      !Y(i) && !i.render && !i.template && (i.template = r.innerHTML), r.nodeType === 1 && (r.textContent = "");
      const a = n(r, false, Cu(r));
      return r instanceof Element && (r.removeAttribute("v-cloak"), r.setAttribute("data-v-app", "")), a;
    }, t;
  });
  function Cu(e) {
    if (e instanceof SVGElement) return "svg";
    if (typeof MathMLElement == "function" && e instanceof MathMLElement) return "mathml";
  }
  function Ru(e) {
    return ve(e) ? document.querySelector(e) : e;
  }
  const Yt = typeof document < "u";
  function ka(e) {
    return typeof e == "object" || "displayName" in e || "props" in e || "__vccOpts" in e;
  }
  function Pu(e) {
    return e.__esModule || e[Symbol.toStringTag] === "Module" || e.default && ka(e.default);
  }
  const pe = Object.assign;
  function Js(e, t) {
    const n = {};
    for (const s in t) {
      const r = t[s];
      n[s] = nt(r) ? r.map(e) : e(r);
    }
    return n;
  }
  const Ln = () => {
  }, nt = Array.isArray;
  function $i(e, t) {
    const n = {};
    for (const s in e) n[s] = s in t ? t[s] : e[s];
    return n;
  }
  const Ea = /#/g, Tu = /&/g, Iu = /\//g, Ou = /=/g, Nu = /\?/g, Aa = /\+/g, $u = /%5B/g, Mu = /%5D/g, xa = /%5E/g, ju = /%60/g, Ca = /%7B/g, Du = /%7C/g, Ra = /%7D/g, Lu = /%20/g;
  function Kr(e) {
    return e == null ? "" : encodeURI("" + e).replace(Du, "|").replace($u, "[").replace(Mu, "]");
  }
  function Uu(e) {
    return Kr(e).replace(Ca, "{").replace(Ra, "}").replace(xa, "^");
  }
  function hr(e) {
    return Kr(e).replace(Aa, "%2B").replace(Lu, "+").replace(Ea, "%23").replace(Tu, "%26").replace(ju, "`").replace(Ca, "{").replace(Ra, "}").replace(xa, "^");
  }
  function Hu(e) {
    return hr(e).replace(Ou, "%3D");
  }
  function Fu(e) {
    return Kr(e).replace(Ea, "%23").replace(Nu, "%3F");
  }
  function qu(e) {
    return Fu(e).replace(Iu, "%2F");
  }
  function Bn(e) {
    if (e == null) return null;
    try {
      return decodeURIComponent("" + e);
    } catch {
    }
    return "" + e;
  }
  const Wu = /\/$/, Vu = (e) => e.replace(Wu, "");
  function zs(e, t, n = "/") {
    let s, r = {}, i = "", a = "";
    const o = t.indexOf("#");
    let l = t.indexOf("?");
    return l = o >= 0 && l > o ? -1 : l, l >= 0 && (s = t.slice(0, l), i = t.slice(l, o > 0 ? o : t.length), r = e(i.slice(1))), o >= 0 && (s = s || t.slice(0, o), a = t.slice(o, t.length)), s = Ju(s ?? t, n), {
      fullPath: s + i + a,
      path: s,
      query: r,
      hash: Bn(a)
    };
  }
  function Bu(e, t) {
    const n = t.query ? e(t.query) : "";
    return t.path + (n && "?") + n + (t.hash || "");
  }
  function Ku(e, t, n) {
    const s = t.matched.length - 1, r = n.matched.length - 1;
    return s > -1 && s === r && an(t.matched[s], n.matched[r]) && Pa(t.params, n.params) && e(t.query) === e(n.query) && t.hash === n.hash;
  }
  function an(e, t) {
    return (e.aliasOf || e) === (t.aliasOf || t);
  }
  function Pa(e, t) {
    if (Object.keys(e).length !== Object.keys(t).length) return false;
    for (var n in e) if (!Gu(e[n], t[n])) return false;
    return true;
  }
  function Gu(e, t) {
    return nt(e) ? Mi(e, t) : nt(t) ? Mi(t, e) : (e == null ? void 0 : e.valueOf()) === (t == null ? void 0 : t.valueOf());
  }
  function Mi(e, t) {
    return nt(t) ? e.length === t.length && e.every((n, s) => n === t[s]) : e.length === 1 && e[0] === t;
  }
  function Ju(e, t) {
    if (e.startsWith("/")) return e;
    if (!e) return t;
    const n = t.split("/"), s = e.split("/"), r = s[s.length - 1];
    (r === ".." || r === ".") && s.push("");
    let i = n.length - 1, a, o;
    for (a = 0; a < s.length; a++) if (o = s[a], o !== ".") if (o === "..") i > 1 && i--;
    else break;
    return n.slice(0, i).join("/") + "/" + s.slice(a).join("/");
  }
  const Pt = {
    path: "/",
    name: void 0,
    params: {},
    query: {},
    hash: "",
    fullPath: "/",
    matched: [],
    meta: {},
    redirectedFrom: void 0
  };
  let pr = (function(e) {
    return e.pop = "pop", e.push = "push", e;
  })({}), ji = (function(e) {
    return e.back = "back", e.forward = "forward", e.unknown = "", e;
  })({});
  const Ys = "";
  function zu(e) {
    if (!e) if (Yt) {
      const t = document.querySelector("base");
      e = t && t.getAttribute("href") || "/", e = e.replace(/^\w+:\/\/[^\/]+/, "");
    } else e = "/";
    return e[0] !== "/" && e[0] !== "#" && (e = "/" + e), Vu(e);
  }
  const Yu = /^[^#]+#/;
  function Qu(e, t) {
    return e.replace(Yu, "#") + t;
  }
  function Xu(e, t) {
    const n = document.documentElement.getBoundingClientRect(), s = e.getBoundingClientRect();
    return {
      behavior: t.behavior,
      left: s.left - n.left - (t.left || 0),
      top: s.top - n.top - (t.top || 0)
    };
  }
  const Zu = () => ({
    left: window.scrollX,
    top: window.scrollY
  });
  function ef(e) {
    let t;
    if ("el" in e) {
      const n = e.el, s = typeof n == "string" && n.startsWith("#"), r = typeof n == "string" ? s ? document.getElementById(n.slice(1)) : document.querySelector(n) : n;
      if (!r) return;
      t = Xu(r, e);
    } else t = e;
    "scrollBehavior" in document.documentElement.style ? window.scrollTo(t) : window.scrollTo(t.left != null ? t.left : window.scrollX, t.top != null ? t.top : window.scrollY);
  }
  function Di(e, t) {
    return (history.state ? history.state.position - t : -1) + e;
  }
  const gr = /* @__PURE__ */ new Map();
  function tf(e, t) {
    gr.set(e, t);
  }
  function nf(e) {
    const t = gr.get(e);
    return gr.delete(e), t;
  }
  function sf(e) {
    return typeof e == "string" || e && typeof e == "object";
  }
  function Ta(e) {
    return typeof e == "string" || typeof e == "symbol";
  }
  let me = (function(e) {
    return e[e.MATCHER_NOT_FOUND = 1] = "MATCHER_NOT_FOUND", e[e.NAVIGATION_GUARD_REDIRECT = 2] = "NAVIGATION_GUARD_REDIRECT", e[e.NAVIGATION_ABORTED = 4] = "NAVIGATION_ABORTED", e[e.NAVIGATION_CANCELLED = 8] = "NAVIGATION_CANCELLED", e[e.NAVIGATION_DUPLICATED = 16] = "NAVIGATION_DUPLICATED", e;
  })({});
  const Ia = Symbol("");
  me.MATCHER_NOT_FOUND + "", me.NAVIGATION_GUARD_REDIRECT + "", me.NAVIGATION_ABORTED + "", me.NAVIGATION_CANCELLED + "", me.NAVIGATION_DUPLICATED + "";
  function ln(e, t) {
    return pe(new Error(), {
      type: e,
      [Ia]: true
    }, t);
  }
  function bt(e, t) {
    return e instanceof Error && Ia in e && (t == null || !!(e.type & t));
  }
  const rf = [
    "params",
    "query",
    "hash"
  ];
  function of(e) {
    if (typeof e == "string") return e;
    if (e.path != null) return e.path;
    const t = {};
    for (const n of rf) n in e && (t[n] = e[n]);
    return JSON.stringify(t, null, 2);
  }
  function af(e) {
    const t = {};
    if (e === "" || e === "?") return t;
    const n = (e[0] === "?" ? e.slice(1) : e).split("&");
    for (let s = 0; s < n.length; ++s) {
      const r = n[s].replace(Aa, " "), i = r.indexOf("="), a = Bn(i < 0 ? r : r.slice(0, i)), o = i < 0 ? null : Bn(r.slice(i + 1));
      if (a in t) {
        let l = t[a];
        nt(l) || (l = t[a] = [
          l
        ]), l.push(o);
      } else t[a] = o;
    }
    return t;
  }
  function Li(e) {
    let t = "";
    for (let n in e) {
      const s = e[n];
      if (n = Hu(n), s == null) {
        s !== void 0 && (t += (t.length ? "&" : "") + n);
        continue;
      }
      (nt(s) ? s.map((r) => r && hr(r)) : [
        s && hr(s)
      ]).forEach((r) => {
        r !== void 0 && (t += (t.length ? "&" : "") + n, r != null && (t += "=" + r));
      });
    }
    return t;
  }
  function lf(e) {
    const t = {};
    for (const n in e) {
      const s = e[n];
      s !== void 0 && (t[n] = nt(s) ? s.map((r) => r == null ? null : "" + r) : s == null ? s : "" + s);
    }
    return t;
  }
  const cf = Symbol(""), Ui = Symbol(""), Os = Symbol(""), Gr = Symbol(""), mr = Symbol("");
  function _n() {
    let e = [];
    function t(s) {
      return e.push(s), () => {
        const r = e.indexOf(s);
        r > -1 && e.splice(r, 1);
      };
    }
    function n() {
      e = [];
    }
    return {
      add: t,
      list: () => e.slice(),
      reset: n
    };
  }
  function Ot(e, t, n, s, r, i = (a) => a()) {
    const a = s && (s.enterCallbacks[r] = s.enterCallbacks[r] || []);
    return () => new Promise((o, l) => {
      const c = (m) => {
        m === false ? l(ln(me.NAVIGATION_ABORTED, {
          from: n,
          to: t
        })) : m instanceof Error ? l(m) : sf(m) ? l(ln(me.NAVIGATION_GUARD_REDIRECT, {
          from: t,
          to: m
        })) : (a && s.enterCallbacks[r] === a && typeof m == "function" && a.push(m), o());
      }, u = i(() => e.call(s && s.instances[r], t, n, c));
      let f = Promise.resolve(u);
      e.length < 3 && (f = f.then(c)), f.catch((m) => l(m));
    });
  }
  function Qs(e, t, n, s, r = (i) => i()) {
    const i = [];
    for (const a of e) for (const o in a.components) {
      let l = a.components[o];
      if (!(t !== "beforeRouteEnter" && !a.instances[o])) if (ka(l)) {
        const c = (l.__vccOpts || l)[t];
        c && i.push(Ot(c, n, s, a, o, r));
      } else {
        let c = l();
        i.push(() => c.then((u) => {
          if (!u) throw new Error(`Couldn't resolve component "${o}" at "${a.path}"`);
          const f = Pu(u) ? u.default : u;
          a.mods[o] = u, a.components[o] = f;
          const m = (f.__vccOpts || f)[t];
          return m && Ot(m, n, s, a, o, r)();
        }));
      }
    }
    return i;
  }
  function uf(e, t) {
    const n = [], s = [], r = [], i = Math.max(t.matched.length, e.matched.length);
    for (let a = 0; a < i; a++) {
      const o = t.matched[a];
      o && (e.matched.find((c) => an(c, o)) ? s.push(o) : n.push(o));
      const l = e.matched[a];
      l && (t.matched.find((c) => an(c, l)) || r.push(l));
    }
    return [
      n,
      s,
      r
    ];
  }
  function ff(e = "") {
    let t = [], n = [
      [
        Ys,
        {}
      ]
    ], s = 0;
    e = zu(e);
    function r(o, l = {}) {
      s++, s !== n.length && n.splice(s), n.push([
        o,
        l
      ]);
    }
    function i(o, l, { direction: c, delta: u }) {
      const f = {
        direction: c,
        delta: u,
        type: pr.pop
      };
      for (const m of t) m(o, l, f);
    }
    const a = {
      location: Ys,
      state: {},
      base: e,
      createHref: Qu.bind(null, e),
      replace(o, l) {
        n.splice(s--, 1), r(o, l);
      },
      push(o, l) {
        r(o, l);
      },
      listen(o) {
        return t.push(o), () => {
          const l = t.indexOf(o);
          l > -1 && t.splice(l, 1);
        };
      },
      destroy() {
        t = [], n = [
          [
            Ys,
            {}
          ]
        ], s = 0;
      },
      go(o, l = true) {
        const c = this.location, u = o < 0 ? ji.back : ji.forward;
        s = Math.max(0, Math.min(s + o, n.length - 1)), l && i(this.location, c, {
          direction: u,
          delta: o
        });
      }
    };
    return Object.defineProperty(a, "location", {
      enumerable: true,
      get: () => n[s][0]
    }), Object.defineProperty(a, "state", {
      enumerable: true,
      get: () => n[s][1]
    }), a;
  }
  let qt = (function(e) {
    return e[e.Static = 0] = "Static", e[e.Param = 1] = "Param", e[e.Group = 2] = "Group", e;
  })({});
  var be = (function(e) {
    return e[e.Static = 0] = "Static", e[e.Param = 1] = "Param", e[e.ParamRegExp = 2] = "ParamRegExp", e[e.ParamRegExpEnd = 3] = "ParamRegExpEnd", e[e.EscapeNext = 4] = "EscapeNext", e;
  })(be || {});
  const df = {
    type: qt.Static,
    value: ""
  }, hf = /[a-zA-Z0-9_]/;
  function pf(e) {
    if (!e) return [
      []
    ];
    if (e === "/") return [
      [
        df
      ]
    ];
    if (!e.startsWith("/")) throw new Error(`Invalid path "${e}"`);
    function t(y) {
      throw new Error(`ERR (${n})/"${c}": ${y}`);
    }
    let n = be.Static, s = n;
    const r = [];
    let i;
    function a() {
      i && r.push(i), i = [];
    }
    let o = 0, l, c = "", u = "";
    function f() {
      c && (n === be.Static ? i.push({
        type: qt.Static,
        value: c
      }) : n === be.Param || n === be.ParamRegExp || n === be.ParamRegExpEnd ? (i.length > 1 && (l === "*" || l === "+") && t(`A repeatable param (${c}) must be alone in its segment. eg: '/:ids+.`), i.push({
        type: qt.Param,
        value: c,
        regexp: u,
        repeatable: l === "*" || l === "+",
        optional: l === "*" || l === "?"
      })) : t("Invalid state to consume buffer"), c = "");
    }
    function m() {
      c += l;
    }
    for (; o < e.length; ) {
      if (l = e[o++], l === "\\" && n !== be.ParamRegExp) {
        s = n, n = be.EscapeNext;
        continue;
      }
      switch (n) {
        case be.Static:
          l === "/" ? (c && f(), a()) : l === ":" ? (f(), n = be.Param) : m();
          break;
        case be.EscapeNext:
          m(), n = s;
          break;
        case be.Param:
          l === "(" ? n = be.ParamRegExp : hf.test(l) ? m() : (f(), n = be.Static, l !== "*" && l !== "?" && l !== "+" && o--);
          break;
        case be.ParamRegExp:
          l === ")" ? u[u.length - 1] == "\\" ? u = u.slice(0, -1) + l : n = be.ParamRegExpEnd : u += l;
          break;
        case be.ParamRegExpEnd:
          f(), n = be.Static, l !== "*" && l !== "?" && l !== "+" && o--, u = "";
          break;
        default:
          t("Unknown state");
          break;
      }
    }
    return n === be.ParamRegExp && t(`Unfinished custom RegExp for param "${c}"`), f(), a(), r;
  }
  const Hi = "[^/]+?", gf = {
    sensitive: false,
    strict: false,
    start: true,
    end: true
  };
  var Oe = (function(e) {
    return e[e._multiplier = 10] = "_multiplier", e[e.Root = 90] = "Root", e[e.Segment = 40] = "Segment", e[e.SubSegment = 30] = "SubSegment", e[e.Static = 40] = "Static", e[e.Dynamic = 20] = "Dynamic", e[e.BonusCustomRegExp = 10] = "BonusCustomRegExp", e[e.BonusWildcard = -50] = "BonusWildcard", e[e.BonusRepeatable = -20] = "BonusRepeatable", e[e.BonusOptional = -8] = "BonusOptional", e[e.BonusStrict = 0.7000000000000001] = "BonusStrict", e[e.BonusCaseSensitive = 0.25] = "BonusCaseSensitive", e;
  })(Oe || {});
  const mf = /[.+*?^${}()[\]/\\]/g;
  function vf(e, t) {
    const n = pe({}, gf, t), s = [];
    let r = n.start ? "^" : "";
    const i = [];
    for (const c of e) {
      const u = c.length ? [] : [
        Oe.Root
      ];
      n.strict && !c.length && (r += "/");
      for (let f = 0; f < c.length; f++) {
        const m = c[f];
        let y = Oe.Segment + (n.sensitive ? Oe.BonusCaseSensitive : 0);
        if (m.type === qt.Static) f || (r += "/"), r += m.value.replace(mf, "\\$&"), y += Oe.Static;
        else if (m.type === qt.Param) {
          const { value: O, repeatable: T, optional: D, regexp: L } = m;
          i.push({
            name: O,
            repeatable: T,
            optional: D
          });
          const E = L || Hi;
          if (E !== Hi) {
            y += Oe.BonusCustomRegExp;
            try {
              `${E}`;
            } catch ($) {
              throw new Error(`Invalid custom RegExp for param "${O}" (${E}): ` + $.message);
            }
          }
          let H = T ? `((?:${E})(?:/(?:${E}))*)` : `(${E})`;
          f || (H = D && c.length < 2 ? `(?:/${H})` : "/" + H), D && (H += "?"), r += H, y += Oe.Dynamic, D && (y += Oe.BonusOptional), T && (y += Oe.BonusRepeatable), E === ".*" && (y += Oe.BonusWildcard);
        }
        u.push(y);
      }
      s.push(u);
    }
    if (n.strict && n.end) {
      const c = s.length - 1;
      s[c][s[c].length - 1] += Oe.BonusStrict;
    }
    n.strict || (r += "/?"), n.end ? r += "$" : n.strict && !r.endsWith("/") && (r += "(?:/|$)");
    const a = new RegExp(r, n.sensitive ? "" : "i");
    function o(c) {
      const u = c.match(a), f = {};
      if (!u) return null;
      for (let m = 1; m < u.length; m++) {
        const y = u[m] || "", O = i[m - 1];
        f[O.name] = y && O.repeatable ? y.split("/") : y;
      }
      return f;
    }
    function l(c) {
      let u = "", f = false;
      for (const m of e) {
        (!f || !u.endsWith("/")) && (u += "/"), f = false;
        for (const y of m) if (y.type === qt.Static) u += y.value;
        else if (y.type === qt.Param) {
          const { value: O, repeatable: T, optional: D } = y, L = O in c ? c[O] : "";
          if (nt(L) && !T) throw new Error(`Provided param "${O}" is an array but it is not repeatable (* or + modifiers)`);
          const E = nt(L) ? L.join("/") : L;
          if (!E) if (D) m.length < 2 && (u.endsWith("/") ? u = u.slice(0, -1) : f = true);
          else throw new Error(`Missing required param "${O}"`);
          u += E;
        }
      }
      return u || "/";
    }
    return {
      re: a,
      score: s,
      keys: i,
      parse: o,
      stringify: l
    };
  }
  function yf(e, t) {
    let n = 0;
    for (; n < e.length && n < t.length; ) {
      const s = t[n] - e[n];
      if (s) return s;
      n++;
    }
    return e.length < t.length ? e.length === 1 && e[0] === Oe.Static + Oe.Segment ? -1 : 1 : e.length > t.length ? t.length === 1 && t[0] === Oe.Static + Oe.Segment ? 1 : -1 : 0;
  }
  function Oa(e, t) {
    let n = 0;
    const s = e.score, r = t.score;
    for (; n < s.length && n < r.length; ) {
      const i = yf(s[n], r[n]);
      if (i) return i;
      n++;
    }
    if (Math.abs(r.length - s.length) === 1) {
      if (Fi(s)) return 1;
      if (Fi(r)) return -1;
    }
    return r.length - s.length;
  }
  function Fi(e) {
    const t = e[e.length - 1];
    return e.length > 0 && t[t.length - 1] < 0;
  }
  const _f = {
    strict: false,
    end: true,
    sensitive: false
  };
  function bf(e, t, n) {
    const s = vf(pf(e.path), n), r = pe(s, {
      record: e,
      parent: t,
      children: [],
      alias: []
    });
    return t && !r.record.aliasOf == !t.record.aliasOf && t.children.push(r), r;
  }
  function wf(e, t) {
    const n = [], s = /* @__PURE__ */ new Map();
    t = $i(_f, t);
    function r(f) {
      return s.get(f);
    }
    function i(f, m, y) {
      const O = !y, T = Wi(f);
      T.aliasOf = y && y.record;
      const D = $i(t, f), L = [
        T
      ];
      if ("alias" in f) {
        const $ = typeof f.alias == "string" ? [
          f.alias
        ] : f.alias;
        for (const X of $) L.push(Wi(pe({}, T, {
          components: y ? y.record.components : T.components,
          path: X,
          aliasOf: y ? y.record : T
        })));
      }
      let E, H;
      for (const $ of L) {
        const { path: X } = $;
        if (m && X[0] !== "/") {
          const se = m.record.path, re = se[se.length - 1] === "/" ? "" : "/";
          $.path = m.record.path + (X && re + X);
        }
        if (E = bf($, m, D), y ? y.alias.push(E) : (H = H || E, H !== E && H.alias.push(E), O && f.name && !Vi(E) && a(f.name)), Na(E) && l(E), T.children) {
          const se = T.children;
          for (let re = 0; re < se.length; re++) i(se[re], E, y && y.children[re]);
        }
        y = y || E;
      }
      return H ? () => {
        a(H);
      } : Ln;
    }
    function a(f) {
      if (Ta(f)) {
        const m = s.get(f);
        m && (s.delete(f), n.splice(n.indexOf(m), 1), m.children.forEach(a), m.alias.forEach(a));
      } else {
        const m = n.indexOf(f);
        m > -1 && (n.splice(m, 1), f.record.name && s.delete(f.record.name), f.children.forEach(a), f.alias.forEach(a));
      }
    }
    function o() {
      return n;
    }
    function l(f) {
      const m = Ef(f, n);
      n.splice(m, 0, f), f.record.name && !Vi(f) && s.set(f.record.name, f);
    }
    function c(f, m) {
      let y, O = {}, T, D;
      if ("name" in f && f.name) {
        if (y = s.get(f.name), !y) throw ln(me.MATCHER_NOT_FOUND, {
          location: f
        });
        D = y.record.name, O = pe(qi(m.params, y.keys.filter((H) => !H.optional).concat(y.parent ? y.parent.keys.filter((H) => H.optional) : []).map((H) => H.name)), f.params && qi(f.params, y.keys.map((H) => H.name))), T = y.stringify(O);
      } else if (f.path != null) T = f.path, y = n.find((H) => H.re.test(T)), y && (O = y.parse(T), D = y.record.name);
      else {
        if (y = m.name ? s.get(m.name) : n.find((H) => H.re.test(m.path)), !y) throw ln(me.MATCHER_NOT_FOUND, {
          location: f,
          currentLocation: m
        });
        D = y.record.name, O = pe({}, m.params, f.params), T = y.stringify(O);
      }
      const L = [];
      let E = y;
      for (; E; ) L.unshift(E.record), E = E.parent;
      return {
        name: D,
        path: T,
        params: O,
        matched: L,
        meta: kf(L)
      };
    }
    e.forEach((f) => i(f));
    function u() {
      n.length = 0, s.clear();
    }
    return {
      addRoute: i,
      resolve: c,
      removeRoute: a,
      clearRoutes: u,
      getRoutes: o,
      getRecordMatcher: r
    };
  }
  function qi(e, t) {
    const n = {};
    for (const s of t) s in e && (n[s] = e[s]);
    return n;
  }
  function Wi(e) {
    const t = {
      path: e.path,
      redirect: e.redirect,
      name: e.name,
      meta: e.meta || {},
      aliasOf: e.aliasOf,
      beforeEnter: e.beforeEnter,
      props: Sf(e),
      children: e.children || [],
      instances: {},
      leaveGuards: /* @__PURE__ */ new Set(),
      updateGuards: /* @__PURE__ */ new Set(),
      enterCallbacks: {},
      components: "components" in e ? e.components || null : e.component && {
        default: e.component
      }
    };
    return Object.defineProperty(t, "mods", {
      value: {}
    }), t;
  }
  function Sf(e) {
    const t = {}, n = e.props || false;
    if ("component" in e) t.default = n;
    else for (const s in e.components) t[s] = typeof n == "object" ? n[s] : n;
    return t;
  }
  function Vi(e) {
    for (; e; ) {
      if (e.record.aliasOf) return true;
      e = e.parent;
    }
    return false;
  }
  function kf(e) {
    return e.reduce((t, n) => pe(t, n.meta), {});
  }
  function Ef(e, t) {
    let n = 0, s = t.length;
    for (; n !== s; ) {
      const i = n + s >> 1;
      Oa(e, t[i]) < 0 ? s = i : n = i + 1;
    }
    const r = Af(e);
    return r && (s = t.lastIndexOf(r, s - 1)), s;
  }
  function Af(e) {
    let t = e;
    for (; t = t.parent; ) if (Na(t) && Oa(e, t) === 0) return t;
  }
  function Na({ record: e }) {
    return !!(e.name || e.components && Object.keys(e.components).length || e.redirect);
  }
  function Bi(e) {
    const t = Ze(Os), n = Ze(Gr), s = Q(() => {
      const l = W(e.to);
      return t.resolve(l);
    }), r = Q(() => {
      const { matched: l } = s.value, { length: c } = l, u = l[c - 1], f = n.matched;
      if (!u || !f.length) return -1;
      const m = f.findIndex(an.bind(null, u));
      if (m > -1) return m;
      const y = Ki(l[c - 2]);
      return c > 1 && Ki(u) === y && f[f.length - 1].path !== y ? f.findIndex(an.bind(null, l[c - 2])) : m;
    }), i = Q(() => r.value > -1 && Tf(n.params, s.value.params)), a = Q(() => r.value > -1 && r.value === n.matched.length - 1 && Pa(n.params, s.value.params));
    function o(l = {}) {
      if (Pf(l)) {
        const c = t[W(e.replace) ? "replace" : "push"](W(e.to)).catch(Ln);
        return e.viewTransition && typeof document < "u" && "startViewTransition" in document && document.startViewTransition(() => c), c;
      }
      return Promise.resolve();
    }
    return {
      route: s,
      href: Q(() => s.value.href),
      isActive: i,
      isExactActive: a,
      navigate: o
    };
  }
  function xf(e) {
    return e.length === 1 ? e[0] : e;
  }
  const Cf = Dt({
    name: "RouterLink",
    compatConfig: {
      MODE: 3
    },
    props: {
      to: {
        type: [
          String,
          Object
        ],
        required: true
      },
      replace: Boolean,
      activeClass: String,
      exactActiveClass: String,
      custom: Boolean,
      ariaCurrentValue: {
        type: String,
        default: "page"
      },
      viewTransition: Boolean
    },
    useLink: Bi,
    setup(e, { slots: t }) {
      const n = hn(Bi(e)), { options: s } = Ze(Os), r = Q(() => ({
        [Gi(e.activeClass, s.linkActiveClass, "router-link-active")]: n.isActive,
        [Gi(e.exactActiveClass, s.linkExactActiveClass, "router-link-exact-active")]: n.isExactActive
      }));
      return () => {
        const i = t.default && xf(t.default(n));
        return e.custom ? i : ba("a", {
          "aria-current": n.isExactActive ? e.ariaCurrentValue : null,
          href: n.href,
          onClick: n.navigate,
          class: r.value
        }, i);
      };
    }
  }), Rf = Cf;
  function Pf(e) {
    if (!(e.metaKey || e.altKey || e.ctrlKey || e.shiftKey) && !e.defaultPrevented && !(e.button !== void 0 && e.button !== 0)) {
      if (e.currentTarget && e.currentTarget.getAttribute) {
        const t = e.currentTarget.getAttribute("target");
        if (/\b_blank\b/i.test(t)) return;
      }
      return e.preventDefault && e.preventDefault(), true;
    }
  }
  function Tf(e, t) {
    for (const n in t) {
      const s = t[n], r = e[n];
      if (typeof s == "string") {
        if (s !== r) return false;
      } else if (!nt(r) || r.length !== s.length || s.some((i, a) => i.valueOf() !== r[a].valueOf())) return false;
    }
    return true;
  }
  function Ki(e) {
    return e ? e.aliasOf ? e.aliasOf.path : e.path : "";
  }
  const Gi = (e, t, n) => e ?? t ?? n, If = Dt({
    name: "RouterView",
    inheritAttrs: false,
    props: {
      name: {
        type: String,
        default: "default"
      },
      route: Object
    },
    compatConfig: {
      MODE: 3
    },
    setup(e, { attrs: t, slots: n }) {
      const s = Ze(mr), r = Q(() => e.route || s.value), i = Ze(Ui, 0), a = Q(() => {
        let c = W(i);
        const { matched: u } = r.value;
        let f;
        for (; (f = u[c]) && !f.components; ) c++;
        return c;
      }), o = Q(() => r.value.matched[a.value]);
      rs(Ui, Q(() => a.value + 1)), rs(cf, o), rs(mr, r);
      const l = ye();
      return Bt(() => [
        l.value,
        o.value,
        e.name
      ], ([c, u, f], [m, y, O]) => {
        u && (u.instances[f] = c, y && y !== u && c && c === m && (u.leaveGuards.size || (u.leaveGuards = y.leaveGuards), u.updateGuards.size || (u.updateGuards = y.updateGuards))), c && u && (!y || !an(u, y) || !m) && (u.enterCallbacks[f] || []).forEach((T) => T(c));
      }, {
        flush: "post"
      }), () => {
        const c = r.value, u = e.name, f = o.value, m = f && f.components[u];
        if (!m) return Ji(n.default, {
          Component: m,
          route: c
        });
        const y = f.props[u], O = y ? y === true ? c.params : typeof y == "function" ? y(c) : y : null, D = ba(m, pe({}, O, t, {
          onVnodeUnmounted: (L) => {
            L.component.isUnmounted && (f.instances[u] = null);
          },
          ref: l
        }));
        return Ji(n.default, {
          Component: D,
          route: c
        }) || D;
      };
    }
  });
  function Ji(e, t) {
    if (!e) return null;
    const n = e(t);
    return n.length === 1 ? n[0] : n;
  }
  const Of = If;
  function Nf(e) {
    const t = wf(e.routes, e), n = e.parseQuery || af, s = e.stringifyQuery || Li, r = e.history, i = _n(), a = _n(), o = _n(), l = Nl(Pt);
    let c = Pt;
    Yt && e.scrollBehavior && "scrollRestoration" in history && (history.scrollRestoration = "manual");
    const u = Js.bind(null, (S) => "" + S), f = Js.bind(null, qu), m = Js.bind(null, Bn);
    function y(S, U) {
      let N, q;
      return Ta(S) ? (N = t.getRecordMatcher(S), q = U) : q = S, t.addRoute(q, N);
    }
    function O(S) {
      const U = t.getRecordMatcher(S);
      U && t.removeRoute(U);
    }
    function T() {
      return t.getRoutes().map((S) => S.record);
    }
    function D(S) {
      return !!t.getRecordMatcher(S);
    }
    function L(S, U) {
      if (U = pe({}, U || l.value), typeof S == "string") {
        const _ = zs(n, S, U.path), b = t.resolve({
          path: _.path
        }, U), g = r.createHref(_.fullPath);
        return pe(_, b, {
          params: m(b.params),
          hash: Bn(_.hash),
          redirectedFrom: void 0,
          href: g
        });
      }
      let N;
      if (S.path != null) N = pe({}, S, {
        path: zs(n, S.path, U.path).path
      });
      else {
        const _ = pe({}, S.params);
        for (const b in _) _[b] == null && delete _[b];
        N = pe({}, S, {
          params: f(_)
        }), U.params = f(U.params);
      }
      const q = t.resolve(N, U), Z = S.hash || "";
      q.params = u(m(q.params));
      const d = Bu(s, pe({}, S, {
        hash: Uu(Z),
        path: q.path
      })), h = r.createHref(d);
      return pe({
        fullPath: d,
        hash: Z,
        query: s === Li ? lf(S.query) : S.query || {}
      }, q, {
        redirectedFrom: void 0,
        href: h
      });
    }
    function E(S) {
      return typeof S == "string" ? zs(n, S, l.value.path) : pe({}, S);
    }
    function H(S, U) {
      if (c !== S) return ln(me.NAVIGATION_CANCELLED, {
        from: U,
        to: S
      });
    }
    function $(S) {
      return re(S);
    }
    function X(S) {
      return $(pe(E(S), {
        replace: true
      }));
    }
    function se(S, U) {
      const N = S.matched[S.matched.length - 1];
      if (N && N.redirect) {
        const { redirect: q } = N;
        let Z = typeof q == "function" ? q(S, U) : q;
        return typeof Z == "string" && (Z = Z.includes("?") || Z.includes("#") ? Z = E(Z) : {
          path: Z
        }, Z.params = {}), pe({
          query: S.query,
          hash: S.hash,
          params: Z.path != null ? {} : S.params
        }, Z);
      }
    }
    function re(S, U) {
      const N = c = L(S), q = l.value, Z = S.state, d = S.force, h = S.replace === true, _ = se(N, q);
      if (_) return re(pe(E(_), {
        state: typeof _ == "object" ? pe({}, Z, _.state) : Z,
        force: d,
        replace: h
      }), U || N);
      const b = N;
      b.redirectedFrom = U;
      let g;
      return !d && Ku(s, q, N) && (g = ln(me.NAVIGATION_DUPLICATED, {
        to: b,
        from: q
      }), M(q, q, true, false)), (g ? Promise.resolve(g) : je(b, q)).catch((p) => bt(p) ? bt(p, me.NAVIGATION_GUARD_REDIRECT) ? p : I(p) : ee(p, b, q)).then((p) => {
        if (p) {
          if (bt(p, me.NAVIGATION_GUARD_REDIRECT)) return re(pe({
            replace: h
          }, E(p.to), {
            state: typeof p.to == "object" ? pe({}, Z, p.to.state) : Z,
            force: d
          }), U || b);
        } else p = Ye(b, q, true, h, Z);
        return De(b, q, p), p;
      });
    }
    function $e(S, U) {
      const N = H(S, U);
      return N ? Promise.reject(N) : Promise.resolve();
    }
    function Me(S) {
      const U = Te.values().next().value;
      return U && typeof U.runWithContext == "function" ? U.runWithContext(S) : S();
    }
    function je(S, U) {
      let N;
      const [q, Z, d] = uf(S, U);
      N = Qs(q.reverse(), "beforeRouteLeave", S, U);
      for (const _ of q) _.leaveGuards.forEach((b) => {
        N.push(Ot(b, S, U));
      });
      const h = $e.bind(null, S, U);
      return N.push(h), Ue(N).then(() => {
        N = [];
        for (const _ of i.list()) N.push(Ot(_, S, U));
        return N.push(h), Ue(N);
      }).then(() => {
        N = Qs(Z, "beforeRouteUpdate", S, U);
        for (const _ of Z) _.updateGuards.forEach((b) => {
          N.push(Ot(b, S, U));
        });
        return N.push(h), Ue(N);
      }).then(() => {
        N = [];
        for (const _ of d) if (_.beforeEnter) if (nt(_.beforeEnter)) for (const b of _.beforeEnter) N.push(Ot(b, S, U));
        else N.push(Ot(_.beforeEnter, S, U));
        return N.push(h), Ue(N);
      }).then(() => (S.matched.forEach((_) => _.enterCallbacks = {}), N = Qs(d, "beforeRouteEnter", S, U, Me), N.push(h), Ue(N))).then(() => {
        N = [];
        for (const _ of a.list()) N.push(Ot(_, S, U));
        return N.push(h), Ue(N);
      }).catch((_) => bt(_, me.NAVIGATION_CANCELLED) ? _ : Promise.reject(_));
    }
    function De(S, U, N) {
      o.list().forEach((q) => Me(() => q(S, U, N)));
    }
    function Ye(S, U, N, q, Z) {
      const d = H(S, U);
      if (d) return d;
      const h = U === Pt, _ = Yt ? history.state : {};
      N && (q || h ? r.replace(S.fullPath, pe({
        scroll: h && _ && _.scroll
      }, Z)) : r.push(S.fullPath, Z)), l.value = S, M(S, U, N, h), I();
    }
    let Le;
    function yt() {
      Le || (Le = r.listen((S, U, N) => {
        if (!rt.listening) return;
        const q = L(S), Z = se(q, rt.currentRoute.value);
        if (Z) {
          re(pe(Z, {
            replace: true,
            force: true
          }), q).catch(Ln);
          return;
        }
        c = q;
        const d = l.value;
        Yt && tf(Di(d.fullPath, N.delta), Zu()), je(q, d).catch((h) => bt(h, me.NAVIGATION_ABORTED | me.NAVIGATION_CANCELLED) ? h : bt(h, me.NAVIGATION_GUARD_REDIRECT) ? (re(pe(E(h.to), {
          force: true
        }), q).then((_) => {
          bt(_, me.NAVIGATION_ABORTED | me.NAVIGATION_DUPLICATED) && !N.delta && N.type === pr.pop && r.go(-1, false);
        }).catch(Ln), Promise.reject()) : (N.delta && r.go(-N.delta, false), ee(h, q, d))).then((h) => {
          h = h || Ye(q, d, false), h && (N.delta && !bt(h, me.NAVIGATION_CANCELLED) ? r.go(-N.delta, false) : N.type === pr.pop && bt(h, me.NAVIGATION_ABORTED | me.NAVIGATION_DUPLICATED) && r.go(-1, false)), De(q, d, h);
        }).catch(Ln);
      }));
    }
    let st = _n(), ge = _n(), te;
    function ee(S, U, N) {
      I(S);
      const q = ge.list();
      return q.length ? q.forEach((Z) => Z(S, U, N)) : console.error(S), Promise.reject(S);
    }
    function Pe() {
      return te && l.value !== Pt ? Promise.resolve() : new Promise((S, U) => {
        st.add([
          S,
          U
        ]);
      });
    }
    function I(S) {
      return te || (te = !S, yt(), st.list().forEach(([U, N]) => S ? N(S) : U()), st.reset()), S;
    }
    function M(S, U, N, q) {
      const { scrollBehavior: Z } = e;
      if (!Yt || !Z) return Promise.resolve();
      const d = !N && nf(Di(S.fullPath, 0)) || (q || !N) && history.state && history.state.scroll || null;
      return pn().then(() => Z(S, U, d)).then((h) => h && ef(h)).catch((h) => ee(h, S, U));
    }
    const F = (S) => r.go(S);
    let ie;
    const Te = /* @__PURE__ */ new Set(), rt = {
      currentRoute: l,
      listening: true,
      addRoute: y,
      removeRoute: O,
      clearRoutes: t.clearRoutes,
      hasRoute: D,
      getRoutes: T,
      resolve: L,
      options: e,
      push: $,
      replace: X,
      go: F,
      back: () => F(-1),
      forward: () => F(1),
      beforeEach: i.add,
      beforeResolve: a.add,
      afterEach: o.add,
      onError: ge.add,
      isReady: Pe,
      install(S) {
        S.component("RouterLink", Rf), S.component("RouterView", Of), S.config.globalProperties.$router = rt, Object.defineProperty(S.config.globalProperties, "$route", {
          enumerable: true,
          get: () => W(l)
        }), Yt && !ie && l.value === Pt && (ie = true, $(r.location).catch((q) => {
        }));
        const U = {};
        for (const q in Pt) Object.defineProperty(U, q, {
          get: () => l.value[q],
          enumerable: true
        });
        S.provide(Os, rt), S.provide(Gr, Uo(U)), S.provide(mr, l);
        const N = S.unmount;
        Te.add(S), S.unmount = function() {
          Te.delete(S), Te.size < 1 && (c = Pt, Le && Le(), Le = null, l.value = Pt, ie = false, te = false), N();
        };
      }
    };
    function Ue(S) {
      return S.reduce((U, N) => U.then(() => Me(N)), Promise.resolve());
    }
    return rt;
  }
  function $f() {
    return Ze(Os);
  }
  function Mf(e) {
    return Ze(Gr);
  }
  const zi = {
    confirm: "Confirm",
    select_cards: "SelectCards",
    select_players: "SelectTargets",
    respond_card: "Custom",
    pick_tag: "Custom"
  };
  function jf(e) {
    if (e === void 0) return [
      null,
      null
    ];
    if (!Array.isArray(e) || e.length !== 2 || !e.every((t) => t === null || typeof t == "object" && !Array.isArray(t) && Number.isInteger(t.def) && t.def >= 0 && t.def <= 4294967295 && typeof t.name == "string" && typeof t.faction == "string" && typeof t.revealed == "boolean")) throw new Error("native: generals metadata invalid");
    return e;
  }
  function Df(e) {
    if (e.prompt != null && typeof e.prompt != "string") throw new Error("native: prompt metadata invalid");
    if (e.native_kind != null && (typeof e.native_kind != "string" || !Object.hasOwn(zi, e.native_kind))) throw new Error("native: native_kind metadata invalid");
    const t = e.native_kind;
    return {
      ...e.prompt == null ? {} : {
        prompt: e.prompt
      },
      ...t == null ? {} : {
        native_kind: t
      },
      kind: t ? zi[t] : e.kind ?? "Custom"
    };
  }
  const Yi = {
    confirm: "\u786E\u8BA4",
    select_cards: "\u9009\u62E9\u5361\u724C",
    select_players: "\u9009\u62E9\u89D2\u8272",
    respond_card: "\u54CD\u5E94\u51FA\u724C",
    pick_tag: "\u9009\u62E9\u9009\u9879",
    Confirm: "\u786E\u8BA4",
    SelectCards: "\u9009\u62E9\u5361\u724C",
    SelectTargets: "\u9009\u62E9\u89D2\u8272",
    Custom: "\u9009\u62E9\u9009\u9879",
    OrderTriggers: "\u6392\u5217\u6280\u80FD",
    DiscardToLimit: "\u5F03\u724C\u81F3\u624B\u724C\u4E0A\u9650"
  }, Qi = {
    trigger_confirm: "\u662F\u5426\u53D1\u52A8\u6280\u80FD\uFF1F",
    respond_shan: "\u8BF7\u6253\u51FA\u95EA",
    respond_sha: "\u8BF7\u6253\u51FA\u6740",
    aoe_response: "\u8BF7\u54CD\u5E94\u7FA4\u4F53\u9526\u56CA",
    dying_save: "\u662F\u5426\u4F7F\u7528\u6843\u6551\u63F4\uFF1F",
    discard_to_limit: "\u8BF7\u5F03\u724C\u81F3\u624B\u724C\u4E0A\u9650",
    tuxi_target: "\u7A81\u88AD\uFF1A\u9009\u62E9\u83B7\u5F97\u624B\u724C\u7684\u89D2\u8272",
    guanxing_arrange: "\u89C2\u661F\uFF1A\u6392\u5217\u724C\u5806\u9876\u7684\u5361\u724C",
    xunxun_arrange: "\u6042\u6042\uFF1A\u9009\u62E9\u83B7\u5F97\u7684\u5361\u724C",
    jijie_give_target: "\u673A\u6377\uFF1A\u9009\u62E9\u83B7\u5F97\u6B64\u724C\u7684\u89D2\u8272",
    gaiming_pick: "\u6539\u547D\uFF1A\u9009\u62E9\u5224\u5B9A\u724C",
    haotianta_pick: "\u660A\u5929\u5854\uFF1A\u9009\u62E9\u5224\u5B9A\u724C",
    liaoyuan_discard: "\u71CE\u539F\uFF1A\u9009\u62E9\u5F03\u7F6E\u7684\u5361\u724C",
    gui_dao_rewrite: "\u9B3C\u9053\uFF1A\u9009\u62E9\u66FF\u6362\u5224\u5B9A\u724C\u7684\u5361\u724C",
    wudian_fuchou_cancel: "\u590D\u4EC7\uFF1A\u662F\u5426\u53D6\u6D88\u76EE\u6807\uFF1F",
    wudian_diyjuntun_recast: "\u5C6F\u7530\uFF1A\u9009\u62E9\u91CD\u94F8\u7684\u88C5\u5907",
    rende_cards: "\u4EC1\u5FB7\uFF1A\u9009\u62E9\u4EA4\u51FA\u7684\u624B\u724C",
    rende_target: "\u4EC1\u5FB7\uFF1A\u9009\u62E9\u83B7\u5F97\u624B\u724C\u7684\u89D2\u8272",
    jieyin_cards: "\u7ED3\u59FB\uFF1A\u9009\u62E9\u5F03\u7F6E\u7684\u624B\u724C",
    jieyin_target: "\u7ED3\u59FB\uFF1A\u9009\u62E9\u76EE\u6807\u89D2\u8272",
    lijian_discard: "\u79BB\u95F4\uFF1A\u9009\u62E9\u5F03\u7F6E\u7684\u5361\u724C",
    lijian_targets: "\u79BB\u95F4\uFF1A\u9009\u62E9\u76EE\u6807\u89D2\u8272",
    jijiang_helper: "\u6FC0\u5C06\uFF1A\u9009\u62E9\u54CD\u5E94\u7684\u89D2\u8272",
    jijiang_respond: "\u6FC0\u5C06\uFF1A\u662F\u5426\u6253\u51FA\u6740\uFF1F",
    jijiang_target: "\u6FC0\u5C06\uFF1A\u9009\u62E9\u6740\u7684\u76EE\u6807",
    liuli_transfer: "\u6D41\u79BB\uFF1A\u9009\u62E9\u8F6C\u79FB\u7684\u76EE\u6807",
    hujia_respond: "\u62A4\u9A7E\uFF1A\u662F\u5426\u6253\u51FA\u95EA\uFF1F",
    qingnang_target: "\u9752\u56CA\uFF1A\u9009\u62E9\u56DE\u590D\u4F53\u529B\u7684\u89D2\u8272",
    qingnang_card: "\u9752\u56CA\uFF1A\u9009\u62E9\u5F03\u7F6E\u7684\u624B\u724C",
    sancai_target: "\u6563\u8D22\uFF1A\u9009\u62E9\u76EE\u6807\u89D2\u8272",
    sancai_card: "\u6563\u8D22\uFF1A\u9009\u62E9\u5361\u724C",
    yishang_target: "\u9057\u6B87\uFF1A\u9009\u62E9\u76EE\u6807\u89D2\u8272",
    zhiheng_select: "\u5236\u8861\uFF1A\u9009\u62E9\u5F03\u7F6E\u7684\u5361\u724C",
    nullify_window: "\u662F\u5426\u4F7F\u7528\u65E0\u61C8\u53EF\u51FB\uFF1F",
    duel_sha: "\u51B3\u6597\uFF1A\u8BF7\u6253\u51FA\u6740",
    jiji_card: "\u6D4E\u75BE\uFF1A\u9009\u62E9\u5361\u724C",
    jiji_target: "\u6D4E\u75BE\uFF1A\u9009\u62E9\u76EE\u6807\u89D2\u8272",
    tian_du_gain: "\u5929\u5992\uFF1A\u662F\u5426\u83B7\u5F97\u5224\u5B9A\u724C\uFF1F",
    host_target_discard_equip: "\u8BF7\u9009\u62E9\u5F03\u7F6E\u7684\u88C5\u5907",
    wuxing_select_card: "\u4E94\u884C\u76D8\uFF1A\u9009\u62E9\u5361\u724C",
    wuxing_select_nature: "\u4E94\u884C\u76D8\uFF1A\u9009\u62E9\u5C5E\u6027",
    d3_jinggou_target: "\u91D1\u94A9\uFF1A\u9009\u62E9\u76EE\u6807\u89D2\u8272",
    wd_sanjian_discard: "\u4E09\u5C16\u4E24\u5203\u5200\uFF1A\u9009\u62E9\u5F03\u7F6E\u7684\u624B\u724C",
    wd_sanjian_target: "\u4E09\u5C16\u4E24\u5203\u5200\uFF1A\u9009\u62E9\u76EE\u6807\u89D2\u8272",
    wd_fudi_target_pick: "\u91DC\u5E95\u62BD\u85AA\uFF1A\u9009\u62E9\u5361\u724C",
    wd_fudi_player_pick: "\u91DC\u5E95\u62BD\u85AA\uFF1A\u9009\u62E9\u5361\u724C",
    wd_fudi_compare: "\u91DC\u5E95\u62BD\u85AA\uFF1A\u9009\u62E9\u62FC\u70B9\u724C",
    wd_touliang_own: "\u5077\u6881\u6362\u67F1\uFF1A\u9009\u62E9\u81EA\u5DF1\u7684\u5361\u724C",
    wd_touliang_target: "\u5077\u6881\u6362\u67F1\uFF1A\u9009\u62E9\u76EE\u6807\u7684\u5361\u724C",
    yc_gean_t1_pick: "\u9694\u5CB8\u89C2\u706B\uFF1A\u9009\u62E9\u5361\u724C",
    yc_gean_t2_pick: "\u9694\u5CB8\u89C2\u706B\uFF1A\u9009\u62E9\u5361\u724C",
    wd_caochuan_branch: "\u8349\u8239\u501F\u7BAD\uFF1A\u9009\u62E9\u5E94\u5BF9\u65B9\u5F0F",
    gj_shujinsan_discard: "\u8212\u7B4B\u6563\uFF1A\u9009\u62E9\u5F03\u7F6E\u7684\u5361\u724C",
    gj_feibiao_discard: "\u98DE\u9556\uFF1A\u9009\u62E9\u5F03\u7F6E\u7684\u5361\u724C",
    gj_jinlianzhu_confirm: "\u91D1\u83B2\u73E0\uFF1A\u662F\u5426\u53D1\u52A8\uFF1F",
    gj_xiajiao_discard: "\u867E\u997A\uFF1A\u9009\u62E9\u5F03\u7F6E\u7684\u5361\u724C",
    gj_luyugeng_cost: "\u9C88\u9C7C\u7FB9\uFF1A\u9009\u62E9\u652F\u4ED8\u7684\u5361\u724C",
    gj_dinvxuanshuang_rider: "\u5E1D\u5973\u7384\u971C\uFF1A\u9009\u62E9\u76EE\u6807\u89D2\u8272",
    hh_dunpai_discard: "\u76FE\u724C\uFF1A\u9009\u62E9\u5F03\u7F6E\u7684\u5361\u724C",
    hh_yuansu_discard: "\u5143\u7D20\uFF1A\u9009\u62E9\u5F03\u7F6E\u7684\u5361\u724C",
    gw_poxiao_branch: "\u7834\u6653\uFF1A\u9009\u62E9\u9009\u9879",
    rvm_choice: "\u8BF7\u9009\u62E9\u6280\u80FD\u9009\u9879",
    rvm_discard_pick: "\u8BF7\u9009\u62E9\u5F03\u7F6E\u7684\u624B\u724C\u6216\u88C5\u5907",
    huanren_pick: "\u6362\u4EBA\uFF1A\u9009\u62E9\u6B66\u5C06",
    d3_moyan_target: "\u9B54\u7130\uFF1A\u9009\u62E9\u76EE\u6807\u89D2\u8272",
    d3_moyan_gain: "\u9B54\u7130\uFF1A\u9009\u62E9\u83B7\u5F97\u7684\u5361\u724C",
    fanjian_target: "\u53CD\u95F4\uFF1A\u9009\u62E9\u76EE\u6807\u89D2\u8272",
    fanjian_suit: "\u53CD\u95F4\uFF1A\u9009\u62E9\u82B1\u8272",
    fanjian_pick: "\u53CD\u95F4\uFF1A\u9009\u62E9\u83B7\u5F97\u7684\u624B\u724C"
  };
  function Jr(e) {
    const t = e.native_kind ?? e.kind ?? "Custom";
    return Object.hasOwn(Yi, t) ? Yi[t] : "\u9009\u62E9\u9009\u9879";
  }
  function vr(e) {
    return e.prompt ? Object.hasOwn(Qi, e.prompt) ? Qi[e.prompt] : `\u9009\u62E9\u63D0\u793A\uFF1A${e.prompt}` : Jr(e);
  }
  function $a(e, t) {
    for (const n of e.seats ?? []) jf(n.generals);
    t && Df(t);
  }
  async function zn(e, t = {}, n) {
    return window.__TAURI_INTERNALS__.invoke(e, t, n);
  }
  function Lf(e, t = "asset") {
    return window.__TAURI_INTERNALS__.convertFileSrc(e, t);
  }
  function Mt() {
    return typeof window < "u" && "__TAURI_INTERNALS__" in window;
  }
  function Xi(e) {
    return Mt() ? zn("set_match_frozen", {
      frozen: e
    }) : Promise.resolve();
  }
  function Uf() {
    return Mt() ? zn("content_bundle") : Promise.resolve(null);
  }
  function Hf(e, t) {
    return Mt() ? zn("content_catalog_page", {
      offset: e,
      limit: t
    }) : Promise.resolve(null);
  }
  async function Ff(e) {
    if (!Mt()) return null;
    const t = await zn("resolve_content_asset", {
      namespace: e.namespace,
      packageId: e.package_id,
      assetId: e.asset_id
    });
    return Lf(t);
  }
  const sn = (e) => new Promise((t) => setTimeout(t, e)), Ns = "\u81EA\u52A8\u4E00\u6B65", qf = "\u8FDE\u7EED\u81EA\u52A8", $s = /终局/;
  let Ma = [], yr = 0;
  function Yn(e) {
    return Array.from(document.querySelectorAll("button")).find((t) => (t.innerText || "").trim() === e) ?? null;
  }
  function Zi() {
    return new Promise((e) => {
      const t = () => {
        const r = Yn(Ns);
        return r ? $s.test(document.body.innerText) ? "finished" : r.disabled ? null : "ready" : "gone";
      }, n = t();
      if (n) {
        e(n);
        return;
      }
      const s = new MutationObserver(() => {
        const r = t();
        r && (s.disconnect(), e(r));
      });
      s.observe(document.body, {
        attributes: true,
        subtree: true,
        childList: true,
        attributeFilter: [
          "disabled"
        ]
      }), setTimeout(() => {
        s.disconnect(), e("timeout");
      }, 15e3);
    });
  }
  function _r() {
    return window.__nonamePerf ?? null;
  }
  function Wf() {
    const e = _r(), t = document.body.innerText.match(/终局[^\n]*/) ?? document.body.innerText.match(/rev \d+[^\n]*/);
    return {
      frames: (e == null ? void 0 : e.frames()) ?? [],
      longTasks: (e == null ? void 0 : e.longTasks()) ?? [],
      eventTimings: (e == null ? void 0 : e.eventTimings()) ?? [],
      interactions: (e == null ? void 0 : e.interactions()) ?? [],
      status: (t == null ? void 0 : t[0]) ?? null,
      ua: navigator.userAgent,
      dpr: window.devicePixelRatio,
      viewport: [
        window.innerWidth,
        window.innerHeight
      ],
      href: location.href,
      engine_transport: globalThis.__engineTransport ?? null,
      memory: null,
      memory_series: []
    };
  }
  async function Vf() {
    const e = performance;
    try {
      if (e.memory) {
        const t = e.memory;
        return {
          used: t.usedJSHeapSize,
          total: t.totalJSHeapSize,
          limit: t.jsHeapSizeLimit,
          api: "performance.memory"
        };
      }
      if (e.measureUserAgentSpecificMemory && crossOriginIsolated) {
        const t = await e.measureUserAgentSpecificMemory();
        return {
          used: t.bytes,
          total: t.bytes,
          limit: 0,
          api: "measureUserAgentSpecificMemory"
        };
      }
    } catch {
    }
    return null;
  }
  async function Bf(e, t) {
    try {
      await fetch(e, {
        method: "POST",
        headers: {
          "Content-Type": "application/json"
        },
        body: JSON.stringify(t)
      });
    } catch {
    }
  }
  async function Kf(e, t) {
    for (let n = 0; n < e; n++) {
      const s = Yn(Ns);
      if (!s || $s.test(document.body.innerText)) return "finished";
      if (s.disabled) {
        if (await Zi() === "finished") return "finished";
        continue;
      }
      if (s.click(), await Zi() === "finished") return "finished";
      await sn(t);
    }
    return "max_steps";
  }
  async function Gf() {
    var _a2;
    const e = Qe("loop") === "1", t = Number(Qe("duration") ?? 0), n = Date.now() + (t > 0 ? t : 6e5), s = () => $s.test(document.body.innerText), r = [], i = performance.memory, a = performance.measureUserAgentSpecificMemory, o = async () => {
      if (i) {
        r.push({
          t_ms: Date.now() - yr,
          bytes: i.usedJSHeapSize
        });
        return;
      }
      if (!(!a || !crossOriginIsolated)) try {
        const l = await a();
        r.push({
          t_ms: Date.now() - yr,
          bytes: l.bytes
        });
      } catch {
      }
    };
    for (; ; ) {
      const l = Yn(qf);
      l && l.click();
      let c = 0;
      for (; Date.now() < n && !s(); ) await sn(1e3), Date.now() - c > 3e4 && (c = Date.now(), await o());
      if (!e || Date.now() >= n) return Ma = r, s() ? "finished" : "auto_timeout";
      (_a2 = Array.from(document.querySelectorAll("button")).find((f) => (f.innerText || "").includes("\u91CD\u65B0\u5F00\u5C40"))) == null ? void 0 : _a2.click(), await sn(1500);
    }
  }
  function Jf(e, t) {
    return new Promise((n) => {
      let s = 0;
      const r = (o) => {
        window.removeEventListener("click", i, true), n(o);
      }, i = (o) => {
        const l = o.target, c = Yn(Ns);
        c && (l === c || c.contains(l)) && (s += 1, s >= e && setTimeout(() => r("manual_steps"), 800));
      };
      window.addEventListener("click", i, true);
      const a = setInterval(() => {
        $s.test(document.body.innerText) && (clearInterval(a), r("finished"));
      }, 500);
      setTimeout(() => {
        clearInterval(a), r(s > 0 ? "manual_timeout" : "manual_no_input");
      }, t);
    });
  }
  function Qe(e) {
    const t = location.hash.split("?")[1], n = t ? new URLSearchParams(t) : null;
    return new URLSearchParams(location.search).get(e) ?? (n == null ? void 0 : n.get(e)) ?? null;
  }
  function zf() {
    if (typeof window > "u") return;
    const e = window;
    if (e.__nonameAutoDriveStarted) return;
    const t = Qe("autodrive");
    if (!t) return;
    e.__nonameAutoDriveStarted = true;
    const n = Qe("report"), s = Number(Qe("steps") ?? 1500), r = Number(Qe("gap") ?? 50), i = Number(Qe("timeout") ?? 12e4);
    (async () => {
      var _a2;
      for (let f = 0; f < 150; f++) {
        const m = Yn(Ns);
        if (m && _r() && !m.disabled) break;
        await sn(200);
      }
      (_a2 = _r()) == null ? void 0 : _a2.reset(), await sn(300);
      const o = Date.now();
      yr = o;
      let l = "unknown";
      t === "click" ? l = await Kf(s, r) : t === "auto" ? l = await Gf() : t === "manual" && (l = await Jf(s, i)), await sn(600);
      const c = Wf();
      c.memory = await Vf(), c.memory_series = Ma, n && await Bf(n, {
        ...c,
        end_state: l,
        wall_ms: Date.now() - o
      });
      const u = window;
      u.__nonameAutoDriveDone = l;
    })();
  }
  function Yf(e) {
    if (e === null || typeof e != "object" || Array.isArray(e)) return e;
    const t = e.SingleWin;
    if (t === null || typeof t != "object" || Array.isArray(t)) return e;
    const n = t.player;
    return typeof n != "number" || !Number.isInteger(n) || n <= 0 ? e : {
      ...e,
      SingleWin: {
        ...t,
        player: n - 1
      }
    };
  }
  const Qf = "noname-next-desktop-engine";
  function Xf() {
    const e = typeof location > "u" ? null : Qe("engineTransport");
    if (e === "native" || e === "wasm") return e;
    try {
      return localStorage.getItem(Qf) === "native" ? "native" : "wasm";
    } catch {
      return "wasm";
    }
  }
  function eo(e) {
    return $a(e.view, e.choice), {
      ...e,
      view: {
        ...e.view,
        outcome: Yf(e.view.outcome)
      }
    };
  }
  function Zf(e, t) {
    var _a2;
    if (!e.ok || !e.data || typeof e.data != "object") return e;
    if (t.type === "chooseGeneral") {
      const n = e.data;
      if ((_a2 = n.envelope) == null ? void 0 : _a2.view) return {
        ...e,
        data: {
          ...n,
          envelope: eo(n.envelope)
        }
      };
    } else {
      const n = e.data;
      if (n.view) return {
        ...e,
        data: eo(n)
      };
    }
    return e;
  }
  class ed {
    constructor(t = zn) {
      __publicField(this, "onmessage", null);
      __publicField(this, "onerror", null);
      __publicField(this, "closed", false);
      __publicField(this, "tail", Promise.resolve());
      __publicField(this, "resource", null);
      this.call = t;
    }
    postMessage(t) {
      if (this.closed) throw new Error("Desktop engine transport closed");
      this.tail = this.tail.then(async () => {
        var _a2;
        if (this.closed) return;
        let n;
        try {
          this.resource ?? (this.resource = this.call("native_engine_open"));
          const s = await this.resource;
          if (this.closed) return;
          if (n = await this.call("native_engine_request", {
            resourceId: s,
            request: t
          }), n = Zf(n, t), n.id !== t.id || typeof n.ok != "boolean") throw new Error("invalid IPC response");
        } catch (s) {
          n = {
            id: t.id,
            ok: false,
            error: `Desktop IPC: ${String(s)}; \u64CD\u4F5C\u7ED3\u679C\u53EF\u80FD\u672A\u77E5\uFF0C\u8BF7\u5148\u786E\u8BA4\u5F53\u524D\u5C40\u9762`
          };
        }
        this.closed || ((_a2 = this.onmessage) == null ? void 0 : _a2.call(this, {
          data: n
        }));
      });
    }
    terminate() {
      var _a2;
      this.closed || (this.closed = true, (_a2 = this.resource) == null ? void 0 : _a2.then((t) => this.call("native_engine_close", {
        resourceId: t
      })).catch((t) => console.error("Desktop engine resource close failed", t)));
    }
    addEventListener(t, n) {
    }
  }
  function Xs(e, t) {
    if (e.length === 0) return null;
    const n = [
      ...e
    ].sort((r, i) => r - i), s = Math.ceil(t / 100 * n.length);
    return n[Math.min(n.length, Math.max(1, s)) - 1];
  }
  function ys(e) {
    const t = e.filter((s) => Number.isFinite(s)), n = t.reduce((s, r) => s + r, 0);
    return {
      count: t.length,
      p50: Xs(t, 50),
      p95: Xs(t, 95),
      p99: Xs(t, 99),
      max: t.length ? Math.max(...t) : null,
      mean: t.length ? n / t.length : null
    };
  }
  function td(e) {
    const t = [];
    for (let n = 1; n < e.length; n++) t.push(e[n] - e[n - 1]);
    return t;
  }
  function nd(e) {
    const t = ys(e), n = (r) => e.length ? e.filter((i) => i > r).length / e.length : null, s = t.p50;
    return {
      ...t,
      refresh_estimate_ms: s,
      over_8_33_ratio: n(1e3 / 120),
      over_16_67_ratio: n(1e3 / 60),
      over_1_5x_refresh_ratio: s === null ? null : n(s * 1.5)
    };
  }
  const wt = (e, t) => e === null || t === null ? null : e - t;
  function to(e) {
    const t = e.worker, n = e.input_ts ?? e.sent;
    return {
      id: e.id,
      type: e.type,
      input_kind: e.input_kind,
      input_to_send_ms: wt(e.sent, e.input_ts),
      roundtrip_ms: wt(e.received, e.sent),
      worker_total_ms: (t == null ? void 0 : t.total_ms) ?? null,
      dispatch_ms: (t == null ? void 0 : t.dispatch_ms) ?? null,
      view_json_ms: (t == null ? void 0 : t.view_json_ms) ?? null,
      view_parse_ms: (t == null ? void 0 : t.view_parse_ms) ?? null,
      adapt_ms: (t == null ? void 0 : t.adapt_ms) ?? null,
      post_ms: (t == null ? void 0 : t.post_ms) ?? null,
      transfer_ms: wt(e.received_epoch_ms, (t == null ? void 0 : t.post_epoch_ms) ?? null),
      receive_to_apply_ms: wt(e.apply_start, e.received),
      apply_ms: wt(e.apply_end, e.apply_start),
      vue_patch_ms: wt(e.patched, e.apply_end),
      patch_to_frame_ms: wt(e.frame, e.patched),
      frame_to_paint_ms: wt(e.after_paint, e.frame),
      input_to_paint_ms: wt(e.after_paint, n),
      view_json_chars: (t == null ? void 0 : t.view_json_chars) ?? null,
      envelope_json_chars: (t == null ? void 0 : t.envelope_json_chars) ?? null
    };
  }
  function sd(e) {
    const t = [
      "input_to_send_ms",
      "roundtrip_ms",
      "worker_total_ms",
      "dispatch_ms",
      "view_json_ms",
      "view_parse_ms",
      "adapt_ms",
      "post_ms",
      "transfer_ms",
      "receive_to_apply_ms",
      "apply_ms",
      "vue_patch_ms",
      "patch_to_frame_ms",
      "frame_to_paint_ms",
      "input_to_paint_ms",
      "view_json_chars",
      "envelope_json_chars"
    ], n = {};
    for (const s of t) {
      const r = e.map((i) => i[s]).filter((i) => typeof i == "number" && Number.isFinite(i));
      n[s] = ys(r);
    }
    return n;
  }
  const rd = "noname-next-perf", id = 3e5, od = 2e4, br = 5e4, ad = 100;
  let no = false, Je = false;
  const bn = [], wn = [], Sn = [], pt = /* @__PURE__ */ new Map(), cn = /* @__PURE__ */ new Map();
  let An = null, un = null;
  function ld() {
    var _a2;
    try {
      if (((_a2 = globalThis.localStorage) == null ? void 0 : _a2.getItem(rd)) === "1") return true;
    } catch {
    }
    if (typeof location > "u") return false;
    if (new URLSearchParams(location.search).get("perf") === "1") return true;
    const e = location.hash.split("?")[1];
    return e ? new URLSearchParams(e).get("perf") === "1" : false;
  }
  const cd = () => performance.timeOrigin + performance.now();
  function so(e, t, n) {
    var _a2;
    if (typeof PerformanceObserver > "u" || !((_a2 = PerformanceObserver.supportedEntryTypes) == null ? void 0 : _a2.includes(e))) return;
    new PerformanceObserver((r) => {
      for (const i of r.getEntries()) n(i);
    }).observe({
      type: e,
      buffered: true,
      ...t
    });
  }
  function ro(e) {
    An = {
      ts: e.timeStamp,
      kind: e.type,
      used: false
    };
  }
  function ud() {
    if (no || (no = true, Je = ld(), !Je || typeof window > "u")) return Je;
    const e = (t) => {
      bn.length < id && bn.push(t), requestAnimationFrame(e);
    };
    return requestAnimationFrame(e), window.addEventListener("click", ro, {
      capture: true
    }), window.addEventListener("keydown", ro, {
      capture: true
    }), so("longtask", {}, (t) => {
      wn.length < br && wn.push({
        start: t.startTime,
        duration: t.duration
      });
    }), so("event", {
      durationThreshold: 16
    }, (t) => {
      const n = t;
      Sn.length < br && Sn.push({
        name: n.name,
        start: n.startTime,
        duration: n.duration,
        input_delay: n.processingStart - n.startTime,
        processing: n.processingEnd - n.processingStart
      });
    }), window.__nonamePerf = {
      version: 1,
      enabled: true,
      frames: () => bn.slice(),
      longTasks: () => wn.slice(),
      eventTimings: () => Sn.slice(),
      interactions: () => [
        ...pt.values()
      ].map(to),
      summary: () => {
        const t = [
          ...pt.values()
        ].map(to);
        return {
          frames: nd(td(bn)),
          interactions: sd(t),
          long_tasks: ys(wn.map((n) => n.duration)),
          event_timings: ys(Sn.map((n) => n.duration))
        };
      },
      reset: () => {
        bn.length = 0, wn.length = 0, Sn.length = 0, pt.clear(), cn.clear(), un = null;
      }
    }, Je && zf(), Je;
  }
  function fd(e, t) {
    if (!Je || pt.size >= od) return;
    const n = performance.now(), s = An && !An.used && n - An.ts <= ad ? An : null;
    s && (s.used = true), pt.set(e, {
      id: e,
      type: t,
      input_ts: (s == null ? void 0 : s.ts) ?? null,
      input_kind: (s == null ? void 0 : s.kind) ?? null,
      sent: n,
      received: null,
      received_epoch_ms: null,
      apply_start: null,
      apply_end: null,
      patched: null,
      frame: null,
      after_paint: null,
      worker: cn.get(e) ?? null
    }), cn.delete(e);
  }
  function io(e) {
    Je && (pt.delete(e), cn.delete(e));
  }
  function oo(e) {
    if (!Je) return;
    const t = pt.get(e);
    t && (t.received = performance.now(), t.received_epoch_ms = cd(), un = e);
  }
  function dd(e, t) {
    if (!Je) return;
    const n = pt.get(e);
    n ? n.worker = t : cn.size < br && cn.set(e, t);
  }
  function hd() {
    if (un === null) return null;
    const e = pt.get(un);
    return e && e.apply_start === null ? e : null;
  }
  function pd() {
    if (!Je) return;
    const e = hd();
    e && (e.apply_start = performance.now());
  }
  function gd() {
    if (!Je || un === null) return;
    const e = pt.get(un);
    !e || e.apply_start === null || e.apply_end !== null || (e.apply_end = performance.now(), pn(() => {
      e.patched = performance.now(), requestAnimationFrame(() => {
        e.frame = performance.now();
        const t = new MessageChannel();
        t.port1.onmessage = () => {
          e.after_paint = performance.now(), t.port1.close();
        }, t.port2.postMessage(0);
      });
    }));
  }
  function md(e) {
    return new Worker("/assets/engine.native.worker-BZvqnzJk.js", {
      type: "module",
      name: e == null ? void 0 : e.name
    });
  }
  function vd(e) {
    return new Worker("/assets/engine.worker-CvPf1BbQ.js", {
      type: "module",
      name: e == null ? void 0 : e.name
    });
  }
  function ao(e) {
    return fetch(e).then((t) => {
      if (!t.ok) throw new Error(`\u538B\u6D4B\u8F7D\u8377\u52A0\u8F7D\u5931\u8D25 ${e}: ${t.status}`);
      return t.text();
    });
  }
  class Ms {
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
      const t = "native".toLowerCase();
      this.native = t !== "legacy" && t !== "wasm" && t !== "old";
      const n = !this.native && Mt() ? await Uf() : void 0, s = this.native && Mt() && Xf() === "native";
      this.transportKind = s ? "desktop-native" : this.native ? "wasm-worker" : "legacy-worker", globalThis.__engineTransport = this.transportKind;
      const r = s ? new ed() : this.native ? new md() : new vd();
      this.worker = r, r.onmessage = (i) => {
        var _a2, _b;
        if (this.worker !== r) return;
        const a = i.data;
        if ((a == null ? void 0 : a.type) === "init_timings" && a.timings) {
          globalThis.__nativeInitTimings ?? (globalThis.__nativeInitTimings = []), globalThis.__nativeInitTimings.push(a.timings);
          return;
        }
        if ((a == null ? void 0 : a.type) === "candidate_timings" && a.timings) {
          globalThis.__nativeCandidateTimings ?? (globalThis.__nativeCandidateTimings = []), globalThis.__nativeCandidateTimings.push(a.timings);
          return;
        }
        if ((a == null ? void 0 : a.type) === "step_timings" && a.timings) {
          const c = i.data;
          dd(c.id, c.timings);
          return;
        }
        const o = i.data, l = this.pending.get(o.id);
        l && (oo(o.id), this.pending.delete(o.id), (_b = (_a2 = globalThis.__latencySabDebug) == null ? void 0 : _a2.settle) == null ? void 0 : _b.call(_a2), o.ok ? l.resolve(o.data) : l.reject(new Error(o.error)));
      }, r.onerror = (i) => {
        this.worker === r && (console.error("engine worker error", i), this.rejectPending(new Error("Worker \u53D1\u751F\u9519\u8BEF\uFF0C\u64CD\u4F5C\u7ED3\u679C\u672A\u77E5\uFF1B\u8BF7\u5148\u786E\u8BA4\u5F53\u524D\u5C40\u9762")));
      }, r.addEventListener("messageerror", () => {
        this.worker === r && this.rejectPending(new Error("Worker \u56DE\u5305\u65E0\u6CD5\u8BFB\u53D6\uFF0C\u64CD\u4F5C\u7ED3\u679C\u672A\u77E5\uFF1B\u8BF7\u5148\u786E\u8BA4\u5F53\u524D\u5C40\u9762"));
      });
      try {
        await this.request({
          id: this.nextId(),
          type: "init",
          bundle: n ?? void 0,
          perf: ud(),
          latencySab: s ? void 0 : this.initSabMirror()
        });
      } catch (i) {
        throw this.worker === r && this.dispose(), i;
      }
    }
    initSabMirror() {
      if (Qe("latency") !== "sab" || !crossOriginIsolated || typeof SharedArrayBuffer > "u") return;
      const t = document.createElement("style");
      t.textContent = ".sab-keepalive{animation:sab-pulse 8ms infinite}@keyframes sab-pulse{50%{margin-left:.5px}}", document.head.appendChild(t);
      const n = document.createElement("div");
      n.className = "sab-keepalive", n.style.display = "none", document.body.appendChild(n);
      const s = 256 * 1024, r = new SharedArrayBuffer(s), i = new Int32Array(r, 0, 2);
      Atomics.store(i, 0, 0), Atomics.store(i, 1, 0);
      const a = new TextDecoder(), o = () => {
        n.style.display = "none";
      }, l = {
        runs: 0,
        seen: 0,
        parseErr: 0,
        miss: 0
      }, c = () => {
        l.runs += 1;
        const u = Atomics.load(i, 0);
        if (u !== this.sabPollSeq) {
          l.seen += 1, this.sabPollSeq = u;
          const f = Atomics.load(i, 1);
          if (f > 0 && 8 + f <= s) try {
            const m = new Uint8Array(f);
            m.set(new Uint8Array(r, 8, f));
            const y = JSON.parse(a.decode(m)), O = this.pending.get(y.id);
            if (O) {
              oo(y.id), this.pending.delete(y.id), o();
              const T = globalThis.__latencySabDebug;
              T && (T.resolvedEarly += 1), y.ok ? O.resolve(y.data) : O.reject(new Error(y.error ?? "worker error"));
            } else l.miss += 1;
          } catch {
            l.parseErr += 1;
          }
        }
        requestAnimationFrame(c);
      };
      return requestAnimationFrame(c), globalThis.__latencySabDebug = {
        seq: () => Atomics.load(i, 0),
        raw: () => {
          const u = Atomics.load(i, 1), f = new Uint8Array(Math.min(u, 120));
          return f.set(new Uint8Array(r, 8, f.length)), {
            len: u,
            head: a.decode(f)
          };
        },
        wake: () => {
          n.style.display = "block";
        },
        settle: o,
        stats: l,
        resolvedEarly: 0,
        resolvedByMessage: 0
      }, r;
    }
    isNative() {
      return this.native;
    }
    dispose() {
      const t = this.worker;
      this.worker = null, t == null ? void 0 : t.terminate(), this.rejectPending(new Error("Worker \u5DF2\u5173\u95ED\uFF0C\u64CD\u4F5C\u7ED3\u679C\u672A\u77E5\uFF1B\u8BF7\u5148\u786E\u8BA4\u5F53\u524D\u5C40\u9762"));
    }
    rejectPending(t) {
      var _a2, _b;
      if (this.pending.size !== 0) {
        for (const [n, s] of this.pending) io(n), s.reject(t);
        this.pending.clear(), (_b = (_a2 = globalThis.__latencySabDebug) == null ? void 0 : _a2.settle) == null ? void 0 : _b.call(_a2);
      }
    }
    request(t) {
      return this.worker ? new Promise((n, s) => {
        var _a2, _b, _c2, _d2;
        this.pending.set(t.id, {
          resolve: n,
          reject: s
        }), fd(t.id, t.type), (_b = (_a2 = globalThis.__latencySabDebug) == null ? void 0 : _a2.wake) == null ? void 0 : _b.call(_a2);
        try {
          this.worker.postMessage(t);
        } catch (r) {
          this.pending.delete(t.id), io(t.id), this.pending.size === 0 && ((_d2 = (_c2 = globalThis.__latencySabDebug) == null ? void 0 : _c2.settle) == null ? void 0 : _d2.call(_c2)), s(r);
        }
      }) : Promise.reject(new Error("worker not connected"));
    }
    nextId() {
      return this.seq++;
    }
    async start(t, n, s = 1, r, i) {
      const a = Qe("packsUrl"), o = Qe("hexUrl"), l = a ? await ao(a) : r;
      return o ? (await this.request({
        id: this.nextId(),
        type: "start",
        players: t,
        seed: n,
        observer: s,
        packsJson: l,
        mode: i
      }), this.request({
        id: this.nextId(),
        type: "restore",
        hex: await ao(o),
        packsJson: l
      })) : this.request({
        id: this.nextId(),
        type: "start",
        players: t,
        seed: n,
        observer: s,
        packsJson: l,
        mode: i
      });
    }
    endPhase(t = 0) {
      return this.request({
        id: this.nextId(),
        type: "endPhase",
        actor: t
      });
    }
    playCard(t, n, s) {
      return this.request({
        id: this.nextId(),
        type: "playCard",
        actor: t,
        card: n,
        targets: [
          ...s
        ]
      });
    }
    activateSkill(t, n, s, r) {
      return this.request({
        id: this.nextId(),
        type: "activateSkill",
        actor: t,
        skill: n,
        targets: [
          ...s
        ],
        cardTargets: [
          ...r
        ]
      });
    }
    useSkillCard(t, n, s, r, i) {
      return this.request({
        id: this.nextId(),
        type: "useSkillCard",
        actor: t,
        skill: n,
        cardKind: s,
        targets: [
          ...r
        ],
        equipSlot: i
      });
    }
    resolveChoice(t, n, s, r = false) {
      return this.request({
        id: this.nextId(),
        type: "resolveChoice",
        actor: t,
        choiceId: n,
        selections: [
          ...s
        ],
        cancelled: r
      });
    }
    autoStep() {
      return this.request({
        id: this.nextId(),
        type: "autoStep"
      });
    }
    generalCandidates() {
      return this.request({
        id: this.nextId(),
        type: "generalCandidates"
      });
    }
    chooseGeneral(t, n) {
      return this.request({
        id: this.nextId(),
        type: "chooseGeneral",
        seat: t,
        name: n
      });
    }
    observer() {
      return this.native ? this.request({
        id: this.nextId(),
        type: "observer"
      }).then((t) => {
        const n = t;
        if (n == null ? void 0 : n.view) return n.view;
        throw new Error("native observer: \u7F3A view");
      }) : this.request({
        id: this.nextId(),
        type: "observer"
      });
    }
    saveHex() {
      return this.request({
        id: this.nextId(),
        type: "save"
      }).then((t) => {
        const n = t;
        if (typeof n == "string") return n;
        if (n == null ? void 0 : n.hex) return n.hex;
        throw new Error("native save: \u7F3A hex");
      });
    }
    restoreHex(t, n) {
      return this.request({
        id: this.nextId(),
        type: "restore",
        hex: t,
        packsJson: n
      });
    }
    async validatePackage(t) {
      await this.request({
        id: this.nextId(),
        type: "validatePackage",
        packJson: t
      });
    }
    loadReplay(t) {
      return this.request({
        id: this.nextId(),
        type: "loadReplay",
        json: t
      });
    }
    seekReplay(t, n, s = 1) {
      return this.request({
        id: this.nextId(),
        type: "seekReplay",
        step: t,
        privacy: n,
        player: s
      });
    }
    status() {
      return this.request({
        id: this.nextId(),
        type: "status"
      });
    }
  }
  const wr = 200, B = hn({
    contentLockHash: "",
    total: 0,
    pages: /* @__PURE__ */ new Map(),
    characters: /* @__PURE__ */ new Map(),
    skins: /* @__PURE__ */ new Map(),
    translations: {},
    selectedSkins: /* @__PURE__ */ new Map(),
    portraitUrls: /* @__PURE__ */ new Map(),
    imageUrls: /* @__PURE__ */ new Map(),
    substituteUrls: /* @__PURE__ */ new Map()
  });
  let Nt = null;
  const xn = /* @__PURE__ */ new Map(), Cn = /* @__PURE__ */ new Map(), Sr = /* @__PURE__ */ new Map();
  function yd(e) {
    if (Mt()) {
      Ar({
        kind: "desktop",
        key: "desktop"
      });
      return;
    }
    const t = `${window.location.protocol === "https:" ? "https:" : "http:"}//${window.location.hostname || "127.0.0.1"}:7700`;
    Ar({
      kind: "http",
      key: `http:${t}`,
      baseUrl: t
    });
  }
  function _d(e) {
    const t = new URL(e.trim(), window.location.href);
    t.protocol = t.protocol === "wss:" || t.protocol === "https:" ? "https:" : "http:";
    const n = t.origin;
    Ar({
      kind: "http",
      key: `http:${n}`,
      baseUrl: n
    });
  }
  async function bd(e) {
    const t = B.characters.get(e);
    if (t) return B.imageUrls.has(e) || await Er(e), t;
    if (!Nt) return;
    const n = Cn.get(e);
    if (n) return n;
    const s = Nt.key, r = (async () => {
      let i = 0;
      for (; (Nt == null ? void 0 : Nt.key) === s; ) {
        const a = await ja(i);
        if (!a) break;
        const o = B.characters.get(e);
        if (o) return await Er(e), o;
        const l = a.offset + a.characters.length;
        if (l >= a.total || l <= i) break;
        i = l;
      }
    })();
    Cn.set(e, r);
    try {
      return await r;
    } finally {
      Cn.get(e) === r && Cn.delete(e);
    }
  }
  function wd(e) {
    var _a2;
    const t = (_a2 = B.characters.get(e)) == null ? void 0 : _a2.definition.name_key;
    return t ? Da(t) : void 0;
  }
  function Sd(e) {
    var _a2;
    return ((_a2 = B.characters.get(e)) == null ? void 0 : _a2.skin_ids.flatMap((n) => {
      const s = B.skins.get(n);
      return s && s.presentation.kind !== "substitute" ? [
        s
      ] : [];
    })) ?? [];
  }
  function kd(e) {
    return Da(e.name_key);
  }
  function kr(e) {
    const t = B.characters.get(e);
    if (!t || !B.contentLockHash) return null;
    if (B.selectedSkins.has(e)) return B.selectedSkins.get(e) ?? null;
    const n = La(e);
    let s = null;
    if (n) try {
      const r = Number(localStorage.getItem(n));
      Number.isInteger(r) && Ua(t).includes(r) && (s = r);
    } catch {
    }
    return B.selectedSkins.set(e, s), s;
  }
  function Ed(e, t) {
    const n = B.characters.get(e);
    if (t != null && (!n || !Ua(n).includes(t))) return;
    B.selectedSkins.set(e, t);
    const s = La(e);
    if (s) try {
      t == null ? localStorage.removeItem(s) : localStorage.setItem(s, String(t));
    } catch {
    }
    Er(e);
  }
  function Ad(e) {
    return B.imageUrls.get(e) ?? js(e);
  }
  function xd(e) {
    return B.portraitUrls.get(e) ?? js(e);
  }
  function Cd(e, t) {
    return B.substituteUrls.get(`${e}:${t}`) ?? null;
  }
  function js(e) {
    return `/original/portraits/${e}.jpg`;
  }
  async function ja(e) {
    const t = B.pages.get(e);
    if (t) return t;
    const n = xn.get(e);
    if (n) return n;
    const s = Nt;
    if (!s) return null;
    const r = (async () => {
      let i;
      if (s.kind === "desktop") i = await Hf(e, wr);
      else {
        const a = new URL("/content-catalog", s.baseUrl);
        a.searchParams.set("offset", String(e)), a.searchParams.set("limit", String(wr));
        const o = await fetch(a);
        if (!o.ok) throw new Error(`\u5185\u5BB9\u76EE\u5F55\u52A0\u8F7D\u5931\u8D25 (${o.status})`);
        i = await o.json();
      }
      if (Nt !== s || i == null) return null;
      if (Pd(i), i.offset !== Math.min(e, i.total)) throw new Error("\u5185\u5BB9\u76EE\u5F55\u5206\u9875\u504F\u79FB\u4E0D\u4E00\u81F4");
      return e !== 0 && B.contentLockHash && B.contentLockHash !== i.content_lock_hash ? (zr(), ja(0)) : (Rd(i), i);
    })();
    xn.set(e, r);
    try {
      return await r;
    } finally {
      xn.get(e) === r && xn.delete(e);
    }
  }
  function Rd(e) {
    B.contentLockHash && B.contentLockHash !== e.content_lock_hash && zr(), B.contentLockHash = e.content_lock_hash, B.total = e.total, B.pages.set(e.offset, e);
    for (const t of e.characters) B.characters.set(t.definition.id, t);
    for (const t of e.skins) B.skins.set(t.id, t);
    for (const [t, n] of Object.entries(e.translations)) B.translations[t] = {
      ...B.translations[t],
      ...n
    };
  }
  async function Er(e) {
    const t = B.characters.get(e);
    if (!t) return;
    const n = kr(e), s = n == null ? void 0 : B.skins.get(n), r = (s == null ? void 0 : s.presentation.kind) === "static" ? s.presentation.asset : null, i = js(e);
    B.portraitUrls.set(e, i), B.imageUrls.set(e, i);
    const a = `${B.contentLockHash}:${n ?? "base"}`, o = t.portrait_asset ? await Zs(t.portrait_asset).catch(() => null) : null, l = r ? await Zs(r).catch(() => null) : null, c = t.skin_ids.map((m) => B.skins.get(m)).find((m) => (m == null ? void 0 : m.presentation.kind) === "substitute" && m.presentation.rule === "dead"), u = (c == null ? void 0 : c.presentation.kind) === "substitute" ? c.presentation.asset : null, f = u ? await Zs(u).catch(() => null) : null;
    if (a === `${B.contentLockHash}:${kr(e) ?? "base"}`) {
      const m = o ?? i;
      B.portraitUrls.set(e, m), B.imageUrls.set(e, l ?? m), f && B.substituteUrls.set(`${e}:dead`, f);
    }
  }
  function Zs(e) {
    if (!Td(e) || !Nt) return Promise.resolve(null);
    const t = Nt, n = [
      t.key,
      e.namespace,
      e.package_id,
      e.asset_id
    ].join("\0"), s = Sr.get(n);
    if (s) return s;
    const r = t.kind === "desktop" ? Ff(e) : Promise.resolve(`${t.baseUrl}/package-assets/${[
      e.namespace,
      e.package_id
    ].map(encodeURIComponent).join("/")}/${e.asset_id.split("/").map(encodeURIComponent).join("/")}?v=${encodeURIComponent(B.contentLockHash)}`);
    return Sr.set(n, r), r;
  }
  function Da(e) {
    var _a2;
    const t = typeof navigator > "u" ? "" : navigator.language, n = [
      t,
      t.replace("-", "_"),
      "zh-CN",
      "zh_CN",
      "zh",
      "en"
    ];
    for (const s of n) {
      const r = (_a2 = B.translations[s]) == null ? void 0 : _a2[e];
      if (r) return r;
    }
    return e;
  }
  function La(e) {
    return B.contentLockHash ? `noname-next.skin.${B.contentLockHash}.${e}` : null;
  }
  function Ar(e) {
    Nt = e, zr();
  }
  function zr() {
    B.contentLockHash = "", B.total = 0, B.pages.clear(), B.characters.clear(), B.skins.clear(), B.translations = {}, B.selectedSkins.clear(), B.portraitUrls.clear(), B.imageUrls.clear(), B.substituteUrls.clear(), xn.clear(), Cn.clear(), Sr.clear();
  }
  function Pd(e) {
    const t = e;
    if (!t || typeof t.content_lock_hash != "string" || !Tt(t.offset) || !Tt(t.limit) || !Tt(t.total) || t.limit < 1 || t.limit > wr || t.offset > t.total || !Array.isArray(t.characters) || t.characters.length > t.limit || !Array.isArray(t.skins) || !Array.isArray(t.aliases) || !t.translations || typeof t.translations != "object" || !t.characters.every((n) => {
      var _a2, _b;
      return Tt((_a2 = n == null ? void 0 : n.definition) == null ? void 0 : _a2.id) && n.definition.id > 0 && typeof ((_b = n == null ? void 0 : n.definition) == null ? void 0 : _b.name_key) == "string" && Array.isArray(n == null ? void 0 : n.skin_ids) && n.skin_ids.every(Tt);
    }) || !t.skins.every((n) => {
      var _a2;
      return Tt(n == null ? void 0 : n.id) && n.id > 0 && typeof (n == null ? void 0 : n.name_key) == "string" && [
        "static",
        "spine",
        "substitute"
      ].includes((_a2 = n == null ? void 0 : n.presentation) == null ? void 0 : _a2.kind);
    }) || !t.aliases.every((n) => Tt(n == null ? void 0 : n.character_id) && Tt(n == null ? void 0 : n.source_character_id) && lo(n == null ? void 0 : n.voice_mappings)) || !Object.values(t.translations).every(lo)) throw new Error("\u5185\u5BB9\u76EE\u5F55\u54CD\u5E94\u65E0\u6548");
  }
  function Tt(e) {
    return Number.isSafeInteger(e) && Number(e) >= 0;
  }
  function Ua(e) {
    return e.skin_ids.filter((t) => {
      var _a2;
      return ((_a2 = B.skins.get(t)) == null ? void 0 : _a2.presentation.kind) !== "substitute";
    });
  }
  function lo(e) {
    return !!e && typeof e == "object" && !Array.isArray(e) && Object.values(e).every((t) => typeof t == "string");
  }
  function Td(e) {
    return typeof e.namespace == "string" && e.namespace.length > 0 && typeof e.package_id == "string" && e.package_id.length > 0 && typeof e.asset_id == "string" && e.asset_id.length > 0;
  }
  function _s(e) {
    return typeof e == "string" && e.trim().length > 0 && e !== "." && e !== ".." && !e.includes("/") && !e.includes("\\");
  }
  function xr(e) {
    return Array.isArray(e) && e.length === 32 && e.every((t) => Number.isInteger(t) && t >= 0 && t <= 255);
  }
  function Ha(e) {
    const t = e, n = t == null ? void 0 : t.manifest;
    if (!n || ![
      n.namespace,
      n.package_id,
      n.version
    ].every(_s) || !xr(n.logic_hash)) throw new Error("\u6269\u5C55\u5305\u8EAB\u4EFD\u6216\u903B\u8F91\u6307\u7EB9\u65E0\u6548");
    if (n.dependencies !== void 0) {
      if (!Array.isArray(n.dependencies)) throw new Error("\u6269\u5C55\u5305\u4F9D\u8D56\u683C\u5F0F\u65E0\u6548");
      const r = /* @__PURE__ */ new Set();
      for (const i of n.dependencies) {
        if (!i || typeof i != "object" || Array.isArray(i) || ![
          i.namespace,
          i.package_id,
          i.version
        ].every(_s) || i.logic_hash !== void 0 && !xr(i.logic_hash)) throw new Error("\u6269\u5C55\u5305\u4F9D\u8D56\u683C\u5F0F\u65E0\u6548");
        const a = `${i.namespace}/${i.package_id}`;
        if (r.has(a) || a === `${n.namespace}/${n.package_id}`) throw new Error(`\u6269\u5C55\u5305\u4F9D\u8D56\u91CD\u590D\u6216\u6307\u5411\u81EA\u8EAB: ${a}`);
        r.add(a);
      }
    }
    if ((t == null ? void 0 : t.modules) !== void 0 && !Array.isArray(t.modules)) throw new Error("\u6269\u5C55\u5305 VM \u6A21\u5757\u683C\u5F0F\u65E0\u6548");
    (t == null ? void 0 : t.modules) !== void 0 && Id(t);
    const s = [
      t == null ? void 0 : t.cards,
      t == null ? void 0 : t.characters,
      t == null ? void 0 : t.pile
    ].filter((r) => r !== void 0);
    if (!s.length && (t == null ? void 0 : t.modules) === void 0 || s.some((r) => r === null || typeof r != "object" || Array.isArray(r))) throw new Error("\u6269\u5C55\u5305\u7F3A\u5C11\u53EF\u88C5\u8F7D\u7684\u5361\u724C\u3001\u6B66\u5C06\u3001\u724C\u5806\u6216 VM \u6570\u636E");
    return t;
  }
  function es(e) {
    return Number.isInteger(e) && e >= 0;
  }
  function Id(e) {
    const t = e.modules, n = /* @__PURE__ */ new Map();
    for (const a of t) {
      const o = a;
      if (!o || typeof o != "object" || Array.isArray(o)) throw new Error("\u6269\u5C55\u5305 VM \u6A21\u5757\u683C\u5F0F\u65E0\u6548");
      if (!_s(o.name) || n.has(o.name)) throw new Error("\u6269\u5C55\u5305 VM \u6A21\u5757\u540D\u7F3A\u5931\u6216\u91CD\u590D");
      const l = o.code;
      if (!Array.isArray(l) || !l.length || !l.every((c) => Number.isInteger(c) && c >= 0 && c <= 255)) throw new Error(`\u6269\u5C55\u5305 VM \u6A21\u5757\u5B57\u8282\u7801\u65E0\u6548: ${o.name}`);
      if (![
        o.local_count,
        o.max_stack,
        o.entry_pc,
        o.caps_mask
      ].every(es) || o.entry_pc >= l.length) throw new Error(`\u6269\u5C55\u5305 VM \u6A21\u5757\u5165\u53E3\u6216\u9884\u7B97\u5B57\u6BB5\u65E0\u6548: ${o.name}`);
      if (!xr(o.logic_hash)) throw new Error(`\u6269\u5C55\u5305 VM \u6A21\u5757\u6307\u7EB9\u683C\u5F0F\u65E0\u6548: ${o.name}`);
      n.set(o.name, l.length);
    }
    const s = /* @__PURE__ */ new Set(), r = e.skill_index;
    if (r !== void 0) {
      if (!Array.isArray(r)) throw new Error("\u6269\u5C55\u5305\u6280\u80FD\u7D22\u5F15\u683C\u5F0F\u65E0\u6548");
      for (const a of r) {
        const o = a;
        if (!o || typeof o != "object" || !es(o.skill_def_id) || !_s(o.name) || s.has(o.skill_def_id)) throw new Error("\u6269\u5C55\u5305\u6280\u80FD\u7D22\u5F15\u6761\u76EE\u65E0\u6548\u6216\u91CD\u590D");
        s.add(o.skill_def_id);
      }
    }
    const i = e.trigger_index;
    if (i !== void 0) {
      if (!Array.isArray(i)) throw new Error("\u6269\u5C55\u5305\u89E6\u53D1\u7D22\u5F15\u683C\u5F0F\u65E0\u6548");
      for (const a of i) {
        const o = a;
        if (!o || typeof o != "object" || typeof o.module != "string" || !n.has(o.module)) throw new Error("\u6269\u5C55\u5305\u89E6\u53D1\u7D22\u5F15\u5F15\u7528\u4E86\u4E0D\u5B58\u5728\u7684 VM \u6A21\u5757");
        if (!es(o.entry_pc) || o.entry_pc >= n.get(o.module) || !es(o.skill_def_id) || r !== void 0 && !s.has(o.skill_def_id)) throw new Error(`\u6269\u5C55\u5305\u89E6\u53D1\u7D22\u5F15\u5165\u53E3\u6216\u6280\u80FD\u5F15\u7528\u65E0\u6548: ${o.module}`);
      }
    }
  }
  function as(e) {
    return `${e.manifest.namespace}/${e.manifest.package_id}`;
  }
  function Fa(e) {
    return e.manifest.logic_hash.map((t) => t.toString(16).padStart(2, "0")).join("");
  }
  const qa = "noname-next.extension-store.v1", Wa = "noname-next.native-saves.v1";
  function ts() {
    return {
      installed: [],
      contentLock: [],
      matchFrozen: false
    };
  }
  function Va() {
    if (typeof localStorage > "u") return ts();
    try {
      const e = localStorage.getItem(qa);
      return e ? {
        ...ts(),
        ...JSON.parse(e)
      } : ts();
    } catch {
      return ts();
    }
  }
  function Od(e) {
    typeof localStorage > "u" || localStorage.setItem(qa, JSON.stringify(e));
  }
  function Ba() {
    return Va();
  }
  function co(e) {
    const t = Va();
    return t.matchFrozen = e, Od(t), t;
  }
  function Ka() {
    if (typeof localStorage > "u") return [];
    try {
      const e = localStorage.getItem(Wa);
      return e ? JSON.parse(e) : [];
    } catch {
      return [];
    }
  }
  function Nd(e) {
    const t = Ka().filter((n) => n.id !== e.id);
    return t.unshift(e), localStorage.setItem(Wa, JSON.stringify(t.slice(0, 20))), t;
  }
  function Ga(e) {
    const t = e.map(as);
    if (new Set(t).size !== t.length) throw new Error("\u5185\u5BB9\u9501\u5305\u542B\u91CD\u590D\u6269\u5C55");
    const n = new Map(e.map((s) => [
      as(s),
      s
    ]));
    for (const s of e) for (const r of s.manifest.dependencies ?? []) {
      const i = `${r.namespace}/${r.package_id}`, a = n.get(i);
      if (!a) throw new Error(`\u6269\u5C55\u5305\u7F3A\u5C11\u4F9D\u8D56: ${i}@${r.version}`);
      const o = r.logic_hash;
      if (a.manifest.version !== r.version || (o == null ? void 0 : o.some((l) => l !== 0)) && o.some((l, c) => l !== a.manifest.logic_hash[c])) throw new Error(`\u6269\u5C55\u5305\u4F9D\u8D56\u7248\u672C\u6216\u6307\u7EB9\u4E0D\u5339\u914D: ${i}@${r.version}`);
    }
    return {
      packsJson: e.length ? JSON.stringify(e) : void 0,
      contentLock: t,
      lockSeed: e.map((s) => `${as(s)}@${s.manifest.version}:${Fa(s)}`).join("|")
    };
  }
  function $d(e = Ba()) {
    const t = e.contentLock.map((n) => {
      var _a2;
      const s = e.installed.find((a) => a.key === n);
      if (!s || s.lifecycle !== "active") throw new Error(`\u5185\u5BB9\u9501\u4E2D\u7684\u5305\u672A\u542F\u7528: ${n}`);
      const r = (_a2 = s.nativeVersions) == null ? void 0 : _a2[s.version];
      if (!r) throw new Error(`\u7F3A\u5C11\u5DF2\u5B89\u88C5\u5305\u5185\u5BB9: ${n}\uFF0C\u8BF7\u5148\u5728\u6269\u5C55\u7BA1\u7406\u9875\u5347\u7EA7\u6216\u91CD\u65B0\u5B89\u88C5`);
      const i = Ha(JSON.parse(r));
      if (as(i) !== n || i.manifest.version !== s.version || !/^[a-f0-9]{8,64}$/.test(s.logicFingerprint) || !Fa(i).startsWith(s.logicFingerprint)) throw new Error(`\u5DF2\u5B89\u88C5\u5305\u7248\u672C\u6216\u6307\u7EB9\u4E0D\u5339\u914D: ${n}`);
      return i;
    });
    return Ga(t);
  }
  function Md(e, t = Ba()) {
    var _a2;
    const n = e.packsJson ? JSON.parse(e.packsJson) : [];
    if (!Array.isArray(n)) throw new Error("\u5B58\u6863\u6269\u5C55\u5185\u5BB9\u5FC5\u987B\u662F\u6570\u7EC4");
    const s = Ga(n.map(Ha));
    if (s.lockSeed !== e.lockSeed || JSON.stringify(s.contentLock) !== JSON.stringify(e.contentLock)) throw new Error("\u5B58\u6863\u5185\u5BB9\u9501\u4E0E\u5305\u526F\u672C\u4E0D\u5339\u914D\uFF1B\u65E7\u6269\u5C55\u5B58\u6863\u7F3A\u5C11\u5B9E\u9645\u5305\u5185\u5BB9\u65F6\u4E0D\u80FD\u6062\u590D");
    for (const r of s.contentLock) if (((_a2 = t.revokedKeys) == null ? void 0 : _a2.includes(r)) || t.installed.some((i) => i.key === r && i.lifecycle === "revoked")) throw new Error(`\u5B58\u6863\u5305\u542B\u5DF2\u64A4\u9500\u5305: ${r}`);
    return s;
  }
  const jd = {
    1: "\u5BF9\u5C40\u5F00\u59CB",
    2: "\u6478\u724C",
    3: "\u4F7F\u7528\u6216\u54CD\u5E94\u724C",
    4: "\u4F24\u5BB3\u7ED3\u7B97",
    5: "\u56DE\u590D\u4F53\u529B",
    6: "\u6FD2\u6B7B\u72B6\u6001\u53D8\u5316",
    7: "\u73A9\u5BB6\u9635\u4EA1",
    8: "\u9636\u6BB5\u6216\u56DE\u5408\u63A8\u8FDB",
    9: "\u5BF9\u5C40\u7ED3\u675F",
    10: "\u9009\u62E9\u72B6\u6001\u53D8\u5316"
  };
  function Dd(e) {
    return jd[e] ?? `\u4E8B\u4EF6 ${e}`;
  }
  const uo = 5, bs = "noname-next.pending-command", Ld = 600 * 1e3;
  function Ud() {
    try {
      if (typeof sessionStorage > "u") return null;
      const e = sessionStorage.getItem(bs);
      if (!e) return null;
      const t = JSON.parse(e);
      return !t || typeof t.roomId != "number" || typeof t.key != "number" || typeof t.createdAt != "number" || !t.kind || Date.now() - t.createdAt > Ld ? null : {
        roomId: t.roomId,
        key: t.key,
        kind: t.kind,
        createdAt: t.createdAt
      };
    } catch {
      return null;
    }
  }
  function Hd(e, t, n) {
    try {
      if (typeof sessionStorage > "u" || !n) return;
      sessionStorage.setItem(bs, JSON.stringify({
        roomId: n,
        key: t,
        kind: e,
        createdAt: Date.now()
      }));
    } catch {
    }
  }
  function ns(e) {
    try {
      if (typeof sessionStorage > "u") return;
      const t = sessionStorage.getItem(bs);
      if (!t) return;
      JSON.parse(t).key === e && sessionStorage.removeItem(bs);
    } catch {
    }
  }
  class Fd {
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
      __publicField(this, "persistedPending", Ud());
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
    async connect(t, n) {
      var _a2;
      if (((_a2 = this.socket) == null ? void 0 : _a2.readyState) === WebSocket.OPEN) return;
      this.disposed = false, this.lastUrl = t, this.lastName = n;
      const s = new Promise((i, a) => {
        this.welcomeWaiter = {
          resolve: i,
          reject: a
        };
      }), r = new Promise((i, a) => {
        const o = new WebSocket(Ja(t));
        this.socket = o, o.onopen = () => {
          if (this.socket !== o) {
            a(new Error("WebSocket \u8FDE\u63A5\u5DF2\u66FF\u6362"));
            return;
          }
          this.connSeq = 0, this.idemSeq = 1, i();
        }, o.onerror = () => a(new Error("WebSocket \u8FDE\u63A5\u5931\u8D25")), o.onmessage = (l) => {
          this.socket === o && this.handleMessage(String(l.data));
        }, o.onclose = () => {
          a(new Error("WebSocket \u8FDE\u63A5\u5DF2\u5173\u95ED")), this.socket === o && this.handleClose();
        };
      });
      s.catch(() => {
      }), await r, await s, this.send({
        type: "Hello",
        payload: {
          versions: {
            engine_api: 1,
            rule_vm_abi: 167,
            content_schema: 3,
            network_protocol: uo,
            replay_schema: 1,
            snapshot_schema: 1,
            package_format: 1,
            engine_build_id: "noname-next-web"
          },
          display_name: n
        }
      });
    }
    createRoom(t, n, s = "") {
      return this.joinRequest({
        type: "CreateRoom",
        payload: {
          players: t,
          seed: n,
          password: s
        }
      }, s);
    }
    joinRoom(t, n = "", s = null) {
      return this.roomId = t, this.joinRequest({
        type: "JoinRoom",
        payload: {
          room_id: t,
          password: n,
          preferred_seat: null,
          reconnect_token: s
        }
      }, n);
    }
    spectate(t) {
      return this.roomId = t, this.joinRequest({
        type: "Spectate",
        payload: {
          room_id: t
        }
      }, "");
    }
    rejoinRoom() {
      const t = this.session;
      return t ? t.role === "Spectator" ? this.spectate(t.roomId) : this.joinRoom(t.roomId, this.roomPassword, t.reconnectToken) : Promise.reject(new Error("\u5C1A\u672A\u52A0\u5165\u623F\u95F4"));
    }
    onEnvelope(t) {
      return this.envelopeListeners.add(t), () => this.envelopeListeners.delete(t);
    }
    onClose(t) {
      return this.closeListeners.add(t), () => this.closeListeners.delete(t);
    }
    onRoster(t) {
      return this.rosterListeners.add(t), () => this.rosterListeners.delete(t);
    }
    cancelPendingCommands(t = "\u5DF2\u53D6\u6D88\u5F85\u786E\u8BA4\u547D\u4EE4") {
      for (const n of this.pending.values()) ns(n.key), n.reject(new Error(t));
      this.pending.clear(), this.persistedPending && (ns(this.persistedPending.key), this.persistedPending = null);
    }
    endPhase(t = 0) {
      return this.submit("EndPhase");
    }
    playCard(t, n, s) {
      return this.submit({
        PlayCard: {
          card: n,
          targets: s,
          card_targets: []
        }
      });
    }
    activateSkill(t, n, s, r) {
      return this.submit({
        ActivateSkill: {
          skill: n,
          targets: s,
          card_targets: r
        }
      });
    }
    useSkillCard(t, n, s, r, i) {
      return this.submit({
        UseSkillCard: {
          skill: n,
          card_kind: s,
          targets: r,
          equip_slot: i
        }
      });
    }
    resolveChoice(t, n, s, r = false) {
      return this.submit({
        ResolveChoice: {
          choice_id: n,
          selections: s,
          cancelled: r
        }
      });
    }
    surrender() {
      return this.submit("Surrender");
    }
    requestSnapshot() {
      const t = this.nextHeader();
      return new Promise((n, s) => {
        this.snapshotWaiters.set(t.conn_seq, {
          resolve: n,
          reject: s
        });
        try {
          this.send({
            type: "RequestSnapshot",
            payload: {
              header: t,
              last_revision: this.revision
            }
          });
        } catch (r) {
          this.snapshotWaiters.delete(t.conn_seq), s(r);
        }
      });
    }
    dispose() {
      this.disposed = true, this.reconnectTimer != null && (window.clearTimeout(this.reconnectTimer), this.reconnectTimer = null), this.reconnectAttempts = 0, this.heartbeat != null && typeof window < "u" && window.clearInterval(this.heartbeat), this.heartbeat = null;
      const t = this.socket;
      this.socket = null, t == null ? void 0 : t.close(1e3, "client dispose"), this.session = null, this.roomPassword = "", this.playerId = 0, this.rejectOutstanding(new Error("\u8FDE\u63A5\u5DF2\u5173\u95ED")), this.envelopeListeners.clear(), this.rosterListeners.clear(), this.closeListeners.clear(), this.reconnectListeners.clear();
    }
    async joinRequest(t, n) {
      var _a2;
      const s = this.waitForJoin();
      try {
        this.send(t), this.roomPassword = n;
      } catch (r) {
        (_a2 = this.joinWaiter) == null ? void 0 : _a2.reject(r instanceof Error ? r : new Error(String(r))), this.joinWaiter = null;
      }
      return s;
    }
    waitForJoin() {
      if (this.joinWaiter) throw new Error("\u5DF2\u6709\u5165\u623F\u8BF7\u6C42");
      return new Promise((t, n) => {
        this.joinWaiter = {
          resolve: t,
          reject: n
        };
      });
    }
    submit(t) {
      if (!this.roomId || !this.playerId) return Promise.reject(new Error("\u5F53\u524D\u4E0D\u662F\u5DF2\u5165\u5EA7\u73A9\u5BB6"));
      const n = this.connectionId * 1e6 + this.idemSeq++;
      let s;
      const r = new Promise((i, a) => {
        s = {
          kind: t,
          key: n,
          resolve: i,
          reject: a
        }, this.pending.set(n, s);
      });
      try {
        this.sendSubmit(s), this.persistedPending = {
          roomId: this.roomId,
          key: n,
          kind: t,
          createdAt: Date.now()
        }, Hd(t, n, this.roomId);
      } catch (i) {
        return this.pending.delete(n), ns(n), Promise.reject(i);
      }
      return r;
    }
    sendSubmit(t, n = false) {
      this.send({
        type: "SubmitCommand",
        payload: {
          header: this.nextHeader(),
          command: {
            actor: this.playerId,
            expected_revision: this.revision,
            idempotency_key: t.key,
            kind: t.kind
          },
          idempotency_key: t.key,
          replay: n
        }
      });
    }
    replayPending() {
      const t = this.persistedPending && this.persistedPending.roomId === this.roomId ? this.persistedPending : null;
      if (t && !this.pending.has(t.key)) {
        const n = {
          kind: t.kind,
          key: t.key,
          resolve: (s) => {
            for (const r of this.envelopeListeners) r(s);
          },
          reject: () => {
          }
        };
        this.pending.set(n.key, n);
      }
      if (this.pending.size) for (const n of [
        ...this.pending.values()
      ]) try {
        this.sendSubmit(n, true);
      } catch {
        return;
      }
    }
    nextHeader() {
      return {
        protocol_version: uo,
        room_id: this.roomId,
        conn_seq: ++this.connSeq,
        revision: this.revision
      };
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
      if (s.view) try {
        $a(s.view, s.choice ?? null);
      } catch (r) {
        this.rejectOutstanding(new Error(`\u670D\u52A1\u7AEF\u5143\u6570\u636E\u65E0\u6548\uFF1A${String(r)}`));
        return;
      }
      switch (n.type) {
        case "Welcome":
          this.connectionId = Number(s.connection_id ?? 0), (_a2 = this.welcomeWaiter) == null ? void 0 : _a2.resolve(), this.welcomeWaiter = null;
          break;
        case "RoomCreated":
          this.roomId = Number(s.room_id ?? 0);
          break;
        case "JoinOk": {
          this.roomId = Number(s.room_id ?? this.roomId), this.playerId = Number(s.player_id ?? 0);
          const r = s.view;
          this.choice = s.choice ?? null, this.revision = r.revision;
          const i = {
            roomId: this.roomId,
            playerId: this.playerId,
            seat: Number(s.seat ?? 255),
            role: String(s.role ?? "Spectator"),
            reconnectToken: String(s.reconnect_token ?? ""),
            view: r,
            envelope: er(r, this.choice, [], s.legal_actions ?? [], null)
          };
          this.session = i, this.replayPending(), (_b = this.joinWaiter) == null ? void 0 : _b.resolve(i), this.joinWaiter = null, this.startHeartbeat();
          break;
        }
        case "RoomRoster": {
          const r = s;
          for (const i of this.rosterListeners) i(r);
          break;
        }
        case "CommandOutcome":
          this.handleOutcome(s);
          break;
        case "Snapshot": {
          const r = s.header, i = s.view;
          this.choice = s.choice ?? null, this.revision = i.revision;
          const a = this.snapshotWaiters.get(r.conn_seq);
          a && (this.snapshotWaiters.delete(r.conn_seq), a.resolve(i));
          const o = er(i, this.choice, [], s.legal_actions ?? [], null);
          for (const l of this.envelopeListeners) l(o);
          break;
        }
        case "Error": {
          const r = new Error(String(s.message ?? "\u670D\u52A1\u7AEF\u9519\u8BEF"));
          this.joinWaiter ? (this.joinWaiter.reject(r), this.joinWaiter = null) : this.rejectOutstanding(r);
          break;
        }
      }
    }
    handleOutcome(t) {
      var _a2;
      const n = t.view, s = this.revision;
      this.revision = n.revision, this.choice = t.choice ?? null;
      const r = t.public_event_codes ?? [], i = er(n, this.choice, r, t.legal_actions ?? [], t.result ?? null), a = Number(t.idempotency_key ?? 0);
      ns(a), ((_a2 = this.persistedPending) == null ? void 0 : _a2.key) === a && (this.persistedPending = null);
      const o = this.pending.get(a);
      if (o) {
        this.pending.delete(a);
        const l = Vd(t.result);
        if (l) {
          for (const c of this.envelopeListeners) c(i);
          o.reject(new Error(l));
        } else o.resolve(i);
      } else for (const l of this.envelopeListeners) l(i);
      s > 0 && n.revision > s + 1 && this.requestSnapshot().catch(() => {
      });
    }
    startHeartbeat() {
      this.heartbeat != null && window.clearInterval(this.heartbeat), this.heartbeat = window.setInterval(() => {
        var _a2;
        if (((_a2 = this.socket) == null ? void 0 : _a2.readyState) !== WebSocket.OPEN) return;
        const t = this.nextHeader();
        this.send({
          type: "Heartbeat",
          payload: {
            header: t
          }
        });
      }, 15e3);
    }
    handleClose() {
      this.heartbeat != null && window.clearInterval(this.heartbeat);
      for (const t of this.closeListeners) try {
        t();
      } catch {
      }
      this.heartbeat = null, this.socket = null, this.playerId = 0, this.disposed || (this.suspendOutstanding(new Error("\u4E0E\u670D\u52A1\u7AEF\u7684\u8FDE\u63A5\u5DF2\u65AD\u5F00")), this.scheduleReconnect());
    }
    scheduleReconnect() {
      if (this.reconnectTimer != null || !this.lastUrl) return;
      if (this.reconnectAttempts >= 5) {
        this.failPendingUncertain();
        for (const n of this.reconnectListeners) n("failed");
        return;
      }
      const t = Math.min(1e3 * 2 ** this.reconnectAttempts, 8e3);
      this.reconnectAttempts += 1;
      for (const n of this.reconnectListeners) n("reconnecting");
      this.reconnectTimer = window.setTimeout(() => {
        this.reconnectTimer = null;
        const n = this.lastUrl;
        !n || this.disposed || this.connect(n, this.lastName).then(() => {
          this.reconnectAttempts = 0;
          for (const s of this.reconnectListeners) s("reconnected");
        }).catch(() => {
          this.scheduleReconnect();
        });
      }, t);
    }
    onReconnect(t) {
      return this.reconnectListeners.add(t), () => this.reconnectListeners.delete(t);
    }
    suspendOutstanding(t) {
      var _a2, _b;
      (_a2 = this.welcomeWaiter) == null ? void 0 : _a2.reject(t), this.welcomeWaiter = null, (_b = this.joinWaiter) == null ? void 0 : _b.reject(t), this.joinWaiter = null;
      for (const n of this.snapshotWaiters.values()) n.reject(t);
      this.snapshotWaiters.clear();
    }
    failPendingUncertain() {
      if (this.pending.size) {
        for (const t of this.pending.values()) t.reject(new Error("\u63D0\u4EA4\u7ED3\u679C\u4E0D\u786E\u5B9A\uFF1A\u8FDE\u63A5\u5DF2\u65AD\u5F00\u4E14\u81EA\u52A8\u91CD\u8FDE\u5931\u8D25\uFF0C\u8BF7\u91CD\u65B0\u8FDB\u5165\u623F\u95F4\u540E\u518D\u64CD\u4F5C"));
        this.pending.clear();
      }
    }
    rejectOutstanding(t) {
      this.suspendOutstanding(t);
      for (const n of this.pending.values()) n.reject(t);
      this.pending.clear();
    }
  }
  function Ja(e) {
    const t = e.trim();
    return t.startsWith("http://") ? `ws://${t.slice(7)}` : t.startsWith("https://") ? `wss://${t.slice(8)}` : t;
  }
  function fo(e, t) {
    const n = new URL(Ja(e)).origin;
    return `noname-next.reconnect.${encodeURIComponent(n)}.${t}`;
  }
  function er(e, t, n, s, r) {
    return {
      result: r,
      events: n.map((i) => ({
        PublicEvent: {
          code: i,
          label: Dd(i)
        }
      })),
      choice: t,
      view: e,
      legal_actions: s,
      hash_hex: qd(e.state_hash),
      revision: e.revision,
      finished: Wd(e.outcome)
    };
  }
  function qd(e) {
    return typeof e == "string" ? e : e.map((t) => t.toString(16).padStart(2, "0")).join("");
  }
  function Wd(e) {
    return e == null || e === "InProgress" ? false : !(typeof e == "object" && "InProgress" in e);
  }
  function Vd(e) {
    var _a2;
    if (!e || typeof e != "object" || !("Rejected" in e)) return null;
    const t = e.Rejected;
    return (t && typeof t == "object" && "IllegalAction" in t ? (_a2 = t.IllegalAction) == null ? void 0 : _a2.code : void 0) === "legacy_replay_no_receipt" ? "\u63D0\u4EA4\u7ED3\u679C\u4E0D\u786E\u5B9A\uFF1A\u65E7\u623F\u95F4\u6CA1\u6709\u53EF\u6838\u5BF9\u7684\u56DE\u6267\uFF0C\u5DF2\u505C\u6B62\u91CD\u653E\uFF1B\u8BF7\u4EE5\u623F\u95F4\u5F53\u524D\u72B6\u6001\u4E3A\u51C6" : `\u547D\u4EE4\u88AB\u62D2\u7EDD\uFF1A${JSON.stringify(t)}`;
  }
  function za(e, t) {
    var _a2, _b;
    const n = (i) => ({
      label: i,
      isSelfWin: null
    });
    if (e == null) return n("\u80DC\u8D1F\u672A\u77E5");
    if (e === "finished") return n("\u5DF2\u7ED3\u675F\uFF08\u80DC\u8D1F\u7C7B\u578B\u672A\u5EFA\u6A21\uFF09");
    const s = typeof e == "object" ? e : {};
    switch (typeof e == "string" ? e : Object.keys(s)[0]) {
      case "LoyalistWin":
        return {
          label: "\u4E3B\u516C\u65B9\u80DC",
          isSelfWin: (t == null ? void 0 : t.identity) ? t.identity === "Lord" || t.identity === "Loyalist" : null
        };
      case "RebelWin":
        return {
          label: "\u53CD\u8D3C\u80DC",
          isSelfWin: (t == null ? void 0 : t.identity) ? t.identity === "Rebel" : null
        };
      case "RenegadeWin":
        return {
          label: "\u5185\u5978\u80DC",
          isSelfWin: (t == null ? void 0 : t.identity) ? t.identity === "Renegade" : null
        };
      case "SingleWin": {
        const i = (_a2 = s.SingleWin) == null ? void 0 : _a2.player;
        if (typeof i == "number" && Number.isInteger(i) && i >= 0) return {
          label: `\u5355\u6311\u80DC\u8005 P${i}`,
          isSelfWin: t ? t.player === i : null
        };
        break;
      }
      case "GroupWin": {
        const i = (_b = s.GroupWin) == null ? void 0 : _b.group;
        if (typeof i == "string") return n(`\u52BF\u529B\u80DC\uFF08${i}\uFF09`);
        break;
      }
      case "Draw":
        return n("\u5E73\u5C40");
      case "Aborted":
        return n("\u5BF9\u5C40\u4E2D\u6B62");
    }
    return n(`\u5DF2\u7ED3\u675F\uFF08${JSON.stringify(e)}\uFF09`);
  }
  function Bd(e) {
    return za(e).label;
  }
  function Kd(e, t, n) {
    var _a2;
    let s = e == null ? void 0 : e.outcome;
    if (s == null) for (const r of t) {
      if (!r || typeof r != "object") continue;
      const i = r;
      if (s = i.type === "MatchEnded" ? i.outcome : (_a2 = i.MatchEnded) == null ? void 0 : _a2.outcome, s != null) break;
    }
    return za(s, e == null ? void 0 : e.seats.find((r) => r.player === n));
  }
  function Gd(e) {
    if (!e || typeof e != "object") return "\u547D\u4EE4\u7ED3\u679C\u65E0\u6CD5\u8BFB\u53D6";
    let t = e;
    if (t.kind === "plan" && typeof t.error == "string") return `\u81EA\u52A8\u64CD\u4F5C\u5931\u8D25\uFF1A${t.error === "stale" ? "\u5C40\u9762\u5DF2\u53D8\u5316\uFF0C\u8BF7\u91CD\u8BD5" : t.error === "cancelled" ? "\u5DF2\u53D6\u6D88" : t.error}`;
    if (t.kind === "plan" || t.kind === "answer") try {
      const r = typeof t.outcome == "string" ? JSON.parse(t.outcome) : null;
      if (!r || typeof r != "object" || Array.isArray(r)) return "\u81EA\u52A8\u64CD\u4F5C\u7ED3\u679C\u65E0\u6CD5\u8BFB\u53D6";
      t = r;
    } catch {
      return "\u81EA\u52A8\u64CD\u4F5C\u7ED3\u679C\u65E0\u6CD5\u8BFB\u53D6";
    }
    if (t.done === true || t.outcome === "accepted" || t.outcome === "pending") return null;
    if (t.outcome === "suspended") return "\u64CD\u4F5C\u5DF2\u6682\u505C\uFF0C\u6682\u65E0\u6CD5\u7EE7\u7EED\u7ED3\u7B97";
    if (t.outcome !== "rejected") return "\u547D\u4EE4\u7ED3\u679C\u65E0\u6CD5\u8BFB\u53D6";
    const n = {
      ChoicePending: "\u8BF7\u5148\u5B8C\u6210\u5F53\u524D\u9009\u62E9",
      NoSuchChoice: "\u5F53\u524D\u9009\u62E9\u5DF2\u5931\u6548",
      InvalidChoice: "\u9009\u62E9\u65E0\u6548",
      NotYourTurn: "\u8FD8\u6CA1\u6709\u8F6E\u5230\u8BE5\u73A9\u5BB6",
      WrongPhase: "\u5F53\u524D\u9636\u6BB5\u4E0D\u80FD\u6267\u884C\u6B64\u64CD\u4F5C",
      CardNotInHand: "\u8FD9\u5F20\u724C\u4E0D\u5728\u624B\u724C\u4E2D",
      IllegalTargets: "\u76EE\u6807\u4E0D\u5408\u6CD5",
      WrongPlayer: "\u5F53\u524D\u73A9\u5BB6\u65E0\u6743\u5904\u7406\u6B64\u9009\u62E9",
      StaleRevision: "\u5C40\u9762\u5DF2\u53D8\u5316\uFF0C\u8BF7\u91CD\u8BD5",
      MatchOver: "\u5BF9\u5C40\u5DF2\u7ED3\u675F"
    }, s = typeof t.reason == "string" && t.reason ? t.reason : "\u539F\u56E0\u672A\u77E5";
    return `\u64CD\u4F5C\u88AB\u62D2\u7EDD\uFF1A${n[s] ?? s}`;
  }
  let Jd = 1;
  function zd() {
    var _a2;
    const e = typeof window < "u" && ((_a2 = window.matchMedia) == null ? void 0 : _a2.call(window, "(prefers-reduced-motion: reduce)").matches);
    return hn({
      connected: false,
      busy: false,
      error: null,
      revision: 0,
      hash: "",
      finished: false,
      view: null,
      choice: null,
      legalActions: [],
      lastEvents: [],
      log: [],
      effects: [],
      speed: 1,
      skipAnims: false,
      reduceMotion: !!e
    });
  }
  let Jt = 1;
  function ho(e, t) {
    pd(), Yd(e, t), gd();
  }
  function Yd(e, t) {
    e.revision = t.revision, e.hash = t.hash_hex, e.finished = t.finished, e.view = t.view, e.choice = t.choice, e.legalActions = t.legal_actions ?? [], e.lastEvents = t.events ?? [], e.error = null;
    const n = Qd(t.events ?? []);
    e.effects = n;
    const s = Zd(t.events ?? []);
    for (const r of s) Ge(e, r);
  }
  function Qd(e) {
    const t = [];
    for (const n of e) {
      if (!n || typeof n != "object") continue;
      const s = n, r = Object.keys(s)[0];
      if (!r) continue;
      const i = s[r];
      switch (r) {
        case "DamageDealt":
          t.push({
            id: Jt++,
            type: "damage",
            player: i == null ? void 0 : i.target,
            amount: i == null ? void 0 : i.amount,
            text: `-${i == null ? void 0 : i.amount}`
          });
          break;
        case "HpRecovered":
          t.push({
            id: Jt++,
            type: "heal",
            player: i == null ? void 0 : i.player,
            amount: i == null ? void 0 : i.amount,
            text: `+${i == null ? void 0 : i.amount}`
          });
          break;
        case "CardsDrawn":
          t.push({
            id: Jt++,
            type: "draw",
            player: i == null ? void 0 : i.player,
            text: `\u6478${Array.isArray(i == null ? void 0 : i.cards) ? i.cards.length : 1}`
          });
          break;
        case "CardUsed":
          t.push({
            id: Jt++,
            type: "play",
            player: i == null ? void 0 : i.player,
            text: "\u51FA\u724C"
          });
          break;
        case "EnteredDying":
          t.push({
            id: Jt++,
            type: "dying",
            player: i == null ? void 0 : i.player,
            text: "\u6FD2\u6B7B"
          });
          break;
        case "PlayerKilled":
          t.push({
            id: Jt++,
            type: "death",
            player: i == null ? void 0 : i.player,
            text: "\u9635\u4EA1"
          });
          break;
      }
    }
    return t;
  }
  function Xd(e) {
    e.effects = [];
  }
  function Ge(e, t) {
    e.log.unshift({
      id: Jd++,
      text: t,
      at: Date.now()
    }), e.log.length > 80 && (e.log.length = 80);
  }
  function Zd(e) {
    const t = [];
    for (const n of e) {
      if (!n || typeof n != "object") continue;
      const s = n, r = Object.keys(s)[0];
      if (!r) continue;
      const i = s[r];
      switch (r) {
        case "GameStarted":
          t.push(`\u5BF9\u5C40\u5F00\u59CB \xB7 ${(i == null ? void 0 : i.player_count) ?? "?"} \u4EBA`);
          break;
        case "CardsDrawn":
          t.push(`\u73A9\u5BB6 ${i == null ? void 0 : i.player} \u6478\u724C \xD7${Array.isArray(i == null ? void 0 : i.cards) ? i.cards.length : "?"}`);
          break;
        case "CardUsed":
          t.push(`\u73A9\u5BB6 ${i == null ? void 0 : i.player} \u4F7F\u7528\u5361\u724C ${i == null ? void 0 : i.card}`);
          break;
        case "DamageDealt":
          t.push(`\u4F24\u5BB3 ${i == null ? void 0 : i.source} \u2192 ${i == null ? void 0 : i.target} (${i == null ? void 0 : i.amount})`);
          break;
        case "HpRecovered":
          t.push(`\u73A9\u5BB6 ${i == null ? void 0 : i.player} \u56DE\u590D ${i == null ? void 0 : i.amount}`);
          break;
        case "EnteredDying":
          t.push(`\u73A9\u5BB6 ${i == null ? void 0 : i.player} \u8FDB\u5165\u6FD2\u6B7B`);
          break;
        case "PlayerKilled":
          t.push(`\u73A9\u5BB6 ${i == null ? void 0 : i.player} \u9635\u4EA1`);
          break;
        case "TurnStateChanged":
          t.push(`\u56DE\u5408 \xB7 \u73A9\u5BB6 ${i == null ? void 0 : i.current} \xB7 ${i == null ? void 0 : i.phase}`);
          break;
        case "MatchEnded":
          t.push(`\u7EC8\u5C40 \xB7 ${Bd(i == null ? void 0 : i.outcome)}`);
          break;
        case "ChoiceRequested":
          t.push(`\u7B49\u5F85\u9009\u62E9 \xB7 \u73A9\u5BB6 ${i == null ? void 0 : i.observer}`);
          break;
        case "PublicEvent":
          t.push(String((i == null ? void 0 : i.label) ?? `\u516C\u5171\u4E8B\u4EF6 ${(i == null ? void 0 : i.code) ?? "?"}`));
          break;
        default:
          t.push(r);
      }
    }
    return t;
  }
  function eh(e, t) {
    if (e.skipAnims || e.reduceMotion || e.speed <= 0) return Promise.resolve();
    const n = Math.max(0, t / e.speed);
    return new Promise((s) => setTimeout(s, n));
  }
  function Qt(e) {
    return typeof e == "object" && e !== null && !Array.isArray(e);
  }
  function ws(e) {
    return typeof e == "string" ? e : e instanceof Error ? e.message : String(e);
  }
  function Rn(e) {
    try {
      const t = JSON.stringify(e);
      return (t === void 0 ? String(e) : t).slice(0, 160);
    } catch {
      return String(e);
    }
  }
  function th(e) {
    return Array.isArray(e) && e.every((t) => typeof t == "string");
  }
  function nh(e) {
    if (!Qt(e)) throw new Error(`\u9009\u5C06\u5019\u9009\u7ED3\u6784\u4E0D\u7B26\uFF08\u9700 {count, candidates}\uFF09\uFF1A${Rn(e)}`);
    const t = e.count, n = e.candidates;
    if (typeof t != "number" || !Number.isInteger(t) || t < 0 || !Array.isArray(n)) throw new Error(`\u9009\u5C06\u5019\u9009\u7ED3\u6784\u4E0D\u7B26\uFF08\u9700 {count:\u975E\u8D1F\u6574\u6570, candidates:\u6570\u7EC4}\uFF09\uFF1A${Rn(e)}`);
    const s = [], r = /* @__PURE__ */ new Set();
    for (const [i, a] of n.entries()) {
      if (!Qt(a)) throw new Error(`\u9009\u5C06\u5019\u9009\u7B2C ${i} \u9879\u7ED3\u6784\u4E0D\u7B26\uFF08\u9700\u5BF9\u8C61\uFF09\uFF1A${Rn(a)}`);
      const o = a.def, l = a.name, c = a.skills;
      if (typeof o != "number" || !Number.isInteger(o) || o < 0 || typeof l != "string" || l.trim().length === 0 || !th(c)) throw new Error(`\u9009\u5C06\u5019\u9009\u7B2C ${i} \u9879\u7ED3\u6784\u4E0D\u7B26\uFF08\u9700 {def:\u975E\u8D1F\u6574\u6570, name:\u975E\u7A7A\u5B57\u7B26\u4E32, skills:\u5B57\u7B26\u4E32\u6570\u7EC4}\uFF09\uFF1A${Rn(a)}`);
      if (r.has(o)) throw new Error(`\u9009\u5C06\u5019\u9009\u7B2C ${i} \u9879 def=${o} \u4E0E\u524D\u9762\u91CD\u590D`);
      r.add(o), s.push({
        def: o,
        name: l,
        skills: [
          ...c
        ],
        skillLabel: c.length > 0 ? c.join(" \xB7 ") : "\u65E0\u6280\u80FD"
      });
    }
    if (s.length !== t) throw new Error(`\u9009\u5C06\u5019\u9009 count=${t} \u4E0E\u5019\u9009\u6570 ${s.length} \u4E0D\u4E00\u81F4`);
    return s;
  }
  function Yr() {
    return {
      characterDef: null,
      skillIds: [],
      skillNames: []
    };
  }
  function sh(e, t = Yr()) {
    return {
      seat: e,
      loadStatus: "idle",
      candidates: [],
      error: null,
      selectionStatus: "idle",
      pendingName: null,
      assignment: null,
      lastStateHash: null,
      projection: Ya(t)
    };
  }
  function Ya(e) {
    return {
      characterDef: e.characterDef,
      skillIds: [
        ...e.skillIds
      ],
      skillNames: [
        ...e.skillNames
      ]
    };
  }
  function Qa(e, t) {
    const s = (Array.isArray(e == null ? void 0 : e.seats) ? e.seats : []).find((o) => o && o.player === t);
    if (!s) return Yr();
    const r = [], i = [], a = Array.isArray(s.skills) ? s.skills : [];
    for (const o of a) {
      const l = o == null ? void 0 : o.id;
      typeof l == "number" && Number.isInteger(l) && !r.includes(l) && r.push(l);
      const c = o == null ? void 0 : o.name;
      typeof c == "string" && c.length > 0 && !i.includes(c) && i.push(c);
    }
    return {
      characterDef: typeof s.character_def == "number" ? s.character_def : null,
      skillIds: r,
      skillNames: i
    };
  }
  function Xa(e, t) {
    return {
      ...e,
      projection: Ya(t)
    };
  }
  function po(e, t) {
    return {
      ...e,
      seat: t,
      projection: Yr(),
      assignment: null,
      lastStateHash: null,
      selectionStatus: "idle",
      pendingName: null,
      error: null
    };
  }
  function rh(e) {
    return {
      ...e,
      loadStatus: "loading",
      error: null
    };
  }
  function ih(e, t) {
    try {
      const n = nh(t);
      return {
        ...e,
        loadStatus: n.length > 0 ? "ready" : "empty",
        candidates: n,
        error: null
      };
    } catch (n) {
      return {
        ...e,
        loadStatus: "error",
        candidates: [],
        error: `\u9009\u5C06\u5019\u9009\u4E0D\u53EF\u7528\uFF1A${ws(n)}`
      };
    }
  }
  function oh(e, t) {
    return {
      ...e,
      loadStatus: "error",
      candidates: [],
      error: `\u9009\u5C06\u5019\u9009\u52A0\u8F7D\u5931\u8D25\uFF1A${ws(t)}`
    };
  }
  function ah(e, t) {
    if (!Qt(e)) return `\u56DE\u5305\u4E0D\u662F\u5BF9\u8C61\uFF1A${Rn(e)}`;
    const n = e.assignment, s = e.envelope;
    return Qt(n) ? n.seat !== t.seat ? `\u5EA7\u4F4D\u56DE\u663E ${String(n.seat)} \u2260 \u8BF7\u6C42 ${t.seat}` : n.name !== t.name ? `\u6B66\u5C06\u56DE\u663E ${String(n.name)} \u2260 \u8BF7\u6C42 ${t.name}` : typeof n.state_hash != "string" || n.state_hash.length === 0 ? "\u7F3A assignment.state_hash" : !Qt(s) || !Qt(s.view) ? "\u7F3A\u5237\u65B0\u4FE1\u5C01 view" : null : "\u7F3A assignment";
  }
  async function lh(e, t, n, s) {
    var _a2;
    if (typeof t != "string" || t.trim().length === 0) return {
      ...e,
      selectionStatus: "rejected",
      pendingName: null,
      error: "\u9009\u5C06\u5931\u8D25\uFF1A\u5019\u9009\u540D\u4E3A\u7A7A"
    };
    const r = {
      ...e,
      selectionStatus: "pending",
      pendingName: t,
      error: null
    };
    s == null ? void 0 : s(r);
    let i;
    try {
      i = await n.chooseGeneral(e.seat, t);
    } catch (f) {
      return {
        ...e,
        selectionStatus: "rejected",
        pendingName: null,
        error: `\u9009\u5C06\u5931\u8D25\uFF08seat=${e.seat}, name=${t}\uFF09\uFF1A${ws(f)}`
      };
    }
    const a = ah(i, {
      seat: e.seat,
      name: t
    });
    if (a) return {
      ...e,
      selectionStatus: "rejected",
      pendingName: null,
      error: `\u9009\u5C06\u8FD4\u56DE\u65E0\u6CD5\u786E\u8BA4\u88C5\u914D\uFF1A${a}`
    };
    const o = i.assignment, l = i.envelope, c = Xa({
      ...r,
      assignment: o,
      lastStateHash: o.state_hash
    }, Qa(l.view, e.seat));
    let u = null;
    try {
      (_a2 = n.applyEnvelope) == null ? void 0 : _a2.call(n, l);
    } catch (f) {
      u = `\u9009\u5C06\u5DF2\u88C5\u914D\uFF0C\u4F46\u89C6\u56FE\u5237\u65B0\u5931\u8D25\uFF1A${ws(f)}`;
    }
    return {
      ...c,
      selectionStatus: "accepted",
      pendingName: null,
      error: u
    };
  }
  const ch = [
    "data-slot",
    "aria-label"
  ], uh = [
    "src",
    "alt"
  ], fh = [
    "aria-label"
  ], dh = {
    class: "name"
  }, hh = {
    key: 2,
    class: "muted"
  }, ph = [
    "value",
    "aria-label"
  ], gh = [
    "value"
  ], mh = Dt({
    __name: "GeneralPortrait",
    props: {
      general: {},
      legacyDef: {},
      alive: {
        type: Boolean
      },
      slot: {}
    },
    setup(e) {
      const t = e, n = Q(() => {
        var _a2;
        return ((_a2 = t.general) == null ? void 0 : _a2.def) ?? t.legacyDef ?? 0;
      }), s = Q(() => t.general !== null || t.legacyDef !== void 0), r = Q(() => {
        var _a2;
        return s.value ? ((_a2 = t.general) == null ? void 0 : _a2.name) || wd(n.value) || `#${n.value}` : "\u6697\u5C06\uFF0F\u672A\u914D\u7F6E";
      }), i = {
        wei: "\u9B4F",
        shu: "\u8700",
        wu: "\u5434",
        qun: "\u7FA4"
      }, a = Q(() => {
        var _a2;
        const L = ((_a2 = t.general) == null ? void 0 : _a2.faction) ?? "";
        return Object.hasOwn(i, L) ? i[L] : "\u672A\u77E5\u52BF\u529B";
      }), o = Q(() => Sd(n.value)), l = Q(() => kr(n.value)), c = Q(() => Ad(n.value)), u = Q(() => xd(n.value)), f = Q(() => js(n.value)), m = Q(() => Array.from(/* @__PURE__ */ new Set([
        ...t.alive ? [] : [
          Cd(n.value, "dead")
        ],
        c.value,
        u.value,
        f.value
      ])).filter((L) => !!L)), y = ye([]), O = Q(() => m.value.find((L) => !y.value.includes(L)) ?? "");
      Bt([
        n,
        s
      ], ([L, E]) => {
        E && bd(L).catch(() => {
        });
      }, {
        immediate: true
      }), Bt(m, () => {
        y.value = [];
      });
      function T() {
        O.value && (y.value = [
          ...y.value,
          O.value
        ]);
      }
      function D(L) {
        const E = L.target.value;
        Ed(n.value, E === "" ? null : Number(E));
      }
      return (L, E) => (x(), C("div", {
        class: "general-slot",
        "data-slot": e.slot,
        "aria-label": `${e.slot === 0 ? "\u4E3B\u5C06" : "\u526F\u5C06"} ${r.value}`
      }, [
        s.value && O.value ? (x(), C("img", {
          key: 0,
          class: "portrait",
          src: O.value,
          alt: `${r.value}\u6B66\u5C06\u5934\u50CF`,
          onError: T
        }, null, 40, uh)) : (x(), C("div", {
          key: 1,
          class: "portrait no-image",
          role: "img",
          "aria-label": `${r.value}\u6682\u65E0\u5934\u50CF`
        }, P(s.value ? "\u65E0\u56FE" : "\u672A\u77E5"), 9, fh)),
        k("span", dh, P(r.value), 1),
        e.general ? (x(), C("span", hh, P(a.value) + " \xB7 " + P(e.general.revealed ? "\u660E\u7F6E" : "\u6697\u7F6E"), 1)) : z("", true),
        s.value && o.value.length ? (x(), C("select", {
          key: 3,
          class: "skin-select",
          value: l.value ?? "",
          "aria-label": `${r.value}\u76AE\u80A4`,
          onClick: E[0] || (E[0] = Ii(() => {
          }, [
            "stop"
          ])),
          onKeydown: E[1] || (E[1] = Ii(() => {
          }, [
            "stop"
          ])),
          onChange: D
        }, [
          E[2] || (E[2] = k("option", {
            value: ""
          }, "\u9ED8\u8BA4\u76AE\u80A4", -1)),
          (x(true), C(de, null, xe(o.value, (H) => (x(), C("option", {
            key: H.id,
            value: H.id
          }, P(W(kd)(H)) + P(H.presentation.kind === "spine" ? "\uFF08\u9759\u6001\u56DE\u9000\uFF09" : ""), 9, gh))), 128))
        ], 40, ph)) : z("", true)
      ], 8, ch));
    }
  }), gn = (e, t) => {
    const n = e.__vccOpts || e;
    for (const [s, r] of t) n[s] = r;
    return n;
  }, vh = gn(mh, [
    [
      "__scopeId",
      "data-v-689e5d5b"
    ]
  ]), yh = [
    "aria-label",
    "tabindex"
  ], _h = {
    key: 0,
    class: "tag"
  }, bh = {
    key: 1,
    class: "tag",
    "aria-label": "\u6B66\u5C06\u724C\u80CC\u9762"
  }, wh = {
    class: "generals",
    "aria-label": "\u53CC\u5C06"
  }, Sh = {
    class: "hp-row"
  }, kh = [
    "aria-label"
  ], Eh = {
    class: "hp",
    "aria-label": "\u4F53\u529B"
  }, Ah = [
    "aria-valuenow",
    "aria-valuemax",
    "aria-label"
  ], xh = {
    class: "meta muted"
  }, Ch = {
    key: 0,
    class: "equip-zone",
    "aria-label": "\u88C5\u5907\u533A"
  }, Rh = [
    "title"
  ], Ph = {
    class: "equip-suit"
  }, Th = {
    class: "equip-id"
  }, Ih = {
    key: 1,
    class: "dead-badge"
  }, Oh = {
    key: 2,
    class: "dying-badge"
  }, Nh = {
    key: 3,
    class: "target-hint"
  }, $h = Dt({
    __name: "SeatCard",
    props: {
      seat: {},
      self: {
        type: Boolean
      },
      identityLabel: {},
      isTargetMode: {
        type: Boolean
      },
      isSelectableTarget: {
        type: Boolean
      },
      isDropTarget: {
        type: Boolean
      },
      equipCards: {}
    },
    emits: [
      "select"
    ],
    setup(e, { emit: t }) {
      const n = e, s = t, r = Q(() => n.seat.generals ?? [
        null,
        null
      ]), i = Q(() => !n.seat.generals || r.value.every((c) => c === null) && n.seat.character_def !== 0 ? n.seat.character_def : void 0), a = Q(() => r.value.map((c, u) => (c == null ? void 0 : c.name) || (u === 0 && i.value !== void 0 ? `#${i.value}` : "\u6697\u5C06\uFF0F\u672A\u914D\u7F6E")).join(" / "));
      function o(c, u, f) {
        return f ? c <= 0 ? "\u26A0" : "\u2713" : "\u2717";
      }
      function l(c, u, f) {
        return f ? c <= 0 ? "var(--color-warning)" : "var(--color-positive)" : "var(--color-negative)";
      }
      return (c, u) => (x(), C("article", {
        class: Fe([
          "seat has-portrait",
          {
            self: e.self,
            dead: !e.seat.alive,
            dying: e.seat.alive && e.seat.hp <= 0,
            "target-mode": e.isTargetMode,
            "target-selectable": e.isTargetMode && e.isSelectableTarget,
            "drop-target": e.isDropTarget,
            "current-turn": false
          }
        ]),
        role: "listitem",
        "aria-label": `\u5EA7\u4F4D ${e.seat.seat} \u73A9\u5BB6 ${e.seat.player} \u6B66\u5C06 ${a.value}`,
        tabindex: e.isTargetMode && e.isSelectableTarget ? 0 : -1,
        onClick: u[0] || (u[0] = (f) => e.isTargetMode && e.isSelectableTarget && s("select", e.seat.player)),
        onKeydown: u[1] || (u[1] = Eu((f) => e.isTargetMode && e.isSelectableTarget && s("select", e.seat.player), [
          "enter"
        ]))
      }, [
        k("header", null, [
          e.self ? (x(), C("span", _h, "\u4F60")) : z("", true),
          e.seat.turned_over ? (x(), C("span", bh, "\u7FFB\u9762")) : z("", true)
        ]),
        k("div", wh, [
          (x(true), C(de, null, xe(r.value, (f, m) => (x(), Vr(vh, {
            key: m,
            general: f,
            slot: m,
            "legacy-def": m === 0 ? i.value : void 0,
            alive: e.seat.alive
          }, null, 8, [
            "general",
            "slot",
            "legacy-def",
            "alive"
          ]))), 128))
        ]),
        k("div", Sh, [
          k("span", {
            class: "state-icon",
            style: Kt({
              color: l(e.seat.hp, e.seat.max_hp, e.seat.alive)
            }),
            "aria-label": e.seat.alive ? e.seat.hp <= 0 ? "\u6FD2\u6B7B" : "\u5B58\u6D3B" : "\u9635\u4EA1"
          }, P(o(e.seat.hp, e.seat.max_hp, e.seat.alive)), 13, kh),
          k("span", Eh, P(e.seat.hp) + " / " + P(e.seat.max_hp), 1)
        ]),
        k("div", {
          class: "hp-bar",
          role: "progressbar",
          "aria-valuenow": Math.max(e.seat.hp, 0),
          "aria-valuemax": e.seat.max_hp,
          "aria-label": `\u4F53\u529B ${e.seat.hp}/${e.seat.max_hp}`
        }, [
          k("div", {
            class: "hp-bar-fill",
            style: Kt({
              width: Math.max(Math.min(e.seat.hp / e.seat.max_hp * 100, 100), 0) + "%",
              background: e.seat.alive ? e.seat.hp <= 0 ? "var(--color-warning)" : "var(--color-positive)" : "var(--color-negative)"
            })
          }, null, 4)
        ], 8, Ah),
        k("div", xh, [
          k("span", null, "\u624B\u724C " + P(e.seat.hand_count), 1),
          k("span", null, P(e.identityLabel), 1)
        ]),
        e.equipCards.length > 0 ? (x(), C("div", Ch, [
          (x(true), C(de, null, xe(e.equipCards, (f) => (x(), C("div", {
            key: f.id,
            class: "equip-card",
            title: `\u5361\u724C #${f.id}${f.def != null ? " (def:" + f.def + ")" : ""}`
          }, [
            k("span", Ph, P(f.suit === "Spade" ? "\u2660" : f.suit === "Heart" ? "\u2665" : f.suit === "Club" ? "\u2663" : f.suit === "Diamond" ? "\u2666" : "?"), 1),
            k("span", Th, "#" + P(f.id), 1)
          ], 8, Rh))), 128))
        ])) : z("", true),
        e.seat.alive ? e.seat.hp <= 0 ? (x(), C("div", Oh, "\u26A0 \u6FD2\u6B7B")) : z("", true) : (x(), C("div", Ih, "\u2717 \u9635\u4EA1")),
        e.isTargetMode && e.isSelectableTarget ? (x(), C("div", Nh, " \u25B8 \u70B9\u51FB\u9009\u62E9\u4E3A\u76EE\u6807 ")) : z("", true)
      ], 42, yh));
    }
  }), Za = gn($h, [
    [
      "__scopeId",
      "data-v-fdcad0a8"
    ]
  ]), Mh = {
    class: "hand",
    "aria-label": "\u624B\u724C"
  }, jh = {
    class: "muted title"
  }, Dh = {
    class: "strip",
    role: "listbox",
    "aria-label": "\u624B\u724C\u5217\u8868"
  }, Lh = [
    "disabled",
    "aria-selected",
    "aria-label",
    "onClick",
    "onDragstart"
  ], Uh = [
    "data-suit"
  ], Hh = {
    class: "rank"
  }, Fh = {
    class: "id muted"
  }, qh = {
    key: 0,
    class: "card-known"
  }, Wh = {
    key: 0,
    class: "muted empty"
  }, Vh = {
    key: 0,
    class: "hint"
  }, Bh = {
    key: 1,
    class: "hint drag-hint"
  }, Kh = Dt({
    __name: "HandStrip",
    props: {
      cards: {},
      actionableIds: {}
    },
    emits: [
      "select",
      "dragStart"
    ],
    setup(e, { emit: t }) {
      const n = e, s = t, r = ye(null), i = ye(null);
      function a(f) {
        switch (f) {
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
      function o(f) {
        return f == null ? "?" : f === 1 ? "A" : f === 11 ? "J" : f === 12 ? "Q" : f === 13 ? "K" : String(f);
      }
      function l(f) {
        n.actionableIds.includes(f) && (r.value = f, s("select", f));
      }
      function c(f, m) {
        if (!n.actionableIds.includes(m)) {
          f.preventDefault();
          return;
        }
        i.value = m, f.dataTransfer && (f.dataTransfer.setData("text/plain", String(m)), f.dataTransfer.effectAllowed = "move"), s("dragStart", m);
      }
      function u() {
        i.value = null;
      }
      return (f, m) => (x(), C("section", Mh, [
        k("h2", jh, " \u624B\u724C\uFF08" + P(e.cards.length) + " \u5F20" + P(r.value != null ? " \xB7 \u5DF2\u9009\u4E2D #" + r.value : " \xB7 \u70B9\u51FB\u6216\u62D6\u62FD") + "\uFF09 ", 1),
        k("div", Dh, [
          (x(true), C(de, null, xe(e.cards, (y) => (x(), C("button", {
            key: y.id,
            type: "button",
            class: Fe([
              "card",
              {
                selected: y.id === r.value,
                dragging: y.id === i.value
              }
            ]),
            disabled: !e.actionableIds.includes(y.id),
            role: "option",
            "aria-selected": y.id === r.value,
            "aria-label": `\u5361\u724C ${y.id}${y.def != null ? " \u5DF2\u77E5" : " \u672A\u77E5"}`,
            draggable: "true",
            onClick: (O) => l(y.id),
            onDragstart: (O) => c(O, y.id),
            onDragend: u
          }, [
            k("span", {
              class: "suit",
              "data-suit": y.suit
            }, P(a(y.suit)), 9, Uh),
            k("span", Hh, P(o(y.rank)), 1),
            k("span", Fh, "#" + P(y.id), 1),
            y.def != null ? (x(), C("span", qh, "\u25B8")) : z("", true)
          ], 42, Lh))), 128)),
          e.cards.length ? z("", true) : (x(), C("p", Wh, "\u65E0\u53EF\u89C1\u624B\u724C"))
        ]),
        r.value != null ? (x(), C("p", Vh, " \u2713 \u5DF2\u9009\u4E2D #" + P(r.value) + " \u2014 \u5728\u68CB\u76D8\u4E0A\u9009\u62E9\u76EE\u6807\uFF0C\u6216\u70B9\u51FB\u300C\u7ED3\u675F\u9636\u6BB5\u300D\u76F4\u63A5\u6253\u51FA ", 1)) : z("", true),
        i.value != null ? (x(), C("p", Bh, " \u{1F0A0} \u62D6\u62FD\u4E2D #" + P(i.value) + " \u2014 \u62D6\u5230\u76EE\u6807\u73A9\u5BB6\u91CA\u653E\u51FA\u724C ", 1)) : z("", true)
      ]));
    }
  }), Gh = gn(Kh, [
    [
      "__scopeId",
      "data-v-b2b16095"
    ]
  ]), Jh = {
    class: "board",
    role: "region",
    "aria-label": "\u6218\u573A"
  }, zh = {
    key: 0,
    class: "action-bar panel",
    role: "toolbar",
    "aria-label": "\u884C\u52A8\u64CD\u4F5C"
  }, Yh = {
    class: "muted"
  }, Qh = {
    key: 2
  }, Xh = {
    key: 3
  }, Zh = {
    class: "row"
  }, ep = [
    "disabled"
  ], tp = [
    "disabled"
  ], np = {
    key: 1,
    class: "skill-bar panel",
    "aria-label": "\u53EF\u53D1\u52A8\u6280\u80FD"
  }, sp = [
    "onClick"
  ], rp = {
    class: "seats",
    role: "list",
    "aria-label": "\u5EA7\u4F4D\u5217\u8868"
  }, ip = [
    "onDragover",
    "onDragleave",
    "onDrop"
  ], op = {
    key: 2,
    class: "judgment-zone",
    "aria-label": "\u5224\u5B9A\u533A"
  }, ap = {
    class: "jz-entries"
  }, lp = [
    "title"
  ], cp = {
    class: "jz-suit"
  }, up = {
    class: "jz-player"
  }, fp = {
    key: 3,
    class: "discard-pile",
    "aria-label": "\u5F03\u724C\u5806"
  }, dp = {
    class: "muted discard-title"
  }, hp = {
    class: "discard-strip"
  }, pp = [
    "title"
  ], gp = {
    class: "d-suit"
  }, mp = {
    key: 0,
    class: "muted discard-more"
  }, vp = Dt({
    __name: "BattleBoard",
    props: {
      view: {},
      selfId: {},
      legalActions: {}
    },
    emits: [
      "playCard",
      "activateSkill"
    ],
    setup(e, { emit: t }) {
      const n = e, s = t, r = ye(null), i = ye(null), a = ye(false), o = ye([]), l = ye(null), c = ye(null);
      function u() {
        var _a2;
        return ((_a2 = n.view) == null ? void 0 : _a2.seats) ?? [];
      }
      function f() {
        return n.view ? n.view.known_cards.filter((I) => I.owner === n.selfId && I.zone === "Hand" && I.def != null) : [];
      }
      function m() {
        var _a2;
        const I = new Set(n.legalActions.flatMap((M) => typeof M == "object" && "ActivateSkill" in M ? [
          M.ActivateSkill.skill
        ] : []));
        return ((_a2 = u().find((M) => M.player === n.selfId)) == null ? void 0 : _a2.skills.filter((M) => M.enabled && (M.active || M.view_as) && I.has(M.id))) ?? [];
      }
      function y(I) {
        switch (I) {
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
      function O() {
        return a.value ? new Set(X().flat()) : /* @__PURE__ */ new Set();
      }
      function T() {
        return n.view ? n.view.known_cards.filter((I) => I.zone === "DiscardPile" && I.def != null) : [];
      }
      function D(I) {
        return n.view ? n.view.known_cards.filter((M) => M.owner === I && M.zone === "Judge" && M.def != null) : [];
      }
      const L = Q(() => O()), E = Q(() => i.value ? Array.from(new Set($(i.value.id).flatMap((I) => I.card_targets))) : Array.from(new Set(H().map((I) => I.card))));
      function H(I) {
        return n.legalActions.flatMap((M) => typeof M != "object" || !("PlayCard" in M) ? [] : I != null && M.PlayCard.card !== I ? [] : [
          M.PlayCard
        ]);
      }
      function $(I) {
        return n.legalActions.flatMap((M) => typeof M != "object" || !("ActivateSkill" in M) ? [] : M.ActivateSkill.skill === I ? [
          M.ActivateSkill
        ] : []);
      }
      function X() {
        return i.value ? $(i.value.id).filter((I) => r.value == null ? I.card_targets.length === 0 : I.card_targets.length === 1 && I.card_targets[0] === r.value).map((I) => I.targets) : r.value == null ? [] : H(r.value).map((I) => I.targets);
      }
      function se(I, M) {
        return I.length === M.length && I.every((F, ie) => F === M[ie]);
      }
      const re = Q(() => X().some((I) => se(I, o.value))), $e = Q(() => X().some((I) => I.length === 0));
      function Me(I) {
        return n.view ? n.view.known_cards.filter((M) => M.owner === I && M.zone === "Equip" && M.def != null) : [];
      }
      function je(I) {
        if (E.value.includes(I)) {
          if (r.value === I) {
            r.value = null, a.value = false, o.value = [];
            return;
          }
          r.value = I, i.value || (i.value = null), a.value = true, o.value = [];
        }
      }
      function De(I) {
        var _a2;
        if (((_a2 = i.value) == null ? void 0 : _a2.id) === I.id) {
          Pe();
          return;
        }
        i.value = I, r.value = null, a.value = true, o.value = [];
      }
      function Ye(I) {
        E.value.includes(I) && (l.value = I, r.value = I, a.value = true);
      }
      function Le(I) {
        if (!(!a.value || !L.value.has(I))) if (o.value.includes(I)) o.value = o.value.filter((M) => M !== I);
        else {
          const M = Math.max(0, ...X().map((F) => F.length));
          o.value = [
            ...o.value,
            I
          ].slice(0, M);
        }
      }
      function yt(I, M) {
        l.value != null && L.value.has(M) && (I.preventDefault(), I.dataTransfer.dropEffect = "move", c.value = M);
      }
      function st(I) {
        c.value === I && (c.value = null);
      }
      function ge(I, M) {
        var _a2;
        I.preventDefault();
        const F = l.value;
        if (c.value = null, l.value = null, F != null && L.value.has(M)) {
          if ((_a2 = i.value) == null ? void 0 : _a2.view_as) {
            const ie = $(i.value.id).find((Te) => se(Te.targets, [
              M
            ]) && Te.card_targets.length === 1 && Te.card_targets[0] === F);
            if (!ie) return;
            s("activateSkill", ie.skill, ie.targets, ie.card_targets);
          } else {
            const ie = H(F).find((Te) => se(Te.targets, [
              M
            ]));
            if (!ie) return;
            s("playCard", ie.card, ie.targets);
          }
          r.value = null, i.value = null, a.value = false, o.value = [];
        }
      }
      function te() {
        if (re.value) {
          if (i.value) {
            const I = $(i.value.id).find((M) => se(M.targets, o.value) && (r.value == null ? M.card_targets.length === 0 : M.card_targets.length === 1 && M.card_targets[0] === r.value));
            if (!I) return;
            s("activateSkill", I.skill, I.targets, I.card_targets);
          } else {
            if (r.value == null) return;
            const I = H(r.value).find((M) => se(M.targets, o.value));
            if (!I) return;
            s("playCard", I.card, I.targets);
          }
          r.value = null, i.value = null, a.value = false, o.value = [];
        }
      }
      function ee() {
        o.value = [], te();
      }
      function Pe() {
        r.value = null, i.value = null, a.value = false, o.value = [];
      }
      return (I, M) => {
        var _a2;
        return x(), C("div", Jh, [
          r.value != null || i.value ? (x(), C("div", zh, [
            k("span", Yh, [
              M[0] || (M[0] = xt(" \u25B8 ", -1)),
              i.value ? (x(), C(de, {
                key: 0
              }, [
                xt("\u6280\u80FD #" + P(i.value.def), 1)
              ], 64)) : (x(), C(de, {
                key: 1
              }, [
                xt("\u624B\u724C #" + P(r.value), 1)
              ], 64)),
              ((_a2 = i.value) == null ? void 0 : _a2.view_as) && r.value != null ? (x(), C("span", Qh, " \xB7 \u7D20\u6750 #" + P(r.value), 1)) : z("", true),
              a.value && o.value.length > 0 ? (x(), C("span", Xh, " \u2192 \u76EE\u6807 P" + P(o.value[0]), 1)) : z("", true)
            ]),
            k("div", Zh, [
              k("button", {
                class: "primary",
                type: "button",
                onClick: te,
                disabled: !re.value,
                "aria-label": "\u786E\u8BA4\u884C\u52A8"
              }, P(i.value ? "\u53D1\u52A8\u6280\u80FD" : "\u786E\u8BA4\u51FA\u724C"), 9, ep),
              k("button", {
                type: "button",
                disabled: !$e.value,
                onClick: ee,
                "aria-label": "\u65E0\u76EE\u6807\u76F4\u63A5\u6253\u51FA"
              }, P(i.value ? "\u65E0\u76EE\u6807\u53D1\u52A8" : "\u76F4\u63A5\u6253\u51FA"), 9, tp),
              k("button", {
                type: "button",
                onClick: Pe,
                "aria-label": "\u53D6\u6D88\u9009\u62E9"
              }, " \u53D6\u6D88 ")
            ])
          ])) : z("", true),
          m().length ? (x(), C("section", np, [
            M[1] || (M[1] = k("span", {
              class: "muted"
            }, "\u6280\u80FD", -1)),
            (x(true), C(de, null, xe(m(), (F) => {
              var _a3;
              return x(), C("button", {
                key: F.id,
                type: "button",
                class: Fe({
                  primary: ((_a3 = i.value) == null ? void 0 : _a3.id) === F.id
                }),
                onClick: (ie) => De(F)
              }, " #" + P(F.def) + P(F.view_as ? " \xB7 \u89C6\u4E3A" : ""), 11, sp);
            }), 128))
          ])) : z("", true),
          k("div", rp, [
            (x(true), C(de, null, xe(u(), (F) => (x(), C("div", {
              key: F.player,
              class: "seat-wrapper",
              onDragover: (ie) => yt(ie, F.player),
              onDragleave: (ie) => st(F.player),
              onDrop: (ie) => ge(ie, F.player)
            }, [
              ke(Za, {
                seat: F,
                self: F.player === e.selfId,
                "identity-label": y(F.identity),
                "is-target-mode": a.value,
                "is-selectable-target": L.value.has(F.player),
                "is-drop-target": c.value === F.player,
                "equip-cards": Me(F.player),
                onSelect: Le
              }, null, 8, [
                "seat",
                "self",
                "identity-label",
                "is-target-mode",
                "is-selectable-target",
                "is-drop-target",
                "equip-cards"
              ])
            ], 40, ip))), 128))
          ]),
          ke(Gh, {
            cards: f(),
            "actionable-ids": E.value,
            onSelect: je,
            onDragStart: Ye
          }, null, 8, [
            "cards",
            "actionable-ids"
          ]),
          u().some((F) => D(F.player).length > 0) ? (x(), C("section", op, [
            M[2] || (M[2] = k("h3", {
              class: "muted jz-title"
            }, "\u2696 \u5224\u5B9A\u533A", -1)),
            k("div", ap, [
              (x(true), C(de, null, xe(u(), (F) => (x(), C("div", {
                key: "jz-" + F.player,
                class: "jz-entry"
              }, [
                (x(true), C(de, null, xe(D(F.player), (ie) => (x(), C("span", {
                  key: ie.id,
                  class: "jz-card",
                  title: `P${F.player} #${ie.id}`
                }, [
                  k("span", cp, P(ie.suit === "Spade" ? "\u2660" : ie.suit === "Heart" ? "\u2665" : ie.suit === "Club" ? "\u2663" : ie.suit === "Diamond" ? "\u2666" : "?"), 1),
                  k("span", up, "P" + P(F.player), 1)
                ], 8, lp))), 128))
              ]))), 128))
            ])
          ])) : z("", true),
          T().length > 0 ? (x(), C("section", fp, [
            k("h3", dp, "\u5F03\u724C\u5806\uFF08" + P(T().length) + "\uFF09", 1),
            k("div", hp, [
              (x(true), C(de, null, xe(T().slice(-10), (F) => (x(), C("div", {
                key: F.id,
                class: "discard-card",
                title: `#${F.id} def:${F.def}`
              }, [
                k("span", gp, P(F.suit === "Spade" ? "\u2660" : F.suit === "Heart" ? "\u2665" : F.suit === "Club" ? "\u2663" : F.suit === "Diamond" ? "\u2666" : "?"), 1)
              ], 8, pp))), 128)),
              T().length > 10 ? (x(), C("span", mp, "+" + P(T().length - 10), 1)) : z("", true)
            ])
          ])) : z("", true)
        ]);
      };
    }
  }), yp = gn(vp, [
    [
      "__scopeId",
      "data-v-9182a2db"
    ]
  ]), _p = {
    class: "panel general-select",
    "aria-label": "\u9009\u5C06\u9762\u677F"
  }, bp = {
    class: "gs-head"
  }, wp = {
    class: "muted tiny"
  }, Sp = [
    "disabled"
  ], kp = {
    key: 0,
    class: "muted",
    role: "status"
  }, Ep = {
    key: 1,
    class: "muted empty",
    role: "status"
  }, Ap = {
    key: 2,
    class: "error",
    role: "alert"
  }, xp = {
    key: 3,
    class: "success",
    role: "status"
  }, Cp = {
    class: "muted"
  }, Rp = {
    class: "muted"
  }, Pp = {
    key: 4,
    class: "gs-grid"
  }, Tp = [
    "disabled",
    "aria-pressed",
    "aria-label",
    "onClick"
  ], Ip = {
    class: "candidate-name"
  }, Op = {
    class: "candidate-skills"
  }, Np = {
    class: "candidate-def muted"
  }, $p = Dt({
    __name: "GeneralSelectPanel",
    props: {
      state: {},
      busy: {
        type: Boolean
      }
    },
    emits: [
      "choose",
      "reload"
    ],
    setup(e, { emit: t }) {
      const n = e, s = t, r = Q(() => n.busy || n.state.selectionStatus === "pending"), i = Q(() => {
        var _a2;
        return ((_a2 = n.state.assignment) == null ? void 0 : _a2.name) ?? null;
      }), a = Q(() => n.state.projection.skillNames.length > 0 ? n.state.projection.skillNames.join(" \xB7 ") : "\u89C6\u56FE\u672A\u66B4\u9732\u6280\u80FD\u540D"), o = Q(() => {
        switch (n.state.loadStatus) {
          case "loading":
            return "\u8BFB\u53D6\u5019\u9009\u2026";
          case "error":
            return "\u5019\u9009\u4E0D\u53EF\u7528";
          case "empty":
            return "\u5019\u9009\u4E3A\u7A7A";
          default:
            return `${n.state.candidates.length} \u9879\u5019\u9009`;
        }
      });
      return (l, c) => (x(), C("section", _p, [
        k("div", bp, [
          k("div", null, [
            c[1] || (c[1] = k("h2", null, "\u9009\u5C06", -1)),
            k("p", wp, " NC05 \xB7 \u76EE\u6807\u5EA7\u4F4D P" + P(e.state.seat) + " \xB7 " + P(o.value) + " \xB7 \u70B9\u51FB\u5019\u9009\u5373\u6309\u540D\u88C5\u914D \uFF08\u539F\u751F\u6B63\u5F0F\u5165\u53E3\uFF0C\u975E debug\uFF09 ", 1)
          ]),
          k("button", {
            type: "button",
            disabled: r.value || e.state.loadStatus === "loading",
            "aria-label": "\u5237\u65B0\u9009\u5C06\u5019\u9009",
            onClick: c[0] || (c[0] = (u) => s("reload"))
          }, " \u5237\u65B0\u5019\u9009 ", 8, Sp)
        ]),
        e.state.loadStatus === "loading" && e.state.candidates.length === 0 ? (x(), C("p", kp, " \u6B63\u5728\u8BFB\u53D6\u539F\u751F\u76EE\u5F55\u5019\u9009\u2026 ")) : e.state.loadStatus === "empty" ? (x(), C("p", Ep, " \u5019\u9009\u4E3A\u7A7A\uFF1A\u5185\u6838\u76EE\u5F55\u6CA1\u6709\u8FD4\u56DE\u4EFB\u4F55\u6B66\u5C06\uFF08\u8FD9\u662F\u5408\u6CD5\u7684\u7A7A\u76EE\u5F55\uFF0C\u4E0D\u662F\u52A0\u8F7D\u5931\u8D25\uFF09\u3002 ")) : z("", true),
        e.state.error ? (x(), C("p", Ap, P(e.state.error), 1)) : z("", true),
        e.state.selectionStatus === "accepted" && e.state.assignment ? (x(), C("p", xp, [
          xt(" \u2705 \u5DF2\u4E3A P" + P(e.state.assignment.seat) + " \u88C5\u914D ", 1),
          k("strong", null, P(e.state.assignment.name), 1),
          k("span", Cp, " \xB7 state_hash " + P(e.state.assignment.state_hash.slice(0, 16)) + "\u2026 ", 1),
          c[2] || (c[2] = k("br", null, null, -1)),
          k("span", Rp, "\u5237\u65B0\u540E\u89C6\u56FE\u6280\u80FD\uFF1A" + P(a.value), 1)
        ])) : z("", true),
        e.state.candidates.length > 0 ? (x(), C("ul", Pp, [
          (x(true), C(de, null, xe(e.state.candidates, (u) => (x(), C("li", {
            key: u.def
          }, [
            k("button", {
              type: "button",
              class: Fe([
                "candidate",
                {
                  selected: i.value === u.name
                }
              ]),
              disabled: r.value,
              "aria-pressed": i.value === u.name,
              "aria-label": `\u9009\u62E9 ${u.name}\uFF08${u.skillLabel}\uFF09`,
              onClick: (f) => s("choose", u.name)
            }, [
              k("span", Ip, P(u.name), 1),
              k("span", Op, P(u.skillLabel), 1),
              k("span", Np, "#" + P(u.def), 1)
            ], 10, Tp)
          ]))), 128))
        ])) : z("", true)
      ]));
    }
  }), Mp = gn($p, [
    [
      "__scopeId",
      "data-v-9fa84d6d"
    ]
  ]), jp = {
    class: "battle stack"
  }, Dp = {
    key: 0,
    class: "panel hotkey-overlay",
    role: "dialog",
    "aria-label": "\u5FEB\u6377\u952E"
  }, Lp = {
    class: "outcome-content"
  }, Up = {
    class: "outcome-icon"
  }, Hp = {
    class: "outcome-label"
  }, Fp = {
    class: "muted"
  }, qp = {
    key: 2,
    class: "panel turn-bar",
    role: "status",
    "aria-live": "polite"
  }, Wp = {
    class: "turn-player"
  }, Vp = [
    "data-phase"
  ], Bp = {
    class: "panel toolbar",
    "aria-label": "\u5BF9\u5C40\u63A7\u5236"
  }, Kp = {
    class: "row"
  }, Gp = [
    "disabled"
  ], Jp = {
    key: 1,
    class: "online-badge"
  }, zp = [
    "disabled"
  ], Yp = [
    "disabled"
  ], Qp = [
    "disabled"
  ], Xp = [
    "disabled"
  ], Zp = [
    "disabled"
  ], eg = [
    "disabled"
  ], tg = {
    class: "row muted controls"
  }, ng = {
    class: "check"
  }, sg = {
    class: "status",
    role: "status",
    "aria-live": "polite"
  }, rg = {
    key: 0,
    class: "error",
    role: "alert"
  }, ig = {
    key: 4,
    class: "panel online-roster",
    "aria-label": "\u8054\u673A\u623F\u95F4\u5EA7\u4F4D"
  }, og = {
    class: "row online-roster-head"
  }, ag = {
    class: "row online-seats"
  }, lg = {
    key: 5,
    class: "panel choice",
    "aria-label": "\u5F85\u9009\u62E9"
  }, cg = {
    class: "muted"
  }, ug = {
    key: 0,
    class: "row"
  }, fg = [
    "disabled",
    "onClick"
  ], dg = {
    key: 1,
    class: "muted"
  }, hg = {
    key: 6,
    class: "panel roster",
    "aria-label": "\u73A9\u5BB6\u6982\u51B5"
  }, pg = {
    class: "roster-grid"
  }, gg = {
    class: "roster-player"
  }, mg = {
    class: "roster-hand muted"
  }, vg = [
    "data-id"
  ], yg = {
    key: 1,
    class: "roster-id muted"
  }, _g = {
    class: "panel battle-section"
  }, bg = {
    key: 0,
    class: "effect-overlay",
    "aria-hidden": "true"
  }, wg = {
    class: "effect-icon"
  }, Sg = {
    class: "effect-text"
  }, kg = {
    key: 0,
    class: "effect-player"
  }, Eg = {
    key: 7,
    class: "panel match-stats",
    "aria-label": "\u5BF9\u5C40\u7EDF\u8BA1"
  }, Ag = {
    class: "stats-grid"
  }, xg = {
    class: "stat"
  }, Cg = {
    class: "stat-value mono"
  }, Rg = {
    class: "stat"
  }, Pg = {
    class: "stat-value mono"
  }, Tg = {
    class: "stat"
  }, Ig = {
    class: "stat-value"
  }, Og = {
    class: "stat"
  }, Ng = {
    class: "stat-value"
  }, $g = {
    key: 8,
    class: "panel seat-status",
    "aria-label": "\u5EA7\u4F4D\u72B6\u6001\u4E00\u89C8"
  }, Mg = {
    class: "seat-status-grid"
  }, jg = {
    class: "ss-header"
  }, Dg = {
    class: "ss-player"
  }, Lg = {
    key: 0,
    class: "ss-tag"
  }, Ug = {
    class: "ss-hp-bar"
  }, Hg = {
    class: "ss-hp-text"
  }, Fg = {
    class: "ss-icon"
  }, qg = {
    class: "ss-meta muted"
  }, Wg = [
    "data-id"
  ], Vg = {
    key: 1,
    class: "muted"
  }, Bg = {
    class: "panel log",
    "aria-label": "\u4E8B\u4EF6\u65E5\u5FD7"
  }, Kg = {
    class: "log-header"
  }, Gg = {
    class: "log-list"
  }, Jg = {
    class: "log-rev"
  }, zg = {
    class: "log-text"
  }, Yg = {
    key: 0,
    class: "muted empty-log"
  }, Qg = Dt({
    __name: "BattleView",
    setup(e) {
      var _a2;
      const t = Mf(), n = $f(), s = zd(), r = new Ms();
      let i;
      const a = new Fd(), o = ye(1);
      try {
        ((_a2 = globalThis.localStorage) == null ? void 0 : _a2.getItem("noname-next-skip-anims")) === "1" && (s.skipAnims = true);
      } catch {
      }
      if (t.query.skipAnims === "1" && (s.skipAnims = true), t.query.speed !== void 0) {
        const g = Number(t.query.speed);
        Number.isFinite(g) && g > 0 && (s.speed = g);
      }
      const l = ye(Number(t.query.players ?? 4) || 4), c = ye(Number(t.query.seed ?? 42) || 42), u = ye(null), f = ye(false), m = ye("\u672A\u8FDE\u63A5"), y = ye(null), O = ye(0), T = ye(false), D = Q(() => t.query.mode === "online"), L = Q(() => D.value && o.value === 0), E = ye(sh(o.value)), H = Q(() => !D.value && s.connected && r.isNative());
      async function $(g) {
        s.busy = true, s.error = null;
        try {
          return await g();
        } catch (p) {
          s.error = p instanceof Error ? p.message : String(p), m.value = s.error;
          return;
        } finally {
          s.busy = false;
        }
      }
      function X(g) {
        globalThis.__nativeStartMarks ?? (globalThis.__nativeStartMarks = {}), globalThis.__nativeStartMarks[g] = Math.round(performance.now() * 100) / 100;
      }
      async function se(g) {
        const p = performance.now();
        ho(s, g);
        const w = performance.now();
        X("t5_apply_envelope"), await eh(s, 120);
        const v = performance.now();
        if (globalThis.__nativeEnvelopeTimings ?? (globalThis.__nativeEnvelopeTimings = []), globalThis.__nativeEnvelopeTimings.push({
          apply_ms: Math.round((w - p) * 100) / 100,
          delay_ms: Math.round((v - w) * 100) / 100
        }), D.value && (!s.connected || s.revision !== g.revision || s.hash !== g.hash_hex)) return;
        Xd(s);
        const R = D.value ? `\u623F\u95F4 #${O.value} \xB7 ` : "", A = (g.hash_hex || "").slice(0, 12);
        m.value = g.finished ? `${R}\u7EC8\u5C40 \xB7 ${h.value.label} \xB7 rev ${g.revision}` : `${R}rev ${g.revision}${A ? ` \xB7 ${A}\u2026` : ""}${r.isNative() ? " \xB7 native" : ""}`, $e();
      }
      async function re(g) {
        if (await se(g), !D.value && r.isNative()) {
          const p = Gd(g.result);
          if (p) throw new Error(p);
        }
      }
      function $e() {
        E.value = Xa(E.value, Qa(s.view, E.value.seat));
      }
      async function Me() {
        if (!(D.value || !r.isNative())) {
          E.value = rh(E.value);
          try {
            const g = await r.generalCandidates();
            E.value = ih(E.value, g);
          } catch (g) {
            E.value = oh(E.value, g);
          }
        }
      }
      async function je(g) {
        if (D.value || !r.isNative() || s.busy || E.value.selectionStatus === "pending") return;
        const p = E.value;
        await $(async () => {
          if (E.value = await lh(p, g, {
            chooseGeneral: (w, v) => r.chooseGeneral(w, v),
            applyEnvelope: (w) => {
              ho(s, w), Ge(s, `\u9009\u5C06 P${p.seat} \u2192 ${g}`);
            }
          }, (w) => {
            E.value = w;
          }), E.value.selectionStatus === "accepted") {
            const w = E.value.lastStateHash ?? "";
            m.value = `\u9009\u5C06 P${p.seat} \xB7 ${g}${w ? ` \xB7 ${w.slice(0, 12)}\u2026` : ""}`;
          } else E.value.selectionStatus === "rejected" && (m.value = E.value.error ?? "\u9009\u5C06\u5931\u8D25");
        });
      }
      async function De() {
        await $(async () => {
          try {
            if (await Xi(true), co(true), D.value) {
              const v = te();
              _d(v.wsUrl), await yt(v);
              return;
            }
            yd(), X("t0_click"), await r.connect(), X("t1_worker_ready"), s.connected = true, m.value = r.isNative() ? "\u539F\u751F Worker \u5C31\u7EEA\uFF0C\u5F00\u5C40\u4E2D\u2026" : "Worker \u5C31\u7EEA\uFF0C\u5F00\u5C40\u4E2D\u2026", r.isNative() && (o.value = Math.max(0, Math.min(o.value, l.value - 1)), E.value = po(E.value, o.value));
            const g = $d();
            if (!r.isNative() && g.contentLock.length) throw new Error("\u5DF2\u5B89\u88C5\u6269\u5C55\u7684\u6B63\u5F0F\u5F00\u5C40\u9700\u8981\u539F\u751F\u540E\u7AEF");
            const p = await r.start(l.value, c.value, o.value, g.packsJson, t.query.mode === "identity" ? "identity" : void 0);
            i = g, X("t2_start_done");
            const w = (r.isNative() ? Me() : Promise.resolve()).then((v) => (X("t3_candidates_loaded"), v));
            await se(p), await pn(), X("t4_first_frame"), await w, Ge(s, `\u5F00\u5C40 ${l.value} \u4EBA seed=${c.value}${g.contentLock.length ? ` \xB7 \u5185\u5BB9\u9501 ${g.contentLock.length} \u5305` : ""}`);
          } catch (g) {
            throw s.connected = false, Z(), g;
          }
        });
      }
      async function Ye() {
        await $(async () => {
          if (D.value) throw new Error("\u8054\u673A\u6A21\u5F0F\u8BF7\u4F7F\u7528\u670D\u52A1\u7AEF\u5B58\u6863");
          if (!r.isNative()) throw new Error("\u5B58\u6863\u4EC5\u539F\u751F\u540E\u7AEF\u652F\u6301");
          if (!i) throw new Error("\u5C1A\u672A\u5EFA\u7ACB\u5BF9\u5C40\u5185\u5BB9\u9501");
          const g = await r.saveHex(), p = `save-${Date.now()}`;
          Nd({
            id: p,
            label: `rev${s.revision} \xB7 ${l.value}p seed=${c.value}`,
            hex: g,
            seed: c.value,
            players: l.value,
            ...i,
            savedAt: (/* @__PURE__ */ new Date()).toISOString(),
            revision: s.revision
          }), Ge(s, `\u5DF2\u5B58\u6863 ${p}\uFF08${g.length / 2} \u5B57\u8282\uFF09`), m.value = `\u5DF2\u5B58\u6863 \xB7 ${p}`;
        });
      }
      async function Le() {
        await $(async () => {
          if (D.value) throw new Error("\u8054\u673A\u6A21\u5F0F\u4E0D\u652F\u6301\u672C\u5730\u8BFB\u6863");
          if (!r.isNative()) throw new Error("\u8BFB\u6863\u4EC5\u539F\u751F\u540E\u7AEF\u652F\u6301");
          const p = Ka()[0];
          if (!p) throw new Error("\u65E0\u672C\u5730\u5B58\u6863");
          const w = Md(p), v = await r.restoreHex(p.hex, w.packsJson);
          i = w, l.value = p.players, c.value = p.seed, E.value = po(E.value, E.value.seat), await se(v), await Me(), Ge(s, `\u8BFB\u6863 ${p.id} \xB7 rev ${p.revision}`);
        });
      }
      async function yt(g) {
        a.onEnvelope((w) => {
          se(w);
        }), a.onClose(() => {
          m.value = "\u8FDE\u63A5\u5DF2\u65AD\u5F00\uFF08\u670D\u52A1\u7AEF\u4E0D\u53EF\u8FBE\uFF09", s.connected = false, Ge(s, "\u8FDE\u63A5\u5DF2\u65AD\u5F00\uFF1A\u670D\u52A1\u7AEF\u4E0D\u53EF\u8FBE\u6216\u5DF2\u505C\u6B62");
        }), a.onReconnect((w) => {
          w === "reconnecting" ? (m.value = "\u8FDE\u63A5\u5DF2\u65AD\u5F00\uFF0C\u6B63\u5728\u91CD\u8FDE\u2026", s.connected = false) : w === "reconnected" ? (m.value = "\u5DF2\u91CD\u8FDE\uFF08\u7B49\u5F85\u623F\u95F4\u72B6\u6001\uFF09", s.connected = false, a.rejoinRoom().then((v) => (o.value = v.playerId, O.value = v.roomId, ge(g, v), s.connected = true, m.value = `\u623F\u95F4 #${v.roomId} \xB7 \u5DF2\u91CD\u8FDE`, se(v.envelope))).catch((v) => {
            s.connected = false;
            const R = v instanceof Error ? v.message : String(v);
            m.value = `\u91CD\u8FDE\u540E\u56DE\u623F\u5931\u8D25\uFF1A${R}`, Ge(s, `\u91CD\u8FDE\u540E\u56DE\u623F\u5931\u8D25\uFF1A${R}`);
          })) : (m.value = "\u91CD\u8FDE\u5931\u8D25\uFF08\u670D\u52A1\u7AEF\u4E0D\u53EF\u8FBE\uFF09", s.connected = false);
        }), a.onRoster((w) => {
          y.value = w, O.value = w.room_id;
          const v = w.started && !T.value;
          T.value = w.started, m.value = w.started ? `\u623F\u95F4 #${w.room_id} \xB7 \u5DF2\u5F00\u5C40` : `\u623F\u95F4 #${w.room_id} \xB7 \u7B49\u5F85\u5165\u5EA7 ${w.seats.length}/${l.value}`, v && a.requestSnapshot().catch(() => {
          });
        }), m.value = `\u8FDE\u63A5 ${g.wsUrl}\u2026`, await a.connect(g.wsUrl, g.displayName);
        let p;
        if (g.action === "create") p = await a.createRoom(g.players, g.seed, g.password);
        else if (g.action === "spectate") p = await a.spectate(g.roomId);
        else {
          const w = localStorage.getItem(fo(g.wsUrl, g.roomId));
          p = await a.joinRoom(g.roomId, g.password, w);
        }
        O.value = p.roomId, o.value = p.playerId, l.value = p.view.seats.length, ge(g, p), s.connected = true, await se(p.envelope), Ge(s, p.role === "Spectator" ? `\u65C1\u89C2\u623F\u95F4 #${p.roomId}` : `\u52A0\u5165\u623F\u95F4 #${p.roomId} \xB7 \u5EA7\u4F4D ${p.seat + 1}`);
      }
      function st() {
        a.cancelPendingCommands(), m.value = "\u5DF2\u53D6\u6D88\u5F85\u786E\u8BA4\u547D\u4EE4\uFF08\u670D\u52A1\u7AEF\u53EF\u80FD\u5DF2\u63D0\u4EA4\uFF0C\u4EE5\u6743\u5A01\u72B6\u6001\u4E3A\u51C6\uFF09", Ge(s, "\u5DF2\u53D6\u6D88\u5F85\u786E\u8BA4\u547D\u4EE4\uFF1B\u7ED3\u679C\u53EF\u80FD\u5DF2\u63D0\u4EA4\uFF0C\u7B49\u5F85/\u5237\u65B0\u540E\u4EE5\u670D\u52A1\u7AEF\u72B6\u6001\u4E3A\u51C6");
      }
      function ge(g, p) {
        p.role === "Player" && p.reconnectToken && localStorage.setItem(fo(g.wsUrl, p.roomId), p.reconnectToken), sessionStorage.setItem("noname-next.online-intent", JSON.stringify({
          ...g,
          action: p.role === "Spectator" ? "spectate" : "join",
          roomId: p.roomId
        }));
      }
      function te() {
        const g = sessionStorage.getItem("noname-next.online-intent");
        if (g) try {
          return JSON.parse(g);
        } catch {
          sessionStorage.removeItem("noname-next.online-intent");
        }
        const p = window.location.protocol === "https:";
        return {
          action: String(t.query.action ?? "join"),
          wsUrl: `${p ? "wss" : "ws"}://${window.location.hostname || "127.0.0.1"}:7700/ws`,
          displayName: "player",
          password: "",
          roomId: Number(t.query.room ?? 1) || 1,
          players: l.value,
          seed: c.value
        };
      }
      function ee() {
        return D.value ? a : r;
      }
      async function Pe() {
        await $(async () => {
          const g = await ee().endPhase(o.value);
          await re(g);
        });
      }
      async function I() {
        await $(async () => {
          if (D.value) throw new Error("\u8054\u673A\u6A21\u5F0F\u4E0D\u63D0\u4F9B\u5BA2\u6237\u7AEF\u81EA\u52A8\u6B65\u8FDB");
          const p = await r.autoStep();
          return await re(p), true;
        }) || S();
      }
      async function M(g, p) {
        await $(async () => {
          const w = await ee().playCard(o.value, g, p);
          await re(w), Ge(s, `\u51FA\u724C #${g}${p.length > 0 ? ` \u2192 P${p[0]}` : ""}`);
        });
      }
      async function F(g, p, w) {
        await $(async () => {
          const v = await ee().activateSkill(o.value, g, p, w);
          await re(v), Ge(s, `\u53D1\u52A8\u6280\u80FD #${g}${p.length > 0 ? ` \u2192 P${p[0]}` : ""}`);
        });
      }
      async function ie(g) {
        await $(async () => {
          const p = await ee().resolveChoice(o.value, g.ResolveChoice.choice_id, g.ResolveChoice.selections, g.ResolveChoice.cancelled);
          await re(p);
        });
      }
      const Te = Q(() => s.legalActions.filter((g) => typeof g == "object" && "ResolveChoice" in g)), rt = Q(() => s.connected && s.legalActions.includes("EndPhase"));
      function Ue() {
        if (f.value) {
          S();
          return;
        }
        f.value = true;
        const g = async () => {
          if (!f.value || s.finished) {
            S();
            return;
          }
          if (await I(), f.value && !s.finished) {
            const p = s.skipAnims || s.reduceMotion ? 30 : 280 / s.speed;
            u.value = window.setTimeout(g, p);
          } else S();
        };
        g();
      }
      function S() {
        f.value = false, u.value != null && (clearTimeout(u.value), u.value = null);
      }
      function U(g) {
        return g.includes("\u4F24\u5BB3") || g.includes("\u9635\u4EA1") ? "log-damage" : g.includes("\u56DE\u590D") || g.includes("\u6478\u724C") ? "log-positive" : g.includes("\u6FD2\u6B7B") || g.includes("\u7B49\u5F85") ? "log-warning" : g.includes("\u56DE\u5408") || g.includes("\u4F7F\u7528") ? "log-action" : "";
      }
      const N = ye(false);
      function q(g) {
        if (g.key === "?") {
          N.value = !N.value;
          return;
        }
        if (g.key === "Escape") {
          N.value = false;
          return;
        }
        s.busy || s.finished || !s.connected || (g.key === " " ? (g.preventDefault(), D.value || I()) : g.key === "e" || g.key === "E" ? rt.value && Pe() : (g.key === "s" || g.key === "S") && (s.skipAnims = !s.skipAnims));
      }
      function Z() {
        S(), r.dispose(), a.dispose(), i = void 0, Xi(false), co(false);
      }
      Xo(() => {
        window.addEventListener("keydown", q), window.addEventListener("pagehide", Z), De().catch((g) => {
          const p = g instanceof Error ? g.message : String(g);
          m.value = `\u8FDE\u63A5\u5931\u8D25\uFF1A${p}`, Ge(s, `\u8FDE\u63A5\u5931\u8D25\uFF1A${p}`), s.connected = false;
        });
      }), Fr(() => {
        window.removeEventListener("keydown", q), window.removeEventListener("pagehide", Z), Z();
      }), Bt(() => s.speed, () => {
      });
      const d = {
        Prepare: "\u51C6\u5907",
        Draw: "\u6478\u724C",
        Play: "\u51FA\u724C",
        Discard: "\u5F03\u724C",
        Judge: "\u5224\u5B9A",
        End: "\u7ED3\u675F"
      }, h = Q(() => Kd(s.view, s.lastEvents ?? [], o.value));
      function _() {
        n.push("/");
      }
      const b = Q(() => {
        var _a3;
        if (s.view && s.view.phase !== "Idle" && (s.view.current_player !== 0 || s.view.seats.some((p) => p.player === 0))) return {
          currentPlayer: s.view.current_player,
          phase: s.view.phase,
          phaseLabel: d[s.view.phase] ?? s.view.phase,
          round: s.view.round
        };
        const g = s.lastEvents ?? [];
        for (const p of g) {
          const w = p;
          if ((w == null ? void 0 : w.type) === "TurnStateChanged") return {
            currentPlayer: w.current ?? 0,
            phase: w.phase ?? "?",
            phaseLabel: d[w.phase ?? ""] ?? w.phase ?? "?",
            round: ((_a3 = s.view) == null ? void 0 : _a3.round) ?? 0
          };
        }
        return null;
      });
      return (g, p) => {
        const w = lc("auto-scroll");
        return x(), C("div", jp, [
          N.value ? (x(), C("section", Dp, [
            ...p[4] || (p[4] = [
              Uc('<h2 data-v-6ac1c19d>\u2328 \u5FEB\u6377\u952E</h2><dl class="hotkey-list" data-v-6ac1c19d><dt data-v-6ac1c19d><kbd data-v-6ac1c19d>Space</kbd></dt><dd data-v-6ac1c19d>\u81EA\u52A8\u4E00\u6B65</dd><dt data-v-6ac1c19d><kbd data-v-6ac1c19d>E</kbd></dt><dd data-v-6ac1c19d>\u7ED3\u675F\u9636\u6BB5</dd><dt data-v-6ac1c19d><kbd data-v-6ac1c19d>S</kbd></dt><dd data-v-6ac1c19d>\u5207\u6362\u8DF3\u8FC7\u52A8\u753B</dd><dt data-v-6ac1c19d><kbd data-v-6ac1c19d>?</kbd></dt><dd data-v-6ac1c19d>\u663E\u793A/\u9690\u85CF\u6B64\u9762\u677F</dd><dt data-v-6ac1c19d><kbd data-v-6ac1c19d>Esc</kbd></dt><dd data-v-6ac1c19d>\u5173\u95ED\u6B64\u9762\u677F</dd></dl>', 2)
            ])
          ])) : z("", true),
          W(s).finished && h.value ? (x(), C("section", {
            key: 1,
            class: Fe([
              "panel outcome-banner",
              {
                win: h.value.isSelfWin === true,
                lose: h.value.isSelfWin === false
              }
            ]),
            role: "alert",
            "aria-live": "assertive"
          }, [
            k("div", Lp, [
              k("span", Up, P(h.value.isSelfWin === null ? "\u2691" : h.value.isSelfWin ? "\u{1F389}" : "\u{1F480}"), 1),
              k("div", null, [
                k("h2", Hp, P(h.value.label), 1),
                k("p", Fp, "rev " + P(W(s).revision), 1)
              ])
            ]),
            k("button", {
              class: "primary",
              type: "button",
              onClick: p[0] || (p[0] = (v) => D.value ? _() : De())
            }, P(D.value ? "\u8FD4\u56DE\u5927\u5385" : "\u518D\u6765\u4E00\u5C40"), 1)
          ], 2)) : z("", true),
          b.value ? (x(), C("section", qp, [
            k("span", Wp, "\u25BA \u7B2C " + P(b.value.round) + " \u8F6E \xB7 P" + P(b.value.currentPlayer), 1),
            k("span", {
              class: "phase-badge",
              "data-phase": b.value.phase
            }, P(b.value.phaseLabel), 9, Vp)
          ])) : z("", true),
          k("section", Bp, [
            k("div", Kp, [
              D.value ? (x(), C("span", Jp, "\u8054\u673A\u623F\u95F4 #" + P(O.value || "\u2026"), 1)) : (x(), C("button", {
                key: 0,
                class: "primary",
                type: "button",
                disabled: W(s).busy,
                onClick: De
              }, " \u91CD\u65B0\u5F00\u5C40 ", 8, Gp)),
              k("button", {
                type: "button",
                disabled: W(s).busy || W(s).finished || L.value || D.value && !T.value || !rt.value,
                onClick: Pe
              }, " \u7ED3\u675F\u9636\u6BB5 ", 8, zp),
              D.value ? z("", true) : (x(), C("button", {
                key: 2,
                type: "button",
                disabled: W(s).busy || W(s).finished,
                onClick: I
              }, " \u81EA\u52A8\u4E00\u6B65 ", 8, Yp)),
              D.value ? z("", true) : (x(), C("button", {
                key: 3,
                type: "button",
                disabled: W(s).busy,
                onClick: Ue
              }, P(f.value ? "\u505C\u6B62\u81EA\u52A8" : "\u8FDE\u7EED\u81EA\u52A8"), 9, Qp)),
              D.value ? (x(), C("button", {
                key: 4,
                type: "button",
                disabled: !W(s).busy,
                onClick: st
              }, " \u53D6\u6D88\u5F85\u786E\u8BA4 ", 8, Xp)) : z("", true),
              D.value ? z("", true) : (x(), C("button", {
                key: 5,
                type: "button",
                disabled: W(s).busy || !W(s).connected,
                onClick: Ye
              }, " \u5B58\u6863 ", 8, Zp)),
              D.value ? z("", true) : (x(), C("button", {
                key: 6,
                type: "button",
                disabled: W(s).busy || !W(s).connected,
                onClick: Le
              }, " \u8BFB\u6863 ", 8, eg))
            ]),
            k("div", tg, [
              k("label", null, [
                p[6] || (p[6] = xt(" \u500D\u901F ", -1)),
                qs(k("select", {
                  "onUpdate:modelValue": p[1] || (p[1] = (v) => W(s).speed = v),
                  "aria-label": "\u64AD\u653E\u500D\u901F"
                }, [
                  ...p[5] || (p[5] = [
                    k("option", {
                      value: 0.5
                    }, "0.5\xD7", -1),
                    k("option", {
                      value: 1
                    }, "1\xD7", -1),
                    k("option", {
                      value: 2
                    }, "2\xD7", -1),
                    k("option", {
                      value: 4
                    }, "4\xD7", -1)
                  ])
                ], 512), [
                  [
                    bu,
                    W(s).speed,
                    void 0,
                    {
                      number: true
                    }
                  ]
                ])
              ]),
              k("label", ng, [
                qs(k("input", {
                  "onUpdate:modelValue": p[2] || (p[2] = (v) => W(s).skipAnims = v),
                  type: "checkbox"
                }, null, 512), [
                  [
                    _u,
                    W(s).skipAnims
                  ]
                ]),
                p[7] || (p[7] = xt(" \u8DF3\u8FC7\u52A8\u753B ", -1))
              ]),
              k("span", null, P(L.value ? "\u516C\u5171\u65C1\u89C2\u6295\u5F71" : `\u89C2\u5BDF\u8005 P${o.value}`), 1)
            ]),
            k("p", sg, P(m.value), 1),
            W(s).error ? (x(), C("p", rg, P(W(s).error), 1)) : z("", true)
          ]),
          H.value ? (x(), Vr(Mp, {
            key: 3,
            state: E.value,
            busy: W(s).busy,
            onChoose: je,
            onReload: Me
          }, null, 8, [
            "state",
            "busy"
          ])) : z("", true),
          D.value && y.value ? (x(), C("section", ig, [
            k("div", og, [
              k("h2", null, "\u623F\u95F4 #" + P(y.value.room_id), 1),
              k("span", {
                class: Fe(y.value.started ? "started" : "waiting")
              }, P(y.value.started ? "\u5BF9\u5C40\u8FDB\u884C\u4E2D" : "\u7B49\u5F85\u73A9\u5BB6"), 3)
            ]),
            k("div", ag, [
              (x(true), C(de, null, xe(y.value.seats, (v) => (x(), C("span", {
                key: v.seat,
                class: "online-seat"
              }, [
                xt(P(v.seat + 1) + " \xB7 " + P(v.display_name) + " ", 1),
                k("i", {
                  class: Fe(v.connected ? "connected" : "offline")
                }, P(v.connected ? "\u5728\u7EBF" : "\u79BB\u7EBF"), 3)
              ]))), 128))
            ])
          ])) : z("", true),
          W(s).choice ? (x(), C("section", lg, [
            k("h2", null, P(W(vr)(W(s).choice)), 1),
            k("p", cg, " choice #" + P(W(s).choice.choice_id) + " \xB7 \u73A9\u5BB6 " + P(W(s).choice.observer) + " \xB7 " + P(W(Jr)(W(s).choice)), 1),
            Te.value.length ? (x(), C("div", ug, [
              (x(true), C(de, null, xe(Te.value, (v, R) => (x(), C("button", {
                key: R,
                type: "button",
                disabled: W(s).busy || !W(s).connected || L.value,
                onClick: (A) => ie(v)
              }, P(v.ResolveChoice.cancelled ? "\u53D6\u6D88" : `\u9009\u62E9 ${v.ResolveChoice.selections.join(", ") || "\u7A7A"}`), 9, fg))), 128))
            ])) : (x(), C("p", dg, "\u5F53\u524D\u89C2\u5BDF\u8005\u65E0\u6743\u5904\u7406\u6B64\u9009\u62E9\u3002"))
          ])) : z("", true),
          W(s).view ? (x(), C("section", hg, [
            p[8] || (p[8] = k("h2", null, "\u73A9\u5BB6\u6982\u51B5", -1)),
            k("div", pg, [
              (x(true), C(de, null, xe(W(s).view.seats, (v) => (x(), C("div", {
                key: v.player,
                class: Fe([
                  "roster-entry",
                  {
                    dead: !v.alive,
                    self: v.player === o.value
                  }
                ])
              }, [
                k("span", gg, "P" + P(v.player) + P(v.turned_over ? " \xB7 \u7FFB\u9762" : ""), 1),
                k("span", {
                  class: "roster-hp",
                  style: Kt({
                    color: v.alive ? v.hp <= 0 ? "var(--color-warning)" : "inherit" : "var(--color-negative)"
                  })
                }, P(v.alive ? v.hp <= 0 ? "\u26A0" : "" : "\u2717") + " " + P(v.hp) + "/" + P(v.max_hp), 5),
                k("span", mg, P(v.hand_count) + "\u{1F0A0}", 1),
                v.identity ? (x(), C("span", {
                  key: 0,
                  class: "roster-id",
                  "data-id": v.identity
                }, P(v.identity === "Lord" ? "\u4E3B" : v.identity === "Loyalist" ? "\u5FE0" : v.identity === "Rebel" ? "\u53CD" : "\u5978"), 9, vg)) : (x(), C("span", yg, "?"))
              ], 2))), 128))
            ])
          ])) : z("", true),
          k("section", _g, [
            ke(yp, {
              view: W(s).view,
              "self-id": o.value,
              "legal-actions": W(s).connected && !W(s).busy ? W(s).legalActions : [],
              onPlayCard: M,
              onActivateSkill: F
            }, null, 8, [
              "view",
              "self-id",
              "legal-actions"
            ]),
            W(s).effects.length ? (x(), C("div", bg, [
              (x(true), C(de, null, xe(W(s).effects, (v) => (x(), C("div", {
                key: v.id,
                class: Fe([
                  "effect-popup",
                  v.type
                ])
              }, [
                k("span", wg, P(v.type === "damage" ? "\u2694" : v.type === "heal" ? "\u271A" : v.type === "draw" ? "\u{1F0A0}" : v.type === "play" ? "\u25B8" : v.type === "dying" ? "\u26A0" : "\u2717"), 1),
                k("span", Sg, P(v.text), 1),
                v.player != null ? (x(), C("span", kg, "P" + P(v.player), 1)) : z("", true)
              ], 2))), 128))
            ])) : z("", true)
          ]),
          W(s).view ? (x(), C("section", Eg, [
            k("div", Ag, [
              k("div", xg, [
                p[9] || (p[9] = k("span", {
                  class: "stat-label muted"
                }, "Revision", -1)),
                k("span", Cg, P(W(s).revision), 1)
              ]),
              k("div", Rg, [
                p[10] || (p[10] = k("span", {
                  class: "stat-label muted"
                }, "State Hash", -1)),
                k("span", Pg, P(W(s).hash.slice(0, 16)) + "\u2026", 1)
              ]),
              k("div", Tg, [
                p[11] || (p[11] = k("span", {
                  class: "stat-label muted"
                }, "\u5DF2\u77E5\u5361\u724C", -1)),
                k("span", Ig, P(W(s).view.known_cards.length), 1)
              ]),
              k("div", Og, [
                p[12] || (p[12] = k("span", {
                  class: "stat-label muted"
                }, "\u5B58\u6D3B\u73A9\u5BB6", -1)),
                k("span", Ng, P(W(s).view.seats.filter((v) => v.alive).length) + " / " + P(W(s).view.seats.length), 1)
              ])
            ])
          ])) : z("", true),
          W(s).view ? (x(), C("section", $g, [
            p[13] || (p[13] = k("h2", null, "\u5EA7\u4F4D\u72B6\u6001", -1)),
            k("div", Mg, [
              (x(true), C(de, null, xe(W(s).view.seats, (v) => (x(), C("div", {
                key: "ss-" + v.player,
                class: Fe([
                  "ss-entry",
                  {
                    "ss-dead": !v.alive,
                    "ss-self": v.player === o.value
                  }
                ])
              }, [
                k("div", jg, [
                  k("span", Dg, "P" + P(v.player), 1),
                  v.player === o.value ? (x(), C("span", Lg, "\u4F60")) : z("", true)
                ]),
                k("div", Ug, [
                  k("div", {
                    class: "ss-hp-fill",
                    style: Kt({
                      width: Math.max(Math.min(v.hp / v.max_hp * 100, 100), 0) + "%",
                      background: v.alive ? v.hp <= 0 ? "var(--color-warning)" : "var(--color-positive)" : "var(--color-negative)"
                    })
                  }, null, 4)
                ]),
                k("div", Hg, [
                  k("span", Fg, P(v.alive ? v.hp <= 0 ? "\u26A0" : "" : "\u2717"), 1),
                  xt(" " + P(Math.max(v.hp, 0)) + "/" + P(v.max_hp), 1)
                ]),
                k("div", qg, [
                  k("span", null, P(v.hand_count) + "\u{1F0A0}", 1),
                  v.identity ? (x(), C("span", {
                    key: 0,
                    class: "ss-id",
                    "data-id": v.identity
                  }, P(v.identity === "Lord" ? "\u4E3B" : v.identity === "Loyalist" ? "\u5FE0" : v.identity === "Rebel" ? "\u53CD" : "\u5978"), 9, Wg)) : (x(), C("span", Vg, "?"))
                ])
              ], 2))), 128))
            ])
          ])) : z("", true),
          k("section", Bg, [
            k("div", Kg, [
              p[14] || (p[14] = k("h2", null, "\u6218\u6597\u65E5\u5FD7", -1)),
              W(s).log.length > 0 ? (x(), C("button", {
                key: 0,
                type: "button",
                class: "clear-btn",
                onClick: p[3] || (p[3] = (v) => W(s).log = []),
                "aria-label": "\u6E05\u7A7A\u65E5\u5FD7"
              }, "\u6E05\u7A7A")) : z("", true)
            ]),
            p[15] || (p[15] = k("p", {
              class: "muted tiny"
            }, " \u65E5\u5FD7\u6765\u81EA DomainEvent \u6458\u8981\uFF0C\u4E0D\u662F\u72B6\u6001\u771F\u76F8\u6E90\u3002\u771F\u76F8\u6E90\u4E3A Command Log + \u6295\u5F71\uFF08\xA722\uFF09\u3002 ", -1)),
            qs((x(), C("ol", Gg, [
              (x(true), C(de, null, xe(W(s).log.slice().reverse(), (v) => (x(), C("li", {
                key: v.id,
                class: Fe([
                  "log-entry",
                  U(v.text)
                ])
              }, [
                k("span", Jg, "rev" + P(v.at), 1),
                k("span", zg, P(v.text), 1)
              ], 2))), 128))
            ])), [
              [
                w
              ]
            ]),
            W(s).log.length === 0 ? (x(), C("p", Yg, "\u6682\u65E0\u4E8B\u4EF6")) : z("", true)
          ])
        ]);
      };
    }
  }), go = gn(Qg, [
    [
      "__scopeId",
      "data-v-6ac1c19d"
    ]
  ]);
  function Ke(e, t) {
    if (!e) throw new Error(t);
  }
  const el = Ms.prototype.start;
  let Pn, tl, Ft, Tn = 1;
  const Qr = [];
  Ms.prototype.start = async function() {
    return Qr.push(this), await el.call(this, Pn.seats.length, 0, Tn, Pn.packsJson), Ft = await this.restoreHex(Pn.archiveHex, Pn.packsJson), tl(), Ft;
  };
  const tr = async (e) => {
    for (let t = 0; t < 300; t++) {
      if (e()) return;
      await new Promise((n) => setTimeout(n, 20));
    }
    throw new Error("UI did not render expected state");
  };
  async function Xg() {
    var _a2, _b, _c2, _d2, _e, _f2, _g2;
    history.replaceState(null, "", Mt() ? "/?engineTransport=native" : "/?engineTransport=wasm"), localStorage.setItem("noname-next-skip-anims", "1");
    const e = await fetch("/metadata-fixtures.json").then((s) => s.json()), t = /* @__PURE__ */ new Set(), n = [];
    for (const s of e) {
      Pn = s, Tn = s.seats.findIndex((l) => l.choice !== null), Tn < 0 && (Tn = 1);
      const r = new Promise((l) => {
        tl = l;
      }), i = document.createElement("div");
      document.body.append(i);
      const a = Nf({
        history: ff(),
        routes: [
          {
            path: "/",
            component: go
          }
        ]
      });
      await a.push("/"), await a.isReady();
      const o = Ni(go);
      o.use(a), o.mount(i);
      try {
        await r;
        const l = Ft.view.seats.flatMap((y) => y.generals ?? [
          null,
          null
        ]);
        await tr(() => i.querySelectorAll(".general-slot").length === l.length);
        const c = [
          ...i.querySelectorAll(".general-slot")
        ];
        for (const [y, O] of l.entries()) {
          const T = c[y].textContent ?? "";
          O ? (Ke(T.includes(O.name), `${s.name}: visible general missing`), Ke(T.includes(O.revealed ? "\u660E\u7F6E" : "\u6697\u7F6E"), `${s.name}: reveal flag missing`)) : Ke(T.includes("\u6697\u5C06\uFF0F\u672A\u914D\u7F6E"), `${s.name}: hidden slot was disclosed`);
        }
        if (Ft.choice) {
          const y = Ft.choice;
          if (t.add(y.native_kind), await tr(() => {
            var _a3, _b2;
            return !!((_b2 = (_a3 = i.querySelector(".choice h2")) == null ? void 0 : _a3.textContent) == null ? void 0 : _b2.includes(vr(y)));
          }), Ke((_b = (_a2 = i.querySelector(".choice")) == null ? void 0 : _a2.textContent) == null ? void 0 : _b.includes(Jr(y)), `${s.name}: readable kind missing`), Ke(!((_d2 = (_c2 = i.querySelector(".choice")) == null ? void 0 : _c2.textContent) == null ? void 0 : _d2.includes(y.prompt)), `${s.name}: internal prompt displayed`), Tn === 1 && [
            "confirm",
            "select_players",
            "select_cards",
            "respond_card",
            "pick_tag-fanjian"
          ].includes(s.name)) {
            const O = i.querySelector(".choice button");
            Ke(O && !O.disabled, `${s.name}: no real answer control`), O.click(), await tr(() => {
              var _a3, _b2, _c3;
              return !((_b2 = (_a3 = i.querySelector(".choice h2")) == null ? void 0 : _a3.textContent) == null ? void 0 : _b2.includes(vr(y))) || ((_c3 = i.textContent) == null ? void 0 : _c3.includes("rev " + (Ft.revision + 1))) === true;
            }), Ke(!((_e = i.textContent) == null ? void 0 : _e.includes("WrongPlayer")), `${s.name}: UI answered as wrong seat`);
          }
        }
        const u = Qr.at(-1), f = await u.saveHex(), m = await u.restoreHex(f, s.packsJson);
        Ke(JSON.stringify(m.view.seats.map((y) => y.generals)) === JSON.stringify((await u.observer()).seats.map((y) => y.generals)), `${s.name}: restored metadata differs`), n.push({
          name: s.name,
          slots: c.length,
          prompt: ((_f2 = Ft.choice) == null ? void 0 : _f2.prompt) ?? null,
          transport: u.getTransportKind(),
          restored: true
        });
      } finally {
        o.unmount(), i.remove();
      }
    }
    Ke(t.size === 5, "not all five real choice kinds reached BattleView");
    for (const [s, r] of [
      [
        "old-payload",
        {
          character_def: 17
        }
      ],
      [
        "unknown-extension",
        {
          character_def: 0,
          generals: [
            {
              def: 0,
              name: "<img src=x onerror=alert(1)>",
              faction: "unknown",
              revealed: false
            },
            null
          ]
        }
      ]
    ]) {
      const i = document.createElement("div");
      document.body.append(i);
      const a = {
        seat: {
          player: 0,
          seat: 0,
          hp: 4,
          max_hp: 4,
          alive: true,
          turned_over: false,
          hand_count: 0,
          identity: null,
          skills: [],
          ...r
        },
        self: true,
        identityLabel: "\u672A\u77E5",
        isTargetMode: false,
        isSelectableTarget: false,
        isDropTarget: false,
        equipCards: []
      }, o = Ni(Za, a);
      o.mount(i), await pn(), Ke((_g2 = i.textContent) == null ? void 0 : _g2.includes(s === "old-payload" ? "#17" : "<img src=x onerror=alert(1)>"), `${s}: fallback missing`), Ke(i.querySelector("img[onerror]") === null, `${s}: untrusted markup`), Ke([
        ...i.querySelectorAll("img")
      ].every((l) => !l.src.includes("onerror")), `${s}: untrusted path`), o.unmount(), i.remove();
    }
    return {
      passed: true,
      cases: n,
      kinds: [
        ...t
      ].sort(),
      oldPayloadFallback: true,
      extensionEscaped: true
    };
  }
  try {
    const e = await Xg();
    await fetch("/report", {
      method: "POST",
      body: JSON.stringify(e)
    }), document.body.textContent = JSON.stringify(e);
  } catch (e) {
    const t = {
      passed: false,
      error: String(e),
      stack: e instanceof Error ? e.stack : null
    };
    await fetch("/report", {
      method: "POST",
      body: JSON.stringify(t)
    }), document.body.textContent = JSON.stringify(t);
  } finally {
    Ms.prototype.start = el;
    for (const e of Qr) e.dispose();
  }
})();
