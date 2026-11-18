use crate::span::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Float,
    Bool,
    Str,
    Char,
    U8,
    Void,
    Array(Box<Type>),
    Slice(Box<Type>),
    StrSlice,
    Named(String),
    Generic {
        name: String,
        type_args: Vec<Type>,
    },
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    Box(Box<Type>),
    Rc(Box<Type>),
    RefCell(Box<Type>),
    JoinHandle(Box<Type>),
    Sender(Box<Type>),
    Receiver(Box<Type>),
    Mutex(Box<Type>),
    TcpListener,
    TcpStream,
    Instant,
    Tuple(Vec<Type>),
    Ref {
        inner: Box<Type>,
        is_mut: bool,
    },
    RawPtr {
        inner: Box<Type>,
        is_mut: bool,
    },
    Fn {
        params: Vec<Type>,
        ret: Box<Type>,
    },
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Bool => write!(f, "bool"),
            Type::Str => write!(f, "str"),
            Type::Char => write!(f, "char"),
            Type::U8 => write!(f, "u8"),
            Type::Void => write!(f, "void"),
            Type::Array(inner) => write!(f, "[{}]", inner),
            Type::Slice(inner) => write!(f, "&[{}]", inner),
            Type::StrSlice => write!(f, "&str"),
            Type::Named(name) => write!(f, "{}", name),
            Type::Generic { name, type_args } => {
                let args_str = type_args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", ");
                write!(f, "{}<{}>", name, args_str)
            }
            Type::Option(inner) => write!(f, "Option<{}>", inner),
            Type::Result(ok, err) => write!(f, "Result<{}, {}>", ok, err),
            Type::Box(inner) => write!(f, "Box<{}>", inner),
            Type::Rc(inner) => write!(f, "Rc<{}>", inner),
            Type::RefCell(inner) => write!(f, "RefCell<{}>", inner),
            Type::JoinHandle(inner) => write!(f, "JoinHandle<{}>", inner),
            Type::Sender(inner) => write!(f, "Sender<{}>", inner),
            Type::Receiver(inner) => write!(f, "Receiver<{}>", inner),
            Type::Mutex(inner) => write!(f, "Mutex<{}>", inner),
            Type::TcpListener => write!(f, "TcpListener"),
            Type::TcpStream => write!(f, "TcpStream"),
            Type::Instant => write!(f, "Instant"),
            Type::Tuple(types) => {
                let inner = types.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", ");
                write!(f, "({})", inner)
            }
            Type::Fn { params, ret } => {
                let p_str = params.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", ");
                write!(f, "fn({}) -> {}", p_str, ret)
            }
            Type::Ref { inner, is_mut } => {
                if *is_mut {
                    write!(f, "&mut {}", inner)
                } else {
                    write!(f, "&{}", inner)
                }
            }
            Type::RawPtr { inner, is_mut } => {
                if *is_mut {
                    write!(f, "*mut {}", inner)
                } else {
                    write!(f, "*const {}", inner)
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FormatPart {
    Lit(String),
    Expr(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Int(i64, Span),
    Float(f64, Span),
    Bool(bool, Span),
    Str(String, Span),
    FormatString(Vec<FormatPart>, Span),
    Char(char, Span),
    U8(u8, Span),
    Ident(String, Span),
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },
    Unary {
        op: UnOp,
        expr: Box<Expr>,
        span: Span,
    },
    Assign {
        target: String,
        value: Box<Expr>,
        span: Span,
    },
    Call {
        callee: String,
        type_args: Vec<Type>,
        args: Vec<Expr>,
        span: Span,
    },
    If {
        condition: Box<Expr>,
        then_branch: Block,
        else_branch: Option<Block>,
        span: Span,
    },
    Array(Vec<Expr>, Span),
    Slice {
        target: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        span: Span,
    },
    Tuple(Vec<Expr>, Span),
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
    IndexAssign {
        target: Box<Expr>,
        index: Box<Expr>,
        value: Box<Expr>,
        span: Span,
    },
    TupleIndex {
        target: Box<Expr>,
        index: usize,
        span: Span,
    },
    TupleIndexAssign {
        target: Box<Expr>,
        index: usize,
        value: Box<Expr>,
        span: Span,
    },
    StructInit {
        name: String,
        type_args: Vec<Type>,
        fields: Vec<(String, Expr)>,
        span: Span,
    },
    FieldAccess {
        target: Box<Expr>,
        field: String,
        span: Span,
    },
    FieldAssign {
        target: Box<Expr>,
        field: String,
        value: Box<Expr>,
        span: Span,
    },
    MethodCall {
        target: Box<Expr>,
        method: String,
        type_args: Vec<Type>,
        args: Vec<Expr>,
        span: Span,
    },
    Path {
        base: String,
        member: String,
        span: Span,
    },
    Match {
        subject: Box<Expr>,
        arms: Vec<MatchArm>,
        span: Span,
    },
    Borrow {
        expr: Box<Expr>,
        is_mut: bool,
        span: Span,
    },
    Deref {
        expr: Box<Expr>,
        span: Span,
    },
    DerefAssign {
        target: Box<Expr>,
        value: Box<Expr>,
        span: Span,
    },
    UnsafeBlock(Block, Span),
    Try(Box<Expr>, Span),
    Closure {
        params: Vec<Param>,
        return_type: Option<Type>,
        body: Block,
        span: Span,
    },
    Block(Block),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Wildcard(Span),
    Int(i64, Span),
    Float(f64, Span),
    Bool(bool, Span),
    Str(String, Span),
    Char(char, Span),
    U8(u8, Span),
    Ident(String, Span),
    EnumVariant {
        enum_name: String,
        variant: String,
        span: Span,
    },
    Some(Box<Pattern>, Span),
    None(Span),
    Ok(Box<Pattern>, Span),
    Err(Box<Pattern>, Span),
    Tuple(Vec<Pattern>, Span),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expr,
    pub span: Span,
}

impl Expr {
    #[allow(dead_code)]
    pub fn span(&self) -> Span {
        match self {
            Expr::Int(_, s)
            | Expr::Float(_, s)
            | Expr::Bool(_, s)
            | Expr::Str(_, s)
            | Expr::FormatString(_, s)
            | Expr::Char(_, s)
            | Expr::U8(_, s)
            | Expr::Ident(_, s)
            | Expr::Array(_, s)
            | Expr::Slice { span: s, .. }
            | Expr::Tuple(_, s)
            | Expr::Index { span: s, .. }
            | Expr::IndexAssign { span: s, .. }
            | Expr::TupleIndex { span: s, .. }
            | Expr::TupleIndexAssign { span: s, .. }
            | Expr::StructInit { span: s, .. }
            | Expr::FieldAccess { span: s, .. }
            | Expr::FieldAssign { span: s, .. }
            | Expr::MethodCall { span: s, .. }
            | Expr::Path { span: s, .. }
            | Expr::Match { span: s, .. }
            | Expr::Binary { span: s, .. }
            | Expr::Unary { span: s, .. }
            | Expr::Assign { span: s, .. }
            | Expr::Call { span: s, .. }
            | Expr::Borrow { span: s, .. }
            | Expr::Deref { span: s, .. }
            | Expr::DerefAssign { span: s, .. }
            | Expr::Closure { span: s, .. }
            | Expr::UnsafeBlock(_, s)
            | Expr::Try(_, s)
            | Expr::If { span: s, .. } => *s,
            Expr::Block(b) => b.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let {
        name: String,
        is_mut: bool,
        ty: Option<Type>,
        init: Option<Expr>,
        span: Span,
    },
    LetTuple {
        elements: Vec<(String, bool)>,
        ty: Option<Type>,
        init: Option<Expr>,
        span: Span,
    },
    While {
        condition: Expr,
        body: Block,
        span: Span,
    },
    ForRange {
        var_name: String,
        start: Expr,
        end: Expr,
        body: Block,
        span: Span,
    },
    ForIn {
        var_name: String,
        iterable: Expr,
        body: Block,
        span: Span,
    },
    Break(Span),
    Continue(Span),
    Return(Option<Expr>, Span),
    Const(ConstDef),
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub trailing_expr: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Type,
    pub is_mut: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<String>,
}

impl GenericParam {
    #[allow(dead_code)]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            bounds: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub generic_params: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub return_type: Type,
    pub body: Block,
    pub span: Span,
    pub doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructDef {
    pub name: String,
    pub generic_params: Vec<GenericParam>,
    pub fields: Vec<(String, Type)>,
    pub span: Span,
    pub doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImplBlock {
    pub trait_name: Option<String>,
    pub target: String,
    pub generic_params: Vec<GenericParam>,
    pub methods: Vec<Function>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitMethod {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Type,
    pub default_body: Option<Block>,
    pub span: Span,
    pub doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitDef {
    pub name: String,
    pub generic_params: Vec<GenericParam>,
    pub methods: Vec<TraitMethod>,
    pub span: Span,
    pub doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<String>,
    pub span: Span,
    pub doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstDef {
    pub name: String,
    pub ty: Type,
    pub value: Expr,
    pub span: Span,
    pub doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExternFn {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExternBlock {
    pub abi: String,
    pub functions: Vec<ExternFn>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Import(String, Span),
    Extern(ExternBlock),
    Const(ConstDef),
    Function(Function),
    Struct(StructDef),
    Impl(ImplBlock),
    Enum(EnumDef),
    Trait(TraitDef),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<Item>,
}
