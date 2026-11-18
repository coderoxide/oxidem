use crate::span::Span;
use crate::token::{Token, TokenKind};

pub struct Lexer<'a> {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    _marker: std::marker::PhantomData<&'a str>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            _marker: std::marker::PhantomData,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.get(self.pos).copied();
        if let Some(c) = ch {
            self.pos += 1;
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        ch
    }

    fn current_span(&self) -> Span {
        Span::new(self.line, self.col)
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.peek() {
            let span = self.current_span();

            // Whitespace
            if ch.is_whitespace() {
                self.advance();
                continue;
            }

            // Comments & DocComments
            if ch == '/' && self.peek_next() == Some('/') {
                let third = self.chars.get(self.pos + 2).copied();
                let fourth = self.chars.get(self.pos + 3).copied();
                if third == Some('/') && fourth != Some('/') {
                    self.advance(); // /
                    self.advance(); // /
                    self.advance(); // /
                    let mut doc = String::new();
                    while let Some(c) = self.peek() {
                        if c == '\n' {
                            break;
                        }
                        doc.push(c);
                        self.advance();
                    }
                    let doc_text = if doc.starts_with(' ') {
                        doc[1..].to_string()
                    } else {
                        doc
                    };
                    tokens.push(Token::new(TokenKind::DocComment(doc_text), span));
                    continue;
                }

                while let Some(c) = self.peek() {
                    if c == '\n' {
                        break;
                    }
                    self.advance();
                }
                continue;
            }

            // Two-character operators
            if ch == '-' && self.peek_next() == Some('>') {
                self.advance();
                self.advance();
                tokens.push(Token::new(TokenKind::Arrow, span));
                continue;
            }
            if ch == '=' && self.peek_next() == Some('>') {
                self.advance();
                self.advance();
                tokens.push(Token::new(TokenKind::FatArrow, span));
                continue;
            }
            if ch == ':' && self.peek_next() == Some(':') {
                self.advance();
                self.advance();
                tokens.push(Token::new(TokenKind::ColonColon, span));
                continue;
            }
            if ch == '=' && self.peek_next() == Some('=') {
                self.advance();
                self.advance();
                tokens.push(Token::new(TokenKind::EqEq, span));
                continue;
            }
            if ch == '!' && self.peek_next() == Some('=') {
                self.advance();
                self.advance();
                tokens.push(Token::new(TokenKind::BangEq, span));
                continue;
            }
            if ch == '<' && self.peek_next() == Some('=') {
                self.advance();
                self.advance();
                tokens.push(Token::new(TokenKind::LtEq, span));
                continue;
            }
            if ch == '>' && self.peek_next() == Some('=') {
                self.advance();
                self.advance();
                tokens.push(Token::new(TokenKind::GtEq, span));
                continue;
            }
            if ch == '&' && self.peek_next() == Some('&') {
                self.advance();
                self.advance();
                tokens.push(Token::new(TokenKind::AndAnd, span));
                continue;
            }
            if ch == '|' && self.peek_next() == Some('|') {
                self.advance();
                self.advance();
                tokens.push(Token::new(TokenKind::OrOr, span));
                continue;
            }
            if ch == '|' {
                self.advance();
                tokens.push(Token::new(TokenKind::Pipe, span));
                continue;
            }
            if ch == '.' && self.peek_next() == Some('.') {
                self.advance();
                self.advance();
                tokens.push(Token::new(TokenKind::DotDot, span));
                continue;
            }

            // Byte literal b'x'
            if ch == 'b' && self.peek_next() == Some('\'') {
                self.advance(); // consume 'b'
                self.advance(); // consume opening quote
                let b_val = match self.advance() {
                    Some('\\') => match self.advance() {
                        Some('n') => b'\n',
                        Some('t') => b'\t',
                        Some('r') => b'\r',
                        Some('\\') => b'\\',
                        Some('\'') => b'\'',
                        Some('0') => b'\0',
                        Some(other) => other as u8,
                        None => return Err(format!("Hata [{}:{}]: Kapanmamış kaçış karakteri", span.line, span.col)),
                    },
                    Some('\'') => return Err(format!("Hata [{}:{}]: Boş bayt sabiti", span.line, span.col)),
                    Some(c) => c as u8,
                    None => return Err(format!("Hata [{}:{}]: Kapanmamış bayt sabiti (b')", span.line, span.col)),
                };
                if self.advance() != Some('\'') {
                    return Err(format!("Hata [{}:{}]: Kapanmamış veya geçersiz bayt sabiti (b')", span.line, span.col));
                }
                tokens.push(Token::new(TokenKind::U8Lit(b_val), span));
                continue;
            }

            // Formatted string literal f"..."
            if ch == 'f' && self.peek_next() == Some('"') {
                self.advance(); // consume 'f'
                self.advance(); // consume opening quote
                let mut s = String::new();
                let mut closed = false;
                while let Some(c) = self.advance() {
                    if c == '"' {
                        closed = true;
                        break;
                    } else if c == '\\' {
                        match self.advance() {
                            Some('n') => s.push('\n'),
                            Some('t') => s.push('\t'),
                            Some('r') => s.push('\r'),
                            Some('\\') => s.push('\\'),
                            Some('"') => s.push('"'),
                            Some(other) => {
                                s.push('\\');
                                s.push(other);
                            }
                            None => return Err(format!("Hata [{}:{}]: Kapanmamış kaçış karakteri", span.line, span.col)),
                        }
                    } else {
                        s.push(c);
                    }
                }
                if !closed {
                    return Err(format!("Hata [{}:{}]: Kapanmamış format metin dizgisi (f\")", span.line, span.col));
                }
                tokens.push(Token::new(TokenKind::FormatString(s), span));
                continue;
            }

            // Single-character tokens
            match ch {
                '+' => { self.advance(); tokens.push(Token::new(TokenKind::Plus, span)); }
                '-' => { self.advance(); tokens.push(Token::new(TokenKind::Minus, span)); }
                '*' => { self.advance(); tokens.push(Token::new(TokenKind::Star, span)); }
                '/' => { self.advance(); tokens.push(Token::new(TokenKind::Slash, span)); }
                '%' => { self.advance(); tokens.push(Token::new(TokenKind::Percent, span)); }
                '=' => { self.advance(); tokens.push(Token::new(TokenKind::Eq, span)); }
                '!' => { self.advance(); tokens.push(Token::new(TokenKind::Bang, span)); }
                '<' => { self.advance(); tokens.push(Token::new(TokenKind::Lt, span)); }
                '>' => { self.advance(); tokens.push(Token::new(TokenKind::Gt, span)); }
                '(' => { self.advance(); tokens.push(Token::new(TokenKind::LParen, span)); }
                ')' => { self.advance(); tokens.push(Token::new(TokenKind::RParen, span)); }
                '{' => { self.advance(); tokens.push(Token::new(TokenKind::LBrace, span)); }
                '}' => { self.advance(); tokens.push(Token::new(TokenKind::RBrace, span)); }
                '[' => { self.advance(); tokens.push(Token::new(TokenKind::LBracket, span)); }
                ']' => { self.advance(); tokens.push(Token::new(TokenKind::RBracket, span)); }
                ':' => { self.advance(); tokens.push(Token::new(TokenKind::Colon, span)); }
                ';' => { self.advance(); tokens.push(Token::new(TokenKind::Semicolon, span)); }
                ',' => { self.advance(); tokens.push(Token::new(TokenKind::Comma, span)); }
                '.' => { self.advance(); tokens.push(Token::new(TokenKind::Dot, span)); }
                '&' => { self.advance(); tokens.push(Token::new(TokenKind::Ampersand, span)); }
                '?' => { self.advance(); tokens.push(Token::new(TokenKind::Question, span)); }

                // String literal
                '"' => {
                    self.advance(); // consume opening quote
                    let mut s = String::new();
                    let mut closed = false;
                    while let Some(c) = self.advance() {
                        if c == '"' {
                            closed = true;
                            break;
                        } else if c == '\\' {
                            match self.advance() {
                                Some('n') => s.push('\n'),
                                Some('t') => s.push('\t'),
                                Some('r') => s.push('\r'),
                                Some('\\') => s.push('\\'),
                                Some('"') => s.push('"'),
                                Some(other) => {
                                    s.push('\\');
                                    s.push(other);
                                }
                                None => return Err(format!("Hata [{}:{}]: Kapanmamış kaçış karakteri", span.line, span.col)),
                            }
                        } else {
                            s.push(c);
                        }
                    }
                    if !closed {
                        return Err(format!("Hata [{}:{}]: Kapanmamış metin dizgisi (\")", span.line, span.col));
                    }
                    tokens.push(Token::new(TokenKind::StringLit(s), span));
                }

                // Character literal
                '\'' => {
                    self.advance(); // consume opening single quote
                    let ch = match self.advance() {
                        Some('\\') => match self.advance() {
                            Some('n') => '\n',
                            Some('t') => '\t',
                            Some('r') => '\r',
                            Some('\\') => '\\',
                            Some('\'') => '\'',
                            Some('0') => '\0',
                            Some(other) => other,
                            None => return Err(format!("Hata [{}:{}]: Kapanmamış kaçış karakteri", span.line, span.col)),
                        },
                        Some('\'') => return Err(format!("Hata [{}:{}]: Boş karakter sabiti", span.line, span.col)),
                        Some(c) => c,
                        None => return Err(format!("Hata [{}:{}]: Kapanmamış karakter sabiti (')", span.line, span.col)),
                    };
                    if self.advance() != Some('\'') {
                        return Err(format!("Hata [{}:{}]: Kapanmamış veya geçersiz karakter sabiti (')", span.line, span.col));
                    }
                    tokens.push(Token::new(TokenKind::CharLit(ch), span));
                }

                // Number literals
                '0'..='9' => {
                    let mut num_str = String::new();
                    let mut is_float = false;

                    while let Some(c) = self.peek() {
                        if c.is_ascii_digit() {
                            num_str.push(c);
                            self.advance();
                        } else if c == '.' && !is_float && self.peek_next().map_or(false, |next| next.is_ascii_digit()) {
                            is_float = true;
                            num_str.push(c);
                            self.advance();
                        } else {
                            break;
                        }
                    }

                    if is_float {
                        match num_str.parse::<f64>() {
                            Ok(val) => tokens.push(Token::new(TokenKind::FloatLit(val), span)),
                            Err(_) => return Err(format!("Hata [{}:{}]: Geçersiz ondalıklı sayı '{}'", span.line, span.col, num_str)),
                        }
                    } else {
                        match num_str.parse::<i64>() {
                            Ok(val) => tokens.push(Token::new(TokenKind::IntLit(val), span)),
                            Err(_) => return Err(format!("Hata [{}:{}]: Geçersiz tamsayı '{}'", span.line, span.col, num_str)),
                        }
                    }
                }

                // Identifiers and Keywords
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut ident = String::new();
                    while let Some(c) = self.peek() {
                        if c.is_alphanumeric() || c == '_' {
                            ident.push(c);
                            self.advance();
                        } else {
                            break;
                        }
                    }

                    let kind = match ident.as_str() {
                        "fn" => TokenKind::Fn,
                        "let" => TokenKind::Let,
                        "mut" => TokenKind::Mut,
                        "if" => TokenKind::If,
                        "else" => TokenKind::Else,
                        "while" => TokenKind::While,
                        "for" => TokenKind::For,
                        "in" => TokenKind::In,
                        "break" => TokenKind::Break,
                        "continue" => TokenKind::Continue,
                        "return" => TokenKind::Return,
                        "struct" => TokenKind::Struct,
                        "impl" => TokenKind::Impl,
                        "trait" => TokenKind::Trait,
                        "enum" => TokenKind::Enum,
                        "import" => TokenKind::Import,
                        "extern" => TokenKind::Extern,
                        "unsafe" => TokenKind::Unsafe,
                        "const" => TokenKind::Const,
                        "match" => TokenKind::Match,
                        "true" => TokenKind::BoolLit(true),
                        "false" => TokenKind::BoolLit(false),
                        "int" => TokenKind::TypeInt,
                        "float" => TokenKind::TypeFloat,
                        "bool" => TokenKind::TypeBool,
                        "str" => TokenKind::TypeStr,
                        "char" => TokenKind::TypeChar,
                        "u8" | "byte" => TokenKind::TypeU8,
                        "void" => TokenKind::TypeVoid,
                        _ => TokenKind::Ident(ident),
                    };
                    tokens.push(Token::new(kind, span));
                }

                unexpected => {
                    return Err(format!("Hata [{}:{}]: Beklenmeyen karakter '{}'", span.line, span.col, unexpected));
                }
            }
        }

        tokens.push(Token::new(TokenKind::Eof, self.current_span()));
        Ok(tokens)
    }
}
