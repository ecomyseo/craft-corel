//! Parser: tokens to a module's syntax tree.

use crate::ast::*;
use crate::lexer::{Tok, Token};
use crate::CompileError;

pub struct Parser<'a> {
    toks: Vec<Token>,
    pos: usize,
    module: &'a str,
    /// Inside a single-line If: `Else` ends a statement.
    single_if: u32,
    /// `Next i, j` closes more than one loop: the loops still to close.
    pending_next: u32,
}

type R<T> = Result<T, CompileError>;

/// Words that cannot start or continue an expression.
const RESERVED: [&str; 50] = [
    "and",
    "or",
    "not",
    "xor",
    "eqv",
    "imp",
    "mod",
    "is",
    "like",
    "then",
    "else",
    "elseif",
    "end",
    "to",
    "step",
    "each",
    "in",
    "next",
    "loop",
    "wend",
    "case",
    "select",
    "as",
    "byval",
    "byref",
    "optional",
    "paramarray",
    "dim",
    "redim",
    "set",
    "let",
    "call",
    "sub",
    "function",
    "property",
    "if",
    "for",
    "do",
    "while",
    "until",
    "with",
    "exit",
    "goto",
    "on",
    "resume",
    "const",
    "public",
    "private",
    "static",
    "global",
];

impl<'a> Parser<'a> {
    pub fn new(module: &'a str, toks: Vec<Token>) -> Self {
        Parser {
            toks,
            pos: 0,
            module,
            single_if: 0,
            pending_next: 0,
        }
    }

    fn err<T>(&self, msg: impl Into<String>) -> R<T> {
        Err(CompileError {
            module: self.module.to_string(),
            line: self.toks[self.pos.min(self.toks.len() - 1)].line,
            message: msg.into(),
        })
    }

    fn peek(&self) -> &Tok {
        &self.toks[self.pos.min(self.toks.len() - 1)].tok
    }

    fn peek_at(&self, n: usize) -> &Tok {
        &self.toks[(self.pos + n).min(self.toks.len() - 1)].tok
    }

    fn tok(&self) -> &Token {
        &self.toks[self.pos.min(self.toks.len() - 1)]
    }

    fn line(&self) -> u32 {
        self.tok().line
    }

    fn bump(&mut self) -> Tok {
        let t = self.peek().clone();
        if self.pos < self.toks.len() - 1 {
            self.pos += 1;
        }
        t
    }

    fn kw(&self, w: &str) -> bool {
        matches!(self.peek(), Tok::Ident(s) if s.eq_ignore_ascii_case(w))
    }

    fn kw_at(&self, n: usize, w: &str) -> bool {
        matches!(self.peek_at(n), Tok::Ident(s) if s.eq_ignore_ascii_case(w))
    }

    fn op(&self, o: &str) -> bool {
        matches!(self.peek(), Tok::Op(x) if *x == o)
    }

    fn eat_kw(&mut self, w: &str) -> bool {
        if self.kw(w) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn eat_op(&mut self, o: &str) -> bool {
        if self.op(o) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn expect_kw(&mut self, w: &str) -> R<()> {
        if self.eat_kw(w) {
            Ok(())
        } else {
            self.err(format!("Expected {w}"))
        }
    }

    fn expect_op(&mut self, o: &str) -> R<()> {
        if self.eat_op(o) {
            Ok(())
        } else {
            self.err(format!("Expected {o}"))
        }
    }

    fn ident(&mut self) -> R<String> {
        match self.bump() {
            Tok::Ident(s) => Ok(s),
            t => {
                self.pos -= 1;
                self.err(format!("Expected a name, found {t:?}"))
            }
        }
    }

    /// End of a statement: a new line, a colon, the end, or `Else` in a
    /// single-line If.
    fn at_eos(&self) -> bool {
        match self.peek() {
            Tok::Newline | Tok::Eof => true,
            Tok::Op(":") => true,
            Tok::Ident(s) if self.single_if > 0 && s.eq_ignore_ascii_case("else") => true,
            _ => false,
        }
    }

    fn skip_seps(&mut self) {
        while matches!(self.peek(), Tok::Newline) || self.op(":") {
            self.bump();
        }
    }

    fn skip_line(&mut self) {
        while !matches!(self.peek(), Tok::Newline | Tok::Eof) {
            self.bump();
        }
    }

    fn end_stmt(&mut self) -> R<()> {
        // A statement that ended with its own line (`Next i, j`).
        if self.pos > 0
            && matches!(self.toks[self.pos - 1].tok, Tok::Newline | Tok::Op(":"))
            && !self.at_eos()
        {
            return Ok(());
        }
        if self.at_eos() {
            if !self.kw("else") && (matches!(self.peek(), Tok::Newline) || self.op(":")) {
                self.bump();
            }
            Ok(())
        } else {
            self.err(format!(
                "Expected end of statement, found {:?}",
                self.peek()
            ))
        }
    }

    // ----- Module level -------------------------------------------------

    pub fn module(&mut self) -> R<ModuleAst> {
        let mut m = ModuleAst::default();
        loop {
            self.skip_seps();
            if matches!(self.peek(), Tok::Eof) {
                break;
            }
            let line = self.line();
            if self.kw("attribute") || self.kw("implements") {
                self.skip_line();
                continue;
            }
            if self.eat_kw("option") {
                if self.eat_kw("explicit") {
                    m.option_explicit = true;
                } else if self.eat_kw("base") {
                    match self.bump() {
                        Tok::Num(n) => m.option_base = n as i32,
                        _ => return self.err("Expected 0 or 1"),
                    }
                } else if self.eat_kw("compare") {
                    m.option_compare_text = self.kw("text");
                    self.bump();
                } else if self.eat_kw("private") {
                    self.expect_kw("module")?;
                    m.option_private = true;
                } else {
                    return self.err("Unknown Option");
                }
                self.end_stmt()?;
                continue;
            }
            let mut public = true;
            let mut explicit_scope = false;
            let mut is_static = false;
            loop {
                if self.eat_kw("public") || self.eat_kw("global") {
                    public = true;
                    explicit_scope = true;
                } else if self.eat_kw("private") {
                    public = false;
                    explicit_scope = true;
                } else if self.eat_kw("friend") {
                    public = true;
                    explicit_scope = true;
                } else if self.eat_kw("static") {
                    is_static = true;
                } else if self.eat_kw("default") {
                } else {
                    break;
                }
            }
            if self.kw("sub") || self.kw("function") || self.kw("property") {
                let p = self.proc(public, is_static)?;
                m.procs.push(p);
                continue;
            }
            if self.eat_kw("const") {
                for (name, ty, value) in self.const_list()? {
                    m.consts.push(ModConst {
                        name,
                        ty,
                        value,
                        is_public: public && explicit_scope,
                        line,
                    });
                }
                self.end_stmt()?;
                continue;
            }
            if self.eat_kw("type") {
                let t = self.user_type(public)?;
                m.types.push(t);
                continue;
            }
            if self.eat_kw("enum") {
                let e = self.enum_decl(public)?;
                m.enums.push(e);
                continue;
            }
            if self.eat_kw("declare") {
                self.eat_kw("ptrsafe");
                if !(self.eat_kw("sub") || self.eat_kw("function")) {
                    return self.err("Expected Sub or Function");
                }
                let name = self.ident()?;
                m.declares.push((name, public));
                self.skip_line();
                continue;
            }
            if self.kw("event") {
                self.skip_line();
                continue;
            }
            let dim = self.eat_kw("dim");
            if dim || explicit_scope {
                // Module-level Dim is private.
                let is_public = explicit_scope && public;
                for decl in self.var_list()? {
                    m.vars.push(ModVar {
                        decl,
                        is_public,
                        line,
                    });
                }
                self.end_stmt()?;
                continue;
            }
            return self.err(format!(
                "Statement not allowed outside a procedure: {:?}",
                self.peek()
            ));
        }
        Ok(m)
    }

    fn type_ref(&mut self) -> R<TypeRef> {
        let mut name = self.ident()?;
        while self.op(".") {
            self.bump();
            name.push('.');
            name.push_str(&self.ident()?);
        }
        // `String * 20`
        if self.op("*") {
            self.bump();
            self.bump();
        }
        Ok(TypeRef { name })
    }

    fn bounds(&mut self) -> R<Vec<Bound>> {
        // After "(": bounds up to ")".
        let mut out = Vec::new();
        if self.eat_op(")") {
            return Ok(out);
        }
        loop {
            let a = self.expr()?;
            if self.eat_kw("to") {
                let b = self.expr()?;
                out.push(Bound {
                    lower: Some(a),
                    upper: b,
                });
            } else {
                out.push(Bound {
                    lower: None,
                    upper: a,
                });
            }
            if self.eat_op(",") {
                continue;
            }
            self.expect_op(")")?;
            break;
        }
        Ok(out)
    }

    fn var_decl(&mut self) -> R<VarDecl> {
        self.eat_kw("withevents");
        let name = self.ident()?;
        let bounds = if self.eat_op("(") {
            Some(self.bounds()?)
        } else {
            None
        };
        let mut ty = None;
        let mut is_new = false;
        if self.eat_kw("as") {
            is_new = self.eat_kw("new");
            ty = Some(self.type_ref()?);
        }
        Ok(VarDecl {
            name,
            bounds,
            ty,
            is_new,
        })
    }

    fn var_list(&mut self) -> R<Vec<VarDecl>> {
        let mut out = vec![self.var_decl()?];
        while self.eat_op(",") {
            out.push(self.var_decl()?);
        }
        Ok(out)
    }

    fn const_list(&mut self) -> R<Vec<(String, Option<TypeRef>, Expr)>> {
        let mut out = Vec::new();
        loop {
            let name = self.ident()?;
            let ty = if self.eat_kw("as") {
                Some(self.type_ref()?)
            } else {
                None
            };
            self.expect_op("=")?;
            let v = self.expr()?;
            out.push((name, ty, v));
            if !self.eat_op(",") {
                break;
            }
        }
        Ok(out)
    }

    fn user_type(&mut self, public: bool) -> R<UserType> {
        let name = self.ident()?;
        self.end_stmt()?;
        let mut fields = Vec::new();
        loop {
            self.skip_seps();
            if self.kw("end") && self.kw_at(1, "type") {
                self.bump();
                self.bump();
                break;
            }
            if matches!(self.peek(), Tok::Eof) {
                return self.err("Missing End Type");
            }
            fields.push(self.var_decl()?);
            self.end_stmt()?;
        }
        Ok(UserType {
            name,
            is_public: public,
            fields,
        })
    }

    fn enum_decl(&mut self, public: bool) -> R<EnumDecl> {
        let name = self.ident()?;
        self.end_stmt()?;
        let mut members = Vec::new();
        loop {
            self.skip_seps();
            if self.kw("end") && self.kw_at(1, "enum") {
                self.bump();
                self.bump();
                break;
            }
            if matches!(self.peek(), Tok::Eof) {
                return self.err("Missing End Enum");
            }
            let n = self.ident()?;
            let v = if self.eat_op("=") {
                Some(self.expr()?)
            } else {
                None
            };
            members.push((n, v));
            self.end_stmt()?;
        }
        Ok(EnumDecl {
            name,
            is_public: public,
            members,
        })
    }

    fn proc(&mut self, public: bool, is_static: bool) -> R<Proc> {
        let line = self.line();
        let kind = if self.eat_kw("sub") {
            ProcKind::Sub
        } else if self.eat_kw("function") {
            ProcKind::Function
        } else {
            self.expect_kw("property")?;
            if self.eat_kw("get") {
                ProcKind::PropertyGet
            } else if self.eat_kw("let") {
                ProcKind::PropertyLet
            } else if self.eat_kw("set") {
                ProcKind::PropertySet
            } else {
                return self.err("Expected Get, Let or Set");
            }
        };
        let name = self.ident()?;
        let mut params = Vec::new();
        if self.eat_op("(") && !self.eat_op(")") {
            loop {
                params.push(self.param()?);
                if self.eat_op(",") {
                    continue;
                }
                self.expect_op(")")?;
                break;
            }
        }
        let mut ret = None;
        let mut ret_is_array = false;
        if self.eat_kw("as") {
            ret = Some(self.type_ref()?);
            if self.eat_op("(") {
                self.expect_op(")")?;
                ret_is_array = true;
            }
        }
        self.end_stmt()?;
        let body = self.block()?;
        let end_word = match kind {
            ProcKind::Sub => "sub",
            ProcKind::Function => "function",
            _ => "property",
        };
        if !(self.kw("end") && self.kw_at(1, end_word)) {
            return self.err(format!("Expected End {}", capital(end_word)));
        }
        self.bump();
        self.bump();
        Ok(Proc {
            name,
            kind,
            is_public: public,
            is_static,
            params,
            ret,
            ret_is_array,
            body,
            line,
        })
    }

    fn param(&mut self) -> R<Param> {
        let mut p = Param {
            name: String::new(),
            by_val: false,
            optional: false,
            default: None,
            param_array: false,
            ty: None,
            is_array: false,
        };
        loop {
            if self.eat_kw("optional") {
                p.optional = true;
            } else if self.eat_kw("byval") {
                p.by_val = true;
            } else if self.eat_kw("byref") {
                p.by_val = false;
            } else if self.eat_kw("paramarray") {
                p.param_array = true;
            } else {
                break;
            }
        }
        p.name = self.ident()?;
        if self.eat_op("(") {
            self.expect_op(")")?;
            p.is_array = true;
        }
        if self.eat_kw("as") {
            p.ty = Some(self.type_ref()?);
        }
        if self.eat_op("=") {
            p.default = Some(self.expr()?);
        }
        Ok(p)
    }

    // ----- Statements -----------------------------------------------------

    /// At a statement start: a word that ends the current block.
    fn at_terminator(&self) -> bool {
        if self.pending_next > 0 {
            return true;
        }
        match self.peek() {
            Tok::Ident(s) => {
                let s = s.to_ascii_lowercase();
                match s.as_str() {
                    "else" | "elseif" | "case" | "next" | "loop" | "wend" => true,
                    "end" => matches!(self.peek_at(1), Tok::Ident(w) if [
                        "if", "sub", "function", "property", "select", "with", "type", "enum"
                    ].iter().any(|k| w.eq_ignore_ascii_case(k))),
                    _ => false,
                }
            }
            Tok::Eof => true,
            _ => false,
        }
    }

    fn block(&mut self) -> R<Vec<Stmt>> {
        let mut out = Vec::new();
        loop {
            self.skip_seps();
            if self.at_terminator() {
                break;
            }
            if let Some(s) = self.statement()? {
                out.push(s);
            }
        }
        Ok(out)
    }

    fn statement(&mut self) -> R<Option<Stmt>> {
        let line = self.line();
        let mk = |kind| Ok(Some(Stmt { line, kind }));
        // Labels at the start of a line.
        if self.tok().line_start {
            if let Tok::Ident(name) = self.peek().clone() {
                // `Do:` and `Else:` start statements; they are not labels.
                let keyword = [
                    "do", "else", "loop", "next", "wend", "end", "exit", "resume",
                ]
                .contains(&name.to_ascii_lowercase().as_str());
                if matches!(self.peek_at(1), Tok::Op(":")) && !self.at_terminator() && !keyword {
                    self.bump();
                    self.bump();
                    return mk(StmtKind::Label(name));
                }
            }
            if let Tok::Num(n) = self.peek().clone() {
                self.bump();
                self.eat_op(":");
                return mk(StmtKind::Label(format!("{}", n as i64)));
            }
        }
        let kind = self.statement_kind()?;
        if !matches!(kind, StmtKind::If { .. }) || self.at_eos() {
            self.end_stmt()?;
        }
        mk(kind)
    }

    fn statement_kind(&mut self) -> R<StmtKind> {
        if self.op(".") {
            return self.simple_statement();
        }
        let Tok::Ident(word) = self.peek().clone() else {
            return self.err(format!("Unexpected {:?}", self.peek()));
        };
        let w = word.to_ascii_lowercase();
        match w.as_str() {
            "attribute" => {
                self.skip_line();
                Ok(StmtKind::Nop)
            }
            "dim" | "static" | "private" | "public" => {
                self.bump();
                if w == "static" && (self.kw("sub") || self.kw("function")) {
                    return self.err("Nested procedures are not allowed");
                }
                Ok(StmtKind::Dim {
                    vars: self.var_list()?,
                    is_static: w == "static",
                })
            }
            "const" => {
                self.bump();
                Ok(StmtKind::Const(self.const_list()?))
            }
            "if" => self.if_statement(),
            "select" => self.select_statement(),
            "for" => self.for_statement(),
            "do" => self.do_statement(),
            "while" => {
                self.bump();
                let cond = self.expr()?;
                let body = self.block()?;
                self.expect_kw("wend")?;
                Ok(StmtKind::While { cond, body })
            }
            "with" => {
                self.bump();
                let obj = self.expr()?;
                let body = self.block()?;
                if !(self.kw("end") && self.kw_at(1, "with")) {
                    return self.err("Expected End With");
                }
                self.bump();
                self.bump();
                Ok(StmtKind::With { obj, body })
            }
            "exit" => {
                self.bump();
                let k = if self.eat_kw("sub") {
                    ExitKind::Sub
                } else if self.eat_kw("function") {
                    ExitKind::Function
                } else if self.eat_kw("property") {
                    ExitKind::Property
                } else if self.eat_kw("for") {
                    ExitKind::For
                } else if self.eat_kw("do") {
                    ExitKind::Do
                } else {
                    return self.err("Expected Sub, Function, Property, For or Do");
                };
                Ok(StmtKind::Exit(k))
            }
            "goto" => {
                self.bump();
                Ok(StmtKind::GoTo(self.label_name()?))
            }
            "gosub" | "return" => {
                self.skip_to_eos();
                Ok(StmtKind::Unsupported(format!("{word} statement")))
            }
            "on" => {
                self.bump();
                if self.eat_kw("error") {
                    if self.eat_kw("resume") {
                        self.expect_kw("next")?;
                        return Ok(StmtKind::OnErrorResumeNext);
                    }
                    self.expect_kw("goto")?;
                    if self.eat_op("-") {
                        self.bump();
                        return Ok(StmtKind::OnErrorGoToMinus1);
                    }
                    if let Tok::Num(n) = self.peek().clone() {
                        self.bump();
                        if n == 0.0 {
                            return Ok(StmtKind::OnErrorGoTo0);
                        }
                        return Ok(StmtKind::OnErrorGoTo(format!("{}", n as i64)));
                    }
                    return Ok(StmtKind::OnErrorGoTo(self.ident()?));
                }
                self.skip_to_eos();
                Ok(StmtKind::Unsupported("On ... GoTo/GoSub statement".into()))
            }
            "resume" => {
                self.bump();
                if self.at_eos() {
                    return Ok(StmtKind::Resume(ResumeKind::Here));
                }
                if self.eat_kw("next") {
                    return Ok(StmtKind::Resume(ResumeKind::Next));
                }
                if let Tok::Num(n) = self.peek().clone() {
                    self.bump();
                    if n == 0.0 {
                        return Ok(StmtKind::Resume(ResumeKind::Here));
                    }
                    return Ok(StmtKind::Resume(ResumeKind::Label(format!("{}", n as i64))));
                }
                Ok(StmtKind::Resume(ResumeKind::Label(self.ident()?)))
            }
            "redim" => {
                self.bump();
                let preserve = self.eat_kw("preserve");
                let mut vars = Vec::new();
                loop {
                    let name = self.ident()?;
                    self.expect_op("(")?;
                    let b = self.bounds()?;
                    let ty = if self.eat_kw("as") {
                        Some(self.type_ref()?)
                    } else {
                        None
                    };
                    vars.push((name, b, ty));
                    if !self.eat_op(",") {
                        break;
                    }
                }
                Ok(StmtKind::ReDim { preserve, vars })
            }
            "erase" => {
                self.bump();
                let mut names = vec![self.ident()?];
                while self.eat_op(",") {
                    names.push(self.ident()?);
                }
                Ok(StmtKind::Erase(names))
            }
            "set" | "let" => {
                self.bump();
                let target = self.postfix()?;
                self.expect_op("=")?;
                let value = self.expr()?;
                Ok(StmtKind::Assign {
                    target,
                    value,
                    set: w == "set",
                })
            }
            "call" => {
                self.bump();
                let e = self.postfix()?;
                Ok(StmtKind::Call(e, None))
            }
            "end" => {
                self.bump();
                Ok(StmtKind::End)
            }
            "stop" => {
                self.bump();
                Ok(StmtKind::Nop)
            }
            "raiseevent" => {
                self.bump();
                let name = self.ident()?;
                let args = if self.eat_op("(") {
                    self.paren_args()?
                } else {
                    Vec::new()
                };
                Ok(StmtKind::RaiseEvent(name, args))
            }
            "unload" | "load" => {
                self.bump();
                let e = self.expr()?;
                Ok(if w == "unload" {
                    StmtKind::Unload(e)
                } else {
                    StmtKind::Load(e)
                })
            }
            "debug" if matches!(self.peek_at(1), Tok::Op(".")) => {
                if self.kw_at(2, "print") {
                    self.bump();
                    self.bump();
                    self.bump();
                    let mut items = Vec::new();
                    while !self.at_eos() {
                        if self.op(";") || self.op(",") {
                            let c = if self.op(";") { ';' } else { ',' };
                            self.bump();
                            items.push((Expr::Str(String::new()), Some(c)));
                            continue;
                        }
                        let e = self.expr()?;
                        let sep = if self.eat_op(";") {
                            Some(';')
                        } else if self.eat_op(",") {
                            Some(',')
                        } else {
                            None
                        };
                        items.push((e, sep));
                    }
                    return Ok(StmtKind::DebugPrint(items));
                }
                if self.kw_at(2, "assert") {
                    self.skip_to_eos();
                    return Ok(StmtKind::Nop);
                }
                self.simple_statement()
            }
            "open" | "close" | "kill" | "mkdir" | "rmdir" | "chdir" | "chdrive" | "filecopy"
            | "setattr" | "sendkeys" | "appactivate" | "lock" | "unlock" | "get" | "put"
            | "seek" | "write" | "input" | "width" | "lset" | "rset" | "reset"
                if self.file_statement_follows(&w) =>
            {
                self.skip_to_eos();
                Ok(StmtKind::Unsupported(format!("{word} statement")))
            }
            "print" if matches!(self.peek_at(1), Tok::Op("#")) => {
                self.skip_to_eos();
                Ok(StmtKind::Unsupported("Print # statement".into()))
            }
            "line" if self.kw_at(1, "input") => {
                self.skip_to_eos();
                Ok(StmtKind::Unsupported("Line Input statement".into()))
            }
            "name" if !matches!(self.peek_at(1), Tok::Op("=") | Tok::Op(".") | Tok::Op("(")) => {
                self.skip_to_eos();
                Ok(StmtKind::Unsupported("Name statement".into()))
            }
            _ => self.simple_statement(),
        }
    }

    /// Whether a file statement word is used as one (and not as a name).
    fn file_statement_follows(&self, w: &str) -> bool {
        let next = self.peek_at(1);
        if matches!(next, Tok::Op("=") | Tok::Op(".") | Tok::Op(":=")) {
            return false;
        }
        match w {
            "lock" | "unlock" | "get" | "put" | "seek" | "write" | "input" | "width" => {
                matches!(next, Tok::Op("#"))
            }
            "close" | "reset" => {
                matches!(next, Tok::Op("#") | Tok::Newline | Tok::Eof | Tok::Num(_))
            }
            "lset" | "rset" => true,
            _ => !matches!(next, Tok::Op("(")) || w == "open",
        }
    }

    fn skip_to_eos(&mut self) {
        while !self.at_eos() {
            self.bump();
        }
    }

    fn label_name(&mut self) -> R<String> {
        match self.bump() {
            Tok::Ident(s) => Ok(s),
            Tok::Num(n) => Ok(format!("{}", n as i64)),
            _ => self.err("Expected a label"),
        }
    }

    /// An assignment or a call statement.
    fn simple_statement(&mut self) -> R<StmtKind> {
        let target = self.postfix_stmt()?;
        if self.op("=") {
            self.bump();
            let value = self.expr()?;
            // Mid(s, start[, len]) = value
            if let Expr::Call(callee, args) = &target {
                if let Expr::Ident(n) = callee.as_ref() {
                    if n.eq_ignore_ascii_case("mid") && (2..=3).contains(&args.len()) {
                        let get = |i: usize| args.get(i).and_then(|a| a.value.clone());
                        return Ok(StmtKind::MidAssign {
                            target: get(0).ok_or_else(|| self.err::<()>("").unwrap_err())?,
                            start: get(1).ok_or_else(|| self.err::<()>("").unwrap_err())?,
                            len: get(2),
                            value,
                        });
                    }
                }
            }
            return Ok(StmtKind::Assign {
                target,
                value,
                set: false,
            });
        }
        if self.at_eos() {
            // `Foo (a)` without Call: the parentheses make a value, passed
            // by value.
            if let Expr::Call(callee, args) = &target {
                if matches!(
                    callee.as_ref(),
                    Expr::Ident(_) | Expr::Member(..) | Expr::WithMember(_)
                ) && args.len() == 1
                    && args[0].name.is_none()
                {
                    if let Some(v) = &args[0].value {
                        let arg = Arg {
                            name: None,
                            value: Some(Expr::Paren(Box::new(v.clone()))),
                        };
                        return Ok(StmtKind::Call(Expr::Call(callee.clone(), vec![arg]), None));
                    }
                }
            }
            return Ok(StmtKind::Call(target, None));
        }
        let args = self.noparen_args()?;
        Ok(StmtKind::Call(target, Some(args)))
    }

    /// The callee of a statement. A parenthesis after a space that is not
    /// the whole argument list (`MsgBox (a) & b`, `Foo (a), b`) starts the
    /// arguments instead.
    fn postfix_stmt(&mut self) -> R<Expr> {
        let mut e = self.primary()?;
        loop {
            if self.op(".") && !self.tok().space_before {
                self.bump();
                let n = self.ident()?;
                e = Expr::Member(Box::new(e), n);
            } else if self.op("!") && !self.tok().space_before {
                self.bump();
                let n = self.ident()?;
                e = Expr::Bang(Box::new(e), n);
            } else if self.op("(") {
                let spaced = self.tok().space_before;
                let save = self.pos;
                self.bump();
                let args = self.paren_args()?;
                if spaced
                    && !matches!(e, Expr::Call(..))
                    && !self.at_eos()
                    && !self.op("=")
                    && !self.op(".")
                    && !self.op("(")
                    && !self.op("!")
                {
                    self.pos = save;
                    return Ok(e);
                }
                e = Expr::Call(Box::new(e), args);
            } else {
                break;
            }
        }
        Ok(e)
    }

    fn noparen_args(&mut self) -> R<Vec<Arg>> {
        let mut args = Vec::new();
        let missing = || Arg {
            name: None,
            value: None,
        };
        loop {
            if self.at_eos() {
                break;
            }
            if self.eat_op(",") {
                args.push(missing());
                continue;
            }
            args.push(self.arg()?);
            if !self.eat_op(",") {
                break;
            }
            if self.at_eos() {
                args.push(missing());
            }
        }
        Ok(args)
    }

    fn arg(&mut self) -> R<Arg> {
        if let Tok::Ident(n) = self.peek().clone() {
            if matches!(self.peek_at(1), Tok::Op(":=")) {
                self.bump();
                self.bump();
                return Ok(Arg {
                    name: Some(n),
                    value: Some(self.expr()?),
                });
            }
        }
        self.eat_kw("byval");
        Ok(Arg {
            name: None,
            value: Some(self.expr()?),
        })
    }

    /// After "(": the arguments up to ")".
    fn paren_args(&mut self) -> R<Vec<Arg>> {
        let mut args = Vec::new();
        if self.eat_op(")") {
            return Ok(args);
        }
        loop {
            if self.op(",") {
                self.bump();
                args.push(Arg {
                    name: None,
                    value: None,
                });
                if self.op(")") {
                    args.push(Arg {
                        name: None,
                        value: None,
                    });
                    self.bump();
                    return Ok(args);
                }
                continue;
            }
            if self.op(")") {
                self.bump();
                return Ok(args);
            }
            args.push(self.arg()?);
            if self.eat_op(",") {
                if self.op(")") {
                    args.push(Arg {
                        name: None,
                        value: None,
                    });
                    self.bump();
                    return Ok(args);
                }
                continue;
            }
            self.expect_op(")")?;
            return Ok(args);
        }
    }

    fn if_statement(&mut self) -> R<StmtKind> {
        self.expect_kw("if")?;
        let cond = self.expr()?;
        self.expect_kw("then")?;
        if matches!(self.peek(), Tok::Newline | Tok::Eof) {
            // Block If.
            let mut branches = vec![(cond, self.block()?)];
            let mut otherwise = None;
            loop {
                if self.eat_kw("elseif") {
                    let c = self.expr()?;
                    self.expect_kw("then")?;
                    let b = self.block()?;
                    branches.push((c, b));
                } else if self.eat_kw("else") {
                    // `Else If` on one line is an If inside the Else.
                    otherwise = Some(self.block()?);
                } else if self.kw("end") && self.kw_at(1, "if") {
                    self.bump();
                    self.bump();
                    break;
                } else {
                    return self.err("Expected End If");
                }
            }
            return Ok(StmtKind::If {
                branches,
                otherwise,
            });
        }
        // Single-line If.
        self.single_if += 1;
        let then = self.single_line_stmts()?;
        let mut otherwise = None;
        if self.eat_kw("else") {
            otherwise = Some(self.single_line_stmts()?);
        }
        self.single_if -= 1;
        Ok(StmtKind::If {
            branches: vec![(cond, then)],
            otherwise,
        })
    }

    fn single_line_stmts(&mut self) -> R<Vec<Stmt>> {
        let mut out = Vec::new();
        // `Then 100`: a line number to go to.
        if let Tok::Num(n) = self.peek().clone() {
            let line = self.line();
            self.bump();
            out.push(Stmt {
                line,
                kind: StmtKind::GoTo(format!("{}", n as i64)),
            });
            return Ok(out);
        }
        loop {
            if matches!(self.peek(), Tok::Newline | Tok::Eof) || self.kw("else") {
                break;
            }
            if self.eat_op(":") {
                continue;
            }
            let line = self.line();
            let kind = self.statement_kind()?;
            out.push(Stmt { line, kind });
        }
        Ok(out)
    }

    fn select_statement(&mut self) -> R<StmtKind> {
        self.expect_kw("select")?;
        self.expect_kw("case")?;
        let test = self.expr()?;
        self.end_stmt()?;
        let mut cases = Vec::new();
        let mut otherwise = None;
        loop {
            self.skip_seps();
            if self.kw("end") && self.kw_at(1, "select") {
                self.bump();
                self.bump();
                break;
            }
            if !self.eat_kw("case") {
                return self.err("Expected Case or End Select");
            }
            if self.eat_kw("else") {
                otherwise = Some(self.block()?);
                continue;
            }
            let mut items = Vec::new();
            loop {
                if self.eat_kw("is") {
                    let op = match self.bump() {
                        Tok::Op("=") => BinOp::Eq,
                        Tok::Op("<>") => BinOp::Ne,
                        Tok::Op("<") => BinOp::Lt,
                        Tok::Op(">") => BinOp::Gt,
                        Tok::Op("<=") => BinOp::Le,
                        Tok::Op(">=") => BinOp::Ge,
                        _ => return self.err("Expected a comparison after Is"),
                    };
                    items.push(CaseItem::Is(op, self.expr()?));
                } else {
                    let a = self.expr()?;
                    if self.eat_kw("to") {
                        items.push(CaseItem::Range(a, self.expr()?));
                    } else {
                        items.push(CaseItem::Value(a));
                    }
                }
                if !self.eat_op(",") {
                    break;
                }
            }
            let body = self.block()?;
            cases.push((items, body));
        }
        Ok(StmtKind::Select {
            test,
            cases,
            otherwise,
        })
    }

    fn close_next(&mut self) -> R<()> {
        if self.pending_next > 0 {
            self.pending_next -= 1;
            return Ok(());
        }
        self.expect_kw("next")?;
        if !self.at_eos() {
            self.ident()?;
            while self.eat_op(",") {
                self.ident()?;
                self.pending_next += 1;
            }
        }
        Ok(())
    }

    fn for_statement(&mut self) -> R<StmtKind> {
        self.expect_kw("for")?;
        if self.eat_kw("each") {
            let var = self.ident()?;
            self.expect_kw("in")?;
            let coll = self.expr()?;
            let body = self.block()?;
            self.close_next()?;
            return Ok(StmtKind::ForEach { var, coll, body });
        }
        let var = self.ident()?;
        self.expect_op("=")?;
        let from = self.expr()?;
        self.expect_kw("to")?;
        let to = self.expr()?;
        let step = if self.eat_kw("step") {
            Some(self.expr()?)
        } else {
            None
        };
        let body = self.block()?;
        self.close_next()?;
        Ok(StmtKind::For {
            var,
            from,
            to,
            step,
            body,
        })
    }

    fn do_statement(&mut self) -> R<StmtKind> {
        self.expect_kw("do")?;
        let mut cond = None;
        if self.eat_kw("while") {
            cond = Some((LoopCond::PreWhile, self.expr()?));
        } else if self.eat_kw("until") {
            cond = Some((LoopCond::PreUntil, self.expr()?));
        }
        let body = self.block()?;
        self.expect_kw("loop")?;
        if self.eat_kw("while") {
            cond = Some((LoopCond::PostWhile, self.expr()?));
        } else if self.eat_kw("until") {
            cond = Some((LoopCond::PostUntil, self.expr()?));
        }
        Ok(StmtKind::DoLoop { cond, body })
    }

    // ----- Expressions ------------------------------------------------------

    pub fn expr(&mut self) -> R<Expr> {
        self.imp()
    }

    fn imp(&mut self) -> R<Expr> {
        let mut e = self.eqv()?;
        while self.eat_kw("imp") {
            e = Expr::Binary(BinOp::Imp, Box::new(e), Box::new(self.eqv()?));
        }
        Ok(e)
    }

    fn eqv(&mut self) -> R<Expr> {
        let mut e = self.xor()?;
        while self.eat_kw("eqv") {
            e = Expr::Binary(BinOp::Eqv, Box::new(e), Box::new(self.xor()?));
        }
        Ok(e)
    }

    fn xor(&mut self) -> R<Expr> {
        let mut e = self.or()?;
        while self.eat_kw("xor") {
            e = Expr::Binary(BinOp::Xor, Box::new(e), Box::new(self.or()?));
        }
        Ok(e)
    }

    fn or(&mut self) -> R<Expr> {
        let mut e = self.and()?;
        while self.eat_kw("or") {
            e = Expr::Binary(BinOp::Or, Box::new(e), Box::new(self.and()?));
        }
        Ok(e)
    }

    fn and(&mut self) -> R<Expr> {
        let mut e = self.not()?;
        while self.eat_kw("and") {
            e = Expr::Binary(BinOp::And, Box::new(e), Box::new(self.not()?));
        }
        Ok(e)
    }

    fn not(&mut self) -> R<Expr> {
        if self.eat_kw("not") {
            return Ok(Expr::Unary(UnOp::Not, Box::new(self.not()?)));
        }
        self.comparison()
    }

    fn comparison(&mut self) -> R<Expr> {
        let mut e = self.concat()?;
        loop {
            let op = match self.peek() {
                Tok::Op("=") => BinOp::Eq,
                Tok::Op("<>") => BinOp::Ne,
                Tok::Op("<") => BinOp::Lt,
                Tok::Op(">") => BinOp::Gt,
                Tok::Op("<=") => BinOp::Le,
                Tok::Op(">=") => BinOp::Ge,
                Tok::Ident(s) if s.eq_ignore_ascii_case("like") => BinOp::Like,
                Tok::Ident(s) if s.eq_ignore_ascii_case("is") => BinOp::Is,
                _ => break,
            };
            self.bump();
            e = Expr::Binary(op, Box::new(e), Box::new(self.concat()?));
        }
        Ok(e)
    }

    fn concat(&mut self) -> R<Expr> {
        let mut e = self.additive()?;
        while self.eat_op("&") {
            e = Expr::Binary(BinOp::Cat, Box::new(e), Box::new(self.additive()?));
        }
        Ok(e)
    }

    fn additive(&mut self) -> R<Expr> {
        let mut e = self.modulo()?;
        loop {
            let op = if self.op("+") {
                BinOp::Add
            } else if self.op("-") {
                BinOp::Sub
            } else {
                break;
            };
            self.bump();
            e = Expr::Binary(op, Box::new(e), Box::new(self.modulo()?));
        }
        Ok(e)
    }

    fn modulo(&mut self) -> R<Expr> {
        let mut e = self.intdiv()?;
        while self.eat_kw("mod") {
            e = Expr::Binary(BinOp::Mod, Box::new(e), Box::new(self.intdiv()?));
        }
        Ok(e)
    }

    fn intdiv(&mut self) -> R<Expr> {
        let mut e = self.multiplicative()?;
        while self.eat_op("\\") {
            e = Expr::Binary(BinOp::IntDiv, Box::new(e), Box::new(self.multiplicative()?));
        }
        Ok(e)
    }

    fn multiplicative(&mut self) -> R<Expr> {
        let mut e = self.unary()?;
        loop {
            let op = if self.op("*") {
                BinOp::Mul
            } else if self.op("/") {
                BinOp::Div
            } else {
                break;
            };
            self.bump();
            e = Expr::Binary(op, Box::new(e), Box::new(self.unary()?));
        }
        Ok(e)
    }

    fn unary(&mut self) -> R<Expr> {
        if self.eat_op("-") {
            return Ok(Expr::Unary(UnOp::Neg, Box::new(self.unary()?)));
        }
        if self.eat_op("+") {
            return self.unary();
        }
        // A file number written `#n` (`Input(n, #f)`, `EOF(#1)`): the number.
        if self.eat_op("#") {
            return self.unary();
        }
        self.power()
    }

    fn power(&mut self) -> R<Expr> {
        let mut e = self.postfix()?;
        while self.eat_op("^") {
            let rhs = if self.eat_op("-") {
                Expr::Unary(UnOp::Neg, Box::new(self.power_operand()?))
            } else {
                self.power_operand()?
            };
            e = Expr::Binary(BinOp::Pow, Box::new(e), Box::new(rhs));
        }
        Ok(e)
    }

    fn power_operand(&mut self) -> R<Expr> {
        if self.eat_op("-") {
            return Ok(Expr::Unary(UnOp::Neg, Box::new(self.power_operand()?)));
        }
        self.postfix()
    }

    pub fn postfix(&mut self) -> R<Expr> {
        let mut e = self.primary()?;
        loop {
            if self.op(".") && !self.tok().space_before {
                self.bump();
                let n = self.ident()?;
                e = Expr::Member(Box::new(e), n);
            } else if self.op("!") && !self.tok().space_before {
                self.bump();
                let n = self.ident()?;
                e = Expr::Bang(Box::new(e), n);
            } else if self.op("(") {
                self.bump();
                let args = self.paren_args()?;
                e = Expr::Call(Box::new(e), args);
            } else {
                break;
            }
        }
        Ok(e)
    }

    fn primary(&mut self) -> R<Expr> {
        let t = self.peek().clone();
        match t {
            Tok::Num(n) => {
                self.bump();
                Ok(Expr::Num(n))
            }
            Tok::Str(s) => {
                self.bump();
                Ok(Expr::Str(s))
            }
            Tok::Date(d) => {
                self.bump();
                Ok(Expr::Date(d))
            }
            Tok::Op("(") => {
                self.bump();
                let e = self.expr()?;
                self.expect_op(")")?;
                Ok(Expr::Paren(Box::new(e)))
            }
            Tok::Op(".") => {
                self.bump();
                let n = self.ident()?;
                Ok(Expr::WithMember(n))
            }
            Tok::Op("!") => {
                self.bump();
                let n = self.ident()?;
                Ok(Expr::Bang(Box::new(Expr::WithMember(String::new())), n))
            }
            Tok::Ident(s) => {
                let l = s.to_ascii_lowercase();
                match l.as_str() {
                    "true" => {
                        self.bump();
                        Ok(Expr::Bool(true))
                    }
                    "false" => {
                        self.bump();
                        Ok(Expr::Bool(false))
                    }
                    "nothing" => {
                        self.bump();
                        Ok(Expr::Nothing)
                    }
                    "empty" => {
                        self.bump();
                        Ok(Expr::Empty)
                    }
                    "null" => {
                        self.bump();
                        Ok(Expr::Null)
                    }
                    "me" => {
                        self.bump();
                        Ok(Expr::Me)
                    }
                    "new" => {
                        self.bump();
                        Ok(Expr::New(self.type_ref()?.name))
                    }
                    "typeof" => {
                        self.bump();
                        let e = self.postfix()?;
                        self.expect_kw("is")?;
                        Ok(Expr::TypeOfIs(Box::new(e), self.type_ref()?.name))
                    }
                    "addressof" => {
                        self.bump();
                        Ok(Expr::AddressOf(self.ident()?))
                    }
                    _ if RESERVED.contains(&l.as_str()) => self.err(format!("Unexpected {s}")),
                    _ => {
                        self.bump();
                        Ok(Expr::Ident(s))
                    }
                }
            }
            _ => self.err(format!("Expected an expression, found {t:?}")),
        }
    }
}

fn capital(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}
