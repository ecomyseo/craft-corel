//! VBA macros for TraceDraw: a compiler from VBA modules to JavaScript
//! that runs on the app's script engine, over the same object model as
//! its scripts. The generated program needs [`RUNTIME_JS`] (the VBA
//! built-ins and the helpers that give member access, calls, errors and
//! conversions VBA's meaning) evaluated first, after the host's object
//! model.
//!
//! ```text
//! host object model  ->  RUNTIME_JS  ->  Program::js  ->  entry_call(..)
//! ```
//!
//! The host may provide `__host_msgbox(text, buttons, title)`,
//! `__host_inputbox(prompt, title, default)` and, for user forms,
//! `__host_form_create(form)`, `__host_form_show(form, modal)`,
//! `__host_form_hide(form)`, `__host_form_unload(form)`,
//! `__host_form_get(form, name)` and `__host_form_set(form, name, value)`.
//! See docs/behavior/vba.md.

mod ast;
mod builtins;
mod codegen;
mod lexer;
mod parser;
mod preprocess;

pub use builtins::{BUILTIN_CONSTANTS, BUILTIN_FUNCTIONS};

/// The JavaScript runtime the compiled programs call into.
pub const RUNTIME_JS: &str = include_str!("runtime.js");

/// What kind of module a source is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleKind {
    /// A standard module (`.bas`).
    Standard,
    /// A class module (`.cls`).
    Class,
    /// A user form's code (`.frm`); its controls come from the host.
    Form,
    /// A document module (the project's own document): compiled as a
    /// standard module.
    Document,
}

/// One module of a VBA project.
#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub name: String,
    pub kind: ModuleKind,
    pub source: String,
}

impl Module {
    pub fn new(name: impl Into<String>, kind: ModuleKind, source: impl Into<String>) -> Self {
        Module {
            name: name.into(),
            kind,
            source: source.into(),
        }
    }
}

/// A compile error: the module, the VBA line (1-based) and the message.
#[derive(Debug, Clone, PartialEq)]
pub struct CompileError {
    pub module: String,
    pub line: u32,
    pub message: String,
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, line {}: {}", self.module, self.line, self.message)
    }
}

impl std::error::Error for CompileError {}

/// What a procedure is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcKind {
    Sub,
    Function,
    PropertyGet,
    PropertyLet,
    PropertySet,
}

/// A procedure of the compiled program.
#[derive(Debug, Clone, PartialEq)]
pub struct ProcInfo {
    pub module: String,
    pub name: String,
    pub kind: ProcKind,
    pub is_public: bool,
    pub params: Vec<String>,
}

/// A line of the generated JavaScript and the VBA line it runs.
#[derive(Debug, Clone, PartialEq)]
pub struct LineMapEntry {
    /// 1-based line in [`Program::js`].
    pub js_line: u32,
    pub module: String,
    /// 1-based line in the module's source.
    pub line: u32,
}

/// A compiled VBA project.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub js: String,
    pub procedures: Vec<ProcInfo>,
    pub line_map: Vec<LineMapEntry>,
}

impl Program {
    /// The VBA module and line of a line of [`Program::js`].
    pub fn vba_line(&self, js_line: u32) -> Option<(&str, u32)> {
        self.line_map
            .iter()
            .rev()
            .find(|e| e.js_line <= js_line)
            .map(|e| (e.module.as_str(), e.line))
    }
}

/// Parse one module.
fn parse(m: &Module) -> Result<ast::ModuleAst, CompileError> {
    let lines = preprocess::preprocess(&m.source);
    let toks = lexer::lex(&m.name, &lines)?;
    parser::Parser::new(&m.name, toks).module()
}

/// What the compiler may know about the host: the global names its
/// object model defines and the controls of each form. An undeclared name
/// a procedure assigns is then the host's or the control (as the
/// reference does) instead of a new variable.
#[derive(Debug, Clone, Default)]
pub struct Options {
    pub host_names: Vec<String>,
    /// (form module name, its control names).
    pub form_controls: Vec<(String, Vec<String>)>,
}

/// Compile a project's modules into one JavaScript program.
pub fn compile(modules: &[Module]) -> Result<Program, CompileError> {
    compile_with(modules, &Options::default())
}

/// [`compile`] knowing the host's names and the forms' controls.
pub fn compile_with(modules: &[Module], opts: &Options) -> Result<Program, CompileError> {
    let mut asts = Vec::new();
    for m in modules {
        asts.push(parse(m)?);
    }
    codegen::generate(modules, &asts, opts)
}

/// The JavaScript that runs one macro (a procedure without arguments) of
/// a compiled program. Run-time errors come out as a JavaScript `Error`
/// whose message reads "Run-time error 'n': description (Module.Proc,
/// line l)"; `End` stops quietly.
pub fn entry_call(program: &Program, module: &str, procedure: &str) -> String {
    let found = program
        .procedures
        .iter()
        .find(|p| p.module.eq_ignore_ascii_case(module) && p.name.eq_ignore_ascii_case(procedure));
    match found {
        Some(p) => format!(
            "__v.run(function(){{ return {}(); }})",
            codegen::proc_js_name(&p.module, &p.name, p.kind)
        ),
        None => format!(
            "__v.run(function(){{ __v.raise(35, {}); }})",
            js_string(&format!("{module}.{procedure}"))
        ),
    }
}

/// The macros a project offers: public Subs without parameters of its
/// standard and document modules (not in modules with
/// `Option Private Module`), as (module, procedure). Modules that do not
/// parse are left out.
pub fn list_macros(modules: &[Module]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for m in modules {
        if !matches!(m.kind, ModuleKind::Standard | ModuleKind::Document) {
            continue;
        }
        let Ok(a) = parse(m) else {
            continue;
        };
        if a.option_private {
            continue;
        }
        for p in &a.procs {
            if p.kind == ast::ProcKind::Sub && p.is_public && p.params.is_empty() {
                out.push((m.name.clone(), p.name.clone()));
            }
        }
    }
    out
}

/// A JavaScript string literal.
pub(crate) fn js_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
