// The host side of the VBA runtime for macros run by the app: MsgBox and
// InputBox through the app's dialogs, user forms and their controls
// (state kept by the app, see macro_forms.rs), the events the user
// raises, and the check that ends a stopped macro. Evaluated after the
// object model and the VBA runtime, before the program.
(function () {
"use strict";
const G = globalThis;

// Every 256 statements: has the user stopped the macro?
let ticks = 0;
G.__chk = function () {
  if ((++ticks & 255) === 0 && __stop()) throw __v.END;
};

G.__host_msgbox = function (prompt, buttons, title) {
  const r = __mb(String(prompt), Number(buttons) || 0, String(title || ""));
  if (r === "stop") throw __v.END;
  return r;
};
G.__host_inputbox = function (prompt, title, def) {
  const r = __ib(String(prompt), String(title || ""), String(def === undefined ? "" : def));
  if (r === "stop") throw __v.END;
  return r;
};

// GetSetting and SaveSetting: kept by the app with the macros.
G.__host_setting = function (op, app, section, key, value) {
  const r = __setting(op, app, section, key, value === null || value === undefined ? "" : value);
  return op === "all" ? (r ? JSON.parse(r) : []) : r;
};

// The macro manager of the drawing suites macros are written for: where
// macro projects live, as far as macros ask.
if (typeof G.GMSManager === "undefined") {
  G.GMSManager = {
    get UserGMSPath() { return __folder(); },
    get GMSPath() { return __folder(); },
  };
}

function parse(s) {
  return s === undefined || s === null ? undefined : JSON.parse(s);
}

// A VBA value as JSON for the app: arrays (one or two dimensions) as
// lists, dates and objects as text.
function plain(v) {
  if (v === undefined || v === null || v === __v.Missing || v === __v.Null) return null;
  if (typeof v === "boolean" || typeof v === "number" || typeof v === "string") return v;
  if (Array.isArray(v)) return v.map(plain);
  if (typeof v === "object" && Array.isArray(v.dims) && typeof v.items === "function") {
    const items = v.items();
    if (v.dims.length === 2) {
      const rows = v.dims[0][1] - v.dims[0][0] + 1, cols = v.dims[1][1] - v.dims[1][0] + 1;
      const out = [];
      for (let r = 0; r < rows; r++) {
        const row = [];
        for (let c = 0; c < cols; c++) row.push(plain(items[r + c * rows]));
        out.push(row);
      }
      return out;
    }
    return items.map(plain);
  }
  try { return __v.cstr(v); } catch (e) { return String(v); }
}

const forms = {};
G.__forms = forms;

function fire(form, control, event, args) {
  if (form && typeof form.__event === "function") form.__event(control, event, args || []);
}

// Set a property and fire what VBA fires for it.
function setProp(form, name, prop, v) {
  const o = parse(__fset(form.__id, name, prop, JSON.stringify(plain(v))));
  if (!o) return;
  for (const off of o.off || []) fire(form, off, "Change", []);
  if (o.changed) fire(form, name, "Change", []);
  if (o.click) fire(form, name, "Click", []);
}

function indexed(get, set) {
  return {
    Item(...a) { return get(a); },
    __setItem(a, v) { set(a, v); },
  };
}

class Ctl {
  constructor(form, name, kind) {
    Object.defineProperty(this, "__f", { value: form });
    Object.defineProperty(this, "__n", { value: name });
    Object.defineProperty(this, "__k", { value: kind });
    // A control assigned by its name sets its Value.
    Object.defineProperty(this, "__default", { value: "Value" });
  }
  get Font() { return fontOf(this.__f, this.__n); }
  get List() {
    const f = this.__f, n = this.__n;
    return indexed(
      (a) => parse(__fcall(f.__id, n, "GetList", JSON.stringify(a.map(plain)))),
      (a, v) => { __fcall(f.__id, n, "SetList", JSON.stringify(a.map(plain).concat([plain(v)]))); }
    );
  }
  set List(v) { setProp(this.__f, this.__n, "List", v); }
  get Column() {
    const f = this.__f, n = this.__n;
    return indexed(
      (a) => parse(__fcall(f.__id, n, "GetList", JSON.stringify([plain(a[1]), plain(a[0])]))),
      (a, v) => { __fcall(f.__id, n, "SetList", JSON.stringify([plain(a[1]), plain(a[0]), plain(v)])); }
    );
  }
  get Selected() {
    const f = this.__f, n = this.__n;
    return indexed(
      (a) => parse(__fcall(f.__id, n, "GetSelected", JSON.stringify(a.map(plain)))),
      (a, v) => {
        __fcall(f.__id, n, "SetSelected", JSON.stringify([plain(a[0]), plain(v)]));
        fire(f, n, "Change", []);
      }
    );
  }
  get Pages() {
    const f = this.__f, n = this.__n;
    const names = parse(__fcall(f.__id, n, "Pages", "[]")) || [];
    const pages = names.map((p) => f.__controls[p] || new Ctl(f, p, "Page"));
    return {
      get Count() { return pages.length; },
      Item(i) {
        if (typeof i === "string") return pages.find((p) => p.__n.toLowerCase() === i.toLowerCase());
        const p = pages[Number(i)];
        if (p === undefined) __v.raise(9);
        return p;
      },
      [Symbol.iterator]() { return pages[Symbol.iterator](); },
    };
  }
  get SelectedItem() { return this.Pages.Item(this.Value); }
  get Object() { return this; }
  get Parent() { return this.__f; }
  AddItem(item, index) {
    __fcall(this.__f.__id, this.__n, "AddItem", JSON.stringify(index === undefined ? [plain(item)] : [plain(item), plain(index)]));
  }
  RemoveItem(index) { __fcall(this.__f.__id, this.__n, "RemoveItem", JSON.stringify([plain(index)])); }
  Clear() { __fcall(this.__f.__id, this.__n, "Clear", "[]"); }
  SetFocus() { __fcall(this.__f.__id, this.__n, "SetFocus", "[]"); }
  Repaint() {}
  Move() {}
  ZOrder() {}
}
Ctl.prototype.AddItem.__params = ["pvargitem", "pvargindex"];
Ctl.prototype.RemoveItem.__params = ["pvargindex"];

const PROPS = ["Caption", "Value", "Text", "Enabled", "Visible", "Locked", "Tag", "ControlTipText",
  "GroupName", "Left", "Top", "Width", "Height", "ListIndex", "ListCount", "ColumnCount",
  "MultiSelect", "Style", "Min", "Max", "SmallChange", "LargeChange", "MaxLength", "MultiLine",
  "PasswordChar", "Default", "Cancel", "TabIndex", "TabStop", "BackColor", "ForeColor", "SelText",
  "SelStart", "SelLength", "SpecialEffect", "BorderStyle", "TextAlign", "AutoSize", "WordWrap",
  "BoundColumn", "TextColumn", "ColumnWidths", "ColumnHeads", "MatchEntry", "ShowDropButtonWhen",
  "Picture", "MousePointer", "Name"];
for (const p of PROPS) {
  Object.defineProperty(Ctl.prototype, p, {
    get() { return parse(__fget(this.__f.__id, this.__n, p)); },
    set(v) { setProp(this.__f, this.__n, p, v); },
    configurable: true,
  });
}

function fontOf(form, name) {
  const o = {};
  for (const p of ["Name", "Size", "Bold", "Italic", "Underline"]) {
    Object.defineProperty(o, p, {
      get() { return parse(__fget(form.__id, name, "Font." + p)); },
      set(v) { __fset(form.__id, name, "Font." + p, JSON.stringify(plain(v))); },
      enumerable: true,
    });
  }
  return o;
}

function controlsOf(form) {
  const list = Object.keys(form.__controls).map((k) => form.__controls[k]);
  return {
    get Count() { return list.length; },
    Item(i) {
      if (typeof i === "string") {
        const c = form.__ctl(i.toLowerCase());
        if (c === undefined) __v.raise(5, i);
        return c;
      }
      const c = list[Number(i)];
      if (c === undefined) __v.raise(9);
      return c;
    },
    [Symbol.iterator]() { return list[Symbol.iterator](); },
  };
}

G.__host_form_create = function (form) {
  const info = parse(__fnew(String(form.__cls)));
  Object.defineProperty(form, "__id", { value: info.id, writable: true });
  // Every property goes to the app (the runtime's own store would hide
  // what the app changes).
  Object.defineProperty(form, "__props", { value: {}, writable: true });
  for (const c of info.controls) form.__controls[c.name] = new Ctl(form, c.name, c.kind);
  forms[info.id] = form;
};
G.__host_form_get = function (form, name) {
  const l = String(name).toLowerCase();
  if (l === "controls") return controlsOf(form);
  if (l === "font") return fontOf(form, "");
  if (l === "activecontrol") {
    const n = parse(__fget(form.__id, "", "ActiveControl"));
    return n ? form.__ctl(String(n).toLowerCase()) : null;
  }
  return parse(__fget(form.__id, "", name));
};
G.__host_form_set = function (form, name, v) {
  delete form.__props[String(name).toLowerCase()];
  __fset(form.__id, "", name, JSON.stringify(plain(v)));
};
G.__host_form_show = function (form, modal) {
  __fshow(form.__id, !!modal);
  fire(form, "UserForm", "Activate", []);
  if (modal) pump(form.__id, false);
};
G.__host_form_hide = function (form) { __fhide(form.__id); };
G.__host_form_unload = function (form) {
  __funload(form.__id);
  delete forms[form.__id];
};

// An event the user raised: run the macro's handler for it.
function dispatch(form, e) {
  const args = e.args || [];
  if (e.control === "") {
    if (e.event === "QueryClose") {
      // The close box: the handler may cancel; else the form unloads.
      let cancel = 0;
      fire(form, "UserForm", "QueryClose", [__v.ref(() => cancel, (v) => { cancel = v; }), args[0] || 0]);
      if (!__v.bool(cancel) && forms[form.__id] === form) __v.unload(form);
      return;
    }
    fire(form, "UserForm", e.event, args);
    return;
  }
  if (e.event === "Exit" || e.event === "DblClick" || e.event === "BeforeUpdate") {
    let cancel = 0;
    fire(form, e.control, e.event, [__v.ref(() => cancel, (v) => { cancel = v; })]);
    return;
  }
  if (e.event === "KeyPress") {
    let key = args[0] || 0;
    fire(form, e.control, e.event, [__v.ref(() => key, (v) => { key = v; })]);
    return;
  }
  if (e.event === "KeyDown" || e.event === "KeyUp") {
    let key = args[0] || 0;
    fire(form, e.control, e.event, [__v.ref(() => key, (v) => { key = v; }), args[1] || 0]);
    return;
  }
  fire(form, e.control, e.event, args);
}

// Wait for the user: events of form `id` (any form for 0) until it is no
// longer on screen. `top`: after the macro's procedure returned (a form
// shown modeless), each handler runs as a macro of its own.
function pump(id, top) {
  for (;;) {
    const r = __fwait(id);
    if (r === "stop") throw __v.END;
    if (r === null || r === undefined) return;
    const e = JSON.parse(r);
    const form = forms[e.form];
    if (!form) continue;
    if (top) __v.run(() => dispatch(form, e));
    else dispatch(form, e);
  }
}
G.__pump = pump;
})();
