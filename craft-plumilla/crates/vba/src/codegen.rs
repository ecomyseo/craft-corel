//! Code generation: VBA syntax trees to one JavaScript program over the
//! runtime in `runtime.js` (`__v`: helpers, `__b`: the VBA library).
//!
//! Names: standard module variables `$v_mod_name`, constants
//! `$k_mod_name`, enum members `$e_mod_name`, procedures `$p_mod_name`
//! (property procedures `$pg_`, `$pl_`, `$ps_`), statics
//! `$s_mod_proc_name`, user types `$T_mod_name`, classes `$C_mod`, forms
//! `$F_mod`; locals `$l_name`, a function's result `$r`. Class and form
//! members are `this.$name`, their property procedures `__get_name`,
//! `__let_name` and `__set_name`. Names are lower case: VBA ignores case.

use crate::ast::*;
use crate::{js_string, CompileError, LineMapEntry, Module, ModuleKind, ProcInfo, Program};
use std::collections::HashMap;

type R<T> = Result<T, CompileError>;

fn low(s: &str) -> String {
    s.to_lowercase()
}

/// The JavaScript name of a standard module's procedure.
pub fn proc_js_name(module: &str, name: &str, kind: crate::ProcKind) -> String {
    let prefix = match kind {
        crate::ProcKind::Sub | crate::ProcKind::Function => "$p_",
        crate::ProcKind::PropertyGet => "$pg_",
        crate::ProcKind::PropertyLet => "$pl_",
        crate::ProcKind::PropertySet => "$ps_",
    };
    format!("{prefix}{}_{}", low(module), low(name))
}

fn public_kind(k: ProcKind) -> crate::ProcKind {
    match k {
        ProcKind::Sub => crate::ProcKind::Sub,
        ProcKind::Function => crate::ProcKind::Function,
        ProcKind::PropertyGet => crate::ProcKind::PropertyGet,
        ProcKind::PropertyLet => crate::ProcKind::PropertyLet,
        ProcKind::PropertySet => crate::ProcKind::PropertySet,
    }
}

/// JavaScript words and globals an unknown VBA name must not be read as
/// directly (it is looked up on the global object instead).
const JS_BLOCK: &[&str] = &[
    "arguments",
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "eval",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "implements",
    "import",
    "in",
    "instanceof",
    "interface",
    "let",
    "new",
    "null",
    "package",
    "private",
    "protected",
    "public",
    "return",
    "static",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
    "undefined",
    "NaN",
    "Infinity",
    "globalThis",
    "Object",
    "Function",
    "Array",
    "Number",
    "String",
    "Boolean",
    "Symbol",
    "Math",
    "JSON",
    "Date",
    "RegExp",
    "Error",
    "Map",
    "Set",
    "Promise",
    "Proxy",
    "Reflect",
    "print",
    "console",
    "Intl",
    "parseInt",
    "parseFloat",
    "isNaN",
    "isFinite",
    "escape",
    "unescape",
    "of",
    "get",
];

#[derive(Clone)]
struct Local {
    js: String,
    ty: Option<TypeRef>,
    is_const: bool,
}

#[derive(Clone)]
enum Res {
    Var {
        js: String,
        ty: Option<TypeRef>,
    },
    Const(String),
    /// A Sub or Function of a standard module.
    Proc(usize, usize),
    /// A Sub or Function of the class being compiled.
    OwnMethod(usize),
    /// A property of a standard module (module, lower name).
    StdProp(usize, String),
    /// A property of the class being compiled (lower name).
    OwnProp(String),
    Enum(String),
    EnumType(usize, usize),
    Module(usize),
    Declared(String),
    BuiltinFn(String),
    BuiltinConst(String),
    Err,
    Unknown,
}

#[derive(Clone, Copy, PartialEq)]
enum LoopKind {
    For,
    Do,
    While,
}

struct Ctx {
    mi: usize,
    proc_name: String,
    proc_lname: Option<String>,
    ret_ty: Option<TypeRef>,
    is_sub: bool,
    locals: HashMap<String, Local>,
    withs: Vec<String>,
    loops: Vec<(LoopKind, String)>,
    labels: HashMap<String, usize>,
    /// Statement lists inside the procedure that hold labels (a loop's
    /// body with `Skip:` before `Next`...), innermost last: their state
    /// machine's number and their labels.
    nested: Vec<(u32, HashMap<String, usize>)>,
    onerror: bool,
    sm: bool,
    top: usize,
    line: u32,
}

impl Ctx {
    fn module_scope(mi: usize) -> Ctx {
        Ctx {
            mi,
            proc_name: String::new(),
            proc_lname: None,
            ret_ty: None,
            is_sub: true,
            locals: HashMap::new(),
            withs: Vec::new(),
            loops: Vec::new(),
            labels: HashMap::new(),
            nested: Vec::new(),
            onerror: false,
            sm: false,
            top: 0,
            line: 0,
        }
    }
}

struct Gen<'a> {
    modules: &'a [Module],
    asts: &'a [ModuleAst],
    /// Names the host defines (lower case) and the controls of each form
    /// module (by module index, lower case): an undeclared name that is
    /// one of them is the host's or the control, not a new variable.
    host_names: std::collections::HashSet<String>,
    form_controls: HashMap<usize, std::collections::HashSet<String>>,
    lnames: Vec<String>,
    tmp: u32,
    statics: String,
}

pub fn generate(modules: &[Module], asts: &[ModuleAst], opts: &crate::Options) -> R<Program> {
    let mut g = Gen {
        modules,
        asts,
        host_names: opts.host_names.iter().map(|n| low(n)).collect(),
        form_controls: modules
            .iter()
            .enumerate()
            .filter_map(|(i, m)| {
                opts.form_controls
                    .iter()
                    .find(|(f, _)| f.eq_ignore_ascii_case(&m.name))
                    .map(|(_, cs)| (i, cs.iter().map(|c| low(c)).collect()))
            })
            .collect(),
        lnames: modules.iter().map(|m| low(&m.name)).collect(),
        tmp: 0,
        statics: String::new(),
    };
    let mut out = String::from("\"use strict\";\n");
    let mut procedures = Vec::new();
    // User types first: variables and fields may hold them.
    for (mi, a) in asts.iter().enumerate() {
        for t in &a.types {
            out.push_str(&g.user_type(mi, t)?);
        }
    }
    for mi in 0..modules.len() {
        out.push_str(&g.enums(mi)?);
    }
    for mi in 0..modules.len() {
        out.push_str(&g.consts(mi)?);
    }
    for (mi, m) in modules.iter().enumerate() {
        for p in &asts[mi].procs {
            procedures.push(ProcInfo {
                module: m.name.clone(),
                name: p.name.clone(),
                kind: public_kind(p.kind),
                is_public: p.is_public,
                params: p.params.iter().map(|p| p.name.clone()).collect(),
            });
        }
    }
    // Classes and forms first: a standard module's variable declared
    // `As New SomeClass` makes one when the program loads.
    for (mi, m) in modules.iter().enumerate() {
        if matches!(m.kind, ModuleKind::Class | ModuleKind::Form) {
            out.push_str(&g.class(mi)?);
        }
    }
    for (mi, m) in modules.iter().enumerate() {
        if matches!(m.kind, ModuleKind::Standard | ModuleKind::Document) {
            out.push_str(&g.standard(mi)?);
        }
    }
    out.push_str(&g.statics);
    let js = out;
    let line_map = line_map(&js);
    Ok(Program {
        js,
        procedures,
        line_map,
    })
}

/// Lines that set `__f.l` (the running VBA line) after a module marker.
fn line_map(js: &str) -> Vec<LineMapEntry> {
    let mut out = Vec::new();
    let mut module = String::new();
    for (i, line) in js.lines().enumerate() {
        if let Some(p) = line.find("/*@mod:") {
            let rest = &line[p + 7..];
            if let Some(e) = rest.find("*/") {
                module = rest[..e].to_string();
            }
        }
        if let Some(p) = line.find("__f.l=") {
            let digits: String = line[p + 6..]
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if let Ok(n) = digits.parse() {
                out.push(LineMapEntry {
                    js_line: i as u32 + 1,
                    module: module.clone(),
                    line: n,
                });
            }
        }
    }
    out
}

fn num_lit(n: f64) -> String {
    if n.is_finite() {
        if n == n.trunc() && n.abs() < 1e15 {
            format!("{}", n as i64)
        } else {
            format!("{n:?}")
        }
    } else {
        "NaN".into()
    }
}

fn is_js_ident(s: &str) -> bool {
    let mut cs = s.chars();
    match cs.next() {
        Some(c) if c.is_alphabetic() || c == '_' => {}
        _ => return false,
    }
    cs.all(|c| c.is_alphanumeric() || c == '_')
}

impl<'a> Gen<'a> {
    fn err<T>(&self, cx: &Ctx, msg: impl Into<String>) -> R<T> {
        Err(CompileError {
            module: self.modules[cx.mi].name.clone(),
            line: cx.line,
            message: msg.into(),
        })
    }

    fn ast(&self, mi: usize) -> &'a ModuleAst {
        let asts: &'a [ModuleAst] = self.asts;
        &asts[mi]
    }

    fn kind(&self, mi: usize) -> ModuleKind {
        self.modules[mi].kind
    }

    fn is_class(&self, mi: usize) -> bool {
        matches!(self.kind(mi), ModuleKind::Class | ModuleKind::Form)
    }

    fn tmp(&mut self, p: &str) -> String {
        self.tmp += 1;
        format!("__{p}{}", self.tmp)
    }

    // ----- Types and defaults -------------------------------------------

    /// A user type visible from module `mi`.
    fn find_type(&self, mi: usize, ty: &TypeRef) -> Option<(usize, usize)> {
        let key = ty.key();
        let qualifier = ty.name.rsplit_once('.').map(|(q, _)| low(q));
        let own = self.ast(mi).types.iter().position(|t| low(&t.name) == key);
        if let Some(t) = own {
            if qualifier.is_none() || qualifier.as_deref() == Some(&self.lnames[mi]) {
                return Some((mi, t));
            }
        }
        for (m, a) in self.asts.iter().enumerate() {
            if let Some(q) = &qualifier {
                if q != &self.lnames[m] {
                    continue;
                }
            }
            if let Some(t) = a
                .types
                .iter()
                .position(|t| t.is_public && low(&t.name) == key)
            {
                return Some((m, t));
            }
        }
        None
    }

    fn class_module(&self, name: &str) -> Option<usize> {
        let key = low(name.rsplit('.').next().unwrap_or(name));
        (0..self.modules.len()).find(|&m| self.is_class(m) && self.lnames[m] == key)
    }

    fn scalar_default(key: &str) -> Option<&'static str> {
        Some(match key {
            "integer" | "long" | "byte" | "single" | "double" | "currency" | "decimal"
            | "longlong" | "longptr" => "0",
            "string" => "\"\"",
            "boolean" => "false",
            "date" => "__v.mkdate(0)",
            "variant" => "undefined",
            _ => return None,
        })
    }

    /// The value a variable of this type starts with (not an array).
    fn default_for(&self, mi: usize, ty: Option<&TypeRef>, is_new: bool) -> String {
        let Some(ty) = ty else {
            return "undefined".into();
        };
        let key = ty.key();
        if let Some(d) = Self::scalar_default(&key) {
            return d.into();
        }
        if let Some((m, t)) = self.find_type(mi, ty) {
            return format!(
                "new $T_{}_{}()",
                self.lnames[m],
                low(&self.asts[m].types[t].name)
            );
        }
        if is_new {
            return self.new_expr(&ty.name);
        }
        "null".into()
    }

    /// The function that converts a value assigned to this type.
    fn coerce_fn(&self, mi: usize, ty: Option<&TypeRef>) -> Option<&'static str> {
        let Some(ty) = ty else {
            return Some("__v.letv");
        };
        Some(match ty.key().as_str() {
            "integer" => "__v.cint",
            "long" | "longlong" | "longptr" => "__v.clng",
            "byte" => "__v.cbyte",
            "single" => "__v.csng",
            "double" | "decimal" => "__v.cdbl",
            "currency" => "__v.ccur",
            "string" => "__v.cstr",
            "boolean" => "__v.cbool",
            "date" => "__v.cdate",
            "variant" => "__v.letv",
            _ => {
                if self.find_type(mi, ty).is_some() {
                    "__v.letv"
                } else {
                    return None;
                }
            }
        })
    }

    fn coerce(&self, mi: usize, ty: Option<&TypeRef>, js: String) -> String {
        match self.coerce_fn(mi, ty) {
            Some(f) => format!("{f}({js})"),
            None => js,
        }
    }

    fn new_expr(&self, name: &str) -> String {
        let key = low(name.rsplit('.').next().unwrap_or(name));
        if let Some(m) = self.class_module(name) {
            let prefix = if self.kind(m) == ModuleKind::Form {
                "$F_"
            } else {
                "$C_"
            };
            return format!("new {prefix}{}()", self.lnames[m]);
        }
        match key.as_str() {
            "collection" => "new __v.Collection()".into(),
            "dictionary" => "new __v.Dictionary()".into(),
            _ => {
                let last = name.rsplit('.').next().unwrap_or(name);
                let value = if is_js_ident(last) && !JS_BLOCK.contains(&last) {
                    format!("(typeof {last}!==\"undefined\"?{last}:undefined)")
                } else {
                    "undefined".into()
                };
                format!("__v.newHost({value},{})", js_string(name))
            }
        }
    }

    /// An array variable: its bounds (or none for a dynamic array).
    fn array_init(
        &mut self,
        cx: &mut Ctx,
        bounds: &[Bound],
        ty: Option<&TypeRef>,
        fixed: bool,
    ) -> R<String> {
        let dims = if bounds.is_empty() {
            "null".to_string()
        } else {
            self.dims(cx, bounds)?
        };
        let def = self.default_for(cx.mi, ty, false);
        let co = self
            .coerce_fn(cx.mi, ty)
            .filter(|f| *f != "__v.letv")
            .unwrap_or("null");
        Ok(format!(
            "new __v.VArray({dims},()=>{def},{co},{})",
            fixed && !bounds.is_empty()
        ))
    }

    fn dims(&mut self, cx: &mut Ctx, bounds: &[Bound]) -> R<String> {
        let base = self.ast(cx.mi).option_base;
        let mut parts = Vec::new();
        for b in bounds {
            let lo = match &b.lower {
                Some(e) => self.expr(cx, e)?,
                None => base.to_string(),
            };
            let hi = self.expr(cx, &b.upper)?;
            parts.push(format!("[__v.clng({lo}),__v.clng({hi})]"));
        }
        Ok(format!("[{}]", parts.join(",")))
    }

    fn var_init(&mut self, cx: &mut Ctx, d: &VarDecl) -> R<String> {
        match &d.bounds {
            Some(b) => self.array_init(cx, b, d.ty.as_ref(), true),
            None => Ok(self.default_for(cx.mi, d.ty.as_ref(), d.is_new)),
        }
    }

    // ----- Module-level declarations ------------------------------------

    fn user_type(&mut self, mi: usize, t: &UserType) -> R<String> {
        let mut cx = Ctx::module_scope(mi);
        let name = format!("$T_{}_{}", self.lnames[mi], low(&t.name));
        let mut s = format!(
            "class {name} extends __v.VbRecord {{ constructor(){{ super({}); ",
            js_string(&t.name)
        );
        for f in &t.fields {
            let init = self.var_init(&mut cx, f)?;
            s.push_str(&format!("this.${}={init}; ", low(&f.name)));
        }
        s.push_str("} }\n");
        Ok(s)
    }

    fn enums(&mut self, mi: usize) -> R<String> {
        let mut s = String::new();
        let mut cx = Ctx::module_scope(mi);
        for e in &self.ast(mi).enums {
            let mut prev = String::from("-1");
            for (n, v) in &e.members {
                let js = format!("$e_{}_{}", self.lnames[mi], low(n));
                let value = match v {
                    Some(v) => self.expr(&mut cx, v)?,
                    None => format!("({prev}+1)"),
                };
                s.push_str(&format!("var {js}={value};\n"));
                prev = js;
            }
        }
        Ok(s)
    }

    fn consts(&mut self, mi: usize) -> R<String> {
        let mut s = String::new();
        let mut cx = Ctx::module_scope(mi);
        for c in &self.ast(mi).consts {
            cx.line = c.line;
            let v = self.expr(&mut cx, &c.value)?;
            let v = self.coerce(mi, c.ty.as_ref(), v);
            s.push_str(&format!(
                "var $k_{}_{}={v};\n",
                self.lnames[mi],
                low(&c.name)
            ));
        }
        Ok(s)
    }

    fn standard(&mut self, mi: usize) -> R<String> {
        let mut s = format!("/*@mod:{}*/\n", self.modules[mi].name);
        let mut cx = Ctx::module_scope(mi);
        for v in &self.ast(mi).vars {
            cx.line = v.line;
            let init = self.var_init(&mut cx, &v.decl)?;
            s.push_str(&format!(
                "var $v_{}_{}={init};\n",
                self.lnames[mi],
                low(&v.decl.name)
            ));
        }
        for (pi, p) in self.ast(mi).procs.iter().enumerate() {
            let name = proc_js_name(&self.modules[mi].name, &p.name, public_kind(p.kind));
            let body = self.procedure(mi, pi, &format!("function {name}"))?;
            s.push_str(&body);
            s.push_str(&format!("{name}.__params={};\n", params_json(&p.params)));
        }
        Ok(s)
    }

    fn class(&mut self, mi: usize) -> R<String> {
        let modules: &'a [Module] = self.modules;
        let m = &modules[mi];
        let form = m.kind == ModuleKind::Form;
        let cname = format!("{}{}", if form { "$F_" } else { "$C_" }, self.lnames[mi]);
        let base = if form { "__v.VbForm" } else { "__v.VbObject" };
        let mut s = format!(
            "/*@mod:{}*/\nclass {cname} extends {base} {{\nconstructor(){{ super({}); ",
            m.name,
            js_string(&m.name)
        );
        let mut cx = Ctx::module_scope(mi);
        for v in &self.ast(mi).vars {
            cx.line = v.line;
            let init = self.var_init(&mut cx, &v.decl)?;
            s.push_str(&format!("this.${}={init}; ", low(&v.decl.name)));
        }
        s.push_str("this.__init(); }\n");
        let mut params = String::new();
        for (pi, p) in self.ast(mi).procs.iter().enumerate() {
            let lname = low(&p.name);
            let method = match p.kind {
                ProcKind::Sub | ProcKind::Function => format!("${lname}"),
                ProcKind::PropertyGet => format!("__get_{lname}"),
                ProcKind::PropertyLet => format!("__let_{lname}"),
                ProcKind::PropertySet => format!("__set_{lname}"),
            };
            s.push_str(&self.procedure(mi, pi, &method)?);
            params.push_str(&format!(
                "{cname}.prototype.{method}.__params={};\n",
                params_json(&p.params)
            ));
        }
        s.push_str("}\n");
        s.push_str(&params);
        let reg = if form { "forms" } else { "classes" };
        s.push_str(&format!(
            "__v.{reg}[{}]={cname};\n",
            js_string(&self.lnames[mi])
        ));
        Ok(s)
    }

    // ----- Name resolution ----------------------------------------------

    fn module_member(&self, cx: &Ctx, m: usize, key: &str, own: bool) -> Option<Res> {
        let a = self.ast(m);
        let class = self.is_class(m);
        let in_own_class = own && class;
        if own || !class {
            if let Some(v) = a
                .vars
                .iter()
                .find(|v| low(&v.decl.name) == key && (own || v.is_public))
            {
                let js = if in_own_class {
                    format!("this.${key}")
                } else {
                    format!("$v_{}_{key}", self.lnames[m])
                };
                return Some(Res::Var {
                    js,
                    ty: v.decl.ty.clone(),
                });
            }
            if a.consts
                .iter()
                .any(|c| low(&c.name) == key && (own || c.is_public))
            {
                return Some(Res::Const(format!("$k_{}_{key}", self.lnames[m])));
            }
            if let Some(pi) = a.procs.iter().position(|p| {
                low(&p.name) == key
                    && matches!(p.kind, ProcKind::Sub | ProcKind::Function)
                    && (own || p.is_public)
            }) {
                return Some(if in_own_class {
                    Res::OwnMethod(pi)
                } else {
                    Res::Proc(m, pi)
                });
            }
            if a.procs.iter().any(|p| {
                low(&p.name) == key
                    && !matches!(p.kind, ProcKind::Sub | ProcKind::Function)
                    && (own || p.is_public)
            }) {
                return Some(if in_own_class {
                    Res::OwnProp(key.to_string())
                } else {
                    Res::StdProp(m, key.to_string())
                });
            }
            if let Some((n, _)) = a
                .declares
                .iter()
                .find(|(n, p)| low(n) == key && (own || *p))
            {
                return Some(Res::Declared(n.clone()));
            }
        }
        for e in &a.enums {
            if !(own || e.is_public) {
                continue;
            }
            if e.members.iter().any(|(n, _)| low(n) == key) {
                return Some(Res::Enum(format!("$e_{}_{key}", self.lnames[m])));
            }
        }
        if let Some(ei) = a
            .enums
            .iter()
            .position(|e| low(&e.name) == key && (own || e.is_public))
        {
            return Some(Res::EnumType(m, ei));
        }
        let _ = cx;
        None
    }

    fn resolve(&self, cx: &Ctx, name: &str) -> Res {
        let key = low(name);
        if let Some(l) = cx.locals.get(&key) {
            return if l.is_const {
                Res::Const(l.js.clone())
            } else {
                Res::Var {
                    js: l.js.clone(),
                    ty: l.ty.clone(),
                }
            };
        }
        if cx.proc_lname.as_deref() == Some(key.as_str()) && !cx.is_sub {
            return Res::Var {
                js: "$r".into(),
                ty: cx.ret_ty.clone(),
            };
        }
        if let Some(r) = self.module_member(cx, cx.mi, &key, true) {
            return r;
        }
        for m in 0..self.modules.len() {
            if m == cx.mi {
                continue;
            }
            if let Some(r) = self.module_member(cx, m, &key, false) {
                return r;
            }
        }
        if let Some(m) = self.lnames.iter().position(|n| *n == key) {
            return Res::Module(m);
        }
        if key == "err" {
            return Res::Err;
        }
        if crate::builtins::is_function(&key) {
            return Res::BuiltinFn(key);
        }
        if crate::builtins::is_constant(&key) {
            return Res::BuiltinConst(key);
        }
        Res::Unknown
    }

    /// A name the program does not define: the host's (or, in a form, a
    /// control of the form).
    fn host(&self, cx: &Ctx, name: &str) -> String {
        let global = if is_js_ident(name) && !JS_BLOCK.contains(&name) && !name.starts_with("__") {
            format!(
                "(typeof {name}!==\"undefined\"?{name}:__v.g({}))",
                js_string(name)
            )
        } else {
            format!("__v.g({})", js_string(name))
        };
        if self.kind(cx.mi) == ModuleKind::Form && !cx.proc_name.is_empty() {
            format!("__v.fid(this,{},()=>{global})", js_string(name))
        } else {
            global
        }
    }

    // ----- Procedures -----------------------------------------------------

    fn procedure(&mut self, mi: usize, pi: usize, header: &str) -> R<String> {
        let p = &self.ast(mi).procs[pi];
        let modules: &'a [Module] = self.modules;
        let mname = &modules[mi].name;
        let mut cx = Ctx::module_scope(mi);
        cx.line = p.line;
        cx.proc_name = p.name.clone();
        cx.is_sub = matches!(
            p.kind,
            ProcKind::Sub | ProcKind::PropertyLet | ProcKind::PropertySet
        );
        cx.proc_lname = Some(low(&p.name));
        cx.ret_ty = p.ret.clone();
        let args: Vec<String> = (0..p.params.len()).map(|i| format!("a{i}")).collect();
        let mut pre = String::new();
        for (i, prm) in p.params.iter().enumerate() {
            let js = format!("$l_{}", low(&prm.name));
            cx.locals.insert(
                low(&prm.name),
                Local {
                    js: js.clone(),
                    ty: if prm.is_array { None } else { prm.ty.clone() },
                    is_const: false,
                },
            );
            let init = if prm.param_array {
                format!("__v.parr(Array.prototype.slice.call(arguments,{i}))")
            } else {
                let given = if prm.is_array {
                    format!("__v.deref(a{i})")
                } else {
                    self.coerce(mi, prm.ty.as_ref(), format!("__v.deref(a{i})"))
                };
                if prm.optional {
                    let default = match &prm.default {
                        Some(d) => {
                            let d = self.expr(&mut cx, d)?;
                            self.coerce(mi, prm.ty.as_ref(), d)
                        }
                        None => match &prm.ty {
                            Some(t) if t.key() != "variant" => self.default_for(mi, Some(t), false),
                            _ => "__v.Missing".into(),
                        },
                    };
                    format!("(arguments.length<={i}||a{i}===__v.Missing||a{i}===undefined&&{})?{default}:{given}", prm.default.is_some())
                } else {
                    given
                }
            };
            pre.push_str(&format!("let {js}={init};\n"));
        }
        // The result of a function or property.
        if !cx.is_sub {
            let init = if p.ret_is_array {
                "new __v.VArray(null,()=>undefined,null,false)".to_string()
            } else {
                self.default_for(mi, p.ret.as_ref(), false)
            };
            pre.push_str(&format!("let $r={init};\n"));
        }
        // Declarations, constants and implicit variables, all hoisted.
        let mut decls = Vec::new();
        collect_decls(&p.body, &mut decls);
        for (line, d) in decls {
            cx.line = line;
            match d {
                Decl::Var(v, is_static) => {
                    let key = low(&v.name);
                    if cx.locals.contains_key(&key) {
                        continue;
                    }
                    let init = self.var_init(&mut cx, v)?;
                    if is_static || p.is_static {
                        let js = format!("$s_{}_{}_{key}", self.lnames[mi], low(&p.name));
                        self.statics.push_str(&format!("var {js}={init};\n"));
                        cx.locals.insert(
                            key,
                            Local {
                                js,
                                ty: if v.bounds.is_some() {
                                    None
                                } else {
                                    v.ty.clone()
                                },
                                is_const: false,
                            },
                        );
                    } else {
                        let js = format!("$l_{key}");
                        pre.push_str(&format!("let {js}={init};\n"));
                        cx.locals.insert(
                            key,
                            Local {
                                js,
                                ty: if v.bounds.is_some() {
                                    None
                                } else {
                                    v.ty.clone()
                                },
                                is_const: false,
                            },
                        );
                    }
                }
                Decl::Const(n, ty, e) => {
                    let key = low(n);
                    let v = self.expr(&mut cx, e)?;
                    let v = self.coerce(mi, ty.as_ref(), v);
                    let js = format!("$l_{key}");
                    pre.push_str(&format!("const {js}={v};\n"));
                    cx.locals.insert(
                        key,
                        Local {
                            js,
                            ty: ty.clone(),
                            is_const: true,
                        },
                    );
                }
            }
        }
        let mut implicit = Vec::new();
        collect_assigned(&p.body, &mut implicit);
        for (line, name) in implicit {
            let key = low(&name);
            if cx.locals.contains_key(&key) {
                continue;
            }
            if matches!(self.resolve(&cx, &name), Res::Unknown) {
                // Under Option Explicit an undeclared name can only be the
                // host's (a global the object model defines) or a form's
                // control: neither is known for sure when compiling, so it
                // is looked up when it runs. Without it, names the host
                // or the form defines are theirs too.
                let _ = line;
                if self.ast(mi).option_explicit
                    || self.host_names.contains(&key)
                    || self
                        .form_controls
                        .get(&mi)
                        .is_some_and(|cs| cs.contains(&key))
                {
                    continue;
                }
                let js = format!("$l_{key}");
                pre.push_str(&format!("let {js};\n"));
                cx.locals.insert(
                    key,
                    Local {
                        js,
                        ty: None,
                        is_const: false,
                    },
                );
            }
        }
        // Labels and error handling.
        cx.onerror = uses_on_error(&p.body);
        cx.sm = uses_jumps(&p.body);
        if cx.sm {
            let mut idx = 0usize;
            for s in &p.body {
                if let StmtKind::Label(n) = &s.kind {
                    cx.labels.insert(low(n), idx);
                } else {
                    idx += 1;
                }
            }
            check_jumps(&p.body, &cx.labels, &mut Vec::new()).map_err(|(line, l)| {
                CompileError {
                    module: mname.clone(),
                    line,
                    message: format!("GoTo to a label inside a block or a missing label: {l}"),
                }
            })?;
        }
        if cx.onerror {
            pre.push_str("let __oe=0,__oeh=0,__inh=false,__rs=0;\n");
        }
        let body = self.body(&mut cx, &p.body)?;
        let mut writeback = String::new();
        for (i, prm) in p.params.iter().enumerate() {
            if !prm.by_val && !prm.param_array {
                writeback.push_str(&format!("__v.wb(a{i},$l_{});", low(&prm.name)));
            }
        }
        let ct = if self.ast(mi).option_compare_text {
            1
        } else {
            0
        };
        let ret = if cx.is_sub { "" } else { "return $r;\n" };
        Ok(format!(
            "{header}({}){{\nconst __f=__v.enter({},{},{ct},{});\n{pre}try{{\n{body}}}catch(__x){{throw __v.locate(__x,__f);}}finally{{{writeback}__v.leave(__f);}}\n{ret}}}\n",
            args.join(","),
            js_string(mname),
            js_string(&p.name),
            if self.is_class(mi) { "this" } else { "null" }
        ))
    }

    fn body(&mut self, cx: &mut Ctx, stmts: &[Stmt]) -> R<String> {
        if !cx.sm {
            let mut s = String::new();
            for st in stmts {
                s.push_str(&self.stmt(cx, st)?);
            }
            return Ok(s);
        }
        let mut s = String::from("let __pc=0;\n__sm:while(true){switch(__pc){\n");
        let mut idx = 0usize;
        for st in stmts {
            if matches!(st.kind, StmtKind::Label(_)) {
                continue;
            }
            cx.top = idx;
            s.push_str(&format!("case {idx}:\n"));
            s.push_str(&self.stmt(cx, st)?);
            idx += 1;
        }
        s.push_str(&format!("case {idx}:\n}}break;}}\n"));
        Ok(s)
    }

    fn block(&mut self, cx: &mut Ctx, stmts: &[Stmt]) -> R<String> {
        if !stmts.iter().any(|s| matches!(s.kind, StmtKind::Label(_))) {
            let mut s = String::new();
            for st in stmts {
                s.push_str(&self.stmt(cx, st)?);
            }
            return Ok(s);
        }
        // A list with labels: its own state machine, so a GoTo from it or
        // from inside it reaches them (the "skip to Next" idiom).
        self.tmp += 1;
        let id = self.tmp;
        let mut labels = HashMap::new();
        let mut idx = 0usize;
        for st in stmts {
            if let StmtKind::Label(n) = &st.kind {
                labels.insert(low(n), idx);
            } else {
                idx += 1;
            }
        }
        cx.nested.push((id, labels));
        let mut s = format!("{{let __pc{id}=0;\n__sm{id}:while(true){{switch(__pc{id}){{\n");
        let mut idx = 0usize;
        for st in stmts {
            if matches!(st.kind, StmtKind::Label(_)) {
                continue;
            }
            s.push_str(&format!("case {idx}:\n"));
            match self.stmt(cx, st) {
                Ok(code) => s.push_str(&code),
                Err(e) => {
                    cx.nested.pop();
                    return Err(e);
                }
            }
            idx += 1;
        }
        s.push_str(&format!("case {idx}:\n}}break;}}}}\n"));
        cx.nested.pop();
        Ok(s)
    }

    fn stmt(&mut self, cx: &mut Ctx, st: &Stmt) -> R<String> {
        cx.line = st.line;
        let code = self.stmt_kind(cx, st)?;
        if code.is_empty() {
            return Ok(String::new());
        }
        let line = format!("__f.l={};", st.line);
        let wrap = cx.onerror
            && !matches!(
                st.kind,
                StmtKind::Label(_)
                    | StmtKind::OnErrorResumeNext
                    | StmtKind::OnErrorGoTo(_)
                    | StmtKind::OnErrorGoTo0
                    | StmtKind::OnErrorGoToMinus1
                    | StmtKind::Resume(_)
                    | StmtKind::Exit(_)
                    | StmtKind::GoTo(_)
                    | StmtKind::End
                    | StmtKind::Nop
            );
        if !wrap {
            return Ok(format!("{line}{code}\n"));
        }
        let goto = if cx.sm {
            format!(
                "else if(__oe===2&&!__inh){{__inh=true;__rs={};__pc=__oeh;continue __sm;}}",
                cx.top
            )
        } else {
            String::new()
        };
        Ok(format!(
            "{line}try{{{code}\n}}catch(__e){{if(__e===__v.END)throw __e;__v.seterr(__e,__f);if(__oe===1){{}}{goto}else throw __e;}}\n"
        ))
    }

    /// An If whose branches hold labels (a GoTo from one branch into
    /// another, which VBA allows): one state machine for all branches.
    /// Case 0 tests the conditions and goes to the branch; each branch
    /// ends by going to the end.
    fn if_with_labels(
        &mut self,
        cx: &mut Ctx,
        branches: &[(Expr, Vec<Stmt>)],
        otherwise: Option<&[Stmt]>,
    ) -> R<String> {
        self.tmp += 1;
        let id = self.tmp;
        let lists: Vec<&[Stmt]> = branches
            .iter()
            .map(|(_, b)| b.as_slice())
            .chain(otherwise)
            .collect();
        // Case numbers: 0 tests; each list's statements follow, then a
        // case that leaves; the last case is the end.
        let mut labels = HashMap::new();
        let mut starts = Vec::new();
        let mut idx = 1usize;
        for l in &lists {
            starts.push(idx);
            for st in l.iter() {
                if let StmtKind::Label(n) = &st.kind {
                    labels.insert(low(n), idx);
                } else {
                    idx += 1;
                }
            }
            idx += 1;
        }
        let end = idx;
        let mut test = String::new();
        for (i, (c, _)) in branches.iter().enumerate() {
            let c = self.expr(cx, c)?;
            if i > 0 {
                test.push_str("else ");
            }
            test.push_str(&format!(
                "if(__v.bool({c})){{__pc{id}={};continue __sm{id};}}",
                starts[i]
            ));
        }
        let fallback = if otherwise.is_some() {
            starts[branches.len()]
        } else {
            end
        };
        test.push_str(&format!(
            "__pc{id}={fallback};continue __sm{id};
"
        ));
        cx.nested.push((id, labels));
        let mut s =
            format!("{{let __pc{id}=0;\n__sm{id}:while(true){{switch(__pc{id}){{\ncase 0:{test}");
        let mut idx = 1usize;
        for l in &lists {
            for st in l.iter() {
                if matches!(st.kind, StmtKind::Label(_)) {
                    continue;
                }
                s.push_str(&format!("case {idx}:\n"));
                match self.stmt(cx, st) {
                    Ok(code) => s.push_str(&code),
                    Err(e) => {
                        cx.nested.pop();
                        return Err(e);
                    }
                }
                idx += 1;
            }
            s.push_str(&format!("case {idx}:__pc{id}={end};continue __sm{id};\n"));
            idx += 1;
        }
        s.push_str(&format!("case {end}:\n}}break;}}}}\n"));
        cx.nested.pop();
        Ok(s)
    }

    fn label_index(&self, cx: &Ctx, l: &str) -> R<usize> {
        match cx.labels.get(&low(l)) {
            Some(i) => Ok(*i),
            None => self.err(cx, format!("Label not defined: {l}")),
        }
    }

    fn stmt_kind(&mut self, cx: &mut Ctx, st: &Stmt) -> R<String> {
        Ok(match &st.kind {
            StmtKind::Dim { .. } | StmtKind::Const(_) | StmtKind::Nop | StmtKind::Label(_) => {
                String::new()
            }
            StmtKind::Assign { target, value, set } => {
                let v = self.expr(cx, value)?;
                self.assign(cx, target, v, *set)?
            }
            StmtKind::Call(e, args) => self.call_stmt(cx, e, args.as_deref())?,
            StmtKind::If {
                branches,
                otherwise,
            } if branches
                .iter()
                .map(|(_, b)| b)
                .chain(otherwise.iter())
                .any(|b| b.iter().any(|st| matches!(st.kind, StmtKind::Label(_)))) =>
            {
                self.if_with_labels(cx, branches, otherwise.as_deref())?
            }
            StmtKind::If {
                branches,
                otherwise,
            } => {
                let mut s = String::new();
                for (i, (c, b)) in branches.iter().enumerate() {
                    let c = self.expr(cx, c)?;
                    let b = self.block(cx, b)?;
                    if i > 0 {
                        s.push_str("else ");
                    }
                    s.push_str(&format!("if(__v.bool({c})){{\n{b}}}"));
                }
                if let Some(o) = otherwise {
                    let b = self.block(cx, o)?;
                    s.push_str(&format!("else{{\n{b}}}"));
                }
                s
            }
            StmtKind::Select {
                test,
                cases,
                otherwise,
            } => {
                let t = self.tmp("s");
                let tv = self.expr(cx, test)?;
                let ct = self.ct(cx);
                let mut s = format!("{{const {t}={tv};\n");
                for (i, (items, body)) in cases.iter().enumerate() {
                    let mut conds = Vec::new();
                    for it in items {
                        conds.push(match it {
                            CaseItem::Value(e) => {
                                let e = self.expr(cx, e)?;
                                format!("__v.bool(__v.cmp(\"=\",{t},{e},{ct}))")
                            }
                            CaseItem::Range(a, b) => {
                                let a = self.expr(cx, a)?;
                                let b = self.expr(cx, b)?;
                                format!("(__v.bool(__v.cmp(\">=\",{t},{a},{ct}))&&__v.bool(__v.cmp(\"<=\",{t},{b},{ct})))")
                            }
                            CaseItem::Is(op, e) => {
                                let e = self.expr(cx, e)?;
                                format!("__v.bool(__v.cmp({},{t},{e},{ct}))", js_string(cmp_op(*op)))
                            }
                        });
                    }
                    let b = self.block(cx, body)?;
                    if i > 0 {
                        s.push_str("else ");
                    }
                    s.push_str(&format!("if({}){{\n{b}}}", conds.join("||")));
                }
                if let Some(o) = otherwise {
                    let b = self.block(cx, o)?;
                    if cases.is_empty() {
                        s.push_str(&format!("{{\n{b}}}"));
                    } else {
                        s.push_str(&format!("else{{\n{b}}}"));
                    }
                }
                s.push('}');
                s
            }
            StmtKind::For {
                var,
                from,
                to,
                step,
                body,
            } => {
                let (read, ty) = self.lvalue_var(cx, var)?;
                let n = self.tmp("");
                let lab = format!("__L{n}");
                let from = self.expr(cx, from)?;
                let to = self.expr(cx, to)?;
                let step = match step {
                    Some(s) => self.expr(cx, s)?,
                    None => "1".into(),
                };
                let set_from = self.coerce(cx.mi, ty.as_ref(), format!("__v.num({from})"));
                let next = self.coerce(cx.mi, ty.as_ref(), format!("__v.num({read})+__s{n}"));
                cx.loops.push((LoopKind::For, lab.clone()));
                let b = self.block(cx, body)?;
                cx.loops.pop();
                format!(
                    "{{const __e{n}=__v.num({to}),__s{n}=__v.num({step});\n{lab}:for({read}={set_from};(__s{n}>=0?__v.num({read})<=__e{n}:__v.num({read})>=__e{n});{read}={next}){{\n{b}}}}}"
                )
            }
            StmtKind::ForEach { var, coll, body } => {
                let (read, _) = self.lvalue_var(cx, var)?;
                let n = self.tmp("");
                let lab = format!("__L{n}");
                let c = self.expr(cx, coll)?;
                cx.loops.push((LoopKind::For, lab.clone()));
                let b = self.block(cx, body)?;
                cx.loops.pop();
                format!("{lab}:for(const __i{n} of __v.each({c})){{{read}=__i{n};\n{b}}}")
            }
            StmtKind::DoLoop { cond, body } => {
                let n = self.tmp("");
                let lab = format!("__L{n}");
                cx.loops.push((LoopKind::Do, lab.clone()));
                let b = self.block(cx, body)?;
                cx.loops.pop();
                match cond {
                    None => format!("{lab}:while(true){{\n{b}}}"),
                    Some((k, c)) => {
                        let c = self.expr(cx, c)?;
                        match k {
                            LoopCond::PreWhile => format!("{lab}:while(__v.bool({c})){{\n{b}}}"),
                            LoopCond::PreUntil => format!("{lab}:while(!__v.bool({c})){{\n{b}}}"),
                            LoopCond::PostWhile => {
                                format!("{lab}:do{{\n{b}}}while(__v.bool({c}));")
                            }
                            LoopCond::PostUntil => {
                                format!("{lab}:do{{\n{b}}}while(!__v.bool({c}));")
                            }
                        }
                    }
                }
            }
            StmtKind::While { cond, body } => {
                let n = self.tmp("");
                let lab = format!("__L{n}");
                let c = self.expr(cx, cond)?;
                cx.loops.push((LoopKind::While, lab.clone()));
                let b = self.block(cx, body)?;
                cx.loops.pop();
                format!("{lab}:while(__v.bool({c})){{\n{b}}}")
            }
            StmtKind::With { obj, body } => {
                let w = self.tmp("w");
                let o = self.expr(cx, obj)?;
                cx.withs.push(w.clone());
                let b = self.block(cx, body)?;
                cx.withs.pop();
                format!("{{const {w}={o};\n{b}}}")
            }
            StmtKind::Exit(k) => match k {
                ExitKind::Sub | ExitKind::Function | ExitKind::Property => {
                    if cx.is_sub {
                        "return;".into()
                    } else {
                        "return $r;".into()
                    }
                }
                ExitKind::For | ExitKind::Do => {
                    let want = if *k == ExitKind::For {
                        LoopKind::For
                    } else {
                        LoopKind::Do
                    };
                    match cx.loops.iter().rev().find(|(kind, _)| *kind == want) {
                        Some((_, l)) => format!("break {l};"),
                        None => return self.err(cx, "Exit outside of a loop"),
                    }
                }
            },
            StmtKind::GoTo(l) => {
                let key = low(l);
                if let Some((id, i)) = cx
                    .nested
                    .iter()
                    .rev()
                    .find_map(|(id, labels)| labels.get(&key).map(|i| (*id, *i)))
                {
                    format!("{{__pc{id}={i};continue __sm{id};}}")
                } else {
                    let i = self.label_index(cx, l)?;
                    format!("{{__pc={i};continue __sm;}}")
                }
            }
            StmtKind::OnErrorResumeNext => "__oe=1;__v.Err.Clear();".into(),
            StmtKind::OnErrorGoTo0 => "__oe=0;__v.Err.Clear();".into(),
            StmtKind::OnErrorGoToMinus1 => "__inh=false;__v.Err.Clear();".into(),
            StmtKind::OnErrorGoTo(l) => {
                let i = self.label_index(cx, l)?;
                format!("__oe=2;__oeh={i};__v.Err.Clear();")
            }
            StmtKind::Resume(k) => {
                if !cx.onerror {
                    return self.err(cx, "Resume without error handling");
                }
                let target = match k {
                    ResumeKind::Here => "__rs".to_string(),
                    ResumeKind::Next => "__rs+1".to_string(),
                    ResumeKind::Label(l) => self.label_index(cx, l)?.to_string(),
                };
                format!("{{__inh=false;__v.Err.Clear();__pc={target};continue __sm;}}")
            }
            StmtKind::ReDim { preserve, vars } => {
                let mut s = String::new();
                for (name, bounds, ty) in vars {
                    let (read, vty) = self.lvalue_var(cx, name)?;
                    let ty = ty.clone().or(vty);
                    let dims = self.dims(cx, bounds)?;
                    let def = self.default_for(cx.mi, ty.as_ref(), false);
                    let co = self
                        .coerce_fn(cx.mi, ty.as_ref())
                        .filter(|f| *f != "__v.letv")
                        .unwrap_or("null");
                    s.push_str(&format!(
                        "{read}=__v.redim({read},{dims},{preserve},()=>{def},{co});"
                    ));
                }
                s
            }
            StmtKind::Erase(names) => {
                let mut s = String::new();
                for n in names {
                    let (read, _) = self.lvalue_var(cx, n)?;
                    s.push_str(&format!("{read}=__v.erase({read});"));
                }
                s
            }
            StmtKind::MidAssign {
                target,
                start,
                len,
                value,
            } => {
                let t = self.expr(cx, target)?;
                let st = self.expr(cx, start)?;
                let l = match len {
                    Some(l) => self.expr(cx, l)?,
                    None => "undefined".into(),
                };
                let v = self.expr(cx, value)?;
                self.assign(cx, target, format!("__v.midset({t},{st},{l},{v})"), false)?
            }
            StmtKind::DebugPrint(items) => {
                let mut parts = Vec::new();
                for (e, sep) in items {
                    let e = self.expr(cx, e)?;
                    let sep = match sep {
                        Some(';') => "\";\"",
                        Some(',') => "\",\"",
                        _ => "\"\"",
                    };
                    parts.push(format!("[{e},{sep}]"));
                }
                format!("__v.dprint([{}]);", parts.join(","))
            }
            StmtKind::End => "throw __v.END;".into(),
            StmtKind::RaiseEvent(..) => String::new(),
            StmtKind::Unload(e) => {
                let e = self.expr(cx, e)?;
                format!("__v.unload({e});")
            }
            StmtKind::Load(e) => {
                let e = self.expr(cx, e)?;
                format!("__v.load({e});")
            }
            StmtKind::Unsupported(what) => format!("__v.unsupported({});", js_string(what)),
        })
    }

    fn ct(&self, cx: &Ctx) -> u8 {
        u8::from(self.ast(cx.mi).option_compare_text)
    }

    /// A variable to assign to by name (loop variables, ReDim, Erase):
    /// its JavaScript and type.
    fn lvalue_var(&self, cx: &Ctx, name: &str) -> R<(String, Option<TypeRef>)> {
        match self.resolve(cx, name) {
            Res::Var { js, ty } => Ok((js, ty)),
            _ => self.err(cx, format!("Not a variable: {name}")),
        }
    }

    fn assign(&mut self, cx: &mut Ctx, target: &Expr, value: String, set: bool) -> R<String> {
        match target {
            Expr::Ident(n) => match self.resolve(cx, n) {
                Res::Var { js, ty } => {
                    let v = if set {
                        value
                    } else {
                        self.coerce(cx.mi, ty.as_ref(), value)
                    };
                    Ok(format!("{js}={v};"))
                }
                Res::OwnProp(l) => Ok(format!(
                    "this.{}_{l}({value});",
                    if set { "__set" } else { "__let" }
                )),
                Res::StdProp(m, l) => {
                    let p = if set { "$ps_" } else { "$pl_" };
                    Ok(format!("{p}{}_{l}({value});", self.lnames[m]))
                }
                Res::Const(_) => self.err(cx, format!("Assignment to constant not permitted: {n}")),
                Res::Unknown => {
                    let h = self.host(cx, n);
                    Ok(format!("__v.setg({h},{},{value});", js_string(n)))
                }
                _ => self.err(cx, format!("Cannot assign to {n}")),
            },
            Expr::Member(base, name) => {
                if let Expr::Ident(b) = base.as_ref() {
                    if let Res::Module(m) = self.resolve(cx, b) {
                        if let Some(Res::Var { js, ty }) =
                            self.module_member(cx, m, &low(name), m == cx.mi)
                        {
                            let v = if set {
                                value
                            } else {
                                self.coerce(m, ty.as_ref(), value)
                            };
                            return Ok(format!("{js}={v};"));
                        }
                    }
                }
                let b = self.expr(cx, base)?;
                Ok(format!("__v.set({b},{},{value},{set});", js_string(name)))
            }
            Expr::WithMember(name) => {
                let w = self.with_obj(cx)?;
                Ok(format!("__v.set({w},{},{value},{set});", js_string(name)))
            }
            Expr::Bang(base, name) => {
                let b = self.expr(cx, base)?;
                Ok(format!("__v.setIdx({b},[{}],{value});", js_string(name)))
            }
            Expr::Call(callee, args) => {
                let a = self.plain_args(cx, args)?;
                match callee.as_ref() {
                    Expr::Member(base, name) => {
                        let b = self.expr(cx, base)?;
                        Ok(format!("__v.setm({b},{},[{a}],{value});", js_string(name)))
                    }
                    Expr::WithMember(name) => {
                        let w = self.with_obj(cx)?;
                        Ok(format!("__v.setm({w},{},[{a}],{value});", js_string(name)))
                    }
                    Expr::Ident(n) => match self.resolve(cx, n) {
                        Res::OwnProp(l) => Ok(format!("this.__let_{l}({a},{value});")),
                        Res::StdProp(m, l) => {
                            Ok(format!("$pl_{}_{l}({a},{value});", self.lnames[m]))
                        }
                        _ => {
                            let c = self.expr(cx, callee)?;
                            Ok(format!("__v.setIdx({c},[{a}],{value});"))
                        }
                    },
                    other => {
                        let c = self.expr(cx, other)?;
                        Ok(format!("__v.setIdx({c},[{a}],{value});"))
                    }
                }
            }
            Expr::Me => self.err(cx, "Cannot assign to Me"),
            _ => self.err(cx, "This cannot be assigned to"),
        }
    }

    fn with_obj(&self, cx: &Ctx) -> R<String> {
        match cx.withs.last() {
            Some(w) => Ok(w.clone()),
            None => self.err(cx, "A leading dot outside a With block"),
        }
    }

    fn call_stmt(&mut self, cx: &mut Ctx, e: &Expr, extra: Option<&[Arg]>) -> R<String> {
        // Written without parentheses: the arguments go to the callee.
        let call = match (e, extra) {
            (_, None) => e.clone(),
            (Expr::Ident(_) | Expr::Member(..) | Expr::WithMember(_), Some(args)) => {
                Expr::Call(Box::new(e.clone()), args.to_vec())
            }
            (Expr::Call(..), Some(args)) => Expr::Call(Box::new(e.clone()), args.to_vec()),
            (_, Some(_)) => return self.err(cx, "Expected a procedure call"),
        };
        let js = match &call {
            Expr::Member(base, name) => {
                if let Expr::Ident(b) = base.as_ref() {
                    if let Res::Module(_) = self.resolve(cx, b) {
                        return Ok(format!("{};", self.expr(cx, &call)?));
                    }
                }
                let b = self.expr(cx, base)?;
                format!("__v.call({b},{},[],null)", js_string(name))
            }
            Expr::WithMember(name) => {
                let w = self.with_obj(cx)?;
                format!("__v.call({w},{},[],null)", js_string(name))
            }
            Expr::Ident(n) => match self.resolve(cx, n) {
                Res::Unknown => {
                    let h = self.host(cx, n);
                    format!("__v.callg({h},{},[],null)", js_string(n))
                }
                _ => self.expr(cx, &call)?,
            },
            _ => self.expr(cx, &call)?,
        };
        Ok(format!("{js};"))
    }

    // ----- Expressions ------------------------------------------------------

    fn plain_args(&mut self, cx: &mut Ctx, args: &[Arg]) -> R<String> {
        let mut out = Vec::new();
        for a in args {
            out.push(match &a.value {
                Some(v) => self.expr(cx, v)?,
                None => "__v.Missing".into(),
            });
        }
        Ok(out.join(","))
    }

    /// Positional arguments and named ones (`{name:value}`) for a call
    /// resolved at run time.
    fn dyn_args(&mut self, cx: &mut Ctx, args: &[Arg]) -> R<(String, String)> {
        let mut pos = Vec::new();
        let mut named = Vec::new();
        for a in args {
            let v = match &a.value {
                Some(v) => self.expr(cx, v)?,
                None => "__v.Missing".into(),
            };
            match &a.name {
                Some(n) => named.push(format!("{}:{v}", js_string(&low(n)))),
                None => pos.push(v),
            }
        }
        let named = if named.is_empty() {
            "null".into()
        } else {
            format!("{{{}}}", named.join(","))
        };
        Ok((pos.join(","), named))
    }

    /// Arguments to a procedure of the project: named ones in place,
    /// missing ones marked, variables passed by reference.
    fn proc_args(&mut self, cx: &mut Ctx, p: &Proc, args: &[Arg]) -> R<String> {
        let mut slots: Vec<Option<String>> = Vec::new();
        let n = p.params.len();
        let has_pa = p.params.last().is_some_and(|q| q.param_array);
        let mut i = 0usize;
        for a in args {
            if let Some(name) = &a.name {
                let Some(idx) = p.params.iter().position(|q| low(&q.name) == low(name)) else {
                    return self.err(cx, format!("Named argument not found: {name}"));
                };
                while slots.len() <= idx {
                    slots.push(None);
                }
                let v = self.arg_value(cx, &p.params[idx], a.value.as_ref())?;
                slots[idx] = Some(v);
                continue;
            }
            if i >= n && !has_pa {
                return self.err(cx, format!("Wrong number of arguments: {}", p.name));
            }
            let param = &p.params[i.min(n - 1)];
            let v = if param.param_array {
                match &a.value {
                    Some(v) => self.expr(cx, v)?,
                    None => "__v.Missing".into(),
                }
            } else {
                self.arg_value(cx, param, a.value.as_ref())?
            };
            if slots.len() <= i {
                slots.push(Some(v));
            } else {
                slots[i] = Some(v);
            }
            i += 1;
        }
        let mut out = Vec::new();
        for (k, s) in slots.iter().enumerate() {
            match s {
                Some(v) => out.push(v.clone()),
                None => {
                    let param = &p.params[k.min(n.saturating_sub(1))];
                    if !param.optional && !param.param_array {
                        return self.err(cx, format!("Argument not optional: {}", param.name));
                    }
                    out.push("__v.Missing".into());
                }
            }
        }
        for param in p.params.iter().skip(slots.len()) {
            if !param.optional && !param.param_array {
                return self.err(cx, format!("Argument not optional: {}", param.name));
            }
        }
        Ok(out.join(","))
    }

    fn arg_value(&mut self, cx: &mut Ctx, param: &Param, value: Option<&Expr>) -> R<String> {
        let Some(e) = value else {
            if !param.optional {
                return self.err(cx, format!("Argument not optional: {}", param.name));
            }
            return Ok("__v.Missing".into());
        };
        if !param.by_val {
            // A variable or an array element goes by reference.
            match e {
                Expr::Ident(n) => {
                    if let Res::Var { js, .. } = self.resolve(cx, n) {
                        return Ok(format!("__v.ref(()=>{js},(__x)=>{{{js}=__x;}})"));
                    }
                }
                Expr::Call(callee, args) => {
                    if let Expr::Ident(n) = callee.as_ref() {
                        if let Res::Var { js, .. } = self.resolve(cx, n) {
                            if args.iter().all(|a| a.name.is_none() && a.value.is_some()) {
                                let a = self.plain_args(cx, args)?;
                                return Ok(format!("__v.refIdx({js},[{a}])"));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        self.expr(cx, e)
    }

    fn expr(&mut self, cx: &mut Ctx, e: &Expr) -> R<String> {
        Ok(match e {
            Expr::Num(n) => num_lit(*n),
            Expr::Str(s) => js_string(s),
            Expr::Date(d) => format!("__v.mkdate({})", num_lit(*d)),
            Expr::Bool(b) => b.to_string(),
            Expr::Empty => "undefined".into(),
            Expr::Null => "__v.Null".into(),
            Expr::Nothing => "null".into(),
            Expr::Me => {
                if !self.is_class(cx.mi) {
                    return self.err(cx, "Invalid use of Me keyword");
                }
                "this".into()
            }
            Expr::Paren(e) => format!("({})", self.expr(cx, e)?),
            Expr::Ident(n) => self.ident(cx, n)?,
            Expr::Member(base, name) => self.member(cx, base, name)?,
            Expr::WithMember(name) => {
                let w = self.with_obj(cx)?;
                if name.is_empty() {
                    w
                } else {
                    format!("__v.get({w},{})", js_string(name))
                }
            }
            Expr::Bang(base, name) => {
                let b = self.expr(cx, base)?;
                format!("__v.index({b},[{}])", js_string(name))
            }
            Expr::Call(callee, args) => self.call(cx, callee, args)?,
            Expr::New(t) => self.new_expr(t),
            Expr::Unary(UnOp::Neg, e) => format!("__v.neg({})", self.expr(cx, e)?),
            Expr::Unary(UnOp::Not, e) => format!("__v.not({})", self.expr(cx, e)?),
            Expr::Binary(op, a, b) => {
                let a = self.expr(cx, a)?;
                let b = self.expr(cx, b)?;
                let ct = self.ct(cx);
                match op {
                    BinOp::Add => format!("__v.add({a},{b})"),
                    BinOp::Sub => format!("__v.sub({a},{b})"),
                    BinOp::Mul => format!("__v.mul({a},{b})"),
                    BinOp::Div => format!("__v.div({a},{b})"),
                    BinOp::IntDiv => format!("__v.idiv({a},{b})"),
                    BinOp::Mod => format!("__v.mod({a},{b})"),
                    BinOp::Pow => format!("__v.pow({a},{b})"),
                    BinOp::Cat => format!("__v.cat({a},{b})"),
                    BinOp::Like => format!("__v.like({a},{b},{ct})"),
                    BinOp::Is => format!("__v.is({a},{b})"),
                    BinOp::And => format!("__v.and({a},{b})"),
                    BinOp::Or => format!("__v.or({a},{b})"),
                    BinOp::Xor => format!("__v.xor({a},{b})"),
                    BinOp::Eqv => format!("__v.eqv({a},{b})"),
                    BinOp::Imp => format!("__v.imp({a},{b})"),
                    cmp => format!("__v.cmp({},{a},{b},{ct})", js_string(cmp_op(*cmp))),
                }
            }
            Expr::TypeOfIs(e, t) => {
                let e = self.expr(cx, e)?;
                format!("__v.typeOf({e},{})", js_string(t))
            }
            Expr::AddressOf(n) => {
                format!("__v.unsupported({})", js_string(&format!("AddressOf {n}")))
            }
        })
    }

    fn ident(&mut self, cx: &mut Ctx, n: &str) -> R<String> {
        Ok(match self.resolve(cx, n) {
            Res::Var { js, .. } => js,
            Res::Const(js) | Res::Enum(js) => js,
            Res::Proc(m, pi) => {
                let p = &self.ast(m).procs[pi];
                let a = self.proc_args(cx, p, &[])?;
                format!(
                    "{}({a})",
                    proc_js_name(&self.modules[m].name, &p.name, public_kind(p.kind))
                )
            }
            Res::OwnMethod(pi) => {
                let p = &self.ast(cx.mi).procs[pi];
                let a = self.proc_args(cx, p, &[])?;
                format!("this.${}({a})", low(&p.name))
            }
            Res::StdProp(m, l) => format!("$pg_{}_{l}()", self.lnames[m]),
            Res::OwnProp(l) => format!("this.__get_{l}()"),
            Res::EnumType(..) => {
                return self.err(cx, format!("Expected a value, not the enum {n}"))
            }
            Res::Module(m) => {
                if self.kind(m) == ModuleKind::Form {
                    format!("__v.form({})", js_string(&self.lnames[m]))
                } else {
                    return self.err(cx, format!("Expected a value, not the module {n}"));
                }
            }
            Res::Declared(name) => format!(
                "__v.unsupported({})",
                js_string(&format!("Windows API function {name}"))
            ),
            Res::BuiltinFn(f) => format!("__b.{f}()"),
            Res::BuiltinConst(c) => format!("__b.{c}"),
            Res::Err => "__v.Err".into(),
            Res::Unknown => self.host(cx, n),
        })
    }

    fn member(&mut self, cx: &mut Ctx, base: &Expr, name: &str) -> R<String> {
        if let Expr::Ident(b) = base {
            match self.resolve(cx, b) {
                Res::Module(m) if self.kind(m) != ModuleKind::Form && !self.is_class(m) => {
                    let own = m == cx.mi;
                    return match self.module_member(cx, m, &low(name), own) {
                        Some(Res::Var { js, .. }) | Some(Res::Const(js)) | Some(Res::Enum(js)) => {
                            Ok(js)
                        }
                        Some(Res::Proc(m2, pi)) => {
                            let p = &self.ast(m2).procs[pi];
                            let a = self.proc_args(cx, p, &[])?;
                            Ok(format!(
                                "{}({a})",
                                proc_js_name(&self.modules[m2].name, &p.name, public_kind(p.kind))
                            ))
                        }
                        Some(Res::StdProp(m2, l)) => Ok(format!("$pg_{}_{l}()", self.lnames[m2])),
                        _ => self.err(cx, format!("Method or data member not found: {b}.{name}")),
                    };
                }
                Res::EnumType(m, ei) => {
                    let e = &self.ast(m).enums[ei];
                    if e.members.iter().any(|(n, _)| low(n) == low(name)) {
                        return Ok(format!("$e_{}_{}", self.lnames[m], low(name)));
                    }
                    return self.err(cx, format!("Enum member not found: {b}.{name}"));
                }
                Res::Unknown if b.eq_ignore_ascii_case("vba") => {
                    let key = low(name);
                    if crate::builtins::is_function(&key) {
                        return Ok(format!("__b.{key}()"));
                    }
                    if crate::builtins::is_constant(&key) {
                        return Ok(format!("__b.{key}"));
                    }
                }
                _ => {}
            }
        }
        let b = self.expr(cx, base)?;
        Ok(format!("__v.get({b},{})", js_string(name)))
    }

    fn call(&mut self, cx: &mut Ctx, callee: &Expr, args: &[Arg]) -> R<String> {
        match callee {
            Expr::Ident(n) => {
                // Inside a function, its name with arguments calls it again.
                let res = if cx.proc_lname.as_deref() == Some(low(n).as_str())
                    && !cx.locals.contains_key(&low(n))
                {
                    self.module_member(cx, cx.mi, &low(n), true)
                        .unwrap_or_else(|| self.resolve(cx, n))
                } else {
                    self.resolve(cx, n)
                };
                match res {
                    Res::Proc(m, pi) => {
                        let p = &self.ast(m).procs[pi];
                        let a = self.proc_args(cx, p, args)?;
                        Ok(format!(
                            "{}({a})",
                            proc_js_name(&self.modules[m].name, &p.name, public_kind(p.kind))
                        ))
                    }
                    Res::OwnMethod(pi) => {
                        let p = &self.ast(cx.mi).procs[pi];
                        let a = self.proc_args(cx, p, args)?;
                        Ok(format!("this.${}({a})", low(&p.name)))
                    }
                    Res::StdProp(m, l) => {
                        let a = self.plain_args(cx, args)?;
                        Ok(format!("$pg_{}_{l}({a})", self.lnames[m]))
                    }
                    Res::OwnProp(l) => {
                        let a = self.plain_args(cx, args)?;
                        Ok(format!("this.__get_{l}({a})"))
                    }
                    Res::BuiltinFn(f) => {
                        if f == "array" {
                            let a = self.plain_args(cx, args)?;
                            let base = self.ast(cx.mi).option_base;
                            return Ok(format!("__v.arr({base},[{a}])"));
                        }
                        let (pos, named) = self.dyn_args(cx, args)?;
                        if named == "null" {
                            Ok(format!("__b.{f}({pos})"))
                        } else {
                            Ok(format!("__v.callf(__b.{f},[{pos}],{named})"))
                        }
                    }
                    Res::Declared(name) => Ok(format!(
                        "__v.unsupported({})",
                        js_string(&format!("Windows API function {name}"))
                    )),
                    Res::Unknown => {
                        let h = self.host(cx, n);
                        let (pos, named) = self.dyn_args(cx, args)?;
                        Ok(format!("__v.callg({h},{},[{pos}],{named})", js_string(n)))
                    }
                    _ => {
                        let v = self.ident(cx, n)?;
                        let a = self.plain_args(cx, args)?;
                        Ok(format!("__v.index({v},[{a}])"))
                    }
                }
            }
            Expr::Member(base, name) => {
                if let Expr::Ident(b) = base.as_ref() {
                    if let Res::Module(m) = self.resolve(cx, b) {
                        if !self.is_class(m) {
                            match self.module_member(cx, m, &low(name), m == cx.mi) {
                                Some(Res::Proc(m2, pi)) => {
                                    let p = &self.ast(m2).procs[pi];
                                    let a = self.proc_args(cx, p, args)?;
                                    return Ok(format!(
                                        "{}({a})",
                                        proc_js_name(
                                            &self.modules[m2].name,
                                            &p.name,
                                            public_kind(p.kind)
                                        )
                                    ));
                                }
                                Some(Res::Var { js, .. }) => {
                                    let a = self.plain_args(cx, args)?;
                                    return Ok(format!("__v.index({js},[{a}])"));
                                }
                                Some(Res::StdProp(m2, l)) => {
                                    let a = self.plain_args(cx, args)?;
                                    return Ok(format!("$pg_{}_{l}({a})", self.lnames[m2]));
                                }
                                _ => {
                                    return self.err(
                                        cx,
                                        format!("Method or data member not found: {b}.{name}"),
                                    )
                                }
                            }
                        }
                    }
                    if let Res::Unknown = self.resolve(cx, b) {
                        if b.eq_ignore_ascii_case("vba") {
                            return self.call(cx, &Expr::Ident(name.clone()), args);
                        }
                    }
                }
                let b = self.expr(cx, base)?;
                let (pos, named) = self.dyn_args(cx, args)?;
                Ok(format!("__v.call({b},{},[{pos}],{named})", js_string(name)))
            }
            Expr::WithMember(name) => {
                let w = self.with_obj(cx)?;
                let (pos, named) = self.dyn_args(cx, args)?;
                Ok(format!("__v.call({w},{},[{pos}],{named})", js_string(name)))
            }
            other => {
                let c = self.expr(cx, other)?;
                let a = self.plain_args(cx, args)?;
                Ok(format!("__v.index({c},[{a}])"))
            }
        }
    }
}

fn cmp_op(op: BinOp) -> &'static str {
    match op {
        BinOp::Eq => "=",
        BinOp::Ne => "<>",
        BinOp::Lt => "<",
        BinOp::Gt => ">",
        BinOp::Le => "<=",
        _ => ">=",
    }
}

fn params_json(params: &[Param]) -> String {
    let names: Vec<String> = params.iter().map(|p| js_string(&low(&p.name))).collect();
    format!("[{}]", names.join(","))
}

enum Decl<'a> {
    Var(&'a VarDecl, bool),
    Const(&'a String, &'a Option<TypeRef>, &'a Expr),
}

fn children(s: &Stmt) -> Vec<&Vec<Stmt>> {
    match &s.kind {
        StmtKind::If {
            branches,
            otherwise,
        } => {
            let mut v: Vec<&Vec<Stmt>> = branches.iter().map(|(_, b)| b).collect();
            if let Some(o) = otherwise {
                v.push(o);
            }
            v
        }
        StmtKind::Select {
            cases, otherwise, ..
        } => {
            let mut v: Vec<&Vec<Stmt>> = cases.iter().map(|(_, b)| b).collect();
            if let Some(o) = otherwise {
                v.push(o);
            }
            v
        }
        StmtKind::For { body, .. }
        | StmtKind::ForEach { body, .. }
        | StmtKind::DoLoop { body, .. }
        | StmtKind::While { body, .. }
        | StmtKind::With { body, .. } => vec![body],
        _ => Vec::new(),
    }
}

fn collect_decls<'a>(stmts: &'a [Stmt], out: &mut Vec<(u32, Decl<'a>)>) {
    for s in stmts {
        match &s.kind {
            StmtKind::Dim { vars, is_static } => {
                for v in vars {
                    out.push((s.line, Decl::Var(v, *is_static)));
                }
            }
            StmtKind::Const(cs) => {
                for (n, t, e) in cs {
                    out.push((s.line, Decl::Const(n, t, e)));
                }
            }
            _ => {}
        }
        for c in children(s) {
            collect_decls(c, out);
        }
    }
}

/// Names a procedure assigns to: implicit variables when not declared.
fn collect_assigned(stmts: &[Stmt], out: &mut Vec<(u32, String)>) {
    for s in stmts {
        match &s.kind {
            StmtKind::Assign {
                target: Expr::Ident(n),
                ..
            }
            | StmtKind::MidAssign {
                target: Expr::Ident(n),
                ..
            } => out.push((s.line, n.clone())),
            StmtKind::For { var, .. } | StmtKind::ForEach { var, .. } => {
                out.push((s.line, var.clone()))
            }
            StmtKind::ReDim { vars, .. } => {
                for (n, _, _) in vars {
                    out.push((s.line, n.clone()));
                }
            }
            _ => {}
        }
        for c in children(s) {
            collect_assigned(c, out);
        }
    }
}

fn any_stmt(stmts: &[Stmt], f: &dyn Fn(&StmtKind) -> bool) -> bool {
    stmts
        .iter()
        .any(|s| f(&s.kind) || children(s).iter().any(|c| any_stmt(c, f)))
}

fn uses_on_error(stmts: &[Stmt]) -> bool {
    any_stmt(stmts, &|k| {
        matches!(
            k,
            StmtKind::OnErrorResumeNext
                | StmtKind::OnErrorGoTo(_)
                | StmtKind::OnErrorGoTo0
                | StmtKind::OnErrorGoToMinus1
                | StmtKind::Resume(_)
        )
    })
}

fn uses_jumps(stmts: &[Stmt]) -> bool {
    any_stmt(stmts, &|k| {
        matches!(
            k,
            StmtKind::GoTo(_) | StmtKind::OnErrorGoTo(_) | StmtKind::Resume(_)
        )
    })
}

/// Every GoTo target is a label of the procedure's top level or of a
/// statement list the GoTo is in; On Error GoTo and Resume go to labels
/// of the top level.
fn check_jumps(
    stmts: &[Stmt],
    labels: &HashMap<String, usize>,
    inner: &mut Vec<Vec<String>>,
) -> Result<(), (u32, String)> {
    let here: Vec<String> = stmts
        .iter()
        .filter_map(|s| match &s.kind {
            StmtKind::Label(n) => Some(low(n)),
            _ => None,
        })
        .collect();
    inner.push(here);
    for s in stmts {
        let (target, any) = match &s.kind {
            StmtKind::GoTo(l) => (Some(l), true),
            StmtKind::OnErrorGoTo(l) => (Some(l), false),
            StmtKind::Resume(ResumeKind::Label(l)) => (Some(l), false),
            _ => (None, false),
        };
        if let Some(l) = target {
            let k = low(l);
            let found = labels.contains_key(&k) || (any && inner.iter().any(|ls| ls.contains(&k)));
            if !found {
                inner.pop();
                return Err((s.line, l.clone()));
            }
        }
        // The branches of an If can jump into each other.
        let kids = children(s);
        let shared: Vec<String> = if matches!(s.kind, StmtKind::If { .. }) {
            kids.iter()
                .flat_map(|c| c.iter())
                .filter_map(|st| match &st.kind {
                    StmtKind::Label(n) => Some(low(n)),
                    _ => None,
                })
                .collect()
        } else {
            Vec::new()
        };
        inner.push(shared);
        for c in kids {
            if let Err(e) = check_jumps(c, labels, inner) {
                inner.pop();
                inner.pop();
                return Err(e);
            }
        }
        inner.pop();
    }
    inner.pop();
    Ok(())
}
