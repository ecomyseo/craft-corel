//! Runs compiled VBA on boa with a small fake object model.

use boa_engine::{Context, Source};
use tracedraw_vba::{compile, entry_call, Module, ModuleKind, RUNTIME_JS};

/// A drawing with shapes, enough for macros that walk the selection.
pub const FAKE_MODEL: &str = r##"
var __lines = [];
function print(s) { __lines.push(String(s)); }
var __msgs = [];
class Color {
  constructor() { this.r = 0; this.g = 0; this.b = 0; }
  RGBAssign(r, g, b) { this.r = r; this.g = g; this.b = b; }
  get HexValue() { return "#" + [this.r, this.g, this.b].map(v => v.toString(16).padStart(2, "0")).join(""); }
}
class Fill {
  constructor() { this.UniformColor = new Color(); this.Type = 0; }
  ApplyNoFill() { this.Type = 0; }
  ApplyUniformFill(c) { this.Type = 1; this.UniformColor = c; }
}
class Shape {
  constructor(name, x, y, w, h) { this.Name = name; this.PositionX = x; this.PositionY = y; this.SizeWidth = w; this.SizeHeight = h; this.Fill = new Fill(); this.deleted = false; }
  Move(dx, dy) { this.PositionX += dx; this.PositionY += (dy || 0); }
  SetSize(w, h) { this.SizeWidth = w; this.SizeHeight = h; }
  Delete() { this.deleted = true; }
  Duplicate(dx, dy) { const s = new Shape(this.Name + " copy", this.PositionX + (dx||0), this.PositionY + (dy||0), this.SizeWidth, this.SizeHeight); __shapes.push(s); return s; }
  get Type() { return 1; }
}
class Shapes {
  constructor(list) { this.list = list; }
  get Count() { return this.list.length; }
  Item(i) { if (typeof i === "string") return this.list.find(s => s.Name === i) || null; return this.list[i - 1]; }
}
class ShapeRange {
  constructor(list) { this.list = list; this.Shapes = new Shapes(list); }
  get Count() { return this.list.length; }
}
var __shapes = [new Shape("A", 0, 0, 10, 10), new Shape("B", 20, 0, 5, 5), new Shape("C", 40, 0, 8, 2)];
class Layer {
  get Shapes() { return new Shapes(__shapes); }
  CreateRectangle(x1, y1, x2, y2) { const s = new Shape("Rect" + (__shapes.length + 1), x1, y1, x2 - x1, y2 - y1); __shapes.push(s); return s; }
}
class Document {
  constructor() { this.Unit = 1; }
  get ActiveLayer() { return new Layer(); }
  get SelectionRange() { return new ShapeRange(__shapes.slice(0, 2)); }
  CreateColor() { return new Color(); }
}
const ActiveDocument = new Document();
const ActiveSelection = new ShapeRange(__shapes.slice(0, 2));
const ActiveLayer = new Layer();
function CreateRGBColor(r, g, b) { const c = new Color(); c.RGBAssign(r, g, b); return c; }
const cdrMillimeter = 3;
var __host_names = ["ActiveDocument", "ActiveSelection", "ActiveLayer", "CreateRGBColor", "cdrMillimeter"];
"##;

pub struct Run {
    pub lines: Vec<String>,
    pub result: Result<(), String>,
}

/// Compile the modules and run `module.proc`; the lines printed and how it
/// ended.
pub fn run_modules(modules: &[Module], module: &str, proc: &str, extra_js: &str) -> Run {
    let program = compile(modules).unwrap_or_else(|e| panic!("compile error: {e}"));
    let mut ctx = Context::default();
    ctx.eval(Source::from_bytes(FAKE_MODEL)).expect("model");
    ctx.eval(Source::from_bytes(extra_js)).expect("extra js");
    ctx.eval(Source::from_bytes(RUNTIME_JS)).expect("runtime");
    if let Err(e) = ctx.eval(Source::from_bytes(&program.js)) {
        panic!("program: {e}\n{}", program.js);
    }
    let call = entry_call(&program, module, proc);
    let result = ctx
        .eval(Source::from_bytes(&call))
        .map(|_| ())
        .map_err(|e| e.to_string());
    let lines = ctx
        .eval(Source::from_bytes("__lines.join('\\u0001')"))
        .expect("lines");
    let text = lines
        .as_string()
        .map(|s| s.to_std_string_escaped())
        .unwrap_or_default();
    let lines = if text.is_empty() {
        Vec::new()
    } else {
        text.split('\u{1}').map(str::to_string).collect()
    };
    Run { lines, result }
}

/// One standard module: run `Main` and return what it printed; panics on
/// a run-time error.
pub fn run(src: &str) -> Vec<String> {
    let r = run_modules(
        &[Module::new("Module1", ModuleKind::Standard, src)],
        "Module1",
        "Main",
        "",
    );
    if let Err(e) = r.result {
        let program = compile(&[Module::new("Module1", ModuleKind::Standard, src)]).unwrap();
        panic!("run-time error: {e}\n{}", program.js);
    }
    r.lines
}

/// Run `Main` expecting a run-time error; returns its message.
pub fn run_err(src: &str) -> String {
    let r = run_modules(
        &[Module::new("Module1", ModuleKind::Standard, src)],
        "Module1",
        "Main",
        "",
    );
    r.result.expect_err("expected a run-time error")
}
