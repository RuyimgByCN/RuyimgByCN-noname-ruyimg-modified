import { a as ht, c as ai, b as Ue, d as En, l as Zn, e as en, f as ea, g as gt, h as ti, j as na, n as aa, k as ii, m as si, o as oi, p as un, q as ft, r as ri, s as li, t as ui, u as pa, v as z, w as ci, x as di, y as se, z as Ge, A as ta, B as _t, C as L, F as Be, G as I, H as Nn, I as g, J as o, K as pe, L as De, M as h, N as W, O as gn, P as mi, Q as f, R as S, S as Z, T as Q, U as Y, V as ia, W as pi, X as hi, Y as gi, Z as fi, _ as _i, $ as oe, a0 as vt, a1 as mn, a2 as sa, a3 as yt, a4 as vi, a5 as yi, E as oa, a6 as ha, a7 as ki, a8 as wi, a9 as bi } from "./client-BZROKSmU.js";
/**
* @vue/runtime-dom v3.5.39
* (c) 2018-present Yuxi (Evan) You and Vue contributors
* @license MIT
**/
let Un;
const ga = typeof window < "u" && window.trustedTypes;
if (ga) try {
  Un = ga.createPolicy("vue", { createHTML: (e) => e });
} catch {
}
const kt = Un ? (e) => Un.createHTML(e) : (e) => e, xi = "http://www.w3.org/2000/svg", $i = "http://www.w3.org/1998/Math/MathML", we = typeof document < "u" ? document : null, fa = we && we.createElement("template"), ji = { insert: (e, n, a) => {
  n.insertBefore(e, a || null);
}, remove: (e) => {
  const n = e.parentNode;
  n && n.removeChild(e);
}, createElement: (e, n, a, t) => {
  const i = n === "svg" ? we.createElementNS(xi, e) : n === "mathml" ? we.createElementNS($i, e) : a ? we.createElement(e, { is: a }) : we.createElement(e);
  return e === "select" && t && t.multiple != null && i.setAttribute("multiple", t.multiple), i;
}, createText: (e) => we.createTextNode(e), createComment: (e) => we.createComment(e), setText: (e, n) => {
  e.nodeValue = n;
}, setElementText: (e, n) => {
  e.textContent = n;
}, parentNode: (e) => e.parentNode, nextSibling: (e) => e.nextSibling, querySelector: (e) => we.querySelector(e), setScopeId(e, n) {
  e.setAttribute(n, "");
}, insertStaticContent(e, n, a, t, i, s) {
  const l = a ? a.previousSibling : n.lastChild;
  if (i && (i === s || i.nextSibling)) for (; n.insertBefore(i.cloneNode(true), a), !(i === s || !(i = i.nextSibling)); ) ;
  else {
    fa.innerHTML = kt(t === "svg" ? `<svg>${e}</svg>` : t === "mathml" ? `<math>${e}</math>` : e);
    const r = fa.content;
    if (t === "svg" || t === "mathml") {
      const u = r.firstChild;
      for (; u.firstChild; ) r.appendChild(u.firstChild);
      r.removeChild(u);
    }
    n.insertBefore(r, a);
  }
  return [l ? l.nextSibling : n.firstChild, a ? a.previousSibling : n.lastChild];
} }, Si = Symbol("_vtc");
function Ei(e, n, a) {
  const t = e[Si];
  t && (n = (n ? [n, ...t] : [...t]).join(" ")), n == null ? e.removeAttribute("class") : a ? e.setAttribute("class", n) : e.className = n;
}
const _a = Symbol("_vod"), zi = Symbol("_vsh"), Ai = Symbol(""), Ci = /(?:^|;)\s*display\s*:/;
function Ri(e, n, a) {
  const t = e.style, i = Ue(a);
  let s = false;
  if (a && !i) {
    if (n) if (Ue(n)) for (const l of n.split(";")) {
      const r = l.slice(0, l.indexOf(":")).trim();
      a[r] == null && sn(t, r, "");
    }
    else for (const l in n) a[l] == null && sn(t, l, "");
    for (const l in a) {
      l === "display" && (s = true);
      const r = a[l];
      r != null ? Pi(e, l, !Ue(n) && n ? n[l] : void 0, r) || sn(t, l, r) : sn(t, l, "");
    }
  } else if (i) {
    if (n !== a) {
      const l = t[Ai];
      l && (a += ";" + l), t.cssText = a, s = Ci.test(a);
    }
  } else n && e.removeAttribute("style");
  _a in e && (e[_a] = s ? t.display : "", e[zi] && (t.display = "none"));
}
const va = /\s*!important$/;
function sn(e, n, a) {
  if (en(a)) a.forEach((t) => sn(e, n, t));
  else if (a == null && (a = ""), n.startsWith("--")) e.setProperty(n, a);
  else {
    const t = Ni(e, n);
    va.test(a) ? e.setProperty(na(t), a.replace(va, ""), "important") : e[t] = a;
  }
}
const ya = ["Webkit", "Moz", "ms"], Pn = {};
function Ni(e, n) {
  const a = Pn[n];
  if (a) return a;
  let t = un(n);
  if (t !== "filter" && t in e) return Pn[n] = t;
  t = ui(t);
  for (let i = 0; i < ya.length; i++) {
    const s = ya[i] + t;
    if (s in e) return Pn[n] = s;
  }
  return n;
}
function Pi(e, n, a, t) {
  return e.tagName === "TEXTAREA" && (n === "width" || n === "height") && Ue(t) && a === t;
}
const ka = "http://www.w3.org/1999/xlink";
function wa(e, n, a, t, i, s = li(n)) {
  t && n.startsWith("xlink:") ? a == null ? e.removeAttributeNS(ka, n.slice(6, n.length)) : e.setAttributeNS(ka, n, a) : a == null || s && !ft(a) ? e.removeAttribute(n) : e.setAttribute(n, s ? "" : ri(a) ? String(a) : a);
}
function ba(e, n, a, t, i) {
  if (n === "innerHTML" || n === "textContent") {
    a != null && (e[n] = n === "innerHTML" ? kt(a) : a);
    return;
  }
  const s = e.tagName;
  if (n === "value" && s !== "PROGRESS" && !s.includes("-")) {
    const r = s === "OPTION" ? e.getAttribute("value") || "" : e.value, u = a == null ? e.type === "checkbox" ? "on" : "" : String(a);
    (r !== u || !("_value" in e)) && (e.value = u), a == null && e.removeAttribute(n), e._value = a;
    return;
  }
  let l = false;
  if (a === "" || a == null) {
    const r = typeof e[n];
    r === "boolean" ? a = ft(a) : a == null && r === "string" ? (a = "", l = true) : r === "number" && (a = 0, l = true);
  }
  try {
    e[n] = a;
  } catch {
  }
  l && e.removeAttribute(i || n);
}
function Ce(e, n, a, t) {
  e.addEventListener(n, a, t);
}
function qi(e, n, a, t) {
  e.removeEventListener(n, a, t);
}
const xa = Symbol("_vei");
function Ii(e, n, a, t, i = null) {
  const s = e[xa] || (e[xa] = {}), l = s[n];
  if (t && l) l.value = t;
  else {
    const [r, u] = Li(n);
    if (t) {
      const c = s[n] = Ui(t, i);
      Ce(e, r, c, u);
    } else l && (qi(e, r, l, u), s[n] = void 0);
  }
}
const Oi = /(Once|Passive|Capture)$/, Ti = /^on:?(?:Once|Passive|Capture)$/;
function Li(e) {
  let n, a;
  for (; (a = e.match(Oi)) && !Ti.test(e); ) n || (n = {}), e = e.slice(0, e.length - a[1].length), n[a[1].toLowerCase()] = true;
  return [e[2] === ":" ? e.slice(3) : na(e.slice(2)), n];
}
let qn = 0;
const Di = Promise.resolve(), Mi = () => qn || (Di.then(() => qn = 0), qn = Date.now());
function Ui(e, n) {
  const a = (t) => {
    if (!t._vts) t._vts = Date.now();
    else if (t._vts <= a.attached) return;
    const i = a.value;
    if (en(i)) {
      const s = t.stopImmediatePropagation;
      t.stopImmediatePropagation = () => {
        s.call(t), t._stopped = true;
      };
      const l = i.slice(), r = [t];
      for (let u = 0; u < l.length && !t._stopped; u++) {
        const c = l[u];
        c && pa(c, n, 5, r);
      }
    } else pa(i, n, 5, [t]);
  };
  return a.value = e, a.attached = Mi(), a;
}
const $a = (e) => e.charCodeAt(0) === 111 && e.charCodeAt(1) === 110 && e.charCodeAt(2) > 96 && e.charCodeAt(2) < 123, Vi = (e, n, a, t, i, s) => {
  const l = i === "svg";
  n === "class" ? Ei(e, t, l) : n === "style" ? Ri(e, a, t) : si(n) ? oi(n) || Ii(e, n, a, t, s) : (n[0] === "." ? (n = n.slice(1), true) : n[0] === "^" ? (n = n.slice(1), false) : Gi(e, n, t, l)) ? (ba(e, n, t), !e.tagName.includes("-") && (n === "value" || n === "checked" || n === "selected") && wa(e, n, t, l, s, n !== "value")) : e._isVueCE && (Bi(e, n) || e._def.__asyncLoader && (/[A-Z]/.test(n) || !Ue(t))) ? ba(e, un(n), t, s, n) : (n === "true-value" ? e._trueValue = t : n === "false-value" && (e._falseValue = t), wa(e, n, t, l));
};
function Gi(e, n, a, t) {
  if (t) return !!(n === "innerHTML" || n === "textContent" || n in e && $a(n) && ht(a));
  if (n === "spellcheck" || n === "draggable" || n === "translate" || n === "autocorrect" || n === "sandbox" && e.tagName === "IFRAME" || n === "form" || n === "list" && e.tagName === "INPUT" || n === "type" && e.tagName === "TEXTAREA") return false;
  if (n === "width" || n === "height") {
    const i = e.tagName;
    if (i === "IMG" || i === "VIDEO" || i === "CANVAS" || i === "SOURCE") return false;
  }
  return $a(n) && Ue(a) ? false : n in e;
}
function Bi(e, n) {
  const a = e._def.props;
  if (!a) return false;
  const t = un(n);
  return Array.isArray(a) ? a.some((i) => un(i) === t) : Object.keys(a).some((i) => un(i) === t);
}
const Ye = (e) => {
  const n = e.props["onUpdate:modelValue"] || false;
  return en(n) ? (a) => ti(n, a) : n;
};
function Fi(e) {
  e.target.composing = true;
}
function ja(e) {
  const n = e.target;
  n.composing && (n.composing = false, n.dispatchEvent(new Event("input")));
}
const be = Symbol("_assign");
function Sa(e, n, a) {
  return n && (e = e.trim()), a && (e = Zn(e)), e;
}
const Re = { created(e, { modifiers: { lazy: n, trim: a, number: t } }, i) {
  e[be] = Ye(i);
  const s = t || i.props && i.props.type === "number";
  Ce(e, n ? "change" : "input", (l) => {
    l.target.composing || e[be](Sa(e.value, a, s));
  }), (a || s) && Ce(e, "change", () => {
    e.value = Sa(e.value, a, s);
  }), n || (Ce(e, "compositionstart", Fi), Ce(e, "compositionend", ja), Ce(e, "change", ja));
}, mounted(e, { value: n }) {
  e.value = n ?? "";
}, beforeUpdate(e, { value: n, oldValue: a, modifiers: { lazy: t, trim: i, number: s } }, l) {
  if (e[be] = Ye(l), e.composing) return;
  const r = (s || e.type === "number") && !/^0\d/.test(e.value) ? Zn(e.value) : e.value, u = n ?? "";
  if (r === u) return;
  const c = e.getRootNode();
  (c instanceof Document || c instanceof ShadowRoot) && c.activeElement === e && e.type !== "range" && (t && n === a || i && e.value.trim() === u) || (e.value = u);
} }, wn = { deep: true, created(e, n, a) {
  e[be] = Ye(a), Ce(e, "change", () => {
    const t = e._modelValue, i = pn(e), s = e.checked, l = e[be];
    if (en(t)) {
      const r = ea(t, i), u = r !== -1;
      if (s && !u) l(t.concat(i));
      else if (!s && u) {
        const c = [...t];
        c.splice(r, 1), l(c);
      }
    } else if (En(t)) {
      const r = new Set(t);
      s ? r.add(i) : r.delete(i), l(r);
    } else l(wt(e, s));
  });
}, mounted: Ea, beforeUpdate(e, n, a) {
  e[be] = Ye(a), Ea(e, n, a);
} };
function Ea(e, { value: n, oldValue: a }, t) {
  e._modelValue = n;
  let i;
  if (en(n)) i = ea(n, t.props.value) > -1;
  else if (En(n)) i = n.has(t.props.value);
  else {
    if (n === a) return;
    i = gt(n, wt(e, true));
  }
  e.checked !== i && (e.checked = i);
}
const _e = { deep: true, created(e, { value: n, modifiers: { number: a } }, t) {
  const i = En(n);
  Ce(e, "change", () => {
    const s = Array.prototype.filter.call(e.options, (l) => l.selected).map((l) => a ? Zn(pn(l)) : pn(l));
    e[be](e.multiple ? i ? new Set(s) : s : s[0]), e._assigning = true, aa(() => {
      e._assigning = false;
    });
  }), e[be] = Ye(t);
}, mounted(e, { value: n }) {
  za(e, n);
}, beforeUpdate(e, n, a) {
  e[be] = Ye(a);
}, updated(e, { value: n }) {
  e._assigning || za(e, n);
} };
function za(e, n) {
  const a = e.multiple, t = en(n);
  if (!(a && !t && !En(n))) {
    for (let i = 0, s = e.options.length; i < s; i++) {
      const l = e.options[i], r = pn(l);
      if (a) if (t) {
        const u = typeof r;
        u === "string" || u === "number" ? l.selected = n.some((c) => String(c) === String(r)) : l.selected = ea(n, r) > -1;
      } else l.selected = n.has(r);
      else if (gt(pn(l), n)) {
        e.selectedIndex !== i && (e.selectedIndex = i);
        return;
      }
    }
    !a && e.selectedIndex !== -1 && (e.selectedIndex = -1);
  }
}
function pn(e) {
  return "_value" in e ? e._value : e.value;
}
function wt(e, n) {
  const a = n ? "_trueValue" : "_falseValue";
  return a in e ? e[a] : n;
}
const Hi = ["ctrl", "shift", "alt", "meta"], Wi = { stop: (e) => e.stopPropagation(), prevent: (e) => e.preventDefault(), self: (e) => e.target !== e.currentTarget, ctrl: (e) => !e.ctrlKey, shift: (e) => !e.shiftKey, alt: (e) => !e.altKey, meta: (e) => !e.metaKey, left: (e) => "button" in e && e.button !== 0, middle: (e) => "button" in e && e.button !== 1, right: (e) => "button" in e && e.button !== 2, exact: (e, n) => Hi.some((a) => e[`${a}Key`] && !n.includes(a)) }, Vn = (e, n) => {
  if (!e) return e;
  const a = e._withMods || (e._withMods = {}), t = n.join(".");
  return a[t] || (a[t] = ((i, ...s) => {
    for (let l = 0; l < n.length; l++) {
      const r = Wi[n[l]];
      if (r && r(i, n)) return;
    }
    return e(i, ...s);
  }));
}, Ki = { esc: "escape", space: " ", up: "arrow-up", left: "arrow-left", right: "arrow-right", down: "arrow-down", delete: "backspace" }, Ji = (e, n) => {
  const a = e._withKeys || (e._withKeys = {}), t = n.join(".");
  return a[t] || (a[t] = ((i) => {
    if (!("key" in i)) return;
    const s = na(i.key);
    if (n.some((l) => l === s || Ki[l] === s)) return e(i);
  }));
}, Qi = ii({ patchProp: Vi }, ji);
let Aa;
function Yi() {
  return Aa || (Aa = ai(Qi));
}
const Xi = ((...e) => {
  const n = Yi().createApp(...e), { mount: a } = n;
  return n.mount = (t) => {
    const i = es(t);
    if (!i) return;
    const s = n._component;
    !ht(s) && !s.render && !s.template && (s.template = i.innerHTML), i.nodeType === 1 && (i.textContent = "");
    const l = a(i, false, Zi(i));
    return i instanceof Element && (i.removeAttribute("v-cloak"), i.setAttribute("data-v-app", "")), l;
  }, n;
});
function Zi(e) {
  if (e instanceof SVGElement) return "svg";
  if (typeof MathMLElement == "function" && e instanceof MathMLElement) return "mathml";
}
function es(e) {
  return Ue(e) ? document.querySelector(e) : e;
}
/*!
* vue-router v4.6.4
* (c) 2025 Eduardo San Martin Morote
* @license MIT
*/
const Je = typeof document < "u";
function bt(e) {
  return typeof e == "object" || "displayName" in e || "props" in e || "__vccOpts" in e;
}
function ns(e) {
  return e.__esModule || e[Symbol.toStringTag] === "Module" || e.default && bt(e.default);
}
const J = Object.assign;
function In(e, n) {
  const a = {};
  for (const t in n) {
    const i = n[t];
    a[t] = he(i) ? i.map(e) : e(i);
  }
  return a;
}
const cn = () => {
}, he = Array.isArray;
function Ca(e, n) {
  const a = {};
  for (const t in e) a[t] = t in n ? n[t] : e[t];
  return a;
}
const xt = /#/g, as = /&/g, ts = /\//g, is = /=/g, ss = /\?/g, $t = /\+/g, os = /%5B/g, rs = /%5D/g, jt = /%5E/g, ls = /%60/g, St = /%7B/g, us = /%7C/g, Et = /%7D/g, cs = /%20/g;
function ra(e) {
  return e == null ? "" : encodeURI("" + e).replace(us, "|").replace(os, "[").replace(rs, "]");
}
function ds(e) {
  return ra(e).replace(St, "{").replace(Et, "}").replace(jt, "^");
}
function Gn(e) {
  return ra(e).replace($t, "%2B").replace(cs, "+").replace(xt, "%23").replace(as, "%26").replace(ls, "`").replace(St, "{").replace(Et, "}").replace(jt, "^");
}
function ms(e) {
  return Gn(e).replace(is, "%3D");
}
function ps(e) {
  return ra(e).replace(xt, "%23").replace(ss, "%3F");
}
function hs(e) {
  return ps(e).replace(ts, "%2F");
}
function hn(e) {
  if (e == null) return null;
  try {
    return decodeURIComponent("" + e);
  } catch {
  }
  return "" + e;
}
const gs = /\/$/, fs = (e) => e.replace(gs, "");
function On(e, n, a = "/") {
  let t, i = {}, s = "", l = "";
  const r = n.indexOf("#");
  let u = n.indexOf("?");
  return u = r >= 0 && u > r ? -1 : u, u >= 0 && (t = n.slice(0, u), s = n.slice(u, r > 0 ? r : n.length), i = e(s.slice(1))), r >= 0 && (t = t || n.slice(0, r), l = n.slice(r, n.length)), t = ks(t ?? n, a), { fullPath: t + s + l, path: t, query: i, hash: hn(l) };
}
function _s(e, n) {
  const a = n.query ? e(n.query) : "";
  return n.path + (a && "?") + a + (n.hash || "");
}
function Ra(e, n) {
  return !n || !e.toLowerCase().startsWith(n.toLowerCase()) ? e : e.slice(n.length) || "/";
}
function vs(e, n, a) {
  const t = n.matched.length - 1, i = a.matched.length - 1;
  return t > -1 && t === i && Xe(n.matched[t], a.matched[i]) && zt(n.params, a.params) && e(n.query) === e(a.query) && n.hash === a.hash;
}
function Xe(e, n) {
  return (e.aliasOf || e) === (n.aliasOf || n);
}
function zt(e, n) {
  if (Object.keys(e).length !== Object.keys(n).length) return false;
  for (var a in e) if (!ys(e[a], n[a])) return false;
  return true;
}
function ys(e, n) {
  return he(e) ? Na(e, n) : he(n) ? Na(n, e) : (e == null ? void 0 : e.valueOf()) === (n == null ? void 0 : n.valueOf());
}
function Na(e, n) {
  return he(n) ? e.length === n.length && e.every((a, t) => a === n[t]) : e.length === 1 && e[0] === n;
}
function ks(e, n) {
  if (e.startsWith("/")) return e;
  if (!e) return n;
  const a = n.split("/"), t = e.split("/"), i = t[t.length - 1];
  (i === ".." || i === ".") && t.push("");
  let s = a.length - 1, l, r;
  for (l = 0; l < t.length; l++) if (r = t[l], r !== ".") if (r === "..") s > 1 && s--;
  else break;
  return a.slice(0, s).join("/") + "/" + t.slice(l).join("/");
}
const Se = { path: "/", name: void 0, params: {}, query: {}, hash: "", fullPath: "/", matched: [], meta: {}, redirectedFrom: void 0 };
let Bn = (function(e) {
  return e.pop = "pop", e.push = "push", e;
})({}), Tn = (function(e) {
  return e.back = "back", e.forward = "forward", e.unknown = "", e;
})({});
function ws(e) {
  if (!e) if (Je) {
    const n = document.querySelector("base");
    e = n && n.getAttribute("href") || "/", e = e.replace(/^\w+:\/\/[^\/]+/, "");
  } else e = "/";
  return e[0] !== "/" && e[0] !== "#" && (e = "/" + e), fs(e);
}
const bs = /^[^#]+#/;
function xs(e, n) {
  return e.replace(bs, "#") + n;
}
function $s(e, n) {
  const a = document.documentElement.getBoundingClientRect(), t = e.getBoundingClientRect();
  return { behavior: n.behavior, left: t.left - a.left - (n.left || 0), top: t.top - a.top - (n.top || 0) };
}
const zn = () => ({ left: window.scrollX, top: window.scrollY });
function js(e) {
  let n;
  if ("el" in e) {
    const a = e.el, t = typeof a == "string" && a.startsWith("#"), i = typeof a == "string" ? t ? document.getElementById(a.slice(1)) : document.querySelector(a) : a;
    if (!i) return;
    n = $s(i, e);
  } else n = e;
  "scrollBehavior" in document.documentElement.style ? window.scrollTo(n) : window.scrollTo(n.left != null ? n.left : window.scrollX, n.top != null ? n.top : window.scrollY);
}
function Pa(e, n) {
  return (history.state ? history.state.position - n : -1) + e;
}
const Fn = /* @__PURE__ */ new Map();
function Ss(e, n) {
  Fn.set(e, n);
}
function Es(e) {
  const n = Fn.get(e);
  return Fn.delete(e), n;
}
function zs(e) {
  return typeof e == "string" || e && typeof e == "object";
}
function At(e) {
  return typeof e == "string" || typeof e == "symbol";
}
let ne = (function(e) {
  return e[e.MATCHER_NOT_FOUND = 1] = "MATCHER_NOT_FOUND", e[e.NAVIGATION_GUARD_REDIRECT = 2] = "NAVIGATION_GUARD_REDIRECT", e[e.NAVIGATION_ABORTED = 4] = "NAVIGATION_ABORTED", e[e.NAVIGATION_CANCELLED = 8] = "NAVIGATION_CANCELLED", e[e.NAVIGATION_DUPLICATED = 16] = "NAVIGATION_DUPLICATED", e;
})({});
const Ct = Symbol("");
ne.MATCHER_NOT_FOUND + "", ne.NAVIGATION_GUARD_REDIRECT + "", ne.NAVIGATION_ABORTED + "", ne.NAVIGATION_CANCELLED + "", ne.NAVIGATION_DUPLICATED + "";
function Ze(e, n) {
  return J(new Error(), { type: e, [Ct]: true }, n);
}
function ke(e, n) {
  return e instanceof Error && Ct in e && (n == null || !!(e.type & n));
}
const As = ["params", "query", "hash"];
function Cs(e) {
  if (typeof e == "string") return e;
  if (e.path != null) return e.path;
  const n = {};
  for (const a of As) a in e && (n[a] = e[a]);
  return JSON.stringify(n, null, 2);
}
function Rs(e) {
  const n = {};
  if (e === "" || e === "?") return n;
  const a = (e[0] === "?" ? e.slice(1) : e).split("&");
  for (let t = 0; t < a.length; ++t) {
    const i = a[t].replace($t, " "), s = i.indexOf("="), l = hn(s < 0 ? i : i.slice(0, s)), r = s < 0 ? null : hn(i.slice(s + 1));
    if (l in n) {
      let u = n[l];
      he(u) || (u = n[l] = [u]), u.push(r);
    } else n[l] = r;
  }
  return n;
}
function qa(e) {
  let n = "";
  for (let a in e) {
    const t = e[a];
    if (a = ms(a), t == null) {
      t !== void 0 && (n += (n.length ? "&" : "") + a);
      continue;
    }
    (he(t) ? t.map((i) => i && Gn(i)) : [t && Gn(t)]).forEach((i) => {
      i !== void 0 && (n += (n.length ? "&" : "") + a, i != null && (n += "=" + i));
    });
  }
  return n;
}
function Ns(e) {
  const n = {};
  for (const a in e) {
    const t = e[a];
    t !== void 0 && (n[a] = he(t) ? t.map((i) => i == null ? null : "" + i) : t == null ? t : "" + t);
  }
  return n;
}
const Ps = Symbol(""), Ia = Symbol(""), An = Symbol(""), la = Symbol(""), Hn = Symbol("");
function an() {
  let e = [];
  function n(t) {
    return e.push(t), () => {
      const i = e.indexOf(t);
      i > -1 && e.splice(i, 1);
    };
  }
  function a() {
    e = [];
  }
  return { add: n, list: () => e.slice(), reset: a };
}
function Ae(e, n, a, t, i, s = (l) => l()) {
  const l = t && (t.enterCallbacks[i] = t.enterCallbacks[i] || []);
  return () => new Promise((r, u) => {
    const c = (p) => {
      p === false ? u(Ze(ne.NAVIGATION_ABORTED, { from: a, to: n })) : p instanceof Error ? u(p) : zs(p) ? u(Ze(ne.NAVIGATION_GUARD_REDIRECT, { from: n, to: p })) : (l && t.enterCallbacks[i] === l && typeof p == "function" && l.push(p), r());
    }, _ = s(() => e.call(t && t.instances[i], n, a, c));
    let m = Promise.resolve(_);
    e.length < 3 && (m = m.then(c)), m.catch((p) => u(p));
  });
}
function Ln(e, n, a, t, i = (s) => s()) {
  const s = [];
  for (const l of e) for (const r in l.components) {
    let u = l.components[r];
    if (!(n !== "beforeRouteEnter" && !l.instances[r])) if (bt(u)) {
      const c = (u.__vccOpts || u)[n];
      c && s.push(Ae(c, a, t, l, r, i));
    } else {
      let c = u();
      s.push(() => c.then((_) => {
        if (!_) throw new Error(`Couldn't resolve component "${r}" at "${l.path}"`);
        const m = ns(_) ? _.default : _;
        l.mods[r] = _, l.components[r] = m;
        const p = (m.__vccOpts || m)[n];
        return p && Ae(p, a, t, l, r, i)();
      }));
    }
  }
  return s;
}
function qs(e, n) {
  const a = [], t = [], i = [], s = Math.max(n.matched.length, e.matched.length);
  for (let l = 0; l < s; l++) {
    const r = n.matched[l];
    r && (e.matched.find((c) => Xe(c, r)) ? t.push(r) : a.push(r));
    const u = e.matched[l];
    u && (n.matched.find((c) => Xe(c, u)) || i.push(u));
  }
  return [a, t, i];
}
/*!
* vue-router v4.6.4
* (c) 2025 Eduardo San Martin Morote
* @license MIT
*/
let Is = () => location.protocol + "//" + location.host;
function Rt(e, n) {
  const { pathname: a, search: t, hash: i } = n, s = e.indexOf("#");
  if (s > -1) {
    let l = i.includes(e.slice(s)) ? e.slice(s).length : 1, r = i.slice(l);
    return r[0] !== "/" && (r = "/" + r), Ra(r, "");
  }
  return Ra(a, e) + t + i;
}
function Os(e, n, a, t) {
  let i = [], s = [], l = null;
  const r = ({ state: p }) => {
    const d = Rt(e, location), v = a.value, P = n.value;
    let V = 0;
    if (p) {
      if (a.value = d, n.value = p, l && l === v) {
        l = null;
        return;
      }
      V = P ? p.position - P.position : 0;
    } else t(d);
    i.forEach((y) => {
      y(a.value, v, { delta: V, type: Bn.pop, direction: V ? V > 0 ? Tn.forward : Tn.back : Tn.unknown });
    });
  };
  function u() {
    l = a.value;
  }
  function c(p) {
    i.push(p);
    const d = () => {
      const v = i.indexOf(p);
      v > -1 && i.splice(v, 1);
    };
    return s.push(d), d;
  }
  function _() {
    if (document.visibilityState === "hidden") {
      const { history: p } = window;
      if (!p.state) return;
      p.replaceState(J({}, p.state, { scroll: zn() }), "");
    }
  }
  function m() {
    for (const p of s) p();
    s = [], window.removeEventListener("popstate", r), window.removeEventListener("pagehide", _), document.removeEventListener("visibilitychange", _);
  }
  return window.addEventListener("popstate", r), window.addEventListener("pagehide", _), document.addEventListener("visibilitychange", _), { pauseListeners: u, listen: c, destroy: m };
}
function Oa(e, n, a, t = false, i = false) {
  return { back: e, current: n, forward: a, replaced: t, position: window.history.length, scroll: i ? zn() : null };
}
function Ts(e) {
  const { history: n, location: a } = window, t = { value: Rt(e, a) }, i = { value: n.state };
  i.value || s(t.value, { back: null, current: t.value, forward: null, position: n.length - 1, replaced: true, scroll: null }, true);
  function s(u, c, _) {
    const m = e.indexOf("#"), p = m > -1 ? (a.host && document.querySelector("base") ? e : e.slice(m)) + u : Is() + e + u;
    try {
      n[_ ? "replaceState" : "pushState"](c, "", p), i.value = c;
    } catch (d) {
      console.error(d), a[_ ? "replace" : "assign"](p);
    }
  }
  function l(u, c) {
    s(u, J({}, n.state, Oa(i.value.back, u, i.value.forward, true), c, { position: i.value.position }), true), t.value = u;
  }
  function r(u, c) {
    const _ = J({}, i.value, n.state, { forward: u, scroll: zn() });
    s(_.current, _, true), s(u, J({}, Oa(t.value, u, null), { position: _.position + 1 }, c), false), t.value = u;
  }
  return { location: t, state: i, push: r, replace: l };
}
function Nt(e) {
  e = ws(e);
  const n = Ts(e), a = Os(e, n.state, n.location, n.replace);
  function t(s, l = true) {
    l || a.pauseListeners(), history.go(s);
  }
  const i = J({ location: "", base: e, go: t, createHref: xs.bind(null, e) }, n, a);
  return Object.defineProperty(i, "location", { enumerable: true, get: () => n.location.value }), Object.defineProperty(i, "state", { enumerable: true, get: () => n.state.value }), i;
}
function Ls(e) {
  return e = location.host ? e || location.pathname + location.search : "", e.includes("#") || (e += "#"), Nt(e);
}
let Me = (function(e) {
  return e[e.Static = 0] = "Static", e[e.Param = 1] = "Param", e[e.Group = 2] = "Group", e;
})({});
var ae = (function(e) {
  return e[e.Static = 0] = "Static", e[e.Param = 1] = "Param", e[e.ParamRegExp = 2] = "ParamRegExp", e[e.ParamRegExpEnd = 3] = "ParamRegExpEnd", e[e.EscapeNext = 4] = "EscapeNext", e;
})(ae || {});
const Ds = { type: Me.Static, value: "" }, Ms = /[a-zA-Z0-9_]/;
function Us(e) {
  if (!e) return [[]];
  if (e === "/") return [[Ds]];
  if (!e.startsWith("/")) throw new Error(`Invalid path "${e}"`);
  function n(d) {
    throw new Error(`ERR (${a})/"${c}": ${d}`);
  }
  let a = ae.Static, t = a;
  const i = [];
  let s;
  function l() {
    s && i.push(s), s = [];
  }
  let r = 0, u, c = "", _ = "";
  function m() {
    c && (a === ae.Static ? s.push({ type: Me.Static, value: c }) : a === ae.Param || a === ae.ParamRegExp || a === ae.ParamRegExpEnd ? (s.length > 1 && (u === "*" || u === "+") && n(`A repeatable param (${c}) must be alone in its segment. eg: '/:ids+.`), s.push({ type: Me.Param, value: c, regexp: _, repeatable: u === "*" || u === "+", optional: u === "*" || u === "?" })) : n("Invalid state to consume buffer"), c = "");
  }
  function p() {
    c += u;
  }
  for (; r < e.length; ) {
    if (u = e[r++], u === "\\" && a !== ae.ParamRegExp) {
      t = a, a = ae.EscapeNext;
      continue;
    }
    switch (a) {
      case ae.Static:
        u === "/" ? (c && m(), l()) : u === ":" ? (m(), a = ae.Param) : p();
        break;
      case ae.EscapeNext:
        p(), a = t;
        break;
      case ae.Param:
        u === "(" ? a = ae.ParamRegExp : Ms.test(u) ? p() : (m(), a = ae.Static, u !== "*" && u !== "?" && u !== "+" && r--);
        break;
      case ae.ParamRegExp:
        u === ")" ? _[_.length - 1] == "\\" ? _ = _.slice(0, -1) + u : a = ae.ParamRegExpEnd : _ += u;
        break;
      case ae.ParamRegExpEnd:
        m(), a = ae.Static, u !== "*" && u !== "?" && u !== "+" && r--, _ = "";
        break;
      default:
        n("Unknown state");
        break;
    }
  }
  return a === ae.ParamRegExp && n(`Unfinished custom RegExp for param "${c}"`), m(), l(), i;
}
const Ta = "[^/]+?", Vs = { sensitive: false, strict: false, start: true, end: true };
var ie = (function(e) {
  return e[e._multiplier = 10] = "_multiplier", e[e.Root = 90] = "Root", e[e.Segment = 40] = "Segment", e[e.SubSegment = 30] = "SubSegment", e[e.Static = 40] = "Static", e[e.Dynamic = 20] = "Dynamic", e[e.BonusCustomRegExp = 10] = "BonusCustomRegExp", e[e.BonusWildcard = -50] = "BonusWildcard", e[e.BonusRepeatable = -20] = "BonusRepeatable", e[e.BonusOptional = -8] = "BonusOptional", e[e.BonusStrict = 0.7000000000000001] = "BonusStrict", e[e.BonusCaseSensitive = 0.25] = "BonusCaseSensitive", e;
})(ie || {});
const Gs = /[.+*?^${}()[\]/\\]/g;
function Bs(e, n) {
  const a = J({}, Vs, n), t = [];
  let i = a.start ? "^" : "";
  const s = [];
  for (const c of e) {
    const _ = c.length ? [] : [ie.Root];
    a.strict && !c.length && (i += "/");
    for (let m = 0; m < c.length; m++) {
      const p = c[m];
      let d = ie.Segment + (a.sensitive ? ie.BonusCaseSensitive : 0);
      if (p.type === Me.Static) m || (i += "/"), i += p.value.replace(Gs, "\\$&"), d += ie.Static;
      else if (p.type === Me.Param) {
        const { value: v, repeatable: P, optional: V, regexp: y } = p;
        s.push({ name: v, repeatable: P, optional: V });
        const k = y || Ta;
        if (k !== Ta) {
          d += ie.BonusCustomRegExp;
          try {
            `${k}`;
          } catch (D) {
            throw new Error(`Invalid custom RegExp for param "${v}" (${k}): ` + D.message);
          }
        }
        let w = P ? `((?:${k})(?:/(?:${k}))*)` : `(${k})`;
        m || (w = V && c.length < 2 ? `(?:/${w})` : "/" + w), V && (w += "?"), i += w, d += ie.Dynamic, V && (d += ie.BonusOptional), P && (d += ie.BonusRepeatable), k === ".*" && (d += ie.BonusWildcard);
      }
      _.push(d);
    }
    t.push(_);
  }
  if (a.strict && a.end) {
    const c = t.length - 1;
    t[c][t[c].length - 1] += ie.BonusStrict;
  }
  a.strict || (i += "/?"), a.end ? i += "$" : a.strict && !i.endsWith("/") && (i += "(?:/|$)");
  const l = new RegExp(i, a.sensitive ? "" : "i");
  function r(c) {
    const _ = c.match(l), m = {};
    if (!_) return null;
    for (let p = 1; p < _.length; p++) {
      const d = _[p] || "", v = s[p - 1];
      m[v.name] = d && v.repeatable ? d.split("/") : d;
    }
    return m;
  }
  function u(c) {
    let _ = "", m = false;
    for (const p of e) {
      (!m || !_.endsWith("/")) && (_ += "/"), m = false;
      for (const d of p) if (d.type === Me.Static) _ += d.value;
      else if (d.type === Me.Param) {
        const { value: v, repeatable: P, optional: V } = d, y = v in c ? c[v] : "";
        if (he(y) && !P) throw new Error(`Provided param "${v}" is an array but it is not repeatable (* or + modifiers)`);
        const k = he(y) ? y.join("/") : y;
        if (!k) if (V) p.length < 2 && (_.endsWith("/") ? _ = _.slice(0, -1) : m = true);
        else throw new Error(`Missing required param "${v}"`);
        _ += k;
      }
    }
    return _ || "/";
  }
  return { re: l, score: t, keys: s, parse: r, stringify: u };
}
function Fs(e, n) {
  let a = 0;
  for (; a < e.length && a < n.length; ) {
    const t = n[a] - e[a];
    if (t) return t;
    a++;
  }
  return e.length < n.length ? e.length === 1 && e[0] === ie.Static + ie.Segment ? -1 : 1 : e.length > n.length ? n.length === 1 && n[0] === ie.Static + ie.Segment ? 1 : -1 : 0;
}
function Pt(e, n) {
  let a = 0;
  const t = e.score, i = n.score;
  for (; a < t.length && a < i.length; ) {
    const s = Fs(t[a], i[a]);
    if (s) return s;
    a++;
  }
  if (Math.abs(i.length - t.length) === 1) {
    if (La(t)) return 1;
    if (La(i)) return -1;
  }
  return i.length - t.length;
}
function La(e) {
  const n = e[e.length - 1];
  return e.length > 0 && n[n.length - 1] < 0;
}
const Hs = { strict: false, end: true, sensitive: false };
function Ws(e, n, a) {
  const t = Bs(Us(e.path), a), i = J(t, { record: e, parent: n, children: [], alias: [] });
  return n && !i.record.aliasOf == !n.record.aliasOf && n.children.push(i), i;
}
function Ks(e, n) {
  const a = [], t = /* @__PURE__ */ new Map();
  n = Ca(Hs, n);
  function i(m) {
    return t.get(m);
  }
  function s(m, p, d) {
    const v = !d, P = Ma(m);
    P.aliasOf = d && d.record;
    const V = Ca(n, m), y = [P];
    if ("alias" in m) {
      const D = typeof m.alias == "string" ? [m.alias] : m.alias;
      for (const A of D) y.push(Ma(J({}, P, { components: d ? d.record.components : P.components, path: A, aliasOf: d ? d.record : P })));
    }
    let k, w;
    for (const D of y) {
      const { path: A } = D;
      if (p && A[0] !== "/") {
        const M = p.record.path, C = M[M.length - 1] === "/" ? "" : "/";
        D.path = p.record.path + (A && C + A);
      }
      if (k = Ws(D, p, V), d ? d.alias.push(k) : (w = w || k, w !== k && w.alias.push(k), v && m.name && !Ua(k) && l(m.name)), qt(k) && u(k), P.children) {
        const M = P.children;
        for (let C = 0; C < M.length; C++) s(M[C], k, d && d.children[C]);
      }
      d = d || k;
    }
    return w ? () => {
      l(w);
    } : cn;
  }
  function l(m) {
    if (At(m)) {
      const p = t.get(m);
      p && (t.delete(m), a.splice(a.indexOf(p), 1), p.children.forEach(l), p.alias.forEach(l));
    } else {
      const p = a.indexOf(m);
      p > -1 && (a.splice(p, 1), m.record.name && t.delete(m.record.name), m.children.forEach(l), m.alias.forEach(l));
    }
  }
  function r() {
    return a;
  }
  function u(m) {
    const p = Ys(m, a);
    a.splice(p, 0, m), m.record.name && !Ua(m) && t.set(m.record.name, m);
  }
  function c(m, p) {
    let d, v = {}, P, V;
    if ("name" in m && m.name) {
      if (d = t.get(m.name), !d) throw Ze(ne.MATCHER_NOT_FOUND, { location: m });
      V = d.record.name, v = J(Da(p.params, d.keys.filter((w) => !w.optional).concat(d.parent ? d.parent.keys.filter((w) => w.optional) : []).map((w) => w.name)), m.params && Da(m.params, d.keys.map((w) => w.name))), P = d.stringify(v);
    } else if (m.path != null) P = m.path, d = a.find((w) => w.re.test(P)), d && (v = d.parse(P), V = d.record.name);
    else {
      if (d = p.name ? t.get(p.name) : a.find((w) => w.re.test(p.path)), !d) throw Ze(ne.MATCHER_NOT_FOUND, { location: m, currentLocation: p });
      V = d.record.name, v = J({}, p.params, m.params), P = d.stringify(v);
    }
    const y = [];
    let k = d;
    for (; k; ) y.unshift(k.record), k = k.parent;
    return { name: V, path: P, params: v, matched: y, meta: Qs(y) };
  }
  e.forEach((m) => s(m));
  function _() {
    a.length = 0, t.clear();
  }
  return { addRoute: s, resolve: c, removeRoute: l, clearRoutes: _, getRoutes: r, getRecordMatcher: i };
}
function Da(e, n) {
  const a = {};
  for (const t of n) t in e && (a[t] = e[t]);
  return a;
}
function Ma(e) {
  const n = { path: e.path, redirect: e.redirect, name: e.name, meta: e.meta || {}, aliasOf: e.aliasOf, beforeEnter: e.beforeEnter, props: Js(e), children: e.children || [], instances: {}, leaveGuards: /* @__PURE__ */ new Set(), updateGuards: /* @__PURE__ */ new Set(), enterCallbacks: {}, components: "components" in e ? e.components || null : e.component && { default: e.component } };
  return Object.defineProperty(n, "mods", { value: {} }), n;
}
function Js(e) {
  const n = {}, a = e.props || false;
  if ("component" in e) n.default = a;
  else for (const t in e.components) n[t] = typeof a == "object" ? a[t] : a;
  return n;
}
function Ua(e) {
  for (; e; ) {
    if (e.record.aliasOf) return true;
    e = e.parent;
  }
  return false;
}
function Qs(e) {
  return e.reduce((n, a) => J(n, a.meta), {});
}
function Ys(e, n) {
  let a = 0, t = n.length;
  for (; a !== t; ) {
    const s = a + t >> 1;
    Pt(e, n[s]) < 0 ? t = s : a = s + 1;
  }
  const i = Xs(e);
  return i && (t = n.lastIndexOf(i, t - 1)), t;
}
function Xs(e) {
  let n = e;
  for (; n = n.parent; ) if (qt(n) && Pt(e, n) === 0) return n;
}
function qt({ record: e }) {
  return !!(e.name || e.components && Object.keys(e.components).length || e.redirect);
}
function Va(e) {
  const n = Ge(An), a = Ge(la), t = L(() => {
    const u = z(e.to);
    return n.resolve(u);
  }), i = L(() => {
    const { matched: u } = t.value, { length: c } = u, _ = u[c - 1], m = a.matched;
    if (!_ || !m.length) return -1;
    const p = m.findIndex(Xe.bind(null, _));
    if (p > -1) return p;
    const d = Ga(u[c - 2]);
    return c > 1 && Ga(_) === d && m[m.length - 1].path !== d ? m.findIndex(Xe.bind(null, u[c - 2])) : p;
  }), s = L(() => i.value > -1 && ao(a.params, t.value.params)), l = L(() => i.value > -1 && i.value === a.matched.length - 1 && zt(a.params, t.value.params));
  function r(u = {}) {
    if (no(u)) {
      const c = n[z(e.replace) ? "replace" : "push"](z(e.to)).catch(cn);
      return e.viewTransition && typeof document < "u" && "startViewTransition" in document && document.startViewTransition(() => c), c;
    }
    return Promise.resolve();
  }
  return { route: t, href: L(() => t.value.href), isActive: s, isExactActive: l, navigate: r };
}
function Zs(e) {
  return e.length === 1 ? e[0] : e;
}
const eo = se({ name: "RouterLink", compatConfig: { MODE: 3 }, props: { to: { type: [String, Object], required: true }, replace: Boolean, activeClass: String, exactActiveClass: String, custom: Boolean, ariaCurrentValue: { type: String, default: "page" }, viewTransition: Boolean }, useLink: Va, setup(e, { slots: n }) {
  const a = ta(Va(e)), { options: t } = Ge(An), i = L(() => ({ [Ba(e.activeClass, t.linkActiveClass, "router-link-active")]: a.isActive, [Ba(e.exactActiveClass, t.linkExactActiveClass, "router-link-exact-active")]: a.isExactActive }));
  return () => {
    const s = n.default && Zs(n.default(a));
    return e.custom ? s : _t("a", { "aria-current": a.isExactActive ? e.ariaCurrentValue : null, href: a.href, onClick: a.navigate, class: i.value }, s);
  };
} }), ze = eo;
function no(e) {
  if (!(e.metaKey || e.altKey || e.ctrlKey || e.shiftKey) && !e.defaultPrevented && !(e.button !== void 0 && e.button !== 0)) {
    if (e.currentTarget && e.currentTarget.getAttribute) {
      const n = e.currentTarget.getAttribute("target");
      if (/\b_blank\b/i.test(n)) return;
    }
    return e.preventDefault && e.preventDefault(), true;
  }
}
function ao(e, n) {
  for (const a in n) {
    const t = n[a], i = e[a];
    if (typeof t == "string") {
      if (t !== i) return false;
    } else if (!he(i) || i.length !== t.length || t.some((s, l) => s.valueOf() !== i[l].valueOf())) return false;
  }
  return true;
}
function Ga(e) {
  return e ? e.aliasOf ? e.aliasOf.path : e.path : "";
}
const Ba = (e, n, a) => e ?? n ?? a, to = se({ name: "RouterView", inheritAttrs: false, props: { name: { type: String, default: "default" }, route: Object }, compatConfig: { MODE: 3 }, setup(e, { attrs: n, slots: a }) {
  const t = Ge(Hn), i = L(() => e.route || t.value), s = Ge(Ia, 0), l = L(() => {
    let c = z(s);
    const { matched: _ } = i.value;
    let m;
    for (; (m = _[c]) && !m.components; ) c++;
    return c;
  }), r = L(() => i.value.matched[l.value]);
  Nn(Ia, L(() => l.value + 1)), Nn(Ps, r), Nn(Hn, i);
  const u = I();
  return Be(() => [u.value, r.value, e.name], ([c, _, m], [p, d, v]) => {
    _ && (_.instances[m] = c, d && d !== _ && c && c === p && (_.leaveGuards.size || (_.leaveGuards = d.leaveGuards), _.updateGuards.size || (_.updateGuards = d.updateGuards))), c && _ && (!d || !Xe(_, d) || !p) && (_.enterCallbacks[m] || []).forEach((P) => P(c));
  }, { flush: "post" }), () => {
    const c = i.value, _ = e.name, m = r.value, p = m && m.components[_];
    if (!p) return Fa(a.default, { Component: p, route: c });
    const d = m.props[_], v = d ? d === true ? c.params : typeof d == "function" ? d(c) : d : null, V = _t(p, J({}, v, n, { onVnodeUnmounted: (y) => {
      y.component.isUnmounted && (m.instances[_] = null);
    }, ref: u }));
    return Fa(a.default, { Component: V, route: c }) || V;
  };
} });
function Fa(e, n) {
  if (!e) return null;
  const a = e(n);
  return a.length === 1 ? a[0] : a;
}
const It = to;
function io(e) {
  const n = Ks(e.routes, e), a = e.parseQuery || Rs, t = e.stringifyQuery || qa, i = e.history, s = an(), l = an(), r = an(), u = di(Se);
  let c = Se;
  Je && e.scrollBehavior && "scrollRestoration" in history && (history.scrollRestoration = "manual");
  const _ = In.bind(null, (b) => "" + b), m = In.bind(null, hs), p = In.bind(null, hn);
  function d(b, q) {
    let R, O;
    return At(b) ? (R = n.getRecordMatcher(b), O = q) : O = b, n.addRoute(O, R);
  }
  function v(b) {
    const q = n.getRecordMatcher(b);
    q && n.removeRoute(q);
  }
  function P() {
    return n.getRoutes().map((b) => b.record);
  }
  function V(b) {
    return !!n.getRecordMatcher(b);
  }
  function y(b, q) {
    if (q = J({}, q || u.value), typeof b == "string") {
      const B = On(a, b, q.path), ee = n.resolve({ path: B.path }, q), le = i.createHref(B.fullPath);
      return J(B, ee, { params: p(ee.params), hash: hn(B.hash), redirectedFrom: void 0, href: le });
    }
    let R;
    if (b.path != null) R = J({}, b, { path: On(a, b.path, q.path).path });
    else {
      const B = J({}, b.params);
      for (const ee in B) B[ee] == null && delete B[ee];
      R = J({}, b, { params: m(B) }), q.params = m(q.params);
    }
    const O = n.resolve(R, q), H = b.hash || "";
    O.params = _(p(O.params));
    const X = _s(t, J({}, b, { hash: ds(H), path: O.path })), F = i.createHref(X);
    return J({ fullPath: X, hash: H, query: t === qa ? Ns(b.query) : b.query || {} }, O, { redirectedFrom: void 0, href: F });
  }
  function k(b) {
    return typeof b == "string" ? On(a, b, u.value.path) : J({}, b);
  }
  function w(b, q) {
    if (c !== b) return Ze(ne.NAVIGATION_CANCELLED, { from: q, to: b });
  }
  function D(b) {
    return C(b);
  }
  function A(b) {
    return D(J(k(b), { replace: true }));
  }
  function M(b, q) {
    const R = b.matched[b.matched.length - 1];
    if (R && R.redirect) {
      const { redirect: O } = R;
      let H = typeof O == "function" ? O(b, q) : O;
      return typeof H == "string" && (H = H.includes("?") || H.includes("#") ? H = k(H) : { path: H }, H.params = {}), J({ query: b.query, hash: b.hash, params: H.path != null ? {} : b.params }, H);
    }
  }
  function C(b, q) {
    const R = c = y(b), O = u.value, H = b.state, X = b.force, F = b.replace === true, B = M(R, O);
    if (B) return C(J(k(B), { state: typeof B == "object" ? J({}, H, B.state) : H, force: X, replace: F }), q || R);
    const ee = R;
    ee.redirectedFrom = q;
    let le;
    return !X && vs(t, O, R) && (le = Ze(ne.NAVIGATION_DUPLICATED, { to: ee, from: O }), N(O, O, true, false)), (le ? Promise.resolve(le) : fe(ee, O)).catch((te) => ke(te) ? ke(te, ne.NAVIGATION_GUARD_REDIRECT) ? te : E(te) : Ie(te, ee, O)).then((te) => {
      if (te) {
        if (ke(te, ne.NAVIGATION_GUARD_REDIRECT)) return C(J({ replace: F }, k(te.to), { state: typeof te.to == "object" ? J({}, H, te.to.state) : H, force: X }), q || ee);
      } else te = $e(ee, O, true, F, H);
      return He(ee, O, te), te;
    });
  }
  function re(b, q) {
    const R = w(b, q);
    return R ? Promise.reject(R) : Promise.resolve();
  }
  function ce(b) {
    const q = de.values().next().value;
    return q && typeof q.runWithContext == "function" ? q.runWithContext(b) : b();
  }
  function fe(b, q) {
    let R;
    const [O, H, X] = qs(b, q);
    R = Ln(O.reverse(), "beforeRouteLeave", b, q);
    for (const B of O) B.leaveGuards.forEach((ee) => {
      R.push(Ae(ee, b, q));
    });
    const F = re.bind(null, b, q);
    return R.push(F), je(R).then(() => {
      R = [];
      for (const B of s.list()) R.push(Ae(B, b, q));
      return R.push(F), je(R);
    }).then(() => {
      R = Ln(H, "beforeRouteUpdate", b, q);
      for (const B of H) B.updateGuards.forEach((ee) => {
        R.push(Ae(ee, b, q));
      });
      return R.push(F), je(R);
    }).then(() => {
      R = [];
      for (const B of X) if (B.beforeEnter) if (he(B.beforeEnter)) for (const ee of B.beforeEnter) R.push(Ae(ee, b, q));
      else R.push(Ae(B.beforeEnter, b, q));
      return R.push(F), je(R);
    }).then(() => (b.matched.forEach((B) => B.enterCallbacks = {}), R = Ln(X, "beforeRouteEnter", b, q, ce), R.push(F), je(R))).then(() => {
      R = [];
      for (const B of l.list()) R.push(Ae(B, b, q));
      return R.push(F), je(R);
    }).catch((B) => ke(B, ne.NAVIGATION_CANCELLED) ? B : Promise.reject(B));
  }
  function He(b, q, R) {
    r.list().forEach((O) => ce(() => O(b, q, R)));
  }
  function $e(b, q, R, O, H) {
    const X = w(b, q);
    if (X) return X;
    const F = q === Se, B = Je ? history.state : {};
    R && (O || F ? i.replace(b.fullPath, J({ scroll: F && B && B.scroll }, H)) : i.push(b.fullPath, H)), u.value = b, N(b, q, R, F), E();
  }
  let ve;
  function Pe() {
    ve || (ve = i.listen((b, q, R) => {
      if (!Te.listening) return;
      const O = y(b), H = M(O, Te.currentRoute.value);
      if (H) {
        C(J(H, { replace: true, force: true }), O).catch(cn);
        return;
      }
      c = O;
      const X = u.value;
      Je && Ss(Pa(X.fullPath, R.delta), zn()), fe(O, X).catch((F) => ke(F, ne.NAVIGATION_ABORTED | ne.NAVIGATION_CANCELLED) ? F : ke(F, ne.NAVIGATION_GUARD_REDIRECT) ? (C(J(k(F.to), { force: true }), O).then((B) => {
        ke(B, ne.NAVIGATION_ABORTED | ne.NAVIGATION_DUPLICATED) && !R.delta && R.type === Bn.pop && i.go(-1, false);
      }).catch(cn), Promise.reject()) : (R.delta && i.go(-R.delta, false), Ie(F, O, X))).then((F) => {
        F = F || $e(O, X, false), F && (R.delta && !ke(F, ne.NAVIGATION_CANCELLED) ? i.go(-R.delta, false) : R.type === Bn.pop && ke(F, ne.NAVIGATION_ABORTED | ne.NAVIGATION_DUPLICATED) && i.go(-1, false)), He(O, X, F);
      }).catch(cn);
    }));
  }
  let qe = an(), We = an(), ye;
  function Ie(b, q, R) {
    E(b);
    const O = We.list();
    return O.length ? O.forEach((H) => H(b, q, R)) : console.error(b), Promise.reject(b);
  }
  function Oe() {
    return ye && u.value !== Se ? Promise.resolve() : new Promise((b, q) => {
      qe.add([b, q]);
    });
  }
  function E(b) {
    return ye || (ye = !b, Pe(), qe.list().forEach(([q, R]) => b ? R(b) : q()), qe.reset()), b;
  }
  function N(b, q, R, O) {
    const { scrollBehavior: H } = e;
    if (!Je || !H) return Promise.resolve();
    const X = !R && Es(Pa(b.fullPath, 0)) || (O || !R) && history.state && history.state.scroll || null;
    return aa().then(() => H(b, q, X)).then((F) => F && js(F)).catch((F) => Ie(F, b, q));
  }
  const U = (b) => i.go(b);
  let K;
  const de = /* @__PURE__ */ new Set(), Te = { currentRoute: u, listening: true, addRoute: d, removeRoute: v, clearRoutes: n.clearRoutes, hasRoute: V, getRoutes: P, resolve: y, options: e, push: D, replace: A, go: U, back: () => U(-1), forward: () => U(1), beforeEach: s.add, beforeResolve: l.add, afterEach: r.add, onError: We.add, isReady: Oe, install(b) {
    b.component("RouterLink", ze), b.component("RouterView", It), b.config.globalProperties.$router = Te, Object.defineProperty(b.config.globalProperties, "$route", { enumerable: true, get: () => z(u) }), Je && !K && u.value === Se && (K = true, D(i.location).catch((O) => {
    }));
    const q = {};
    for (const O in Se) Object.defineProperty(q, O, { get: () => u.value[O], enumerable: true });
    b.provide(An, Te), b.provide(la, ci(q)), b.provide(Hn, u);
    const R = b.unmount;
    de.add(b), b.unmount = function() {
      de.delete(b), de.size < 1 && (c = Se, ve && ve(), ve = null, u.value = Se, K = false, ye = false), R();
    };
  } };
  function je(b) {
    return b.reduce((q, R) => q.then(() => ce(R)), Promise.resolve());
  }
  return Te;
}
function Ot() {
  return Ge(An);
}
function so(e) {
  return Ge(la);
}
const oo = { class: "app-shell" }, ro = { class: "topbar", role: "banner" }, lo = { class: "nav", "aria-label": "\u4E3B\u5BFC\u822A" }, uo = { class: "main", id: "main", tabindex: "-1" }, co = se({ __name: "App", setup(e) {
  return (n, a) => (h(), g("div", oo, [o("header", ro, [pe(z(ze), { class: "brand", to: "/", "aria-label": "\u65E0\u540D\u6740 Next \u9996\u9875" }, { default: De(() => [...a[0] || (a[0] = [W(" \u65E0\u540D\u6740 ", -1), o("span", { class: "accent" }, "Next", -1)])]), _: 1 }), o("nav", lo, [pe(z(ze), { to: "/" }, { default: De(() => [...a[1] || (a[1] = [W("\u5927\u5385", -1)])]), _: 1 }), pe(z(ze), { to: "/battle" }, { default: De(() => [...a[2] || (a[2] = [W("\u5BF9\u5C40", -1)])]), _: 1 }), pe(z(ze), { to: "/extensions" }, { default: De(() => [...a[3] || (a[3] = [W("\u6269\u5C55", -1)])]), _: 1 }), pe(z(ze), { to: "/settings" }, { default: De(() => [...a[4] || (a[4] = [W("\u8BBE\u7F6E", -1)])]), _: 1 }), pe(z(ze), { to: "/replay" }, { default: De(() => [...a[5] || (a[5] = [W("\u5F55\u50CF", -1)])]), _: 1 }), pe(z(ze), { to: "/collection" }, { default: De(() => [...a[6] || (a[6] = [W("\u6B66\u5C06\u5E93", -1)])]), _: 1 })])]), o("main", uo, [pe(z(It))])]));
} }), mo = { class: "stack" }, po = { class: "panel hero" }, ho = { key: 0, class: "shell-badge" }, go = { class: "row form" }, fo = ["value"], _o = { class: "field" }, vo = { class: "panel" }, yo = { class: "online-grid" }, ko = { class: "wide" }, wo = ["value"], bo = { class: "row online-actions" }, xo = { key: 0, class: "error", role: "alert" }, $o = se({ __name: "HomeView", setup(e) {
  const n = Ot(), a = I(4), t = I(42), i = I("prototype"), s = I(`player-${Math.floor(Math.random() * 900 + 100)}`), l = I(""), r = I(1), u = I(30), c = I(v()), _ = I(""), m = I(null);
  gn(async () => {
    m.value = await mi();
  });
  function p() {
    n.push({ path: "/battle", query: { players: String(a.value), seed: String(t.value), ...i.value === "identity" ? { mode: "identity" } : {} } });
  }
  function d(V) {
    if (_.value = "", !c.value.trim()) {
      _.value = "\u8BF7\u586B\u5199 WebSocket \u5730\u5740";
      return;
    }
    if (V !== "create" && r.value < 1) {
      _.value = "\u623F\u95F4\u53F7\u5FC5\u987B\u5927\u4E8E 0";
      return;
    }
    sessionStorage.setItem("noname-next.online-intent", JSON.stringify({ action: V, wsUrl: c.value, displayName: s.value, password: l.value, roomId: r.value, players: a.value, seed: t.value, choiceTimeoutSecs: u.value })), n.push({ path: "/battle", query: { mode: "online", action: V, ...V === "create" ? {} : { room: String(r.value) } } });
  }
  function v() {
    if (typeof window > "u") return "ws://127.0.0.1:7700/ws";
    const V = window.location.protocol === "https:" ? "wss:" : "ws:", y = window.location.hostname || "127.0.0.1";
    return `${V}//${y}:7700/ws`;
  }
  const P = L(() => r.value > 0 ? `#${r.value}` : "\u2014");
  return (V, y) => (h(), g("div", mo, [o("section", po, [y[15] || (y[15] = o("h1", null, "\u786E\u5B9A\u6027\u8EAB\u4EFD\u5C40", -1)), y[16] || (y[16] = o("p", { class: "muted" }, [W(" \u9ED8\u8BA4\u539F\u751F\u5F15\u64CE\u540E\u7AEF\uFF08engine-native-wasm \u7A84\u58F3\uFF0CWorker \u5185\u8FD0\u884C\uFF09\u3002\u754C\u9762\u53EA\u6D88\u8D39\u89C2\u5BDF\u8005\u6295\u5F71\uFF0C\u4E0D\u6301\u6709\u5B8C\u6574\u6743\u5A01\u72B6\u6001\u3002\u53EF\u7528 "), o("code", null, "VITE_ENGINE_BACKEND=legacy"), W(" \u56DE\u9000\u65E7\u7EBF\u3002 ")], -1)), m.value ? (h(), g("p", ho, " Tauri 2 \xB7 " + f(m.value.os) + " \xB7 " + f(m.value.arch), 1)) : S("v-if", true), o("div", go, [o("label", null, [y[11] || (y[11] = W(" \u4EBA\u6570 ", -1)), Z(o("select", { "onUpdate:modelValue": y[0] || (y[0] = (k) => a.value = k), "aria-label": "\u73A9\u5BB6\u4EBA\u6570" }, [(h(), g(Q, null, Y([2, 3, 4, 5, 6, 7, 8], (k) => o("option", { key: k, value: k }, f(k), 9, fo)), 64))], 512), [[_e, a.value, void 0, { number: true }]])]), o("label", null, [y[12] || (y[12] = W(" \u79CD\u5B50 ", -1)), Z(o("input", { "onUpdate:modelValue": y[1] || (y[1] = (k) => t.value = k), type: "number", min: "0", "aria-label": "\u968F\u673A\u79CD\u5B50" }, null, 512), [[Re, t.value, void 0, { number: true }]])]), o("label", _o, [y[14] || (y[14] = o("span", null, "\u6A21\u5F0F", -1)), Z(o("select", { "onUpdate:modelValue": y[2] || (y[2] = (k) => i.value = k), "aria-label": "\u5BF9\u5C40\u6A21\u5F0F" }, [...y[13] || (y[13] = [o("option", { value: "prototype" }, "\u539F\u578B\uFF08\u552F\u4E00\u5B58\u6D3B\u8005\uFF09", -1), o("option", { value: "identity" }, "\u8EAB\u4EFD\u5C40\uFF08\u4E3B\u5FE0/\u53CD\u8D3C/\u5185\u5978\uFF09", -1)])], 512), [[_e, i.value]])]), o("button", { class: "primary", type: "button", onClick: p }, " \u672C\u5730\u5F00\u5C40 ")])]), o("section", vo, [y[21] || (y[21] = o("h2", null, "\u8054\u673A\u5927\u5385", -1)), y[22] || (y[22] = o("p", { class: "muted" }, " \u901A\u8FC7 game-server WebSocket \u52A0\u5165\u6743\u5A01\u623F\u95F4\u3002\u6D4F\u89C8\u5668\u4EC5\u63D0\u4EA4\u547D\u4EE4\u5E76\u63A5\u6536\u81EA\u5DF1\u7684\u89C2\u5BDF\u8005\u6295\u5F71\u3002 ", -1)), o("div", yo, [o("label", ko, [y[17] || (y[17] = W(" \u670D\u52A1\u7AEF ", -1)), Z(o("input", { "onUpdate:modelValue": y[3] || (y[3] = (k) => c.value = k), type: "url", autocomplete: "url" }, null, 512), [[Re, c.value, void 0, { trim: true }]])]), o("label", null, [y[18] || (y[18] = W(" \u6635\u79F0 ", -1)), Z(o("input", { "onUpdate:modelValue": y[4] || (y[4] = (k) => s.value = k), maxlength: "24", autocomplete: "nickname" }, null, 512), [[Re, s.value, void 0, { trim: true }]])]), o("label", null, [y[19] || (y[19] = W(" \u9009\u62E9\u65F6\u9650 ", -1)), Z(o("select", { "onUpdate:modelValue": y[5] || (y[5] = (k) => u.value = k), "aria-label": "\u9009\u62E9\u65F6\u9650" }, [(h(), g(Q, null, Y([10, 15, 30, 60, 90], (k) => o("option", { key: k, value: k }, f(k) + " \u79D2", 9, wo)), 64))], 512), [[_e, u.value, void 0, { number: true }]])]), o("label", null, [y[20] || (y[20] = W(" \u623F\u95F4\u5BC6\u7801 ", -1)), Z(o("input", { "onUpdate:modelValue": y[6] || (y[6] = (k) => l.value = k), type: "password", autocomplete: "current-password" }, null, 512), [[Re, l.value]])]), o("label", null, [W(" \u623F\u95F4\u53F7 " + f(P.value) + " ", 1), Z(o("input", { "onUpdate:modelValue": y[7] || (y[7] = (k) => r.value = k), type: "number", min: "1", inputmode: "numeric" }, null, 512), [[Re, r.value, void 0, { number: true }]])])]), o("div", bo, [o("button", { class: "primary", type: "button", onClick: y[8] || (y[8] = (k) => d("create")) }, "\u521B\u5EFA\u5E76\u5165\u5EA7"), o("button", { type: "button", onClick: y[9] || (y[9] = (k) => d("join")) }, "\u52A0\u5165\u623F\u95F4"), o("button", { type: "button", onClick: y[10] || (y[10] = (k) => d("spectate")) }, "\u65C1\u89C2")]), _.value ? (h(), g("p", xo, f(_.value), 1)) : S("v-if", true)]), y[23] || (y[23] = o("section", { class: "panel" }, [o("h2", null, "\u67B6\u6784\u8981\u70B9"), o("ul", { class: "muted list" }, [o("li", null, "\u4E3B\u7EBF\u7A0B\uFF1AVue Shell + DOM \u6218\u6597\u5C42 + \u52A8\u753B\u65F6\u95F4\u7EBF"), o("li", null, "\u672C\u5730\uFF1AWorker + \u539F\u751F\u7A84\u58F3\uFF08\u9ED8\u8BA4\uFF09/ \u65E7\u7EBF WASM\uFF1B\u8054\u673A\uFF1AWebSocket + \u6743\u5A01\u670D\u52A1\u5668"), o("li", null, "\u6269\u5C55\u5305\u7BA1\u7406\uFF1A\u5B89\u88C5 / \u542F\u505C / \u5347\u7EA7 / \u5378\u8F7D\uFF08\u5BFC\u822A\u300C\u6269\u5C55\u300D\uFF09"), o("li", null, "\u652F\u6301\u81EA\u52A8\u6B65\u8FDB\u3001\u5B58\u8BFB\u6863\u3001\u500D\u901F\u3001\u8DF3\u8FC7\u52A8\u753B\u3001\u51CF\u5C11\u52A8\u6001\u6548\u679C")])], -1))]));
} }), ge = (e, n) => {
  const a = e.__vccOpts || e;
  for (const [t, i] of n) a[t] = i;
  return a;
}, jo = ge($o, [["__scopeId", "data-v-bb2f538c"]]), Wn = 200, T = ta({ contentLockHash: "", total: 0, pages: /* @__PURE__ */ new Map(), characters: /* @__PURE__ */ new Map(), skins: /* @__PURE__ */ new Map(), translations: {}, selectedSkins: /* @__PURE__ */ new Map(), portraitUrls: /* @__PURE__ */ new Map(), imageUrls: /* @__PURE__ */ new Map(), substituteUrls: /* @__PURE__ */ new Map() });
let Ne = null;
const on = /* @__PURE__ */ new Map(), rn = /* @__PURE__ */ new Map(), Kn = /* @__PURE__ */ new Map();
function So(e) {
  if (ia()) {
    Yn({ kind: "desktop", key: "desktop" });
    return;
  }
  const n = `${window.location.protocol === "https:" ? "https:" : "http:"}//${window.location.hostname || "127.0.0.1"}:7700`;
  Yn({ kind: "http", key: `http:${n}`, baseUrl: n });
}
function Eo(e) {
  const n = new URL(e.trim(), window.location.href);
  n.protocol = n.protocol === "wss:" || n.protocol === "https:" ? "https:" : "http:";
  const a = n.origin;
  Yn({ kind: "http", key: `http:${a}`, baseUrl: a });
}
async function zo(e) {
  const n = T.characters.get(e);
  if (n) return T.imageUrls.has(e) || await Qn(e), n;
  if (!Ne) return;
  const a = rn.get(e);
  if (a) return a;
  const t = Ne.key, i = (async () => {
    let s = 0;
    for (; (Ne == null ? void 0 : Ne.key) === t; ) {
      const l = await Tt(s);
      if (!l) break;
      const r = T.characters.get(e);
      if (r) return await Qn(e), r;
      const u = l.offset + l.characters.length;
      if (u >= l.total || u <= s) break;
      s = u;
    }
  })();
  rn.set(e, i);
  try {
    return await i;
  } finally {
    rn.get(e) === i && rn.delete(e);
  }
}
function Ao(e) {
  var _a2;
  const n = (_a2 = T.characters.get(e)) == null ? void 0 : _a2.definition.name_key;
  return n ? Lt(n) : void 0;
}
function Co(e) {
  var _a2;
  return ((_a2 = T.characters.get(e)) == null ? void 0 : _a2.skin_ids.flatMap((a) => {
    const t = T.skins.get(a);
    return t && t.presentation.kind !== "substitute" ? [t] : [];
  })) ?? [];
}
function Ro(e) {
  return Lt(e.name_key);
}
function Jn(e) {
  const n = T.characters.get(e);
  if (!n || !T.contentLockHash) return null;
  if (T.selectedSkins.has(e)) return T.selectedSkins.get(e) ?? null;
  const a = Dt(e);
  let t = null;
  if (a) try {
    const i = Number(localStorage.getItem(a));
    Number.isInteger(i) && Mt(n).includes(i) && (t = i);
  } catch {
  }
  return T.selectedSkins.set(e, t), t;
}
function No(e, n) {
  const a = T.characters.get(e);
  if (n != null && (!a || !Mt(a).includes(n))) return;
  T.selectedSkins.set(e, n);
  const t = Dt(e);
  if (t) try {
    n == null ? localStorage.removeItem(t) : localStorage.setItem(t, String(n));
  } catch {
  }
  Qn(e);
}
function Po(e) {
  return T.imageUrls.get(e) ?? Cn(e);
}
function qo(e) {
  return T.portraitUrls.get(e) ?? Cn(e);
}
function Io(e, n) {
  return T.substituteUrls.get(`${e}:${n}`) ?? null;
}
function Cn(e) {
  return `/original/portraits/${e}.jpg`;
}
async function Tt(e) {
  const n = T.pages.get(e);
  if (n) return n;
  const a = on.get(e);
  if (a) return a;
  const t = Ne;
  if (!t) return null;
  const i = (async () => {
    let s;
    if (t.kind === "desktop") s = await hi(e, Wn);
    else {
      const l = new URL("/content-catalog", t.baseUrl);
      l.searchParams.set("offset", String(e)), l.searchParams.set("limit", String(Wn));
      const r = await fetch(l);
      if (!r.ok) throw new Error(`\u5185\u5BB9\u76EE\u5F55\u52A0\u8F7D\u5931\u8D25 (${r.status})`);
      s = await r.json();
    }
    if (Ne !== t || s == null) return null;
    if (To(s), s.offset !== Math.min(e, s.total)) throw new Error("\u5185\u5BB9\u76EE\u5F55\u5206\u9875\u504F\u79FB\u4E0D\u4E00\u81F4");
    return e !== 0 && T.contentLockHash && T.contentLockHash !== s.content_lock_hash ? (ua(), Tt(0)) : (Oo(s), s);
  })();
  on.set(e, i);
  try {
    return await i;
  } finally {
    on.get(e) === i && on.delete(e);
  }
}
function Oo(e) {
  T.contentLockHash && T.contentLockHash !== e.content_lock_hash && ua(), T.contentLockHash = e.content_lock_hash, T.total = e.total, T.pages.set(e.offset, e);
  for (const n of e.characters) T.characters.set(n.definition.id, n);
  for (const n of e.skins) T.skins.set(n.id, n);
  for (const [n, a] of Object.entries(e.translations)) T.translations[n] = { ...T.translations[n], ...a };
}
async function Qn(e) {
  const n = T.characters.get(e);
  if (!n) return;
  const a = Jn(e), t = a == null ? void 0 : T.skins.get(a), i = (t == null ? void 0 : t.presentation.kind) === "static" ? t.presentation.asset : null, s = Cn(e);
  T.portraitUrls.set(e, s), T.imageUrls.set(e, s);
  const l = `${T.contentLockHash}:${a ?? "base"}`, r = n.portrait_asset ? await Dn(n.portrait_asset).catch(() => null) : null, u = i ? await Dn(i).catch(() => null) : null, c = n.skin_ids.map((p) => T.skins.get(p)).find((p) => (p == null ? void 0 : p.presentation.kind) === "substitute" && p.presentation.rule === "dead"), _ = (c == null ? void 0 : c.presentation.kind) === "substitute" ? c.presentation.asset : null, m = _ ? await Dn(_).catch(() => null) : null;
  if (l === `${T.contentLockHash}:${Jn(e) ?? "base"}`) {
    const p = r ?? s;
    T.portraitUrls.set(e, p), T.imageUrls.set(e, u ?? p), m && T.substituteUrls.set(`${e}:dead`, m);
  }
}
function Dn(e) {
  if (!Lo(e) || !Ne) return Promise.resolve(null);
  const n = Ne, a = [n.key, e.namespace, e.package_id, e.asset_id].join("\0"), t = Kn.get(a);
  if (t) return t;
  const i = n.kind === "desktop" ? pi(e) : Promise.resolve(`${n.baseUrl}/package-assets/${[e.namespace, e.package_id].map(encodeURIComponent).join("/")}/${e.asset_id.split("/").map(encodeURIComponent).join("/")}?v=${encodeURIComponent(T.contentLockHash)}`);
  return Kn.set(a, i), i;
}
function Lt(e) {
  var _a2;
  const n = typeof navigator > "u" ? "" : navigator.language, a = [n, n.replace("-", "_"), "zh-CN", "zh_CN", "zh", "en"];
  for (const t of a) {
    const i = (_a2 = T.translations[t]) == null ? void 0 : _a2[e];
    if (i) return i;
  }
  return e;
}
function Dt(e) {
  return T.contentLockHash ? `noname-next.skin.${T.contentLockHash}.${e}` : null;
}
function Yn(e) {
  Ne = e, ua();
}
function ua() {
  T.contentLockHash = "", T.total = 0, T.pages.clear(), T.characters.clear(), T.skins.clear(), T.translations = {}, T.selectedSkins.clear(), T.portraitUrls.clear(), T.imageUrls.clear(), T.substituteUrls.clear(), on.clear(), rn.clear(), Kn.clear();
}
function To(e) {
  const n = e;
  if (!n || typeof n.content_lock_hash != "string" || !Ee(n.offset) || !Ee(n.limit) || !Ee(n.total) || n.limit < 1 || n.limit > Wn || n.offset > n.total || !Array.isArray(n.characters) || n.characters.length > n.limit || !Array.isArray(n.skins) || !Array.isArray(n.aliases) || !n.translations || typeof n.translations != "object" || !n.characters.every((a) => {
    var _a2, _b;
    return Ee((_a2 = a == null ? void 0 : a.definition) == null ? void 0 : _a2.id) && a.definition.id > 0 && typeof ((_b = a == null ? void 0 : a.definition) == null ? void 0 : _b.name_key) == "string" && Array.isArray(a == null ? void 0 : a.skin_ids) && a.skin_ids.every(Ee);
  }) || !n.skins.every((a) => {
    var _a2;
    return Ee(a == null ? void 0 : a.id) && a.id > 0 && typeof (a == null ? void 0 : a.name_key) == "string" && ["static", "spine", "substitute"].includes((_a2 = a == null ? void 0 : a.presentation) == null ? void 0 : _a2.kind);
  }) || !n.aliases.every((a) => Ee(a == null ? void 0 : a.character_id) && Ee(a == null ? void 0 : a.source_character_id) && Ha(a == null ? void 0 : a.voice_mappings)) || !Object.values(n.translations).every(Ha)) throw new Error("\u5185\u5BB9\u76EE\u5F55\u54CD\u5E94\u65E0\u6548");
}
function Ee(e) {
  return Number.isSafeInteger(e) && Number(e) >= 0;
}
function Mt(e) {
  return e.skin_ids.filter((n) => {
    var _a2;
    return ((_a2 = T.skins.get(n)) == null ? void 0 : _a2.presentation.kind) !== "substitute";
  });
}
function Ha(e) {
  return !!e && typeof e == "object" && !Array.isArray(e) && Object.values(e).every((n) => typeof n == "string");
}
function Lo(e) {
  return typeof e.namespace == "string" && e.namespace.length > 0 && typeof e.package_id == "string" && e.package_id.length > 0 && typeof e.asset_id == "string" && e.asset_id.length > 0;
}
const dn = [{ dir: "cardpile-upstream-liuli", packageId: "cardpile", namespace: "upstream-liuli", version: "1.0.0", title: "\u724C\u5806\uFF08\u7409\u7483\uFF09", logicFingerprint: "8dd99eb2" }, { dir: "cardpile-upstream-noname", packageId: "cardpile", namespace: "upstream-noname", version: "1.0.0", title: "\u724C\u5806\uFF08\u65E0\u540D\uFF09", logicFingerprint: "83aa81c4" }, { dir: "d3-upstream-noname", packageId: "d3-select", namespace: "upstream-noname", version: "1.0.0", title: "3D\u7CBE\u9009", logicFingerprint: "6f257a96" }, { dir: "huanle-upstream-noname", packageId: "huanle-cards", namespace: "upstream-noname", version: "1.0.0", title: "\u6B22\u4E50\u5361\u724C", logicFingerprint: "6ae1907c" }, { dir: "shaai-ow-upstream-noname", packageId: "shaai-ow", namespace: "upstream-noname", version: "1.0.0", title: "\u5B88\u671B\u5148\u950B", logicFingerprint: "6117a13e" }, { dir: "shaai-xianjian-upstream-noname", packageId: "shaai-xianjian", namespace: "upstream-noname", version: "1.0.0", title: "\u4ED9\u5251\u5947\u4FA0\u4F20", logicFingerprint: "d9ee1863" }, { dir: "shahai-gujian-upstream-noname", packageId: "shahai-gujian", namespace: "upstream-noname", version: "1.0.0", title: "\u53E4\u5251\u5947\u8C2D", logicFingerprint: "ae13018f" }, { dir: "shahai-gwent-upstream-noname", packageId: "shahai-gwent", namespace: "upstream-noname", version: "1.0.0", title: "\u6606\u7279\u724C", logicFingerprint: "0bd2569a" }, { dir: "shahai-hearth-upstream-noname", packageId: "shahai-hearth", namespace: "upstream-noname", version: "1.0.0", title: "\u7089\u77F3\u4F20\u8BF4", logicFingerprint: "deafc31c" }, { dir: "shahai-mtg-upstream-noname", packageId: "shahai-mtg", namespace: "upstream-noname", version: "1.0.0", title: "\u4E07\u667A\u724C", logicFingerprint: "6417be73" }, { dir: "shahai-swd-upstream-noname", packageId: "shahai-swd", namespace: "upstream-noname", version: "1.0.0", title: "\u8F69\u8F95\u5251", logicFingerprint: "31ccea98" }, { dir: "shahai-yunchou-upstream-noname", packageId: "shahai-yunchou", namespace: "upstream-noname", version: "1.0.0", title: "\u8FD0\u7B79\u5E37\u5E44", logicFingerprint: "57491089" }, { dir: "shahai-zhenfa-upstream-noname", packageId: "shahai-zhenfa", namespace: "upstream-noname", version: "1.0.0", title: "\u9635\u6CD5", logicFingerprint: "53945b47" }, { dir: "wudian-upstream-noname", packageId: "wudian", namespace: "upstream-noname", version: "1.0.0", title: "\u73A9\u70B9\u8BBA\u6740", logicFingerprint: "e0d488b6" }, { dir: "wuxing-upstream-liuli", packageId: "wuxing", namespace: "upstream-liuli", version: "1.0.0", title: "\u4E94\u884C", logicFingerprint: "845ea6fa" }];
function ca(e) {
  return `${e.namespace}/${e.packageId}`;
}
const Do = { package_format_version: 1, namespace: "upstream-liuli", package_id: "cardpile", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 753, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [141, 217, 158, 178, 102, 250, 23, 48, 46, 154, 229, 230, 185, 21, 142, 207, 121, 191, 21, 160, 114, 238, 113, 65, 184, 219, 63, 10, 143, 211, 102, 196], asset_hash: [5, 133, 68, 112, 135, 182, 90, 250, 76, 92, 107, 127, 63, 60, 145, 39, 21, 177, 136, 200, 170, 224, 209, 228, 167, 139, 124, 218, 97, 232, 139, 249], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, Mo = [], Uo = [], Vo = [], Go = [], Bo = [], Fo = [], Ho = [], Wo = [], Ko = { "zh-CN": { cardpile: "\u724C\u5806", huosha: "\u706B\u6740", jiu: "\u9152", leisha: "\u96F7\u6740" } }, Jo = { schema_version: 1, package_id: "cardpile", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "cardpile:pile", source_hash: [223, 149, 103, 52, 153, 51, 90, 98, 219, 54, 199, 28, 186, 38, 121, 216, 91, 164, 250, 84, 188, 239, 98, 143, 239, 121, 48, 90, 56, 228, 25, 104], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, Qo = { "cardpile:pile": "assets/pile.json" }, Yo = "gen-1", Wa = { manifest: Do, modules: Mo, skill_index: Uo, character_index: Vo, card_index: Go, trigger_index: Bo, modifier_index: Fo, active_index: Ho, view_as_index: Wo, translations: Ko, asset_manifest: Jo, asset_paths: Qo, compiler_version: Yo }, Xo = "noname-liuli", Zo = "noname/extension/cardpile/extension.js", er = "8485dbb29ff16f9b258a766b1a17ef75f883aade", nr = 160, ar = [{ card: "sha", counts: { diamond: 6, club: 14, heart: 3, spade: 7 } }, { card: "huosha", counts: { diamond: 2, heart: 3 } }, { card: "leisha", counts: { spade: 5, club: 4 } }, { card: "shan", counts: { heart: 6, diamond: 18 } }, { card: "jiu", counts: { diamond: 1, spade: 2, club: 2 } }, { card: "tao", counts: { heart: 9, diamond: 3 } }, { card: "wanjian", counts: { heart: 1 } }, { card: "nanman", counts: { spade: 2, club: 1 } }, { card: "guohe", counts: { spade: 3, club: 2, heart: 1 } }, { card: "shunshou", counts: { spade: 3, diamond: 2 } }, { card: "wuxie", counts: { heart: 2, diamond: 1, spade: 2, club: 2 } }, { card: "tiesuo", counts: { spade: 2, club: 4 } }], tr = { source: Xo, upstream_path: Zo, upstream_commit: er, total: nr, entries: ar }, ir = { package_format_version: 1, namespace: "upstream-noname", package_id: "cardpile", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 753, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [131, 170, 129, 196, 137, 101, 152, 220, 92, 26, 119, 144, 188, 18, 226, 118, 124, 202, 213, 56, 37, 221, 167, 227, 108, 52, 173, 68, 188, 187, 99, 220], asset_hash: [128, 188, 216, 204, 239, 216, 63, 133, 226, 76, 81, 90, 3, 222, 209, 209, 187, 152, 128, 170, 74, 141, 193, 125, 195, 157, 166, 254, 81, 218, 111, 12], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, sr = [], or = [], rr = [], lr = [], ur = [], cr = [], dr = [], mr = [], pr = { "zh-CN": { cardpile: "\u724C\u5806", huosha: "\u706B\u6740", jiu: "\u9152", leisha: "\u96F7\u6740" } }, hr = { schema_version: 1, package_id: "cardpile", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "cardpile:pile", source_hash: [107, 194, 232, 96, 169, 46, 18, 235, 51, 137, 142, 3, 98, 227, 253, 110, 137, 37, 246, 88, 93, 161, 193, 213, 246, 39, 29, 229, 204, 37, 149, 34], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, gr = { "cardpile:pile": "assets/pile.json" }, fr = "gen-1", Ka = { manifest: ir, modules: sr, skill_index: or, character_index: rr, card_index: lr, trigger_index: ur, modifier_index: cr, active_index: dr, view_as_index: mr, translations: pr, asset_manifest: hr, asset_paths: gr, compiler_version: fr }, _r = "noname", vr = "apps/core/extension/cardpile/extension.js", yr = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", kr = 160, wr = [{ card: "sha", counts: { diamond: 6, club: 14, heart: 3, spade: 7 } }, { card: "huosha", counts: { diamond: 2, heart: 3 } }, { card: "leisha", counts: { spade: 5, club: 4 } }, { card: "shan", counts: { heart: 6, diamond: 18 } }, { card: "jiu", counts: { diamond: 1, spade: 2, club: 2 } }, { card: "tao", counts: { heart: 9, diamond: 3 } }, { card: "wanjian", counts: { heart: 1 } }, { card: "nanman", counts: { spade: 2, club: 1 } }, { card: "guohe", counts: { spade: 3, club: 2, heart: 1 } }, { card: "shunshou", counts: { spade: 3, diamond: 2 } }, { card: "wuxie", counts: { heart: 2, diamond: 1, spade: 2, club: 2 } }, { card: "tiesuo", counts: { spade: 2, club: 4 } }], br = { source: _r, upstream_path: vr, upstream_commit: yr, total: kr, entries: wr }, xr = { package_format_version: 1, namespace: "upstream-noname", package_id: "d3-select", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [111, 37, 122, 150, 119, 48, 147, 56, 241, 49, 200, 32, 57, 124, 107, 75, 23, 118, 103, 218, 206, 252, 0, 59, 192, 24, 0, 59, 108, 78, 62, 128], asset_hash: [16, 5, 163, 133, 161, 153, 19, 47, 137, 85, 182, 46, 7, 149, 75, 218, 226, 85, 46, 131, 186, 196, 41, 78, 176, 6, 210, 119, 138, 54, 121, 189], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, $r = [], jr = [], Sr = [], Er = [], zr = [], Ar = [], Cr = [], Rr = [], Nr = { "zh-CN": { ddd_liuchong: "ddd\u5218\u51B2", dddjinggou: "\u7CBE\u5F40", dddmoyan: "\u672B\u7131" } }, Pr = { schema_version: 1, package_id: "d3-select", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "d3-select:characters", source_hash: [50, 247, 124, 76, 209, 138, 160, 245, 215, 11, 118, 39, 184, 31, 122, 238, 107, 84, 39, 122, 206, 75, 86, 186, 245, 248, 189, 22, 28, 40, 135, 250], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, qr = { "d3-select:characters": "assets/character.json" }, Ir = "gen-1", Ja = { manifest: xr, modules: $r, skill_index: jr, character_index: Sr, card_index: Er, trigger_index: zr, modifier_index: Ar, active_index: Cr, view_as_index: Rr, translations: Nr, asset_manifest: Pr, asset_paths: qr, compiler_version: Ir }, Or = "noname", Tr = "apps/core/extension/3D\u7CBE\u9009/", Lr = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", Dr = { name: "d3-upstream-noname", version: 1, description: "G1 3D\u7CBE\u9009", card_prefix: "ddd_" }, Mr = [{ id: "ddd_liuchong", name: "ddd\u5218\u51B2", faction: "qun", hp: 4, gender: "male", skills: [{ name: "dddjinggou", title: "\u7CBE\u5F40", mapping: "host:max_range_weapon_damage" }, { name: "dddmoyan", title: "\u672B\u7131", mapping: "host:d3_moyan" }] }], Ur = { source: Or, upstream_path: Tr, upstream_commit: Lr, pack: Dr, generals: Mr }, Vr = { package_format_version: 1, namespace: "upstream-noname", package_id: "huanle-cards", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [106, 225, 144, 124, 144, 13, 45, 180, 159, 95, 125, 53, 98, 74, 29, 117, 24, 49, 199, 10, 60, 244, 167, 94, 32, 142, 255, 23, 184, 119, 161, 55], asset_hash: [25, 84, 214, 199, 128, 168, 24, 45, 132, 188, 58, 79, 25, 130, 141, 162, 163, 191, 146, 92, 141, 235, 240, 14, 64, 255, 184, 47, 114, 101, 54, 250], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, Gr = [], Br = [], Fr = [], Hr = [], Wr = [], Kr = [], Jr = [], Qr = [], Yr = { "zh-CN": { huanle: "\u6B22\u4E50\u5361\u724C", jiuwei: "\u9152\u5C3E", kuwu: "\u82E6\u65E0", shoulijian: "\u624B\u91CC\u5251" } }, Xr = { schema_version: 1, package_id: "huanle-cards", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "huanle-cards:cards", source_hash: [8, 4, 171, 213, 237, 11, 78, 79, 232, 10, 93, 164, 129, 27, 245, 153, 138, 180, 85, 242, 69, 243, 148, 105, 243, 156, 24, 165, 76, 211, 77, 17], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, Zr = { "huanle-cards:cards": "assets/cards.json" }, el = "gen-1", Qa = { manifest: Vr, modules: Gr, skill_index: Br, character_index: Fr, card_index: Hr, trigger_index: Wr, modifier_index: Kr, active_index: Jr, view_as_index: Qr, translations: Yr, asset_manifest: Xr, asset_paths: Zr, compiler_version: el }, nl = "noname", al = "apps/core/extension/\u6B22\u4E50\u5361\u724C/", tl = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", il = { name: "huanle-upstream-noname", version: 1, description: "G1 \u6B22\u4E50\u5361\u724C", card_prefix: "hnl_" }, sl = [{ name: "kuwu", kind: "weapon", range: 2, equip_skills: ["huanle_kuwu"], suit: "spade", rank: 6 }, { name: "shoulijian", kind: "shoulijian", suit: "diamond", rank: 4 }, { name: "shoulijian", kind: "shoulijian", suit: "spade", rank: 4 }, { name: "monkey", kind: "monkey", suit: "diamond", rank: 5 }, { name: "xuelunyang", kind: "xuelunyang", suit: "heart", rank: 2 }, { name: "mianju", kind: "mianju", suit: "club", rank: 3 }, { name: "jiuwei", kind: "jiuwei", suit: "heart", rank: 9, equip_skills: ["huanle_jiuwei"] }], ol = [{ name: "huanle_kuwu", mapping: "host:sha_damage_force_discard" }, { name: "huanle_jiuwei", mapping: "host:phase_end_recover_or_draw" }], rl = { source: nl, upstream_path: al, upstream_commit: tl, pack: il, cards: sl, skills: ol }, ll = { package_format_version: 1, namespace: "upstream-noname", package_id: "shaai-ow", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [97, 23, 161, 62, 250, 238, 201, 81, 165, 68, 177, 185, 128, 62, 23, 122, 149, 20, 247, 219, 47, 165, 34, 207, 63, 74, 111, 21, 134, 193, 80, 44], asset_hash: [234, 208, 248, 98, 248, 214, 252, 238, 255, 142, 242, 125, 150, 77, 115, 114, 87, 46, 204, 14, 51, 9, 132, 104, 243, 158, 19, 237, 178, 87, 25, 1], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, ul = [], cl = [], dl = [], ml = [], pl = [], hl = [], gl = [], fl = [], _l = { "zh-CN": { diaobingqianjiang: "\u8C03\u5175\u9063\u5C06", sanjian: "\u4E09\u5C16", wangmeizhike: "\u671B\u6885\u6B62\u6E34", youdishenru: "\u8BF1\u654C\u6DF1\u5165" } }, vl = { schema_version: 1, package_id: "shaai-ow", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "shaai-ow:cards", source_hash: [156, 170, 113, 113, 238, 249, 132, 22, 250, 155, 70, 184, 125, 243, 73, 245, 56, 16, 110, 98, 179, 110, 249, 104, 140, 229, 198, 10, 8, 170, 103, 219], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, yl = { "shaai-ow:cards": "assets/cards.json" }, kl = "gen-1", Ya = { manifest: ll, modules: ul, skill_index: cl, character_index: dl, card_index: ml, trigger_index: pl, modifier_index: hl, active_index: gl, view_as_index: fl, translations: _l, asset_manifest: vl, asset_paths: yl, compiler_version: kl }, wl = "qita/sanguosha/noname/extension/\u6740\u6D77\u62FE\u9057/character/ow", bl = "extension/\u6740\u6D77\u62FE\u9057/character/ow/", xl = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", $l = { name: "shaai-ow-upstream-noname", version: 1, description: "G1 \u7EB5\u5207\uFF1A\u4E0A\u6E38 ow \u58EB\u517576\uFF08\u7A81\u51FB+\u76EE\u955C\uFF09", card_prefix: "ow_" }, jl = [], Sl = [], El = [], zl = [{ name: "guangshu", general: "ow_zhixuzhiguang", mapping: "host:wudian_guangshu" }, { name: "guangshu_heart", general: "ow_zhixuzhiguang", mapping: "host:wudian_guangshu_heart" }, { name: "guangshu_diamond", general: "ow_zhixuzhiguang", mapping: "host:wudian_guangshu_diamond" }, { name: "guangshu_club", general: "ow_zhixuzhiguang", mapping: "host:wudian_guangshu_club" }, { name: "guangshu_spade", general: "ow_zhixuzhiguang", mapping: "host:wudian_guangshu_spade" }, { name: "ziyu", general: "ow_chanyata", mapping: "host:wudian_ziyu" }, { name: "shouhu", general: "ow_tianshi", mapping: "host:wudian_shouhu" }, { name: "duwen", general: "ow_heibaihe", mapping: "host:wudian_duwen" }, { name: "paotai", general: "ow_tuobiang", mapping: "host:wudian_paotai" }, { name: "paotai2", general: "ow_tuobiang", mapping: "host:wudian_paotai_fire" }, { name: "bshaowei", general: "ow_baolei", mapping: "host:wudian_baolei_switch" }], Al = [{ name: "\u58EB\u517576", faction: "shu", hp: 4, gender: "male", skills: [{ name: "tuji", mapping: "host:wudian_tuji" }, { name: "mujing", mapping: "host:wudian_mujing" }, { name: "lichang", mapping: null, reason: "\u4E0B\u51C6\u5907\u9636\u6BB5\u5B50\u56DE\u5408\u65F6\u70B9\u672A\u5EFA\u6A21\u2014\u20142026-10-08 \u6682\u7F13" }] }, { name: "\u865A\u65E0\u4E4B\u5149", faction: "qun", hp: 3, gender: "female", skills: [{ name: "guangshu", mapping: "host:wudian_guangshu" }] }, { name: "\u7985\u96C5\u5854", faction: "qun", hp: 3, gender: "male", skills: [{ name: "ziyu", mapping: "host:wudian_ziyu" }] }, { name: "\u5929\u4F7F", faction: "shu", hp: 3, gender: "female", skills: [{ name: "shouhu", mapping: "host:wudian_shouhu" }, { name: "ziyu", mapping: "host:wudian_ziyu" }, { name: "feiying", mapping: null, reason: "\u6E90\u5B9A\u4E49\u4E0D\u5728 ow/skill.js\uFF08\u5171\u4EAB\u6280\u80FD\u6587\u4EF6\uFF09\u2014\u20142026-10-08 \u6682\u7F13" }] }, { name: "\u9ED1\u767E\u5408", faction: "wei", hp: 3, gender: "female", skills: [{ name: "duwen", mapping: "host:wudian_duwen" }] }, { name: "\u6258\u6BD4\u6602", faction: "shu", hp: 4, gender: "male", skills: [{ name: "paotai", mapping: "host:wudian_paotai" }, { name: "paotai2", mapping: "host:wudian_paotai_fire" }, { name: "maoding", mapping: null, reason: "\u96F6\u4EF6\u724C\u6C60\uFF08hslingjian\uFF09\u4E0D\u5728\u539F\u751F\u76EE\u5F55\u2014\u20142026-10-08 \u6682\u7F13" }] }, { name: "\u5821\u5792", faction: "shu", hp: 4, gender: "male", skills: [{ name: "bshaowei", mapping: "host:wudian_baolei_switch" }, { name: "zhencha", mapping: "host:wudian_baolei_switch" }, { name: "zhencha_draw", mapping: "host:wudian_zhencha_draw" }] }, { name: "\u5362\u897F\u5965", faction: "wu", hp: 3, gender: "male", skills: [{ name: "yuedong", mapping: "host:wudian_yuedong" }, { name: "kuoyin", mapping: "host:wudian_kuoyin" }, { name: "huhuan", mapping: "host:wudian_huhuan" }] }, { name: "\u7F8E", faction: "wei", hp: 3, gender: "female", skills: [{ name: "jidong", mapping: "host:wudian_jidong" }, { name: "baoxue", mapping: "host:wudian_baoxue" }, { name: "bingqiang", mapping: null, reason: "\u5EA7\u4F4D\u76F8\u90BB/\u8DDD\u79BB\u4FEE\u9970\u591A\u76EE\u6807\uFF08bingqiang2-5 \u5B50\u6280\u65CF+changeTarget \u76F8\u90BB\u6269\u5C55\uFF09\u2014\u20142026-10-08 \u6682\u7F13" }] }, { name: "\u8DEF\u9738", faction: "shu", hp: 4, gender: "male", skills: [{ name: "liangou", mapping: "host:wudian_liangou" }, { name: "xiyang", mapping: "host:wudian_xiyang" }] }, { name: "\u6E90\u6C0F", faction: "qun", hp: 3, gender: "male", skills: [{ name: "feiren", mapping: "host:wudian_feiren" }, { name: "lianpo", mapping: null, reason: "\u6E90\u5B9A\u4E49\u4E0D\u5728 ow/skill.js\uFF08\u5171\u4EAB\u6280\u80FD\u6587\u4EF6\uFF0Cfeiying \u540C\u6B3E\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }, { name: "zhanlong", mapping: null, reason: "limited+createCard \u51ED\u7A7A\u9020\u724C 3 \u6740+zhanlong2 \u4E34\u65F6\u6280\u94FE\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u6CD5\u8001\u4E4B\u9E70", faction: "shu", hp: 3, gender: "female", skills: [{ name: "feidan", mapping: "host:wudian_feidan" }, { name: "huoyu", mapping: null, reason: "limited+\u865A\u62DF useCard\uFF08chiyuxi \u51ED\u7A7A\u7528\u724C\xD72\uFF09\u2014\u2014\u865A\u62DF\u7528\u724C\u539F\u8BED\u540C youzhan \u7F3A\u53E3\uFF0C2026-10-09 \u6682\u7F13" }, { name: "feiying", mapping: null, reason: "\u6E90\u5B9A\u4E49\u4E0D\u5728 ow/skill.js\uFF08\u5171\u4EAB\u6280\u80FD\u6587\u4EF6\uFF09\u2014\u20142026-10-08 \u6682\u7F13" }] }, { name: "\u5B89\u5A1C", faction: "wei", hp: 3, gender: "female", skills: [{ name: "zhiyuan", mapping: "host:wudian_zhiyuan" }, { name: "mianzhen", mapping: null, reason: "\u76EE\u6807\u4FA7 chooseToRespond \u54CD\u5E94\u8BE2\u95EE+mianzhen2 \u6C38\u4E45\u5B50\u6280\u2014\u20142026-10-09 \u6682\u7F13" }, { name: "aqianghua", mapping: null, reason: "\u7ED9\u724C+\u62A4\u7532+aqianghua2 \u5B50\u6280\u94FE\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u9EA6\u514B\u96F7", faction: "shu", hp: 4, gender: "male", skills: [{ name: "shenqiang", mapping: "host:wudian_shenqiang" }, { name: "shanguang", mapping: null, reason: "shanguang2 \u5168\u5361\u724C\u7981\u7528\u5B50\u6280\uFF08mod cardEnabled/cardUsable/cardRespondable/cardSavable \u5168 false\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }, { name: "tiandan", mapping: null, reason: "createCard \u51ED\u7A7A\u9020\u6740\xD7N+\u4E3B\u52A8\u8DF3\u8FC7\u51FA\u724C/\u5F03\u724C\u9636\u6BB5\uFF08skip\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u72C2\u9F20", faction: "shu", hp: 3, gender: "male", skills: [{ name: "shihuo", mapping: "host:wudian_shihuo" }, { name: "liudan", mapping: null, reason: "\u6740\u4F7F\u7528\u65F6 50% \u6982\u7387\u968F\u673A\u8FFD\u52A0\u5408\u6CD5\u76EE\u6807\uFF08RNG \u76EE\u6807\u8FFD\u52A0+lib.filter.targetEnabled \u590D\u7528\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }, { name: "shoujia", mapping: null, reason: "\u7ED9\u724C+shoujia2 \u6302\u724C\u5B50\u6280\uFF08storage \u53CC\u5411\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u83B1\u56E0\u54C8\u7279", faction: "qun", hp: 4, gender: "male", skills: [{ name: "mengji", mapping: "host:wudian_mengji" }, { name: "zhongdun", mapping: null, reason: "init2 \u5F00\u5C40 +8 \u62A4\u7532\uFF08\u65E0 GameStart \u4E8B\u4EF6\u79CD\u7C7B=\u5F00\u5C40\u4E8B\u4EF6\u57FA\u5EFA\u7F3A\u53E3\uFF09+\u4E3B\u52A8\u8F6C\u7532\u6280\u2014\u20142026-10-09 \u6682\u7F13\uFF1Bmengji \u7684 storage.zhongdun \u7F6E\u4F4D\u4EE5\u6301\u6280\u7B49\u4EF7" }] }, { name: "\u534A\u85CF", faction: "qun", hp: 4, gender: "male", skills: [{ name: "yinbo", mapping: "host:wudian_yinbo" }, { name: "bfengshi", mapping: null, reason: "shaBegin Math.random \u6982\u7387\u76F4\u51FB+countUsed \u653B\u8DDD+\u7EC4\u6280 bfengshi2/3/4\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u5965\u4E3D\u838E", faction: "wu", hp: 4, gender: "female", skills: [{ name: "qianggu", mapping: "host:wudian_qianggu" }, { name: "qianggu2", mapping: "host:wudian_qianggu_guard" }, { name: "woliu", mapping: null, reason: "\u56DE\u5408\u7ED3\u675F\u9009 1-2 \u76EE\u6807\u5171\u6302 woliu2 \u7EC4\u8DDD\u79BB/\u6740\u9650\u5B9A+dieBegin/phaseBegin \u6E05\u9664\u7EC4\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u6E29\u65AF\u987F", faction: "shu", hp: 4, gender: "male", skills: [{ name: "feitiao", mapping: "host:wudian_feitiao" }, { name: "dianji", mapping: null, reason: "viewAs \u60CA\u96F7\u6740\uFF08jingleishan \u724C\u79CD\u4E0D\u5728\u539F\u751F\u76EE\u5F55\uFF09+\u8DDD\u79BB 2 \u9650\u5B9A\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u67E5\u8389\u5A05", faction: "wei", hp: 4, gender: "female", skills: [{ name: "pingzhang", mapping: "host:wudian_pingzhang" }, { name: "owliyong", mapping: "host:wudian_owliyong" }] }, { name: "\u9ED1\u5F71", faction: "wei", hp: 3, gender: "female", skills: [{ name: "yinshen", mapping: "host:wudian_yinshen" }, { name: "qinru", mapping: null, reason: "useCardToBegin \u5224\u5B9A+\u8DE8\u56DE\u5408\u76EE\u6807\u5B58\u50A8\u5217\u8868\uFF08qinru_turn \u5206\u6863\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }, { name: "maichong", mapping: null, reason: "qinru \u5B58\u50A8\u94FE\uFF08\u5BF9\u5DF2\u5165\u4FB5\u8005\u968F\u673A\u5F03\uFF09\u2014\u2014\u968F qinru \u6682\u7F13" }] }, { name: "\u6B7B\u795E", faction: "shu", hp: 4, gender: "male", skills: [{ name: "xiandan", mapping: "host:wudian_xiandan" }, { name: "yihun", mapping: null, reason: "\u865A\u62DF useCard\uFF08useCard({name:sha}) \u6307\u5B9A\u76EE\u6807\u56DE\u5408\uFF09\u2014\u2014\u865A\u62DF\u7528\u724C\u539F\u8BED\u540C youzhan/huoyu \u7F3A\u53E3\uFF0C2026-10-09 \u6682\u7F13" }, { name: "shouge", mapping: null, reason: "createCard \u6CBB\u7597\u6CE2\uFF08\u724C\u79CD\u8DE8\u5305\u4F9D\u8D56\u7089\u77F3\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "D.Va", faction: "shu", hp: 2, gender: "female", skills: [{ name: "jijia", mapping: "host:wudian_jijia" }, { name: "tuijin", mapping: null, reason: "\u673A\u7532\u8840>0 \u65F6\u8DDD\u79BB>1 \u76EE\u6807\u63A8\u79FB\uFF08tuijin2 \u8DDD\u79BB\u9650\u5B9A\u5B50\u6280\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }, { name: "zihui", mapping: null, reason: "\u81EA\u7206 AOE\uFF08chooseToDiscard n \u5F20\u6216 2 \u706B\u4F24\uFF0C\u673A\u7532\u8840=\u5F03\u724C\u6570\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }, { name: "chongzhuang", mapping: null, reason: "\u91CD\u88C5\u7D2F\u8BA1\uFF08\u673A\u7532\u8840\u5F52\u96F6\u540E\u4F24\u5BB3\u8BA1\u6570 4 \u590D\u6D3B\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u730E\u7A7A", faction: "shu", hp: 3, gender: "female", skills: [{ name: "shanxian", mapping: null, reason: "\u533A\u57DF\u79BB\u573A\uFF08he \u533A\u724C\u5165 storage \u79BB\u573A\u533A\uFF0Cshanhui \u56DE\u5F52\u590D\u6D3B\u542B\u4F4D\u7F6E s \u7684 createCard \u91CD\u9020\uFF09\u2014\u2014\u724C\u5B88\u6052\u7EAA\u5F8B\u7ED3\u6784\u6027\u4F9D\u8D56\uFF0C2026-10-09 \u6682\u7F13" }, { name: "shanhui", mapping: null, reason: "\u968F shanxian \u79BB\u573A\u5B58\u50A8\u94FE\u2014\u20142026-10-09 \u6682\u7F13" }] }], Cl = { source: wl, upstream_path: bl, upstream_commit: xl, pack: $l, cards: jl, host_skills: Sl, deferred: El, skills: zl, generals: Al }, Rl = { package_format_version: 1, namespace: "upstream-noname", package_id: "shaai-xianjian", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [217, 238, 24, 99, 75, 40, 128, 69, 19, 80, 34, 62, 215, 207, 28, 63, 239, 132, 117, 217, 39, 40, 203, 249, 147, 20, 32, 1, 10, 43, 15, 178], asset_hash: [139, 23, 211, 24, 76, 140, 174, 144, 67, 188, 243, 216, 101, 242, 48, 219, 197, 44, 103, 185, 96, 125, 164, 50, 213, 22, 31, 210, 83, 3, 86, 181], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, Nl = [], Pl = [], ql = [], Il = [], Ol = [], Tl = [], Ll = [], Dl = [], Ml = { "zh-CN": { diaobingqianjiang: "\u8C03\u5175\u9063\u5C06", sanjian: "\u4E09\u5C16", wangmeizhike: "\u671B\u6885\u6B62\u6E34", youdishenru: "\u8BF1\u654C\u6DF1\u5165" } }, Ul = { schema_version: 1, package_id: "shaai-xianjian", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "shaai-xianjian:cards", source_hash: [125, 162, 65, 35, 137, 186, 77, 70, 223, 174, 61, 71, 106, 251, 20, 203, 164, 54, 152, 191, 36, 6, 206, 58, 190, 205, 23, 208, 222, 190, 253, 65], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, Vl = { "shaai-xianjian:cards": "assets/cards.json" }, Gl = "gen-1", Xa = { manifest: Rl, modules: Nl, skill_index: Pl, character_index: ql, card_index: Il, trigger_index: Ol, modifier_index: Tl, active_index: Ll, view_as_index: Dl, translations: Ml, asset_manifest: Ul, asset_paths: Vl, compiler_version: Gl }, Bl = "qita/sanguosha/noname/extension/\u6740\u6D77\u62FE\u9057/character/xianjian", Fl = "extension/\u6740\u6D77\u62FE\u9057/character/xianjian/", Hl = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", Wl = { name: "shaai-xianjian-upstream-noname", version: 1, description: "G1 \u7EB5\u5207\uFF1A\u4E0A\u6E38 xianjian \u661F\u7487\uFF08\u7075\u96EA\uFF09", card_prefix: "xj_" }, Kl = [], Jl = [], Ql = [], Yl = [{ name: "lingxue", general: "pal_xingxuan", mapping: "host:xj_lingxue" }, { name: "huimeng", general: "pal_zhaoliner", mapping: "host:xj_huimeng" }, { name: "tianshe", general: "pal_zhaoliner", mapping: "host:xj_tianshe" }, { name: "tianshe2", general: "pal_zhaoliner", mapping: "host:xj_tianshe2" }, { name: "fanling", general: "pal_xia", mapping: "host:xj_fanling" }, { name: "guiyuan", general: "pal_linyueru", mapping: "host:xj_guiyuan" }, { name: "tannang", general: "pal_hanlingsha", mapping: "host:xj_tannang" }, { name: "feixu", general: "pal_wangpengxu", mapping: "host:xj_feixu" }, { name: "feng", general: "pal_leiyuange", mapping: "host:xj_feng" }, { name: "ya", general: "pal_leiyuange", mapping: "host:xj_ya" }, { name: "song", general: "pal_leiyuange", mapping: "host:xj_song" }, { name: "xjyufeng", general: "pal_lixiaoyao", mapping: "host:xj_xjyufeng" }, { name: "xuanyan", general: "pal_xuanxiao", mapping: "host:xj_xuanyan" }, { name: "xuanyan2", general: "pal_xuanxiao", mapping: "host:xj_xuanyan2" }, { name: "xuanyan3", general: "pal_xuanxiao", mapping: "host:xj_xuanyan3" }, { name: "ningbin", general: "pal_xuanxiao", mapping: "host:xj_ningbin" }, { name: "diewu", general: "pal_longkui", mapping: "host:xj_diewu" }, { name: "lingyu", general: "pal_longkui", mapping: "host:xj_lingyu" }, { name: "lueying", general: "pal_muchanglan", mapping: "host:xj_lueying" }, { name: "anwugu", general: "pal_anu", mapping: "host:xj_anwugu" }, { name: "anwugu2", general: "pal_anu", mapping: "host:xj_anwugu2" }, { name: "txianqu", general: "pal_tangyurou", mapping: "host:xj_txianqu" }], Xl = [{ name: "\u661F\u7487", faction: "qun", hp: 3, gender: "male", skills: [{ name: "lingxue", mapping: "host:xj_lingxue" }, { name: "feizhua", mapping: null, reason: "useCard \u6740\u5355\u76EE\u6807+\u7EAF\u8DDD 1 \u5185\u73A9\u5BB6\u8BE2\u95EE\u6362\u4F4D\u2014\u20142026-10-09 \u6682\u7F13" }, { name: "leiyu", mapping: null, reason: "phaseEnd \u96F7\u57DF\u5B58\u50A8\u5217\u8868+\u8DE8\u56DE\u5408\u6807\u8BB0\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u6C88\u6B3A\u971C", faction: "wei", hp: 3, gender: "female", skills: [{ name: "binxin", mapping: "host:xj_binxin" }, { name: "qixia", mapping: null, reason: "\u82B1\u8272\u96C6\u96C6\u9F50 4 \u8272 insertPhase \u989D\u5916\u56DE\u5408\uFF08\u5F15\u64CE\u7F3A\u53E3 ADR0646\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }, { name: "jianzhen", mapping: null, reason: "shaAfter \u9010\u4EBA chooseToUse \u5BF9\u540C\u4E00\u76EE\u6807\u7528\u6740\u8BE2\u95EE\u94FE\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u8D75\u7075\u513F", faction: "wei", hp: 3, gender: "female", skills: [{ name: "huimeng", mapping: "host:xj_huimeng" }, { name: "tianshe", mapping: "host:xj_tianshe" }, { name: "tianshe2", mapping: "host:xj_tianshe2" }] }, { name: "\u7455", faction: "wu", hp: 3, gender: "female", skills: [{ name: "xjzongyu", mapping: null, reason: "\u865A\u62DF useCard \u98DE\u9556\uFF08useCard({name:feibiao}) \u968F\u673A 2 \u654C\uFF09\u2014\u2014\u865A\u62DF\u7528\u724C\u539F\u8BED\u7F3A\u53E3\uFF0C2026-10-09 \u6682\u7F13" }, { name: "fanling", mapping: "host:xj_fanling" }] }, { name: "\u6E29\u6167", faction: "wei", hp: 4, gender: "male", skills: [{ name: "longxiang", mapping: "host:xj_longxiang" }, { name: "huxi", mapping: null, reason: "chooseToUse viewAs sha\uFF08\u65E0\u5B9E\u4F53\u724C\u865A\u62DF\u7528\u724C\uFF09+respondSha\u2014\u2014\u865A\u62DF\u7528\u724C\u539F\u8BED\u7F3A\u53E3\uFF0C2026-10-09 \u6682\u7F13" }] }, { name: "\u6797\u6708\u5982", faction: "wei", hp: 3, gender: "female", skills: [{ name: "guiyuan", mapping: "host:xj_guiyuan" }, { name: "qijian", mapping: null, reason: "phaseDiscardEnd \u5F03\u724C\u2192\u865A\u62DF useCard \u6740\uFF08\u65E0\u5B9E\u4F53\u724C\uFF09\u2014\u2014\u865A\u62DF\u7528\u724C\u539F\u8BED\u7F3A\u53E3\uFF0C2026-10-09 \u6682\u7F13" }] }, { name: "\u97E9\u83F1\u7EB1", faction: "qun", hp: 3, gender: "female", skills: [{ name: "tannang", mapping: "host:xj_tannang" }, { name: "tuoqiao", mapping: null, reason: "\u9ED1\u724C\u5F53\u77F3\u4F1A\u7C89\uFF08\u77F3\u4F1A\u7C89\u4E3A\u53E4\u5251\u5361\u7247\u2014\u2014\u8DE8\u5305\u724C\u79CD\u4F9D\u8D56\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u738B\u84EC\u7D6E", faction: "wei", hp: 3, gender: "female", skills: [{ name: "feixu", mapping: "host:xj_feixu" }, { name: "duxinshu", mapping: null, reason: "\u8BFB\u5FC3\uFF1A\u770B\u76EE\u6807\u624B\u724C chooseCardButton \u9009\u4E00\u5F20\u7528\u81EA\u5DF1\u7684\u624B\u724C\u66FF\u6362\uFF08\u6309\u94AE\u9009\u62E9+\u53CC\u5411\u6362\u724C\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u96F7\u5143\u6208", faction: "qun", hp: 4, gender: "male", skills: [{ name: "feng", mapping: "host:xj_feng" }, { name: "ya", mapping: "host:xj_ya" }, { name: "song", mapping: "host:xj_song" }] }, { name: "\u674E\u900D\u9065", faction: "qun", hp: 4, gender: "male", skills: [{ name: "xjyufeng", mapping: "host:xj_xjyufeng" }, { name: "tianjian", mapping: null, reason: "viewAs \u4E07\u7BAD\u9F50\u53D1\uFF08\u6740\u724C\u5F53 AOE\uFF09+tianjian_discard \u4F24\u5BB3\u5F52\u5C5E\u2014\u2014AOE \u865A\u62DF\u7528\u724C+\u5F52\u5C5E\u7F3A\u53E3\uFF0C2026-10-09 \u6682\u7F13" }] }, { name: "\u7384\u9704", faction: "qun", hp: 4, gender: "male", skills: [{ name: "xuanyan", mapping: "host:xj_xuanyan" }, { name: "xuanyan2", mapping: "host:xj_xuanyan2" }, { name: "xuanyan3", mapping: "host:xj_xuanyan3" }, { name: "ningbin", mapping: "host:xj_ningbin" }, { name: "xfenxin", mapping: null, reason: "changeHp \u4E8B\u4EF6\uFF08\u6389\u8840/\u56DE\u590D\u589E\u91CF\u62BD\u724C\uFF09\u2014\u2014\u539F\u751F\u65E0 changeHp \u4E8B\u4EF6\u79CD\uFF08jijia \u7528 commit_damage \u62E6\u622A\u66FF\u4EE3\u53E3\u5F84\uFF09\uFF0C2026-10-09 \u6682\u7F13" }] }, { name: "\u9F99\u8475", faction: "wu", hp: 3, gender: "female", skills: [{ name: "diewu", mapping: "host:xj_diewu" }, { name: "lingyu", mapping: "host:xj_lingyu" }, { name: "fenxing", mapping: null, reason: "\u53CC\u5F62\u6001\u53D8\u8EAB\uFF08Math.random \u5207\u6362 avatar+\u884D\u751F\u6280\u7EC4\uFF09\u2014\u2014\u7ED3\u6784\u6027\u4F9D\u8D56\uFF0C2026-10-09 \u6682\u7F13" }] }, { name: "\u6155\u957F\u5170", faction: "wei", hp: 3, gender: "female", skills: [{ name: "lueying", mapping: "host:xj_lueying" }, { name: "feixia", mapping: null, reason: "\u865A\u62DF useCard \u6740\uFF08\u968F\u673A\u654C\uFF09\u2014\u2014\u865A\u62DF\u7528\u724C\u539F\u8BED\u7F3A\u53E3\uFF0C2026-10-09 \u6682\u7F13" }] }, { name: "\u963F\u5974", faction: "wu", hp: 3, gender: "female", skills: [{ name: "anwugu", mapping: "host:xj_anwugu" }, { name: "anwugu2", mapping: "host:xj_anwugu2" }, { name: "lingdi", mapping: null, reason: "\u8DDD\u79BB\u9012\u589E\u76EE\u6807+\u82B1\u8272\u53BB\u91CD\u5B58\u50A8\u2014\u2014\u591A\u94A9\u5B58\u50A8\u7CFB\uFF0C2026-10-09 \u6682\u7F13" }] }, { name: "\u5510\u96E8\u67D4", faction: "wu", hp: 3, gender: "female", skills: [{ name: "txianqu", mapping: "host:xj_txianqu" }, { name: "qiongguang", mapping: null, reason: "\u7EAF\u8868\u73B0\u6280\uFF08line/expose/delay\uFF0C\u65E0\u673A\u5236\u672C\u4F53\uFF09\u2014\u2014\u8868\u73B0\u5C42 noop\uFF0C2026-10-09 \u767B\u8BB0\u4E0D\u8F6C" }] }, { name: "\u59DC\u57CE", faction: "qun", hp: 4, gender: "male", skills: [{ name: "yanzhan", mapping: null, reason: "\u7EA2\u724C\u8F6C\u706B\u6740 usable:1+\u76EE\u6807 yanzhan3 \u54CD\u5E94\u82B1\u8272 mod+yanzhan2\u21924 \u6740\u9650\u6B21 mod\u2014\u2014viewAs/mod \u7CFB\uFF0C2026-10-09 \u6682\u7F13" }, { name: "fenshi", mapping: "host:xj_fenshi" }, { name: "longhuo", mapping: "host:xj_longhuo", reason_ref: "\u884D\u751F\u6280\u5E38\u9A7B\u5B9E\u4F8B+PlayerState.jiangcheng_longhuo \u95E8\u63A7\u5EFA\u6A21\uFF08\u4E0A\u6E38 addSkill \u52A8\u6001\u83B7\u5F97\uFF09\uFF0C\u89C1\u89C4\u5219\u57FA\u51C6\u767B\u8BB0" }] }, { name: "\u666F\u5929", faction: "wu", hp: 3, gender: "male", skills: [{ name: "sajin", mapping: "host:xj_sajin" }, { name: "jtjubao", mapping: "host:xj_jtjubao" }] }, { name: "\u9F99\u5E7D", faction: "wei", hp: 4, gender: "male", skills: [{ name: "yuexing", mapping: "host:xj_yuexing" }, { name: "minsha", mapping: "host:xj_minsha" }] }, { name: "\u660E\u7EE3", faction: "shu", hp: 3, gender: "female", skills: [{ name: "linghuo", mapping: "host:xj_linghuo" }, { name: "guijin", mapping: "host:xj_guijin" }, { name: "chengxin", mapping: "host:xj_chengxin" }] }, { name: "\u4E91\u5929\u6CB3", faction: "wu", hp: 4, gender: "male", skills: [{ name: "longxi", mapping: null, reason: "chooseToRespond/chooseToUse \u54CD\u5E94\u8BE2\u95EE\u524D\u7F6E\u94A9\u7F3A\u53E3\uFF08\u8BE2\u95EE\u94FE\u6539\u5199\u65CF\uFF09\uFF0C2026-10-09 \u6682\u7F13" }, { name: "zhuyue", mapping: "host:xj_zhuyue" }, { name: "guanri", mapping: "host:xj_guanri" }] }, { name: "\u7D2B\u8431", faction: "shu", hp: 3, gender: "female", skills: [{ name: "shuiyun", mapping: "host:xj_shuiyun" }, { name: "shuiyun5", mapping: "host:xj_shuiyun5", reason_ref: "shuiyun \u5B50\u6280\u5E38\u9A7B\u5B9E\u4F8B\u5316\uFF08\u4E0A\u6E38 subSkill \u4E0D\u5217\u5C06\u8868\uFF09\uFF0C\u6FD2\u6B7B\u6551\u63F4\u901A\u9053\u9996\u4F8B" }, { name: "wangyou", mapping: "host:xj_wangyou" }, { name: "wangyou2", mapping: "host:xj_wangyou2", reason_ref: "wangyou \u5B50\u6280\u5E38\u9A7B\u5B9E\u4F8B\u5316\uFF08silent \u6807\u8BB0\uFF09" }, { name: "changnian", mapping: "host:xj_changnian" }] }, { name: "\u5C0F\u86EE", faction: "shu", hp: 3, gender: "female", skills: [{ name: "anwugu", mapping: "host:xj_anwugu", reason_ref: "\u4E0E\u963F\u5974\u540C\u6280\u2014\u2014mapping \u590D\u7528" }, { name: "lingquan", mapping: "host:xj_lingquan" }, { name: "shenwu", mapping: "host:xj_shenwu" }] }, { name: "\u96EA\u89C1", faction: "shu", hp: 3, gender: "female", skills: [{ name: "xshuangren", mapping: "host:xj_xshuangren" }, { name: "shenmu", mapping: "host:xj_shenmu" }, { name: "duci", mapping: "host:xj_duci" }, { name: "xshuangren2", mapping: "host:xj_xshuangren2", reason_ref: "xshuangren \u5B50\u6280\u5E38\u9A7B\u5B9E\u4F8B\u5316\uFF08dieBegin \u5F03\u7F6E\uFF09" }] }, { name: "\u67F3\u68A6\u7483", faction: "shu", hp: 3, gender: "female", skills: [{ name: "tianxian", mapping: "host:xj_tianxian" }, { name: "runxin", mapping: "host:xj_runxin" }, { name: "xjzhimeng", mapping: "host:xj_xjzhimeng" }, { name: "xjzhimeng2", mapping: "host:xj_xjzhimeng2", reason_ref: "xjzhimeng \u5B50\u6280\u5E38\u9A7B\u5B9E\u4F8B\u5316\uFF08\u76EE\u6807 runtime \u6388\u6280 def \u6E90\uFF09" }, { name: "changnian2", mapping: "host:xj_changnian2", reason_ref: "changnian \u5B50\u6280\u5E38\u9A7B\u5B9E\u4F8B\u5316\uFF08runtime \u6388\u6280 def \u6E90\uFF09" }] }, { name: "\u5357\u5BAB\u714C", faction: "wei", hp: 3, gender: "male", skills: [{ name: "zhaoyao", mapping: "host:xj_zhaoyao" }, { name: "sheling", mapping: "host:xj_sheling" }, { name: "zhangmu", mapping: "host:xj_zhangmu" }] }, { name: "\u738B\u5C0F\u864E", faction: "qun", hp: 4, gender: "male", skills: [{ name: "husha_count", mapping: "host:xj_husha_count", reason_ref: "husha \u5B50\u6280\u5E38\u9A7B\u5B9E\u4F8B\u5316\uFF08source damageEnd \u96C6\u6807\u8BB0\uFF09" }, { name: "husha", mapping: "host:xj_husha", reason_ref: "\u5143\u7D20\u6BC1\u706D\uFF083 \u679A\u9009\u9879\uFF09\u724C\u79CD\u4E0D\u5B58\u5728\u76EE\u5F55\u767B\u8BB0\u4E0D\u8F6C" }] }, { name: "\u6708\u7426", faction: "wei", hp: 3, gender: "female", skills: [{ name: "tianwu", mapping: null, reason: "\u7EAF\u8868\u73B0\u6280\uFF08getDebuff+addTempSkill tianwu2 \u7A7A\u58F3\uFF09\u2014\u2014\u8868\u73B0\u5C42 noop\uFF0C2026-10-09 \u767B\u8BB0\u4E0D\u8F6C" }, { name: "liguang", mapping: "host:xj_liguang" }, { name: "shiying", mapping: "host:xj_shiying" }] }, { name: "\u590F\u4FAF\u747E\u8F69", faction: "wei", hp: 3, gender: "male", skills: [{ name: "xuanmo", mapping: "host:xj_xuanmo" }, { name: "danqing", mapping: null, reason: "\u56DB\u82B1\u8272\u6536\u96C6\u2192getBuff(false) \u7EAF\u8868\u73B0\u76EE\u6807\u9009\u62E9\uFF0C\u65E0\u673A\u5236\u672C\u4F53\u2014\u2014\u8868\u73B0\u5C42\u767B\u8BB0\u4E0D\u8F6C\uFF0C2026-10-09" }] }, { name: "\u82CF\u5A9A", faction: "shu", hp: 3, gender: "female", skills: [{ name: "sheying", mapping: "host:xj_sheying" }, { name: "dujiang", mapping: null, reason: "\u83B7\u5F97\u3010\u6BD2\u3011\u724C\u2014\u2014\u6BD2\u724C\u79CD\u4E0D\u5728\u76EE\u5F55\uFF08createCard \u5B9E\u4F53\u53D1\u73B0\u4EA6\u4E0D\u53EF\u7528\uFF09\uFF0C2026-10-09 \u6682\u7F13" }, { name: "huahu", mapping: "host:xj_huahu" }] }, { name: "\u4ED9\u537F", faction: "qun", hp: 4, gender: "male", skills: [{ name: "xtanxi", mapping: "host:xj_xtanxi" }, { name: "xiaoyue", mapping: null, reason: "roundStart \u4E8B\u4EF6\u7F3A\u53E3+\u81EA\u52A8\u865A\u62DF\u7528\u724C\uFF08useCard \u6740+\u65E0\u8DDD\u79BB\u9650\u5236 mod\uFF09\u590D\u5408\u7F3A\u53E3\uFF0C2026-10-09 \u6682\u7F13" }] }, { name: "\u7F57\u62DB\u5AE3", faction: "shu", hp: 4, gender: "female", skills: [{ name: "fenglue", mapping: "host:xj_fenglue" }, { name: "tanhua", mapping: null, reason: "Recover \u524D\u7F6E\u7A97\u53E3\u7F3A\u53E3\uFF08\u56DE\u590D\u6539\u4E3A\u6478\u724C\u9700\u56DE\u590D\u53D6\u6D88\u8BED\u4E49\uFF09\u2014\u20142026-10-09 \u6682\u7F13" }] }, { name: "\u59DC\u4E91\u51E1", faction: "wei", hp: 4, gender: "male", skills: [{ name: "xunying", mapping: "host:xj_xunying" }, { name: "liefeng", mapping: "host:xj_liefeng" }, { name: "tianjian", mapping: "host:xj_tianjian", reason_ref: "liefeng runtime \u6388\u6280\u76EE\u6807 def \u6E90\uFF08\u76EE\u5F55\u7ECF mapping \u6CE8\u518C\uFF09" }] }, { name: "\u6155\u5BB9\u7D2B\u82F1", faction: "wei", hp: 4, gender: "male", skills: [{ name: "poyun", mapping: "host:xj_poyun" }, { name: "qianfang", mapping: "host:xj_qianfang" }, { name: "xuanning", mapping: null, reason: "\u8BA1\u6570\u6807\u8BB0\u83B7\u5F97\u8DEF\u5F84\u4E0A\u6E38\u7F3A\u5931\uFF08storage.xuanning \u65E0\u6280\u80FD\u5199\u5165\uFF09\u2014\u2014\u539F\u751F\u4EC5\u652F\u6301\u8C03\u8BD5/\u5BBF\u4E3B\u6CE8\u5165\uFF0C2026-10-09 \u767B\u8BB0\u5DEE\u5F02" }] }, { name: "\u8D8A\u4ECA\u671D", faction: "wei", hp: 4, gender: "male", skills: [{ name: "ywuhun", mapping: null, reason: "\u56DE\u5408\u5F00\u59CB\u524D\u989D\u5916\u56DE\u5408+\u56DE\u5408\u540E\u590D\u539F\u573A\u4E0A\u53CA\u724C\u5806\u5168\u90E8\u724C\u2014\u2014\u53CC\u5F15\u64CE\u7F3A\u53E3\uFF08\u989D\u5916\u56DE\u5408=ADR0646 \u767B\u8BB0\uFF09\uFF0C2026-10-10 \u6682\u7F13" }, { name: "xjyingfeng", mapping: "host:xj_xjyingfeng" }] }, { name: "\u5C45\u5341\u65B9", faction: "shu", hp: 3, gender: "male", skills: [{ name: "xiepan", mapping: "host:xj_xiepan" }, { name: "yujia", mapping: null, reason: "\u673A\u5173\u724C\u79CD\u4E0D\u5728\u76EE\u5F55\uFF08\u89C2\u770B/\u8BA1\u6570\u4F9D\u8D56\u673A\u5173\u724C\u5B9E\u4F53\u6C60\uFF09\u2014\u2014dujiang \u540C\u65CF\u7F3A\u53E3\uFF0C2026-10-10 \u6682\u7F13" }, { name: "yanshi", mapping: null, reason: "\u673A\u5173\u724C\u79CD\u4E0D\u5728\u76EE\u5F55\uFF08\u7D2F\u8BA1\u4F7F\u7528\u673A\u5173\u724C\u8BA1\u6570+\u968F\u673A\u88C5\u5907\u586B\u6EE1\u53EF\u6302\u5B9E\u4F53\u53D1\u73B0\uFF0C\u524D\u7F6E=\u673A\u5173\u724C\u79CD\u57FA\u5EFA\uFF09\uFF0C2026-10-10 \u6682\u7F13" }] }, { name: "\u957F\u537F", faction: "wei", hp: 4, gender: "male", skills: [{ name: "luanjian", mapping: "host:xj_luanjian" }, { name: "luanjian2", mapping: "host:xj_luanjian2", reason_ref: "luanjian \u589E\u4F24 subskill\uFF08DamageBefore forced 50%\uFF0C\u8F7D\u4F53\u724C\u951A\u5B9A\uFF09" }, { name: "ctianfu", mapping: "host:xj_ctianfu" }, { name: "ctianfu2", mapping: "host:xj_ctianfu2", reason_ref: "\u5929\u7B26\xB7\u53CD\u566C subskill\uFF08Damage \u4F24\u5BB3\u540E\u7A97\uFF0C\u6302\u9644\u6807\u8BB0+\u5B9E\u4F8B\u6210\u5BF9\uFF09" }, { name: "ctianfu3", mapping: "host:xj_ctianfu3", reason_ref: "\u5929\u7B26\xB7\u6E05\u6302 subskill\uFF08Death \u7A97\u6E05\u7406\uFF09" }] }], Zl = { source: Bl, upstream_path: Fl, upstream_commit: Hl, pack: Wl, cards: Kl, host_skills: Jl, deferred: Ql, skills: Yl, generals: Xl }, eu = { package_format_version: 1, namespace: "upstream-noname", package_id: "shahai-gujian", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [174, 19, 1, 143, 26, 75, 62, 188, 159, 67, 126, 42, 118, 12, 107, 113, 203, 163, 207, 147, 194, 208, 143, 29, 74, 238, 216, 95, 37, 52, 158, 6], asset_hash: [64, 167, 122, 159, 149, 48, 209, 126, 144, 16, 221, 246, 166, 188, 56, 151, 231, 155, 253, 19, 203, 98, 144, 68, 6, 23, 156, 243, 43, 177, 22, 35], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, nu = [], au = [], tu = [], iu = [], su = [], ou = [], ru = [], lu = [], uu = { "zh-CN": { bingpotong: "\u51B0\u9B44\u901A", chunbing: "\u6625\u997C", dinvxuanshuang: "\u5E1D\u5973\u7384\u971C", feibiao: "\u98DE\u9556", gjyuheng: "\u7389\u8861", gudonggeng: "\u5495\u549A\u7FB9", gujian: "\u53E4\u5251\u5947\u8C2D", heilonglinpian: "\u9ED1\u9F99\u9CDE\u7247", jinlianzhu: "\u91D1\u83B2\u70DB", liufengsan: "\u516D\u98CE\u6563", liutouge: "\u786B\u5934\u9601", liyutang: "\u9CA4\u9C7C\u6C64", longxugou: "\u9F99\u987B\u94A9", luyugeng: "\u5364\u7389\u7FB9", mapodoufu: "\u9EBB\u5A46\u8C46\u8150", mianlijinzhen: "\u9762\u91CC\u91D1\u9488", mizhilianou: "\u871C\u94FE\u85D5", molicha: "\u9EBB\u8336", mutoumianju: "\u6728\u5934\u9762\u5177", qiankunbiao: "\u4E7E\u5764\u9556", qingtuan: "\u9752\u56E2", shatang: "\u7802\u7CD6", shenhuofeiya: "\u795E\u706B\u98DE\u9E26", shihuifen: "\u77F3\u4F1A\u7C89", shujinsan: "\u8212\u7B4B\u6563", tanhuadong: "\u644A\u9EC4\u51BB", xiajiao: "\u83F1\u89D2", yougeng: "\u6709\u66F4", yuanbaorou: "\u5143\u5B9D\u8089", yunvyuanshen: "\u7389\u5973\u5143\u795E", ziyangdan: "\u7D2B\u9633\u4E39" } }, cu = { schema_version: 1, package_id: "shahai-gujian", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "shahai-gujian:cards", source_hash: [59, 14, 97, 152, 140, 199, 211, 59, 2, 113, 112, 213, 241, 164, 249, 98, 134, 209, 174, 76, 85, 135, 152, 123, 139, 5, 42, 72, 174, 40, 183, 10], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, du = { "shahai-gujian:cards": "assets/cards.json" }, mu = "gen-1", Za = { manifest: eu, modules: nu, skill_index: au, character_index: tu, card_index: iu, trigger_index: su, modifier_index: ou, active_index: ru, view_as_index: lu, translations: uu, asset_manifest: cu, asset_paths: du, compiler_version: mu }, pu = "noname", hu = "apps/core/extension/\u6740\u6D77\u62FE\u9057/card/gujian/", gu = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", fu = { name: "shahai-gujian-upstream-noname", version: 1, description: "G1 \u6740\u6D77\u62FE\u9057\xB7\u53E4\u5251\u5207\u7247", card_prefix: "gj_" }, _u = [{ name: "shujinsan", kind: "shujinsan", suit: "diamond", rank: 6 }, { name: "shujinsan", kind: "shujinsan", suit: "spade", rank: 2 }, { name: "mianlijinzhen", kind: "mianlijinzhen", suit: "spade", rank: 1 }, { name: "feibiao", kind: "feibiao", suit: "club", rank: 8 }, { name: "feibiao", kind: "feibiao", suit: "diamond", rank: 9 }, { name: "jinlianzhu", kind: "jinlianzhu", suit: "heart", rank: 9 }, { name: "jinlianzhu", kind: "jinlianzhu", suit: "spade", rank: 7 }, { name: "tanhuadong", kind: "tanhuadong", suit: "spade", rank: 2 }, { name: "yougeng", kind: "yougeng", suit: "heart", rank: 12 }, { name: "qingtuan", kind: "qingtuan", suit: "spade", rank: 3 }, { name: "yuanbaorou", kind: "yuanbaorou", suit: "spade", rank: 8 }, { name: "gudonggeng", kind: "gudonggeng", suit: "heart", rank: 8 }, { name: "mapodoufu", kind: "mapodoufu", suit: "heart", rank: 4 }, { name: "chunbing", kind: "chunbing", suit: "club", rank: 3 }, { name: "liyutang", kind: "liyutang", suit: "heart", rank: 1 }, { name: "molicha", kind: "molicha", suit: "club", rank: 1 }, { name: "xiajiao", kind: "xiajiao", suit: "diamond", rank: 6 }, { name: "mizhilianou", kind: "mizhilianou", suit: "diamond", rank: 4 }, { name: "luyugeng", kind: "luyugeng", suit: "club", rank: 11 }, { name: "bingpotong", kind: "bingpotong", suit: "spade", rank: 3 }, { name: "bingpotong", kind: "bingpotong", suit: "club", rank: 12 }, { name: "qiankunbiao", kind: "qiankunbiao", suit: "spade", rank: 9 }, { name: "qiankunbiao", kind: "qiankunbiao", suit: "club", rank: 13 }, { name: "shenhuofeiya", kind: "shenhuofeiya", suit: "diamond", rank: 9 }, { name: "longxugou", kind: "longxugou", suit: "spade", rank: 7 }, { name: "ziyangdan", kind: "ziyangdan", suit: "diamond", rank: 6 }, { name: "ziyangdan", kind: "ziyangdan", suit: "heart", rank: 1 }, { name: "yunvyuanshen", kind: "yunvyuanshen", suit: "heart", rank: 13 }, { name: "shihuifen", kind: "shihuifen", suit: "club", rank: 5 }, { name: "shihuifen", kind: "shihuifen", suit: "club", rank: 1 }, { name: "shihuifen", kind: "shihuifen", suit: "spade", rank: 13 }, { name: "liufengsan", kind: "liufengsan", suit: "club", rank: 6 }, { name: "liufengsan", kind: "liufengsan", suit: "club", rank: 3 }, { name: "shatang", kind: "shatang", suit: "heart", rank: 13 }, { name: "heilonglinpian", kind: "heilonglinpian", suit: "spade", rank: 2 }, { name: "liutouge", kind: "liutouge", suit: "heart", rank: 6 }, { name: "liutouge", kind: "liutouge", suit: "club", rank: 6 }, { name: "mutoumianju", kind: "mutoumianju", suit: "club", rank: 4 }, { name: "dinvxuanshuang", kind: "dinvxuanshuang", suit: "heart", rank: 9 }, { name: "gjyuheng", kind: "gjyuheng", suit: "spade", rank: 7 }], vu = [], yu = { source: pu, upstream_path: hu, upstream_commit: gu, pack: fu, cards: _u, deferred: vu }, ku = { package_format_version: 1, namespace: "upstream-noname", package_id: "shahai-gwent", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [11, 210, 86, 154, 147, 71, 161, 144, 244, 225, 236, 145, 131, 209, 115, 183, 233, 66, 140, 230, 24, 60, 18, 155, 254, 3, 178, 198, 44, 142, 171, 205], asset_hash: [81, 129, 91, 229, 139, 77, 173, 158, 200, 141, 186, 65, 162, 83, 196, 174, 250, 135, 67, 199, 255, 82, 208, 132, 191, 127, 138, 83, 57, 29, 198, 231], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, wu = [], bu = [], xu = [], $u = [], ju = [], Su = [], Eu = [], zu = [], Au = { "zh-CN": { aozuzhilei: "\u5965\u7956\u4E4B\u96F7", baoxueyaoshui: "\u66B4\u96EA\u836F\u6C34", birinongwu: "\u78A7\u65E5\u6D53\u96FE", ciguhanshuang: "\u523A\u9AA8\u5BD2\u971C", gwent: "gwent", kunenfayin: "\u6606\u6069\u6CD5\u5370", poxiao: "\u7834\u6653", qinpendayu: "\u503E\u76C6\u5927\u96E8", shanbengshu: "\u5C71\u5D29\u672F", wenyi: "\u761F\u75AB", yanziyaoshui: "\u71D5\u5B50\u836F\u6C34", zhihuanjun: "\u7F6E\u6362\u519B", zumoshoukao: "\u7956\u58A8\u624B\u94E0" } }, Cu = { schema_version: 1, package_id: "shahai-gwent", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "shahai-gwent:cards", source_hash: [166, 48, 247, 15, 211, 138, 196, 50, 148, 242, 250, 36, 174, 15, 79, 156, 9, 156, 56, 156, 45, 66, 129, 66, 55, 226, 111, 253, 188, 124, 214, 218], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, Ru = { "shahai-gwent:cards": "assets/cards.json" }, Nu = "gen-1", et = { manifest: ku, modules: wu, skill_index: bu, character_index: xu, card_index: $u, trigger_index: ju, modifier_index: Su, active_index: Eu, view_as_index: zu, translations: Au, asset_manifest: Cu, asset_paths: Ru, compiler_version: Nu }, Pu = "noname", qu = "apps/core/extension/\u6740\u6D77\u62FE\u9057/card/gwent/", Iu = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", Ou = { name: "shahai-gwent-upstream-noname", version: 1, description: "G1 \u6740\u6D77\u62FE\u9057\xB7gwent \u5207\u7247", card_prefix: "gw_" }, Tu = [{ name: "yanziyaoshui", kind: "yanziyaoshui", suit: "heart", rank: 8 }, { name: "yanziyaoshui", kind: "yanziyaoshui", suit: "club", rank: 7 }, { name: "aozuzhilei", kind: "aozuzhilei", suit: "spade", rank: 9 }, { name: "aozuzhilei", kind: "aozuzhilei", suit: "club", rank: 3 }, { name: "wenyi", kind: "wenyi", suit: "club", rank: 5 }, { name: "zhihuanjun", kind: "zhihuanjun", suit: "club", rank: 1 }, { name: "zhihuanjun", kind: "zhihuanjun", suit: "spade", rank: 3 }, { name: "baoxueyaoshui", kind: "baoxueyaoshui", suit: "heart", rank: 4 }, { name: "baoxueyaoshui", kind: "baoxueyaoshui", suit: "spade", rank: 8 }, { name: "qinpendayu", kind: "qinpendayu", suit: "diamond", rank: 5 }, { name: "qinpendayu", kind: "qinpendayu", suit: "club", rank: 7 }, { name: "birinongwu", kind: "birinongwu", suit: "spade", rank: 9 }, { name: "birinongwu", kind: "birinongwu", suit: "heart", rank: 13 }, { name: "ciguhanshuang", kind: "ciguhanshuang", suit: "diamond", rank: 11 }, { name: "ciguhanshuang", kind: "ciguhanshuang", suit: "club", rank: 7 }, { name: "kunenfayin", kind: "kunenfayin", suit: "spade", rank: 11 }, { name: "shanbengshu", kind: "shanbengshu", suit: "spade", rank: 8 }, { name: "poxiao", kind: "poxiao", suit: "heart", rank: 7 }, { name: "poxiao", kind: "poxiao", suit: "diamond", rank: 4 }, { name: "zumoshoukao", kind: "zumoshoukao", suit: "spade", rank: 2 }, { name: "zumoshoukao", kind: "zumoshoukao", suit: "club", rank: 5 }], Lu = [], Du = { source: Pu, upstream_path: qu, upstream_commit: Iu, pack: Ou, cards: Tu, deferred: Lu }, Mu = { package_format_version: 1, namespace: "upstream-noname", package_id: "shahai-hearth", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [222, 175, 195, 28, 126, 229, 205, 192, 134, 104, 195, 255, 215, 43, 199, 51, 28, 232, 140, 32, 174, 71, 203, 169, 230, 49, 163, 84, 168, 127, 59, 118], asset_hash: [19, 173, 66, 158, 166, 22, 115, 40, 86, 41, 2, 15, 142, 182, 166, 113, 120, 248, 54, 209, 90, 199, 119, 75, 143, 71, 43, 245, 55, 14, 41, 93], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, Uu = [], Vu = [], Gu = [], Bu = [], Fu = [], Hu = [], Wu = [], Ku = [], Ju = { "zh-CN": { chuansongmen: "\u4F20\u9001\u95E8", dunpaigedang: "\u76FE\u724C\u683C\u6321", hearth: "\u7089\u77F3\u4F20\u8BF4", jihuocard: "\u6025\u8017\u5361", linghunzhihuo: "\u7075\u9B42\u4E4B\u706B", shandianjian: "\u95EA\u7535\u5251", shenenshu: "\u795E\u6069\u672F", tanshezhiren: "\u5766\u86C7\u81F3\u4EBA", xingjiegoutong: "\u661F\u754C\u6C9F\u901A", yuansuhuimie: "\u5143\u7D20\u6BC1\u706D", zhaomingdan: "\u7167\u660E\u5F39", zhiliaobo: "\u6CBB\u7597\u6CE2" } }, Qu = { schema_version: 1, package_id: "shahai-hearth", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "shahai-hearth:cards", source_hash: [224, 12, 104, 161, 208, 213, 217, 129, 88, 249, 189, 119, 106, 114, 77, 244, 33, 155, 19, 99, 87, 24, 224, 112, 101, 175, 244, 191, 178, 203, 181, 243], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, Yu = { "shahai-hearth:cards": "assets/cards.json" }, Xu = "gen-1", nt = { manifest: Mu, modules: Uu, skill_index: Vu, character_index: Gu, card_index: Bu, trigger_index: Fu, modifier_index: Hu, active_index: Wu, view_as_index: Ku, translations: Ju, asset_manifest: Qu, asset_paths: Yu, compiler_version: Xu }, Zu = "noname", ec = "apps/core/extension/\u6740\u6D77\u62FE\u9057/card/hearth/", nc = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", ac = { name: "shahai-hearth-upstream-noname", version: 1, description: "G1 \u7089\u77F3\u4F20\u8BF4\u5207\u7247", card_prefix: "ht_" }, tc = [{ name: "linghunzhihuo", kind: "linghunzhihuo", suit: "diamond", rank: 2 }, { name: "zhaomingdan", kind: "zhaomingdan", suit: "diamond", rank: 5 }, { name: "zhaomingdan", kind: "zhaomingdan", suit: "heart", rank: 10 }, { name: "jihuocard", kind: "jihuocard", suit: "diamond", rank: 2 }, { name: "shandianjian", kind: "leisha", suit: "club", rank: 3 }, { name: "shandianjian", kind: "leisha", suit: "spade", rank: 1 }, { name: "shenenshu", kind: "jihuocard", suit: "heart", rank: 2 }, { name: "shenenshu", kind: "jihuocard", suit: "diamond", rank: 12 }, { name: "dunpaigedang", kind: "dunpaigedang", suit: "club", rank: 3 }, { name: "zhiliaobo", kind: "zhiliaobo", suit: "club", rank: 7 }, { name: "zhiliaobo", kind: "zhiliaobo", suit: "spade", rank: 1 }, { name: "yuansuhuimie", kind: "yuansuhuimie", suit: "spade", rank: 13 }, { name: "xingjiegoutong", kind: "xingjiegoutong", suit: "spade", rank: 13 }, { name: "chuansongmen", kind: "chuansongmen", suit: "diamond", rank: 2 }, { name: "chuansongmen", kind: "chuansongmen", suit: "heart", rank: 2 }, { name: "tanshezhiren", kind: "tanshezhiren", suit: "diamond", rank: 2 }, { name: "shijieshu", kind: "shijieshu", suit: "spade", rank: 7 }], ic = [], sc = { source: Zu, upstream_path: ec, upstream_commit: nc, pack: ac, cards: tc, deferred: ic }, oc = { package_format_version: 1, namespace: "upstream-noname", package_id: "shahai-mtg", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [100, 23, 190, 115, 240, 231, 83, 4, 12, 207, 180, 163, 203, 162, 240, 254, 137, 75, 251, 157, 145, 232, 234, 173, 222, 68, 39, 243, 228, 135, 129, 98], asset_hash: [138, 246, 212, 134, 176, 90, 183, 36, 117, 169, 120, 78, 6, 33, 28, 70, 108, 98, 68, 51, 36, 115, 35, 162, 65, 214, 7, 150, 89, 14, 168, 195], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, rc = [], lc = [], uc = [], cc = [], dc = [], mc = [], pc = [], hc = [], gc = { schema_version: 1, package_id: "shahai-mtg", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "shahai-mtg:cards", source_hash: [90, 88, 252, 38, 173, 223, 90, 144, 111, 137, 77, 35, 246, 175, 221, 252, 104, 231, 28, 131, 135, 65, 225, 157, 151, 243, 9, 35, 13, 249, 156, 220], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, fc = { "shahai-mtg:cards": "assets/cards.json" }, _c = "test", at = { manifest: oc, modules: rc, skill_index: lc, character_index: uc, card_index: cc, trigger_index: dc, modifier_index: mc, active_index: pc, view_as_index: hc, asset_manifest: gc, asset_paths: fc, compiler_version: _c }, vc = "qita/sanguosha/noname/extension/\u6740\u6D77\u62FE\u9057/card/mtg", yc = "extension/\u6740\u6D77\u62FE\u9057/card/mtg/", kc = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", wc = { name: "shahai-mtg-upstream-noname", version: 1, description: "G1 \u8F6C\u6362\uFF1A\u4E0A\u6E38 mtg \u9996\u5207\u7247\uFF08\u9F99\u9885\u5C71\u5CF0\uFF09", card_prefix: "mtg_" }, bc = [{ name: "mtg_longlushanfeng", kind: "longlushanfeng", suit: "club", rank: 10 }, { name: "mtg_haidao", kind: "haidao", suit: "diamond", rank: 5 }, { name: "mtg_shuimomuxue", kind: "shuimomuxue", suit: "spade", rank: 11 }, { name: "mtg_feixu", kind: "feixu", suit: "club", rank: 8 }, { name: "mtg_yixialan", kind: "yixialan", suit: "club", rank: 12 }, { name: "mtg_shamolvzhou", kind: "shamolvzhou", suit: "heart", rank: 6 }, { name: "mtg_linzhongjianta", kind: "linzhongjianta", suit: "heart", rank: 6 }, { name: "mtg_bingheyaosai", kind: "bingheyaosai", suit: "diamond", rank: 5 }, { name: "mtg_youlin", kind: "youlin", suit: "club", rank: 10 }, { name: "mtg_lindixiliu", kind: "lindixiliu", suit: "spade", rank: 11 }], xc = [], $c = [{ name: "mtg_duzhao", reason: "\u4F9D\u8D56 xianxia \u5305\u3010\u6BD2\u3011\u5361\uFF08\u8DE8\u5305\u724C\u79CD\u4F9D\u8D56\u540C tuoqiao \u5148\u4F8B\uFF09\u2014\u20142026-10-11 \u6682\u7F13\uFF0C\u968F\u6BD2\u5361\u6E90\u8F6C\u6362\u89E3\u9501" }, { name: "mtg_cangbaohaiwan", reason: "\u4F9D\u8D56\u6D3E\u751F\u5305\u9009\u62E9 UI\uFF08derivation \u5305\u5217\u8868\u4EA4\u4E92\u9009\u724C\uFF09\u2014\u2014\u539F\u751F\u65E0\u8BE5\u673A\u5236\uFF0C2026-10-11 \u6682\u7F13" }], jc = { source: vc, upstream_path: yc, upstream_commit: kc, pack: wc, cards: bc, host_skills: xc, deferred: $c }, Sc = { package_format_version: 1, namespace: "upstream-noname", package_id: "shahai-swd", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [49, 204, 234, 152, 242, 141, 74, 177, 190, 148, 159, 91, 147, 112, 4, 119, 125, 19, 159, 58, 106, 197, 84, 110, 16, 191, 141, 30, 209, 141, 195, 127], asset_hash: [199, 119, 92, 146, 166, 89, 110, 115, 216, 130, 121, 121, 62, 47, 65, 102, 152, 166, 25, 69, 169, 82, 236, 165, 122, 112, 127, 20, 153, 39, 8, 196], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, Ec = [], zc = [], Ac = [], Cc = [], Rc = [], Nc = [], Pc = [], qc = [], Ic = { schema_version: 1, package_id: "shahai-swd", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "shahai-swd:cards", source_hash: [70, 23, 137, 164, 203, 28, 38, 181, 129, 120, 81, 208, 138, 2, 175, 224, 228, 196, 230, 118, 108, 225, 193, 254, 181, 155, 126, 170, 60, 93, 71, 82], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, Oc = { "shahai-swd:cards": "assets/cards.json" }, Tc = "test", tt = { manifest: Sc, modules: Ec, skill_index: zc, character_index: Ac, card_index: Cc, trigger_index: Rc, modifier_index: Nc, active_index: Pc, view_as_index: qc, asset_manifest: Ic, asset_paths: Oc, compiler_version: Tc }, Lc = "noname", Dc = "apps/core/extension/\u6740\u6D77\u62FE\u9057/card/swd/", Mc = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", Uc = { name: "shahai-swd-upstream-noname", version: 1, description: "G1 \u8F6C\u6362\uFF1A\u4E0A\u6E38 swd \u9996\u5207\u7247\uFF08\u62A4\u7B26\uFF09", card_prefix: "swd_", init_hook: "swd_jiqi_held" }, Vc = [{ name: "swd_hufu", kind: "hufu", suit: "spade", rank: 1 }, { name: "swd_hufu2", kind: "hufu", suit: "heart", rank: 1 }, { name: "swd_liuxinghuoyu1", kind: "liuxinghuoyu", suit: "diamond", rank: 3 }, { name: "swd_liuxinghuoyu2", kind: "liuxinghuoyu", suit: "heart", rank: 6 }, { name: "swd_liuxinghuoyu3", kind: "liuxinghuoyu", suit: "heart", rank: 9 }, { name: "swd_caoyao1", kind: "caoyao", suit: "club", rank: 3 }, { name: "swd_caoyao2", kind: "caoyao", suit: "club", rank: 4 }, { name: "swd_caoyao3", kind: "caoyao", suit: "club", rank: 5 }, { name: "swd_chilongya", kind: "chilongya", suit: "diamond", rank: 3 }, { name: "swd_tianxianjiu1", kind: "tianxianjiu", suit: "heart", rank: 9 }, { name: "swd_tianxianjiu2", kind: "tianxianjiu", suit: "diamond", rank: 2 }, { name: "swd_xixueguizhihuan", kind: "xixueguizhihuan", suit: "spade", rank: 5 }, { name: "swd_guiyoujie1", kind: "guiyoujie", suit: "spade", rank: 3 }, { name: "swd_guiyoujie2", kind: "guiyoujie", suit: "club", rank: 9 }, { name: "swd_shentoumianju", kind: "shentoumianju", suit: "club", rank: 6 }, { name: "swd_xingjunyan", kind: "xingjunyan", suit: "spade", rank: 7 }, { name: "swd_qinglonglingzhu", kind: "qinglonglingzhu", suit: "spade", rank: 2 }, { name: "swd_zhufangshenshi", kind: "zhufangshenshi", suit: "spade", rank: 4 }, { name: "swd_fengxueren", kind: "fengxueren", suit: "club", rank: 1 }, { name: "swd_yufulu", kind: "yufulu", suit: "spade", rank: 6 }, { name: "swd_yangpijuan", kind: "yangpijuan", suit: "spade", rank: 9 }, { name: "swd_daihuofenglun", kind: "daihuofenglun", suit: "heart", rank: 1 }, { name: "swd_xiayuncailing", kind: "xiayuncailing", suit: "diamond", rank: 2 }, { name: "swd_langeguaiyi", kind: "langeguaiyi", suit: "diamond", rank: 1 }, { name: "swd_zhiluxiaohu", kind: "jiguan", suit: "heart", rank: 3 }, { name: "swd_mujiaren", kind: "jiguan", suit: "club", rank: 7 }, { name: "swd_gouhunluo1", kind: "jiguan", suit: "spade", rank: 9 }, { name: "swd_gouhunluo2", kind: "jiguan", suit: "club", rank: 7 }, { name: "swd_jiguantong1", kind: "jiguan", suit: "diamond", rank: 4 }, { name: "swd_jiguantong2", kind: "jiguan", suit: "club", rank: 7 }, { name: "swd_jiguanfeng1", kind: "jiguan", suit: "club", rank: 3 }, { name: "swd_jiguanfeng2", kind: "jiguan", suit: "spade", rank: 4 }, { name: "swd_jiguanyuan1", kind: "jiguan", suit: "club", rank: 3 }, { name: "swd_jiguanyuan2", kind: "jiguan", suit: "diamond", rank: 2 }, { name: "swd_jiguanyaoshu1", kind: "jiguan", suit: "club", rank: 3 }, { name: "swd_jiguanyaoshu2", kind: "jiguan", suit: "spade", rank: 3 }, { name: "swd_xuanfengzhiren1", kind: "hslingjian", suit: "diamond", rank: 4 }, { name: "swd_zhongxinghujia1", kind: "hslingjian", suit: "heart", rank: 4 }, { name: "swd_xingtigaizao1", kind: "hslingjian", suit: "spade", rank: 6 }, { name: "swd_shijianhuisu1", kind: "hslingjian", suit: "diamond", rank: 6 }, { name: "swd_jinjilengdong1", kind: "hslingjian", suit: "spade", rank: 8 }, { name: "swd_shengxiuhaojiao1", kind: "hslingjian", suit: "heart", rank: 8 }, { name: "swd_yinmilichang1", kind: "hslingjian", suit: "club", rank: 10 }, { name: "swd_jiguanshu1", kind: "jiguan", suit: "club", rank: 2 }, { name: "swd_jiguanshu2", kind: "jiguan", suit: "diamond", rank: 2 }, { name: "swd_zhufangshenshi2", kind: "zhufangshenshi", suit: "spade", rank: 8 }, { name: "swd_zhiluxiaohu2", kind: "jiguan", suit: "diamond", rank: 4 }, { name: "swd_mujiaren2", kind: "jiguan", suit: "heart", rank: 6 }, { name: "swd_mujiaren3", kind: "jiguan", suit: "diamond", rank: 11 }, { name: "swd_yangpijuan2", kind: "yangpijuan", suit: "club", rank: 7 }, { name: "swd_xuejibingbao", kind: "xuejibingbao", suit: "club", rank: 1 }, { name: "swd_xuejibingbao2", kind: "xuejibingbao", suit: "spade", rank: 1 }, { name: "swd_fengyinzhidan", kind: "fengyinzhidan", suit: "spade", rank: 1 }, { name: "swd_shenmiguo", kind: "shenmiguo", suit: "spade", rank: 2 }, { name: "swd_shenmiguo2", kind: "shenmiguo", suit: "heart", rank: 1 }, { name: "swd_lianyaohu", kind: "lianyaohu", suit: "club", rank: 13, equip_skills: ["lianyaohu_shouna", "lianyaohu_lianhua"] }, { name: "swd_qiankundai", kind: "qiankundai", suit: "club", rank: 1 }, { name: "swd_shennongding", kind: "shennongding", suit: "club", rank: 13, equip_skills: ["shennongding"] }, { name: "swd_nvwashi", kind: "nvwashi", suit: "heart", rank: 13, equip_skills: ["nvwashi"] }, { name: "swd_kunlunjingc", kind: "kunlunjingc", suit: "spade", rank: 13, equip_skills: ["kunlunjingc"] }, { name: "swd_donghuangzhong", kind: "donghuangzhong", suit: "diamond", rank: 13, equip_skills: ["donghuangzhong"] }, { name: "swd_kongdongyin", kind: "kongdongyin", suit: "heart", rank: 13, equip_skills: ["kongdongyin"] }, { name: "swd_jingleishan", kind: "jingleishan", suit: "club", rank: 7 }, { name: "swd_chiyuxi", kind: "chiyuxi", suit: "diamond", rank: 7 }, { name: "swd_guisheqi", kind: "guisheqi", suit: "spade", rank: 9 }, { name: "swd_guisheqi2", kind: "guisheqi", suit: "club", rank: 7 }, { name: "swd_xianluhui", kind: "xianluhui", suit: "spade", rank: 13 }, { name: "swd_guilingzhitao", kind: "guilingzhitao", suit: "spade", rank: 7, equip_skills: ["nigong_accumulate", "nigong_strike"] }, { name: "swd_sifeizhenmian", kind: "sifeizhenmian", suit: "spade", rank: 4, equip_skills: ["yiluan"] }, { name: "swd_qinglongzhigui", kind: "qinglongzhigui", suit: "heart", rank: 6 }, { name: "swd_zhuquezhizhang", kind: "zhuquezhizhang", suit: "diamond", rank: 6 }, { name: "swd_baishouzhihu", kind: "baishouzhihu", suit: "spade", rank: 6 }, { name: "swd_xuanwuzhihuang", kind: "xuanwuzhihuang", suit: "club", rank: 6 }, { name: "swd_huanglinzhicong", kind: "huanglinzhicong", suit: "heart", rank: 5 }, { name: "swd_shuchui", kind: "shuchui", suit: "club", rank: 6, equip_skills: ["shuchui"] }, { name: "swd_sadengjinhuan", kind: "sadengjinhuan", suit: "club", rank: 13 }, { name: "swd_haotianta", kind: "haotianta", suit: "diamond", rank: 13 }, { name: "swd_yuruyi", kind: "yuruyi", suit: "heart", rank: 9 }, { name: "swd_yuchanqian", kind: "yuchanqian", suit: "spade", rank: 1, deferred_reason: "\u6362\u5F62\u6001\u968F\u673A mutation+\u7145\u9020\u88C5\u5907\u9762\u6302\u6279\u246C\uFF08autoViewAs \u6740\u672C\u6279\u5DF2\u8F6C\u6B63\uFF09" }, { name: "swd_yuchankun", kind: "yuchankun", suit: "club", rank: 2, deferred_reason: "\u6362\u5F62\u6001\u968F\u673A mutation+\u7145\u9020\u88C5\u5907\u9762\u6302\u6279\u246C\uFF08autoViewAs \u8349\u836F\u672C\u6279\u5DF2\u8F6C\u6B63\uFF09" }, { name: "swd_yuchanzhen", kind: "yuchanzhen", suit: "diamond", rank: 3, deferred_reason: "\u6362\u5F62\u6001\u968F\u673A mutation+\u7145\u9020\u88C5\u5907\u9762\u6302\u6279\u246C\uFF08autoViewAs \u9152\u672C\u6279\u5DF2\u8F6C\u6B63\uFF09" }, { name: "swd_yuchanxun", kind: "yuchanxun", suit: "heart", rank: 4, deferred_reason: "\u6362\u5F62\u6001\u968F\u673A mutation+\u7145\u9020\u88C5\u5907\u9762\u6302\u6279\u246C\uFF08autoViewAs \u6843\u672C\u6279\u5DF2\u8F6C\u6B63\uFF09" }, { name: "swd_yuchankan", kind: "yuchankan", suit: "spade", rank: 5, deferred_reason: "\u6362\u5F62\u6001\u968F\u673A mutation+\u7145\u9020\u88C5\u5907\u9762\u6302\u6279\u246C" }, { name: "swd_yuchanli", kind: "yuchanli", suit: "club", rank: 6, deferred_reason: "\u6362\u5F62\u6001\u968F\u673A mutation+\u7145\u9020\u88C5\u5907\u9762\u6302\u6279\u246C\uFF08autoViewAs \u5929\u4ED9\u9152\u672C\u6279\u5DF2\u8F6C\u6B63\uFF09" }, { name: "swd_yuchangen", kind: "yuchangen", suit: "diamond", rank: 7, deferred_reason: "\u6362\u5F62\u6001\u968F\u673A mutation+\u7145\u9020\u88C5\u5907\u9762\u6302\u6279\u246C\uFF08autoViewAs \u5C01\u5370\u4E4B\u4E39\u672C\u6279\u5DF2\u8F6C\u6B63\uFF09" }, { name: "swd_yuchandui", kind: "yuchandui", suit: "heart", rank: 8, deferred_reason: "\u6362\u5F62\u6001\u968F\u673A mutation+\u7145\u9020\u88C5\u5907\u9762\u6302\u6279\u246C\uFF08autoViewAs \u96EA\u9701\u51B0\u96F9\u672C\u6279\u5DF2\u8F6C\u6B63\uFF09" }, { name: "swd_shencaojie", kind: "shencaojie", suit: "club", rank: 4 }, { name: "swd_shencaojie2", kind: "shencaojie", suit: "diamond", rank: 4 }, { name: "swd_shencaojie3", kind: "shencaojie", suit: "spade", rank: 4 }, { name: "xuanyuanjian", kind: "xuanyuanjian", suit: "spade", rank: 1 }], Gc = [], Bc = [], Fc = JSON.parse('[{"name":"swd\u97E9\u7F57","faction":"qun","hp":5,"gender":"male","skills":[{"name":"tiebi","mapping":"host:tiebi"}]},{"name":"swd\u4E60\u5AE3","faction":"shu","hp":3,"gender":"female","skills":[{"name":"jiefen","mapping":"host:jiefen"},{"name":"datong","mapping":"host:datong"}]},{"name":"swd\u8F69\u8F95\u5251\u4F8D","faction":"qun","hp":3,"gender":"male","skills":[{"name":"chengjian","mapping":"host:chengjian"},{"name":"huanling","mapping":"host:huanling"}]},{"name":"swd\u7A0B\u54AC\u91D1","faction":"qun","hp":4,"gender":"male","skills":[{"name":"jiuchi","mapping":"builtin:\u9152\u6C60"},{"name":"jufu","mapping":"host:jufu"}]},{"name":"swd\u79E6\u53D4\u5B9D","faction":"qun","hp":3,"gender":"male","skills":[{"name":"huajing","mapping":"host:huajing"},{"name":"pingxu","mapping":"host:pingxu"}]},{"name":"swd\u5B50\u8FBE\u4E66","faction":"wei","hp":3,"gender":"male","skills":[{"name":"shoulie","mapping":"host:shoulie"},{"name":"hudun","mapping":"host:hudun"}]},{"name":"swd\u5F20\u9AD8","faction":"shu","hp":4,"gender":"male","skills":[{"name":"yicong","mapping":"builtin:\u4E49\u4ECE"},{"name":"poxing","mapping":"host:poxing"}]},{"name":"swd\u62D3\u8DCB\u7389\u513F","faction":"shu","hp":4,"gender":"female","skills":[{"name":"swdliuhong","mapping":"host:swdliuhong"},{"name":"poyue","mapping":"host:poyue"},{"name":"niepan","mapping":"host:niepan"}]},{"name":"swd\u6C34\u955C","faction":"qun","hp":4,"gender":"female","skills":[{"name":"mojian","mapping":"host:mojian"},{"name":"duanyue","mapping":"host:duanyue"},{"name":"tuzhen","mapping":"host:tuzhen"}]},{"name":"swd\u5C48\u5A34","faction":"qun","hp":3,"gender":"female","skills":[{"name":"mojian","mapping":"host:mojian"},{"name":"huanxia","mapping":"host:huanxia"}]},{"name":"swd\u58F6\u4E2D\u4ED9","faction":"wu","hp":3,"gender":"male","skills":[{"name":"daofa","mapping":"host:daofa"},{"name":"xielv","mapping":"host:xielv"},{"name":"hujing","mapping":"host:hujing"}]},{"name":"swd\u5170\u8335","faction":"wei","hp":3,"gender":"female","skills":[{"name":"xingdian","mapping":"host:xingdian"},{"name":"yulin","mapping":"host:yulin"},{"name":"luomei","mapping":"host:luomei"}]},{"name":"swd\u829D\u8335","faction":"wei","hp":3,"gender":"female","skills":[{"name":"xuehuang","mapping":"host:xuehuang"},{"name":"ningshuang","mapping":"host:ningshuang"},{"name":"zhuyu","mapping":"host:zhuyu"}]},{"name":"swd\u9EC4\u96F7","faction":"qun","hp":3,"gender":"male","skills":[{"name":"jilve","mapping":"host:jilve"},{"name":"gongshen","mapping":"host:gongshen"},{"name":"gaizao","mapping":"host:gaizao"}]},{"name":"swd_septem","faction":"qun","hp":4,"gender":"male","skills":[{"name":"jiying","mapping":"host:jiying"},{"name":"liaoyuan","mapping":"host:liaoyuan"},{"name":"yishan","mapping":"host:yishan"}]},{"name":"swd_huanyuanzhi","faction":"qun","hp":3,"gender":"male","skills":[{"name":"swdtianshu","mapping":"host:swdtianshu"},{"name":"lanzhi","mapping":"host:lanzhi"},{"name":"mufeng","mapping":"host:mufeng"}]},{"name":"swd_guyue","faction":"wei","hp":3,"gender":"male","skills":[{"name":"gtiandao","mapping":"host:g_tiandao"},{"name":"gxianyin","mapping":"host:gxianyin"},{"name":"wangchen","mapping":"host:wangchen"}]},{"name":"swd_shangzhang","faction":"shu","hp":4,"gender":"male","skills":[{"name":"lianwu","mapping":"host:lianwu"}]},{"name":"swd_youzhao","faction":"shu","hp":4,"gender":"male","skills":[{"name":"longdan","mapping":"builtin:\u9F99\u80C6"},{"name":"yuchen","mapping":"host:yuchen"}]},{"name":"swd_wangsiyue","faction":"wei","hp":3,"gender":"female","skills":[{"name":"duishi","mapping":"host:duishi"},{"name":"biyue","mapping":"builtin:\u95ED\u6708"}]},{"name":"swd_yuwentuo","faction":"shu","hp":4,"gender":"male","skills":[{"name":"wushuang","mapping":"builtin:\u65E0\u53CC"},{"name":"xielei","mapping":"host:xielei"},{"name":"kunlunjing","mapping":"host:kunlunjing"}]},{"name":"swd_fuyan","faction":"qun","hp":4,"gender":"male","skills":[{"name":"lianda","mapping":"host:lianda"}]},{"name":"swd_rongshuang","faction":"wu","hp":3,"gender":"female","skills":[{"name":"suiyan","mapping":"host:suiyan"},{"name":"duanxing","mapping":"host:duanxing"}]},{"name":"swd_muyun","faction":"wei","hp":4,"gender":"male","skills":[{"name":"zhuhai","mapping":"host:zhuhai"},{"name":"polang","mapping":"host:polang"},{"name":"jikong","mapping":"host:jikong"}]},{"name":"swd_chenjingchou","faction":"wu","hp":3,"gender":"male","skills":[{"name":"youyin","mapping":"host:youyin"},{"name":"yihua","mapping":"host:yihua"}]},{"name":"swd_tuwei","faction":"shu","hp":3,"gender":"female","skills":[{"name":"zhanlu","mapping":"host:zhanlu"},{"name":"susheng","mapping":"host:susheng"}]},{"name":"swd_xiarou","faction":"shu","hp":3,"gender":"male","skills":[{"name":"xianghui","mapping":"host:xianghui"},{"name":"huiqi","mapping":"host:huiqi"}]},{"name":"swd_zhuoshanzhu","faction":"wu","hp":4,"gender":"male","skills":[{"name":"suiyan","mapping":"host:suiyan"},{"name":"wanjun","mapping":"host:wanjun"}]},{"name":"swd_moye","faction":"wu","hp":3,"gender":"female","skills":[{"name":"rexue","mapping":"host:rexue"},{"name":"liuli","mapping":"builtin:\u6D41\u79BB"},{"name":"senluo","mapping":"host:senluo"}]},{"name":"swd_yeyaxi","faction":"shu","hp":3,"gender":"female","skills":[{"name":"rexue","mapping":"host:rexue"},{"name":"huopu","mapping":"host:huopu"},{"name":"shenyan","mapping":"host:shenyan"}]},{"name":"swd_xiaohuanglong","faction":"wei","hp":3,"gender":"male","skills":[{"name":"yeying","mapping":"host:yeying"},{"name":"juxi","mapping":"host:juxi"},{"name":"juxi_count","mapping":"host:juxi_count"}]},{"name":"swd_hupo","faction":"wu","hp":3,"gender":"male","skills":[{"name":"dunxing","mapping":"host:dunxing"},{"name":"guiying","mapping":"host:guiying"}]},{"name":"swd_situqiang","faction":"shu","hp":3,"gender":"female","skills":[{"name":"fengze","mapping":"host:fengze"},{"name":"lingyue","mapping":"host:lingyue"},{"name":"jinlin","mapping":"host:jinlin"}]},{"name":"swd_anka","faction":"qun","hp":3,"gender":"male","skills":[{"name":"songci","mapping":"host:songci"},{"name":"anlianying","mapping":"host:anlianying"}]},{"name":"swd_luchengxuan","faction":"wu","hp":4,"gender":"male","skills":[{"name":"ljifeng","mapping":"host:ljifeng"},{"name":"lxianglong","mapping":"host:lxianglong"}]},{"name":"swd_murongshi","faction":"shu","hp":3,"gender":"female","skills":[{"name":"duanyi","mapping":"host:duanyi"},{"name":"guxing","mapping":"host:guxing"},{"name":"guxing3","mapping":"host:guxing3"}]},{"name":"swd_yuxiaoxue","faction":"wei","hp":3,"gender":"female","skills":[{"name":"huanhun","mapping":"host:huanhun"},{"name":"daixing","mapping":"host:daixing"},{"name":"yinyue","mapping":"host:yinyue"},{"name":"daixing2","mapping":"host:daixing2"},{"name":"daixing3","mapping":"host:daixing3"}]},{"name":"swd_yuli","faction":"wu","hp":3,"gender":"female","skills":[{"name":"lingxin","mapping":"host:lingxin"},{"name":"tianxiang","mapping":"host:tianxiang"}]},{"name":"swd_kama","faction":"qun","hp":3,"gender":"female","skills":[{"name":"yueren","mapping":"host:yueren"},{"name":"shangshi","mapping":"builtin:\u4F24\u901D"}]},{"name":"swd_nicole","faction":"qun","hp":3,"gender":"female","skills":[{"name":"huanjian","mapping":"host:huanjian"},{"name":"lingwu","mapping":"host:lingwu"},{"name":"minjing","mapping":"host:minjing"}]},{"name":"swd_weida","faction":"qun","hp":3,"gender":"female","skills":[{"name":"yueren","mapping":"host:yueren"},{"name":"zhenlie","mapping":"host:zhenlie"},{"name":"duijue","mapping":null}]},{"name":"swd_chunyuheng","faction":"wei","hp":2,"gender":"male","skills":[{"name":"jueqing","mapping":"builtin:\u7EDD\u60C5"},{"name":"shengshou","mapping":"host:shengshou"},{"name":"xuying","mapping":"host:xuying"}]},{"name":"swd_ziqiao","faction":"shu","hp":3,"gender":"female","skills":[{"name":"guaili","mapping":"host:guaili"},{"name":"fuyan","mapping":"host:fuyan"}]},{"name":"swd_lilian","faction":"qun","hp":3,"gender":"female","skills":[{"name":"swd_wuxie","mapping":"host:wuxie"},{"name":"lqingcheng","mapping":"host:lqingcheng"}]},{"name":"swd_jiting","faction":"wei","hp":4,"gender":"female","skills":[{"name":"guanhu","mapping":"host:guanhu"},{"name":"lingshi","mapping":"host:lingshi"}]},{"name":"swd_hengai","faction":"shu","hp":3,"gender":"female","skills":[{"name":"hlingbo","mapping":"host:hlingbo"},{"name":"funiao","mapping":"host:funiao"},{"name":"ningxian","mapping":"host:ningxian"}]},{"name":"swd_miles","faction":"qun","hp":4,"gender":"male","skills":[{"name":"aojian","mapping":"host:aojian"},{"name":"miles_xueyi","mapping":"host:miles_xueyi"},{"name":"mohua2","mapping":"host:mohua2"}]},{"name":"swd_kangnalishi","faction":"qun","hp":1,"gender":"male","skills":[{"name":"busi","mapping":"host:busi"},{"name":"xuying","mapping":"host:xuying"},{"name":"yinguo","mapping":"host:yinguo"}]},{"name":"swd_zhaoyun","faction":"shu","hp":4,"gender":"male","skills":[{"name":"longdan","mapping":"builtin:\u9F99\u80C6"},{"name":"pozhen","mapping":"host:pozhen"},{"name":"tanlin","mapping":null}]},{"name":"swd_huyue","faction":"wu","hp":3,"gender":"female","skills":[{"name":"hyunshen","mapping":"host:hyunshen"},{"name":"fengming","mapping":"host:fengming"},{"name":"hyunshen2","mapping":"host:hyunshen2"}]},{"name":"swd_shanxiaoxiao","faction":"wu","hp":3,"gender":"female","skills":[{"name":"shehun","mapping":"host:shehun"},{"name":"xiaomoyu","mapping":"host:xiaomoyu"}]},{"name":"swd_duguningke","faction":"qun","hp":3,"gender":"female","skills":[{"name":"nlianji","mapping":"host:nlianji"},{"name":"touxi","mapping":"builtin:\u7A81\u88AD"}]},{"name":"swd_haidapang","faction":"wu","hp":3,"gender":"female","skills":[{"name":"bingjian","mapping":"host:bingjian"},{"name":"rumeng","mapping":"host:rumeng"}]},{"name":"swd_maixing","faction":"wu","hp":3,"gender":"male","skills":[{"name":"toudan","mapping":"host:toudan"},{"name":"shending","mapping":"host:shending"}]},{"name":"swd_fengtianling","faction":"shu","hp":4,"gender":"male","skills":[{"name":"guiyan","mapping":"host:guiyan"},{"name":"jiang","mapping":"host:jiang"}]},{"name":"swd_fengyu","faction":"shu","hp":4,"gender":"male","skills":[{"name":"fzhenwei","mapping":"host:fzhenwei"},{"name":"shangxi","mapping":"host:shangxi"}]},{"name":"swd_muyue","faction":"wei","hp":3,"gender":"female","skills":[{"name":"xingzhui","mapping":"host:xingzhui"},{"name":"lingxian","mapping":"host:lingxian"},{"name":"shouyin","mapping":"host:shouyin"}]},{"name":"swd_shaowei","faction":"shu","hp":3,"gender":"female","skills":[{"name":"jianji","mapping":"host:jianji"},{"name":"huangyu","mapping":"host:huangyu"}]},{"name":"swd_jialanduo","faction":"qun","hp":4,"gender":"male","skills":[{"name":"xianyin","mapping":"host:xianyin"},{"name":"mailun","mapping":"host:mailun"}]},{"name":"swd_qi","faction":"qun","hp":3,"gender":"male","skills":[{"name":"yaotong","mapping":"host:yaotong"},{"name":"heihuo","mapping":"host:heihuo"},{"name":"pojian","mapping":"host:pojian"}]},{"name":"swd_huiyan","faction":"qun","hp":4,"gender":"male","skills":[{"name":"hwendao","mapping":"host:hwendao"},{"name":"lingfeng","mapping":"host:lingfeng"},{"name":"hxunzhi","mapping":"host:hxunzhi"}]},{"name":"swd_qiner","faction":"wei","hp":3,"gender":"female","skills":[{"name":"huanyin","mapping":"host:huanyin"},{"name":"tianhuo","mapping":"host:tianhuo"},{"name":"xuanzhou","mapping":"host:xuanzhou"}]},{"name":"swd_jiliang","faction":"wu","hp":3,"gender":"male","skills":[{"name":"yunchou","mapping":"host:yunchou"},{"name":"gongxin","mapping":"host:gongxin"},{"name":"jqimou","mapping":"host:jqimou"}]},{"name":"swd_zhanglie","faction":"qun","hp":4,"gender":"male","skills":[{"name":"huajin","mapping":"host:huajin"},{"name":"poxiao","mapping":"host:poxiao"}]},{"name":"swd_xuanyuanjianxian","faction":"qun","hp":4,"gender":"male","skills":[{"name":"pozhou","mapping":"host:pozhou"},{"name":"huajian","mapping":"host:huajian"},{"name":"xuanyuan","mapping":"host:xuanyuan"},{"name":"pozhou2","mapping":"host:pozhou2"}]},{"name":"swd_sikongyu","faction":"qun","hp":4,"gender":"male","skills":[{"name":"sliufeng","mapping":"host:sliufeng"},{"name":"linyun","mapping":"host:linyun"},{"name":"hutian","mapping":"host:hutian"}]},{"name":"swd_jipeng","faction":"wu","hp":3,"gender":"male","skills":[{"name":"reyingzi","mapping":"host:reyingzi"},{"name":"guozao","mapping":"host:guozao"}]},{"name":"swd_duopeng","faction":"wu","hp":3,"gender":"male","skills":[{"name":"luanji","mapping":"host:luanji"},{"name":"reyingzi","mapping":"host:reyingzi"}]},{"name":"swd_fu","faction":"qun","hp":5,"gender":"male","skills":[{"name":"yudun","mapping":"host:yudun"}]},{"name":"swd_kendi","faction":"qun","hp":3,"gender":"male","skills":[{"name":"zhanxing","mapping":"host:zhanxing"},{"name":"kbolan","mapping":"host:kbolan"}]},{"name":"swd_jiuyou","faction":"wei","hp":3,"gender":"male","skills":[{"name":"lexue","mapping":"host:lexue"}]},{"name":"swd_linyue","faction":"wei","hp":3,"gender":"male","skills":[{"name":"zhenjiu","mapping":"host:zhenjiu"},{"name":"lmazui","mapping":"host:lmazui"}]},{"name":"swd_shuwaner","faction":"shu","hp":3,"gender":"female","skills":[{"name":"sxianjing","mapping":"host:sxianjing"},{"name":"huodan","mapping":"host:huodan"},{"name":"sxianjing_gain","mapping":"host:sxianjing_gain"},{"name":"sxianjing_damage","mapping":"host:sxianjing_damage"}]},{"name":"swd_jiangwu","faction":"shu","hp":4,"gender":"male","skills":[{"name":"yijue","mapping":"host:yijue"},{"name":"dangping","mapping":"host:dangping"}]},{"name":"swd_yuchiyanhong","faction":"shu","hp":3,"gender":"female","skills":[{"name":"huanxing","mapping":"host:huanxing"},{"name":"meihuo","mapping":"host:meihuo"}]},{"name":"swd_jiangziya","faction":"wu","hp":3,"gender":"male","skills":[{"name":"mingfu","mapping":"host:mingfu"},{"name":"tianlun","mapping":"host:tianlun"}]},{"name":"swd_cheyun","faction":"wu","hp":3,"gender":"female","skills":[{"name":"cyxianjiang","mapping":"host:cyxianjiang"},{"name":"cyqiaoxie","mapping":"host:cyqiaoxie"},{"name":"shengong","mapping":"host:shengong"}]},{"name":"swd_duanmeng","faction":"shu","hp":4,"gender":"female","skills":[{"name":"xuanying","mapping":"host:xuanying"},{"name":"lieren","mapping":"host:lieren"}]},{"name":"swd_chenfu","faction":"qun","hp":4,"gender":"male","skills":[{"name":"xuanzhou","mapping":"host:xuanzhou"},{"name":"bingfeng","mapping":"host:bingfeng"}]}]'), Hc = [{ name: "swd_zhenjiu2", mapping: "host:zhenjiu2" }, { name: "swd_lmazui2", mapping: "host:lmazui2" }, { name: "swd_huanxing_state", mapping: "host:huanxing2" }, { name: "swd_guiyan2", mapping: "host:guiyan2" }, { name: "swd_huajin2", mapping: "host:huajin2" }, { name: "swd_jinlin2", mapping: "host:jinlin2" }, { name: "tianxiang2", mapping: "host:tianxiang2" }, { name: "hxunzhi2", mapping: "host:hxunzhi2" }, { name: "bingfeng2", mapping: "host:bingfeng2" }, { name: "xuanyuanjian2", mapping: "host:xuanyuanjian2" }, { name: "xuanyuanjian3", mapping: "host:xuanyuanjian3" }, { name: "xuanyuan", mapping: "host:xuanyuan" }, { name: "jifeng", mapping: "host:jifeng" }, { name: "moyan", mapping: "host:moyan" }, { name: "miedao", mapping: "host:miedao" }, { name: "swd_xiuluo", mapping: "host:swd_xiuluo" }], Wc = { source: Lc, upstream_path: Dc, upstream_commit: Mc, pack: Uc, cards: Vc, host_skills: Gc, deferred: Bc, generals: Fc, skills: Hc }, Kc = { package_format_version: 1, namespace: "upstream-noname", package_id: "shahai-yunchou", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [87, 73, 16, 137, 17, 228, 185, 148, 29, 192, 248, 121, 46, 123, 51, 178, 99, 59, 227, 5, 238, 24, 234, 223, 153, 43, 225, 121, 118, 175, 237, 81], asset_hash: [34, 111, 194, 231, 244, 54, 173, 146, 254, 193, 36, 14, 84, 194, 87, 207, 61, 22, 100, 151, 193, 186, 122, 45, 222, 132, 90, 79, 201, 18, 92, 112], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, Jc = [], Qc = [], Yc = [], Xc = [], Zc = [], ed = [], nd = [], ad = [], td = { "zh-CN": { fudichouxin: "\u91DC\u5E95\u62BD\u85AA", huoshan: "\u706B\u6740", shuigong: "\u6C34\u653B", yunchou: "\u8FD0\u7B79\u5E37\u5E44" } }, id = { schema_version: 1, package_id: "shahai-yunchou", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "shahai-yunchou:cards", source_hash: [242, 7, 161, 150, 106, 106, 85, 215, 72, 11, 50, 144, 208, 98, 108, 44, 119, 194, 137, 133, 206, 167, 18, 228, 203, 241, 192, 44, 92, 190, 91, 119], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, sd = { "shahai-yunchou:cards": "assets/cards.json" }, od = "gen-1", it = { manifest: Kc, modules: Jc, skill_index: Qc, character_index: Yc, card_index: Xc, trigger_index: Zc, modifier_index: ed, active_index: nd, view_as_index: ad, translations: td, asset_manifest: id, asset_paths: sd, compiler_version: od }, rd = "noname", ld = "apps/core/extension/\u6740\u6D77\u62FE\u9057/card/yunchou/", ud = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", cd = { name: "shahai-yunchou-upstream-noname", version: 2, description: "G1 \u6740\u6D77\u62FE\u9057\xB7\u8FD0\u7B79\u5E37\u5E44\u5207\u7247", card_prefix: "yc_" }, dd = [{ name: "huoshan", kind: "huosha", suit: "heart", rank: 6 }, { name: "guohe", kind: "guohe", suit: "diamond", rank: 3 }, { name: "guohe", kind: "guohe", suit: "club", rank: 8 }, { name: "guohe", kind: "guohe", suit: "spade", rank: 13 }, { name: "shuigong", kind: "shuigong", suit: "club", rank: 7 }, { name: "shuigong", kind: "shuigong", suit: "club", rank: 7 }, { name: "shuigong", kind: "shuigong", suit: "spade", rank: 9 }, { name: "fudichouxin", kind: "fudichouxin", suit: "diamond", rank: 4 }, { name: "fudichouxin", kind: "fudichouxin", suit: "club", rank: 6 }, { name: "fudichouxin", kind: "fudichouxin", suit: "spade", rank: 1 }, { name: "chenhuodajie", kind: "chenhuodajie", suit: "diamond", rank: 6 }, { name: "chenhuodajie", kind: "chenhuodajie", suit: "diamond", rank: 9 }, { name: "chenhuodajie", kind: "chenhuodajie", suit: "club", rank: 3 }, { name: "suolianjia", kind: "suolianjia", suit: "club", rank: 13, link_on_equip: true }, { name: "hongshui", kind: "hongshui", suit: "club", rank: 7 }, { name: "toulianghuanzhu", kind: "toulianghuanzhu", suit: "heart", rank: 9 }, { name: "toulianghuanzhu", kind: "toulianghuanzhu", suit: "club", rank: 10 }, { name: "geanguanhuo", kind: "geanguanhuo", suit: "heart", rank: 1 }, { name: "geanguanhuo", kind: "geanguanhuo", suit: "spade", rank: 6 }, { name: "caochuanjiejian", kind: "caochuanjiejian", suit: "spade", rank: 7 }, { name: "caochuanjiejian", kind: "caochuanjiejian", suit: "spade", rank: 7 }, { name: "shezhanqunru", kind: "shezhanqunru", suit: "heart", rank: 4 }, { name: "shezhanqunru", kind: "shezhanqunru", suit: "club", rank: 8 }], md = [], pd = { source: rd, upstream_path: ld, upstream_commit: ud, pack: cd, cards: dd, deferred: md }, hd = "noname", gd = "apps/core/extension/\u6740\u6D77\u62FE\u9057/character/yunchou/", fd = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", _d = { name: "shahai-yunchou-generals-b9", version: 1, description: "G1 \u8FD0\u7B79\u6B66\u5C06\u9762\u62799\uFF08\u4E8E\u5409\u6574\u5C06\uFF1A\u8749\u7F18 max_hp \u51CF\u9762\u8F6C\u6B63\uFF09", card_prefix: "ycn_" }, vd = [{ id: "ycn_diy_caocao", name: "ycn\u66F9\u64CD", faction: "wei", hp: 4, gender: "male", skills: [{ name: "ycnxicai", title: "\u620F\u624D", mapping: "host:xicai" }, { name: "ycndiyjianxiong", title: "\u5978\u96C4", mapping: null }, { name: "hujia", title: "\u62A4\u9A7E", mapping: "builtin:\u62A4\u9A7E" }] }, { id: "ycn_diy_weiyan", name: "ycn\u9B4F\u5EF6", faction: "shu", hp: 4, gender: "male", skills: [{ name: "ycndiykuanggu", title: "\u72C2\u9AA8", mapping: "host:diykuanggu" }] }, { id: "ycn_diy_sp_zhouyu", name: "ycn\u5468\u745C", faction: "wu", hp: 3, gender: "male", skills: [{ name: "ycnxiongzi", title: "\u96C4\u59FF", mapping: "host:xiongzi" }, { name: "ycnyaliang", title: "\u96C5\u91CF", mapping: "host:yaliang" }] }, { id: "ycn_diy_luxun", name: "ycn\u9646\u900A", faction: "wu", hp: 3, gender: "male", skills: [{ name: "ycnzonghuo", title: "\u7EB5\u706B", mapping: "host:zonghuo" }, { name: "ycnshaoying", title: "\u70E7\u8425", mapping: "host:shaoying" }] }, { id: "ycn_diy_yuji", name: "ycn\u4E8E\u5409", faction: "qun", hp: 3, gender: "male", skills: [{ name: "ycndiyguhuo", title: "\u86CA\u60D1", mapping: "host:diyguhuo" }, { name: "ycndiychanyuan", title: "\u8749\u7F18", mapping: "host:diychanyuan" }] }, { id: "ycn_diy_zhouyu", name: "ycn\u5468\u745C", faction: "wu", hp: 3, gender: "male", skills: [{ name: "ycnjieyan", title: "\u9699\u7130", mapping: "host:jieyan" }, { name: "ycnhonglian", title: "\u7EA2\u83B2", mapping: "host:honglian" }] }, { id: "ycn_diy_zhenji", name: "ycn\u7504\u59EC", faction: "wei", hp: 3, gender: "female", skills: [{ name: "ycnjiaoxia", title: "\u72E1\u9EE0", mapping: "host:jiaoxia" }, { name: "ycnyiesheng", title: "\u91CE\u5347", mapping: "host:yiesheng" }] }, { id: "ycn_diy_caiwenji", name: "ycn\u8521\u6587\u59EC", faction: "qun", hp: 3, gender: "female", skills: [{ name: "beige", title: "\u60B2\u6B4C", mapping: "host:beige" }, { name: "ycnguihan", title: "\u5F52\u6C49", mapping: "host:guihan" }] }, { id: "ycn_diy_lukang", name: "ycn\u9646\u6297", faction: "wu", hp: 4, gender: "male", skills: [{ name: "ycnluweiyan", title: "\u5389\u7130", mapping: "host:luweiyan" }, { name: "ycnluweiyan2", title: "\u5389\u7130\xB7\u94FE\u6740", mapping: "host:luweiyan2" }, { name: "qianxun", title: "\u8C26\u900A", mapping: "builtin:\u8C26\u900A" }] }, { id: "ycn_old_majun", name: "ycn\u9A6C\u94A7", faction: "wei", hp: 3, gender: "male", skills: [{ name: "xinfu_jingxie", title: "\u7CBE\u68B0", mapping: null }, { name: "ycnxinfu_qiaosi", title: "\u5DE7\u601D", mapping: null }] }], yd = { source: hd, upstream_path: gd, upstream_commit: fd, pack: _d, generals: vd }, kd = { package_format_version: 1, namespace: "upstream-noname", package_id: "shahai-zhenfa", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [83, 148, 91, 71, 176, 142, 85, 235, 241, 190, 176, 53, 230, 223, 245, 74, 197, 187, 203, 191, 104, 12, 134, 170, 95, 74, 42, 149, 171, 33, 19, 178], asset_hash: [2, 250, 107, 122, 186, 246, 73, 201, 2, 120, 23, 125, 30, 171, 190, 145, 67, 30, 150, 250, 27, 69, 102, 73, 94, 115, 5, 218, 205, 27, 76, 4], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, wd = [], bd = [], xd = [], $d = [], jd = [], Sd = [], Ed = [], zd = [], Ad = { schema_version: 1, package_id: "shahai-zhenfa", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "shahai-zhenfa:cards", source_hash: [212, 242, 140, 245, 41, 62, 165, 171, 122, 180, 21, 0, 119, 163, 234, 243, 149, 248, 103, 207, 9, 191, 218, 139, 73, 11, 55, 57, 198, 30, 168, 136], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, Cd = { "shahai-zhenfa:cards": "assets/cards.json" }, Rd = "test", st = { manifest: kd, modules: wd, skill_index: bd, character_index: xd, card_index: $d, trigger_index: jd, modifier_index: Sd, active_index: Ed, view_as_index: zd, asset_manifest: Ad, asset_paths: Cd, compiler_version: Rd }, Nd = "qita/sanguosha/noname/extension/\u6740\u6D77\u62FE\u9057/card/zhenfa", Pd = "extension/\u6740\u6D77\u62FE\u9057/card/zhenfa/", qd = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", Id = { name: "shahai-zhenfa-upstream-noname", version: 1, description: "G1 \u8F6C\u6362\uFF1A\u4E0A\u6E38 zhenfa \u9996\u5207\u7247\uFF08\u5730\u8F7D\u9635\uFF09", card_prefix: "zf_" }, Od = [{ name: "zf_dizaizhen", kind: "dizaizhen", suit: "diamond", rank: 3 }, { name: "zf_dizaizhen", kind: "dizaizhen", suit: "spade", rank: 3 }, { name: "zf_dizaizhen", kind: "dizaizhen", suit: "heart", rank: 3 }, { name: "swd_tianfuzhen1", kind: "tianfuzhen", suit: "spade", rank: 1 }, { name: "swd_qixingzhen1", kind: "qixingzhen", suit: "club", rank: 9 }, { name: "swd_niaoxiangzhen1", kind: "niaoxiangzhen", suit: "heart", rank: 3 }, { name: "swd_changshezhen1", kind: "changshezhen", suit: "diamond", rank: 1 }, { name: "swd_changshezhen2", kind: "changshezhen", suit: "club", rank: 1 }], Td = [], Ld = [{ name: "pozhenjue", reason: "\u4F9D\u8D56\u56FD\u6218\u961F\u5217/\u56F4\u653B/\u5EA7\u4F4D\u91CD\u6392\u4F53\u7CFB\uFF08inline/isFriendOf/swapSeat\uFF09\uFF0C\u539F\u751F\u56FD\u6218\u539F\u578B\u672A\u5EFA\u6A21\u2014\u2014\u7ED3\u6784\u6027\u4FDD\u7559\u9879\u540C\u65CF" }], Dd = { source: Nd, upstream_path: Pd, upstream_commit: qd, pack: Id, cards: Od, host_skills: Td, deferred: Ld }, Md = { package_format_version: 1, namespace: "upstream-noname", package_id: "wudian", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [224, 212, 136, 182, 142, 127, 174, 224, 64, 168, 249, 131, 136, 232, 56, 12, 82, 126, 204, 103, 169, 162, 33, 160, 174, 233, 63, 60, 228, 9, 190, 247], asset_hash: [101, 186, 252, 146, 248, 33, 244, 97, 247, 227, 110, 37, 19, 234, 69, 143, 65, 21, 226, 79, 251, 191, 128, 16, 8, 88, 15, 225, 249, 137, 146, 161], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, Ud = [], Vd = [], Gd = [], Bd = [], Fd = [], Hd = [], Wd = [], Kd = [], Jd = { "zh-CN": { diaobingqianjiang: "\u8C03\u5175\u9063\u5C06", sanjian: "\u4E09\u5C16", wangmeizhike: "\u671B\u6885\u6B62\u6E34", youdishenru: "\u8BF1\u654C\u6DF1\u5165" } }, Qd = { schema_version: 1, package_id: "wudian", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "wudian:cards", source_hash: [243, 4, 96, 226, 174, 224, 22, 240, 77, 131, 190, 254, 60, 103, 228, 236, 232, 29, 21, 165, 250, 11, 71, 138, 97, 179, 174, 47, 29, 135, 117, 67], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, Yd = { "wudian:cards": "assets/cards.json" }, Xd = "gen-1", ot = { manifest: Md, modules: Ud, skill_index: Vd, character_index: Gd, card_index: Bd, trigger_index: Fd, modifier_index: Hd, active_index: Wd, view_as_index: Kd, translations: Jd, asset_manifest: Qd, asset_paths: Yd, compiler_version: Xd }, Zd = "noname", em = "apps/core/extension/\u73A9\u70B9\u8BBA\u6740/", nm = "5c02a745eb33b61b2e61e1167e1b71ca163911f6", am = { name: "wudian-upstream-noname", version: 2, description: "G1 \u73A9\u70B9\u8BBA\u6740", card_prefix: "wd_" }, tm = [{ name: "sha", kind: "sha", suit: "diamond", rank: 9 }, { name: "sha", kind: "sha", suit: "spade", rank: 6 }, { name: "shan", kind: "shan", suit: "heart", rank: 6 }, { name: "shan", kind: "shan", suit: "diamond", rank: 3 }, { name: "shan", kind: "shan", suit: "diamond", rank: 11 }, { name: "tao", kind: "tao", suit: "heart", rank: 5 }, { name: "tao", kind: "tao", suit: "heart", rank: 7 }, { name: "wangmeizhike", kind: "wangmeizhike", suit: "club", rank: 8 }, { name: "wangmeizhike", kind: "wangmeizhike", suit: "club", rank: 11 }, { name: "youdishenru", kind: "youdishenru", suit: "club", rank: 9 }, { name: "youdishenru", kind: "youdishenru", suit: "spade", rank: 11 }, { name: "youdishenru", kind: "youdishenru", suit: "spade", rank: 12 }, { name: "sanjian", kind: "sanjian", suit: "club", rank: 5, range: 3, equip_skills: ["wd_sanjian"] }, { name: "tiesuo", kind: "tiesuo", suit: "spade", rank: 4 }], im = [{ name: "diaobingqianjiang", count: 2, reason: "\u591A\u4EBA\u65B9\u5411\u9009\u62E9\u591A\u6B65\u7ED3\u7B97" }], sm = [{ name: "wd_wangmeizhike", card: "wangmeizhike", mapping: "host:wangmeizhike_card" }, { name: "wd_sanjian", card: "sanjian", mapping: "host:sanjian_weapon" }, { name: "moshou", general: "wdb_feishi", mapping: "host:moshou_immunity" }, { name: "ciqiu", general: "wdb_hanlong", mapping: "host:wudian_ciqiu" }, { name: "nsshuaiyan", general: "wdb_feishi", mapping: "host:wudian_shuaiyan" }, { name: "zhucheng", general: "wdb_liufu", mapping: "host:wudian_zhucheng" }, { name: "zhucheng2", general: "wdb_liufu", mapping: "host:wudian_zhucheng_guard" }, { name: "kangyin", general: "wdb_liuzan", mapping: "host:wudian_kangyin" }, { name: "chezhen", general: "wdb_tianyu", mapping: "host:wudian_chezhen" }, { name: "juedao", general: "wdb_liuyan", mapping: "host:wudian_juedao" }, { name: "geju", general: "wdb_liuyan", mapping: "host:wudian_geju" }, { name: "duoqi", general: "wdb_liufu", mapping: null, reason: "phaseUse \u4E2D\u6B62\u672A\u5EFA\u6A21+\u4F9D\u8D56\u7B51\u57CE\u6807\u8BB0\u2014\u20142026-10-08 \u6B63\u5F0F\u6682\u7F13" }, { name: "youzhan", general: "wdb_tianyu", mapping: null, reason: "shaBefore \u65F6\u70B9\u6620\u5C04\u504F\u665A+\u865A\u62DF useCard \u7EC4\u5408\u2014\u20142026-10-08 \u6B63\u5F0F\u6682\u7F13" }], om = [{ name: "\u8D39\u8BD7", faction: "shu", hp: 3, gender: "male", skills: [{ name: "nsshuaiyan", mapping: "host:wudian_shuaiyan" }, { name: "moshou", mapping: "host:moshou_immunity" }] }, { name: "\u97E9\u9F99", faction: "wei", hp: 4, gender: "male", skills: [{ name: "siji", mapping: "host:siji_discard_draw" }, { name: "ciqiu", mapping: "host:wudian_ciqiu" }] }, { name: "\u5218\u99A5", faction: "wei", hp: 3, gender: "male", skills: [{ name: "zhucheng", mapping: "host:wudian_zhucheng" }, { name: "zhucheng2", mapping: "host:wudian_zhucheng_guard" }, { name: "duoqi", mapping: null, reason: "phaseUse \u4E2D\u6B62\uFF08evt.skipped\uFF09\u672A\u5EFA\u6A21+\u4F9D\u8D56\u7B51\u57CE\u6807\u8BB0\u2014\u20142026-10-08 \u6B63\u5F0F\u6682\u7F13" }] }, { name: "\u5218\u7109", faction: "qun", hp: 3, gender: "male", skills: [{ name: "juedao", mapping: "host:wudian_juedao" }, { name: "geju", mapping: "host:wudian_geju" }] }, { name: "\u7559\u8D5E", faction: "wu", hp: 4, gender: "male", skills: [{ name: "kangyin", mapping: "host:wudian_kangyin" }] }, { name: "\u7530\u8C6B", faction: "wei", hp: 4, gender: "male", skills: [{ name: "chezhen", mapping: "host:wudian_chezhen" }, { name: "youzhan", mapping: null, reason: "shaBefore \u65F6\u70B9\uFF08\u539F\u751F\u6620\u5C04 AfterTriggers \u504F\u665A\uFF09+\u4EE4\u76EE\u6807\u865A\u62DF useCard \u8BF1\u654C\u6DF1\u5165\u7EC4\u5408\u2014\u20142026-10-08 \u6B63\u5F0F\u6682\u7F13" }] }, { name: "\u897F\u771F\u5915\u7EA2", faction: "shu", hp: 4, gender: "male", skills: [{ name: "jinyan", mapping: "host:wudian_jinyan" }, { name: "fuchou", mapping: "host:fuchou" }, { name: "fuchou2", mapping: "host:fuchou2" }, { name: "jianbi", mapping: "host:wudian_jianbi" }] }, { name: "\u9020\u4E4B\u4EC1\u541B", faction: "wei", hp: 3, gender: "male", skills: [{ name: "liangce", mapping: "host:liangce_remain" }, { name: "jianbi", mapping: null }, { name: "diyjuntun", mapping: "host:wudian_diyjuntun" }] }, { name: "\u6768\u4EEA", faction: "shu", hp: 3, gender: "male", skills: [{ name: "choudu", mapping: null }, { name: "liduan", mapping: "host:wudian_liduan" }] }], rm = { source: Zd, upstream_path: em, upstream_commit: nm, pack: am, cards: tm, deferred: im, skills: sm, generals: om }, lm = { package_format_version: 1, namespace: "upstream-liuli", package_id: "wuxing", version: "1.0.0", content_schema_version: 3, rule_vm_abi_version: 734, min_engine_api: 1, max_engine_api: 1, capabilities: { emit_effects: true, run_queries: true, request_choice: true, read_skill_state: true, write_skill_state: true, read_marks_self: true, read_public_seats: true, _reserved: null }, logic_hash: [132, 94, 166, 250, 221, 202, 90, 254, 41, 167, 212, 147, 169, 28, 169, 39, 198, 78, 32, 87, 170, 107, 26, 155, 120, 216, 242, 221, 92, 75, 95, 25], asset_hash: [60, 107, 102, 15, 60, 169, 0, 83, 63, 35, 181, 217, 139, 24, 141, 42, 38, 138, 111, 102, 49, 137, 183, 48, 164, 187, 206, 145, 39, 244, 93, 134], author: "noname-next", license: { schema_version: 1, code_license: { type: "Unspecified" }, default_asset_license: { type: "Unspecified" }, asset_sources: [], redistribution: "FreeRedistribution", trademark_notes: null, changelog: [] } }, um = [], cm = [], dm = [], mm = [], pm = [], hm = [], gm = [], fm = [], _m = { "zh-CN": { fire: "\u706B", metal: "\u91D1", soil: "\u571F", water: "\u6C34", wood: "\u6728", wuxingpan: "\u4E94\u884C\u76D8", wuxingpan_skill: "\u4E94\u884C" } }, vm = { schema_version: 1, package_id: "wuxing", total_compressed_size: 0, total_decoded_size: 0, entries: [{ asset_id: "wuxing:config", source_hash: [209, 188, 252, 3, 138, 195, 23, 199, 174, 66, 240, 192, 138, 68, 185, 89, 196, 5, 25, 128, 172, 49, 209, 89, 211, 121, 60, 76, 41, 255, 75, 222], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }, { asset_id: "wuxing:cards", source_hash: [239, 161, 205, 89, 208, 240, 33, 205, 87, 200, 152, 104, 171, 72, 97, 172, 223, 238, 146, 97, 40, 117, 119, 75, 157, 86, 203, 219, 159, 135, 28, 177], kind: "Data", image_variants: [], audio_variants: [], priority: "Lazy", depends_on: [], platforms: [] }] }, ym = { "wuxing:cards": "assets/cards.json", "wuxing:config": "assets/wuxing.json" }, km = "gen-1", rt = { manifest: lm, modules: um, skill_index: cm, character_index: dm, card_index: mm, trigger_index: pm, modifier_index: hm, active_index: gm, view_as_index: fm, translations: _m, asset_manifest: vm, asset_paths: ym, compiler_version: km }, wm = "noname-liuli", bm = "noname/extension/wuxing/extension.js", xm = "8485dbb29ff16f9b258a766b1a17ef75f883aade", $m = { name: "wuxing-upstream-liuli", version: 1, description: "G1 wuxing\uFF1A\u4E94\u884C\u5C5E\u6027\u4E0E\u751F\u514B", card_prefix: "wxl_", init_hook: "wuxing_natures" }, jm = [{ name: "wuxingpan", kind: "wuxingpan", suit: "spade", rank: 5, equip_skills: ["wuxingpan_skill"] }], Sm = [{ name: "wuxing_shengke", mapping: "host:shengke_check" }, { name: "wuxingpan_skill", mapping: "host:change_card_nature" }], Em = { source: wm, upstream_path: bm, upstream_commit: xm, pack: $m, cards: jm, skills: Sm }, zm = { "cardpile-upstream-liuli": { package: Wa, payload: { manifest: Wa.manifest, pile: tr } }, "cardpile-upstream-noname": { package: Ka, payload: { manifest: Ka.manifest, pile: br } }, "d3-upstream-noname": { package: Ja, payload: { manifest: Ja.manifest, characters: Ur } }, "huanle-upstream-noname": { package: Qa, payload: { manifest: Qa.manifest, cards: rl } }, "shaai-ow-upstream-noname": { package: Ya, payload: { manifest: Ya.manifest, cards: Cl } }, "shaai-xianjian-upstream-noname": { package: Xa, payload: { manifest: Xa.manifest, cards: Zl } }, "shahai-gujian-upstream-noname": { package: Za, payload: { manifest: Za.manifest, cards: yu } }, "shahai-gwent-upstream-noname": { package: et, payload: { manifest: et.manifest, cards: Du } }, "shahai-hearth-upstream-noname": { package: nt, payload: { manifest: nt.manifest, cards: sc } }, "shahai-mtg-upstream-noname": { package: at, payload: { manifest: at.manifest, cards: jc } }, "shahai-swd-upstream-noname": { package: tt, payload: { manifest: tt.manifest, cards: Wc } }, "shahai-yunchou-upstream-noname": { package: it, payload: { manifest: it.manifest, cards: pd, characters: yd } }, "shahai-zhenfa-upstream-noname": { package: st, payload: { manifest: st.manifest, cards: Dd } }, "wudian-upstream-noname": { package: ot, payload: { manifest: ot.manifest, cards: rm } }, "wuxing-upstream-liuli": { package: rt, payload: { manifest: rt.manifest, cards: Em } } };
function xn(e) {
  return typeof e == "string" && e.trim().length > 0 && e !== "." && e !== ".." && !e.includes("/") && !e.includes("\\");
}
function Xn(e) {
  return Array.isArray(e) && e.length === 32 && e.every((n) => Number.isInteger(n) && n >= 0 && n <= 255);
}
function fn(e) {
  const n = e, a = n == null ? void 0 : n.manifest;
  if (!a || ![a.namespace, a.package_id, a.version].every(xn) || !Xn(a.logic_hash)) throw new Error("\u6269\u5C55\u5305\u8EAB\u4EFD\u6216\u903B\u8F91\u6307\u7EB9\u65E0\u6548");
  if (a.dependencies !== void 0) {
    if (!Array.isArray(a.dependencies)) throw new Error("\u6269\u5C55\u5305\u4F9D\u8D56\u683C\u5F0F\u65E0\u6548");
    const i = /* @__PURE__ */ new Set();
    for (const s of a.dependencies) {
      if (!s || typeof s != "object" || Array.isArray(s) || ![s.namespace, s.package_id, s.version].every(xn) || s.logic_hash !== void 0 && !Xn(s.logic_hash)) throw new Error("\u6269\u5C55\u5305\u4F9D\u8D56\u683C\u5F0F\u65E0\u6548");
      const l = `${s.namespace}/${s.package_id}`;
      if (i.has(l) || l === `${a.namespace}/${a.package_id}`) throw new Error(`\u6269\u5C55\u5305\u4F9D\u8D56\u91CD\u590D\u6216\u6307\u5411\u81EA\u8EAB: ${l}`);
      i.add(l);
    }
  }
  if ((n == null ? void 0 : n.modules) !== void 0 && !Array.isArray(n.modules)) throw new Error("\u6269\u5C55\u5305 VM \u6A21\u5757\u683C\u5F0F\u65E0\u6548");
  (n == null ? void 0 : n.modules) !== void 0 && Am(n);
  const t = [n == null ? void 0 : n.cards, n == null ? void 0 : n.characters, n == null ? void 0 : n.pile].filter((i) => i !== void 0);
  if (!t.length && (n == null ? void 0 : n.modules) === void 0 || t.some((i) => i === null || typeof i != "object" || Array.isArray(i))) throw new Error("\u6269\u5C55\u5305\u7F3A\u5C11\u53EF\u88C5\u8F7D\u7684\u5361\u724C\u3001\u6B66\u5C06\u3001\u724C\u5806\u6216 VM \u6570\u636E");
  return n;
}
function vn(e) {
  return Number.isInteger(e) && e >= 0;
}
function Am(e) {
  const n = e.modules, a = /* @__PURE__ */ new Map();
  for (const l of n) {
    const r = l;
    if (!r || typeof r != "object" || Array.isArray(r)) throw new Error("\u6269\u5C55\u5305 VM \u6A21\u5757\u683C\u5F0F\u65E0\u6548");
    if (!xn(r.name) || a.has(r.name)) throw new Error("\u6269\u5C55\u5305 VM \u6A21\u5757\u540D\u7F3A\u5931\u6216\u91CD\u590D");
    const u = r.code;
    if (!Array.isArray(u) || !u.length || !u.every((c) => Number.isInteger(c) && c >= 0 && c <= 255)) throw new Error(`\u6269\u5C55\u5305 VM \u6A21\u5757\u5B57\u8282\u7801\u65E0\u6548: ${r.name}`);
    if (![r.local_count, r.max_stack, r.entry_pc, r.caps_mask].every(vn) || r.entry_pc >= u.length) throw new Error(`\u6269\u5C55\u5305 VM \u6A21\u5757\u5165\u53E3\u6216\u9884\u7B97\u5B57\u6BB5\u65E0\u6548: ${r.name}`);
    if (!Xn(r.logic_hash)) throw new Error(`\u6269\u5C55\u5305 VM \u6A21\u5757\u6307\u7EB9\u683C\u5F0F\u65E0\u6548: ${r.name}`);
    a.set(r.name, u.length);
  }
  const t = /* @__PURE__ */ new Set(), i = e.skill_index;
  if (i !== void 0) {
    if (!Array.isArray(i)) throw new Error("\u6269\u5C55\u5305\u6280\u80FD\u7D22\u5F15\u683C\u5F0F\u65E0\u6548");
    for (const l of i) {
      const r = l;
      if (!r || typeof r != "object" || !vn(r.skill_def_id) || !xn(r.name) || t.has(r.skill_def_id)) throw new Error("\u6269\u5C55\u5305\u6280\u80FD\u7D22\u5F15\u6761\u76EE\u65E0\u6548\u6216\u91CD\u590D");
      t.add(r.skill_def_id);
    }
  }
  const s = e.trigger_index;
  if (s !== void 0) {
    if (!Array.isArray(s)) throw new Error("\u6269\u5C55\u5305\u89E6\u53D1\u7D22\u5F15\u683C\u5F0F\u65E0\u6548");
    for (const l of s) {
      const r = l;
      if (!r || typeof r != "object" || typeof r.module != "string" || !a.has(r.module)) throw new Error("\u6269\u5C55\u5305\u89E6\u53D1\u7D22\u5F15\u5F15\u7528\u4E86\u4E0D\u5B58\u5728\u7684 VM \u6A21\u5757");
      if (!vn(r.entry_pc) || r.entry_pc >= a.get(r.module) || !vn(r.skill_def_id) || i !== void 0 && !t.has(r.skill_def_id)) throw new Error(`\u6269\u5C55\u5305\u89E6\u53D1\u7D22\u5F15\u5165\u53E3\u6216\u6280\u80FD\u5F15\u7528\u65E0\u6548: ${r.module}`);
    }
  }
}
function Ve(e) {
  return `${e.manifest.namespace}/${e.manifest.package_id}`;
}
function _n(e) {
  return e.manifest.logic_hash.map((n) => n.toString(16).padStart(2, "0")).join("");
}
function Ut(e) {
  const n = zm[e.dir];
  if (!n) throw new Error(`\u65E0\u53EF\u5B89\u88C5\u5305\u6570\u636E: ${e.dir}`);
  if (!Array.isArray(n.package.modules)) throw new Error(`\u6269\u5C55\u5305 VM \u6A21\u5757\u683C\u5F0F\u65E0\u6548: ${e.dir}`);
  const a = n.package.modules.length ? { ...n.package, ...n.payload } : n.payload, t = fn(a);
  if (Ve(t) !== `${e.namespace}/${e.packageId}` || t.manifest.version !== e.version || !_n(t).startsWith(e.logicFingerprint)) throw new Error(`\u5305\u76EE\u5F55\u4E0E\u5B9E\u9645\u5B89\u88C5\u5185\u5BB9\u4E0D\u4E00\u81F4: ${e.dir}`);
  return JSON.stringify(t);
}
const Vt = "noname-next.extension-store.v1", Gt = "noname-next.native-saves.v1";
function yn() {
  return { installed: [], contentLock: [], matchFrozen: false };
}
function ue() {
  if (typeof localStorage > "u") return yn();
  try {
    const e = localStorage.getItem(Vt);
    return e ? { ...yn(), ...JSON.parse(e) } : yn();
  } catch {
    return yn();
  }
}
function xe(e) {
  typeof localStorage > "u" || localStorage.setItem(Vt, JSON.stringify(e));
}
function Fe() {
  return (/* @__PURE__ */ new Date()).toISOString();
}
function Bt(e) {
  return dn.find((n) => ca(n) === e);
}
function da(e, n, a) {
  const t = e == null ? void 0 : e[n];
  if (t === void 0) return;
  const i = (s) => JSON.stringify(JSON.parse(s), (l, r) => r === null || typeof r != "object" || Array.isArray(r) ? r : Object.fromEntries(Object.entries(r).sort(([u], [c]) => u < c ? -1 : u > c ? 1 : 0)));
  if (i(t) !== i(a)) throw new Error(`\u540C\u7248\u672C\u5305\u5185\u5BB9\u51B2\u7A81: ${n}`);
}
function $n() {
  return ue();
}
function lt(e) {
  const n = ue();
  return n.matchFrozen = e, xe(n), n;
}
function Ft(e) {
  var _a2;
  const n = Bt(e);
  if (!n) throw new Error(`\u672A\u77E5\u6269\u5C55: ${e}`);
  const a = ue();
  if (a.matchFrozen) throw new Error("\u5BF9\u5C40\u4E2D\u4E0D\u53EF\u5B89\u88C5\u6216\u66FF\u6362\u5305\u5185\u5BB9");
  if ((_a2 = a.revokedKeys) == null ? void 0 : _a2.includes(e)) throw new Error(`\u5DF2\u64A4\u9500\u5305\u4E0D\u53EF\u91CD\u88C5: ${e}`);
  const t = Ut(n), i = a.installed.find((l) => l.key === e), s = Fe();
  if (i) {
    if (i.lifecycle === "revoked") throw new Error(`\u5DF2\u64A4\u9500\u5305\u4E0D\u53EF\u91CD\u88C5: ${e}\uFF08\u9700\u6E05\u9664\u64A4\u9500\u8BB0\u5F55\uFF09`);
    da(i.nativeVersions, n.version, t), i.updatedAt = s, i.nativeVersions ?? (i.nativeVersions = {}), i.version === n.version && !i.nativeVersions[n.version] && (i.nativeVersions[n.version] = t), i.lifecycle === "available" && (i.lifecycle = "installed");
  } else a.installed.push({ key: e, dir: n.dir, packageId: n.packageId, namespace: n.namespace, version: n.version, history: [n.version], lifecycle: "installed", installedAt: s, updatedAt: s, logicFingerprint: n.logicFingerprint, nativeVersions: { [n.version]: t } });
  return xe(a), a;
}
async function Cm(e, n) {
  const a = Rm(e);
  return await n(a.canonical), Nm(a);
}
function Rm(e) {
  const n = ue();
  if (n.matchFrozen) throw new Error("\u5BF9\u5C40\u4E2D\u4E0D\u53EF\u5B89\u88C5\u6216\u66FF\u6362\u5305\u5185\u5BB9");
  let a;
  try {
    a = JSON.parse(e);
  } catch {
    throw new Error("\u6269\u5C55\u5305\u4E0D\u662F\u5408\u6CD5 JSON");
  }
  const t = fn(a), i = Ve(t), s = t.manifest.version, l = JSON.stringify(t);
  return Ht(n, i, s, l), { payload: t, key: i, version: s, canonical: l, fingerprint: _n(t).slice(0, 8) };
}
function Ht(e, n, a, t) {
  var _a2;
  if (e.matchFrozen) throw new Error("\u5BF9\u5C40\u4E2D\u4E0D\u53EF\u5B89\u88C5\u6216\u66FF\u6362\u5305\u5185\u5BB9");
  if ((_a2 = e.revokedKeys) == null ? void 0 : _a2.includes(n)) throw new Error(`\u5DF2\u64A4\u9500\u5305\u4E0D\u53EF\u91CD\u88C5: ${n}`);
  const i = e.installed.find((s) => s.key === n);
  if ((i == null ? void 0 : i.lifecycle) === "revoked") throw new Error(`\u5DF2\u64A4\u9500\u5305\u4E0D\u53EF\u91CD\u88C5: ${n}\uFF08\u9700\u6E05\u9664\u64A4\u9500\u8BB0\u5F55\uFF09`);
  i && da(i.nativeVersions, a, t);
}
function Nm(e) {
  var _a2;
  const n = ue(), { payload: a, key: t, version: i, canonical: s, fingerprint: l } = e;
  Ht(n, t, i, s);
  const r = n.installed.find((c) => c.key === t), u = Fe();
  return r ? (r.nativeVersions ?? (r.nativeVersions = {}), (_a2 = r.nativeVersions)[i] ?? (_a2[i] = s), r.version !== i && (r.history.includes(r.version) || r.history.push(r.version), r.version = i, r.history.includes(i) || r.history.push(i)), r.logicFingerprint = l, r.updatedAt = u, r.signature = "unsigned", r.lifecycle === "available" && (r.lifecycle = "installed")) : n.installed.push({ key: t, dir: "imported", packageId: a.manifest.package_id, namespace: a.manifest.namespace, version: i, history: [i], lifecycle: "installed", installedAt: u, updatedAt: u, logicFingerprint: l, nativeVersions: { [i]: s }, signature: "unsigned" }), xe(n), n;
}
function Wt(e) {
  const n = ue();
  if (n.matchFrozen) throw new Error("\u5BF9\u5C40\u4E2D\u4E0D\u53EF\u53D8\u66F4\u5185\u5BB9\u9501");
  const a = n.installed.find((t) => t.key === e);
  if (!a || a.lifecycle === "revoked") throw new Error(`\u672A\u5B89\u88C5\u6216\u5DF2\u64A4\u9500: ${e}`);
  return a.lifecycle = "active", a.updatedAt = Fe(), n.contentLock.includes(e) || n.contentLock.push(e), n.contentLock.sort(), xe(n), n;
}
function Pm(e) {
  const n = ue();
  if (n.matchFrozen) throw new Error("\u5BF9\u5C40\u4E2D\u4E0D\u53EF\u53D8\u66F4\u5185\u5BB9\u9501");
  const a = n.installed.find((t) => t.key === e);
  if (!a) throw new Error(`\u672A\u5B89\u88C5: ${e}`);
  if (a.lifecycle === "revoked") throw new Error(`\u5DF2\u64A4\u9500: ${e}`);
  return a.lifecycle = "disabled", a.updatedAt = Fe(), n.contentLock = n.contentLock.filter((t) => t !== e), xe(n), n;
}
function qm(e, n) {
  var _a2;
  const a = Bt(e), t = ue();
  if (!a) throw t.installed.some((r) => r.key === e) ? new Error(`\u5BFC\u5165\u5305\u6CA1\u6709\u5185\u7F6E\u76EE\u5F55\u7248\u672C\uFF0C\u8BF7\u5BFC\u5165\u65B0\u7248\u672C\u5305\u4EE5\u5347\u7EA7: ${e}`) : new Error(`\u672A\u77E5\u6269\u5C55: ${e}`);
  if (t.matchFrozen) throw new Error("\u5BF9\u5C40\u4E2D\u4E0D\u53EF\u5347\u7EA7\uFF08\u907F\u514D\u5185\u5BB9\u9501\u6F02\u79FB\uFF09");
  const i = t.installed.find((r) => r.key === e);
  if (!i || i.lifecycle === "revoked") throw new Error(`\u672A\u5B89\u88C5\u6216\u5DF2\u64A4\u9500: ${e}`);
  const s = n ?? a.version;
  if (s !== a.version) throw new Error(`\u65E0\u53EF\u5B89\u88C5\u7684\u7248\u672C ${s}`);
  const l = Ut(a);
  return da(i.nativeVersions, s, l), i.nativeVersions ?? (i.nativeVersions = {}), (_a2 = i.nativeVersions)[s] ?? (_a2[s] = l), i.version !== s && (i.history.includes(i.version) || i.history.push(i.version), i.version = s, i.history.includes(s) || i.history.push(s)), i.logicFingerprint = a.logicFingerprint, i.updatedAt = Fe(), i.lifecycle === "installed" || i.lifecycle, xe(t), t;
}
function Im(e, n) {
  var _a2;
  const a = ue();
  if (a.matchFrozen) throw new Error("\u5BF9\u5C40\u4E2D\u4E0D\u53EF\u56DE\u9000");
  const t = a.installed.find((s) => s.key === e);
  if (!t || t.lifecycle === "revoked") throw new Error(`\u672A\u5B89\u88C5\u6216\u5DF2\u64A4\u9500: ${e}`);
  if (!t.history.includes(n)) throw new Error(`\u65E0\u5386\u53F2\u7248\u672C ${n}`);
  if (!((_a2 = t.nativeVersions) == null ? void 0 : _a2[n])) throw new Error(`\u7F3A\u5C11\u5DF2\u5B89\u88C5\u7684\u7248\u672C\u5185\u5BB9 ${n}`);
  const i = fn(JSON.parse(t.nativeVersions[n]));
  if (Ve(i) !== e || i.manifest.version !== n) throw new Error(`\u5386\u53F2\u5305\u8EAB\u4EFD\u6216\u7248\u672C\u4E0D\u5339\u914D: ${e}`);
  return t.version = n, t.logicFingerprint = _n(i).slice(0, 8), t.updatedAt = Fe(), xe(a), a;
}
function Om(e) {
  const n = ue();
  if (n.matchFrozen) throw new Error("\u5BF9\u5C40\u4E2D\u4E0D\u53EF\u5378\u8F7D");
  return n.installed.some((a) => a.key === e && a.lifecycle === "revoked") && (n.revokedKeys ?? (n.revokedKeys = []), n.revokedKeys.includes(e) || n.revokedKeys.push(e)), n.contentLock = n.contentLock.filter((a) => a !== e), n.installed = n.installed.filter((a) => a.key !== e), xe(n), n;
}
function Tm(e) {
  const n = ue();
  if (n.matchFrozen) throw new Error("\u5BF9\u5C40\u4E2D\u4E0D\u53EF\u64A4\u9500");
  const a = n.installed.find((t) => t.key === e);
  if (!a) throw new Error(`\u672A\u5B89\u88C5: ${e}`);
  return a.lifecycle = "revoked", n.revokedKeys ?? (n.revokedKeys = []), n.revokedKeys.includes(e) || n.revokedKeys.push(e), a.updatedAt = Fe(), n.contentLock = n.contentLock.filter((t) => t !== e), xe(n), n;
}
function Lm(e = ue()) {
  const n = e.contentLock.map((t) => {
    const i = e.installed.find((s) => s.key === t);
    return i ? { key: t, version: i.version, fingerprint: i.logicFingerprint } : null;
  }).filter(Boolean), a = n.map((t) => `${t.key}@${t.version}:${t.fingerprint}`).join("|");
  return { packages: n, hashSeed: a };
}
function Dm() {
  let e = ue();
  for (const n of dn) {
    const a = ca(n);
    e = Ft(a), e = Wt(a);
  }
  return e;
}
function Kt() {
  if (typeof localStorage > "u") return [];
  try {
    const e = localStorage.getItem(Gt);
    return e ? JSON.parse(e) : [];
  } catch {
    return [];
  }
}
function Mm(e) {
  const n = Kt().filter((a) => a.id !== e.id);
  return n.unshift(e), localStorage.setItem(Gt, JSON.stringify(n.slice(0, 20))), n;
}
function Jt(e) {
  const n = e.map(Ve);
  if (new Set(n).size !== n.length) throw new Error("\u5185\u5BB9\u9501\u5305\u542B\u91CD\u590D\u6269\u5C55");
  const a = new Map(e.map((t) => [Ve(t), t]));
  for (const t of e) for (const i of t.manifest.dependencies ?? []) {
    const s = `${i.namespace}/${i.package_id}`, l = a.get(s);
    if (!l) throw new Error(`\u6269\u5C55\u5305\u7F3A\u5C11\u4F9D\u8D56: ${s}@${i.version}`);
    const r = i.logic_hash;
    if (l.manifest.version !== i.version || (r == null ? void 0 : r.some((u) => u !== 0)) && r.some((u, c) => u !== l.manifest.logic_hash[c])) throw new Error(`\u6269\u5C55\u5305\u4F9D\u8D56\u7248\u672C\u6216\u6307\u7EB9\u4E0D\u5339\u914D: ${s}@${i.version}`);
  }
  return { packsJson: e.length ? JSON.stringify(e) : void 0, contentLock: n, lockSeed: e.map((t) => `${Ve(t)}@${t.manifest.version}:${_n(t)}`).join("|") };
}
function Um(e = $n()) {
  const n = e.contentLock.map((a) => {
    var _a2;
    const t = e.installed.find((l) => l.key === a);
    if (!t || t.lifecycle !== "active") throw new Error(`\u5185\u5BB9\u9501\u4E2D\u7684\u5305\u672A\u542F\u7528: ${a}`);
    const i = (_a2 = t.nativeVersions) == null ? void 0 : _a2[t.version];
    if (!i) throw new Error(`\u7F3A\u5C11\u5DF2\u5B89\u88C5\u5305\u5185\u5BB9: ${a}\uFF0C\u8BF7\u5148\u5728\u6269\u5C55\u7BA1\u7406\u9875\u5347\u7EA7\u6216\u91CD\u65B0\u5B89\u88C5`);
    const s = fn(JSON.parse(i));
    if (Ve(s) !== a || s.manifest.version !== t.version || !/^[a-f0-9]{8,64}$/.test(t.logicFingerprint) || !_n(s).startsWith(t.logicFingerprint)) throw new Error(`\u5DF2\u5B89\u88C5\u5305\u7248\u672C\u6216\u6307\u7EB9\u4E0D\u5339\u914D: ${a}`);
    return s;
  });
  return Jt(n);
}
function Vm(e, n = $n()) {
  var _a2;
  const a = e.packsJson ? JSON.parse(e.packsJson) : [];
  if (!Array.isArray(a)) throw new Error("\u5B58\u6863\u6269\u5C55\u5185\u5BB9\u5FC5\u987B\u662F\u6570\u7EC4");
  const t = Jt(a.map(fn));
  if (t.lockSeed !== e.lockSeed || JSON.stringify(t.contentLock) !== JSON.stringify(e.contentLock)) throw new Error("\u5B58\u6863\u5185\u5BB9\u9501\u4E0E\u5305\u526F\u672C\u4E0D\u5339\u914D\uFF1B\u65E7\u6269\u5C55\u5B58\u6863\u7F3A\u5C11\u5B9E\u9645\u5305\u5185\u5BB9\u65F6\u4E0D\u80FD\u6062\u590D");
  for (const i of t.contentLock) if (((_a2 = n.revokedKeys) == null ? void 0 : _a2.includes(i)) || n.installed.some((s) => s.key === i && s.lifecycle === "revoked")) throw new Error(`\u5B58\u6863\u5305\u542B\u5DF2\u64A4\u9500\u5305: ${i}`);
  return t;
}
const Gm = { 1: "\u5BF9\u5C40\u5F00\u59CB", 2: "\u6478\u724C", 3: "\u4F7F\u7528\u6216\u54CD\u5E94\u724C", 4: "\u4F24\u5BB3\u7ED3\u7B97", 5: "\u56DE\u590D\u4F53\u529B", 6: "\u6FD2\u6B7B\u72B6\u6001\u53D8\u5316", 7: "\u73A9\u5BB6\u9635\u4EA1", 8: "\u9636\u6BB5\u6216\u56DE\u5408\u63A8\u8FDB", 9: "\u5BF9\u5C40\u7ED3\u675F", 10: "\u9009\u62E9\u72B6\u6001\u53D8\u5316" };
function Bm(e) {
  return Gm[e] ?? `\u4E8B\u4EF6 ${e}`;
}
const tn = 6, jn = "noname-next.pending-command", Fm = 600 * 1e3;
function Hm() {
  try {
    if (typeof sessionStorage > "u") return null;
    const e = sessionStorage.getItem(jn);
    if (!e) return null;
    const n = JSON.parse(e);
    return !n || typeof n.roomId != "number" || typeof n.key != "number" || typeof n.createdAt != "number" || !n.kind || Date.now() - n.createdAt > Fm ? null : { roomId: n.roomId, key: n.key, kind: n.kind, createdAt: n.createdAt };
  } catch {
    return null;
  }
}
function Wm(e, n, a) {
  try {
    if (typeof sessionStorage > "u" || !a) return;
    sessionStorage.setItem(jn, JSON.stringify({ roomId: a, key: n, kind: e, createdAt: Date.now() }));
  } catch {
  }
}
function kn(e) {
  try {
    if (typeof sessionStorage > "u") return;
    const n = sessionStorage.getItem(jn);
    if (!n) return;
    JSON.parse(n).key === e && sessionStorage.removeItem(jn);
  } catch {
  }
}
class Km {
  constructor() {
    this.socket = null, this.connectionId = 0, this.roomId = 0, this.playerId = 0, this.revision = 0, this.connSeq = 0, this.idemSeq = 1, this.choice = null, this.session = null, this.roomPassword = "", this.welcomeWaiter = null, this.joinWaiter = null, this.pending = /* @__PURE__ */ new Map(), this.persistedPending = Hm(), this.snapshotWaiters = /* @__PURE__ */ new Map(), this.envelopeListeners = /* @__PURE__ */ new Set(), this.rosterListeners = /* @__PURE__ */ new Set(), this.closeListeners = /* @__PURE__ */ new Set(), this.heartbeat = null, this.clockProbe = null, this.serverTimeAnchor = null, this.disposed = false, this.lastUrl = null, this.lastName = "", this.reconnectTimer = null, this.reconnectAttempts = 0, this.reconnectListeners = /* @__PURE__ */ new Set();
  }
  async connect(n, a) {
    var _a2;
    if (((_a2 = this.socket) == null ? void 0 : _a2.readyState) === WebSocket.OPEN) return;
    this.disposed = false, this.clockProbe = null, this.serverTimeAnchor = null, this.lastUrl = n, this.lastName = a;
    const t = new Promise((s, l) => {
      this.welcomeWaiter = { resolve: s, reject: l };
    }), i = new Promise((s, l) => {
      const r = new WebSocket(Qt(n));
      this.socket = r, r.onopen = () => {
        if (this.socket !== r) {
          l(new Error("WebSocket \u8FDE\u63A5\u5DF2\u66FF\u6362"));
          return;
        }
        this.connSeq = 0, this.idemSeq = 1, s();
      }, r.onerror = () => l(new Error("WebSocket \u8FDE\u63A5\u5931\u8D25")), r.onmessage = (u) => {
        this.socket === r && this.handleMessage(String(u.data));
      }, r.onclose = () => {
        l(new Error("WebSocket \u8FDE\u63A5\u5DF2\u5173\u95ED")), this.socket === r && this.handleClose();
      };
    });
    t.catch(() => {
    }), await i, await t, this.send({ type: "Hello", payload: { versions: { engine_api: 1, rule_vm_abi: 167, content_schema: 3, network_protocol: tn, replay_schema: 1, snapshot_schema: 1, package_format: 1, engine_build_id: "noname-next-web" }, display_name: a } }), this.sendHeartbeat();
  }
  secondsUntilServerDeadline(n) {
    if (n == null || !this.serverTimeAnchor) return null;
    const a = performance.now() - this.serverTimeAnchor.monotonicMs;
    return Math.max(0, Math.ceil((n - this.serverTimeAnchor.unixMs - a) / 1e3));
  }
  createRoom(n, a, t = "", i = 30) {
    return this.joinRequest({ type: "CreateRoom", payload: { players: n, seed: a, password: t, choice_timeout_secs: i } }, t);
  }
  joinRoom(n, a = "", t = null) {
    return this.roomId = n, this.joinRequest({ type: "JoinRoom", payload: { room_id: n, password: a, preferred_seat: null, reconnect_token: t } }, a);
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
    for (const a of this.pending.values()) kn(a.key), a.reject(new Error(n));
    this.pending.clear(), this.persistedPending && (kn(this.persistedPending.key), this.persistedPending = null);
  }
  endPhase(n = 0) {
    return this.submit("EndPhase");
  }
  playCard(n, a, t) {
    return this.submit({ PlayCard: { card: a, targets: t, card_targets: [] } });
  }
  activateSkill(n, a, t, i) {
    return this.submit({ ActivateSkill: { skill: a, targets: t, card_targets: i } });
  }
  useSkillCard(n, a, t, i, s) {
    return this.submit({ UseSkillCard: { skill: a, card_kind: t, targets: i, equip_slot: s } });
  }
  resolveChoice(n, a, t, i = false) {
    return this.submit({ ResolveChoice: { choice_id: a, selections: t, cancelled: i } });
  }
  surrender() {
    return this.submit("Surrender");
  }
  requestSnapshot() {
    const n = this.nextHeader();
    return new Promise((a, t) => {
      this.snapshotWaiters.set(n.conn_seq, { resolve: a, reject: t });
      try {
        this.send({ type: "RequestSnapshot", payload: { header: n, last_revision: this.revision } });
      } catch (i) {
        this.snapshotWaiters.delete(n.conn_seq), t(i);
      }
    });
  }
  dispose() {
    this.disposed = true, this.reconnectTimer != null && (window.clearTimeout(this.reconnectTimer), this.reconnectTimer = null), this.reconnectAttempts = 0, this.heartbeat != null && typeof window < "u" && window.clearInterval(this.heartbeat), this.heartbeat = null, this.clockProbe = null, this.serverTimeAnchor = null;
    const n = this.socket;
    this.socket = null, n == null ? void 0 : n.close(1e3, "client dispose"), this.session = null, this.roomPassword = "", this.playerId = 0, this.rejectOutstanding(new Error("\u8FDE\u63A5\u5DF2\u5173\u95ED")), this.envelopeListeners.clear(), this.rosterListeners.clear(), this.closeListeners.clear(), this.reconnectListeners.clear();
  }
  async joinRequest(n, a) {
    var _a2;
    const t = this.waitForJoin();
    try {
      this.send(n), this.roomPassword = a;
    } catch (i) {
      (_a2 = this.joinWaiter) == null ? void 0 : _a2.reject(i instanceof Error ? i : new Error(String(i))), this.joinWaiter = null;
    }
    return t;
  }
  waitForJoin() {
    if (this.joinWaiter) throw new Error("\u5DF2\u6709\u5165\u623F\u8BF7\u6C42");
    return new Promise((n, a) => {
      this.joinWaiter = { resolve: n, reject: a };
    });
  }
  submit(n) {
    if (!this.roomId || !this.playerId) return Promise.reject(new Error("\u5F53\u524D\u4E0D\u662F\u5DF2\u5165\u5EA7\u73A9\u5BB6"));
    const a = this.connectionId * 1e6 + this.idemSeq++;
    let t;
    const i = new Promise((s, l) => {
      t = { kind: n, key: a, resolve: s, reject: l }, this.pending.set(a, t);
    });
    try {
      this.sendSubmit(t), this.persistedPending = { roomId: this.roomId, key: a, kind: n, createdAt: Date.now() }, Wm(n, a, this.roomId);
    } catch (s) {
      return this.pending.delete(a), kn(a), Promise.reject(s);
    }
    return i;
  }
  sendSubmit(n, a = false) {
    this.send({ type: "SubmitCommand", payload: { header: this.nextHeader(), command: { actor: this.playerId, expected_revision: this.revision, idempotency_key: n.key, kind: n.kind }, idempotency_key: n.key, replay: a } });
  }
  replayPending() {
    const n = this.persistedPending && this.persistedPending.roomId === this.roomId ? this.persistedPending : null;
    if (n && !this.pending.has(n.key)) {
      const a = { kind: n.kind, key: n.key, resolve: (t) => {
        for (const i of this.envelopeListeners) i(t);
      }, reject: () => {
      } };
      this.pending.set(a.key, a);
    }
    if (this.pending.size) for (const a of [...this.pending.values()]) try {
      this.sendSubmit(a, true);
    } catch {
      return;
    }
  }
  nextHeader() {
    return { protocol_version: tn, room_id: this.roomId, conn_seq: ++this.connSeq, revision: this.revision };
  }
  send(n) {
    var _a2;
    if (((_a2 = this.socket) == null ? void 0 : _a2.readyState) !== WebSocket.OPEN) throw new Error("WebSocket \u672A\u8FDE\u63A5");
    this.socket.send(JSON.stringify(n));
  }
  handleMessage(n) {
    var _a2, _b, _c2, _d2;
    let a;
    try {
      a = JSON.parse(n);
    } catch {
      this.rejectOutstanding(new Error("\u670D\u52A1\u7AEF\u8FD4\u56DE\u4E86\u65E0\u6548 JSON"));
      return;
    }
    const t = a.payload ?? {};
    if (t.view) try {
      gi(t.view, t.choice ?? null);
    } catch (i) {
      this.rejectOutstanding(new Error(`\u670D\u52A1\u7AEF\u5143\u6570\u636E\u65E0\u6548\uFF1A${String(i)}`));
      return;
    }
    switch (a.type) {
      case "Welcome":
        if (t.protocol_version !== tn || ((_a2 = t.versions) == null ? void 0 : _a2.network_protocol) !== tn) {
          this.rejectOutstanding(new Error("\u670D\u52A1\u7AEF\u534F\u8BAE\u7248\u672C\u4E0D\u517C\u5BB9\uFF1A\u9700\u8981 WS6")), (_b = this.socket) == null ? void 0 : _b.close();
          return;
        }
        this.connectionId = Number(t.connection_id ?? 0), (_c2 = this.welcomeWaiter) == null ? void 0 : _c2.resolve(), this.welcomeWaiter = null;
        break;
      case "RoomCreated":
        this.roomId = Number(t.room_id ?? 0);
        break;
      case "JoinOk": {
        this.roomId = Number(t.room_id ?? this.roomId), this.playerId = Number(t.player_id ?? 0);
        const i = t.view;
        this.choice = t.choice ?? null, this.revision = i.revision;
        const s = { roomId: this.roomId, playerId: this.playerId, seat: Number(t.seat ?? 255), role: String(t.role ?? "Spectator"), reconnectToken: String(t.reconnect_token ?? ""), view: i, envelope: Mn(i, this.choice, [], t.legal_actions ?? [], null) };
        this.session = s, this.replayPending(), (_d2 = this.joinWaiter) == null ? void 0 : _d2.resolve(s), this.joinWaiter = null, this.startHeartbeat();
        break;
      }
      case "RoomRoster": {
        const i = t;
        if (i.choice_timeout_secs !== null && ![10, 15, 30, 60, 90].includes(i.choice_timeout_secs) || !Array.isArray(i.seats) || i.seats.some((s) => typeof s.managed != "boolean")) {
          this.rejectOutstanding(new Error("\u670D\u52A1\u7AEF\u623F\u95F4\u8D85\u65F6\u914D\u7F6E\u65E0\u6548"));
          return;
        }
        for (const s of this.rosterListeners) s(i);
        break;
      }
      case "CommandOutcome":
        this.handleOutcome(t);
        break;
      case "Snapshot": {
        const i = t.header, s = t.view;
        this.choice = t.choice ?? null, this.revision = s.revision;
        const l = this.snapshotWaiters.get(i.conn_seq);
        l && (this.snapshotWaiters.delete(i.conn_seq), l.resolve(s));
        const r = Mn(s, this.choice, [], t.legal_actions ?? [], null);
        for (const u of this.envelopeListeners) u(r);
        break;
      }
      case "Pong": {
        const i = t.header, s = this.clockProbe, l = performance.now(), r = t.server_time_ms;
        s && (i == null ? void 0 : i.protocol_version) === tn && i.conn_seq === s.connSeq && i.room_id === s.roomId && typeof r == "number" && Number.isSafeInteger(r) && r > 0 && l >= s.sentAt && (this.serverTimeAnchor = { unixMs: r + (l - s.sentAt) / 2, monotonicMs: l }, this.clockProbe = null);
        break;
      }
      case "Error": {
        const i = new Error(String(t.message ?? "\u670D\u52A1\u7AEF\u9519\u8BEF"));
        this.joinWaiter ? (this.joinWaiter.reject(i), this.joinWaiter = null) : this.rejectOutstanding(i);
        break;
      }
    }
  }
  handleOutcome(n) {
    var _a2;
    const a = n.view, t = this.revision;
    this.revision = a.revision, this.choice = n.choice ?? null;
    const i = n.public_event_codes ?? [], s = Mn(a, this.choice, i, n.legal_actions ?? [], n.result ?? null), l = Number(n.idempotency_key ?? 0);
    kn(l), ((_a2 = this.persistedPending) == null ? void 0 : _a2.key) === l && (this.persistedPending = null);
    const r = this.pending.get(l);
    if (r) {
      this.pending.delete(l);
      const u = Ym(n.result);
      if (u) {
        for (const c of this.envelopeListeners) c(s);
        r.reject(new Error(u));
      } else r.resolve(s);
    } else for (const u of this.envelopeListeners) u(s);
    t > 0 && a.revision > t + 1 && this.requestSnapshot().catch(() => {
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
      for (const a of this.reconnectListeners) a("failed");
      return;
    }
    const n = Math.min(1e3 * 2 ** this.reconnectAttempts, 8e3);
    this.reconnectAttempts += 1;
    for (const a of this.reconnectListeners) a("reconnecting");
    this.reconnectTimer = window.setTimeout(() => {
      this.reconnectTimer = null;
      const a = this.lastUrl;
      !a || this.disposed || this.connect(a, this.lastName).then(() => {
        this.reconnectAttempts = 0;
        for (const t of this.reconnectListeners) t("reconnected");
      }).catch(() => {
        this.scheduleReconnect();
      });
    }, n);
  }
  onReconnect(n) {
    return this.reconnectListeners.add(n), () => this.reconnectListeners.delete(n);
  }
  suspendOutstanding(n) {
    var _a2, _b;
    (_a2 = this.welcomeWaiter) == null ? void 0 : _a2.reject(n), this.welcomeWaiter = null, (_b = this.joinWaiter) == null ? void 0 : _b.reject(n), this.joinWaiter = null;
    for (const a of this.snapshotWaiters.values()) a.reject(n);
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
    for (const a of this.pending.values()) a.reject(n);
    this.pending.clear();
  }
}
function Qt(e) {
  const n = e.trim();
  return n.startsWith("http://") ? `ws://${n.slice(7)}` : n.startsWith("https://") ? `wss://${n.slice(8)}` : n;
}
function ut(e, n) {
  const a = new URL(Qt(e)).origin;
  return `noname-next.reconnect.${encodeURIComponent(a)}.${n}`;
}
function Mn(e, n, a, t, i) {
  return { result: i, events: a.map((s) => ({ PublicEvent: { code: s, label: Bm(s) } })), choice: n, view: e, legal_actions: t, hash_hex: Jm(e.state_hash), revision: e.revision, finished: Qm(e.outcome) };
}
function Jm(e) {
  return typeof e == "string" ? e : e.map((n) => n.toString(16).padStart(2, "0")).join("");
}
function Qm(e) {
  return e == null || e === "InProgress" ? false : !(typeof e == "object" && "InProgress" in e);
}
function Ym(e) {
  var _a2;
  if (!e || typeof e != "object" || !("Rejected" in e)) return null;
  const n = e.Rejected;
  return (n && typeof n == "object" && "IllegalAction" in n ? (_a2 = n.IllegalAction) == null ? void 0 : _a2.code : void 0) === "legacy_replay_no_receipt" ? "\u63D0\u4EA4\u7ED3\u679C\u4E0D\u786E\u5B9A\uFF1A\u65E7\u623F\u95F4\u6CA1\u6709\u53EF\u6838\u5BF9\u7684\u56DE\u6267\uFF0C\u5DF2\u505C\u6B62\u91CD\u653E\uFF1B\u8BF7\u4EE5\u623F\u95F4\u5F53\u524D\u72B6\u6001\u4E3A\u51C6" : `\u547D\u4EE4\u88AB\u62D2\u7EDD\uFF1A${JSON.stringify(n)}`;
}
function Yt(e, n) {
  var _a2, _b;
  const a = (s) => ({ label: s, isSelfWin: null });
  if (e == null) return a("\u80DC\u8D1F\u672A\u77E5");
  if (e === "finished") return a("\u5DF2\u7ED3\u675F\uFF08\u80DC\u8D1F\u7C7B\u578B\u672A\u5EFA\u6A21\uFF09");
  const t = typeof e == "object" ? e : {};
  switch (typeof e == "string" ? e : Object.keys(t)[0]) {
    case "LoyalistWin":
      return { label: "\u4E3B\u516C\u65B9\u80DC", isSelfWin: (n == null ? void 0 : n.identity) ? n.identity === "Lord" || n.identity === "Loyalist" : null };
    case "RebelWin":
      return { label: "\u53CD\u8D3C\u80DC", isSelfWin: (n == null ? void 0 : n.identity) ? n.identity === "Rebel" : null };
    case "RenegadeWin":
      return { label: "\u5185\u5978\u80DC", isSelfWin: (n == null ? void 0 : n.identity) ? n.identity === "Renegade" : null };
    case "SingleWin": {
      const s = (_a2 = t.SingleWin) == null ? void 0 : _a2.player;
      if (typeof s == "number" && Number.isInteger(s) && s >= 0) return { label: `\u5355\u6311\u80DC\u8005 P${s}`, isSelfWin: n ? n.player === s : null };
      break;
    }
    case "GroupWin": {
      const s = (_b = t.GroupWin) == null ? void 0 : _b.group;
      if (typeof s == "string") return a(`\u52BF\u529B\u80DC\uFF08${s}\uFF09`);
      break;
    }
    case "Draw":
      return a("\u5E73\u5C40");
    case "Aborted":
      return a("\u5BF9\u5C40\u4E2D\u6B62");
  }
  return a(`\u5DF2\u7ED3\u675F\uFF08${JSON.stringify(e)}\uFF09`);
}
function Xm(e) {
  return Yt(e).label;
}
function Zm(e, n, a) {
  var _a2;
  let t = e == null ? void 0 : e.outcome;
  if (t == null) for (const i of n) {
    if (!i || typeof i != "object") continue;
    const s = i;
    if (t = s.type === "MatchEnded" ? s.outcome : (_a2 = s.MatchEnded) == null ? void 0 : _a2.outcome, t != null) break;
  }
  return Yt(t, e == null ? void 0 : e.seats.find((i) => i.player === a));
}
function ep(e) {
  if (!e || typeof e != "object") return "\u547D\u4EE4\u7ED3\u679C\u65E0\u6CD5\u8BFB\u53D6";
  let n = e;
  if (n.kind === "plan" && typeof n.error == "string") return `\u81EA\u52A8\u64CD\u4F5C\u5931\u8D25\uFF1A${n.error === "stale" ? "\u5C40\u9762\u5DF2\u53D8\u5316\uFF0C\u8BF7\u91CD\u8BD5" : n.error === "cancelled" ? "\u5DF2\u53D6\u6D88" : n.error}`;
  if (n.kind === "plan" || n.kind === "answer") try {
    const i = typeof n.outcome == "string" ? JSON.parse(n.outcome) : null;
    if (!i || typeof i != "object" || Array.isArray(i)) return "\u81EA\u52A8\u64CD\u4F5C\u7ED3\u679C\u65E0\u6CD5\u8BFB\u53D6";
    n = i;
  } catch {
    return "\u81EA\u52A8\u64CD\u4F5C\u7ED3\u679C\u65E0\u6CD5\u8BFB\u53D6";
  }
  if (n.done === true || n.outcome === "accepted" || n.outcome === "pending") return null;
  if (n.outcome === "suspended") return "\u64CD\u4F5C\u5DF2\u6682\u505C\uFF0C\u6682\u65E0\u6CD5\u7EE7\u7EED\u7ED3\u7B97";
  if (n.outcome !== "rejected") return "\u547D\u4EE4\u7ED3\u679C\u65E0\u6CD5\u8BFB\u53D6";
  const a = { ChoicePending: "\u8BF7\u5148\u5B8C\u6210\u5F53\u524D\u9009\u62E9", NoSuchChoice: "\u5F53\u524D\u9009\u62E9\u5DF2\u5931\u6548", InvalidChoice: "\u9009\u62E9\u65E0\u6548", NotYourTurn: "\u8FD8\u6CA1\u6709\u8F6E\u5230\u8BE5\u73A9\u5BB6", WrongPhase: "\u5F53\u524D\u9636\u6BB5\u4E0D\u80FD\u6267\u884C\u6B64\u64CD\u4F5C", CardNotInHand: "\u8FD9\u5F20\u724C\u4E0D\u5728\u624B\u724C\u4E2D", IllegalTargets: "\u76EE\u6807\u4E0D\u5408\u6CD5", WrongPlayer: "\u5F53\u524D\u73A9\u5BB6\u65E0\u6743\u5904\u7406\u6B64\u9009\u62E9", StaleRevision: "\u5C40\u9762\u5DF2\u53D8\u5316\uFF0C\u8BF7\u91CD\u8BD5", MatchOver: "\u5BF9\u5C40\u5DF2\u7ED3\u675F" }, t = typeof n.reason == "string" && n.reason ? n.reason : "\u539F\u56E0\u672A\u77E5";
  return `\u64CD\u4F5C\u88AB\u62D2\u7EDD\uFF1A${a[t] ?? t}`;
}
let np = 1;
function ap() {
  var _a2;
  const e = typeof window < "u" && ((_a2 = window.matchMedia) == null ? void 0 : _a2.call(window, "(prefers-reduced-motion: reduce)").matches);
  return ta({ connected: false, busy: false, error: null, revision: 0, hash: "", finished: false, view: null, choice: null, legalActions: [], lastEvents: [], log: [], effects: [], speed: 1, skipAnims: false, reduceMotion: !!e });
}
let Ke = 1;
function ct(e, n) {
  fi(), tp(e, n), _i();
}
function tp(e, n) {
  e.revision = n.revision, e.hash = n.hash_hex, e.finished = n.finished, e.view = n.view, e.choice = n.choice, e.legalActions = n.legal_actions ?? [], e.lastEvents = n.events ?? [], e.error = null;
  const a = ip(n.events ?? []);
  e.effects = a;
  const t = op(n.events ?? []);
  for (const i of t) me(e, i);
}
function ip(e) {
  const n = [];
  for (const a of e) {
    if (!a || typeof a != "object") continue;
    const t = a, i = Object.keys(t)[0];
    if (!i) continue;
    const s = t[i];
    switch (i) {
      case "DamageDealt":
        n.push({ id: Ke++, type: "damage", player: s == null ? void 0 : s.target, amount: s == null ? void 0 : s.amount, text: `-${s == null ? void 0 : s.amount}` });
        break;
      case "HpRecovered":
        n.push({ id: Ke++, type: "heal", player: s == null ? void 0 : s.player, amount: s == null ? void 0 : s.amount, text: `+${s == null ? void 0 : s.amount}` });
        break;
      case "CardsDrawn":
        n.push({ id: Ke++, type: "draw", player: s == null ? void 0 : s.player, text: `\u6478${Array.isArray(s == null ? void 0 : s.cards) ? s.cards.length : 1}` });
        break;
      case "CardUsed":
        n.push({ id: Ke++, type: "play", player: s == null ? void 0 : s.player, text: "\u51FA\u724C" });
        break;
      case "EnteredDying":
        n.push({ id: Ke++, type: "dying", player: s == null ? void 0 : s.player, text: "\u6FD2\u6B7B" });
        break;
      case "PlayerKilled":
        n.push({ id: Ke++, type: "death", player: s == null ? void 0 : s.player, text: "\u9635\u4EA1" });
        break;
    }
  }
  return n;
}
function sp(e) {
  e.effects = [];
}
function me(e, n) {
  e.log.unshift({ id: np++, text: n, at: Date.now() }), e.log.length > 80 && (e.log.length = 80);
}
function op(e) {
  const n = [];
  for (const a of e) {
    if (!a || typeof a != "object") continue;
    const t = a, i = Object.keys(t)[0];
    if (!i) continue;
    const s = t[i];
    switch (i) {
      case "GameStarted":
        n.push(`\u5BF9\u5C40\u5F00\u59CB \xB7 ${(s == null ? void 0 : s.player_count) ?? "?"} \u4EBA`);
        break;
      case "CardsDrawn":
        n.push(`\u73A9\u5BB6 ${s == null ? void 0 : s.player} \u6478\u724C \xD7${Array.isArray(s == null ? void 0 : s.cards) ? s.cards.length : "?"}`);
        break;
      case "CardUsed":
        n.push(`\u73A9\u5BB6 ${s == null ? void 0 : s.player} \u4F7F\u7528\u5361\u724C ${s == null ? void 0 : s.card}`);
        break;
      case "DamageDealt":
        n.push(`\u4F24\u5BB3 ${s == null ? void 0 : s.source} \u2192 ${s == null ? void 0 : s.target} (${s == null ? void 0 : s.amount})`);
        break;
      case "HpRecovered":
        n.push(`\u73A9\u5BB6 ${s == null ? void 0 : s.player} \u56DE\u590D ${s == null ? void 0 : s.amount}`);
        break;
      case "EnteredDying":
        n.push(`\u73A9\u5BB6 ${s == null ? void 0 : s.player} \u8FDB\u5165\u6FD2\u6B7B`);
        break;
      case "PlayerKilled":
        n.push(`\u73A9\u5BB6 ${s == null ? void 0 : s.player} \u9635\u4EA1`);
        break;
      case "TurnStateChanged":
        n.push(`\u56DE\u5408 \xB7 \u73A9\u5BB6 ${s == null ? void 0 : s.current} \xB7 ${s == null ? void 0 : s.phase}`);
        break;
      case "MatchEnded":
        n.push(`\u7EC8\u5C40 \xB7 ${Xm(s == null ? void 0 : s.outcome)}`);
        break;
      case "ChoiceRequested":
        n.push(`\u7B49\u5F85\u9009\u62E9 \xB7 \u73A9\u5BB6 ${s == null ? void 0 : s.observer}`);
        break;
      case "PublicEvent":
        n.push(String((s == null ? void 0 : s.label) ?? `\u516C\u5171\u4E8B\u4EF6 ${(s == null ? void 0 : s.code) ?? "?"}`));
        break;
      default:
        n.push(i);
    }
  }
  return n;
}
function rp(e, n) {
  if (e.skipAnims || e.reduceMotion || e.speed <= 0) return Promise.resolve();
  const a = Math.max(0, n / e.speed);
  return new Promise((t) => setTimeout(t, a));
}
function Qe(e) {
  return typeof e == "object" && e !== null && !Array.isArray(e);
}
function Sn(e) {
  return typeof e == "string" ? e : e instanceof Error ? e.message : String(e);
}
function ln(e) {
  try {
    const n = JSON.stringify(e);
    return (n === void 0 ? String(e) : n).slice(0, 160);
  } catch {
    return String(e);
  }
}
function lp(e) {
  return Array.isArray(e) && e.every((n) => typeof n == "string");
}
function up(e) {
  if (!Qe(e)) throw new Error(`\u9009\u5C06\u5019\u9009\u7ED3\u6784\u4E0D\u7B26\uFF08\u9700 {count, candidates}\uFF09\uFF1A${ln(e)}`);
  const n = e.count, a = e.candidates;
  if (typeof n != "number" || !Number.isInteger(n) || n < 0 || !Array.isArray(a)) throw new Error(`\u9009\u5C06\u5019\u9009\u7ED3\u6784\u4E0D\u7B26\uFF08\u9700 {count:\u975E\u8D1F\u6574\u6570, candidates:\u6570\u7EC4}\uFF09\uFF1A${ln(e)}`);
  const t = [], i = /* @__PURE__ */ new Set();
  for (const [s, l] of a.entries()) {
    if (!Qe(l)) throw new Error(`\u9009\u5C06\u5019\u9009\u7B2C ${s} \u9879\u7ED3\u6784\u4E0D\u7B26\uFF08\u9700\u5BF9\u8C61\uFF09\uFF1A${ln(l)}`);
    const r = l.def, u = l.name, c = l.skills;
    if (typeof r != "number" || !Number.isInteger(r) || r < 0 || typeof u != "string" || u.trim().length === 0 || !lp(c)) throw new Error(`\u9009\u5C06\u5019\u9009\u7B2C ${s} \u9879\u7ED3\u6784\u4E0D\u7B26\uFF08\u9700 {def:\u975E\u8D1F\u6574\u6570, name:\u975E\u7A7A\u5B57\u7B26\u4E32, skills:\u5B57\u7B26\u4E32\u6570\u7EC4}\uFF09\uFF1A${ln(l)}`);
    if (i.has(r)) throw new Error(`\u9009\u5C06\u5019\u9009\u7B2C ${s} \u9879 def=${r} \u4E0E\u524D\u9762\u91CD\u590D`);
    i.add(r), t.push({ def: r, name: u, skills: [...c], skillLabel: c.length > 0 ? c.join(" \xB7 ") : "\u65E0\u6280\u80FD" });
  }
  if (t.length !== n) throw new Error(`\u9009\u5C06\u5019\u9009 count=${n} \u4E0E\u5019\u9009\u6570 ${t.length} \u4E0D\u4E00\u81F4`);
  return t;
}
function ma() {
  return { characterDef: null, skillIds: [], skillNames: [] };
}
function cp(e, n = ma()) {
  return { seat: e, loadStatus: "idle", candidates: [], error: null, selectionStatus: "idle", pendingName: null, assignment: null, lastStateHash: null, projection: Xt(n) };
}
function Xt(e) {
  return { characterDef: e.characterDef, skillIds: [...e.skillIds], skillNames: [...e.skillNames] };
}
function Zt(e, n) {
  const t = (Array.isArray(e == null ? void 0 : e.seats) ? e.seats : []).find((r) => r && r.player === n);
  if (!t) return ma();
  const i = [], s = [], l = Array.isArray(t.skills) ? t.skills : [];
  for (const r of l) {
    const u = r == null ? void 0 : r.id;
    typeof u == "number" && Number.isInteger(u) && !i.includes(u) && i.push(u);
    const c = r == null ? void 0 : r.name;
    typeof c == "string" && c.length > 0 && !s.includes(c) && s.push(c);
  }
  return { characterDef: typeof t.character_def == "number" ? t.character_def : null, skillIds: i, skillNames: s };
}
function ei(e, n) {
  return { ...e, projection: Xt(n) };
}
function dt(e, n) {
  return { ...e, seat: n, projection: ma(), assignment: null, lastStateHash: null, selectionStatus: "idle", pendingName: null, error: null };
}
function dp(e) {
  return { ...e, loadStatus: "loading", error: null };
}
function mp(e, n) {
  try {
    const a = up(n);
    return { ...e, loadStatus: a.length > 0 ? "ready" : "empty", candidates: a, error: null };
  } catch (a) {
    return { ...e, loadStatus: "error", candidates: [], error: `\u9009\u5C06\u5019\u9009\u4E0D\u53EF\u7528\uFF1A${Sn(a)}` };
  }
}
function pp(e, n) {
  return { ...e, loadStatus: "error", candidates: [], error: `\u9009\u5C06\u5019\u9009\u52A0\u8F7D\u5931\u8D25\uFF1A${Sn(n)}` };
}
function hp(e, n) {
  if (!Qe(e)) return `\u56DE\u5305\u4E0D\u662F\u5BF9\u8C61\uFF1A${ln(e)}`;
  const a = e.assignment, t = e.envelope;
  return Qe(a) ? a.seat !== n.seat ? `\u5EA7\u4F4D\u56DE\u663E ${String(a.seat)} \u2260 \u8BF7\u6C42 ${n.seat}` : a.name !== n.name ? `\u6B66\u5C06\u56DE\u663E ${String(a.name)} \u2260 \u8BF7\u6C42 ${n.name}` : typeof a.state_hash != "string" || a.state_hash.length === 0 ? "\u7F3A assignment.state_hash" : !Qe(t) || !Qe(t.view) ? "\u7F3A\u5237\u65B0\u4FE1\u5C01 view" : null : "\u7F3A assignment";
}
async function gp(e, n, a, t) {
  var _a2;
  if (typeof n != "string" || n.trim().length === 0) return { ...e, selectionStatus: "rejected", pendingName: null, error: "\u9009\u5C06\u5931\u8D25\uFF1A\u5019\u9009\u540D\u4E3A\u7A7A" };
  const i = { ...e, selectionStatus: "pending", pendingName: n, error: null };
  t == null ? void 0 : t(i);
  let s;
  try {
    s = await a.chooseGeneral(e.seat, n);
  } catch (m) {
    return { ...e, selectionStatus: "rejected", pendingName: null, error: `\u9009\u5C06\u5931\u8D25\uFF08seat=${e.seat}, name=${n}\uFF09\uFF1A${Sn(m)}` };
  }
  const l = hp(s, { seat: e.seat, name: n });
  if (l) return { ...e, selectionStatus: "rejected", pendingName: null, error: `\u9009\u5C06\u8FD4\u56DE\u65E0\u6CD5\u786E\u8BA4\u88C5\u914D\uFF1A${l}` };
  const r = s.assignment, u = s.envelope, c = ei({ ...i, assignment: r, lastStateHash: r.state_hash }, Zt(u.view, e.seat));
  let _ = null;
  try {
    (_a2 = a.applyEnvelope) == null ? void 0 : _a2.call(a, u);
  } catch (m) {
    _ = `\u9009\u5C06\u5DF2\u88C5\u914D\uFF0C\u4F46\u89C6\u56FE\u5237\u65B0\u5931\u8D25\uFF1A${Sn(m)}`;
  }
  return { ...c, selectionStatus: "accepted", pendingName: null, error: _ };
}
const fp = ["data-slot", "aria-label"], _p = ["src", "alt"], vp = ["aria-label"], yp = { class: "name" }, kp = { key: 2, class: "muted" }, wp = ["value", "aria-label"], bp = ["value"], xp = se({ __name: "GeneralPortrait", props: { general: {}, legacyDef: {}, alive: { type: Boolean }, slot: {} }, setup(e) {
  const n = e, a = L(() => {
    var _a2;
    return ((_a2 = n.general) == null ? void 0 : _a2.def) ?? n.legacyDef ?? 0;
  }), t = L(() => n.general !== null || n.legacyDef !== void 0), i = L(() => {
    var _a2;
    return t.value ? ((_a2 = n.general) == null ? void 0 : _a2.name) || Ao(a.value) || `#${a.value}` : "\u6697\u5C06\uFF0F\u672A\u914D\u7F6E";
  }), s = { wei: "\u9B4F", shu: "\u8700", wu: "\u5434", qun: "\u7FA4" }, l = L(() => {
    var _a2;
    const y = ((_a2 = n.general) == null ? void 0 : _a2.faction) ?? "";
    return Object.hasOwn(s, y) ? s[y] : "\u672A\u77E5\u52BF\u529B";
  }), r = L(() => Co(a.value)), u = L(() => Jn(a.value)), c = L(() => Po(a.value)), _ = L(() => qo(a.value)), m = L(() => Cn(a.value)), p = L(() => Array.from(/* @__PURE__ */ new Set([...n.alive ? [] : [Io(a.value, "dead")], c.value, _.value, m.value])).filter((y) => !!y)), d = I([]), v = L(() => p.value.find((y) => !d.value.includes(y)) ?? "");
  Be([a, t], ([y, k]) => {
    k && zo(y).catch(() => {
    });
  }, { immediate: true }), Be(p, () => {
    d.value = [];
  });
  function P() {
    v.value && (d.value = [...d.value, v.value]);
  }
  function V(y) {
    const k = y.target.value;
    No(a.value, k === "" ? null : Number(k));
  }
  return (y, k) => (h(), g("div", { class: "general-slot", "data-slot": e.slot, "aria-label": `${e.slot === 0 ? "\u4E3B\u5C06" : "\u526F\u5C06"} ${i.value}` }, [t.value && v.value ? (h(), g("img", { key: 0, class: "portrait", src: v.value, alt: `${i.value}\u6B66\u5C06\u5934\u50CF`, onError: P }, null, 40, _p)) : (h(), g("div", { key: 1, class: "portrait no-image", role: "img", "aria-label": `${i.value}\u6682\u65E0\u5934\u50CF` }, f(t.value ? "\u65E0\u56FE" : "\u672A\u77E5"), 9, vp)), o("span", yp, f(i.value), 1), e.general ? (h(), g("span", kp, f(l.value) + " \xB7 " + f(e.general.revealed ? "\u660E\u7F6E" : "\u6697\u7F6E"), 1)) : S("v-if", true), t.value && r.value.length ? (h(), g("select", { key: 3, class: "skin-select", value: u.value ?? "", "aria-label": `${i.value}\u76AE\u80A4`, onClick: k[0] || (k[0] = Vn(() => {
  }, ["stop"])), onKeydown: k[1] || (k[1] = Vn(() => {
  }, ["stop"])), onChange: V }, [k[2] || (k[2] = o("option", { value: "" }, "\u9ED8\u8BA4\u76AE\u80A4", -1)), (h(true), g(Q, null, Y(r.value, (w) => (h(), g("option", { key: w.id, value: w.id }, f(z(Ro)(w)) + f(w.presentation.kind === "spine" ? "\uFF08\u9759\u6001\u56DE\u9000\uFF09" : ""), 9, bp))), 128))], 40, wp)) : S("v-if", true)], 8, fp));
} }), $p = ge(xp, [["__scopeId", "data-v-689e5d5b"]]), jp = ["aria-label", "tabindex"], Sp = { key: 0, class: "tag" }, Ep = { key: 1, class: "tag", "aria-label": "\u6B66\u5C06\u724C\u80CC\u9762" }, zp = { class: "generals", "aria-label": "\u53CC\u5C06" }, Ap = { class: "hp-row" }, Cp = ["aria-label"], Rp = { class: "hp", "aria-label": "\u4F53\u529B" }, Np = ["aria-valuenow", "aria-valuemax", "aria-label"], Pp = { class: "meta muted" }, qp = { key: 0, class: "equip-zone", "aria-label": "\u88C5\u5907\u533A" }, Ip = ["title"], Op = { class: "equip-suit" }, Tp = { class: "equip-id" }, Lp = { key: 1, class: "dead-badge" }, Dp = { key: 2, class: "dying-badge" }, Mp = { key: 3, class: "target-hint" }, Up = se({ __name: "SeatCard", props: { seat: {}, self: { type: Boolean }, identityLabel: {}, isTargetMode: { type: Boolean }, isSelectableTarget: { type: Boolean }, isDropTarget: { type: Boolean }, equipCards: {} }, emits: ["select"], setup(e, { emit: n }) {
  const a = e, t = n, i = L(() => a.seat.generals ?? [null, null]), s = L(() => !a.seat.generals || i.value.every((c) => c === null) && a.seat.character_def !== 0 ? a.seat.character_def : void 0), l = L(() => i.value.map((c, _) => (c == null ? void 0 : c.name) || (_ === 0 && s.value !== void 0 ? `#${s.value}` : "\u6697\u5C06\uFF0F\u672A\u914D\u7F6E")).join(" / "));
  function r(c, _, m) {
    return m ? c <= 0 ? "\u26A0" : "\u2713" : "\u2717";
  }
  function u(c, _, m) {
    return m ? c <= 0 ? "var(--color-warning)" : "var(--color-positive)" : "var(--color-negative)";
  }
  return (c, _) => (h(), g("article", { class: oe(["seat has-portrait", { self: e.self, dead: !e.seat.alive, dying: e.seat.alive && e.seat.hp <= 0, "target-mode": e.isTargetMode, "target-selectable": e.isTargetMode && e.isSelectableTarget, "drop-target": e.isDropTarget, "current-turn": false }]), role: "listitem", "aria-label": `\u5EA7\u4F4D ${e.seat.seat} \u73A9\u5BB6 ${e.seat.player} \u6B66\u5C06 ${l.value}`, tabindex: e.isTargetMode && e.isSelectableTarget ? 0 : -1, onClick: _[0] || (_[0] = (m) => e.isTargetMode && e.isSelectableTarget && t("select", e.seat.player)), onKeydown: _[1] || (_[1] = Ji((m) => e.isTargetMode && e.isSelectableTarget && t("select", e.seat.player), ["enter"])) }, [o("header", null, [e.self ? (h(), g("span", Sp, "\u4F60")) : S("v-if", true), e.seat.turned_over ? (h(), g("span", Ep, "\u7FFB\u9762")) : S("v-if", true)]), o("div", zp, [(h(true), g(Q, null, Y(i.value, (m, p) => (h(), vt($p, { key: p, general: m, slot: p, "legacy-def": p === 0 ? s.value : void 0, alive: e.seat.alive }, null, 8, ["general", "slot", "legacy-def", "alive"]))), 128))]), o("div", Ap, [o("span", { class: "state-icon", style: mn({ color: u(e.seat.hp, e.seat.max_hp, e.seat.alive) }), "aria-label": e.seat.alive ? e.seat.hp <= 0 ? "\u6FD2\u6B7B" : "\u5B58\u6D3B" : "\u9635\u4EA1" }, f(r(e.seat.hp, e.seat.max_hp, e.seat.alive)), 13, Cp), o("span", Rp, f(e.seat.hp) + " / " + f(e.seat.max_hp), 1)]), S(" HP bar "), o("div", { class: "hp-bar", role: "progressbar", "aria-valuenow": Math.max(e.seat.hp, 0), "aria-valuemax": e.seat.max_hp, "aria-label": `\u4F53\u529B ${e.seat.hp}/${e.seat.max_hp}` }, [o("div", { class: "hp-bar-fill", style: mn({ width: Math.max(Math.min(e.seat.hp / e.seat.max_hp * 100, 100), 0) + "%", background: e.seat.alive ? e.seat.hp <= 0 ? "var(--color-warning)" : "var(--color-positive)" : "var(--color-negative)" }) }, null, 4)], 8, Np), o("div", Pp, [o("span", null, "\u624B\u724C " + f(e.seat.hand_count), 1), o("span", null, f(e.identityLabel), 1)]), e.equipCards.length > 0 ? (h(), g("div", qp, [(h(true), g(Q, null, Y(e.equipCards, (m) => (h(), g("div", { key: m.id, class: "equip-card", title: `\u5361\u724C #${m.id}${m.def != null ? " (def:" + m.def + ")" : ""}` }, [o("span", Op, f(m.suit === "Spade" ? "\u2660" : m.suit === "Heart" ? "\u2665" : m.suit === "Club" ? "\u2663" : m.suit === "Diamond" ? "\u2666" : "?"), 1), o("span", Tp, "#" + f(m.id), 1)], 8, Ip))), 128))])) : S("v-if", true), e.seat.alive ? e.seat.hp <= 0 ? (h(), g("div", Dp, "\u26A0 \u6FD2\u6B7B")) : S("v-if", true) : (h(), g("div", Lp, "\u2717 \u9635\u4EA1")), e.isTargetMode && e.isSelectableTarget ? (h(), g("div", Mp, " \u25B8 \u70B9\u51FB\u9009\u62E9\u4E3A\u76EE\u6807 ")) : S("v-if", true)], 42, jp));
} }), Vp = ge(Up, [["__scopeId", "data-v-fdcad0a8"]]), Gp = { class: "hand", "aria-label": "\u624B\u724C" }, Bp = { class: "muted title" }, Fp = { class: "strip", role: "listbox", "aria-label": "\u624B\u724C\u5217\u8868" }, Hp = ["disabled", "aria-selected", "aria-label", "onClick", "onDragstart"], Wp = ["data-suit"], Kp = { class: "rank" }, Jp = { class: "id muted" }, Qp = { key: 0, class: "card-known" }, Yp = { key: 0, class: "muted empty" }, Xp = { key: 0, class: "hint" }, Zp = { key: 1, class: "hint drag-hint" }, eh = se({ __name: "HandStrip", props: { cards: {}, actionableIds: {} }, emits: ["select", "dragStart"], setup(e, { emit: n }) {
  const a = e, t = n, i = I(null), s = I(null);
  function l(m) {
    switch (m) {
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
  function r(m) {
    return m == null ? "?" : m === 1 ? "A" : m === 11 ? "J" : m === 12 ? "Q" : m === 13 ? "K" : String(m);
  }
  function u(m) {
    a.actionableIds.includes(m) && (i.value = m, t("select", m));
  }
  function c(m, p) {
    if (!a.actionableIds.includes(p)) {
      m.preventDefault();
      return;
    }
    s.value = p, m.dataTransfer && (m.dataTransfer.setData("text/plain", String(p)), m.dataTransfer.effectAllowed = "move"), t("dragStart", p);
  }
  function _() {
    s.value = null;
  }
  return (m, p) => (h(), g("section", Gp, [o("h2", Bp, " \u624B\u724C\uFF08" + f(e.cards.length) + " \u5F20" + f(i.value != null ? " \xB7 \u5DF2\u9009\u4E2D #" + i.value : " \xB7 \u70B9\u51FB\u6216\u62D6\u62FD") + "\uFF09 ", 1), o("div", Fp, [(h(true), g(Q, null, Y(e.cards, (d) => (h(), g("button", { key: d.id, type: "button", class: oe(["card", { selected: d.id === i.value, dragging: d.id === s.value }]), disabled: !e.actionableIds.includes(d.id), role: "option", "aria-selected": d.id === i.value, "aria-label": `\u5361\u724C ${d.id}${d.def != null ? " \u5DF2\u77E5" : " \u672A\u77E5"}`, draggable: "true", onClick: (v) => u(d.id), onDragstart: (v) => c(v, d.id), onDragend: _ }, [o("span", { class: "suit", "data-suit": d.suit }, f(l(d.suit)), 9, Wp), o("span", Kp, f(r(d.rank)), 1), o("span", Jp, "#" + f(d.id), 1), d.def != null ? (h(), g("span", Qp, "\u25B8")) : S("v-if", true)], 42, Hp))), 128)), e.cards.length ? S("v-if", true) : (h(), g("p", Yp, "\u65E0\u53EF\u89C1\u624B\u724C"))]), i.value != null ? (h(), g("p", Xp, " \u2713 \u5DF2\u9009\u4E2D #" + f(i.value) + " \u2014 \u5728\u68CB\u76D8\u4E0A\u9009\u62E9\u76EE\u6807\uFF0C\u6216\u70B9\u51FB\u300C\u7ED3\u675F\u9636\u6BB5\u300D\u76F4\u63A5\u6253\u51FA ", 1)) : S("v-if", true), s.value != null ? (h(), g("p", Zp, " \u{1F0A0} \u62D6\u62FD\u4E2D #" + f(s.value) + " \u2014 \u62D6\u5230\u76EE\u6807\u73A9\u5BB6\u91CA\u653E\u51FA\u724C ", 1)) : S("v-if", true)]));
} }), nh = ge(eh, [["__scopeId", "data-v-b2b16095"]]), ah = { class: "board", role: "region", "aria-label": "\u6218\u573A" }, th = { key: 0, class: "action-bar panel", role: "toolbar", "aria-label": "\u884C\u52A8\u64CD\u4F5C" }, ih = { class: "muted" }, sh = { key: 2 }, oh = { key: 3 }, rh = { class: "row" }, lh = ["disabled"], uh = ["disabled"], ch = { key: 1, class: "skill-bar panel", "aria-label": "\u53EF\u53D1\u52A8\u6280\u80FD" }, dh = ["onClick"], mh = { class: "seats", role: "list", "aria-label": "\u5EA7\u4F4D\u5217\u8868" }, ph = ["onDragover", "onDragleave", "onDrop"], hh = { key: 2, class: "judgment-zone", "aria-label": "\u5224\u5B9A\u533A" }, gh = { class: "jz-entries" }, fh = ["title"], _h = { class: "jz-suit" }, vh = { class: "jz-player" }, yh = { key: 3, class: "discard-pile", "aria-label": "\u5F03\u724C\u5806" }, kh = { class: "muted discard-title" }, wh = { class: "discard-strip" }, bh = ["title"], xh = { class: "d-suit" }, $h = { key: 0, class: "muted discard-more" }, jh = se({ __name: "BattleBoard", props: { view: {}, selfId: {}, legalActions: {} }, emits: ["playCard", "activateSkill"], setup(e, { emit: n }) {
  const a = e, t = n, i = I(null), s = I(null), l = I(false), r = I([]), u = I(null), c = I(null);
  function _() {
    var _a2;
    return ((_a2 = a.view) == null ? void 0 : _a2.seats) ?? [];
  }
  function m() {
    return a.view ? a.view.known_cards.filter((E) => E.owner === a.selfId && E.zone === "Hand" && E.def != null) : [];
  }
  function p() {
    var _a2;
    const E = new Set(a.legalActions.flatMap((N) => typeof N == "object" && "ActivateSkill" in N ? [N.ActivateSkill.skill] : []));
    return ((_a2 = _().find((N) => N.player === a.selfId)) == null ? void 0 : _a2.skills.filter((N) => N.enabled && (N.active || N.view_as) && E.has(N.id))) ?? [];
  }
  function d(E) {
    switch (E) {
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
  function v() {
    return l.value ? new Set(A().flat()) : /* @__PURE__ */ new Set();
  }
  function P() {
    return a.view ? a.view.known_cards.filter((E) => E.zone === "DiscardPile" && E.def != null) : [];
  }
  function V(E) {
    return a.view ? a.view.known_cards.filter((N) => N.owner === E && N.zone === "Judge" && N.def != null) : [];
  }
  const y = L(() => v()), k = L(() => s.value ? Array.from(new Set(D(s.value.id).flatMap((E) => E.card_targets))) : Array.from(new Set(w().map((E) => E.card))));
  function w(E) {
    return a.legalActions.flatMap((N) => typeof N != "object" || !("PlayCard" in N) ? [] : E != null && N.PlayCard.card !== E ? [] : [N.PlayCard]);
  }
  function D(E) {
    return a.legalActions.flatMap((N) => typeof N != "object" || !("ActivateSkill" in N) ? [] : N.ActivateSkill.skill === E ? [N.ActivateSkill] : []);
  }
  function A() {
    return s.value ? D(s.value.id).filter((E) => i.value == null ? E.card_targets.length === 0 : E.card_targets.length === 1 && E.card_targets[0] === i.value).map((E) => E.targets) : i.value == null ? [] : w(i.value).map((E) => E.targets);
  }
  function M(E, N) {
    return E.length === N.length && E.every((U, K) => U === N[K]);
  }
  const C = L(() => A().some((E) => M(E, r.value))), re = L(() => A().some((E) => E.length === 0));
  function ce(E) {
    return a.view ? a.view.known_cards.filter((N) => N.owner === E && N.zone === "Equip" && N.def != null) : [];
  }
  function fe(E) {
    if (k.value.includes(E)) {
      if (i.value === E) {
        i.value = null, l.value = false, r.value = [];
        return;
      }
      i.value = E, s.value || (s.value = null), l.value = true, r.value = [];
    }
  }
  function He(E) {
    var _a2;
    if (((_a2 = s.value) == null ? void 0 : _a2.id) === E.id) {
      Oe();
      return;
    }
    s.value = E, i.value = null, l.value = true, r.value = [];
  }
  function $e(E) {
    k.value.includes(E) && (u.value = E, i.value = E, l.value = true);
  }
  function ve(E) {
    if (!(!l.value || !y.value.has(E))) if (r.value.includes(E)) r.value = r.value.filter((N) => N !== E);
    else {
      const N = Math.max(0, ...A().map((U) => U.length));
      r.value = [...r.value, E].slice(0, N);
    }
  }
  function Pe(E, N) {
    u.value != null && y.value.has(N) && (E.preventDefault(), E.dataTransfer.dropEffect = "move", c.value = N);
  }
  function qe(E) {
    c.value === E && (c.value = null);
  }
  function We(E, N) {
    var _a2;
    E.preventDefault();
    const U = u.value;
    if (c.value = null, u.value = null, U != null && y.value.has(N)) {
      if ((_a2 = s.value) == null ? void 0 : _a2.view_as) {
        const K = D(s.value.id).find((de) => M(de.targets, [N]) && de.card_targets.length === 1 && de.card_targets[0] === U);
        if (!K) return;
        t("activateSkill", K.skill, K.targets, K.card_targets);
      } else {
        const K = w(U).find((de) => M(de.targets, [N]));
        if (!K) return;
        t("playCard", K.card, K.targets);
      }
      i.value = null, s.value = null, l.value = false, r.value = [];
    }
  }
  function ye() {
    if (C.value) {
      if (s.value) {
        const E = D(s.value.id).find((N) => M(N.targets, r.value) && (i.value == null ? N.card_targets.length === 0 : N.card_targets.length === 1 && N.card_targets[0] === i.value));
        if (!E) return;
        t("activateSkill", E.skill, E.targets, E.card_targets);
      } else {
        if (i.value == null) return;
        const E = w(i.value).find((N) => M(N.targets, r.value));
        if (!E) return;
        t("playCard", E.card, E.targets);
      }
      i.value = null, s.value = null, l.value = false, r.value = [];
    }
  }
  function Ie() {
    r.value = [], ye();
  }
  function Oe() {
    i.value = null, s.value = null, l.value = false, r.value = [];
  }
  return (E, N) => {
    var _a2;
    return h(), g("div", ah, [S(" Action bar when a card is selected "), i.value != null || s.value ? (h(), g("div", th, [o("span", ih, [N[0] || (N[0] = W(" \u25B8 ", -1)), s.value ? (h(), g(Q, { key: 0 }, [W("\u6280\u80FD #" + f(s.value.def), 1)], 64)) : (h(), g(Q, { key: 1 }, [W("\u624B\u724C #" + f(i.value), 1)], 64)), ((_a2 = s.value) == null ? void 0 : _a2.view_as) && i.value != null ? (h(), g("span", sh, " \xB7 \u7D20\u6750 #" + f(i.value), 1)) : S("v-if", true), l.value && r.value.length > 0 ? (h(), g("span", oh, " \u2192 \u76EE\u6807 P" + f(r.value[0]), 1)) : S("v-if", true)]), o("div", rh, [o("button", { class: "primary", type: "button", onClick: ye, disabled: !C.value, "aria-label": "\u786E\u8BA4\u884C\u52A8" }, f(s.value ? "\u53D1\u52A8\u6280\u80FD" : "\u786E\u8BA4\u51FA\u724C"), 9, lh), o("button", { type: "button", disabled: !re.value, onClick: Ie, "aria-label": "\u65E0\u76EE\u6807\u76F4\u63A5\u6253\u51FA" }, f(s.value ? "\u65E0\u76EE\u6807\u53D1\u52A8" : "\u76F4\u63A5\u6253\u51FA"), 9, uh), o("button", { type: "button", onClick: Oe, "aria-label": "\u53D6\u6D88\u9009\u62E9" }, " \u53D6\u6D88 ")])])) : S("v-if", true), p().length ? (h(), g("section", ch, [N[1] || (N[1] = o("span", { class: "muted" }, "\u6280\u80FD", -1)), (h(true), g(Q, null, Y(p(), (U) => {
      var _a3;
      return h(), g("button", { key: U.id, type: "button", class: oe({ primary: ((_a3 = s.value) == null ? void 0 : _a3.id) === U.id }), onClick: (K) => He(U) }, " #" + f(U.def) + f(U.view_as ? " \xB7 \u89C6\u4E3A" : ""), 11, dh);
    }), 128))])) : S("v-if", true), o("div", mh, [(h(true), g(Q, null, Y(_(), (U) => (h(), g("div", { key: U.player, class: "seat-wrapper", onDragover: (K) => Pe(K, U.player), onDragleave: (K) => qe(U.player), onDrop: (K) => We(K, U.player) }, [pe(Vp, { seat: U, self: U.player === e.selfId, "identity-label": d(U.identity), "is-target-mode": l.value, "is-selectable-target": y.value.has(U.player), "is-drop-target": c.value === U.player, "equip-cards": ce(U.player), onSelect: ve }, null, 8, ["seat", "self", "identity-label", "is-target-mode", "is-selectable-target", "is-drop-target", "equip-cards"])], 40, ph))), 128))]), pe(nh, { cards: m(), "actionable-ids": k.value, onSelect: fe, onDragStart: $e }, null, 8, ["cards", "actionable-ids"]), S(" Judgment zone (delayed tricks: Le/Bingliang/Shandian) "), _().some((U) => V(U.player).length > 0) ? (h(), g("section", hh, [N[2] || (N[2] = o("h3", { class: "muted jz-title" }, "\u2696 \u5224\u5B9A\u533A", -1)), o("div", gh, [(h(true), g(Q, null, Y(_(), (U) => (h(), g("div", { key: "jz-" + U.player, class: "jz-entry" }, [(h(true), g(Q, null, Y(V(U.player), (K) => (h(), g("span", { key: K.id, class: "jz-card", title: `P${U.player} #${K.id}` }, [o("span", _h, f(K.suit === "Spade" ? "\u2660" : K.suit === "Heart" ? "\u2665" : K.suit === "Club" ? "\u2663" : K.suit === "Diamond" ? "\u2666" : "?"), 1), o("span", vh, "P" + f(U.player), 1)], 8, fh))), 128))]))), 128))])])) : S("v-if", true), S(" Discard pile "), P().length > 0 ? (h(), g("section", yh, [o("h3", kh, "\u5F03\u724C\u5806\uFF08" + f(P().length) + "\uFF09", 1), o("div", wh, [(h(true), g(Q, null, Y(P().slice(-10), (U) => (h(), g("div", { key: U.id, class: "discard-card", title: `#${U.id} def:${U.def}` }, [o("span", xh, f(U.suit === "Spade" ? "\u2660" : U.suit === "Heart" ? "\u2665" : U.suit === "Club" ? "\u2663" : U.suit === "Diamond" ? "\u2666" : "?"), 1)], 8, bh))), 128)), P().length > 10 ? (h(), g("span", $h, "+" + f(P().length - 10), 1)) : S("v-if", true)])])) : S("v-if", true)]);
  };
} }), Sh = ge(jh, [["__scopeId", "data-v-9182a2db"]]), Eh = { class: "panel general-select", "aria-label": "\u9009\u5C06\u9762\u677F" }, zh = { class: "gs-head" }, Ah = { class: "muted tiny" }, Ch = ["disabled"], Rh = { key: 0, class: "muted", role: "status" }, Nh = { key: 1, class: "muted empty", role: "status" }, Ph = { key: 2, class: "error", role: "alert" }, qh = { key: 3, class: "success", role: "status" }, Ih = { class: "muted" }, Oh = { class: "muted" }, Th = { key: 4, class: "gs-grid" }, Lh = ["disabled", "aria-pressed", "aria-label", "onClick"], Dh = { class: "candidate-name" }, Mh = { class: "candidate-skills" }, Uh = { class: "candidate-def muted" }, Vh = se({ __name: "GeneralSelectPanel", props: { state: {}, busy: { type: Boolean } }, emits: ["choose", "reload"], setup(e, { emit: n }) {
  const a = e, t = n, i = L(() => a.busy || a.state.selectionStatus === "pending"), s = L(() => {
    var _a2;
    return ((_a2 = a.state.assignment) == null ? void 0 : _a2.name) ?? null;
  }), l = L(() => a.state.projection.skillNames.length > 0 ? a.state.projection.skillNames.join(" \xB7 ") : "\u89C6\u56FE\u672A\u66B4\u9732\u6280\u80FD\u540D"), r = L(() => {
    switch (a.state.loadStatus) {
      case "loading":
        return "\u8BFB\u53D6\u5019\u9009\u2026";
      case "error":
        return "\u5019\u9009\u4E0D\u53EF\u7528";
      case "empty":
        return "\u5019\u9009\u4E3A\u7A7A";
      default:
        return `${a.state.candidates.length} \u9879\u5019\u9009`;
    }
  });
  return (u, c) => (h(), g("section", Eh, [o("div", zh, [o("div", null, [c[1] || (c[1] = o("h2", null, "\u9009\u5C06", -1)), o("p", Ah, " NC05 \xB7 \u76EE\u6807\u5EA7\u4F4D P" + f(e.state.seat) + " \xB7 " + f(r.value) + " \xB7 \u70B9\u51FB\u5019\u9009\u5373\u6309\u540D\u88C5\u914D \uFF08\u539F\u751F\u6B63\u5F0F\u5165\u53E3\uFF0C\u975E debug\uFF09 ", 1)]), o("button", { type: "button", disabled: i.value || e.state.loadStatus === "loading", "aria-label": "\u5237\u65B0\u9009\u5C06\u5019\u9009", onClick: c[0] || (c[0] = (_) => t("reload")) }, " \u5237\u65B0\u5019\u9009 ", 8, Ch)]), e.state.loadStatus === "loading" && e.state.candidates.length === 0 ? (h(), g("p", Rh, " \u6B63\u5728\u8BFB\u53D6\u539F\u751F\u76EE\u5F55\u5019\u9009\u2026 ")) : e.state.loadStatus === "empty" ? (h(), g("p", Nh, " \u5019\u9009\u4E3A\u7A7A\uFF1A\u5185\u6838\u76EE\u5F55\u6CA1\u6709\u8FD4\u56DE\u4EFB\u4F55\u6B66\u5C06\uFF08\u8FD9\u662F\u5408\u6CD5\u7684\u7A7A\u76EE\u5F55\uFF0C\u4E0D\u662F\u52A0\u8F7D\u5931\u8D25\uFF09\u3002 ")) : S("v-if", true), e.state.error ? (h(), g("p", Ph, f(e.state.error), 1)) : S("v-if", true), e.state.selectionStatus === "accepted" && e.state.assignment ? (h(), g("p", qh, [W(" \u2705 \u5DF2\u4E3A P" + f(e.state.assignment.seat) + " \u88C5\u914D ", 1), o("strong", null, f(e.state.assignment.name), 1), o("span", Ih, " \xB7 state_hash " + f(e.state.assignment.state_hash.slice(0, 16)) + "\u2026 ", 1), c[2] || (c[2] = o("br", null, null, -1)), o("span", Oh, "\u5237\u65B0\u540E\u89C6\u56FE\u6280\u80FD\uFF1A" + f(l.value), 1)])) : S("v-if", true), e.state.candidates.length > 0 ? (h(), g("ul", Th, [(h(true), g(Q, null, Y(e.state.candidates, (_) => (h(), g("li", { key: _.def }, [o("button", { type: "button", class: oe(["candidate", { selected: s.value === _.name }]), disabled: i.value, "aria-pressed": s.value === _.name, "aria-label": `\u9009\u62E9 ${_.name}\uFF08${_.skillLabel}\uFF09`, onClick: (m) => t("choose", _.name) }, [o("span", Dh, f(_.name), 1), o("span", Mh, f(_.skillLabel), 1), o("span", Uh, "#" + f(_.def), 1)], 10, Lh)]))), 128))])) : S("v-if", true)]));
} }), Gh = ge(Vh, [["__scopeId", "data-v-9fa84d6d"]]), Bh = { class: "battle stack" }, Fh = { key: 0, class: "panel hotkey-overlay", role: "dialog", "aria-label": "\u5FEB\u6377\u952E" }, Hh = { class: "outcome-content" }, Wh = { class: "outcome-icon" }, Kh = { class: "outcome-label" }, Jh = { class: "muted" }, Qh = { key: 2, class: "panel turn-bar", role: "status", "aria-live": "polite" }, Yh = { class: "turn-player" }, Xh = ["data-phase"], Zh = { class: "panel toolbar", "aria-label": "\u5BF9\u5C40\u63A7\u5236" }, eg = { class: "row" }, ng = ["disabled"], ag = { key: 1, class: "online-badge" }, tg = ["disabled"], ig = ["disabled"], sg = ["disabled"], og = ["disabled"], rg = ["disabled"], lg = ["disabled"], ug = { class: "row muted controls" }, cg = { class: "check" }, dg = { class: "status", role: "status", "aria-live": "polite" }, mg = { key: 0, class: "error", role: "alert" }, pg = { key: 4, class: "panel online-roster", "aria-label": "\u8054\u673A\u623F\u95F4\u5EA7\u4F4D" }, hg = { class: "row online-roster-head" }, gg = { key: 0 }, fg = { class: "row online-seats" }, _g = { key: 0 }, vg = { key: 5, class: "panel choice", "aria-label": "\u5F85\u9009\u62E9" }, yg = { key: 0, role: "timer" }, kg = { class: "muted" }, wg = { key: 1, class: "row" }, bg = ["disabled", "onClick"], xg = { key: 2, class: "muted" }, $g = { key: 6, class: "panel roster", "aria-label": "\u73A9\u5BB6\u6982\u51B5" }, jg = { class: "roster-grid" }, Sg = { class: "roster-player" }, Eg = { class: "roster-hand muted" }, zg = ["data-id"], Ag = { key: 1, class: "roster-id muted" }, Cg = { class: "panel battle-section" }, Rg = { key: 0, class: "effect-overlay", "aria-hidden": "true" }, Ng = { class: "effect-icon" }, Pg = { class: "effect-text" }, qg = { key: 0, class: "effect-player" }, Ig = { key: 7, class: "panel match-stats", "aria-label": "\u5BF9\u5C40\u7EDF\u8BA1" }, Og = { class: "stats-grid" }, Tg = { class: "stat" }, Lg = { class: "stat-value mono" }, Dg = { class: "stat" }, Mg = { class: "stat-value mono" }, Ug = { class: "stat" }, Vg = { class: "stat-value" }, Gg = { class: "stat" }, Bg = { class: "stat-value" }, Fg = { key: 8, class: "panel seat-status", "aria-label": "\u5EA7\u4F4D\u72B6\u6001\u4E00\u89C8" }, Hg = { class: "seat-status-grid" }, Wg = { class: "ss-header" }, Kg = { class: "ss-player" }, Jg = { key: 0, class: "ss-tag" }, Qg = { class: "ss-hp-bar" }, Yg = { class: "ss-hp-text" }, Xg = { class: "ss-icon" }, Zg = { class: "ss-meta muted" }, ef = ["data-id"], nf = { key: 1, class: "muted" }, af = { class: "panel log", "aria-label": "\u4E8B\u4EF6\u65E5\u5FD7" }, tf = { class: "log-header" }, sf = { class: "log-list" }, of = { class: "log-rev" }, rf = { class: "log-text" }, lf = { key: 0, class: "muted empty-log" }, uf = se({ __name: "BattleView", setup(e) {
  var _a2;
  const n = so(), a = Ot(), t = ap(), i = new oa();
  let s;
  const l = new Km(), r = I(1);
  try {
    ((_a2 = globalThis.localStorage) == null ? void 0 : _a2.getItem("noname-next-skip-anims")) === "1" && (t.skipAnims = true);
  } catch {
  }
  if (n.query.skipAnims === "1" && (t.skipAnims = true), n.query.speed !== void 0) {
    const x = Number(n.query.speed);
    Number.isFinite(x) && x > 0 && (t.speed = x);
  }
  const u = I(Number(n.query.players ?? 4) || 4), c = I(Number(n.query.seed ?? 42) || 42), _ = I(null), m = I(false), p = I("\u672A\u8FDE\u63A5"), d = I(null), v = I(0), P = setInterval(() => {
    v.value++;
  }, 250), V = L(() => {
    var _a3;
    return v.value, l.secondsUntilServerDeadline((_a3 = t.choice) == null ? void 0 : _a3.deadline_unix_ms);
  }), y = I(0), k = I(false), w = L(() => n.query.mode === "online"), D = L(() => w.value && r.value === 0), A = I(cp(r.value)), M = L(() => !w.value && t.connected && i.isNative());
  async function C(x) {
    t.busy = true, t.error = null;
    try {
      return await x();
    } catch ($) {
      t.error = $ instanceof Error ? $.message : String($), p.value = t.error;
      return;
    } finally {
      t.busy = false;
    }
  }
  function re(x) {
    globalThis.__nativeStartMarks ?? (globalThis.__nativeStartMarks = {}), globalThis.__nativeStartMarks[x] = Math.round(performance.now() * 100) / 100;
  }
  async function ce(x) {
    const $ = performance.now();
    ct(t, x);
    const G = performance.now();
    re("t5_apply_envelope"), await rp(t, 120);
    const j = performance.now();
    if (globalThis.__nativeEnvelopeTimings ?? (globalThis.__nativeEnvelopeTimings = []), globalThis.__nativeEnvelopeTimings.push({ apply_ms: Math.round((G - $) * 100) / 100, delay_ms: Math.round((j - G) * 100) / 100 }), w.value && (!t.connected || t.revision !== x.revision || t.hash !== x.hash_hex)) return;
    sp(t);
    const Le = w.value ? `\u623F\u95F4 #${y.value} \xB7 ` : "", Rn = (x.hash_hex || "").slice(0, 12);
    p.value = x.finished ? `${Le}\u7EC8\u5C40 \xB7 ${le.value.label} \xB7 rev ${x.revision}` : `${Le}rev ${x.revision}${Rn ? ` \xB7 ${Rn}\u2026` : ""}${i.isNative() ? " \xB7 native" : ""}`, He();
  }
  async function fe(x) {
    if (await ce(x), !w.value && i.isNative()) {
      const $ = ep(x.result);
      if ($) throw new Error($);
    }
  }
  function He() {
    A.value = ei(A.value, Zt(t.view, A.value.seat));
  }
  async function $e() {
    if (!(w.value || !i.isNative())) {
      A.value = dp(A.value);
      try {
        const x = await i.generalCandidates();
        A.value = mp(A.value, x);
      } catch (x) {
        A.value = pp(A.value, x);
      }
    }
  }
  async function ve(x) {
    if (w.value || !i.isNative() || t.busy || A.value.selectionStatus === "pending") return;
    const $ = A.value;
    await C(async () => {
      if (A.value = await gp($, x, { chooseGeneral: (G, j) => i.chooseGeneral(G, j), applyEnvelope: (G) => {
        ct(t, G), me(t, `\u9009\u5C06 P${$.seat} \u2192 ${x}`);
      } }, (G) => {
        A.value = G;
      }), A.value.selectionStatus === "accepted") {
        const G = A.value.lastStateHash ?? "";
        p.value = `\u9009\u5C06 P${$.seat} \xB7 ${x}${G ? ` \xB7 ${G.slice(0, 12)}\u2026` : ""}`;
      } else A.value.selectionStatus === "rejected" && (p.value = A.value.error ?? "\u9009\u5C06\u5931\u8D25");
    });
  }
  async function Pe() {
    await C(async () => {
      try {
        if (await ha(true), lt(true), w.value) {
          const j = E();
          Eo(j.wsUrl), await ye(j);
          return;
        }
        So(), re("t0_click"), await i.connect(), re("t1_worker_ready"), t.connected = true, p.value = i.isNative() ? "\u539F\u751F Worker \u5C31\u7EEA\uFF0C\u5F00\u5C40\u4E2D\u2026" : "Worker \u5C31\u7EEA\uFF0C\u5F00\u5C40\u4E2D\u2026", i.isNative() && (r.value = Math.max(0, Math.min(r.value, u.value - 1)), A.value = dt(A.value, r.value));
        const x = Um();
        if (!i.isNative() && x.contentLock.length) throw new Error("\u5DF2\u5B89\u88C5\u6269\u5C55\u7684\u6B63\u5F0F\u5F00\u5C40\u9700\u8981\u539F\u751F\u540E\u7AEF");
        const $ = await i.start(u.value, c.value, r.value, x.packsJson, n.query.mode === "identity" ? "identity" : void 0);
        s = x, re("t2_start_done");
        const G = (i.isNative() ? $e() : Promise.resolve()).then((j) => (re("t3_candidates_loaded"), j));
        await ce($), await aa(), re("t4_first_frame"), await G, me(t, `\u5F00\u5C40 ${u.value} \u4EBA seed=${c.value}${x.contentLock.length ? ` \xB7 \u5185\u5BB9\u9501 ${x.contentLock.length} \u5305` : ""}`);
      } catch (x) {
        throw t.connected = false, B(), x;
      }
    });
  }
  async function qe() {
    await C(async () => {
      if (w.value) throw new Error("\u8054\u673A\u6A21\u5F0F\u8BF7\u4F7F\u7528\u670D\u52A1\u7AEF\u5B58\u6863");
      if (!i.isNative()) throw new Error("\u5B58\u6863\u4EC5\u539F\u751F\u540E\u7AEF\u652F\u6301");
      if (!s) throw new Error("\u5C1A\u672A\u5EFA\u7ACB\u5BF9\u5C40\u5185\u5BB9\u9501");
      const x = await i.saveHex(), $ = `save-${Date.now()}`;
      Mm({ id: $, label: `rev${t.revision} \xB7 ${u.value}p seed=${c.value}`, hex: x, seed: c.value, players: u.value, ...s, savedAt: (/* @__PURE__ */ new Date()).toISOString(), revision: t.revision }), me(t, `\u5DF2\u5B58\u6863 ${$}\uFF08${x.length / 2} \u5B57\u8282\uFF09`), p.value = `\u5DF2\u5B58\u6863 \xB7 ${$}`;
    });
  }
  async function We() {
    await C(async () => {
      if (w.value) throw new Error("\u8054\u673A\u6A21\u5F0F\u4E0D\u652F\u6301\u672C\u5730\u8BFB\u6863");
      if (!i.isNative()) throw new Error("\u8BFB\u6863\u4EC5\u539F\u751F\u540E\u7AEF\u652F\u6301");
      const $ = Kt()[0];
      if (!$) throw new Error("\u65E0\u672C\u5730\u5B58\u6863");
      const G = Vm($), j = await i.restoreHex($.hex, G.packsJson);
      s = G, u.value = $.players, c.value = $.seed, A.value = dt(A.value, A.value.seat), await ce(j), await $e(), me(t, `\u8BFB\u6863 ${$.id} \xB7 rev ${$.revision}`);
    });
  }
  async function ye(x) {
    l.onEnvelope((G) => {
      ce(G);
    }), l.onClose(() => {
      p.value = "\u8FDE\u63A5\u5DF2\u65AD\u5F00\uFF08\u670D\u52A1\u7AEF\u4E0D\u53EF\u8FBE\uFF09", t.connected = false, me(t, "\u8FDE\u63A5\u5DF2\u65AD\u5F00\uFF1A\u670D\u52A1\u7AEF\u4E0D\u53EF\u8FBE\u6216\u5DF2\u505C\u6B62");
    }), l.onReconnect((G) => {
      G === "reconnecting" ? (p.value = "\u8FDE\u63A5\u5DF2\u65AD\u5F00\uFF0C\u6B63\u5728\u91CD\u8FDE\u2026", t.connected = false) : G === "reconnected" ? (p.value = "\u5DF2\u91CD\u8FDE\uFF08\u7B49\u5F85\u623F\u95F4\u72B6\u6001\uFF09", t.connected = false, l.rejoinRoom().then((j) => (r.value = j.playerId, y.value = j.roomId, Oe(x, j), t.connected = true, p.value = `\u623F\u95F4 #${j.roomId} \xB7 \u5DF2\u91CD\u8FDE`, ce(j.envelope))).catch((j) => {
        t.connected = false;
        const Le = j instanceof Error ? j.message : String(j);
        p.value = `\u91CD\u8FDE\u540E\u56DE\u623F\u5931\u8D25\uFF1A${Le}`, me(t, `\u91CD\u8FDE\u540E\u56DE\u623F\u5931\u8D25\uFF1A${Le}`);
      })) : (p.value = "\u91CD\u8FDE\u5931\u8D25\uFF08\u670D\u52A1\u7AEF\u4E0D\u53EF\u8FBE\uFF09", t.connected = false);
    }), l.onRoster((G) => {
      d.value = G, y.value = G.room_id;
      const j = G.started && !k.value;
      k.value = G.started, p.value = G.started ? `\u623F\u95F4 #${G.room_id} \xB7 \u5DF2\u5F00\u5C40` : `\u623F\u95F4 #${G.room_id} \xB7 \u7B49\u5F85\u5165\u5EA7 ${G.seats.length}/${u.value}`, j && l.requestSnapshot().catch(() => {
      });
    }), p.value = `\u8FDE\u63A5 ${x.wsUrl}\u2026`, await l.connect(x.wsUrl, x.displayName);
    let $;
    if (x.action === "create") $ = await l.createRoom(x.players, x.seed, x.password, x.choiceTimeoutSecs ?? 30);
    else if (x.action === "spectate") $ = await l.spectate(x.roomId);
    else {
      const G = localStorage.getItem(ut(x.wsUrl, x.roomId));
      $ = await l.joinRoom(x.roomId, x.password, G);
    }
    y.value = $.roomId, r.value = $.playerId, u.value = $.view.seats.length, Oe(x, $), t.connected = true, await ce($.envelope), me(t, $.role === "Spectator" ? `\u65C1\u89C2\u623F\u95F4 #${$.roomId}` : `\u52A0\u5165\u623F\u95F4 #${$.roomId} \xB7 \u5EA7\u4F4D ${$.seat + 1}`);
  }
  function Ie() {
    l.cancelPendingCommands(), p.value = "\u5DF2\u53D6\u6D88\u5F85\u786E\u8BA4\u547D\u4EE4\uFF08\u670D\u52A1\u7AEF\u53EF\u80FD\u5DF2\u63D0\u4EA4\uFF0C\u4EE5\u6743\u5A01\u72B6\u6001\u4E3A\u51C6\uFF09", me(t, "\u5DF2\u53D6\u6D88\u5F85\u786E\u8BA4\u547D\u4EE4\uFF1B\u7ED3\u679C\u53EF\u80FD\u5DF2\u63D0\u4EA4\uFF0C\u7B49\u5F85/\u5237\u65B0\u540E\u4EE5\u670D\u52A1\u7AEF\u72B6\u6001\u4E3A\u51C6");
  }
  function Oe(x, $) {
    $.role === "Player" && $.reconnectToken && localStorage.setItem(ut(x.wsUrl, $.roomId), $.reconnectToken), sessionStorage.setItem("noname-next.online-intent", JSON.stringify({ ...x, action: $.role === "Spectator" ? "spectate" : "join", roomId: $.roomId }));
  }
  function E() {
    const x = sessionStorage.getItem("noname-next.online-intent");
    if (x) try {
      return JSON.parse(x);
    } catch {
      sessionStorage.removeItem("noname-next.online-intent");
    }
    const $ = window.location.protocol === "https:";
    return { action: String(n.query.action ?? "join"), wsUrl: `${$ ? "wss" : "ws"}://${window.location.hostname || "127.0.0.1"}:7700/ws`, displayName: "player", password: "", roomId: Number(n.query.room ?? 1) || 1, players: u.value, seed: c.value };
  }
  function N() {
    return w.value ? l : i;
  }
  async function U() {
    await C(async () => {
      const x = await N().endPhase(r.value);
      await fe(x);
    });
  }
  async function K() {
    await C(async () => {
      if (w.value) throw new Error("\u8054\u673A\u6A21\u5F0F\u4E0D\u63D0\u4F9B\u5BA2\u6237\u7AEF\u81EA\u52A8\u6B65\u8FDB");
      const $ = await i.autoStep();
      return await fe($), true;
    }) || O();
  }
  async function de(x, $) {
    await C(async () => {
      const G = await N().playCard(r.value, x, $);
      await fe(G), me(t, `\u51FA\u724C #${x}${$.length > 0 ? ` \u2192 P${$[0]}` : ""}`);
    });
  }
  async function Te(x, $, G) {
    await C(async () => {
      const j = await N().activateSkill(r.value, x, $, G);
      await fe(j), me(t, `\u53D1\u52A8\u6280\u80FD #${x}${$.length > 0 ? ` \u2192 P${$[0]}` : ""}`);
    });
  }
  async function je(x) {
    await C(async () => {
      const $ = await N().resolveChoice(r.value, x.ResolveChoice.choice_id, x.ResolveChoice.selections, x.ResolveChoice.cancelled);
      await fe($);
    });
  }
  const b = L(() => t.legalActions.filter((x) => typeof x == "object" && "ResolveChoice" in x)), q = L(() => t.connected && t.legalActions.includes("EndPhase"));
  function R() {
    if (m.value) {
      O();
      return;
    }
    m.value = true;
    const x = async () => {
      if (!m.value || t.finished) {
        O();
        return;
      }
      if (await K(), m.value && !t.finished) {
        const $ = t.skipAnims || t.reduceMotion ? 30 : 280 / t.speed;
        _.value = window.setTimeout(x, $);
      } else O();
    };
    x();
  }
  function O() {
    m.value = false, _.value != null && (clearTimeout(_.value), _.value = null);
  }
  function H(x) {
    return x.includes("\u4F24\u5BB3") || x.includes("\u9635\u4EA1") ? "log-damage" : x.includes("\u56DE\u590D") || x.includes("\u6478\u724C") ? "log-positive" : x.includes("\u6FD2\u6B7B") || x.includes("\u7B49\u5F85") ? "log-warning" : x.includes("\u56DE\u5408") || x.includes("\u4F7F\u7528") ? "log-action" : "";
  }
  const X = I(false);
  function F(x) {
    if (x.key === "?") {
      X.value = !X.value;
      return;
    }
    if (x.key === "Escape") {
      X.value = false;
      return;
    }
    t.busy || t.finished || !t.connected || (x.key === " " ? (x.preventDefault(), w.value || K()) : x.key === "e" || x.key === "E" ? q.value && U() : (x.key === "s" || x.key === "S") && (t.skipAnims = !t.skipAnims));
  }
  function B() {
    O(), i.dispose(), l.dispose(), s = void 0, ha(false), lt(false);
  }
  gn(() => {
    window.addEventListener("keydown", F), window.addEventListener("pagehide", B), Pe().catch((x) => {
      const $ = x instanceof Error ? x.message : String(x);
      p.value = `\u8FDE\u63A5\u5931\u8D25\uFF1A${$}`, me(t, `\u8FDE\u63A5\u5931\u8D25\uFF1A${$}`), t.connected = false;
    });
  }), sa(() => {
    clearInterval(P), window.removeEventListener("keydown", F), window.removeEventListener("pagehide", B), B();
  }), Be(() => t.speed, () => {
  });
  const ee = { Prepare: "\u51C6\u5907", Draw: "\u6478\u724C", Play: "\u51FA\u724C", Discard: "\u5F03\u724C", Judge: "\u5224\u5B9A", End: "\u7ED3\u675F" }, le = L(() => Zm(t.view, t.lastEvents ?? [], r.value));
  function te() {
    a.push("/");
  }
  const nn = L(() => {
    var _a3;
    if (t.view && t.view.phase !== "Idle" && (t.view.current_player !== 0 || t.view.seats.some(($) => $.player === 0))) return { currentPlayer: t.view.current_player, phase: t.view.phase, phaseLabel: ee[t.view.phase] ?? t.view.phase, round: t.view.round };
    const x = t.lastEvents ?? [];
    for (const $ of x) {
      const G = $;
      if ((G == null ? void 0 : G.type) === "TurnStateChanged") return { currentPlayer: G.current ?? 0, phase: G.phase ?? "?", phaseLabel: ee[G.phase ?? ""] ?? G.phase ?? "?", round: ((_a3 = t.view) == null ? void 0 : _a3.round) ?? 0 };
    }
    return null;
  });
  return (x, $) => {
    const G = ki("auto-scroll");
    return h(), g("div", Bh, [S(" Keyboard hotkey overlay "), X.value ? (h(), g("section", Fh, [...$[4] || ($[4] = [yt('<h2 data-v-14eda8d9>\u2328 \u5FEB\u6377\u952E</h2><dl class="hotkey-list" data-v-14eda8d9><dt data-v-14eda8d9><kbd data-v-14eda8d9>Space</kbd></dt><dd data-v-14eda8d9>\u81EA\u52A8\u4E00\u6B65</dd><dt data-v-14eda8d9><kbd data-v-14eda8d9>E</kbd></dt><dd data-v-14eda8d9>\u7ED3\u675F\u9636\u6BB5</dd><dt data-v-14eda8d9><kbd data-v-14eda8d9>S</kbd></dt><dd data-v-14eda8d9>\u5207\u6362\u8DF3\u8FC7\u52A8\u753B</dd><dt data-v-14eda8d9><kbd data-v-14eda8d9>?</kbd></dt><dd data-v-14eda8d9>\u663E\u793A/\u9690\u85CF\u6B64\u9762\u677F</dd><dt data-v-14eda8d9><kbd data-v-14eda8d9>Esc</kbd></dt><dd data-v-14eda8d9>\u5173\u95ED\u6B64\u9762\u677F</dd></dl>', 2)])])) : S("v-if", true), S(" Match outcome banner "), z(t).finished && le.value ? (h(), g("section", { key: 1, class: oe(["panel outcome-banner", { win: le.value.isSelfWin === true, lose: le.value.isSelfWin === false }]), role: "alert", "aria-live": "assertive" }, [o("div", Hh, [o("span", Wh, f(le.value.isSelfWin === null ? "\u2691" : le.value.isSelfWin ? "\u{1F389}" : "\u{1F480}"), 1), o("div", null, [o("h2", Kh, f(le.value.label), 1), o("p", Jh, "rev " + f(z(t).revision), 1)])]), o("button", { class: "primary", type: "button", onClick: $[0] || ($[0] = (j) => w.value ? te() : Pe()) }, f(w.value ? "\u8FD4\u56DE\u5927\u5385" : "\u518D\u6765\u4E00\u5C40"), 1)], 2)) : S("v-if", true), nn.value ? (h(), g("section", Qh, [o("span", Yh, "\u25BA \u7B2C " + f(nn.value.round) + " \u8F6E \xB7 P" + f(nn.value.currentPlayer), 1), o("span", { class: "phase-badge", "data-phase": nn.value.phase }, f(nn.value.phaseLabel), 9, Xh)])) : S("v-if", true), o("section", Zh, [o("div", eg, [w.value ? (h(), g("span", ag, "\u8054\u673A\u623F\u95F4 #" + f(y.value || "\u2026"), 1)) : (h(), g("button", { key: 0, class: "primary", type: "button", disabled: z(t).busy, onClick: Pe }, " \u91CD\u65B0\u5F00\u5C40 ", 8, ng)), o("button", { type: "button", disabled: z(t).busy || z(t).finished || D.value || w.value && !k.value || !q.value, onClick: U }, " \u7ED3\u675F\u9636\u6BB5 ", 8, tg), w.value ? S("v-if", true) : (h(), g("button", { key: 2, type: "button", disabled: z(t).busy || z(t).finished, onClick: K }, " \u81EA\u52A8\u4E00\u6B65 ", 8, ig)), w.value ? S("v-if", true) : (h(), g("button", { key: 3, type: "button", disabled: z(t).busy, onClick: R }, f(m.value ? "\u505C\u6B62\u81EA\u52A8" : "\u8FDE\u7EED\u81EA\u52A8"), 9, sg)), w.value ? (h(), g("button", { key: 4, type: "button", disabled: !z(t).busy, onClick: Ie }, " \u53D6\u6D88\u5F85\u786E\u8BA4 ", 8, og)) : S("v-if", true), w.value ? S("v-if", true) : (h(), g("button", { key: 5, type: "button", disabled: z(t).busy || !z(t).connected, onClick: qe }, " \u5B58\u6863 ", 8, rg)), w.value ? S("v-if", true) : (h(), g("button", { key: 6, type: "button", disabled: z(t).busy || !z(t).connected, onClick: We }, " \u8BFB\u6863 ", 8, lg))]), o("div", ug, [o("label", null, [$[6] || ($[6] = W(" \u500D\u901F ", -1)), Z(o("select", { "onUpdate:modelValue": $[1] || ($[1] = (j) => z(t).speed = j), "aria-label": "\u64AD\u653E\u500D\u901F" }, [...$[5] || ($[5] = [o("option", { value: 0.5 }, "0.5\xD7", -1), o("option", { value: 1 }, "1\xD7", -1), o("option", { value: 2 }, "2\xD7", -1), o("option", { value: 4 }, "4\xD7", -1)])], 512), [[_e, z(t).speed, void 0, { number: true }]])]), o("label", cg, [Z(o("input", { "onUpdate:modelValue": $[2] || ($[2] = (j) => z(t).skipAnims = j), type: "checkbox" }, null, 512), [[wn, z(t).skipAnims]]), $[7] || ($[7] = W(" \u8DF3\u8FC7\u52A8\u753B ", -1))]), o("span", null, f(D.value ? "\u516C\u5171\u65C1\u89C2\u6295\u5F71" : `\u89C2\u5BDF\u8005 P${r.value}`), 1)]), o("p", dg, f(p.value), 1), z(t).error ? (h(), g("p", mg, f(z(t).error), 1)) : S("v-if", true)]), M.value ? (h(), vt(Gh, { key: 3, state: A.value, busy: z(t).busy, onChoose: ve, onReload: $e }, null, 8, ["state", "busy"])) : S("v-if", true), w.value && d.value ? (h(), g("section", pg, [o("div", hg, [o("h2", null, "\u623F\u95F4 #" + f(d.value.room_id), 1), d.value.choice_timeout_secs ? (h(), g("span", gg, "\u9009\u62E9\u65F6\u9650 " + f(d.value.choice_timeout_secs) + " \u79D2", 1)) : S("v-if", true), o("span", { class: oe(d.value.started ? "started" : "waiting") }, f(d.value.started ? "\u5BF9\u5C40\u8FDB\u884C\u4E2D" : "\u7B49\u5F85\u73A9\u5BB6"), 3)]), o("div", fg, [(h(true), g(Q, null, Y(d.value.seats, (j) => (h(), g("span", { key: j.seat, class: "online-seat" }, [W(f(j.seat + 1) + " \xB7 " + f(j.display_name) + " ", 1), j.managed ? (h(), g("strong", _g, "\u6258\u7BA1\u4E2D")) : S("v-if", true), o("i", { class: oe(j.connected ? "connected" : "offline") }, f(j.connected ? "\u5728\u7EBF" : "\u79BB\u7EBF"), 3)]))), 128))])])) : S("v-if", true), z(t).choice ? (h(), g("section", vg, [o("h2", null, f(z(vi)(z(t).choice)), 1), z(t).choice.deadline_unix_ms != null ? (h(), g("p", yg, f(V.value === null ? "\u6B63\u5728\u6821\u51C6\u670D\u52A1\u5668\u65F6\u95F4\u2026" : `\u5269\u4F59 ${V.value} \u79D2 \xB7 \u5230\u65F6\u6258\u7BA1`), 1)) : S("v-if", true), o("p", kg, " choice #" + f(z(t).choice.choice_id) + " \xB7 \u73A9\u5BB6 " + f(z(t).choice.observer) + " \xB7 " + f(z(yi)(z(t).choice)), 1), b.value.length ? (h(), g("div", wg, [(h(true), g(Q, null, Y(b.value, (j, Le) => (h(), g("button", { key: Le, type: "button", disabled: z(t).busy || !z(t).connected || D.value, onClick: (Rn) => je(j) }, f(j.ResolveChoice.cancelled ? "\u53D6\u6D88" : `\u9009\u62E9 ${j.ResolveChoice.selections.join(", ") || "\u7A7A"}`), 9, bg))), 128))])) : (h(), g("p", xg, "\u5F53\u524D\u89C2\u5BDF\u8005\u65E0\u6743\u5904\u7406\u6B64\u9009\u62E9\u3002"))])) : S("v-if", true), S(" Player overview roster "), z(t).view ? (h(), g("section", $g, [$[8] || ($[8] = o("h2", null, "\u73A9\u5BB6\u6982\u51B5", -1)), o("div", jg, [(h(true), g(Q, null, Y(z(t).view.seats, (j) => (h(), g("div", { key: j.player, class: oe(["roster-entry", { dead: !j.alive, self: j.player === r.value }]) }, [o("span", Sg, "P" + f(j.player) + f(j.turned_over ? " \xB7 \u7FFB\u9762" : ""), 1), o("span", { class: "roster-hp", style: mn({ color: j.alive ? j.hp <= 0 ? "var(--color-warning)" : "inherit" : "var(--color-negative)" }) }, f(j.alive ? j.hp <= 0 ? "\u26A0" : "" : "\u2717") + " " + f(j.hp) + "/" + f(j.max_hp), 5), o("span", Eg, f(j.hand_count) + "\u{1F0A0}", 1), j.identity ? (h(), g("span", { key: 0, class: "roster-id", "data-id": j.identity }, f(j.identity === "Lord" ? "\u4E3B" : j.identity === "Loyalist" ? "\u5FE0" : j.identity === "Rebel" ? "\u53CD" : "\u5978"), 9, zg)) : (h(), g("span", Ag, "?"))], 2))), 128))])])) : S("v-if", true), o("section", Cg, [pe(Sh, { view: z(t).view, "self-id": r.value, "legal-actions": z(t).connected && !z(t).busy ? z(t).legalActions : [], onPlayCard: de, onActivateSkill: Te }, null, 8, ["view", "self-id", "legal-actions"]), S(" Effect overlay "), z(t).effects.length ? (h(), g("div", Rg, [(h(true), g(Q, null, Y(z(t).effects, (j) => (h(), g("div", { key: j.id, class: oe(["effect-popup", j.type]) }, [o("span", Ng, f(j.type === "damage" ? "\u2694" : j.type === "heal" ? "\u271A" : j.type === "draw" ? "\u{1F0A0}" : j.type === "play" ? "\u25B8" : j.type === "dying" ? "\u26A0" : "\u2717"), 1), o("span", Pg, f(j.text), 1), j.player != null ? (h(), g("span", qg, "P" + f(j.player), 1)) : S("v-if", true)], 2))), 128))])) : S("v-if", true)]), S(" Match stats panel "), z(t).view ? (h(), g("section", Ig, [o("div", Og, [o("div", Tg, [$[9] || ($[9] = o("span", { class: "stat-label muted" }, "Revision", -1)), o("span", Lg, f(z(t).revision), 1)]), o("div", Dg, [$[10] || ($[10] = o("span", { class: "stat-label muted" }, "State Hash", -1)), o("span", Mg, f(z(t).hash.slice(0, 16)) + "\u2026", 1)]), o("div", Ug, [$[11] || ($[11] = o("span", { class: "stat-label muted" }, "\u5DF2\u77E5\u5361\u724C", -1)), o("span", Vg, f(z(t).view.known_cards.length), 1)]), o("div", Gg, [$[12] || ($[12] = o("span", { class: "stat-label muted" }, "\u5B58\u6D3B\u73A9\u5BB6", -1)), o("span", Bg, f(z(t).view.seats.filter((j) => j.alive).length) + " / " + f(z(t).view.seats.length), 1)])])])) : S("v-if", true), S(" Seat status overview "), z(t).view ? (h(), g("section", Fg, [$[13] || ($[13] = o("h2", null, "\u5EA7\u4F4D\u72B6\u6001", -1)), o("div", Hg, [(h(true), g(Q, null, Y(z(t).view.seats, (j) => (h(), g("div", { key: "ss-" + j.player, class: oe(["ss-entry", { "ss-dead": !j.alive, "ss-self": j.player === r.value }]) }, [o("div", Wg, [o("span", Kg, "P" + f(j.player), 1), j.player === r.value ? (h(), g("span", Jg, "\u4F60")) : S("v-if", true)]), o("div", Qg, [o("div", { class: "ss-hp-fill", style: mn({ width: Math.max(Math.min(j.hp / j.max_hp * 100, 100), 0) + "%", background: j.alive ? j.hp <= 0 ? "var(--color-warning)" : "var(--color-positive)" : "var(--color-negative)" }) }, null, 4)]), o("div", Yg, [o("span", Xg, f(j.alive ? j.hp <= 0 ? "\u26A0" : "" : "\u2717"), 1), W(" " + f(Math.max(j.hp, 0)) + "/" + f(j.max_hp), 1)]), o("div", Zg, [o("span", null, f(j.hand_count) + "\u{1F0A0}", 1), j.identity ? (h(), g("span", { key: 0, class: "ss-id", "data-id": j.identity }, f(j.identity === "Lord" ? "\u4E3B" : j.identity === "Loyalist" ? "\u5FE0" : j.identity === "Rebel" ? "\u53CD" : "\u5978"), 9, ef)) : (h(), g("span", nf, "?"))])], 2))), 128))])])) : S("v-if", true), o("section", af, [o("div", tf, [$[14] || ($[14] = o("h2", null, "\u6218\u6597\u65E5\u5FD7", -1)), z(t).log.length > 0 ? (h(), g("button", { key: 0, type: "button", class: "clear-btn", onClick: $[3] || ($[3] = (j) => z(t).log = []), "aria-label": "\u6E05\u7A7A\u65E5\u5FD7" }, "\u6E05\u7A7A")) : S("v-if", true)]), $[15] || ($[15] = o("p", { class: "muted tiny" }, " \u65E5\u5FD7\u6765\u81EA DomainEvent \u6458\u8981\uFF0C\u4E0D\u662F\u72B6\u6001\u771F\u76F8\u6E90\u3002\u771F\u76F8\u6E90\u4E3A Command Log + \u6295\u5F71\uFF08\xA722\uFF09\u3002 ", -1)), Z((h(), g("ol", sf, [(h(true), g(Q, null, Y(z(t).log.slice().reverse(), (j) => (h(), g("li", { key: j.id, class: oe(["log-entry", H(j.text)]) }, [o("span", of, "rev" + f(j.at), 1), o("span", rf, f(j.text), 1)], 2))), 128))])), [[G]]), z(t).log.length === 0 ? (h(), g("p", lf, "\u6682\u65E0\u4E8B\u4EF6")) : S("v-if", true)])]);
  };
} }), cf = ge(uf, [["__scopeId", "data-v-14eda8d9"]]), df = { class: "a11y-group" }, mf = { class: "row" }, pf = ["disabled"], hf = ["disabled"], gf = { class: "legend-list", "aria-live": "polite" }, ff = ["disabled", "onClick"], _f = ["disabled", "onClick"], vf = ["disabled", "onClick"], yf = { key: 0, class: "muted" }, kf = { class: "status", role: "status" }, mt = "noname-next.native-server", wf = se({ __name: "NativePackagesPanel", setup(e) {
  const n = I(typeof localStorage < "u" && localStorage.getItem(mt) || "http://127.0.0.1:7700"), a = I([]), t = I(false), i = I("");
  function s() {
    try {
      localStorage.setItem(mt, n.value);
    } catch {
    }
  }
  async function l(_, m) {
    const p = await fetch(new URL(_, n.value), { ...m, headers: { "content-type": "application/json", ...(m == null ? void 0 : m.headers) ?? {} } });
    if (!p.ok) throw new Error(await p.text() || `HTTP ${p.status}`);
    return p;
  }
  async function r() {
    t.value = true;
    try {
      const _ = await l("/native/packages");
      a.value = await _.json(), i.value = `\u5DF2\u8F7D\u5165 ${a.value.length} \u4E2A\u539F\u751F\u5305`;
    } catch (_) {
      i.value = `\u8BFB\u53D6\u5931\u8D25\uFF1A${_ instanceof Error ? _.message : String(_)}`;
    } finally {
      t.value = false;
    }
  }
  async function u() {
    t.value = true;
    try {
      await l("/native/packages/import", { method: "POST", body: "{}" }), await r(), i.value = "\u5B89\u88C5\u626B\u63CF\u5B8C\u6210";
    } catch (_) {
      i.value = `\u5B89\u88C5\u5931\u8D25\uFF1A${_ instanceof Error ? _.message : String(_)}`;
    } finally {
      t.value = false;
    }
  }
  async function c(_, m) {
    if (!(_ === "uninstall" && !window.confirm(`\u5378\u8F7D ${m.package_id} \u7684\u6240\u6709\u7248\u672C\uFF1F`))) {
      t.value = true;
      try {
        await l(`/native/packages/${_}`, { method: "POST", body: JSON.stringify({ namespace: m.namespace, package_id: m.package_id, version: m.version }) }), await r(), i.value = `${_} \u5B8C\u6210\uFF1A${m.package_id}`;
      } catch (p) {
        i.value = `${_} \u5931\u8D25\uFF1A${p instanceof Error ? p.message : String(p)}`;
      } finally {
        t.value = false;
      }
    }
  }
  return gn(r), (_, m) => (h(), g("fieldset", df, [m[2] || (m[2] = o("legend", null, "\u539F\u751F\u6269\u5C55\u5305", -1)), m[3] || (m[3] = o("p", { class: "muted", role: "note" }, " \u901A\u8FC7\u670D\u52A1\u7AEF\u6B63\u5F0F\u5165\u53E3\u5B89\u88C5\u3001\u542F\u505C\u3001\u5347\u7EA7\u6216\u5378\u8F7D\u539F\u751F\u5305\uFF1B\u5BA2\u6237\u7AEF\u65E0\u9700\u91CD\u65B0\u6784\u5EFA\u3002 ", -1)), o("div", mf, [o("label", null, [m[1] || (m[1] = W(" \u670D\u52A1\u7AEF ", -1)), Z(o("input", { "onUpdate:modelValue": m[0] || (m[0] = (p) => n.value = p), type: "url", "aria-label": "\u539F\u751F\u670D\u52A1\u7AEF\u5730\u5740", onChange: s }, null, 544), [[Re, n.value]])]), o("button", { type: "button", disabled: t.value, onClick: r }, "\u5237\u65B0", 8, pf), o("button", { type: "button", disabled: t.value, onClick: u }, "\u626B\u63CF\u5B89\u88C5", 8, hf)]), o("ul", gf, [(h(true), g(Q, null, Y(a.value, (p) => (h(), g("li", { key: `${p.namespace}/${p.package_id}@${p.version}` }, [o("code", null, f(p.namespace) + "/" + f(p.package_id) + "@" + f(p.version), 1), o("button", { type: "button", disabled: t.value, onClick: (d) => c("activate", p) }, "\u6FC0\u6D3B", 8, ff), o("button", { type: "button", disabled: t.value, onClick: (d) => c("disable", p) }, "\u505C\u7528", 8, _f), o("button", { type: "button", disabled: t.value, onClick: (d) => c("uninstall", p) }, "\u5378\u8F7D", 8, vf)]))), 128))]), a.value.length === 0 ? (h(), g("p", yf, "\u5C1A\u672A\u8BFB\u53D6\u5230\u5DF2\u5B89\u88C5\u5305\u3002")) : S("v-if", true), o("p", kf, f(i.value), 1)]));
} }), bf = { class: "panel stack", "aria-labelledby": "settings-title" }, xf = { key: 0, class: "a11y-group" }, $f = { class: "select-row" }, jf = { class: "a11y-group" }, Sf = { class: "check" }, Ef = { class: "check" }, zf = { class: "check" }, Af = { class: "select-row" }, Cf = { class: "select-row" }, Rf = { class: "select-row" }, pt = "noname-next-a11y", Nf = se({ __name: "SettingsView", setup(e) {
  const n = ia(), a = I(wi());
  Be(a, (p) => localStorage.setItem(bi, p));
  const t = I(typeof window < "u" && window.matchMedia("(prefers-reduced-motion: reduce)").matches), i = I(false), s = I(false), l = I("normal"), r = I("none"), u = I("default");
  function c() {
    if (typeof window > "u") return null;
    try {
      const p = localStorage.getItem(pt);
      return p ? JSON.parse(p) : null;
    } catch {
      return null;
    }
  }
  function _() {
    if (typeof window > "u") return;
    const p = { highContrast: i.value, skipAnimations: s.value, fontScale: l.value, colorBlind: r.value, screenReader: u.value };
    localStorage.setItem(pt, JSON.stringify(p));
  }
  function m() {
    const p = document.documentElement;
    p.dataset.contrast = i.value ? "high" : "", p.dataset.skipAnimations = s.value ? "true" : "false", p.dataset.fontScale = l.value, p.dataset.colorBlind = r.value !== "none" ? r.value : "";
  }
  return gn(() => {
    const p = c();
    p && (i.value = p.highContrast, s.value = p.skipAnimations, l.value = p.fontScale || "normal", r.value = p.colorBlind || "none", u.value = p.screenReader || "default"), m();
  }), Be([i, s, l, r, u], () => {
    m(), _();
  }), (p, d) => (h(), g("section", bf, [d[23] || (d[23] = o("h1", { id: "settings-title" }, "\u8BBE\u7F6E", -1)), d[24] || (d[24] = o("p", { class: "muted", role: "note" }, " \u8868\u73B0\u5C42\u8BBE\u7F6E\u4E0D\u5F71\u54CD\u89C4\u5219\u6838\u5FC3\uFF08\xA730 \u53EF\u8BBF\u95EE\u6027\uFF09\u3002\u6240\u6709\u8BBE\u7F6E\u5373\u65F6\u751F\u6548\u5E76\u4FDD\u5B58\u5230\u672C\u5730\u3002 ", -1)), z(n) ? (h(), g("fieldset", xf, [d[9] || (d[9] = o("legend", null, "\u684C\u9762\u5F15\u64CE", -1)), o("label", $f, [d[8] || (d[8] = o("span", null, "\u4E0B\u6B21\u8FDE\u63A5\u4F7F\u7528", -1)), Z(o("select", { "onUpdate:modelValue": d[0] || (d[0] = (v) => a.value = v), "aria-label": "\u684C\u9762\u5F15\u64CE" }, [...d[7] || (d[7] = [o("option", { value: "wasm" }, "WASM Worker\uFF08\u9ED8\u8BA4 / \u56DE\u9000\uFF09", -1), o("option", { value: "native" }, "\u539F\u751F\u5F15\u64CE\uFF08\u5B9E\u9A8C\u539F\u578B\uFF09", -1)])], 512), [[_e, a.value]])]), d[10] || (d[10] = o("p", { class: "muted" }, "\u5F53\u524D\u5BF9\u5C40\u4E0D\u5207\u6362\u3002\u64CD\u4F5C\u5931\u8D25\u65F6\u4FDD\u7559\u5F53\u524D\u5F15\u64CE\uFF0C\u8BF7\u5148\u6838\u5BF9\u5C40\u9762\u3002", -1))])) : S("v-if", true), o("fieldset", jf, [d[20] || (d[20] = o("legend", null, "\u53EF\u8BBF\u95EE\u6027\uFF08\xA730\uFF09", -1)), S(" \xA730: \u51CF\u5C11\u52A8\u6001\u6548\u679C "), o("label", Sf, [Z(o("input", { "onUpdate:modelValue": d[1] || (d[1] = (v) => t.value = v), type: "checkbox", disabled: "", "aria-describedby": "reduce-motion-desc" }, null, 512), [[wn, t.value]]), d[11] || (d[11] = o("span", null, "\u51CF\u5C11\u52A8\u6001\u6548\u679C\uFF08\u7CFB\u7EDF\u504F\u597D\uFF0C\u53EA\u8BFB\uFF09", -1))]), d[21] || (d[21] = o("p", { id: "reduce-motion-desc", class: "sr-only" }, " \u9075\u5FAA\u7CFB\u7EDF prefers-reduced-motion \u8BBE\u7F6E\u3002 ", -1)), S(" \xA730: \u52A8\u753B\u8DF3\u8FC7 "), o("label", Ef, [Z(o("input", { "onUpdate:modelValue": d[2] || (d[2] = (v) => s.value = v), type: "checkbox", "aria-describedby": "skip-anim-desc" }, null, 512), [[wn, s.value]]), d[12] || (d[12] = o("span", null, "\u8DF3\u8FC7\u52A8\u753B\uFF08\u8DF3\u8FC7\u540E\u4ECD\u53EF\u7406\u89E3\u89C4\u5219\u7ED3\u679C\uFF09", -1))]), d[22] || (d[22] = o("p", { id: "skip-anim-desc", class: "sr-only" }, " \u8DF3\u8FC7\u6240\u6709\u52A8\u753B\uFF1B\u89C4\u5219\u7ED3\u679C\u4ECD\u901A\u8FC7\u6587\u5B57\u65E5\u5FD7\u548C\u56FE\u6807\u5B8C\u6574\u4F20\u8FBE\u3002 ", -1)), S(" \xA730: \u9AD8\u5BF9\u6BD4\u5EA6 "), o("label", zf, [Z(o("input", { "onUpdate:modelValue": d[3] || (d[3] = (v) => i.value = v), type: "checkbox" }, null, 512), [[wn, i.value]]), d[13] || (d[13] = o("span", null, "\u9AD8\u5BF9\u6BD4\u5EA6\u6A21\u5F0F", -1))]), S(" \xA730: \u5B57\u53F7\u7F29\u653E "), o("label", Af, [d[15] || (d[15] = o("span", null, "\u5B57\u53F7\u7F29\u653E", -1)), Z(o("select", { "onUpdate:modelValue": d[4] || (d[4] = (v) => l.value = v), "aria-label": "\u5B57\u53F7\u7F29\u653E" }, [...d[14] || (d[14] = [o("option", { value: "small" }, "\u5C0F (85%)", -1), o("option", { value: "normal" }, "\u6807\u51C6 (100%)", -1), o("option", { value: "large" }, "\u5927 (125%)", -1), o("option", { value: "xlarge" }, "\u7279\u5927 (150%)", -1)])], 512), [[_e, l.value]])]), S(" \xA730: \u8272\u89C9\u53CB\u597D "), o("label", Cf, [d[17] || (d[17] = o("span", null, "\u8272\u89C9\u53CB\u597D\u6A21\u5F0F", -1)), Z(o("select", { "onUpdate:modelValue": d[5] || (d[5] = (v) => r.value = v), "aria-label": "\u8272\u89C9\u53CB\u597D\u6A21\u5F0F" }, [...d[16] || (d[16] = [yt('<option value="none" data-v-637706ba>\u5173\u95ED</option><option value="protanopia" data-v-637706ba>\u7EA2\u8272\u76F2\uFF08Protanopia\uFF09</option><option value="deuteranopia" data-v-637706ba>\u7EFF\u8272\u76F2\uFF08Deuteranopia\uFF09</option><option value="tritanopia" data-v-637706ba>\u84DD\u8272\u76F2\uFF08Tritanopia\uFF09</option><option value="achromatopsia" data-v-637706ba>\u5168\u8272\u76F2\uFF08Achromatopsia\uFF09</option>', 5)])], 512), [[_e, r.value]])]), S(" \xA730: \u5C4F\u5E55\u9605\u8BFB\u5668 "), o("label", Rf, [d[19] || (d[19] = o("span", null, "\u5C4F\u5E55\u9605\u8BFB\u5668\u8BE6\u7EC6\u5EA6", -1)), Z(o("select", { "onUpdate:modelValue": d[6] || (d[6] = (v) => u.value = v), "aria-label": "\u5C4F\u5E55\u9605\u8BFB\u5668\u8BE6\u7EC6\u5EA6" }, [...d[18] || (d[18] = [o("option", { value: "off" }, "\u5173\u95ED", -1), o("option", { value: "default" }, "\u6807\u51C6", -1), o("option", { value: "verbose" }, "\u8BE6\u7EC6\uFF08\u64AD\u62A5\u6240\u6709\u72B6\u6001\u53D8\u5316\uFF09", -1)])], 512), [[_e, u.value]])])]), d[25] || (d[25] = o("div", { class: "state-legend", role: "region", "aria-label": "\u72B6\u6001\u4FE1\u53F7\u56FE\u4F8B" }, [o("h2", null, "\u72B6\u6001\u4FE1\u53F7\u8BF4\u660E"), o("p", { class: "muted" }, " \u6240\u6709\u6E38\u620F\u72B6\u6001\u901A\u8FC7\u56FE\u6807 + \u6587\u5B57\u4F20\u8FBE\uFF0C\u4E0D\u53EA\u4F9D\u8D56\u989C\u8272\uFF08\xA730: \u4E0D\u53EA\u4F9D\u8D56\u989C\u8272\u8868\u8FBE\u72B6\u6001\uFF09\u3002 "), o("ul", { class: "legend-list" }, [o("li", null, [o("span", { class: "state-icon", style: { color: "var(--color-positive)" } }, "\u2713"), W(" \u5B58\u6D3B")]), o("li", null, [o("span", { class: "state-icon", style: { color: "var(--color-warning)" } }, "\u26A0"), W(" \u6FD2\u6B7B")]), o("li", null, [o("span", { class: "state-icon", style: { color: "var(--color-negative)" } }, "\u2717"), W(" \u9635\u4EA1")]), o("li", null, [o("span", { class: "state-icon", style: { color: "var(--color-positive)" } }, "\u25B8"), W(" \u53EF\u51FA\u724C")]), o("li", null, [o("span", { class: "state-icon", style: { color: "var(--color-negative)" } }, "\u2715"), W(" \u4E0D\u53EF\u7528")]), o("li", null, [o("span", { class: "state-icon", style: { color: "var(--color-accent)" } }, "\u25BA"), W(" \u5F53\u524D\u56DE\u5408")]), o("li", null, [o("span", { class: "state-icon", style: { color: "var(--color-warning)" } }, "?"), W(" \u8BF7\u9009\u62E9")])])], -1)), pe(wf)]));
} }), Pf = ge(Nf, [["__scopeId", "data-v-637706ba"]]), qf = { class: "replay stack" }, If = { key: 0, class: "muted" }, Of = { key: 1, class: "error", role: "alert" }, Tf = { key: 0, class: "panel" }, Lf = { class: "info-grid" }, Df = { class: "mono" }, Mf = { class: "mono" }, Uf = ["disabled"], Vf = { key: 1, class: "panel" }, Gf = { class: "row playback-controls" }, Bf = ["disabled"], Ff = ["disabled"], Hf = { class: "step-display mono" }, Wf = ["disabled"], Kf = ["disabled"], Jf = ["aria-valuenow", "aria-valuemax"], Qf = { class: "info-grid frame-info" }, Yf = { class: "mono" }, Xf = { key: 2, class: "panel" }, Zf = { key: 0, class: "seat-grid" }, e_ = { key: 1, class: "muted" }, n_ = { key: 3, class: "panel privacy-notice", role: "note" }, a_ = se({ __name: "ReplayView", setup(e) {
  const n = new oa(), a = I(null), t = I(null), i = I(null), s = I(null), l = I(false), r = I("full");
  async function u(d) {
    l.value = true, s.value = null;
    try {
      const v = JSON.parse(d);
      await n.connect(), t.value = await n.loadReplay(d), a.value = v, i.value = await n.seekReplay(0, r.value);
    } catch (v) {
      a.value = null, t.value = null, i.value = null, s.value = v instanceof Error ? v.message : String(v);
    } finally {
      l.value = false;
    }
  }
  function c(d) {
    const v = new FileReader();
    v.onload = () => void u(v.result), v.onerror = () => {
      s.value = "\u6587\u4EF6\u8BFB\u53D6\u5931\u8D25";
    }, v.readAsText(d);
  }
  function _(d) {
    var _a2;
    const v = (_a2 = d.target.files) == null ? void 0 : _a2[0];
    v && c(v);
  }
  function m(d) {
    var _a2, _b;
    d.preventDefault();
    const v = (_b = (_a2 = d.dataTransfer) == null ? void 0 : _a2.files) == null ? void 0 : _b[0];
    v && c(v);
  }
  async function p(d) {
    if (!(!t.value || l.value)) {
      l.value = true, s.value = null;
      try {
        i.value = await n.seekReplay(Math.max(0, Math.min(d, t.value.total)), r.value);
      } catch (v) {
        s.value = v instanceof Error ? v.message : String(v);
      } finally {
        l.value = false;
      }
    }
  }
  return Be(r, () => {
    i.value && p(i.value.step);
  }), sa(() => n.dispose()), (d, v) => {
    var _a2;
    return h(), g("div", qf, [v[22] || (v[22] = o("section", { class: "panel" }, [o("h1", null, "\u5F55\u50CF\u56DE\u653E"), o("p", { class: "muted" }, "\u5BFC\u5165 Command Log \u540E\uFF0C\u89C4\u5219\u5F15\u64CE\u4F1A\u771F\u5B9E\u91CD\u6F14\u5E76\u9010\u6B65\u6838\u5BF9\u72B6\u6001\u54C8\u5E0C\u3002")], -1)), o("section", { class: "panel drop-zone", onDrop: m, onDragover: v[0] || (v[0] = Vn(() => {
    }, ["prevent"])), "aria-label": "\u5BFC\u5165\u5F55\u50CF\u6587\u4EF6" }, [v[6] || (v[6] = o("h2", null, "\u5BFC\u5165\u5F55\u50CF", -1)), v[7] || (v[7] = o("p", { class: "muted" }, "\u62D6\u62FD .json \u5F55\u50CF\u6587\u4EF6\u5230\u6B64\u5904\uFF0C\u6216\u70B9\u51FB\u9009\u62E9", -1)), o("input", { type: "file", accept: ".json,application/json", onChange: _, "aria-label": "\u9009\u62E9\u5F55\u50CF\u6587\u4EF6" }, null, 32), l.value ? (h(), g("p", If, "\u6B63\u5728\u9A8C\u8BC1\u5E76\u91CD\u6F14\u2026")) : S("v-if", true), s.value ? (h(), g("p", Of, f(s.value), 1)) : S("v-if", true)], 32), a.value && t.value ? (h(), g("section", Tf, [v[15] || (v[15] = o("h2", null, "\u5F55\u50CF\u4FE1\u606F", -1)), o("dl", Lf, [v[9] || (v[9] = o("dt", null, "Manifest Hash", -1)), o("dd", Df, f(t.value.manifest_hash), 1), v[10] || (v[10] = o("dt", null, "\u5F15\u64CE\u6784\u5EFA", -1)), o("dd", Mf, f(((_a2 = a.value.header) == null ? void 0 : _a2.engine_build_id) ?? "\u2014"), 1), v[11] || (v[11] = o("dt", null, "\u603B\u6B65\u6570", -1)), o("dd", null, f(t.value.total), 1), v[12] || (v[12] = o("dt", null, "\u5B8C\u6574\u6027", -1)), v[13] || (v[13] = o("dd", { class: "verified" }, "\u2713 \u54C8\u5E0C\u94FE\u4E0E\u5168\u90E8\u72B6\u6001\u68C0\u67E5\u70B9\u5DF2\u9A8C\u8BC1", -1)), v[14] || (v[14] = o("dt", null, "\u9690\u79C1\u7B49\u7EA7", -1)), o("dd", null, [Z(o("select", { "onUpdate:modelValue": v[1] || (v[1] = (P) => r.value = P), disabled: l.value, "aria-label": "\u9690\u79C1\u7B49\u7EA7" }, [...v[8] || (v[8] = [o("option", { value: "full" }, "\u5B8C\u6574\uFF08\u88C1\u5224\u89C6\u89D2\uFF09", -1), o("option", { value: "player" }, "\u73A9\u5BB6 1 \u89C6\u89D2", -1), o("option", { value: "public" }, "\u516C\u5F00\u89C6\u89D2", -1)])], 8, Uf), [[_e, r.value]])])])])) : S("v-if", true), i.value && t.value ? (h(), g("section", Vf, [v[19] || (v[19] = o("h2", null, "\u56DE\u653E\u63A7\u5236", -1)), o("div", Gf, [o("button", { type: "button", onClick: v[2] || (v[2] = (P) => p(0)), disabled: l.value || i.value.step === 0 }, "\u23EE \u5F00\u5934", 8, Bf), o("button", { type: "button", onClick: v[3] || (v[3] = (P) => p(i.value.step - 1)), disabled: l.value || i.value.step === 0 }, "\u25C0 \u540E\u9000", 8, Ff), o("span", Hf, f(i.value.step) + " / " + f(i.value.total), 1), o("button", { type: "button", onClick: v[4] || (v[4] = (P) => p(i.value.step + 1)), disabled: l.value || i.value.step >= i.value.total }, "\u524D\u8FDB \u25B6", 8, Wf), o("button", { type: "button", onClick: v[5] || (v[5] = (P) => p(i.value.total)), disabled: l.value || i.value.step >= i.value.total }, "\u7ED3\u5C3E \u23ED", 8, Kf)]), o("div", { class: "progress-bar", role: "progressbar", "aria-valuenow": i.value.step, "aria-valuemax": i.value.total }, [o("div", { class: "progress-fill", style: mn({ width: i.value.step / Math.max(i.value.total, 1) * 100 + "%" }) }, null, 4)], 8, Jf), o("dl", Qf, [v[16] || (v[16] = o("dt", null, "Revision", -1)), o("dd", null, f(i.value.revision), 1), v[17] || (v[17] = o("dt", null, "State Hash", -1)), o("dd", Yf, f(i.value.hash_hex), 1), v[18] || (v[18] = o("dt", null, "\u6BD4\u8D5B\u72B6\u6001", -1)), o("dd", null, f(i.value.finished ? "\u5DF2\u7ED3\u675F" : "\u8FDB\u884C\u4E2D"), 1)])])) : S("v-if", true), i.value ? (h(), g("section", Xf, [v[20] || (v[20] = o("h2", null, "\u5F53\u524D\u5C40\u9762", -1)), i.value.view.seats.length ? (h(), g("div", Zf, [(h(true), g(Q, null, Y(i.value.view.seats, (P) => (h(), g("article", { key: P.player, class: "seat" }, [o("strong", null, "\u5EA7\u4F4D " + f(P.seat) + " \xB7 \u73A9\u5BB6 " + f(P.player), 1), o("span", null, "\u4F53\u529B " + f(P.hp) + " / " + f(P.max_hp), 1), o("span", null, "\u624B\u724C " + f(P.hand_count) + " \xB7 " + f(P.alive ? "\u5B58\u6D3B" : "\u9635\u4EA1") + f(P.turned_over ? " \xB7 \u7FFB\u9762" : ""), 1), o("span", null, "\u8EAB\u4EFD " + f(P.identity ?? "\u672A\u77E5"), 1)]))), 128))])) : (h(), g("p", e_, "\u6BD4\u8D5B\u5C1A\u672A\u5F00\u59CB\u3002"))])) : S("v-if", true), a.value ? (h(), g("section", n_, [...v[21] || (v[21] = [o("h2", null, "\u9690\u79C1\u5206\u7EA7\uFF08\xA722.5\uFF09", -1), o("p", { class: "muted tiny" }, "\u6BCF\u4E2A\u753B\u9762\u76F4\u63A5\u4ECE\u5BF9\u5E94\u7684\u88C1\u5224\u3001\u73A9\u5BB6\u6216\u516C\u5F00 Observer \u6295\u5F71\u6784\u5EFA\uFF0C\u4E0D\u4F1A\u5148\u751F\u6210\u5B8C\u6574\u89C6\u56FE\u518D\u5220\u5B57\u6BB5\u3002", -1)])])) : S("v-if", true)]);
  };
} }), t_ = ge(a_, [["__scopeId", "data-v-91fcaceb"]]);
function bn(e, n) {
  return typeof n != "string" || !n || !n.split("/").every((a) => a && a !== "." && a !== ".." && !/[\\%?#:\u0000-\u001f\u007f]/.test(a)) ? "" : `${e}/${n.split("/").map(encodeURIComponent).join("/")}`;
}
function i_(e, n) {
  const a = (r) => typeof r == "string" && !r.includes("/") && !!bn("", r);
  if (!a(n.pack) || !a(n.name)) return [];
  const t = /* @__PURE__ */ new Map();
  for (const r of e.assets) if (!(!r || typeof r.path != "string")) {
    if (t.has(r.path)) return [];
    t.set(r.path, r);
  }
  const i = `audio-declared/${n.pack}/character/${n.name}/`, s = [`audio-declared/${n.pack}/die/${n.name}/`, `${i}die/`, ...n.skills.filter(a).map((r) => `${i}skill/${r}/`)], l = /* @__PURE__ */ new Set();
  return e.assets.filter((r) => {
    if (!r || typeof r.path != "string" || !s.some((c) => r.path.startsWith(c)) || typeof r.source_path != "string" || typeof r.sha256 != "string" || r.kind !== "audio" || r.source_category !== "core" || !/^[a-f0-9]{64}$/.test(r.sha256) || !Number.isSafeInteger(r.size) || r.size <= 0 || !bn("", r.path) || !bn("", r.source_path) || !r.source_path.startsWith("audio/") || !/\.(?:mp3|ogg|wav|m4a|aac|flac|opus|mp4)$/.test(r.path)) return false;
    const u = t.get(r.source_path);
    return !u || u.kind !== "audio" || u.source_category !== "core" || u.path !== u.source_path || u.sha256 !== r.sha256 || u.size !== r.size || r.path.split("/").at(-1) !== u.path.split("/").at(-1) || l.has(r.source_path) ? false : (l.add(r.source_path), true);
  });
}
const s_ = { class: "stack" }, o_ = { class: "panel" }, r_ = { key: 0, class: "counts" }, l_ = { key: 1, class: "muted", role: "status" }, u_ = { key: 2, class: "error", role: "alert" }, c_ = { key: 0, class: "browser" }, d_ = { class: "panel roster" }, m_ = { class: "filters" }, p_ = ["value"], h_ = ["onClick"], g_ = ["src"], f_ = { key: 0, class: "muted limit" }, __ = { key: 0, class: "panel detail" }, v_ = ["src", "alt"], y_ = ["title"], k_ = { class: "muted" }, w_ = { key: 0, class: "skills" }, b_ = ["title"], x_ = { key: 1, class: "muted" }, $_ = { key: 2, class: "error", role: "alert" }, j_ = { key: 3, class: "skills" }, S_ = ["src", "aria-label", "onError", "onLoadeddata"], E_ = { key: 0, class: "error", role: "alert" }, z_ = { key: 4, class: "muted" }, A_ = { key: 5, class: "skins" }, C_ = ["src", "alt"], R_ = { key: 6, class: "muted" }, N_ = { class: "muted" }, P_ = se({ __name: "CollectionView", setup(e) {
  const n = I(null), a = I({ assets: [] }), t = I(""), i = I(/* @__PURE__ */ new Set()), s = I(""), l = I(false), r = I(""), u = I(""), c = I(null), _ = D(), m = L(() => {
    var _a2;
    return new Map((_a2 = n.value) == null ? void 0 : _a2.skills.map((A) => [A.id, A]));
  }), p = L(() => {
    var _a2;
    const A = r.value.trim().toLowerCase();
    return (((_a2 = n.value) == null ? void 0 : _a2.characters) ?? []).filter((M) => !u.value || M.pack === u.value).filter((M) => !A || M.name.toLowerCase().includes(A) || M.display_name.toLowerCase().includes(A)).slice(0, 80);
  }), d = L(() => {
    var _a2;
    return ((_a2 = c.value) == null ? void 0 : _a2.resolved_skills.map((A) => m.value.get(A)).filter(Boolean)) ?? [];
  }), v = L(() => {
    var _a2;
    return (((_a2 = n.value) == null ? void 0 : _a2.skins.static) ?? []).filter((A) => {
      var _a3;
      return A.character === ((_a3 = c.value) == null ? void 0 : _a3.name);
    });
  }), P = L(() => {
    var _a2;
    return (((_a2 = n.value) == null ? void 0 : _a2.skins.dynamic) ?? []).filter((A) => {
      var _a3;
      return A.character === ((_a3 = c.value) == null ? void 0 : _a3.name);
    });
  }), V = L(() => c.value ? i_(a.value, c.value) : []), y = L(() => a.value.assets.filter((A) => (A == null ? void 0 : A.kind) === "audio" && A.path === A.source_path).length);
  gn(k);
  async function k() {
    l.value = true, s.value = "";
    try {
      const A = await fetch(`${_}/catalog.json`);
      if (!A.ok) throw new Error(`${A.status} ${A.statusText}`);
      n.value = await A.json(), c.value = n.value.characters[0] ?? null;
      try {
        const M = await fetch(`${_}/asset-manifest.json`);
        if (!M.ok) throw new Error(`${M.status} ${M.statusText}`);
        const C = await M.json();
        if (C.schema_version !== 2 || !Array.isArray(C.assets)) throw new Error("\u8D44\u6E90\u6E05\u5355\u65E0\u6548");
        a.value = C;
      } catch (M) {
        t.value = `\u65E0\u6CD5\u8BFB\u53D6\u8BED\u97F3\u8D44\u6E90\u6E05\u5355\uFF08${String(M)}\uFF09\u3002`;
      }
    } catch (A) {
      s.value = `\u65E0\u6CD5\u8BFB\u53D6\u539F\u4ED3\u5E93\u5185\u5BB9\u76EE\u5F55\uFF08${String(A)}\uFF09\u3002\u8BF7\u5148\u542F\u52A8 game-server\u3002`;
    } finally {
      l.value = false;
    }
  }
  function w(A) {
    return bn(_, A);
  }
  function D() {
    const M = !window.location.hostname || window.location.hostname.endsWith("tauri.localhost") ? "127.0.0.1" : window.location.hostname;
    return `${window.location.protocol === "https:" ? "https:" : "http:"}//${M}:7700/original-content`;
  }
  return (A, M) => (h(), g("div", s_, [o("section", o_, [M[2] || (M[2] = o("h1", null, "\u539F\u4ED3\u5E93\u6B66\u5C06\u5E93", -1)), M[3] || (M[3] = o("p", { class: "muted" }, " \u6B66\u5C06\u3001\u6280\u80FD\u6E90\u7801\u3001\u9759\u6001\u76AE\u80A4\u548C\u52A8\u6001\u76AE\u80A4\u5747\u4ECE\u672C\u5730\u539F\u4ED3\u5E93\u76EE\u5F55\u8BFB\u53D6\uFF1B\u65E7 JavaScript \u4E0D\u4F1A\u5728\u5BA2\u6237\u7AEF\u6267\u884C\u3002 ", -1)), n.value ? (h(), g("p", r_, f(n.value.counts.packs) + " \u5305 \xB7 " + f(n.value.counts.characters) + " \u6B66\u5C06 \xB7 " + f(n.value.counts.skills) + " \u6280\u80FD \xB7 " + f(n.value.counts.static_skins) + " \u9759\u6001\u76AE\u80A4 \xB7 " + f(n.value.counts.dynamic_skins) + " \u52A8\u6001\u76AE\u80A4 \xB7 " + f(y.value) + " \u97F3\u9891\u8D44\u6E90 ", 1)) : S("v-if", true), l.value ? (h(), g("p", l_, "\u6B63\u5728\u52A0\u8F7D\u76EE\u5F55\u2026")) : S("v-if", true), s.value ? (h(), g("p", u_, f(s.value), 1)) : S("v-if", true)]), n.value ? (h(), g("section", c_, [o("aside", d_, [o("div", m_, [Z(o("input", { "onUpdate:modelValue": M[0] || (M[0] = (C) => r.value = C), type: "search", placeholder: "\u641C\u7D22\u6B66\u5C06", "aria-label": "\u641C\u7D22\u6B66\u5C06" }, null, 512), [[Re, r.value]]), Z(o("select", { "onUpdate:modelValue": M[1] || (M[1] = (C) => u.value = C), "aria-label": "\u6B66\u5C06\u5305" }, [M[4] || (M[4] = o("option", { value: "" }, "\u5168\u90E8\u6B66\u5C06\u5305", -1)), (h(true), g(Q, null, Y(n.value.packages, (C) => (h(), g("option", { key: C, value: C }, f(C), 9, p_))), 128))], 512), [[_e, u.value]])]), (h(true), g(Q, null, Y(p.value, (C) => {
    var _a2;
    return h(), g("button", { key: C.id, type: "button", class: oe(["character-row", { selected: ((_a2 = c.value) == null ? void 0 : _a2.id) === C.id }]), onClick: (re) => c.value = C }, [C.portrait ? (h(), g("img", { key: 0, src: w(C.portrait), alt: "", loading: "lazy" }, null, 8, g_)) : S("v-if", true), o("span", null, [o("strong", null, f(C.display_name), 1), o("small", null, f(C.pack) + " \xB7 " + f(C.name), 1)])], 10, h_);
  }), 128)), p.value.length === 80 ? (h(), g("p", f_, "\u4EC5\u663E\u793A\u524D 80 \u9879\uFF0C\u8BF7\u7EE7\u7EED\u7B5B\u9009\u3002")) : S("v-if", true)]), c.value ? (h(), g("article", __, [o("header", null, [c.value.portrait ? (h(), g("img", { key: 0, src: w(c.value.portrait), alt: `${c.value.display_name}\u6B66\u5C06\u5934\u50CF` }, null, 8, v_)) : S("v-if", true), o("div", null, [o("h2", null, f(c.value.display_name), 1), o("span", { class: oe(["character-status", c.value.migration_status]), title: c.value.migration_status === "ported" ? "\u5B8C\u6574\u6B66\u5C06\u5B9A\u4E49\u5DF2\u63A5\u5165\u786E\u5B9A\u6027\u5185\u5BB9\u5305" : "\u4EC5\u540C\u6B65\u76EE\u5F55\u3001\u6E90\u7801\u4E0E\u8D44\u6E90\uFF0C\u5C1A\u4E0D\u80FD\u8FDB\u5165\u786E\u5B9A\u6027\u5BF9\u5C40" }, f(c.value.migration_status === "ported" ? "\u6B66\u5C06\u5305\u5DF2\u63A5\u5165" : "\u5F85\u5B8C\u6574\u79FB\u690D"), 11, y_), o("p", k_, f(c.value.pack) + " \xB7 " + f(c.value.name) + " \xB7 " + f(c.value.group ?? "\u672A\u77E5\u52BF\u529B") + " \xB7 " + f(c.value.hp) + " \u4F53\u529B", 1)])]), M[5] || (M[5] = o("h3", null, "\u6280\u80FD", -1)), d.value.length ? (h(), g("ul", w_, [(h(true), g(Q, null, Y(d.value, (C) => (h(), g("li", { key: C.id }, [o("strong", null, f(C.display_name), 1), o("span", { class: oe(C.migration_status), title: C.migration_status === "ported" ? "\u5DF2\u63A5\u5165\u786E\u5B9A\u6027\u5185\u5BB9\u5305\uFF1B\u662F\u5426\u4E0E\u539F\u6280\u80FD\u5B8C\u5168\u7B49\u4EF7\u4ECD\u4EE5 Golden \u4E3A\u51C6" : "\u4EC5\u4FDD\u7559\u539F\u6E90\u7801\u4E0E\u76EE\u5F55\uFF0C\u4E0D\u80FD\u8FDB\u5165\u786E\u5B9A\u6027\u5BF9\u5C40" }, f(C.migration_status === "ported" ? "VM \u5DF2\u63A5\u5165" : "\u5F85 DSL \u91CD\u5199"), 11, b_), o("p", null, f(C.description ?? C.name), 1)]))), 128))])) : (h(), g("p", x_, "\u6CA1\u6709\u53EF\u89E3\u6790\u7684\u6B66\u5C06\u5305\u6280\u80FD\u3002")), o("h3", null, "\u8BED\u97F3\u8BD5\u542C\uFF08" + f(V.value.length) + "\uFF09", 1), t.value ? (h(), g("p", $_, f(t.value), 1)) : S("v-if", true), V.value.length ? (h(), g("ul", j_, [(h(true), g(Q, null, Y(V.value, (C) => (h(), g("li", { key: C.path }, [o("label", null, [o("span", null, f(C.source_path.startsWith("audio/die/") ? "\u9635\u4EA1" : C.source_path.slice(12, -4)), 1), o("audio", { controls: "", preload: "none", src: w(`assets/${C.path}`), "aria-label": `${c.value.display_name} \xB7 ${C.source_path}`, onError: (re) => i.value.add(C.path), onLoadeddata: (re) => i.value.delete(C.path) }, null, 40, S_)]), i.value.has(C.path) ? (h(), g("div", E_, "\u8BED\u97F3\u52A0\u8F7D\u6216\u89E3\u7801\u5931\u8D25\uFF0C\u8BF7\u68C0\u67E5\u672C\u5730\u8D44\u6E90\u3002")) : S("v-if", true)]))), 128))])) : (h(), g("p", z_, "\u5F53\u524D\u8D44\u6E90\u6E05\u5355\u672A\u63D0\u4F9B\u8BE5\u6B66\u5C06\u5DF2\u58F0\u660E\u4E14\u6E90\u6587\u4EF6\u53EF\u7528\u7684\u6280\u80FD\u6216\u9635\u4EA1\u8BED\u97F3\u3002")), o("h3", null, "\u9759\u6001\u76AE\u80A4\uFF08" + f(v.value.length) + "\uFF09", 1), v.value.length ? (h(), g("div", A_, [(h(true), g(Q, null, Y(v.value, (C) => (h(), g("figure", { key: C.id }, [o("img", { src: w(C.asset), alt: `${c.value.display_name} \xB7 ${C.name}`, loading: "lazy" }, null, 8, C_), o("figcaption", null, f(C.name), 1)]))), 128))])) : (h(), g("p", R_, "\u539F\u4ED3\u5E93\u672A\u63D0\u4F9B\u9759\u6001\u76AE\u80A4\u3002")), o("h3", null, "\u52A8\u6001\u76AE\u80A4\uFF08" + f(P.value.length) + "\uFF09", 1), o("p", N_, f(P.value.map((C) => C.name).join("\u3001") || "\u539F\u4ED3\u5E93\u672A\u63D0\u4F9B\u52A8\u6001\u76AE\u80A4\u3002"), 1)])) : S("v-if", true)])) : S("v-if", true)]));
} }), q_ = ge(P_, [["__scopeId", "data-v-fcfc63fc"]]), I_ = { class: "stack" }, O_ = { class: "panel" }, T_ = { class: "muted" }, L_ = { class: "row" }, D_ = { key: 0, class: "ok", role: "status" }, M_ = { key: 1, class: "error", role: "alert" }, U_ = { key: 2, class: "warn", role: "status" }, V_ = { class: "panel" }, G_ = { class: "row" }, B_ = ["disabled"], F_ = ["disabled"], H_ = { class: "panel" }, W_ = { class: "muted mono" }, K_ = { key: 0, class: "lock-list" }, J_ = { key: 1, class: "muted" }, Q_ = { class: "panel" }, Y_ = { class: "table", role: "table" }, X_ = { key: 0, class: "badge-unsigned", title: "\u672C\u5730\u5BFC\u5165\u7684\u5305\u672A\u7ECF\u7B7E\u540D\u6821\u9A8C" }, Z_ = { class: "mono" }, e1 = { class: "mono" }, n1 = { class: "actions" }, a1 = ["disabled", "onClick"], t1 = ["disabled", "onClick"], i1 = ["disabled", "onClick"], s1 = ["disabled", "title", "onClick"], o1 = ["disabled", "onClick"], r1 = ["disabled", "onClick"], l1 = ["disabled", "onClick"], u1 = se({ __name: "ExtensionsView", setup(e) {
  const n = I($n()), a = I(""), t = I(""), i = I(""), s = I(false);
  let l = null;
  sa(() => {
    l == null ? void 0 : l.then((y) => y.dispose(), () => {
    });
  });
  function r() {
    return l ?? (l = (async () => {
      const y = new oa();
      return await y.connect(), y;
    })().catch((y) => {
      throw l = null, y;
    })), l;
  }
  async function u(y) {
    await (await r()).validatePackage(y);
  }
  function c() {
    n.value = $n(), a.value = "";
  }
  function _(y) {
    a.value = "", t.value = "";
    try {
      y(), c();
    } catch (k) {
      a.value = k instanceof Error ? k.message : String(k);
    }
  }
  async function m() {
    if (!s.value) {
      a.value = "", t.value = "", s.value = true;
      try {
        const y = i.value;
        await Cm(y, u);
        const k = JSON.parse(y).manifest;
        t.value = `\u5DF2\u5BFC\u5165 ${k.namespace}/${k.package_id} @ ${k.version}\uFF08\u672A\u7B7E\u540D\uFF09`, i.value = "", c();
      } catch (y) {
        a.value = y instanceof Error ? y.message : String(y);
      } finally {
        s.value = false;
      }
    }
  }
  async function p(y) {
    var _a2;
    const k = y.target, w = (_a2 = k.files) == null ? void 0 : _a2[0];
    if (w) try {
      i.value = await w.text(), await m();
    } catch (D) {
      a.value = D instanceof Error ? D.message : String(D);
    } finally {
      k.value = "";
    }
  }
  function d(y) {
    return n.value.installed.find((k) => k.key === y);
  }
  function v(y) {
    const k = d(y);
    return k ? k.lifecycle === "active" ? "\u5DF2\u542F\u7528" : k.lifecycle === "disabled" ? "\u5DF2\u505C\u7528" : k.lifecycle === "revoked" ? "\u5DF2\u64A4\u9500" : "\u5DF2\u5B89\u88C5" : "\u672A\u5B89\u88C5";
  }
  const P = L(() => Lm(n.value)), V = L(() => {
    const y = dn.map((D) => {
      const A = ca(D), M = d(A);
      return { title: D.title, version: D.version, key: A, record: M, status: v(A), imported: false };
    }), k = new Set(y.map((D) => D.key)), w = n.value.installed.filter((D) => !k.has(D.key)).map((D) => ({ title: `${D.packageId}\uFF08\u5BFC\u5165\uFF09`, version: D.version, key: D.key, record: D, status: v(D.key), imported: true }));
    return [...y, ...w];
  });
  return (y, k) => (h(), g("div", I_, [o("section", O_, [k[2] || (k[2] = o("h1", null, "\u6269\u5C55\u5305\u7BA1\u7406", -1)), o("p", T_, " \u5BF9\u9F50\u5BBF\u4E3B\u5951\u7EA6\uFF1A\u5B89\u88C5 / \u542F\u7528 / \u505C\u7528 / \u5347\u7EA7 / \u56DE\u9000 / \u5378\u8F7D / \u64A4\u9500\u3002\u5F53\u524D\u76EE\u5F55 " + f(z(dn).length) + " \u4E2A G1 \u8F6C\u6362\u5305\u3002\u5F00\u5C40\u5185\u5BB9\u9501\u7531\u542F\u7528\u96C6\u6784\u6210\uFF1B\u5BF9\u5C40\u4E2D\u51BB\u7ED3\u53D8\u66F4\u3002 \u5B89\u88C5\u65F6\u4FDD\u5B58\u5B9E\u9645\u5305\u5185\u5BB9\uFF0C\u672C\u5730\u539F\u751F\u5BF9\u5C40\u76F4\u63A5\u88C5\u8F7D\uFF1B\u5B58\u6863\u4FDD\u7559\u539F\u5C40\u5305\u526F\u672C\u3002 ", 1), o("div", L_, [o("button", { class: "primary", type: "button", onClick: k[0] || (k[0] = (w) => {
    _(() => {
      z(Dm)(), t.value = `\u5DF2\u5B89\u88C5\u5E76\u542F\u7528\u5168\u90E8 ${z(dn).length} \u5305`;
    });
  }) }, " \u5168\u90E8\u5B89\u88C5\u5E76\u542F\u7528 "), o("button", { type: "button", onClick: c }, "\u5237\u65B0")]), t.value ? (h(), g("p", D_, f(t.value), 1)) : S("v-if", true), a.value ? (h(), g("p", M_, f(a.value), 1)) : S("v-if", true), n.value.matchFrozen ? (h(), g("p", U_, "\u5BF9\u5C40\u8FDB\u884C\u4E2D \xB7 \u5185\u5BB9\u9501\u5DF2\u51BB\u7ED3")) : S("v-if", true)]), o("section", V_, [k[4] || (k[4] = o("h2", null, "\u5BFC\u5165\u6269\u5C55\u5305", -1)), k[5] || (k[5] = o("p", { class: "muted" }, " \u6B63\u5F0F\u5BFC\u5165\u5165\u53E3\uFF1A\u7C98\u8D34\u6216\u9009\u62E9\u4F5C\u8005\u4EA7\u7269\uFF08\u5B8C\u6574 npack JSON\uFF0C\u53EF\u542B VM \u6A21\u5757\u4E0E\u5361\u724C/\u6B66\u5C06/\u724C\u5806\u8D44\u4EA7\uFF09\u3002 \u540C\u5305\u65B0\u7248\u672C\u6309\u5347\u7EA7\u5904\u7406\u5E76\u4FDD\u7559\u65E7\u7248\u672C\u526F\u672C\uFF1B\u540C\u7248\u672C\u5185\u5BB9\u51B2\u7A81\u3001\u5DF2\u64A4\u9500\u3001\u5BF9\u5C40\u51BB\u7ED3\u3001\u7ED3\u6784\u6821\u9A8C\u6216 Rust \u6DF1\u5C42\u6821\u9A8C\uFF08\u5305/\u6A21\u5757\u6307\u7EB9\u3001\u5B57\u8282\u7801\u3001\u5BBF\u4E3B ABI \u7A97\u53E3\uFF09\u5931\u8D25\u65F6\u62D2\u7EDD\u4E14\u4E0D\u6539\u5199\u5DF2\u5B89\u88C5\u5185\u5BB9\u3002 \u672C\u5730\u5BFC\u5165\u7684\u5305\u672A\u7ECF\u7B7E\u540D\u6821\u9A8C\uFF0C\u5217\u8868\u4E2D\u6807\u4E3A\u300C\u672A\u7B7E\u540D\u300D\u3002 ", -1)), Z(o("textarea", { "onUpdate:modelValue": k[1] || (k[1] = (w) => i.value = w), class: "import-box", rows: "4", spellcheck: "false", "aria-label": "\u6269\u5C55\u5305 JSON", placeholder: '{"manifest": {...}, "modules": [...], ...}' }, null, 512), [[Re, i.value]]), o("div", G_, [o("button", { class: "primary", type: "button", disabled: !i.value.trim() || n.value.matchFrozen || s.value, onClick: m }, f(s.value ? "\u6821\u9A8C\u4E2D\u2026" : "\u5BFC\u5165"), 9, B_), o("label", null, [k[3] || (k[3] = W(" \u9009\u62E9\u6587\u4EF6 ", -1)), o("input", { type: "file", accept: ".json,application/json", "aria-label": "\u9009\u62E9\u6269\u5C55\u5305\u6587\u4EF6", disabled: n.value.matchFrozen, onChange: p }, null, 40, F_)])])]), o("section", H_, [k[6] || (k[6] = o("h2", null, "\u5185\u5BB9\u9501", -1)), o("p", W_, f(P.value.hashSeed || "\uFF08\u7A7A\uFF09"), 1), P.value.packages.length ? (h(), g("ul", K_, [(h(true), g(Q, null, Y(P.value.packages, (w) => (h(), g("li", { key: w.key }, f(w.key) + " @ " + f(w.version) + " \xB7 " + f(w.fingerprint), 1))), 128))])) : (h(), g("p", J_, "\u5C1A\u672A\u542F\u7528\u4EFB\u4F55\u6269\u5C55\u3002"))]), o("section", Q_, [k[8] || (k[8] = o("h2", null, "\u5305\u5217\u8868", -1)), o("div", Y_, [k[7] || (k[7] = o("div", { class: "thead", role: "row" }, [o("span", null, "\u540D\u79F0"), o("span", null, "\u6807\u8BC6"), o("span", null, "\u7248\u672C"), o("span", null, "\u72B6\u6001"), o("span", null, "\u64CD\u4F5C")], -1)), (h(true), g(Q, null, Y(V.value, (w) => {
    var _a2, _b, _c2;
    return h(), g("div", { key: w.key, class: "trow", role: "row" }, [o("span", null, [W(f(w.title) + " ", 1), ((_a2 = w.record) == null ? void 0 : _a2.signature) === "unsigned" ? (h(), g("span", X_, "\u672A\u7B7E\u540D")) : S("v-if", true)]), o("span", Z_, f(w.key), 1), o("span", e1, f(((_b = w.record) == null ? void 0 : _b.version) ?? w.version), 1), o("span", null, f(w.status), 1), o("span", n1, [o("button", { type: "button", disabled: w.imported || !!w.record && w.record.lifecycle !== "revoked", onClick: (D) => _(() => z(Ft)(w.key)) }, " \u5B89\u88C5 ", 8, a1), o("button", { type: "button", disabled: !w.record || w.record.lifecycle === "active" || w.record.lifecycle === "revoked", onClick: (D) => _(() => z(Wt)(w.key)) }, " \u542F\u7528 ", 8, t1), o("button", { type: "button", disabled: !w.record || w.record.lifecycle !== "active", onClick: (D) => _(() => z(Pm)(w.key)) }, " \u505C\u7528 ", 8, i1), o("button", { type: "button", disabled: w.imported || !w.record || w.record.lifecycle === "revoked", title: w.imported ? "\u5BFC\u5165\u5305\u8BF7\u5BFC\u5165\u65B0\u7248\u672C\u5305\u4EE5\u5347\u7EA7" : "", onClick: (D) => _(() => z(qm)(w.key)) }, " \u5347\u7EA7 ", 8, s1), o("button", { type: "button", disabled: !w.record || (((_c2 = w.record.history) == null ? void 0 : _c2.length) ?? 0) < 2, onClick: (D) => {
      _(() => {
        const A = w.record.history.filter((M) => M !== w.record.version);
        A.length && z(Im)(w.key, A[A.length - 1]);
      });
    } }, " \u56DE\u9000 ", 8, o1), o("button", { type: "button", disabled: !w.record, onClick: (D) => _(() => z(Om)(w.key)) }, " \u5378\u8F7D ", 8, r1), o("button", { type: "button", disabled: !w.record || w.record.lifecycle === "revoked", onClick: (D) => _(() => z(Tm)(w.key)) }, " \u64A4\u9500 ", 8, l1)])]);
  }), 128))])])]));
} }), c1 = ge(u1, [["__scopeId", "data-v-2bd63404"]]), ni = io({ history: ia() ? Ls() : Nt(), routes: [{ path: "/", name: "home", component: jo }, { path: "/battle", name: "battle", component: cf }, { path: "/extensions", name: "extensions", component: c1 }, { path: "/settings", name: "settings", component: Pf }, { path: "/replay", name: "replay", component: t_ }, { path: "/collection", name: "collection", component: q_ }], scrollBehavior: () => ({ top: 0 }) });
function d1() {
  const e = sessionStorage.getItem("noname-next.online-intent");
  if (e) try {
    const n = JSON.parse(e);
    if (n.action === "spectate") return;
    ni.push({ path: "/battle", query: { mode: "online", action: n.action === "create" ? "join" : n.action ?? "join", room: String(n.roomId ?? 1), players: String(n.players ?? 4), seed: String(n.seed ?? 42) } });
  } catch {
    sessionStorage.removeItem("noname-next.online-intent");
  }
}
const m1 = Xi(co).use(ni);
m1.mount("#app");
d1();
