// VBA runtime for TraceDraw: the helpers compiled VBA calls (`__v`) and
// the VBA library (`__b`). Evaluated after the host's object model and
// before the compiled program.
var __v, __b;
(function () {
"use strict";
const G = globalThis;
const V = {};
const B = {};

// ----- Errors -------------------------------------------------------------

const MSG = {
  3: "Return without GoSub", 5: "Invalid procedure call or argument", 6: "Overflow",
  7: "Out of memory", 9: "Subscript out of range",
  10: "This array is fixed or temporarily locked", 11: "Division by zero", 13: "Type mismatch",
  28: "Out of stack space", 35: "Sub or Function not defined",
  51: "Internal error", 91: "Object variable or With block variable not set",
  92: "For loop not initialized", 94: "Invalid use of Null", 424: "Object required",
  429: "ActiveX component can't create object",
  438: "Object doesn't support this property or method", 440: "Automation error",
  445: "Object doesn't support this action", 448: "Named argument not found",
  449: "Argument not optional", 450: "Wrong number of arguments or invalid property assignment",
  453: "Specified DLL function not found",
  457: "This key is already associated with an element of this collection",
  32811: "Element not found",
};
function msgOf(n) { return MSG[n] || "Application-defined or object-defined error"; }

class VbError extends Error {
  constructor(n, desc, src) {
    super(desc);
    this.vbNumber = n;
    this.vbDescription = desc;
    this.vbSource = src || "";
  }
}
V.VbError = VbError;
V.END = { vbEnd: true };

V.raise = function (n, detail) {
  let d = msgOf(n);
  if (detail !== undefined && detail !== "") d += ": " + detail;
  throw new VbError(n, d);
};

V.unsupported = function (what) {
  throw new VbError(445, "Not supported in TraceDraw: " + what);
};

V.toVbError = function (e) {
  if (e instanceof VbError) return e;
  if (e === V.END) return e;
  if (e && typeof e === "object") {
    let n = 5;
    const msg = String(e.message !== undefined ? e.message : e);
    if (e instanceof RangeError && /stack|recursion/i.test(msg)) n = 28;
    const ve = new VbError(n, msg);
    if (e.vbLoc) ve.vbLoc = e.vbLoc;
    return ve;
  }
  return new VbError(5, String(e));
};

// The Err object.
V.Err = {
  Number: 0, Description: "", Source: "", HelpFile: "", HelpContext: 0, LastDllError: 0,
  Raise(number, source, description) {
    const n = V.clng(number);
    const d = miss(description) ? msgOf(n) : V.cstr(description);
    throw new VbError(n, d, miss(source) ? "" : V.cstr(source));
  },
  Clear() { this.Number = 0; this.Description = ""; this.Source = ""; },
};
V.Err.Raise.__params = ["number", "source", "description", "helpfile", "helpcontext"];

V.seterr = function (e) {
  if (e === V.END) return;
  const ve = V.toVbError(e);
  V.Err.Number = ve.vbNumber;
  V.Err.Description = ve.vbDescription;
  V.Err.Source = ve.vbSource || (ve.vbLoc ? ve.vbLoc.m : "");
};

// ----- Frames: where the program is, for error messages ------------------

V.stack = [];
V.enter = function (m, p, ct, self) {
  const f = { m: m, p: p, l: 0, ct: ct };
  V.stack.push(f);
  return f;
};
V.leave = function (f) {
  const i = V.stack.lastIndexOf(f);
  if (i >= 0) V.stack.length = i;
};
V.locate = function (e, f) {
  if (e === V.END) return e;
  const ve = V.toVbError(e);
  if (!ve.vbLoc) ve.vbLoc = { m: f.m, p: f.p, l: f.l };
  return ve;
};
function textCompare() {
  const f = V.stack[V.stack.length - 1];
  return f ? f.ct === 1 : false;
}

V.describe = function (e) {
  let s = "Run-time error '" + e.vbNumber + "': " + e.vbDescription;
  if (e.vbLoc) s += " (" + e.vbLoc.m + "." + e.vbLoc.p + ", line " + e.vbLoc.l + ")";
  return s;
};

// Run a macro: errors become one readable Error; End stops quietly.
V.run = function (fn) {
  V.stack = [];
  V.Err.Clear();
  try {
    return fn();
  } catch (e) {
    if (e === V.END) return undefined;
    const ve = V.toVbError(e);
    const err = new Error(V.describe(ve));
    err.vbNumber = ve.vbNumber;
    err.vbLoc = ve.vbLoc;
    throw err;
  } finally {
    V.stack = [];
    V.flush();
  }
};

// ----- Special values -----------------------------------------------------

V.Missing = Object.freeze({ vbMissing: true });
V.Null = Object.freeze({ vbNull: true });
function miss(x) { return x === undefined || x === V.Missing; }
V.miss = miss;

class VDate {
  constructor(v) { this.v = v; }
  valueOf() { return this.v; }
  toString() { return V.cstr(this); }
}
V.VDate = VDate;
V.mkdate = function (v) { return new VDate(v); };

// ----- References (ByRef arguments) --------------------------------------

class Ref {
  constructor(g, s) { this.g = g; this.s = s; }
}
V.ref = function (g, s) { return new Ref(g, s); };
V.refIdx = function (o, args) {
  return new Ref(() => V.index(o, args), (x) => V.setIdx(o, args, x));
};
V.deref = function (x) { return x instanceof Ref ? x.g() : x; };
V.wb = function (x, v) { if (x instanceof Ref) x.s(v); };

// ----- Arrays ---------------------------------------------------------------

class VArray {
  constructor(dims, def, co, fixed) {
    this.def = def || (() => undefined);
    this.co = co || null;
    this.fixed = !!fixed;
    this.setDims(dims);
  }
  setDims(dims) {
    this.dims = dims;
    if (!dims) { this.data = null; return; }
    let n = 1;
    for (const d of dims) {
      const c = d[1] - d[0] + 1;
      if (c < 0) V.raise(9);
      n *= c;
    }
    this.data = new Array(n);
    for (let i = 0; i < n; i++) this.data[i] = this.def();
  }
  off(args) {
    if (!this.data) V.raise(9);
    if (args.length !== this.dims.length) V.raise(9);
    let o = 0, stride = 1;
    for (let k = 0; k < args.length; k++) {
      const i = V.clng(args[k]);
      const lo = this.dims[k][0], hi = this.dims[k][1];
      if (i < lo || i > hi) V.raise(9);
      o += (i - lo) * stride;
      stride *= hi - lo + 1;
    }
    return o;
  }
  get(args) { return this.data[this.off(args)]; }
  set(args, v) { this.data[this.off(args)] = this.co ? this.co(v) : V.letv(v); }
  items() {
    if (!this.data) V.raise(92);
    return this.data.slice();
  }
  clone() {
    const c = new VArray(null, this.def, this.co, this.fixed);
    c.dims = this.dims ? this.dims.map((d) => d.slice()) : null;
    c.data = this.data ? this.data.map((x) => V.letv(x)) : null;
    return c;
  }
}
V.VArray = VArray;

// An array of values with lower bound `base`.
V.arr = function (base, items) {
  const a = new VArray([[base, base + items.length - 1]], null, null, false);
  for (let i = 0; i < items.length; i++) a.data[i] = items[i];
  return a;
};
V.parr = function (items) {
  return V.arr(0, items.map((x) => V.deref(x)));
};

V.redim = function (arr, dims, preserve, def, co) {
  if (arr instanceof VArray) {
    if (arr.fixed) V.raise(10);
    if (preserve && arr.data) {
      const old = arr.clone();
      if (old.dims.length !== dims.length) V.raise(9);
      for (let k = 0; k < dims.length - 1; k++) {
        if (old.dims[k][0] !== dims[k][0] || old.dims[k][1] !== dims[k][1]) V.raise(9);
      }
      arr.setDims(dims);
      // Copy every element whose indexes exist in both.
      const idx = old.dims.map((d) => d[0]);
      const total = old.data.length;
      for (let n = 0; n < total; n++) {
        let fits = true;
        for (let k = 0; k < dims.length; k++) {
          if (idx[k] < dims[k][0] || idx[k] > dims[k][1]) fits = false;
        }
        if (fits) arr.data[arr.off(idx)] = old.data[n];
        for (let k = 0; k < idx.length; k++) {
          idx[k]++;
          if (idx[k] <= old.dims[k][1]) break;
          idx[k] = old.dims[k][0];
        }
      }
      return arr;
    }
    if (def) arr.def = def;
    if (co !== undefined && co !== null) arr.co = co;
    arr.setDims(dims);
    return arr;
  }
  return new VArray(dims, def, co, false);
};

V.erase = function (arr) {
  if (!(arr instanceof VArray)) V.raise(13);
  if (arr.fixed) { arr.setDims(arr.dims); return arr; }
  arr.setDims(null);
  return arr;
};

// ----- Objects of the program: classes, user types and forms -------------

class VbObject {
  constructor(cls) {
    Object.defineProperty(this, "__cls", { value: cls, writable: true, enumerable: false });
  }
  __init() {
    if (typeof this.$class_initialize === "function") this.$class_initialize();
  }
  __ctl(l) { return undefined; }
  __get(name) {
    const l = name.toLowerCase();
    const g = this["__get_" + l];
    if (typeof g === "function") return g.call(this);
    if (("$" + l) in this) {
      const v = this["$" + l];
      return typeof v === "function" ? v.call(this) : v;
    }
    const c = this.__ctl(l);
    if (c !== undefined) return c;
    return this.__getx(name);
  }
  __getx(name) { V.raise(438, name); }
  __call(name, args, named) {
    const l = name.toLowerCase();
    const f = this["$" + l];
    if (typeof f === "function") return f.apply(this, V.named(f, args, named));
    const g = this["__get_" + l];
    if (typeof g === "function") {
      if (args.length > 0 && g.__params && g.__params.length > 0) {
        return g.apply(this, V.named(g, args, named));
      }
      const v = g.call(this);
      return args.length ? V.index(v, args) : v;
    }
    if (("$" + l) in this) {
      const v = this["$" + l];
      return args.length ? V.index(v, args) : v;
    }
    const c = this.__ctl(l);
    if (c !== undefined) return args.length ? V.index(c, args) : c;
    return this.__callx(name, args, named);
  }
  __callx(name, args, named) { V.raise(438, name); }
  __set(name, v, isSet) {
    const l = name.toLowerCase();
    let f = this[(isSet ? "__set_" : "__let_") + l];
    if (typeof f !== "function") f = this["__let_" + l] || this["__set_" + l];
    if (typeof f === "function") { f.call(this, v); return; }
    if (("$" + l) in this && typeof this["$" + l] !== "function") {
      this["$" + l] = isSet ? v : V.letv(v);
      return;
    }
    this.__setx(name, v, isSet);
  }
  __setx(name, v, isSet) { V.raise(438, name); }
  __index(args) { return this.__call("Item", args, null); }
  __setIdx(args, v) {
    const f = this.__let_item || this.__set_item;
    if (typeof f === "function") return f.apply(this, args.concat([v]));
    V.raise(438, "Item");
  }
  __each() {
    const f = this.$_newenum || this.__get__newenum;
    if (typeof f === "function") return V.each(f.call(this));
    V.raise(438, "For Each");
  }
}
V.VbObject = VbObject;

class VbRecord extends VbObject {
  __init() {}
  clone() {
    const c = new this.constructor();
    for (const k of Object.keys(this)) if (k[0] === "$") c[k] = V.letv(this[k]);
    return c;
  }
}
V.VbRecord = VbRecord;

V.classes = {};
V.forms = {};
const formInstances = {};

class VbForm extends VbObject {
  constructor(cls) {
    super(cls);
    Object.defineProperty(this, "__controls", { value: {}, writable: true, enumerable: false });
    Object.defineProperty(this, "__props", { value: { caption: cls }, writable: true, enumerable: false });
    if (typeof G.__host_form_create === "function") G.__host_form_create(this);
  }
  __init() {
    if (typeof this.$userform_initialize === "function") this.$userform_initialize();
  }
  __ctl(l) {
    const c = this.__controls[l];
    if (c !== undefined) return c;
    for (const k of Object.keys(this.__controls)) if (k.toLowerCase() === l) return this.__controls[k];
    return undefined;
  }
  __getx(name) {
    const l = name.toLowerCase();
    if (l in this.__props) return this.__props[l];
    if (typeof G.__host_form_get === "function") return G.__host_form_get(this, name);
    V.raise(438, name);
  }
  __callx(name, args, named) {
    const v = this.__getx(name);
    return args.length ? V.index(v, args) : v;
  }
  __setx(name, v) {
    this.__props[name.toLowerCase()] = v;
    if (typeof G.__host_form_set === "function") G.__host_form_set(this, name, v);
  }
  $show(modal) {
    if (typeof G.__host_form_show !== "function") V.unsupported("showing a form here");
    G.__host_form_show(this, miss(modal) ? true : V.cbool(modal));
  }
  $hide() {
    if (typeof G.__host_form_hide === "function") G.__host_form_hide(this);
  }
  $repaint() {}
  // The host calls this when a control raises an event.
  __event(control, event, args) {
    const f = this["$" + String(control).toLowerCase() + "_" + String(event).toLowerCase()];
    if (typeof f === "function") return f.apply(this, args || []);
    return undefined;
  }
}
V.VbForm = VbForm;

// The default instance of a form, made on first use.
V.form = function (name) {
  const l = name.toLowerCase();
  if (!formInstances[l]) {
    const C = V.forms[l];
    if (!C) V.raise(424, name);
    formInstances[l] = new C();
  }
  return formInstances[l];
};
V.unload = function (f) {
  if (!(f instanceof VbForm)) V.raise(13);
  if (typeof f.$userform_terminate === "function") f.$userform_terminate();
  if (typeof G.__host_form_unload === "function") G.__host_form_unload(f);
  for (const k of Object.keys(formInstances)) if (formInstances[k] === f) delete formInstances[k];
};
V.load = function (f) { return f; };

// A name inside a form's code: one of its controls, else the host's.
V.fid = function (self, name, host) {
  if (self instanceof VbForm) {
    const c = self.__ctl(name.toLowerCase());
    if (c !== undefined) return c;
  }
  return host();
};

V.newHost = function (C, name) {
  if (typeof C === "function") return new C();
  V.raise(429, name);
};

V.typeOf = function (x, t) {
  if (x === null || x === undefined || typeof x !== "object") return false;
  const key = t.split(".").pop().toLowerCase();
  if (key === "object") return true;
  if (x instanceof VbObject) return String(x.__cls).toLowerCase() === key;
  if (x instanceof Collection) return key === "collection";
  if (x instanceof Dictionary) return key === "dictionary";
  let p = Object.getPrototypeOf(x);
  while (p && p !== Object.prototype) {
    if (p.constructor && String(p.constructor.name).toLowerCase() === key) return true;
    p = Object.getPrototypeOf(p);
  }
  return false;
};

// ----- Member access on any object ----------------------------------------

const keyCache = new Map();
function protoKeys(p) {
  let m = keyCache.get(p);
  if (m) return m;
  m = new Map();
  let q = p;
  while (q && q !== Object.prototype && q !== Function.prototype) {
    for (const k of Object.getOwnPropertyNames(q)) {
      const l = k.toLowerCase();
      if (!m.has(l)) m.set(l, k);
    }
    q = Object.getPrototypeOf(q);
  }
  keyCache.set(p, m);
  return m;
}
// The real name of a member, matched without regard to case.
function findKey(o, name) {
  if (name in o) return name;
  const l = name.toLowerCase();
  for (const k of Object.getOwnPropertyNames(o)) if (k.toLowerCase() === l) return k;
  const p = Object.getPrototypeOf(o);
  if (p) {
    const k = protoKeys(p).get(l);
    if (k !== undefined) return k;
  }
  return null;
}
V.findKey = findKey;

function needObject(o) {
  if (o === null) V.raise(91);
  if (o === undefined || (typeof o !== "object" && typeof o !== "function")) V.raise(424);
  if (o === V.Null || o === V.Missing) V.raise(424);
}

// The names of a function's parameters (`__params`, else read from its
// source where the engine keeps it), for named arguments.
const paramCache = new Map();
function paramNames(f) {
  if (f.__params) return f.__params;
  let names = paramCache.get(f);
  if (names) return names;
  names = [];
  try {
    const src = Function.prototype.toString.call(f);
    const m = /^[^(]*\(([^)]*)\)/.exec(src);
    if (m) {
      names = m[1].split(",").map((s) => s.replace(/=.*$/, "").replace(/[\s.]/g, "").toLowerCase()).filter((s) => s);
    }
  } catch (e) {}
  paramCache.set(f, names);
  return names;
}

// Positional and named arguments in the order of the parameters.
V.named = function (f, args, named) {
  const out = args.slice();
  if (named) {
    const names = paramNames(f);
    for (const k of Object.keys(named)) {
      const i = names.indexOf(k.toLowerCase());
      if (i < 0) V.raise(448, k);
      while (out.length <= i) out.push(V.Missing);
      out[i] = named[k];
    }
  }
  return out;
};

// Arguments for a host function: missing ones are undefined.
function hostArgs(f, args, named) {
  const a = V.named(f, args, named);
  for (let i = 0; i < a.length; i++) if (a[i] === V.Missing) a[i] = undefined;
  while (a.length && a[a.length - 1] === undefined) a.pop();
  return a;
}

V.get = function (o, name) {
  needObject(o);
  if (o instanceof VbObject) return o.__get(name);
  if (o === V.Err) {
    const k = findKey(o, name);
    if (k === null) V.raise(438, name);
    const v = o[k];
    return typeof v === "function" ? v.call(o) : v;
  }
  if (Array.isArray(o) && name.toLowerCase() === "count") return o.length;
  const k = findKey(o, name);
  if (k === null) V.raise(438, name);
  const v = o[k];
  if (typeof v === "function") return callOrClass(v, o);
  return v;
};

// A method read without arguments is called; a class held as a value
// (which cannot be called without `new`) is returned as it is.
function callOrClass(f, self) {
  try {
    return f.call(self);
  } catch (e) {
    if (e instanceof TypeError && /constructor|without 'new'|new/i.test(String(e.message))) return f;
    throw e;
  }
}

V.call = function (o, name, args, named) {
  needObject(o);
  if (o instanceof VbObject) return o.__call(name, args, named);
  if (Array.isArray(o) && name.toLowerCase() === "count" && !args.length) return o.length;
  const k = findKey(o, name);
  if (k === null) V.raise(438, name);
  const v = o[k];
  if (typeof v === "function") return v.apply(o, hostArgs(v, args, named));
  if (!args.length && !named) return v;
  return V.index(v, args);
};

V.set = function (o, name, v, isSet) {
  needObject(o);
  if (o instanceof VbObject) { o.__set(name, v, isSet); return; }
  const k = findKey(o, name);
  if (k === null) V.raise(438, name);
  o[k] = v;
};

// `o.name(args) = v`: an indexed property, or an element of what the
// property returns.
V.setm = function (o, name, args, v) {
  needObject(o);
  if (o instanceof VbObject) {
    const f = o["__let_" + name.toLowerCase()];
    if (typeof f === "function") { f.apply(o, args.concat([v])); return; }
  }
  V.setIdx(V.get(o, name), args, v);
};

// `o(args)`: an array element, a collection item or the default member.
V.index = function (o, args) {
  if (o instanceof VArray) return o.get(args);
  if (o === null) V.raise(91);
  if (o === undefined) V.raise(13);
  if (typeof o === "function") return o.apply(null, args.map((a) => (a === V.Missing ? undefined : a)));
  if (typeof o !== "object") V.raise(13);
  if (o instanceof VbObject) return o.__index(args);
  if (Array.isArray(o)) {
    if (args.length !== 1) V.raise(9);
    const i = V.clng(args[0]);
    if (i < 1 || i > o.length) V.raise(9);
    return o[i - 1];
  }
  for (const d of ["Item", "_Default", "Value"]) {
    const k = findKey(o, d);
    if (k !== null) {
      const it = o[k];
      if (typeof it === "function") return it.apply(o, args);
      return args.length ? V.index(it, args) : it;
    }
  }
  V.raise(438, "Item");
};

V.setIdx = function (o, args, v) {
  if (o instanceof VArray) { o.set(args, v); return; }
  if (o === null) V.raise(91);
  if (o === undefined || typeof o !== "object") V.raise(13);
  if (o instanceof VbObject) { o.__setIdx(args, v); return; }
  if (typeof o.__setItem === "function") { o.__setItem(args, v); return; }
  if (Array.isArray(o) && args.length === 1) {
    const i = V.clng(args[0]);
    if (i < 1 || i > o.length) V.raise(9);
    o[i - 1] = v;
    return;
  }
  V.raise(438, "Item");
};

// ----- Host globals ---------------------------------------------------------

const globalCache = new Map();
// A host name written in another case. Global properties are found
// directly; names the host declares with const, let or class (which are
// not properties of the global object) are found through the list the
// host may give in `__host_names`.
V.g = function (name) {
  if (name in G) return G[name];
  const l = name.toLowerCase();
  let k = globalCache.get(l);
  if (k === undefined) {
    k = null;
    for (const n of Object.getOwnPropertyNames(G)) if (n.toLowerCase() === l) { k = { p: n }; break; }
    if (k === null && Array.isArray(G.__host_names)) {
      for (const n of G.__host_names) if (String(n).toLowerCase() === l) { k = { e: String(n) }; break; }
    }
    globalCache.set(l, k);
  }
  if (k === null) return undefined;
  if (k.p !== undefined) return G[k.p];
  return (0, eval)(k.e);
};
V.setg = function (cur, name, v) {
  // A form's control assigned by its name: its default property.
  if (cur !== null && typeof cur === "object" && typeof cur.__default === "string") {
    cur[cur.__default] = v;
    return;
  }
  const k = findKey(G, name);
  G[k === null ? name : k] = v;
};

// A name called with arguments: a host function, or an indexed value.
V.callg = function (value, name, args, named) {
  if (typeof value === "function") return value.apply(null, hostArgs(value, args, named));
  if (value === undefined) V.raise(35, name);
  if (!args.length && !named) return value;
  return V.index(value, args);
};

V.callf = function (f, args, named) {
  return f.apply(null, V.named(f, args, named));
};

// ----- For Each -------------------------------------------------------------

V.each = function (c) {
  if (c instanceof VArray) return c.items();
  if (c === null) V.raise(91);
  if (c === undefined || typeof c !== "object") V.raise(424);
  if (Array.isArray(c)) return c.slice();
  if (c instanceof Collection) return c._items.map((x) => x.v);
  if (c instanceof Dictionary) return c.Keys().items();
  if (c instanceof VbObject) return c.__each();
  if (typeof c[Symbol.iterator] === "function") return Array.from(c);
  const kc = findKey(c, "Count"), ki = findKey(c, "Item");
  if (kc !== null && ki !== null) {
    const n = V.num(V.get(c, "Count"));
    const out = [];
    for (let i = 1; i <= n; i++) out.push(V.call(c, "Item", [i], null));
    return out;
  }
  if (typeof c.forEach === "function") {
    const out = [];
    c.forEach((x) => out.push(x));
    return out;
  }
  V.raise(438, "For Each");
};

// ----- Conversions ------------------------------------------------------------

// Banker's rounding (half to even), as VBA rounds to integers.
function bround(x) {
  const r = Math.round(x);
  if (Math.abs(x % 1) === 0.5) return 2 * Math.round(x / 2);
  return r;
}
V.bround = bround;

const NUMERIC = /^\s*[+-]?(\d+\.?\d*|\.\d+)([eEdD][+-]?\d+)?\s*$/;
const HEXNUM = /^\s*&[hH]([0-9a-fA-F]+)&?\s*$/;
const OCTNUM = /^\s*&[oO]([0-7]+)&?\s*$/;

function strNum(s) {
  if (NUMERIC.test(s)) return parseFloat(s.replace(/[dD]/, "e"));
  let m = HEXNUM.exec(s);
  if (m) return parseInt(m[1], 16);
  m = OCTNUM.exec(s);
  if (m) return parseInt(m[1], 8);
  return null;
}

// The default value of an object used as a value.
function dv(x) {
  if (x === V.Err) return V.Err.Number;
  // A form control: its default property (Value).
  if (x !== null && typeof x === "object" && typeof x.__default === "string") return x[x.__default];
  return x;
}

V.num = function (x) {
  x = dv(x);
  switch (typeof x) {
    case "number": return x;
    case "boolean": return x ? -1 : 0;
    case "undefined": return 0;
    case "string": {
      const n = strNum(x);
      if (n === null) {
        const d = parseDate(x);
        if (d !== null) return d;
        V.raise(13);
      }
      return n;
    }
  }
  if (x instanceof VDate) return x.v;
  if (x === V.Null) V.raise(94);
  if (x === null) V.raise(91);
  V.raise(13);
};

V.cint = function (x) {
  const n = bround(V.num(x));
  if (n < -32768 || n > 32767) V.raise(6);
  return n;
};
V.clng = function (x) {
  const n = bround(V.num(x));
  if (n < -2147483648 || n > 2147483647) V.raise(6);
  return n;
};
V.cbyte = function (x) {
  const n = bround(V.num(x));
  if (n < 0 || n > 255) V.raise(6);
  return n;
};
V.cdbl = function (x) { return V.num(x); };
V.csng = function (x) { return V.num(x); };
V.ccur = function (x) { return bround(V.num(x) * 10000) / 10000; };
V.cbool = function (x) {
  x = dv(x);
  if (typeof x === "boolean") return x;
  if (typeof x === "string") {
    const t = x.trim().toLowerCase();
    if (t === "true") return true;
    if (t === "false") return false;
    const n = strNum(x);
    if (n === null) V.raise(13);
    return n !== 0;
  }
  return V.num(x) !== 0;
};
V.cdate = function (x) {
  x = dv(x);
  if (x instanceof VDate) return x;
  if (typeof x === "string") {
    const d = parseDate(x);
    if (d === null) {
      const n = strNum(x);
      if (n === null) V.raise(13);
      return new VDate(n);
    }
    return new VDate(d);
  }
  return new VDate(V.num(x));
};

// A number as VBA writes it: up to 15 significant digits.
function fmtNum(n) {
  if (!isFinite(n)) return String(n);
  if (Number.isInteger(n) && Math.abs(n) < 1e15) return String(n);
  const p = Number(n.toPrecision(15));
  const a = Math.abs(p);
  if (a !== 0 && (a >= 1e15 || a < 1e-4)) {
    const e = p.toExponential();
    const m = /^(-?[\d.]+)e([+-])(\d+)$/.exec(e);
    if (m) {
      let mant = m[1];
      if (mant.indexOf(".") >= 0) mant = mant.replace(/0+$/, "").replace(/\.$/, "");
      const exp = m[3].length < 2 ? "0" + m[3] : m[3];
      return mant + "E" + m[2] + exp;
    }
  }
  let s = String(p);
  if (s.indexOf("e") >= 0) s = p.toFixed(20).replace(/0+$/, "").replace(/\.$/, "");
  return s;
}
V.fmtNum = fmtNum;

V.cstr = function (x) {
  x = dv(x);
  switch (typeof x) {
    case "string": return x;
    case "number": return fmtNum(x);
    case "boolean": return x ? "True" : "False";
    case "undefined": return "";
  }
  if (x instanceof VDate) return fmtDate(x.v, generalDateFormat(x.v));
  if (x === V.Null) V.raise(94);
  if (x === V.Missing) V.raise(448);
  if (x === null) V.raise(91);
  if (x instanceof VArray) V.raise(13);
  if (x && typeof x === "object") {
    const k = findKey(x, "Value");
    if (k !== null && typeof x[k] !== "function") return V.cstr(x[k]);
  }
  V.raise(13);
};

// Assigning without Set: arrays and user types are copied.
V.letv = function (x) {
  if (x instanceof VArray) return x.clone();
  if (x instanceof VbRecord) return x.clone();
  if (x instanceof Ref) return x.g();
  return x;
};

V.bool = function (x) {
  if (typeof x === "boolean") return x;
  if (x === V.Null || x === undefined) return false;
  return V.cbool(x);
};

// ----- Operators ----------------------------------------------------------

function isNull(a) { return a === V.Null; }

V.add = function (a, b) {
  a = dv(a); b = dv(b);
  if (isNull(a) || isNull(b)) return V.Null;
  if (typeof a === "string" && typeof b === "string") return a + b;
  if (a === undefined && typeof b === "string") return b;
  if (b === undefined && typeof a === "string") return a;
  if (a instanceof VDate || b instanceof VDate) return new VDate(V.num(a) + V.num(b));
  return V.num(a) + V.num(b);
};
V.sub = function (a, b) {
  a = dv(a); b = dv(b);
  if (isNull(a) || isNull(b)) return V.Null;
  if (a instanceof VDate && !(b instanceof VDate)) return new VDate(a.v - V.num(b));
  return V.num(a) - V.num(b);
};
V.mul = function (a, b) {
  if (isNull(a) || isNull(b)) return V.Null;
  return V.num(a) * V.num(b);
};
V.div = function (a, b) {
  if (isNull(a) || isNull(b)) return V.Null;
  const x = V.num(a), y = V.num(b);
  if (y === 0) V.raise(x === 0 ? 6 : 11);
  return x / y;
};
V.idiv = function (a, b) {
  if (isNull(a) || isNull(b)) return V.Null;
  const x = V.clng(a), y = V.clng(b);
  if (y === 0) V.raise(11);
  return Math.trunc(x / y);
};
V.mod = function (a, b) {
  if (isNull(a) || isNull(b)) return V.Null;
  const x = V.clng(a), y = V.clng(b);
  if (y === 0) V.raise(11);
  const r = x % y;
  return r === 0 ? 0 : r;
};
V.pow = function (a, b) {
  if (isNull(a) || isNull(b)) return V.Null;
  const x = V.num(a), y = V.num(b);
  if (x < 0 && !Number.isInteger(y)) V.raise(5);
  return Math.pow(x, y);
};
V.neg = function (a) {
  if (isNull(a)) return V.Null;
  return -V.num(a);
};
V.cat = function (a, b) {
  const s = (x) => (isNull(x) ? "" : V.cstr(x));
  if (isNull(a) && isNull(b)) return V.Null;
  return s(a) + s(b);
};

function bits(x) { return V.clng(x) | 0; }
function logical(a, b, fb, fn) {
  if (typeof a === "boolean" && typeof b === "boolean") return fb(a, b);
  return fn(bits(a), bits(b));
}
V.not = function (a) {
  if (isNull(a)) return V.Null;
  if (typeof a === "boolean") return !a;
  return ~bits(a);
};
V.and = function (a, b) {
  if (isNull(a) || isNull(b)) {
    if (a === false || b === false || a === 0 || b === 0) return a === 0 || b === 0 ? 0 : false;
    return V.Null;
  }
  return logical(a, b, (x, y) => x && y, (x, y) => x & y);
};
V.or = function (a, b) {
  if (isNull(a) || isNull(b)) {
    if (a === true || b === true) return true;
    return V.Null;
  }
  return logical(a, b, (x, y) => x || y, (x, y) => x | y);
};
V.xor = function (a, b) {
  if (isNull(a) || isNull(b)) return V.Null;
  return logical(a, b, (x, y) => x !== y, (x, y) => x ^ y);
};
V.eqv = function (a, b) {
  if (isNull(a) || isNull(b)) return V.Null;
  return logical(a, b, (x, y) => x === y, (x, y) => ~(x ^ y));
};
V.imp = function (a, b) {
  if (isNull(a) || isNull(b)) return V.Null;
  return logical(a, b, (x, y) => !x || y, (x, y) => ~x | y);
};

// Comparison as VBA does it for Variants: numbers and dates by value,
// strings as text (case-insensitive with Option Compare Text), a number
// below a string that is not a number.
V.cmp = function (op, a, b, text) {
  a = dv(a); b = dv(b);
  if (isNull(a) || isNull(b)) return V.Null;
  if (typeof a === "boolean") a = a ? -1 : 0;
  if (typeof b === "boolean") b = b ? -1 : 0;
  if (a instanceof VDate) a = a.v;
  if (b instanceof VDate) b = b.v;
  if (a === undefined) a = typeof b === "string" ? "" : 0;
  if (b === undefined) b = typeof a === "string" ? "" : 0;
  if ((a === null || typeof a === "object") || (b === null || typeof b === "object")) {
    if (op === "=") return a === b;
    if (op === "<>") return a !== b;
    V.raise(13);
  }
  let c;
  if (typeof a === "string" && typeof b === "string") {
    let x = a, y = b;
    if (text) { x = x.toLowerCase(); y = y.toLowerCase(); }
    c = x < y ? -1 : x > y ? 1 : 0;
  } else if (typeof a === "number" && typeof b === "number") {
    c = a < b ? -1 : a > b ? 1 : 0;
  } else {
    // A number and a string.
    const sa = typeof a === "string";
    const s = sa ? a : b;
    const n = strNum(s);
    if (n !== null) {
      const x = sa ? n : a, y = sa ? b : n;
      c = x < y ? -1 : x > y ? 1 : 0;
    } else {
      c = sa ? 1 : -1;
    }
  }
  switch (op) {
    case "=": return c === 0;
    case "<>": return c !== 0;
    case "<": return c < 0;
    case ">": return c > 0;
    case "<=": return c <= 0;
    default: return c >= 0;
  }
};
V.is = function (a, b) {
  return a === b || (a === null && b === undefined) || (a === undefined && b === null);
};

const likeCache = new Map();
V.like = function (s, p, text) {
  if (isNull(s) || isNull(p)) return V.Null;
  s = V.cstr(s); p = V.cstr(p);
  const key = (text ? "i" : "c") + p;
  let re = likeCache.get(key);
  if (!re) {
    let r = "^";
    for (let i = 0; i < p.length; i++) {
      const c = p[i];
      if (c === "?") r += "[\\s\\S]";
      else if (c === "*") r += "[\\s\\S]*";
      else if (c === "#") r += "[0-9]";
      else if (c === "[") {
        const end = p.indexOf("]", i + 1);
        if (end < 0) V.raise(93);
        let body = p.slice(i + 1, end);
        let neg = false;
        if (body[0] === "!") { neg = true; body = body.slice(1); }
        body = body.replace(/[\\\]^]/g, "\\$&");
        r += "[" + (neg ? "^" : "") + body + "]";
        i = end;
      } else r += c.replace(/[.*+?^${}()|[\]\\\/]/g, "\\$&");
    }
    re = new RegExp(r + "$", text ? "i" : "");
    likeCache.set(key, re);
  }
  return re.test(s);
};

// ----- Statements helpers -------------------------------------------------

V.midset = function (s, start, len, v) {
  s = V.cstr(s); v = V.cstr(v);
  const st = V.clng(start);
  if (st < 1) V.raise(5);
  if (st > s.length) V.raise(5);
  let n = miss(len) ? v.length : V.clng(len);
  n = Math.min(n, v.length, s.length - st + 1);
  return s.slice(0, st - 1) + v.slice(0, n) + s.slice(st - 1 + n);
};

function out(line) {
  if (typeof G.print === "function") G.print(line);
  else if (typeof G.__host_print === "function") G.__host_print(line);
}
V.print = out;
// Debug.Print: items after ";" follow on, after "," go to the next
// 14-column zone.
// A Debug.Print that ends with a separator leaves its line open.
let pending = null;
V.dprint = function (items) {
  let s = pending === null ? "" : pending;
  pending = null;
  for (const [e, sep] of items) {
    s += isNull(e) ? "Null" : V.cstr(e);
    if (sep === ",") s += " ".repeat(14 - (s.length % 14));
  }
  const last = items.length ? items[items.length - 1][1] : "";
  if (last === ";" || last === ",") pending = s;
  else out(s);
};
V.flush = function () {
  if (pending !== null) { out(pending); pending = null; }
};

// ----- Dates ----------------------------------------------------------------

const MS_DAY = 86400000;
function serialOf(y, m, d, h, mi, s) {
  return Date.UTC(y, m - 1, d, h || 0, mi || 0, s || 0) / MS_DAY + 25569;
}
function partsOf(v) {
  // Round to the second, as VBA shows dates.
  const ms = Math.round((v - 25569) * 86400) * 1000;
  const d = new Date(ms);
  return {
    y: d.getUTCFullYear(), m: d.getUTCMonth() + 1, d: d.getUTCDate(),
    h: d.getUTCHours(), n: d.getUTCMinutes(), s: d.getUTCSeconds(), w: d.getUTCDay(),
  };
}
V.serialOf = serialOf;
V.partsOf = partsOf;

const MONTHS = ["January", "February", "March", "April", "May", "June", "July", "August",
  "September", "October", "November", "December"];
const DAYS = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];

function parseDate(str) {
  const t = String(str).trim().toLowerCase();
  if (!t || !/\d/.test(t)) return null;
  let m;
  let time = 0, rest = t;
  const tm = /(\d{1,2}):(\d{2})(?::(\d{2}))?\s*(am|pm)?$/.exec(t);
  if (tm) {
    let h = +tm[1];
    if (tm[4] === "pm" && h < 12) h += 12;
    if (tm[4] === "am" && h === 12) h = 0;
    time = (h * 3600 + +tm[2] * 60 + +(tm[3] || 0)) / 86400;
    rest = t.slice(0, tm.index).trim();
    if (!rest) return time;
  }
  rest = rest.replace(/,/g, " ").replace(/\s+/g, " ").replace(/t$/, "").trim();
  const mon = (s) => {
    const i = MONTHS.findIndex((n) => s.length >= 3 && n.toLowerCase().startsWith(s));
    return i < 0 ? null : i + 1;
  };
  let y, mo, d;
  if ((m = /^(\d{4})[-\/.](\d{1,2})[-\/.](\d{1,2})$/.exec(rest))) { y = +m[1]; mo = +m[2]; d = +m[3]; }
  else if ((m = /^(\d{1,2})[-\/.](\d{1,2})[-\/.](\d{2,4})$/.exec(rest))) { mo = +m[1]; d = +m[2]; y = +m[3]; }
  else if ((m = /^(\d{1,2})[ -]([a-z]+)[ -](\d{2,4})$/.exec(rest)) && mon(m[2])) { d = +m[1]; mo = mon(m[2]); y = +m[3]; }
  else if ((m = /^([a-z]+) (\d{1,2}) (\d{2,4})$/.exec(rest)) && mon(m[1])) { mo = mon(m[1]); d = +m[2]; y = +m[3]; }
  else return null;
  if (y < 30) y += 2000; else if (y < 100) y += 1900;
  if (mo < 1 || mo > 12 || d < 1 || d > 31) return null;
  return serialOf(y, mo, d) + time;
}
V.parseDate = parseDate;

function generalDateFormat(v) {
  const hasDate = Math.trunc(v) !== 0;
  const hasTime = Math.abs(v - Math.trunc(v)) > 1e-9;
  if (hasDate && hasTime) return "m/d/yyyy h:nn:ss AM/PM";
  if (hasTime) return "h:nn:ss AM/PM";
  return "m/d/yyyy";
}

function pad(n, w) { let s = String(n); while (s.length < w) s = "0" + s; return s; }

function fmtDate(v, f) {
  const p = partsOf(v);
  // Tokens: date parts and literal text.
  const toks = [];
  let i = 0;
  while (i < f.length) {
    const rest = f.slice(i);
    const lr = rest.toLowerCase();
    let m;
    if (lr.startsWith("am/pm")) { toks.push({ t: "ampm", up: rest[0] === "A" }); i += 5; continue; }
    if (lr.startsWith("a/p")) { toks.push({ t: "ap", up: rest[0] === "A" }); i += 3; continue; }
    if (rest[0] === '"') {
      const e = f.indexOf('"', i + 1);
      toks.push({ lit: f.slice(i + 1, e < 0 ? f.length : e) });
      i = e < 0 ? f.length : e + 1;
      continue;
    }
    if (rest[0] === "\\") { toks.push({ lit: rest[1] || "" }); i += 2; continue; }
    if ((m = /^(y{4}|y{2}|y|q|m{1,4}|d{1,4}|h{1,2}|n{1,2}|s{1,2}|w{1,2})/i.exec(rest))) {
      toks.push({ t: m[1].toLowerCase() });
      i += m[1].length;
      continue;
    }
    toks.push({ lit: rest[0] });
    i += 1;
  }
  const ampm = toks.some((x) => x.t === "ampm" || x.t === "ap");
  const parts = toks.filter((x) => x.t);
  // m and mm right after an hour or right before seconds are minutes.
  parts.forEach((x, k) => {
    if (x.t === "m" || x.t === "mm") {
      const prev = parts[k - 1], next = parts[k + 1];
      if ((prev && prev.t[0] === "h") || (next && next.t[0] === "s")) x.t = x.t === "m" ? "n" : "nn";
    }
  });
  const h12 = ((p.h + 11) % 12) + 1;
  let out = "";
  for (const x of toks) {
    if (x.lit !== undefined) { out += x.lit; continue; }
    switch (x.t) {
      case "yyyy": out += pad(p.y, 4); break;
      case "yy": out += pad(p.y % 100, 2); break;
      case "y": out += String(Math.floor(v) - serialOf(p.y, 1, 1) + 1); break;
      case "q": out += String(Math.floor((p.m - 1) / 3) + 1); break;
      case "mmmm": out += MONTHS[p.m - 1]; break;
      case "mmm": out += MONTHS[p.m - 1].slice(0, 3); break;
      case "mm": out += pad(p.m, 2); break;
      case "m": out += String(p.m); break;
      case "dddd": out += DAYS[p.w]; break;
      case "ddd": out += DAYS[p.w].slice(0, 3); break;
      case "dd": out += pad(p.d, 2); break;
      case "d": out += String(p.d); break;
      case "hh": out += pad(ampm ? h12 : p.h, 2); break;
      case "h": out += String(ampm ? h12 : p.h); break;
      case "nn": out += pad(p.n, 2); break;
      case "n": out += String(p.n); break;
      case "ss": out += pad(p.s, 2); break;
      case "s": out += String(p.s); break;
      case "w": out += String(p.w + 1); break;
      case "ww": out += String(Math.floor((Math.floor(v) - serialOf(p.y, 1, 1) + new Date(Date.UTC(p.y, 0, 1)).getUTCDay()) / 7) + 1); break;
      case "ampm": out += p.h < 12 ? (x.up ? "AM" : "am") : (x.up ? "PM" : "pm"); break;
      case "ap": out += p.h < 12 ? (x.up ? "A" : "a") : (x.up ? "P" : "p"); break;
    }
  }
  return out;
}
V.fmtDate = fmtDate;

function today() {
  const d = new Date();
  return serialOf(d.getFullYear(), d.getMonth() + 1, d.getDate());
}
function nowSerial() {
  const d = new Date();
  return serialOf(d.getFullYear(), d.getMonth() + 1, d.getDate(), d.getHours(), d.getMinutes(), d.getSeconds());
}

// ----- Number formats ---------------------------------------------------------

function roundHalfUp(x, d) {
  const f = Math.pow(10, d);
  const r = Math.round(Math.abs(x) * f + 1e-9) / f;
  return x < 0 ? -r : r;
}

function fmtNumber(x, f) {
  // Sections: positive;negative;zero;null
  const secs = splitSections(f);
  let sec = secs[0];
  let neg = x < 0;
  if (x < 0 && secs.length > 1 && secs[1] !== "") { sec = secs[1]; }
  else if (x === 0 && secs.length > 2 && secs[2] !== "") { sec = secs[2]; neg = false; }
  const own = sec !== secs[0];
  let a = Math.abs(x);
  // Literals, placeholders and options.
  let percent = false, thousands = false, sci = null;
  const parts = [];
  for (let i = 0; i < sec.length; i++) {
    const c = sec[i];
    if (c === '"') {
      const e = sec.indexOf('"', i + 1);
      parts.push({ lit: sec.slice(i + 1, e < 0 ? sec.length : e) });
      i = e < 0 ? sec.length : e;
    } else if (c === "\\") { parts.push({ lit: sec[i + 1] || "" }); i++; }
    else if (c === "0" || c === "#") parts.push({ ph: c });
    else if (c === ".") parts.push({ dot: true });
    else if (c === ",") parts.push({ comma: true });
    else if (c === "%") { percent = true; parts.push({ lit: "%" }); }
    else if ((c === "E" || c === "e") && (sec[i + 1] === "+" || sec[i + 1] === "-")) {
      let j = i + 2; let zs = 0;
      while (sec[j] === "0" || sec[j] === "#") { zs++; j++; }
      sci = { sign: sec[i + 1], digits: zs, ch: c };
      parts.push({ sci: true });
      i = j - 1;
    } else parts.push({ lit: c });
  }
  if (percent) a *= 100;
  const dotAt = parts.findIndex((p) => p.dot);
  const intPh = [], decPh = [];
  parts.forEach((p, k) => {
    if (p.ph) (dotAt >= 0 && k > dotAt ? decPh : intPh).push(k);
  });
  if (!intPh.length && !decPh.length) {
    return (neg && !own ? "-" : "") + parts.map((p) => p.lit || "").join("");
  }
  // Thousands: a comma between integer placeholders.
  parts.forEach((p, k) => {
    if (p.comma && intPh.length && k > intPh[0] && k < intPh[intPh.length - 1] + 1 && (dotAt < 0 || k < dotAt)) thousands = true;
  });
  let exp = 0;
  if (sci) {
    if (a !== 0) {
      exp = Math.floor(Math.log10(a)) - (Math.max(1, intPh.length) - 1);
      a = a / Math.pow(10, exp);
    }
  }
  const minDec = decPh.filter((k) => parts[k].ph === "0").length;
  const maxDec = decPh.length;
  a = roundHalfUp(a, maxDec);
  if (sci && a >= Math.pow(10, Math.max(1, intPh.length))) { a /= 10; exp++; }
  let [ip, dp] = a.toFixed(maxDec).split(".");
  dp = dp || "";
  while (dp.length > minDec && dp[dp.length - 1] === "0") dp = dp.slice(0, -1);
  const minInt = intPh.filter((k) => parts[k].ph === "0").length;
  if (ip === "0" && minInt === 0) ip = "";
  while (ip.length < minInt) ip = "0" + ip;
  if (thousands) ip = ip.replace(/\B(?=(\d{3})+(?!\d))/g, ",");
  // Integer digits fill the integer placeholders from the right; extra
  // digits go to the first one.
  const ints = new Array(parts.length).fill("");
  let di = ip.length;
  for (let k = intPh.length - 1; k >= 0; k--) {
    if (k === 0) { ints[intPh[k]] = ip.slice(0, di); di = 0; }
    else if (di > 0) {
      const ch = ip[di - 1];
      if (ch === "," && thousands) { ints[intPh[k]] = ip.slice(Math.max(0, di - 2), di); di -= 2; }
      else { ints[intPh[k]] = ch; di -= 1; }
    }
  }
  if (thousands) {
    // With thousands separators the digits go in one piece.
    intPh.forEach((k, n) => { ints[k] = n === intPh.length - 1 ? ip : ""; });
  }
  let s = "";
  let decIdx = 0;
  parts.forEach((p, k) => {
    if (p.lit !== undefined) s += p.lit;
    else if (p.ph && intPh.includes(k)) s += ints[k];
    else if (p.ph) { if (decIdx < dp.length) s += dp[decIdx]; decIdx++; }
    else if (p.dot) s += ".";
    else if (p.sci) {
      const es = Math.abs(exp);
      s += sci.ch + (exp < 0 ? "-" : sci.sign === "+" ? "+" : "") + pad(es, sci.digits);
    }
  });
  return (neg && !own && Math.abs(x) >= Math.pow(10, -maxDec) / 2 ? "-" : "") + s;
}
function splitSections(f) {
  const out = [];
  let cur = "", q = false;
  for (let i = 0; i < f.length; i++) {
    const c = f[i];
    if (c === '"') q = !q;
    if (c === ";" && !q) { out.push(cur); cur = ""; continue; }
    if (c === "\\" && !q) { cur += c + (f[i + 1] || ""); i++; continue; }
    cur += c;
  }
  out.push(cur);
  return out;
}
V.fmtNumber = fmtNumber;

const NAMED_NUM = {
  "general number": null, "currency": "$#,##0.00;($#,##0.00)", "fixed": "0.00",
  "standard": "#,##0.00", "percent": "0.00%", "scientific": "0.00E+00",
};
const NAMED_DATE = {
  "general date": null, "long date": "dddd, mmmm d, yyyy", "medium date": "dd-mmm-yy",
  "short date": "m/d/yyyy", "long time": "h:nn:ss AM/PM", "medium time": "hh:nn AM/PM",
  "short time": "hh:nn",
};

function isDateFormat(f) {
  return /^(general date|long date|medium date|short date|long time|medium time|short time)$/i.test(f)
    || (/[dyhnsq]|mmm/i.test(f.replace(/"[^"]*"/g, "")) && !/[0#]/.test(f));
}

function format(x, f) {
  if (isNull(x)) return "";
  if (miss(f) || f === "") return V.cstr(x);
  f = V.cstr(f);
  const lf = f.toLowerCase();
  if (lf === "yes/no") return V.cbool(x) ? "Yes" : "No";
  if (lf === "true/false") return V.cbool(x) ? "True" : "False";
  if (lf === "on/off") return V.cbool(x) ? "On" : "Off";
  if (lf === ">" || lf === "<") return lf === ">" ? V.cstr(x).toUpperCase() : V.cstr(x).toLowerCase();
  if (/^[@&<>!]+$/.test(f)) return V.cstr(x);
  if (x instanceof VDate || (isDateFormat(f) && !(typeof x === "number"))) {
    const v = x instanceof VDate ? x.v : V.cdate(x).v;
    if (lf in NAMED_DATE) return fmtDate(v, NAMED_DATE[lf] || generalDateFormat(v));
    return fmtDate(v, f);
  }
  if (isDateFormat(f) && typeof x === "number") return fmtDate(x, NAMED_DATE[lf] || f);
  let n;
  if (typeof x === "string") {
    n = strNum(x);
    if (n === null) return x;
  } else n = V.num(x);
  if (lf in NAMED_NUM) {
    const nf = NAMED_NUM[lf];
    return nf === null ? fmtNum(n) : fmtNumber(n, nf);
  }
  return fmtNumber(n, f);
}
V.format = format;

// ----- Collection and Dictionary --------------------------------------------

class Collection {
  constructor() { this._items = []; }
  _find(key) {
    const l = String(key).toLowerCase();
    return this._items.findIndex((x) => x.k !== null && x.k === l);
  }
  _pos(index) {
    if (typeof index === "string") {
      const i = this._find(index);
      if (i < 0) V.raise(5);
      return i;
    }
    const i = V.clng(index);
    if (i < 1 || i > this._items.length) V.raise(9);
    return i - 1;
  }
  Add(item, key, before, after) {
    const k = miss(key) ? null : V.cstr(key).toLowerCase();
    if (k !== null && this._find(k) >= 0) V.raise(457);
    const e = { k: k, v: item };
    if (!miss(before)) this._items.splice(this._pos(before), 0, e);
    else if (!miss(after)) this._items.splice(this._pos(after) + 1, 0, e);
    else this._items.push(e);
  }
  Item(index) { return this._items[this._pos(index)].v; }
  Remove(index) { this._items.splice(this._pos(index), 1); }
  get Count() { return this._items.length; }
  [Symbol.iterator]() { return this._items.map((x) => x.v)[Symbol.iterator](); }
}
Collection.prototype.Add.__params = ["item", "key", "before", "after"];
Collection.prototype.Item.__params = ["index"];
V.Collection = Collection;

class Dictionary {
  constructor() { this._m = new Map(); this._text = false; }
  _k(key) {
    if (typeof key === "string") return this._text ? "s:" + key.toLowerCase() : "s:" + key;
    if (typeof key === "number") return "n:" + key;
    if (typeof key === "boolean") return "n:" + (key ? -1 : 0);
    if (key instanceof VDate) return "n:" + key.v;
    if (key === undefined) return "e:";
    return key;
  }
  get CompareMode() { return this._text ? 1 : 0; }
  set CompareMode(v) {
    if (this._m.size) V.raise(5);
    this._text = V.num(v) === 1;
  }
  Add(key, item) {
    const k = this._k(key);
    if (this._m.has(k)) V.raise(457);
    this._m.set(k, { key: key, v: item });
  }
  Exists(key) { return this._m.has(this._k(key)); }
  Item(key) {
    const k = this._k(key);
    if (!this._m.has(k)) this._m.set(k, { key: key, v: undefined });
    return this._m.get(k).v;
  }
  __setItem(args, v) {
    const k = this._k(args[0]);
    const e = this._m.get(k);
    if (e) e.v = v; else this._m.set(k, { key: args[0], v: v });
  }
  Remove(key) {
    const k = this._k(key);
    if (!this._m.has(k)) V.raise(32811);
    this._m.delete(k);
  }
  RemoveAll() { this._m.clear(); }
  Keys() { return V.arr(0, Array.from(this._m.values()).map((e) => e.key)); }
  Items() { return V.arr(0, Array.from(this._m.values()).map((e) => e.v)); }
  get Count() { return this._m.size; }
  [Symbol.iterator]() { return Array.from(this._m.values()).map((e) => e.key)[Symbol.iterator](); }
}
Dictionary.prototype.Add.__params = ["key", "item"];
V.Dictionary = Dictionary;

// ----- The VBA library ------------------------------------------------------

function str(x) { return isNull(x) ? null : V.cstr(x); }
function lng(x, d) { return miss(x) ? d : V.clng(x); }
function cmpMode(x) { return miss(x) ? (textCompare() ? 1 : 0) : V.clng(x); }

B.abs = (x) => (isNull(x) ? V.Null : Math.abs(V.num(x)));
B.asc = (s) => { s = V.cstr(s); if (!s.length) V.raise(5); return s.charCodeAt(0); };
B.ascw = B.asc;
B.atn = (x) => Math.atan(V.num(x));
B.beep = () => undefined;
B.callbyname = function (o, name, kind) {
  const args = Array.prototype.slice.call(arguments, 3);
  const k = V.clng(kind);
  if (k === 2) return V.get(o, name);
  if (k === 4 || k === 8) { V.set(o, name, args[0], k === 8); return undefined; }
  return V.call(o, name, args, null);
};
B.cbool = V.cbool;
B.cbyte = V.cbyte;
B.ccur = V.ccur;
B.cdate = V.cdate;
B.cdbl = V.cdbl;
B.cdec = V.cdbl;
B.choose = function (i) {
  const n = V.clng(i);
  if (n < 1 || n >= arguments.length) return V.Null;
  return arguments[n];
};
B.chr = (n) => { const c = V.clng(n); if (c < 0 || c > 65535) V.raise(5); return String.fromCharCode(c); };
B.chrw = B.chr;
B.cint = V.cint;
B.clng = V.clng;
B.clnglng = V.clng;
B.clngptr = V.clng;
B.command = () => "";
B.cos = (x) => Math.cos(V.num(x));
B.createobject = function (name) {
  const n = V.cstr(name).toLowerCase();
  if (n === "scripting.dictionary") return new Dictionary();
  if (n === "vba.collection") return new Collection();
  V.raise(429, V.cstr(name));
};
B.csng = V.csng;
B.cstr = V.cstr;
B.curdir = () => "";
B.cvar = (x) => x;
B.cvdate = V.cdate;
B.cverr = (n) => new VbError(V.clng(n), msgOf(V.clng(n)));
B.date = () => new VDate(today());
B.dateadd = function (interval, n, date) {
  const i = V.cstr(interval).toLowerCase();
  const k = V.num(n);
  const v = V.cdate(date).v;
  const p = partsOf(v);
  const frac = v - Math.floor(v);
  const months = (m) => {
    let y = p.y, mo = p.m + m;
    y += Math.floor((mo - 1) / 12);
    mo = ((mo - 1) % 12 + 12) % 12 + 1;
    const last = new Date(Date.UTC(y, mo, 0)).getUTCDate();
    return new VDate(serialOf(y, mo, Math.min(p.d, last)) + frac);
  };
  switch (i) {
    case "yyyy": return months(12 * Math.trunc(k));
    case "q": return months(3 * Math.trunc(k));
    case "m": return months(Math.trunc(k));
    case "y": case "d": case "w": return new VDate(v + Math.trunc(k));
    case "ww": return new VDate(v + 7 * Math.trunc(k));
    case "h": return new VDate(v + k / 24);
    case "n": return new VDate(v + k / 1440);
    case "s": return new VDate(v + k / 86400);
  }
  V.raise(5);
};
B.datediff = function (interval, d1, d2) {
  const i = V.cstr(interval).toLowerCase();
  const a = V.cdate(d1).v, b = V.cdate(d2).v;
  const pa = partsOf(a), pb = partsOf(b);
  switch (i) {
    case "yyyy": return pb.y - pa.y;
    case "q": return (pb.y - pa.y) * 4 + Math.floor((pb.m - 1) / 3) - Math.floor((pa.m - 1) / 3);
    case "m": return (pb.y - pa.y) * 12 + pb.m - pa.m;
    case "y": case "d": return Math.floor(b) - Math.floor(a);
    case "w": return Math.trunc((Math.floor(b) - Math.floor(a)) / 7);
    case "ww": return Math.floor((Math.floor(b) + 1) / 7) - Math.floor((Math.floor(a) + 1) / 7);
    case "h": return Math.floor(Math.round(b * 86400) / 3600) - Math.floor(Math.round(a * 86400) / 3600);
    case "n": return Math.floor(Math.round(b * 86400) / 60) - Math.floor(Math.round(a * 86400) / 60);
    case "s": return Math.round(b * 86400) - Math.round(a * 86400);
  }
  V.raise(5);
};
B.datepart = function (interval, date) {
  const i = V.cstr(interval).toLowerCase();
  const v = V.cdate(date).v;
  const p = partsOf(v);
  switch (i) {
    case "yyyy": return p.y;
    case "q": return Math.floor((p.m - 1) / 3) + 1;
    case "m": return p.m;
    case "y": return Math.floor(v) - serialOf(p.y, 1, 1) + 1;
    case "d": return p.d;
    case "w": return p.w + 1;
    case "ww": return Math.floor((Math.floor(v) - serialOf(p.y, 1, 1) + new Date(Date.UTC(p.y, 0, 1)).getUTCDay()) / 7) + 1;
    case "h": return p.h;
    case "n": return p.n;
    case "s": return p.s;
  }
  V.raise(5);
};
B.dateserial = (y, m, d) => {
  let yy = V.clng(y);
  if (yy >= 0 && yy < 30) yy += 2000; else if (yy >= 30 && yy < 100) yy += 1900;
  return new VDate(serialOf(yy, V.clng(m), V.clng(d)));
};
B.datevalue = (s) => new VDate(Math.floor(V.cdate(s).v));
B.day = (d) => partsOf(V.cdate(d).v).d;
B.dir = () => V.unsupported("Dir");
B.doevents = () => 0;
B.environ = () => "";
B.eof = () => V.unsupported("EOF");
B.erl = () => 0;
B.error = (n) => msgOf(miss(n) ? V.Err.Number : V.clng(n));
B.exp = (x) => Math.exp(V.num(x));
B.filelen = () => V.unsupported("FileLen");
B.filter = function (arr, match, include, compare) {
  const m = V.cstr(match);
  const inc = miss(include) ? true : V.cbool(include);
  const text = cmpMode(compare) === 1;
  const items = V.each(arr).map(V.cstr);
  const hit = (s) => (text ? s.toLowerCase().indexOf(m.toLowerCase()) : s.indexOf(m)) >= 0;
  return V.arr(0, items.filter((s) => hit(s) === inc));
};
B.fix = (x) => (isNull(x) ? V.Null : Math.trunc(V.num(x)));
B.format = (x, f) => format(x, f);
B.format.__params = ["expression", "format", "firstdayofweek", "firstweekofyear"];
B.formatnumber = function (x, digits, lead, parens, group) {
  const d = miss(digits) || V.clng(digits) < 0 ? 2 : V.clng(digits);
  const g = miss(group) || V.clng(group) !== 0;
  let f = (g ? "#,##0" : "0") + (d > 0 ? "." + "0".repeat(d) : "");
  if (!miss(lead) && V.clng(lead) === 0) f = f.replace(/0(?=\.|$)/, "#");
  if (!miss(parens) && V.clng(parens) !== 0) f = f + ";(" + f + ")";
  return fmtNumber(V.num(x), f);
};
B.formatcurrency = (x, digits) => "$" + B.formatnumber(x, digits);
B.formatpercent = (x, digits) => B.formatnumber(V.num(x) * 100, digits) + "%";
B.formatdatetime = function (d, kind) {
  const v = V.cdate(d).v;
  const k = miss(kind) ? 0 : V.clng(kind);
  const f = ["", "dddd, mmmm d, yyyy", "m/d/yyyy", "h:nn:ss AM/PM", "hh:nn"][k] || generalDateFormat(v);
  return fmtDate(v, f);
};
B.freefile = () => V.unsupported("FreeFile");
B.getobject = (p, c) => V.raise(429, miss(c) ? V.cstr(p) : V.cstr(c));
// The registry settings of a program (GetSetting and the like): the host
// keeps them (`__host_setting`); without one they last for this run.
const settingsStore = {};
function settingOp(op, app, section, key, value) {
  if (typeof G.__host_setting === "function") return G.__host_setting(op, app, section, key, value);
  const id = (app + "\u0001" + section).toLowerCase();
  const s = settingsStore[id] || (settingsStore[id] = {});
  if (op === "get") return Object.prototype.hasOwnProperty.call(s, key.toLowerCase()) ? s[key.toLowerCase()][1] : null;
  if (op === "set") { s[key.toLowerCase()] = [key, value]; return null; }
  if (op === "delete") { if (key === null) delete settingsStore[id]; else delete s[key.toLowerCase()]; return null; }
  return Object.keys(s).map((k) => s[k]);
}
B.getsetting = function (app, section, key, def) {
  const r = settingOp("get", V.cstr(app), V.cstr(section), V.cstr(key), null);
  return r === null || r === undefined ? (miss(def) ? "" : V.cstr(def)) : String(r);
};
B.getsetting.__params = ["appname", "section", "key", "default"];
B.savesetting = function (app, section, key, value) {
  settingOp("set", V.cstr(app), V.cstr(section), V.cstr(key), V.cstr(value));
};
B.savesetting.__params = ["appname", "section", "key", "setting"];
B.deletesetting = function (app, section, key) {
  settingOp("delete", V.cstr(app), V.cstr(section), miss(key) ? null : V.cstr(key), null);
};
B.deletesetting.__params = ["appname", "section", "key"];
B.getallsettings = function (app, section) {
  const pairs = settingOp("all", V.cstr(app), V.cstr(section), null, null) || [];
  if (!pairs.length) return undefined;
  const a = new VArray([[0, pairs.length - 1], [0, 1]], null, null, false);
  pairs.forEach((p, i) => { a.set([i, 0], String(p[0])); a.set([i, 1], String(p[1])); });
  return a;
};
B.hex = (x) => { const n = V.clng(x); return (n < 0 ? (n >>> 0) : n).toString(16).toUpperCase(); };
B.hour = (d) => partsOf(V.cdate(d).v).h;
B.iif = (c, a, b) => (V.bool(c) ? a : b);
B.inputbox = function (prompt, title, def) {
  const d = miss(def) ? "" : V.cstr(def);
  if (typeof G.__host_inputbox === "function") {
    const r = G.__host_inputbox(V.cstr(prompt), miss(title) ? "" : V.cstr(title), d);
    return r === undefined || r === null ? "" : String(r);
  }
  return d;
};
B.inputbox.__params = ["prompt", "title", "default", "xpos", "ypos", "helpfile", "context"];
B.instr = function (a, b, c, d) {
  let start = 1, s1, s2, mode;
  if (arguments.length >= 3 && !miss(c)) { start = V.clng(a); s1 = b; s2 = c; mode = d; }
  else { s1 = a; s2 = b; mode = c; }
  if (isNull(s1) || isNull(s2)) return V.Null;
  s1 = V.cstr(s1); s2 = V.cstr(s2);
  if (start < 1) V.raise(5);
  if (s2 === "") return start <= s1.length ? start : (start === 1 ? 1 : 0);
  if (cmpMode(mode) === 1) { s1 = s1.toLowerCase(); s2 = s2.toLowerCase(); }
  return s1.indexOf(s2, start - 1) + 1;
};
B.instrrev = function (s1, s2, start, mode) {
  if (isNull(s1) || isNull(s2)) return V.Null;
  s1 = V.cstr(s1); s2 = V.cstr(s2);
  let st = miss(start) ? -1 : V.clng(start);
  if (st === -1) st = s1.length;
  if (cmpMode(mode) === 1) { s1 = s1.toLowerCase(); s2 = s2.toLowerCase(); }
  if (s2 === "") return st;
  return s1.lastIndexOf(s2, st - s2.length) + 1;
};
B.int = (x) => (isNull(x) ? V.Null : Math.floor(V.num(x)));
B.isarray = (x) => x instanceof VArray || Array.isArray(x);
B.isdate = (x) => x instanceof VDate || (typeof x === "string" && parseDate(x) !== null);
B.isempty = (x) => x === undefined;
B.iserror = (x) => x instanceof VbError;
B.ismissing = (x) => x === V.Missing;
B.isnull = (x) => x === V.Null;
B.isnumeric = (x) => {
  if (typeof x === "number" || typeof x === "boolean" || x === undefined) return true;
  if (typeof x === "string") return strNum(x) !== null;
  return false;
};
B.isobject = (x) => x === null || (typeof x === "object" && !(x instanceof VArray) && !(x instanceof VDate) && x !== V.Null && x !== V.Missing);
B.join = function (arr, delim) {
  const d = miss(delim) ? " " : V.cstr(delim);
  return V.each(arr).map(V.cstr).join(d);
};
B.lbound = function (a, dim) {
  if (!(a instanceof VArray)) { if (Array.isArray(a)) return 1; V.raise(13); }
  const k = lng(dim, 1);
  if (!a.dims || k < 1 || k > a.dims.length) V.raise(9);
  return a.dims[k - 1][0];
};
B.ubound = function (a, dim) {
  if (!(a instanceof VArray)) { if (Array.isArray(a)) return a.length; V.raise(13); }
  const k = lng(dim, 1);
  if (!a.dims || k < 1 || k > a.dims.length) V.raise(9);
  return a.dims[k - 1][1];
};
B.lcase = (s) => (isNull(s) ? V.Null : V.cstr(s).toLowerCase());
B.ucase = (s) => (isNull(s) ? V.Null : V.cstr(s).toUpperCase());
B.left = (s, n) => {
  if (isNull(s)) return V.Null;
  const k = V.clng(n); if (k < 0) V.raise(5);
  return V.cstr(s).slice(0, k);
};
B.right = (s, n) => {
  if (isNull(s)) return V.Null;
  const k = V.clng(n); if (k < 0) V.raise(5);
  const t = V.cstr(s);
  return k === 0 ? "" : t.slice(Math.max(0, t.length - k));
};
B.mid = (s, start, n) => {
  if (isNull(s)) return V.Null;
  const st = V.clng(start); if (st < 1) V.raise(5);
  const t = V.cstr(s);
  if (miss(n)) return t.slice(st - 1);
  const k = V.clng(n); if (k < 0) V.raise(5);
  return t.slice(st - 1, st - 1 + k);
};
B.mid.__params = ["string", "start", "length"];
B.len = (s) => (isNull(s) ? V.Null : V.cstr(s).length);
B.lof = () => V.unsupported("LOF");
B.log = (x) => { const v = V.num(x); if (v <= 0) V.raise(5); return Math.log(v); };
B.ltrim = (s) => (isNull(s) ? V.Null : V.cstr(s).replace(/^ +/, ""));
B.rtrim = (s) => (isNull(s) ? V.Null : V.cstr(s).replace(/ +$/, ""));
B.trim = (s) => (isNull(s) ? V.Null : V.cstr(s).replace(/^ +| +$/g, ""));
B.minute = (d) => partsOf(V.cdate(d).v).n;
B.month = (d) => partsOf(V.cdate(d).v).m;
B.monthname = (m, abbr) => { const n = MONTHS[V.clng(m) - 1]; if (!n) V.raise(5); return !miss(abbr) && V.cbool(abbr) ? n.slice(0, 3) : n; };
B.msgbox = function (prompt, buttons, title) {
  const p = isNull(prompt) ? "Null" : V.cstr(miss(prompt) ? "" : prompt);
  const b = miss(buttons) ? 0 : V.clng(buttons);
  const t = miss(title) ? "" : V.cstr(title);
  if (typeof G.__host_msgbox === "function") {
    const r = G.__host_msgbox(p, b, t);
    return r === undefined || r === null ? 1 : V.clng(r);
  }
  out(p);
  // Without a dialog: the first button.
  const first = [1, 1, 3, 6, 6, 4][b & 7];
  return first === undefined ? 1 : first;
};
B.msgbox.__params = ["prompt", "buttons", "title", "helpfile", "context"];
B.now = () => new VDate(nowSerial());
B.oct = (x) => { const n = V.clng(x); return (n < 0 ? (n >>> 0) : n).toString(8); };
B.qbcolor = (c) => [0x000000, 0x800000, 0x008000, 0x808000, 0x000080, 0x800080, 0x008080, 0xC0C0C0,
  0x808080, 0xFF0000, 0x00FF00, 0xFFFF00, 0x0000FF, 0xFF00FF, 0x00FFFF, 0xFFFFFF][V.clng(c)];
// VBA's own generator, so Rnd(-n) repeats as in VBA.
let seed = 327680;
let last = seed / 16777216;
B.rnd = function (n) {
  const v = miss(n) ? 1 : V.num(n);
  if (v === 0) return last;
  if (v < 0) {
    const buf = new DataView(new ArrayBuffer(4));
    buf.setFloat32(0, v);
    const b = buf.getUint32(0);
    seed = ((b & 0xFFFFFF) + (b >>> 24)) & 0xFFFFFF;
  }
  seed = (seed * 1140671485 + 12820163) % 16777216;
  last = seed / 16777216;
  return last;
};
B.randomize = function (n) {
  const v = miss(n) ? Date.now() / 1000 : V.num(n);
  const buf = new DataView(new ArrayBuffer(8));
  buf.setFloat64(0, v);
  const hi = buf.getUint32(0);
  seed = (seed & 0xFF) | (((hi & 0xFFFF) ^ (hi >>> 16)) << 8) & 0xFFFF00;
  return undefined;
};
B.replace = function (expr, find, repl, start, count, mode) {
  if (isNull(expr)) return V.Null;
  let s = V.cstr(expr);
  const f = V.cstr(find), r = V.cstr(repl);
  const st = lng(start, 1);
  if (st < 1) V.raise(5);
  let c = lng(count, -1);
  s = s.slice(st - 1);
  if (f === "") return s;
  const text = cmpMode(mode) === 1;
  let outS = "", i = 0;
  const hay = text ? s.toLowerCase() : s, needle = text ? f.toLowerCase() : f;
  while (c !== 0) {
    const j = hay.indexOf(needle, i);
    if (j < 0) break;
    outS += s.slice(i, j) + r;
    i = j + f.length;
    if (c > 0) c--;
  }
  return outS + s.slice(i);
};
B.replace.__params = ["expression", "find", "replace", "start", "count", "compare"];
B.rgb = (r, g, b) => (V.clng(r) & 255) + (V.clng(g) & 255) * 256 + (V.clng(b) & 255) * 65536;
B.round = (x, d) => {
  if (isNull(x)) return V.Null;
  const n = V.num(x);
  const k = lng(d, 0);
  if (k < 0) V.raise(5);
  const f = Math.pow(10, k);
  const y = Number((n * f).toPrecision(15));
  return bround(y) / f;
};
B.second = (d) => partsOf(V.cdate(d).v).s;
B.sgn = (x) => { const v = V.num(x); return v > 0 ? 1 : v < 0 ? -1 : 0; };
B.shell = () => V.unsupported("Shell");
B.sin = (x) => Math.sin(V.num(x));
B.space = (n) => " ".repeat(Math.max(0, V.clng(n)));
B.split = function (expr, delim, limit, mode) {
  const s = V.cstr(expr);
  if (s === "") return new VArray([[0, -1]], null, null, false);
  const d = miss(delim) ? " " : V.cstr(delim);
  const lim = lng(limit, -1);
  if (d === "") return V.arr(0, [s]);
  const text = cmpMode(mode) === 1;
  const parts = [];
  const hay = text ? s.toLowerCase() : s, needle = text ? d.toLowerCase() : d;
  let i = 0;
  while (lim < 0 || parts.length < lim - 1) {
    const j = hay.indexOf(needle, i);
    if (j < 0) break;
    parts.push(s.slice(i, j));
    i = j + d.length;
  }
  parts.push(s.slice(i));
  return V.arr(0, parts);
};
B.split.__params = ["expression", "delimiter", "limit", "compare"];
B.sqr = (x) => { const v = V.num(x); if (v < 0) V.raise(5); return Math.sqrt(v); };
B.str = (x) => { const v = V.num(x); return (v >= 0 ? " " : "") + fmtNum(v); };
B.strcomp = function (a, b, mode) {
  if (isNull(a) || isNull(b)) return V.Null;
  let x = V.cstr(a), y = V.cstr(b);
  if (cmpMode(mode) === 1) { x = x.toLowerCase(); y = y.toLowerCase(); }
  return x < y ? -1 : x > y ? 1 : 0;
};
B.strconv = function (s, conv) {
  const t = V.cstr(s);
  const c = V.clng(conv) & 3;
  if (c === 3) return t.toLowerCase().replace(/(^|[^a-z0-9'])([a-z])/g, (m, p, l) => p + l.toUpperCase());
  if (c === 1) return t.toUpperCase();
  if (c === 2) return t.toLowerCase();
  return t;
};
B.string = (n, ch) => {
  const k = V.clng(n); if (k < 0) V.raise(5);
  const c = typeof ch === "string" ? ch[0] : String.fromCharCode(V.clng(ch) & 255);
  if (c === undefined) V.raise(5);
  return c.repeat(k);
};
B.strreverse = (s) => V.cstr(s).split("").reverse().join("");
B.switch = function () {
  for (let i = 0; i + 1 < arguments.length; i += 2) if (V.bool(arguments[i])) return arguments[i + 1];
  return V.Null;
};
B.tan = (x) => Math.tan(V.num(x));
B.time = () => { const v = nowSerial(); return new VDate(v - Math.floor(v)); };
B.timer = () => { const d = new Date(); return d.getHours() * 3600 + d.getMinutes() * 60 + d.getSeconds() + d.getMilliseconds() / 1000; };
B.timeserial = (h, m, s) => new VDate((V.num(h) * 3600 + V.num(m) * 60 + V.num(s)) / 86400);
B.timevalue = (s) => { const v = V.cdate(s).v; return new VDate(v - Math.floor(v)); };
B.typename = function (x) {
  if (x === undefined) return "Empty";
  if (x === null) return "Nothing";
  if (x === V.Null) return "Null";
  if (x === V.Missing) return "Error";
  if (typeof x === "string") return "String";
  if (typeof x === "boolean") return "Boolean";
  if (typeof x === "number") {
    if (Number.isInteger(x) && x >= -32768 && x <= 32767) return "Integer";
    if (Number.isInteger(x) && x >= -2147483648 && x <= 2147483647) return "Long";
    return "Double";
  }
  if (x instanceof VDate) return "Date";
  if (x instanceof VArray) return "Variant()";
  if (x instanceof VbError) return "Error";
  if (x instanceof VbObject) return x.__cls;
  if (x instanceof Collection) return "Collection";
  if (x instanceof Dictionary) return "Dictionary";
  if (Array.isArray(x)) return "Variant()";
  const c = Object.getPrototypeOf(x);
  return c && c.constructor && c.constructor.name && c.constructor !== Object ? c.constructor.name : "Object";
};
B.val = function (s) {
  if (typeof s === "number") return s;
  const t = V.cstr(s).replace(/[ \t\n\r]/g, "");
  let m = /^&[hH]([0-9a-fA-F]+)/.exec(t);
  if (m) { const n = parseInt(m[1], 16); return n > 32767 && n <= 65535 && m[1].length <= 4 ? n - 65536 : n; }
  m = /^&[oO]([0-7]+)/.exec(t);
  if (m) return parseInt(m[1], 8);
  m = /^[+-]?(\d+\.?\d*|\.\d+)([eEdD][+-]?\d+)?/.exec(t);
  return m ? parseFloat(m[0].replace(/[dD]/, "e")) : 0;
};
B.vartype = function (x) {
  const t = B.typename(x);
  const map = { Empty: 0, Null: 1, Integer: 2, Long: 3, Double: 5, Date: 7, String: 8, Error: 10, Boolean: 11 };
  if (t in map) return map[t];
  if (x instanceof VArray || Array.isArray(x)) return 8204;
  return 9;
};
B.weekday = (d, first) => {
  const w = partsOf(V.cdate(d).v).w;
  const f = miss(first) || V.clng(first) === 0 ? 1 : V.clng(first);
  return ((w - (f - 1)) % 7 + 7) % 7 + 1;
};
B.weekdayname = (n, abbr, first) => {
  const f = miss(first) || V.clng(first) === 0 ? 1 : V.clng(first);
  const i = (V.clng(n) - 1 + f - 1) % 7;
  const d = DAYS[i]; if (!d) V.raise(5);
  return !miss(abbr) && V.cbool(abbr) ? d.slice(0, 3) : d;
};
B.year = (d) => partsOf(V.cdate(d).v).y;

// Constants.
const C = {
  vbabort: 3, vbabortretryignore: 2, vbapplicationmodal: 0, vbarray: 8192, vbback: "\b",
  vbbinarycompare: 0, vbblack: 0, vbblue: 0xFF0000, vbboolean: 11, vbbyte: 17, vbcancel: 2,
  vbcr: "\r", vbcritical: 16, vbcrlf: "\r\n", vbcurrency: 6, vbcyan: 0xFFFF00,
  vbdatabasecompare: 2, vbdate: 7, vbdecimal: 14, vbdefaultbutton1: 0, vbdefaultbutton2: 256,
  vbdefaultbutton3: 512, vbdefaultbutton4: 768, vbdouble: 5, vbempty: 0, vberror: 10,
  vbexclamation: 48, vbfalse: 0, vbfirstfourdays: 2, vbfirstfullweek: 3, vbfirstjan1: 1,
  vbformfeed: "\f", vbfriday: 6, vbgeneraldate: 0, vbgreen: 0x00FF00, vbignore: 5,
  vbinformation: 64, vbinteger: 2, vblf: "\n", vblong: 3, vblongdate: 1, vblongtime: 3,
  vblowercase: 2, vbmagenta: 0xFF00FF, vbmonday: 2, vbmsgboxhelpbutton: 16384,
  vbmsgboxright: 524288, vbmsgboxrtlreading: 1048576, vbmsgboxsetforeground: 65536,
  vbnewline: "\r\n", vbno: 7, vbnull: 1, vbnullchar: "\0", vbnullstring: "", vbobject: 9,
  vbobjecterror: -2147221504, vbok: 1, vbokcancel: 1, vbokonly: 0, vbpropercase: 3,
  vbquestion: 32, vbred: 0x0000FF, vbretry: 4, vbretrycancel: 5, vbsaturday: 7,
  vbshortdate: 2, vbshorttime: 4, vbsingle: 4, vbstring: 8, vbsunday: 1, vbsystemmodal: 4096,
  vbtab: "\t", vbtextcompare: 1, vbthursday: 5, vbtrue: -1, vbtuesday: 3, vbuppercase: 1,
  vbusedefault: -2, vbusesystem: 0, vbusesystemdayofweek: 0, vbvariant: 12,
  vbverticaltab: "\v", vbwednesday: 4, vbwhite: 0xFFFFFF, vbyellow: 0x00FFFF, vbyes: 6,
  vbyesno: 4, vbyesnocancel: 3,
};
for (const k of Object.keys(C)) B[k] = C[k];

__v = V;
__b = B;
})();
