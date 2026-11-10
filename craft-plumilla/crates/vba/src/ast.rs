//! The syntax tree of a VBA module.

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Num(f64),
    Str(String),
    Date(f64),
    Bool(bool),
    Empty,
    Null,
    Nothing,
    Me,
    Ident(String),
    /// `base.name`
    Member(Box<Expr>, String),
    /// `.name` inside a With block.
    WithMember(String),
    /// `base!name` (the default member with the name as a string).
    Bang(Box<Expr>, String),
    /// `callee(args)`: a call, an index or a default member.
    Call(Box<Expr>, Vec<Arg>),
    New(String),
    Unary(UnOp, Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    TypeOfIs(Box<Expr>, String),
    AddressOf(String),
    /// `(expr)`: passed by value.
    Paren(Box<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    IntDiv,
    Mod,
    Pow,
    Cat,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    Like,
    Is,
    And,
    Or,
    Xor,
    Eqv,
    Imp,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Arg {
    /// `name:=value`
    pub name: Option<String>,
    /// `None`: a missing argument (`f(a, , c)`).
    pub value: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeRef {
    /// As written, possibly qualified (`Scripting.Dictionary`).
    pub name: String,
}

impl TypeRef {
    /// The last part of the name, lower case.
    pub fn key(&self) -> String {
        self.name
            .rsplit('.')
            .next()
            .unwrap_or(&self.name)
            .to_lowercase()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bound {
    pub lower: Option<Expr>,
    pub upper: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarDecl {
    pub name: String,
    /// `Some(vec![])` for a dynamic array `x()`.
    pub bounds: Option<Vec<Bound>>,
    pub ty: Option<TypeRef>,
    pub is_new: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CaseItem {
    Value(Expr),
    Range(Expr, Expr),
    Is(BinOp, Expr),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExitKind {
    Sub,
    Function,
    Property,
    For,
    Do,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResumeKind {
    Here,
    Next,
    Label(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoopCond {
    PreWhile,
    PreUntil,
    PostWhile,
    PostUntil,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stmt {
    pub line: u32,
    pub kind: StmtKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind {
    Dim {
        vars: Vec<VarDecl>,
        is_static: bool,
    },
    Const(Vec<(String, Option<TypeRef>, Expr)>),
    Assign {
        target: Expr,
        value: Expr,
        set: bool,
    },
    /// A call statement: the expression and, when written without
    /// parentheses, its arguments.
    Call(Expr, Option<Vec<Arg>>),
    If {
        branches: Vec<(Expr, Vec<Stmt>)>,
        otherwise: Option<Vec<Stmt>>,
    },
    Select {
        test: Expr,
        cases: Vec<(Vec<CaseItem>, Vec<Stmt>)>,
        otherwise: Option<Vec<Stmt>>,
    },
    For {
        var: String,
        from: Expr,
        to: Expr,
        step: Option<Expr>,
        body: Vec<Stmt>,
    },
    ForEach {
        var: String,
        coll: Expr,
        body: Vec<Stmt>,
    },
    DoLoop {
        cond: Option<(LoopCond, Expr)>,
        body: Vec<Stmt>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    With {
        obj: Expr,
        body: Vec<Stmt>,
    },
    Exit(ExitKind),
    GoTo(String),
    Label(String),
    OnErrorResumeNext,
    OnErrorGoTo(String),
    OnErrorGoTo0,
    OnErrorGoToMinus1,
    Resume(ResumeKind),
    ReDim {
        preserve: bool,
        vars: Vec<(String, Vec<Bound>, Option<TypeRef>)>,
    },
    Erase(Vec<String>),
    MidAssign {
        target: Expr,
        start: Expr,
        len: Option<Expr>,
        value: Expr,
    },
    /// `Debug.Print` items and the separator after each (`;`, `,` or none).
    DebugPrint(Vec<(Expr, Option<char>)>),
    End,
    Nop,
    RaiseEvent(String, Vec<Arg>),
    Unload(Expr),
    Load(Expr),
    Unsupported(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcKind {
    Sub,
    Function,
    PropertyGet,
    PropertyLet,
    PropertySet,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub by_val: bool,
    pub optional: bool,
    pub default: Option<Expr>,
    pub param_array: bool,
    pub ty: Option<TypeRef>,
    pub is_array: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Proc {
    pub name: String,
    pub kind: ProcKind,
    pub is_public: bool,
    pub is_static: bool,
    pub params: Vec<Param>,
    pub ret: Option<TypeRef>,
    pub ret_is_array: bool,
    pub body: Vec<Stmt>,
    pub line: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UserType {
    pub name: String,
    pub is_public: bool,
    pub fields: Vec<VarDecl>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDecl {
    pub name: String,
    pub is_public: bool,
    pub members: Vec<(String, Option<Expr>)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModVar {
    pub decl: VarDecl,
    pub is_public: bool,
    pub line: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModConst {
    pub name: String,
    pub ty: Option<TypeRef>,
    pub value: Expr,
    pub is_public: bool,
    pub line: u32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ModuleAst {
    pub option_explicit: bool,
    pub option_base: i32,
    pub option_compare_text: bool,
    pub option_private: bool,
    pub vars: Vec<ModVar>,
    pub consts: Vec<ModConst>,
    pub types: Vec<UserType>,
    pub enums: Vec<EnumDecl>,
    /// Declared external (Windows API) procedures: name and public.
    pub declares: Vec<(String, bool)>,
    pub procs: Vec<Proc>,
}
