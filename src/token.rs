use crate::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    IntLit(i64),
    FloatLit(f64),
    StringLit(String),
    FormatString(String),
    CharLit(char),
    U8Lit(u8),
    BoolLit(bool),
    Ident(String),
    DocComment(String),

    // Keywords
    Fn,
    Let,
    Mut,
    If,
    Else,
    While,
    Return,
    Struct,
    Impl,
    Trait,
    Enum,
    Match,
    For,
    In,
    Break,
    Continue,
    Import,
    Extern,
    Unsafe,
    Const,

    // Types
    TypeInt,
    TypeFloat,
    TypeBool,
    TypeStr,
    TypeChar,
    TypeU8,
    TypeVoid,

    // Symbols & Operators
    Plus,       // +
    Minus,      // -
    Star,       // *
    Slash,      // /
    Percent,    // %
    Eq,         // =
    EqEq,       // ==
    BangEq,     // !=
    Lt,         // <
    LtEq,       // <=
    Gt,         // >
    GtEq,       // >=
    AndAnd,     // &&
    Ampersand,  // &
    OrOr,       // ||
    Pipe,       // |
    Bang,       // !
    Question,   // ?

    Arrow,      // ->
    FatArrow,   // =>
    Colon,      // :
    ColonColon, // ::
    Semicolon,  // ;
    Comma,      // ,
    Dot,        // .
    DotDot,     // ..

    LParen,     // (
    RParen,     // )
    LBrace,     // {
    RBrace,     // }
    LBracket,   // [
    RBracket,   // ]

    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}
