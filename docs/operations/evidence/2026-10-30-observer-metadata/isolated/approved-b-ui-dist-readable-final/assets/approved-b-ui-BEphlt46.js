var __defProp = Object.defineProperty;
var __defNormalProp = (obj, key, value) => key in obj ? __defProp(obj, key, { enumerable: true, configurable: true, writable: true, value }) : obj[key] = value;
var __publicField = (obj, key, value) => __defNormalProp(obj, typeof key !== "symbol" ? key + "" : key, value);
(async () => {
  function Rr(e) {
    const t = /* @__PURE__ */ Object.create(null);
    for (const n of e.split(",")) t[n] = 1;
    return (n) => n in t;
  }
  const fe = {}, Zt = [], pt = () => {
  }, mo = () => false, ks = (e) => e.charCodeAt(0) === 111 && e.charCodeAt(1) === 110 && (e.charCodeAt(2) > 122 || e.charCodeAt(2) < 97), Es = (e) => e.startsWith("onUpdate:"), Ae = Object.assign, Pr = (e, t) => {
    const n = e.indexOf(t);
    n > -1 && e.splice(n, 1);
  }, nl = Object.prototype.hasOwnProperty, ae = (e, t) => nl.call(e, t), G = Array.isArray, en = (e) => Kn(e) === "[object Map]", dn = (e) => Kn(e) === "[object Set]", ti = (e) => Kn(e) === "[object Date]", Y = (e) => typeof e == "function", ve = (e) => typeof e == "string", mt = (e) => typeof e == "symbol", le = (e) => e !== null && typeof e == "object", vo = (e) => (le(e) || Y(e)) && Y(e.then) && Y(e.catch), yo = Object.prototype.toString, Kn = (e) => yo.call(e), sl = (e) => Kn(e).slice(8, -1), _o = (e) => Kn(e) === "[object Object]", Tr = (e) => ve(e) && e !== "NaN" && e[0] !== "-" && "" + parseInt(e, 10) === e, In = Rr(",key,ref,ref_for,ref_key,onVnodeBeforeMount,onVnodeMounted,onVnodeBeforeUpdate,onVnodeUpdated,onVnodeBeforeUnmount,onVnodeUnmounted"), As = (e) => {
    const t = /* @__PURE__ */ Object.create(null);
    return ((n) => t[n] || (t[n] = e(n)));
  }, rl = /-\w/g, Ge = As((e) => e.replace(rl, (t) => t.slice(1).toUpperCase())), il = /\B([A-Z])/g, Lt = As((e) => e.replace(il, "-$1").toLowerCase()), Ir = As((e) => e.charAt(0).toUpperCase() + e.slice(1)), Ls = As((e) => e ? `on${Ir(e)}` : ""), ht = (e, t) => !Object.is(e, t), ss = (e, ...t) => {
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
  const xs = () => ni || (ni = typeof globalThis < "u" ? globalThis : typeof self < "u" ? self : typeof window < "u" ? window : typeof global < "u" ? global : {});
  function Gt(e) {
    if (G(e)) {
      const t = {};
      for (let n = 0; n < e.length; n++) {
        const s = e[n], r = ve(s) ? cl(s) : Gt(s);
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
  function We(e) {
    let t = "";
    if (ve(e)) t = e;
    else if (G(e)) for (let n = 0; n < e.length; n++) {
      const s = We(e[n]);
      s && (t += s + " ");
    }
    else if (le(e)) for (const n in e) e[n] && (t += n + " ");
    return t.trim();
  }
  const ul = "itemscope,allowfullscreen,formnovalidate,ismap,nomodule,novalidate,readonly", fl = Rr(ul);
  function So(e) {
    return !!e || e === "";
  }
  function dl(e, t) {
    if (e.length !== t.length) return false;
    let n = true;
    for (let s = 0; n && s < e.length; s++) n = hn(e[s], t[s]);
    return n;
  }
  function hn(e, t) {
    if (e === t) return true;
    let n = ti(e), s = ti(t);
    if (n || s) return n && s ? e.getTime() === t.getTime() : false;
    if (n = mt(e), s = mt(t), n || s) return e === t;
    if (n = G(e), s = G(t), n || s) return n && s ? dl(e, t) : false;
    if (n = le(e), s = le(t), n || s) {
      if (!n || !s) return false;
      const r = Object.keys(e).length, i = Object.keys(t).length;
      if (r !== i) return false;
      for (const o in e) {
        const a = e.hasOwnProperty(o), l = t.hasOwnProperty(o);
        if (a && !l || !a && l || !hn(e[o], t[o])) return false;
      }
    }
    return String(e) === String(t);
  }
  function Or(e, t) {
    return e.findIndex((n) => hn(n, t));
  }
  const ko = (e) => !!(e && e.__v_isRef === true), I = (e) => ve(e) ? e : e == null ? "" : G(e) || le(e) && (e.toString === yo || !Y(e.toString)) ? ko(e) ? I(e.value) : JSON.stringify(e, Eo, 2) : String(e), Eo = (e, t) => ko(t) ? Eo(e, t.value) : en(t) ? {
    [`Map(${t.size})`]: [
      ...t.entries()
    ].reduce((n, [s, r], i) => (n[Us(s, i) + " =>"] = r, n), {})
  } : dn(t) ? {
    [`Set(${t.size})`]: [
      ...t.values()
    ].map((n) => Us(n))
  } : mt(t) ? Us(t) : le(t) && !G(t) && !_o(t) ? String(t) : t, Us = (e, t = "") => {
    var n;
    return mt(e) ? `Symbol(${(n = e.description) != null ? n : t})` : e;
  };
  let ke;
  class hl {
    constructor(t = false) {
      this.detached = t, this._active = true, this._on = 0, this.effects = [], this.cleanups = [], this._isPaused = false, this._warnOnRun = true, this.__v_skip = true, !t && ke && (ke.active ? (this.parent = ke, this.index = (ke.scopes || (ke.scopes = [])).push(this) - 1) : (this._active = false, this._warnOnRun = false));
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
        const n = ke;
        try {
          return ke = this, t();
        } finally {
          ke = n;
        }
      }
    }
    on() {
      ++this._on === 1 && (this.prevScope = ke, ke = this);
    }
    off() {
      if (this._on > 0 && --this._on === 0) {
        if (ke === this) ke = this.prevScope;
        else {
          let t = ke;
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
    return ke;
  }
  let he;
  const Hs = /* @__PURE__ */ new WeakSet();
  class Ao {
    constructor(t) {
      this.fn = t, this.deps = void 0, this.depsTail = void 0, this.flags = 5, this.next = void 0, this.cleanup = void 0, this.scheduler = void 0, ke && (ke.active ? ke.effects.push(this) : this.flags &= -2);
    }
    pause() {
      this.flags |= 64;
    }
    resume() {
      this.flags & 64 && (this.flags &= -65, Hs.has(this) && (Hs.delete(this), this.trigger()));
    }
    notify() {
      this.flags & 2 && !(this.flags & 32) || this.flags & 8 || Co(this);
    }
    run() {
      if (!(this.flags & 1)) return this.fn();
      this.flags |= 2, si(this), Ro(this);
      const t = he, n = Ze;
      he = this, Ze = true;
      try {
        return this.fn();
      } finally {
        Po(this), he = t, Ze = n, this.flags &= -3;
      }
    }
    stop() {
      if (this.flags & 1) {
        for (let t = this.deps; t; t = t.nextDep) Mr(t);
        this.deps = this.depsTail = void 0, si(this), this.onStop && this.onStop(), this.flags &= -2;
      }
    }
    trigger() {
      this.flags & 64 ? Hs.add(this) : this.scheduler ? this.scheduler() : this.runIfDirty();
    }
    runIfDirty() {
      sr(this) && this.run();
    }
    get dirty() {
      return sr(this);
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
  function Nr() {
    xo++;
  }
  function $r() {
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
      s.version === -1 ? (s === n && (n = r), Mr(s), gl(s)) : t = s, s.dep.activeLink = s.prevActiveLink, s.prevActiveLink = void 0, s = r;
    }
    e.deps = t, e.depsTail = n;
  }
  function sr(e) {
    for (let t = e.deps; t; t = t.nextDep) if (t.dep.version !== t.version || t.dep.computed && (To(t.dep.computed) || t.dep.version !== t.version)) return true;
    return !!e._dirty;
  }
  function To(e) {
    if (e.flags & 4 && !(e.flags & 16) || (e.flags &= -17, e.globalVersion === Un) || (e.globalVersion = Un, !e.isSSR && e.flags & 128 && (!e.deps && !e._dirty || !sr(e)))) return;
    e.flags |= 2;
    const t = e.dep, n = he, s = Ze;
    he = e, Ze = true;
    try {
      Ro(e);
      const r = e.fn(e._value);
      (t.version === 0 || ht(r, e._value)) && (e.flags |= 128, e._value = r, t.version++);
    } catch (r) {
      throw t.version++, r;
    } finally {
      he = n, Ze = s, Po(e), e.flags &= -3;
    }
  }
  function Mr(e, t = false) {
    const { dep: n, prevSub: s, nextSub: r } = e;
    if (s && (s.nextSub = r, e.prevSub = void 0), r && (r.prevSub = s, e.nextSub = void 0), n.subs === e && (n.subs = s, !s && n.computed)) {
      n.computed.flags &= -5;
      for (let i = n.computed.deps; i; i = i.nextDep) Mr(i, true);
    }
    !t && !--n.sc && n.map && n.map.delete(n.key);
  }
  function gl(e) {
    const { prevDep: t, nextDep: n } = e;
    t && (t.nextDep = n, e.prevDep = void 0), n && (n.prevDep = t, e.nextDep = void 0);
  }
  let Ze = true;
  const Io = [];
  function vt() {
    Io.push(Ze), Ze = false;
  }
  function yt() {
    const e = Io.pop();
    Ze = e === void 0 ? true : e;
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
  class Dr {
    constructor(t) {
      this.computed = t, this.version = 0, this.activeLink = void 0, this.subs = void 0, this.map = void 0, this.key = void 0, this.sc = 0, this.__v_skip = true;
    }
    track(t) {
      if (!he || !Ze || he === this.computed) return;
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
      Nr();
      try {
        for (let n = this.subs; n; n = n.prevSub) n.sub.notify() && n.sub.dep.notify();
      } finally {
        $r();
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
  const rr = /* @__PURE__ */ new WeakMap(), Vt = Symbol(""), ir = Symbol(""), Hn = Symbol("");
  function xe(e, t, n) {
    if (Ze && he) {
      let s = rr.get(e);
      s || rr.set(e, s = /* @__PURE__ */ new Map());
      let r = s.get(n);
      r || (s.set(n, r = new Dr()), r.map = s, r.key = n), r.track();
    }
  }
  function At(e, t, n, s, r, i) {
    const o = rr.get(e);
    if (!o) {
      Un++;
      return;
    }
    const a = (l) => {
      l && l.trigger();
    };
    if (Nr(), t === "clear") o.forEach(a);
    else {
      const l = G(e), c = l && Tr(n);
      if (l && n === "length") {
        const u = Number(s);
        o.forEach((f, m) => {
          (m === "length" || m === Hn || !mt(m) && m >= u) && a(f);
        });
      } else switch ((n !== void 0 || o.has(void 0)) && a(o.get(n)), c && a(o.get(Hn)), t) {
        case "add":
          l ? c && a(o.get("length")) : (a(o.get(Vt)), en(e) && a(o.get(ir)));
          break;
        case "delete":
          l || (a(o.get(Vt)), en(e) && a(o.get(ir)));
          break;
        case "set":
          en(e) && a(o.get(Vt));
          break;
      }
    }
    $r();
  }
  function Jt(e) {
    const t = oe(e);
    return t === e ? t : (xe(t, "iterate", Hn), Ye(e) ? t : t.map(tt));
  }
  function Cs(e) {
    return xe(e = oe(e), "iterate", Hn), e;
  }
  function ft(e, t) {
    return Rt(e) ? on(Bt(e) ? tt(t) : t) : tt(t);
  }
  const vl = {
    __proto__: null,
    [Symbol.iterator]() {
      return Fs(this, Symbol.iterator, (e) => ft(this, e));
    },
    concat(...e) {
      return Jt(this).concat(...e.map((t) => G(t) ? Jt(t) : t));
    },
    entries() {
      return Fs(this, "entries", (e) => (e[1] = ft(this, e[1]), e));
    },
    every(e, t) {
      return bt(this, "every", e, t, void 0, arguments);
    },
    filter(e, t) {
      return bt(this, "filter", e, t, (n) => n.map((s) => ft(this, s)), arguments);
    },
    find(e, t) {
      return bt(this, "find", e, t, (n) => ft(this, n), arguments);
    },
    findIndex(e, t) {
      return bt(this, "findIndex", e, t, void 0, arguments);
    },
    findLast(e, t) {
      return bt(this, "findLast", e, t, (n) => ft(this, n), arguments);
    },
    findLastIndex(e, t) {
      return bt(this, "findLastIndex", e, t, void 0, arguments);
    },
    forEach(e, t) {
      return bt(this, "forEach", e, t, void 0, arguments);
    },
    includes(...e) {
      return Ws(this, "includes", e);
    },
    indexOf(...e) {
      return Ws(this, "indexOf", e);
    },
    join(e) {
      return Jt(this).join(e);
    },
    lastIndexOf(...e) {
      return Ws(this, "lastIndexOf", e);
    },
    map(e, t) {
      return bt(this, "map", e, t, void 0, arguments);
    },
    pop() {
      return yn(this, "pop");
    },
    push(...e) {
      return yn(this, "push", e);
    },
    reduce(e, ...t) {
      return ri(this, "reduce", e, t);
    },
    reduceRight(e, ...t) {
      return ri(this, "reduceRight", e, t);
    },
    shift() {
      return yn(this, "shift");
    },
    some(e, t) {
      return bt(this, "some", e, t, void 0, arguments);
    },
    splice(...e) {
      return yn(this, "splice", e);
    },
    toReversed() {
      return Jt(this).toReversed();
    },
    toSorted(e) {
      return Jt(this).toSorted(e);
    },
    toSpliced(...e) {
      return Jt(this).toSpliced(...e);
    },
    unshift(...e) {
      return yn(this, "unshift", e);
    },
    values() {
      return Fs(this, "values", (e) => ft(this, e));
    }
  };
  function Fs(e, t, n) {
    const s = Cs(e), r = s[t]();
    return s !== e && !Ye(e) && (r._next = r.next, r.next = () => {
      const i = r._next();
      return i.done || (i.value = n(i.value)), i;
    }), r;
  }
  const yl = Array.prototype;
  function bt(e, t, n, s, r, i) {
    const o = Cs(e), a = o !== e && !Ye(e), l = o[t];
    if (l !== yl[t]) {
      const f = l.apply(e, i);
      return a ? tt(f) : f;
    }
    let c = n;
    o !== e && (a ? c = function(f, m) {
      return n.call(this, ft(e, f), m, e);
    } : n.length > 2 && (c = function(f, m) {
      return n.call(this, f, m, e);
    }));
    const u = l.call(o, c, s);
    return a && r ? r(u) : u;
  }
  function ri(e, t, n, s) {
    const r = Cs(e), i = r !== e && !Ye(e);
    let o = n, a = false;
    r !== e && (i ? (a = s.length === 0, o = function(c, u, f) {
      return a && (a = false, c = ft(e, c)), n.call(this, c, ft(e, u), f, e);
    }) : n.length > 3 && (o = function(c, u, f) {
      return n.call(this, c, u, f, e);
    }));
    const l = r[t](o, ...s);
    return a ? ft(e, l) : l;
  }
  function Ws(e, t, n) {
    const s = oe(e);
    xe(s, "iterate", Hn);
    const r = s[t](...n);
    return (r === -1 || r === false) && Ur(n[0]) ? (n[0] = oe(n[0]), s[t](...n)) : r;
  }
  function yn(e, t, n = []) {
    vt(), Nr();
    const s = oe(e)[t].apply(e, n);
    return $r(), yt(), s;
  }
  const _l = Rr("__proto__,__v_isRef,__isVue"), No = new Set(Object.getOwnPropertyNames(Symbol).filter((e) => e !== "arguments" && e !== "caller").map((e) => Symbol[e]).filter(mt));
  function bl(e) {
    mt(e) || (e = String(e));
    const t = oe(this);
    return xe(t, "has", e), t.hasOwnProperty(e);
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
      if (n === "__v_raw") return s === (r ? i ? Tl : Lo : i ? jo : Do).get(t) || Object.getPrototypeOf(t) === Object.getPrototypeOf(s) ? t : void 0;
      const o = G(t);
      if (!r) {
        let l;
        if (o && (l = vl[n])) return l;
        if (n === "hasOwnProperty") return bl;
      }
      const a = Reflect.get(t, n, Pe(t) ? t : s);
      if ((mt(n) ? No.has(n) : _l(n)) || (r || xe(t, "get", n), i)) return a;
      if (Pe(a)) {
        const l = o && Tr(n) ? a : a.value;
        return r && le(l) ? ar(l) : l;
      }
      return le(a) ? r ? ar(a) : pn(a) : a;
    }
  }
  class Mo extends $o {
    constructor(t = false) {
      super(false, t);
    }
    set(t, n, s, r) {
      let i = t[n];
      const o = G(t) && Tr(n);
      if (!this._isShallow) {
        const c = Rt(i);
        if (!Ye(s) && !Rt(s) && (i = oe(i), s = oe(s)), !o && Pe(i) && !Pe(s)) return c || (i.value = s), true;
      }
      const a = o ? Number(n) < t.length : ae(t, n), l = Reflect.set(t, n, s, Pe(t) ? t : r);
      return t === oe(r) && l && (a ? ht(s, i) && At(t, "set", n, s) : At(t, "add", n, s)), l;
    }
    deleteProperty(t, n) {
      const s = ae(t, n);
      t[n];
      const r = Reflect.deleteProperty(t, n);
      return r && s && At(t, "delete", n, void 0), r;
    }
    has(t, n) {
      const s = Reflect.has(t, n);
      return (!mt(n) || !No.has(n)) && xe(t, "has", n), s;
    }
    ownKeys(t) {
      return xe(t, "iterate", G(t) ? "length" : Vt), Reflect.ownKeys(t);
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
  const or = (e) => e, Qn = (e) => Reflect.getPrototypeOf(e);
  function Al(e, t, n) {
    return function(...s) {
      const r = this.__v_raw, i = oe(r), o = en(i), a = e === "entries" || e === Symbol.iterator && o, l = e === "keys" && o, c = r[e](...s), u = n ? or : t ? on : tt;
      return !t && xe(i, "iterate", l ? ir : Vt), Ae(Object.create(c), {
        next() {
          const { value: f, done: m } = c.next();
          return m ? {
            value: f,
            done: m
          } : {
            value: a ? [
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
        const i = this.__v_raw, o = oe(i), a = oe(r);
        e || (ht(r, a) && xe(o, "get", r), xe(o, "get", a));
        const { has: l } = Qn(o), c = t ? or : e ? on : tt;
        if (l.call(o, r)) return c(i.get(r));
        if (l.call(o, a)) return c(i.get(a));
        i !== o && i.get(r);
      },
      get size() {
        const r = this.__v_raw;
        return !e && xe(oe(r), "iterate", Vt), r.size;
      },
      has(r) {
        const i = this.__v_raw, o = oe(i), a = oe(r);
        return e || (ht(r, a) && xe(o, "has", r), xe(o, "has", a)), r === a ? i.has(r) : i.has(r) || i.has(a);
      },
      forEach(r, i) {
        const o = this, a = o.__v_raw, l = oe(a), c = t ? or : e ? on : tt;
        return !e && xe(l, "iterate", Vt), a.forEach((u, f) => r.call(i, c(u), c(f), o));
      }
    };
    return Ae(n, e ? {
      add: Xn("add"),
      set: Xn("set"),
      delete: Xn("delete"),
      clear: Xn("clear")
    } : {
      add(r) {
        const i = oe(this), o = Qn(i), a = oe(r), l = !t && !Ye(r) && !Rt(r) ? a : r;
        return o.has.call(i, l) || ht(r, l) && o.has.call(i, r) || ht(a, l) && o.has.call(i, a) || (i.add(l), At(i, "add", l, l)), this;
      },
      set(r, i) {
        !t && !Ye(i) && !Rt(i) && (i = oe(i));
        const o = oe(this), { has: a, get: l } = Qn(o);
        let c = a.call(o, r);
        c || (r = oe(r), c = a.call(o, r));
        const u = l.call(o, r);
        return o.set(r, i), c ? ht(i, u) && At(o, "set", r, i) : At(o, "add", r, i), this;
      },
      delete(r) {
        const i = oe(this), { has: o, get: a } = Qn(i);
        let l = o.call(i, r);
        l || (r = oe(r), l = o.call(i, r)), a && a.call(i, r);
        const c = i.delete(r);
        return l && At(i, "delete", r, void 0), c;
      },
      clear() {
        const r = oe(this), i = r.size !== 0, o = r.clear();
        return i && At(r, "clear", void 0, void 0), o;
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
  const Do = /* @__PURE__ */ new WeakMap(), jo = /* @__PURE__ */ new WeakMap(), Lo = /* @__PURE__ */ new WeakMap(), Tl = /* @__PURE__ */ new WeakMap();
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
  function pn(e) {
    return Rt(e) ? e : Lr(e, false, Sl, Cl, Do);
  }
  function Uo(e) {
    return Lr(e, false, El, Rl, jo);
  }
  function ar(e) {
    return Lr(e, true, kl, Pl, Lo);
  }
  function Lr(e, t, n, s, r) {
    if (!le(e) || e.__v_raw && !(t && e.__v_isReactive) || e.__v_skip || !Object.isExtensible(e)) return e;
    const i = r.get(e);
    if (i) return i;
    const o = Il(sl(e));
    if (o === 0) return e;
    const a = new Proxy(e, o === 2 ? s : n);
    return r.set(e, a), a;
  }
  function Bt(e) {
    return Rt(e) ? Bt(e.__v_raw) : !!(e && e.__v_isReactive);
  }
  function Rt(e) {
    return !!(e && e.__v_isReadonly);
  }
  function Ye(e) {
    return !!(e && e.__v_isShallow);
  }
  function Ur(e) {
    return e ? !!e.__v_raw : false;
  }
  function oe(e) {
    const t = e && e.__v_raw;
    return t ? oe(t) : e;
  }
  function Ol(e) {
    return !ae(e, "__v_skip") && Object.isExtensible(e) && bo(e, "__v_skip", true), e;
  }
  const tt = (e) => le(e) ? pn(e) : e, on = (e) => le(e) ? ar(e) : e;
  function Pe(e) {
    return e ? e.__v_isRef === true : false;
  }
  function ye(e) {
    return Ho(e, false);
  }
  function Nl(e) {
    return Ho(e, true);
  }
  function Ho(e, t) {
    return Pe(e) ? e : new $l(e, t);
  }
  class $l {
    constructor(t, n) {
      this.dep = new Dr(), this.__v_isRef = true, this.__v_isShallow = false, this._rawValue = n ? t : oe(t), this._value = n ? t : tt(t), this.__v_isShallow = n;
    }
    get value() {
      return this.dep.track(), this._value;
    }
    set value(t) {
      const n = this._rawValue, s = this.__v_isShallow || Ye(t) || Rt(t);
      t = s ? t : oe(t), ht(t, n) && (this._rawValue = t, this._value = s ? t : tt(t), this.dep.trigger());
    }
  }
  function q(e) {
    return Pe(e) ? e.value : e;
  }
  const Ml = {
    get: (e, t, n) => t === "__v_raw" ? e : q(Reflect.get(e, t, n)),
    set: (e, t, n, s) => {
      const r = e[t];
      return Pe(r) && !Pe(n) ? (r.value = n, true) : Reflect.set(e, t, n, s);
    }
  };
  function Fo(e) {
    return Bt(e) ? e : new Proxy(e, Ml);
  }
  class Dl {
    constructor(t, n, s) {
      this.fn = t, this.setter = n, this._value = void 0, this.dep = new Dr(this), this.__v_isRef = true, this.deps = void 0, this.depsTail = void 0, this.flags = 16, this.globalVersion = Un - 1, this.next = void 0, this.effect = this, this.__v_isReadonly = !n, this.isSSR = s;
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
  function jl(e, t, n = false) {
    let s, r;
    return Y(e) ? s = e : (s = e.get, r = e.set), new Dl(s, r, n);
  }
  const Zn = {}, ls = /* @__PURE__ */ new WeakMap();
  let Wt;
  function Ll(e, t = false, n = Wt) {
    if (n) {
      let s = ls.get(n);
      s || ls.set(n, s = []), s.push(e);
    }
  }
  function Ul(e, t, n = fe) {
    const { immediate: s, deep: r, once: i, scheduler: o, augmentJob: a, call: l } = n, c = ($) => r ? $ : Ye($) || r === false || r === 0 ? xt($, 1) : xt($);
    let u, f, m, _, P = false, R = false;
    if (Pe(e) ? (f = () => e.value, P = Ye(e)) : Bt(e) ? (f = () => c(e), P = true) : G(e) ? (R = true, P = e.some(($) => Bt($) || Ye($)), f = () => e.map(($) => {
      if (Pe($)) return $.value;
      if (Bt($)) return c($);
      if (Y($)) return l ? l($, 2) : $();
    })) : Y(e) ? t ? f = l ? () => l(e, 2) : e : f = () => {
      if (m) {
        vt();
        try {
          m();
        } finally {
          yt();
        }
      }
      const $ = Wt;
      Wt = u;
      try {
        return l ? l(e, 3, [
          _
        ]) : e(_);
      } finally {
        Wt = $;
      }
    } : f = pt, t && r) {
      const $ = f, X = r === true ? 1 / 0 : r;
      f = () => xt($(), X);
    }
    const M = pl(), L = () => {
      u.stop(), M && M.active && Pr(M.effects, u);
    };
    if (i && t) {
      const $ = t;
      t = (...X) => {
        const se = $(...X);
        return L(), se;
      };
    }
    let E = R ? new Array(e.length).fill(Zn) : Zn;
    const H = ($) => {
      if (!(!(u.flags & 1) || !u.dirty && !$)) if (t) {
        const X = u.run();
        if ($ || r || P || (R ? X.some((se, re) => ht(se, E[re])) : ht(X, E))) {
          m && m();
          const se = Wt;
          Wt = u;
          try {
            const re = [
              X,
              E === Zn ? void 0 : R && E[0] === Zn ? [] : E,
              _
            ];
            E = X, l ? l(t, 3, re) : t(...re);
          } finally {
            Wt = se;
          }
        }
      } else u.run();
    };
    return a && a(H), u = new Ao(f), u.scheduler = o ? () => o(H, false) : H, _ = ($) => Ll($, false, u), m = u.onStop = () => {
      const $ = ls.get(u);
      if ($) {
        if (l) l($, 4);
        else for (const X of $) X();
        ls.delete(u);
      }
    }, t ? s ? H(true) : E = u.run() : o ? o(H.bind(null, true), true) : u.run(), L.pause = u.pause.bind(u), L.resume = u.resume.bind(u), L.stop = L, L;
  }
  function xt(e, t = 1 / 0, n) {
    if (t <= 0 || !le(e) || e.__v_skip || (n = n || /* @__PURE__ */ new Map(), (n.get(e) || 0) >= t)) return e;
    if (n.set(e, t), t--, Pe(e)) xt(e.value, t, n);
    else if (G(e)) for (let s = 0; s < e.length; s++) xt(e[s], t, n);
    else if (dn(e) || en(e)) e.forEach((s) => {
      xt(s, t, n);
    });
    else if (_o(e)) {
      for (const s in e) xt(e[s], t, n);
      for (const s of Object.getOwnPropertySymbols(e)) Object.prototype.propertyIsEnumerable.call(e, s) && xt(e[s], t, n);
    }
    return e;
  }
  function Gn(e, t, n, s) {
    try {
      return s ? e(...s) : e();
    } catch (r) {
      Rs(r, t, n);
    }
  }
  function nt(e, t, n, s) {
    if (Y(e)) {
      const r = Gn(e, t, n, s);
      return r && vo(r) && r.catch((i) => {
        Rs(i, t, n);
      }), r;
    }
    if (G(e)) {
      const r = [];
      for (let i = 0; i < e.length; i++) r.push(nt(e[i], t, n, s));
      return r;
    }
  }
  function Rs(e, t, n, s = true) {
    const r = t ? t.vnode : null, { errorHandler: i, throwUnhandledErrorInProduction: o } = t && t.appContext.config || fe;
    if (t) {
      let a = t.parent;
      const l = t.proxy, c = `https://vuejs.org/error-reference/#runtime-${n}`;
      for (; a; ) {
        const u = a.ec;
        if (u) {
          for (let f = 0; f < u.length; f++) if (u[f](e, l, c) === false) return;
        }
        a = a.parent;
      }
      if (i) {
        vt(), Gn(i, null, 10, [
          e,
          l,
          c
        ]), yt();
        return;
      }
    }
    Hl(e, n, r, s, o);
  }
  function Hl(e, t, n, s = true, r = false) {
    if (r) throw e;
    console.error(e);
  }
  const $e = [];
  let ut = -1;
  const tn = [];
  let Nt = null, Yt = 0;
  const Wo = Promise.resolve();
  let cs = null;
  function gn(e) {
    const t = cs || Wo;
    return e ? t.then(this ? e.bind(this) : e) : t;
  }
  function Fl(e) {
    let t = ut + 1, n = $e.length;
    for (; t < n; ) {
      const s = t + n >>> 1, r = $e[s], i = Fn(r);
      i < e || i === e && r.flags & 2 ? t = s + 1 : n = s;
    }
    return t;
  }
  function Hr(e) {
    if (!(e.flags & 1)) {
      const t = Fn(e), n = $e[$e.length - 1];
      !n || !(e.flags & 2) && t >= Fn(n) ? $e.push(e) : $e.splice(Fl(t), 0, e), e.flags |= 1, qo();
    }
  }
  function qo() {
    cs || (cs = Wo.then(Bo));
  }
  function Wl(e) {
    G(e) ? tn.push(...e) : Nt && e.id === -1 ? Nt.splice(Yt + 1, 0, e) : e.flags & 1 || (tn.push(e), e.flags |= 1), qo();
  }
  function ii(e, t, n = ut + 1) {
    for (; n < $e.length; n++) {
      const s = $e[n];
      if (s && s.flags & 2) {
        if (e && s.id !== e.uid) continue;
        $e.splice(n, 1), n--, s.flags & 4 && (s.flags &= -2), s(), s.flags & 4 || (s.flags &= -2);
      }
    }
  }
  function Vo(e) {
    if (tn.length) {
      const t = [
        ...new Set(tn)
      ].sort((n, s) => Fn(n) - Fn(s));
      if (tn.length = 0, Nt) {
        Nt.push(...t);
        return;
      }
      for (Nt = t, Yt = 0; Yt < Nt.length; Yt++) {
        const n = Nt[Yt];
        n.flags & 4 && (n.flags &= -2), n.flags & 8 || n(), n.flags &= -2;
      }
      Nt = null, Yt = 0;
    }
  }
  const Fn = (e) => e.id == null ? e.flags & 2 ? -1 : 1 / 0 : e.id;
  function Bo(e) {
    try {
      for (ut = 0; ut < $e.length; ut++) {
        const t = $e[ut];
        t && !(t.flags & 8) && (t.flags & 4 && (t.flags &= -2), Gn(t, t.i, t.i ? 15 : 14), t.flags & 4 || (t.flags &= -2));
      }
    } finally {
      for (; ut < $e.length; ut++) {
        const t = $e[ut];
        t && (t.flags &= -2);
      }
      ut = -1, $e.length = 0, Vo(), cs = null, ($e.length || tn.length) && Bo();
    }
  }
  let Be = null, Ko = null;
  function us(e) {
    const t = Be;
    return Be = e, Ko = e && e.type.__scopeId || null, t;
  }
  function ql(e, t = Be, n) {
    if (!t || e._n) return e;
    const s = (...r) => {
      s._d && hs(-1);
      const i = us(t);
      let o;
      try {
        o = e(...r);
      } finally {
        us(i), s._d && hs(1);
      }
      return o;
    };
    return s._n = true, s._c = true, s._d = true, s;
  }
  function qs(e, t) {
    if (Be === null) return e;
    const n = Os(Be), s = e.dirs || (e.dirs = []);
    for (let r = 0; r < t.length; r++) {
      let [i, o, a, l = fe] = t[r];
      i && (Y(i) && (i = {
        mounted: i,
        updated: i
      }), i.deep && xt(o), s.push({
        dir: i,
        instance: n,
        value: o,
        oldValue: void 0,
        arg: a,
        modifiers: l
      }));
    }
    return e;
  }
  function Ht(e, t, n, s) {
    const r = e.dirs, i = t && t.dirs;
    for (let o = 0; o < r.length; o++) {
      const a = r[o];
      i && (a.oldValue = i[o].value);
      let l = a.dir[s];
      l && (vt(), nt(l, n, 8, [
        e.el,
        a,
        e,
        t
      ]), yt());
    }
  }
  function rs(e, t) {
    if (Re) {
      let n = Re.provides;
      const s = Re.parent && Re.parent.provides;
      s === n && (n = Re.provides = Object.create(s)), n[e] = t;
    }
  }
  function et(e, t, n = false) {
    const s = Vc();
    if (s || nn) {
      let r = nn ? nn._context.provides : s ? s.parent == null || s.ce ? s.vnode.appContext && s.vnode.appContext.provides : s.parent.provides : void 0;
      if (r && e in r) return r[e];
      if (arguments.length > 1) return n && Y(t) ? t.call(s && s.proxy) : t;
    }
  }
  const Vl = Symbol.for("v-scx"), Bl = () => et(Vl);
  function Kt(e, t, n) {
    return Go(e, t, n);
  }
  function Go(e, t, n = fe) {
    const { immediate: s, deep: r, flush: i, once: o } = n, a = Ae({}, n), l = t && s || !t && i !== "post";
    let c;
    if (qn) {
      if (i === "sync") {
        const _ = Bl();
        c = _.__watcherHandles || (_.__watcherHandles = []);
      } else if (!l) {
        const _ = () => {
        };
        return _.stop = pt, _.resume = pt, _.pause = pt, _;
      }
    }
    const u = Re;
    a.call = (_, P, R) => nt(_, u, P, R);
    let f = false;
    i === "post" ? a.scheduler = (_) => {
      Fe(_, u && u.suspense);
    } : i !== "sync" && (f = true, a.scheduler = (_, P) => {
      P ? _() : Hr(_);
    }), a.augmentJob = (_) => {
      t && (_.flags |= 4), f && (_.flags |= 2, u && (_.id = u.uid, _.i = u));
    };
    const m = Ul(e, t, a);
    return qn && (c ? c.push(m) : l && m()), m;
  }
  function Kl(e, t, n) {
    const s = this.proxy, r = ve(e) ? e.includes(".") ? Jo(s, e) : () => s[e] : e.bind(s, s);
    let i;
    Y(t) ? i = t : (i = t.handler, n = t);
    const o = Jn(this), a = Go(r, i.bind(s), n);
    return o(), a;
  }
  function Jo(e, t) {
    const n = t.split(".");
    return () => {
      let s = e;
      for (let r = 0; r < n.length && s; r++) s = s[n[r]];
      return s;
    };
  }
  const Gl = Symbol("_vte"), Jl = (e) => e.__isTeleport, Vs = Symbol("_leaveCb");
  function Fr(e, t) {
    e.shapeFlag & 6 && e.component ? (e.transition = t, Fr(e.component.subTree, t)) : e.shapeFlag & 128 ? (e.ssContent.transition = t.clone(e.ssContent), e.ssFallback.transition = t.clone(e.ssFallback)) : e.transition = t;
  }
  function Ut(e, t) {
    return Y(e) ? Ae({
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
      e.forEach((R, M) => $n(R, t && (G(t) ? t[M] : t), n, s, r));
      return;
    }
    if (Mn(s) && !r) {
      s.shapeFlag & 512 && s.type.__asyncResolved && s.component.subTree.component && $n(e, t, n, s.component.subTree);
      return;
    }
    const i = s.shapeFlag & 4 ? Os(s.component) : s.el, o = r ? null : i, { i: a, r: l } = e, c = t && t.r, u = a.refs === fe ? a.refs = {} : a.refs, f = a.setupState, m = oe(f), _ = f === fe ? mo : (R) => oi(u, R) ? false : ae(m, R), P = (R, M) => !(M && oi(u, M));
    if (c != null && c !== l) {
      if (ai(t), ve(c)) u[c] = null, _(c) && (f[c] = null);
      else if (Pe(c)) {
        const R = t;
        P(c, R.k) && (c.value = null), R.k && (u[R.k] = null);
      }
    }
    if (Y(l)) {
      vt();
      try {
        Gn(l, a, 12, [
          o,
          u
        ]);
      } finally {
        yt();
      }
    } else {
      const R = ve(l), M = Pe(l);
      if (R || M) {
        const L = () => {
          if (e.f) {
            const E = R ? _(l) ? f[l] : u[l] : P() || !e.k ? l.value : u[e.k];
            if (r) G(E) && Pr(E, i);
            else if (G(E)) E.includes(i) || E.push(i);
            else if (R) u[l] = [
              i
            ], _(l) && (f[l] = u[l]);
            else {
              const H = [
                i
              ];
              P(l, e.k) && (l.value = H), e.k && (u[e.k] = H);
            }
          } else R ? (u[l] = o, _(l) && (f[l] = o)) : M && (P(l, e.k) && (l.value = o), e.k && (u[e.k] = o));
        };
        if (o) {
          const E = () => {
            L(), fs.delete(e);
          };
          E.id = -1, fs.set(e, E), Fe(E, n);
        } else ai(e), L();
      }
    }
  }
  function ai(e) {
    const t = fs.get(e);
    t && (t.flags |= 8, fs.delete(e));
  }
  xs().requestIdleCallback;
  xs().cancelIdleCallback;
  const Mn = (e) => !!e.type.__asyncLoader, Yo = (e) => e.type.__isKeepAlive;
  function zl(e, t) {
    Qo(e, "a", t);
  }
  function Yl(e, t) {
    Qo(e, "da", t);
  }
  function Qo(e, t, n = Re) {
    const s = e.__wdc || (e.__wdc = () => {
      let r = n;
      for (; r; ) {
        if (r.isDeactivated) return;
        r = r.parent;
      }
      return e();
    });
    if (Ps(t, s, n), n) {
      let r = n.parent;
      for (; r && r.parent; ) Yo(r.parent.vnode) && Ql(s, t, n, r), r = r.parent;
    }
  }
  function Ql(e, t, n, s) {
    const r = Ps(t, e, s, true);
    Wr(() => {
      Pr(s[t], r);
    }, n);
  }
  function Ps(e, t, n = Re, s = false) {
    if (n) {
      const r = n[e] || (n[e] = []), i = t.__weh || (t.__weh = (...o) => {
        vt();
        const a = Jn(n), l = nt(t, n, e, o);
        return a(), yt(), l;
      });
      return s ? r.unshift(i) : r.push(i), i;
    }
  }
  const Pt = (e) => (t, n = Re) => {
    (!qn || e === "sp") && Ps(e, (...s) => t(...s), n);
  }, Xl = Pt("bm"), Xo = Pt("m"), Zl = Pt("bu"), ec = Pt("u"), tc = Pt("bum"), Wr = Pt("um"), nc = Pt("sp"), sc = Pt("rtg"), rc = Pt("rtc");
  function ic(e, t = Re) {
    Ps("ec", e, t);
  }
  const oc = "directives", ac = Symbol.for("v-ndc");
  function lc(e) {
    return cc(oc, e);
  }
  function cc(e, t, n = true, s = false) {
    const r = Be || Re;
    if (r) {
      const i = r.type, o = li(r[e] || i[e], t) || li(r.appContext[e], t);
      return !o && s ? i : o;
    }
  }
  function li(e, t) {
    return e && (e[t] || e[Ge(t)] || e[Ir(Ge(t))]);
  }
  function Ce(e, t, n, s) {
    let r;
    const i = n, o = G(e);
    if (o || ve(e)) {
      const a = o && Bt(e);
      let l = false, c = false;
      a && (l = !Ye(e), c = Rt(e), e = Cs(e)), r = new Array(e.length);
      for (let u = 0, f = e.length; u < f; u++) r[u] = t(l ? c ? on(tt(e[u])) : tt(e[u]) : e[u], u, void 0, i);
    } else if (typeof e == "number") {
      r = new Array(e);
      for (let a = 0; a < e; a++) r[a] = t(a + 1, a, void 0, i);
    } else if (le(e)) if (e[Symbol.iterator]) r = Array.from(e, (a, l) => t(a, l, void 0, i));
    else {
      const a = Object.keys(e);
      r = new Array(a.length);
      for (let l = 0, c = a.length; l < c; l++) {
        const u = a[l];
        r[l] = t(e[u], u, l, i);
      }
    }
    else r = [];
    return r;
  }
  const lr = (e) => e ? ya(e) ? Os(e) : lr(e.parent) : null, Dn = Ae(/* @__PURE__ */ Object.create(null), {
    $: (e) => e,
    $el: (e) => e.vnode.el,
    $data: (e) => e.data,
    $props: (e) => e.props,
    $attrs: (e) => e.attrs,
    $slots: (e) => e.slots,
    $refs: (e) => e.refs,
    $parent: (e) => lr(e.parent),
    $root: (e) => lr(e.root),
    $host: (e) => e.ce,
    $emit: (e) => e.emit,
    $options: (e) => ea(e),
    $forceUpdate: (e) => e.f || (e.f = () => {
      Hr(e.update);
    }),
    $nextTick: (e) => e.n || (e.n = gn.bind(e.proxy)),
    $watch: (e) => Kl.bind(e)
  }), Bs = (e, t) => e !== fe && !e.__isScriptSetup && ae(e, t), uc = {
    get({ _: e }, t) {
      if (t === "__v_skip") return true;
      const { ctx: n, setupState: s, data: r, props: i, accessCache: o, type: a, appContext: l } = e;
      if (t[0] !== "$") {
        const m = o[t];
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
          if (Bs(s, t)) return o[t] = 1, s[t];
          if (r !== fe && ae(r, t)) return o[t] = 2, r[t];
          if (ae(i, t)) return o[t] = 3, i[t];
          if (n !== fe && ae(n, t)) return o[t] = 4, n[t];
          cr && (o[t] = 0);
        }
      }
      const c = Dn[t];
      let u, f;
      if (c) return t === "$attrs" && xe(e.attrs, "get", ""), c(e);
      if ((u = a.__cssModules) && (u = u[t])) return u;
      if (n !== fe && ae(n, t)) return o[t] = 4, n[t];
      if (f = l.config.globalProperties, ae(f, t)) return f[t];
    },
    set({ _: e }, t, n) {
      const { data: s, setupState: r, ctx: i } = e;
      return Bs(r, t) ? (r[t] = n, true) : s !== fe && ae(s, t) ? (s[t] = n, true) : ae(e.props, t) || t[0] === "$" && t.slice(1) in e ? false : (i[t] = n, true);
    },
    has({ _: { data: e, setupState: t, accessCache: n, ctx: s, appContext: r, props: i, type: o } }, a) {
      let l;
      return !!(n[a] || e !== fe && a[0] !== "$" && ae(e, a) || Bs(t, a) || ae(i, a) || ae(s, a) || ae(Dn, a) || ae(r.config.globalProperties, a) || (l = o.__cssModules) && l[a]);
    },
    defineProperty(e, t, n) {
      return n.get != null ? e._.accessCache[t] = 0 : ae(n, "value") && this.set(e, t, n.value, null), Reflect.defineProperty(e, t, n);
    }
  };
  function ci(e) {
    return G(e) ? e.reduce((t, n) => (t[n] = null, t), {}) : e;
  }
  let cr = true;
  function fc(e) {
    const t = ea(e), n = e.proxy, s = e.ctx;
    cr = false, t.beforeCreate && ui(t.beforeCreate, e, "bc");
    const { data: r, computed: i, methods: o, watch: a, provide: l, inject: c, created: u, beforeMount: f, mounted: m, beforeUpdate: _, updated: P, activated: R, deactivated: M, beforeDestroy: L, beforeUnmount: E, destroyed: H, unmounted: $, render: X, renderTracked: se, renderTriggered: re, errorCaptured: Me, serverPrefetch: De, expose: je, inheritAttrs: Le, components: Qe, directives: Ue, filters: _t } = t;
    if (c && dc(c, s, null), o) for (const te in o) {
      const ee = o[te];
      Y(ee) && (s[te] = ee.bind(n));
    }
    if (r) {
      const te = r.call(n, n);
      le(te) && (e.data = pn(te));
    }
    if (cr = true, i) for (const te in i) {
      const ee = i[te], Te = Y(ee) ? ee.bind(n, n) : Y(ee.get) ? ee.get.bind(n, n) : pt, O = !Y(ee) && Y(ee.set) ? ee.set.bind(n) : pt, D = Q({
        get: Te,
        set: O
      });
      Object.defineProperty(s, te, {
        enumerable: true,
        configurable: true,
        get: () => D.value,
        set: (F) => D.value = F
      });
    }
    if (a) for (const te in a) Zo(a[te], s, n, te);
    if (l) {
      const te = Y(l) ? l.call(n) : l;
      Reflect.ownKeys(te).forEach((ee) => {
        rs(ee, te[ee]);
      });
    }
    u && ui(u, e, "c");
    function ge(te, ee) {
      G(ee) ? ee.forEach((Te) => te(Te.bind(n))) : ee && te(ee.bind(n));
    }
    if (ge(Xl, f), ge(Xo, m), ge(Zl, _), ge(ec, P), ge(zl, R), ge(Yl, M), ge(ic, Me), ge(rc, se), ge(sc, re), ge(tc, E), ge(Wr, $), ge(nc, De), G(je)) if (je.length) {
      const te = e.exposed || (e.exposed = {});
      je.forEach((ee) => {
        Object.defineProperty(te, ee, {
          get: () => n[ee],
          set: (Te) => n[ee] = Te,
          enumerable: true
        });
      });
    } else e.exposed || (e.exposed = {});
    X && e.render === pt && (e.render = X), Le != null && (e.inheritAttrs = Le), Qe && (e.components = Qe), Ue && (e.directives = Ue), De && zo(e);
  }
  function dc(e, t, n = pt) {
    G(e) && (e = ur(e));
    for (const s in e) {
      const r = e[s];
      let i;
      le(r) ? "default" in r ? i = et(r.from || s, r.default, true) : i = et(r.from || s) : i = et(r), Pe(i) ? Object.defineProperty(t, s, {
        enumerable: true,
        configurable: true,
        get: () => i.value,
        set: (o) => i.value = o
      }) : t[s] = i;
    }
  }
  function ui(e, t, n) {
    nt(G(e) ? e.map((s) => s.bind(t.proxy)) : e.bind(t.proxy), t, n);
  }
  function Zo(e, t, n, s) {
    let r = s.includes(".") ? Jo(n, s) : () => n[s];
    if (ve(e)) {
      const i = t[e];
      Y(i) && Kt(r, i);
    } else if (Y(e)) Kt(r, e.bind(n));
    else if (le(e)) if (G(e)) e.forEach((i) => Zo(i, t, n, s));
    else {
      const i = Y(e.handler) ? e.handler.bind(n) : t[e.handler];
      Y(i) && Kt(r, i, e);
    }
  }
  function ea(e) {
    const t = e.type, { mixins: n, extends: s } = t, { mixins: r, optionsCache: i, config: { optionMergeStrategies: o } } = e.appContext, a = i.get(t);
    let l;
    return a ? l = a : !r.length && !n && !s ? l = t : (l = {}, r.length && r.forEach((c) => ds(l, c, o, true)), ds(l, t, o)), le(t) && i.set(t, l), l;
  }
  function ds(e, t, n, s = false) {
    const { mixins: r, extends: i } = t;
    i && ds(e, i, n, true), r && r.forEach((o) => ds(e, o, n, true));
    for (const o in t) if (!(s && o === "expose")) {
      const a = hc[o] || n && n[o];
      e[o] = a ? a(e[o], t[o]) : t[o];
    }
    return e;
  }
  const hc = {
    data: fi,
    props: di,
    emits: di,
    methods: En,
    computed: En,
    beforeCreate: Oe,
    created: Oe,
    beforeMount: Oe,
    mounted: Oe,
    beforeUpdate: Oe,
    updated: Oe,
    beforeDestroy: Oe,
    beforeUnmount: Oe,
    destroyed: Oe,
    unmounted: Oe,
    activated: Oe,
    deactivated: Oe,
    errorCaptured: Oe,
    serverPrefetch: Oe,
    components: En,
    directives: En,
    watch: gc,
    provide: fi,
    inject: pc
  };
  function fi(e, t) {
    return t ? e ? function() {
      return Ae(Y(e) ? e.call(this, this) : e, Y(t) ? t.call(this, this) : t);
    } : t : e;
  }
  function pc(e, t) {
    return En(ur(e), ur(t));
  }
  function ur(e) {
    if (G(e)) {
      const t = {};
      for (let n = 0; n < e.length; n++) t[e[n]] = e[n];
      return t;
    }
    return e;
  }
  function Oe(e, t) {
    return e ? [
      ...new Set([].concat(e, t))
    ] : t;
  }
  function En(e, t) {
    return e ? Ae(/* @__PURE__ */ Object.create(null), e, t) : t;
  }
  function di(e, t) {
    return e ? G(e) && G(t) ? [
      .../* @__PURE__ */ new Set([
        ...e,
        ...t
      ])
    ] : Ae(/* @__PURE__ */ Object.create(null), ci(e), ci(t ?? {})) : t;
  }
  function gc(e, t) {
    if (!e) return t;
    if (!t) return e;
    const n = Ae(/* @__PURE__ */ Object.create(null), e);
    for (const s in t) n[s] = Oe(e[s], t[s]);
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
      Y(s) || (s = Ae({}, s)), r != null && !le(r) && (r = null);
      const i = ta(), o = /* @__PURE__ */ new WeakSet(), a = [];
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
          return o.has(u) || (u && Y(u.install) ? (o.add(u), u.install(c, ...f)) : Y(u) && (o.add(u), u(c, ...f))), c;
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
            const _ = c._ceVNode || Ee(s, r);
            return _.appContext = i, m === true ? m = "svg" : m === false && (m = void 0), e(_, u, m), l = true, c._container = u, u.__vue_app__ = c, Os(_.component);
          }
        },
        onUnmount(u) {
          a.push(u);
        },
        unmount() {
          l && (nt(a, c._instance, 16), e(null, c._container), delete c._container.__vue_app__);
        },
        provide(u, f) {
          return i.provides[u] = f, c;
        },
        runWithContext(u) {
          const f = nn;
          nn = c;
          try {
            return u();
          } finally {
            nn = f;
          }
        }
      };
      return c;
    };
  }
  let nn = null;
  const yc = (e, t) => t === "modelValue" || t === "model-value" ? e.modelModifiers : e[`${t}Modifiers`] || e[`${Ge(t)}Modifiers`] || e[`${Lt(t)}Modifiers`];
  function _c(e, t, ...n) {
    if (e.isUnmounted) return;
    const s = e.vnode.props || fe;
    let r = n;
    const i = t.startsWith("update:"), o = i && yc(s, t.slice(7));
    o && (o.trim && (r = n.map((u) => ve(u) ? u.trim() : u)), o.number && (r = n.map(wo)));
    let a, l = s[a = Ls(t)] || s[a = Ls(Ge(t))];
    !l && i && (l = s[a = Ls(Lt(t))]), l && nt(l, e, 6, r);
    const c = s[a + "Once"];
    if (c) {
      if (!e.emitted) e.emitted = {};
      else if (e.emitted[a]) return;
      e.emitted[a] = true, nt(c, e, 6, r);
    }
  }
  const bc = /* @__PURE__ */ new WeakMap();
  function na(e, t, n = false) {
    const s = n ? bc : t.emitsCache, r = s.get(e);
    if (r !== void 0) return r;
    const i = e.emits;
    let o = {}, a = false;
    if (!Y(e)) {
      const l = (c) => {
        const u = na(c, t, true);
        u && (a = true, Ae(o, u));
      };
      !n && t.mixins.length && t.mixins.forEach(l), e.extends && l(e.extends), e.mixins && e.mixins.forEach(l);
    }
    return !i && !a ? (le(e) && s.set(e, null), null) : (G(i) ? i.forEach((l) => o[l] = null) : Ae(o, i), le(e) && s.set(e, o), o);
  }
  function Ts(e, t) {
    return !e || !ks(t) ? false : (t = t.slice(2), t = t === "Once" ? t : t.replace(/Once$/, ""), ae(e, t[0].toLowerCase() + t.slice(1)) || ae(e, Lt(t)) || ae(e, t));
  }
  function hi(e) {
    const { type: t, vnode: n, proxy: s, withProxy: r, propsOptions: [i], slots: o, attrs: a, emit: l, render: c, renderCache: u, props: f, data: m, setupState: _, ctx: P, inheritAttrs: R } = e, M = us(e);
    let L, E;
    try {
      if (n.shapeFlag & 4) {
        const $ = r || s, X = $;
        L = dt(c.call(X, $, u, f, _, m, P)), E = a;
      } else {
        const $ = t;
        L = dt($.length > 1 ? $(f, {
          attrs: a,
          slots: o,
          emit: l
        }) : $(f, null)), E = t.props ? a : wc(a);
      }
    } catch ($) {
      jn.length = 0, Rs($, e, 1), L = Ee(Dt);
    }
    let H = L;
    if (E && R !== false) {
      const $ = Object.keys(E), { shapeFlag: X } = H;
      $.length && X & 7 && (i && $.some(Es) && (E = Sc(E, i)), H = an(H, E, false, true));
    }
    return n.dirs && (H = an(H, null, false, true), H.dirs = H.dirs ? H.dirs.concat(n.dirs) : n.dirs), n.transition && Fr(H, n.transition), L = H, us(M), L;
  }
  const wc = (e) => {
    let t;
    for (const n in e) (n === "class" || n === "style" || ks(n)) && ((t || (t = {}))[n] = e[n]);
    return t;
  }, Sc = (e, t) => {
    const n = {};
    for (const s in e) (!Es(s) || !(s.slice(9) in t)) && (n[s] = e[s]);
    return n;
  };
  function kc(e, t, n) {
    const { props: s, children: r, component: i } = e, { props: o, children: a, patchFlag: l } = t, c = i.emitsOptions;
    if (t.dirs || t.transition) return true;
    if (n && l >= 0) {
      if (l & 1024) return true;
      if (l & 16) return s ? pi(s, o, c) : !!o;
      if (l & 8) {
        const u = t.dynamicProps;
        for (let f = 0; f < u.length; f++) {
          const m = u[f];
          if (sa(o, s, m) && !Ts(c, m)) return true;
        }
      }
    } else return (r || a) && (!a || !a.$stable) ? true : s === o ? false : s ? o ? pi(s, o, c) : true : !!o;
    return false;
  }
  function pi(e, t, n) {
    const s = Object.keys(t);
    if (s.length !== Object.keys(e).length) return true;
    for (let r = 0; r < s.length; r++) {
      const i = s[r];
      if (sa(t, e, i) && !Ts(n, i)) return true;
    }
    return false;
  }
  function sa(e, t, n) {
    const s = e[n], r = t[n];
    return n === "style" && le(s) && le(r) ? !hn(s, r) : s !== r;
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
    for (const o in e.propsOptions[0]) o in r || (r[o] = void 0);
    n ? e.props = s ? r : Uo(r) : e.type.props ? e.props = r : e.props = i, e.attrs = i;
  }
  function xc(e, t, n, s) {
    const { props: r, attrs: i, vnode: { patchFlag: o } } = e, a = oe(r), [l] = e.propsOptions;
    let c = false;
    if ((s || o > 0) && !(o & 16)) {
      if (o & 8) {
        const u = e.vnode.dynamicProps;
        for (let f = 0; f < u.length; f++) {
          let m = u[f];
          if (Ts(e.emitsOptions, m)) continue;
          const _ = t[m];
          if (l) if (ae(i, m)) _ !== i[m] && (i[m] = _, c = true);
          else {
            const P = Ge(m);
            r[P] = fr(l, a, P, _, e, false);
          }
          else _ !== i[m] && (i[m] = _, c = true);
        }
      }
    } else {
      aa(e, t, r, i) && (c = true);
      let u;
      for (const f in a) (!t || !ae(t, f) && ((u = Lt(f)) === f || !ae(t, u))) && (l ? n && (n[f] !== void 0 || n[u] !== void 0) && (r[f] = fr(l, a, f, void 0, e, true)) : delete r[f]);
      if (i !== a) for (const f in i) (!t || !ae(t, f)) && (delete i[f], c = true);
    }
    c && At(e.attrs, "set", "");
  }
  function aa(e, t, n, s) {
    const [r, i] = e.propsOptions;
    let o = false, a;
    if (t) for (let l in t) {
      if (In(l)) continue;
      const c = t[l];
      let u;
      r && ae(r, u = Ge(l)) ? !i || !i.includes(u) ? n[u] = c : (a || (a = {}))[u] = c : Ts(e.emitsOptions, l) || (!(l in s) || c !== s[l]) && (s[l] = c, o = true);
    }
    if (i) {
      const l = oe(n), c = a || fe;
      for (let u = 0; u < i.length; u++) {
        const f = i[u];
        n[f] = fr(r, l, f, c[f], e, !ae(c, f));
      }
    }
    return o;
  }
  function fr(e, t, n, s, r, i) {
    const o = e[n];
    if (o != null) {
      const a = ae(o, "default");
      if (a && s === void 0) {
        const l = o.default;
        if (o.type !== Function && !o.skipFactory && Y(l)) {
          const { propsDefaults: c } = r;
          if (n in c) s = c[n];
          else {
            const u = Jn(r);
            s = c[n] = l.call(null, t), u();
          }
        } else s = l;
        r.ce && r.ce._setProp(n, s);
      }
      o[0] && (i && !a ? s = false : o[1] && (s === "" || s === Lt(n)) && (s = true));
    }
    return s;
  }
  const Cc = /* @__PURE__ */ new WeakMap();
  function la(e, t, n = false) {
    const s = n ? Cc : t.propsCache, r = s.get(e);
    if (r) return r;
    const i = e.props, o = {}, a = [];
    let l = false;
    if (!Y(e)) {
      const u = (f) => {
        l = true;
        const [m, _] = la(f, t, true);
        Ae(o, m), _ && a.push(..._);
      };
      !n && t.mixins.length && t.mixins.forEach(u), e.extends && u(e.extends), e.mixins && e.mixins.forEach(u);
    }
    if (!i && !l) return le(e) && s.set(e, Zt), Zt;
    if (G(i)) for (let u = 0; u < i.length; u++) {
      const f = Ge(i[u]);
      gi(f) && (o[f] = fe);
    }
    else if (i) for (const u in i) {
      const f = Ge(u);
      if (gi(f)) {
        const m = i[u], _ = o[f] = G(m) || Y(m) ? {
          type: m
        } : Ae({}, m), P = _.type;
        let R = false, M = true;
        if (G(P)) for (let L = 0; L < P.length; ++L) {
          const E = P[L], H = Y(E) && E.name;
          if (H === "Boolean") {
            R = true;
            break;
          } else H === "String" && (M = false);
        }
        else R = Y(P) && P.name === "Boolean";
        _[0] = R, _[1] = M, (R || ae(_, "default")) && a.push(f);
      }
    }
    const c = [
      o,
      a
    ];
    return le(e) && s.set(e, c), c;
  }
  function gi(e) {
    return e[0] !== "$" && !In(e);
  }
  const qr = (e) => e === "_" || e === "_ctx" || e === "$stable", Vr = (e) => G(e) ? e.map(dt) : [
    dt(e)
  ], Rc = (e, t, n) => {
    if (t._n) return t;
    const s = ql((...r) => Vr(t(...r)), n);
    return s._c = false, s;
  }, ca = (e, t, n) => {
    const s = e._ctx;
    for (const r in e) {
      if (qr(r)) continue;
      const i = e[r];
      if (Y(i)) t[r] = Rc(r, i, s);
      else if (i != null) {
        const o = Vr(i);
        t[r] = () => o;
      }
    }
  }, ua = (e, t) => {
    const n = Vr(t);
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
    let i = true, o = fe;
    if (s.shapeFlag & 32) {
      const a = t._;
      a ? n && a === 1 ? i = false : fa(r, t, n) : (i = !t.$stable, ca(t, r)), o = t;
    } else t && (ua(e, t), o = {
      default: 1
    });
    if (i) for (const a in r) !qr(a) && o[a] == null && delete r[a];
  }, Fe = Mc;
  function Ic(e) {
    return Oc(e);
  }
  function Oc(e, t) {
    const n = xs();
    n.__VUE__ = true;
    const { insert: s, remove: r, patchProp: i, createElement: o, createText: a, createComment: l, setText: c, setElementText: u, parentNode: f, nextSibling: m, setScopeId: _ = pt, insertStaticContent: P } = e, R = (d, h, y, b = null, g = null, p = null, w = void 0, v = null, T = !!h.dynamicChildren) => {
      if (d === h) return;
      d && !_n(d, h) && (b = S(d), F(d, g, p, true), d = null), h.patchFlag === -2 && (T = false, h.dynamicChildren = null);
      const { type: A, ref: K, shapeFlag: j } = h;
      switch (A) {
        case Is:
          M(d, h, y, b);
          break;
        case Dt:
          L(d, h, y, b);
          break;
        case is:
          d == null && E(h, y, b, w);
          break;
        case de:
          Qe(d, h, y, b, g, p, w, v, T);
          break;
        default:
          j & 1 ? X(d, h, y, b, g, p, w, v, T) : j & 6 ? Ue(d, h, y, b, g, p, w, v, T) : (j & 64 || j & 128) && A.process(d, h, y, b, g, p, w, v, T, W);
      }
      K != null && g ? $n(K, d && d.ref, p, h || d, !h) : K == null && d && d.ref != null && $n(d.ref, null, p, d, true);
    }, M = (d, h, y, b) => {
      if (d == null) s(h.el = a(h.children), y, b);
      else {
        const g = h.el = d.el;
        h.children !== d.children && c(g, h.children);
      }
    }, L = (d, h, y, b) => {
      d == null ? s(h.el = l(h.children || ""), y, b) : h.el = d.el;
    }, E = (d, h, y, b) => {
      [d.el, d.anchor] = P(d.children, h, y, b, d.el, d.anchor);
    }, H = ({ el: d, anchor: h }, y, b) => {
      let g;
      for (; d && d !== h; ) g = m(d), s(d, y, b), d = g;
      s(h, y, b);
    }, $ = ({ el: d, anchor: h }) => {
      let y;
      for (; d && d !== h; ) y = m(d), r(d), d = y;
      r(h);
    }, X = (d, h, y, b, g, p, w, v, T) => {
      if (h.type === "svg" ? w = "svg" : h.type === "math" && (w = "mathml"), d == null) se(h, y, b, g, p, w, v, T);
      else {
        const A = d.el && d.el._isVueCE ? d.el : null;
        try {
          A && A._beginPatch(), De(d, h, g, p, w, v, T);
        } finally {
          A && A._endPatch();
        }
      }
    }, se = (d, h, y, b, g, p, w, v) => {
      let T, A;
      const { props: K, shapeFlag: j, transition: V, dirs: J } = d;
      if (T = d.el = o(d.type, p, K && K.is, K), j & 8 ? u(T, d.children) : j & 16 && Me(d.children, T, null, b, g, Ks(d, p), w, v), J && Ht(d, null, b, "created"), re(T, d, d.scopeId, w, b), K) {
        for (const ue in K) ue !== "value" && !In(ue) && i(T, ue, null, K[ue], p, b);
        "value" in K && i(T, "value", null, K.value, p), (A = K.onVnodeBeforeMount) && ct(A, b, d);
      }
      J && Ht(d, null, b, "beforeMount");
      const ne = Nc(g, V);
      ne && V.beforeEnter(T), s(T, h, y), ((A = K && K.onVnodeMounted) || ne || J) && Fe(() => {
        try {
          A && ct(A, b, d), ne && V.enter(T), J && Ht(d, null, b, "mounted");
        } finally {
        }
      }, g);
    }, re = (d, h, y, b, g) => {
      if (y && _(d, y), b) for (let p = 0; p < b.length; p++) _(d, b[p]);
      if (g) {
        let p = g.subTree;
        if (h === p || ga(p.type) && (p.ssContent === h || p.ssFallback === h)) {
          const w = g.vnode;
          re(d, w, w.scopeId, w.slotScopeIds, g.parent);
        }
      }
    }, Me = (d, h, y, b, g, p, w, v, T = 0) => {
      for (let A = T; A < d.length; A++) {
        const K = d[A] = v ? Et(d[A]) : dt(d[A]);
        R(null, K, h, y, b, g, p, w, v);
      }
    }, De = (d, h, y, b, g, p, w) => {
      const v = h.el = d.el;
      let { patchFlag: T, dynamicChildren: A, dirs: K } = h;
      T |= d.patchFlag & 16;
      const j = d.props || fe, V = h.props || fe;
      let J;
      if (y && Ft(y, false), (J = V.onVnodeBeforeUpdate) && ct(J, y, h, d), K && Ht(h, d, y, "beforeUpdate"), y && Ft(y, true), A && (!d.dynamicChildren || d.dynamicChildren.length !== A.length) && (T = 0, w = false, A = null), (j.innerHTML && V.innerHTML == null || j.textContent && V.textContent == null) && u(v, ""), A ? je(d.dynamicChildren, A, v, y, b, Ks(h, g), p) : w || ee(d, h, v, null, y, b, Ks(h, g), p, false), T > 0) {
        if (T & 16) Le(v, j, V, y, g);
        else if (T & 2 && j.class !== V.class && i(v, "class", null, V.class, g), T & 4 && i(v, "style", j.style, V.style, g), T & 8) {
          const ne = h.dynamicProps;
          for (let ue = 0; ue < ne.length; ue++) {
            const ce = ne[ue], _e = j[ce], Se = V[ce];
            (Se !== _e || ce === "value") && i(v, ce, _e, Se, g, y);
          }
        }
        T & 1 && d.children !== h.children && u(v, h.children);
      } else !w && A == null && Le(v, j, V, y, g);
      ((J = V.onVnodeUpdated) || K) && Fe(() => {
        J && ct(J, y, h, d), K && Ht(h, d, y, "updated");
      }, b);
    }, je = (d, h, y, b, g, p, w) => {
      for (let v = 0; v < h.length; v++) {
        const T = d[v], A = h[v], K = T.el && (T.type === de || !_n(T, A) || T.shapeFlag & 198) ? f(T.el) : y;
        R(T, A, K, null, b, g, p, w, true);
      }
    }, Le = (d, h, y, b, g) => {
      if (h !== y) {
        if (h !== fe) for (const p in h) !In(p) && !(p in y) && i(d, p, h[p], null, g, b);
        for (const p in y) {
          if (In(p)) continue;
          const w = y[p], v = h[p];
          w !== v && p !== "value" && i(d, p, v, w, g, b);
        }
        "value" in y && i(d, "value", h.value, y.value, g);
      }
    }, Qe = (d, h, y, b, g, p, w, v, T) => {
      const A = h.el = d ? d.el : a(""), K = h.anchor = d ? d.anchor : a("");
      let { patchFlag: j, dynamicChildren: V, slotScopeIds: J } = h;
      J && (v = v ? v.concat(J) : J), d == null ? (s(A, y, b), s(K, y, b), Me(h.children || [], y, K, g, p, w, v, T)) : j > 0 && j & 64 && V && d.dynamicChildren && d.dynamicChildren.length === V.length ? (je(d.dynamicChildren, V, y, g, p, w, v), (h.key != null || g && h === g.subTree) && da(d, h, true)) : ee(d, h, y, K, g, p, w, v, T);
    }, Ue = (d, h, y, b, g, p, w, v, T) => {
      h.slotScopeIds = v, d == null ? h.shapeFlag & 512 ? g.ctx.activate(h, y, b, w, T) : _t(h, y, b, g, p, w, T) : rt(d, h, T);
    }, _t = (d, h, y, b, g, p, w) => {
      const v = d.component = qc(d, b, g);
      if (Yo(d) && (v.ctx.renderer = W), Bc(v, false, w), v.asyncDep) {
        if (g && g.registerDep(v, ge, w), !d.el) {
          const T = v.subTree = Ee(Dt);
          L(null, T, h, y), d.placeholder = T.el;
        }
      } else ge(v, d, h, y, g, p, w);
    }, rt = (d, h, y) => {
      const b = h.component = d.component;
      if (kc(d, h, y)) if (b.asyncDep && !b.asyncResolved) {
        te(b, h, y);
        return;
      } else b.next = h, b.update();
      else h.el = d.el, b.vnode = h;
    }, ge = (d, h, y, b, g, p, w) => {
      const v = () => {
        if (d.isMounted) {
          let { next: j, bu: V, u: J, parent: ne, vnode: ue } = d;
          {
            const at = ha(d);
            if (at) {
              j && (j.el = ue.el, te(d, j, w)), at.asyncDep.then(() => {
                Fe(() => {
                  d.isUnmounted || A();
                }, g);
              });
              return;
            }
          }
          let ce = j, _e;
          Ft(d, false), j ? (j.el = ue.el, te(d, j, w)) : j = ue, V && ss(V), (_e = j.props && j.props.onVnodeBeforeUpdate) && ct(_e, ne, j, ue), Ft(d, true);
          const Se = hi(d), ot = d.subTree;
          d.subTree = Se, R(ot, Se, f(ot.el), S(ot), d, g, p), j.el = Se.el, ce === null && Ec(d, Se.el), J && Fe(J, g), (_e = j.props && j.props.onVnodeUpdated) && Fe(() => ct(_e, ne, j, ue), g);
        } else {
          let j;
          const { el: V, props: J } = h, { bm: ne, m: ue, parent: ce, root: _e, type: Se } = d, ot = Mn(h);
          Ft(d, false), ne && ss(ne), !ot && (j = J && J.onVnodeBeforeMount) && ct(j, ce, h), Ft(d, true);
          {
            _e.ce && _e.ce._hasShadowRoot() && _e.ce._injectChildStyle(Se, d.parent ? d.parent.type : void 0);
            const at = d.subTree = hi(d);
            R(null, at, y, b, d, g, p), h.el = at.el;
          }
          if (ue && Fe(ue, g), !ot && (j = J && J.onVnodeMounted)) {
            const at = h;
            Fe(() => ct(j, ce, at), g);
          }
          (h.shapeFlag & 256 || ce && Mn(ce.vnode) && ce.vnode.shapeFlag & 256) && d.a && Fe(d.a, g), d.isMounted = true, h = y = b = null;
        }
      };
      d.scope.on();
      const T = d.effect = new Ao(v);
      d.scope.off();
      const A = d.update = T.run.bind(T), K = d.job = T.runIfDirty.bind(T);
      K.i = d, K.id = d.uid, T.scheduler = () => Hr(K), Ft(d, true), A();
    }, te = (d, h, y) => {
      h.component = d;
      const b = d.vnode.props;
      d.vnode = h, d.next = null, xc(d, h.props, b, y), Tc(d, h.children, y), vt(), ii(d), yt();
    }, ee = (d, h, y, b, g, p, w, v, T = false) => {
      const A = d && d.children, K = d ? d.shapeFlag : 0, j = h.children, { patchFlag: V, shapeFlag: J } = h;
      if (V > 0) {
        if (V & 128) {
          O(A, j, y, b, g, p, w, v, T);
          return;
        } else if (V & 256) {
          Te(A, j, y, b, g, p, w, v, T);
          return;
        }
      }
      J & 8 ? (K & 16 && He(A, g, p), j !== A && u(y, j)) : K & 16 ? J & 16 ? O(A, j, y, b, g, p, w, v, T) : He(A, g, p, true) : (K & 8 && u(y, ""), J & 16 && Me(j, y, b, g, p, w, v, T));
    }, Te = (d, h, y, b, g, p, w, v, T) => {
      d = d || Zt, h = h || Zt;
      const A = d.length, K = h.length, j = Math.min(A, K);
      let V;
      for (V = 0; V < j; V++) {
        const J = h[V] = T ? Et(h[V]) : dt(h[V]);
        R(d[V], J, y, null, g, p, w, v, T);
      }
      A > K ? He(d, g, p, true, false, j) : Me(h, y, b, g, p, w, v, T, j);
    }, O = (d, h, y, b, g, p, w, v, T) => {
      let A = 0;
      const K = h.length;
      let j = d.length - 1, V = K - 1;
      for (; A <= j && A <= V; ) {
        const J = d[A], ne = h[A] = T ? Et(h[A]) : dt(h[A]);
        if (_n(J, ne)) R(J, ne, y, null, g, p, w, v, T);
        else break;
        A++;
      }
      for (; A <= j && A <= V; ) {
        const J = d[j], ne = h[V] = T ? Et(h[V]) : dt(h[V]);
        if (_n(J, ne)) R(J, ne, y, null, g, p, w, v, T);
        else break;
        j--, V--;
      }
      if (A > j) {
        if (A <= V) {
          const J = V + 1, ne = J < K ? h[J].el : b;
          for (; A <= V; ) R(null, h[A] = T ? Et(h[A]) : dt(h[A]), y, ne, g, p, w, v, T), A++;
        }
      } else if (A > V) for (; A <= j; ) F(d[A], g, p, true), A++;
      else {
        const J = A, ne = A, ue = /* @__PURE__ */ new Map();
        for (A = ne; A <= V; A++) {
          const qe = h[A] = T ? Et(h[A]) : dt(h[A]);
          qe.key != null && ue.set(qe.key, A);
        }
        let ce, _e = 0;
        const Se = V - ne + 1;
        let ot = false, at = 0;
        const vn = new Array(Se);
        for (A = 0; A < Se; A++) vn[A] = 0;
        for (A = J; A <= j; A++) {
          const qe = d[A];
          if (_e >= Se) {
            F(qe, g, p, true);
            continue;
          }
          let lt;
          if (qe.key != null) lt = ue.get(qe.key);
          else for (ce = ne; ce <= V; ce++) if (vn[ce - ne] === 0 && _n(qe, h[ce])) {
            lt = ce;
            break;
          }
          lt === void 0 ? F(qe, g, p, true) : (vn[lt - ne] = A + 1, lt >= at ? at = lt : ot = true, R(qe, h[lt], y, null, g, p, w, v, T), _e++);
        }
        const Xr = ot ? $c(vn) : Zt;
        for (ce = Xr.length - 1, A = Se - 1; A >= 0; A--) {
          const qe = ne + A, lt = h[qe], Zr = h[qe + 1], ei = qe + 1 < K ? Zr.el || pa(Zr) : b;
          vn[A] === 0 ? R(null, lt, y, ei, g, p, w, v, T) : ot && (ce < 0 || A !== Xr[ce] ? D(lt, y, ei, 2) : ce--);
        }
      }
    }, D = (d, h, y, b, g = null) => {
      const { el: p, type: w, transition: v, children: T, shapeFlag: A } = d;
      if (A & 6) {
        D(d.component.subTree, h, y, b);
        return;
      }
      if (A & 128) {
        d.suspense.move(h, y, b);
        return;
      }
      if (A & 64) {
        w.move(d, h, y, W);
        return;
      }
      if (w === de) {
        s(p, h, y);
        for (let j = 0; j < T.length; j++) D(T[j], h, y, b);
        s(d.anchor, h, y);
        return;
      }
      if (w === is) {
        H(d, h, y);
        return;
      }
      if (b !== 2 && A & 1 && v) if (b === 0) v.persisted && !p[Vs] ? s(p, h, y) : (v.beforeEnter(p), s(p, h, y), Fe(() => v.enter(p), g));
      else {
        const { leave: j, delayLeave: V, afterLeave: J } = v, ne = () => {
          d.ctx.isUnmounted ? r(p) : s(p, h, y);
        }, ue = () => {
          const ce = p._isLeaving || !!p[Vs];
          p._isLeaving && p[Vs](true), v.persisted && !ce ? ne() : j(p, () => {
            ne(), J && J();
          });
        };
        V ? V(p, ne, ue) : ue();
      }
      else s(p, h, y);
    }, F = (d, h, y, b = false, g = false) => {
      const { type: p, props: w, ref: v, children: T, dynamicChildren: A, shapeFlag: K, patchFlag: j, dirs: V, cacheIndex: J, memo: ne } = d;
      if (j === -2 && (g = false), v != null && (vt(), $n(v, null, y, d, true), yt()), J != null && (h.renderCache[J] = void 0), K & 256) {
        h.ctx.deactivate(d);
        return;
      }
      const ue = K & 1 && V, ce = !Mn(d);
      let _e;
      if (ce && (_e = w && w.onVnodeBeforeUnmount) && ct(_e, h, d), K & 6) it(d.component, y, b);
      else {
        if (K & 128) {
          d.suspense.unmount(y, b);
          return;
        }
        ue && Ht(d, null, h, "beforeUnmount"), K & 64 ? d.type.remove(d, h, y, W, b) : A && !A.hasOnce && (p !== de || j > 0 && j & 64) ? He(A, h, y, false, true) : (p === de && j & 384 || !g && K & 16) && He(T, h, y), b && ie(d);
      }
      const Se = ne != null && J == null;
      (ce && (_e = w && w.onVnodeUnmounted) || ue || Se) && Fe(() => {
        _e && ct(_e, h, d), ue && Ht(d, null, h, "unmounted"), Se && (d.el = null);
      }, y);
    }, ie = (d) => {
      const { type: h, el: y, anchor: b, transition: g } = d;
      if (h === de) {
        Ie(y, b);
        return;
      }
      if (h === is) {
        $(d);
        return;
      }
      const p = () => {
        r(y), g && !g.persisted && g.afterLeave && g.afterLeave();
      };
      if (d.shapeFlag & 1 && g && !g.persisted) {
        const { leave: w, delayLeave: v } = g, T = () => w(y, p);
        v ? v(d.el, p, T) : T();
      } else p();
    }, Ie = (d, h) => {
      let y;
      for (; d !== h; ) y = m(d), r(d), d = y;
      r(h);
    }, it = (d, h, y) => {
      const { bum: b, scope: g, job: p, subTree: w, um: v, m: T, a: A } = d;
      mi(T), mi(A), b && ss(b), g.stop(), p && (p.flags |= 8, F(w, d, h, y)), v && Fe(v, h), Fe(() => {
        d.isUnmounted = true;
      }, h);
    }, He = (d, h, y, b = false, g = false, p = 0) => {
      for (let w = p; w < d.length; w++) F(d[w], h, y, b, g);
    }, S = (d) => {
      if (d.shapeFlag & 6) return S(d.component.subTree);
      if (d.shapeFlag & 128) return d.suspense.next();
      const h = m(d.anchor || d.el), y = h && h[Gl];
      return y ? m(y) : h;
    };
    let U = false;
    const N = (d, h, y) => {
      let b;
      d == null ? h._vnode && (F(h._vnode, null, null, true), b = h._vnode.component) : R(h._vnode || null, d, h, null, null, null, y), h._vnode = d, U || (U = true, ii(b), Vo(), U = false);
    }, W = {
      p: R,
      um: F,
      m: D,
      r: ie,
      mt: _t,
      mc: Me,
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
  function Ks({ type: e, props: t }, n) {
    return n === "svg" && e === "foreignObject" || n === "mathml" && e === "annotation-xml" && t && t.encoding && t.encoding.includes("html") ? void 0 : n;
  }
  function Ft({ effect: e, job: t }, n) {
    n ? (e.flags |= 32, t.flags |= 4) : (e.flags &= -33, t.flags &= -5);
  }
  function Nc(e, t) {
    return (!e || e && !e.pendingBranch) && t && !t.persisted;
  }
  function da(e, t, n = false) {
    const s = e.children, r = t.children;
    if (G(s) && G(r)) for (let i = 0; i < s.length; i++) {
      const o = s[i];
      let a = r[i];
      a.shapeFlag & 1 && !a.dynamicChildren && ((a.patchFlag <= 0 || a.patchFlag === 32) && (a = r[i] = Et(r[i]), a.el = o.el), !n && a.patchFlag !== -2 && da(o, a)), a.type === Is && (a.patchFlag === -1 && (a = r[i] = Et(a)), a.el = o.el), a.type === Dt && !a.el && (a.el = o.el);
    }
  }
  function $c(e) {
    const t = e.slice(), n = [
      0
    ];
    let s, r, i, o, a;
    const l = e.length;
    for (s = 0; s < l; s++) {
      const c = e[s];
      if (c !== 0) {
        if (r = n[n.length - 1], e[r] < c) {
          t[s] = r, n.push(s);
          continue;
        }
        for (i = 0, o = n.length - 1; i < o; ) a = i + o >> 1, e[n[a]] < c ? i = a + 1 : o = a;
        c < e[n[i]] && (i > 0 && (t[s] = n[i - 1]), n[i] = s);
      }
    }
    for (i = n.length, o = n[i - 1]; i-- > 0; ) n[i] = o, o = t[o];
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
    t && t.pendingBranch ? G(e) ? t.effects.push(...e) : t.effects.push(e) : Wl(e);
  }
  const de = Symbol.for("v-fgt"), Is = Symbol.for("v-txt"), Dt = Symbol.for("v-cmt"), is = Symbol.for("v-stc"), jn = [];
  let Ke = null;
  function x(e = false) {
    jn.push(Ke = e ? null : []);
  }
  function Dc() {
    jn.pop(), Ke = jn[jn.length - 1] || null;
  }
  let Wn = 1;
  function hs(e, t = false) {
    Wn += e, e < 0 && Ke && t && (Ke.hasOnce = true);
  }
  function ma(e) {
    return e.dynamicChildren = Wn > 0 ? Ke || Zt : null, Dc(), Wn > 0 && Ke && Ke.push(e), e;
  }
  function C(e, t, n, s, r, i) {
    return ma(k(e, t, n, s, r, i, true));
  }
  function Br(e, t, n, s, r) {
    return ma(Ee(e, t, n, s, r, true));
  }
  function ps(e) {
    return e ? e.__v_isVNode === true : false;
  }
  function _n(e, t) {
    return e.type === t.type && e.key === t.key;
  }
  const va = ({ key: e }) => e ?? null, os = ({ ref: e, ref_key: t, ref_for: n }) => (typeof e == "number" && (e = "" + e), e != null ? ve(e) || Pe(e) || Y(e) ? {
    i: Be,
    r: e,
    k: t,
    f: !!n
  } : e : null);
  function k(e, t = null, n = null, s = 0, r = null, i = e === de ? 0 : 1, o = false, a = false) {
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
      ctx: Be
    };
    return a ? (gs(l, n), i & 128 && e.normalize(l)) : n && (l.shapeFlag |= ve(n) ? 8 : 16), Wn > 0 && !o && Ke && (l.patchFlag > 0 || i & 6) && l.patchFlag !== 32 && Ke.push(l), l;
  }
  const Ee = jc;
  function jc(e, t = null, n = null, s = 0, r = null, i = false) {
    if ((!e || e === ac) && (e = Dt), ps(e)) {
      const a = an(e, t, true);
      return n && gs(a, n), Wn > 0 && !i && Ke && (a.shapeFlag & 6 ? Ke[Ke.indexOf(e)] = a : Ke.push(a)), a.patchFlag = -2, a;
    }
    if (zc(e) && (e = e.__vccOpts), t) {
      t = Lc(t);
      let { class: a, style: l } = t;
      a && !ve(a) && (t.class = We(a)), le(l) && (Ur(l) && !G(l) && (l = Ae({}, l)), t.style = Gt(l));
    }
    const o = ve(e) ? 1 : ga(e) ? 128 : Jl(e) ? 64 : le(e) ? 4 : Y(e) ? 2 : 0;
    return k(e, t, n, s, r, o, i, true);
  }
  function Lc(e) {
    return e ? Ur(e) || oa(e) ? Ae({}, e) : e : null;
  }
  function an(e, t, n = false, s = false) {
    const { props: r, ref: i, patchFlag: o, children: a, transition: l } = e, c = t ? Hc(r || {}, t) : r, u = {
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
      children: a,
      target: e.target,
      targetStart: e.targetStart,
      targetAnchor: e.targetAnchor,
      staticCount: e.staticCount,
      shapeFlag: e.shapeFlag,
      patchFlag: t && e.type !== de ? o === -1 ? 16 : o | 16 : o,
      dynamicProps: e.dynamicProps,
      dynamicChildren: e.dynamicChildren,
      appContext: e.appContext,
      dirs: e.dirs,
      transition: l,
      component: e.component,
      suspense: e.suspense,
      ssContent: e.ssContent && an(e.ssContent),
      ssFallback: e.ssFallback && an(e.ssFallback),
      placeholder: e.placeholder,
      el: e.el,
      anchor: e.anchor,
      ctx: e.ctx,
      ce: e.ce
    };
    return l && s && Fr(u, l.clone(u)), u;
  }
  function Ct(e = " ", t = 0) {
    return Ee(Is, null, e, t);
  }
  function Uc(e, t) {
    const n = Ee(is, null, e);
    return n.staticCount = t, n;
  }
  function z(e = "", t = false) {
    return t ? (x(), Br(Dt, null, e)) : Ee(Dt, null, e);
  }
  function dt(e) {
    return e == null || typeof e == "boolean" ? Ee(Dt) : G(e) ? Ee(de, null, e.slice()) : ps(e) ? Et(e) : Ee(Is, null, String(e));
  }
  function Et(e) {
    return e.el === null && e.patchFlag !== -1 || e.memo ? e : an(e);
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
      !r && !oa(t) ? t._ctx = Be : r === 3 && Be && (Be.slots._ === 1 ? t._ = 1 : (t._ = 2, e.patchFlag |= 1024));
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
        _ctx: Be
      }, n = 32;
    } else t = String(t), s & 64 ? (n = 16, t = [
      Ct(t)
    ]) : n = 8;
    e.children = t, e.shapeFlag |= n;
  }
  function Hc(...e) {
    const t = {};
    for (let n = 0; n < e.length; n++) {
      const s = e[n];
      for (const r in s) if (r === "class") t.class !== s.class && (t.class = We([
        t.class,
        s.class
      ]));
      else if (r === "style") t.style = Gt([
        t.style,
        s.style
      ]);
      else if (ks(r)) {
        const i = t[r], o = s[r];
        o && i !== o && !(G(i) && i.includes(o)) ? t[r] = i ? [].concat(i, o) : o : o == null && i == null && !Es(r) && (t[r] = o);
      } else r !== "" && (t[r] = s[r]);
    }
    return t;
  }
  function ct(e, t, n, s = null) {
    nt(e, t, 7, [
      n,
      s
    ]);
  }
  const Fc = ta();
  let Wc = 0;
  function qc(e, t, n) {
    const s = e.type, r = (t ? t.appContext : e.appContext) || Fc, i = {
      uid: Wc++,
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
  let Re = null;
  const Vc = () => Re || Be;
  let ms, dr;
  {
    const e = xs(), t = (n, s) => {
      let r;
      return (r = e[n]) || (r = e[n] = []), r.push(s), (i) => {
        r.length > 1 ? r.forEach((o) => o(i)) : r[0](i);
      };
    };
    ms = t("__VUE_INSTANCE_SETTERS__", (n) => Re = n), dr = t("__VUE_SSR_SETTERS__", (n) => qn = n);
  }
  const Jn = (e) => {
    const t = Re;
    return ms(e), e.scope.on(), () => {
      e.scope.off(), ms(t);
    };
  }, vi = () => {
    Re && Re.scope.off(), ms(null);
  };
  function ya(e) {
    return e.vnode.shapeFlag & 4;
  }
  let qn = false;
  function Bc(e, t = false, n = false) {
    t && dr(t);
    const { props: s, children: r } = e.vnode, i = ya(e);
    Ac(e, s, i, t), Pc(e, r, n || t);
    const o = i ? Kc(e, t) : void 0;
    return t && dr(false), o;
  }
  function Kc(e, t) {
    const n = e.type;
    e.accessCache = /* @__PURE__ */ Object.create(null), e.proxy = new Proxy(e.ctx, uc);
    const { setup: s } = n;
    if (s) {
      vt();
      const r = e.setupContext = s.length > 1 ? Jc(e) : null, i = Jn(e), o = Gn(s, e, 0, [
        e.props,
        r
      ]), a = vo(o);
      if (yt(), i(), (a || e.sp) && !Mn(e) && zo(e), a) {
        if (o.then(vi, vi), t) return o.then((l) => {
          yi(e, l);
        }).catch((l) => {
          Rs(l, e, 0);
        });
        e.asyncDep = o;
      } else yi(e, o);
    } else _a(e);
  }
  function yi(e, t, n) {
    Y(t) ? e.type.__ssrInlineRender ? e.ssrRender = t : e.render = t : le(t) && (e.setupState = Fo(t)), _a(e);
  }
  function _a(e, t, n) {
    const s = e.type;
    e.render || (e.render = s.render || pt);
    {
      const r = Jn(e);
      vt();
      try {
        fc(e);
      } finally {
        yt(), r();
      }
    }
  }
  const Gc = {
    get(e, t) {
      return xe(e, "get", ""), e[t];
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
  function Os(e) {
    return e.exposed ? e.exposeProxy || (e.exposeProxy = new Proxy(Fo(Ol(e.exposed)), {
      get(t, n) {
        if (n in t) return t[n];
        if (n in Dn) return Dn[n](e);
      },
      has(t, n) {
        return n in t || n in Dn;
      }
    })) : e.proxy;
  }
  function zc(e) {
    return Y(e) && "__vccOpts" in e;
  }
  const Q = (e, t) => jl(e, t, qn);
  function ba(e, t, n) {
    try {
      hs(-1);
      const s = arguments.length;
      return s === 2 ? le(t) && !G(t) ? ps(t) ? Ee(e, null, [
        t
      ]) : Ee(e, t) : Ee(e, null, t) : (s > 3 ? n = Array.prototype.slice.call(arguments, 2) : s === 3 && ps(n) && (n = [
        n
      ]), Ee(e, t, n));
    } finally {
      hs(1);
    }
  }
  const Yc = "3.5.39";
  let hr;
  const _i = typeof window < "u" && window.trustedTypes;
  if (_i) try {
    hr = _i.createPolicy("vue", {
      createHTML: (e) => e
    });
  } catch {
  }
  const wa = hr ? (e) => hr.createHTML(e) : (e) => e, Qc = "http://www.w3.org/2000/svg", Xc = "http://www.w3.org/1998/Math/MathML", kt = typeof document < "u" ? document : null, bi = kt && kt.createElement("template"), Zc = {
    insert: (e, t, n) => {
      t.insertBefore(e, n || null);
    },
    remove: (e) => {
      const t = e.parentNode;
      t && t.removeChild(e);
    },
    createElement: (e, t, n, s) => {
      const r = t === "svg" ? kt.createElementNS(Qc, e) : t === "mathml" ? kt.createElementNS(Xc, e) : n ? kt.createElement(e, {
        is: n
      }) : kt.createElement(e);
      return e === "select" && s && s.multiple != null && r.setAttribute("multiple", s.multiple), r;
    },
    createText: (e) => kt.createTextNode(e),
    createComment: (e) => kt.createComment(e),
    setText: (e, t) => {
      e.nodeValue = t;
    },
    setElementText: (e, t) => {
      e.textContent = t;
    },
    parentNode: (e) => e.parentNode,
    nextSibling: (e) => e.nextSibling,
    querySelector: (e) => kt.querySelector(e),
    setScopeId(e, t) {
      e.setAttribute(t, "");
    },
    insertStaticContent(e, t, n, s, r, i) {
      const o = n ? n.previousSibling : t.lastChild;
      if (r && (r === i || r.nextSibling)) for (; t.insertBefore(r.cloneNode(true), n), !(r === i || !(r = r.nextSibling)); ) ;
      else {
        bi.innerHTML = wa(s === "svg" ? `<svg>${e}</svg>` : s === "mathml" ? `<math>${e}</math>` : e);
        const a = bi.content;
        if (s === "svg" || s === "mathml") {
          const l = a.firstChild;
          for (; l.firstChild; ) a.appendChild(l.firstChild);
          a.removeChild(l);
        }
        t.insertBefore(a, n);
      }
      return [
        o ? o.nextSibling : t.firstChild,
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
      if (t) if (ve(t)) for (const o of t.split(";")) {
        const a = o.slice(0, o.indexOf(":")).trim();
        n[a] == null && An(s, a, "");
      }
      else for (const o in t) n[o] == null && An(s, o, "");
      for (const o in n) {
        o === "display" && (i = true);
        const a = n[o];
        a != null ? au(e, o, !ve(t) && t ? t[o] : void 0, a) || An(s, o, a) : An(s, o, "");
      }
    } else if (r) {
      if (t !== n) {
        const o = s[su];
        o && (n += ";" + o), s.cssText = n, i = ru.test(n);
      }
    } else t && e.removeAttribute("style");
    wi in e && (e[wi] = i ? s.display : "", e[nu] && (s.display = "none"));
  }
  const Si = /\s*!important$/;
  function An(e, t, n) {
    if (G(n)) n.forEach((s) => An(e, t, s));
    else if (n == null && (n = ""), t.startsWith("--")) e.setProperty(t, n);
    else {
      const s = ou(e, t);
      Si.test(n) ? e.setProperty(Lt(s), n.replace(Si, ""), "important") : e[s] = n;
    }
  }
  const ki = [
    "Webkit",
    "Moz",
    "ms"
  ], Gs = {};
  function ou(e, t) {
    const n = Gs[t];
    if (n) return n;
    let s = Ge(t);
    if (s !== "filter" && s in e) return Gs[t] = s;
    s = Ir(s);
    for (let r = 0; r < ki.length; r++) {
      const i = ki[r] + s;
      if (i in e) return Gs[t] = i;
    }
    return t;
  }
  function au(e, t, n, s) {
    return e.tagName === "TEXTAREA" && (t === "width" || t === "height") && ve(s) && n === s;
  }
  const Ei = "http://www.w3.org/1999/xlink";
  function Ai(e, t, n, s, r, i = fl(t)) {
    s && t.startsWith("xlink:") ? n == null ? e.removeAttributeNS(Ei, t.slice(6, t.length)) : e.setAttributeNS(Ei, t, n) : n == null || i && !So(n) ? e.removeAttribute(t) : e.setAttribute(t, i ? "" : mt(n) ? String(n) : n);
  }
  function xi(e, t, n, s, r) {
    if (t === "innerHTML" || t === "textContent") {
      n != null && (e[t] = t === "innerHTML" ? wa(n) : n);
      return;
    }
    const i = e.tagName;
    if (t === "value" && i !== "PROGRESS" && !i.includes("-")) {
      const a = i === "OPTION" ? e.getAttribute("value") || "" : e.value, l = n == null ? e.type === "checkbox" ? "on" : "" : String(n);
      (a !== l || !("_value" in e)) && (e.value = l), n == null && e.removeAttribute(t), e._value = n;
      return;
    }
    let o = false;
    if (n === "" || n == null) {
      const a = typeof e[t];
      a === "boolean" ? n = So(n) : n == null && a === "string" ? (n = "", o = true) : a === "number" && (n = 0, o = true);
    }
    try {
      e[t] = n;
    } catch {
    }
    o && e.removeAttribute(r || t);
  }
  function Kr(e, t, n, s) {
    e.addEventListener(t, n, s);
  }
  function lu(e, t, n, s) {
    e.removeEventListener(t, n, s);
  }
  const Ci = Symbol("_vei");
  function cu(e, t, n, s, r = null) {
    const i = e[Ci] || (e[Ci] = {}), o = i[t];
    if (s && o) o.value = s;
    else {
      const [a, l] = du(t);
      if (s) {
        const c = i[t] = gu(s, r);
        Kr(e, a, c, l);
      } else o && (lu(e, a, o, l), i[t] = void 0);
    }
  }
  const uu = /(Once|Passive|Capture)$/, fu = /^on:?(?:Once|Passive|Capture)$/;
  function du(e) {
    let t, n;
    for (; (n = e.match(uu)) && !fu.test(e); ) t || (t = {}), e = e.slice(0, e.length - n[1].length), t[n[1].toLowerCase()] = true;
    return [
      e[2] === ":" ? e.slice(3) : Lt(e.slice(2)),
      t
    ];
  }
  let Js = 0;
  const hu = Promise.resolve(), pu = () => Js || (hu.then(() => Js = 0), Js = Date.now());
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
        const o = r.slice(), a = [
          s
        ];
        for (let l = 0; l < o.length && !s._stopped; l++) {
          const c = o[l];
          c && nt(c, t, 5, a);
        }
      } else nt(r, t, 5, [
        s
      ]);
    };
    return n.value = e, n.attached = pu(), n;
  }
  const Ri = (e) => e.charCodeAt(0) === 111 && e.charCodeAt(1) === 110 && e.charCodeAt(2) > 96 && e.charCodeAt(2) < 123, mu = (e, t, n, s, r, i) => {
    const o = r === "svg";
    t === "class" ? tu(e, s, o) : t === "style" ? iu(e, n, s) : ks(t) ? Es(t) || cu(e, t, n, s, i) : (t[0] === "." ? (t = t.slice(1), true) : t[0] === "^" ? (t = t.slice(1), false) : vu(e, t, s, o)) ? (xi(e, t, s), !e.tagName.includes("-") && (t === "value" || t === "checked" || t === "selected") && Ai(e, t, s, o, i, t !== "value")) : e._isVueCE && (yu(e, t) || e._def.__asyncLoader && (/[A-Z]/.test(t) || !ve(s))) ? xi(e, Ge(t), s, i, t) : (t === "true-value" ? e._trueValue = s : t === "false-value" && (e._falseValue = s), Ai(e, t, s, o));
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
    const s = Ge(t);
    return Array.isArray(n) ? n.some((r) => Ge(r) === s) : Object.keys(n).some((r) => Ge(r) === s);
  }
  const vs = (e) => {
    const t = e.props["onUpdate:modelValue"] || false;
    return G(t) ? (n) => ss(t, n) : t;
  }, sn = Symbol("_assign"), _u = {
    deep: true,
    created(e, t, n) {
      e[sn] = vs(n), Kr(e, "change", () => {
        const s = e._modelValue, r = Vn(e), i = e.checked, o = e[sn];
        if (G(s)) {
          const a = Or(s, r), l = a !== -1;
          if (i && !l) o(s.concat(r));
          else if (!i && l) {
            const c = [
              ...s
            ];
            c.splice(a, 1), o(c);
          }
        } else if (dn(s)) {
          const a = new Set(s);
          i ? a.add(r) : a.delete(r), o(a);
        } else o(Sa(e, i));
      });
    },
    mounted: Pi,
    beforeUpdate(e, t, n) {
      e[sn] = vs(n), Pi(e, t, n);
    }
  };
  function Pi(e, { value: t, oldValue: n }, s) {
    e._modelValue = t;
    let r;
    if (G(t)) r = Or(t, s.props.value) > -1;
    else if (dn(t)) r = t.has(s.props.value);
    else {
      if (t === n) return;
      r = hn(t, Sa(e, true));
    }
    e.checked !== r && (e.checked = r);
  }
  const bu = {
    deep: true,
    created(e, { value: t, modifiers: { number: n } }, s) {
      const r = dn(t);
      Kr(e, "change", () => {
        const i = Array.prototype.filter.call(e.options, (o) => o.selected).map((o) => n ? wo(Vn(o)) : Vn(o));
        e[sn](e.multiple ? r ? new Set(i) : i : i[0]), e._assigning = true, gn(() => {
          e._assigning = false;
        });
      }), e[sn] = vs(s);
    },
    mounted(e, { value: t }) {
      Ti(e, t);
    },
    beforeUpdate(e, t, n) {
      e[sn] = vs(n);
    },
    updated(e, { value: t }) {
      e._assigning || Ti(e, t);
    }
  };
  function Ti(e, t) {
    const n = e.multiple, s = G(t);
    if (!(n && !s && !dn(t))) {
      for (let r = 0, i = e.options.length; r < i; r++) {
        const o = e.options[r], a = Vn(o);
        if (n) if (s) {
          const l = typeof a;
          l === "string" || l === "number" ? o.selected = t.some((c) => String(c) === String(a)) : o.selected = Or(t, a) > -1;
        } else o.selected = t.has(a);
        else if (hn(Vn(o), t)) {
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
      for (let o = 0; o < t.length; o++) {
        const a = Su[t[o]];
        if (a && a(r, t)) return;
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
      const i = Lt(r.key);
      if (t.some((o) => o === i || ku[o] === i)) return e(r);
    }));
  }, Au = Ae({
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
      const o = n(r, false, Cu(r));
      return r instanceof Element && (r.removeAttribute("v-cloak"), r.setAttribute("data-v-app", "")), o;
    }, t;
  });
  function Cu(e) {
    if (e instanceof SVGElement) return "svg";
    if (typeof MathMLElement == "function" && e instanceof MathMLElement) return "mathml";
  }
  function Ru(e) {
    return ve(e) ? document.querySelector(e) : e;
  }
  const Qt = typeof document < "u";
  function ka(e) {
    return typeof e == "object" || "displayName" in e || "props" in e || "__vccOpts" in e;
  }
  function Pu(e) {
    return e.__esModule || e[Symbol.toStringTag] === "Module" || e.default && ka(e.default);
  }
  const pe = Object.assign;
  function zs(e, t) {
    const n = {};
    for (const s in t) {
      const r = t[s];
      n[s] = st(r) ? r.map(e) : e(r);
    }
    return n;
  }
  const Ln = () => {
  }, st = Array.isArray;
  function $i(e, t) {
    const n = {};
    for (const s in e) n[s] = s in t ? t[s] : e[s];
    return n;
  }
  const Ea = /#/g, Tu = /&/g, Iu = /\//g, Ou = /=/g, Nu = /\?/g, Aa = /\+/g, $u = /%5B/g, Mu = /%5D/g, xa = /%5E/g, Du = /%60/g, Ca = /%7B/g, ju = /%7C/g, Ra = /%7D/g, Lu = /%20/g;
  function Gr(e) {
    return e == null ? "" : encodeURI("" + e).replace(ju, "|").replace($u, "[").replace(Mu, "]");
  }
  function Uu(e) {
    return Gr(e).replace(Ca, "{").replace(Ra, "}").replace(xa, "^");
  }
  function pr(e) {
    return Gr(e).replace(Aa, "%2B").replace(Lu, "+").replace(Ea, "%23").replace(Tu, "%26").replace(Du, "`").replace(Ca, "{").replace(Ra, "}").replace(xa, "^");
  }
  function Hu(e) {
    return pr(e).replace(Ou, "%3D");
  }
  function Fu(e) {
    return Gr(e).replace(Ea, "%23").replace(Nu, "%3F");
  }
  function Wu(e) {
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
  const qu = /\/$/, Vu = (e) => e.replace(qu, "");
  function Ys(e, t, n = "/") {
    let s, r = {}, i = "", o = "";
    const a = t.indexOf("#");
    let l = t.indexOf("?");
    return l = a >= 0 && l > a ? -1 : l, l >= 0 && (s = t.slice(0, l), i = t.slice(l, a > 0 ? a : t.length), r = e(i.slice(1))), a >= 0 && (s = s || t.slice(0, a), o = t.slice(a, t.length)), s = Ju(s ?? t, n), {
      fullPath: s + i + o,
      path: s,
      query: r,
      hash: Bn(o)
    };
  }
  function Bu(e, t) {
    const n = t.query ? e(t.query) : "";
    return t.path + (n && "?") + n + (t.hash || "");
  }
  function Ku(e, t, n) {
    const s = t.matched.length - 1, r = n.matched.length - 1;
    return s > -1 && s === r && ln(t.matched[s], n.matched[r]) && Pa(t.params, n.params) && e(t.query) === e(n.query) && t.hash === n.hash;
  }
  function ln(e, t) {
    return (e.aliasOf || e) === (t.aliasOf || t);
  }
  function Pa(e, t) {
    if (Object.keys(e).length !== Object.keys(t).length) return false;
    for (var n in e) if (!Gu(e[n], t[n])) return false;
    return true;
  }
  function Gu(e, t) {
    return st(e) ? Mi(e, t) : st(t) ? Mi(t, e) : (e == null ? void 0 : e.valueOf()) === (t == null ? void 0 : t.valueOf());
  }
  function Mi(e, t) {
    return st(t) ? e.length === t.length && e.every((n, s) => n === t[s]) : e.length === 1 && e[0] === t;
  }
  function Ju(e, t) {
    if (e.startsWith("/")) return e;
    if (!e) return t;
    const n = t.split("/"), s = e.split("/"), r = s[s.length - 1];
    (r === ".." || r === ".") && s.push("");
    let i = n.length - 1, o, a;
    for (o = 0; o < s.length; o++) if (a = s[o], a !== ".") if (a === "..") i > 1 && i--;
    else break;
    return n.slice(0, i).join("/") + "/" + s.slice(o).join("/");
  }
  const Tt = {
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
  let gr = (function(e) {
    return e.pop = "pop", e.push = "push", e;
  })({}), Di = (function(e) {
    return e.back = "back", e.forward = "forward", e.unknown = "", e;
  })({});
  const Qs = "";
  function zu(e) {
    if (!e) if (Qt) {
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
  function ji(e, t) {
    return (history.state ? history.state.position - t : -1) + e;
  }
  const mr = /* @__PURE__ */ new Map();
  function tf(e, t) {
    mr.set(e, t);
  }
  function nf(e) {
    const t = mr.get(e);
    return mr.delete(e), t;
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
  function cn(e, t) {
    return pe(new Error(), {
      type: e,
      [Ia]: true
    }, t);
  }
  function wt(e, t) {
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
      const r = n[s].replace(Aa, " "), i = r.indexOf("="), o = Bn(i < 0 ? r : r.slice(0, i)), a = i < 0 ? null : Bn(r.slice(i + 1));
      if (o in t) {
        let l = t[o];
        st(l) || (l = t[o] = [
          l
        ]), l.push(a);
      } else t[o] = a;
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
      (st(s) ? s.map((r) => r && pr(r)) : [
        s && pr(s)
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
      s !== void 0 && (t[n] = st(s) ? s.map((r) => r == null ? null : "" + r) : s == null ? s : "" + s);
    }
    return t;
  }
  const cf = Symbol(""), Ui = Symbol(""), Ns = Symbol(""), Jr = Symbol(""), vr = Symbol("");
  function bn() {
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
  function $t(e, t, n, s, r, i = (o) => o()) {
    const o = s && (s.enterCallbacks[r] = s.enterCallbacks[r] || []);
    return () => new Promise((a, l) => {
      const c = (m) => {
        m === false ? l(cn(me.NAVIGATION_ABORTED, {
          from: n,
          to: t
        })) : m instanceof Error ? l(m) : sf(m) ? l(cn(me.NAVIGATION_GUARD_REDIRECT, {
          from: t,
          to: m
        })) : (o && s.enterCallbacks[r] === o && typeof m == "function" && o.push(m), a());
      }, u = i(() => e.call(s && s.instances[r], t, n, c));
      let f = Promise.resolve(u);
      e.length < 3 && (f = f.then(c)), f.catch((m) => l(m));
    });
  }
  function Xs(e, t, n, s, r = (i) => i()) {
    const i = [];
    for (const o of e) for (const a in o.components) {
      let l = o.components[a];
      if (!(t !== "beforeRouteEnter" && !o.instances[a])) if (ka(l)) {
        const c = (l.__vccOpts || l)[t];
        c && i.push($t(c, n, s, o, a, r));
      } else {
        let c = l();
        i.push(() => c.then((u) => {
          if (!u) throw new Error(`Couldn't resolve component "${a}" at "${o.path}"`);
          const f = Pu(u) ? u.default : u;
          o.mods[a] = u, o.components[a] = f;
          const m = (f.__vccOpts || f)[t];
          return m && $t(m, n, s, o, a, r)();
        }));
      }
    }
    return i;
  }
  function uf(e, t) {
    const n = [], s = [], r = [], i = Math.max(t.matched.length, e.matched.length);
    for (let o = 0; o < i; o++) {
      const a = t.matched[o];
      a && (e.matched.find((c) => ln(c, a)) ? s.push(a) : n.push(a));
      const l = e.matched[o];
      l && (t.matched.find((c) => ln(c, l)) || r.push(l));
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
        Qs,
        {}
      ]
    ], s = 0;
    e = zu(e);
    function r(a, l = {}) {
      s++, s !== n.length && n.splice(s), n.push([
        a,
        l
      ]);
    }
    function i(a, l, { direction: c, delta: u }) {
      const f = {
        direction: c,
        delta: u,
        type: gr.pop
      };
      for (const m of t) m(a, l, f);
    }
    const o = {
      location: Qs,
      state: {},
      base: e,
      createHref: Qu.bind(null, e),
      replace(a, l) {
        n.splice(s--, 1), r(a, l);
      },
      push(a, l) {
        r(a, l);
      },
      listen(a) {
        return t.push(a), () => {
          const l = t.indexOf(a);
          l > -1 && t.splice(l, 1);
        };
      },
      destroy() {
        t = [], n = [
          [
            Qs,
            {}
          ]
        ], s = 0;
      },
      go(a, l = true) {
        const c = this.location, u = a < 0 ? Di.back : Di.forward;
        s = Math.max(0, Math.min(s + a, n.length - 1)), l && i(this.location, c, {
          direction: u,
          delta: a
        });
      }
    };
    return Object.defineProperty(o, "location", {
      enumerable: true,
      get: () => n[s][0]
    }), Object.defineProperty(o, "state", {
      enumerable: true,
      get: () => n[s][1]
    }), o;
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
    function t(_) {
      throw new Error(`ERR (${n})/"${c}": ${_}`);
    }
    let n = be.Static, s = n;
    const r = [];
    let i;
    function o() {
      i && r.push(i), i = [];
    }
    let a = 0, l, c = "", u = "";
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
    for (; a < e.length; ) {
      if (l = e[a++], l === "\\" && n !== be.ParamRegExp) {
        s = n, n = be.EscapeNext;
        continue;
      }
      switch (n) {
        case be.Static:
          l === "/" ? (c && f(), o()) : l === ":" ? (f(), n = be.Param) : m();
          break;
        case be.EscapeNext:
          m(), n = s;
          break;
        case be.Param:
          l === "(" ? n = be.ParamRegExp : hf.test(l) ? m() : (f(), n = be.Static, l !== "*" && l !== "?" && l !== "+" && a--);
          break;
        case be.ParamRegExp:
          l === ")" ? u[u.length - 1] == "\\" ? u = u.slice(0, -1) + l : n = be.ParamRegExpEnd : u += l;
          break;
        case be.ParamRegExpEnd:
          f(), n = be.Static, l !== "*" && l !== "?" && l !== "+" && a--, u = "";
          break;
        default:
          t("Unknown state");
          break;
      }
    }
    return n === be.ParamRegExp && t(`Unfinished custom RegExp for param "${c}"`), f(), o(), r;
  }
  const Hi = "[^/]+?", gf = {
    sensitive: false,
    strict: false,
    start: true,
    end: true
  };
  var Ne = (function(e) {
    return e[e._multiplier = 10] = "_multiplier", e[e.Root = 90] = "Root", e[e.Segment = 40] = "Segment", e[e.SubSegment = 30] = "SubSegment", e[e.Static = 40] = "Static", e[e.Dynamic = 20] = "Dynamic", e[e.BonusCustomRegExp = 10] = "BonusCustomRegExp", e[e.BonusWildcard = -50] = "BonusWildcard", e[e.BonusRepeatable = -20] = "BonusRepeatable", e[e.BonusOptional = -8] = "BonusOptional", e[e.BonusStrict = 0.7000000000000001] = "BonusStrict", e[e.BonusCaseSensitive = 0.25] = "BonusCaseSensitive", e;
  })(Ne || {});
  const mf = /[.+*?^${}()[\]/\\]/g;
  function vf(e, t) {
    const n = pe({}, gf, t), s = [];
    let r = n.start ? "^" : "";
    const i = [];
    for (const c of e) {
      const u = c.length ? [] : [
        Ne.Root
      ];
      n.strict && !c.length && (r += "/");
      for (let f = 0; f < c.length; f++) {
        const m = c[f];
        let _ = Ne.Segment + (n.sensitive ? Ne.BonusCaseSensitive : 0);
        if (m.type === qt.Static) f || (r += "/"), r += m.value.replace(mf, "\\$&"), _ += Ne.Static;
        else if (m.type === qt.Param) {
          const { value: P, repeatable: R, optional: M, regexp: L } = m;
          i.push({
            name: P,
            repeatable: R,
            optional: M
          });
          const E = L || Hi;
          if (E !== Hi) {
            _ += Ne.BonusCustomRegExp;
            try {
              `${E}`;
            } catch ($) {
              throw new Error(`Invalid custom RegExp for param "${P}" (${E}): ` + $.message);
            }
          }
          let H = R ? `((?:${E})(?:/(?:${E}))*)` : `(${E})`;
          f || (H = M && c.length < 2 ? `(?:/${H})` : "/" + H), M && (H += "?"), r += H, _ += Ne.Dynamic, M && (_ += Ne.BonusOptional), R && (_ += Ne.BonusRepeatable), E === ".*" && (_ += Ne.BonusWildcard);
        }
        u.push(_);
      }
      s.push(u);
    }
    if (n.strict && n.end) {
      const c = s.length - 1;
      s[c][s[c].length - 1] += Ne.BonusStrict;
    }
    n.strict || (r += "/?"), n.end ? r += "$" : n.strict && !r.endsWith("/") && (r += "(?:/|$)");
    const o = new RegExp(r, n.sensitive ? "" : "i");
    function a(c) {
      const u = c.match(o), f = {};
      if (!u) return null;
      for (let m = 1; m < u.length; m++) {
        const _ = u[m] || "", P = i[m - 1];
        f[P.name] = _ && P.repeatable ? _.split("/") : _;
      }
      return f;
    }
    function l(c) {
      let u = "", f = false;
      for (const m of e) {
        (!f || !u.endsWith("/")) && (u += "/"), f = false;
        for (const _ of m) if (_.type === qt.Static) u += _.value;
        else if (_.type === qt.Param) {
          const { value: P, repeatable: R, optional: M } = _, L = P in c ? c[P] : "";
          if (st(L) && !R) throw new Error(`Provided param "${P}" is an array but it is not repeatable (* or + modifiers)`);
          const E = st(L) ? L.join("/") : L;
          if (!E) if (M) m.length < 2 && (u.endsWith("/") ? u = u.slice(0, -1) : f = true);
          else throw new Error(`Missing required param "${P}"`);
          u += E;
        }
      }
      return u || "/";
    }
    return {
      re: o,
      score: s,
      keys: i,
      parse: a,
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
    return e.length < t.length ? e.length === 1 && e[0] === Ne.Static + Ne.Segment ? -1 : 1 : e.length > t.length ? t.length === 1 && t[0] === Ne.Static + Ne.Segment ? 1 : -1 : 0;
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
    function i(f, m, _) {
      const P = !_, R = qi(f);
      R.aliasOf = _ && _.record;
      const M = $i(t, f), L = [
        R
      ];
      if ("alias" in f) {
        const $ = typeof f.alias == "string" ? [
          f.alias
        ] : f.alias;
        for (const X of $) L.push(qi(pe({}, R, {
          components: _ ? _.record.components : R.components,
          path: X,
          aliasOf: _ ? _.record : R
        })));
      }
      let E, H;
      for (const $ of L) {
        const { path: X } = $;
        if (m && X[0] !== "/") {
          const se = m.record.path, re = se[se.length - 1] === "/" ? "" : "/";
          $.path = m.record.path + (X && re + X);
        }
        if (E = bf($, m, M), _ ? _.alias.push(E) : (H = H || E, H !== E && H.alias.push(E), P && f.name && !Vi(E) && o(f.name)), Na(E) && l(E), R.children) {
          const se = R.children;
          for (let re = 0; re < se.length; re++) i(se[re], E, _ && _.children[re]);
        }
        _ = _ || E;
      }
      return H ? () => {
        o(H);
      } : Ln;
    }
    function o(f) {
      if (Ta(f)) {
        const m = s.get(f);
        m && (s.delete(f), n.splice(n.indexOf(m), 1), m.children.forEach(o), m.alias.forEach(o));
      } else {
        const m = n.indexOf(f);
        m > -1 && (n.splice(m, 1), f.record.name && s.delete(f.record.name), f.children.forEach(o), f.alias.forEach(o));
      }
    }
    function a() {
      return n;
    }
    function l(f) {
      const m = Ef(f, n);
      n.splice(m, 0, f), f.record.name && !Vi(f) && s.set(f.record.name, f);
    }
    function c(f, m) {
      let _, P = {}, R, M;
      if ("name" in f && f.name) {
        if (_ = s.get(f.name), !_) throw cn(me.MATCHER_NOT_FOUND, {
          location: f
        });
        M = _.record.name, P = pe(Wi(m.params, _.keys.filter((H) => !H.optional).concat(_.parent ? _.parent.keys.filter((H) => H.optional) : []).map((H) => H.name)), f.params && Wi(f.params, _.keys.map((H) => H.name))), R = _.stringify(P);
      } else if (f.path != null) R = f.path, _ = n.find((H) => H.re.test(R)), _ && (P = _.parse(R), M = _.record.name);
      else {
        if (_ = m.name ? s.get(m.name) : n.find((H) => H.re.test(m.path)), !_) throw cn(me.MATCHER_NOT_FOUND, {
          location: f,
          currentLocation: m
        });
        M = _.record.name, P = pe({}, m.params, f.params), R = _.stringify(P);
      }
      const L = [];
      let E = _;
      for (; E; ) L.unshift(E.record), E = E.parent;
      return {
        name: M,
        path: R,
        params: P,
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
      removeRoute: o,
      clearRoutes: u,
      getRoutes: a,
      getRecordMatcher: r
    };
  }
  function Wi(e, t) {
    const n = {};
    for (const s of t) s in e && (n[s] = e[s]);
    return n;
  }
  function qi(e) {
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
    const t = et(Ns), n = et(Jr), s = Q(() => {
      const l = q(e.to);
      return t.resolve(l);
    }), r = Q(() => {
      const { matched: l } = s.value, { length: c } = l, u = l[c - 1], f = n.matched;
      if (!u || !f.length) return -1;
      const m = f.findIndex(ln.bind(null, u));
      if (m > -1) return m;
      const _ = Ki(l[c - 2]);
      return c > 1 && Ki(u) === _ && f[f.length - 1].path !== _ ? f.findIndex(ln.bind(null, l[c - 2])) : m;
    }), i = Q(() => r.value > -1 && Tf(n.params, s.value.params)), o = Q(() => r.value > -1 && r.value === n.matched.length - 1 && Pa(n.params, s.value.params));
    function a(l = {}) {
      if (Pf(l)) {
        const c = t[q(e.replace) ? "replace" : "push"](q(e.to)).catch(Ln);
        return e.viewTransition && typeof document < "u" && "startViewTransition" in document && document.startViewTransition(() => c), c;
      }
      return Promise.resolve();
    }
    return {
      route: s,
      href: Q(() => s.value.href),
      isActive: i,
      isExactActive: o,
      navigate: a
    };
  }
  function xf(e) {
    return e.length === 1 ? e[0] : e;
  }
  const Cf = Ut({
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
      const n = pn(Bi(e)), { options: s } = et(Ns), r = Q(() => ({
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
      } else if (!st(r) || r.length !== s.length || s.some((i, o) => i.valueOf() !== r[o].valueOf())) return false;
    }
    return true;
  }
  function Ki(e) {
    return e ? e.aliasOf ? e.aliasOf.path : e.path : "";
  }
  const Gi = (e, t, n) => e ?? t ?? n, If = Ut({
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
      const s = et(vr), r = Q(() => e.route || s.value), i = et(Ui, 0), o = Q(() => {
        let c = q(i);
        const { matched: u } = r.value;
        let f;
        for (; (f = u[c]) && !f.components; ) c++;
        return c;
      }), a = Q(() => r.value.matched[o.value]);
      rs(Ui, Q(() => o.value + 1)), rs(cf, a), rs(vr, r);
      const l = ye();
      return Kt(() => [
        l.value,
        a.value,
        e.name
      ], ([c, u, f], [m, _, P]) => {
        u && (u.instances[f] = c, _ && _ !== u && c && c === m && (u.leaveGuards.size || (u.leaveGuards = _.leaveGuards), u.updateGuards.size || (u.updateGuards = _.updateGuards))), c && u && (!_ || !ln(u, _) || !m) && (u.enterCallbacks[f] || []).forEach((R) => R(c));
      }, {
        flush: "post"
      }), () => {
        const c = r.value, u = e.name, f = a.value, m = f && f.components[u];
        if (!m) return Ji(n.default, {
          Component: m,
          route: c
        });
        const _ = f.props[u], P = _ ? _ === true ? c.params : typeof _ == "function" ? _(c) : _ : null, M = ba(m, pe({}, P, t, {
          onVnodeUnmounted: (L) => {
            L.component.isUnmounted && (f.instances[u] = null);
          },
          ref: l
        }));
        return Ji(n.default, {
          Component: M,
          route: c
        }) || M;
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
    const t = wf(e.routes, e), n = e.parseQuery || af, s = e.stringifyQuery || Li, r = e.history, i = bn(), o = bn(), a = bn(), l = Nl(Tt);
    let c = Tt;
    Qt && e.scrollBehavior && "scrollRestoration" in history && (history.scrollRestoration = "manual");
    const u = zs.bind(null, (S) => "" + S), f = zs.bind(null, Wu), m = zs.bind(null, Bn);
    function _(S, U) {
      let N, W;
      return Ta(S) ? (N = t.getRecordMatcher(S), W = U) : W = S, t.addRoute(W, N);
    }
    function P(S) {
      const U = t.getRecordMatcher(S);
      U && t.removeRoute(U);
    }
    function R() {
      return t.getRoutes().map((S) => S.record);
    }
    function M(S) {
      return !!t.getRecordMatcher(S);
    }
    function L(S, U) {
      if (U = pe({}, U || l.value), typeof S == "string") {
        const y = Ys(n, S, U.path), b = t.resolve({
          path: y.path
        }, U), g = r.createHref(y.fullPath);
        return pe(y, b, {
          params: m(b.params),
          hash: Bn(y.hash),
          redirectedFrom: void 0,
          href: g
        });
      }
      let N;
      if (S.path != null) N = pe({}, S, {
        path: Ys(n, S.path, U.path).path
      });
      else {
        const y = pe({}, S.params);
        for (const b in y) y[b] == null && delete y[b];
        N = pe({}, S, {
          params: f(y)
        }), U.params = f(U.params);
      }
      const W = t.resolve(N, U), Z = S.hash || "";
      W.params = u(m(W.params));
      const d = Bu(s, pe({}, S, {
        hash: Uu(Z),
        path: W.path
      })), h = r.createHref(d);
      return pe({
        fullPath: d,
        hash: Z,
        query: s === Li ? lf(S.query) : S.query || {}
      }, W, {
        redirectedFrom: void 0,
        href: h
      });
    }
    function E(S) {
      return typeof S == "string" ? Ys(n, S, l.value.path) : pe({}, S);
    }
    function H(S, U) {
      if (c !== S) return cn(me.NAVIGATION_CANCELLED, {
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
        const { redirect: W } = N;
        let Z = typeof W == "function" ? W(S, U) : W;
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
      const N = c = L(S), W = l.value, Z = S.state, d = S.force, h = S.replace === true, y = se(N, W);
      if (y) return re(pe(E(y), {
        state: typeof y == "object" ? pe({}, Z, y.state) : Z,
        force: d,
        replace: h
      }), U || N);
      const b = N;
      b.redirectedFrom = U;
      let g;
      return !d && Ku(s, W, N) && (g = cn(me.NAVIGATION_DUPLICATED, {
        to: b,
        from: W
      }), D(W, W, true, false)), (g ? Promise.resolve(g) : je(b, W)).catch((p) => wt(p) ? wt(p, me.NAVIGATION_GUARD_REDIRECT) ? p : O(p) : ee(p, b, W)).then((p) => {
        if (p) {
          if (wt(p, me.NAVIGATION_GUARD_REDIRECT)) return re(pe({
            replace: h
          }, E(p.to), {
            state: typeof p.to == "object" ? pe({}, Z, p.to.state) : Z,
            force: d
          }), U || b);
        } else p = Qe(b, W, true, h, Z);
        return Le(b, W, p), p;
      });
    }
    function Me(S, U) {
      const N = H(S, U);
      return N ? Promise.reject(N) : Promise.resolve();
    }
    function De(S) {
      const U = Ie.values().next().value;
      return U && typeof U.runWithContext == "function" ? U.runWithContext(S) : S();
    }
    function je(S, U) {
      let N;
      const [W, Z, d] = uf(S, U);
      N = Xs(W.reverse(), "beforeRouteLeave", S, U);
      for (const y of W) y.leaveGuards.forEach((b) => {
        N.push($t(b, S, U));
      });
      const h = Me.bind(null, S, U);
      return N.push(h), He(N).then(() => {
        N = [];
        for (const y of i.list()) N.push($t(y, S, U));
        return N.push(h), He(N);
      }).then(() => {
        N = Xs(Z, "beforeRouteUpdate", S, U);
        for (const y of Z) y.updateGuards.forEach((b) => {
          N.push($t(b, S, U));
        });
        return N.push(h), He(N);
      }).then(() => {
        N = [];
        for (const y of d) if (y.beforeEnter) if (st(y.beforeEnter)) for (const b of y.beforeEnter) N.push($t(b, S, U));
        else N.push($t(y.beforeEnter, S, U));
        return N.push(h), He(N);
      }).then(() => (S.matched.forEach((y) => y.enterCallbacks = {}), N = Xs(d, "beforeRouteEnter", S, U, De), N.push(h), He(N))).then(() => {
        N = [];
        for (const y of o.list()) N.push($t(y, S, U));
        return N.push(h), He(N);
      }).catch((y) => wt(y, me.NAVIGATION_CANCELLED) ? y : Promise.reject(y));
    }
    function Le(S, U, N) {
      a.list().forEach((W) => De(() => W(S, U, N)));
    }
    function Qe(S, U, N, W, Z) {
      const d = H(S, U);
      if (d) return d;
      const h = U === Tt, y = Qt ? history.state : {};
      N && (W || h ? r.replace(S.fullPath, pe({
        scroll: h && y && y.scroll
      }, Z)) : r.push(S.fullPath, Z)), l.value = S, D(S, U, N, h), O();
    }
    let Ue;
    function _t() {
      Ue || (Ue = r.listen((S, U, N) => {
        if (!it.listening) return;
        const W = L(S), Z = se(W, it.currentRoute.value);
        if (Z) {
          re(pe(Z, {
            replace: true,
            force: true
          }), W).catch(Ln);
          return;
        }
        c = W;
        const d = l.value;
        Qt && tf(ji(d.fullPath, N.delta), Zu()), je(W, d).catch((h) => wt(h, me.NAVIGATION_ABORTED | me.NAVIGATION_CANCELLED) ? h : wt(h, me.NAVIGATION_GUARD_REDIRECT) ? (re(pe(E(h.to), {
          force: true
        }), W).then((y) => {
          wt(y, me.NAVIGATION_ABORTED | me.NAVIGATION_DUPLICATED) && !N.delta && N.type === gr.pop && r.go(-1, false);
        }).catch(Ln), Promise.reject()) : (N.delta && r.go(-N.delta, false), ee(h, W, d))).then((h) => {
          h = h || Qe(W, d, false), h && (N.delta && !wt(h, me.NAVIGATION_CANCELLED) ? r.go(-N.delta, false) : N.type === gr.pop && wt(h, me.NAVIGATION_ABORTED | me.NAVIGATION_DUPLICATED) && r.go(-1, false)), Le(W, d, h);
        }).catch(Ln);
      }));
    }
    let rt = bn(), ge = bn(), te;
    function ee(S, U, N) {
      O(S);
      const W = ge.list();
      return W.length ? W.forEach((Z) => Z(S, U, N)) : console.error(S), Promise.reject(S);
    }
    function Te() {
      return te && l.value !== Tt ? Promise.resolve() : new Promise((S, U) => {
        rt.add([
          S,
          U
        ]);
      });
    }
    function O(S) {
      return te || (te = !S, _t(), rt.list().forEach(([U, N]) => S ? N(S) : U()), rt.reset()), S;
    }
    function D(S, U, N, W) {
      const { scrollBehavior: Z } = e;
      if (!Qt || !Z) return Promise.resolve();
      const d = !N && nf(ji(S.fullPath, 0)) || (W || !N) && history.state && history.state.scroll || null;
      return gn().then(() => Z(S, U, d)).then((h) => h && ef(h)).catch((h) => ee(h, S, U));
    }
    const F = (S) => r.go(S);
    let ie;
    const Ie = /* @__PURE__ */ new Set(), it = {
      currentRoute: l,
      listening: true,
      addRoute: _,
      removeRoute: P,
      clearRoutes: t.clearRoutes,
      hasRoute: M,
      getRoutes: R,
      resolve: L,
      options: e,
      push: $,
      replace: X,
      go: F,
      back: () => F(-1),
      forward: () => F(1),
      beforeEach: i.add,
      beforeResolve: o.add,
      afterEach: a.add,
      onError: ge.add,
      isReady: Te,
      install(S) {
        S.component("RouterLink", Rf), S.component("RouterView", Of), S.config.globalProperties.$router = it, Object.defineProperty(S.config.globalProperties, "$route", {
          enumerable: true,
          get: () => q(l)
        }), Qt && !ie && l.value === Tt && (ie = true, $(r.location).catch((W) => {
        }));
        const U = {};
        for (const W in Tt) Object.defineProperty(U, W, {
          get: () => l.value[W],
          enumerable: true
        });
        S.provide(Ns, it), S.provide(Jr, Uo(U)), S.provide(vr, l);
        const N = S.unmount;
        Ie.add(S), S.unmount = function() {
          Ie.delete(S), Ie.size < 1 && (c = Tt, Ue && Ue(), Ue = null, l.value = Tt, ie = false, te = false), N();
        };
      }
    };
    function He(S) {
      return S.reduce((U, N) => U.then(() => De(N)), Promise.resolve());
    }
    return it;
  }
  function $f() {
    return et(Ns);
  }
  function Mf(e) {
    return et(Jr);
  }
  const zi = {
    confirm: "Confirm",
    select_cards: "SelectCards",
    select_players: "SelectTargets",
    respond_card: "Custom",
    pick_tag: "Custom"
  };
  function Df(e) {
    if (e === void 0) return [
      null,
      null
    ];
    if (!Array.isArray(e) || e.length !== 2 || !e.every((t) => t === null || typeof t == "object" && !Array.isArray(t) && Number.isInteger(t.def) && t.def >= 0 && t.def <= 4294967295 && typeof t.name == "string" && typeof t.faction == "string" && typeof t.revealed == "boolean")) throw new Error("native: generals metadata invalid");
    return e;
  }
  function jf(e) {
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
    dying_save: "\u8BF7\u9009\u62E9\u6551\u63F4\u724C",
    discard_to_limit: "\u8BF7\u5F03\u724C\u81F3\u624B\u724C\u4E0A\u9650",
    tuxi_target: "\u7A81\u88AD\uFF1A\u9009\u62E9\u83B7\u5F97\u624B\u724C\u7684\u89D2\u8272",
    guanxing_arrange: "\u89C2\u661F\uFF1A\u6392\u5217\u724C\u5806\u9876\u7684\u5361\u724C",
    xunxun_arrange: "\u6042\u6042\uFF1A\u9009\u62E9\u83B7\u5F97\u7684\u5361\u724C",
    jijie_give_target: "\u673A\u6377\uFF1A\u9009\u62E9\u83B7\u5F97\u6B64\u724C\u7684\u89D2\u8272",
    gaiming_pick: "\u6539\u547D\uFF1A\u9009\u62E9\u5224\u5B9A\u724C",
    haotianta_pick: "\u660A\u5929\u5854\uFF1A\u9009\u62E9\u5224\u5B9A\u724C",
    liaoyuan_discard: "\u71CE\u539F\uFF1A\u9009\u62E9\u5F03\u7F6E\u7684\u5361\u724C",
    fanjian_target: "\u53CD\u95F4\uFF1A\u9009\u62E9\u76EE\u6807\u89D2\u8272",
    fanjian_suit: "\u53CD\u95F4\uFF1A\u9009\u62E9\u82B1\u8272",
    fanjian_pick: "\u53CD\u95F4\uFF1A\u9009\u62E9\u83B7\u5F97\u7684\u624B\u724C"
  };
  function ys(e) {
    const t = e.native_kind ?? e.kind ?? "Custom";
    return Object.hasOwn(Yi, t) ? Yi[t] : "\u9009\u62E9\u9009\u9879";
  }
  function yr(e) {
    return e.prompt && Object.hasOwn(Qi, e.prompt) ? Qi[e.prompt] : ys(e);
  }
  function $a(e, t) {
    for (const n of e.seats ?? []) Df(n.generals);
    t && jf(t);
  }
  async function zn(e, t = {}, n) {
    return window.__TAURI_INTERNALS__.invoke(e, t, n);
  }
  function Lf(e, t = "asset") {
    return window.__TAURI_INTERNALS__.convertFileSrc(e, t);
  }
  function jt() {
    return typeof window < "u" && "__TAURI_INTERNALS__" in window;
  }
  function Xi(e) {
    return jt() ? zn("set_match_frozen", {
      frozen: e
    }) : Promise.resolve();
  }
  function Uf() {
    return jt() ? zn("content_bundle") : Promise.resolve(null);
  }
  function Hf(e, t) {
    return jt() ? zn("content_catalog_page", {
      offset: e,
      limit: t
    }) : Promise.resolve(null);
  }
  async function Ff(e) {
    if (!jt()) return null;
    const t = await zn("resolve_content_asset", {
      namespace: e.namespace,
      packageId: e.package_id,
      assetId: e.asset_id
    });
    return Lf(t);
  }
  const rn = (e) => new Promise((t) => setTimeout(t, e)), $s = "\u81EA\u52A8\u4E00\u6B65", Wf = "\u8FDE\u7EED\u81EA\u52A8", Ms = /终局/;
  let Ma = [], _r = 0;
  function Yn(e) {
    return Array.from(document.querySelectorAll("button")).find((t) => (t.innerText || "").trim() === e) ?? null;
  }
  function Zi() {
    return new Promise((e) => {
      const t = () => {
        const r = Yn($s);
        return r ? Ms.test(document.body.innerText) ? "finished" : r.disabled ? null : "ready" : "gone";
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
  function br() {
    return window.__nonamePerf ?? null;
  }
  function qf() {
    const e = br(), t = document.body.innerText.match(/终局[^\n]*/) ?? document.body.innerText.match(/rev \d+[^\n]*/);
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
      const s = Yn($s);
      if (!s || Ms.test(document.body.innerText)) return "finished";
      if (s.disabled) {
        if (await Zi() === "finished") return "finished";
        continue;
      }
      if (s.click(), await Zi() === "finished") return "finished";
      await rn(t);
    }
    return "max_steps";
  }
  async function Gf() {
    var _a2;
    const e = Xe("loop") === "1", t = Number(Xe("duration") ?? 0), n = Date.now() + (t > 0 ? t : 6e5), s = () => Ms.test(document.body.innerText), r = [], i = performance.memory, o = performance.measureUserAgentSpecificMemory, a = async () => {
      if (i) {
        r.push({
          t_ms: Date.now() - _r,
          bytes: i.usedJSHeapSize
        });
        return;
      }
      if (!(!o || !crossOriginIsolated)) try {
        const l = await o();
        r.push({
          t_ms: Date.now() - _r,
          bytes: l.bytes
        });
      } catch {
      }
    };
    for (; ; ) {
      const l = Yn(Wf);
      l && l.click();
      let c = 0;
      for (; Date.now() < n && !s(); ) await rn(1e3), Date.now() - c > 3e4 && (c = Date.now(), await a());
      if (!e || Date.now() >= n) return Ma = r, s() ? "finished" : "auto_timeout";
      (_a2 = Array.from(document.querySelectorAll("button")).find((f) => (f.innerText || "").includes("\u91CD\u65B0\u5F00\u5C40"))) == null ? void 0 : _a2.click(), await rn(1500);
    }
  }
  function Jf(e, t) {
    return new Promise((n) => {
      let s = 0;
      const r = (a) => {
        window.removeEventListener("click", i, true), n(a);
      }, i = (a) => {
        const l = a.target, c = Yn($s);
        c && (l === c || c.contains(l)) && (s += 1, s >= e && setTimeout(() => r("manual_steps"), 800));
      };
      window.addEventListener("click", i, true);
      const o = setInterval(() => {
        Ms.test(document.body.innerText) && (clearInterval(o), r("finished"));
      }, 500);
      setTimeout(() => {
        clearInterval(o), r(s > 0 ? "manual_timeout" : "manual_no_input");
      }, t);
    });
  }
  function Xe(e) {
    const t = location.hash.split("?")[1], n = t ? new URLSearchParams(t) : null;
    return new URLSearchParams(location.search).get(e) ?? (n == null ? void 0 : n.get(e)) ?? null;
  }
  function zf() {
    if (typeof window > "u") return;
    const e = window;
    if (e.__nonameAutoDriveStarted) return;
    const t = Xe("autodrive");
    if (!t) return;
    e.__nonameAutoDriveStarted = true;
    const n = Xe("report"), s = Number(Xe("steps") ?? 1500), r = Number(Xe("gap") ?? 50), i = Number(Xe("timeout") ?? 12e4);
    (async () => {
      var _a2;
      for (let f = 0; f < 150; f++) {
        const m = Yn($s);
        if (m && br() && !m.disabled) break;
        await rn(200);
      }
      (_a2 = br()) == null ? void 0 : _a2.reset(), await rn(300);
      const a = Date.now();
      _r = a;
      let l = "unknown";
      t === "click" ? l = await Kf(s, r) : t === "auto" ? l = await Gf() : t === "manual" && (l = await Jf(s, i)), await rn(600);
      const c = qf();
      c.memory = await Vf(), c.memory_series = Ma, n && await Bf(n, {
        ...c,
        end_state: l,
        wall_ms: Date.now() - a
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
    const e = typeof location > "u" ? null : Xe("engineTransport");
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
  function Zs(e, t) {
    if (e.length === 0) return null;
    const n = [
      ...e
    ].sort((r, i) => r - i), s = Math.ceil(t / 100 * n.length);
    return n[Math.min(n.length, Math.max(1, s)) - 1];
  }
  function _s(e) {
    const t = e.filter((s) => Number.isFinite(s)), n = t.reduce((s, r) => s + r, 0);
    return {
      count: t.length,
      p50: Zs(t, 50),
      p95: Zs(t, 95),
      p99: Zs(t, 99),
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
    const t = _s(e), n = (r) => e.length ? e.filter((i) => i > r).length / e.length : null, s = t.p50;
    return {
      ...t,
      refresh_estimate_ms: s,
      over_8_33_ratio: n(1e3 / 120),
      over_16_67_ratio: n(1e3 / 60),
      over_1_5x_refresh_ratio: s === null ? null : n(s * 1.5)
    };
  }
  const St = (e, t) => e === null || t === null ? null : e - t;
  function to(e) {
    const t = e.worker, n = e.input_ts ?? e.sent;
    return {
      id: e.id,
      type: e.type,
      input_kind: e.input_kind,
      input_to_send_ms: St(e.sent, e.input_ts),
      roundtrip_ms: St(e.received, e.sent),
      worker_total_ms: (t == null ? void 0 : t.total_ms) ?? null,
      dispatch_ms: (t == null ? void 0 : t.dispatch_ms) ?? null,
      view_json_ms: (t == null ? void 0 : t.view_json_ms) ?? null,
      view_parse_ms: (t == null ? void 0 : t.view_parse_ms) ?? null,
      adapt_ms: (t == null ? void 0 : t.adapt_ms) ?? null,
      post_ms: (t == null ? void 0 : t.post_ms) ?? null,
      transfer_ms: St(e.received_epoch_ms, (t == null ? void 0 : t.post_epoch_ms) ?? null),
      receive_to_apply_ms: St(e.apply_start, e.received),
      apply_ms: St(e.apply_end, e.apply_start),
      vue_patch_ms: St(e.patched, e.apply_end),
      patch_to_frame_ms: St(e.frame, e.patched),
      frame_to_paint_ms: St(e.after_paint, e.frame),
      input_to_paint_ms: St(e.after_paint, n),
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
      n[s] = _s(r);
    }
    return n;
  }
  const rd = "noname-next-perf", id = 3e5, od = 2e4, wr = 5e4, ad = 100;
  let no = false, ze = false;
  const wn = [], Sn = [], kn = [], gt = /* @__PURE__ */ new Map(), un = /* @__PURE__ */ new Map();
  let xn = null, fn = null;
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
    xn = {
      ts: e.timeStamp,
      kind: e.type,
      used: false
    };
  }
  function ud() {
    if (no || (no = true, ze = ld(), !ze || typeof window > "u")) return ze;
    const e = (t) => {
      wn.length < id && wn.push(t), requestAnimationFrame(e);
    };
    return requestAnimationFrame(e), window.addEventListener("click", ro, {
      capture: true
    }), window.addEventListener("keydown", ro, {
      capture: true
    }), so("longtask", {}, (t) => {
      Sn.length < wr && Sn.push({
        start: t.startTime,
        duration: t.duration
      });
    }), so("event", {
      durationThreshold: 16
    }, (t) => {
      const n = t;
      kn.length < wr && kn.push({
        name: n.name,
        start: n.startTime,
        duration: n.duration,
        input_delay: n.processingStart - n.startTime,
        processing: n.processingEnd - n.processingStart
      });
    }), window.__nonamePerf = {
      version: 1,
      enabled: true,
      frames: () => wn.slice(),
      longTasks: () => Sn.slice(),
      eventTimings: () => kn.slice(),
      interactions: () => [
        ...gt.values()
      ].map(to),
      summary: () => {
        const t = [
          ...gt.values()
        ].map(to);
        return {
          frames: nd(td(wn)),
          interactions: sd(t),
          long_tasks: _s(Sn.map((n) => n.duration)),
          event_timings: _s(kn.map((n) => n.duration))
        };
      },
      reset: () => {
        wn.length = 0, Sn.length = 0, kn.length = 0, gt.clear(), un.clear(), fn = null;
      }
    }, ze && zf(), ze;
  }
  function fd(e, t) {
    if (!ze || gt.size >= od) return;
    const n = performance.now(), s = xn && !xn.used && n - xn.ts <= ad ? xn : null;
    s && (s.used = true), gt.set(e, {
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
      worker: un.get(e) ?? null
    }), un.delete(e);
  }
  function io(e) {
    ze && (gt.delete(e), un.delete(e));
  }
  function oo(e) {
    if (!ze) return;
    const t = gt.get(e);
    t && (t.received = performance.now(), t.received_epoch_ms = cd(), fn = e);
  }
  function dd(e, t) {
    if (!ze) return;
    const n = gt.get(e);
    n ? n.worker = t : un.size < wr && un.set(e, t);
  }
  function hd() {
    if (fn === null) return null;
    const e = gt.get(fn);
    return e && e.apply_start === null ? e : null;
  }
  function pd() {
    if (!ze) return;
    const e = hd();
    e && (e.apply_start = performance.now());
  }
  function gd() {
    if (!ze || fn === null) return;
    const e = gt.get(fn);
    !e || e.apply_start === null || e.apply_end !== null || (e.apply_end = performance.now(), gn(() => {
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
    return new Worker("/assets/engine.native.worker-7GMLoZcq.js", {
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
  class Ds {
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
      const n = !this.native && jt() ? await Uf() : void 0, s = this.native && jt() && Xf() === "native";
      this.transportKind = s ? "desktop-native" : this.native ? "wasm-worker" : "legacy-worker", globalThis.__engineTransport = this.transportKind;
      const r = s ? new ed() : this.native ? new md() : new vd();
      this.worker = r, r.onmessage = (i) => {
        var _a2, _b;
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
          const c = i.data;
          dd(c.id, c.timings);
          return;
        }
        const a = i.data, l = this.pending.get(a.id);
        l && (oo(a.id), this.pending.delete(a.id), (_b = (_a2 = globalThis.__latencySabDebug) == null ? void 0 : _a2.settle) == null ? void 0 : _b.call(_a2), a.ok ? l.resolve(a.data) : l.reject(new Error(a.error)));
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
      if (Xe("latency") !== "sab" || !crossOriginIsolated || typeof SharedArrayBuffer > "u") return;
      const t = document.createElement("style");
      t.textContent = ".sab-keepalive{animation:sab-pulse 8ms infinite}@keyframes sab-pulse{50%{margin-left:.5px}}", document.head.appendChild(t);
      const n = document.createElement("div");
      n.className = "sab-keepalive", n.style.display = "none", document.body.appendChild(n);
      const s = 256 * 1024, r = new SharedArrayBuffer(s), i = new Int32Array(r, 0, 2);
      Atomics.store(i, 0, 0), Atomics.store(i, 1, 0);
      const o = new TextDecoder(), a = () => {
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
            const _ = JSON.parse(o.decode(m)), P = this.pending.get(_.id);
            if (P) {
              oo(_.id), this.pending.delete(_.id), a();
              const R = globalThis.__latencySabDebug;
              R && (R.resolvedEarly += 1), _.ok ? P.resolve(_.data) : P.reject(new Error(_.error ?? "worker error"));
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
            head: o.decode(f)
          };
        },
        wake: () => {
          n.style.display = "block";
        },
        settle: a,
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
      const o = Xe("packsUrl"), a = Xe("hexUrl"), l = o ? await ao(o) : r;
      return a ? (await this.request({
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
        hex: await ao(a),
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
  const Sr = 200, B = pn({
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
  let Mt = null;
  const Cn = /* @__PURE__ */ new Map(), Rn = /* @__PURE__ */ new Map(), kr = /* @__PURE__ */ new Map();
  function yd(e) {
    if (jt()) {
      xr({
        kind: "desktop",
        key: "desktop"
      });
      return;
    }
    const t = `${window.location.protocol === "https:" ? "https:" : "http:"}//${window.location.hostname || "127.0.0.1"}:7700`;
    xr({
      kind: "http",
      key: `http:${t}`,
      baseUrl: t
    });
  }
  function _d(e) {
    const t = new URL(e.trim(), window.location.href);
    t.protocol = t.protocol === "wss:" || t.protocol === "https:" ? "https:" : "http:";
    const n = t.origin;
    xr({
      kind: "http",
      key: `http:${n}`,
      baseUrl: n
    });
  }
  async function bd(e) {
    const t = B.characters.get(e);
    if (t) return B.imageUrls.has(e) || await Ar(e), t;
    if (!Mt) return;
    const n = Rn.get(e);
    if (n) return n;
    const s = Mt.key, r = (async () => {
      let i = 0;
      for (; (Mt == null ? void 0 : Mt.key) === s; ) {
        const o = await Da(i);
        if (!o) break;
        const a = B.characters.get(e);
        if (a) return await Ar(e), a;
        const l = o.offset + o.characters.length;
        if (l >= o.total || l <= i) break;
        i = l;
      }
    })();
    Rn.set(e, r);
    try {
      return await r;
    } finally {
      Rn.get(e) === r && Rn.delete(e);
    }
  }
  function wd(e) {
    var _a2;
    const t = (_a2 = B.characters.get(e)) == null ? void 0 : _a2.definition.name_key;
    return t ? ja(t) : void 0;
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
    return ja(e.name_key);
  }
  function Er(e) {
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
    Ar(e);
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
  async function Da(e) {
    const t = B.pages.get(e);
    if (t) return t;
    const n = Cn.get(e);
    if (n) return n;
    const s = Mt;
    if (!s) return null;
    const r = (async () => {
      let i;
      if (s.kind === "desktop") i = await Hf(e, Sr);
      else {
        const o = new URL("/content-catalog", s.baseUrl);
        o.searchParams.set("offset", String(e)), o.searchParams.set("limit", String(Sr));
        const a = await fetch(o);
        if (!a.ok) throw new Error(`\u5185\u5BB9\u76EE\u5F55\u52A0\u8F7D\u5931\u8D25 (${a.status})`);
        i = await a.json();
      }
      if (Mt !== s || i == null) return null;
      if (Pd(i), i.offset !== Math.min(e, i.total)) throw new Error("\u5185\u5BB9\u76EE\u5F55\u5206\u9875\u504F\u79FB\u4E0D\u4E00\u81F4");
      return e !== 0 && B.contentLockHash && B.contentLockHash !== i.content_lock_hash ? (zr(), Da(0)) : (Rd(i), i);
    })();
    Cn.set(e, r);
    try {
      return await r;
    } finally {
      Cn.get(e) === r && Cn.delete(e);
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
  async function Ar(e) {
    const t = B.characters.get(e);
    if (!t) return;
    const n = Er(e), s = n == null ? void 0 : B.skins.get(n), r = (s == null ? void 0 : s.presentation.kind) === "static" ? s.presentation.asset : null, i = js(e);
    B.portraitUrls.set(e, i), B.imageUrls.set(e, i);
    const o = `${B.contentLockHash}:${n ?? "base"}`, a = t.portrait_asset ? await er(t.portrait_asset).catch(() => null) : null, l = r ? await er(r).catch(() => null) : null, c = t.skin_ids.map((m) => B.skins.get(m)).find((m) => (m == null ? void 0 : m.presentation.kind) === "substitute" && m.presentation.rule === "dead"), u = (c == null ? void 0 : c.presentation.kind) === "substitute" ? c.presentation.asset : null, f = u ? await er(u).catch(() => null) : null;
    if (o === `${B.contentLockHash}:${Er(e) ?? "base"}`) {
      const m = a ?? i;
      B.portraitUrls.set(e, m), B.imageUrls.set(e, l ?? m), f && B.substituteUrls.set(`${e}:dead`, f);
    }
  }
  function er(e) {
    if (!Td(e) || !Mt) return Promise.resolve(null);
    const t = Mt, n = [
      t.key,
      e.namespace,
      e.package_id,
      e.asset_id
    ].join("\0"), s = kr.get(n);
    if (s) return s;
    const r = t.kind === "desktop" ? Ff(e) : Promise.resolve(`${t.baseUrl}/package-assets/${[
      e.namespace,
      e.package_id
    ].map(encodeURIComponent).join("/")}/${e.asset_id.split("/").map(encodeURIComponent).join("/")}?v=${encodeURIComponent(B.contentLockHash)}`);
    return kr.set(n, r), r;
  }
  function ja(e) {
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
  function xr(e) {
    Mt = e, zr();
  }
  function zr() {
    B.contentLockHash = "", B.total = 0, B.pages.clear(), B.characters.clear(), B.skins.clear(), B.translations = {}, B.selectedSkins.clear(), B.portraitUrls.clear(), B.imageUrls.clear(), B.substituteUrls.clear(), Cn.clear(), Rn.clear(), kr.clear();
  }
  function Pd(e) {
    const t = e;
    if (!t || typeof t.content_lock_hash != "string" || !It(t.offset) || !It(t.limit) || !It(t.total) || t.limit < 1 || t.limit > Sr || t.offset > t.total || !Array.isArray(t.characters) || t.characters.length > t.limit || !Array.isArray(t.skins) || !Array.isArray(t.aliases) || !t.translations || typeof t.translations != "object" || !t.characters.every((n) => {
      var _a2, _b;
      return It((_a2 = n == null ? void 0 : n.definition) == null ? void 0 : _a2.id) && n.definition.id > 0 && typeof ((_b = n == null ? void 0 : n.definition) == null ? void 0 : _b.name_key) == "string" && Array.isArray(n == null ? void 0 : n.skin_ids) && n.skin_ids.every(It);
    }) || !t.skins.every((n) => {
      var _a2;
      return It(n == null ? void 0 : n.id) && n.id > 0 && typeof (n == null ? void 0 : n.name_key) == "string" && [
        "static",
        "spine",
        "substitute"
      ].includes((_a2 = n == null ? void 0 : n.presentation) == null ? void 0 : _a2.kind);
    }) || !t.aliases.every((n) => It(n == null ? void 0 : n.character_id) && It(n == null ? void 0 : n.source_character_id) && lo(n == null ? void 0 : n.voice_mappings)) || !Object.values(t.translations).every(lo)) throw new Error("\u5185\u5BB9\u76EE\u5F55\u54CD\u5E94\u65E0\u6548");
  }
  function It(e) {
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
  function bs(e) {
    return typeof e == "string" && e.trim().length > 0 && e !== "." && e !== ".." && !e.includes("/") && !e.includes("\\");
  }
  function Cr(e) {
    return Array.isArray(e) && e.length === 32 && e.every((t) => Number.isInteger(t) && t >= 0 && t <= 255);
  }
  function Ha(e) {
    const t = e, n = t == null ? void 0 : t.manifest;
    if (!n || ![
      n.namespace,
      n.package_id,
      n.version
    ].every(bs) || !Cr(n.logic_hash)) throw new Error("\u6269\u5C55\u5305\u8EAB\u4EFD\u6216\u903B\u8F91\u6307\u7EB9\u65E0\u6548");
    if (n.dependencies !== void 0) {
      if (!Array.isArray(n.dependencies)) throw new Error("\u6269\u5C55\u5305\u4F9D\u8D56\u683C\u5F0F\u65E0\u6548");
      const r = /* @__PURE__ */ new Set();
      for (const i of n.dependencies) {
        if (!i || typeof i != "object" || Array.isArray(i) || ![
          i.namespace,
          i.package_id,
          i.version
        ].every(bs) || i.logic_hash !== void 0 && !Cr(i.logic_hash)) throw new Error("\u6269\u5C55\u5305\u4F9D\u8D56\u683C\u5F0F\u65E0\u6548");
        const o = `${i.namespace}/${i.package_id}`;
        if (r.has(o) || o === `${n.namespace}/${n.package_id}`) throw new Error(`\u6269\u5C55\u5305\u4F9D\u8D56\u91CD\u590D\u6216\u6307\u5411\u81EA\u8EAB: ${o}`);
        r.add(o);
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
    for (const o of t) {
      const a = o;
      if (!a || typeof a != "object" || Array.isArray(a)) throw new Error("\u6269\u5C55\u5305 VM \u6A21\u5757\u683C\u5F0F\u65E0\u6548");
      if (!bs(a.name) || n.has(a.name)) throw new Error("\u6269\u5C55\u5305 VM \u6A21\u5757\u540D\u7F3A\u5931\u6216\u91CD\u590D");
      const l = a.code;
      if (!Array.isArray(l) || !l.length || !l.every((c) => Number.isInteger(c) && c >= 0 && c <= 255)) throw new Error(`\u6269\u5C55\u5305 VM \u6A21\u5757\u5B57\u8282\u7801\u65E0\u6548: ${a.name}`);
      if (![
        a.local_count,
        a.max_stack,
        a.entry_pc,
        a.caps_mask
      ].every(es) || a.entry_pc >= l.length) throw new Error(`\u6269\u5C55\u5305 VM \u6A21\u5757\u5165\u53E3\u6216\u9884\u7B97\u5B57\u6BB5\u65E0\u6548: ${a.name}`);
      if (!Cr(a.logic_hash)) throw new Error(`\u6269\u5C55\u5305 VM \u6A21\u5757\u6307\u7EB9\u683C\u5F0F\u65E0\u6548: ${a.name}`);
      n.set(a.name, l.length);
    }
    const s = /* @__PURE__ */ new Set(), r = e.skill_index;
    if (r !== void 0) {
      if (!Array.isArray(r)) throw new Error("\u6269\u5C55\u5305\u6280\u80FD\u7D22\u5F15\u683C\u5F0F\u65E0\u6548");
      for (const o of r) {
        const a = o;
        if (!a || typeof a != "object" || !es(a.skill_def_id) || !bs(a.name) || s.has(a.skill_def_id)) throw new Error("\u6269\u5C55\u5305\u6280\u80FD\u7D22\u5F15\u6761\u76EE\u65E0\u6548\u6216\u91CD\u590D");
        s.add(a.skill_def_id);
      }
    }
    const i = e.trigger_index;
    if (i !== void 0) {
      if (!Array.isArray(i)) throw new Error("\u6269\u5C55\u5305\u89E6\u53D1\u7D22\u5F15\u683C\u5F0F\u65E0\u6548");
      for (const o of i) {
        const a = o;
        if (!a || typeof a != "object" || typeof a.module != "string" || !n.has(a.module)) throw new Error("\u6269\u5C55\u5305\u89E6\u53D1\u7D22\u5F15\u5F15\u7528\u4E86\u4E0D\u5B58\u5728\u7684 VM \u6A21\u5757");
        if (!es(a.entry_pc) || a.entry_pc >= n.get(a.module) || !es(a.skill_def_id) || r !== void 0 && !s.has(a.skill_def_id)) throw new Error(`\u6269\u5C55\u5305\u89E6\u53D1\u7D22\u5F15\u5165\u53E3\u6216\u6280\u80FD\u5F15\u7528\u65E0\u6548: ${a.module}`);
      }
    }
  }
  function as(e) {
    return `${e.manifest.namespace}/${e.manifest.package_id}`;
  }
  function Fa(e) {
    return e.manifest.logic_hash.map((t) => t.toString(16).padStart(2, "0")).join("");
  }
  const Wa = "noname-next.extension-store.v1", qa = "noname-next.native-saves.v1";
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
      const e = localStorage.getItem(Wa);
      return e ? {
        ...ts(),
        ...JSON.parse(e)
      } : ts();
    } catch {
      return ts();
    }
  }
  function Od(e) {
    typeof localStorage > "u" || localStorage.setItem(Wa, JSON.stringify(e));
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
      const e = localStorage.getItem(qa);
      return e ? JSON.parse(e) : [];
    } catch {
      return [];
    }
  }
  function Nd(e) {
    const t = Ka().filter((n) => n.id !== e.id);
    return t.unshift(e), localStorage.setItem(qa, JSON.stringify(t.slice(0, 20))), t;
  }
  function Ga(e) {
    const t = e.map(as);
    if (new Set(t).size !== t.length) throw new Error("\u5185\u5BB9\u9501\u5305\u542B\u91CD\u590D\u6269\u5C55");
    const n = new Map(e.map((s) => [
      as(s),
      s
    ]));
    for (const s of e) for (const r of s.manifest.dependencies ?? []) {
      const i = `${r.namespace}/${r.package_id}`, o = n.get(i);
      if (!o) throw new Error(`\u6269\u5C55\u5305\u7F3A\u5C11\u4F9D\u8D56: ${i}@${r.version}`);
      const a = r.logic_hash;
      if (o.manifest.version !== r.version || (a == null ? void 0 : a.some((l) => l !== 0)) && a.some((l, c) => l !== o.manifest.logic_hash[c])) throw new Error(`\u6269\u5C55\u5305\u4F9D\u8D56\u7248\u672C\u6216\u6307\u7EB9\u4E0D\u5339\u914D: ${i}@${r.version}`);
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
      const s = e.installed.find((o) => o.key === n);
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
  const Dd = {
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
  function jd(e) {
    return Dd[e] ?? `\u4E8B\u4EF6 ${e}`;
  }
  const uo = 5, ws = "noname-next.pending-command", Ld = 600 * 1e3;
  function Ud() {
    try {
      if (typeof sessionStorage > "u") return null;
      const e = sessionStorage.getItem(ws);
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
      sessionStorage.setItem(ws, JSON.stringify({
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
      const t = sessionStorage.getItem(ws);
      if (!t) return;
      JSON.parse(t).key === e && sessionStorage.removeItem(ws);
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
      const s = new Promise((i, o) => {
        this.welcomeWaiter = {
          resolve: i,
          reject: o
        };
      }), r = new Promise((i, o) => {
        const a = new WebSocket(Ja(t));
        this.socket = a, a.onopen = () => {
          if (this.socket !== a) {
            o(new Error("WebSocket \u8FDE\u63A5\u5DF2\u66FF\u6362"));
            return;
          }
          this.connSeq = 0, this.idemSeq = 1, i();
        }, a.onerror = () => o(new Error("WebSocket \u8FDE\u63A5\u5931\u8D25")), a.onmessage = (l) => {
          this.socket === a && this.handleMessage(String(l.data));
        }, a.onclose = () => {
          o(new Error("WebSocket \u8FDE\u63A5\u5DF2\u5173\u95ED")), this.socket === a && this.handleClose();
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
      const r = new Promise((i, o) => {
        s = {
          kind: t,
          key: n,
          resolve: i,
          reject: o
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
            envelope: tr(r, this.choice, [], s.legal_actions ?? [], null)
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
          const o = this.snapshotWaiters.get(r.conn_seq);
          o && (this.snapshotWaiters.delete(r.conn_seq), o.resolve(i));
          const a = tr(i, this.choice, [], s.legal_actions ?? [], null);
          for (const l of this.envelopeListeners) l(a);
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
      const r = t.public_event_codes ?? [], i = tr(n, this.choice, r, t.legal_actions ?? [], t.result ?? null), o = Number(t.idempotency_key ?? 0);
      ns(o), ((_a2 = this.persistedPending) == null ? void 0 : _a2.key) === o && (this.persistedPending = null);
      const a = this.pending.get(o);
      if (a) {
        this.pending.delete(o);
        const l = Vd(t.result);
        if (l) {
          for (const c of this.envelopeListeners) c(i);
          a.reject(new Error(l));
        } else a.resolve(i);
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
  function tr(e, t, n, s, r) {
    return {
      result: r,
      events: n.map((i) => ({
        PublicEvent: {
          code: i,
          label: jd(i)
        }
      })),
      choice: t,
      view: e,
      legal_actions: s,
      hash_hex: Wd(e.state_hash),
      revision: e.revision,
      finished: qd(e.outcome)
    };
  }
  function Wd(e) {
    return typeof e == "string" ? e : e.map((t) => t.toString(16).padStart(2, "0")).join("");
  }
  function qd(e) {
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
    return pn({
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
  let zt = 1;
  function ho(e, t) {
    pd(), Yd(e, t), gd();
  }
  function Yd(e, t) {
    e.revision = t.revision, e.hash = t.hash_hex, e.finished = t.finished, e.view = t.view, e.choice = t.choice, e.legalActions = t.legal_actions ?? [], e.lastEvents = t.events ?? [], e.error = null;
    const n = Qd(t.events ?? []);
    e.effects = n;
    const s = Zd(t.events ?? []);
    for (const r of s) Je(e, r);
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
            id: zt++,
            type: "damage",
            player: i == null ? void 0 : i.target,
            amount: i == null ? void 0 : i.amount,
            text: `-${i == null ? void 0 : i.amount}`
          });
          break;
        case "HpRecovered":
          t.push({
            id: zt++,
            type: "heal",
            player: i == null ? void 0 : i.player,
            amount: i == null ? void 0 : i.amount,
            text: `+${i == null ? void 0 : i.amount}`
          });
          break;
        case "CardsDrawn":
          t.push({
            id: zt++,
            type: "draw",
            player: i == null ? void 0 : i.player,
            text: `\u6478${Array.isArray(i == null ? void 0 : i.cards) ? i.cards.length : 1}`
          });
          break;
        case "CardUsed":
          t.push({
            id: zt++,
            type: "play",
            player: i == null ? void 0 : i.player,
            text: "\u51FA\u724C"
          });
          break;
        case "EnteredDying":
          t.push({
            id: zt++,
            type: "dying",
            player: i == null ? void 0 : i.player,
            text: "\u6FD2\u6B7B"
          });
          break;
        case "PlayerKilled":
          t.push({
            id: zt++,
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
  function Je(e, t) {
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
  function Xt(e) {
    return typeof e == "object" && e !== null && !Array.isArray(e);
  }
  function Ss(e) {
    return typeof e == "string" ? e : e instanceof Error ? e.message : String(e);
  }
  function Pn(e) {
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
    if (!Xt(e)) throw new Error(`\u9009\u5C06\u5019\u9009\u7ED3\u6784\u4E0D\u7B26\uFF08\u9700 {count, candidates}\uFF09\uFF1A${Pn(e)}`);
    const t = e.count, n = e.candidates;
    if (typeof t != "number" || !Number.isInteger(t) || t < 0 || !Array.isArray(n)) throw new Error(`\u9009\u5C06\u5019\u9009\u7ED3\u6784\u4E0D\u7B26\uFF08\u9700 {count:\u975E\u8D1F\u6574\u6570, candidates:\u6570\u7EC4}\uFF09\uFF1A${Pn(e)}`);
    const s = [], r = /* @__PURE__ */ new Set();
    for (const [i, o] of n.entries()) {
      if (!Xt(o)) throw new Error(`\u9009\u5C06\u5019\u9009\u7B2C ${i} \u9879\u7ED3\u6784\u4E0D\u7B26\uFF08\u9700\u5BF9\u8C61\uFF09\uFF1A${Pn(o)}`);
      const a = o.def, l = o.name, c = o.skills;
      if (typeof a != "number" || !Number.isInteger(a) || a < 0 || typeof l != "string" || l.trim().length === 0 || !th(c)) throw new Error(`\u9009\u5C06\u5019\u9009\u7B2C ${i} \u9879\u7ED3\u6784\u4E0D\u7B26\uFF08\u9700 {def:\u975E\u8D1F\u6574\u6570, name:\u975E\u7A7A\u5B57\u7B26\u4E32, skills:\u5B57\u7B26\u4E32\u6570\u7EC4}\uFF09\uFF1A${Pn(o)}`);
      if (r.has(a)) throw new Error(`\u9009\u5C06\u5019\u9009\u7B2C ${i} \u9879 def=${a} \u4E0E\u524D\u9762\u91CD\u590D`);
      r.add(a), s.push({
        def: a,
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
    const s = (Array.isArray(e == null ? void 0 : e.seats) ? e.seats : []).find((a) => a && a.player === t);
    if (!s) return Yr();
    const r = [], i = [], o = Array.isArray(s.skills) ? s.skills : [];
    for (const a of o) {
      const l = a == null ? void 0 : a.id;
      typeof l == "number" && Number.isInteger(l) && !r.includes(l) && r.push(l);
      const c = a == null ? void 0 : a.name;
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
        error: `\u9009\u5C06\u5019\u9009\u4E0D\u53EF\u7528\uFF1A${Ss(n)}`
      };
    }
  }
  function oh(e, t) {
    return {
      ...e,
      loadStatus: "error",
      candidates: [],
      error: `\u9009\u5C06\u5019\u9009\u52A0\u8F7D\u5931\u8D25\uFF1A${Ss(t)}`
    };
  }
  function ah(e, t) {
    if (!Xt(e)) return `\u56DE\u5305\u4E0D\u662F\u5BF9\u8C61\uFF1A${Pn(e)}`;
    const n = e.assignment, s = e.envelope;
    return Xt(n) ? n.seat !== t.seat ? `\u5EA7\u4F4D\u56DE\u663E ${String(n.seat)} \u2260 \u8BF7\u6C42 ${t.seat}` : n.name !== t.name ? `\u6B66\u5C06\u56DE\u663E ${String(n.name)} \u2260 \u8BF7\u6C42 ${t.name}` : typeof n.state_hash != "string" || n.state_hash.length === 0 ? "\u7F3A assignment.state_hash" : !Xt(s) || !Xt(s.view) ? "\u7F3A\u5237\u65B0\u4FE1\u5C01 view" : null : "\u7F3A assignment";
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
        error: `\u9009\u5C06\u5931\u8D25\uFF08seat=${e.seat}, name=${t}\uFF09\uFF1A${Ss(f)}`
      };
    }
    const o = ah(i, {
      seat: e.seat,
      name: t
    });
    if (o) return {
      ...e,
      selectionStatus: "rejected",
      pendingName: null,
      error: `\u9009\u5C06\u8FD4\u56DE\u65E0\u6CD5\u786E\u8BA4\u88C5\u914D\uFF1A${o}`
    };
    const a = i.assignment, l = i.envelope, c = Xa({
      ...r,
      assignment: a,
      lastStateHash: a.state_hash
    }, Qa(l.view, e.seat));
    let u = null;
    try {
      (_a2 = n.applyEnvelope) == null ? void 0 : _a2.call(n, l);
    } catch (f) {
      u = `\u9009\u5C06\u5DF2\u88C5\u914D\uFF0C\u4F46\u89C6\u56FE\u5237\u65B0\u5931\u8D25\uFF1A${Ss(f)}`;
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
  ], mh = Ut({
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
      }, o = Q(() => {
        var _a2;
        const L = ((_a2 = t.general) == null ? void 0 : _a2.faction) ?? "";
        return Object.hasOwn(i, L) ? i[L] : "\u672A\u77E5\u52BF\u529B";
      }), a = Q(() => Sd(n.value)), l = Q(() => Er(n.value)), c = Q(() => Ad(n.value)), u = Q(() => xd(n.value)), f = Q(() => js(n.value)), m = Q(() => Array.from(/* @__PURE__ */ new Set([
        ...t.alive ? [] : [
          Cd(n.value, "dead")
        ],
        c.value,
        u.value,
        f.value
      ])).filter((L) => !!L)), _ = ye([]), P = Q(() => m.value.find((L) => !_.value.includes(L)) ?? "");
      Kt([
        n,
        s
      ], ([L, E]) => {
        E && bd(L).catch(() => {
        });
      }, {
        immediate: true
      }), Kt(m, () => {
        _.value = [];
      });
      function R() {
        P.value && (_.value = [
          ..._.value,
          P.value
        ]);
      }
      function M(L) {
        const E = L.target.value;
        Ed(n.value, E === "" ? null : Number(E));
      }
      return (L, E) => (x(), C("div", {
        class: "general-slot",
        "data-slot": e.slot,
        "aria-label": `${e.slot === 0 ? "\u4E3B\u5C06" : "\u526F\u5C06"} ${r.value}`
      }, [
        s.value && P.value ? (x(), C("img", {
          key: 0,
          class: "portrait",
          src: P.value,
          alt: `${r.value}\u6B66\u5C06\u5934\u50CF`,
          onError: R
        }, null, 40, uh)) : (x(), C("div", {
          key: 1,
          class: "portrait no-image",
          role: "img",
          "aria-label": `${r.value}\u6682\u65E0\u5934\u50CF`
        }, I(s.value ? "\u65E0\u56FE" : "\u672A\u77E5"), 9, fh)),
        k("span", dh, I(r.value), 1),
        e.general ? (x(), C("span", hh, I(o.value) + " \xB7 " + I(e.general.revealed ? "\u660E\u7F6E" : "\u6697\u7F6E"), 1)) : z("", true),
        s.value && a.value.length ? (x(), C("select", {
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
          onChange: M
        }, [
          E[2] || (E[2] = k("option", {
            value: ""
          }, "\u9ED8\u8BA4\u76AE\u80A4", -1)),
          (x(true), C(de, null, Ce(a.value, (H) => (x(), C("option", {
            key: H.id,
            value: H.id
          }, I(q(kd)(H)) + I(H.presentation.kind === "spine" ? "\uFF08\u9759\u6001\u56DE\u9000\uFF09" : ""), 9, gh))), 128))
        ], 40, ph)) : z("", true)
      ], 8, ch));
    }
  }), mn = (e, t) => {
    const n = e.__vccOpts || e;
    for (const [s, r] of t) n[s] = r;
    return n;
  }, vh = mn(mh, [
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
  }, $h = Ut({
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
      ]), i = Q(() => !n.seat.generals || r.value.every((c) => c === null) && n.seat.character_def !== 0 ? n.seat.character_def : void 0), o = Q(() => r.value.map((c, u) => (c == null ? void 0 : c.name) || (u === 0 && i.value !== void 0 ? `#${i.value}` : "\u6697\u5C06\uFF0F\u672A\u914D\u7F6E")).join(" / "));
      function a(c, u, f) {
        return f ? c <= 0 ? "\u26A0" : "\u2713" : "\u2717";
      }
      function l(c, u, f) {
        return f ? c <= 0 ? "var(--color-warning)" : "var(--color-positive)" : "var(--color-negative)";
      }
      return (c, u) => (x(), C("article", {
        class: We([
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
        "aria-label": `\u5EA7\u4F4D ${e.seat.seat} \u73A9\u5BB6 ${e.seat.player} \u6B66\u5C06 ${o.value}`,
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
          (x(true), C(de, null, Ce(r.value, (f, m) => (x(), Br(vh, {
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
            style: Gt({
              color: l(e.seat.hp, e.seat.max_hp, e.seat.alive)
            }),
            "aria-label": e.seat.alive ? e.seat.hp <= 0 ? "\u6FD2\u6B7B" : "\u5B58\u6D3B" : "\u9635\u4EA1"
          }, I(a(e.seat.hp, e.seat.max_hp, e.seat.alive)), 13, kh),
          k("span", Eh, I(e.seat.hp) + " / " + I(e.seat.max_hp), 1)
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
            style: Gt({
              width: Math.max(Math.min(e.seat.hp / e.seat.max_hp * 100, 100), 0) + "%",
              background: e.seat.alive ? e.seat.hp <= 0 ? "var(--color-warning)" : "var(--color-positive)" : "var(--color-negative)"
            })
          }, null, 4)
        ], 8, Ah),
        k("div", xh, [
          k("span", null, "\u624B\u724C " + I(e.seat.hand_count), 1),
          k("span", null, I(e.identityLabel), 1)
        ]),
        e.equipCards.length > 0 ? (x(), C("div", Ch, [
          (x(true), C(de, null, Ce(e.equipCards, (f) => (x(), C("div", {
            key: f.id,
            class: "equip-card",
            title: `\u5361\u724C #${f.id}${f.def != null ? " (def:" + f.def + ")" : ""}`
          }, [
            k("span", Ph, I(f.suit === "Spade" ? "\u2660" : f.suit === "Heart" ? "\u2665" : f.suit === "Club" ? "\u2663" : f.suit === "Diamond" ? "\u2666" : "?"), 1),
            k("span", Th, "#" + I(f.id), 1)
          ], 8, Rh))), 128))
        ])) : z("", true),
        e.seat.alive ? e.seat.hp <= 0 ? (x(), C("div", Oh, "\u26A0 \u6FD2\u6B7B")) : z("", true) : (x(), C("div", Ih, "\u2717 \u9635\u4EA1")),
        e.isTargetMode && e.isSelectableTarget ? (x(), C("div", Nh, " \u25B8 \u70B9\u51FB\u9009\u62E9\u4E3A\u76EE\u6807 ")) : z("", true)
      ], 42, yh));
    }
  }), Za = mn($h, [
    [
      "__scopeId",
      "data-v-fdcad0a8"
    ]
  ]), Mh = {
    class: "hand",
    "aria-label": "\u624B\u724C"
  }, Dh = {
    class: "muted title"
  }, jh = {
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
  }, Wh = {
    key: 0,
    class: "card-known"
  }, qh = {
    key: 0,
    class: "muted empty"
  }, Vh = {
    key: 0,
    class: "hint"
  }, Bh = {
    key: 1,
    class: "hint drag-hint"
  }, Kh = Ut({
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
      function o(f) {
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
      function a(f) {
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
        k("h2", Dh, " \u624B\u724C\uFF08" + I(e.cards.length) + " \u5F20" + I(r.value != null ? " \xB7 \u5DF2\u9009\u4E2D #" + r.value : " \xB7 \u70B9\u51FB\u6216\u62D6\u62FD") + "\uFF09 ", 1),
        k("div", jh, [
          (x(true), C(de, null, Ce(e.cards, (_) => (x(), C("button", {
            key: _.id,
            type: "button",
            class: We([
              "card",
              {
                selected: _.id === r.value,
                dragging: _.id === i.value
              }
            ]),
            disabled: !e.actionableIds.includes(_.id),
            role: "option",
            "aria-selected": _.id === r.value,
            "aria-label": `\u5361\u724C ${_.id}${_.def != null ? " \u5DF2\u77E5" : " \u672A\u77E5"}`,
            draggable: "true",
            onClick: (P) => l(_.id),
            onDragstart: (P) => c(P, _.id),
            onDragend: u
          }, [
            k("span", {
              class: "suit",
              "data-suit": _.suit
            }, I(o(_.suit)), 9, Uh),
            k("span", Hh, I(a(_.rank)), 1),
            k("span", Fh, "#" + I(_.id), 1),
            _.def != null ? (x(), C("span", Wh, "\u25B8")) : z("", true)
          ], 42, Lh))), 128)),
          e.cards.length ? z("", true) : (x(), C("p", qh, "\u65E0\u53EF\u89C1\u624B\u724C"))
        ]),
        r.value != null ? (x(), C("p", Vh, " \u2713 \u5DF2\u9009\u4E2D #" + I(r.value) + " \u2014 \u5728\u68CB\u76D8\u4E0A\u9009\u62E9\u76EE\u6807\uFF0C\u6216\u70B9\u51FB\u300C\u7ED3\u675F\u9636\u6BB5\u300D\u76F4\u63A5\u6253\u51FA ", 1)) : z("", true),
        i.value != null ? (x(), C("p", Bh, " \u{1F0A0} \u62D6\u62FD\u4E2D #" + I(i.value) + " \u2014 \u62D6\u5230\u76EE\u6807\u73A9\u5BB6\u91CA\u653E\u51FA\u724C ", 1)) : z("", true)
      ]));
    }
  }), Gh = mn(Kh, [
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
  }, vp = Ut({
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
      const n = e, s = t, r = ye(null), i = ye(null), o = ye(false), a = ye([]), l = ye(null), c = ye(null);
      function u() {
        var _a2;
        return ((_a2 = n.view) == null ? void 0 : _a2.seats) ?? [];
      }
      function f() {
        return n.view ? n.view.known_cards.filter((O) => O.owner === n.selfId && O.zone === "Hand" && O.def != null) : [];
      }
      function m() {
        var _a2;
        const O = new Set(n.legalActions.flatMap((D) => typeof D == "object" && "ActivateSkill" in D ? [
          D.ActivateSkill.skill
        ] : []));
        return ((_a2 = u().find((D) => D.player === n.selfId)) == null ? void 0 : _a2.skills.filter((D) => D.enabled && (D.active || D.view_as) && O.has(D.id))) ?? [];
      }
      function _(O) {
        switch (O) {
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
      function P() {
        return o.value ? new Set(X().flat()) : /* @__PURE__ */ new Set();
      }
      function R() {
        return n.view ? n.view.known_cards.filter((O) => O.zone === "DiscardPile" && O.def != null) : [];
      }
      function M(O) {
        return n.view ? n.view.known_cards.filter((D) => D.owner === O && D.zone === "Judge" && D.def != null) : [];
      }
      const L = Q(() => P()), E = Q(() => i.value ? Array.from(new Set($(i.value.id).flatMap((O) => O.card_targets))) : Array.from(new Set(H().map((O) => O.card))));
      function H(O) {
        return n.legalActions.flatMap((D) => typeof D != "object" || !("PlayCard" in D) ? [] : O != null && D.PlayCard.card !== O ? [] : [
          D.PlayCard
        ]);
      }
      function $(O) {
        return n.legalActions.flatMap((D) => typeof D != "object" || !("ActivateSkill" in D) ? [] : D.ActivateSkill.skill === O ? [
          D.ActivateSkill
        ] : []);
      }
      function X() {
        return i.value ? $(i.value.id).filter((O) => r.value == null ? O.card_targets.length === 0 : O.card_targets.length === 1 && O.card_targets[0] === r.value).map((O) => O.targets) : r.value == null ? [] : H(r.value).map((O) => O.targets);
      }
      function se(O, D) {
        return O.length === D.length && O.every((F, ie) => F === D[ie]);
      }
      const re = Q(() => X().some((O) => se(O, a.value))), Me = Q(() => X().some((O) => O.length === 0));
      function De(O) {
        return n.view ? n.view.known_cards.filter((D) => D.owner === O && D.zone === "Equip" && D.def != null) : [];
      }
      function je(O) {
        if (E.value.includes(O)) {
          if (r.value === O) {
            r.value = null, o.value = false, a.value = [];
            return;
          }
          r.value = O, i.value || (i.value = null), o.value = true, a.value = [];
        }
      }
      function Le(O) {
        var _a2;
        if (((_a2 = i.value) == null ? void 0 : _a2.id) === O.id) {
          Te();
          return;
        }
        i.value = O, r.value = null, o.value = true, a.value = [];
      }
      function Qe(O) {
        E.value.includes(O) && (l.value = O, r.value = O, o.value = true);
      }
      function Ue(O) {
        if (!(!o.value || !L.value.has(O))) if (a.value.includes(O)) a.value = a.value.filter((D) => D !== O);
        else {
          const D = Math.max(0, ...X().map((F) => F.length));
          a.value = [
            ...a.value,
            O
          ].slice(0, D);
        }
      }
      function _t(O, D) {
        l.value != null && L.value.has(D) && (O.preventDefault(), O.dataTransfer.dropEffect = "move", c.value = D);
      }
      function rt(O) {
        c.value === O && (c.value = null);
      }
      function ge(O, D) {
        var _a2;
        O.preventDefault();
        const F = l.value;
        if (c.value = null, l.value = null, F != null && L.value.has(D)) {
          if ((_a2 = i.value) == null ? void 0 : _a2.view_as) {
            const ie = $(i.value.id).find((Ie) => se(Ie.targets, [
              D
            ]) && Ie.card_targets.length === 1 && Ie.card_targets[0] === F);
            if (!ie) return;
            s("activateSkill", ie.skill, ie.targets, ie.card_targets);
          } else {
            const ie = H(F).find((Ie) => se(Ie.targets, [
              D
            ]));
            if (!ie) return;
            s("playCard", ie.card, ie.targets);
          }
          r.value = null, i.value = null, o.value = false, a.value = [];
        }
      }
      function te() {
        if (re.value) {
          if (i.value) {
            const O = $(i.value.id).find((D) => se(D.targets, a.value) && (r.value == null ? D.card_targets.length === 0 : D.card_targets.length === 1 && D.card_targets[0] === r.value));
            if (!O) return;
            s("activateSkill", O.skill, O.targets, O.card_targets);
          } else {
            if (r.value == null) return;
            const O = H(r.value).find((D) => se(D.targets, a.value));
            if (!O) return;
            s("playCard", O.card, O.targets);
          }
          r.value = null, i.value = null, o.value = false, a.value = [];
        }
      }
      function ee() {
        a.value = [], te();
      }
      function Te() {
        r.value = null, i.value = null, o.value = false, a.value = [];
      }
      return (O, D) => {
        var _a2;
        return x(), C("div", Jh, [
          r.value != null || i.value ? (x(), C("div", zh, [
            k("span", Yh, [
              D[0] || (D[0] = Ct(" \u25B8 ", -1)),
              i.value ? (x(), C(de, {
                key: 0
              }, [
                Ct("\u6280\u80FD #" + I(i.value.def), 1)
              ], 64)) : (x(), C(de, {
                key: 1
              }, [
                Ct("\u624B\u724C #" + I(r.value), 1)
              ], 64)),
              ((_a2 = i.value) == null ? void 0 : _a2.view_as) && r.value != null ? (x(), C("span", Qh, " \xB7 \u7D20\u6750 #" + I(r.value), 1)) : z("", true),
              o.value && a.value.length > 0 ? (x(), C("span", Xh, " \u2192 \u76EE\u6807 P" + I(a.value[0]), 1)) : z("", true)
            ]),
            k("div", Zh, [
              k("button", {
                class: "primary",
                type: "button",
                onClick: te,
                disabled: !re.value,
                "aria-label": "\u786E\u8BA4\u884C\u52A8"
              }, I(i.value ? "\u53D1\u52A8\u6280\u80FD" : "\u786E\u8BA4\u51FA\u724C"), 9, ep),
              k("button", {
                type: "button",
                disabled: !Me.value,
                onClick: ee,
                "aria-label": "\u65E0\u76EE\u6807\u76F4\u63A5\u6253\u51FA"
              }, I(i.value ? "\u65E0\u76EE\u6807\u53D1\u52A8" : "\u76F4\u63A5\u6253\u51FA"), 9, tp),
              k("button", {
                type: "button",
                onClick: Te,
                "aria-label": "\u53D6\u6D88\u9009\u62E9"
              }, " \u53D6\u6D88 ")
            ])
          ])) : z("", true),
          m().length ? (x(), C("section", np, [
            D[1] || (D[1] = k("span", {
              class: "muted"
            }, "\u6280\u80FD", -1)),
            (x(true), C(de, null, Ce(m(), (F) => {
              var _a3;
              return x(), C("button", {
                key: F.id,
                type: "button",
                class: We({
                  primary: ((_a3 = i.value) == null ? void 0 : _a3.id) === F.id
                }),
                onClick: (ie) => Le(F)
              }, " #" + I(F.def) + I(F.view_as ? " \xB7 \u89C6\u4E3A" : ""), 11, sp);
            }), 128))
          ])) : z("", true),
          k("div", rp, [
            (x(true), C(de, null, Ce(u(), (F) => (x(), C("div", {
              key: F.player,
              class: "seat-wrapper",
              onDragover: (ie) => _t(ie, F.player),
              onDragleave: (ie) => rt(F.player),
              onDrop: (ie) => ge(ie, F.player)
            }, [
              Ee(Za, {
                seat: F,
                self: F.player === e.selfId,
                "identity-label": _(F.identity),
                "is-target-mode": o.value,
                "is-selectable-target": L.value.has(F.player),
                "is-drop-target": c.value === F.player,
                "equip-cards": De(F.player),
                onSelect: Ue
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
          Ee(Gh, {
            cards: f(),
            "actionable-ids": E.value,
            onSelect: je,
            onDragStart: Qe
          }, null, 8, [
            "cards",
            "actionable-ids"
          ]),
          u().some((F) => M(F.player).length > 0) ? (x(), C("section", op, [
            D[2] || (D[2] = k("h3", {
              class: "muted jz-title"
            }, "\u2696 \u5224\u5B9A\u533A", -1)),
            k("div", ap, [
              (x(true), C(de, null, Ce(u(), (F) => (x(), C("div", {
                key: "jz-" + F.player,
                class: "jz-entry"
              }, [
                (x(true), C(de, null, Ce(M(F.player), (ie) => (x(), C("span", {
                  key: ie.id,
                  class: "jz-card",
                  title: `P${F.player} #${ie.id}`
                }, [
                  k("span", cp, I(ie.suit === "Spade" ? "\u2660" : ie.suit === "Heart" ? "\u2665" : ie.suit === "Club" ? "\u2663" : ie.suit === "Diamond" ? "\u2666" : "?"), 1),
                  k("span", up, "P" + I(F.player), 1)
                ], 8, lp))), 128))
              ]))), 128))
            ])
          ])) : z("", true),
          R().length > 0 ? (x(), C("section", fp, [
            k("h3", dp, "\u5F03\u724C\u5806\uFF08" + I(R().length) + "\uFF09", 1),
            k("div", hp, [
              (x(true), C(de, null, Ce(R().slice(-10), (F) => (x(), C("div", {
                key: F.id,
                class: "discard-card",
                title: `#${F.id} def:${F.def}`
              }, [
                k("span", gp, I(F.suit === "Spade" ? "\u2660" : F.suit === "Heart" ? "\u2665" : F.suit === "Club" ? "\u2663" : F.suit === "Diamond" ? "\u2666" : "?"), 1)
              ], 8, pp))), 128)),
              R().length > 10 ? (x(), C("span", mp, "+" + I(R().length - 10), 1)) : z("", true)
            ])
          ])) : z("", true)
        ]);
      };
    }
  }), yp = mn(vp, [
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
  }, $p = Ut({
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
      }), o = Q(() => n.state.projection.skillNames.length > 0 ? n.state.projection.skillNames.join(" \xB7 ") : "\u89C6\u56FE\u672A\u66B4\u9732\u6280\u80FD\u540D"), a = Q(() => {
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
            k("p", wp, " NC05 \xB7 \u76EE\u6807\u5EA7\u4F4D P" + I(e.state.seat) + " \xB7 " + I(a.value) + " \xB7 \u70B9\u51FB\u5019\u9009\u5373\u6309\u540D\u88C5\u914D \uFF08\u539F\u751F\u6B63\u5F0F\u5165\u53E3\uFF0C\u975E debug\uFF09 ", 1)
          ]),
          k("button", {
            type: "button",
            disabled: r.value || e.state.loadStatus === "loading",
            "aria-label": "\u5237\u65B0\u9009\u5C06\u5019\u9009",
            onClick: c[0] || (c[0] = (u) => s("reload"))
          }, " \u5237\u65B0\u5019\u9009 ", 8, Sp)
        ]),
        e.state.loadStatus === "loading" && e.state.candidates.length === 0 ? (x(), C("p", kp, " \u6B63\u5728\u8BFB\u53D6\u539F\u751F\u76EE\u5F55\u5019\u9009\u2026 ")) : e.state.loadStatus === "empty" ? (x(), C("p", Ep, " \u5019\u9009\u4E3A\u7A7A\uFF1A\u5185\u6838\u76EE\u5F55\u6CA1\u6709\u8FD4\u56DE\u4EFB\u4F55\u6B66\u5C06\uFF08\u8FD9\u662F\u5408\u6CD5\u7684\u7A7A\u76EE\u5F55\uFF0C\u4E0D\u662F\u52A0\u8F7D\u5931\u8D25\uFF09\u3002 ")) : z("", true),
        e.state.error ? (x(), C("p", Ap, I(e.state.error), 1)) : z("", true),
        e.state.selectionStatus === "accepted" && e.state.assignment ? (x(), C("p", xp, [
          Ct(" \u2705 \u5DF2\u4E3A P" + I(e.state.assignment.seat) + " \u88C5\u914D ", 1),
          k("strong", null, I(e.state.assignment.name), 1),
          k("span", Cp, " \xB7 state_hash " + I(e.state.assignment.state_hash.slice(0, 16)) + "\u2026 ", 1),
          c[2] || (c[2] = k("br", null, null, -1)),
          k("span", Rp, "\u5237\u65B0\u540E\u89C6\u56FE\u6280\u80FD\uFF1A" + I(o.value), 1)
        ])) : z("", true),
        e.state.candidates.length > 0 ? (x(), C("ul", Pp, [
          (x(true), C(de, null, Ce(e.state.candidates, (u) => (x(), C("li", {
            key: u.def
          }, [
            k("button", {
              type: "button",
              class: We([
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
              k("span", Ip, I(u.name), 1),
              k("span", Op, I(u.skillLabel), 1),
              k("span", Np, "#" + I(u.def), 1)
            ], 10, Tp)
          ]))), 128))
        ])) : z("", true)
      ]));
    }
  }), Mp = mn($p, [
    [
      "__scopeId",
      "data-v-9fa84d6d"
    ]
  ]), Dp = {
    class: "battle stack"
  }, jp = {
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
  }, Wp = {
    key: 2,
    class: "panel turn-bar",
    role: "status",
    "aria-live": "polite"
  }, qp = {
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
  }, Dg = {
    class: "ss-header"
  }, jg = {
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
  }, Wg = {
    class: "ss-meta muted"
  }, qg = [
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
  }, Qg = Ut({
    __name: "BattleView",
    setup(e) {
      var _a2;
      const t = Mf(), n = $f(), s = zd(), r = new Ds();
      let i;
      const o = new Fd(), a = ye(1);
      try {
        ((_a2 = globalThis.localStorage) == null ? void 0 : _a2.getItem("noname-next-skip-anims")) === "1" && (s.skipAnims = true);
      } catch {
      }
      if (t.query.skipAnims === "1" && (s.skipAnims = true), t.query.speed !== void 0) {
        const g = Number(t.query.speed);
        Number.isFinite(g) && g > 0 && (s.speed = g);
      }
      const l = ye(Number(t.query.players ?? 4) || 4), c = ye(Number(t.query.seed ?? 42) || 42), u = ye(null), f = ye(false), m = ye("\u672A\u8FDE\u63A5"), _ = ye(null), P = ye(0), R = ye(false), M = Q(() => t.query.mode === "online"), L = Q(() => M.value && a.value === 0), E = ye(sh(a.value)), H = Q(() => !M.value && s.connected && r.isNative());
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
        }), M.value && (!s.connected || s.revision !== g.revision || s.hash !== g.hash_hex)) return;
        Xd(s);
        const T = M.value ? `\u623F\u95F4 #${P.value} \xB7 ` : "", A = (g.hash_hex || "").slice(0, 12);
        m.value = g.finished ? `${T}\u7EC8\u5C40 \xB7 ${h.value.label} \xB7 rev ${g.revision}` : `${T}rev ${g.revision}${A ? ` \xB7 ${A}\u2026` : ""}${r.isNative() ? " \xB7 native" : ""}`, Me();
      }
      async function re(g) {
        if (await se(g), !M.value && r.isNative()) {
          const p = Gd(g.result);
          if (p) throw new Error(p);
        }
      }
      function Me() {
        E.value = Xa(E.value, Qa(s.view, E.value.seat));
      }
      async function De() {
        if (!(M.value || !r.isNative())) {
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
        if (M.value || !r.isNative() || s.busy || E.value.selectionStatus === "pending") return;
        const p = E.value;
        await $(async () => {
          if (E.value = await lh(p, g, {
            chooseGeneral: (w, v) => r.chooseGeneral(w, v),
            applyEnvelope: (w) => {
              ho(s, w), Je(s, `\u9009\u5C06 P${p.seat} \u2192 ${g}`);
            }
          }, (w) => {
            E.value = w;
          }), E.value.selectionStatus === "accepted") {
            const w = E.value.lastStateHash ?? "";
            m.value = `\u9009\u5C06 P${p.seat} \xB7 ${g}${w ? ` \xB7 ${w.slice(0, 12)}\u2026` : ""}`;
          } else E.value.selectionStatus === "rejected" && (m.value = E.value.error ?? "\u9009\u5C06\u5931\u8D25");
        });
      }
      async function Le() {
        await $(async () => {
          try {
            if (await Xi(true), co(true), M.value) {
              const v = te();
              _d(v.wsUrl), await _t(v);
              return;
            }
            yd(), X("t0_click"), await r.connect(), X("t1_worker_ready"), s.connected = true, m.value = r.isNative() ? "\u539F\u751F Worker \u5C31\u7EEA\uFF0C\u5F00\u5C40\u4E2D\u2026" : "Worker \u5C31\u7EEA\uFF0C\u5F00\u5C40\u4E2D\u2026", r.isNative() && (a.value = Math.max(0, Math.min(a.value, l.value - 1)), E.value = po(E.value, a.value));
            const g = $d();
            if (!r.isNative() && g.contentLock.length) throw new Error("\u5DF2\u5B89\u88C5\u6269\u5C55\u7684\u6B63\u5F0F\u5F00\u5C40\u9700\u8981\u539F\u751F\u540E\u7AEF");
            const p = await r.start(l.value, c.value, a.value, g.packsJson, t.query.mode === "identity" ? "identity" : void 0);
            i = g, X("t2_start_done");
            const w = (r.isNative() ? De() : Promise.resolve()).then((v) => (X("t3_candidates_loaded"), v));
            await se(p), await gn(), X("t4_first_frame"), await w, Je(s, `\u5F00\u5C40 ${l.value} \u4EBA seed=${c.value}${g.contentLock.length ? ` \xB7 \u5185\u5BB9\u9501 ${g.contentLock.length} \u5305` : ""}`);
          } catch (g) {
            throw s.connected = false, Z(), g;
          }
        });
      }
      async function Qe() {
        await $(async () => {
          if (M.value) throw new Error("\u8054\u673A\u6A21\u5F0F\u8BF7\u4F7F\u7528\u670D\u52A1\u7AEF\u5B58\u6863");
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
          }), Je(s, `\u5DF2\u5B58\u6863 ${p}\uFF08${g.length / 2} \u5B57\u8282\uFF09`), m.value = `\u5DF2\u5B58\u6863 \xB7 ${p}`;
        });
      }
      async function Ue() {
        await $(async () => {
          if (M.value) throw new Error("\u8054\u673A\u6A21\u5F0F\u4E0D\u652F\u6301\u672C\u5730\u8BFB\u6863");
          if (!r.isNative()) throw new Error("\u8BFB\u6863\u4EC5\u539F\u751F\u540E\u7AEF\u652F\u6301");
          const p = Ka()[0];
          if (!p) throw new Error("\u65E0\u672C\u5730\u5B58\u6863");
          const w = Md(p), v = await r.restoreHex(p.hex, w.packsJson);
          i = w, l.value = p.players, c.value = p.seed, E.value = po(E.value, E.value.seat), await se(v), await De(), Je(s, `\u8BFB\u6863 ${p.id} \xB7 rev ${p.revision}`);
        });
      }
      async function _t(g) {
        o.onEnvelope((w) => {
          se(w);
        }), o.onClose(() => {
          m.value = "\u8FDE\u63A5\u5DF2\u65AD\u5F00\uFF08\u670D\u52A1\u7AEF\u4E0D\u53EF\u8FBE\uFF09", s.connected = false, Je(s, "\u8FDE\u63A5\u5DF2\u65AD\u5F00\uFF1A\u670D\u52A1\u7AEF\u4E0D\u53EF\u8FBE\u6216\u5DF2\u505C\u6B62");
        }), o.onReconnect((w) => {
          w === "reconnecting" ? (m.value = "\u8FDE\u63A5\u5DF2\u65AD\u5F00\uFF0C\u6B63\u5728\u91CD\u8FDE\u2026", s.connected = false) : w === "reconnected" ? (m.value = "\u5DF2\u91CD\u8FDE\uFF08\u7B49\u5F85\u623F\u95F4\u72B6\u6001\uFF09", s.connected = false, o.rejoinRoom().then((v) => (a.value = v.playerId, P.value = v.roomId, ge(g, v), s.connected = true, m.value = `\u623F\u95F4 #${v.roomId} \xB7 \u5DF2\u91CD\u8FDE`, se(v.envelope))).catch((v) => {
            s.connected = false;
            const T = v instanceof Error ? v.message : String(v);
            m.value = `\u91CD\u8FDE\u540E\u56DE\u623F\u5931\u8D25\uFF1A${T}`, Je(s, `\u91CD\u8FDE\u540E\u56DE\u623F\u5931\u8D25\uFF1A${T}`);
          })) : (m.value = "\u91CD\u8FDE\u5931\u8D25\uFF08\u670D\u52A1\u7AEF\u4E0D\u53EF\u8FBE\uFF09", s.connected = false);
        }), o.onRoster((w) => {
          _.value = w, P.value = w.room_id;
          const v = w.started && !R.value;
          R.value = w.started, m.value = w.started ? `\u623F\u95F4 #${w.room_id} \xB7 \u5DF2\u5F00\u5C40` : `\u623F\u95F4 #${w.room_id} \xB7 \u7B49\u5F85\u5165\u5EA7 ${w.seats.length}/${l.value}`, v && o.requestSnapshot().catch(() => {
          });
        }), m.value = `\u8FDE\u63A5 ${g.wsUrl}\u2026`, await o.connect(g.wsUrl, g.displayName);
        let p;
        if (g.action === "create") p = await o.createRoom(g.players, g.seed, g.password);
        else if (g.action === "spectate") p = await o.spectate(g.roomId);
        else {
          const w = localStorage.getItem(fo(g.wsUrl, g.roomId));
          p = await o.joinRoom(g.roomId, g.password, w);
        }
        P.value = p.roomId, a.value = p.playerId, l.value = p.view.seats.length, ge(g, p), s.connected = true, await se(p.envelope), Je(s, p.role === "Spectator" ? `\u65C1\u89C2\u623F\u95F4 #${p.roomId}` : `\u52A0\u5165\u623F\u95F4 #${p.roomId} \xB7 \u5EA7\u4F4D ${p.seat + 1}`);
      }
      function rt() {
        o.cancelPendingCommands(), m.value = "\u5DF2\u53D6\u6D88\u5F85\u786E\u8BA4\u547D\u4EE4\uFF08\u670D\u52A1\u7AEF\u53EF\u80FD\u5DF2\u63D0\u4EA4\uFF0C\u4EE5\u6743\u5A01\u72B6\u6001\u4E3A\u51C6\uFF09", Je(s, "\u5DF2\u53D6\u6D88\u5F85\u786E\u8BA4\u547D\u4EE4\uFF1B\u7ED3\u679C\u53EF\u80FD\u5DF2\u63D0\u4EA4\uFF0C\u7B49\u5F85/\u5237\u65B0\u540E\u4EE5\u670D\u52A1\u7AEF\u72B6\u6001\u4E3A\u51C6");
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
        return M.value ? o : r;
      }
      async function Te() {
        await $(async () => {
          const g = await ee().endPhase(a.value);
          await re(g);
        });
      }
      async function O() {
        await $(async () => {
          if (M.value) throw new Error("\u8054\u673A\u6A21\u5F0F\u4E0D\u63D0\u4F9B\u5BA2\u6237\u7AEF\u81EA\u52A8\u6B65\u8FDB");
          const p = await r.autoStep();
          return await re(p), true;
        }) || S();
      }
      async function D(g, p) {
        await $(async () => {
          const w = await ee().playCard(a.value, g, p);
          await re(w), Je(s, `\u51FA\u724C #${g}${p.length > 0 ? ` \u2192 P${p[0]}` : ""}`);
        });
      }
      async function F(g, p, w) {
        await $(async () => {
          const v = await ee().activateSkill(a.value, g, p, w);
          await re(v), Je(s, `\u53D1\u52A8\u6280\u80FD #${g}${p.length > 0 ? ` \u2192 P${p[0]}` : ""}`);
        });
      }
      async function ie(g) {
        await $(async () => {
          const p = await ee().resolveChoice(a.value, g.ResolveChoice.choice_id, g.ResolveChoice.selections, g.ResolveChoice.cancelled);
          await re(p);
        });
      }
      const Ie = Q(() => s.legalActions.filter((g) => typeof g == "object" && "ResolveChoice" in g)), it = Q(() => s.connected && s.legalActions.includes("EndPhase"));
      function He() {
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
          if (await O(), f.value && !s.finished) {
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
      function W(g) {
        if (g.key === "?") {
          N.value = !N.value;
          return;
        }
        if (g.key === "Escape") {
          N.value = false;
          return;
        }
        s.busy || s.finished || !s.connected || (g.key === " " ? (g.preventDefault(), M.value || O()) : g.key === "e" || g.key === "E" ? it.value && Te() : (g.key === "s" || g.key === "S") && (s.skipAnims = !s.skipAnims));
      }
      function Z() {
        S(), r.dispose(), o.dispose(), i = void 0, Xi(false), co(false);
      }
      Xo(() => {
        window.addEventListener("keydown", W), window.addEventListener("pagehide", Z), Le().catch((g) => {
          const p = g instanceof Error ? g.message : String(g);
          m.value = `\u8FDE\u63A5\u5931\u8D25\uFF1A${p}`, Je(s, `\u8FDE\u63A5\u5931\u8D25\uFF1A${p}`), s.connected = false;
        });
      }), Wr(() => {
        window.removeEventListener("keydown", W), window.removeEventListener("pagehide", Z), Z();
      }), Kt(() => s.speed, () => {
      });
      const d = {
        Prepare: "\u51C6\u5907",
        Draw: "\u6478\u724C",
        Play: "\u51FA\u724C",
        Discard: "\u5F03\u724C",
        Judge: "\u5224\u5B9A",
        End: "\u7ED3\u675F"
      }, h = Q(() => Kd(s.view, s.lastEvents ?? [], a.value));
      function y() {
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
        return x(), C("div", Dp, [
          N.value ? (x(), C("section", jp, [
            ...p[4] || (p[4] = [
              Uc('<h2 data-v-6ac1c19d>\u2328 \u5FEB\u6377\u952E</h2><dl class="hotkey-list" data-v-6ac1c19d><dt data-v-6ac1c19d><kbd data-v-6ac1c19d>Space</kbd></dt><dd data-v-6ac1c19d>\u81EA\u52A8\u4E00\u6B65</dd><dt data-v-6ac1c19d><kbd data-v-6ac1c19d>E</kbd></dt><dd data-v-6ac1c19d>\u7ED3\u675F\u9636\u6BB5</dd><dt data-v-6ac1c19d><kbd data-v-6ac1c19d>S</kbd></dt><dd data-v-6ac1c19d>\u5207\u6362\u8DF3\u8FC7\u52A8\u753B</dd><dt data-v-6ac1c19d><kbd data-v-6ac1c19d>?</kbd></dt><dd data-v-6ac1c19d>\u663E\u793A/\u9690\u85CF\u6B64\u9762\u677F</dd><dt data-v-6ac1c19d><kbd data-v-6ac1c19d>Esc</kbd></dt><dd data-v-6ac1c19d>\u5173\u95ED\u6B64\u9762\u677F</dd></dl>', 2)
            ])
          ])) : z("", true),
          q(s).finished && h.value ? (x(), C("section", {
            key: 1,
            class: We([
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
              k("span", Up, I(h.value.isSelfWin === null ? "\u2691" : h.value.isSelfWin ? "\u{1F389}" : "\u{1F480}"), 1),
              k("div", null, [
                k("h2", Hp, I(h.value.label), 1),
                k("p", Fp, "rev " + I(q(s).revision), 1)
              ])
            ]),
            k("button", {
              class: "primary",
              type: "button",
              onClick: p[0] || (p[0] = (v) => M.value ? y() : Le())
            }, I(M.value ? "\u8FD4\u56DE\u5927\u5385" : "\u518D\u6765\u4E00\u5C40"), 1)
          ], 2)) : z("", true),
          b.value ? (x(), C("section", Wp, [
            k("span", qp, "\u25BA \u7B2C " + I(b.value.round) + " \u8F6E \xB7 P" + I(b.value.currentPlayer), 1),
            k("span", {
              class: "phase-badge",
              "data-phase": b.value.phase
            }, I(b.value.phaseLabel), 9, Vp)
          ])) : z("", true),
          k("section", Bp, [
            k("div", Kp, [
              M.value ? (x(), C("span", Jp, "\u8054\u673A\u623F\u95F4 #" + I(P.value || "\u2026"), 1)) : (x(), C("button", {
                key: 0,
                class: "primary",
                type: "button",
                disabled: q(s).busy,
                onClick: Le
              }, " \u91CD\u65B0\u5F00\u5C40 ", 8, Gp)),
              k("button", {
                type: "button",
                disabled: q(s).busy || q(s).finished || L.value || M.value && !R.value || !it.value,
                onClick: Te
              }, " \u7ED3\u675F\u9636\u6BB5 ", 8, zp),
              M.value ? z("", true) : (x(), C("button", {
                key: 2,
                type: "button",
                disabled: q(s).busy || q(s).finished,
                onClick: O
              }, " \u81EA\u52A8\u4E00\u6B65 ", 8, Yp)),
              M.value ? z("", true) : (x(), C("button", {
                key: 3,
                type: "button",
                disabled: q(s).busy,
                onClick: He
              }, I(f.value ? "\u505C\u6B62\u81EA\u52A8" : "\u8FDE\u7EED\u81EA\u52A8"), 9, Qp)),
              M.value ? (x(), C("button", {
                key: 4,
                type: "button",
                disabled: !q(s).busy,
                onClick: rt
              }, " \u53D6\u6D88\u5F85\u786E\u8BA4 ", 8, Xp)) : z("", true),
              M.value ? z("", true) : (x(), C("button", {
                key: 5,
                type: "button",
                disabled: q(s).busy || !q(s).connected,
                onClick: Qe
              }, " \u5B58\u6863 ", 8, Zp)),
              M.value ? z("", true) : (x(), C("button", {
                key: 6,
                type: "button",
                disabled: q(s).busy || !q(s).connected,
                onClick: Ue
              }, " \u8BFB\u6863 ", 8, eg))
            ]),
            k("div", tg, [
              k("label", null, [
                p[6] || (p[6] = Ct(" \u500D\u901F ", -1)),
                qs(k("select", {
                  "onUpdate:modelValue": p[1] || (p[1] = (v) => q(s).speed = v),
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
                    q(s).speed,
                    void 0,
                    {
                      number: true
                    }
                  ]
                ])
              ]),
              k("label", ng, [
                qs(k("input", {
                  "onUpdate:modelValue": p[2] || (p[2] = (v) => q(s).skipAnims = v),
                  type: "checkbox"
                }, null, 512), [
                  [
                    _u,
                    q(s).skipAnims
                  ]
                ]),
                p[7] || (p[7] = Ct(" \u8DF3\u8FC7\u52A8\u753B ", -1))
              ]),
              k("span", null, I(L.value ? "\u516C\u5171\u65C1\u89C2\u6295\u5F71" : `\u89C2\u5BDF\u8005 P${a.value}`), 1)
            ]),
            k("p", sg, I(m.value), 1),
            q(s).error ? (x(), C("p", rg, I(q(s).error), 1)) : z("", true)
          ]),
          H.value ? (x(), Br(Mp, {
            key: 3,
            state: E.value,
            busy: q(s).busy,
            onChoose: je,
            onReload: De
          }, null, 8, [
            "state",
            "busy"
          ])) : z("", true),
          M.value && _.value ? (x(), C("section", ig, [
            k("div", og, [
              k("h2", null, "\u623F\u95F4 #" + I(_.value.room_id), 1),
              k("span", {
                class: We(_.value.started ? "started" : "waiting")
              }, I(_.value.started ? "\u5BF9\u5C40\u8FDB\u884C\u4E2D" : "\u7B49\u5F85\u73A9\u5BB6"), 3)
            ]),
            k("div", ag, [
              (x(true), C(de, null, Ce(_.value.seats, (v) => (x(), C("span", {
                key: v.seat,
                class: "online-seat"
              }, [
                Ct(I(v.seat + 1) + " \xB7 " + I(v.display_name) + " ", 1),
                k("i", {
                  class: We(v.connected ? "connected" : "offline")
                }, I(v.connected ? "\u5728\u7EBF" : "\u79BB\u7EBF"), 3)
              ]))), 128))
            ])
          ])) : z("", true),
          q(s).choice ? (x(), C("section", lg, [
            k("h2", null, I(q(yr)(q(s).choice)), 1),
            k("p", cg, " choice #" + I(q(s).choice.choice_id) + " \xB7 \u73A9\u5BB6 " + I(q(s).choice.observer) + " \xB7 " + I(q(ys)(q(s).choice)), 1),
            Ie.value.length ? (x(), C("div", ug, [
              (x(true), C(de, null, Ce(Ie.value, (v, T) => (x(), C("button", {
                key: T,
                type: "button",
                disabled: q(s).busy || !q(s).connected || L.value,
                onClick: (A) => ie(v)
              }, I(v.ResolveChoice.cancelled ? "\u53D6\u6D88" : `\u9009\u62E9 ${v.ResolveChoice.selections.join(", ") || "\u7A7A"}`), 9, fg))), 128))
            ])) : (x(), C("p", dg, "\u5F53\u524D\u89C2\u5BDF\u8005\u65E0\u6743\u5904\u7406\u6B64\u9009\u62E9\u3002"))
          ])) : z("", true),
          q(s).view ? (x(), C("section", hg, [
            p[8] || (p[8] = k("h2", null, "\u73A9\u5BB6\u6982\u51B5", -1)),
            k("div", pg, [
              (x(true), C(de, null, Ce(q(s).view.seats, (v) => (x(), C("div", {
                key: v.player,
                class: We([
                  "roster-entry",
                  {
                    dead: !v.alive,
                    self: v.player === a.value
                  }
                ])
              }, [
                k("span", gg, "P" + I(v.player) + I(v.turned_over ? " \xB7 \u7FFB\u9762" : ""), 1),
                k("span", {
                  class: "roster-hp",
                  style: Gt({
                    color: v.alive ? v.hp <= 0 ? "var(--color-warning)" : "inherit" : "var(--color-negative)"
                  })
                }, I(v.alive ? v.hp <= 0 ? "\u26A0" : "" : "\u2717") + " " + I(v.hp) + "/" + I(v.max_hp), 5),
                k("span", mg, I(v.hand_count) + "\u{1F0A0}", 1),
                v.identity ? (x(), C("span", {
                  key: 0,
                  class: "roster-id",
                  "data-id": v.identity
                }, I(v.identity === "Lord" ? "\u4E3B" : v.identity === "Loyalist" ? "\u5FE0" : v.identity === "Rebel" ? "\u53CD" : "\u5978"), 9, vg)) : (x(), C("span", yg, "?"))
              ], 2))), 128))
            ])
          ])) : z("", true),
          k("section", _g, [
            Ee(yp, {
              view: q(s).view,
              "self-id": a.value,
              "legal-actions": q(s).connected && !q(s).busy ? q(s).legalActions : [],
              onPlayCard: D,
              onActivateSkill: F
            }, null, 8, [
              "view",
              "self-id",
              "legal-actions"
            ]),
            q(s).effects.length ? (x(), C("div", bg, [
              (x(true), C(de, null, Ce(q(s).effects, (v) => (x(), C("div", {
                key: v.id,
                class: We([
                  "effect-popup",
                  v.type
                ])
              }, [
                k("span", wg, I(v.type === "damage" ? "\u2694" : v.type === "heal" ? "\u271A" : v.type === "draw" ? "\u{1F0A0}" : v.type === "play" ? "\u25B8" : v.type === "dying" ? "\u26A0" : "\u2717"), 1),
                k("span", Sg, I(v.text), 1),
                v.player != null ? (x(), C("span", kg, "P" + I(v.player), 1)) : z("", true)
              ], 2))), 128))
            ])) : z("", true)
          ]),
          q(s).view ? (x(), C("section", Eg, [
            k("div", Ag, [
              k("div", xg, [
                p[9] || (p[9] = k("span", {
                  class: "stat-label muted"
                }, "Revision", -1)),
                k("span", Cg, I(q(s).revision), 1)
              ]),
              k("div", Rg, [
                p[10] || (p[10] = k("span", {
                  class: "stat-label muted"
                }, "State Hash", -1)),
                k("span", Pg, I(q(s).hash.slice(0, 16)) + "\u2026", 1)
              ]),
              k("div", Tg, [
                p[11] || (p[11] = k("span", {
                  class: "stat-label muted"
                }, "\u5DF2\u77E5\u5361\u724C", -1)),
                k("span", Ig, I(q(s).view.known_cards.length), 1)
              ]),
              k("div", Og, [
                p[12] || (p[12] = k("span", {
                  class: "stat-label muted"
                }, "\u5B58\u6D3B\u73A9\u5BB6", -1)),
                k("span", Ng, I(q(s).view.seats.filter((v) => v.alive).length) + " / " + I(q(s).view.seats.length), 1)
              ])
            ])
          ])) : z("", true),
          q(s).view ? (x(), C("section", $g, [
            p[13] || (p[13] = k("h2", null, "\u5EA7\u4F4D\u72B6\u6001", -1)),
            k("div", Mg, [
              (x(true), C(de, null, Ce(q(s).view.seats, (v) => (x(), C("div", {
                key: "ss-" + v.player,
                class: We([
                  "ss-entry",
                  {
                    "ss-dead": !v.alive,
                    "ss-self": v.player === a.value
                  }
                ])
              }, [
                k("div", Dg, [
                  k("span", jg, "P" + I(v.player), 1),
                  v.player === a.value ? (x(), C("span", Lg, "\u4F60")) : z("", true)
                ]),
                k("div", Ug, [
                  k("div", {
                    class: "ss-hp-fill",
                    style: Gt({
                      width: Math.max(Math.min(v.hp / v.max_hp * 100, 100), 0) + "%",
                      background: v.alive ? v.hp <= 0 ? "var(--color-warning)" : "var(--color-positive)" : "var(--color-negative)"
                    })
                  }, null, 4)
                ]),
                k("div", Hg, [
                  k("span", Fg, I(v.alive ? v.hp <= 0 ? "\u26A0" : "" : "\u2717"), 1),
                  Ct(" " + I(Math.max(v.hp, 0)) + "/" + I(v.max_hp), 1)
                ]),
                k("div", Wg, [
                  k("span", null, I(v.hand_count) + "\u{1F0A0}", 1),
                  v.identity ? (x(), C("span", {
                    key: 0,
                    class: "ss-id",
                    "data-id": v.identity
                  }, I(v.identity === "Lord" ? "\u4E3B" : v.identity === "Loyalist" ? "\u5FE0" : v.identity === "Rebel" ? "\u53CD" : "\u5978"), 9, qg)) : (x(), C("span", Vg, "?"))
                ])
              ], 2))), 128))
            ])
          ])) : z("", true),
          k("section", Bg, [
            k("div", Kg, [
              p[14] || (p[14] = k("h2", null, "\u6218\u6597\u65E5\u5FD7", -1)),
              q(s).log.length > 0 ? (x(), C("button", {
                key: 0,
                type: "button",
                class: "clear-btn",
                onClick: p[3] || (p[3] = (v) => q(s).log = []),
                "aria-label": "\u6E05\u7A7A\u65E5\u5FD7"
              }, "\u6E05\u7A7A")) : z("", true)
            ]),
            p[15] || (p[15] = k("p", {
              class: "muted tiny"
            }, " \u65E5\u5FD7\u6765\u81EA DomainEvent \u6458\u8981\uFF0C\u4E0D\u662F\u72B6\u6001\u771F\u76F8\u6E90\u3002\u771F\u76F8\u6E90\u4E3A Command Log + \u6295\u5F71\uFF08\xA722\uFF09\u3002 ", -1)),
            qs((x(), C("ol", Gg, [
              (x(true), C(de, null, Ce(q(s).log.slice().reverse(), (v) => (x(), C("li", {
                key: v.id,
                class: We([
                  "log-entry",
                  U(v.text)
                ])
              }, [
                k("span", Jg, "rev" + I(v.at), 1),
                k("span", zg, I(v.text), 1)
              ], 2))), 128))
            ])), [
              [
                w
              ]
            ]),
            q(s).log.length === 0 ? (x(), C("p", Yg, "\u6682\u65E0\u4E8B\u4EF6")) : z("", true)
          ])
        ]);
      };
    }
  }), go = mn(Qg, [
    [
      "__scopeId",
      "data-v-6ac1c19d"
    ]
  ]);
  function Ve(e, t) {
    if (!e) throw new Error(t);
  }
  const el = Ds.prototype.start;
  let Ot, tl, we, Tn = 1;
  const Qr = [];
  Ds.prototype.start = async function() {
    return Qr.push(this), await el.call(this, Ot.seats.length, 0, Tn, Ot.packsJson), we = await this.restoreHex(Ot.archiveHex, Ot.packsJson), Ot.legacy && (we = {
      ...we,
      view: {
        ...we.view,
        seats: we.view.seats.map(({ generals: e, ...t }) => t)
      },
      choice: we.choice ? (({ prompt: e, native_kind: t, ...n }) => n)(we.choice) : null
    }), Ot.extensionPrompt && we.choice && (we = {
      ...we,
      choice: {
        ...we.choice,
        prompt: Ot.extensionPrompt
      }
    }), tl(), we;
  };
  const nr = async (e) => {
    for (let t = 0; t < 300; t++) {
      if (e()) return;
      await new Promise((n) => setTimeout(n, 20));
    }
    throw new Error("UI did not render expected state");
  };
  async function Xg() {
    var _a2, _b, _c2, _d2, _e, _f2, _g2;
    history.replaceState(null, "", jt() ? "/?engineTransport=native" : "/?engineTransport=wasm"), localStorage.setItem("noname-next-skip-anims", "1");
    const e = await fetch("/metadata-fixtures.json").then((r) => r.json()), t = e.find((r) => r.name === "respond_card");
    e.push({
      ...t,
      name: "legacy-choice-flow",
      legacy: true
    }, {
      ...t,
      name: "extension-prompt-flow",
      extensionPrompt: "<img src=x onerror=alert(1)>constructor"
    });
    const n = /* @__PURE__ */ new Set(), s = [];
    for (const r of e) {
      Ot = r, Tn = r.seats.findIndex((c) => c.choice !== null), Tn < 0 && (Tn = 1);
      const i = new Promise((c) => {
        tl = c;
      }), o = document.createElement("div");
      document.body.append(o);
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
      const l = Ni(go);
      l.use(a), l.mount(o);
      try {
        await i;
        const c = we.view.seats.flatMap((P) => P.generals ?? [
          null,
          null
        ]);
        await nr(() => o.querySelectorAll(".general-slot").length === c.length);
        const u = [
          ...o.querySelectorAll(".general-slot")
        ];
        for (const [P, R] of c.entries()) {
          const M = u[P].textContent ?? "";
          R ? (Ve(M.includes(R.name), `${r.name}: visible general missing`), Ve(M.includes(R.revealed ? "\u660E\u7F6E" : "\u6697\u7F6E"), `${r.name}: reveal flag missing`)) : Ve(M.includes(we.view.seats[Math.floor(P / 2)].generals === void 0 && P % 2 === 0 ? "#" : "\u6697\u5C06\uFF0F\u672A\u914D\u7F6E"), `${r.name}: hidden slot was disclosed`);
        }
        if (we.choice) {
          const P = we.choice;
          if (P.native_kind && n.add(P.native_kind), await nr(() => {
            var _a3, _b2;
            return !!((_b2 = (_a3 = o.querySelector(".choice h2")) == null ? void 0 : _a3.textContent) == null ? void 0 : _b2.includes(yr(P)));
          }), Ve((_b = (_a2 = o.querySelector(".choice")) == null ? void 0 : _a2.textContent) == null ? void 0 : _b.includes(ys(P)), `${r.name}: readable kind missing`), P.prompt && Ve(!((_d2 = (_c2 = o.querySelector(".choice")) == null ? void 0 : _c2.textContent) == null ? void 0 : _d2.includes(P.prompt)), `${r.name}: internal prompt displayed`), Ve(o.querySelector(".choice img[onerror]") === null, `${r.name}: extension prompt executed as markup`), Tn === 1 && [
            "confirm",
            "select_players",
            "select_cards",
            "respond_card",
            "pick_tag-fanjian"
          ].includes(r.name)) {
            const R = o.querySelector(".choice button");
            Ve(R && !R.disabled, `${r.name}: no real answer control`), R.click(), await nr(() => {
              var _a3, _b2, _c3;
              return !((_b2 = (_a3 = o.querySelector(".choice h2")) == null ? void 0 : _a3.textContent) == null ? void 0 : _b2.includes(yr(P))) || ((_c3 = o.textContent) == null ? void 0 : _c3.includes("rev " + (we.revision + 1))) === true;
            }), Ve(!((_e = o.textContent) == null ? void 0 : _e.includes("WrongPlayer")), `${r.name}: UI answered as wrong seat`);
          }
        }
        const f = Qr.at(-1), m = await f.saveHex(), _ = await f.restoreHex(m, r.packsJson);
        Ve(JSON.stringify(_.view.seats.map((P) => P.generals)) === JSON.stringify((await f.observer()).seats.map((P) => P.generals)), `${r.name}: restored metadata differs`), s.push({
          name: r.name,
          slots: u.length,
          prompt: ((_f2 = we.choice) == null ? void 0 : _f2.prompt) ?? null,
          transport: f.getTransportKind(),
          restored: true
        });
      } finally {
        l.unmount(), o.remove();
      }
    }
    Ve(n.size === 5, "not all five real choice kinds reached BattleView");
    for (const [r, i] of [
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
      const o = document.createElement("div");
      document.body.append(o);
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
          ...i
        },
        self: true,
        identityLabel: "\u672A\u77E5",
        isTargetMode: false,
        isSelectableTarget: false,
        isDropTarget: false,
        equipCards: []
      }, l = Ni(Za, a);
      l.mount(o), await gn(), Ve((_g2 = o.textContent) == null ? void 0 : _g2.includes(r === "old-payload" ? "#17" : "<img src=x onerror=alert(1)>"), `${r}: fallback missing`), Ve(o.querySelector("img[onerror]") === null, `${r}: untrusted markup`), Ve([
        ...o.querySelectorAll("img")
      ].every((c) => !c.src.includes("onerror")), `${r}: untrusted path`), l.unmount(), o.remove();
    }
    return {
      passed: true,
      cases: s,
      kinds: [
        ...n
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
    Ds.prototype.start = el;
    for (const e of Qr) e.dispose();
  }
})();
